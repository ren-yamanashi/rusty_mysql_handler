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

//! `rust__hton__*` miscellaneous encryption / redo / stats / DDL callbacks.
//!
//! - `rotate_encryption_master_key` is wired only under
//!   [`HtonCapabilities::ENCRYPTION`].
//! - `redo_log_set_state` is wired only under
//!   [`HtonCapabilities::ENGINE_LOG`].
//! - `post_ddl` and `post_recover` are always wired on a registered
//!   handlerton (notifications).
//! - The three statistics callbacks (`get_table_statistics`,
//!   `get_index_column_cardinality`, `get_tablespace_statistics`) are bound
//!   for completeness but their handlerton pointers stay NULL — the output
//!   structs (`ha_statistics`, `ha_tablespace_statistics`) cannot yet be
//!   populated through the opaque pass-through. `get_index_column_cardinality`
//!   round-trips `ulonglong *` through a local `u64` for LP64 safety.
//!
//! [`HtonCapabilities::ENCRYPTION`]: crate::hton::HtonCapabilities::ENCRYPTION
//! [`HtonCapabilities::ENGINE_LOG`]: crate::hton::HtonCapabilities::ENGINE_LOG

#![allow(unsafe_code)]

use crate::hton::result::result_to_error;
use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::runtime::FfiPtr;
use crate::sys;

/// # Safety
/// Takes no MySQL-owned pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__rotate_encryption_master_key() -> bool {
    FfiBoundary::run_default(true, || match runtime::handlerton() {
        Some(h) => result_to_error(h.rotate_encryption_master_key()),
        None => false,
    })
}

/// # Safety
/// `thd` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__redo_log_set_state(
    thd: *const sys::THD,
    enable: bool,
) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => result_to_error(h.redo_log_set_state(thd_ref, enable)),
            None => false,
        }
    })
}

/// `get_table_statistics`. The `ha_statistics*` output is opaque to Rust
/// today, so the handlerton pointer stays NULL; this FFI symbol exists for
/// completeness.
///
/// # Safety
/// `db_name` / `table_name` non-null and cover their stated lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__get_table_statistics(
    db_name: *const u8,
    db_name_len: usize,
    table_name: *const u8,
    table_name_len: usize,
    se_private_id: u64,
    flags: u32,
) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: db_name is non-null and covers its length here.
        let db = match unsafe { FfiPtr::bytes_to_str(db_name, db_name_len) } {
            Ok(s) => s,
            Err(_) => return true,
        };
        // SAFETY: table_name is non-null and covers its length here.
        let tab = match unsafe { FfiPtr::bytes_to_str(table_name, table_name_len) } {
            Ok(s) => s,
            Err(_) => return true,
        };
        match runtime::handlerton() {
            Some(h) => result_to_error(h.get_table_statistics(db, tab, se_private_id, flags)),
            None => false,
        }
    })
}

/// `get_index_column_cardinality`. Trait returns `Option<u64>` and this
/// callback writes it through `out_cardinality` when present.
///
/// # Safety
/// `db_name` / `table_name` / `index_name` non-null and cover their lengths;
/// `out_cardinality` null or writable for one `u64`.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rust__hton__get_index_column_cardinality(
    db_name: *const u8,
    db_name_len: usize,
    table_name: *const u8,
    table_name_len: usize,
    index_name: *const u8,
    index_name_len: usize,
    index_ordinal_position: u32,
    column_ordinal_position: u32,
    se_private_id: u64,
    out_cardinality: *mut u64,
) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: each byte pointer covers its stated length for this call.
        let (db, tab, idx) = unsafe {
            match (
                FfiPtr::bytes_to_str(db_name, db_name_len),
                FfiPtr::bytes_to_str(table_name, table_name_len),
                FfiPtr::bytes_to_str(index_name, index_name_len),
            ) {
                (Ok(a), Ok(b), Ok(c)) => (a, b, c),
                _ => return true,
            }
        };
        let result = match runtime::handlerton() {
            Some(h) => h.get_index_column_cardinality(
                db,
                tab,
                idx,
                index_ordinal_position,
                column_ordinal_position,
                se_private_id,
            ),
            None => Ok(None),
        };
        match result {
            Ok(Some(card)) => {
                if out_cardinality.is_null() {
                    // Fail closed: reporting success without writing leaves
                    // MySQL with whatever the slot held before the call.
                    return true;
                }
                // SAFETY: caller guarantees out_cardinality is writable for one u64.
                unsafe { out_cardinality.write(card) };
                false
            }
            Ok(None) | Err(_) => true,
        }
    })
}

/// `get_tablespace_statistics`. Same opaque-output limitation as
/// [`rust__hton__get_table_statistics`].
///
/// # Safety
/// `tablespace_name` / `file_name` non-null and cover their stated lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__get_tablespace_statistics(
    tablespace_name: *const u8,
    tablespace_name_len: usize,
    file_name: *const u8,
    file_name_len: usize,
) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: tablespace_name is non-null and covers its length.
        let ts = match unsafe { FfiPtr::bytes_to_str(tablespace_name, tablespace_name_len) } {
            Ok(s) => s,
            Err(_) => return true,
        };
        // SAFETY: file_name is non-null and covers its length.
        let file = match unsafe { FfiPtr::bytes_to_str(file_name, file_name_len) } {
            Ok(s) => s,
            Err(_) => return true,
        };
        match runtime::handlerton() {
            Some(h) => result_to_error(h.get_tablespace_statistics(ts, file)),
            None => false,
        }
    })
}

/// # Safety
/// `thd` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__post_ddl(thd: *const sys::THD) {
    FfiBoundary::run_void(|| {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        if let Some(h) = runtime::handlerton() {
            h.post_ddl(thd_ref);
        }
    });
}

/// # Safety
/// Takes no MySQL-owned pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__post_recover() {
    FfiBoundary::run_void(|| {
        if let Some(h) = runtime::handlerton() {
            h.post_recover();
        }
    });
}
