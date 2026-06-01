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

#ifndef SHIM_RUST_CALLBACKS_HTON_TRANSACTIONS_HPP
#define SHIM_RUST_CALLBACKS_HTON_TRANSACTIONS_HPP

#include <cstddef>
#include <cstdint>

// Per-connection transaction lifecycle. The shim owns the connection's
// `ha_data` slot (it has the handlerton); the context pointer below is the
// opaque Rust TxnContext stored there, passed back on every commit / rollback.
extern "C" {
void *rust__hton__txn_begin();
int32_t rust__hton__txn_commit(void *ctx, bool all);
int32_t rust__hton__txn_rollback(void *ctx, bool all);
int32_t rust__hton__txn_prepare(void *ctx, bool all);
// Stage a row write (table name + row image) into the transaction context
int32_t rust__hton__txn_write_row(void *ctx, const uint8_t *table,
                                  size_t table_len, const uint8_t *row,
                                  size_t row_len);
// Stage a row update: old + new row images.
int32_t rust__hton__txn_update_row(void *ctx, const uint8_t *table,
                                   size_t table_len, const uint8_t *old,
                                   size_t old_len, const uint8_t *new_row,
                                   size_t new_len);
// Stage a row delete: pre-image of the row to remove.
int32_t rust__hton__txn_delete_row(void *ctx, const uint8_t *table,
                                   size_t table_len, const uint8_t *row,
                                   size_t row_len);
void rust__hton__txn_free(void *ctx);
}

#endif
