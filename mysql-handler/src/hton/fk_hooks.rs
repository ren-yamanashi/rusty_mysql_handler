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

//! `rust__hton__*` foreign-key compatibility check + plugin-observer
//! transaction hooks. All four are always wired on a registered handlerton —
//! the FK check returns a compatibility bool and the `se_*` hooks are
//! observer-style notifications, so neither carries capability semantics.

#![allow(unsafe_code)]

use core::ffi::c_void;

use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::sys;

// Observer hook `arg` is discarded — it belongs to the observer plugin, not
// to the storage engine, so the trait sees no parameter.

/// `check_fk_column_compat`. On panic returns `false` (incompatible) so MySQL
/// rejects rather than silently accepts a bad FK.
///
/// # Safety
/// `child` / `parent` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__check_fk_column_compat(
    child: *const sys::HaFkColumnType,
    parent: *const sys::HaFkColumnType,
    check_charsets: bool,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: child null or valid for read for this call.
        let child_ref = unsafe { child.as_ref() };
        // SAFETY: parent null or valid for read for this call.
        let parent_ref = unsafe { parent.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.check_fk_column_compat(child_ref, parent_ref, check_charsets),
            None => true,
        }
    })
}

/// `se_before_commit`. The observer's `arg` is intentionally discarded.
///
/// # Safety
/// Takes no MySQL-owned pointer the engine needs to dereference; `arg` is
/// opaque to this side of the boundary.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__se_before_commit(_arg: *mut c_void) {
    FfiBoundary::run_void(|| {
        if let Some(h) = runtime::handlerton() {
            h.se_before_commit();
        }
    });
}

/// `se_after_commit`
///
/// # Safety
/// See [`rust__hton__se_before_commit`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__se_after_commit(_arg: *mut c_void) {
    FfiBoundary::run_void(|| {
        if let Some(h) = runtime::handlerton() {
            h.se_after_commit();
        }
    });
}

/// `se_before_rollback`
///
/// # Safety
/// See [`rust__hton__se_before_commit`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__se_before_rollback(_arg: *mut c_void) {
    FfiBoundary::run_void(|| {
        if let Some(h) = runtime::handlerton() {
            h.se_before_rollback();
        }
    });
}
