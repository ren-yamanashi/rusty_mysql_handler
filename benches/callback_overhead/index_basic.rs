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

//! Bench groups for `src/handler/index_basic.rs` callbacks.

#![allow(unreachable_pub)]

use std::hint::black_box;

use criterion::Criterion;

use mysql_handler::engine::{IndexedEngine, RKeyFunction};
use mysql_handler::handler::index_basic::{
    rust__handler__index_end, rust__handler__index_init, rust__handler__index_next,
    rust__handler__index_read_map,
};
use mysql_handler::runtime::EngineContext;

use super::common::{CtxGuard, native_engine};

pub fn bench_index_init(c: &mut Criterion) {
    let ctx = CtxGuard::new();
    let mut engine = native_engine();
    let fp_ffi: unsafe extern "C" fn(*mut EngineContext, u32, bool) -> i32 =
        rust__handler__index_init;

    let mut group = c.benchmark_group("index_init");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: `ctx.as_ptr()` is valid for the lifetime of `ctx`.
            black_box(unsafe {
                rust__handler__index_init(ctx.as_ptr(), black_box(0), black_box(false))
            })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.index_init(black_box(0), black_box(false))));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: `fp_ffi` points at `rust__handler__index_init`;
            // `ctx.as_ptr()` is valid for the lifetime of `ctx`.
            black_box(unsafe { f(ctx.as_ptr(), black_box(0), black_box(false)) })
        });
    });
    group.finish();
}

pub fn bench_index_end(c: &mut Criterion) {
    let ctx = CtxGuard::new();
    let mut engine = native_engine();
    let fp_ffi: unsafe extern "C" fn(*mut EngineContext) -> i32 = rust__handler__index_end;

    let mut group = c.benchmark_group("index_end");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: `ctx.as_ptr()` is valid for the lifetime of `ctx`.
            black_box(unsafe { rust__handler__index_end(ctx.as_ptr()) })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.index_end()));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: see `bench_index_init` above.
            black_box(unsafe { f(ctx.as_ptr()) })
        });
    });
    group.finish();
}

pub fn bench_index_read_map(c: &mut Criterion) {
    let ctx = CtxGuard::new();
    let mut engine = native_engine();
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
            // SAFETY: `ctx.as_ptr()` is valid; `buf`/`key` outlive this
            // closure.
            black_box(unsafe {
                rust__handler__index_read_map(
                    ctx.as_ptr(),
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
            // SAFETY: see `bench_index_read_map` `via_ffi` above.
            black_box(unsafe {
                f(
                    ctx.as_ptr(),
                    buf.as_mut_ptr(),
                    black_box(buf.len()),
                    key.as_ptr(),
                    black_box(key.len()),
                    black_box(0),
                )
            })
        });
    });
    group.finish();
}

pub fn bench_index_next(c: &mut Criterion) {
    let ctx = CtxGuard::new();
    let mut engine = native_engine();
    let fp_ffi: unsafe extern "C" fn(*mut EngineContext, *mut u8, usize) -> i32 =
        rust__handler__index_next;
    let mut buf = [0u8; 16];

    let mut group = c.benchmark_group("index_next");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: `ctx.as_ptr()` is valid; `buf` outlives the closure.
            black_box(unsafe {
                rust__handler__index_next(ctx.as_ptr(), buf.as_mut_ptr(), black_box(buf.len()))
            })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.index_next(&mut buf)));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: see `bench_index_next` `via_ffi` above.
            black_box(unsafe { f(ctx.as_ptr(), buf.as_mut_ptr(), black_box(buf.len())) })
        });
    });
    group.finish();
}
