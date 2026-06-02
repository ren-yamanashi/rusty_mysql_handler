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

# Run the OLTP matrix: N trials per (engine, scenario, threads, dataset)
# cell, write per-cell JSON, aggregate to matrix.json.

set -euo pipefail

# shellcheck source=lib/env.sh
source tests/sysbench/lib/env.sh
# shellcheck source=lib/mysqld.sh
source tests/sysbench/lib/mysqld.sh
# shellcheck source=lib/sysbench.sh
source tests/sysbench/lib/sysbench.sh

mkdir -p "$SYSBENCH_OUTPUT_DIR"
# Drop stale cells so the aggregation glob does not pick up prior runs.
find "$SYSBENCH_OUTPUT_DIR" -maxdepth 1 -name 'RUSTY-*.json' -delete
find "$SYSBENCH_OUTPUT_DIR" -maxdepth 1 -name 'MEMORY-*.json' -delete
rm -f "$SYSBENCH_OUTPUT_DIR/matrix.json"

trap 'sysbench_stop_mysqld' EXIT
sysbench_start_mysqld

engines=(RUSTY MEMORY)
scenarios=(oltp_point_select oltp_read_only oltp_read_write)
threads_list=(1 4 16)
datasets=(10000 100000)

cell_count=0
cell_total=$(( ${#engines[@]} * ${#scenarios[@]} * ${#threads_list[@]} * ${#datasets[@]} ))

for engine in "${engines[@]}"; do
  sysbench_set_engine "$engine"
  for scenario in "${scenarios[@]}"; do
    for threads in "${threads_list[@]}"; do
      for dataset in "${datasets[@]}"; do
        cell_count=$((cell_count + 1))
        cell="$engine-$scenario-${threads}t-$dataset"
        echo "matrix: [$cell_count/$cell_total] $cell"

        sysbench_cleanup "$scenario"
        sysbench_prepare "$scenario" "$dataset"
        if [[ "$engine" == "MEMORY" ]]; then
          sysbench_alter_memory_index
        fi
        mysql_query "SELECT 1;" >/dev/null

        trials_json=()
        for trial in $(seq 1 "$SYSBENCH_TRIALS"); do
          output="$(sysbench_run_one "$scenario" "$threads" "$dataset" "$trial")"
          parsed="$(printf '%s' "$output" | sysbench_parse_run)"
          trials_json+=("$parsed")
        done

        cell_file="$SYSBENCH_OUTPUT_DIR/$cell.json"
        {
          echo "{"
          echo "  \"engine\": \"$engine\","
          echo "  \"scenario\": \"$scenario\","
          echo "  \"threads\": $threads,"
          echo "  \"dataset\": $dataset,"
          echo "  \"trials\": ["
          for ((i=0; i<${#trials_json[@]}; i++)); do
            printf '    %s' "${trials_json[$i]}"
            if (( i < ${#trials_json[@]} - 1 )); then echo ","; else echo; fi
          done
          echo "  ]"
          echo "}"
        } > "$cell_file"
      done
    done
  done
done

echo "matrix: wrote $cell_total cell files to $SYSBENCH_OUTPUT_DIR"

cells_glob=("$SYSBENCH_OUTPUT_DIR"/RUSTY-*.json "$SYSBENCH_OUTPUT_DIR"/MEMORY-*.json)
jq -s '.' "${cells_glob[@]}" \
  | docker exec -i "$SYSBENCH_CONTAINER" python3 /usr/local/bin/aggregate.py \
  > "$SYSBENCH_OUTPUT_DIR/matrix.json"
echo "matrix: aggregated → $SYSBENCH_OUTPUT_DIR/matrix.json"
