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

//! `rust__hton__*` secondary-engine offload failure-reason callbacks. Split
//! out of `secondary_engine.rs` because the `std::string_view` round-trip
//! introduces the `write_optional_str` helper that is only used by this
//! family. `set_*` lives next to its readers here so the three callbacks stay
//! together.
//!
//! [`HtonCapabilities::SECONDARY_ENGINE`]: crate::hton::HtonCapabilities::SECONDARY_ENGINE

#![allow(unsafe_code)]

use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::runtime::FfiPtr;
use crate::sys;

fn result_to_error(r: crate::engine::EngineResult) -> bool {
    match r {
        Ok(()) => false,
        Err(_) => true,
    }
}

/// Trait returns `Option<String>`; this helper writes the `(ptr, len)` pair
/// back through caller-owned out-pointers, mapping `None` to `(NULL, 0)` so
/// the shim sees an empty `std::string_view`.
pub(crate) fn write_optional_str(
    value: Option<&str>,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) {
    let (ptr, len) = match value {
        Some(s) => (s.as_ptr(), s.len()),
        None => (core::ptr::null(), 0_usize),
    };
    if !out_ptr.is_null() {
        // SAFETY: caller guarantees `out_ptr` is writable for one `*const u8`.
        unsafe { out_ptr.write(ptr) };
    }
    if !out_len.is_null() {
        // SAFETY: caller guarantees `out_len` is writable for one `usize`.
        unsafe { out_len.write(len) };
    }
}

/// `get_secondary_engine_offload_or_exec_fail_reason`. The shim owns the
/// destination buffer and copies the bytes the trait returns, so the
/// `String` is dropped as soon as the copy finishes.
///
/// # Safety
/// `thd` null or valid; `out_ptr` / `out_len` are non-null and writable for
/// one element each.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__get_secondary_engine_offload_or_exec_fail_reason(
    thd: *const sys::THD,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        let value = match runtime::handlerton() {
            Some(h) => h.get_secondary_engine_offload_or_exec_fail_reason(thd_ref),
            None => None,
        };
        write_optional_str(value.as_deref(), out_ptr, out_len);
        true
    })
}

/// # Safety
/// See [`rust__hton__get_secondary_engine_offload_or_exec_fail_reason`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__find_secondary_engine_offload_fail_reason(
    thd: *const sys::THD,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        let value = match runtime::handlerton() {
            Some(h) => h.find_secondary_engine_offload_fail_reason(thd_ref),
            None => None,
        };
        write_optional_str(value.as_deref(), out_ptr, out_len);
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
    fn write_optional_str_some_writes_ptr_and_len() {
        let s = "abc";
        let mut p: *const u8 = core::ptr::null();
        let mut l: usize = 999;
        write_optional_str(Some(s), &raw mut p, &raw mut l);
        assert_eq!(p, s.as_ptr());
        assert_eq!(l, 3);
    }

    #[test]
    fn write_optional_str_none_writes_null_and_zero() {
        let mut p: *const u8 = 0x1 as *const u8;
        let mut l: usize = 7;
        write_optional_str(None, &raw mut p, &raw mut l);
        assert!(p.is_null());
        assert_eq!(l, 0);
    }

    #[test]
    fn write_optional_str_tolerates_null_outs() {
        write_optional_str(Some("x"), core::ptr::null_mut(), core::ptr::null_mut());
        write_optional_str(None, core::ptr::null_mut(), core::ptr::null_mut());
    }
}
