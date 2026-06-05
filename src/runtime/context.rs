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

//! Per-handler Rust-side state owned across the C++ FFI boundary.

use std::fmt;

use crate::engine::EngineCapabilities;

/// Per-handler Rust-side state owned through `Box::into_raw`. The C++
/// `RustHandlerBase` keeps a `void*` to one of these.
#[non_exhaustive]
pub struct EngineContext {
    engine: Box<dyn EngineCapabilities>,
}

impl EngineContext {
    pub(crate) fn new(engine: Box<dyn EngineCapabilities>) -> Self {
        Self { engine }
    }

    pub(crate) fn engine_mut(&mut self) -> &mut dyn EngineCapabilities {
        &mut *self.engine
    }
}

impl fmt::Debug for EngineContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EngineContext").finish_non_exhaustive()
    }
}
