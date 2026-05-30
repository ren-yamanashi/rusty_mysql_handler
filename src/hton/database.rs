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

//! `rust__hton__drop_database`. Always wired on a registered handlerton — it
//! is a notification, not a capability-gating callback, so the stub forwarder
//! never changes how MySQL classifies the engine.

#![allow(unsafe_code)]

use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::runtime::FfiPtr;

/// `drop_database`.
///
/// # Safety
/// `path` is non-null and covers `path_len` readable bytes for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__drop_database(path: *const u8, path_len: usize) {
    FfiBoundary::run_void(|| {
        // SAFETY: path is non-null and covers path_len readable bytes here.
        let s = match unsafe { FfiPtr::bytes_to_str(path, path_len) } {
            Ok(v) => v,
            Err(_) => return,
        };
        if let Some(h) = runtime::handlerton() {
            h.drop_database(s);
        }
    });
}
