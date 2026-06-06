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

//! Writer handle the engine uses to push prepared-in-TC XA transactions
//! into MySQL's `Xa_state_list` during `recover_prepared_in_tc`.

#![allow(unsafe_code)]

use core::ffi::c_void;

use crate::hton::RecoverXaState;

unsafe extern "C" {
    fn mysql__xa_state_list__add(
        xa_list: *mut c_void,
        format_id: i64,
        gtrid_ptr: *const u8,
        gtrid_len: usize,
        bqual_ptr: *const u8,
        bqual_len: usize,
        state: i32,
    );
}

/// Writeable handle into MySQL's `Xa_state_list`. The engine constructs
/// XIDs from its persistent storage and pushes each (XID, state) pair via
/// [`Self::add`]; the shim copies the data into a `XID` instance and calls
/// `Xa_state_list::add` on the underlying map.
#[derive(Debug)]
pub struct XaStateListCollector {
    xa_list: *mut c_void,
}

impl XaStateListCollector {
    /// Construct a collector that writes into the MySQL-owned
    /// `Xa_state_list` pointed to by `xa_list`. The pointer must outlive
    /// every call to [`Self::add`] (it always does — the shim only hands
    /// it to the engine for the duration of a single
    /// `recover_prepared_in_tc` callback).
    pub(crate) fn new(xa_list: *mut c_void) -> Self {
        Self { xa_list }
    }

    /// Push a single prepared XA transaction into the list. `gtrid` and
    /// `bqual` are the X/Open distributed transaction identifier and
    /// branch qualifier; together they must fit within `XIDDATASIZE`
    /// (128 bytes) and each individually within 64 bytes per the X/Open
    /// spec. The shim copies the data into a fresh `XID` and forwards
    /// it to `Xa_state_list::add`.
    pub fn add(&mut self, format_id: i64, gtrid: &[u8], bqual: &[u8], state: RecoverXaState) {
        // SAFETY: xa_list is the MySQL-owned pointer the shim handed us
        // for this callback only; the slice pointers point into Rust
        // memory the C++ side only reads (and copies) before returning.
        unsafe {
            mysql__xa_state_list__add(
                self.xa_list,
                format_id,
                gtrid.as_ptr(),
                gtrid.len(),
                bqual.as_ptr(),
                bqual.len(),
                state.to_raw(),
            );
        }
    }
}
