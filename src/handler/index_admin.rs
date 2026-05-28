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

//! `rust__handler__*` callbacks for the key-cache and tablespace admin commands
//! (handler.h #136–#140). Shares the FFI safety contract documented at
//! [`crate::handler`]. The `CHECK` / `REPAIR` / `OPTIMIZE` / `ANALYZE` commands
//! live in [`crate::handler::maintenance`].
//!
//! Each callback returns `true` when the engine overrides (the raw handler code
//! written through the out-pointer) and `false` to fall back to the handler
//! base. `THD`, `HA_CHECK_OPT` and `dd::Table` cross as opaque pointers.

#![allow(unsafe_code)]

use super::report::report_i32;
use crate::panic_guard::FfiBoundary;
use crate::runtime::EngineContext;
use crate::sys;

/// `ASSIGN_TO_KEYCACHE`; returns whether the engine supplied a code
///
/// # Safety
/// `ctx` non-null; `thd`/`check_opt` null-or-valid for the call; `out` writable
/// for one `i32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__assign_to_keycache(
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
        report_i32(out, engine.assign_to_keycache(thd, check_opt))
    })
}

/// `LOAD INDEX` (preload keys); returns whether the engine supplied a code
///
/// # Safety
/// `ctx` non-null; `thd`/`check_opt` null-or-valid for the call; `out` writable
/// for one `i32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__preload_keys(
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
        report_i32(out, engine.preload_keys(thd, check_opt))
    })
}

/// `ALTER TABLE ... DISABLE KEYS`; returns whether the engine supplied a code
///
/// # Safety
/// `ctx` non-null; `out` writable for one `i32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__disable_indexes(
    ctx: *mut EngineContext,
    mode: u32,
    out: *mut i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_i32(out, unsafe { &mut *ctx }.engine_mut().disable_indexes(mode))
    })
}

/// `ALTER TABLE ... ENABLE KEYS`; returns whether the engine supplied a code
///
/// # Safety
/// `ctx` non-null; `out` writable for one `i32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__enable_indexes(
    ctx: *mut EngineContext,
    mode: u32,
    out: *mut i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_i32(out, unsafe { &mut *ctx }.engine_mut().enable_indexes(mode))
    })
}

/// `DISCARD` / `IMPORT TABLESPACE`; returns whether the engine supplied a code
///
/// # Safety
/// `ctx` non-null; `table_def` null-or-valid for the call; `out` writable for
/// one `i32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__discard_or_import_tablespace(
    ctx: *mut EngineContext,
    discard: bool,
    table_def: *const sys::DdTable,
    out: *mut i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: table_def is null or valid for read for the call.
        let value = engine.discard_or_import_tablespace(discard, unsafe { table_def.as_ref() });
        report_i32(out, value)
    })
}
