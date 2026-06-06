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

#ifndef SHIM_RUST_CALLBACKS_HTON_MISC_HPP
#define SHIM_RUST_CALLBACKS_HTON_MISC_HPP

#include <cstddef>
#include <cstdint>

// Miscellaneous handlerton callbacks. Wiring split: always-wire for
// is_dict_readonly / rm_tmp_tables / replace_native_transaction_in_thd /
// post_ddl / post_recover / push_to_engine; capability-gated for
// rotate_encryption_master_key (ENCRYPTION) and redo_log_set_state
// (ENGINE_LOG). get_cost_constants and the three statistics callbacks are
// bound here for completeness but their handlerton pointers stay NULL until
// the setter reverse callbacks for SE_cost_constants / ha_statistics /
// ha_tablespace_statistics land.
extern "C" {
bool rust__hton__is_dict_readonly();
bool rust__hton__rm_tmp_tables(const void *thd);
void rust__hton__replace_native_transaction_in_thd(const void *thd,
                                                   void *new_trx_arg,
                                                   void **ptr_trx_arg);
int32_t rust__hton__push_to_engine(const void *thd, const void *query,
                                   const void *join);
void rust__hton__get_cost_constants(uint32_t storage_category);

bool rust__hton__rotate_encryption_master_key();
bool rust__hton__redo_log_set_state(const void *thd, bool enable);
bool rust__hton__get_table_statistics(const uint8_t *db_name, size_t db_name_len,
                                      const uint8_t *table_name,
                                      size_t table_name_len,
                                      uint64_t se_private_id, uint32_t flags);
bool rust__hton__get_index_column_cardinality(
    const uint8_t *db_name, size_t db_name_len, const uint8_t *table_name,
    size_t table_name_len, const uint8_t *index_name, size_t index_name_len,
    uint32_t index_ordinal_position, uint32_t column_ordinal_position,
    uint64_t se_private_id, uint64_t *out_cardinality);
bool rust__hton__get_tablespace_statistics(const uint8_t *tablespace_name,
                                           size_t tablespace_name_len,
                                           const uint8_t *file_name,
                                           size_t file_name_len);
void rust__hton__post_ddl(const void *thd);
void rust__hton__post_recover();
}

#endif
