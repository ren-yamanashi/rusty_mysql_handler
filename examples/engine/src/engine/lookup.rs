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

//! Index point lookup + range bound decoding for [`TrivialEngine`].
//! `index_read_map` / `index_next_same` route through this module; the
//! cursor mechanics they trigger live in the sibling [`super::scan`].

use mysql_handler::engine::{EngineError, EngineResult, RKeyFunction, RangeKey};

use super::TrivialEngine;
use super::scan::{Endpoint, ScanDir};
use crate::store::{self, IndexMeta, Key, TableMeta};

/// Whether a decoded search key covers every part of the active index
/// or only a leading prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyShape {
    Full,
    Partial,
}

impl TrivialEngine {
    /// Convert a [`RangeKey`] endpoint to a `Bound<Key>` for the active
    /// index. Missing endpoints and decode failures fall back to
    /// `Unbounded`. Partial-prefix end bounds bump to
    /// [`Key::next_prefix`].
    pub(in crate::engine) fn decode_bound(
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

    /// Position the cursor for `index_read_map`'s requested semantics
    /// and yield the first match.
    pub(in crate::engine) fn index_read_at(
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

    /// Yield the next row matching the last `index_read_map` key (or
    /// the supplied `key` when no prior read_map ran).
    pub(in crate::engine) fn index_next_same_at(
        &mut self,
        buf: &mut [u8],
        key: &[u8],
    ) -> EngineResult {
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

    /// Set up the snapshot for a `read_range_first` window and yield
    /// the first row.
    pub(in crate::engine) fn read_range_first_at(
        &mut self,
        buf: &mut [u8],
        start: Option<RangeKey<'_>>,
        end: Option<RangeKey<'_>>,
    ) -> EngineResult {
        let meta = self.meta.as_ref().ok_or(EngineError::WrongCommand)?;
        let active = self.active_index(meta).ok_or(EngineError::WrongCommand)?;
        let start_b = Self::decode_bound(meta, active, start, Endpoint::Start);
        let end_b = Self::decode_bound(meta, active, end, Endpoint::End);
        self.narrow_to_range(&start_b, &end_b);
        self.yield_and_advance(buf)
    }
}

/// True when `target` equals `k` or is a strict leading prefix of `k`
/// (composite-key partial-equality case).
fn key_matches_target(k: &Key, target: &Key) -> bool {
    if target.parts().len() > k.parts().len() {
        return false;
    }
    k.parts()[..target.parts().len()] == *target.parts()
}

/// Position the cursor for `find_flag` against `target`, returning the
/// snapshot index and the natural follow-up direction.
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
