# Performance Results

Per [p6-05 design](../../.claude/tasks/p6-05-perf-design.md) and
[implementation plan](../../.claude/tasks/p6-05-perf-implementation.md).
This file is populated session-by-session; the headline table at the
top is always the most recent canonical session.

## Environment

| Field | Value |
|---|---|
| Session date | _placeholder_ |
| Host CPU | _placeholder_ |
| Host cores (physical / logical) | _placeholder_ |
| Host RAM | _placeholder_ |
| Kernel | _placeholder_ |
| Docker version | _placeholder_ |
| Container CPU / memory budget | _placeholder_ |
| mysqld build hash | _placeholder_ |
| Plugin commit SHA | _placeholder_ |
| Plugin tree dirty flag | _placeholder_ |
| `rustc --version` | _placeholder_ |
| sysbench version | _placeholder_ |
| N (trials), warmup, run duration | _placeholder_ |

## Phase 0 — Captured callback profile per scenario

Per-transaction `Yᵢ` values for each callback, captured per
[implementation plan](../../.claude/tasks/p6-05-perf-implementation.md#phase-0-capture-method).

| Callback | `oltp_point_select` | `oltp_read_only` | `oltp_read_write` |
|---|---|---|---|
| `index_read_map` | _placeholder_ | _placeholder_ | _placeholder_ |
| `index_next` | _placeholder_ | _placeholder_ | _placeholder_ |
| `rnd_pos` | _placeholder_ | _placeholder_ | _placeholder_ |
| `rnd_next` | _placeholder_ | _placeholder_ | _placeholder_ |
| `write_row` | _placeholder_ | _placeholder_ | _placeholder_ |
| `update_row` | _placeholder_ | _placeholder_ | _placeholder_ |
| `delete_row` | _placeholder_ | _placeholder_ | _placeholder_ |
| `index_init` | _placeholder_ | _placeholder_ | _placeholder_ |
| `index_end` | _placeholder_ | _placeholder_ | _placeholder_ |
| `position` | _placeholder_ | _placeholder_ | _placeholder_ |
| `info` | _placeholder_ | _placeholder_ | _placeholder_ |

(`rnd_init` / `rnd_end` hard-coded to 0 for `oltp_*` scenarios; an
assertion in `phase0.sh` confirms `Handler_read_rnd_next` delta is 0.
If non-zero, the cell shows the discovered value with a flag.)

## L1 — Per-callback FFI overhead

Measured by `cargo bench --bench callback_overhead`. Δ = `via_ffi −
native`. `via_fn_ptr − native` is an indirect-call upper bound
(does not model PLT lazy binding).

| Callback | via_ffi (ns) | native (ns) | Δ (ns) | via_fn_ptr (ns) |
|---|---|---|---|---|
| `index_init` | _placeholder_ | _placeholder_ | _placeholder_ | _placeholder_ |
| `index_end` | _placeholder_ | _placeholder_ | _placeholder_ | _placeholder_ |
| `index_read_map` | _placeholder_ | _placeholder_ | _placeholder_ | _placeholder_ |
| `index_next` | _placeholder_ | _placeholder_ | _placeholder_ | _placeholder_ |
| `rnd_next` | _placeholder_ | _placeholder_ | _placeholder_ | _placeholder_ |
| `rnd_pos` | _placeholder_ | _placeholder_ | _placeholder_ | _placeholder_ |
| `write_row` | _placeholder_ | _placeholder_ | _placeholder_ | _placeholder_ |
| `update_row` | _placeholder_ | _placeholder_ | _placeholder_ | _placeholder_ |
| `delete_row` | _placeholder_ | _placeholder_ | _placeholder_ | _placeholder_ |
| `info` | _placeholder_ | _placeholder_ | _placeholder_ | _placeholder_ |

## L2 — OLTP throughput (rusty vs MEMORY)

Measured by `make perf-matrix`. tps is the median across N trials;
`stddev/median` is the variance ratio diagnostic. Cells with
`stddev/median > 10 %` after N = 5 re-runs are flagged.

| Engine | Scenario | Threads | Dataset | tps (median ± stddev) | p50 / p95 / p99 (ms) | rusty/MEMORY | Errors |
|---|---|---|---|---|---|---|---|
| rusty | oltp_point_select | 1 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_point_select | 1 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| rusty | oltp_point_select | 4 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_point_select | 4 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| rusty | oltp_point_select | 16 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_point_select | 16 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| rusty | oltp_point_select | 1 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_point_select | 1 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| rusty | oltp_point_select | 4 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_point_select | 4 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| rusty | oltp_point_select | 16 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_point_select | 16 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| rusty | oltp_read_only | 1 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_read_only | 1 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| rusty | oltp_read_only | 4 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_read_only | 4 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| rusty | oltp_read_only | 16 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_read_only | 16 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| rusty | oltp_read_only | 1 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_read_only | 1 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| rusty | oltp_read_only | 4 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_read_only | 4 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| rusty | oltp_read_only | 16 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_read_only | 16 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| rusty | oltp_read_write | 1 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_read_write | 1 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| rusty | oltp_read_write | 4 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_read_write | 4 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| rusty | oltp_read_write | 16 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_read_write | 16 | 10k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| rusty | oltp_read_write | 1 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_read_write | 1 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| rusty | oltp_read_write | 4 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_read_write | 4 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| rusty | oltp_read_write | 16 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |
| MEMORY | oltp_read_write | 16 | 100k | _placeholder_ | _placeholder_ | — | _placeholder_ |

## Composing the two

PR 3 fills this section using the formulae from
[design § Integrating L1 + L2](../../.claude/tasks/p6-05-perf-design.md#integrating-l1--l2):

- `FFI_tx = Σᵢ Yᵢ × Zᵢ` (ns), summed over callbacks present in the scenario per Phase 0
- `Gap = T_rusty − T_memory` (ns)
- `share = FFI_tx / Gap`
- `residual = Gap − FFI_tx`, attributable to BTreeMap-vs-MEMORY-BTREE structural diff

A reconciliation paragraph relates the per-callback Δ (this file's
L1 column) to PR #54's `FfiBoundary` wrapper bench: the per-callback
Δ decomposes as `(FfiBoundary wrapper) + (EngineContext trait
dispatch + opaque pointer cast)`. The two are overlapping measures
of FFI cost from different angles, not independent quantities to
sum.

## Caveats

- Per D4, the MEMORY-side index is `USING BTREE`. The residual after
  subtracting FFI cost is the constant-factor gap between
  `std::collections::BTreeMap` and MEMORY's B-tree implementation
  (node layout, allocator, cache behaviour) — not a HEAP-vs-tree gap.
- `via_ffi` does not include PLT lazy-binding cost on real dlopen
  plugins; `via_fn_ptr` approximates the indirect-call half of that
  cost. Read it as an indirect-call upper bound, not a full PLT model.
- Docker-on-macOS goes through a Linux VM. Primary results are taken
  on a Linux host with specs declared in the Environment header.
- Variance > 10 % cells flagged and re-measured at higher N once
  per [D2](../../.claude/tasks/p6-05-perf-design.md#d2-trial-count-n)
  termination rule; if still > 10 %, reported as-is with the flag.
- The reference engine is demo-grade; absolute numbers are not a
  claim about what a production engine built on this binding could
  achieve.

## Session history (appendix)

| Session date | Plugin commit SHA | Headline (point_select 1t/10k rusty/MEMORY ratio) |
|---|---|---|

(empty until PR 3's canonical session lands)
