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

# Smoke-test image: compiles the engine cdylib against the prebuilt mysql-server
# tree (downloaded from a Release asset) and bakes the plugin into a mysql:8.4
# runtime. Driven by tests/e2e/run.sh; build base is documented in CONTRIBUTING.md.

FROM rust:1.95.0-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
      cmake \
      make \
      clang \
      libclang-dev \
      g++ \
      pkg-config \
      bison \
      libssl-dev \
      libncurses-dev \
      libtirpc-dev \
      zlib1g-dev \
      curl \
      ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

ARG MYSQL_BASE_URL=https://github.com/ren-yamanashi/rusty_mysql_handler/releases/download/mysql-base-8.4.9/mysql-build-base-8.4.9.tar.gz
# Verified against this pin (fail-closed); updated when the base is re-published.
ARG MYSQL_BASE_SHA256=e15fb561edccf787e9e5280f3e1f2b9538bc1efd8845a112abf406d00955ac96
RUN test -n "$MYSQL_BASE_SHA256" \
      || { echo "MYSQL_BASE_SHA256 unset; pin it from publish-mysql-base.yml" >&2; exit 1; }; \
    curl -fsSL "$MYSQL_BASE_URL" -o base.tar.gz \
    && echo "$MYSQL_BASE_SHA256  base.tar.gz" | sha256sum -c - \
    && tar xzf base.tar.gz \
    && rm base.tar.gz

# build.rs (MYSQL_HANDLER_FROM_SOURCE) reads these to locate the source and the
# cmake-configured build dir extracted above.
ENV MYSQL_SOURCE_DIR=/workspace/mysql-server \
    MYSQL_BUILD_DIR=/workspace/build/mysql

COPY Cargo.toml Cargo.lock build.rs rust-toolchain.toml ./
COPY src ./src
COPY shim ./shim
COPY examples ./examples

RUN MYSQL_HANDLER_FROM_SOURCE=1 cargo build --release -p engine

FROM mysql:8.4.9

COPY --from=builder \
     /workspace/target/release/libengine.so \
     /usr/lib64/mysql/plugin/ha_rusty.so
