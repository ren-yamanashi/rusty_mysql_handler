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

#include "rust_callbacks/dd_table.hpp"

#include <algorithm>
#include <cstring>

#include "sql/dd/types/abstract_table.h"
#include "sql/dd/types/column.h"
#include "sql/dd/types/index.h"
#include "sql/dd/types/index_element.h"
#include "sql/dd/types/table.h"

namespace {

const dd::Table *as_table(const void *p) {
  return static_cast<const dd::Table *>(p);
}
const dd::Column *as_column(const void *p) {
  return static_cast<const dd::Column *>(p);
}
const dd::Index *as_index(const void *p) {
  return static_cast<const dd::Index *>(p);
}
const dd::Index_element *as_index_element(const void *p) {
  return static_cast<const dd::Index_element *>(p);
}

// Write up to `cap` bytes of `s` into `buf`; return the full source length so
// the caller can detect truncation. Does not null-terminate.
size_t copy_name(const dd::String_type &s, uint8_t *buf, size_t cap) {
  if (buf && cap > 0) {
    const size_t n = std::min(s.size(), cap);
    std::memcpy(buf, s.data(), n);
  }
  return s.size();
}

}  // namespace

extern "C" {

size_t mysql__DdTable__column_count(const void *table) {
  const dd::Table *t = as_table(table);
  return t ? t->columns().size() : 0;
}

const void *mysql__DdTable__column_at(const void *table, size_t i) {
  const dd::Table *t = as_table(table);
  if (!t || i >= t->columns().size()) return nullptr;
  return static_cast<const void *>(t->columns().at(i));
}

size_t mysql__DdTable__index_count(const void *table) {
  const dd::Table *t = as_table(table);
  return t ? t->indexes().size() : 0;
}

const void *mysql__DdTable__index_at(const void *table, size_t i) {
  const dd::Table *t = as_table(table);
  if (!t || i >= t->indexes().size()) return nullptr;
  return static_cast<const void *>(t->indexes().at(i));
}

size_t mysql__DdColumn__name(const void *column, uint8_t *buf, size_t cap) {
  const dd::Column *c = as_column(column);
  if (!c) return 0;
  return copy_name(c->name(), buf, cap);
}

int32_t mysql__DdColumn__type(const void *column) {
  const dd::Column *c = as_column(column);
  if (!c) return -1;
  return static_cast<int32_t>(c->type());
}

bool mysql__DdColumn__is_nullable(const void *column) {
  const dd::Column *c = as_column(column);
  return c && c->is_nullable();
}

bool mysql__DdColumn__is_unsigned(const void *column) {
  const dd::Column *c = as_column(column);
  return c && c->is_unsigned();
}

uint32_t mysql__DdColumn__char_length(const void *column) {
  const dd::Column *c = as_column(column);
  return c ? static_cast<uint32_t>(c->char_length()) : 0;
}

// Hidden = anything not HT_VISIBLE (SE-hidden, SQL-hidden, USER-hidden).
bool mysql__DdColumn__is_hidden(const void *column) {
  const dd::Column *c = as_column(column);
  return c && c->hidden() != dd::Column::enum_hidden_type::HT_VISIBLE;
}

uint32_t mysql__DdColumn__ordinal_position(const void *column) {
  const dd::Column *c = as_column(column);
  return c ? c->ordinal_position() : 0;
}

size_t mysql__DdIndex__name(const void *index, uint8_t *buf, size_t cap) {
  const dd::Index *i = as_index(index);
  if (!i) return 0;
  return copy_name(i->name(), buf, cap);
}

int32_t mysql__DdIndex__type(const void *index) {
  const dd::Index *i = as_index(index);
  if (!i) return -1;
  return static_cast<int32_t>(i->type());
}

bool mysql__DdIndex__is_hidden(const void *index) {
  const dd::Index *i = as_index(index);
  return i && i->is_hidden();
}

size_t mysql__DdIndex__element_count(const void *index) {
  const dd::Index *i = as_index(index);
  return i ? i->elements().size() : 0;
}

const void *mysql__DdIndex__element_at(const void *index, size_t i) {
  const dd::Index *idx = as_index(index);
  if (!idx || i >= idx->elements().size()) return nullptr;
  return static_cast<const void *>(idx->elements().at(i));
}

uint32_t mysql__DdIndexElement__column_ordinal(const void *elt) {
  const dd::Index_element *e = as_index_element(elt);
  return e ? e->column().ordinal_position() : 0;
}

uint32_t mysql__DdIndexElement__length(const void *elt) {
  const dd::Index_element *e = as_index_element(elt);
  return e ? e->length() : 0;
}

int32_t mysql__DdIndexElement__order(const void *elt) {
  const dd::Index_element *e = as_index_element(elt);
  return e ? static_cast<int32_t>(e->order()) : 0;
}

bool mysql__DdIndexElement__is_hidden(const void *elt) {
  const dd::Index_element *e = as_index_element(elt);
  return e && e->is_hidden();
}

}  // extern "C"
