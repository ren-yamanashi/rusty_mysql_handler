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

// Secondary-engine offload, clone, mv-key and partitioning overrides
// (handler.h #154-#158)

#include "binding.hpp"
#include "rust_callbacks.hpp"
#include "safe_name.hpp"

// A null reason / name falls through to the base: FfiPtr::bytes_to_str requires
// a non-null pointer even at len 0, so null must never reach the Rust side.
// shim::safe_name_len bounds the scan if the null-termination contract breaks.

void RustHandlerBase::set_external_table_offload_error(const char *reason) {
  if (rust_ctx_ && reason) {
    rust__handler__set_external_table_offload_error(
        rust_ctx_, reinterpret_cast<const uint8_t *>(reason),
        shim::safe_name_len(reason));
    return;
  }
  handler::set_external_table_offload_error(reason);
}

void RustHandlerBase::external_table_offload_error() const {
  if (rust_ctx_) {
    rust__handler__external_table_offload_error(rust_ctx_);
    return;
  }
  handler::external_table_offload_error();
}

handler *RustHandlerBase::clone(const char *name, MEM_ROOT *mem_root) {
  if (rust_ctx_ && name) {
    handler *cloned = static_cast<handler *>(rust__handler__clone(
        rust_ctx_, reinterpret_cast<const uint8_t *>(name),
        shim::safe_name_len(name), static_cast<void *>(mem_root)));
    if (cloned) return cloned;
  }
  return handler::clone(name, mem_root);
}

void RustHandlerBase::mv_key_capacity(uint *num_keys,
                                      size_t *keys_length) const {
  if (rust_ctx_) {
    uint32_t n = 0;
    uint64_t b = 0;
    if (rust__handler__mv_key_capacity(rust_ctx_, &n, &b)) {
      *num_keys = n;
      *keys_length = b;
      return;
    }
  }
  // handler::mv_key_capacity is private (NVI); reproduce its trivial base.
  *num_keys = 0;
  *keys_length = 0;
}

Partition_handler *RustHandlerBase::get_partition_handler() {
  if (rust_ctx_) {
    Partition_handler *ph = static_cast<Partition_handler *>(
        rust__handler__get_partition_handler(rust_ctx_));
    if (ph) return ph;
  }
  return handler::get_partition_handler();
}
