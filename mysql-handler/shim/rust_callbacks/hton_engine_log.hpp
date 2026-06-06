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

#ifndef SHIM_RUST_CALLBACKS_HTON_ENGINE_LOG_HPP
#define SHIM_RUST_CALLBACKS_HTON_ENGINE_LOG_HPP

// Engine-level redo / transaction-log callbacks delegating to the registered
// Rust handlerton singleton. Wired only under HtonCapabilities::ENGINE_LOG.
// Json_dom crosses as opaque `const void *`; not retained past the call.
extern "C" {
bool rust__hton__lock_hton_log();
bool rust__hton__unlock_hton_log();
bool rust__hton__collect_hton_log_info(const void *json);
}

#endif
