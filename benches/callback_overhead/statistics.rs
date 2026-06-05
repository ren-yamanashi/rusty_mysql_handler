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

//! Bench group for `src/handler/statistics.rs::rust__handler__info`.

#![allow(unreachable_pub)]

use std::hint::black_box;

use criterion::Criterion;

use mysql_handler::handler::statistics::rust__handler__info;
use mysql_handler::runtime::EngineContext;

use super::common::{CtxGuard, native_engine};

pub fn bench_info(c: &mut Criterion) {
    let ctx = CtxGuard::new();
    let mut engine = native_engine();
    let fp_ffi: unsafe extern "C" fn(*mut EngineContext, u32) -> i32 = rust__handler__info;

    let mut group = c.benchmark_group("info");
    group.bench_function("via_ffi", |b| {
        b.iter(|| {
            // SAFETY: `ctx.as_ptr()` is valid for the lifetime of `ctx`.
            black_box(unsafe { rust__handler__info(ctx.as_ptr(), black_box(0)) })
        });
    });
    group.bench_function("native", |b| {
        b.iter(|| black_box(engine.info(black_box(0))));
    });
    group.bench_function("via_fn_ptr", |b| {
        b.iter(|| {
            let f = black_box(fp_ffi);
            // SAFETY: see `bench_info` `via_ffi` above.
            black_box(unsafe { f(ctx.as_ptr(), black_box(0)) })
        });
    });
    group.finish();
}
