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
//! `index_flags` advertises range / ordered / forward / backward
//! capabilities per index, gating `HA_READ_ORDER` and `HA_READ_PREV` on
//! the index actually being single-column ASCending — the only shape
//! whose natural [`Key`] order matches what the optimizer expects.
//!
//! `update_row` / `delete_row` route through the per-connection
//! [`TrivialTxn`](crate::trivial_txn::TrivialTxn) op log whenever the
//! handlerton is registered as transactional, so `BEGIN..ROLLBACK`
//! discards the change and `BEGIN..COMMIT` replays it. The
//! `StorageEngine::update_row` / `delete_row` impls below remain the
//! non-transactional fallback the shim chooses when no txn context is
//! attached to the connection.
//!
//! **Line-limit note.** This file exceeds the 250-line ceiling because
//! its single responsibility is the `impl StorageEngine for TrivialEngine`
//! block (one `impl Trait for Type` per the coding-style exemption).
//! Splitting cursor / key / range helpers out leaves the impl thin and
//! useful as a reference, so the assistants live in this file too.

use std::ffi::CStr;

use mysql_handler::engine::{EngineError, EngineResult, RKeyFunction, RangeKey, StorageEngine};
use mysql_handler::sys::{
    self, HA_BINLOG_ROW_CAPABLE, HA_BINLOG_STMT_CAPABLE, HA_READ_NEXT, HA_READ_ORDER, HA_READ_PREV,
    HA_READ_RANGE,
};

use crate::store::{self, IndexMeta, Key, TableMeta};

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

/// Which endpoint of a range a [`RangeKey`] describes. Used as the second
/// argument of [`TrivialEngine::decode_bound`] instead of a bare `bool`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Endpoint {
    Start,
    End,
}

/// Whether a decoded search key covers every part of the active index
/// or only a leading prefix. Used by [`TrivialEngine::decode_bound`] so
/// partial-prefix endpoints can choose the right
/// [`Key::next_prefix`] semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyShape {
    Full,
    Partial,
}

/// Reference engine backed by [`crate::store::TableStore`].
#[derive(Debug)]
pub struct TrivialEngine {
    table: String,
    /// Schema snapshot taken from `dd::Table` in `open` / `create`.
    meta: Option<TableMeta>,
    /// Index ordinal active for the current scan. Recorded by
    /// `index_init`; drives the snapshot reordering for secondary
    /// indexes and the column-set used when decoding search buffers.
    active_idx: usize,
    /// Per-statement snapshot of `(Key, row)` pairs in key order.
    /// `rnd_init` takes the full table keyed by primary; `index_init`
    /// re-keys the rows by `active_idx`'s columns; `read_range_first`
    /// refines to the relevant window.
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
            active_idx: 0,
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

    /// Refresh the cursor with a snapshot sorted by the active index.
    /// Primary scans walk the natural BTree order; secondary scans
    /// re-extract keys under the secondary's columns and re-sort.
    fn refresh_snapshot(&mut self) {
        let primary = store::pairs_sorted(&self.table);
        self.snapshot = match self.active_secondary_index() {
            Some(idx) => Self::resort_by_secondary(primary, self.meta.as_ref(), idx),
            None => primary,
        };
        self.scan_pos = (!self.snapshot.is_empty()).then_some(0);
        self.scan_dir = ScanDir::Forward;
        self.last_search_key = None;
    }

    /// The active index's [`IndexMeta`] when it is *not* the primary
    /// one, `None` otherwise (primary scans walk the natural BTree
    /// order).
    fn active_secondary_index(&self) -> Option<&IndexMeta> {
        let meta = self.meta.as_ref()?;
        if meta.primary_index_ordinal() == Some(self.active_idx) {
            return None;
        }
        meta.indexes().get(self.active_idx)
    }

    /// Re-key `primary` (rows already in primary order) by `index`'s
    /// columns and re-sort. Rows whose secondary key cannot be
    /// extracted (e.g. hidden columns, unsupported types) are dropped.
    fn resort_by_secondary(
        primary: Vec<(Key, Vec<u8>)>,
        meta: Option<&TableMeta>,
        index: &IndexMeta,
    ) -> Vec<(Key, Vec<u8>)> {
        let meta = match meta {
            Some(m) => m,
            None => return primary,
        };
        let mut out: Vec<(Key, Vec<u8>)> = primary
            .into_iter()
            .filter_map(|(_, row)| {
                let k = store::extract_index_key_from_row(&row, meta, index)?;
                Some((k, row))
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    /// Convert a [`RangeKey`] endpoint to a `Bound<Key>` suitable for
    /// the active index. Missing endpoints become
    /// [`std::ops::Bound::Unbounded`]. Type / schema mismatches fall
    /// back to `Unbounded` rather than yielding wrong rows.
    ///
    /// Partial-prefix buffers (`WHERE a = 1` on a `KEY (a, b)`) decode
    /// into a [`Key`] with fewer parts than the index's declared key
    /// parts. For the end endpoint of a partial-prefix range, the
    /// bound bumps to [`Key::next_prefix`] so the BTree walks across
    /// every row whose leading parts equal the prefix.
    fn decode_bound(
        meta: &TableMeta,
        index: &IndexMeta,
        endpoint: Option<RangeKey<'_>>,
        kind: Endpoint,
    ) -> std::ops::Bound<Key> {
        let rk = match endpoint {
            Some(r) => r,
            None => return std::ops::Bound::Unbounded,
        };
        let key = match store::decode_index_search_buffer(rk.key(), meta, index) {
            Some(k) => k,
            None => return std::ops::Bound::Unbounded,
        };
        let shape = if key.parts().len() < index.parts.len() {
            KeyShape::Partial
        } else {
            KeyShape::Full
        };
        let bumped = || match key.next_prefix() {
            Some(k) => std::ops::Bound::Excluded(k),
            None => std::ops::Bound::Unbounded,
        };
        match (rk.flag(), kind, shape) {
            // Partial-prefix `WHERE a > 1` on `KEY (a, b)` must skip
            // every `(1, *)` row; bumping to the next prefix sentinel
            // is the same trick that covers the End side of an
            // inclusive prefix range.
            (RKeyFunction::AfterKey, Endpoint::Start, KeyShape::Partial)
            | (
                RKeyFunction::AfterKey | RKeyFunction::KeyOrPrev,
                Endpoint::End,
                KeyShape::Partial,
            ) => bumped(),
            (RKeyFunction::AfterKey, Endpoint::Start, KeyShape::Full)
            | (RKeyFunction::BeforeKey, Endpoint::End, _) => std::ops::Bound::Excluded(key),
            _ => std::ops::Bound::Included(key),
        }
    }

    /// The active index's [`IndexMeta`], or the primary fallback.
    fn active_index<'a>(&self, meta: &'a TableMeta) -> Option<&'a IndexMeta> {
        match meta.indexes().get(self.active_idx) {
            Some(i) => Some(i),
            None => meta.primary_index(),
        }
    }

    /// Replace the cursor's snapshot with the rows in `[start, end]`.
    /// `read_range_first` may be called multiple times within a single
    /// `index_init` (multi-range plans, MRR, `WHERE id IN (..)`), so
    /// the snapshot is re-fetched and re-sorted under the active index
    /// rather than destructively shrinking the previous window.
    fn narrow_to_range(&mut self, start: &std::ops::Bound<Key>, end: &std::ops::Bound<Key>) {
        let fresh = store::pairs_sorted(&self.table);
        let resorted = match self.active_secondary_index() {
            Some(idx) => Self::resort_by_secondary(fresh, self.meta.as_ref(), idx),
            None => fresh,
        };
        self.snapshot = resorted
            .into_iter()
            .filter(|(k, _)| key_in_bounds(k, start, end))
            .collect();
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

    fn index_flags(&self, idx: u32, _part: u32, _all_parts: bool) -> u32 {
        // Range scans work for any index — the cursor honours
        // [`Key`]-ordered bounds either way. Ordered iteration only
        // matches MySQL's expectation when the index is single-column
        // ASC, so gate `HA_READ_ORDER` (and `HA_READ_PREV`, which only
        // makes sense once order is established) on the snapshot.
        let mut flags = HA_READ_NEXT | HA_READ_RANGE;
        let is_ordered = self
            .meta
            .as_ref()
            .and_then(|m| m.indexes().get(idx as usize))
            .is_some_and(IndexMeta::is_single_column_ascending);
        if is_ordered {
            flags |= HA_READ_ORDER | HA_READ_PREV;
        }
        flags
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
        // scan_pos points one past the yielded row; back off by one in
        // the scan direction. When the cursor has been exhausted, the
        // yielded row sat at the natural endpoint for that direction
        // (`len - 1` after a forward walk, `0` after a backward walk).
        let yielded_index = match (self.scan_pos, self.scan_dir) {
            (Some(p), ScanDir::Forward) => p.saturating_sub(1),
            (Some(p), ScanDir::Backward) => p.saturating_add(1),
            (None, ScanDir::Forward) => self.snapshot.len().saturating_sub(1),
            (None, ScanDir::Backward) => 0,
        } as u64;
        if ref_out.len() >= 8 {
            ref_out[..8].copy_from_slice(&yielded_index.to_le_bytes());
        }
    }

    fn update_row(&mut self, old: &[u8], new: &[u8]) -> EngineResult {
        let meta = match self.meta.as_ref() {
            Some(m) => m,
            None => return finish_replace(store::replace_by_bytes(&self.table, old, new.to_vec())),
        };
        let old_key = store::extract_key_from_row(old, meta);
        let new_key = store::extract_key_from_row(new, meta);
        match (old_key, new_key) {
            (Some(k_old), Some(k_new)) if k_old == k_new => {
                finish_replace(store::replace_by_key(&self.table, &k_old, new.to_vec()))
            }
            (Some(k_old), Some(k_new)) => {
                // Indexed column changed: drop the old entry and reinsert
                // under the new key so a later `WHERE id = new` finds it.
                let removed = store::remove_by_key(&self.table, &k_old);
                if !removed {
                    return Err(EngineError::EndOfFile);
                }
                store::commit_keyed(&self.table, vec![(k_new, new.to_vec())]);
                Ok(())
            }
            _ => finish_replace(store::replace_by_bytes(&self.table, old, new.to_vec())),
        }
    }

    fn delete_row(&mut self, buf: &[u8]) -> EngineResult {
        let key = match self.meta.as_ref() {
            Some(m) => store::extract_key_from_row(buf, m),
            None => None,
        };
        let removed = match key {
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

    fn index_init(&mut self, idx: u32, _sorted: bool) -> EngineResult {
        self.active_idx = idx as usize;
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
        let active = self.active_index(meta).ok_or(EngineError::WrongCommand)?;
        let target = store::decode_index_search_buffer(key, meta, active)
            .ok_or(EngineError::WrongCommand)?;
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

    fn index_next_same(&mut self, buf: &mut [u8], key: &[u8]) -> EngineResult {
        // Prefer the key recorded by `index_read_map` so a follow-up call
        // does not have to re-decode the same bytes. When the caller has
        // not been through `index_read_map` (e.g. `index_first` then
        // `index_next_same`), decode the supplied `key` through the same
        // path instead of giving up.
        let target = match self.last_search_key.clone() {
            Some(k) => k,
            None => {
                let meta = self.meta.as_ref().ok_or(EngineError::WrongCommand)?;
                let active = self.active_index(meta).ok_or(EngineError::WrongCommand)?;
                store::decode_index_search_buffer(key, meta, active)
                    .ok_or(EngineError::WrongCommand)?
            }
        };
        let pos = self.scan_pos.ok_or(EngineError::EndOfFile)?;
        let (k, _) = self.snapshot.get(pos).ok_or(EngineError::EndOfFile)?;
        if !key_matches_target(k, &target) {
            return Err(EngineError::EndOfFile);
        }
        self.scan_dir = ScanDir::Forward;
        self.yield_and_advance(buf)
    }

    fn records_in_range(
        &mut self,
        inx: u32,
        min: Option<RangeKey<'_>>,
        max: Option<RangeKey<'_>>,
    ) -> Option<u64> {
        let meta = self.meta.as_ref()?;
        let index = meta.indexes().get(inx as usize)?;
        let start = Self::decode_bound(meta, index, min, Endpoint::Start);
        let end = Self::decode_bound(meta, index, max, Endpoint::End);
        // Secondary: snapshot not yet built when records_in_range fires;
        // rekey on demand from the primary store. The primary path
        // counts directly off the BTreeMap.
        if meta.primary_index_ordinal() == Some(inx as usize) {
            return Some(store::range_len(&self.table, &start, &end));
        }
        let count = store::pairs_sorted(&self.table)
            .iter()
            .filter_map(|(_, row)| store::extract_index_key_from_row(row, meta, index))
            .filter(|k| key_in_bounds(k, &start, &end))
            .count();
        Some(u64::try_from(count).unwrap_or(u64::MAX))
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
        let active = self.active_index(meta).ok_or(EngineError::WrongCommand)?;
        let start_b = Self::decode_bound(meta, active, start, Endpoint::Start);
        let end_b = Self::decode_bound(meta, active, end, Endpoint::End);
        self.narrow_to_range(&start_b, &end_b);
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

/// `true` when `key` falls within `[start, end]` per the bound flags.
fn key_in_bounds(key: &Key, start: &std::ops::Bound<Key>, end: &std::ops::Bound<Key>) -> bool {
    let after_start = match start {
        std::ops::Bound::Unbounded => true,
        std::ops::Bound::Included(s) => key >= s,
        std::ops::Bound::Excluded(s) => key > s,
    };
    let before_end = match end {
        std::ops::Bound::Unbounded => true,
        std::ops::Bound::Included(e) => key <= e,
        std::ops::Bound::Excluded(e) => key < e,
    };
    after_start && before_end
}

/// True when `target` equals `k` or is a strict leading prefix of `k`
/// (composite-key partial-equality case).
fn key_matches_target(k: &Key, target: &Key) -> bool {
    if target.parts().len() > k.parts().len() {
        return false;
    }
    k.parts()[..target.parts().len()] == *target.parts()
}

/// `Ok(())` when the store reported a row was replaced, `EndOfFile`
/// otherwise — `update_row` / `delete_row` use this so a missed lookup
/// surfaces as MySQL's documented "no row matched" sentinel rather than a
/// silent success.
fn finish_replace(replaced: bool) -> EngineResult {
    if replaced {
        Ok(())
    } else {
        Err(EngineError::EndOfFile)
    }
}

/// Position the cursor at the first row matching `find_flag` against
/// `target`, returning the index into `snapshot` and the natural follow-up
/// direction. The forward / backward semantics mirror MySQL's:
/// `KeyExact` / `KeyOrNext` / `AfterKey` walk forward, `KeyOrPrev` /
/// `BeforeKey` walk backward, and unrecognised flags fall back to
/// `KeyOrNext`.
///
/// `KeyExact` against a partial target (`WHERE a = ?` on a composite
/// `KEY (a, b)`) matches every row whose leading parts equal the target,
/// not just rows whose full key equals it.
fn locate(
    snapshot: &[(Key, Vec<u8>)],
    target: &Key,
    find_flag: RKeyFunction,
) -> (Option<usize>, ScanDir) {
    match find_flag {
        RKeyFunction::KeyExact => (
            snapshot
                .iter()
                .position(|(k, _)| key_matches_target(k, target)),
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
    use mysql_handler::dd::{ColumnType, IndexElementOrder, IndexType};
    use store::{ColumnMeta, IndexMeta, KeyPartMeta, KeyValue};

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

    fn engine_with_index(parts: Vec<KeyPartMeta>) -> TrivialEngine {
        let column = ColumnMeta {
            column_type: ColumnType::Long,
            is_nullable: false,
            is_unsigned: false,
            char_length: 0,
            is_hidden: false,
        };
        let mut e = TrivialEngine::new();
        e.meta = Some(TableMeta::from_parts(
            vec![column],
            vec![IndexMeta {
                index_type: IndexType::Multiple,
                parts,
            }],
        ));
        e
    }

    fn part(column_ordinal: u32, order: IndexElementOrder) -> KeyPartMeta {
        KeyPartMeta {
            column_ordinal,
            order,
        }
    }

    #[test]
    fn index_flags_advertises_range_only_when_no_meta_is_registered() {
        let e = TrivialEngine::new();
        let f = e.index_flags(0, 0, false);
        assert!(f & HA_READ_RANGE != 0);
        assert!(f & HA_READ_NEXT != 0);
        assert!(f & HA_READ_ORDER == 0);
        assert!(f & HA_READ_PREV == 0);
    }

    #[test]
    fn index_flags_adds_order_for_single_column_ascending_index() {
        let e = engine_with_index(vec![part(1, IndexElementOrder::Ascending)]);
        let f = e.index_flags(0, 0, false);
        assert!(f & HA_READ_ORDER != 0);
        assert!(f & HA_READ_PREV != 0);
    }

    #[test]
    fn index_flags_drops_order_for_multi_column_index() {
        let e = engine_with_index(vec![
            part(1, IndexElementOrder::Ascending),
            part(2, IndexElementOrder::Ascending),
        ]);
        let f = e.index_flags(0, 0, false);
        assert!(f & HA_READ_RANGE != 0);
        assert!(f & HA_READ_ORDER == 0);
        assert!(f & HA_READ_PREV == 0);
    }

    #[test]
    fn index_flags_drops_order_for_descending_index() {
        let e = engine_with_index(vec![part(1, IndexElementOrder::Descending)]);
        let f = e.index_flags(0, 0, false);
        assert!(f & HA_READ_ORDER == 0);
        assert!(f & HA_READ_PREV == 0);
    }
}
