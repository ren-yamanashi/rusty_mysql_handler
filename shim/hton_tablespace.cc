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

// Database / tablespace handlerton callbacks (handler.h #19-#27). drop_database
// is wired on every registered handlerton; tablespace-specific entries are
// gated by HtonCapabilities::TABLESPACES. get_tablespace must zero the
// LEX_CSTRING output up front so that an engine returning success-but-empty
// leaves no stale pointer behind. Tablespace_type is a C++ scoped enum; the
// shim writes its underlying integer into the engine-allocated u32 staging
// slot, then to the typed pointer, so a future change to the enum's underlying
// type is caught at compile time rather than silently producing a wrong write.

#include <cstring>

#include "binding.hpp"
#include "rust_callbacks.hpp"
#include "sql/handler.h"

namespace {
const uint8_t *nz(const char *s) {
  return reinterpret_cast<const uint8_t *>(s ? s : "");
}
size_t safe_len(const char *s) { return s ? std::strlen(s) : 0u; }

void rusty_hton_drop_database(handlerton *, char *path) {
  rust__hton__drop_database(nz(path), safe_len(path));
}

bool rusty_hton_is_valid_tablespace_name(ts_command_type cmd,
                                         const char *name) {
  return rust__hton__is_valid_tablespace_name(static_cast<int32_t>(cmd),
                                              nz(name), safe_len(name));
}

int rusty_hton_get_tablespace(THD *thd, LEX_CSTRING db_name,
                              LEX_CSTRING table_name,
                              LEX_CSTRING *tablespace_name) {
  if (tablespace_name) {
    tablespace_name->str = nullptr;
    tablespace_name->length = 0;
  }
  return rust__hton__get_tablespace(
      static_cast<const void *>(thd),
      reinterpret_cast<const uint8_t *>(db_name.str), db_name.length,
      reinterpret_cast<const uint8_t *>(table_name.str), table_name.length);
}

int rusty_hton_alter_tablespace(handlerton *, THD *thd,
                                st_alter_tablespace *ts_info,
                                const dd::Tablespace *, dd::Tablespace *) {
  return rust__hton__alter_tablespace(static_cast<const void *>(thd),
                                      static_cast<const void *>(ts_info));
}

const char *rusty_hton_tablespace_filename_ext() {
  return rust__hton__tablespace_filename_ext();
}

int rusty_hton_upgrade_tablespace(THD *thd) {
  return rust__hton__upgrade_tablespace(static_cast<const void *>(thd));
}

bool rusty_hton_upgrade_space_version(dd::Tablespace *tablespace) {
  return rust__hton__upgrade_space_version(static_cast<const void *>(tablespace));
}

bool rusty_hton_get_tablespace_type(const dd::Tablespace &space,
                                    Tablespace_type *space_type) {
  uint32_t raw = 0;
  bool err = rust__hton__get_tablespace_type(static_cast<const void *>(&space),
                                             &raw);
  if (!err && space_type) {
    *space_type = static_cast<Tablespace_type>(raw);
  }
  return err;
}

bool rusty_hton_get_tablespace_type_by_name(const char *tablespace_name,
                                            Tablespace_type *space_type) {
  uint32_t raw = 0;
  bool err = rust__hton__get_tablespace_type_by_name(
      nz(tablespace_name), safe_len(tablespace_name), &raw);
  if (!err && space_type) {
    *space_type = static_cast<Tablespace_type>(raw);
  }
  return err;
}
}  // namespace

void rusty_hton_wire_drop_database(handlerton *hton) {
  hton->drop_database = rusty_hton_drop_database;
}

void rusty_hton_wire_tablespaces(handlerton *hton) {
  hton->is_valid_tablespace_name = rusty_hton_is_valid_tablespace_name;
  hton->get_tablespace = rusty_hton_get_tablespace;
  hton->alter_tablespace = rusty_hton_alter_tablespace;
  hton->get_tablespace_filename_ext = rusty_hton_tablespace_filename_ext;
  hton->upgrade_tablespace = rusty_hton_upgrade_tablespace;
  hton->upgrade_space_version = rusty_hton_upgrade_space_version;
  hton->get_tablespace_type = rusty_hton_get_tablespace_type;
  hton->get_tablespace_type_by_name = rusty_hton_get_tablespace_type_by_name;
}
