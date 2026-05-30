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

static handler *rusty_create_handler(handlerton *hton, TABLE_SHARE *table,
                                     bool, MEM_ROOT *mem_root) {
  return new (mem_root) RustHandlerBase(hton, table);
}

// Mirrors the hand-written `HTON_CAN_RECREATE` value in `src/sys.rs`; the
// Rust accessor returns it as the zero-config default, so a drift in the
// upstream macro would silently change the flag an unregistered engine gets.
static_assert(HTON_CAN_RECREATE == (1u << 2),
              "HTON_CAN_RECREATE drifted; update src/sys.rs HTON_CAN_RECREATE");

extern "C" int rusty_init_func(void *p) {
  DBUG_TRACE;
  rust__plugin_init();
  auto *hton = static_cast<handlerton *>(p);
  hton->state = SHOW_OPTION_YES;
  hton->create = rusty_create_handler;
  hton->flags = rust__hton__flags();
  // Always-on hooks are wired only when an engine registers a Handlerton, so a
  // zero-config engine keeps these handlerton pointers NULL as before.
  if (rust__hton__is_registered()) {
    rusty_hton_wire_lifecycle(hton);
    rusty_hton_wire_status(hton);
    rusty_hton_wire_discovery(hton);
    rusty_hton_wire_notifications(hton);
    rusty_hton_wire_binlog(hton);
    rusty_hton_wire_drop_database(hton);
    rusty_hton_wire_fk_hooks(hton);
    // commit/rollback/prepare are capability-gated: a non-NULL commit is what
    // tells MySQL the engine is transactional, so only wire them when declared.
    if (rust__hton__is_transactional()) {
      rusty_hton_wire_transactions(hton);
      // start_consistent_snapshot only makes sense on a transactional engine;
      // a non-NULL pointer commits the engine to honouring snapshot reads.
      rusty_hton_wire_consistent_snapshot(hton);
    }
    // XA recovery acts on prepared transactions, which require the engine to
    // be transactional, so wire the by-xid callbacks only when both hold;
    // recover stays NULL regardless.
    if (rust__hton__is_transactional() && rust__hton__is_xa()) {
      rusty_hton_wire_xa(hton);
    }
    // Savepoints live inside a transaction, so they likewise require the
    // transactional capability.
    if (rust__hton__is_transactional() && rust__hton__is_savepoints()) {
      rusty_hton_wire_savepoints(hton);
    }
    // partition_flags is the only signal MySQL uses to decide the engine
    // implements handler::get_partition_handler, so leave it NULL unless the
    // engine explicitly opts in via the PARTITIONING capability.
    if (rust__hton__is_partitioning()) {
      rusty_hton_wire_partitioning(hton);
    }
    // Tablespace callbacks must stay NULL on a tablespace-less engine — a
    // non-NULL get_tablespace makes MySQL route tablespace work here.
    if (rust__hton__is_tablespaces()) {
      rusty_hton_wire_tablespaces(hton);
    }
    // Only the storage engine acting as the data dictionary backend (today,
    // just InnoDB) may declare DICT_BACKEND.
    if (rust__hton__is_dict_backend()) {
      rusty_hton_wire_dict(hton);
    }
    // SDI callbacks are only meaningful for engines that own their SDI
    // store (InnoDB-style).
    if (rust__hton__is_sdi()) {
      rusty_hton_wire_sdi(hton);
    }
    // ENGINE_LOG opt-in publishes the engine's redo / transaction log to
    // performance_schema.log_status; a non-log engine keeps these NULL.
    if (rust__hton__is_engine_log()) {
      rusty_hton_wire_engine_log(hton);
    }
  }
  return 0;
}

extern "C" int rusty_deinit_func(void *) {
  DBUG_TRACE;
  return 0;
}

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

int RustHandlerBase::open(const char *name, int mode, uint, const dd::Table *) {
  DBUG_TRACE;
  if (!(share_ = get_share())) return HA_ERR_OUT_OF_MEM;
  thr_lock_data_init(&share_->lock, &lock_data_, nullptr);
  if (!rust_ctx_ || !name) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__open(rust_ctx_,
                             reinterpret_cast<const uint8_t *>(name),
                             shim::safe_name_len(name), mode);
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
                            dd::Table *) {
  DBUG_TRACE;
  if (!rust_ctx_ || !name) return HA_ERR_INTERNAL_ERROR;
  return rust__handler__create(rust_ctx_,
                               reinterpret_cast<const uint8_t *>(name),
                               shim::safe_name_len(name));
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
  return rust__handler__info(rust_ctx_, flag);
}

THR_LOCK_DATA **RustHandlerBase::store_lock(THD *, THR_LOCK_DATA **to,
                                            enum thr_lock_type lock_type) {
  if (lock_type != TL_IGNORE && lock_data_.type == TL_UNLOCK)
    lock_data_.type = lock_type;
  *to++ = &lock_data_;
  return to;
}
