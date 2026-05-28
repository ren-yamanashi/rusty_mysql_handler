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

set -euo pipefail
cd "$(dirname "$0")/../.."

# Builds tests/e2e/Dockerfile (which pulls the prebuilt mysql base from a
# Release asset, so no local submodule is needed) and runs the smoke test.

IMAGE="rusty-mysql-handler-e2e"
CONTAINER="rusty-e2e-$$"
ROOT_PW="root"

trap 'docker rm -f "$CONTAINER" >/dev/null 2>&1 || true' EXIT

# MYSQL_PWD avoids the "Using a password on the command line interface" warning
# that `-p<pw>` triggers. Container-scoped, never reaches the host process list.
mysql() { docker exec -i -e MYSQL_PWD="$ROOT_PW" "$CONTAINER" mysql -uroot "$@"; }
# The mysql:8.4 entrypoint runs a throwaway temporary server to initialise the
# data dir before starting the real one, and it sets the root password during
# that phase — so an authenticated `SELECT 1` can succeed against the temp
# server, break the wait loop early, and then race its shutdown before the real
# server is up (observed as a false "not ready"). The temp server runs with
# `--skip-networking`; the real server does not. Gate readiness on
# `@@skip_networking` so the loop only trips for the real server.
ping_mysqld() {
  local sn
  sn="$(docker exec -e MYSQL_PWD="$ROOT_PW" "$CONTAINER" \
    mysql -uroot --batch --skip-column-names -e "SELECT @@skip_networking" 2>/dev/null)" || return 1
  [[ "$sn" == "0" ]]
}

docker build -f tests/e2e/Dockerfile -t "$IMAGE" .
docker run -d --name "$CONTAINER" -e MYSQL_ROOT_PASSWORD="$ROOT_PW" "$IMAGE" >/dev/null

for _ in $(seq 1 60); do
  if ping_mysqld; then
    break
  fi
  sleep 2
done
if ! ping_mysqld; then
  echo "e2e: mysqld did not become ready within 120s; container state + log tail:" >&2
  docker ps -a --filter "name=$CONTAINER" >&2
  docker logs "$CONTAINER" 2>&1 | tail -100 >&2
  exit 1
fi

mysql -e "INSTALL PLUGIN rusty SONAME 'ha_rusty.so';" || {
  echo "e2e: INSTALL PLUGIN failed; mysqld log tail:" >&2
  docker logs "$CONTAINER" 2>&1 | tail -50 >&2
  exit 1
}
mysql -e "CREATE DATABASE e2e;"

LAST="$(mysql --batch --skip-column-names e2e < tests/e2e/test.sql \
  | awk 'NF{last=$0} END{print last}')"
[[ "$LAST" == "3" ]] || {
  echo "e2e: expected last non-empty line = 3, got: $LAST" >&2
  exit 1
}

echo "e2e test passed"
