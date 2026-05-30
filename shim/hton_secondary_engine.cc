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

// Secondary-engine handlerton callbacks (handler.h #62-#71). Wired only
// under HtonCapabilities::SECONDARY_ENGINE. The `std::string_view` returns
// are reassembled from `(ptr, len)` pairs the Rust side produces, and the
// SecondaryEngineGraphSimplificationRequestParameters struct is reassembled
// from two integer out-params. Crossed opaque types: LEX, JOIN,
// JoinHypergraph, AccessPath.

#include <string_view>

#include "binding.hpp"
#include "rust_callbacks.hpp"
#include "sql/handler.h"

namespace {
bool rusty_hton_prepare_secondary_engine(THD *thd, LEX *lex) {
  return rust__hton__prepare_secondary_engine(static_cast<const void *>(thd),
                                              static_cast<const void *>(lex));
}

bool rusty_hton_optimize_secondary_engine(THD *thd, LEX *lex) {
  return rust__hton__optimize_secondary_engine(static_cast<const void *>(thd),
                                               static_cast<const void *>(lex));
}

bool rusty_hton_compare_secondary_engine_cost(THD *thd, const JOIN &join,
                                              double optimizer_cost,
                                              bool *use_best_so_far,
                                              bool *cheaper,
                                              double *secondary_engine_cost) {
  return rust__hton__compare_secondary_engine_cost(
      static_cast<const void *>(thd), static_cast<const void *>(&join),
      optimizer_cost, use_best_so_far, cheaper, secondary_engine_cost);
}

bool rusty_hton_secondary_engine_modify_access_path_cost(
    THD *thd, const JoinHypergraph &hypergraph, AccessPath *access_path) {
  return rust__hton__secondary_engine_modify_access_path_cost(
      static_cast<const void *>(thd),
      static_cast<const void *>(&hypergraph),
      static_cast<const void *>(access_path));
}

bool rusty_hton_external_engine_explain_check(THD *thd) {
  return rust__hton__external_engine_explain_check(
      static_cast<const void *>(thd));
}

// The Rust side hands back `(ptr, len)`; we wrap them in a string_view. The
// memory backing the bytes is owned by the Rust trait return for the duration
// of this call only — MySQL is documented to copy or discard the view before
// the statement completes, which is sooner than the next call returns.
std::string_view rusty_hton_get_secondary_engine_offload_or_exec_fail_reason(
    const THD *thd) {
  const uint8_t *p = nullptr;
  size_t len = 0;
  rust__hton__get_secondary_engine_offload_or_exec_fail_reason(
      static_cast<const void *>(thd), &p, &len);
  return std::string_view(reinterpret_cast<const char *>(p), len);
}

std::string_view rusty_hton_find_secondary_engine_offload_fail_reason(
    THD *thd) {
  const uint8_t *p = nullptr;
  size_t len = 0;
  rust__hton__find_secondary_engine_offload_fail_reason(
      static_cast<const void *>(thd), &p, &len);
  return std::string_view(reinterpret_cast<const char *>(p), len);
}

bool rusty_hton_set_secondary_engine_offload_fail_reason(
    const THD *thd, std::string_view reason) {
  return rust__hton__set_secondary_engine_offload_fail_reason(
      static_cast<const void *>(thd),
      reinterpret_cast<const uint8_t *>(reason.data()), reason.size());
}

// Reconstruct the C++ struct from the two integer out-params the Rust side
// writes back. The C++ enum is `int` underneath.
SecondaryEngineGraphSimplificationRequestParameters
rusty_hton_secondary_engine_check_optimizer_request(
    THD *thd, const JoinHypergraph &hypergraph, const AccessPath *access_path,
    int current_subgraph_pairs, int current_subgraph_pairs_limit,
    bool is_root_access_path, std::string * /* trace */) {
  int32_t request_raw = 0;
  int32_t subgraph_pair_limit = 0;
  rust__hton__secondary_engine_check_optimizer_request(
      static_cast<const void *>(thd), static_cast<const void *>(&hypergraph),
      static_cast<const void *>(access_path), current_subgraph_pairs,
      current_subgraph_pairs_limit, is_root_access_path, &request_raw,
      &subgraph_pair_limit);
  return SecondaryEngineGraphSimplificationRequestParameters{
      static_cast<SecondaryEngineGraphSimplificationRequest>(request_raw),
      subgraph_pair_limit};
}

bool rusty_hton_secondary_engine_pre_prepare_hook(THD *thd) {
  return rust__hton__secondary_engine_pre_prepare_hook(
      static_cast<const void *>(thd));
}
}  // namespace

void rusty_hton_wire_secondary_engine(handlerton *hton) {
  hton->prepare_secondary_engine = rusty_hton_prepare_secondary_engine;
  hton->optimize_secondary_engine = rusty_hton_optimize_secondary_engine;
  hton->compare_secondary_engine_cost = rusty_hton_compare_secondary_engine_cost;
  hton->secondary_engine_modify_access_path_cost =
      rusty_hton_secondary_engine_modify_access_path_cost;
  hton->external_engine_explain_check = rusty_hton_external_engine_explain_check;
  hton->get_secondary_engine_offload_or_exec_fail_reason =
      rusty_hton_get_secondary_engine_offload_or_exec_fail_reason;
  hton->find_secondary_engine_offload_fail_reason =
      rusty_hton_find_secondary_engine_offload_fail_reason;
  hton->set_secondary_engine_offload_fail_reason =
      rusty_hton_set_secondary_engine_offload_fail_reason;
  hton->secondary_engine_check_optimizer_request =
      rusty_hton_secondary_engine_check_optimizer_request;
  hton->secondary_engine_pre_prepare_hook =
      rusty_hton_secondary_engine_pre_prepare_hook;
}
