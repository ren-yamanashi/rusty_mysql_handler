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

//! Per-`rust__handler__*` microbenchmark. Each callback's bench group
//! has three arms isolating FFI dispatch cost:
//!
//! - **via_ffi**: direct `rust__handler__xxx(ctx, args…)` — full C ABI,
//!   `FfiBoundary::run*` `catch_unwind`, opaque-pointer dispatch.
//! - **native**: `engine.method(args…)` on a local `Box<dyn StorageEngine>`.
//! - **via_fn_ptr**: indirect call through a `black_box`-fenced function
//!   pointer; upper bound on register pressure / branch prediction.
//!   Does not model PLT lazy binding.
//!
//! `ctx` is managed by [`common::CtxGuard`] for RAII teardown.
//! Compile-time integer / bool args are `black_box`-fenced uniformly
//! across all three arms so cross-group `via_ffi` numbers stay
//! comparable.
//!
//! The bench bodies are split per `src/handler/*` module so no single
//! source file exceeds the 250-line ceiling.

#![allow(missing_docs)]
#![allow(clippy::unit_arg, clippy::redundant_closure_call)]
#![allow(unsafe_code)]

#[path = "../common/mod.rs"]
mod shared;

mod common;
mod index_basic;
mod row_operations;
mod scan;
mod statistics;

use criterion::{criterion_group, criterion_main};

criterion_group!(
    benches,
    index_basic::bench_index_init,
    index_basic::bench_index_end,
    index_basic::bench_index_read_map,
    index_basic::bench_index_next,
    scan::bench_rnd_next,
    scan::bench_rnd_pos,
    row_operations::bench_write_row,
    row_operations::bench_update_row,
    row_operations::bench_delete_row,
    statistics::bench_info,
);
criterion_main!(benches);
