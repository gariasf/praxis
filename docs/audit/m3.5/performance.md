# Audit lane 3, performance, where we left off

The lane's report, in the form `docs/agents/audit.md` gives: each timed
pass on each Reference scene benched at the frozen commit
`8bc34eb18fcfb79307193551e96619316aef566c` against its own hardware floor
on the Apple M4 Pro, as the verdict, and against its source's published
cost, as a 2x profiling trigger only; the "revive when" fog lines of the
ladder's map re-judged; and `town`'s remaining gap decomposed into named
options with a cost estimate each. Two readers, one per module, each a
section below; the report session wrote the Summary, the counts, the
verifier's re-run and the Killed section, and ran the verifier over
every quoted line.
Finding ids carry the module's letter so that sections written in
parallel never collide: the dense scenes `L3-D<n>`, the rest `L3-L<n>`.
The bench JSON of every round, the readers' and the verifier's, is under
`docs/audit/m3.5/bench/`, each file stamped with its run's Provenance
and quiet verdict.

## Summary

Two readers benched all eleven Reference scenes at the frozen commit on
the Apple M4 Pro and filed 11 Findings; the verifier re-ran every scene
in four rounds of its own and confirmed 4, narrowed 6 and killed 1.
Of the ten that stand, five are `budget`: `town`'s GPU frame is 8.0 ms
over its 12 ms Budget, all of it shadow-ray time, and its gate 2.6 to 2.7
ms over 6 ms with its geometry passes at or past their no-reuse
vertex-fetch bound, so fewer indices drawn (LOD) is the one option sized
for the gate; texture-wall's frame and gate are 2.9 and 7.1 ms over, about
10 ms of the gate in time no pass line covers; and `town`'s dynamic TLAS
CPU line crosses its 1.1 ms ceiling in every round, all taken under load.
The other five are `health` (culling phase one at 2.28 times its floor on
`town`, Baselines whose pass levels make `compare` fail or warn on lines
that did not move, and CPU frame lines over their Baselines that no quiet
run has settled), and of the map's 36 revive-when lines 5 have fired, 31
wait and none is dead.

## Counts

After the verifier. Each module section's text is its reader's, before
the verifier, but for the corrections listed at the end of the
verifier's re-run; the verdicts sit in each Findings table's `verifier`
column. Of the 6 `health` Findings, 5 stand.

| Severity class | `confirmed` | `narrowed` | `killed` | Findings |
|---|---:|---:|---:|---:|
| `image` | 0 | 0 | 0 | 0 |
| `budget` | 2 (L3-D01, L3-D02) | 3 (L3-D03, L3-D05, L3-D06) | 0 | 5 |
| `correctness` | 0 | 0 | 0 | 0 |
| `drift` | 0 | 0 | 0 | 0 |
| `health` | 2 (L3-D04, L3-D09) | 3 (L3-D08, L3-L01, L3-L02) | 1 (L3-D07) | 6 |
| All | 4 | 6 | 1 | 11 |

No verdict moved a Finding between classes. The revive-when lines are
judged, not Findings: `fired` 5, `waiting` 31 and both offenders, `dead`
none.

## The verifier's re-run

The verifier trusted no reader's figure. The report session ran four
fresh rounds for it, so that the session alone held the GPU and the
peers' messages, and the verifier re-read every quoted line from them:
each round every Reference scene in one `praxis bench <dir>
--no-publish` on a release build of the frozen commit in a worktree
detached at it (`dirty: false` in all 44 Provenances), 2560×1440 with 4x
MSAA, 248 warm-up and 1,800 measured frames, rays on then off, aliasing
counters included, under `caffeinate -dims`, each round starting once
preflight's GPU load line read `ok` (the reading logged seconds later
before round 3 read 84%; below). The rounds ran on 2026-11-02
from 19:38 to 19:50 UTC back to back and the fourth from 20:20 to 20:24,
on AC power with the battery charged, no other praxis process, and every
peer session confirming by message beforehand that it ran nothing on the
GPU and no cargo build; `--no-publish` left the Bench history unchanged.
The JSON, with each round's `machine.txt`, is under
`docs/audit/m3.5/bench/verifier-round-<n>/`.

- **Load.** The one-minute load average read 12.7, 19.3, 18.7 and 14.7
  at the four starts, with Spotlight's `mds` and `mdworker_shared` at
  the top of `ps`, about 2.4 cores between them, after the session had
  created two worktrees; the fourth round waited 30 minutes for a load under 3
  and did not get one. So no round here or in the readers' sections
  (3.0 to 17.0) is quiet by load average, and this re-run cannot settle
  what L3-D03, L3-D04 and L3-L01 leave open about it: their CPU lines
  reproduce under similar or heavier load, and read higher as the load
  rises. A run on a machine with Spotlight idle is what settles them.
- **GPU load.** Preflight read 0% before rounds 1, 2 and 4, and 84%
  before round 3, whose eleven Provenances stamp 46% and mark every run
  not quiet; what ran on the GPU then was not traced. Round 3's GPU
  medians agree with the other three rounds' (`town`'s GPU frame within
  0.3%), so it is kept and named wherever it matters. In rounds 1, 2 and
  4, 32 of 33 runs are stamped quiet, texture-wall at p95 over median
  1.019 to 1.049, which is what killed L3-D07.
- **Agreement with the readers**, at the median of medians: GPU frame
  within 0.1% on `town` and 1.2% on every light scene, gate within 1.2%
  and 1.7%; texture-wall 3.0% lower on its GPU frame (14,870.5 against
  15,337.4 µs) and 5.2% on its gate (13,112.7 against 13,830.3), outside
  either side's round-to-round spread, which narrowed L3-D05 and L3-D06
  to both sets of rounds' range.
- **Floors** the Findings use were recomputed: culling phase one's
  139.36 MB at 273 GB/s [ref:apple-m4-pro-bandwidth] is 510.5 µs, as
  L3-D09 says.
- **Notes the verifier could not close.** Three architectural notes put
  a pass over twice the only floor they compute. The lane's rule makes a
  pass over 2x its floor a Finding; these stay open notes instead,
  because what is in doubt is each floor, not the pass's time, and the
  measurement that fixes the floor is not taken. The triage decides
  whether to take it: `town`'s Froxel pass reads
  604.4 µs against a 138 µs bandwidth floor (4.4x), but its line overlaps
  culling phase one's, so the pass alone needs timing; `town`'s depth
  pre-pass and lit pass read 2.45 and 2.50 times their every-vertex-once
  floors (870 and 1,499 µs), a floor that assumes full Meshlets where
  most of `town`'s Instances draw a Mesh of one Meshlet, so the cooked
  Meshlets' vertex count or the constant-pull probe settles it; the
  helmet's shadow mask pass is 3.26 times its floor at uv-grid's ray
  rate, and no ray rate measured outside the scene on a BLAS of its size
  exists. texture-wall's passes over twice their floors are not closed
  by a note: L3-D06 and L3-D08 carry them, the bound Textures' untimed
  stretch and the slow pass level.
- **Corrected in the readers' text**: the dense section called its
  round 3 the round with the lowest load, where its own run conditions
  give round 3 the highest (17.0 against 3.0 and 3.2), so the phrase is
  removed; `QUIET_P95_OVER_MEDIAN` is cited at `src/bench/provenance.rs:30`
  where it was cited at its doc comment's line 29, and `is_quiet` at
  130 to 140; the two sentences that pointed at L3-D07 now say it was
  killed; and the light section's evidence paragraph for L3-D07 moved
  with it to Killed.

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
(1.06 in `src/bench/provenance.rs:30`): its GPU load, processes and power
checks passed in every round. Its GPU frame p95 sits 7 to 11% over its
median in these rounds at 2 to 9% GPU load, 5.4% in its Baseline run
(16,839 against 15,975 µs) and 1.6 to 3.5 ms over it in four of the M2
exit's six runs, so the stamp flips with the scene's own spread, not
the machine's state: L3-D07, which the verifier's re-run killed (see
Killed).

### `town`

Round 3 as `praxis report` prints it, the round with the tightest
p95s; the other rounds' tables are in the bench JSON and
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

| id | claim | severity | where | evidence | expected | verifier | placement |
|---|---|---|---|---|---|---|---|
| L3-D01 | `town`'s GPU frame breaches its 12 ms Budget by 8.0 ms at the frozen commit | `budget`, 7,988 µs over at the median of three rounds (7,984 to 8,075) | scene `town`; `bench/budgets.json`, `gpu_frame_microseconds` | three quiet rounds, GPU frame medians 19,983.7, 19,987.8 and 20,075.1 µs (p95 20,210 to 20,874), `docs/audit/m3.5/bench/round-{1,2,3}/town.json`; shadow-ray time 10,750 to 10,858 of it, the frame with rays off 9,130.0 to 9,240.3 (the GPU frame less shadow-ray time, per round) | ≤ 12,000 µs (ADR 0013) | `confirmed`: `town`'s GPU frame reads 19,937.8 to 19,993.2 µs in all four rounds, 7,963 µs over 12,000 at the median of medians, within 0.3% of the reader's 7,988. | fog (#17): M4 charting, in the LOD line with the gap table's sizes |
| L3-D02 | `town`'s gate breaches ADR 0019's 6 ms by 2.7 ms with rays off | `budget`, 2,702 µs over (2,603 to 2,713) | scene `town`; `bench/budgets.json`, `gate_microseconds` | gate medians 8,603.3, 8,702.2 and 8,713.1 µs, same files; its passes: culling 1,341, depth pre-pass 2,142, pyramid 222, phase two's depth 56, lit 3,764 (Froxel 601 beside culling) | ≤ 6,000 µs (ADR 0019); the owner accepted the breach until LOD, which has not landed | `confirmed`: `town`'s gate reads 8,583.2 to 8,636.5 µs in all four rounds, 2,599 µs over 6,000 at the median of medians, within 4% of the reader's 2,702, and the ranges overlap. | fog (#17): M4 charting, in the LOD line with the gap table's sizes |
| L3-D03 | `town`'s dynamic TLAS CPU line is over its 1.1 ms ceiling in every round | `budget`, 92.5 µs over at the median of medians (71.4 to 103.0) | scene `town`; `bench/budgets.json`, `dynamic_tlas_cpu_microseconds`; `src/render/acceleration_structures.rs:356` (`record`, the build's recording) and `src/render/renderer.rs:70` (`dynamic_tlas_submit`) | medians 1,171.4, 1,192.5 and 1,203.0 µs, p95 1,321 to 1,396, same files; its Baseline reads 1,097.4; the note of 2026-10-30 on `town`'s CPU frame read 934 to 987 quiet and 1,100 to 1,260 under load, and found the line to be wgpu-core's per-Instance dependency walk and record allocation | ≤ 1,100 µs (ADR 0020's ceiling in `bench/budgets.json`) | `narrowed`: `town`'s dynamic TLAS CPU line is over ADR 0013's 1.1 ms ceiling in every round taken under Spotlight load, by 71 to 218 µs (93 µs at the readers' median of medians, 193 at the verifier's) as the load rises; whether it breaches on a quiet machine is open, as its Baseline reads 1,097.4 and the note of 2026-10-30 read 934 to 987 quiet. The breach holds in all four rounds, but at 1,277.2 to 1,318.2 µs (193 µs over at the median of medians, about twice the reader's 92.5) under heavier load, and the ceiling is in ADR 0013, not ADR 0020. | fog (#17): wgpu 31 published, in the wgpu 31 bump line; #528's quiet re-record shows whether it breaches at all |
| L3-D04 | `town`'s CPU frame reads 20 to 22% over its Baseline at the median in every round, and over 6 ms at p95, which `compare` reports but does not judge | `health`, 944 to 1,058 µs over its Baseline median; p95 121 to 418 µs over the Budget the medians (207 to 321 under) do not breach | scene `town`; `bench/budgets.json`, `cpu_frame_microseconds` | medians 5,679.2, 5,709.8 and 5,793.1 µs against a Baseline of 4,734.8; p95 6,121.0, 6,151.0 and 6,418.2 against 6,000; the five-minute load average was 8 to 16 during the rounds from Spotlight indexing, which the preflight checks do not see, so the magnitude needs a re-run on a machine without it; `compare` fails the line against its Baseline in every round | within 5% of its Baseline median; medians count for the Budget (`src/bench/mod.rs:629-630`) | `confirmed`: Under similar or heavier load the CPU frame reads 5,706.2 to 5,861.1 µs, 1.205 to 1.238 times its Baseline, with `compare` failing it in every round and p95 over 6,000 in every round (6,083.9 to 6,318.0); the load caveat the Finding states still applies. | fix #566 (with L3-L01) |
| L3-D05 | texture-wall's GPU frame breaches its 12 ms Budget by 3.3 ms | `budget`, 3,337 µs over (3,304 to 3,465) | scene texture-wall; `bench/budgets.json`, `gpu_frame_microseconds` | GPU frame medians 15,304.0, 15,337.4 and 15,465.0 µs, `docs/audit/m3.5/bench/round-{1,2,3}/texture-wall.json`; GPU load 2 to 9%, AC power, no other praxis process; stamped not quiet by the p95 rule alone (L3-D07) | ≤ 12,000 µs | `narrowed`: texture-wall's GPU frame breaches its 12 ms Budget in every round, by 2.7 to 3.5 ms across the seven rounds (2,870 µs at the median of the verifier's four, 3,337 at the readers' three). The breach holds in all four rounds, but at 14,699.6 to 14,972.3 µs, 2,870 µs over at the median of medians, 14% under the reader's 3,337, and the two sets of rounds do not overlap. | fix #565 (with L3-D06) |
| L3-D06 | texture-wall's gate breaches 6 ms by 7.8 ms; of its 13.8 ms, about 11.0 ms is a stretch no pass times, between culling phase two's end and the lit pass's start, in a scene of 200 triangles and 10,005 bound Textures | `budget`, 7,830 µs over (7,530 to 7,885) | scene texture-wall; `bench/budgets.json`, `gate_microseconds`; the material bind group that binds every Texture, `src/render/renderer.rs:278` and `:636` (`MATERIALS`) | gate medians 13,530.1, 13,830.3 and 13,884.9 µs; the gate's timed passes sum to 2,850 µs; the RTX 2070 SUPER Baselines give 514 (Vulkan) and 502 µs (DX12) for the same scene | ≤ 6,000 µs; a floor under 1 µs for the scene's geometry | `narrowed`: texture-wall's gate breaches 6 ms by 7.1 to 7.9 ms across the seven rounds (7,113 µs at the median of the verifier's four); its rays-on pass lines up to the lit pass sum to 2,852 µs, which leaves about 10.2 to 10.4 ms of its 13.1 ms that no pass line covers, in a scene of 200 triangles and 10,005 bound Textures. The note of 2026-10-30 places that time between culling phase two's end and the lit pass's start. The breach holds in all four rounds, but by 7,113 µs at the median of medians (9% under the reader's 7,830); the verifier's medians leave about 10.3 ms that no pass line covers, not 11.0; and the pass lines it subtracts come from the rays-on frames, not the rays-off frames the gate times. | fix #565 |
| L3-D08 | texture-wall's Baseline holds the fast of its two pass levels, so every run on the slow level, all three here, fails seven pass rows in `compare` and warns on two while its GPU frame reads 4% under the Baseline | `health`, 7 pass rows failed and 2 warned per run, 1 scene | `bench/baselines/apple-m4-pro/texture-wall.json` (`depth_prepass` median 398.6, the fast level); `src/bench/mod.rs:135` (`FAIL_RATIO`), `:143` (the 20 µs noise floor) and `:710` (the pass-row compare) | `praxis compare` on each round: `depth_prepass` 725.8 to 726.2 against 398.6 (+82%), `depth_pyramid_pass` 886.2 to 886.5 against 490.5, `tonemapping_pass` 1,480.7 to 1,480.8 against 811.2, `lit_pass` 1,030.8 to 1,032.5 against 607.2, `shadow_mask_pass` 4,961.4 to 4,964.1 against 2,973.2, both culling phases +65 to +72%; `froxel_pass` (+14.2 µs) and `phase_two_depth_pass` (+12.3 µs) warn under the 20 µs noise floor; the GPU frame 15,304 to 15,465 against 15,975; the pyramid and tonemapping do `town`'s exact work in 4.0 and 4.6 times its lines | pass rows judged at the level a run lands on, or a Baseline at the slow level as the M3 exit kept, until the bound-Textures fix settles the levels | `narrowed`: texture-wall's Baseline holds the faster of its two pass levels, so every run on the slow level (all seven here) fails seven pass rows in `compare` and warns on two, while its GPU frame reads 3 to 8% under the Baseline. `compare` fails the same seven texture-wall pass rows and warns on the same two in all four rounds. But the GPU frame reads 6.3 to 8.0% under its Baseline, not 4%, and the pass-row compare is `src/bench/mod.rs:706-715` (line 710 is the noise-floor assignment). | fix #528 (folded into the exit's quiet re-record, texture-wall at the slow level) |
| L3-D09 | culling phase one's line reads over twice its bandwidth floor on `town` | `health`, 654 µs over its 510 µs floor (2.28x); 1,164 µs a frame | `src/render/culling_pass.rs:284-320` (`record`, phase one's four dispatches); `src/render/shaders/culling.wgsl:401` (`append_triangles`) | medians 1,148.5, 1,164.3 and 1,165.1 µs against 139.3 MB / 273 GB/s = 510 µs (the arithmetic above); the stage split of 2026-10-30 summed its stages to 886 µs (1.74x) with 685 of them the 66.6 MB of packed-index writes (1.7x their own floor), and found the line to include time beside the Froxel pass, which the bench starts beside it | within 2x of its floor; or the line counting the pass alone, as the render passes' lines do since the note of 2026-10-30 | `confirmed`: Culling phase one reads 1,154.5 to 1,167.2 µs (1,163.3 at the median of medians); the floor recomputes at 139.36 MB / 273,000 = 510.5 µs, so the ratio is 2.28 as the reader says. | fog (#17): M4 charting, in the LOD line with the gap table's sizes |

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

## The rest: the light scenes, and the revive-when lines

Scope: the nine Reference scenes with a Baseline other than `town` and
texture-wall (crowd, far-coplanar, helmet, minification, normal-tangent, orrery, pbr-sweep, shadows and uv-grid), benched on the frozen commit as
the dense section benched its two, every timed pass against its hardware
floor on the Apple M4 Pro and against its source's published cost; then
every "revive when" line of the ladder's map (*Not yet specified*) and the
two open offender tickets placed at M4's charting, each re-judged `fired`,
`waiting` or `dead`. One reader at high effort with the lane brief
verbatim; two read-only sub-readers gathered the revive lines' evidence,
half each, and this reader checked it as the revive section says. Finding
ids are `L3-L<n>`. The report
tables are pasted from `praxis report` and `praxis compare`; every other
figure taken from this section's runs, in tables and prose alike, is
computed from the bench JSON under `docs/audit/m3.5/bench/light-round-<n>/`,
the captures' logs and the round stamps by one script kept in the owner's
checkout under `pending-artifacts/502/` (`light_section.py`, which wrote
this section's text). Figures from earlier notes in `docs/performance.md`
are named with their note.

### Run conditions

Apple M4 Pro (16-core GPU), macOS 27.0.1, wgpu 30.0.1 on Metal, release
build of `8bc34eb18fcfb79307193551e96619316aef566c` in a worktree detached
at it (every Provenance: commit 8bc34eb1, `dirty: false`),
its Cooked cache and fetched files copied from a checkout of `main` whose
measured paths are byte-identical to the frozen commit (`git diff --stat
8bc34eb1 main -- src assets Cargo.toml Cargo.lock .cargo` prints nothing).
`praxis bench <dir> helmet pbr-sweep uv-grid normal-tangent far-coplanar
orrery minification shadows crowd` at 2560×1440 with 4x MSAA, 248 warm-up
and 1,800 measured frames per scene, rays on then off, aliasing counters
included, under `caffeinate -i`, four rounds back to back on
2026-11-02 from 17:39 to 17:48 UTC, 98 to 101 s each and
24 to 61 s apart, each round's start and end in its `machine.txt`: AC
power with the battery 100%, no other praxis process (`other_processes`
empty in every Provenance), and the bench's own GPU load reading at each
round's start 0 to 1%. The preflight lines in `machine.txt`, taken seconds
before each bench, read 1%, 99%, 99% and 0%: at rounds 2 and 3 the same `ioreg` counter read 99% and,
moments later in the bench, 1% and 0%. That counter covers about the last second:
read one second after a short bench it says 94% and a second later 0%
(`preflight-after-bench.txt`, kept with the script), so some GPU work
ran in the second before those two preflights; what it was was not
traced. Four peer sessions confirmed by message that they ran nothing on
the GPU and no cargo build during the rounds. One-minute load averages
at the starts were 7.3, 5.8, 3.7 and 5.8 (5.9 to 10.9 over five), while this
ticket's two read-only sub-readers and peer sessions read files; nothing
built. The ticket asked for three rounds; a fourth ran after round 2's
p95 stamps came back noisy, and all four are kept, listed and used, the
median of medians being the mean of the middle two. The rounds published
their records to `bench-history` as the bench does by default (ADR 0024),
which keeps a record under its key "only when quieter"
(`src/bench/history.rs:928-933`), so round 1 published nine, round 2 one
(far-coplanar's, quiet in round 2 and not in round 1) and rounds 3 and 4
none.

The RTX 2070 SUPER has Baselines for all nine scenes; that machine was not
run here. Its Baselines, for context only, as the brief allows:

| Scene | GPU frame, Vulkan, µs | GPU frame, DX12 | Gate, Vulkan | Gate, DX12 |
|---|---:|---:|---:|---:|
| helmet | 2,736.3 | 2,261.0 | 865.1 | 918.8 |
| pbr-sweep | 1,692.5 | 1,173.5 | 548.0 | 524.3 |
| uv-grid | 1,747.0 | 1,462.3 | 537.6 | 506.9 |
| normal-tangent | 1,680.2 | 1,257.5 | 565.2 | 565.2 |
| far-coplanar | 1,594.7 | 1,353.7 | 715.6 | 819.5 |
| orrery | 1,424.8 | 804.4 | 422.4 | 409.9 |
| minification | 1,846.3 | 1,589.2 | 634.8 | 670.0 |
| shadows | 2,338.8 | 1,985.3 | 672.9 | 688.7 |
| crowd | 2,169.8 | 1,550.1 | 840.5 | 850.7 |

Quiet verdicts from the Provenance in each file (`quiet`,
`gpu_frame_p95_over_median`), and the ratio in the scene's Baseline run:

| Scene | Round 1 | Round 2 | Round 3 | Round 4 | Baseline run |
|---|---|---|---|---|---|
| helmet | quiet, 1.004 | not quiet, 1.090 | quiet, 1.028 | quiet, 1.029 | 1.003 |
| pbr-sweep | quiet, 1.027 | not quiet, 1.160 | quiet, 1.048 | not quiet, 1.062 | 1.030 |
| uv-grid | quiet, 1.027 | quiet, 1.057 | quiet, 1.058 | not quiet, 1.066 | 1.007 |
| normal-tangent | quiet, 1.044 | quiet, 1.056 | quiet, 1.047 | quiet, 1.059 | 1.006 |
| far-coplanar | not quiet, 1.067 | quiet, 1.028 | not quiet, 1.068 | not quiet, 1.126 | 1.010 |
| orrery | quiet, 1.041 | not quiet, 1.162 | quiet, 1.052 | not quiet, 1.117 | 1.041 |
| minification | quiet, 1.017 | not quiet, 1.207 | quiet, 1.058 | not quiet, 1.105 | 1.004 |
| shadows | quiet, 1.012 | not quiet, 1.156 | quiet, 1.045 | not quiet, 1.083 | 1.007 |
| crowd | quiet, 1.041 | quiet, 1.054 | quiet, 1.047 | not quiet, 1.090 | 1.010 |

22 of the 36 runs are stamped quiet. The 14 others (1, 5, 1 and 7 in rounds
1 to 4, at 1.062 to 1.207) fail the p95-over-median rule alone
(`src/bench/provenance.rs:30`, 1.06), their GPU load, process and power
checks having passed. Across the four rounds every scene's GPU frame
median agrees within 0.97%, its gate within 1.01% and every pass line
over 20 µs within 4.5%, so the medians stand and the stamps record
the p95s' spread; far-coplanar is stamped not quiet in 3 of 4 rounds at
1.067 to 1.126 against 1.010 in its Baseline run. This is L3-D07's
defect on nine more scenes, and is written up as evidence for it, not
as a Finding of its own; it now sits with L3-D07 under Killed.

### Against the Budgets and the Baselines

No light scene breaches a Budget at the frozen commit. At the median of
the four rounds' medians: GPU frames from 2,627.1 µs (far-coplanar) to
6,683.6 (helmet) against 12,000; the gate, rays off, from
648.4 (orrery) to 1,272.7 (crowd) against 6,000; CPU frames
from 210.3 (uv-grid) to 792.8 (crowd) against 6,000, and at p95 at
most 1,064.7 (crowd); the dynamic TLAS's CPU line 198.7 on the
orrery and 381.1 on the crowd against 1,100, under 1 µs elsewhere, its
build 0 at the median on both; shadow-ray time from 1,619.5
(far-coplanar) to 5,144.5 (helmet); GPU memory from 663,286,515 B
(far-coplanar) to 683,925,797 (helmet) of 8 GB. Every counter equals
its Baseline's in every round (true for `draw_calls`, `instances_drawn`, `triangles_drawn`, `instances_overflowed`, `uploaded_bytes`, `gpu_memory_bytes`, `arena_bytes_in_use`, `heap_allocations`),
and the aliasing counters too (true).

`praxis compare` against the M4 Pro Baselines (recorded at 732e58c7,
2026-11-01; the four commits since that touch measured paths, 91fda48b,
e3a96ae0, 9978e8d2 and 2c30aa92, change `src/bench/` and
`src/bin/praxis.rs` only), on each round:

| Scene | CPU frame medians, rounds | Baseline | Ratio | CPU p95s | `culling_phase_one` medians | Baseline | Ratio | GPU frame against Baseline |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| helmet | 237.9, 307.7, 220.0, 251.2 | 186.1 | 1.18 to 1.65 | 452–557 | 75.6, 75.3, 75.6, 74.8 | 76.5 | 0.98 to 0.99 | 0.999 to 0.999 |
| pbr-sweep | 219.9, 273.5, 208.8, 206.1 | 181.6 | 1.14 to 1.51 | 332–506 | 33.5, 32.8, 33.6, 33.2 | 33.8 | 0.97 to 1.00 | 1.003 to 1.009 |
| uv-grid | 245.7, 208.2, 202.8, 212.5 | 184.0 | 1.10 to 1.34 | 333–467 | 27.4, 27.5, 28.3, 27.1 | 24.2 | 1.12 to 1.17 | 1.005 to 1.006 |
| normal-tangent | 214.0, 234.2, 209.4, 222.4 | 208.6 | 1.00 to 1.12 | 339–462 | 40.7, 40.3, 40.8, 40.2 | 37.7 | 1.07 to 1.08 | 1.005 to 1.006 |
| far-coplanar | 227.3, 225.3, 241.0, 244.8 | 185.8 | 1.21 to 1.32 | 394–513 | 28.8, 28.0, 28.5, 28.4 | 24.2 | 1.15 to 1.19 | 1.009 to 1.012 |
| orrery | 386.0, 413.1, 443.4, 418.3 | 358.8 | 1.08 to 1.24 | 566–760 | 33.5, 33.1, 33.3, 33.1 | 34.2 | 0.97 to 0.98 | 1.006 to 1.016 |
| minification | 246.4, 232.3, 230.7, 233.4 | 192.7 | 1.20 to 1.28 | 419–459 | 27.6, 27.2, 27.5, 27.2 | 28.5 | 0.96 to 0.97 | 1.001 to 1.002 |
| shadows | 261.0, 322.8, 240.4, 266.7 | 265.9 | 0.90 to 1.21 | 457–567 | 33.4, 33.3, 33.4, 33.6 | 28.6 | 1.16 to 1.17 | 1.006 to 1.010 |
| crowd | 816.6, 793.9, 739.0, 791.8 | 744.5 | 0.99 to 1.10 | 982–1,065 | 92.9, 92.9, 92.3, 92.7 | 93.0 | 0.99 to 1.00 | 1.001 to 1.005 |

- **The CPU frame line fails on 5 scenes in every round**
  (helmet, pbr-sweep, uv-grid, far-coplanar and minification: 1.10 to 1.65 times the Baseline); of the
  others, normal-tangent fails in round 2 and warns in round 4; orrery fails in rounds 2, 3 and 4 and warns in round 1; shadows fails in round 2; crowd warns in rounds 1, 2 and 4. `compare` fails 5, 8, 6 and 6 lines in rounds 1 to 4,
  every one a CPU frame; the GPU frame is within 1.6% of its Baseline on every
  scene in every round. The lines are 4 to 13% of the 6 ms Budget: L3-L01.
- **Culling phase one reads 27.1 to 28.8 µs on the 3 scenes of two to
  four triangles in all 12 of their runs** (uv-grid, far-coplanar and minification), against
  Baselines of 24.2, 24.2 and 28.5; `compare` warns on it for
  uv-grid, normal-tangent, far-coplanar and shadows in every round, under its 20 µs noise floor
  (`src/bench/mod.rs:143`); helmet, pbr-sweep, orrery, minification and crowd read 0.96 to 1.00
  of theirs: L3-L02.
- **far-coplanar's lit pass** reads 359.8 to 364.7 against 344.5 (+4.4
  to +5.8%), a warning in 3 rounds; its gate and GPU frame are within
  1.23% of theirs.

### Floors

Apple's figure for the M4 Pro is 273 GB/s of memory bandwidth
[ref:apple-m4-pro-bandwidth], 273,000 bytes a microsecond; every bandwidth
floor below is bytes moved, uncompressed, over it, with the dense
section's byte counts: a depth sample 4 B, an HDR sample 8 B, the shadow
mask's two words 12 B a pixel, an Instance 128 B, a Meshlet record 12 B, a
packed index 4 B, a Meshlet vertex index 4 B and a cooked vertex 28 B, so
an index pulled with no vertex reuse moves 36 B; 3,686,400 pixels of 4
samples, so the depth target is 59.0 MB, the HDR target 118.0 MB and the
mask words 44.2 MB. The depth pyramid's and the tonemapping pass's floors
are the dense section's, 234 and 486 µs, the work being the same on
every scene. Culling and the Froxel pass move almost nothing on these
scenes, so their floors add the dispatches' fixed cost as uv-grid's own
lines read it on two triangles (27.4 µs for phase one's four
dispatches, 20.4 for phase two's, 6.9 for the Froxel pass) to the bytes.

The shadow mask pass's floor is its fixed cost plus its rays. Apple
publishes no ray rate for the M4 Pro, so both come from the note of
2026-10-30 on where the shadow mask pass's time goes, measured at
`99fdb38e`: with its ray code held and every ray skipped the pass read
1,141 µs on uv-grid and 1,150 on the helmet, and a ray cost at least 0.08
ns on uv-grid's two triangles and 0.96 on the helmet's 257,000. The
shipped pass holds that code, so its floor takes the 1,150 and the rays
at either rate; the dense section's range for `town` ran from the code
compiled out to the code held, and the code is not compiled out in any
shipped build. The note's figures predate the merge of every ray into
one ray-query call (2026-10-31), which the note found to cost uv-grid
0.09 to 0.12 ms and to save the helmet 0.83 to 0.95 ms. A scene's
traversal is taken to be no cheaper than uv-grid's two triangles. Rays a
pixel come from `praxis capture <scene> <png> 2560x1440` on the frozen
build, its last frame's lit pixels and the point lights whose range
reaches each:

| Scene | Lit pixels | Point lights in the scene | Reaching each lit pixel |
|---|---:|---:|---:|
| helmet | 904,822 | 4 | 4.00 |
| pbr-sweep | 576,047 | 4 | 4.00 |
| uv-grid | 1,071,689 | 4 | 4.00 |
| normal-tangent | 824,786 | 4 | 4.00 |
| far-coplanar | 2,147,448 | 4 | 0.00 |
| orrery | 137,997 | 4 | 4.00 |
| minification | 2,449,361 | 1 | 1.00 |
| shadows | 3,248,640 | 1 | 1.00 |
| crowd | 2,018,982 | 0 | 0.00 |

far-coplanar's planes lie 2,000 m out (`FAR_COPLANAR_DISTANCE`,
`src/scenes/mod.rs:588`) and its four coloured lights reach 20 m from 2 m
off the origin, so the sun alone reaches its pixels. The crowd's bend
pass floor is its 100 figures' vertices read and written, 66 × 41 = 2,706
a figure (`uv_sphere(1.0, 40, 65)`, `src/scenes/mod.rs:932-933`), 28 B
each way. Source costs scale from 1920×1080 by pixels, ×1.7778.

### helmet

Round 1 as `praxis report` prints it; the rounds table and the floors
follow.

**helmet** on Apple M4 Pro, release, 2560x1440, 1800 measured frames

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 237.9 | 467.5 | 186.1 | 264.8 | +51.8 | 1.278 | 6,000.0 |
| Dynamic TLAS, CPU | 0.1 | 0.5 | 0.1 | 0.2 | +0.0 | 1.494 | 1,100.0 |
| GPU frame | 6,679.3 | 6,703.8 | 6,689.1 | 6,711.9 | -9.8 | 0.999 | 12,000.0 |
| Gate, rays off | 1,214.2 | 1,223.6 | 1,210.1 | 1,216.5 | +4.1 | 1.003 | 6,000.0 |
| Shadow-ray time | 5,141.1 | 5,165.8 | 5,155.2 | 5,178.7 | -14.0 | 0.997 | — |
| `culling_phase_one` | 75.6 | 76.8 | 76.5 | 77.5 | -0.9 | 0.988 | — |
| `culling_phase_two` | 20.7 | 21.6 | 20.8 | 24.1 | -0.2 | 0.992 | — |
| `depth_prepass` | 237.0 | 238.8 | 238.9 | 240.8 | -1.9 | 0.992 | — |
| `depth_pyramid_pass` | 205.2 | 206.7 | 205.5 | 206.6 | -0.3 | 0.998 | — |
| `dynamic_tlas_build` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | 1,000.0 |
| `froxel_pass` | 6.9 | 7.3 | 7.0 | 7.4 | -0.1 | 0.988 | — |
| `lit_pass` | 807.5 | 813.2 | 808.5 | 812.3 | -1.0 | 0.999 | — |
| `phase_two_depth_pass` | 12.2 | 12.6 | 12.2 | 12.7 | +0.0 | 1.000 | — |
| `shadow_mask_pass` | 4,967.9 | 4,989.6 | 4,972.3 | 4,994.0 | -4.5 | 0.999 | — |
| `tonemapping_pass` | 319.9 | 322.6 | 322.2 | 322.7 | -2.3 | 0.993 | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 4 | 4 | +0 | 1.000 | — |
| `instances_drawn` | 11 | 11 | +0 | 1.000 | — |
| `triangles_drawn` | 256,938 | 256,938 | +0 | 1.000 | — |
| `uploaded_bytes` | 416 | 416 | +0 | 1.000 | — |
| `gpu_memory_bytes` | 683,925,797 | 683,925,797 | +0 | 1.000 | 8,000,000,000 |
| `arena_bytes_in_use` | 844,804 | 844,804 | +0 | 1.000 | — |
| `heap_allocations` | 347 | 347 | +0 | 1.000 | — |
| `aliasing_rmse` | 13,706 | 13,706 | +0 | 1.000 | — |
| `aliasing_p99` | 39,984 | 39,984 | +0 | 1.000 | — |

The four rounds, each line's medians lowest to highest, then the median
of medians, then the p95s; every counter identical across the rounds:

| Line | Medians, lowest to highest | Median of medians | p95s |
|---|---:|---:|---:|
| CPU frame | 220.0–237.9–251.2–307.7 | 244.5 | 451.7–467.5–553.0–556.9 |
| Dynamic TLAS, CPU | 0.1–0.1–0.1–0.1 | 0.1 | 0.4–0.5–0.5–0.6 |
| GPU frame | 6,679.3–6,681.5–6,685.6–6,685.7 | 6,683.6 | 6,703.8–6,866.8–6,876.5–7,288.8 |
| Gate, rays off | 1,213.5–1,214.2–1,214.7–1,215.5 | 1,214.4 | 1,223.6–1,226.3–1,297.5–1,376.4 |
| Shadow-ray time | 5,141.1–5,143.2–5,145.8–5,146.1 | 5,144.5 | 5,165.8–5,330.0–5,337.6–5,742.3 |
| `culling_phase_one` | 74.8–75.3–75.6–75.6 | 75.4 | 76.2–76.8–77.0–77.3 |
| `culling_phase_two` | 20.3–20.7–20.7–20.7 | 20.7 | 21.6–21.6–21.7–22.8 |
| `depth_prepass` | 236.5–237.0–237.0–237.0 | 237.0 | 238.8–239.3–239.4–239.6 |
| `depth_pyramid_pass` | 205.0–205.1–205.2–205.2 | 205.1 | 206.6–206.7–207.0–207.5 |
| `froxel_pass` | 6.8–6.8–6.8–6.9 | 6.8 | 7.2–7.2–7.3–7.3 |
| `lit_pass` | 807.5–807.8–809.0–809.1 | 808.4 | 813.2–814.8–817.0–834.4 |
| `phase_two_depth_pass` | 12.2–12.2–12.2–12.2 | 12.2 | 12.6–12.6–12.7–12.7 |
| `shadow_mask_pass` | 4,967.9–4,972.6–4,973.5–4,977.9 | 4,973.1 | 4,989.6–5,155.5–5,161.2–5,500.5 |
| `tonemapping_pass` | 319.6–319.7–319.8–319.9 | 319.8 | 322.2–322.6–322.6–323.0 |

Each timed pass against its floor and, where one exists, its source (the
median of medians; p95s over the rounds):

| Pass | Median µs | p95 µs (rounds) | Floor µs and arithmetic | Ratio to floor | Source µs, scaled | Ratio to source | Verdict |
|---|---:|---:|---|---:|---|---:|---|
| `culling_phase_one` | 75.4 | 76–77 | 42: 14.2 by bytes (3.9 MB: 11 Instances × 128 B scanned, 770,814 × 4 B of packed indices written, 256,938 × 3 B of Meshlet triangles, the Instances re-read, 2,073 records × 12 B) + 27.4, the four dispatches' fixed cost as uv-grid's line reads it | 1.81 | 670 (Haar and Aaltonen) counts 250,000 objects | does not apply | within 2x of its floor |
| `culling_phase_two` | 20.7 | 22–23 | 20: 0.1 by bytes (the same scan and the records) + 20.4, uv-grid's line | 1.01 | counted with phase one | — | within 2x of its floor |
| `depth_prepass` | 237.0 | 239–240 | 243 to 318: 7.3 MB (every vertex pulled once) to 27.7 MB (770,814 indices × 36 B) of vertex pull, the Instances and records, and 59.0 MB of depth written | 0.75 to 0.98 | none cited | — | under its uncompressed floor: the target is read compressed (note) |
| `depth_pyramid_pass` | 205.1 | 207–208 | 234: 63.9 MB: 59.0 MB of samples read, level 0 and the levels after written | 0.88 | 107 (Haar and Aaltonen, from HTILE); 73 (Wihlidal, 16-bit tiles) | 1.92 / 2.81 | under its uncompressed floor: the target is read compressed (note); over 2x a source (note) |
| `froxel_pass` | 6.8 | 7–7 | 8: 1.4 by bytes (369 KB: 18,432 Froxels × a count and 4 lights × 4 B) + 6.9, uv-grid's line | 0.83 | 320 to 900 (Drobot) counts 256 lights | does not apply | at its floor, the dispatches' fixed cost |
| `lit_pass` | 808.4 | 813–834 | 838 to 913: the vertex pull, 59.0 MB of depth read at Equal, 118.0 MB of HDR written, 44.2 MB of mask words read and the Froxel lists: 228.9 MB to 249.3 MB | 0.89 to 0.96 | 8,180 to 10,130 (Drobot, forward+) counts 256 lights; 4 here | does not apply | under its uncompressed floor: the target is read compressed (note) |
| `phase_two_depth_pass` | 12.2 | 13–13 | not floored: phase two's triangle count is not in the counters | — | none cited | — | not floored; under 0.1 ms |
| `shadow_mask_pass` | 4,973.1 | 4,990–5,501 | 1,512 to 5,493: 1,150, the held ray code's measured fixed cost, + 904,822 lit pixels × (the sun + 4.00 point lights reaching) = 4,524,110 rays × 0.08 to 0.96 ns (362 to 4,343) | 0.91 to 3.29 | 6,040 to 8,530 (Boksansky et al., four lights; five reach here, scaled by pixels) | 0.58 to 0.82 | floor not computable: no ray rate for a BLAS of this size is measured apart from this scene (note) |
| `tonemapping_pass` | 319.8 | 322–323 | 486: 132.7 MB: 118.0 MB of HDR samples read, 14.7 MB written | 0.66 | none with a stated resolution | — | under its uncompressed floor: the target is read compressed (note) |

### pbr-sweep

Round 1 as `praxis report` prints it; the rounds table and the floors
follow.

**pbr-sweep** on Apple M4 Pro, release, 2560x1440, 1800 measured frames

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 219.9 | 365.2 | 181.6 | 221.5 | +38.3 | 1.211 | 6,000.0 |
| Dynamic TLAS, CPU | 0.1 | 0.5 | 0.1 | 0.1 | +0.0 | 1.012 | 1,100.0 |
| GPU frame | 3,864.6 | 3,968.1 | 3,851.5 | 3,967.1 | +13.1 | 1.003 | 12,000.0 |
| Gate, rays off | 777.2 | 796.3 | 773.8 | 789.2 | +3.5 | 1.004 | 6,000.0 |
| Shadow-ray time | 2,761.5 | 2,853.0 | 2,751.6 | 2,856.8 | +9.9 | 1.004 | — |
| `culling_phase_one` | 33.5 | 34.8 | 33.8 | 35.1 | -0.2 | 0.993 | — |
| `culling_phase_two` | 20.5 | 21.3 | 20.3 | 21.0 | +0.2 | 1.008 | — |
| `depth_prepass` | 163.9 | 164.8 | 164.2 | 167.8 | -0.4 | 0.998 | — |
| `depth_pyramid_pass` | 204.5 | 207.3 | 204.3 | 207.0 | +0.2 | 1.001 | — |
| `dynamic_tlas_build` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | 1,000.0 |
| `froxel_pass` | 6.9 | 7.2 | 6.9 | 7.3 | +0.0 | 0.994 | — |
| `lit_pass` | 474.4 | 490.4 | 470.2 | 488.5 | +4.2 | 1.009 | — |
| `phase_two_depth_pass` | 12.1 | 12.5 | 12.1 | 12.5 | +0.0 | 1.003 | — |
| `shadow_mask_pass` | 2,604.2 | 2,695.4 | 2,606.3 | 2,695.6 | -2.1 | 0.999 | — |
| `tonemapping_pass` | 321.7 | 322.5 | 322.1 | 322.7 | -0.4 | 0.999 | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 4 | 4 | +0 | 1.000 | — |
| `instances_drawn` | 10 | 10 | +0 | 1.000 | — |
| `triangles_drawn` | 22,080 | 22,080 | +0 | 1.000 | — |
| `uploaded_bytes` | 416 | 416 | +0 | 1.000 | — |
| `gpu_memory_bytes` | 663,389,960 | 663,389,960 | +0 | 1.000 | 8,000,000,000 |
| `arena_bytes_in_use` | 76,680 | 76,680 | +0 | 1.000 | — |
| `heap_allocations` | 347 | 347 | +0 | 1.000 | — |
| `aliasing_rmse` | 10,958 | 10,958 | +0 | 1.000 | — |
| `aliasing_p99` | 15,042 | 15,042 | +0 | 1.000 | — |

The four rounds, each line's medians lowest to highest, then the median
of medians, then the p95s; every counter identical across the rounds:

| Line | Medians, lowest to highest | Median of medians | p95s |
|---|---:|---:|---:|
| CPU frame | 206.1–208.8–219.9–273.5 | 214.4 | 331.5–348.8–365.2–505.7 |
| Dynamic TLAS, CPU | 0.1–0.1–0.1–0.1 | 0.1 | 0.4–0.5–0.5–0.6 |
| GPU frame | 3,864.6–3,869.9–3,874.3–3,884.2 | 3,872.1 | 3,968.1–4,056.4–4,113.0–4,505.4 |
| Gate, rays off | 774.5–777.2–779.7–780.1 | 778.4 | 796.3–799.1–812.0–818.3 |
| Shadow-ray time | 2,759.6–2,761.5–2,764.1–2,767.9 | 2,762.8 | 2,853.0–2,950.0–3,002.3–3,393.9 |
| `culling_phase_one` | 32.8–33.2–33.5–33.6 | 33.4 | 34.8–34.8–34.8–34.9 |
| `culling_phase_two` | 20.3–20.3–20.5–20.5 | 20.4 | 21.0–21.1–21.3–21.7 |
| `depth_prepass` | 163.9–164.0–164.1–164.2 | 164.1 | 164.8–165.1–166.8–167.4 |
| `depth_pyramid_pass` | 204.0–204.0–204.5–204.8 | 204.3 | 206.5–206.8–207.3–209.8 |
| `froxel_pass` | 6.8–6.8–6.9–6.9 | 6.9 | 7.2–7.2–7.3–7.4 |
| `lit_pass` | 474.4–475.9–477.6–478.1 | 476.8 | 490.4–492.4–496.8–510.2 |
| `phase_two_depth_pass` | 12.1–12.1–12.2–12.2 | 12.1 | 12.5–12.5–12.5–12.7 |
| `shadow_mask_pass` | 2,604.2–2,606.4–2,609.0–2,614.0 | 2,607.7 | 2,695.4–2,711.3–2,811.5–3,130.1 |
| `tonemapping_pass` | 319.6–319.6–319.8–321.7 | 319.7 | 322.5–322.5–322.7–325.3 |

Each timed pass against its floor and, where one exists, its source (the
median of medians; p95s over the rounds):

| Pass | Median µs | p95 µs (rounds) | Floor µs and arithmetic | Ratio to floor | Source µs, scaled | Ratio to source | Verdict |
|---|---:|---:|---|---:|---|---:|---|
| `culling_phase_one` | 33.4 | 35–35 | 29: 1.2 by bytes (0.3 MB: 10 Instances × 128 B scanned, 66,240 × 4 B of packed indices written, 22,080 × 3 B of Meshlet triangles, the Instances re-read, 179 records × 12 B) + 27.4, the four dispatches' fixed cost as uv-grid's line reads it | 1.16 | 670 (Haar and Aaltonen) counts 250,000 objects | does not apply | within 2x of its floor |
| `culling_phase_two` | 20.4 | 21–22 | 20: 0.0 by bytes (the same scan and the records) + 20.4, uv-grid's line | 1.00 | counted with phase one | — | within 2x of its floor |
| `depth_prepass` | 164.1 | 165–167 | 218 to 225: 0.6 MB (every vertex pulled once) to 2.4 MB (66,240 indices × 36 B) of vertex pull, the Instances and records, and 59.0 MB of depth written | 0.73 to 0.75 | none cited | — | under its uncompressed floor: the target is read compressed (note) |
| `depth_pyramid_pass` | 204.3 | 207–210 | 234: 63.9 MB: 59.0 MB of samples read, level 0 and the levels after written | 0.87 | 107 (Haar and Aaltonen, from HTILE); 73 (Wihlidal, 16-bit tiles) | 1.92 / 2.80 | under its uncompressed floor: the target is read compressed (note); over 2x a source (note) |
| `froxel_pass` | 6.9 | 7–7 | 8: 1.4 by bytes (369 KB: 18,432 Froxels × a count and 4 lights × 4 B) + 6.9, uv-grid's line | 0.84 | 320 to 900 (Drobot) counts 256 lights | does not apply | at its floor, the dispatches' fixed cost |
| `lit_pass` | 476.8 | 490–510 | 814 to 820: the vertex pull, 59.0 MB of depth read at Equal, 118.0 MB of HDR written, 44.2 MB of mask words read and the Froxel lists: 222.2 MB to 223.9 MB | 0.58 to 0.59 | 8,180 to 10,130 (Drobot, forward+) counts 256 lights; 4 here | does not apply | under its uncompressed floor: the target is read compressed (note) |
| `phase_two_depth_pass` | 12.1 | 12–13 | not floored: phase two's triangle count is not in the counters | — | none cited | — | not floored; under 0.1 ms |
| `shadow_mask_pass` | 2,607.7 | 2,695–3,130 | 1,380 to 3,915: 1,150, the held ray code's measured fixed cost, + 576,047 lit pixels × (the sun + 4.00 point lights reaching) = 2,880,235 rays × 0.08 to 0.96 ns (230 to 2,765) | 0.67 to 1.89 | 6,040 to 8,530 (Boksansky et al., four lights; five reach here, scaled by pixels) | 0.31 to 0.43 | within 2x of its floor even at uv-grid's rate |
| `tonemapping_pass` | 319.7 | 322–325 | 486: 132.7 MB: 118.0 MB of HDR samples read, 14.7 MB written | 0.66 | none with a stated resolution | — | under its uncompressed floor: the target is read compressed (note) |

### uv-grid

Round 1 as `praxis report` prints it; the rounds table and the floors
follow.

**uv-grid** on Apple M4 Pro, release, 2560x1440, 1800 measured frames

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 245.7 | 467.3 | 184.0 | 216.7 | +61.7 | 1.335 | 6,000.0 |
| Dynamic TLAS, CPU | 0.1 | 0.5 | 0.1 | 0.1 | +0.0 | 1.012 | 1,100.0 |
| GPU frame | 2,933.4 | 3,012.9 | 2,917.0 | 2,938.4 | +16.4 | 1.006 | 12,000.0 |
| Gate, rays off | 771.1 | 798.8 | 763.0 | 771.1 | +8.1 | 1.011 | 6,000.0 |
| Shadow-ray time | 1,836.4 | 1,906.3 | 1,830.1 | 1,849.5 | +6.2 | 1.003 | — |
| `culling_phase_one` | 27.4 | 29.2 | 24.2 | 29.3 | +3.2 | 1.131 | — |
| `culling_phase_two` | 20.5 | 21.4 | 20.2 | 20.8 | +0.3 | 1.017 | — |
| `depth_prepass` | 157.2 | 159.7 | 158.8 | 160.4 | -1.6 | 0.990 | — |
| `depth_pyramid_pass` | 205.1 | 208.6 | 205.0 | 208.4 | +0.1 | 1.001 | — |
| `dynamic_tlas_build` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | 1,000.0 |
| `froxel_pass` | 6.9 | 7.3 | 6.9 | 7.4 | +0.0 | 1.000 | — |
| `lit_pass` | 456.2 | 469.2 | 447.8 | 458.9 | +8.4 | 1.019 | — |
| `phase_two_depth_pass` | 12.1 | 12.5 | 12.0 | 12.4 | +0.1 | 1.010 | — |
| `shadow_mask_pass` | 1,707.8 | 1,718.5 | 1,705.8 | 1,712.1 | +2.0 | 1.001 | — |
| `tonemapping_pass` | 319.7 | 323.0 | 322.3 | 322.9 | -2.7 | 0.992 | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 4 | 4 | +0 | 1.000 | — |
| `instances_drawn` | 1 | 1 | +0 | 1.000 | — |
| `triangles_drawn` | 2 | 2 | +0 | 1.000 | — |
| `uploaded_bytes` | 416 | 416 | +0 | 1.000 | — |
| `gpu_memory_bytes` | 668,878,947 | 668,878,947 | +0 | 1.000 | 8,000,000,000 |
| `arena_bytes_in_use` | 176 | 176 | +0 | 1.000 | — |
| `heap_allocations` | 345 | 345 | +0 | 1.000 | — |
| `aliasing_rmse` | 6,558 | 6,558 | +0 | 1.000 | — |
| `aliasing_p99` | 33,364 | 33,364 | +0 | 1.000 | — |

The four rounds, each line's medians lowest to highest, then the median
of medians, then the p95s; every counter identical across the rounds:

| Line | Medians, lowest to highest | Median of medians | p95s |
|---|---:|---:|---:|
| CPU frame | 202.8–208.2–212.5–245.7 | 210.3 | 333.2–338.3–381.5–467.3 |
| Dynamic TLAS, CPU | 0.1–0.1–0.1–0.1 | 0.1 | 0.3–0.4–0.5–0.5 |
| GPU frame | 2,932.2–2,933.4–2,933.7–2,933.8 | 2,933.5 | 3,012.9–3,101.6–3,102.1–3,128.8 |
| Gate, rays off | 765.1–767.0–768.5–771.1 | 767.8 | 780.9–784.7–798.8–843.6 |
| Shadow-ray time | 1,836.4–1,840.7–1,841.2–1,843.7 | 1,841.0 | 1,906.3–2,010.3–2,013.4–2,035.8 |
| `culling_phase_one` | 27.1–27.4–27.5–28.3 | 27.4 | 29.0–29.2–29.2–29.7 |
| `culling_phase_two` | 20.2–20.3–20.4–20.5 | 20.4 | 20.9–21.1–21.2–21.4 |
| `depth_prepass` | 157.0–157.0–157.1–157.2 | 157.1 | 158.0–158.0–159.5–159.7 |
| `depth_pyramid_pass` | 204.7–204.8–205.0–205.1 | 204.9 | 207.6–208.2–208.4–208.6 |
| `froxel_pass` | 6.8–6.8–6.9–6.9 | 6.9 | 7.2–7.3–7.3–7.3 |
| `lit_pass` | 455.3–455.4–456.2–456.5 | 455.8 | 466.2–469.2–470.1–471.0 |
| `phase_two_depth_pass` | 12.1–12.1–12.1–12.1 | 12.1 | 12.5–12.5–12.5–12.5 |
| `shadow_mask_pass` | 1,706.8–1,707.2–1,707.8–1,707.8 | 1,707.5 | 1,718.5–1,863.4–1,864.8–1,896.9 |
| `tonemapping_pass` | 319.5–319.5–319.7–320.8 | 319.6 | 322.5–322.6–322.8–323.0 |

Each timed pass against its floor and, where one exists, its source (the
median of medians; p95s over the rounds):

| Pass | Median µs | p95 µs (rounds) | Floor µs and arithmetic | Ratio to floor | Source µs, scaled | Ratio to source | Verdict |
|---|---:|---:|---|---:|---|---:|---|
| `culling_phase_one` | 27.4 | 29–30 | 27: 0.0 by bytes (0.0 MB: 1 Instances × 128 B scanned, 6 × 4 B of packed indices written, 2 × 3 B of Meshlet triangles, the Instances re-read, 1 records × 12 B) + 27.4, the four dispatches' fixed cost as uv-grid's line reads it | 1.00 | 670 (Haar and Aaltonen) counts 250,000 objects | does not apply | at its floor, the dispatches' fixed cost |
| `culling_phase_two` | 20.4 | 21–21 | 20: 0.0 by bytes (the same scan and the records) + 20.4, uv-grid's line | 1.00 | counted with phase one | — | at its floor, the dispatches' fixed cost |
| `depth_prepass` | 157.1 | 158–160 | 216: 0.0 MB (every vertex pulled once) to 0.0 MB (6 indices × 36 B) of vertex pull, the Instances and records, and 59.0 MB of depth written | 0.73 to 0.73 | none cited | — | under its uncompressed floor: the target is read compressed (note) |
| `depth_pyramid_pass` | 204.9 | 208–209 | 234: 63.9 MB: 59.0 MB of samples read, level 0 and the levels after written | 0.88 | 107 (Haar and Aaltonen, from HTILE); 73 (Wihlidal, 16-bit tiles) | 1.92 / 2.81 | under its uncompressed floor: the target is read compressed (note); over 2x a source (note) |
| `froxel_pass` | 6.9 | 7–7 | 8: 1.4 by bytes (369 KB: 18,432 Froxels × a count and 4 lights × 4 B) + 6.9, uv-grid's line | 0.84 | 320 to 900 (Drobot) counts 256 lights | does not apply | at its floor, the dispatches' fixed cost |
| `lit_pass` | 455.8 | 466–471 | 812: the vertex pull, 59.0 MB of depth read at Equal, 118.0 MB of HDR written, 44.2 MB of mask words read and the Froxel lists: 221.6 MB to 221.6 MB | 0.56 to 0.56 | 8,180 to 10,130 (Drobot, forward+) counts 256 lights; 4 here | does not apply | under its uncompressed floor: the target is read compressed (note) |
| `phase_two_depth_pass` | 12.1 | 12–13 | not floored: phase two's triangle count is not in the counters | — | none cited | — | not floored; under 0.1 ms |
| `shadow_mask_pass` | 1,707.5 | 1,719–1,897 | 1,579 to 6,294: 1,150, the held ray code's measured fixed cost, + 1,071,689 lit pixels × (the sun + 4.00 point lights reaching) = 5,358,445 rays × 0.08 to 0.96 ns (429 to 5,144) | 0.27 to 1.08 | 6,040 to 8,530 (Boksansky et al., four lights; five reach here, scaled by pixels) | 0.20 to 0.28 | within 2x of its floor even at uv-grid's rate |
| `tonemapping_pass` | 319.6 | 322–323 | 486: 132.7 MB: 118.0 MB of HDR samples read, 14.7 MB written | 0.66 | none with a stated resolution | — | under its uncompressed floor: the target is read compressed (note) |

### normal-tangent

Round 1 as `praxis report` prints it; the rounds table and the floors
follow.

**normal-tangent** on Apple M4 Pro, release, 2560x1440, 1800 measured frames

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 214.0 | 372.9 | 208.6 | 306.8 | +5.4 | 1.026 | 6,000.0 |
| Dynamic TLAS, CPU | 0.1 | 0.4 | 0.1 | 0.4 | +0.0 | 1.000 | 1,100.0 |
| GPU frame | 3,684.9 | 3,845.9 | 3,664.5 | 3,686.3 | +20.4 | 1.006 | 12,000.0 |
| Gate, rays off | 823.5 | 835.8 | 818.2 | 823.1 | +5.3 | 1.006 | 6,000.0 |
| Shadow-ray time | 2,537.2 | 2,694.3 | 2,522.9 | 2,544.9 | +14.3 | 1.006 | — |
| `culling_phase_one` | 40.7 | 42.2 | 37.7 | 42.2 | +3.0 | 1.080 | — |
| `culling_phase_two` | 20.1 | 21.0 | 19.8 | 20.4 | +0.3 | 1.017 | — |
| `depth_prepass` | 163.1 | 163.9 | 165.2 | 167.9 | -2.1 | 0.987 | — |
| `depth_pyramid_pass` | 204.6 | 206.1 | 204.6 | 206.2 | +0.0 | 1.000 | — |
| `dynamic_tlas_build` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | 1,000.0 |
| `froxel_pass` | 6.8 | 7.2 | 6.8 | 7.2 | +0.0 | 1.000 | — |
| `lit_pass` | 489.2 | 498.7 | 475.8 | 488.9 | +13.4 | 1.028 | — |
| `phase_two_depth_pass` | 12.1 | 12.5 | 12.1 | 12.6 | +0.0 | 1.000 | — |
| `shadow_mask_pass` | 2,407.0 | 2,513.3 | 2,406.0 | 2,413.3 | +1.0 | 1.000 | — |
| `tonemapping_pass` | 319.6 | 320.3 | 319.5 | 320.0 | +0.1 | 1.000 | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 4 | 4 | +0 | 1.000 | — |
| `instances_drawn` | 1 | 1 | +0 | 1.000 | — |
| `triangles_drawn` | 7,774 | 7,774 | +0 | 1.000 | — |
| `uploaded_bytes` | 416 | 416 | +0 | 1.000 | — |
| `gpu_memory_bytes` | 683,213,653 | 683,213,653 | +0 | 1.000 | 8,000,000,000 |
| `arena_bytes_in_use` | 258,368 | 258,368 | +0 | 1.000 | — |
| `heap_allocations` | 345 | 345 | +0 | 1.000 | — |
| `aliasing_rmse` | 18,973 | 18,973 | +0 | 1.000 | — |
| `aliasing_p99` | 35,784 | 35,784 | +0 | 1.000 | — |

The four rounds, each line's medians lowest to highest, then the median
of medians, then the p95s; every counter identical across the rounds:

| Line | Medians, lowest to highest | Median of medians | p95s |
|---|---:|---:|---:|
| CPU frame | 209.4–214.0–222.4–234.2 | 218.2 | 338.5–372.9–443.6–461.8 |
| Dynamic TLAS, CPU | 0.1–0.1–0.1–0.1 | 0.1 | 0.4–0.4–0.5–0.5 |
| GPU frame | 3,683.6–3,684.6–3,684.9–3,685.2 | 3,684.8 | 3,845.9–3,859.8–3,889.8–3,900.2 |
| Gate, rays off | 818.6–819.6–821.9–823.5 | 820.8 | 830.4–835.8–916.5–1,213.8 |
| Shadow-ray time | 2,537.2–2,537.5–2,542.0–2,542.1 | 2,539.7 | 2,694.3–2,715.8–2,739.2–2,754.2 |
| `culling_phase_one` | 40.2–40.3–40.7–40.8 | 40.5 | 42.0–42.2–42.2–42.3 |
| `culling_phase_two` | 20.0–20.0–20.1–20.2 | 20.1 | 20.7–20.8–21.0–21.2 |
| `depth_prepass` | 163.1–163.2–163.3–163.3 | 163.3 | 163.9–164.0–164.7–166.2 |
| `depth_pyramid_pass` | 204.4–204.5–204.6–204.6 | 204.6 | 206.1–207.0–207.2–207.4 |
| `froxel_pass` | 6.8–6.8–6.8–7.0 | 6.8 | 7.2–7.2–7.3–7.3 |
| `lit_pass` | 488.4–488.5–489.2–489.5 | 488.9 | 496.0–498.6–498.7–505.1 |
| `phase_two_depth_pass` | 12.1–12.1–12.2–12.2 | 12.1 | 12.5–12.5–12.5–12.6 |
| `shadow_mask_pass` | 2,406.3–2,406.4–2,406.4–2,407.0 | 2,406.4 | 2,513.3–2,579.5–2,592.2–2,613.7 |
| `tonemapping_pass` | 319.5–319.6–319.6–319.6 | 319.6 | 320.3–321.3–322.5–323.2 |

Each timed pass against its floor and, where one exists, its source (the
median of medians; p95s over the rounds):

| Pass | Median µs | p95 µs (rounds) | Floor µs and arithmetic | Ratio to floor | Source µs, scaled | Ratio to source | Verdict |
|---|---:|---:|---|---:|---|---:|---|
| `culling_phase_one` | 40.5 | 42–42 | 28: 0.4 by bytes (0.1 MB: 1 Instances × 128 B scanned, 23,322 × 4 B of packed indices written, 7,774 × 3 B of Meshlet triangles, the Instances re-read, 63 records × 12 B) + 27.4, the four dispatches' fixed cost as uv-grid's line reads it | 1.45 | 670 (Haar and Aaltonen) counts 250,000 objects | does not apply | within 2x of its floor |
| `culling_phase_two` | 20.1 | 21–21 | 20: 0.0 by bytes (the same scan and the records) + 20.4, uv-grid's line | 0.98 | counted with phase one | — | at its floor, the dispatches' fixed cost |
| `depth_prepass` | 163.3 | 164–166 | 217 to 219: 0.2 MB (every vertex pulled once) to 0.8 MB (23,322 indices × 36 B) of vertex pull, the Instances and records, and 59.0 MB of depth written | 0.75 to 0.75 | none cited | — | under its uncompressed floor: the target is read compressed (note) |
| `depth_pyramid_pass` | 204.6 | 206–207 | 234: 63.9 MB: 59.0 MB of samples read, level 0 and the levels after written | 0.87 | 107 (Haar and Aaltonen, from HTILE); 73 (Wihlidal, 16-bit tiles) | 1.92 / 2.81 | under its uncompressed floor: the target is read compressed (note); over 2x a source (note) |
| `froxel_pass` | 6.8 | 7–7 | 8: 1.4 by bytes (369 KB: 18,432 Froxels × a count and 4 lights × 4 B) + 6.9, uv-grid's line | 0.83 | 320 to 900 (Drobot) counts 256 lights | does not apply | at its floor, the dispatches' fixed cost |
| `lit_pass` | 488.9 | 496–505 | 812 to 815: the vertex pull, 59.0 MB of depth read at Equal, 118.0 MB of HDR written, 44.2 MB of mask words read and the Froxel lists: 221.8 MB to 222.4 MB | 0.60 to 0.60 | 8,180 to 10,130 (Drobot, forward+) counts 256 lights; 4 here | does not apply | under its uncompressed floor: the target is read compressed (note) |
| `phase_two_depth_pass` | 12.1 | 12–13 | not floored: phase two's triangle count is not in the counters | — | none cited | — | not floored; under 0.1 ms |
| `shadow_mask_pass` | 2,406.4 | 2,513–2,614 | 1,480 to 5,109: 1,150, the held ray code's measured fixed cost, + 824,786 lit pixels × (the sun + 4.00 point lights reaching) = 4,123,930 rays × 0.08 to 0.96 ns (330 to 3,959) | 0.47 to 1.63 | 6,040 to 8,530 (Boksansky et al., four lights; five reach here, scaled by pixels) | 0.28 to 0.40 | within 2x of its floor even at uv-grid's rate |
| `tonemapping_pass` | 319.6 | 320–323 | 486: 132.7 MB: 118.0 MB of HDR samples read, 14.7 MB written | 0.66 | none with a stated resolution | — | under its uncompressed floor: the target is read compressed (note) |

### far-coplanar

Round 1 as `praxis report` prints it; the rounds table and the floors
follow.

**far-coplanar** on Apple M4 Pro, release, 2560x1440, 1800 measured frames

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 227.3 | 394.4 | 185.8 | 221.6 | +41.6 | 1.224 | 6,000.0 |
| Dynamic TLAS, CPU | 0.1 | 0.6 | 0.1 | 0.1 | +0.0 | 1.012 | 1,100.0 |
| GPU frame | 2,627.6 | 2,804.2 | 2,598.5 | 2,625.5 | +29.1 | 1.011 | 12,000.0 |
| Gate, rays off | 682.0 | 703.6 | 677.2 | 681.8 | +4.8 | 1.007 | 6,000.0 |
| Shadow-ray time | 1,620.7 | 1,800.5 | 1,597.9 | 1,625.5 | +22.8 | 1.014 | — |
| `culling_phase_one` | 28.8 | 30.5 | 24.2 | 30.1 | +4.6 | 1.189 | — |
| `culling_phase_two` | 20.7 | 21.4 | 20.2 | 20.8 | +0.5 | 1.023 | — |
| `depth_prepass` | 161.7 | 165.2 | 164.2 | 167.9 | -2.5 | 0.985 | — |
| `depth_pyramid_pass` | 213.5 | 218.8 | 213.4 | 215.8 | +0.1 | 1.001 | — |
| `dynamic_tlas_build` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | 1,000.0 |
| `froxel_pass` | 6.7 | 7.1 | 6.7 | 7.1 | +0.0 | 1.006 | — |
| `lit_pass` | 364.7 | 386.6 | 344.5 | 362.7 | +20.1 | 1.058 | — |
| `phase_two_depth_pass` | 12.2 | 12.5 | 12.0 | 12.5 | +0.1 | 1.010 | — |
| `shadow_mask_pass` | 1,478.6 | 1,573.0 | 1,478.1 | 1,486.6 | +0.5 | 1.000 | — |
| `tonemapping_pass` | 319.5 | 320.0 | 319.6 | 320.1 | -0.1 | 1.000 | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 4 | 4 | +0 | 1.000 | — |
| `instances_drawn` | 2 | 2 | +0 | 1.000 | — |
| `triangles_drawn` | 4 | 4 | +0 | 1.000 | — |
| `uploaded_bytes` | 416 | 416 | +0 | 1.000 | — |
| `gpu_memory_bytes` | 663,286,515 | 663,286,515 | +0 | 1.000 | 8,000,000,000 |
| `arena_bytes_in_use` | 176 | 176 | +0 | 1.000 | — |
| `heap_allocations` | 345 | 345 | +0 | 1.000 | — |
| `aliasing_rmse` | 1,557 | 1,557 | +0 | 1.000 | — |
| `aliasing_p99` | 3,033 | 3,033 | +0 | 1.000 | — |

The four rounds, each line's medians lowest to highest, then the median
of medians, then the p95s; every counter identical across the rounds:

| Line | Medians, lowest to highest | Median of medians | p95s |
|---|---:|---:|---:|
| CPU frame | 225.3–227.3–241.0–244.8 | 234.2 | 394.4–423.0–423.2–513.3 |
| Dynamic TLAS, CPU | 0.1–0.1–0.1–0.1 | 0.1 | 0.6–0.6–0.7–0.8 |
| GPU frame | 2,622.2–2,626.5–2,627.6–2,629.9 | 2,627.1 | 2,696.5–2,804.2–2,804.8–2,962.1 |
| Gate, rays off | 679.8–682.0–682.3–685.6 | 682.1 | 695.4–701.5–703.6–770.5 |
| Shadow-ray time | 1,617.6–1,619.5–1,619.5–1,620.7 | 1,619.5 | 1,694.7–1,798.2–1,800.5–1,926.4 |
| `culling_phase_one` | 28.0–28.4–28.5–28.8 | 28.5 | 30.3–30.3–30.4–30.5 |
| `culling_phase_two` | 20.6–20.6–20.7–20.8 | 20.7 | 21.4–21.4–21.5–21.5 |
| `depth_prepass` | 161.1–161.2–161.7–161.7 | 161.5 | 162.4–163.2–165.2–166.4 |
| `depth_pyramid_pass` | 213.5–213.9–214.5–214.9 | 214.2 | 218.7–218.8–220.9–221.1 |
| `froxel_pass` | 6.6–6.6–6.6–6.7 | 6.6 | 7.0–7.1–7.1–7.1 |
| `lit_pass` | 359.8–364.1–364.2–364.7 | 364.2 | 384.3–386.6–389.9–390.1 |
| `phase_two_depth_pass` | 12.1–12.2–12.2–12.2 | 12.2 | 12.5–12.6–12.6–12.7 |
| `shadow_mask_pass` | 1,478.1–1,478.6–1,479.2–1,480.1 | 1,478.9 | 1,499.8–1,503.2–1,573.0–1,745.3 |
| `tonemapping_pass` | 319.4–319.5–319.6–319.6 | 319.6 | 320.0–320.0–322.8–323.6 |

Each timed pass against its floor and, where one exists, its source (the
median of medians; p95s over the rounds):

| Pass | Median µs | p95 µs (rounds) | Floor µs and arithmetic | Ratio to floor | Source µs, scaled | Ratio to source | Verdict |
|---|---:|---:|---|---:|---|---:|---|
| `culling_phase_one` | 28.5 | 30–31 | 27: 0.0 by bytes (0.0 MB: 2 Instances × 128 B scanned, 12 × 4 B of packed indices written, 4 × 3 B of Meshlet triangles, the Instances re-read, 1 records × 12 B) + 27.4, the four dispatches' fixed cost as uv-grid's line reads it | 1.04 | 670 (Haar and Aaltonen) counts 250,000 objects | does not apply | within 2x of its floor |
| `culling_phase_two` | 20.7 | 21–22 | 20: 0.0 by bytes (the same scan and the records) + 20.4, uv-grid's line | 1.01 | counted with phase one | — | within 2x of its floor |
| `depth_prepass` | 161.5 | 162–166 | 216: 0.0 MB (every vertex pulled once) to 0.0 MB (12 indices × 36 B) of vertex pull, the Instances and records, and 59.0 MB of depth written | 0.75 to 0.75 | none cited | — | under its uncompressed floor: the target is read compressed (note) |
| `depth_pyramid_pass` | 214.2 | 219–221 | 234: 63.9 MB: 59.0 MB of samples read, level 0 and the levels after written | 0.92 | 107 (Haar and Aaltonen, from HTILE); 73 (Wihlidal, 16-bit tiles) | 2.01 / 2.94 | under its uncompressed floor: the target is read compressed (note); over 2x a source (note) |
| `froxel_pass` | 6.6 | 7–7 | 8: 1.4 by bytes (369 KB: 18,432 Froxels × a count and 4 lights × 4 B) + 6.9, uv-grid's line | 0.81 | 320 to 900 (Drobot) counts 256 lights | does not apply | at its floor, the dispatches' fixed cost |
| `lit_pass` | 364.2 | 384–390 | 812: the vertex pull, 59.0 MB of depth read at Equal, 118.0 MB of HDR written, 44.2 MB of mask words read and the Froxel lists: 221.6 MB to 221.6 MB | 0.45 to 0.45 | 8,180 to 10,130 (Drobot, forward+) counts 256 lights; 4 here | does not apply | under its uncompressed floor: the target is read compressed (note) |
| `phase_two_depth_pass` | 12.2 | 13–13 | not floored: phase two's triangle count is not in the counters | — | none cited | — | not floored; under 0.1 ms |
| `shadow_mask_pass` | 1,478.9 | 1,500–1,745 | 1,322 to 3,212: 1,150, the held ray code's measured fixed cost, + 2,147,448 lit pixels × (the sun + 0.00 point lights reaching) = 2,147,448 rays × 0.08 to 0.96 ns (172 to 2,062) | 0.46 to 1.12 | 2,310 to 2,840 (Boksansky et al., one light, the sun alone reaching, scaled by pixels) | 0.52 to 0.64 | within 2x of its floor even at uv-grid's rate |
| `tonemapping_pass` | 319.6 | 320–324 | 486: 132.7 MB: 118.0 MB of HDR samples read, 14.7 MB written | 0.66 | none with a stated resolution | — | under its uncompressed floor: the target is read compressed (note) |

### orrery

Round 1 as `praxis report` prints it; the rounds table and the floors
follow.

**orrery** on Apple M4 Pro, release, 2560x1440, 1800 measured frames

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 386.0 | 566.4 | 358.8 | 386.6 | +27.2 | 1.076 | 6,000.0 |
| Dynamic TLAS, CPU | 184.4 | 252.3 | 180.8 | 199.8 | +3.7 | 1.020 | 1,100.0 |
| GPU frame | 2,934.9 | 3,055.8 | 2,917.3 | 3,035.5 | +17.6 | 1.006 | 12,000.0 |
| Gate, rays off | 647.8 | 673.0 | 642.6 | 656.7 | +5.2 | 1.008 | 6,000.0 |
| Shadow-ray time | 1,957.2 | 2,076.8 | 1,952.4 | 2,060.9 | +4.9 | 1.002 | — |
| `culling_phase_one` | 33.5 | 34.8 | 34.2 | 35.1 | -0.7 | 0.979 | — |
| `culling_phase_two` | 20.5 | 22.6 | 20.1 | 21.1 | +0.4 | 1.021 | — |
| `depth_prepass` | 160.0 | 163.2 | 159.9 | 161.0 | +0.1 | 1.001 | — |
| `depth_pyramid_pass` | 203.1 | 204.8 | 203.2 | 206.0 | -0.1 | 1.000 | — |
| `dynamic_tlas_build` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | 1,000.0 |
| `froxel_pass` | 6.8 | 7.2 | 6.9 | 7.4 | -0.1 | 0.988 | — |
| `lit_pass` | 342.8 | 356.5 | 335.0 | 347.8 | +7.7 | 1.023 | — |
| `phase_two_depth_pass` | 11.9 | 12.5 | 12.0 | 12.6 | -0.1 | 0.993 | — |
| `shadow_mask_pass` | 1,824.2 | 1,934.8 | 1,815.7 | 1,924.7 | +8.5 | 1.005 | — |
| `tonemapping_pass` | 319.7 | 320.2 | 322.5 | 323.0 | -2.8 | 0.991 | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 4 | 4 | +0 | 1.000 | — |
| `instances_drawn` | 5 | 5 | +0 | 1.000 | — |
| `triangles_drawn` | 11,040 | 11,040 | +0 | 1.000 | — |
| `uploaded_bytes` | 1,416 | 1,416 | +0 | 1.000 | — |
| `gpu_memory_bytes` | 668,982,392 | 668,982,392 | +0 | 1.000 | 8,000,000,000 |
| `arena_bytes_in_use` | 76,680 | 76,680 | +0 | 1.000 | — |
| `heap_allocations` | 455 | 455 | +0 | 1.000 | — |
| `aliasing_rmse` | 2,817 | 2,817 | +0 | 1.000 | — |
| `aliasing_p99` | 4,686 | 4,686 | +0 | 1.000 | — |

The four rounds, each line's medians lowest to highest, then the median
of medians, then the p95s; every counter identical across the rounds:

| Line | Medians, lowest to highest | Median of medians | p95s |
|---|---:|---:|---:|
| CPU frame | 386.0–413.1–418.3–443.4 | 415.7 | 566.4–664.6–694.4–760.0 |
| Dynamic TLAS, CPU | 184.4–198.3–199.1–205.8 | 198.7 | 252.3–300.7–314.2–335.1 |
| GPU frame | 2,934.9–2,943.1–2,948.2–2,963.5 | 2,945.7 | 3,055.8–3,096.5–3,292.9–3,442.7 |
| Gate, rays off | 647.6–647.8–648.9–649.2 | 648.4 | 673.0–673.2–699.8–702.2 |
| Shadow-ray time | 1,957.2–1,963.5–1,966.2–1,979.1 | 1,964.8 | 2,076.8–2,115.2–2,314.9–2,458.3 |
| `culling_phase_one` | 33.1–33.1–33.3–33.5 | 33.2 | 34.7–34.8–34.9–35.2 |
| `culling_phase_two` | 20.0–20.2–20.5–20.5 | 20.3 | 21.7–22.6–22.6–23.7 |
| `depth_prepass` | 160.0–160.0–160.2–160.2 | 160.1 | 161.0–163.0–163.2–163.4 |
| `depth_pyramid_pass` | 203.1–203.4–203.4–203.8 | 203.4 | 204.8–205.9–206.1–209.8 |
| `froxel_pass` | 6.8–6.8–6.8–6.8 | 6.8 | 7.2–7.2–7.3–7.3 |
| `lit_pass` | 342.0–342.2–342.8–345.0 | 342.5 | 356.5–360.9–366.9–370.0 |
| `phase_two_depth_pass` | 11.9–11.9–11.9–12.0 | 11.9 | 12.5–12.5–12.5–12.7 |
| `shadow_mask_pass` | 1,824.2–1,827.3–1,830.4–1,839.7 | 1,828.9 | 1,934.8–1,944.9–2,069.5–2,235.1 |
| `tonemapping_pass` | 319.6–319.7–319.7–319.7 | 319.7 | 320.2–322.9–322.9–323.1 |

Each timed pass against its floor and, where one exists, its source (the
median of medians; p95s over the rounds):

| Pass | Median µs | p95 µs (rounds) | Floor µs and arithmetic | Ratio to floor | Source µs, scaled | Ratio to source | Verdict |
|---|---:|---:|---|---:|---|---:|---|
| `culling_phase_one` | 33.2 | 35–35 | 28: 0.6 by bytes (0.2 MB: 5 Instances × 128 B scanned, 33,120 × 4 B of packed indices written, 11,040 × 3 B of Meshlet triangles, the Instances re-read, 90 records × 12 B) + 27.4, the four dispatches' fixed cost as uv-grid's line reads it | 1.19 | 670 (Haar and Aaltonen) counts 250,000 objects | does not apply | within 2x of its floor |
| `culling_phase_two` | 20.3 | 22–24 | 20: 0.0 by bytes (the same scan and the records) + 20.4, uv-grid's line | 1.00 | counted with phase one | — | at its floor, the dispatches' fixed cost |
| `depth_prepass` | 160.1 | 161–163 | 217 to 220: 0.3 MB (every vertex pulled once) to 1.2 MB (33,120 indices × 36 B) of vertex pull, the Instances and records, and 59.0 MB of depth written | 0.73 to 0.74 | none cited | — | under its uncompressed floor: the target is read compressed (note) |
| `depth_pyramid_pass` | 203.4 | 205–210 | 234: 63.9 MB: 59.0 MB of samples read, level 0 and the levels after written | 0.87 | 107 (Haar and Aaltonen, from HTILE); 73 (Wihlidal, 16-bit tiles) | 1.91 / 2.79 | under its uncompressed floor: the target is read compressed (note); over 2x a source (note) |
| `froxel_pass` | 6.8 | 7–7 | 8: 1.4 by bytes (369 KB: 18,432 Froxels × a count and 4 lights × 4 B) + 6.9, uv-grid's line | 0.82 | 320 to 900 (Drobot) counts 256 lights | does not apply | at its floor, the dispatches' fixed cost |
| `lit_pass` | 342.5 | 356–370 | 813 to 816: the vertex pull, 59.0 MB of depth read at Equal, 118.0 MB of HDR written, 44.2 MB of mask words read and the Froxel lists: 221.9 MB to 222.7 MB | 0.42 to 0.42 | 8,180 to 10,130 (Drobot, forward+) counts 256 lights; 4 here | does not apply | under its uncompressed floor: the target is read compressed (note) |
| `phase_two_depth_pass` | 11.9 | 12–13 | not floored: phase two's triangle count is not in the counters | — | none cited | — | not floored; under 0.1 ms |
| `shadow_mask_pass` | 1,828.9 | 1,935–2,235 | 1,205 to 1,812: 1,150, the held ray code's measured fixed cost, + 137,997 lit pixels × (the sun + 4.00 point lights reaching) = 689,985 rays × 0.08 to 0.96 ns (55 to 662) | 1.01 to 1.52 | 6,040 to 8,530 (Boksansky et al., four lights; five reach here, scaled by pixels) | 0.21 to 0.30 | within 2x of its floor even at uv-grid's rate |
| `tonemapping_pass` | 319.7 | 320–323 | 486: 132.7 MB: 118.0 MB of HDR samples read, 14.7 MB written | 0.66 | none with a stated resolution | — | under its uncompressed floor: the target is read compressed (note) |
| `dynamic_tlas_build` | 0.0 | 0–0 | runs beside the previous frame and reads 0 at the median | — | none cited; its Budget is 1,000 | — | within its Budget |

### minification

Round 1 as `praxis report` prints it; the rounds table and the floors
follow.

**minification** on Apple M4 Pro, release, 2560x1440, 1800 measured frames

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 246.4 | 425.4 | 192.7 | 225.8 | +53.8 | 1.279 | 6,000.0 |
| Dynamic TLAS, CPU | 0.1 | 0.8 | 0.1 | 0.1 | +0.0 | 1.012 | 1,100.0 |
| GPU frame | 3,006.8 | 3,057.0 | 3,003.5 | 3,014.9 | +3.4 | 1.001 | 12,000.0 |
| Gate, rays off | 843.1 | 862.6 | 835.2 | 842.4 | +7.8 | 1.009 | 6,000.0 |
| Shadow-ray time | 1,840.4 | 1,887.5 | 1,844.0 | 1,857.6 | -3.5 | 0.998 | — |
| `culling_phase_one` | 27.6 | 29.1 | 28.5 | 29.7 | -0.9 | 0.969 | — |
| `culling_phase_two` | 19.5 | 20.2 | 19.4 | 19.9 | +0.1 | 1.006 | — |
| `depth_prepass` | 159.8 | 162.7 | 159.5 | 162.7 | +0.3 | 1.002 | — |
| `depth_pyramid_pass` | 215.6 | 220.0 | 214.1 | 217.6 | +1.5 | 1.007 | — |
| `dynamic_tlas_build` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | 1,000.0 |
| `froxel_pass` | 5.8 | 6.2 | 5.9 | 6.3 | -0.1 | 0.986 | — |
| `lit_pass` | 531.8 | 542.6 | 528.5 | 534.4 | +3.3 | 1.006 | — |
| `phase_two_depth_pass` | 12.1 | 12.5 | 12.0 | 12.4 | +0.0 | 1.003 | — |
| `shadow_mask_pass` | 1,693.6 | 1,741.0 | 1,694.2 | 1,701.4 | -0.5 | 1.000 | — |
| `tonemapping_pass` | 319.5 | 320.0 | 322.2 | 322.8 | -2.7 | 0.991 | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 4 | 4 | +0 | 1.000 | — |
| `instances_drawn` | 1 | 1 | +0 | 1.000 | — |
| `triangles_drawn` | 2 | 2 | +0 | 1.000 | — |
| `uploaded_bytes` | 272 | 272 | +0 | 1.000 | — |
| `gpu_memory_bytes` | 663,547,203 | 663,547,203 | +0 | 1.000 | 8,000,000,000 |
| `arena_bytes_in_use` | 176 | 176 | +0 | 1.000 | — |
| `heap_allocations` | 345 | 345 | +0 | 1.000 | — |
| `aliasing_rmse` | 32,960 | 32,960 | +0 | 1.000 | — |
| `aliasing_p99` | 143,799 | 143,799 | +0 | 1.000 | — |

The four rounds, each line's medians lowest to highest, then the median
of medians, then the p95s; every counter identical across the rounds:

| Line | Medians, lowest to highest | Median of medians | p95s |
|---|---:|---:|---:|
| CPU frame | 230.7–232.3–233.4–246.4 | 232.8 | 419.2–422.5–425.4–459.3 |
| Dynamic TLAS, CPU | 0.1–0.1–0.1–0.1 | 0.1 | 0.6–0.6–0.8–0.8 |
| GPU frame | 3,005.5–3,006.8–3,007.0–3,009.2 | 3,006.9 | 3,057.0–3,182.0–3,320.5–3,632.4 |
| Gate, rays off | 839.1–840.4–843.1–846.7 | 841.8 | 858.2–862.6–867.5–875.0 |
| Shadow-ray time | 1,838.3–1,840.4–1,843.0–1,844.2 | 1,841.7 | 1,887.5–2,009.5–2,156.0–2,459.8 |
| `culling_phase_one` | 27.2–27.2–27.5–27.6 | 27.4 | 29.0–29.1–29.2–29.5 |
| `culling_phase_two` | 19.3–19.5–19.5–19.5 | 19.5 | 20.1–20.2–20.2–20.2 |
| `depth_prepass` | 159.5–159.7–159.8–160.3 | 159.7 | 162.2–162.7–162.7–165.9 |
| `depth_pyramid_pass` | 215.3–215.5–215.6–216.3 | 215.6 | 220.0–221.2–221.4–227.1 |
| `froxel_pass` | 5.8–5.8–5.8–5.8 | 5.8 | 6.2–6.2–6.2–6.3 |
| `lit_pass` | 531.8–532.8–533.4–533.6 | 533.1 | 542.6–544.3–544.4–612.4 |
| `phase_two_depth_pass` | 12.1–12.1–12.1–12.2 | 12.1 | 12.5–12.5–12.5–12.6 |
| `shadow_mask_pass` | 1,692.5–1,692.5–1,693.3–1,693.6 | 1,692.9 | 1,712.8–1,741.0–1,965.5–2,090.5 |
| `tonemapping_pass` | 319.3–319.4–319.5–319.5 | 319.4 | 319.8–320.0–320.3–321.2 |

Each timed pass against its floor and, where one exists, its source (the
median of medians; p95s over the rounds):

| Pass | Median µs | p95 µs (rounds) | Floor µs and arithmetic | Ratio to floor | Source µs, scaled | Ratio to source | Verdict |
|---|---:|---:|---|---:|---|---:|---|
| `culling_phase_one` | 27.4 | 29–29 | 27: 0.0 by bytes (0.0 MB: 1 Instances × 128 B scanned, 6 × 4 B of packed indices written, 2 × 3 B of Meshlet triangles, the Instances re-read, 1 records × 12 B) + 27.4, the four dispatches' fixed cost as uv-grid's line reads it | 1.00 | 670 (Haar and Aaltonen) counts 250,000 objects | does not apply | at its floor, the dispatches' fixed cost |
| `culling_phase_two` | 19.5 | 20–20 | 20: 0.0 by bytes (the same scan and the records) + 20.4, uv-grid's line | 0.96 | counted with phase one | — | at its floor, the dispatches' fixed cost |
| `depth_prepass` | 159.7 | 162–166 | 216: 0.0 MB (every vertex pulled once) to 0.0 MB (6 indices × 36 B) of vertex pull, the Instances and records, and 59.0 MB of depth written | 0.74 to 0.74 | none cited | — | under its uncompressed floor: the target is read compressed (note) |
| `depth_pyramid_pass` | 215.6 | 220–227 | 234: 63.9 MB: 59.0 MB of samples read, level 0 and the levels after written | 0.92 | 107 (Haar and Aaltonen, from HTILE); 73 (Wihlidal, 16-bit tiles) | 2.02 / 2.96 | under its uncompressed floor: the target is read compressed (note); over 2x a source (note) |
| `froxel_pass` | 5.8 | 6–6 | 7: 0.5 by bytes (147 KB: 18,432 Froxels × a count and 1 lights × 4 B) + 6.9, uv-grid's line | 0.79 | 320 to 900 (Drobot) counts 256 lights | does not apply | at its floor, the dispatches' fixed cost |
| `lit_pass` | 533.1 | 543–612 | 811: the vertex pull, 59.0 MB of depth read at Equal, 118.0 MB of HDR written, 44.2 MB of mask words read and the Froxel lists: 221.3 MB to 221.3 MB | 0.66 to 0.66 | 8,180 to 10,130 (Drobot, forward+) counts 256 lights; 1 here | does not apply | under its uncompressed floor: the target is read compressed (note) |
| `phase_two_depth_pass` | 12.1 | 12–13 | not floored: phase two's triangle count is not in the counters | — | none cited | — | not floored; under 0.1 ms |
| `shadow_mask_pass` | 1,692.9 | 1,713–2,091 | 1,542 to 5,853: 1,150, the held ray code's measured fixed cost, + 2,449,361 lit pixels × (the sun + 1.00 point lights reaching) = 4,898,722 rays × 0.08 to 0.96 ns (392 to 4,703) | 0.29 to 1.10 | 2,310 to 8,530 (Boksansky et al., one to four lights; two reach here, scaled by pixels) | 0.20 to 0.73 | within 2x of its floor even at uv-grid's rate |
| `tonemapping_pass` | 319.4 | 320–321 | 486: 132.7 MB: 118.0 MB of HDR samples read, 14.7 MB written | 0.66 | none with a stated resolution | — | under its uncompressed floor: the target is read compressed (note) |

### shadows

Round 1 as `praxis report` prints it; the rounds table and the floors
follow.

**shadows** on Apple M4 Pro, release, 2560x1440, 1800 measured frames

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 261.0 | 468.2 | 265.9 | 316.7 | -4.9 | 0.982 | 6,000.0 |
| Dynamic TLAS, CPU | 0.1 | 0.7 | 0.1 | 0.1 | +0.0 | 1.012 | 1,100.0 |
| GPU frame | 4,120.8 | 4,171.3 | 4,094.7 | 4,123.7 | +26.0 | 1.006 | 12,000.0 |
| Gate, rays off | 847.8 | 858.0 | 849.5 | 854.2 | -1.8 | 0.998 | 6,000.0 |
| Shadow-ray time | 2,949.5 | 3,000.0 | 2,922.7 | 2,953.0 | +26.7 | 1.009 | — |
| `culling_phase_one` | 33.4 | 35.1 | 28.6 | 33.7 | +4.8 | 1.166 | — |
| `culling_phase_two` | 19.4 | 20.1 | 19.8 | 20.4 | -0.4 | 0.981 | — |
| `depth_prepass` | 167.6 | 169.1 | 171.7 | 175.0 | -4.1 | 0.976 | — |
| `depth_pyramid_pass` | 213.0 | 218.4 | 213.0 | 215.7 | +0.0 | 1.000 | — |
| `dynamic_tlas_build` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | 1,000.0 |
| `froxel_pass` | 5.8 | 6.2 | 5.8 | 6.2 | +0.0 | 0.993 | — |
| `lit_pass` | 515.5 | 519.8 | 496.8 | 514.8 | +18.6 | 1.037 | — |
| `phase_two_depth_pass` | 12.1 | 12.5 | 11.9 | 12.4 | +0.2 | 1.017 | — |
| `shadow_mask_pass` | 2,812.2 | 2,861.3 | 2,813.4 | 2,820.9 | -1.2 | 1.000 | — |
| `tonemapping_pass` | 319.8 | 322.6 | 319.5 | 320.0 | +0.2 | 1.001 | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 4 | 4 | +0 | 1.000 | — |
| `instances_drawn` | 1 | 1 | +0 | 1.000 | — |
| `triangles_drawn` | 2,210 | 2,210 | +0 | 1.000 | — |
| `uploaded_bytes` | 272 | 272 | +0 | 1.000 | — |
| `gpu_memory_bytes` | 663,385,443 | 663,385,443 | +0 | 1.000 | 8,000,000,000 |
| `arena_bytes_in_use` | 76,856 | 76,856 | +0 | 1.000 | — |
| `heap_allocations` | 345 | 345 | +0 | 1.000 | — |
| `aliasing_rmse` | 3,182 | 3,182 | +0 | 1.000 | — |
| `aliasing_p99` | 6,670 | 6,670 | +0 | 1.000 | — |

The four rounds, each line's medians lowest to highest, then the median
of medians, then the p95s; every counter identical across the rounds:

| Line | Medians, lowest to highest | Median of medians | p95s |
|---|---:|---:|---:|
| CPU frame | 240.4–261.0–266.7–322.8 | 263.9 | 456.9–468.2–478.1–567.4 |
| Dynamic TLAS, CPU | 0.1–0.1–0.1–0.1 | 0.1 | 0.7–0.7–0.7–0.9 |
| GPU frame | 4,118.8–4,120.8–4,121.5–4,134.2 | 4,121.1 | 4,171.3–4,303.6–4,462.2–4,778.8 |
| Gate, rays off | 845.2–847.8–847.9–853.7 | 847.8 | 855.2–858.0–909.8–949.2 |
| Shadow-ray time | 2,949.5–2,949.5–2,950.3–2,955.9 | 2,949.9 | 3,000.0–3,133.9–3,288.2–3,582.8 |
| `culling_phase_one` | 33.3–33.4–33.4–33.6 | 33.4 | 35.0–35.1–35.5–36.0 |
| `culling_phase_two` | 19.4–19.4–19.4–19.6 | 19.4 | 20.1–20.1–20.3–20.5 |
| `depth_prepass` | 167.6–167.7–167.7–167.8 | 167.7 | 169.1–170.8–170.9–172.3 |
| `depth_pyramid_pass` | 213.0–213.2–213.5–215.3 | 213.3 | 218.0–218.4–220.6–230.3 |
| `froxel_pass` | 5.8–5.8–5.8–5.8 | 5.8 | 6.2–6.2–6.2–6.2 |
| `lit_pass` | 515.0–515.5–515.8–516.3 | 515.6 | 518.7–519.8–519.8–605.8 |
| `phase_two_depth_pass` | 12.1–12.1–12.2–12.2 | 12.1 | 12.5–12.6–12.6–12.6 |
| `shadow_mask_pass` | 2,810.9–2,811.6–2,812.2–2,815.8 | 2,811.9 | 2,861.3–2,993.1–3,143.5–3,264.8 |
| `tonemapping_pass` | 319.4–319.4–319.5–319.8 | 319.5 | 319.8–320.1–322.6–394.2 |

Each timed pass against its floor and, where one exists, its source (the
median of medians; p95s over the rounds):

| Pass | Median µs | p95 µs (rounds) | Floor µs and arithmetic | Ratio to floor | Source µs, scaled | Ratio to source | Verdict |
|---|---:|---:|---|---:|---|---:|---|
| `culling_phase_one` | 33.4 | 35–36 | 28: 0.1 by bytes (0.0 MB: 1 Instances × 128 B scanned, 6,630 × 4 B of packed indices written, 2,210 × 3 B of Meshlet triangles, the Instances re-read, 18 records × 12 B) + 27.4, the four dispatches' fixed cost as uv-grid's line reads it | 1.21 | 670 (Haar and Aaltonen) counts 250,000 objects | does not apply | within 2x of its floor |
| `culling_phase_two` | 19.4 | 20–20 | 20: 0.0 by bytes (the same scan and the records) + 20.4, uv-grid's line | 0.95 | counted with phase one | — | at its floor, the dispatches' fixed cost |
| `depth_prepass` | 167.7 | 169–172 | 216 to 217: 0.1 MB (every vertex pulled once) to 0.2 MB (6,630 indices × 36 B) of vertex pull, the Instances and records, and 59.0 MB of depth written | 0.77 to 0.78 | none cited | — | under its uncompressed floor: the target is read compressed (note) |
| `depth_pyramid_pass` | 213.3 | 218–230 | 234: 63.9 MB: 59.0 MB of samples read, level 0 and the levels after written | 0.91 | 107 (Haar and Aaltonen, from HTILE); 73 (Wihlidal, 16-bit tiles) | 2.00 / 2.93 | under its uncompressed floor: the target is read compressed (note); over 2x a source (note) |
| `froxel_pass` | 5.8 | 6–6 | 7: 0.5 by bytes (147 KB: 18,432 Froxels × a count and 1 lights × 4 B) + 6.9, uv-grid's line | 0.78 | 320 to 900 (Drobot) counts 256 lights | does not apply | at its floor, the dispatches' fixed cost |
| `lit_pass` | 515.6 | 519–606 | 811 to 812: the vertex pull, 59.0 MB of depth read at Equal, 118.0 MB of HDR written, 44.2 MB of mask words read and the Froxel lists: 221.4 MB to 221.6 MB | 0.64 to 0.64 | 8,180 to 10,130 (Drobot, forward+) counts 256 lights; 1 here | does not apply | under its uncompressed floor: the target is read compressed (note) |
| `phase_two_depth_pass` | 12.1 | 13–13 | not floored: phase two's triangle count is not in the counters | — | none cited | — | not floored; under 0.1 ms |
| `shadow_mask_pass` | 2,811.9 | 2,861–3,265 | 1,670 to 7,387: 1,150, the held ray code's measured fixed cost, + 3,248,640 lit pixels × (the sun + 1.00 point lights reaching) = 6,497,280 rays × 0.08 to 0.96 ns (520 to 6,237) | 0.38 to 1.68 | 2,310 to 8,530 (Boksansky et al., one to four lights; two reach here, scaled by pixels) | 0.33 to 1.22 | within 2x of its floor even at uv-grid's rate |
| `tonemapping_pass` | 319.5 | 320–394 | 486: 132.7 MB: 118.0 MB of HDR samples read, 14.7 MB written | 0.66 | none with a stated resolution | — | under its uncompressed floor: the target is read compressed (note) |

### crowd

Round 1 as `praxis report` prints it; the rounds table and the floors
follow.

**crowd** on Apple M4 Pro, release, 2560x1440, 1800 measured frames

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 816.6 | 1,056.3 | 744.5 | 854.9 | +72.1 | 1.097 | 6,000.0 |
| Dynamic TLAS, CPU | 384.3 | 506.7 | 343.7 | 388.3 | +40.6 | 1.118 | 1,100.0 |
| GPU frame | 3,756.4 | 3,910.4 | 3,747.5 | 3,785.8 | +8.8 | 1.002 | 12,000.0 |
| Gate, rays off | 1,268.0 | 1,334.8 | 1,297.0 | 1,340.2 | -28.9 | 0.978 | 6,000.0 |
| Shadow-ray time | 2,157.3 | 2,318.5 | 2,128.6 | 2,184.3 | +28.7 | 1.013 | — |
| `bend_pass` | 27.8 | 30.4 | 27.8 | 30.7 | +0.0 | 0.999 | — |
| `culling_phase_one` | 92.9 | 96.4 | 93.0 | 95.5 | -0.1 | 0.999 | — |
| `culling_phase_two` | 21.4 | 22.2 | 21.2 | 22.2 | +0.2 | 1.008 | — |
| `depth_prepass` | 378.2 | 391.3 | 384.0 | 394.2 | -5.8 | 0.985 | — |
| `depth_pyramid_pass` | 214.8 | 220.4 | 213.7 | 217.1 | +1.1 | 1.005 | — |
| `dynamic_tlas_build` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | 1,000.0 |
| `froxel_pass` | 9.4 | 12.8 | 10.2 | 12.6 | -0.8 | 0.922 | — |
| `lit_pass` | 717.4 | 736.5 | 712.5 | 726.9 | +5.0 | 1.007 | — |
| `phase_two_depth_pass` | 12.4 | 12.7 | 12.3 | 12.7 | +0.0 | 1.003 | — |
| `shadow_mask_pass` | 1,954.3 | 2,017.5 | 1,950.3 | 1,973.6 | +4.0 | 1.002 | — |
| `tonemapping_pass` | 319.9 | 320.6 | 319.7 | 320.1 | +0.3 | 1.001 | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 4 | 4 | +0 | 1.000 | — |
| `instances_drawn` | 101 | 101 | +0 | 1.000 | — |
| `triangles_drawn` | 512,002 | 512,002 | +0 | 1.000 | — |
| `uploaded_bytes` | 322,224 | 322,224 | +0 | 1.000 | — |
| `gpu_memory_bytes` | 664,077,519 | 664,077,519 | +0 | 1.000 | 8,000,000,000 |
| `arena_bytes_in_use` | 11,325,896 | 11,325,896 | +0 | 1.000 | — |
| `heap_allocations` | 2,099 | 2,099 | +0 | 1.000 | — |
| `aliasing_rmse` | 4,161 | 4,161 | +0 | 1.000 | — |
| `aliasing_p99` | 17,296 | 17,296 | +0 | 1.000 | — |

The four rounds, each line's medians lowest to highest, then the median
of medians, then the p95s; every counter identical across the rounds:

| Line | Medians, lowest to highest | Median of medians | p95s |
|---|---:|---:|---:|
| CPU frame | 739.0–791.8–793.9–816.6 | 792.8 | 982.2–1,056.3–1,060.6–1,064.7 |
| Dynamic TLAS, CPU | 351.9–381.0–381.2–384.3 | 381.1 | 474.5–506.7–512.1–519.1 |
| GPU frame | 3,749.8–3,756.4–3,759.4–3,765.0 | 3,757.9 | 3,910.4–3,924.8–3,967.5–4,097.7 |
| Gate, rays off | 1,267.2–1,268.0–1,277.3–1,279.6 | 1,272.7 | 1,334.8–1,345.0–1,348.9–1,455.4 |
| Shadow-ray time | 2,151.4–2,152.8–2,156.3–2,157.3 | 2,154.5 | 2,318.5–2,326.9–2,360.9–2,478.2 |
| `bend_pass` | 27.8–27.8–27.9–28.8 | 27.9 | 30.4–30.5–30.6–30.9 |
| `culling_phase_one` | 92.3–92.7–92.9–92.9 | 92.8 | 95.2–96.0–96.2–96.4 |
| `culling_phase_two` | 21.3–21.3–21.4–21.6 | 21.3 | 22.1–22.2–22.2–22.5 |
| `depth_prepass` | 378.2–379.1–380.1–381.9 | 379.6 | 390.7–391.3–392.9–393.8 |
| `depth_pyramid_pass` | 214.5–214.7–214.8–215.2 | 214.8 | 219.2–219.2–220.4–220.6 |
| `froxel_pass` | 8.9–9.4–9.4–9.4 | 9.4 | 12.6–12.7–12.8–12.8 |
| `lit_pass` | 714.8–717.4–717.4–720.9 | 717.4 | 732.8–736.5–743.5–745.1 |
| `phase_two_depth_pass` | 12.3–12.3–12.4–12.4 | 12.4 | 12.7–12.7–12.7–12.8 |
| `shadow_mask_pass` | 1,951.6–1,954.3–1,954.6–1,954.8 | 1,954.5 | 2,017.5–2,101.6–2,140.5–2,206.8 |
| `tonemapping_pass` | 319.7–319.8–319.9–319.9 | 319.8 | 320.1–320.4–320.6–321.3 |

Each timed pass against its floor and, where one exists, its source (the
median of medians; p95s over the rounds):

| Pass | Median µs | p95 µs (rounds) | Floor µs and arithmetic | Ratio to floor | Source µs, scaled | Ratio to source | Verdict |
|---|---:|---:|---|---:|---|---:|---|
| `culling_phase_one` | 92.8 | 95–96 | 56: 28.4 by bytes (7.8 MB: 101 Instances × 128 B scanned, 1,536,006 × 4 B of packed indices written, 512,002 × 3 B of Meshlet triangles, the Instances re-read, 4,130 records × 12 B) + 27.4, the four dispatches' fixed cost as uv-grid's line reads it | 1.66 | 670 (Haar and Aaltonen) counts 250,000 objects | does not apply | within 2x of its floor |
| `culling_phase_two` | 21.3 | 22–22 | 21: 0.2 by bytes (the same scan and the records) + 20.4, uv-grid's line | 1.04 | counted with phase one | — | within 2x of its floor |
| `depth_prepass` | 379.6 | 391–394 | 270 to 419: 14.6 MB (every vertex pulled once) to 55.3 MB (1,536,006 indices × 36 B) of vertex pull, the Instances and records, and 59.0 MB of depth written | 0.91 to 1.41 | none cited | — | within 2x of its floor |
| `depth_pyramid_pass` | 214.8 | 219–221 | 234: 63.9 MB: 59.0 MB of samples read, level 0 and the levels after written | 0.92 | 107 (Haar and Aaltonen, from HTILE); 73 (Wihlidal, 16-bit tiles) | 2.01 / 2.95 | under its uncompressed floor: the target is read compressed (note); over 2x a source (note) |
| `froxel_pass` | 9.4 | 13–13 | 7: 0.3 by bytes (74 KB: 18,432 Froxels × a count and 0 lights × 4 B) + 6.9, uv-grid's line | 1.32 | 320 to 900 (Drobot) counts 256 lights | does not apply | within 2x of its floor |
| `lit_pass` | 717.4 | 733–745 | 864 to 1,013: the vertex pull, 59.0 MB of depth read at Equal, 118.0 MB of HDR written, 44.2 MB of mask words read and the Froxel lists: 235.9 MB to 276.6 MB | 0.71 to 0.83 | 8,180 to 10,130 (Drobot, forward+) counts 256 lights; 0 here | does not apply | under its uncompressed floor: the target is read compressed (note) |
| `phase_two_depth_pass` | 12.4 | 13–13 | not floored: phase two's triangle count is not in the counters | — | none cited | — | not floored; under 0.1 ms |
| `shadow_mask_pass` | 1,954.5 | 2,017–2,207 | 1,312 to 3,088: 1,150, the held ray code's measured fixed cost, + 2,018,982 lit pixels × (the sun + 0.00 point lights reaching) = 2,018,982 rays × 0.08 to 0.96 ns (162 to 1,938) | 0.63 to 1.49 | 2,310 to 2,840 (Boksansky et al., one light, the sun alone reaching, scaled by pixels) | 0.69 to 0.85 | within 2x of its floor even at uv-grid's rate |
| `tonemapping_pass` | 319.8 | 320–321 | 486: 132.7 MB: 118.0 MB of HDR samples read, 14.7 MB written | 0.66 | none with a stated resolution | — | under its uncompressed floor: the target is read compressed (note) |
| `bend_pass` | 27.9 | 30–31 | 56: 15.2 MB: 270,600 vertices × 28 B read and written | 0.50 | none cited | — | under its floor: the working set need not reach DRAM (note) |
| `dynamic_tlas_build` | 0.0 | 0–0 | runs beside the previous frame and reads 0 at the median | — | none cited; its Budget is 1,000 | — | within its Budget |

### Findings

| id | claim | severity | where | evidence | expected | verifier | placement |
|---|---|---|---|---|---|---|---|
| L3-L01 | The light scenes' CPU frame lines fail `compare` against their Baselines in every round on 5 scenes, 1.10 to 1.65 times them, while every GPU line is within 1.6% and the lines are at most 13% of the 6 ms Budget; whether the machine or the code moved them is not settled | `health`, 5, 8, 6 and 6 CPU lines failed in rounds 1 to 4 | `src/bench/mod.rs:135-139` (`FAIL_RATIO` 1.10, `CPU_NOISE_FLOOR_MICROSECONDS` 10); `bench/baselines/apple-m4-pro/*.json`, which carry no Provenance, so not the load they were recorded under | the table above; one-minute load averages 3.7 to 7.3 at the rounds' starts with two of this ticket's sub-readers and peer sessions reading files; L3-D04 saw `town`'s line 20 to 22% over its Baseline under Spotlight's load | the CPU lines within 5% of their Baselines in a re-run at a one-minute load under 2, or Baselines that carry the load they were recorded under so `compare` can tell the two apart | `narrowed`: The light scenes' CPU frame lines fail `compare` against their Baselines in every round under Spotlight load: on 5 scenes at 1.10 to 1.65 times them in the readers' rounds (one-minute load 3.7 to 7.3), and on 8 at 1.19 to 2.25 times in the verifier's (12.7 to 19.3). Every GPU frame is within 2.0% of its Baseline, and the lines are at most 17% of the 6 ms Budget. The rise with load points at the machine, but no quiet re-run has settled whether the machine or the code moved them. Under heavier load the CPU frame fails `compare` on 8 light scenes in every round, not 5, at 1.19 to 2.25 times the Baseline. The GPU frames are within 2.0% and the lines reach 17% of the Budget, and the lines grow with the load. | fix #566 |
| L3-L02 | Culling phase one's fixed cost holds one level in every run here and another in some Baselines: 27.1 to 28.8 µs in all 12 runs of the two-to-four-triangle scenes, whose Baselines hold 24.2, 24.2 and 28.5 for the same work, so `compare` warns on whichever scenes' Baselines caught the other level | `health`, 4 scenes warned in every round, about 4 µs a frame | `bench/baselines/apple-m4-pro/{uv-grid,normal-tangent,far-coplanar,shadows}.json` (`culling_phase_one`); `src/render/culling_pass.rs:284-320` (`record`, the four dispatches); `src/bench/mod.rs:143` (the 20 µs noise floor that keeps it a warning) | uv-grid 27.4, 27.5, 28.3, 27.1 against 24.2; normal-tangent 40.7, 40.3, 40.8, 40.2 against 37.7; far-coplanar 28.8, 28.0, 28.5, 28.4 against 24.2; shadows 33.4, 33.3, 33.4, 33.6 against 28.6; helmet, pbr-sweep, orrery, minification and crowd within 4.4% of theirs; the commits since the Baselines touch `src/bench/` and `src/bin/` only; the note of 2026-10-30 on the depth pyramid and culling read uv-grid's phase one at 28.8 | Baselines recorded in one bench run on a quiet machine, so a fixed cost the scenes share holds one level across them | `narrowed`: Culling phase one's fixed cost on the small light scenes takes one of two levels about 3 to 4 µs apart, from run to run at the same commit: uv-grid 24.0 or 26.9 to 28.3 µs, far-coplanar 23.9 or 27.5 to 28.8, normal-tangent 36.8 or 39.9 to 40.8. So `compare` warns on whichever scenes land above their Baseline's level, 2 to 4 scenes a round, about 4 µs a frame. The two levels are real, but they flip from run to run at the frozen commit (far-coplanar 23.9 in round 1, uv-grid 24.0 in round 4, normal-tangent 36.8 in round 1), not only between runs and Baselines. Only shadows is warned in every one of the verifier's rounds (2, 4, 4 and 3 scenes warned in rounds 1 to 4). | fix #528 (folded into the exit's quiet re-record, every scene in one run) |

### Architectural notes

Closed here, not Findings: each pass within 2x of its floor, or whose
floor no published figure gives. Outside the shadow mask pass, the
highest ratio of any light-scene pass to the most generous end of its
floor is 1.81 (helmet's `culling_phase_one`).

- **Where a pass's bytes are its targets, it reads under its uncompressed
  floor.** On the 7 scenes drawing under 70,000 indices, the depth pre-pass
  reads 157 to 168 µs against 216 to 225 and the lit pass 342 to 533 µs against 811 to 820; on all nine,
  the depth pyramid reads 203 to 216 µs against 234 and the tonemapping pass
  319 to 320 µs against 486. The MSAA depth and HDR targets are read and
  written compressed, as the dense section's note on the tonemapping pass
  says [ref:apple-metal-lossless-compression], so on this GPU a bandwidth
  floor over an MSAA target is an upper bound. The helmet's and the
  crowd's geometry passes sit inside their vertex-fetch bounds, the depth
  pre-pass at 0.75 to 0.98 and 0.91 to 1.41 of them, the lit pass 0.89 to 0.96 and
  0.71 to 0.83: the same open bound as `town`'s, which the probe the dense
  section names decides.
- **The depth pyramid is 1.9 to 3.0 times its sources on every scene**
  and under its own floor: the comparison the dense section and the note
  of 2026-10-30 closed as not applying, the sources reducing HTILE or a
  16-bit single-sample buffer.
- **The shadow mask pass is within 2x of its floor on every scene but the
  helmet, even at uv-grid's rate.** Against the held code's 1,150 µs plus
  its rays at 0.08 ns: helmet 3.29, pbr-sweep 1.89, uv-grid 1.08, normal-tangent 1.63, far-coplanar 1.12, orrery 1.52, minification 1.10, shadows 1.68, crowd 1.49. The helmet's
  3.29 is at a rate measured on two triangles; the only rate measured on a
  BLAS of its size is its own, from the same pass's time before the
  one-call merge, against which it reads 0.91, a ratio that tests nothing.
  So its floor is not computable from a figure apart from the scene, as
  the dense section found for `town`'s Froxel pass. The held code's
  1,150 µs is more than half of the pass on uv-grid, far-coplanar, orrery, minification and crowd
  (67%, 78%, 63%, 68%, 59%): there the pass's cost is its dispatches and its held
  ray code, as that note found, not its rays. No scene's pass is over 2x its
  source's figure for its lights (helmet 0.58 to 0.82, pbr-sweep 0.31 to 0.43, uv-grid 0.20 to 0.28, normal-tangent 0.28 to 0.40, far-coplanar 0.52 to 0.64, orrery 0.21 to 0.30, minification 0.20 to 0.73, shadows 0.33 to 1.22, crowd 0.69 to 0.85).
- **Culling phase one's line is mostly its fixed cost on these scenes.**
  27.4 µs for two triangles; the helmet's 75.4 is 1.81 of its floor and the
  crowd's 92.8 1.66. The note of 2026-10-30 on the depth pyramid and
  culling split the helmet's phase one into `append_records` 41.3 µs,
  `append_triangles` 20.2 and the scans 19.1, a chain of dependent loads
  over at least 2,073 Meshlets (256,938 triangles at 124 a Meshlet at most),
  so the gap above the bytes is latency. The source's 250,000-object
  figure does not apply.
- **The crowd's bend pass reads under its DRAM floor**, 27.9 µs against
  56 for 15.2 MB read and written: the floor assumes every byte
  reaches DRAM, and no reference here sizes the M4 Pro's caches, so the
  floor does not bind. The crowd's dynamic TLAS build reads 0 at the
  median, beside the previous frame, as on `town`.
- **The crowd's vertex Arena is 67.5% full**: 11,325,896 B of
  `arena_bytes_in_use` against 16 MiB (`src/assets/pools.rs:9`), the
  highest of any scene; nothing has been refused.

### The revive-when lines, re-judged

The map's *Not yet specified* holds 36 lines containing "revive" on
2026-11-02 (its body last edited 2026-11-01), against the 27 the ticket
counted at charting; all 36 are judged, numbered R01 to R36 in the order
they appear, then the two open offender tickets the map places at M4's
charting. A line is `fired` when its named trigger has happened, with the
measurement or line that proves it; `waiting` when the trigger is named
and not met, with what was checked; `dead` when the trigger can no longer
fire or the code it names is gone. Two lines name code that has moved
rather than gone (R31 and R32: the lit pass's ray queries now live in the
shadow mask pass), so each is judged against where the code is now. Two
read-only sub-readers at high effort gathered the evidence, R01 to R18
and R19 to R36 with the offenders; this reader judged six lines one of
them skipped (R21, R23 to R27), re-ran every `fired` verdict and R01, R09,
R16, R22, R31, R32 and R35, and ran the lone counter benches R01 rests
on (every counter equal to the full rounds). Counts: `fired` 5 (R04, R12, R20, R31, R34),
`waiting` 31 lines and both offenders, `dead` none. A `fired` line is the
map's to act on: it revives the work or moves the trigger.

| Line | Subject | Verdict | Evidence |
|---|---|---|---|
| R01 | Heap-allocation counters, a lone run against a full run | `waiting` | Trigger: a scene's counters differ between `praxis bench <dir> <scene>` and a full run. On the M4 Pro they do not: a lone `praxis bench <dir> --counters --skip-aliasing --no-publish <scene>` per Reference scene, one process each, under `docs/audit/m3.5/bench/lone-counters/`, reads every counter equal to the full rounds' (this section's four rounds for the nine light scenes, the dense section's for `town` and texture-wall: `heap_allocations` 347, 347, 345, 345, 345, 455, 345, 350, 345, 2,099 and 10,498). The line's case is the RTX 2070 SUPER, whose texture-wall Baselines read 357 (Vulkan) and 335 (DX12) (`bench/baselines/nvidia-geforce-rtx-2070-super*/texture-wall.json`); that machine was not run here. The ticket "Bench figures on the RTX 2070 SUPER depend on the scenes run before them" closed 2026-10-31, and no later note in `docs/performance.md` records a lone-against-full difference. |
| R02 | wgpu 31 bump | `waiting` | `curl -s https://crates.io/api/v1/crates/wgpu`: `max_version` 30.0.1, `max_stable_version` 30.0.1, updated 2026-08-22; `gh api repos/gfx-rs/wgpu/releases`: newest tag v30.0.1. `Cargo.toml:8` requires `wgpu = { version = "30", ... }`. The TLAS dependency dedup the line waits for (https://github.com/gfx-rs/wgpu/pull/9835) merged upstream 2026-07-17 and is in no release. |
| R03 | Mirror reflections with water | `waiting` | `docs/adr/0020-shadows-reflections-shadow-mask.md:21`: mirror reflections "return with water". `grep -rn -i water src` hits only `src/scenes/town.rs` (`TownMesh::WaterTank`, a static Mesh); no water surface, shader or Material. Milestones: M0 to M3 closed, M3.5 open (`gh api 'repos/gariasf/praxis/milestones?state=all'`). |
| R04 | Layout check, element strides | `fired` | Trigger: a `mat3x3`, `mat2xN` or `array<vec3>` field joins a struct shared with the CPU. One has: `src/render/shaders/common.wgsl:176-177` (`InstanceData { rotation_scale: mat3x3<f32>, normal_matrix: mat3x3<f32>, ... }`), its Rust twin `src/render/instance.rs:39-40` (`[[f32; 4]; 3]`, padded per column by `padded_columns`, `:21-23`), checked by `twin!(InstanceData {...})` at `src/render/shader_tests.rs:268`. It joined at 39ccb000 (2026-10-14), after the ticket "Shader and layout validation in tests" closed (2026-10-12). The check still compares each member's offset and total size (`shader_tests.rs:700-704`; `size_of_val` at `:224`), so `[[f32; 3]; 4]` (48 bytes) would pass against `mat3x3<f32>` (48 bytes); the review checklist in `CLAUDE.md` carries the rule by hand ("the test compares sizes, not strides"). |
| R05 | Specular BRDF, the remaining terms | `waiting` | Trigger: ADR 0020's reflection cubemaps (M5) or a BRDF survey. `grep -rn -i cubemap src`: 0 hits; no M5 milestone; `ls docs/research` holds no BRDF survey (the ticket the line names is the oracle harness, closed). M3's Smith term is height-correlated per glTF Appendix B (`src/render/shaders/shading.wgsl:124-130`). |
| R06 | Frame pacing and present timing | `waiting` | `gh api repos/gfx-rs/wgpu/issues/2869 --jq '.state,.title,.closed_at'`: `open`, "Extended Presentation API Investigation", `null`. `grep -rn -i 'promotion\|variable.refresh\|vrr' src`: 0 hits. |
| R07 | Input latency measurement | `waiting` | Trigger: input first drives a Reference scene. No Reference scene run consumes input: `grep -rn Input src/bench src/capture src/scenes/mod.rs`: 0 hits; camera paths are frames (`src/scenes/mod.rs:512`, `orbit(frame, ...)`). Input drives the interactive fly camera (`src/camera/mod.rs:104-119`) and the helmet spawner (`src/helmet/mod.rs:27-34`) only. No latency measurement exists (`grep -rn -i latency src`: `src/lib.rs:165`, `desired_maximum_frame_latency`, and one comment). |
| R08 | Vertex Arena past 128 MiB | `waiting` | `src/assets/pools.rs:9` `ARENA_CAPACITY_BYTES: u64 = 16 << 20`; the floor raises no `max_storage_buffer_binding_size` (`src/gpu/mod.rs:296-308`); `src/gpu/mod.rs:757-760` asserts the Arena fits one binding of the floor; no M4 milestone. |
| R09 | Transform removed from a parent | `waiting` | `grep -rn 'remove::<' src`: `src/render/prepare.rs:763` (MeshRef), `:907` (PreviousWorldTransform), `:999` (MaterialRef), `src/transform/mod.rs:83` (Teleported) and `:264` (ChildOf, in `#[cfg(test)]`); no `remove::<Transform>`. Propagation still keys on `Moved` and `RemovedComponents<ChildOf>` (`src/transform/mod.rs:95-108`). |
| R10 | Archive packaging | `waiting` | `grep -rn -i 'archive\|content-address\|64 KiB' src`: one unrelated hit (`src/render/acceleration_structures.rs:66`, placement granularity). Blobs load one per file from `cooked/` (ADR 0018); no milestone holds ship builds or direct I/O. |
| R11 | Attachment reset in the Tick blend | `waiting` | `Teleported` is inserted in a test only (`src/state.rs:528`, under `#[cfg(test)]`). Children of moving parents spawn at scene setup, before Tick 0: the orrery `src/scenes/mod.rs:649`, `town`'s flocks `src/scenes/town.rs:803`, the crowd `src/scenes/mod.rs:1029`; the one runtime spawn, `spawn_helmet` (`src/helmet/mod.rs:31-34`), has no parent. |
| R12 | Rotation in the Tick blend | `fired` | Trigger: an entity turns faster than about 6 rad/s. The orrery's bodies spin at `ORRERY_SPIN_RATES = [0.5, 1.5, 2.5, 3.5, 4.5]` rad/s (`src/scenes/mod.rs:619`), each a child of the one before (`:641-651`) about the same +Y axis (`:668`, `Quat::from_rotation_y`), and World = parent World × local (`src/transform/mod.rs:134`) sums the rates: 0.5, 2.0, 4.5, 8.0 and 12.5 rad/s in World, so bodies 4 and 5 exceed 6 rad/s. `WorldTransform::drawn` still lerps every matrix entry (`src/transform/mod.rs:21-35`). With a Tick of 1/64 s (`src/time/mod.rs:21`), body 5 turns 12.5/64 = 0.195 rad a Tick, a mid-Tick shrink of 1 − cos(0.098) = 0.48% (body 4: 0.20%); the line's own figure checks (5.8/64 = 0.091 rad gives 0.10%). The rates date from 3cf054a9 (2026-10-14), so the trigger had fired when the line was written. `town`'s flocks spin at 0.2 to 0.6 rad/s (`src/scenes/town.rs:241`); the crowd's sway is a Bend, not a rotation. |
| R13 | Toksvig's ceilings | `waiting` | Cooker unchanged (`src/cook/texture.rs:440-442`). Content (python3 over `assets/SciFiHelmet/SciFiHelmet.gltf` and the JSON chunk of `assets/NormalTangentTest/NormalTangentTest.glb`): one Material each, `roughnessFactor` absent (1), one metallic-roughness image used by one Material, metallic-roughness and normal images both 2048×2048, so no coarser or other-aspect normal map; no other glTF source under `assets/`. |
| R14 | glTF's default Material | `waiting` | Every Primitive of both glTF files names material 0. `src/render/prepare.rs:370` still warns and skips the case; the generated scenes' Material-less Primitives (`src/scenes/mod.rs:278,284,309`) carry `MaterialRef` on their Renderables (`:554,611`), the path the line allows. |
| R15 | Particles, bloom and motion blur | `waiting` | `grep -rn -i -l 'particle\|bloom\|motion blur' src`: no file. No milestone past M3.5 names them. |
| R16 | Frame stats' GPU memory sum | `waiting` | `grep -rn '\.frame_stats()' src tests`: `src/capture/mod.rs:262` and `:434` only, which the bench runs through; no per-frame caller. `MaterialPool::gpu_memory_bytes` (`src/assets/material.rs:228-236`) still walks each Texture's mip chain (`src/stats/mod.rs:193-197`). No Texture streaming; no M4 milestone. |
| R17 | Per-Meshlet culling tests | `waiting` | Trigger: the gate's profile shows a raster-bound pass. The dense section puts `town`'s depth pre-pass at 2,142 µs against a vertex-fetch floor of 870 to 2,486 and its lit pass at 3,764 against 1,499 to 3,115, "which bound binds is open", the deciding probe unrun; no profile names a raster-bound pass. Meshlet culling was prototyped and declined (`docs/performance.md`, "Meshlet culling prototyped on `town`", 2026-10-28: the gate unchanged at 8.6 ms). |
| R18 | Composing each Instance once, in the culling pass | `waiting` | The floor holds nine storage buffers in the compute stage (`src/gpu/mod.rs:304-306`; ADR 0021); the vertex shader still composes per vertex (`src/render/shaders/main.wgsl:96-98`); no profile names it (`grep -n -i 'compos\|vertex shader' docs/performance.md`: one hit, the visibility-buffer prototype's prose). |
| R19 | Composing point lights once a frame | `waiting` | Code as the line describes: `src/render/shaders/main.wgsl:279-285` composes each Froxel light per fragment, `src/render/shaders/froxel.wgsl:38` per Froxel. No profile names the lit pass's light loop: `grep -n -i "light loop" docs/performance.md` hits the shadow mask pass's loop only; the dense section puts `town`'s lit pass within its vertex-fetch floor. |
| R20 | Depth pyramid dispatches | `fired` | Trigger: a profile names the pyramid's tail. The note of 2026-10-30 on the depth pyramid and culling does (`docs/performance.md:347-353`, "Pyramid lines, level 0 then the levels after it": `town` 225.3 + 40.0 µs, the helmet 200.0 + 39.2, uv-grid 186.0 + 38.8) and lists its option P3 (`:377`, "One dispatch for the levels after level 3 or 4 ... at most 40 µs ... unmeasured"); P2 measured 2.8 µs slower (`:376`); only the per-pixel level 0 shipped (6aec9278, 2026-10-31). The pass still dispatches a level per iteration (`src/render/depth_pyramid_pass.rs:113-125`), and no open ticket carries P3. |
| R21 | Spot lights | `waiting` | No spot light in code or content: `grep -rn -i spot src` hits only comments and messages about lights "at one spot" (`src/render/renderer.rs:2024,2145,2176`); `assets/SciFiHelmet/SciFiHelmet.gltf` has no `extensionsUsed`. |
| R22 | GPU memory counted where it is created | `waiting` | No resource turned up missing: `git log --oneline -S gpu_memory_bytes -- src` shows each new resource counted at creation (6f607e83 the BLASes, 2b12b776 the Froxel lists, 5c570f7e the bend pass, bd7b9886 the shadow mask) and one removal, 5264094f ("Leave the GPU stats readback out of gpu_memory_bytes"), which counts less, not more; `gh issue list --state all --search "gpu_memory_bytes"` lists no uncounted resource. Per-category bytes are not needed: `bench/budgets.json:7` holds one `gpu_memory_bytes`; M4 is not charted. |
| R23 | USD as an authoring source | `waiting` | Trigger: M4's charting, M9's World editor, M7's save/load, or content shipped only as USD. Milestones: M0 to M3 closed, M3.5 open; `grep -rn -i 'usd' src Cargo.toml`: 0 hits; the assets are glTF (`assets/manifest.json`). |
| R24 | Exposure adaptation and local tone mapping | `waiting` | Trigger: a scene joining an interior and sunlight (M5) or a time of day (M6). `grep -rn -i 'interior\|time of day\|histogram\|adaptation' src`: 0 hits; every Reference scene sets one EV100 by hand (`src/scenes/mod.rs:354-367`). |
| R25 | Reference scenes' sky and ambient | `waiting` | Trigger: M5's probes or M6's atmosphere. `src/environment/mod.rs:4` still says "irradiance probes (M5) and M6's atmosphere replace it"; every scene keeps `ambient_illuminance` at a tenth of the sun (`src/scenes/mod.rs:462-464`); neither milestone exists. |
| R26 | HDR display output | `waiting` | Trigger: the owner wants HDR output. `grep -rn -i 'edr\|hdr10\|scrgb' src/gpu src/lib.rs`: 0 hits; `gh issue list --state all --search "HDR output OR EDR"` returns the HDR survey and M3's tonemapping ticket, both closed, and no output ticket. |
| R27 | Colour grading | `waiting` | Trigger: the owner wants a look beyond GT7's. `grep -rn -i 'lut\|grading\|agx' src/render/shaders/tonemapping.wgsl src/render/tonemapping_pass.rs`: 0 hits; `gh issue list --state all --search "colour grading OR color grading OR LUT"` returns the map, the HDR survey and M3's main pass ticket only. |
| R28 | GPU stats readback batching | `waiting` | The ring still resolves one slot a frame (`src/render/readback.rs:227-231`, `:364-375`, `:491`). No profile names the readback: `grep -n -i readback docs/performance.md` hits only its GPU memory count and an environment variable; the note of 2026-10-30 on `town`'s CPU frame attributes its samples to `submit`, the dynamic TLAS and `prepare`. |
| R29 | Specular filtering under anisotropic sampling | `waiting` | Trigger: content shows sparkle or a missing mirror image on glossy ground. `gh issue list --state open --search "sparkle OR glint OR shimmer"` returns the map and "town's sub-pixel wires, railings and pigeons alias into dots", geometry aliasing, not specular sparkle on glossy ground. The data Channels still sample at 4x (`src/render/shaders/main.wgsl:175-178`). |
| R30 | Channel kinds named once | `waiting` | Trigger: a Channel is added or changes kind. `MaterialChannel` holds Albedo, Normal, MetallicRoughness, Occlusion and Emissive (`src/assets/material.rs:47-53`), unchanged since f4c7442d (2026-10-22, the line's own change: `git diff f4c7442d 8bc34eb1 -- src/assets/material.rs` changes no variant); the restated split persists (`src/cook/texture.rs:163,198,244,417,515`, `src/assets/material.rs:184`, `src/render/shaders/main.wgsl:173-178`). |
| R31 | Lit pass without the dynamic TLAS's query while nothing moves | `fired` | Trigger: a scene with no Moving Instance nears its GPU Budget. texture-wall has none (its dynamic TLAS CPU line reads 0.2 to 0.3 µs in the dense section's rounds) and is over its Budget (L3-D05). The code the line names has moved: the lit pass holds no ray query since bd7b9886 ("Trace shadow rays from a compute pass into a shadow mask"; `grep -n -i "rayQuery\|acceleration_structure" src/render/shaders/*.wgsl` hits `shadow_mask.wgsl` and `dynamic_tlas_marker.wgsl` only), and the shadow mask pass holds the dynamic query's code (`src/render/shaders/shadow_mask.wgsl:99-115`), skipping it at run time while nothing moves (`:103`, `camera.dynamic_tlas_entries == 0u`), the form the line found not enough. A shadow mask pipeline without the dynamic query, chosen while nothing moves, is the line's fix on today's code. It would not close texture-wall's breach, whose cost is the untimed stretch of its bound Textures (L3-D06). |
| R32 | Empty static TLAS | `waiting` | Trigger: a scene of Moving Instances alone matters. The orrery is one (every body carries `PreviousWorldTransform`, `src/scenes/mod.rs:633-639`) and reads a 2.9 ms GPU frame against 12, so it does not yet matter. The query moved with R31's: `src/render/shaders/shadow_mask.wgsl:99-101` initialises and proceeds the static query unconditionally, and no `static_tlas_entries` exists (`src/render/uniforms.rs:58` holds only `dynamic_tlas_entries`), so the line holds against the shadow mask pass, not the lit pass. |
| R33 | Falling back to DX12 below the Capability floor | `waiting` | Code as described: `src/gpu/mod.rs:479-483` (`BACKEND_ORDER`: Vulkan, Metal, Dx12), `:498-509` takes the first adapter, `:60-66` checks the floor once on it. No adapter has hit it: `gh issue list --state all --search "Capability floor"` lists no such report; the RTX 2070 SUPER meets the floor on both backends (its Baselines under `bench/baselines/nvidia-geforce-rtx-2070-super*/`). |
| R34 | Path-filtered PR CI | `fired` | Trigger: docs-only PRs become a larger share, "as at a milestone boundary's research tickets". `gh pr list --state merged --limit 60 --json number,files`, a PR counted docs-only when every path is under `docs/` or a root `*.md`: 18 of the last 60 merged (2026-10-30 to 2026-11-02; the largest has 43 files, under the listing's cap; 16 of 60 counting only those merged by the frozen commit), against the 3 of 60 the line quotes. `.github/workflows/ci.yml:3-11` still runs every engine job on PRs; `paths-ignore` applies to `push` only. |
| R35 | More convention rules | `waiting` | Trigger: a review catches one by hand. `tests/conventions.rs:18-38` has no rule for `.expect(`, `dbg!`, `/* */`, raw strings or PR-number citations. In `src/` and `tests/`: `dbg!(` 0, `/*` 0, PR-number citations 0, `r#"` 15 (test data in `src/bench/*.rs`), `.expect(` 3 in the render path (`src/render/staging.rs:201,222,228`). `gh api search/issues -f q='repo:gariasf/praxis is:pr "dbg!"'` finds only the change that wrote the line. |
| R36 | Toksvig widening and the normal scale | `waiting` | Every asset's `normalTexture.scale` is 1 (SciFiHelmet's one Material, NormalTangentTest's one Material); the only other scale is a unit test's (`src/assets/material.rs:430`, 0.5). |
| Offender: bound Textures | 10,005 bound Textures cost 1.1 to 1.8 ms of CPU a frame | `waiting` | Ordered with the survey "State of the art: bindless residency cost on Metal and wgpu" when M4 is charted. No M4 milestone; the survey has no ticket (`gh issue list --state all --search "bindless residency"`: none), and `docs/research/gpu-memory-residency-streaming.md:79` calls Metal's per-Texture `useResource` cost "not measured". The cost persists at the frozen commit: L3-D05 and L3-D06. |
| Offender: fixed Arenas | The fixed Arenas are most of every scene's GPU memory | `waiting` | Ordered at M4's split of the 8 GB Budget; no M4 milestone; `bench/budgets.json` holds one `gpu_memory_bytes`. `src/assets/pools.rs:9` sets 16 MiB. The highest `arena_bytes_in_use` of any Baseline is the crowd's 11,325,896 B (67.5% of one Arena), the same in this section's rounds; no ticket names a refusal. |

## Killed

The verifier re-ran each of these and found the claim not to hold;
each stays here, with the reader's evidence, so the next audit does
not re-find it. Its id stays retired.

| id | claim | severity | where | evidence | killed because |
|---|---|---|---|---|---|
| L3-D07 | The quiet stamp marks every texture-wall run not quiet on a quiet machine, because its GPU frame p95 is 7 to 11% over its median by the scene's own nature | `health`, 3 runs of 1 scene; every Series record of texture-wall on this machine | `src/bench/provenance.rs:30` (`QUIET_P95_OVER_MEDIAN = 1.06`) and `:130-140` (`is_quiet`) | `gpu_frame_p95_over_median` 1.092, 1.072 and 1.105 with `gpu_load_percent` 9, 3 and 2, `other_processes` empty and AC power, in the three texture-wall files; `town` in the same rounds 1.040, 1.043 and 1.011; texture-wall's own Baseline run has 1.054 (under the rule) and the M2 exit saw p95 jumps of 1.6 to 3.5 ms in four of six runs | It does not reproduce: in the verifier's three rounds at GPU load 0, texture-wall reads p95/median 1.043, 1.049 and 1.019 and is stamped quiet, and the only round stamped not quiet (round 3, 1.126) had a measured 46% GPU load, so the stamp tracked the machine, not the scene. |

**Evidence for L3-D07** (the quiet stamp reads a scene's spread, not the
machine's state), from the light section's runs: 14 of 36 light-scene runs
stamped not quiet at `gpu_frame_p95_over_median` 1.062 to 1.207, with
`gpu_load_percent` 0 to 1, no other praxis process and AC power, in rounds
whose GPU frame medians agree within 0.97%; far-coplanar not quiet in 3 of
4 rounds; the Bench history then keeps one quiet round per scene and
drops the rest.

What the kill leaves open: the light section's rounds stamped 14 of 36
runs not quiet while the bench's GPU load read 0 to 1% at each start,
and preflight read 99% seconds before two of those rounds; the verifier's
rounds at 0% stamped 32 of 33 runs quiet, and its round at 46% stamped
all eleven not quiet. The stamp reads the GPU load once, when `praxis
bench` starts (`BenchStart::read`, `src/bench/provenance.rs:74-89`), for
every scene it then runs; a later audit that sees stamps flip with the
load at 0% reads the load across the run before filing it again.
