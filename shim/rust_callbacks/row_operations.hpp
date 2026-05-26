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

#ifndef SHIM_RUST_CALLBACKS_ROW_OPERATIONS_HPP
#define SHIM_RUST_CALLBACKS_ROW_OPERATIONS_HPP

#include <cstddef>
#include <cstdint>

// Row operations (handler.h #35-#38). Record buffers cross the FFI as
// `const uint8_t *` + length; the engine reads them but must not retain them.
extern "C" {
int32_t rust__handler__write_row(void *ctx, const uint8_t *buf, size_t buf_len);
int32_t rust__handler__update_row(void *ctx, const uint8_t *old, size_t old_len,
                                  const uint8_t *new_row, size_t new_len);
int32_t rust__handler__delete_row(void *ctx, const uint8_t *buf, size_t buf_len);
int32_t rust__handler__delete_all_rows(void *ctx);
}

#endif
