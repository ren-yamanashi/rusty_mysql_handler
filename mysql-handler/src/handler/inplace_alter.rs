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

//! `rust__handler__*` callbacks for in-place `ALTER TABLE` methods (handler.h
//! #124–#129). Shares the FFI safety contract documented at [`crate::handler`].
//!
//! Each callback returns `true` when the engine overrides (result written
//! through the out-pointer) and `false` to fall back to the handler base, which
//! drives the default in-place / copy ALTER protocol. `notify_table_changed` is
//! a plain void delegation. All MySQL objects cross as opaque pointers and are
//! never dereferenced from Rust.

#![allow(unsafe_code)]

use super::report::{report_bool, report_i32};
use crate::panic_guard::FfiBoundary;
use crate::runtime::EngineContext;
use crate::sys;

/// Report the supported in-place ALTER algorithm; returns whether overridden
///
/// # Safety
/// `ctx` non-null; `altered_table`/`alter_info` null-or-valid for the call;
/// `out` writable for one `i32` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__check_if_supported_inplace_alter(
    ctx: *mut EngineContext,
    altered_table: *const sys::TABLE,
    alter_info: *const sys::AlterInplaceInfo,
    out: *mut i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: both pointers are null or valid for read for the call.
        let (table, info) = unsafe { (altered_table.as_ref(), alter_info.as_ref()) };
        report_i32(out, engine.check_if_supported_inplace_alter(table, info))
    })
}

/// Prepare the in-place ALTER; returns whether the engine handled it
///
/// # Safety
/// `ctx` non-null; the four MySQL pointers null-or-valid for the call; `out`
/// writable for one `bool` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__prepare_inplace_alter_table(
    ctx: *mut EngineContext,
    altered_table: *const sys::TABLE,
    alter_info: *const sys::AlterInplaceInfo,
    old_table_def: *const sys::DdTable,
    new_table_def: *const sys::DdTable,
    out: *mut bool,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: all four pointers are null or valid for read for the call.
        let (table, info, old, new) = unsafe {
            (
                altered_table.as_ref(),
                alter_info.as_ref(),
                old_table_def.as_ref(),
                new_table_def.as_ref(),
            )
        };
        report_bool(
            out,
            engine.prepare_inplace_alter_table(table, info, old, new),
        )
    })
}

/// Apply the in-place ALTER; returns whether the engine handled it
///
/// # Safety
/// `ctx` non-null; the four MySQL pointers null-or-valid for the call; `out`
/// writable for one `bool` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__inplace_alter_table(
    ctx: *mut EngineContext,
    altered_table: *const sys::TABLE,
    alter_info: *const sys::AlterInplaceInfo,
    old_table_def: *const sys::DdTable,
    new_table_def: *const sys::DdTable,
    out: *mut bool,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: all four pointers are null or valid for read for the call.
        let (table, info, old, new) = unsafe {
            (
                altered_table.as_ref(),
                alter_info.as_ref(),
                old_table_def.as_ref(),
                new_table_def.as_ref(),
            )
        };
        report_bool(out, engine.inplace_alter_table(table, info, old, new))
    })
}

/// Commit or roll back the in-place ALTER; returns whether the engine handled it
///
/// # Safety
/// `ctx` non-null; the four MySQL pointers null-or-valid for the call; `out`
/// writable for one `bool` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__commit_inplace_alter_table(
    ctx: *mut EngineContext,
    altered_table: *const sys::TABLE,
    alter_info: *const sys::AlterInplaceInfo,
    commit: bool,
    old_table_def: *const sys::DdTable,
    new_table_def: *const sys::DdTable,
    out: *mut bool,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: all four pointers are null or valid for read for the call.
        let (table, info, old, new) = unsafe {
            (
                altered_table.as_ref(),
                alter_info.as_ref(),
                old_table_def.as_ref(),
                new_table_def.as_ref(),
            )
        };
        report_bool(
            out,
            engine.commit_inplace_alter_table(table, info, commit, old, new),
        )
    })
}

/// Notify the engine the table definition changed after ALTER
///
/// # Safety
/// `ctx` non-null; `alter_info` null-or-valid for the call.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__notify_table_changed(
    ctx: *mut EngineContext,
    alter_info: *const sys::AlterInplaceInfo,
) {
    FfiBoundary::run_void(|| {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: alter_info is null or valid for read for the call.
        engine.notify_table_changed(unsafe { alter_info.as_ref() });
    });
}

/// Report data compatibility for the copy-based ALTER path; returns overridden
///
/// # Safety
/// `ctx` non-null; `create_info` null-or-valid for the call; `out` writable for
/// one `bool` when non-null.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__handler__check_if_incompatible_data(
    ctx: *mut EngineContext,
    create_info: *const sys::HA_CREATE_INFO,
    table_changes: u32,
    out: *mut bool,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: caller guarantees ctx is non-null and exclusively owned.
        let engine = unsafe { &mut *ctx }.engine_mut();
        // SAFETY: create_info is null or valid for read for the call.
        let create_info = unsafe { create_info.as_ref() };
        report_bool(
            out,
            engine.check_if_incompatible_data(create_info, table_changes),
        )
    })
}
