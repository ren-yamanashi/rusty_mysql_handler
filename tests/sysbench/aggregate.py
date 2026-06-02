#!/usr/bin/env python3
# Copyright (C) 2026 ren-yamanashi
#
# (license trimmed; full text in tests/e2e/run.sh)
#
# Aggregate per-cell trial JSON into median, stddev, and variance
# ratio per metric. Reads each per-cell file from argv, emits a single
# `matrix.json` shape:
#
#   {
#     "cells": [
#       {"engine": "...", "scenario": "...", "threads": N, "dataset": N,
#        "tps": {"median": ..., "stddev": ..., "ratio": ...},
#        "latency_avg": {...}, "latency_p95": {...}, ...},
#       ...
#     ]
#   }

import json
import os
import statistics
import sys


def stats(values):
    """Median, stddev, and stddev/median ratio. Skips None entries."""
    clean = [v for v in values if v is not None]
    if not clean:
        return {"median": None, "stddev": None, "ratio": None}
    median = statistics.median(clean)
    stddev = statistics.pstdev(clean) if len(clean) > 1 else 0.0
    ratio = (stddev / median) if median else None
    return {"median": median, "stddev": stddev, "ratio": ratio}


def aggregate_cell(cell):
    trials = cell.get("trials", [])
    metrics = ["tps", "latency_min", "latency_avg", "latency_max",
               "latency_p95", "latency_sum"]
    out = {
        "engine": cell["engine"],
        "scenario": cell["scenario"],
        "threads": cell["threads"],
        "dataset": cell["dataset"],
        "trial_count": len(trials),
    }
    for m in metrics:
        out[m] = stats([t.get(m) for t in trials])
    return out


def main():
    paths = [p for p in sys.argv[1:] if os.path.basename(p) != "matrix.json"
             and os.path.basename(p) != "phase0.json"]
    cells = []
    for p in paths:
        with open(p) as f:
            cells.append(aggregate_cell(json.load(f)))
    json.dump({"cells": cells}, sys.stdout, indent=2)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
