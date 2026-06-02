# Performance Results

Results of the sysbench-driven plugin performance baseline. Populated
session-by-session; the headline table at the top is always the most
recent canonical session.

The OLTP throughput section is intentionally deferred. macOS-on-Apple-Silicon
runs the mysql:8.4.9 image through Rosetta amd64 emulation, which adds
~1.5–2× wall-time overhead unrelated to FFI cost and would distort the
rusty-vs-MEMORY ratio. A follow-up canonical session via
`.github/workflows/perf.yml` will collect the OLTP numbers on a Linux
runner and fill the L2 / Composing sections; the L1 per-callback FFI
overhead and the callback profile are immune to that overhead (the
former runs native arm64 via `cargo bench`, the latter records counter
deltas that are insensitive to execution speed) and are filled below.

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

_Deferred._ A canonical Linux session via
`.github/workflows/perf.yml` will fill this section. macOS-on-Apple-
Silicon cannot host this measurement at canonical quality because
the mysql:8.4.9 image runs through Rosetta amd64 emulation and the
~1.5–2× wall-time overhead distorts the rusty-vs-MEMORY ratio.

## Composing per-callback and OLTP measurements

_Deferred._ Pending the OLTP throughput section. The composition
combines this file's per-callback Δ with the OLTP gap to attribute
the gap to FFI cost vs. structural diff
(`BTreeMap` vs. `MEMORY`-BTREE); without `T_rusty` and `T_memory`
numbers the formula has no inputs.

The per-callback Δ table above decomposes as
`(FfiBoundary wrapper) + (EngineContext trait dispatch + opaque
pointer cast)`. The wrapper component is also measured separately
by the `ffi_overhead` bench in this repository; the two numbers are
overlapping measures of FFI cost from different angles, not
independent quantities to sum.

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
| 2026-06-02 | 78f5e9f | Environment, Callback profile, L1 | macOS arm64 (L1) + Rosetta-emulated mysqld (callback profile). L2 / Composing deferred to a Linux canonical session. |
