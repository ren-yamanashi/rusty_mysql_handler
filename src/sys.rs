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

//! Raw FFI bindings: MySQL handler constants and opaque C++ types.

#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
#![allow(missing_docs, unreachable_pub, missing_debug_implementations)]
#![allow(clippy::all, clippy::pedantic)]

include!(concat!(env!("OUT_DIR"), "/sys_bindings.rs"));

// Opaque C++ classes referenced by FFI signatures but not produced by bindgen.
#[repr(C)]
pub struct RustHandlerBase([u8; 0]);

#[repr(C)]
pub struct TABLE([u8; 0]);

#[repr(C)]
pub struct TABLE_SHARE([u8; 0]);

#[repr(C)]
pub struct THD([u8; 0]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ha_err_end_of_file_is_137() {
        assert_eq!(HA_ERR_END_OF_FILE, 137);
    }

    #[test]
    fn ha_binlog_stmt_capable_bit() {
        assert_eq!(HA_BINLOG_STMT_CAPABLE, 1i64 << 35);
    }
}
