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

    /// The next-prefix sentinel: a key that compares strictly greater
    /// than every key starting with `self`'s parts. Used as the
    /// exclusive end bound for partial-prefix range scans
    /// (`WHERE a = 1` on `KEY (a, b)` becomes `[Key([1]), Key([2]))`).
    /// `None` when the last part has no representable successor
    /// (numeric overflow or a variant the engine does not know how to
    /// bump).
    #[must_use]
    pub fn next_prefix(&self) -> Option<Self> {
        let mut parts = self.parts.clone();
        let last = parts.last_mut()?;
        match last {
            KeyValue::Signed(n) => *n = n.checked_add(1)?,
            KeyValue::Unsigned(n) => *n = n.checked_add(1)?,
            _ => return None,
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

/// Decode a multi-column search-key buffer using `index`'s declared key
/// parts. The buffer concatenates each part's bytes in declaration order
/// (no null prefix; the engine only advertises indexes on `NOT NULL`
/// columns today). When the buffer is shorter than the full key, the
/// returned [`Key`] holds however many leading parts fit — MySQL passes
/// partial prefix buffers for queries that only constrain the leading
/// columns of a composite index (`WHERE a = ?` on `KEY (a, b)`).
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

#[cfg(test)]
mod tests {
    use super::*;

    fn signed(ty: ColumnType) -> ColumnMeta {
        ColumnMeta {
            column_type: ty,
            is_nullable: false,
            is_unsigned: false,
            char_length: 0,
            is_hidden: false,
        }
    }

    fn unsigned(ty: ColumnType) -> ColumnMeta {
        ColumnMeta {
            column_type: ty,
            is_nullable: false,
            is_unsigned: true,
            char_length: 0,
            is_hidden: false,
        }
    }

    #[test]
    fn signed_int_decodes_round_trip_for_each_width() {
        for (ty, bytes, want) in [
            (ColumnType::Tiny, vec![0x80u8], -128i64),
            (ColumnType::Short, vec![0xFF, 0xFF], -1),
            (ColumnType::Long, vec![0x01, 0x00, 0x00, 0x00], 1),
            (ColumnType::Long, vec![0xFF, 0xFF, 0xFF, 0xFF], -1),
        ] {
            let v = decode_int_key_buffer(&bytes, &signed(ty));
            assert_eq!(v, Some(KeyValue::Signed(want)), "{ty:?} {bytes:?}");
        }
    }

    #[test]
    fn unsigned_long_decodes_high_bit_as_large_positive() {
        // UNSIGNED 0xFFFFFFFF stays positive at u64 level.
        let v = decode_int_key_buffer(&[0xFF, 0xFF, 0xFF, 0xFF], &unsigned(ColumnType::Long));
        assert_eq!(v, Some(KeyValue::Unsigned(u64::from(u32::MAX))));
    }

    #[test]
    fn decode_int_returns_none_for_non_int_type() {
        let col = signed(ColumnType::VarChar);
        assert_eq!(decode_int_key_buffer(&[0; 8], &col), None);
    }

    #[test]
    fn ord_arranges_null_then_numeric_then_bytes() {
        assert!(KeyValue::Null < KeyValue::Signed(0));
        assert!(KeyValue::Signed(-1) < KeyValue::Signed(1));
        assert!(KeyValue::Null < KeyValue::Bytes(vec![]));
        assert!(KeyValue::Bytes(vec![1, 2]) < KeyValue::Bytes(vec![1, 3]));
    }

    #[test]
    fn key_compares_lex_over_parts() {
        let lo = Key::single(KeyValue::Signed(1));
        let hi = Key::single(KeyValue::Signed(2));
        assert!(lo < hi);
    }

    #[test]
    fn extract_from_record_honours_offset() {
        // record[0] layout: [null bits | pad | id]
        //                   [   0xFE  | 0..4 | 0xAA, 0x00, 0x00, 0x00]
        let row = [0xFE, 0, 0, 0, 0, 0xAA, 0x00, 0x00, 0x00];
        let v = extract_int_from_record(&row, 5, &signed(ColumnType::Long));
        assert_eq!(v, Some(KeyValue::Signed(0xAA)));
    }
}
