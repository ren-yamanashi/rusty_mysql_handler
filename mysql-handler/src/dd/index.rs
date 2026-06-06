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

//! [`DdIndex`] / [`DdIndexElement`] safe accessors and the [`IndexType`] /
//! [`IndexElementOrder`] enums.

#![allow(unsafe_code)]

use crate::dd::ffi;
use crate::sys::{DdIndex, DdIndexElement};

/// MySQL index type. Mirrors `dd::Index::enum_index_type`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)] // Variants mirror upstream `dd::Index::enum_index_type` 1:1.
pub enum IndexType {
    Primary,
    Unique,
    Multiple,
    FullText,
    Spatial,
    /// Unknown / out-of-range value.
    Unknown,
}

impl IndexType {
    /// Map the raw `dd::Index::enum_index_type` integer to an [`IndexType`].
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Primary,
            2 => Self::Unique,
            3 => Self::Multiple,
            4 => Self::FullText,
            5 => Self::Spatial,
            _ => Self::Unknown,
        }
    }
}

/// Sort order of an index element. Mirrors
/// `dd::Index_element::enum_index_element_order`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)] // Variants mirror upstream `enum_index_element_order` 1:1.
pub enum IndexElementOrder {
    Undefined,
    Ascending,
    Descending,
    /// Unknown / out-of-range value.
    Unknown,
}

impl IndexElementOrder {
    /// Map the raw order integer to an [`IndexElementOrder`].
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Undefined,
            2 => Self::Ascending,
            3 => Self::Descending,
            _ => Self::Unknown,
        }
    }
}

impl DdIndex {
    /// Index name as stored in the data dictionary (`PRIMARY` for the
    /// primary key).
    #[must_use]
    pub fn name(&self) -> String {
        let p: *const DdIndex = self;
        // SAFETY: `self` is a valid borrow.
        ffi::read_name(|buf, cap| unsafe { ffi::mysql__DdIndex__name(p, buf, cap) })
    }

    /// Index kind (primary / unique / non-unique / fulltext / spatial).
    #[must_use]
    pub fn index_type(&self) -> IndexType {
        let p: *const DdIndex = self;
        // SAFETY: `self` is a valid borrow.
        let raw = unsafe { ffi::mysql__DdIndex__type(p) };
        IndexType::from_raw(raw)
    }

    /// `true` for indexes the SE itself added and is not exposed via SQL.
    #[must_use]
    pub fn is_hidden(&self) -> bool {
        let p: *const DdIndex = self;
        // SAFETY: `self` is a valid borrow.
        unsafe { ffi::mysql__DdIndex__is_hidden(p) }
    }

    /// Number of key parts in the index.
    #[must_use]
    pub fn element_count(&self) -> usize {
        let p: *const DdIndex = self;
        // SAFETY: `self` is a valid borrow.
        unsafe { ffi::mysql__DdIndex__element_count(p) }
    }

    /// Borrow the `i`th key part (0-based). Returns `None` past the end.
    #[must_use]
    pub fn element_at(&self, i: usize) -> Option<&DdIndexElement> {
        let p: *const DdIndex = self;
        // SAFETY: `self` is a valid borrow; the FFI returns null when `i`
        // is out of range or `p` is null.
        let raw = unsafe { ffi::mysql__DdIndex__element_at(p, i) };
        // SAFETY: a non-null result references a key part owned by the same
        // dd::Table tree as `self` and is valid for `self`'s lifetime.
        unsafe { raw.as_ref() }
    }
}

impl DdIndexElement {
    /// 1-based ordinal position of the underlying column in the table.
    #[must_use]
    pub fn column_ordinal(&self) -> u32 {
        let p: *const DdIndexElement = self;
        // SAFETY: `self` is a valid borrow.
        unsafe { ffi::mysql__DdIndexElement__column_ordinal(p) }
    }

    /// Prefix length in bytes (0 means the whole column).
    #[must_use]
    pub fn length(&self) -> u32 {
        let p: *const DdIndexElement = self;
        // SAFETY: `self` is a valid borrow.
        unsafe { ffi::mysql__DdIndexElement__length(p) }
    }

    /// Sort order declared on this key part.
    #[must_use]
    pub fn order(&self) -> IndexElementOrder {
        let p: *const DdIndexElement = self;
        // SAFETY: `self` is a valid borrow.
        let raw = unsafe { ffi::mysql__DdIndexElement__order(p) };
        IndexElementOrder::from_raw(raw)
    }

    /// `true` for key parts the SE added that are not exposed via SQL.
    #[must_use]
    pub fn is_hidden(&self) -> bool {
        let p: *const DdIndexElement = self;
        // SAFETY: `self` is a valid borrow.
        unsafe { ffi::mysql__DdIndexElement__is_hidden(p) }
    }
}

#[cfg(test)]
mod tests {
    use super::{IndexElementOrder, IndexType};

    #[test]
    fn index_type_from_raw_maps_known_variants() {
        assert_eq!(IndexType::from_raw(1), IndexType::Primary);
        assert_eq!(IndexType::from_raw(2), IndexType::Unique);
        assert_eq!(IndexType::from_raw(3), IndexType::Multiple);
        assert_eq!(IndexType::from_raw(4), IndexType::FullText);
        assert_eq!(IndexType::from_raw(5), IndexType::Spatial);
    }

    #[test]
    fn index_type_from_raw_returns_unknown_for_out_of_range() {
        assert_eq!(IndexType::from_raw(0), IndexType::Unknown);
        assert_eq!(IndexType::from_raw(99), IndexType::Unknown);
    }

    #[test]
    fn order_from_raw_maps_known_variants() {
        assert_eq!(IndexElementOrder::from_raw(1), IndexElementOrder::Undefined);
        assert_eq!(IndexElementOrder::from_raw(2), IndexElementOrder::Ascending);
        assert_eq!(
            IndexElementOrder::from_raw(3),
            IndexElementOrder::Descending
        );
        assert_eq!(IndexElementOrder::from_raw(4), IndexElementOrder::Unknown);
    }
}
