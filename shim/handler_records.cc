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

// Row-count overrides (handler.h #99-#102)

#include "binding.hpp"
#include "rust_callbacks.hpp"

// records / records_from_index let the engine supply an exact count, falling
// back to the handler base scan when the engine declines (handled == false).

int RustHandlerBase::records(ha_rows *num_rows) {
  if (rust_ctx_) {
    uint64_t n = 0;
    bool handled = false;
    int rc = rust__handler__records(rust_ctx_, &n, &handled);
    if (handled) {
      if (rc == 0) *num_rows = n;
      return rc;
    }
  }
  return handler::records(num_rows);
}

int RustHandlerBase::records_from_index(ha_rows *num_rows, uint index) {
  if (rust_ctx_) {
    uint64_t n = 0;
    bool handled = false;
    int rc = rust__handler__records_from_index(rust_ctx_, index, &n, &handled);
    if (handled) {
      if (rc == 0) *num_rows = n;
      return rc;
    }
  }
  return handler::records_from_index(num_rows, index);
}

ha_rows RustHandlerBase::estimate_rows_upper_bound() {
  if (rust_ctx_) {
    uint64_t n = 0;
    if (rust__handler__estimate_rows_upper_bound(rust_ctx_, &n)) return n;
  }
  return handler::estimate_rows_upper_bound();
}

uint32 RustHandlerBase::calculate_key_hash_value(Field **field_array) {
  if (rust_ctx_) {
    uint32_t v = 0;
    if (rust__handler__calculate_key_hash_value(
            rust_ctx_, static_cast<const void *>(field_array), &v))
      return v;
  }
  return handler::calculate_key_hash_value(field_array);
}
