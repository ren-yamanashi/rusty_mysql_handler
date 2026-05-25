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

// Bulk-operation overrides (handler.h #39-#46)

#include "binding.hpp"
#include "my_dbug.h"
#include "rust_callbacks.hpp"
#include "sql/table.h"

void RustHandlerBase::start_bulk_insert(ha_rows rows) {
  DBUG_TRACE;
  rust__handler__start_bulk_insert(rust_ctx_, rows);
}

int RustHandlerBase::end_bulk_insert() {
  DBUG_TRACE;
  return rust__handler__end_bulk_insert(rust_ctx_);
}

bool RustHandlerBase::start_bulk_update() {
  DBUG_TRACE;
  return rust__handler__start_bulk_update(rust_ctx_);
}

int RustHandlerBase::exec_bulk_update(uint *dup_key_found) {
  DBUG_TRACE;
  return rust__handler__exec_bulk_update(rust_ctx_, dup_key_found);
}

void RustHandlerBase::end_bulk_update() {
  DBUG_TRACE;
  rust__handler__end_bulk_update(rust_ctx_);
}

int RustHandlerBase::bulk_update_row(const uchar *old_data, uchar *new_data,
                                     uint *dup_key_found) {
  DBUG_TRACE;
  return rust__handler__bulk_update_row(
      rust_ctx_, old_data, table->s->rec_buff_length, new_data,
      table->s->rec_buff_length, dup_key_found);
}

bool RustHandlerBase::start_bulk_delete() {
  DBUG_TRACE;
  return rust__handler__start_bulk_delete(rust_ctx_);
}

int RustHandlerBase::end_bulk_delete() {
  DBUG_TRACE;
  return rust__handler__end_bulk_delete(rust_ctx_);
}
