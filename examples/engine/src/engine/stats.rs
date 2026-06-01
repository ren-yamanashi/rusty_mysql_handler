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

//! Optimizer-visible statistics for [`TrivialEngine`] — `index_flags`,
//! `records_in_range`, and the related row / scan accessors.

use mysql_handler::engine::RangeKey;
use mysql_handler::sys::{HA_READ_NEXT, HA_READ_ORDER, HA_READ_PREV, HA_READ_RANGE};

use super::TrivialEngine;
use super::scan::{Endpoint, key_in_bounds};
use crate::store::{self, IndexMeta};

impl TrivialEngine {
    /// Per-index capability bitfield. Range scans work for any index —
    /// the cursor honours [`Key`](crate::store::Key)-ordered bounds
    /// either way. Ordered iteration only matches MySQL's expectation
    /// when the index is single-column ASC, so gate `HA_READ_ORDER`
    /// (and `HA_READ_PREV`, which only makes sense once order is
    /// established) on the snapshot.
    pub(super) fn index_flags_for(&self, idx: u32) -> u32 {
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

    /// Estimate the number of rows in `[min, max]` for index `inx`.
    /// The primary path counts off the BTreeMap; secondary indexes
    /// walk the primary store on demand because the optimizer calls
    /// `records_in_range` before `index_init` populates the snapshot.
    pub(super) fn records_in_range_for(
        &self,
        inx: u32,
        min: Option<RangeKey<'_>>,
        max: Option<RangeKey<'_>>,
    ) -> Option<u64> {
        let meta = self.meta.as_ref()?;
        let index = meta.indexes().get(inx as usize)?;
        let start = Self::decode_bound(meta, index, min, Endpoint::Start);
        let end = Self::decode_bound(meta, index, max, Endpoint::End);
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

    /// Reserve `nb_desired` auto-increment values starting at the
    /// engine's current high-water mark.
    pub(super) fn reserve_auto_increment(&mut self, increment: u64, nb_desired: u64) -> (u64, u64) {
        let first = self.next_auto_inc;
        let reserved = nb_desired.max(1);
        self.next_auto_inc = first.saturating_add(reserved.saturating_mul(increment.max(1)));
        (first, reserved)
    }
}
