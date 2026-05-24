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

//! FFI lifecycle and shared helpers. Per-method `rust__handler__*` callbacks
//! live in [`crate::ffi_handler`]; both modules share the safety contract
//! documented there.

#![allow(unsafe_code)]

use std::fmt;
use std::slice;
use std::sync::OnceLock;

use crate::engine::{EngineError, EngineResult, StorageEngine};
use crate::panic_guard::FfiBoundary;

/// Per-handler Rust-side state owned through `Box::into_raw`. The C++
/// `RustHandlerBase` keeps a `void*` to one of these.
#[non_exhaustive]
pub struct EngineContext {
    engine: Box<dyn StorageEngine>,
}

impl EngineContext {
    pub(crate) fn new(engine: Box<dyn StorageEngine>) -> Self {
        Self { engine }
    }

    pub(crate) fn engine_mut(&mut self) -> &mut dyn StorageEngine {
        &mut *self.engine
    }
}

impl fmt::Debug for EngineContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EngineContext").finish_non_exhaustive()
    }
}

/// Factory closure that produces a fresh engine instance per opened table
pub type EngineFactory = fn() -> Box<dyn StorageEngine>;

/// Process-wide singleton holding the engine factory. The plugin's
/// `rust__plugin_init` registers the factory once at startup;
/// `rust__create_engine` reads back through the same registry on every
/// handler instantiation.
#[derive(Debug)]
#[non_exhaustive]
pub(crate) struct EngineRegistry {
    factory: OnceLock<EngineFactory>,
}

impl EngineRegistry {
    pub(crate) const fn new() -> Self {
        Self {
            factory: OnceLock::new(),
        }
    }

    pub(crate) fn register(&self, factory: EngineFactory) {
        match self.factory.set(factory) {
            Ok(()) => {}
            Err(_) => {
                tracing::debug!(
                    "engine factory already registered; ignoring duplicate registration"
                );
            }
        }
    }

    pub(crate) fn create_context(&self) -> Option<EngineContext> {
        let factory = self.factory.get().copied()?;
        Some(EngineContext::new(factory()))
    }
}

impl Default for EngineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

static REGISTRY: EngineRegistry = EngineRegistry::new();

/// Install `factory` on the process-wide engine registry. Call this once
/// from the plugin's `rust__plugin_init`; later calls are ignored.
pub fn register_engine_factory(factory: EngineFactory) {
    REGISTRY.register(factory);
}

/// Raw-pointer helpers that turn shim-supplied pointers into bounded references
#[derive(Debug)]
#[non_exhaustive]
pub(crate) struct FfiPtr;

impl FfiPtr {
    /// Decode `len` bytes at `p` as a UTF-8 `&str`; length is caller-measured
    /// so this side performs no `strlen`-style scan.
    ///
    /// # Safety
    /// `p` must be non-null, aligned, and readable for `len` bytes for the
    /// returned reference's lifetime.
    pub(crate) unsafe fn bytes_to_str<'a>(p: *const u8, len: usize) -> EngineResult<&'a str> {
        // SAFETY: caller guarantees `p` covers `len` readable bytes;
        // `from_raw_parts` requires non-null even when `len == 0`.
        let bytes = unsafe { slice::from_raw_parts(p, len) };
        match core::str::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(_) => Err(EngineError::InvalidName),
        }
    }

    /// View `len` writable bytes at `p` as `&mut [u8]`
    ///
    /// # Safety
    /// `p` must be non-null, aligned, and writable for `len` bytes for the
    /// returned reference's lifetime.
    pub(crate) unsafe fn slice_mut<'a>(p: *mut u8, len: usize) -> &'a mut [u8] {
        // SAFETY: caller guarantees `p` covers `len` writable bytes;
        // `from_raw_parts_mut` requires non-null even when `len == 0`.
        unsafe { slice::from_raw_parts_mut(p, len) }
    }

    /// View `len` readable bytes at `p` as `&[u8]`
    ///
    /// # Safety
    /// `p` must be non-null, aligned, and readable for `len` bytes for the
    /// returned reference's lifetime.
    pub(crate) unsafe fn slice_const<'a>(p: *const u8, len: usize) -> &'a [u8] {
        // SAFETY: caller guarantees `p` covers `len` readable bytes;
        // `from_raw_parts` requires non-null even when `len == 0`.
        unsafe { slice::from_raw_parts(p, len) }
    }
}

/// Allocate an `EngineContext`; null if no factory or the factory panics
///
/// # Safety
/// Safe after `rust__plugin_init`; release via `rust__destroy_engine` once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__create_engine() -> *mut EngineContext {
    FfiBoundary::run_default(std::ptr::null_mut(), || match REGISTRY.create_context() {
        Some(ctx) => Box::into_raw(Box::new(ctx)),
        None => std::ptr::null_mut(),
    })
}

/// Drop a context returned by `rust__create_engine`
///
/// # Safety
/// `ctx` must come from `rust__create_engine` and not be released twice; null
/// is ignored.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__destroy_engine(ctx: *mut EngineContext) {
    FfiBoundary::run_void(|| {
        if !ctx.is_null() {
            // SAFETY: pointer originates from `Box::into_raw` and is dropped once.
            drop(unsafe { Box::from_raw(ctx) });
        }
    });
}
