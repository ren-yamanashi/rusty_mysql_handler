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

#ifndef SHIM_RUST_CALLBACKS_HTON_CORE_HPP
#define SHIM_RUST_CALLBACKS_HTON_CORE_HPP

#include <cstdint>

// Engine-level handlerton accessors queried by rusty_init_func to populate the
// handlerton struct from the registered Rust Handlerton singleton. Returns the
// zero-config default when no handlerton is registered.
extern "C" {
uint32_t rust__hton__flags();
// Whether a Rust Handlerton is registered; gates wiring of the always-on hooks.
bool rust__hton__is_registered();
// Whether the handlerton declares TRANSACTIONS; gates commit/rollback/prepare
// wiring and transaction registration in external_lock.
bool rust__hton__is_transactional();
// Whether the handlerton declares XA; gates the XA recovery callbacks.
bool rust__hton__is_xa();
// Whether the handlerton declares SAVEPOINTS; gates the savepoint callbacks.
bool rust__hton__is_savepoints();
// Bytes of per-savepoint scratch the engine needs (used for both the
// handlerton field and as the sv buffer length, since MySQL repurposes the
// field to an offset after init).
uint32_t rust__hton__savepoint_offset();
// Whether the handlerton declares PARTITIONING; gates the partition_flags
// accessor on the handlerton (a non-NULL pointer there is what tells MySQL the
// engine implements handler::get_partition_handler).
bool rust__hton__is_partitioning();
// Whether the handlerton declares TABLESPACES; gates the tablespace callbacks.
bool rust__hton__is_tablespaces();
// Whether the handlerton declares DICT_BACKEND; gates the dict_* callbacks.
bool rust__hton__is_dict_backend();
// Whether the handlerton declares SDI; gates the sdi_* callbacks.
bool rust__hton__is_sdi();
// Whether the handlerton declares ENGINE_LOG; gates the lock/unlock/collect
// engine-log callbacks consumed by performance_schema.log_status.
bool rust__hton__is_engine_log();
}

#endif
