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

//! `rust__handler__records_in_range` callback (handler.h #32). Shares the FFI
//! safety contract documented at [`crate::handler`].

#![allow(unsafe_code)]

use crate::engine::{RKeyFunction, RangeKey};
use crate::panic_guard::FfiBoundary;
use crate::runtime::{EngineContext, FfiPtr};

const HA_POS_ERROR: u64 = u64::MAX;

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

/// Estimate the row count on index `inx`; returns `HA_POS_ERROR` when unknown
///
/// # Safety
/// `ctx` non-null; each non-null range key readable for its length.
#[doc(hidden)]
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rust__handler__records_in_range(
    ctx: *mut EngineContext,
    inx: u32,
    min_key: *const u8,
    min_len: usize,
    min_flag: i32,
    max_key: *const u8,
    max_len: usize,
    max_flag: i32,
) -> u64 {
    FfiBoundary::run_default(HA_POS_ERROR, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: caller guarantees min_key covers min_len readable bytes.
        let min = unsafe { range_key(min_key, min_len, min_flag) };
        // SAFETY: caller guarantees max_key covers max_len readable bytes.
        let max = unsafe { range_key(max_key, max_len, max_flag) };
        let rows = match engine.as_indexed() {
            Some(indexed) => indexed.records_in_range(inx, min, max),
            None => None,
        };
        match rows {
            Some(rows) => rows,
            None => HA_POS_ERROR,
        }
    })
}
