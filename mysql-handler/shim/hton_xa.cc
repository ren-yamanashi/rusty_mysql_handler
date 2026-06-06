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

// XA recovery handlerton callbacks (handler.h #9-#14). The four by-xid /
// set_prepared callbacks pass XID through as an opaque pointer; the Rust side
// returns 0 on success or an HA_ERR code, mapped to xa_status_code here.
// recover_prepared_in_tc passes an opaque Xa_state_list pointer that the
// engine pushes entries into via the mysql__xa_state_list__add reverse
// callback defined below. recover remains deferred — it needs the same
// push-entry pattern over XA_recover_txn[].

#include <algorithm>
#include <cstddef>
#include <cstdint>

#include "binding.hpp"
#include "rust_callbacks.hpp"
#include "sql/xa.h"

namespace {
xa_status_code to_xa_status(int32_t rc) { return rc == 0 ? XA_OK : XAER_RMERR; }

xa_status_code rusty_hton_commit_by_xid(handlerton *, XID *xid) {
  return to_xa_status(rust__hton__commit_by_xid(static_cast<const void *>(xid)));
}

xa_status_code rusty_hton_rollback_by_xid(handlerton *, XID *xid) {
  return to_xa_status(
      rust__hton__rollback_by_xid(static_cast<const void *>(xid)));
}

int rusty_hton_set_prepared_in_tc(handlerton *, THD *thd) {
  return rust__hton__set_prepared_in_tc(static_cast<const void *>(thd));
}

xa_status_code rusty_hton_set_prepared_in_tc_by_xid(handlerton *, XID *xid) {
  return to_xa_status(
      rust__hton__set_prepared_in_tc_by_xid(static_cast<const void *>(xid)));
}

int rusty_hton_recover_prepared_in_tc(handlerton *, Xa_state_list &xa_list) {
  return rust__hton__recover_prepared_in_tc(static_cast<void *>(&xa_list));
}
}  // namespace

// Reverse callback: copy (gtrid, bqual, format_id, state) into a fresh XID
// and feed it to Xa_state_list::add. The XA spec caps gtrid_length and
// bqual_length at 64 bytes each, and XIDDATASIZE is 128; we clamp the
// incoming slices to those limits so a misbehaving engine cannot drive
// XID::set into an assertion-fail or buffer overrun.
extern "C" void mysql__xa_state_list__add(void *xa_list_void, int64_t format_id,
                                          const uint8_t *gtrid_ptr,
                                          size_t gtrid_len,
                                          const uint8_t *bqual_ptr,
                                          size_t bqual_len, int32_t state) {
  if (!xa_list_void) return;
  size_t g = std::min<size_t>(gtrid_len, 64);
  size_t b = std::min<size_t>(bqual_len, 64);
  XID xid;
  xid.set(static_cast<long>(format_id),
          reinterpret_cast<const char *>(gtrid_ptr), static_cast<long>(g),
          reinterpret_cast<const char *>(bqual_ptr), static_cast<long>(b));
  auto *xa_list = static_cast<Xa_state_list *>(xa_list_void);
  xa_list->add(xid, static_cast<enum_ha_recover_xa_state>(state));
}

void rusty_hton_wire_xa(handlerton *hton) {
  hton->commit_by_xid = rusty_hton_commit_by_xid;
  hton->rollback_by_xid = rusty_hton_rollback_by_xid;
  hton->set_prepared_in_tc = rusty_hton_set_prepared_in_tc;
  hton->set_prepared_in_tc_by_xid = rusty_hton_set_prepared_in_tc_by_xid;
  hton->recover_prepared_in_tc = rusty_hton_recover_prepared_in_tc;
}
