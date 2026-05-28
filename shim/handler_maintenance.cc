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

// CHECK / REPAIR / OPTIMIZE / ANALYZE admin overrides (handler.h #130-#135)

#include "binding.hpp"
#include "rust_callbacks.hpp"

// Each override lets the engine handle the admin command, falling back to the
// handler base when it declines. check / repair / check_for_upgrade are private
// NVI virtuals, so their trivial base result is reproduced inline; the rest call
// the public handler:: base.

int RustHandlerBase::check(THD *thd, HA_CHECK_OPT *check_opt) {
  if (rust_ctx_) {
    int32_t v = 0;
    if (rust__handler__check(rust_ctx_, static_cast<const void *>(thd),
                             static_cast<const void *>(check_opt), &v))
      return v;
  }
  return HA_ADMIN_NOT_IMPLEMENTED;
}

int RustHandlerBase::repair(THD *thd, HA_CHECK_OPT *check_opt) {
  if (rust_ctx_) {
    int32_t v = 0;
    if (rust__handler__repair(rust_ctx_, static_cast<const void *>(thd),
                              static_cast<const void *>(check_opt), &v))
      return v;
  }
  return HA_ADMIN_NOT_IMPLEMENTED;
}

int RustHandlerBase::optimize(THD *thd, HA_CHECK_OPT *check_opt) {
  if (rust_ctx_) {
    int32_t v = 0;
    if (rust__handler__optimize(rust_ctx_, static_cast<const void *>(thd),
                                static_cast<const void *>(check_opt), &v))
      return v;
  }
  return handler::optimize(thd, check_opt);
}

int RustHandlerBase::analyze(THD *thd, HA_CHECK_OPT *check_opt) {
  if (rust_ctx_) {
    int32_t v = 0;
    if (rust__handler__analyze(rust_ctx_, static_cast<const void *>(thd),
                               static_cast<const void *>(check_opt), &v))
      return v;
  }
  return handler::analyze(thd, check_opt);
}

bool RustHandlerBase::check_and_repair(THD *thd) {
  if (rust_ctx_) {
    bool v = false;
    if (rust__handler__check_and_repair(rust_ctx_,
                                        static_cast<const void *>(thd), &v))
      return v;
  }
  return handler::check_and_repair(thd);
}

int RustHandlerBase::check_for_upgrade(HA_CHECK_OPT *check_opt) {
  if (rust_ctx_) {
    int32_t v = 0;
    if (rust__handler__check_for_upgrade(
            rust_ctx_, static_cast<const void *>(check_opt), &v))
      return v;
  }
  return 0;
}
