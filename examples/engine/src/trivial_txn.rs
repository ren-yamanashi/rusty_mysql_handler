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

//! Per-connection transaction state for the reference engine.
//!
//! Writes, updates, and deletes accumulate as an `Op` log under their
//! target table. `commit(all=true)` replays the log against the committed
//! [`crate::store::TableStore`]; `rollback(all=true)` discards it.
//! Savepoints snapshot the whole op log so `ROLLBACK TO SAVEPOINT` can
//! restore the staging state.
//!
//! Known limitations (acceptable for the reference demo; downstream
//! engines that need richer semantics should diverge here):
//!
//! - **No read-your-own-writes.** Scans always go through
//!   `crate::store::pairs_sorted`, which only sees committed rows.
//!   A second statement within the same transaction observes the
//!   pre-transaction state, so a sequence like
//!   `BEGIN; UPDATE t SET x=2 WHERE id=1; UPDATE t SET x=3 WHERE x=2;`
//!   silently loses the second update.
//! - **`rollback(all=false)` is a no-op.** A per-statement rollback
//!   (e.g. on a constraint violation mid-transaction) leaves the
//!   failed statement's staged ops in place, so they replay at
//!   `COMMIT`. Statement-boundary tracking would split the log into
//!   per-statement segments; the demo skips it.

use std::collections::HashMap;

use mysql_handler::engine::EngineResult;
use mysql_handler::hton::TxnSession;

use crate::store;

/// Read the snapshot-stack index `TrivialTxn` wrote into a savepoint's `sv`.
fn sv_index(sv: &[u8]) -> Option<usize> {
    let bytes: [u8; 8] = sv.get(..8)?.try_into().ok()?;
    usize::try_from(u64::from_le_bytes(bytes)).ok()
}

/// One staged change against a table's committed store.
#[derive(Debug, Clone)]
pub(crate) enum Op {
    Insert(Vec<u8>),
    Update { old: Vec<u8>, new: Vec<u8> },
    Delete(Vec<u8>),
}

/// Per-connection transaction. Each statement's writes / updates /
/// deletes append to the op log keyed by table; commit (`all=true`)
/// replays them in order, rollback discards.
#[derive(Debug, Default)]
pub struct TrivialTxn {
    staged: HashMap<String, Vec<Op>>,
    savepoints: Vec<HashMap<String, Vec<Op>>>,
}

impl TxnSession for TrivialTxn {
    fn write_row(&mut self, table: &str, row: &[u8]) -> EngineResult {
        self.append(table, Op::Insert(row.to_vec()));
        Ok(())
    }

    fn update_row(&mut self, table: &str, old: &[u8], new: &[u8]) -> EngineResult {
        self.append(
            table,
            Op::Update {
                old: old.to_vec(),
                new: new.to_vec(),
            },
        );
        Ok(())
    }

    fn delete_row(&mut self, table: &str, row: &[u8]) -> EngineResult {
        self.append(table, Op::Delete(row.to_vec()));
        Ok(())
    }

    fn commit(&mut self, all: bool) -> EngineResult {
        // Flush only on the whole-transaction boundary. The shim upgrades
        // autocommit statement commits to all=true so they flush too.
        if all {
            for (table, ops) in self.staged.drain() {
                replay(&table, ops);
            }
        }
        Ok(())
    }

    fn rollback(&mut self, all: bool) -> EngineResult {
        if all {
            self.staged.clear();
        }
        Ok(())
    }

    fn savepoint_set(&mut self, sv: &mut [u8]) -> EngineResult {
        let index = self.savepoints.len() as u64;
        // Demo-grade: clones the full op log (including old + new
        // images for every staged update). `O(rows × row_size ×
        // savepoints)` is intentional — a copy-on-write or persistent
        // op log would scale, but the reference engine prefers the
        // straightforward Vec<Op> shape.
        self.savepoints.push(self.staged.clone());
        if sv.len() >= 8 {
            sv[..8].copy_from_slice(&index.to_le_bytes());
        }
        Ok(())
    }

    fn savepoint_rollback(&mut self, sv: &[u8]) -> EngineResult {
        let index = match sv_index(sv) {
            Some(i) => i,
            None => return Ok(()),
        };
        if let Some(snapshot) = self.savepoints.get(index) {
            self.staged = snapshot.clone();
        }
        // ROLLBACK TO destroys later savepoints; drop their snapshots.
        self.savepoints.truncate(index.saturating_add(1));
        Ok(())
    }

    fn savepoint_release(&mut self, sv: &[u8]) -> EngineResult {
        // Keep staged work; drop this and later snapshots (LIFO).
        if let Some(index) = sv_index(sv) {
            self.savepoints.truncate(index);
        }
        Ok(())
    }
}

impl TrivialTxn {
    fn append(&mut self, table: &str, op: Op) {
        self.staged.entry(table.to_owned()).or_default().push(op);
    }
}

/// Replay `ops` against `table`'s committed store. Each variant chooses
/// the index-aware path when a [`crate::store::TableMeta`] is registered
/// and the byte-equality fallback otherwise.
fn replay(table: &str, ops: Vec<Op>) {
    let meta = store::lookup_meta(table);
    for op in ops {
        match op {
            Op::Insert(row) => insert_row(table, meta.as_ref(), row),
            Op::Update { old, new } => update_row(table, meta.as_ref(), &old, new),
            Op::Delete(row) => delete_row(table, meta.as_ref(), &row),
        }
    }
}

fn insert_row(table: &str, meta: Option<&store::TableMeta>, row: Vec<u8>) {
    let key = match meta {
        Some(m) => store::extract_key_from_row(&row, m),
        None => None,
    };
    match key {
        Some(k) => store::commit_keyed(table, vec![(k, row)]),
        None => store::commit_seq(table, vec![row]),
    }
}

fn update_row(table: &str, meta: Option<&store::TableMeta>, old: &[u8], new: Vec<u8>) {
    let (old_key, new_key) = match meta {
        Some(m) => (
            store::extract_key_from_row(old, m),
            store::extract_key_from_row(&new, m),
        ),
        None => (None, None),
    };
    match (old_key, new_key) {
        (Some(k_old), Some(k_new)) if k_old == k_new => {
            let _applied = store::replace_by_key(table, &k_old, new);
        }
        (Some(k_old), Some(k_new)) => {
            if store::remove_by_key(table, &k_old) {
                store::commit_keyed(table, vec![(k_new, new)]);
            }
        }
        _ => {
            // No schema info: fall back to byte-equality against the
            // committed pre-image. Two staged updates against the same
            // row both carry the committed `old` image, so the second
            // replay no longer finds it and silently drops the change.
            // Schemas with a primary key avoid this via the keyed path
            // above.
            let _applied = store::replace_by_bytes(table, old, new);
        }
    }
}

fn delete_row(table: &str, meta: Option<&store::TableMeta>, row: &[u8]) {
    let key = match meta {
        Some(m) => store::extract_key_from_row(row, m),
        None => None,
    };
    match key {
        Some(k) => {
            let _applied = store::remove_by_key(table, &k);
        }
        None => {
            let _applied = store::remove_by_bytes(table, row);
        }
    }
}
