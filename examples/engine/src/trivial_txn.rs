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

use std::collections::HashMap;

use mysql_handler::engine::EngineResult;
use mysql_handler::hton::TxnSession;

use crate::store;

/// Read the snapshot-stack index `TrivialTxn` wrote into a savepoint's `sv`.
fn sv_index(sv: &[u8]) -> Option<usize> {
    let bytes: [u8; 8] = sv.get(..8)?.try_into().ok()?;
    usize::try_from(u64::from_le_bytes(bytes)).ok()
}

/// The reference engine's per-connection transaction. It buffers each row
/// write per table as a `Vec<Vec<u8>>` of raw `record[0]` byte images; a
/// transaction commit (`all`) flushes the staged rows to the shared committed
/// store, and a transaction rollback (`all`) discards them, so the effect of
/// COMMIT vs ROLLBACK is visible to later statements. A savepoint snapshots
/// the staged rows; rolling back to it restores that snapshot and drops later
/// snapshots, while releasing it drops its snapshot but keeps the work.
#[derive(Debug, Default)]
pub struct TrivialTxn {
    staged: HashMap<String, Vec<Vec<u8>>>,
    savepoints: Vec<HashMap<String, Vec<Vec<u8>>>>,
}

impl TxnSession for TrivialTxn {
    fn write_row(&mut self, table: &str, row: &[u8]) -> EngineResult {
        self.staged
            .entry(table.to_owned())
            .or_default()
            .push(row.to_vec());
        Ok(())
    }

    fn commit(&mut self, all: bool) -> EngineResult {
        // Flush only on the whole-transaction boundary; a statement boundary
        // (all=false) keeps the staged rows for the rest of the transaction.
        // The shim's `rusty_hton_commit` upgrades autocommit statement commits
        // to all=true so the demo path also flushes there.
        if all {
            for (table, rows) in self.staged.drain() {
                store::commit_rows(&table, rows);
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
        // ROLLBACK TO destroys savepoints established after this one, so drop
        // their snapshots to keep the stack bounded.
        self.savepoints.truncate(index.saturating_add(1));
        Ok(())
    }

    fn savepoint_release(&mut self, sv: &[u8]) -> EngineResult {
        // Releasing keeps the work (staged is untouched) and drops the
        // savepoint's snapshot (and later ones, LIFO) to bound the stack.
        if let Some(index) = sv_index(sv) {
            self.savepoints.truncate(index);
        }
        Ok(())
    }
}
