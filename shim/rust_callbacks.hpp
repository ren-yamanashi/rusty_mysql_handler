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

#ifndef SHIM_RUST_CALLBACKS_HPP
#define SHIM_RUST_CALLBACKS_HPP

#include <cstddef>
#include <cstdint>

extern "C" {

void rust__plugin_init();
void *rust__create_engine();
void rust__destroy_engine(void *ctx);

const char *rust__handler__table_type(void *ctx);
uint64_t rust__handler__table_flags(void *ctx);
uint32_t rust__handler__index_flags(void *ctx, uint32_t idx, uint32_t part,
                                    bool all_parts);

int32_t rust__handler__create(void *ctx, const uint8_t *name, size_t name_len);
int32_t rust__handler__open(void *ctx, const uint8_t *name, size_t name_len,
                            int32_t mode);
int32_t rust__handler__close(void *ctx);

int32_t rust__handler__rnd_init(void *ctx, bool scan);
int32_t rust__handler__rnd_next(void *ctx, uint8_t *buf, size_t buf_len);
int32_t rust__handler__rnd_pos(void *ctx, uint8_t *buf, size_t buf_len,
                               const uint8_t *pos, size_t pos_len);
void rust__handler__position(void *ctx, const uint8_t *record,
                             size_t record_len);
int32_t rust__handler__info(void *ctx, uint32_t flag);

// Table lifecycle (handler.h #4-#11). Opaque MySQL pointers (dd::Table,
// TABLE, TABLE_SHARE, HA_CREATE_INFO, List<Create_field>, KEY, THD) cross
// the FFI as `const void *` to keep this header free of the server-internal
// type dependencies; the Rust side re-types them through `sys::*` opaque
// structs.
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

// Index — basic (handler.h #18-#19, #21, #25-#29). The shim resolves the
// original key_part_map to a leading-bytes length before crossing the FFI, so
// the key arrives as `const uint8_t *` + length; `find_flag` is the raw
// ha_rkey_function integer.
int32_t rust__handler__index_init(void *ctx, uint32_t idx, bool sorted);
int32_t rust__handler__index_end(void *ctx);
int32_t rust__handler__index_read_map(void *ctx, uint8_t *buf, size_t buf_len,
                                      const uint8_t *key, size_t key_len,
                                      int32_t find_flag);
int32_t rust__handler__index_next(void *ctx, uint8_t *buf, size_t buf_len);
int32_t rust__handler__index_prev(void *ctx, uint8_t *buf, size_t buf_len);
int32_t rust__handler__index_first(void *ctx, uint8_t *buf, size_t buf_len);
int32_t rust__handler__index_last(void *ctx, uint8_t *buf, size_t buf_len);
int32_t rust__handler__index_next_same(void *ctx, uint8_t *buf, size_t buf_len,
                                       const uint8_t *key, size_t key_len);

// Row operations (handler.h #35-#38). Record buffers cross the FFI as
// `const uint8_t *` + length; the engine reads them but must not retain them.
int32_t rust__handler__write_row(void *ctx, const uint8_t *buf, size_t buf_len);
int32_t rust__handler__update_row(void *ctx, const uint8_t *old, size_t old_len,
                                  const uint8_t *new_row, size_t new_len);
int32_t rust__handler__delete_row(void *ctx, const uint8_t *buf, size_t buf_len);
int32_t rust__handler__delete_all_rows(void *ctx);
}

#endif
