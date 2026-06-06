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

// SDI handlerton callbacks (handler.h #52-#57). Wired only under
// HtonCapabilities::SDI. dd::Tablespace, dd::Table, sdi_key_t, sdi_vector_t
// cross as opaque pointers; sdi_get round-trips `uint64 *sdi_len` through a
// local u64 for LP64 safety and so the Rust side sees the buffer's current
// capacity in `sdi_capacity` instead of mutating a typed pointer.

#include "binding.hpp"
#include "rust_callbacks.hpp"
#include "sql/handler.h"

namespace {
bool rusty_hton_sdi_create(dd::Tablespace *tablespace) {
  return rust__hton__sdi_create(static_cast<const void *>(tablespace));
}

bool rusty_hton_sdi_drop(dd::Tablespace *tablespace) {
  return rust__hton__sdi_drop(static_cast<const void *>(tablespace));
}

bool rusty_hton_sdi_get_keys(const dd::Tablespace &tablespace,
                             sdi_vector_t &vector) {
  return rust__hton__sdi_get_keys(static_cast<const void *>(&tablespace),
                                  static_cast<const void *>(&vector));
}

bool rusty_hton_sdi_get(const dd::Tablespace &tablespace,
                        const sdi_key_t *sdi_key, void *sdi, uint64 *sdi_len) {
  uint64_t cap = sdi_len ? static_cast<uint64_t>(*sdi_len) : 0;
  uint64_t local = cap;
  bool err = rust__hton__sdi_get(
      static_cast<const void *>(&tablespace),
      static_cast<const void *>(sdi_key),
      static_cast<uint8_t *>(sdi), cap, &local);
  if (sdi_len) {
    *sdi_len = static_cast<uint64>(local);
  }
  return err;
}

bool rusty_hton_sdi_set(handlerton *, const dd::Tablespace &tablespace,
                        const dd::Table *table, const sdi_key_t *sdi_key,
                        const void *sdi, uint64 sdi_len) {
  return rust__hton__sdi_set(
      static_cast<const void *>(&tablespace),
      static_cast<const void *>(table),
      static_cast<const void *>(sdi_key),
      static_cast<const uint8_t *>(sdi),
      static_cast<uint64_t>(sdi_len));
}

bool rusty_hton_sdi_delete(const dd::Tablespace &tablespace,
                           const dd::Table *table, const sdi_key_t *sdi_key) {
  return rust__hton__sdi_delete(static_cast<const void *>(&tablespace),
                                static_cast<const void *>(table),
                                static_cast<const void *>(sdi_key));
}
}  // namespace

void rusty_hton_wire_sdi(handlerton *hton) {
  hton->sdi_create = rusty_hton_sdi_create;
  hton->sdi_drop = rusty_hton_sdi_drop;
  hton->sdi_get_keys = rusty_hton_sdi_get_keys;
  hton->sdi_get = rusty_hton_sdi_get;
  hton->sdi_set = rusty_hton_sdi_set;
  hton->sdi_delete = rusty_hton_sdi_delete;
}
