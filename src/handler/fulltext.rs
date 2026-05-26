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

//! `rust__handler__*` callbacks for full-text search methods (handler.h
//! #60–#63). Shares the FFI safety contract documented at [`crate::handler`].

#![allow(unsafe_code)]

use core::ffi::c_void;

use crate::panic_guard::FfiBoundary;
use crate::runtime::{EngineContext, FfiPtr};
use crate::sys;

/// Begin a full-text search scan
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__ft_init(ctx: *mut EngineContext) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().ft_init()
    })
}

/// Create a full-text search handle, returning an engine-owned FT_INFO pointer
///
/// # Safety
/// `ctx` non-null; `key` null-or-valid for the call. The returned pointer is
/// engine-owned and round-tripped through MySQL without dereference.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__ft_init_ext(
    ctx: *mut EngineContext,
    flags: u32,
    inx: u32,
    key: *const sys::MysqlString,
) -> *mut c_void {
    FfiBoundary::run_default(core::ptr::null_mut(), || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: key is null or valid for read for the call's duration.
        let key = unsafe { key.as_ref() };
        engine.ft_init_ext(flags, inx, key)
    })
}

/// Create a full-text search handle from pre-extracted hint flags
///
/// # Safety
/// `ctx` non-null; `key` / `hints` null-or-valid for the call. The returned
/// pointer is engine-owned and round-tripped without dereference.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__ft_init_ext_with_hints(
    ctx: *mut EngineContext,
    flags: u32,
    inx: u32,
    key: *const sys::MysqlString,
    hints: *const sys::FtHints,
) -> *mut c_void {
    FfiBoundary::run_default(core::ptr::null_mut(), || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: key is null or valid for read for the call's duration.
        let key = unsafe { key.as_ref() };
        // SAFETY: hints is null or valid for read for the call's duration.
        let hints = unsafe { hints.as_ref() };
        engine.ft_init_ext_with_hints(flags, inx, key, hints)
    })
}

/// Read the next full-text match into `buf`
///
/// # Safety
/// `ctx` non-null; `buf` writable for `buf_len`.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__ft_read(
    ctx: *mut EngineContext,
    buf: *mut u8,
    buf_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees buf covers buf_len writable bytes.
        engine.ft_read(unsafe { FfiPtr::slice_mut(buf, buf_len) })
    })
}
