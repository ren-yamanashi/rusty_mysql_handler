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

//! `rust__handler__*` callbacks for row-operation methods (handler.h #35–#38).
//! Shares the FFI safety contract documented at [`crate::ffi_handler`].

#![allow(unsafe_code)]

use crate::ffi::{EngineContext, FfiPtr};
use crate::panic_guard::FfiBoundary;

/// Insert the row encoded in `buf`
///
/// # Safety
/// `ctx` non-null; `buf` readable for `buf_len` bytes for the call's duration.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__write_row(
    ctx: *mut EngineContext,
    buf: *const u8,
    buf_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees buf covers buf_len readable bytes.
        let buf = unsafe { FfiPtr::slice_const(buf, buf_len) };
        engine.write_row(buf)
    })
}

/// Replace the row imaged by `old` with the row imaged by `new`
///
/// # Safety
/// `ctx` non-null; `old`/`new` readable for their lengths for the call.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__update_row(
    ctx: *mut EngineContext,
    old: *const u8,
    old_len: usize,
    new: *const u8,
    new_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees old covers old_len readable bytes.
        let old = unsafe { FfiPtr::slice_const(old, old_len) };
        // SAFETY: caller guarantees new covers new_len readable bytes.
        let new = unsafe { FfiPtr::slice_const(new, new_len) };
        engine.update_row(old, new)
    })
}

/// Delete the row imaged by `buf`
///
/// # Safety
/// `ctx` non-null; `buf` readable for `buf_len` bytes for the call's duration.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__delete_row(
    ctx: *mut EngineContext,
    buf: *const u8,
    buf_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees buf covers buf_len readable bytes.
        let buf = unsafe { FfiPtr::slice_const(buf, buf_len) };
        engine.delete_row(buf)
    })
}

/// Delete every row in the table in one operation
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__delete_all_rows(ctx: *mut EngineContext) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().delete_all_rows()
    })
}
