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

# Validate that a PR title follows the project's Conventional Commits
# convention. Reads the title from $PR_TITLE if set, otherwise from the
# GitHub event payload at $GITHUB_EVENT_PATH (pull_request.title).
# Exits non-zero with a `::error::` line readable by GitHub Actions on
# mismatch.

set -euo pipefail

readonly ALLOWED='feat|fix|chore|docs|test|ci|refactor|style|perf|build|revert'
# `^(prefix)(\(scope\))?(!)?: <subject>` mirrors the rule in
# .claude/rules/coding-style.md. The optional `!` marks a breaking
# change in Conventional Commits.
readonly PATTERN="^(${ALLOWED})(\\([^)]+\\))?!?: .+"

title="${PR_TITLE:-}"
if [[ -z "$title" ]]; then
  if [[ -z "${GITHUB_EVENT_PATH:-}" || ! -r "$GITHUB_EVENT_PATH" ]]; then
    echo "::error::PR_TITLE is empty and GITHUB_EVENT_PATH is not readable" >&2
    exit 2
  fi
  title="$(jq -r '.pull_request.title // empty' < "$GITHUB_EVENT_PATH")"
fi

if [[ -z "$title" ]]; then
  echo "::error::could not resolve PR title from PR_TITLE or event payload" >&2
  exit 2
fi

if [[ ! "$title" =~ $PATTERN ]]; then
  {
    echo "::error::PR title must follow Conventional Commits."
    echo "  Got: \"$title\""
    echo "  Allowed prefixes: ${ALLOWED//|/, }"
    echo "  Examples:"
    echo "    feat: add transaction commit hook"
    echo "    fix(handler): position writes correct ref after index scan"
    echo "    refactor!: drop deprecated FFI symbols"
  } >&2
  exit 1
fi

echo "PR title OK: $title"
