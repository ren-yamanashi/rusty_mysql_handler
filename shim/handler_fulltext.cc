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

// Full-text search overrides (handler.h #60-#63)

#include "binding.hpp"
#include "my_dbug.h"
#include "rust_callbacks.hpp"
#include "sql/table.h"

int RustHandlerBase::ft_init() {
  DBUG_TRACE;
  if (!rust_ctx_) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__ft_init(rust_ctx_);
}

FT_INFO *RustHandlerBase::ft_init_ext(uint flags, uint inx, String *key) {
  DBUG_TRACE;
  if (!rust_ctx_) return nullptr;
  return static_cast<FT_INFO *>(rust__handler__ft_init_ext(
      rust_ctx_, flags, inx, static_cast<const void *>(key)));
}

FT_INFO *RustHandlerBase::ft_init_ext_with_hints(uint inx, String *key,
                                                 Ft_hints *hints) {
  DBUG_TRACE;
  if (!rust_ctx_) return nullptr;
  const uint flags = hints ? hints->get_flags() : 0;
  return static_cast<FT_INFO *>(rust__handler__ft_init_ext_with_hints(
      rust_ctx_, flags, inx, static_cast<const void *>(key),
      static_cast<const void *>(hints)));
}

int RustHandlerBase::ft_read(uchar *buf) {
  DBUG_TRACE;
  if (!rust_ctx_) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__ft_read(rust_ctx_, buf, table->s->rec_buff_length);
}
