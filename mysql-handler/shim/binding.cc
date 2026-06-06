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

#include "binding.hpp"

#include <new>

#include "my_dbug.h"
#include "mysql/plugin.h"
#include "rust_callbacks.hpp"
#include "safe_name.hpp"
#include "sql/table.h"

// The Rust-side plugin manifest hand-declares `StMysqlPlugin` to mirror this
// layout. Pinning the size and alignment here catches any C++-side header
// drift at compile time; the runtime `_mysql_sizeof_struct_st_plugin_` symbol
// catches the converse (Rust side diverging from the C++ header).
#if defined(__LP64__) || defined(_LP64)
static_assert(sizeof(st_mysql_plugin) == 112,
              "st_mysql_plugin layout drifted; update plugin_manifest in "
              "examples/engine/src/lib.rs");
static_assert(alignof(st_mysql_plugin) == 8,
              "st_mysql_plugin alignment drifted; update plugin_manifest in "
              "examples/engine/src/lib.rs");
#endif

RustyShare::RustyShare() { thr_lock_init(&lock); }
RustyShare::~RustyShare() { thr_lock_delete(&lock); }

// `rust_ctx_` is null only when no engine factory was registered. `create`
// and `open` reject that case; later methods may dereference unconditionally.
RustHandlerBase::RustHandlerBase(handlerton *hton, TABLE_SHARE *table_arg)
    : handler(hton, table_arg) {
  rust_ctx_ = rust__create_engine();
}

RustHandlerBase::~RustHandlerBase() {
  if (rust_ctx_) {
    rust__destroy_engine(rust_ctx_);
  }
}

RustyShare *RustHandlerBase::get_share() {
  RustyShare *tmp_share;
  DBUG_TRACE;
  lock_shared_ha_data();
  if (!(tmp_share = static_cast<RustyShare *>(get_ha_share_ptr()))) {
    tmp_share = new (std::nothrow) RustyShare;
    if (tmp_share) {
      set_ha_share_ptr(static_cast<Handler_share *>(tmp_share));
    }
  }
  unlock_shared_ha_data();
  return tmp_share;
}

const char *RustHandlerBase::table_type() const {
  // Always return a name so SHOW ENGINES lists the plugin even when the Rust
  // factory failed to register (rust_ctx_ == nullptr).
  if (rust_ctx_) return rust__handler__table_type(rust_ctx_);
  return "RUSTY";
}

handler::Table_flags RustHandlerBase::table_flags() const {
  if (rust_ctx_) return rust__handler__table_flags(rust_ctx_);
  return 0;
}

ulong RustHandlerBase::index_flags(uint idx, uint part, bool all_parts) const {
  if (rust_ctx_) return rust__handler__index_flags(rust_ctx_, idx, part, all_parts);
  return 0;
}

int RustHandlerBase::open(const char *name, int mode, uint,
                          const dd::Table *table_def) {
  DBUG_TRACE;
  if (!(share_ = get_share())) return HA_ERR_OUT_OF_MEM;
  thr_lock_data_init(&share_->lock, &lock_data_, nullptr);
  if (!rust_ctx_ || !name) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__open(rust_ctx_,
                             reinterpret_cast<const uint8_t *>(name),
                             shim::safe_name_len(name), mode,
                             static_cast<const void *>(table_def));
}

// Reachable from the `handler::drop_table` chain even when no Rust engine has
// been bound; treat that case as already-closed rather than calling into Rust
// with a null context.
int RustHandlerBase::close() {
  DBUG_TRACE;
  if (!rust_ctx_) return 0;
  return rust__handler__close(rust_ctx_);
}

int RustHandlerBase::create(const char *name, TABLE *, HA_CREATE_INFO *,
                            dd::Table *table_def) {
  DBUG_TRACE;
  if (!rust_ctx_ || !name) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__create(rust_ctx_,
                               reinterpret_cast<const uint8_t *>(name),
                               shim::safe_name_len(name),
                               static_cast<const void *>(table_def));
}

int RustHandlerBase::rnd_init(bool scan) {
  DBUG_TRACE;
  return rust__handler__rnd_init(rust_ctx_, scan);
}

int RustHandlerBase::rnd_end() {
  DBUG_TRACE;
  return rust__handler__rnd_end(rust_ctx_);
}

int RustHandlerBase::rnd_next(uchar *buf) {
  DBUG_TRACE;
  return rust__handler__rnd_next(rust_ctx_, buf, table->s->rec_buff_length);
}

int RustHandlerBase::rnd_pos(uchar *buf, uchar *pos) {
  DBUG_TRACE;
  return rust__handler__rnd_pos(rust_ctx_, buf, table->s->rec_buff_length,
                                pos, ref_length);
}

void RustHandlerBase::position(const uchar *record) {
  DBUG_TRACE;
  // record is the row buffer (rec_buff_length); ref is the position output
  // buffer the base class allocated to ref_length bytes in ha_open().
  rust__handler__position(rust_ctx_, record, table->s->rec_buff_length, ref,
                          ref_length);
}

int RustHandlerBase::rnd_pos_by_record(uchar *record) {
  DBUG_TRACE;
  return rust__handler__rnd_pos_by_record(rust_ctx_, record,
                                          table->s->rec_buff_length);
}

int RustHandlerBase::info(uint flag) {
  DBUG_TRACE;
  int rc = rust__handler__info(rust_ctx_, flag);
  // Refresh stats.records on HA_STATUS_VARIABLE so the optimizer sees
  // real rows. Bail early if engine info() failed (leaving stats partly
  // stale would be worse than leaving them as-is) or if the call is
  // not the variable-stats slot.
  if (rc != 0 || !(flag & HA_STATUS_VARIABLE) || !rust_ctx_ || !table ||
      !table->s) {
    return rc;
  }
  uint64_t n = 0;
  bool handled = false;
  if (rust__handler__records(rust_ctx_, &n, &handled) != 0 || !handled) {
    return rc;
  }
  stats.records = static_cast<ha_rows>(n);
  stats.mean_rec_length = table->s->rec_buff_length;
  stats.data_file_length =
      static_cast<ulonglong>(stats.records) * stats.mean_rec_length;
  stats.deleted = 0;
  return rc;
}

THR_LOCK_DATA **RustHandlerBase::store_lock(THD *, THR_LOCK_DATA **to,
                                            enum thr_lock_type lock_type) {
  enum thr_lock_type chosen = lock_type;
  if (rust_ctx_) {
    chosen = static_cast<enum thr_lock_type>(
        rust__handler__store_lock(rust_ctx_, static_cast<int32_t>(lock_type)));
  }
  if (chosen != TL_IGNORE && lock_data_.type == TL_UNLOCK)
    lock_data_.type = chosen;
  *to++ = &lock_data_;
  return to;
}
