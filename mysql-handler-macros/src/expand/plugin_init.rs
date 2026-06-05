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

//! `rust__plugin_init` entry point emitted by `#[plugin]`.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::TypePath;

pub(super) fn plugin_init(ty: &syn::Ident, handlerton: Option<&TypePath>) -> TokenStream2 {
    let handlerton_init = match handlerton {
        Some(path) => quote! {
            ::mysql_handler::runtime::register_handlerton(::std::boxed::Box::new(
                <#path as ::core::default::Default>::default(),
            ));
        },
        None => quote! {},
    };
    quote! {
        /// Plugin entry point; the shim calls this once at `INSTALL PLUGIN`.
        ///
        /// # Safety
        /// Called once on the mysqld thread running `INSTALL PLUGIN`;
        /// panic-safe via the `FfiBoundary` wrapper.
        #[unsafe(no_mangle)]
        #[doc(hidden)]
        pub unsafe extern "C" fn rust__plugin_init() {
            ::mysql_handler::panic_guard::FfiBoundary::run_void(|| {
                ::mysql_handler::runtime::register_engine_factory(|| {
                    let engine: ::std::boxed::Box<
                        dyn ::mysql_handler::engine::EngineCapabilities,
                    > = ::std::boxed::Box::new(
                        <#ty as ::core::default::Default>::default(),
                    );
                    engine
                });
                #handlerton_init
            });
        }
    }
}
