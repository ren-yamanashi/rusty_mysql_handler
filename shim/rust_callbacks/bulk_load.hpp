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

#ifndef SHIM_RUST_CALLBACKS_BULK_LOAD_HPP
#define SHIM_RUST_CALLBACKS_BULK_LOAD_HPP

#include <cstddef>
#include <cstdint>

// Bulk load + secondary-engine load (handler.h #47-#53). Opaque MySQL types
// (THD, Rows_mysql, Bulk_load::Stat_callbacks, TABLE) cross as void pointers;
// bulk_load_begin returns an engine-owned context that execute / end pass back
// unchanged. load_table writes its skip-metadata flag through the out-pointer.
extern "C" {
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
