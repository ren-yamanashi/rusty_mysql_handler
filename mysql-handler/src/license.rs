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

//! License tag for the plugin manifest.

use core::ffi::c_int;

/// MySQL plugin license tag. The discriminant matches the
/// `PLUGIN_LICENSE_*` constants in `include/mysql/plugin.h`, so the
/// value can be embedded directly into the manifest's `license` field
/// at compile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum License {
    /// Closed-source license. Maps to `PLUGIN_LICENSE_PROPRIETARY` (0).
    Proprietary,
    /// GNU General Public License. Maps to `PLUGIN_LICENSE_GPL` (1).
    Gpl,
    /// BSD-style license. Maps to `PLUGIN_LICENSE_BSD` (2).
    Bsd,
}

impl License {
    /// Returns the wire-level `c_int` discriminant the MySQL plugin
    /// manifest expects in its `license` field. `const` so the value
    /// can be embedded in a `static` initialiser.
    #[must_use]
    pub const fn code(self) -> c_int {
        match self {
            License::Proprietary => 0,
            License::Gpl => 1,
            License::Bsd => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::License;

    #[test]
    fn code_matches_mysql_plugin_h_constants() {
        assert_eq!(License::Proprietary.code(), 0);
        assert_eq!(License::Gpl.code(), 1);
        assert_eq!(License::Bsd.code(), 2);
    }
}
