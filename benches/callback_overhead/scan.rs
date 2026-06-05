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

//! Bench groups for `src/handler/scan.rs` callbacks.

#![allow(unreachable_pub)]

use std::hint::black_box;

use criterion::Criterion;

use mysql_handler::engine::StorageEngine;
use mysql_handler::handler::scan::{rust__handler__rnd_next, rust__handler__rnd_pos};
use mysql_handler::runtime::EngineContext;

use super::common::{CtxGuard, native_engine};

pub fn bench_rnd_next(c: &mut Criterion) {
    let ctx = CtxGuard::new();
    let mut engine = native_engine();
    let fp_ffi: unsafe extern "C" fn(*mut EngineContext, *mut u8, usize) -> i32 =
        rust__handler__rnd_next;
    let mut buf = [0u8; 16];

    let mut group = c.benchmark_group("rnd_next");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: `ctx.as_ptr()` is valid; `buf` outlives the closure.
            black_box(unsafe {
                rust__handler__rnd_next(ctx.as_ptr(), buf.as_mut_ptr(), black_box(buf.len()))
            })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.rnd_next(&mut buf)));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: see `bench_rnd_next` `via_ffi` above.
            black_box(unsafe { f(ctx.as_ptr(), buf.as_mut_ptr(), black_box(buf.len())) })
        });
    });
    group.finish();
}

pub fn bench_rnd_pos(c: &mut Criterion) {
    let ctx = CtxGuard::new();
    let mut engine = native_engine();
    let fp_ffi: unsafe extern "C" fn(*mut EngineContext, *mut u8, usize, *const u8, usize) -> i32 =
        rust__handler__rnd_pos;
    let mut buf = [0u8; 16];
    let pos = [0u8; 8];

    let mut group = c.benchmark_group("rnd_pos");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: `ctx.as_ptr()` is valid; `buf`/`pos` outlive the
            // closure.
            black_box(unsafe {
                rust__handler__rnd_pos(
                    ctx.as_ptr(),
                    buf.as_mut_ptr(),
                    black_box(buf.len()),
                    pos.as_ptr(),
                    black_box(pos.len()),
                )
            })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.rnd_pos(&mut buf, &pos)));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: see `bench_rnd_pos` `via_ffi` above.
            black_box(unsafe {
                f(
                    ctx.as_ptr(),
                    buf.as_mut_ptr(),
                    black_box(buf.len()),
                    pos.as_ptr(),
                    black_box(pos.len()),
                )
            })
        });
    });
    group.finish();
}
