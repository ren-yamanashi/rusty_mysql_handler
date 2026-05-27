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

# Smoke-test image: compiles the engine cdylib against a prebuilt, configured
# mysql-server tree downloaded from a GitHub Release asset (produced by
# .github/workflows/publish-mysql-base.yml from tests/e2e/Dockerfile.base), then
# bakes the plugin into a mysql:8.4 runtime container. Used by tests/e2e/run.sh
# to run end-to-end INSTALL PLUGIN / SELECT verification without polluting the
# host mysqld. Only the Rust + C++ shim are compiled here; the heavy MySQL
# clone + cmake configure is done once at publish time and reused across runs.

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

# Configured mysql-server tree (source + cmake build dir) as a release tarball.
# Pinned to the mysql-8.4.9 base release; override to test a different base.
ARG MYSQL_BASE_URL=https://github.com/ren-yamanashi/rusty_mysql_handler/releases/download/mysql-base-8.4.9/mysql-build-base-8.4.9.tar.gz
# Expected SHA-256 of the tarball, pinned in-repo so a re-uploaded release asset
# cannot silently swap the build base. To (re)publish the base, run the
# publish-mysql-base.yml workflow (workflow_dispatch), then commit the digest it
# prints here. Use the 64-hex digest only — not the whole `<hash>  <file>` line
# of the .sha256 asset.
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
