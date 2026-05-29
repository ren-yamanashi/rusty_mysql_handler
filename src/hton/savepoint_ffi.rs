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

//! `rust__hton__savepoint_*` callbacks. The shim owns the connection's ha_data
//! (the `TxnContext`) and MySQL's per-savepoint scratch (`sv`); these forward
//! both to the per-connection [`TxnSession`](crate::hton::TxnSession). Dispatch
//! is split into safe helpers so the null / dispatch contract is tested without
//! the raw-pointer unsafe of the extern boundary.

#![allow(unsafe_code)]

use super::txn_context::TxnContext;
use crate::engine::EngineResult;
use crate::panic_guard::FfiBoundary;
use crate::runtime::FfiPtr;

fn set_ctx(ctx: Option<&mut TxnContext>, sv: &mut [u8]) -> EngineResult {
    match ctx {
        Some(c) => c.session_mut().savepoint_set(sv),
        None => Ok(()),
    }
}

fn rollback_ctx(ctx: Option<&mut TxnContext>, sv: &[u8]) -> EngineResult {
    match ctx {
        Some(c) => c.session_mut().savepoint_rollback(sv),
        None => Ok(()),
    }
}

fn release_ctx(ctx: Option<&mut TxnContext>, sv: &[u8]) -> EngineResult {
    match ctx {
        Some(c) => c.session_mut().savepoint_release(sv),
        None => Ok(()),
    }
}

fn can_release_mdl_ctx(ctx: Option<&mut TxnContext>) -> bool {
    match ctx {
        Some(c) => c.session_mut().savepoint_rollback_can_release_mdl(),
        None => true,
    }
}

/// Establish a savepoint, writing the engine's per-savepoint state into `sv`.
///
/// # Safety
/// `ctx` is null or a [`TxnContext`] from `rust__hton__txn_begin`; `sv` covers
/// `sv_len` writable bytes for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__savepoint_set(
    ctx: *mut TxnContext,
    sv: *mut u8,
    sv_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: sv covers sv_len writable bytes for this call.
        let sv = unsafe { FfiPtr::slice_mut(sv, sv_len) };
        // SAFETY: ctx is null or a valid, exclusively-owned TxnContext.
        set_ctx(unsafe { ctx.as_mut() }, sv)
    })
}

/// Roll back to the savepoint whose state is in `sv`.
///
/// # Safety
/// `ctx` is null or a valid [`TxnContext`]; `sv` covers `sv_len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__savepoint_rollback(
    ctx: *mut TxnContext,
    sv: *const u8,
    sv_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: sv covers sv_len readable bytes for this call.
        let sv = unsafe { FfiPtr::slice_const(sv, sv_len) };
        // SAFETY: ctx is null or a valid, exclusively-owned TxnContext.
        rollback_ctx(unsafe { ctx.as_mut() }, sv)
    })
}

/// Release the savepoint whose state is in `sv`.
///
/// # Safety
/// `ctx` is null or a valid [`TxnContext`]; `sv` covers `sv_len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__savepoint_release(
    ctx: *mut TxnContext,
    sv: *const u8,
    sv_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: sv covers sv_len readable bytes for this call.
        let sv = unsafe { FfiPtr::slice_const(sv, sv_len) };
        // SAFETY: ctx is null or a valid, exclusively-owned TxnContext.
        release_ctx(unsafe { ctx.as_mut() }, sv)
    })
}

/// Whether rolling back to a savepoint may release later-acquired MDL.
///
/// # Safety
/// `ctx` is null or a valid [`TxnContext`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__savepoint_can_release_mdl(ctx: *mut TxnContext) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: ctx is null or a valid, exclusively-owned TxnContext.
        can_release_mdl_ctx(unsafe { ctx.as_mut() })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hton::TxnSession;

    #[derive(Default)]
    struct RecordingTxn {
        set: u32,
        rolled_back: u32,
    }

    impl TxnSession for RecordingTxn {
        fn commit(&mut self, _all: bool) -> EngineResult {
            Ok(())
        }
        fn rollback(&mut self, _all: bool) -> EngineResult {
            Ok(())
        }
        fn savepoint_set(&mut self, _sv: &mut [u8]) -> EngineResult {
            self.set += 1;
            Ok(())
        }
        fn savepoint_rollback(&mut self, _sv: &[u8]) -> EngineResult {
            self.rolled_back += 1;
            Ok(())
        }
        fn savepoint_rollback_can_release_mdl(&self) -> bool {
            false
        }
    }

    #[test]
    fn null_ctx_defaults() {
        let mut sv = [0u8; 8];
        assert_eq!(set_ctx(None, &mut sv), Ok(()));
        assert_eq!(rollback_ctx(None, &sv), Ok(()));
        assert_eq!(release_ctx(None, &sv), Ok(()));
        assert!(can_release_mdl_ctx(None));
    }

    #[test]
    fn some_ctx_dispatches() {
        let mut ctx = TxnContext::new(Box::new(RecordingTxn::default()));
        let mut sv = [0u8; 8];
        assert_eq!(set_ctx(Some(&mut ctx), &mut sv), Ok(()));
        assert_eq!(rollback_ctx(Some(&mut ctx), &sv), Ok(()));
        assert!(!can_release_mdl_ctx(Some(&mut ctx)));
    }
}
