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

//! Top-level orchestrator for `#[plugin(...)]` expansion. Each emitted
//! fragment lives in its own submodule so this file just composes them.

use std::ffi::CString;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ItemStruct, LitCStr, LitStr};

use crate::args::PluginArgs;

mod manifest;
mod plugin_init;

pub(crate) fn plugin(args: &PluginArgs, target: &ItemStruct) -> syn::Result<TokenStream2> {
    let ty = &target.ident;
    let name_lit = cstr_lit(&args.name)?;
    let descr_lit = cstr_lit(&args.description)?;
    let author_lit = cstr_lit(&args.author)?;
    let manifest_module = manifest::manifest_module();
    let manifest_statics = manifest::manifest_statics(
        &name_lit,
        &descr_lit,
        &author_lit,
        &args.version,
        &args.license,
    );
    let plugin_init = plugin_init::plugin_init(ty, args.handlerton.as_ref());

    Ok(quote! {
        #target
        #manifest_module
        #manifest_statics
        #plugin_init
    })
}

fn cstr_lit(lit: &LitStr) -> syn::Result<LitCStr> {
    let value = lit.value();
    let cstring = CString::new(value).map_err(|_| {
        syn::Error::new(
            lit.span(),
            "#[plugin] string literal must not contain a NUL byte",
        )
    })?;
    Ok(LitCStr::new(&cstring, lit.span()))
}
