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

//! Raw C declarations for `mysql__Dd*__*` accessors implemented in
//! `shim/dd_table.cc`. All pointer parameters refer to MySQL-owned opaque
//! types and must outlive the call.

#![allow(unsafe_code)]

use crate::sys::{DdColumn, DdIndex, DdIndexElement, DdTable};

unsafe extern "C" {
    pub(super) fn mysql__DdTable__column_count(table: *const DdTable) -> usize;
    pub(super) fn mysql__DdTable__column_at(table: *const DdTable, i: usize) -> *const DdColumn;
    pub(super) fn mysql__DdTable__index_count(table: *const DdTable) -> usize;
    pub(super) fn mysql__DdTable__index_at(table: *const DdTable, i: usize) -> *const DdIndex;

    pub(super) fn mysql__DdColumn__name(column: *const DdColumn, buf: *mut u8, cap: usize)
    -> usize;
    pub(super) fn mysql__DdColumn__type(column: *const DdColumn) -> i32;
    pub(super) fn mysql__DdColumn__is_nullable(column: *const DdColumn) -> bool;
    pub(super) fn mysql__DdColumn__is_unsigned(column: *const DdColumn) -> bool;
    pub(super) fn mysql__DdColumn__char_length(column: *const DdColumn) -> u32;
    pub(super) fn mysql__DdColumn__is_hidden(column: *const DdColumn) -> bool;
    pub(super) fn mysql__DdColumn__ordinal_position(column: *const DdColumn) -> u32;

    pub(super) fn mysql__DdIndex__name(index: *const DdIndex, buf: *mut u8, cap: usize) -> usize;
    pub(super) fn mysql__DdIndex__type(index: *const DdIndex) -> i32;
    pub(super) fn mysql__DdIndex__is_hidden(index: *const DdIndex) -> bool;
    pub(super) fn mysql__DdIndex__element_count(index: *const DdIndex) -> usize;
    pub(super) fn mysql__DdIndex__element_at(
        index: *const DdIndex,
        i: usize,
    ) -> *const DdIndexElement;

    pub(super) fn mysql__DdIndexElement__column_ordinal(elt: *const DdIndexElement) -> u32;
    pub(super) fn mysql__DdIndexElement__length(elt: *const DdIndexElement) -> u32;
    pub(super) fn mysql__DdIndexElement__order(elt: *const DdIndexElement) -> i32;
    pub(super) fn mysql__DdIndexElement__is_hidden(elt: *const DdIndexElement) -> bool;
}

/// Read a NUL-terminated-or-bounded name from a `mysql__Dd*__name` accessor.
/// First call with `cap = 0` to learn the size, then allocate and re-call.
pub(super) fn read_name<F>(reader: F) -> String
where
    F: Fn(*mut u8, usize) -> usize,
{
    let needed = reader(core::ptr::null_mut(), 0);
    if needed == 0 {
        return String::new();
    }
    let mut buf = vec![0u8; needed];
    let written = reader(buf.as_mut_ptr(), needed);
    let n = core::cmp::min(written, needed);
    buf.truncate(n);
    String::from_utf8(buf).unwrap_or_default()
}
