"""Per-run medians of each pass's start/end (µs from culling phase one's
start), rays on (ticks.csv) and off (ticks.1.csv), plus the stretch: lit
pass start less the latest end of the passes before it."""
import csv, statistics, sys, pathlib
PASSES_BEFORE_LIT = ["culling_phase_one","froxel_pass","depth_prepass","depth_pyramid_pass","culling_phase_two","phase_two_depth_pass","shadow_mask_pass"]
def load(path):
    rows = list(csv.DictReader(open(path)))
    return [r for r in rows if int(r["frame"]) >= 100]
def value(r, k):
    v = r.get(k, "")
    return float(v)/1000 if v != "" else None
for run in sys.argv[1:]:
    for name, label in (("ticks.csv","on"),("ticks.1.csv","off")):
        p = pathlib.Path(run)/name
        if not p.exists(): continue
        rows = load(p)
        cols = [c[:-6] for c in rows[0] if c.endswith("_start")]
        line = []
        stretch = []
        gate = []
        for r in rows:
            ends = [value(r, f"{x}_end") for x in PASSES_BEFORE_LIT if f"{x}_end" in r]
            ends = [e for e in ends if e is not None and e > -1e5]
            if label == "off":
                ends = [value(r, f"{x}_end") for x in PASSES_BEFORE_LIT if x not in ("phase_two_depth_pass","shadow_mask_pass")]
            lit = value(r, "lit_pass_start")
            stretch.append(lit - max(ends))
            gate.append(value(r, "lit_pass_end"))
        med = lambda xs: statistics.median(xs)
        print(f"{run} rays {label}: frames {len(rows)} stretch {med(stretch):.0f} gate~ {med(gate):.0f}")
        for c in cols:
            s = [value(r,f"{c}_start") for r in rows]; e=[value(r,f"{c}_end") for r in rows]
            s=[x for x in s if x is not None]; e=[x for x in e if x is not None]
            if s and e and abs(med(e))<1e5:
                print(f"   {c:24s} {med(s):9.0f} {med(e):9.0f}  ({med(e)-med(s):.0f})")
