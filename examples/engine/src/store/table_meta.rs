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

//! Schema snapshot for an open table.
//!
//! Populated from `dd::Table` during `open` / `create`; outlives the
//! dd::Table borrow. Byte offsets mirror MySQL's `record[0]` packing:
//!
//! ```text
//!   [ null bits | col_0 | col_1 | ... ]
//! ```
//!
//! `null bits` is `ceil(num_nullable_columns / 8)` bytes (`0` when none).

use mysql_handler::dd::IndexType;
use mysql_handler::sys::DdTable;

use crate::store::{ColumnMeta, IndexMeta};

/// Snapshot of column and index metadata for one table.
#[derive(Debug, Clone)]
pub struct TableMeta {
    columns: Vec<ColumnMeta>,
    indexes: Vec<IndexMeta>,
}

impl TableMeta {
    /// Build a [`TableMeta`] directly from pre-populated column and
    /// index lists (used by sibling tests that need a deterministic
    /// snapshot without going through `dd::Table`).
    #[cfg(test)]
    #[must_use]
    pub(crate) fn from_parts(columns: Vec<ColumnMeta>, indexes: Vec<IndexMeta>) -> Self {
        Self { columns, indexes }
    }

    /// Build a [`TableMeta`] by walking `table_def`'s column and index
    /// collections.
    #[must_use]
    pub fn from_dd_table(table_def: &DdTable) -> Self {
        let mut columns = Vec::with_capacity(table_def.column_count());
        for i in 0..table_def.column_count() {
            if let Some(c) = table_def.column_at(i) {
                columns.push(ColumnMeta::from_dd_column(c));
            }
        }
        let mut indexes = Vec::with_capacity(table_def.index_count());
        for i in 0..table_def.index_count() {
            if let Some(x) = table_def.index_at(i) {
                indexes.push(IndexMeta::from_dd_index(x));
            }
        }
        Self { columns, indexes }
    }

    /// Borrow the column snapshot.
    #[must_use]
    pub fn columns(&self) -> &[ColumnMeta] {
        &self.columns
    }

    /// Borrow the index snapshot.
    #[must_use]
    pub fn indexes(&self) -> &[IndexMeta] {
        &self.indexes
    }

    /// Number of bytes of NULL bits at the start of `record[0]`.
    ///
    /// MySQL reserves one leading bit for the row's delete-mark / presence
    /// flag (`null_bit_pos = 1` in `sql/table.cc`) unless the table is
    /// created with `HA_OPTION_PACK_RECORD`. The reference engine does
    /// not opt into packed records, so the leading bit is always present
    /// and the count becomes `ceil((nullable_columns + 1) / 8)` — `1`
    /// byte even when no column is nullable.
    #[must_use]
    pub fn null_bits_bytes(&self) -> usize {
        let n = self.columns.iter().filter(|c| c.is_nullable).count();
        (n + 1).div_ceil(8)
    }

    /// Byte offset of the column at `column_index` inside `record[0]`.
    /// Returns `None` past the end, when a preceding column has unknown
    /// width, or when a hidden column sits in the prefix. The hidden-column
    /// case is conservative: `HT_HIDDEN_SE` columns are absent from
    /// `record[0]` but `HT_HIDDEN_SQL` (functional-index expression) and
    /// `HT_HIDDEN_USER` (INVISIBLE) columns are not, and the snapshot does
    /// not distinguish the three, so refuse rather than mis-compute.
    #[must_use]
    pub fn data_offset(&self, column_index: usize) -> Option<usize> {
        if column_index >= self.columns.len() {
            return None;
        }
        let mut offset = self.null_bits_bytes();
        for col in &self.columns[..column_index] {
            if col.is_hidden {
                return None;
            }
            offset = offset.checked_add(col.data_width()?)?;
        }
        Some(offset)
    }

    /// Byte offset of the first visible column referenced by the index
    /// whose [`IndexMeta::index_type`] is [`IndexType::Primary`], or the
    /// first index when no primary key exists. `None` when the table has
    /// no indexes.
    #[must_use]
    pub fn primary_key_offset(&self) -> Option<usize> {
        let (offset, _) = self.primary_key_column()?;
        Some(offset)
    }

    /// `(offset, column)` for the first column of the primary (or, when
    /// no primary key exists, the first declared) index. `None` when the
    /// table has no indexes, when the underlying column is hidden, or
    /// when the byte offset cannot be computed.
    #[must_use]
    pub fn primary_key_column(&self) -> Option<(usize, &ColumnMeta)> {
        let idx = self
            .indexes
            .iter()
            .find(|i| i.index_type == IndexType::Primary)
            .or_else(|| self.indexes.first())?;
        let first_part = idx.parts.first()?;
        let column_index = (first_part.column_ordinal as usize).checked_sub(1)?;
        let offset = self.data_offset(column_index)?;
        let column = self.columns.get(column_index)?;
        Some((offset, column))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::KeyPartMeta;
    use mysql_handler::dd::ColumnType;

    fn col(_name: &str, ty: ColumnType, nullable: bool, char_len: u32) -> ColumnMeta {
        ColumnMeta {
            column_type: ty,
            is_nullable: nullable,
            is_unsigned: false,
            char_length: char_len,
            is_hidden: false,
        }
    }

    fn primary_on(column_ordinal: u32) -> IndexMeta {
        IndexMeta {
            index_type: IndexType::Primary,
            parts: vec![KeyPartMeta {
                column_ordinal,
                order: mysql_handler::dd::IndexElementOrder::Ascending,
            }],
        }
    }

    #[test]
    fn null_bits_bytes_accounts_for_the_delete_mark_bit() {
        // Always at least 1 byte (1 delete-mark bit + nullables). Rolls
        // to 2 bytes once total bits exceed 8.
        let no_null = TableMeta {
            columns: vec![col("id", ColumnType::Long, false, 0)],
            indexes: vec![],
        };
        assert_eq!(no_null.null_bits_bytes(), 1);
        let eight_null: Vec<ColumnMeta> = (0..8)
            .map(|i| col(&format!("c{i}"), ColumnType::Long, true, 0))
            .collect();
        let m = TableMeta {
            columns: eight_null,
            indexes: vec![],
        };
        assert_eq!(m.null_bits_bytes(), 2);
    }

    #[test]
    fn data_offset_accumulates_column_widths_after_null_prefix() {
        // id INT (4 bytes) at offset 1, then name VARCHAR(50) at offset 5.
        let m = TableMeta {
            columns: vec![
                col("id", ColumnType::Long, true, 0),
                col("name", ColumnType::VarChar, true, 50),
            ],
            indexes: vec![],
        };
        assert_eq!(m.data_offset(0), Some(1));
        assert_eq!(m.data_offset(1), Some(1 + 4));
    }

    #[test]
    fn data_offset_none_when_hidden_column_sits_in_prefix() {
        // Hidden column packing depends on hidden-kind (SE / SQL / USER),
        // which the snapshot does not retain. Bail rather than mis-count.
        let hidden = ColumnMeta {
            column_type: ColumnType::Long,
            is_nullable: false,
            is_unsigned: false,
            char_length: 0,
            is_hidden: true,
        };
        let m = TableMeta {
            columns: vec![hidden, col("id", ColumnType::Long, false, 0)],
            indexes: vec![],
        };
        // First column starts after the 1-byte null/delete-mark prefix.
        assert_eq!(m.data_offset(0), Some(1));
        assert_eq!(m.data_offset(1), None);
    }

    #[test]
    fn primary_key_offset_locates_first_key_column() {
        // CREATE TABLE t (id INT NOT NULL, name VARCHAR(50), PRIMARY KEY (id))
        let m = TableMeta {
            columns: vec![
                col("id", ColumnType::Long, false, 0),
                col("name", ColumnType::VarChar, true, 50),
            ],
            indexes: vec![primary_on(1)],
        };
        // 1 nullable col → 1 null bits byte, id at offset 1.
        assert_eq!(m.primary_key_offset(), Some(1));
    }

    #[test]
    fn primary_key_offset_falls_back_to_first_index() {
        // No PRIMARY, only a secondary KEY on id.
        let m = TableMeta {
            columns: vec![
                col("id", ColumnType::Long, false, 0),
                col("name", ColumnType::VarChar, true, 50),
            ],
            indexes: vec![IndexMeta {
                index_type: IndexType::Multiple,
                parts: vec![KeyPartMeta {
                    column_ordinal: 1,
                    order: mysql_handler::dd::IndexElementOrder::Ascending,
                }],
            }],
        };
        assert_eq!(m.primary_key_offset(), Some(1));
    }

    #[test]
    fn primary_key_offset_none_when_no_indexes() {
        let m = TableMeta {
            columns: vec![col("id", ColumnType::Long, false, 0)],
            indexes: vec![],
        };
        assert_eq!(m.primary_key_offset(), None);
    }
}
