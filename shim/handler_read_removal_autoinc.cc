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

// Read-before-write removal and auto-increment overrides (handler.h #110-#113)

#include "binding.hpp"
#include "rust_callbacks.hpp"

// start_read_removal / end_read_removal / get_auto_increment fall back to the
// handler base when the engine declines, so they guard on rust_ctx_.
// release_auto_increment has a trivial base, so it delegates straight through.

bool RustHandlerBase::start_read_removal() {
  if (rust_ctx_) {
    bool v = false;
    if (rust__handler__start_read_removal(rust_ctx_, &v)) return v;
  }
  return handler::start_read_removal();
}

ha_rows RustHandlerBase::end_read_removal() {
  if (rust_ctx_) {
    uint64_t n = 0;
    if (rust__handler__end_read_removal(rust_ctx_, &n)) return n;
  }
  return handler::end_read_removal();
}

void RustHandlerBase::get_auto_increment(ulonglong offset, ulonglong increment,
                                         ulonglong nb_desired_values,
                                         ulonglong *first_value,
                                         ulonglong *nb_reserved_values) {
  if (rust_ctx_) {
    // ulonglong and uint64_t differ on LP64 (unsigned long long vs unsigned
    // long), so round-trip through uint64_t locals rather than aliasing the
    // ulonglong out-pointers, which would be a pointer-type mismatch.
    uint64_t first = 0;
    uint64_t reserved = 0;
    if (rust__handler__get_auto_increment(rust_ctx_, offset, increment,
                                          nb_desired_values, &first,
                                          &reserved)) {
      *first_value = first;
      *nb_reserved_values = reserved;
      return;
    }
  }
  handler::get_auto_increment(offset, increment, nb_desired_values, first_value,
                              nb_reserved_values);
}

void RustHandlerBase::release_auto_increment() {
  rust__handler__release_auto_increment(rust_ctx_);
}
