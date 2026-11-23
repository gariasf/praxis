"""Usage: python3 scripts/probe-1006-table.py <results directory>

Prints, per variant of scripts/probe-1006.sh, each round's medians in
microseconds and the median of the rounds, then what each variant's rays hit.
"""

import json
import pathlib
import re
import statistics
import sys

results = pathlib.Path(sys.argv[1])
rounds = sorted(results.glob("round-*"))
variants = sorted({directory.name for round_directory in rounds for directory in round_directory.iterdir()})

metrics = {
    "alone": lambda run: run.get("probe_ray_pass_alone_microseconds"),
    "frame_growth": lambda run: run.get("probe_ray_frame_growth_microseconds"),
    "timestamps": lambda run: run["gpu_pass_microseconds"].get("probe_ray_pass", {}).get("median"),
    "gpu_frame": lambda run: run["gpu_frame_microseconds"]["median"],
    "tlas_build_gpu": lambda run: run["gpu_pass_microseconds"]["dynamic_tlas_build"]["median"],
    "tlas_build_gpu_p95": lambda run: run["gpu_pass_microseconds"]["dynamic_tlas_build"]["p95"],
    "tlas_build_cpu": lambda run: run["dynamic_tlas_cpu_microseconds"]["median"],
}

print("| variant | metric | rounds | median | quiet |")
print("|---|---|---|---|---|")
for variant in variants:
    runs = []
    for round_directory in rounds:
        files = list((round_directory / variant).glob("*.json"))
        if files:
            runs.append(json.loads(files[0].read_text()))
    for metric, read in metrics.items():
        values = [read(run) for run in runs]
        if any(value is None for value in values) or not values:
            continue
        quiet = sum(bool(run.get("provenance", {}).get("quiet")) for run in runs)
        listed = ", ".join(f"{value:.1f}" for value in values)
        print(f"| {variant} | {metric} | {listed} | {statistics.median(values):.1f} | {quiet}/{len(runs)} |")

print()
print("| variant | rays | hits | lit | mean hit metres |")
print("|---|---|---|---|---|")
for variant in variants:
    for log in sorted(results.glob(f"round-*/{variant}/log.txt"))[:1]:
        counts = re.search(r"rays=(\d+).*?hits=(\d+).*?lit=(\d+).*?mean_hit_metres=([0-9.]+)", re.sub(r"\x1b\[[0-9;]*m", "", log.read_text()))
        if counts:
            rays, hits, lit, metres = counts.groups()
            print(f"| {variant} | {rays} | {hits} | {lit} | {float(metres):.1f} |")
