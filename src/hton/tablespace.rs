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

//! `rust__hton__*` tablespace callbacks. Wired only under
//! [`HtonCapabilities::TABLESPACES`]. `report_tablespace_type` projects an
//! `Option<TablespaceType>` to MySQL's "false = success, true = failure"
//! convention while writing the raw enum into the C output pointer.
//!
//! [`HtonCapabilities::TABLESPACES`]: crate::hton::HtonCapabilities::TABLESPACES

#![allow(unsafe_code)]

use crate::hton::{TablespaceType, TsCommandType};
use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::runtime::FfiPtr;
use crate::sys;

fn report_tablespace_type(value: Option<TablespaceType>, out: *mut u32) -> bool {
    match value {
        Some(t) => {
            if !out.is_null() {
                // SAFETY: caller guarantees `out` is writable for one u32.
                unsafe { out.write(t.to_raw()) };
            }
            false
        }
        None => true,
    }
}

/// # Safety
/// `name` is non-null and covers `name_len` readable bytes for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__is_valid_tablespace_name(
    cmd: i32,
    name: *const u8,
    name_len: usize,
) -> bool {
    // Fail closed on panic: this returns true = valid, so reporting "valid"
    // on panic would let malformed names through into the DDL path.
    FfiBoundary::run_default(false, || {
        // SAFETY: name is non-null and covers name_len readable bytes here.
        let decoded = unsafe { FfiPtr::bytes_to_str(name, name_len) };
        match (decoded, runtime::handlerton()) {
            (Ok(s), Some(h)) => h.is_valid_tablespace_name(TsCommandType::from_raw(cmd), s),
            _ => true,
        }
    })
}

/// `get_tablespace`. The shim sets `LEX_CSTRING*` empty before calling; an
/// `Ok(())` return leaves it as "no tablespace info".
///
/// # Safety
/// `thd` null or valid; `db`/`table` non-null and cover their stated lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__get_tablespace(
    thd: *const sys::THD,
    db: *const u8,
    db_len: usize,
    table: *const u8,
    table_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: each byte pointer is non-null and covers its stated length.
        let (db_str, table_str) = unsafe {
            match (
                FfiPtr::bytes_to_str(db, db_len),
                FfiPtr::bytes_to_str(table, table_len),
            ) {
                (Ok(a), Ok(b)) => (a, b),
                (Err(e), _) | (_, Err(e)) => return Err(e),
            }
        };
        // SAFETY: thd is null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.get_tablespace(thd_ref, db_str, table_str),
            None => Ok(()),
        }
    })
}

/// # Safety
/// `thd` / `ts_info` null or valid for the call; neither is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__alter_tablespace(
    thd: *const sys::THD,
    ts_info: *const sys::StAlterTablespace,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: thd is null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        // SAFETY: ts_info is null or valid for read for this call.
        let ts_ref = unsafe { ts_info.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.alter_tablespace(thd_ref, ts_ref),
            None => Ok(()),
        }
    })
}

/// Static, null-terminated extension string or NULL.
///
/// # Safety
/// Takes no MySQL-owned pointer; returned pointer is `'static`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__tablespace_filename_ext() -> *const core::ffi::c_char {
    FfiBoundary::run_default(core::ptr::null(), || match runtime::handlerton() {
        Some(h) => match h.tablespace_filename_ext() {
            Some(ext) => ext.as_ptr(),
            None => core::ptr::null(),
        },
        None => core::ptr::null(),
    })
}

/// # Safety
/// `thd` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__upgrade_tablespace(thd: *const sys::THD) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: thd is null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.upgrade_tablespace(thd_ref),
            None => Ok(()),
        }
    })
}

/// Returns `true` on error.
///
/// # Safety
/// `tablespace` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__upgrade_space_version(
    tablespace: *const sys::DdTablespace,
) -> bool {
    // Fail-on-panic, MySQL "true = error" convention.
    FfiBoundary::run_default(true, || {
        // SAFETY: tablespace is null or valid for read for this call.
        let ts_ref = unsafe { tablespace.as_ref() };
        match runtime::handlerton() {
            Some(h) => match h.upgrade_space_version(ts_ref) {
                Ok(()) => false,
                Err(_) => true,
            },
            None => false,
        }
    })
}

/// `get_tablespace_type`. Writes the resolved type into `out` (a `u32` chosen
/// to match `Tablespace_type` on every supported platform); returns `true` on
/// failure (no type determined).
///
/// # Safety
/// `tablespace` null or valid for the call; `out` is null or writable for one
/// `u32`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__get_tablespace_type(
    tablespace: *const sys::DdTablespace,
    out: *mut u32,
) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: tablespace is null or valid for read for this call.
        let ts_ref = unsafe { tablespace.as_ref() };
        let value = match runtime::handlerton() {
            Some(h) => h.get_tablespace_type(ts_ref),
            None => None,
        };
        // SAFETY: caller guarantees `out` is null or writable for one u32.
        report_tablespace_type(value, out)
    })
}

/// `get_tablespace_type_by_name`. Same projection as
/// [`rust__hton__get_tablespace_type`].
///
/// # Safety
/// `name` non-null and covers `name_len` readable bytes; `out` null or
/// writable for one `u32`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__get_tablespace_type_by_name(
    name: *const u8,
    name_len: usize,
    out: *mut u32,
) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: name is non-null and covers name_len readable bytes here.
        let decoded = unsafe { FfiPtr::bytes_to_str(name, name_len) };
        let value = match (decoded, runtime::handlerton()) {
            (Ok(s), Some(h)) => h.get_tablespace_type_by_name(s),
            _ => None,
        };
        // SAFETY: caller guarantees `out` is null or writable for one u32.
        report_tablespace_type(value, out)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_tablespace_type_writes_on_some_and_returns_false() {
        let mut out: u32 = 0xAAAA_AAAA;
        let res = report_tablespace_type(Some(TablespaceType::Undo), &raw mut out);
        assert!(!res);
        assert_eq!(out, TablespaceType::Undo.to_raw());
    }

    #[test]
    fn report_tablespace_type_none_leaves_buffer_and_returns_true() {
        let mut out: u32 = 7;
        let res = report_tablespace_type(None, &raw mut out);
        assert!(res);
        assert_eq!(out, 7);
    }

    #[test]
    fn report_tablespace_type_tolerates_null_out() {
        assert!(!report_tablespace_type(
            Some(TablespaceType::System),
            core::ptr::null_mut()
        ));
        assert!(report_tablespace_type(None, core::ptr::null_mut()));
    }
}
