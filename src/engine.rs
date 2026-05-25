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

use std::ffi::CStr;

use crate::sys;

/// Errors a storage engine can return; each maps to a MySQL `HA_ERR_*` code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EngineError {
    /// End of a table or index scan, returned from [`StorageEngine::rnd_next`]
    /// when the scan is exhausted.
    EndOfFile,
    /// The engine does not support the requested operation
    WrongCommand,
    /// The supplied table or schema name is not valid UTF-8 or otherwise
    /// unusable. Mapped to `HA_ERR_WRONG_TABLE_NAME` so operators see a
    /// name-level diagnostic instead of a generic internal error.
    InvalidName,
    /// Generic internal error; prefer a more specific variant when possible
    Internal,
}

impl EngineError {
    /// Convert to the matching MySQL `HA_ERR_*` integer expected at the
    /// `extern "C"` boundary.
    #[must_use]
    pub fn to_mysql_errno(self) -> i32 {
        match self {
            Self::EndOfFile => sys::HA_ERR_END_OF_FILE,
            Self::WrongCommand => sys::HA_ERR_WRONG_COMMAND,
            Self::InvalidName => sys::HA_ERR_WRONG_TABLE_NAME,
            Self::Internal => sys::HA_ERR_INTERNAL_ERROR,
        }
    }
}

/// Result alias used throughout the [`StorageEngine`] trait
pub type EngineResult<T = ()> = Result<T, EngineError>;

/// Whether MySQL has just reset the data-dictionary entry and any cached
/// engine-private metadata should be re-emitted from scratch
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResetCachedState {
    /// Reuse whatever the engine has cached
    Keep,
    /// Discard cached state and re-emit from authoritative source
    Reset,
}

impl From<bool> for ResetCachedState {
    fn from(needs_reset: bool) -> Self {
        if needs_reset { Self::Reset } else { Self::Keep }
    }
}

/// Search semantics for an index lookup, mirroring MySQL's `ha_rkey_function`.
///
/// Passed to [`StorageEngine::index_read_map`] to describe how the supplied key
/// should be matched: an exact hit, the nearest neighbour in a direction, a
/// prefix, or one of the spatial (minimum-bounding-rectangle) relations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RKeyFunction {
    /// Find the first record with exactly this key, else error
    KeyExact,
    /// This record or the next one
    KeyOrNext,
    /// This record or the previous one
    KeyOrPrev,
    /// First record after this key
    AfterKey,
    /// First record before this key
    BeforeKey,
    /// First record sharing this key prefix
    Prefix,
    /// Last record sharing this key prefix
    PrefixLast,
    /// Last record with this prefix, or the previous one
    PrefixLastOrPrev,
    /// Minimum bounding rectangle contains the key
    MbrContain,
    /// Minimum bounding rectangle intersects the key
    MbrIntersect,
    /// Minimum bounding rectangle is within the key
    MbrWithin,
    /// Minimum bounding rectangle is disjoint from the key
    MbrDisjoint,
    /// Minimum bounding rectangle equals the key
    MbrEqual,
    /// Nearest-neighbour spatial search
    NearestNeighbor,
    /// Unrecognised value; MySQL's `HA_READ_INVALID` or an out-of-range code
    Invalid,
}

impl RKeyFunction {
    /// Map the raw `ha_rkey_function` integer supplied at the FFI boundary to a
    /// variant. Any unknown code (including `HA_READ_INVALID == -1`) becomes
    /// [`RKeyFunction::Invalid`] so the engine never observes an undefined value.
    pub(crate) fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::KeyExact,
            1 => Self::KeyOrNext,
            2 => Self::KeyOrPrev,
            3 => Self::AfterKey,
            4 => Self::BeforeKey,
            5 => Self::Prefix,
            6 => Self::PrefixLast,
            7 => Self::PrefixLastOrPrev,
            8 => Self::MbrContain,
            9 => Self::MbrIntersect,
            10 => Self::MbrWithin,
            11 => Self::MbrDisjoint,
            12 => Self::MbrEqual,
            13 => Self::NearestNeighbor,
            _ => Self::Invalid,
        }
    }
}

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
}
