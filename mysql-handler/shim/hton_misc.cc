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
// push_to_engine, get_cost_constants are wired on every registered
// handlerton; rotate_encryption_master_key is gated by ENCRYPTION;
// redo_log_set_state is gated by ENGINE_LOG. The three statistics callbacks
// (get_table_statistics, get_index_column_cardinality,
// get_tablespace_statistics) keep their handlerton pointers NULL today — they
// need setter reverse callbacks that are not wired yet. They are deferred,
// not impossible; the bind path is tracked in docs/api/coverage.md.

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
  // and the default trait method returns None which the thunk turns into
  // nullptr — so engines that do not override see the same defaults they
  // saw before the wire.
  hton->get_cost_constants = rusty_hton_get_cost_constants;
}

void rusty_hton_wire_encryption(handlerton *hton) {
  hton->rotate_encryption_master_key = rusty_hton_rotate_encryption_master_key;
}

void rusty_hton_wire_redo_log_set_state(handlerton *hton) {
  hton->redo_log_set_state = rusty_hton_redo_log_set_state;
}
