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

//! `rust__handler__*` callbacks for locking methods (handler.h #104–#109).
//! Shares the FFI safety contract documented at [`crate::handler`]. The handler
//! base default of each method is trivial, so these delegate straight to the
//! engine; the trait defaults reproduce the base behaviour.

#![allow(unsafe_code)]

use crate::panic_guard::FfiBoundary;
use crate::runtime::EngineContext;
use crate::sys;

/// Acquire or release a table-level lock; returns an HA error code (0 = success)
///
/// # Safety
/// `ctx` non-null; `thd` null-or-valid for the call.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__external_lock(
    ctx: *mut EngineContext,
    thd: *const sys::THD,
    lock_type: i32,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: thd is null or valid for read for the call's duration.
        let thd = unsafe { thd.as_ref() };
        engine.external_lock(thd, lock_type)
    })
}

/// Number of `THR_LOCK` entries the engine reports
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__lock_count(ctx: *mut EngineContext) -> u32 {
    FfiBoundary::run_default(1, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().lock_count()
    })
}

/// Release the lock on the most recently read row
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__unlock_row(ctx: *mut EngineContext) {
    FfiBoundary::run_void(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().unlock_row();
    });
}

/// Begin a statement while the table is already locked; returns an HA error code
///
/// # Safety
/// `ctx` non-null; `thd` null-or-valid for the call.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__start_stmt(
    ctx: *mut EngineContext,
    thd: *const sys::THD,
    lock_type: i32,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: thd is null or valid for read for the call's duration.
        let thd = unsafe { thd.as_ref() };
        engine.start_stmt(thd, lock_type)
    })
}

/// Whether the last row was read with a semi-consistent read
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__was_semi_consistent_read(ctx: *mut EngineContext) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().was_semi_consistent_read()
    })
}

/// Enable or disable semi-consistent reads for subsequent reads
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__try_semi_consistent_read(
    ctx: *mut EngineContext,
    enable: bool,
) {
    FfiBoundary::run_void(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }
            .engine_mut()
            .try_semi_consistent_read(enable);
    });
}
