# sysbench performance harness

Docker-based sysbench harness for the reference engine. Two
subcommands:

- `make perf-phase0` — capture `Handler_%` deltas per scenario; output
  `tests/sysbench/output/phase0.json`.
- `make perf-matrix` — 36-cell matrix (engine × scenario × threads ×
  dataset), N trials per cell; output per-cell JSON plus a single
  `matrix.json` aggregating median / stddev / variance ratio.

Both wrap a single mysql:8.4.9 container with the rusty plugin baked
in and sysbench installed. The container is created, used, and torn
down per invocation (`trap` cleans up on success or interrupt).

## What gets measured

Three sysbench OLTP scenarios — `oltp_point_select`, `oltp_read_only`,
`oltp_read_write` — across two engines (rusty reference, MySQL built-in
MEMORY), three thread counts (1 / 4 / 16), and two dataset sizes
(10 000 / 100 000 rows).

`tps`, latency p50/avg/p95, error count, plus the variance ratio
(`stddev/median`) as a diagnostic per the design.

## Running

```bash
make perf-phase0
make perf-matrix
```

The Makefile target builds the `rusty-plugin-build:local` stage from
the e2e Dockerfile, then builds `rusty-sysbench:local` on top of
mysql:8.4.9. Subsequent runs reuse the cached layers.

A full matrix session takes ≈ 72 minutes at `SYSBENCH_TIME=30`
(default). For a smoke run, override the tunables:

```bash
SYSBENCH_TIME=5 SYSBENCH_WARMUP=2 SYSBENCH_TRIALS=1 make perf-matrix
```

The N=1 smoke variant is intended for harness verification (does
sysbench actually drive the rusty engine end-to-end?) and not for
publishable numbers.

## Output

```
tests/sysbench/output/
├── phase0.json                          (per-scenario Yᵢ)
├── RUSTY-oltp_point_select-1t-10000.json
├── ...                                  (36 per-cell files)
└── matrix.json                          (aggregated)
```

The aggregated `matrix.json` feeds `RESULTS.md`'s OLTP throughput
table; `phase0.json` feeds the callback-profile table. RESULTS.md
itself is filled by a separate canonical session.

## Tunables

| Variable | Default | Purpose |
|---|---|---|
| `SYSBENCH_TIME` | `30` | sysbench run duration (seconds) |
| `SYSBENCH_WARMUP` | `10` | sysbench warmup time (seconds) |
| `SYSBENCH_TRIALS` | `3` | per-cell trial count |
| `SYSBENCH_CPUS` | `4` | container CPU budget |
| `SYSBENCH_MEMORY` | `4g` | container memory budget |
| `SYSBENCH_OUTPUT_DIR` | `tests/sysbench/output` | where per-cell JSON lands |

## Files

- `Dockerfile` — `FROM rusty-plugin-build:local`, installs sysbench
- `Dockerfile.dockerignore` — minimal build context (`tests/sysbench`
  only)
- `prepare.sql` — `INSTALL PLUGIN rusty SONAME 'ha_rusty.so'`,
  creates `sbtest` database
- `lib/env.sh` — shared env vars
- `lib/mysqld.sh` — container lifecycle, mysqld wait, handler counter
  capture
- `lib/sysbench.sh` — sysbench prepare / run / cleanup wrappers + text
  output parser
- `phase0.sh` — Phase 0 subcommand
- `matrix.sh` — matrix subcommand
- `run.sh` — dispatcher
- `aggregate.py` — per-cell JSON → median / stddev rollup
