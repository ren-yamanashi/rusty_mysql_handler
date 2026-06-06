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

#ifndef SHIM_RUST_CALLBACKS_HTON_SDI_HPP
#define SHIM_RUST_CALLBACKS_HTON_SDI_HPP

#include <cstddef>
#include <cstdint>

// Engine-level SDI callbacks delegating to the registered Rust handlerton
// singleton. Wired only under HtonCapabilities::SDI. dd::Tablespace,
// dd::Table, sdi_key_t, and sdi_vector_t all cross as opaque `const void *`;
// SDI payloads come with explicit lengths so Rust never strlen-scans.
extern "C" {
bool rust__hton__sdi_create(const void *tablespace);
bool rust__hton__sdi_drop(const void *tablespace);
bool rust__hton__sdi_get_keys(const void *tablespace, const void *vector);
bool rust__hton__sdi_get(const void *tablespace, const void *key, uint8_t *sdi,
                         uint64_t sdi_capacity, uint64_t *len_out);
bool rust__hton__sdi_set(const void *tablespace, const void *table,
                         const void *key, const uint8_t *payload,
                         uint64_t payload_len);
bool rust__hton__sdi_delete(const void *tablespace, const void *table,
                            const void *key);
}

#endif
