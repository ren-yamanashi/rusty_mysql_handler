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

//! `rust__handler__*` callbacks for error-handling methods (handler.h
//! #114–#118). Shares the FFI safety contract documented at [`crate::handler`].
//!
//! Each callback returns `true` when the engine overrides (message/flag written
//! out) and `false` to fall back to the handler base, whose error
//! classification and message formatting are non-trivial.

#![allow(unsafe_code)]

use core::ffi::c_void;

use super::report::report_bool;
use crate::panic_guard::FfiBoundary;
use crate::runtime::EngineContext;

unsafe extern "C" {
    // Defined by the shim: copies len bytes into the MySQL `String` out-buffer
    // so error_message can return an owned string across the FFI.
    fn mysql__mysql_string__set(buf: *mut c_void, bytes: *const u8, len: usize);
}

// Copy `s` into a caller-owned C string buffer of `cap` bytes, truncating to
// leave room for the NUL terminator. No-op when the buffer is null or empty.
fn write_cstr_bounded(buf: *mut u8, cap: u32, s: &str) {
    let cap = cap as usize;
    if buf.is_null() || cap == 0 {
        return;
    }
    let bytes = s.as_bytes();
    let n = bytes.len().min(cap - 1);
    // SAFETY: caller guarantees buf is writable for cap bytes; n < cap leaves
    // room for the NUL written at index n.
    unsafe {
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, n);
        *buf.add(n) = 0;
    }
}

// Write both child-table and child-key names into their bounded buffers; report
// whether the engine supplied a pair.
fn report_foreign_dup_key(
    table_buf: *mut u8,
    table_cap: u32,
    key_buf: *mut u8,
    key_cap: u32,
    value: Option<(String, String)>,
) -> bool {
    match value {
        Some((table, key)) => {
            write_cstr_bounded(table_buf, table_cap, &table);
            write_cstr_bounded(key_buf, key_cap, &key);
            true
        }
        None => false,
    }
}

/// Print an engine-specific diagnostic; returns whether the engine handled it
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__print_error(
    ctx: *mut EngineContext,
    error: i32,
    errflag: u64,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }
            .engine_mut()
            .print_error(error, errflag)
    })
}

/// Write an engine-specific error message into `buf`; returns the temporary-error
/// flag. Message *presence* is signalled to the shim by `buf` being non-empty
/// (matching MySQL's `handler::get_error_message` contract), so the return value
/// carries only the temporary/permanent distinction.
///
/// # Safety
/// `ctx` non-null; `buf` a valid MySQL `String *` for the call.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__get_error_message(
    ctx: *mut EngineContext,
    error: i32,
    buf: *mut c_void,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        match unsafe { &mut *ctx }.engine_mut().error_message(error) {
            Some((msg, temporary)) => {
                // SAFETY: buf is a valid String* for the call; bytes/len describe
                // msg's buffer, copied by the shim before returning.
                unsafe { mysql__mysql_string__set(buf, msg.as_ptr(), msg.len()) };
                temporary
            }
            None => false,
        }
    })
}

/// Report the foreign-key duplicate table/key names; returns whether available
///
/// # Safety
/// `ctx` non-null; `table_buf`/`key_buf` writable for their stated capacities.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__get_foreign_dup_key(
    ctx: *mut EngineContext,
    table_buf: *mut u8,
    table_cap: u32,
    key_buf: *mut u8,
    key_cap: u32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let value = unsafe { &mut *ctx }.engine_mut().foreign_dup_key();
        report_foreign_dup_key(table_buf, table_cap, key_buf, key_cap, value)
    })
}

/// Classify whether `error` is ignorable; returns whether the engine overrode it
///
/// # Safety
/// `ctx` non-null; `out` writable for one `bool` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__is_ignorable_error(
    ctx: *mut EngineContext,
    error: i32,
    out: *mut bool,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_bool(
            out,
            unsafe { &mut *ctx }.engine_mut().is_ignorable_error(error),
        )
    })
}

/// Classify whether `error` is fatal; returns whether the engine overrode it
///
/// # Safety
/// `ctx` non-null; `out` writable for one `bool` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__is_fatal_error(
    ctx: *mut EngineContext,
    error: i32,
    out: *mut bool,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_bool(out, unsafe { &mut *ctx }.engine_mut().is_fatal_error(error))
    })
}

#[cfg(test)]
mod tests {
    use super::{report_foreign_dup_key, write_cstr_bounded};

    #[test]
    fn write_cstr_bounded_truncates_and_nul_terminates() {
        let mut buf = [0xFFu8; 4];
        write_cstr_bounded(buf.as_mut_ptr(), 4, "abcdef");
        assert_eq!(&buf, b"abc\0");
    }

    #[test]
    fn write_cstr_bounded_writes_full_string_with_nul() {
        let mut buf = [0xFFu8; 8];
        write_cstr_bounded(buf.as_mut_ptr(), 8, "ab");
        assert_eq!(&buf[..3], b"ab\0");
    }

    #[test]
    fn write_cstr_bounded_tolerates_null_and_zero_cap() {
        write_cstr_bounded(core::ptr::null_mut(), 4, "x");
        let mut buf = [7u8; 2];
        write_cstr_bounded(buf.as_mut_ptr(), 0, "x");
        assert_eq!(buf, [7, 7]);
    }

    #[test]
    fn report_foreign_dup_key_writes_both_and_signals_handled() {
        let mut t = [0u8; 8];
        let mut k = [0u8; 8];
        let handled = report_foreign_dup_key(
            t.as_mut_ptr(),
            8,
            k.as_mut_ptr(),
            8,
            Some(("tbl".to_owned(), "key".to_owned())),
        );
        assert!(handled);
        assert_eq!(&t[..4], b"tbl\0");
        assert_eq!(&k[..4], b"key\0");
    }

    #[test]
    fn report_foreign_dup_key_none_signals_unhandled() {
        assert!(!report_foreign_dup_key(
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
            None
        ));
    }
}
