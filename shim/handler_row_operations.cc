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

// Row-operation overrides (handler.h #35-#38)

#include "binding.hpp"
#include "my_dbug.h"
#include "mysql/plugin.h"
#include "rust_callbacks.hpp"
#include "sql/table.h"

int RustHandlerBase::write_row(uchar *buf) {
  DBUG_TRACE;
  void *txn =
      rust__hton__is_transactional() ? thd_get_ha_data(ha_thd(), ht) : nullptr;
  // Non-transactional, or no transaction context yet: the row goes straight to
  // the per-table engine.
  if (!txn) {
    return rust__handler__write_row(rust_ctx_, buf, table->s->rec_buff_length);
  }
  // Transactional: the row joins the connection's transaction so commit /
  // rollback decide its fate.
  return rust__hton__txn_write_row(
      txn, reinterpret_cast<const uint8_t *>(table->s->table_name.str),
      table->s->table_name.length, buf, table->s->rec_buff_length);
}

int RustHandlerBase::update_row(const uchar *old_data, uchar *new_data) {
  DBUG_TRACE;
  void *txn =
      rust__hton__is_transactional() ? thd_get_ha_data(ha_thd(), ht) : nullptr;
  // Transactional path: stage so commit / rollback decides the row's fate.
  if (txn) {
    return rust__hton__txn_update_row(
        txn, reinterpret_cast<const uint8_t *>(table->s->table_name.str),
        table->s->table_name.length, old_data, table->s->rec_buff_length,
        new_data, table->s->rec_buff_length);
  }
  return rust__handler__update_row(rust_ctx_, old_data,
                                   table->s->rec_buff_length, new_data,
                                   table->s->rec_buff_length);
}

int RustHandlerBase::delete_row(const uchar *buf) {
  DBUG_TRACE;
  void *txn =
      rust__hton__is_transactional() ? thd_get_ha_data(ha_thd(), ht) : nullptr;
  if (txn) {
    return rust__hton__txn_delete_row(
        txn, reinterpret_cast<const uint8_t *>(table->s->table_name.str),
        table->s->table_name.length, buf, table->s->rec_buff_length);
  }
  return rust__handler__delete_row(rust_ctx_, buf, table->s->rec_buff_length);
}

int RustHandlerBase::delete_all_rows() {
  DBUG_TRACE;
  return rust__handler__delete_all_rows(rust_ctx_);
}
