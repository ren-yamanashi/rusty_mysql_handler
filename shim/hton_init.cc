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

// Handlerton lifecycle entry points. `rusty_init_func` is the
// plugin-manifest-declared init callback that MySQL invokes once at plugin
// load; it queries the Rust side for the engine's capabilities and wires the
// corresponding handlerton callback pointers. `rusty_deinit_func` is the
// matching unload hook (no-op today). Split out of binding.cc so the
// handler-instance binding code stays focused.

#include "binding.hpp"
#include "my_dbug.h"
#include "rust_callbacks.hpp"

// Mirrors the hand-written `HTON_CAN_RECREATE` value in `src/sys.rs`; the
// Rust accessor returns it as the zero-config default, so a drift in the
// upstream macro would silently change the flag an unregistered engine gets.
static_assert(HTON_CAN_RECREATE == (1u << 2),
              "HTON_CAN_RECREATE drifted; update src/sys.rs HTON_CAN_RECREATE");

static handler *rusty_create_handler(handlerton *hton, TABLE_SHARE *table,
                                     bool, MEM_ROOT *mem_root) {
  return new (mem_root) RustHandlerBase(hton, table);
}

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
    rusty_hton_wire_misc(hton);
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
    // redo_log_set_state lives under the same capability since both signal
    // the engine owns a redo log surface.
    if (rust__hton__is_engine_log()) {
      rusty_hton_wire_engine_log(hton);
      rusty_hton_wire_redo_log_set_state(hton);
    }
    // ENCRYPTION opt-in puts the engine on MySQL's master-key rotation path;
    // non-encrypting engines keep rotate_encryption_master_key NULL.
    if (rust__hton__is_encryption()) {
      rusty_hton_wire_encryption(hton);
    }
    // SECONDARY_ENGINE opt-in connects the engine to MySQL's offload /
    // hypergraph-optimizer interface (RAPID / HeatWave style); a primary
    // engine keeps these NULL.
    if (rust__hton__is_secondary_engine()) {
      rusty_hton_wire_secondary_engine(hton);
    }
  }
  return 0;
}

extern "C" int rusty_deinit_func(void *) {
  DBUG_TRACE;
  return 0;
}
