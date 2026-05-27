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

//! `rust__handler__*` callbacks for read-before-write removal and
//! auto-increment methods (handler.h #110–#113). Shares the FFI safety contract
//! documented at [`crate::handler`].
//!
//! The read-removal and `get_auto_increment` callbacks return `true` when the
//! engine overrides (values written through the out-pointers) and `false` to
//! fall back to the handler base; `release_auto_increment` is a plain void
//! delegation.

#![allow(unsafe_code)]

use super::report::{report_bool, report_u64};
use crate::panic_guard::FfiBoundary;
use crate::runtime::EngineContext;

// Write an engine-supplied (first_value, nb_reserved) auto-increment block
// through the out-pointers; report whether the engine supplied one.
fn report_auto_increment(
    first_value: *mut u64,
    nb_reserved: *mut u64,
    value: Option<(u64, u64)>,
) -> bool {
    match value {
        Some((first, reserved)) => {
            // SAFETY: each out-pointer is writable for one u64 when non-null.
            unsafe {
                if !first_value.is_null() {
                    *first_value = first;
                }
                if !nb_reserved.is_null() {
                    *nb_reserved = reserved;
                }
            }
            true
        }
        None => false,
    }
}

/// Begin read-before-write removal; returns whether the engine handled it
///
/// # Safety
/// `ctx` non-null; `out` writable for one `bool` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__start_read_removal(
    ctx: *mut EngineContext,
    out: *mut bool,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_bool(out, unsafe { &mut *ctx }.engine_mut().start_read_removal())
    })
}

/// End read-before-write removal; returns whether the engine handled it
///
/// # Safety
/// `ctx` non-null; `out` writable for one `u64` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__end_read_removal(
    ctx: *mut EngineContext,
    out: *mut u64,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_u64(out, unsafe { &mut *ctx }.engine_mut().end_read_removal())
    })
}

/// Reserve an auto-increment block; returns whether the engine handled it
///
/// # Safety
/// `ctx` non-null; `first_value` and `nb_reserved` writable for one `u64` each
/// when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__get_auto_increment(
    ctx: *mut EngineContext,
    offset: u64,
    increment: u64,
    nb_desired: u64,
    first_value: *mut u64,
    nb_reserved: *mut u64,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let value = unsafe { &mut *ctx }
            .engine_mut()
            .get_auto_increment(offset, increment, nb_desired);
        report_auto_increment(first_value, nb_reserved, value)
    })
}

/// Release reserved-but-unused auto-increment values
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__release_auto_increment(ctx: *mut EngineContext) {
    FfiBoundary::run_void(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().release_auto_increment();
    });
}

#[cfg(test)]
mod tests {
    use super::report_auto_increment;

    #[test]
    fn report_auto_increment_writes_block_and_signals_handled() {
        let (mut first, mut reserved) = (0, 0);
        assert!(report_auto_increment(
            &mut first,
            &mut reserved,
            Some((10, 5))
        ));
        assert_eq!((first, reserved), (10, 5));
    }

    #[test]
    fn report_auto_increment_none_leaves_buffers_and_signals_unhandled() {
        let (mut first, mut reserved) = (7, 9);
        assert!(!report_auto_increment(&mut first, &mut reserved, None));
        assert_eq!((first, reserved), (7, 9));
    }

    #[test]
    fn report_auto_increment_tolerates_null_out_pointers() {
        let null = core::ptr::null_mut();
        assert!(report_auto_increment(null, null, Some((1, 1))));
    }
}
