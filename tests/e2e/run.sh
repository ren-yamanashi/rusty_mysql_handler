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

[[ -f mysql-server/CMakeLists.txt ]] || {
  echo "e2e: mysql-server submodule missing. run: make setup" >&2
  exit 1
}

IMAGE="rusty-mysql-handler-e2e"
CONTAINER="rusty-e2e-$$"
ROOT_PW="root"

trap 'docker rm -f "$CONTAINER" >/dev/null 2>&1 || true' EXIT

# MYSQL_PWD avoids the "Using a password on the command line interface" warning
# that `-p<pw>` triggers. Container-scoped, never reaches the host process list.
mysql() { docker exec -i -e MYSQL_PWD="$ROOT_PW" "$CONTAINER" mysql -uroot "$@"; }
ping_mysqld() {
  docker exec -e MYSQL_PWD="$ROOT_PW" "$CONTAINER" mysqladmin ping -uroot --silent &>/dev/null
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
