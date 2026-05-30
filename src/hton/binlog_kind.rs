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

//! `enum_binlog_func` and `enum_binlog_command` from `sql/handler.h`.

/// The binlog-related function MySQL is asking the engine to perform.
///
/// Mirrors `enum enum_binlog_func` in `mysql-server/sql/handler.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BinlogFunc {
    /// `BFN_RESET_LOGS = 1`
    ResetLogs,
    /// `BFN_RESET_SLAVE = 2`
    ResetSlave,
    /// `BFN_BINLOG_WAIT = 3`
    BinlogWait,
    /// `BFN_BINLOG_END = 4`
    BinlogEnd,
    /// `BFN_BINLOG_PURGE_FILE = 5`
    BinlogPurgeFile,
    /// `BFN_BINLOG_PURGE_WAIT = 6`
    BinlogPurgeWait,
    /// Forward-compatible fallback for any value MySQL adds in the future.
    Unknown,
}

impl BinlogFunc {
    /// Decode the C `enum enum_binlog_func` value. Unknown values map to
    /// [`BinlogFunc::Unknown`].
    #[must_use]
    pub const fn from_raw(value: u32) -> Self {
        match value {
            1 => Self::ResetLogs,
            2 => Self::ResetSlave,
            3 => Self::BinlogWait,
            4 => Self::BinlogEnd,
            5 => Self::BinlogPurgeFile,
            6 => Self::BinlogPurgeWait,
            _ => Self::Unknown,
        }
    }
}

/// The DDL command flavour MySQL is logging through the binary log.
///
/// Mirrors `enum enum_binlog_command` in `mysql-server/sql/handler.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BinlogCommand {
    /// `LOGCOM_CREATE_TABLE`
    CreateTable,
    /// `LOGCOM_ALTER_TABLE`
    AlterTable,
    /// `LOGCOM_RENAME_TABLE`
    RenameTable,
    /// `LOGCOM_DROP_TABLE`
    DropTable,
    /// `LOGCOM_CREATE_DB`
    CreateDb,
    /// `LOGCOM_ALTER_DB`
    AlterDb,
    /// `LOGCOM_DROP_DB`
    DropDb,
    /// Forward-compatible fallback for any value MySQL adds in the future.
    Unknown,
}

impl BinlogCommand {
    /// Decode the C `enum enum_binlog_command` value. Unknown values map to
    /// [`BinlogCommand::Unknown`].
    #[must_use]
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::CreateTable,
            1 => Self::AlterTable,
            2 => Self::RenameTable,
            3 => Self::DropTable,
            4 => Self::CreateDb,
            5 => Self::AlterDb,
            6 => Self::DropDb,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binlog_func_known_values() {
        assert_eq!(BinlogFunc::from_raw(1), BinlogFunc::ResetLogs);
        assert_eq!(BinlogFunc::from_raw(6), BinlogFunc::BinlogPurgeWait);
    }

    #[test]
    fn binlog_func_unknown_falls_back() {
        assert_eq!(BinlogFunc::from_raw(0), BinlogFunc::Unknown);
        assert_eq!(BinlogFunc::from_raw(99), BinlogFunc::Unknown);
    }

    #[test]
    fn binlog_command_known_values() {
        assert_eq!(BinlogCommand::from_raw(0), BinlogCommand::CreateTable);
        assert_eq!(BinlogCommand::from_raw(6), BinlogCommand::DropDb);
    }

    #[test]
    fn binlog_command_unknown_falls_back() {
        assert_eq!(BinlogCommand::from_raw(99), BinlogCommand::Unknown);
    }
}
