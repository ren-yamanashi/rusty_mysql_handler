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

//! `rust__hton__page_track_*` callbacks. Wired only under
//! [`HtonCapabilities::PAGE_TRACKING`]; the shim assigns `hton->page_track`
//! as a unit when the capability is declared. `Page_Track_Callback` and its
//! `void *` context are opaque today; engines that fetch page IDs through
//! the callback will need a reverse-callback surface. `get_status` returns
//! a `std::vector` by value upstream — the FFI ignores the output until that
//! can be marshalled safely.
//!
//! [`HtonCapabilities::PAGE_TRACKING`]: crate::hton::HtonCapabilities::PAGE_TRACKING

#![allow(unsafe_code)]

use crate::engine::EngineError;
use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::runtime::FfiPtr;

fn write_u64(out: *mut u64, value: u64) {
    if !out.is_null() {
        // SAFETY: caller guarantees `out` is writable for one u64.
        unsafe { out.write(value) };
    }
}

fn report_u64(value: Result<u64, EngineError>, out: *mut u64) -> i32 {
    match value {
        Ok(v) => {
            if out.is_null() {
                return EngineError::Internal.to_mysql_errno();
            }
            write_u64(out, v);
            0
        }
        Err(e) => e.to_mysql_errno(),
    }
}

/// `page_track_start`.
///
/// # Safety
/// `start_id` null or writable for one `u64`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__page_track_start(start_id: *mut u64) -> i32 {
    FfiBoundary::run_default(EngineError::Internal.to_mysql_errno(), || {
        let result = match runtime::handlerton() {
            Some(h) => h.page_track_start(),
            None => Err(EngineError::Unsupported),
        };
        report_u64(result, start_id)
    })
}

/// `page_track_stop`.
///
/// # Safety
/// `stop_id` null or writable for one `u64`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__page_track_stop(stop_id: *mut u64) -> i32 {
    FfiBoundary::run_default(EngineError::Internal.to_mysql_errno(), || {
        let result = match runtime::handlerton() {
            Some(h) => h.page_track_stop(),
            None => Err(EngineError::Unsupported),
        };
        report_u64(result, stop_id)
    })
}

/// `page_track_purge`. `purge_id` is in/out: input is the boundary MySQL
/// asks the engine to purge through; output is the boundary actually
/// reached.
///
/// # Safety
/// `purge_id` null or writable for one `u64`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__page_track_purge(purge_id: *mut u64) -> i32 {
    FfiBoundary::run_default(EngineError::Internal.to_mysql_errno(), || {
        let requested = if purge_id.is_null() {
            0
        } else {
            // SAFETY: caller guarantees `purge_id` is readable for one u64.
            unsafe { purge_id.read() }
        };
        let result = match runtime::handlerton() {
            Some(h) => h.page_track_purge(requested),
            None => Err(EngineError::Unsupported),
        };
        report_u64(result, purge_id)
    })
}

/// `page_track_get_page_ids`. The MySQL callback / context that receives
/// page IDs is opaque today; the trait receives only the (start, stop)
/// range and the destination buffer.
///
/// # Safety
/// `start_id` / `stop_id` non-null and writable for one `u64` each;
/// `buffer` null or writable for `buffer_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__page_track_get_page_ids(
    start_id: *mut u64,
    stop_id: *mut u64,
    buffer: *mut u8,
    buffer_len: usize,
) -> i32 {
    FfiBoundary::run_default(EngineError::Internal.to_mysql_errno(), || {
        let range_start = if start_id.is_null() {
            return EngineError::Internal.to_mysql_errno();
        } else {
            // SAFETY: caller guarantees `start_id` is readable for one u64.
            unsafe { start_id.read() }
        };
        let range_stop = if stop_id.is_null() {
            return EngineError::Internal.to_mysql_errno();
        } else {
            // SAFETY: caller guarantees `stop_id` is readable for one u64.
            unsafe { stop_id.read() }
        };
        let buf: &mut [u8] = if buffer.is_null() {
            &mut []
        } else {
            // SAFETY: buffer non-null here and writable for buffer_len bytes.
            unsafe { FfiPtr::slice_mut(buffer, buffer_len) }
        };
        let result = match runtime::handlerton() {
            Some(h) => h.page_track_get_page_ids(range_start, range_stop, buf),
            None => Err(EngineError::Unsupported),
        };
        match result {
            Ok(()) => 0,
            Err(e) => e.to_mysql_errno(),
        }
    })
}

/// `page_track_get_num_page_ids`.
///
/// # Safety
/// `start_id` / `stop_id` non-null and readable for one `u64` each;
/// `num_pages` non-null and writable for one `u64`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__page_track_get_num_page_ids(
    start_id: *mut u64,
    stop_id: *mut u64,
    num_pages: *mut u64,
) -> i32 {
    FfiBoundary::run_default(EngineError::Internal.to_mysql_errno(), || {
        let range_start = if start_id.is_null() {
            return EngineError::Internal.to_mysql_errno();
        } else {
            // SAFETY: caller guarantees `start_id` is readable for one u64.
            unsafe { start_id.read() }
        };
        let range_stop = if stop_id.is_null() {
            return EngineError::Internal.to_mysql_errno();
        } else {
            // SAFETY: caller guarantees `stop_id` is readable for one u64.
            unsafe { stop_id.read() }
        };
        let result = match runtime::handlerton() {
            Some(h) => h.page_track_get_num_page_ids(range_start, range_stop),
            None => Err(EngineError::Unsupported),
        };
        report_u64(result, num_pages)
    })
}

/// `page_track_get_status`. The C signature returns a `std::vector` by
/// value; that container cannot be synthesised through the opaque
/// pass-through, so the FFI invokes the trait method (default no-op) but
/// surfaces no output to MySQL.
///
/// # Safety
/// Takes no MySQL-owned pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__page_track_get_status() {
    FfiBoundary::run_void(|| {
        if let Some(h) = runtime::handlerton() {
            h.page_track_get_status();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_u64_some_writes_and_returns_zero() {
        let mut out: u64 = 0;
        assert_eq!(report_u64(Ok(42), &raw mut out), 0);
        assert_eq!(out, 42);
    }

    #[test]
    fn report_u64_null_out_reports_internal_error() {
        assert_eq!(
            report_u64(Ok(7), core::ptr::null_mut()),
            EngineError::Internal.to_mysql_errno()
        );
    }

    #[test]
    fn report_u64_err_propagates_errno() {
        let mut out: u64 = 9;
        assert_eq!(
            report_u64(Err(EngineError::Unsupported), &raw mut out),
            EngineError::Unsupported.to_mysql_errno()
        );
        // Output left untouched on error.
        assert_eq!(out, 9);
    }
}
