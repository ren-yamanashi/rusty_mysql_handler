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

//! `rust__handler__*` callbacks for the remaining specialised methods —
//! secondary-engine offload, clone, multi-valued-index capacity, and
//! partitioning (handler.h #154–#158). Shares the FFI safety contract
//! documented at [`crate::handler`]. The create-info / metadata methods live in
//! [`crate::handler::metadata`].

#![allow(unsafe_code)]

use core::ffi::c_void;

use crate::panic_guard::FfiBoundary;
use crate::runtime::{EngineContext, FfiPtr};

// Write an engine-supplied multi-valued-index capacity through the out-pointers;
// report whether the engine supplied one.
fn report_mv_capacity(
    num_keys: *mut u32,
    keys_length: *mut u64,
    value: Option<(u32, u64)>,
) -> bool {
    match value {
        Some((keys, bytes)) => {
            // SAFETY: each out-pointer is writable for one value when non-null.
            unsafe {
                if !num_keys.is_null() {
                    *num_keys = keys;
                }
                if !keys_length.is_null() {
                    *keys_length = bytes;
                }
            }
            true
        }
        None => false,
    }
}

/// Record the secondary-engine offload error message
///
/// # Safety
/// `ctx` non-null; `reason` readable for `len` bytes.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__set_external_table_offload_error(
    ctx: *mut EngineContext,
    reason: *const u8,
    len: usize,
) {
    FfiBoundary::run_void(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: reason is readable for `len` bytes for the call.
        if let Ok(reason) = unsafe { FfiPtr::bytes_to_str(reason, len) } {
            engine.set_external_table_offload_error(reason);
        }
    });
}

/// Raise the recorded secondary-engine offload error
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__external_table_offload_error(ctx: *mut EngineContext) {
    FfiBoundary::run_void(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }
            .engine_mut()
            .external_table_offload_error();
    });
}

/// Clone the handler; returns an opaque `handler *` or null to use the base
///
/// # Safety
/// `ctx` non-null; `name` readable for `len` bytes; `mem_root` round-tripped.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__clone(
    ctx: *mut EngineContext,
    name: *const u8,
    len: usize,
    mem_root: *mut c_void,
) -> *mut c_void {
    FfiBoundary::run_default(core::ptr::null_mut(), || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: name is readable for `len` bytes for the call.
        match unsafe { FfiPtr::bytes_to_str(name, len) } {
            Ok(name) => engine.clone_handler(name, mem_root),
            Err(_) => core::ptr::null_mut(),
        }
    })
}

/// Report multi-valued-index capacity; returns whether the engine overrode it
///
/// # Safety
/// `ctx` non-null; `num_keys`/`keys_length` writable for one value each when
/// non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__mv_key_capacity(
    ctx: *mut EngineContext,
    num_keys: *mut u32,
    keys_length: *mut u64,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let value = unsafe { &mut *ctx }.engine_mut().mv_key_capacity();
        report_mv_capacity(num_keys, keys_length, value)
    })
}

/// The engine's partition handler (opaque, may be null)
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__get_partition_handler(
    ctx: *mut EngineContext,
) -> *mut c_void {
    FfiBoundary::run_default(core::ptr::null_mut(), || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().get_partition_handler()
    })
}

#[cfg(test)]
mod tests {
    use super::report_mv_capacity;

    #[test]
    fn report_mv_capacity_writes_pair_and_signals_handled() {
        let (mut n, mut b) = (0, 0);
        assert!(report_mv_capacity(&mut n, &mut b, Some((4, 1024))));
        assert_eq!((n, b), (4, 1024));
    }

    #[test]
    fn report_mv_capacity_none_leaves_buffers_and_signals_unhandled() {
        let (mut n, mut b) = (7, 9);
        assert!(!report_mv_capacity(&mut n, &mut b, None));
        assert_eq!((n, b), (7, 9));
    }

    #[test]
    fn report_mv_capacity_tolerates_null_out_pointers() {
        assert!(report_mv_capacity(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            Some((1, 1))
        ));
    }
}
