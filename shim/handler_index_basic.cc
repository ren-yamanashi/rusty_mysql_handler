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

// Basic index-scan overrides (handler.h #18-#19, #21, #25-#29)

#include "binding.hpp"
#include "my_dbug.h"
#include "rust_callbacks.hpp"
#include "sql/table.h"

int RustHandlerBase::index_init(uint idx, bool sorted) {
  DBUG_TRACE;
  active_index = idx;
  return rust__handler__index_init(rust_ctx_, idx, sorted);
}

int RustHandlerBase::index_end() {
  DBUG_TRACE;
  active_index = MAX_KEY;
  return rust__handler__index_end(rust_ctx_);
}

int RustHandlerBase::index_read_map(uchar *buf, const uchar *key,
                                    key_part_map keypart_map,
                                    enum ha_rkey_function find_flag) {
  DBUG_TRACE;
  const uint key_len = calculate_key_len(table, active_index, keypart_map);
  return rust__handler__index_read_map(rust_ctx_, buf, table->s->rec_buff_length,
                                       key, key_len,
                                       static_cast<int32_t>(find_flag));
}

int RustHandlerBase::index_next(uchar *buf) {
  DBUG_TRACE;
  return rust__handler__index_next(rust_ctx_, buf, table->s->rec_buff_length);
}

int RustHandlerBase::index_prev(uchar *buf) {
  DBUG_TRACE;
  return rust__handler__index_prev(rust_ctx_, buf, table->s->rec_buff_length);
}

int RustHandlerBase::index_first(uchar *buf) {
  DBUG_TRACE;
  return rust__handler__index_first(rust_ctx_, buf, table->s->rec_buff_length);
}

int RustHandlerBase::index_last(uchar *buf) {
  DBUG_TRACE;
  return rust__handler__index_last(rust_ctx_, buf, table->s->rec_buff_length);
}

int RustHandlerBase::index_next_same(uchar *buf, const uchar *key, uint keylen) {
  DBUG_TRACE;
  return rust__handler__index_next_same(rust_ctx_, buf,
                                        table->s->rec_buff_length, key, keylen);
}
