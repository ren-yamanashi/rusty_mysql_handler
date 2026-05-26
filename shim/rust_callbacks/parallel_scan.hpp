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

#ifndef SHIM_RUST_CALLBACKS_PARALLEL_SCAN_HPP
#define SHIM_RUST_CALLBACKS_PARALLEL_SCAN_HPP

#include <cstddef>
#include <cstdint>

// Parallel scan (handler.h #54-#56). Scan contexts are engine-owned pointers
// round-tripped verbatim; parallel_scan_init writes the context and thread
// count through out-pointers. parallel_scan's load callbacks are MySQL
// std::function objects passed as opaque pointers the engine cannot yet invoke.
extern "C" {
int32_t rust__handler__parallel_scan_init(void *ctx, void **scan_ctx,
                                          size_t *num_threads,
                                          bool use_reserved_threads,
                                          size_t max_desired_threads);
int32_t rust__handler__parallel_scan(void *ctx, void *scan_ctx,
                                     void **thread_ctxs, const void *init_fn,
                                     const void *load_fn, const void *end_fn);
void rust__handler__parallel_scan_end(void *ctx, void *scan_ctx);
}

#endif
