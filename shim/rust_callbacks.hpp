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
int32_t rust__handler__rnd_end(void *ctx);
int32_t rust__handler__rnd_next(void *ctx, uint8_t *buf, size_t buf_len);
int32_t rust__handler__rnd_pos(void *ctx, uint8_t *buf, size_t buf_len,
                               const uint8_t *pos, size_t pos_len);
void rust__handler__position(void *ctx, const uint8_t *record,
                             size_t record_len);
int32_t rust__handler__rnd_pos_by_record(void *ctx, uint8_t *record,
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

// Index — pushed join (handler.h #33-#34). NDB-style; index_read_pushed takes
// only an exact key (the shim resolves the key_part_map to a leading-bytes
// length, a null key => nullptr) with no find_flag.
int32_t rust__handler__index_read_pushed(void *ctx, uint8_t *buf,
                                         size_t buf_len, const uint8_t *key,
                                         size_t key_len);
int32_t rust__handler__index_next_pushed(void *ctx, uint8_t *buf,
                                         size_t buf_len);

// Index — read & range (handler.h #20, #22-#24, #30-#32). Keys arrive
// length-resolved as `const uint8_t *` + length (a null key => nullptr);
// find_flag is the raw ha_rkey_function integer. Each range bound decomposes
// its key_range into (key ptr, length, flag); a null bound denotes an open
// end. records_in_range returns the estimated row count, or HA_POS_ERROR
// (~uint64_t{0}) when the engine cannot estimate.
int32_t rust__handler__index_read(void *ctx, uint8_t *buf, size_t buf_len,
                                  const uint8_t *key, size_t key_len,
                                  int32_t find_flag);
int32_t rust__handler__index_read_idx_map(void *ctx, uint8_t *buf,
                                          size_t buf_len, uint32_t index,
                                          const uint8_t *key, size_t key_len,
                                          int32_t find_flag);
int32_t rust__handler__index_read_last(void *ctx, uint8_t *buf, size_t buf_len,
                                       const uint8_t *key, size_t key_len);
int32_t rust__handler__index_read_last_map(void *ctx, uint8_t *buf,
                                           size_t buf_len, const uint8_t *key,
                                           size_t key_len);
int32_t rust__handler__read_range_first(void *ctx, uint8_t *buf, size_t buf_len,
                                        const uint8_t *start_key,
                                        size_t start_len, int32_t start_flag,
                                        const uint8_t *end_key, size_t end_len,
                                        int32_t end_flag, bool eq_range,
                                        bool sorted);
int32_t rust__handler__read_range_next(void *ctx, uint8_t *buf, size_t buf_len);
uint64_t rust__handler__records_in_range(void *ctx, uint32_t inx,
                                         const uint8_t *min_key, size_t min_len,
                                         int32_t min_flag,
                                         const uint8_t *max_key, size_t max_len,
                                         int32_t max_flag);

// Row operations (handler.h #35-#38). Record buffers cross the FFI as
// `const uint8_t *` + length; the engine reads them but must not retain them.
int32_t rust__handler__write_row(void *ctx, const uint8_t *buf, size_t buf_len);
int32_t rust__handler__update_row(void *ctx, const uint8_t *old, size_t old_len,
                                  const uint8_t *new_row, size_t new_len);
int32_t rust__handler__delete_row(void *ctx, const uint8_t *buf, size_t buf_len);
int32_t rust__handler__delete_all_rows(void *ctx);

// Bulk operations (handler.h #39-#46). The start_bulk_* callbacks return the
// inverted MySQL bool (true => bulk not used, normal per-row operation);
// exec_bulk_update / bulk_update_row write the duplicate-key count through
// dup_key_found. Record buffers cross as `const uint8_t *` + length and must
// not be retained by the engine.
void rust__handler__start_bulk_insert(void *ctx, uint64_t rows);
int32_t rust__handler__end_bulk_insert(void *ctx);
bool rust__handler__start_bulk_update(void *ctx);
int32_t rust__handler__exec_bulk_update(void *ctx, uint32_t *dup_key_found);
void rust__handler__end_bulk_update(void *ctx);
int32_t rust__handler__bulk_update_row(void *ctx, const uint8_t *old,
                                       size_t old_len, const uint8_t *new_row,
                                       size_t new_len, uint32_t *dup_key_found);
bool rust__handler__start_bulk_delete(void *ctx);
int32_t rust__handler__end_bulk_delete(void *ctx);

// Bulk load + secondary-engine load (handler.h #47-#53). Opaque MySQL types
// (THD, Rows_mysql, Bulk_load::Stat_callbacks, TABLE) cross as void pointers;
// bulk_load_begin returns an engine-owned context that execute / end pass back
// unchanged. load_table writes its skip-metadata flag through the out-pointer.
bool rust__handler__bulk_load_check(void *ctx, const void *thd);
size_t rust__handler__bulk_load_available_memory(void *ctx, const void *thd);
void *rust__handler__bulk_load_begin(void *ctx, const void *thd,
                                     size_t data_size, size_t memory,
                                     size_t num_threads);
int32_t rust__handler__bulk_load_execute(void *ctx, const void *thd,
                                         void *load_ctx, size_t thread_idx,
                                         const void *rows,
                                         const void *stat_callbacks);
int32_t rust__handler__bulk_load_end(void *ctx, const void *thd,
                                     void *load_ctx, bool is_error);
int32_t rust__handler__load_table(void *ctx, const void *table,
                                  bool *skip_metadata_update);
int32_t rust__handler__unload_table(void *ctx, const uint8_t *db_name,
                                    size_t db_name_len,
                                    const uint8_t *table_name,
                                    size_t table_name_len,
                                    bool error_if_not_loaded);
}

#endif
