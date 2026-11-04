"""Each frame's command buffer times against its GPU timestamps, µs from
culling phase one's start: commit (CPU), kernel start and end, GPU start,
lit pass start, GPU end."""
import csv, sys, bisect, statistics
run, cb_path = sys.argv[1], sys.argv[2]
cbs = sorted(tuple(float(x) * 1e6 for x in line.split(",")) for line in open(cb_path))
gpu_starts = [cb[3] for cb in cbs]
for name in ("ticks.csv", "ticks.1.csv"):
    rows = [r for r in csv.DictReader(open(f"{run}/{name}")) if int(r["frame"]) >= 248]
    columns = {k: [] for k in ("commit", "kernel_start", "kernel_end", "gpu_start", "lit_start", "lit_end", "gpu_end", "kernel_span", "commit_to_kernel_start")}
    for r in rows:
        culling = float(r["culling_start_ns"]) / 1000
        index = bisect.bisect_right(gpu_starts, culling) - 1
        if index < 0: continue
        commit, kernel_start, kernel_end, gpu_start, gpu_end = cbs[index]
        if not (gpu_start <= culling <= gpu_end): continue
        for key, value in (("commit", commit), ("kernel_start", kernel_start), ("kernel_end", kernel_end), ("gpu_start", gpu_start), ("gpu_end", gpu_end)):
            columns[key].append(value - culling)
        columns["lit_start"].append(float(r["lit_pass_start"]) / 1000)
        columns["lit_end"].append(float(r["lit_pass_end"]) / 1000)
        columns["kernel_span"].append(kernel_end - kernel_start)
        columns["commit_to_kernel_start"].append(kernel_start - commit)
    print(run, name, "frames matched", len(columns["commit"]), "of", len(rows))
    for key, values in columns.items():
        values = sorted(values)
        print(f"   {key:24s} median {statistics.median(values):9.0f}  p5 {values[len(values)//20]:9.0f}  p95 {values[len(values)*19//20]:9.0f}")
