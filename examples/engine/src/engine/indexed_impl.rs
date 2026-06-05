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

//! `IndexedEngine` impl for [`TrivialEngine`]. The trait-method bodies
//! delegate to the same `engine/{lookup,scan,stats}` helpers the base
//! `StorageEngine` impl in [`super`] uses, so the split is purely about
//! keeping each `impl Trait for Type` block in its own file.

use mysql_handler::engine::{EngineResult, IndexedEngine, RKeyFunction, RangeKey};

use super::TrivialEngine;
use super::scan::ScanDir;

impl IndexedEngine for TrivialEngine {
    fn index_flags(&self, idx: u32, _part: u32, _all_parts: bool) -> u32 {
        self.index_flags_for(idx)
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
}
