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

#ifndef SHIM_RUST_CALLBACKS_MAINTENANCE_HPP
#define SHIM_RUST_CALLBACKS_MAINTENANCE_HPP

#include <cstdint>

// CHECK / REPAIR / OPTIMIZE / ANALYZE admin commands (handler.h #130-#135).
// Each returns true when the engine overrides (HA_ADMIN_* code / flag written
// through the out-pointer) and false to fall back to the handler base. THD and
// HA_CHECK_OPT cross as opaque `const void *`.
extern "C" {
bool rust__handler__check(void *ctx, const void *thd, const void *check_opt,
                          int32_t *out);
bool rust__handler__repair(void *ctx, const void *thd, const void *check_opt,
                           int32_t *out);
bool rust__handler__optimize(void *ctx, const void *thd, const void *check_opt,
                             int32_t *out);
bool rust__handler__analyze(void *ctx, const void *thd, const void *check_opt,
                            int32_t *out);
bool rust__handler__check_and_repair(void *ctx, const void *thd, bool *out);
bool rust__handler__check_for_upgrade(void *ctx, const void *check_opt,
                                      int32_t *out);
}

#endif
