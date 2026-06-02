#!/usr/bin/env bash
# Copyright (C) 2026 ren-yamanashi
#
# (license trimmed; full text in tests/e2e/run.sh)

# Dispatcher: `phase0` captures per-scenario callback counts, `matrix`
# runs the L2 matrix. Build the harness image with the Makefile target
# before invoking either subcommand.

set -euo pipefail
cd "$(dirname "$0")/../.."

cmd="${1:-}"
case "$cmd" in
  phase0)
    exec bash tests/sysbench/phase0.sh
    ;;
  matrix)
    exec bash tests/sysbench/matrix.sh
    ;;
  *)
    echo "usage: $0 phase0|matrix" >&2
    exit 2
    ;;
esac
