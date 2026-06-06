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

#ifndef SHIM_RUST_CALLBACKS_LOCKING_HPP
#define SHIM_RUST_CALLBACKS_LOCKING_HPP

#include <cstdint>

// Locking methods (handler.h #103-#109). The handler base default of each is
// trivial, so these delegate straight to the engine (no base fallback). THD
// crosses as an opaque `const void *`. store_lock passes the raw
// `enum thr_lock_type` as an int32_t and expects the engine's chosen type back.
extern "C" {
int rust__handler__external_lock(void *ctx, const void *thd, int32_t lock_type);
uint32_t rust__handler__lock_count(void *ctx);
int32_t rust__handler__store_lock(void *ctx, int32_t requested);
void rust__handler__unlock_row(void *ctx);
int rust__handler__start_stmt(void *ctx, const void *thd, int32_t lock_type);
bool rust__handler__was_semi_consistent_read(void *ctx);
void rust__handler__try_semi_consistent_read(void *ctx, bool enable);
}

#endif
