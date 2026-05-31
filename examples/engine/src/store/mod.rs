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
//! Rows are the raw `record[0]` byte images, keyed by bare table name so
//! the next statement's fresh handler reads them back. Unbounded; a real
//! engine would cap, evict, or persist them.

mod column_meta;
mod index_meta;
mod key_part_meta;
mod table_meta;

pub use column_meta::ColumnMeta;
pub use index_meta::IndexMeta;
pub use key_part_meta::KeyPartMeta;
pub use table_meta::TableMeta;

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};

static COMMITTED: LazyLock<Mutex<HashMap<String, Vec<Vec<u8>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn committed() -> MutexGuard<'static, HashMap<String, Vec<Vec<u8>>>> {
    // A panic while holding the lock cannot corrupt an in-memory row map, so
    // recover the guard rather than propagate the poison.
    COMMITTED.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Snapshot of the committed rows for `table` (empty if none). Returned by
/// value so the caller can iterate without holding the global mutex.
#[must_use]
pub fn committed_rows(table: &str) -> Vec<Vec<u8>> {
    match committed().get(table) {
        Some(rows) => rows.clone(),
        None => Vec::new(),
    }
}

/// Committed row count for `table` (0 if it has none). Cheaper than
/// `committed_rows(...).len()` when only the count is needed.
#[must_use]
pub fn committed_row_count(table: &str) -> u64 {
    match committed().get(table) {
        Some(rows) => rows.len() as u64,
        None => 0,
    }
}

/// Append `rows` to `table`'s committed image (a transaction committing)
pub fn commit_rows(table: &str, rows: Vec<Vec<u8>>) {
    if rows.is_empty() {
        return;
    }
    committed()
        .entry(table.to_owned())
        .or_default()
        .extend(rows);
}

/// Drop `table`'s committed rows (TRUNCATE / delete-all)
pub fn reset_table(table: &str) {
    committed().remove(table);
}

/// Mutable row vector for `table`, or `None` if the table is empty.
fn rows_mut<'a>(
    guard: &'a mut MutexGuard<'static, HashMap<String, Vec<Vec<u8>>>>,
    table: &str,
) -> Option<&'a mut Vec<Vec<u8>>> {
    guard.get_mut(table)
}

/// Replace the first row in `table` whose bytes equal `old` with `new`.
/// Returns `false` if no match was found. First-match-wins on byte
/// equality; the demo never produces colliding rows.
#[must_use]
pub fn replace_row(table: &str, old: &[u8], new: &[u8]) -> bool {
    let mut guard = committed();
    let rows = match rows_mut(&mut guard, table) {
        Some(r) => r,
        None => return false,
    };
    match rows.iter().position(|row| row.as_slice() == old) {
        Some(i) => {
            rows[i] = new.to_vec();
            true
        }
        None => false,
    }
}

/// Remove the first row in `table` whose bytes equal `target`. Returns
/// `false` if no match. Same first-match-wins caveat as [`replace_row`].
#[must_use]
pub fn remove_row(table: &str, target: &[u8]) -> bool {
    let mut guard = committed();
    let rows = match rows_mut(&mut guard, table) {
        Some(r) => r,
        None => return false,
    };
    match rows.iter().position(|row| row.as_slice() == target) {
        Some(i) => {
            rows.remove(i);
            true
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Each test scopes itself to a unique table name and resets at start so the
    // tests can coexist with other tests on the same process-wide store.

    #[test]
    fn commit_rows_then_committed_rows_round_trips_bytes() {
        let t = "_t_store_round_trip";
        reset_table(t);
        commit_rows(t, vec![vec![1, 2, 3], vec![4, 5]]);
        assert_eq!(committed_rows(t), vec![vec![1, 2, 3], vec![4, 5]]);
        reset_table(t);
    }

    #[test]
    fn commit_rows_extends_existing_table() {
        let t = "_t_store_extend";
        reset_table(t);
        commit_rows(t, vec![vec![1]]);
        commit_rows(t, vec![vec![2], vec![3]]);
        assert_eq!(committed_row_count(t), 3);
        reset_table(t);
    }

    #[test]
    fn commit_rows_with_empty_vec_is_a_noop() {
        let t = "_t_store_noop";
        reset_table(t);
        commit_rows(t, Vec::new());
        assert!(committed_rows(t).is_empty());
        assert_eq!(committed_row_count(t), 0);
    }

    #[test]
    fn reset_table_drops_all_rows() {
        let t = "_t_store_reset";
        commit_rows(t, vec![vec![9]]);
        reset_table(t);
        assert!(committed_rows(t).is_empty());
        assert_eq!(committed_row_count(t), 0);
    }

    #[test]
    fn replace_row_swaps_a_matching_row() {
        let t = "_t_store_replace";
        reset_table(t);
        commit_rows(t, vec![vec![1, 2], vec![3, 4]]);
        assert!(replace_row(t, &[1, 2], &[9, 9]));
        assert_eq!(committed_rows(t), vec![vec![9, 9], vec![3, 4]]);
        reset_table(t);
    }

    #[test]
    fn replace_row_returns_false_when_no_match() {
        let t = "_t_store_replace_miss";
        reset_table(t);
        commit_rows(t, vec![vec![1, 2]]);
        assert!(!replace_row(t, &[7, 7], &[8, 8]));
        assert_eq!(committed_rows(t), vec![vec![1, 2]]);
        reset_table(t);
    }

    #[test]
    fn remove_row_drops_a_matching_row() {
        let t = "_t_store_remove";
        reset_table(t);
        commit_rows(t, vec![vec![1], vec![2], vec![3]]);
        assert!(remove_row(t, &[2]));
        assert_eq!(committed_rows(t), vec![vec![1], vec![3]]);
        reset_table(t);
    }

    #[test]
    fn remove_row_returns_false_when_no_match() {
        let t = "_t_store_remove_miss";
        reset_table(t);
        commit_rows(t, vec![vec![1]]);
        assert!(!remove_row(t, &[7]));
        assert_eq!(committed_rows(t), vec![vec![1]]);
        reset_table(t);
    }
}
