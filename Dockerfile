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

# Smoke-test image: builds the engine cdylib as a Linux ELF and bakes it into
# a mysql:8.4 runtime container. Used by e2e/smoke/run.sh to run end-to-end
# INSTALL PLUGIN / SELECT verification without polluting the host mysqld.

FROM rust:1.95.0-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
      cmake \
      make \
      clang \
      libclang-dev \
      g++ \
      pkg-config \
      bison \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

# Stage the mysql-server submodule first so the Boost-download + cmake-configure
# layer survives across plugin source edits.
COPY mysql-server ./mysql-server
RUN mkdir -p build/mysql \
    && cd build/mysql \
    && cmake /workspace/mysql-server \
        -DDOWNLOAD_BOOST=1 \
        -DWITH_BOOST=/workspace/mysql-server/extra/boost \
        -DWITHOUT_SERVER=ON \
        -DCMAKE_BUILD_TYPE=Release

COPY Cargo.toml Cargo.lock build.rs rust-toolchain.toml ./
COPY src ./src
COPY shim ./shim
COPY examples ./examples

RUN MYSQL_HANDLER_FROM_SOURCE=1 cargo build --release -p engine

FROM mysql:8.4.6

COPY --from=builder \
     /workspace/target/release/libengine.so \
     /usr/lib64/mysql/plugin/ha_rusty.so
