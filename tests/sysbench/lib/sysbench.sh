# Copyright (C) 2026 ren-yamanashi
# shellcheck shell=bash
#
# sysbench command wrappers. `prepare` / `run` / `cleanup` plus a tiny
# parser that pulls tps / latency from sysbench's text output. The
# aggregate.py script consumes the per-trial JSON these wrappers emit.

sysbench_args() {
  echo \
    --mysql-host=127.0.0.1 \
    --mysql-user=root \
    --mysql-password="$SYSBENCH_ROOT_PW" \
    --mysql-db=sbtest \
    --db-driver=mysql \
    --tables=1
}

sysbench_in_container() {
  docker exec -i "$SYSBENCH_CONTAINER" sysbench "$@"
}

sysbench_prepare() {
  local scenario="$1" dataset="$2"
  # shellcheck disable=SC2046
  sysbench_in_container "$scenario" $(sysbench_args) \
    --table-size="$dataset" prepare >/dev/null
}

sysbench_cleanup() {
  local scenario="$1"
  # shellcheck disable=SC2046
  sysbench_in_container "$scenario" $(sysbench_args) cleanup >/dev/null 2>&1 || true
}

sysbench_run_one() {
  local scenario="$1" threads="$2" dataset="$3"
  # shellcheck disable=SC2046
  sysbench_in_container "$scenario" $(sysbench_args) \
    --table-size="$dataset" \
    --threads="$threads" \
    --warmup-time="$SYSBENCH_WARMUP" \
    --time="$SYSBENCH_TIME" \
    run
}

# Extract tps + latency p50/p95/p99 from sysbench's text report. Returns
# JSON-shaped output on stdout for aggregate.py to consume.
sysbench_parse_run() {
  awk '
    /transactions:/ { tps=$3 }
    /^[[:space:]]*min:/ { min=$2 }
    /^[[:space:]]*avg:/ { avg=$2 }
    /^[[:space:]]*max:/ { max=$2 }
    /95th percentile:/ { p95=$3 }
    /^[[:space:]]*sum:/ { sum=$2 }
    END {
      printf "{\"tps\":%s,\"latency_min\":%s,\"latency_avg\":%s,\"latency_max\":%s,\"latency_p95\":%s,\"latency_sum\":%s}\n",
        tps?tps:"null", min?min:"null", avg?avg:"null", max?max:"null",
        p95?p95:"null", sum?sum:"null"
    }
  '
}

# Convert MEMORY-engine index to BTREE so the rusty vs MEMORY comparison
# stays apples-to-apples on big-O behaviour (per the perf design's D4).
sysbench_alter_memory_index() {
  mysql_query "ALTER TABLE sbtest.sbtest1 DROP INDEX k_1, ADD INDEX k_1 (k) USING BTREE;"
}
