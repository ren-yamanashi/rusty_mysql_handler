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

//! `rust__hton__*` core engine-lifecycle callbacks. The shim wires these onto
//! the `handlerton` only when an engine-level handlerton is registered; each
//! delegates to the singleton, treating an unregistered handlerton as no-op.

#![allow(unsafe_code)]

use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::sys;

/// Connection close: release any per-connection engine state.
///
/// # Safety
/// `thd` is null or a valid `THD` pointer for the duration of the call; it is
/// not retained past it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__close_connection(thd: *const sys::THD) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: thd is null or valid for read for the call's duration.
        let thd = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.close_connection(thd),
            None => Ok(()),
        }
    })
}

/// Connection / statement kill notification.
///
/// # Safety
/// `thd` is null or a valid `THD` pointer for the duration of the call; it is
/// not retained past it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__kill_connection(thd: *const sys::THD) {
    FfiBoundary::run_void(|| {
        // SAFETY: thd is null or valid for read for the call's duration.
        let thd = unsafe { thd.as_ref() };
        if let Some(h) = runtime::handlerton() {
            h.kill_connection(thd);
        }
    });
}

/// Pre-data-dictionary shutdown notification.
///
/// # Safety
/// Takes no MySQL-owned pointer; safe to call after `rust__plugin_init`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__pre_dd_shutdown() {
    FfiBoundary::run_void(|| {
        if let Some(h) = runtime::handlerton() {
            h.pre_dd_shutdown();
        }
    });
}

/// Session plugin-variable reset before a connection ends.
///
/// # Safety
/// `thd` is null or a valid `THD` pointer for the duration of the call; it is
/// not retained past it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__reset_plugin_vars(thd: *const sys::THD) {
    FfiBoundary::run_void(|| {
        // SAFETY: thd is null or valid for read for the call's duration.
        let thd = unsafe { thd.as_ref() };
        if let Some(h) = runtime::handlerton() {
            h.reset_plugin_vars(thd);
        }
    });
}
