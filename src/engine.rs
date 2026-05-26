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
mod error;
mod parallel_scan_init;
mod range_key;
mod reset_cached_state;
mod rkey_function;
mod sampling_method;

pub use bulk_access::BulkAccess;
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
}
