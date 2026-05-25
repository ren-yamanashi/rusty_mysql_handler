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

//! `rust__handler__*` callbacks invoked by the C++ shim, split by handler-API
//! category. Each submodule holds the callbacks for one section of
//! `docs/api/handler.md`.
//!
//! # Safety (every callback in these submodules)
//!
//! - `ctx` comes from `rust__create_engine` and has not been destroyed; the
//!   C++ shim guards every callback against null on its side, so each Rust
//!   callback requires non-null.
//! - The shim never calls a callback for the same `ctx` from two threads
//!   concurrently, so `&mut *ctx` is sound inside each callback.
//! - Pointer/length pairs are valid for the call only; engines must not
//!   retain them.

#[doc(hidden)]
pub mod bulk_operations;
#[doc(hidden)]
pub mod index_basic;
#[doc(hidden)]
pub mod index_range;
#[doc(hidden)]
pub mod open_close;
#[doc(hidden)]
pub mod properties;
#[doc(hidden)]
pub mod row_operations;
#[doc(hidden)]
pub mod scan;
#[doc(hidden)]
pub mod statistics;
#[doc(hidden)]
pub mod table_lifecycle;
