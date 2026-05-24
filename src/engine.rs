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

/// Errors a storage engine can return; each maps to a MySQL `HA_ERR_*` code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EngineError {
    /// End of a table or index scan, returned from [`StorageEngine::rnd_next`]
    /// when the scan is exhausted.
    EndOfFile,
    /// The engine does not support the requested operation.
    WrongCommand,
    /// Generic internal error; prefer a more specific variant when possible.
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
            Self::Internal => sys::HA_ERR_INTERNAL_ERROR,
        }
    }
}

/// Result alias used throughout the [`StorageEngine`] trait.
pub type EngineResult<T = ()> = Result<T, EngineError>;

/// The safe interface every storage engine implements.
///
/// MySQL constructs one instance per opened table per session worker thread,
/// so the trait requires `Send`. The `EngineContext` that owns a
/// `Box<dyn StorageEngine>` crosses the C++ FFI boundary as a raw pointer;
/// the `Send` bound is the only compile-time guarantee that this stays sound.
pub trait StorageEngine: Send {
    /// Engine display name shown by `SHOW ENGINES` and used as the `ENGINE=`
    /// value in `CREATE TABLE`. Must be a null-terminated `'static` C string
    /// (e.g. `c"RUSTY"`) because the pointer is handed straight to MySQL.
    fn table_type(&self) -> &'static CStr;

    /// `HA_*` capability bitfield advertised to the optimizer.
    fn table_flags(&self) -> u64;

    /// Per-index capability bitfield. `idx` is the index, `part` the key part;
    /// when `all_parts` is set MySQL wants the combined flags up to and
    /// including `part`.
    fn index_flags(&self, idx: u32, part: u32, all_parts: bool) -> u32;

    /// Create the on-disk representation for a new table named `name`.
    fn create(&mut self, name: &str) -> EngineResult;

    /// Open an existing table named `name` in the given `mode`.
    fn open(&mut self, name: &str, mode: i32) -> EngineResult;

    /// Release any resources acquired by [`open`](Self::open).
    fn close(&mut self) -> EngineResult;

    /// Begin a full table scan. `scan == false` indicates the optimizer will
    /// only use positioned access (`rnd_pos`).
    fn rnd_init(&mut self, scan: bool) -> EngineResult;

    /// Fetch the next row into `buf`. Returns [`EngineError::EndOfFile`] once
    /// the scan is exhausted.
    fn rnd_next(&mut self, buf: &mut [u8]) -> EngineResult;

    /// Fetch a row by the position previously recorded with
    /// [`position`](Self::position).
    fn rnd_pos(&mut self, buf: &mut [u8], pos: &[u8]) -> EngineResult;

    /// Record the current row's position. The bytes written are passed back
    /// to [`rnd_pos`](Self::rnd_pos) on subsequent positioned reads.
    fn position(&mut self, record: &[u8]);

    /// Refresh statistics (rows, deleted rows, data length, ...) for the
    /// optimizer.
    fn info(&mut self, flag: u32) -> EngineResult;
}
