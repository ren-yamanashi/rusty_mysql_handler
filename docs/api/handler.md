# handler — Table-Level Interface

`handler` is a C++ class with virtual methods.
One instance per thread per open table, created by `handlerton::create`.

Source: `mysql-server/sql/handler.h`

Legend: **PV** = pure virtual (must override), **D** = has default implementation

## Table Lifecycle

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 1 | `open` | 6661 | PV | `int(const char *name, int mode, uint test_if_locked, const dd::Table*)` | Open a table |
| 2 | `close` | 6663 | PV | `int()` | Close a table |
| 3 | `create` | 7052 | D | `int(const char *name, TABLE*, HA_CREATE_INFO*, dd::Table*)` | Create a table |
| 4 | `delete_table` | 6650 | D | `int(const char *name, const dd::Table*)` | Drop a table |
| 5 | `rename_table` | 6630 | D | `int(const char *from, const char *to, const dd::Table*, dd::Table*)` | Rename a table |
| 6 | `drop_table` | 7031 | D | `void(const char *name)` | Drop table (called from handler) |
| 7 | `truncate` | 6960 | D | `int(dd::Table*)` | Truncate table |
| 8 | `change_table_ptr` | 5169 | D | `void(TABLE*, TABLE_SHARE*)` | Update internal table/share pointers |
| 9 | `get_se_private_data` | 7055 | D | `bool(dd::Table*, bool)` | Set SE private data in DD |
| 10 | `get_extra_columns_and_keys` | 7078 | D | `int(const HA_CREATE_INFO*, const List<Create_field>*, uint, dd::Table*)` | Add hidden columns/keys |
| 11 | `upgrade_table` | 6806 | D | `bool(THD*, const char*, TABLE*, dd::Table*)` | Upgrade table for new version |

## Full Table Scan

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 12 | `rnd_init` | 6679 | PV | `int(bool scan)` | Begin full table scan |
| 13 | `rnd_end` | 6680 | D | `int()` | End full table scan |
| 14 | `rnd_next` | 5693 | PV | `int(uchar *buf)` | Fetch next row |
| 15 | `rnd_pos` | 5695 | PV | `int(uchar *buf, uchar *pos)` | Fetch row by position |
| 16 | `rnd_pos_by_record` | 5705 | D | `int(uchar *record)` | Fetch position from record |
| 17 | `position` | 5745 | PV | `void(const uchar *record)` | Store current row position |

## Index Operations

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 18 | `index_init` | 6664 | D | `int(uint idx, bool sorted)` | Begin index scan |
| 19 | `index_end` | 6668 | D | `int()` | End index scan |
| 20 | `index_read` | 6878 | D | `int(uchar*, const uchar*, uint, ha_rkey_function)` | Index read (raw key) |
| 21 | `index_read_map` | 5617 | D | `int(uchar*, const uchar*, key_part_map, ha_rkey_function)` | Index read (key part map) |
| 22 | `index_read_idx_map` | 5629 | D | `int(uchar*, uint, const uchar*, key_part_map, ha_rkey_function)` | Index read on specific index |
| 23 | `index_read_last` | 6884 | D | `int(uchar*, const uchar*, uint)` | Read last matching key |
| 24 | `index_read_last_map` | 5657 | D | `int(uchar*, const uchar*, key_part_map)` | Read last (key part map) |
| 25 | `index_next` | 5638 | D | `int(uchar*)` | Next in index order |
| 26 | `index_prev` | 5641 | D | `int(uchar*)` | Previous in index order |
| 27 | `index_first` | 5644 | D | `int(uchar*)` | First in index |
| 28 | `index_last` | 5647 | D | `int(uchar*)` | Last in index |
| 29 | `index_next_same` | 5650 | D | `int(uchar*, const uchar*, uint)` | Next with same key |
| 30 | `read_range_first` | 5663 | D | `int(const key_range*, const key_range*, bool, bool)` | Begin range scan |
| 31 | `read_range_next` | 5666 | D | `int()` | Next in range scan |
| 32 | `records_in_range` | 5734 | D | `ha_rows(uint, key_range*, key_range*)` | Estimate rows in range |
| 33 | `index_read_pushed` | 6200 | D | `int(uchar*, const uchar*, key_part_map)` | Read from pushed join |
| 34 | `index_next_pushed` | 6204 | D | `int(uchar*)` | Next from pushed join |

## Row Operations

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 35 | `write_row` | 6702 | D | `int(uchar *buf)` | Insert a row |
| 36 | `update_row` | 6714 | D | `int(const uchar *old, uchar *new)` | Update a row |
| 37 | `delete_row` | 6719 | D | `int(const uchar *buf)` | Delete a row |
| 38 | `delete_all_rows` | 6925 | D | `int()` | Delete all rows |

## Bulk Operations

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 39 | `start_bulk_insert` | 6781 | D | `void(ha_rows)` | Begin bulk insert |
| 40 | `end_bulk_insert` | 6782 | D | `int()` | End bulk insert |
| 41 | `start_bulk_update` | 5572 | D | `bool()` | Begin bulk update |
| 42 | `exec_bulk_update` | 5589 | D | `int(uint*)` | Execute bulk update |
| 43 | `end_bulk_update` | 5597 | D | `void()` | End bulk update |
| 44 | `bulk_update_row` | 6908 | D | `int(const uchar*, uchar*, uint*)` | Bulk update single row |
| 45 | `start_bulk_delete` | 5577 | D | `bool()` | Begin bulk delete |
| 46 | `end_bulk_delete` | 5604 | D | `int()` | End bulk delete |

## Bulk Load

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 47 | `bulk_load_check` | 5063 | D | `bool(THD*) const` | Check if bulk load is possible |
| 48 | `bulk_load_available_memory` | 5070 | D | `size_t(THD*) const` | Available memory for bulk load |
| 49 | `bulk_load_execute` | 5094 | D | `int(THD*, size_t, const uchar*, size_t*)` | Execute bulk load |
| 50 | `bulk_load_end` | 5109 | D | `int(THD*, bool)` | End bulk load |
| 51 | `load_table` | 6848 | D | `int(const TABLE&, bool*)` | Load table into engine |
| 52 | `unload_table` | 6868 | D | `int(const char*, const char*, bool)` | Unload table from engine |

## Parallel Scan

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 53 | `parallel_scan_init` | 4981 | D | `int(void*&, size_t*, bool, size_t)` | Init parallel scan |
| 54 | `parallel_scan` | 5046 | D | `int(void*, void**, Reader::Init_fn, Reader::Row_fn, Reader::End_fn)` | Execute parallel scan |
| 55 | `parallel_scan_end` | 5058 | D | `void(void*)` | End parallel scan |

## Sampling

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 56 | `sample_init` | 6822 | D | `int(void*&, double, int, enum_sampling_method)` | Init sampling |
| 57 | `sample_next` | 6831 | D | `int(void*, uchar*)` | Next sample row |
| 58 | `sample_end` | 6836 | D | `int(void*)` | End sampling |

## Full-Text Search

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 59 | `ft_init` | 5682 | D | `int()` | Init full-text search |
| 60 | `ft_read` | 5697 | D | `int(uchar*)` | Read next full-text result |

## Multi-Range Read (MRR)

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 61 | `multi_range_read_info_const` | 5397 | D | `ha_rows(uint, RANGE_SEQ_IF*, void*, uint, uint*, ha_rows*, Cost_estimate*)` | MRR cost estimate (const) |
| 62 | `multi_range_read_info` | 5400 | D | `ha_rows(uint, uint, uint, uint*, Cost_estimate*)` | MRR cost estimate |
| 63 | `multi_range_read_init` | 5403 | D | `int(RANGE_SEQ_IF*, void*, uint, uint, HANDLER_BUFFER*)` | Init MRR scan |
| 64 | `multi_range_read_next` | 5457 | D | `int(char**)` | Next MRR result |

## Engine Properties & Metadata

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 65 | `table_type` | 5984 | PV | `const char*() const` | Engine name string |
| 66 | `table_flags` | 6728 | PV | `Table_flags() const` | Engine capability bitmap |
| 67 | `index_flags` | 5986 | PV | `ulong(uint, uint, bool) const` | Per-index capabilities |
| 68 | `max_supported_record_length` | 6004 | D | `uint() const` | Max row size |
| 69 | `max_supported_keys` | 6005 | D | `uint() const` | Max number of indexes |
| 70 | `max_supported_key_parts` | 6006 | D | `uint() const` | Max key parts |
| 71 | `max_supported_key_length` | 6007 | D | `uint() const` | Max key length in bytes |
| 72 | `max_supported_key_part_length` | 6008 | D | `uint(HA_CREATE_INFO*) const` | Max single key part length |
| 73 | `min_record_length` | 6012 | D | `uint(uint) const` | Min row size |
| 74 | `low_byte_first` | 6016 | D | `bool() const` | Little-endian storage? |
| 75 | `checksum` | 6017 | D | `ha_checksum() const` | Table checksum |
| 76 | `is_crashed` | 6026 | D | `bool() const` | Is table crashed? |
| 77 | `auto_repair` | 6035 | D | `bool() const` | Auto-repair on open? |
| 78 | `primary_key_is_clustered` | 6094 | D | `bool() const` | Is PK clustered? |
| 79 | `get_real_row_type` | 5530 | D | `enum row_type(const HA_CREATE_INFO*) const` | Actual row format |
| 80 | `get_default_index_algorithm` | 5543 | D | `enum ha_key_alg() const` | Default index algorithm |
| 81 | `is_index_algorithm_supported` | 5554 | D | `bool(ha_key_alg) const` | Is index algo supported? |
| 82 | `extra_rec_buf_length` | 5416 | D | `uint() const` | Extra record buffer space needed |
| 83 | `get_memory_buffer_size` | 5313 | D | `longlong() const` | Memory buffer size |
| 84 | `is_record_buffer_wanted` | 6800 | D | `bool(ha_rows*) const` | Wants record buffer? |
| 85 | `explain_extra` | 4842 | D | `std::string() const` | Extra EXPLAIN output |
| 86 | `indexes_are_disabled` | 5978 | D | `int()` | Are indexes disabled? |

## Cost Estimation

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 87 | `scan_time` | 5183 | D | `double()` | Full scan cost |
| 88 | `read_time` | 5202 | D | `double(uint, uint, ha_rows)` | Index read cost |
| 89 | `index_only_read_time` | 5212 | D | `double(uint, double)` | Covering index cost |
| 90 | `table_scan_cost` | 5223 | D | `Cost_estimate()` | Full scan cost (Cost_estimate) |
| 91 | `index_scan_cost` | 5245 | D | `Cost_estimate(uint, double, double)` | Index scan cost |
| 92 | `read_cost` | 5261 | D | `Cost_estimate(uint, double, double)` | Read cost |
| 93 | `page_read_cost` | 5295 | D | `double(uint, double)` | Page read cost |
| 94 | `worst_seek_times` | 5306 | D | `double(double)` | Worst-case seek cost |
| 95 | `records` | 5467 | D | `int(ha_rows*)` | Exact row count |
| 96 | `records_from_index` | 5478 | D | `int(ha_rows*, uint)` | Row count from index |
| 97 | `estimate_rows_upper_bound` | 5522 | D | `ha_rows()` | Upper bound row estimate |
| 98 | `calculate_key_hash_value` | 5775 | D | `uint32(Field**)` | Key hash for partitioning |

## Locking

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 99 | `store_lock` | 6083 | PV | `THR_LOCK_DATA**(THD*, THR_LOCK_DATA**, thr_lock_type)` | Provide lock data |
| 100 | `external_lock` | 6763 | D | `int(THD*, int)` | Statement-level lock/unlock |
| 101 | `lock_count` | 6050 | D | `uint() const` | Number of locks needed |
| 102 | `unlock_row` | 5908 | D | `void()` | Unlock current row |
| 103 | `start_stmt` | 5923 | D | `int(THD*, thr_lock_type)` | Start statement in transaction |
| 104 | `was_semi_consistent_read` | 5892 | D | `bool()` | Semi-consistent read occurred? |
| 105 | `try_semi_consistent_read` | 5899 | D | `void(bool)` | Enable semi-consistent read |

## Read Removal

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 106 | `start_read_removal` | 5834 | D | `bool()` | Start read-free replication |
| 107 | `end_read_removal` | 5844 | D | `ha_rows()` | End read-free replication |

## Auto-Increment

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 108 | `get_auto_increment` | 5927 | D | `void(ulonglong, ulonglong, ulonglong, ulonglong*, ulonglong*)` | Get next auto-inc value |
| 109 | `release_auto_increment` | 6767 | D | `void()` | Release unused auto-inc values |

## Error Handling

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 110 | `print_error` | 5135 | D | `void(int, myf)` | Print error message |
| 111 | `get_error_message` | 5136 | D | `bool(int, String*)` | Get error message text |
| 112 | `get_foreign_dup_key` | 5156 | D | `bool(char*, size_t, char*, size_t)` | FK duplicate key info |
| 113 | `is_ignorable_error` | 5437 | D | `bool(int)` | Can error be ignored? |
| 114 | `is_fatal_error` | 5454 | D | `bool(int)` | Is error fatal? |

## Hints & Extras

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 115 | `extra` | 5802 | D | `int(ha_extra_function)` | Server hint to engine |
| 116 | `extra_opt` | 5807 | D | `int(ha_extra_function, ulong)` | Server hint with cache size |
| 117 | `reset` | 6727 | D | `int()` | Reset state between statements |
| 118 | `column_bitmaps_signal` | 5565 | D | `void()` | Column bitmap changed |
| 119 | `init_table_handle_for_HANDLER` | 5980 | D | `void()` | Init for HANDLER command |

## ALTER TABLE (In-Place)

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 120 | `check_if_supported_inplace_alter` | 6378 | D | `enum_alter_inplace_result(TABLE*, Alter_inplace_info*)` | Check in-place alter support |
| 121 | `prepare_inplace_alter_table` | 6460 | D | `bool(TABLE*, Alter_inplace_info*, const dd::Table*, dd::Table*)` | Prepare in-place alter |
| 122 | `inplace_alter_table` | 6497 | D | `bool(TABLE*, Alter_inplace_info*, const dd::Table*, dd::Table*)` | Execute in-place alter |
| 123 | `commit_inplace_alter_table` | 6555 | D | `bool(TABLE*, Alter_inplace_info*, bool, const dd::Table*, dd::Table*)` | Commit in-place alter |
| 124 | `notify_table_changed` | 6585 | D | `void(Alter_inplace_info*)` | Post-alter notification |
| 125 | `set_shared_data` | 3340 | D | `void(const inplace_alter_handler_ctx*)` | Share data between old/new handler |
| 126 | `check_if_incompatible_data` | 6210 | D | `bool(HA_CREATE_INFO*, uint)` | Check data compatibility |

## Table Maintenance

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 127 | `check` | 6770 | D | `int(THD*, HA_CHECK_OPT*)` | CHECK TABLE |
| 128 | `repair` | 6777 | D | `int(THD*, HA_CHECK_OPT*)` | REPAIR TABLE |
| 129 | `optimize` | 6963 | D | `int(THD*, HA_CHECK_OPT*)` | OPTIMIZE TABLE |
| 130 | `analyze` | 6966 | D | `int(THD*, HA_CHECK_OPT*)` | ANALYZE TABLE |
| 131 | `check_and_repair` | 6981 | D | `bool(THD*)` | Auto check and repair |
| 132 | `check_for_upgrade` | 6769 | D | `int(HA_CHECK_OPT*)` | Check for upgrade needs |
| 133 | `assign_to_keycache` | 5963 | D | `int(THD*, HA_CHECK_OPT*)` | Assign to key cache |
| 134 | `preload_keys` | 5966 | D | `int(THD*, HA_CHECK_OPT*)` | Preload keys into cache |
| 135 | `disable_indexes` | 6992 | D | `int(uint)` | Disable indexes |
| 136 | `enable_indexes` | 7005 | D | `int(uint)` | Enable indexes |
| 137 | `discard_or_import_tablespace` | 7024 | D | `int(bool, dd::Table*)` | Discard/import tablespace |

## Pushed Joins & Conditions

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 138 | `cancel_pushed_idx_cond` | 6166 | D | `void()` | Cancel pushed index condition |
| 139 | `number_of_pushed_joins` | 6176 | D | `uint() const` | Count of pushed joins |
| 140 | `tables_in_pushed_join` | 6192 | D | `table_map() const` | Tables in pushed join |

## CREATE INFO & Metadata

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 141 | `update_create_info` | 5961 | D | `void(HA_CREATE_INFO*)` | Update create info from table |
| 142 | `append_create_info` | 5979 | D | `void(String*)` | Append to SHOW CREATE |
| 143 | `use_hidden_primary_key` | 6596 | D | `void()` | Use hidden PK |
| 144 | `set_ha_share_ref` | 7086 | D | `bool(Handler_share**)` | Set shared handler data |
| 145 | `cmp_ref` | 6107 | D | `int(const uchar*, const uchar*) const` | Compare row references |

## External Table Offload

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 146 | `set_external_table_offload_error` | 7187 | D | `void(const char*)` | Set offload error |
| 147 | `external_table_offload_error` | 7193 | D | `void() const` | Report offload error |

## Multi-Valued Index

| # | Method | Line | PV | Signature | Description |
| - | ------ | ---- | -- | --------- | ----------- |
| 148 | `mv_key_capacity` | 7201 | D | `void(uint*, size_t*) const` | Multi-valued key capacity |

## Total: 148 virtual methods

- **Pure virtual (must override)**: 12
  - `table_type`, `table_flags`, `index_flags`
  - `open`, `close`
  - `rnd_init`, `rnd_next`, `rnd_pos`, `position`
  - `info`
  - `store_lock`
- **With default implementation**: 136
