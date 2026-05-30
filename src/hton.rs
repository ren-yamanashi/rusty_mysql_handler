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
pub mod database;
#[doc(hidden)]
pub mod dict;
mod dict_kind;
#[doc(hidden)]
pub mod discovery;
#[doc(hidden)]
pub mod engine_log;
#[doc(hidden)]
pub mod ffi;
#[doc(hidden)]
pub mod fk_hooks;
mod flags;
#[doc(hidden)]
pub mod lifecycle;
mod notification_kind;
#[doc(hidden)]
pub mod notifications;
mod panic_function;
#[doc(hidden)]
pub mod savepoint_ffi;
#[doc(hidden)]
pub mod sdi;
#[doc(hidden)]
pub mod secondary_engine;
#[doc(hidden)]
pub mod secondary_engine_fail_reason;
mod secondary_engine_kind;
mod stat_print_sink;
mod stat_type;
#[doc(hidden)]
pub mod status;
#[doc(hidden)]
pub mod tablespace;
mod tablespace_kind;
mod transaction;
#[doc(hidden)]
pub mod txn_context;
#[doc(hidden)]
pub mod txn_ffi;
#[doc(hidden)]
pub mod xa;

pub use binlog_kind::{BinlogCommand, BinlogFunc};
pub use capabilities::HtonCapabilities;
pub use dict_kind::{DictInitMode, DictRecoveryMode};
pub use flags::HtonFlags;
pub use notification_kind::{HaNotificationType, SelectExecutedIn};
pub use panic_function::HaPanicFunction;
pub use secondary_engine_kind::{
    SecondaryEngineGraphSimplificationRequest, SecondaryEngineOptimizerRequest,
};
pub use stat_print_sink::StatPrintSink;
pub use stat_type::HaStatType;
pub use tablespace_kind::{TablespaceType, TsCommandType};
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
    fn binlog_func(&self, _thd: Option<&sys::THD>, _func: BinlogFunc) -> EngineResult {
        Ok(())
    }

    /// Notification of a DDL command being written to the binary log. The
    /// `query` text is bounded; per the security rule the trait must not log
    /// it. Defaults to no-op.
    fn binlog_log_query(
        &self,
        _thd: Option<&sys::THD>,
        _command: BinlogCommand,
        _query: &str,
        _db: &str,
        _table: &str,
    ) {
    }

    /// Notification of an ACL change (`GRANT` / `REVOKE` / privilege table
    /// modification). The `Acl_change_notification` parameter is opaque to
    /// Rust today and is not surfaced. Defaults to no-op.
    fn acl_notify(&self, _thd: Option<&sys::THD>) {}

    /// Notification of `DROP DATABASE`. `path` is the schema's storage path
    /// (typically `./<dbname>`). Always wired on a registered handlerton.
    /// Defaults to no-op.
    fn drop_database(&self, _path: &str) {}

    /// Whether `tablespace_name` is acceptable for the given DDL command. Wired
    /// only under [`HtonCapabilities::TABLESPACES`]; defaults to `true` so the
    /// engine does not reject names for an unrelated reason.
    fn is_valid_tablespace_name(&self, _cmd: TsCommandType, _tablespace_name: &str) -> bool {
        true
    }

    /// Look up the tablespace name that holds `db.table_name`. The current
    /// binding does not yet round-trip the output back to MySQL — the shim
    /// leaves the `LEX_CSTRING*` empty — so an override returning `Ok(())` is
    /// equivalent to "no tablespace information available".
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) on lookup
    /// failure.
    fn get_tablespace(
        &self,
        _thd: Option<&sys::THD>,
        _db_name: &str,
        _table_name: &str,
    ) -> EngineResult {
        Ok(())
    }

    /// Apply a tablespace DDL (`CREATE TABLESPACE`, `ALTER TABLESPACE`, ...).
    /// `ts_info` is opaque today. Wired only under
    /// [`HtonCapabilities::TABLESPACES`]; defaults to success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the DDL fails.
    fn alter_tablespace(
        &self,
        _thd: Option<&sys::THD>,
        _ts_info: Option<&sys::StAlterTablespace>,
    ) -> EngineResult {
        Ok(())
    }

    /// Default file-extension MySQL appends to tablespace data files. Wired
    /// only under [`HtonCapabilities::TABLESPACES`]; default `None` produces a
    /// NULL pointer at the C boundary (no extension).
    fn tablespace_filename_ext(&self) -> Option<&'static core::ffi::CStr> {
        None
    }

    /// Upgrade tablespace-level state from a previous server version.
    /// Defaults to success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the upgrade
    /// step fails.
    fn upgrade_tablespace(&self, _thd: Option<&sys::THD>) -> EngineResult {
        Ok(())
    }

    /// Upgrade the on-disk version of `tablespace`. Defaults to success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the upgrade
    /// step fails.
    fn upgrade_space_version(&self, _tablespace: Option<&sys::DdTablespace>) -> EngineResult {
        Ok(())
    }

    /// Classification of the given tablespace. Defaults to `None`, which the
    /// shim reports back to MySQL as failure to determine the type (so MySQL
    /// keeps whatever it already knows).
    fn get_tablespace_type(
        &self,
        _tablespace: Option<&sys::DdTablespace>,
    ) -> Option<TablespaceType> {
        None
    }

    /// Classification of the tablespace identified by `tablespace_name`.
    /// Defaults to `None` (see [`Self::get_tablespace_type`]).
    fn get_tablespace_type_by_name(&self, _tablespace_name: &str) -> Option<TablespaceType> {
        None
    }

    /// Initialise the engine as the data-dictionary backend. The DD-tables /
    /// DD-tablespaces output lists are not surfaced today (they are produced
    /// only by the DD backend). Wired only under
    /// [`HtonCapabilities::DICT_BACKEND`]; defaults to success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) on init failure.
    fn dict_init(&self, _mode: DictInitMode, _version: u32) -> EngineResult {
        Ok(())
    }

    /// DD-backend variant of [`Self::dict_init`] used by the DDSE-specific
    /// startup path. Defaults to success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) on init failure.
    fn ddse_dict_init(&self, _mode: DictInitMode, _version: u32) -> EngineResult {
        Ok(())
    }

    /// Register the hard-coded DD table-id range with the engine. `table_id`
    /// is `dd::Object_id` (a 64-bit integer). Defaults to no-op.
    fn dict_register_dd_table_id(&self, _table_id: u64) {}

    /// Invalidate the engine's local cache entry for `schema.table`.
    /// Defaults to no-op.
    fn dict_cache_reset(&self, _schema_name: &str, _table_name: &str) {}

    /// Invalidate every table and tablespace entry in the engine's local
    /// dictionary cache. Defaults to no-op.
    fn dict_cache_reset_tables_and_tablespaces(&self) {}

    /// Perform engine-side recovery work as part of dictionary initialisation.
    /// Defaults to success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if recovery
    /// fails.
    fn dict_recover(&self, _mode: DictRecoveryMode, _version: u32) -> EngineResult {
        Ok(())
    }

    /// Read back the server version stored in the dictionary tablespace
    /// header. Defaults to `None`, which the shim reports back as failure.
    fn dict_get_server_version(&self) -> Option<u32> {
        None
    }

    /// Persist the current server version into the dictionary tablespace
    /// header. Defaults to success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) on write
    /// failure.
    fn dict_set_server_version(&self) -> EngineResult {
        Ok(())
    }

    /// Create the SDI store for `tablespace`. Wired only under
    /// [`HtonCapabilities::SDI`]; defaults to unsupported.
    ///
    /// # Errors
    /// Returns [`EngineError::Unsupported`](crate::engine::EngineError::Unsupported)
    /// by default.
    fn sdi_create(&self, _tablespace: Option<&sys::DdTablespace>) -> EngineResult {
        Err(crate::engine::EngineError::Unsupported)
    }

    /// Drop the SDI store from `tablespace`. Defaults to unsupported.
    ///
    /// # Errors
    /// Returns [`EngineError::Unsupported`](crate::engine::EngineError::Unsupported)
    /// by default.
    fn sdi_drop(&self, _tablespace: Option<&sys::DdTablespace>) -> EngineResult {
        Err(crate::engine::EngineError::Unsupported)
    }

    /// Populate `vector` with the SDI keys present in `tablespace`. The
    /// `sdi_vector_t` output cannot be filled through the opaque
    /// pass-through today; defaults to unsupported.
    ///
    /// # Errors
    /// Returns [`EngineError::Unsupported`](crate::engine::EngineError::Unsupported)
    /// by default.
    fn sdi_get_keys(
        &self,
        _tablespace: Option<&sys::DdTablespace>,
        _vector: Option<&sys::SdiVector>,
    ) -> EngineResult {
        Err(crate::engine::EngineError::Unsupported)
    }

    /// Look up the SDI payload identified by `key` and write it into `buf`.
    /// On success the engine must set `*len_out` to the bytes actually
    /// written. The shim treats `buf` as a `&mut [u8]` whose length is the
    /// caller-provided capacity. Defaults to unsupported.
    ///
    /// MySQL distinguishes the two error paths by inspecting `*len_out`:
    /// - **Buffer too small** — return an error and set `*len_out` to the
    ///   required payload size so the caller can retry with a larger buffer.
    /// - **Genuine error** (key not found, I/O failure, ...) — return an
    ///   error and set `*len_out = u64::MAX`. Without this sentinel MySQL
    ///   re-enters the call expecting a retry and may loop indefinitely.
    ///
    /// # Errors
    /// Returns [`EngineError::Unsupported`](crate::engine::EngineError::Unsupported)
    /// by default. Engines that store SDI follow the two-path convention
    /// above.
    fn sdi_get(
        &self,
        _tablespace: Option<&sys::DdTablespace>,
        _key: Option<&sys::SdiKey>,
        _buf: &mut [u8],
        _len_out: &mut u64,
    ) -> EngineResult {
        Err(crate::engine::EngineError::Unsupported)
    }

    /// Store `payload` as the SDI value for `key` against `table` /
    /// `tablespace`. Defaults to unsupported.
    ///
    /// # Errors
    /// Returns [`EngineError::Unsupported`](crate::engine::EngineError::Unsupported)
    /// by default.
    fn sdi_set(
        &self,
        _tablespace: Option<&sys::DdTablespace>,
        _table: Option<&sys::DdTable>,
        _key: Option<&sys::SdiKey>,
        _payload: &[u8],
    ) -> EngineResult {
        Err(crate::engine::EngineError::Unsupported)
    }

    /// Delete the SDI value identified by `key`. Defaults to unsupported.
    ///
    /// # Errors
    /// Returns [`EngineError::Unsupported`](crate::engine::EngineError::Unsupported)
    /// by default.
    fn sdi_delete(
        &self,
        _tablespace: Option<&sys::DdTablespace>,
        _table: Option<&sys::DdTable>,
        _key: Option<&sys::SdiKey>,
    ) -> EngineResult {
        Err(crate::engine::EngineError::Unsupported)
    }

    /// Acquire the engine-log mutex so a backup tool can snapshot a
    /// consistent log state. Wired only under
    /// [`HtonCapabilities::ENGINE_LOG`]; defaults to success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) when the lock
    /// cannot be taken.
    fn lock_hton_log(&self) -> EngineResult {
        Ok(())
    }

    /// Release the mutex taken by [`Self::lock_hton_log`]. Defaults to
    /// success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) on release
    /// failure.
    fn unlock_hton_log(&self) -> EngineResult {
        Ok(())
    }

    /// Append the engine's redo / transaction log status into the
    /// `performance_schema.log_status` collector. The `Json_dom` parameter is
    /// opaque today; an engine with log info will need a reverse callback to
    /// populate the JSON tree. Defaults to no-op success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) on collection
    /// failure.
    fn collect_hton_log_info(&self, _json: Option<&sys::JsonDom>) -> EngineResult {
        Ok(())
    }

    /// Whether the engine considers the two foreign-key column type
    /// descriptors compatible (used when MySQL validates an `ADD FOREIGN KEY`
    /// referencing this engine's tables). `Ha_fk_column_type` is opaque
    /// today; the trait default returns `true` (accept any), which matches the
    /// permissive base-engine behaviour. Override to enforce stricter typing.
    fn check_fk_column_compat(
        &self,
        _child: Option<&sys::HaFkColumnType>,
        _parent: Option<&sys::HaFkColumnType>,
        _check_charsets: bool,
    ) -> bool {
        true
    }

    /// Plugin-observer hook fired before a transaction commits. The shim
    /// discards the observer's `void*` argument because it belongs to the
    /// observer plugin that registered the hook, not to the storage engine;
    /// engines that need observer data must coordinate with the observer
    /// through a separate channel. Defaults to no-op.
    fn se_before_commit(&self) {}

    /// Plugin-observer hook fired after a transaction commits. See
    /// [`Self::se_before_commit`] for the observer-arg discussion. Defaults
    /// to no-op.
    fn se_after_commit(&self) {}

    /// Plugin-observer hook fired before a transaction rolls back. See
    /// [`Self::se_before_commit`] for the observer-arg discussion. Defaults
    /// to no-op.
    fn se_before_rollback(&self) {}

    /// Prepare the secondary engine for executing a statement. `lex` is
    /// opaque today. Wired only under
    /// [`HtonCapabilities::SECONDARY_ENGINE`]; defaults to success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if preparation
    /// fails; MySQL falls back to the primary engine.
    fn prepare_secondary_engine(
        &self,
        _thd: Option<&sys::THD>,
        _lex: Option<&sys::Lex>,
    ) -> EngineResult {
        Ok(())
    }

    /// Optimize a statement for execution on the secondary engine. `lex` is
    /// opaque today. Defaults to success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the
    /// optimization fails; MySQL reprepares for the primary engine.
    fn optimize_secondary_engine(
        &self,
        _thd: Option<&sys::THD>,
        _lex: Option<&sys::Lex>,
    ) -> EngineResult {
        Ok(())
    }

    /// Compare the cost of `join` against the best plan seen so far. The
    /// trait returns `(use_best_so_far, cheaper, secondary_engine_cost)`;
    /// `None` defaults the triple to `(false, false, optimizer_cost)`, which
    /// the shim writes back to the C `bool*` / `double*` outputs.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) on error; the
    /// optimizer drops the candidate plan.
    fn compare_secondary_engine_cost(
        &self,
        _thd: Option<&sys::THD>,
        _join: Option<&sys::Join>,
        _optimizer_cost: f64,
    ) -> EngineResult<Option<(bool, bool, f64)>> {
        Ok(None)
    }

    /// Evaluate (and potentially modify) the cost estimates on `access_path`
    /// from the hypergraph optimizer. `access_path` is opaque to Rust today,
    /// so the default cannot modify costs; returning `Ok(())` accepts the
    /// path unchanged.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) to reject the
    /// access path entirely.
    fn secondary_engine_modify_access_path_cost(
        &self,
        _thd: Option<&sys::THD>,
        _hypergraph: Option<&sys::JoinHypergraph>,
        _access_path: Option<&sys::AccessPath>,
    ) -> EngineResult {
        Ok(())
    }

    /// Whether `EXPLAIN` references tables not loaded into the secondary
    /// engine. Defaults to `false` (all referenced tables are loaded).
    fn external_engine_explain_check(&self, _thd: Option<&sys::THD>) -> bool {
        false
    }

    // `get_secondary_engine_offload_or_exec_fail_reason` and
    // `find_secondary_engine_offload_fail_reason` are wired at the FFI layer
    // but intentionally not surfaced as trait methods today: returning
    // engine-owned bytes by value would drop the buffer before MySQL wrapped
    // it in `std::string_view`, exposing freed heap memory. The FFI returns an
    // empty view until a future setter reverse-callback can hand
    // statement-scoped bytes to MySQL safely. `set_*` below is unaffected.

    /// Persist `reason` as the offload failure reason for the query
    /// represented by `thd`. Defaults to success.
    ///
    /// # Errors
    /// Returns an [`EngineError`](crate::engine::EngineError) if the engine
    /// could not record the reason.
    fn set_secondary_engine_offload_fail_reason(
        &self,
        _thd: Option<&sys::THD>,
        _reason: &str,
    ) -> EngineResult {
        Ok(())
    }

    /// Hook the hypergraph optimizer calls after
    /// [`Self::secondary_engine_modify_access_path_cost`] to decide whether
    /// to keep optimizing, restart with a different subgraph budget, etc.
    /// Defaults to [`SecondaryEngineOptimizerRequest::keep_going`]. The
    /// `JoinHypergraph` / `AccessPath` / `trace` parameters are opaque.
    fn secondary_engine_check_optimizer_request(
        &self,
        _thd: Option<&sys::THD>,
        _hypergraph: Option<&sys::JoinHypergraph>,
        _access_path: Option<&sys::AccessPath>,
        _current_subgraph_pairs: i32,
        _current_subgraph_pairs_limit: i32,
        _is_root_access_path: bool,
    ) -> SecondaryEngineOptimizerRequest {
        SecondaryEngineOptimizerRequest::keep_going()
    }

    /// Pre-prepare hook called early in optimization to decide whether the
    /// secondary engine's full prepare path should run. Defaults to `false`
    /// (skip the prepare path), matching the upstream "no further prepare"
    /// signal.
    fn secondary_engine_pre_prepare_hook(&self, _thd: Option<&sys::THD>) -> bool {
        false
    }
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
