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

//! `rust__handler__*` callbacks for bulk-operation methods (handler.h #39–#46).
//! Shares the FFI safety contract documented at [`crate::handler`].

#![allow(unsafe_code)]

use crate::engine::EngineError;
use crate::panic_guard::FfiBoundary;
use crate::runtime::{EngineContext, FfiPtr};

/// Hint that a bulk insert of `rows` rows is starting
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__start_bulk_insert(ctx: *mut EngineContext, rows: u64) {
    FfiBoundary::run_void(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().start_bulk_insert(rows);
    });
}

/// Flush rows buffered since the bulk insert began
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__end_bulk_insert(ctx: *mut EngineContext) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().end_bulk_insert()
    })
}

/// Ask the engine whether it declines batching a multi-row UPDATE
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__start_bulk_update(ctx: *mut EngineContext) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        engine.start_bulk_update().to_mysql_bool()
    })
}

/// Apply buffered updates, writing the duplicate-key count to `dup_key_found`
///
/// # Safety
/// `ctx` non-null; `dup_key_found` writable for one `u32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__exec_bulk_update(
    ctx: *mut EngineContext,
    dup_key_found: *mut u32,
) -> i32 {
    FfiBoundary::run_default(EngineError::Internal.to_mysql_errno(), || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        match engine.exec_bulk_update() {
            Ok(dups) => {
                if !dup_key_found.is_null() {
                    // SAFETY: caller guarantees dup_key_found is writable when non-null.
                    unsafe { *dup_key_found = dups };
                }
                0
            }
            Err(e) => e.to_mysql_errno(),
        }
    })
}

/// Release bulk-update batch state
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__end_bulk_update(ctx: *mut EngineContext) {
    FfiBoundary::run_void(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().end_bulk_update();
    });
}

/// Buffer one row update for a later exec_bulk_update
///
/// # Safety
/// `ctx` non-null; `old`/`new_row` readable for their lengths; `dup_key_found`
/// writable for one `u32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__bulk_update_row(
    ctx: *mut EngineContext,
    old: *const u8,
    old_len: usize,
    new_row: *const u8,
    new_len: usize,
    dup_key_found: *mut u32,
) -> i32 {
    FfiBoundary::run_default(EngineError::Internal.to_mysql_errno(), || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees old covers old_len readable bytes.
        let old = unsafe { FfiPtr::slice_const(old, old_len) };
        // SAFETY: caller guarantees new_row covers new_len readable bytes.
        let new_row = unsafe { FfiPtr::slice_const(new_row, new_len) };
        match engine.bulk_update_row(old, new_row) {
            Ok(dups) => {
                if !dup_key_found.is_null() {
                    // SAFETY: caller guarantees dup_key_found is writable when non-null.
                    unsafe { *dup_key_found = dups };
                }
                0
            }
            Err(e) => e.to_mysql_errno(),
        }
    })
}

/// Ask the engine whether it declines batching a multi-row DELETE
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__start_bulk_delete(ctx: *mut EngineContext) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        engine.start_bulk_delete().to_mysql_bool()
    })
}

/// Execute buffered deletes and close the batch
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__end_bulk_delete(ctx: *mut EngineContext) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().end_bulk_delete()
    })
}
