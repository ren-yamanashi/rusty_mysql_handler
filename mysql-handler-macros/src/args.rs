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

use syn::{
    Expr, LitStr, Token,
    parse::{Parse, ParseStream},
};

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
}

impl Parse for PluginArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name: Option<LitStr> = None;
        let mut description: Option<LitStr> = None;
        let mut version: Option<Expr> = None;
        let mut license: Option<Expr> = None;
        let mut author: Option<LitStr> = None;
        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "name" => set_once(&mut name, input.parse()?, &key)?,
                "description" => set_once(&mut description, input.parse()?, &key)?,
                "version" => set_once(&mut version, input.parse()?, &key)?,
                "license" => set_once(&mut license, input.parse()?, &key)?,
                "author" => set_once(&mut author, input.parse()?, &key)?,
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[plugin] argument `{other}` (expected one of: name, description, version, license, author)"
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
        })
    }
}

fn set_once<T>(slot: &mut Option<T>, value: T, key: &syn::Ident) -> syn::Result<()> {
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
    use super::{MAX_PLUGIN_NAME_LEN, validate_name};
    use proc_macro2::Span;
    use syn::LitStr;

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
}
