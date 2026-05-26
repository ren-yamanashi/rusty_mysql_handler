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

#ifndef SHIM_RUST_CALLBACKS_CORE_HPP
#define SHIM_RUST_CALLBACKS_CORE_HPP

#include <cstddef>
#include <cstdint>

// Runtime lifecycle + core handler operations (create / open / close, random
// scan, position, info).
extern "C" {
void rust__plugin_init();
void *rust__create_engine();
void rust__destroy_engine(void *ctx);

int32_t rust__handler__create(void *ctx, const uint8_t *name, size_t name_len);
int32_t rust__handler__open(void *ctx, const uint8_t *name, size_t name_len,
                            int32_t mode);
int32_t rust__handler__close(void *ctx);

int32_t rust__handler__rnd_init(void *ctx, bool scan);
int32_t rust__handler__rnd_end(void *ctx);
int32_t rust__handler__rnd_next(void *ctx, uint8_t *buf, size_t buf_len);
int32_t rust__handler__rnd_pos(void *ctx, uint8_t *buf, size_t buf_len,
                               const uint8_t *pos, size_t pos_len);
void rust__handler__position(void *ctx, const uint8_t *record,
                             size_t record_len);
int32_t rust__handler__rnd_pos_by_record(void *ctx, uint8_t *record,
                                         size_t record_len);
int32_t rust__handler__info(void *ctx, uint32_t flag);
}

#endif
