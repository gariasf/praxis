# Praxis

<img src="tests/goldens/metal/town.png" width="640" alt="town: a procedural town at street level, lit by street lamps">

A 3D engine in Rust, built from scratch for large open-world RPGs on current
high-end GPUs. It is early: the rendering foundations are in, and the main
pass is being finished. There is no editor, game layer or scripting yet.

*Above: `town`, about 250,000 Instances and 1,024 street lamps, with
ray-traced shadows. Its Golden on Metal, 640×360.*

## Where it stands

Work moves through milestones, one at a time. Each has exit criteria written
as commands and numbers before work starts, and closes only once every one
has been run and its output recorded.

| Milestone | What it built | State |
| --- | --- | --- |
| M0 | The checks everything else relies on: one verify command, Goldens, benches and frame stats | done |
| M1 | Frame foundations: f64 World positions drawn camera-relative, reverse-Z depth, a compute pool under a main thread that owns the frame, a fixed 64 Hz simulation | done |
| M2 | Cooked assets: stable Asset IDs, content-hashed Blobs, BC Textures with full mip chains, fixed geometry Arenas | done |
| M3 | The main pass: GPU culling against a two-phase Hi-Z pyramid, 4x MSAA, Froxel light lists, glTF PBR shading, HDR and tonemapping, hard ray-traced shadows | in progress |

Next, M4 brings LOD and streaming and M5 global illumination. Each is
charted into tickets as the one before it nears its end;
[ROADMAP.md](ROADMAP.md) has the long view.

The [Bench history](https://gariasf.github.io/praxis/bench/) shows where
every scene stands on each machine, against the Budgets: 60 fps at
2560×1440 on an M4 Pro, with at most 6 ms of CPU and 12 ms of GPU per frame
and 8 GB of GPU memory ([ADR 0013](docs/adr/0013-reference-hardware-and-provisional-budgets.md)).

## Design choices

Each is an [ADR](docs/adr/), decided for the engine's eventual scope.

- **A stable image at native resolution:** 4x MSAA, and no TAA, upscaling or
  frame generation ([0008](docs/adr/0008-stable-image-without-temporal-tricks.md),
  [0019](docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md)).
- **Ray-traced lighting:** hard ray-traced shadows now, ray-traced
  irradiance probes for global illumination later
  ([0020](docs/adr/0020-ray-traced-probe-gi-and-hard-ray-traced-shadows.md)).
- **Big worlds without jitter:** World positions are f64 and the GPU works
  camera-relative ([0014](docs/adr/0014-world-positions-are-f64-the-gpu-works-camera-relative.md)),
  with reverse-Z depth and no far plane
  ([0015](docs/adr/0015-reverse-z-depth-with-an-infinite-far-plane.md)).
- **Physical light units:** lux, candela and EV100 exposure, as glTF
  defines them ([0023](docs/adr/0023-lights-are-photometric-as-gltf-defines-them.md)).
- **One graphics API:** wgpu, for good
  ([0007](docs/adr/0007-wgpu-is-the-permanent-graphics-api.md)), with
  `bevy_ecs` as a standalone ECS
  ([0009](docs/adr/0009-bevy-ecs-as-a-standalone-crate.md)).
- **Single-player, desktop:** Windows, Linux and macOS
  ([0010](docs/adr/0010-single-player-desktop-only.md)).

## Running it

You need Rust with edition 2024 support and a GPU that meets the Capability
floor ([ADR 0021](docs/adr/0021-capability-floor.md)), which includes inline
ray queries. Development runs on an Apple M4 Pro (Metal) and an RTX 2070
SUPER (Vulkan and DX12); CI renders on lavapipe, Mesa's software Vulkan.

```sh
cargo run --release                          # the helmet scene, in a window
cargo run --release -- town                  # any Reference scene, by name
cargo run --release -- capabilities          # what this GPU has, and what the floor needs
cargo run --release -- capture town town.png 1280x720
scripts/verify.sh                            # format, lint, build, tests with Goldens, bench counters
```

A first launch downloads the helmet model (`praxis fetch` does the same) and
cooks what the scene loads into `cooked/`.

In the window, the mouse looks around, W A S D move, Space rises and Left
Shift sinks, and Esc quits. In the helmet scene, G drops a helmet in front of
the camera and H takes the last one away.

## Reference scenes

Each one tests something specific, and each is benched. The images are the
committed Goldens in `tests/goldens/metal/`, rendered on the M4 Pro at
640×360 with `praxis capture <scene>`, so re-recording one updates this page.
`cargo test --test goldens` checks this list against the scenes it renders.

| Golden | Scene |
| --- | --- |
| <img src="tests/goldens/metal/pbr-sweep.png" width="320" alt="pbr-sweep"> | **pbr-sweep**: spheres across metallic (rows) and roughness (columns), the lit pass's BRDF at a glance. |
| <img src="tests/goldens/metal/uv-grid.png" width="320" alt="uv-grid"> | **uv-grid**: a ground plane with a generated grid texture, tiled twice. |
| <img src="tests/goldens/metal/normal-tangent.png" width="320" alt="normal-tangent"> | **normal-tangent**: Khronos NormalTangentTest face on, real bumps beside normal-mapped quads that should shade the same. |
| <img src="tests/goldens/metal/far-coplanar.png" width="320" alt="far-coplanar"> | **far-coplanar**: two planes 1 cm apart, 2 km away at a grazing angle; under reversed depth the upper one covers the lower everywhere. |
| <img src="tests/goldens/metal/orrery.png" width="320" alt="orrery"> | **orrery**: five textured spheres circling and spinning in their parents' frames, moved per Tick and drawn blended between Ticks. |
| <img src="tests/goldens/metal/minification.png" width="320" alt="minification"> | **minification**: a glossy checker floor 100 m long under one raking point light, showing mips, anisotropic filtering and Toksvig. |
| <img src="tests/goldens/metal/texture-wall.png" width="320" alt="texture-wall"> | **texture-wall**: 10,000 resident Textures, each one colour encoding its index, sampled by 100 quads across the whole Layer range. |
| <img src="tests/goldens/metal/shadows.png" width="320" alt="shadows"> | **shadows**: a sphere over a plane under the sun and one point light, its ray-traced shadow matching the analytic one. |

`town`, shown at the top, completes the Goldens. Two more scenes have none:

- **helmet**: the SciFiHelmet sample model and a material sweep. Its model is
  downloaded, not committed, so it runs locally only.
- **crowd**: 100 figures the GPU bends every frame, a stand-in for skinned
  characters, casting shadows through rigid per-bone proxies.

## Learn more

- [Field guides](https://gariasf.github.io/praxis/guides/): illustrated
  explainers of each concept the engine has gained, with live demos. Each
  names the commit it describes and what has changed since.
- [Bench history](https://gariasf.github.io/praxis/bench/): every bench
  record of `main`, by scene and machine.
- [CONTEXT.md](CONTEXT.md): the glossary. Its terms are the vocabulary of
  the code, the tickets and the reviews.
- [docs/adr/](docs/adr/): the decisions that are hard to reverse, and why.
- [docs/agents/workflow.md](docs/agents/workflow.md): how work moves, from
  milestone to ticket to reviewed pull request.

## License

[MIT](LICENSE). Files under `assets/` keep their own licences, listed in
[assets/ATTRIBUTION.md](assets/ATTRIBUTION.md).
