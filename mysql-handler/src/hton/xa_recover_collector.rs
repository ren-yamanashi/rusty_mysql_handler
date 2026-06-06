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

//! Writer handle the engine uses to push prepared XA transactions into
//! MySQL's `XA_recover_txn[]` array during `recover`.

#![allow(unsafe_code)]

use core::ffi::c_void;

unsafe extern "C" {
    fn mysql__xa_recover__set_entry(
        xid_list: *mut c_void,
        index: u32,
        format_id: i64,
        gtrid_ptr: *const u8,
        gtrid_len: usize,
        bqual_ptr: *const u8,
        bqual_len: usize,
    );
}

/// Bounded writer handle into the MySQL-owned `XA_recover_txn[]` array
/// the server passes to `recover`. The engine pushes prepared XIDs via
/// [`Self::add`]; the shim writes each one into the next free slot and
/// sets the slot's `mod_tables` pointer to NULL (engines do not yet
/// expose the modified-tables list). [`Self::filled`] is what the FFI
/// callback returns to MySQL as the recovered-transaction count.
#[derive(Debug)]
pub struct XaRecoverCollector {
    xid_list: *mut c_void,
    capacity: u32,
    filled: u32,
}

impl XaRecoverCollector {
    /// Construct a collector that writes into `xid_list[0..capacity]`.
    /// The pointer must outlive every [`Self::add`] call (it always
    /// does — the shim only hands it to the engine for the duration
    /// of one `recover` callback).
    pub(crate) fn new(xid_list: *mut c_void, capacity: u32) -> Self {
        Self {
            xid_list,
            capacity,
            filled: 0,
        }
    }

    /// Push one prepared XA transaction into the next free slot. Returns
    /// `false` when the array is already full (the engine should stop
    /// recovering in that case; MySQL will call back with a larger
    /// buffer if more transactions are expected).
    pub fn add(&mut self, format_id: i64, gtrid: &[u8], bqual: &[u8]) -> bool {
        if self.filled >= self.capacity {
            return false;
        }
        // SAFETY: xid_list is the MySQL-owned array valid for the
        // duration of this call; the shim only reads from gtrid/bqual.
        unsafe {
            mysql__xa_recover__set_entry(
                self.xid_list,
                self.filled,
                format_id,
                gtrid.as_ptr(),
                gtrid.len(),
                bqual.as_ptr(),
                bqual.len(),
            );
        }
        self.filled += 1;
        true
    }

    /// The number of XIDs pushed so far. Returned from the FFI callback.
    #[must_use]
    pub fn filled(&self) -> u32 {
        self.filled
    }
}
