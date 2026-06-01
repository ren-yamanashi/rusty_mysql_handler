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

/// Per-column metadata snapshot taken from `dd::Column`. Only the fields
/// the reference engine consumes today are stored; other `dd::Column`
/// attributes (column name, ordinal position, ...) are intentionally
/// left off until a downstream consumer needs them.
#[derive(Debug, Clone)]
pub struct ColumnMeta {
    /// MySQL data-dictionary type.
    pub(crate) column_type: ColumnType,
    /// `NULL`-allowed?
    pub(crate) is_nullable: bool,
    /// `true` for unsigned integer columns. Drives the
    /// signed-vs-unsigned interpretation of [`crate::store::KeyValue`]
    /// when encoding `record[0]` bytes into a key.
    pub(crate) is_unsigned: bool,
    /// Declared character length for string types; `0` for non-string
    /// types where it is not meaningful. For multi-byte charsets the
    /// in-row storage is `prefix + char_length * mbmaxlen` — see
    /// [`Self::data_width`].
    pub(crate) char_length: u32,
    /// `true` for any non-`HT_VISIBLE` column. The snapshot does not
    /// distinguish `HT_HIDDEN_SE` (absent from `record[0]`),
    /// `HT_HIDDEN_SQL` (functional-index expression — present),
    /// and `HT_HIDDEN_USER` (`INVISIBLE` — present), so
    /// [`crate::store::TableMeta::data_offset`] bails when one of these
    /// sits in the prefix.
    pub(crate) is_hidden: bool,
}

impl ColumnMeta {
    /// Snapshot the fields of `column` into an owned [`ColumnMeta`].
    #[must_use]
    pub fn from_dd_column(column: &DdColumn) -> Self {
        Self {
            column_type: column.column_type(),
            is_nullable: column.is_nullable(),
            is_unsigned: column.is_unsigned(),
            char_length: column.char_length(),
            is_hidden: column.is_hidden(),
        }
    }

    /// Width in bytes the column occupies inside `record[0]`. `None` for
    /// types the reference engine does not have to address by offset.
    ///
    /// VARCHAR / CHAR widths assume a single-byte character set (e.g.
    /// `ascii`, `latin1`). For multi-byte charsets the actual storage is
    /// `prefix + char_length * mbmaxlen`, and the length prefix itself may
    /// flip from 1 to 2 bytes once the byte length exceeds 255 even when
    /// `char_length <= 255`. The reference demo only uses ASCII, but
    /// downstream callers building on this snapshot must account for
    /// `mbmaxlen` themselves.
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
