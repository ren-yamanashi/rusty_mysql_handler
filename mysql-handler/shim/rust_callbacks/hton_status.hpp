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

#ifndef SHIM_RUST_CALLBACKS_HTON_STATUS_HPP
#define SHIM_RUST_CALLBACKS_HTON_STATUS_HPP

#include <cstddef>
#include <cstdint>

// Engine-level status / lifecycle hooks delegating to the registered Rust
// handlerton singleton. THD crosses as opaque `const void *`; print_fn is the
// stat_print_fn pointer MySQL handed in, also opaque to Rust. None are retained
// past the call.
extern "C" {
int32_t rust__hton__panic(uint32_t flag);
int32_t rust__hton__start_consistent_snapshot(const void *thd);
bool rust__hton__flush_logs(bool binlog_group_flush);
bool rust__hton__show_status(const void *thd, const void *print_fn,
                             uint32_t stat);
uint32_t rust__hton__partition_flags();
int32_t rust__hton__fill_is_table(const void *thd);
int32_t rust__hton__upgrade_logs(const void *thd);
int32_t rust__hton__finish_upgrade(const void *thd, bool failed_upgrade);
bool rust__hton__is_reserved_db_name(const uint8_t *name, size_t name_len);
}

#endif
