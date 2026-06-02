# sysbench performance harness

Docker-based sysbench harness for the reference engine. Two
subcommands:

- `make perf-callback-profile` — capture `Handler_%` deltas per
  scenario; output `tests/sysbench/output/callback_profile.json`.
- `make perf-matrix` — 36-cell matrix (engine × scenario × threads ×
  dataset), N trials per cell; output per-cell JSON plus a single
  `matrix.json` aggregating median / sample stddev / variance ratio.

Both wrap a single mysql:8.4.9 container with the rusty plugin baked
in and sysbench installed. The container is created, used, and torn
down per invocation (`trap` cleans up on success or interrupt). The
per-cell output dir is cleared at the start of each invocation so
re-runs after parameter changes do not fold stale cells into the
aggregation.

## What gets measured

Three sysbench OLTP scenarios — `oltp_point_select`, `oltp_read_only`,
`oltp_read_write` — across two engines (rusty reference, MySQL built-in
MEMORY), three thread counts (1 / 4 / 16), and two dataset sizes
(10 000 / 100 000 rows).

`tps`, latency p50/avg/p95, error count, plus the variance ratio
(`stddev/median`) as a diagnostic. Each trial seeds sysbench with
`--rand-seed=<trial-index>` so a given
`(engine, scenario, threads, dataset, trial)` tuple is reproducible
across sessions.

## Running

```bash
make perf-callback-profile
make perf-matrix
```

The Makefile target builds the `rusty-plugin-build:local` stage from
the e2e Dockerfile, then builds `rusty-sysbench:local` on top of
mysql:8.4.9. Both builds use BuildKit (`DOCKER_BUILDKIT=1`) so the
per-Dockerfile dockerignore works. Subsequent runs reuse the cached
layers.

A full matrix session takes ≈ 72 minutes at `SYSBENCH_TIME=30`
(default). For a smoke run, override the tunables:

```bash
SYSBENCH_TIME=5 SYSBENCH_WARMUP=2 SYSBENCH_TRIALS=1 make perf-matrix
```

The `SYSBENCH_TRIALS=1` smoke variant is intended for harness
verification (does sysbench actually drive the rusty engine
end-to-end?) and not for publishable numbers.

## Output

```
tests/sysbench/output/
├── callback_profile.json                (per-scenario Yᵢ)
├── RUSTY-oltp_point_select-1t-10000.json
├── ...                                  (36 per-cell files)
└── matrix.json                          (aggregated)
```

The aggregated `matrix.json` feeds `RESULTS.md`'s OLTP throughput
table; `callback_profile.json` feeds the callback-profile table.
`RESULTS.md` itself is filled by a separate canonical session.

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

- `Dockerfile` — `FROM rusty-plugin-build:local`, installs pinned
  sysbench / python3 / jq, COPYs `aggregate.py` into the image
- `Dockerfile.dockerignore` — BuildKit-only allowlist that keeps the
  per-image context to just `aggregate.py`
- `prepare.sql` — `INSTALL PLUGIN rusty SONAME 'ha_rusty.so'`,
  creates `sbtest` database
- `lib/env.sh` — shared env vars
- `lib/mysqld.sh` — container lifecycle, mysqld wait, handler counter
  capture
- `lib/sysbench.sh` — sysbench prepare / run / cleanup wrappers + text
  output parser
- `callback_profile.sh` — per-scenario `Handler_%` capture
- `matrix.sh` — OLTP matrix
- `run.sh` — dispatcher
- `aggregate.py` — per-cell JSON list (stdin) → median / sample stddev
  rollup (stdout). Runs inside the harness container.
