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
//! A handler instance lives only for one statement, so committed rows cannot
//! live in the engine: they go here, keyed by table name, where the next
//! statement's fresh handler (and the per-connection transaction's commit) can
//! reach them. This demonstrates that a committed transaction is durable across
//! statements while a rolled-back one leaves no trace. It is a counting store
//! (rows are counted, not stored) — enough to make COMMIT vs ROLLBACK
//! observable via `SELECT COUNT(*)` — and keys by bare table name, so it does
//! not distinguish same-named tables in different databases.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};

static COMMITTED: LazyLock<Mutex<HashMap<String, u64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn committed() -> MutexGuard<'static, HashMap<String, u64>> {
    // A panic while holding the lock cannot corrupt a row-count map, so recover
    // the guard rather than propagate the poison.
    COMMITTED.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Committed row count for `table` (0 if it has none)
#[must_use]
pub fn committed_rows(table: &str) -> u64 {
    match committed().get(table) {
        Some(n) => *n,
        None => 0,
    }
}

/// Add `n` rows to `table`'s committed count (a transaction committing)
pub fn commit_rows(table: &str, n: u64) {
    if n == 0 {
        return;
    }
    *committed().entry(table.to_owned()).or_insert(0) += n;
}

/// Drop `table`'s committed rows (TRUNCATE / delete-all)
pub fn reset_table(table: &str) {
    committed().remove(table);
}
