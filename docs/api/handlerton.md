# handlerton — Engine-Level Interface

`handlerton` is a C struct with function pointers (not a C++ class).
One singleton per storage engine, set up in `init_func`.

Source: `mysql-server/sql/handler.h:2734`

## Fields

| Field | Type | Description |
| ----- | ---- | ----------- |
| `state` | `SHOW_COMP_OPTION` | Engine availability |
| `db_type` | `enum legacy_db_type` | Legacy engine type ID |
| `slot` | `uint` | Per-connection data slot (set by MySQL) |
| `savepoint_offset` | `uint` | Size of per-savepoint data |
| `flags` | `uint32` | Engine flags (e.g. `HTON_CAN_RECREATE`) |
| `foreign_keys_flags` | `uint32` | FK support detail flags |
| `fk_name_suffix` | `LEX_CSTRING` | Suffix for auto-generated FK names |
| `secondary_engine_flags` | `SecondaryEngineFlags` | Hypergraph optimizer support flags |
| `license` | `uint32` | Engine license flag |
| `data` | `void*` | Engine private data pointer |
| `file_extensions` | `const char**` | NULL-terminated array of file extensions |

## Callbacks — Core

| # | Callback | Description |
| - | -------- | ----------- |
| 1 | `create` | **Required.** Factory: returns a new `handler` instance |
| 2 | `close_connection` | Cleanup when a connection closes |
| 3 | `kill_connection` | Kill a connection |
| 4 | `pre_dd_shutdown` | Pre-data-dictionary shutdown |
| 5 | `reset_plugin_vars` | Reset plugin variables |

## Callbacks — Transactions & Savepoints

| # | Callback | Description |
| - | -------- | ----------- |
| 6 | `commit` | Commit a transaction |
| 7 | `rollback` | Rollback a transaction |
| 8 | `prepare` | Prepare for 2PC |
| 9 | `recover` | Recover prepared transactions |
| 10 | `recover_prepared_in_tc` | Recover prepared in TC |
| 11 | `commit_by_xid` | Commit by XA ID |
| 12 | `rollback_by_xid` | Rollback by XA ID |
| 13 | `set_prepared_in_tc` | Set prepared state in TC |
| 14 | `set_prepared_in_tc_by_xid` | Set prepared state by XA ID |
| 15 | `savepoint_set` | Set a savepoint |
| 16 | `savepoint_rollback` | Rollback to a savepoint |
| 17 | `savepoint_rollback_can_release_mdl` | Can release MDL on rollback? |
| 18 | `savepoint_release` | Release a savepoint |

## Callbacks — Database & Tablespace

| # | Callback | Description |
| - | -------- | ----------- |
| 19 | `drop_database` | Called when a database is dropped |
| 20 | `is_valid_tablespace_name` | Validate tablespace name |
| 21 | `get_tablespace` | Get tablespace info |
| 22 | `alter_tablespace` | Alter tablespace |
| 23 | `get_tablespace_filename_ext` | Tablespace file extensions |
| 24 | `upgrade_tablespace` | Upgrade tablespace |
| 25 | `upgrade_space_version` | Upgrade space version |
| 26 | `get_tablespace_type` | Get tablespace type |
| 27 | `get_tablespace_type_by_name` | Get tablespace type by name |

## Callbacks — Data Dictionary

| # | Callback | Description |
| - | -------- | ----------- |
| 28 | `dict_init` | Initialize data dictionary |
| 29 | `ddse_dict_init` | DD SE dictionary init |
| 30 | `dict_register_dd_table_id` | Register DD table ID |
| 31 | `dict_cache_reset` | Reset dictionary cache |
| 32 | `dict_cache_reset_tables_and_tablespaces` | Reset table/tablespace cache |
| 33 | `dict_recover` | Recover dictionary |
| 34 | `dict_get_server_version` | Get server version from DD |
| 35 | `dict_set_server_version` | Set server version in DD |

## Callbacks — Status & Discovery

| # | Callback | Description |
| - | -------- | ----------- |
| 36 | `panic` | Called on server shutdown/crash |
| 37 | `start_consistent_snapshot` | Start consistent snapshot |
| 38 | `flush_logs` | Flush engine logs |
| 39 | `show_status` | SHOW ENGINE STATUS |
| 40 | `partition_flags` | Partition support flags |
| 41 | `fill_is_table` | Fill INFORMATION_SCHEMA |
| 42 | `upgrade_logs` | Upgrade log files |
| 43 | `finish_upgrade` | Finish upgrade process |
| 44 | `is_reserved_db_name` | Is DB name reserved? |
| 45 | `discover` | Discover tables |
| 46 | `find_files` | Find files for discovery |
| 47 | `table_exists_in_engine` | Check table existence |
| 48 | `is_supported_system_table` | Check system table support |

## Callbacks — Binlog

| # | Callback | Description |
| - | -------- | ----------- |
| 49 | `binlog_func` | Binlog function |
| 50 | `binlog_log_query` | Log query to binlog |
| 51 | `acl_notify` | ACL change notification |

## Callbacks — SDI (Serialized Dictionary Information)

| # | Callback | Description |
| - | -------- | ----------- |
| 52 | `sdi_create` | Create SDI |
| 53 | `sdi_drop` | Drop SDI |
| 54 | `sdi_get_keys` | Get SDI keys |
| 55 | `sdi_get` | Get SDI data |
| 56 | `sdi_set` | Set SDI data |
| 57 | `sdi_delete` | Delete SDI entry |

## Callbacks — Logging

| # | Callback | Description |
| - | -------- | ----------- |
| 58 | `lock_hton_log` | Lock engine log |
| 59 | `unlock_hton_log` | Unlock engine log |
| 60 | `collect_hton_log_info` | Collect engine log info |

## Callbacks — Foreign Keys

| # | Callback | Description |
| - | -------- | ----------- |
| 61 | `check_fk_column_compat` | Check FK column compatibility |

## Callbacks — Secondary Engine

| # | Callback | Description |
| - | -------- | ----------- |
| 62 | `prepare_secondary_engine` | Prepare statement for secondary engine |
| 63 | `optimize_secondary_engine` | Optimize for secondary engine |
| 64 | `compare_secondary_engine_cost` | Compare join cost in secondary engine |
| 65 | `external_engine_explain_check` | Check if table loaded for EXPLAIN |
| 66 | `secondary_engine_modify_access_path_cost` | Modify access path cost |
| 67 | `get_secondary_engine_offload_or_exec_fail_reason` | Get offload/exec fail reason |
| 68 | `find_secondary_engine_offload_fail_reason` | Find offload fail reason |
| 69 | `set_secondary_engine_offload_fail_reason` | Set offload fail reason |
| 70 | `secondary_engine_check_optimizer_request` | Check optimizer request |
| 71 | `secondary_engine_pre_prepare_hook` | Pre-prepare hook |

## Callbacks — Transaction Hooks

| # | Callback | Description |
| - | -------- | ----------- |
| 72 | `se_before_commit` | Hook before commit |
| 73 | `se_after_commit` | Hook after commit |
| 74 | `se_before_rollback` | Hook before rollback |

## Callbacks — Event Notifications

| # | Callback | Description |
| - | -------- | ----------- |
| 75 | `notify_after_select` | Notification after SELECT |
| 76 | `notify_create_table` | Notification on CREATE TABLE |
| 77 | `notify_drop_table` | Notification on DROP TABLE |

## Callbacks — Misc

| # | Callback | Description |
| - | -------- | ----------- |
| 78 | `push_to_engine` | Push conditions to engine |
| 79 | `is_dict_readonly` | Is dictionary read-only? |
| 80 | `rm_tmp_tables` | Remove temporary tables |
| 81 | `get_cost_constants` | Provide cost constants |
| 82 | `replace_native_transaction_in_thd` | Replace native transaction |
| 83 | `notify_exclusive_mdl` | Exclusive MDL notification |
| 84 | `notify_alter_table` | ALTER TABLE notification |
| 85 | `notify_rename_table` | RENAME TABLE notification |
| 86 | `notify_truncate_table` | TRUNCATE notification |
| 87 | `rotate_encryption_master_key` | Rotate encryption key |
| 88 | `redo_log_set_state` | Set redo log state |
| 89 | `get_table_statistics` | Get table statistics |
| 90 | `get_index_column_cardinality` | Get index cardinality |
| 91 | `get_tablespace_statistics` | Get tablespace statistics |
| 92 | `post_ddl` | Post-DDL callback |
| 93 | `post_recover` | Post-recovery callback |
| 94 | `clone_interface` | Clone data transfer interface |

## Page Tracking

| Field | Type | Description |
| ----- | ---- | ----------- |
| `page_track` | `Page_track_t` | Page tracking interface (struct with start/stop/purge/get_page_ids/get_num_page_ids/get_status) |

## Total: 94 callbacks + `page_track` struct

Only `create` is required. All others default to NULL.

## Plugin Registration

```cpp
struct st_mysql_storage_engine engine = { MYSQL_HANDLERTON_INTERFACE_VERSION };

mysql_declare_plugin(name) {
    MYSQL_STORAGE_ENGINE_PLUGIN,
    &engine,
    "ENGINE_NAME",
    "Author",
    "Description",
    PLUGIN_LICENSE_GPL,
    init_func,       // Sets up handlerton
    nullptr,         // check_uninstall
    deinit_func,     // Cleanup
    0x0001,          // Version
    status_vars,     // SHOW STATUS
    system_vars,     // SHOW VARIABLES
    nullptr,         // config
    0                // flags
} mysql_declare_plugin_end;
```
