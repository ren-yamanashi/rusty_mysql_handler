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
//! Storage is a sorted [`TableStore`](crate::store::TableStore) per
//! table (a `BTreeMap<Key, Vec<u8>>`). The single `impl StorageEngine
//! for TrivialEngine` in this file dispatches each trait method into
//! a thin helper on a sibling module:
//!
//! - `scan` — cursor / snapshot / range / index lookup machinery.
//! - `crud` — non-transactional `update_row` / `delete_row`.
//! - `stats` — `index_flags`, `records_in_range`, auto-increment.
//!
//! `update_row` / `delete_row` route through the per-connection
//! [`TrivialTxn`](crate::trivial_txn::TrivialTxn) op log whenever the
//! handlerton is registered as transactional, so `BEGIN..ROLLBACK`
//! discards the change and `BEGIN..COMMIT` replays it; the trait impls
//! here are the non-transactional fallback the shim picks when no txn
//! context is attached.
//!
//! **Line-limit note.** This file exceeds the 250-line ceiling because
//! its single responsibility is the `impl StorageEngine for
//! TrivialEngine` block (one `impl Trait for Type` per the coding-style
//! exemption). Helper code lives in `scan`, `crud`, and `stats`;
//! splitting the trait impl itself by method group would force every
//! virtual to grow a public re-export across two modules.

use std::ffi::CStr;

use mysql_handler::prelude::*;
use mysql_handler::sys::{self, HA_BINLOG_ROW_CAPABLE, HA_BINLOG_STMT_CAPABLE};

use crate::store::{self, Key, TableMeta};
use scan::ScanDir;

mod crud;
mod lookup;
mod scan;
mod stats;

/// The committed-row store keys by table name; `name` from create / open
/// is a path-like `./db/table`, so reduce it to the bare table name that
/// the write path (`table_name`) also uses.
fn table_key(name: &str) -> String {
    name.rsplit('/').next().unwrap_or(name).to_owned()
}

/// Reference engine backed by [`crate::store::TableStore`].
#[plugin(
    name = "RUSTY",
    description = "Rusty storage engine",
    version = 0x0001,
    license = License::Gpl,
    author = "ren-yamanashi",
    handlerton = crate::TrivialHandlerton,
)]
#[derive(Debug)]
pub struct TrivialEngine {
    pub(in crate::engine) table: String,
    /// Schema snapshot taken from `dd::Table` in `open` / `create`.
    pub(in crate::engine) meta: Option<TableMeta>,
    /// Index ordinal active for the current scan. Recorded by
    /// `index_init`; drives the snapshot reordering for secondary
    /// indexes and the column-set used when decoding search buffers.
    pub(in crate::engine) active_idx: usize,
    /// Per-statement snapshot of `(Key, row)` pairs in key order.
    /// `rnd_init` takes the full table keyed by primary; `index_init`
    /// re-keys the rows by `active_idx`'s columns; `read_range_first`
    /// refines to the relevant window.
    pub(in crate::engine) snapshot: Vec<(Key, Vec<u8>)>,
    /// Cursor into `snapshot`. `None` once the scan is exhausted.
    pub(in crate::engine) scan_pos: Option<usize>,
    /// Cursor direction; flipped to `Backward` by `index_last` /
    /// `index_prev`, `Forward` everywhere else.
    pub(in crate::engine) scan_dir: ScanDir,
    /// Search key recorded by `index_read_map`; `index_next_same` reads
    /// it back to stop when the cursor walks off the equality range.
    pub(in crate::engine) last_search_key: Option<Key>,
    pub(in crate::engine) next_auto_inc: u64,
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
        self.index_flags_for(idx)
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
        self.rnd_pos_at(buf, pos)
    }

    fn position(&mut self, _record: &[u8], ref_out: &mut [u8]) {
        self.write_position(ref_out);
    }

    fn update_row(&mut self, old: &[u8], new: &[u8]) -> EngineResult {
        self.apply_update(old, new)
    }

    fn delete_row(&mut self, buf: &[u8]) -> EngineResult {
        self.apply_delete(buf)
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
        self.index_read_at(buf, key, find_flag)
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
        self.index_next_same_at(buf, key)
    }

    fn records_in_range(
        &mut self,
        inx: u32,
        min: Option<RangeKey<'_>>,
        max: Option<RangeKey<'_>>,
    ) -> Option<u64> {
        self.records_in_range_for(inx, min, max)
    }

    fn read_range_first(
        &mut self,
        buf: &mut [u8],
        start: Option<RangeKey<'_>>,
        end: Option<RangeKey<'_>>,
        _eq_range: bool,
        _sorted: bool,
    ) -> EngineResult {
        self.read_range_first_at(buf, start, end)
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
        Some(self.reserve_auto_increment(increment, nb_desired))
    }

    fn reset(&mut self) -> EngineResult {
        self.snapshot.clear();
        self.scan_pos = None;
        self.last_search_key = None;
        Ok(())
    }
}
