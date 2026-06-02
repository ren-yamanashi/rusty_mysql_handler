# Copyright (C) 2026 ren-yamanashi
# shellcheck shell=bash
#
# mysqld lifecycle helpers: container start, ready-wait, plugin install,
# default-engine switching, Handler_% counter capture.

mysql_exec() {
  docker exec -i -e MYSQL_PWD="$SYSBENCH_ROOT_PW" "$SYSBENCH_CONTAINER" \
    mysql -uroot --batch --skip-column-names "$@"
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
  docker logs "$SYSBENCH_CONTAINER" 2>&1 | tail -50 >&2
  return 1
}

sysbench_start_mysqld() {
  docker rm -f "$SYSBENCH_CONTAINER" >/dev/null 2>&1 || true
  docker run -d --name "$SYSBENCH_CONTAINER" \
    --cpus="$SYSBENCH_CPUS" \
    --memory="$SYSBENCH_MEMORY" \
    -e MYSQL_ROOT_PASSWORD="$SYSBENCH_ROOT_PW" \
    "$SYSBENCH_IMAGE" \
    --max-heap-table-size=4G \
    --innodb-buffer-pool-size=128M \
    >/dev/null
  sysbench_wait_mysqld
  mysql_exec < tests/sysbench/prepare.sql
}

sysbench_stop_mysqld() {
  docker rm -f "$SYSBENCH_CONTAINER" >/dev/null 2>&1 || true
}

# Set the default storage engine. New tables created without an explicit
# ENGINE clause use this. Matrix script calls before each `sysbench
# prepare` so the test tables land on the engine under measurement.
sysbench_set_engine() {
  mysql_query "SET GLOBAL default_storage_engine = '$1';"
}

# Capture the named Handler_% session counters as a flat KEY=VALUE list
# for stdin / stdout consumption. The phase0 capture diffs two such
# snapshots.
sysbench_handler_counters() {
  mysql_query "SHOW GLOBAL STATUS LIKE 'Handler_%'" | awk '{print $1"="$2}'
}
