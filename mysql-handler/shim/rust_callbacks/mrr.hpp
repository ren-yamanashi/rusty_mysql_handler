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

#ifndef SHIM_RUST_CALLBACKS_MRR_HPP
#define SHIM_RUST_CALLBACKS_MRR_HPP

#include <cstddef>
#include <cstdint>

// Multi-range read (handler.h #64-#67). The base disk-sweep MRR impl drives
// read_range_first/next, so each callback returns true only when the engine
// handles the call (result written through the out-pointer) and false to fall
// back to handler::multi_range_read_*. RANGE_SEQ_IF / Cost_estimate /
// HANDLER_BUFFER cross as opaque pointers the engine cannot yet drive;
// seq_init_param and range_info are round-tripped without dereference.
extern "C" {
bool rust__handler__multi_range_read_info_const(
    void *ctx, uint32_t keyno, const void *seq, void *seq_init_param,
    uint32_t n_ranges, const void *cost, uint64_t *out_rows);
bool rust__handler__multi_range_read_info(void *ctx, uint32_t keyno,
                                          uint32_t n_ranges, uint32_t keys,
                                          const void *cost, uint64_t *out_rows);
bool rust__handler__multi_range_read_init(void *ctx, const void *seq,
                                          void *seq_init_param,
                                          uint32_t n_ranges, uint32_t mode,
                                          const void *buf, int32_t *out_result);
bool rust__handler__multi_range_read_next(void *ctx, uint8_t *buf,
                                          size_t buf_len, void **range_info,
                                          int32_t *out_result);
}

#endif
