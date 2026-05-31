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

//! `TrivialEngine`: the reference `StorageEngine` implementation.

use std::ffi::CStr;

use mysql_handler::engine::{EngineError, EngineResult, RKeyFunction, RangeKey, StorageEngine};
use mysql_handler::sys::{self, HA_BINLOG_ROW_CAPABLE, HA_BINLOG_STMT_CAPABLE};

use crate::store;

/// The committed-row store keys by table name; `name` from create / open is a
/// path-like `./db/table`, so reduce it to the bare table name that the write
/// path (`table_name`) also uses.
fn table_key(name: &str) -> String {
    name.rsplit('/').next().unwrap_or(name).to_owned()
}

/// Reference engine that scans the rows committed to its table via the shared
/// committed-row store, so a committed transaction becomes visible to the
/// fresh handler a later statement opens
#[derive(Debug)]
pub struct TrivialEngine {
    table: String,
    current_row: u64,
    rnd_snapshot: Vec<Vec<u8>>,
    rnd_pos: usize,
    next_auto_inc: u64,
}

impl TrivialEngine {
    /// New engine not yet bound to a table
    pub const fn new() -> Self {
        Self {
            table: String::new(),
            current_row: 0,
            rnd_snapshot: Vec::new(),
            rnd_pos: 0,
            next_auto_inc: 1,
        }
    }

    /// Yield the next committed row's count slot, or `EndOfFile` once
    /// exhausted. Used by the index-scan paths that do not yet copy row
    /// bytes back to the caller; the random-scan path (`rnd_next`) goes
    /// through `rnd_snapshot` and copies real bytes instead.
    fn yield_next(&mut self) -> EngineResult {
        if self.current_row >= store::committed_row_count(&self.table) {
            return Err(EngineError::EndOfFile);
        }
        self.current_row += 1;
        Ok(())
    }

    /// Copy `row` into `buf`, truncating to the shorter length so a
    /// mis-sized demo buffer does not panic. A length mismatch indicates
    /// a `rec_buff_length` / schema bug; the demo silently truncates and
    /// the dropped tail is invisible — a production engine would treat
    /// the mismatch as an error.
    fn copy_row_into(buf: &mut [u8], row: &[u8]) {
        let n = buf.len().min(row.len());
        buf[..n].copy_from_slice(&row[..n]);
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

    fn create(&mut self, name: &str) -> EngineResult {
        self.table = table_key(name);
        Ok(())
    }

    fn open(&mut self, name: &str, _mode: i32) -> EngineResult {
        self.table = table_key(name);
        Ok(())
    }

    fn close(&mut self) -> EngineResult {
        Ok(())
    }

    fn rnd_init(&mut self, _scan: bool) -> EngineResult {
        self.rnd_snapshot = store::committed_rows(&self.table);
        self.rnd_pos = 0;
        Ok(())
    }

    fn rnd_end(&mut self) -> EngineResult {
        self.rnd_snapshot.clear();
        self.rnd_pos = 0;
        Ok(())
    }

    fn rnd_next(&mut self, buf: &mut [u8]) -> EngineResult {
        let row = match self.rnd_snapshot.get(self.rnd_pos) {
            Some(r) => r,
            None => return Err(EngineError::EndOfFile),
        };
        Self::copy_row_into(buf, row);
        self.rnd_pos += 1;
        Ok(())
    }

    fn rnd_pos(&mut self, _buf: &mut [u8], _pos: &[u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    fn position(&mut self, _record: &[u8], _ref_out: &mut [u8]) {}

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
        store::reset_table(&self.table);
        self.current_row = 0;
        self.rnd_snapshot.clear();
        self.rnd_pos = 0;
        Ok(())
    }

    fn delete_all_rows(&mut self) -> EngineResult {
        store::reset_table(&self.table);
        self.current_row = 0;
        self.rnd_snapshot.clear();
        self.rnd_pos = 0;
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
        let rows = u32::try_from(store::committed_row_count(&self.table)).unwrap_or(u32::MAX);
        Some(f64::from(rows))
    }

    fn records(&mut self) -> Option<EngineResult<u64>> {
        Some(Ok(store::committed_row_count(&self.table)))
    }

    fn estimate_rows_upper_bound(&mut self) -> Option<u64> {
        Some(store::committed_row_count(&self.table))
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

    fn reset(&mut self) -> EngineResult {
        self.current_row = 0;
        self.rnd_pos = 0;
        self.rnd_snapshot.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_row_into_writes_full_row_when_lengths_match() {
        let mut buf = [0u8; 4];
        TrivialEngine::copy_row_into(&mut buf, &[1, 2, 3, 4]);
        assert_eq!(buf, [1, 2, 3, 4]);
    }

    #[test]
    fn copy_row_into_truncates_to_buf_length() {
        let mut buf = [0u8; 2];
        TrivialEngine::copy_row_into(&mut buf, &[1, 2, 3, 4]);
        assert_eq!(buf, [1, 2]);
    }

    #[test]
    fn copy_row_into_leaves_trailing_bytes_when_row_is_shorter() {
        let mut buf = [9u8; 4];
        TrivialEngine::copy_row_into(&mut buf, &[1, 2]);
        assert_eq!(buf, [1, 2, 9, 9]);
    }

    #[test]
    fn table_key_strips_path_prefix() {
        assert_eq!(table_key("./e2e/rv"), "rv");
        assert_eq!(table_key("rv"), "rv");
        assert_eq!(table_key("/abs/db/t1"), "t1");
    }
}
