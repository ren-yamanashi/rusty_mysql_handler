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

#ifndef SHIM_SAFE_NAME_HPP
#define SHIM_SAFE_NAME_HPP

#include <cstddef>
#include <cstring>

namespace shim {

// Handler API names are documented as null-terminated; strnlen with this cap
// keeps the scan bounded if that contract is ever violated.
constexpr std::size_t MAX_NAME_LEN = 4096;

inline std::size_t safe_name_len(const char *name) {
  return ::strnlen(name, MAX_NAME_LEN);
}

}  // namespace shim

#endif
