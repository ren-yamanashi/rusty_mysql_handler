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

//! `rust__handler__*` callbacks for table-instance lifecycle (handler.h #1–#3):
//! create, open, close. Shares the FFI safety contract documented at
//! [`crate::ffi_handler`].

#![allow(unsafe_code)]

use crate::ffi::{EngineContext, FfiPtr};
use crate::panic_guard::FfiBoundary;

/// Create a new table
///
/// # Safety
/// `ctx` must be non-null; `name` must cover `name_len` readable bytes.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__create(
    ctx: *mut EngineContext,
    name: *const u8,
    name_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees name covers name_len readable bytes.
        let name = unsafe { FfiPtr::bytes_to_str(name, name_len) }?;
        engine.create(name)
    })
}

/// Open an existing table
///
/// # Safety
/// `ctx` must be non-null; `name` must cover `name_len` readable bytes.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__open(
    ctx: *mut EngineContext,
    name: *const u8,
    name_len: usize,
    mode: i32,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees name covers name_len readable bytes.
        let name = unsafe { FfiPtr::bytes_to_str(name, name_len) }?;
        engine.open(name, mode)
    })
}

/// Close the table
///
/// # Safety
/// `ctx` must be non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__close(ctx: *mut EngineContext) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().close()
    })
}
