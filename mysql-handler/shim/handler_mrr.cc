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

// Multi-range read overrides (handler.h #64-#67)

#include "binding.hpp"
#include "my_dbug.h"
#include "rust_callbacks.hpp"
#include "sql/table.h"

// The base disk-sweep MRR implementation drives read_range_first/next, which
// are already bound to Rust; fall back to it whenever the engine declines to
// handle the call so an engine relying on defaults keeps a working MRR path.

ha_rows RustHandlerBase::multi_range_read_info_const(
    uint keyno, RANGE_SEQ_IF *seq, void *seq_init_param, uint n_ranges,
    uint *bufsz, uint *flags, bool *force_default_mrr, Cost_estimate *cost) {
  DBUG_TRACE;
  if (rust_ctx_) {
    uint64_t rows = 0;
    if (rust__handler__multi_range_read_info_const(
            rust_ctx_, keyno, static_cast<const void *>(seq), seq_init_param,
            n_ranges, static_cast<const void *>(cost), &rows))
      return rows;
  }
  return handler::multi_range_read_info_const(keyno, seq, seq_init_param,
                                              n_ranges, bufsz, flags,
                                              force_default_mrr, cost);
}

ha_rows RustHandlerBase::multi_range_read_info(uint keyno, uint n_ranges,
                                               uint keys, uint *bufsz,
                                               uint *flags, Cost_estimate *cost) {
  DBUG_TRACE;
  if (rust_ctx_) {
    uint64_t rows = 0;
    if (rust__handler__multi_range_read_info(rust_ctx_, keyno, n_ranges, keys,
                                             static_cast<const void *>(cost),
                                             &rows))
      return rows;
  }
  return handler::multi_range_read_info(keyno, n_ranges, keys, bufsz, flags,
                                        cost);
}

int RustHandlerBase::multi_range_read_init(RANGE_SEQ_IF *seq,
                                           void *seq_init_param, uint n_ranges,
                                           uint mode, HANDLER_BUFFER *buf) {
  DBUG_TRACE;
  if (rust_ctx_) {
    int32_t result = 0;
    if (rust__handler__multi_range_read_init(
            rust_ctx_, static_cast<const void *>(seq), seq_init_param, n_ranges,
            mode, static_cast<const void *>(buf), &result))
      return result;
  }
  return handler::multi_range_read_init(seq, seq_init_param, n_ranges, mode,
                                        buf);
}

int RustHandlerBase::multi_range_read_next(char **range_info) {
  DBUG_TRACE;
  if (rust_ctx_) {
    int32_t result = 0;
    if (rust__handler__multi_range_read_next(
            rust_ctx_, table->record[0], table->s->rec_buff_length,
            reinterpret_cast<void **>(range_info), &result))
      return result;
  }
  return handler::multi_range_read_next(range_info);
}
