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

//! Per-column snapshot taken from `dd::Column`.

use mysql_handler::dd::ColumnType;
use mysql_handler::sys::DdColumn;

/// Per-column metadata snapshot taken from `dd::Column`.
#[derive(Debug, Clone)]
pub struct ColumnMeta {
    /// Column name (UTF-8).
    pub name: String,
    /// MySQL data-dictionary type.
    pub column_type: ColumnType,
    /// `NULL`-allowed?
    pub is_nullable: bool,
    /// Unsigned integer?
    pub is_unsigned: bool,
    /// Declared character length for string types; `0` for non-string types
    /// where it is not meaningful.
    pub char_length: u32,
    /// `true` for non-`HT_VISIBLE` columns (SE-hidden, SQL-hidden,
    /// user-hidden). Excluded from the row image and from key offsets.
    pub is_hidden: bool,
}

impl ColumnMeta {
    /// Snapshot the fields of `column` into an owned [`ColumnMeta`].
    #[must_use]
    pub fn from_dd_column(column: &DdColumn) -> Self {
        Self {
            name: column.name(),
            column_type: column.column_type(),
            is_nullable: column.is_nullable(),
            is_unsigned: column.is_unsigned(),
            char_length: column.char_length(),
            is_hidden: column.is_hidden(),
        }
    }

    /// Width in bytes the column occupies inside `record[0]`. `None` for
    /// types the reference engine does not have to address by offset.
    #[must_use]
    pub const fn data_width(&self) -> Option<usize> {
        let len = self.char_length as usize;
        match self.column_type {
            ColumnType::Tiny => Some(1),
            ColumnType::Short | ColumnType::Year => Some(2),
            ColumnType::Int24 | ColumnType::NewDate | ColumnType::Date => Some(3),
            ColumnType::Long | ColumnType::Float => Some(4),
            ColumnType::LongLong | ColumnType::Double => Some(8),
            // VARCHAR(N) in record[0]: 1 length byte if N <= 255 else 2.
            ColumnType::VarChar | ColumnType::VarString => {
                let prefix = if len <= 255 { 1 } else { 2 };
                Some(prefix + len)
            }
            // CHAR(N): fixed N bytes (ignoring multi-byte charset packing).
            ColumnType::String => Some(len),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn col(ty: ColumnType, char_len: u32) -> ColumnMeta {
        ColumnMeta {
            name: String::new(),
            column_type: ty,
            is_nullable: false,
            is_unsigned: false,
            char_length: char_len,
            is_hidden: false,
        }
    }

    #[test]
    fn data_width_int_family() {
        assert_eq!(col(ColumnType::Tiny, 0).data_width(), Some(1));
        assert_eq!(col(ColumnType::Short, 0).data_width(), Some(2));
        assert_eq!(col(ColumnType::Int24, 0).data_width(), Some(3));
        assert_eq!(col(ColumnType::Long, 0).data_width(), Some(4));
        assert_eq!(col(ColumnType::LongLong, 0).data_width(), Some(8));
    }

    #[test]
    fn data_width_varchar_uses_one_byte_prefix_when_short() {
        // VARCHAR(50): 1 length byte + 50 data bytes.
        assert_eq!(col(ColumnType::VarChar, 50).data_width(), Some(51));
    }

    #[test]
    fn data_width_varchar_uses_two_byte_prefix_when_over_255() {
        assert_eq!(col(ColumnType::VarChar, 1000).data_width(), Some(1002));
    }

    #[test]
    fn data_width_unknown_for_unsupported_types() {
        assert_eq!(col(ColumnType::Json, 0).data_width(), None);
        assert_eq!(col(ColumnType::Blob, 0).data_width(), None);
    }
}
