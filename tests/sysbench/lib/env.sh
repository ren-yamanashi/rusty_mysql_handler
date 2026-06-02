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

# Common environment for sysbench harness scripts. Sourced by
# callback_profile.sh and matrix.sh through run.sh.

SYSBENCH_PLUGIN_IMAGE="${SYSBENCH_PLUGIN_IMAGE:-rusty-plugin-build:local}"
SYSBENCH_IMAGE="${SYSBENCH_IMAGE:-rusty-sysbench:local}"
SYSBENCH_CONTAINER="${SYSBENCH_CONTAINER:-rusty-sysbench-$$}"
SYSBENCH_ROOT_PW="${SYSBENCH_ROOT_PW:-root}"

# Per-cell tunables. Overridable for a short smoke run.
SYSBENCH_WARMUP="${SYSBENCH_WARMUP:-10}"
SYSBENCH_TIME="${SYSBENCH_TIME:-30}"
SYSBENCH_TRIALS="${SYSBENCH_TRIALS:-3}"
SYSBENCH_CPUS="${SYSBENCH_CPUS:-4}"
SYSBENCH_MEMORY="${SYSBENCH_MEMORY:-4g}"

# Output dir for per-cell JSON. Cleaned at run start by each
# subcommand so a re-run does not fold stale cells into the
# aggregation.
SYSBENCH_OUTPUT_DIR="${SYSBENCH_OUTPUT_DIR:-tests/sysbench/output}"
