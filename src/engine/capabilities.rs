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

//! Capability dispatcher super-trait used to advertise optional sub-traits.

use super::StorageEngine;
use super::indexed::IndexedEngine;

/// Capability dispatcher every engine implements alongside [`StorageEngine`].
///
/// [`as_indexed`](Self::as_indexed) returns `Some(self)` when the engine opts
/// into [`IndexedEngine`]; the default returns `None` so the FFI boundary can
/// fall back to `HA_ERR_WRONG_COMMAND` (or each callback's documented base
/// behaviour) without dragging the unsupported callback set into every
/// implementation.
///
/// Downstream crates do not implement this trait by hand; the [`plugin`]
/// attribute macro emits an explicit impl whose `as_*` overrides reflect the
/// `capabilities = [...]` list on the macro invocation. A blanket impl over
/// [`StorageEngine`] is intentionally absent because it would collide with the
/// macro-generated specialisations.
///
/// [`plugin`]: mysql_handler_macros::plugin
pub trait EngineCapabilities: StorageEngine {
    /// Engine-supplied [`IndexedEngine`] view when index callbacks are wired,
    /// or `None` to leave them at the base behaviour.
    fn as_indexed(&mut self) -> Option<&mut dyn IndexedEngine> {
        None
    }
}
