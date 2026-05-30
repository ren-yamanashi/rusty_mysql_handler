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

//! `rust__hton__*` callbacks: the C++ shim queries the registered handlerton
//! singleton through these to populate the `handlerton` struct in
//! `rusty_init_func`.

#![allow(unsafe_code)]

use crate::hton::{HtonCapabilities, HtonFlags};
use crate::panic_guard::FfiBoundary;
use crate::runtime;

/// The `handlerton` flags (`HTON_*`) `rusty_init_func` should set.
///
/// # Safety
/// Call after `rust__plugin_init`. Reads the process-wide handlerton singleton
/// and returns the zero-config default ([`HtonFlags::CAN_RECREATE`]) when no
/// handlerton is registered, so an engine that skips registration is wired
/// exactly as before.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__flags() -> u32 {
    FfiBoundary::run_default(
        HtonFlags::CAN_RECREATE.bits(),
        || match runtime::handlerton() {
            Some(h) => h.flags().bits(),
            None => HtonFlags::CAN_RECREATE.bits(),
        },
    )
}

/// Whether an engine-level [`Handlerton`](crate::hton::Handlerton) is
/// registered.
///
/// `rusty_init_func` uses this to decide whether to wire the always-on hook
/// callbacks (`close_connection`, `kill_connection`, ...): a zero-config
/// engine that registers no handlerton keeps those pointers NULL, exactly as
/// before.
///
/// # Safety
/// Call after `rust__plugin_init`. Reads the process-wide handlerton singleton.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_registered() -> bool {
    FfiBoundary::run_default(false, || runtime::handlerton().is_some())
}

/// Whether the registered handlerton declares
/// [`HtonCapabilities::TRANSACTIONS`](crate::hton::HtonCapabilities::TRANSACTIONS).
///
/// `rusty_init_func` uses this to gate the transaction callbacks
/// (`commit` / `rollback` / `prepare`), and the handler's `external_lock` uses
/// it to decide whether to register the engine in the transaction. Non-NULL
/// `commit` is what tells MySQL the engine is transactional, so this must be
/// false unless the engine truly implements transactions.
///
/// # Safety
/// Call after `rust__plugin_init`. Reads the process-wide handlerton singleton.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_transactional() -> bool {
    FfiBoundary::run_default(false, || match runtime::handlerton() {
        Some(h) => h.capabilities().contains(HtonCapabilities::TRANSACTIONS),
        None => false,
    })
}

/// Whether the registered handlerton declares
/// [`HtonCapabilities::XA`](crate::hton::HtonCapabilities::XA).
///
/// `rusty_init_func` uses this to gate the XA recovery callbacks
/// (`commit_by_xid` / `rollback_by_xid` / `set_prepared_in_tc[_by_xid]`).
///
/// # Safety
/// Call after `rust__plugin_init`. Reads the process-wide handlerton singleton.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_xa() -> bool {
    FfiBoundary::run_default(false, || match runtime::handlerton() {
        Some(h) => h.capabilities().contains(HtonCapabilities::XA),
        None => false,
    })
}

/// Whether the registered handlerton declares
/// [`HtonCapabilities::SAVEPOINTS`](crate::hton::HtonCapabilities::SAVEPOINTS).
///
/// `rusty_init_func` uses this to gate the savepoint callbacks and the
/// `savepoint_offset` field.
///
/// # Safety
/// Call after `rust__plugin_init`. Reads the process-wide handlerton singleton.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_savepoints() -> bool {
    FfiBoundary::run_default(false, || match runtime::handlerton() {
        Some(h) => h.capabilities().contains(HtonCapabilities::SAVEPOINTS),
        None => false,
    })
}

/// The `handlerton` `savepoint_offset`: bytes of per-savepoint scratch the
/// engine needs. `rusty_init_func` sets it only when the engine declares
/// `SAVEPOINTS`; 0 otherwise.
///
/// # Safety
/// Call after `rust__plugin_init`. Reads the process-wide handlerton singleton.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__savepoint_offset() -> u32 {
    FfiBoundary::run_default(0, || match runtime::handlerton() {
        Some(h) => h.savepoint_offset(),
        None => 0,
    })
}

/// Whether the registered handlerton declares
/// [`HtonCapabilities::PARTITIONING`](crate::hton::HtonCapabilities::PARTITIONING).
///
/// `rusty_init_func` uses this to gate the `partition_flags` accessor on the
/// handlerton: a non-NULL pointer there is what tells MySQL the engine
/// implements `handler::get_partition_handler`, so leave it NULL unless the
/// engine actually does.
///
/// # Safety
/// Call after `rust__plugin_init`. Reads the process-wide handlerton singleton.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_partitioning() -> bool {
    FfiBoundary::run_default(false, || match runtime::handlerton() {
        Some(h) => h.capabilities().contains(HtonCapabilities::PARTITIONING),
        None => false,
    })
}

/// Whether the registered handlerton declares
/// [`HtonCapabilities::TABLESPACES`](crate::hton::HtonCapabilities::TABLESPACES).
///
/// Gates the tablespace callbacks. A non-tablespace engine must keep them
/// unwired so MySQL does not try to route tablespace work here.
///
/// # Safety
/// Call after `rust__plugin_init`. Reads the process-wide handlerton singleton.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_tablespaces() -> bool {
    FfiBoundary::run_default(false, || match runtime::handlerton() {
        Some(h) => h.capabilities().contains(HtonCapabilities::TABLESPACES),
        None => false,
    })
}

/// Whether the registered handlerton declares
/// [`HtonCapabilities::DICT_BACKEND`](crate::hton::HtonCapabilities::DICT_BACKEND).
///
/// Gates the `dict_*` callbacks; only the storage engine acting as the data
/// dictionary backend may declare this.
///
/// # Safety
/// Call after `rust__plugin_init`. Reads the process-wide handlerton singleton.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_dict_backend() -> bool {
    FfiBoundary::run_default(false, || match runtime::handlerton() {
        Some(h) => h.capabilities().contains(HtonCapabilities::DICT_BACKEND),
        None => false,
    })
}

/// Whether the registered handlerton declares
/// [`HtonCapabilities::SDI`](crate::hton::HtonCapabilities::SDI).
///
/// Gates the `sdi_*` callbacks; declared by engines that own their SDI
/// (InnoDB-style).
///
/// # Safety
/// Call after `rust__plugin_init`. Reads the process-wide handlerton singleton.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_sdi() -> bool {
    FfiBoundary::run_default(false, || match runtime::handlerton() {
        Some(h) => h.capabilities().contains(HtonCapabilities::SDI),
        None => false,
    })
}

/// Whether the registered handlerton declares
/// [`HtonCapabilities::ENGINE_LOG`](crate::hton::HtonCapabilities::ENGINE_LOG).
///
/// Gates the `lock_hton_log` / `unlock_hton_log` / `collect_hton_log_info`
/// callbacks used by `performance_schema.log_status`.
///
/// # Safety
/// Call after `rust__plugin_init`. Reads the process-wide handlerton singleton.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_engine_log() -> bool {
    FfiBoundary::run_default(false, || match runtime::handlerton() {
        Some(h) => h.capabilities().contains(HtonCapabilities::ENGINE_LOG),
        None => false,
    })
}
