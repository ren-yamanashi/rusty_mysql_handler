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

//! Engine-level capabilities declared by a [`Handlerton`](crate::hton::Handlerton).

/// The set of engine-level features a [`Handlerton`](crate::hton::Handlerton)
/// opts into.
///
/// Each capability gates a group of `handlerton` callbacks. A group will be
/// wired into the `handlerton` struct only when its bit is set here, because
/// MySQL reads a non-NULL function pointer as a declaration that the engine
/// supports that feature — a non-NULL `commit`, for example, marks the engine
/// transactional. Declaring a capability the engine does not implement would
/// route work to callbacks that cannot honour it, so default to the smallest
/// set that is actually backed by code.
///
/// Combine capabilities with `|`:
///
/// ```
/// use mysql_handler::hton::HtonCapabilities;
///
/// let caps = HtonCapabilities::TRANSACTIONS | HtonCapabilities::SAVEPOINTS;
/// assert!(caps.contains(HtonCapabilities::TRANSACTIONS));
/// assert!(!caps.contains(HtonCapabilities::XA));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct HtonCapabilities(u32);

impl HtonCapabilities {
    /// Transaction callbacks (`commit`, `rollback`, `prepare`)
    pub const TRANSACTIONS: Self = Self(1 << 0);
    /// XA / 2PC recovery callbacks (`recover`, `commit_by_xid`, ...)
    pub const XA: Self = Self(1 << 1);
    /// Savepoint callbacks (`savepoint_set`, `savepoint_rollback`, ...)
    pub const SAVEPOINTS: Self = Self(1 << 2);
    /// Serialized Dictionary Information callbacks (`sdi_*`)
    pub const SDI: Self = Self(1 << 3);
    /// Secondary-engine callbacks (`prepare_secondary_engine`, ...)
    pub const SECONDARY_ENGINE: Self = Self(1 << 4);
    /// Clone-interface sub-callbacks
    pub const CLONE: Self = Self(1 << 5);
    /// Page-tracking sub-callbacks
    pub const PAGE_TRACKING: Self = Self(1 << 6);

    /// An empty capability set: a handler-only engine (the zero-config default)
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// The raw bits, for handing the set across the FFI boundary
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Whether every capability in `other` is present in `self`
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// The union of two capability sets
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl core::ops::BitOr for HtonCapabilities {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_contains_only_empty() {
        let e = HtonCapabilities::empty();
        assert_eq!(e.bits(), 0);
        assert!(e.contains(HtonCapabilities::empty()));
        assert!(!e.contains(HtonCapabilities::TRANSACTIONS));
    }

    #[test]
    fn union_sets_both_bits() {
        let c = HtonCapabilities::TRANSACTIONS | HtonCapabilities::SAVEPOINTS;
        assert!(c.contains(HtonCapabilities::TRANSACTIONS));
        assert!(c.contains(HtonCapabilities::SAVEPOINTS));
        assert!(!c.contains(HtonCapabilities::XA));
    }

    #[test]
    fn each_capability_has_a_distinct_bit() {
        let all = [
            HtonCapabilities::TRANSACTIONS,
            HtonCapabilities::XA,
            HtonCapabilities::SAVEPOINTS,
            HtonCapabilities::SDI,
            HtonCapabilities::SECONDARY_ENGINE,
            HtonCapabilities::CLONE,
            HtonCapabilities::PAGE_TRACKING,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a.bits(), b.bits());
            }
        }
    }
}
