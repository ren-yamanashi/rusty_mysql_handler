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

//! `rust__handler__*` callbacks invoked by the C++ shim.
//!
//! # Safety (every function below)
//!
//! - `ctx` comes from `rust__create_engine` and has not been destroyed; the
//!   C++ shim guards every callback against null on its side, so each Rust
//!   callback requires non-null.
//! - The shim never calls a callback for the same `ctx` from two threads
//!   concurrently, so `&mut *ctx` is sound inside each callback.
//! - Pointer/length pairs are valid for the call only; engines must not
//!   retain them.

#![allow(unsafe_code)]

pub mod table_lifecycle;

use std::ffi::c_char;

use crate::ffi::{EngineContext, FfiPtr};
use crate::panic_guard::FfiBoundary;

/// Engine display name; null-terminated `'static`
///
/// # Safety
/// `ctx` must be non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__table_type(ctx: *mut EngineContext) -> *const c_char {
    FfiBoundary::run_default(std::ptr::null(), || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().table_type().as_ptr()
    })
}

/// `HA_*` capability bitfield
///
/// # Safety
/// `ctx` must be non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__table_flags(ctx: *mut EngineContext) -> u64 {
    FfiBoundary::run_default(0, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().table_flags()
    })
}

/// Per-index capability bitfield
///
/// # Safety
/// `ctx` must be non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__index_flags(
    ctx: *mut EngineContext,
    idx: u32,
    part: u32,
    all_parts: bool,
) -> u32 {
    FfiBoundary::run_default(0, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }
            .engine_mut()
            .index_flags(idx, part, all_parts)
    })
}

/// Create a new table
///
/// # Safety
/// `ctx` must be non-null; `name` must cover `name_len` readable bytes.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__create(
    ctx: *mut EngineContext,
    name: *const u8,
    name_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees name covers name_len readable bytes.
        let name = unsafe { FfiPtr::bytes_to_str(name, name_len) }?;
        engine.create(name)
    })
}

/// Open an existing table
///
/// # Safety
/// `ctx` must be non-null; `name` must cover `name_len` readable bytes.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__open(
    ctx: *mut EngineContext,
    name: *const u8,
    name_len: usize,
    mode: i32,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees name covers name_len readable bytes.
        let name = unsafe { FfiPtr::bytes_to_str(name, name_len) }?;
        engine.open(name, mode)
    })
}

/// Close the table
///
/// # Safety
/// `ctx` must be non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__close(ctx: *mut EngineContext) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().close()
    })
}

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

/// Refresh table statistics
///
/// # Safety
/// `ctx` must be non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__info(ctx: *mut EngineContext, flag: u32) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().info(flag)
    })
}
