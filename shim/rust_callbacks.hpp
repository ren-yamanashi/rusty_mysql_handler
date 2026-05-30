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

#ifndef SHIM_RUST_CALLBACKS_HPP
#define SHIM_RUST_CALLBACKS_HPP

// Umbrella for the `rust__*` callback declarations, split by handler-API
// category to keep each header focused and under the source-file size limit.
// Every shim translation unit includes this header to see the full surface;
// the per-category headers under rust_callbacks/ map one-to-one to the
// handler_*.cc files (and to the Rust callback modules under src/handler/).
#include "rust_callbacks/bulk_load.hpp"
#include "rust_callbacks/bulk_operations.hpp"
#include "rust_callbacks/caps.hpp"
#include "rust_callbacks/core.hpp"
#include "rust_callbacks/cost.hpp"
#include "rust_callbacks/error_handling.hpp"
#include "rust_callbacks/fulltext.hpp"
#include "rust_callbacks/hints.hpp"
#include "rust_callbacks/hton_binlog.hpp"
#include "rust_callbacks/hton_clone.hpp"
#include "rust_callbacks/hton_core.hpp"
#include "rust_callbacks/hton_dict.hpp"
#include "rust_callbacks/hton_discovery.hpp"
#include "rust_callbacks/hton_engine_log.hpp"
#include "rust_callbacks/hton_fk_hooks.hpp"
#include "rust_callbacks/hton_lifecycle.hpp"
#include "rust_callbacks/hton_misc.hpp"
#include "rust_callbacks/hton_notifications.hpp"
#include "rust_callbacks/hton_page_track.hpp"
#include "rust_callbacks/hton_savepoint.hpp"
#include "rust_callbacks/hton_sdi.hpp"
#include "rust_callbacks/hton_secondary_engine.hpp"
#include "rust_callbacks/hton_status.hpp"
#include "rust_callbacks/hton_tablespace.hpp"
#include "rust_callbacks/hton_transactions.hpp"
#include "rust_callbacks/hton_xa.hpp"
#include "rust_callbacks/index_admin.hpp"
#include "rust_callbacks/index_basic.hpp"
#include "rust_callbacks/index_pushed.hpp"
#include "rust_callbacks/index_range.hpp"
#include "rust_callbacks/inplace_alter.hpp"
#include "rust_callbacks/lifecycle.hpp"
#include "rust_callbacks/limits.hpp"
#include "rust_callbacks/locking.hpp"
#include "rust_callbacks/maintenance.hpp"
#include "rust_callbacks/metadata.hpp"
#include "rust_callbacks/misc.hpp"
#include "rust_callbacks/mrr.hpp"
#include "rust_callbacks/parallel_scan.hpp"
#include "rust_callbacks/properties.hpp"
#include "rust_callbacks/pushdown.hpp"
#include "rust_callbacks/read_removal_autoinc.hpp"
#include "rust_callbacks/records.hpp"
#include "rust_callbacks/row_operations.hpp"
#include "rust_callbacks/sampling.hpp"

#endif
