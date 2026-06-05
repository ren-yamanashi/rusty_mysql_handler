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

//! Capability sub-trait reserved for secondary-engine handler callbacks.

use super::StorageEngine;

/// Opt-in sub-trait reserved for secondary-engine `load_table` /
/// `unload_table` and the handlerton-level secondary-engine callbacks.
///
/// The trait is intentionally empty in this revision: the relevant methods
/// stay on [`StorageEngine`] and the handlerton today, with the migration
/// scheduled for a follow-up cycle. The marker exists so
/// [`EngineCapabilities`] can advertise secondary-engine intent and the
/// `#[plugin]` macro can accept `capabilities = [Secondary]` without rewiring
/// the underlying callback dispatch.
///
/// [`EngineCapabilities`]: crate::engine::EngineCapabilities
pub trait SecondaryEngine: StorageEngine {}
