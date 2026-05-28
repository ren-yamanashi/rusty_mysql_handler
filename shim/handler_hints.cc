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

// Hint and extra overrides (handler.h #119-#123)

#include "binding.hpp"
#include "rust_callbacks.hpp"

// These bind hint/reset/HANDLER-setup methods that can be invoked outside the
// strict open window, so they guard on rust_ctx_ and fall back to the handler
// base (all trivial) when no engine is attached.

int RustHandlerBase::extra(enum ha_extra_function operation) {
  if (rust_ctx_)
    return rust__handler__extra(rust_ctx_, static_cast<int>(operation));
  // handler::extra is private (NVI); reproduce its trivial base (success).
  return 0;
}

int RustHandlerBase::extra_opt(enum ha_extra_function operation,
                               ulong cache_size) {
  if (rust_ctx_)
    return rust__handler__extra_opt(rust_ctx_, static_cast<int>(operation),
                                    static_cast<uint64_t>(cache_size));
  return handler::extra_opt(operation, cache_size);
}

int RustHandlerBase::reset() {
  if (rust_ctx_) return rust__handler__reset(rust_ctx_);
  // handler::reset is private (NVI); reproduce its trivial base (success).
  return 0;
}

void RustHandlerBase::column_bitmaps_signal() {
  if (rust_ctx_) {
    rust__handler__column_bitmaps_signal(rust_ctx_);
    return;
  }
  handler::column_bitmaps_signal();
}

void RustHandlerBase::init_table_handle_for_HANDLER() {
  if (rust_ctx_) {
    rust__handler__init_table_handle_for_handler(rust_ctx_);
    return;
  }
  handler::init_table_handle_for_HANDLER();
}
