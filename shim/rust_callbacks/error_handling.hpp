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

#ifndef SHIM_RUST_CALLBACKS_ERROR_HANDLING_HPP
#define SHIM_RUST_CALLBACKS_ERROR_HANDLING_HPP

#include <cstddef>
#include <cstdint>

// Error-handling methods (handler.h #114-#118). Each returns true when the
// engine overrides (message/flag written through the out-parameter) and false
// to fall back to the handler base. The MySQL `String` out-buffer crosses as an
// opaque `void *` written via mysql__mysql_string__set.
extern "C" {
bool rust__handler__print_error(void *ctx, int error, uint64_t errflag);
bool rust__handler__get_error_message(void *ctx, int error, void *buf);
bool rust__handler__get_foreign_dup_key(void *ctx, uint8_t *table_buf,
                                        uint32_t table_cap, uint8_t *key_buf,
                                        uint32_t key_cap);
bool rust__handler__is_ignorable_error(void *ctx, int error, bool *out);
bool rust__handler__is_fatal_error(void *ctx, int error, bool *out);

// Implemented in the shim (handler_error_handling.cc): copies len bytes into
// the MySQL String out-buffer for get_error_message.
void mysql__mysql_string__set(void *buf, const uint8_t *bytes,
                              size_t len) noexcept;
}

#endif
