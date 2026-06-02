# Copyright (C) 2026 ren-yamanashi
# shellcheck shell=bash
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

# sysbench wrappers. All sysbench / mysql traffic flows through the
# client container; mysqld stays a clean `mysql:8.4.9` + plugin image.

sysbench_args() {
  echo \
    --mysql-host="$SYSBENCH_MYSQLD_CONTAINER" \
    --mysql-user=root \
    --mysql-password="$SYSBENCH_ROOT_PW" \
    --mysql-db=sbtest \
    --db-driver=mysql \
    --tables=1
}

sysbench_in_container() {
  docker exec -i "$SYSBENCH_CLIENT_CONTAINER" sysbench "$@"
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

# sysbench 1.0.20 (debian bookworm) lacks `--warmup-time` (added in
# 1.1.x); a short discarded pre-run primes caches when SYSBENCH_WARMUP
# > 0. `--rand-seed=$trial` keeps each (engine, scenario, threads,
# dataset, trial) tuple reproducible across sessions.
sysbench_run_one() {
  local scenario="$1" threads="$2" dataset="$3" trial="${4:-1}"
  if [[ "$SYSBENCH_WARMUP" -gt 0 ]]; then
    # shellcheck disable=SC2046
    sysbench_in_container "$scenario" $(sysbench_args) \
      --table-size="$dataset" \
      --threads="$threads" \
      --time="$SYSBENCH_WARMUP" \
      --rand-seed="$trial" \
      run >/dev/null
  fi
  # shellcheck disable=SC2046
  sysbench_in_container "$scenario" $(sysbench_args) \
    --table-size="$dataset" \
    --threads="$threads" \
    --time="$SYSBENCH_TIME" \
    --rand-seed="$trial" \
    run
}

# Emits `{"failed": true}` and exits 1 when sysbench produced no
# `transactions:` line, so the aggregator surfaces failure rather than
# silently averaging over nulls.
sysbench_parse_run() {
  awk '
    /transactions:/ { tps=$3; gsub("[()]", "", tps) }
    /^[[:space:]]*min:/ { min=$2 }
    /^[[:space:]]*avg:/ { avg=$2 }
    /^[[:space:]]*max:/ { max=$2 }
    /95th percentile:/ { p95=$3 }
    /^[[:space:]]*sum:/ { sum=$2 }
    END {
      if (!tps) {
        print "{\"failed\": true}"
        exit 1
      }
      printf "{\"tps\":%s,\"latency_min\":%s,\"latency_avg\":%s,\"latency_max\":%s,\"latency_p95\":%s,\"latency_sum\":%s}\n",
        tps, min?min:"null", avg?avg:"null", max?max:"null", p95?p95:"null", sum?sum:"null"
    }
  '
}

# Switch MEMORY's default HASH index to BTREE so big-O matches rusty's
# `BTreeMap`; otherwise MEMORY gets a constant-factor lead unrelated to FFI.
sysbench_alter_memory_index() {
  mysql_query "ALTER TABLE sbtest.sbtest1 DROP INDEX k_1, ADD INDEX k_1 (k) USING BTREE;"
}
