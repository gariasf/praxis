## Resolution

**Summary.** On the M4 Pro, 2 ms of GPU buys about 1.36 million probe rays a frame on `world`'s densest view, 1.33 million on `town` with 10,000 Moving Instances and 1.80 million on `town` with the dynamic TLAS empty, each ray traced to its closest hit with a triangle fetch and a sun shadow ray from the hit. The figure is the pass timed alone; inside a frame the same pass costs between a fifth less and three fifths more, depending on how Metal schedules it. Nothing to decide here: the table feeds lane 4's decision comment and M6's charting, and the RTX 2070 SUPER's leg runs from this branch.

**Impact.** ADR 0020's 2 ms for the probe update; M5.5's exit criterion 3; the ticket for the RTX leg.

### The table

Medians of three interleaved rounds, in µs, at 2560×1440. *Pass alone* is the pass with the GPU to itself. *Frame growth* is what the frame's GPU time grows by when the pass traces, with a direct dispatch. *Pass timestamps* are the pass's own start and end in the frame.

| Scene | Rays a frame | Pass alone, µs (rounds) | Frame growth, µs (rounds) | Pass timestamps, µs | Quiet |
|---|---|---|---|---|---|
| `world`, rays to 256 m | 65,536 | **155** (155.1, 154.9, 156.2) | 129 (130.6, 125.3, 128.7) | 276 | 3/3 |
| `world`, rays to 256 m | 524,288 | **824** (824.3, 825.3, 822.1) | 691 (687.3, 692.5, 690.5) | 1,024 | 2/3 |
| `world`, rays to 256 m | 2,097,152 | **3,031** (3,031.0, 3,028.7, 3,032.4) | 2,777 (2,776.6, 2,937.5, 2,768.6) | 3,420 | 1/3 |
| `world`, rays to 2 km | 65,536 | **156** (155.9, 153.3, 156.3) | 128 (128.5, 129.2, 128.0) | 277 | 2/3 |
| `world`, rays to 2 km | 524,288 | **825** (827.6, 823.1, 824.7) | 695 (694.8, 684.7, 694.6) | 1,025 | 3/3 |
| `world`, rays to 2 km | 2,097,152 | **3,032** (3,031.6, 3,033.6, 3,029.1) | 2,779 (2,855.3, 2,778.7, 2,768.8) | 3,422 | 2/3 |
| `town`, dynamic TLAS empty | 65,536 | **140** (140.2, 139.1, 140.1) | 108 (77.6, 109.1, 108.1) | 235 | 3/3 |
| `town`, dynamic TLAS empty | 524,288 | **668** (667.6, 666.6, 667.7) | 647 (669.5, 640.0, 647.2) | 1,107 | 0/3 |
| `town`, dynamic TLAS empty | 2,097,152 | **2,307** (2,307.9, 2,307.3, 2,307.5) | 2,845 (3,420.0, 2,845.5, 2,234.6) | 3,102 | 0/3 |
| `town`, 10,000 Moving Instances | 65,536 | **181** (181.9, 181.0, 181.1) | 142 (146.9, 140.2, 142.1) | 320 | 2/3 |
| `town`, 10,000 Moving Instances | 524,288 | **887** (886.5, 887.1, 886.7) | 829 (826.1, 828.8, 829.0) | 1,111 | 2/3 |
| `town`, 10,000 Moving Instances | 2,097,152 | **3,057** (3,057.1, 3,060.2, 3,056.9) | 2,886 (2,884.8, 2,903.2, 2,885.5) | 3,807 | 2/3 |

| Row | Figure |
|---|---|
| `town`, dynamic TLAS at 10,000, no rays: its build on the CPU | 622 µs, recording and submit (634.6, 620.2, 622.3) |
| the same build on the GPU, as the bench times it, past the end of the frame before | 0 µs median, 23 µs p95 (p95 47.8, 21.9, 23.3): it ends under the frame before |
| the same build on the GPU with each frame waited for, so nothing hides it | 315 µs (315.4, 316.8, 313.0), 64 frames a round |
| `world`'s own dynamic TLAS (its birds), for scale | CPU 648 µs; GPU 128 µs past the frame before |
| Shader runtime checks off, on `town`, dynamic TLAS empty, 2,097,152 rays, the row nearest 2 ms | pass alone 2,266 µs (2,266.2, 2,264.3, 2,269.4) against 2,307 µs: 41 µs less, 1.8% |

### What 2 ms buys

From the pass alone, linear between the two rows either side of 2 ms:

| Scene | Rays a frame in 2 ms |
|---|---|
| `world`, densest view, rays to 256 m | 1,362,000 |
| `world`, densest view, rays to 2 km | 1,362,000 |
| `town`, dynamic TLAS empty | 1,802,000 |
| `town`, 10,000 Moving Instances | 1,331,000 |

By the frame's growth with a direct dispatch the same sum gives 1,512,000 on `world` and 1,420,000 on `town` with 10,000 Moving Instances. Each count is of probe rays; every hit adds a sun ray (see What the rays met).

### What the three measures say

- **The pass's own timestamps are not its cost.** They read 13% to 78% above the pass alone. Metal runs the pass beside the passes recorded around it, whose output it does not read, so its start and end take in their work.
- **The pass alone repeats.** Rounds agree within 2%, and within 0.6% from 524,288 rays up. It is 45 submissions of 8 passes each, waited for one by one, each timed from its first pass's start to its last's end, the first 5 dropped, taken at the end of each run on the scene as the run left it.
- **Inside a frame the cost follows the scheduling.** With a direct dispatch the frame grows by 6% to 22% less than the pass alone on `world` and on `town` with 10,000 Moving Instances. With the dynamic TLAS empty the frames of a run differ by up to two thirds (p95 of a frame over its neighbours 1.14 at 524,288 rays, 1.67 at 2,097,152), so those two frame-growth cells are not figures.
- **An indirect dispatch changes it again** (three rows, 524,288 rays): alone it costs 1.4% to 1.9% more (838, 680 and 899 µs); in the frame, `world` grows by 1,241 µs against 691, `town` with 10,000 Moving Instances by 1,421 against 829, and `town` with the dynamic TLAS empty by 536 against 647, steadier. What moves it was not measured; the shadow mask pass's own time is the same in both forms.
- **Ray length changes nothing on `world`:** 256 m and 2 km agree within 0.5%, as the rays end in the settlement around the probes.
- **Cost is less than linear in rays:** 8 times the rays cost 4.8 to 5.3 times as much, and 4 times more cost 3.4 to 3.7 times.

### What the rays met

| Scene | Probe rays | Hits | Hits the sun reaches | Mean hit distance | Rays with the sun rays |
|---|---|---|---|---|---|
| `world`, 256 m | 2,097,152 | 1,511,972 (72.1%) | 568,441 | 18.1 m | 3,609,124 |
| `town`, dynamic TLAS empty | 2,097,152 | 1,277,424 (60.9%) | 525,112 | 25.5 m | 3,374,576 |
| `town`, 10,000 Moving Instances | 2,097,152 | 1,277,424 (60.9%) | 518,814 | 25.5 m | 3,374,576 |

Every hit's fetched triangle was checked to hold the hit, on the final binary, at 524,288 rays: 320,620 of 320,620 on `town`, 378,635 of 378,635 on `world` at 256 m and 379,043 of 379,043 at 2 km. The survey's scratch program traced 3.0 to 3.2 million rays in 2.7 to 3.5 ms on its open scene and 5.5 to 6.5 ms on its dense one; here 3.4 to 3.6 million take 2.3 to 3.1 ms alone.

### Run conditions

- **Adapter:** Apple M4 Pro, Metal, macOS 27.2.1 (Mac16,8).
- **Source:** `prototype/1006-probe-ray-pricing` at `14d1f08d91e6511542dbb7b1b53250ca81cfc142`, source hash `eb94bfc7c16802c19405a4152a1c929f85df7a52`, clean. It branches from `3d557f81`: the frozen commit `e5c55a8d` plus two commits that change no engine source. meshopt 0.7 is in it, so the Far proxies are 0.7's.
- **Runs:** 19 variants in 3 interleaved rounds, 57 runs, 2026-11-23 14:21 to 15:09 CET, release build, 2560×1440, 248 warm-up and 600 measured frames with the camera held. The build-with-frames-waited row was taken at 15:10, three rounds of the bench's 64 counter frames.
- **Power:** `pmset -g batt` read `AC Power` before every run; `pmset -g log` shows AC from 11:29 with no switch and no sleep.
- **Load:** one-minute load average 1.54 to 2.70 at the runs' starts; GPU load 0 to 2% at 52 starts, 7, 7, 10, 18 and 57% at the others.
- **Peers:** the five `praxis-*` sessions held a quiet turn for both windows and ran nothing. The owner's desktop stayed open.
- **Quiet stamps:** 37 of 57 runs. Of the 20 others, 9 are the `town` rows with the dynamic TLAS empty and rays on, whose frames spread as above in every round; the rest had a process over half a core at a start or an end (`launchd`, `BDLDaemon`, `replayd`, `homed`, `BackgroundShortcutRunner`), a load average of 2.0 to 2.7, a GPU load of 18 or 57% at the start, or a frame spread of 1.07 to 1.13. The pass alone in those runs agrees with its quiet rounds within 0.5%, but for one 65,536-ray round on `world` at 1.9%. The build-with-frames-waited runs started at a load average of 3.44.

### Experiment records

On `bench-history`, one per variant, each the round whose pass alone is the median (for the rows without rays, the round whose GPU frame is): `probe-rays-world-256m-{none,8,64,256}`, `probe-rays-world-2000m-{8,64,256}`, `probe-rays-town-empty-{none,8,64,256}`, `probe-rays-town-crowd-{none,8,64,256}`, `probe-rays-{world-256m,town-empty,town-crowd}-64-indirect` and `probe-rays-town-empty-256-unchecked`. 8, 64 and 256 are rays a probe: 65,536, 524,288 and 2,097,152 a frame. Each record carries `probe_ray_pass_alone_microseconds`, `probe_ray_frame_growth_microseconds` and `gpu_pass_microseconds.probe_ray_pass`.

### Choices the prototype made

- **Camera:** held still, 16 m up, so the grid's eight layers stand 2 to 30 m over the ground: `world` at (4416, 16, 1920), the camera of the 64 m grid with the most static TLAS entries (253,548), `town` at (0, 16, 0). Probes inside buildings trace like any other.
- **Rays:** fixed spherical Fibonacci directions; `town`'s rays run to 256 m; the sun ray is unbounded, ends at any hit, and is traced from every hit, from the side the probe ray arrived on, offset as the shadow mask pass offsets.
- **`town`'s dynamic TLAS:** its birds are left out; at 10,000 it holds the shadow proxies of 625 of `crowd`'s figures, undrawn, 5.12 m apart over the grid's 128 m square, swaying as `crowd`'s do.
- **Frame growth:** measured inside one run, between blocks of 50 frames that trace and blocks that do not, three frames either side of a block's edge left out.
- **Not run:** `scripts/verify.sh`; the branch has no Golden or Baseline. Its readback, shader and bench tests pass, and clippy is clean.

### To run it again

From the branch, with the Cooked cache present: `scripts/probe-1006.sh <results directory>`, then `python3 scripts/probe-1006-table.py <results directory>`. The RTX 2070 SUPER's leg runs the same two.

### Open questions

1. Which measure M6's 2 ms is held against, the pass alone or the frame's growth, and with which dispatch form: M6's charting.
2. Whether the probe pass's frames with the dynamic TLAS empty need the indirect dispatch the shadow mask pass has: M6's ticket for the pass.

Earlier, superseded: a first set of rounds at `5a077023` without the alone measure, whose frame growth agrees with the table's (three passes in a frame grew it by 2,422 µs, 807 each, against 822 for one).

