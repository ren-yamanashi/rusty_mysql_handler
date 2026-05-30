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

#ifndef SHIM_RUST_CALLBACKS_HTON_SECONDARY_ENGINE_HPP
#define SHIM_RUST_CALLBACKS_HTON_SECONDARY_ENGINE_HPP

#include <cstddef>
#include <cstdint>

// Engine-level secondary-engine callbacks. Wired only under
// HtonCapabilities::SECONDARY_ENGINE. THD / LEX / JOIN / JoinHypergraph /
// AccessPath all cross as opaque `const void *`. `std::string_view` outputs
// are flattened to `(ptr, len)` out-params; the C++ side reconstructs the
// `std::string_view` and the C++ side also reconstructs the
// SecondaryEngineGraphSimplificationRequestParameters struct from the two
// integer out-params.
extern "C" {
bool rust__hton__prepare_secondary_engine(const void *thd, const void *lex);
bool rust__hton__optimize_secondary_engine(const void *thd, const void *lex);
bool rust__hton__compare_secondary_engine_cost(const void *thd,
                                               const void *join,
                                               double optimizer_cost,
                                               bool *use_best_so_far,
                                               bool *cheaper,
                                               double *secondary_engine_cost);
bool rust__hton__secondary_engine_modify_access_path_cost(
    const void *thd, const void *hypergraph, const void *access_path);
bool rust__hton__external_engine_explain_check(const void *thd);
bool rust__hton__get_secondary_engine_offload_or_exec_fail_reason(
    const void *thd, const uint8_t **out_ptr, size_t *out_len);
bool rust__hton__find_secondary_engine_offload_fail_reason(
    const void *thd, const uint8_t **out_ptr, size_t *out_len);
bool rust__hton__set_secondary_engine_offload_fail_reason(
    const void *thd, const uint8_t *reason, size_t reason_len);
void rust__hton__secondary_engine_check_optimizer_request(
    const void *thd, const void *hypergraph, const void *access_path,
    int32_t current_subgraph_pairs, int32_t current_subgraph_pairs_limit,
    bool is_root_access_path, int32_t *out_request,
    int32_t *out_subgraph_pair_limit);
bool rust__hton__secondary_engine_pre_prepare_hook(const void *thd);
}

#endif
