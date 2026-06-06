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

// Error-handling overrides (handler.h #114-#118)

#include "binding.hpp"
#include "rust_callbacks.hpp"
#include "sql/mysqld_cs.h"  // system_charset_info
#include "sql_string.h"     // String (complete type for ->copy)

// Copies len bytes into the MySQL String the shim is about to inspect, so the
// Rust get_error_message callback can hand an owned message back across the FFI.
extern "C" void mysql__mysql_string__set(void *buf, const uint8_t *bytes,
                                         size_t len) noexcept {
  String *s = static_cast<String *>(buf);
  s->copy(reinterpret_cast<const char *>(bytes), len, system_charset_info);
}

// Each override lets the engine handle the diagnostic, falling back to the
// handler base (non-trivial error classification / formatting) when it declines.

void RustHandlerBase::print_error(int error, myf errflag) {
  if (rust_ctx_ && rust__handler__print_error(rust_ctx_, error,
                                              static_cast<uint64_t>(errflag)))
    return;
  handler::print_error(error, errflag);
}

bool RustHandlerBase::get_error_message(int error, String *buf) {
  if (rust_ctx_) {
    // MySQL detects message presence via a non-empty buf and reads the bool
    // return as "is this a temporary error" (see handler::print_error). Clear
    // buf, let the callback fill it when the engine has a message, and fall back
    // to the base only when nothing was written.
    buf->length(0);
    const bool temporary =
        rust__handler__get_error_message(rust_ctx_, error,
                                         static_cast<void *>(buf));
    if (!buf->is_empty()) return temporary;
  }
  return handler::get_error_message(error, buf);
}

bool RustHandlerBase::get_foreign_dup_key(char *child_table_name,
                                          uint child_table_name_len,
                                          char *child_key_name,
                                          uint child_key_name_len) {
  if (rust_ctx_ &&
      rust__handler__get_foreign_dup_key(
          rust_ctx_, reinterpret_cast<uint8_t *>(child_table_name),
          child_table_name_len, reinterpret_cast<uint8_t *>(child_key_name),
          child_key_name_len))
    return true;
  return handler::get_foreign_dup_key(child_table_name, child_table_name_len,
                                      child_key_name, child_key_name_len);
}

bool RustHandlerBase::is_ignorable_error(int error) {
  if (rust_ctx_) {
    bool v = false;
    if (rust__handler__is_ignorable_error(rust_ctx_, error, &v)) return v;
  }
  return handler::is_ignorable_error(error);
}

bool RustHandlerBase::is_fatal_error(int error) {
  if (rust_ctx_) {
    bool v = false;
    if (rust__handler__is_fatal_error(rust_ctx_, error, &v)) return v;
  }
  return handler::is_fatal_error(error);
}
