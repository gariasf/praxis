# Performance notes

Profiles of the Reference scenes, one section per milestone exit, and
measurements that name an offender's cause, newest first. Each exit names
its top three offenders and the tickets that take them.

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

- **Coverage depends on the draw's neighbours.** With the IDs written in
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
  long. It is outside the gate, and its cause is not yet measured.

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
