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

//! `rust__handler__*` callbacks for the scalar engine-capability flags
//! (handler.h #77–#80, #89; checksum #78). Shares the FFI safety contract
//! documented at [`crate::handler`]. The index / feature capabilities live in
//! [`crate::handler::caps_features`].
//!
//! Each callback returns `true` when the engine overrides the capability (value
//! written through the out-pointer) and `false` to use the handler base default.

#![allow(unsafe_code)]

use super::report::{report_bool, report_i32};
use crate::panic_guard::FfiBoundary;
use crate::runtime::EngineContext;

/// Little-endian storage flag
///
/// # Safety
/// `ctx` non-null; `out` writable for one `bool` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__low_byte_first(
    ctx: *mut EngineContext,
    out: *mut bool,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_bool(out, unsafe { &mut *ctx }.engine_mut().low_byte_first())
    })
}

/// Live table checksum
///
/// # Safety
/// `ctx` non-null; `out` writable for one `u32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__checksum(ctx: *mut EngineContext, out: *mut u32) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        match unsafe { &mut *ctx }.engine_mut().checksum() {
            Some(v) => {
                if !out.is_null() {
                    // SAFETY: out is writable for one u32 when non-null.
                    unsafe { *out = v };
                }
                true
            }
            None => false,
        }
    })
}

/// Table-crashed flag
///
/// # Safety
/// `ctx` non-null; `out` writable for one `bool` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__is_crashed(
    ctx: *mut EngineContext,
    out: *mut bool,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_bool(out, unsafe { &mut *ctx }.engine_mut().is_crashed())
    })
}

/// Auto-repair-on-open flag
///
/// # Safety
/// `ctx` non-null; `out` writable for one `bool` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__auto_repair(
    ctx: *mut EngineContext,
    out: *mut bool,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_bool(out, unsafe { &mut *ctx }.engine_mut().auto_repair())
    })
}

/// Clustered-primary-key flag
///
/// # Safety
/// `ctx` non-null; `out` writable for one `bool` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__primary_key_is_clustered(
    ctx: *mut EngineContext,
    out: *mut bool,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_bool(
            out,
            unsafe { &mut *ctx }.engine_mut().primary_key_is_clustered(),
        )
    })
}

/// Indexes-disabled status code
///
/// # Safety
/// `ctx` non-null; `out` writable for one `i32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__indexes_are_disabled(
    ctx: *mut EngineContext,
    out: *mut i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_i32(
            out,
            unsafe { &mut *ctx }.engine_mut().indexes_are_disabled(),
        )
    })
}
