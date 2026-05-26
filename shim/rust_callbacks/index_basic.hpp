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

#ifndef SHIM_RUST_CALLBACKS_INDEX_BASIC_HPP
#define SHIM_RUST_CALLBACKS_INDEX_BASIC_HPP

#include <cstddef>
#include <cstdint>

// Index — basic (handler.h #18-#19, #21, #25-#29). The shim resolves the
// original key_part_map to a leading-bytes length before crossing the FFI, so
// the key arrives as `const uint8_t *` + length; `find_flag` is the raw
// ha_rkey_function integer.
extern "C" {
int32_t rust__handler__index_init(void *ctx, uint32_t idx, bool sorted);
int32_t rust__handler__index_end(void *ctx);
int32_t rust__handler__index_read_map(void *ctx, uint8_t *buf, size_t buf_len,
                                      const uint8_t *key, size_t key_len,
                                      int32_t find_flag);
int32_t rust__handler__index_next(void *ctx, uint8_t *buf, size_t buf_len);
int32_t rust__handler__index_prev(void *ctx, uint8_t *buf, size_t buf_len);
int32_t rust__handler__index_first(void *ctx, uint8_t *buf, size_t buf_len);
int32_t rust__handler__index_last(void *ctx, uint8_t *buf, size_t buf_len);
int32_t rust__handler__index_next_same(void *ctx, uint8_t *buf, size_t buf_len,
                                       const uint8_t *key, size_t key_len);
}

#endif
