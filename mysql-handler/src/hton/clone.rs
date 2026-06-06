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

//! `rust__hton__clone_*` callbacks. Wired only under
//! [`HtonCapabilities::CLONE`]; the shim assigns `hton->clone_interface` as
//! a unit when the capability is declared. The locator out-pointers and
//! `Ha_clone_cbk` data-transfer object cannot be safely round-tripped
//! through the opaque pass-through today; the trait sees the
//! `task_id` / `mode` / `type` / error-code arguments only, and the FFI
//! writes empty locators back to MySQL — engines wanting to drive a real
//! clone session will need a reverse-callback surface for the locator and
//! data channels.
//!
//! [`HtonCapabilities::CLONE`]: crate::hton::HtonCapabilities::CLONE

#![allow(unsafe_code)]

use crate::hton::{HaCloneMode, HaCloneType};
use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::runtime::FfiPtr;
use crate::sys;

/// # Safety
/// `out_flags` is null or writable for one `u64`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__clone_capability(out_flags: *mut u64) {
    FfiBoundary::run_void(|| {
        let flags = match runtime::handlerton() {
            Some(h) => h.clone_capability(),
            None => 0,
        };
        if !out_flags.is_null() {
            // SAFETY: caller guarantees `out_flags` is writable for one u64.
            unsafe { out_flags.write(flags) };
        }
    });
}

/// `clone_begin`. The locator out-pointers (`loc`, `loc_len`) and `task_id`
/// are written by the shim — Rust returns success/failure only.
///
/// # Safety
/// `thd` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__clone_begin(
    thd: *const sys::THD,
    clone_type: usize,
    mode: u32,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.clone_begin(
                thd_ref,
                HaCloneType::from_raw(clone_type),
                HaCloneMode::from_raw(mode),
            ),
            None => Ok(()),
        }
    })
}

/// `clone_copy`.
///
/// # Safety
/// `thd` / `cbk` null or valid for the call; `cbk` is the opaque
/// `Ha_clone_cbk` MySQL hands in. None of the pointers are retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__clone_copy(
    thd: *const sys::THD,
    task_id: u32,
    cbk: *const sys::HaCloneCbk,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        // SAFETY: cbk null or valid for read for this call.
        let cbk_ref = unsafe { cbk.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.clone_copy(thd_ref, task_id, cbk_ref),
            None => Ok(()),
        }
    })
}

/// `clone_ack`.
///
/// # Safety
/// `thd` / `cbk` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__clone_ack(
    thd: *const sys::THD,
    task_id: u32,
    in_err: i32,
    cbk: *const sys::HaCloneCbk,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        // SAFETY: cbk null or valid for read for this call.
        let cbk_ref = unsafe { cbk.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.clone_ack(thd_ref, task_id, in_err, cbk_ref),
            None => Ok(()),
        }
    })
}

/// `clone_end`.
///
/// # Safety
/// `thd` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__clone_end(
    thd: *const sys::THD,
    task_id: u32,
    in_err: i32,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.clone_end(thd_ref, task_id, in_err),
            None => Ok(()),
        }
    })
}

/// `clone_apply_begin`. `data_dir` is a bounded `&str`.
///
/// # Safety
/// `thd` null or valid; `data_dir` non-null and covers `data_dir_len`
/// readable bytes for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__clone_apply_begin(
    thd: *const sys::THD,
    mode: u32,
    data_dir: *const u8,
    data_dir_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: data_dir is non-null and covers data_dir_len readable bytes here.
        let data_dir_str = match unsafe { FfiPtr::bytes_to_str(data_dir, data_dir_len) } {
            Ok(s) => s,
            Err(e) => return Err(e),
        };
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.clone_apply_begin(thd_ref, HaCloneMode::from_raw(mode), data_dir_str),
            None => Ok(()),
        }
    })
}

/// `clone_apply`.
///
/// # Safety
/// `thd` / `cbk` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__clone_apply(
    thd: *const sys::THD,
    task_id: u32,
    in_err: i32,
    cbk: *const sys::HaCloneCbk,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        // SAFETY: cbk null or valid for read for this call.
        let cbk_ref = unsafe { cbk.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.clone_apply(thd_ref, task_id, in_err, cbk_ref),
            None => Ok(()),
        }
    })
}

/// `clone_apply_end`.
///
/// # Safety
/// `thd` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__clone_apply_end(
    thd: *const sys::THD,
    task_id: u32,
    in_err: i32,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.clone_apply_end(thd_ref, task_id, in_err),
            None => Ok(()),
        }
    })
}
