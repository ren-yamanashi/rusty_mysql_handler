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

#ifndef SHIM_BINDGEN_INPUT_H
#define SHIM_BINDGEN_INPUT_H

// HA_ERR_* error codes.
#include "my_base.h"

// Mirrored from sql/handler.h (server-internal dependencies make the original
// header unsuitable for direct bindgen consumption). HA_BINLOG_* flag values
// live in src/sys.rs as `u64` constants because MySQL's `Table_flags` is
// `unsigned long long`; defining them here as macros would force bindgen's
// signed default to widen them to `i64`.
typedef unsigned long long Table_flags;

// Mirrored from thr_lock.h (avoids the mysql/psi/* dependency chain).
enum thr_lock_type {
  TL_IGNORE = -1,
  TL_UNLOCK,
  TL_READ_DEFAULT,
  TL_READ,
  TL_READ_WITH_SHARED_LOCKS,
  TL_READ_HIGH_PRIORITY,
  TL_READ_NO_INSERT,
  TL_WRITE_ALLOW_WRITE,
  TL_WRITE_CONCURRENT_DEFAULT,
  TL_WRITE_CONCURRENT_INSERT,
  TL_WRITE_DEFAULT,
  TL_WRITE_LOW_PRIORITY,
  TL_WRITE,
  TL_WRITE_ONLY
};

#endif
