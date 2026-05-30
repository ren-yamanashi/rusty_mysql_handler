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

//! Engine-level handlerton interface.
//!
//! Where [`StorageEngine`](crate::engine::StorageEngine) is per-table, the
//! handlerton is a per-engine singleton: one instance serves every connection.
//! An engine opts into engine-level behaviour by implementing [`Handlerton`]
//! and registering it with
//! [`register_handlerton`](crate::runtime::register_handlerton). Engines that
//! need nothing beyond table handling skip registration and keep the
//! zero-config defaults.

#[doc(hidden)]
pub mod binlog;
mod binlog_kind;
mod capabilities;
#[doc(hidden)]
pub mod discovery;
#[doc(hidden)]
pub mod ffi;
mod flags;
#[doc(hidden)]
pub mod lifecycle;
mod notification_kind;
#[doc(hidden)]
pub mod notifications;
mod panic_function;
#[doc(hidden)]
pub mod savepoint_ffi;
mod stat_print_sink;
mod stat_type;
#[doc(hidden)]
pub mod status;
mod transaction;
#[doc(hidden)]
pub mod txn_context;
#[doc(hidden)]
pub mod txn_ffi;
#[doc(hidden)]
pub mod xa;

pub use binlog_kind::{EnumBinlogCommand, EnumBinlogFunc};
pub use capabilities::HtonCapabilities;
pub use flags::HtonFlags;
pub use notification_kind::{HaNotificationType, SelectExecutedIn};
pub use panic_function::HaPanicFunction;
pub use stat_print_sink::StatPrintSink;
pub use stat_type::HaStatType;
pub use transaction::TxnSession;

use crate::engine::EngineResult;
use crate::sys;

/// The engine-level handlerton interface: the capabilities and `handlerton`
/// struct fields that apply to the engine as a whole rather than to a single
/// table.
///
/// Every method has a default, so an engine implements only what it needs and
/// an empty `impl Handlerton for MyEngine {}` is a valid handler-only
/// handlerton. The singleton is shared across all connection threads, hence
/// the `Send + Sync` bound — do not relax it.
///
/// # Examples
///
/// ```
/// use mysql_handler::hton::{Handlerton, HtonCapabilities, HtonFlags};
///
/// struct MyHandlerton;
/// impl Handlerton for MyHandlerton {
///     fn capabilities(&self) -> HtonCapabilities {
///         HtonCapabilities::TRANSACTIONS
///     }
/// }
///
/// assert!(MyHandlerton.capabilities().contains(HtonCapabilities::TRANSACTIONS));
/// assert_eq!(MyHandlerton.flags(), HtonFlags::CAN_RECREATE);
/// ```
pub trait Handlerton: Send + Sync {
    /// The engine-level callback groups this handlerton implements.
    ///
    /// Each capability gates a group of `handlerton` callbacks; a group is
    /// wired into MySQL only when its bit is set here. Defaults to
    /// [`HtonCapabilities::empty`] — a handler-only engine.
    fn capabilities(&self) -> HtonCapabilities {
        HtonCapabilities::empty()
    }

    /// The `handlerton` flags (`HTON_*`).
    ///
    /// Defaults to [`HtonFlags::CAN_RECREATE`], matching the flag the
    /// zero-config engine sets today. Return [`HtonFlags::NONE`] to opt out.
    fn flags(&self) -> HtonFlags {
        HtonFlags::CAN_RECREATE
    }

    /// Bytes of per-savepoint scratch the engine needs (`handlerton`'s
    /// `savepoint_offset`). MySQL allocates this much per savepoint and hands it
    /// to the savepoint callbacks as their `sv` buffer. Only consulted when the
    /// engine declares [`HtonCapabilities::SAVEPOINTS`]; defaults to 0.
    fn savepoint_offset(&self) -> u32 {
        0
    }

    /// Called when a connection that has touched this engine closes, so the
    /// engine can release per-connection state.
    ///
    /// MySQL only invokes this for a connection whose `thd->ha_data` slot is
    /// non-empty, so a handler-only engine that never stores per-connection
    /// state does not see it. Defaults to success (nothing to release).
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if releasing the
    /// connection's state fails; MySQL logs it and the connection still closes.
    fn close_connection(&self, _thd: Option<&sys::THD>) -> EngineResult {
        Ok(())
    }

    /// Notification that a connection or its current statement is being
    /// terminated (`KILL`). Defaults to no-op.
    fn kill_connection(&self, _thd: Option<&sys::THD>) {}

    /// Called before the data dictionary shuts down so the engine can stop
    /// background tasks that might still access it. Defaults to no-op.
    fn pre_dd_shutdown(&self) {}

    /// Reset session-scoped plugin variables before the connection ends.
    /// Defaults to no-op.
    fn reset_plugin_vars(&self, _thd: Option<&sys::THD>) {}

    /// Create the per-connection [`TxnSession`] for a new transaction.
    ///
    /// Invoked (through the shim) when a connection first joins a transaction,
    /// but only for an engine that declares
    /// [`HtonCapabilities::TRANSACTIONS`]. The returned session is stored in
    /// the connection's `ha_data` and driven through `commit` / `rollback`
    /// until the transaction ends. The default returns an inert session, so an
    /// engine declaring `TRANSACTIONS` must override this to do real work.
    fn begin_transaction(&self) -> Box<dyn TxnSession> {
        Box::new(NoopTxnSession)
    }

    /// Commit the prepared XA transaction identified by `xid`, found during
    /// recovery. Wired only under [`HtonCapabilities::XA`]; `xid` is opaque
    /// (inspect only the bytes the engine needs). Defaults to unsupported.
    ///
    /// # Errors
    /// Returns [`EngineError::Unsupported`](crate::engine::EngineError::Unsupported)
    /// by default; an XA engine overrides this and errors only on a real failure.
    fn commit_by_xid(&self, xid: Option<&sys::XID>) -> EngineResult {
        let _ = xid;
        Err(crate::engine::EngineError::Unsupported)
    }

    /// Roll back the prepared XA transaction identified by `xid`. Wired only
    /// under [`HtonCapabilities::XA`]. Defaults to unsupported.
    ///
    /// # Errors
    /// Returns [`EngineError::Unsupported`](crate::engine::EngineError::Unsupported)
    /// by default.
    fn rollback_by_xid(&self, xid: Option<&sys::XID>) -> EngineResult {
        let _ = xid;
        Err(crate::engine::EngineError::Unsupported)
    }

    /// Mark the connection's externally-coordinated transactions as prepared in
    /// the server transaction coordinator. Wired only under
    /// [`HtonCapabilities::XA`]. Defaults to unsupported.
    ///
    /// # Errors
    /// Returns [`EngineError::Unsupported`](crate::engine::EngineError::Unsupported)
    /// by default.
    fn set_prepared_in_tc(&self, thd: Option<&sys::THD>) -> EngineResult {
        let _ = thd;
        Err(crate::engine::EngineError::Unsupported)
    }

    /// Mark the prepared XA transaction identified by `xid` as prepared in the
    /// server transaction coordinator. Wired only under
    /// [`HtonCapabilities::XA`]. Defaults to unsupported.
    ///
    /// # Errors
    /// Returns [`EngineError::Unsupported`](crate::engine::EngineError::Unsupported)
    /// by default.
    fn set_prepared_in_tc_by_xid(&self, xid: Option<&sys::XID>) -> EngineResult {
        let _ = xid;
        Err(crate::engine::EngineError::Unsupported)
    }

    /// Shutdown notification: the server is invoking `ha_panic` to wind every
    /// engine down. Defaults to success — the engine has nothing to flush on
    /// process exit.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the engine
    /// fails to shut down cleanly; MySQL logs it and continues stopping.
    fn panic(&self, flag: HaPanicFunction) -> EngineResult {
        let _ = flag;
        Ok(())
    }

    /// Start a consistent-snapshot read for the connection, as requested by
    /// `START TRANSACTION WITH CONSISTENT SNAPSHOT`. Wired only when the engine
    /// declares [`HtonCapabilities::TRANSACTIONS`]; defaults to success (the
    /// engine has no snapshot semantics to install).
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the snapshot
    /// cannot be set up.
    fn start_consistent_snapshot(&self, _thd: Option<&sys::THD>) -> EngineResult {
        Ok(())
    }

    /// Flush durable state to disk. `binlog_group_flush` is true when the
    /// invocation comes from the binary log group-commit flush stage, false
    /// from `FLUSH LOGS` or shutdown. Defaults to success (nothing to flush).
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the flush
    /// fails; MySQL reports the failure to the client.
    fn flush_logs(&self, _binlog_group_flush: bool) -> EngineResult {
        Ok(())
    }

    /// Populate `SHOW ENGINE <name> STATUS` / `LOGS` / `MUTEX` output by
    /// emitting rows through `sink`. The trait default emits no rows, leaving
    /// the engine status empty.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if collecting or
    /// emitting status fails.
    fn show_status(
        &self,
        _thd: Option<&sys::THD>,
        _sink: &StatPrintSink<'_>,
        _stat: HaStatType,
    ) -> EngineResult {
        Ok(())
    }

    /// Per-partition capability bitfield (`HA_*_PARTITION_*` flags from
    /// `sql_partition.h`). Consulted only when the engine declares
    /// [`HtonCapabilities::PARTITIONING`]; defaults to 0 — no special
    /// partitioning behaviour.
    fn partition_flags(&self) -> u32 {
        0
    }

    /// Fill engine-defined rows of an `INFORMATION_SCHEMA` table. The default
    /// adds no rows, which is correct for an engine with no engine-only I_S
    /// surface.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if collecting
    /// rows fails.
    fn fill_is_table(&self, _thd: Option<&sys::THD>) -> EngineResult {
        Ok(())
    }

    /// Roll the engine's log files forward as part of an in-place server
    /// upgrade. Defaults to success — the engine has no upgrade-specific log
    /// work to do.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the upgrade
    /// step fails; MySQL aborts the upgrade.
    fn upgrade_logs(&self, _thd: Option<&sys::THD>) -> EngineResult {
        Ok(())
    }

    /// Finalize upgrade-specific state, called regardless of whether the
    /// upgrade succeeded. `failed_upgrade` is true when MySQL rolled back the
    /// upgrade. Defaults to success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the cleanup
    /// step fails.
    fn finish_upgrade(&self, _thd: Option<&sys::THD>, _failed_upgrade: bool) -> EngineResult {
        Ok(())
    }

    /// Whether the supplied database name is reserved by the engine and must
    /// not be created at the SQL layer. Defaults to `false` — the engine
    /// reserves no names.
    fn is_reserved_db_name(&self, _name: &str) -> bool {
        false
    }

    /// Recover a table whose dictionary entry is missing by reading the
    /// engine-side description back into `db.name`. Defaults to "not found"
    /// (the trait returns `Unsupported`, which the shim translates to
    /// `HA_ERR_NO_SUCH_TABLE`).
    ///
    /// The shim does not yet marshal the engine's SDI blob back through the
    /// `frmblob` / `frmlen` output parameters, so overriding this to return
    /// `Ok(())` would claim "found" without supplying the table definition and
    /// MySQL would fail downstream with `ER_NO_SUCH_TABLE` anyway. Keep the
    /// default until that return path is wired.
    ///
    /// # Errors
    /// Returns [`EngineError::Unsupported`](crate::engine::EngineError::Unsupported)
    /// by default to report "no such table".
    fn discover(&self, _thd: Option<&sys::THD>, _db: &str, _name: &str) -> EngineResult {
        Err(crate::engine::EngineError::Unsupported)
    }

    /// List the tables (or directory entries) the engine knows about under
    /// `db` / `path`. `wild` is an optional shell-style filter; `dir` is true
    /// when MySQL is asking for sub-directories. Defaults to success with no
    /// entries reported.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if enumeration
    /// fails.
    fn find_files(
        &self,
        _thd: Option<&sys::THD>,
        _db: &str,
        _path: &str,
        _wild: Option<&str>,
        _dir: bool,
    ) -> EngineResult {
        Ok(())
    }

    /// Whether `db.name` exists in the engine. The handler.cc caller maps
    /// `true` to `HA_ERR_TABLE_EXIST` and `false` to `HA_ERR_NO_SUCH_TABLE`,
    /// so the default `false` matches "engine has no such table".
    fn table_exists_in_engine(&self, _thd: Option<&sys::THD>, _db: &str, _name: &str) -> bool {
        false
    }

    /// Whether `db.table_name` is a system table this engine supports.
    /// `is_sql_layer_system_table` is `true` when the table is an SQL-layer
    /// system table (such as `mysql.*`); the engine should answer `false`
    /// unless it specifically supports those at the engine layer. Defaults to
    /// `false` — the engine supports no system tables.
    fn is_supported_system_table(
        &self,
        _db: &str,
        _table_name: &str,
        _is_sql_layer_system_table: bool,
    ) -> bool {
        false
    }

    /// Notification fired after a `SELECT` completed, with the engine that
    /// actually executed it. Defaults to no-op.
    fn notify_after_select(&self, _thd: Option<&sys::THD>, _executed_in: SelectExecutedIn) {}

    /// Notification fired when a table is created, with the bare `db` /
    /// `table_name` strings. The `HA_CREATE_INFO` parameter is opaque to Rust
    /// today and is not surfaced. Defaults to no-op.
    fn notify_create_table(&self, _db: &str, _table_name: &str) {}

    /// Notification fired when a table is dropped. The `Table_ref` parameter
    /// is opaque to Rust and is not surfaced. Defaults to no-op.
    fn notify_drop_table(&self) {}

    /// Pre/post notification around an exclusive metadata lock acquisition.
    /// Returning an error from the pre-event hook tells MySQL to abort
    /// acquisition (the shim translates `Err` to `true`, matching MySQL's
    /// "failure" convention). Defaults to success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) to veto the
    /// pre-event lock acquisition. Returning `Err` from a post-event
    /// notification is logged by MySQL but does not undo the lock.
    fn notify_exclusive_mdl(
        &self,
        _thd: Option<&sys::THD>,
        _mdl_key: Option<&sys::MdlKey>,
        _kind: HaNotificationType,
    ) -> EngineResult {
        Ok(())
    }

    /// Pre/post notification around an `ALTER TABLE`. Defaults to success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) to veto a
    /// pre-event alter.
    fn notify_alter_table(
        &self,
        _thd: Option<&sys::THD>,
        _mdl_key: Option<&sys::MdlKey>,
        _kind: HaNotificationType,
    ) -> EngineResult {
        Ok(())
    }

    /// Pre/post notification around a `RENAME TABLE`. The old / new
    /// db.table names are passed as borrowed `&str`. Defaults to success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) to veto a
    /// pre-event rename.
    #[allow(clippy::too_many_arguments)]
    fn notify_rename_table(
        &self,
        _thd: Option<&sys::THD>,
        _mdl_key: Option<&sys::MdlKey>,
        _kind: HaNotificationType,
        _old_db: &str,
        _old_name: &str,
        _new_db: &str,
        _new_name: &str,
    ) -> EngineResult {
        Ok(())
    }

    /// Pre/post notification around a `TRUNCATE TABLE`. Defaults to success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) to veto a
    /// pre-event truncate.
    fn notify_truncate_table(
        &self,
        _thd: Option<&sys::THD>,
        _mdl_key: Option<&sys::MdlKey>,
        _kind: HaNotificationType,
    ) -> EngineResult {
        Ok(())
    }

    /// Binlog-related operation MySQL is asking the engine to handle
    /// (`BFN_*`). Defaults to success (engine is binlog-agnostic).
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the operation
    /// fails.
    fn binlog_func(&self, _thd: Option<&sys::THD>, _func: EnumBinlogFunc) -> EngineResult {
        Ok(())
    }

    /// Notification of a DDL command being written to the binary log. The
    /// `query` text is bounded; per the security rule the trait must not log
    /// it. Defaults to no-op.
    fn binlog_log_query(
        &self,
        _thd: Option<&sys::THD>,
        _command: EnumBinlogCommand,
        _query: &str,
        _db: &str,
        _table: &str,
    ) {
    }

    /// Notification of an ACL change (`GRANT` / `REVOKE` / privilege table
    /// modification). The `Acl_change_notification` parameter is opaque to
    /// Rust today and is not surfaced. Defaults to no-op.
    fn acl_notify(&self, _thd: Option<&sys::THD>) {}
}

/// Inert default session returned by [`Handlerton::begin_transaction`] (see
/// there). Accepts and discards transaction boundaries without doing work.
#[derive(Debug)]
struct NoopTxnSession;

impl TxnSession for NoopTxnSession {
    fn commit(&mut self, _all: bool) -> EngineResult {
        Ok(())
    }

    fn rollback(&mut self, _all: bool) -> EngineResult {
        Ok(())
    }
}
