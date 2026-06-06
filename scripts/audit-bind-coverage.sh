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

# Cross-reference docs/api/{handler,handlerton}.md against the bind surface
# in mysql-handler/src and mysql-handler/shim. Emits docs/api/coverage.md.
#
# Usage: scripts/audit-bind-coverage.sh > docs/api/coverage.md
#
# A row is "bound" only when all three layers carry the method name:
#   T (trait)    — fn <name>( in src/engine.rs or src/hton{.rs,/*.rs}
#   C (callback) — rust__handler__<name>( in src/handler/*.rs
#                  rust__hton__<name>(    in src/hton/*.rs
#   S (shim)     — RustHandlerBase::<name>( in shim/handler_*.cc
#                  rust__hton__<name>(      in shim/hton_*.cc
#
# Exits non-zero when any "missing" or "partial" row remains.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/mysql-handler/src"
SHIM="$ROOT/mysql-handler/shim"

mark() { [[ -n $1 ]] && printf '%s' '✓' || printf '%s' '✗'; }

# Look up the human-applied Notes column ($1=section, $2=method) from the
# existing coverage.md so a rerun does not strip annotations.
NOTES_FILE=""
lookup_note() {
  [[ -z $NOTES_FILE || ! -f $NOTES_FILE ]] && { printf ''; return; }
  awk -F '\t' -v sec="$1" -v n="$2" '
    $1==sec && $2==n { print $3; exit }
  ' "$NOTES_FILE"
}

# Comma-joined basenames of files matching fixed string $1 in remaining args.
matches_in() {
  local pat=$1
  shift
  [[ $# -eq 0 ]] && { printf ''; return; }
  local hits
  hits=$(grep -lF -- "$pat" "$@" 2>/dev/null || true)
  [[ -z $hits ]] && { printf ''; return; }
  printf '%s' "$hits" | xargs -n1 basename | sort -u | paste -sd, -
}

# Read TAB-separated (name, line, pv) from a docs/api/*.md table.
parse_rows() {
  awk -F '|' '
    /^\| *[0-9]+ *\| *`[^`]+` *\|/ {
      name = $3; gsub(/[ `]/, "", name)
      line = $4; gsub(/ /, "", line)
      pv   = $5; gsub(/ /, "", pv)
      print name "\t" line "\t" pv
    }
  ' "$1"
}

# Walk handler rows, emit table rows to stdout; emit "total bound" to $1.
audit_handler_rows() {
  local stats_out=$1
  local trait_file="$SRC/engine.rs"
  local cb_files=( "$SRC/handler"/*.rs )
  local shim_files=( "$SHIM"/handler_*.cc "$SHIM"/binding.cc )
  local total=0 bound=0
  while IFS=$'\t' read -r name line pv; do
    [[ -z $name ]] && continue
    total=$((total + 1))
    local t c s
    t=$(matches_in "fn $name(" "$trait_file")
    c=$(matches_in "rust__handler__${name}(" "${cb_files[@]}")
    s=$(matches_in "RustHandlerBase::${name}(" "${shim_files[@]}")
    local count=0 paths=()
    [[ -n $t ]] && { count=$((count + 1)); paths+=("$t"); }
    [[ -n $c ]] && { count=$((count + 1)); paths+=("$c"); }
    [[ -n $s ]] && { count=$((count + 1)); paths+=("$s"); }
    local status
    case $count in
      3) status="bound"; bound=$((bound + 1)) ;;
      0) status="missing" ;;
      *) status="partial" ;;
    esac
    local path_col note
    path_col=$(IFS=,; printf '%s' "${paths[*]:-}")
    note=$(lookup_note handler "$name")
    printf '| `%s` | %s | %s | %s | %s | %s | %s | %s |\n' \
      "$name" "$line" "$(mark "$t")" "$(mark "$c")" "$(mark "$s")" \
      "$status" "$path_col" "$note"
  done < <(parse_rows "$ROOT/docs/api/handler.md")
  printf '%d %d\n' "$total" "$bound" > "$stats_out"
}

# Walk handlerton rows, emit table rows to stdout; emit "total bound" to $1.
audit_hton_rows() {
  local stats_out=$1
  local trait_files=( "$SRC/hton.rs" "$SRC/hton"/*.rs )
  local cb_files=( "$SRC/hton"/*.rs )
  local shim_files=( "$SHIM"/hton_*.cc )
  local total=0 bound=0
  while IFS=$'\t' read -r name line pv; do
    [[ -z $name ]] && continue
    total=$((total + 1))
    local t c s
    t=$(matches_in "fn $name(" "${trait_files[@]}")
    c=$(matches_in "rust__hton__${name}(" "${cb_files[@]}")
    s=$(matches_in "rust__hton__${name}(" "${shim_files[@]}")
    local count=0 paths=()
    [[ -n $t ]] && { count=$((count + 1)); paths+=("$t"); }
    [[ -n $c ]] && { count=$((count + 1)); paths+=("$c"); }
    [[ -n $s ]] && { count=$((count + 1)); paths+=("$s"); }
    local status
    case $count in
      3) status="bound"; bound=$((bound + 1)) ;;
      0) status="missing" ;;
      *) status="partial" ;;
    esac
    local path_col note
    path_col=$(IFS=,; printf '%s' "${paths[*]:-}")
    note=$(lookup_note hton "$name")
    printf '| `%s` | %s | %s | %s | %s | %s | %s |\n' \
      "$name" "$(mark "$t")" "$(mark "$c")" "$(mark "$s")" \
      "$status" "$path_col" "$note"
  done < <(parse_rows "$ROOT/docs/api/handlerton.md")
  printf '%d %d\n' "$total" "$bound" > "$stats_out"
}

# Build TAB-separated (section, name, note) records from the existing
# coverage.md so future runs preserve human annotations.
build_notes_file() {
  local existing="$ROOT/docs/api/coverage.md"
  NOTES_FILE="$TMPDIR_AUDIT/notes.tsv"
  : > "$NOTES_FILE"
  [[ -f $existing ]] || return 0
  awk -F '|' '
    /^## handler — / { section="handler"; next }
    /^## handlerton — / { section="hton"; next }
    /^\| `[^`]+` \|/ && section != "" {
      name = $2; gsub(/[ `]/, "", name)
      notes = $(NF-1); sub(/^[ \t]+/, "", notes); sub(/[ \t]+$/, "", notes)
      print section "\t" name "\t" notes
    }
  ' "$existing" > "$NOTES_FILE"
}

TMPDIR_AUDIT=""
cleanup() { [[ -n "$TMPDIR_AUDIT" ]] && rm -rf "$TMPDIR_AUDIT"; }
trap cleanup EXIT

main() {
  TMPDIR_AUDIT=$(mktemp -d)
  local tmpdir=$TMPDIR_AUDIT
  local handler_table hton_table handler_stats hton_stats
  handler_table="$tmpdir/handler.tbl"
  hton_table="$tmpdir/hton.tbl"
  handler_stats="$tmpdir/handler.stats"
  hton_stats="$tmpdir/hton.stats"
  build_notes_file
  audit_handler_rows "$handler_stats" > "$handler_table"
  audit_hton_rows "$hton_stats" > "$hton_table"

  local handler_total handler_bound hton_total hton_bound
  read -r handler_total handler_bound < "$handler_stats"
  read -r hton_total hton_bound < "$hton_stats"

  cat <<'EOF'
<!--
Regenerate this file with `scripts/audit-bind-coverage.sh > docs/api/coverage.md`.
Annotations in the "Notes" column are preserved manually after each rerun;
the script overwrites everything else.
-->

# API Bind Coverage

Cross-reference between the upstream MySQL 8.4 handler / handlerton
surface (documented in [`handler.md`](handler.md) and
[`handlerton.md`](handlerton.md)) and the bindings under
`mysql-handler/src/` + `mysql-handler/shim/`.

## Columns

- **T / C / S** — presence in trait (T), `rust__*` callback (C), and
  shim override (S). `✓` if found, `✗` if not.
- **Status** — `bound` when T + C + S are all present;
  `partial` when 1 or 2 layers exist; `missing` when none do.
- **Bind path** — basenames of the files matched, for navigation.

EOF
  printf '## handler — %d / %d bound\n\n' "$handler_bound" "$handler_total"
  echo "| Method | handler.h Line | T | C | S | Status | Bind path | Notes |"
  echo "| ------ | -------------- | - | - | - | ------ | --------- | ----- |"
  cat "$handler_table"
  echo ""
  printf '## handlerton — %d / %d bound\n\n' "$hton_bound" "$hton_total"
  echo "| Callback | T | C | S | Status | Bind path | Notes |"
  echo "| -------- | - | - | - | ------ | --------- | ----- |"
  cat "$hton_table"
  echo ""

  local missing=$((handler_total - handler_bound + hton_total - hton_bound))
  if [[ $missing -gt 0 ]]; then
    printf '_%d rows are not fully bound. Review each row above and ' "$missing" >&2
    printf 'either close the gap or annotate Notes._\n' >&2
    return 1
  fi
}

main "$@"
