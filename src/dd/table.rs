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

//! [`DdTable`] safe accessors over the column and index collections.

#![allow(unsafe_code)]

use crate::dd::ffi;
use crate::sys::{DdColumn, DdIndex, DdTable};

impl DdTable {
    /// Number of columns in the table.
    #[must_use]
    pub fn column_count(&self) -> usize {
        let p: *const DdTable = self;
        // SAFETY: `self` is a valid borrow.
        unsafe { ffi::mysql__DdTable__column_count(p) }
    }

    /// Borrow the `i`th column (0-based). Returns `None` past the end.
    #[must_use]
    pub fn column_at(&self, i: usize) -> Option<&DdColumn> {
        let p: *const DdTable = self;
        // SAFETY: `self` is a valid borrow; the FFI returns null when `i`
        // is out of range or `p` is null.
        let raw = unsafe { ffi::mysql__DdTable__column_at(p, i) };
        // SAFETY: a non-null result references a column owned by the same
        // dd::Table tree as `self` and is valid for `self`'s lifetime.
        unsafe { raw.as_ref() }
    }

    /// Number of indexes in the table.
    #[must_use]
    pub fn index_count(&self) -> usize {
        let p: *const DdTable = self;
        // SAFETY: `self` is a valid borrow.
        unsafe { ffi::mysql__DdTable__index_count(p) }
    }

    /// Borrow the `i`th index (0-based). Returns `None` past the end.
    #[must_use]
    pub fn index_at(&self, i: usize) -> Option<&DdIndex> {
        let p: *const DdTable = self;
        // SAFETY: `self` is a valid borrow; the FFI returns null when `i`
        // is out of range or `p` is null.
        let raw = unsafe { ffi::mysql__DdTable__index_at(p, i) };
        // SAFETY: a non-null result references an index owned by the same
        // dd::Table tree as `self` and is valid for `self`'s lifetime.
        unsafe { raw.as_ref() }
    }
}
