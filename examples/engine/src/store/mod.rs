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

//! Process-wide committed-row store for the reference engine.
//!
//! Rows are kept sorted by [`Key`] in a per-table [`TableStore`] so the
//! engine can answer ordered scans, `BETWEEN` ranges, and key lookups in
//! `O(log n)`. Unbounded by row count; a real engine would cap, evict,
//! or persist them.

mod btree_store;
mod column_meta;
mod index_meta;
mod key;
mod key_part_meta;
mod table_meta;

pub use btree_store::TableStore;
pub use column_meta::ColumnMeta;
pub use index_meta::IndexMeta;
pub use key::{
    Key, KeyValue, build_key_from_search_buffer, decode_int_key_buffer, extract_int_from_record,
    extract_key_from_row,
};
pub use key_part_meta::KeyPartMeta;
pub use table_meta::TableMeta;

use std::collections::HashMap;
use std::ops::Bound;
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};

static COMMITTED: LazyLock<Mutex<HashMap<String, TableStore>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static META_REGISTRY: LazyLock<Mutex<HashMap<String, TableMeta>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn committed() -> MutexGuard<'static, HashMap<String, TableStore>> {
    // A panic while holding the lock cannot corrupt an in-memory row map, so
    // recover the guard rather than propagate the poison.
    COMMITTED.lock().unwrap_or_else(PoisonError::into_inner)
}

fn meta_guard() -> MutexGuard<'static, HashMap<String, TableMeta>> {
    META_REGISTRY.lock().unwrap_or_else(PoisonError::into_inner)
}

fn with_store<F, R>(table: &str, default: R, f: F) -> R
where
    F: FnOnce(&TableStore) -> R,
{
    match committed().get(table) {
        Some(s) => f(s),
        None => default,
    }
}

fn with_store_mut<F, R>(table: &str, f: F) -> R
where
    F: FnOnce(&mut TableStore) -> R,
{
    f(committed().entry(table.to_owned()).or_default())
}

/// Snapshot of `(key, row)` pairs in key order.
#[must_use]
pub(crate) fn pairs_sorted(table: &str) -> Vec<(Key, Vec<u8>)> {
    with_store(table, Vec::new(), TableStore::pairs_sorted)
}

/// Snapshot of `(key, row)` pairs whose key falls in `start..end`.
#[must_use]
pub(crate) fn range_pairs(
    table: &str,
    start: &Bound<Key>,
    end: &Bound<Key>,
) -> Vec<(Key, Vec<u8>)> {
    with_store(table, Vec::new(), |s| s.range_pairs(start, end))
}

/// Count of rows whose key falls in `start..end`. Cheaper than
/// `range_pairs(table, start, end).len()` since no row bytes are cloned.
#[must_use]
pub(crate) fn range_len(table: &str, start: &Bound<Key>, end: &Bound<Key>) -> u64 {
    with_store(table, 0, |s| s.range_len(start, end))
}

/// Number of rows committed to `table`.
#[must_use]
pub(crate) fn row_count(table: &str) -> u64 {
    with_store(table, 0, TableStore::len)
}

/// Append a batch of `(key, row)` pairs to `table` (a transaction
/// committing rows that the engine could key from `record[0]`).
pub(crate) fn commit_keyed(table: &str, rows: Vec<(Key, Vec<u8>)>) {
    if rows.is_empty() {
        return;
    }
    with_store_mut(table, |s| {
        for (k, v) in rows {
            s.insert(k, v);
        }
    });
}

/// Append a batch of rows under synthetic sequence keys (used when the
/// table has no extractable primary key).
pub(crate) fn commit_seq(table: &str, rows: Vec<Vec<u8>>) {
    if rows.is_empty() {
        return;
    }
    with_store_mut(table, |s| {
        for v in rows {
            s.insert_seq(v);
        }
    });
}

/// Drop `table`'s committed rows (TRUNCATE / delete-all).
pub(crate) fn reset_table(table: &str) {
    committed().remove(table);
}

/// Forget `table`'s rows and schema (DROP TABLE / DELETE TABLE).
pub(crate) fn forget_table(table: &str) {
    committed().remove(table);
    meta_guard().remove(table);
}

/// Register the schema snapshot the engine extracted from `dd::Table`
/// in `open` / `create`. Later writes (which only see the table name)
/// can look it up to derive a [`Key`] for the row.
pub(crate) fn register_meta(table: &str, meta: TableMeta) {
    meta_guard().insert(table.to_owned(), meta);
}

/// Borrow a clone of `table`'s registered schema, if any.
#[must_use]
pub(crate) fn lookup_meta(table: &str) -> Option<TableMeta> {
    meta_guard().get(table).cloned()
}

/// Replace the row at `key` with `new` in `table`. `false` when the key
/// is absent or the table has never been touched.
#[must_use]
pub(crate) fn replace_by_key(table: &str, key: &Key, new: Vec<u8>) -> bool {
    with_store_mut(table, |s| s.replace_by_key(key, new))
}

/// Remove the row at `key` from `table`. `false` when the key is absent
/// or the table has never been touched.
#[must_use]
pub(crate) fn remove_by_key(table: &str, key: &Key) -> bool {
    with_store_mut(table, |s| s.remove_by_key(key))
}

/// Replace the first row whose bytes equal `old`. Fallback used when the
/// engine cannot extract a [`Key`] (no schema info, no indexed column).
#[must_use]
pub(crate) fn replace_by_bytes(table: &str, old: &[u8], new: Vec<u8>) -> bool {
    with_store_mut(table, |s| s.replace_by_bytes(old, new))
}

/// Remove the first row whose bytes equal `target`. Fallback as above.
#[must_use]
pub(crate) fn remove_by_bytes(table: &str, target: &[u8]) -> bool {
    with_store_mut(table, |s| s.remove_by_bytes(target))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(n: i64) -> Key {
        Key::single(KeyValue::Signed(n))
    }

    fn rows_only(table: &str) -> Vec<Vec<u8>> {
        pairs_sorted(table).into_iter().map(|(_, v)| v).collect()
    }

    #[test]
    fn commit_keyed_then_rows_sorted_returns_in_key_order() {
        let t = "_t_store_keyed";
        reset_table(t);
        commit_keyed(t, vec![(k(3), vec![3]), (k(1), vec![1]), (k(2), vec![2])]);
        assert_eq!(rows_only(t), vec![vec![1], vec![2], vec![3]]);
        reset_table(t);
    }

    #[test]
    fn commit_seq_preserves_insertion_order() {
        let t = "_t_store_seq";
        reset_table(t);
        commit_seq(t, vec![vec![b'a'], vec![b'b'], vec![b'c']]);
        assert_eq!(rows_only(t), vec![vec![b'a'], vec![b'b'], vec![b'c']]);
        reset_table(t);
    }

    #[test]
    fn reset_table_drops_all_rows_and_empty_commits_are_noops() {
        let t = "_t_store_reset";
        reset_table(t);
        commit_keyed(t, Vec::new());
        commit_seq(t, Vec::new());
        assert_eq!(row_count(t), 0);
        commit_keyed(t, vec![(k(1), vec![1])]);
        reset_table(t);
        assert_eq!(row_count(t), 0);
    }

    #[test]
    fn replace_and_remove_by_key_locate_existing_rows() {
        let t = "_t_store_replace_key";
        reset_table(t);
        commit_keyed(t, vec![(k(1), vec![1, 2]), (k(2), vec![3, 4])]);
        assert!(replace_by_key(t, &k(1), vec![9, 9]));
        assert!(!remove_by_key(t, &k(7)));
        assert!(remove_by_key(t, &k(2)));
        assert_eq!(rows_only(t), vec![vec![9, 9]]);
        reset_table(t);
    }

    #[test]
    fn range_pairs_yields_keys_in_window() {
        let t = "_t_store_range";
        reset_table(t);
        commit_keyed(t, (1..=5).map(|n| (k(n), vec![n as u8])).collect());
        let win = range_pairs(t, &Bound::Included(k(2)), &Bound::Included(k(3)));
        assert_eq!(
            win.into_iter().map(|(_, v)| v[0]).collect::<Vec<_>>(),
            vec![2, 3]
        );
        reset_table(t);
    }

    #[test]
    fn replace_by_bytes_fallback_finds_match() {
        let t = "_t_store_replace_bytes";
        reset_table(t);
        commit_keyed(t, vec![(k(1), vec![1, 1]), (k(2), vec![2, 2])]);
        assert!(replace_by_bytes(t, &[1, 1], vec![9, 9]));
        assert_eq!(rows_only(t), vec![vec![9, 9], vec![2, 2]]);
        reset_table(t);
    }
}
