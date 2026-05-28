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

#ifndef SHIM_RUST_CALLBACKS_MISC_HPP
#define SHIM_RUST_CALLBACKS_MISC_HPP

#include <cstddef>
#include <cstdint>

// Secondary-engine offload, clone, multi-valued-index and partitioning methods
// (handler.h #154-#158). handler / MEM_ROOT / Partition_handler cross as opaque
// pointers; clone / get_partition_handler return null to fall back to the
// handler base, mv_key_capacity returns true when the engine overrode it.
extern "C" {
void rust__handler__set_external_table_offload_error(void *ctx,
                                                     const uint8_t *reason,
                                                     size_t len);
void rust__handler__external_table_offload_error(void *ctx);
void *rust__handler__clone(void *ctx, const uint8_t *name, size_t len,
                           void *mem_root);
bool rust__handler__mv_key_capacity(void *ctx, uint32_t *num_keys,
                                    uint64_t *keys_length);
void *rust__handler__get_partition_handler(void *ctx);
}

#endif
