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

// Table-lifecycle overrides (handler.h #4-#11). Split out of binding.cc to
// keep both files under the 250-line cap.

#include "binding.hpp"
#include "my_dbug.h"
#include "rust_callbacks.hpp"
#include "safe_name.hpp"

int RustHandlerBase::delete_table(const char *name,
                                  const dd::Table *table_def) {
  DBUG_TRACE;
  if (!rust_ctx_) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__delete_table(
      rust_ctx_, reinterpret_cast<const uint8_t *>(name),
      shim::safe_name_len(name), static_cast<const void *>(table_def));
}

int RustHandlerBase::rename_table(const char *from, const char *to,
                                  const dd::Table *from_table_def,
                                  dd::Table *to_table_def) {
  DBUG_TRACE;
  if (!rust_ctx_) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__rename_table(
      rust_ctx_, reinterpret_cast<const uint8_t *>(from),
      shim::safe_name_len(from), reinterpret_cast<const uint8_t *>(to),
      shim::safe_name_len(to), static_cast<const void *>(from_table_def),
      static_cast<const void *>(to_table_def));
}

void RustHandlerBase::drop_table(const char *name) {
  DBUG_TRACE;
  if (!rust_ctx_) return;
  rust__handler__drop_table(rust_ctx_,
                            reinterpret_cast<const uint8_t *>(name),
                            shim::safe_name_len(name));
}

int RustHandlerBase::truncate(dd::Table *table_def) {
  DBUG_TRACE;
  if (!rust_ctx_) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__truncate(rust_ctx_,
                                 static_cast<const void *>(table_def));
}

// Preserve base behaviour so handler::table / table_share stay coherent before
// the engine observes the change.
void RustHandlerBase::change_table_ptr(TABLE *table_arg, TABLE_SHARE *share) {
  handler::change_table_ptr(table_arg, share);
  if (rust_ctx_)
    rust__handler__change_table_ptr(rust_ctx_, table_arg, share);
}

bool RustHandlerBase::get_se_private_data(dd::Table *dd_table, bool reset) {
  DBUG_TRACE;
  if (!rust_ctx_) return false;
  return rust__handler__get_se_private_data(
      rust_ctx_, static_cast<const void *>(dd_table), reset);
}

int RustHandlerBase::get_extra_columns_and_keys(
    const HA_CREATE_INFO *create_info, const List<Create_field> *create_list,
    const KEY *key_info, uint key_count, dd::Table *table_obj) {
  DBUG_TRACE;
  if (!rust_ctx_) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__get_extra_columns_and_keys(
      rust_ctx_, static_cast<const void *>(create_info),
      static_cast<const void *>(create_list),
      static_cast<const void *>(key_info), key_count,
      static_cast<const void *>(table_obj));
}

bool RustHandlerBase::upgrade_table(THD *thd, const char *dbname,
                                    const char *table_name,
                                    dd::Table *dd_table) {
  DBUG_TRACE;
  if (!rust_ctx_) return false;
  return rust__handler__upgrade_table(
      rust_ctx_, static_cast<const void *>(thd),
      reinterpret_cast<const uint8_t *>(dbname), shim::safe_name_len(dbname),
      reinterpret_cast<const uint8_t *>(table_name),
      shim::safe_name_len(table_name), static_cast<const void *>(dd_table));
}
