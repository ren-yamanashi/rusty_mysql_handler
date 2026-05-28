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

#ifndef SHIM_RUST_CALLBACKS_METADATA_HPP
#define SHIM_RUST_CALLBACKS_METADATA_HPP

#include <cstddef>
#include <cstdint>

// Create-info and metadata methods (handler.h #149-#153). HA_CREATE_INFO,
// String and Handler_share** cross as opaque pointers. set_ha_share_ref / cmp_ref
// return true when the engine overrides (value written through the out-pointer)
// and false to fall back to the handler base.
extern "C" {
void rust__handler__update_create_info(void *ctx, const void *create_info);
void rust__handler__append_create_info(void *ctx, void *packet);
void rust__handler__use_hidden_primary_key(void *ctx);
bool rust__handler__set_ha_share_ref(void *ctx, void *arg, bool *out);
bool rust__handler__cmp_ref(void *ctx, const uint8_t *ref1, const uint8_t *ref2,
                            size_t len, int32_t *out);

// Implemented in the shim (handler_metadata.cc): appends len bytes to the
// MySQL String packet for append_create_info.
void mysql__mysql_string__append(void *packet, const uint8_t *bytes,
                                 size_t len) noexcept;
}

#endif
