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

//! `rust__hton__*` callbacks that publish engine-side statistics to MySQL.
//!
//! - `get_table_statistics` fills `ha_statistics` through the
//!   [`TableStatistics`](crate::hton::TableStatistics) builder; the shim
//!   copies each field via the `mysql__hton__set_table_statistics` reverse
//!   callback declared below.
//! - `get_index_column_cardinality` round-trips a single `u64` through an
//!   out-pointer; LP64 safety is preserved at the shim boundary.
//! - `get_tablespace_statistics` keeps its handlerton pointer NULL today —
//!   it needs the same setter reverse-callback pattern over
//!   `ha_tablespace_statistics`.

#![allow(unsafe_code)]

use core::ffi::c_void;

use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::runtime::FfiPtr;

unsafe extern "C" {
    fn mysql__hton__set_table_statistics(
        stats: *mut c_void,
        records: u64,
        data_file_length: u64,
        max_data_file_length: u64,
        index_file_length: u64,
        max_index_file_length: u64,
        delete_length: u64,
        auto_increment_value: u64,
        deleted: u64,
        mean_rec_length: u64,
        create_time: i64,
        check_time: u64,
        update_time: u64,
        block_size: u32,
    );
    fn mysql__hton__set_tablespace_statistics(
        stats: *mut c_void,
        id: u64,
        logfile_group_number: u64,
        free_extents: u64,
        total_extents: u64,
        extent_size: u64,
        initial_size: u64,
        maximum_size: u64,
        autoextend_size: u64,
        version: u64,
        data_free: u64,
    );
}

/// Fill `stats` with engine-published statistics for the
/// (`db_name`, `table_name`) pair when the engine has any. Returns `true`
/// (failure) when the engine reports nothing or errors.
///
/// # Safety
/// `db_name` / `table_name` non-null and cover their stated lengths;
/// `stats` is a valid `ha_statistics *` writable for the call's duration.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__get_table_statistics(
    db_name: *const u8,
    db_name_len: usize,
    table_name: *const u8,
    table_name_len: usize,
    se_private_id: u64,
    flags: u32,
    stats: *mut c_void,
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
            Some(h) => match h.get_table_statistics(db, tab, se_private_id, flags) {
                Ok(Some(ts)) => {
                    // SAFETY: stats is a valid `ha_statistics *` for this
                    // callback only; the shim copies each field into the
                    // matching slot before returning to MySQL.
                    unsafe {
                        mysql__hton__set_table_statistics(
                            stats,
                            ts.records(),
                            ts.data_file_length(),
                            ts.max_data_file_length(),
                            ts.index_file_length(),
                            ts.max_index_file_length(),
                            ts.delete_length(),
                            ts.auto_increment_value(),
                            ts.deleted(),
                            ts.mean_rec_length(),
                            ts.create_time(),
                            ts.check_time(),
                            ts.update_time(),
                            ts.block_size(),
                        );
                    }
                    false
                }
                Ok(None) | Err(_) => true,
            },
            None => true,
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

/// Fill `stats` with engine-published tablespace statistics for the
/// (`tablespace_name`, `file_name`) pair when the engine has any. Returns
/// `true` (failure) when the engine reports nothing or fails.
///
/// # Safety
/// `tablespace_name` / `file_name` non-null and cover their stated lengths;
/// `stats` is a valid `ha_tablespace_statistics *` writable for the call's
/// duration.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__get_tablespace_statistics(
    tablespace_name: *const u8,
    tablespace_name_len: usize,
    file_name: *const u8,
    file_name_len: usize,
    stats: *mut c_void,
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
            Some(h) => match h.get_tablespace_statistics(ts, file) {
                Ok(Some(tss)) => {
                    // SAFETY: stats is a valid `ha_tablespace_statistics *`
                    // for this callback only; the shim copies each field
                    // into the matching slot before returning to MySQL.
                    unsafe {
                        mysql__hton__set_tablespace_statistics(
                            stats,
                            tss.id(),
                            tss.logfile_group_number(),
                            tss.free_extents(),
                            tss.total_extents(),
                            tss.extent_size(),
                            tss.initial_size(),
                            tss.maximum_size(),
                            tss.autoextend_size(),
                            tss.version(),
                            tss.data_free(),
                        );
                    }
                    false
                }
                Ok(None) | Err(_) => true,
            },
            None => true,
        }
    })
}
