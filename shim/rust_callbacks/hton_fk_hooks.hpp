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

#ifndef SHIM_RUST_CALLBACKS_HTON_FK_HOOKS_HPP
#define SHIM_RUST_CALLBACKS_HTON_FK_HOOKS_HPP

// Engine-level FK compatibility check + plugin-observer transaction hooks.
// Ha_fk_column_type crosses as opaque `const void *`; the observer hook `arg`
// crosses as the same `void *` MySQL hands in, not retained past the call.
extern "C" {
bool rust__hton__check_fk_column_compat(const void *child, const void *parent,
                                        bool check_charsets);
void rust__hton__se_before_commit(void *arg);
void rust__hton__se_after_commit(void *arg);
void rust__hton__se_before_rollback(void *arg);
}

#endif
