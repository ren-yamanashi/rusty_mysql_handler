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

//! Unit tests for [`super`] — split off so `args.rs` stays under the
//! 250-line ceiling without exposing the parser internals as `pub`.

use proc_macro2::Span;
use syn::LitStr;
use syn::parse_quote;

use super::{MAX_PLUGIN_NAME_LEN, PluginArgs, validate_name};
use crate::capability::Capability;

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
fn capabilities_parse_indexed() {
    let args: PluginArgs = parse_quote! {
        name = "ex",
        description = "ex",
        version = 1u32,
        license = License::Gpl,
        author = "me",
        capabilities = [Indexed],
    };
    assert_eq!(args.capabilities, vec![Capability::Indexed]);
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
