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

//! Outcome of [`StorageEngine::parallel_scan_init`].

use core::ffi::c_void;

/// The two outputs of [`StorageEngine::parallel_scan_init`]: the engine-owned
/// scan context that every later `parallel_scan*` call receives back, and the
/// number of worker threads the engine will drive.
///
/// The context pointer is round-tripped through MySQL verbatim; the binding
/// never dereferences it and the engine owns its lifetime (freed in
/// [`StorageEngine::parallel_scan_end`]).
///
/// [`StorageEngine::parallel_scan_init`]: crate::engine::StorageEngine::parallel_scan_init
/// [`StorageEngine::parallel_scan_end`]: crate::engine::StorageEngine::parallel_scan_end
#[derive(Debug)]
#[non_exhaustive]
pub struct ParallelScanInit {
    scan_ctx: *mut c_void,
    num_threads: usize,
}

impl ParallelScanInit {
    /// Build the outcome from the engine's scan context and worker-thread count.
    /// A null `scan_ctx` with `num_threads == 0` signals "no parallel scan".
    #[must_use]
    pub fn new(scan_ctx: *mut c_void, num_threads: usize) -> Self {
        Self {
            scan_ctx,
            num_threads,
        }
    }

    /// Engine-owned scan-context pointer handed back to MySQL
    #[must_use]
    pub fn scan_ctx(&self) -> *mut c_void {
        self.scan_ctx
    }

    /// Number of worker threads the engine will drive for the scan
    #[must_use]
    pub fn num_threads(&self) -> usize {
        self.num_threads
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_context_and_thread_count() {
        let mut sentinel = 0u8;
        let ptr: *mut c_void = (&raw mut sentinel).cast();
        let init = ParallelScanInit::new(ptr, 4);
        assert_eq!(init.scan_ctx(), ptr);
        assert_eq!(init.num_threads(), 4);
    }

    #[test]
    fn null_context_signals_no_parallel_scan() {
        let init = ParallelScanInit::new(core::ptr::null_mut(), 0);
        assert!(init.scan_ctx().is_null());
        assert_eq!(init.num_threads(), 0);
    }
}
