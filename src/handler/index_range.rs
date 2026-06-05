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

//! `rust__handler__*` callbacks for index read and range-scan methods
//! (handler.h #20, #22–#24, #30, #31). `records_in_range` (#32) lives in
//! [`crate::handler::index_records`]. Shares the FFI safety contract
//! documented at [`crate::handler`].

#![allow(unsafe_code)]

use crate::engine::{EngineError, RKeyFunction, RangeKey};
use crate::panic_guard::FfiBoundary;
use crate::runtime::{EngineContext, FfiPtr};

/// Resolve a possibly-null key pointer to the slice the trait expects; a null
/// pointer ("begin at the first key") becomes an empty slice
///
/// # Safety
/// When non-null, `key` must be readable for `key_len` bytes for `'a`.
unsafe fn key_slice<'a>(key: *const u8, key_len: usize) -> &'a [u8] {
    if key.is_null() {
        &[][..]
    } else {
        // SAFETY: caller guarantees key covers key_len readable bytes.
        unsafe { FfiPtr::slice_const(key, key_len) }
    }
}

/// Rebuild one range endpoint; a null pointer denotes an open-ended bound
///
/// # Safety
/// When non-null, `key` must be readable for `key_len` bytes for `'a`.
unsafe fn range_key<'a>(key: *const u8, key_len: usize, flag: i32) -> Option<RangeKey<'a>> {
    if key.is_null() {
        None
    } else {
        // SAFETY: caller guarantees key covers key_len readable bytes.
        let bytes = unsafe { FfiPtr::slice_const(key, key_len) };
        Some(RangeKey::new(bytes, RKeyFunction::from_raw(flag)))
    }
}

/// Position the cursor at `key` per `find_flag` (explicit key length) and read
/// the matching row
///
/// # Safety
/// `ctx` non-null; `buf` writable for `buf_len`; `key` readable for `key_len`
/// when non-null (a null `key` is passed to the engine as an empty slice).
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__index_read(
    ctx: *mut EngineContext,
    buf: *mut u8,
    buf_len: usize,
    key: *const u8,
    key_len: usize,
    find_flag: i32,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees buf covers buf_len writable bytes.
        let buf = unsafe { FfiPtr::slice_mut(buf, buf_len) };
        // SAFETY: caller guarantees key covers key_len readable bytes.
        let key = unsafe { key_slice(key, key_len) };
        match engine.as_indexed() {
            Some(indexed) => indexed.index_read(buf, key, RKeyFunction::from_raw(find_flag)),
            None => Err(EngineError::WrongCommand),
        }
    })
}

/// Read from index `index` at `key` per `find_flag`
///
/// # Safety
/// `ctx` non-null; `buf` writable for `buf_len`; `key` readable for `key_len`
/// when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__index_read_idx_map(
    ctx: *mut EngineContext,
    buf: *mut u8,
    buf_len: usize,
    index: u32,
    key: *const u8,
    key_len: usize,
    find_flag: i32,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees buf covers buf_len writable bytes.
        let buf = unsafe { FfiPtr::slice_mut(buf, buf_len) };
        // SAFETY: caller guarantees key covers key_len readable bytes.
        let key = unsafe { key_slice(key, key_len) };
        match engine.as_indexed() {
            Some(indexed) => {
                indexed.index_read_idx_map(buf, index, key, RKeyFunction::from_raw(find_flag))
            }
            None => Err(EngineError::WrongCommand),
        }
    })
}

/// Read the last row matching `key` (explicit key length)
///
/// # Safety
/// `ctx` non-null; `buf` writable for `buf_len`; `key` readable for `key_len`
/// when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__index_read_last(
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
        // SAFETY: caller guarantees key covers key_len readable bytes.
        let key = unsafe { key_slice(key, key_len) };
        match engine.as_indexed() {
            Some(indexed) => indexed.index_read_last(buf, key),
            None => Err(EngineError::WrongCommand),
        }
    })
}

/// Read the last row matching `key` (key length resolved from `key_part_map`)
///
/// # Safety
/// `ctx` non-null; `buf` writable for `buf_len`; `key` readable for `key_len`
/// when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__index_read_last_map(
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
        // SAFETY: caller guarantees key covers key_len readable bytes.
        let key = unsafe { key_slice(key, key_len) };
        match engine.as_indexed() {
            Some(indexed) => indexed.index_read_last_map(buf, key),
            None => Err(EngineError::WrongCommand),
        }
    })
}

/// Begin a range scan and read its first row
///
/// # Safety
/// `ctx` non-null; `buf` writable for `buf_len`; each non-null range key
/// readable for its length.
#[doc(hidden)]
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rust__handler__read_range_first(
    ctx: *mut EngineContext,
    buf: *mut u8,
    buf_len: usize,
    start_key: *const u8,
    start_len: usize,
    start_flag: i32,
    end_key: *const u8,
    end_len: usize,
    end_flag: i32,
    eq_range: bool,
    sorted: bool,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees buf covers buf_len writable bytes.
        let buf = unsafe { FfiPtr::slice_mut(buf, buf_len) };
        // SAFETY: caller guarantees start_key covers start_len readable bytes.
        let start = unsafe { range_key(start_key, start_len, start_flag) };
        // SAFETY: caller guarantees end_key covers end_len readable bytes.
        let end = unsafe { range_key(end_key, end_len, end_flag) };
        match engine.as_indexed() {
            Some(indexed) => indexed.read_range_first(buf, start, end, eq_range, sorted),
            None => Err(EngineError::WrongCommand),
        }
    })
}

/// Read the next row of the current range scan
///
/// # Safety
/// `ctx` non-null; `buf` writable for `buf_len`.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__read_range_next(
    ctx: *mut EngineContext,
    buf: *mut u8,
    buf_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees buf covers buf_len writable bytes.
        let buf = unsafe { FfiPtr::slice_mut(buf, buf_len) };
        match engine.as_indexed() {
            Some(indexed) => indexed.read_range_next(buf),
            None => Err(EngineError::WrongCommand),
        }
    })
}
