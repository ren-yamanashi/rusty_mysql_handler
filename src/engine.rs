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
    pub fn as_mysql_errno(self) -> i32 {
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
    /// # Errors
    /// The default returns [`EngineError::WrongCommand`] (no-op engines need
    /// not implement this).
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

    /// Engine-internal hook fired before [`delete_table`](Self::delete_table)
    /// (or in lieu of it for some paths). Default is a no-op.
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

    /// Populate engine-private metadata in `dd_table`. `reset` indicates that
    /// the data-dictionary entry has been reset and any cached state should
    /// be re-emitted. The default returns `false` (no private data written).
    fn get_se_private_data(&mut self, _dd_table: Option<&sys::DdTable>, _reset: bool) -> bool {
        false
    }

    /// Inject implicit columns and indexes the engine requires for `table_obj`
    /// to be created. The default leaves the definition unchanged.
    ///
    /// # Errors
    /// The default returns `Ok(())`.
    fn get_extra_columns_and_keys(
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
    /// upgrade. Returns `true` to signal failure (matches the C++ bool
    /// convention). The default returns `false`.
    fn upgrade_table(
        &mut self,
        _thd: Option<&sys::THD>,
        _dbname: &str,
        _table_name: &str,
        _dd_table: Option<&sys::DdTable>,
    ) -> bool {
        false
    }
}
