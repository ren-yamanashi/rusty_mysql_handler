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

//! `rust__handler__*` callbacks for multi-range read (MRR) methods (handler.h
//! #64–#67). Shares the FFI safety contract documented at [`crate::handler`].
//!
//! Each callback returns `true` when the engine handled the call (its result is
//! written through the out-pointer) and `false` to fall back to the C++ base
//! disk-sweep MRR implementation. The trait defaults return the fall-back, so
//! an engine relying on the base path is driven through its own
//! `read_range_first` / `read_range_next`.

#![allow(unsafe_code)]

use core::ffi::c_void;

use crate::panic_guard::FfiBoundary;
use crate::runtime::{EngineContext, FfiPtr};
use crate::sys;

// Map an engine-provided MRR result to the MySQL return code the shim returns
fn result_code(result: crate::engine::EngineResult) -> i32 {
    match result {
        Ok(()) => 0,
        Err(e) => e.to_mysql_errno(),
    }
}

/// Estimate const-range MRR cost; returns whether the engine handled it
///
/// # Safety
/// `ctx` non-null; `seq` / `cost` null-or-valid for the call; `out_rows`
/// writable for one `u64` when non-null. `seq_init_param` is round-tripped
/// without dereference.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__multi_range_read_info_const(
    ctx: *mut EngineContext,
    keyno: u32,
    seq: *const sys::RangeSeqIf,
    seq_init_param: *mut c_void,
    n_ranges: u32,
    cost: *const sys::CostEstimate,
    out_rows: *mut u64,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: seq is null or valid for read for the call's duration.
        let seq = unsafe { seq.as_ref() };
        // SAFETY: cost is null or valid for read for the call's duration.
        let cost = unsafe { cost.as_ref() };
        let outcome = match engine.as_indexed() {
            Some(indexed) => {
                indexed.multi_range_read_info_const(keyno, seq, seq_init_param, n_ranges, cost)
            }
            None => None,
        };
        match outcome {
            Some(rows) => {
                if !out_rows.is_null() {
                    // SAFETY: out_rows is writable for one u64 when non-null.
                    unsafe { *out_rows = rows };
                }
                true
            }
            None => false,
        }
    })
}

/// Estimate MRR cost over a row span; returns whether the engine handled it
///
/// # Safety
/// `ctx` non-null; `cost` null-or-valid for the call; `out_rows` writable for
/// one `u64` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__multi_range_read_info(
    ctx: *mut EngineContext,
    keyno: u32,
    n_ranges: u32,
    keys: u32,
    cost: *const sys::CostEstimate,
    out_rows: *mut u64,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: cost is null or valid for read for the call's duration.
        let cost = unsafe { cost.as_ref() };
        let outcome = match engine.as_indexed() {
            Some(indexed) => indexed.multi_range_read_info(keyno, n_ranges, keys, cost),
            None => None,
        };
        match outcome {
            Some(rows) => {
                if !out_rows.is_null() {
                    // SAFETY: out_rows is writable for one u64 when non-null.
                    unsafe { *out_rows = rows };
                }
                true
            }
            None => false,
        }
    })
}

/// Initialize an MRR scan; returns whether the engine handled it
///
/// # Safety
/// `ctx` non-null; `seq` / `buf` null-or-valid for the call; `out_result`
/// writable for one `i32` when non-null. `seq_init_param` is round-tripped
/// without dereference.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__multi_range_read_init(
    ctx: *mut EngineContext,
    seq: *const sys::RangeSeqIf,
    seq_init_param: *mut c_void,
    n_ranges: u32,
    mode: u32,
    buf: *const sys::HandlerBuffer,
    out_result: *mut i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: seq is null or valid for read for the call's duration.
        let seq = unsafe { seq.as_ref() };
        // SAFETY: buf is null or valid for read for the call's duration.
        let buf = unsafe { buf.as_ref() };
        let outcome = match engine.as_indexed() {
            Some(indexed) => {
                indexed.multi_range_read_init(seq, seq_init_param, n_ranges, mode, buf)
            }
            None => None,
        };
        match outcome {
            Some(result) => {
                if !out_result.is_null() {
                    // SAFETY: out_result is writable for one i32 when non-null.
                    unsafe { *out_result = result_code(result) };
                }
                true
            }
            None => false,
        }
    })
}

/// Read the next MRR row; returns whether the engine handled it
///
/// # Safety
/// `ctx` non-null; `buf` writable for `buf_len`; `out_result` writable for one
/// `i32` when non-null. `range_info` is round-tripped without dereference.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__multi_range_read_next(
    ctx: *mut EngineContext,
    buf: *mut u8,
    buf_len: usize,
    range_info: *mut *mut c_void,
    out_result: *mut i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees buf covers buf_len writable bytes.
        let buf = unsafe { FfiPtr::slice_mut(buf, buf_len) };
        let outcome = match engine.as_indexed() {
            Some(indexed) => indexed.multi_range_read_next(buf, range_info),
            None => None,
        };
        match outcome {
            Some(result) => {
                if !out_result.is_null() {
                    // SAFETY: out_result is writable for one i32 when non-null.
                    unsafe { *out_result = result_code(result) };
                }
                true
            }
            None => false,
        }
    })
}
