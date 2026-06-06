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

// Condition / index pushdown and pushed-join overrides (handler.h #141-#148)

#include "binding.hpp"
#include "rust_callbacks.hpp"

// Each override delegates to the engine, falling back to the public handler::
// base when no engine is attached. Item / handlerton / TABLE are round-tripped
// as opaque pointers.

const Item *RustHandlerBase::cond_push(const Item *cond) {
  if (rust_ctx_)
    return static_cast<const Item *>(
        rust__handler__cond_push(rust_ctx_, static_cast<const void *>(cond)));
  return handler::cond_push(cond);
}

Item *RustHandlerBase::idx_cond_push(uint keyno, Item *idx_cond) {
  if (rust_ctx_)
    return static_cast<Item *>(rust__handler__idx_cond_push(
        rust_ctx_, keyno, static_cast<void *>(idx_cond)));
  return handler::idx_cond_push(keyno, idx_cond);
}

void RustHandlerBase::cancel_pushed_idx_cond() {
  // handler::cancel_pushed_idx_cond resets pushed_idx_cond / keyno /
  // in_range_check_pushed_down, so run it first, then notify the engine.
  handler::cancel_pushed_idx_cond();
  if (rust_ctx_) rust__handler__cancel_pushed_idx_cond(rust_ctx_);
}

const handlerton *RustHandlerBase::hton_supporting_engine_pushdown() {
  if (rust_ctx_)
    return static_cast<const handlerton *>(
        rust__handler__hton_supporting_engine_pushdown(rust_ctx_));
  return handler::hton_supporting_engine_pushdown();
}

uint RustHandlerBase::number_of_pushed_joins() const {
  if (rust_ctx_) return rust__handler__number_of_pushed_joins(rust_ctx_);
  return handler::number_of_pushed_joins();
}

const TABLE *RustHandlerBase::member_of_pushed_join() const {
  if (rust_ctx_)
    return static_cast<const TABLE *>(
        rust__handler__member_of_pushed_join(rust_ctx_));
  return handler::member_of_pushed_join();
}

const TABLE *RustHandlerBase::parent_of_pushed_join() const {
  if (rust_ctx_)
    return static_cast<const TABLE *>(
        rust__handler__parent_of_pushed_join(rust_ctx_));
  return handler::parent_of_pushed_join();
}

table_map RustHandlerBase::tables_in_pushed_join() const {
  if (rust_ctx_) return rust__handler__tables_in_pushed_join(rust_ctx_);
  return handler::tables_in_pushed_join();
}
