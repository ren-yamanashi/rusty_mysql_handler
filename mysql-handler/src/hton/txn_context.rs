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

//! Per-connection transaction state owned across the C++ FFI boundary.

use std::fmt;

use super::transaction::TxnSession;

/// Per-connection transaction state owned through `Box::into_raw`. MySQL keeps
/// a single `void*` per connection (`thd->ha_data`); `Box<dyn TxnSession>` is a
/// fat pointer, so it is boxed once more here to give the shim a thin
/// `*mut TxnContext` to store there.
#[non_exhaustive]
pub struct TxnContext {
    session: Box<dyn TxnSession>,
}

impl TxnContext {
    pub(crate) fn new(session: Box<dyn TxnSession>) -> Self {
        Self { session }
    }

    pub(crate) fn session_mut(&mut self) -> &mut dyn TxnSession {
        &mut *self.session
    }
}

impl fmt::Debug for TxnContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TxnContext").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{EngineError, EngineResult};

    struct CountingTxn {
        commits: u32,
        rollbacks: u32,
    }

    impl TxnSession for CountingTxn {
        fn commit(&mut self, _all: bool) -> EngineResult {
            self.commits += 1;
            Ok(())
        }
        fn rollback(&mut self, _all: bool) -> EngineResult {
            self.rollbacks += 1;
            Err(EngineError::Internal)
        }
    }

    #[test]
    fn session_mut_dispatches_to_the_boxed_session() {
        let mut ctx = TxnContext::new(Box::new(CountingTxn {
            commits: 0,
            rollbacks: 0,
        }));
        assert_eq!(ctx.session_mut().commit(true), Ok(()));
        assert_eq!(ctx.session_mut().commit(false), Ok(()));
        assert_eq!(ctx.session_mut().rollback(true), Err(EngineError::Internal));
    }

    #[test]
    fn prepare_defaults_to_ok() {
        let mut ctx = TxnContext::new(Box::new(CountingTxn {
            commits: 0,
            rollbacks: 0,
        }));
        assert_eq!(ctx.session_mut().prepare(true), Ok(()));
    }
}
