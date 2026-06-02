#!/usr/bin/env bash
# Copyright (C) 2026 ren-yamanashi
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License, version 2.0,
# as published by the Free Software Foundation.
#
# This program is designed to work with certain software (including
# but not limited to OpenSSL) that is licensed under separate terms,
# as designated in a particular file or component or in included license
# documentation. The authors of this program hereby grant you an additional
# permission to link the program and your derivative works with the
# separately licensed software that they have either included with
# the program or referenced in the documentation.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program; if not, see <https://www.gnu.org/licenses/>.

# Capture per-transaction `Yᵢ` callback counts for each scenario.
# Snapshots `Handler_%` global counters around a short sysbench run,
# diffs them, divides by transaction count, and emits per-scenario JSON.

set -euo pipefail

# shellcheck source=lib/env.sh
source tests/sysbench/lib/env.sh
# shellcheck source=lib/mysqld.sh
source tests/sysbench/lib/mysqld.sh
# shellcheck source=lib/sysbench.sh
source tests/sysbench/lib/sysbench.sh

mkdir -p "$SYSBENCH_OUTPUT_DIR"
rm -f "$SYSBENCH_OUTPUT_DIR/callback_profile.json"
out="$SYSBENCH_OUTPUT_DIR/callback_profile.json"

trap 'sysbench_stop_mysqld' EXIT
sysbench_start_mysqld
sysbench_set_engine RUSTY

scenarios=(oltp_point_select oltp_read_only oltp_read_write)
results=()

for scenario in "${scenarios[@]}"; do
  echo "callback-profile: $scenario"
  sysbench_cleanup "$scenario"
  sysbench_prepare "$scenario" 10000

  before="$(sysbench_handler_counters)"
  output="$(sysbench_run_one "$scenario" 1 10000 1)"
  after="$(sysbench_handler_counters)"

  # sysbench prints `transactions: 12345  (411.22 per sec.)`; `$2` is the count.
  tx="$(printf '%s' "$output" | awk '/transactions:/ { print $2; exit }')"
  : "${tx:=0}"
  if [[ "$tx" -eq 0 ]]; then
    echo "callback-profile: $scenario sysbench produced no transactions" >&2
    exit 1
  fi

  delta="$(
    paste <(printf '%s\n' "$before") <(printf '%s\n' "$after") \
      | awk -F'[=\t]' -v tx="$tx" '$1==$3 { d=$4-$2; printf "  \"%s\": %g,\n", $1, d/tx }'
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

echo "callback-profile: wrote $out"
