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

//! Engine-facing handle to MySQL's `stat_print_fn` for emitting
//! `SHOW ENGINE <name> STATUS` rows.

#![allow(unsafe_code)]

use core::ffi::c_void;

use crate::sys;

unsafe extern "C" {
    fn mysql__hton__emit_status_row(
        thd: *const c_void,
        print_fn: *const c_void,
        kind: *const u8,
        kind_len: usize,
        key: *const u8,
        key_len: usize,
        value: *const u8,
        value_len: usize,
    ) -> bool;
}

/// A handle that lets a [`Handlerton::show_status`] implementation push one or
/// more rows of engine status back to MySQL. Bound to the connection and
/// `stat_print_fn` MySQL handed in, so it does not outlive the callback.
///
/// Each [`Self::emit`] call surfaces as one row of `SHOW ENGINE <name> STATUS`:
/// `kind` is the row's type (typically the engine name), `key` a sub-label, and
/// `value` the displayed text.
///
/// [`Handlerton::show_status`]: crate::hton::Handlerton::show_status
#[derive(Debug)]
pub struct StatPrintSink<'a> {
    thd: Option<&'a sys::THD>,
    print_fn: *const c_void,
}

impl<'a> StatPrintSink<'a> {
    /// Bind a sink to the connection and `stat_print_fn` MySQL handed in.
    /// `print_fn` is opaque to Rust; the reverse callback re-casts it before
    /// invoking it on the C++ side. Crate-private: external `Handlerton`
    /// implementors receive a sink as a parameter and call [`Self::emit`];
    /// they never construct one.
    #[must_use]
    pub(crate) fn new(thd: Option<&'a sys::THD>, print_fn: *const c_void) -> Self {
        Self { thd, print_fn }
    }

    /// Push one row. Returns `true` when MySQL accepted the row, `false` when
    /// the print function reported back-pressure / failure.
    #[must_use]
    pub fn emit(&self, kind: &str, key: &str, value: &str) -> bool {
        if self.print_fn.is_null() {
            return false;
        }
        let thd = match self.thd {
            Some(t) => std::ptr::from_ref(t).cast::<c_void>(),
            None => std::ptr::null(),
        };
        // SAFETY: the shim implements mysql__hton__emit_status_row; the byte
        // pointers cover their stated lengths for the duration of this call.
        let ok = unsafe {
            mysql__hton__emit_status_row(
                thd,
                self.print_fn,
                kind.as_ptr(),
                kind.len(),
                key.as_ptr(),
                key.len(),
                value.as_ptr(),
                value.len(),
            )
        };
        // stat_print_fn returns true on error in MySQL convention; invert so
        // Rust callers can write `if !sink.emit(...) { ... }` on failure.
        !ok
    }
}
