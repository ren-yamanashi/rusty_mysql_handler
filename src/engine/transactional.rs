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

//! Capability sub-trait reserved for handlerton-level transaction hooks.

use super::StorageEngine;

/// Opt-in sub-trait that exposes the engine's transaction-driving surface to
/// the FFI boundary.
///
/// The trait is intentionally empty in this revision: handler-level methods
/// stay on [`StorageEngine`], and the matching handlerton txn / savepoint /
/// XA hooks still live on [`Handlerton`](crate::hton::Handlerton) until a
/// later cycle folds them in. The marker exists so [`EngineCapabilities`]
/// can advertise transactional intent and the `#[plugin]` macro can wire
/// `capabilities = [Transactional]` today without breaking churn when the
/// methods migrate.
///
/// [`EngineCapabilities`]: crate::engine::EngineCapabilities
pub trait TransactionalEngine: StorageEngine {}
