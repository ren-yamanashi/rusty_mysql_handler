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

//! FFI boundary microbenchmark.
//!
//! Every `extern "C"` callback this crate exposes funnels its body
//! through [`FfiBoundary::run`] / [`run_void`] / [`run_default`], which
//! wrap the closure in `catch_unwind` so a Rust panic cannot unwind into
//! the MySQL server. This bench measures the cost of that wrapping by
//! comparing each helper against the same closure invoked natively.
//!
//! `black_box` brackets every value that enters or leaves a closure so
//! LLVM cannot fold the trivial bodies away — without that, the native
//! baseline optimizes to zero and the wrapper looks like pure overhead.
//!
//! Run with `cargo bench --bench ffi_overhead`. Results land in
//! `target/criterion/`.

#![allow(missing_docs)]
// `unit_arg` and `redundant_closure_call` fire on the very pattern that
// makes this bench meaningful: passing `black_box(())` through `Ok` /
// `Err` and invoking a single-call closure prevent LLVM from
// constant-folding the body into the call site, which would erase the
// FFI-boundary overhead the bench is supposed to measure.
#![allow(clippy::unit_arg, clippy::redundant_closure_call)]

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use mysql_handler::engine::EngineError;
use mysql_handler::panic_guard::FfiBoundary;

fn bench_run_ok(c: &mut Criterion) {
    let mut group = c.benchmark_group("run_ok");
    group.bench_function("via_ffi_boundary", |b| {
        b.iter(|| black_box(FfiBoundary::run(|| Ok::<_, EngineError>(black_box(())))));
    });
    group.bench_function("native_call", |b| {
        b.iter(|| {
            let r: Result<(), EngineError> = Ok(black_box(()));
            black_box(match r {
                Ok(()) => 0i32,
                Err(e) => e.to_mysql_errno(),
            })
        });
    });
    group.finish();
}

fn bench_run_err(c: &mut Criterion) {
    let mut group = c.benchmark_group("run_err");
    group.bench_function("via_ffi_boundary", |b| {
        b.iter(|| {
            black_box(FfiBoundary::run(|| {
                Err::<(), _>(black_box(EngineError::EndOfFile))
            }))
        });
    });
    group.bench_function("native_call", |b| {
        b.iter(|| {
            let r: Result<(), EngineError> = Err(black_box(EngineError::EndOfFile));
            black_box(match r {
                Ok(()) => 0i32,
                Err(e) => e.to_mysql_errno(),
            })
        });
    });
    group.finish();
}

fn bench_run_void(c: &mut Criterion) {
    let mut group = c.benchmark_group("run_void");
    group.bench_function("via_ffi_boundary", |b| {
        b.iter(|| FfiBoundary::run_void(|| black_box(())));
    });
    group.bench_function("native_call", |b| {
        b.iter(|| (|| black_box(()))());
    });
    group.finish();
}

fn bench_run_default(c: &mut Criterion) {
    let mut group = c.benchmark_group("run_default");
    group.bench_function("via_ffi_boundary", |b| {
        b.iter(|| black_box(FfiBoundary::run_default(0u32, || black_box(42u32))));
    });
    group.bench_function("native_call", |b| {
        b.iter(|| black_box((|| black_box(42u32))()));
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_run_ok,
    bench_run_err,
    bench_run_void,
    bench_run_default
);
criterion_main!(benches);
