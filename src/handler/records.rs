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

//! `rust__handler__*` callbacks for row-count methods (handler.h #99–#102).
//! Shares the FFI safety contract documented at [`crate::handler`].
//!
//! `records` / `records_from_index` return the HA error code and set `*handled`
//! when the engine supplied a count (written through `num_rows`), so an engine
//! that declines falls back to the handler base scan. The other two report the
//! handled flag through the bool return like the capability callbacks.

#![allow(unsafe_code)]

use core::ffi::c_void;

use super::report::report_u64;
use crate::engine::EngineResult;
use crate::panic_guard::FfiBoundary;
use crate::runtime::EngineContext;

// Map an engine row-count result onto the (num_rows, handled, error-code)
// out-protocol the shim expects: None leaves the count untouched and reports
// unhandled; Ok writes the count and reports handled with code 0; Err reports
// handled with the matching HA_ERR_* code.
fn report_records(num_rows: *mut u64, handled: *mut bool, value: Option<EngineResult<u64>>) -> i32 {
    let (code, did_handle) = match value {
        None => (0, false),
        Some(Ok(rows)) => {
            if !num_rows.is_null() {
                // SAFETY: num_rows is writable for one u64 when non-null.
                unsafe { *num_rows = rows };
            }
            (0, true)
        }
        Some(Err(e)) => (e.to_mysql_errno(), true),
    };
    if !handled.is_null() {
        // SAFETY: handled is writable for one bool when non-null.
        unsafe { *handled = did_handle };
    }
    code
}

/// Exact table row count
///
/// # Safety
/// `ctx` non-null; `num_rows` and `handled` writable for one value each when
/// non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__records(
    ctx: *mut EngineContext,
    num_rows: *mut u64,
    handled: *mut bool,
) -> i32 {
    FfiBoundary::run_default(0, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_records(
            num_rows,
            handled,
            unsafe { &mut *ctx }.engine_mut().records(),
        )
    })
}

/// Exact row count through a chosen index
///
/// # Safety
/// `ctx` non-null; `num_rows` and `handled` writable for one value each when
/// non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__records_from_index(
    ctx: *mut EngineContext,
    index: u32,
    num_rows: *mut u64,
    handled: *mut bool,
) -> i32 {
    FfiBoundary::run_default(0, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let value = unsafe { &mut *ctx }.engine_mut().records_from_index(index);
        report_records(num_rows, handled, value)
    })
}

/// Upper-bound row-count estimate
///
/// # Safety
/// `ctx` non-null; `out` writable for one `u64` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__estimate_rows_upper_bound(
    ctx: *mut EngineContext,
    out: *mut u64,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_u64(
            out,
            unsafe { &mut *ctx }
                .engine_mut()
                .estimate_rows_upper_bound(),
        )
    })
}

/// Hash of the key columns for hash partitioning
///
/// # Safety
/// `ctx` non-null; `field_array` valid for the call only; `out` writable for one
/// `u32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__calculate_key_hash_value(
    ctx: *mut EngineContext,
    field_array: *const c_void,
    out: *mut u32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        match unsafe { &mut *ctx }
            .engine_mut()
            .calculate_key_hash_value(field_array)
        {
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

#[cfg(test)]
mod tests {
    use super::report_records;
    use crate::engine::EngineError;
    use crate::sys;

    #[test]
    fn report_records_none_signals_unhandled_with_code_zero() {
        let mut num_rows = 7;
        let mut handled = true;
        let code = report_records(&mut num_rows, &mut handled, None);
        assert_eq!(code, 0);
        assert!(!handled);
        assert_eq!(num_rows, 7);
    }

    #[test]
    fn report_records_ok_writes_count_and_signals_handled() {
        let mut num_rows = 0;
        let mut handled = false;
        let code = report_records(&mut num_rows, &mut handled, Some(Ok(42)));
        assert_eq!(code, 0);
        assert!(handled);
        assert_eq!(num_rows, 42);
    }

    #[test]
    fn report_records_err_signals_handled_with_errno() {
        let mut num_rows = 5;
        let mut handled = false;
        let code = report_records(
            &mut num_rows,
            &mut handled,
            Some(Err(EngineError::Internal)),
        );
        assert_eq!(code, sys::HA_ERR_INTERNAL_ERROR);
        assert!(handled);
        assert_eq!(num_rows, 5);
    }

    #[test]
    fn report_records_tolerates_null_out_pointers() {
        let code = report_records(core::ptr::null_mut(), core::ptr::null_mut(), Some(Ok(1)));
        assert_eq!(code, 0);
    }
}
