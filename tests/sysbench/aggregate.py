#!/usr/bin/env python3
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
#
# Aggregate per-cell trial JSON into median, sample stddev, and the
# variance ratio per metric. Reads a JSON list of per-cell objects on
# stdin (matrix.sh slurps the per-cell files via `jq -s '.'`) and
# emits a single rollup on stdout.

import json
import statistics
import sys


def stats(values):
    """Median + sample stddev (`pstdev` would under-state spread at N=3)."""
    clean = [v for v in values if v is not None]
    if not clean:
        return {"median": None, "stddev": None, "ratio": None}
    median = statistics.median(clean)
    stddev = statistics.stdev(clean) if len(clean) > 1 else 0.0
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
        "failed_trials": sum(1 for t in trials if t.get("failed")),
    }
    for m in metrics:
        out[m] = stats([t.get(m) for t in trials])
    return out


def main():
    cells_in = json.load(sys.stdin)
    cells_out = [aggregate_cell(c) for c in cells_in]
    json.dump({"cells": cells_out}, sys.stdout, indent=2)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
