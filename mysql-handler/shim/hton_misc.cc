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

// Miscellaneous handlerton callbacks (handler.h #78-#93). is_dict_readonly,
// rm_tmp_tables, replace_native_transaction_in_thd, post_ddl, post_recover,
// push_to_engine, get_cost_constants, get_table_statistics are wired on every
// registered handlerton; rotate_encryption_master_key is gated by ENCRYPTION;
// redo_log_set_state is gated by ENGINE_LOG. The remaining two statistics
// callbacks (get_index_column_cardinality, get_tablespace_statistics) keep
// their handlerton pointers NULL today — they need setter reverse callbacks
// that are not wired yet. They are deferred, not impossible; the bind path
// is tracked in docs/api/coverage.md.

#include <cstring>

#include "binding.hpp"
#include "rust_callbacks.hpp"
#include "sql/handler.h"
#include "sql/opt_costconstants.h"

namespace {
bool rusty_hton_is_dict_readonly() { return rust__hton__is_dict_readonly(); }

bool rusty_hton_rm_tmp_tables(handlerton *, THD *thd, List<LEX_STRING> *) {
  return rust__hton__rm_tmp_tables(static_cast<const void *>(thd));
}

void rusty_hton_replace_native_transaction_in_thd(THD *thd, void *new_trx_arg,
                                                  void **ptr_trx_arg) {
  rust__hton__replace_native_transaction_in_thd(static_cast<const void *>(thd),
                                                new_trx_arg, ptr_trx_arg);
}

bool rusty_hton_rotate_encryption_master_key() {
  return rust__hton__rotate_encryption_master_key();
}

bool rusty_hton_redo_log_set_state(THD *thd, bool enable) {
  return rust__hton__redo_log_set_state(static_cast<const void *>(thd), enable);
}

void rusty_hton_post_ddl(THD *thd) {
  rust__hton__post_ddl(static_cast<const void *>(thd));
}

void rusty_hton_post_recover() { rust__hton__post_recover(); }

int rusty_hton_push_to_engine(THD *thd, AccessPath *query, JOIN *join) {
  return rust__hton__push_to_engine(static_cast<const void *>(thd),
                                    static_cast<const void *>(query),
                                    static_cast<const void *>(join));
}

// Subclass of SE_cost_constants that lets us push values through the
// upstream-protected update() setter. update() looks the name up
// case-insensitively, so the bare lowercase form is enough.
class RustySECostConstants : public SE_cost_constants {
 public:
  RustySECostConstants(Optimizer opt, double mem_cost, double io_cost)
      : SE_cost_constants(opt) {
    update({"memory_block_read_cost", 22}, mem_cost);
    update({"io_block_read_cost", 18}, io_cost);
  }
};

bool rusty_hton_get_table_statistics(const char *db_name,
                                     const char *table_name,
                                     dd::Object_id se_private_id,
                                     const dd::Properties &,
                                     const dd::Properties &, uint flags,
                                     ha_statistics *stats) {
  if (!db_name || !table_name || !stats) return true;
  return rust__hton__get_table_statistics(
      reinterpret_cast<const uint8_t *>(db_name), std::strlen(db_name),
      reinterpret_cast<const uint8_t *>(table_name), std::strlen(table_name),
      static_cast<uint64_t>(se_private_id), static_cast<uint32_t>(flags),
      static_cast<void *>(stats));
}

SE_cost_constants *rusty_hton_get_cost_constants(uint storage_category) {
  double mem_cost = 0.0;
  double io_cost = 0.0;
  if (!rust__hton__get_cost_constants(static_cast<uint32_t>(storage_category),
                                      &mem_cost, &io_cost)) {
    return nullptr;
  }
  if (mem_cost <= 0.0 || io_cost <= 0.0) {
    // Match SE_cost_constants::set's INVALID_COST_VALUE guard rather than
    // silently passing a zero/negative cost through to the optimizer.
    return nullptr;
  }
  // The optimizer kind is not threaded through the handlerton callback by
  // upstream, and the only constructor option that does not reference
  // private state we cannot see is kOriginal. The kHypergraph case applies
  // the same defaults anyway today, so this is safe.
  return new RustySECostConstants(Optimizer::kOriginal, mem_cost, io_cost);
}
}  // namespace

// Reverse callback: copy each `TableStatistics` field into the matching
// `ha_statistics` slot. The Rust side hands per-field values rather than a
// shared struct layout, so the casts here are the only place that knows
// about ha_statistics's mixed `ha_rows` / `ulong` / `time_t` types.
extern "C" void mysql__hton__set_table_statistics(
    void *stats_void, uint64_t records, uint64_t data_file_length,
    uint64_t max_data_file_length, uint64_t index_file_length,
    uint64_t max_index_file_length, uint64_t delete_length,
    uint64_t auto_increment_value, uint64_t deleted, uint64_t mean_rec_length,
    int64_t create_time, uint64_t check_time, uint64_t update_time,
    uint32_t block_size) {
  if (!stats_void) return;
  auto *stats = static_cast<ha_statistics *>(stats_void);
  stats->records = static_cast<ha_rows>(records);
  stats->data_file_length = data_file_length;
  stats->max_data_file_length = max_data_file_length;
  stats->index_file_length = index_file_length;
  stats->max_index_file_length = max_index_file_length;
  stats->delete_length = delete_length;
  stats->auto_increment_value = auto_increment_value;
  stats->deleted = static_cast<ha_rows>(deleted);
  stats->mean_rec_length = static_cast<ulong>(mean_rec_length);
  stats->create_time = static_cast<time_t>(create_time);
  stats->check_time = static_cast<ulong>(check_time);
  stats->update_time = static_cast<ulong>(update_time);
  stats->block_size = block_size;
}

void rusty_hton_wire_misc(handlerton *hton) {
  hton->is_dict_readonly = rusty_hton_is_dict_readonly;
  hton->rm_tmp_tables = rusty_hton_rm_tmp_tables;
  // replace_native_transaction_in_thd intentionally stays NULL: the upstream
  // XA slave-applier detach / reattach (sql/xa.cc) swaps the native txn
  // pointer through `ptr_trx_arg`, but the trait method drops both that
  // out-param and `new_trx_arg` today. A non-NULL wire would let MySQL call
  // a stub that ignores the swap and corrupts XA state, while NULL makes
  // MySQL's null-checked path skip the swap entirely.
  hton->post_ddl = rusty_hton_post_ddl;
  hton->post_recover = rusty_hton_post_recover;
  // Safe to wire unconditionally: the optimizer in sql_optimizer.cc only
  // dereferences `hton->push_to_engine` after the handler-level
  // `hton_supporting_engine_pushdown()` returns non-NULL, so engines that
  // do not opt into pushdown never see this callback fire.
  hton->push_to_engine = rusty_hton_push_to_engine;
  // Safe to wire unconditionally: the optimizer falls back to
  // `new SE_cost_constants(optimizer)` when the callback returns nullptr,
  // and the default trait method returns None → nullptr, so engines that
  // do not override observe MySQL's stock cost defaults.
  hton->get_cost_constants = rusty_hton_get_cost_constants;
  // Safe to wire unconditionally: MySQL only consults this callback when
  // populating I_S queries, the default trait returns `Ok(None)` which the
  // thunk turns into `true` (failure), and MySQL handles the failure as
  // "engine has no stats for this table". Engines that override fill the
  // `ha_statistics` slot through the setter reverse callback above.
  hton->get_table_statistics = rusty_hton_get_table_statistics;
}

void rusty_hton_wire_encryption(handlerton *hton) {
  hton->rotate_encryption_master_key = rusty_hton_rotate_encryption_master_key;
}

void rusty_hton_wire_redo_log_set_state(handlerton *hton) {
  hton->redo_log_set_state = rusty_hton_redo_log_set_state;
}
