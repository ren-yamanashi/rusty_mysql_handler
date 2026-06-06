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

// Engine limit / size overrides (handler.h #71-#76, #85-#86)

#include "binding.hpp"
#include "rust_callbacks.hpp"

// Each override lets the engine supply a limit, falling back to the handler
// base default when it declines so MySQL's built-in limits stay in force.

uint RustHandlerBase::max_supported_record_length() const {
  if (rust_ctx_) {
    uint32_t v = 0;
    if (rust__handler__max_supported_record_length(rust_ctx_, &v)) return v;
  }
  return handler::max_supported_record_length();
}

uint RustHandlerBase::max_supported_keys() const {
  if (rust_ctx_) {
    uint32_t v = 0;
    if (rust__handler__max_supported_keys(rust_ctx_, &v)) return v;
  }
  return handler::max_supported_keys();
}

uint RustHandlerBase::max_supported_key_parts() const {
  if (rust_ctx_) {
    uint32_t v = 0;
    if (rust__handler__max_supported_key_parts(rust_ctx_, &v)) return v;
  }
  return handler::max_supported_key_parts();
}

uint RustHandlerBase::max_supported_key_length() const {
  if (rust_ctx_) {
    uint32_t v = 0;
    if (rust__handler__max_supported_key_length(rust_ctx_, &v)) return v;
  }
  return handler::max_supported_key_length();
}

uint RustHandlerBase::max_supported_key_part_length(
    HA_CREATE_INFO *create_info) const {
  if (rust_ctx_) {
    uint32_t v = 0;
    if (rust__handler__max_supported_key_part_length(
            rust_ctx_, static_cast<const void *>(create_info), &v))
      return v;
  }
  return handler::max_supported_key_part_length(create_info);
}

uint RustHandlerBase::min_record_length(uint options) const {
  if (rust_ctx_) {
    uint32_t v = 0;
    if (rust__handler__min_record_length(rust_ctx_, options, &v)) return v;
  }
  return handler::min_record_length(options);
}

uint RustHandlerBase::extra_rec_buf_length() const {
  if (rust_ctx_) {
    uint32_t v = 0;
    if (rust__handler__extra_rec_buf_length(rust_ctx_, &v)) return v;
  }
  return handler::extra_rec_buf_length();
}

longlong RustHandlerBase::get_memory_buffer_size() const {
  if (rust_ctx_) {
    int64_t v = 0;
    if (rust__handler__memory_buffer_size(rust_ctx_, &v)) return v;
  }
  return handler::get_memory_buffer_size();
}
