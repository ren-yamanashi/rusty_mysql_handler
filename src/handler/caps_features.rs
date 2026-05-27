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

//! `rust__handler__*` callbacks for the index / row-type / EXPLAIN engine
//! capabilities (handler.h #82–#84, #87–#88). Shares the FFI safety contract
//! documented at [`crate::handler`]. The scalar flags live in
//! [`crate::handler::caps`].
//!
//! Enums (`row_type`, `ha_key_alg`) cross as their raw int value; each callback
//! returns `true` when the engine overrides the capability and `false` to use
//! the handler base default.

#![allow(unsafe_code)]

use core::ffi::c_void;

use super::report::{report_bool, report_i32};
use crate::panic_guard::FfiBoundary;
use crate::runtime::EngineContext;
use crate::sys;

unsafe extern "C" {
    /// Assign `len` bytes at `bytes` into the C++ `std::string` at `s`. Provided
    /// by the shim so `explain_extra` can return an owned string across the FFI.
    fn mysql__std_string__assign(s: *mut c_void, bytes: *const u8, len: usize);
}

/// Real row_type for the given create options (enum as raw int)
///
/// # Safety
/// `ctx` non-null; `create_info` null-or-valid for the call; `out` writable for
/// one `i32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__real_row_type(
    ctx: *mut EngineContext,
    create_info: *const sys::HA_CREATE_INFO,
    out: *mut i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: create_info is null or valid for read for the call's duration.
        let create_info = unsafe { create_info.as_ref() };
        report_i32(out, engine.real_row_type(create_info))
    })
}

/// Default index algorithm (enum as raw int)
///
/// # Safety
/// `ctx` non-null; `out` writable for one `i32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__default_index_algorithm(
    ctx: *mut EngineContext,
    out: *mut i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_i32(
            out,
            unsafe { &mut *ctx }.engine_mut().default_index_algorithm(),
        )
    })
}

/// Whether index algorithm `key_alg` (raw int) is supported
///
/// # Safety
/// `ctx` non-null; `out` writable for one `bool` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__is_index_algorithm_supported(
    ctx: *mut EngineContext,
    key_alg: i32,
    out: *mut bool,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        report_bool(out, engine.is_index_algorithm_supported(key_alg))
    })
}

/// Record-buffer request; writes the wanted row count when the engine wants one
///
/// # Safety
/// `ctx` non-null; `max_rows` writable for one `u64` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__record_buffer_wanted(
    ctx: *mut EngineContext,
    max_rows: *mut u64,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        match unsafe { &mut *ctx }.engine_mut().record_buffer_wanted() {
            Some(n) => {
                if !max_rows.is_null() {
                    // SAFETY: max_rows is writable for one u64 when non-null.
                    unsafe { *max_rows = n };
                }
                true
            }
            None => false,
        }
    })
}

/// EXPLAIN extra text; assigns into the shim-provided std::string when present
///
/// # Safety
/// `ctx` non-null; `out` is a valid `std::string *` the shim owns for the call.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__explain_extra(
    ctx: *mut EngineContext,
    out: *mut c_void,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        match unsafe { &mut *ctx }.engine_mut().explain_extra() {
            Some(s) => {
                // SAFETY: out is a valid std::string* for the call; bytes/len
                // describe s's buffer, copied by the shim before returning.
                unsafe { mysql__std_string__assign(out, s.as_ptr(), s.len()) };
                true
            }
            None => false,
        }
    })
}
