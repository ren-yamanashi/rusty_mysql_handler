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

//! Per-connection transaction state.

use crate::engine::EngineResult;

/// The transaction state for one connection.
///
/// A transactional [`Handlerton`](crate::hton::Handlerton) creates one of these
/// per connection (via [`Handlerton::begin_transaction`]). MySQL stores it in
/// the connection's `ha_data` slot and drives it through `commit` / `rollback`,
/// so a `TxnSession` outlives the per-table handler and accumulates work across
/// every statement of the transaction.
///
/// The `Send` bound is required because a connection may be served by different
/// threads over its lifetime, so the session moves across threads — do not
/// relax it.
///
/// `all` distinguishes the two boundaries MySQL signals on the same callback:
/// `true` is a real transaction commit/rollback (the connection is in
/// autocommit, or `COMMIT` / `ROLLBACK` ran); `false` is the end of a single
/// statement within a larger transaction.
///
/// [`Handlerton::begin_transaction`]: crate::hton::Handlerton::begin_transaction
pub trait TxnSession: Send {
    /// Commit the work accumulated so far.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the commit
    /// fails; MySQL surfaces it to the client and the statement / transaction
    /// is reported as failed.
    fn commit(&mut self, all: bool) -> EngineResult;

    /// Discard the work accumulated so far.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the rollback
    /// fails; this is reported to MySQL but the transaction is still considered
    /// rolled back.
    fn rollback(&mut self, all: bool) -> EngineResult;

    /// Stage a row written into `table` as part of this transaction.
    ///
    /// A transactional engine receives each row write here (rather than on the
    /// per-table handler) so the change is buffered until `commit` makes it
    /// visible or `rollback` discards it. `row` is the MySQL row image
    /// (`record[0]`). The default ignores the write; an engine that stores data
    /// overrides this to buffer it.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the row cannot
    /// be staged (e.g. out of memory).
    fn write_row(&mut self, table: &str, row: &[u8]) -> EngineResult {
        let _ = (table, row);
        Ok(())
    }

    /// Stage a row update into `table` as part of this transaction.
    ///
    /// `old` is the row image before the change, `new` after. A
    /// transactional engine receives each update here (rather than on
    /// the per-table handler) so the change is buffered until `commit`
    /// makes it visible or `rollback` discards it. The default ignores
    /// the update; an engine that stores data overrides this to buffer
    /// the (old, new) pair so rollback can restore the pre-image.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the
    /// update cannot be staged.
    fn update_row(&mut self, table: &str, old: &[u8], new: &[u8]) -> EngineResult {
        let _ = (table, old, new);
        Ok(())
    }

    /// Stage a row deletion in `table` as part of this transaction.
    ///
    /// `row` is the image of the row about to be removed. A
    /// transactional engine receives each delete here so the change is
    /// buffered until `commit` makes it visible or `rollback` discards
    /// it. The default ignores the delete; an engine that stores data
    /// overrides this to buffer the pre-image so rollback can restore
    /// the row.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the
    /// delete cannot be staged.
    fn delete_row(&mut self, table: &str, row: &[u8]) -> EngineResult {
        let _ = (table, row);
        Ok(())
    }

    /// Prepare phase: flush the transaction so a following `commit` is durable.
    ///
    /// MySQL drives this whenever the engine takes part in two-phase commit —
    /// most importantly alongside the binary log, which is on by default — so a
    /// transactional engine must handle it. The default reports success, which
    /// is correct for an engine with nothing durable to prepare; override it to
    /// flush real state.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the engine
    /// cannot prepare; MySQL then rolls the transaction back.
    fn prepare(&mut self, all: bool) -> EngineResult {
        let _ = all;
        Ok(())
    }

    /// Establish a savepoint at the transaction's current point. `sv` is the
    /// engine's per-savepoint scratch (`savepoint_offset` bytes, declared by
    /// [`Handlerton::savepoint_offset`](crate::hton::Handlerton::savepoint_offset)):
    /// write whatever the engine needs to identify this savepoint on rollback.
    /// `sv` is only byte-aligned, so write it through `copy_from_slice` /
    /// `to_le_bytes` rather than a typed pointer store. Defaults to no-op success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the savepoint
    /// cannot be recorded.
    fn savepoint_set(&mut self, sv: &mut [u8]) -> EngineResult {
        let _ = sv;
        Ok(())
    }

    /// Roll the transaction back to the savepoint whose scratch is `sv` (as
    /// written by [`savepoint_set`](Self::savepoint_set)), discarding work done
    /// since. Defaults to no-op success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the rollback
    /// to savepoint fails.
    fn savepoint_rollback(&mut self, sv: &[u8]) -> EngineResult {
        let _ = sv;
        Ok(())
    }

    /// Release (forget) the savepoint whose scratch is `sv`, keeping its work
    /// part of the transaction. Defaults to no-op success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the release
    /// fails.
    fn savepoint_release(&mut self, sv: &[u8]) -> EngineResult {
        let _ = sv;
        Ok(())
    }

    /// Whether it is safe to release metadata locks acquired after a savepoint
    /// when rolling back to it. Defaults to `true` (the engine holds no locks
    /// that a savepoint rollback must preserve).
    fn savepoint_rollback_can_release_mdl(&self) -> bool {
        true
    }
}
