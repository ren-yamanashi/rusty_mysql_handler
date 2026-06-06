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

#ifndef SHIM_RUST_CALLBACKS_RECORDS_HPP
#define SHIM_RUST_CALLBACKS_RECORDS_HPP

#include <cstdint>

// Row-count overrides (handler.h #99-#102). records / records_from_index
// return the HA error code and set *handled when the engine supplied a count
// (written through num_rows); estimate_rows_upper_bound and
// calculate_key_hash_value return true when the engine overrode the value.
// The Field** array crosses as an opaque `const void *`.
extern "C" {
int rust__handler__records(void *ctx, uint64_t *num_rows, bool *handled);
int rust__handler__records_from_index(void *ctx, uint32_t index,
                                      uint64_t *num_rows, bool *handled);
bool rust__handler__estimate_rows_upper_bound(void *ctx, uint64_t *out);
bool rust__handler__calculate_key_hash_value(void *ctx, const void *field_array,
                                             uint32_t *out);
}

#endif
