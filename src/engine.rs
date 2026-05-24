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

//! Safe storage-engine interface. Implementors plug into the C++ shim via
//! `Box<dyn StorageEngine>` held inside an `EngineContext`.

use std::ffi::CStr;

use crate::sys;

/// Storage-engine error variants. Each variant maps to a MySQL `HA_ERR_*` code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EngineError {
    /// End of a table or index scan.
    EndOfFile,
    /// Operation not supported by this engine.
    WrongCommand,
    /// Generic internal error.
    Internal,
}

impl EngineError {
    /// Convert to the matching MySQL `HA_ERR_*` integer for `extern "C"` returns.
    #[must_use]
    pub fn to_mysql_errno(self) -> i32 {
        match self {
            Self::EndOfFile => sys::HA_ERR_END_OF_FILE,
            Self::WrongCommand => sys::HA_ERR_WRONG_COMMAND,
            Self::Internal => sys::HA_ERR_INTERNAL_ERROR,
        }
    }
}

/// Shorthand for results returned by [`StorageEngine`] methods.
pub type EngineResult<T = ()> = Result<T, EngineError>;

/// Safe trait that storage-engine implementations must satisfy.
///
/// One instance per opened table per thread. The `Send` bound is required
/// because MySQL hands each session its own worker thread; the
/// `EngineContext` is owned through a raw pointer, so the trait bound is
/// the only compile-time guarantee that implementations stay sound across
/// the FFI boundary.
pub trait StorageEngine: Send {
    /// Engine display name (e.g. `c"RUSTY"`). Must be a null-terminated, static
    /// string because the pointer is handed straight to MySQL.
    fn table_type(&self) -> &'static CStr;

    /// `HA_*` capability bitfield returned to the optimizer.
    fn table_flags(&self) -> u64;

    /// Per-index capability bitfield. `part` is the key part; if `all_parts` is
    /// set MySQL wants the combined flags up to and including `part`.
    fn index_flags(&self, idx: u32, part: u32, all_parts: bool) -> u32;

    /// Create the on-disk representation for a new table named `name`.
    fn create(&mut self, name: &str) -> EngineResult;

    /// Open an existing table named `name` in the given `mode`.
    fn open(&mut self, name: &str, mode: i32) -> EngineResult;

    /// Release any resources acquired by [`open`](Self::open).
    fn close(&mut self) -> EngineResult;

    /// Begin a full table scan. `scan == false` indicates a positioned (rnd_pos)
    /// access pattern only.
    fn rnd_init(&mut self, scan: bool) -> EngineResult;

    /// Fetch the next row into `buf`. Returns `EndOfFile` when exhausted.
    fn rnd_next(&mut self, buf: &mut [u8]) -> EngineResult;

    /// Fetch a row by the position previously stored via [`position`].
    fn rnd_pos(&mut self, buf: &mut [u8], pos: &[u8]) -> EngineResult;

    /// Store the position of the current row. The recorded bytes are passed back
    /// to [`rnd_pos`](Self::rnd_pos) on subsequent positioned reads.
    fn position(&mut self, record: &[u8]);

    /// Refresh statistics (rows, deleted rows, data length, etc.).
    fn info(&mut self, flag: u32) -> EngineResult;
}
