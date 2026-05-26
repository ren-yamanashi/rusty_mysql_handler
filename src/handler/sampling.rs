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

//! `rust__handler__*` callbacks for sampling methods (handler.h #57–#59).
//! Shares the FFI safety contract documented at [`crate::handler`].

#![allow(unsafe_code)]

use core::ffi::c_void;

use crate::engine::{EngineError, SamplingMethod};
use crate::panic_guard::FfiBoundary;
use crate::runtime::{EngineContext, FfiPtr};

/// Initialize sampling, writing the engine scan context out
///
/// # Safety
/// `ctx` non-null; `scan_ctx` writable for one pointer when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__sample_init(
    ctx: *mut EngineContext,
    scan_ctx: *mut *mut c_void,
    sampling_percentage: f64,
    sampling_seed: i32,
    sampling_method: i32,
    tablesample: bool,
) -> i32 {
    FfiBoundary::run_default(EngineError::Internal.to_mysql_errno(), || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        let method = SamplingMethod::from_raw(sampling_method);
        match engine.sample_init(sampling_percentage, sampling_seed, method, tablesample) {
            Ok(engine_ctx) => {
                if !scan_ctx.is_null() {
                    // SAFETY: scan_ctx is writable for one pointer when non-null.
                    unsafe { *scan_ctx = engine_ctx };
                }
                0
            }
            Err(e) => e.to_mysql_errno(),
        }
    })
}

/// Read the next sampled row into `buf`
///
/// # Safety
/// `ctx` non-null; `buf` writable for `buf_len`. `scan_ctx` is the engine's own
/// pointer, passed through without dereference.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__sample_next(
    ctx: *mut EngineContext,
    scan_ctx: *mut c_void,
    buf: *mut u8,
    buf_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees buf covers buf_len writable bytes.
        let buf = unsafe { FfiPtr::slice_mut(buf, buf_len) };
        engine.sample_next(scan_ctx, buf)
    })
}

/// End sampling and release the engine context
///
/// # Safety
/// `ctx` non-null. `scan_ctx` is the engine's own pointer, passed through
/// without dereference.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__sample_end(
    ctx: *mut EngineContext,
    scan_ctx: *mut c_void,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().sample_end(scan_ctx)
    })
}
