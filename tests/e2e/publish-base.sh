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

# Publishes the configured mysql-server tree as a GitHub Release asset.
# Per-arch: ARCH=x86_64|arm64 names the asset.

set -euo pipefail
cd "$(dirname "$0")/../.."

VERSION="${MYSQL_VERSION:?MYSQL_VERSION must be set}"
ARCH="${ARCH:?ARCH must be set (x86_64 or arm64)}"
TAG="mysql-base-${VERSION}"
# x86_64 keeps the legacy unsuffixed name so the in-repo Dockerfile pin keeps
# working; arm64 (added with multi-arch support) carries the arch suffix.
if [[ "$ARCH" == "x86_64" ]]; then
  ASSET="mysql-build-base-${VERSION}.tar.gz"
else
  ASSET="mysql-build-base-${VERSION}-${ARCH}.tar.gz"
fi

docker build -f tests/e2e/Dockerfile.base -t mysql-base-builder tests/e2e

cid="$(docker create mysql-base-builder)"
trap 'docker rm -f "$cid" >/dev/null 2>&1 || true' EXIT

# Stage in a fresh dir: docker cp nests under an existing destination, which the
# checkout's empty mysql-server/ submodule mount point would trigger.
rm -rf base-stage && mkdir base-stage
docker cp "$cid:/workspace/mysql-server" base-stage/mysql-server
docker cp "$cid:/workspace/build" base-stage/build

tar czf "$ASSET" -C base-stage mysql-server build
sha256sum "$ASSET" | tee "$ASSET.sha256"

if gh release view "$TAG" >/dev/null 2>&1; then
  gh release upload "$TAG" "$ASSET" "$ASSET.sha256" --clobber
else
  gh release create "$TAG" "$ASSET" "$ASSET.sha256" \
    --prerelease \
    --title "MySQL build base ${VERSION}" \
    --notes "Configured mysql-server tree consumed by tests/e2e/Dockerfile. Build artifact, not a product release."
fi
