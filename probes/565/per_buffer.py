"""Per command buffer index of a frame's submission (count 20: rays off,
25: rays on), median µs from the frame's culling phase one start: commit,
kernel start, kernel end, GPU start, GPU end; then the lit pass's start."""
import csv, sys, statistics, collections
run, path, count = sys.argv[1], sys.argv[2], int(sys.argv[3])
ticks = "ticks.1.csv" if count == 20 else "ticks.csv"
rows = {int(r["frame"]): r for r in csv.DictReader(open(f"{run}/{ticks}"))}
lines = [l.strip().split(",") for l in open(path)]
subs = []
current = []
for l in lines:
    index, total = int(l[0]), int(l[1])
    if total != count: continue
    current.append([float(x) * 1e6 for x in l[2:]])
    if len(current) == count:
        subs.append(current); current = []
# match submissions to frames by GPU start nearest culling start
cullings = sorted((float(r["culling_start_ns"]) / 1000, f) for f, r in rows.items() if f >= 248)
import bisect
starts = [c for c, _ in cullings]
table = collections.defaultdict(lambda: collections.defaultdict(list))
lit = []
for sub in subs:
    first_gpu = sub[5][3]
    i = bisect.bisect_left(starts, first_gpu - 50)
    if i >= len(starts) or abs(starts[i] - first_gpu) > 100: continue
    culling, frame = cullings[i]
    for index, cb in enumerate(sub):
        for key, value in zip(("commit", "kernel_start", "kernel_end", "gpu_start", "gpu_end"), cb):
            table[index][key].append(value - culling)
    lit.append(float(rows[frame]["lit_pass_start"]) / 1000)
print(run, "count", count, "submissions matched", len(lit))
print("index   commit  k_start   k_end  gpu_start gpu_end  (k_end-k_start)")
for index in sorted(table):
    m = {k: statistics.median(v) for k, v in table[index].items()}
    span = statistics.median([e - s for s, e in zip(table[index]["kernel_start"], table[index]["kernel_end"])])
    print(f"{index:5d} {m['commit']:8.0f} {m['kernel_start']:8.0f} {m['kernel_end']:8.0f} {m['gpu_start']:8.0f} {m['gpu_end']:8.0f}   {span:7.0f}")
print("lit pass start median", round(statistics.median(lit)))
