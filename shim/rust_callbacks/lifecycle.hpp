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

#ifndef SHIM_RUST_CALLBACKS_LIFECYCLE_HPP
#define SHIM_RUST_CALLBACKS_LIFECYCLE_HPP

#include <cstddef>
#include <cstdint>

// Table lifecycle (handler.h #4-#11). Opaque MySQL pointers (dd::Table,
// TABLE, TABLE_SHARE, HA_CREATE_INFO, List<Create_field>, KEY, THD) cross
// the FFI as `const void *` to keep this header free of the server-internal
// type dependencies; the Rust side re-types them through `sys::*` opaque
// structs.
extern "C" {
int32_t rust__handler__delete_table(void *ctx, const uint8_t *name,
                                    size_t name_len, const void *table_def);
int32_t rust__handler__rename_table(void *ctx, const uint8_t *from,
                                    size_t from_len, const uint8_t *to,
                                    size_t to_len, const void *from_def,
                                    const void *to_def);
void rust__handler__drop_table(void *ctx, const uint8_t *name, size_t name_len);
int32_t rust__handler__truncate(void *ctx, const void *table_def);
void rust__handler__change_table_ptr(void *ctx, const void *table,
                                     const void *share);
bool rust__handler__get_se_private_data(void *ctx, const void *dd_table,
                                        bool reset);
int32_t rust__handler__get_extra_columns_and_keys(
    void *ctx, const void *create_info, const void *create_list,
    const void *key_info, uint32_t key_count, const void *table_obj);
bool rust__handler__upgrade_table(void *ctx, const void *thd,
                                  const uint8_t *dbname, size_t dbname_len,
                                  const uint8_t *table_name,
                                  size_t table_name_len, const void *dd_table);
}

#endif
