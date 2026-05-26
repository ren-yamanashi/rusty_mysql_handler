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

#ifndef SHIM_RUST_CALLBACKS_BULK_OPERATIONS_HPP
#define SHIM_RUST_CALLBACKS_BULK_OPERATIONS_HPP

#include <cstddef>
#include <cstdint>

// Bulk operations (handler.h #39-#46). The start_bulk_* callbacks return the
// inverted MySQL bool (true => bulk not used, normal per-row operation);
// exec_bulk_update / bulk_update_row write the duplicate-key count through
// dup_key_found. Record buffers cross as `const uint8_t *` + length and must
// not be retained by the engine.
extern "C" {
void rust__handler__start_bulk_insert(void *ctx, uint64_t rows);
int32_t rust__handler__end_bulk_insert(void *ctx);
bool rust__handler__start_bulk_update(void *ctx);
int32_t rust__handler__exec_bulk_update(void *ctx, uint32_t *dup_key_found);
void rust__handler__end_bulk_update(void *ctx);
int32_t rust__handler__bulk_update_row(void *ctx, const uint8_t *old,
                                       size_t old_len, const uint8_t *new_row,
                                       size_t new_len, uint32_t *dup_key_found);
bool rust__handler__start_bulk_delete(void *ctx);
int32_t rust__handler__end_bulk_delete(void *ctx);
}

#endif
