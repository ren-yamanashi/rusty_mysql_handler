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
    /// whose `index_type` is [`IndexType::Primary`], or the first index
    /// when no primary key exists. `None` when the table has no indexes.
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
        let idx = self.primary_index()?;
        let first_part = idx.parts.first()?;
        let column_index = (first_part.column_ordinal as usize).checked_sub(1)?;
        let offset = self.data_offset(column_index)?;
        let column = self.columns.get(column_index)?;
        Some((offset, column))
    }

    /// The index treated as primary: the declared `PRIMARY` index, or
    /// the first index when no `PRIMARY` is declared. `None` only when
    /// the table has no indexes.
    #[must_use]
    pub fn primary_index(&self) -> Option<&IndexMeta> {
        let declared = self
            .indexes
            .iter()
            .find(|i| i.index_type == IndexType::Primary);
        match declared {
            Some(i) => Some(i),
            None => self.indexes.first(),
        }
    }

    /// Ordinal of [`Self::primary_index`] within [`Self::indexes`]; `None`
    /// when the table has no indexes.
    #[must_use]
    pub fn primary_index_ordinal(&self) -> Option<usize> {
        let primary = self.primary_index()?;
        self.indexes.iter().position(|i| std::ptr::eq(i, primary))
    }

    /// Resolve `(offset, &ColumnMeta)` for each column referenced by
    /// `index`'s key parts (in declared order). `None` when any
    /// referenced column is hidden or its offset cannot be computed.
    #[must_use]
    pub fn index_columns(&self, index: &IndexMeta) -> Option<Vec<(usize, &ColumnMeta)>> {
        let mut out = Vec::with_capacity(index.parts.len());
        for part in &index.parts {
            let col_idx = (part.column_ordinal as usize).checked_sub(1)?;
            let offset = self.data_offset(col_idx)?;
            let column = self.columns.get(col_idx)?;
            out.push((offset, column));
        }
        Some(out)
    }
}
