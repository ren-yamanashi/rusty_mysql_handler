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

# Snapshot of the canonical-session environment so the RESULTS.md
# Environment header can be reconstructed from a workflow artifact.

set -euo pipefail

# shellcheck source=lib/env.sh
source tests/sysbench/lib/env.sh

mkdir -p "$SYSBENCH_OUTPUT_DIR"
out="$SYSBENCH_OUTPUT_DIR/_env.txt"

{
  echo "=== runner ==="
  uname -srm
  grep 'model name' /proc/cpuinfo | head -1 || true
  nproc
  free -h | head -2
  echo "=== docker ==="
  docker version --format '{{.Server.Version}}'
  echo "=== rust ==="
  rustc --version 2>/dev/null || echo "(rustc not on host)"
  echo "=== plugin sha ==="
  git rev-parse HEAD
} > "$out"
cat "$out"
