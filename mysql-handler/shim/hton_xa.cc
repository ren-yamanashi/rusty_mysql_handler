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

// XA recovery handlerton callbacks (handler.h #9-#14). recover and
// recover_prepared_in_tc are intentionally left NULL (they fill MySQL-owned
// output the opaque pass-through cannot populate). The Rust side returns 0 on
// success or an HA_ERR code; map that to xa_status_code for the by-xid
// callbacks.

#include "binding.hpp"
#include "rust_callbacks.hpp"

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
}  // namespace

void rusty_hton_wire_xa(handlerton *hton) {
  hton->commit_by_xid = rusty_hton_commit_by_xid;
  hton->rollback_by_xid = rusty_hton_rollback_by_xid;
  hton->set_prepared_in_tc = rusty_hton_set_prepared_in_tc;
  hton->set_prepared_in_tc_by_xid = rusty_hton_set_prepared_in_tc_by_xid;
}
