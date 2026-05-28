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

//! Engine-instance lifecycle across the FFI boundary: the factory registry,
//! the per-handler [`EngineContext`], create/destroy entry points, and the
//! `FfiPtr` pointer helpers. Per-method `rust__handler__*` callbacks live in
//! [`crate::handler`]; both modules share the safety contract documented there.

#![allow(unsafe_code)]

mod context;
#[doc(hidden)]
pub mod hton_registry;
#[doc(hidden)]
pub mod ptr;
#[doc(hidden)]
pub mod registry;

pub use context::EngineContext;
use hton_registry::HandlertonRegistry;
pub(crate) use ptr::FfiPtr;
pub use registry::EngineFactory;
use registry::EngineRegistry;

use crate::hton::Handlerton;
use crate::panic_guard::FfiBoundary;

static REGISTRY: EngineRegistry = EngineRegistry::new();

/// Install `factory` on the process-wide engine registry. Call this once
/// from the plugin's `rust__plugin_init`; later calls are ignored.
pub fn register_engine_factory(factory: EngineFactory) {
    REGISTRY.register(factory);
}

static HANDLERTON: HandlertonRegistry = HandlertonRegistry::new();

/// Install `handlerton` as the process-wide engine-level handlerton.
///
/// Optional and independent of [`register_engine_factory`]: call it once from
/// the plugin's `rust__plugin_init` only when the engine implements
/// engine-level behaviour (transactions, savepoints, ...). Later calls are
/// ignored, and an engine that never calls it keeps the zero-config handlerton
/// defaults.
pub fn register_handlerton(handlerton: Box<dyn Handlerton>) {
    HANDLERTON.register(handlerton);
}

/// The registered engine-level handlerton, or `None` when the engine kept the
/// zero-config defaults. Read by the `rust__hton__*` FFI accessors.
pub(crate) fn handlerton() -> Option<&'static dyn Handlerton> {
    HANDLERTON.get()
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
