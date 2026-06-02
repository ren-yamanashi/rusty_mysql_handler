#!/usr/bin/env bash
# Copyright (C) 2026 ren-yamanashi
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License, version 2.0,
# as published by the Free Software Foundation.
#
# (license body trimmed; full text in tests/e2e/run.sh)

# Phase 0 — capture per-transaction Yᵢ (callback counts) for each scenario.
# Snapshots Handler_% session counters around a short sysbench run, diffs
# them, divides by transaction count, and emits per-scenario JSON.

set -euo pipefail

# shellcheck source=lib/env.sh
source tests/sysbench/lib/env.sh
# shellcheck source=lib/mysqld.sh
source tests/sysbench/lib/mysqld.sh
# shellcheck source=lib/sysbench.sh
source tests/sysbench/lib/sysbench.sh

mkdir -p "$SYSBENCH_OUTPUT_DIR"
out="$SYSBENCH_OUTPUT_DIR/phase0.json"

trap 'sysbench_stop_mysqld' EXIT
sysbench_start_mysqld
sysbench_set_engine RUSTY

scenarios=(oltp_point_select oltp_read_only oltp_read_write)
results=()

for scenario in "${scenarios[@]}"; do
  echo "phase0: $scenario"
  sysbench_cleanup "$scenario"
  sysbench_prepare "$scenario" 10000

  before="$(sysbench_handler_counters)"
  output="$(sysbench_run_one "$scenario" 1 10000)"
  after="$(sysbench_handler_counters)"

  tx="$(printf '%s' "$output" | awk '/transactions:/ { gsub("\\(", "", $3); print $3; exit }')"
  : "${tx:=0}"

  # Diff each counter via two AWK passes (per-key delta), then divide
  # by tx. Emits JSON.
  delta="$(
    paste <(printf '%s\n' "$before") <(printf '%s\n' "$after") \
      | awk -F'[=\t]' -v tx="$tx" 'tx>0 && $1==$3 { d=$4-$2; printf "  \"%s\": %g,\n", $1, d/tx }'
  )"
  results+=("\"$scenario\": {
$delta
  \"_transactions\": $tx
  }")
done

{
  echo "{"
  for ((i=0; i<${#results[@]}; i++)); do
    printf '  %s' "${results[$i]}"
    if (( i < ${#results[@]} - 1 )); then echo ","; else echo; fi
  done
  echo "}"
} > "$out"

echo "phase0: wrote $out"
