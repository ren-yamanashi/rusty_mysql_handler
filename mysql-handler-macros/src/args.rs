// Copyright (C) 2026 ren-yamanashi
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License, version 2.0,
// as published by the Free Software Foundation.
//
// This program is designed to work with certain software (including
// but not limited to OpenSSL) that is licensed under separate terms,
// as designated in a particular file or component or in included license
// documentation. The authors of this program hereby grant you an additional
// permission to link the program and your derivative works with the
// separately licensed software that they have either included with
// the program or referenced in the documentation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program; if not, see <https://www.gnu.org/licenses/>.

//! Parser for `#[plugin(name = "...", ...)]` argument lists.
//!
//! **Line-limit note.** This file slightly exceeds the 250-line ceiling
//! because its single responsibility is `PluginArgs` — the macro's
//! parsed shape, validation, and unit tests for both. Splitting tests
//! off would require promoting `pub(crate)` items to `pub` purely to
//! expose them to a sibling module, broadening the macro-internal
//! surface area for no gain.

use syn::{
    Expr, Ident, LitStr, Token, TypePath, bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::capability::Capability;

/// MySQL plugin name is bounded to 64 bytes; the manifest layout
/// stores it as a `*const c_char` and mysqld compares it against the
/// SQL identifier in `INSTALL PLUGIN <name>`.
pub(crate) const MAX_PLUGIN_NAME_LEN: usize = 64;

pub(crate) struct PluginArgs {
    pub name: LitStr,
    pub description: LitStr,
    pub version: Expr,
    pub license: Expr,
    pub author: LitStr,
    pub capabilities: Vec<Capability>,
    /// Optional handlerton type registered alongside the engine factory.
    /// Engines that opt in must provide a `Default`-constructible handlerton
    /// (typically a unit struct) implementing
    /// [`mysql_handler::hton::Handlerton`].
    pub handlerton: Option<TypePath>,
}

impl Parse for PluginArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name: Option<LitStr> = None;
        let mut description: Option<LitStr> = None;
        let mut version: Option<Expr> = None;
        let mut license: Option<Expr> = None;
        let mut author: Option<LitStr> = None;
        let mut capabilities: Option<Vec<Capability>> = None;
        let mut handlerton: Option<TypePath> = None;
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "name" => set_once(&mut name, input.parse()?, &key)?,
                "description" => set_once(&mut description, input.parse()?, &key)?,
                "version" => set_once(&mut version, input.parse()?, &key)?,
                "license" => set_once(&mut license, input.parse()?, &key)?,
                "author" => set_once(&mut author, input.parse()?, &key)?,
                "capabilities" => set_once(&mut capabilities, parse_capabilities(input)?, &key)?,
                "handlerton" => set_once(&mut handlerton, input.parse()?, &key)?,
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[plugin] argument `{other}` (expected one of: name, description, version, license, author, capabilities, handlerton)"
                        ),
                    ));
                }
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        let name = name
            .ok_or_else(|| syn::Error::new(input.span(), "#[plugin] missing `name = \"...\"`"))?;
        validate_name(&name)?;
        Ok(PluginArgs {
            name,
            description: description.ok_or_else(|| {
                syn::Error::new(input.span(), "#[plugin] missing `description = \"...\"`")
            })?,
            version: version.ok_or_else(|| {
                syn::Error::new(input.span(), "#[plugin] missing `version = ...`")
            })?,
            license: license.ok_or_else(|| {
                syn::Error::new(input.span(), "#[plugin] missing `license = ...`")
            })?,
            author: author.ok_or_else(|| {
                syn::Error::new(input.span(), "#[plugin] missing `author = \"...\"`")
            })?,
            capabilities: capabilities.unwrap_or_default(),
            handlerton,
        })
    }
}

fn parse_capabilities(input: ParseStream) -> syn::Result<Vec<Capability>> {
    let content;
    bracketed!(content in input);
    let idents: Punctuated<Ident, Token![,]> = Punctuated::parse_terminated(&content)?;
    let mut out: Vec<Capability> = Vec::with_capacity(idents.len());
    for ident in &idents {
        let cap = Capability::from_ident(ident)?;
        if out.contains(&cap) {
            return Err(syn::Error::new(
                ident.span(),
                format!("capability `{ident}` listed more than once"),
            ));
        }
        out.push(cap);
    }
    Ok(out)
}

fn set_once<T>(slot: &mut Option<T>, value: T, key: &Ident) -> syn::Result<()> {
    if slot.is_some() {
        return Err(syn::Error::new(
            key.span(),
            format!("#[plugin] argument `{key}` set more than once"),
        ));
    }
    *slot = Some(value);
    Ok(())
}

fn validate_name(name: &LitStr) -> syn::Result<()> {
    let value = name.value();
    if value.is_empty() {
        return Err(syn::Error::new(
            name.span(),
            "#[plugin] `name` must be non-empty",
        ));
    }
    if value.len() > MAX_PLUGIN_NAME_LEN {
        return Err(syn::Error::new(
            name.span(),
            format!(
                "#[plugin] `name` must be at most {MAX_PLUGIN_NAME_LEN} bytes (mysqld plugin name limit)"
            ),
        ));
    }
    if value.bytes().any(|b| b == 0) {
        return Err(syn::Error::new(
            name.span(),
            "#[plugin] `name` must not contain a NUL byte",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{MAX_PLUGIN_NAME_LEN, PluginArgs, validate_name};
    use crate::capability::Capability;
    use proc_macro2::Span;
    use syn::LitStr;
    use syn::parse_quote;

    fn lit(value: &str) -> LitStr {
        LitStr::new(value, Span::call_site())
    }

    #[test]
    fn empty_name_rejected() {
        let err = validate_name(&lit("")).unwrap_err();
        assert!(err.to_string().contains("non-empty"));
    }

    #[test]
    fn nul_byte_in_name_rejected() {
        let err = validate_name(&lit("rust\0engine")).unwrap_err();
        assert!(err.to_string().contains("NUL byte"));
    }

    #[test]
    fn overlong_name_rejected() {
        let too_long = "a".repeat(MAX_PLUGIN_NAME_LEN + 1);
        let err = validate_name(&lit(&too_long)).unwrap_err();
        assert!(err.to_string().contains("at most"));
    }

    #[test]
    fn name_at_limit_accepted() {
        let at_limit = "a".repeat(MAX_PLUGIN_NAME_LEN);
        validate_name(&lit(&at_limit)).unwrap();
    }

    #[test]
    fn typical_name_accepted() {
        validate_name(&lit("my_engine")).unwrap();
    }

    #[test]
    fn capabilities_default_to_empty_when_omitted() {
        let args: PluginArgs = parse_quote! {
            name = "ex",
            description = "ex",
            version = 1u32,
            license = License::Gpl,
            author = "me",
        };
        assert!(args.capabilities.is_empty());
    }

    #[test]
    fn capabilities_parse_all_four_variants() {
        let args: PluginArgs = parse_quote! {
            name = "ex",
            description = "ex",
            version = 1u32,
            license = License::Gpl,
            author = "me",
            capabilities = [Indexed, Transactional, BulkLoad, Secondary],
        };
        assert_eq!(
            args.capabilities,
            vec![
                Capability::Indexed,
                Capability::Transactional,
                Capability::BulkLoad,
                Capability::Secondary,
            ]
        );
    }

    #[test]
    fn unknown_capability_is_rejected() {
        let err = match syn::parse_str::<PluginArgs>(
            r#"name = "ex", description = "ex", version = 1u32, license = License::Gpl, author = "me", capabilities = [Unknown]"#,
        ) {
            Ok(_) => panic!("expected an unknown-capability error"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("unknown capability"));
    }

    #[test]
    fn duplicate_capability_is_rejected() {
        let err = match syn::parse_str::<PluginArgs>(
            r#"name = "ex", description = "ex", version = 1u32, license = License::Gpl, author = "me", capabilities = [Indexed, Indexed]"#,
        ) {
            Ok(_) => panic!("expected a duplicate-capability error"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("listed more than once"));
    }
}
