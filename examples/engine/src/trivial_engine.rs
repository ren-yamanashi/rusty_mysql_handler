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
//! Storage is a sorted [`TableStore`](crate::store::TableStore) per table
//! (a `BTreeMap<Key, Vec<u8>>`). Index scans walk the BTree directly via
//! a per-statement snapshot of `(Key, row)` pairs taken at `index_init`.
//! `index_flags` advertises `HA_READ_NEXT | HA_READ_PREV | HA_READ_ORDER
//! | HA_READ_RANGE` so the optimizer picks the index path for `BETWEEN`,
//! `ORDER BY`, and equality predicates instead of falling back to a
//! server-side filter on a full scan.
//!
//! `update_row` / `delete_row` still mutate the committed store directly,
//! so `BEGIN..ROLLBACK` does **not** undo them — the transactional path
//! stays limited to `INSERT` via
//! [`TrivialTxn`](crate::trivial_txn::TrivialTxn).

use std::ffi::CStr;

use mysql_handler::engine::{EngineError, EngineResult, RKeyFunction, RangeKey, StorageEngine};
use mysql_handler::sys::{
    self, HA_BINLOG_ROW_CAPABLE, HA_BINLOG_STMT_CAPABLE, HA_READ_NEXT, HA_READ_ORDER, HA_READ_PREV,
    HA_READ_RANGE,
};

use crate::store::{self, Key, TableMeta};

/// The committed-row store keys by table name; `name` from create / open is a
/// path-like `./db/table`, so reduce it to the bare table name that the write
/// path (`table_name`) also uses.
fn table_key(name: &str) -> String {
    name.rsplit('/').next().unwrap_or(name).to_owned()
}

/// Direction the cursor walks the [`TrivialEngine::snapshot`].
#[derive(Debug, Clone, Copy)]
enum ScanDir {
    Forward,
    Backward,
}

/// Reference engine backed by [`crate::store::TableStore`].
#[derive(Debug)]
pub struct TrivialEngine {
    table: String,
    /// Schema snapshot taken from `dd::Table` in `open` / `create`.
    meta: Option<TableMeta>,
    /// Per-statement snapshot of `(Key, row)` pairs in key order.
    /// `rnd_init` takes the full table; `index_init` and `read_range_first`
    /// refine to the relevant window.
    snapshot: Vec<(Key, Vec<u8>)>,
    /// Cursor into `snapshot`. `None` once the scan is exhausted.
    scan_pos: Option<usize>,
    /// Cursor direction; flipped to `Backward` by `index_last` /
    /// `index_prev`, `Forward` everywhere else.
    scan_dir: ScanDir,
    /// Search key recorded by `index_read_map`; `index_next_same` reads
    /// it back to stop when the cursor walks off the equality range.
    last_search_key: Option<Key>,
    next_auto_inc: u64,
}

impl TrivialEngine {
    /// New engine not yet bound to a table.
    pub const fn new() -> Self {
        Self {
            table: String::new(),
            meta: None,
            snapshot: Vec::new(),
            scan_pos: None,
            scan_dir: ScanDir::Forward,
            last_search_key: None,
            next_auto_inc: 1,
        }
    }

    /// Copy `row` into `buf`, truncating to the shorter length.
    fn copy_row_into(buf: &mut [u8], row: &[u8]) {
        let n = buf.len().min(row.len());
        buf[..n].copy_from_slice(&row[..n]);
    }

    /// Yield the row at `self.scan_pos`, then advance the cursor in the
    /// current direction. Returns [`EngineError::EndOfFile`] when the
    /// cursor has already walked off the end.
    fn yield_and_advance(&mut self, buf: &mut [u8]) -> EngineResult {
        let pos = self.scan_pos.ok_or(EngineError::EndOfFile)?;
        let (_, row) = self.snapshot.get(pos).ok_or(EngineError::EndOfFile)?;
        Self::copy_row_into(buf, row);
        self.scan_pos = match self.scan_dir {
            ScanDir::Forward => {
                let next = pos.saturating_add(1);
                (next < self.snapshot.len()).then_some(next)
            }
            ScanDir::Backward => (pos > 0).then(|| pos - 1),
        };
        Ok(())
    }

    /// Take a fresh sorted snapshot of the whole table and position the
    /// cursor at the start.
    fn refresh_snapshot(&mut self) {
        self.snapshot = store::pairs_sorted(&self.table);
        self.scan_pos = (!self.snapshot.is_empty()).then_some(0);
        self.scan_dir = ScanDir::Forward;
        self.last_search_key = None;
    }

    /// Convert a [`RangeKey`] endpoint to a `Bound<Key>` suitable for
    /// [`crate::store::range_pairs`]. Missing endpoints become
    /// [`std::ops::Bound::Unbounded`]. Type / schema mismatches fall back
    /// to `Unbounded` rather than yielding wrong rows.
    fn decode_bound(
        meta: &TableMeta,
        endpoint: Option<RangeKey<'_>>,
        is_start: bool,
    ) -> std::ops::Bound<Key> {
        let rk = match endpoint {
            Some(r) => r,
            None => return std::ops::Bound::Unbounded,
        };
        let key = match store::build_key_from_search_buffer(rk.key(), meta) {
            Some(k) => k,
            None => return std::ops::Bound::Unbounded,
        };
        match (rk.flag(), is_start) {
            (RKeyFunction::AfterKey, true) | (RKeyFunction::BeforeKey, false) => {
                std::ops::Bound::Excluded(key)
            }
            // KeyExact / KeyOrNext / KeyOrPrev all include the endpoint;
            // any unrecognised flag falls back to inclusive.
            _ => std::ops::Bound::Included(key),
        }
    }

    /// Refresh `snapshot` to the rows whose key falls in `[start, end]`,
    /// positioned at the first row.
    fn refresh_range(&mut self, start: &std::ops::Bound<Key>, end: &std::ops::Bound<Key>) {
        self.snapshot = store::range_pairs(&self.table, start, end);
        self.scan_pos = (!self.snapshot.is_empty()).then_some(0);
        self.scan_dir = ScanDir::Forward;
        self.last_search_key = None;
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
        HA_READ_NEXT | HA_READ_PREV | HA_READ_ORDER | HA_READ_RANGE
    }

    fn create(&mut self, name: &str, table_def: Option<&sys::DdTable>) -> EngineResult {
        self.table = table_key(name);
        self.meta = table_def.map(TableMeta::from_dd_table);
        if let Some(m) = self.meta.clone() {
            store::register_meta(&self.table, m);
        }
        Ok(())
    }

    fn open(&mut self, name: &str, _mode: i32, table_def: Option<&sys::DdTable>) -> EngineResult {
        self.table = table_key(name);
        self.meta = table_def.map(TableMeta::from_dd_table);
        if let Some(m) = self.meta.clone() {
            store::register_meta(&self.table, m);
        }
        Ok(())
    }

    fn close(&mut self) -> EngineResult {
        Ok(())
    }

    fn rnd_init(&mut self, _scan: bool) -> EngineResult {
        self.refresh_snapshot();
        Ok(())
    }

    fn rnd_end(&mut self) -> EngineResult {
        self.snapshot.clear();
        self.scan_pos = None;
        Ok(())
    }

    fn rnd_next(&mut self, buf: &mut [u8]) -> EngineResult {
        self.yield_and_advance(buf)
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
        let (_, row) = match self.snapshot.get(idx) {
            Some(p) => p,
            None => return Err(EngineError::EndOfFile),
        };
        Self::copy_row_into(buf, row);
        Ok(())
    }

    fn position(&mut self, _record: &[u8], ref_out: &mut [u8]) {
        // Encode the index of the row this scan just yielded as u64 LE.
        // `yield_and_advance` already moved `scan_pos`, so the yielded
        // row is the one just before whatever `scan_pos` now points at
        // (or the last element when `scan_pos` became `None`).
        let yielded_index = match (self.scan_pos, self.scan_dir) {
            (Some(p), ScanDir::Forward) => p.saturating_sub(1),
            (Some(p), ScanDir::Backward) => p.saturating_add(1),
            (None, _) => self.snapshot.len().saturating_sub(1),
        } as u64;
        if ref_out.len() >= 8 {
            ref_out[..8].copy_from_slice(&yielded_index.to_le_bytes());
        }
    }

    fn update_row(&mut self, old: &[u8], new: &[u8]) -> EngineResult {
        let replaced = match self
            .meta
            .as_ref()
            .and_then(|m| store::extract_key_from_row(old, m))
        {
            Some(k) => store::replace_by_key(&self.table, &k, new.to_vec()),
            None => store::replace_by_bytes(&self.table, old, new.to_vec()),
        };
        if replaced {
            Ok(())
        } else {
            Err(EngineError::EndOfFile)
        }
    }

    fn delete_row(&mut self, buf: &[u8]) -> EngineResult {
        let removed = match self
            .meta
            .as_ref()
            .and_then(|m| store::extract_key_from_row(buf, m))
        {
            Some(k) => store::remove_by_key(&self.table, &k),
            None => store::remove_by_bytes(&self.table, buf),
        };
        if removed {
            Ok(())
        } else {
            Err(EngineError::EndOfFile)
        }
    }

    fn info(&mut self, _flag: u32) -> EngineResult {
        Ok(())
    }

    fn delete_table(&mut self, name: &str, _table_def: Option<&sys::DdTable>) -> EngineResult {
        store::forget_table(&table_key(name));
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

    fn drop_table(&mut self, name: &str) {
        store::forget_table(&table_key(name));
    }

    fn truncate(&mut self, _table_def: Option<&sys::DdTable>) -> EngineResult {
        store::reset_table(&self.table);
        self.snapshot.clear();
        self.scan_pos = None;
        self.last_search_key = None;
        Ok(())
    }

    fn delete_all_rows(&mut self) -> EngineResult {
        store::reset_table(&self.table);
        self.snapshot.clear();
        self.scan_pos = None;
        self.last_search_key = None;
        Ok(())
    }

    fn index_init(&mut self, _idx: u32, _sorted: bool) -> EngineResult {
        self.refresh_snapshot();
        Ok(())
    }

    fn index_end(&mut self) -> EngineResult {
        self.snapshot.clear();
        self.scan_pos = None;
        self.last_search_key = None;
        Ok(())
    }

    fn index_read_map(
        &mut self,
        buf: &mut [u8],
        key: &[u8],
        find_flag: RKeyFunction,
    ) -> EngineResult {
        let meta = self.meta.as_ref().ok_or(EngineError::WrongCommand)?;
        let target =
            store::build_key_from_search_buffer(key, meta).ok_or(EngineError::WrongCommand)?;
        let (pos, dir) = locate(&self.snapshot, &target, find_flag);
        self.scan_pos = pos;
        self.scan_dir = dir;
        self.last_search_key = Some(target);
        self.yield_and_advance(buf)
    }

    fn index_next(&mut self, buf: &mut [u8]) -> EngineResult {
        self.scan_dir = ScanDir::Forward;
        self.yield_and_advance(buf)
    }

    fn index_prev(&mut self, buf: &mut [u8]) -> EngineResult {
        self.scan_dir = ScanDir::Backward;
        self.last_search_key = None;
        self.yield_and_advance(buf)
    }

    fn index_first(&mut self, buf: &mut [u8]) -> EngineResult {
        self.scan_pos = (!self.snapshot.is_empty()).then_some(0);
        self.scan_dir = ScanDir::Forward;
        self.last_search_key = None;
        self.yield_and_advance(buf)
    }

    fn index_last(&mut self, buf: &mut [u8]) -> EngineResult {
        self.scan_pos = self.snapshot.len().checked_sub(1);
        self.scan_dir = ScanDir::Backward;
        self.last_search_key = None;
        self.yield_and_advance(buf)
    }

    fn index_next_same(&mut self, buf: &mut [u8], _key: &[u8]) -> EngineResult {
        // Walk forward as long as the next row's key equals the one we
        // recorded in `index_read_map`. The trailing `key` buffer MySQL
        // hands here is the same bytes the previous `index_read_map` saw,
        // so we trust the recorded `Key` and avoid re-decoding.
        let target = match self.last_search_key.clone() {
            Some(k) => k,
            None => return Err(EngineError::EndOfFile),
        };
        let pos = self.scan_pos.ok_or(EngineError::EndOfFile)?;
        let (key, _) = self.snapshot.get(pos).ok_or(EngineError::EndOfFile)?;
        if key != &target {
            return Err(EngineError::EndOfFile);
        }
        self.scan_dir = ScanDir::Forward;
        self.yield_and_advance(buf)
    }

    fn records_in_range(
        &mut self,
        _inx: u32,
        min: Option<RangeKey<'_>>,
        max: Option<RangeKey<'_>>,
    ) -> Option<u64> {
        let meta = self.meta.as_ref()?;
        let start = Self::decode_bound(meta, min, true);
        let end = Self::decode_bound(meta, max, false);
        Some(store::range_pairs(&self.table, &start, &end).len() as u64)
    }

    fn read_range_first(
        &mut self,
        buf: &mut [u8],
        start: Option<RangeKey<'_>>,
        end: Option<RangeKey<'_>>,
        _eq_range: bool,
        _sorted: bool,
    ) -> EngineResult {
        let meta = self.meta.as_ref().ok_or(EngineError::WrongCommand)?;
        let start_b = Self::decode_bound(meta, start, true);
        let end_b = Self::decode_bound(meta, end, false);
        self.refresh_range(&start_b, &end_b);
        self.yield_and_advance(buf)
    }

    fn read_range_next(&mut self, buf: &mut [u8]) -> EngineResult {
        self.scan_dir = ScanDir::Forward;
        self.yield_and_advance(buf)
    }

    fn max_supported_keys(&self) -> Option<u32> {
        Some(8)
    }

    fn max_supported_key_parts(&self) -> Option<u32> {
        Some(8)
    }

    fn scan_time(&mut self) -> Option<f64> {
        let rows = u32::try_from(store::row_count(&self.table)).unwrap_or(u32::MAX);
        Some(f64::from(rows))
    }

    fn records(&mut self) -> Option<EngineResult<u64>> {
        Some(Ok(store::row_count(&self.table)))
    }

    fn estimate_rows_upper_bound(&mut self) -> Option<u64> {
        Some(store::row_count(&self.table))
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
        self.scan_pos = None;
        self.last_search_key = None;
        Ok(())
    }
}

/// Position the cursor at the first row matching `find_flag` against
/// `target`, returning the index into `snapshot` and the natural follow-up
/// direction. The forward / backward semantics mirror MySQL's:
/// `KeyExact` / `KeyOrNext` / `AfterKey` walk forward, `KeyOrPrev` /
/// `BeforeKey` walk backward, and unrecognised flags fall back to
/// `KeyOrNext`.
fn locate(
    snapshot: &[(Key, Vec<u8>)],
    target: &Key,
    find_flag: RKeyFunction,
) -> (Option<usize>, ScanDir) {
    match find_flag {
        RKeyFunction::KeyExact => (
            snapshot.iter().position(|(k, _)| k == target),
            ScanDir::Forward,
        ),
        RKeyFunction::KeyOrNext => (
            snapshot.iter().position(|(k, _)| k >= target),
            ScanDir::Forward,
        ),
        RKeyFunction::AfterKey => (
            snapshot.iter().position(|(k, _)| k > target),
            ScanDir::Forward,
        ),
        RKeyFunction::KeyOrPrev => (
            snapshot.iter().rposition(|(k, _)| k <= target),
            ScanDir::Backward,
        ),
        RKeyFunction::BeforeKey => (
            snapshot.iter().rposition(|(k, _)| k < target),
            ScanDir::Backward,
        ),
        _ => (
            snapshot.iter().position(|(k, _)| k >= target),
            ScanDir::Forward,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mysql_handler::dd::ColumnType;
    use store::KeyValue;

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

    fn fake_snapshot(values: &[i64]) -> Vec<(Key, Vec<u8>)> {
        values
            .iter()
            .map(|n| (Key::single(KeyValue::Signed(*n)), vec![*n as u8]))
            .collect()
    }

    #[test]
    fn locate_key_exact_returns_the_matching_index() {
        let snap = fake_snapshot(&[10, 20, 30]);
        let target = Key::single(KeyValue::Signed(20));
        let (pos, _) = locate(&snap, &target, RKeyFunction::KeyExact);
        assert_eq!(pos, Some(1));
    }

    #[test]
    fn locate_key_exact_misses_when_no_match() {
        let snap = fake_snapshot(&[10, 20, 30]);
        let target = Key::single(KeyValue::Signed(25));
        let (pos, _) = locate(&snap, &target, RKeyFunction::KeyExact);
        assert_eq!(pos, None);
    }

    #[test]
    fn locate_key_or_next_returns_first_geq() {
        let snap = fake_snapshot(&[10, 20, 30]);
        let target = Key::single(KeyValue::Signed(15));
        let (pos, dir) = locate(&snap, &target, RKeyFunction::KeyOrNext);
        assert_eq!(pos, Some(1));
        assert!(matches!(dir, ScanDir::Forward));
    }

    #[test]
    fn locate_key_or_prev_returns_last_leq_and_backward() {
        let snap = fake_snapshot(&[10, 20, 30]);
        let target = Key::single(KeyValue::Signed(25));
        let (pos, dir) = locate(&snap, &target, RKeyFunction::KeyOrPrev);
        assert_eq!(pos, Some(1));
        assert!(matches!(dir, ScanDir::Backward));
    }

    #[test]
    fn yield_and_advance_walks_forward_then_terminates() {
        let mut e = TrivialEngine::new();
        e.snapshot = fake_snapshot(&[1, 2]);
        e.scan_pos = Some(0);
        e.scan_dir = ScanDir::Forward;
        let mut buf = [0u8; 1];
        assert!(e.yield_and_advance(&mut buf).is_ok());
        assert_eq!(buf, [1]);
        assert!(e.yield_and_advance(&mut buf).is_ok());
        assert_eq!(buf, [2]);
        assert!(matches!(
            e.yield_and_advance(&mut buf),
            Err(EngineError::EndOfFile)
        ));
    }

    #[test]
    fn yield_and_advance_walks_backward() {
        let mut e = TrivialEngine::new();
        e.snapshot = fake_snapshot(&[1, 2, 3]);
        e.scan_pos = Some(2);
        e.scan_dir = ScanDir::Backward;
        let mut buf = [0u8; 1];
        e.yield_and_advance(&mut buf).unwrap();
        assert_eq!(buf, [3]);
        e.yield_and_advance(&mut buf).unwrap();
        assert_eq!(buf, [2]);
        e.yield_and_advance(&mut buf).unwrap();
        assert_eq!(buf, [1]);
        assert!(matches!(
            e.yield_and_advance(&mut buf),
            Err(EngineError::EndOfFile)
        ));
    }

    #[test]
    fn index_flags_advertises_range_and_order_capabilities() {
        let e = TrivialEngine::new();
        let f = e.index_flags(0, 0, false);
        assert!(f & HA_READ_RANGE != 0);
        assert!(f & HA_READ_ORDER != 0);
        assert!(f & HA_READ_NEXT != 0);
        assert!(f & HA_READ_PREV != 0);
    }
}
