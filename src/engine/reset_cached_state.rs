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

//! Cached-state reset signal passed to engine-private metadata callbacks.

/// Whether MySQL has just reset the data-dictionary entry and any cached
/// engine-private metadata should be re-emitted from scratch
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResetCachedState {
    /// Reuse whatever the engine has cached
    Keep,
    /// Discard cached state and re-emit from authoritative source
    Reset,
}

impl From<bool> for ResetCachedState {
    fn from(needs_reset: bool) -> Self {
        if needs_reset { Self::Reset } else { Self::Keep }
    }
}
