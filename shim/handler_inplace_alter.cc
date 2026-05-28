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

// In-place ALTER TABLE overrides (handler.h #124-#129)

#include "binding.hpp"
#include "rust_callbacks.hpp"

// Each override lets the engine drive its part of the in-place ALTER protocol,
// falling back to the handler base when it declines so the default in-place /
// copy ALTER flow stays intact.

enum_alter_inplace_result RustHandlerBase::check_if_supported_inplace_alter(
    TABLE *altered_table, Alter_inplace_info *ha_alter_info) {
  if (rust_ctx_) {
    int32_t v = 0;
    if (rust__handler__check_if_supported_inplace_alter(
            rust_ctx_, static_cast<const void *>(altered_table),
            static_cast<const void *>(ha_alter_info), &v))
      return static_cast<enum_alter_inplace_result>(v);
  }
  return handler::check_if_supported_inplace_alter(altered_table, ha_alter_info);
}

bool RustHandlerBase::prepare_inplace_alter_table(
    TABLE *altered_table, Alter_inplace_info *ha_alter_info,
    const dd::Table *old_table_def, dd::Table *new_table_def) {
  if (rust_ctx_) {
    bool v = false;
    if (rust__handler__prepare_inplace_alter_table(
            rust_ctx_, static_cast<const void *>(altered_table),
            static_cast<const void *>(ha_alter_info),
            static_cast<const void *>(old_table_def),
            static_cast<const void *>(new_table_def), &v))
      return v;
  }
  return handler::prepare_inplace_alter_table(altered_table, ha_alter_info,
                                              old_table_def, new_table_def);
}

bool RustHandlerBase::inplace_alter_table(TABLE *altered_table,
                                          Alter_inplace_info *ha_alter_info,
                                          const dd::Table *old_table_def,
                                          dd::Table *new_table_def) {
  if (rust_ctx_) {
    bool v = false;
    if (rust__handler__inplace_alter_table(
            rust_ctx_, static_cast<const void *>(altered_table),
            static_cast<const void *>(ha_alter_info),
            static_cast<const void *>(old_table_def),
            static_cast<const void *>(new_table_def), &v))
      return v;
  }
  return handler::inplace_alter_table(altered_table, ha_alter_info,
                                      old_table_def, new_table_def);
}

bool RustHandlerBase::commit_inplace_alter_table(
    TABLE *altered_table, Alter_inplace_info *ha_alter_info, bool commit,
    const dd::Table *old_table_def, dd::Table *new_table_def) {
  if (rust_ctx_) {
    bool v = false;
    if (rust__handler__commit_inplace_alter_table(
            rust_ctx_, static_cast<const void *>(altered_table),
            static_cast<const void *>(ha_alter_info), commit,
            static_cast<const void *>(old_table_def),
            static_cast<const void *>(new_table_def), &v))
      return v;
  }
  return handler::commit_inplace_alter_table(
      altered_table, ha_alter_info, commit, old_table_def, new_table_def);
}

void RustHandlerBase::notify_table_changed(Alter_inplace_info *ha_alter_info) {
  if (rust_ctx_) {
    rust__handler__notify_table_changed(
        rust_ctx_, static_cast<const void *>(ha_alter_info));
    return;
  }
  handler::notify_table_changed(ha_alter_info);
}

bool RustHandlerBase::check_if_incompatible_data(HA_CREATE_INFO *create_info,
                                                 uint table_changes) {
  if (rust_ctx_) {
    bool v = false;
    if (rust__handler__check_if_incompatible_data(
            rust_ctx_, static_cast<const void *>(create_info), table_changes,
            &v))
      return v;
  }
  return handler::check_if_incompatible_data(create_info, table_changes);
}
