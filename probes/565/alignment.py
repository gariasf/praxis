"""Per frame: the lit pass's command buffer's kernel span, the stretch
(lit pass start less the latest end of the passes before it), and the lit
pass start less that buffer's kernel end; matched on the buffer whose GPU
start is nearest the lit pass's start."""
import csv, sys, statistics, bisect
run, path, count, lit_index = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
ticks = "ticks.1.csv" if count == 20 else "ticks.csv"
before = ["culling_phase_one","froxel_pass","depth_prepass","depth_pyramid_pass","culling_phase_two"] + ([] if count == 20 else ["phase_two_depth_pass","shadow_mask_pass"])
lines = [l.strip().split(",") for l in open(path)]
lit_buffers = sorted([float(x) * 1e6 for x in l[2:]] for l in lines if int(l[1]) == count and int(l[0]) == lit_index)
starts = [b[3] for b in lit_buffers]
spans, stretches, gaps = [], [], []
for r in csv.DictReader(open(f"{run}/{ticks}")):
    if int(r["frame"]) < 248: continue
    culling = float(r["culling_start_ns"]) / 1000
    lit = culling + float(r["lit_pass_start"]) / 1000
    i = bisect.bisect_left(starts, lit - 1000)
    candidates = [b for b in lit_buffers[i:i+3] if abs(b[3] - lit) < 1000]
    if not candidates: continue
    b = min(candidates, key=lambda b: abs(b[3] - lit))
    latest = max(float(r[f"{p}_end"]) / 1000 for p in before)
    spans.append(b[2] - b[1]); stretches.append(lit - culling - latest); gaps.append(lit - b[2])
q = lambda v, p: sorted(v)[int(p * (len(v) - 1))]
for name, v in (("kernel span of the lit pass's buffer", spans), ("stretch before the lit pass", stretches), ("lit start less kernel end", gaps)):
    print(f"{run} {count}: {name}: median {statistics.median(v):.0f} p5 {q(v,.05):.0f} p95 {q(v,.95):.0f} (n={len(v)})")
