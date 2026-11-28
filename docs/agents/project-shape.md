# Project shape

What each directory and file of the tree holds. Read when looking for where something lives, or when a change adds, moves or renames a file; `CLAUDE.md` keeps a line per directory. A change to the layout changes this file in the same PR.

```
src/
  lib.rs, state.rs             # library root (window shell) + orchestrator
  bin/praxis.rs                # thin binary: command dispatch
  gpu/                         # GPU context, windowed or headless
  capture/                     # offscreen render read back as an image,
                                 or as the light the Probes hold, where
                                 its Goldens live, and how two images,
                                 or two shadow masks, differ
  preflight/                   # `praxis preflight`: a session's starting
                                 state, a line per check
  bench/                       # Reference scene medians, compared with
                                 Baselines and Budgets, published as
                                 Bench records on `bench-history`, and
                                 `praxis traverse`, the world's traversal;
                                 history/import.rs, `praxis history
                                 import`: the Baselines committed before
                                 it, by Harness version era;
                                 history/suspects.rs, which Series
                                 records are Suspects;
                                 history/night.rs, a Night's directory:
                                 its Verdict, then a directory per run;
                                 history/rerecord.rs, `praxis history
                                 rerecord`, which a session runs on runs
                                 of `main`: they re-record the counters
                                 of the Baselines they moved, timings
                                 kept, with the commits between
  stats/                       # frame stats: CPU and GPU times, counts,
                                 the GPU memory ledger by kind,
                                 counting allocator
  streaming/                   # the Chunk streamer: which Chunks of a
                                 Game world to request, hold and unload
                                 around the camera, its Late Chunks, hold
                                 after a teleport and Settled report,
                                 reaching IO through a narrow trait;
                                 io.rs, the IO pool (ADR 0016) that reads
                                 and decodes Chunks and their Assets, a
                                 Texture's Resident tail first, and the
                                 mips streamed after it, and
                                 ChunkStreaming, which the State runs each
                                 frame, Settled for captures and the
                                 bench's frames through its counter
                                 frames, timing a Fast travel's hold,
                                 and which streams a Far field's Far
                                 proxies through a second Chunk streamer
                                 whose radius is the View distance;
                                 mips.rs, mip priority: the
                                 finest mip each streamed Texture needs
                                 and the requests for it, finest-needed
                                 first, and how eviction ranks a Texture
                                 whose finest mip could give way;
                                 chunks.rs,
                                 a Chunk loaded through the AssetLoader
                                 into the World's LoadedChunks, which
                                 prepare writes into the Chunk's range of
                                 Instance slots, and a Far proxy into its
                                 ResidentFarProxies, drawn the same way,
                                 as is its FarGround, the ground past the
                                 View distance
  scenes/                      # Reference scenes: generated content,
                                 camera paths in frames, frame deltas,
                                 the Spin system (the orrery, town's and
                                 world's flocks), the wrap that keeps
                                 world's flocks about its camera and the
                                 crowd's Sway system;
                                 simulation.rs, the Simulation: a
                                 scene's ECS World as its spawn leaves
                                 it, and the schedules that advance it;
                                 town.rs, the dense scene's generator;
                                 world.rs, the streamed Game world's
                                 generator
  components.rs                # ECS components (Transform, MeshRef)
  threading.rs                 # compute pool, schedule executor (ADR 0016)
  camera/, time/, input/,      # feature modules: resource (+ systems)
    helmet/, transform/,          per domain in one mod.rs
    environment/
  assets/                      # asset types, pools, the Cooked cache
                                 reader (Asset IDs, Blobs), and the
                                 loader from one into the other
  cook/                        # the cooker behind the default-on `cook`
                                 feature: `praxis cook` turns manifest
                                 sources and Reference scenes into Blobs
                                 in git-ignored `cooked/` (ADR 0018), and
                                 launch.rs cooks a dev launch's Assets as
                                 each load finds its Blob missing or stale;
                                 far_proxy.rs cooks each Chunk of a scene
                                 with a Far field its Far proxy's Mesh,
                                 far_proxy/bake.rs its atlas and the
                                 Material baked into it
                                 (ADR 0026);
                                 `praxis fetch` downloads the manifest's
                                 fetched sources, as a dev launch that
                                 lacks one does; `praxis export-scene`
                                 writes what prepare reads of a Reference
                                 scene at a frame, from source data, for
                                 a reference renderer
  render/                      # Renderer (owns the Pools, reads no
                                 ECS World, calls the passes in order),
                                 a module per pass (bend_pass: the
                                 compute pass that bends Meshes from
                                 their rest Meshes, a stand-in for
                                 skinning; blas_refit: their BLASes
                                 refitted through wgpu-hal on Metal,
                                 or rebuilt, before the dynamic TLAS's
                                 build; froxel_pass: the
                                 compute pass that lists each Froxel's
                                 lights from the light buffer;
                                 culling_pass: the
                                 compute passes that build the frame's
                                 draw; main_pass: the depth pre-pass,
                                 phase two's depth pass and the lit pass
                                 that draw it, their pipelines and
                                 recording; shadow_mask_pass: the compute
                                 pass that traces each pixel's shadow
                                 rays from the depth target into the
                                 shadow mask the lit pass reads, and
                                 directional lights' shadow edges again
                                 per sample;
                                 depth_pyramid_pass: the compute pass
                                 that reduces the pre-pass's depth into
                                 the depth pyramid, ADR 0019's Hi-Z
                                 pyramid; tonemapping_pass: the pass
                                 that turns the HDR target into the
                                 target's display colour;
                                 light_count_pass: with the
                                 count_point_lights Render switch, the
                                 compute pass that counts each lit
                                 pixel's point lights), targets
                                 (every size-dependent target, the depth
                                 pyramid, the HDR target and the shadow
                                 mask among them,
                                 the depth and colour conventions),
                                 prepare (ECS
                                 World to Frame data), bind_groups (each
                                 defined once), readback (each frame's
                                 GPU stats through a ring of mapped
                                 buffers),
                                 acceleration_structures (the static
                                 and dynamic TLAS, the sizes of
                                 acceleration structures through
                                 wgpu-hal), ray_scene (the Ray scene:
                                 both TLAS, the BLAS refit, the order
                                 of their steps in a frame and the bind
                                 group a pass that traces sets), probes
                                 (one Cascade's Probes: their irradiance,
                                 depth moments and state, and how a
                                 read-back's bytes split by Probe),
                                 instance,
                                 uniforms, staging,
                                 shaders/ (WGSL parts and the modules
                                 each pipeline assembles from them)
tests/                         # integration tests at agreed seams
  goldens/<backend>[-cpu]/     # committed Goldens; PRAXIS_UPDATE_GOLDENS=1
                                 cargo test --test goldens rewrites them,
                                 and a `<golden>-lod-0.png` beside each
                                 that LOD selection changes;
                                 the root README.md shows the metal
                                 ones, and cargo test checks it lists
                                 each scene
bench/                         # budgets.json (ADR 0013) and baselines/
                                 <adapter>/<scene>.json: the M4 Pro's,
                                 CI's lavapipe (`--ci-counters`) and the RTX
                                 2070 SUPER's, a `-dx12` directory for
                                 its DX12 runs; `praxis
                                 bench <dir>` then `praxis compare <dir>`;
                                 `praxis report <dir>` prints the tables
                                 PRs quote; cycles/<scene>.json, each
                                 exported scene's Cycles Baseline, which
                                 scripts/gallery.sh with PRAXIS_BLENDER
                                 judges a run against
scripts/pr-lint.sh             # CI's PR form check: title and body as the
                                 squash commit, size, commits and a
                                 rendering PR's gallery images against
                                 docs/agents/workflow.md, tested by
                                 scripts/pr-lint-test.sh
scripts/gallery.sh             # the visual audit's capture gallery: each
                                 Reference scene at 2560x1440, with
                                 difference images against an earlier one,
                                 and with PRAXIS_BLENDER each scene's Cycles
                                 render beside it
scripts/cycles-reference/      # the Cycles reference, Blender 5.2's Python:
                                 calibrate.py checks its light units,
                                 render.py builds and renders what
                                 `praxis export-scene` writes, compare.py
                                 ports the display transform and compares;
                                 DIFFERENCES.md, what praxis differs in
                                 on purpose or not yet
scripts/sweep.sh               # the nightly sweep: re-benches a slower
                                 Suspect beside the record before it and
                                 bisects it (`praxis history bisect`), opens
                                 a needs-triage issue per pinned Regression
                                 (`praxis history regressions`), then
                                 benches main's scenes lacking a record on a
                                 Series here, on a quiet M4 Pro; beside it
                                 its launchd definition (sweep.plist)
scripts/verify.sh              # the verify command: fmt, clippy, build,
                                 tests with Goldens, bench counters
                                 against Baselines and Budgets
scripts/lavapipe-files.sh      # has CI record a pushed branch's lavapipe
                                 Goldens and Baselines, or takes a run's,
                                 and copies the changed ones into the tree,
                                 removing those the run removed
vendor/                        # crates.io crates carrying upstream fixes
                                 until their release (README.md): wgpu-core
                                 30.0.2 with the TLAS dependency dedup
```
