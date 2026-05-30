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

//! `rust__hton__sdi_*` callbacks. Wired only under
//! [`HtonCapabilities::SDI`]; declared by engines that own their SDI store
//! (InnoDB-style). Each MySQL typedef returns `bool` with the "true = error"
//! convention; `result_to_error` performs the conversion and `run_default(true,
//! ...)` reports failure if a panic crosses the boundary.
//!
//! [`HtonCapabilities::SDI`]: crate::hton::HtonCapabilities::SDI

#![allow(unsafe_code)]

use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::runtime::FfiPtr;
use crate::sys;

fn result_to_error(r: crate::engine::EngineResult) -> bool {
    match r {
        Ok(()) => false,
        Err(_) => true,
    }
}

/// # Safety
/// `tablespace` is null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__sdi_create(tablespace: *const sys::DdTablespace) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: tablespace null or valid for read for this call.
        let ts = unsafe { tablespace.as_ref() };
        match runtime::handlerton() {
            Some(h) => result_to_error(h.sdi_create(ts)),
            None => true,
        }
    })
}

/// # Safety
/// `tablespace` is null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__sdi_drop(tablespace: *const sys::DdTablespace) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: tablespace null or valid for read for this call.
        let ts = unsafe { tablespace.as_ref() };
        match runtime::handlerton() {
            Some(h) => result_to_error(h.sdi_drop(ts)),
            None => true,
        }
    })
}

/// # Safety
/// `tablespace` / `vector` null or valid for the call; neither is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__sdi_get_keys(
    tablespace: *const sys::DdTablespace,
    vector: *const sys::SdiVector,
) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: tablespace null or valid for read for this call.
        let ts = unsafe { tablespace.as_ref() };
        // SAFETY: vector null or valid for read for this call.
        let vec_ref = unsafe { vector.as_ref() };
        match runtime::handlerton() {
            Some(h) => result_to_error(h.sdi_get_keys(ts, vec_ref)),
            None => true,
        }
    })
}

/// `sdi_get`. Writes payload into `sdi` and updates `*sdi_len` to bytes
/// written. The trait sees `&mut [u8]` for the buffer; the shim ensures
/// `sdi_len` is non-null and reads / writes a `u64` through a local for LP64
/// safety.
///
/// # Safety
/// `tablespace` / `key` null or valid; `sdi` is non-null and writable for
/// `sdi_capacity` bytes (the value the shim copies in from `*sdi_len`);
/// `len_out` is non-null and writable for one `u64`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__sdi_get(
    tablespace: *const sys::DdTablespace,
    key: *const sys::SdiKey,
    sdi: *mut u8,
    sdi_capacity: u64,
    len_out: *mut u64,
) -> bool {
    FfiBoundary::run_default(true, || {
        let cap = usize::try_from(sdi_capacity).unwrap_or(0);
        // SAFETY: sdi is non-null and writable for sdi_capacity bytes here.
        let buf = unsafe { FfiPtr::slice_mut(sdi, cap) };
        let mut local = sdi_capacity;
        // SAFETY: tablespace null or valid for read for this call.
        let ts = unsafe { tablespace.as_ref() };
        // SAFETY: key null or valid for read for this call.
        let key_ref = unsafe { key.as_ref() };
        let err = match runtime::handlerton() {
            Some(h) => result_to_error(h.sdi_get(ts, key_ref, buf, &mut local)),
            None => true,
        };
        if !len_out.is_null() {
            // SAFETY: caller guarantees `len_out` is writable for one u64.
            unsafe { len_out.write(local) };
        }
        err
    })
}

/// `sdi_set`.
///
/// # Safety
/// `tablespace` / `table` / `key` null or valid; `payload` is non-null and
/// covers `payload_len` readable bytes for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__sdi_set(
    tablespace: *const sys::DdTablespace,
    table: *const sys::DdTable,
    key: *const sys::SdiKey,
    payload: *const u8,
    payload_len: u64,
) -> bool {
    FfiBoundary::run_default(true, || {
        let len = usize::try_from(payload_len).unwrap_or(0);
        // SAFETY: payload is non-null and covers `len` readable bytes here.
        let bytes = unsafe { FfiPtr::slice_const(payload, len) };
        // SAFETY: tablespace null or valid for read for this call.
        let ts = unsafe { tablespace.as_ref() };
        // SAFETY: table null or valid for read for this call.
        let tab = unsafe { table.as_ref() };
        // SAFETY: key null or valid for read for this call.
        let key_ref = unsafe { key.as_ref() };
        match runtime::handlerton() {
            Some(h) => result_to_error(h.sdi_set(ts, tab, key_ref, bytes)),
            None => true,
        }
    })
}

/// `sdi_delete`.
///
/// # Safety
/// `tablespace` / `table` / `key` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__sdi_delete(
    tablespace: *const sys::DdTablespace,
    table: *const sys::DdTable,
    key: *const sys::SdiKey,
) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: tablespace null or valid for read for this call.
        let ts = unsafe { tablespace.as_ref() };
        // SAFETY: table null or valid for read for this call.
        let tab = unsafe { table.as_ref() };
        // SAFETY: key null or valid for read for this call.
        let key_ref = unsafe { key.as_ref() };
        match runtime::handlerton() {
            Some(h) => result_to_error(h.sdi_delete(ts, tab, key_ref)),
            None => true,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EngineError;

    #[test]
    fn ok_maps_to_no_error() {
        assert!(!result_to_error(Ok(())));
    }

    #[test]
    fn err_maps_to_error() {
        assert!(result_to_error(Err(EngineError::Unsupported)));
        assert!(result_to_error(Err(EngineError::Internal)));
    }
}
