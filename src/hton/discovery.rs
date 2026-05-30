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

//! `rust__hton__*` table-discovery callbacks. The shim wires `discover`,
//! `find_files`, `table_exists_in_engine`, `is_supported_system_table`
//! unconditionally on a registered handlerton. `discover` cannot produce its
//! `frmblob`/`frmlen` output through the opaque pass-through yet; the trait
//! default reports "no such table" and the shim leaves the blob empty.

#![allow(unsafe_code)]

use crate::engine::EngineError;
use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::runtime::FfiPtr;
use crate::sys;

/// Recover the table's frm description. Today's binding keeps the output blob
/// empty and lets the return code carry the "found / not found" decision; an
/// engine override returning `Ok(())` reports "found" without populating data.
///
/// # Safety
/// `thd` is null or a valid `THD` pointer for the call. `db` covers `db_len`
/// bytes and `name` covers `name_len` bytes, both readable for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__discover(
    thd: *const sys::THD,
    db: *const u8,
    db_len: usize,
    name: *const u8,
    name_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: db covers db_len readable bytes for this call.
        let db_str = match unsafe { FfiPtr::bytes_to_str(db, db_len) } {
            Ok(s) => s,
            Err(e) => return Err(e),
        };
        // SAFETY: name covers name_len readable bytes for this call.
        let name_str = match unsafe { FfiPtr::bytes_to_str(name, name_len) } {
            Ok(s) => s,
            Err(e) => return Err(e),
        };
        // SAFETY: thd is null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.discover(thd_ref, db_str, name_str),
            None => Err(EngineError::Unsupported),
        }
    })
}

/// Enumerate engine-known tables under `db`/`path`. The `List<LEX_STRING>*`
/// output cannot be populated through the opaque pass-through; the engine
/// returns `Ok(())` with no entries (the shim leaves the list untouched).
///
/// # Safety
/// `thd` is null or a valid `THD` pointer; `db` / `path` cover their lengths.
/// `wild` is null or covers `wild_len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__find_files(
    thd: *const sys::THD,
    db: *const u8,
    db_len: usize,
    path: *const u8,
    path_len: usize,
    wild: *const u8,
    wild_len: usize,
    dir: bool,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: db covers db_len readable bytes for this call.
        let db_str = match unsafe { FfiPtr::bytes_to_str(db, db_len) } {
            Ok(s) => s,
            Err(e) => return Err(e),
        };
        // SAFETY: path covers path_len readable bytes for this call.
        let path_str = match unsafe { FfiPtr::bytes_to_str(path, path_len) } {
            Ok(s) => s,
            Err(e) => return Err(e),
        };
        let wild_str = if wild.is_null() {
            None
        } else {
            // SAFETY: wild is non-null and covers wild_len readable bytes here.
            match unsafe { FfiPtr::bytes_to_str(wild, wild_len) } {
                Ok(s) => Some(s),
                Err(e) => return Err(e),
            }
        };
        // SAFETY: thd is null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.find_files(thd_ref, db_str, path_str, wild_str, dir),
            None => Ok(()),
        }
    })
}

/// `table_exists_in_engine`: the C++ side maps the returned bool to either
/// `HA_ERR_TABLE_EXIST` (true) or `HA_ERR_NO_SUCH_TABLE` (false), so the Rust
/// boolean is the engine's answer to "does this table live here?".
///
/// # Safety
/// `thd` is null or a valid `THD` pointer; `db` / `name` cover their lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__table_exists_in_engine(
    thd: *const sys::THD,
    db: *const u8,
    db_len: usize,
    name: *const u8,
    name_len: usize,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: db covers db_len readable bytes for this call.
        let db_str = match unsafe { FfiPtr::bytes_to_str(db, db_len) } {
            Ok(s) => s,
            Err(_) => return false,
        };
        // SAFETY: name covers name_len readable bytes for this call.
        let name_str = match unsafe { FfiPtr::bytes_to_str(name, name_len) } {
            Ok(s) => s,
            Err(_) => return false,
        };
        // SAFETY: thd is null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.table_exists_in_engine(thd_ref, db_str, name_str),
            None => false,
        }
    })
}

/// `is_supported_system_table`: whether the engine claims `db.table_name` as
/// one of its system tables.
///
/// # Safety
/// `db` / `name` cover their stated lengths readable for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_supported_system_table(
    db: *const u8,
    db_len: usize,
    name: *const u8,
    name_len: usize,
    is_sql_layer_system_table: bool,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: db covers db_len readable bytes for this call.
        let db_str = match unsafe { FfiPtr::bytes_to_str(db, db_len) } {
            Ok(s) => s,
            Err(_) => return false,
        };
        // SAFETY: name covers name_len readable bytes for this call.
        let name_str = match unsafe { FfiPtr::bytes_to_str(name, name_len) } {
            Ok(s) => s,
            Err(_) => return false,
        };
        match runtime::handlerton() {
            Some(h) => h.is_supported_system_table(db_str, name_str, is_sql_layer_system_table),
            None => false,
        }
    })
}
