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

//! Raw FFI bindings: MySQL handler constants and opaque C++ types

#[allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
#[allow(missing_docs, unreachable_pub, missing_debug_implementations)]
#[allow(clippy::all, clippy::pedantic)]
mod generated {
    include!("sys_bindings.rs");
}

pub use generated::*;

// Hand-written as `u64` because `Table_flags` is `unsigned long long` upstream
// but bindgen's default macro type would render these as `i64`.

/// `HA_BINLOG_ROW_CAPABLE` from `sql/handler.h`
pub const HA_BINLOG_ROW_CAPABLE: u64 = 1 << 34;

/// `HA_BINLOG_STMT_CAPABLE` from `sql/handler.h`
pub const HA_BINLOG_STMT_CAPABLE: u64 = 1 << 35;

/// Opaque C++ `RustHandlerBase` from `shim/binding.hpp`
#[repr(C)]
#[derive(Debug)]
pub struct RustHandlerBase([u8; 0]);

/// Opaque MySQL `TABLE`
#[repr(C)]
#[derive(Debug)]
pub struct TABLE([u8; 0]);

/// Opaque MySQL `TABLE_SHARE`
#[repr(C)]
#[derive(Debug)]
pub struct TABLE_SHARE([u8; 0]);

/// Opaque MySQL `THD` (thread handle)
#[repr(C)]
#[derive(Debug)]
pub struct THD([u8; 0]);

/// Opaque MySQL data-dictionary `dd::Table`
#[repr(C)]
#[derive(Debug)]
pub struct DdTable([u8; 0]);

/// Opaque MySQL `HA_CREATE_INFO`
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct HA_CREATE_INFO([u8; 0]);

/// Opaque MySQL `KEY` (index descriptor)
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct KEY([u8; 0]);

/// Opaque MySQL `List<Create_field>` (C++ template instantiation)
#[repr(C)]
#[derive(Debug)]
pub struct ListCreateField([u8; 0]);

/// Opaque MySQL `Rows_mysql` (bulk-load row batch)
#[repr(C)]
#[derive(Debug)]
pub struct RowsMysql([u8; 0]);

/// Opaque MySQL `Bulk_load::Stat_callbacks` (bulk-load progress callbacks)
#[repr(C)]
#[derive(Debug)]
pub struct BulkLoadStatCallbacks([u8; 0]);

/// Opaque MySQL `String` (a full-text query string, among other uses)
#[repr(C)]
#[derive(Debug)]
pub struct MysqlString([u8; 0]);

/// Opaque MySQL `Ft_hints` (full-text search hints)
#[repr(C)]
#[derive(Debug)]
pub struct FtHints([u8; 0]);

/// Opaque MySQL `RANGE_SEQ_IF` (multi-range read range-sequence interface)
#[repr(C)]
#[derive(Debug)]
pub struct RangeSeqIf([u8; 0]);

/// Opaque MySQL `Cost_estimate` (optimizer cost accumulator)
#[repr(C)]
#[derive(Debug)]
pub struct CostEstimate([u8; 0]);

/// Opaque MySQL `HANDLER_BUFFER` (caller-owned multi-range read scratch buffer)
#[repr(C)]
#[derive(Debug)]
pub struct HandlerBuffer([u8; 0]);

/// Opaque MySQL `Field` (one table column's metadata and value accessors)
#[repr(C)]
#[derive(Debug)]
pub struct Field([u8; 0]);

/// Opaque MySQL `Alter_inplace_info` (in-place `ALTER TABLE` change descriptor)
#[repr(C)]
#[derive(Debug)]
pub struct AlterInplaceInfo([u8; 0]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ha_err_end_of_file_is_137() {
        assert_eq!(HA_ERR_END_OF_FILE, 137);
    }

    #[test]
    fn ha_binlog_stmt_capable_bit() {
        assert_eq!(HA_BINLOG_STMT_CAPABLE, 1u64 << 35);
    }
}
