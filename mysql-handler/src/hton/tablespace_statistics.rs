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

//! Engine-published per-tablespace statistics.

/// Snapshot of the numeric fields of MySQL's `ha_tablespace_statistics`.
///
/// Returned from
/// [`Handlerton::get_tablespace_statistics`](crate::hton::Handlerton::get_tablespace_statistics);
/// the shim copies each field into the matching `ha_tablespace_statistics`
/// slot. The five string fields (`m_type`, `m_logfile_group_name`,
/// `m_row_format`, `m_status`, `m_extra`) stay at their default-empty
/// values today — they need a separate dd::String_type setter that has
/// not been wired yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct TablespaceStatistics {
    id: u64,
    logfile_group_number: u64,
    free_extents: u64,
    total_extents: u64,
    extent_size: u64,
    initial_size: u64,
    maximum_size: u64,
    autoextend_size: u64,
    version: u64,
    data_free: u64,
}

impl TablespaceStatistics {
    /// Construct a snapshot with every numeric field zeroed.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            id: 0,
            logfile_group_number: 0,
            free_extents: 0,
            total_extents: 0,
            extent_size: 0,
            initial_size: 0,
            maximum_size: 0,
            autoextend_size: 0,
            version: 0,
            data_free: 0,
        }
    }

    /// Tablespace identifier (`m_id`).
    #[must_use]
    pub const fn with_id(mut self, value: u64) -> Self {
        self.id = value;
        self
    }
    /// Logfile group number — NDB only (`m_logfile_group_number`).
    #[must_use]
    pub const fn with_logfile_group_number(mut self, value: u64) -> Self {
        self.logfile_group_number = value;
        self
    }
    /// Number of free extents (`m_free_extents`).
    #[must_use]
    pub const fn with_free_extents(mut self, value: u64) -> Self {
        self.free_extents = value;
        self
    }
    /// Total number of extents (`m_total_extents`).
    #[must_use]
    pub const fn with_total_extents(mut self, value: u64) -> Self {
        self.total_extents = value;
        self
    }
    /// Extent size in bytes (`m_extent_size`).
    #[must_use]
    pub const fn with_extent_size(mut self, value: u64) -> Self {
        self.extent_size = value;
        self
    }
    /// Initial tablespace size in bytes (`m_initial_size`).
    #[must_use]
    pub const fn with_initial_size(mut self, value: u64) -> Self {
        self.initial_size = value;
        self
    }
    /// Maximum tablespace size in bytes (`m_maximum_size`).
    #[must_use]
    pub const fn with_maximum_size(mut self, value: u64) -> Self {
        self.maximum_size = value;
        self
    }
    /// Auto-extend increment in bytes (`m_autoextend_size`).
    #[must_use]
    pub const fn with_autoextend_size(mut self, value: u64) -> Self {
        self.autoextend_size = value;
        self
    }
    /// Tablespace version — NDB only (`m_version`).
    #[must_use]
    pub const fn with_version(mut self, value: u64) -> Self {
        self.version = value;
        self
    }
    /// Reclaimable bytes — primarily InnoDB (`m_data_free`).
    #[must_use]
    pub const fn with_data_free(mut self, value: u64) -> Self {
        self.data_free = value;
        self
    }

    /// Tablespace identifier.
    pub const fn id(&self) -> u64 {
        self.id
    }
    /// Logfile group number — NDB only.
    pub const fn logfile_group_number(&self) -> u64 {
        self.logfile_group_number
    }
    /// Free-extent count.
    pub const fn free_extents(&self) -> u64 {
        self.free_extents
    }
    /// Total-extent count.
    pub const fn total_extents(&self) -> u64 {
        self.total_extents
    }
    /// Extent size in bytes.
    pub const fn extent_size(&self) -> u64 {
        self.extent_size
    }
    /// Initial size in bytes.
    pub const fn initial_size(&self) -> u64 {
        self.initial_size
    }
    /// Maximum size in bytes.
    pub const fn maximum_size(&self) -> u64 {
        self.maximum_size
    }
    /// Auto-extend increment in bytes.
    pub const fn autoextend_size(&self) -> u64 {
        self.autoextend_size
    }
    /// Tablespace version (NDB only).
    pub const fn version(&self) -> u64 {
        self.version
    }
    /// Reclaimable bytes (InnoDB).
    pub const fn data_free(&self) -> u64 {
        self.data_free
    }
}

impl Default for TablespaceStatistics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_zeros_every_field() {
        let s = TablespaceStatistics::new();
        assert_eq!(s.id(), 0);
        assert_eq!(s.data_free(), 0);
    }

    #[test]
    fn builder_preserves_each_setter() {
        let s = TablespaceStatistics::new()
            .with_id(7)
            .with_data_free(4096)
            .with_extent_size(64);
        assert_eq!(s.id(), 7);
        assert_eq!(s.data_free(), 4096);
        assert_eq!(s.extent_size(), 64);
    }
}
