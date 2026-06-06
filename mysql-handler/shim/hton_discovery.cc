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

// Table-discovery handlerton callbacks (handler.h #45-#48). discover and
// find_files cannot populate their MySQL-owned output through the opaque
// pass-through, so the C++ side wires them but leaves the outputs untouched:
// discover lets the int return code carry the found / not-found decision and
// returns HA_ERR_NO_SUCH_TABLE on the default unsupported path; find_files
// keeps the List<LEX_STRING>* the caller passed unchanged.

#include <cstring>

#include "binding.hpp"
#include "my_base.h"
#include "rust_callbacks.hpp"
#include "sql/handler.h"

namespace {
// safe_len handles both null pointer (no string) and a non-null one whose
// length we do not yet know. discover / find_files only see paths that
// already lived through MySQL's filesystem helpers, so they are safe to
// strlen-scan when non-null.
size_t safe_len(const char *s) { return s ? std::strlen(s) : 0; }

int rusty_hton_discover(handlerton *, THD *thd, const char *db,
                        const char *name, uchar **frmblob, size_t *frmlen) {
  if (frmblob) *frmblob = nullptr;
  if (frmlen) *frmlen = 0;
  if (!db || !name) return HA_ERR_NO_SUCH_TABLE;
  return rust__hton__discover(static_cast<const void *>(thd),
                              reinterpret_cast<const uint8_t *>(db),
                              std::strlen(db),
                              reinterpret_cast<const uint8_t *>(name),
                              std::strlen(name));
}

int rusty_hton_find_files(handlerton *, THD *thd, const char *db,
                          const char *path, const char *wild, bool dir,
                          List<LEX_STRING> *) {
  if (!db || !path) return 0;
  return rust__hton__find_files(
      static_cast<const void *>(thd),
      reinterpret_cast<const uint8_t *>(db), std::strlen(db),
      reinterpret_cast<const uint8_t *>(path), std::strlen(path),
      reinterpret_cast<const uint8_t *>(wild), safe_len(wild), dir);
}

int rusty_hton_table_exists_in_engine(handlerton *, THD *thd, const char *db,
                                      const char *name) {
  if (!db || !name) return HA_ERR_NO_SUCH_TABLE;
  bool exists = rust__hton__table_exists_in_engine(
      static_cast<const void *>(thd),
      reinterpret_cast<const uint8_t *>(db), std::strlen(db),
      reinterpret_cast<const uint8_t *>(name), std::strlen(name));
  return exists ? HA_ERR_TABLE_EXIST : HA_ERR_NO_SUCH_TABLE;
}

bool rusty_hton_is_supported_system_table(const char *db,
                                          const char *table_name,
                                          bool is_sql_layer_system_table) {
  if (!db || !table_name) return false;
  return rust__hton__is_supported_system_table(
      reinterpret_cast<const uint8_t *>(db), std::strlen(db),
      reinterpret_cast<const uint8_t *>(table_name), std::strlen(table_name),
      is_sql_layer_system_table);
}
}  // namespace

void rusty_hton_wire_discovery(handlerton *hton) {
  hton->discover = rusty_hton_discover;
  hton->find_files = rusty_hton_find_files;
  hton->table_exists_in_engine = rusty_hton_table_exists_in_engine;
  hton->is_supported_system_table = rusty_hton_is_supported_system_table;
}
