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

//! Engine-level `handlerton` flags (the `HTON_*` bits).

use crate::sys;

/// A set of `handlerton` flags (the `HTON_*` bits from `sql/handler.h`).
///
/// Returned from [`Handlerton::flags`](crate::hton::Handlerton::flags). The
/// zero-config engine sets [`HtonFlags::CAN_RECREATE`], which is therefore the
/// trait default; return [`HtonFlags::NONE`] to clear it.
///
/// ```
/// use mysql_handler::hton::HtonFlags;
///
/// let f = HtonFlags::NONE | HtonFlags::CAN_RECREATE;
/// assert!(f.contains(HtonFlags::CAN_RECREATE));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct HtonFlags(u32);

impl HtonFlags {
    /// No flags set (`HTON_NO_FLAGS`)
    pub const NONE: Self = Self(0);
    /// `HTON_CAN_RECREATE`: the engine implements `TRUNCATE` by recreating the
    /// table. The flag the zero-config engine sets today.
    pub const CAN_RECREATE: Self = Self(sys::HTON_CAN_RECREATE);

    /// An empty flag set
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// The raw bits, for handing the set across the FFI boundary
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Whether every flag in `other` is set in `self`
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// The union of two flag sets
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl core::ops::BitOr for HtonFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_recreate_matches_sys_constant() {
        assert_eq!(HtonFlags::CAN_RECREATE.bits(), sys::HTON_CAN_RECREATE);
    }

    #[test]
    fn none_is_empty() {
        assert_eq!(HtonFlags::NONE, HtonFlags::empty());
        assert_eq!(HtonFlags::empty().bits(), 0);
    }

    #[test]
    fn union_and_contains() {
        let f = HtonFlags::NONE | HtonFlags::CAN_RECREATE;
        assert!(f.contains(HtonFlags::CAN_RECREATE));
        assert!(HtonFlags::NONE.contains(HtonFlags::NONE));
        assert!(!HtonFlags::NONE.contains(HtonFlags::CAN_RECREATE));
    }
}
