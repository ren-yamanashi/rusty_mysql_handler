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

//! `dict_init_mode_t` and `dict_recovery_mode_t` from `sql/handler.h`.

/// How a data-dictionary backend should initialise its on-disk files.
///
/// Mirrors `enum dict_init_mode_t` in `mysql-server/sql/handler.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DictInitMode {
    /// `DICT_INIT_CREATE_FILES`: create all required SE files.
    CreateFiles,
    /// `DICT_INIT_CHECK_FILES`: verify existence of expected files.
    CheckFiles,
}

impl DictInitMode {
    /// Decode the C `enum dict_init_mode_t` value. Unknown values map to
    /// [`DictInitMode::CheckFiles`] so the engine still observes a defined
    /// variant.
    #[must_use]
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::CreateFiles,
            _ => Self::CheckFiles,
        }
    }
}

/// Mode for data-dictionary recovery during startup.
///
/// Mirrors `enum dict_recovery_mode_t` in `mysql-server/sql/handler.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DictRecoveryMode {
    /// `DICT_RECOVERY_INITIALIZE_SERVER`: first start of a new server.
    InitializeServer,
    /// `DICT_RECOVERY_INITIALIZE_TABLESPACES`: first start, create tablespaces.
    InitializeTablespaces,
    /// `DICT_RECOVERY_RESTART_SERVER`: restart of an existing server.
    RestartServer,
}

impl DictRecoveryMode {
    /// Decode the C `enum dict_recovery_mode_t` value. Unknown values map to
    /// [`DictRecoveryMode::RestartServer`].
    #[must_use]
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::InitializeServer,
            1 => Self::InitializeTablespaces,
            _ => Self::RestartServer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dict_init_mode_round_trip() {
        assert_eq!(DictInitMode::from_raw(0), DictInitMode::CreateFiles);
        assert_eq!(DictInitMode::from_raw(1), DictInitMode::CheckFiles);
        assert_eq!(DictInitMode::from_raw(99), DictInitMode::CheckFiles);
    }

    #[test]
    fn dict_recovery_mode_round_trip() {
        assert_eq!(
            DictRecoveryMode::from_raw(0),
            DictRecoveryMode::InitializeServer
        );
        assert_eq!(
            DictRecoveryMode::from_raw(1),
            DictRecoveryMode::InitializeTablespaces
        );
        assert_eq!(
            DictRecoveryMode::from_raw(2),
            DictRecoveryMode::RestartServer
        );
        assert_eq!(
            DictRecoveryMode::from_raw(99),
            DictRecoveryMode::RestartServer
        );
    }
}
