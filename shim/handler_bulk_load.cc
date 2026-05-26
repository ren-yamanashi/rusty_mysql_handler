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

// Bulk-load + secondary-engine-load overrides (handler.h #47-#53)

#include "binding.hpp"
#include "my_dbug.h"
#include "rust_callbacks.hpp"
#include "safe_name.hpp"

bool RustHandlerBase::bulk_load_check(THD *thd) const {
  DBUG_TRACE;
  if (!rust_ctx_) return false;
  return rust__handler__bulk_load_check(rust_ctx_,
                                        static_cast<const void *>(thd));
}

size_t RustHandlerBase::bulk_load_available_memory(THD *thd) const {
  DBUG_TRACE;
  if (!rust_ctx_) return 0;
  return rust__handler__bulk_load_available_memory(
      rust_ctx_, static_cast<const void *>(thd));
}

void *RustHandlerBase::bulk_load_begin(THD *thd, size_t data_size,
                                       size_t memory, size_t num_threads) {
  DBUG_TRACE;
  if (!rust_ctx_) return nullptr;
  return rust__handler__bulk_load_begin(rust_ctx_,
                                        static_cast<const void *>(thd),
                                        data_size, memory, num_threads);
}

int RustHandlerBase::bulk_load_execute(THD *thd, void *load_ctx,
                                       size_t thread_idx,
                                       const Rows_mysql &rows,
                                       Bulk_load::Stat_callbacks &wait_cbk) {
  DBUG_TRACE;
  if (!rust_ctx_) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__bulk_load_execute(
      rust_ctx_, static_cast<const void *>(thd), load_ctx, thread_idx,
      static_cast<const void *>(&rows), static_cast<const void *>(&wait_cbk));
}

int RustHandlerBase::bulk_load_end(THD *thd, void *load_ctx, bool is_error) {
  DBUG_TRACE;
  if (!rust_ctx_) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__bulk_load_end(
      rust_ctx_, static_cast<const void *>(thd), load_ctx, is_error);
}

int RustHandlerBase::load_table(const TABLE &table,
                                bool *skip_metadata_update) {
  DBUG_TRACE;
  if (!rust_ctx_) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__load_table(rust_ctx_,
                                   static_cast<const void *>(&table),
                                   skip_metadata_update);
}

int RustHandlerBase::unload_table(const char *db_name, const char *table_name,
                                  bool error_if_not_loaded) {
  DBUG_TRACE;
  if (!rust_ctx_ || !db_name || !table_name) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__unload_table(
      rust_ctx_, reinterpret_cast<const uint8_t *>(db_name),
      shim::safe_name_len(db_name),
      reinterpret_cast<const uint8_t *>(table_name),
      shim::safe_name_len(table_name), error_if_not_loaded);
}
