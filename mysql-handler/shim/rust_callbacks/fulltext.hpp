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

#ifndef SHIM_RUST_CALLBACKS_FULLTEXT_HPP
#define SHIM_RUST_CALLBACKS_FULLTEXT_HPP

#include <cstddef>
#include <cstdint>

// Full-text search (handler.h #60-#63). ft_init_ext / _with_hints return an
// engine-owned FT_INFO pointer round-tripped verbatim; the String query and
// Ft_hints cross as opaque pointers. _with_hints' flags are pre-extracted from
// hints by the shim so the engine never reaches into the opaque hints object.
extern "C" {
int32_t rust__handler__ft_init(void *ctx);
void *rust__handler__ft_init_ext(void *ctx, uint32_t flags, uint32_t inx,
                                 const void *key);
void *rust__handler__ft_init_ext_with_hints(void *ctx, uint32_t flags,
                                            uint32_t inx, const void *key,
                                            const void *hints);
int32_t rust__handler__ft_read(void *ctx, uint8_t *buf, size_t buf_len);
}

#endif
