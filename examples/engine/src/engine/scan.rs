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

//! Cursor + snapshot primitives for [`TrivialEngine`]. Point lookups,
//! key decoding, and range bounds live in the sibling
//! [`super::lookup`] module so this file stays focused on cursor walk
//! and snapshot construction.

use mysql_handler::engine::{EngineError, EngineResult};

use super::TrivialEngine;
use crate::store::{self, IndexMeta, Key, TableMeta};

/// Direction the cursor walks the [`TrivialEngine::snapshot`].
#[derive(Debug, Clone, Copy)]
pub(in crate::engine) enum ScanDir {
    Forward,
    Backward,
}

/// Which endpoint of a range a [`RangeKey`](mysql_handler::engine::RangeKey)
/// describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::engine) enum Endpoint {
    Start,
    End,
}

impl TrivialEngine {
    /// Copy `row` into `buf`, truncating to the shorter length.
    pub(in crate::engine) fn copy_row_into(buf: &mut [u8], row: &[u8]) {
        let n = buf.len().min(row.len());
        buf[..n].copy_from_slice(&row[..n]);
    }

    /// Yield the row at `self.scan_pos`, then advance the cursor in the
    /// current direction.
    pub(in crate::engine) fn yield_and_advance(&mut self, buf: &mut [u8]) -> EngineResult {
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
    pub(in crate::engine) fn refresh_snapshot(&mut self) {
        let primary = store::pairs_sorted(&self.table);
        self.snapshot = match self.active_secondary_index() {
            Some(idx) => Self::resort_by_secondary(primary, self.meta.as_ref(), idx),
            None => primary,
        };
        self.scan_pos = (!self.snapshot.is_empty()).then_some(0);
        self.scan_dir = ScanDir::Forward;
        self.last_search_key = None;
    }

    /// The active index when it is *not* the primary one, `None`
    /// otherwise (primary scans walk the natural BTree order).
    pub(in crate::engine) fn active_secondary_index(&self) -> Option<&IndexMeta> {
        let meta = self.meta.as_ref()?;
        if meta.primary_index_ordinal() == Some(self.active_idx) {
            return None;
        }
        meta.indexes().get(self.active_idx)
    }

    /// Re-key `primary` by `index`'s columns and re-sort.
    pub(in crate::engine) fn resort_by_secondary(
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

    /// The active index, or the primary fallback.
    pub(in crate::engine) fn active_index<'a>(&self, meta: &'a TableMeta) -> Option<&'a IndexMeta> {
        match meta.indexes().get(self.active_idx) {
            Some(i) => Some(i),
            None => meta.primary_index(),
        }
    }

    /// Re-fetch and re-sort the snapshot under the active index, then
    /// filter to keys within `[start, end]`.
    pub(in crate::engine) fn narrow_to_range(
        &mut self,
        start: &std::ops::Bound<Key>,
        end: &std::ops::Bound<Key>,
    ) {
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

    /// Read the row at the snapshot index encoded in `pos`.
    pub(in crate::engine) fn rnd_pos_at(&mut self, buf: &mut [u8], pos: &[u8]) -> EngineResult {
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

    /// Encode the just-yielded row's snapshot index into `ref_out`.
    pub(in crate::engine) fn write_position(&self, ref_out: &mut [u8]) {
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
}

/// `true` when `key` falls within `[start, end]` per the bound flags.
pub(in crate::engine) fn key_in_bounds(
    key: &Key,
    start: &std::ops::Bound<Key>,
    end: &std::ops::Bound<Key>,
) -> bool {
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
