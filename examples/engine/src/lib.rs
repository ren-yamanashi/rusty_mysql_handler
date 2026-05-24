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

//! Minimal storage engine for testing `mysql-handler`. Returns three empty
//! rows from `rnd_next` and then signals end-of-file.

#![allow(unsafe_code)]

use std::ffi::CStr;

use mysql_handler::engine::{EngineError, EngineResult, StorageEngine};
use mysql_handler::ffi::register_engine_factory;
use mysql_handler::panic_guard::FfiBoundary;
use mysql_handler::sys::HA_BINLOG_STMT_CAPABLE;

// Plugin manifest. Lives in Rust (not in the C++ shim) because Rust cdylib's
// auto-generated linker version script wraps every non-`pub no_mangle` symbol
// in `local: *;`, which would otherwise hide the three data symbols mysqld
// looks up via dlsym at `INSTALL PLUGIN` time. Declaring them as
// `#[no_mangle] pub static` puts them on the `global:` side of that script
// on Linux ELF and on Mach-O alike, so no platform-specific export gymnastics
// are needed.
// `pub mod` is needed to satisfy `unreachable_pub` on the exported statics,
// which Rust's cdylib export path requires to be `pub`. The contents are
// MySQL ABI symbols, not a Rust public API, so doc / Debug lints are silenced
// at the module scope.
#[cfg(not(test))]
#[doc(hidden)]
#[allow(missing_docs, missing_debug_implementations)]
pub mod plugin_manifest {
    use core::ffi::{c_char, c_int, c_uint, c_ulong, c_void};
    use core::ptr;

    // Layout copy of MySQL's `struct st_mysql_plugin` from include/mysql/plugin.h.
    // The C struct is ABI-stable across MySQL 8.x. Status / system vars and the
    // reserved slot are kept as opaque `*mut c_void` because this example
    // engine does not expose any of those.
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

    // SAFETY: every field is either Copy POD or a pointer to a `'static` symbol
    // owned by the linker; the struct itself is never mutated.
    unsafe impl Sync for StMysqlPlugin {}

    // Storage-engine interface tag defined by the C++ shim (shim/plugin.cc).
    // We only need its address; opaque `[u8; 0]` keeps the layout intentionally
    // unspecified on the Rust side.
    unsafe extern "C" {
        static rusty_storage_engine: [u8; 0];
        fn rusty_init_func(p: *mut c_void) -> c_int;
        fn rusty_deinit_func(p: *mut c_void) -> c_int;
    }

    // Constants from include/mysql/plugin.h. Hardcoded rather than pulled in
    // via bindgen because exposing `st_mysql_plugin` transitively drags in
    // `SHOW_VAR` / `SYS_VAR` for no real benefit; these three integers are
    // ABI-stable on every MySQL 8.x release.
    const MYSQL_STORAGE_ENGINE_PLUGIN: c_int = 1;
    const PLUGIN_LICENSE_GPL: c_int = 1;
    const MYSQL_PLUGIN_INTERFACE_VERSION: c_int = 0x010B;

    #[unsafe(no_mangle)]
    pub static _mysql_plugin_interface_version_: c_int = MYSQL_PLUGIN_INTERFACE_VERSION;

    // Force the size into a const so `pub static` carries a plain c_int value
    // (not a const-eval expression). Otherwise the macOS Mach-O linker drops
    // the symbol from the exports trie under LTO; the link_section attribute
    // alone is not enough.
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    const SIZEOF_ST_PLUGIN: c_int = size_of::<StMysqlPlugin>() as c_int;

    // Force into __DATA on macOS: otherwise both 4-byte c_int statics end up
    // adjacent in __TEXT,__const and the Mach-O export trie collapses them —
    // only the first c_int gets a dlsym-visible entry. Linux ELF does not
    // suffer from this and keeps the default section.
    #[unsafe(no_mangle)]
    #[cfg_attr(target_os = "macos", unsafe(link_section = "__DATA,__data"))]
    pub static _mysql_sizeof_struct_st_plugin_: c_int = SIZEOF_ST_PLUGIN;

    #[unsafe(no_mangle)]
    pub static _mysql_plugin_declarations_: [StMysqlPlugin; 2] = [
        // `info` is `*mut c_void` per MySQL's ABI but `rusty_storage_engine`
        // is a Rust `static` (immutable). The C++ side only reads its address,
        // so the const-to-mut cast is sound; the underlying memory is never
        // written through this pointer.
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
        // mysql_declare_plugin_end sentinel: zeroed st_mysql_plugin terminator.
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
}

/// Trivial in-memory engine that yields a fixed number of empty rows.
#[derive(Debug)]
pub struct TrivialEngine {
    num_rows: u32,
    current_row: u32,
}

impl TrivialEngine {
    /// Construct a new engine that will yield three empty rows on the next scan.
    pub const fn new() -> Self {
        Self {
            num_rows: 3,
            current_row: 0,
        }
    }
}

impl Default for TrivialEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageEngine for TrivialEngine {
    fn table_type(&self) -> &'static CStr {
        c"RUSTY"
    }

    fn table_flags(&self) -> u64 {
        HA_BINLOG_STMT_CAPABLE as u64
    }

    fn index_flags(&self, _idx: u32, _part: u32, _all_parts: bool) -> u32 {
        0
    }

    fn create(&mut self, _name: &str) -> EngineResult {
        Ok(())
    }

    fn open(&mut self, _name: &str, _mode: i32) -> EngineResult {
        Ok(())
    }

    fn close(&mut self) -> EngineResult {
        Ok(())
    }

    fn rnd_init(&mut self, _scan: bool) -> EngineResult {
        self.current_row = 0;
        Ok(())
    }

    fn rnd_next(&mut self, _buf: &mut [u8]) -> EngineResult {
        if self.current_row >= self.num_rows {
            return Err(EngineError::EndOfFile);
        }
        self.current_row += 1;
        Ok(())
    }

    fn rnd_pos(&mut self, _buf: &mut [u8], _pos: &[u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    fn position(&mut self, _record: &[u8]) {}

    fn info(&mut self, _flag: u32) -> EngineResult {
        Ok(())
    }
}

/// Plugin entry point. The C++ shim calls this once at `INSTALL PLUGIN` so the
/// factory is wired up before any table is opened.
///
/// # Safety
/// Must be called once from `rusty_init_func` on the mysqld thread that runs
/// `INSTALL PLUGIN`. The body is panic-safe via [`FfiBoundary::run_void`] and
/// takes no inputs from the caller, so the soundness requirement reduces to
/// the FFI calling-convention invariants (same as every other `rust__*`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__plugin_init() {
    FfiBoundary::run_void(|| {
        register_engine_factory(|| Box::new(TrivialEngine::default()));
    });
}
