# Copyright (C) 2026 ren-yamanashi
# shellcheck shell=bash
#
# Common environment for sysbench harness scripts. Sourced by phase0.sh
# and matrix.sh through run.sh.

SYSBENCH_PLUGIN_IMAGE="${SYSBENCH_PLUGIN_IMAGE:-rusty-plugin-build:local}"
SYSBENCH_IMAGE="${SYSBENCH_IMAGE:-rusty-sysbench:local}"
SYSBENCH_CONTAINER="${SYSBENCH_CONTAINER:-rusty-sysbench-$$}"
SYSBENCH_ROOT_PW="${SYSBENCH_ROOT_PW:-root}"

# Per-cell tunables. Overridable for a short smoke run.
SYSBENCH_WARMUP="${SYSBENCH_WARMUP:-10}"
SYSBENCH_TIME="${SYSBENCH_TIME:-30}"
SYSBENCH_TRIALS="${SYSBENCH_TRIALS:-3}"
SYSBENCH_CPUS="${SYSBENCH_CPUS:-4}"
SYSBENCH_MEMORY="${SYSBENCH_MEMORY:-4g}"

# Output dir for per-cell JSON. Cleaned at run start.
SYSBENCH_OUTPUT_DIR="${SYSBENCH_OUTPUT_DIR:-tests/sysbench/output}"
