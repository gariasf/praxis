# Performance notes

Profiles of the Reference scenes, one section per milestone exit, and
measurements that name an offender's cause, newest first. Each exit names
its top three offenders and the tickets that take them.

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
