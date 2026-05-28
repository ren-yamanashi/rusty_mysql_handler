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

//! Safe storage-engine interface for downstream implementations

use core::ffi::c_void;
use std::ffi::CStr;

use crate::sys;

mod bulk_access;
mod cost_estimate;
mod error;
mod parallel_scan_init;
mod range_key;
mod reset_cached_state;
mod rkey_function;
mod sampling_method;

pub use bulk_access::BulkAccess;
pub use cost_estimate::CostEstimate;
pub use error::{EngineError, EngineResult};
pub use parallel_scan_init::ParallelScanInit;
pub use range_key::RangeKey;
pub use reset_cached_state::ResetCachedState;
pub use rkey_function::RKeyFunction;
pub use sampling_method::SamplingMethod;

/// The safe interface every storage engine implements.
///
/// MySQL constructs one instance per opened table per session worker thread,
/// so the trait requires `Send`. The `EngineContext` that owns a
/// `Box<dyn StorageEngine>` crosses the C++ FFI boundary as a raw pointer;
/// the `Send` bound is the only compile-time guarantee that this stays sound.
#[allow(clippy::missing_errors_doc)]
pub trait StorageEngine: Send {
    /// Engine display name shown by `SHOW ENGINES` and used as the `ENGINE=`
    /// value in `CREATE TABLE`. Must be a null-terminated `'static` C string
    /// (e.g. `c"RUSTY"`) because the pointer is handed straight to MySQL.
    fn table_type(&self) -> &'static CStr;

    /// `HA_*` capability bitfield advertised to the optimizer
    fn table_flags(&self) -> u64;

    /// Per-index capability bitfield. `idx` is the index, `part` the key part;
    /// when `all_parts` is set MySQL wants the combined flags up to and
    /// including `part`.
    fn index_flags(&self, idx: u32, part: u32, all_parts: bool) -> u32;

    /// Create the on-disk representation for a new table named `name`.
    /// Errors are implementation-defined.
    fn create(&mut self, name: &str) -> EngineResult;

    /// Open an existing table named `name` in the given `mode`.
    /// Errors are implementation-defined.
    fn open(&mut self, name: &str, mode: i32) -> EngineResult;

    /// Release any resources acquired by [`open`](Self::open).
    /// Errors are implementation-defined.
    fn close(&mut self) -> EngineResult;

    /// Begin a full table scan. `scan == false` indicates the optimizer will
    /// only use positioned access (`rnd_pos`). Errors are implementation-defined.
    fn rnd_init(&mut self, scan: bool) -> EngineResult;

    /// End the full table scan started by [`rnd_init`](Self::rnd_init),
    /// releasing any cursor state. MySQL may call [`rnd_init`](Self::rnd_init)
    /// again without an intervening `rnd_end`, so implementations must tolerate
    /// a re-init.
    ///
    /// # Errors
    /// The default returns `Ok(())`, matching the handler base.
    fn rnd_end(&mut self) -> EngineResult {
        Ok(())
    }

    /// Fetch the next row into `buf`.
    ///
    /// # Errors
    /// Returns [`EngineError::EndOfFile`] once the scan is exhausted; other
    /// variants are implementation-defined.
    fn rnd_next(&mut self, buf: &mut [u8]) -> EngineResult;

    /// Fetch a row by the position previously recorded with
    /// [`position`](Self::position).
    ///
    /// # Errors
    /// Returns [`EngineError::WrongCommand`] when the engine has no positioned
    /// access path; other variants are implementation-defined.
    fn rnd_pos(&mut self, buf: &mut [u8], pos: &[u8]) -> EngineResult;

    /// Notify the engine of the row just read so a later
    /// [`rnd_pos`](Self::rnd_pos) can replay it. The shim does not yet
    /// expose MySQL's `handler::ref` buffer to Rust, so engines have no
    /// place to persist a position — implementations either remember the
    /// row internally or return [`EngineError::WrongCommand`] from
    /// `rnd_pos` until the wiring is added.
    fn position(&mut self, record: &[u8]);

    /// Read the row whose primary key matches the one encoded in `record` (in
    /// MySQL's internal record format), overwriting `record` with the full row.
    /// Only meaningful for engines that advertise
    /// `HA_PRIMARY_KEY_REQUIRED_FOR_POSITION`.
    ///
    /// The handler base implements this by orchestrating
    /// [`rnd_init`](Self::rnd_init) / [`position`](Self::position) /
    /// [`rnd_pos`](Self::rnd_pos) / [`rnd_end`](Self::rnd_end) through its
    /// internal `ref` buffer; the binding does not expose that buffer, so it
    /// hands the whole operation to the engine instead. The borrow may not be
    /// retained past the call.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`].
    fn rnd_pos_by_record(&mut self, _record: &mut [u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Refresh statistics (rows, deleted rows, data length, ...) for the
    /// optimizer. Errors are implementation-defined.
    fn info(&mut self, flag: u32) -> EngineResult;

    /// Drop a table. `table_def` is the data-dictionary descriptor of the
    /// table being deleted; it may be `None` for temporary tables created
    /// by the optimizer.
    ///
    /// May be invoked twice per `DROP TABLE` of a temporary table: once
    /// directly, and once as part of the `handler::drop_table` chain (close +
    /// delete_table with `table_def = None`) that fires before
    /// [`drop_table`](Self::drop_table). Implementations that count calls
    /// must tolerate the repeat.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`]. This deliberately
    /// diverges from MySQL's `handler::delete_table` base, which deletes the
    /// on-disk artefact via `my_delete`; the binding leaves any artefact
    /// cleanup to the engine implementation.
    fn delete_table(&mut self, _name: &str, _table_def: Option<&sys::DdTable>) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Rename a table from `from` to `to`. `from_table_def` and `to_table_def`
    /// are the data-dictionary descriptors before and after the rename.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`].
    fn rename_table(
        &mut self,
        _from: &str,
        _to: &str,
        _from_table_def: Option<&sys::DdTable>,
        _to_table_def: Option<&sys::DdTable>,
    ) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Notification that MySQL is dropping the table, invoked from
    /// `ha_drop_table` on temporary-table cleanup paths. The binding mirrors
    /// upstream's `handler::drop_table` chain (`close()` then
    /// [`delete_table`](Self::delete_table) with `table_def = None`) on the
    /// C++ side, so this callback fires after the chain completes and serves
    /// purely as a post-cleanup hook. Default is a no-op.
    ///
    /// Any error returned by the in-chain [`delete_table`](Self::delete_table)
    /// is swallowed by MySQL's void `handler::drop_table`; engines that need
    /// to surface a failure during cleanup must do so out-of-band.
    fn drop_table(&mut self, _name: &str) {}

    /// Reset the table to an empty state without dropping it.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`], matching the
    /// MySQL handler base implementation.
    fn truncate(&mut self, _table_def: Option<&sys::DdTable>) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Notification that MySQL has reassigned the underlying `TABLE` and
    /// `TABLE_SHARE`. The base C++ handler updates its own pointers; this
    /// callback lets the engine react if it caches per-table state. Default
    /// is a no-op.
    fn change_table_ptr(&mut self, _table: Option<&sys::TABLE>, _share: Option<&sys::TABLE_SHARE>) {
    }

    /// Populate engine-private metadata in `dd_table`. `reset` distinguishes
    /// the case where the data-dictionary entry has been reset and any cached
    /// state must be re-emitted. Returns `true` when private data was written.
    /// The default returns `false`.
    fn se_private_data(
        &mut self,
        _dd_table: Option<&sys::DdTable>,
        _reset: ResetCachedState,
    ) -> bool {
        false
    }

    /// Inject implicit columns and indexes the engine requires for `table_obj`
    /// to be created.
    ///
    /// # Errors
    /// The default never errors; overrides choose which [`EngineError`]
    /// variants they emit.
    fn extra_columns_and_keys(
        &mut self,
        _create_info: Option<&sys::HA_CREATE_INFO>,
        _create_list: Option<&sys::ListCreateField>,
        _key_info: Option<&sys::KEY>,
        _key_count: u32,
        _table_obj: Option<&sys::DdTable>,
    ) -> EngineResult {
        Ok(())
    }

    /// Adjust the data-dictionary entry of an old-format table during a server
    /// upgrade. Returning `Err` aborts the upgrade (mapped to C++ `bool true`).
    ///
    /// # Errors
    /// The default returns `Ok(())`; overrides surface an [`EngineError`] to
    /// abort the upgrade.
    fn upgrade_table(
        &mut self,
        _thd: Option<&sys::THD>,
        _dbname: &str,
        _table_name: &str,
        _dd_table: Option<&sys::DdTable>,
    ) -> EngineResult {
        Ok(())
    }

    /// Insert the row held in `buf`, encoded in MySQL's internal record format
    /// (the contents of `record[0]`). The engine must copy out whatever it
    /// needs during the call; the borrow may not be retained afterwards.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`], matching the MySQL
    /// handler base which rejects writes on engines that do not support them.
    fn write_row(&mut self, _buf: &[u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Replace the row whose existing image is `old` with the new image `new`,
    /// both in MySQL's internal record format. Neither borrow may be retained
    /// past the call.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`].
    fn update_row(&mut self, _old: &[u8], _new: &[u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Delete the row whose current image is `buf`, in MySQL's internal record
    /// format. The borrow may not be retained past the call.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`].
    fn delete_row(&mut self, _buf: &[u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Delete every row in the table in a single operation, the fast path MySQL
    /// takes for an unqualified `DELETE` when the engine advertises support.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`].
    fn delete_all_rows(&mut self) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Hint that a multi-row INSERT is about to begin; `rows` is MySQL's
    /// estimate of how many rows will be written (`0` when unknown). Engines may
    /// pre-size buffers here. The default is a no-op, matching the handler base.
    fn start_bulk_insert(&mut self, _rows: u64) {}

    /// Flush any rows buffered since
    /// [`start_bulk_insert`](Self::start_bulk_insert).
    ///
    /// # Errors
    /// The default returns `Ok(())`, matching the handler base which always
    /// succeeds.
    fn end_bulk_insert(&mut self) -> EngineResult {
        Ok(())
    }

    /// Decide whether to batch the rows of a multi-row UPDATE.
    /// [`BulkAccess::Batched`] routes subsequent rows through
    /// [`bulk_update_row`](Self::bulk_update_row) and
    /// [`exec_bulk_update`](Self::exec_bulk_update); [`BulkAccess::PerRow`] keeps
    /// MySQL on the per-row [`update_row`](Self::update_row) path. The default is
    /// [`BulkAccess::PerRow`], matching the handler base.
    fn start_bulk_update(&mut self) -> BulkAccess {
        BulkAccess::PerRow
    }

    /// Apply all updates buffered since
    /// [`start_bulk_update`](Self::start_bulk_update), returning the number of
    /// duplicate-key collisions encountered. MySQL may continue batching after
    /// this call until [`end_bulk_update`](Self::end_bulk_update).
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`], matching the handler
    /// base which rejects the bulk path unless the engine opts in.
    fn exec_bulk_update(&mut self) -> EngineResult<u32> {
        Err(EngineError::WrongCommand)
    }

    /// Release any state held for the bulk-update batch, called once the
    /// statement's updates are concluded. The default is a no-op.
    fn end_bulk_update(&mut self) {}

    /// Buffer one row update for a later
    /// [`exec_bulk_update`](Self::exec_bulk_update), replacing the image `old`
    /// with `new` (both in MySQL's internal record format). Returns the running
    /// count of duplicate-key collisions. Neither borrow may be retained past
    /// the call.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`], matching the handler
    /// base.
    fn bulk_update_row(&mut self, _old: &[u8], _new: &[u8]) -> EngineResult<u32> {
        Err(EngineError::WrongCommand)
    }

    /// Decide whether to batch the rows of a multi-row DELETE.
    /// [`BulkAccess::Batched`] routes the deletes through the bulk path closed
    /// by [`end_bulk_delete`](Self::end_bulk_delete); [`BulkAccess::PerRow`]
    /// keeps MySQL on [`delete_row`](Self::delete_row). The default is
    /// [`BulkAccess::PerRow`], matching the handler base.
    fn start_bulk_delete(&mut self) -> BulkAccess {
        BulkAccess::PerRow
    }

    /// Execute all buffered deletes and close the bulk-delete batch.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`], matching the handler
    /// base.
    fn end_bulk_delete(&mut self) -> EngineResult {
        Err(EngineError::WrongCommand)
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

    /// Report whether the table is ready for a bulk load on session `thd`.
    /// The default returns `false`, matching the handler base; engines that
    /// support `ALTER TABLE ... SECONDARY_LOAD`-style bulk loads return `true`.
    fn bulk_load_check(&self, _thd: Option<&sys::THD>) -> bool {
        false
    }

    /// Report the memory budget (in bytes) the engine can devote to a bulk
    /// load on session `thd`. The default returns `0`, matching the handler
    /// base.
    fn bulk_load_available_memory(&self, _thd: Option<&sys::THD>) -> usize {
        0
    }

    /// Begin a parallel bulk load, returning an engine-owned context pointer
    /// that [`bulk_load_execute`](Self::bulk_load_execute) and
    /// [`bulk_load_end`](Self::bulk_load_end) receive back unchanged. `data_size`
    /// is the total bytes to load, `memory` the budget granted, `num_threads`
    /// the concurrency. The binding round-trips the pointer through MySQL
    /// verbatim and never dereferences it; the engine owns its lifetime and
    /// must free it in [`bulk_load_end`](Self::bulk_load_end). The default
    /// returns a null pointer, matching the handler base (load not started).
    fn bulk_load_begin(
        &mut self,
        _thd: Option<&sys::THD>,
        _data_size: usize,
        _memory: usize,
        _num_threads: usize,
    ) -> *mut c_void {
        core::ptr::null_mut()
    }

    /// Load `rows` into the table on thread `thread_idx`, using the context
    /// from [`bulk_load_begin`](Self::bulk_load_begin). `rows` and
    /// `stat_callbacks` are opaque MySQL handles the binding cannot yet read
    /// into, so a functioning bulk load is not expressible until that wiring
    /// lands; the callback exists so the surface is complete. `load_ctx` is the
    /// engine's own pointer and must be dereferenced only by the engine.
    ///
    /// # Errors
    /// The default returns [`EngineError::Unsupported`], matching the handler
    /// base which reports `HA_ERR_UNSUPPORTED` until the engine opts in.
    fn bulk_load_execute(
        &mut self,
        _thd: Option<&sys::THD>,
        _load_ctx: *mut c_void,
        _thread_idx: usize,
        _rows: Option<&sys::RowsMysql>,
        _stat_callbacks: Option<&sys::BulkLoadStatCallbacks>,
    ) -> EngineResult {
        Err(EngineError::Unsupported)
    }

    /// End the bulk load and release the context from
    /// [`bulk_load_begin`](Self::bulk_load_begin). Always called once after all
    /// execute threads finish, even when `is_error` is `true`, so the engine
    /// can free `load_ctx` on both paths.
    ///
    /// # Errors
    /// The default returns `Ok(())`, matching the handler base.
    fn bulk_load_end(
        &mut self,
        _thd: Option<&sys::THD>,
        _load_ctx: *mut c_void,
        _is_error: bool,
    ) -> EngineResult {
        Ok(())
    }

    /// Load `table` (opened in the primary engine) into this secondary engine;
    /// its read-set selects which columns to load. Returns whether MySQL should
    /// skip updating the data-dictionary metadata for this load.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`]. This diverges from the
    /// handler base, which asserts (secondary-engine-only); the binding returns
    /// the error instead of aborting in debug builds.
    fn load_table(&mut self, _table: Option<&sys::TABLE>) -> EngineResult<bool> {
        Err(EngineError::WrongCommand)
    }

    /// Unload the table named `db_name`.`table_name` from this secondary engine.
    /// When `error_if_not_loaded` is `false`, a missing table must fail
    /// silently so a `DROP TABLE` cleanup path is not blocked.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`]. This diverges from the
    /// handler base, which asserts (secondary-engine-only); the binding returns
    /// the error instead of aborting in debug builds.
    fn unload_table(
        &mut self,
        _db_name: &str,
        _table_name: &str,
        _error_if_not_loaded: bool,
    ) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Initialize a parallel scan, returning the engine-owned scan context and
    /// the number of worker threads the engine will drive (see
    /// [`ParallelScanInit`]). `use_reserved_threads` permits dipping into the
    /// reserved pool when the parallel-read cap is hit; `max_desired_threads`
    /// caps the thread count (`0` means no cap). The default returns a null
    /// context and zero threads, matching the handler base (no parallel scan).
    ///
    /// # Errors
    /// The default never errors; overrides choose their own variants.
    fn parallel_scan_init(
        &mut self,
        _use_reserved_threads: bool,
        _max_desired_threads: usize,
    ) -> EngineResult<ParallelScanInit> {
        Ok(ParallelScanInit::new(core::ptr::null_mut(), 0))
    }

    /// Run the parallel read using the context from
    /// [`parallel_scan_init`](Self::parallel_scan_init). `thread_ctxs` is the
    /// caller's per-thread context array; `init_fn` / `load_fn` / `end_fn` are
    /// MySQL `std::function` callbacks passed as opaque pointers. The binding
    /// cannot invoke those callbacks from Rust yet, so a functioning parallel
    /// read is not expressible until that wiring lands; the callback exists so
    /// the surface is complete. None of these pointers may be dereferenced
    /// except by the code that owns them.
    ///
    /// # Errors
    /// The default returns `Ok(())`, matching the handler base.
    fn parallel_scan(
        &mut self,
        _scan_ctx: *mut c_void,
        _thread_ctxs: *mut *mut c_void,
        _init_fn: *const c_void,
        _load_fn: *const c_void,
        _end_fn: *const c_void,
    ) -> EngineResult {
        Ok(())
    }

    /// Release the parallel-scan context from
    /// [`parallel_scan_init`](Self::parallel_scan_init). The default is a no-op.
    fn parallel_scan_end(&mut self, _scan_ctx: *mut c_void) {}

    /// Initialize sampling, returning the engine-owned scan context used by
    /// [`sample_next`](Self::sample_next). `sampling_percentage` is the share of
    /// rows to return (0–100), `sampling_seed` seeds the engine RNG,
    /// `sampling_method` selects the algorithm, and `tablesample` marks an SQL
    /// `TABLESAMPLE` rather than an internal sample. The context pointer is
    /// round-tripped verbatim and never dereferenced by the binding.
    ///
    /// # Errors
    /// The default delegates to [`rnd_init`](Self::rnd_init) with `scan = true`
    /// and returns a null context, mirroring the handler base which samples by
    /// scanning. The percentage filter the base applies relies on handler-
    /// internal RNG state the binding does not expose, so the default yields
    /// every row (an effective 100% sample) until an engine overrides this.
    fn sample_init(
        &mut self,
        _sampling_percentage: f64,
        _sampling_seed: i32,
        _sampling_method: SamplingMethod,
        _tablesample: bool,
    ) -> EngineResult<*mut c_void> {
        match self.rnd_init(true) {
            Ok(()) => Ok(core::ptr::null_mut()),
            Err(e) => Err(e),
        }
    }

    /// Read the next sampled row into `buf`, using the context from
    /// [`sample_init`](Self::sample_init).
    ///
    /// # Errors
    /// The default delegates to [`rnd_next`](Self::rnd_next) (no percentage
    /// filtering); engines return [`EngineError::EndOfFile`] once the sample is
    /// exhausted.
    fn sample_next(&mut self, _scan_ctx: *mut c_void, buf: &mut [u8]) -> EngineResult {
        self.rnd_next(buf)
    }

    /// End sampling and release the context from
    /// [`sample_init`](Self::sample_init).
    ///
    /// # Errors
    /// The default delegates to [`rnd_end`](Self::rnd_end), matching the handler
    /// base.
    fn sample_end(&mut self, _scan_ctx: *mut c_void) -> EngineResult {
        self.rnd_end()
    }

    /// Begin a full-text search scan.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`], matching the handler
    /// base.
    fn ft_init(&mut self) -> EngineResult {
        Err(EngineError::WrongCommand)
    }

    /// Create a full-text search handle for index `inx` and query `key`, with
    /// `flags` selecting the search mode. Returns an engine-owned
    /// `FT_INFO`-compatible pointer that MySQL drives through its vtable, or
    /// null when the engine cannot serve the search. `key` is MySQL's `String`
    /// query object, opaque to the binding. The pointer is round-tripped
    /// verbatim and never dereferenced by the binding; the engine owns its
    /// lifetime. The default returns null, matching the handler base (which
    /// raises `ER_TABLE_CANT_HANDLE_FT`).
    fn ft_init_ext(
        &mut self,
        _flags: u32,
        _inx: u32,
        _key: Option<&sys::MysqlString>,
    ) -> *mut c_void {
        core::ptr::null_mut()
    }

    /// Hint-aware variant of [`ft_init_ext`](Self::ft_init_ext). `flags` is
    /// pre-extracted from `hints` by the shim (the binding cannot read the
    /// opaque `hints` object from Rust); `hints` is still passed for engines
    /// that grow richer hint handling. The default delegates to
    /// [`ft_init_ext`](Self::ft_init_ext), mirroring the handler base.
    fn ft_init_ext_with_hints(
        &mut self,
        flags: u32,
        inx: u32,
        key: Option<&sys::MysqlString>,
        _hints: Option<&sys::FtHints>,
    ) -> *mut c_void {
        self.ft_init_ext(flags, inx, key)
    }

    /// Read the next row matching the active full-text search into `buf`.
    ///
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`]; engines return
    /// [`EngineError::EndOfFile`] once the matches are exhausted.
    fn ft_read(&mut self, _buf: &mut [u8]) -> EngineResult {
        Err(EngineError::WrongCommand)
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

    /// Maximum row length the engine supports, in bytes. Return `None` (the
    /// default) to use the handler base (`HA_MAX_REC_LENGTH`); engines with a
    /// tighter cap return `Some(len)`.
    fn max_supported_record_length(&self) -> Option<u32> {
        None
    }

    /// Maximum number of indexes the engine supports on one table. Return `None`
    /// (the default) to use the handler base (`0`, i.e. no indexes); engines
    /// that support indexes return `Some(count)` — this is the gate MySQL checks
    /// before allowing `CREATE TABLE ... KEY(...)`.
    fn max_supported_keys(&self) -> Option<u32> {
        None
    }

    /// Maximum number of key parts in one index. Return `None` (the default) to
    /// use the handler base (`MAX_REF_PARTS`); engines with a tighter cap return
    /// `Some(parts)`.
    fn max_supported_key_parts(&self) -> Option<u32> {
        None
    }

    /// Maximum total key length in bytes. Return `None` (the default) to use the
    /// handler base (`MAX_KEY_LENGTH`); engines with a tighter cap return
    /// `Some(len)`.
    fn max_supported_key_length(&self) -> Option<u32> {
        None
    }

    /// Maximum length in bytes of a single key part for the table described by
    /// `create_info` (an opaque MySQL `HA_CREATE_INFO`). Return `None` (the
    /// default) to use the handler base (`255`); engines with a different cap
    /// return `Some(len)`.
    fn max_supported_key_part_length(
        &self,
        _create_info: Option<&sys::HA_CREATE_INFO>,
    ) -> Option<u32> {
        None
    }

    /// Minimum row length in bytes for a table created with `options` (the
    /// `HA_CREATE_INFO` table-option bitfield). Return `None` (the default) to
    /// use the handler base (`1`); engines with a larger floor return
    /// `Some(len)`.
    fn min_record_length(&self, _options: u32) -> Option<u32> {
        None
    }

    /// Extra per-record buffer space the engine needs beyond the row image, in
    /// bytes. Return `None` (the default) to use the handler base (`0`); engines
    /// needing scratch space return `Some(len)`.
    fn extra_rec_buf_length(&self) -> Option<u32> {
        None
    }

    /// In-memory buffer size the engine reports to the optimizer, in bytes, or a
    /// negative value when not applicable. Return `None` (the default) to use the
    /// handler base (`-1`); engines return `Some(bytes)`.
    fn memory_buffer_size(&self) -> Option<i64> {
        None
    }

    /// Whether the engine stores multi-byte values low byte first
    /// (little-endian). Return `None` (the default) to use the handler base
    /// (`true`); engines return `Some(flag)`.
    fn low_byte_first(&self) -> Option<bool> {
        None
    }

    /// Live checksum of the table, or `None` (the default) to use the handler
    /// base (`0`, no checksum). Engines that maintain one return `Some(sum)`.
    fn checksum(&self) -> Option<u32> {
        None
    }

    /// Whether the table is marked crashed and needs repair. Return `None` (the
    /// default) to use the handler base (`false`); engines return `Some(flag)`.
    fn is_crashed(&self) -> Option<bool> {
        None
    }

    /// Whether MySQL should attempt automatic repair when the table is found
    /// crashed on open. Return `None` (the default) to use the handler base
    /// (`false`); engines return `Some(flag)`.
    fn auto_repair(&self) -> Option<bool> {
        None
    }

    /// Whether the primary key is clustered (rows stored in PK order). Return
    /// `None` (the default) to use the handler base (`false`); engines return
    /// `Some(flag)`.
    fn primary_key_is_clustered(&self) -> Option<bool> {
        None
    }

    /// Resolve the real `row_type` for a table created from `create_info` (an
    /// opaque MySQL `HA_CREATE_INFO`), as the raw `enum row_type` integer.
    /// Return `None` (the default) to use the handler base, which derives the
    /// type from the create options; engines return `Some(row_type)`.
    fn real_row_type(&self, _create_info: Option<&sys::HA_CREATE_INFO>) -> Option<i32> {
        None
    }

    /// Default index algorithm as the raw `enum ha_key_alg` integer, used when
    /// the user did not specify one. Return `None` (the default) to use the
    /// handler base (`HA_KEY_ALG_SE_SPECIFIC`); engines return `Some(alg)`.
    fn default_index_algorithm(&self) -> Option<i32> {
        None
    }

    /// Whether the engine supports index algorithm `key_alg` (a raw
    /// `enum ha_key_alg` integer). Return `None` (the default) to use the
    /// handler base (supports only its default algorithm); engines return
    /// `Some(flag)`.
    fn is_index_algorithm_supported(&self, _key_alg: i32) -> Option<bool> {
        None
    }

    /// Whether the engine wants MySQL to allocate a record buffer for
    /// prefetching, and for how many rows. Return `Some(max_rows)` to request a
    /// buffer sized for `max_rows`; `None` (the default) uses the handler base
    /// (no buffer wanted).
    fn record_buffer_wanted(&self) -> Option<u64> {
        None
    }

    /// Engine-specific text appended to the `Extra` column of `EXPLAIN`. Return
    /// `None` (the default) to use the handler base (empty string); engines
    /// return `Some(text)`.
    fn explain_extra(&self) -> Option<String> {
        None
    }

    /// Whether indexes are currently disabled (e.g. after `ALTER TABLE ...
    /// DISABLE KEYS`), as the raw handler int (`0` = enabled). Return `None`
    /// (the default) to use the handler base (`0`); engines return `Some(code)`.
    fn indexes_are_disabled(&mut self) -> Option<i32> {
        None
    }

    /// Estimated cost of a full table scan, in MySQL's legacy cost unit. Return
    /// `None` (the default) to use the handler base, which derives it from
    /// `stats.data_file_length`; engines return `Some(time)`.
    ///
    /// MySQL recommends overriding this rather than
    /// [`table_scan_cost`](Self::table_scan_cost), whose base implementation is
    /// built from this value.
    fn scan_time(&mut self) -> Option<f64> {
        None
    }

    /// Estimated cost of reading `ranges` ranges totalling `rows` rows through
    /// index `index`, in MySQL's legacy cost unit. Return `None` (the default)
    /// to use the handler base; engines return `Some(time)`.
    fn read_time(&mut self, _index: u32, _ranges: u32, _rows: u64) -> Option<f64> {
        None
    }

    /// Estimated cost of an index-only read of `records` rows through index
    /// `keynr`, in MySQL's legacy cost unit. Return `None` (the default) to use
    /// the handler base; engines return `Some(time)`.
    fn index_only_read_time(&mut self, _keynr: u32, _records: f64) -> Option<f64> {
        None
    }

    /// Cost estimate for a full table scan. Return `None` (the default) to use
    /// the handler base, which derives it from [`scan_time`](Self::scan_time);
    /// engines return `Some(cost)`.
    fn table_scan_cost(&mut self) -> Option<CostEstimate> {
        None
    }

    /// Cost estimate for reading `ranges` ranges spanning `rows` rows from index
    /// `index` without fetching the full row. Return `None` (the default) to use
    /// the handler base, derived from
    /// [`index_only_read_time`](Self::index_only_read_time); engines return
    /// `Some(cost)`.
    fn index_scan_cost(&mut self, _index: u32, _ranges: f64, _rows: f64) -> Option<CostEstimate> {
        None
    }

    /// Cost estimate for reading `ranges` ranges spanning `rows` rows from index
    /// `index`, including fetching the full rows. Return `None` (the default) to
    /// use the handler base, derived from [`read_time`](Self::read_time); engines
    /// return `Some(cost)`.
    fn read_cost(&mut self, _index: u32, _ranges: f64, _rows: f64) -> Option<CostEstimate> {
        None
    }

    /// Estimated cost of `reads` non-sequential accesses against index `index`,
    /// in the same unit as [`worst_seek_times`](Self::worst_seek_times). Return
    /// `None` (the default) to use the handler base (`Cost_model::page_read_cost`);
    /// engines return `Some(cost)`.
    fn page_read_cost(&mut self, _index: u32, _reads: f64) -> Option<f64> {
        None
    }

    /// Upper-bound cost of `reads` seek-and-read key lookups, in the same unit as
    /// [`page_read_cost`](Self::page_read_cost). Return `None` (the default) to
    /// use the handler base; engines return `Some(cost)`.
    fn worst_seek_times(&mut self, _reads: f64) -> Option<f64> {
        None
    }

    /// Exact number of rows in the table. Return `None` (the default) to use the
    /// handler base, which counts rows with a full table scan; engines that can
    /// answer directly return `Some(Ok(rows))`, or `Some(Err(_))` to surface a
    /// failure.
    ///
    /// # Errors
    /// The error variant is implementation-defined and maps to the matching
    /// `HA_ERR_*` code at the FFI boundary.
    fn records(&mut self) -> Option<EngineResult<u64>> {
        None
    }

    /// Exact number of rows counted through index `index`. Return `None` (the
    /// default) to use the handler base, which counts rows with an index scan;
    /// engines return `Some(Ok(rows))` or `Some(Err(_))`.
    ///
    /// # Errors
    /// The error variant is implementation-defined and maps to the matching
    /// `HA_ERR_*` code at the FFI boundary.
    fn records_from_index(&mut self, _index: u32) -> Option<EngineResult<u64>> {
        None
    }

    /// Upper bound on the number of rows a full table scan may return. Return
    /// `None` (the default) to use the handler base (`stats.records` plus a
    /// margin); engines return `Some(rows)`.
    fn estimate_rows_upper_bound(&mut self) -> Option<u64> {
        None
    }

    /// Hash value of the key columns in `field_array` for hash partitioning.
    /// `field_array` is a null-terminated `Field**` the binding round-trips as an
    /// opaque pointer valid for the call only (it cannot yet drive `Field` from
    /// Rust). Return `None` (the default) to use the handler base, which asserts
    /// — so only engines advertising hash partitioning should override and return
    /// `Some(hash)`.
    fn calculate_key_hash_value(&mut self, _field_array: *const c_void) -> Option<u32> {
        None
    }

    /// Acquire or release a table-level lock for the session `thd`. `lock_type`
    /// is the raw `F_RDLCK` / `F_WRLCK` / `F_UNLCK` integer.
    ///
    /// # Errors
    /// The default returns `Ok(())`, matching the handler base (always succeeds).
    fn external_lock(&mut self, _thd: Option<&sys::THD>, _lock_type: i32) -> EngineResult {
        Ok(())
    }

    /// Number of `THR_LOCK` entries the engine hands MySQL via `store_lock`. The
    /// default is `1`, matching the handler base.
    fn lock_count(&self) -> u32 {
        1
    }

    /// Release the lock held on the most recently read row. The default is a
    /// no-op, matching the handler base.
    fn unlock_row(&mut self) {}

    /// Begin a statement while the table is already locked (called instead of
    /// [`external_lock`](Self::external_lock) under `LOCK TABLES`). `lock_type`
    /// is the raw `thr_lock_type` integer.
    ///
    /// # Errors
    /// The default returns `Ok(())`, matching the handler base.
    fn start_stmt(&mut self, _thd: Option<&sys::THD>, _lock_type: i32) -> EngineResult {
        Ok(())
    }

    /// Whether the last row was read with a semi-consistent read (skipped under
    /// an existing lock rather than waiting). The default is `false`, matching
    /// the handler base.
    fn was_semi_consistent_read(&mut self) -> bool {
        false
    }

    /// Enable or disable semi-consistent reads for subsequent row reads. The
    /// default is a no-op, matching the handler base.
    fn try_semi_consistent_read(&mut self, _enable: bool) {}

    /// Begin read-before-write removal (`HA_READ_BEFORE_WRITE_REMOVAL`). Return
    /// `None` (the default) to use the handler base, which asserts — only engines
    /// advertising the capability should override and return `Some(active)`.
    fn start_read_removal(&mut self) -> Option<bool> {
        None
    }

    /// End read-before-write removal and report the number of rows actually
    /// written. Return `None` (the default) to use the handler base, which
    /// asserts; engines advertising the capability return `Some(rows)`.
    fn end_read_removal(&mut self) -> Option<u64> {
        None
    }

    /// Reserve a block of auto-increment values. `offset` and `increment` define
    /// the value series and `nb_desired` how many values MySQL wants. Return
    /// `Some((first_value, nb_reserved))` to supply the block, or `None` (the
    /// default) to use the handler base, which derives values from table stats.
    fn get_auto_increment(
        &mut self,
        _offset: u64,
        _increment: u64,
        _nb_desired: u64,
    ) -> Option<(u64, u64)> {
        None
    }

    /// Release auto-increment values reserved by
    /// [`get_auto_increment`](Self::get_auto_increment) but not used. The default
    /// is a no-op, matching the handler base.
    fn release_auto_increment(&mut self) {}

    /// Print a diagnostic for handler error code `error` (`errflag` carries the
    /// `myf` formatting flags). Return `true` when the engine emitted its own
    /// message; `false` (the default) lets the handler base print the standard
    /// `HA_ERR_*` diagnostic.
    fn print_error(&mut self, _error: i32, _errflag: u64) -> bool {
        false
    }

    /// Engine-specific message for handler error code `error`, paired with a
    /// flag marking the error as transient. Return `Some((message, temporary))`
    /// to surface `message` to the client — formatted as a temporary error when
    /// `temporary` is `true` — or `None` (the default) to use the handler base
    /// (no engine message).
    fn error_message(&mut self, _error: i32) -> Option<(String, bool)> {
        None
    }

    /// Names of the child table and key for the most recent
    /// `HA_ERR_FOREIGN_DUPLICATE_KEY`. Return `Some((table, key))` to report
    /// them, or `None` (the default) to use the handler base (names unavailable).
    fn foreign_dup_key(&mut self) -> Option<(String, String)> {
        None
    }

    /// Whether handler error code `error` may be ignored (e.g. duplicate-key
    /// under `INSERT IGNORE`). Return `None` (the default) to use the handler
    /// base classification; engines return `Some(flag)` to override it.
    fn is_ignorable_error(&mut self, _error: i32) -> Option<bool> {
        None
    }

    /// Whether handler error code `error` is fatal to the running statement.
    /// Return `None` (the default) to use the handler base classification;
    /// engines return `Some(flag)` to override it.
    fn is_fatal_error(&mut self, _error: i32) -> Option<bool> {
        None
    }

    /// Perform an `HA_EXTRA_*` hint operation (`operation` is the raw
    /// `ha_extra_function` integer). Hints are advisory.
    ///
    /// # Errors
    /// The default returns `Ok(())`, matching the handler base (hints ignored).
    fn extra(&mut self, _operation: i32) -> EngineResult {
        Ok(())
    }

    /// Perform an `HA_EXTRA_*` hint with a size argument (`cache_size`). The
    /// default forwards to [`extra`](Self::extra), matching the handler base.
    ///
    /// # Errors
    /// Propagates whatever [`extra`](Self::extra) returns.
    fn extra_opt(&mut self, operation: i32, _cache_size: u64) -> EngineResult {
        self.extra(operation)
    }

    /// Reset per-statement state so the handler can be reused for the next
    /// statement (clears hints, range state, etc.).
    ///
    /// # Errors
    /// The default returns `Ok(())`, matching the handler base.
    fn reset(&mut self) -> EngineResult {
        Ok(())
    }

    /// Notify the engine that MySQL changed the read/write column bitmaps. The
    /// default is a no-op, matching the handler base.
    fn column_bitmaps_signal(&mut self) {}

    /// Prepare engine state for use through the SQL `HANDLER` interface. The
    /// default is a no-op, matching the handler base.
    fn init_table_handle_for_handler(&mut self) {}

    /// Report which in-place `ALTER TABLE` algorithm the engine supports for the
    /// change described by `alter_info` on `altered_table`, as the raw
    /// `enum_alter_inplace_result` integer. Return `None` (the default) to use
    /// the handler base, which classifies the change from the alter flags;
    /// engines return `Some(result)` to override.
    fn check_if_supported_inplace_alter(
        &mut self,
        _altered_table: Option<&sys::TABLE>,
        _alter_info: Option<&sys::AlterInplaceInfo>,
    ) -> Option<i32> {
        None
    }

    /// Prepare an in-place `ALTER TABLE` (allocate resources, validate) before
    /// the change is applied. Return `Some(true)` on error, `Some(false)` on
    /// success, or `None` (the default) to use the handler base (success).
    fn prepare_inplace_alter_table(
        &mut self,
        _altered_table: Option<&sys::TABLE>,
        _alter_info: Option<&sys::AlterInplaceInfo>,
        _old_table_def: Option<&sys::DdTable>,
        _new_table_def: Option<&sys::DdTable>,
    ) -> Option<bool> {
        None
    }

    /// Apply an in-place `ALTER TABLE` change. Return `Some(true)` on error,
    /// `Some(false)` on success, or `None` (the default) to use the handler base
    /// (success / no-op).
    fn inplace_alter_table(
        &mut self,
        _altered_table: Option<&sys::TABLE>,
        _alter_info: Option<&sys::AlterInplaceInfo>,
        _old_table_def: Option<&sys::DdTable>,
        _new_table_def: Option<&sys::DdTable>,
    ) -> Option<bool> {
        None
    }

    /// Commit (`commit == true`) or roll back an in-place `ALTER TABLE`. Return
    /// `Some(true)` on error, `Some(false)` on success, or `None` (the default)
    /// to use the handler base, which clears the group-commit context.
    fn commit_inplace_alter_table(
        &mut self,
        _altered_table: Option<&sys::TABLE>,
        _alter_info: Option<&sys::AlterInplaceInfo>,
        _commit: bool,
        _old_table_def: Option<&sys::DdTable>,
        _new_table_def: Option<&sys::DdTable>,
    ) -> Option<bool> {
        None
    }

    /// Notify the engine that an in-place `ALTER TABLE` finished and the table
    /// definition was updated. The default is a no-op, matching the handler
    /// base. No error may be reported here.
    fn notify_table_changed(&mut self, _alter_info: Option<&sys::AlterInplaceInfo>) {}

    /// Whether the create options in `create_info` (with `table_changes` flags)
    /// are incompatible with the existing data, for the deprecated copy-based
    /// ALTER path. Return `None` (the default) to use the handler base
    /// (`COMPATIBLE_DATA_NO`, i.e. incompatible); engines return `Some(flag)`.
    fn check_if_incompatible_data(
        &mut self,
        _create_info: Option<&sys::HA_CREATE_INFO>,
        _table_changes: u32,
    ) -> Option<bool> {
        None
    }

    /// Run `CHECK TABLE` for `check_opt`, returning a raw `HA_ADMIN_*` code.
    /// Return `None` (the default) to use the handler base
    /// (`HA_ADMIN_NOT_IMPLEMENTED`); engines return `Some(code)`.
    fn check(
        &mut self,
        _thd: Option<&sys::THD>,
        _check_opt: Option<&sys::HaCheckOpt>,
    ) -> Option<i32> {
        None
    }

    /// Run `REPAIR TABLE` for `check_opt`, returning a raw `HA_ADMIN_*` code.
    /// Return `None` (the default) to use the handler base
    /// (`HA_ADMIN_NOT_IMPLEMENTED`); engines that advertise `HA_CAN_REPAIR`
    /// return `Some(code)`.
    fn repair(
        &mut self,
        _thd: Option<&sys::THD>,
        _check_opt: Option<&sys::HaCheckOpt>,
    ) -> Option<i32> {
        None
    }

    /// Run `OPTIMIZE TABLE` for `check_opt`, returning a raw `HA_ADMIN_*` code.
    /// Return `None` (the default) to use the handler base
    /// (`HA_ADMIN_NOT_IMPLEMENTED`); engines return `Some(code)`.
    fn optimize(
        &mut self,
        _thd: Option<&sys::THD>,
        _check_opt: Option<&sys::HaCheckOpt>,
    ) -> Option<i32> {
        None
    }

    /// Run `ANALYZE TABLE` for `check_opt`, returning a raw `HA_ADMIN_*` code.
    /// Return `None` (the default) to use the handler base
    /// (`HA_ADMIN_NOT_IMPLEMENTED`); engines return `Some(code)`.
    fn analyze(
        &mut self,
        _thd: Option<&sys::THD>,
        _check_opt: Option<&sys::HaCheckOpt>,
    ) -> Option<i32> {
        None
    }

    /// Check and, if needed, repair the table on crash recovery. Return
    /// `Some(true)` on error / not supported, `Some(false)` on success, or
    /// `None` (the default) to use the handler base (`true`).
    fn check_and_repair(&mut self, _thd: Option<&sys::THD>) -> Option<bool> {
        None
    }

    /// Check whether the table needs upgrading, returning a raw `HA_ADMIN_*`
    /// code. Return `None` (the default) to use the handler base (`0`, no
    /// upgrade needed); engines return `Some(code)`.
    fn check_for_upgrade(&mut self, _check_opt: Option<&sys::HaCheckOpt>) -> Option<i32> {
        None
    }

    /// Preload indexes into a named key cache (`ASSIGN_TO_KEYCACHE`), returning a
    /// raw `HA_ADMIN_*` code. Return `None` (the default) to use the handler base
    /// (`HA_ADMIN_NOT_IMPLEMENTED`); engines return `Some(code)`.
    fn assign_to_keycache(
        &mut self,
        _thd: Option<&sys::THD>,
        _check_opt: Option<&sys::HaCheckOpt>,
    ) -> Option<i32> {
        None
    }

    /// Preload index blocks into the default key cache (`LOAD INDEX`), returning
    /// a raw `HA_ADMIN_*` code. Return `None` (the default) to use the handler
    /// base (`HA_ADMIN_NOT_IMPLEMENTED`); engines return `Some(code)`.
    fn preload_keys(
        &mut self,
        _thd: Option<&sys::THD>,
        _check_opt: Option<&sys::HaCheckOpt>,
    ) -> Option<i32> {
        None
    }

    /// Disable indexes in the given `mode` (`ALTER TABLE ... DISABLE KEYS`),
    /// returning a raw handler code. Return `None` (the default) to use the
    /// handler base (`HA_ERR_WRONG_COMMAND`); engines return `Some(code)`.
    fn disable_indexes(&mut self, _mode: u32) -> Option<i32> {
        None
    }

    /// Enable indexes in the given `mode` (`ALTER TABLE ... ENABLE KEYS`),
    /// returning a raw handler code. Return `None` (the default) to use the
    /// handler base (`HA_ERR_WRONG_COMMAND`); engines return `Some(code)`.
    fn enable_indexes(&mut self, _mode: u32) -> Option<i32> {
        None
    }

    /// Discard (`discard == true`) or import the tablespace for `table_def`,
    /// returning a raw handler code. Return `None` (the default) to use the
    /// handler base (`HA_ERR_WRONG_COMMAND`); engines return `Some(code)`.
    fn discard_or_import_tablespace(
        &mut self,
        _discard: bool,
        _table_def: Option<&sys::DdTable>,
    ) -> Option<i32> {
        None
    }

    /// Offer the WHERE condition `cond` (an opaque `Item *` the binding
    /// round-trips without dereference) for engine-side evaluation. Return the
    /// part the engine will *not* handle: `cond` (the default) means no
    /// pushdown, a null pointer means the engine took the whole condition.
    /// Engines cannot yet construct `Item`s, so only pass-through or null are
    /// expressible.
    fn cond_push(&mut self, cond: *const c_void) -> *const c_void {
        cond
    }

    /// Offer the index condition `idx_cond` on index `keyno` for engine-side
    /// evaluation (an opaque `Item *` the binding round-trips without
    /// dereference). Return the part not handled: `idx_cond` (the default) means
    /// no pushdown, null means fully handled.
    fn idx_cond_push(&mut self, _keyno: u32, idx_cond: *mut c_void) -> *mut c_void {
        idx_cond
    }

    /// Discard any index condition previously accepted via
    /// [`idx_cond_push`](Self::idx_cond_push). The default is a no-op; the shim
    /// always resets the handler base's pushed-condition state regardless.
    fn cancel_pushed_idx_cond(&mut self) {}

    /// The `handlerton *` of the secondary engine this handler can push work
    /// down to, as an opaque pointer. Return null (the default) when the engine
    /// supports no pushdown; round-trip a handlerton pointer otherwise.
    fn hton_supporting_engine_pushdown(&mut self) -> *const c_void {
        core::ptr::null()
    }

    /// Number of joins pushed down to the engine for the current query. The
    /// default is `0`, matching the handler base.
    fn number_of_pushed_joins(&self) -> u32 {
        0
    }

    /// The `TABLE *` of this handler's member in a pushed join, as an opaque
    /// pointer, or null (the default) when not part of a pushed join.
    fn member_of_pushed_join(&self) -> *const c_void {
        core::ptr::null()
    }

    /// The `TABLE *` of the root of this handler's pushed join, as an opaque
    /// pointer, or null (the default) when not part of a pushed join.
    fn parent_of_pushed_join(&self) -> *const c_void {
        core::ptr::null()
    }

    /// Bitmap (`table_map`) of the tables in this handler's pushed join. The
    /// default is `0`, matching the handler base.
    fn tables_in_pushed_join(&self) -> u64 {
        0
    }

    /// Populate engine-specific fields of `create_info` (an opaque MySQL
    /// `HA_CREATE_INFO`) before `SHOW CREATE TABLE`. The default is a no-op,
    /// matching the handler base; the binding cannot mutate `HA_CREATE_INFO`
    /// from Rust yet, so this is a notification.
    fn update_create_info(&mut self, _create_info: Option<&sys::HA_CREATE_INFO>) {}

    /// Engine-specific text appended to the `CREATE TABLE` statement (after the
    /// closing paren). Return `Some(text)` to append it, or `None` (the default)
    /// to append nothing, matching the handler base.
    fn append_create_info(&mut self) -> Option<String> {
        None
    }

    /// Prepare the handler to position rows by a hidden primary key. The default
    /// is a no-op notification; the shim always runs the handler base, which
    /// sets up the hidden-key iteration state.
    fn use_hidden_primary_key(&mut self) {}

    /// Adopt the shared `Handler_share` state (`arg` is an opaque
    /// `Handler_share **` the binding round-trips). Return `Some(false)` on
    /// success, `Some(true)` on error, or `None` (the default) to use the handler
    /// base, which stores the reference for cross-handler sharing.
    fn set_ha_share_ref(&mut self, _arg: *mut c_void) -> Option<bool> {
        None
    }

    /// Compare two row-position references `ref1` and `ref2` (each the handler's
    /// `ref_length` bytes), returning an ordering as a negative / zero / positive
    /// `i32`. Return `None` (the default) to use the handler base (`memcmp`);
    /// engines with a structured position return `Some(ordering)`.
    fn cmp_ref(&mut self, _ref1: &[u8], _ref2: &[u8]) -> Option<i32> {
        None
    }

    /// Record `reason` as the error to raise for a failed external (secondary)
    /// engine offload. The default is a no-op, matching the handler base.
    fn set_external_table_offload_error(&mut self, _reason: &str) {}

    /// Raise the error previously recorded by
    /// [`set_external_table_offload_error`](Self::set_external_table_offload_error).
    /// The default is a no-op, matching the handler base.
    fn external_table_offload_error(&self) {}

    /// Create a clone of this handler for `name` allocated in `mem_root` (an
    /// opaque `MEM_ROOT *`), returning an opaque `handler *`. Return a null
    /// pointer (the default) to use the handler base, which builds a fresh
    /// handler of the same type — engines cannot construct a `handler` from Rust.
    fn clone_handler(&mut self, _name: &str, _mem_root: *mut c_void) -> *mut c_void {
        core::ptr::null_mut()
    }

    /// Capacity for multi-valued index keys as `(max_keys, max_total_bytes)`.
    /// Return `None` (the default) to use the handler base (`(0, 0)`, no
    /// multi-valued index support); engines return `Some((keys, bytes))`.
    fn mv_key_capacity(&self) -> Option<(u32, u64)> {
        None
    }

    /// The engine's `Partition_handler *` as an opaque pointer, or null (the
    /// default) when the engine does not implement native partitioning.
    fn get_partition_handler(&mut self) -> *mut c_void {
        core::ptr::null_mut()
    }
}
