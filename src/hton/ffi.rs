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

//! Core `rust__hton__*` accessors the shim queries from `rusty_init_func` to
//! populate the `handlerton` struct: the engine's flag bitfield, whether a
//! handlerton is registered, and the savepoint scratch size. The
//! capability-bit `is_*` accessors live next door in
//! [`super::capability_ffi`].

#![allow(unsafe_code)]

use crate::hton::HtonFlags;
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
