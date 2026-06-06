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
//! `recover` and `recover_prepared_in_tc` are deferred (not impossible):
//! they fill MySQL-owned output structures and need push-entry reverse
//! callbacks the shim does not yet provide. A NULL pointer currently
//! reports "nothing to recover", which is correct only because no engine
//! reaches this code path yet. The shim maps the `EngineResult` returned
//! here to the `xa_status_code` MySQL expects for the by-xid callbacks.

#![allow(unsafe_code)]

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
