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

//! `rust__handler__*` callbacks for parallel-scan methods (handler.h #54–#56).
//! Shares the FFI safety contract documented at [`crate::handler`].

#![allow(unsafe_code)]

use core::ffi::c_void;

use crate::engine::EngineError;
use crate::panic_guard::FfiBoundary;
use crate::runtime::EngineContext;

/// Initialize a parallel scan, writing the scan context and thread count out
///
/// # Safety
/// `ctx` non-null; `scan_ctx` writable for one pointer and `num_threads`
/// writable for one `usize` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__parallel_scan_init(
    ctx: *mut EngineContext,
    scan_ctx: *mut *mut c_void,
    num_threads: *mut usize,
    use_reserved_threads: bool,
    max_desired_threads: usize,
) -> i32 {
    FfiBoundary::run_default(EngineError::Internal.to_mysql_errno(), || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        match engine.parallel_scan_init(use_reserved_threads, max_desired_threads) {
            Ok(init) => {
                if !scan_ctx.is_null() {
                    // SAFETY: scan_ctx is writable for one pointer when non-null.
                    unsafe { *scan_ctx = init.scan_ctx() };
                }
                if !num_threads.is_null() {
                    // SAFETY: num_threads is writable for one usize when non-null.
                    unsafe { *num_threads = init.num_threads() };
                }
                0
            }
            Err(e) => e.to_mysql_errno(),
        }
    })
}

/// Run the parallel read; load callbacks are opaque MySQL `std::function`s
///
/// # Safety
/// `ctx` non-null. `scan_ctx` / `thread_ctxs` / `init_fn` / `load_fn` /
/// `end_fn` are passed through without dereference.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__parallel_scan(
    ctx: *mut EngineContext,
    scan_ctx: *mut c_void,
    thread_ctxs: *mut *mut c_void,
    init_fn: *const c_void,
    load_fn: *const c_void,
    end_fn: *const c_void,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        engine.parallel_scan(scan_ctx, thread_ctxs, init_fn, load_fn, end_fn)
    })
}

/// End the parallel scan and release the engine context
///
/// # Safety
/// `ctx` non-null. `scan_ctx` is the engine's own pointer, passed through
/// without dereference.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__parallel_scan_end(
    ctx: *mut EngineContext,
    scan_ctx: *mut c_void,
) {
    FfiBoundary::run_void(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }
            .engine_mut()
            .parallel_scan_end(scan_ctx);
    });
}
