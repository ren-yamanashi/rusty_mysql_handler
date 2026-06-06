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

//! `rust__handler__*` callbacks for the scalar cost methods that return a single
//! time/cost figure (handler.h #91–#93, #97–#98). The `Cost_estimate`-returning
//! methods live in [`crate::handler::cost`]. Shares the FFI safety contract
//! documented at [`crate::handler`].
//!
//! Each callback writes its estimate through `out` and returns `true` when the
//! engine overrides, or `false` to fall back to the handler base.

#![allow(unsafe_code)]

use super::report::report_f64;
use crate::panic_guard::FfiBoundary;
use crate::runtime::EngineContext;

/// Full-table-scan time estimate
///
/// # Safety
/// `ctx` non-null; `out` writable for one `f64` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__scan_time(ctx: *mut EngineContext, out: *mut f64) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_f64(out, unsafe { &mut *ctx }.engine_mut().scan_time())
    })
}

/// Index range-read time estimate
///
/// # Safety
/// `ctx` non-null; `out` writable for one `f64` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__read_time(
    ctx: *mut EngineContext,
    index: u32,
    ranges: u32,
    rows: u64,
    out: *mut f64,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_f64(
            out,
            unsafe { &mut *ctx }
                .engine_mut()
                .read_time(index, ranges, rows),
        )
    })
}

/// Index-only read time estimate
///
/// # Safety
/// `ctx` non-null; `out` writable for one `f64` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__index_only_read_time(
    ctx: *mut EngineContext,
    keynr: u32,
    records: f64,
    out: *mut f64,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_f64(
            out,
            unsafe { &mut *ctx }
                .engine_mut()
                .index_only_read_time(keynr, records),
        )
    })
}

/// Non-sequential page-read cost estimate
///
/// # Safety
/// `ctx` non-null; `out` writable for one `f64` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__page_read_cost(
    ctx: *mut EngineContext,
    index: u32,
    reads: f64,
    out: *mut f64,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_f64(
            out,
            unsafe { &mut *ctx }
                .engine_mut()
                .page_read_cost(index, reads),
        )
    })
}

/// Worst-case seek-and-read cost estimate
///
/// # Safety
/// `ctx` non-null; `out` writable for one `f64` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__worst_seek_times(
    ctx: *mut EngineContext,
    reads: f64,
    out: *mut f64,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_f64(
            out,
            unsafe { &mut *ctx }.engine_mut().worst_seek_times(reads),
        )
    })
}
