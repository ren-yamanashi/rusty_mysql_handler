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

//! Engine decision returned from the `start_bulk_*` callbacks.

/// Whether the engine batches the rows of a multi-row UPDATE or DELETE itself,
/// or lets MySQL drive the statement one row at a time.
///
/// MySQL's `start_bulk_update` / `start_bulk_delete` encode this as an inverted
/// bool (`true` means "bulk not used"); this enum exists so engine code never
/// has to reason about that inversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BulkAccess {
    /// Engine batches the rows; MySQL routes them through the bulk path
    /// (`bulk_update_row` + `exec_bulk_update` for updates, `end_bulk_delete`
    /// for deletes). Maps to C++ `false`.
    Batched,
    /// Engine declines batching; MySQL performs the statement row by row via
    /// `update_row` / `delete_row`. Maps to C++ `true`, the handler-base default.
    PerRow,
}

impl BulkAccess {
    /// The MySQL bool expected by `start_bulk_update` / `start_bulk_delete`:
    /// `true` when the engine declines batching (normal per-row operation),
    /// matching the inverted handler-base contract.
    #[must_use]
    pub fn to_mysql_bool(self) -> bool {
        match self {
            Self::Batched => false,
            Self::PerRow => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_row_declines_batching() {
        assert!(BulkAccess::PerRow.to_mysql_bool());
    }

    #[test]
    fn batched_accepts_the_bulk_path() {
        assert!(!BulkAccess::Batched.to_mysql_bool());
    }
}
