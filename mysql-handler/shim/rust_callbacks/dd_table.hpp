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

#ifndef SHIM_RUST_CALLBACKS_DD_TABLE_HPP
#define SHIM_RUST_CALLBACKS_DD_TABLE_HPP

#include <cstddef>
#include <cstdint>

// Rust-callable accessors over `dd::Table`, `dd::Column`, `dd::Index`,
// `dd::Index_element`. Each opaque pointer is `const void *` so the shim's
// public header does not pull in server-internal dd headers.
//
// Returned pointers refer to objects owned by the dd::Table tree the caller
// already holds. They are valid only for the duration of the originating
// handler callback; do not retain them across boundaries.
//
// String accessors write up to `cap` bytes into `buf` and return the full
// length (so the caller can detect truncation). They do not null-terminate.
extern "C" {
size_t mysql__DdTable__column_count(const void *table);
const void *mysql__DdTable__column_at(const void *table, size_t i);
size_t mysql__DdTable__index_count(const void *table);
const void *mysql__DdTable__index_at(const void *table, size_t i);

size_t mysql__DdColumn__name(const void *column, uint8_t *buf, size_t cap);
int32_t mysql__DdColumn__type(const void *column);
bool mysql__DdColumn__is_nullable(const void *column);
bool mysql__DdColumn__is_unsigned(const void *column);
uint32_t mysql__DdColumn__char_length(const void *column);
bool mysql__DdColumn__is_hidden(const void *column);
uint32_t mysql__DdColumn__ordinal_position(const void *column);

size_t mysql__DdIndex__name(const void *index, uint8_t *buf, size_t cap);
int32_t mysql__DdIndex__type(const void *index);
bool mysql__DdIndex__is_hidden(const void *index);
size_t mysql__DdIndex__element_count(const void *index);
const void *mysql__DdIndex__element_at(const void *index, size_t i);

uint32_t mysql__DdIndexElement__column_ordinal(const void *elt);
uint32_t mysql__DdIndexElement__length(const void *elt);
int32_t mysql__DdIndexElement__order(const void *elt);
bool mysql__DdIndexElement__is_hidden(const void *elt);
}

#endif
