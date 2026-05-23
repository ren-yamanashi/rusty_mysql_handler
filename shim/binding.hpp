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

#ifndef SHIM_BINDING_HPP
#define SHIM_BINDING_HPP

#include "my_base.h"
#include "sql/handler.h"
#include "thr_lock.h"

class RustyShare : public Handler_share {
 public:
  THR_LOCK lock;
  RustyShare() { thr_lock_init(&lock); }
  ~RustyShare() override { thr_lock_delete(&lock); }
};

class RustHandlerBase : public handler {
  THR_LOCK_DATA lock_data_;
  RustyShare *share_ = nullptr;

  RustyShare *get_share();

 public:
  RustHandlerBase(handlerton *hton, TABLE_SHARE *table_arg);
  ~RustHandlerBase() override = default;

  const char *table_type() const override;
  Table_flags table_flags() const override;
  ulong index_flags(uint idx, uint part, bool all_parts) const override;

  int open(const char *name, int mode, uint test_if_locked,
           const dd::Table *table_def) override;
  int close() override;
  int create(const char *name, TABLE *form, HA_CREATE_INFO *create_info,
             dd::Table *table_def) override;

  int rnd_init(bool scan) override;
  int rnd_next(uchar *buf) override;
  int rnd_pos(uchar *buf, uchar *pos) override;
  void position(const uchar *record) override;

  int info(uint flag) override;
  THR_LOCK_DATA **store_lock(THD *thd, THR_LOCK_DATA **to,
                             enum thr_lock_type lock_type) override;
};

int rusty_init_func(void *p);
int rusty_deinit_func(void *p);

#endif
