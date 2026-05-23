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

cd "$(dirname "$0")"
cd ../

LICENSE_TEMPLATE="scripts/license-header.txt"
LICENSE_PATTERN=$(head -1 "$LICENSE_TEMPLATE")

comment_prefix() {
  case "$1" in
    *.rs|*.cc|*.cpp|*.h|*.c)          echo "//" ;;
    *.sh|*.bash)                      echo "#"  ;;
    Makefile|*.mk)                    echo "#"  ;;
  esac
}

build_header() {
  local prefix="$1"
  while IFS= read -r line; do
    if [[ -z "$line" ]]; then
      echo "$prefix"
    else
      echo "$prefix $line"
    fi
  done < "$LICENSE_TEMPLATE"
}

missing=()

while IFS= read -r file; do
  [[ -z "$file" ]] && continue
  [[ -z $(comment_prefix "$file") ]] && continue

  if ! git show ":$file" 2>/dev/null | head -n 15 | grep -q "$LICENSE_PATTERN"; then
    missing+=("$file")
  fi
done < <(git diff --cached --name-only --diff-filter=ACM)

[[ ${#missing[@]} -eq 0 ]] && exit 0

if [[ "${1:-}" != "--fix" ]]; then
  echo "ERROR: GPL-2.0 license header missing:"
  printf "  %s\n" "${missing[@]}"
  exit 1
fi

for file in "${missing[@]}"; do
  license_block=$(build_header "$(comment_prefix "$file")")

  if head -1 "$file" | grep -q '^#!'; then
    { head -1 "$file"; echo "$license_block"; echo ""; tail -n +2 "$file"; } > "$file.tmp"
  else
    { echo "$license_block"; echo ""; cat "$file"; } > "$file.tmp"
  fi

  mv "$file.tmp" "$file"
  git add "$file"
  echo "  $file"
done
