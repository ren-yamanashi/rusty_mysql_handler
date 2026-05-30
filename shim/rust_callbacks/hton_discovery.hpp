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

#ifndef SHIM_RUST_CALLBACKS_HTON_DISCOVERY_HPP
#define SHIM_RUST_CALLBACKS_HTON_DISCOVERY_HPP

#include <cstddef>
#include <cstdint>

// Engine-level table-discovery callbacks delegating to the registered Rust
// handlerton singleton. THD crosses as opaque `const void *`; bounded byte
// pointers for db / name / path / wild come with explicit lengths so the Rust
// side never strlen-scans. None of the pointers are retained past the call.
extern "C" {
int32_t rust__hton__discover(const void *thd, const uint8_t *db, size_t db_len,
                             const uint8_t *name, size_t name_len);
int32_t rust__hton__find_files(const void *thd, const uint8_t *db,
                               size_t db_len, const uint8_t *path,
                               size_t path_len, const uint8_t *wild,
                               size_t wild_len, bool dir);
bool rust__hton__table_exists_in_engine(const void *thd, const uint8_t *db,
                                        size_t db_len, const uint8_t *name,
                                        size_t name_len);
bool rust__hton__is_supported_system_table(const uint8_t *db, size_t db_len,
                                           const uint8_t *name, size_t name_len,
                                           bool is_sql_layer_system_table);
}

#endif
