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

//! `rust__hton__notify_*` callbacks. The shim wires every notification
//! unconditionally on a registered handlerton — they are notifications, not
//! capability-gating, so a stub default never changes how MySQL classifies
//! the engine. The MDL-keyed hooks return `bool` in MySQL's "true =
//! error/veto, false = success" convention; `result_to_veto` performs the
//! conversion.

#![allow(unsafe_code)]

use crate::hton::{HaNotificationType, SelectExecutedIn};
use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::runtime::FfiPtr;
use crate::sys;

fn result_to_veto(r: crate::engine::EngineResult) -> bool {
    match r {
        Ok(()) => false,
        Err(_) => true,
    }
}

/// `notify_after_select`.
///
/// # Safety
/// `thd` null or a valid `THD` for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__notify_after_select(thd: *const sys::THD, executed_in: bool) {
    FfiBoundary::run_void(|| {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        if let Some(h) = runtime::handlerton() {
            h.notify_after_select(thd_ref, SelectExecutedIn::from_raw(executed_in));
        }
    });
}

/// `notify_create_table`. `HA_CREATE_INFO*` omitted (opaque).
///
/// # Safety
/// `db` / `table` are non-null and cover their stated lengths readable for the
/// call (the shim drops the call when either is NULL).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__notify_create_table(
    db: *const u8,
    db_len: usize,
    table: *const u8,
    table_len: usize,
) {
    FfiBoundary::run_void(|| {
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
        if let Some(h) = runtime::handlerton() {
            h.notify_create_table(db_str, table_str);
        }
    });
}

/// `notify_drop_table`. `Table_ref*` omitted (opaque).
///
/// # Safety
/// Takes no MySQL-owned pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__notify_drop_table() {
    FfiBoundary::run_void(|| {
        if let Some(h) = runtime::handlerton() {
            h.notify_drop_table();
        }
    });
}

/// `notify_exclusive_mdl`. Returns true (veto/error) only when engine returns `Err`.
///
/// # Safety
/// `thd` / `mdl_key` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__notify_exclusive_mdl(
    thd: *const sys::THD,
    mdl_key: *const sys::MdlKey,
    kind: i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        // SAFETY: mdl_key null or valid for read for this call.
        let key_ref = unsafe { mdl_key.as_ref() };
        match runtime::handlerton() {
            Some(h) => result_to_veto(h.notify_exclusive_mdl(
                thd_ref,
                key_ref,
                HaNotificationType::from_raw(kind),
            )),
            None => false,
        }
    })
}

/// `notify_alter_table`.
///
/// # Safety
/// `thd` / `mdl_key` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__notify_alter_table(
    thd: *const sys::THD,
    mdl_key: *const sys::MdlKey,
    kind: i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        // SAFETY: mdl_key null or valid for read for this call.
        let key_ref = unsafe { mdl_key.as_ref() };
        match runtime::handlerton() {
            Some(h) => result_to_veto(h.notify_alter_table(
                thd_ref,
                key_ref,
                HaNotificationType::from_raw(kind),
            )),
            None => false,
        }
    })
}

/// `notify_rename_table`. Old / new db.table names are bounded strings.
///
/// # Safety
/// Each byte pointer is non-null (the shim substitutes a non-null empty
/// sentinel for a NULL input, matching `FfiPtr::bytes_to_str`'s non-null
/// contract) and covers its stated length readable for the call.
/// `thd` / `mdl_key` null or valid for the call.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rust__hton__notify_rename_table(
    thd: *const sys::THD,
    mdl_key: *const sys::MdlKey,
    kind: i32,
    old_db: *const u8,
    old_db_len: usize,
    old_name: *const u8,
    old_name_len: usize,
    new_db: *const u8,
    new_db_len: usize,
    new_name: *const u8,
    new_name_len: usize,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: each (p, len) pair covers `len` readable bytes for this call.
        let decoded = unsafe {
            (
                FfiPtr::bytes_to_str(old_db, old_db_len),
                FfiPtr::bytes_to_str(old_name, old_name_len),
                FfiPtr::bytes_to_str(new_db, new_db_len),
                FfiPtr::bytes_to_str(new_name, new_name_len),
            )
        };
        let (old_db_str, old_name_str, new_db_str, new_name_str) = match decoded {
            (Ok(a), Ok(b), Ok(c), Ok(d)) => (a, b, c, d),
            _ => return false,
        };
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        // SAFETY: mdl_key null or valid for read for this call.
        let key_ref = unsafe { mdl_key.as_ref() };
        match runtime::handlerton() {
            Some(h) => result_to_veto(h.notify_rename_table(
                thd_ref,
                key_ref,
                HaNotificationType::from_raw(kind),
                old_db_str,
                old_name_str,
                new_db_str,
                new_name_str,
            )),
            None => false,
        }
    })
}

/// `notify_truncate_table`.
///
/// # Safety
/// `thd` / `mdl_key` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__notify_truncate_table(
    thd: *const sys::THD,
    mdl_key: *const sys::MdlKey,
    kind: i32,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        // SAFETY: mdl_key null or valid for read for this call.
        let key_ref = unsafe { mdl_key.as_ref() };
        match runtime::handlerton() {
            Some(h) => result_to_veto(h.notify_truncate_table(
                thd_ref,
                key_ref,
                HaNotificationType::from_raw(kind),
            )),
            None => false,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EngineError;

    #[test]
    fn ok_maps_to_no_veto() {
        assert!(!result_to_veto(Ok(())));
    }

    #[test]
    fn err_maps_to_veto() {
        assert!(result_to_veto(Err(EngineError::Unsupported)));
        assert!(result_to_veto(Err(EngineError::Internal)));
    }
}
