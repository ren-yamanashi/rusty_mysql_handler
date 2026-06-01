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

//! Per-key-part snapshot taken from `dd::Index_element`.

use mysql_handler::dd::IndexElementOrder;
use mysql_handler::sys::DdIndexElement;

/// Per-key-part snapshot taken from `dd::Index_element`. Only the fields
/// the reference engine consumes today are stored; the prefix length is
/// intentionally left off until a downstream consumer needs it.
#[derive(Debug, Clone)]
pub struct KeyPartMeta {
    /// 1-based ordinal position of the underlying column in the table.
    pub(crate) column_ordinal: u32,
    /// Declared sort order. The engine only advertises `HA_READ_ORDER`
    /// for indexes whose every key part is `Ascending`, because the
    /// `Key` type's `Ord` produces ASC order.
    pub(crate) order: IndexElementOrder,
}

impl KeyPartMeta {
    /// Snapshot the fields of `elt` into an owned [`KeyPartMeta`].
    #[must_use]
    pub fn from_dd_index_element(elt: &DdIndexElement) -> Self {
        Self {
            column_ordinal: elt.column_ordinal(),
            order: elt.order(),
        }
    }
}
