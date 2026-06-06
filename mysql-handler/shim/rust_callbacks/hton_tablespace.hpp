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

#ifndef SHIM_RUST_CALLBACKS_HTON_TABLESPACE_HPP
#define SHIM_RUST_CALLBACKS_HTON_TABLESPACE_HPP

#include <cstddef>
#include <cstdint>

// Engine-level database / tablespace callbacks. drop_database is always wired
// on a registered handlerton; the tablespace-specific entries are gated by
// HtonCapabilities::TABLESPACES. THD / dd::Tablespace / st_alter_tablespace
// cross as opaque `const void *`; byte pointers come with explicit lengths.
extern "C" {
void rust__hton__drop_database(const uint8_t *path, size_t path_len);
bool rust__hton__is_valid_tablespace_name(int32_t cmd, const uint8_t *name,
                                          size_t name_len);
int32_t rust__hton__get_tablespace(const void *thd, const uint8_t *db,
                                   size_t db_len, const uint8_t *table,
                                   size_t table_len);
int32_t rust__hton__alter_tablespace(const void *thd, const void *ts_info);
const char *rust__hton__tablespace_filename_ext();
int32_t rust__hton__upgrade_tablespace(const void *thd);
bool rust__hton__upgrade_space_version(const void *tablespace);
bool rust__hton__get_tablespace_type(const void *tablespace, uint32_t *out);
bool rust__hton__get_tablespace_type_by_name(const uint8_t *name,
                                             size_t name_len, uint32_t *out);
}

#endif
