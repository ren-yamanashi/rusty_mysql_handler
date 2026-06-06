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

//! Per-capability `rust__hton__is_*` accessors. Split from
//! [`super::ffi`] because the capability surface grew past the per-file
//! line cap as more handlerton groups were bound; the shim's `rusty_init_func`
//! still imports both modules through `shim/rust_callbacks/hton_core.hpp`.
//!
//! Each accessor reads the process-wide handlerton singleton and returns
//! `false` when no handlerton is registered, so a zero-config engine keeps
//! every capability-gated wire path off as before.

#![allow(unsafe_code)]

use crate::hton::HtonCapabilities;
use crate::panic_guard::FfiBoundary;
use crate::runtime;

fn has(cap: HtonCapabilities) -> bool {
    match runtime::handlerton() {
        Some(h) => h.capabilities().contains(cap),
        None => false,
    }
}

/// Whether the registered handlerton declares
/// [`HtonCapabilities::TRANSACTIONS`](crate::hton::HtonCapabilities::TRANSACTIONS).
///
/// # Safety
/// Call after `rust__plugin_init`. Reads the process-wide handlerton singleton.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_transactional() -> bool {
    FfiBoundary::run_default(false, || has(HtonCapabilities::TRANSACTIONS))
}

/// `HtonCapabilities::XA` — gates `commit_by_xid` / `rollback_by_xid` /
/// `set_prepared_in_tc[_by_xid]`.
///
/// # Safety
/// Call after `rust__plugin_init`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_xa() -> bool {
    FfiBoundary::run_default(false, || has(HtonCapabilities::XA))
}

/// `HtonCapabilities::SAVEPOINTS` — gates the savepoint callbacks and the
/// `savepoint_offset` field.
///
/// # Safety
/// Call after `rust__plugin_init`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_savepoints() -> bool {
    FfiBoundary::run_default(false, || has(HtonCapabilities::SAVEPOINTS))
}

/// `HtonCapabilities::PARTITIONING` — gates `partition_flags`.
///
/// # Safety
/// Call after `rust__plugin_init`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_partitioning() -> bool {
    FfiBoundary::run_default(false, || has(HtonCapabilities::PARTITIONING))
}

/// `HtonCapabilities::TABLESPACES` — gates the tablespace callbacks.
///
/// # Safety
/// Call after `rust__plugin_init`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_tablespaces() -> bool {
    FfiBoundary::run_default(false, || has(HtonCapabilities::TABLESPACES))
}

/// `HtonCapabilities::DICT_BACKEND` — gates the `dict_*` callbacks.
///
/// # Safety
/// Call after `rust__plugin_init`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_dict_backend() -> bool {
    FfiBoundary::run_default(false, || has(HtonCapabilities::DICT_BACKEND))
}

/// `HtonCapabilities::SDI` — gates the `sdi_*` callbacks.
///
/// # Safety
/// Call after `rust__plugin_init`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_sdi() -> bool {
    FfiBoundary::run_default(false, || has(HtonCapabilities::SDI))
}

/// `HtonCapabilities::ENGINE_LOG` — gates `lock_hton_log` / `unlock_hton_log`
/// / `collect_hton_log_info` / `redo_log_set_state`.
///
/// # Safety
/// Call after `rust__plugin_init`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_engine_log() -> bool {
    FfiBoundary::run_default(false, || has(HtonCapabilities::ENGINE_LOG))
}

/// `HtonCapabilities::SECONDARY_ENGINE` — gates the ten secondary-engine
/// callbacks.
///
/// # Safety
/// Call after `rust__plugin_init`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_secondary_engine() -> bool {
    FfiBoundary::run_default(false, || has(HtonCapabilities::SECONDARY_ENGINE))
}

/// `HtonCapabilities::ENCRYPTION` — gates `rotate_encryption_master_key`.
///
/// # Safety
/// Call after `rust__plugin_init`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_encryption() -> bool {
    FfiBoundary::run_default(false, || has(HtonCapabilities::ENCRYPTION))
}

/// `HtonCapabilities::CLONE` — gates the eight `clone_interface`
/// sub-callbacks.
///
/// # Safety
/// Call after `rust__plugin_init`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_clone() -> bool {
    FfiBoundary::run_default(false, || has(HtonCapabilities::CLONE))
}

/// `HtonCapabilities::PAGE_TRACKING` — gates the six `page_track`
/// sub-callbacks.
///
/// # Safety
/// Call after `rust__plugin_init`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_page_tracking() -> bool {
    FfiBoundary::run_default(false, || has(HtonCapabilities::PAGE_TRACKING))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_handlerton_reports_no_capability() {
        // The process-wide handlerton singleton is normally set during
        // `rust__plugin_init`; in unit tests it stays None and `has` must
        // safely return false.
        assert!(!has(HtonCapabilities::TRANSACTIONS));
        assert!(!has(HtonCapabilities::CLONE));
    }
}
