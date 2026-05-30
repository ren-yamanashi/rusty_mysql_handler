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

//! `Ha_clone_mode` and `Ha_clone_type` from `sql/handler.h`. Used as the
//! `mode` / `type` arguments to the clone interface trait methods.

/// Mode for starting a clone operation.
///
/// Mirrors `enum Ha_clone_mode` in `mysql-server/sql/handler.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum HaCloneMode {
    /// `HA_CLONE_MODE_START`: first start of a clone.
    Start,
    /// `HA_CLONE_MODE_RESTART`: restart of a previously paused clone.
    Restart,
    /// `HA_CLONE_MODE_ADD_TASK`: add a worker task to an existing clone.
    AddTask,
    /// `HA_CLONE_MODE_VERSION`: query supported version.
    Version,
    /// `HA_CLONE_MODE_MAX`: sentinel; treated as `Start` for forward-compat.
    Max,
}

impl HaCloneMode {
    /// Decode the C `enum Ha_clone_mode` value. Unknown values fall back to
    /// [`HaCloneMode::Start`].
    #[must_use]
    pub const fn from_raw(value: u32) -> Self {
        match value {
            1 => Self::Restart,
            2 => Self::AddTask,
            3 => Self::Version,
            4 => Self::Max,
            _ => Self::Start,
        }
    }
}

/// Clone-transfer type the source engine should perform.
///
/// Mirrors `enum Ha_clone_type` in `mysql-server/sql/handler.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum HaCloneType {
    /// `HA_CLONE_BLOCKING`: serialised copy (write operations must block).
    Blocking,
    /// `HA_CLONE_REDO`: archive redo log to support concurrent DML.
    Redo,
    /// `HA_CLONE_PAGE`: page-tracked incremental.
    Page,
    /// `HA_CLONE_HYBRID`: page tracking + redo (currently InnoDB).
    Hybrid,
    /// `HA_CLONE_MULTI_TASK`: multiple worker threads.
    MultiTask,
    /// `HA_CLONE_RESTART`: restart after network failure.
    Restart,
    /// Forward-compatible fallback.
    Unknown,
}

impl HaCloneType {
    /// Decode the C `enum Ha_clone_type` value (a `size_t` upstream).
    #[must_use]
    pub const fn from_raw(value: usize) -> Self {
        match value {
            0 => Self::Blocking,
            1 => Self::Redo,
            2 => Self::Page,
            3 => Self::Hybrid,
            4 => Self::MultiTask,
            5 => Self::Restart,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_mode_known_values() {
        assert_eq!(HaCloneMode::from_raw(0), HaCloneMode::Start);
        assert_eq!(HaCloneMode::from_raw(1), HaCloneMode::Restart);
        assert_eq!(HaCloneMode::from_raw(2), HaCloneMode::AddTask);
        assert_eq!(HaCloneMode::from_raw(3), HaCloneMode::Version);
        assert_eq!(HaCloneMode::from_raw(4), HaCloneMode::Max);
        assert_eq!(HaCloneMode::from_raw(99), HaCloneMode::Start);
    }

    #[test]
    fn clone_type_known_values() {
        assert_eq!(HaCloneType::from_raw(0), HaCloneType::Blocking);
        assert_eq!(HaCloneType::from_raw(1), HaCloneType::Redo);
        assert_eq!(HaCloneType::from_raw(2), HaCloneType::Page);
        assert_eq!(HaCloneType::from_raw(3), HaCloneType::Hybrid);
        assert_eq!(HaCloneType::from_raw(4), HaCloneType::MultiTask);
        assert_eq!(HaCloneType::from_raw(5), HaCloneType::Restart);
        assert_eq!(HaCloneType::from_raw(99), HaCloneType::Unknown);
    }
}
