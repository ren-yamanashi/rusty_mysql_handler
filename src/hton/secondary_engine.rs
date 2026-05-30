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

//! `rust__hton__*` secondary-engine callbacks (prepare / optimize / cost /
//! explain / pre-prepare). Wired only under
//! [`HtonCapabilities::SECONDARY_ENGINE`]. The two MySQL callbacks that
//! return / take `std::string_view` live in `secondary_engine_fail_reason`;
//! `compare_secondary_engine_cost` and
//! `secondary_engine_check_optimizer_request` flatten their C struct / triple
//! into pointer out-params.
//!
//! [`HtonCapabilities::SECONDARY_ENGINE`]: crate::hton::HtonCapabilities::SECONDARY_ENGINE

#![allow(unsafe_code)]

use crate::hton::SecondaryEngineOptimizerRequest;
use crate::panic_guard::FfiBoundary;
use crate::runtime;
use crate::sys;

fn result_to_error(r: crate::engine::EngineResult) -> bool {
    match r {
        Ok(()) => false,
        Err(_) => true,
    }
}

/// # Safety
/// `thd` / `lex` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__prepare_secondary_engine(
    thd: *const sys::THD,
    lex: *const sys::Lex,
) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        // SAFETY: lex null or valid for read for this call.
        let lex_ref = unsafe { lex.as_ref() };
        match runtime::handlerton() {
            Some(h) => result_to_error(h.prepare_secondary_engine(thd_ref, lex_ref)),
            None => false,
        }
    })
}

/// # Safety
/// `thd` / `lex` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__optimize_secondary_engine(
    thd: *const sys::THD,
    lex: *const sys::Lex,
) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        // SAFETY: lex null or valid for read for this call.
        let lex_ref = unsafe { lex.as_ref() };
        match runtime::handlerton() {
            Some(h) => result_to_error(h.optimize_secondary_engine(thd_ref, lex_ref)),
            None => false,
        }
    })
}

/// `compare_secondary_engine_cost`. Writes `(use_best_so_far, cheaper,
/// secondary_engine_cost)` back through the three caller-owned pointers.
///
/// # Safety
/// `thd` / `join` null or valid; the three out pointers are null or writable
/// for one element each.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__compare_secondary_engine_cost(
    thd: *const sys::THD,
    join: *const sys::Join,
    optimizer_cost: f64,
    use_best_so_far: *mut bool,
    cheaper: *mut bool,
    secondary_engine_cost: *mut f64,
) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        // SAFETY: join null or valid for read for this call.
        let join_ref = unsafe { join.as_ref() };
        let triple = match runtime::handlerton() {
            Some(h) => h.compare_secondary_engine_cost(thd_ref, join_ref, optimizer_cost),
            None => Ok(None),
        };
        let (best, ch, cost) = match triple {
            Ok(Some(t)) => t,
            Ok(None) => (false, false, optimizer_cost),
            Err(_) => return true,
        };
        if !use_best_so_far.is_null() {
            // SAFETY: caller guarantees `use_best_so_far` is writable for one bool.
            unsafe { use_best_so_far.write(best) };
        }
        if !cheaper.is_null() {
            // SAFETY: caller guarantees `cheaper` is writable for one bool.
            unsafe { cheaper.write(ch) };
        }
        if !secondary_engine_cost.is_null() {
            // SAFETY: caller guarantees `secondary_engine_cost` is writable for one f64.
            unsafe { secondary_engine_cost.write(cost) };
        }
        false
    })
}

/// # Safety
/// `thd` / `hypergraph` / `access_path` null or valid; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__secondary_engine_modify_access_path_cost(
    thd: *const sys::THD,
    hypergraph: *const sys::JoinHypergraph,
    access_path: *const sys::AccessPath,
) -> bool {
    FfiBoundary::run_default(true, || {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        // SAFETY: hypergraph null or valid for read for this call.
        let hg_ref = unsafe { hypergraph.as_ref() };
        // SAFETY: access_path null or valid for read for this call.
        let ap_ref = unsafe { access_path.as_ref() };
        match runtime::handlerton() {
            Some(h) => {
                result_to_error(h.secondary_engine_modify_access_path_cost(thd_ref, hg_ref, ap_ref))
            }
            None => false,
        }
    })
}

/// # Safety
/// `thd` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__external_engine_explain_check(thd: *const sys::THD) -> bool {
    // Fail closed on panic: `true = some table not loaded`, so reporting `true`
    // on panic keeps MySQL from routing the explain through a secondary-engine
    // path the engine can no longer vouch for. The non-panic default still
    // returns `false` (all loaded) per the trait method's documented semantics.
    FfiBoundary::run_default(true, || {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.external_engine_explain_check(thd_ref),
            None => false,
        }
    })
}

/// `secondary_engine_check_optimizer_request`. Unpacks the
/// `SecondaryEngineOptimizerRequest` the trait returns into two integer
/// out-params, since the C signature returns the
/// `SecondaryEngineGraphSimplificationRequestParameters` struct by value.
///
/// # Safety
/// `thd` / `hypergraph` / `access_path` null or valid; `out_request` /
/// `out_subgraph_pair_limit` are non-null and writable for one element each.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rust__hton__secondary_engine_check_optimizer_request(
    thd: *const sys::THD,
    hypergraph: *const sys::JoinHypergraph,
    access_path: *const sys::AccessPath,
    current_subgraph_pairs: i32,
    current_subgraph_pairs_limit: i32,
    is_root_access_path: bool,
    out_request: *mut i32,
    out_subgraph_pair_limit: *mut i32,
) {
    FfiBoundary::run_void(|| {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        // SAFETY: hypergraph null or valid for read for this call.
        let hg_ref = unsafe { hypergraph.as_ref() };
        // SAFETY: access_path null or valid for read for this call.
        let ap_ref = unsafe { access_path.as_ref() };
        let req = match runtime::handlerton() {
            Some(h) => h.secondary_engine_check_optimizer_request(
                thd_ref,
                hg_ref,
                ap_ref,
                current_subgraph_pairs,
                current_subgraph_pairs_limit,
                is_root_access_path,
            ),
            None => SecondaryEngineOptimizerRequest::keep_going(),
        };
        if !out_request.is_null() {
            // SAFETY: caller guarantees `out_request` is writable for one i32.
            unsafe { out_request.write(req.request.to_raw()) };
        }
        if !out_subgraph_pair_limit.is_null() {
            // SAFETY: caller guarantees `out_subgraph_pair_limit` is writable for one i32.
            unsafe { out_subgraph_pair_limit.write(req.subgraph_pair_limit) };
        }
    });
}

/// # Safety
/// `thd` null or valid for the call; not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust__hton__secondary_engine_pre_prepare_hook(
    thd: *const sys::THD,
) -> bool {
    FfiBoundary::run_default(false, || {
        // SAFETY: thd null or valid for read for this call.
        let thd_ref = unsafe { thd.as_ref() };
        match runtime::handlerton() {
            Some(h) => h.secondary_engine_pre_prepare_hook(thd_ref),
            None => false,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EngineError;

    #[test]
    fn ok_maps_to_no_error() {
        assert!(!result_to_error(Ok(())));
    }

    #[test]
    fn err_maps_to_error() {
        assert!(result_to_error(Err(EngineError::Unsupported)));
    }
}
