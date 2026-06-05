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

//! `rust__handler__*` callbacks for pushed-join index methods (handler.h
//! #33–#34). Shares the FFI safety contract documented at [`crate::handler`].

#![allow(unsafe_code)]

use crate::panic_guard::FfiBoundary;
use crate::runtime::{EngineContext, FfiPtr};

/// Read the keyed root row of a pushed join
///
/// # Safety
/// `ctx` non-null; `buf` writable for `buf_len`; `key` readable for `key_len`
/// when non-null. A null `key` denotes "begin at the first key" and is passed
/// to the engine as an empty slice.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__index_read_pushed(
    ctx: *mut EngineContext,
    buf: *mut u8,
    buf_len: usize,
    key: *const u8,
    key_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees buf covers buf_len writable bytes.
        let buf = unsafe { FfiPtr::slice_mut(buf, buf_len) };
        let key = if key.is_null() {
            &[][..]
        } else {
            // SAFETY: caller guarantees key covers key_len readable bytes.
            unsafe { FfiPtr::slice_const(key, key_len) }
        };
        engine.index_read_pushed(buf, key)
    })
}

/// Read the next row of a pushed join
///
/// # Safety
/// `ctx` non-null; `buf` writable for `buf_len`.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__index_next_pushed(
    ctx: *mut EngineContext,
    buf: *mut u8,
    buf_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees buf covers buf_len writable bytes.
        engine.index_next_pushed(unsafe { FfiPtr::slice_mut(buf, buf_len) })
    })
}
