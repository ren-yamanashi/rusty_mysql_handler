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

#ifndef SHIM_RUST_CALLBACKS_HTON_BINLOG_HPP
#define SHIM_RUST_CALLBACKS_HTON_BINLOG_HPP

#include <cstddef>
#include <cstdint>

// Engine-level binlog / ACL notification callbacks delegating to the
// registered Rust handlerton singleton. THD crosses as opaque `const void *`;
// byte pointers come with explicit lengths. None of the pointers are retained
// past the call.
extern "C" {
int32_t rust__hton__binlog_func(const void *thd, uint32_t func);
void rust__hton__binlog_log_query(const void *thd, uint32_t command,
                                  const uint8_t *query, size_t query_len,
                                  const uint8_t *db, size_t db_len,
                                  const uint8_t *table, size_t table_len);
void rust__hton__acl_notify(const void *thd);
}

#endif
