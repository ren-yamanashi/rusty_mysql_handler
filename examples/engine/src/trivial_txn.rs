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
/// effect of COMMIT vs ROLLBACK is visible to later statements.
#[derive(Debug, Default)]
pub struct TrivialTxn {
    staged: HashMap<String, u64>,
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
}
