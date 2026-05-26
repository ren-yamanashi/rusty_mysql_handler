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

//! `rust__handler__*` callbacks for full-table-scan methods (rnd_init,
//! rnd_next, rnd_pos, position). Shares the FFI safety contract documented at
//! [`crate::handler`].

#![allow(unsafe_code)]

use crate::panic_guard::FfiBoundary;
use crate::runtime::{EngineContext, FfiPtr};

/// Begin a full table scan
///
/// # Safety
/// `ctx` must be non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__rnd_init(ctx: *mut EngineContext, scan: bool) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().rnd_init(scan)
    })
}

/// Fetch the next row
///
/// # Safety
/// `ctx` must be non-null; `buf` must be writable for `buf_len` bytes.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__rnd_next(
    ctx: *mut EngineContext,
    buf: *mut u8,
    buf_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees buf covers buf_len writable bytes.
        engine.rnd_next(unsafe { FfiPtr::slice_mut(buf, buf_len) })
    })
}

/// End the full table scan
///
/// # Safety
/// `ctx` must be non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__rnd_end(ctx: *mut EngineContext) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().rnd_end()
    })
}

/// Fetch a row by stored position
///
/// # Safety
/// `ctx` must be non-null; `buf` writable for `buf_len`, `pos` readable for
/// `pos_len`.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__rnd_pos(
    ctx: *mut EngineContext,
    buf: *mut u8,
    buf_len: usize,
    pos: *const u8,
    pos_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees buf covers buf_len writable bytes.
        let buf = unsafe { FfiPtr::slice_mut(buf, buf_len) };
        // SAFETY: caller guarantees pos covers pos_len readable bytes.
        let pos = unsafe { FfiPtr::slice_const(pos, pos_len) };
        engine.rnd_pos(buf, pos)
    })
}

/// Store the current row's position
///
/// # Safety
/// `ctx` must be non-null; `record` must cover `record_len` readable bytes.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__position(
    ctx: *mut EngineContext,
    record: *const u8,
    record_len: usize,
) {
    FfiBoundary::run_void(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees record covers record_len readable bytes.
        let record = unsafe { FfiPtr::slice_const(record, record_len) };
        engine.position(record);
    });
}

/// Read the row matching the primary key encoded in `record`
///
/// # Safety
/// `ctx` must be non-null; `record` must cover `record_len` readable and
/// writable bytes (it carries the key in and receives the row out).
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__rnd_pos_by_record(
    ctx: *mut EngineContext,
    record: *mut u8,
    record_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees record covers record_len read/write bytes.
        let record = unsafe { FfiPtr::slice_mut(record, record_len) };
        engine.rnd_pos_by_record(record)
    })
}
