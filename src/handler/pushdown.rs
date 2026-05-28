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

//! `rust__handler__*` callbacks for condition / index pushdown and pushed-join
//! methods (handler.h #141–#148). Shares the FFI safety contract documented at
//! [`crate::handler`].
//!
//! `Item` / `handlerton` / `TABLE` pointers are round-tripped as opaque
//! pointers and never dereferenced from Rust. The trait defaults reproduce the
//! handler base (pass-through condition, null, `0`); `cancel_pushed_idx_cond`'s
//! base state reset is handled on the shim side.

#![allow(unsafe_code)]

use core::ffi::c_void;

use crate::panic_guard::FfiBoundary;
use crate::runtime::EngineContext;

/// Offer a WHERE condition for pushdown; returns the unhandled remainder
///
/// # Safety
/// `ctx` non-null; `cond` round-tripped without dereference.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__cond_push(
    ctx: *mut EngineContext,
    cond: *const c_void,
) -> *const c_void {
    FfiBoundary::run_default(cond, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().cond_push(cond)
    })
}

/// Offer an index condition for pushdown; returns the unhandled remainder
///
/// # Safety
/// `ctx` non-null; `idx_cond` round-tripped without dereference.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__idx_cond_push(
    ctx: *mut EngineContext,
    keyno: u32,
    idx_cond: *mut c_void,
) -> *mut c_void {
    FfiBoundary::run_default(idx_cond, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }
            .engine_mut()
            .idx_cond_push(keyno, idx_cond)
    })
}

/// Notify the engine that a pushed index condition was cancelled
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__cancel_pushed_idx_cond(ctx: *mut EngineContext) {
    FfiBoundary::run_void(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().cancel_pushed_idx_cond();
    });
}

/// The handlerton the engine can push work down to (opaque, may be null)
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__hton_supporting_engine_pushdown(
    ctx: *mut EngineContext,
) -> *const c_void {
    FfiBoundary::run_default(core::ptr::null(), || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }
            .engine_mut()
            .hton_supporting_engine_pushdown()
    })
}

/// Number of joins pushed down to the engine
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__number_of_pushed_joins(ctx: *mut EngineContext) -> u32 {
    FfiBoundary::run_default(0, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().number_of_pushed_joins()
    })
}

/// This handler's member TABLE in a pushed join (opaque, may be null)
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__member_of_pushed_join(
    ctx: *mut EngineContext,
) -> *const c_void {
    FfiBoundary::run_default(core::ptr::null(), || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().member_of_pushed_join()
    })
}

/// The root TABLE of this handler's pushed join (opaque, may be null)
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__parent_of_pushed_join(
    ctx: *mut EngineContext,
) -> *const c_void {
    FfiBoundary::run_default(core::ptr::null(), || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().parent_of_pushed_join()
    })
}

/// Bitmap of tables in this handler's pushed join
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__tables_in_pushed_join(ctx: *mut EngineContext) -> u64 {
    FfiBoundary::run_default(0, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().tables_in_pushed_join()
    })
}
