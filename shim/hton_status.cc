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

// Status / lifecycle handlerton callbacks (handler.h #36-#44). panic,
// flush_logs, show_status, fill_is_table, upgrade_logs, finish_upgrade and
// is_reserved_db_name are always wired on a registered handlerton;
// start_consistent_snapshot and partition_flags are gated by the corresponding
// capabilities. The reverse callback mysql__hton__emit_status_row lets a Rust
// show_status implementation push rows back through the stat_print_fn MySQL
// handed in.

#include <cstring>

#include "binding.hpp"
#include "my_base.h"
#include "rust_callbacks.hpp"
#include "sql/handler.h"

namespace {
int rusty_hton_panic(handlerton *, enum ha_panic_function flag) {
  return rust__hton__panic(static_cast<uint32_t>(flag));
}

int rusty_hton_start_consistent_snapshot(handlerton *, THD *thd) {
  return rust__hton__start_consistent_snapshot(static_cast<const void *>(thd));
}

bool rusty_hton_flush_logs(handlerton *, bool binlog_group_flush) {
  return rust__hton__flush_logs(binlog_group_flush);
}

bool rusty_hton_show_status(handlerton *, THD *thd, stat_print_fn *print,
                            enum ha_stat_type stat) {
  return rust__hton__show_status(static_cast<const void *>(thd),
                                 reinterpret_cast<const void *>(print),
                                 static_cast<uint32_t>(stat));
}

uint rusty_hton_partition_flags() { return rust__hton__partition_flags(); }

int rusty_hton_fill_is_table(handlerton *, THD *thd, Table_ref *, Item *,
                             enum enum_schema_tables) {
  return rust__hton__fill_is_table(static_cast<const void *>(thd));
}

int rusty_hton_upgrade_logs(THD *thd) {
  return rust__hton__upgrade_logs(static_cast<const void *>(thd));
}

int rusty_hton_finish_upgrade(THD *thd, bool failed_upgrade) {
  return rust__hton__finish_upgrade(static_cast<const void *>(thd),
                                    failed_upgrade);
}

bool rusty_hton_is_reserved_db_name(handlerton *, const char *name) {
  if (!name) return false;
  return rust__hton__is_reserved_db_name(reinterpret_cast<const uint8_t *>(name),
                                         std::strlen(name));
}
}  // namespace

void rusty_hton_wire_status(handlerton *hton) {
  hton->panic = rusty_hton_panic;
  hton->flush_logs = rusty_hton_flush_logs;
  hton->show_status = rusty_hton_show_status;
  hton->fill_is_table = rusty_hton_fill_is_table;
  hton->upgrade_logs = rusty_hton_upgrade_logs;
  hton->finish_upgrade = rusty_hton_finish_upgrade;
  hton->is_reserved_db_name = rusty_hton_is_reserved_db_name;
}

void rusty_hton_wire_consistent_snapshot(handlerton *hton) {
  hton->start_consistent_snapshot = rusty_hton_start_consistent_snapshot;
}

void rusty_hton_wire_partitioning(handlerton *hton) {
  hton->partition_flags = rusty_hton_partition_flags;
}

// Rust → C++ reverse callback: the engine's StatPrintSink::emit lands here.
// stat_print_fn returns true on error in MySQL convention, so propagate it
// unchanged for the Rust side to invert into success-on-true.
extern "C" bool mysql__hton__emit_status_row(const void *thd,
                                             const void *print_fn,
                                             const uint8_t *kind,
                                             size_t kind_len,
                                             const uint8_t *key, size_t key_len,
                                             const uint8_t *value,
                                             size_t value_len) {
  if (!print_fn) return true;
  auto *print = reinterpret_cast<stat_print_fn *>(const_cast<void *>(print_fn));
  return print(const_cast<THD *>(static_cast<const THD *>(thd)),
               reinterpret_cast<const char *>(kind), kind_len,
               reinterpret_cast<const char *>(key), key_len,
               reinterpret_cast<const char *>(value), value_len);
}
