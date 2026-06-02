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

# Two-container lifecycle: mysqld + sysbench client on a private bridge.
# `mysql` and `sysbench` invocations go through the client container so
# the mysqld image stays a clean `mysql:8.4.9` + plugin.

mysql_exec() {
  docker exec -i -e MYSQL_PWD="$SYSBENCH_ROOT_PW" "$SYSBENCH_CLIENT_CONTAINER" \
    mysql -h "$SYSBENCH_MYSQLD_CONTAINER" -uroot --batch --skip-column-names "$@"
}

mysql_query() {
  mysql_exec -e "$1"
}

# mysql:8.4 entrypoint runs a temp server during initialisation that
# breaks naive `SELECT 1` readiness probes. Gate on `@@skip_networking`
# being 0, which is true only on the real server.
sysbench_ping_mysqld() {
  local sn
  sn="$(mysql_query "SELECT @@skip_networking" 2>/dev/null)" || return 1
  [[ "$sn" == "0" ]]
}

sysbench_wait_mysqld() {
  local i
  for i in $(seq 1 60); do
    if sysbench_ping_mysqld; then
      return 0
    fi
    sleep 2
  done
  echo "sysbench: mysqld did not become ready within 120s" >&2
  docker logs "$SYSBENCH_MYSQLD_CONTAINER" 2>&1 | tail -50 >&2
  return 1
}

sysbench_start_mysqld() {
  docker rm -f "$SYSBENCH_MYSQLD_CONTAINER" "$SYSBENCH_CLIENT_CONTAINER" \
    >/dev/null 2>&1 || true
  docker network rm "$SYSBENCH_NETWORK" >/dev/null 2>&1 || true
  docker network create "$SYSBENCH_NETWORK" >/dev/null
  mkdir -p "$SYSBENCH_OUTPUT_DIR"
  local abs_output
  abs_output="$(cd "$SYSBENCH_OUTPUT_DIR" && pwd)"
  docker run -d --name "$SYSBENCH_MYSQLD_CONTAINER" \
    --network "$SYSBENCH_NETWORK" \
    --cpus="$SYSBENCH_CPUS" \
    --memory="$SYSBENCH_MEMORY" \
    -e MYSQL_ROOT_PASSWORD="$SYSBENCH_ROOT_PW" \
    "$SYSBENCH_MYSQLD_IMAGE" \
    --max-heap-table-size=4G \
    --innodb-buffer-pool-size=128M \
    >/dev/null
  docker run -d --name "$SYSBENCH_CLIENT_CONTAINER" \
    --network "$SYSBENCH_NETWORK" \
    -v "$abs_output:/output" \
    "$SYSBENCH_CLIENT_IMAGE" \
    sleep infinity \
    >/dev/null
  sysbench_wait_mysqld
  mysql_exec < tests/sysbench/prepare.sql
}

sysbench_stop_mysqld() {
  docker rm -f "$SYSBENCH_MYSQLD_CONTAINER" "$SYSBENCH_CLIENT_CONTAINER" \
    >/dev/null 2>&1 || true
  docker network rm "$SYSBENCH_NETWORK" >/dev/null 2>&1 || true
}

sysbench_set_engine() {
  mysql_query "SET GLOBAL default_storage_engine = '$1';"
}

# `Handler_%` as KEY=VALUE. Global (each mysql invocation opens a fresh
# connection so session counters would be zero), so background mysqld
# activity between snapshots is part of the noise floor.
sysbench_handler_counters() {
  mysql_query "SHOW GLOBAL STATUS LIKE 'Handler_%'" | awk '{print $1"="$2}'
}
