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

//! Per-index snapshot taken from `dd::Index`.

use mysql_handler::dd::{IndexElementOrder, IndexType};
use mysql_handler::sys::DdIndex;

use crate::store::KeyPartMeta;

/// Per-index snapshot taken from `dd::Index`. Only the fields the
/// reference engine consumes today are stored; `dd::Index::name`,
/// `engine_attribute`, and friends are intentionally left off until a
/// downstream consumer needs them.
#[derive(Debug, Clone)]
pub struct IndexMeta {
    /// Index kind.
    pub(crate) index_type: IndexType,
    /// Key parts in declared order.
    pub(crate) parts: Vec<KeyPartMeta>,
}

impl IndexMeta {
    /// Snapshot the fields of `index` (and each of its key parts) into an
    /// owned [`IndexMeta`].
    #[must_use]
    pub fn from_dd_index(index: &DdIndex) -> Self {
        let mut parts = Vec::with_capacity(index.element_count());
        for i in 0..index.element_count() {
            if let Some(e) = index.element_at(i) {
                parts.push(KeyPartMeta::from_dd_index_element(e));
            }
        }
        Self {
            index_type: index.index_type(),
            parts,
        }
    }

    /// `true` when the index is a single key part declared `Ascending`
    /// (the only shape whose natural [`crate::store::Key`] order matches
    /// MySQL's expected `HA_READ_ORDER` row sequence).
    #[must_use]
    pub fn is_single_column_ascending(&self) -> bool {
        match self.parts.as_slice() {
            [only] => matches!(
                only.order,
                IndexElementOrder::Ascending | IndexElementOrder::Undefined
            ),
            _ => false,
        }
    }
}
