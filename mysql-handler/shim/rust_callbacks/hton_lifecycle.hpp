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

#ifndef SHIM_RUST_CALLBACKS_HTON_LIFECYCLE_HPP
#define SHIM_RUST_CALLBACKS_HTON_LIFECYCLE_HPP

#include <cstdint>

// Core engine-lifecycle callbacks delegating to the registered Rust Handlerton
// singleton. THD crosses as an opaque `const void *` (never retained past the
// call).
extern "C" {
int32_t rust__hton__close_connection(const void *thd);
void rust__hton__kill_connection(const void *thd);
void rust__hton__pre_dd_shutdown();
void rust__hton__reset_plugin_vars(const void *thd);
}

#endif
