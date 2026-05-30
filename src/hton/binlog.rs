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

//! `rust__hton__binlog_*` and `rust__hton__acl_notify` callbacks. All three are
//! always wired on a registered handlerton — they are notifications, not
//! capability-gating callbacks. The `Acl_change_notification` argument is
//! opaque and not exposed; the binlog callbacks pass enums and bounded strings.

#![allow(unsafe_code)]

use crate::hton::{EnumBinlogCommand, EnumBinlogFunc};
use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::runtime::FfiPtr;
use crate::sys;

/// `binlog_func`.
///
/// # Safety
/// `thd` is null or a valid `THD` for the call's duration; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__binlog_func(thd: *const sys::THD, func: u32) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: thd is null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.binlog_func(thd_ref, EnumBinlogFunc::from_raw(func)),
            None => Ok(()),
        }
    })
}

/// `binlog_log_query`. Query / db / table are bounded strings — the trait must
/// not log the query text per the project's security rule.
///
/// # Safety
/// Each byte pointer covers its stated length readable for the call;
/// `thd` is null or valid for the call.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rust__hton__binlog_log_query(
    thd: *const sys::THD,
    command: u32,
    query: *const u8,
    query_len: usize,
    db: *const u8,
    db_len: usize,
    table: *const u8,
    table_len: usize,
) {
    FfiBoundary::run_void(|| {
        // SAFETY: query covers query_len readable bytes for this call.
        let query_str = match unsafe { FfiPtr::bytes_to_str(query, query_len) } {
            Ok(s) => s,
            Err(_) => return,
        };
        // SAFETY: db covers db_len readable bytes for this call.
        let db_str = match unsafe { FfiPtr::bytes_to_str(db, db_len) } {
            Ok(s) => s,
            Err(_) => return,
        };
        // SAFETY: table covers table_len readable bytes for this call.
        let table_str = match unsafe { FfiPtr::bytes_to_str(table, table_len) } {
            Ok(s) => s,
            Err(_) => return,
        };
        // SAFETY: thd is null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        if let Some(h) = runtime::handlerton() {
            h.binlog_log_query(
                thd_ref,
                EnumBinlogCommand::from_raw(command),
                query_str,
                db_str,
                table_str,
            );
        }
    });
}

/// `acl_notify`. `Acl_change_notification*` is omitted (opaque).
///
/// # Safety
/// `thd` is null or a valid `THD` for the call's duration; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__acl_notify(thd: *const sys::THD) {
    FfiBoundary::run_void(|| {
        // SAFETY: thd is null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        if let Some(h) = runtime::handlerton() {
            h.acl_notify(thd_ref);
        }
    });
}
