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

// Binlog / ACL notification handlerton callbacks (handler.h #49-51). All
// always wired on a registered handlerton — they are notifications, not
// gating. binlog_func's void* arg is opaque on both sides; binlog_log_query's
// query / db / table are bounded reads that the Rust side never logs.

#include <cstring>

#include "binding.hpp"
#include "rust_callbacks.hpp"
#include "sql/handler.h"

namespace {
// FfiPtr::bytes_to_str's SAFETY contract requires a non-null pointer even when
// length == 0 (slice::from_raw_parts rejects null), so substitute a non-null
// empty-string sentinel whenever MySQL hands us NULL. binlog_log_query is the
// reachable case — LOGCOM_CREATE_DB-style events legitimately pass NULL for
// db/table/query — but the same conversion is applied wherever the shim sees
// an optional C string.
const uint8_t *nz(const char *s) {
  return reinterpret_cast<const uint8_t *>(s ? s : "");
}

int rusty_hton_binlog_func(handlerton *, THD *thd, enum_binlog_func func,
                           void *) {
  return rust__hton__binlog_func(static_cast<const void *>(thd),
                                 static_cast<uint32_t>(func));
}

void rusty_hton_binlog_log_query(handlerton *, THD *thd,
                                 enum_binlog_command command, const char *query,
                                 uint query_length, const char *db,
                                 const char *table) {
  rust__hton__binlog_log_query(
      static_cast<const void *>(thd), static_cast<uint32_t>(command),
      nz(query), query ? query_length : 0u,
      nz(db), db ? std::strlen(db) : 0u,
      nz(table), table ? std::strlen(table) : 0u);
}

void rusty_hton_acl_notify(THD *thd, const Acl_change_notification *) {
  rust__hton__acl_notify(static_cast<const void *>(thd));
}
}  // namespace

void rusty_hton_wire_binlog(handlerton *hton) {
  hton->binlog_func = rusty_hton_binlog_func;
  hton->binlog_log_query = rusty_hton_binlog_log_query;
  hton->acl_notify = rusty_hton_acl_notify;
}
