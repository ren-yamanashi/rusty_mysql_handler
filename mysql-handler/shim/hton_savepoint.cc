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

// Savepoint handlerton callbacks (handler.h #15-#18). Each forwards the
// connection's TxnContext (from ha_data) and MySQL's per-savepoint scratch
// (sv) to Rust. MySQL repurposes hton->savepoint_offset to an offset after
// init, so the sv length comes from the stable Rust-side accessor instead.

#include "binding.hpp"
#include "mysql/plugin.h"
#include "rust_callbacks.hpp"

namespace {
int rusty_hton_savepoint_set(handlerton *hton, THD *thd, void *sv) {
  return rust__hton__savepoint_set(thd_get_ha_data(thd, hton),
                                   static_cast<uint8_t *>(sv),
                                   rust__hton__savepoint_offset());
}

int rusty_hton_savepoint_rollback(handlerton *hton, THD *thd, void *sv) {
  return rust__hton__savepoint_rollback(thd_get_ha_data(thd, hton),
                                        static_cast<const uint8_t *>(sv),
                                        rust__hton__savepoint_offset());
}

int rusty_hton_savepoint_release(handlerton *hton, THD *thd, void *sv) {
  return rust__hton__savepoint_release(thd_get_ha_data(thd, hton),
                                       static_cast<const uint8_t *>(sv),
                                       rust__hton__savepoint_offset());
}

bool rusty_hton_savepoint_can_release_mdl(handlerton *hton, THD *thd) {
  return rust__hton__savepoint_can_release_mdl(thd_get_ha_data(thd, hton));
}
}  // namespace

void rusty_hton_wire_savepoints(handlerton *hton) {
  hton->savepoint_offset = rust__hton__savepoint_offset();
  hton->savepoint_set = rusty_hton_savepoint_set;
  hton->savepoint_rollback = rusty_hton_savepoint_rollback;
  hton->savepoint_rollback_can_release_mdl = rusty_hton_savepoint_can_release_mdl;
  hton->savepoint_release = rusty_hton_savepoint_release;
}
