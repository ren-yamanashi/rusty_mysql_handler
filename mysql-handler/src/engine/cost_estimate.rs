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

//! Cost-estimate value an engine reports for the optimizer's cost methods.

/// A cost estimate an engine reports to the optimizer, mirroring MySQL's
/// `Cost_estimate`.
///
/// The first three components are time-consuming costs the optimizer sums to
/// compare access paths: I/O, CPU, and import (remote) cost. The fourth,
/// memory cost, tracks bytes the engine expects to allocate and is kept apart
/// from the time-based total. All four use MySQL's internal cost units.
///
/// Returned as `Some` from [`table_scan_cost`], [`index_scan_cost`], and
/// [`read_cost`]; the shim assembles a `Cost_estimate` from the components.
/// This is distinct from the opaque [`crate::sys::CostEstimate`], which is a
/// borrowed handle to a live MySQL accumulator the binding cannot construct.
///
/// [`table_scan_cost`]: crate::engine::StorageEngine::table_scan_cost
/// [`index_scan_cost`]: crate::engine::StorageEngine::index_scan_cost
/// [`read_cost`]: crate::engine::StorageEngine::read_cost
///
/// # Examples
///
/// ```
/// use mysql_handler::engine::CostEstimate;
///
/// let cost = CostEstimate::new(10.0, 2.0, 0.0, 0.0);
/// assert_eq!(cost.io_cost(), 10.0);
/// assert_eq!(cost.cpu_cost(), 2.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct CostEstimate {
    io: f64,
    cpu: f64,
    import: f64,
    mem: f64,
}

impl CostEstimate {
    /// Build a cost estimate from its I/O, CPU, import, and memory components,
    /// each in MySQL's internal cost units.
    #[must_use]
    pub const fn new(io_cost: f64, cpu_cost: f64, import_cost: f64, mem_cost: f64) -> Self {
        Self {
            io: io_cost,
            cpu: cpu_cost,
            import: import_cost,
            mem: mem_cost,
        }
    }

    /// Cost of the I/O operations the access path performs.
    #[must_use]
    pub const fn io_cost(&self) -> f64 {
        self.io
    }

    /// Cost of the CPU work the access path performs.
    #[must_use]
    pub const fn cpu_cost(&self) -> f64 {
        self.cpu
    }

    /// Cost of remote (import) operations the access path performs.
    #[must_use]
    pub const fn import_cost(&self) -> f64 {
        self.import
    }

    /// Memory the access path expects to use, in bytes.
    #[must_use]
    pub const fn mem_cost(&self) -> f64 {
        self.mem
    }
}

#[cfg(test)]
mod tests {
    use super::CostEstimate;

    #[test]
    fn accessors_return_constructor_components() {
        let cost = CostEstimate::new(1.5, 2.5, 3.5, 4.5);
        assert_eq!(cost.io_cost().to_bits(), 1.5_f64.to_bits());
        assert_eq!(cost.cpu_cost().to_bits(), 2.5_f64.to_bits());
        assert_eq!(cost.import_cost().to_bits(), 3.5_f64.to_bits());
        assert_eq!(cost.mem_cost().to_bits(), 4.5_f64.to_bits());
    }
}
