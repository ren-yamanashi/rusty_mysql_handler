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

//! Sortable key for the BTreeMap-backed row store.
//!
//! [`Key`] is the BTreeMap map key. It carries a single [`KeyValue`] for
//! the reference engine's single-column primary / secondary indexes. The
//! [`Ord`] impl is numeric for the INT family so a row inserted with
//! `id = -1` sorts before `id = 1` (raw LE byte comparison would invert
//! the sign).
//!
//! `extract_int_from_record` decodes the bytes MySQL passes in `record[0]`
//! at a `TableMeta`-resolved offset, using `column.is_unsigned` to pick
//! signed or unsigned interpretation. The same encoding is also reachable
//! from `decode_int_key_buffer`, which reads the bytes MySQL hands to
//! `index_read_map` as the search key.

use core::cmp::Ordering;

use mysql_handler::dd::ColumnType;

use crate::store::{ColumnMeta, IndexMeta, TableMeta};

/// One indexed value. The variants intentionally cover only the cases the
/// reference engine exercises today (INT family, sequence counter, opaque
/// bytes); VARCHAR / CHAR with collation-aware comparison is left for a
/// downstream consumer that needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyValue {
    /// SQL `NULL`.
    Null,
    /// Signed integer (TINY / SHORT / INT24 / LONG / LONGLONG).
    Signed(i64),
    /// Unsigned integer of the same family.
    Unsigned(u64),
    /// Opaque bytes used for unindexed tables (per-table sequence
    /// counter) and as a fallback when no [`ColumnType`] yields a
    /// numeric decode.
    Bytes(Vec<u8>),
}

impl Ord for KeyValue {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Null, Self::Null) => Ordering::Equal,
            (Self::Signed(a), Self::Signed(b)) => a.cmp(b),
            (Self::Unsigned(a), Self::Unsigned(b)) => a.cmp(b),
            (Self::Bytes(a), Self::Bytes(b)) => a.cmp(b),
            // Mixed signed / unsigned: compare numerically by widening to
            // i128 so the demo can survive a stray mismatch without panic.
            (Self::Signed(a), Self::Unsigned(b)) => i128::from(*a).cmp(&i128::from(*b)),
            (Self::Unsigned(a), Self::Signed(b)) => i128::from(*a).cmp(&i128::from(*b)),
            // Sort discriminant: `Null` < numeric < bytes. The engine
            // never intentionally produces these arms but must not panic.
            (Self::Null, Self::Signed(_) | Self::Unsigned(_) | Self::Bytes(_))
            | (Self::Signed(_) | Self::Unsigned(_), Self::Bytes(_)) => Ordering::Less,
            (Self::Signed(_) | Self::Unsigned(_) | Self::Bytes(_), Self::Null)
            | (Self::Bytes(_), Self::Signed(_) | Self::Unsigned(_)) => Ordering::Greater,
        }
    }
}

impl PartialOrd for KeyValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// BTreeMap key. Single-element for the current single-column indexes;
/// multi-element support is left to a downstream consumer.
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct Key {
    parts: Vec<KeyValue>,
}

impl Key {
    /// Build a single-element key.
    #[must_use]
    pub fn single(v: KeyValue) -> Self {
        Self { parts: vec![v] }
    }

    /// Build a key from an explicit list of parts. Composite primary
    /// keys and multi-column secondary indexes share this constructor.
    #[must_use]
    pub fn from_parts(parts: Vec<KeyValue>) -> Self {
        Self { parts }
    }

    /// Borrow the underlying parts.
    #[must_use]
    pub fn parts(&self) -> &[KeyValue] {
        &self.parts
    }

    /// Smallest key strictly greater than every key with `self` as a
    /// prefix — the exclusive end bound for partial-prefix scans
    /// (`WHERE a = 1` on `KEY (a, b)` becomes `[Key([1]), Key([2]))`).
    /// `None` on overflow or a non-incrementable tail variant.
    #[must_use]
    pub fn next_prefix(&self) -> Option<Self> {
        let mut parts = self.parts.clone();
        let last = parts.last_mut()?;
        match last {
            KeyValue::Signed(n) => *n = n.checked_add(1)?,
            KeyValue::Unsigned(n) => *n = n.checked_add(1)?,
            KeyValue::Null | KeyValue::Bytes(_) => return None,
        }
        Some(Self { parts })
    }
}

/// Decode `record[offset..]` as `column`'s integer value. `None` when the
/// type is outside the INT family.
#[must_use]
pub fn extract_int_from_record(
    record: &[u8],
    offset: usize,
    column: &ColumnMeta,
) -> Option<KeyValue> {
    decode_int(record, offset, column.column_type, column.is_unsigned)
}

/// Decode the search-key bytes MySQL hands to `index_read_map` for the
/// given column. The buffer starts at the column's first byte (no null
/// prefix). `None` when the type is outside the INT family.
#[must_use]
pub fn decode_int_key_buffer(buf: &[u8], column: &ColumnMeta) -> Option<KeyValue> {
    decode_int(buf, 0, column.column_type, column.is_unsigned)
}

/// Build a [`Key`] for `row` using `meta`'s primary index columns.
/// Composite primary keys yield a multi-part `Key`.
#[must_use]
pub fn extract_key_from_row(row: &[u8], meta: &TableMeta) -> Option<Key> {
    let primary = meta.primary_index()?;
    extract_index_key_from_row(row, meta, primary)
}

/// Build a [`Key`] for `row` using `index`'s declared key parts.
#[must_use]
pub fn extract_index_key_from_row(row: &[u8], meta: &TableMeta, index: &IndexMeta) -> Option<Key> {
    let cols = meta.index_columns(index)?;
    let mut parts = Vec::with_capacity(cols.len());
    for (offset, column) in cols {
        parts.push(extract_int_from_record(row, offset, column)?);
    }
    Some(Key::from_parts(parts))
}

/// Decode `index`'s search-key buffer into a [`Key`]. Each part is read
/// in declared order from the buffer's leading bytes. A buffer shorter
/// than the full key yields a partial-prefix [`Key`] — used by MySQL
/// when only the leading columns are constrained (`WHERE a = ?` on
/// `KEY (a, b)`).
#[must_use]
pub fn decode_index_search_buffer(buf: &[u8], meta: &TableMeta, index: &IndexMeta) -> Option<Key> {
    let cols = meta.index_columns(index)?;
    let mut parts = Vec::with_capacity(cols.len());
    let mut cursor: usize = 0;
    for (_, column) in cols {
        let width = int_width(column.column_type)?;
        let end = match cursor.checked_add(width) {
            Some(e) if e <= buf.len() => e,
            _ => break,
        };
        let value = decode_int(buf, cursor, column.column_type, column.is_unsigned)?;
        parts.push(value);
        cursor = end;
    }
    if parts.is_empty() {
        return None;
    }
    Some(Key::from_parts(parts))
}

fn decode_int(
    bytes: &[u8],
    offset: usize,
    column_type: ColumnType,
    unsigned: bool,
) -> Option<KeyValue> {
    let width = int_width(column_type)?;
    let end = offset.checked_add(width)?;
    let slice = bytes.get(offset..end)?;
    Some(decode_int_slice(slice, unsigned))
}

const fn int_width(column_type: ColumnType) -> Option<usize> {
    match column_type {
        ColumnType::Tiny => Some(1),
        ColumnType::Short => Some(2),
        ColumnType::Int24 => Some(3),
        ColumnType::Long => Some(4),
        ColumnType::LongLong => Some(8),
        _ => None,
    }
}

fn decode_int_slice(slice: &[u8], unsigned: bool) -> KeyValue {
    let mut buf = [0u8; 8];
    buf[..slice.len()].copy_from_slice(slice);
    if unsigned {
        return KeyValue::Unsigned(u64::from_le_bytes(buf));
    }
    let raw = i64::from_le_bytes(buf);
    // `slice.len()` is bounded by `int_width(column_type)` which never
    // exceeds 8, so the conversion fits in `u32` without truncation.
    let bits = u32::try_from(slice.len()).unwrap_or(0) * 8;
    KeyValue::Signed(sign_extend(raw, bits))
}

/// Sign-extend the lowest `bits` of `raw` to a full i64.
const fn sign_extend(raw: i64, bits: u32) -> i64 {
    if bits == 0 || bits >= 64 {
        return raw;
    }
    let shift = 64 - bits;
    (raw << shift) >> shift
}
