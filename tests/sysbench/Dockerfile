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

# Perf-test image. Reuses the e2e Dockerfile's `builder` stage by
# pulling a locally-tagged image (built first via `docker build --target
# builder -t rusty-plugin-build:local -f tests/e2e/Dockerfile .`).
# The Makefile target wires both builds in order.

FROM rusty-plugin-build:local AS plugin

FROM mysql:8.4.9

COPY --from=plugin \
     /workspace/target/release/libengine.so \
     /usr/lib64/mysql/plugin/ha_rusty.so

# Pinned to the EPEL-9 snapshot at authorship; bump deliberately and
# re-declare in the next canonical session's RESULTS.md Environment header.
RUN microdnf install -y oracle-epel-release-el9-1.0-2.el9 \
    && microdnf install -y \
        sysbench-1.0.20-15.el9 \
        python3-3.9.21-2.0.1.el9 \
        jq-1.6-17.el9 \
    && microdnf clean all

COPY aggregate.py /usr/local/bin/aggregate.py
