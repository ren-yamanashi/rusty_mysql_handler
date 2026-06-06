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

// Cost-estimation overrides (handler.h #91-#98)

#include "binding.hpp"
#include "rust_callbacks.hpp"

// Each override lets the engine supply a cost estimate, falling back to the
// handler base when it declines. The Cost_estimate-returning methods reassemble
// the engine's io/cpu/import/mem components via add_io/add_cpu/add_import/add_mem.

static Cost_estimate make_cost(double io, double cpu, double import, double mem) {
  Cost_estimate cost;
  cost.add_io(io);
  cost.add_cpu(cpu);
  cost.add_import(import);
  cost.add_mem(mem);
  return cost;
}

double RustHandlerBase::scan_time() {
  if (rust_ctx_) {
    double v = 0;
    if (rust__handler__scan_time(rust_ctx_, &v)) return v;
  }
  return handler::scan_time();
}

double RustHandlerBase::read_time(uint index, uint ranges, ha_rows rows) {
  if (rust_ctx_) {
    double v = 0;
    if (rust__handler__read_time(rust_ctx_, index, ranges, rows, &v)) return v;
  }
  return handler::read_time(index, ranges, rows);
}

double RustHandlerBase::index_only_read_time(uint keynr, double records) {
  if (rust_ctx_) {
    double v = 0;
    if (rust__handler__index_only_read_time(rust_ctx_, keynr, records, &v))
      return v;
  }
  return handler::index_only_read_time(keynr, records);
}

Cost_estimate RustHandlerBase::table_scan_cost() {
  if (rust_ctx_) {
    double io = 0, cpu = 0, import = 0, mem = 0;
    if (rust__handler__table_scan_cost(rust_ctx_, &io, &cpu, &import, &mem))
      return make_cost(io, cpu, import, mem);
  }
  return handler::table_scan_cost();
}

Cost_estimate RustHandlerBase::index_scan_cost(uint index, double ranges,
                                               double rows) {
  if (rust_ctx_) {
    double io = 0, cpu = 0, import = 0, mem = 0;
    if (rust__handler__index_scan_cost(rust_ctx_, index, ranges, rows, &io,
                                       &cpu, &import, &mem))
      return make_cost(io, cpu, import, mem);
  }
  return handler::index_scan_cost(index, ranges, rows);
}

Cost_estimate RustHandlerBase::read_cost(uint index, double ranges,
                                         double rows) {
  if (rust_ctx_) {
    double io = 0, cpu = 0, import = 0, mem = 0;
    if (rust__handler__read_cost(rust_ctx_, index, ranges, rows, &io, &cpu,
                                 &import, &mem))
      return make_cost(io, cpu, import, mem);
  }
  return handler::read_cost(index, ranges, rows);
}

double RustHandlerBase::page_read_cost(uint index, double reads) {
  if (rust_ctx_) {
    double v = 0;
    if (rust__handler__page_read_cost(rust_ctx_, index, reads, &v)) return v;
  }
  return handler::page_read_cost(index, reads);
}

double RustHandlerBase::worst_seek_times(double reads) {
  if (rust_ctx_) {
    double v = 0;
    if (rust__handler__worst_seek_times(rust_ctx_, reads, &v)) return v;
  }
  return handler::worst_seek_times(reads);
}
