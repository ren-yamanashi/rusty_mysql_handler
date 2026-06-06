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

// Data-dictionary backend handlerton callbacks (handler.h #28-#35). Wired only
// under HtonCapabilities::DICT_BACKEND. dict_init / ddse_dict_init have List
// output parameters the opaque pass-through cannot fill, so the bound stubs
// leave them untouched — only an engine that is the DD backend (today, just
// InnoDB) needs to populate them, and that engine will hand-write a setter
// reverse-callback when it lands. The dict_get_server_version uint* output
// round-trips through a local u32 so a future widening of the C `uint` type
// is caught at compile time rather than silently truncated.

#include <cstring>

#include "binding.hpp"
#include "rust_callbacks.hpp"
#include "sql/handler.h"

namespace {
bool rusty_hton_dict_init(dict_init_mode_t mode, uint version,
                          List<const Plugin_table> *,
                          List<const Plugin_tablespace> *) {
  return rust__hton__dict_init(static_cast<uint32_t>(mode),
                               static_cast<uint32_t>(version));
}

bool rusty_hton_ddse_dict_init(dict_init_mode_t mode, uint version,
                               List<const dd::Object_table> *,
                               List<const Plugin_tablespace> *) {
  return rust__hton__ddse_dict_init(static_cast<uint32_t>(mode),
                                    static_cast<uint32_t>(version));
}

void rusty_hton_dict_register_dd_table_id(dd::Object_id table_id) {
  rust__hton__dict_register_dd_table_id(static_cast<uint64_t>(table_id));
}

void rusty_hton_dict_cache_reset(const char *schema_name,
                                 const char *table_name) {
  rust__hton__dict_cache_reset(
      reinterpret_cast<const uint8_t *>(schema_name ? schema_name : ""),
      schema_name ? std::strlen(schema_name) : 0u,
      reinterpret_cast<const uint8_t *>(table_name ? table_name : ""),
      table_name ? std::strlen(table_name) : 0u);
}

void rusty_hton_dict_cache_reset_tables_and_tablespaces() {
  rust__hton__dict_cache_reset_tables_and_tablespaces();
}

bool rusty_hton_dict_recover(dict_recovery_mode_t mode, uint version) {
  return rust__hton__dict_recover(static_cast<uint32_t>(mode),
                                  static_cast<uint32_t>(version));
}

bool rusty_hton_dict_get_server_version(uint *version) {
  uint32_t local = 0;
  bool err = rust__hton__dict_get_server_version(&local);
  if (!err && version) {
    *version = static_cast<uint>(local);
  }
  return err;
}

bool rusty_hton_dict_set_server_version() {
  return rust__hton__dict_set_server_version();
}
}  // namespace

void rusty_hton_wire_dict(handlerton *hton) {
  hton->dict_init = rusty_hton_dict_init;
  hton->ddse_dict_init = rusty_hton_ddse_dict_init;
  hton->dict_register_dd_table_id = rusty_hton_dict_register_dd_table_id;
  hton->dict_cache_reset = rusty_hton_dict_cache_reset;
  hton->dict_cache_reset_tables_and_tablespaces =
      rusty_hton_dict_cache_reset_tables_and_tablespaces;
  hton->dict_recover = rusty_hton_dict_recover;
  hton->dict_get_server_version = rusty_hton_dict_get_server_version;
  hton->dict_set_server_version = rusty_hton_dict_set_server_version;
}
