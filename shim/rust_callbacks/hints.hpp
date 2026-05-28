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

#ifndef SHIM_RUST_CALLBACKS_HINTS_HPP
#define SHIM_RUST_CALLBACKS_HINTS_HPP

#include <cstdint>

// Hint and extra methods (handler.h #119-#123). The handler base default of
// each is trivial, so the shim delegates straight to the engine when an engine
// is attached. `ha_extra_function` crosses as its raw `int` value.
extern "C" {
int rust__handler__extra(void *ctx, int operation);
int rust__handler__extra_opt(void *ctx, int operation, uint64_t cache_size);
int rust__handler__reset(void *ctx);
void rust__handler__column_bitmaps_signal(void *ctx);
void rust__handler__init_table_handle_for_handler(void *ctx);
}

#endif
