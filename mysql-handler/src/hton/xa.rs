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

//! `rust__hton__*` XA recovery callbacks delegating to the engine-level
//! handlerton singleton. The shim wires these only under the XA capability.
//! `recover_prepared_in_tc` and `recover` are bound through writer handles
//! ([`XaStateListCollector`](crate::hton::XaStateListCollector) and
//! [`XaRecoverCollector`](crate::hton::XaRecoverCollector)) that the engine
//! pushes prepared XA transactions into; the shim copies each entry into
//! a `XID` and forwards it to the corresponding MySQL container. The shim
//! maps the `EngineResult` returned here to the `xa_status_code` MySQL
//! expects for the by-xid callbacks.

#![allow(unsafe_code)]

use core::ffi::c_void;

use crate::hton::{XaRecoverCollector, XaStateListCollector};
use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::sys;

/// Commit a prepared XA transaction by its (opaque) XID.
///
/// # Safety
/// `xid` is null or a valid `XID` pointer for the call's duration; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__commit_by_xid(xid: *const sys::XID) -> i32 {
    FfiBoundary::run(|| match runtime::handlerton() {
        // SAFETY: xid is null or valid for read for this call.
        Some(h) => h.commit_by_xid(unsafe { xid.as_ref() }),
        None => Ok(()),
    })
}

/// Roll back a prepared XA transaction by its (opaque) XID.
///
/// # Safety
/// `xid` is null or a valid `XID` pointer for the call's duration; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__rollback_by_xid(xid: *const sys::XID) -> i32 {
    FfiBoundary::run(|| match runtime::handlerton() {
        // SAFETY: xid is null or valid for read for this call.
        Some(h) => h.rollback_by_xid(unsafe { xid.as_ref() }),
        None => Ok(()),
    })
}

/// Mark the connection's externally-coordinated transactions prepared in the TC.
///
/// # Safety
/// `thd` is null or a valid `THD` pointer for the call's duration; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__set_prepared_in_tc(thd: *const sys::THD) -> i32 {
    FfiBoundary::run(|| match runtime::handlerton() {
        // SAFETY: thd is null or valid for read for this call.
        Some(h) => h.set_prepared_in_tc(unsafe { thd.as_ref() }),
        None => Ok(()),
    })
}

/// Mark the prepared XA transaction identified by `xid` prepared in the TC.
///
/// # Safety
/// `xid` is null or a valid `XID` pointer for the call's duration; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__set_prepared_in_tc_by_xid(xid: *const sys::XID) -> i32 {
    FfiBoundary::run(|| match runtime::handlerton() {
        // SAFETY: xid is null or valid for read for this call.
        Some(h) => h.set_prepared_in_tc_by_xid(unsafe { xid.as_ref() }),
        None => Ok(()),
    })
}

/// Report every transaction the engine considers prepared in the TC by
/// pushing it into the `Xa_state_list` `xa_list` references.
///
/// # Safety
/// `xa_list` is a valid `Xa_state_list *` for the call's duration; not
/// retained. The engine writes into it only through the safe
/// `XaStateListCollector` wrapper.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__recover_prepared_in_tc(xa_list: *mut c_void) -> i32 {
    FfiBoundary::run(|| match runtime::handlerton() {
        Some(h) => {
            let mut collector = XaStateListCollector::new(xa_list);
            h.recover_prepared_in_tc(&mut collector)
        }
        None => Ok(()),
    })
}

/// Fill `xid_list[0..len]` with prepared XA transactions; returns the
/// count actually written (0 when no engine is registered).
///
/// # Safety
/// `xid_list` is a valid `XA_recover_txn *` array of length `len`, valid
/// for the call's duration; the engine writes only via the safe
/// `XaRecoverCollector` wrapper.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__recover(xid_list: *mut c_void, len: u32) -> u32 {
    FfiBoundary::run_default(0, || match runtime::handlerton() {
        Some(h) => {
            let mut collector = XaRecoverCollector::new(xid_list, len);
            h.recover(&mut collector);
            collector.filled()
        }
        None => 0,
    })
}
