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

//! `rust__handler__*` callbacks for create-info and metadata methods (handler.h
//! #149–#153). Shares the FFI safety contract documented at [`crate::handler`].
//! The remaining specialised methods live in [`crate::handler::misc`].

#![allow(unsafe_code)]

use core::ffi::c_void;

use super::report::{report_bool, report_i32};
use crate::panic_guard::FfiBoundary;
use crate::runtime::{EngineContext, FfiPtr};
use crate::sys;

unsafe extern "C" {
    // Defined by the shim: appends len bytes to the MySQL `String` packet so
    // append_create_info can add engine-specific CREATE TABLE text.
    fn mysql__mysql_string__append(packet: *mut c_void, bytes: *const u8, len: usize);
}

/// Notify the engine to populate create-info before `SHOW CREATE TABLE`
///
/// # Safety
/// `ctx` non-null; `create_info` null-or-valid for the call.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__update_create_info(
    ctx: *mut EngineContext,
    create_info: *const sys::HA_CREATE_INFO,
) {
    FfiBoundary::run_void(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: create_info is null or valid for read for the call.
        engine.update_create_info(unsafe { create_info.as_ref() });
    });
}

/// Append engine-specific text to the `CREATE TABLE` statement
///
/// # Safety
/// `ctx` non-null; `packet` a valid MySQL `String *` for the call.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__append_create_info(
    ctx: *mut EngineContext,
    packet: *mut c_void,
) {
    FfiBoundary::run_void(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        if let Some(text) = unsafe { &mut *ctx }.engine_mut().append_create_info() {
            // SAFETY: packet is a valid String* for the call; bytes/len describe
            // text's buffer, appended by the shim before returning.
            unsafe { mysql__mysql_string__append(packet, text.as_ptr(), text.len()) };
        }
    });
}

/// Notify the engine that hidden-primary-key positioning is in use
///
/// # Safety
/// `ctx` non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__use_hidden_primary_key(ctx: *mut EngineContext) {
    FfiBoundary::run_void(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().use_hidden_primary_key();
    });
}

/// Adopt the shared `Handler_share`; returns whether the engine handled it
///
/// # Safety
/// `ctx` non-null; `arg` round-tripped without dereference; `out` writable for
/// one `bool` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__set_ha_share_ref(
    ctx: *mut EngineContext,
    arg: *mut c_void,
    out: *mut bool,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        report_bool(out, unsafe { &mut *ctx }.engine_mut().set_ha_share_ref(arg))
    })
}

/// Compare two row-position refs; returns whether the engine handled it
///
/// # Safety
/// `ctx` non-null; `ref1`/`ref2` readable for `len` bytes; `out` writable for
/// one `i32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__cmp_ref(
    ctx: *mut EngineContext,
    ref1: *const u8,
    ref2: *const u8,
    len: usize,
    out: *mut i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: ref1/ref2 are readable for `len` bytes for the call.
        let (a, b) = unsafe {
            (
                FfiPtr::slice_const(ref1, len),
                FfiPtr::slice_const(ref2, len),
            )
        };
        report_i32(out, engine.cmp_ref(a, b))
    })
}
