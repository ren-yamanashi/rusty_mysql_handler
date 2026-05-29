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

// Engine-level lifecycle hooks (handler.h handlerton callbacks #2-#5). These
// thunks match the handlerton function-pointer typedefs and forward to the
// Rust singleton; close_connection additionally uses the handlerton to reach
// the connection's ha_data, the others ignore it (the singleton is global).

#include "binding.hpp"
#include "mysql/plugin.h"
#include "rust_callbacks.hpp"

namespace {
int rusty_hton_close_connection(handlerton *hton, THD *thd) {
  int rc = rust__hton__close_connection(static_cast<const void *>(thd));
  // Safety net: a transaction context still attached here means the connection
  // ended abnormally (KILL / disconnect mid-transaction) before commit/rollback
  // freed it. close_connection only fires when ha_data is set, so free it.
  void *ctx = thd_get_ha_data(thd, hton);
  if (ctx) {
    rust__hton__txn_free(ctx);
    thd_set_ha_data(thd, hton, nullptr);
  }
  return rc;
}
void rusty_hton_kill_connection(handlerton *, THD *thd) {
  rust__hton__kill_connection(static_cast<const void *>(thd));
}
void rusty_hton_pre_dd_shutdown(handlerton *) { rust__hton__pre_dd_shutdown(); }
void rusty_hton_reset_plugin_vars(THD *thd) {
  rust__hton__reset_plugin_vars(static_cast<const void *>(thd));
}
}  // namespace

void rusty_hton_wire_lifecycle(handlerton *hton) {
  hton->close_connection = rusty_hton_close_connection;
  hton->kill_connection = rusty_hton_kill_connection;
  hton->pre_dd_shutdown = rusty_hton_pre_dd_shutdown;
  hton->reset_plugin_vars = rusty_hton_reset_plugin_vars;
}
