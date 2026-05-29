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

#ifndef SHIM_RUST_CALLBACKS_HTON_XA_HPP
#define SHIM_RUST_CALLBACKS_HTON_XA_HPP

#include <cstdint>

// XA recovery callbacks. XID / THD cross as opaque `const void *`. Each returns
// 0 on success or an HA_ERR code; the shim maps that to xa_status_code for the
// by-xid callbacks. recover / recover_prepared_in_tc are deliberately not bound
// (they fill MySQL-owned output the opaque pass-through cannot populate).
extern "C" {
int32_t rust__hton__commit_by_xid(const void *xid);
int32_t rust__hton__rollback_by_xid(const void *xid);
int32_t rust__hton__set_prepared_in_tc(const void *thd);
int32_t rust__hton__set_prepared_in_tc_by_xid(const void *xid);
}

#endif
