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

//! Per-`rust__handler__*` microbenchmark. Three benchmark variants per
//! target callback isolate the FFI dispatch cost:
//!
//! - **via_ffi**: invoke `rust__handler__xxx(ctx, args…)` directly.
//!   Captures the C ABI calling convention, `FfiBoundary::run*` wrap
//!   (`catch_unwind`), opaque-pointer dispatch through
//!   `EngineContext::engine_mut()`, and the trait method body.
//! - **native**: invoke `engine.method(args…)` on a local
//!   `Box<dyn StorageEngine>`. No `catch_unwind`, no C ABI, no opaque
//!   pointer cast.
//! - **via_fn_ptr**: invoke `rust__handler__xxx` through an
//!   `extern "C" fn` pointer cast that goes through `black_box`. An
//!   indirect-call upper bound that captures the register-pressure and
//!   branch-prediction effects of a function-pointer invocation. Does
//!   **not** model PLT lazy binding (no GOT lookup, no resolver path).
//!
//! # Safety invariants shared across this bench
//!
//! - `ctx` always comes from a successful `rust__create_engine()` in
//!   `make_ctx()` and is exclusively owned by the calling bench
//!   function until `drop_ctx(ctx)` at the end. No two `b.iter`
//!   closures share the pointer concurrently because criterion runs
//!   bench functions sequentially within a single thread.
//! - Buffer pointers (`buf`, `key`, `old`, `new`, `pos`) are backed
//!   by stack-allocated `[u8; N]` arrays whose scope encloses every
//!   `b.iter` closure that reads them.
//! - Function-pointer values (`fp_ffi`) are typed exactly as the
//!   `rust__handler__*` symbol they point to; the cast does not
//!   change signature.

#![allow(missing_docs)]
// `unit_arg` and similar fire on the patterns that prevent LLVM from
// constant-folding the trivial bench bodies. Matches PR #54's approach.
#![allow(clippy::unit_arg, clippy::redundant_closure_call)]
// The whole point of this bench is to drive `unsafe extern "C"` FFI
// callbacks. The workspace's `unsafe_code = "warn"` lint would fire
// on every call site; we annotate the file once.
#![allow(unsafe_code)]

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use mysql_handler::engine::{RKeyFunction, StorageEngine};
use mysql_handler::handler::{
    index_basic::{
        rust__handler__index_end, rust__handler__index_init, rust__handler__index_next,
        rust__handler__index_read_map,
    },
    row_operations::{
        rust__handler__delete_row, rust__handler__update_row, rust__handler__write_row,
    },
    scan::{rust__handler__rnd_next, rust__handler__rnd_pos},
    statistics::rust__handler__info,
};
use mysql_handler::runtime::{
    EngineContext, register_engine_factory, rust__create_engine, rust__destroy_engine,
};

mod common;
use common::noop_engine::NoopEngine;

/// Build the EngineContext used by `via_ffi` and `via_fn_ptr` variants.
/// `register_engine_factory` ignores subsequent calls, so registering
/// `NoopEngine` here means every `rust__create_engine` call in this
/// binary returns a context whose dispatch hits `NoopEngine`.
fn make_ctx() -> *mut EngineContext {
    register_engine_factory(|| Box::new(NoopEngine::new()));
    // SAFETY: `rust__create_engine` is safe after `register_engine_factory`;
    // the returned pointer is null only when no factory is registered or the
    // factory panics, neither of which applies here.
    let ctx = unsafe { rust__create_engine() };
    assert!(!ctx.is_null(), "rust__create_engine returned null");
    ctx
}

/// Release a context obtained from `make_ctx`.
///
/// # Safety
/// `ctx` must be the pointer returned by a prior `make_ctx` call and
/// must not be reused after this returns.
unsafe fn drop_ctx(ctx: *mut EngineContext) {
    // SAFETY: caller upholds the contract above (ctx came from
    // `rust__create_engine` via `make_ctx` and is not reused).
    unsafe { rust__destroy_engine(ctx) };
}

fn bench_index_init(c: &mut Criterion) {
    let ctx = make_ctx();
    let mut engine: Box<dyn StorageEngine> = Box::new(NoopEngine::new());
    let fp_ffi: unsafe extern "C" fn(*mut EngineContext, u32, bool) -> i32 =
        rust__handler__index_init;

    let mut group = c.benchmark_group("index_init");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: see file-level safety invariants.
            black_box(unsafe { rust__handler__index_init(ctx, black_box(0), black_box(false)) })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.index_init(black_box(0), black_box(false))));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: `fp_ffi` is the same `rust__handler__index_init`
            // symbol; see file-level safety invariants for `ctx`.
            black_box(unsafe { f(ctx, black_box(0), black_box(false)) })
        });
    });
    group.finish();
    // SAFETY: `ctx` came from `make_ctx`; no further use after this line.
    unsafe { drop_ctx(ctx) };
}

fn bench_index_end(c: &mut Criterion) {
    let ctx = make_ctx();
    let mut engine: Box<dyn StorageEngine> = Box::new(NoopEngine::new());
    let fp_ffi: unsafe extern "C" fn(*mut EngineContext) -> i32 = rust__handler__index_end;

    let mut group = c.benchmark_group("index_end");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: see file-level safety invariants.
            black_box(unsafe { rust__handler__index_end(ctx) })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.index_end()));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: see file-level safety invariants.
            black_box(unsafe { f(ctx) })
        });
    });
    group.finish();
    // SAFETY: ctx came from make_ctx; no further use after this line.
    unsafe { drop_ctx(ctx) };
}

fn bench_index_read_map(c: &mut Criterion) {
    let ctx = make_ctx();
    let mut engine: Box<dyn StorageEngine> = Box::new(NoopEngine::new());
    let fp_ffi: unsafe extern "C" fn(
        *mut EngineContext,
        *mut u8,
        usize,
        *const u8,
        usize,
        i32,
    ) -> i32 = rust__handler__index_read_map;
    let mut buf = [0u8; 16];
    let key = [0u8; 8];

    let mut group = c.benchmark_group("index_read_map");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: see file-level safety invariants. `buf` / `key`
            // outlive this closure.
            black_box(unsafe {
                rust__handler__index_read_map(
                    ctx,
                    buf.as_mut_ptr(),
                    black_box(buf.len()),
                    key.as_ptr(),
                    black_box(key.len()),
                    black_box(0),
                )
            })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| {
            black_box(engine.index_read_map(&mut buf, &key, black_box(RKeyFunction::KeyExact)))
        });
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: see file-level safety invariants.
            black_box(unsafe { f(ctx, buf.as_mut_ptr(), buf.len(), key.as_ptr(), key.len(), 0) })
        });
    });
    group.finish();
    // SAFETY: ctx came from make_ctx; no further use after this line.
    unsafe { drop_ctx(ctx) };
}

fn bench_index_next(c: &mut Criterion) {
    let ctx = make_ctx();
    let mut engine: Box<dyn StorageEngine> = Box::new(NoopEngine::new());
    let fp_ffi: unsafe extern "C" fn(*mut EngineContext, *mut u8, usize) -> i32 =
        rust__handler__index_next;
    let mut buf = [0u8; 16];

    let mut group = c.benchmark_group("index_next");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: see file-level safety invariants.
            black_box(unsafe {
                rust__handler__index_next(ctx, buf.as_mut_ptr(), black_box(buf.len()))
            })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.index_next(&mut buf)));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: see file-level safety invariants.
            black_box(unsafe { f(ctx, buf.as_mut_ptr(), buf.len()) })
        });
    });
    group.finish();
    // SAFETY: ctx came from make_ctx; no further use after this line.
    unsafe { drop_ctx(ctx) };
}

fn bench_rnd_next(c: &mut Criterion) {
    let ctx = make_ctx();
    let mut engine: Box<dyn StorageEngine> = Box::new(NoopEngine::new());
    let fp_ffi: unsafe extern "C" fn(*mut EngineContext, *mut u8, usize) -> i32 =
        rust__handler__rnd_next;
    let mut buf = [0u8; 16];

    let mut group = c.benchmark_group("rnd_next");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: see file-level safety invariants.
            black_box(unsafe {
                rust__handler__rnd_next(ctx, buf.as_mut_ptr(), black_box(buf.len()))
            })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.rnd_next(&mut buf)));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: see file-level safety invariants.
            black_box(unsafe { f(ctx, buf.as_mut_ptr(), buf.len()) })
        });
    });
    group.finish();
    // SAFETY: ctx came from make_ctx; no further use after this line.
    unsafe { drop_ctx(ctx) };
}

fn bench_rnd_pos(c: &mut Criterion) {
    let ctx = make_ctx();
    let mut engine: Box<dyn StorageEngine> = Box::new(NoopEngine::new());
    let fp_ffi: unsafe extern "C" fn(*mut EngineContext, *mut u8, usize, *const u8, usize) -> i32 =
        rust__handler__rnd_pos;
    let mut buf = [0u8; 16];
    let pos = [0u8; 8];

    let mut group = c.benchmark_group("rnd_pos");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: see file-level safety invariants.
            black_box(unsafe {
                rust__handler__rnd_pos(ctx, buf.as_mut_ptr(), buf.len(), pos.as_ptr(), pos.len())
            })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.rnd_pos(&mut buf, &pos)));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: see file-level safety invariants.
            black_box(unsafe { f(ctx, buf.as_mut_ptr(), buf.len(), pos.as_ptr(), pos.len()) })
        });
    });
    group.finish();
    // SAFETY: ctx came from make_ctx; no further use after this line.
    unsafe { drop_ctx(ctx) };
}

fn bench_write_row(c: &mut Criterion) {
    let ctx = make_ctx();
    let mut engine: Box<dyn StorageEngine> = Box::new(NoopEngine::new());
    let fp_ffi: unsafe extern "C" fn(*mut EngineContext, *const u8, usize) -> i32 =
        rust__handler__write_row;
    let buf = [0u8; 16];

    let mut group = c.benchmark_group("write_row");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: see file-level safety invariants.
            black_box(unsafe { rust__handler__write_row(ctx, buf.as_ptr(), black_box(buf.len())) })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.write_row(&buf)));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: see file-level safety invariants.
            black_box(unsafe { f(ctx, buf.as_ptr(), buf.len()) })
        });
    });
    group.finish();
    // SAFETY: ctx came from make_ctx; no further use after this line.
    unsafe { drop_ctx(ctx) };
}

fn bench_update_row(c: &mut Criterion) {
    let ctx = make_ctx();
    let mut engine: Box<dyn StorageEngine> = Box::new(NoopEngine::new());
    let fp_ffi: unsafe extern "C" fn(
        *mut EngineContext,
        *const u8,
        usize,
        *const u8,
        usize,
    ) -> i32 = rust__handler__update_row;
    let old = [0u8; 16];
    let new = [0u8; 16];

    let mut group = c.benchmark_group("update_row");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: see file-level safety invariants.
            black_box(unsafe {
                rust__handler__update_row(ctx, old.as_ptr(), old.len(), new.as_ptr(), new.len())
            })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.update_row(&old, &new)));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: see file-level safety invariants.
            black_box(unsafe { f(ctx, old.as_ptr(), old.len(), new.as_ptr(), new.len()) })
        });
    });
    group.finish();
    // SAFETY: ctx came from make_ctx; no further use after this line.
    unsafe { drop_ctx(ctx) };
}

fn bench_delete_row(c: &mut Criterion) {
    let ctx = make_ctx();
    let mut engine: Box<dyn StorageEngine> = Box::new(NoopEngine::new());
    let fp_ffi: unsafe extern "C" fn(*mut EngineContext, *const u8, usize) -> i32 =
        rust__handler__delete_row;
    let buf = [0u8; 16];

    let mut group = c.benchmark_group("delete_row");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: see file-level safety invariants.
            black_box(unsafe { rust__handler__delete_row(ctx, buf.as_ptr(), black_box(buf.len())) })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.delete_row(&buf)));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: see file-level safety invariants.
            black_box(unsafe { f(ctx, buf.as_ptr(), buf.len()) })
        });
    });
    group.finish();
    // SAFETY: ctx came from make_ctx; no further use after this line.
    unsafe { drop_ctx(ctx) };
}

fn bench_info(c: &mut Criterion) {
    let ctx = make_ctx();
    let mut engine: Box<dyn StorageEngine> = Box::new(NoopEngine::new());
    let fp_ffi: unsafe extern "C" fn(*mut EngineContext, u32) -> i32 = rust__handler__info;

    let mut group = c.benchmark_group("info");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: see file-level safety invariants.
            black_box(unsafe { rust__handler__info(ctx, black_box(0)) })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.info(black_box(0))));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: see file-level safety invariants.
            black_box(unsafe { f(ctx, 0) })
        });
    });
    group.finish();
    // SAFETY: ctx came from make_ctx; no further use after this line.
    unsafe { drop_ctx(ctx) };
}

criterion_group!(
    benches,
    bench_index_init,
    bench_index_end,
    bench_index_read_map,
    bench_index_next,
    bench_rnd_next,
    bench_rnd_pos,
    bench_write_row,
    bench_update_row,
    bench_delete_row,
    bench_info,
);
criterion_main!(benches);
