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

//! Bench groups for `src/handler/row_operations.rs` callbacks.

#![allow(unreachable_pub)]

use std::hint::black_box;

use criterion::Criterion;

use mysql_handler::engine::StorageEngine;
use mysql_handler::handler::row_operations::{
    rust__handler__delete_row, rust__handler__update_row, rust__handler__write_row,
};
use mysql_handler::runtime::EngineContext;

use super::common::{CtxGuard, native_engine};

pub fn bench_write_row(c: &mut Criterion) {
    let ctx = CtxGuard::new();
    let mut engine = native_engine();
    let fp_ffi: unsafe extern "C" fn(*mut EngineContext, *const u8, usize) -> i32 =
        rust__handler__write_row;
    let buf = [0u8; 16];

    let mut group = c.benchmark_group("write_row");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: `ctx.as_ptr()` is valid; `buf` outlives the closure.
            black_box(unsafe {
                rust__handler__write_row(ctx.as_ptr(), buf.as_ptr(), black_box(buf.len()))
            })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.write_row(&buf)));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: see `bench_write_row` `via_ffi` above.
            black_box(unsafe { f(ctx.as_ptr(), buf.as_ptr(), black_box(buf.len())) })
        });
    });
    group.finish();
}

pub fn bench_update_row(c: &mut Criterion) {
    let ctx = CtxGuard::new();
    let mut engine = native_engine();
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
            // SAFETY: `ctx.as_ptr()` is valid; `old`/`new` outlive the
            // closure.
            black_box(unsafe {
                rust__handler__update_row(
                    ctx.as_ptr(),
                    old.as_ptr(),
                    black_box(old.len()),
                    new.as_ptr(),
                    black_box(new.len()),
                )
            })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.update_row(&old, &new)));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: see `bench_update_row` `via_ffi` above.
            black_box(unsafe {
                f(
                    ctx.as_ptr(),
                    old.as_ptr(),
                    black_box(old.len()),
                    new.as_ptr(),
                    black_box(new.len()),
                )
            })
        });
    });
    group.finish();
}

pub fn bench_delete_row(c: &mut Criterion) {
    let ctx = CtxGuard::new();
    let mut engine = native_engine();
    let fp_ffi: unsafe extern "C" fn(*mut EngineContext, *const u8, usize) -> i32 =
        rust__handler__delete_row;
    let buf = [0u8; 16];

    let mut group = c.benchmark_group("delete_row");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: `ctx.as_ptr()` is valid; `buf` outlives the closure.
            black_box(unsafe {
                rust__handler__delete_row(ctx.as_ptr(), buf.as_ptr(), black_box(buf.len()))
            })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.delete_row(&buf)));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: see `bench_delete_row` `via_ffi` above.
            black_box(unsafe { f(ctx.as_ptr(), buf.as_ptr(), black_box(buf.len())) })
        });
    });
    group.finish();
}
