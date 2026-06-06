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

#ifndef SHIM_RUST_CALLBACKS_PUSHDOWN_HPP
#define SHIM_RUST_CALLBACKS_PUSHDOWN_HPP

#include <cstdint>

// Condition / index pushdown and pushed-join methods (handler.h #141-#148).
// Item / handlerton / TABLE pointers are round-tripped as opaque `void *` and
// never dereferenced from Rust; the engine returns the input pointer (no
// pushdown), null, or 0 by default. cancel_pushed_idx_cond is a plain void
// notification (the shim resets the base state itself).
extern "C" {
const void *rust__handler__cond_push(void *ctx, const void *cond);
void *rust__handler__idx_cond_push(void *ctx, uint32_t keyno, void *idx_cond);
void rust__handler__cancel_pushed_idx_cond(void *ctx);
const void *rust__handler__hton_supporting_engine_pushdown(void *ctx);
uint32_t rust__handler__number_of_pushed_joins(void *ctx);
const void *rust__handler__member_of_pushed_join(void *ctx);
const void *rust__handler__parent_of_pushed_join(void *ctx);
uint64_t rust__handler__tables_in_pushed_join(void *ctx);
}

#endif
