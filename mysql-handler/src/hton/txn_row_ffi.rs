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

//! `rust__hton__txn_{write,update,delete}_row` callbacks: the shim
//! routes per-row work through these whenever a transactional
//! handlerton's [`TxnContext`] is attached to the connection. The
//! lifecycle (begin / commit / rollback / prepare / free) lives in
//! [`super::txn_ffi`].

#![allow(unsafe_code)]

use super::txn_context::TxnContext;
use crate::engine::EngineResult;
use crate::panic_guard::FfiBoundary;
use crate::runtime::FfiPtr;

fn write_row_ctx(ctx: Option<&mut TxnContext>, table: &str, row: &[u8]) -> EngineResult {
    match ctx {
        Some(c) => c.session_mut().write_row(table, row),
        None => Ok(()),
    }
}

fn update_row_ctx(
    ctx: Option<&mut TxnContext>,
    table: &str,
    old: &[u8],
    new: &[u8],
) -> EngineResult {
    match ctx {
        Some(c) => c.session_mut().update_row(table, old, new),
        None => Ok(()),
    }
}

fn delete_row_ctx(ctx: Option<&mut TxnContext>, table: &str, row: &[u8]) -> EngineResult {
    match ctx {
        Some(c) => c.session_mut().delete_row(table, row),
        None => Ok(()),
    }
}

/// Stage a row write into the transaction. `table` is the row's table name and
/// `row` its MySQL row image; both are borrowed only for this call.
///
/// # Safety
/// `ctx` is null or a [`TxnContext`] from `rust__hton__txn_begin`; `table`
/// covers `table_len` readable bytes and `row` covers `row_len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__txn_write_row(
    ctx: *mut TxnContext,
    table: *const u8,
    table_len: usize,
    row: *const u8,
    row_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: table is non-null and covers table_len readable bytes here.
        let table = match unsafe { FfiPtr::bytes_to_str(table, table_len) } {
            Ok(t) => t,
            Err(e) => return Err(e),
        };
        // SAFETY: row is non-null and covers row_len readable bytes here.
        let row = unsafe { FfiPtr::slice_const(row, row_len) };
        // SAFETY: ctx is null or a valid, exclusively-owned TxnContext.
        write_row_ctx(unsafe { ctx.as_mut() }, table, row)
    })
}

/// Stage a row update into the transaction. `old` is the pre-image and
/// `new` the post-image; both are borrowed only for this call.
///
/// # Safety
/// `ctx` is null or a [`TxnContext`] from `rust__hton__txn_begin`;
/// `table` covers `table_len` readable bytes, and `old` / `new` each
/// cover their respective length parameters.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__txn_update_row(
    ctx: *mut TxnContext,
    table: *const u8,
    table_len: usize,
    old: *const u8,
    old_len: usize,
    new: *const u8,
    new_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: table covers table_len readable bytes for the call.
        let table = match unsafe { FfiPtr::bytes_to_str(table, table_len) } {
            Ok(t) => t,
            Err(e) => return Err(e),
        };
        // SAFETY: old covers old_len readable bytes for the call.
        let old = unsafe { FfiPtr::slice_const(old, old_len) };
        // SAFETY: new covers new_len readable bytes for the call.
        let new = unsafe { FfiPtr::slice_const(new, new_len) };
        // SAFETY: ctx is null or a valid, exclusively-owned TxnContext.
        update_row_ctx(unsafe { ctx.as_mut() }, table, old, new)
    })
}

/// Stage a row deletion in the transaction. `row` is the pre-image of
/// the row about to be removed.
///
/// # Safety
/// `ctx` is null or a [`TxnContext`] from `rust__hton__txn_begin`;
/// `table` covers `table_len` readable bytes and `row` covers `row_len`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__txn_delete_row(
    ctx: *mut TxnContext,
    table: *const u8,
    table_len: usize,
    row: *const u8,
    row_len: usize,
) -> i32 {
    FfiBoundary::run(|| {
        // SAFETY: table covers table_len readable bytes for the call.
        let table = match unsafe { FfiPtr::bytes_to_str(table, table_len) } {
            Ok(t) => t,
            Err(e) => return Err(e),
        };
        // SAFETY: row covers row_len readable bytes for the call.
        let row = unsafe { FfiPtr::slice_const(row, row_len) };
        // SAFETY: ctx is null or a valid, exclusively-owned TxnContext.
        delete_row_ctx(unsafe { ctx.as_mut() }, table, row)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hton::TxnSession;

    #[derive(Default)]
    struct RecordingTxn {
        writes: u32,
        updates: u32,
        deletes: u32,
    }

    impl TxnSession for RecordingTxn {
        fn commit(&mut self, _all: bool) -> EngineResult {
            Ok(())
        }
        fn rollback(&mut self, _all: bool) -> EngineResult {
            Ok(())
        }
        fn write_row(&mut self, _table: &str, _row: &[u8]) -> EngineResult {
            self.writes += 1;
            Ok(())
        }
        fn update_row(&mut self, _table: &str, _old: &[u8], _new: &[u8]) -> EngineResult {
            self.updates += 1;
            Ok(())
        }
        fn delete_row(&mut self, _table: &str, _row: &[u8]) -> EngineResult {
            self.deletes += 1;
            Ok(())
        }
    }

    #[test]
    fn null_ctx_row_staging_is_a_noop_success() {
        assert_eq!(write_row_ctx(None, "t", b"row"), Ok(()));
        assert_eq!(update_row_ctx(None, "t", b"old", b"new"), Ok(()));
        assert_eq!(delete_row_ctx(None, "t", b"row"), Ok(()));
    }

    #[test]
    fn some_ctx_row_staging_dispatches_each_variant() {
        let mut ctx = TxnContext::new(Box::new(RecordingTxn::default()));
        assert_eq!(write_row_ctx(Some(&mut ctx), "t", b"row"), Ok(()));
        assert_eq!(update_row_ctx(Some(&mut ctx), "t", b"old", b"new"), Ok(()));
        assert_eq!(delete_row_ctx(Some(&mut ctx), "t", b"row"), Ok(()));
    }
}
