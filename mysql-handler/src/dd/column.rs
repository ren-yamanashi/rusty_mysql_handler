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

//! [`DdColumn`] safe accessors and the [`ColumnType`] enum mirroring
//! `dd::enum_column_types` from `sql/dd/types/column.h`.

#![allow(unsafe_code)]

use crate::dd::ffi;
use crate::sys::DdColumn;

/// MySQL data-dictionary column type. Mirrors `dd::enum_column_types`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)] // Variants mirror upstream `dd::enum_column_types` 1:1.
pub enum ColumnType {
    Decimal,
    Tiny,
    Short,
    Long,
    Float,
    Double,
    Null,
    Timestamp,
    LongLong,
    Int24,
    Date,
    Time,
    DateTime,
    Year,
    NewDate,
    VarChar,
    Bit,
    Timestamp2,
    DateTime2,
    Time2,
    NewDecimal,
    Enum,
    Set,
    TinyBlob,
    MediumBlob,
    LongBlob,
    Blob,
    VarString,
    String,
    Geometry,
    Json,
    /// Unknown / out-of-range value. Future MySQL versions may add new types.
    Unknown,
}

impl ColumnType {
    /// Map the raw `dd::enum_column_types` integer to a [`ColumnType`].
    /// Returns [`ColumnType::Unknown`] for unrecognised values.
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Decimal,
            2 => Self::Tiny,
            3 => Self::Short,
            4 => Self::Long,
            5 => Self::Float,
            6 => Self::Double,
            7 => Self::Null,
            8 => Self::Timestamp,
            9 => Self::LongLong,
            10 => Self::Int24,
            11 => Self::Date,
            12 => Self::Time,
            13 => Self::DateTime,
            14 => Self::Year,
            15 => Self::NewDate,
            16 => Self::VarChar,
            17 => Self::Bit,
            18 => Self::Timestamp2,
            19 => Self::DateTime2,
            20 => Self::Time2,
            21 => Self::NewDecimal,
            22 => Self::Enum,
            23 => Self::Set,
            24 => Self::TinyBlob,
            25 => Self::MediumBlob,
            26 => Self::LongBlob,
            27 => Self::Blob,
            28 => Self::VarString,
            29 => Self::String,
            30 => Self::Geometry,
            31 => Self::Json,
            _ => Self::Unknown,
        }
    }
}

impl DdColumn {
    /// Column name as stored in the data dictionary.
    #[must_use]
    pub fn name(&self) -> String {
        let p: *const DdColumn = self;
        // SAFETY: `self` is a valid borrow; the FFI accessor only reads from `p`
        // and writes into the caller-owned buffer.
        ffi::read_name(|buf, cap| unsafe { ffi::mysql__DdColumn__name(p, buf, cap) })
    }

    /// Column type.
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        let p: *const DdColumn = self;
        // SAFETY: `self` is a valid borrow.
        let raw = unsafe { ffi::mysql__DdColumn__type(p) };
        ColumnType::from_raw(raw)
    }

    /// `true` when the column allows `NULL`.
    #[must_use]
    pub fn is_nullable(&self) -> bool {
        let p: *const DdColumn = self;
        // SAFETY: `self` is a valid borrow.
        unsafe { ffi::mysql__DdColumn__is_nullable(p) }
    }

    /// `true` for unsigned integer columns.
    #[must_use]
    pub fn is_unsigned(&self) -> bool {
        let p: *const DdColumn = self;
        // SAFETY: `self` is a valid borrow.
        unsafe { ffi::mysql__DdColumn__is_unsigned(p) }
    }

    /// Declared character length (`VARCHAR(N)` returns `N`).
    #[must_use]
    pub fn char_length(&self) -> u32 {
        let p: *const DdColumn = self;
        // SAFETY: `self` is a valid borrow.
        unsafe { ffi::mysql__DdColumn__char_length(p) }
    }

    /// `true` for any non-`HT_VISIBLE` column (SE-hidden, SQL-hidden,
    /// USER-hidden). Engines that build their own row layout should skip these.
    #[must_use]
    pub fn is_hidden(&self) -> bool {
        let p: *const DdColumn = self;
        // SAFETY: `self` is a valid borrow.
        unsafe { ffi::mysql__DdColumn__is_hidden(p) }
    }

    /// 1-based ordinal position within the table.
    #[must_use]
    pub fn ordinal_position(&self) -> u32 {
        let p: *const DdColumn = self;
        // SAFETY: `self` is a valid borrow.
        unsafe { ffi::mysql__DdColumn__ordinal_position(p) }
    }
}

#[cfg(test)]
mod tests {
    use super::ColumnType;

    #[test]
    fn from_raw_maps_known_variants() {
        assert_eq!(ColumnType::from_raw(2), ColumnType::Tiny);
        assert_eq!(ColumnType::from_raw(4), ColumnType::Long);
        assert_eq!(ColumnType::from_raw(9), ColumnType::LongLong);
        assert_eq!(ColumnType::from_raw(16), ColumnType::VarChar);
        assert_eq!(ColumnType::from_raw(31), ColumnType::Json);
    }

    #[test]
    fn from_raw_returns_unknown_for_out_of_range() {
        assert_eq!(ColumnType::from_raw(0), ColumnType::Unknown);
        assert_eq!(ColumnType::from_raw(-1), ColumnType::Unknown);
        assert_eq!(ColumnType::from_raw(99), ColumnType::Unknown);
    }
}
