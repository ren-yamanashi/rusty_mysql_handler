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

#ifndef SHIM_RUST_CALLBACKS_CAPS_HPP
#define SHIM_RUST_CALLBACKS_CAPS_HPP

#include <cstddef>
#include <cstdint>

// Engine capabilities / features (handler.h #77-#84, #87-#89). Each callback
// returns true when the engine overrides the capability (value written through
// the out-pointer) and false to fall back to the handler base default. Enums
// (row_type, ha_key_alg) cross as their raw int value; HA_CREATE_INFO is an
// opaque `const void *`. explain_extra assigns into the shim-owned std::string
// via mysql__std_string__assign (defined in handler_caps.cc).
extern "C" {
bool rust__handler__low_byte_first(void *ctx, bool *out);
bool rust__handler__checksum(void *ctx, uint32_t *out);
bool rust__handler__is_crashed(void *ctx, bool *out);
bool rust__handler__auto_repair(void *ctx, bool *out);
bool rust__handler__primary_key_is_clustered(void *ctx, bool *out);
bool rust__handler__real_row_type(void *ctx, const void *create_info,
                                  int32_t *out);
bool rust__handler__default_index_algorithm(void *ctx, int32_t *out);
bool rust__handler__is_index_algorithm_supported(void *ctx, int32_t key_alg,
                                                 bool *out);
bool rust__handler__record_buffer_wanted(void *ctx, uint64_t *max_rows);
bool rust__handler__explain_extra(void *ctx, void *out);
bool rust__handler__indexes_are_disabled(void *ctx, int32_t *out);
}

#endif
