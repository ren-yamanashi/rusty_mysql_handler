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

//! `TrivialEngine`: the reference `StorageEngine` implementation. The single
//! `impl StorageEngine for TrivialEngine` block cannot be split across files
//! (one trait impl per type), so — like the `StorageEngine` trait in
//! `mysql-handler` — this file is exempt from the 250-line limit and grows as
//! more handler methods gain meaningful example overrides.

use std::ffi::CStr;

use mysql_handler::engine::{EngineError, EngineResult, RKeyFunction, RangeKey, StorageEngine};
use mysql_handler::sys::{self, HA_BINLOG_ROW_CAPABLE, HA_BINLOG_STMT_CAPABLE};

/// Trivial in-memory engine yielding a fixed number of empty rows
#[derive(Debug)]
pub struct TrivialEngine {
    num_rows: u32,
    current_row: u32,
    next_auto_inc: u64,
}

impl TrivialEngine {
    /// New engine that yields three empty rows
    pub const fn new() -> Self {
        Self {
            num_rows: 3,
            current_row: 0,
            next_auto_inc: 1,
        }
    }

    /// Yield the next empty row, or `EndOfFile` once `num_rows` are exhausted.
    /// Shared by the random-scan and index-scan read paths.
    fn yield_next(&mut self) -> EngineResult {
        if self.current_row >= self.num_rows {
            return Err(EngineError::EndOfFile);
        }
        self.current_row += 1;
        Ok(())
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

    fn rnd_end(&mut self) -> EngineResult {
        self.current_row = 0;
        Ok(())
    }

    fn rnd_next(&mut self, _buf: &mut [u8]) -> EngineResult {
        self.yield_next()
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
        self.num_rows = self.num_rows.saturating_add(1);
        Ok(())
    }

    fn delete_all_rows(&mut self) -> EngineResult {
        self.num_rows = 0;
        self.current_row = 0;
        Ok(())
    }

    fn index_init(&mut self, _idx: u32, _sorted: bool) -> EngineResult {
        self.current_row = 0;
        Ok(())
    }

    fn index_end(&mut self) -> EngineResult {
        Ok(())
    }

    fn index_read_map(
        &mut self,
        _buf: &mut [u8],
        _key: &[u8],
        _find_flag: RKeyFunction,
    ) -> EngineResult {
        self.current_row = 0;
        self.yield_next()
    }

    fn index_next(&mut self, _buf: &mut [u8]) -> EngineResult {
        self.yield_next()
    }

    fn index_prev(&mut self, _buf: &mut [u8]) -> EngineResult {
        self.yield_next()
    }

    fn index_first(&mut self, _buf: &mut [u8]) -> EngineResult {
        self.current_row = 0;
        self.yield_next()
    }

    fn index_last(&mut self, _buf: &mut [u8]) -> EngineResult {
        self.current_row = 0;
        self.yield_next()
    }

    fn index_next_same(&mut self, _buf: &mut [u8], _key: &[u8]) -> EngineResult {
        self.yield_next()
    }

    fn records_in_range(
        &mut self,
        _inx: u32,
        _min: Option<RangeKey<'_>>,
        _max: Option<RangeKey<'_>>,
    ) -> Option<u64> {
        None
    }

    fn read_range_first(
        &mut self,
        _buf: &mut [u8],
        _start: Option<RangeKey<'_>>,
        _end: Option<RangeKey<'_>>,
        _eq_range: bool,
        _sorted: bool,
    ) -> EngineResult {
        self.current_row = 0;
        self.yield_next()
    }

    fn read_range_next(&mut self, _buf: &mut [u8]) -> EngineResult {
        self.yield_next()
    }

    fn max_supported_keys(&self) -> Option<u32> {
        Some(8)
    }

    fn max_supported_key_parts(&self) -> Option<u32> {
        Some(8)
    }

    fn scan_time(&mut self) -> Option<f64> {
        Some(f64::from(self.num_rows))
    }

    fn records(&mut self) -> Option<EngineResult<u64>> {
        Some(Ok(u64::from(self.num_rows)))
    }

    fn estimate_rows_upper_bound(&mut self) -> Option<u64> {
        Some(u64::from(self.num_rows))
    }

    fn get_auto_increment(
        &mut self,
        _offset: u64,
        increment: u64,
        nb_desired: u64,
    ) -> Option<(u64, u64)> {
        let first = self.next_auto_inc;
        let reserved = nb_desired.max(1);
        self.next_auto_inc = first.saturating_add(reserved.saturating_mul(increment.max(1)));
        Some((first, reserved))
    }
}
