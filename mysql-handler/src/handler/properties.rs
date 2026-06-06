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

//! `rust__handler__*` callbacks for engine-property methods (table_type,
//! table_flags, index_flags). Shares the FFI safety contract documented at
//! [`crate::handler`].

#![allow(unsafe_code)]

use std::ffi::c_char;

use crate::panic_guard::FfiBoundary;
use crate::runtime::EngineContext;

/// Engine display name; null-terminated `'static`
///
/// # Safety
/// `ctx` must be non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__table_type(ctx: *mut EngineContext) -> *const c_char {
    FfiBoundary::run_default(std::ptr::null(), || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().table_type().as_ptr()
    })
}

/// `HA_*` capability bitfield
///
/// # Safety
/// `ctx` must be non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__table_flags(ctx: *mut EngineContext) -> u64 {
    FfiBoundary::run_default(0, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }.engine_mut().table_flags()
    })
}

/// Per-index capability bitfield
///
/// # Safety
/// `ctx` must be non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__index_flags(
    ctx: *mut EngineContext,
    idx: u32,
    part: u32,
    all_parts: bool,
) -> u32 {
    FfiBoundary::run_default(0, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        unsafe { &mut *ctx }
            .engine_mut()
            .index_flags(idx, part, all_parts)
    })
}
