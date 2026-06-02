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

//! Shared per-callback bench fixtures: RAII guard around the raw
//! `EngineContext` pointer so the destructor runs even if a `b.iter`
//! body unwinds.

#![allow(unreachable_pub)]

use mysql_handler::engine::StorageEngine;
use mysql_handler::runtime::{
    EngineContext, register_engine_factory, rust__create_engine, rust__destroy_engine,
};

use super::shared::noop_engine::NoopEngine;

/// RAII wrapper around the raw `*mut EngineContext` returned by
/// `rust__create_engine`. Drops via `rust__destroy_engine` so the
/// engine box is freed even when a `b.iter` body unwinds.
pub struct CtxGuard(*mut EngineContext);

impl CtxGuard {
    /// Register the `NoopEngine` factory (idempotent — later calls are
    /// ignored by the runtime registry) and allocate an
    /// `EngineContext`. The benchmark binary calls this once per
    /// `bench_*` function; criterion runs the closures sequentially
    /// so no two `b.iter` calls share the pointer concurrently.
    pub fn new() -> Self {
        register_engine_factory(|| Box::new(NoopEngine::new()));
        // SAFETY: `rust__create_engine` is safe after
        // `register_engine_factory`; the returned pointer is null only
        // when no factory is registered or the factory panics, neither
        // of which applies here.
        let ctx = unsafe { rust__create_engine() };
        assert!(!ctx.is_null(), "rust__create_engine returned null");
        Self(ctx)
    }

    /// Raw pointer for passing to the `rust__handler__*` callbacks.
    pub fn as_ptr(&self) -> *mut EngineContext {
        self.0
    }
}

impl Drop for CtxGuard {
    fn drop(&mut self) {
        // SAFETY: `self.0` came from `rust__create_engine` in `new`
        // and is not used after this `Drop` runs.
        unsafe { rust__destroy_engine(self.0) };
    }
}

/// Standalone `Box<dyn StorageEngine>` for the `native` arm. Calls
/// the trait method directly with no FFI layer.
pub fn native_engine() -> Box<dyn StorageEngine> {
    Box::new(NoopEngine::new())
}
