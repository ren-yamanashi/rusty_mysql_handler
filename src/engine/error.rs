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

//! Error type and result alias for the storage-engine interface.

use crate::sys;

/// Errors a storage engine can return; each maps to a MySQL `HA_ERR_*` code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EngineError {
    /// End of a table or index scan, returned from [`StorageEngine::rnd_next`]
    /// when the scan is exhausted.
    ///
    /// [`StorageEngine::rnd_next`]: crate::engine::StorageEngine::rnd_next
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

/// Result alias used throughout the [`StorageEngine`](crate::engine::StorageEngine) trait
pub type EngineResult<T = ()> = Result<T, EngineError>;
