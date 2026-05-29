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

// Locking overrides (handler.h #104-#109)

#include "binding.hpp"
#include "rust_callbacks.hpp"

// The handler base default of each locking method is trivial, so these delegate
// straight to the engine; the Rust trait defaults reproduce the base behaviour.

int RustHandlerBase::external_lock(THD *thd, int lock_type) {
  // A transactional engine must register in the current transaction here, or
  // MySQL never calls its commit / rollback. Register on lock acquisition
  // (not on F_UNLCK release).
  if (lock_type != F_UNLCK && rust__hton__is_transactional()) {
    rusty_hton_register_txn(thd, ht);
  }
  return rust__handler__external_lock(rust_ctx_, static_cast<const void *>(thd),
                                      lock_type);
}

uint RustHandlerBase::lock_count() const {
  return rust__handler__lock_count(rust_ctx_);
}

void RustHandlerBase::unlock_row() { rust__handler__unlock_row(rust_ctx_); }

int RustHandlerBase::start_stmt(THD *thd, thr_lock_type lock_type) {
  // Statements under LOCK TABLES skip external_lock, so register here too.
  if (rust__hton__is_transactional()) {
    rusty_hton_register_txn(thd, ht);
  }
  return rust__handler__start_stmt(rust_ctx_, static_cast<const void *>(thd),
                                   static_cast<int32_t>(lock_type));
}

bool RustHandlerBase::was_semi_consistent_read() {
  return rust__handler__was_semi_consistent_read(rust_ctx_);
}

void RustHandlerBase::try_semi_consistent_read(bool yes) {
  rust__handler__try_semi_consistent_read(rust_ctx_, yes);
}
