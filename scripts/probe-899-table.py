#!/usr/bin/env python3
# Usage: scripts/probe-899-table.py <scene> <output directory of probe-899.sh>...
import json
import pathlib
import statistics
import sys

scene = sys.argv[1]
outputs = [pathlib.Path(argument) for argument in sys.argv[2:]]
variants = ["main", "checked", "no-bounds", "no-loop", "no-int-div", "unchecked"]
FAST_BELOW_MICROSECONDS = 1200.0
SLOW_ABOVE_MICROSECONDS = 1240.0


def lines_of(result):
    lines = {
        "Gate, rays off": result["gate_microseconds"]["median"],
        "GPU frame": result["gpu_frame_microseconds"]["median"],
    }
    for gate_pass, timing in result["gate_pass_microseconds"].items():
        lines[f"`{gate_pass}`"] = timing["median"]
    return lines


def state_of(lines):
    depth_prepass = lines["`depth_prepass`"]
    if depth_prepass < FAST_BELOW_MICROSECONDS:
        return "fast"
    if depth_prepass > SLOW_ABOVE_MICROSECONDS:
        return "slow"
    return "between"


runs = {variant: [] for variant in variants}
for output in outputs:
    for variant in variants:
        for path in sorted((output / variant).glob(f"round-*/{scene}.json")):
            runs[variant].append(lines_of(json.loads(path.read_text())))

print("Runs in each state, by `depth_prepass`:\n")
print("| Variant | fast | slow | between |")
print("|---|---:|---:|---:|")
for variant in variants:
    states = [state_of(lines) for lines in runs[variant]]
    print(f"| `{variant}` | {states.count('fast')} | {states.count('slow')} | {states.count('between')} |")

for state in ("slow", "fast"):
    in_state = {
        variant: [lines for lines in runs[variant] if state_of(lines) == state]
        for variant in variants
    }
    present = [variant for variant in variants if in_state[variant]]
    if "main" not in present:
        continue
    print(f"\nThe {state} state: `main`'s mean and range, then each variant's mean less `main`'s and its own range, µs:\n")
    print("| Line | " + " | ".join(f"`{variant}` ({len(in_state[variant])})" for variant in present) + " |")
    print("|---|" + "---:|" * len(present))
    for line in in_state["main"][0]:
        values = {variant: [lines[line] for lines in in_state[variant]] for variant in present}
        main_mean = statistics.mean(values["main"])
        cells = [f"{main_mean:,.1f} ({min(values['main']):,.1f} to {max(values['main']):,.1f})"]
        for variant in present[1:]:
            difference = statistics.mean(values[variant]) - main_mean
            cells.append(f"{difference:+,.1f} ({min(values[variant]):,.1f} to {max(values[variant]):,.1f})")
        print(f"| {line} | " + " | ".join(cells) + " |")
