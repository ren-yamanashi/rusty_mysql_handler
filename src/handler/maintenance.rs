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

//! `rust__handler__*` callbacks for the `CHECK` / `REPAIR` / `OPTIMIZE` /
//! `ANALYZE` admin commands (handler.h #130–#135). Shares the FFI safety
//! contract documented at [`crate::handler`]. The index/tablespace admin
//! methods live in [`crate::handler::index_admin`].
//!
//! Each callback returns `true` when the engine overrides (the `HA_ADMIN_*`
//! code / flag written through the out-pointer) and `false` to fall back to the
//! handler base. `THD` and `HA_CHECK_OPT` cross as opaque pointers.

#![allow(unsafe_code)]

use super::report::{report_bool, report_i32};
use crate::panic_guard::FfiBoundary;
use crate::runtime::EngineContext;
use crate::sys;

/// `CHECK TABLE`; returns whether the engine supplied an `HA_ADMIN_*` code
///
/// # Safety
/// `ctx` non-null; `thd`/`check_opt` null-or-valid for the call; `out` writable
/// for one `i32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__check(
    ctx: *mut EngineContext,
    thd: *const sys::THD,
    check_opt: *const sys::HaCheckOpt,
    out: *mut i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: thd/check_opt are null or valid for read for the call.
        let (thd, check_opt) = unsafe { (thd.as_ref(), check_opt.as_ref()) };
        report_i32(out, engine.check(thd, check_opt))
    })
}

/// `REPAIR TABLE`; returns whether the engine supplied an `HA_ADMIN_*` code
///
/// # Safety
/// `ctx` non-null; `thd`/`check_opt` null-or-valid for the call; `out` writable
/// for one `i32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__repair(
    ctx: *mut EngineContext,
    thd: *const sys::THD,
    check_opt: *const sys::HaCheckOpt,
    out: *mut i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: thd/check_opt are null or valid for read for the call.
        let (thd, check_opt) = unsafe { (thd.as_ref(), check_opt.as_ref()) };
        report_i32(out, engine.repair(thd, check_opt))
    })
}

/// `OPTIMIZE TABLE`; returns whether the engine supplied an `HA_ADMIN_*` code
///
/// # Safety
/// `ctx` non-null; `thd`/`check_opt` null-or-valid for the call; `out` writable
/// for one `i32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__optimize(
    ctx: *mut EngineContext,
    thd: *const sys::THD,
    check_opt: *const sys::HaCheckOpt,
    out: *mut i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: thd/check_opt are null or valid for read for the call.
        let (thd, check_opt) = unsafe { (thd.as_ref(), check_opt.as_ref()) };
        report_i32(out, engine.optimize(thd, check_opt))
    })
}

/// `ANALYZE TABLE`; returns whether the engine supplied an `HA_ADMIN_*` code
///
/// # Safety
/// `ctx` non-null; `thd`/`check_opt` null-or-valid for the call; `out` writable
/// for one `i32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__analyze(
    ctx: *mut EngineContext,
    thd: *const sys::THD,
    check_opt: *const sys::HaCheckOpt,
    out: *mut i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: thd/check_opt are null or valid for read for the call.
        let (thd, check_opt) = unsafe { (thd.as_ref(), check_opt.as_ref()) };
        report_i32(out, engine.analyze(thd, check_opt))
    })
}

/// Crash-recovery check-and-repair; returns whether the engine overrode it
///
/// # Safety
/// `ctx` non-null; `thd` null-or-valid for the call; `out` writable for one
/// `bool` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__check_and_repair(
    ctx: *mut EngineContext,
    thd: *const sys::THD,
    out: *mut bool,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: thd is null or valid for read for the call.
        report_bool(out, engine.check_and_repair(unsafe { thd.as_ref() }))
    })
}

/// Check whether the table needs upgrading; returns whether overridden
///
/// # Safety
/// `ctx` non-null; `check_opt` null-or-valid for the call; `out` writable for
/// one `i32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__check_for_upgrade(
    ctx: *mut EngineContext,
    check_opt: *const sys::HaCheckOpt,
    out: *mut i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: check_opt is null or valid for read for the call.
        report_i32(out, engine.check_for_upgrade(unsafe { check_opt.as_ref() }))
    })
}
