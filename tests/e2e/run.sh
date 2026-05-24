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

trap 'docker rm -f "$CONTAINER" >/dev/null 2>&1 || true' EXIT

mysql() { docker exec -i "$CONTAINER" mysql -uroot -proot "$@"; }

docker build -f tests/e2e/Dockerfile -t "$IMAGE" .
docker run -d --name "$CONTAINER" -e MYSQL_ROOT_PASSWORD=root "$IMAGE" >/dev/null

echo "e2e: waiting for mysqld..."
for _ in {1..60}; do
  docker exec "$CONTAINER" mysqladmin ping -uroot -proot --silent &>/dev/null && break
  sleep 2
done

mysql -e "INSTALL PLUGIN rusty SONAME 'ha_rusty.so'; CREATE DATABASE e2e;"

LAST="$(mysql --batch --skip-column-names e2e < tests/e2e/test.sql \
  | awk 'NF{last=$0} END{print last}')"
[[ "$LAST" == "3" ]] || {
  echo "e2e: expected last non-empty line = 3, got: $LAST" >&2
  exit 1
}

echo "e2e test passed"
