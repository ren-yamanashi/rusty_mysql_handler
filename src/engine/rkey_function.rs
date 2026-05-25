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

//! Index-lookup search semantics, mirroring MySQL's `ha_rkey_function`.

/// Search semantics for an index lookup, mirroring MySQL's `ha_rkey_function`.
///
/// Passed to [`StorageEngine::index_read_map`] to describe how the supplied key
/// should be matched: an exact hit, the nearest neighbour in a direction, a
/// prefix, or one of the spatial (minimum-bounding-rectangle) relations.
///
/// [`StorageEngine::index_read_map`]: crate::engine::StorageEngine::index_read_map
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RKeyFunction {
    /// Find the first record with exactly this key, else error
    KeyExact,
    /// This record or the next one
    KeyOrNext,
    /// This record or the previous one
    KeyOrPrev,
    /// First record after this key
    AfterKey,
    /// First record before this key
    BeforeKey,
    /// First record sharing this key prefix
    Prefix,
    /// Last record sharing this key prefix
    PrefixLast,
    /// Last record with this prefix, or the previous one
    PrefixLastOrPrev,
    /// Minimum bounding rectangle contains the key
    MbrContain,
    /// Minimum bounding rectangle intersects the key
    MbrIntersect,
    /// Minimum bounding rectangle is within the key
    MbrWithin,
    /// Minimum bounding rectangle is disjoint from the key
    MbrDisjoint,
    /// Minimum bounding rectangle equals the key
    MbrEqual,
    /// Nearest-neighbour spatial search
    NearestNeighbor,
    /// Unrecognised value; MySQL's `HA_READ_INVALID` or an out-of-range code
    Invalid,
}

impl RKeyFunction {
    /// Map the raw `ha_rkey_function` integer supplied at the FFI boundary to a
    /// variant. Any unknown code (including `HA_READ_INVALID == -1`) becomes
    /// [`RKeyFunction::Invalid`] so the engine never observes an undefined value.
    pub(crate) fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::KeyExact,
            1 => Self::KeyOrNext,
            2 => Self::KeyOrPrev,
            3 => Self::AfterKey,
            4 => Self::BeforeKey,
            5 => Self::Prefix,
            6 => Self::PrefixLast,
            7 => Self::PrefixLastOrPrev,
            8 => Self::MbrContain,
            9 => Self::MbrIntersect,
            10 => Self::MbrWithin,
            11 => Self::MbrDisjoint,
            12 => Self::MbrEqual,
            13 => Self::NearestNeighbor,
            _ => Self::Invalid,
        }
    }
}
