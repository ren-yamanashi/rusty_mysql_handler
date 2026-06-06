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

//! `rust__handler__*` callbacks for bulk-load and secondary-engine-load methods
//! (handler.h #47–#53). Shares the FFI safety contract documented at
//! [`crate::handler`].

#![allow(unsafe_code)]

use core::ffi::c_void;

use crate::engine::EngineError;
use crate::panic_guard::FfiBoundary;
use crate::runtime::{EngineContext, FfiPtr};
use crate::sys;

/// Report whether the table is ready for bulk load
///
/// # Safety
/// `ctx` non-null; `thd` null-or-valid for the call.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__bulk_load_check(
    ctx: *mut EngineContext,
    thd: *const sys::THD,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: thd is null or valid for read for the call's duration.
        engine.bulk_load_check(unsafe { thd.as_ref() })
    })
}

/// Report the memory budget available for bulk load
///
/// # Safety
/// `ctx` non-null; `thd` null-or-valid for the call.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__bulk_load_available_memory(
    ctx: *mut EngineContext,
    thd: *const sys::THD,
) -> usize {
    FfiBoundary::run_default(0, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: thd is null or valid for read for the call's duration.
        engine.bulk_load_available_memory(unsafe { thd.as_ref() })
    })
}

/// Begin a bulk load, returning the engine-owned context pointer
///
/// # Safety
/// `ctx` non-null; `thd` null-or-valid for the call. The returned pointer is
/// engine-owned and round-trips back through `execute` / `end` unchanged.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__bulk_load_begin(
    ctx: *mut EngineContext,
    thd: *const sys::THD,
    data_size: usize,
    memory: usize,
    num_threads: usize,
) -> *mut c_void {
    FfiBoundary::run_default(core::ptr::null_mut(), || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: thd is null or valid for read for the call's duration.
        let thd = unsafe { thd.as_ref() };
        engine.bulk_load_begin(thd, data_size, memory, num_threads)
    })
}

/// Load one batch of rows on a bulk-load worker thread
///
/// # Safety
/// `ctx` non-null; `thd`/`rows`/`stat_callbacks` null-or-valid for the call.
/// `load_ctx` is the engine's own pointer from `bulk_load_begin` and is passed
/// through without dereference.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__bulk_load_execute(
    ctx: *mut EngineContext,
    thd: *const sys::THD,
    load_ctx: *mut c_void,
    thread_idx: usize,
    rows: *const sys::RowsMysql,
    stat_callbacks: *const sys::BulkLoadStatCallbacks,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: thd is null or valid for read for the call's duration.
        let thd = unsafe { thd.as_ref() };
        // SAFETY: rows is null or valid for read for the call's duration.
        let rows = unsafe { rows.as_ref() };
        // SAFETY: stat_callbacks is null or valid for read for the call.
        let stat_callbacks = unsafe { stat_callbacks.as_ref() };
        engine.bulk_load_execute(thd, load_ctx, thread_idx, rows, stat_callbacks)
    })
}

/// End the bulk load and release the engine context
///
/// # Safety
/// `ctx` non-null; `thd` null-or-valid for the call. `load_ctx` is the engine's
/// own pointer from `bulk_load_begin`, passed through without dereference.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__bulk_load_end(
    ctx: *mut EngineContext,
    thd: *const sys::THD,
    load_ctx: *mut c_void,
    is_error: bool,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: thd is null or valid for read for the call's duration.
        let thd = unsafe { thd.as_ref() };
        engine.bulk_load_end(thd, load_ctx, is_error)
    })
}

/// Load a primary-engine table into this secondary engine
///
/// # Safety
/// `ctx` non-null; `table` null-or-valid for the call; `skip_metadata_update`
/// writable for one `bool` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__load_table(
    ctx: *mut EngineContext,
    table: *const sys::TABLE,
    skip_metadata_update: *mut bool,
) -> i32 {
    FfiBoundary::run_default(EngineError::Internal.to_mysql_errno(), || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: table is null or valid for read for the call's duration.
        let table = unsafe { table.as_ref() };
        match engine.load_table(table) {
            Ok(skip) => {
                if !skip_metadata_update.is_null() {
                    // SAFETY: skip_metadata_update is writable when non-null.
                    unsafe { *skip_metadata_update = skip };
                }
                0
            }
            Err(e) => e.to_mysql_errno(),
        }
    })
}

/// Unload a table from this secondary engine
///
/// # Safety
/// `ctx` non-null; `db_name`/`table_name` cover their lengths for the call.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__unload_table(
    ctx: *mut EngineContext,
    db_name: *const u8,
    db_name_len: usize,
    table_name: *const u8,
    table_name_len: usize,
    error_if_not_loaded: bool,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees db_name covers db_name_len readable bytes.
        let db_name = unsafe { FfiPtr::bytes_to_str(db_name, db_name_len) }?;
        // SAFETY: caller guarantees table_name covers table_name_len bytes.
        let table_name = unsafe { FfiPtr::bytes_to_str(table_name, table_name_len) }?;
        engine.unload_table(db_name, table_name, error_if_not_loaded)
    })
}
