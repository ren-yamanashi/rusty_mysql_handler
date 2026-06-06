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

//! Engine-published per-table statistics.

/// Snapshot of the per-table statistics MySQL stores in `ha_statistics`.
///
/// Returned from
/// [`Handlerton::get_table_statistics`](crate::hton::Handlerton::get_table_statistics);
/// the shim copies each field into the matching `ha_statistics` slot
/// before handing control back to the optimizer. Engines build instances
/// through [`Self::new`] and the chained `with_*` setters; missing fields
/// stay zero, which matches MySQL's `ha_statistics()` default constructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct TableStatistics {
    records: u64,
    data_file_length: u64,
    max_data_file_length: u64,
    index_file_length: u64,
    max_index_file_length: u64,
    delete_length: u64,
    auto_increment_value: u64,
    deleted: u64,
    mean_rec_length: u64,
    create_time: i64,
    check_time: u64,
    update_time: u64,
    block_size: u32,
}

impl TableStatistics {
    /// Construct a statistics snapshot with every field zeroed — the same
    /// initial state `ha_statistics()` produces on the C++ side.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            records: 0,
            data_file_length: 0,
            max_data_file_length: 0,
            index_file_length: 0,
            max_index_file_length: 0,
            delete_length: 0,
            auto_increment_value: 0,
            deleted: 0,
            mean_rec_length: 0,
            create_time: 0,
            check_time: 0,
            update_time: 0,
            block_size: 0,
        }
    }

    /// Number of live rows in the table.
    #[must_use]
    pub const fn with_records(mut self, value: u64) -> Self {
        self.records = value;
        self
    }

    /// Size of the data segment in bytes.
    #[must_use]
    pub const fn with_data_file_length(mut self, value: u64) -> Self {
        self.data_file_length = value;
        self
    }

    /// Hard cap MySQL should believe for `data_file_length`.
    #[must_use]
    pub const fn with_max_data_file_length(mut self, value: u64) -> Self {
        self.max_data_file_length = value;
        self
    }

    /// Size of the index segment in bytes.
    #[must_use]
    pub const fn with_index_file_length(mut self, value: u64) -> Self {
        self.index_file_length = value;
        self
    }

    /// Hard cap MySQL should believe for `index_file_length`.
    #[must_use]
    pub const fn with_max_index_file_length(mut self, value: u64) -> Self {
        self.max_index_file_length = value;
        self
    }

    /// Bytes reclaimable by compacting deleted rows.
    #[must_use]
    pub const fn with_delete_length(mut self, value: u64) -> Self {
        self.delete_length = value;
        self
    }

    /// Next auto-increment value the engine intends to hand out.
    #[must_use]
    pub const fn with_auto_increment_value(mut self, value: u64) -> Self {
        self.auto_increment_value = value;
        self
    }

    /// Number of tombstoned rows still occupying space.
    #[must_use]
    pub const fn with_deleted(mut self, value: u64) -> Self {
        self.deleted = value;
        self
    }

    /// Physical record length (`stats.mean_rec_length` on the C++ side).
    #[must_use]
    pub const fn with_mean_rec_length(mut self, value: u64) -> Self {
        self.mean_rec_length = value;
        self
    }

    /// UNIX timestamp the table was created at; mirrors C++ `time_t`.
    #[must_use]
    pub const fn with_create_time(mut self, value: i64) -> Self {
        self.create_time = value;
        self
    }

    /// UNIX timestamp of the last CHECK TABLE run.
    #[must_use]
    pub const fn with_check_time(mut self, value: u64) -> Self {
        self.check_time = value;
        self
    }

    /// UNIX timestamp of the last data modification.
    #[must_use]
    pub const fn with_update_time(mut self, value: u64) -> Self {
        self.update_time = value;
        self
    }

    /// Index block size in bytes.
    #[must_use]
    pub const fn with_block_size(mut self, value: u32) -> Self {
        self.block_size = value;
        self
    }

    /// Raw record count (read by the shim when copying into `ha_statistics`).
    pub const fn records(&self) -> u64 {
        self.records
    }
    /// Data segment length in bytes.
    pub const fn data_file_length(&self) -> u64 {
        self.data_file_length
    }
    /// Hard cap for `data_file_length`.
    pub const fn max_data_file_length(&self) -> u64 {
        self.max_data_file_length
    }
    /// Index segment length in bytes.
    pub const fn index_file_length(&self) -> u64 {
        self.index_file_length
    }
    /// Hard cap for `index_file_length`.
    pub const fn max_index_file_length(&self) -> u64 {
        self.max_index_file_length
    }
    /// Reclaimable byte count.
    pub const fn delete_length(&self) -> u64 {
        self.delete_length
    }
    /// Next auto-increment value.
    pub const fn auto_increment_value(&self) -> u64 {
        self.auto_increment_value
    }
    /// Tombstoned row count.
    pub const fn deleted(&self) -> u64 {
        self.deleted
    }
    /// Mean record length.
    pub const fn mean_rec_length(&self) -> u64 {
        self.mean_rec_length
    }
    /// Create-time UNIX timestamp.
    pub const fn create_time(&self) -> i64 {
        self.create_time
    }
    /// Last-check UNIX timestamp.
    pub const fn check_time(&self) -> u64 {
        self.check_time
    }
    /// Last-update UNIX timestamp.
    pub const fn update_time(&self) -> u64 {
        self.update_time
    }
    /// Index block size in bytes.
    pub const fn block_size(&self) -> u32 {
        self.block_size
    }
}

impl Default for TableStatistics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_zeros_every_field() {
        let s = TableStatistics::new();
        assert_eq!(s.records(), 0);
        assert_eq!(s.data_file_length(), 0);
        assert_eq!(s.block_size(), 0);
    }

    #[test]
    fn builder_preserves_each_setter() {
        let s = TableStatistics::new()
            .with_records(100)
            .with_data_file_length(4096)
            .with_block_size(16);
        assert_eq!(s.records(), 100);
        assert_eq!(s.data_file_length(), 4096);
        assert_eq!(s.block_size(), 16);
    }
}
