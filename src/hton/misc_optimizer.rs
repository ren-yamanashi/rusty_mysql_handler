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

//! `rust__hton__*` miscellaneous optimizer / txn / cache callbacks.
//!
//! - `is_dict_readonly`, `rm_tmp_tables`, `replace_native_transaction_in_thd`
//!   are always wired on a registered handlerton — none of them carry
//!   capability semantics that change MySQL's classification of the engine.
//! - `push_to_engine` and `get_cost_constants` are bound at the FFI / shim
//!   layer but their handlerton pointers stay NULL today. `push_to_engine`
//!   non-NULL declares the engine handles hypergraph pushdown, and
//!   `get_cost_constants` returns an engine-allocated `SE_cost_constants*`
//!   that the opaque pass-through cannot synthesise; both wait for richer
//!   reverse-callback machinery.

#![allow(unsafe_code)]

use core::ffi::c_void;

use crate::hton::result::result_to_error;
use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::sys;

/// # Safety
/// Takes no MySQL-owned pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_dict_readonly() -> bool {
    FfiBoundary::run_default(false, || match runtime::handlerton() {
        Some(h) => h.is_dict_readonly(),
        None => false,
    })
}

/// `rm_tmp_tables`. The `List<LEX_STRING>*` MySQL passes in is opaque to
/// Rust today; the trait sees only `thd`.
///
/// # Safety
/// `thd` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__rm_tmp_tables(thd: *const sys::THD) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => result_to_error(h.rm_tmp_tables(thd_ref)),
            None => false,
        }
    })
}

/// `replace_native_transaction_in_thd`. `new_trx_arg` / `ptr_trx_arg` are
/// opaque observer-style pointers dropped at the FFI boundary; the trait
/// sees only `thd`.
///
/// # Safety
/// `thd` null or valid; `new_trx_arg` / `ptr_trx_arg` are observer-owned
/// pointers not dereferenced here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__replace_native_transaction_in_thd(
    thd: *const sys::THD,
    _new_trx_arg: *mut c_void,
    _ptr_trx_arg: *mut *mut c_void,
) {
    FfiBoundary::run_void(|| {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        if let Some(h) = runtime::handlerton() {
            h.replace_native_transaction_in_thd(thd_ref);
        }
    });
}

/// `push_to_engine`. `AccessPath*` / `JOIN*` are opaque; trait sees only
/// `thd`. The handlerton pointer for this callback stays NULL today (a
/// non-NULL pointer would tell MySQL the engine accepts pushdown), so this
/// FFI symbol is bound for completeness and never called from the shim's
/// wired surface.
///
/// # Safety
/// `thd` null or valid; the access-path / join pointers are observer-owned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__push_to_engine(
    thd: *const sys::THD,
    _query: *const c_void,
    _join: *const c_void,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.push_to_engine(thd_ref),
            None => Ok(()),
        }
    })
}

/// `get_cost_constants`. The C signature returns an engine-allocated
/// `SE_cost_constants*` that MySQL takes ownership of. The shim cannot
/// safely allocate one through the opaque pass-through today; this FFI
/// symbol is bound for completeness but the handlerton pointer stays NULL.
///
/// # Safety
/// Takes no MySQL-owned pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__get_cost_constants(storage_category: u32) {
    FfiBoundary::run_void(|| {
        if let Some(h) = runtime::handlerton() {
            h.get_cost_constants(storage_category);
        }
    });
}
