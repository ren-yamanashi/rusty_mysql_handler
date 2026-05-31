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
    rnd_snapshot: Vec<Vec<u8>>,
    rnd_pos: usize,
    index_snapshot: Vec<Vec<u8>>,
    index_pos: usize,
    last_index_key: Vec<u8>,
    next_auto_inc: u64,
}

/// Offset where the first column's bytes start in MySQL's `record[0]`.
/// One NULL-bits byte precedes the first column for any table with at least
/// one nullable column, which is the common demo case (the `CREATE TABLE`
/// fixtures here all have one nullable column). A schema with zero nullable
/// columns or with the indexed column not first would need a less naive
/// layout decoder; that is out of scope for the reference engine.
const ROW_KEY_OFFSET: usize = 1;

impl TrivialEngine {
    /// New engine not yet bound to a table
    pub const fn new() -> Self {
        Self {
            table: String::new(),
            rnd_snapshot: Vec::new(),
            rnd_pos: 0,
            index_snapshot: Vec::new(),
            index_pos: 0,
            last_index_key: Vec::new(),
            next_auto_inc: 1,
        }
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

    /// Yield the row at `*pos` in `snapshot`, advance `*pos`, return
    /// `EndOfFile` once exhausted. Shared by the index-scan helpers below
    /// (all of them are essentially a sequential walk after the initial
    /// positioning).
    fn yield_from(snapshot: &[Vec<u8>], pos: &mut usize, buf: &mut [u8]) -> EngineResult {
        let row = match snapshot.get(*pos) {
            Some(r) => r,
            None => return Err(EngineError::EndOfFile),
        };
        Self::copy_row_into(buf, row);
        *pos += 1;
        Ok(())
    }

    /// True when `row`'s key column matches `key` byte-for-byte at the
    /// demo's fixed [`ROW_KEY_OFFSET`]. Treated as "the engine confirms
    /// this row satisfies the index lookup" by MySQL's `type=ref` path,
    /// which trusts the engine to filter and does not re-apply the
    /// `WHERE` predicate.
    fn row_matches_key(row: &[u8], key: &[u8]) -> bool {
        let end = ROW_KEY_OFFSET.saturating_add(key.len());
        row.get(ROW_KEY_OFFSET..end)
            .is_some_and(|slice| slice == key)
    }

    /// Advance `*pos` past `snapshot` looking for the next row whose key
    /// column matches `key`, yield its bytes, and return. `EndOfFile`
    /// when no further row matches.
    fn yield_next_matching(
        snapshot: &[Vec<u8>],
        pos: &mut usize,
        buf: &mut [u8],
        key: &[u8],
    ) -> EngineResult {
        while let Some(row) = snapshot.get(*pos) {
            *pos += 1;
            if Self::row_matches_key(row, key) {
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

    fn rnd_pos(&mut self, buf: &mut [u8], pos: &[u8]) -> EngineResult {
        let bytes: [u8; 8] = match pos.get(..8) {
            Some(b) => b.try_into().expect("8-byte slice"),
            None => return Err(EngineError::WrongCommand),
        };
        let idx = usize::try_from(u64::from_le_bytes(bytes)).unwrap_or(usize::MAX);
        let row = match self.rnd_snapshot.get(idx) {
            Some(r) => r,
            None => return Err(EngineError::EndOfFile),
        };
        Self::copy_row_into(buf, row);
        Ok(())
    }

    fn position(&mut self, _record: &[u8], ref_out: &mut [u8]) {
        // The row just yielded by rnd_next / index_* sits at pos-1 in the
        // active snapshot. MySQL hands the 8-byte ref back to rnd_pos on a
        // later re-read, so write it big enough for a u64 row index.
        let idx = self.rnd_pos.saturating_sub(1) as u64;
        if ref_out.len() >= 8 {
            ref_out[..8].copy_from_slice(&idx.to_le_bytes());
        }
    }

    fn update_row(&mut self, old: &[u8], new: &[u8]) -> EngineResult {
        // The demo updates the committed store directly. Inside an explicit
        // BEGIN..ROLLBACK the change will not be undone; the transactional
        // story is intentionally limited to INSERT for the reference engine.
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
        self.rnd_snapshot.clear();
        self.rnd_pos = 0;
        self.index_snapshot.clear();
        self.index_pos = 0;
        Ok(())
    }

    fn delete_all_rows(&mut self) -> EngineResult {
        store::reset_table(&self.table);
        self.rnd_snapshot.clear();
        self.rnd_pos = 0;
        self.index_snapshot.clear();
        self.index_pos = 0;
        Ok(())
    }

    fn index_init(&mut self, _idx: u32, _sorted: bool) -> EngineResult {
        self.index_snapshot = store::committed_rows(&self.table);
        self.index_pos = 0;
        self.last_index_key.clear();
        Ok(())
    }

    fn index_end(&mut self) -> EngineResult {
        self.index_snapshot.clear();
        self.index_pos = 0;
        self.last_index_key.clear();
        Ok(())
    }

    // The demo's index path is a linear scan over the committed rows that
    // matches each row's first column bytes against the key MySQL provides.
    // Sorted iteration (index_first / last in key order) and range filtering
    // beyond exact-match are intentionally not implemented; MySQL falls back
    // to its own filter for ORDER BY / range queries where this matters.

    fn index_read_map(
        &mut self,
        buf: &mut [u8],
        key: &[u8],
        _find_flag: RKeyFunction,
    ) -> EngineResult {
        self.index_pos = 0;
        self.last_index_key = key.to_vec();
        Self::yield_next_matching(&self.index_snapshot, &mut self.index_pos, buf, key)
    }

    fn index_next(&mut self, buf: &mut [u8]) -> EngineResult {
        // If we are still serving an index_read_map / index_next_same scan,
        // honour the remembered key so MySQL's type=ref plan does not see
        // non-matching rows. Otherwise (index_first followed by index_next)
        // do a plain sequential walk.
        if self.last_index_key.is_empty() {
            Self::yield_from(&self.index_snapshot, &mut self.index_pos, buf)
        } else {
            let key = self.last_index_key.clone();
            Self::yield_next_matching(&self.index_snapshot, &mut self.index_pos, buf, &key)
        }
    }

    fn index_prev(&mut self, buf: &mut [u8]) -> EngineResult {
        // The demo never sorted the snapshot, so prev is the same linear
        // pass forward — fine for WHERE filtering, wrong for an ORDER BY DESC
        // that relies on engine ordering.
        Self::yield_from(&self.index_snapshot, &mut self.index_pos, buf)
    }

    fn index_first(&mut self, buf: &mut [u8]) -> EngineResult {
        self.index_pos = 0;
        self.last_index_key.clear();
        Self::yield_from(&self.index_snapshot, &mut self.index_pos, buf)
    }

    fn index_last(&mut self, buf: &mut [u8]) -> EngineResult {
        // Same caveat as index_prev: no sort in the snapshot.
        self.index_pos = 0;
        self.last_index_key.clear();
        Self::yield_from(&self.index_snapshot, &mut self.index_pos, buf)
    }

    fn index_next_same(&mut self, buf: &mut [u8], key: &[u8]) -> EngineResult {
        Self::yield_next_matching(&self.index_snapshot, &mut self.index_pos, buf, key)
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
        self.index_pos = 0;
        Self::yield_from(&self.index_snapshot, &mut self.index_pos, buf)
    }

    fn read_range_next(&mut self, buf: &mut [u8]) -> EngineResult {
        Self::yield_from(&self.index_snapshot, &mut self.index_pos, buf)
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
        self.rnd_pos = 0;
        self.rnd_snapshot.clear();
        self.index_pos = 0;
        self.index_snapshot.clear();
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
    fn row_matches_key_compares_at_fixed_offset() {
        // row layout: [null-bits, id (4 bytes LE), ...]
        let row_20 = [0xFE, 20, 0, 0, 0, 1, b'b', 0];
        assert!(TrivialEngine::row_matches_key(&row_20, &[20, 0, 0, 0]));
        assert!(!TrivialEngine::row_matches_key(&row_20, &[21, 0, 0, 0]));
    }

    #[test]
    fn row_matches_key_rejects_short_rows() {
        // A row shorter than the offset + key cannot match.
        let row = [0xFE, 1];
        assert!(!TrivialEngine::row_matches_key(&row, &[1, 0, 0, 0]));
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
        let r = TrivialEngine::yield_next_matching(&rows, &mut pos, &mut buf, &[20, 0, 0, 0]);
        assert!(r.is_ok());
        assert_eq!(buf, [0xFE, 20, 0, 0, 0, 1, b'b', 0]);
        assert_eq!(pos, 2);
        // No further match → EndOfFile, pos advances past the end.
        let r2 = TrivialEngine::yield_next_matching(&rows, &mut pos, &mut buf, &[20, 0, 0, 0]);
        assert!(matches!(r2, Err(EngineError::EndOfFile)));
        assert_eq!(pos, 3);
    }
}
