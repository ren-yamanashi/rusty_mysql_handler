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

// Event-notification handlerton callbacks (handler.h #75-77, #83-86). All are
// always wired on a registered handlerton — notifications are never gating, so
// a stub forwarding to Rust never falsely advertises a capability. The MDL
// hooks return bool with MySQL's "true = veto / error" convention, which the
// Rust side already produces via result_to_veto.

#include <cstring>

#include "binding.hpp"
#include "rust_callbacks.hpp"
#include "sql/handler.h"

namespace {
size_t safe_len(const char *s) { return s ? std::strlen(s) : 0; }

void rusty_hton_notify_after_select(THD *thd, SelectExecutedIn executed_in) {
  rust__hton__notify_after_select(static_cast<const void *>(thd),
                                  executed_in == SelectExecutedIn::kSecondaryEngine);
}

void rusty_hton_notify_create_table(HA_CREATE_INFO *, const char *db,
                                    const char *table_name) {
  if (!db || !table_name) return;
  rust__hton__notify_create_table(reinterpret_cast<const uint8_t *>(db),
                                  std::strlen(db),
                                  reinterpret_cast<const uint8_t *>(table_name),
                                  std::strlen(table_name));
}

void rusty_hton_notify_drop_table(Table_ref *) { rust__hton__notify_drop_table(); }

bool rusty_hton_notify_exclusive_mdl(THD *thd, const MDL_key *mdl_key,
                                     ha_notification_type kind,
                                     bool *victimized) {
  // The trivial / default engine never victimizes; let MySQL keep its
  // pre-initialised value if the caller did not zero it.
  if (victimized) *victimized = false;
  return rust__hton__notify_exclusive_mdl(static_cast<const void *>(thd),
                                          static_cast<const void *>(mdl_key),
                                          static_cast<int32_t>(kind));
}

bool rusty_hton_notify_alter_table(THD *thd, const MDL_key *mdl_key,
                                   ha_notification_type kind) {
  return rust__hton__notify_alter_table(static_cast<const void *>(thd),
                                        static_cast<const void *>(mdl_key),
                                        static_cast<int32_t>(kind));
}

bool rusty_hton_notify_rename_table(THD *thd, const MDL_key *mdl_key,
                                    ha_notification_type kind,
                                    const char *old_db, const char *old_name,
                                    const char *new_db, const char *new_name) {
  return rust__hton__notify_rename_table(
      static_cast<const void *>(thd), static_cast<const void *>(mdl_key),
      static_cast<int32_t>(kind),
      reinterpret_cast<const uint8_t *>(old_db), safe_len(old_db),
      reinterpret_cast<const uint8_t *>(old_name), safe_len(old_name),
      reinterpret_cast<const uint8_t *>(new_db), safe_len(new_db),
      reinterpret_cast<const uint8_t *>(new_name), safe_len(new_name));
}

bool rusty_hton_notify_truncate_table(THD *thd, const MDL_key *mdl_key,
                                      ha_notification_type kind) {
  return rust__hton__notify_truncate_table(static_cast<const void *>(thd),
                                           static_cast<const void *>(mdl_key),
                                           static_cast<int32_t>(kind));
}
}  // namespace

void rusty_hton_wire_notifications(handlerton *hton) {
  hton->notify_after_select = rusty_hton_notify_after_select;
  hton->notify_create_table = rusty_hton_notify_create_table;
  hton->notify_drop_table = rusty_hton_notify_drop_table;
  hton->notify_exclusive_mdl = rusty_hton_notify_exclusive_mdl;
  hton->notify_alter_table = rusty_hton_notify_alter_table;
  hton->notify_rename_table = rusty_hton_notify_rename_table;
  hton->notify_truncate_table = rusty_hton_notify_truncate_table;
}
