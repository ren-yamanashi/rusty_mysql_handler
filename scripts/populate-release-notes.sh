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

# Populate the GitHub release body for the latest mysql-handler umbrella
# tag with the PR-link auto-notes that GitHub's
# `repos/{owner}/{repo}/releases/generate-notes` endpoint produces.
#
# Called from `.github/workflows/release-plz.yml` after `release-plz
# release` runs, because `changelog_update = false` leaves release-plz
# nothing to render into the body.

set -euo pipefail

: "${GH_TOKEN:?GH_TOKEN must be set}"
: "${GITHUB_REPOSITORY:?GITHUB_REPOSITORY must be set}"

git fetch --tags --force origin >/dev/null

new_tag=$(git tag --sort=-creatordate --list 'mysql-handler-v*' | head -n 1)
prev_tag=$(git tag --sort=-creatordate --list 'mysql-handler-v*' | sed -n 2p)

if [ -z "$new_tag" ]; then
  echo "no mysql-handler-v* tag found; nothing to populate"
  exit 0
fi

notes_args=(-f tag_name="$new_tag")
if [ -n "$prev_tag" ]; then
  notes_args+=(-f previous_tag_name="$prev_tag")
fi

notes=$(gh api -X POST \
  "repos/$GITHUB_REPOSITORY/releases/generate-notes" \
  "${notes_args[@]}" \
  --jq '.body')

gh release edit "$new_tag" --notes "$notes"
