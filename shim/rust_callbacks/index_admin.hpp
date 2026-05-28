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

#ifndef SHIM_RUST_CALLBACKS_INDEX_ADMIN_HPP
#define SHIM_RUST_CALLBACKS_INDEX_ADMIN_HPP

#include <cstdint>

// Key-cache and tablespace admin commands (handler.h #136-#140). Each returns
// true when the engine overrides (the raw handler code written through the
// out-pointer) and false to fall back to the handler base. THD, HA_CHECK_OPT
// and dd::Table cross as opaque `const void *`.
extern "C" {
bool rust__handler__assign_to_keycache(void *ctx, const void *thd,
                                       const void *check_opt, int32_t *out);
bool rust__handler__preload_keys(void *ctx, const void *thd,
                                 const void *check_opt, int32_t *out);
bool rust__handler__disable_indexes(void *ctx, uint32_t mode, int32_t *out);
bool rust__handler__enable_indexes(void *ctx, uint32_t mode, int32_t *out);
bool rust__handler__discard_or_import_tablespace(void *ctx, bool discard,
                                                 const void *table_def,
                                                 int32_t *out);
}

#endif
