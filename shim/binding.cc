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

#include "my_dbug.h"
#include "mysql/plugin.h"

static handlerton *rusty_hton = nullptr;

static handler *rusty_create_handler(handlerton *hton, TABLE_SHARE *table,
                                     bool, MEM_ROOT *mem_root) {
  return new (mem_root) RustHandlerBase(hton, table);
}

int rusty_init_func(void *p) {
  DBUG_TRACE;
  rusty_hton = static_cast<handlerton *>(p);
  rusty_hton->state = SHOW_OPTION_YES;
  rusty_hton->create = rusty_create_handler;
  rusty_hton->flags = HTON_CAN_RECREATE;
  return 0;
}

int rusty_deinit_func(void *p [[maybe_unused]]) {
  DBUG_TRACE;
  return 0;
}

RustHandlerBase::RustHandlerBase(handlerton *hton, TABLE_SHARE *table_arg)
    : handler(hton, table_arg) {}

RustyShare *RustHandlerBase::get_share() {
  RustyShare *tmp_share;
  DBUG_TRACE;
  lock_shared_ha_data();
  if (!(tmp_share = static_cast<RustyShare *>(get_ha_share_ptr()))) {
    tmp_share = new RustyShare;
    if (tmp_share) {
      set_ha_share_ptr(static_cast<Handler_share *>(tmp_share));
    }
  }
  unlock_shared_ha_data();
  return tmp_share;
}

const char *RustHandlerBase::table_type() const { return "RUSTY"; }

handler::Table_flags RustHandlerBase::table_flags() const {
  return HA_BINLOG_STMT_CAPABLE;
}

ulong RustHandlerBase::index_flags(uint, uint, bool) const { return 0; }

int RustHandlerBase::open(const char *, int, uint, const dd::Table *) {
  DBUG_TRACE;
  if (!(share_ = get_share())) return 1;
  thr_lock_data_init(&share_->lock, &lock_data_, nullptr);
  return 0;
}

int RustHandlerBase::close() {
  DBUG_TRACE;
  return 0;
}

int RustHandlerBase::create(const char *, TABLE *, HA_CREATE_INFO *,
                            dd::Table *) {
  DBUG_TRACE;
  return 0;
}

int RustHandlerBase::rnd_init(bool) {
  DBUG_TRACE;
  return 0;
}

int RustHandlerBase::rnd_next(uchar *) {
  DBUG_TRACE;
  return HA_ERR_END_OF_FILE;
}

int RustHandlerBase::rnd_pos(uchar *, uchar *) {
  DBUG_TRACE;
  return HA_ERR_WRONG_COMMAND;
}

void RustHandlerBase::position(const uchar *) { DBUG_TRACE; }

int RustHandlerBase::info(uint) {
  DBUG_TRACE;
  return 0;
}

THR_LOCK_DATA **RustHandlerBase::store_lock(THD *, THR_LOCK_DATA **to,
                                            enum thr_lock_type lock_type) {
  if (lock_type != TL_IGNORE && lock_data_.type == TL_UNLOCK)
    lock_data_.type = lock_type;
  *to++ = &lock_data_;
  return to;
}
