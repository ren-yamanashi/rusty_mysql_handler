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

//! Manifest module and `pub static` triple emitted by `#[plugin]`.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Expr, LitCStr};

pub(super) fn manifest_module() -> TokenStream2 {
    quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, missing_docs, missing_debug_implementations)]
        pub mod __mysql_handler_plugin {
            use ::core::ffi::{c_char, c_int, c_uint, c_ulong, c_void};

            #[repr(C)]
            pub struct StMysqlPlugin {
                pub type_: c_int,
                pub info: *mut c_void,
                pub name: *const c_char,
                pub author: *const c_char,
                pub descr: *const c_char,
                pub license: c_int,
                pub init: ::core::option::Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
                pub check_uninstall:
                    ::core::option::Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
                pub deinit: ::core::option::Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
                pub version: c_uint,
                pub status_vars: *mut c_void,
                pub system_vars: *mut c_void,
                pub reserved1: *mut c_void,
                pub flags: c_ulong,
            }

            // SAFETY: fields are Copy POD or pointers to 'static
            // linker-owned symbols; the struct is never mutated.
            unsafe impl ::core::marker::Sync for StMysqlPlugin {}

            unsafe extern "C" {
                pub static rusty_storage_engine: [u8; 0];
                pub fn rusty_init_func(p: *mut c_void) -> c_int;
                pub fn rusty_deinit_func(p: *mut c_void) -> c_int;
            }

            pub const STORAGE_ENGINE_TYPE: c_int = 1;
            pub const INTERFACE_VERSION: c_int = 0x010B;
        }
    }
}

pub(super) fn manifest_statics(
    name_lit: &LitCStr,
    descr_lit: &LitCStr,
    author_lit: &LitCStr,
    version: &Expr,
    license: &Expr,
) -> TokenStream2 {
    quote! {
        #[unsafe(no_mangle)]
        #[doc(hidden)]
        pub static _mysql_plugin_interface_version_: ::core::ffi::c_int =
            self::__mysql_handler_plugin::INTERFACE_VERSION;

        // Routed through a `const` so the `pub static` carries a plain value;
        // otherwise macOS LTO drops the symbol from the export trie.
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        const __MYSQL_HANDLER_SIZEOF_ST_PLUGIN: ::core::ffi::c_int =
            ::core::mem::size_of::<self::__mysql_handler_plugin::StMysqlPlugin>()
                as ::core::ffi::c_int;

        // Forced into __DATA on macOS so the Mach-O export trie keeps both
        // 4-byte c_int statics distinct; Linux ELF does not need this.
        #[unsafe(no_mangle)]
        #[doc(hidden)]
        #[cfg_attr(target_os = "macos", unsafe(link_section = "__DATA,__data"))]
        pub static _mysql_sizeof_struct_st_plugin_: ::core::ffi::c_int =
            __MYSQL_HANDLER_SIZEOF_ST_PLUGIN;

        #[unsafe(no_mangle)]
        #[doc(hidden)]
        pub static _mysql_plugin_declarations_:
            [self::__mysql_handler_plugin::StMysqlPlugin; 2] = [
            self::__mysql_handler_plugin::StMysqlPlugin {
                type_: self::__mysql_handler_plugin::STORAGE_ENGINE_TYPE,
                info: &raw const self::__mysql_handler_plugin::rusty_storage_engine
                    as *mut ::core::ffi::c_void,
                name: #name_lit.as_ptr(),
                author: #author_lit.as_ptr(),
                descr: #descr_lit.as_ptr(),
                license: (#license).code(),
                init: ::core::option::Option::Some(
                    self::__mysql_handler_plugin::rusty_init_func,
                ),
                check_uninstall: ::core::option::Option::None,
                deinit: ::core::option::Option::Some(
                    self::__mysql_handler_plugin::rusty_deinit_func,
                ),
                version: #version,
                status_vars: ::core::ptr::null_mut(),
                system_vars: ::core::ptr::null_mut(),
                reserved1: ::core::ptr::null_mut(),
                flags: 0,
            },
            self::__mysql_handler_plugin::StMysqlPlugin {
                type_: 0,
                info: ::core::ptr::null_mut(),
                name: ::core::ptr::null(),
                author: ::core::ptr::null(),
                descr: ::core::ptr::null(),
                license: 0,
                init: ::core::option::Option::None,
                check_uninstall: ::core::option::Option::None,
                deinit: ::core::option::Option::None,
                version: 0,
                status_vars: ::core::ptr::null_mut(),
                system_vars: ::core::ptr::null_mut(),
                reserved1: ::core::ptr::null_mut(),
                flags: 0,
            },
        ];
    }
}
