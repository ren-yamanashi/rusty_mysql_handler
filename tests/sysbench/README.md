# sysbench performance harness

Docker-based sysbench harness for the reference engine. Two
subcommands:

- `make perf-callback-profile` — capture `Handler_%` deltas per
  scenario; output `tests/sysbench/output/callback_profile.json`.
- `make perf-matrix` — 36-cell matrix (engine × scenario × threads ×
  dataset), N trials per cell; output per-cell JSON plus a single
  `matrix.json` aggregating median / sample stddev / variance ratio.

The harness runs **two** containers connected by a private bridge
network: a `mysql:8.4.9` image with the plugin baked in, and a
`debian:bookworm-slim` client image with `sysbench`, `mysql-client`,
`python3`, and `jq` installed from apt. Both containers (and the
network) are torn down per invocation via `trap`. The output dir is
cleared at the start of each subcommand so re-runs do not fold stale
cells into the aggregation.

## Why two containers

`mysql:8.4.9` is built on Oracle Linux; the EPEL-9 sysbench package
drifts and patch-level pins go stale within months. Debian bookworm's
apt archive ships a long-term-stable sysbench (`1.0.20+ds-5`), and
snapshot.debian.org preserves historical packages if a session needs
to be reproduced after Debian rolls a new release. Splitting the
client out of the server image keeps the published numbers
reproducible without policing EPEL drift.

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

The Makefile target builds three images in order:
`rusty-plugin-build:local` (e2e `builder` stage), then
`rusty-sysbench-mysqld:local` (plugin + mysql:8.4.9), then
`rusty-sysbench-client:local` (debian:bookworm-slim + apt pins). All
three use BuildKit (`DOCKER_BUILDKIT=1`). Subsequent runs reuse the
cached layers.

A full matrix session takes ≈ 72 minutes at `SYSBENCH_TIME=30`
(default). For a smoke run, override the tunables:

```bash
SYSBENCH_TIME=5 SYSBENCH_WARMUP=2 SYSBENCH_TRIALS=1 make perf-matrix
```

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
`RESULTS.md` is updated by hand from these outputs when a new
measurement session is recorded.

## Tunables

| Variable | Default | Purpose |
|---|---|---|
| `SYSBENCH_TIME` | `30` | sysbench run duration (seconds) |
| `SYSBENCH_WARMUP` | `10` | sysbench warmup time (seconds) |
| `SYSBENCH_TRIALS` | `3` | per-cell trial count |
| `SYSBENCH_CPUS` | `4` | mysqld container CPU budget |
| `SYSBENCH_MEMORY` | `4g` | mysqld container memory budget |
| `SYSBENCH_OUTPUT_DIR` | `tests/sysbench/output` | where per-cell JSON lands |

## Files

- `Dockerfile.mysqld` — `FROM mysql:8.4.9` + plugin from
  `rusty-plugin-build:local`
- `Dockerfile.client` — `FROM debian:bookworm-slim`, apt-pinned
  sysbench / mysql-client / python3 / jq, ships `aggregate.py` in
  `/usr/local/bin`
- `.dockerignore` — keeps the build context to `aggregate.py`
- `prepare.sql` — piped to the mysqld container's `mysql` over the
  client container's stdin: `INSTALL PLUGIN`, creates `sbtest`
- `lib/env.sh` — shared env vars (image names, container names,
  network name, tunables)
- `lib/mysqld.sh` — container lifecycle, mysqld wait, handler counter
  capture
- `lib/sysbench.sh` — sysbench prepare / run / cleanup wrappers + text
  output parser
- `callback_profile.sh` — per-scenario `Handler_%` capture
- `matrix.sh` — OLTP matrix
- `run.sh` — dispatcher
- `aggregate.py` — bind-mounted `/output` JSON list → median / sample
  stddev rollup on stdout, runs inside the client container
