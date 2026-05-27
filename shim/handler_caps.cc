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

// Engine capability / feature overrides (handler.h #77-#84, #87-#89)

#include <string>

#include "binding.hpp"
#include "rust_callbacks.hpp"

// Lets the Rust explain_extra callback hand an owned string back across the
// FFI: it copies len bytes into the std::string the shim is about to return.
extern "C" void mysql__std_string__assign(void *s, const uint8_t *bytes,
                                          size_t len) noexcept {
  static_cast<std::string *>(s)->assign(reinterpret_cast<const char *>(bytes),
                                        len);
}

// Each override lets the engine supply a capability, falling back to the
// handler base default when it declines.

bool RustHandlerBase::low_byte_first() const {
  if (rust_ctx_) {
    bool v = false;
    if (rust__handler__low_byte_first(rust_ctx_, &v)) return v;
  }
  return handler::low_byte_first();
}

ha_checksum RustHandlerBase::checksum() const {
  if (rust_ctx_) {
    uint32_t v = 0;
    if (rust__handler__checksum(rust_ctx_, &v)) return v;
  }
  return handler::checksum();
}

bool RustHandlerBase::is_crashed() const {
  if (rust_ctx_) {
    bool v = false;
    if (rust__handler__is_crashed(rust_ctx_, &v)) return v;
  }
  return handler::is_crashed();
}

bool RustHandlerBase::auto_repair() const {
  if (rust_ctx_) {
    bool v = false;
    if (rust__handler__auto_repair(rust_ctx_, &v)) return v;
  }
  return handler::auto_repair();
}

bool RustHandlerBase::primary_key_is_clustered() const {
  if (rust_ctx_) {
    bool v = false;
    if (rust__handler__primary_key_is_clustered(rust_ctx_, &v)) return v;
  }
  return handler::primary_key_is_clustered();
}

enum row_type RustHandlerBase::get_real_row_type(
    const HA_CREATE_INFO *create_info) const {
  if (rust_ctx_) {
    int32_t v = 0;
    if (rust__handler__real_row_type(
            rust_ctx_, static_cast<const void *>(create_info), &v))
      return static_cast<enum row_type>(v);
  }
  return handler::get_real_row_type(create_info);
}

enum ha_key_alg RustHandlerBase::get_default_index_algorithm() const {
  if (rust_ctx_) {
    int32_t v = 0;
    if (rust__handler__default_index_algorithm(rust_ctx_, &v))
      return static_cast<enum ha_key_alg>(v);
  }
  return handler::get_default_index_algorithm();
}

bool RustHandlerBase::is_index_algorithm_supported(
    enum ha_key_alg key_alg) const {
  if (rust_ctx_) {
    bool v = false;
    if (rust__handler__is_index_algorithm_supported(
            rust_ctx_, static_cast<int32_t>(key_alg), &v))
      return v;
  }
  return handler::is_index_algorithm_supported(key_alg);
}

bool RustHandlerBase::is_record_buffer_wanted(ha_rows *max_rows) const {
  if (rust_ctx_) {
    uint64_t n = 0;
    if (rust__handler__record_buffer_wanted(rust_ctx_, &n)) {
      *max_rows = n;
      return true;
    }
  }
  // handler::is_record_buffer_wanted is private (NVI); reproduce its trivial
  // base behaviour (no buffer wanted) directly.
  *max_rows = 0;
  return false;
}

std::string RustHandlerBase::explain_extra() const {
  if (rust_ctx_) {
    std::string out;
    if (rust__handler__explain_extra(rust_ctx_, &out)) return out;
  }
  return handler::explain_extra();
}

int RustHandlerBase::indexes_are_disabled() {
  if (rust_ctx_) {
    int32_t v = 0;
    if (rust__handler__indexes_are_disabled(rust_ctx_, &v)) return v;
  }
  return handler::indexes_are_disabled();
}
