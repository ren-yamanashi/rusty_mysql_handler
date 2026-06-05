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

//! `capabilities = [...]` discriminants accepted by `#[plugin]`.

use syn::Ident;

/// Identifiers accepted inside `capabilities = [...]`. Each one maps to a
/// sub-trait the engine has opted into; the `#[plugin]` macro emits an
/// `as_*` override on the generated `EngineCapabilities` impl per entry.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Capability {
    Indexed,
    Transactional,
    BulkLoad,
    Secondary,
}

impl Capability {
    pub(crate) fn from_ident(ident: &Ident) -> syn::Result<Self> {
        match ident.to_string().as_str() {
            "Indexed" => Ok(Self::Indexed),
            "Transactional" => Ok(Self::Transactional),
            "BulkLoad" => Ok(Self::BulkLoad),
            "Secondary" => Ok(Self::Secondary),
            other => Err(syn::Error::new(
                ident.span(),
                format!(
                    "unknown capability `{other}` (expected one of: Indexed, Transactional, BulkLoad, Secondary)"
                ),
            )),
        }
    }
}
