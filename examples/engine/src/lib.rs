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

//! Minimal storage engine for testing `mysql-handler`; `rnd_next` yields
//! three empty rows then `EndOfFile`.

#![allow(unsafe_code)]

use std::ffi::CStr;

use mysql_handler::engine::{EngineError, EngineResult, StorageEngine};
use mysql_handler::ffi::register_engine_factory;
use mysql_handler::panic_guard::FfiBoundary;
use mysql_handler::sys::{self, HA_BINLOG_ROW_CAPABLE, HA_BINLOG_STMT_CAPABLE};

#[cfg(not(test))]
#[doc(hidden)]
#[allow(missing_docs, missing_debug_implementations)]
pub mod plugin_manifest;

/// Trivial in-memory engine yielding a fixed number of empty rows
#[derive(Debug)]
pub struct TrivialEngine {
    num_rows: u32,
    current_row: u32,
}

impl TrivialEngine {
    /// New engine that yields three empty rows
    pub const fn new() -> Self {
        Self {
            num_rows: 3,
            current_row: 0,
        }
    }
}

impl Default for TrivialEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageEngine for TrivialEngine {
    fn table_type(&self) -> &'static CStr {
        c"RUSTY"
    }

    fn table_flags(&self) -> u64 {
        HA_BINLOG_STMT_CAPABLE | HA_BINLOG_ROW_CAPABLE
    }

    fn index_flags(&self, _idx: u32, _part: u32, _all_parts: bool) -> u32 {
        0
    }

    fn create(&mut self, _name: &str) -> EngineResult {
        Ok(())
    }

    fn open(&mut self, _name: &str, _mode: i32) -> EngineResult {
        Ok(())
    }

    fn close(&mut self) -> EngineResult {
        Ok(())
    }

    fn rnd_init(&mut self, _scan: bool) -> EngineResult {
        self.current_row = 0;
        Ok(())
    }

    fn rnd_next(&mut self, _buf: &mut [u8]) -> EngineResult {
        if self.current_row >= self.num_rows {
            return Err(EngineError::EndOfFile);
        }
        self.current_row += 1;
        Ok(())
    }

    fn rnd_pos(&mut self, _buf: &mut [u8], _pos: &[u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    fn position(&mut self, _record: &[u8]) {}

    fn info(&mut self, _flag: u32) -> EngineResult {
        Ok(())
    }

    fn delete_table(&mut self, _name: &str, _table_def: Option<&sys::DdTable>) -> EngineResult {
        Ok(())
    }

    fn rename_table(
        &mut self,
        _from: &str,
        _to: &str,
        _from_table_def: Option<&sys::DdTable>,
        _to_table_def: Option<&sys::DdTable>,
    ) -> EngineResult {
        Ok(())
    }

    fn drop_table(&mut self, _name: &str) {}

    fn truncate(&mut self, _table_def: Option<&sys::DdTable>) -> EngineResult {
        self.num_rows = 0;
        self.current_row = 0;
        Ok(())
    }

    fn write_row(&mut self, _buf: &[u8]) -> EngineResult {
        self.num_rows += 1;
        Ok(())
    }

    fn delete_all_rows(&mut self) -> EngineResult {
        self.num_rows = 0;
        self.current_row = 0;
        Ok(())
    }
}

/// Plugin entry point; the shim calls this once at `INSTALL PLUGIN`.
///
/// # Safety
/// Called once from `rusty_init_func` on the mysqld thread running
/// `INSTALL PLUGIN`. Panic-safe via [`FfiBoundary::run_void`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__plugin_init() {
    FfiBoundary::run_void(|| {
        register_engine_factory(|| Box::new(TrivialEngine::default()));
    });
}
