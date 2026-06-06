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

//! `rust__handler__*` callbacks for engine limit / size methods (handler.h
//! #71–#76, #85–#86). Shares the FFI safety contract documented at
//! [`crate::handler`].
//!
//! Each callback returns `true` when the engine overrides the limit (value
//! written through the out-pointer) and `false` to use the handler base
//! default, so an engine that does not care keeps MySQL's built-in limits.

#![allow(unsafe_code)]

use crate::panic_guard::FfiBoundary;
use crate::runtime::EngineContext;
use crate::sys;

// Write `value` through `out` when the engine supplied one, reporting whether
// it did so to the shim.
fn report_u32(out: *mut u32, value: Option<u32>) -> bool {
    match value {
        Some(v) => {
            if !out.is_null() {
                // SAFETY: out is writable for one u32 when non-null.
                unsafe { *out = v };
            }
            true
        }
        None => false,
    }
}

/// Maximum supported row length; returns whether the engine overrode it
///
/// # Safety
/// `ctx` non-null; `out` writable for one `u32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__max_supported_record_length(
    ctx: *mut EngineContext,
    out: *mut u32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_u32(
            out,
            unsafe { &mut *ctx }
                .engine_mut()
                .max_supported_record_length(),
        )
    })
}

/// Maximum supported index count; returns whether the engine overrode it
///
/// # Safety
/// `ctx` non-null; `out` writable for one `u32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__max_supported_keys(
    ctx: *mut EngineContext,
    out: *mut u32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_u32(out, unsafe { &mut *ctx }.engine_mut().max_supported_keys())
    })
}

/// Maximum supported key-part count; returns whether the engine overrode it
///
/// # Safety
/// `ctx` non-null; `out` writable for one `u32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__max_supported_key_parts(
    ctx: *mut EngineContext,
    out: *mut u32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_u32(
            out,
            unsafe { &mut *ctx }.engine_mut().max_supported_key_parts(),
        )
    })
}

/// Maximum supported total key length; returns whether the engine overrode it
///
/// # Safety
/// `ctx` non-null; `out` writable for one `u32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__max_supported_key_length(
    ctx: *mut EngineContext,
    out: *mut u32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_u32(
            out,
            unsafe { &mut *ctx }.engine_mut().max_supported_key_length(),
        )
    })
}

/// Maximum supported single key-part length; returns whether the engine overrode it
///
/// # Safety
/// `ctx` non-null; `create_info` null-or-valid for the call; `out` writable for
/// one `u32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__max_supported_key_part_length(
    ctx: *mut EngineContext,
    create_info: *const sys::HA_CREATE_INFO,
    out: *mut u32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: create_info is null or valid for read for the call's duration.
        let create_info = unsafe { create_info.as_ref() };
        report_u32(out, engine.max_supported_key_part_length(create_info))
    })
}

/// Minimum row length for the given create options; returns whether overridden
///
/// # Safety
/// `ctx` non-null; `out` writable for one `u32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__min_record_length(
    ctx: *mut EngineContext,
    options: u32,
    out: *mut u32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_u32(
            out,
            unsafe { &mut *ctx }.engine_mut().min_record_length(options),
        )
    })
}

/// Extra per-record buffer length; returns whether the engine overrode it
///
/// # Safety
/// `ctx` non-null; `out` writable for one `u32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__extra_rec_buf_length(
    ctx: *mut EngineContext,
    out: *mut u32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_u32(
            out,
            unsafe { &mut *ctx }.engine_mut().extra_rec_buf_length(),
        )
    })
}

/// In-memory buffer size; returns whether the engine overrode it
///
/// # Safety
/// `ctx` non-null; `out` writable for one `i64` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__memory_buffer_size(
    ctx: *mut EngineContext,
    out: *mut i64,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        match unsafe { &mut *ctx }.engine_mut().memory_buffer_size() {
            Some(v) => {
                if !out.is_null() {
                    // SAFETY: out is writable for one i64 when non-null.
                    unsafe { *out = v };
                }
                true
            }
            None => false,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::report_u32;

    #[test]
    fn report_u32_writes_value_and_signals_handled() {
        let mut out: u32 = 0;
        assert!(report_u32(&raw mut out, Some(42)));
        assert_eq!(out, 42);
    }

    #[test]
    fn report_u32_none_leaves_buffer_untouched_and_signals_unhandled() {
        let mut out: u32 = 7;
        assert!(!report_u32(&raw mut out, None));
        assert_eq!(out, 7);
    }

    #[test]
    fn report_u32_tolerates_null_out_pointer() {
        assert!(report_u32(core::ptr::null_mut(), Some(99)));
    }
}
