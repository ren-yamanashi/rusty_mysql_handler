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

//! Plugin manifest lives here (not in the C++ shim) because the Rust cdylib's
//! linker version script wraps every non-`pub no_mangle` symbol in `local: *;`,
//! which would hide the three data symbols mysqld dlsyms at `INSTALL PLUGIN`.

use core::ffi::{c_char, c_int, c_uint, c_ulong, c_void};
use core::ptr;

// Layout copy of `struct st_mysql_plugin` (include/mysql/plugin.h); stable
// across MySQL 8.x. Status / system vars stay opaque — unused here.
#[repr(C)]
pub struct StMysqlPlugin {
    pub type_: c_int,
    pub info: *mut c_void,
    pub name: *const c_char,
    pub author: *const c_char,
    pub descr: *const c_char,
    pub license: c_int,
    pub init: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
    pub check_uninstall: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
    pub deinit: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
    pub version: c_uint,
    pub status_vars: *mut c_void,
    pub system_vars: *mut c_void,
    pub reserved1: *mut c_void,
    pub flags: c_ulong,
}

// SAFETY: fields are Copy POD or pointers to `'static` linker-owned
// symbols; the struct is never mutated.
unsafe impl Sync for StMysqlPlugin {}

// Interface tag defined in shim/plugin.cc; opaque on the Rust side.
unsafe extern "C" {
    static rusty_storage_engine: [u8; 0];
    fn rusty_init_func(p: *mut c_void) -> c_int;
    fn rusty_deinit_func(p: *mut c_void) -> c_int;
}

// From include/mysql/plugin.h; hand-written to skip the SHOW_VAR / SYS_VAR
// transitive types bindgen would pull in.
const MYSQL_STORAGE_ENGINE_PLUGIN: c_int = 1;
const PLUGIN_LICENSE_GPL: c_int = 1;
const MYSQL_PLUGIN_INTERFACE_VERSION: c_int = 0x010B;

#[unsafe(no_mangle)]
pub static _mysql_plugin_interface_version_: c_int = MYSQL_PLUGIN_INTERFACE_VERSION;

// Routed through a `const` so the `pub static` carries a plain value;
// otherwise macOS LTO drops the symbol from the export trie.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
const SIZEOF_ST_PLUGIN: c_int = size_of::<StMysqlPlugin>() as c_int;

// Forced into __DATA on macOS so the Mach-O export trie keeps both 4-byte
// c_int statics distinct; Linux ELF does not need this.
#[unsafe(no_mangle)]
#[cfg_attr(target_os = "macos", unsafe(link_section = "__DATA,__data"))]
pub static _mysql_sizeof_struct_st_plugin_: c_int = SIZEOF_ST_PLUGIN;

#[unsafe(no_mangle)]
pub static _mysql_plugin_declarations_: [StMysqlPlugin; 2] = [
    // `info` is `*mut c_void` per ABI but the C++ side only reads the
    // address; the const-to-mut cast never writes through the pointer.
    StMysqlPlugin {
        type_: MYSQL_STORAGE_ENGINE_PLUGIN,
        info: &raw const rusty_storage_engine as *mut c_void,
        name: c"RUSTY".as_ptr(),
        author: c"ren-yamanashi".as_ptr(),
        descr: c"Rusty storage engine".as_ptr(),
        license: PLUGIN_LICENSE_GPL,
        init: Some(rusty_init_func),
        check_uninstall: None,
        deinit: Some(rusty_deinit_func),
        version: 0x0001,
        status_vars: ptr::null_mut(),
        system_vars: ptr::null_mut(),
        reserved1: ptr::null_mut(),
        flags: 0,
    },
    // `mysql_declare_plugin_end` sentinel: zeroed terminator
    StMysqlPlugin {
        type_: 0,
        info: ptr::null_mut(),
        name: ptr::null(),
        author: ptr::null(),
        descr: ptr::null(),
        license: 0,
        init: None,
        check_uninstall: None,
        deinit: None,
        version: 0,
        status_vars: ptr::null_mut(),
        system_vars: ptr::null_mut(),
        reserved1: ptr::null_mut(),
        flags: 0,
    },
];
