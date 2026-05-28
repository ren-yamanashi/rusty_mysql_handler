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

#ifndef SHIM_RUST_CALLBACKS_INPLACE_ALTER_HPP
#define SHIM_RUST_CALLBACKS_INPLACE_ALTER_HPP

#include <cstdint>

// In-place ALTER TABLE methods (handler.h #124-#129). Each bool-returning
// callback reports true when the engine overrides (result written through the
// out-pointer) and false to fall back to the handler base. TABLE,
// Alter_inplace_info, dd::Table and HA_CREATE_INFO all cross as opaque
// `const void *`.
extern "C" {
bool rust__handler__check_if_supported_inplace_alter(void *ctx,
                                                     const void *altered_table,
                                                     const void *alter_info,
                                                     int32_t *out);
bool rust__handler__prepare_inplace_alter_table(
    void *ctx, const void *altered_table, const void *alter_info,
    const void *old_table_def, const void *new_table_def, bool *out);
bool rust__handler__inplace_alter_table(void *ctx, const void *altered_table,
                                        const void *alter_info,
                                        const void *old_table_def,
                                        const void *new_table_def, bool *out);
bool rust__handler__commit_inplace_alter_table(
    void *ctx, const void *altered_table, const void *alter_info, bool commit,
    const void *old_table_def, const void *new_table_def, bool *out);
void rust__handler__notify_table_changed(void *ctx, const void *alter_info);
bool rust__handler__check_if_incompatible_data(void *ctx,
                                               const void *create_info,
                                               uint32_t table_changes,
                                               bool *out);
}

#endif
