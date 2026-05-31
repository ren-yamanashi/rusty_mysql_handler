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
//!
//! Index scan is a linear pass with a first-column key match whose byte
//! offset comes from [`store::TableMeta`] (populated from `dd::Table` in
//! `open` / `create`). No sorted iteration. `update_row` / `delete_row`
//! mutate the committed store directly, so `BEGIN..ROLLBACK` does **not**
//! undo them — the transactional path stays limited to `INSERT` via
//! [`TxnSession`](crate::trivial_txn::TrivialTxn).

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
    /// Schema snapshot taken from `dd::Table` in `open` / `create`; drives the
    /// key-column offset used by `index_read_map`'s linear key match.
    meta: Option<store::TableMeta>,
    /// Per-statement snapshot; populated by `rnd_init` / `index_init`.
    snapshot: Vec<Vec<u8>>,
    /// Cursor into `snapshot`. `position()` writes `scan_pos - 1`.
    scan_pos: usize,
    /// Key from the last `index_read_map`; cleared on sequential walks.
    last_index_key: Vec<u8>,
    next_auto_inc: u64,
}

/// Fallback first-column offset used when no [`store::TableMeta`] is
/// available: one NULL-bits byte for the single-nullable-column fixtures
/// the demo uses. Real schemas always populate `meta`, but `delete_table`
/// and other code paths called before `open` see `None`.
const DEFAULT_KEY_OFFSET: usize = 1;

impl TrivialEngine {
    /// New engine not yet bound to a table
    pub const fn new() -> Self {
        Self {
            table: String::new(),
            meta: None,
            snapshot: Vec::new(),
            scan_pos: 0,
            last_index_key: Vec::new(),
            next_auto_inc: 1,
        }
    }

    /// Byte offset of the primary-key column in `record[0]`, resolved from
    /// `meta` when present; otherwise [`DEFAULT_KEY_OFFSET`].
    fn key_offset(&self) -> usize {
        self.meta
            .as_ref()
            .and_then(store::TableMeta::primary_key_offset)
            .unwrap_or(DEFAULT_KEY_OFFSET)
    }

    /// Copy `row` into `buf`, truncating to the shorter length.
    /// A length mismatch is a schema bug silently dropped for the demo.
    fn copy_row_into(buf: &mut [u8], row: &[u8]) {
        let n = buf.len().min(row.len());
        buf[..n].copy_from_slice(&row[..n]);
    }

    /// Yield `snapshot[*pos]`, advance `*pos`. `EndOfFile` once exhausted.
    fn yield_from(snapshot: &[Vec<u8>], pos: &mut usize, buf: &mut [u8]) -> EngineResult {
        let row = match snapshot.get(*pos) {
            Some(r) => r,
            None => return Err(EngineError::EndOfFile),
        };
        Self::copy_row_into(buf, row);
        *pos += 1;
        Ok(())
    }

    /// Byte-compare `row`'s key column against `key` at the byte offset the
    /// caller resolved from `TableMeta`. MySQL's `type=ref` trusts this
    /// filter. Empty `key` returns `false`.
    fn row_matches_key(row: &[u8], key: &[u8], offset: usize) -> bool {
        if key.is_empty() {
            return false;
        }
        let end = offset.saturating_add(key.len());
        row.get(offset..end).is_some_and(|slice| slice == key)
    }

    /// Yield the next `snapshot[*pos]` matching `key`. `EndOfFile` at end.
    fn yield_next_matching(
        snapshot: &[Vec<u8>],
        pos: &mut usize,
        buf: &mut [u8],
        key: &[u8],
        offset: usize,
    ) -> EngineResult {
        while let Some(row) = snapshot.get(*pos) {
            *pos += 1;
            if Self::row_matches_key(row, key, offset) {
                Self::copy_row_into(buf, row);
                return Ok(());
            }
        }
        Err(EngineError::EndOfFile)
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

    fn create(&mut self, name: &str, table_def: Option<&sys::DdTable>) -> EngineResult {
        self.table = table_key(name);
        self.meta = table_def.map(store::TableMeta::from_dd_table);
        Ok(())
    }

    fn open(&mut self, name: &str, _mode: i32, table_def: Option<&sys::DdTable>) -> EngineResult {
        self.table = table_key(name);
        self.meta = table_def.map(store::TableMeta::from_dd_table);
        Ok(())
    }

    fn close(&mut self) -> EngineResult {
        Ok(())
    }

    fn rnd_init(&mut self, _scan: bool) -> EngineResult {
        self.snapshot = store::committed_rows(&self.table);
        self.scan_pos = 0;
        Ok(())
    }

    fn rnd_end(&mut self) -> EngineResult {
        self.snapshot.clear();
        self.scan_pos = 0;
        Ok(())
    }

    fn rnd_next(&mut self, buf: &mut [u8]) -> EngineResult {
        Self::yield_from(&self.snapshot, &mut self.scan_pos, buf)
    }

    fn rnd_pos(&mut self, buf: &mut [u8], pos: &[u8]) -> EngineResult {
        let mut bytes = [0u8; 8];
        match pos.get(..8) {
            Some(b) => bytes.copy_from_slice(b),
            None => return Err(EngineError::WrongCommand),
        }
        let idx = match usize::try_from(u64::from_le_bytes(bytes)) {
            Ok(n) => n,
            Err(_) => usize::MAX,
        };
        let row = match self.snapshot.get(idx) {
            Some(r) => r,
            None => return Err(EngineError::EndOfFile),
        };
        Self::copy_row_into(buf, row);
        Ok(())
    }

    fn position(&mut self, _record: &[u8], ref_out: &mut [u8]) {
        // Encode the just-yielded row's snapshot index as u64 LE for rnd_pos.
        let idx = self.scan_pos.saturating_sub(1) as u64;
        if ref_out.len() >= 8 {
            ref_out[..8].copy_from_slice(&idx.to_le_bytes());
        }
    }

    fn update_row(&mut self, old: &[u8], new: &[u8]) -> EngineResult {
        if store::replace_row(&self.table, old, new) {
            Ok(())
        } else {
            Err(EngineError::EndOfFile)
        }
    }

    fn delete_row(&mut self, buf: &[u8]) -> EngineResult {
        if store::remove_row(&self.table, buf) {
            Ok(())
        } else {
            Err(EngineError::EndOfFile)
        }
    }

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
        self.snapshot.clear();
        self.scan_pos = 0;
        self.last_index_key.clear();
        Ok(())
    }

    fn delete_all_rows(&mut self) -> EngineResult {
        store::reset_table(&self.table);
        self.snapshot.clear();
        self.scan_pos = 0;
        self.last_index_key.clear();
        Ok(())
    }

    fn index_init(&mut self, _idx: u32, _sorted: bool) -> EngineResult {
        self.snapshot = store::committed_rows(&self.table);
        self.scan_pos = 0;
        self.last_index_key.clear();
        Ok(())
    }

    fn index_end(&mut self) -> EngineResult {
        self.snapshot.clear();
        self.scan_pos = 0;
        self.last_index_key.clear();
        Ok(())
    }

    fn index_read_map(
        &mut self,
        buf: &mut [u8],
        key: &[u8],
        _find_flag: RKeyFunction,
    ) -> EngineResult {
        self.scan_pos = 0;
        self.last_index_key = key.to_vec();
        let offset = self.key_offset();
        Self::yield_next_matching(&self.snapshot, &mut self.scan_pos, buf, key, offset)
    }

    fn index_next(&mut self, buf: &mut [u8]) -> EngineResult {
        // Honour the index_read_map key; empty means sequential walk.
        if self.last_index_key.is_empty() {
            Self::yield_from(&self.snapshot, &mut self.scan_pos, buf)
        } else {
            let key = self.last_index_key.clone();
            let offset = self.key_offset();
            Self::yield_next_matching(&self.snapshot, &mut self.scan_pos, buf, &key, offset)
        }
    }

    fn index_prev(&mut self, buf: &mut [u8]) -> EngineResult {
        // Clear key like index_first / index_last to avoid leaked filtering.
        self.last_index_key.clear();
        Self::yield_from(&self.snapshot, &mut self.scan_pos, buf)
    }

    fn index_first(&mut self, buf: &mut [u8]) -> EngineResult {
        self.scan_pos = 0;
        self.last_index_key.clear();
        Self::yield_from(&self.snapshot, &mut self.scan_pos, buf)
    }

    fn index_last(&mut self, buf: &mut [u8]) -> EngineResult {
        // The snapshot is unsorted, so this is just `index_first`.
        self.scan_pos = 0;
        self.last_index_key.clear();
        Self::yield_from(&self.snapshot, &mut self.scan_pos, buf)
    }

    fn index_next_same(&mut self, buf: &mut [u8], key: &[u8]) -> EngineResult {
        let offset = self.key_offset();
        Self::yield_next_matching(&self.snapshot, &mut self.scan_pos, buf, key, offset)
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
        buf: &mut [u8],
        _start: Option<RangeKey<'_>>,
        _end: Option<RangeKey<'_>>,
        _eq_range: bool,
        _sorted: bool,
    ) -> EngineResult {
        self.scan_pos = 0;
        self.last_index_key.clear();
        Self::yield_from(&self.snapshot, &mut self.scan_pos, buf)
    }

    fn read_range_next(&mut self, buf: &mut [u8]) -> EngineResult {
        Self::yield_from(&self.snapshot, &mut self.scan_pos, buf)
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
        self.snapshot.clear();
        self.scan_pos = 0;
        self.last_index_key.clear();
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

    #[test]
    fn row_matches_key_compares_at_given_offset() {
        // row layout: [null-bits, id (4 bytes LE), ...]
        let row_20 = [0xFE, 20, 0, 0, 0, 1, b'b', 0];
        assert!(TrivialEngine::row_matches_key(&row_20, &[20, 0, 0, 0], 1));
        assert!(!TrivialEngine::row_matches_key(&row_20, &[21, 0, 0, 0], 1));
    }

    #[test]
    fn row_matches_key_honours_a_non_default_offset() {
        // id at offset 2 (e.g. table with 9-16 nullable columns -> 2 null bytes).
        let row = [0xFE, 0xFE, 7, 0, 0, 0];
        assert!(TrivialEngine::row_matches_key(&row, &[7, 0, 0, 0], 2));
        assert!(!TrivialEngine::row_matches_key(&row, &[7, 0, 0, 0], 1));
    }

    #[test]
    fn row_matches_key_rejects_short_rows() {
        // A row shorter than the offset + key cannot match.
        let row = [0xFE, 1];
        assert!(!TrivialEngine::row_matches_key(&row, &[1, 0, 0, 0], 1));
    }

    #[test]
    fn yield_next_matching_skips_non_matches_and_advances_past_hit() {
        let rows = vec![
            vec![0xFE, 10, 0, 0, 0, 1, b'a', 0],
            vec![0xFE, 20, 0, 0, 0, 1, b'b', 0],
            vec![0xFE, 30, 0, 0, 0, 1, b'c', 0],
        ];
        let mut pos = 0usize;
        let mut buf = [0u8; 8];
        let r = TrivialEngine::yield_next_matching(&rows, &mut pos, &mut buf, &[20, 0, 0, 0], 1);
        assert!(r.is_ok());
        assert_eq!(buf, [0xFE, 20, 0, 0, 0, 1, b'b', 0]);
        assert_eq!(pos, 2);
        // No further match → EndOfFile, pos advances past the end.
        let r2 = TrivialEngine::yield_next_matching(&rows, &mut pos, &mut buf, &[20, 0, 0, 0], 1);
        assert!(matches!(r2, Err(EngineError::EndOfFile)));
        assert_eq!(pos, 3);
    }

    #[test]
    fn key_offset_falls_back_to_default_when_meta_is_none() {
        let eng = TrivialEngine::new();
        assert_eq!(eng.key_offset(), DEFAULT_KEY_OFFSET);
    }
}
