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

//! Indexed-access sub-trait every engine that exposes index callbacks
//! implements alongside [`StorageEngine`].
//!
//! **Line-limit note.** This file exceeds the 250-line ceiling because its
//! single responsibility is the `IndexedEngine` trait definition (one Rust
//! `pub trait` per the coding-style exemption). Splitting by method group
//! would force every callback to traverse a public sub-module re-export
//! purely to satisfy the limit, broadening the public surface for no gain.

use core::ffi::c_void;

use super::range_key::RangeKey;
use super::rkey_function::RKeyFunction;
use super::{EngineError, EngineResult, StorageEngine};
use crate::sys;

/// Indexed-access surface engines opt into via [`EngineCapabilities`].
///
/// Every method has a default that mirrors the MySQL handler base behaviour
/// (returning [`EngineError::WrongCommand`] for fallible scans,
/// [`None`] for the optional `Option`-returning getters). Engines override
/// only the callbacks they actually serve; the FFI boundary dispatches
/// through [`as_indexed`](crate::engine::EngineCapabilities::as_indexed) so
/// callers without an [`IndexedEngine`] impl never reach this trait.
///
/// [`EngineCapabilities`]: crate::engine::EngineCapabilities
#[allow(clippy::missing_errors_doc)]
pub trait IndexedEngine: StorageEngine {
    /// Per-index capability bitfield. `idx` is the index, `part` the key part;
    /// when `all_parts` is set MySQL wants the combined flags up to and
    /// including `part`. The default returns `0`, matching engines that
    /// advertise no index capabilities.
    fn index_flags(&self, _idx: u32, _part: u32, _all_parts: bool) -> u32 {
        0
    }

    /// Begin an index scan on index `idx`. `sorted` requests that subsequent
    /// reads return rows in index order. The base handler merely records the
    /// active index and returns success.
    ///
    /// # Errors
    /// The default returns `Ok(())`, matching the MySQL handler base.
    fn index_init(&mut self, _idx: u32, _sorted: bool) -> EngineResult {
        Ok(())
    }

    /// End the index scan started by [`index_init`](Self::index_init).
    ///
    /// # Errors
    /// The default returns `Ok(())`, matching the MySQL handler base.
    fn index_end(&mut self) -> EngineResult {
        Ok(())
    }

    /// Position the index cursor at `key` according to `find_flag` and read the
    /// matching row into `buf`. `key` is the leading key bytes whose length the
    /// shim resolved from the original `key_part_map`; it is empty when MySQL
    /// passed a null key (begin at the first key of the index). Neither borrow
    /// may be retained past the call.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`]; engines return
    /// [`EngineError::EndOfFile`] when no row matches.
    fn index_read_map(
        &mut self,
        _buf: &mut [u8],
        _key: &[u8],
        _find_flag: RKeyFunction,
    ) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Read the next row in the index scan into `buf`.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`]; engines return
    /// [`EngineError::EndOfFile`] once the scan is exhausted.
    fn index_next(&mut self, _buf: &mut [u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Read the previous row in the index scan into `buf`.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`]; engines return
    /// [`EngineError::EndOfFile`] once the scan is exhausted.
    fn index_prev(&mut self, _buf: &mut [u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Read the first row of the index into `buf`.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`]; engines return
    /// [`EngineError::EndOfFile`] when the index is empty.
    fn index_first(&mut self, _buf: &mut [u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Read the last row of the index into `buf`.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`]; engines return
    /// [`EngineError::EndOfFile`] when the index is empty.
    fn index_last(&mut self, _buf: &mut [u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Read the next row that shares the leading `key` bytes with the current
    /// position, into `buf`. The borrow on `key` may not be retained past the
    /// call.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`]; engines return
    /// [`EngineError::EndOfFile`] when no further row shares the key.
    fn index_next_same(&mut self, _buf: &mut [u8], _key: &[u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Position the index cursor at `key` according to `find_flag` and read the
    /// matching row into `buf`. This is the explicit-length sibling of
    /// [`index_read_map`](Self::index_read_map): MySQL supplied the key length
    /// directly rather than as a `key_part_map`, but the shim resolves both to
    /// the same leading key bytes. `key` is empty when MySQL passed a null key
    /// (begin at the first key). Neither borrow may be retained past the call.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`]; engines return
    /// [`EngineError::EndOfFile`] when no row matches.
    fn index_read(
        &mut self,
        _buf: &mut [u8],
        _key: &[u8],
        _find_flag: RKeyFunction,
    ) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Read from index `index` (rather than the active index) at `key` per
    /// `find_flag`, into `buf`. The base handler brackets this with an
    /// `index_init` / `index_end` pair; the binding instead passes `index`
    /// explicitly so the engine never has to track an implicit active index.
    /// `key` is empty for a null key. Neither borrow may be retained past the
    /// call.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`]; engines return
    /// [`EngineError::EndOfFile`] when no row matches.
    fn index_read_idx_map(
        &mut self,
        _buf: &mut [u8],
        _index: u32,
        _key: &[u8],
        _find_flag: RKeyFunction,
    ) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Read the last row matching `key` (or its prefix) on the active index
    /// into `buf`. The explicit-length counterpart of
    /// [`index_read_last_map`](Self::index_read_last_map). `key` is empty for a
    /// null key. Neither borrow may be retained past the call.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`]; engines return
    /// [`EngineError::EndOfFile`] when no row matches.
    fn index_read_last(&mut self, _buf: &mut [u8], _key: &[u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Read the last row matching `key` (or its prefix) on the active index
    /// into `buf`, with the key length resolved from the original
    /// `key_part_map`. `key` is empty for a null key. Neither borrow may be
    /// retained past the call.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`]; engines return
    /// [`EngineError::EndOfFile`] when no row matches.
    fn index_read_last_map(&mut self, _buf: &mut [u8], _key: &[u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Position the index cursor at `key` (resolved from a `key_part_map` like
    /// [`index_read_map`](Self::index_read_map)) and read the matching row into
    /// `buf` as the root of a pushed join. Pushed-join execution is
    /// engine-specific (NDB-style); the binding exposes the callback so a
    /// participating engine can implement it, but there is no `find_flag` —
    /// MySQL only ever issues an exact-key lookup here. `key` is empty for a
    /// null key. Neither borrow may be retained past the call.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`], matching the handler
    /// base.
    fn index_read_pushed(&mut self, _buf: &mut [u8], _key: &[u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Read the next row of the pushed-join result started by
    /// [`index_read_pushed`](Self::index_read_pushed) into `buf`.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`], matching the handler
    /// base.
    fn index_next_pushed(&mut self, _buf: &mut [u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Begin a range scan and read its first row into `buf`. `start` and `end`
    /// are the lower and upper bounds; either is `None` for an open end.
    /// `eq_range` marks an equality range (`start == end`), and `sorted`
    /// requests rows in index order. The handler base implements this by
    /// orchestrating the index read and navigation methods plus its own
    /// end-of-range comparison; the binding hands the whole operation to the
    /// engine, so an overriding engine owns range-boundary enforcement.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`]; engines return
    /// [`EngineError::EndOfFile`] when the range is empty.
    fn read_range_first(
        &mut self,
        _buf: &mut [u8],
        _start: Option<RangeKey<'_>>,
        _end: Option<RangeKey<'_>>,
        _eq_range: bool,
        _sorted: bool,
    ) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Read the next row of the range scan started by
    /// [`read_range_first`](Self::read_range_first) into `buf`.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`]; engines return
    /// [`EngineError::EndOfFile`] once the range is exhausted.
    fn read_range_next(&mut self, _buf: &mut [u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Estimate the number of rows on index `inx` between `min` and `max`
    /// (either `None` for an open end). Used by the optimizer to cost an index
    /// access path. Return `None` to signal "cannot estimate" (MySQL's
    /// `HA_POS_ERROR`); the default returns `Some(10)`, mirroring the handler
    /// base's fixed guess.
    fn records_in_range(
        &mut self,
        _inx: u32,
        _min: Option<RangeKey<'_>>,
        _max: Option<RangeKey<'_>>,
    ) -> Option<u64> {
        Some(10)
    }

    /// Estimate the cost of a multi-range read over a known set of ranges on
    /// index `keyno`, for the optimizer's const-range path. `seq` is MySQL's
    /// `RANGE_SEQ_IF` range-sequence interface, `seq_init_param` its init
    /// argument (round-tripped without dereference), and `cost` the
    /// `Cost_estimate` accumulator. These are opaque MySQL objects the binding
    /// cannot drive from Rust yet, so a custom estimate is not expressible until
    /// that wiring lands; the callback exists so the surface is complete.
    ///
    /// Return `None` (the default) to use the base disk-sweep MRR
    /// implementation, which is built on
    /// [`read_range_first`](Self::read_range_first) /
    /// [`read_range_next`](Self::read_range_next). Engines providing a custom
    /// multi-range read return `Some(rows)`.
    fn multi_range_read_info_const(
        &mut self,
        _keyno: u32,
        _seq: Option<&sys::RangeSeqIf>,
        _seq_init_param: *mut c_void,
        _n_ranges: u32,
        _cost: Option<&sys::CostEstimate>,
    ) -> Option<u64> {
        None
    }

    /// Estimate the cost of a multi-range read over `n_ranges` ranges spanning
    /// `keys` rows on index `keyno`. `cost` is the `Cost_estimate` accumulator,
    /// an opaque MySQL object the binding cannot drive from Rust yet.
    ///
    /// Return `None` (the default) to use the base disk-sweep MRR
    /// implementation; engines providing a custom multi-range read return
    /// `Some(rows)`.
    fn multi_range_read_info(
        &mut self,
        _keyno: u32,
        _n_ranges: u32,
        _keys: u32,
        _cost: Option<&sys::CostEstimate>,
    ) -> Option<u64> {
        None
    }

    /// Initialize a multi-range read scan over the ranges from `seq` (init
    /// argument `seq_init_param`), with `mode` carrying the `HA_MRR_*` flags and
    /// `buf` a caller-owned `HANDLER_BUFFER` scratch area. `seq` and `buf` are
    /// opaque MySQL objects the binding cannot drive from Rust yet.
    ///
    /// Return `None` (the default) to use the base disk-sweep MRR
    /// implementation, which drives
    /// [`read_range_first`](Self::read_range_first) /
    /// [`read_range_next`](Self::read_range_next). Engines providing a custom
    /// multi-range read return `Some(result)`.
    fn multi_range_read_init(
        &mut self,
        _seq: Option<&sys::RangeSeqIf>,
        _seq_init_param: *mut c_void,
        _n_ranges: u32,
        _mode: u32,
        _buf: Option<&sys::HandlerBuffer>,
    ) -> Option<EngineResult> {
        None
    }

    /// Read the next row of the multi-range read scan into `buf`, writing the
    /// range association through `range_info` (an opaque `char**` out-pointer
    /// the binding round-trips without dereference).
    ///
    /// Return `None` (the default) to use the base disk-sweep MRR
    /// implementation; engines providing a custom multi-range read return
    /// `Some(result)`, where [`EngineError::EndOfFile`] marks the end of the
    /// scan.
    fn multi_range_read_next(
        &mut self,
        _buf: &mut [u8],
        _range_info: *mut *mut c_void,
    ) -> Option<EngineResult> {
        None
    }

    /// Whether indexes are currently disabled (e.g. after `ALTER TABLE ...
    /// DISABLE KEYS`), as the raw handler int (`0` = enabled). Return `None`
    /// (the default) to use the handler base (`0`); engines return `Some(code)`.
    fn indexes_are_disabled(&mut self) -> Option<i32> {
        None
    }

    /// Estimated cost of an index-only read of `records` rows through index
    /// `keynr`, in MySQL's legacy cost unit. Return `None` (the default) to use
    /// the handler base; engines return `Some(time)`.
    fn index_only_read_time(&mut self, _keynr: u32, _records: f64) -> Option<f64> {
        None
    }

    /// Cost estimate for reading `ranges` ranges spanning `rows` rows from index
    /// `index` without fetching the full row. Return `None` (the default) to use
    /// the handler base, derived from
    /// [`index_only_read_time`](Self::index_only_read_time); engines return
    /// `Some(cost)`.
    fn index_scan_cost(
        &mut self,
        _index: u32,
        _ranges: f64,
        _rows: f64,
    ) -> Option<super::CostEstimate> {
        None
    }
}
