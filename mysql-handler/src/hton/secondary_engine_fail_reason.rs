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

//! `rust__hton__*` secondary-engine offload failure-reason callbacks.
//!
//! `get_*` and `find_*` cannot safely round-trip an engine-owned buffer back
//! to MySQL through this layer today — the shim wraps the returned bytes in a
//! `std::string_view` that outlives the FFI call, so a trait method returning
//! `Option<String>` would drop the buffer before MySQL inspects it
//! (use-after-free + adjacent-heap-byte disclosure). The two read callbacks
//! therefore unconditionally return an empty `string_view`; a future setter
//! reverse-callback that hands engine-owned, statement-scoped bytes to MySQL
//! can re-add the read path without that lifetime gap. `set_*` is unaffected:
//! it borrows bytes from MySQL for the call duration and the trait copies if
//! it wants to retain them.

#![allow(unsafe_code)]

use crate::hton::result::result_to_error;
use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::runtime::FfiPtr;
use crate::sys;

/// Write `(NULL, 0)` into the two out-pointers. Lifted so the two read
/// callbacks share one tiny helper that is trivial to unit-test.
fn write_empty_string_view(out_ptr: *mut *const u8, out_len: *mut usize) {
    if !out_ptr.is_null() {
        // SAFETY: caller guarantees `out_ptr` is writable for one `*const u8`.
        unsafe { out_ptr.write(core::ptr::null()) };
    }
    if !out_len.is_null() {
        // SAFETY: caller guarantees `out_len` is writable for one `usize`.
        unsafe { out_len.write(0) };
    }
}

/// `get_secondary_engine_offload_or_exec_fail_reason`. Returns an empty
/// `std::string_view` (see module doc for the lifetime rationale).
///
/// # Safety
/// `thd` null or valid; `out_ptr` / `out_len` null or writable for one
/// element each.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__get_secondary_engine_offload_or_exec_fail_reason(
    _thd: *const sys::THD,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> bool {
    FfiBoundary::run_default(false, || {
        write_empty_string_view(out_ptr, out_len);
        true
    })
}

/// `find_secondary_engine_offload_fail_reason`. Same empty-view treatment as
/// [`rust__hton__get_secondary_engine_offload_or_exec_fail_reason`].
///
/// # Safety
/// See [`rust__hton__get_secondary_engine_offload_or_exec_fail_reason`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__find_secondary_engine_offload_fail_reason(
    _thd: *const sys::THD,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> bool {
    FfiBoundary::run_default(false, || {
        write_empty_string_view(out_ptr, out_len);
        true
    })
}

/// # Safety
/// `thd` null or valid; `reason` is null or readable for `reason_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__set_secondary_engine_offload_fail_reason(
    thd: *const sys::THD,
    reason: *const u8,
    reason_len: usize,
) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        let reason_str = if reason.is_null() {
            ""
        } else {
            // SAFETY: reason is non-null and covers reason_len readable bytes here.
            match unsafe { FfiPtr::bytes_to_str(reason, reason_len) } {
                Ok(s) => s,
                Err(_) => return true,
            }
        };
        match runtime::handlerton() {
            Some(h) => {
                result_to_error(h.set_secondary_engine_offload_fail_reason(thd_ref, reason_str))
            }
            None => false,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_empty_string_view_writes_null_and_zero() {
        let mut p: *const u8 = std::ptr::dangling::<u8>();
        let mut l: usize = 7;
        write_empty_string_view(&raw mut p, &raw mut l);
        assert!(p.is_null());
        assert_eq!(l, 0);
    }

    #[test]
    fn write_empty_string_view_tolerates_null_outs() {
        write_empty_string_view(core::ptr::null_mut(), core::ptr::null_mut());
    }
}
