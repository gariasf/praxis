# Performance notes

Profiles of the Reference scenes, one section per milestone exit, and
measurements that name an offender's cause, newest first. Each exit names
its top three offenders and the tickets that take them.

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
| Today: one sampler | 16, 16, 16, 16, 16 | 556.7 / 557.2 / 557.5 | 1,274.3 / 1,274.2 / 1,274.8 |
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
  their capture matches today's Golden pixel for pixel.
- 16x costs 214 µs of the lit pass, 38% of it (557.2 against 343.0 µs at
  1x, medians of the rounds' medians), as M2's exit measured in a single
  pass without MSAA (222 µs). Over 1x, 2x costs nothing, 4x 26 µs and 8x
  104 µs. GPU frame totals move by the same amounts, within 4 µs.
- The 1×1 Textures' anisotropy costs nothing. The Channels' costs
  overlap: albedo alone at 16x costs 42 µs over 1x everywhere, but
  dropping it to 1x from 16x everywhere saves 84 µs.
- Against today: 8x saves 110 µs; colour 16x with data 8x 74 µs, data
  4x 126 µs, data 2x 158 µs. Data 4x saves 1.1% of ADR 0013's 12 ms GPU
  Budget.

Captures at the Golden's size, 640×360, against today's Golden:

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

Images, on the `pr-images` branch at `2a87cbb`: today's
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
the band (rows 484 to 759; the horizon is at 488), and within it over the
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
