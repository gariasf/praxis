#!/usr/bin/env python3
# Usage: scripts/probe-899-table.py <output directory of probe-899.sh> <scene>
import json
import pathlib
import statistics
import sys

output = pathlib.Path(sys.argv[1])
scene = sys.argv[2]
variants = ["main", "checked", "no-bounds", "no-loop", "no-int-div", "unchecked"]


def lines_of(result):
    lines = {
        "Gate, rays off": result["gate_microseconds"]["median"],
        "GPU frame": result["gpu_frame_microseconds"]["median"],
    }
    for gate_pass, timing in result["gate_pass_microseconds"].items():
        lines[f"`{gate_pass}`"] = timing["median"]
    return lines


rounds = {}
for variant in variants:
    results = sorted((output / variant).glob(f"round-*/{scene}.json"))
    rounds[variant] = [lines_of(json.loads(path.read_text())) for path in results]

present = [variant for variant in variants if rounds[variant]]
print("| Line, µs | " + " | ".join(f"`{variant}` ({len(rounds[variant])})" for variant in present) + " |")
print("|---|" + "---:|" * len(present))
for line in rounds[present[0]][0]:
    medians = {
        variant: statistics.median(round_lines[line] for round_lines in rounds[variant])
        for variant in present
    }
    spreads = {
        variant: max(round_lines[line] for round_lines in rounds[variant])
        - min(round_lines[line] for round_lines in rounds[variant])
        for variant in present
    }
    cells = [f"{medians[present[0]]:,.1f} (spread {spreads[present[0]]:,.1f})"]
    cells += [
        f"{medians[variant] - medians[present[0]]:+,.1f} (spread {spreads[variant]:,.1f})"
        for variant in present[1:]
    ]
    print(f"| {line} | " + " | ".join(cells) + " |")
