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

//! Minimal engine-level handlerton for the reference engine.

use mysql_handler::engine::EngineResult;
use mysql_handler::hton::{HaStatType, Handlerton, HtonCapabilities, StatPrintSink, TxnSession};
use mysql_handler::sys;

use crate::TrivialTxn;

/// The reference engine's handlerton. Declares the `TRANSACTIONS` capability and
/// hands out a [`TrivialTxn`] per connection so the commit / rollback callbacks
/// are wired and exercised; everything else keeps the zero-config defaults
/// except `show_status`, which emits a single demo row so the binding is
/// exercised by `SHOW ENGINE RUSTY STATUS`.
#[derive(Debug, Default)]
pub struct TrivialHandlerton;

impl Handlerton for TrivialHandlerton {
    fn capabilities(&self) -> HtonCapabilities {
        // TABLESPACES opts the trivial engine into the tablespace callbacks so
        // CREATE / DROP TABLESPACE land on `alter_tablespace`; the trait
        // defaults accept anything, so MySQL records the tablespace in DD and
        // the callback wiring is exercised end to end.
        HtonCapabilities::TRANSACTIONS
            | HtonCapabilities::SAVEPOINTS
            | HtonCapabilities::TABLESPACES
    }

    fn savepoint_offset(&self) -> u32 {
        // 8 bytes hold the u64 snapshot index TrivialTxn writes per savepoint
        8
    }

    fn begin_transaction(&self) -> Box<dyn TxnSession> {
        Box::new(TrivialTxn::default())
    }

    fn show_status(
        &self,
        _thd: Option<&sys::THD>,
        sink: &StatPrintSink<'_>,
        stat: HaStatType,
    ) -> EngineResult {
        if matches!(stat, HaStatType::Status) {
            let _ = sink.emit("RUSTY", "state", "ok");
        }
        Ok(())
    }
}
