# Audit lane 3, performance, where we left off

The lane's report, in the form `docs/agents/audit.md` gives: each timed
pass on each Reference scene benched at the frozen commit
`8bc34eb18fcfb79307193551e96619316aef566c` against its own hardware floor
on the Apple M4 Pro, as the verdict, and against its source's published
cost, as a 2x profiling trigger only; the "revive when" fog lines of the
ladder's map re-judged; and `town`'s remaining gap decomposed into named
options with a cost estimate each. Two readers, one per module, each a
section below; the lane's report session writes the Summary, the counts
and the Killed section and runs the verifier over every quoted line.
Finding ids carry the module's letter so that sections written in
parallel never collide: the dense scenes `L3-D<n>`, the rest `L3-L<n>`.
The bench JSON of every round is under `docs/audit/m3.5/bench/`, each
file stamped with its run's Provenance and quiet verdict.

## Summary

Written by the report session.

## Counts

Written by the report session.

## Dense scenes: `town` and texture-wall

Scope: the two Reference scenes whose Baselines carry the milestone's
Budget breaches, benched on the frozen commit in a worktree checked out
at it, every timed pass against its hardware floor on the Apple M4 Pro
and against its source's published cost, and `town`'s remaining gap
decomposed. One reader at high effort, with the lane brief verbatim.
Finding ids are `L3-D<n>`. Every figure below is pasted from `praxis
report` or from the bench JSON under `docs/audit/m3.5/bench/round-<n>/`
by a script that lists each line's three medians; none is retyped.

### Run conditions

Apple M4 Pro (16-core GPU), macOS 27.0.1, wgpu 30.0.1 on Metal, release
build of `8bc34eb18fcfb79307193551e96619316aef566c` on the branch
`audit/m3.5-lane-3`, the tree clean (`dirty: false` in every Provenance).
`praxis bench <dir> town texture-wall` at 2560×1440 with 4x MSAA, 248
warm-up and 1,800 measured frames per scene, rays on then off, aliasing
counters included, three rounds back to back on 2026-11-02 from 16:40 to
16:49 UTC, each round's start written to its `machine.txt`: on AC power
(the rounds waited for the charger; the bench stamps a Mac on battery
not quiet), GPU load 7%, 3% and 0% at the three starts
(the bench's own reading 9%, 3% and 2%), no other praxis process, no
peer session benching (three peer sessions confirmed by message that
they ran nothing on the GPU; one ran `scripts/verify.sh` before the
rounds and finished at 16:34 UTC). Load averages at the starts were
3.0, 3.2 and 17.0 over one minute (8 to 16 over five), from Spotlight's
`mds_stores` indexing the new worktrees, which this reader could not
stop; round 3's GPU frame agrees with rounds 1 and 2 within 0.5%, its gate
within 1.3% and its pass lines within 3.7% (the Froxel pass; every other
pass within 1.5%), and its CPU lines are the lowest of the three, so the GPU figures stand and the
CPU figures carry that caveat. The rounds published their records to
`bench-history` as the bench does by default and ADR 0024 asks; their
source hash is `main`'s at the frozen commit and the tree was clean, so
they are Series records of this machine for that commit, not an
Experiment's.

Quiet verdicts from the Provenance in each file (`quiet`,
`gpu_frame_p95_over_median`):

| Round | `town` | texture-wall |
|---|---|---|
| 1 | quiet, p95/median 1.040 | not quiet, p95/median 1.092 |
| 2 | quiet, 1.043 | not quiet, 1.072 |
| 3 | quiet, 1.011 | not quiet, 1.105 |

texture-wall's three stamps come from the p95-over-median rule alone
(1.06 in `src/bench/provenance.rs:29`): its GPU load, processes and power
checks passed in every round. Its GPU frame p95 sits 7 to 11% over its
median in these rounds at 2 to 9% GPU load, 5.4% in its Baseline run
(16,839 against 15,975 µs) and 1.6 to 3.5 ms over it in four of the M2
exit's six runs, so the stamp flips with the scene's own spread, not
the machine's state: L3-D07.

### `town`

Round 3 as `praxis report` prints it, the round with the lowest load and
the tightest p95s; the other rounds' tables are in the bench JSON and
agree as the rounds table below shows.

**town** on Apple M4 Pro, release, 2560x1440, 1800 measured frames

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 5,679.2 | 6,121.0 | 4,734.8 | 5,088.3 | +944.5 | 1.199 | 6,000.0 |
| Dynamic TLAS, CPU | 1,171.4 | 1,321.2 | 1,097.4 | 1,181.7 | +74.0 | 1.067 | 1,100.0 |
| GPU frame | 19,987.8 | 20,210.1 | 19,935.0 | 20,132.9 | +52.7 | 1.003 | 12,000.0 |
| Gate, rays off | 8,603.3 | 8,889.9 | 8,612.1 | 8,878.4 | -8.8 | 0.999 | 6,000.0 |
| Shadow-ray time | 10,857.8 | 11,431.0 | 10,792.9 | 11,345.0 | +64.9 | 1.006 | — |
| `bend_pass` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | — |
| `blas_refit` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | — |
| `culling_phase_one` | 1,164.3 | 1,208.2 | 1,145.2 | 1,181.2 | +19.0 | 1.017 | — |
| `culling_phase_two` | 175.7 | 192.5 | 169.2 | 187.4 | +6.5 | 1.038 | — |
| `depth_prepass` | 2,139.0 | 2,233.3 | 2,156.0 | 2,246.0 | -17.0 | 0.992 | — |
| `depth_pyramid_pass` | 222.2 | 231.5 | 219.7 | 226.0 | +2.5 | 1.012 | — |
| `dynamic_tlas_build` | 0.0 | 81.2 | 0.0 | 89.8 | +0.0 | — | 1,000.0 |
| `froxel_pass` | 604.0 | 634.7 | 579.5 | 597.7 | +24.5 | 1.042 | — |
| `lit_pass` | 3,743.9 | 3,871.8 | 3,762.6 | 3,890.5 | -18.7 | 0.995 | — |
| `phase_two_depth_pass` | 55.2 | 107.8 | 56.3 | 109.7 | -1.0 | 0.981 | — |
| `shadow_mask_pass` | 12,012.5 | 12,436.2 | 11,924.1 | 12,359.3 | +88.4 | 1.007 | — |
| `tonemapping_pass` | 323.3 | 368.7 | 324.7 | 372.5 | -1.4 | 0.996 | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 4 | 4 | +0 | 1.000 | — |
| `instances_drawn` | 137,730 | 137,730 | +0 | 1.000 | — |
| `triangles_drawn` | 5,552,214 | 5,552,214 | +0 | 1.000 | — |
| `instances_overflowed` | 0 | 0 | +0 | — | — |
| `uploaded_bytes` | 2,049,376 | 2,049,376 | +0 | 1.000 | — |
| `gpu_memory_bytes` | 764,009,620 | 764,009,620 | +0 | 1.000 | 8,000,000,000 |
| `arena_bytes_in_use` | 85,280 | 85,280 | +0 | 1.000 | — |
| `heap_allocations` | 10,498 | 10,498 | +0 | 1.000 | — |
| `aliasing_rmse` | 5,078 | 5,078 | +0 | 1.000 | — |
| `aliasing_p99` | 15,451 | 15,451 | +0 | 1.000 | — |

The three rounds, each line's medians lowest to highest, then their
median, then the p95s; every counter identical across the rounds:

| Line | Medians, lowest to highest | Median of medians | p95s |
|---|---:|---:|---:|
| CPU frame | 5,679.2–5,709.8–5,793.1 | 5,709.8 | 6,121.0–6,151.0–6,418.2 |
| Dynamic TLAS, CPU | 1,171.4–1,192.5–1,203.0 | 1,192.5 | 1,321.2–1,338.4–1,396.0 |
| GPU frame | 19,983.7–19,987.8–20,075.1 | 19,987.8 | 20,210.1–20,835.7–20,874.0 |
| Gate, rays off | 8,603.3–8,702.2–8,713.1 | 8,702.2 | 8,889.9–9,345.6–9,779.9 |
| Shadow-ray time | 10,750.1–10,834.8–10,857.8 | 10,834.8 | 11,431.0–11,689.8–11,826.2 |
| `culling_phase_one` | 1,148.5–1,164.3–1,165.1 | 1,164.3 | 1,208.2–1,211.5–1,244.5 |
| `culling_phase_two` | 175.7–176.6–177.0 | 176.6 | 192.5–193.4–196.6 |
| `depth_prepass` | 2,139.0–2,142.0–2,145.1 | 2,142.0 | 2,233.3–2,239.4–2,250.1 |
| `depth_pyramid_pass` | 220.8–221.5–222.2 | 221.5 | 231.5–234.0–255.7 |
| `dynamic_tlas_build` | 0.0–0.0–0.0 | 0.0 | 77.2–78.0–81.2 |
| `froxel_pass` | 582.3–601.1–604.0 | 601.1 | 609.2–634.7–635.9 |
| `lit_pass` | 3,743.9–3,763.7–3,786.1 | 3,763.7 | 3,871.8–4,061.6–4,178.6 |
| `phase_two_depth_pass` | 55.2–55.7–55.8 | 55.7 | 107.8–111.3–126.7 |
| `shadow_mask_pass` | 12,003.1–12,012.5–12,046.2 | 12,012.5 | 12,436.2–12,627.9–12,778.3 |
| `tonemapping_pass` | 323.3–324.3–324.6 | 324.3 | 368.7–371.5–568.0 |

Against the Budgets: the GPU frame reads 19,984 to 20,075 µs against
12,000 in every round (L3-D01); the gate 8,603 to 8,713 against 6,000
(L3-D02); the dynamic TLAS's CPU 1,171 to 1,203 against its 1,100 ceiling
(L3-D03); the CPU frame 5,679 to 5,793 at the median, under 6,000, and
6,121 to 6,418 at p95, over it, in every round, 20 to 22% over its
Baseline's median (L3-D04). GPU memory is 764,009,620 bytes of 8 GB.

For context only, as the brief allows where an RTX Baseline exists, the
RTX 2070 SUPER's Baselines for `town` read a GPU frame of 9,395 µs
(Vulkan) and 9,449 (DX12), a gate of 4,992 and 7,492, and shadow-ray
time of 3,995 and 1,477: that GPU is under the 12 ms Budget and, on
Vulkan, under the gate. Not re-measured here.

### Floors

Apple's figure for the M4 Pro is 273 GB/s of memory bandwidth
[ref:apple-m4-pro-bandwidth], 273,000 bytes a microsecond; every
bandwidth floor below is bytes moved, uncompressed, over that. Apple
publishes no ray rate and no shading rate for the M4 Pro, so where a
pass's bound is rays or arithmetic, its floor uses a rate measured on
this machine in `docs/performance.md` and says which. Fixed counts at
2560×1440: 3,686,400 pixels, 4 samples each; `town` draws 5,552,214
triangles (16,656,642 indices) from 137,730 of its 254,256 Instances;
texture-wall draws 200 triangles from 100 Instances. Lit pixels and the
point lights reaching them come from `praxis capture <scene> <png>
2560x1440` on the frozen build: `town` 3,354,085 lit pixels, 0.64 point
lights reaching each (1.09 listed); texture-wall 925,087 lit pixels, no
point light.

Bytes a pass moves on `town`, from the structs at the frozen commit: an
Instance is 128 bytes (`src/render/instance.rs`), a Meshlet record 12
(`src/render/culling_pass.rs:191-196`), a cooked vertex 28
(`src/assets/cooked.rs:394-403`), a Meshlet vertex index 4, a packed
index 4, a Meshlet triangle 3, a depth sample 4 (`Depth32Float`), an HDR
sample 8 (`Rgba16Float`), a shadow mask word 8 (`Rg32Uint`) and its
sample word 4 (`R32Uint`), a Froxel list 128 slots of 4 bytes over
18,432 Froxels (`src/render/froxel_pass.rs:14-29`). The vertex stage
pulls each index's vertex through `meshlet_records[record]`,
`instances[slot]`, `meshlet_vertices[...]` and `vertices[...]`
(`src/render/shaders/main.wgsl:90-94`); a Meshlet holds at most 64
vertices and 124 triangles (`src/assets/cooked.rs:40-44`), so a frame
pulls between 16,656,642 vertices (no reuse of a vertex between its
triangles) and 16,656,642 × 64 / 372 = 2,865,659 (every Meshlet full and
every vertex pulled once). Both bounds are given where they differ.

The measured ray rate is the one `docs/performance.md` measured for the
shadow mask pass (the section of 2026-10-30 on where its time goes): at
least 0.96 ns a ray on the helmet's 257,000 triangles, 0.08 on uv-grid's
two, and a fixed cost of 1,420 µs for the pass's dispatches with no ray
code to 1,907 with the ray code held and every ray skipped, on `town`.
`town`'s TLAS holds 5.5 million triangles in 254,256 Instances, so 0.96 ns
is a lower bound of its rate, and the floor below counts only the rays
every lit pixel traces (the sun's and the lamps' nearest-surface rays),
not the edge rays or the second surfaces.

Each timed pass on `town`, the median of the three rounds' medians, its
floor with the arithmetic, the ratio to the floor (the verdict), its
source's scaled cost from the M3 exit's table (`docs/performance.md`,
"Passes against their sources") and the ratio to it (the 2x profiling
trigger):

| Pass | Median µs | p95 µs (rounds) | Floor µs and arithmetic | Ratio to floor | Source µs, scaled | Ratio to source | Verdict |
|---|---:|---:|---|---:|---:|---:|---|
| `culling_phase_one` | 1,164.3 | 1,208–1,245 | 510: 254,256 × 128 B scanned (32.5 MB) + 16,656,642 × 4 B of packed indices written (66.6 MB) + 5,552,214 × 3 B of Meshlet triangles (16.7 MB) + 137,730 × 128 B of Instances re-read (17.6 MB) + about 210,000 records and Meshlets (5.9 MB) = 139.3 MB / 273,000 | 2.28 | 670 (Haar and Aaltonen, cluster culling); 540 (object culling alone); both phases 1,340.9 against them | 2.00 / 2.48 (both phases) | over 2x its floor on its line: L3-D09. Measured apart in the section of 2026-10-30 on the depth pyramid and culling, its stages sum to 886 µs (1.74x); the line counts time beside the Froxel pass, which starts beside it |
| `culling_phase_two` | 176.6 | 193–197 | 128: the same 32.5 MB scan + about 210,000 × 12 B of records (2.5 MB) = 35.0 MB / 273,000; its depth pyramid reads are from 4.9 MB | 1.38 | counted with phase one | — | within |
| `depth_prepass` | 2,142.0 | 2,233–2,250 | 870 to 2,486: with every vertex pulled once, 16,656,642 × 4 B of indices + 2,865,659 × 32 B of Meshlet vertex index and vertex + 17.6 MB of Instances + 2.5 MB of records = 178.5 MB, plus 3,686,400 × 4 samples × 4 B of depth written (59.0 MB), 237.5 MB / 273,000 = 870; with no reuse 16,656,642 × 36 B + 20.1 MB = 619.8 MB, plus the depth, 678.8 MB / 273,000 = 2,486 | 0.86 to 2.46 | none cited | — | within the no-reuse floor; which bound binds is open (architectural note) |
| `depth_pyramid_pass` | 221.5 | 232–256 | 234: 59.0 MB of samples read + 3.7 MB of level 0 written + 1.2 MB for the levels after = 63.9 MB / 273,000 | 0.95 | 107 (Haar and Aaltonen, from HTILE); 73 (Wihlidal, 16-bit tiles) | 2.07 / 3.03 | at its floor, over 2x its source: architectural note, closed below |
| `froxel_pass` | 601.1 | 609–636 | 138 by bandwidth: 18,432 × 128 × 4 B of lists written (9.4 MB) + `town`'s 1,024 lamps, 48 B each in the light buffer (49 KB), read once per 32-Froxel workgroup, 576 × 49 KB (28.3 MB) = 37.7 MB / 273,000. Its bound is arithmetic, 18,432 × 1,024 = 18,874,368 light-against-Froxel tests, for which Apple publishes no M4 Pro rate | 4.35 by bandwidth; not computable by arithmetic | 320 to 900 (Drobot, 256 lights in 76,800 clusters) | 0.67 to 1.88 | floor not computable from a published figure; within 2x of its source: architectural note |
| `lit_pass` | 3,763.7 | 3,872–4,179 | 1,499 to 3,115: the depth pre-pass's vertex pull (178.5 or 619.8 MB) + 59.0 MB of depth read at Equal + 3,686,400 × 4 × 8 B of HDR written (118.0 MB) + 3,686,400 × 12 B of shadow mask and sample words read (44.2 MB) + 9.4 MB of Froxel lists = 409.1 or 850.4 MB / 273,000 | 1.21 to 2.51 | 8,180 to 10,130 (Drobot, forward+ opaque, 256 lights) | 0.37 to 0.46 | within the no-reuse floor; the same open bound as the depth pre-pass (architectural note) |
| `phase_two_depth_pass` | 55.7 | 108–127 | not floored: phase two's triangle count is not in the counters; under 0.1 ms | — | none cited | — | within |
| `shadow_mask_pass` | 12,012.5 | 12,436–12,778 | 6,701 to 7,188: (3,354,085 sun rays + 3,354,085 × 0.64 = 2,146,614 lamp rays) × 0.96 ns = 5,281, plus the measured fixed cost of 1,420 to 1,907 | 1.67 to 1.79 | 6,040 to 8,530 (Boksansky et al., four lights at 1920×1080, scaled ×1.78) | 1.41 to 1.99 | within 2x of its floor and under 2x its source: no Finding on the pass; it is the frame's gap, decomposed below |
| `tonemapping_pass` | 324.3 | 369–568 | 486: 118.0 MB of HDR samples read + 3,686,400 × 4 B written (14.7 MB) = 132.7 MB / 273,000 | 0.67 | none with a stated resolution | — | under its uncompressed floor: the HDR target is losslessly compressed in memory (architectural note) |
| `dynamic_tlas_build` | 0.0 (p95 77–81) | — | about 2: 10,000 Instances × 64 B; it runs beside the previous frame and reads 0 at the median | — | none cited; its Budget is 1,000 | — | within |
| `bend_pass`, `blas_refit` | 0.0 | — | do not run on `town` | — | — | — | — |

### texture-wall

Round 1 as `praxis report` prints it; the rounds table follows. Its
Baseline holds the fast of its two pass levels (a depth pre-pass of 399
µs), and all three rounds landed on the slow level (726 µs), so every
pass row reads 1.65 to 1.83 times its Baseline (the Froxel pass 2.26,
on 14 µs) while the GPU frame reads 4% under it, as the note of 2026-10-30 on a render pass's line described
(L3-D08).

**texture-wall** on Apple M4 Pro, release, 2560x1440, 1800 measured frames

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 2,264.0 | 2,816.7 | 2,138.5 | 2,325.0 | +125.5 | 1.059 | 6,000.0 |
| Dynamic TLAS, CPU | 0.2 | 0.6 | 0.3 | 0.6 | +0.0 | 0.992 | 1,100.0 |
| GPU frame | 15,304.0 | 16,706.8 | 15,975.0 | 16,839.2 | -671.1 | 0.958 | 12,000.0 |
| Gate, rays off | 13,530.1 | 14,912.8 | 14,109.0 | 14,518.2 | -578.9 | 0.959 | 6,000.0 |
| Shadow-ray time | 260.6 | 2,368.9 | 387.6 | 1,559.6 | -127.0 | 0.672 | — |
| `bend_pass` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | — |
| `blas_refit` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | — |
| `culling_phase_one` | 84.1 | 89.7 | 50.9 | 86.3 | +33.2 | 1.654 | — |
| `culling_phase_two` | 67.0 | 70.1 | 39.4 | 67.3 | +27.6 | 1.701 | — |
| `depth_prepass` | 725.8 | 1,035.6 | 398.6 | 726.4 | +327.2 | 1.821 | — |
| `depth_pyramid_pass` | 886.2 | 1,007.5 | 490.5 | 887.4 | +395.7 | 1.807 | — |
| `dynamic_tlas_build` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | 1,000.0 |
| `froxel_pass` | 25.5 | 28.3 | 11.3 | 27.8 | +14.2 | 2.258 | — |
| `lit_pass` | 1,031.1 | 1,467.4 | 607.2 | 1,032.2 | +423.9 | 1.698 | — |
| `phase_two_depth_pass` | 29.0 | 177.8 | 16.7 | 29.6 | +12.3 | 1.738 | — |
| `shadow_mask_pass` | 4,961.4 | 5,789.2 | 2,973.2 | 4,969.2 | +1,988.2 | 1.669 | — |
| `tonemapping_pass` | 1,480.7 | 1,746.5 | 811.2 | 1,481.5 | +669.5 | 1.825 | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 4 | 4 | +0 | 1.000 | — |
| `instances_drawn` | 100 | 100 | +0 | 1.000 | — |
| `triangles_drawn` | 200 | 200 | +0 | 1.000 | — |
| `instances_overflowed` | 0 | 0 | +0 | — | — |
| `uploaded_bytes` | 224 | 224 | +0 | 1.000 | — |
| `gpu_memory_bytes` | 663,812,595 | 663,812,595 | +0 | 1.000 | 8,000,000,000 |
| `arena_bytes_in_use` | 176 | 176 | +0 | 1.000 | — |
| `heap_allocations` | 350 | 350 | +0 | 1.000 | — |
| `aliasing_rmse` | 3,221 | 3,221 | +0 | 1.000 | — |
| `aliasing_p99` | 13,754 | 13,754 | +0 | 1.000 | — |

| Line | Medians, lowest to highest | Median of medians | p95s |
|---|---:|---:|---:|
| CPU frame | 2,232.1–2,264.0–2,407.0 | 2,264.0 | 2,708.8–2,816.7–3,212.9 |
| Dynamic TLAS, CPU | 0.2–0.3–0.3 | 0.3 | 0.6–0.6–0.7 |
| GPU frame | 15,304.0–15,337.4–15,465.0 | 15,337.4 | 16,448.0–16,706.8–17,091.7 |
| Gate, rays off | 13,530.1–13,830.3–13,884.9 | 13,830.3 | 14,912.8–16,070.2–16,105.3 |
| Shadow-ray time | -20.5–95.7–260.6 | 95.7 | 1,316.5–2,245.0–2,368.9 |
| `culling_phase_one` | 84.1–85.0–85.0 | 85.0 | 87.0–88.5–89.7 |
| `culling_phase_two` | 67.0–67.0–67.5 | 67.0 | 67.9–69.5–70.1 |
| `depth_prepass` | 725.8–726.1–726.2 | 726.1 | 727.0–956.9–1,035.6 |
| `depth_pyramid_pass` | 886.2–886.5–886.5 | 886.5 | 891.0–1,007.5–1,055.6 |
| `froxel_pass` | 25.5–25.8–25.8 | 25.8 | 28.0–28.0–28.3 |
| `lit_pass` | 1,030.8–1,031.1–1,032.5 | 1,031.1 | 1,035.6–1,467.4–1,656.4 |
| `phase_two_depth_pass` | 29.0–29.1–29.2 | 29.1 | 29.8–29.8–177.8 |
| `shadow_mask_pass` | 4,961.4–4,963.0–4,964.1 | 4,963.0 | 5,224.9–5,789.2–7,024.8 |
| `tonemapping_pass` | 1,480.7–1,480.8–1,480.8 | 1,480.8 | 1,481.9–1,655.4–1,746.5 |

Against the Budgets: the GPU frame reads 15,304 to 15,465 µs against
12,000 (L3-D05) and the gate 13,530 to 13,885 against 6,000 (L3-D06). The
gate's timed passes, culling to the lit pass, sum to 2,850 µs at the
medians (85.0 + 67.0 + 726.1 + 886.5 + 25.8 + 1,031.1 + 29.1), so about
10,980 µs of the gate is a stretch no pass times, between culling phase
two's end and the lit pass's start; the scene draws 200 triangles and
binds 10,005 Textures, and the RTX 2070 SUPER's Baselines put the same
scene's gate at 514 µs (Vulkan) and 502 (DX12). Shadow-ray time reads −21
to 261 µs, two GPU frames of 15 ms differenced, as its Baseline note
says.

Each pass against its floor and, where one exists, its source. The
scene's work is fixed per pixel for most passes, so the floors are
`town`'s where the pass does the same work:

| Pass | Median µs | p95 µs (rounds) | Floor µs and arithmetic | Ratio to floor | Source | Verdict |
|---|---:|---:|---|---:|---|---|
| `culling_phase_one` | 85.0 | 87–90 | under 1: 100 × 128 B + 600 × 4 B | about 85x; a dispatch's fixed cost | 670 scaled for 250,000 objects does not apply to 100 | within: 85 µs is 7% of `town`'s line for 0.04% of its Instances; it is the eight dispatches' overhead at this clock |
| `culling_phase_two` | 67.0 | 68–70 | under 1, as above | as above | — | within, as above |
| `depth_prepass` | 726.1 | 727–1,036 | 54 to 216: 925,087 covered pixels × 16 B (14.8 MB) to the whole 59.0 MB depth target flushed | 3.4 to 13.4 | none | over 2x its floor: part of L3-D08, the slow level: `town` writes the same target in 2,142 µs while also pulling 620 MB |
| `depth_pyramid_pass` | 886.5 | 891–1,056 | 234, `town`'s exactly | 3.79 | 107 / 73 | over 2x its floor, 4.0x `town`'s 221.5 for identical work: L3-D08 |
| `froxel_pass` | 25.8 | 28 | about 1: 9.4 MB of lists for no point light; `town`'s one light buffer read is 0 here | — | 320 to 900 does not apply | within: a dispatch's fixed cost |
| `lit_pass` | 1,031.1 | 1,036–1,656 | 810: 118.0 MB of HDR written + 59.0 MB of depth read + 44.2 MB of mask words = 221.2 MB / 273,000 | 1.27 | 8,180 to 10,130 does not apply (one sun, no point light) | within 2x of its floor |
| `phase_two_depth_pass` | 29.1 | 30–178 | not floored; under 0.1 ms | — | — | within |
| `shadow_mask_pass` | 4,963.0 | 5,225–7,025 | 1,494 to 2,795: 925,087 sun rays × 0.08 to 0.96 ns (74 to 888) + the fixed 1,420 to 1,907 | 1.78 to 3.32 | 6,040 to 8,530 for four lights does not apply to one | over 2x its lower floor; the pass reads 2,973 in its fast-level Baseline (1.06 to 1.99x): L3-D08 |
| `tonemapping_pass` | 1,480.8 | 1,482–1,747 | 486, `town`'s exactly | 3.05 | none | over 2x its floor, 4.6x `town`'s 324.3 for identical work: L3-D08 |

Every texture-wall pass that does `town`'s work (the pyramid, the
tonemapping, the lit pass's target writes) reads 1.8 to 4.6 times `town`'s
line, compute and render alike, while the frame it sits in is paced by
the untimed stretch. The note of 2026-10-30 found the same scaling and
could read no GPU clock (Metal exposes none; `powermetrics` needs root).
This reader read none either. Whatever the mechanism, the passes'
ratios to their floors on texture-wall say nothing about the passes:
the scene's two Findings are the frame and the gate.

### Findings

| id | claim | severity | where | evidence | expected |
|---|---|---|---|---|---|
| L3-D01 | `town`'s GPU frame breaches its 12 ms Budget by 8.0 ms at the frozen commit | `budget`, 7,988 µs over at the median of three rounds (7,984 to 8,075) | scene `town`; `bench/budgets.json`, `gpu_frame_microseconds` | three quiet rounds, GPU frame medians 19,983.7, 19,987.8 and 20,075.1 µs (p95 20,210 to 20,874), `docs/audit/m3.5/bench/round-{1,2,3}/town.json`; shadow-ray time 10,750 to 10,858 of it, the frame with rays off 9,130.0 to 9,240.3 (the GPU frame less shadow-ray time, per round) | ≤ 12,000 µs (ADR 0013) |
| L3-D02 | `town`'s gate breaches ADR 0019's 6 ms by 2.7 ms with rays off | `budget`, 2,702 µs over (2,603 to 2,713) | scene `town`; `bench/budgets.json`, `gate_microseconds` | gate medians 8,603.3, 8,702.2 and 8,713.1 µs, same files; its passes: culling 1,341, depth pre-pass 2,142, pyramid 222, phase two's depth 56, lit 3,764 (Froxel 601 beside culling) | ≤ 6,000 µs (ADR 0019); the owner accepted the breach until LOD, which has not landed |
| L3-D03 | `town`'s dynamic TLAS CPU line is over its 1.1 ms ceiling in every round | `budget`, 92.5 µs over at the median of medians (71.4 to 103.0) | scene `town`; `bench/budgets.json`, `dynamic_tlas_cpu_microseconds`; `src/render/acceleration_structures.rs:356` (`record`, the build's recording) and `src/render/renderer.rs:70` (`dynamic_tlas_submit`) | medians 1,171.4, 1,192.5 and 1,203.0 µs, p95 1,321 to 1,396, same files; its Baseline reads 1,097.4; the note of 2026-10-30 on `town`'s CPU frame read 934 to 987 quiet and 1,100 to 1,260 under load, and found the line to be wgpu-core's per-Instance dependency walk and record allocation | ≤ 1,100 µs (ADR 0020's ceiling in `bench/budgets.json`) |
| L3-D04 | `town`'s CPU frame reads 20 to 22% over its Baseline at the median in every round, and over 6 ms at p95, which `compare` reports but does not judge | `health`, 944 to 1,058 µs over its Baseline median; p95 121 to 418 µs over the Budget the medians (207 to 321 under) do not breach | scene `town`; `bench/budgets.json`, `cpu_frame_microseconds` | medians 5,679.2, 5,709.8 and 5,793.1 µs against a Baseline of 4,734.8; p95 6,121.0, 6,151.0 and 6,418.2 against 6,000; the five-minute load average was 8 to 16 during the rounds from Spotlight indexing, which the preflight checks do not see, so the magnitude needs a re-run on a machine without it; `compare` fails the line against its Baseline in every round | within 5% of its Baseline median; medians count for the Budget (`src/bench/mod.rs:629-630`) |
| L3-D05 | texture-wall's GPU frame breaches its 12 ms Budget by 3.3 ms | `budget`, 3,337 µs over (3,304 to 3,465) | scene texture-wall; `bench/budgets.json`, `gpu_frame_microseconds` | GPU frame medians 15,304.0, 15,337.4 and 15,465.0 µs, `docs/audit/m3.5/bench/round-{1,2,3}/texture-wall.json`; GPU load 2 to 9%, AC power, no other praxis process; stamped not quiet by the p95 rule alone (L3-D07) | ≤ 12,000 µs |
| L3-D06 | texture-wall's gate breaches 6 ms by 7.8 ms; of its 13.8 ms, about 11.0 ms is a stretch no pass times, between culling phase two's end and the lit pass's start, in a scene of 200 triangles and 10,005 bound Textures | `budget`, 7,830 µs over (7,530 to 7,885) | scene texture-wall; `bench/budgets.json`, `gate_microseconds`; the material bind group that binds every Texture, `src/render/renderer.rs:278` and `:636` (`MATERIALS`) | gate medians 13,530.1, 13,830.3 and 13,884.9 µs; the gate's timed passes sum to 2,850 µs; the RTX 2070 SUPER Baselines give 514 (Vulkan) and 502 µs (DX12) for the same scene | ≤ 6,000 µs; a floor under 1 µs for the scene's geometry |
| L3-D07 | The quiet stamp marks every texture-wall run not quiet on a quiet machine, because its GPU frame p95 is 7 to 11% over its median by the scene's own nature | `health`, 3 runs of 1 scene; every Series record of texture-wall on this machine | `src/bench/provenance.rs:29` (`QUIET_P95_OVER_MEDIAN = 1.06`) and `:130-141` (`is_quiet`) | `gpu_frame_p95_over_median` 1.092, 1.072 and 1.105 with `gpu_load_percent` 9, 3 and 2, `other_processes` empty and AC power, in the three texture-wall files; `town` in the same rounds 1.040, 1.043 and 1.011; texture-wall's own Baseline run has 1.054 (under the rule) and the M2 exit saw p95 jumps of 1.6 to 3.5 ms in four of six runs | a stamp that reads the machine's state, not a scene's spread; a per-scene threshold or a rule on the lines the machine moves |
| L3-D08 | texture-wall's Baseline holds the fast of its two pass levels, so every run on the slow level, all three here, fails seven pass rows in `compare` and warns on two while its GPU frame reads 4% under the Baseline | `health`, 7 pass rows failed and 2 warned per run, 1 scene | `bench/baselines/apple-m4-pro/texture-wall.json` (`depth_prepass` median 398.6, the fast level); `src/bench/mod.rs:135` (`FAIL_RATIO`), `:143` (the 20 µs noise floor) and `:710` (the pass-row compare) | `praxis compare` on each round: `depth_prepass` 725.8 to 726.2 against 398.6 (+82%), `depth_pyramid_pass` 886.2 to 886.5 against 490.5, `tonemapping_pass` 1,480.7 to 1,480.8 against 811.2, `lit_pass` 1,030.8 to 1,032.5 against 607.2, `shadow_mask_pass` 4,961.4 to 4,964.1 against 2,973.2, both culling phases +65 to +72%; `froxel_pass` (+14.2 µs) and `phase_two_depth_pass` (+12.3 µs) warn under the 20 µs noise floor; the GPU frame 15,304 to 15,465 against 15,975; the pyramid and tonemapping do `town`'s exact work in 4.0 and 4.6 times its lines | pass rows judged at the level a run lands on, or a Baseline at the slow level as the M3 exit kept, until the bound-Textures fix settles the levels |
| L3-D09 | culling phase one's line reads over twice its bandwidth floor on `town` | `health`, 654 µs over its 510 µs floor (2.28x); 1,164 µs a frame | `src/render/culling_pass.rs:284-320` (`record`, phase one's four dispatches); `src/render/shaders/culling.wgsl:401` (`append_triangles`) | medians 1,148.5, 1,164.3 and 1,165.1 µs against 139.3 MB / 273 GB/s = 510 µs (the arithmetic above); the stage split of 2026-10-30 summed its stages to 886 µs (1.74x) with 685 of them the 66.6 MB of packed-index writes (1.7x their own floor), and found the line to include time beside the Froxel pass, which the bench starts beside it | within 2x of its floor; or the line counting the pass alone, as the render passes' lines do since the note of 2026-10-30 |

### Architectural notes

Closed here, not Findings: each pass within 2x of its floor, or whose
floor no published figure gives.

- **The geometry passes sit at the no-reuse vertex-fetch bound.** The
  depth pre-pass reads 2,142 µs against 870 (every vertex pulled once)
  to 2,486 (every index pulling its own 36 bytes); the lit pass 3,764
  against 1,499 to 3,115. If the hardware reuses a vertex between the
  triangles of a Meshlet, both passes are 2.5x over their floor and the
  cost is elsewhere (the rasterizer's 5.5 million triangles, the lit
  pass's shading); if it does not, both are at their floor and every
  index costs 36 bytes twice a frame. The three prototypes that drew
  every triangle (a visibility pre-pass, Meshlet culling, a visibility
  buffer) each left the gate at 8.6 ms and concluded that the triangles
  drawn are what breaches it; this floor says the same in bytes. The
  probe that decides between the bounds is a vertex stage that pulls a
  constant in place of the vertex, a code change no reader runs; it
  belongs to the gate's fix ticket.
- **The depth pyramid is at its floor and over twice its sources.** 221.5
  µs against 234 by bandwidth; 2.07 and 3.03 times the two source
  figures, which reduce tile metadata or a 16-bit single-sample buffer,
  not four 32-bit samples a pixel. The comparison does not apply, as the
  section of 2026-10-30 on the pyramid concluded; the pass has since
  shipped its per-pixel level 0.
- **The Froxel pass's floor is arithmetic Apple does not publish.** 601
  µs for 18,874,368 light-against-Froxel tests, 32 ps a test; by
  bandwidth 4.4x over 138 µs, but it moves 38 MB. Within 2x of Drobot's
  figure on a PS4. No Finding; the source's 2x trigger does not fire.
- **The shadow mask pass is within 2x of its ray floor.** 12,012 µs
  against 6,701 to 7,188 from the sun's and lamps' rays at the helmet's
  measured rate plus the pass's measured fixed cost; the edge rays and
  second surfaces (2.2 and 1.7 ms in the ablation of 2026-10-30) are
  work the floor leaves out, so the true ratio is lower. Under 2x its
  source (1.41 to 1.99 of four lights' cost scaled). The pass is not a
  Finding; the frame it breaches is (L3-D01), and the gap below is its.
- **The tonemapping pass reads under its uncompressed floor**, 324 µs
  against 486: the 118 MB HDR target is read compressed, as Metal
  losslessly compresses a non-block-compressed texture whose usage has
  no shader write [ref:apple-metal-lossless-compression], which the HDR
  target meets. Any floor that reads an MSAA target on this GPU is an
  upper bound.
- **Culling phase one's line and the Froxel pass overlap.** The two
  compute passes start together; the bench counts each compute pass
  from its own begin to its own end, so phase one's line holds about
  260 µs it shares with the Froxel pass (L3-D09's evidence). A rule that
  counted compute passes from the end of the passes before them would
  flip the Froxel line instead, as the note of 2026-10-30 on a render
  pass's line weighed.

### `town`'s remaining gap, decomposed

Two gaps, measured above at the frozen commit: the GPU frame against
ADR 0013's 12 ms, 7,988 µs at the median of medians (19,987.8 against 12,000), of which
shadow-ray time is 10,835 µs (the frame with rays off reads 9,130.0 to
9,240.3, itself under 12 ms); and ADR 0019's
gate against 6 ms, 2,702 µs (8,702.2 against 6,000), with rays off. Every option below
names what it saves on `town`, measured where a prototype or ablation
measured it and estimated from the floors above where not, and what it
costs elsewhere. Options first weighed on the shadow mask (A to E) and
the re-trace (F) are carried forward with their verdicts; the raster
options (G to K) follow from the vertex-fetch floor, which the three
prototypes that drew every triangle (a visibility pre-pass, Meshlet
culling and a visibility buffer) each ran into.

| Option | Saving on `town` | Costs elsewhere | Standing |
|---|---:|---|---|
| A. Skip the dynamic TLAS query for a ray that misses the Moving Instances' box | 0.27 ms, measured; shipped at the frozen commit | the dynamic TLAS's CPU line +2% | done; what the lamps' dynamic queries still cost after the one-call merge was not measured apart, at most about 0.6 ms |
| B. Merge each streaming cell's static Instances into one BLAS, so a ray crosses fewer overlapping Instances | unmeasured; bounded above by the rays' traversal share, about 9.3 ms of the pass (the ablation's rays-against-fixed split), of which the three vendors name overlapping Instances as a part they do not size; a tenth of it, about 0.9 ms, is the guess to test | BLAS memory per cell (instancing lost), builds at streaming time, and a merge can slow traversal: needs a prototype | open; sized for the gap, belongs with M4's cells |
| C. Order a surface's lamps by contribution and stop under a display step; tighten each lamp's range to where it stops contributing | at most 1.2 ms, the ablation's cap at one lamp a surface | the untested lamps' shadows within a set bound; changes ADR 0020's one ray per light | open |
| D. Trace only the tiles a conservative classifier lists | unmeasured; bounded above by the sun's nearest-surface rays, 3,354,085 × 0.96 ns = 3.2 ms, for tiles wholly lit or wholly shadowed; the share of such tiles on `town` is not counted | needs a conservative occluder bound for the sun, which Praxis has no shadow map to give | open, unlikely |
| E. Upstream: naga lowering a single opaque first-hit query to Metal's intersector | unmeasured; Apple sizes the intersection query's extra cost as scratch memory traffic and gives no ratio, so the bound is the rays' 9.3 ms and the saving is some fraction of it, unknown | a naga change, then a wgpu bump | open |
| F. Trace the turned point lights' re-trace where its latency hides, in a dispatch that may overlap | at most 0.7 ms, what the turned point lights' facing-sample search costs; the second-dispatch form prototyped read 0.14 to 0.29 ms slower than `main` instead | a wgpu Metal backend change (overlapping dispatches), or a read-write mask format under ADR 0021 for at most a fifth of it | open, upstream |
| G. The sun's edge rays | 2.2 ms, measured within the sun's 5.7 | the shadow edges the owner chose at a 4 ms ceiling: an image change | ruled out by the image unless the ceiling moves |
| H. The second surfaces | 1.7 ms, measured | ADR 0020's terminator and silhouette shadows: an image change | ruled out by the image |
| I. Fewer indices drawn: LOD at distance | the two geometry passes sit at their no-reuse vertex-fetch floor (36 bytes an index), so each index not drawn saves about 36 B × 2 / 273 GB/s; at half the indices about 1.1 ms off the depth pre-pass, 1.4 ms off the lit pass and 0.3 ms off culling phase one's index writes, with the gate near 6 ms | LOD Meshes cooked and chosen, M4's charting; BLASes unchanged, so the rays do not move | the one option sized for the gate; the three draw-everything prototypes agree |
| J. Fewer bytes an index: a 16- or 20-byte vertex (f16 position in the Mesh's box, f16 uv, the qtangent as is) and the Meshlet vertex indirection folded into the packed index | up to a third of the fetch bytes: about 0.7 ms off the depth pre-pass and 0.9 off the lit pass if they stay at the no-reuse bound; nothing if the hardware already reuses vertices (floor 0.87 ms) | a cook format change (ADR 0018 Blobs re-cooked), precision at the box's scale, the packed index's bit budget (`pack_index`: record and Meshlet vertex in 32 bits) | open; the probe that decides between the two bounds is a vertex stage that pulls a constant, which no reader may run |
| K. `append_records` writes each record's first packed index so `append_triangles` reads no Instance, Mesh or slot offsets | about 65 µs by bytes (17.6 MB of Instances), unmeasured | one u32 a record, 4 MiB at capacity | open, small |
| L. wgpu 31's TLAS dependency dedup | CPU only: the CPU frame from 4.6 to 1.8 ms, measured with the patch | none on the GPU frame or the gate | waiting on wgpu 31 |

Summed, the exact options that keep the image (A done, B unmeasured, C bounded, E and F upstream, I, J, K) reach the frame's gap only if B or I delivers several milliseconds: the rays' fixed cost (about 1.4 to 1.9 ms with the ray code held) and the sun's 5.7 ms stay whatever the lamps do. The gate's gap is I's alone.
