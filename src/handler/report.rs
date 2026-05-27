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

//! Out-pointer write helpers shared by the base-fallback capability callbacks.
//! Each writes `value` through `out` (when both are present) and reports to the
//! shim whether the engine supplied a value (`true`) or wants the handler base
//! default (`false`).

#![allow(unsafe_code)]

pub(super) fn report_bool(out: *mut bool, value: Option<bool>) -> bool {
    match value {
        Some(v) => {
            if !out.is_null() {
                // SAFETY: out is writable for one bool when non-null.
                unsafe { *out = v };
            }
            true
        }
        None => false,
    }
}

pub(super) fn report_i32(out: *mut i32, value: Option<i32>) -> bool {
    match value {
        Some(v) => {
            if !out.is_null() {
                // SAFETY: out is writable for one i32 when non-null.
                unsafe { *out = v };
            }
            true
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{report_bool, report_i32};

    #[test]
    fn report_bool_writes_and_signals_handled() {
        let mut out = false;
        assert!(report_bool(&mut out, Some(true)));
        assert!(out);
    }

    #[test]
    fn report_bool_none_leaves_buffer_and_signals_unhandled() {
        let mut out = true;
        assert!(!report_bool(&mut out, None));
        assert!(out);
    }

    #[test]
    fn report_i32_writes_and_signals_handled() {
        let mut out = 0;
        assert!(report_i32(&mut out, Some(-5)));
        assert_eq!(out, -5);
    }

    #[test]
    fn report_i32_none_leaves_buffer_and_signals_unhandled() {
        let mut out = 9;
        assert!(!report_i32(&mut out, None));
        assert_eq!(out, 9);
    }

    #[test]
    fn report_helpers_tolerate_null_out() {
        assert!(report_bool(core::ptr::null_mut(), Some(true)));
        assert!(report_i32(core::ptr::null_mut(), Some(1)));
    }
}
