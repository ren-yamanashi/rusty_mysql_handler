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

//! `rust__hton__dict_*` callbacks. Wired only under
//! [`HtonCapabilities::DICT_BACKEND`] — only the storage engine acting as the
//! data dictionary backend (InnoDB in practice) may declare it. The
//! `dict_init` / `ddse_dict_init` output lists are not surfaced today because
//! the trivial example engine never opts into DICT_BACKEND; a future DD backend
//! will need a setter reverse-callback to populate them.
//!
//! [`HtonCapabilities::DICT_BACKEND`]: crate::hton::HtonCapabilities::DICT_BACKEND

#![allow(unsafe_code)]

use crate::hton::{DictInitMode, DictRecoveryMode};
use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::runtime::FfiPtr;

fn report_u32(value: Option<u32>, out: *mut u32) -> bool {
    match value {
        Some(v) => {
            if !out.is_null() {
                // SAFETY: caller guarantees `out` is writable for one u32.
                unsafe { out.write(v) };
            }
            false
        }
        None => true,
    }
}

/// `dict_init`. Returns `true` on error (MySQL convention).
///
/// # Safety
/// Takes no MySQL-owned pointer beyond the enum and version.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__dict_init(mode: u32, version: u32) -> bool {
    FfiBoundary::run_default(false, || match runtime::handlerton() {
        Some(h) => match h.dict_init(DictInitMode::from_raw(mode), version) {
            Ok(()) => false,
            Err(_) => true,
        },
        None => false,
    })
}

/// `ddse_dict_init`.
///
/// # Safety
/// Takes no MySQL-owned pointer beyond the enum and version.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__ddse_dict_init(mode: u32, version: u32) -> bool {
    FfiBoundary::run_default(false, || match runtime::handlerton() {
        Some(h) => match h.ddse_dict_init(DictInitMode::from_raw(mode), version) {
            Ok(()) => false,
            Err(_) => true,
        },
        None => false,
    })
}

/// `dict_register_dd_table_id`. `table_id` is `dd::Object_id` (64-bit).
///
/// # Safety
/// Takes no MySQL-owned pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__dict_register_dd_table_id(table_id: u64) {
    FfiBoundary::run_void(|| {
        if let Some(h) = runtime::handlerton() {
            h.dict_register_dd_table_id(table_id);
        }
    });
}

/// `dict_cache_reset`.
///
/// # Safety
/// `schema` / `table` are non-null and cover their stated lengths for the
/// call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__dict_cache_reset(
    schema: *const u8,
    schema_len: usize,
    table: *const u8,
    table_len: usize,
) {
    FfiBoundary::run_void(|| {
        // SAFETY: schema is non-null and covers schema_len readable bytes here.
        let schema_str = match unsafe { FfiPtr::bytes_to_str(schema, schema_len) } {
            Ok(s) => s,
            Err(_) => return,
        };
        // SAFETY: table is non-null and covers table_len readable bytes here.
        let table_str = match unsafe { FfiPtr::bytes_to_str(table, table_len) } {
            Ok(s) => s,
            Err(_) => return,
        };
        if let Some(h) = runtime::handlerton() {
            h.dict_cache_reset(schema_str, table_str);
        }
    });
}

/// `dict_cache_reset_tables_and_tablespaces`.
///
/// # Safety
/// Takes no MySQL-owned pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__dict_cache_reset_tables_and_tablespaces() {
    FfiBoundary::run_void(|| {
        if let Some(h) = runtime::handlerton() {
            h.dict_cache_reset_tables_and_tablespaces();
        }
    });
}

/// `dict_recover`.
///
/// # Safety
/// Takes no MySQL-owned pointer beyond the enum and version.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__dict_recover(mode: u32, version: u32) -> bool {
    FfiBoundary::run_default(false, || match runtime::handlerton() {
        Some(h) => match h.dict_recover(DictRecoveryMode::from_raw(mode), version) {
            Ok(()) => false,
            Err(_) => true,
        },
        None => false,
    })
}

/// `dict_get_server_version`. Writes the version into `out` and returns
/// `false` on success; returns `true` (error) when the engine has no version
/// to report.
///
/// # Safety
/// `out` is null or writable for one `u32`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__dict_get_server_version(out: *mut u32) -> bool {
    FfiBoundary::run_default(true, || {
        let value = match runtime::handlerton() {
            Some(h) => h.dict_get_server_version(),
            None => None,
        };
        // SAFETY: caller guarantees `out` is null or writable for one u32.
        report_u32(value, out)
    })
}

/// `dict_set_server_version`.
///
/// # Safety
/// Takes no MySQL-owned pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__dict_set_server_version() -> bool {
    FfiBoundary::run_default(false, || match runtime::handlerton() {
        Some(h) => match h.dict_set_server_version() {
            Ok(()) => false,
            Err(_) => true,
        },
        None => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_u32_some_writes_and_returns_false() {
        let mut out: u32 = 0;
        let res = report_u32(Some(42), &raw mut out);
        assert!(!res);
        assert_eq!(out, 42);
    }

    #[test]
    fn report_u32_none_returns_true_without_writing() {
        let mut out: u32 = 99;
        let res = report_u32(None, &raw mut out);
        assert!(res);
        assert_eq!(out, 99);
    }

    #[test]
    fn report_u32_tolerates_null_out() {
        assert!(!report_u32(Some(0), core::ptr::null_mut()));
        assert!(report_u32(None, core::ptr::null_mut()));
    }
}
