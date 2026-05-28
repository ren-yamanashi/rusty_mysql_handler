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

//! Process-wide engine factory registry.

use std::sync::OnceLock;

use super::context::EngineContext;
use crate::engine::StorageEngine;

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

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use super::*;
    use crate::engine::{EngineError, EngineResult};

    struct MockEngine;

    impl StorageEngine for MockEngine {
        fn table_type(&self) -> &'static CStr {
            c"MOCK"
        }
        fn table_flags(&self) -> u64 {
            0
        }
        fn index_flags(&self, _idx: u32, _part: u32, _all_parts: bool) -> u32 {
            0
        }
        fn create(&mut self, _name: &str) -> EngineResult {
            Ok(())
        }
        fn open(&mut self, _name: &str, _mode: i32) -> EngineResult {
            Ok(())
        }
        fn close(&mut self) -> EngineResult {
            Ok(())
        }
        fn rnd_init(&mut self, _scan: bool) -> EngineResult {
            Ok(())
        }
        fn rnd_next(&mut self, _buf: &mut [u8]) -> EngineResult {
            Err(EngineError::EndOfFile)
        }
        fn rnd_pos(&mut self, _buf: &mut [u8], _pos: &[u8]) -> EngineResult {
            Err(EngineError::WrongCommand)
        }
        fn position(&mut self, _record: &[u8], _ref_out: &mut [u8]) {}
        fn info(&mut self, _flag: u32) -> EngineResult {
            Ok(())
        }
    }

    #[test]
    fn create_context_is_none_before_register() {
        let registry = EngineRegistry::new();
        assert!(registry.create_context().is_none());
    }

    #[test]
    fn register_then_create_yields_context() {
        let registry = EngineRegistry::new();
        registry.register(|| Box::new(MockEngine));
        assert!(registry.create_context().is_some());
    }

    #[test]
    fn duplicate_register_is_ignored() {
        let registry = EngineRegistry::new();
        registry.register(|| Box::new(MockEngine));
        registry.register(|| Box::new(MockEngine));
        assert!(registry.create_context().is_some());
    }
}
