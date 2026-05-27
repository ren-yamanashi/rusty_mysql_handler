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

//! Plugin bootstrap: registers the engine factory MySQL calls at load time.

#![allow(unsafe_code)]

use mysql_handler::panic_guard::FfiBoundary;
use mysql_handler::runtime::register_engine_factory;

use crate::TrivialEngine;

/// Plugin entry point; the shim calls this once at `INSTALL PLUGIN`.
///
/// # Safety
/// Called once from `rusty_init_func` on the mysqld thread running
/// `INSTALL PLUGIN`. Panic-safe via [`FfiBoundary::run_void`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__plugin_init() {
    FfiBoundary::run_void(|| {
        register_engine_factory(|| Box::new(TrivialEngine::default()));
    });
}
