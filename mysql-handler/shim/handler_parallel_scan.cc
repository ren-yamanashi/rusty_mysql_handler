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

// Parallel-scan overrides (handler.h #54-#56)

#include "binding.hpp"
#include "my_dbug.h"
#include "rust_callbacks.hpp"

int RustHandlerBase::parallel_scan_init(void *&scan_ctx, size_t *num_threads,
                                        bool use_reserved_threads,
                                        size_t max_desired_threads) {
  DBUG_TRACE;
  if (!rust_ctx_) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__parallel_scan_init(rust_ctx_, &scan_ctx, num_threads,
                                           use_reserved_threads,
                                           max_desired_threads);
}

int RustHandlerBase::parallel_scan(void *scan_ctx, void **thread_ctxs,
                                   Load_init_cbk init_fn, Load_cbk load_fn,
                                   Load_end_cbk end_fn) {
  DBUG_TRACE;
  if (!rust_ctx_) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__parallel_scan(
      rust_ctx_, scan_ctx, thread_ctxs, static_cast<const void *>(&init_fn),
      static_cast<const void *>(&load_fn), static_cast<const void *>(&end_fn));
}

void RustHandlerBase::parallel_scan_end(void *scan_ctx) {
  DBUG_TRACE;
  if (!rust_ctx_) return;
  rust__handler__parallel_scan_end(rust_ctx_, scan_ctx);
}
