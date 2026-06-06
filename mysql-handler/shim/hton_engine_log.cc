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

// Engine-log handlerton callbacks (handler.h #58-#60). Wired only under
// HtonCapabilities::ENGINE_LOG. Json_dom crosses as an opaque pointer; a
// future engine that reports log info will need a Rust→C++ reverse callback
// to append entries.

#include "binding.hpp"
#include "rust_callbacks.hpp"
#include "sql/handler.h"

namespace {
bool rusty_hton_lock_hton_log(handlerton *) {
  return rust__hton__lock_hton_log();
}

bool rusty_hton_unlock_hton_log(handlerton *) {
  return rust__hton__unlock_hton_log();
}

bool rusty_hton_collect_hton_log_info(handlerton *, Json_dom *json) {
  return rust__hton__collect_hton_log_info(static_cast<const void *>(json));
}
}  // namespace

void rusty_hton_wire_engine_log(handlerton *hton) {
  hton->lock_hton_log = rusty_hton_lock_hton_log;
  hton->unlock_hton_log = rusty_hton_unlock_hton_log;
  hton->collect_hton_log_info = rusty_hton_collect_hton_log_info;
}
