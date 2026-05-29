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

// Transaction handlerton callbacks (handler.h #6-#8) plus the per-connection
// registration the handler issues from external_lock / start_stmt. The shim
// owns the connection's ha_data slot because it holds the handlerton; the Rust
// side only ever sees the opaque TxnContext pointer.

#include "binding.hpp"
#include "mysql/plugin.h"
#include "rust_callbacks.hpp"
#include "sql/query_options.h"

namespace {
// Commit / rollback free the context and clear ha_data on the real transaction
// boundary (`all`); the statement boundary keeps the context for the rest of
// the transaction.
int rusty_hton_commit(handlerton *hton, THD *thd, bool all) {
  void *ctx = thd_get_ha_data(thd, hton);
  int rc = rust__hton__txn_commit(ctx, all);
  if (all) {
    rust__hton__txn_free(ctx);
    thd_set_ha_data(thd, hton, nullptr);
  }
  return rc;
}

int rusty_hton_rollback(handlerton *hton, THD *thd, bool all) {
  void *ctx = thd_get_ha_data(thd, hton);
  int rc = rust__hton__txn_rollback(ctx, all);
  if (all) {
    rust__hton__txn_free(ctx);
    thd_set_ha_data(thd, hton, nullptr);
  }
  return rc;
}

int rusty_hton_prepare(handlerton *hton, THD *thd, bool all) {
  return rust__hton__txn_prepare(thd_get_ha_data(thd, hton), all);
}
}  // namespace

void rusty_hton_wire_transactions(handlerton *hton) {
  hton->commit = rusty_hton_commit;
  hton->rollback = rusty_hton_rollback;
  hton->prepare = rusty_hton_prepare;
}

void rusty_hton_register_txn(THD *thd, handlerton *ht) {
  if (!thd_get_ha_data(thd, ht)) {
    void *ctx = rust__hton__txn_begin();
    if (!ctx) return;
    thd_set_ha_data(thd, ht, ctx);
  }
  // Register for the statement, and additionally for the whole transaction when
  // the connection is not in autocommit (an explicit BEGIN or autocommit=0), so
  // a later COMMIT / ROLLBACK reaches this engine.
  trans_register_ha(thd, false, ht, nullptr);
  if (thd_test_options(thd, OPTION_NOT_AUTOCOMMIT | OPTION_BEGIN)) {
    trans_register_ha(thd, true, ht, nullptr);
  }
}
