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

#ifndef SHIM_BINDING_HPP
#define SHIM_BINDING_HPP

#include <string>

#include "my_base.h"
#include "sql/handler.h"
#include "thr_lock.h"

class RustyShare : public Handler_share {
 public:
  THR_LOCK lock;
  RustyShare();
  ~RustyShare() override;
};

class RustHandlerBase : public handler {
  THR_LOCK_DATA lock_data_;
  RustyShare *share_ = nullptr;
  // Non-null once open() succeeds; row-op overrides rely on this invariant
  // and pass it to the Rust callbacks without a per-call null guard
  void *rust_ctx_ = nullptr;

  RustyShare *get_share();

 public:
  RustHandlerBase(handlerton *hton, TABLE_SHARE *table_arg);
  ~RustHandlerBase() override;

  const char *table_type() const override;
  Table_flags table_flags() const override;
  ulong index_flags(uint idx, uint part, bool all_parts) const override;

  int open(const char *name, int mode, uint test_if_locked,
           const dd::Table *table_def) override;
  int close() override;
  int create(const char *name, TABLE *form, HA_CREATE_INFO *create_info,
             dd::Table *table_def) override;

  int rnd_init(bool scan) override;
  int rnd_end() override;
  int rnd_next(uchar *buf) override;
  int rnd_pos(uchar *buf, uchar *pos) override;
  void position(const uchar *record) override;
  int rnd_pos_by_record(uchar *record) override;

  int info(uint flag) override;
  THR_LOCK_DATA **store_lock(THD *thd, THR_LOCK_DATA **to,
                             enum thr_lock_type lock_type) override;

  int delete_table(const char *name, const dd::Table *table_def) override;
  int rename_table(const char *from, const char *to,
                   const dd::Table *from_table_def,
                   dd::Table *to_table_def) override;
  void drop_table(const char *name) override;
  int truncate(dd::Table *table_def) override;
  void change_table_ptr(TABLE *table_arg, TABLE_SHARE *share) override;
  bool get_se_private_data(dd::Table *dd_table, bool reset) override;
  int get_extra_columns_and_keys(const HA_CREATE_INFO *create_info,
                                 const List<Create_field> *create_list,
                                 const KEY *key_info, uint key_count,
                                 dd::Table *table_obj) override;
  bool upgrade_table(THD *thd, const char *dbname, const char *table_name,
                     dd::Table *dd_table) override;

  int index_init(uint idx, bool sorted) override;
  int index_end() override;
  int index_read_map(uchar *buf, const uchar *key, key_part_map keypart_map,
                     enum ha_rkey_function find_flag) override;
  int index_next(uchar *buf) override;
  int index_prev(uchar *buf) override;
  int index_first(uchar *buf) override;
  int index_last(uchar *buf) override;
  int index_next_same(uchar *buf, const uchar *key, uint keylen) override;
  int index_read_pushed(uchar *buf, const uchar *key,
                        key_part_map keypart_map) override;
  int index_next_pushed(uchar *buf) override;

  int index_read(uchar *buf, const uchar *key, uint key_len,
                 enum ha_rkey_function find_flag) override;
  int index_read_idx_map(uchar *buf, uint index, const uchar *key,
                         key_part_map keypart_map,
                         enum ha_rkey_function find_flag) override;
  int index_read_last(uchar *buf, const uchar *key, uint key_len) override;
  int index_read_last_map(uchar *buf, const uchar *key,
                          key_part_map keypart_map) override;
  int read_range_first(const key_range *start_key, const key_range *end_key,
                       bool eq_range, bool sorted) override;
  int read_range_next() override;
  ha_rows records_in_range(uint inx, key_range *min_key,
                           key_range *max_key) override;

  int write_row(uchar *buf) override;
  int update_row(const uchar *old_data, uchar *new_data) override;
  int delete_row(const uchar *buf) override;
  int delete_all_rows() override;

  void start_bulk_insert(ha_rows rows) override;
  int end_bulk_insert() override;
  bool start_bulk_update() override;
  int exec_bulk_update(uint *dup_key_found) override;
  void end_bulk_update() override;
  int bulk_update_row(const uchar *old_data, uchar *new_data,
                      uint *dup_key_found) override;
  bool start_bulk_delete() override;
  int end_bulk_delete() override;

  bool bulk_load_check(THD *thd) const override;
  size_t bulk_load_available_memory(THD *thd) const override;
  void *bulk_load_begin(THD *thd, size_t data_size, size_t memory,
                        size_t num_threads) override;
  int bulk_load_execute(THD *thd, void *load_ctx, size_t thread_idx,
                        const Rows_mysql &rows,
                        Bulk_load::Stat_callbacks &wait_cbk) override;
  int bulk_load_end(THD *thd, void *load_ctx, bool is_error) override;
  int load_table(const TABLE &table, bool *skip_metadata_update) override;
  int unload_table(const char *db_name, const char *table_name,
                   bool error_if_not_loaded) override;

  int parallel_scan_init(void *&scan_ctx, size_t *num_threads,
                         bool use_reserved_threads,
                         size_t max_desired_threads) override;
  int parallel_scan(void *scan_ctx, void **thread_ctxs, Load_init_cbk init_fn,
                    Load_cbk load_fn, Load_end_cbk end_fn) override;
  void parallel_scan_end(void *scan_ctx) override;
  int sample_init(void *&scan_ctx, double sampling_percentage,
                  int sampling_seed, enum_sampling_method sampling_method,
                  const bool tablesample) override;
  int sample_next(void *scan_ctx, uchar *buf) override;
  int sample_end(void *scan_ctx) override;

  int ft_init() override;
  FT_INFO *ft_init_ext(uint flags, uint inx, String *key) override;
  FT_INFO *ft_init_ext_with_hints(uint inx, String *key,
                                  Ft_hints *hints) override;
  int ft_read(uchar *buf) override;

  ha_rows multi_range_read_info_const(uint keyno, RANGE_SEQ_IF *seq,
                                      void *seq_init_param, uint n_ranges,
                                      uint *bufsz, uint *flags,
                                      bool *force_default_mrr,
                                      Cost_estimate *cost) override;
  ha_rows multi_range_read_info(uint keyno, uint n_ranges, uint keys,
                                uint *bufsz, uint *flags,
                                Cost_estimate *cost) override;
  int multi_range_read_init(RANGE_SEQ_IF *seq, void *seq_init_param,
                            uint n_ranges, uint mode,
                            HANDLER_BUFFER *buf) override;
  int multi_range_read_next(char **range_info) override;

  uint max_supported_record_length() const override;
  uint max_supported_keys() const override;
  uint max_supported_key_parts() const override;
  uint max_supported_key_length() const override;
  uint max_supported_key_part_length(
      HA_CREATE_INFO *create_info) const override;
  uint min_record_length(uint options) const override;
  uint extra_rec_buf_length() const override;
  longlong get_memory_buffer_size() const override;

  bool low_byte_first() const override;
  ha_checksum checksum() const override;
  bool is_crashed() const override;
  bool auto_repair() const override;
  bool primary_key_is_clustered() const override;
  enum row_type get_real_row_type(
      const HA_CREATE_INFO *create_info) const override;
  enum ha_key_alg get_default_index_algorithm() const override;
  bool is_index_algorithm_supported(enum ha_key_alg key_alg) const override;
  bool is_record_buffer_wanted(ha_rows *max_rows) const override;
  std::string explain_extra() const override;
  int indexes_are_disabled() override;

  double scan_time() override;
  double read_time(uint index, uint ranges, ha_rows rows) override;
  double index_only_read_time(uint keynr, double records) override;
  Cost_estimate table_scan_cost() override;
  Cost_estimate index_scan_cost(uint index, double ranges, double rows) override;
  Cost_estimate read_cost(uint index, double ranges, double rows) override;
  double page_read_cost(uint index, double reads) override;
  double worst_seek_times(double reads) override;

  int records(ha_rows *num_rows) override;
  int records_from_index(ha_rows *num_rows, uint index) override;
  ha_rows estimate_rows_upper_bound() override;
  uint32 calculate_key_hash_value(Field **field_array) override;

  int external_lock(THD *thd, int lock_type) override;
  uint lock_count() const override;
  void unlock_row() override;
  int start_stmt(THD *thd, thr_lock_type lock_type) override;
  bool was_semi_consistent_read() override;
  void try_semi_consistent_read(bool yes) override;
  bool start_read_removal() override;
  ha_rows end_read_removal() override;
  void get_auto_increment(ulonglong offset, ulonglong increment,
                          ulonglong nb_desired_values, ulonglong *first_value,
                          ulonglong *nb_reserved_values) override;
  void release_auto_increment() override;

  void print_error(int error, myf errflag) override;
  bool get_error_message(int error, String *buf) override;
  bool get_foreign_dup_key(char *child_table_name, uint child_table_name_len,
                           char *child_key_name,
                           uint child_key_name_len) override;
  bool is_ignorable_error(int error) override;
  bool is_fatal_error(int error) override;
  int extra(enum ha_extra_function operation) override;
  int extra_opt(enum ha_extra_function operation, ulong cache_size) override;
  int reset() override;
  void column_bitmaps_signal() override;
  void init_table_handle_for_HANDLER() override;

  enum_alter_inplace_result check_if_supported_inplace_alter(
      TABLE *altered_table, Alter_inplace_info *ha_alter_info) override;
  bool prepare_inplace_alter_table(TABLE *altered_table,
                                   Alter_inplace_info *ha_alter_info,
                                   const dd::Table *old_table_def,
                                   dd::Table *new_table_def) override;
  bool inplace_alter_table(TABLE *altered_table,
                           Alter_inplace_info *ha_alter_info,
                           const dd::Table *old_table_def,
                           dd::Table *new_table_def) override;
  bool commit_inplace_alter_table(TABLE *altered_table,
                                  Alter_inplace_info *ha_alter_info, bool commit,
                                  const dd::Table *old_table_def,
                                  dd::Table *new_table_def) override;
  void notify_table_changed(Alter_inplace_info *ha_alter_info) override;
  bool check_if_incompatible_data(HA_CREATE_INFO *create_info,
                                  uint table_changes) override;

  int check(THD *thd, HA_CHECK_OPT *check_opt) override;
  int repair(THD *thd, HA_CHECK_OPT *check_opt) override;
  int optimize(THD *thd, HA_CHECK_OPT *check_opt) override;
  int analyze(THD *thd, HA_CHECK_OPT *check_opt) override;
  bool check_and_repair(THD *thd) override;
  int check_for_upgrade(HA_CHECK_OPT *check_opt) override;
  int assign_to_keycache(THD *thd, HA_CHECK_OPT *check_opt) override;
  int preload_keys(THD *thd, HA_CHECK_OPT *check_opt) override;
  int disable_indexes(uint mode) override;
  int enable_indexes(uint mode) override;
  int discard_or_import_tablespace(bool discard,
                                   dd::Table *table_def) override;

  const Item *cond_push(const Item *cond) override;
  Item *idx_cond_push(uint keyno, Item *idx_cond) override;
  void cancel_pushed_idx_cond() override;
  const handlerton *hton_supporting_engine_pushdown() override;
  uint number_of_pushed_joins() const override;
  const TABLE *member_of_pushed_join() const override;
  const TABLE *parent_of_pushed_join() const override;
  table_map tables_in_pushed_join() const override;

  void update_create_info(HA_CREATE_INFO *create_info) override;
  void append_create_info(String *packet) override;
  void use_hidden_primary_key() override;
  bool set_ha_share_ref(Handler_share **arg_ha_share) override;
  int cmp_ref(const uchar *ref1, const uchar *ref2) const override;
  void set_external_table_offload_error(const char *reason) override;
  void external_table_offload_error() const override;
  handler *clone(const char *name, MEM_ROOT *mem_root) override;
  void mv_key_capacity(uint *num_keys, size_t *keys_length) const override;
  Partition_handler *get_partition_handler() override;
};

// C linkage so the Rust-side plugin manifest in examples/engine/src/lib.rs
// can refer to these by their unmangled names via `extern "C"`.
extern "C" {
int rusty_init_func(void *p);
int rusty_deinit_func(void *p);
}

// Wires the always-on engine-lifecycle hooks (close_connection, kill_connection,
// pre_dd_shutdown, reset_plugin_vars) onto the handlerton. Called from
// rusty_init_func only when a Rust Handlerton is registered; internal to the
// shim, so C++ linkage.
void rusty_hton_wire_lifecycle(handlerton *hton);

// Wires the transaction callbacks (commit, rollback, prepare) onto the
// handlerton. Called from rusty_init_func only when the handlerton declares the
// TRANSACTIONS capability.
void rusty_hton_wire_transactions(handlerton *hton);

// Allocates the per-connection transaction context (if absent) and registers
// the engine in the current statement / transaction. Called from the handler's
// external_lock / start_stmt for a transactional engine.
void rusty_hton_register_txn(THD *thd, handlerton *ht);

// Wires the XA recovery callbacks (commit_by_xid, rollback_by_xid,
// set_prepared_in_tc, set_prepared_in_tc_by_xid) onto the handlerton. Called
// from rusty_init_func only when the handlerton declares the XA capability;
// recover / recover_prepared_in_tc stay NULL.
void rusty_hton_wire_xa(handlerton *hton);

// Wires the savepoint callbacks and sets savepoint_offset. Called from
// rusty_init_func only when the handlerton declares the SAVEPOINTS capability.
void rusty_hton_wire_savepoints(handlerton *hton);

// Wires the always-on status / lifecycle hooks (panic, flush_logs, show_status,
// fill_is_table, upgrade_logs, finish_upgrade, is_reserved_db_name). Called
// from rusty_init_func only when a Rust Handlerton is registered.
void rusty_hton_wire_status(handlerton *hton);

// Wires start_consistent_snapshot. Called from rusty_init_func only when the
// handlerton declares the TRANSACTIONS capability.
void rusty_hton_wire_consistent_snapshot(handlerton *hton);

// Wires partition_flags. Called from rusty_init_func only when the handlerton
// declares the PARTITIONING capability.
void rusty_hton_wire_partitioning(handlerton *hton);

// Wires table-discovery callbacks (discover, find_files, table_exists_in_engine,
// is_supported_system_table). Called from rusty_init_func only when a Rust
// Handlerton is registered.
void rusty_hton_wire_discovery(handlerton *hton);

// Wires the DDL / select event-notification callbacks (notify_after_select,
// notify_create_table, notify_drop_table, notify_exclusive_mdl,
// notify_alter_table, notify_rename_table, notify_truncate_table). Always
// wired on a registered handlerton.
void rusty_hton_wire_notifications(handlerton *hton);

// Wires binlog / ACL notification callbacks (binlog_func, binlog_log_query,
// acl_notify). Always wired on a registered handlerton.
void rusty_hton_wire_binlog(handlerton *hton);

// Wires drop_database — always wired on a registered handlerton.
void rusty_hton_wire_drop_database(handlerton *hton);

// Wires the tablespace callbacks (is_valid_tablespace_name, get_tablespace,
// alter_tablespace, get_tablespace_filename_ext, upgrade_tablespace,
// upgrade_space_version, get_tablespace_type{,_by_name}). Wired only when the
// handlerton declares the TABLESPACES capability.
void rusty_hton_wire_tablespaces(handlerton *hton);

// Wires the data-dictionary backend callbacks (dict_init, ddse_dict_init,
// dict_register_dd_table_id, dict_cache_reset{,_tables_and_tablespaces},
// dict_recover, dict_{get,set}_server_version). Wired only when the
// handlerton declares the DICT_BACKEND capability.
void rusty_hton_wire_dict(handlerton *hton);

// Wires the SDI callbacks (sdi_create, sdi_drop, sdi_get_keys, sdi_get,
// sdi_set, sdi_delete). Wired only when the handlerton declares the SDI
// capability.
void rusty_hton_wire_sdi(handlerton *hton);

// Wires the engine-log callbacks (lock_hton_log, unlock_hton_log,
// collect_hton_log_info) consumed by performance_schema.log_status. Wired
// only when the handlerton declares the ENGINE_LOG capability.
void rusty_hton_wire_engine_log(handlerton *hton);

#endif
