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

//! Engine-level handlerton interface.
//!
//! Where [`StorageEngine`](crate::engine::StorageEngine) is per-table, the
//! handlerton is a per-engine singleton: one instance serves every connection.
//! An engine opts into engine-level behaviour by implementing [`Handlerton`]
//! and registering it with
//! [`register_handlerton`](crate::runtime::register_handlerton). Engines that
//! need nothing beyond table handling skip registration and keep the
//! zero-config defaults.

mod capabilities;
#[doc(hidden)]
pub mod ffi;
mod flags;

pub use capabilities::HtonCapabilities;
pub use flags::HtonFlags;

/// The engine-level handlerton interface: the capabilities and `handlerton`
/// struct fields that apply to the engine as a whole rather than to a single
/// table.
///
/// Every method has a default, so an engine implements only what it needs and
/// an empty `impl Handlerton for MyEngine {}` is a valid handler-only
/// handlerton. The singleton is shared across all connection threads, hence
/// the `Send + Sync` bound — do not relax it.
///
/// # Examples
///
/// ```
/// use mysql_handler::hton::{Handlerton, HtonCapabilities, HtonFlags};
///
/// struct MyHandlerton;
/// impl Handlerton for MyHandlerton {
///     fn capabilities(&self) -> HtonCapabilities {
///         HtonCapabilities::TRANSACTIONS
///     }
/// }
///
/// assert!(MyHandlerton.capabilities().contains(HtonCapabilities::TRANSACTIONS));
/// assert_eq!(MyHandlerton.flags(), HtonFlags::CAN_RECREATE);
/// ```
pub trait Handlerton: Send + Sync {
    /// The engine-level callback groups this handlerton implements.
    ///
    /// Each capability gates a group of `handlerton` callbacks; a group is
    /// wired into MySQL only when its bit is set here. Defaults to
    /// [`HtonCapabilities::empty`] — a handler-only engine.
    fn capabilities(&self) -> HtonCapabilities {
        HtonCapabilities::empty()
    }

    /// The `handlerton` flags (`HTON_*`).
    ///
    /// Defaults to [`HtonFlags::CAN_RECREATE`], matching the flag the
    /// zero-config engine sets today. Return [`HtonFlags::NONE`] to opt out.
    fn flags(&self) -> HtonFlags {
        HtonFlags::CAN_RECREATE
    }
}
