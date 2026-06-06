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

#ifndef SHIM_RUST_CALLBACKS_READ_REMOVAL_AUTOINC_HPP
#define SHIM_RUST_CALLBACKS_READ_REMOVAL_AUTOINC_HPP

#include <cstdint>

// Read-before-write removal and auto-increment methods (handler.h #110-#113).
// The bool-returning callbacks report true when the engine overrides (values
// written through the out-pointers) and false to fall back to the handler base;
// release_auto_increment is a plain void delegation.
extern "C" {
bool rust__handler__start_read_removal(void *ctx, bool *out);
bool rust__handler__end_read_removal(void *ctx, uint64_t *out);
bool rust__handler__get_auto_increment(void *ctx, uint64_t offset,
                                       uint64_t increment, uint64_t nb_desired,
                                       uint64_t *first_value,
                                       uint64_t *nb_reserved);
void rust__handler__release_auto_increment(void *ctx);
}

#endif
