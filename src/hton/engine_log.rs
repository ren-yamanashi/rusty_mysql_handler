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

//! `rust__hton__*` engine-log callbacks. Wired only under
//! [`HtonCapabilities::ENGINE_LOG`]; declared by engines whose log surface is
//! collected by `performance_schema.log_status`. Each returns `bool` with
//! MySQL's "true = error" convention.
//!
//! [`HtonCapabilities::ENGINE_LOG`]: crate::hton::HtonCapabilities::ENGINE_LOG

#![allow(unsafe_code)]

use crate::hton::result::result_to_error;
use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::sys;

/// # Safety
/// Takes no MySQL-owned pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__lock_hton_log() -> bool {
    FfiBoundary::run_default(true, || match runtime::handlerton() {
        Some(h) => result_to_error(h.lock_hton_log()),
        None => false,
    })
}

/// # Safety
/// Takes no MySQL-owned pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__unlock_hton_log() -> bool {
    FfiBoundary::run_default(true, || match runtime::handlerton() {
        Some(h) => result_to_error(h.unlock_hton_log()),
        None => false,
    })
}

/// # Safety
/// `json` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__collect_hton_log_info(json: *const sys::JsonDom) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: json null or valid for read for this call.
        let json_ref = unsafe { json.as_ref() };
        match runtime::handlerton() {
            Some(h) => result_to_error(h.collect_hton_log_info(json_ref)),
            None => false,
        }
    })
}
