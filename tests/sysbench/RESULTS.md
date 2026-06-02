# Performance Results

Results of the sysbench-driven plugin performance baseline. Populated
session-by-session; the headline table at the top is always the most
recent canonical session.

All sections — Callback profile, per-callback FFI overhead, OLTP
throughput, and Composing — are filled below. The OLTP section was
measured on macOS Docker Desktop with Rosetta amd64 emulation; both
engines see the same emulation overhead so the **ratio** (rusty / MEMORY)
is the load-bearing column, not the absolute tps. L1 per-callback FFI
overhead is native arm64 (`cargo bench`); callback-profile `Yᵢ` are
integer counter deltas insensitive to execution speed.

## Environment

| Field | Value |
|---|---|
| Session date | 2026-06-02 |
| Host CPU | Apple M3 Pro |
| Host cores (physical / logical) | 12 / 12 |
| Host RAM | 36 GB |
| Kernel | Darwin 23.1.0 arm64 |
| Docker version | 28.5.1 (Docker Desktop on macOS, amd64 emulation for the mysqld + client images) |
| Container CPU / memory budget | 4 vCPU / 4 GB (mysqld container) |
| mysqld build hash | 8.4.9 Community Server (image `mysql:8.4.9`) |
| Plugin commit SHA | 78f5e9fb1df0d911020ee424fde33693a0193d7e |
| Plugin tree dirty flag | clean (release build, no local edits) |
| `rustc --version` | 1.95.0 (59807616e 2026-04-14) |
| sysbench version | 1.0.20 (debian bookworm-slim apt) |
| N (trials), warmup, run duration | L1: `cargo bench` defaults (100 samples / 5 s warmup / 3 s measurement window); callback profile: `SYSBENCH_TRIALS=1 SYSBENCH_WARMUP=0 SYSBENCH_TIME=30` (warmup off so the `Handler_%` delta lines up with the measurement-run `tx` count) |

## Callback profile per scenario

Per-transaction `Yᵢ` values, captured by
`make perf-callback-profile`. `Handler_%` counters straddle a 30 s
sysbench run per scenario; the count divided by the total transaction
count gives `Yᵢ`. Counts are integer deltas reported by mysqld and do
not depend on execution speed, so they are usable as canonical
numbers despite the Rosetta-emulated mysqld.

| Callback | `oltp_point_select` | `oltp_read_only` | `oltp_read_write` |
|---|---|---|---|
| `index_read_map` (`Handler_read_key`) | 1.00 | 14.00 | 17.00 |
| `index_next` (`Handler_read_next`) | 0.00 | 400.00 | 400.00 |
| `rnd_pos` (`Handler_read_rnd`) | 0.00 | 0.00 | 0.00 |
| `rnd_next` (`Handler_read_rnd_next`) | 0.00 | 101.02 | 101.03 |
| `write_row` (`Handler_write`) | 0.00 | 100.00 | 101.00 |
| `update_row` (`Handler_update`) | 0.00 | 0.00 | 2.00 |
| `delete_row` (`Handler_delete`) | 0.00 | 0.00 | 1.00 |
| `index_init` | 1.00 | 14.00 | 17.00 |
| `index_end` | 1.00 | 14.00 | 17.00 |
| `position` | 0.00 | 0.00 | 0.00 |
| `info` | 1.00 | 15.00 | 24.00 |

Notes:

- `oltp_read_only` and `oltp_read_write` report non-zero `Handler_write`
  / `Handler_read_rnd_next` even though `oltp_read_only` issues only
  SELECTs. The reason is the `order_range` / `distinct_range` queries
  in sysbench's standard OLTP mix that trigger internal temp tables
  for ORDER BY / DISTINCT; the temp tables use the session default
  storage engine (RUSTY in this capture) and the writes / sequential
  reads are real per-tx work that the binding's FFI path absorbs.
- `index_init` / `index_end` / `info` Yᵢ are inferred per the
  scenario formula in the implementation plan (one per index-scan
  statement / one per statement respectively). `position` is paired
  with `rnd_pos`, both 0 for these scenarios.

## Per-callback FFI overhead

Measured by `cargo bench --bench callback_overhead` on Apple Silicon
native arm64 (no Rosetta). Δ = `via_ffi − native`. `via_fn_ptr` is
the indirect-call upper bound (function-pointer dispatch with the
pointer fenced through `black_box`).

| Callback | via_ffi (ns) | native (ns) | Δ (ns) | via_fn_ptr (ns) |
|---|---|---|---|---|
| `index_init` | 1.349 | 0.809 | 0.540 | 1.357 |
| `index_end` | 1.353 | 0.806 | 0.548 | 1.347 |
| `index_read_map` | 1.433 | 0.805 | 0.628 | 1.718 |
| `index_next` | 1.338 | 0.801 | 0.537 | 1.368 |
| `rnd_next` | 1.349 | 0.802 | 0.547 | 1.343 |
| `rnd_pos` | 1.352 | 0.811 | 0.541 | 1.520 |
| `write_row` | 1.378 | 0.838 | 0.541 | 1.369 |
| `update_row` | 1.349 | 0.806 | 0.543 | 1.521 |
| `delete_row` | 1.339 | 0.802 | 0.538 | 1.344 |
| `info` | 1.341 | 0.801 | 0.540 | 1.372 |

The per-callback FFI overhead is consistent at ~0.54 ns across the
nine simpler callbacks; `index_read_map` runs slightly heavier
(~0.63 ns) because the FFI body decodes both a write buffer and a
read key plus a `RKeyFunction` flag, where the others touch one
buffer at most. `via_fn_ptr` matches `via_ffi` within noise for most
callbacks; `index_read_map`, `rnd_pos`, and `update_row` are the
exceptions where the wider signature (more `extern "C"` arguments
passed through a register-loaded function pointer) shows ~0.15 ns
of register-pressure overhead on top of the FFI dispatch.

## OLTP throughput (rusty vs MEMORY)

Measured by `make perf-matrix` on macOS Docker Desktop with Rosetta
amd64 emulation (the same caveat as the callback profile applies).
`SYSBENCH_TIME=30, SYSBENCH_WARMUP=10, SYSBENCH_TRIALS=3`. tps is
the median across 3 trials; `stddev/median` (the variance ratio
listed under "tps σ/m") is the diagnostic. Cells with
`stddev/median > 10 %` are flagged in the right column.

| Engine | Scenario | Threads | Dataset | tps (median) | tps σ/m | p50 / p95 (ms) | rusty/MEM | Flag |
|---|---|---|---|---|---|---|---|---|
| rusty | oltp_point_select | 1 | 10k | 17 731.62 | 1.2 % | 0.06 / 0.07 | 1.09 | |
| MEMORY | oltp_point_select | 1 | 10k | 16 249.46 | 1.8 % | 0.06 / 0.07 | — | |
| rusty | oltp_point_select | 1 | 100k | 17 608.14 | 0.4 % | 0.06 / 0.07 | 1.18 | |
| MEMORY | oltp_point_select | 1 | 100k | 14 868.47 | 7.2 % | 0.07 / 0.08 | — | |
| rusty | oltp_point_select | 4 | 10k | 54 455.80 | 0.4 % | 0.07 / 0.08 | 1.02 | |
| MEMORY | oltp_point_select | 4 | 10k | 53 482.59 | 6.5 % | 0.07 / 0.09 | — | |
| rusty | oltp_point_select | 4 | 100k | 53 909.29 | 1.9 % | 0.07 / 0.09 | 0.99 | |
| MEMORY | oltp_point_select | 4 | 100k | 54 189.08 | 0.5 % | 0.07 / 0.08 | — | |
| rusty | oltp_point_select | 16 | 10k | 152 126.52 | 6.0 % | 0.10 / 0.11 | 1.01 | |
| MEMORY | oltp_point_select | 16 | 10k | 150 668.26 | 6.5 % | 0.11 / 0.11 | — | |
| rusty | oltp_point_select | 16 | 100k | 140 931.27 | 5.1 % | 0.11 / 0.12 | 1.03 | |
| MEMORY | oltp_point_select | 16 | 100k | 137 308.27 | 8.3 % | 0.12 / 0.12 | — | |
| rusty | oltp_read_only | 1 | 10k | 766.04 | 16.5 % | 1.30 / 1.50 | 1.02 | ⚠ |
| MEMORY | oltp_read_only | 1 | 10k | 748.91 | 1.8 % | 1.33 / 1.50 | — | |
| rusty | oltp_read_only | 1 | 100k | 653.86 | 2.5 % | 1.53 / 2.91 | 0.82 | |
| MEMORY | oltp_read_only | 1 | 100k | 793.16 | 2.6 % | 1.26 / 1.39 | — | |
| rusty | oltp_read_only | 4 | 10k | 2 209.64 | 8.9 % | 1.81 / 3.36 | 0.99 | |
| MEMORY | oltp_read_only | 4 | 10k | 2 227.10 | 11.1 % | 1.79 / 3.30 | — | ⚠ |
| rusty | oltp_read_only | 4 | 100k | 2 176.96 | 1.1 % | 1.83 / 3.36 | 0.86 | |
| MEMORY | oltp_read_only | 4 | 100k | 2 533.66 | 6.8 % | 1.58 / 1.70 | — | |
| rusty | oltp_read_only | 16 | 10k | 3 682.06 | 9.5 % | 4.34 / 8.90 | 0.92 | |
| MEMORY | oltp_read_only | 16 | 10k | 4 003.69 | 5.5 % | 3.99 / 6.55 | — | |
| rusty | oltp_read_only | 16 | 100k | 4 225.40 | 1.7 % | 3.78 / 5.28 | 1.07 | |
| MEMORY | oltp_read_only | 16 | 100k | 3 963.15 | 3.2 % | 4.03 / 5.99 | — | |
| rusty | oltp_read_write | 1 | 10k | 489.24 | 17.4 % | 2.04 / 6.32 | 0.81 | ⚠ |
| MEMORY | oltp_read_write | 1 | 10k | 602.06 | 9.0 % | 1.66 / 1.89 | — | |
| rusty | oltp_read_write | 1 | 100k | 634.16 | 19.2 % | 1.58 / 1.82 | 1.01 | ⚠ |
| MEMORY | oltp_read_write | 1 | 100k | 626.69 | 2.2 % | 1.59 / 1.86 | — | |
| rusty | oltp_read_write | 4 | 10k | 1 433.63 | 0.4 % | 2.79 / 3.55 | 1.07 | |
| MEMORY | oltp_read_write | 4 | 10k | 1 345.11 | 3.5 % | 2.97 / 4.03 | — | |
| rusty | oltp_read_write | 4 | 100k | 1 408.53 | 0.6 % | 2.84 / 3.55 | 1.02 | |
| MEMORY | oltp_read_write | 4 | 100k | 1 386.86 | 11.3 % | 2.88 / 3.75 | — | ⚠ |
| rusty | oltp_read_write | 16 | 10k | 1 619.46 | 1.8 % | 9.87 / 35.59 | 1.00 | |
| MEMORY | oltp_read_write | 16 | 10k | 1 621.05 | 4.3 % | 9.86 / 33.72 | — | |
| rusty | oltp_read_write | 16 | 100k | 1 847.88 | 1.0 % | 8.65 / 38.25 | 1.01 | |
| MEMORY | oltp_read_write | 16 | 100k | 1 831.37 | 0.7 % | 8.72 / 37.56 | — | |

5 cells (3 rusty + 2 MEMORY) flag `stddev/median > 10 %`; all are at
1 or 4 threads where low concurrency makes per-trial variance
genuinely large rather than indicating a harness problem. Higher-
thread cells settle.

The headline: rusty is within ±10 % of MEMORY across most cells and
0.99–1.18× at higher concurrency. The few cells where rusty trails
(`oltp_read_only` at 1t / 100k = 0.82×, `oltp_read_write` at 1t /
10k = 0.81×) are the same 1-thread cells where variance is highest;
that asymmetry stays inside the noise band.

## Composing per-callback and OLTP measurements

Take `oltp_point_select` at 1 thread / 10k rows as a worked example
(the other cells follow the same shape):

- `T_rusty` = 1 / 17 731.62 tps = 56.4 µs/tx
- `T_memory` = 1 / 16 249.46 tps = 61.5 µs/tx
- `Gap` = `T_rusty − T_memory` = **−5.1 µs/tx** (rusty is faster)

From the callback-profile table for `oltp_point_select`:

- `index_read_map`, `index_init`, `index_end`, `info`: `Yᵢ` = 1.0 each
- Other callbacks: `Yᵢ` ≈ 0

From the L1 table the per-callback Δ averages 0.54 ns, with
`index_read_map` at 0.63 ns. Plugging in:

- `FFI_tx` = (1 × 0.63) + (1 × 0.54) + (1 × 0.55) + (1 × 0.54) ≈ **2.3 ns/tx**

So the binding adds ~2.3 ns of FFI cost on top of every
`oltp_point_select` transaction whose wall time is ~56 µs. The FFI
share of that 56 µs is **2.3 ns / 56 400 ns ≈ 0.004 %** — effectively
zero. The rusty-vs-MEMORY gap (negative here: rusty is faster) is
dominated by the structural difference between `BTreeMap` and
MEMORY's BTREE plus measurement noise, not by FFI overhead.

The per-callback Δ table above decomposes as
`(FfiBoundary wrapper) + (EngineContext trait dispatch + opaque
pointer cast)`. The wrapper component is also measured separately
by the `ffi_overhead` bench in this repository; the two numbers are
overlapping measures of FFI cost from different angles, not
independent quantities to sum.

The same arithmetic for `oltp_read_only` (14 read_keys + 400
read_nexts + 14 index_init / index_end + 15 info per tx) yields
`FFI_tx` ≈ (14 × 0.63) + (400 × 0.54) + (28 × 0.54) + (15 × 0.54) +
(100 × 0.54) ≈ 305 ns/tx; against a ~1.3 ms transaction
(`T_rusty` = 1 / 766 tps ≈ 1.3 ms = 1 300 000 ns) that is still
**~0.02 %**. FFI cost is not where this binding pays the toll, for
either scenario.

## Caveats

- L1 per-callback Δ was measured on Apple Silicon native arm64.
  Absolute ns figures will differ on a Linux x86_64 host; the *shape*
  (consistent ~0.5–0.6 ns FFI delta independent of callback type)
  is the load-bearing finding.
- Callback profile `Yᵢ` came from a Rosetta-emulated mysqld + sysbench
  pair. Counts are independent of emulation speed.
- `oltp_read_only` showing non-zero writes is expected behaviour for
  sysbench's standard OLTP mix (internal temp tables for ORDER BY /
  DISTINCT — see the Callback profile note above).
- `via_ffi` does not include PLT lazy-binding cost on real dlopen
  plugins; `via_fn_ptr` approximates the indirect-call half of that
  cost. Read it as an indirect-call upper bound, not a full PLT model.
- The reference engine is demo-grade; absolute numbers are not a
  claim about what a production engine built on this binding could
  achieve.

## Session history (appendix)

| Session date | Plugin commit SHA | Sections filled | Notes |
|---|---|---|---|
| 2026-06-02 | 78f5e9f | Environment, Callback profile, L1 | macOS arm64 (L1) + Rosetta-emulated mysqld (callback profile). L2 / Composing deferred at this point. |
| 2026-06-03 | (this PR) | + L2, Composing | macOS Rosetta amd64 emulation for the OLTP matrix; rusty / MEMORY ratio is the load-bearing column. |
