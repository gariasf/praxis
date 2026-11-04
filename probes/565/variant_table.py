"""The note's variant table: each variant's gate, GPU frame and CPU frame
medians, lowest to highest over its rounds, from each run's results."""
import json
variants = [
    ("All 10,005, every pass", "full", ["2", "3", "4"]),
    ("The lit pass's group 1, the depth passes' all", "litlight", ["1", "2", "3", "4"]),
    ("The depth passes' group 1, the lit pass's all", "depthlight", ["1", "2"]),
    ("Both 1", "none", ["1", "2"]),
    ("Both 1, the 100 sampled Layers kept", "ks1", ["3", "4"]),
    ("Both 2,500, sampled kept", "ks2500", ["3", "4"]),
    ("Both 5,000, sampled kept", "ks5000", ["3", "4"]),
    ("Both 7,500, sampled kept", "ks7500", ["3", "4"]),
    ("Both 2,500, sampled not kept", "both2500", ["1", "2"]),
    ("Both 5,000, sampled not kept", "both5000", ["1", "2"]),
    ("Both 7,500, sampled not kept", "both7500", ["1", "2"]),
]
def span(values):
    low, high = min(values), max(values)
    return f"{low:,.1f}–{high:,.1f}"
print("| Variant: distinct Textures bound | Rounds | Gate, µs | GPU frame, µs | CPU frame, µs |")
print("|---|---:|---|---|---|")
for label, name, rounds in variants:
    runs = [json.load(open(f"runs/{name}.r{r}/texture-wall.json")) for r in rounds]
    gate = span([run["gate_microseconds"]["median"] for run in runs])
    gpu = span([run["gpu_frame_microseconds"]["median"] for run in runs])
    cpu = span([run["cpu_frame_microseconds"]["median"] for run in runs])
    print(f"| {label} | {len(runs)} | {gate} | {gpu} | {cpu} |")
