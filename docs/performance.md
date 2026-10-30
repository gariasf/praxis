# Performance notes

Profiles of the Reference scenes, one section per milestone exit, and
measurements that name an offender's cause, newest first. Each exit names
its top three offenders and the tickets that take them.

## `town`'s CPU frame: prepare walked every Renderable, and wgpu-core's TLAS bookkeeping is the rest (2026-10-30, #405)

**Verdict.** Quiet, `town`'s CPU frame read 5.0 to 5.8 ms at the median
and 5.4 to 6.3 ms at p95 on `main`, and prepare was 1.3 ms of it: it
visited all 254,256 Renderables every frame, a pool lookup and a hash
lookup each, to find the 10,000 birds that move. Prepare now visits
only the Renderables whose World transform, Mesh or Material changed,
the blended ones, and those it skipped or refused the frame before; it
reads 340 µs, and the CPU frame 4.0 to 4.5 ms at the median and 4.3 to
4.8 ms at p95. The dynamic TLAS's CPU read 934 to 987 µs over the same
four runs, under its 1.1 ms ceiling on both builds: the M3 exit read
1.10 to 1.26 ms at load 2.7 to 4.2. Of what is left, 2.1 to 2.6 ms
is `submit` and most of the dynamic TLAS's line is its build inside
wgpu-core, both work per TLAS Instance that wgpu-core 30.0.1 does
[ref:wgpu-30-tlas-dependency-per-instance]
[ref:wgpu-30-submit-tlas-dependencies]
[ref:wgpu-30-tlas-instance-staging]. Patched with upstream's dedup
[ref:wgpu-pr-9835], the CPU frame reads 1.8 to 1.9 ms and the dynamic
TLAS 619 to 665 µs, with the GPU lines unchanged. That patch is an
option for the owner, on the issue.

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release builds of
`main` at a5280dce and of this change, each with a measurement patch,
not committed, that logged each CPU stage's median and p95 over the
measured frames (`FrameStats::cpu_times`, the systems summed per frame).
`praxis bench <dir> --skip-aliasing town` at 2560×1440 with 4x MSAA,
1,800 measured frames, the builds interleaved, nothing else on the GPU,
load average 1.8 to 2.7. CPU frame and dynamic TLAS from the bench's
JSON, stages from the patch's log; µs. Systems is the sum of the
systems' medians, the Ticks' propagation most of it.

| Build | Run | CPU frame | CPU frame p95 | Dynamic TLAS, CPU | Its p95 | `prepare_frame` | Systems | `dynamic_tlas` | `dynamic_tlas_submit` | `encode` | `submit` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Main | 1 | 5,554.5 | 5,970.8 | 1,003.8 | 1,139.4 | 1,315.2 | 389.8 | 594.8 | 409.9 | 205.9 | 2,576.4 |
| Main | 2 | 5,816.3 | 6,345.3 | 1,049.9 | 1,278.3 | 1,393.9 | 402.3 | 625.5 | 419.3 | 206.0 | 2,635.3 |
| Main | 3 | 5,002.5 | 5,372.6 | 933.7 | 1,030.7 | 1,317.9 | 390.3 | 580.5 | 355.5 | 197.3 | 2,103.5 |
| Main | 4 | 5,060.6 | 5,396.5 | 944.7 | 1,012.6 | 1,314.3 | 389.4 | 580.9 | 364.0 | 196.2 | 2,162.0 |
| Change | 1 | 4,001.4 | 4,315.4 | 934.1 | 1,009.9 | 340.8 | 388.9 | 584.5 | 353.3 | 188.1 | 2,094.2 |
| Change | 2 | 4,081.2 | 4,428.0 | 961.4 | 1,045.4 | 343.1 | 395.7 | 607.1 | 356.8 | 190.6 | 2,136.4 |
| Change | 3 | 4,451.3 | 4,837.5 | 979.9 | 1,088.6 | 339.6 | 388.8 | 584.0 | 393.2 | 186.7 | 2,506.9 |
| Change | 4 | 4,476.8 | 4,803.6 | 986.5 | 1,073.1 | 340.0 | 389.3 | 587.3 | 401.5 | 186.6 | 2,517.2 |

`upload` read 30 to 38 µs throughout. `submit` takes one of two levels
for a whole run, about 2.1 or 2.5 ms, on either build; the CPU frame
follows it. Counters were identical between the builds: 2,049,344
uploaded bytes and 10,501 heap allocations a frame.

**Where the rest goes.** `sample` at 1 ms over 20 s of one run of the
change:

- **`submit`**: 978 of its 1,291 samples are wgpu-core's
  `set_acceleration_structure_dependencies`, which walks the static
  TLAS's dependencies, one BLAS per Instance, and wgpu-hal then adds
  each to a new Metal residency set, 508 samples
  [ref:wgpu-hal-30-metal-residency-per-submit].
- **`dynamic_tlas`**: 511 samples are `CommandEncoder::finish` on its
  encoder, where wgpu-core records the build: per Instance it allocates
  the backend's record and frees it once staged, 161 samples of
  allocation and release [ref:wgpu-hal-30-metal-tlas-instance-bytes].
  Filling the 10,000 entries in `DynamicTlas::record` is about 110.
- **`dynamic_tlas_submit`**: its own walk of the 10,000 dependencies,
  for the build and for each of the two markers that bind it.

None of these is ours to cut short of the TLAS holding fewer Instances.

**Upstream's dedup.** wgpu-core 30.0.1 with gfx-rs/wgpu#9835's 49-line
change applied by a `[patch.crates-io]` entry, so each build records one
dependency per distinct BLAS. Runs interleaved with the change alone,
load average 2.7 to 4.5:

| Build | Run | CPU frame | CPU frame p95 | Dynamic TLAS, CPU | Its p95 | `prepare_frame` | Systems | `dynamic_tlas` | `dynamic_tlas_submit` | `encode` | `submit` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Change | 1 | 4,650.3 | 5,067.7 | 1,045.8 | 1,186.6 | 342.0 | 394.2 | 631.3 | 418.3 | 230.6 | 2,576.7 |
| Change | 2 | 4,610.6 | 4,966.5 | 1,030.7 | 1,147.2 | 341.1 | 390.7 | 620.6 | 413.9 | 218.0 | 2,571.6 |
| Change | 3 | 4,570.7 | 4,950.1 | 1,023.5 | 1,136.2 | 341.3 | 391.2 | 614.2 | 411.4 | 215.2 | 2,542.6 |
| Change + wgpu#9835 | 1 | 1,858.5 | 2,206.0 | 664.9 | 803.4 | 455.8 | 394.5 | 630.9 | 33.8 | 234.6 | 43.1 |
| Change + wgpu#9835 | 2 | 1,767.2 | 2,092.2 | 625.8 | 715.7 | 457.4 | 388.2 | 598.0 | 28.2 | 207.7 | 40.0 |
| Change + wgpu#9835 | 3 | 1,757.9 | 2,094.2 | 618.8 | 708.2 | 455.3 | 388.5 | 591.5 | 27.8 | 205.1 | 40.2 |

The GPU frame read 20,169.7 to 20,193.0 µs patched against 20,178.4 to
20,247.8, the gate 8,676.6 to 8,681.6 against 8,673.0 to 8,684.8, and the
patched build allocates 37 fewer times a frame. The build's per-Instance
records stay, so `dynamic_tlas` does not move. `prepare_frame` rises
115 µs in the patched build, though its code is the same; this note does
not find why. Upstream's `v30` branch does not carry the change, and no
wgpu 31 is released.

## Where the shadow mask pass's time goes (2026-10-30, #404)

**Verdict.** On `town`, shadow rays cost 11.0 ms: about 9.3 ms is rays
and 1.7 ms is fixed. The sun's rays take about 5.7 ms of the rays,
2.2 ms of them its edge rays. The lamps' rays take 4.1 ms, 0.86 ms of
that the dynamic TLAS's queries. The second surfaces take 1.7 ms.
These overlap (the second surfaces trace both kinds of ray), so they do
not sum. A ray costs what its traversal costs: about 0.96 ns on the
helmet's 257,000 triangles against 0.08 ns on uv-grid's two. On the
light scenes, the pass also reads about 0.45 ms longer because the
shader holds ray-query code at all, even where no ray is traced. Tracing every ray of a surface through a
single call, from a single light loop, cuts `town`'s shadow-ray time
by 0.79 to 0.97 ms and the helmet's by 0.83 to 0.95 ms, tracing the
same rays; 18 single pixels of the gallery's 19 images change, within
the Goldens' tolerance. It costs uv-grid 0.09 to 0.12 ms. `town`'s GPU frame
stays about 7 ms over its 12 ms Budget; the options at the end are
the owner's.

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release builds of
`99fdb38e`. `praxis bench <dir> --skip-aliasing` at 2560×1440 with 4x
MSAA, 248 warm-up and 1,800 measured frames, rays on then off, read as
shadow-ray time (GPU frame with rays minus without) and the shadow mask
pass's line. A probe patch, not committed, added pipeline overrides
that turn off one stage or one kind of ray at a time. Its variants ran
interleaved, two rounds of each, and the tables give the median of the
rounds, from the bench JSON by a script kept with the probe:
`praxis report` reads one run against the Baselines, not a variant's
rounds. Load average was 29 at the first run, just after a peer
session's bench, 18 and 7.5 at the next two, then 2.4 to 5.5, and 1.8
to 5.0 over the second batch; no other session benched.
Rays a pixel come from `praxis capture <scene> <png> 2560x1440`: the
lit pixels and the point lights reaching each one, in the last frame.

### Each stage and each kind of ray, off in turn

Shadow-ray time, then the shadow mask pass's line, µs.

| Variant | `town` | helmet | uv-grid |
|---|---:|---:|---:|
| As shipped | 11,007 / 12,239 | 5,636 / 5,450 | 1,712 / 1,570 |
| Second surfaces off | 9,317 / 10,145 | 5,352 / 5,119 | 1,712 / 1,567 |
| Edge rays off (listing and tracing) | 8,779 / 9,748 | 5,126 / 4,957 | 1,533 / 1,383 |
| Directional rays off (edges too) | 5,266 / 6,290 | 4,138 / 3,962 | 1,604 / 1,473 |
| Point-light rays off | 6,860 / 7,809 | 2,654 / 2,474 | 1,352 / 1,268 |
| Dynamic TLAS queries off | 9,281 / 10,369 | 5,340 / 5,182 | 1,704 / 1,563 |
| No rays, code compiled out | 1,738 / 1,420 | 873 / 730 | 811 / 672 |
| No rays, nearest surfaces only | 1,961 / 813 | 758 / 514 | 654 / 489 |
| Bare dispatches: load depth, store words | 1,565 / 256 | 698 / 241 | 527 / 241 |

A second probe batch, interleaved the same way, gave these; the
helmet's and uv-grid's caps ran in a third, at load 2.5 to 7.8:

| Variant | `town` | helmet | uv-grid |
|---|---:|---:|---:|
| As shipped | 11,223 / 12,400 | 5,580 / 5,471 | 1,704 / 1,570 |
| Ray code kept, every ray skipped at run time | 1,493 / 1,907 | 1,280 / 1,150 | 1,281 / 1,141 |
| No point-light ray, code kept | | 2,562 / 2,406 | 1,405 / 1,257 |
| At most 1 point-light ray a surface | 10,011 / 11,372 | 3,444 / 3,306 | 1,497 / 1,349 |
| At most 2 | | 4,245 / 4,058 | 1,562 / 1,420 |
| At most 4 | 11,257 / 12,394 | 5,633 / 5,442 | 1,707 / 1,564 |
| At most 16 | 11,265 / 12,388 | | |
| No dynamic TLAS query for point-light rays | 10,359 / 11,553 | | |

- **Fixed cost.** The bare dispatches take about 0.25 ms on every
  scene. Rebuilding the surface and walking its lights with no ray
  adds 0.25 to 0.56 ms. Holding ray-query code that never runs adds
  0.42 to 0.49 ms to the pass's line (uv-grid 1,141 against 672 µs, the
  helmet 1,150 against 730, `town` 1,907 against 1,420, from the two
  batches), consistent with the occupancy that a live query's state
  costs [ref:nvidia-2022-rtx-best-practices-shadow-rays]
  [ref:dxr-spec-inline-shadow-fast-path]; occupancy itself was not
  measured. Shadow-ray time shows it on uv-grid (1,281 against 811 µs)
  and the helmet (1,280 against 873), but not on `town` (1,493 against
  1,738), whose two figures come from different batches.
  Outside the pass, the frame with rays on spends 0.29 ms (uv-grid) to
  1.3 ms (`town`) more than the frame without, with the pass reduced to
  its bare dispatches.
- **Per ray.** uv-grid traces at most five rays at each of 1.07
  million lit pixels, and its pass reads 0.43 ms more with them than
  with every ray skipped at run time (1,570 against 1,141 µs): at least
  0.08 ns a ray. The helmet traces at most five at each of 0.90 million
  for 4.3 ms (5,471 against 1,150), at least 0.96 ns a ray. Both scenes
  light every lit pixel with the sun and four point lights, so the
  difference is traversal: 257,000 triangles against two.
- **Per light.** On the helmet and uv-grid, where every lit pixel
  reaches four point lights, each light costs about the same: 0.69 to
  0.88 ms on the helmet and 0.07 to 0.09 ms on uv-grid, with the ray
  code held throughout. `town`'s surfaces reach 0.64 point lights each on
  average, and capping them at 4 a surface changes nothing. Capping
  them at 1 saves 1.2 ms: most surfaces reach one lamp or none, so the
  rays past a surface's first cost 1.2 ms in all. The sun costs 5.7 ms
  on `town`, with its edge rays, against 4.1 ms for all 1,024 lamps.
- **The dynamic TLAS.** Its queries cost 1.7 ms on `town`, 0.86 ms of
  that for the lamps' rays, which run below 20 m while the flocks fly
  from 26 m to 59 m. Each ray that misses the static TLAS starts a
  second traversal from the dynamic TLAS's root. Apple keeps the split
  under one root through instancing levels, at "minor" trace cost
  [ref:apple-wwdc23-10128-static-dynamic-split], which wgpu does not
  offer.
- **Edge rays and MSAA samples.** The edge rays retrace the sun from a
  pixel's other samples only where its verdict differs from a
  neighbour's, listed for a dispatch of their own as per-sample
  shading is [ref:lauritzen-2010-per-sample-scheduling]; Crassin et
  al. warn that shadowing per sample scales with the samples and the
  lights [ref:crassin-2015-agaa-shadowing]. They cost 2.2 ms on `town`
  within the sun's 5.7 ms, the price of the edges that the owner chose
  with a 4 ms ceiling (#358).
- **Against the source.** Boksansky, Wimmer and Bittner measure 1.3 to
  1.6 ms for one light and 3.4 to 4.8 ms for four at 1920×1080 on a
  GeForce RTX 2080 Ti [ref:boksansky-2019-rt-hard-shadows], a different
  GPU. `town`'s sun alone, 5.7 ms at
  2560×1440 with its edge rays, is 2 to 2.5 times their one light
  scaled (2.3 to 2.8 ms).

### One ray-query call a surface

`build_shadow_mask` and `trace_second_surfaces` traced each surface's
directional and point lights in two loops. Each loop inlined
`shadowed`, and `shadowed` held six ray-query initialisations: an
any-hit query, a nearest-hit query and a second any-hit query, each
against both TLAS. Now one function, `first_intersection`, holds the
static and the dynamic query, with its flags passed in, as AMD advises
one live ray query a shader [ref:amd-rdna-guide-ray-tracing]. `shadowed` calls it once, or
twice in a loop for a ray that starts inside a smooth mesh's
terminator. One loop walks the directional lights and then the point
lights. The rays, their flags and their order are as before. Metal's
compiler rounds the reshaped code differently, though: `scripts/gallery.sh`
against `main` finds 18 single pixels that differ by more than 1, at
most 8 in an image, in `town`'s frames 900, 1,200 and 1,800 and in
pbr-sweep, the same in a second run, and no other scene changes. Each
is a lone pixel, as a grazing ray's flipped verdict would leave, and the
two looked at in 8x crops sit on a shading edge; the Goldens pass.

Three interleaved A/B runs of three rounds each, against `main`'s
binary, shadow-ray time in µs (medians of the rounds). Load average was
1.9 to 2.8 over the first run; the second started at 9.9, then read 2.2
to 4.9; the third read 2.5 to 5.1 but for one run at 11.5.

| Scene | A/B 1 | A/B 2 | A/B 3 |
|---|---:|---:|---:|
| `town` | 11,273 → 10,301 | 11,358 → 10,568 | 11,325 → 10,395 |
| helmet | 5,543 → 4,710 | 5,721 → 4,776 | 5,700 → 4,828 |
| uv-grid | 1,688 → 1,807 | 1,702 → 1,815 | 1,721 → 1,813 |
| shadows | | 3,075 → 3,045 | |
| far-coplanar | | 1,582 → 1,597 | |

uv-grid's rays are the cheapest of any scene, and they pay for the
flags arriving as a value rather than a constant: a variant that kept
the merged light loop but called `first_intersection` with constant
flags from three places matched `main` on uv-grid (1,723 against
1,721), but `town` got slower (11,523) and the helmet kept less than
half the gain (5,348). uv-grid's shadow-ray time grows 5 to 7% and its
shadow mask pass line 6%, past `praxis compare`'s 5% warning against
the Baselines before this change, which are re-recorded with it. ADR
0019's gate runs with rays off, so it does not move: 774 against 777 µs
on uv-grid, 1,236 against 1,235 on the helmet, 8,946 against 8,844 on
`town`, the medians of all nine rounds.

A last interleaved round on the committed shader, two rounds against
`main` at `a5280dce`, read 4,720 against 5,798 µs on the helmet, 1,807
against 1,674 on uv-grid and 10,271 against 12,683 on `town`, whose
`main` rounds (14,133 and 11,233) ran at load 8 to 21. The M4 Pro
Baselines' shadow-ray, GPU frame and shadow mask pass lines are taken
from one `praxis bench` run of this change, 21:04 to 21:08 at load 8.0
falling to 5.3. Their other lines keep their values: `praxis compare`
read that run's within 5% of them, but the helmet's lit pass (+6.3%)
and the shadows' first culling phase (+6.6%), which this change does
not run. texture-wall's shadow-ray time moves from 42.5 to 279.5 µs:
three interleaved rounds read 200 to 261 µs on `main` and 184 to 333 on
this change, the difference of two GPU frames of 15.2 ms.

### Options for the owner

Each traces the same rays for the same pixels unless it says so.

| Option | `town` saving | Image | Cost and fit |
|---|---:|---|---|
| A. Skip the dynamic TLAS query for a ray that misses the Moving Instances' world bounds | up to 0.86 ms (the lamps' rays) | none | Moving Instances' bounds each frame: on the CPU, in the dynamic TLAS's stage that #405 already finds over its ceiling, or in a GPU reduction; the bent Meshes need bounds that hold every bend |
| B. Fewer, non-overlapping static Instances: merge each cell's static Instances into one BLAS | unmeasured; all three vendors name overlapping Instances as a per-ray cost [ref:apple-wwdc22-10105-instance-overhead] [ref:nvidia-2022-rtx-best-practices-shadow-rays] [ref:amd-rra-instance-overlap] | none | BLAS memory (instancing lost per cell) and builds; streaming's cells are M4's, and AMD warns a merge can also slow traversal, so it needs a prototype |
| C. Trace a surface's lamps in order of their contribution and stop where the rest falls below a display step [ref:ward-1991-adaptive-shadow-testing] [ref:walter-2005-lightcuts-error-bound]; tighten each lamp's range to where it stops contributing [ref:boksansky-2019-distance-light-culling] | at most 1.2 ms (the cap at 1 lamp) | the untested lamps' shadows, within a set bound | a per-surface selection, changes ADR 0020's one ray per light |
| D. Skip tiles a cheap conservative test decides, tracing only listed tiles [ref:amd-fidelityfx-shadow-classifier] | unmeasured | none | needs a conservative occluder bound for the sun, such as a shadow map, which Praxis does not have |
| E. Upstream: naga lowers a single opaque, first-hit query to Metal's intersector rather than an intersection query, which Apple says costs scratch memory traffic [ref:apple-techtalk-111375-intersection-query-cost] | unmeasured | none | a naga change, then a wgpu bump |

Recommendation: ticket A now, as it is exact and small. Take B into
M4's charting with the streaming cells, as the one option sized for
`town`'s gap. Leave C, D and E unless B falls short.

## A render pass's line counted the passes it started beside (2026-10-30, #399)

**Verdict.** The light scenes' lit and tonemapping lines switched
between two modes because a render pass's start timestamp is written
when its vertex stage starts, and on the M4 Pro that stage runs beside
the passes recorded before it while the fragment stage waits for them
[ref:apple-metal-fence-per-stage]. In some frames the lit pass starts
beside the shadow mask pass, and its line then reads the rest of the
mask's time as its own: 2.6–2.8 ms against 480 µs. The tonemapping pass
starts about 12 µs after the lit pass in every frame, so its line always
read most of the lit pass. The share of frames whose lit pass starts
early changes from run to run, 0 to 34% here, so the median falls on
either mode once it nears half. Frame stats now count a render pass from
the latest end of the passes before it, when that is later than its
start. Compute passes keep their own begin-to-end times, as the Froxel
pass runs beside culling phase one and reads none of its output. Five
runs then give the light scenes' pass lines within 2.8% of each other,
inside `compare`'s 5% warning, and the GPU frame and gate do not move,
because both are spans of the raw timestamps. texture-wall's pass lines
still move between two levels for another reason, below, and its
options are #409.

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release build of
`99fdb38e` with a measurement patch, not committed, that wrote each
frame's raw timestamps to a CSV file. `praxis bench <dir> --skip-aliasing
<scene>` at 2560×1440 with 4x MSAA, 248 warm-up and 1,800 measured
frames, rays on, with Chrome and WindowServer loading the GPU 12–27%.
The pass times below come from those timestamps: each line's median and
p95 in µs, first as Frame stats read them on `main` (end less start),
then counted from the latest end of the passes before the pass.

| Run | Scene | Lit pass starts inside the shadow mask pass | Lit, own | Lit, counted | Tonemapping, own | Tonemapping, counted |
|---|---|---:|---:|---:|---:|---:|
| 1 | pbr-sweep | 0 of 1,800 | 479 / 498 | 479 / 498 | 785 / 810 | 319 / 322 |
| 2 | pbr-sweep | 212 (11.8%) | 481 / 2,663 | 477 / 497 | 788 / 2,973 | 319 / 322 |
| 1 | normal-tangent | 453 (25.2%) | 490 / 2,735 | 485 / 499 | 796 / 3,041 | 319 / 322 |
| 2 | normal-tangent | 606 (33.7%) | 491 / 2,839 | 486 / 498 | 797 / 3,205 | 319 / 323 |

- **One timestamp names the mode.** In each run, the frames whose lit
  pass starts before the shadow mask pass ends are exactly the frames
  whose lit pass reads over 1.5 ms. The Froxel pass starts after culling
  phase one ends in those frames and beside it in the others, in 1,790
  to 1,800 of each run's 1,800: Metal schedules the whole frame
  differently, and why is the driver's.
- **The shadow mask pass and the GPU frame** stay within 16 µs of their
  Baselines in every run.
- **`town`**, from #382's timestamps of `main` with two frames in
  flight, counted the same way: the lit pass reads 3,753 / 4,095 µs
  against 16,032 / 16,529, and tonemapping 324 / 370 against
  6,512 / 6,923. Every other pass is unchanged. So the lit and
  tonemapping figures of earlier sections, #211's comparison with their
  sources among them, are pre-#399 figures that include time beside
  the shadow mask pass.
- **Why not every pass.** A rule that also counted a compute pass from
  the end of the passes before it whenever it outlasts them would cut
  `town`'s Froxel line from about 590 to about 70 µs in the 13–15% of
  frames whose Froxel pass ends after culling phase one, the same flip
  in another pass.

Five runs of `praxis bench <dir> --skip-aliasing pbr-sweep
normal-tangent texture-wall` after the change, same machine and load.
Run 3 as `praxis report` prints it, against the re-recorded Baselines:

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| pbr-sweep GPU frame | 3,863.5 | 4,282.7 | 3,840.4 | 4,035.7 | +23.0 | 1.006 | 12,000.0 |
| pbr-sweep `lit_pass` | 480.2 | 523.8 | 473.8 | 493.0 | +6.4 | 1.013 | — |
| pbr-sweep `tonemapping_pass` | 318.8 | 325.0 | 321.2 | 322.1 | -2.4 | 0.993 | — |
| normal-tangent GPU frame | 3,520.0 | 3,761.2 | 3,518.3 | 3,747.0 | +1.7 | 1.000 | 12,000.0 |
| normal-tangent `lit_pass` | 488.2 | 509.5 | 489.0 | 504.9 | -0.8 | 0.998 | — |
| normal-tangent `tonemapping_pass` | 318.8 | 321.7 | 319.0 | 323.2 | -0.2 | 0.999 | — |

Across the five runs, each pass line of pbr-sweep and normal-tangent
stays within 1.028 times its lowest median (the depth pyramid on
normal-tangent is the widest); `lit_pass` within 1.004 and 1.013,
`tonemapping_pass` within 1.001. `praxis compare` fails no pass row in
any run. It fails texture-wall's shadow-ray time in every run, as it did
before the change (159 and 177 µs against 42.5), and CPU lines under
the machine's load; neither depends on pass attribution.

### texture-wall: two levels of every pass

texture-wall's passes read one of two levels, about 1.8 times apart,
while its GPU frame reads 15.0–15.6 ms in both. Five runs on `main`,
then five after the change, `depth_prepass` medians in µs:

| Build | Runs |
|---|---|
| `main` | 725.8, 726.0, 398.6, 725.6, 398.7 |
| #399 | 725.8, 398.8, 725.5, 725.5, 725.6 |

- **Every pass scales together**: culling 1.66–1.68 times, the depth
  pre-pass, depth pyramid, shadow mask and tonemapping 1.79–1.83, the lit
  pass 1.69, compute and render alike, as a lower GPU clock would make them. Metal exposes
  no GPU clock and `powermetrics` needs root, so the clock was not read.
- **The level also changes within a run.** In a fast run, the 82 frames
  whose lit pass reads over 1.5 ms read the slow level for every pass. With rays
  off, the depth pyramid reads the slow level while the depth pre-pass
  reads the fast one.
- **An untimed stretch paces the frame.** In every frame, rays on or
  off, the lit pass starts 12.2–13.5 ms after culling phase one. With
  rays off, nothing timed runs between culling phase two's end (1.4 ms)
  and the lit pass's start. That stretch belongs to its 10,000 Textures
  (#183), and the passes' level barely moves the frame.
- **The Baseline holds the slow level**, so a run on either level passes
  `compare`, and a regression of up to 1.8 times in one of its passes
  goes unflagged. How to judge its pass rows is #409.

### Baselines

The M4 Pro Baselines' four render-pass lines (`depth_prepass`,
`phase_two_depth_pass`, `lit_pass`, `tonemapping_pass`) are re-recorded
from one `praxis bench` run of every scene on this branch. Every other
line stays as #211 recorded it on a quiet machine: the change moves no
other line, and this machine's load moved CPU lines by up to 22% run to
run. `tonemapping_pass` now reads 319–321 µs on every scene but
texture-wall (1,478), and `lit_pass` 3,769 µs on `town` against 15,997,
343 on the orrery against 2,110 and 735 on the helmet against 808.

## M3 exit (2026-10-30, #211)

**Verdict.** `town` breaches three Budgets: its GPU frame is 20.1 ms
against 12, 11.0 ms of it shadow rays; its CPU frame is over 6 ms at
p95; and ADR 0019's gate is 8.6 ms against 6, as accepted on #297.
texture-wall's gate, 13.6 ms, is the largest breach of any scene. The
owner kept every Budget, so the top three offenders are ticketed: #183,
#404 and #405. The depth pyramid and culling cost over twice their
sources' published times (#406).

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release build of
`515c33be`. `praxis bench <dir>` at 2560×1440 with 4x MSAA, 248 warm-up
and 1,800 measured frames per scene, each scene with shadow rays on and
then off. Four runs back to back, 16:48 to 17:03. Load average fell
from 23.8 at the first run's start (a verify had just finished) to 2.7
to 4.2 over runs 2 to 4, and 3.2 to 4.2 over run 4, from the browser and
the window server. No other session benched; a peer session ran a
two-second CPU test at 16:52:21, during run 2. GPU
medians agree within 1.2% across runs 1, 3 and 4 for every scene but
texture-wall (4.1%); run 2 reads 11.6% higher on `shadows`, 5.2% on
`town` and 4.5% on the crowd. The Baselines are run 4, the quietest;
the llvmpipe Baselines are CI's
`bench-results` artifact from [run
41084281627](https://github.com/gariasf/praxis/actions/runs/41084281627)
(throwaway PR #402), every counter identical to the ones before.

### Measurements

Four runs of `praxis report <dir>` (medians and p95s; counters identical
on every run), against the Baselines recorded before this exit:

| Scene | CPU median µs | CPU p95 µs | GPU frame median µs | Gate median µs | Shadow-ray time median µs | CPU / GPU Baseline before, µs |
|---|---|---|---|---|---|---|
| crowd | 870.9 / 945.3 / 846.5 / 835.6 | 1,240.7 / 1,321.5 / 1,186.9 / 1,228.5 | 3,796.2 / 3,962.3 / 3,795.1 / 3,793.1 | 1,311.8 / 1,320.6 / 1,307.5 / 1,316.0 | 2,156.9 / 2,293.9 / 2,162.7 / 2,155.1 | 758.0 / 3,784.1 |
| far-coplanar | 247.4 / 245.9 / 227.2 / 226.0 | 490.8 / 495.9 / 480.1 / 484.9 | 2,618.1 / 2,617.8 / 2,606.8 / 2,608.1 | 722.8 / 719.5 / 719.7 / 716.2 | 1,568.3 / 1,574.5 / 1,564.4 / 1,568.8 | 197.2 / 2,634.0 |
| helmet | 284.0 / 360.6 / 267.9 / 250.3 | 516.3 / 568.5 / 537.4 / 537.8 | 7,181.0 / 7,164.8 / 7,156.1 / 7,149.2 | 1,235.0 / 1,238.2 / 1,233.7 / 1,232.2 | 5,619.6 / 5,603.5 / 5,600.3 / 5,595.1 | 188.1 / 7,092.6 |
| minification | 316.4 / 251.3 / 234.4 / 206.5 | 586.5 / 501.7 / 488.0 / 429.9 | 2,979.1 / 2,969.2 / 2,966.0 / 2,961.9 | 887.0 / 886.1 / 884.7 / 882.5 | 1,768.0 / 1,759.3 / 1,758.0 / 1,755.8 | 202.0 / 4,976.1 |
| normal-tangent | 259.9 / 234.2 / 222.6 / 216.7 | 484.1 / 477.7 / 451.0 / 455.2 | 3,528.5 / 3,522.7 / 3,517.8 / 3,518.3 | 811.7 / 814.4 / 812.1 / 810.8 | 2,393.8 / 2,384.8 / 2,382.3 / 2,384.1 | 221.5 / 3,505.3 |
| orrery | 491.1 / 402.2 / 392.4 / 363.7 | 747.0 / 731.7 / 680.7 / 590.1 | 2,896.1 / 2,880.8 / 2,872.7 / 2,861.5 | 647.2 / 640.6 / 637.6 / 634.4 | 1,910.0 / 1,911.5 / 1,905.5 / 1,901.8 | 367.1 / 2,900.9 |
| pbr-sweep | 233.6 / 246.1 / 225.2 / 214.4 | 500.4 / 496.0 / 459.1 / 445.0 | 3,861.4 / 3,849.3 / 3,831.8 / 3,840.4 | 796.0 / 784.9 / 778.5 / 776.9 | 2,721.2 / 2,722.8 / 2,715.9 / 2,723.5 | 184.1 / 3,817.0 |
| shadows | 352.0 / 365.9 / 338.0 / 322.6 | 624.9 / 626.2 / 594.8 / 652.4 | 4,064.5 / 4,532.0 / 4,060.9 / 4,063.4 | 902.7 / 943.5 / 901.3 / 901.8 | 2,839.5 / 3,260.1 / 2,837.3 / 2,838.4 | 264.8 / 4,009.5 |
| texture-wall | 2,389.3 / 2,360.0 / 2,293.1 / 2,097.3 | 3,074.5 / 3,031.9 / 2,836.1 / 2,363.5 | 15,689.5 / 15,581.8 / 15,449.9 / 15,078.0 | 13,843.3 / 13,887.9 / 13,698.2 / 13,551.7 | 349.4 / 223.7 / 289.3 / 42.5 | 2,201.2 / 15,471.6 |
| town | 6,215.1 / 6,392.0 / 6,016.1 / 5,796.1 | 6,919.5 / 6,878.8 / 6,508.5 / 6,209.5 | 20,351.8 / 21,172.9 / 20,197.4 / 20,123.3 | 8,731.5 / 8,840.4 / 8,656.8 / 8,606.2 | 11,122.3 / 11,845.2 / 11,087.5 / 10,973.5 | 5,597.8 / 20,153.0 |
| uv-grid | 241.7 / 238.2 / 212.8 / 209.6 | 433.6 / 496.3 / 446.0 / 427.4 | 2,806.8 / 2,803.8 / 2,797.6 / 2,797.0 | 780.5 / 779.6 / 775.1 / 775.3 | 1,702.7 / 1,701.7 / 1,698.8 / 1,698.9 | 187.4 / 2,789.7 |

Every Baseline before but `town`'s was recorded at #394, before #397
sized the shadow rays' offset and #381 made the bench spin-wait. Against
them, run 4's CPU frames read −5 to +33%, and its GPU frames within 2.5%
but minification's: 2.96 ms against 4.98, its shadow mask pass 1.6 ms
against 3.6, a change from #397 or later whose cause is not measured
here. `town`'s run 4 in full:

**town** on Apple M4 Pro, release, 2560x1440, 1800 measured frames

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 5,796.1 | 6,209.5 | 5,597.8 | 6,149.2 | +198.4 | 1.035 | 6,000.0 |
| Dynamic TLAS, CPU | 1,096.5 | 1,244.9 | 1,018.9 | 1,210.1 | +77.6 | 1.076 | 1,100.0 |
| GPU frame | 20,123.3 | 20,372.5 | 20,153.0 | 20,775.0 | -29.7 | 0.999 | 12,000.0 |
| Gate, rays off | 8,606.2 | 8,909.7 | 8,689.8 | 9,006.1 | -83.5 | 0.990 | 6,000.0 |
| Shadow-ray time | 10,973.5 | 11,587.8 | 10,988.5 | 11,643.7 | -15.0 | 0.999 | — |
| `bend_pass` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | — |
| `blas_refit` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | — |
| `culling_phase_one` | 1,152.6 | 1,206.1 | 1,152.5 | 1,196.5 | +0.2 | 1.000 | — |
| `culling_phase_two` | 175.5 | 190.9 | 171.8 | 185.1 | +3.8 | 1.022 | — |
| `depth_prepass` | 2,124.4 | 2,215.3 | 2,134.6 | 2,225.6 | -10.2 | 0.995 | — |
| `depth_pyramid_pass` | 260.5 | 268.7 | 260.0 | 270.4 | +0.5 | 1.002 | — |
| `dynamic_tlas_build` | 0.0 | 82.1 | 0.0 | 99.1 | +0.0 | — | 1,000.0 |
| `froxel_pass` | 586.7 | 627.7 | 589.2 | 617.7 | -2.5 | 0.996 | — |
| `lit_pass` | 15,997.0 | 16,318.5 | 16,024.0 | 16,388.9 | -27.0 | 0.998 | — |
| `phase_two_depth_pass` | 55.4 | 106.1 | 56.0 | 107.7 | -0.6 | 0.990 | — |
| `shadow_mask_pass` | 12,114.1 | 12,604.1 | 12,161.0 | 12,627.1 | -46.9 | 0.996 | — |
| `tonemapping_pass` | 6,498.2 | 6,741.5 | 6,517.2 | 6,783.2 | -19.0 | 0.997 | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 4 | 4 | +0 | 1.000 | — |
| `instances_drawn` | 137,730 | 137,730 | +0 | 1.000 | — |
| `triangles_drawn` | 5,552,214 | 5,552,214 | +0 | 1.000 | — |
| `instances_overflowed` | 0 | 0 | +0 | — | — |
| `uploaded_bytes` | 2,049,344 | 2,049,344 | +0 | 1.000 | — |
| `gpu_memory_bytes` | 764,009,588 | 764,009,588 | +0 | 1.000 | 8,000,000,000 |
| `arena_bytes_in_use` | 85,280 | 85,280 | +0 | 1.000 | — |
| `heap_allocations` | 10,501 | 10,501 | +0 | 1.000 | — |
| `aliasing_rmse` | 5,142 | 5,142 | +0 | 1.000 | — |
| `aliasing_p99` | 15,988 | 15,988 | +0 | 1.000 | — |

Against ADR 0013's Budgets, which the owner kept at this exit
([decision](https://github.com/gariasf/praxis/issues/211#issuecomment-6528039522)):

- **GPU frame, 12 ms**: `town` 20.1 to 21.2 ms (168 to 176%) and
  texture-wall 15.1 to 15.7 ms (126 to 131%) are over; the helmet's
  7.2 ms is the next, every other scene at most 4.5 ms.
- **ADR 0019's gate, 6 ms**: `town` 8.6 to 8.8 ms, over as accepted
  until LOD ([#297](https://github.com/gariasf/praxis/issues/297));
  texture-wall 13.6 to 13.9 ms, over; every other scene at most 1.3 ms.
- **CPU frame, 6 ms**: `town` 5.80 to 6.39 ms at the median and 6.21 to
  6.92 ms at p95, over at p95 in every run; texture-wall 2.1 to 2.4 ms;
  every other scene at most 0.95 ms.
- **Dynamic TLAS, 1.1 ms CPU and 1.0 ms GPU**: `town`'s CPU reads 1.10
  to 1.26 ms, over in three of the four runs; its build reads 0 at the
  median (hidden behind the previous frame, #207).
- **GPU memory, 8 GB**: `town` 764,009,588 B (9.6%), every other scene
  663 to 684 MB.

### The crowd's two forms

`praxis bench <dir> --crowd-form <form> crowd`, three rounds alternating
the forms, update CPU and GPU as #209 defined them:

| Form, round | Update CPU µs, median (p95) | Refit GPU µs, median (p95) | Build GPU µs, median (p95) | Update GPU µs, medians |
|---|---|---|---|---|
| Proxies 1 | 308.1 (457.1) | — | 0.0 (0.0) | 0.0 |
| Proxies 2 | 297.9 (342.8) | — | 0.0 (0.0) | 0.0 |
| Proxies 3 | 300.0 (328.0) | — | 0.0 (0.0) | 0.0 |
| Refit 1 | 407.9 (574.0) | 1,099.2 (1,142.0) | 255.2 (292.3) | 1,354.5 |
| Refit 2 | 404.3 (568.5) | 1,101.7 (1,153.3) | 254.9 (296.0) | 1,356.5 |
| Refit 3 | 383.4 (440.3) | 1,102.0 (1,153.0) | 259.4 (302.5) | 1,361.5 |

The proxies, the form the owner picked for M7, stay within 0.5 ms of CPU
and 1.0 ms of GPU: their build now runs beside the previous frame and
reads 0 at the median. The refit stays over the GPU Budget, as at #209.

### Passes against their sources

Each timed pass on `town` (run 4) beside its source's published cost,
from the `cost` quotes in `docs/references.toml`. Costs that scale with
pixels are scaled from 1920×1080 to 2560×1440 (×1.78). Culling scales
with Instances, 250,000 objects in the source against `town`'s 254,256,
and the Froxel lists with Froxels times lights, 19.7 million against
18.9 million, so those two are compared unscaled. No source measured on
an Apple GPU was found.

| Pass | `town` µs | Source, hardware | Source µs, scaled | Ratio |
|---|---:|---|---:|---:|
| Depth pyramid | 260.5 | Haar and Aaltonen 2015, Xbox One, from the HTILE min/max buffer [ref:haar-aaltonen-benchmark] | 107 | 2.4 |
| Depth pyramid | 260.5 | Wihlidal 2016, Xbox One, 16×16 depth tiles from 16-bit depth [ref:wihlidal-2016-depth-tile-reduction] | 73 | 3.6 |
| Culling, both phases | 1,328.1 | Haar and Aaltonen 2015, Xbox One, 250,000 objects, with cluster culling [ref:haar-aaltonen-benchmark] | 670 | 2.0 |
| Culling, both phases | 1,328.1 | the same, object culling and LOD alone | 540 | 2.5 |
| Froxel lists | 586.7 | Drobot 2017, PS4, 256 lights in 76,800 clusters [ref:drobot-2017-cluster-rasterizer] | 320 to 900 | 0.7 to 1.8 |
| Shadow mask | 12,114.1 | Boksansky et al. 2019, RTX 2080 Ti, four lights [ref:boksansky-2019-rt-hard-shadows] | 6,040 to 8,530 | 1.4 to 2.004 |
| Lit pass | 15,997.0 | Drobot 2017, PS4, forward+ opaque, 256 lights [ref:drobot-2017-forward-plus-opaque] | 8,180 to 10,130 | 1.6 to 2.0 |
| Tonemapping | 6,498.2 | none found with a stated resolution | — | — |
| Depth pre-pass | 2,124.4 | none cited | — | — |
| Phase two's depth pass | 55.4 | none cited | — | — |
| Dynamic TLAS build | 0.0 | none cited; its ceiling is 1.0 ms | — | — |
| Bend pass, BLAS refit | 0.0 | do not run on `town`; the crowd's are under Crowd | — | — |

- **The depth pyramid is over twice its source** by either figure, so
  it is an offender whatever its Budget (#406). Wihlidal's figure is one
  reduction to 16×16 tiles, not a whole pyramid, so 3.6 overstates the
  gap. It reads 195 to 264 µs on every scene but texture-wall (791 µs,
  #399): the cost is the pass's, not `town`'s.
- **Culling is at or over twice its source**: 1.98 times the figure with
  Haar's cluster culling, 2.46 times their object culling alone. `town`'s
  phases write each drawn Meshlet's records and indices but cull no
  cluster (#347), so it lies between the two; it is counted as an
  offender with the depth pyramid (#406).
- **The shadow mask is over twice its source by one figure**: 2.004
  times the Resort scene's four lights. `town` traces a ray for each
  light whose range reaches the pixel, up to the mask's 59 (#260),
  against the source's four, so the ratio is an upper bound; it is an
  offender by its Budget share either way (#404). On
  the helmet, the sun and four lights, it reads 5,408 µs (0.6 to 0.9
  times); on the crowd, the sun alone, 1,961 µs against 2,310 to 2,840
  (0.7 to 0.85 times).
- **The lit and tonemapping passes' timestamps** include time beside the
  shadow mask pass: tonemapping's own cost is about 0.3 ms (#352), and
  the lit pass read 5.15 ms with rays off at #207.

### Where the cost goes

- **GPU.** Shadow rays: on `town` the frame with rays on takes 11.0 ms
  more than with them off, 91% of the 12 ms Budget; the shadow mask pass
  reads 12.1 ms. It also has a fixed cost: 1.56 ms on uv-grid, two
  triangles. On the helmet it read 2.5 ms when the mask landed (#260)
  and 5.4 ms now, after the two-surface mask (#357) and the sun's
  per-sample edge rays (#358). With rays off, `town`'s gate is the depth
  pre-pass (2.1 ms), the culling phases (1.3 ms), the Froxel lists
  (0.6 ms), the depth pyramid (0.26 ms) and the lit pass. texture-wall's
  gate spans 13.6 ms while its passes from culling to the lit pass time
  2.8 ms between them (#183, #399).
- **CPU, `town`.** `sample` at 1 ms over one `praxis bench town` run
  (rays on, then off), 12,734 main-thread samples: 5,808 (46%) wait in
  `Device::poll`. Of the 6,926 others, `FramePreparer::prepare`, the walk
  of `town`'s 254,256 Instances, is 2,086; `Queue::submit` 1,535, of which
  wgpu-core's `set_acceleration_structure_dependencies` is 1,044 (#352);
  `CommandEncoder::finish` 1,070, 451 of them the dynamic TLAS's encoder;
  the fixed schedule's Ticks 608.
- **CPU, the small scenes.** `sample` over helmet and pbr-sweep, 9,425
  main-thread samples: 7,388 in `Device::poll`. Of the 2,037 others,
  `Renderer::render` is 531, `CommandEncoder::finish` 407 of them. The
  frame's upload copies are 62 samples here and 70 on `town` (#140,
  closed).
- **GPU memory.** `town`'s 764 MB is 9.6% of 8 GB; every other scene
  holds 663 to 684 MB at 2560×1440.

### Top three offenders

Ranked by their share of the Budget each breaches:

1. **texture-wall's gate, 13.6 ms, and GPU frame, 15.1 ms** (226% and
   126% of their Budgets), its 10,005 bound Textures. #183
2. **`town`'s GPU frame, 20.1 ms** (168% of the 12 ms Budget), 11.0 ms
   of it shadow rays, with a fixed cost of 1.4 to 1.6 ms on the simplest
   scenes. #404
3. **`town`'s CPU frame, 6.21 to 6.92 ms at p95** (103 to 115% of 6 ms
   in every run), with the dynamic TLAS's CPU over its 1.1 ms ceiling in
   three of four runs: the Instance walk in prepare and wgpu-core's TLAS
   bookkeeping. #405

Also offenders, by their sources: the depth pyramid, 2.4 to 3.6 times
its source's scaled cost, and culling, 1.98 to 2.46 times (#406). #140,
M1's upload copies, misses the top three and is closed. `town`'s gate,
143% of 6 ms, stays over by the owner's decision on #297.

### Also seen

- Run to run, the light scenes' pass lines switch between two modes
  while the GPU frame holds: texture-wall's passes read 70 to 138% over
  their Baseline in all four runs, minification's shadow mask pass 55%
  under it (#399).
- The visual audit's gallery found one new defect: dark specks at the
  crowd's sunlit silhouettes, eleven pixels in both forms (#407).

## Queued `town` frames keep their pass overlap since the shadow mask pass dispatched indirectly (2026-10-30, #382)

**Verdict.** The overlap #378 saw queued frames lose is back on `main`,
so the engine needs no fix. Since #387 (`7442717a`), with two frames in
flight, `town`'s lit pass starts beside the shadow mask pass again, the
Froxel pass starts beside culling phase one, and the passes span what
they span with one frame in flight. The lever is the shadow mask pass's
indirect dispatches. Bisected, #385 loses the overlap and #387, which
gave the pass its first indirect dispatch, keeps it (#394 added the
second). On `main`, leaving both out brings the loss back. One indirect dispatch of
zero workgroups at the pass's start, ahead of its direct dispatches,
removes it again. The mechanism is the driver's, and Apple does not
document it. Tracking promises that commands after a write wait for it
[ref:apple-metal-hazard-tracking-tracked], and Apple GPUs can make a
pass's fragment stage wait while its vertex stage runs
[ref:apple-metal-fence-per-stage]. Neither says when Metal runs a
command buffer's passes one after another. That happens here when the
shadow mask pass traces rays from direct dispatches only and its command
buffer is committed behind one still running. An empty shadow mask pass
keeps the overlap and a doubled one loses it, so the cause is not the
pass's length. The per-submission residency set is not the cause either
[ref:wgpu-hal-30-metal-residency-per-submit]: with one set held by the
queue for its lifetime, the loss stays. Submitting the dynamic TLAS's
build together with the frame's commands changes nothing. A shadow mask
pass with only direct dispatches would cost `town`'s queued frames 0.4
to 1.2 ms of GPU frame again, and a bench against the Baseline shows it.

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release builds of
`17a7fa91` (#383), `14e3446e` (#385), `7442717a` (#387) and `7f318ffd`
(`main`), each with a measurement patch, not committed. The patch logged
each pass's median start and end over all but the first 100 frames,
counted from culling phase one's start, plus each frame's raw
timestamps. Environment variables set the frames in flight and switched
the shadow mask pass's variants. One variant also patched wgpu-hal
30.0.1 to keep one residency set on the queue for its lifetime instead
of a new one per submission. `praxis bench <dir> town` at 2560×1440 with
4x MSAA and shadow rays on, 1,800 measured frames, no other GPU work.
The machine ran slower over the afternoon, so compare rows only within
one table. CPU frame, GPU frame and `dynamic_tlas_build` are medians as
`praxis report` prints them, in µs. Each pass column gives the median
start and end, in µs, from culling phase one's start.

| Build | In flight | CPU frame | GPU frame | `dynamic_tlas_build` | Froxel | Shadow mask | Lit | Tonemapping |
|---|---:|---:|---:|---:|---|---|---|---|
| #383 | 2 | 7,009.0 | 16,512.2 | 0.0 | 886–1,271 | 3,896–10,496 | 10,509–16,163 | 12,503–16,488 |
| #383 | 1 | 4,922.2 | 16,019.6 | 668.0 | 3–589 | 3,783–10,814 | 3,752–15,007 | 11,344–15,328 |
| #385 | 2 | 7,378.8 | 16,506.0 | 0.0 | 886–1,272 | 3,900–10,508 | 10,521–16,164 | 12,504–16,487 |
| #387 | 2 | 7,615.8 | 17,615.1 | 0.0 | −3–569 | 3,772–13,407 | 3,757–17,252 | 13,514–17,580 |
| `main` | 2 | 8,063.7 | 20,192.5 | 0.0 | −3–581 | 3,782–15,981 | 3,755–19,836 | 13,605–20,161 |
| `main` | 1 | 4,903.4 | 20,702.8 | 670.6 | 56–601 | 3,784–15,984 | 3,752–19,678 | 13,596–20,002 |

#383 reproduces #378's figures: 16.5 ms queued against 15.3 ms of passes
with one frame in flight. On `main`, the queued GPU frame, 20,192.5, is
the one-in-flight frame less its exposed build (20,032), within noise.

The variants of `main`'s shadow mask pass, two interleaved rounds. *Direct
only* leaves out the two indirect dispatches and the work they launch.
*Zero indirect* is *direct only* with an indirect dispatch of zero
workgroups added at the pass's start. Its arguments come from the buffer
the pass clears before it, and it launches nothing.

| Variant | Round | In flight | CPU frame | GPU frame | `dynamic_tlas_build` | Froxel | Shadow mask | Lit | Tonemapping |
|---|---:|---:|---:|---:|---:|---|---|---|---|
| Direct only | 1 | 2 | 7,365.2 | 17,491.7 | 0.0 | 892–1,277 | 3,913–11,155 | 11,167–17,123 | 13,157–17,456 |
| Zero indirect | 1 | 2 | 7,207.1 | 17,114.6 | 0.0 | −3–583 | 3,848–11,853 | 3,879–16,625 | 11,928–17,011 |
| Direct only | 1 | 1 | 5,061.3 | 17,305.2 | 674.0 | 55–590 | 3,792–11,562 | 3,758–16,073 | 11,836–16,474 |
| Direct only | 2 | 2 | 7,136.3 | 17,563.5 | 0.0 | 893–1,278 | 3,915–11,218 | 11,232–17,192 | 13,239–17,525 |
| Zero indirect | 2 | 2 | 6,940.3 | 16,431.6 | 0.0 | −3–598 | 3,801–11,536 | 3,784–16,059 | 11,958–16,397 |
| Direct only | 2 | 1 | 5,035.9 | 16,958.4 | 670.9 | 55–582 | 3,765–11,508 | 3,734–15,914 | 11,849–16,253 |

Single runs, two frames in flight, all losing the overlap: `main` with
only the pass's first dispatch (GPU frame 17,212.2), *direct only* with
the queue's residency set (17,433.4) and with the build submitted with
the frame (17,408.0), and #385 with its one dispatch issued twice
(23,127.8). #385 with no workgroups dispatched keeps it: Froxel −3–581,
lit 3,749–9,798.

The raw timestamps show the whole queued frame running one pass after
another, and not only the lit pass waiting. In *direct only*, the lit
pass starts 11 µs after the shadow mask pass ends. The next frame's
build, which waits for this frame's shadow mask pass, runs beside the
lit pass's start. That next frame's culling then waits for this frame's
tonemapping, and its Froxel pass waits for culling. With the overlap
kept, the lit pass's vertex work runs beside the shadow mask pass, and
the build ends near the lit pass's end. With rays off there is no shadow
mask pass, and #378 measured queued frames keeping the overlap.

## `town`'s CPU stages slow because the P-core clocks down while the main thread waits (2026-10-30, #381)

**Verdict.** When `town` is GPU-bound, the main thread blocks for the
oldest frame in flight in every frame, and in most runs its P-core then
runs about 30% slower for the whole frame. The clock is read through
work, not a counter (`powermetrics` needs root): a fixed chain of 2,000
dependent multiply-adds, timed on the main thread at three points of
each frame, takes 2.58–3.04 µs in those runs against 1.92–2.04 µs
spin-waiting, and `prepare_frame` slows by the same ratio, 1,812 against
1,307 µs. The thread never runs on an efficiency core and stays on one
P-core cluster across the wait, so neither migration nor a cold L2
explains it; its priority does not move it either. Spin-waiting for the
frame instead of blocking keeps the clock up: the CPU frame reads
4,977 µs against 6,877–7,279, the dynamic TLAS's CPU line 931–937 µs
against 1,302–1,326, under its 1.1 ms ceiling, and the GPU frame moves
by at most 1.4%. Apple asks games not to busy-wait [ref:apple-tech-talk-110147-busy-waits],
so the engine's runtime keeps blocking, and the owner chose option 1
below: `praxis bench` alone spin-waits.

Apple M4 Pro (8 performance and 4 efficiency cores), macOS 27.0.1, wgpu
30.0.1 on Metal, release build of `102b22c3` with a measurement patch,
not committed. The patch timed the multiply-add chain at each frame's
start (after the wait), after `update` and after `render`, read the core
number from `TPIDR_EL0`'s low bits (cores 0–3 are the efficiency
cluster, 4–7 and 8–11 the two performance clusters, mapped by timing the
same chain on each), logged each CPU stage's median, and switched by
environment variable the frames in flight, a spin-wait (`device.poll`
with a zero timeout, in a loop) and a priority of 45 set with
`pthread_setschedparam` under the default time-sharing policy. `praxis bench <dir> --skip-aliasing town` at
2560×1440 with 4x MSAA, 1,800 measured frames, no other GPU work; the
patch's core counts span every frame after the first 100, 1,948 a run.

Medians in µs, from the patch's log rather than `praxis report`, whose
tables hold neither stages nor the chain; the chain's medians with rays
on, at the frame's start, after `update` and after `render`. Runs in the
order they ran; the default's first run was the only one with a fast
clock.

| Run | CPU frame | Dynamic TLAS, CPU | GPU frame | `prepare_frame` | `propagate_world_transforms` | `encode` | `submit` | Chain: start, update, render |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| Default (2 in flight) | 5,985.8 | 1,136.2 | 20,155.0 | 1,324.5 | 362.2 | 283.2 | 2,710.1 | 2.04, 1.96, 2.04 |
| 1 in flight | 4,913.6 | 911.5 | 20,630.5 | 1,285.7 | 344.4 | 166.7 | 2,129.0 | 2.46, 1.92, 2.00 |
| Default | 7,278.5 | 1,325.5 | 20,150.9 | 1,804.0 | 481.7 | 242.1 | 3,311.2 | 2.67, 2.58, 2.92 |
| Spin-wait | 4,977.5 | 937.2 | 20,119.1 | 1,307.2 | 351.5 | 201.2 | 2,072.5 | 2.00, 1.92, 2.00 |
| Priority 45 | 6,904.2 | 1,301.9 | 20,135.0 | 1,811.1 | 488.2 | 244.7 | 2,935.3 | 2.75, 2.75, 3.17 |
| Default | 6,877.1 | 1,301.7 | 20,136.3 | 1,812.3 | 483.4 | 244.9 | 2,918.0 | 2.75, 2.75, 3.04 |
| 1 in flight | 5,347.5 | 963.3 | 20,638.9 | 1,259.7 | 310.1 | 165.2 | 2,568.9 | 2.25, 1.96, 2.04 |
| Spin-wait | 4,976.1 | 931.0 | 20,140.8 | 1,306.5 | 353.2 | 204.7 | 2,066.9 | 2.00, 1.96, 2.04 |
| Default | 6,907.4 | 1,307.8 | 20,140.1 | 1,812.4 | 482.7 | 245.2 | 2,928.0 | 2.75, 2.75, 3.00 |

In every run, at most 1 of 1,948 frames started on an efficiency core,
and at most 14 changed performance cluster between one frame's `render`
and the next frame's start; with one exception, a few hundred changed
core within a cluster, which shares its L2. With rays off, the GPU frame
is shorter but still longer than the CPU frame, and the default runs'
chain reads 2.54 µs at the start, the fast first run's included.

What the measurements say about the cause:

- **The clock, not the work.** The chain does the same work in every
  frame, so its time is the core's speed. Where it reads 2.75 µs, 1.4
  times 1.96, `prepare_frame` and `propagate_world_transforms` read 1.39
  and 1.37 times their spin-wait medians.
- **Blocking, not the GPU's load.** Spin-waiting leaves the GPU exactly
  as busy, two frames in flight and the GPU frame within 40 µs, yet the
  clock stays fast. Apple names the kernel's scheduler as what sets a
  cluster's frequency from its workload
  [ref:apple-tech-talk-110147-cluster-frequency]; how it weighs a thread
  that blocks for most of each frame is not published.
- **The wait's length and timing.** With one frame in flight the thread
  blocks for longer, about 15 ms a frame, and the chain is slow at the
  frame's start, 2.25–2.46 µs, but fast again by the end of `update`.
  The frame then runs while the GPU is idle. With two in flight it runs
  beside the GPU and stays slow throughout.
- **Not priority or QoS.** The main thread of a process started from a
  shell reports `QOS_CLASS_USER_INTERACTIVE` (`0x21`), but the kernel
  holds it at priority 31, the default class's, whichever class a QoS
  call asks for. The kernel lowers the interactive and initiated classes
  to the legacy one for an app-type task without an application role
  [ref:xnu-task-policy-role-qos-ceiling], which fits what was measured;
  the patch did not read the process's app type or role. Priority 45,
  what Apple recommends for a game's render thread
  [ref:apple-wwdc18-612-render-thread-priority], took effect under the
  default time-sharing policy and left the stages as slow as the
  default; the round-robin policy, whose priority does not decay, was
  not tried.
- **Not migration.** No frame ran on an efficiency core and the
  performance cluster held across the wait, so the L2 stays warm.
- **Not all of it.** The first default run kept a fast clock, yet its
  `encode` and `submit` read 283 and 2,710 µs against spin-waiting's
  201–205 and 2,067–2,073, and its CPU frame 5,986. Blocking costs
  about 1 ms there that the clock does not explain; this note does not
  find what.

The ticket's stages from #378 do not all reproduce: `prepare_frame` now
reads 1,812 µs blocked and 1,307 spin-waiting. #358's sun shadow edge
rays have since made the GPU frame 20.1 ms, so the thread waits longer
in each frame.

**The options the owner chose from.** M3's CPU ceilings, the CPU frame's 6 ms and
the dynamic TLAS's 1.1 ms, measure the main thread's cost when it is on
the critical path. Blocked behind a GPU-bound frame it is not, and its
clock falls because it has the time.

1. **The bench spin-waits; the window keeps blocking** (recommended).
   `praxis bench` waits for the oldest frame by polling with a zero
   timeout, so the CPU lines read the work at full clock, and the GPU
   lines are unchanged. No extra run; the busy-wait burns a P-core for
   the bench's few minutes only. The bench then differs from the window
   in how it waits, which the window's own CPU times would show.
2. **The bench reads the CPU lines from a run with one frame in flight.**
   The thread blocks, as Apple asks, and the lines read 4,914–5,348 µs
   and 912–963 µs; the clock's ramp after each wait stays in them. The
   GPU lines keep today's run, so each scene runs a third time.
3. **The engine spin-waits everywhere.** The window's CPU frame would
   read the same as the bench's, but Apple asks games not to: a
   busy-wait holds a P-core and spends the thermal headroom a GPU-bound
   frame needs [ref:apple-tech-talk-110147-busy-waits].
4. **No change.** The CPU lines keep reading the slowed clock whenever
   `town` is GPU-bound, over both ceilings, for a cost the frame does
   not pay.

**After the change.** `town`'s Baseline is re-recorded from the build
that spin-waits, the only Baseline whose CPU lines moved: every other
scene's compare passes on its CPU lines. By the time it was recorded the
machine read slower than an hour before, the blocking build's CPU frame
8,180 µs where it had read 6,877–7,279. The re-recorded run against the
previous Baseline:

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 5,597.8 | 6,149.2 | 8,223.8 | 9,431.9 | -2,626.1 | 0.681 | 6,000.0 |
| Dynamic TLAS, CPU | 1,018.9 | 1,210.1 | 1,566.1 | 1,792.4 | -547.2 | 0.651 | 1,100.0 |
| GPU frame | 20,153.0 | 20,775.0 | 20,489.1 | 21,334.4 | -336.1 | 0.984 | 12,000.0 |
| Gate, rays off | 8,689.8 | 9,006.1 | 8,686.5 | 8,968.4 | +3.2 | 1.000 | 6,000.0 |
| Shadow-ray time | 10,988.5 | 11,643.7 | 11,271.8 | 12,273.6 | -283.2 | 0.975 | — |

The dynamic TLAS's CPU line is under its 1.1 ms ceiling at the median,
not at the p95; the CPU frame is under 6 ms at the median but not at the
p95. Two full runs read 5,529–5,598 µs for the CPU frame, against
4,977–5,127 in the `--skip-aliasing` pair below, so the Baseline is the
looser of the two forms.

Against the runs with one frame in flight, the GPU idle at each frame's
start, spin-waiting's stages read `prepare_frame` 1,307 against
1,260–1,286 µs, within 4%, but `propagate_world_transforms` up to 14%
over and `encode` 21–24% over; `submit`, 2,067–2,073 against
2,129–2,569, is under. The ticket's 5% holds for `prepare_frame` and
`submit`, not for `propagate_world_transforms` or `encode`. CI's lavapipe keeps the blocking wait: its rasterizer runs
on the CPU, where a spinning main thread would take a core from it. The
RTX 2070 SUPER's Baselines were recorded blocking. An
interleaved pair from one build, switched between the two waits by a
patch, `--skip-aliasing`, each against the previous Baseline; blocking:

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 8,180.0 | 9,032.9 | 8,223.8 | 9,431.9 | -43.8 | 0.995 | 6,000.0 |
| Dynamic TLAS, CPU | 1,567.8 | 1,752.5 | 1,566.1 | 1,792.4 | +1.7 | 1.001 | 1,100.0 |
| GPU frame | 20,473.1 | 21,132.3 | 20,489.1 | 21,334.4 | -16.0 | 0.999 | 12,000.0 |
| Gate, rays off | 8,798.4 | 9,263.1 | 8,686.5 | 8,968.4 | +111.9 | 1.013 | 6,000.0 |
| Shadow-ray time | 11,167.7 | 12,096.2 | 11,271.8 | 12,273.6 | -104.1 | 0.991 | — |

Spin-waiting:

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 4,977.2 | 5,392.8 | 8,223.8 | 9,431.9 | -3,246.6 | 0.605 | 6,000.0 |
| Dynamic TLAS, CPU | 943.5 | 1,057.5 | 1,566.1 | 1,792.4 | -622.6 | 0.602 | 1,100.0 |
| GPU frame | 20,759.1 | 21,608.2 | 20,489.1 | 21,334.4 | +270.0 | 1.013 | 12,000.0 |
| Gate, rays off | 8,911.1 | 9,474.8 | 8,686.5 | 8,968.4 | +224.6 | 1.026 | 6,000.0 |
| Shadow-ray time | 11,287.0 | 12,390.2 | 11,271.8 | 12,273.6 | +15.2 | 1.001 | — |

That pair's GPU frame is 1.4% longer spin-waiting, where the earlier
runs' were within 40 µs; a busy P-core may take power the GPU would
otherwise have, and judging GPU lines needs that in mind.

On the light scenes, `pbr-sweep` and `normal-tangent`, `lit_pass` and
`tonemapping_pass` read about 490 and 795 µs in some runs and 3,000 and
3,300 in others, with either wait and the GPU frame unchanged, and
`texture-wall`'s passes read 70–80% over their Baseline while its GPU
frame matches it. Pass lines on those scenes are not yet a reliable
measure; #399 takes them.

## minification's far moiré is in the box-filtered reference (2026-10-30, #359)

**Verdict.** The arcs and diagonal stripes in `minification`'s far
checker band at 2560×1440 are in the supersampled reference too, so no
step of the texture path aliases them: they are what a box over each
pixel lets through. A reference that integrates the checker exactly over
each of 3×3 subpixels, box-filtered in linear light, shows the same arcs
at the same places and strength, and the capture lies 1.68 8-bit steps
RMSE from it in the band's sides. The Apple sampler, the cooker's box
mips and their BC encoding each move the band by less than a step, the
16x clamp by 1.3 steps, about as much as the 1x and 3x references
differ, and none of them makes or removes the arcs. A Gaussian
reconstruction of the same exact subpixels instead of a box removes most
of the arcs, at the cost of blurring the resolved checks; that is a
choice of filter, a technique decision, not a defect to fix here.

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release build of
`7442717a`, and of `7442717a` with a measurement patch, not committed,
that replaces the lit pass's albedo sample with a variant chosen by an
environment variable and reads the colour sampler's anisotropy from
another. Its control, today's sample at 16x, matches `7442717a`'s
capture pixel for pixel. Every image is `praxis capture minification` at
the Golden's frame, 59, at 2560×1440 or at 7680×4320 reduced to
2560×1440 with a box in linear light (`magick -colorspace RGB -filter box
-resize`). The far band is rows 495 to 599, the floor from about 60 m to
15 m out along the centre, where a pixel's footprint along the floor is
14 to 0.9 periods of the checker's 1/8 m; it is scored on its outer 800
pixels each side, where the moiré is strongest and the light's
reflection, which specular antialiasing widens more at 1x than at 3x,
does not reach. The checker's period across the view stays about 4 to
14 pixels there, so the checks stay resolved across and unresolved along.

The variants, each an albedo for the same lighting:

- **Exact.** The checker's mean over the pixel's footprint, the
  parallelogram `uv ± dpdx(uv)/2 ± dpdy(uv)/2`: exact along its longer
  side, stepping cell to cell, at 32 lines across the shorter. A 16×16
  grid of point samples of the checker agrees with it within 1.32 steps
  at 3x, so the reference has converged.
- **Software filter, Vulkan rule.** N bilinear taps of the cooked mips,
  spaced evenly over the pixel along its longer screen derivative, at the
  level and count the Vulkan specification gives, `log2(major / N)` with
  N = `ceil(major / minor)`, clamped at 16 or not
  [ref:vulkan-anisotropy-lod].
- **Mip 0.** A 16×16 grid of bilinear samples of the cooked mip 0, BC
  included, at 3x.
- **Anisotropy 8x and 1x.** Today's sample with the sampler's anisotropy
  clamped lower.

Far band, in 8-bit steps: the moiré's strength as the standard deviation
of the band's grey minus its Gaussian blur of σ 6 px, and RMSE against the
two exact references:

| Image | Moiré | RMSE against exact, 3x box | RMSE against exact, 1x |
|---|---:|---:|---:|
| Capture, `7442717a` | 6.21 | 1.68 | 1.44 |
| Exact, 3x box (the reference) | 6.01 | 0 | 1.17 |
| Exact, 1x | 6.35 | 1.17 | 0 |
| Hardware sample, 3x box (the ticket's reference) | 6.00 | 0.48 | 1.24 |
| Software filter, 16 taps at most | 6.20 | 1.67 | 1.43 |
| Software filter, unclamped | 6.27 | 1.13 | 0.49 |
| Mip 0, 3x box | 5.99 | 0.91 | 1.45 |
| Anisotropy 8x | 5.63 | 2.99 | 3.14 |
| Anisotropy 1x | 0.38 | 10.52 | 11.02 |
| Exact, 3x, Gaussian of σ 0.75 px (`magick -filter gaussian`) | 1.48 | 9.05 | 9.59 |

- **The reference has it.** Both 3x references, exact and hardware,
  carry the moiré at the capture's strength, and their crops show the
  same arcs in the same places. A box's mean of detail finer than itself
  never settles to zero: it leaves a residue that shrinks only slowly as
  the footprint grows and changes sign as the footprint gains or loses
  part of a period, and one sample per pixel of that residue beats into
  the arcs.
- **The adapter's anisotropic approximation.** The capture lies 0.23
  steps from the software filter at 16 taps, so on this floor the M4
  Pro's sampler filters as one built from the Vulkan rule does; the
  specification leaves the scheme to the implementation.
- **The LOD the clamp picks.** Past 16 the clamp reads a coarser mip and
  blurs across the view, as the ticket expected, but along the view its
  16 taps still cover the footprint, which is where the moiré lives:
  unclamped, the software filter comes within 0.49 steps of the exact 1x
  mean, and keeps the moiré. Clamped and unclamped differ by 1.30
  steps.
- **The cooker's mips and BC.** The unclamped software filter reads the
  cooked mips and comes within 0.49 steps of the exact mean; mip 0, BC encoded,
  within 0.91 of the exact reference.
- **What would remove it.** A filter wider than the pixel, here a
  Gaussian over the 3x subpixels, takes the moiré from 6.0 to 1.5 and
  blurs the checks that the box resolves, 9 steps from the box reference.
  Choosing a pixel filter, or a texture filter wider than the pixel, is a
  technique decision (`docs/agents/workflow.md`, Tickets), proposed as
  #388; ADR 0008 rules out the temporal kind.

Images, on the `pr-images` branch at `47f4fe3`: the
[capture](https://raw.githubusercontent.com/gariasf/praxis/47f4fe3ee0354fc0c574c3f81ea81cd680be9704/359/capture.png),
the [exact reference](https://raw.githubusercontent.com/gariasf/praxis/47f4fe3ee0354fc0c574c3f81ea81cd680be9704/359/reference-exact.png),
the [hardware reference](https://raw.githubusercontent.com/gariasf/praxis/47f4fe3ee0354fc0c574c3f81ea81cd680be9704/359/reference-hardware.png),
the capture's [difference](https://raw.githubusercontent.com/gariasf/praxis/47f4fe3ee0354fc0c574c3f81ea81cd680be9704/359/difference-capture-vs-reference-exact.png)
from the exact reference, mostly the light's reflection, and the far
band's sides doubled, the capture above the references and the Gaussian
([left](https://raw.githubusercontent.com/gariasf/praxis/47f4fe3ee0354fc0c574c3f81ea81cd680be9704/359/far-band-left-2x.png),
[right](https://raw.githubusercontent.com/gariasf/praxis/47f4fe3ee0354fc0c574c3f81ea81cd680be9704/359/far-band-right-2x.png))
and the software filters beside the exact 1x mean
([left](https://raw.githubusercontent.com/gariasf/praxis/47f4fe3ee0354fc0c574c3f81ea81cd680be9704/359/far-band-left-2x-filters.png)).

## Option D, a visibility buffer, prototyped on `town`: the gate stays past 6 ms (2026-10-28, #297)

**Verdict.** Option D does not reach ADR 0019's gate on `town` and does
not beat A there: in the median rounds the gate reads 8.77 ms against
A's 8.67 and the 6 ms limit, 1.2% slower (D 8.77 to 8.81 ms over three
rounds, A 8.63 to 8.69), and 8.43 ms, 2.3% faster than A's 8.63 in its
round, with each pixel's own derivatives in place of A's per-quad ones. Shading
from IDs in compute costs on the M4 Pro about what the forward lit pass
costs: the IDs themselves are cheap, about 0.22 ms in the depth pre-pass,
but nothing D removes from the gate (quad overdraw, the lit pass's vertex
work at depth equal) outweighs the per-pixel triangle rebuild. With shadow
rays on, D's GPU frame reads 1.5 ms longer than A's (16.9 against 15.4 ms),
and it takes 58,982,432 bytes more GPU memory, the 4x IDs and a 32-byte
uniform. Its CPU frame reads 2.1 ms shorter. The image matches A's at
`town`'s eight frames within the Goldens' tolerance but at 5 to 617 of
230,400 pixels at 640×360, and at 23 to 8,947 of 3,686,400 at 2560×1440:
isolated pixels on the silhouettes of sub-pixel geometry. The prototype
stays on its branch, `prototype/visibility-d` at b846161. Both options
ADR 0019 named are now measured on `town`, and neither moves the gate:
what breaches it is the triangles every option draws.

**Shape as built.** With the `visibility_buffer` Render switch on
(`--visibility-buffer` on `bench` and `capture`), both depth passes write
each sample's triangle, its primitive index in its phase's draw with a bit
marking phase two's, into a 4x Rg16Uint target beside depth, and with
shadow rays on the surface normals beside it; phase two's depth is a pass
of its own either way. The IDs are written in the depth passes, not in a
pass at depth equal as option B's prototype wrote them, so they cost 0.22
ms in the pre-pass rather than B's 2.08 ms pass; D draws nothing again, so
the IDs need only agree with themselves. Each ID entry point reads a flat
input, the Material ID, so the M4 Pro merges no fragments (#348). A compute
material pass replaces the lit pass: per pixel it reads the four samples'
IDs, and for each distinct triangle among them, rebuilds its corners from
the frame's draw as the vertex shader does, takes perspective-correct
weights from the corners' clip x, y and w (2D homogeneous coordinates, no
divide by w, so corners behind the camera need no clipping), samples every
Material with `textureSampleGrad` in one shader, lights the surface with
the lit pass's own code, moved into a part both share, and averages the
triangles' colours weighted by the samples each covers, uncovered samples
taking the clear colour, into the HDR target. That is the lit pass's
shading rate under 4x MSAA, once per triangle per pixel at the pixel's
centre, resolved as its hardware resolve averages: neither every sample
shaded at edges nor sample 0 alone. The pass reads 9 storage buffers in
its compute stage, the Capability floor's limit, so it declares the frame
and geometry buffers it reads in groups of its own and has the GPU copy
phase two's first index into its uniform rather than bind the frame's
draw. The device requests `PRIMITIVE_INDEX` beside the floor where the
adapter has it, as B's prototype did.

**Derivatives.** Gradients come from evaluating the triangle on its plane
at neighbouring pixel centres, as helper lanes would. Taken from each
pixel's own right and lower neighbours, `town` differed from A past the
tolerance at 132 to 2,054 pixels per frame at 640×360; taken across the
pixel's 2×2 quad, from its top left pixel, at 5 to 617, so the lit
pass's `dpdx` and `dpdy` on the M4 Pro behave as one difference per quad.
The prototype measures the quad form, which matches A's image; it costs
the material pass one more normal-Texture sample and weight evaluation per
triangle, and the gate 365 µs (round 4 below).
The per-pixel form is the one option D's research note credits with right
filtering, and neither is wrong; it is what a D adopted for its own sake
would ship.

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release builds of the
prototype branch. `praxis bench <dir> town` at 2560×1440 with 4x MSAA,
1,800 measured frames, three rounds, each A (the switch off) then D
(`--visibility-buffer`), no other GPU work, load average 4.2 at the start; round
4 adds D after the review's fix (b846161) and F, D with per-pixel
derivatives, a local variant. Medians per round, in µs:

| Round | Build | Gate, rays off | GPU frame | CPU frame | `depth_prepass` | `material_pass` |
|---|---|---:|---:|---:|---:|---:|
| 1 | A | 8,667.3 | 15,378.7 | 16,439.0 | 2,139.1 | — |
| 1 | D | 8,806.8 | 16,941.5 | 14,583.0 | 2,355.4 | 5,390.5 |
| 2 | A | 8,691.3 | 15,404.7 | 16,850.0 | 2,138.1 | — |
| 2 | D | 8,770.2 | 16,961.7 | 14,372.5 | 2,355.8 | 5,412.8 |
| 3 | A | 8,631.6 | 15,328.0 | 16,350.5 | 2,140.0 | — |
| 3 | D | 8,767.2 | 16,883.3 | 14,197.6 | 2,357.0 | 5,375.0 |
| 4 | A | 8,631.7 | 15,329.5 | 16,528.8 | 2,141.0 | — |
| 4 | D | 8,795.0 | 16,945.0 | 14,360.0 | 2,357.4 | 5,419.5 |
| 4 | F | 8,430.5 | 16,634.8 | 14,470.6 | 2,337.4 | 5,064.0 |

Pass times are taken in frames with shadow rays on and overlap on Metal:
A's `lit_pass` reads 11,256 µs and D's `tonemapping_pass` 12,424 µs
against A's 4,029, each spanning time it spends waiting on the pass
before. They do not add up to either frame, and the A/B compares spans,
the gate and the GPU frame. D's GPU frame grows by more than its gate,
with shadow-ray time 7,154 µs against A's 6,265, by the mechanism #378
found (the section below): D's CPU frame, 14.4 ms, is shorter than its
GPU frame, so frames queue and Metal stops overlapping the passes it
overlaps when the GPU is idle at a frame's start, as it is under A's
16.4 ms CPU frame. That CPU frame is 2.1 ms shorter because the material
pass binds no TLAS, each use of which cost wgpu-core 30.0.1 a walk over
every Instance's BLAS (#352). Since #378 only the shadow mask pass binds
them, so on `main` A loses this CPU gap and the overlap alike, and the
GPU-frame comparison above holds for the code it measured, not for
today's.

A's median round, 1, against `town`'s Baseline:

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 16,439.0 | 17,179.0 | 16,901.6 | 18,152.2 | -462.6 | 0.973 | 6,000.0 |
| Dynamic TLAS, CPU | 974.1 | 1,100.1 | 1,014.2 | 1,155.1 | -40.1 | 0.960 | 1,100.0 |
| GPU frame | 15,378.7 | 16,476.0 | 15,399.8 | 16,383.5 | -21.1 | 0.999 | 12,000.0 |
| Gate, rays off | 8,667.3 | 9,381.3 | 8,654.9 | 9,256.8 | +12.4 | 1.001 | 6,000.0 |
| Shadow-ray time | 6,265.3 | 7,488.9 | 6,338.3 | 7,388.1 | -73.0 | 0.988 | — |
| `bend_pass` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | — |
| `blas_refit` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | — |
| `culling_phase_one` | 1,107.9 | 1,154.2 | 1,113.1 | 1,154.9 | -5.2 | 0.995 | — |
| `culling_phase_two` | 169.4 | 193.2 | 169.4 | 192.6 | +0.0 | 1.000 | — |
| `depth_prepass` | 2,139.1 | 2,235.1 | 2,117.8 | 2,213.8 | +21.3 | 1.010 | — |
| `depth_pyramid_pass` | 260.4 | 322.7 | 260.3 | 305.2 | +0.1 | 1.000 | — |
| `dynamic_tlas_build` | 49.7 | 544.7 | 100.5 | 573.2 | -50.8 | 0.494 | 1,000.0 |
| `froxel_pass` | 528.2 | 680.1 | 536.5 | 684.5 | -8.2 | 0.985 | — |
| `lit_pass` | 11,256.4 | 11,999.5 | 11,269.8 | 12,014.8 | -13.4 | 0.999 | — |
| `material_pass` | 0.0 | 0.0 | — | — | — | — | — |
| `phase_two_depth_pass` | 58.3 | 112.8 | 59.0 | 113.6 | -0.7 | 0.988 | — |
| `shadow_mask_pass` | 6,983.8 | 7,758.0 | 7,041.0 | 7,802.9 | -57.2 | 0.992 | — |
| `tonemapping_pass` | 4,028.5 | 7,053.5 | 4,010.0 | 6,448.4 | +18.5 | 1.005 | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 4 | 4 | +0 | 1.000 | — |
| `instances_drawn` | 137,730 | 137,730 | +0 | 1.000 | — |
| `triangles_drawn` | 5,552,214 | 5,552,214 | +0 | 1.000 | — |
| `instances_overflowed` | 0 | 0 | +0 | — | — |
| `uploaded_bytes` | 2,049,344 | 2,049,344 | +0 | 1.000 | — |
| `gpu_memory_bytes` | 675,535,944 | 675,537,864 | -1,920 | 1.000 | 8,000,000,000 |
| `arena_bytes_in_use` | 85,280 | 85,280 | +0 | 1.000 | — |
| `heap_allocations` | 10,489 | 10,489 | +0 | 1.000 | — |

D's median round, 2, against the same Baseline:

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 14,372.5 | 14,907.5 | 16,901.6 | 18,152.2 | -2,529.1 | 0.850 | 6,000.0 |
| Dynamic TLAS, CPU | 1,001.5 | 1,096.8 | 1,014.2 | 1,155.1 | -12.6 | 0.988 | 1,100.0 |
| GPU frame | 16,961.7 | 18,031.7 | 15,399.8 | 16,383.5 | +1,561.9 | 1.101 | 12,000.0 |
| Gate, rays off | 8,770.2 | 9,133.8 | 8,654.9 | 9,256.8 | +115.3 | 1.013 | 6,000.0 |
| Shadow-ray time | 7,154.0 | 8,255.9 | 6,338.3 | 7,388.1 | +815.7 | 1.129 | — |
| `bend_pass` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | — |
| `blas_refit` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | — |
| `culling_phase_one` | 1,131.8 | 1,170.8 | 1,113.1 | 1,154.9 | +18.8 | 1.017 | — |
| `culling_phase_two` | 168.2 | 184.4 | 169.4 | 192.6 | -1.2 | 0.993 | — |
| `depth_prepass` | 2,355.8 | 2,619.8 | 2,117.8 | 2,213.8 | +238.0 | 1.112 | — |
| `depth_pyramid_pass` | 258.7 | 269.0 | 260.3 | 305.2 | -1.6 | 0.994 | — |
| `dynamic_tlas_build` | 511.0 | 663.5 | 100.5 | 573.2 | +410.5 | 5.085 | 1,000.0 |
| `froxel_pass` | 560.2 | 582.8 | 536.5 | 684.5 | +23.8 | 1.044 | — |
| `lit_pass` | 0.0 | 0.0 | 11,269.8 | 12,014.8 | -11,269.8 | 0.000 | — |
| `material_pass` | 5,412.8 | 5,920.3 | — | — | — | — | — |
| `phase_two_depth_pass` | 60.9 | 115.8 | 59.0 | 113.6 | +1.9 | 1.032 | — |
| `shadow_mask_pass` | 6,569.5 | 7,272.7 | 7,041.0 | 7,802.9 | -471.5 | 0.933 | — |
| `tonemapping_pass` | 12,423.6 | 13,334.3 | 4,010.0 | 6,448.4 | +8,413.5 | 3.098 | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 3 | 4 | -1 | 0.750 | — |
| `instances_drawn` | 137,730 | 137,730 | +0 | 1.000 | — |
| `triangles_drawn` | 5,552,214 | 5,552,214 | +0 | 1.000 | — |
| `instances_overflowed` | 0 | 0 | +0 | — | — |
| `uploaded_bytes` | 2,049,360 | 2,049,344 | +16 | 1.000 | — |
| `gpu_memory_bytes` | 734,518,376 | 675,537,864 | +58,980,512 | 1.087 | 8,000,000,000 |
| `arena_bytes_in_use` | 85,280 | 85,280 | +0 | 1.000 | — |
| `heap_allocations` | 10,484 | 10,489 | -5 | 1.000 | — |

`gpu_memory_bytes` reads 1,920 bytes under the Baseline on both sides: the
GPU stats readback left the count after the Baseline was recorded (#349).
D adds the 4x visibility IDs, 58,982,400 bytes, and its uniform; its 4x
colour target stays allocated unused, memoryless on the M4 Pro. `uploaded_bytes`
grows by the clear colour's 16 bytes a frame.

**Images.** `cargo test --release --test visibility_buffer` captures
`town` at the Goldens test's eight frames with the switch off and on, and
with `PRAXIS_VISIBILITY_IMAGES=<directory>` writes both and their
difference; it allows 700 pixels past the tolerance per frame. Pixels past
one 8-bit step, per frame, at 640×360: 5, 10, 107, 561, 604, 657, 590 and
617; at 2560×1440 (`praxis capture town <png> 2560x1440 --frame <n>` with
and without `--visibility-buffer`, then `praxis difference`): 23, 29, 735,
8,060, 8,802, 8,947, 8,693 and 8,719, the largest channel difference 132.
The differing pixels sit alone on the edges of rooftop objects and other
geometry under a pixel wide.

**What it leaves.** The owner kept A and the 6 ms, and ADR 0019 now
records the reopen clause as spent on `town` until LOD
([decision](https://github.com/gariasf/praxis/issues/297#issuecomment-6491023439)): B and D
are prototyped, and no form of either takes more than 2.3% off A's gate
(B added 1.9 to 3.5 ms). The gate's 8.6 ms is the frame's triangles: culling 1.3 ms, the depth
pre-pass 2.1 and shading, in the lit pass or the material pass, roughly 5,
the pass times overlapping as above. Fewer triangles is the lever every
option shares, which on `town` means LOD (#360, M4's charting). A D for its
own sake would need `PRIMITIVE_INDEX` in the floor, 59 MB at 1440p, a
storage-buffer limit above the floor's 9 for any more inputs, and the
transparent pass forward beside it.

## The TLAS bound in the shadow mask pass alone: queued frames, not the split, cost the GPU 1.1 ms (2026-10-28, #378)

**Verdict.** Binding both TLAS only in the shadow mask pass adds no GPU
work. `town`'s `submit` falls from 13.3 to 2.8 ms with rays on, and from
11.3 ms to 26 µs with rays off; the CPU frame falls from 16.9 to 7.0 ms
in the re-recorded Baseline. The GPU frame grows 1.1 ms, as #352's probe
saw, but the cause is the GPU becoming the bottleneck, which any faster
CPU frame brings, not the split. Once the CPU outpaces the GPU, each
frame is queued behind the one running. With rays on, Metal then stops
overlapping passes it overlapped when the GPU was idle at each frame's
start. The lit pass no longer starts beside the shadow mask pass, nor
the Froxel pass beside culling phase one, and the frame's passes span
16.5 ms instead of 15.3. Base's binding layout with upstream's TLAS
dependency dedup patched in, whose CPU also outpaces the GPU, loses the
same overlap; one frame in flight keeps it. #382 takes the lost
overlap. The dynamic TLAS's build, about 0.67 ms of GPU, runs beside the
previous frame when frames queue (it reads 0), and is exposed when the
GPU is idle at the frame's start.

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release builds of
`main` at 4ef4f7d3 (base) and of this change (split), with measurement
patches, not committed. The patches logged the median interval between
frames over all but the first 100 frames, each CPU stage's median and
each frame's raw pass timestamps. Switches, by environment variable,
made each frame sleep out the rest of 17 ms after it was submitted
(paced; the frames then took about 20 ms) and set the frames in
flight. One variant patched wgpu-core 30.0.1 with gfx-rs/wgpu#9835's
dedup [ref:wgpu-pr-9835], so each frame's TLAS uses walk their distinct
BLASes only. Another had the dynamic TLAS's first marker read the HDR
target, so the next frame's build waited for this frame's lit pass.
`praxis bench <dir> town` at 2560×1440 with 4x MSAA, 1,800 measured
frames, no other GPU work.

Medians in µs, as `praxis report` prints them; base, split and paced
interleaved over two rounds. The interval is the patch's, with rays on.

| Run | Round | Interval | CPU frame | GPU frame | `dynamic_tlas_build` | Gate, rays off | Shadow-ray time | Dynamic TLAS, CPU |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Base | 1 | 16,731 | 16,727.3 | 15,392.4 | 104.5 | 8,606.1 | 6,357.9 | 963.0 |
| Split | 1 | 16,427 | 7,229.3 | 16,482.3 | 0.0 | 8,687.4 | 7,367.5 | 1,282.5 |
| Split, paced | 1 | 20,017 | 4,984.2 | 15,972.5 | 670.4 | 8,730.3 | 6,103.2 | 918.0 |
| Base | 2 | 16,471 | 16,480.6 | 15,384.3 | 95.2 | 8,614.2 | 6,323.4 | 967.7 |
| Split | 2 | 16,438 | 6,846.2 | 16,483.9 | 0.0 | 8,700.6 | 6,617.5 | 1,240.5 |
| Split, paced | 2 | 19,889 | 5,490.8 | 16,006.5 | 671.4 | 8,618.9 | 6,298.5 | 969.8 |

Paced, the GPU frame less the build reads 15,302.1 and 15,335.1, against
base's 15,287.9 and 15,289.1. Round 1's passes, base against paced:
`culling_phase_one` 1,113.2 and 1,109.7, `depth_prepass` 2,138.7 and
2,156.8, `shadow_mask_pass` 7,009.1 and 6,972.8, `lit_pass` 11,244.3 and
11,224.6, and the gate within 1.5%. The variants, one run each:

| Run | Frames in flight | CPU frame | GPU frame | `dynamic_tlas_build` | `froxel_pass` | `lit_pass` | Dynamic TLAS, CPU |
|---|---:|---:|---:|---:|---:|---:|---:|
| Split | 1 | 4,870.9 | 15,988.1 | 667.9 | 534.9 | 11,222.9 | 909.1 |
| Split, build after the lit pass | 2 | 7,021.4 | 16,988.3 | 511.7 | 385.6 | 5,545.5 | 1,261.9 |
| Split with the dedup | 2 | 2,548.2 | 16,494.4 | 0.0 | 385.8 | 5,594.2 | 586.5 |
| Base with the dedup | 2 | 2,575.3 | 16,578.2 | 0.0 | 385.7 | 5,623.2 | 583.9 |

Where each pass runs, from the raw timestamps: medians of each pass's
start and end, from culling phase one's start, rays on.

| Run | Froxel | Shadow mask | Lit | Tonemapping |
|---|---|---|---|---|
| Base, 2 in flight, the CPU slower | 51–577 | 3,761–10,805 | 3,725–14,983 | 11,333–15,305 |
| Split, 2 in flight | 888–1,273 | 3,915–10,514 | 10,526–16,166 | 12,521–16,488 |
| Split, 1 in flight | 60–593 | 3,797–10,831 | 3,767–15,004 | 11,354–15,326 |
| Split, build after the lit pass | 897–1,283 | 3,929–10,526 | 10,540–16,127 | 12,486–16,478 |

When the GPU is idle at the frame's start, the lit pass's vertex work
runs beside the shadow mask pass and only its fragment work waits for
the mask; queued, it waits for the whole pass. With rays off, the split's
queued frames keep the overlap: Froxel −3–552, lit 3,333–8,664,
tonemapping 5,312–8,988, as base's. Unpaced, `lit_pass` and
`froxel_pass` change with the overlap, so the GPU frame is the figure
to judge.

In the split, each frame's build waits for the previous frame's shadow
mask pass and runs beside its lit pass. In base it waited for the
previous lit pass, which held the TLAS too, and ran beside the
tonemapping pass. Made to wait for the lit pass again, the build ran
exposed at the frame boundary and the passes still spanned 16.5 ms,
so the build's placement is not the cause. Nor is the size of each
frame's residency set, which the dedup shrinks to the distinct BLASes:
the GPU frame stays 16.5 ms. The dedup arrives with wgpu 31; it takes
the CPU frame to 2.5 ms but leaves the GPU frame as is.

**Every CPU stage runs slower when frames queue.** Stage medians from a
third round, rays on, base, split and paced: `prepare_frame` 1,284,
1,671 and 1,288; `propagate_world_transforms` 366, 453 and 346; the
dynamic TLAS's recording 596, 721 and 552, and its submit 331, 474 and
415; `submit` 13,310, 2,783 and 2,566. With rays off, `submit` reads
11,268, 26 and 35. Queued, the main thread waits for the GPU in each
frame, and stages that touch no TLAS slow by the same 20–45% as the
dynamic TLAS's. Paced with rays off, where the thread sleeps most of
each frame, they slow too (`prepare_frame` 2,445). So the dynamic TLAS's
CPU line's 1.24–1.31 ms, over the 1.1 ms ceiling, is not the split's
cost; with one frame in flight it reads 909 µs. #381 measures the cause.

## `town`'s submit is wgpu's TLAS bookkeeping, not a GPU wait (2026-10-28, #352)

**Verdict.** `town`'s 13 ms `submit` stage is CPU work inside
`queue.submit`, not the main thread waiting on the GPU. wgpu-core 30.0.1
records a TLAS's dependencies as one BLAS per Instance it holds, not one
per distinct BLAS [ref:wgpu-30-tlas-dependency-per-instance]: about
244,000 in the static TLAS. Each flush of a bind group holding a TLAS
adds a use [ref:wgpu-30-use-tlas-per-flush], and at submit every use
walks that whole list twice. Validation reads each BLAS's build index
[ref:wgpu-30-validate-tlas-dependencies], and every dependency joins a residency set made fresh for the submission
[ref:wgpu-30-submit-tlas-dependencies], one `addAllocation` each on Metal
[ref:wgpu-hal-30-metal-residency-per-submit]. The scene-geometry group
holds both TLAS, and the culling phases, the depth passes, the shadow
mask pass and the lit pass all bind it. #204's hand-off to #207 named
this mechanism; these runs measure it. The CPU frame already excludes
GPU waits: capture waits on the oldest frame in flight before `update`,
and a window waits in `get_current_texture`. So the CPU frame is the
main thread's critical path, and the owner kept criterion 1 and ADR
0013's CPU Budget on it, with no new line
([decision](https://github.com/gariasf/praxis/issues/352#issuecomment-6479018726)). Upstream
fixed the duplication on trunk after 30.0.0, in
gfx-rs/wgpu#9835 [ref:wgpu-pr-9835], which is not in 30.0.1.

**Tonemapping's 4 ms is overlap.** In every measured frame the tonemapping
pass's start timestamp lands about 3.7 ms before the lit pass's end, and
its end lands 0.32 ms after it. With its draw removed, the pass reads 32
µs and the GPU frame drops 0.24 to 0.28 ms. Its own cost is about 0.3 ms, and the
rest of its 4 ms is the lit pass's tail. Per-pass times on Metal overlap,
so they do not sum to the GPU frame. The GPU frame and the gate are the
figures to judge.

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release builds of `main`
at 5d03cf2 with a measurement patch, not committed. The patch logged each
CPU stage's median and each frame's lit and tonemapping timestamps. It
took frames in flight from an environment variable, with the readback
ring and staging sized for three, which adds 33,552,512 bytes to
`gpu_memory_bytes`. Switches turned off the tonemapping pass's draw, and
bound a twin scene-geometry group whose TLAS bindings held an empty TLAS
in every pass but the shadow mask pass. `praxis bench <dir> town` at
2560×1440 with 4x MSAA, 1,800 measured frames, no other GPU work. A
5-second `sample` of the main thread during the 2-in-flight run found
2,695 of its 3,288 samples (82%) in these two wgpu-core functions, and
the rest of `submit` at 36.

Medians in µs. The rays-off figures come from the same run's second pass
with shadow rays off.

| Run | Frames in flight | `submit` | CPU frame | `submit`, rays off | CPU frame, rays off |
|---|---:|---:|---:|---:|---:|
| Base | 1 | 13,434 | 16,349 | 11,570 | 14,590 |
| Base | 2 | 13,358 | 16,363 | 11,404 | 14,361 |
| Base | 2 | 13,687 | 16,749 | 11,334 | 14,285 |
| Base | 3 | 14,100 | 17,388 | 11,471 | 14,578 |
| TLAS in the shadow mask pass only | 2 | 2,799 | 6,280 | 32 | 3,559 |
| TLAS in the shadow mask pass only | 2 | 2,850 | 6,355 | 31 | 3,781 |

With one frame in flight, the GPU is idle when `submit` starts, and
`submit` reads the same: nothing in it waits on the GPU. With rays off,
the split run's frame binds no TLAS, and its `submit` falls to 31 µs. The
split run against `town`'s Baseline, as `praxis report` prints it:

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 6,280.1 | 12,374.8 | 16,901.6 | 18,152.2 | -10,621.5 | 0.372 | 6,000.0 |
| Dynamic TLAS, CPU | 1,208.8 | 2,430.7 | 1,014.2 | 1,155.1 | +194.6 | 1.192 | 1,100.0 |
| GPU frame | 16,505.6 | 17,392.0 | 15,399.8 | 16,383.5 | +1,105.9 | 1.072 | 12,000.0 |
| Gate, rays off | 8,736.2 | 9,433.3 | 8,654.9 | 9,256.8 | +81.2 | 1.009 | 6,000.0 |
| Shadow-ray time | 7,273.2 | 8,279.4 | 6,338.3 | 7,388.1 | +934.9 | 1.147 | — |
| `dynamic_tlas_build` | 0.0 | 0.0 | 100.5 | 573.2 | -100.5 | 0.000 | 1,000.0 |
| `lit_pass` | 5,605.7 | 6,122.8 | 11,269.8 | 12,014.8 | -5,664.0 | 0.497 | — |
| `shadow_mask_pass` | 6,579.5 | 7,206.2 | 7,041.0 | 7,802.9 | -461.6 | 0.934 | — |
| `tonemapping_pass` | 3,960.0 | 4,356.8 | 4,010.0 | 6,448.4 | -50.1 | 0.988 | — |

The split takes 10.6 ms off the CPU frame, but the GPU frame grows 1.1
ms, 0.9 ms of it in shadow-ray time. The cause is not yet measured: #378
finds it before the split lands. Upstream's fix comes
with wgpu 31, a dependency bump that is the owner's call.

The tonemapping runs, both with two frames in flight. Timestamps are
medians over the measured frames, relative to the lit pass's start:

| Run | GPU frame | `tonemapping_pass` | Lit pass end | Tonemapping start | Tonemapping end |
|---|---:|---:|---:|---:|---:|
| Base | 15,374.6 | 4,029.4 | 11,271.9 | 7,564.8 | 11,594.6 |
| Base | 15,417.4 | 4,035.3 | 11,255.6 | 7,568.4 | 11,583.5 |
| Tonemapping draw removed | 15,134.2 | 32.4 | 11,265.1 | — | — |

The tonemapping start came before the lit pass's end in every frame of
both base runs, and in the one-in-flight run, whose end-to-end gap read
320.3 µs.

## Meshlet culling prototyped on `town`: the gate stays past 6 ms (2026-10-28, #347)

**Verdict.** Testing each Meshlet of a drawn Instance does not pay on
`town`: it drops 4.9% of the triangles drawn, the lit pass and depth
pre-pass give back about 90 µs, the culling phases spend about 130 µs
more (rounds 2 and 3 below, each against its A), and the gate is
unchanged at 8.6 ms against 6. The culling
phases read 1,451 µs, 60% of #347's 2.4 ms, not under half. Fewer
triangles is the only lever on phase one, whose cost is its packed-index
writes (#363), and 4.9% fewer could not bring the phases under 1.2 ms even
if the tests cost nothing. The prototype stays on its branch,
`prototype/347-meshlet-culling` at d242a68; culling still tests
Instances, as ADR 0019 records. With the gate breached under option A
after #363 and this, #297 (option D) is the next prototype on `town`.

**Why so few.** Of the 244,256 Instances `town` places besides its
10,000 pigeons, 171,649 draw a Mesh of one Meshlet, among them 109,760
railing posts and 56,932 boxes, and each pigeon does too. A box's one
Meshlet faces every way, so its normal cone culls nothing; only the
rasterizer drops its six back-facing triangles, after their vertex work.
The other Meshes hold 2 to 5 Meshlets, cut in index order, and 33 of
their 37 Meshlets have a normal cone too wide to cull by, at its Snorm8
cutoff of 127, so only 4 can be dropped for facing away.
Instances already pass the frustum test and, in phase two, the depth
pyramid; the Meshlet test adds only what a Meshlet's own box and cone
rule out.

**What the prototype did.** Per phase, `append_records` tests each
Meshlet of a drawn Instance once against the frustum, in phase two the
depth pyramid, and its normal cone with meshoptimizer's apex-free test on
the sphere around its box [ref:meshoptimizer-0.25-cluster-bounds]. It
writes every record with its index offset within the Instance, or none,
and scans the kept indices per block; a one-workgroup pass scans the
blocks and a pass per slot counts the Instances drawn. Testing once
matters: Metal compiles with fast math by default [ref:msl-4.1-fast-math],
so two entry points testing the same Meshlet could disagree and corrupt
the offsets. The Mesh pool computes each Meshlet's box and cone at load,
since SciFiHelmet's Mesh Blob reads 13.822 bytes per triangle against its
13.83 ceiling, with no room for them. Culling on and off drew the same
image at `town`'s eight frames. On `helmet`, 23 pixels differed with
shadow rays on, the bug #369 names: the normals pass reads no flat input,
so which fragments merge (#348) depends on the triangles the draw holds.

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release builds of the
prototype (M) and of `main` at 7525b7d (A). `praxis bench <dir> town` at
2560×1440 with 4x MSAA, 1,800 measured frames, three rounds, each M then
A, no other GPU work. Medians per round, in µs:

| Round | Build | Gate, rays off | `culling_phase_one` | `culling_phase_two` | `depth_prepass` | `lit_pass` | Triangles drawn |
|---|---|---:|---:|---:|---:|---:|---:|
| 1 | A (main) | 8,621 | 1,110 | 168 | 2,119 | 11,283 | 5,552,214 |
| 1 | M (Meshlet test) | 8,620 | 1,197 | 211 | 2,081 | 9,656 | 5,280,720 |
| 2 | A (main) | 8,728 | 1,129 | 180 | 2,089 | 11,288 | 5,552,214 |
| 2 | M (Meshlet test) | 8,735 | 1,219 | 211 | 2,081 | 11,220 | 5,280,720 |
| 3 | A (main) | 8,625 | 1,128 | 178 | 2,094 | 11,267 | 5,552,214 |
| 3 | M (Meshlet test) | 8,631 | 1,234 | 217 | 2,062 | 11,201 | 5,280,720 |

Round 1's M `lit_pass` median stands apart from the other five, while its
gate does not. M's median round, 3, against `town`'s Baseline, which is A
as `main` runs it:

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| GPU frame | 15,441.8 | 16,476.7 | 15,399.8 | 16,383.5 | +42.1 | 1.003 | 12,000.0 |
| Gate, rays off | 8,630.9 | 8,997.9 | 8,654.9 | 9,256.8 | -24.0 | 0.997 | 6,000.0 |
| `culling_phase_one` | 1,233.6 | 1,290.7 | 1,113.1 | 1,154.9 | +120.5 | 1.108 | — |
| `culling_phase_two` | 217.3 | 247.6 | 169.4 | 192.6 | +47.9 | 1.283 | — |
| `depth_prepass` | 2,061.5 | 2,122.3 | 2,117.8 | 2,213.8 | -56.2 | 0.973 | — |
| `lit_pass` | 11,200.8 | 11,912.8 | 11,269.8 | 12,014.8 | -68.9 | 0.994 | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `instances_drawn` | 137,697 | 137,730 | -33 | 1.000 | — |
| `triangles_drawn` | 5,280,720 | 5,552,214 | -271,494 | 0.951 | — |
| `gpu_memory_bytes` | 679,732,168 | 675,537,864 | +4,194,304 | 1.006 | 8,000,000,000 |

`instances_drawn` counts, with the test, only Instances that keep a
Meshlet; `gpu_memory_bytes` grows by the fourth field of each of the 2^20
Meshlet records.

**What would make it pay.** A cut that gives Meshlets narrow cones, such
as a box as its faces, at the cost of more records per Instance; or
denser Meshes than `town`'s, where a Meshlet is a small part of an
Instance. Neither is ticketed: option D is the next step on the gate.

## Index-sharing fragments merge on the M4 Pro unless a flat input is read (2026-10-28, #348)

**Verdict.** Under 2x or 4x MSAA, when the fragment stage reads no
flat-interpolated input, the M4 Pro can merge the fragments of nearby
triangles of one draw that share vertex indices: in a pixel both cover,
one fragment, with one triangle's interpolated inputs and
`primitive_index`, writes the samples of both. The samples they cover
together are exact; which triangle each shows depends on the triangles
near them in the draw. Reading any flat input stops it, and a triangle
drawn alone covers exactly its own samples. #296's ID pass read no flat
input, so it credited samples to a triangle's neighbour, dropped the
neighbour, and the draw of the named triangles lost those samples; whole
Meshlets keep both and lose none. It resembles quad-fragment merging,
which Fatahalian et al. proposed as a way "to gather and merge rasterized
fragments from adjacent triangles in a mesh"
[ref:fatahalian-2010-quad-fragment-merging]. No Apple documentation of
it was found, and only the M4 Pro was measured. The rule is in ADR 0019's
Consequences.

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release build at
`5f0f5f8`. The experiments are `src/render/coverage_tests.rs`: a raster
of quads with sides of 0.3 to 1.5 px at 256×256 whose fragment stage
writes each sample's triangle ID into a multisampled ID target, read back
per sample; the ID is `primitive_index` unless a row says otherwise. The
flat scene's 4,096 quads do not overlap, so an analytic oracle gives each
sample's owner, leaving out samples within 0.01 px of an edge: 9,001 at
4x. Every count below is identical over three runs; `cargo test
--release --lib coverage_experiments -- --ignored --nocapture` prints the
table.

Samples whose ID departs from the oracle, as the partner's (the other
triangle of the same quad) and as any other; a quad is drawn `0 1 2`, `0 2
3`, as the cooker draws one. Each quad's `q1` carries a weight only the
first triangle interpolates and `q3` one only the partner does, and every
vertex the quad's index as a smooth input, so a fragment's smooth inputs
alone say which triangle they were interpolated over:

| Draw | 4x: partner's, other | 2x | 1x |
|---|---:|---:|---:|
| Partner with its own indices (another Meshlet or Instance) | 0, 0 | 0, 0 | 0, 0 |
| Partner sharing two indices, as within a Meshlet | 1421, 0 | 268, 0 | 0, 0 |
| The same, sample-rate shading | 1421, 0 | 268, 0 | 0, 0 |
| The same, IDs from the weights each interpolates | 0, 0 | 0, 0 | 0, 0 |
| The same, IDs from smooth inputs alone | 1421, 0 | 268, 0 | 0, 0 |
| The same, primitive_index beside a smooth input | 1421, 0 | 268, 0 | 0, 0 |
| The same, primitive_index beside a flat input | 0, 0 | 0, 0 | 0, 0 |
| The same, both IDs from one fragment: primitive_index's | 0, 0 | 0, 0 | 0, 0 |
| The same, both IDs from one fragment: the weights' | 0, 0 | 0, 0 | 0, 0 |
| Partner sharing them, led by another vertex | 1421, 0 | 268, 0 | 0, 0 |
| The same, IDs from each first vertex's flat output | 0, 0 | 0, 0 | 0, 0 |
| Partner removed | 0, 0 | 0, 0 | 0, 0 |
| Every other quad removed | 760, 0 | 145, 0 | 0, 0 |
| Triangle 1 removed, so later pairs shift by one | 1413, 0 | 267, 0 | 0, 0 |
| One triangle between partners | 1286, 0 | 257, 0 | 0, 0 |
| Every partner after every first triangle | 0, 0 | 0, 0 | 0, 0 |
| Partners in a later draw call | 0, 0 | 0, 0 | 0, 0 |
| Partners in an earlier draw call | 0, 0 | 0, 0 | 0, 0 |

- **Fragments, not just the index.** IDs built from smooth inputs alone,
  with no flat input and no `primitive_index`, depart exactly where
  `primitive_index` does, so a merged fragment carries one triangle's
  interpolated inputs onto the other's samples. Every row that reads a
  flat input (the weights' IDs, which take the quad from one, the flat
  IDs, and `primitive_index` beside a flat input) departs nowhere, even
  where the flat value is the same for both triangles. Every departure
  goes to the partner, never to another quad or to an empty sample.
- **What triggers it.** No flat input, shared vertex indices, nearness in
  the draw (one triangle between still merges most; a partner thousands
  of triangles later, or in another draw call, never) and more than one
  sample per pixel. Sample-rate shading does not stop it.
- **Another Meshlet or Instance.** The renderer's packed indices carry
  the Meshlet record in their high bits, so triangles of different
  Meshlets or Instances never share an index, whatever their positions:
  the first row stands for both.
- **The prototype's loss, reproduced.** On #296's tilted scene (7,871
  quads 0.3 m wide at 40 m), `primitive_index` IDs at depth equal after a
  pre-pass, then a draw of the triangles they name: 697 samples lost and
  554 credited to the partner. The same with whole quads kept: none lost,
  368 credited to the partner. With own indices per triangle: none of
  either. On the prototype branch itself (`5bb7abd`), an ID pass over its
  compacted draw showed the same 697 and 554.
- **Ruled out**, on earlier revisions of the harness and the prototype
  branch rather than by anything on `main`: the prototype's IDs still
  changed at 1,259 samples at depth compare Always, and the clip position
  interpolated at every sample one triangle covers in both draws was
  bit-identical; the vertex entry point its ID pass used; direct against
  indirect draws; back faces culled; triangles crossing the viewport's
  edge; clip positions computed on the GPU.
- **What it means.** Meshlet culling (#347) decides from geometry, which
  this does not touch. An ID pass or a visibility buffer (#297) reads a
  flat input, which costs nothing, or keeps whole Meshlets. #369 is this:
  the depth passes' `fs_surface_normals` reads no flat input and the lit
  pass's `fs_main` reads `material_id`, so at shared-edge samples the
  normals can come from a merged fragment the lit pass never sees.

## Option B, a visibility pre-pass, prototyped on `town` (2026-10-27, #296)

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release builds of the
prototype branch `prototype/visibility-b` at 5bb7abd, which stays on its
branch. `praxis bench <dir> town` at 2560×1440 with 4x MSAA, 248 warm-up
and 1,800 measured frames, three rounds, each round A (the switch off),
then B (`--visibility-prepass`, Meshlet granularity), then T
(`--visibility-prepass-triangles`, triangle granularity), interleaved so
load falls on each alike. Load average 3.9 to 5.3, no other GPU work.
The gate's medians across rounds spread under 0.3% (A 9,652 to 9,672 µs,
B 13,157 to 13,194, T 11,512 to 11,533); the tables are the median
round's, each against `town`'s Baseline of 2026-10-26, which is A as
`main` runs it.

**Verdict.** Option B does not reach ADR 0019's gate on `town`, and
cannot on this GPU: the gate reads 13.2 ms at Meshlet granularity and
11.5 ms at triangle granularity against A's 9.7 and the 6 ms limit. The
lit pass gives back 2.9 ms, but the visibility ID pass costs 2.1 ms and
the visibility pass 2.4 to 3.4 ms. Even with both free, the gate would
read about 7.0 to 7.7 ms; and the one way to make the IDs free, writing
them in the depth pre-pass, loses samples on the M4 Pro (below).
Triangle granularity, the ticket's shape, is a lower bound, not a
candidate: it draws a different image.

**Shape as built.** With the switch on, phase two's depth goes in its own
pass, as with shadow rays; a visibility ID pass then draws both phases'
draw again at depth equal, the lit pass's own depth state, and writes
each sample's triangle, the draw's primitive index, into a 4x Rg16Uint
target (58,982,400 bytes; R32Uint has no MSAA among WebGPU's guaranteed
format features, so the two halves of the ID fill Rg16Uint instead and
no adapter-specific feature is requested). A visibility pass marks those
triangles, or with Meshlet granularity every triangle of their Meshlets,
sums the marks with the culling pass's scan, and copies the marked
triangles' packed indices in draw order into a compacted draw, which the
lit pass draws. The device requests `PRIMITIVE_INDEX` beside the
Capability floor where the adapter has it; it is Tier 1 in ADR 0021, so
raising the floor is an owner decision.

**B, Meshlet granularity**, median round, against the Baseline:

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 19,084.2 | 19,844.2 | 17,652.7 | 18,184.2 | +1,431.5 | 1.081 | 6,000.0 |
| Dynamic TLAS, CPU | 975.9 | 1,071.0 | 1,039.2 | 1,109.5 | -63.3 | 0.939 | 1,100.0 |
| GPU frame | 17,031.6 | 18,415.2 | 13,568.3 | 14,188.4 | +3,463.3 | 1.255 | 12,000.0 |
| Gate, rays off | 13,173.9 | 13,641.2 | 9,820.7 | 10,483.3 | +3,353.2 | 1.341 | 6,000.0 |
| Shadow-ray time | 3,454.0 | 4,764.3 | 3,228.0 | 3,984.7 | +226.1 | 1.070 | — |
| `bend_pass` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | — |
| `blas_refit` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | — |
| `culling_phase_one` | 1,525.3 | 1,669.3 | 1,513.0 | 1,585.4 | +12.3 | 1.008 | — |
| `culling_phase_two` | 901.0 | 1,034.8 | 897.8 | 936.2 | +3.2 | 1.004 | — |
| `depth_prepass` | 1,753.8 | 1,880.4 | 1,756.3 | 1,814.6 | -2.6 | 0.999 | — |
| `depth_pyramid_pass` | 249.0 | 270.7 | 250.4 | 266.8 | -1.4 | 0.994 | — |
| `dynamic_tlas_build` | 0.0 | 596.0 | 48.6 | 462.8 | -48.6 | 0.000 | 1,000.0 |
| `froxel_pass` | 509.0 | 550.3 | 517.2 | 556.3 | -8.3 | 0.984 | — |
| `lit_pass` | 5,916.0 | 8,348.5 | 8,714.4 | 9,186.1 | -2,798.5 | 0.679 | — |
| `phase_two_depth_pass` | 46.3 | 96.0 | 48.9 | 98.6 | -2.6 | 0.946 | — |
| `shadow_mask_pass` | 5,426.6 | 6,155.9 | 4,648.3 | 4,940.3 | +778.3 | 1.167 | — |
| `tonemapping_pass` | 3,825.2 | 4,096.2 | 3,825.2 | 4,499.8 | +0.0 | 1.000 | — |
| `visibility_id_pass` | 2,079.5 | 2,213.8 | — | — | — | — | — |
| `visibility_pass` | 3,436.4 | 5,981.8 | — | — | — | — | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 5 | 4 | +1 | 1.250 | — |
| `instances_drawn` | 137,730 | 137,730 | +0 | 1.000 | — |
| `triangles_drawn` | 5,552,214 | 5,552,214 | +0 | 1.000 | — |
| `instances_overflowed` | 0 | 0 | +0 | — | — |
| `uploaded_bytes` | 2,049,344 | 2,049,344 | +0 | 1.000 | — |
| `gpu_memory_bytes` | 812,688,388 | 616,555,272 | +196,133,116 | 1.318 | 8,000,000,000 |
| `arena_bytes_in_use` | 85,280 | 85,280 | +0 | 1.000 | — |
| `heap_allocations` | 10,535 | 10,478 | +57 | 1.005 | — |

**T, triangle granularity**, median round, against the Baseline:

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 19,096.5 | 19,743.0 | 17,652.7 | 18,184.2 | +1,443.8 | 1.082 | 6,000.0 |
| Dynamic TLAS, CPU | 969.3 | 1,046.3 | 1,039.2 | 1,109.5 | -69.9 | 0.933 | 1,100.0 |
| GPU frame | 15,361.4 | 16,107.2 | 13,568.3 | 14,188.4 | +1,793.1 | 1.132 | 12,000.0 |
| Gate, rays off | 11,527.7 | 11,970.1 | 9,820.7 | 10,483.3 | +1,707.0 | 1.174 | 6,000.0 |
| Shadow-ray time | 3,417.6 | 4,175.2 | 3,228.0 | 3,984.7 | +189.7 | 1.059 | — |
| `bend_pass` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | — |
| `blas_refit` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | — |
| `culling_phase_one` | 1,517.2 | 1,673.5 | 1,513.0 | 1,585.4 | +4.2 | 1.003 | — |
| `culling_phase_two` | 899.3 | 1,033.6 | 897.8 | 936.2 | +1.5 | 1.002 | — |
| `depth_prepass` | 1,753.0 | 1,879.4 | 1,756.3 | 1,814.6 | -3.3 | 0.998 | — |
| `depth_pyramid_pass` | 248.8 | 282.0 | 250.4 | 266.8 | -1.6 | 0.994 | — |
| `dynamic_tlas_build` | 0.0 | 229.8 | 48.6 | 462.8 | -48.6 | 0.000 | 1,000.0 |
| `froxel_pass` | 514.0 | 551.9 | 517.2 | 556.3 | -3.2 | 0.994 | — |
| `lit_pass` | 5,767.4 | 7,854.3 | 8,714.4 | 9,186.1 | -2,947.0 | 0.662 | — |
| `phase_two_depth_pass` | 46.4 | 95.1 | 48.9 | 98.6 | -2.5 | 0.948 | — |
| `shadow_mask_pass` | 4,817.8 | 5,137.0 | 4,648.3 | 4,940.3 | +169.5 | 1.036 | — |
| `tonemapping_pass` | 4,739.9 | 6,890.5 | 3,825.2 | 4,499.8 | +914.7 | 1.239 | — |
| `visibility_id_pass` | 2,083.2 | 2,225.6 | — | — | — | — | — |
| `visibility_pass` | 2,434.0 | 3,281.9 | — | — | — | — | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 5 | 4 | +1 | 1.250 | — |
| `instances_drawn` | 137,730 | 137,730 | +0 | 1.000 | — |
| `triangles_drawn` | 5,552,214 | 5,552,214 | +0 | 1.000 | — |
| `instances_overflowed` | 0 | 0 | +0 | — | — |
| `uploaded_bytes` | 2,049,344 | 2,049,344 | +0 | 1.000 | — |
| `gpu_memory_bytes` | 812,688,388 | 616,555,272 | +196,133,116 | 1.318 | 8,000,000,000 |
| `arena_bytes_in_use` | 85,280 | 85,280 | +0 | 1.000 | — |
| `heap_allocations` | 10,535 | 10,478 | +57 | 1.005 | — |

The memory the switch adds, 196,132,892 bytes, is the visibility IDs
(58,982,400), the compacted indices at the draw's capacity of 2^25
(134,217,728), the triangle bits (1,398,104), their offsets (1,403,568),
the record bits (131,072) and the compacted draw's arguments. The
`shadow_mask_pass` and `tonemapping_pass` readings move because Metal
runs them beside the new passes, not because they do more work; the
gate, a span with rays off, is the figure to judge. The two new timed
passes grow every Baseline's `gpu_memory_bytes` by 224 bytes of
readback, so `compare` refuses the Baselines on the branch.

- **Coverage depends on the draw's neighbours** (superseded 2026-10-28:
  fragments merge, section above). With the IDs written in
  the depth pre-pass, as option B is described, `town` differed from A at
  176 to 4,335 pixels per frame over the eight frames the Goldens test
  checks, scattered over distant geometry: wires, far cubes. Writing the
  IDs at depth equal in a pass of their own, with the lit pass's depth
  state and draw order, still left 180 to 4,276. Forcing every triangle
  marked left 0, so the compaction is right and the marks are short.
  Marking each ID's previous triangle too removed most of the difference,
  and its two neighbours either side nearly all: the lost triangles are
  the other triangle of the same quad face. Encoding each fragment's
  sample mask into its ID showed why: the same triangle covers samples
  0 and 1 of a pixel when its quad partner is in the draw, and sample 0
  alone when drawn without it (7,871 tilted quads 0.3 m wide at 40 m;
  527 pixels differ at 256×256, with depth equal, greater-equal and
  less-equal alike, so the fragments are not generated). On the M4 Pro,
  which samples a triangle under a pixel covers depends on the triangles
  drawn beside it. Drawing whole Meshlets instead reproduces A: 0 pixels
  differ on all eight frames
  ([frame 1800, A | B | difference](https://raw.githubusercontent.com/gariasf/praxis/2ab7f5a7e141446b715dff42e465db69c3144a8d/296/town-1800-a-b-difference.png);
  [every frame](https://github.com/gariasf/praxis/tree/2ab7f5a7e141446b715dff42e465db69c3144a8d/296)), while triangle
  granularity loses 180, 168, 1,216, 3,811, 4,256, 4,265, 4,250 and
  4,276 pixels at frames 59, 300, 500, 700, 900, 1,200, 1,500 and 1,800
  ([where, frame 1800](https://raw.githubusercontent.com/gariasf/praxis/2ab7f5a7e141446b715dff42e465db69c3144a8d/296/town-1800-triangles-lost.png);
  [an 8x crop, A | T | difference](https://raw.githubusercontent.com/gariasf/praxis/2ab7f5a7e141446b715dff42e465db69c3144a8d/296/town-1800-crop-a-triangles-difference.png)).
- **Where B's time goes.** The ID pass, 2.08 ms, is a second raster of
  every drawn triangle: a depth-only raster of them costs 1.75 ms (the
  pre-pass), and the IDs add about 19%, as the research note's probe found for
  `primitive_index`. The visibility pass, 3.44 ms at Meshlet granularity
  and 2.43 at triangle granularity, is the mark over 3.7 million pixels
  and the compaction's writes: bounding its scan to the words the draw
  reaches moved the gate by about 40 µs.
- **What the lit pass gives back.** 2.9 ms with rays on (8.80 to 5.92
  ms), the vertex work and quad overdraw of the triangles that own no
  sample. A's lit pass with rays off reads 5.15 ms, so B's saving there
  is at most that; with both new passes free the gate would read about
  7.0 to 7.7 ms.
- **What the gate is made of.** A's 9.66 ms is both culling phases, 2.4
  ms before any triangle draws; the depth pre-pass, 1.76; the lit pass,
  5.15, of which about 2.9 is vertex work on triangles no sample shows,
  by B's own split, and about 2.3 is shading; and the Froxel pass and
  the depth pyramid, 0.8. About 7 ms of the 9.66 is transforming 5.5
  million triangles twice and culling 524,288 slots, which B and D pay
  alike: D measured today would start from culling plus an ID raster,
  4.5 ms, before a pixel shades. The owner's decision (PR #346): keep A
  and keep the 6 ms, and earn it where the time is, Meshlet culling and
  the culling pass's dispatch, both owed under any option, then
  re-measure; #297 is prototyped after them if the gate still breaches,
  so its reading isolates the architecture.
- **For option D (#297).** D shades from the IDs in compute and draws
  nothing again, so the coverage finding does not bite it the same way,
  but its ID pass is this one: at depth equal after a depth pre-pass,
  2.08 ms on `town`, or the pre-pass's samples with IDs the lit pass
  would not reproduce.

## The RTX 2070 SUPER beside the M4 Pro (2026-10-27, #284)

NVIDIA GeForce RTX 2070 SUPER (Turing, 8 GB), driver 621.01, AMD Ryzen 5
5600X, Windows 11 Pro build 28120, wgpu 30.0.1, release builds of
`faf130b`. This is the comparison adapter, the floor of the hardware
benched; its figures are not Budgets ([decision](https://github.com/gariasf/praxis/issues/17#issuecomment-6389150836)).
`praxis bench <dir>` at 2560×1440 with 4x MSAA, 248 warm-up and 1,800
measured frames, each run twice: with shadow rays on, then off. Three
runs on Vulkan, GPU load 1% before the first and no other GPU process
rendering; each scene's Baseline is its run with the lowest GPU median.
GPU medians agree within 1.1% across the three runs, and CPU medians
within 2.2% but the orrery's, which read 748 to 872 µs. One run on DX12
(`WGPU_BACKEND=dx12 WGPU_DX12_COMPILER=dxc`), whose Baselines sit in
`nvidia-geforce-rtx-2070-super-dx12`. `praxis compare` of each Vulkan run
against the Baselines passes every metric against its Baseline; it fails
`town` on three Budgets alone, as it does on the M4 Pro.

Medians in µs, each as `praxis report bench/baselines/<adapter>` prints
it for the M4 Pro's Baselines and this adapter's two sets, gathered into
one table per figure.

| Scene | GPU frame, M4 Pro | GPU frame, RTX Vulkan | GPU frame, RTX DX12 | Gate, M4 Pro | Gate, RTX Vulkan | Gate, RTX DX12 | CPU frame, M4 Pro | CPU frame, RTX Vulkan | CPU frame, RTX DX12 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| crowd | 2,679.3 | 1,988.5 | 1,550.1 | 1,277.9 | 815.2 | 850.7 | 950.5 | 1,549.7 | 1,875.9 |
| far-coplanar | 2,073.3 | 1,461.8 | 1,353.7 | 703.8 | 728.6 | 819.5 | 229.9 | 303.1 | 446.4 |
| helmet | 4,212.0 | 2,406.9 | 2,261.0 | 1,292.8 | 915.1 | 918.8 | 256.8 | 308.2 | 453.1 |
| minification | 2,296.5 | 1,693.0 | 1,589.2 | 859.0 | 632.1 | 670.0 | 241.5 | 292.2 | 454.1 |
| normal-tangent | 2,537.2 | 1,493.3 | 1,257.5 | 823.0 | 574.2 | 565.2 | 205.6 | 315.3 | 446.2 |
| orrery | 2,140.0 | 1,237.6 | 804.4 | 631.8 | 433.1 | 409.9 | 403.6 | 871.6 | 961.6 |
| pbr-sweep | 2,530.4 | 1,436.2 | 1,173.5 | 766.0 | 540.8 | 524.3 | 213.0 | 304.4 | 448.7 |
| shadows | 3,119.3 | 2,097.6 | 1,985.3 | 872.2 | 664.3 | 688.7 | 317.8 | 438.6 | 698.2 |
| texture-wall | 15,940.8 | 1,092.6 | 816.3 | 14,327.8 | 501.6 | 501.9 | 2,380.3 | 1,127.2 | 1,292.0 |
| town | 13,568.3 | 9,303.4 | 9,449.4 | 9,820.7 | 6,123.2 | 7,492.0 | 17,652.7 | 26,191.1 | 24,815.8 |
| uv-grid | 2,308.1 | 1,626.4 | 1,462.3 | 749.8 | 530.0 | 506.9 | 199.8 | 301.2 | 452.8 |

Every scene's passes, from the same reports:

M4 Pro, pass medians in µs:

| Pass | crowd | far-coplanar | helmet | minification | normal-tangent | orrery | pbr-sweep | shadows | texture-wall | town | uv-grid |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `culling_phase_one` | 86.2 | 22.8 | 165.5 | 22.6 | 65.4 | 31.5 | 35.7 | 37.7 | 41.2 | 1,513.0 | 22.3 |
| `culling_phase_two` | 22.8 | 17.8 | 18.1 | 16.8 | 17.3 | 17.5 | 17.6 | 16.9 | 35.2 | 897.8 | 17.7 |
| `depth_prepass` | 251.8 | 104.8 | 154.2 | 106.5 | 94.8 | 93.9 | 95.5 | 110.9 | 226.0 | 1,756.3 | 89.5 |
| `depth_pyramid_pass` | 254.1 | 251.4 | 229.1 | 257.2 | 199.2 | 192.3 | 204.0 | 270.0 | 438.2 | 250.4 | 216.0 |
| `dynamic_tlas_build` | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 48.6 | 0.0 |
| `froxel_pass` | 11.2 | 6.6 | 6.7 | 5.7 | 6.7 | 6.7 | 6.8 | 5.8 | 11.2 | 517.2 | 6.8 |
| `lit_pass` | 1,708.2 | 329.9 | 758.9 | 488.7 | 461.2 | 324.5 | 447.8 | 461.7 | 585.2 | 8,714.4 | 430.7 |
| `phase_two_depth_pass` | 11.8 | 12.1 | 12.2 | 12.1 | 12.2 | 12.1 | 12.1 | 12.1 | 16.5 | 48.9 | 12.1 |
| `shadow_mask_pass` | 1,154.7 | 989.5 | 2,522.6 | 1,047.9 | 1,340.3 | 1,116.1 | 1,368.5 | 1,862.8 | 2,196.1 | 4,648.3 | 1,174.8 |
| `tonemapping_pass` | 1,794.5 | 636.4 | 1,002.6 | 797.8 | 767.9 | 626.8 | 752.8 | 763.2 | 1,370.9 | 3,825.2 | 741.1 |
| `bend_pass` | 27.6 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

RTX Vulkan, pass medians in µs:

| Pass | crowd | far-coplanar | helmet | minification | normal-tangent | orrery | pbr-sweep | shadows | texture-wall | town | uv-grid |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `culling_phase_one` | 61.8 | 13.2 | 126.8 | 13.2 | 47.2 | 23.3 | 23.6 | 24.5 | 13.1 | 990.4 | 13.2 |
| `culling_phase_two` | 15.9 | 14.1 | 13.8 | 12.4 | 13.9 | 13.9 | 13.9 | 12.5 | 14.1 | 482.5 | 13.9 |
| `depth_prepass` | 128.6 | 184.3 | 127.1 | 39.9 | 30.2 | 17.8 | 33.4 | 55.9 | 24.7 | 1,135.6 | 23.2 |
| `depth_pyramid_pass` | 149.7 | 147.9 | 146.8 | 149.1 | 143.8 | 139.3 | 142.4 | 152.4 | 144.6 | 183.4 | 143.9 |
| `dynamic_tlas_build` | 160.9 | 0.0 | 0.0 | 0.0 | 0.0 | 51.8 | 0.0 | 0.0 | 0.0 | 245.5 | 0.0 |
| `froxel_pass` | 8.6 | 6.7 | 8.0 | 6.8 | 7.5 | 7.5 | 7.6 | 6.1 | 5.4 | 369.3 | 7.7 |
| `lit_pass` | 401.6 | 312.7 | 451.4 | 372.7 | 295.2 | 185.8 | 278.8 | 369.8 | 197.2 | 3,293.0 | 278.5 |
| `phase_two_depth_pass` | 2.5 | 2.4 | 2.4 | 2.4 | 2.4 | 2.4 | 2.4 | 2.4 | 2.4 | 6.9 | 2.4 |
| `shadow_mask_pass` | 740.2 | 507.0 | 1,248.2 | 821.3 | 678.9 | 524.6 | 652.2 | 1,196.9 | 421.0 | 2,321.4 | 864.5 |
| `tonemapping_pass` | 187.1 | 187.1 | 192.4 | 187.4 | 185.8 | 185.9 | 191.5 | 184.2 | 185.4 | 219.1 | 193.0 |
| `bend_pass` | 39.3 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

RTX DX12, pass medians in µs:

| Pass | crowd | far-coplanar | helmet | minification | normal-tangent | orrery | pbr-sweep | shadows | texture-wall | town | uv-grid |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `culling_phase_one` | 61.4 | 14.3 | 141.3 | 14.3 | 49.7 | 24.9 | 25.2 | 26.0 | 14.6 | 1,411.4 | 14.3 |
| `culling_phase_two` | 22.9 | 14.5 | 14.4 | 14.0 | 14.3 | 17.4 | 14.5 | 14.0 | 15.3 | 1,025.4 | 14.3 |
| `depth_prepass` | 127.0 | 190.8 | 125.1 | 43.8 | 29.1 | 17.2 | 33.5 | 65.5 | 25.6 | 1,076.7 | 20.5 |
| `depth_pyramid_pass` | 173.2 | 171.7 | 168.5 | 173.8 | 165.9 | 159.8 | 163.9 | 177.9 | 165.9 | 203.0 | 166.0 |
| `dynamic_tlas_build` | 157.9 | 0.0 | 0.0 | 0.0 | 0.0 | 43.3 | 0.0 | 0.0 | 0.0 | 274.4 | 0.0 |
| `froxel_pass` | 10.9 | 9.6 | 10.0 | 9.6 | 9.2 | 10.0 | 9.2 | 8.4 | 8.2 | 350.3 | 9.3 |
| `lit_pass` | 436.8 | 402.1 | 471.0 | 423.2 | 295.7 | 165.9 | 274.4 | 393.5 | 189.5 | 3,433.5 | 274.9 |
| `phase_two_depth_pass` | 0.4 | 0.4 | 0.4 | 0.4 | 0.4 | 0.4 | 0.4 | 0.4 | 0.4 | 7.7 | 0.4 |
| `shadow_mask_pass` | 297.3 | 327.7 | 1,105.8 | 685.0 | 469.6 | 141.5 | 425.0 | 1,078.7 | 174.1 | 1,454.1 | 739.3 |
| `tonemapping_pass` | 188.4 | 188.5 | 188.4 | 191.4 | 188.6 | 188.5 | 188.5 | 186.8 | 188.3 | 213.0 | 188.8 |
| `bend_pass` | 36.2 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

`town`'s passes beside each other, with its shadow-ray time and the
dynamic TLAS's CPU time:

| Pass | M4 Pro | RTX Vulkan | RTX DX12 |
|---|---:|---:|---:|
| `culling_phase_one` | 1,513.0 | 990.4 | 1,411.4 |
| `culling_phase_two` | 897.8 | 482.5 | 1,025.4 |
| `depth_prepass` | 1,756.3 | 1,135.6 | 1,076.7 |
| `depth_pyramid_pass` | 250.4 | 183.4 | 203.0 |
| `dynamic_tlas_build` | 48.6 | 245.5 | 274.4 |
| `froxel_pass` | 517.2 | 369.3 | 350.3 |
| `lit_pass` | 8,714.4 | 3,293.0 | 3,433.5 |
| `phase_two_depth_pass` | 48.9 | 6.9 | 7.7 |
| `shadow_mask_pass` | 4,648.3 | 2,321.4 | 1,454.1 |
| `tonemapping_pass` | 3,825.2 | 219.1 | 213.0 |
| Shadow-ray time | 3,228.0 | 2,759.2 | 1,477.1 |
| Dynamic TLAS, CPU | 1,039.2 | 1,918.9 | 2,008.8 |

`town` on Vulkan, `praxis report` of the first run against the Baseline,
the second:

**town** on NVIDIA GeForce RTX 2070 SUPER, release, 2560x1440, 1800 measured frames

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 25,819.6 | 37,641.1 | 26,191.1 | 37,897.0 | -371.5 | 0.986 | 6,000.0 |
| Dynamic TLAS, CPU | 1,870.4 | 2,631.0 | 1,918.9 | 2,819.4 | -48.5 | 0.975 | 1,100.0 |
| GPU frame | 9,381.0 | 9,742.0 | 9,303.4 | 9,693.3 | +77.6 | 1.008 | 12,000.0 |
| Gate, rays off | 6,166.7 | 6,355.3 | 6,123.2 | 6,323.3 | +43.5 | 1.007 | 6,000.0 |
| Shadow-ray time | 2,793.6 | 3,174.2 | 2,759.2 | 3,280.0 | +34.4 | 1.012 | — |
| `bend_pass` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | — |
| `blas_refit` | 0.0 | 0.0 | 0.0 | 0.0 | +0.0 | — | — |
| `culling_phase_one` | 993.8 | 1,020.0 | 990.4 | 1,018.0 | +3.4 | 1.003 | — |
| `culling_phase_two` | 482.0 | 503.8 | 482.5 | 499.8 | -0.4 | 0.999 | — |
| `depth_prepass` | 1,187.8 | 1,216.4 | 1,135.6 | 1,173.2 | +52.2 | 1.046 | — |
| `depth_pyramid_pass` | 184.0 | 186.1 | 183.4 | 186.0 | +0.6 | 1.003 | — |
| `dynamic_tlas_build` | 245.8 | 251.9 | 245.5 | 251.9 | +0.3 | 1.001 | 1,000.0 |
| `froxel_pass` | 369.7 | 371.7 | 369.3 | 371.2 | +0.4 | 1.001 | — |
| `lit_pass` | 3,300.4 | 3,457.0 | 3,293.0 | 3,453.3 | +7.4 | 1.002 | — |
| `phase_two_depth_pass` | 8.2 | 20.5 | 6.9 | 18.5 | +1.3 | 1.191 | — |
| `shadow_mask_pass` | 2,346.5 | 3,005.6 | 2,321.4 | 2,998.8 | +25.1 | 1.011 | — |
| `tonemapping_pass` | 219.1 | 219.5 | 219.1 | 219.3 | +0.0 | 1.000 | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 4 | 4 | +0 | 1.000 | — |
| `instances_drawn` | 137,729 | 137,729 | +0 | 1.000 | — |
| `triangles_drawn` | 5,552,214 | 5,552,214 | +0 | 1.000 | — |
| `instances_overflowed` | 0 | 0 | +0 | — | — |
| `uploaded_bytes` | 1,969,344 | 1,969,344 | +0 | 1.000 | — |
| `gpu_memory_bytes` | 720,808,670 | 720,808,670 | +0 | 1.000 | 8,000,000,000 |
| `arena_bytes_in_use` | 85,280 | 85,280 | +0 | 1.000 | — |
| `heap_allocations` | 10,495 | 10,495 | +0 | 1.000 | — |

GPU memory, from the `gpu_memory` line that `praxis capture <scene> <png>
2560x1440` logs for its last frame on each backend, in bytes: the
engine's count (`gpu_memory_bytes`, acceleration structures by the
measured NVIDIA estimate) against wgpu's allocator report
(`allocator_report_bytes`). The engine counts the same on both backends.

| Scene | Engine count | Allocator, Vulkan | Ratio | Allocator, DX12 | Ratio |
|---|---:|---:|---:|---:|---:|
| crowd | 620,896,357 | 650,594,112 | 1.048 | 658,309,120 | 1.060 |
| far-coplanar | 620,108,091 | 648,878,144 | 1.046 | 656,867,328 | 1.059 |
| helmet | 640,366,671 | 669,272,128 | 1.045 | 677,380,096 | 1.058 |
| minification | 620,368,779 | 649,147,264 | 1.046 | 657,260,544 | 1.059 |
| normal-tangent | 639,908,546 | 668,713,792 | 1.045 | 676,790,272 | 1.058 |
| orrery | 625,768,011 | 655,372,416 | 1.047 | 662,372,352 | 1.058 |
| pbr-sweep | 620,175,579 | 648,953,152 | 1.046 | 656,932,864 | 1.059 |
| shadows | 620,171,029 | 648,937,280 | 1.046 | 656,867,328 | 1.059 |
| texture-wall | 620,634,171 | 669,403,728 | 1.079 | 1,312,030,720 | 2.114 |
| town | 720,808,670 | 749,594,128 | 1.040 | 758,317,056 | 1.052 |
| uv-grid | 625,700,523 | 654,472,768 | 1.046 | 662,503,424 | 1.059 |

This adapter's top three offenders are the first three below; the rest
are also seen.

- **1. `town`'s CPU frame: 26.2 ms against the 6 ms Budget**, and 17.7 ms
  on the M4 Pro. Its p95, 37.9 ms, is 1.45 times its median, where
  every other scene's on Vulkan is under 1.2. The dynamic TLAS takes 1.9 ms of it,
  over its 1.1 ms ceiling; on the orrery and the crowd it takes 0.59 and
  0.82 ms, 2 to 3 times the M4 Pro's. Where the rest goes is not yet
  measured.
- **2. `town`'s gate: 6.1 ms on Vulkan and 7.5 on DX12, over the 6 ms
  Budget**, which the M4 Pro breaches at 9.8. The lit pass, which costs
  the M4 Pro 8.7 ms, costs 3.3 here, both with rays on. After the lit
  pass, the two culling phases are the gate's largest part: 1.47 ms on Vulkan and 2.44 on
  DX12, against a depth pre-pass of 1.14 and 1.08.
- **3. Vulkan's shadow mask pass costs 1.1 to 3.7 times DX12's on the
  same GPU**, and more than 1.5 times on six scenes: 2,321 µs against
  1,454 on `town`, 525 against 142 on the orrery, 740 against 297 on the
  crowd, 421 against 174 on texture-wall; the shadows scene's, 1.11, is
  the closest.
  Both trace the same rays through the same ray queries; the cause is not
  yet measured.
- **DX12's allocator holds 2.1 times the engine's count on
  texture-wall**, 1.31 GB against 621 MB, where Vulkan's holds 1.08
  times and every other scene 1.04 to 1.06 on either backend. texture-wall
  binds 10,005 Textures, and DX12's allocator holds about 64 KB more per
  Texture than Vulkan's.
- **The M4 Pro's slow passes are not this adapter's.** Tonemapping, which
  reads 0.6 to 3.8 ms on the M4 Pro, reads 184 to 219 µs on every scene
  here; texture-wall's GPU frame, 15.9 ms there, is 1.1 ms here. Both
  point at the M4 Pro rather than at the passes.
- **The dynamic TLAS's build is not hidden here.** Metal runs it beside
  the previous frame's passes and reads 0 on the orrery and the crowd;
  this adapter reads 52 and 161 µs there, and 246 µs on `town`, under
  the 1 ms ceiling.


## `town`'s Baselines, ADR 0019's gate and the dynamic TLAS (2026-10-26, #207)

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release builds of
#207's second part. `praxis bench <dir> town` at 2560×1440 with 4x MSAA,
248 warm-up and 1,800 measured frames, each run twice: with shadow rays
on, then off. Five runs of `town`. Load average about 3, from system
processes, with no other session benching. GPU medians agree within
1.2% across runs; CPU medians read 17.7 to 20.6 ms. The Baseline is
the run with the lowest CPU. At this load, main's binary reads the same
CPU frame as the branch on uv-grid and pbr-sweep (207 to 231 µs against
225 to 232). Every scene's CPU frame reads 15 to 35% above its earlier
Baseline, the load's doing, and the counters are unchanged.

**town** on Apple M4 Pro, release, 2560x1440, 1800 measured frames

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 17,652.7 | 18,184.2 | — | — | — | — | 6,000.0 |
| Dynamic TLAS, CPU | 1,039.2 | 1,109.5 | — | — | — | — | 1,100.0 |
| GPU frame | 13,568.3 | 14,188.4 | — | — | — | — | 12,000.0 |
| Gate, rays off | 9,820.7 | 10,483.3 | — | — | — | — | 6,000.0 |
| Shadow-ray time | 3,228.0 | 3,984.7 | — | — | — | — | — |
| `bend_pass` | 0.0 | 0.0 | — | — | — | — | — |
| `blas_refit` | 0.0 | 0.0 | — | — | — | — | — |
| `culling_phase_one` | 1,513.0 | 1,585.4 | — | — | — | — | — |
| `culling_phase_two` | 897.8 | 936.2 | — | — | — | — | — |
| `depth_prepass` | 1,756.3 | 1,814.6 | — | — | — | — | — |
| `depth_pyramid_pass` | 250.4 | 266.8 | — | — | — | — | — |
| `dynamic_tlas_build` | 48.6 | 462.8 | — | — | — | — | 1,000.0 |
| `froxel_pass` | 517.2 | 556.3 | — | — | — | — | — |
| `lit_pass` | 8,714.4 | 9,186.1 | — | — | — | — | — |
| `phase_two_depth_pass` | 48.9 | 98.6 | — | — | — | — | — |
| `shadow_mask_pass` | 4,648.3 | 4,940.3 | — | — | — | — | — |
| `tonemapping_pass` | 3,825.2 | 4,499.8 | — | — | — | — | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 4 | — | — | — | — |
| `instances_drawn` | 137,730 | — | — | — | — |
| `triangles_drawn` | 5,552,214 | — | — | — | — |
| `instances_overflowed` | 0 | — | — | — | — |
| `uploaded_bytes` | 2,049,344 | — | — | — | — |
| `gpu_memory_bytes` | 616,555,272 | — | — | — | 8,000,000,000 |
| `arena_bytes_in_use` | 85,280 | — | — | — | — |
| `heap_allocations` | 10,478 | — | — | — | — |

The dynamic TLAS holds 10,000 Moving Instances. Shadow-ray time is each
frame's GPU frame with shadow rays on, less the same frame's with them
off. The GPU frame, the CPU frame and the gate are over their Budgets.

- **ADR 0019's gate breaches.** It runs from the start of culling phase
  one to the end of the lit pass, with rays off. It reads 9.6 to 10.3 ms
  over seven runs. It reads the same with the dynamic TLAS emptied (9.58
  ms) and with one frame in flight (9.58 ms). With rays off, the lit pass
  takes 5.15 ms, the depth pre-pass 1.76, the culling phases 1.52 and
  0.89, the Froxel pass 0.53 and the depth pyramid 0.25. As ADR 0019
  says, B and D are prototyped on `town` next: #296 and #297, which block
  M3's exit ([decision](https://github.com/gariasf/praxis/issues/17#issuecomment-6415601177)).
- **The dynamic TLAS's build hid behind the frame before.** Its
  submission comes before the frame's passes, and Metal ran it alongside
  the previous frame's lit and tonemapping passes. Timed from the end of
  that frame's shadow mask pass, it read 4.1 ms, and the GPU frame total
  counted that time twice: 17.6 to 18.1 ms. Timed from the end of that
  frame's last pass, as it now is, the build reads 48 µs and the total
  13.6 ms. Run alone, with one frame in flight, the build takes 665 µs.
  The build still slows the passes it runs beside: without it, the frame
  total with rays on read 12.8 ms. On the orrery and the crowd the build
  is now hidden entirely, and reads 0.
- **Ceilings.** The owner set them from these figures: 1.1 ms of CPU and
  1.0 ms of GPU build, both medians, which `compare` enforces on every
  scene. The CPU read 0.96 ms at load average 2.8 and 1.04 to 1.15 ms at
  3 to 3.6, so it sits near its ceiling under load.
- **Lit pass with rays on.** It reads 8.7 ms against 5.15 with rays off,
  and 8.66 ms with one frame in flight, so the 3.5 ms is not overlap. On
  the other Reference scenes, reading the mask cost 22 to 49 µs (#260).
  Where it goes on `town` is not yet measured.
- **Tonemapping reads 0.6 to 4 ms on every scene**, and 3.8 ms on `town`
  with one frame in flight. One full-screen pass should not take that
  long. It is outside the gate. Measured 2026-10-28 (#352, section
  above): it overlaps the lit pass, and its own cost is about 0.3 ms.
- **CPU frame.** 13 of its 17 ms are `queue.submit`, which wgpu-core
  30.0.1 spends walking the TLASes' per-Instance dependencies, not
  waiting on the GPU (#352, section above).

## The crowd's two forms (2026-10-26, #209)

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release builds of
#209's second part on `66b5b57`. `praxis bench <dir> crowd` and
`praxis bench <dir> --crowd-form refit` at 2560×1440 with 4x MSAA, 248
warm-up and 1,800 measured frames, four rounds alternating the forms.
Load average 4.2 to 5.5, with a browser playing video: GPU medians agree
within 1% across rounds.

Form (b) casts the 100 figures' shadows through 1,600 rigid per-bone
proxies (#208). Form (a) gives each figure a BLAS, refitted to its bent
shape each frame through wgpu-hal: 100 BLASes, 512,000 triangles. The
Budget for the update is 0.5 ms of CPU and 1.0 ms of GPU per frame
(criterion 57):

- **CPU** is `dynamic_tlas_cpu_microseconds`: the dynamic TLAS's
  recording and submit, plus in form (a) the refits' recording and their
  submission, with the frame's commands up to the bend pass.
- **GPU** is the build between its markers (`dynamic_tlas_build`), plus
  in form (a) the refits (`blas_refit`), from the end of the bend pass to
  the start of a marker that reads their scratch. The bend pass, the
  stand-in for skinning that both forms run, is outside it: 21 to 27 µs.

| Form, round | Update CPU µs, median (p95) | Refit GPU µs, median (p95) | Build GPU µs, median (p95) | Update GPU µs, medians |
|---|---|---|---|---|
| Proxies 1 | 322.5 (466.1) | — | 603.3 (670.5) | 603.3 |
| Proxies 2 | 318.6 (396.9) | — | 597.5 (661.0) | 597.5 |
| Proxies 3 | 325.5 (452.5) | — | 600.5 (667.8) | 600.5 |
| Proxies 4 | 325.0 (449.4) | — | 596.8 (757.2) | 596.8 |
| Refit 1 | 450.5 (628.2) | 1,114.4 (1,284.9) | 253.2 (314.0) | 1,367.6 |
| Refit 2 | 394.2 (511.9) | 1,119.8 (1,267.5) | 257.3 (313.8) | 1,377.1 |
| Refit 3 | 393.0 (503.8) | 1,123.3 (1,268.2) | 259.2 (314.9) | 1,382.5 |
| Refit 4 | 418.2 (608.8) | 1,117.4 (1,302.9) | 256.9 (324.4) | 1,374.3 |

- **Proxies: within both.** The build takes 597 to 603 µs, 60% of the
  GPU Budget, and the CPU 319 to 326 µs at the median, its p95s 397 to
  466 µs.
- **Refit: over the GPU Budget.** The refits alone take 1,114 to 1,123
  µs, about 2.2 ns a triangle with some 15 to 25 µs of fixed cost between
  their command buffers, and the update 1.37 to 1.38 ms. Its CPU is
  within the Budget at the median, 393 to 451 µs, and over it at p95, 504
  to 628 µs; that stage also submits the frame's commands up to the bend
  pass, which form (b) submits with the rest of the frame, so form (a)'s
  CPU reads a little high. The build, over 100 entries rather than 1,600,
  takes 253 to 259 µs.
- **The refit's timing holds.** With every refit recorded three times (a
  local patch), `blas_refit` read 3,332 and 3,333 µs against 1,127 and
  1,131 once, and the GPU frame grew by the same 2.2 ms: the marker
  waits for the refits.
- **Strict event sync costs nothing measurable: every figure moved within
  noise or fell.** Form (a) turns on
  wgpu-hal's strict event sync for the queue, which adds a wait to each
  command buffer and a command buffer to each submission. Forcing it on
  in form (b) (a local patch, three rounds alternating) moved the
  update's CPU from 297–315 µs to 287–292, the CPU frame from 727–756 µs
  to 711–723, and the GPU frame from 3,301–3,308 µs to 3,181–3,257. The
  build read 483 to 520 µs against 593 to 610: its first marker no longer
  starts while the frame before still runs. Part of form (a)'s lower
  build time is this, so form (b)'s build figure is an upper bound.
- **Beyond the update** (medians, round 2):

  | | Proxies | Refit |
  |---|---|---|
  | CPU frame µs | 756.6 | 590.2 |
  | GPU frame µs | 3,314.3 | 4,121.2 |
  | Shadow mask pass µs | 1,174.6 | 1,224.4 |
  | Uploaded B per frame | 322,192 | 22,192 |
  | Heap allocations | 2,078 | 861 |
  | GPU memory B | 516,623,171 | 564,006,507 |

  - Form (a) places no proxies, so its CPU frame is lower, by 90 to 200
    µs across the rounds.
  - Its rays traverse 100 BLASes of 5,120 triangles rather than 1,600
    proxies: the shadow mask pass takes 50 µs more.
  - Its GPU memory is 47.4 MB more: the BLASes are uncompacted and keep
    their index streams for the refit, plus the refits' scratch.
- **Shapes.** At 1280×720 the two forms' images differ in 5,398 of
  921,600 pixels, all along the shadows' outlines: the proxies' shadows
  are a little thinner than the figures'.
- **The rebuild form**, the oracle, rebuilds every BLAS through wgpu, the
  update every backend can run. Its rebuild and build are not timed. In
  one run its GPU frame spanned 9,303 µs, waits between submissions
  included, and its update's CPU took 709 µs.

The owner picked form (b), the proxies, for M7, as recommended
([decision](https://github.com/gariasf/praxis/issues/17#issuecomment-6410351140)). It fits both Budgets with 40% of the GPU's to spare, runs on
every backend, and needs no `unsafe`. Form (a) is over the GPU Budget by
0.37 ms on Metal alone, where it also needs strict event sync. Form (b)'s
costs are the thinner shadows, 16 of the dynamic TLAS's 10,000 entries
per figure, and the heap allocations wgpu-core makes per entry.

## Shadow rays: from the lit pass to a shadow mask (2026-10-23, #260)

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release builds of main at
`d12da41` and of #260's branch on it. `praxis bench` at 2560×1440 with 4x
MSAA, 248 warm-up and 1800 measured frames. Four binaries, each scene's
rays on and off for main and the branch, run interleaved in three rounds
in rotated order, on a machine otherwise idle. Medians over the rounds of
each run's GPU medians, in µs; the rounds agreed within 1% for every scene
but texture-wall, and within 5% for main's orrery, pbr-sweep and helmet.

| Scene | Lit pass, inline: rays on / off | Rays' cost, inline | Lit pass, mask: rays on / off | Phase two's depth pass | Shadow mask pass | Rays' cost, mask | GPU frame, inline → mask |
|---|---|---|---|---|---|---|---|
| far-coplanar | 2,822 / 312 | +2.53 ms | 333 / 311 | 12 | 989 | +1.05 ms | 3,555 → 2,074 |
| minification | 3,811 / 461 | +3.36 ms | 487 / 459 | 12 | 1,048 | +1.11 ms | 4,547 → 2,292 |
| uv-grid | 3,502 / 404 | +3.11 ms | 433 / 406 | 12 | 1,175 | +1.23 ms | 4,184 → 2,310 |
| orrery | 5,694 / 289 | +5.69 ms | 328 / 287 | 12 | 1,115 | +1.20 ms | 6,645 → 2,317 |
| normal-tangent | 8,203 / 439 | +7.80 ms | 466 / 438 | 12 | 1,347 | +1.40 ms | 8,943 → 2,545 |
| shadows | 11,233 / 439 | +10.81 ms | 462 / 435 | 12 | 1,881 | +1.94 ms | 12,012 → 3,136 |
| pbr-sweep | 9,454 / 404 | +9.07 ms | 448 / 404 | 12 | 1,368 | +1.44 ms | 10,152 → 2,528 |
| helmet | 35,313 / 712 | +34.61 ms | 760 / 711 | 12 | 2,520 | +2.59 ms | 36,226 → 4,207 |
| texture-wall | 3,387 / 966 | +0.13 ms | 585 / 963 | 17 | 2,388 | +0.34 ms | 15,623 → 15,756 |

- **The rays cost 2.4 to 13.4 times less from the mask.** The cost is
  rays on minus rays off on the GPU frame. The rays are not quite the same:
  inline, one per fragment, from its centroid, and skipped where the
  shading normal turns from the light; from the mask, one per pixel, from
  its nearest sample, and skipped where the face turns from the light. The
  mask's cost is phase two's depth pass (12 µs as timed; Metal overlaps it
  with its neighbours), the mask pass, and the lit pass's read of the mask:
  rays-off time plus 22 to 49 µs.
- **texture-wall** spends its 15 ms frame in its 10,000 Textures (#183). Its
  timestamps swing by up to 1 ms from round to round, so its rows are
  within noise.
- **Rays off.** The branch's GPU frame with rays off is main's within ±4
  µs on every scene but texture-wall, inside its spread, and the orrery.
  The mask pass does not run, and phase two's depth stays in the lit pass.
  ADR 0019's gate does not move.
- **The orrery's dynamic TLAS build** is now counted with rays off: about
  160 µs, the +160 µs on its GPU frame. Main measured the build from the
  previous frame's lit pass, which the build no longer waits for once
  nothing in the lit pass reads the TLAS, so the build read 0 with rays off
  on main, and with rays on on the branch. It now starts no earlier than
  the end of the previous frame's shadow mask pass, which traces the rays,
  and reads about 170 µs with rays on, main's 178.
- **Ray-query code in the lit pass costs even where it never runs.** A
  first version kept an inline fallback in `fs_main` for lights past the
  mask's bits, which no Reference scene reached. With that fallback the lit
  pass took 1,362 to 4,611 µs, against 429 to 754 µs without it (two
  rounds each on uv-grid, pbr-sweep, shadows and the helmet, at `470221e`).
  The owner chose a fixed capacity of 59 lights per pixel over the
  fallback, with the overflow counted and warned
  ([decision](https://github.com/gariasf/praxis/issues/17#issuecomment-6369952061)).
- **Memory and allocations.** The mask takes 29,491,200 bytes at 1440p,
  8 bytes per pixel. The two new passes add 43 or 44 heap allocations a
  frame (279 to 322 on most scenes).
- **`crowd`.** #274 added it after this A/B. Its M4 Pro Baseline is 12,861 µs
  on main at `0743418` and 3,283 µs on this branch on it: lit pass 11,339
  µs against a shadow mask pass of 1,174. These are Baselines, not an
  interleaved A/B.
- **`town`.** On #206's branch, with inline rays, the GPU frame median was
  92.7 ms at 1440p over its circling path, with 0.6 to 0.8 point lights
  reaching each lit pixel and 6 or 7 listed per Froxel (#206's session,
  2026-10-23). #207 measures it with the mask.

## The crowd's per-bone proxies (2026-10-23, #208)

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release builds of the
`crowd` scene without its shadow proxies (#274) and with them
(#276). `praxis bench <dir> crowd` at 2560×1440 with 4x MSAA,
248 warm-up and 1,800 measured frames, eight runs alternating the two
builds. Load average 5.4 to 10.3, from a peer session's build: CPU p95s
are noisy, GPU medians agree within 2% across runs.

The scene's 100 figures, 5,120 triangles each, sway in the fixed schedule
and are bent on the GPU every frame. Their Meshes have no BLAS. Form (b)
gives each figure's middle 16 bones, each with a shadow proxy of 168
triangles: **1,600 proxies, Moving Instances in the dynamic TLAS, 16% of
its 10,000 entries.** Every frame rebuilds the dynamic TLAS from them.

The Budget for the crowd's acceleration-structure update is 0.5 ms of CPU
and 1.0 ms of GPU per frame (decision on #17, item 14). CPU is the build's
recording plus its submit (`dynamic_tlas_cpu_microseconds`), GPU the build
between its markers (`dynamic_tlas_build`), as #204 times them:

| Run | Update CPU µs, median (p95) | Update GPU µs, median (p95) |
|---|---|---|
| 1 | 538.0 (1,282.4) | 521.5 (596.2) |
| 2 | 428.4 (588.1) | 543.6 (593.0) |
| 3 | 473.0 (902.5) | 539.3 (590.5) |
| 4 | 516.0 (1,143.6) | 526.2 (584.8) |
| 5 | 462.4 (894.0) | 527.2 (585.0) |

- **GPU: within the Budget.** The build takes 522 to 544 µs at the median
  and under 600 µs at p95, about half of the 1.0 ms.
- **CPU: at the Budget.** Medians of 428 to 538 µs straddle 0.5 ms; p95s
  run 588 to 1,282 µs on this loaded machine. On the orrery, 5 entries
  cost about 200 µs, most of it wgpu walking all 10,000 slots at every
  build (#204); 1,600 entries add the rest, and wgpu-core allocates a
  `Vec` per entry: the frame's heap allocations go from 295 to 2,034.
- **Beyond the update** (the quietest run of each build, medians):

  | | Without proxies | With proxies |
  |---|---|---|
  | CPU frame µs | 243.5 | 938.2 |
  | GPU frame µs | 7,414.6 | 12,860.9 |
  | Lit pass µs | 6,468.6 | 11,339.0 |
  | Uploaded B per frame | 2,192 | 322,192 |
  | Heap allocations | 295 | 2,034 |

  - The CPU frame grows by 695 µs: the update, plus placing 1,600
    proxies every Tick, propagating their World transforms and writing
    their Instances. In one frame that ran two Ticks, the update took
    465 µs (356 recording, 109 submitting); each Tick's sway placed the
    proxies in 21 to 22 µs and propagated their World transforms in 37 to
    39 µs; prepare wrote their Instances in 38 µs; and the frame's own
    encoding and submit grew by 95 and 94 µs. #204 found that wgpu-core
    walks a TLAS's entries at submit for each pass that sets a bind group
    holding it, four passes here.
  - The lit pass grows by 4.9 ms. Without proxies the figures cast no
    shadow and the dynamic TLAS is empty, which the lit pass skips; with
    them, every lit pixel's sun ray that the static TLAS lets through is
    traced against 1,600 proxies. This is shadow-ray cost, outside the
    update's Budget; form (a) traces 100 refitted BLASes of 5,120
    triangles instead.
  - Uploads grow by 320,000 B a frame: each proxy's 128-byte Instance
    and its 72-byte TLAS entry.
- **Shapes.** A proxy is an ellipsoid along its bone, 0.85 of the
  figure's radius across, so a figure's shadow is a chain of overlapping
  ellipsoids a little thinner than the figure. The end bones' proxies
  stop short of the head and the foot: reaching 2.8 cm past the head, the
  top one spotted every figure's top with its own shadow.

#209 times form (a), refit through wgpu-hal, against the same Budget, and
puts both forms to the owner, who picks M7's.

## The dynamic TLAS's build on the orrery (2026-10-23, #204)

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release builds of
`3b79286` (#266) and of main at `79cdabb` (#264). `praxis bench` at
2560×1440 with 4x MSAA, 248 warm-up and 1800 measured frames. The machine
carried an antivirus scan and the owner's interactive use: GPU p95 ran 1.0
to 1.3 times the median; the orrery's three runs agreed within 1.5% at the
median.

The orrery's five planets are Moving Instances: every frame rebuilds the
dynamic TLAS from their five entries.

| Metric | Three runs, medians (p95) |
|---|---|
| Dynamic build on the CPU: recording, `finish` and its own submit (`dynamic_tlas_cpu_microseconds`) | 198.5 (269), 197.3 (252), 203.0 (291) µs |
| Dynamic build on the GPU (`dynamic_tlas_build`) | 174.2 (209), 175.9 (213), 174.7 (194) µs |
| GPU frame total, the build included | 2,588, 2,590, 2,553 µs |

- **Recording.** wgpu 30 walks every one of the TLAS's 10,000 entries at
  each build, filled or not, about 12 ns each (#202), which is most of the
  CPU time with 5 entries. ADR 0020's figure for 10,000 filled entries, 0.64
  to 0.92 ms of recording plus submit, comes from a scratch program; #207
  measures it on `town`.
- **Not in the CPU figure.** Each of the four passes that set the
  scene-geometry bind group makes the frame's own submit walk the dynamic
  TLAS's entries again, in wgpu-core's validation and Metal's residency
  set.
- **What timing the build adds.** Its own command buffer and submit and
  the two markers cost the orrery 73 heap allocations a frame (317 to 390)
  and, in an interleaved A/B of `0bfd654` against main (two rounds; the
  last commit only moves where the frame records the build), 27 µs of CPU
  frame. The GPU frame total grew by 177 µs, the build it now counts.
  Scenes where nothing moves build nothing, record no markers and did not
  change: helmet −3 µs of CPU and +6 µs of GPU, uv-grid −12 and −2, shadows
  +12 and −1.
- **Frames in flight.** The marker before the build can run while the
  previous frame still traces rays against the TLAS, and the build waits
  for it. Timed from that marker's end alone, the build read 1,650 to 1,750
  µs, the previous frame's whole lit pass. It now starts at the later of the
  marker's end and the previous frame's lit-pass end.
- **Bracketing on Metal.** Empty compute passes write no timestamps, and
  passes that touch no TLAS do not bracket the build, which Metal runs
  alongside them; markers that query the TLAS do. In a probe on an idle GPU
  at low clocks, the build took 0.14 ms at 0 entries, 0.27 at 5, 1.37 at
  1,000 and 2.26 at 10,000 (debug build, medians of 40).

## Anisotropy candidates (2026-10-22, #184)

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release builds of
`064d2ba`, and of `064d2ba` with a measurement patch, not committed, that
gives each Material Channel its own sampler and reads each sampler's
anisotropy from an environment variable. `praxis bench <dir> minification`
at 2560×1440 with 4x MSAA, 248 warm-up and 1800 measured frames, three
rounds, each running every variant once in rotated order. Load average
about 13, from peer sessions' builds and a browser.

minification's floor samples three Textures, in the albedo, normal and
metallic-roughness Channels; occlusion and emissive sample Layer 0's 1×1
Textures. The colour Channels are albedo and emissive, the data Channels
normal, metallic-roughness and occlusion, as `MaterialPool::new` splits
them by encoding. The lit pass does all the sampling; GPU frame totals
are given too, since the tonemapping pass's timestamps overlap the lit
pass on Apple's tile-based GPU.

| Variant | Anisotropy: albedo, normal, metallic-roughness, occlusion, emissive | Lit pass µs (3 rounds) | GPU frame µs (3 rounds) |
|---|---|---|---|
| `064d2ba`: one sampler | 16, 16, 16, 16, 16 | 556.7 / 557.2 / 557.5 | 1,274.3 / 1,274.2 / 1,274.8 |
| Patch at 16x | 16, 16, 16, 16, 16 | 557.5 / 558.0 / 557.8 | 1,272.7 / 1,272.8 / 1,274.9 |
| 8x | 8, 8, 8, 8, 8 | 446.5 / 446.9 / 454.0 | 1,162.6 / 1,163.2 / 1,193.5 |
| 4x | 4, 4, 4, 4, 4 | 368.8 / 368.0 / 369.5 | 1,085.6 / 1,082.9 / 1,088.5 |
| 2x | 2, 2, 2, 2, 2 | 341.7 / 343.9 / 346.3 | 1,058.5 / 1,060.6 / 1,067.8 |
| 1x | 1, 1, 1, 1, 1 | 343.0 / 343.9 / 340.0 | 1,059.1 / 1,061.8 / 1,057.1 |
| Colour 16x, data 8x | 16, 8, 8, 8, 16 | 486.6 / 483.6 / 482.9 | 1,207.7 / 1,199.7 / 1,199.2 |
| Colour 16x, data 4x | 16, 4, 4, 4, 16 | 430.6 / 430.8 / 432.9 | 1,147.1 / 1,147.5 / 1,155.7 |
| Colour 16x, data 2x | 16, 2, 2, 2, 16 | 399.1 / 397.4 / 401.5 | 1,115.2 / 1,113.1 / 1,121.4 |
| Colour 16x, data 1x | 16, 1, 1, 1, 16 | 392.1 / 384.7 / 383.8 | 1,128.0 / 1,101.0 / 1,099.5 |
| Albedo 1x | 1, 16, 16, 16, 16 | 470.2 / 473.5 / 473.1 | 1,186.0 / 1,193.9 / 1,192.0 |
| Occlusion and emissive 1x | 16, 16, 16, 1, 1 | 561.2 / 557.3 / 557.5 | 1,285.2 / 1,274.1 / 1,274.4 |

- The patch alone costs nothing: five samplers at 16x time as one, and
  their capture matches `064d2ba`'s Golden pixel for pixel.
- 16x costs 214 µs of the lit pass, 38% of it (557.2 against 343.0 µs at
  1x, medians of the rounds' medians), as M2's exit measured in a single
  pass without MSAA (222 µs). Over 1x, 2x costs nothing, 4x 26 µs and 8x
  104 µs. GPU frame totals move by the same amounts, within 2.1 µs.
- The 1×1 Textures' anisotropy costs nothing. The Channels' costs
  overlap: albedo alone at 16x costs 42 µs over 1x everywhere, but
  dropping it to 1x from 16x everywhere saves 84 µs.
- Against `064d2ba`: 8x saves 110 µs; colour 16x with data 8x 74 µs, data
  4x 126 µs, data 2x 158 µs. Data 4x saves 1.1% of ADR 0013's 12 ms GPU
  Budget.

Captures at the Golden's size, 640×360, against `064d2ba`'s Golden:

| Variant | Pixels changed | Beyond 1 step | Largest step |
|---|---|---|---|
| 8x | 18,892 | 7,229 | 10 |
| Colour 16x, data 8x | 221 | 0 | 1 |
| Colour 16x, data 4x | 18,119 | 4,099 | 29 |
| Colour 16x, data 2x | 74,414 | 37,702 | 102 |

Every difference sits in the band below the horizon, where a pixel's
footprint is longer, in texels, than the lower anisotropy times its
width. The Golden's size hides some of it: colour 16x with data 8x
changes 221 pixels at 640×360 (0.1%) but 273,950 at 1440p (7.4%), whose
pixels sample finer mips that still hold the bumps.

Images, on the `pr-images` branch at `2a87cbb`: `064d2ba`'s
[Golden](https://raw.githubusercontent.com/gariasf/praxis/2a87cbb5201eda665cb141a7ccf8a722226677d2/257/golden.png)
and, at 640×360, [8x](https://raw.githubusercontent.com/gariasf/praxis/2a87cbb5201eda665cb141a7ccf8a722226677d2/257/8x.png),
[colour 16x, data 4x](https://raw.githubusercontent.com/gariasf/praxis/2a87cbb5201eda665cb141a7ccf8a722226677d2/257/colour-16x-data-4x.png)
and [colour 16x, data 2x](https://raw.githubusercontent.com/gariasf/praxis/2a87cbb5201eda665cb141a7ccf8a722226677d2/257/colour-16x-data-2x.png),
each with its difference image against the Golden
([8x](https://raw.githubusercontent.com/gariasf/praxis/2a87cbb5201eda665cb141a7ccf8a722226677d2/257/8x-difference.png),
[data 4x](https://raw.githubusercontent.com/gariasf/praxis/2a87cbb5201eda665cb141a7ccf8a722226677d2/257/colour-16x-data-4x-difference.png),
[data 2x](https://raw.githubusercontent.com/gariasf/praxis/2a87cbb5201eda665cb141a7ccf8a722226677d2/257/colour-16x-data-2x-difference.png))
and against 16x at 1440p, halved
([8x](https://raw.githubusercontent.com/gariasf/praxis/2a87cbb5201eda665cb141a7ccf8a722226677d2/257/8x-difference-1440p.png),
[data 4x](https://raw.githubusercontent.com/gariasf/praxis/2a87cbb5201eda665cb141a7ccf8a722226677d2/257/colour-16x-data-4x-difference-1440p.png),
[data 2x](https://raw.githubusercontent.com/gariasf/praxis/2a87cbb5201eda665cb141a7ccf8a722226677d2/257/colour-16x-data-2x-difference-1440p.png));
and 1440p zooms of the band, doubled, beside the reference below:
[the highlight](https://raw.githubusercontent.com/gariasf/praxis/2a87cbb5201eda665cb141a7ccf8a722226677d2/257/highlight-zoom.png)
and [the checker](https://raw.githubusercontent.com/gariasf/praxis/2a87cbb5201eda665cb141a7ccf8a722226677d2/257/checker-zoom.png).

Image quality, against a reference: 16x at 7680×4320, 9 samples per
pixel, box-filtered to 2560×1440 in linear light. RMSE in 8-bit steps over
the band (rows 484 to 759, from 3 rows below the floor's far edge, at
row 481 across the middle of the frame), and within it over the
highlight's columns (1000 to 1559) and the checker either side (0 to 899
and 1660 to 2559):

| Variant | Band | Highlight | Checker |
|---|---|---|---|
| 16x | 16.09 | 33.89 | 2.98 |
| 8x | 16.72 | 35.05 | 3.44 |
| 4x | 16.89 | 34.50 | 5.40 |
| 2x | 16.04 | 28.98 | 9.54 |
| 1x | 19.39 | 29.92 | 15.15 |
| Colour 16x, data 8x | 16.63 | 35.02 | 2.98 |
| Colour 16x, data 4x | 16.34 | 34.32 | 3.06 |
| Colour 16x, data 2x | 13.61 | 28.09 | 3.55 |
| Colour 16x, data 1x | 13.42 | 27.65 | 3.59 |

- The checker is fixed in texture space, so the reference and every
  variant show it in the same place, and its error measures filtering:
  albedo at 16x is closest. 8x is 15% further off, 4x 81%. Keeping albedo
  at 16x keeps the checker within 3% (data 4x) or 19% (data 2x, through
  the bumps' shading).
- The highlight is not a fair test: the reference resolves glints that a
  one-sample render cannot place, so RMSE favours the smoother images
  there. By eye, the reference shows a narrow streak, the light's mirror
  image and glints below it; 16x and 8x a broad white column with
  single-pixel sparkle along its flanks and lower end, specular aliasing:
  their normal and roughness come from a finer mip, whose Toksvig
  widening covers the footprint's width but not its length. Data
  Channels at 4x or 2x show a smoother, narrower V without the sparkle.
  No variant shows the mirror image: Toksvig widens one lobe and keeps no
  sharp peak.

The owner chose colour Channels at 16x and data Channels at 4x on
2026-10-22, judging the images on #257; the Goldens and Baselines there
are that sampler's.

## M2 exit (2026-10-19, #154)

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release build of
`507cff5`. `praxis bench` at 1440p, 248 warm-up and 1800 measured frames
per scene, one pass (the main pass), no MSAA yet. Load average 2.4 to 3.2
during the runs.

### Measurements

Three runs of `praxis bench <dir>` (medians; counters identical on every
run). The Baselines are a fourth run, the quietest (load 2.4 to 2.6). The
llvmpipe Baselines are CI's `bench-results` artifact from run
39276496878, with every counter unchanged.

| Scene | CPU median µs (3 runs) | CPU p95 µs (3 runs) | GPU main pass µs (3 runs) | Draws | Uploaded B | GPU memory B | Arena B in use | Heap allocations |
|---|---|---|---|---|---|---|---|---|
| helmet | 24.8 / 24.6 / 24.8 | 34.1 / 29.0 / 28.4 | 355.6 / 357.1 / 357.1 | 11 | 288 | 67,902,524 | 675,852 | 62 |
| pbr-sweep | 27.7 / 23.1 / 23.4 | 37.7 / 31.3 / 29.8 | 166.1 / 168.8 / 169.4 | 10 | 288 | 48,329,012 | 60,740 | 62 |
| uv-grid | 26.5 / 22.7 / 22.3 | 38.9 / 34.8 / 29.7 | 218.0 / 219.1 / 220.8 | 1 | 288 | 53,916,836 | 136 | 60 |
| normal-tangent | 28.0 / 22.5 / 22.1 | 48.3 / 27.0 / 26.8 | 199.2 / 202.5 / 202.3 | 1 | 288 | 67,897,916 | 204,812 | 60 |
| far-coplanar | 21.8 / 23.4 / 23.4 | 34.0 / 29.9 / 27.6 | 400.7 / 403.9 / 403.8 | 2 | 288 | 48,325,940 | 136 | 61 |
| orrery | 28.6 / 24.8 / 24.9 | 40.7 / 37.2 / 38.2 | 80.7 / 80.7 / 80.9 | 5 | 928 | 53,918,372 | 60,740 | 62 |
| minification | 24.3 / 22.9 / 23.0 | 40.4 / 39.5 / 31.5 | 474.9 / 475.7 / 478.1 | 1 | 288 | 48,586,628 | 136 | 60 |
| texture-wall | 1,782.9 / 1,373.7 / 1,139.2 | 2,054.9 / 1,851.0 / 1,772.7 | 445.1 / 444.8 / 444.6 | 100 | 288 | 48,866,356 | 136 | 64 |

Against M1's Baselines: GPU memory fell from 687–690 MB to 48–68 MB (90
to 93% less), with the 640 MiB of Channel arrays gone (#101). The
helmet's main pass fell from 598 to 357 µs (the cooked Mesh, #102 and
#151); pbr-sweep's, uv-grid's and orrery's fell 15 to 17%,
normal-tangent's 5% and far-coplanar's 1%. Heap allocations fell from
73–75 to 60–64; uploaded bytes are unchanged.

Noise floor across the three runs: GPU medians within 2.0% (pbr-sweep);
CPU medians within 5.9 µs on every scene but texture-wall, whose median
fell from 1,783 to 1,139 µs over the three runs (1,156 µs in the fourth).
texture-wall's GPU p95 jumps to 1.6–3.5 ms, against a 445 µs median, in
four of six runs: runs 1 and 2 here, this exit's verify and #153's
Baseline.

Against ADR 0013's Budgets, texture-wall's CPU frame is 19–30% of 6 ms
at the median and 30–34% at p95. Every other scene's is at most 28.6 µs
median and 48.3 µs p95 (0.5% and 0.8%). The worst GPU main pass,
minification's, uses 4.0% of 12 ms, and the worst GPU memory, the
helmet's, 0.85% of 8 GB.

### ADR 0017's gate, quiet

The four runs also log texture-wall's 10,005-entry bind-group build:

| Run | Build ms | Frame median / p95 ms | Build + frame median / p95 ms |
|---|---|---|---|
| 1 | 2.96 | 1.78 / 2.05 | 4.74 / 5.01 |
| 2 | 2.55 | 1.37 / 1.85 | 3.93 / 4.40 |
| 3 | 2.97 | 1.14 / 1.77 | 4.11 / 4.74 |
| 4 | 2.64 | 1.16 / 1.81 | 3.80 / 4.45 |

On a quiet machine the build plus one frame fits the 6 ms critical path
in every run, median and p95: at most 4.74 ms at the median and 5.01 ms
at p95. At load 6 to 10 (#153) it sat at the line.

### Scene setup after cooking

`State::new` per scene, from its `state_init` span (a local patch logs
span closes; not committed): the median of five launches of `praxis
bench <dir> --ci <scene>` per build, interleaved, at load 1.9 to 2.7.
M1's build (`610fb4f`) decodes glTF and PNG sources. M2's dev launch
checks the Cook keys of what the scene loads, then loads its Blobs. M2's
shipping build (`--no-default-features`) only loads Blobs.

| Scene | M1, from sources ms | M2 dev launch ms | M2 shipping build ms |
|---|---|---|---|
| helmet | 146.0 | 39.7 | 19.8 |
| pbr-sweep | 12.5 | 5.3 | 4.1 |
| uv-grid | 16.1 | 14.5 | 3.7 |
| normal-tangent | 81.3 | 11.0 | 8.1 |
| far-coplanar | 11.3 | 2.9 | 2.6 |
| orrery | 17.6 | 14.5 | 3.6 |
| minification | | 4.7 | 3.1 |
| texture-wall | | 1,330 | 904 |

The shipping build sets the helmet up in 86% less time than M1 and
normal-tangent in 90% less: no PNG decode. `Renderer::new`, which at M1
created the 640 MiB of Channel arrays, fell from 10–11 ms to 1.7 ms. The
dev launch's Cook-key check costs the rest: about 20 ms to re-read and
hash SciFiHelmet's 30 MB of sources, 11 ms to regenerate and hash the
procedural 2048² grid (uv-grid, orrery), and 426 ms for texture-wall's
10,101 Assets.

texture-wall's setup is file opens. In `sample` (below), 1,546 of its
2,070 setup samples are in `open`: an index entry and a Blob for each of
its 10,000 Textures, and each index entry again in the dev launch's
check. Texture creation is 80 samples.

### Where the cost goes

- **GPU memory.** The two fixed Arenas (`ARENA_CAPACITY_BYTES`, 16 MiB
  each, 33,554,432 B) are 49% (helmet) to 69% (pbr-sweep) of each scene's
  GPU memory, with at most 675,852 B in use (helmet). The 1440p
  Depth32Float target is 14,745,600 B. BC Textures with full mip chains
  are 19.6 MB for the helmet's or NormalTangentTest's four 2048²
  Textures, 5.6 MB for the grid and 0.5 MB for texture-wall's 10,000.
- **GPU time.** minification's main pass is the most expensive, at
  478 µs. An interleaved A/B, three rounds, with the material sampler's
  anisotropy clamped to 1 (local patch): minification 478 → 256 µs,
  helmet 357 → 356 µs, normal-tangent 202 → 202 µs. 16x anisotropic
  filtering of the grazing ground costs 222 µs, 47% of the pass.
  texture-wall's 445 µs includes 347 µs for its 10,005 bound Textures
  (#153). far-coplanar's 401 µs is two planes covering most of the frame,
  as at M1.
- **CPU, per frame.** `sample` at 1 ms over `praxis bench` runs, at load
  2.6 to 5.3:
  - every scene but texture-wall, four runs, 11,128 main-thread samples:
    9,278 (83%) wait in `Device::poll`, outside the CPU frame time. About
    1,605 are per-frame work. `CommandEncoder::finish` is 1,096 of them
    (68%): 776 encode the render pass (273 open Metal's render encoder,
    116 reset the bench's timestamp queries), and 228 are the frame's
    upload copies (153 open a blit encoder, #140). `Queue::submit` is 304
    (19%) and `State::update` 134 (8%);
  - texture-wall, two runs, 7,342 main-thread samples: 4,424 (60%) are
    `Renderer::render`, about 1.1 ms a frame. Metal's `set_bind_group`
    adds every bound Texture to the command buffer's resource list
    (`useResource`): 2,205 (50%). wgpu-core's per-Texture tracking
    (usage-scope merges, barriers, initialisation) is 1,252 (28%), and
    `Queue::submit` 503 (11%).

### Top three offenders

Ranked by their share of a Budget, one per budgeted metric:

1. **texture-wall's 10,005 bound Textures, 1.1–1.8 ms of CPU per frame**
   (19–30% of the 6 ms Budget at the median, up to 34% at p95), half of
   it Metal's per-Texture `useResource`. They also cost 347 µs of its GPU
   main pass, and texture-wall alone shows GPU p95 jumps. #183
2. **minification's 16x anisotropic filtering, 222 µs of GPU time** (47%
   of its 478 µs main pass, the costliest; 1.9% of the 12 ms Budget).
   #184
3. **The fixed Arenas, 33.5 MB of GPU memory** (0.42% of the 8 GB Budget;
   49–69% of each scene's GPU memory), at most 2% in use. Their capacity
   is provisional until M4's Budget split. #185

M1's third offender, the frame's upload copies (#140), is still 14% of
the per-frame CPU of every scene but texture-wall: about 3 µs a frame.

### Also seen

- The bench sums every Texture's mip chain for the frame stats each frame
  (`MaterialPool::gpu_memory_bytes`): 613 of texture-wall's samples, about
  150 µs a frame. It runs outside the CPU frame time, and only the bench
  and capture call it, but it grows with the Textures resident.

## ADR 0017's gate: 10,000 bound Textures (2026-10-19, #153)

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release build of the
#153 branch. `praxis bench <dir> texture-wall` at 1440p, three rounds,
each also running two variants built from local patches, not committed:
*rebuild every frame* rebuilds the material bind group in every frame,
and *sampled only* loads just the 100 Textures the quads sample. Load
average 6.3 to 9.9, from other processes.

texture-wall binds 10,005 Textures: its 10,000 in the albedo Channel and
each Channel's Layer 0. The variant binds 105 and renders the same image.
The build is timed once per process, as the scene loads (`material bind
group built` in the log).

| Round | Build at load ms | Frame median / p95 ms | Build + frame median / p95 ms | Rebuild every frame, median / p95 ms | Sampled only: frame median µs, GPU main pass µs |
|---|---|---|---|---|---|
| 1 | 2.90 | 2.20 / 3.12 | 5.10 / 6.02 | 3.14 / 3.75 | 41.9, 97.6 |
| 2 | 3.02 | 2.09 / 2.53 | 5.11 / 5.55 | 2.89 / 3.57 | 40.6, 97.5 |
| 3 | 2.53 | 2.12 / 3.40 | 4.65 / 5.93 | 3.83 / 7.61 | 38.4, 97.7 |

**The gate is at the line.** The build plus one frame fits ADR 0013's
6 ms critical path at the median in every round, and at p95 in two of
three (6.02 ms in round 1). An earlier set of three rounds, at load 6 to
12, gave builds of 3.3 to 4.4 ms and p95 sums of 6.8 to 9.3 ms, all over.
The decision on ADR 0017's fallbacks is on #173.

Binding the Textures costs more than rebuilding them. With no rebuild,
10,005 bound Textures cost 2.1 to 2.2 ms of CPU per frame (35 to 37% of
6 ms) against 38 to 42 µs with 105, about 0.21 µs per Texture per frame,
and 347 µs of GPU per main pass (445 against 98). A build costs 0.25 to
0.30 µs per Texture.

Where the frame goes, from `sample` at 1 ms over 3 s of the bench, 400
main-thread samples recording and submitting frames: 181 (45%) in
wgpu-hal's Metal `set_bind_group`, which calls `useResource` once per
bound Texture; 66 (17%) in wgpu-core merging each Texture into the
pass's usage scope and tracking its barriers; 37 (9%) in
`Queue::submit`.

Cooking texture-wall cold writes 20,202 files (its 10,101 Assets), each
flushed to the drive before it is renamed into place (`File::sync_all`,
`F_FULLFSYNC` on macOS, 3 to 5 ms per file here): 84 s on the M4 Pro.
CI's `praxis cook` of every scene takes 40 s on Linux.

## Helmet main pass: the Mesh, not the Textures (2026-10-19, #102)

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release builds of
`7d271dc` plus one local measurement patch per variant, none committed.
`praxis bench <dir> helmet` at 1440p, as M1 exit, run interleaved: each
round runs every variant once. Load average 9 to 18.

The helmet scene draws SciFiHelmet 11 times: the textured helmet and a
factor sweep of 10 copies on 1×1 neutral Layers. Only one draw samples the
2048² Textures. The runtime still loads the Mesh from glTF: 23,358
triangles on 70,074 vertices, three per triangle, so no vertex is shared.
pbr-sweep draws a 2,304-triangle indexed sphere 10 times, covering 0.64× the
pixels the helmets cover.

| Variant | Patch | GPU main pass µs (3 runs) |
|---|---|---|
| RGBA8, no mips | the build before #150 (`da8c5e9`) | 578.4 / 578.3 / 578.1 |
| Full mip chain | none: BC7, BC5, BC4 with mips (#150) | 581.9 / 581.7 / 581.0 |
| Mip 0 only | sampler `lod_max_clamp: 0.0` | 579.7 / 579.8 / 579.4 |
| Cooked Mesh | the helmet's Mesh Blob decoded into today's vertex | 357.8 / 357.4 / 357.7 |

The helmet's main pass is within 0.7% across RGBA8, mip 0 and the full
chain, so mips are not its cost. M1's RGBA8 Baseline was 598.2 µs, at
higher load (M1 exit). The cooked Mesh (#160: welded, then re-split where
tangents differ, to 14,127 vertices, ordered for vertex cache and
overdraw) takes 224 µs off, 38.5%.

Where the time goes, from a second interleaved set of three rounds (full
chain 581.7 / 580.8 / 582.2). *Flat* returns the Material's base colour
before any Texture sample or light; *vertex only* moves every vertex
outside the clip volume, so nothing rasterises. The floor is the vertex-only
variant on pbr-sweep (48.7 / 54.8 / 48.7, run after the first set): clears
and draw overhead.

| Stage, µs (round-1 medians) | glTF Mesh | Cooked Mesh |
|---|---|---|
| Whole pass | 581.7 | 356.5 |
| Floor | 49 | 49 |
| Vertex (vertex only − floor) | 161 | 45 |
| Raster and depth (flat − vertex only) | 172 | 68 |
| Shading (whole − flat) | 200 | 195 |

Welding and the cooker's ordering remove 116 µs of vertex work and 104 µs of
raster and depth. Shading is unchanged. With the cooked Mesh the helmet
pass is still 2.1× pbr-sweep (169 µs), most of it shading.

**Cause:** the runtime's unwelded glTF Mesh, drawn 11 times. #151, which
renders every Reference scene from its Mesh Blob, takes it. #151 also packs
the vertex to 28 bytes, which this measurement does not include.

## M1 exit (2026-10-14, #123)

Apple M4 Pro, macOS 27.0.1, wgpu 30.0.1 on Metal, release build of
`c340c94`. `praxis bench` at 1440p, 248 warm-up and 1800 measured frames
per scene, one pass (the main pass), no MSAA yet. Load average 5 to 6 during
the runs, with a browser holding about two cores.

### Measurements

Three runs of `praxis bench <dir>` (medians; counters identical on every
run). The Baselines are a fourth run, the quietest; far-coplanar's are new
(it had none on either adapter, and CI now benches it):

| Scene | CPU median µs (3 runs) | CPU p95 µs (3 runs) | GPU main pass µs (3 runs) | Draws | Uploaded B | GPU memory B | Heap allocations |
|---|---|---|---|---|---|---|---|
| helmet | 44.0 / 52.5 / 56.2 | 111.6 / 242.9 / 183.7 | 602.8 / 606.4 / 603.1 | 11 | 288 | 689,751,040 | 75 |
| pbr-sweep | 30.9 / 26.2 / 26.9 | 62.2 / 52.2 / 48.0 | 199.7 / 200.5 / 199.9 | 10 | 288 | 687,173,920 | 75 |
| uv-grid | 28.8 / 33.5 / 31.8 | 53.1 / 85.1 / 72.1 | 263.8 / 266.3 / 267.0 | 1 | 288 | 687,169,312 | 73 |
| normal-tangent | 28.8 / 25.7 / 29.3 | 59.0 / 58.6 / 60.9 | 212.0 / 212.2 / 212.0 | 1 | 288 | 687,169,312 | 73 |
| far-coplanar | 31.5 / 30.1 / 43.3 | 70.1 / 97.5 / 157.3 | 405.5 / 405.2 / 405.7 | 2 | 288 | 687,170,848 | 74 |
| orrery | 40.4 / 25.3 / 35.5 | 192.5 / 44.7 / 129.2 | 99.5 / 97.1 / 99.3 | 5 | 928 | 687,170,848 | 75 |

Against M0: uploaded bytes fell from 416–1,856 to 288 per frame (#121's
persistent Instance slots: camera motion writes no Instance) and heap
allocations from 81–83 to 73–75; GPU medians are within 1.2% of M0's.

Noise floor across the three runs: GPU medians within 2.5% (orrery);
CPU medians within 15.1 µs (orrery), above M0's 5.4 µs and above the
compare's 10 µs floor, so a local `praxis compare` of CPU times fails on
noise at this load. Helmet, benched first in each process, runs 30–38 µs
when benched after pbr-sweep: part of its spread is warm-up of the
process, not the scene.

Against ADR 0013's Budgets the main-thread critical path (the CPU frame)
is at most 56.2 µs median and 242.9 µs p95, 0.9% and 4.0% of 6 ms; the worst
GPU main pass uses 5.1% of 12 ms and the worst GPU memory 8.6% of 8 GB.

### Where the cost goes

- **GPU memory.** Unchanged from M0: the five 2048² RGBA8 material
  Channel arrays are 671,088,640 B, 97.3–97.7% of each scene's GPU memory.
- **GPU time.** The helmet's main pass is 3.0× pbr-sweep's at about the
  same draw count. far-coplanar, new, is second at 405 µs: two 4 km planes
  each covering most of the 1440p frame.
- **CPU, per frame.** `sample` at 1 ms over four `praxis bench` runs of
  all six scenes; 11,259 main-thread samples:
  - 9,858 (87.6%) wait in `Device::poll`: the bench waits for each frame,
    and CPU frame time excludes the wait;
  - 520 (4.6%) are scene setup (`State::new`), 264 of them glTF load with
    PNG decode;
  - about 851 are per-frame work. Of those, `CommandEncoder::finish` is
    517 (61%): 353 encode the render pass (152 open Metal's render
    encoder), 120 are the frame's upload copies (77 open a blit encoder);
    `Queue::submit` is 195 (23%); `State::update` 89 (10%: Ticks 31,
    upload 27, prepare 20).

### Top three offenders

Ranked by their share of a Budget, one per budgeted metric:

1. **Material Channel arrays, 640 MiB of GPU memory** (8.4% of the 8 GB
   Budget). Carried over from M0. #101
2. **Helmet main pass, 598–606 µs of GPU time** (5.0% of the 12 ms
   Budget, 3.0× pbr-sweep). Carried over from M0. #102
3. **Per-frame upload copies open a blit pass, 14% of per-frame CPU**
   (the largest per-frame cost the engine controls; M0's `write_buffer`
   offender, #103, moved here when uploads went through the frame's
   staging ring). #140

## M0 exit (2026-10-13, #38)

Apple M4 Pro, macOS, wgpu 30.0.1 on Metal, release build of
`ac2bdec`. `praxis bench` at 1440p, 248 warm-up and 1800 measured frames
per scene, one pass (the main pass), no MSAA yet. Load average 11 during the runs.

### Measurements

Three runs of `praxis bench <dir>` (medians; counters identical on every
run). The helmet Baseline is run 1 as written; the other Baselines predate
these runs and pass `compare` against all three (CPU warns of 6–8%, under
the 10 µs floor):

| Scene | CPU µs (3 runs) | GPU main pass µs (3 runs) | Draws | Uploaded B | GPU memory B | Heap allocations |
|---|---|---|---|---|---|---|
| helmet | 36.9 / 36.7 / 37.0 | 602.6 / 602.4 / 602.9 | 11 | 1,856 | 689,745,136 | 83 |
| pbr-sweep | 31.0 / 35.3 / 30.0 | 199.5 / 199.2 / 198.2 | 10 | 1,712 | 687,168,016 | 83 |
| uv-grid | 35.0 / 29.6 / 34.8 | 264.0 / 264.4 / 265.5 | 1 | 416 | 687,168,016 | 81 |
| normal-tangent | 28.6 / 33.2 / 28.6 | 213.8 / 213.6 / 213.5 | 1 | 416 | 687,168,016 | 81 |

Noise floor across the three runs: GPU medians within 0.66%, CPU medians
within 5.4 µs. Against ADR 0013's Budgets the worst scene uses 0.6% of the
6 ms CPU frame, 5.0% of the 12 ms GPU frame and 8.6% of the 8 GB of GPU
memory.

### Where the cost goes

- **GPU memory.** The five material Channel arrays are 2048² RGBA8, eight
  Layers, one mip level each (`src/assets/material.rs`): 5 × 128 MiB =
  671,088,640 B, 97.3–97.7% of each scene's GPU memory, for at most a few used
  Layers. The rest: the 1440p Depth32Float target (14,745,600 B) and
  buffers (1.3 MB; 3.9 MB with the helmet).
- **GPU time.** The helmet's main pass costs 2.3–3.0× the other scenes' with
  about the same draw count as pbr-sweep. It samples four 2048² textures with
  one mip level, so minified texels miss the cache; that is the leading
  hypothesis, not yet measured.
- **CPU, per frame.** `sample` at 1 ms over 10 s of `praxis bench` across all
  four scenes; 5,624 main-thread samples:
  - 4,333 (77%) wait in `Device::poll`: the bench waits for each frame, and
    CPU frame time excludes the wait;
  - 575 (10%) are scene setup, mostly PNG decode of glTF images in
    `load_model` (load time, not frame time; cooked Blobs, ADR 0018, remove
    it);
  - about 700 are per-frame work. Of those, `CommandEncoder::finish`
    (with Metal's render encoder creation) is 268 (38%), `Queue::write_buffer`
    245 (35%; wgpu-core creates a staging buffer per call, three calls per
    frame, at least 115 samples in `StagingBuffer::new`), and `Queue::submit` 128
    (18%).

### Top three offenders

Ranked by their share of a Budget, one per budgeted metric:

1. **Material Channel arrays, 640 MiB of GPU memory** (8.4% of the 8 GB
   Budget, 97.3–97.7% of the scenes'). #101
2. **Helmet main pass, 603 µs of GPU time** (5.0% of the 12 ms Budget,
   2.3–3.0× the other scenes). #102
3. **Per-frame `write_buffer` uploads, 35% of per-frame CPU** (the largest
   cost the engine controls: encoder finish and submit are one pass's fixed
   cost inside wgpu and Metal). ADR 0017's `StagingBelt` is for streaming
   uploads only ([decision](https://github.com/gariasf/praxis/issues/17#issuecomment-6150843518)). #103

### Also seen

- The interactive app, while its surface reports `Occluded` (window not
  visible), still runs `State::update` on every redraw, three
  `write_buffer` calls and their staging buffers included, then skips
  render and submit: in a 5 s `sample`, 3,217 of 3,307 main-thread samples
  were in `update` and none in `render`. CPU stayed at about 6% and memory
  flat over 20 s.
