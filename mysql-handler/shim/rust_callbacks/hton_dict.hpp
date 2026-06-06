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

#ifndef SHIM_RUST_CALLBACKS_HTON_DICT_HPP
#define SHIM_RUST_CALLBACKS_HTON_DICT_HPP

#include <cstddef>
#include <cstdint>

// Engine-level data-dictionary backend callbacks. Wired only under
// HtonCapabilities::DICT_BACKEND; only the storage engine acting as the DD
// backend may declare it.
extern "C" {
bool rust__hton__dict_init(uint32_t mode, uint32_t version);
bool rust__hton__ddse_dict_init(uint32_t mode, uint32_t version);
void rust__hton__dict_register_dd_table_id(uint64_t table_id);
void rust__hton__dict_cache_reset(const uint8_t *schema, size_t schema_len,
                                  const uint8_t *table, size_t table_len);
void rust__hton__dict_cache_reset_tables_and_tablespaces();
bool rust__hton__dict_recover(uint32_t mode, uint32_t version);
bool rust__hton__dict_get_server_version(uint32_t *out);
bool rust__hton__dict_set_server_version();
}

#endif
