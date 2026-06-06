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

//! Raw-pointer helpers that turn shim-supplied pointers into bounded references.

#![allow(unsafe_code)]

use std::slice;

use crate::engine::{EngineError, EngineResult};

/// Raw-pointer helpers that turn shim-supplied pointers into bounded references
#[derive(Debug)]
pub(crate) struct FfiPtr;

impl FfiPtr {
    /// Decode `len` bytes at `p` as a UTF-8 `&str`; length is caller-measured
    /// so this side performs no `strlen`-style scan.
    ///
    /// # Safety
    /// `p` must be non-null, aligned, and readable for `len` bytes for the
    /// returned reference's lifetime.
    pub(crate) unsafe fn bytes_to_str<'a>(p: *const u8, len: usize) -> EngineResult<&'a str> {
        // SAFETY: caller guarantees `p` covers `len` readable bytes;
        // `from_raw_parts` requires non-null even when `len == 0`.
        let bytes = unsafe { slice::from_raw_parts(p, len) };
        match core::str::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(_) => Err(EngineError::InvalidName),
        }
    }

    /// View `len` writable bytes at `p` as `&mut [u8]`
    ///
    /// # Safety
    /// `p` must be non-null, aligned, and writable for `len` bytes for the
    /// returned reference's lifetime.
    pub(crate) unsafe fn slice_mut<'a>(p: *mut u8, len: usize) -> &'a mut [u8] {
        // SAFETY: caller guarantees `p` covers `len` writable bytes;
        // `from_raw_parts_mut` requires non-null even when `len == 0`.
        unsafe { slice::from_raw_parts_mut(p, len) }
    }

    /// View `len` readable bytes at `p` as `&[u8]`
    ///
    /// # Safety
    /// `p` must be non-null, aligned, and readable for `len` bytes for the
    /// returned reference's lifetime.
    pub(crate) unsafe fn slice_const<'a>(p: *const u8, len: usize) -> &'a [u8] {
        // SAFETY: caller guarantees `p` covers `len` readable bytes;
        // `from_raw_parts` requires non-null even when `len == 0`.
        unsafe { slice::from_raw_parts(p, len) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_to_str_decodes_valid_utf8() {
        let bytes = b"rusty";
        // SAFETY: bytes is non-null and covers its length for the call.
        let decoded = unsafe { FfiPtr::bytes_to_str(bytes.as_ptr(), bytes.len()) };
        assert_eq!(decoded, Ok("rusty"));
    }

    #[test]
    fn bytes_to_str_rejects_invalid_utf8() {
        let bytes = [0xff_u8, 0xfe];
        // SAFETY: bytes is non-null and covers its length for the call.
        let decoded = unsafe { FfiPtr::bytes_to_str(bytes.as_ptr(), bytes.len()) };
        assert_eq!(decoded, Err(EngineError::InvalidName));
    }

    #[test]
    fn slice_const_views_the_bytes() {
        let data = [10u8, 20, 30];
        // SAFETY: data is non-null and covers 3 readable bytes for the call.
        let view = unsafe { FfiPtr::slice_const(data.as_ptr(), data.len()) };
        assert_eq!(view, &data);
    }

    #[test]
    fn slice_mut_allows_write_back() {
        let mut data = [0u8; 3];
        // SAFETY: data is non-null and covers 3 writable bytes for the call.
        let view = unsafe { FfiPtr::slice_mut(data.as_mut_ptr(), data.len()) };
        view[1] = 42;
        assert_eq!(data, [0, 42, 0]);
    }
}
