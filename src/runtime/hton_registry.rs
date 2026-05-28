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

//! Process-wide engine-level handlerton singleton registry.

use std::sync::OnceLock;

use crate::hton::Handlerton;

/// Holds the optional process-wide engine-level [`Handlerton`]. Mirrors the
/// engine factory registry but stores one singleton rather than a per-table
/// factory: there is a single handlerton per engine for the whole process.
/// Held for the process lifetime, so there is no removal path.
#[non_exhaustive]
pub(crate) struct HandlertonRegistry {
    handlerton: OnceLock<Box<dyn Handlerton>>,
}

impl HandlertonRegistry {
    pub(crate) const fn new() -> Self {
        Self {
            handlerton: OnceLock::new(),
        }
    }

    pub(crate) fn register(&self, handlerton: Box<dyn Handlerton>) {
        match self.handlerton.set(handlerton) {
            Ok(()) => {}
            Err(_) => {
                tracing::debug!("handlerton already registered; ignoring duplicate registration");
            }
        }
    }

    pub(crate) fn get(&self) -> Option<&dyn Handlerton> {
        self.handlerton.get().map(|h| &**h)
    }
}

impl Default for HandlertonRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for HandlertonRegistry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HandlertonRegistry")
            .field("registered", &self.handlerton.get().is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hton::{HtonCapabilities, HtonFlags};

    struct MockHandlerton;

    impl Handlerton for MockHandlerton {
        fn capabilities(&self) -> HtonCapabilities {
            HtonCapabilities::TRANSACTIONS
        }
        fn flags(&self) -> HtonFlags {
            HtonFlags::NONE
        }
    }

    #[test]
    fn get_is_none_before_register() {
        let registry = HandlertonRegistry::new();
        assert!(registry.get().is_none());
    }

    #[test]
    fn register_then_get_yields_handlerton() {
        let registry = HandlertonRegistry::new();
        registry.register(Box::new(MockHandlerton));
        let h = registry
            .get()
            .expect("registered handlerton is retrievable");
        assert!(h.capabilities().contains(HtonCapabilities::TRANSACTIONS));
        assert_eq!(h.flags(), HtonFlags::NONE);
    }

    #[test]
    fn duplicate_register_keeps_first() {
        let registry = HandlertonRegistry::new();
        registry.register(Box::new(MockHandlerton));
        registry.register(Box::new(MockHandlerton));
        assert!(registry.get().is_some());
    }
}
