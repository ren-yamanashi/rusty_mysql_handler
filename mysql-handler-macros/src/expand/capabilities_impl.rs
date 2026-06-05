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

//! `EngineCapabilities` impl emitted from the `capabilities = [...]` list.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::capability::Capability;

pub(super) fn capabilities_impl(ty: &syn::Ident, caps: &[Capability]) -> TokenStream2 {
    let mut overrides = TokenStream2::new();
    for cap in caps {
        overrides.extend(capability_override(*cap));
    }
    quote! {
        impl ::mysql_handler::engine::EngineCapabilities for #ty {
            #overrides
        }
    }
}

fn capability_override(cap: Capability) -> TokenStream2 {
    match cap {
        Capability::Indexed => quote! {
            fn as_indexed(
                &mut self,
            ) -> ::core::option::Option<&mut dyn ::mysql_handler::engine::IndexedEngine> {
                ::core::option::Option::Some(self)
            }
        },
    }
}
