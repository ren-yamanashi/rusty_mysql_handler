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

// Create-info and metadata overrides (handler.h #149-#153)

#include "binding.hpp"
#include "rust_callbacks.hpp"
#include "sql_string.h"  // String (complete type for ->append)

// Appends len bytes to the MySQL String the shim is about to return, so the
// Rust append_create_info callback can add engine-specific CREATE TABLE text.
extern "C" void mysql__mysql_string__append(void *packet, const uint8_t *bytes,
                                            size_t len) noexcept {
  String *s = static_cast<String *>(packet);
  s->append(reinterpret_cast<const char *>(bytes), len);
}

void RustHandlerBase::update_create_info(HA_CREATE_INFO *create_info) {
  if (rust_ctx_) {
    rust__handler__update_create_info(rust_ctx_,
                                      static_cast<const void *>(create_info));
    return;
  }
  handler::update_create_info(create_info);
}

void RustHandlerBase::append_create_info(String *packet) {
  if (rust_ctx_) {
    rust__handler__append_create_info(rust_ctx_, static_cast<void *>(packet));
    return;
  }
  handler::append_create_info(packet);
}

void RustHandlerBase::use_hidden_primary_key() {
  // The base sets up hidden-primary-key iteration state, so run it first, then
  // notify the engine.
  handler::use_hidden_primary_key();
  if (rust_ctx_) rust__handler__use_hidden_primary_key(rust_ctx_);
}

bool RustHandlerBase::set_ha_share_ref(Handler_share **arg_ha_share) {
  if (rust_ctx_) {
    bool v = false;
    if (rust__handler__set_ha_share_ref(
            rust_ctx_, static_cast<void *>(arg_ha_share), &v))
      return v;
  }
  return handler::set_ha_share_ref(arg_ha_share);
}

int RustHandlerBase::cmp_ref(const uchar *ref1, const uchar *ref2) const {
  if (rust_ctx_) {
    int32_t v = 0;
    if (rust__handler__cmp_ref(rust_ctx_, static_cast<const uint8_t *>(ref1),
                               static_cast<const uint8_t *>(ref2), ref_length,
                               &v))
      return v;
  }
  return handler::cmp_ref(ref1, ref2);
}
