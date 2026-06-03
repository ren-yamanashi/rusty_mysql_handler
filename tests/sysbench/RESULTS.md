# Performance Results

## TL;DR

- **Comparison**: rusty reference engine (this binding) vs MySQL
  built-in MEMORY engine, sysbench OLTP scenarios.
- **OLTP throughput**: rusty / MEMORY = **0.81–1.18× across 18
  ratio cells** (3 scenarios × 3 thread counts × 2 datasets);
  **0.86–1.07× at 4 / 16 threads** where per-trial variance settles.
- **Per-callback FFI overhead**: ~**0.54 ns** per callback
  (Δ = FFI call − native direct call).
- **Share of an OLTP transaction**: ≈ **0.004 %** at
  `oltp_point_select`, ≈ **0.03 %** at `oltp_read_only`.
- **Verdict**: FFI cost is not where this binding pays the toll.

Detailed tables follow. OLTP numbers come from Rosetta amd64
emulation on macOS, so read the ratio column, not absolute tps.

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
| N (trials), warmup, run duration | Per-callback bench: `cargo bench` defaults (100 samples / 5 s warmup / 3 s measurement window); callback profile: `SYSBENCH_TRIALS=1 SYSBENCH_WARMUP=0 SYSBENCH_TIME=30` (warmup off so the `Handler_%` delta lines up with the measurement-run `tx` count) |

## Callback profile per scenario

From `make perf-callback-profile`. Cell value = `Handler_%`
counter delta divided by total tx count for the scenario.

| Callback (cells = calls per tx, `Yᵢ`) | `oltp_point_select` | `oltp_read_only` | `oltp_read_write` |
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

- `oltp_read_only` writes / sequential reads come from sysbench's
  `order_range` / `distinct_range` queries materialising internal
  temp tables on the session default engine.
- `index_init` / `index_end` / `info` Yᵢ are inferred (one per
  index-scan statement / one per statement); `position` is paired
  with `rnd_pos`.

## Per-callback FFI overhead

`cargo bench --bench callback_overhead`, Apple Silicon arm64
native. The Δ column = "Call via FFI" − "Native direct call",
i.e. the cost this binding adds on top of a direct in-Rust call.

| Callback | **Library overhead Δ (ns) per call** | Call via FFI total (ns) | Native direct call (ns) | Indirect call via fn-pointer (ns, upper bound) |
|---|---|---|---|---|
| `index_init` | **0.540** | 1.349 | 0.809 | 1.357 |
| `index_end` | **0.548** | 1.353 | 0.806 | 1.347 |
| `index_read_map` | **0.628** | 1.433 | 0.805 | 1.718 |
| `index_next` | **0.537** | 1.338 | 0.801 | 1.368 |
| `rnd_next` | **0.547** | 1.349 | 0.802 | 1.343 |
| `rnd_pos` | **0.541** | 1.352 | 0.811 | 1.520 |
| `write_row` | **0.541** | 1.378 | 0.838 | 1.369 |
| `update_row` | **0.543** | 1.349 | 0.806 | 1.521 |
| `delete_row` | **0.538** | 1.339 | 0.802 | 1.344 |
| `info` | **0.540** | 1.341 | 0.801 | 1.372 |

Δ ≈ 0.54 ns across nine callbacks; `index_read_map` is heavier
(~0.63 ns) due to its wider signature. The fn-pointer column ≈
the FFI column except for `index_read_map` / `rnd_pos` / `update_row`
where wider argument lists add ~0.15 ns of register pressure.

## OLTP throughput (rusty vs MEMORY)

`make perf-matrix`. The throughput-ratio column is what to read
first (1.00× = parity, > 1 = rusty faster than MEMORY).

| Scenario | Threads | Rows | rusty tps (median of 3 trials) | MEMORY tps (median of 3 trials) | **Throughput ratio rusty ÷ MEMORY** | p95 latency rusty / MEMORY (ms) | High-variance flag (σ/median > 10 %) |
|---|---|---|---|---|---|---|---|
| oltp_point_select | 1 | 10k | 17 731.62 | 16 249.46 | **1.09×** | 0.07 / 0.07 | |
| oltp_point_select | 1 | 100k | 17 608.14 | 14 868.47 | **1.18×** | 0.07 / 0.08 | |
| oltp_point_select | 4 | 10k | 54 455.80 | 53 482.59 | **1.02×** | 0.08 / 0.09 | |
| oltp_point_select | 4 | 100k | 53 909.29 | 54 189.08 | **0.99×** | 0.09 / 0.08 | |
| oltp_point_select | 16 | 10k | 152 126.52 | 150 668.26 | **1.01×** | 0.11 / 0.11 | |
| oltp_point_select | 16 | 100k | 140 931.27 | 137 308.27 | **1.03×** | 0.12 / 0.12 | |
| oltp_read_only | 1 | 10k | 766.04 | 748.91 | **1.02×** | 1.50 / 1.50 | ⚠ rusty σ/m 16.5 % |
| oltp_read_only | 1 | 100k | 653.86 | 793.16 | **0.82×** | 2.91 / 1.39 | |
| oltp_read_only | 4 | 10k | 2 209.64 | 2 227.10 | **0.99×** | 3.36 / 3.30 | ⚠ MEM σ/m 11.1 % |
| oltp_read_only | 4 | 100k | 2 176.96 | 2 533.66 | **0.86×** | 3.36 / 1.70 | |
| oltp_read_only | 16 | 10k | 3 682.06 | 4 003.69 | **0.92×** | 8.90 / 6.55 | |
| oltp_read_only | 16 | 100k | 4 225.40 | 3 963.15 | **1.07×** | 5.28 / 5.99 | |
| oltp_read_write | 1 | 10k | 489.24 | 602.06 | **0.81×** | 6.32 / 1.89 | ⚠ rusty σ/m 17.4 % |
| oltp_read_write | 1 | 100k | 634.16 | 626.69 | **1.01×** | 1.82 / 1.86 | ⚠ rusty σ/m 19.2 % |
| oltp_read_write | 4 | 10k | 1 433.63 | 1 345.11 | **1.07×** | 3.55 / 4.03 | |
| oltp_read_write | 4 | 100k | 1 408.53 | 1 386.86 | **1.02×** | 3.55 / 3.75 | ⚠ MEM σ/m 11.3 % |
| oltp_read_write | 16 | 10k | 1 619.46 | 1 621.05 | **1.00×** | 35.59 / 33.72 | |
| oltp_read_write | 16 | 100k | 1 847.88 | 1 831.37 | **1.01×** | 38.25 / 37.56 | |

5 flagged cells are all at 1 or 4 threads; high-thread cells
settle. Rusty trails most where variance is highest
(`oltp_read_only` 1t/100k = 0.82×, `oltp_read_write` 1t/10k =
0.81×); the 4 / 16 thread range is 0.86–1.07×.

## Per-transaction FFI share

Two scenarios, 1 thread / 10k rows. The right-hand column is
the headline: FFI cost as a fraction of the full transaction.

| Scenario | FFI cost per tx (`Σ Yᵢ × Δᵢ`, callbacks × overhead) | rusty wall time per tx (`1 / tps`) | **FFI share of tx (FFI cost ÷ wall time)** |
|---|---|---|---|
| `oltp_point_select` | ~2.3 ns | 56.4 µs | **0.004 %** |
| `oltp_read_only` | ~358 ns | 1.3 ms | **0.03 %** |

Per-callback Δ decomposes as `FfiBoundary wrapper +
EngineContext trait dispatch + opaque pointer cast`. The wrapper
component is also measured by the `ffi_overhead` bench; the two
are overlapping views of the same cost, not additive.

## Caveats

- Per-callback Δ was measured on Apple Silicon native arm64.
  Absolute ns figures will differ on a Linux x86_64 host; the *shape*
  (consistent ~0.5–0.6 ns FFI delta independent of callback type)
  is the load-bearing finding.
- Callback profile `Yᵢ` came from a Rosetta-emulated mysqld + sysbench
  pair. Counts are independent of emulation speed.
- `oltp_read_only` showing non-zero writes is expected behaviour for
  sysbench's standard OLTP mix (internal temp tables for ORDER BY /
  DISTINCT — see the Callback profile note above).
- The "Call via FFI" measurement does not include PLT lazy-binding
  cost on real dlopen plugins; the "Indirect call via fn-pointer"
  column approximates the indirect-call half of that cost. Read it
  as an indirect-call upper bound, not a full PLT model.
- The reference engine is demo-grade; absolute numbers are not a
  claim about what a production engine built on this binding could
  achieve.

## Session history (appendix)

| Session date | Plugin commit SHA | Sections filled | Notes |
|---|---|---|---|
| 2026-06-02 | 78f5e9f | Environment, Callback profile, Per-callback FFI overhead | macOS arm64 (per-callback bench) + Rosetta-emulated mysqld (callback profile). OLTP throughput and the per-transaction FFI share analysis added in the next session. |
| 2026-06-03 | 4c2dc27 | + OLTP throughput, Per-transaction FFI share | macOS Rosetta amd64 emulation for the OLTP matrix; rusty / MEMORY ratio is the load-bearing column. |
