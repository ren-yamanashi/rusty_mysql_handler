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
// rm_tmp_tables, replace_native_transaction_in_thd, post_ddl, post_recover
// are wired on every registered handlerton; rotate_encryption_master_key is
// gated by ENCRYPTION; redo_log_set_state is gated by ENGINE_LOG. The
// five output-shaped callbacks (push_to_engine, get_cost_constants, the
// three statistics callbacks) keep their handlerton pointers NULL — the
// engine-owned outputs they return cannot be synthesised through the opaque
// pass-through. get_index_column_cardinality round-trips `ulonglong *` via
// a local uint64_t for LP64 safety.

#include "binding.hpp"
#include "rust_callbacks.hpp"
#include "sql/handler.h"

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
}

void rusty_hton_wire_encryption(handlerton *hton) {
  hton->rotate_encryption_master_key = rusty_hton_rotate_encryption_master_key;
}

void rusty_hton_wire_redo_log_set_state(handlerton *hton) {
  hton->redo_log_set_state = rusty_hton_redo_log_set_state;
}
