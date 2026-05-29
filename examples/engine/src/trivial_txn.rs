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

use mysql_handler::engine::EngineResult;
use mysql_handler::hton::TxnSession;

/// The reference engine's per-connection transaction. It accepts both commit
/// and rollback boundaries and succeeds; the reference engine does not persist
/// rows, so there is nothing to flush or undo.
#[derive(Debug, Default)]
pub struct TrivialTxn;

impl TxnSession for TrivialTxn {
    fn commit(&mut self, _all: bool) -> EngineResult {
        Ok(())
    }

    fn rollback(&mut self, _all: bool) -> EngineResult {
        Ok(())
    }
}
