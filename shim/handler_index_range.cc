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

// Index read & range overrides (handler.h #20, #22-#24, #30-#32)

#include "binding.hpp"
#include "my_dbug.h"
#include "rust_callbacks.hpp"
#include "sql/table.h"

namespace {
// (key ptr, length, flag) triple the Rust range callbacks expect; a null
// key_range collapses to an open-ended bound.
struct RangeArgs {
  const uint8_t *key;
  size_t len;
  int32_t flag;
};

RangeArgs range_args(const key_range *range) {
  if (!range) return {nullptr, 0, 0};
  return {range->key, range->length, static_cast<int32_t>(range->flag)};
}
}  // namespace

int RustHandlerBase::index_read(uchar *buf, const uchar *key, uint key_len,
                                enum ha_rkey_function find_flag) {
  DBUG_TRACE;
  return rust__handler__index_read(rust_ctx_, buf, table->s->rec_buff_length,
                                   key, key_len,
                                   static_cast<int32_t>(find_flag));
}

int RustHandlerBase::index_read_idx_map(uchar *buf, uint index,
                                        const uchar *key,
                                        key_part_map keypart_map,
                                        enum ha_rkey_function find_flag) {
  DBUG_TRACE;
  const uint key_len = calculate_key_len(table, index, keypart_map);
  return rust__handler__index_read_idx_map(
      rust_ctx_, buf, table->s->rec_buff_length, index, key, key_len,
      static_cast<int32_t>(find_flag));
}

int RustHandlerBase::index_read_last(uchar *buf, const uchar *key,
                                     uint key_len) {
  DBUG_TRACE;
  return rust__handler__index_read_last(
      rust_ctx_, buf, table->s->rec_buff_length, key, key_len);
}

int RustHandlerBase::index_read_last_map(uchar *buf, const uchar *key,
                                         key_part_map keypart_map) {
  DBUG_TRACE;
  const uint key_len = calculate_key_len(table, active_index, keypart_map);
  return rust__handler__index_read_last_map(
      rust_ctx_, buf, table->s->rec_buff_length, key, key_len);
}

int RustHandlerBase::read_range_first(const key_range *start_key,
                                      const key_range *end_key, bool eq_range,
                                      bool sorted) {
  DBUG_TRACE;
  const RangeArgs start = range_args(start_key);
  const RangeArgs end = range_args(end_key);
  return rust__handler__read_range_first(
      rust_ctx_, table->record[0], table->s->rec_buff_length, start.key,
      start.len, start.flag, end.key, end.len, end.flag, eq_range, sorted);
}

int RustHandlerBase::read_range_next() {
  DBUG_TRACE;
  return rust__handler__read_range_next(rust_ctx_, table->record[0],
                                        table->s->rec_buff_length);
}

ha_rows RustHandlerBase::records_in_range(uint inx, key_range *min_key,
                                          key_range *max_key) {
  DBUG_TRACE;
  const RangeArgs min = range_args(min_key);
  const RangeArgs max = range_args(max_key);
  return rust__handler__records_in_range(rust_ctx_, inx, min.key, min.len,
                                         min.flag, max.key, max.len, max.flag);
}
