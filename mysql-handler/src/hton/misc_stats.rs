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

//! `rust__hton__*` miscellaneous encryption / redo / DDL callbacks.
//!
//! - `rotate_encryption_master_key` is wired only under
//!   [`HtonCapabilities::ENCRYPTION`].
//! - `redo_log_set_state` is wired only under
//!   [`HtonCapabilities::ENGINE_LOG`].
//! - `post_ddl` and `post_recover` are always wired on a registered
//!   handlerton (notifications).
//!
//! The three statistics callbacks live in
//! [`crate::hton::statistics_callbacks`].
//!
//! [`HtonCapabilities::ENCRYPTION`]: crate::hton::HtonCapabilities::ENCRYPTION
//! [`HtonCapabilities::ENGINE_LOG`]: crate::hton::HtonCapabilities::ENGINE_LOG

#![allow(unsafe_code)]

use crate::hton::result::result_to_error;
use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::sys;

/// # Safety
/// Takes no MySQL-owned pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__rotate_encryption_master_key() -> bool {
    FfiBoundary::run_default(true, || match runtime::handlerton() {
        Some(h) => result_to_error(h.rotate_encryption_master_key()),
        None => false,
    })
}

/// # Safety
/// `thd` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__redo_log_set_state(
    thd: *const sys::THD,
    enable: bool,
) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => result_to_error(h.redo_log_set_state(thd_ref, enable)),
            None => false,
        }
    })
}

/// # Safety
/// `thd` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__post_ddl(thd: *const sys::THD) {
    FfiBoundary::run_void(|| {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        if let Some(h) = runtime::handlerton() {
            h.post_ddl(thd_ref);
        }
    });
}

/// # Safety
/// Takes no MySQL-owned pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__post_recover() {
    FfiBoundary::run_void(|| {
        if let Some(h) = runtime::handlerton() {
            h.post_recover();
        }
    });
}
