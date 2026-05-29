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

//! `rust__hton__txn_*` callbacks: the shim owns the per-connection `ha_data`
//! slot (it has the `handlerton`); these create, drive, and free the
//! [`TxnContext`] it stores there. The shim passes the context pointer back on
//! every commit / rollback so Rust never has to touch MySQL's `handlerton`.

#![allow(unsafe_code)]

use super::txn_context::TxnContext;
use crate::engine::EngineResult;
use crate::panic_guard::FfiBoundary;
use crate::runtime;

// Dispatch shared by the extern callbacks: a null context (the engine was
// registered but produced no session) is a no-op success; otherwise drive the
// session. Split out so the null / dispatch contract is unit-tested without the
// raw-pointer unsafe of the extern boundary.
fn commit_ctx(ctx: Option<&mut TxnContext>, all: bool) -> EngineResult {
    match ctx {
        Some(c) => c.session_mut().commit(all),
        None => Ok(()),
    }
}

fn rollback_ctx(ctx: Option<&mut TxnContext>, all: bool) -> EngineResult {
    match ctx {
        Some(c) => c.session_mut().rollback(all),
        None => Ok(()),
    }
}

fn prepare_ctx(ctx: Option<&mut TxnContext>, all: bool) -> EngineResult {
    match ctx {
        Some(c) => c.session_mut().prepare(all),
        None => Ok(()),
    }
}

/// Begin a transaction: allocate the per-connection [`TxnContext`] the shim
/// stores in `ha_data`. Returns null when no handlerton is registered (the
/// shim then does not register the engine in the transaction).
///
/// # Safety
/// Call after `rust__plugin_init`. The returned pointer must be released once
/// via [`rust__hton__txn_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__txn_begin() -> *mut TxnContext {
    FfiBoundary::run_default(std::ptr::null_mut(), || match runtime::handlerton() {
        Some(h) => Box::into_raw(Box::new(TxnContext::new(h.begin_transaction()))),
        None => std::ptr::null_mut(),
    })
}

/// Commit the transaction. `all` is true for a real transaction commit, false
/// for statement end. The context is not freed here; the shim calls
/// [`rust__hton__txn_free`] after the final (`all`) commit.
///
/// # Safety
/// `ctx` is null or a [`TxnContext`] from [`rust__hton__txn_begin`], valid and
/// exclusively borrowed for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__txn_commit(ctx: *mut TxnContext, all: bool) -> i32 {
    // SAFETY: ctx is null or a valid, exclusively-owned TxnContext.
    FfiBoundary::run(|| commit_ctx(unsafe { ctx.as_mut() }, all))
}

/// Roll back the transaction. `all` distinguishes a real rollback from
/// statement end, mirroring [`rust__hton__txn_commit`].
///
/// # Safety
/// `ctx` is null or a [`TxnContext`] from [`rust__hton__txn_begin`], valid and
/// exclusively borrowed for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__txn_rollback(ctx: *mut TxnContext, all: bool) -> i32 {
    // SAFETY: ctx is null or a valid, exclusively-owned TxnContext.
    FfiBoundary::run(|| rollback_ctx(unsafe { ctx.as_mut() }, all))
}

/// Prepare phase of two-phase commit.
///
/// # Safety
/// `ctx` is null or a [`TxnContext`] from [`rust__hton__txn_begin`], valid and
/// exclusively borrowed for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__txn_prepare(ctx: *mut TxnContext, all: bool) -> i32 {
    // SAFETY: ctx is null or a valid, exclusively-owned TxnContext.
    FfiBoundary::run(|| prepare_ctx(unsafe { ctx.as_mut() }, all))
}

/// Free a [`TxnContext`] returned by [`rust__hton__txn_begin`].
///
/// # Safety
/// `ctx` must come from [`rust__hton__txn_begin`] and not be freed twice; null
/// is ignored.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__txn_free(ctx: *mut TxnContext) {
    FfiBoundary::run_void(|| {
        if !ctx.is_null() {
            // SAFETY: ctx originates from txn_begin's Box::into_raw, freed once.
            drop(unsafe { Box::from_raw(ctx) });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hton::TxnSession;

    struct RecordingTxn {
        committed: bool,
        rolled_back: bool,
    }

    impl TxnSession for RecordingTxn {
        fn commit(&mut self, _all: bool) -> EngineResult {
            self.committed = true;
            Ok(())
        }
        fn rollback(&mut self, _all: bool) -> EngineResult {
            self.rolled_back = true;
            Ok(())
        }
    }

    #[test]
    fn null_ctx_is_a_noop_success() {
        assert_eq!(commit_ctx(None, true), Ok(()));
        assert_eq!(rollback_ctx(None, true), Ok(()));
        assert_eq!(prepare_ctx(None, true), Ok(()));
    }

    #[test]
    fn some_ctx_dispatches_to_the_session() {
        let mut ctx = TxnContext::new(Box::new(RecordingTxn {
            committed: false,
            rolled_back: false,
        }));
        assert_eq!(commit_ctx(Some(&mut ctx), true), Ok(()));
        assert_eq!(rollback_ctx(Some(&mut ctx), false), Ok(()));
    }
}
