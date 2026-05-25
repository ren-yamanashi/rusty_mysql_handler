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

//! `rust__handler__*` callbacks for table-lifecycle methods (handler.h #4–#11).
//! Shares the FFI safety contract documented at [`crate::ffi_handler`].

#![allow(unsafe_code)]

use crate::ffi::{EngineContext, FfiPtr};
use crate::panic_guard::FfiBoundary;
use crate::sys;

/// Drop a table by name; `table_def` may be null for temporary tables
///
/// # Safety
/// `ctx` non-null; `name` covers `name_len` readable bytes; `table_def` is
/// null or valid for read for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__delete_table(
    ctx: *mut EngineContext,
    name: *const u8,
    name_len: usize,
    table_def: *const sys::DdTable,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees name covers name_len readable bytes.
        let name = unsafe { FfiPtr::bytes_to_str(name, name_len) }?;
        // SAFETY: table_def is null or valid for read for the call's duration.
        let table_def = unsafe { table_def.as_ref() };
        engine.delete_table(name, table_def)
    })
}

/// Rename a table from `from` to `to`
///
/// # Safety
/// `ctx` non-null; `from` / `to` cover their declared lengths; `from_def` /
/// `to_def` are null or valid for read for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__rename_table(
    ctx: *mut EngineContext,
    from: *const u8,
    from_len: usize,
    to: *const u8,
    to_len: usize,
    from_def: *const sys::DdTable,
    to_def: *const sys::DdTable,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees from covers from_len readable bytes.
        let from = unsafe { FfiPtr::bytes_to_str(from, from_len) }?;
        // SAFETY: caller guarantees to covers to_len readable bytes.
        let to = unsafe { FfiPtr::bytes_to_str(to, to_len) }?;
        // SAFETY: from_def is null or valid for read for the call's duration.
        let from_def = unsafe { from_def.as_ref() };
        // SAFETY: to_def is null or valid for read for the call's duration.
        let to_def = unsafe { to_def.as_ref() };
        engine.rename_table(from, to, from_def, to_def)
    })
}

/// Notify the engine that the table is being dropped (void in C++)
///
/// # Safety
/// `ctx` non-null; `name` covers `name_len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__drop_table(
    ctx: *mut EngineContext,
    name: *const u8,
    name_len: usize,
) {
    FfiBoundary::run_void(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees name covers name_len readable bytes.
        match unsafe { FfiPtr::bytes_to_str(name, name_len) } {
            Ok(name) => engine.drop_table(name),
            Err(_) => tracing::warn!("drop_table: invalid utf-8 in table name"),
        }
    });
}

/// Truncate (empty) the table
///
/// # Safety
/// `ctx` non-null; `table_def` is null or valid for read for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__truncate(
    ctx: *mut EngineContext,
    table_def: *const sys::DdTable,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: table_def is null or valid for read for the call's duration.
        let table_def = unsafe { table_def.as_ref() };
        engine.truncate(table_def)
    })
}

/// Notify the engine that MySQL reassigned the `TABLE` and `TABLE_SHARE`
/// pointers. The callback receives the new pointers directly as arguments;
/// the shim additionally invokes `handler::change_table_ptr` first so that
/// any subsequent virtual call observing `handler::table` / `table_share`
/// sees the updated state.
///
/// # Safety
/// `ctx` non-null; `table` / `share` are null or valid for read for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__change_table_ptr(
    ctx: *mut EngineContext,
    table: *const sys::TABLE,
    share: *const sys::TABLE_SHARE,
) {
    FfiBoundary::run_void(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: table is null or valid for read for the call's duration.
        let table = unsafe { table.as_ref() };
        // SAFETY: share is null or valid for read for the call's duration.
        let share = unsafe { share.as_ref() };
        engine.change_table_ptr(table, share);
    });
}

/// Populate engine-private metadata in `dd_table`; returns `true` to signal
/// that private data was written.
///
/// # Safety
/// `ctx` non-null; `dd_table` is null or valid for read for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__get_se_private_data(
    ctx: *mut EngineContext,
    dd_table: *const sys::DdTable,
    reset: bool,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: dd_table is null or valid for read for the call's duration.
        let dd_table = unsafe { dd_table.as_ref() };
        engine.get_se_private_data(dd_table, reset)
    })
}

/// Inject implicit columns / keys required by the engine into `table_obj`
///
/// # Safety
/// `ctx` non-null; every other pointer is null or valid for read for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__get_extra_columns_and_keys(
    ctx: *mut EngineContext,
    create_info: *const sys::HA_CREATE_INFO,
    create_list: *const sys::ListCreateField,
    key_info: *const sys::KEY,
    key_count: u32,
    table_obj: *const sys::DdTable,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: create_info is null or valid for read for the call's duration.
        let create_info = unsafe { create_info.as_ref() };
        // SAFETY: create_list is null or valid for read for the call's duration.
        let create_list = unsafe { create_list.as_ref() };
        // SAFETY: key_info is null or valid for read for the call's duration.
        let key_info = unsafe { key_info.as_ref() };
        // SAFETY: table_obj is null or valid for read for the call's duration.
        let table_obj = unsafe { table_obj.as_ref() };
        engine.get_extra_columns_and_keys(create_info, create_list, key_info, key_count, table_obj)
    })
}

/// Adjust an old-format DD entry during a server upgrade. Returns `true` on
/// failure to match the C++ bool convention; the panic-safe default is also
/// `true` so a panicking engine forces an upgrade abort.
///
/// # Safety
/// `ctx` non-null; name buffers cover their declared lengths; `thd` /
/// `dd_table` are null or valid for read for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__upgrade_table(
    ctx: *mut EngineContext,
    thd: *const sys::THD,
    dbname: *const u8,
    dbname_len: usize,
    table_name: *const u8,
    table_name_len: usize,
    dd_table: *const sys::DdTable,
) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: thd is null or valid for read for the call's duration.
        let thd = unsafe { thd.as_ref() };
        // SAFETY: dd_table is null or valid for read for the call's duration.
        let dd_table = unsafe { dd_table.as_ref() };
        // SAFETY: caller guarantees dbname covers dbname_len readable bytes.
        let Ok(dbname) = (unsafe { FfiPtr::bytes_to_str(dbname, dbname_len) }) else {
            tracing::warn!("upgrade_table: invalid utf-8 in dbname");
            return true;
        };
        // SAFETY: caller guarantees table_name covers table_name_len readable bytes.
        let Ok(table_name) = (unsafe { FfiPtr::bytes_to_str(table_name, table_name_len) }) else {
            tracing::warn!("upgrade_table: invalid utf-8 in table name");
            return true;
        };
        engine.upgrade_table(thd, dbname, table_name, dd_table)
    })
}
