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

//! [`TableStore`]: per-table sorted row image keyed by [`Key`].
//!
//! Replaces the unsorted `Vec<Vec<u8>>` used in earlier stages, so the
//! reference engine can answer ordered scans, `BETWEEN` range queries,
//! and key lookups in `O(log n)` against the same data the demo
//! transactions commit.
//!
//! Unindexed tables fall back to a per-store sequence counter that gets
//! wrapped in a [`KeyValue::Unsigned`] — insertion order is preserved
//! and rows do not collide, which is enough for the demo's INSERT /
//! SELECT-only fixtures.

use std::collections::BTreeMap;
use std::ops::Bound;

use crate::store::{Key, KeyValue};

/// Sorted row image for one table.
#[derive(Debug, Default)]
pub struct TableStore {
    rows: BTreeMap<Key, Vec<u8>>,
    next_seq: u64,
}

impl TableStore {
    /// New empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert (or replace, last-write-wins) `row` under `key`.
    pub fn insert(&mut self, key: Key, row: Vec<u8>) {
        self.rows.insert(key, row);
    }

    /// Insert `row` under the next synthetic sequence key. Used by
    /// unindexed tables that have no natural map key.
    pub fn insert_seq(&mut self, row: Vec<u8>) {
        let key = Key::single(KeyValue::Unsigned(self.next_seq));
        self.next_seq = self.next_seq.saturating_add(1);
        self.rows.insert(key, row);
    }

    /// Replace the row at `key`. Returns `false` when the key is absent.
    #[must_use]
    pub fn replace_by_key(&mut self, key: &Key, new: Vec<u8>) -> bool {
        let slot = match self.rows.get_mut(key) {
            Some(s) => s,
            None => return false,
        };
        *slot = new;
        true
    }

    /// Remove the row at `key`. Returns `false` when the key is absent.
    #[must_use]
    pub fn remove_by_key(&mut self, key: &Key) -> bool {
        self.rows.remove(key).is_some()
    }

    /// Replace the first row whose bytes equal `old`. Fallback path for
    /// stores whose engine cannot extract a [`Key`] from `old` (no
    /// schema info, no indexed column).
    #[must_use]
    pub fn replace_by_bytes(&mut self, old: &[u8], new: Vec<u8>) -> bool {
        let key = match self.find_key_by_bytes(old) {
            Some(k) => k,
            None => return false,
        };
        self.rows.insert(key, new);
        true
    }

    /// Remove the first row whose bytes equal `target`.
    #[must_use]
    pub fn remove_by_bytes(&mut self, target: &[u8]) -> bool {
        let key = match self.find_key_by_bytes(target) {
            Some(k) => k,
            None => return false,
        };
        self.rows.remove(&key);
        true
    }

    fn find_key_by_bytes(&self, target: &[u8]) -> Option<Key> {
        self.rows
            .iter()
            .find(|(_, v)| v.as_slice() == target)
            .map(|(k, _)| k.clone())
    }

    /// Snapshot of `(key, row)` pairs in key order.
    #[must_use]
    pub fn pairs_sorted(&self) -> Vec<(Key, Vec<u8>)> {
        self.rows
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Snapshot of rows in key order (`pairs_sorted` without the keys).
    #[must_use]
    pub fn rows_sorted(&self) -> Vec<Vec<u8>> {
        self.rows.values().cloned().collect()
    }

    /// Snapshot of `(key, row)` pairs whose key falls in `start..end`.
    #[must_use]
    pub fn range_pairs(&self, start: &Bound<Key>, end: &Bound<Key>) -> Vec<(Key, Vec<u8>)> {
        self.rows
            .range((bound_ref(start), bound_ref(end)))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Count of rows whose key falls in `start..end`. Cheaper than
    /// `range_pairs(start, end).len()` because no row bytes are cloned.
    #[must_use]
    pub fn range_len(&self, start: &Bound<Key>, end: &Bound<Key>) -> u64 {
        self.rows.range((bound_ref(start), bound_ref(end))).count() as u64
    }

    /// Number of rows committed to this store.
    #[must_use]
    pub fn len(&self) -> u64 {
        self.rows.len() as u64
    }

    /// `true` when no rows have been committed (or all were removed).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

fn bound_ref(b: &Bound<Key>) -> Bound<&Key> {
    match b {
        Bound::Included(k) => Bound::Included(k),
        Bound::Excluded(k) => Bound::Excluded(k),
        Bound::Unbounded => Bound::Unbounded,
    }
}
