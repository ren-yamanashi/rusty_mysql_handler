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

//! `ha_panic_function` from `include/my_base.h`.

/// The reason MySQL is invoking the engine-level `panic` callback.
///
/// Mirrors `enum ha_panic_function` in `mysql-server/include/my_base.h`. Used
/// only as the argument to [`Handlerton::panic`](crate::hton::Handlerton::panic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum HaPanicFunction {
    /// `HA_PANIC_CLOSE`: close all databases before shutdown.
    Close,
    /// `HA_PANIC_WRITE`: unlock and write status.
    Write,
    /// `HA_PANIC_READ`: lock and read keyinfo.
    Read,
}

impl HaPanicFunction {
    /// Decode the C `enum ha_panic_function` value. Unknown values map to
    /// [`HaPanicFunction::Close`] so the engine still observes a defined variant
    /// rather than a panic on a forward-compatible MySQL change.
    #[must_use]
    pub const fn from_raw(value: u32) -> Self {
        match value {
            1 => Self::Write,
            2 => Self::Read,
            _ => Self::Close,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_raw_maps_each_known_value() {
        assert_eq!(HaPanicFunction::from_raw(0), HaPanicFunction::Close);
        assert_eq!(HaPanicFunction::from_raw(1), HaPanicFunction::Write);
        assert_eq!(HaPanicFunction::from_raw(2), HaPanicFunction::Read);
    }

    #[test]
    fn from_raw_unknown_value_falls_back_to_close() {
        assert_eq!(HaPanicFunction::from_raw(7), HaPanicFunction::Close);
        assert_eq!(HaPanicFunction::from_raw(u32::MAX), HaPanicFunction::Close);
    }
}
