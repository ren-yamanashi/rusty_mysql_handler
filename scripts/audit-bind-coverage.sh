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
# in mysql-handler/src and mysql-handler/shim. Writes docs/api/coverage.md
# in place, preserving the human-applied Notes column from the previous run.
# See docs/api/coverage.md for the rendered table and the per-status legend
# (bound / intentionally unbound / deferred / needs review).
#
# Usage: scripts/audit-bind-coverage.sh
# Exits 0 when every row is bound or intentionally unbound, 1 when any row
# is still deferred or needs review.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/mysql-handler/src"
SHIM="$ROOT/mysql-handler/shim"

mark() { [[ -n $1 ]] && printf '%s' '✓' || printf '%s' '✗'; }

# Resolve the final status from (T, C, S marks, note text).
classify_final() {
  local t=$1 c=$2 s=$3 note=$4
  if [[ -n $t && -n $c && -n $s ]]; then
    printf 'bound'
    return
  fi
  case "$note" in
    "Deferred:"*)               printf 'deferred' ;;
    *"Intentionally"*)          printf 'intentionally unbound' ;;
    *"fully bound"*|"Bound via"*|"FFI-only binding"*|"Trait renamed"*)
                                printf 'bound' ;;
    *)                          printf 'needs review' ;;
  esac
}

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

# Walk rows from $2 (a docs/api/*.md path); emit table rows to stdout and
# "total bound unbound needs_review" to $1. $3 selects handler vs hton
# globs and the row format (handler keeps the source-line column).
audit_rows() {
  local stats_out=$1 doc=$2 kind=$3
  local trait_files cb_files shim_files cb_prefix shim_prefix
  if [[ $kind == handler ]]; then
    trait_files=( "$SRC/engine.rs" )
    cb_files=( "$SRC/handler"/*.rs )
    shim_files=( "$SHIM"/handler_*.cc "$SHIM"/binding.cc )
    cb_prefix="rust__handler__"
    shim_prefix="RustHandlerBase::"
  else
    trait_files=( "$SRC/hton.rs" "$SRC/hton"/*.rs )
    cb_files=( "$SRC/hton"/*.rs )
    shim_files=( "$SHIM"/hton_*.cc )
    cb_prefix="rust__hton__"
    shim_prefix="rust__hton__"
  fi
  local total=0 bound=0 unbound=0 deferred=0 review=0
  while IFS=$'\t' read -r name line pv; do
    [[ -z $name ]] && continue
    total=$((total + 1))
    local t c s
    t=$(matches_in "fn $name(" "${trait_files[@]}")
    c=$(matches_in "${cb_prefix}${name}(" "${cb_files[@]}")
    s=$(matches_in "${shim_prefix}${name}(" "${shim_files[@]}")
    local paths=()
    [[ -n $t ]] && paths+=("$t")
    [[ -n $c ]] && paths+=("$c")
    [[ -n $s ]] && paths+=("$s")
    local note status path_col
    note=$(lookup_note "$kind" "$name")
    status=$(classify_final "$t" "$c" "$s" "$note")
    case "$status" in
      bound)                  bound=$((bound + 1)) ;;
      "intentionally unbound") unbound=$((unbound + 1)) ;;
      deferred)               deferred=$((deferred + 1)) ;;
      *)                      review=$((review + 1)) ;;
    esac
    path_col=$(IFS=,; printf '%s' "${paths[*]:-}")
    local line_col=""
    [[ $kind == handler ]] && line_col=" $line |"
    printf '| `%s` |%s %s | %s | %s | %s | %s | %s |\n' \
      "$name" "$line_col" "$(mark "$t")" "$(mark "$c")" "$(mark "$s")" \
      "$status" "$path_col" "$note"
  done < <(parse_rows "$doc")
  printf '%d %d %d %d %d\n' "$total" "$bound" "$unbound" "$deferred" "$review" > "$stats_out"
}

# Build TAB-separated (section, name, note) records from $1 (a snapshot of
# the previous coverage.md). Empty/non-existent snapshot is fine.
build_notes_file() {
  local snapshot=$1
  NOTES_FILE="$TMPDIR_AUDIT/notes.tsv"
  : > "$NOTES_FILE"
  [[ -s $snapshot ]] || return 0
  awk -F '|' '
    /^## handler — / { section="handler"; next }
    /^## handlerton — / { section="hton"; next }
    /^\| `[^`]+` \|/ && section != "" {
      name = $2; gsub(/[ `]/, "", name)
      notes = $(NF-1); sub(/^[ \t]+/, "", notes); sub(/[ \t]+$/, "", notes)
      print section "\t" name "\t" notes
    }
  ' "$snapshot" > "$NOTES_FILE"
}

TMPDIR_AUDIT=""
cleanup() { [[ -n "$TMPDIR_AUDIT" ]] && rm -rf "$TMPDIR_AUDIT"; }
trap cleanup EXIT

main() {
  TMPDIR_AUDIT=$(mktemp -d)
  local tmpdir=$TMPDIR_AUDIT
  local target="$ROOT/docs/api/coverage.md"
  local snapshot="$tmpdir/existing.md"
  [[ -f $target ]] && cp "$target" "$snapshot"
  local handler_table="$tmpdir/handler.tbl"
  local hton_table="$tmpdir/hton.tbl"
  local handler_stats="$tmpdir/handler.stats"
  local hton_stats="$tmpdir/hton.stats"
  build_notes_file "$snapshot"
  audit_rows "$handler_stats" "$ROOT/docs/api/handler.md" handler > "$handler_table"
  audit_rows "$hton_stats" "$ROOT/docs/api/handlerton.md" hton > "$hton_table"

  local handler_total handler_bound handler_unbound handler_deferred handler_review
  local hton_total hton_bound hton_unbound hton_deferred hton_review
  read -r handler_total handler_bound handler_unbound handler_deferred handler_review < "$handler_stats"
  read -r hton_total hton_bound hton_unbound hton_deferred hton_review < "$hton_stats"

  {
  cat <<'EOF'
<!--
Regenerate this file with `scripts/audit-bind-coverage.sh` (writes in place).
The script preserves the "Notes" column from the previous version of this
file via an internal snapshot, so human-applied annotations survive reruns;
every other column is recomputed.
-->

# API Bind Coverage

Cross-reference between the upstream MySQL 8.4 handler / handlerton
surface (documented in [`handler.md`](handler.md) and
[`handlerton.md`](handlerton.md)) and the bindings under
`mysql-handler/src/` + `mysql-handler/shim/`.

## Columns

- **T / C / S** — presence in trait (T), `rust__*` callback (C), and
  shim override (S). `✓` if found, `✗` if not.
- **Status** — verdict produced by combining the auto T/C/S detection
  with the Notes column. Possible values: `bound`,
  `intentionally unbound` (genuinely unbindable, not a placeholder),
  `deferred` (bind path known, follow-up tracked in the Notes), or
  `needs review` (annotation missing or ambiguous).
- **Bind path** — basenames of the files matched, for navigation.

EOF
  printf '## handler — %d bound, %d deferred, %d intentionally unbound (%d total)\n\n' \
    "$handler_bound" "$handler_deferred" "$handler_unbound" "$handler_total"
  echo "| Method | handler.h Line | T | C | S | Status | Bind path | Notes |"
  echo "| ------ | -------------- | - | - | - | ------ | --------- | ----- |"
  cat "$handler_table"
  echo ""
  printf '## handlerton — %d bound, %d deferred, %d intentionally unbound (%d total)\n\n' \
    "$hton_bound" "$hton_deferred" "$hton_unbound" "$hton_total"
  echo "| Callback | T | C | S | Status | Bind path | Notes |"
  echo "| -------- | - | - | - | ------ | --------- | ----- |"
  cat "$hton_table"
  echo ""

  } > "$target"

  local open=$((handler_deferred + hton_deferred + handler_review + hton_review))
  if [[ $open -gt 0 ]]; then
    printf '_%d rows are still open (deferred + needs review). ' "$open" >&2
    printf 'Bind them or refine the Notes annotation._\n' >&2
    return 1
  fi
}

main "$@"
