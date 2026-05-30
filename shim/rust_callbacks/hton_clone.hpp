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

#ifndef SHIM_RUST_CALLBACKS_HTON_CLONE_HPP
#define SHIM_RUST_CALLBACKS_HTON_CLONE_HPP

#include <cstddef>
#include <cstdint>

// Clone-interface sub-callbacks. Wired only under HtonCapabilities::CLONE.
// THD / Ha_clone_cbk cross as opaque `const void *`; the engine-owned
// locator is not surfaced through these FFI symbols today (the shim writes
// a NULL locator back to MySQL).
extern "C" {
void rust__hton__clone_capability(uint64_t *out_flags);
int32_t rust__hton__clone_begin(const void *thd, size_t clone_type,
                                uint32_t mode);
int32_t rust__hton__clone_copy(const void *thd, uint32_t task_id,
                               const void *cbk);
int32_t rust__hton__clone_ack(const void *thd, uint32_t task_id,
                              int32_t in_err, const void *cbk);
int32_t rust__hton__clone_end(const void *thd, uint32_t task_id,
                              int32_t in_err);
int32_t rust__hton__clone_apply_begin(const void *thd, uint32_t mode,
                                      const uint8_t *data_dir,
                                      size_t data_dir_len);
int32_t rust__hton__clone_apply(const void *thd, uint32_t task_id,
                                int32_t in_err, const void *cbk);
int32_t rust__hton__clone_apply_end(const void *thd, uint32_t task_id,
                                    int32_t in_err);
}

#endif
