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

//! `ts_command_type` and `Tablespace_type` from `sql/handler.h`.

/// The tablespace DDL command MySQL is asking the engine to validate.
///
/// Mirrors `enum ts_command_type` in `mysql-server/sql/handler.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TsCommandType {
    /// `TS_CMD_NOT_DEFINED = -1`
    NotDefined,
    /// `CREATE_TABLESPACE = 0`
    CreateTablespace,
    /// `ALTER_TABLESPACE = 1`
    AlterTablespace,
    /// `CREATE_LOGFILE_GROUP = 2`
    CreateLogfileGroup,
    /// `ALTER_LOGFILE_GROUP = 3`
    AlterLogfileGroup,
    /// `DROP_TABLESPACE = 4`
    DropTablespace,
    /// `DROP_LOGFILE_GROUP = 5`
    DropLogfileGroup,
    /// `CHANGE_FILE_TABLESPACE = 6`
    ChangeFileTablespace,
    /// `ALTER_ACCESS_MODE_TABLESPACE = 7`
    AlterAccessModeTablespace,
    /// `CREATE_UNDO_TABLESPACE = 8`
    CreateUndoTablespace,
    /// `ALTER_UNDO_TABLESPACE = 9`
    AlterUndoTablespace,
    /// `DROP_UNDO_TABLESPACE = 10`
    DropUndoTablespace,
    /// Forward-compatible fallback for any value MySQL adds in the future.
    Unknown,
}

impl TsCommandType {
    /// Decode the C `enum ts_command_type` value. The C enum uses -1 for the
    /// not-defined sentinel; unknown positive values map to
    /// [`TsCommandType::Unknown`].
    #[must_use]
    pub const fn from_raw(value: i32) -> Self {
        match value {
            -1 => Self::NotDefined,
            0 => Self::CreateTablespace,
            1 => Self::AlterTablespace,
            2 => Self::CreateLogfileGroup,
            3 => Self::AlterLogfileGroup,
            4 => Self::DropTablespace,
            5 => Self::DropLogfileGroup,
            6 => Self::ChangeFileTablespace,
            7 => Self::AlterAccessModeTablespace,
            8 => Self::CreateUndoTablespace,
            9 => Self::AlterUndoTablespace,
            10 => Self::DropUndoTablespace,
            _ => Self::Unknown,
        }
    }
}

/// The classification of a data-dictionary tablespace.
///
/// Mirrors `enum class Tablespace_type` in `mysql-server/sql/handler.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TablespaceType {
    /// `SPACE_TYPE_DICTIONARY`
    Dictionary,
    /// `SPACE_TYPE_SYSTEM`
    System,
    /// `SPACE_TYPE_UNDO`
    Undo,
    /// `SPACE_TYPE_TEMPORARY`
    Temporary,
    /// `SPACE_TYPE_SHARED`
    Shared,
    /// `SPACE_TYPE_IMPLICIT`
    Implicit,
}

impl TablespaceType {
    /// Convert to the raw C value the shim writes into `Tablespace_type*`.
    #[must_use]
    pub const fn to_raw(self) -> u32 {
        match self {
            Self::Dictionary => 0,
            Self::System => 1,
            Self::Undo => 2,
            Self::Temporary => 3,
            Self::Shared => 4,
            Self::Implicit => 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ts_command_known_values() {
        assert_eq!(TsCommandType::from_raw(-1), TsCommandType::NotDefined);
        assert_eq!(TsCommandType::from_raw(0), TsCommandType::CreateTablespace);
        assert_eq!(
            TsCommandType::from_raw(10),
            TsCommandType::DropUndoTablespace
        );
    }

    #[test]
    fn ts_command_unknown_falls_back() {
        assert_eq!(TsCommandType::from_raw(99), TsCommandType::Unknown);
        assert_eq!(TsCommandType::from_raw(-99), TsCommandType::Unknown);
    }

    #[test]
    fn tablespace_type_to_raw_round_trip() {
        let pairs = [
            (TablespaceType::Dictionary, 0),
            (TablespaceType::System, 1),
            (TablespaceType::Undo, 2),
            (TablespaceType::Temporary, 3),
            (TablespaceType::Shared, 4),
            (TablespaceType::Implicit, 5),
        ];
        for (v, raw) in pairs {
            assert_eq!(v.to_raw(), raw);
        }
    }
}
