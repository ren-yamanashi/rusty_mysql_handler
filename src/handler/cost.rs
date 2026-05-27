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

//! `rust__handler__*` callbacks for the `Cost_estimate`-returning cost methods
//! (handler.h #94–#96). The scalar time methods live in
//! [`crate::handler::cost_time`]. Shares the FFI safety contract documented at
//! [`crate::handler`].
//!
//! Each callback returns `true` when the engine overrides, decomposing the
//! engine's [`CostEstimate`] into io/cpu/import/mem out-pointers the shim
//! reassembles, or `false` to fall back to the handler base.

#![allow(unsafe_code)]

use crate::engine::CostEstimate;
use crate::panic_guard::FfiBoundary;
use crate::runtime::EngineContext;

// Decompose an engine-supplied CostEstimate into the four out-pointers the shim
// assembles into a Cost_estimate; report whether the engine supplied one.
fn report_cost(
    io: *mut f64,
    cpu: *mut f64,
    import: *mut f64,
    mem: *mut f64,
    value: Option<CostEstimate>,
) -> bool {
    match value {
        Some(c) => {
            // SAFETY: each out-pointer is writable for one f64 when non-null.
            unsafe {
                if !io.is_null() {
                    *io = c.io_cost();
                }
                if !cpu.is_null() {
                    *cpu = c.cpu_cost();
                }
                if !import.is_null() {
                    *import = c.import_cost();
                }
                if !mem.is_null() {
                    *mem = c.mem_cost();
                }
            }
            true
        }
        None => false,
    }
}

/// Full-table-scan cost estimate
///
/// # Safety
/// `ctx` non-null; each out-pointer writable for one `f64` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__table_scan_cost(
    ctx: *mut EngineContext,
    io: *mut f64,
    cpu: *mut f64,
    import: *mut f64,
    mem: *mut f64,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let value = unsafe { &mut *ctx }.engine_mut().table_scan_cost();
        report_cost(io, cpu, import, mem, value)
    })
}

/// Index-only scan cost estimate
///
/// # Safety
/// `ctx` non-null; each out-pointer writable for one `f64` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__index_scan_cost(
    ctx: *mut EngineContext,
    index: u32,
    ranges: f64,
    rows: f64,
    io: *mut f64,
    cpu: *mut f64,
    import: *mut f64,
    mem: *mut f64,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let value = unsafe { &mut *ctx }
            .engine_mut()
            .index_scan_cost(index, ranges, rows);
        report_cost(io, cpu, import, mem, value)
    })
}

/// Index range-read cost estimate
///
/// # Safety
/// `ctx` non-null; each out-pointer writable for one `f64` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__read_cost(
    ctx: *mut EngineContext,
    index: u32,
    ranges: f64,
    rows: f64,
    io: *mut f64,
    cpu: *mut f64,
    import: *mut f64,
    mem: *mut f64,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let value = unsafe { &mut *ctx }
            .engine_mut()
            .read_cost(index, ranges, rows);
        report_cost(io, cpu, import, mem, value)
    })
}

#[cfg(test)]
mod tests {
    use super::report_cost;
    use crate::engine::CostEstimate;

    #[test]
    fn report_cost_writes_components_and_signals_handled() {
        let (mut io, mut cpu, mut import, mut mem) = (0.0, 0.0, 0.0, 0.0);
        let handled = report_cost(
            &mut io,
            &mut cpu,
            &mut import,
            &mut mem,
            Some(CostEstimate::new(1.0, 2.0, 3.0, 4.0)),
        );
        assert!(handled);
        assert_eq!((io, cpu, import, mem), (1.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn report_cost_none_leaves_buffers_and_signals_unhandled() {
        let (mut io, mut cpu, mut import, mut mem) = (9.0, 9.0, 9.0, 9.0);
        let handled = report_cost(&mut io, &mut cpu, &mut import, &mut mem, None);
        assert!(!handled);
        assert_eq!((io, cpu, import, mem), (9.0, 9.0, 9.0, 9.0));
    }

    #[test]
    fn report_cost_tolerates_null_out_pointers() {
        let null = core::ptr::null_mut();
        assert!(report_cost(
            null,
            null,
            null,
            null,
            Some(CostEstimate::new(1.0, 2.0, 3.0, 4.0)),
        ));
    }
}
