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

// Clone-interface sub-callbacks (handler.h Clone_interface_t). Wired only
// under HtonCapabilities::CLONE — the wire function assigns
// hton->clone_interface as a unit. The in/out locator parameters (`loc`,
// `loc_len`, `task_id`) cannot be round-tripped through the opaque
// pass-through today: the shim writes empty defaults back to MySQL so the
// call returns without crashing, and engines that drive a real clone session
// will need a reverse-callback surface for the locator and data channels.

#include <cstring>

#include "binding.hpp"
#include "rust_callbacks.hpp"
#include "sql/handler.h"

namespace {
const uint8_t *nz(const char *s) {
  return reinterpret_cast<const uint8_t *>(s ? s : "");
}
size_t safe_len(const char *s) { return s ? std::strlen(s) : 0u; }

void rusty_hton_clone_capability(Ha_clone_flagset &flags) {
  uint64_t out = 0;
  rust__hton__clone_capability(&out);
  flags = Ha_clone_flagset{out};
}

int rusty_hton_clone_begin(handlerton *, THD *thd, const uchar *&loc,
                           uint &loc_len, uint &task_id, Ha_clone_type type,
                           Ha_clone_mode mode) {
  // Locator out-params cannot be filled through the opaque pass-through;
  // leave them empty so MySQL observes "no clone session" if the Rust side
  // returns success without a real implementation.
  loc = nullptr;
  loc_len = 0;
  task_id = 0;
  return rust__hton__clone_begin(static_cast<const void *>(thd),
                                 static_cast<size_t>(type),
                                 static_cast<uint32_t>(mode));
}

int rusty_hton_clone_copy(handlerton *, THD *thd, const uchar *, uint,
                          uint task_id, Ha_clone_cbk *cbk) {
  return rust__hton__clone_copy(static_cast<const void *>(thd), task_id,
                                static_cast<const void *>(cbk));
}

int rusty_hton_clone_ack(handlerton *, THD *thd, const uchar *, uint,
                         uint task_id, int in_err, Ha_clone_cbk *cbk) {
  return rust__hton__clone_ack(static_cast<const void *>(thd), task_id, in_err,
                               static_cast<const void *>(cbk));
}

int rusty_hton_clone_end(handlerton *, THD *thd, const uchar *, uint,
                         uint task_id, int in_err) {
  return rust__hton__clone_end(static_cast<const void *>(thd), task_id,
                               in_err);
}

int rusty_hton_clone_apply_begin(handlerton *, THD *thd, const uchar *&loc,
                                 uint &loc_len, uint &task_id,
                                 Ha_clone_mode mode, const char *data_dir) {
  loc = nullptr;
  loc_len = 0;
  task_id = 0;
  return rust__hton__clone_apply_begin(static_cast<const void *>(thd),
                                       static_cast<uint32_t>(mode),
                                       nz(data_dir), safe_len(data_dir));
}

int rusty_hton_clone_apply(handlerton *, THD *thd, const uchar *, uint,
                           uint task_id, int in_err, Ha_clone_cbk *cbk) {
  return rust__hton__clone_apply(static_cast<const void *>(thd), task_id,
                                 in_err, static_cast<const void *>(cbk));
}

int rusty_hton_clone_apply_end(handlerton *, THD *thd, const uchar *, uint,
                               uint task_id, int in_err) {
  return rust__hton__clone_apply_end(static_cast<const void *>(thd), task_id,
                                     in_err);
}
}  // namespace

void rusty_hton_wire_clone(handlerton *hton) {
  hton->clone_interface.clone_capability = rusty_hton_clone_capability;
  hton->clone_interface.clone_begin = rusty_hton_clone_begin;
  hton->clone_interface.clone_copy = rusty_hton_clone_copy;
  hton->clone_interface.clone_ack = rusty_hton_clone_ack;
  hton->clone_interface.clone_end = rusty_hton_clone_end;
  hton->clone_interface.clone_apply_begin = rusty_hton_clone_apply_begin;
  hton->clone_interface.clone_apply = rusty_hton_clone_apply;
  hton->clone_interface.clone_apply_end = rusty_hton_clone_apply_end;
}
