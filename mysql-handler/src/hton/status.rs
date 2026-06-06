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

//! `rust__hton__*` status / lifecycle callbacks. The shim wires `panic`,
//! `flush_logs`, `show_status`, `upgrade_logs`, `finish_upgrade`,
//! `fill_is_table`, `is_reserved_db_name` unconditionally on a registered
//! handlerton; `start_consistent_snapshot` requires
//! [`HtonCapabilities::TRANSACTIONS`] and `partition_flags` requires
//! [`HtonCapabilities::PARTITIONING`].
//!
//! [`HtonCapabilities::TRANSACTIONS`]: crate::hton::HtonCapabilities::TRANSACTIONS
//! [`HtonCapabilities::PARTITIONING`]: crate::hton::HtonCapabilities::PARTITIONING

#![allow(unsafe_code)]

use core::ffi::c_void;

use crate::hton::{HaPanicFunction, HaStatType, StatPrintSink};
use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::runtime::FfiPtr;
use crate::sys;

/// `ha_panic` shutdown notification.
///
/// # Safety
/// Takes no MySQL-owned pointer; the integer is the `enum ha_panic_function`
/// MySQL passed in.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__panic(flag: u32) -> i32 {
    FfiBoundary::run(|| match runtime::handlerton() {
        Some(h) => h.panic(HaPanicFunction::from_raw(flag)),
        None => Ok(()),
    })
}

/// `start_consistent_snapshot` for `START TRANSACTION WITH CONSISTENT SNAPSHOT`.
///
/// # Safety
/// `thd` is null or a valid `THD` pointer for the call's duration; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__start_consistent_snapshot(thd: *const sys::THD) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: thd is null or valid for read for this call.
        let thd = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.start_consistent_snapshot(thd),
            None => Ok(()),
        }
    })
}

/// `flush_logs`: returns `false` for success in MySQL convention. The shim
/// turns an `EngineResult::Err` into `true` (error) on the C++ side.
///
/// # Safety
/// Takes no MySQL-owned pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__flush_logs(binlog_group_flush: bool) -> bool {
    FfiBoundary::run_default(false, || match runtime::handlerton() {
        Some(h) => match h.flush_logs(binlog_group_flush) {
            Ok(()) => false,
            Err(_) => true,
        },
        None => false,
    })
}

/// `show_status`: build a [`StatPrintSink`] from the opaque `THD*` and
/// `stat_print_fn*` MySQL handed in, then dispatch to the handlerton. Returns
/// `false` for success.
///
/// # Safety
/// `thd` is null or a valid `THD` pointer for the call's duration; `print_fn`
/// is the opaque `stat_print_fn` pointer MySQL handed in (the shim re-casts it
/// before invoking). Neither pointer is retained beyond this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__show_status(
    thd: *const sys::THD,
    print_fn: *const c_void,
    stat: u32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: thd is null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        let sink = StatPrintSink::new(thd_ref, print_fn);
        match runtime::handlerton() {
            Some(h) => match h.show_status(thd_ref, &sink, HaStatType::from_raw(stat)) {
                Ok(()) => false,
                Err(_) => true,
            },
            None => false,
        }
    })
}

/// `partition_flags`: wired only when the handlerton declares the
/// [`PARTITIONING`](crate::hton::HtonCapabilities::PARTITIONING) capability.
///
/// # Safety
/// Takes no MySQL-owned pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__partition_flags() -> u32 {
    FfiBoundary::run_default(0, || match runtime::handlerton() {
        Some(h) => h.partition_flags(),
        None => 0,
    })
}

/// `fill_is_table`: populate engine-defined `INFORMATION_SCHEMA` rows. The
/// `Table_ref*`, `Item*`, and `enum_schema_tables` parameters are opaque to
/// Rust today; the engine sees only `thd`.
///
/// # Safety
/// `thd` is null or a valid `THD` pointer for the call's duration; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__fill_is_table(thd: *const sys::THD) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: thd is null or valid for read for this call.
        let thd = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.fill_is_table(thd),
            None => Ok(()),
        }
    })
}

/// `upgrade_logs`: roll engine log files forward during an in-place upgrade.
///
/// # Safety
/// `thd` is null or a valid `THD` pointer for the call's duration; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__upgrade_logs(thd: *const sys::THD) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: thd is null or valid for read for this call.
        let thd = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.upgrade_logs(thd),
            None => Ok(()),
        }
    })
}

/// `finish_upgrade`: finalize upgrade state regardless of success.
///
/// # Safety
/// `thd` is null or a valid `THD` pointer for the call's duration; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__finish_upgrade(
    thd: *const sys::THD,
    failed_upgrade: bool,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: thd is null or valid for read for this call.
        let thd = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.finish_upgrade(thd, failed_upgrade),
            None => Ok(()),
        }
    })
}

/// `is_reserved_db_name`: whether the engine reserves the given database name.
///
/// # Safety
/// `name` covers `name_len` readable bytes for the call's duration.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_reserved_db_name(name: *const u8, name_len: usize) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: name covers name_len readable bytes for this call.
        let decoded = unsafe { FfiPtr::bytes_to_str(name, name_len) };
        match decoded {
            Ok(s) => match runtime::handlerton() {
                Some(h) => h.is_reserved_db_name(s),
                None => false,
            },
            // An undecodable name was never reserved by the engine; report
            // false rather than reject the lookup outright.
            Err(_) => false,
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::hton::{HaPanicFunction, HaStatType};

    #[test]
    fn panic_function_from_raw_round_trip() {
        assert_eq!(HaPanicFunction::from_raw(0), HaPanicFunction::Close);
        assert_eq!(HaPanicFunction::from_raw(1), HaPanicFunction::Write);
    }

    #[test]
    fn stat_type_from_raw_round_trip() {
        assert_eq!(HaStatType::from_raw(0), HaStatType::Status);
        assert_eq!(HaStatType::from_raw(2), HaStatType::Mutex);
    }
}
