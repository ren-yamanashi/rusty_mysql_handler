<!--
Regenerate this file with `scripts/audit-bind-coverage.sh` (writes in place).
The script preserves the "Notes" column from the previous version of this
file via an internal snapshot, so human-applied annotations survive reruns;
every other column is recomputed.
-->

# API Bind Coverage

Cross-reference between the upstream MySQL 8.4 handler / handlerton
surface (documented in [`handler.md`](handler.md) and
[`handlerton.md`](handlerton.md)) and the bindings under
`mysql-handler/src/` + `mysql-handler/shim/`.

## Columns

- **T / C / S** — presence in trait (T), `rust__*` callback (C), and
  shim override (S). `✓` if found, `✗` if not.
- **Status** — verdict produced by combining the auto T/C/S detection
  with the Notes column. Possible values: `bound`,
  `intentionally unbound` (genuinely unbindable, not a placeholder),
  `deferred` (bind path known, follow-up tracked in the Notes), or
  `needs review` (annotation missing or ambiguous).
- **Bind path** — basenames of the files matched, for navigation.

## handler — 158 bound, 0 deferred, 0 intentionally unbound (158 total)

| Method | handler.h Line | T | C | S | Status | Bind path | Notes |
| ------ | -------------- | - | - | - | ------ | --------- | ----- |
| `open` | 6661 | ✓ | ✓ | ✓ | bound | engine.rs,open_close.rs,binding.cc |  |
| `close` | 6663 | ✓ | ✓ | ✓ | bound | engine.rs,open_close.rs,binding.cc |  |
| `create` | 7052 | ✓ | ✓ | ✓ | bound | engine.rs,open_close.rs,binding.cc |  |
| `delete_table` | 6650 | ✓ | ✓ | ✓ | bound | engine.rs,table_lifecycle.rs,handler_lifecycle.cc |  |
| `rename_table` | 6630 | ✓ | ✓ | ✓ | bound | engine.rs,table_lifecycle.rs,handler_lifecycle.cc |  |
| `drop_table` | 7031 | ✓ | ✓ | ✓ | bound | engine.rs,table_lifecycle.rs,handler_lifecycle.cc |  |
| `truncate` | 6960 | ✓ | ✓ | ✓ | bound | engine.rs,table_lifecycle.rs,handler_lifecycle.cc |  |
| `change_table_ptr` | 5169 | ✓ | ✓ | ✓ | bound | engine.rs,table_lifecycle.rs,handler_lifecycle.cc |  |
| `get_se_private_data` | 7055 | ✗ | ✓ | ✓ | bound | table_lifecycle.rs,handler_lifecycle.cc | Trait renamed `se_private_data` (Rust API drops `get_`); fully bound. |
| `get_extra_columns_and_keys` | 7078 | ✗ | ✓ | ✓ | bound | table_lifecycle.rs,handler_lifecycle.cc | Trait renamed `extra_columns_and_keys`; fully bound. |
| `upgrade_table` | 6806 | ✓ | ✓ | ✓ | bound | engine.rs,table_lifecycle.rs,handler_lifecycle.cc |  |
| `rnd_init` | 6679 | ✓ | ✓ | ✓ | bound | engine.rs,scan.rs,binding.cc |  |
| `rnd_end` | 6680 | ✓ | ✓ | ✓ | bound | engine.rs,scan.rs,binding.cc |  |
| `rnd_next` | 5693 | ✓ | ✓ | ✓ | bound | engine.rs,scan.rs,binding.cc |  |
| `rnd_pos` | 5695 | ✓ | ✓ | ✓ | bound | engine.rs,scan.rs,binding.cc |  |
| `rnd_pos_by_record` | 5705 | ✓ | ✓ | ✓ | bound | engine.rs,scan.rs,binding.cc |  |
| `position` | 5745 | ✓ | ✓ | ✓ | bound | engine.rs,scan.rs,binding.cc |  |
| `index_init` | 6664 | ✓ | ✓ | ✓ | bound | engine.rs,index_basic.rs,handler_index_basic.cc |  |
| `index_end` | 6668 | ✓ | ✓ | ✓ | bound | engine.rs,index_basic.rs,handler_index_basic.cc |  |
| `index_read` | 6878 | ✓ | ✓ | ✓ | bound | engine.rs,index_range.rs,handler_index_range.cc |  |
| `index_read_map` | 5617 | ✓ | ✓ | ✓ | bound | engine.rs,index_basic.rs,handler_index_basic.cc |  |
| `index_read_idx_map` | 5629 | ✓ | ✓ | ✓ | bound | engine.rs,index_range.rs,handler_index_range.cc |  |
| `index_read_last` | 6884 | ✓ | ✓ | ✓ | bound | engine.rs,index_range.rs,handler_index_range.cc |  |
| `index_read_last_map` | 5657 | ✓ | ✓ | ✓ | bound | engine.rs,index_range.rs,handler_index_range.cc |  |
| `index_next` | 5638 | ✓ | ✓ | ✓ | bound | engine.rs,index_basic.rs,handler_index_basic.cc |  |
| `index_prev` | 5641 | ✓ | ✓ | ✓ | bound | engine.rs,index_basic.rs,handler_index_basic.cc |  |
| `index_first` | 5644 | ✓ | ✓ | ✓ | bound | engine.rs,index_basic.rs,handler_index_basic.cc |  |
| `index_last` | 5647 | ✓ | ✓ | ✓ | bound | engine.rs,index_basic.rs,handler_index_basic.cc |  |
| `index_next_same` | 5650 | ✓ | ✓ | ✓ | bound | engine.rs,index_basic.rs,handler_index_basic.cc |  |
| `read_range_first` | 5663 | ✓ | ✓ | ✓ | bound | engine.rs,index_range.rs,handler_index_range.cc |  |
| `read_range_next` | 5666 | ✓ | ✓ | ✓ | bound | engine.rs,index_range.rs,handler_index_range.cc |  |
| `records_in_range` | 5734 | ✓ | ✓ | ✓ | bound | engine.rs,index_range.rs,handler_index_range.cc |  |
| `index_read_pushed` | 6200 | ✓ | ✓ | ✓ | bound | engine.rs,index_pushed.rs,handler_index_pushed.cc |  |
| `index_next_pushed` | 6204 | ✓ | ✓ | ✓ | bound | engine.rs,index_pushed.rs,handler_index_pushed.cc |  |
| `write_row` | 6702 | ✓ | ✓ | ✓ | bound | engine.rs,row_operations.rs,handler_row_operations.cc |  |
| `update_row` | 6714 | ✓ | ✓ | ✓ | bound | engine.rs,row_operations.rs,handler_row_operations.cc |  |
| `delete_row` | 6719 | ✓ | ✓ | ✓ | bound | engine.rs,row_operations.rs,handler_row_operations.cc |  |
| `delete_all_rows` | 6925 | ✓ | ✓ | ✓ | bound | engine.rs,row_operations.rs,handler_row_operations.cc |  |
| `start_bulk_insert` | 6781 | ✓ | ✓ | ✓ | bound | engine.rs,bulk_operations.rs,handler_bulk_operations.cc |  |
| `end_bulk_insert` | 6782 | ✓ | ✓ | ✓ | bound | engine.rs,bulk_operations.rs,handler_bulk_operations.cc |  |
| `start_bulk_update` | 5572 | ✓ | ✓ | ✓ | bound | engine.rs,bulk_operations.rs,handler_bulk_operations.cc |  |
| `exec_bulk_update` | 5589 | ✓ | ✓ | ✓ | bound | engine.rs,bulk_operations.rs,handler_bulk_operations.cc |  |
| `end_bulk_update` | 5597 | ✓ | ✓ | ✓ | bound | engine.rs,bulk_operations.rs,handler_bulk_operations.cc |  |
| `bulk_update_row` | 6908 | ✓ | ✓ | ✓ | bound | engine.rs,bulk_operations.rs,handler_bulk_operations.cc |  |
| `start_bulk_delete` | 5577 | ✓ | ✓ | ✓ | bound | engine.rs,bulk_operations.rs,handler_bulk_operations.cc |  |
| `end_bulk_delete` | 5604 | ✓ | ✓ | ✓ | bound | engine.rs,bulk_operations.rs,handler_bulk_operations.cc |  |
| `bulk_load_check` | 5063 | ✓ | ✓ | ✓ | bound | engine.rs,bulk_load.rs,handler_bulk_load.cc |  |
| `bulk_load_available_memory` | 5070 | ✓ | ✓ | ✓ | bound | engine.rs,bulk_load.rs,handler_bulk_load.cc |  |
| `bulk_load_begin` | 5080 | ✓ | ✓ | ✓ | bound | engine.rs,bulk_load.rs,handler_bulk_load.cc |  |
| `bulk_load_execute` | 5094 | ✓ | ✓ | ✓ | bound | engine.rs,bulk_load.rs,handler_bulk_load.cc |  |
| `bulk_load_end` | 5109 | ✓ | ✓ | ✓ | bound | engine.rs,bulk_load.rs,handler_bulk_load.cc |  |
| `load_table` | 6848 | ✓ | ✓ | ✓ | bound | engine.rs,bulk_load.rs,handler_bulk_load.cc |  |
| `unload_table` | 6868 | ✓ | ✓ | ✓ | bound | engine.rs,bulk_load.rs,handler_bulk_load.cc |  |
| `parallel_scan_init` | 4981 | ✓ | ✓ | ✓ | bound | engine.rs,parallel_scan.rs,handler_parallel_scan.cc |  |
| `parallel_scan` | 5046 | ✓ | ✓ | ✓ | bound | engine.rs,parallel_scan.rs,handler_parallel_scan.cc |  |
| `parallel_scan_end` | 5058 | ✓ | ✓ | ✓ | bound | engine.rs,parallel_scan.rs,handler_parallel_scan.cc |  |
| `sample_init` | 6822 | ✓ | ✓ | ✓ | bound | engine.rs,sampling.rs,handler_sampling.cc |  |
| `sample_next` | 6831 | ✓ | ✓ | ✓ | bound | engine.rs,sampling.rs,handler_sampling.cc |  |
| `sample_end` | 6836 | ✓ | ✓ | ✓ | bound | engine.rs,sampling.rs,handler_sampling.cc |  |
| `ft_init` | 5682 | ✓ | ✓ | ✓ | bound | engine.rs,fulltext.rs,handler_fulltext.cc |  |
| `ft_init_ext` | 5683 | ✓ | ✓ | ✓ | bound | engine.rs,fulltext.rs,handler_fulltext.cc |  |
| `ft_init_ext_with_hints` | 5684 | ✓ | ✓ | ✓ | bound | engine.rs,fulltext.rs,handler_fulltext.cc |  |
| `ft_read` | 5697 | ✓ | ✓ | ✓ | bound | engine.rs,fulltext.rs,handler_fulltext.cc |  |
| `multi_range_read_info_const` | 5397 | ✓ | ✓ | ✓ | bound | engine.rs,mrr.rs,handler_mrr.cc |  |
| `multi_range_read_info` | 5400 | ✓ | ✓ | ✓ | bound | engine.rs,mrr.rs,handler_mrr.cc |  |
| `multi_range_read_init` | 5403 | ✓ | ✓ | ✓ | bound | engine.rs,mrr.rs,handler_mrr.cc |  |
| `multi_range_read_next` | 5457 | ✓ | ✓ | ✓ | bound | engine.rs,mrr.rs,handler_mrr.cc |  |
| `table_type` | 5984 | ✓ | ✓ | ✓ | bound | engine.rs,properties.rs,binding.cc |  |
| `table_flags` | 6728 | ✓ | ✓ | ✓ | bound | engine.rs,properties.rs,binding.cc |  |
| `index_flags` | 5986 | ✓ | ✓ | ✓ | bound | engine.rs,properties.rs,binding.cc |  |
| `max_supported_record_length` | 6004 | ✓ | ✓ | ✓ | bound | engine.rs,limits.rs,handler_limits.cc |  |
| `max_supported_keys` | 6005 | ✓ | ✓ | ✓ | bound | engine.rs,limits.rs,handler_limits.cc |  |
| `max_supported_key_parts` | 6006 | ✓ | ✓ | ✓ | bound | engine.rs,limits.rs,handler_limits.cc |  |
| `max_supported_key_length` | 6007 | ✓ | ✓ | ✓ | bound | engine.rs,limits.rs,handler_limits.cc |  |
| `max_supported_key_part_length` | 6008 | ✓ | ✓ | ✓ | bound | engine.rs,limits.rs,handler_limits.cc |  |
| `min_record_length` | 6012 | ✓ | ✓ | ✓ | bound | engine.rs,limits.rs,handler_limits.cc |  |
| `low_byte_first` | 6016 | ✓ | ✓ | ✓ | bound | engine.rs,caps.rs,handler_caps.cc |  |
| `checksum` | 6017 | ✓ | ✓ | ✓ | bound | engine.rs,caps.rs,handler_caps.cc |  |
| `is_crashed` | 6026 | ✓ | ✓ | ✓ | bound | engine.rs,caps.rs,handler_caps.cc |  |
| `auto_repair` | 6035 | ✓ | ✓ | ✓ | bound | engine.rs,caps.rs,handler_caps.cc |  |
| `primary_key_is_clustered` | 6094 | ✓ | ✓ | ✓ | bound | engine.rs,caps.rs,handler_caps.cc |  |
| `get_real_row_type` | 5530 | ✗ | ✗ | ✓ | bound | handler_caps.cc | Trait renamed `real_row_type`; fully bound. |
| `get_default_index_algorithm` | 5543 | ✗ | ✗ | ✓ | bound | handler_caps.cc | Trait renamed `default_index_algorithm`; fully bound. |
| `is_index_algorithm_supported` | 5554 | ✓ | ✓ | ✓ | bound | engine.rs,caps_features.rs,handler_caps.cc |  |
| `extra_rec_buf_length` | 5416 | ✓ | ✓ | ✓ | bound | engine.rs,limits.rs,handler_limits.cc |  |
| `get_memory_buffer_size` | 5313 | ✗ | ✗ | ✓ | bound | handler_limits.cc | Trait renamed `memory_buffer_size`; fully bound. |
| `is_record_buffer_wanted` | 6800 | ✗ | ✗ | ✓ | bound | handler_caps.cc | Trait renamed `record_buffer_wanted` (Rust API drops `is_`); fully bound. |
| `explain_extra` | 4842 | ✓ | ✓ | ✓ | bound | engine.rs,caps_features.rs,handler_caps.cc |  |
| `indexes_are_disabled` | 5978 | ✓ | ✓ | ✓ | bound | engine.rs,caps.rs,handler_caps.cc |  |
| `info` | 5774 | ✓ | ✓ | ✓ | bound | engine.rs,statistics.rs,binding.cc |  |
| `scan_time` | 5183 | ✓ | ✓ | ✓ | bound | engine.rs,cost_time.rs,handler_cost.cc |  |
| `read_time` | 5202 | ✓ | ✓ | ✓ | bound | engine.rs,cost_time.rs,handler_cost.cc |  |
| `index_only_read_time` | 5212 | ✓ | ✓ | ✓ | bound | engine.rs,cost_time.rs,handler_cost.cc |  |
| `table_scan_cost` | 5223 | ✓ | ✓ | ✓ | bound | engine.rs,cost.rs,handler_cost.cc |  |
| `index_scan_cost` | 5245 | ✓ | ✓ | ✓ | bound | engine.rs,cost.rs,handler_cost.cc |  |
| `read_cost` | 5261 | ✓ | ✓ | ✓ | bound | engine.rs,cost.rs,handler_cost.cc |  |
| `page_read_cost` | 5295 | ✓ | ✓ | ✓ | bound | engine.rs,cost_time.rs,handler_cost.cc |  |
| `worst_seek_times` | 5306 | ✓ | ✓ | ✓ | bound | engine.rs,cost_time.rs,handler_cost.cc |  |
| `records` | 5467 | ✓ | ✓ | ✓ | bound | engine.rs,records.rs,handler_records.cc |  |
| `records_from_index` | 5478 | ✓ | ✓ | ✓ | bound | engine.rs,records.rs,handler_records.cc |  |
| `estimate_rows_upper_bound` | 5522 | ✓ | ✓ | ✓ | bound | engine.rs,records.rs,handler_records.cc |  |
| `calculate_key_hash_value` | 5775 | ✓ | ✓ | ✓ | bound | engine.rs,records.rs,handler_records.cc |  |
| `store_lock` | 6083 | ✓ | ✓ | ✓ | bound | engine.rs,locking.rs,binding.cc |  |
| `external_lock` | 6763 | ✓ | ✓ | ✓ | bound | engine.rs,locking.rs,handler_locking.cc |  |
| `lock_count` | 6050 | ✓ | ✓ | ✓ | bound | engine.rs,locking.rs,handler_locking.cc |  |
| `unlock_row` | 5908 | ✓ | ✓ | ✓ | bound | engine.rs,locking.rs,handler_locking.cc |  |
| `start_stmt` | 5923 | ✓ | ✓ | ✓ | bound | engine.rs,locking.rs,handler_locking.cc |  |
| `was_semi_consistent_read` | 5892 | ✓ | ✓ | ✓ | bound | engine.rs,locking.rs,handler_locking.cc |  |
| `try_semi_consistent_read` | 5899 | ✓ | ✓ | ✓ | bound | engine.rs,locking.rs,handler_locking.cc |  |
| `start_read_removal` | 5834 | ✓ | ✓ | ✓ | bound | engine.rs,read_removal_autoinc.rs,handler_read_removal_autoinc.cc |  |
| `end_read_removal` | 5844 | ✓ | ✓ | ✓ | bound | engine.rs,read_removal_autoinc.rs,handler_read_removal_autoinc.cc |  |
| `get_auto_increment` | 5927 | ✓ | ✓ | ✓ | bound | engine.rs,read_removal_autoinc.rs,handler_read_removal_autoinc.cc |  |
| `release_auto_increment` | 6767 | ✓ | ✓ | ✓ | bound | engine.rs,read_removal_autoinc.rs,handler_read_removal_autoinc.cc |  |
| `print_error` | 5135 | ✓ | ✓ | ✓ | bound | engine.rs,error_handling.rs,handler_error_handling.cc |  |
| `get_error_message` | 5136 | ✗ | ✓ | ✓ | bound | error_handling.rs,handler_error_handling.cc | Trait renamed `error_message`; fully bound. |
| `get_foreign_dup_key` | 5156 | ✗ | ✓ | ✓ | bound | error_handling.rs,handler_error_handling.cc | Trait renamed `foreign_dup_key`; fully bound. |
| `is_ignorable_error` | 5437 | ✓ | ✓ | ✓ | bound | engine.rs,error_handling.rs,handler_error_handling.cc |  |
| `is_fatal_error` | 5454 | ✓ | ✓ | ✓ | bound | engine.rs,error_handling.rs,handler_error_handling.cc |  |
| `extra` | 5802 | ✓ | ✓ | ✓ | bound | engine.rs,hints.rs,handler_hints.cc |  |
| `extra_opt` | 5807 | ✓ | ✓ | ✓ | bound | engine.rs,hints.rs,handler_hints.cc |  |
| `reset` | 6727 | ✓ | ✓ | ✓ | bound | engine.rs,hints.rs,handler_hints.cc |  |
| `column_bitmaps_signal` | 5565 | ✓ | ✓ | ✓ | bound | engine.rs,hints.rs,handler_hints.cc |  |
| `init_table_handle_for_HANDLER` | 5980 | ✗ | ✗ | ✓ | bound | handler_hints.cc | Trait renamed `init_table_handle_for_handler` (snake_case); fully bound. |
| `check_if_supported_inplace_alter` | 6378 | ✓ | ✓ | ✓ | bound | engine.rs,inplace_alter.rs,handler_inplace_alter.cc |  |
| `prepare_inplace_alter_table` | 6460 | ✓ | ✓ | ✓ | bound | engine.rs,inplace_alter.rs,handler_inplace_alter.cc |  |
| `inplace_alter_table` | 6497 | ✓ | ✓ | ✓ | bound | engine.rs,inplace_alter.rs,handler_inplace_alter.cc |  |
| `commit_inplace_alter_table` | 6555 | ✓ | ✓ | ✓ | bound | engine.rs,inplace_alter.rs,handler_inplace_alter.cc |  |
| `notify_table_changed` | 6585 | ✓ | ✓ | ✓ | bound | engine.rs,inplace_alter.rs,handler_inplace_alter.cc |  |
| `check_if_incompatible_data` | 6210 | ✓ | ✓ | ✓ | bound | engine.rs,inplace_alter.rs,handler_inplace_alter.cc |  |
| `check` | 6770 | ✓ | ✓ | ✓ | bound | engine.rs,maintenance.rs,handler_maintenance.cc |  |
| `repair` | 6777 | ✓ | ✓ | ✓ | bound | engine.rs,maintenance.rs,handler_maintenance.cc |  |
| `optimize` | 6963 | ✓ | ✓ | ✓ | bound | engine.rs,maintenance.rs,handler_maintenance.cc |  |
| `analyze` | 6966 | ✓ | ✓ | ✓ | bound | engine.rs,maintenance.rs,handler_maintenance.cc |  |
| `check_and_repair` | 6981 | ✓ | ✓ | ✓ | bound | engine.rs,maintenance.rs,handler_maintenance.cc |  |
| `check_for_upgrade` | 6769 | ✓ | ✓ | ✓ | bound | engine.rs,maintenance.rs,handler_maintenance.cc |  |
| `assign_to_keycache` | 5963 | ✓ | ✓ | ✓ | bound | engine.rs,index_admin.rs,handler_index_admin.cc |  |
| `preload_keys` | 5966 | ✓ | ✓ | ✓ | bound | engine.rs,index_admin.rs,handler_index_admin.cc |  |
| `disable_indexes` | 6992 | ✓ | ✓ | ✓ | bound | engine.rs,index_admin.rs,handler_index_admin.cc |  |
| `enable_indexes` | 7005 | ✓ | ✓ | ✓ | bound | engine.rs,index_admin.rs,handler_index_admin.cc |  |
| `discard_or_import_tablespace` | 7024 | ✓ | ✓ | ✓ | bound | engine.rs,index_admin.rs,handler_index_admin.cc |  |
| `cond_push` | 6131 | ✓ | ✓ | ✓ | bound | engine.rs,pushdown.rs,handler_pushdown.cc |  |
| `idx_cond_push` | 6161 | ✓ | ✓ | ✓ | bound | engine.rs,pushdown.rs,handler_pushdown.cc |  |
| `cancel_pushed_idx_cond` | 6166 | ✓ | ✓ | ✓ | bound | engine.rs,pushdown.rs,handler_pushdown.cc |  |
| `hton_supporting_engine_pushdown` | 5826 | ✓ | ✓ | ✓ | bound | engine.rs,pushdown.rs,handler_pushdown.cc |  |
| `number_of_pushed_joins` | 6176 | ✓ | ✓ | ✓ | bound | engine.rs,pushdown.rs,handler_pushdown.cc |  |
| `member_of_pushed_join` | 6182 | ✓ | ✓ | ✓ | bound | engine.rs,pushdown.rs,handler_pushdown.cc |  |
| `parent_of_pushed_join` | 6188 | ✓ | ✓ | ✓ | bound | engine.rs,pushdown.rs,handler_pushdown.cc |  |
| `tables_in_pushed_join` | 6192 | ✓ | ✓ | ✓ | bound | engine.rs,pushdown.rs,handler_pushdown.cc |  |
| `update_create_info` | 5961 | ✓ | ✓ | ✓ | bound | engine.rs,metadata.rs,handler_metadata.cc |  |
| `append_create_info` | 5979 | ✓ | ✓ | ✓ | bound | engine.rs,metadata.rs,handler_metadata.cc |  |
| `use_hidden_primary_key` | 6596 | ✓ | ✓ | ✓ | bound | engine.rs,metadata.rs,handler_metadata.cc |  |
| `set_ha_share_ref` | 7086 | ✓ | ✓ | ✓ | bound | engine.rs,metadata.rs,handler_metadata.cc |  |
| `cmp_ref` | 6107 | ✓ | ✓ | ✓ | bound | engine.rs,metadata.rs,handler_metadata.cc |  |
| `set_external_table_offload_error` | 7187 | ✓ | ✓ | ✓ | bound | engine.rs,misc.rs,handler_misc.cc |  |
| `external_table_offload_error` | 7193 | ✓ | ✓ | ✓ | bound | engine.rs,misc.rs,handler_misc.cc |  |
| `clone` | 4847 | ✗ | ✓ | ✓ | bound | misc.rs,handler_misc.cc | Trait renamed `clone_handler` to avoid clashing with `std::clone::Clone`; fully bound. |
| `mv_key_capacity` | 7201 | ✓ | ✓ | ✓ | bound | engine.rs,misc.rs,handler_misc.cc |  |
| `get_partition_handler` | 7140 | ✓ | ✓ | ✓ | bound | engine.rs,misc.rs,handler_misc.cc |  |

## handlerton — 92 bound, 1 deferred, 0 intentionally unbound (93 total)

| Callback | T | C | S | Status | Bind path | Notes |
| -------- | - | - | - | ------ | --------- | ----- |
| `create` | ✗ | ✗ | ✗ | bound |  | Bound via the shim factory `rusty_create_handler` in `hton_init.cc`; no engine-side override needed. |
| `close_connection` | ✓ | ✓ | ✓ | bound | hton.rs,lifecycle.rs,hton_lifecycle.cc |  |
| `kill_connection` | ✓ | ✓ | ✓ | bound | hton.rs,lifecycle.rs,hton_lifecycle.cc |  |
| `pre_dd_shutdown` | ✓ | ✓ | ✓ | bound | hton.rs,lifecycle.rs,hton_lifecycle.cc |  |
| `reset_plugin_vars` | ✓ | ✓ | ✓ | bound | hton.rs,lifecycle.rs,hton_lifecycle.cc |  |
| `commit` | ✓ | ✗ | ✗ | bound | hton.rs,savepoint_ffi.rs,transaction.rs,txn_context.rs,txn_ffi.rs,txn_row_ffi.rs | Bound via `Transaction::commit` + `rust__hton__txn_commit` (renamed at FFI for txn-context disambiguation). |
| `rollback` | ✓ | ✗ | ✗ | bound | hton.rs,savepoint_ffi.rs,transaction.rs,txn_context.rs,txn_ffi.rs,txn_row_ffi.rs | Bound via `Transaction::rollback` + `rust__hton__txn_rollback`. |
| `prepare` | ✓ | ✗ | ✗ | bound | transaction.rs | Bound via `Transaction::prepare` + `rust__hton__txn_prepare`. |
| `recover` | ✓ | ✓ | ✓ | bound | hton.rs,xa.rs,hton_xa.cc |  |
| `recover_prepared_in_tc` | ✓ | ✓ | ✓ | bound | hton.rs,xa.rs,hton_xa.cc |  |
| `commit_by_xid` | ✓ | ✓ | ✓ | bound | hton.rs,xa.rs,hton_xa.cc |  |
| `rollback_by_xid` | ✓ | ✓ | ✓ | bound | hton.rs,xa.rs,hton_xa.cc |  |
| `set_prepared_in_tc` | ✓ | ✓ | ✓ | bound | hton.rs,xa.rs,hton_xa.cc |  |
| `set_prepared_in_tc_by_xid` | ✓ | ✓ | ✓ | bound | hton.rs,xa.rs,hton_xa.cc |  |
| `savepoint_set` | ✓ | ✓ | ✓ | bound | savepoint_ffi.rs,transaction.rs,savepoint_ffi.rs,hton_savepoint.cc |  |
| `savepoint_rollback` | ✓ | ✓ | ✓ | bound | savepoint_ffi.rs,transaction.rs,savepoint_ffi.rs,hton_savepoint.cc |  |
| `savepoint_rollback_can_release_mdl` | ✓ | ✗ | ✗ | bound | savepoint_ffi.rs,transaction.rs | Bound via `Transaction::savepoint_rollback_can_release_mdl`, routed through the `txn_*` callback family. |
| `savepoint_release` | ✓ | ✓ | ✓ | bound | transaction.rs,savepoint_ffi.rs,hton_savepoint.cc |  |
| `drop_database` | ✓ | ✓ | ✓ | bound | hton.rs,database.rs,hton_tablespace.cc |  |
| `is_valid_tablespace_name` | ✓ | ✓ | ✓ | bound | hton.rs,tablespace.rs,hton_tablespace.cc |  |
| `get_tablespace` | ✓ | ✓ | ✓ | bound | hton.rs,tablespace.rs,hton_tablespace.cc |  |
| `alter_tablespace` | ✓ | ✓ | ✓ | bound | hton.rs,tablespace.rs,hton_tablespace.cc |  |
| `get_tablespace_filename_ext` | ✗ | ✗ | ✗ | bound |  | Trait renamed `tablespace_filename_ext` (Rust API drops `get_`); fully bound. |
| `upgrade_tablespace` | ✓ | ✓ | ✓ | bound | hton.rs,tablespace.rs,hton_tablespace.cc |  |
| `upgrade_space_version` | ✓ | ✓ | ✓ | bound | hton.rs,tablespace.rs,hton_tablespace.cc |  |
| `get_tablespace_type` | ✓ | ✓ | ✓ | bound | hton.rs,tablespace.rs,hton_tablespace.cc |  |
| `get_tablespace_type_by_name` | ✓ | ✓ | ✓ | bound | hton.rs,tablespace.rs,hton_tablespace.cc |  |
| `dict_init` | ✓ | ✓ | ✓ | bound | hton.rs,dict.rs,hton_dict.cc |  |
| `ddse_dict_init` | ✓ | ✓ | ✓ | bound | hton.rs,dict.rs,hton_dict.cc |  |
| `dict_register_dd_table_id` | ✓ | ✓ | ✓ | bound | hton.rs,dict.rs,hton_dict.cc |  |
| `dict_cache_reset` | ✓ | ✓ | ✓ | bound | hton.rs,dict.rs,hton_dict.cc |  |
| `dict_cache_reset_tables_and_tablespaces` | ✓ | ✓ | ✓ | bound | hton.rs,dict.rs,hton_dict.cc |  |
| `dict_recover` | ✓ | ✓ | ✓ | bound | hton.rs,dict.rs,hton_dict.cc |  |
| `dict_get_server_version` | ✓ | ✓ | ✓ | bound | hton.rs,dict.rs,hton_dict.cc |  |
| `dict_set_server_version` | ✓ | ✓ | ✓ | bound | hton.rs,dict.rs,hton_dict.cc |  |
| `panic` | ✓ | ✓ | ✓ | bound | hton.rs,status.rs,hton_status.cc |  |
| `start_consistent_snapshot` | ✓ | ✓ | ✓ | bound | hton.rs,status.rs,hton_status.cc |  |
| `flush_logs` | ✓ | ✓ | ✓ | bound | hton.rs,status.rs,hton_status.cc |  |
| `show_status` | ✓ | ✓ | ✓ | bound | hton.rs,status.rs,hton_status.cc |  |
| `partition_flags` | ✓ | ✓ | ✓ | bound | hton.rs,status.rs,hton_status.cc |  |
| `fill_is_table` | ✓ | ✓ | ✓ | bound | hton.rs,status.rs,hton_status.cc |  |
| `upgrade_logs` | ✓ | ✓ | ✓ | bound | hton.rs,status.rs,hton_status.cc |  |
| `finish_upgrade` | ✓ | ✓ | ✓ | bound | hton.rs,status.rs,hton_status.cc |  |
| `is_reserved_db_name` | ✓ | ✓ | ✓ | bound | hton.rs,status.rs,hton_status.cc |  |
| `discover` | ✓ | ✓ | ✓ | bound | hton.rs,discovery.rs,hton_discovery.cc |  |
| `find_files` | ✓ | ✓ | ✓ | bound | hton.rs,discovery.rs,hton_discovery.cc |  |
| `table_exists_in_engine` | ✓ | ✓ | ✓ | bound | hton.rs,discovery.rs,hton_discovery.cc |  |
| `is_supported_system_table` | ✓ | ✓ | ✓ | bound | hton.rs,discovery.rs,hton_discovery.cc |  |
| `binlog_func` | ✓ | ✓ | ✓ | bound | hton.rs,binlog.rs,hton_binlog.cc |  |
| `binlog_log_query` | ✓ | ✓ | ✓ | bound | hton.rs,binlog.rs,hton_binlog.cc |  |
| `acl_notify` | ✓ | ✓ | ✓ | bound | hton.rs,binlog.rs,hton_binlog.cc |  |
| `sdi_create` | ✓ | ✓ | ✓ | bound | hton.rs,sdi.rs,hton_sdi.cc |  |
| `sdi_drop` | ✓ | ✓ | ✓ | bound | hton.rs,sdi.rs,hton_sdi.cc |  |
| `sdi_get_keys` | ✓ | ✓ | ✓ | bound | hton.rs,sdi.rs,hton_sdi.cc |  |
| `sdi_get` | ✓ | ✓ | ✓ | bound | hton.rs,sdi.rs,hton_sdi.cc |  |
| `sdi_set` | ✓ | ✓ | ✓ | bound | hton.rs,sdi.rs,hton_sdi.cc |  |
| `sdi_delete` | ✓ | ✓ | ✓ | bound | hton.rs,sdi.rs,hton_sdi.cc |  |
| `lock_hton_log` | ✓ | ✓ | ✓ | bound | hton.rs,engine_log.rs,hton_engine_log.cc |  |
| `unlock_hton_log` | ✓ | ✓ | ✓ | bound | hton.rs,engine_log.rs,hton_engine_log.cc |  |
| `collect_hton_log_info` | ✓ | ✓ | ✓ | bound | hton.rs,engine_log.rs,hton_engine_log.cc |  |
| `check_fk_column_compat` | ✓ | ✓ | ✓ | bound | hton.rs,fk_hooks.rs,hton_fk_hooks.cc |  |
| `prepare_secondary_engine` | ✓ | ✓ | ✓ | bound | hton.rs,secondary_engine.rs,hton_secondary_engine.cc |  |
| `optimize_secondary_engine` | ✓ | ✓ | ✓ | bound | hton.rs,secondary_engine.rs,hton_secondary_engine.cc |  |
| `compare_secondary_engine_cost` | ✓ | ✓ | ✓ | bound | hton.rs,secondary_engine.rs,hton_secondary_engine.cc |  |
| `external_engine_explain_check` | ✓ | ✓ | ✓ | bound | hton.rs,secondary_engine.rs,hton_secondary_engine.cc |  |
| `secondary_engine_modify_access_path_cost` | ✓ | ✓ | ✓ | bound | hton.rs,secondary_engine.rs,hton_secondary_engine.cc |  |
| `get_secondary_engine_offload_or_exec_fail_reason` | ✗ | ✓ | ✓ | bound | secondary_engine_fail_reason.rs,hton_secondary_engine.cc | FFI-only binding: engine-side hook is exposed through the secondary-engine fail-reason FFI buffer, not a trait method. |
| `find_secondary_engine_offload_fail_reason` | ✗ | ✓ | ✓ | bound | secondary_engine_fail_reason.rs,hton_secondary_engine.cc | FFI-only binding: same fail-reason buffer path as the `get_*` variant above. |
| `set_secondary_engine_offload_fail_reason` | ✓ | ✓ | ✓ | bound | hton.rs,secondary_engine_fail_reason.rs,hton_secondary_engine.cc |  |
| `secondary_engine_check_optimizer_request` | ✓ | ✓ | ✓ | bound | hton.rs,secondary_engine.rs,hton_secondary_engine.cc |  |
| `secondary_engine_pre_prepare_hook` | ✓ | ✓ | ✓ | bound | hton.rs,secondary_engine.rs,hton_secondary_engine.cc |  |
| `se_before_commit` | ✓ | ✓ | ✓ | bound | hton.rs,fk_hooks.rs,hton_fk_hooks.cc |  |
| `se_after_commit` | ✓ | ✓ | ✓ | bound | hton.rs,fk_hooks.rs,hton_fk_hooks.cc |  |
| `se_before_rollback` | ✓ | ✓ | ✓ | bound | hton.rs,fk_hooks.rs,hton_fk_hooks.cc |  |
| `notify_after_select` | ✓ | ✓ | ✓ | bound | hton.rs,notifications.rs,hton_notifications.cc |  |
| `notify_create_table` | ✓ | ✓ | ✓ | bound | hton.rs,notifications.rs,hton_notifications.cc |  |
| `notify_drop_table` | ✓ | ✓ | ✓ | bound | hton.rs,notifications.rs,hton_notifications.cc |  |
| `push_to_engine` | ✓ | ✓ | ✓ | bound | hton.rs,misc_optimizer.rs,hton_misc.cc |  |
| `is_dict_readonly` | ✓ | ✓ | ✓ | bound | hton.rs,misc_optimizer.rs,hton_misc.cc |  |
| `rm_tmp_tables` | ✓ | ✓ | ✓ | bound | hton.rs,misc_optimizer.rs,hton_misc.cc |  |
| `get_cost_constants` | ✓ | ✓ | ✓ | bound | hton.rs,misc_optimizer.rs,hton_misc.cc |  |
| `replace_native_transaction_in_thd` | ✓ | ✓ | ✓ | bound | hton.rs,misc_optimizer.rs,hton_misc.cc |  |
| `notify_exclusive_mdl` | ✓ | ✓ | ✓ | bound | hton.rs,notifications.rs,hton_notifications.cc |  |
| `notify_alter_table` | ✓ | ✓ | ✓ | bound | hton.rs,notifications.rs,hton_notifications.cc |  |
| `notify_rename_table` | ✓ | ✓ | ✓ | bound | hton.rs,notifications.rs,hton_notifications.cc |  |
| `notify_truncate_table` | ✓ | ✓ | ✓ | bound | hton.rs,notifications.rs,hton_notifications.cc |  |
| `rotate_encryption_master_key` | ✓ | ✓ | ✓ | bound | hton.rs,misc_stats.rs,hton_misc.cc |  |
| `redo_log_set_state` | ✓ | ✓ | ✓ | bound | hton.rs,misc_stats.rs,hton_misc.cc |  |
| `get_table_statistics` | ✓ | ✓ | ✓ | bound | hton.rs,statistics_callbacks.rs,hton_misc.cc |  |
| `get_index_column_cardinality` | ✓ | ✓ | ✓ | bound | hton.rs,statistics_callbacks.rs,hton_misc.cc |  |
| `get_tablespace_statistics` | ✓ | ✓ | ✗ | deferred | hton.rs,statistics_callbacks.rs | Deferred: needs a setter reverse callback to populate `ha_tablespace_statistics` from the engine. Follow-up: p9-09. |
| `post_ddl` | ✓ | ✓ | ✓ | bound | hton.rs,misc_stats.rs,hton_misc.cc |  |
| `post_recover` | ✓ | ✓ | ✓ | bound | hton.rs,misc_stats.rs,hton_misc.cc |  |

