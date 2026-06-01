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

//! Reference storage engine for `mysql-handler`. [`TrivialEngine`] is the
//! `StorageEngine` impl orchestrator under `engine/`, `trivial_handlerton`
//! holds the engine-level [`TrivialHandlerton`], and `registration` is
//! the plugin entry point.

#![allow(unsafe_code)]

#[cfg(not(test))]
#[doc(hidden)]
#[allow(missing_docs, missing_debug_implementations)]
pub mod plugin_manifest;

pub mod engine;
#[doc(hidden)]
pub mod registration;
pub mod store;
pub mod trivial_handlerton;
pub mod trivial_txn;

pub use engine::TrivialEngine;
pub use trivial_handlerton::TrivialHandlerton;
pub use trivial_txn::TrivialTxn;
