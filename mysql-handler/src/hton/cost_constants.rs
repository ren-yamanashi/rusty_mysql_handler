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

//! Engine-specific optimizer cost constants.

/// Engine-specific cost constants the handlerton returns from
/// [`get_cost_constants`](crate::hton::Handlerton::get_cost_constants).
///
/// Both fields mirror the per-storage-engine values MySQL stores in the
/// `mysql.engine_cost` table; the defaults match what
/// `SE_cost_constants(Optimizer::kOriginal)` would have produced on the
/// C++ side, so an engine that supplies only one field starts from a sane
/// fallback for the other.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct CostConstants {
    memory_block_read_cost: f64,
    io_block_read_cost: f64,
}

impl CostConstants {
    /// Construct a cost-constants tuple with the given per-block read
    /// costs. Both values must be positive; MySQL's
    /// `SE_cost_constants::set` rejects zero or negative values with
    /// `INVALID_COST_VALUE`, so the shim does the same and an engine
    /// returning either value reverts to the defaults.
    #[must_use]
    pub const fn new(memory_block_read_cost: f64, io_block_read_cost: f64) -> Self {
        Self {
            memory_block_read_cost,
            io_block_read_cost,
        }
    }

    /// The defaults `SE_cost_constants(Optimizer::kOriginal)` initialises
    /// — `memory_block_read_cost = 0.25`, `io_block_read_cost = 1.0`.
    /// Engines that want to override one field commonly start from these.
    #[must_use]
    pub const fn defaults() -> Self {
        Self::new(0.25, 1.0)
    }

    /// Cost of reading one random block from an in-memory buffer.
    #[must_use]
    pub const fn memory_block_read_cost(&self) -> f64 {
        self.memory_block_read_cost
    }

    /// Cost of reading one random block from disk.
    #[must_use]
    pub const fn io_block_read_cost(&self) -> f64 {
        self.io_block_read_cost
    }
}

impl Default for CostConstants {
    fn default() -> Self {
        Self::defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_se_cost_constants_original() {
        // Values pinned to mysql-server's
        // `SE_cost_constants(Optimizer::kOriginal)` initialiser.
        let c = CostConstants::defaults();
        assert!((c.memory_block_read_cost() - 0.25).abs() < f64::EPSILON);
        assert!((c.io_block_read_cost() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn new_preserves_values() {
        let c = CostConstants::new(0.5, 2.5);
        assert!((c.memory_block_read_cost() - 0.5).abs() < f64::EPSILON);
        assert!((c.io_block_read_cost() - 2.5).abs() < f64::EPSILON);
    }
}
