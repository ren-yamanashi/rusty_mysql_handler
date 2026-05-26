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

//! One endpoint of a range scan, mirroring MySQL's `key_range`.

use crate::engine::RKeyFunction;

/// One endpoint of a range scan, mirroring the relevant fields of MySQL's
/// `key_range`. The shim resolves the original `key_part_map` to the leading
/// key bytes before crossing the FFI boundary, so [`key`](Self::key) is already
/// length-resolved; the borrow may not be retained past the callback that
/// supplied it.
///
/// A range endpoint is optional at the call site — MySQL passes a null
/// `key_range` for an open-ended bound — so the trait methods receive an
/// `Option<RangeKey<'_>>` and `None` denotes "no bound on this side".
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct RangeKey<'a> {
    key: &'a [u8],
    flag: RKeyFunction,
}

impl<'a> RangeKey<'a> {
    pub(crate) fn new(key: &'a [u8], flag: RKeyFunction) -> Self {
        Self { key, flag }
    }

    /// Leading key bytes that position the scan at this endpoint
    #[must_use]
    pub fn key(&self) -> &[u8] {
        self.key
    }

    /// Search semantics MySQL attached to this endpoint
    #[must_use]
    pub fn flag(&self) -> RKeyFunction {
        self.flag
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_key_and_flag() {
        let key = [1u8, 2, 3];
        let endpoint = RangeKey::new(&key, RKeyFunction::KeyOrNext);
        assert_eq!(endpoint.key(), &key);
        assert_eq!(endpoint.flag(), RKeyFunction::KeyOrNext);
    }

    #[test]
    fn empty_key_is_preserved() {
        let endpoint = RangeKey::new(&[], RKeyFunction::KeyExact);
        assert!(endpoint.key().is_empty());
    }
}
