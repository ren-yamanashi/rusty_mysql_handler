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

#ifndef SHIM_RUST_CALLBACKS_COST_HPP
#define SHIM_RUST_CALLBACKS_COST_HPP

#include <cstdint>

// Cost-estimation overrides (handler.h #91-#98). Each returns true when the
// engine overrides and false to fall back to the handler base. The double
// methods write one out-pointer; the Cost_estimate methods decompose into
// io/cpu/import/mem out-pointers the shim reassembles via add_io/add_cpu/etc.
extern "C" {
bool rust__handler__scan_time(void *ctx, double *out);
bool rust__handler__read_time(void *ctx, uint32_t index, uint32_t ranges,
                              uint64_t rows, double *out);
bool rust__handler__index_only_read_time(void *ctx, uint32_t keynr,
                                         double records, double *out);
bool rust__handler__table_scan_cost(void *ctx, double *io, double *cpu,
                                    double *import_cost, double *mem);
bool rust__handler__index_scan_cost(void *ctx, uint32_t index, double ranges,
                                    double rows, double *io, double *cpu,
                                    double *import_cost, double *mem);
bool rust__handler__read_cost(void *ctx, uint32_t index, double ranges,
                              double rows, double *io, double *cpu,
                              double *import_cost, double *mem);
bool rust__handler__page_read_cost(void *ctx, uint32_t index, double reads,
                                   double *out);
bool rust__handler__worst_seek_times(void *ctx, double reads, double *out);
}

#endif
