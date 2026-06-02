#!/usr/bin/env bash
# Copyright (C) 2026 ren-yamanashi
#
# (license trimmed; full text in tests/e2e/run.sh)

# L2 matrix — 36 cells × N trials, per-cell median + stddev via
# aggregate.py.

set -euo pipefail

# shellcheck source=lib/env.sh
source tests/sysbench/lib/env.sh
# shellcheck source=lib/mysqld.sh
source tests/sysbench/lib/mysqld.sh
# shellcheck source=lib/sysbench.sh
source tests/sysbench/lib/sysbench.sh

mkdir -p "$SYSBENCH_OUTPUT_DIR"

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
          output="$(sysbench_run_one "$scenario" "$threads" "$dataset")"
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
docker exec -i "$SYSBENCH_CONTAINER" python3 \
  < tests/sysbench/aggregate.py \
  > "$SYSBENCH_OUTPUT_DIR/matrix.json" <<< "$(cat "$SYSBENCH_OUTPUT_DIR"/*.json | jq -s '.')" || \
  python3 tests/sysbench/aggregate.py "$SYSBENCH_OUTPUT_DIR"/*.json \
    > "$SYSBENCH_OUTPUT_DIR/matrix.json"
echo "matrix: aggregated → $SYSBENCH_OUTPUT_DIR/matrix.json"
