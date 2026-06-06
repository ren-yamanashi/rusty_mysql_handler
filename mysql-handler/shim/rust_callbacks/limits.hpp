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

#ifndef SHIM_RUST_CALLBACKS_LIMITS_HPP
#define SHIM_RUST_CALLBACKS_LIMITS_HPP

#include <cstdint>

// Engine limits / sizes (handler.h #71-#76, #85-#86). Each callback returns
// true when the engine overrides the limit (value written through the
// out-pointer) and false to fall back to the handler base default.
// HA_CREATE_INFO crosses as an opaque `const void *`.
extern "C" {
bool rust__handler__max_supported_record_length(void *ctx, uint32_t *out);
bool rust__handler__max_supported_keys(void *ctx, uint32_t *out);
bool rust__handler__max_supported_key_parts(void *ctx, uint32_t *out);
bool rust__handler__max_supported_key_length(void *ctx, uint32_t *out);
bool rust__handler__max_supported_key_part_length(void *ctx,
                                                  const void *create_info,
                                                  uint32_t *out);
bool rust__handler__min_record_length(void *ctx, uint32_t options,
                                      uint32_t *out);
bool rust__handler__extra_rec_buf_length(void *ctx, uint32_t *out);
bool rust__handler__memory_buffer_size(void *ctx, int64_t *out);
}

#endif
