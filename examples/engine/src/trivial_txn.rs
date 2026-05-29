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

/// The reference engine's per-connection transaction. It buffers each row write
/// per table; a transaction commit (`all`) flushes the counts to the shared
/// committed store, and a transaction rollback (`all`) discards them, so the
/// effect of COMMIT vs ROLLBACK is visible to later statements. A savepoint
/// snapshots the staged counts; rolling back to it restores that snapshot.
#[derive(Debug, Default)]
pub struct TrivialTxn {
    staged: HashMap<String, u64>,
    savepoints: Vec<HashMap<String, u64>>,
}

impl TxnSession for TrivialTxn {
    fn write_row(&mut self, table: &str, _row: &[u8]) -> EngineResult {
        *self.staged.entry(table.to_owned()).or_insert(0) += 1;
        Ok(())
    }

    fn commit(&mut self, all: bool) -> EngineResult {
        // Flush only on the whole-transaction boundary; a statement boundary
        // (all=false) keeps the staged rows for the rest of the transaction.
        if all {
            for (table, n) in self.staged.drain() {
                store::commit_rows(&table, n);
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
        if sv.len() < 8 {
            return Ok(());
        }
        let mut idx = [0u8; 8];
        idx.copy_from_slice(&sv[..8]);
        let index = usize::try_from(u64::from_le_bytes(idx)).unwrap_or(usize::MAX);
        if let Some(snapshot) = self.savepoints.get(index) {
            self.staged = snapshot.clone();
        }
        Ok(())
    }
}
