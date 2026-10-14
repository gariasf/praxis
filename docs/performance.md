# Performance notes

Profiles of the Reference scenes, one section per milestone exit, newest
first. Each names its top three offenders and the tickets that take them.

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
   cost inside wgpu and Metal). ADR 0017 routes uploads through a
   `StagingBelt`. #103

### Also seen

- The interactive app, while its surface reports `Occluded` (window not
  visible), still runs `State::update` on every redraw, three
  `write_buffer` calls and their staging buffers included, then skips
  render and submit: in a 5 s `sample`, 3,217 of 3,307 main-thread samples
  were in `update` and none in `render`. CPU stayed at about 6% and memory
  flat over 20 s.
