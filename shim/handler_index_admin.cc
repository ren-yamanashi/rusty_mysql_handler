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

// Key-cache and tablespace admin overrides (handler.h #136-#140)

#include "binding.hpp"
#include "rust_callbacks.hpp"

// Each override lets the engine handle the command, falling back to the public
// handler:: base when it declines.

int RustHandlerBase::assign_to_keycache(THD *thd, HA_CHECK_OPT *check_opt) {
  if (rust_ctx_) {
    int32_t v = 0;
    if (rust__handler__assign_to_keycache(rust_ctx_,
                                          static_cast<const void *>(thd),
                                          static_cast<const void *>(check_opt),
                                          &v))
      return v;
  }
  return handler::assign_to_keycache(thd, check_opt);
}

int RustHandlerBase::preload_keys(THD *thd, HA_CHECK_OPT *check_opt) {
  if (rust_ctx_) {
    int32_t v = 0;
    if (rust__handler__preload_keys(rust_ctx_, static_cast<const void *>(thd),
                                    static_cast<const void *>(check_opt), &v))
      return v;
  }
  return handler::preload_keys(thd, check_opt);
}

int RustHandlerBase::disable_indexes(uint mode) {
  if (rust_ctx_) {
    int32_t v = 0;
    if (rust__handler__disable_indexes(rust_ctx_, mode, &v)) return v;
  }
  return handler::disable_indexes(mode);
}

int RustHandlerBase::enable_indexes(uint mode) {
  if (rust_ctx_) {
    int32_t v = 0;
    if (rust__handler__enable_indexes(rust_ctx_, mode, &v)) return v;
  }
  return handler::enable_indexes(mode);
}

int RustHandlerBase::discard_or_import_tablespace(bool discard,
                                                  dd::Table *table_def) {
  if (rust_ctx_) {
    int32_t v = 0;
    if (rust__handler__discard_or_import_tablespace(
            rust_ctx_, discard, static_cast<const void *>(table_def), &v))
      return v;
  }
  return handler::discard_or_import_tablespace(discard, table_def);
}
