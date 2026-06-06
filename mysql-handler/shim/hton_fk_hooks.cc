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

// FK compatibility + plugin-observer transaction hooks (handler.h #61,
// #72-#74). Always wired on a registered handlerton — the FK check returns a
// compatibility bool and the se_* hooks are observer notifications, so wiring
// a stub forwarder does not change how MySQL classifies the engine.

#include "binding.hpp"
#include "rust_callbacks.hpp"
#include "sql/handler.h"

namespace {
bool rusty_hton_check_fk_column_compat(const Ha_fk_column_type *child,
                                       const Ha_fk_column_type *parent,
                                       bool check_charsets) {
  return rust__hton__check_fk_column_compat(static_cast<const void *>(child),
                                            static_cast<const void *>(parent),
                                            check_charsets);
}

void rusty_hton_se_before_commit(void *arg) {
  rust__hton__se_before_commit(arg);
}

void rusty_hton_se_after_commit(void *arg) {
  rust__hton__se_after_commit(arg);
}

void rusty_hton_se_before_rollback(void *arg) {
  rust__hton__se_before_rollback(arg);
}
}  // namespace

void rusty_hton_wire_fk_hooks(handlerton *hton) {
  hton->check_fk_column_compat = rusty_hton_check_fk_column_compat;
  hton->se_before_commit = rusty_hton_se_before_commit;
  hton->se_after_commit = rusty_hton_se_after_commit;
  hton->se_before_rollback = rusty_hton_se_before_rollback;
}
