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

#ifndef SHIM_RUST_CALLBACKS_INDEX_RANGE_HPP
#define SHIM_RUST_CALLBACKS_INDEX_RANGE_HPP

#include <cstddef>
#include <cstdint>

// Index — read & range (handler.h #20, #22-#24, #30-#32). Keys arrive
// length-resolved as `const uint8_t *` + length (a null key => nullptr);
// find_flag is the raw ha_rkey_function integer. Each range bound decomposes
// its key_range into (key ptr, length, flag); a null bound denotes an open
// end. records_in_range returns the estimated row count, or HA_POS_ERROR
// (~uint64_t{0}) when the engine cannot estimate.
extern "C" {
int32_t rust__handler__index_read(void *ctx, uint8_t *buf, size_t buf_len,
                                  const uint8_t *key, size_t key_len,
                                  int32_t find_flag);
int32_t rust__handler__index_read_idx_map(void *ctx, uint8_t *buf,
                                          size_t buf_len, uint32_t index,
                                          const uint8_t *key, size_t key_len,
                                          int32_t find_flag);
int32_t rust__handler__index_read_last(void *ctx, uint8_t *buf, size_t buf_len,
                                       const uint8_t *key, size_t key_len);
int32_t rust__handler__index_read_last_map(void *ctx, uint8_t *buf,
                                           size_t buf_len, const uint8_t *key,
                                           size_t key_len);
int32_t rust__handler__read_range_first(void *ctx, uint8_t *buf, size_t buf_len,
                                        const uint8_t *start_key,
                                        size_t start_len, int32_t start_flag,
                                        const uint8_t *end_key, size_t end_len,
                                        int32_t end_flag, bool eq_range,
                                        bool sorted);
int32_t rust__handler__read_range_next(void *ctx, uint8_t *buf, size_t buf_len);
uint64_t rust__handler__records_in_range(void *ctx, uint32_t inx,
                                         const uint8_t *min_key, size_t min_len,
                                         int32_t min_flag,
                                         const uint8_t *max_key, size_t max_len,
                                         int32_t max_flag);
}

#endif
