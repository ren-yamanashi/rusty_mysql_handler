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

// Sampling overrides (handler.h #57-#59)

#include "binding.hpp"
#include "my_dbug.h"
#include "rust_callbacks.hpp"
#include "sql/table.h"

int RustHandlerBase::sample_init(void *&scan_ctx, double sampling_percentage,
                                 int sampling_seed,
                                 enum_sampling_method sampling_method,
                                 const bool tablesample) {
  DBUG_TRACE;
  if (!rust_ctx_) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__sample_init(rust_ctx_, &scan_ctx, sampling_percentage,
                                    sampling_seed,
                                    static_cast<int32_t>(sampling_method),
                                    tablesample);
}

int RustHandlerBase::sample_next(void *scan_ctx, uchar *buf) {
  DBUG_TRACE;
  if (!rust_ctx_) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__sample_next(rust_ctx_, scan_ctx, buf,
                                    table->s->rec_buff_length);
}

int RustHandlerBase::sample_end(void *scan_ctx) {
  DBUG_TRACE;
  if (!rust_ctx_) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__sample_end(rust_ctx_, scan_ctx);
}
