# Audit lane 7, docs and rules drift

The lane's report, in the form `docs/agents/audit.md` gives: the ADRs,
`CONTEXT.md`, `CLAUDE.md`, `docs/agents/*` and `docs/references.toml` read
against the code at the frozen commit
`8bc34eb18fcfb79307193551e96619316aef566c`, the code winning, with a verdict per
ADR clause, glossary term, rule and workflow clause read, and the registry
checked by the source-check rubric (load-bearing entries in full, every
fifth of the rest in document order, a wrong sample opening its key
family). Eleven readers at high effort, each with the lane brief verbatim:
three on the ADRs (0001 to 0016, 0017 to 0020, 0021 to 0024), one on the
glossary, one on `CLAUDE.md` with `docs/agents/*`, six on the registry in
document order. One adversarial verifier reopened every Finding. Finding
ids carry the reader's letter so that sections written in parallel never
collide: `L7-A`, `L7-B`, `L7-C` the ADR readers, `L7-G` the glossary,
`L7-R` the rules, `L7-S` to `L7-X` the registry readers. Every `where` is
a `path:line` at the frozen commit. Severity is `drift` throughout, as
the lane brief sets; the magnitude says whether the clause is
load-bearing and who it misleads. `docs/audit/deviations.md` holds no
row at this commit.

## Summary

The docs say what the code does almost everywhere, and where they do not the slip is one kind: a sentence that was true when written and that the PR which changed the code never touched, which is 20 of the 45 surviving Findings and two of the three load-bearing ones (ADR 0013's reference hardware, ADR 0021's attachment-format floor); the third, the glossary's Series, on which Suspect, Experiment and Regression are defined, is a definition narrower than the code's. Of 425 ADR clauses, 58 glossary terms, 195 rule and workflow clauses and 284 registry entries read, the verifier confirmed 41 Findings, narrowed 4 and killed 1; the registry is in good order, its seven Findings being form (a missing `checked`, a year in a `section`, an indirect citation) and one family of live-database figures that one archived-url fix closes, with one quote whose source does not hold it. Twelve Rule candidates stand, each on two or more Findings; four of them are forms of the one slip above, and the slip the brief's research note cannot catch mechanically, a prose clause gone false, is the lane's whole yield, so the rules ticket should weigh a process rule (the PR that changes a pass, a buffer, a format or a Blob greps the ADRs and the glossary for its words) above the identifier and number checks.

## Counts

Every Finding is `drift`, the lane's class; no reader found cause for another. Three surviving Findings are load-bearing (L7-A03, L7-C01 and L7-G01); the verifier moved L7-G03 from load-bearing to not, since the Budget term means the same under either tense. The rest mislead a reader of the ADR or glossary, or a reviewer applying the checklist, at low magnitude.

| Read | Rows | Findings filed | Surviving |
|---|---:|---:|---:|
| ADR clauses, 0001 to 0024 | 425 | 24 | 23 |
| Glossary terms | 58 | 9 | 9 |
| `CLAUDE.md` clauses | 111 | 4 | 4 |
| `docs/agents/*` clauses | 84 | 0 | 0 |
| Registry entries opened (201 load-bearing, 76 sampled, 7 opened by a stale or unsupported sample) | 284 | 9 | 9 |

| Registry verdict, after the verifier's retries | Entries |
|---|---:|
| `confirmed` | 276 |
| `stale` | 5 |
| `unsupported` | 1 |
| `wrong` | 1 |
| `unopened` (paywalled; needs a reader with the book) | 1 |

| Severity class | Findings |
|---|---:|
| `drift` | 45 |
| `image`, `budget`, `correctness`, `health` | 0 |

| Verifier verdict | Findings |
|---|---:|
| `confirmed` | 41 |
| `narrowed` | 4 |
| `killed` | 1 |
| filed | 46 |

## ADRs 0001 to 0016

I read the sixteen files `docs/adr/0001-*.md` through `docs/adr/0016-*.md` in full at the frozen commit `8bc34eb1` in the worktree `/Users/guillem.arias/Documents/gariasf/praxis-512`, and checked every sentence that states what the code, tree, a test, a script, a Budget, a format or a dependency does against the code there. Files opened: `Cargo.toml`, `Cargo.lock` (crate list only), `bench/budgets.json`, `bench/baselines/apple-m4-pro/town.json` (and its history through `git log`/`git show`, read-only), `src/render/instance.rs`, `src/render/prepare.rs`, `src/render/renderer.rs`, `src/render/main_pass.rs`, `src/render/staging.rs`, `src/render/readback.rs`, `src/render/targets.rs`, `src/render/acceleration_structures.rs`, `src/render/bind_groups.rs`, `src/render/shaders/{main,culling,common,froxel,depth_pyramid,shading}.wgsl`, `src/render/shader_function_tests.rs` (the precision oracle), `src/assets/{pools,mesh,material,cooked}.rs`, `src/cook/{mod,texture}.rs`, `src/components.rs`, `src/transform/mod.rs`, `src/camera/mod.rs`, `src/threading.rs`, `src/stats/mod.rs`, `src/state.rs`, `src/lib.rs`, `src/gpu/mod.rs`, `src/scenes/mod.rs`, `src/bench/mod.rs`, `tests/compute_pool.rs`, `tests/goldens/metal/far-coplanar.png`, `scripts/verify.sh`, `.github/workflows/ci.yml`, `CONTEXT.md` (the Baseline entry only), `docs/agents/workflow.md` (the budget rule only), `docs/research/job-systems-and-frame-threading.md` (headings only), and the status lines of ADRs 0017, 0019 and 0020 to tell `planned` from `drift`. Greps run: the constants and names each ADR states (`524_288`/`1 << 19`, `16 << 20`, `2_147_483_647`/`1 << 31`, `ANCHOR`/`64.0`, `Depth32Float`, `Greater`, `LoadOp::Clear`, `SAMPLE_COUNT`, `desired_maximum_frame_latency`, `write_buffer`/`write_texture`, `par_iter`/`ComputeTaskPool`/`IoTaskPool`, `wgpu_hal`/`as_hal`, `ash`, `mlua`, `smaa`, `Box<dyn`/`dyn Fn` beside `derive(Component)`, `perspective_`, `Chunk`, `probe`, `qos`, `latency`, `far-coplanar`, `FAR_COPLANAR_COLORS`, `NO_MESH`, `Changed<`, `material_id`, `anchor`, `bevy_transform`). Left to other readers: every registry quote an ADR cites (`wgpu-types-30-ray-query-experimental`, `mlua-0.12.2-*`, `wgpu-hal-30-allocator-report-default`, `apple-mtldevice-current-allocated-size`, `msl-4.1-*`, `apple-nsview-main-actor`, `wgpu-hal-30-metal-acquire-window`, `bevy-ecs-0.18.1-*`, `bevy-tasks-0.18.1-*`, `wgpu-types-30-frame-latency-two`, `wgpu-30-write-buffer-before-submit`, `wgpu-core-30-*`), the glossary's and ADR 0019's own text where my Findings touch them, and `CLAUDE.md`'s checklist.

### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0001 | "Materials are entries in one structured GPU buffer" | matches | `src/assets/material.rs:163-164`, `src/render/bind_groups.rs:53-54` |
| 0001 | "indexed by the Material ID that each Instance carries" | drift (L7-A01) | `src/render/shaders/culling.wgsl:381-386`, `src/render/shaders/main.wgsl:113`, `:172` |
| 0001 | "the shader reads factors and Layers from it" | matches | `src/render/shaders/main.wgsl:172`, `src/assets/material.rs:69-74` |
| 0001 | "so one bind group serves every material" | matches | `src/assets/material.rs:149-150`, `src/render/bind_groups.rs:44-50` |
| 0001 | "stays 'look up by ID' through bindless textures ... which swap only how textures are sampled" | matches | `src/render/bind_groups.rs:45-46` (each Channel a binding array indexed by Layer) |
| 0001 | "Implemented in Phase 6 Step 2 (a8d188e)" | not checkable from the tree (history; the commit exists: `git cat-file -t a8d188e` is `commit`) | — |
| 0002 | "Meshes are (vertex offset, index offset, index count) sub-allocations of one shared vertex buffer and one shared index buffer in the Mesh pool" | superseded by ADR 0019 (L7-A02) | `src/assets/mesh.rs:3-12`, `:17-30`; `src/assets/pools.rs:74-79`, `:105` |
| 0002 | "Indirect draws, BLAS construction and instancing all need geometry addressable by offset in a single buffer" | matches | `src/assets/pools.rs:349-360`, `src/render/main_pass.rs:262` |
| 0002 | "Implemented in Phase 6 Step 1 (fc98395), growing by CPU re-upload" | not checkable from the tree (history; commit exists); today no growth: `src/assets/pools.rs:523-524` "A GPU buffer of fixed capacity, never grown" | `src/assets/pools.rs:523-524` |
| 0002 | "ADR 0017 replaces growth with fixed arenas sized from the Budget" | fixed arenas: matches; "sized from the Budget": planned (the sentence itself says "until the Budget split") | `src/assets/pools.rs:7-9`, `:523-531` |
| 0002 | "sub-allocated by an offset allocator" | matches | `src/assets/pools.rs:553`, `Cargo.toml:28` |
| 0002 | "each at most 2^31 − 1 bytes" | matches | `src/assets/pools.rs:542-545` (`assert!(capacity_bytes < 1 << 31, ...)`) |
| 0002 | "implemented in [...] at a provisional 16 MiB each until the Budget split" | matches | `src/assets/pools.rs:9` (`ARENA_CAPACITY_BYTES: u64 = 16 << 20`), `src/render/renderer.rs:340` |
| 0003 | "Transform-propagation results live in one GPU buffer, the Instance buffer, in a slot per Renderable" | matches | `src/render/renderer.rs:335-337`, `src/render/prepare.rs:135-138` |
| 0003 | "updated incrementally through dirty flags; there are no per-entity uniform buffers" | matches | `src/render/prepare.rs:81-88` (`Changed<WorldTransform>`, `Changed<MeshRef>`, `Changed<MaterialRef>`) |
| 0003 | "GPU-driven rendering, ray-tracing TLAS rebuilds and instancing all consume exactly this shape" | matches | `src/render/shaders/culling.wgsl:123-127`, `src/render/acceleration_structures.rs:476-482` |
| 0003 | "each Renderable keeps a slot ... until it stops being a Renderable, and a freed slot is taken again before a fresh one" | matches | `src/render/prepare.rs:154-181` (`self.free.pop().or(fresh)`) |
| 0003 | "the buffer holds a fixed 524,288 slots" | matches | `src/render/instance.rs:4` (`INSTANCE_CAPACITY: u32 = 1 << 19`), `tests/frame_stats.rs:122` |
| 0003 | "past which a Renderable is refused, warned about once, counted in Frame stats and retried every frame" | matches | `src/render/prepare.rs:161-167`, `src/stats/mod.rs:26-27`, `src/state.rs:271` |
| 0003 | "prepare writes only the slots of new Renderables and of those whose World transform, Mesh or Material changed, found by bevy_ecs change detection" | matches (the blended Renderables, `With<PreviousWorldTransform>`, are written every frame too) | `src/render/prepare.rs:77-88` |
| 0003 | "through the per-frame staging path, so camera motion writes no Instance" | matches | `src/render/renderer.rs:549-551`, `src/render/instance.rs:32-35` |
| 0003 | "A freed slot is written an Instance with no Mesh, which the GPU does not draw, and is taken again from the next frame" | matches | `src/render/prepare.rs:420-432`, `src/render/instance.rs:25-26` |
| 0003 | "Refined by ADR 0014: the buffer stores an integer anchor plus an f32 offset per Renderable" | matches | `src/render/instance.rs:41-43` |
| 0004 | "a prepare pass builds Frame data (the Instances that changed, lights, camera) and the renderer consumes only that" | matches | `src/render/prepare.rs:18-45`, `src/render/renderer.rs:163-167` |
| 0004 | "it lists no draws, since the GPU builds the frame's draw from the Instances" | matches | `src/render/prepare.rs:19-20`, `src/render/culling_pass.rs:352` |
| 0004 | "The Instance buffer persists across frames (ADR 0003), so every Frame data must reach the renderer, in order" | matches | `src/render/renderer.rs:206-208` (a lost upload forces a full rewrite) |
| 0004 | "Frame data is an owned, `Send` value with no borrow into the ECS World" | matches | `src/render/prepare.rs:47-50` (`assert_send_static::<FrameData>()`) |
| 0004 | "the renderer can be tested headless without an ECS World" | matches | `src/render/renderer.rs:221-222`, `src/capture/mod.rs` |
| 0004 | "Today prepare (`src/render/prepare.rs`) fills Frame data from the ECS World on the main thread" | matches | `src/render/prepare.rs` exists; `src/state.rs:263-268` |
| 0004 | "the `Renderer`, which owns the Mesh and Material pools, draws it without reading the ECS World" | matches | `src/render/renderer.rs:163-167`, `:204-205` |
| 0005 | "Per-Instance data (model matrix, normal matrix, Material ID) sits in a structured buffer" | matches (model matrix as rotation-scale plus anchor and offset, per the refinement the ADR names) | `src/render/instance.rs:38-45` |
| 0005 | "there is no bind group per mesh or per instance" | matches | `src/render/bind_groups.rs:5-9` |
| 0005 | "Implemented as the instance buffer in Phase 5" | not checkable from the tree (history) | — |
| 0005 | "Refined by ADR 0014: the model matrix is relative to an integer anchor, which the Instance also carries" | matches | `src/render/instance.rs:39-43` |
| 0005 | "Since one draw spans every Instance (ADR 0019), the lookup key is the Instance slot in the Meshlet record a packed index names, not `@builtin(instance_index)`" | matches | `src/render/shaders/main.wgsl:17`, `:87-92`; `src/render/main_pass.rs:333` (the lit draw at offset 0 covers both phases) |
| 0006 | "Components hold data only: no closures, no `Box<dyn Trait>`, no behaviour methods" | matches (see note N3) | `src/components.rs:9-101`, `src/transform/mod.rs:8-48`, `src/scenes/mod.rs:657-660`, `:1041-1044`, `:1057-1058`, `src/state.rs:388-389` |
| 0006 | "Behaviour lives in systems or, later, script handlers" | matches (systems); scripts planned (ADR 0012) | `src/transform/mod.rs:95`, `src/scenes/mod.rs:665` |
| 0006 | "Pure-data components serialise cleanly for save/load and let scripts hot-reload" | not checkable from the tree (no save/load or scripting exists) | — |
| 0007 | "The engine stays on wgpu (Vulkan, DX12, Metal)" | matches | `Cargo.toml:8` |
| 0007 | "using `wgpu-hal` only for a subsystem that exceeds wgpu's safe API" | matches | `src/render/blas_refit.rs:233-237`, `:272` (the one `as_hal` user) |
| 0007 | "migration to raw Vulkan through `ash` is not planned" | matches | `Cargo.toml:7-28` (no `ash`; `Cargo.lock:114` only as wgpu-hal's dependency) |
| 0007 | "wgpu's experimental inline ray queries cover the planned RT features" | matches (the registry quote is another reader's) | `src/gpu/mod.rs:170`, `:269` |
| 0007 | "Consoles are out of scope (0010)" | not checkable from the tree (scope) | — |
| 0008 | "Anti-aliasing is spatial (MSAA, SMAA); TAA, upscaling stacks and frame generation are not mandatory, and native resolution is the default" | matches (4x MSAA; no SMAA, upscaler or history target exists, none claimed built) | `src/render/targets.rs:16`, `:148-166` |
| 0008 | "Nothing the image depends on reads screen-space history ... the main pass reads none (ADR 0019)" | matches | `src/render/targets.rs` (no previous-frame target; grep `previous`/`history` empty) |
| 0008 | "lighting terms are class (a) or (b) (ADR 0020)" | not checkable from the tree beyond the shadow mask being per-frame | `src/render/shadow_mask_pass.rs` |
| 0008 | "Lighting is ray-traced probe GI with hard ray-traced shadows (amended 2026-10-13, ADR 0020)" | shadows: matches; probe GI: planned (ADR 0020's status line says "Partly implemented" and names no probes) | `src/render/shadow_mask_pass.rs`, `src/render/shaders/shading.wgsl:261` ("a stand-in for ADR 0020's irradiance probes (M5)") |
| 0008 | "the main pass is clustered forward with 4x MSAA (ADR 0019)" | matches | `src/render/targets.rs:16`, `src/render/main_pass.rs:92`, `src/render/froxel_pass.rs:28` |
| 0009 | "The ECS is `bevy_ecs` used on its own, not the Bevy engine" | matches | `Cargo.toml:17-18`; `Cargo.lock:190-334` (no `bevy`, no `bevy_transform`) |
| 0009 | "the engine keeps its own `Transform` ... ADR 0014 replaces that newtype with an f64 translation and f32 rotation and scale" | matches | `src/components.rs:11-15` |
| 0009 | "A hand-written sparse-set ECS was considered and rejected on 2026-04-28" | not checkable from the tree (history) | — |
| 0010 | "Praxis targets single-player games on Windows, Linux and macOS" | matches | `Cargo.toml:8` (dx12, metal, vulkan); `bench/baselines/` (M4 Pro, llvmpipe, RTX 2070 SUPER on Vulkan and DX12) |
| 0010 | "Consoles, netcode and WASM are out of scope ... wgpu's desktop backends are enough" | matches | `Cargo.toml:8` (no `webgpu`/`webgl` feature); `.github/workflows/ci.yml:35-218` (no wasm target) |
| 0010 | "Accepted 2026-10-12" | not checkable from the tree | — |
| 0011 | "glTF (from Blender) is the authoring source" | matches | `Cargo.toml:16`, `:41`; `src/cook/mesh.rs` |
| 0011 | "the runtime loads Blobs with mip chains, GPU block compression (BC) and byte ranges sized for streaming" | matches | `src/assets/cooked.rs:209-232`, `:252-254`, `:361-364` |
| 0011 | "since then a development launch reaches glTF only through the cooker, which cooks what changed before the runtime loads Blobs" | matches | `src/state.rs:160-170`, `src/cook/mod.rs:43-47`, `:160-166`, `Cargo.toml:41` |
| 0011 | "Textures were uncompressed RGBA8 without mips when this was accepted" | not checkable from the tree (history); today BC7/BC5/BC4 with full mips | `src/assets/cooked.rs:221-232` |
| 0011 | "Textures cook into KTX2, a standard container" | matches | `src/assets/cooked.rs:252-310`, `src/cook/texture.rs:143`, `Cargo.toml:25` |
| 0011 | "Asset identity, Cook keys and payloads are ADR 0018" | matches (cross-reference) | `src/assets/cooked.rs:77`, `src/cook/mod.rs:1-3` |
| 0012 | "Gameplay logic ... lives in Lua through mlua, with LuaJIT where supported; the engine stays in Rust" | planned | `Cargo.toml:7-28`, `Cargo.lock` (no `mlua`) |
| 0012 | "Lua responds to engine events and runs coroutines but never authors per-frame systems" | planned | — |
| 0012 | "Planned; sequenced by the milestone ladder" | matches (status) | `Cargo.lock` (no `mlua`) |
| 0013 | "Budgets are judged on an Apple M4 Pro until a Windows GPU machine exists; that machine gets its own ADR" | drift (L7-A03) | `bench/baselines/nvidia-geforce-rtx-2070-super/town.json`, `bench/baselines/nvidia-geforce-rtx-2070-super-dx12/town.json`, `CONTEXT.md:188`, `docs/adr/` (0017 to 0024 name no such machine) |
| 0013 | "provisional until the first Baseline of a Reference scene at 100k visible Instances and about 5M visible triangles, M3's dense scene" | matches | `bench/baselines/apple-m4-pro/town.json:80-81` (137,730 Instances, 5,552,214 triangles) |
| 0013 | "60 fps at 1440p native with CPU at most 6 ms and GPU at most 12 ms per frame" | matches | `bench/budgets.json:2-3`, `src/bench/mod.rs:53`, `:453` |
| 0013 | "the 6 ms bounds the wall time of each pipeline stage's critical path, waits on worker threads included" | matches (main-thread wall time; no worker does frame work yet) | `src/stats/mod.rs:11-15`, `src/bench/mod.rs:684-696` |
| 0013 | "CPU time summed across threads is measured and reported but not budgeted until then" | drift (L7-A04) | `src/stats/mod.rs:9-60` |
| 0013 | "Input latency at most 2 frame periods (33.3 ms at 60 fps) ... and 2 frame periods plus one Tick (49.0 ms) ..., judged at p95" | not checkable from the tree: no latency measurement exists (grep `latency` in `src/stats`, `src/bench` empty); the arithmetic holds (33.3 + 15.6) | — |
| 0013 | "100k visible Instances, about 5M visible triangles, a 10 by 10 km streamed Game world, a 2 km LOD'd view distance" | Instances and triangles: matches; streamed world and LOD: planned (no streaming or LOD in `src/`) | `bench/baselines/apple-m4-pro/town.json:80-81` |
| 0013 | "GPU memory at most 8 GB at 1440p" | matches | `bench/budgets.json:7` (`8000000000`) |
| 0013 | "counted as the bytes of the buffers and textures the engine has created, summed by the engine itself on every backend" | matches | `src/render/renderer.rs:755-786`, `src/stats/mod.rs:39-45` |
| 0013 | "the GPU stats readback's buffers are left out of that sum" | matches | `src/render/renderer.rs:757-758`, `src/stats/mod.rs:42-44` |
| 0013 | "wgpu's allocator report and Metal's `currentAllocatedSize` are diagnostics only" | matches (the two research notes exist) | `src/stats/mod.rs:53-55`, `docs/research/gpu-memory-residency-streaming.md`, `docs/research/wgpu-capability-matrix.md` |
| 0013 | "the 8 GB splits into Budgets for geometry arenas, the texture pool, Resident tails, render targets, acceleration structures and staging, whose sizes M4 sets" | planned | `bench/budgets.json:1-8` (one `gpu_memory_bytes`), `src/assets/pools.rs:7-9` |
| 0013 | "acceleration structures join the count" | matches | `src/render/renderer.rs:779`, `src/stats/mod.rs:46-49` |
| 0013 | "At M3's exit the Budgets were re-decided against that first Baseline, `town`'s at 2560×1440 with 4x MSAA on the M4 Pro (CPU frame 5.80 ms at the median and 6.21 ms at p95, GPU frame 20.1 ms, GPU memory 764 MB)" | matches as history (see note N1): the Baseline at commit `99fdb38e` held 5796.125 / 6209.458 / 20123.291 µs / 764009588 bytes; today's holds 4734.791 / 5088.292 / 19935.042 / 764009620 | `bench/baselines/apple-m4-pro/town.json:9-19`, `:84` |
| 0013 | "each stays as written and is no longer provisional, and so do the dynamic TLAS's ceilings of 1.1 ms of CPU and 1.0 ms of GPU" | matches | `bench/budgets.json:5-6` |
| 0013 | "what `town` breaches is ticketed as an offender, not budgeted away" | matches (the Budget stayed at 12 ms against a 19.9 ms GPU frame) | `bench/budgets.json:3`, `bench/baselines/apple-m4-pro/town.json:18` |
| 0013 | "A budget moves down freely and up only through an ADR or a map comment (`docs/agents/workflow.md`)" | matches | `docs/agents/workflow.md:56` |
| 0014 | "f32 positions lose it beyond 512 m from their origin" | not checkable from the tree beyond the oracle, which shows f32 World positions failing within 100 km | `src/render/shader_function_tests.rs:1849-1853` |
| 0014 | "World positions are f64, right-handed, +Y up (gravity along −Y), in metres, with cameras looking down −Z" | matches | `src/components.rs:12`, `src/camera/mod.rs:35`, `:81` (`look_at_rh`, `Vec3::Y`) |
| 0014 | "`Transform` holds translation as `DVec3` (relative to the parent for a child) with rotation (`Quat`) and scale (`Vec3`) in f32, 64 bytes like the `Affine3A` it replaces" | matches | `src/components.rs:6-18` (`size_of::<Transform>() == 64`) |
| 0014 | "it reads an integer anchor on a 64 m grid plus an f32 offset, and shaders compose positions relative to the camera" | matches | `src/render/instance.rs:8`, `:13-17`; `src/render/shaders/common.wgsl:31-39` |
| 0014 | "`Transform` and the camera hold f64 positions" | matches | `src/components.rs:12`, `src/camera/mod.rs:20-21` |
| 0014 | "point lights reach the GPU as an anchor plus an offset, which the Froxel pass and the lit pass compose camera-relative" | matches | `src/render/uniforms.rs:25`, `src/render/shaders/froxel.wgsl:38`, `src/render/shaders/main.wgsl:285` |
| 0014 | "A child's Transform composes with its ancestors' into a World transform in f64, recomputed only when it or an ancestor moved" | matches | `src/transform/mod.rs:88-140` |
| 0014 | "Each Instance and the camera reach the GPU as an anchor on the 64 m grid plus an f32 offset, which the vertex shader composes camera-relative" | matches | `src/render/instance.rs:41-43`, `src/render/uniforms.rs:46-48`, `src/render/shaders/main.wgsl:95-101` |
| 0014 | "a shader-function oracle holds the composition to the precision bar in every evaluation order within 100 km, and shows f32 World positions failing it" | matches | `src/render/shader_function_tests.rs:1767-1853` |
| 0014 | Precision bar: "a point 0.3 m from the camera lands within 0.25 px at 1440p and a 45° vertical field of view (0.043 mm)" | matches | `src/camera/mod.rs:86` (`FRAC_PI_4`), `src/render/shader_function_tests.rs:1767-1769` (`PRECISION_BAR: f64 = 4.3e-5`) |
| 0014 | Precision bar: "simulated positions resolve to 0.1 mm; saves round-trip positions bit-exactly" | planned (no physics or saves exist) | — |
| 0014 | Options: "f64 rotation and scale buy nothing: the GPU applies rotation in f32 regardless" | matches | `src/render/instance.rs:39` |
| 0014 | Options: "f32 positions fail ... A floating origin must shift ... An integer cell plus an f32 position in the ECS ... f64 on the GPU: Metal has no f64 shaders" | not checkable from the tree (rejected options; the Metal claims are registry quotes) | — |
| 0014 | Consequences: "The Instance buffer (ADR 0003) and the per-Instance data (ADR 0005) store anchor plus offset, so camera motion dirties no Instance" | matches | `src/render/instance.rs:32-35`, `:41-43`; `src/render/prepare.rs:81-88` |
| 0014 | Consequences: "The vertex shader composes camera-relative positions until draws are GPU-driven; then the culling pass does it once per Instance" | drift (L7-A05) | `src/render/shaders/main.wgsl:95-101`, `src/render/shaders/culling.wgsl:123-129`, `src/render/main_pass.rs:262` |
| 0014 | Consequences: "Worst cases, measured separately, sum to 0.041 mm, 0.0185 and 0.0225 ... A 128 m grid sums to 0.045 mm and fails" | not checkable from the tree (measured in an issue); the oracle's local positions reach 128 m from the pivot, consistent | `src/render/shader_function_tests.rs:1806-1810` |
| 0014 | Consequences: "Subsystems that compute in f32 (physics, audio, particles, ray-tracing acceleration structures) convert at their boundary into a local frame they own" | acceleration structures: matches; physics, audio, particles: planned | `src/render/acceleration_structures.rs:103-109`, `:476-482` |
| 0014 | Consequences: "The physics engine, not yet chosen, meets the simulation bar with f64 internals or f32 frames within 1 km" | planned | `Cargo.toml:7-28` (no physics crate) |
| 0014 | Consequences: "Lua receives positions as numbers, which are doubles. Saves store f64" | planned (ADR 0012 Planned; no saves) | — |
| 0014 | Consequences: "A Chunk is a vertical column addressed by integer (x, z) ... cooked meshes stay within 128 m of their pivot" | planned (no `Chunk` in `src/`; no pivot bound in `src/cook/mesh.rs`) | — |
| 0014 | Consequences: "World-anchored shader effects (noise, wind, triplanar mapping) derive their periodic coordinates from the anchor" | planned (no such effect in `src/render/shaders/`) | — |
| 0014 | Consequences: "One Game world is loaded at a time, so a World position carries no Game world id" | matches | `src/components.rs:11-15` |
| 0015 | "The view distance reaches 2 km (ADR 0013)" | matches (cross-reference; the far-coplanar scene sits at 2,000 m) | `src/scenes/mod.rs:588` |
| 0015 | "With a 0.1 m near plane and `Depth32Float`, standard depth resolves 2.4 m at 2 km; reversed depth with an infinite far plane resolves 0.15 mm" | matches: `NEAR_PLANE = 0.1` and `Depth32Float`; the arithmetic checks (ulp(1) = 2^-24 over n/z² gives 2.4 m; ulp(5e-5) = 2^-38 over the same gives 0.146 mm) | `src/camera/mod.rs:88`, `src/render/targets.rs:12` |
| 0015 | "Depth is cleared to 0 and compared with `Greater`, in `Depth32Float`, with no far plane" | matches | `src/render/targets.rs:12`, `:292`, `:387`; `src/camera/mod.rs:87-97` |
| 0015 | "shadow comparisons follow the same direction" | not checkable from the tree: no shadow-map comparison exists, shadows are ray-traced (ADR 0020) | `src/render/shadow_mask_pass.rs` |
| 0015 | "a Hi-Z pyramid keeps the minimum depth, which is the farthest" | matches | `src/render/shaders/depth_pyramid.wgsl:2`, `:29-33` |
| 0015 | "`projection_matrix` in `src/camera/` is the one projection the window, capture and bench paths use" | matches | `src/camera/mod.rs:95-97`; `src/render/prepare.rs:290` (the only non-test caller; grep `perspective_` finds no other) |
| 0015 | "the `far-coplanar` Reference scene's Golden turns red under standard depth" | drift (L7-A06) | `src/scenes/mod.rs:38-39`, `:176-178`, `:596`, `:606`; `tests/goldens/metal/far-coplanar.png` |
| 0016 | "The main thread runs winit, input, the simulation schedule, prepare and command recording, then submits and presents" | matches | `src/lib.rs:71-82`, `:211-212`; `src/state.rs:121-125`, `:263-282`; `src/render/renderer.rs:485` |
| 0016 | "a compute pool lends it workers inside the frame, and no stage runs a frame behind another" | matches (the pool exists and is idle) | `src/threading.rs:12-22`, `src/state.rs:144` |
| 0016 | "the lowest Input latency of the options surveyed (`docs/research/job-systems-and-frame-threading.md`, option A)" | matches (the note's option A exists) | `docs/research/job-systems-and-frame-threading.md:91-93` |
| 0016 | "wgpu-hal 30.0.1's Metal acquire sends `-[NSView window]` ... on the calling thread" | not checkable by me (registry quote); the version pin matches | `Cargo.lock` (`wgpu-hal` 30.0.1) |
| 0016 | "Partly built: the `multi_threaded` feature is on" | matches | `Cargo.toml:17-18` |
| 0016 | "the `ComputeTaskPool` is created with `available_parallelism() − 1` workers (at least 1)" | matches | `src/threading.rs:14-19`, `tests/compute_pool.rs:4-8` |
| 0016 | "the schedule stays `SingleThreaded`" | matches | `src/threading.rs:27-30`, `src/state.rs:317`, `src/time/mod.rs:70` |
| 0016 | "no system uses the pool yet, and the rest is planned" | matches | grep `par_iter`/`ComputeTaskPool`/`.scope(` outside `src/threading.rs`: none (the cooker uses `std::thread::scope`, `src/cook/mod.rs:175`, and is no system) |
| 0016 | Considered options (render thread, OS-only main thread, job graph) | not checkable from the tree (rejected options) | — |
| 0016 | "Frame data (ADR 0004) is an owned, `Send` value with no borrow into the ECS World" | matches | `src/render/prepare.rs:47-50` |
| 0016 | "Trigger: a Reference scene's main-thread critical path exceeds 6 ms while the pool is saturated, measured, not predicted" | not checkable from the tree (a rule) | — |
| 0016 | "bevy_ecs's `multi_threaded` feature is on, for `par_iter` and scoped pool tasks, while the schedule keeps `ExecutorKind::SingleThreaded`" | matches | `Cargo.toml:17`, `src/threading.rs:29` |
| 0016 | "the multi-threaded executor costs 0.4 to 1.2 µs per system per run on the M4 Pro ... It switches when a Baseline shows the gain" | not checkable from the tree (measurement) | — |
| 0016 | "The `ComputeTaskPool` is created explicitly with `available_parallelism() − 1` workers, since the main thread joins scopes; `TaskPool::default` would put 13 threads on 12 cores" | matches (the default's count is a registry claim) | `src/threading.rs:13-21` |
| 0016 | "No QoS class is set: it made no measured difference" | matches | grep `qos`/`QoS` in `src/`, `scripts/`: none |
| 0016 | "Asset reads and decoding run on a separate IO pool outside the compute pool" | planned (the status line's "the rest is planned"; no IO pool in `src/`) | — |
| 0016 | "They decode into CPU memory, and the main thread copies it into the `StagingBelt`" | matches | `src/render/staging.rs:171-173`, `src/assets/material.rs:128` |
| 0016 | "Only the main thread submits, presents or polls the wgpu `Queue` and `Device`, so map and work-done callbacks run at a known point of the frame" | matches (no other thread touches them) | `src/render/renderer.rs:485`, `src/lib.rs:211-212` |
| 0016 | "Workers record `CommandEncoder`s and return `CommandBuffer`s; they push no error scopes" | planned (no worker records) | — |
| 0016 | "`desired_maximum_frame_latency` stays 2" | matches | `src/lib.rs:165`, `src/render/mod.rs:43` (`FRAMES_IN_FLIGHT: usize = 2`) |
| 0016 | "Per-frame data the GPU reads is written through `Queue::write_*`, which stages it and orders the copy before the next submit" | superseded by ADR 0017 (L7-A07) | `src/render/staging.rs:11-13`, `src/render/renderer.rs:549-551` |
| 0016 | "or into a ring of mapped buffers whose slot is reused only after its `map_async` callback fires ... so the ring decides whether the CPU waits, not whether reads are safe" | matches | `src/render/readback.rs:1-16`, `:74-83`; `src/render/staging.rs:6-9`, `:31-37` |
| 0016 | "Lua (ADR 0012) runs in one state on the main thread, with mlua's `send` feature off ... Lua handler time counts against the 6 ms critical path" | planned (no `mlua`) | `Cargo.lock` |
| 0016 | "Recording 100k draws, one per Instance, has a 90th percentile of 5.1 to 6.9 ms ... under wgpu 29.0.1 (13.5 to 15.5% more ... on 30.0.1 ...)" | not checkable from the tree (measurement) | — |
| 0016 | "which ADR 0019 answers with one `draw_indexed_indirect` per pipeline" | drift (L7-A08) | `src/render/main_pass.rs:253`, `:262`, `:290`, `:296`, `:330-333`; `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:15` |


## ADRs 0017 to 0020

I read the four ADRs in full at the frozen commit (`docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md`, `0018-cooked-assets-are-named-by-stable-ids-and-stored-by-content-hash.md`, `0019-clustered-forward-main-pass-with-4x-msaa.md`, `0020-ray-traced-probe-gi-and-hard-ray-traced-shadows.md`), title, status line, amendments, Considered options and Consequences, and checked every sentence that states what the code, tree, a test, a script, a Budget, a format or a dependency does. Files opened: `src/assets/{pools,mesh,material,cooked,loader}.rs`, `src/cook/{mod,texture,mesh}.rs`, `src/render/{staging,targets,main_pass,culling_pass,depth_pyramid_pass,froxel_pass,shadow_mask_pass,acceleration_structures,blas_refit,bend_pass,tonemapping_pass,renderer,uniforms,bind_groups}.rs`, `src/render/shaders/{main,culling,depth_pyramid,froxel,shadow_mask,shading,common}.wgsl`, `src/gpu/mod.rs`, `src/stats/mod.rs`, `src/bench/mod.rs`, `src/threading.rs`, `src/state.rs`, `src/bin/praxis.rs`, `src/scenes/mod.rs`, `tests/cook.rs`, `Cargo.toml`, `Cargo.lock`, `.gitignore`, `.github/workflows/ci.yml`, `assets/manifest.json`, `bench/budgets.json`, `docs/performance.md` (grepped), and the `wgpu-types-30.0.1` and dependency crate sources under `~/.cargo/registry` for the default storage-binding limit and licence fields. Greps run over `src/`, `tests/`, `scripts/`, `.github/` for the constants and names each ADR states: arena capacities, `offset_allocator`, `TEXTURE_BINDING_ARRAY`, `anisotropy_clamp`, `StagingBelt`, `write_texture`, `transient_saves_memory`, the acceleration-structure byte constants, `texel_density`/`resident`/`priority` (streaming terms), Blob magics and format versions, `MESHLET_MAX_*`, `QTANGENT_SMALLEST_W`, `encode_vertex_buffer`, BC formats, `DITHER_MASK`, `Chunk`/`Placement`, `draw_indexed_indirect`, `multi_draw_indirect`, `alpha_to_coverage`, `SHADOW_MASK_LIGHTS`, `SHADOW_EDGE_LIGHTS`, `DYNAMIC_TLAS_CAPACITY`, `LIGHT_CAPACITY`, `CrowdForm`, `ambient`, `shadow_map`/`cascad`, `gate`. Left to other readers: every registry quote cited inside these ADRs (`docs/references.toml` keys), ADR 0008, 0011, 0013, 0014, 0015, 0016 and 0021's own claims where these ADRs cross-reference them, `CONTEXT.md` terms, and `docs/performance.md`'s measurements. Measurements the ADRs quote (milliseconds, decibels, byte counts of a fetched asset, percentages) are marked not checkable from the tree.

### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0017 | "Textures are ordinary mip chains streamed whole mip at a time ... chosen by CPU heuristics" | planned (status line: "still without streaming"; a Texture loads its whole chain at load) | `docs/adr/0017-...md:3`; `src/assets/material.rs:95-133` |
| 0017 | "vertices and indices live in fixed arenas sized from the Budget at device creation and sub-allocated by an offset allocator" | matches (fixed, sub-allocated; the size is a provisional constant until the Budget split) | `src/assets/pools.rs:7-9`, `:527-556` |
| 0017 | "option A of `docs/research/gpu-memory-residency-streaming.md`" | matches (file exists) | `docs/research/gpu-memory-residency-streaming.md` |
| 0017 | "it needs only Tier 1 bindless" | matches (the floor requests the binding-array features and nothing above) | `src/gpu/mod.rs:261-272` |
| 0017 | "each Channel is a partially bound binding array of Textures sized to their source" | matches | `src/assets/material.rs:77-84`, `:112-126` |
| 0017 | "loaded from cooked Blobs with their full mip chains through a `StagingBelt`" | matches | `src/assets/material.rs:120`, `:127-128`; `src/render/staging.rs:166-174` |
| 0017 | "sampled trilinear, the colour Channels at 16x anisotropy and the data Channels at 4x" | matches | `src/assets/material.rs:28-29`, `:346-357` |
| 0017 | "still without streaming" | matches (no residency, priority or eviction code; grep `resident`/`priority` finds none in `src/`) | `src/assets/material.rs:95-134` |
| 0017 | "one vertex and one index arena of a provisional 16 MiB each, which refuse a Mesh that does not fit" | matches | `src/assets/pools.rs:9`, `:97-104`, `:219-228` |
| 0017 | "three Meshlet arenas of the same size, for the Meshlets, their vertices and their triangles" | matches | `src/assets/pools.rs:106-120` |
| 0017 | Considered options: software VT, sparse textures, virtual geometry pages, growable buffers | not checkable from the tree (rejected designs and vendor samples) | `docs/adr/0017-...md:7-10` |
| 0017 | "ADR 0019 turned virtual geometry down until its reopen condition holds" | matches (ADR 0019 says so) | `docs/adr/0019-...md:9` |
| 0017 | "Each Channel is one partially bound texture binding array (`TEXTURE_BINDING_ARRAY`, `PARTIALLY_BOUND_BINDING_ARRAY`) in one bind group" | matches | `src/gpu/mod.rs:263-264`; `src/assets/material.rs:362-387` |
| 0017 | "a Layer is an index into its Channel's array, Layer 0 stays the neutral default" | matches | `src/assets/material.rs:77-78`, `:307-308` |
| 0017 | "the fixed 8 Layers per Channel go" | matches (13,107 per Channel) | `src/assets/material.rs:12-16` |
| 0017 | "Residency changes are batched so that bind group is rebuilt at most once per frame" | matches (stale flag; one refresh per `render`) | `src/assets/material.rs:311`, `:315-340`; `src/render/renderer.rs:582` |
| 0017 | "Gate ... measured at 10k textures ... Measured ... at the line (`docs/performance.md`) and recorded as marginal; a survey before M4's residency work decides what follows" | matches (the record exists; the cost itself is a measurement, not checkable) | `docs/performance.md:2887-2900`, `:2992-3016` |
| 0017 | "Priority is computed on the CPU: the finest mip any visible Instance needs ... Chunks inside the load radius are requested ahead" | planned (streaming) | `docs/adr/0017-...md:15` |
| 0017 | "No GPU feedback readback until a measured miss rate asks for it" | matches (no readback for residency exists) | `src/render/readback.rs` (GPU stats only) |
| 0017 | "Each texture's Resident tail (mips of 64 KiB and smaller) never leaves. When a request would exceed the texture pool's capacity ... evicted only below p − 1" | planned (streaming) | `docs/adr/0017-...md:16` |
| 0017 | "One vertex arena and one index arena, and the Meshlet arenas, each at most 2^31 − 1 bytes" | matches | `src/assets/pools.rs:540-545`; `src/gpu/mod.rs:300` |
| 0017 | "sub-allocated with the `offset-allocator` crate (... MIT ...; new dependency ...)" | matches (0.2.0, licence MIT in the crate's `Cargo.toml`) | `Cargo.toml:28`; `Cargo.lock:1792-1793`; `src/assets/pools.rs:527` |
| 0017 | "the vertex shader reads the whole vertex arena through one storage binding (ADR 0019)" | matches | `src/render/bind_groups.rs:80`, `:193`; `src/render/shaders/main.wgsl:94`; `src/assets/pools.rs:97-100` |
| 0017 | "the Capability floor's 128 MiB `max_storage_buffer_binding_size` caps that arena until its binding splits or the floor rises (ADR 0021)" | matches (floor keeps the default; wgpu-types 30.0.1 default is `128 << 20`) | `src/gpu/mod.rs:297-308`; `~/.cargo/registry/src/*/wgpu-types-30.0.1/src/limits.rs:441` |
| 0017 | "No growth at runtime: a load that does not fit is refused and logged" | matches (tested) | `src/assets/pools.rs:189-192`, `:219-228`, `:965-991` |
| 0017 | "The CPU shadow of vertex and index data goes" | matches (`Mesh` holds ranges and bounds only) | `src/assets/mesh.rs:3-12` |
| 0017 | "The 8 GB of ADR 0013 splits into Budgets for geometry arenas, the texture pool, Resident tails, render targets, acceleration structures and staging; M4 sets their sizes" | planned (one `gpu_memory_bytes` Budget today) | `bench/budgets.json:7` |
| 0017 | "A `TRANSIENT_ATTACHMENT` counts as zero only when `transient_saves_memory` is `Some(true)`" | matches | `src/render/renderer.rs:436`; `src/stats/mod.rs:185-192` |
| 0017 | "94 bytes per BLAS triangle (45.5 compacted) and 288.5 per TLAS instance on the M4 Pro, and on lavapipe (Mesa 25.2.8) 104 ..., which compaction leaves unchanged, and 240" | matches | `src/render/acceleration_structures.rs:46-60` |
| 0017 | "`gpu_memory_bytes` counts them as estimates by backend until M4 sets exact sizes" | drift (L7-B01: a third estimate, NVIDIA's, chosen by vendor) | `src/render/acceleration_structures.rs:62-85` |
| 0017 | "Streaming uploads go through a `StagingBelt`, never `Queue::write_texture`" | matches (no `Queue::write_texture` call in `src/`) | `src/render/staging.rs:166-193`, `:198-251` |
| 0017 | "The belt's `chunk_size` is only its allocation unit (it allocates more on demand)" | matches | `src/render/staging.rs:162-164` |
| 0017 | "the engine caps the bytes written into it per frame at the staging Budget, and an upload past the cap waits for a later frame" | planned (no staging Budget; load staging is uncapped, frame staging asserts its bound) | `src/render/staging.rs:67-71`, `:184-193` |
| 0017 | "the IO pool (ADR 0016) decodes into CPU memory and the main thread copies it into the belt" | planned (streaming); note: the tree has only a `ComputeTaskPool`, no IO pool (see Notes) | `src/threading.rs:1-27` |
| 0017 | "The cooked format ... stores each mip as a separately loadable byte range of the Texture's Blob, with the Resident tail as one range" | matches (KTX2 level index, smallest mip first) | `src/assets/cooked.rs:248-290` |
| 0017 | "and each Primitive's UV texel density (ADR 0018)" | drift (L7-B02) | `src/assets/cooked.rs:405-414`, `:522-528` |
| 0018 | "Cooked data is a disposable cache. Every format inside it can change by bumping a version and re-cooking" | matches (tested) | `src/assets/cooked.rs:17-37`, `:638-646`; `tests/cook.rs:250` |
| 0018 | "each source file gets a 128-bit GUID, written once into its `assets/manifest.json` entry" | matches (8 `guid` fields for 8 `path` entries) | `assets/manifest.json:4`; `src/cook/mod.rs:71` |
| 0018 | "An Asset ID is the first 128 bits of SHA-256 over (GUID, Asset kind, glTF name, Channel for a Texture)" | matches | `src/assets/cooked.rs:79-117` |
| 0018 | "The glTF index replaces the name when the name is missing or not unique" | matches | `src/assets/cooked.rs:67-73`; `src/cook/mod.rs:481` |
| 0018 | "A cooked Asset is a Blob named by the SHA-256 of its bytes, found through a cook index that maps Asset ID to Blob, and rebuilt when its Cook key changes" | matches | `src/assets/cooked.rs:1-4`, `:745-746`; `src/cook/mod.rs:331-336` |
| 0018 | "option A of `docs/research/asset-compression-and-cooked-data.md`" | matches (file exists) | `docs/research/asset-compression-and-cooked-data.md` |
| 0018 | "`praxis cook` and every dev launch cook Materials, Textures and Meshes into Blobs" | matches | `src/bin/praxis.rs:100-101`, `:374`; `src/state.rs:163`; `src/cook/mod.rs:110-190` |
| 0018 | "the runtime loads Materials and Textures ... and Meshes ... from them by Asset ID" | matches | `src/assets/loader.rs:61-88`; `src/assets/material.rs:300-313` |
| 0018 | "the reader rebuilds their Meshlets" | matches | `src/assets/cooked.rs:418-430` |
| 0018 | "which the renderer does not keep yet" | drift (L7-B03) | `src/assets/pools.rs:76-78`, `:106-120`; `src/render/renderer.rs:367-374` |
| 0018 | Considered options: source paths, sidecars, content hashes, archive, Basis, neural, BLAKE3/XXH3 (dB and GB/s figures) | not checkable from the tree (rejected designs and measurements) | `docs/adr/0018-...md:7-13` |
| 0018 | "The runtime reads Blobs through one store interface, so an archive can replace per-Asset files behind it later" | matches | `src/assets/cooked.rs:657-740` |
| 0018 | "A glTF mesh renamed in Blender gets a new Asset ID" | matches (the name is hashed) | `src/assets/cooked.rs:100-104` |
| 0018 | "The cooker warns when an ID that something references no longer exists" | matches | `src/cook/mod.rs:193`, `:350` |
| 0018 | "Cook key: SHA-256 over what the Asset cooks from (... by content ...), the Cook keys of its dependencies, every cook setting, the format version and the encoder versions" | matches | `src/cook/texture.rs:115-140`; `src/cook/mesh.rs:25-49`; `src/cook/mod.rs:323-326` |
| 0018 | "The metallic-roughness Texture depends on the normal Texture (Toksvig, below)" | matches | `src/cook/texture.rs:87`, `:127-130` |
| 0018 | "Each dev launch recomputes the keys of the Assets it references (hashing about 30 MB ... about 10 ms) and cooks only the stale ones" | matches for the mechanism (tested); the timing is not checkable | `src/state.rs:163`; `src/cook/mod.rs:124-166`; `tests/cook.rs:611` |
| 0018 | "The Cooked cache lives in git-ignored `cooked/` and is never evicted. Deleting it is always safe." | matches (tested) | `.gitignore:21-22`; `src/assets/cooked.rs:653`; `tests/cook.rs:250` |
| 0018 | "The runtime loads Blobs only (ADR 0011). Each Blob starts with a magic and a format version (a Texture Blob is KTX2, with its format version in a key-value entry). A mismatch is refused with a re-cook message; there is no migration code" | matches (tested) | `src/assets/cooked.rs:17-37`, `:324-336`, `:638-646`, `:804-930` |
| 0018 | "Engine data that no cook format holds exactly is compiled into the binary instead, from a copy a test pins to its source in `assets/`: the tonemapping pass's 64² blue-noise dither mask, whose 256 levels a lossy BC4 cook would unbalance" | matches (tested) | `src/render/tonemapping_pass.rs:17-25`, `:184-196`; `assets/manifest.json:80-85` |
| 0018 | "The runtime still opens no source path" | matches (CI's no-`cook` build renders every Golden with `assets/` absent) | `.github/workflows/ci.yml:94-96`, `:115-126` |
| 0018 | "Textures are KTX2 with every level zstd-supercompressed, stored smallest mip first, so the Resident tail ... is one contiguous byte range" | matches (tested) | `src/assets/cooked.rs:248-290`, `:828` |
| 0018 | "Rows are stored unpadded, and the loader re-pitches them to 256 bytes as it writes into the `StagingBelt`" | matches (tested) | `src/render/staging.rs:195-232`, `:304-309` |
| 0018 | "Formats per Channel, without RDO: albedo and emissive BC7 sRGB, normal BC5 (z rebuilt in the shader), metallic-roughness BC5 linear, occlusion BC4" | matches | `src/cook/texture.rs:162-169`; `src/assets/cooked.rs:221-235` |
| 0018 | "BC6H waits for HDR" | planned (no HDR Texture Channel) | `src/assets/cooked.rs:221-235` |
| 0018 | "The encoder is `intel_tex_2` at its basic profile (... dB figures ..., as `tests/cook.rs` prints it; slow buys 0.1 to 0.7 dB for about 3.5x the time)" | matches for encoder, profile, version and the printing test; the dB figures are measurements, not checkable (the test asserts only ≥ 44.0 dB) | `src/cook/texture.rs:21`, `:500-502`; `Cargo.toml:24`; `tests/cook.rs:328-342` |
| 0018 | "Mips: a 2×2 box filter in linear light, with normals renormalised per mip" | matches (tested) | `src/cook/texture.rs:181-184`, `:230`, `:300-305`; `tests/cook.rs:349` |
| 0018 | "Toksvig widens each mip's roughness from the normal Texture's variance" | matches (tested) | `src/cook/texture.rs:186-196`, `:443-459`; `tests/cook.rs:482` |
| 0018 | "The cooker refuses a Texture whose size is not a multiple of 4 or exceeds 8192, rather than resizing it" | matches (tested) | `src/cook/texture.rs:152-160`; `src/assets/cooked.rs:31-33`; `tests/cook.rs:374` |
| 0018 | "a 28-byte vertex (position `Float32x3`, UV `Float32x2`, normal, tangent and bitangent sign as a QTangent in `Snorm16x4`, its w kept non-zero ...)" | matches (compile-time size assert) | `src/assets/cooked.rs:393-403`, `:45-47` |
| 0018 | "and u32 indices" | matches | `src/assets/cooked.rs:492`; `src/assets/pools.rs:104` |
| 0018 | "Positions stay f32 because 16 bits over a 256 m pivot range is 3.9 mm" | matches for f32 positions; the rationale is arithmetic, not checkable from the tree | `src/assets/cooked.rs:397` |
| 0018 | "`meshopt` reorders for vertex cache, overdraw and fetch, then encodes, then zstd compresses" | matches | `src/cook/mesh.rs:97`, `:147`, `:154`; `src/assets/cooked.rs:512-543` |
| 0018 | "Meshlets of at most 64 vertices ... and at most 124 triangles" | matches | `src/assets/cooked.rs:38-44` |
| 0018 | "Each Meshlet is the longest run the limits allow, as meshoptimizer's scan builder cuts them" | matches | `src/cook/mesh.rs:160-199` |
| 0018 | "A Meshlet is stored as one byte, its triangle count; the reader rebuilds its vertex remap and local triangles from its run of the index stream, which BLAS builds read" | matches | `src/assets/cooked.rs:503-545`, `:430`; `src/assets/pools.rs:101-104`, `:184-189` |
| 0018 | "SciFiHelmet's Mesh Blob is 322,850 bytes this way, 13.82 per triangle" | not checkable from the tree (fetched asset; no test asserts the size) | `tests/cook.rs:684`; `src/cook/mesh.rs:690` |
| 0018 | "The Blob also holds the Mesh's bounding box for culling; per-Meshlet bounds and cones wait for per-Meshlet tests" | matches | `src/assets/cooked.rs:494-495`, `:524-526`; `src/assets/mesh.rs:24-29`; `src/render/shaders/culling.wgsl:119-122` |
| 0018 | "Each Primitive stores its UV texel density, because each Primitive's Material has its own Textures" | drift (L7-B04) | `src/assets/cooked.rs:405-414`, `:522-528` |
| 0018 | "A Material is its own Asset: PBR factors plus one Texture Asset ID per Channel. Primitives reference it by ID." | matches | `src/assets/cooked.rs:131-147`, `:407-412` |
| 0018 | "A Chunk's edge is 256 m ... A v1 Chunk is a list of Placements. Each holds a Mesh Asset ID, an f64 translation ..." | planned (no `Chunk` or `Placement` type in `src/`; status line lists Materials, Textures and Meshes only) | `docs/adr/0018-...md:26` |
| 0018 | "The cooker lives in `src/cook/` behind the `praxis cook` command, with its encoder dependencies behind a default-on `cook` feature that a shipping build turns off" | matches (`intel_tex_2` optional; decoders `ktx2`, `meshopt`, `zstd` stay unconditional, as the runtime reads them) | `Cargo.toml:23-28`, `:37-41`; `src/bin/praxis.rs:100-101`; `.github/workflows/ci.yml:115-122` |
| 0018 | "Procedural Reference scenes cook too, under Asset IDs derived from the scene name, so Goldens test the cooked path" | matches | `src/assets/cooked.rs:63-64`, `:90-93`; `src/cook/mod.rs:124` |
| 0018 | "Goldens regenerate once, and CI cooks before the golden test" | matches (CI's Cook step precedes the tests; the regeneration is historical) | `.github/workflows/ci.yml:75-78` |
| 0018 | "New dependencies ...: `intel_tex_2` (MIT or Apache-2.0 ...), `ktx2` (Apache-2.0), `meshopt` (MIT or Apache-2.0), `zstd` (BSD-3-Clause); `sha2` moves from dev-dependency to dependency" | matches (licence fields in each crate's `Cargo.toml` under the registry agree) | `Cargo.toml:23-27`; `Cargo.lock:1136`, `:1240`, `:1352`, `:2272`, `:3586` |
| 0019 | "a 4x MSAA reverse-Z depth pre-pass, then a lit pass at depth equal that loops over its Froxel's lights and reads each one's shadow from a shadow mask, which a compute pass traces from the depth target before it, one ray per pixel and light" | matches | `src/render/targets.rs:12-16`, `:288-297`, `:383-400`; `src/render/shaders/main.wgsl:269-288`; `src/render/shadow_mask_pass.rs:1-11` |
| 0019 | amendment: "traced inline, the rays cost ... 2.4 to 13.4 times ...; a ray query left in the lit pass's shader cost it 0.9 to 3.9 ms" | not checkable from the tree (measurements) | `docs/adr/0019-...md:3` |
| 0019 | "Geometric edges get 4x MSAA (the Apple9 maximum ...)" | matches for 4x; the Apple9 claim is a registry quote (other reader) | `src/render/targets.rs:14-16` |
| 0019 | "with alpha-to-coverage for masked materials" | planned (no masked Material; the pipelines set it off) | `src/render/main_pass.rs:94` |
| 0019 | "its 4x depth, 58,982,400 bytes at 1440p, which the pre-pass stores for the Hi-Z pyramid and the lit pass" | matches (2560 × 1440 × 4 samples × 4 bytes; depth stored) | `src/render/targets.rs:157-162`, `:288-297` |
| 0019 | "4x colour needs none when the lit and transparent passes share one render pass and resolve in it" | matches for the lit pass (transient, counted 0 where memoryless); the transparent pass is planned | `src/render/targets.rs:163-168`, `:336-358`; `src/stats/mod.rs:185-192` |
| 0019 | amendment: "the depth passes also write each sample's triangle normal and the normal interpolated over it, 4 bytes per sample, 58,982,400 bytes at 1440p, which the shadow mask pass reads ...; the target is allocated either way" | matches (`Rgba8Unorm` at 4x, allocated unconditionally; the ms figures are not checkable) | `src/render/targets.rs:42-45`, `:207-212`; `src/render/main_pass.rs:114-125`; `src/render/shaders/shadow_mask.wgsl:213-218` |
| 0019 | "today a compute pass appends the Meshlets of each Instance whose Mesh bounds reach into the view frustum to the compacted index buffer in slot order, in two phases" | matches | `src/render/shaders/culling.wgsl:1-10`, `:119-160`; `src/render/culling_pass.rs:23-31` |
| 0019 | "Phase one appends those visible the frame before, one bit per slot, and a depth pre-pass draws them into 4x MSAA reverse-Z `Depth32Float` cleared to 0 (ADR 0015) and stores it" | matches | `src/render/culling_pass.rs:150-152`; `src/render/shaders/culling.wgsl:226-259`; `src/render/targets.rs:288-297`; `src/render/main_pass.rs:231-262` |
| 0019 | "A compute pass reduces that depth into the Hi-Z pyramid, each texel the minimum of every sample below it" | matches | `src/render/shaders/depth_pyramid.wgsl:26-33`, `:56-60` |
| 0019 | "Phase two appends the others whose bounds lie anywhere nearer than the pyramid's minimum over the texels their screen rectangle covers, and rewrites the bits" | matches | `src/render/shaders/culling.wgsl:167-222`, `:237-270` |
| 0019 | "Phase two's Instances draw into depth: with shadow rays off, at the start of the lit pass; with them on, in a pass of their own, which like the pre-pass then writes each sample's surface normals too" | matches | `src/render/main_pass.rs:265-296`, `:328-331` |
| 0019 | "a compute pass traces, from each pixel's nearest sample, one shadow ray toward each light that reaches it and keeps whether it is hidden in the shadow mask, and, where the pixel's other samples show another surface, traces from the farthest of those too, into the mask's second layer" | matches | `src/render/shaders/shadow_mask.wgsl:388-435`, `:481-493`; `src/render/targets.rs:28-33` |
| 0019 | "A lit pass draws both phases' at depth equal into a transient 4x colour attachment, memoryless on Apple GPUs, that it resolves into the `Rgba16Float` HDR target, which a tonemapping pass then writes into the target" | matches | `src/render/targets.rs:22`, `:60-64`, `:163-174`, `:336-358`, `:395-400`; `src/render/tonemapping_pass.rs:148` |
| 0019 | "with one `draw_indexed_indirect` per phase for the depth pipeline and one of both phases for the lit pipeline" | matches | `src/render/main_pass.rs:262`, `:296`, `:330`, `:333` |
| 0019 | "lit by every directional light and by the point lights of each pixel's Froxel, which a compute pass at the start of the frame lists from a light buffer of fixed capacity: those whose range reaches into the Froxel's bounds" | matches | `src/render/shaders/main.wgsl:269-288`; `src/render/shaders/froxel.wgsl:48-53`; `src/render/uniforms.rs:1-4`; `src/render/froxel_pass.rs:26` |
| 0019 | "each hidden where the shadow mask says so, the directional lights' rays first, then those point lights', up to 59 lights per pixel; a light past them casts no shadow there and is counted" | matches | `src/render/shadow_mask_pass.rs:29-32`; `src/render/shaders/shadow_mask.wgsl:266-269`, `:304-307`; `src/stats/mod.rs:95-97`; `src/render/readback.rs:430` |
| 0019 | Considered options B, C, D: "59 MB at 4x and 1440p", "19 to 24% slower", "1.2 to 3.7 times" | not checkable from the tree (measurements on prototype branches) | `docs/adr/0019-...md:7-8` |
| 0019 | Option E: no MSAA in Nanite or Bevy, wgpu 30 sizes every BLAS on the CPU; "Dense static meshes use discrete LOD" | registry quotes are another reader's; discrete LOD is planned (no LOD in `src/`) | `docs/adr/0019-...md:9` |
| 0019 | "Lighting is ADR 0020 ..., which shades from position and normal" | matches (cross-ADR; the shadow mask pass works from depth and normals, no G-buffer materials) | `src/render/shaders/shadow_mask.wgsl:210-218` |
| 0019 | "Froxels, each with its own list built once against its bounds, serve the opaque and transparent passes alike" | matches for the Froxel lists (built once per frame against bounds); the transparent pass is planned | `src/render/froxel_pass.rs:1-30`; `src/render/shaders/froxel.wgsl:8-53` |
| 0019 | "One compute culling pass composes each Instance's camera-relative transform (ADR 0014), tests the frustum and a two-phase Hi-Z pyramid ..., and writes a compacted index buffer drawn with one `draw_indexed_indirect` per pipeline and Hi-Z phase, one native call on every backend" | matches | `src/render/shaders/culling.wgsl:111-135`, `:147-222`; `src/render/main_pass.rs:262`, `:296`, `:333` |
| 0019 | "No count draws: wgpu's Metal backend has none, and runs `multi_draw_indirect` as a CPU loop" | matches for the code (no `multi_draw_indirect` or count draw in `src/`); the wgpu claims are registry quotes | `src/render/main_pass.rs:262`, `:296`, `:333` |
| 0019 | "each index packs a Meshlet record in its high bits and a vertex within that Meshlet in its low six: the cooker builds Meshlets of at most 64 vertices into each Mesh Blob, and the vertex shader reads the record for its Instance, Material and vertex base, then pulls its vertex from the vertex Arena" | matches | `src/assets/cooked.rs:38-40`; `src/render/culling_pass.rs:166-168`; `src/render/shaders/common.wgsl:209-213`; `src/render/shaders/main.wgsl:81-113` |
| 0019 | "Each Primitive draws with its own Material unless its Instance overrides it" | matches | `src/render/shaders/culling.wgsl:381-386`; `src/assets/pools.rs:234-240` |
| 0019 | "Culling tests Instances; tests per Meshlet wait for a profile that shows the pass raster-bound (... it stays a prototype, `docs/performance.md`)" | matches (per-Instance bounds; the prototype is recorded) | `src/render/shaders/culling.wgsl:240-260`; `docs/performance.md:1821-1856` |
| 0019 | "On the M4 Pro ... fragments of nearby triangles ... can be merged in a pixel ... A fragment stage that reads any flat input is never merged" | not checkable from the tree (hardware behaviour; recorded in `docs/performance.md:1468`) | `docs/adr/0019-...md:16` |
| 0019 | "Two passes that must agree on a sample's triangle, as the depth passes' normals and the lit pass must ..., both read a flat input" | matches (`@interpolate(flat) material_id` in both fragment stages) | `src/render/shaders/main.wgsl:12`, `:30`, `:51-72`, `:171-172` |
| 0019 | "The depth pre-pass writes 4x MSAA depth only" | drift (L7-B05) | `src/render/main_pass.rs:114-125`, `:240-246` |
| 0019 | "it seeds the Hi-Z pyramid and the lit pass's depth-equal test, and restores front-to-back cost when the culling pass writes draws out of order" | matches | `src/render/shaders/depth_pyramid.wgsl:26-33`; `src/render/targets.rs:393-400` |
| 0019 | "With phase two's added, the shadow mask pass rebuilds each pixel's position and face normal from it, so shadow rays need no G-buffer" | drift (L7-B06: the position is rebuilt from depth; the normals are read from the surface normals target) | `src/render/shaders/shadow_mask.wgsl:210-218`; `src/render/targets.rs:91-93`, `:207-212` |
| 0019 | "at an edge pixel, each fragment reads the bits traced from the surface it shades: the nearest sample's, or, where the fragment covers a sample off that sample's plane, the farthest such sample's ..." | matches | `src/render/shaders/shadow_mask.wgsl:368-435`; `src/render/targets.rs:28-40` |
| 0019 | "the mask's second layer, which samples show it and the list of pixels the second dispatch traces, 16 bytes per pixel, 58,982,404 bytes at 1440p" | matches (8 + 4 + 4 bytes per pixel plus a 4-byte count); the ms and RMSE figures are not checkable | `src/render/targets.rs:26`, `:33`, `:40`, `:130-138`, `:186-199` |
| 0019 | "the second surfaces of 13.7% of its pixels are traced from a packed list ..., since tracing them where they lie cost it 4.1 ms" | matches for the packed list and indirect dispatch; the percentage and ms are not checkable | `src/render/shaders/shadow_mask.wgsl:434-435`, `:481-493`; `src/render/shadow_mask_pass.rs:361-367` |
| 0019 | "Lights are culled into Froxels, each with its own light list built against the Froxel's bounds" | matches | `src/render/froxel_pass.rs:1-30`; `src/render/shaders/froxel.wgsl:8-53` |
| 0019 | "today's up to four point lights become a light buffer" | drift (L7-B07) | `src/render/uniforms.rs:1-4`; `src/render/froxel_pass.rs:1-2`, `:26` |
| 0019 | "Transparency is a sorted forward pass after the opaque pass, sharing the Froxel lists, MSAA and the opaque depth" | planned (no transparent pass in `src/render/`) | `src/render/renderer.rs` (pass list) |
| 0019 | "Render pipelines are vertex pipelines" | matches (no mesh-shader feature in the floor; vertex entry points) | `src/gpu/mod.rs:261-272`; `src/render/shaders/main.wgsl:17`, `:34`, `:81` |
| 0019 | "Metal's ray tracing in render pipelines is not compatible with mesh shading ..., which kept the lit pass from being one until ... its rays [moved] into the shadow mask pass, a compute pass" | matches for the compute pass; the Metal claim is a registry quote | `src/render/shadow_mask_pass.rs:71-80`, `:334-385` |
| 0019 | "a mesh-shaded pre-pass returns with Meshlet culling, which needs the cooked format (ADR 0011)" | planned | `docs/adr/0019-...md:20` |
| 0019 | "Reopen when ... the opaque main pass takes more than 6 ms of GPU, half of ADR 0013's 12 ms, as the median over the scene's camera path, timed from the first culling dispatch to the end of the lit pass with shadow rays off" | matches (Budget 6000 µs; gate from culling phase one to the lit pass, shadow rays off; tested) | `bench/budgets.json:2-4`; `src/stats/mod.rs:81-83`; `src/bench/mod.rs:167-169`, `:1142-1145` |
| 0019 | "Shadow-ray time is reported beside the gate as its own line" | matches | `src/bench/mod.rs:170-172`, `:612-616` |
| 0019 | "The clause fired on `town` and is spent there: B ... added 1.9 to 3.5 ms ..., and D ... read from 2.3% under A's 8.6 ms to 1.2% over it ..., so A stays and the 6 ms with it; ... re-checked on `town` once LOD reduces them" | not checkable from the tree (prototype-branch measurements); the 6 ms Budget stays | `bench/budgets.json:4` |
| 0020 | "Indirect diffuse light comes from ray-traced irradiance probes in camera-centred clipmap cascades (DDGI form ...), updated a subset per frame and shaded ... in ADR 0019's lit pass" | planned (a constant ambient term stands in, as the status line says) | `src/render/shaders/shading.wgsl:260-266` |
| 0020 | "Direct light is one hard shadow ray per pixel per light" | matches | `src/render/shaders/shadow_mask.wgsl:124-178`, `:244-269` |
| 0020 | "where a directional light's verdict differs from a neighbour's, or where the surface's nearest sample turns from it and another of its samples faces it, one from each other sample that shows the same surface" | matches | `src/render/shaders/shadow_mask.wgsl:516-631` |
| 0020 | amendment: "from `docs/research/shadow-edge-antialiasing.md`'s option B ...; on `town` the extra rays cost ... 2.7 to 3.1 ms, against a 4 ms ceiling" | matches for the file; the costs are not checkable | `docs/research/shadow-edge-antialiasing.md` |
| 0020 | amendment: "such pixels now join the edge rays, as do those along a smooth surface's terminator, which now resolves per sample, and the edge rays list 13.8% ..., cost ... 0.57 to 0.60 ms, and halve the error ..." | matches for the mechanism; the figures are not checkable | `src/render/shaders/shadow_mask.wgsl:551-590`, `:600-631` |
| 0020 | "option C of `docs/research/global-illumination-and-lighting.md`, and it supersedes ADR 0008's lighting clause" | matches (file exists; ADR 0008 is another reader's) | `docs/research/global-illumination-and-lighting.md` |
| 0020 | "Direct light is Temporal class (a); indirect light is class (b), and class (c) is used nowhere" | matches for direct light (no history read in the main pass or the shadow mask pass); indirect is planned | `src/render/shadow_mask_pass.rs:1-11`; `src/render/targets.rs:54-94` |
| 0020 | "each Mesh gets a compacted BLAS at load" | drift (L7-B08: a bent Mesh gets none and a refitted one is never compacted) | `src/assets/pools.rs:19-27`, `:314`, `:341-343` |
| 0020 | "the static TLAS covers the static Instances in a frame anchored at load, rebuilt in the frame its static Instances change, as when one is spawned or despawned" | matches (tested) | `src/render/acceleration_structures.rs:100-180`, `:650-712` |
| 0020 | "One hard shadow ray per pixel is traced toward each directional light against the static TLAS, skipped where the light adds nothing (... a light the surface's face turns from is hidden without one), from an origin offset along the surface's normal and composed in the TLAS's frame" | matches | `src/render/shaders/shadow_mask.wgsl:96-104`, `:136-162`, `:225-227`, `:279-283` |
| 0020 | "and one toward each point light that reaches the pixel, ending at the light" | matches | `src/render/shaders/shadow_mask.wgsl:295-310` |
| 0020 | "A compute pass traces them from the depth target, at each pixel's nearest sample and, where its other samples show another surface, at the farthest of those too, each fragment then reading its own surface's" | matches | `src/render/shaders/shadow_mask.wgsl:388-435`, `:481-493` |
| 0020 | "at a pixel whose verdict for one of the first seven directional lights differs from a 4-neighbour's among the lights both traced, or whose nearest sample turns from that light while another sample of its nearest surface faces it, toward that light again from each other sample ..., the fragment shading it by the share of its samples that see it" | matches | `src/render/shadow_mask_pass.rs:48-52`; `src/render/shaders/shadow_mask.wgsl:520-631` |
| 0020 | "where the nearest sample turns from a point light while another sample of its nearest surface faces it, the surface's one bit for that light is the verdict of a ray from the first such sample" | matches | `src/render/shaders/shadow_mask.wgsl:311-330` |
| 0020 | "into a shadow mask the lit pass reads, which holds 59 lights per pixel: a light past them casts no shadow there, and Frame stats count it" | matches | `src/render/shadow_mask_pass.rs:32`; `src/stats/mod.rs:95-97`, `:136-141` |
| 0020 | "Moving Instances form the dynamic TLAS, rebuilt each frame in the static TLAS's frame from at most 10,000 of them, those in the lowest slots" | matches | `src/render/acceleration_structures.rs:15-19`, `:208-216` |
| 0020 | "a shadow ray the static TLAS lets through is traced against it" | matches | `src/render/shaders/shadow_mask.wgsl:85-121` |
| 0020 | "its build's recording and submit, and the build itself on the GPU, are timed in each frame that builds it" | matches | `src/render/renderer.rs:64-70`, `:491-503`; `src/render/acceleration_structures.rs:21-23`; `bench/budgets.json:5-6` |
| 0020 | "The `crowd` Reference scene's figures, which the GPU bends each frame, have no BLAS and cast their shadows through rigid per-bone shadow proxies, Moving Instances in that TLAS, the default form" | matches | `src/render/bend_pass.rs:1-7`; `src/assets/pools.rs:19-23`; `src/assets/loader.rs:64-76`; `src/scenes/mod.rs:895-907`, `:961-963`, `:989-990`; `src/render/renderer.rs:146` |
| 0020 | "The crowd's refit Render switch instead gives each figure a BLAS, refitted to its bent shape each frame through wgpu-hal on Metal, its Instance a Moving Instance, and the dynamic TLAS's build waits for the refits on the GPU through a Metal shared event; the refits are timed on the GPU from the end of the bend pass to a marker after them" | matches (the switch is `crowd_form`, value `refit`; see Notes) | `src/render/blas_refit.rs:1-20`, `:29-43`; `src/render/renderer.rs:96-124`, `:597`; `src/scenes/mod.rs:1019` |
| 0020 | "A constant ambient term stands in for indirect light" | matches | `src/render/shaders/shading.wgsl:260-266`; `src/render/uniforms.rs:7-11` |
| 0020 | Considered options A, B, D, E, surfels, soft sun shadows (bake sizes, ms figures, penumbra arithmetic) | not checkable from the tree (rejected designs and measurements) | `docs/adr/0020-...md:7-12` |
| 0020 | "Ray queries are a required capability, not a high-preset path; cascaded shadow maps are dropped" | matches (floor feature; no shadow-map code in `src/`) | `src/gpu/mod.rs:268-269` |
| 0020 | "Indirect light converges to 90% ... within 0.5 s ... the probe update takes at most 2 ms of GPU ... If the update cannot meet both, this decision reopens" | planned (no probes) | `docs/adr/0020-...md:17` |
| 0020 | "No bake is needed for correct lighting. The one cooked lighting term is a per-Chunk sky-visibility field ... (ADR 0018's cooker)" | planned (no Chunk, no sky-visibility cook) | `src/cook/mod.rs:110-190` |
| 0020 | "Interiors are entered without a load. Walls thinner than the probe spacing leak ..., so authored Interior volumes mask probes" | planned | `docs/adr/0020-...md:19` |
| 0020 | "The TLAS holds the near field plus merged far proxies, so a 100k-Instance rebuild (6 to 8 ms of CPU ...) stays out of the frame; its radius is decided with Chunk streaming (M4) ..., and until then the static TLAS is built at scene load in a frame anchored there and never re-anchored" | matches for the "until then" half; far proxies are planned; the CPU figure is not checkable | `src/render/acceleration_structures.rs:100-106`, `:134-141` |
| 0020 | "a static TLAS in an anchor-relative frame (ADR 0014), rebuilt only when its static Instances change (...) or the anchor moves" | matches for the static-change rebuild; "or the anchor moves" is planned (never re-anchored in M3) | `src/render/acceleration_structures.rs:103`, `:134-180` |
| 0020 | "a dynamic TLAS of Moving Instances rebuilt each frame and capped at 10k Instances (0.64 to 0.92 ms of CPU ..., measured)" | matches for the cap and rebuild; the CPU figure is not checkable (its Budget is 1100 µs) | `src/render/acceleration_structures.rs:19`, `:208-216`; `bench/budgets.json:5` |
| 0020 | "Shadow and probe rays query both" | matches for shadow rays; probe rays are planned | `src/render/shaders/shadow_mask.wgsl:85-121` |
| 0020 | "Rough specular reads the irradiance probes; glossy specular reads reflection cubemaps ...; Mirror reflections ... return with water" | planned | `docs/adr/0020-...md:21` |
| 0020 | "Probes are smooth, so contact darkening is missing beyond what a Material's occlusion Channel authors (ADR 0018)" | planned for probes; the occlusion Channel exists | `src/assets/material.rs:51`; `src/cook/texture.rs:169` |
| 0020 | "Sun penumbrae reopen when area lights arrive or a Golden shows hard sun edges at distance" | matches (hard rays only, no penumbra; the reopen trigger is not checkable) | `src/render/shaders/shadow_mask.wgsl:124-178` |


## ADRs 0021 to 0024

I read the four ADRs in full in `docs/adr/` of the worktree at `8bc34eb18fcfb79307193551e96619316aef566c`: `0021-capability-floor.md`, `0022-fixed-64-hz-simulation-with-interpolation.md`, `0023-lights-are-photometric-as-gltf-defines-them.md` and `0024-bench-history-is-git-stored-bench-records-keyed-by-engine-source.md`, title, status paragraph, Considered options and Consequences, and gave every sentence that states what the code, tree, a test, a script, a Budget, a format or a dependency does a verdict row below. Against them I opened in full `src/gpu/mod.rs`, `src/time/mod.rs`, `src/render/shaders/hdr.wgsl`, `scripts/sweep.sh`, the `publish-bench` and bench jobs of `.github/workflows/ci.yml`, the README constant and the `path`, `tags`, `apply`, `unrecorded_scenes` and `same_definition` parts of `src/bench/history.rs`, and the header of `src/bench/history/import.rs`; and I grepped `src/`, `tests/`, `scripts/`, `bench/`, `.github/`, `Cargo.toml`, `Cargo.lock` and `docs/research/` for every feature, downlevel flag, limit, constant, format, path, test name and number the ADRs state (`TEXTURE_BINDING_ARRAY` through `INDIRECT_FIRST_INSTANCE`, `max_*` limits, `MULTISAMPLE_X4`, `ExperimentalFeatures`, `draw_indexed_indirect`, `first_instance`, `SAMPLE_COUNT`, `TICK`, `MAX_TICKS_PER_FRAME`, `MAX_FRAME_DELTA`, `LogLevel::Error`, `PreviousWorldTransform`, `Teleported`, `Instant`, `ev100`, `65504`, `100_000`, `GT7`, `emissive_strength`, `SpotLight`, `bench-history`, `MEASURED_PATHS`, `HARNESS_VERSION`, `QUIET_P95_OVER_MEDIAN`, `publishes_once`, `baseline-commit`, `HISTORY_STARTED`, `--no-publish`). I counted the culling pipeline's compute-visible storage buffers from its bind group definitions, counted `main`'s first-parent commits touching `bench/baselines/` before `HISTORY_STARTED` with `git log`, and checked `origin/bench-history` shares no ancestor with `HEAD`. Left to other readers: every registry quote inside the ADRs (the `wgpu-*`, `vulkan-*`, `mesa-*`, `d3d12-*`, `metal-*`, `gpuinfo-*`, `frostbite-*`, `bevy-*`, `gltf-*`, `khr-*`, `blender-*`, `box2d-*`, `physx-*`, `rust-*` keys), which I mark `registry` in the table; hardware and vendor claims I mark `not checkable from the tree`.

### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0021 | "Device creation requires a fixed set of wgpu features, downlevel flags and limits ... fails with every gap listed ... nothing in the floor has a fallback" | matches | src/gpu/mod.rs:54-66, :333-348 |
| 0021 | "drawn from Tiers 1 and 2 of `docs/research/wgpu-capability-matrix.md`" | matches: the file exists and has the tiers table | docs/research/wgpu-capability-matrix.md:191-199 |
| 0021 | "a Tier 3 to 6 feature is always optional, probed at runtime, with a path for adapters without it" | matches (vacuously: no Tier 3 to 6 feature is requested or used) | src/gpu/mod.rs:273-287 |
| 0021 | "`GpuContext` checks the adapter against the floor, then requests exactly its features, limits and experimental opt-in" | matches | src/gpu/mod.rs:54-80 |
| 0021 | Considered: "`SUBGROUP`, `SHADER_F16`, `SHADER_INT64`, `TEXTURE_ATOMIC` and `PRIMITIVE_INDEX`, which nothing uses" | matches: no WGSL part uses subgroups, f16, 64-bit integers, texture atomics or the primitive index; none requested | src/gpu/mod.rs:273-287; src/render/shaders/*.wgsl (grep) |
| 0021 | Considered: "Tiers 1 and 2 in full (matrix option D): adds mesh shaders, which ADR 0019 does not use" | matches: option D is the matrix's; `EXPERIMENTAL_MESH_SHADER` is only reported | docs/research/wgpu-capability-matrix.md:209; src/gpu/mod.rs:155-162, :273-287 |
| 0021 | Considered: "the M4 Pro allows (1,000,000 binding-array elements per stage, 256 bytes of immediates)" | matches the recorded M4 Pro limits | src/gpu/mod.rs:603, :629 |
| 0021 | Considered: "Mesa 25.2.8's `features.txt` lists lavapipe for `VK_KHR_ray_query` ..." | registry (`mesa-25.2.8-features-lavapipe-ray-query`); CI pins Mesa 25.2.8 | .github/workflows/ci.yml:29 |
| 0021 | Considered: "so no carve-out is planned" | matches: headless contexts request the same floor | src/gpu/mod.rs:31-35 |
| 0021 | Considered: "its limits are not yet measured" | drift (L7-C03) | docs/adr/0021-capability-floor.md:10; .github/workflows/ci.yml:58-90 |
| 0021 | "Required features: `TEXTURE_BINDING_ARRAY`, `PARTIALLY_BOUND_BINDING_ARRAY` and `SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING` ..., `TEXTURE_COMPRESSION_BC` ..., `EXPERIMENTAL_RAY_QUERY` ..., `TIMESTAMP_QUERY`" | matches | src/gpu/mod.rs:275-286 |
| 0021 | "4x MSAA on every attachment format the main pass uses, read per format from the adapter's texture format features (`MULTISAMPLE_X4`) ..., with its resolve (`MULTISAMPLE_RESOLVE`) on the colour formats" | drift (L7-C01) on "every": the mechanism matches, the format list the floor checks does not | src/gpu/mod.rs:55-58, :268-269, :349-357; src/render/targets.rs:45, :207-212 |
| 0021 | "Those formats are `Depth32Float` and the HDR target's `Rgba16Float`" | drift (L7-C01): a third multisampled attachment format, `Rgba8Unorm`, exists | src/render/targets.rs:12, :22, :45, :16, :207-212; src/render/main_pass.rs:240-244, :281 |
| 0021 | "the window's surface and the capture target take only the tonemapping pass's single-sample output" | matches | src/render/targets.rs:148; src/gpu/mod.rs:450-470 |
| 0021 | "The `ANISOTROPIC_FILTERING` downlevel flag is required too ... the colour Channels at 16x and the data Channels at 4x" | matches | src/gpu/mod.rs:296-300; src/assets/material.rs:28-29, :201-202 |
| 0021 | "So is `VERTEX_STORAGE`: the vertex shader reads Instances and pulls its vertices from the vertex Arena through storage buffers" | matches | src/gpu/mod.rs:301-303; src/render/bind_groups.rs:74-83 |
| 0021 | "Vulkan, Metal and DX12 report it on every adapter in wgpu-hal 30.0.1" | registry (`wgpu-hal-30-*-downlevel-*`); the pinned version matches | Cargo.lock:3047-3048 |
| 0021 | "So are `COMPUTE_SHADERS` and `INDIRECT_EXECUTION`: a compute pass builds the frame's draw and the main pass draws it with one `draw_indexed_indirect`" | drift (L7-C02) on "one"; the flags match | src/gpu/mod.rs:304-305; src/render/main_pass.rs:262, :296, :330, :333 |
| 0021 | "`INDIRECT_FIRST_INSTANCE` ..., which left the floor, since the one draw spans every Instance through packed indices and draws from first instance 0" | matches for first instance 0 (the buffer is cleared each frame and the shader never writes the field); "the one draw" is L7-C02 | src/gpu/mod.rs:273-287; src/render/culling_pass.rs:294; src/render/shaders/culling.wgsl:20-28, :332-335 |
| 0021 | "`TIMESTAMP_QUERY_INSIDE_ENCODERS` and `_INSIDE_PASSES` stay optional" | matches: not in the floor; passes use `timestamp_writes` only, no `write_timestamp` in `src/` | src/gpu/mod.rs:273-287; src/render/culling_pass.rs:290-301 |
| 0021 | "the M4 Pro reports neither on wgpu 30.0.1" | not checkable from the tree (hardware); the recorded M4 Pro features agree | src/gpu/mod.rs:538-544 |
| 0021 | "Ray queries need `ExperimentalFeatures::enabled()`, an `unsafe` call ...; it is the one `unsafe` this ADR adds, and its comment cites ADR 0020" | matches | src/gpu/mod.rs:67-74 |
| 0021 | "Required limits are `wgpu::Limits::default()` with `using_minimum_supported_acceleration_structure_values()`" | matches | src/gpu/mod.rs:328 |
| 0021 | "`max_buffer_size` at least 2^31 − 1" | matches | src/gpu/mod.rs:320; :664-668 |
| 0021 | "`max_binding_array_elements_per_shader_stage` at least 65,536" | matches | src/gpu/mod.rs:313, :321 |
| 0021 | "`max_acceleration_structures_per_shader_stage` at least 2 (ADR 0020's static and dynamic TLAS)" | matches | src/gpu/mod.rs:323; src/render/bind_groups.rs:110-113 |
| 0021 | "`max_storage_buffers_per_shader_stage` at least 9, since the culling pass's compute stage reads and writes nine" | matches: four compute-visible read-only buffers in group 2 plus five read-write in group 3 | src/gpu/mod.rs:326; src/render/bind_groups.rs:74-95, :127-131; src/render/renderer.rs:264-269; src/render/culling_pass.rs:232-238 |
| 0021 | "wgpu-hal 30.0.1 reports 29 on Metal, 262,144 (2^18) on DX12 ..., and lavapipe's 1,000,000 halved on Vulkan" | registry / hardware; the Metal 29 agrees with the recorded M4 Pro | src/gpu/mod.rs:600 |
| 0021 | "`max_storage_buffer_binding_size` stays at the default 128 MiB" | matches: not overridden | src/gpu/mod.rs:318-329 |
| 0021 | "the vertex shader reads the whole vertex Arena through one storage binding ..., so that Arena stays within it, as a test checks" | matches | src/gpu/mod.rs:752-761; src/render/bind_groups.rs:79-83 |
| 0021 | "A ticket that needs more raises the floor in its PR, and adds Tier 1 or 2 features the same way without a new ADR" | not checkable from the tree (process) | — |
| 0021 | "`GpuContext` checks the whole floor before requesting the device and returns one typed error naming each missing feature and downlevel flag, each attachment format without 4x MSAA or, on colour, its resolve, and each limit with its required and adapter values" | matches | src/gpu/mod.rs:333-348, :360-425, :670-712 |
| 0021 | "the binary logs it with `tracing::error!` and exits non-zero" | matches | src/bin/praxis.rs:15-27 |
| 0021 | "`praxis capabilities` runs without the floor, since it diagnoses that failure" | matches | src/gpu/mod.rs:113-120; src/bin/praxis.rs:98, :346 |
| 0021 | "Headless contexts, tests and Goldens included, use the same floor" | matches | src/gpu/mod.rs:31-35 |
| 0021 | "If lavapipe misses part of it, the implementing PR stops and brings the gap to the owner" | not checkable from the tree (process); CI requests the floor on lavapipe | .github/workflows/ci.yml:58-90 |
| 0021 | "Hardware out of scope follows from the floor: macOS before 15, DX12 below raytracing Tier 1.1 ..., Vulkan devices without `VK_KHR_ray_query` (52% ...)" | not checkable from the tree (hardware); the percentage is registry (`gpuinfo-vulkan-ray-query-coverage`) | — |
| 0021 | "The full floor, ray queries included, is enforced in M1, before ray queries are first used in M3" | matches: the floor requests ray queries and the shadow mask pass uses them | src/gpu/mod.rs:283; src/render/shadow_mask_pass.rs |
| 0022 | "Simulation advances in fixed Ticks of 1/64 s (15.625 ms)" | matches | src/time/mod.rs:20-21 |
| 0022 | "each frame draws a blend of the two newest Tick states" | matches | src/transform/mod.rs:21-33; src/render/prepare.rs:308-313, :383 |
| 0022 | "(`docs/research/simulation-stepping.md`, option C)" | matches: the doc's option C is "Fixed step with interpolation" | docs/research/simulation-stepping.md:101 |
| 0022 | "`src/time/` runs the fixed schedule once per Tick, at most 4 per frame" | matches | src/time/mod.rs:27, :80-87 |
| 0022 | "with ambiguity detection at `Error`" | matches | src/time/mod.rs:70-73 |
| 0022 | "the Tick counter and blend weight α in `FixedTime`" | matches | src/time/mod.rs:38-47 |
| 0022 | "the dropped time in frame stats" | matches | src/stats/mod.rs:60, :166; src/state.rs:256 |
| 0022 | "the helmet spawn and despawn run in it" | matches | src/state.rs:332-334 |
| 0022 | "World transforms propagate at the end of every Tick" | matches | src/state.rs:339 |
| 0022 | "an entity the fixed schedule moves keeps its previous Tick's World transform in `PreviousWorldTransform`, and prepare blends the two in f64" | matches: `DAffine3` blend | src/transform/mod.rs:21-33, :40-41, :52-58; src/render/prepare.rs:383 |
| 0022 | "a spawned entity, or one marked `Teleported`, is drawn at its current World transform until the next Tick" | matches: `None` draws the current transform | src/transform/mod.rs:22-24, :36-41, :71-84 |
| 0022 | "The `orrery` Reference scene shows it" | matches | src/state.rs:83; src/scenes/mod.rs:369-371 |
| 0022 | Considered: option A "used until" the fixed loop; p99, mm figures; Box2D and PhysX advice | history and measurements: not checkable from the tree; advice is registry (`box2d-3.1.1-fixed-step`, `physx-5.6.1-variable-step`) | — |
| 0022 | Considered: options B, D, E; 60 Hz and 120 Hz figures | measurements and design rationale: not checkable from the tree | — |
| 0022 | "The frame delta is measured with `Instant` at frame start, before surface acquire" | matches | src/lib.rs:221-227 |
| 0022 | "clamped to 250 ms" | matches | src/time/mod.rs:18, :33-35 |
| 0022 | "The fixed loop runs at most 4 Ticks per frame; accumulated time past the cap is dropped, the game slows, and the dropped time is reported in frame stats" | matches | src/time/mod.rs:53-63, :200-222; src/stats/mod.rs:60 |
| 0022 | "Snapping deltas to refresh periods waits for present timing in wgpu" | matches: the measured delta is used unsnapped; the wgpu claim is external | src/lib.rs:221-224 |
| 0022 | "Simulated time is an integer Tick counter; systems read a per-Tick delta derived from it, never an f32 sum of seconds" | matches | src/time/mod.rs:24, :41-42; src/scenes/mod.rs:664-665, :1063-1068 |
| 0022 | "an accumulator resource, a fixed schedule run once per Tick, and the blend weight α as a resource" | matches: one resource, `FixedTime`, holds the accumulator and α | src/time/mod.rs:38-47 |
| 0022 | "Everything that changes game state runs in the fixed schedule: gameplay, physics, AI and Lua event reactions" | matches for what exists (spin, sway, helmet spawn and despawn); physics, AI and Lua: planned | src/state.rs:328-343 |
| 0022 | "Input sampling, the camera, the blend, prepare and presentation-only effects run once per frame" | matches: `fly_camera` is the per-frame schedule; prepare runs per frame; `clear_just_pressed` runs per Tick by design | src/state.rs:320, :335-337; src/lib.rs:248 |
| 0022 | "The camera's rotation reads the newest input every frame, outside the fixed loop" | matches | src/state.rs:320; src/camera/mod.rs:99 |
| 0022 | "its position follows whatever it is attached to, interpolated" | drift (L7-C04) | src/camera/mod.rs:99; src/state.rs:320 |
| 0022 | "Entities the fixed schedule moves keep their previous Tick's World transform; prepare blends previous and current in f64 into Frame data, which stays one owned state" | matches | src/transform/mod.rs:52-58; src/render/prepare.rs:55, :383 |
| 0022 | "Static Instances store nothing extra" | matches: `PreviousWorldTransform` is optional in prepare's query | src/transform/mod.rs:36-39; src/render/prepare.rs:55, :66 |
| 0022 | "A spawned entity starts with previous equal to current" | matches in effect: `None` until the next Tick, drawn at current | src/transform/mod.rs:36-41 |
| 0022 | "a teleport goes through an explicit marker that resets previous to current" | matches in effect: `Teleported`, reset to `None` | src/transform/mod.rs:43-48, :71-84; src/state.rs:325-338 |
| 0022 | "Blending 100k Instances costs 0.12 to 0.29 ms more than copying them on the M4 Pro" | measurement: not checkable from the tree | — |
| 0022 | "Inside the fixed schedule, bevy_ecs ambiguity detection is `Error`" | matches | src/time/mod.rs:70-73 |
| 0022 | "no system reads the wall clock" | matches: fixed systems read `FixedTime`; `Time` is per frame and the fixed schedule never reads it | src/time/mod.rs:9-14; src/scenes/mod.rs:664, :1064; src/helmet/mod.rs (no `Instant` or `Time`) |
| 0022 | "randomness comes from a seeded resource advanced per Tick" | matches (vacuously): no fixed-schedule system draws randomness; the town's generator hashes coordinates at spawn | src/scenes/town.rs:255-272 |
| 0022 | "no result depends on hash-map iteration order or on reduction order in a parallel iterator" | not checkable from the tree mechanically; the fixed schedule is single-threaded | src/time/mod.rs:68 |
| 0022 | "Nothing is guaranteed across platforms or builds: `sin` and its kin are not bit-stable" | registry (`rust-1.96-f64-sin-precision`) | — |
| 0022 | "ADR 0013's budget splits, judged at p95 once Input latency is measured: ... 33.3 ms ..., 49.0 ms" | planned: no input latency metric or Budget exists yet, as the clause's "once ... measured" says | bench/budgets.json:1-8 |
| 0022 | "The blend reads two simulation states, never screen-space history" | matches | src/transform/mod.rs:21-33 |
| 0022 | "Budgets stay judged at 60 Hz, the display the measurements ran on" | matches: the bench renders at a 60 Hz delta | src/bench/mod.rs:453-454; src/time/mod.rs:30 |
| 0022 | "the step model must also stay judder-free at refresh rates that do not divide 64 Hz (120, 144, variable)" | partly checkable: the orrery capture runs at 144 Hz; judder is a visual claim | src/scenes/mod.rs:369-371 |
| 0022 | "A physics engine chosen in M7 steps once per Tick" | planned | — |
| 0023 | "a `DirectionalLight` gives illuminance in lux and a `PointLight` luminous intensity in candela" | matches | src/components.rs:83, :99 |
| 0023 | "emissive and background give luminance in cd/m²" | matches: background is `background_luminance`; emissive adds to luminance unscaled | src/environment/mod.rs:5-11; src/render/shaders/main.wgsl:301-302 |
| 0023 | "a Reference scene's ambient term gives illuminance in lux" | matches | src/environment/mod.rs:10; src/scenes/mod.rs:463 |
| 0023 | "Exposure is an EV100, which scales luminance by 1/(1.2 · 2^EV100)" | matches | src/camera/mod.rs:11-15, :44-49 |
| 0023 | "glTF lights then import without a scale" | not checkable from the tree: the cooker imports no glTF lights (the `gltf` crate's `KHR_lights_punctual` feature is off); scene lights are code constants | Cargo.toml:16; src/scenes/mod.rs:422-445 |
| 0023 | "ADR 0020's ray-traced shadows and probe GI work on real ratios" | shadows: matches (the shadow mask pass); probe GI: planned (M5) | src/render/shadow_mask_pass.rs |
| 0023 | "29.9 stops from a moonless night to direct sun" | registry (`bevy-lux-constants`) | — |
| 0023 | "(`docs/research/hdr-exposure-and-tonemapping.md`, option U1, rows 11, 14 and 15)" | matches | docs/research/hdr-exposure-and-tonemapping.md:21, :24, :25, :120 |
| 0023 | "every Reference scene's lights, ambient and background are in these units, under a sun of 100,000 lx" | matches: every scene spawns `sun()` at `SUN_ILLUMINANCE` | src/scenes/mod.rs:422-445, :448-466, :470-473; src/state.rs:95 |
| 0023 | "each scene has an EV100, which the lit pass applies before the `Rgba16Float` HDR target stores its colour, clamped to 65504" | matches | src/scenes/mod.rs:347, :354-367; src/render/shaders/main.wgsl:303-306; src/render/shaders/hdr.wgsl:7; src/render/targets.rs:22 |
| 0023 | "the tonemapping pass then maps it through GT7 for display" | matches | src/render/shaders/mod.rs:70-71, :141; src/render/shaders/display.wgsl:20-23 |
| 0023 | Considered: relative units (U2) and lumens; Frostbite, Filament, Bevy, Godot practice | registry (`frostbite-2014-*`, `filament-light-units`, `bevy-point-light-lumens`, `godot-light-units`, `bevy-gltf-point-candela`) | — |
| 0023 | "under 100,000 lx, a dielectric's sun glint at roughness 0.04 peaks at 1.24×10⁸ cd/m², 10.9 stops past fp16's 65504" | computed figure: matches the research doc's row 15; not checkable against code | docs/research/hdr-exposure-and-tonemapping.md:25 |
| 0023 | "Shading runs in f32 and pre-exposes colour before fp16 stores it" | matches | src/render/shaders/main.wgsl:303-306 |
| 0023 | "every path clamps it to 65504" | drift (L7-C05): the clear colour path does not | src/render/prepare.rs:301-307; src/render/main_pass.rs:318-321 |
| 0023 | "on overflow the M4 Pro saturates while Vulkan may store infinity" | not checkable from the tree (hardware); registry (`vulkan-fp-conversion`, `frostbite-2014-fp16`, `unreal-scene-color-overflow`) | src/render/shaders/hdr.wgsl:4-6 |
| 0023 | "Each Reference scene sets its EV100 by hand in M3, which keeps Goldens exact" | matches: one fixed EV100 per scene (16.35 for all) | src/scenes/mod.rs:351-367 |
| 0023 | "Histogram adaptation and local tone mapping wait for the first scene that joins an interior and sunlight" | planned | — |
| 0023 | "Core glTF caps emissive at 1 cd/m² ..., which an exposure of EV100 15 maps to 2.5×10⁻⁵ of display white" | registry (`gltf-2.0-emissive-factor-schema`); the arithmetic agrees with the camera's 1/39,321.6 at EV100 15 | src/camera/mod.rs:128-134 |
| 0023 | "Brighter emissives need KHR_materials_emissive_strength, which the cooker does not read yet" | matches: the cooker reads `emissive_factor` only and the `gltf` crate's extension feature is off | src/cook/mod.rs:471; Cargo.toml:16 |
| 0023 | "Point lights fall off by inverse square inside KHR_lights_punctual's range window" | matches | src/render/shaders/shading.wgsl:229-242; src/components.rs:85 |
| 0023 | "Spot lights, out of M3, take candela too" | planned: no spot light component exists | src/components.rs |
| 0023 | "Blender's glTF exporter, in its physically based lighting mode (`SPEC`), writes ..." | registry (`blender-gltf-light-export`) | — |
| 0024 | "Every bench run is kept as a Bench record" | matches: `praxis bench` publishes unless `--no-publish`; a duplicate under its key is the one exception | src/bin/praxis.rs:183, :226, :467-480; src/bench/history.rs:952-980 |
| 0024 | "A record is a file on an orphan branch, `bench-history`, in this repo" | matches: `origin/bench-history`'s root shares no ancestor with `HEAD` | src/bench/history.rs:21; git merge-base |
| 0024 | "keyed by source hash × adapter × backend × scene × Harness version" (status) and "The key is source hash × adapter × backend × scene × Harness version" (Consequences) | drift (L7-C06): the path omits the Harness version | src/bench/history.rs:204-212 |
| 0024 | "The source hash is one git hash over the paths that can change a measurement" | matches | src/bench/provenance.rs:26, :225 |
| 0024 | "Only clean, release-profile records of `main`'s source taken by a quiet machine form a Series" | matches | src/bench/history.rs:808-840 |
| 0024 | "The history observes and never gates: Baselines stay in `bench/baselines/` on `main`, and `verify` and `compare` judge against them as before" | matches | bench/baselines/; scripts/verify.sh:49-52; .github/workflows/ci.yml:176-181 |
| 0024 | "Records publish themselves: CI's lavapipe on every push to `main`" | matches, with a note: CI skips pushes touching only `docs/**` or `*.md`, whose source hash is unchanged, so no record is missing | .github/workflows/ci.yml:7-11, :214-222 |
| 0024 | "`praxis bench` by default in sessions" | matches | src/bin/praxis.rs:183, :226 |
| 0024 | "a nightly sweep on the reference machine and the RTX 2070 SUPER that benches `main` only when its source has no quiet record and the machine is quiet" | matches: `history unrecorded` reads `records/` only; the sweep exits on any preflight warning | scripts/sweep.sh:73-86, :180-186; src/bench/history.rs:596-621; scripts/sweep.plist; scripts/sweep-task.xml |
| 0024 | "A page in `docs/` draws the Series" | matches | docs/bench/index.html |
| 0024 | Considered: Bencher.dev, github-action-benchmark, Grafana, commit or tree hash, self-hosted runners and GitHub's guidance | external and design rationale: not checkable from the tree | — |
| 0024 | "The source hash covers `src/`, `assets/`, `Cargo.toml`, `Cargo.lock` and `.cargo/`, a list held in one place in code" | matches | src/bench/provenance.rs:26 |
| 0024 | "a test fails when a new top-level path is classified neither way" | matches | src/bench/provenance.rs:310-311 |
| 0024 | "A commit whose source hash didn't change inherits the record before it" | matches: a record is looked up by `main`'s source hash | src/bench/history.rs:596-621, :206-207 |
| 0024 | "A record path is its key, so a duplicate is a file that already exists" | matches, modulo L7-C06 | src/bench/history.rs:204-212, :950-951 |
| 0024 | "When two records share a key, the quieter one stays: the one with the lower GPU p95/median" | matches | src/bench/history.rs:961-980 |
| 0024 | "the machine quiet (preflight warned nothing, and p95/median below about 1.06)" | matches | src/bench/provenance.rs:30, :126-139, :424-447 |
| 0024 | "lavapipe's records are the exception to the quiet test ... a record from CI's configuration joins them whether or not the runner was quiet, and the first record on a Series under a key stays" | matches | src/bench/history.rs:161, :830, :966; :1866 |
| 0024 | "Every other record is kept, tagged with its reasons, and drawn only on request" | matches: `tagged/` and the page's "Tagged records" switch, off by default | src/bench/history.rs:24-25, :808-840; docs/bench/index.html:156, :303, :393 |
| 0024 | "An Experiment's records are kept under its name and never join a Series" | matches | src/bench/history.rs:29, :184-201, :944-949 |
| 0024 | "The Harness version rises whenever a change alters what a metric means" | matches | src/bench/provenance.rs:21; src/bench/history/import.rs:25-60 |
| 0024 | "each record carries its scene's content hash" | matches (imported Baselines lack it, as the README says) | src/bench/history.rs:261-262, :1109 |
| 0024 | "A chart breaks its line where either changes, so a new definition never reads as a Regression" | matches | docs/bench/index.html:361-368; src/bench/history.rs:1141-1148 |
| 0024 | "The 61 commits that touched `bench/baselines/` before the history existed are imported as records" | drift (L7-C07): 64 first-parent commits | src/bench/history/import.rs:19-22; git log |
| 0024 | "each with the Harness version in force at its commit" | matches | src/bench/history/import.rs:25-60 |
| 0024 | "Publishing never blocks work. A failed publish warns and the bench still succeeds" | matches | src/bin/praxis.rs:467-480 |
| 0024 | "`verify` never publishes" | matches | scripts/verify.sh:52 |
| 0024 | "Baselines are still chosen by hand" | matches | .github/workflows/ci.yml:183-190; bench/baselines/ |
| 0024 | "a re-recorded Baseline's record is marked as one" | drift (L7-C08) | src/bench/history.rs:258-260; src/bench/history/import.rs:19-22 |
| 0024 | "A timing record far from its Series' recent median is a suspect" | matches | src/bench/history.rs:77-83; src/bench/history/suspects.rs |
| 0024 | "The sweep re-benches it, bisects the commits behind it, and files a confirmed Regression that no re-recorded Baseline covers as one `needs-triage` issue" | matches | scripts/sweep.sh:131-176; src/bench/history/verdicts.rs:424-432, :761 |
| 0024 | "A bad publish is undone with `git revert` on `bench-history`" | matches (the README says so; no code path reverts) | src/bench/history.rs:103 |
| 0024 | "The branch holds only data and a README, is never merged" | matches | src/bench/history.rs:42-46, :1000; git merge-base |
| 0024 | "CI writes it only from pushes to `main`, never from a pull request" | matches: `contents: read` at the workflow, `contents: write` on `publish-bench` alone, gated on `push` | .github/workflows/ci.yml:18-19, :214-222 |


## The glossary

I read every bold term of `CONTEXT.md` at the frozen commit `8bc34eb18fcfb79307193551e96619316aef566c` in the worktree `/Users/guillem.arias/Documents/gariasf/praxis-512` (58 terms, `CONTEXT.md:9` to `CONTEXT.md:249`), and for each one opened the code that defines what the definition claims: the component, resource, constant, pass, struct or shader part, and the test or doc comment that pins its number. Numbers in definitions (59 lights, 64 m, 1/64 s, 8 records, 4 median absolute deviations, 3%, half the shift, 10,000 Moving Instances) were checked against the constants that hold them (`SHADOW_MASK_LIGHTS`, `ANCHOR_CELL_SIZE`, `TICK`, `WINDOW_RECORDS`, `MEDIAN_ABSOLUTE_DEVIATIONS`, `SMALLEST_SHIFT_FRACTION`, `SMALLEST_HELD_SHIFT`, `DYNAMIC_TLAS_CAPACITY`). Terms that name future scope (Chunk, Game world, Placement, Interior volume, Input latency, Resident tail) were checked for any code counterpart by `grep -rn` over `src/` and `tests/` and marked `planned`. The five audit-process terms were read against `docs/agents/audit.md` and `docs/agents/workflow.md`, which are their defining text. For the `_Avoid_` lines I grepped item declarations (`struct`, `enum`, `fn`, `const`, `type`) in `src/` and `tests/`, and WGSL identifiers in `src/render/shaders/`, for every banned word; each hit was read to decide whether the glossary's term was plainly what the identifier meant. Nine Findings result, all `drift`: two definitions false of the bench history's data model, two Budget clauses that state ADR scope as present, one slot rule, one shadow-edge cap, one Meshlet capability, and two glossary terms whose names the code gives to other things.

### Term verdicts

| Term | Verdict | Where |
|---|---|---|
| Renderable | matches | `src/render/prepare.rs:62-69` (`RenderableData`: `WorldTransform`, `MeshRef`, `Option<MaterialRef>`); `src/components.rs:9-15` (`Transform` requires `WorldTransform`); `src/render/shaders/culling.wgsl:385` (the Instance's Material ID overrides the Meshlet's) |
| Mesh | matches | `src/assets/cooked.rs:487-496` (`CookedMesh` with `primitives: Vec<CookedPrimitive>`); `src/assets/mesh.rs:1-12` |
| Primitive | matches | `src/assets/cooked.rs:405-413` (`CookedPrimitive`: `first_index`, `index_count`, one Material's run of the indices) |
| Material | matches | `src/assets/material.rs:65-75` (`MaterialData`: factors plus `layers` and the emissive Layer); `src/assets/cooked.rs:131-135` |
| Channel | matches | `src/assets/material.rs:46-53` (five Channels); `src/assets/material.rs:147-148` (`colour_sampler`, `data_sampler`); `src/assets/material.rs:180-190` and `src/cook/texture.rs:162-169` (colour Channels sRGB, data Channels linear) |
| Layer | matches | `src/assets/material.rs:156-160` (`DEFAULT_LAYER = 0`, a 1x1 neutral texture per Channel); `src/assets/material.rs:77-78` |
| Texture | matches | `src/assets/material.rs:95-134` (`ChannelTextures::push` creates one texture with its full mip chain and returns its Layer); `src/assets/material.rs:145-146` |
| Resident tail | planned | no code: every mip is uploaded at load, `src/assets/material.rs:127-129`; `docs/adr/0017-*.md:3` ("still without streaming") and `:16` (the tail's rule) |
| Pool | matches | `src/assets/pools.rs:57-80` (`MeshPool`); `src/assets/material.rs:137-153` (`MaterialPool`); `src/components.rs:43-47` (components hold Handles) |
| Handle | matches | `src/assets/mesh.rs:14-15` (`MeshHandle(pub u32)`); `src/assets/material.rs:31-40` (`MaterialHandle`, `index()`) |
| Arena | matches | `src/assets/pools.rs:523-530` ("A GPU buffer of fixed capacity, never grown, sub-allocated in ranges"); `src/assets/pools.rs:74-78` (vertex, index, Meshlet, Meshlet vertex, Meshlet triangle Arenas) |
| Material ID | matches | `src/assets/mesh.rs:32-44` (`MeshletData::material_id`); `src/render/instance.rs:28-30` (`NO_MATERIAL`); `src/render/prepare.rs:392-393` (an Instance carries the Handle's index only with a `MaterialRef`) |
| Asset | matches (Chunk: planned) | `src/assets/cooked.rs:49-56` (`AssetKind`: `Mesh`, `Material`, `Texture(MaterialChannel)`; no Chunk) |
| Asset ID | matches | `src/assets/cooked.rs:58-73` (`AssetSource::File(guid)`, `AssetSource::Scene(name)`, `AssetName::Name` or `Index` "when the name is missing or not unique"); `src/assets/cooked.rs:75-118` |
| Blob | matches | `src/assets/cooked.rs:1-4` ("Blobs named by the SHA-256 of their bytes"); `src/assets/cooked.rs:745-755` (`write` stores `blob` under `hex(Sha256::digest(blob))`) |
| Cook key | matches | `src/cook/mesh.rs:37-40` and `:41-70` (format, encoder versions and settings, each Primitive's geometry and Material); `src/cook/texture.rs:116-141`; `src/cook/mod.rs:322-328` (`material_cook_key`); `src/cook/mod.rs:331-336` (`is_fresh`) |
| Cooked cache | matches | `src/assets/cooked.rs:655-661` (`blobs/<SHA-256 of the Blob>` and `index/<Asset ID>`); `src/cook/mod.rs:1-3` (rebuilt by `praxis cook`) |
| Placement | planned | no code: `src/scenes/town.rs:343` (`fn placements()`) is the town generator's own list, inside no Chunk |
| Instance | drift (L7-G05) | `src/render/instance.rs:32-45` (`InstanceData`: `anchor`, `offset`, `rotation_scale`, `mesh`, `material_id`); `src/render/prepare.rs:135-138` (slot "kept until it stops being a Renderable or is not drawn") |
| Moving Instance | matches | `src/render/prepare.rs:249-250` ("An Instance is a Moving Instance while its entity carries its previous World transform"); `src/render/prepare.rs:81-88` (`With<PreviousWorldTransform>` is rewritten each frame); `src/render/prepare.rs:38-41` (`moving_instances` for the dynamic TLAS) |
| Bend | matches | `src/components.rs:49-69` (`Bend { rest, azimuth, curvature, previous_curvature }`, "the fixed schedule sets the curvature each Tick"); `src/render/bend_pass.rs:19` (`BEND_CAPACITY`) |
| Shadow proxy | matches | `src/assets/pools.rs:22-24` (`MeshUse::Traced`: "a shadow proxy, which no pass draws"); `src/scenes/mod.rs:895-907` (one per bone of a figure's middle) |
| Frame data | matches | `src/render/prepare.rs:18-45` (`FrameData`: camera, lights, `instances`, `instance_writes`, `moving_instances`, `bends`; "It lists no draws") |
| Froxel | matches | `src/render/froxel_pass.rs:1-4` and `:12-29` (`FROXEL_COLUMNS`, `FROXEL_ROWS`, `FROXEL_SLICES`, a list of lights per Froxel) |
| Shadow mask | drift (L7-G06) | `src/render/shadow_mask_pass.rs:27-32` (`SHADOW_MASK_LIGHTS = 59`); `src/render/shadow_mask_pass.rs:48-52` (`SHADOW_EDGE_LIGHTS = 7`); `src/render/shaders/shadow_mask.wgsl:1-19`, `:227-228`, `:311-338`, `:600-609`; `src/render/targets.rs:24-40`; `src/render/prepare.rs:550-561` (lights that cast none follow the others) |
| Surface normals | matches | `src/render/targets.rs:42-45` (`SURFACE_NORMALS_FORMAT`: triangle normal then interpolated normal per sample); `src/render/main_pass.rs:240-247` and `:265-282` (written by the depth passes with shadow rays on); `src/render/shaders/shadow_mask.wgsl:129-131` |
| Meshlet | drift (L7-G07) | `src/assets/cooked.rs:38-44` (`MESHLET_MAX_VERTICES = 64`, `MESHLET_MAX_TRIANGLES = 124`); `src/assets/mesh.rs:37-44` (`MeshletData` holds no bounds); `src/render/shaders/culling.wgsl:1-10` (culling tests Instances' Mesh bounds) |
| Temporal class | matches (no code counterpart) | `docs/adr/0020-*.md:3` ("Direct light is Temporal class (a); indirect light is class (b), and class (c) is used nowhere") |
| Interior volume | planned | no code; `docs/adr/0020-*.md:19` |
| Input latency | planned | no code measures it; `docs/adr/0022-*.md:21` ("judged at p95 once Input latency is measured") |
| Tick | matches | `src/time/mod.rs:20-21` (`TICK = Duration::from_nanos(15_625_000)`, "One Tick, 1/64 s"); `src/time/mod.rs:43-45` (`blend_weight`); `src/transform/mod.rs:16-31` (`drawn` blends the two newest Ticks) |
| ECS World | matches | `bevy_ecs::prelude::World`, used as the container at `src/render/prepare.rs:336` and `src/time/mod.rs:80` |
| Game world | planned | no code; the phrase appears only in doc comments, `src/components.rs:6`, `src/transform/mod.rs:5` |
| Chunk | planned | no code (`grep -rn '\bChunk\b' src tests` finds none) |
| World position | matches | `src/components.rs:6-12` (`Transform::translation: glam::DVec3`, "translation in f64 metres"); `src/render/instance.rs:10-17` (camera-relative and Anchor-relative derive from it) |
| World transform | matches | `src/transform/mod.rs:5-9` (`WorldTransform(pub glam::DAffine3)`: "its `Transform` composed with every ancestor's, in f64 ... never authored"); `src/transform/mod.rs:90-95` (`propagate_world_transforms`) |
| Anchor | matches | `src/render/instance.rs:6-8` (`ANCHOR_CELL_SIZE: f64 = 64.0`); `src/render/instance.rs:10-17` (`anchor_and_offset`: integer cell plus f32 offset from its corner) |
| Scene | matches (serialisable: planned) | `src/scenes/mod.rs:537-1130` (each `spawn_*` loads a set of entities and their hierarchy into the `World`); no scene serialisation exists; `src/cook/export.rs:29-40` writes a Reference scene's frame for a reference renderer, which does not load back |
| Reference scene | matches | `src/scenes/mod.rs:24-27` (`ReferenceScene`, "each with a camera path in frames and a frame delta"); `src/time/mod.rs:10-12` (delta time injected; "a capture fixes it"); `tests/goldens.rs`, `bench/baselines/*` |
| Golden | matches | `src/capture/mod.rs:33` (`GOLDEN_TOLERANCE: u8 = 1`); `src/capture/mod.rs:44-58`; `tests/goldens/<backend>/*.png` |
| Render switches | matches | `src/render/renderer.rs:72-97` (`RenderSwitches { culling, froxel_lights, shadow_rays, count_point_lights, crowd_form }`; "The defaults are what ships"); `src/render/renderer.rs:139-149` (`Default`) |
| Frame stats | matches | `src/stats/mod.rs:8-65` (`FrameStats`) and `:67-105` (`GpuFrameStats`); `src/render/renderer.rs:35-48` (`TIMED_PASSES` holds `DYNAMIC_TLAS_BUILD` and `BLAS_REFIT_PASS`); `src/render/readback.rs:568-575` (the frame total's build plus each submission's span, the gate) |
| Baseline | matches | `src/bench/mod.rs:28-35` (`BASELINE_DIRECTORY`, "the reference machine's, CI's lavapipe for counters only, and the comparison adapter's"); `src/bench/mod.rs:391-398` (`CompareMode::Ci`); `bench/baselines/{apple-m4-pro,llvmpipe-llvm-20-1-2-256-bits,nvidia-geforce-rtx-2070-super,nvidia-geforce-rtx-2070-super-dx12}` |
| Bench record | matches | `src/bench/provenance.rs:32-55` (`Provenance`: checkout, `driver`, `driver_info`, `harness_version`, `quiet`); `src/bench/history.rs:55-57` (the release build's record: binary size and wall time) |
| Bench history | matches | `src/bench/history.rs:1-3` and `:24-29` (`records/`, `tagged/`, `experiments/` on `bench-history`) |
| Series | drift (L7-G01) | `src/bench/history.rs:58-61` (`series/<adapter>-<backend>/<scene>.json`: one row per record); `src/bench/history.rs:420` ("One Series file's row: a record's source and its metrics"); `src/bench/history/verdicts.rs:710` ("Series: `{scene}` on `{adapter_key}`") |
| Harness version | matches | `src/bench/provenance.rs:17-21` (`HARNESS_VERSION: u32 = 10`, "raised by any change to how a metric is measured, so that two Bench records compare only when they share it") |
| Experiment | matches | `src/bench/history.rs:28-29` and `:62-65` (`experiments/<name>/...`, "on no Series", compared with its merge base's record); `src/bench/history.rs:137` |
| Suspect | drift (L7-G02) | `src/bench/history/suspects.rs:6-19` (`WINDOW_RECORDS = 8`, `MEDIAN_ABSOLUTE_DEVIATIONS = 4.0`, `SMALLEST_SHIFT_FRACTION = 0.03`); `src/bench/history/suspects.rs:40-87`; `src/bench/history.rs:356-364` (`starts_line` includes `line_starts`, the pinned sources); `src/bench/history.rs:1025-1027` |
| Verdict | drift (L7-G08) | `src/bench/history/verdicts.rs:79-101` (`Verdict`: `state: Held | Cleared`, `good`, `bad`, `pinned`, `issue`); `src/bench/history/verdicts.rs:791-797` (held at half the shift and `SMALLEST_HELD_SHIFT`); `src/bench/history/verdicts.rs:598-605` (`--first-parent`); `src/bench/mod.rs:400-403` and `src/preflight/mod.rs:59-67` (other `pub enum Verdict`s) |
| Regression | matches | `src/bench/history/verdicts.rs:444-451` (`file_regressions`, one `needs-triage` issue per scene); `src/bench/history/verdicts.rs:49-51` (`covered_by_baseline`); `src/bench/history/verdicts.rs:97-100` (`issue` on the Verdict); `src/bench/history/verdicts.rs:761` |
| Budget | drift (L7-G03, L7-G04) | `bench/budgets.json:1-8` (six keys, one `gpu_memory_bytes`, no Input latency); `src/bench/mod.rs:372-389` (`Budgets`); `src/assets/pools.rs:7-9` ("until ADR 0013's split of the 8 GB Budget replaces it"); `src/bench/mod.rs:129-131` (counters held to Baselines) |
| Capability floor | matches | `src/gpu/mod.rs:259-301` (`floor_features`, `floor_downlevel_flags`, `floor_limits`); `src/gpu/mod.rs:361-375` (`BelowCapabilityFloor`, every gap named); `src/gpu/mod.rs:410` |
| Consolidation milestone | matches | `docs/agents/workflow.md:12` ("A Consolidation milestone (`M3.5` first) implements no ADR ... closes by its exit criteria like any other"); `docs/agents/audit.md:3` |
| Audit lane | matches | `docs/agents/audit.md:3` (brief, readers, verifier, report, triage); `docs/agents/audit.md:21-22` (`docs/audit/<milestone>/<lane>.md`) |
| Finding | drift (L7-G09) | `docs/agents/audit.md:27-38` (the eight fields); `docs/agents/audit.md:21` (killed Findings stay on record); `src/bench/mod.rs:405-411` (`pub struct Finding`, another meaning) |
| Severity class | matches | `docs/agents/audit.md:40-50` (the five classes in fixed rank, no numeric score) |
| Deviation | matches | `docs/agents/audit.md:52-54`; `docs/audit/deviations.md` (exists, no rows); `scripts/cycles-reference/DIFFERENCES.md` |


## The rules

Read in full at `8bc34eb18fcfb79307193551e96619316aef566c` in `/Users/guillem.arias/Documents/gariasf/praxis-512`: `CLAUDE.md` (419 lines) and the six files of `docs/agents/` (`workflow.md`, `sessions.md`, `triage-labels.md`, `issue-tracker.md`, `domain.md`, `audit.md`). Every path, file, module, command, flag, number, label, branch and test the text names was checked against the tree: `Cargo.toml` and `Cargo.lock` for the Stack; `find` over `src/`, `tests/`, `scripts/`, `bench/`, `docs/` and `.github/` for the Project shape and the Documents; `tests/conventions.rs`, `tests/goldens.rs`, `tests/asset_manifest.rs`, `tests/dependency_licences.rs`, `tests/references.rs`, `tests/field_guides.rs`, `src/render/shader_tests.rs` and `src/render/shader_function_tests.rs` for every "tested" clause, each traced to a test function by name; `scripts/verify.sh`, `scripts/pr-lint.sh`, `scripts/pr-lint-test.sh`, `scripts/gallery.sh`, `scripts/sweep.sh`, `.github/workflows/ci.yml`, `.github/workflows/pr-lint.yml`, `.github/pull_request_template.md`, `src/bin/praxis.rs` (its usage string at lines 313-321 and its parser), `src/preflight/mod.rs` and `src/bench/` for `workflow.md`; `~/.claude/skills/` for every skill `sessions.md`, `audit.md` and `domain.md` name; `grep` over `src/` for each Review checklist rule and the import conventions, with the bevy_ecs 0.18.1 prelude read under `~/.cargo/registry` to decide what it exports. Clauses about process, people, chat or GitHub state (labels, issue numbers, decision comments, merge habits) are marked `not checkable from the tree`. Nothing in the worktree was changed and nothing that writes to it was run.

### CLAUDE.md

| Section | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| Preamble | "agents implement, commit, open PRs and merge them once reviewed, without asking ... (`docs/agents/workflow.md`)" | not checkable from the tree (process); the file exists | `docs/agents/workflow.md:5` |
| Project posture | "Load-bearing decisions (`docs/adr/`)" | matches: 24 ADRs, 0001 to 0024 | `docs/adr/0001-material-as-id-not-bind-group.md`, `docs/adr/0024-bench-history-is-git-stored-bench-records-keyed-by-engine-source.md` |
| Project posture | "Work traces to a ticket in the live milestone (`docs/agents/workflow.md`)" | not checkable from the tree (GitHub) | `docs/agents/workflow.md:9` |
| Project posture | "the standing request is a field guide for each session's important new concepts (`docs/agents/workflow.md`)" | not checkable from the tree (process); see Notes: workflow.md now phrases the duty as a page per mechanism, written or rewritten by the session that adds or changes one | `docs/agents/workflow.md:31` |
| Documents | "`docs/agents/workflow.md` — how work moves: ladder, tickets, PRs, budgets, session start" | matches: headings The ladder, Tickets, PRs, Review, Performance and memory, Correctness, Session start, Context hygiene | `docs/agents/workflow.md:7,16,26,37,52,60,66,74` |
| Documents | "`docs/agents/sessions.md` — which skill starts which kind of session" | matches | `docs/agents/sessions.md:1-16` |
| Documents | "`CONTEXT.md` — glossary" | matches: exists at the root | `CONTEXT.md` (16499 bytes) |
| Documents | "`docs/adr/` — decisions that are hard to reverse" | matches | `docs/adr/` (24 files) |
| Documents | "`docs/references.toml` — ... each with its section and a verbatim quote; code, ADRs and notes cite its keys (the `ref:` form with its key)" | matches: fields `url`, `section`, `quote`, `checked`; 581 entries; 147 citations under `src/` | `docs/references.toml:4-14`; `grep -rn "ref:" src` (147) |
| Documents | "`docs/guides/` — the field guides ... a page per mechanism, and until each page exists the volumes ... which GitHub Pages serves from `main`" | matches as far as the tree shows: 29 HTML pages plus `img/`; every page is still a volume (an "In brief" and "Reading list" shape, a "Describes ... (PR #n)" line); Pages hosting is not checkable from the tree | `docs/guides/index.html`, `docs/guides/depth-pyramid.html` (h2s "In brief" ... "Reading list", "Glossary") |
| Documents | "`docs/bench/` — the page that draws the Bench history's Series from `bench-history` in the browser (ADR 0024)" | matches | `docs/bench/index.html:166-167`; `docs/adr/0024-...md` |
| Documents | "`docs/audit/` — ... `deviations.md` ...; `<milestone>/<lane>.md`, each Audit lane's report" | matches: `deviations.md` exists with an empty table; no lane report yet, as none has landed | `docs/audit/deviations.md:21` (table header, no rows) |
| Documents | "`ROADMAP.md` — long-range vision: target shape, design principles, scripting plan" | matches: headings Design Principles (1-4), Scripting plan | `ROADMAP.md:32,36-51,59` |
| Agent skills, Issue tracker | "GitHub Issues ... via the `gh` CLI. See `docs/agents/issue-tracker.md`" | matches | `docs/agents/issue-tracker.md:1-3` |
| Agent skills, Triage labels | "five-role vocabulary (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`) ... `fog` marks a ticket placed at a later milestone's charting" | matches the table in `triage-labels.md`; the labels themselves are GitHub state | `docs/agents/triage-labels.md:5-16` |
| Agent skills, Audit | "shared brief, parallel read-only readers, adversarial verifier, report, triage. See `docs/agents/audit.md`" | matches | `docs/agents/audit.md:17-23` |
| Agent skills, Domain docs | "Single-context: one `CONTEXT.md` plus `docs/adr/` at the repo root. See `docs/agents/domain.md`" | matches: no `CONTEXT-MAP.md` at the root | `ls` of the root; `docs/agents/domain.md:15-24` |
| Stack | "Rust (edition 2024)" | matches | `Cargo.toml:4` |
| Stack | "wgpu 30, WGSL shaders" | matches: `wgpu = { version = "30" ... features = [..., "wgsl"] }`, lock 30.0.1 | `Cargo.toml:8`; `Cargo.lock` (`wgpu` 30.0.1) |
| Stack | "winit 0.30" | matches: lock 0.30.13 | `Cargo.toml:9` |
| Stack | "glam" | matches: 0.32 (lock 0.32.1; a 0.30.10 also in the lock is a transitive copy) | `Cargo.toml:10` |
| Stack | "`bevy_ecs` (sub-crate only — no full Bevy stack), with `bevy_tasks` for its compute pool" | matches: only `bevy_ecs`, `bevy_tasks`, `bevy_mikktspace` of the bevy crates; the compute pool is `bevy_tasks::ComputeTaskPool` | `Cargo.toml:17-18,20`; `src/threading.rs:7` |
| Stack | "glTF via `gltf`, images via `image`, tangents via `bevy_mikktspace`, BC encoding via `intel_tex_2`, all four behind the `cook` feature" | matches: all four `optional = true`; `cook = ["dep:bevy_mikktspace", "dep:gltf", "dep:image", "dep:intel_tex_2"]`; CI checks none links without `cook` | `Cargo.toml:15,16,20,24,41`; `.github/workflows/ci.yml:118-123` |
| Stack | "Blobs via `ktx2`, `zstd`, `meshopt` and `sha2`, Arenas via `offset-allocator` (ADRs 0017, 0018)" | matches: all non-optional; used in `src/cook/`, `src/assets/cooked.rs`, `src/assets/pools.rs` | `Cargo.toml:23,25-28`; `src/assets/pools.rs` (`offset_allocator`) |
| Stack | "`tracing` + `tracing-subscriber` (with `env-filter`)" | matches | `Cargo.toml:11,19` |
| Stack | "Single crate: a library plus a thin binary" | matches | `src/lib.rs`, `src/bin/praxis.rs` |
| Stack | "Pinned in `Cargo.toml`" | matches by convention (caret requirements in `Cargo.toml`, exact versions in `Cargo.lock`) | `Cargo.toml:7-28`; `Cargo.lock` |
| Project shape | "lib.rs, state.rs # library root (window shell) + orchestrator" | matches: `lib.rs` declares the modules and `pub fn run`; `state.rs` builds the World and runs the frame | `src/lib.rs:18-36,269`; `src/state.rs` |
| Project shape | "bin/praxis.rs # thin binary: command dispatch" | matches: `Command` enum, `run_command`, `parse_command` | `src/bin/praxis.rs:34,95,154` |
| Project shape | "gpu/ # GPU context, windowed or headless" | matches | `src/gpu/mod.rs:22,31` |
| Project shape | "capture/ # offscreen render read back as an image, where its Goldens live, and how two images differ" | matches: `GOLDEN_DIRECTORY`, `GOLDEN_TOLERANCE`, the difference image | `src/capture/mod.rs:33,38,66`; `src/preflight/mod.rs:11` |
| Project shape | "preflight/ # `praxis preflight`: a session's starting state, a line per check" | matches | `src/preflight/mod.rs:1-3`; `src/bin/praxis.rs:232` |
| Project shape | "bench/ # Reference scene medians, compared with Baselines and Budgets, published as Bench records on `bench-history`; history/import.rs, `praxis history import` ...; history/suspects.rs, which Series records are Suspects" | matches: `history/import.rs`, `history/suspects.rs` exist; `history import` parses; `history.rs`, `history/verdicts.rs`, `provenance.rs`, `report.rs` are covered by the directory's description | `src/bench/history/import.rs`, `src/bench/history/suspects.rs`; `src/bin/praxis.rs:282-297` |
| Project shape | "stats/ # frame stats: CPU and GPU times, counts, GPU memory, counting allocator" | matches | `src/stats/mod.rs:10,69,438` |
| Project shape | "scenes/ # ... the Spin system (the orrery, town's flocks) and the crowd's Sway system; town.rs" | matches | `src/scenes/mod.rs:658,1042,1063`; `src/scenes/town.rs` |
| Project shape | "components.rs # ECS components (Transform, MeshRef)" | matches (also `MaterialRef`, `Bend`, `PointLight`, `DirectionalLight`; no `wgpu::` in the file) | `src/components.rs:11,44,47,60,81,97` |
| Project shape | "threading.rs # compute pool, schedule executor (ADR 0016)" | matches | `src/threading.rs:1-7` |
| Project shape | "camera/, time/, input/, helmet/, transform/, environment/ # ... per domain in one mod.rs" | matches: each directory holds only `mod.rs` | `find src -type f` |
| Project shape | "assets/ # asset types, pools, the Cooked cache reader (Asset IDs, Blobs), and the loader" | matches: `cooked.rs`, `loader.rs`, `material.rs`, `mesh.rs`, `pools.rs` | `src/assets/loader.rs:4,11-28` |
| Project shape | "cook/ # ... `praxis cook` turns manifest sources and Reference scenes into Blobs in git-ignored `cooked/`" | matches | `.gitignore` (`/cooked/`); `src/bin/praxis.rs:234` |
| Project shape | "`praxis fetch` downloads the manifest's fetched sources, as a dev launch that lacks one does" | matches: `fetch_for_launch` runs from the cook path a launch takes | `src/cook/mod.rs:18,130`; `src/bin/praxis.rs:236` |
| Project shape | "`praxis export-scene` writes what prepare reads of a Reference scene at a frame" | matches | `src/bin/praxis.rs:254-258,315`; `src/cook/export.rs` |
| Project shape | "render/ # Renderer (owns the Pools, reads no ECS World, calls the passes in order)" | matches: `renderer.rs` | `src/render/mod.rs:13,35` |
| Project shape | "bend_pass ... blas_refit: their BLASes refitted through wgpu-hal on Metal, or rebuilt, before the dynamic TLAS's build" | matches | `src/render/blas_refit.rs:4-6,29,64` |
| Project shape | "froxel_pass ... culling_pass: the compute pass that builds the frame's draw" | matches | `src/render/culling_pass.rs:3,102`; `src/render/froxel_pass.rs` |
| Project shape | "main_pass: the depth pre-pass, phase two's depth pass and the lit pass" | matches | `src/render/main_pass.rs:21,30,39` |
| Project shape | "shadow_mask_pass: ... directional lights' shadow edges again per sample" | matches | `src/render/shadow_mask_pass.rs:8-11,76-79` |
| Project shape | "depth_pyramid_pass ... tonemapping_pass ... light_count_pass: with the count_point_lights Render switch" | matches | `src/render/renderer.rs:93,145`; `src/render/bind_groups.rs:304` |
| Project shape | "targets (every size-dependent target, the depth pyramid, the HDR target and the shadow mask among them ...)" | matches | `src/render/targets.rs:58-80` (`hdr`, `depth_pyramid`, `depth_pyramid_levels`, `shadow_mask`, `shadow_mask_samples`) |
| Project shape | "prepare, bind_groups (each defined once), readback (each frame's GPU stats through a ring of mapped buffers), acceleration_structures (the static and dynamic TLAS, their memory estimates), instance, uniforms, staging, shaders/" | matches | `src/render/mod.rs:1-19`; `src/render/readback.rs:1,74`; `src/render/mod.rs:28-31` |
| Project shape | render/ listing against the files present | matches with a note: `coverage_tests.rs`, `shader_tests.rs`, `shader_function_tests.rs` (declared `#[cfg(test)]`) and `dither_mask.bin` (the tonemapping pass's `include_bytes!`) are not named; the two shader test files are named in the Review checklist | `src/render/mod.rs:21-26`; `src/render/tonemapping_pass.rs:25` |
| Project shape | "tests/ # integration tests at agreed seams" | matches: 15 test files | `tests/*.rs` |
| Project shape | "goldens/<backend>[-cpu]/ # committed Goldens; PRAXIS_UPDATE_GOLDENS=1 cargo test --test goldens rewrites them" | matches: `metal/`, `vulkan/`, `vulkan-cpu/`, nine scenes each | `tests/goldens.rs:73` (`UPDATE_GOLDENS_VARIABLE`), `tests/goldens.rs:34` (`SCENES: [ReferenceScene; 9]`) |
| Project shape | "README.md shows the metal ones, and cargo test checks it lists each scene" | drift (L7-R01): no `tests/goldens/README.md`; the README is the repository root's, which the test reads | `tests/goldens.rs:25-26,386-388`; `README.md:5,105-112` |
| Project shape | "bench/ # budgets.json (ADR 0013) and baselines/<adapter>/<scene>.json: the M4 Pro's, CI's lavapipe (`--ci-counters`) and the RTX 2070 SUPER's, a `-dx12` directory for its DX12 runs" | matches: `apple-m4-pro`, `llvmpipe-llvm-20-1-2-256-bits`, `nvidia-geforce-rtx-2070-super`, `nvidia-geforce-rtx-2070-super-dx12` | `bench/budgets.json`; `bench/baselines/*/`; `.github/workflows/ci.yml:178` |
| Project shape | "`praxis bench <dir>` then `praxis compare <dir>`; `praxis report <dir>` prints the tables PRs quote" | matches | `src/bin/praxis.rs:274,299,307,317-320`; `src/bench/report.rs:1,55,122` |
| Project shape | "scripts/pr-lint.sh # CI's PR form check: body, commits and a rendering PR's gallery images against docs/agents/workflow.md, tested by scripts/pr-lint-test.sh" | matches | `scripts/pr-lint.sh:2-7,62-85,87-106`; `scripts/pr-lint-test.sh:1-2`; `.github/workflows/pr-lint.yml:35-36,57` |
| Project shape | "scripts/gallery.sh # ... each Reference scene at 2560x1440, with difference images against an earlier one, and with PRAXIS_BLENDER each scene's Cycles render beside it" | matches | `scripts/gallery.sh:2-17,30` |
| Project shape | "scripts/cycles-reference/ # ... Blender 5.2's Python: calibrate.py ..., render.py ..., compare.py ...; DIFFERENCES.md" | matches: the four files exist (plus `display-ramp.txt` and `differences/*.png`, covered by the directory) | `scripts/cycles-reference/{calibrate.py,render.py,compare.py,DIFFERENCES.md}`; `scripts/gallery.sh:11` |
| Project shape | "scripts/sweep.sh # the nightly sweep: re-benches a slower Suspect ... bisects it (`praxis history bisect`), opens a needs-triage issue per pinned Regression (`praxis history regressions`), then benches main's scenes lacking a record ...; beside it its launchd (sweep.plist) and Task Scheduler (sweep-task.xml) definitions" | matches | `scripts/sweep.sh:2-11,155,193,197-199`; `scripts/sweep.plist`; `scripts/sweep-task.xml` |
| Project shape | "scripts/verify.sh # the verify command: fmt, clippy, build, tests with Goldens, bench counters against Baselines and Budgets" | matches | `scripts/verify.sh:72-78,61,52-53` |
| Project shape | "Rendering pulls from `World` (Design Principle #3 in `ROADMAP.md`) ... (ADR 0004). ECS types never hold GPU buffers (Design Principle #1)" | matches: Principle 3 "Renderer pulls, entities don't push", Principle 1 "Handle-based resources"; no `wgpu::` in `components.rs` or the feature modules | `ROADMAP.md:36,46`; `grep wgpu:: src/components.rs src/{camera,time,input,helmet,transform,environment}/mod.rs` (nothing) |
| Project shape | "When a feature folder grows past one file, split into `feature/{resource,system,...}.rs`. Don't pre-split" | matches: every feature folder is one `mod.rs` | `find src -type f` |
| How agents work here | "Implement tickets end to end ... A rendering change or a visual check shows before and after images in its PR (`docs/agents/workflow.md`, PRs)" | not checkable from the tree (process); the PR check exists | `scripts/pr-lint.sh:62-85` |
| How agents work here | "Work the live milestone's frontier" | not checkable from the tree | — |
| How agents work here | "`/implement` closes with a local `/code-review`" | matches: the skill's last step calls `code-review` | `~/.claude/skills/implement/SKILL.md:15` |
| How agents work here | "A bug-fix is not a refactor ..."; "The one licence to refactor is an audit's code-health fix ticket" | not checkable from the tree (process); `audit.md` Lane 5 carries the same licence | `docs/agents/audit.md:104` |
| How agents work here | "Don't add comments ... to code you didn't change"; "`cargo fmt` and `clippy` own style" | not checkable from the tree; fmt and clippy run in verify and CI | `scripts/verify.sh:72-73`; `.github/workflows/ci.yml:45-49` |
| How agents work here | "skills: `codebase-design`, `tdd`" | matches: both installed | `~/.claude/skills/codebase-design`, `~/.claude/skills/tdd` |
| Code conventions | "`cargo fmt` and `cargo clippy` must pass" | matches: both gate verify and CI with `-D warnings` | `scripts/verify.sh:72-73`; `.github/workflows/ci.yml:46,49,116` |
| Code conventions | "never `println!` or `log` (tested)" | matches: rule "print or `log` instead of `tracing`", function `printing`, in `the_source_tree_follows_the_conventions` | `tests/conventions.rs:17-20,44,148-157` |
| Code conventions | "Prefer structured fields" | not checkable from the tree (judgement) | — |
| Code conventions | "Default log filter is `praxis=debug,wgpu=warn,naga=warn` via `.cargo/config.toml`" | matches | `.cargo/config.toml:3` |
| Code conventions | "Bevy imports: `use bevy_ecs::prelude::*;` (not piecemeal)" | drift (L7-R02): three shipping files import items the prelude lacks by path | `src/state.rs:5`; `src/threading.rs:6`; `src/time/mod.rs:4` |
| Code conventions | "WGPU types: full path (`wgpu::Buffer`), don't import individual types" | drift (L7-R03): `blas_refit.rs` imports `wgpu::hal::api::Metal` | `src/render/blas_refit.rs:174,287` |
| Code conventions | "Identifiers are full words, WGSL included (... shading names tested)" | matches: rule "abbreviated shading name", function `abbreviated_shading_names` over `.rs` and `.wgsl` | `tests/conventions.rs:25-28,81,192-214` |
| Code conventions | "Pool lookups return `Option<&T>` ... never `.unwrap()` in a hot loop" | matches: `render_unwraps` forbids `.unwrap()` in shipping `src/render/` | `tests/conventions.rs:37-40,276-286` |
| Code conventions | "Shaders are WGSL parts in `src/render/shaders/`, assembled into modules by its `mod.rs` after the bindings and constants it declares from Rust; a function under an oracle lives in a shared part" | matches | `src/render/shaders/mod.rs:3-5,231,278-281`; `src/render/shader_tests.rs:124` (`every_wgsl_file_is_a_part_of_some_module`) |
| Code conventions | "No `unsafe` unless absolutely necessary and clearly explained" | matches: `gpu/mod.rs` has a SAFETY comment; `blas_refit.rs` has a SAFETY comment on its wgpu-hal path; `stats` implements `GlobalAlloc` | `src/gpu/mod.rs:71-77`; `src/render/blas_refit.rs:198`; `src/stats/mod.rs:438` |
| Code conventions | "Keep dependencies minimal. Use feature flags (`image = { version = "0.25", default-features = false, features = ["png"] }`)" | matches (the dependency also carries `optional = true`) | `Cargo.toml:15,32` |
| Code conventions | "Tests use the built-in harness (`cargo test`) ... Test-framework crates are an owner decision" | matches: dev-dependencies are `image`, `texture2ddecoder`, `wgpu-naga-bridge` only | `Cargo.toml:30-35` |
| Code conventions | "Every file under `assets/` has an entry in `assets/manifest.json` and its credit in `assets/ATTRIBUTION.md`; `cargo test` checks both. Policy: `docs/research/test-assets.md`" | matches: `every_asset_is_listed_licensed_and_credited`; the policy file exists | `tests/asset_manifest.rs:9-10,37`; `docs/research/test-assets.md` |
| Code conventions | "Every dependency's licence expression is on the allowlist in `tests/dependency_licences.rs`; `cargo test` checks it" | matches: `ALLOWED`, `every_dependency_has_an_allowed_licence` | `tests/dependency_licences.rs:8,36` |
| Code conventions | "Docs, tickets and code comments name no patents" | not checkable mechanically; `workflow.md` carries the same rule | `docs/agents/workflow.md:24` |
| Code conventions | "Comments ... citing no ticket, PR or decision: only an upstream bug, adapted code's source and licence, a pinned third-party source or an ADR (ticket numbers tested)" | matches: rule "ticket number in a comment", function `ticket_numbers`, upstream `owner/repo#n` allowed | `tests/conventions.rs:21-24,168-188` |
| Review checklist | "A size-dependent target lives in the targets module (`src/render/targets.rs`), whose `RenderTargets::new` creates every target and runs again on resize" | matches | `src/render/targets.rs:58,98` |
| Review checklist | "A binding is written once, in its bind group's definition (`src/render/bind_groups.rs`) ... `cargo test` checks each binding's stages against the entry points that use it, and the structs the CPU writes field by field against their Rust twins (`src/render/shader_tests.rs`); Rust-only padding fields start with `_`" | matches: `every_binding_is_visible_to_exactly_the_stages_that_use_it`, `structs_shared_with_the_cpu_match_their_rust_twins`; underscored fields skipped as padding; the tree's padding fields are `_pad*`, `_padding` | `src/render/shader_tests.rs:35,232,708-709`; `src/render/uniforms.rs:14,37,52,63`; `src/render/culling_pass.rs:90` |
| Review checklist | "No World position reaches the GPU in f32: only an anchor plus an f32 offset, composed camera-relative (ADR 0014)" | matches: `anchor_and_offset` for the camera and each Instance | `src/render/prepare.rs:10,291,385`; `src/render/instance.rs` |
| Review checklist | "Depth is cleared to 0 and compared with `Greater` ... (ADR 0015; no `Less` and no nonzero depth clear tested)" | matches: rule "depth not cleared to 0 or compared with `Less`", function `reversed_depth`; no `CompareFunction::Less` under `src/` | `tests/conventions.rs:33-36,249-266`; `grep CompareFunction::Less src` (nothing) |
| Review checklist | "Light values are photometric as glTF defines them (lux, candela, cd/m²) and exposure is an EV100; HDR colour is pre-exposed before fp16 stores it and clamped to 65504 (ADR 0023)" | matches | `src/render/uniforms.rs:34`; `src/scenes/mod.rs:351-354`; `src/camera/mod.rs:7-11`; `src/render/shaders/hdr.wgsl:7`; `src/render/targets.rs:339` |
| Review checklist | "Matrices reach WGSL column-major (glam `to_cols_array_2d`); a `mat3x3` or `array<vec3>` field is padded per column or element on the Rust side (the test compares sizes, not strides)" | matches: 11 `to_cols_array_2d` uses; `padded_columns`; the layout test compares member offsets and sizes | `src/render/prepare.rs:10`; `src/render/shader_tests.rs:700-711` |
| Review checklist | "A new or changed shading function in WGSL has an oracle test in `src/render/shader_function_tests.rs`" | matches: the file exists with 32 oracle tests | `src/render/shader_function_tests.rs:20-1865` |
| Review checklist | "Texture formats match between creation and surface config, and sRGB is encoded once: the tonemapping pass alone encodes it ...; no target is an sRGB format (this last clause tested)" | matches: rule "sRGB texture format outside sampled textures and the surface", function `srgb_formats`; the surface test | `tests/conventions.rs:29-32,226-235`; `src/gpu/mod.rs:657-662` (`the_surface_format_is_an_8_bit_unorm_one`) |
| Review checklist | "`unwrap()` on GPU operations that could fail at runtime ... (tested in `src/render/`)" | matches: `render_unwraps`, shipping code of `src/render/` only | `tests/conventions.rs:37-40,276-286` |
| Review checklist | "No GPU resources stored on ECS components" | matches: see Project shape row on `components.rs` | `src/components.rs` |
| Review checklist | "Code adapted from elsewhere cites its source and licence" | not checkable mechanically from the tree | — |
| Review checklist | "Every reference the change relies on or adds exists ... (`docs/agents/workflow.md`, Review: Sources)" | matches: the heading and the registry test exist | `docs/agents/workflow.md:42`; `tests/references.rs:11` |
| Review checklist | "Frame data holds no borrow into the ECS World, and only the main thread submits, presents or polls (ADR 0016)" | matches by design: `FrameData` is owned, built by `FramePreparer` | `src/render/mod.rs:33`; `src/render/prepare.rs` |
| Review checklist | "Whatever changes what an Instance holds makes prepare write its slot; other slots keep what earlier frames wrote (ADR 0003)" | matches | `src/render/prepare.rs:33-35,91-94` |
| Review checklist | "The GPU builds the frame's draw in slot order within each culling phase, never in the order atomics land" | matches | `src/render/culling_pass.rs:3,102`; `src/render/shaders/culling.wgsl:2,14` |
| Review checklist | "Frame stats still see new work: GPU buffers and textures join their owner's `gpu_memory_bytes`, and so do acceleration structures ... (`AccelerationStructureBytes`); ... per-frame GPU writes add to `uploaded_bytes`, and systems are added inside `SystemTimer::scope`" | matches: all three names exist | `src/stats/mod.rs:22,45-46,316,334`; `src/render/mod.rs:28-29` |
| Review checklist | "Streaming uploads go through the `StagingBelt`, never `Queue::write_texture`; no GPU buffer grows at runtime, and a request past a buffer's capacity is refused and logged. Only per-frame staging grows" | matches: `wgpu::util::StagingBelt` in `staging.rs`; no `queue.write_texture` under `src/`; the one `queue.write_buffer` is a test's; `renderables_refused` counted | `src/render/staging.rs:13,26,173,198`; `src/render/depth_pyramid_pass.rs:245-253` (test); `src/stats/mod.rs:157` |
| Review checklist | "The runtime loads cooked Blobs only and references Assets by Asset ID, never by source path (ADR 0018)" | matches: `AssetLoader` takes `AssetId`; CI's shipping job removes `assets/` | `src/assets/loader.rs:11-28`; `.github/workflows/ci.yml:93-134` |
| Review checklist | "The main pass reads no screen-space history, and no change rules out its 4x MSAA (ADRs 0008, 0019, 0020)" | matches: `SAMPLE_COUNT = 4` | `src/render/targets.rs:16` |
| Review checklist | "A newly used wgpu feature or limit joins the Capability floor, or, outside the capability matrix's Tiers 1-2, has a path for adapters without it (ADR 0021)" | matches: `floor_features`, `floor_limits`, `floor_downlevel_flags`; the matrix exists | `src/gpu/mod.rs:261,276,297`; `docs/research/wgpu-capability-matrix.md` |
| Review checklist | "Game-state changes run in the fixed schedule, which reads no wall clock and has no ambiguous system order (ADR 0022); ... `PreviousWorldTransform`, ... `Teleported` before `reset_teleported` runs" | matches | `src/time/mod.rs:12,37-41,69`; `src/transform/mod.rs:41,48,71` |
| Review checklist | "WGSL keeps sRGB and linear apart, has no divide-by-zero or NaN ..., and calls no `textureSample` under non-uniform control flow" | not checkable mechanically from the tree (judgement; five `textureSample` sites) | `grep textureSample src/render/shaders/*.wgsl` (5) |
| Review checklist | "Workflows keep least-privilege permissions and pinned action versions" | drift (L7-R04): permissions are least-privilege in both workflows; `ci.yml` uses floating refs (`@v4`, `@v2`, `@stable`) while `pr-lint.yml` pins a SHA | `.github/workflows/ci.yml:20-21,37,39,43,220-221`; `.github/workflows/pr-lint.yml:14-16,29` |
| Review checklist | "`cargo fmt` and `cargo clippy` pass" | matches (see Code conventions) | `scripts/verify.sh:72-73` |
| Maintenance | "Update this file in the same PR when: Folder layout changes ..." | not checkable from the tree (process) | — |
| Agent fleet | "No `AGENTS.md`, `.cursor/rules/`, `.junie/guidelines.md`, or similar" | matches: none at the root | `ls -la` of the root |

### docs/agents

| File, heading | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| workflow.md, intro | "A decision that lives only in chat is not made: it lands as an ADR (`docs/adr/`), a comment on the map issue, or a ticket comment" | not checkable from the tree (GitHub); `docs/adr/` exists | `docs/adr/` |
| workflow.md, The ladder | "Work is ordered as GitHub milestones (`M0`, `M1`, …)"; "`track:<name>`, today `track:observability`, [...]"; "label `wayfinder:map`" | not checkable from the tree (GitHub); preflight finds the live milestone as "the lowest-numbered open one" | `src/preflight/mod.rs:612,766` |
| workflow.md, The ladder | "A **Consolidation milestone** (`M3.5` first) ... auditing everything on `main` along the Audit lanes of `docs/agents/audit.md`" | matches | `docs/agents/audit.md:3,68-125` |
| workflow.md, The ladder | "the verify command (`scripts/verify.sh`) exits 0 within 30 lines" | matches: one line per step (fmt, clippy, build, test, bench, an optional aliasing note, verify), seven at most on success | `scripts/verify.sh:14-26,72-78` |
| workflow.md, The ladder | "with its Goldens committed on Metal and software Vulkan" | matches: `tests/goldens/metal/`, `tests/goldens/vulkan-cpu/` (and `vulkan/`) | `tests/goldens/` |
| workflow.md, The ladder | "a profile of the Reference scenes is recorded in `docs/performance.md`" | matches: the file exists with per-exit sections | `docs/performance.md:1-5` |
| workflow.md, The ladder | "the visual audit (see Correctness) judges every Reference scene ... [...]'s rubric ... The audit skill, `docs/agents/audit.md`, is the standing form of the visual audit and the source check: its image and drift lanes" | matches: lanes 2 and 7 | `docs/agents/audit.md:3,78-83,114-119` |
| workflow.md, The ladder | "Reordering is a decision comment on the map issue ... The rail numbers in this file (size ceiling, review rounds)" | not checkable from the tree | — |
| workflow.md, Tickets | "Size ceiling: about 400 changed lines, not counting goldens, lockfile, and generated or cooked data" | not checkable from the tree (no test or CI step counts it; the text does not claim one) | — |
| workflow.md, Tickets | "Every ticket carries a verification seam agreed before coding (`/tdd`)" | matches: the skill is installed | `~/.claude/skills/tdd` |
| workflow.md, Tickets | "Every ticket has a **Start here** section ... the form is in `docs/agents/issue-tracker.md`" | matches | `docs/agents/issue-tracker.md:29-32,47` |
| workflow.md, Tickets | "blocked by a `wayfinder:research` ticket titled "State of the art: …" ... fit (`docs/research/wgpu-capability-matrix.md`, ADR 0013's budgets on the M4 Pro, ADR 0008's stance on temporal accumulation) ... stays compact in `docs/research/`" | matches as far as the tree shows: the matrix, ADR 0013, ADR 0008 and `docs/research/` (13 notes) exist; the label is GitHub state | `docs/research/wgpu-capability-matrix.md`; `docs/adr/0013-...md`; `docs/adr/0008-...md` |
| workflow.md, PRs | "One ticket per PR (`Closes #N`)" | matches: pr-lint requires `Closes #N`, `Part k of #N` or `Ticket: n/a (reason)` | `scripts/pr-lint.sh:29-31` |
| workflow.md, PRs | "The verify command, `scripts/verify.sh`, exits 0 before the PR opens: formatting, lint, build, tests (Goldens required) and the bench counters against Baselines and Budgets, one line per step, about a minute once built" | matches (`PRAXIS_REQUIRE_GOLDENS=1`, `--counters`, `compare --ci`); the minute is not checkable | `scripts/verify.sh:52-53,61,72-78` |
| workflow.md, PRs | "The field guides, `docs/guides/`, ... GitHub Pages serves from `main` at https://gariasf.github.io/praxis/guides/" | not checkable from the tree (hosting); the directory exists | `docs/guides/` |
| workflow.md, PRs | "a page per mechanism ... A page opens with one sentence of definition, a *Describes* line ..., an *Interface* box ...; then *How it is tested*, *What it costs*, *Why this way*, *Limits*, *Sources* ... and a one-paragraph *History* note" | matches as a target shape: no page is in it yet (none has an *Interface*, *Why this way*, *Limits*, *Sources* or *History* section); the text's next clause keeps the volumes until each page exists | `grep -l "Why this way" docs/guides/*.html` (nothing); `docs/guides/depth-pyramid.html` h2 list |
| workflow.md, PRs | "a guide adds no entry to `docs/references.toml`" | matches: the registry test reads `.rs`, `.wgsl`, `.md`, `.sh`, `.toml`, not `.html` | `tests/references.rs:155-158` |
| workflow.md, PRs | "The hub, `docs/guides/index.html`, is the frame overview in data-flow order" | matches: sections "Suggested reading order" and "The frame on main" | `docs/guides/index.html` (h2 `order`, `frame`) |
| workflow.md, PRs | "`cargo test` checks every link and image between the pages" | matches: `every_link_between_the_field_guides_lands` | `tests/field_guides.rs:109` |
| workflow.md, PRs | "the rebuild adds the checks that each page's commit is an ancestor of `main`, that every identifier a page names in code font exists on `main`, and that no page's prose names a ticket or PR" | matches as stated (future tense): none of the three checks exists yet | `tests/field_guides.rs` (only the link check) |
| workflow.md, PRs | "Until a mechanism's page exists, the volume that narrates it stays, the hub lists it, and a change that makes its text untrue says so in its Status line" | matches: 28 of 29 pages carry a `Status:` box; the hub links every page (tested) | `docs/guides/*.html`; `tests/field_guides.rs:57-65` |
| workflow.md, PRs | "Description follows `.github/pull_request_template.md`: what changed, how it was verified (command and output, 30 lines at most), metric deltas, and under Decide only the owner items" | matches: sections What, Verification, Metrics, Images, Decide | `.github/pull_request_template.md:5,9,15,19,23` |
| workflow.md, PRs | "CI's PR form check (`scripts/pr-lint.sh`) fails a description or commit that breaks this section's rules or Review's counts and "Sources checked" line" | matches | `scripts/pr-lint.sh:33-60`; `.github/workflows/pr-lint.yml:41-58` |
| workflow.md, PRs | "`praxis capture <scene> <png> [<width>x<height>] [--frame <n>]` renders a scene headless, at its Golden's frame or the frame of its camera path it names" | matches the usage string | `src/bin/praxis.rs:238-252,314`; `tests/capture.rs:68` (`capture_command_renders_the_frame_it_names`) |
| workflow.md, PRs | "The images go on the `pr-images` branch, one directory per PR, never merged" | matches: the branch exists locally and on origin; its contents are not in the tree | `git branch -a` (`pr-images`, `remotes/origin/pr-images`) |
| workflow.md, PRs | "A PR that changes `src/render/` (shaders included), the cooker's textures or meshes (`src/cook/texture.rs`, `src/cook/mesh.rs`) or ADR 0019 or 0020 carries `scripts/gallery.sh`'s images ... run on the base (copied out of `target/tmp/gallery/`)" | matches: the path regex and the copy-out guard | `scripts/pr-lint.sh:67`; `scripts/gallery.sh:28,35-36` |
| workflow.md, PRs | "CI's PR form check fails a rendering PR whose Images section lacks any of the three (`before`, `after`, `difference`)" | matches | `scripts/pr-lint.sh:68-85` |
| workflow.md, PRs | "A PR that changes shading, lighting or shadows (`shading.wgsl`, the lighting in `main.wgsl`, `shadow_mask.wgsl`, `froxel.wgsl` ...) also carries the Cycles images ... with `PRAXIS_BLENDER` naming Blender 5.2 (`scripts/cycles-reference/`) ... `scripts/cycles-reference/DIFFERENCES.md`" | matches: the four parts and the files exist; gallery.sh takes `PRAXIS_BLENDER` and names Blender 5.2 | `src/render/shaders/{shading,main,shadow_mask,froxel}.wgsl`; `scripts/gallery.sh:11-17,54-64`; `scripts/cycles-reference/DIFFERENCES.md` |
| workflow.md, PRs | "Commit subject 72 characters at most ... No commit names an AI co-author or carries a "Generated with Claude" line" | matches: all three checked by pr-lint | `scripts/pr-lint.sh:94-105` |
| workflow.md, Review | "CI owns mechanical checks ... No external reviewer runs (CodeRabbit was removed, 2026-10-14)" | matches as far as the tree shows: no CodeRabbit configuration at the root; only `ci.yml` and `pr-lint.yml` run | `ls -a` (no `.coderabbit*`); `.github/workflows/` |
| workflow.md, Review | "run `/code-review` against the base branch: a Standards axis ... and a Spec axis ..., in parallel sub-agents, plus a third sub-agent beside them for a Sources axis" | not checkable from the tree: the installed skill runs the two axes; the third is the session's own (see Notes) | `~/.claude/skills/code-review/SKILL.md:3,8-9` |
| workflow.md, Review | "The Standards sub-agent runs at medium effort, the Spec and Sources sub-agents at high (`docs/agents/sessions.md`, Effort)" | matches | `docs/agents/sessions.md:25-26` |
| workflow.md, Review | "The registry, `docs/references.toml`, holds each external source's URL, section, verbatim quote and the date it was last checked" | matches: `url`, `section`, `quote`, `checked` | `docs/references.toml:7-12` |
| workflow.md, Review | "`cargo test --test references` fails on a dangling key or an unused entry" | matches: `every_citation_has_an_entry_and_every_entry_a_citation`, with `a_dangling_key_and_an_unused_entry_are_found` | `tests/references.rs:11,26` |
| workflow.md, Review | "Fill the PR template, with the review's counts (`N found, N fixed, N declined, N deferred`), the deferred list and a "Sources checked" line under Verification. Open the PR as a draft and run `gh pr ready`" | matches: counts and Sources line enforced; the `gh` steps are process | `scripts/pr-lint.sh:44-60`; `.github/pull_request_template.md:11-13` |
| workflow.md, Review | "Any agent squash-merges ... then removes the worktree and branch" | not checkable from the tree | — |
| workflow.md, Review | "A declined finding class that recurs becomes a line in the `CLAUDE.md` review checklist or a `CODING_STANDARDS.md` rule" | not checkable from the tree (process); no `CODING_STANDARDS.md` exists, which Correctness says may be so ("when it exists") | `docs/agents/workflow.md:63`; `ls` of the root |
| workflow.md, Performance and memory | "Budgets and Baselines live in the repo beside the benches (`bench/`, since M0)" | matches | `bench/budgets.json`; `bench/baselines/` |
| workflow.md, Performance and memory | "Figures ... are pasted from `praxis report <results directory>`, which prints them as markdown tables against their Baselines and the Budgets" | matches | `src/bench/report.rs:1-17,55,122`; `src/bin/praxis.rs:307` |
| workflow.md, Performance and memory | "`praxis bench` publishes its results as Bench records on the `bench-history` branch unless given `--no-publish`, and `praxis publish <results directory>` publishes them later. With `--experiment <name>`, either publishes ... as that Experiment's records, on no Series" | matches: flags parsed for `bench` and `publish` only; `bench-history` exists on origin | `src/bin/praxis.rs:178-199,277-280,316-319`; `git branch -a` (`remotes/origin/bench-history`) |
| workflow.md, Performance and memory | "A failed publish never fails the bench, and `verify` never publishes" | matches: verify passes `--no-publish` | `scripts/verify.sh:52` |
| workflow.md, Performance and memory | "CI's `publish-bench` job publishes lavapipe's records and the release build's size and wall time on every push to `main`; PR runs publish nothing" | matches: `publish-bench` runs on `push` only, with `--release-build-seconds`; the bench step passes `--no-publish`; `binary_bytes` recorded | `.github/workflows/ci.yml:178,214-238`; `src/bench/history.rs:293` |
| workflow.md, Performance and memory | "Baselines are still chosen by hand and still gate" | matches: `compare --ci` gates verify and CI | `scripts/verify.sh:53`; `.github/workflows/ci.yml:181` |
| workflow.md, Correctness | "Tests live at pre-agreed seams. Expected values come from an independent source" | not checkable from the tree (judgement); `shadows.rs` and the oracle tests are examples | `tests/shadows.rs:139`; `src/render/shader_function_tests.rs` |
| workflow.md, Correctness | "Mechanical rules belong in CI. `CODING_STANDARDS.md`, when it exists, carries judgement calls only" | matches: the file does not exist; conventions run under `cargo test` in CI | `tests/conventions.rs:44`; `.github/workflows/ci.yml:55` |
| workflow.md, Correctness | "`scripts/gallery.sh [<earlier gallery>]` captures every Reference scene at 2560×1440 into `target/tmp/gallery/` (`town` at the Goldens test's eight frames, `crowd` in both shadow forms), with difference images against an earlier gallery when given one" | matches: `size=2560x1440`, `town_frames` equal `TOWN_FRAMES` (59 300 500 700 900 1200 1500 1800), `crowd-proxies` and `crowd-refit` | `scripts/gallery.sh:19-33,111-120`; `tests/goldens.rs:294-299` |
| workflow.md, Correctness | "A milestone's visual audit runs it with `PRAXIS_BLENDER` set ... where `export-scene` exports the scene; a difference listed in `scripts/cycles-reference/DIFFERENCES.md` is known" | matches: crowd is skipped as export-scene refuses it | `scripts/gallery.sh:105-108` |
| workflow.md, Session start | "Run `cargo run --release -- preflight` ... ten lines or fewer: the adapter and which machine it is, whether its Goldens, Baselines, fetched files and Cooked cache are present, GPU load and other praxis processes, the open PRs and the live milestone's assigned tickets" | matches: at most ten checks (gpu load, processes, power, display, adapter, goldens, baselines, assets, prs, tickets) | `src/preflight/mod.rs:102-135,236-269,524-558` |
| workflow.md, Session start | "A `FAIL` line blocks `scripts/verify.sh` and names its fix; a `warn` line holds off a bench" | matches: preflight's own statement, and the sweep skips on `warn` | `src/preflight/mod.rs:1-3`; `scripts/sweep.sh:85-92` |
| workflow.md, Session start | "Orient: this file, `CLAUDE.md`, the map issue body (`gh issue list --label wayfinder:map`)" | not checkable from the tree (GitHub) | — |
| workflow.md, Session start | "`gh issue list --milestone <live> --label ready-for-agent ...`; `gh api repos/gariasf/praxis/issues/<n> --jq .issue_dependencies_summary.blocked_by`" | not checkable from the tree (GitHub); `issue-tracker.md` gives the same field | `docs/agents/issue-tracker.md:75-76` |
| workflow.md, Session start | "a new worktree has no fetched files: `cargo run --release -- fetch`), then `scripts/verify.sh` ... the failing step's log is in `target/verify/`" | matches | `src/bin/praxis.rs:236`; `scripts/verify.sh:9,22` |
| workflow.md, Context hygiene | "Validation commands print 30 lines at most on success" | matches for verify (7) and preflight (10); see Notes on gallery.sh | `scripts/verify.sh:72-78`; `src/preflight/mod.rs:102-135` |
| workflow.md, Context hygiene | "Research results are cached in `docs/research/`" | matches: 13 notes | `docs/research/` |
| sessions.md, intro | "type `/` and pick `mattpocock-skills:<name>`. The agent invokes `tdd`, `diagnosing-bugs`, `research`, `prototype`, `domain-modeling`, `codebase-design`, and `code-review` itself" | matches: all seven installed | `~/.claude/skills/{tdd,diagnosing-bugs,research,prototype,domain-modeling,codebase-design,code-review}` |
| sessions.md, table | "`/implement #<n>`", "`/wayfinder #<map>`", "`/grill-with-docs`", "`/to-spec`", "`/to-tickets`", "`/handoff`", "`/prototype`", "`/triage`", "`/improve-codebase-architecture`", "`/ask-matt`" | matches: all installed | `~/.claude/skills/{implement,wayfinder,grill-with-docs,to-spec,to-tickets,handoff,prototype,triage,improve-codebase-architecture,ask-matt}` |
| sessions.md, table | "`/audit <lane>`, or "Audit lane <n> per docs/agents/audit.md" ... report PR at `docs/audit/<milestone>/<lane>.md`" | matches with a note: no `audit` skill is installed; the row and `audit.md:3` both give the prompt form as the alternative | `ls ~/.claude/skills` (no `audit`); `docs/agents/audit.md:3,22` |
| sessions.md, table | "Research ticket ... cited note under `docs/research/`" | matches | `docs/research/` |
| sessions.md, Effort | "low: `cargo run --release -- preflight`, `cargo run --release -- fetch`, `scripts/verify.sh`, bench runs ..." | matches: the commands exist | `src/bin/praxis.rs:232,236`; `scripts/verify.sh` |
| sessions.md, Rules of thumb | "`/implement` closes with a local `/code-review` ... (workflow.md, Review)"; "(workflow.md, The ladder)"; "(workflow.md, PRs)" | matches: the skill step and the three headings | `~/.claude/skills/implement/SKILL.md:15`; `docs/agents/workflow.md:7,26,37` |
| triage-labels.md | the five roles and `fog`, each label string equal to its role | matches `CLAUDE.md`; the labels themselves are GitHub state | `CLAUDE.md:70` |
| triage-labels.md | "Charting a milestone starts from `gh issue list --label fog --state open`" | not checkable from the tree | — |
| issue-tracker.md, Conventions | the `gh issue` commands | not checkable from the tree (GitHub CLI usage) | — |
| issue-tracker.md, Ticket form | "A ticket body has these sections, in this order (`docs/agents/workflow.md`, Tickets): Parent, What to build, Start here, Acceptance criteria, Verification seam, Blocked by" | matches workflow.md's Tickets clauses (Start here, seam agreed with `/tdd`, acceptance criteria) | `docs/agents/workflow.md:20-23` |
| issue-tracker.md, Pull requests as a triage surface | "`/triage` reads this flag" | matches: the skill is installed | `~/.claude/skills/triage` |
| issue-tracker.md, Wayfinding operations | "Used by `/wayfinder`"; `wayfinder:map`, `wayfinder:<type>`; `issue_dependencies_summary.blocked_by` | matches for the skill; labels and the API are GitHub state | `~/.claude/skills/wayfinder` |
| domain.md | "`CONTEXT.md` at the repo root, or `CONTEXT-MAP.md` ... if it exists"; "`docs/adr/`" | matches: `CONTEXT.md` present, no `CONTEXT-MAP.md` | `ls` of the root |
| domain.md | "The `/domain-modeling` skill (reached via `/grill-with-docs` and `/improve-codebase-architecture`)" | matches: all three installed | `~/.claude/skills/{domain-modeling,grill-with-docs,improve-codebase-architecture}` |
| domain.md, File structure | "docs/adr/0001-material-as-id-not-bind-group.md" | matches | `docs/adr/0001-material-as-id-not-bind-group.md` |
| domain.md, Flag ADR conflicts | "Contradicts ADR 0007 (wgpu is the permanent graphics API)" | matches the ADR's title | `docs/adr/0007-wgpu-is-the-permanent-graphics-api.md` |
| audit.md, intro | "Run it for each lane of a Consolidation milestone (`M3.5` first ...) ... `/audit <lane>` where that skill is installed" | matches: hedged; the skill is not installed | `ls ~/.claude/skills` |
| audit.md, Inputs | "`docs/audit/deviations.md` ...; `scripts/cycles-reference/DIFFERENCES.md` ...; `docs/references.toml`" | matches: all three exist | the three paths |
| audit.md, The pipeline | "at high effort (`docs/agents/sessions.md`, Effort)"; "`docs/agents/workflow.md`, Performance and memory, says how a figure is quoted" | matches | `docs/agents/sessions.md:18-28`; `docs/agents/workflow.md:55` |
| audit.md, Report | "`docs/audit/<milestone>/<lane>.md` on `main`, by PR" | matches: the directory exists; no report yet | `docs/audit/` |
| audit.md, Deviations | "`docs/audit/deviations.md` holds, in one table ..." | matches the file's own description | `docs/audit/deviations.md:1-21` |
| audit.md, Lane 1 | "the milestone spec's stories (for `M3.5`, [...])"; "every function under an oracle ... the WGSL accuracy floor ... beside the test's constant" | not checkable from the tree (GitHub); the oracle tests exist | `src/render/shader_function_tests.rs` |
| audit.md, Lane 2 | "`scripts/gallery.sh` with `PRAXIS_BLENDER` at 2560×1440, and the 3x box-filtered supersampled reference" | matches: gallery size; the bench's aliasing pass renders at three times the Goldens' size | `scripts/gallery.sh:30`; `scripts/verify.sh:34-36` |
| audit.md, Lane 2 | "Each `DIFFERENCES.md` item is tagged in that file `deliberate until <trigger>` or `ticketed #n`" | matches as a method (what the lane does): no item is tagged yet | `grep -c "deliberate until\|ticketed" scripts/cycles-reference/DIFFERENCES.md` (0) |
| audit.md, Lane 3 | "the ladder's map issue ([...]; 27 of them at `M3.5`)"; "`pmset -g batt`" | not checkable from the tree (GitHub; a host command preflight also reads) | `src/preflight/mod.rs:360-378` |
| audit.md, Lane 5 | "clippy's `cognitive_complexity` lint run as a probe"; "Fix tickets carry the refactor licence (`CLAUDE.md`, How agents work here)" | matches the CLAUDE.md clause; the lint is clippy's, outside the tree | `CLAUDE.md:257-260` |
| audit.md, Lane 6 | "Golden tolerances kept at their values (1 of 255 and 0 pixels)" | matches: `GOLDEN_TOLERANCE: u8 = 1`, `HARDWARE_UNCULLED_PIXELS = 0` (software 1) | `src/capture/mod.rs:33`; `tests/goldens.rs:63,71` |
| audit.md, Lane 6 | "every WGSL shading function against an oracle in `src/render/shader_function_tests.rs`; every conventions test against the rule it enforces" | matches: the files exist | `src/render/shader_function_tests.rs`; `tests/conventions.rs` |
| audit.md, Lane 7 | "the ADRs, `CONTEXT.md`, `CLAUDE.md`, `docs/agents/*` and `docs/references.toml` ... ([...] ...)" | matches: all paths exist; the rubric is GitHub state (quoted in this brief) | the paths |
| audit.md, Lane 8 | "the page in the guide's shape (`docs/agents/workflow.md`, PRs)" | matches | `docs/agents/workflow.md:31` |


## Registry entries 1 to 47

47 entries, all 47 with check `full` (every one is cited from `docs/adr/`, `src/`, `tests/` or `scripts/`), 0 `sample`, 0 opened by a wrong sample. 46 confirmed against the source opened at the url (GitHub blob urls read as raw files at the pinned commit or tag; PDFs read at the cited page with pdftotext after downloading the url; HTML pages downloaded and read at the cited anchor or section), and every code site under `src/`, `tests/` or `scripts/` read at the frozen commit `8bc34eb1` in `/Users/guillem.arias/Documents/gariasf/praxis-512`. 1 entry `unopened`: `higham-3.1`, whose DOI resolves (302) to SIAM's book landing page, which answers 403 to the fetcher (Cloudflare challenge) and would in any case carry only the book's metadata, not pp. 62-63. One Finding, against a code comment that cites the sRGB EOTF⁻¹ entry for the EOTF's constants.

### Verdicts

| index | key | check | verdict | what was opened, and what it says |
|---|---|---|---|---|
| 1 | `higham-3.1` | full | unopened | `https://doi.org/10.1137/1.9780898718027` redirects to `epubs.siam.org/doi/book/10.1137/1.9780898718027`, which returns 403 (Cloudflare "Just a moment") to curl, WebFetch and the archived copy; the book's §3.1 is behind SIAM's paywall. Code site `src/render/shader_function_tests.rs:415`: the comment states γ₃ = 3u/(1 − 3u) and line 420 computes `3.0 * unit_roundoff / (1.0 - 3.0 * unit_roundoff)`, which is Lemma 3.1's γₙ = nu/(1 − nu) at n = 3 as the registry quotes it; consistent with the quote, unverified against the book. |
| 2 | `vulkan-spirv-precision` | full | confirmed | Raw `appendices/spirvenv.adoc` at commit `8c9361ba`, anchor `[[spirvenv-precision-core-table]]` lines 3991-4016: `OpFAdd` "Correctly rounded.", `OpFMul, OpVectorTimesScalar, OpMatrixTimesScalar` "Correctly rounded.", `OpDot(x, y)` "Inherited from Σᵢ₌₀ⁿ⁻¹ xᵢ × yᵢ" (latexmath). Code site `src/render/shader_function_tests.rs:407` says Vulkan rounds the dot's products and sums correctly; matches. |
| 3 | `vulkan-vendor-id-mesa` | full | confirmed | Raw `xml/vk.xml` at commit `a8b088a3`, line 13498: `<enum value="0x10005" name="VK_VENDOR_ID_MESA" comment="Mesa vendor ID" />`, verbatim. Code site `src/render/acceleration_structures.rs:508`: the test's doc comment says lavapipe reports Mesa's vendor ID; matches. |
| 4 | `pci-vendor-id-nvidia` | full | confirmed | Raw `pci.ids` at commit `3447a24b`, header line 4 `# Version: 2026.10.26`, line 10357 `10de  NVIDIA Corporation`, verbatim. Code site `src/render/acceleration_structures.rs:26-29`: `const NVIDIA_VENDOR: u32 = 0x10DE;`; matches. |
| 5 | `d3d11.3-fp32-rules` | full | confirmed | `D3D11_3_FunctionalSpec.htm`, §3.1.3.1 "Partial Listing of Honored IEEE-754 Rules": the quote is verbatim, the source's own typo "preceise" included. Code site `src/render/shader_function_tests.rs:408` says D3D rounds to 0.5 ULP; matches. |
| 6 | `msl-4.1-fast-math` | full | confirmed | The archived PDF and the live `developer.apple.com` PDF are byte-identical (SHA-256 `41538b30…6cf5`), title page "Version 4.1", footer "2026-06-04", p. 15: `-fmetal-math-mode=<fast, relaxed, safe>` ... "The default is fast. If you set the option to fast, it lets the compiler make aggressive, potentially lossy assumptions about floating-point math. These include no NaNs, no INFs, no signed zeros, allow reciprocal, allow reassociation, and FP contract to be fast." p. 370: "Table 8.2. Accuracy of single-precision operations and functions with fast math enabled", rows `x + y`, `x - y`, `x * y`, `fma` all "Correctly rounded". Code sites `src/render/shader_function_tests.rs:411` (fast math rounds adds, multiplies and fused multiply-adds correctly but may reorder) and `src/render/shaders/bend.wgsl:25` (Metal compiles with fast math) match; ADR 0014 line 15 matches. |
| 7 | `msl-4.1-rounding` | full | confirmed | Same PDF, p. 368, §8.2 Rounding Mode: "Either round ties to even or round toward zero rounding mode may be supported for single-precision, half-precision, and brain floating-point operations." verbatim. Code site `src/render/shader_function_tests.rs:412-416`: takes u = 2⁻²³ to cover rounding toward zero; matches. |
| 8 | `gltf-2.0-specular-brdf` | full | confirmed | Raw `Specification.adoc` at `7cee0fcd`, lines 3675-3730 (Specular BRDF): D and ν as quoted, in latexmath. Code site `src/render/shaders/shading.wgsl:102-121` `distribution_ggx` computes α²/(π d²) with d = (N·H)²α² + |N×H|², the Heaviside by `select`; lines 124-145 `smith_visibility` is ν; matches. |
| 9 | `gltf-2.0-roughness-mapping` | full | confirmed | Same file, lines 3622-3627: "The mapping α = `roughness`^2^ results in more perceptually linear changes in the roughness." and the note "This mapping was suggested by <<Burley2012,Burley (2012)>>." Code sites: `shading.wgsl:37,47,174` α = roughness · roughness; `shader_function_tests.rs:117,121` squares roughness; `src/cook/texture.rs:438-439` "GGX's alpha is its square"; all match. |
| 10 | `gltf-2.0-zero-roughness` | full | confirmed | Same file, line 3741 note, verbatim. Code sites: `shading.wgsl:18-22,86` clamps roughness to `MINIMUM_ROUGHNESS` 0.04; `scripts/cycles-reference/render.py:311-317` clamps with a MAXIMUM node and says praxis clamps rather than mirrors; match. |
| 11 | `gltf-2.0-fresnel` | full | confirmed | Same file, lines 3775-3810: Schlick's F as quoted and `f0 = ((1-ior)/(1+ior))^2` in `fresnel_mix`. Code site `shading.wgsl:92-100` `fresnel_weight` = (1 − |V·H|)⁵ multiplied out; matches. |
| 12 | `gltf-2.0-metal-and-dielectric-brdf` | full | confirmed | Same file, lines 3815-3852: "Note that the dielectric index of refraction `ior = 1.5` is now `f0 = 0.04`." and the full code block line for line as quoted. Code sites: `shading.wgsl:16` `DIELECTRIC_BASE_REFLECTANCE = 0.04`; `shading.wgsl:162-175` mixes the two lobes; `shader_function_tests.rs:1255-1284` `reference_brdf` transcribes the full code; match. |
| 13 | `gltf-2.0-coupling` | full | confirmed | Same file, lines 3875-3877, verbatim. Code site `shader_function_tests.rs:284-288`: the dielectric is held to no energy bound because fresnel_mix is not energy conserving; matches. |
| 14 | `gltf-2.0.1-sample-brdf` | full | confirmed | `registry.khronos.org/glTF/specs/2.0/glTF-2.0.html`, B.3.2 ("and the separable form of the Smith joint masking-shadowing function") and B.3.5 ("we can simplify the mix and arrive at the final BRDF", `c_diff = lerp(baseColor.rgb, black, metallic)`, `f0 = lerp(0.04, baseColor.rgb, metallic)`, `f_diffuse = (1 - F) * (1 / π) * c_diff`). Code site `shading.wgsl:4-7` says 2.0.1 gives a separable Smith G and a simplified mix, which the source's Appendix B revised; matches. |
| 15 | `gltf-2.0-metallic-roughness` | full | confirmed | Raw `Specification.adoc` lines 2582 and 2638: "The factor value is a linear multiplier for the corresponding texture components." and "Its green channel contains roughness values and its blue channel contains metalness values." Code sites: `shading.wgsl:62-89` scales each texel by its factor and says the cooked layout differs (roughness in red, metallic in green, BC5); `render.py:302-309` reads Green and Blue; match. |
| 16 | `gltf-2.0-occlusion` | full | confirmed | Same file, line 2781, verbatim. Code site `shading.wgsl:260-268` applies occlusion to the ambient term only; matches. |
| 17 | `gltf-2.0-emissive-units` | full | confirmed | Same file, line 2691: "The units for the multiplicative product of the emissive texture and factor are candela per square meter (**cd / m^2^**), sometimes called _nits_." ADR 0023 line 3 says emissive gives luminance in cd/m²; matches. |
| 18 | `gltf-2.0-emissive-factor-schema` | full | confirmed | Raw `material.schema.json` lines 44-50, verbatim. ADR 0023 line 14 "texture and factor each at most 1"; matches. |
| 19 | `khr-lights-punctual-units` | full | confirmed | Raw `KHR_lights_punctual/README.md` at `854a757a`, line 90 (intensity row), verbatim. Code sites `src/cook/export.rs:230` (lux) and `:238` (candela); match. |
| 20 | `khr-lights-punctual-range` | full | confirmed | Same file, line 106, verbatim (superscripts as Unicode). Code sites: `shading.wgsl:228-238` `point_light_attenuation` = clamp(1 − (d²/r²)², 0, 1)/d²; `render.py:95-100` the window clamp(1 − (d/r)⁴, 0, 1) with Cycles' own inverse square; `export.rs:240-242`; match. |
| 21 | `walter-2007-projected-area` | full | confirmed | `microfacetbsdf.pdf` p. 3, §3.1: "The (signed) projected area of the microsurface is the same as the projected area of the macrosurface for any direction v" ... "and in the special case, v = n: 1 = ∫ D(m)(n · m) dωm (4)". Code site `shader_function_tests.rs:54-60` integrates D(cos θ) cos θ over the hemisphere to 1; matches. |
| 22 | `walter-2007-ggx` | full | confirmed | Same PDF p. 7, §5.2: "GGX Distribution with width parameter αg: D(m) = αg² χ⁺(m · n) / (π cos⁴θm (αg² + tan²θm)²) (33)". Code site `shader_function_tests.rs:115-125`: the peak at n·h = 1 is 1/(π α²), which eq. 33 gives at θm = 0; matches. |
| 23 | `filament-punctual-light-1cm` | full | confirmed | Raw `shaders/src/surface_light_punctual.fs` at `26b42777`, lines 107-108, verbatim. Code site `shading.wgsl:223-226,254` `MINIMUM_LIGHT_DISTANCE_SQUARED = 1e-4` as a `max` on distance²; matches. |
| 24 | `hoffman-2013-f0` | full | confirmed | `s2013_pbs_physics_math_notes.pdf` p. 17, Table 1: header "Material F0 (Linear) F0 (sRGB) Color", row "Gold 1.00,0.71,0.29 1.00,0.86,0.57", caption "table from “Real-Time Rendering, 3rd edition”". Code sites `src/scenes/mod.rs:518-522` `[1.0, 0.71, 0.29, 1.0]` and `shader_function_tests.rs:1179-1182` `[1.0, 0.71, 0.29]`; match. |
| 25 | `gt7-tone-mapping-cpp` | full | confirmed | `gt7_tone_mapping.cpp` downloaded, SHA-256 `94df7e7e310b9423ebb5aaaa29416f376ebbae66e0948fb259f8688314b29822` (matches the registry's `94df7e7e…9822`); line 43, 50 and 347 verbatim; lines 396-429 are `applyToneMapping`. Code sites `src/render/shaders/gt7.wgsl:5,30-32` (`GT7_PAPER_WHITE = 2.5` = 250/100) and `scripts/cycles-reference/compare.py:27-31` cite it as the ported sample under its MIT notice; match. |
| 26 | `gt7-slides` | full | confirmed | `s2025_pbs_pdi_slides_v1.1.pdf` p. 153 "Sample Implementation": "We assume a paper white = 250 nits", "In sRGB, 1.0 = 100 nits; we assume 2.5x brightness.", "We multiply by 1 / 2.5 after SDR tone mapping to adjust." Code site `gt7.wgsl:30-32`; matches. |
| 27 | `bt2100-pq` | full | confirmed | `R-REC-BT.2100-3-202502-I!!PDF-E.pdf` p. 6, Table 4: "FD = EOTF[E′] = 10000 Y", m1, m2, c1, c2, c3 exactly as quoted. Code site `gt7.wgsl:34-40` constants 0.1593017578125, 78.84375, 0.8359375, 18.8515625, 18.6875; `:79` scales by 10000/100; match. |
| 28 | `bt2100-ictcp` | full | confirmed | Same PDF p. 10, Table 7: L, M, S rows and I = 0.5L′ + 0.5M′, CT = (6610L′ − 13613M′ + 7003S′)/4096, CP = (17933L′ − 17390M′ − 543S′)/4096 as quoted (the HLG column differs and is not quoted). Code site `gt7.wgsl:82-95`: LMS rows 1688/2146/262, 683/2951/462, 99/309/3688 over 4096, I as 2048/2048/0 over 4096; `shader_function_tests.rs:837`; match. |
| 29 | `bt2087-rec709-to-rec2020` | full | confirmed | `R-REC-BT.2087-0-201510-I!!PDF-E.pdf` p. 6, Annex 1 M2: the sentence, the 3×3 matrix 0.6274 0.3293 0.0433 / 0.0691 0.9195 0.0114 / 0.0164 0.0880 0.8956 and NOTE 3 as quoted. Code site `src/render/shaders/display.wgsl:4-12`: the seven-place matrix rounds to BT.2087's four places in every cell (0.6274039→0.6274, 0.0113623→0.0114, 0.8955953→0.8956, …); matches. |
| 30 | `bt709-6-primaries` | full | confirmed | `R-REC-BT.709-6-201506-I!!PDF-E.pdf` p. 5, items 1.3 and 1.4: R 0.640 0.330, G 0.300 0.600, B 0.150 0.060, D65 0.3127 0.3290. Code sites `display.wgsl:6` and `shader_function_tests.rs:925` cite them for the derived matrix; the matrix is consistent with these primaries (see 29). |
| 31 | `bt2020-2-primaries` | full | confirmed | `R-REC-BT.2020-2-201510-I!!PDF-E.pdf` p. 5, Table 3 System colorimetry: R 0.708 0.292, G 0.170 0.797, B 0.131 0.046, D65 0.3127 0.3290. Code sites `display.wgsl:7`, `shader_function_tests.rs:925`; consistent. |
| 32 | `bt709-6-luma` | full | confirmed | BT.709-6 p. 6, item 3.2 "Derivation of luminance signal E′Y: E′Y = 0.2126 E′R + 0.7152 E′G + 0.0722 E′B". Code sites `src/render/targets.rs:47-52` `LUMINANCE_COEFFICIENTS = [0.2126, 0.7152, 0.0722]` and `shader_function_tests.rs:654-657`; match. |
| 33 | `khronos-dfs-srgb` | full | confirmed, with a Finding at one code site | `dataformat.1.3.html` §13.3.2 sRGB EOTF⁻¹: "The corresponding sRGB EOTF⁻¹ conversion from linear (R, G, B) encoding to non-linear (R′, G′, B′) encoding is: R′ = R × 12.92, R ≤ 0.0031308; 1.055 × R^(1/2.4) − 0.055, R > 0.0031308", verbatim. Code sites `display.wgsl:27-35` (`select` with `clamped <= 0.0031308`), `src/cook/texture.rs:397-404` (`linear <= 0.0031308`) and `shader_function_tests.rs:1020-1025` implement the encode as quoted. `shader_function_tests.rs:1169-1177` cites this key for `srgb_decode`, the EOTF with knee 0.04045, which this entry's quote does not state: Finding L7-S01. |
| 34 | `khronos-dfs-srgb-eotf` | full | confirmed | Same page §13.3.1 sRGB EOTF: "The EOTF conversion from non-linear (R′, G′, B′) encoding to linear (R, G, B) encoding is: R = R′ / 12.92, R′ ≤ 0.04045; ((R′ + 0.055) / 1.055)^2.4, R′ > 0.04045", verbatim. Code site `src/bench/mod.rs:284-292`: `encoded <= 0.04045` then `/ 12.92`, else `((encoded + 0.055) / 1.055).powf(2.4)`; matches. |
| 35 | `wgsl-f16-largest` | full | confirmed | `w3.org/TR/2026/CRD-WGSL-20261021/#floating-point-types`, table "Extreme values for floating point types", f16 row: Largest positive finite `65504.0h`, `0x1.ffcp+15h`. Code site `src/render/shaders/hdr.wgsl:4-7` `LARGEST_HALF: f32 = 65504.0`; matches. |
| 36 | `vulkan-fp-conversion` | full | confirmed | Raw `chapters/fundamentals.adoc` at `a8b088a3`, anchor `[[fundamentals-fp-conversion]]` lines 1626-1635, verbatim. Code site `hdr.wgsl:5-7` (clamped so Vulkan's allowed infinity on overflow is never stored) and ADR 0023 line 12; match. |
| 37 | `frostbite-2014-exposure` | full | confirmed | `course_notes_moving_frostbite_to_pbr_v32.pdf` PDF pp. 85-87: p. 85 "with L the incident luminance and q the lens and vignetting attenuation (a typical value is q = 0.65, [Wikc])"; p. 86 eq. (75) `Lmax = 78 N² / (S q t)` (the quote's `78/S · N²/(q t)` is the same expression regrouped) and Listing 28 lines 27-30 `// maxLum = 78 / ( S * q ) * N^2 / t`, `= 78 / (100 * 0.65) * 2^EV_100`, `= 1.2 * 2^EV`, line 32 `float maxLuminance = 1.2f * pow(2.0f, EV100);`; p. 87 line 33 `return 1.0f / maxLuminance;`. Code sites `src/camera/mod.rs:7-15` (`SATURATING_LUMINANCE_AT_EV100_ZERO = 1.2`), `:44-50` (`1.0 / (1.2 · 2^EV100)`), `:128-134` (test at EV100 0, −1, 15); match. |
| 38 | `frostbite-2014-relative-lights` | full | confirmed | Same PDF p. 95, §6: "For lights: We have found lights to be extremely hard to convert from arbitrary units to physical based units." verbatim. ADR 0023 line 7 quotes "extremely hard"; matches. |
| 39 | `frostbite-2014-light-units` | full | confirmed | Same PDF p. 29, Table 8: row "Punctual Luminous power (lm)", caption "Table 8: Light units associated with various Frostbite light types." ADR 0023 line 8 (Frostbite authors point lights in lumens); matches. |
| 40 | `filament-ggx-lagrange-identity` | full | confirmed | Raw `shaders/src/surface_brdf.fs` at `26b42777`, lines 62-70: the Lagrange's-identity comment and, under `TARGET_MOBILE`, `vec3 NxH = cross(shading_normal, h); float oneMinusNoHSquared = dot(NxH, NxH);`, verbatim. Code site `shading.wgsl:105-121` takes `normal_cross_half_squared` as 1 − (N·H)² and names Filament's mobile path; matches. |
| 41 | `filament-light-units` | full | confirmed | Raw `docs/Filament.md.html` at `26b42777`, line 1238: `Point light              | Luminous power ($lm$)`. ADR 0023 line 8; matches. |
| 42 | `bevy-point-light-lumens` | full | confirmed | Raw `crates/bevy_light/src/point_light.rs` at tag `v0.20.0`, line 53: `/// Luminous power in lumens, representing the amount of light emitted by this source in all directions.` ADR 0023 line 8; matches. |
| 43 | `godot-light-units` | full | confirmed | `docs.godotengine.org/en/4.7/tutorials/3d/physical_light_and_camera_units.html`: "As Godot favors ease of use by default, physical light units are disabled by default." and, under "Configure the light nodes", "Intensity: The light's intensity in lux (DirectionalLight3D) or lumens (OmniLight3D/SpotLight3D/AreaLight3D)." ADR 0023 line 8 ("Godot with its physical light units enabled"); matches. |
| 44 | `bevy-gltf-point-candela` | full | confirmed | Raw `crates/bevy_gltf/src/loader/mod.rs` at `v0.20.0`, lines 1793-1796, verbatim. ADR 0023 line 8 ("Bevy's glTF loader multiplies by 4π"); matches. |
| 45 | `bevy-lux-constants` | full | confirmed | Raw `crates/bevy_light/src/lib.rs` at `v0.20.0`, lines 131-132 and 157: `MOONLESS_NIGHT: f32 = 0.0001` with its doc line, `DIRECT_SUNLIGHT: f32 = 100_000.`, verbatim. Code site `src/scenes/mod.rs:470-473` `SUN_ILLUMINANCE = 100_000.0`; ADR 0023 line 3 "29.9 stops" = log₂(100 000 / 0.0001) = 29.897; match. |
| 46 | `blender-gltf-light-export` | full | confirmed | Raw `addons/io_scene_gltf2/blender/exp/lights.py` at `e03d6603`, lines 178-192 as quoted (gaps marked), and `blender/com/conversion.py` line 20 `PBR_WATTS_TO_LUMENS = 683`. ADR 0023 line 16 (W/(4π)·683 candela, W/m²·683 lux, times 2^exposure); matches. |
| 47 | `karis-2013-weighted-average` | full | confirmed | `graphicrants.blogspot.com/2013/12/tone-mapping.html`: "This operation, when used to reduce fireflies, can also be thought of as a weighting function for each sample: weight = 1/(1 + luma)" (MathJax) "For a weighted average, sum all samples and divide by the summed weights. The result will be the same as if the samples were tone mapped using T with range of 1, averaged, then inverse tone mapped using T_inverse." Code sites `hdr.wgsl:16-30` (`color / (1 + luminance)` and its inverse) and `shader_function_tests.rs:680-690`; match. |


## Registry entries 48 to 97

47 entries (indices 48 to 71, 73 to 83, 85 to 94, 96 and 97; 72, 84 and 95 are not in this slice): 46 `full` and 1 `sample` (59, `vulkan-anisotropy-lod`). No sample was wrong, so no `skip` entry was opened. 0 unopened. Every source was opened: the 19 wgpu and glam entries against the crates under `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` at the versions `Cargo.lock` pins (wgpu 30.0.1, wgpu-core 30.0.1, wgpu-hal 30.0.1, wgpu-types 30.0.1, glam 0.32.1); the pinned GitHub and GitLab sources as raw files at their commit or tag; the PDFs downloaded and read at the cited page with `pdftotext`; the web.archive.org pages fetched with curl, since WebFetch refuses that host; Apple's page through its documentation JSON endpoint; the wgpu issue through `gh issue view`. Every `full` entry's code site under `src/` was opened and matches its quote. 0 Findings.

### Verdicts

| index | key | check | verdict | what was opened, and what it says |
|---|---|---|---|---|
| 48 | peters-blue-noise | full | confirmed | momentsingraphics.de/BlueNoise.html, section Dithering (Listing 1) and the CC0 sentence, word for word; `FreeBlueNoiseTextures.zip` downloaded, holds `Data/64_64/LDR_LLL1_0.png` (the page itself names only the zip). `src/render/shaders/display.wgsl:47-50` implements the remap `2u−1`, `sign(c)(1−√(1−|c|))` as Listing 1; `src/render/tonemapping_pass.rs:21` cites the file and CC0, as `assets/manifest.json:80-86` and `assets/ATTRIBUTION.md:21` do. |
| 49 | lipshitz-1992-triangular-dither | full | confirmed | Lipshitz_1992.pdf page 18 (journal p. 372, §5.4 Summary), item 5), word for word. `src/render/shaders/display.wgsl:44-46`: "triangular noise of two steps peak to peak adds the least noise" of the dithers removing noise modulation; matches. |
| 50 | frostbite-2014-fp16 | full | confirmed | course_notes_moving_frostbite_to_pbr_v32.pdf PDF page 91, inside §5.2 Manipulation of high values (heading on p. 90, §5.3 on p. 92): the 65504 sentence then the table row "Light buffer" / "Float16. Exposure is pre-applied.", adjacent in the source. `docs/adr/0023:12` claims it so. |
| 51 | unreal-scene-color-overflow | full | confirmed | web.archive.org/web/20260302051755 copy of Using Physical Lighting Units, fetched with curl: the "black patches" bullet under the physically based lighting workflow suggestions, word for word. `docs/adr/0023:12` claims it so. |
| 52 | bt2087-rec709-to-xyz | full | confirmed | R-REC-BT.2087-0 PDF page 6, Annex 1, M2: the right-hand factor reads 0.4124 0.3576 0.1805 / 0.2126 0.7152 0.0722 / 0.0193 0.1192 0.9505. `src/render/targets.rs:52` `LUMINANCE_COEFFICIENTS = [0.2126, 0.7152, 0.0722]`, the Y row; matches. |
| 53 | wgpu-hal-30-metal-compile-options | full | confirmed | wgpu-hal-30.0.1/src/metal/device.rs:272-278, word for word (L274 is the comment the `[...]` skips); no `MathMode` or fast-math setter anywhere under `src/metal/` (grep). `src/render/shader_function_tests.rs:408-409`: "Metal's fast math, which wgpu-hal leaves at its default"; matches. |
| 54 | gltf-2.0-bitangent | full | confirmed | raw Specification.adoc at 7cee0fc, line 1763, word for word. `src/render/shaders/main.wgsl:187` `cross(geometric_normal, tangent) * in.tangent_world.w`; matches. |
| 55 | vulkan-vendorid-pci | full | confirmed | raw devsandqueues.adoc at a8b088a, lines 155-157, word for word. `src/render/acceleration_structures.rs:25-28` `NVIDIA_VENDOR = 0x10DE` reported by Vulkan and DXGI; matches. |
| 56 | dxgi-adapter-desc2-vendorid | full | confirmed | learn.microsoft.com DXGI_ADAPTER_DESC2, member VendorId, word for word. Code site as 55; matches. |
| 57 | lavapipe-vendor-id-mesa | full | confirmed | gitlab raw lvp_device.c at mesa-25.2.8, line 886 `.vendorID = VK_VENDOR_ID_MESA,`. `src/render/acceleration_structures.rs:506-509` test comment: lavapipe reports Mesa's vendor ID; matches. |
| 58 | vulkan-standard-sample-locations | full | confirmed | raw primsrast.adoc at a8b088a, lines 721-750: the standardSampleLocations sentence, the upper-left origin, and the 1, 2 and 4-sample rows, word for word. `src/render/coverage_tests.rs:98-108` and `src/render/shadow_mask_pass.rs:38-45` hold exactly those positions; matches. |
| 59 | vulkan-anisotropy-lod | sample | confirmed | raw textures.adoc at v1.4.367, lines 1645-1694: η = min(ρmax/ρmin, maxAniso), N = ⌈η⌉, "An implementation may: use the value of N as an approximation of η", λbase log₂(ρmax/η) otherwise; word for word modulo asciidoc markup. |
| 60 | vulkan-acceleration-structure-instance | full | confirmed | raw vk.xml at a8b088a, lines 6713-6725, word for word; float[3][4] + 24:8 + 24:8 + uint64 = 64 bytes as the section says. `src/render/acceleration_structures.rs:291-296` `_ => 64`; matches. |
| 61 | d3d12-raytracing-instance-desc | full | confirmed | raw Raytracing.md at f2917db, lines 4378-4386, word for word; 64 bytes. Code site as 60; matches. |
| 62 | metal-indirect-instance-descriptor | full | confirmed | developer.apple.com documentation JSON for MTLIndirectAccelerationStructureInstanceDescriptor: the memberwise init signature word for word; properties MTLPackedFloat4x3 (48) + four UInt32 (16) + MTLResourceID (8) = 72. `src/render/acceleration_structures.rs:288-290` `Metal => 72`; matches (wgpu-hal metal/adapter.rs:1503-1506 sizes it by `size_of` of the same struct). |
| 63 | msl-4.1-sin-fast-accuracy | full | confirmed | archived Metal-Shading-Language-Specification.pdf (Version 4.1, dated 2026-06-04) page 372, Table 8.2 row sin(x), word for word (2⁻¹³ in Unicode). `src/render/acceleration_structures.rs:444-445` "sine errs by up to 2^-13 within ±π" and `src/render/shaders/bend.wgsl:24-27`; both match. |
| 64 | glam-0.32.1-perspective-infinite-reverse-rh | full | confirmed | glam-0.32.1/src/f32/neon/mat4.rs:1186-1204, word for word; the sse2 (L1197) and scalar (L1100) bodies are identical. `src/camera/mod.rs:89-96` depth 1 at the near plane, 0 at infinity, cleared to 0 with `Greater`; matches. |
| 65 | metal-feature-set-tables-msaa-max | full | confirmed | archived Metal-Feature-Set-Tables.pdf (footer "May 21, 2026", 18 pages) page 9, Render targets table (heading on p. 8), row "Maximum sample count in render passes with MSAA 4 4 4 4 4 4 4 4 8" under columns Apple2 to Apple10. `docs/adr/0019:3` "4x MSAA (the Apple9 maximum)": Apple9's column is 4, Apple10's 8; matches. |
| 66 | metal-feature-set-tables-mesh-ray-tracing | full | confirmed | same PDF page 6, note 6, word for word (the apostrophe drops in extraction). `docs/adr/0019:20` and `docs/adr/0021:8` claim it so. |
| 67 | nasa-sun-apparent-diameter | full | confirmed | web.archive.org/web/20260930033014 copy of sunfact.html via curl: "Apparent diameter from Earth At 1 A.U.(seconds of arc) 1919.", "Last Updated: 9 May 2024"; 1919″/3600 = 0.533°. `docs/adr/0020:12` "The sun's 0.53°"; matches. |
| 68 | wgpu-30-blas-rebuild-only | full | confirmed | wgpu-core-30.0.1/src/command/ray_tracing.rs:1106-1126: `log::debug!("only rebuild implemented")` at L1107, `mode: hal::AccelerationStructureBuildMode::Build` at L1126. `src/render/blas_refit.rs:5-6` "wgpu-core rebuilds BLASes but never updates them"; matches. |
| 69 | wgpu-30-tlas-rebuild-only | full | confirmed | same file L336-341, both quoted lines word for word. `docs/adr/0020:20` "wgpu-core has no refit"; matches. |
| 70 | wgpu-30-tlas-blas-build-order | full | confirmed | same file L515-534: the `UseTlas` arm with `if blas_build_index.unwrap() > tlas_build_index.unwrap()` at L530; L244-250 stage `blas_address: blas.handle`. `src/render/blas_refit.rs:228-230` the refit changes no handle or build index so the TLAS validates; matches. |
| 71 | wgpu-30-tlas-instance-staging | full | confirmed | same file L235-252: `instance_buffer_staging_source.extend(state.device.raw().tlas_instance_to_bytes(` at L244. `src/render/acceleration_structures.rs:222-225`; matches. |
| 73 | wgpu-30-blas-sizes | full | confirmed | wgpu-types-30.0.1/src/ray_tracing.rs:11-24, the doc line and the two fields word for word; `features.rs` has no "cluster" (grep). `docs/adr/0019:9` "sizes every BLAS on the CPU"; matches. |
| 74 | wgpu-30-as-hal-mut-safety | full | confirmed | wgpu-30.0.1/src/api/command_encoder.rs:270-275, Safety list, L273 word for word. `src/render/blas_refit.rs:204-206` "never ended here, and finished by wgpu"; matches. |
| 75 | wgpu-30-mixed-encoding-panics | full | confirmed | wgpu-core-30.0.1/src/command/mod.rs:562-571, `EncodingApi::set`, panic at L570 word for word. `src/render/blas_refit.rs:207`; matches. |
| 76 | wgpu-30-metal-retain-references | full | confirmed | wgpu-hal-30.0.1/src/metal/mod.rs:383-393, `Settings::default` with `retain_command_buffer_references: true` at L391; command.rs:503-508 picks `queue.commandBuffer()` when set. `src/render/blas_refit.rs:208-212`; matches. |
| 77 | wgpu-30-metal-buffer-tracked | full | confirmed | wgpu-hal-30.0.1/src/metal/device.rs:465-478, `create_buffer`, `//TODO: HazardTrackingModeUntracked` at L478 and no untracked option set. `src/render/blas_refit.rs:220-222` "wgpu-hal never opts out"; matches. |
| 78 | wgpu-30-metal-barriers-empty | full | confirmed | wgpu-hal-30.0.1/src/metal/command.rs:582-592, both bodies empty, word for word. `src/render/blas_refit.rs:231-232`; matches. |
| 79 | wgpu-30-metal-scratch-alignment | full | confirmed | wgpu-hal-30.0.1/src/metal/adapter.rs:1503-1507, `ray_tracing_scratch_buffer_alignment: 1,` at L1507. `src/render/blas_refit.rs:295-297` `SCRATCH_ALIGNMENT = 256`, a margin over 1; matches. |
| 80 | wgpu-30-raw-boxed | full | confirmed | wgpu-core-30.0.1/src/resource.rs:509 word for word; `Blas.raw: Snatchable<Box<dyn hal::DynAccelerationStructure>>` at L2528. `src/render/blas_refit.rs:311-313`; matches. |
| 81 | wgpu-30-metal-indirect-loop | full | confirmed | wgpu-hal-30.0.1/src/metal/command.rs:1608-1649, `for _ in 0..draw_count {` in both `draw_indexed_indirect` and `draw_mesh_tasks_indirect`. `docs/adr/0019:15` "runs `multi_draw_indirect` as a CPU loop"; matches. |
| 82 | wgpu-30-metal-count-draws-empty | full | confirmed | same file L1651-1670, both count draws with `//TODO` bodies, word for word. `docs/adr/0019:15` "wgpu's Metal backend has none"; matches. |
| 83 | wgpu-30-metal-memoryless | full | confirmed | wgpu-hal-30.0.1/src/metal/device.rs:561-564 word for word. `docs/adr/0019:3` "memoryless on Apple GPUs"; matches. |
| 85 | wgpu-issue-9215 | full | confirmed | `gh issue view 9215 -R gfx-rs/wgpu`: title "Metal BLAS -> TLAS builds need synchronisation", OPEN, body sentence word for word. `src/render/blas_refit.rs:39-41` "nothing orders a TLAS build after the BLASes it reads are written"; matches. |
| 86 | smaa-subpixel-grid | full | confirmed | SMAA PDF page 7, Figure 10 caption, word for word ("[JME 11]" spaced by extraction). `docs/adr/0019:3` "SMAA 1x cannot reconstruct sub-pixel geometry"; matches. |
| 87 | nanite-taa-lod | full | confirmed | Karis_Nanite PDF page 67, slide Seamless LOD, speaker notes, word for word. `docs/adr/0019:9` "Nanite hides its LOD switches with TAA"; matches. |
| 88 | nanite-two-pass-occlusion | full | confirmed | same PDF page 19, slide Two pass occlusion culling, the four quoted bullets word for word. `docs/adr/0019:15`; matches. |
| 89 | ue-nanite-unsupported | full | confirmed | web.archive.org/web/20251221042229 copy of the UE 5.7 Nanite page via curl: "The following rendering features are not currently supported:" list with Forward Rendering and Multisampling Anti-Aliasing (MSAA); the page adds "(Experimental) Initial support for native ray-tracing of Nanite meshes". `docs/adr/0019:9` "no MSAA in Nanite"; matches. |
| 90 | bevy-meshlet-limits | full | confirmed | raw bevy v0.20.0 crates/bevy_pbr/src/meshlet/mod.rs:94-99, the two Msaa sentences word for word at L98-99. `docs/adr/0019:9` "no MSAA in ... Bevy"; matches. |
| 91 | forward-plus-tile-depth | full | confirmed | forward_plus.pdf page 2, under 4.1 Gather Approach ("It executes a thread group per tile. A frustum of the tile is calculated..."), word for word. `docs/adr/0019:11` "tiled culling bounds each tile's lights by its opaque depth range"; matches. |
| 92 | persson-forward-plus-transparency | full | confirmed | PracticalClusteredShading.pdf page 5, slide Solutions we've been eyeing: Forward+ bullets "Forces Pre-Z pass", "MSAA works fine", "Transparency requires another pass"; Clustered shading "No Pre-Z necessary", "Transparency works fine"; word for word. Read from the copy WebFetch saved from humus.name and from a web.archive.org copy (curl alone gets 403 from humus.name). `docs/adr/0019:11`; matches. |
| 93 | drobot-zbinning | full | confirmed | 2017_Sig_Improved_Culling_final.pdf page 30, "F+ Renderer : Z-binning algorithm", the four quoted bullets word for word. `docs/adr/0019:11` "Z-binning intersects tile and depth bins per pixel at shading time"; matches. |
| 94 | greene-hiz | full | confirmed | greene93.pdf page 3, §3.2 Image-space Z pyramid, both quoted passages word for word. `docs/adr/0019:15` Hi-Z pyramid of minimum reverse-Z depth (farthest Z, in reverse-Z the minimum); matches. |
| 96 | wgpu-30-dx12-vendor-id | full | confirmed | wgpu-hal-30.0.1/src/dx12/adapter.rs: `let desc = unsafe { adapter.GetDesc2() }.unwrap();` at L76 and `vendor: desc.VendorId,` at L194, inside `Adapter::expose` (L65). `src/render/acceleration_structures.rs:27-28`; matches. |
| 97 | wgpu-30-metal-refit | full | confirmed | wgpu-hal-30.0.1/src/metal/command.rs:1855-1873: `Build` calls `buildAccelerationStructure_...`, `Update` calls `refitAccelerationStructure_descriptor_destination_scratchBuffer_scratchBufferOffset(` at L1866-1867. `src/render/blas_refit.rs:3-4` "refits each in place through wgpu-hal"; matches. |


## Registry entries 98 to 217

47 entries in the slice: 28 `full` (cited from `src/`, `tests/` or `docs/adr/`) and 19 `sample`. One sample, `ign-mdt05-distribution` (152), was `unsupported`, which opened the two `skip` entries of the `ign` family (150, 151); both confirmed. One entry is `unopened`: `hillis-steele-partial-sums` (102), whose `doi.org` url redirects to `dl.acm.org`, which answers 403 to WebFetch and to curl, and the archive is unreachable from this tool. 47 verdicts are `confirmed`, 1 is `unsupported`, 1 is `unopened`. Local sources were used for every wgpu, wgpu-core, wgpu-hal, wgpu-types (all 30.0.1), meshopt 0.6.2 (which vendors meshoptimizer 0.25, `MESHOPTIMIZER_VERSION 250`) and offset-allocator 0.2.0 entry; pinned GitHub urls were read as raw files at their commit or tag; PDFs were read at the cited page with `pdftotext` after WebFetch saved them.

### Verdicts

| index | key | check | verdict | what was opened, and what it says |
|---|---|---|---|---|
| 98 | `metal-hazard-tracking-default` | full | confirmed | The url renders through JavaScript and WebFetch saw only its title, so Apple's data endpoint for the same page (`developer.apple.com/tutorials/data/documentation/metal/mtlhazardtrackingmode/default.json`) was read: its Discussion says word for word "The default tracking mode for a type that inherits MTLResource is MTLHazardTrackingMode.tracked because individual resources benefit from automatic hazard tracking." Code site `src/render/blas_refit.rs:221` claims only that Metal tracks a resource by default; matches. |
| 99 | `wgpu-30-metal-event-staging` | full | confirmed | `~/.cargo/registry/src/*/wgpu-hal-30.0.1/src/metal/mod.rs` L551-L553, the doc comment of `add_wait_event`: the quoted sentence is there word for word, L557 is the `pub fn`. Code site `src/render/blas_refit.rs:531` says the submission right after the staging takes it, whichever it is, and relies on only the main thread submitting; matches. |
| 100 | `wgpu-30-core-flush-submit` | full | confirmed | `wgpu-core-30.0.1/src/device/queue.rs` L1286-L1304, `flush_writes_for_buffer`: the three quoted statements are there in order. Code site `src/render/blas_refit.rs:535` says mapping a buffer with pending writes makes wgpu-core submit on its own; matches. |
| 101 | `webgpu-standard-sample-pattern` | full | confirmed | Raw `spec/index.bs` at commit 688f043e, L16579-L16597: the introductory sentence, the 4x table (0.375, 0.125), (0.875, 0.375), (0.125, 0.625), (0.625, 0.875) and the "Implementations must use" sentence all match. Code site `src/render/shaders/light_count.wgsl:9-10`: `FIRST_SAMPLE = vec2(0.375, 0.125)`; matches. |
| 102 | `hillis-steele-partial-sums` | full | unopened | `doi.org/10.1145/7902.7903` redirects (302) to `dl.acm.org/doi/10.1145/7902.7903`, which answered 403 to WebFetch and to curl; the archive is not reachable from this tool. Code site `src/render/shaders/culling.wgsl:78-95` is a doubling-stride workgroup scan read one place back, which is what the entry's section names. |
| 104 | `toksvig-2004-variance` | full | confirmed | NVIDIA PDF (TB-01256-001_v01, "Technical Brief", p. 1), read with pdftotext: "A good middle ground is to assume a Gaussian distribution of the angular deviation, α with a standard deviation of σ." sits at the foot of PDF page 3 (the quote drops the inline symbol "α", see Notes), and eq. (1) `|Na| = 1/(1+σ²) ⇔ σ² = (1−|Na|)/|Na|` is on PDF page 4 as cited. Code site `src/cook/texture.rs:448-456`: `angular_variance = variance / ((1 + m) m)` with `m = sqrt(1 − variance)`, which equals (1 − m)/m; matches. |
| 105 | `kaplanyan-2016-ndf-filtering` | full | confirmed | NVIDIA research PDF, page 6: eq. (14) `Σ_B = [α_s²/2 0; 0 α_t²/2]`, "where we use the relation α² = 2σ² [BS63] between the standard deviation of a Gaussian distribution and the roughness parameter of a Beckmann distribution", eq. (15) `Σ′_B = Σ_B + Σ_P`; page 9: "we call it a Beckmann proxy for GGX, or simply GGX Proxy." Code site `src/cook/texture.rs:457-467` adds `2 * angular_variance` to alpha²; matches. |
| 106 | `tokuyoshi-2019-roughness-matrix` | full | confirmed | Square Enix PDF, page 3 §2.2: "In this paper, we refer to the above matrix as a roughness matrix, which is two times the covariance matrix by definition of the Beckmann NDF. It was also shown that this roughness matrix can be used to approximate the filtering of the GGX NDF."; page 4: eq. (1) `ᾱ² = α² + min(2λ_max, κ)`, "where α is the original roughness parameter for the isotropic NDF, and κ = 0.18 is the clamping threshold used in the Kaplanyan et al. [2016] implementation". Code sites: `src/render/shaders/shading.wgsl:26-28` `MAXIMUM_FILTER_ALPHA_SQUARED = 0.18`; `src/render/shader_function_tests.rs:1348` κ = 0.18; `src/cook/texture.rs:459-464` says Tokuyoshi and Kaplanyan keep Kaplanyan et al.'s cap of 0.18 and the cooker omits it for a texel-measured variance, which the source supports; all match. |
| 107 | `tokuyoshi-2019-normal-derivatives` | full | confirmed | Same PDF, page 3: "where σ = 0.5 is the standard deviation of the pixel filter kernel in image space measured in pixels"; page 5: "Hence, our roughness for the filtered isotropic NDF is yielded as ᾱ² = α² + min(2σ²(∥∆nᵤ∥² + ∥∆nᵥ∥²), κ). (4)" and Listing 2 `float filteredRoughness2 = saturate(roughness2 + kernelRoughness2);`. Code sites: `shading.wgsl:25,27` `PIXEL_FILTER_VARIANCE = 0.25` (σ² for σ = 0.5), `shading.wgsl:36` the formula capped at 1; `shader_function_tests.rs:1346` the same; match. |
| 108 | `gltf-2.0-texture-origin` | full | confirmed | Raw `Specification.adoc` at commit 7cee0fcd, L2299: "The origin of the texture coordinates (0, 0) corresponds to the upper left corner of a texture image." Code site `src/cook/mesh.rs:258`: glTF's v runs down the image, so v is flipped for MikkTSpace; matches. |
| 109 | `gltf-2.0-normal-texture-convention` | full | confirmed | Raw `material.schema.json` at the same commit, L65 (`normalTexture.gltf_detailedDescription`): the three quoted sentences are there word for word. Code site `src/cook/mesh.rs:259`: normal Textures take +Y as up; matches. |
| 110 | `gltf-2.0-normal-texture-scale` | full | confirmed | Raw `Specification.adoc`, L2769-L2776: the RGB-to-XYZ mapping, the `scale` sentence, the latexmath `(sampledNormalValue_rgb · 2.0 − 1.0) ⊙ ⟨scale, scale, 1.0⟩` and "Finally, the scaled normal vector is normalized before being used in lighting equations." all match (the quote skips the SHOULD NOT sentence with `[...]`). Code sites `src/render/shaders/shading.wgsl:152-160` (x, y scaled, z rebuilt, normalised last in World space) and `src/render/shader_function_tests.rs:1292`; match. |
| 111 | `gltf-2.0-tangent-bitangent` | full | confirmed | Raw `Specification.adoc`, L1602 (`TANGENT` row: "XYZW vertex tangents where the XYZ portion is normalized, and the W component is a sign value (-1 or +1) indicating handedness of the tangent basis") and L1763 (the bitangent cross-product sentence); both word for word. Code sites `src/cook/export.rs:617` and `src/cook/mesh.rs:299` call w the bitangent sign and build cross(normal, tangent); match. |
| 112 | `gltf-2.0-mikktspace-tangents` | full | confirmed | Raw `Specification.adoc`, L1759: the sentence matches word for word (the source's `<<mikktspace,MikkTSpace>>` cross-reference rendered as "MikkTSpace"). Code site `src/cook/mesh.rs:278`: glTF computes a Primitive's missing tangents this way; matches. |
| 113 | `ktx2-level-order` | full | confirmed | Raw `ktxspec.adoc` at commit ac39917e, L598-L600: "Mip level data is ordered from the level with the smallest size images, level_p to that with the largest size images, level_base" (stem macros rendered). Code site `src/assets/cooked.rs:255`: smallest mip first; matches. |
| 114 | `ktx2-supercompression-per-level` | full | confirmed | Same file, L634 "| 2 | Zstandard | <<RFC8478>> | n/a" and L647-L648 "The supercompression scheme is applied independently to each mip level to permit streaming and random access to the levels." Code site `src/assets/cooked.rs:256`: each level zstd-compressed on its own; matches. |
| 115 | `ktx2-level-alignment` | full | confirmed | Same file, L888-L893: the stem block `required_alignment = lcm(texel_block_size, 4)` for `supercompressionScheme = 0` and `1` otherwise. Code site `src/assets/cooked.rs:257`: no level alignment under supercompression; matches. |
| 116 | `ktx2-key-value-alignment` | full | confirmed | Same file, L1403-L1407: the sentence matches word for word. Code sites `src/assets/cooked.rs:23-26` (`PXTXVERSION`, 11 bytes plus NUL = 12, a multiple of 4), `:259` and the test at `:854-858`, which asserts `key.len().is_multiple_of(4)`; match. |
| 117 | `wgpu-hal-30-dx12-max-buffer` | full | confirmed | `wgpu-hal-30.0.1/src/dx12/adapter.rs` L913-L916: the three comment lines and `max_buffer_size: i32::MAX as u64,` word for word. ADR sites `docs/adr/0021-capability-floor.md:16` (`max_buffer_size` at least 2^31 − 1) and `docs/adr/0017-...:17` (DX12, whose API exposes no maximum); match. |
| 118 | `wgpu-hal-30-vulkan-max-buffer` | full | confirmed | `wgpu-hal-30.0.1/src/vulkan/adapter.rs` L1504-L1514: the comment, `is_nvidia` and the cap `i32::MAX as u64` on Linux/Android non-NVIDIA, else `1u64 << 52`, word for word with the `[...]` over the `is_nvidia` line. ADR 0017:17 "Linux and Android Vulkan other than NVIDIA's"; matches. |
| 119 | `offset-allocator-tlsf` | full | confirmed | Raw `README.md` of sebbbi/OffsetAllocator at commit 3610a737: L66 "This allocator is similar to the two-level segregated fit (TLSF) algorithm." and L75 "MIT license (see file: LICENSE)". ADR 0017:17 "a TLSF-like allocator, MIT"; matches. |
| 120 | `offset-allocator-0.2.0-port` | full | confirmed | `~/.cargo/registry/src/*/offset-allocator-0.2.0/README.md`: "This is a port of [Sebastian Aaltonen's `OffsetAllocator`] package for C++ to 100% safe Rust." and "Licensed under the MIT license. See `LICENSE-MIT` for details."; `LICENSE-MIT` is in the crate. ADR 0017:17; matches. |
| 121 | `meshoptimizer-0.25-scan-meshlets` | full | confirmed | `meshopt-0.6.2/vendor/src/clusterizer.cpp` L360 `appendMeshlet`, L368-L370 the two quoted lines word for word; `meshopt_buildMeshletsScan` (L1264) calls it per triangle at L1289. Code site `src/cook/mesh.rs:158-162`: counts a degenerate triangle's repeated corner twice, which `(av < 0) + (bv < 0) + (cv < 0)` does when two indices alias one `used` slot; matches. |
| 122 | `meshoptimizer-0.25-cluster-bounds` | sample | confirmed | `meshopt-0.6.2/vendor/src/meshoptimizer.h` L721-L744, the `meshopt_computeClusterBounds` comment: every quoted fragment, the apex-free formula and the Real-Time Rendering reference are there word for word. Doc site `docs/performance.md:1851` names the apex-free test on the sphere. |
| 123 | `meshoptimizer-0.25-vertex-codec` | full | confirmed | `vertexcodec.cpp` L125 `static int gEncodeVertexVersion = 0;` and L1781 the `meshopt_encodeVertexBufferLevel(..., meshopt::kEncodeDefaultLevel, meshopt::gEncodeVertexVersion)` return. Code site `src/cook/mesh.rs:26-31`: "vertex codec 0" by default in the Cook key; matches. |
| 128 | `aras-2020-texture-compression` | sample | confirmed | The blog post, live: all seven quoted fragments present word for word (only markup differs: bold, code and italic spans, a curly apostrophe in "There's"). |
| 133 | `blake3-1.8.7-readme` | sample | confirmed | Raw `README.md` at tag 1.8.7, L5 and L8-L9: "Much faster than MD5, SHA-1, SHA-2, SHA-3, and BLAKE2." and "Highly parallelizable across any number of threads and SIMD lanes, because it's a Merkle tree on the inside." (bold markers dropped). |
| 138 | `wgpu-types-30-copy-row-alignment` | sample | confirmed | `wgpu-types-30.0.1/src/lib.rs` L107-L116: the doc comment and `pub const COPY_BYTES_PER_ROW_ALIGNMENT: u32 = 256;` word for word. |
| 139 | `wgpu-core-30-write-texture-staging` | full | confirmed | `wgpu-core-30.0.1/src/device/queue.rs`: `pub fn write_texture(` at L853; L994-L1013 the `if stage_bytes_per_row == bytes_per_row` branch and the `else` branch each call `StagingBuffer::new(&self.device, stage_size)?`. ADR 0017:19 "allocates fresh staging on every call"; matches. |
| 140 | `wgpu-30-staging-belt-chunk-size` | full | confirmed | `wgpu-30.0.1/src/util/belt.rs` L56-L57 ("The `chunk_size` is the unit of internal buffer allocation; writes will be sub-allocated within each chunk.") and L132-L133 ("If the `size` is greater than the size of any free internal buffer, a new buffer will be allocated for it."). ADR 0017:19 "only its allocation unit (it allocates more on demand)"; matches. |
| 141 | `wgpu-types-30-transient-saves-memory` | full | confirmed | `wgpu-types-30.0.1/src/adapter.rs` L212-L214: the two doc lines and `pub transient_saves_memory: Option<bool>,` word for word. ADR 0017:18 counts a `TRANSIENT_ATTACHMENT` as zero only when `Some(true)`; matches. |
| 142 | `wgpu-hal-30-metal-use-resource` | full | confirmed | `wgpu-hal-30.0.1/src/metal/command.rs`: `set_bind_group` at L1088; L1145-L1147 the comment "Call useResource on all textures and buffers used indirectly so they are alive", the `for (resource, use_info) in group.resources_to_use.iter()` loop and `encoder.useResource_usage_stages(`; the compute encoder's loop at L1176-L1181. ADR 0017:14 "Metal's per-texture `useResource` on each `set_bind_group`"; matches. |
| 147 | `kenney-support-license` | sample | confirmed | `kenney.nl/support`, question "Can I use the game assets in a (commercial) project?": "Yes, all game assets on the asset pages are public domain licensed (CC0). You're free to use them, even in commercial projects." (curly apostrophe in the source). |
| 150 | `ign-mdt05-metadata` | opened | confirmed | The live CSW record (curl, 200): `gmd:title` "Modelo Digital del Terreno con paso de malla de 5 metros (MDT05) de España" and `gmd:otherConstraints` "MDT05-cob1 2008-2015 CC-BY 4.0 scne.es", both word for word. |
| 151 | `ign-mdt05-inspire-record` | opened | confirmed | The archived INSPIRE geoportal page (curl, 200; WebFetch cannot reach web.archive.org): its "constraints" field reads "MDT05 CC BY 4.0 SCNE", word for word. |
| 152 | `ign-mdt05-distribution` | sample | unsupported | The live CSW record's `gmd:distributionInfo` `gmd:onLine` linkages are four: the WMS, WMTS and WCS GetCapabilities urls and `https://centrodedescargas.cnig.es/CentroDescargas/modelo-digital-terreno-mdt05-primera-cobertura`. The WCS url matches the quote; `https://centrodedescargas.cnig.es/CentroDescargas/busquedaSerie.do?codSerie=MDT05` is nowhere in the record, whose `gmd:dateStamp` is 2026-10-14, before the entry's `checked = 2026-10-26`. Finding L7-U01. |
| 157 | `gltf-asset-generator` | sample | confirmed | Raw `README.md` at commit 3d99767e: L1-L2 "What is this project?" and the C# Visual Studio sentence word for word; L7 "What does it currently cover?"; L11 Accessor Sparse, L13 Animation Node, L18 Buffer Interleaved, L20-L22 Compatibility, Instancing, Material, L29 Mesh Primitive Attribute, L34 Node Attribute, L36 Texture Sampler, each as a "Positive Tests" bullet. |
| 162 | `damagedhelmet-readme-legal` | sample | confirmed | Raw `Models/DamagedHelmet/README.md` at commit edc7c9e6, "Legal" L32-L40: © 2018 ctxwing, CC BY 4.0, "ctxwing for Rebuild and conversion to glTF"; © 2016 theblueturtle_, CC BY-NC 4.0, "theblueturtle_ for Earlier version of model"; word for word with the links dropped. |
| 167 | `flighthelmet-lens-textures` | sample | confirmed | The GitHub contents API for `Models/FlightHelmet/glTF` at commit edc7c9e6 lists exactly the three LensesMat PNGs named; the first 33 bytes of each raw file (HTTP 206) carry a PNG signature and an IHDR of 1024 x 1024, as quoted. |
| 172 | `bc7enc-rdo-size-reduction` | sample | confirmed | Raw `README.md` at commit b9438627, L5 and L7: every quoted fragment is there word for word, in order. |
| 177 | `oodle-data-compression-claims` | sample | confirmed | `radgametools.com/oodlecompressors.htm`, under "The right compressor for you": both sentences word for word ("decodes 10× faster", "3-5× faster than zlib, with way more compression"). |
| 182 | `basisu-2.50-xuastc` | sample | confirmed | Raw `README.md` at tag v2_50, L115: "XUASTC LDR 4x4-12x12 ("GPU Photo LDR/SDR"): Supercompressed ASTC with Weight Grid DCT, supporting all 14 standard ASTC block sizes" and "Bitrates range from approximately 0.3–5.7 bpp, depending on content, profile, block size, windowed RDO, and Weight Grid DCT quality settings." word for word (bold markers dropped). |
| 187 | `rtxntc-0.10-bitrate` | sample | confirmed | Raw `README.md` at tag v0.10.0-beta, L31: the BC7/BC5/BC7 clause and "For NTC we have found that many real-world texture bundles of this format can be compressed with results comparable to BCn (a PSNR of 40 to 50 dB) with a latent shape requiring about 5 bits/texel." word for word. |
| 192 | `rtxntc-0.10-licence` | sample | confirmed | Same file, L250-L252: "## License" then "[NVIDIA RTX SDKs LICENSE](LICENSE.txt)". |
| 197 | `fujieda-2024-ntbc-setup` | sample | confirmed | arXiv 2407.09543v2 (43 MB, past WebFetch's limit; downloaded with curl and read with pdftotext), page 6: "All the evaluations in this paper are performed on a single AMD Radeon™ RX 7900 XT GPU on a Windows machine." and "Our proposed method, NTBC, encodes block-compressed data in BC1 and BC4 formats in this research." |
| 202 | `meshoptimizer-1.3-licence` | sample | confirmed | Raw `LICENSE.md` at tag v1.3, L1: "MIT License". |
| 207 | `zstd-1.5.7-licence` | sample | confirmed | Raw `README.md` at tag v1.5.7, L230: "Zstandard is dual-licensed under [BSD](LICENSE) OR [GPLv2](COPYING)." under "## License" (L228). |
| 212 | `nvcomp-licence` | sample | confirmed | Raw `LICENSE` at commit f0f982e1, L1 and L3: "LICENSE AGREEMENT FOR NVIDIA SOFTWARE DEVELOPMENT KITS" and "governs your use of the NVIDIA nvCOMP software development kit". |
| 217 | `ispc-texture-compressor-licence` | sample | confirmed | Raw `license.txt` at commit 79ddbc90, L1-L11: "Copyright 2017 Intel Corporation", the MIT grant and "The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software." word for word. |


## Registry entries 222 to 272

46 entries in this slice (indices 222 and 227 to 272; 223 to 226 are `skip` entries of the `crate` family and were not opened): 44 `full`, 2 `sample` (222, 264). Both samples are `confirmed`, so no `skip` entry was opened. 0 `unopened`. 46 `confirmed`, 0 Findings. Every wgpu and bevy entry was read in the crate source under `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` at the version `Cargo.lock` pins (wgpu-types, wgpu-core, wgpu-hal 30.0.1; bevy_ecs, bevy_tasks 0.18.1), where every cited line number matched the quoted text; Mesa, Vulkan-Docs, MicrosoftDocs, DirectX-Specs and the bevy_ecs manifest were read as raw files at their pinned tag or commit; the two Apple PDFs were downloaded from the archived urls and read with pdftotext at the cited page; crates.io, Apple's NSView page (its documentation JSON) and gpuinfo.org were fetched live. Each `full` entry's `src/` code site was opened and matches its quote.

### Verdicts

| index | key | check | verdict | what was opened, and what it says |
|---|---|---|---|---|
| 222 | `crate-ruzstd-0.9.1-licence` | sample | confirmed | crates.io API `crates/ruzstd/0.9.1`, `version.license` is `"MIT"`. |
| 227 | `wgpu-types-30-ray-query-experimental` | full | confirmed | wgpu-types-30.0.1 `src/features.rs` L1049-L1060: the doc comment and `const EXPERIMENTAL_RAY_QUERY = 1 << 32;` word for word. Cited only from docs (ADRs 0007, 0021), no `src/` site. |
| 228 | `wgpu-types-30-experimental-features-unsafe` | full | confirmed | wgpu-types-30.0.1 `src/tokens.rs` L29-L37: the Safety list and `pub const unsafe fn enabled() -> Self {` word for word. Code site `src/gpu/mod.rs:71-77`: the SAFETY comment says enabling accepts that a wgpu bug may cause UB from safe calls and calls `unsafe { wgpu::ExperimentalFeatures::enabled() }`; matches. |
| 229 | `wgpu-core-30-experimental-features-gate` | full | confirmed | wgpu-core-30.0.1 `src/instance.rs` L907-L916: the check and `return Err(RequestDeviceError::ExperimentalFeaturesNotEnabled(` word for word. Docs-only citation. |
| 230 | `wgpu-core-30-anisotropy-clamp` | full | confirmed | wgpu-core-30.0.1 `src/device/resource.rs` L2318-L2328: `anisotropy_clamp` is `desc.anisotropy_clamp.min(16)` with the flag, `1` without. Code site `src/gpu/mod.rs:277-281`: "without the flag, wgpu silently clamps every sampler's anisotropy to 1" and requires `ANISOTROPIC_FILTERING`; matches. |
| 231 | `wgpu-core-30-vertex-storage-required` | full | confirmed | wgpu-core-30.0.1 `src/device/resource.rs` L2907-L2917: a vertex-visible storage buffer entry adds `DownlevelFlags::VERTEX_STORAGE` to the required flags. Docs-only citation. |
| 232 | `wgpu-core-30-compute-required` | full | confirmed | wgpu-core-30.0.1 `src/device/resource.rs` L4036-L4042: `create_compute_pipeline_inner` calls `require_downlevel_flags(COMPUTE_SHADERS)`. Docs-only citation. |
| 233 | `wgpu-core-30-indirect-required` | full | confirmed | wgpu-core-30.0.1 `src/device/resource.rs` L1056-L1057: an `INDIRECT` buffer requires `INDIRECT_EXECUTION`. Docs-only citation. |
| 234 | `wgpu-types-30-downlevel-default-all` | full | confirmed | wgpu-types-30.0.1 `src/limits.rs` L1063-L1071: `Default for DownlevelCapabilities` sets `flags: DownlevelFlags::all()`. Docs-only citation. |
| 235 | `wgpu-hal-30-dx12-downlevel-default` | full | confirmed | wgpu-hal-30.0.1 `src/dx12/adapter.rs` L741: `let downlevel = wgt::DownlevelCapabilities::default();`. Docs-only citation. |
| 236 | `wgpu-hal-30-vulkan-downlevel-flags` | full | confirmed | wgpu-hal-30.0.1 `src/vulkan/adapter.rs` L684-L707: the `dl_flags` list with `COMPUTE_SHADERS`, `VERTEX_STORAGE`, `INDIRECT_EXECUTION`, the `SURFACE_VIEW_FORMATS` set from `swapchain_mutable_format` or no `khr::swapchain`, and the anisotropy set. Code site `src/gpu/mod.rs:446-449`: "Vulkan reports [SURFACE_VIEW_FORMATS] only with `VK_KHR_swapchain_mutable_format`"; true for any adapter that has a swapchain, which a surface implies (see Notes). |
| 237 | `wgpu-hal-30-metal-downlevel-flags` | full | confirmed | wgpu-hal-30.0.1 `src/metal/adapter.rs` L1333-L1359: default (all) flags, `INDIRECT_EXECUTION` from `indirect_draw_dispatch`, `ANISOTROPIC_FILTERING` set true. Code site `src/gpu/mod.rs:577-584`: Metal reports anisotropy, vertex storage and compute on every adapter and indirect execution on the M4 Pro's family; matches (the first three come from the default or the unconditional set). |
| 238 | `wgpu-hal-30-metal-indirect-first-instance` | full | confirmed | wgpu-hal-30.0.1 `src/metal/adapter.rs` L1202: `features.set(F::INDIRECT_FIRST_INSTANCE, self.indirect_draw_dispatch);`. Docs-only citation. |
| 239 | `wgpu-core-30-resolve-format` | full | confirmed | wgpu-core-30.0.1 `src/command/render.rs` L1583-L1592: a resolve view without `MULTISAMPLE_RESOLVE` returns `UnsupportedResolveTargetFormat`. Docs-only citation. |
| 240 | `wgpu-hal-30-metal-timestamp-inside` | full | confirmed | wgpu-hal-30.0.1 `src/metal/adapter.rs` L1208-L1217 word for word; `INSIDE_WGPU_PASSES` is defined at `src/metal/mod.rs` L224 as the section says. Docs-only citation. |
| 241 | `wgpu-types-30-acceleration-structure-minimums` | full | confirmed | wgpu-types-30.0.1 `src/limits.rs` L774-L786: the doc line and the five limit assignments with their comments word for word. Docs-only citation; `src/gpu/mod.rs:307` applies `using_minimum_supported_acceleration_structure_values()`. |
| 242 | `wgpu-hal-30-metal-buffer-slots` | full | confirmed | wgpu-hal-30.0.1 `src/metal/adapter.rs` L1376-L1396: the two comments, `MAX_USABLE_BUFFERS = MAX_BUFFERS - 2` and the three assignments; `MAX_BUFFERS: u32 = 31` at L54. Docs-only citation. |
| 243 | `wgpu-hal-30-dx12-binding-limits` | full | confirmed | wgpu-hal-30.0.1 `src/dx12/adapter.rs` L773-L825: `full_heap_count` (1_000_000 for T1/T2, `1 << 20` for T3), `ResourceBindingTier::T3 => full_heap_count,` for UAVs, `MAX_SHADER_STAGES_PER_PIPELINE = 2` and the two halvings. Docs-only citation; ADR 0021's 2^18 storage buffers at Tier 3 follows (2^20 / 2 / 2). |
| 244 | `wgpu-hal-30-vulkan-storage-buffers-halved` | full | confirmed | wgpu-hal-30.0.1 `src/vulkan/adapter.rs` L1534-L1553: the constant and `max_storage_buffers_per_shader_stage = ...min(max_descriptor_set_storage_buffers / MAX_SHADER_STAGES_PER_PIPELINE)`. Docs-only citation. |
| 245 | `wgpu-types-30-default-storage-binding-size` | full | confirmed | wgpu-types-30.0.1 `src/limits.rs` L421 and L440: `max_storage_buffer_binding_size: 128 << 20, // (128 MiB)`. Docs-only citation; `src/gpu/mod.rs:296-307` leaves this limit at `Limits::default()`. |
| 246 | `wgpu-types-30-default-workgroups-per-dimension` | full | confirmed | wgpu-types-30.0.1 `src/limits.rs` L421 and L456: `max_compute_workgroups_per_dimension: 65535,`. Code site `src/render/culling_pass.rs:48-52`: `MAX_WORKGROUPS_PER_DIMENSION: u32 = 65_535`, "wgpu's default"; matches. |
| 247 | `wgpu-core-30-dispatch-usage-scope` | full | confirmed | wgpu-core-30.0.1 `src/command/compute.rs` L344-L356 (inside `flush_bindings`, L340-L357): bind groups and the indirect buffer merge into one per-dispatch usage scope, word for word. Code site `src/render/culling_pass.rs:145-149`: a dispatch cannot take its arguments from a buffer its bindings write, "as they and its indirect buffer are one usage scope", hence `frame_draw_stand_in`; matches. |
| 248 | `wgpu-core-30-backend-order` | full | confirmed | wgpu-core-30.0.1 `src/instance.rs` L83-L88: Vulkan, Metal, Dx12 in that order, word for word. Code site `src/gpu/mod.rs:472-483`: `BACKEND_ORDER` is `[Vulkan, Metal, Dx12]`, "Vulkan leads, as in wgpu's own backend order"; matches. |
| 249 | `wgpu-hal-30-allocator-report-default` | full | confirmed | wgpu-hal-30.0.1 `src/lib.rs` L1210-L1212: the default `generate_allocator_report` returns `None`; overridden in `src/dx12/device.rs:2643` and `src/vulkan/device.rs:2954` only (`src/dynamic/device.rs:615` is the `DynDevice` forwarder, not a backend); identical in wgpu-hal-29.0.1 `src/lib.rs` L1164-L1166 as the section says. Code site `src/stats/mod.rs:53-55`: "wgpu's allocator report, a diagnostic only; Metal has none", `Option<u64>`; matches. |
| 250 | `mesa-25.2.8-lvp-max-storage-buffers` | full | confirmed | Raw `mesa-25.2.8` `lvp_device.c` L905: `.maxPerStageDescriptorStorageBuffers      = MAX_DESCRIPTORS,`. Docs-only citation. |
| 251 | `mesa-25.2.8-lvp-max-descriptors` | full | confirmed | Raw `mesa-25.2.8` `lvp_private.h` L102: `#define MAX_DESCRIPTORS 1000000 /* Required by vkd3d-proton */`. Docs-only citation. |
| 252 | `mesa-25.2.8-lvp-max-storage-buffer-range` | full | confirmed | Raw `mesa-25.2.8` `lvp_device.c` L896: `.maxStorageBufferRange                    = device->pscreen->caps.max_shader_buffer_size,`. Docs-only citation. |
| 253 | `mesa-25.2.8-llvmpipe-max-shader-buffer-size` | full | confirmed | Raw `mesa-25.2.8` `lp_screen.c` L352: `caps->max_shader_buffer_size = LP_MAX_TGSI_SHADER_BUFFER_SIZE;`. Docs-only citation. |
| 254 | `mesa-25.2.8-gallivm-max-shader-buffer-size` | full | confirmed | Raw `mesa-25.2.8` `lp_bld_limits.h` L57: `#define LP_MAX_TGSI_SHADER_BUFFER_SIZE (1 << 27)`, 128 MiB as the section says. Code site `src/gpu/mod.rs:752-760`: the test says the floor holds `max_storage_buffer_binding_size` at the default 128 MiB, lavapipe's `maxStorageBufferRange`, and asserts the Arena fits it; `floor_limits` (L296-L307) leaves that limit at the default; matches. |
| 255 | `mesa-25.2.8-features-lavapipe-ray-query` | full | confirmed | Raw `mesa-25.2.8` `docs/features.txt` L463, L480, L534, L540, L562: the five `DONE (...)` lines with `lvp` in each, word for word. Docs-only citation. |
| 256 | `vulkan-max-storage-buffer-range` | full | confirmed | Raw Vulkan-Docs at `8c9361ba` `chapters/limits.adoc` L6847: `| pname:maxStorageBufferRange | - | 2^27^ | min` (row written "cell: cell" in the quote). Code site `src/gpu/mod.rs:752-760`: "the default 128 MiB: Vulkan's minimum"; 2^27 bytes is 128 MiB; matches. |
| 257 | `vulkan-update-after-bind-minimums` | full | confirmed | Raw Vulkan-Docs at `8c9361ba` `limits.adoc` L7109-L7126: the six 500000 ^9^ rows quoted, and footnote 9 at L7598-L7602 word for word. Docs-only citation. |
| 258 | `vulkan-max-acceleration-structures-per-stage` | full | confirmed | Raw Vulkan-Docs at `8c9361ba` `limits.adoc` L7237: `| pname:maxPerStageDescriptorAccelerationStructures | - | 16 | min`. Docs-only citation. |
| 259 | `d3d12-resource-binding-tiers` | full | confirmed | Raw MicrosoftDocs/win32 at `ca8274de` `hardware-support.md` L21-L27: the three rows quoted, cells 1,000,000 / 1,000,000 / 1,000,000+; 14 / 14 / full heap; 128 / full heap / full heap (bold markers dropped, allowed). Docs-only citation. |
| 260 | `metal-feature-set-tables-buffer-argument-table` | full | confirmed | Archived `Metal-Feature-Set-Tables.pdf` (20260924114533), dated May 21, 2026, 18 pages; p. 7 "GPU implementation limits by family", "Function arguments": "Maximum number of entries in the buffer argument table, per graphics or kernel function 2", 31 in all nine columns (GPU family 1, Apple2 to Apple10). Docs-only citation. |
| 261 | `metal-feature-set-tables-sample-buffers` | full | confirmed | Same PDF, p. 8: "Maximum counter sample buffer length 32 KB" and "Maximum number of sample buffers 32" in all nine columns. Code site `src/render/readback.rs:76-79`: "Apple GPUs allow 32 counter sample buffers, and on Metal each set is one", so one `QuerySet` holds every slot's queries; matches. |
| 262 | `msl-4.1-no-double` | full | confirmed | Archived `Metal-Shading-Language-Specification.pdf` (20260925145438), "Version 4.1", footer "2026-06-04"; p. 25, "2.1 Scalar Data Types": "Metal supports the scalar types listed in Table 2.1. Metal does not support the double, long long, unsigned long long, and long double data types." word for word. Docs-only citation (ADR 0014). |
| 263 | `apple-nsview-main-actor` | full | confirmed | Apple's documentation data for `appkit/nsview` (`tutorials/data/documentation/appkit/nsview.json`, the JSON the cited page renders): declaration tokens `@MainActor class NSView`, platform macOS. Docs-only citation (ADR 0016). |
| 264 | `d3d12-feature-level-12-2` | sample | confirmed | Raw DirectX-Specs at `f2917db8` `D3D12_FeatureLevel12_2.md` L39-L72: "If a device is feature level 12_2, it has", Shader Model 6.5, Raytracing tier Tier 1.1, Mesh shader tier Tier 1, Resource Binding Tier Tier 3, "Additionally, it has the following flags set", WaveOps TRUE, Int64ShaderOps TRUE. |
| 265 | `gpuinfo-vulkan-ray-query-coverage` | full | confirmed | Live `vulkan.gpuinfo.org/listextensions.php?platform=windows&age=recent`: heading "Extension coverage for Windows", "Age: Recent (1y)"; rows exist for both extensions. Today (2026-11-02) they read `VK_EXT_mesh_shader` 48.97 % and `VK_KHR_ray_query` 47.29 %, against the quote's 49.35 % and 47.66 % read 2026-10-27; the section says the figures move and are read on the date checked. Docs-only citation (ADR 0021 L19). |
| 266 | `wgpu-hal-30-metal-acquire-window` | full | confirmed | wgpu-hal-30.0.1 `src/metal/surface.rs` L354-L375: the macOS block, `hosting_window(Retained::into_super(render_layer.clone()))`, the `occlusionState` send; `hosting_window` at L24-L35 with the quoted `layer.delegate()` branch, its doc comment ("Walks up from `start` to the `NSWindow` hosting this layer ...") at L20-L21. Docs-only citation (ADR 0016). |
| 267 | `bevy-tasks-0.18.1-default-threads` | full | confirmed | bevy_tasks-0.18.1 `src/task_pool.rs` L165-L167 word for word; `Default for TaskPool` calls `Self::new()` at L603-L606; `available_parallelism` at `src/lib.rs` L167-L172 wraps `std::thread::available_parallelism()`. Docs-only citation (ADR 0016). |
| 268 | `bevy-ecs-0.18.1-executor-default` | full | confirmed | bevy_ecs-0.18.1 `src/schedule/executor/mod.rs` L45-L66: the doc lines and both `cfg_attr(..., default)` attributes as quoted. Docs-only citation (ADR 0016). |
| 269 | `bevy-ecs-0.18.1-par-iter-multi-threaded` | full | confirmed | bevy_ecs-0.18.1 `src/query/par_iter.rs` L86-L104: the serial `fold(init, func)` under `not(feature = "multi_threaded")` and the `ComputeTaskPool::get().thread_num()` branch under it. Docs-only citation (ADR 0016). |
| 270 | `bevy-tasks-0.18.1-multi-threaded-pool` | full | confirmed | bevy_tasks-0.18.1 `src/lib.rs` L103-L115 word for word; GitHub `v0.18.1` `crates/bevy_ecs/Cargo.toml` L20 is `multi_threaded = ["bevy_tasks/multi_threaded", "dep:arrayvec"]` as the section says (the crates.io copy of the manifest is normalised and puts it at L71-L72). Docs-only citation (ADR 0016). |
| 271 | `wgpu-core-30-map-after-last-submission` | full | confirmed | wgpu-core-30.0.1 `src/device/life.rs` L163-L164 word for word; `LifetimeTracker::map` at L236-L250 finds the newest active submission containing the buffer (`.iter_mut().rev().find(...)`) as the section says. Docs-only citation (ADR 0016). |
| 272 | `wgpu-core-30-submit-mapped-buffer` | full | confirmed | wgpu-core-30.0.1 `src/device/queue.rs` L2072-L2075: the `map_state` match returning `BufferStillMapped`; the error text `"{0} is still mapped"` at L564-L565. Docs-only citation (ADR 0016). |


## Registry entries 273 to 451

The slice holds 47 entries: 14 `full` and 33 `sample`. One sample, `gpuinfo-memory-extensions-windows`, is `stale`, which opened the five `skip` entries of the `gpuinfo` family (398, 399, 400, 401, 403), listed below with check `opened`: 52 rows in all. None is `unopened`: the fifteen wgpu, wgpu-core, wgpu-hal and naga entries were read in the crates under `~/.cargo/registry/src/` at 29.0.1 and 30.0.1 (the CHANGELOG, which the crate omits, as the raw file at tag v30.0.1), the GitHub entries as raw files at their pinned commit or tag, the PDFs downloaded and read at the cited page, the Apple pages through their documentation JSON, the DOI through the Crossref API, the wgpu issue through the GitHub API, and the rest as fetched HTML. Verdicts: 47 `confirmed`, 5 `stale`. Findings: 6, one of them the one entry in the whole registry without a `checked` field.

### Verdicts

| index | key | check | verdict | what was opened, and what it says |
|---|---|---|---|---|
| 273 | `wgpu-30-write-buffer-before-submit` | full | confirmed | `wgpu-30.0.1/src/api/queue.rs` L143-L158: the three quoted sentences are there word for word, and L245-L247 under `write_texture` says it "has the same performance considerations as `Queue::write_buffer()`". Cited at `docs/adr/0016-the-main-thread-runs-the-frame.md:18`, which says `Queue::write_*` "stages it and orders the copy before the next submit": matches. |
| 274 | `wgpu-types-30-frame-latency-two` | full | confirmed | `wgpu-types-30.0.1/src/surface.rs` L947-L948: the "Choose `2`" bullet matches word for word; `wgpu-30.0.1/src/api/surface.rs` L96 sets `desired_maximum_frame_latency: 2` in `get_default_config`. ADR 0016 L18 says it "stays 2"; `src/lib.rs:165` passes `render::FRAMES_IN_FLIGHT`, which `src/render/mod.rs:43` sets to 2: matches. |
| 275 | `mlua-0.12.2-send-feature` | full | confirmed | Raw `mlua-rs/mlua` README at v0.12.2, L52: the `send` feature line matches word for word. ADR 0016 L19 says the feature is off and so no `Send + Sync` bound on userdata: consistent with the quote. |
| 276 | `mlua-0.12.2-send-lock` | full | confirmed | Raw `src/types/sync.rs` at v0.12.2 L1-L36: L9 aliases parking_lot's `ReentrantMutex` under `send`, L34-L35 is the no-op `lock` without it, as the section says. ADR 0016 L19 "no lock per call": matches. |
| 277 | `mlua-0.12.2-luajit` | full | confirmed | README at v0.12.2 L23 lists Lua 5.5, 5.4, 5.3, 5.2, 5.1 (including LuaJIT) and Luau; L44 is the `luajit` feature line. `ROADMAP.md:67` and `docs/adr/0012-lua-reacts-to-events.md:3` ("with LuaJIT where supported"): match. |
| 278 | `wgpu-issue-2869` | full | confirmed | GitHub API: issue 2869 "Extended Presentation API Investigation", state open; the body's first sentence and the five primitives match word for word ("primatives" is the source's spelling). ADR 0022 L15 waits on it for present timing: matches. |
| 279 | `box2d-3.1.1-fixed-step` | full | confirmed | Raw `docs/hello.md` at v3.1.1 L176-L178: the three sentences match word for word (L178 continues "Without further ado,", outside the quote). ADR 0022 L7 "Box2D and PhysX advise against variable steps": matches. |
| 280 | `physx-5.6.1-variable-step` | full | confirmed | `nvidia-omniverse.github.io/PhysX/physx/5.6.1/docs/Simulation.html`, The Simulation Loop: the note's "Be very careful if you do this [...] jittery simulation." matches word for word. ADR 0022 L7: matches. |
| 281 | `rust-1.96-f64-sin-precision` | full | confirmed | `doc.rust-lang.org/1.96.0/std/primitive.f64.html#method.sin`, "Unspecified precision": the two sentences match word for word, and `cos` and `tan` carry the same text. ADR 0022 L20 "`sin` and its kin are not bit-stable": matches. |
| 283 | `apple-mtldevice-current-allocated-size` | full | confirmed | Apple documentation JSON for `MTLDevice.currentAllocatedSize`: the abstract matches word for word; platforms list macOS 10.13+. ADR 0013 L3 calls it a diagnostic only, which no code implements: consistent. The entry has no `checked` field, the only such entry of 581 (Finding L7-W01). |
| 287 | `wgpu-29-vulkan-texture-memory-external` | sample | confirmed | `wgpu-hal-29.0.1/src/vulkan/mod.rs` L713-L722: `pub enum TextureMemory` with the `// memory not owned by wgpu` / `External,` arm: matches. |
| 292 | `wgpu-29-metal-ignores-memory-hints` | sample | confirmed | `wgpu-hal-29.0.1/src/metal/adapter.rs` L63: `_memory_hints: &wgt::MemoryHints,` in `Adapter::open`'s signature: matches. |
| 297 | `wgpu-29-vulkan-bind-group-write` | sample | confirmed | `wgpu-hal-29.0.1/src/vulkan/device.rs` L1708: the `update_descriptor_sets(&writes, &[])` line matches; `create_bind_group` starts at L1500 and a grep of `src/vulkan/*.rs` finds that one call only, as the section says. |
| 302 | `wgpu-29-hal-open-device-one-queue` | sample | confirmed | `wgpu-hal-29.0.1/src/lib.rs` L2014-L2017: `pub struct OpenDevice<A: Api>` with `device` and `queue` fields: matches. |
| 307 | `wgpu-29-vulkan-raw-queue` | sample | confirmed | `wgpu-hal-29.0.1/src/vulkan/device.rs` L767: `pub fn raw_queue(&self) -> vk::Queue {`: matches. |
| 312 | `wgpu-core-29-user-memory-flags-empty` | sample | confirmed | `wgpu-core-29.0.1/src/device/resource.rs` L1093-L1098 (`hal::BufferDescriptor` with `memory_flags: hal::MemoryFlags::empty()`) and L1618-L1626 (`hal::TextureDescriptor`, same flag): match. |
| 317 | `naga-29-sample-level-image-query` | sample | confirmed | `naga-29.0.1/src/ir/mod.rs` L1459-L1493: `SampleLevel` with Auto, Zero, Exact, Bias, Gradient{x,y} and `ImageQuery` with Size{level}, NumLevels, NumLayers, NumSamples, doc comments elided as the section says: match. |
| 322 | `wgpu-core-29-write-staging` | sample | confirmed | `wgpu-core-29.0.1/src/device/queue.rs` L545 (`write_buffer`), L573-L582 (`create_staging_buffer`), L880 and L889 (both `write_texture` paths): each `StagingBuffer::new` call matches. |
| 327 | `wgpu-29-map-async` | sample | confirmed | `wgpu-29.0.1/src/api/buffer.rs` L361-L371: the three quoted passages of the `map_async` docs match word for word. |
| 332 | `wgpu-30-changelog-from-hal-initial-state` | sample | confirmed | Raw `CHANGELOG.md` at tag v30.0.1, L191: the `create_texture_from_hal` `initial_state` bullet matches word for word. |
| 337 | `barb-2017-tf2-probes` | sample | confirmed | Barb PDF p. 10, "How do we Choose Probes?": "Chop geometry into 16ft² columns" and "Use k-means to combine into max 8 probes per column": match. |
| 342 | `barb-2017-tf2-streaming-budget` | sample | confirmed | Barb PDF p. 27, "How Much RAM do we Need?": the ~600MiB / ~1000MiB bullet and "Plus permanent MIPs (~400MiB on typical level)": match. |
| 347 | `vanwaveren-2012-svt-storage` | sample | confirmed | van Waveren PDF p. 24, §6.3 and §6.4: the 0.5 ms at 80x60, the 128 x 128 pages with 120 x 120 payload, and the three 4096 x 4096 textures in 40 MB: match word for word. |
| 352 | `nanite-prior-work` | sample | confirmed | Karis PDF p. 44, "LOD crack options", bullets Quick-VDR and Batched Multi-Triangulation; speaker notes: "This is the direction I took and the prior work we primarily build upon.": match. |
| 357 | `nanite-resident-hierarchy` | sample | confirmed | Karis PDF p. 126, "Deciding what to stream": bullet "Full culling hierarchy always resident": matches. |
| 362 | `dgf-2024-block` | sample | confirmed | DGF PDF p. 3 (printed 1:3), §3.2 Format Description: the two sentences match word for word; footer "Proc. ACM Comput. Graph. Interact. Tech., Vol. 7, No. 3 [...] July 2024" matches the section. |
| 367 | `odonnell-2017-framegraph-aliasing-barriers` | sample | confirmed | O'Donnell PDF p. 50 (printed 48), speaker notes: both sentences match word for word. |
| 372 | `odonnell-2017-framegraph-4k-non-aliasing` | sample | confirmed | O'Donnell PDF p. 59 (printed 57): the slide image reads "Non-aliasing memory layout (4K, DX12 PC)" and "1042 MB total": matches the transcription. |
| 377 | `intel-sfs-how-it-works` | sample | confirmed | Raw README at 714e144, L175-L183: the four quoted passages match; the quote writes the markdown link targets as `(...)`, the words are unchanged. |
| 382 | `d3d12-sampler-feedback-minmip` | sample | confirmed | Raw `d3d/SamplerFeedback.md` at a610933, L105-L107 inside "How to adopt Sampler Feedback for Streaming" (L101-L112): the three bullets match word for word. |
| 387 | `directstorage-1.1-preview-sample` | sample | confirmed | `devblogs.microsoft.com/directx/directstorage-1-1-coming-soon/`: the sample caption matches word for word. |
| 392 | `vulkan-sparse-bind-queue` | sample | confirmed | Raw `chapters/sparsemem.adoc` at 193ebe1, L1119-L1121: the sentence matches word for word, asciidoc markup included. |
| 397 | `vulkan-ext-memory-decompression-gdeflate` | sample | confirmed | Raw `appendices/VK_EXT_memory_decompression.adoc` at 193ebe1, L32-L33 inside Issues 1 (L28-L38): matches; L10 "Last Modified Date 2025-01-23" matches the section. |
| 402 | `gpuinfo-memory-extensions-windows` | sample | stale | `vulkan.gpuinfo.org/listextensions.php?platform=windows` today (Recent (1y) is the page's default and selected filter): budget 91.03 % as quoted, but priority 78.32 % (quoted 78.13), pageable 49.35 % (49.72), NV decompression 31.59 % (31.78), EXT decompression 21.31 % (21.5). A live database; the quote was its 2026-10-27 reading (Finding L7-W02). |
| 407 | `offset-allocator-bins` | sample | confirmed | Raw README at 3610a73, L4: both quoted passages match word for word; LICENSE L3 "Copyright (c) 2023 Sebastian Aaltonen" matches the section. |
| 412 | `metal-feature-set-tables-sparse-textures` | sample | confirmed | Archived Metal Feature Set Tables PDF (footer May 21, 2026), p. 4 row "Sparse textures ⁴: Metal 3 & 4: Apple6" and p. 6 note 4 "GPU devices starting with the Apple6 family support sparse color textures with automatic heap backing.": match. |
| 417 | `apple-metal-sparse-sample` | sample | confirmed | Apple documentation JSON, "Reading and writing to sparse textures", heading "Test for unmapped regions in your shader": both quoted sentences match word for word. |
| 422 | `apple-metal-heap-hazard-tracking` | sample | confirmed | Apple documentation JSON for `MTLHeapDescriptor.hazardTrackingMode`, Discussion: the two sentences match, the inline references resolving to `MTLHazardTrackingMode.default` and `.untracked`. |
| 427 | `ue-5.6-texture-streaming-pool` | sample | confirmed | web.archive.org snapshot 20250810013810 of the Unreal texture streaming page (labelled Unreal Engine 5.6): the `r.Streaming.PoolSize` and `r.Streaming.Boost` descriptions match word for word. |
| 432 | `bevy-0.20-meshlet-buffer-growth` | sample | confirmed | Raw `persistent_buffer.rs` at v0.20.0, L41-L53: `queue_write`'s signature, the `allocate_range` branch, `double_buffer_size`, `new_size` and `grow_to(buffer_size.end + new_size)` match; L19 declares `allocation_planner: RangeAllocator<BufferAddress>`. |
| 437 | `tanner-1998-clipmap` | sample | confirmed | Crossref API for 10.1145/280814.280855: title "The clipmap", container "Proceedings of the 25th annual conference on Computer graphics and interactive techniques - SIGGRAPH '98", pages 151-158, authors Tanner, Migdal, Jones: match. |
| 442 | `barrett-svt-public-domain` | sample | confirmed | `silverspaceship.com/src/svt/`, Downloads: "(This is essentially the same as my GDC 2008 presentation, although I only took 0:50 at GDC.)" and "All the source code is in the public domain.": match. |
| 445 | `fatahalian-2010-quad-fragment-merging` | full | confirmed | `microsoft.com/en-us/research/?p=159994`, abstract: the sentence matches word for word; the page lists the seven authors and ACM Trans. Graph. 29(4). Cited at `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:16` as what the observed merging "resembles", no stronger claim: matches. |
| 446 | `wgpu-30-tlas-dependency-per-instance` | full | confirmed | `wgpu-core-30.0.1/src/command/ray_tracing.rs` L269-L271: `instance_count += 1;` then `dependencies.push(blas.clone());` inside the per-instance loop: matches. Code site `src/render/bind_groups.rs:98-105` says wgpu-core "walks a TLAS's dependencies, one BLAS per Instance it holds": matches. |
| 448 | `wgpu-30-use-tlas-per-flush` | full | confirmed | `wgpu-core-30.0.1/src/command/pass.rs` L144-L174: `flush_bindings_helper` maps each flushed bind group's `used.acceleration_structures` to `AsAction::UseTlas` and extends `as_actions`: matches. `src/render/bind_groups.rs:102-103` "for each flush of a group holding it, at least once per pass that binds one": matches. |
| 450 | `wgpu-30-submit-tlas-dependencies` | full | confirmed | `wgpu-core-30.0.1/src/command/ray_tracing.rs` L570-L577: the `AsAction::UseTlas` arm pushes every dependency: matches word for word. `src/render/bind_groups.rs:101` "at submit": matches. |
| 451 | `wgpu-hal-30-metal-residency-per-submit` | sample | confirmed | `wgpu-hal-30.0.1/src/metal/command.rs` L1896-L1917: `set_acceleration_structure_dependencies` creates one residency set with `newResidencySetWithDescriptor_error`, one `addAllocation` per dependency, then `commit()`: matches. |
| 398 | `gpuinfo-sparse-residency-image2d-windows` | opened | stale | `listfeaturescore10.php?platform=windows` today, Recent (1y) selected: `sparseResidencyImage2D 77.01 % 22.99 %`; quoted 76.64 % / 23.36 % (Finding L7-W03). |
| 399 | `gpuinfo-sparse-residency-image2d-linux` | opened | stale | `listfeaturescore10.php?platform=linux` today: `sparseResidencyImage2D 71.92 % 28.08 %`; quoted 72.11 % / 27.89 % (Finding L7-W04). |
| 400 | `gpuinfo-lazily-allocated-windows` | opened | confirmed | `listmemory.php?platform=windows` today, Recent (1y) selected: `DEVICE_LOCAL_BIT LAZILY_ALLOCATED_BIT 5.05 % 94.95 %`: matches today. |
| 401 | `gpuinfo-lazily-allocated-linux` | opened | stale | `listmemory.php?platform=linux` today: `DEVICE_LOCAL_BIT LAZILY_ALLOCATED_BIT 3.88 % 96.12 %`; quoted 3.85 % / 96.15 % (Finding L7-W05). |
| 403 | `gpuinfo-memory-extensions-linux` | opened | stale | `listextensions.php?platform=linux` today, Recent (1y) selected: budget 93.61 % (quoted 93.65), priority 55.25 % (55.56), pageable 26.71 % (27.21), NV decompression 21.69 % (21.54), EXT decompression 11.42 % (11.34) (Finding L7-W06). |


## Registry entries 453 to 581

42 entries: 21 `full` (cited from `src/`, `scripts/` or `docs/adr/`) and 21 `sample`. None opened by a wrong sample: the one entry not confirmed (`apple-tech-talk-110147-busy-waits`, index 535) is a `full` entry whose quote is verbatim but whose `section` field misdates the talk, so no family was opened. No entry is `unopened`: the nine wgpu and gpu-allocator entries were read in the crates `Cargo.lock` pins under `~/.cargo/registry/src/`; the PDFs WebFetch could not read as text (binary or above its size limit) were downloaded and read with `pdftotext` or rendered page by page; the two Springer links that answer with a login redirect were read in the editors' free Ray Tracing Gems v1.9 PDF at the cited section and page; the Apple MTLFence page, which renders by script only, was read through Apple's documentation data endpoint for the same page; the Heckbert thesis, a scan with no text layer, was read as a page image.

### Verdicts

| index | key | check | verdict | what was opened, and what it says |
|---|---|---|---|---|
| 453 | `lauritzen-2010-edge-detection` | full | confirmed | Intel PDF, slides 19 "Anti-aliasing" and 20 "Identifying Edges" (pdftotext): every quoted bullet verbatim. `src/render/shaders/shadow_mask.wgsl:405-410` says the second samples are found as deferred renderers find per-sample pixels, comparing each sample's depth with the nearest sample's plane: the slides' "Compare sample depths to depth derivatives"; matches. |
| 454 | `salvi-2012-sbaa-surfaces` | full | confirmed | vidimce.org PDF §3.1 (pdftotext lines 154-182): the three quoted passages verbatim, "Sᵢ" for the paper's Si. `src/render/targets.rs:28-32` (two surfaces per pixel, each shaded once) and `src/render/shaders/main.wgsl:219-223` (a fragment takes its surface's word) claim what the quote says; `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:17` the same. |
| 455 | `lauritzen-2010-per-sample-scheduling` | full | confirmed | Intel PDF, slides 23 "Per-Sample Scheduling" and 24 "MSAA with Tile-Based Methods": every quoted bullet verbatim. `src/render/shaders/shadow_mask.wgsl:381-386` and `:592-596` describe a packed list of pixels traced apart from the tile's first pass; matches "Pack a list of pixels that require per-sample shading". |
| 459 | `crassin-2015-agaa-shadowing` | sample | confirmed | NVIDIA authors' PDF, page 5, heading "3.5.2 Shadowing" (downloaded, pdftotext): both quoted passages verbatim, including "especially when many lights need to be evaluated". |
| 464 | `jimenez-2012-smaa-1x-subpixel` | sample | confirmed | iryoku.com PDF (pdftotext): "MLAA algorithms work with a single sample per pixel. This translates into subsampling, which makes it impossible to recover real subpixel features" on page 6, and the "Post-processing techniques using 1x (final display resolution) color inputs" sentence on page 8; verbatim. |
| 469 | `marrs-2019-ataa-subpixel-miss` | sample | confirmed | The Springer PDF and chapter page both answer 303 to idp.springer.com (a cookie login); the chapter was read in the editors' free Ray Tracing Gems v1.9 PDF, PDF page 390, printed page 366, heading "22.5 LIMITATIONS": the quote verbatim through "increasing the base sample rate". |
| 474 | `story-2015-hybrid-shadows-aa` | sample | confirmed | NVIDIA GDC15 PDF, slide 26 (pdftotext, layout): title "Anti-Aliasing" and the three bullets verbatim, the dash and parenthesis included. |
| 479 | `unity-urp-screen-space-shadows-texture` | sample | confirmed | raw.githubusercontent.com at commit 0efc28ea: line 110 is the "reconstructs world positions from the depth prepass (cameraDepthTexture)" comment, lines 173-175 the descriptor, 178-180 the R8_UNorm choice; verbatim. |
| 484 | `wyman-2016-frustum-traced-izb` | sample | confirmed | cwyman.org PDF (pdftotext): §4.1 lines "With the µQuad and shadow planes ... 32 samples from a 2D Halton sampling" and "We use this 10-bit lookup ... visibility from the triangle"; §6.6 "Our final render pass loads from the G-buffer (Section 6.1) and this mask and performs Phong shading modulated by the shadow mask"; §8 "stored in a visibility mask buffer ... multiples of 32 bits for each pixel"; verbatim. |
| 489 | `pharr-2023-pbrt-mitchell` | sample | confirmed | pbr-book.org 4ed Image Reconstruction, Mitchell Filter section: "trading off between ringing (phantom edges next to actual edges in the image) and blurring (excessively blurred results)—two common artifacts" and "In practice these negative regions ... clamped to a legal output range"; verbatim. |
| 494 | `pettineo-2012-msaa-resolve-filters` | sample | confirmed | therealmjp.github.io post dated October 29, 2012: all seven quoted sentences verbatim (the quote's double space in "find  that" aside). |
| 499 | `webgpu-2026-per-sample-shading` | sample | confirmed | w3.org CRD 2026-09-15 (downloaded, tags stripped), heading "23.2.10. Per-Sample Shading": the three quoted sentences verbatim; status line "Candidate Recommendation Draft, 15 September 2026". |
| 504 | `heckbert-1989-box-side-lobes` | sample | confirmed | Berkeley PDF page 45, a scan read as an image: printed page 39, heading "B-Spline Filter", "We start with an unweighted average or first order box:" then "Its frequency response is a sinc, as shown at right. This filter, when sampled, is trivial to implement, (involving several additions and one multiplication), but its frequency response is far from ideal, with high side lobes."; verbatim. |
| 509 | `mccormack-1999-ewa-blur-and-setup` | sample | confirmed | graphicon.ru PDF (WebFetch fails on the host's certificate; downloaded with verification off, pdftotext), §2.1: the passage from "These setup computations" to "a factor of four or nine" verbatim. |
| 514 | `mavridis-2011-results-window-blur` | sample | confirmed | aueb.gr PDF (pdftotext), under "Results" and "Discussion": "The ellipse approximation method has some aliasing artifacts and the details on the left are blurred out, but still provides a tremendous improvement over the hardware filtering" and "We observe that even the reference EWA implementation ... introducing some blur"; verbatim ("extend" is the paper's). |
| 519 | `wgpu-types-30-sampler-descriptor-no-bias` | sample | confirmed | `~/.cargo/registry/src/*/wgpu-types-30.0.1/src/texture.rs:658-683`, struct `SamplerDescriptor`, and the same file at commit 40f4a34 on raw.githubusercontent.com, which the v30.0.1 release page names as the tag's commit: the five quoted fields verbatim, no `lod_bias` field anywhere in the crate. |
| 524 | `pharr-2024-stf-discussion` | sample | confirmed | arXiv 2407.06107v1 (downloaded, pdftotext), under "RECOMMENDATIONS" and "DISCUSSION AND FUTURE WORK": "Filtering after shading and STF work especially well for minification and reduce aliasing." and "We demonstrated that the minor noise ... patterns like a checkerboard."; verbatim. The section field calls the latter "Section 7 Discussion", the heading's first word. |
| 529 | `golus-2019-kaiser-mips` | sample | confirmed | web.archive.org capture 20241214090822 (WebFetch is barred from the host; downloaded, tags stripped): post dated Aug 10, 2019, section "Kaiser-filtered Mipmaps"; every quoted sentence verbatim, the "Note Nvidia's Kaiser filtering actually makes the lines blurrier overall!" passage included. |
| 532 | `wachter-binder-2019-offset-error` | full | confirmed | doi.org resolves to link.springer.com, which answers 303 to a login; read in the editors' free Ray Tracing Gems v1.9 PDF, PDF page 118, printed page 82, heading "6.2.2.4 ADAPTIVE OFFSETTING ALONG THE GEOMETRIC NORMAL": the quote verbatim. `src/render/shaders/shadow_mask.wgsl:25-34` scales the offset by the point's largest coordinate "as a ray's hit point is offset in proportion to its distance from the origin"; matches the quote's proportionality. |
| 533 | `d3d11.3-rasterizer-precision` | full | confirmed | microsoft.github.io D3D11_3_FunctionalSpec.htm (downloaded, tags stripped), §15.16 Rasterizer Precision: the paragraph verbatim, "EXACTLY 8 bits" included. `src/render/shaders/shadow_mask.wgsl:36-38` says vertices snap to 1/256 of a pixel on D3D11; matches 8 fractional bits. |
| 534 | `vulkan-subpixel-precision-bits` | full | confirmed | raw limits.adoc at commit b94f8bd2, lines 537-539 (the definition) and 7003-7004 (`subPixelPrecisionBits | - | 4 + 8 ({limit2026}) | min`): verbatim as cells. `src/render/shaders/shadow_mask.wgsl:38-39` says the grid is "as coarse as 1/16 under Vulkan"; matches the 4-bit minimum. |
| 535 | `apple-tech-talk-110147-busy-waits` | full | wrong | developer.apple.com Tech Talk 110147 transcript: the quote verbatim, Apple's "They also prevents" and "from and E" included. The page's `datePublished` and `uploadDate` metadata read 2021-12-07 and the transcript places the M1 Pro and M1 Max in "this year", so the section field's "(2022)" is wrong. `src/capture/mod.rs:283-286` ("Apple asks games not to busy-wait, which holds a P-core and spends thermal headroom") and `docs/performance.md:1165`, `:1267` claim what the quote says. Finding L7-X01. |
| 538 | `xnu-task-policy-role-qos-ceiling` | sample | confirmed | raw task_policy.c at commit f6217f89: lines 833-834 test `TASK_APPTYPE_APP_DEFAULT || TASK_APPTYPE_APP_NONUI`, lines 879-883 are the quoted `TASK_UNSPECIFIED`/`default` arm setting `THREAD_QOS_LEGACY`; verbatim. |
| 540 | `apple-metal-fence-per-stage` | full | confirmed | developer.apple.com/documentation/metal/mtlfence renders by script only; its documentation data (`/tutorials/data/documentation/metal/mtlfence.json`), the same page's content, carries the Overview paragraph verbatim from "Apple family GPUs can update and respond to fences on a per-stage basis" to "until another pass updates a fence". `src/render/readback.rs:629-631` ("Apple GPUs can run a render pass's vertex stage while its fragment stage waits for the passes before it") and `docs/performance.md:722`, `:1078` claim what the quote says. |
| 544 | `boksansky-2019-rt-hard-shadows` | sample | confirmed | realtimerendering.com v1.9 PDF, PDF pages 205-207, printed 174-176: the quote on page 175 under 13.5.4 (the paper's soft hyphen in "depth-buffer" aside), "The screen resolution for all tests was 1920 × 1080" and the Table 13-1 caption under 13.6 on pages 175-176; the table itself is an image on page 176, read rendered: Pub 281k triangles, RT hard 1.4 and 4.8; Resort 376k, 1.3 and 3.4; Breakfast 1.4M, 1.6 and 3.7; the `cost` field matches. |
| 546 | `nvidia-2022-rtx-best-practices-shadow-rays` | full | confirmed | developer.nvidia.com post by Juha Sjoholm, July 25, 2022: the BLAS-merging bullet under Acceleration structures, the first-hit bullet under Hit shading: General tips, and the register-consumption bullet with "Variables initialized before using the query object and used after that may consume additional registers" under Inline ray tracing; verbatim. `src/render/shaders/shadow_mask.wgsl:90-92` ("A shader's live ray queries hold registers that lower its occupancy, so every ray of a surface goes through this one call") and `docs/performance.md:598`, `:707` claim what the quote says. |
| 550 | `apple-wwdc22-10105-instance-overhead` | sample | confirmed | developer.apple.com WWDC22 session 10105 transcript, "Maximize your Metal ray tracing performance": the passage from "Focusing on the instance acceleration structure" to "both the boxes and the sphere", then, after the code walkthrough the gap marks, "This primitive acceleration structure will take less time to build than separate acceleration structures and will be faster to intersect."; verbatim. |
| 555 | `walter-2005-lightcuts-error-bound` | sample | confirmed | cornell.edu PDF (downloaded, pdftotext): "To prevent this, we only use clusters ... fixed percentage of the base signal", "In our experience, an error ratio of 2% results in no visible artifacts across a wide variety of scenes and was used for all our results", "so we will use the trivial upper bound of one for the visibility term (i.e. all lights are potentially visible)"; the `cost` sentence in §1; verbatim. |
| 560 | `wihlidal-2016-ordered-compaction` | sample | confirmed | web.archive.org capture of the Frostbite GDC 2016 PDF (downloaded), pages 42-43 with speaker notes (pdftotext, layout): the ds_ordered_count sentence, "Write out Surviving Indices (16 bit)", the InterlockedOr sentence, and the `cost` bullets "Maintain vertex reuse across all wavefronts - ds_ordered_count [5][15]" and "+0.1ms for ~3906 work items – use wavefront limits"; verbatim. |
| 565 | `webgpu-spd-coherence-limit` | sample | confirmed | raw README.md at commit adc666e3: line 30, under "Usage", has the `maxStorageTexturesPerShaderStage` sentence and line 154, under "Limit the number of mip levels and array layers per pass", the coherence sentence; verbatim. |
| 569 | `wgpu-hal-30-manual-memory-hints` | full | confirmed | `~/.cargo/registry/src/*/wgpu-hal-30.0.1/src/lib.rs:444-473`: the `Performance` arm's 128 and 256 MB, the `Manual` arm and its clamp comment, verbatim; `vulkan/adapter.rs:2951` and `dx12/suballocation.rs:78` call `from_memory_hints` as the section says. `src/gpu/mod.rs:3-5,19,79-81` passes `MemoryHints::Manual` with a 16 MiB block "in place of the `Performance` hint's 128 to 256 MiB", inside the 4 to 256 MiB clamp; matches. |
| 570 | `wgpu-hal-30-vulkan-texture-allocation` | full | confirmed | `~/.cargo/registry/src/*/wgpu-hal-30.0.1/src/vulkan/device.rs:1192-1205`: `create_texture` allocates with `AllocationScheme::GpuAllocatorManaged`; verbatim. `src/gpu/mod.rs:13-15` says wgpu-hal's Vulkan backend puts no texture it creates in dedicated memory; matches. |
| 571 | `gpu-allocator-0.28-personal-block` | full | confirmed | `~/.cargo/registry/src/*/gpu-allocator-0.28.0/src/vulkan/mod.rs:484-488` (`requires_personal_block = size > memblock_size`, the dedicated-block comment and branch) and `:667-673` (the free-path comment); verbatim; `Cargo.lock` pins 0.28.0. `src/gpu/mod.rs:5-7` ("A resource larger than a block gets a block of its own, destroyed with it") and `:883` (`allocation.size > DEVICE_MEMORY_BLOCK_BYTES`) match. |
| 572 | `nvidia-vulkan-dos-donts-dedicated-memory` | full | confirmed | developer.nvidia.com "Tips and Tricks: Vulkan Dos and Don'ts", published Jun 06, 2019, updated 1/14/2025, Memory Management, Do: both quoted sentences verbatim. `src/gpu/mod.rs:15-17` ("which NVIDIA says may speed colour and depth attachments, most on pre-Turing GPUs") matches. |
| 574 | `apple-launchd-start-calendar-interval-sleep` | full | confirmed | developer.apple.com archive, Scheduling Timed Jobs, "Effects of Sleeping and Powering Off": the paragraph verbatim. `scripts/sweep.sh:18-19` ("launchd, 04:00 local, or on the next wake when asleep then") matches. |
| 575 | `microsoft-query-user-notification-state` | full | confirmed | learn.microsoft.com QUERY_USER_NOTIFICATION_STATE, Constants: values 1, 2, 3 and 5 with the quoted descriptions verbatim (the gap after QUNS_BUSY's first sentence marked). `src/preflight/mod.rs:409-414` warns on 2 and 3, "2 is a full-screen application and 3 an exclusive full-screen Direct3D one"; matches. |
| 576 | `nvidia-smi-utilization-sample-period` | full | confirmed | docs.nvidia.com nvidia-smi, GPU Attributes, Utilization, GPU: both sentences verbatim. `scripts/sweep.sh:95-96` ("nvidia-smi's covers the past 1/6 s to 1 s") matches. |
| 577 | `wgpu-30-command-encoder-pool` | full | confirmed | `~/.cargo/registry/src/*/wgpu-core-30.0.1/src/command/allocator.rs:32-51`: `acquire_encoder` pops `free_encoders`, `release_encoder` pushes; verbatim. `src/render/acceleration_structures.rs:373-376` ("A new encoder reuses the encoder the GPU finished with last") matches a LIFO pool. |
| 578 | `wgpu-30-map-after-submission` | full | confirmed | `~/.cargo/registry/src/*/wgpu-core-30.0.1/src/device/life.rs:236-250`: `map` finds the latest active submission holding the buffer (`.rev().find`) and pushes to its `mapped` list, else to `ready_to_map`; verbatim. `src/render/readback.rs:37-39` ("maps in whichever submission or poll first sees the copy complete") matches. |
| 579 | `wgpu-30-handle-mapping` | full | confirmed | `~/.cargo/registry/src/*/wgpu-core-30.0.1/src/device/life.rs:396-413`: `handle_mapping` returns early on an empty list, else `Vec::with_capacity(self.ready_to_map.len())`; verbatim. `src/render/staging.rs:18-22` and `src/render/readback.rs:39` ("wgpu allocates for it there") match. |
| 580 | `wgpu-30-submit-maintains` | full | confirmed | `~/.cargo/registry/src/*/wgpu-core-30.0.1/src/device/queue.rs:1536-1541`: the cleanup scope, the three comment lines and `self.device.maintain(wgt::PollType::Poll, snatch_guard)`; verbatim. `src/render/readback.rs:34-36` ("the frame's first submission maps it") matches. |
| 581 | `wgpu-hal-30-metal-command-state` | full | confirmed | `~/.cargo/registry/src/*/wgpu-hal-30.0.1/src/metal/command.rs:432-441`: `CommandState::reset` clears the two maps, the five stage infos and `immediates`; verbatim, and `clear` keeps capacity as the section says. `src/render/acceleration_structures.rs:380-382` ("the state Metal keeps per encoder, which grows to the largest pipeline and bindings it recorded and never shrinks") matches. |


## Registry entries the verifier retried

The entries the readers marked `unopened`, retried by the verifier by another route.

| index | key | verdict | what was opened |
|---|---|---|---|
| 1 | `higham-3.1` | unopened | Retried by SIAM's chapter PDF, `https://epubs.siam.org/doi/pdf/10.1137/1.9780898718027.ch3` (HTTP 403, an HTML challenge page), and the archived landing page `web.archive.org/web/20241202175257id_/https://epubs.siam.org/doi/book/10.1137/1.9780898718027` (HTTP 200, a script shell with no book text). Crossref (`api.crossref.org/works/10.1137/1.9780898718027`) resolves the DOI to "Accuracy and Stability of Numerical Algorithms", Higham, SIAM, 2002, so the url is live, not `dead`; pp. 62-63 stay behind the paywall. The code site `src/render/shader_function_tests.rs:415-420` computes `3u/(1 − 3u)`, Lemma 3.1's γₙ at n = 3 as quoted. Needs a reader with the book. |
| 102 | `hillis-steele-partial-sums` | confirmed | The archived ACM PDF, `web.archive.org/web/20240815002852id_/https://dl.acm.org/doi/pdf/10.1145/7902.7903` (HTTP 200, `application/pdf`, 14 pages, title "Data parallel algorithms"), read with `pdftotext`: PDF page 4 carries the printed page number 1173 and, beside the heading "All Partial Sums of an Array", the sentence "the summation algorithm can compute all partial sums of the array in the same amount of time it took to compute the single sum." word for word (the two-column layout interleaves the extraction, the words are unchanged). Crossref confirms CACM 29(12), pp. 1170-1183, December 1986, as the section says. The live `dl.acm.org` urls answer 403 to curl, so the registry could carry the archived url beside the DOI. |

## Findings

The 45 Findings the verifier confirmed or narrowed, in id order; the
`placement` column is the triage's. A narrowed Finding keeps the reader's
claim in its `claim` cell and carries the rewritten claim in its `verifier`
cell. The evidence blocks follow the table; the killed Findings' blocks
follow them, kept so that the next audit does not re-find them.

| id | claim | severity | where | evidence | expected | verifier | placement |
|---|---|---|---|---|---|---|---|
| L7-A01 | ADR 0001 says the Material buffer is indexed by the Material ID each Instance carries, but the ID the shader indexes with is the Meshlet's Primitive's unless the Instance overrides it. | drift; not load-bearing (the one-buffer, one-bind-group decision holds); misleads a reader of the ADR | `docs/adr/0001-material-as-id-not-bind-group.md:3` | block below | The sentence names the Primitive's Material ID, carried in the Meshlet record, with the Instance's as an override. | `confirmed`: `docs/adr/0001-material-as-id-not-bind-group.md:3` says the buffer is "indexed by the Material ID that each Instance carries"; `src/render/shaders/culling.wgsl:381-386` writes the Meshlet record's ID as `select(meshlets[..].material_id, instance.material_id, overridden)` and `src/render/shaders/main.wgsl:113`, `:172` index `materials[in.material_id]` from the record, so an Instance without an override (`NO_MATERIAL`, `src/render/instance.rs:28-30`) indexes with its Primitive's ID. Not load-bearing, as the reader wrote. |  |
| L7-A02 | ADR 0002 describes a Mesh as a (vertex offset, index offset, index count) sub-allocation of one vertex and one index buffer, but since ADR 0019 the main pass draws a Mesh from Meshlet ranges in three further Arenas and the index Arena feeds BLAS builds only; the ADR names ADR 0017's refinement and not ADR 0019's. | drift (superseded by ADR 0019, unnamed); not load-bearing; misleads a reader of the ADR | `docs/adr/0002-one-mega-buffer-for-vertices-one-for-indices.md:3` | block below | A "Refined by ADR 0019" clause: the Mesh table names Meshlet ranges in the Meshlet Arenas, which the main pass draws; the index Arena serves the BLAS builds. | `confirmed`: `docs/adr/0002-one-mega-buffer-for-vertices-one-for-indices.md:3` describes a Mesh as "(vertex offset, index offset, index count)" in one vertex and one index buffer and names only ADR 0017's refinement; `src/assets/mesh.rs:22-30` is `MeshData { first_meshlet, meshlet_count, first_triangle, triangle_count, bounds }`, `src/assets/pools.rs:74-79` holds five Arenas and a Mesh table, and `:105` makes the index Arena `BLAS_INPUT` only, with the comment "which no pass draws from". |  |
| L7-A03 | ADR 0013 says Budgets are judged on the M4 Pro "until a Windows GPU machine exists; that machine gets its own ADR", but the RTX 2070 SUPER exists with Baselines on Vulkan and DX12, no ADR names it, and the glossary makes it a floor benched for comparison, not a Budget machine. | drift; load-bearing: the Budgets' reference hardware rests on it; misleads a reader of the ADR | `docs/adr/0013-reference-hardware-and-provisional-budgets.md:3` | block below | The sentence says the M4 Pro is the reference machine where Budgets are judged and the RTX 2070 SUPER the hardware floor benched for comparison, dropping or re-dating the promised ADR. | `confirmed`: `docs/adr/0013-reference-hardware-and-provisional-budgets.md:3`: "until a Windows GPU machine exists; that machine gets its own ADR". `bench/baselines/nvidia-geforce-rtx-2070-super/` and `-dx12/` hold eleven Baselines each; `docs/adr/` lists 0001 to 0024 and none is that ADR (0024 names the machine only as a sweep host); `CONTEXT.md:188` makes it "the hardware floor benched for comparison". Load-bearing, as the reader wrote: the decision of where Budgets are judged rests on the sentence. |  |
| L7-A04 | ADR 0013 says CPU time summed across threads "is measured and reported", but Frame stats hold only main-thread wall times per stage and per frame; nothing sums CPU time across threads. | drift; not load-bearing (the 6 ms Budget binds the critical path, which is measured); misleads a reader of the ADR | `docs/adr/0013-reference-hardware-and-provisional-budgets.md:3` | block below | The sentence says thread-summed CPU time is not yet measured, or names the stat that will carry it. | `confirmed`: The same sentence says "CPU time summed across threads is measured and reported"; `src/stats/mod.rs:10-15` holds `cpu_times` ("wall-clock time ... of each ECS system and renderer stage") and `cpu_frame_time`, and a grep of `src/` for any cross-thread CPU sum (`getrusage`, `process_cpu`, `user_time`, `across threads`) finds nothing. The reader's doubt (N2) does not hold: the sentence names a figure, and no stat carries it. |  |
| L7-A05 | ADR 0014's Consequences say that once draws are GPU-driven the culling pass composes camera-relative positions once per Instance in the vertex shader's place; draws are GPU-driven and the vertex shader still composes every vertex, the culling pass composing only each Instance's bounds centre for its test. | drift; not load-bearing; misleads a reader of the ADR | `docs/adr/0014-world-positions-are-f64-the-gpu-works-camera-relative.md:19` | block below | The sentence says the vertex shader composes each vertex camera-relative and the culling pass composes each Instance's bounds centre the same way. | `confirmed`: `docs/adr/0014-world-positions-are-f64-the-gpu-works-camera-relative.md:19`: "until draws are GPU-driven; then the culling pass does it once per Instance". Draws are indirect (`src/render/main_pass.rs:262`); `src/render/shaders/main.wgsl:95-101` still calls `camera_relative_position` per vertex, and `src/render/shaders/culling.wgsl:123-129` composes only the bounds centre for `ClipBox`, writing no transform the vertex shader reads. |  |
| L7-A06 | ADR 0015 says the far-coplanar Golden "turns red under standard depth"; the Golden is red under reversed depth (the red upper plane covers the green lower one) and would interleave red and green under standard depth. | drift; not load-bearing; misleads a reader of the ADR and an agent checking the Golden | `docs/adr/0015-reverse-z-depth-with-an-infinite-far-plane.md:3` | block below | The sentence says the Golden is red, the upper plane covering the lower everywhere, and interleaves red and green under standard depth. | `narrowed`: "ADR 0015's clause 'the `far-coplanar` Reference scene's Golden turns red under standard depth' is ambiguous and misleads either way: read as a colour it is false, since the committed Golden is red under reversed depth (the red upper plane covers the green lower one, `src/scenes/mod.rs:38-39`, `:596`, `:606`; `tests/goldens/metal/far-coplanar.png` is a uniform red ground) and would interleave red and green under standard depth; read as 'the Golden test fails' it is true but, in a scene of a red and a green plane, is taken for the colour." `docs/adr/0015-reverse-z-depth-with-an-infinite-far-plane.md:3`. The reader read only the colour sense. |  |
| L7-A07 | ADR 0016 says per-frame data the GPU reads is written through `Queue::write_*`; it is written through a ring of mapped staging slots, copied on the frame's encoder, because `Queue::write_buffer` allocates fresh staging per call (ADR 0017's rule); the ADR does not name the refinement. | drift (superseded by ADR 0017, unnamed); not load-bearing for the decision; misleads a reader of the ADR and a reviewer applying the checklist's staging rule | `docs/adr/0016-the-main-thread-runs-the-frame.md:18` | block below | The sentence says per-frame data goes through the per-frame staging ring (ADR 0017), whose slot is reused only after its map callback fires, and keeps the readback ring clause. | `narrowed`: "ADR 0016 names `Queue::write_*` as the path per-frame data takes; no shipping code calls `Queue::write_buffer` or `Queue::write_texture` (`grep -rn` over `src/`: `staging.rs:191` is the belt's method, `:198` the engine's own, the rest test modules), every per-frame upload going through the mapped staging ring of `src/render/staging.rs:11-23` on the frame's encoder (`src/render/renderer.rs:549-551`), which the sentence's second alternative ('or into a ring of mapped buffers whose slot is reused only after its `map_async` callback fires') describes without naming ADR 0017's rule." `docs/adr/0016-the-main-thread-runs-the-frame.md:18`. The sentence is not false, since its "or" covers the ring; the drift is a dead first alternative and the missing refinement. |  |
| L7-A08 | ADR 0016 says ADR 0019 answers the draw count with one `draw_indexed_indirect` per pipeline; the depth pre-pass pipeline draws twice, once per Hi-Z phase, as ADR 0019 itself says ("per pipeline and Hi-Z phase"). | drift; not load-bearing; misleads a reader of the ADR | `docs/adr/0016-the-main-thread-runs-the-frame.md:20` | block below | The sentence says "one `draw_indexed_indirect` per pipeline and Hi-Z phase", as ADR 0019 does. | `confirmed`: `docs/adr/0016-the-main-thread-runs-the-frame.md:20`: "one `draw_indexed_indirect` per pipeline". `src/render/main_pass.rs:253`, `:262` and `:290`, `:296` set `depth_prepass_pipeline` twice and draw twice (offsets 0 and `PHASE_TWO_DRAW_OFFSET`); `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:15` itself says "per pipeline and Hi-Z phase". |  |
| L7-B01 | ADR 0017 says acceleration-structure bytes are estimated "by backend" from two measured adapters, but the code holds a third estimate, NVIDIA's, and picks by vendor first, then backend. | drift; not load-bearing (the ledger is an estimate either way); misleads a reader of the ADR and a reviewer checking `gpu_memory_bytes` on an NVIDIA adapter | `docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:18` | block below | The sentence names the three estimates and that the adapter's vendor, then its backend, selects one. | `confirmed`: `docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:18` names two measured estimates and "estimates by backend"; `src/render/acceleration_structures.rs:62-71` holds a third, `NVIDIA` (70.7, 29.2, 238.2), and `:73-85` `estimate` picks by `adapter.vendor == NVIDIA_VENDOR` first, then by backend. |  |
| L7-B02 | ADR 0017 says the cooked format stores each Primitive's UV texel density; no cooked Primitive holds one. | drift; not load-bearing (streaming is unbuilt); misleads a reader of the ADR, who takes the format clause as built like the mip-range clause beside it | `docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:20` | block below | The clause marks the texel density as planned with streaming, or the status line says the Mesh Blob holds none yet. | `confirmed`: `docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:20`: "The cooked format ... stores ... each Primitive's UV texel density". `src/assets/cooked.rs:405-414` `CookedPrimitive` has `first_index`, `index_count`, `material`, `meshlets`; `:522-527` writes those four per Primitive; `grep -rn texel_density src/` finds nothing. The status line's "still without streaming" (`:3`) does not reach a format clause written beside the built mip-range clause. |  |
| L7-B03 | ADR 0018's status line says the renderer does not keep the rebuilt Meshlets yet; the Mesh pool holds three Meshlet Arenas the main pass draws from. | drift; not load-bearing; misleads a reader of the status line about what is built | `docs/adr/0018-cooked-assets-are-named-by-stable-ids-and-stored-by-content-hash.md:3` | block below | The status line says the Mesh pool keeps the Meshlets in Arenas the main pass draws from. | `confirmed`: `docs/adr/0018-cooked-assets-are-named-by-stable-ids-and-stored-by-content-hash.md:3`: "the reader rebuilds their Meshlets ([...]), which the renderer does not keep yet". `src/assets/pools.rs:74-78` holds `meshlet_arena`, `meshlet_vertex_arena`, `meshlet_triangle_arena`, and `src/render/renderer.rs:367-374` binds all three as `meshlets`, `meshlet_vertices`, `meshlet_triangles` for the main pass. |  |
| L7-B04 | ADR 0018 says each Primitive stores its UV texel density; `CookedPrimitive` and the Mesh Blob hold none. | drift; not load-bearing; misleads a reader of the ADR (same slip as L7-B02) | `docs/adr/0018-cooked-assets-are-named-by-stable-ids-and-stored-by-content-hash.md:24` | block below | The sentence says the Primitive will store it when streaming lands, or is cut until it does. | `confirmed`: `docs/adr/0018-cooked-assets-are-named-by-stable-ids-and-stored-by-content-hash.md:24`: "Each Primitive stores its UV texel density". Same code as L7-B02 (`src/assets/cooked.rs:405-414`, `:522-527`): no such field in the struct or the Blob. A second location of the same slip, not a duplicate. |  |
| L7-B05 | ADR 0019's Consequences say the depth pre-pass writes 4x MSAA depth only; with shadow rays on (the default) it also writes the surface normals attachment. | drift; not load-bearing (the summary's amendment states the normals); misleads a reviewer reading the Consequences for what the pre-pass writes | `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:17` | block below | "The depth pre-pass writes 4x MSAA depth, and with shadow rays on each sample's surface normals". | `confirmed`: `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:17`: "The depth pre-pass writes 4x MSAA depth only". `src/render/main_pass.rs:114-125` gives the pre-pass pipeline a `SURFACE_NORMALS_FORMAT` colour target with shadow rays on, `:240-246` attaches `surface_normals` in that case, and `src/render/renderer.rs:144` defaults `shadow_rays: true`. |  |
| L7-B06 | ADR 0019's Consequences say the shadow mask pass rebuilds each pixel's face normal from the depth target; it reads the normals from the surface normals target the depth passes write. | drift; not load-bearing; misleads a reader of the ADR (same slip as L7-B05: a pass clause not re-read by the amendment) | `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:17` | block below | "the shadow mask pass rebuilds each pixel's position from it and reads its normals from the surface normals the depth passes write". | `confirmed`: The same bullet: "the shadow mask pass rebuilds each pixel's position and face normal from it [the depth target]". `src/render/shaders/shadow_mask.wgsl:210-218` rebuilds the position from depth (`position_at`) and takes both normals from `textureLoad(surface_normals, pixel, sample)`, the target `src/render/targets.rs:91-93` says the depth passes write. |  |
| L7-B07 | ADR 0019 says "today's up to four point lights become a light buffer"; the light buffer exists, with a capacity of 4096, and the Froxel pass lists from it. | drift; not load-bearing; misleads a reader of the ADR about the present | `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:18` | block below | The "today" clause goes, or says the light buffer holds 4096 lights of fixed capacity. | `confirmed`: `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:18`: "today's up to four point lights become a light buffer". `src/render/uniforms.rs:1-4` `LIGHT_CAPACITY: u32 = 4096` and `src/render/froxel_pass.rs:1-4` list from that buffer; the "today" is the one before the buffer landed. |  |
| L7-B08 | ADR 0020's status line says each Mesh gets a compacted BLAS at load; a Mesh loaded to be bent gets no BLAS and a refitted one is built to allow updates and never compacted. | drift; not load-bearing; misleads a reader of the status line (the bent-figure exception appears four sentences later, the uncompacted refit nowhere) | `docs/adr/0020-ray-traced-probe-gi-and-hard-ray-traced-shadows.md:3` | block below | "each Mesh rays trace gets a BLAS at load, compacted unless it is refitted". | `confirmed`: `docs/adr/0020-ray-traced-probe-gi-and-hard-ray-traced-shadows.md:3`: "each Mesh gets a compacted BLAS at load". `src/assets/pools.rs:303-306` pushes `None` for a Mesh that `!mesh_use.traced()` (`MeshUse::Drawn`, `:19-21`), and `:321-327`, `:341-342` build a `Refitted` Mesh's BLAS with `REFITTED_BLAS_FLAGS` and never queue it for compaction ("never compacted", `:26-27`). The bent exception appears four sentences later in the paragraph, as the reader said; the refitted one nowhere. |  |
| L7-C01 | ADR 0021 says the main pass's multisampled attachment formats are `Depth32Float` and `Rgba16Float` and that the floor checks 4x MSAA on every one; the Surface Normals target, `Rgba8Unorm` at 4 samples, is a third main-pass attachment the floor's list omits | drift; load-bearing: a Capability floor entry rests on it; misleads a reviewer applying the checklist's floor item and an agent adding an attachment | docs/adr/0021-capability-floor.md:14 | block below | The sentence lists every multisampled attachment format the main pass uses, or says the floor's list covers depth and HDR colour only and why | `confirmed`: `docs/adr/0021-capability-floor.md:14`: "4x MSAA on every attachment format the main pass uses ... Those formats are `Depth32Float` and the HDR target's `Rgba16Float`". `src/render/targets.rs:45` `SURFACE_NORMALS_FORMAT = Rgba8Unorm`, created at `SAMPLE_COUNT` 4 (`:207-212`) and attached by the depth pre-pass (`src/render/main_pass.rs:240-246`) and the phase-two depth pass (`:281`); the floor's `MAIN_PASS_FORMATS` (`src/gpu/mod.rs:267-269`) lists the two the ADR lists. Load-bearing, as the reader wrote: the sentence defines a floor entry. Lane 1 may see the code half (`MAIN_PASS_FORMATS` itself omits the third format). |  |
| L7-C02 | ADR 0021 says the main pass draws the frame's draw with one `draw_indexed_indirect`; it issues four, across the depth pre-pass, the phase-two depth pass and the lit pass | drift; not load-bearing (`INDIRECT_EXECUTION` is needed for any indirect draw); misleads a reader of the ADR | docs/adr/0021-capability-floor.md:14 | block below | "draws it with indirect draws, one per phase and pipeline" or equivalent | `confirmed`: The same line: "the main pass draws it with one `draw_indexed_indirect`" and "the one draw spans every Instance". `src/render/main_pass.rs:262`, `:296`, `:330`, `:333` issue up to four indirect draws across three passes; `src/render/shaders/culling.wgsl:2-4` says "one indirect call per phase and depth pipeline, and one of both phases for the lit pipeline". The same slip as L7-A08 in another ADR, not a duplicate. |  |
| L7-C03 | ADR 0021 says lavapipe's limits "are not yet measured"; CI has requested the floor's device on lavapipe on every run since, and the ADR's own later amendments quote lavapipe's limits | drift; not load-bearing; misleads a reader of the ADR | docs/adr/0021-capability-floor.md:10 | block below | Drop the clause or say CI proves lavapipe meets the floor | `confirmed`: `docs/adr/0021-capability-floor.md:10`: "its limits are not yet measured". The same ADR's `:16` quotes lavapipe's `maxStorageBufferRange` and storage-buffer count from Mesa 25.2.8's source, and `.github/workflows/ci.yml:58-90` (`software-goldens`, Mesa `25.2.8-0ubuntu0.24.04.4` at `:30`) requests the floor's device on lavapipe through `GpuContext::headless` (`src/gpu/mod.rs:31-35`) on every run; the clause is stale under either reading of "measured". |  |
| L7-C04 | ADR 0022 says the camera's "position follows whatever it is attached to, interpolated"; the camera is a resource moved per frame by `fly_camera` and attaches to nothing | drift; not load-bearing; misleads a reader of the ADR looking for an attachment mechanism | docs/adr/0022-fixed-64-hz-simulation-with-interpolation.md:18 | block below | Say the camera is free-flying today and that an attached camera would interpolate | `confirmed`: `docs/adr/0022-fixed-64-hz-simulation-with-interpolation.md:18`: "its position follows whatever it is attached to, interpolated". `src/camera/mod.rs:99-110` `fly_camera` moves the `Camera` resource from key presses, `src/state.rs:320` runs it per frame, and a grep for `attach` over `src/camera`, `src/state.rs`, `src/components.rs` finds nothing; the status line (`:3`) marks no attached camera as planned. |  |
| L7-C06 | ADR 0024 says the key, and so the record path, is source hash × adapter × backend × scene × Harness version; the path has no Harness version, which the source hash implies | drift; not load-bearing; misleads an agent querying the branch with `git` and `jq` | docs/adr/0024-bench-history-is-git-stored-bench-records-keyed-by-engine-source.md:3, :15-16 | block below | Say the Harness version is a function of the source hash and is held in the record, not the path | `confirmed`: `docs/adr/0024-bench-history-is-git-stored-bench-records-keyed-by-engine-source.md:3`, `:15` give a five-factor key and `:16` "A record path is its key". `src/bench/history.rs:204-212` `path` joins `adapter_key`, `subject` and `<source_hash>.json`, and its comment says "The Harness version it holds is not in the path, as it is defined under `src/`, so a source hash names one." |  |
| L7-C07 | ADR 0024 says 61 commits touched `bench/baselines/` before the history existed; 64 first-parent commits before `HISTORY_STARTED` did | drift; not load-bearing; misleads a reader of the ADR | docs/adr/0024-bench-history-is-git-stored-bench-records-keyed-by-engine-source.md:19 | block below | The count at `HISTORY_STARTED`, or no count | `confirmed`: `docs/adr/0024-bench-history-is-git-stored-bench-records-keyed-by-engine-source.md:19`: "The 61 commits". With `HISTORY_STARTED = "3c6b9d92"` (`src/bench/history/import.rs:19-22`), `git log --oneline --first-parent 3c6b9d92^ -- bench/baselines | wc -l` prints 64 at the frozen commit, and so does the walk without `--first-parent`. |  |
| L7-C08 | ADR 0024 says a re-recorded Baseline's record is marked as one; only imported Baselines carry a `baseline-commit` source, and a Regression's Baseline cover is decided by `git diff`, not by a mark on the record | drift; not load-bearing (the Regression rule works through git); misleads an agent looking for the mark | docs/adr/0024-bench-history-is-git-stored-bench-records-keyed-by-engine-source.md:20 | block below | Say imported Baselines are marked and a later Baseline is recognised by the commit that changed its file | `confirmed`: `docs/adr/0024-bench-history-is-git-stored-bench-records-keyed-by-engine-source.md:20`: "a re-recorded Baseline's record is marked as one". The only writer of the mark is the import: `IMPORTED_SOURCE = "baseline-commit"` at `src/bench/history/import.rs:69`, written at `:277` for Baselines before `HISTORY_STARTED` ("every bench since has published its own", `:20-21`); `src/bench/history.rs:258-260` says "none for a bench's result", and a pinned commit's Baseline cover is read from `git diff` at `src/bench/history/verdicts.rs:424-432`. |  |
| L7-G01 | The glossary's Series is "One metric's Bench records", but a Series in the code is one scene's records on one adapter and backend, carrying every metric. | drift; load-bearing (Suspect, Experiment and Regression are defined on it, and the bench-history guide copies the sentence); misleads readers of the bench page, the guide and the Regression issues | `CONTEXT.md:199-200` | block below | "The Bench records of one Reference scene on one adapter and backend, taken on `main`'s source by a quiet machine ..., in source order; a chart draws one metric of it, and a Regression is found in one metric of it." | `confirmed`: `CONTEXT.md:200`: "One metric's Bench records". `src/bench/history.rs:58-61` (`series/<adapter>-<backend>/<scene>.json`: one row per record, holding every counter), `:420` ("a record's source and its metrics"), `:386-397` (two timings judged over one Series' rows), `src/bench/history/verdicts.rs:710` ("Series: `{scene}` on `{adapter_key}`") and `docs/bench/index.html:392`, `:399-400` (one Series per machine and scene) all make a Series one scene's records on one adapter, carrying every metric. Load-bearing, as the reader wrote: Suspect, Experiment and Regression are defined on it. |  |
| L7-G02 | The glossary's Suspect says only a Harness version, a scene content or an imported Baseline starts the records that judge it, but a commit a Verdict pinned also starts a line. | drift; not load-bearing; misleads a reader judging why a record is or is not marked (the guide's tooltip and its "Line" entry copy the sentence) | `CONTEXT.md:212` | block below | "Only the records since the last Harness version, scene content, imported Baseline or commit a Verdict pinned judge it." | `confirmed`: `CONTEXT.md:212`: "Only the records since the last Harness version, scene content or imported Baseline judge it." `src/bench/history.rs:356-364` also starts a line where `line_starts.contains(row.source_hash)`, the sources `verdicts::pinned_sources` returns (`:1025-1027`), and the README states the fourth start (`:81-83`). |  |
| L7-G03 | The glossary says the GPU memory Budget "splits into one Budget per kind of GPU memory", but the code holds one `gpu_memory_bytes` Budget and names the split as pending. | drift; load-bearing (the Budget term and the Arena, texture pool and staging capacities rest on it; ADR 0017 says the sizes are set later); misleads a reviewer applying the Budgets and an agent sizing a buffer | `CONTEXT.md:224` | block below | "The GPU memory Budget is to split into one Budget per kind ... (ADR 0017); today it is one number and each kind's capacity is provisional." | `confirmed`: `CONTEXT.md:224`: "The GPU memory Budget splits into one Budget per kind of GPU memory". `bench/budgets.json` holds one `gpu_memory_bytes`, `src/bench/mod.rs:375-389` has the same six fields, and `src/assets/pools.rs:7-9` calls the Arena capacity provisional "until ADR 0013's split of the 8 GB Budget replaces it". Magnitude corrected: not load-bearing; the term Budget means the same under either status, and no decision, Budget, floor entry or exit criterion rests on the glossary's tense. |  |
| L7-G04 | The glossary lists Input latency among the metrics a Budget limits, but no Budget, Baseline or measurement of Input latency exists. | drift; not load-bearing (ADR 0013 names the figure, judged "once Input latency is measured"); misleads a reader expecting `praxis compare` to judge it | `CONTEXT.md:224` | block below | List the metrics the Budgets file holds (CPU ms, GPU ms, the gate, the dynamic TLAS's ceilings, GPU memory) and mark Input latency as planned. | `confirmed`: `CONTEXT.md:224` lists "Input latency" among a Budget's metrics. `bench/budgets.json` and `src/bench/mod.rs:375-389` hold none, `grep -rn -i "input latency\|input_latency" src bench` finds nothing, and ADR 0022's `:21` defers it to "once Input latency is measured"; ADR 0013 decides the figure, but no Budget, Baseline or stat carries it. |  |
| L7-G05 | The glossary says a Renderable keeps its Instance slot "while it is one", but prepare frees the slot of a Renderable it skips: one whose Mesh is not in the pool, or that has no Material Handle where a Primitive names no Material. | drift; not load-bearing; misleads a reader of the Instance term about which slots stay written (ADR 0003's rule) | `CONTEXT.md:88` | block below | "... in a slot of the Instance buffer that the Renderable keeps while it is one and is drawn; one skipped frees its slot." | `confirmed`: `CONTEXT.md:88`: a slot "that the Renderable keeps while it is one". `src/render/prepare.rs:358-372` frees the slot (`instance_slots.free(entity)`) of a Renderable whose Mesh is not in the pool or that lacks a Material Handle where a Primitive names none, both still Renderables by `CONTEXT.md:10`; the code's own comment says so at `:135-136` and `:244-246`. |  |
| L7-G06 | The glossary says any directional light whose bit differs from a 4-neighbour's, or that the nearest sample turns from while another faces, is traced again per sample, but only the first seven directional lights are. | drift; not load-bearing (every Reference scene has one directional light); misleads a reader of the term and of the shadow-edge guide about the eighth light | `CONTEXT.md:112` | block below | "Where one of the first seven directional lights' bit at the nearest surface differs from a 4-neighbour's ..." | `confirmed`: `CONTEXT.md:112`: "Where a directional light's bit ... differs from a 4-neighbour's ... that light is traced again". `src/render/shadow_mask_pass.rs:48-52` `SHADOW_EDGE_LIGHTS: u32 = 7` ("Any other keeps one ray per pixel"), `src/render/shaders/shadow_mask.wgsl:461-462` masks `min(light.directional_light_count, SHADOW_EDGE_LIGHTS)` lights and `:582` loops over `SHADOW_EDGE_LIGHTS`; ADR 0020 `:3` says "one of the first seven directional lights". |  |
| L7-G07 | The glossary says the GPU "can cull [a Meshlet] as one", but Meshlets carry no bounds and the culling pass tests Instances only. | drift; not load-bearing (ADR 0019 defers per-Meshlet tests); misleads a reader of the term and a reviewer of the culling pass | `CONTEXT.md:120` | block below | "... that the GPU draws as a unit; culling tests the Instance's Mesh bounds, not the Meshlet (ADR 0019)." | `confirmed`: `CONTEXT.md:120`: a Meshlet "that the GPU ... can cull as one". `src/assets/mesh.rs:37-44` `MeshletData` carries no bounds, `src/render/shaders/culling.wgsl:8-10` and `:245-250` test `clip_space_bounds(instance, mesh)` per Instance and append every Meshlet of a drawn one (`:382-386`); ADR 0019 `:15` defers per-Meshlet tests. |  |
| L7-G08 | The glossary defines Verdict as the sweep's record only, while `pub enum Verdict` in the bench module (compare's warn or fail) and in preflight (a check's ok, note, warn or fail) name other things. | drift; not load-bearing; misleads a reader of `praxis compare` or `praxis preflight` output and of the code, where "verdict" means a different outcome from the glossary's | `CONTEXT.md:215-217` | block below | The term names the sweep's Verdict and its `_Avoid_` line or a note distinguishes compare's and preflight's verdicts (warn or fail of one check or metric), or the glossary adds the second sense. | `confirmed`: `CONTEXT.md:215-217` defines Verdict as the sweep's record, which `src/bench/history/verdicts.rs:79-101` matches; `src/bench/mod.rs:400-403` `pub enum Verdict { Warn, Fail }` (compare's, printed at `src/bin/praxis.rs:657`) and `src/preflight/mod.rs:61-69` `pub enum Verdict { Ok, Note, Warn, Fail }` give the word two other public meanings the glossary neither names nor excludes. |  |
| L7-G09 | The glossary defines Finding as an Audit lane's defect record, while `pub struct Finding` in the bench module is "A metric that regressed against its Baseline or exceeds its Budget". | drift; not load-bearing; misleads a reader of `praxis compare` and `scripts/verify.sh` output, which print bench Findings that are not audit Findings | `CONTEXT.md:239-241` | block below | The term's `_Avoid_` line or a note distinguishes compare's Finding (one metric past its Baseline or Budget), or the glossary adds the second sense. | `confirmed`: `CONTEXT.md:239-241` defines Finding as an Audit lane's eight-field record; `src/bench/mod.rs:405-411` `pub struct Finding { metric, verdict, detail }`, "A metric that regressed against its Baseline or exceeds its Budget", is what `praxis compare` prints (`:413-420`) and `scripts/verify.sh` runs. |  |
| L7-R01 | The Project shape places `README.md` under `tests/goldens/`, where no README exists; the Goldens the text means are shown by the repository root's `README.md`, which the test reads. | drift; not load-bearing; a reader of the shape is misled | `CLAUDE.md:191-194` | block below | The shape names the root `README.md` (or says "the root README") as the file that shows the Metal Goldens. | `confirmed`: `CLAUDE.md:191-194` puts "README.md shows the metal ones" in the `tests/goldens/<backend>[-cpu]/` entry; `ls tests/goldens` is `metal`, `vulkan`, `vulkan-cpu` and `find tests -name 'README*'` finds nothing; `tests/goldens.rs:386-388` reads `root.join("README.md")`, the repository root's, whose lines 5 and 105-112 link the nine Metal Goldens. |  |
| L7-R02 | The convention "Bevy imports: `use bevy_ecs::prelude::*;` (not piecemeal)" is not what the tree does: three shipping files import items the prelude does not export by their paths. | drift; not load-bearing; a reviewer applying the convention is misled | `CLAUDE.md:277` | block below | The convention says the prelude is used for what it exports and an item outside it is imported by its path. | `narrowed`: "The convention 'Bevy imports: `use bevy_ecs::prelude::*;` (not piecemeal)' (`CLAUDE.md:277`) names no exception for items the prelude lacks: `src/state.rs:5` (`RunSystemOnce`), `src/threading.rs:6` (`ExecutorKind`) and `src/time/mod.rs:4` (`LogLevel`, `ScheduleBuildSettings`) import by path beside the prelude glob, and bevy_ecs 0.18.1's `pub mod prelude` (`~/.cargo/registry/src/*/bevy_ecs-0.18.1/src/lib.rs`) exports none of the four, so the convention cannot be followed literally there." Every file does use the glob, so the reader's "is not what the tree does" overstates; the drift is a missing exception. |  |
| L7-R03 | The convention "WGPU types: full path (`wgpu::Buffer`), don't import individual types" is not what the tree does: `blas_refit.rs` imports the type `wgpu::hal::api::Metal` twice. | drift; not load-bearing; a reviewer applying the convention is misled | `CLAUDE.md:278` | block below | The convention names its exceptions (wgpu-hal's `api::Metal`, and traits imported `as _` for their methods) or says "wgpu's own types". | `confirmed`: `CLAUDE.md:278`: "don't import individual types". `src/render/blas_refit.rs:174` and `:287` import `wgpu::hal::{self, CommandEncoder as _, api::Metal}` and `{self, Device as _, api::Metal}` in shipping code, `api::Metal` being a type; `grep -rn '^\s*use wgpu::' src tests` finds the rest in test modules only (`src/gpu/mod.rs:662`, `:797-798`, `:817`, `src/preflight/mod.rs:635`, `coverage_tests.rs:13`, `shader_tests.rs:10`). Low magnitude, as the reader wrote. |  |
| L7-R04 | The checklist item "Workflows keep least-privilege permissions and pinned action versions" is not what the tree does for versions: `ci.yml` uses floating refs (`@v4`, `@v2`, `@stable`) while `pr-lint.yml` pins a commit SHA. | drift; not load-bearing; a reviewer applying the checklist is misled, and the two workflows disagree on what "pinned" means | `CLAUDE.md:387-388` | block below | The item says which form pins (a commit SHA with the version in a comment, as `pr-lint.yml` does) or asks for "versioned actions" and names `@stable` as the exception, if the owner keeps it. | `confirmed`: `CLAUDE.md:387-388`: "pinned action versions". `.github/workflows/ci.yml:37`, `:39`, `:43` and their repeats use `actions/checkout@v4`, `dtolnay/rust-toolchain@stable` (a branch, not a version) and `Swatinem/rust-cache@v2`; `.github/workflows/pr-lint.yml:29` pins `actions/checkout@11d5960a... # v4.4.0`. Permissions hold (`ci.yml:20-21`, `:220-221`; `pr-lint.yml:14-16`); the two workflows disagree on what pinned means. |  |
| L7-S01 | The comment on `srgb_decode` cites `khronos-dfs-srgb`, whose quote is the sRGB EOTF⁻¹ (knee 0.0031308, × 12.92, power 1/2.4), for a function that implements the sRGB EOTF (knee 0.04045, ÷ 12.92, power 2.4), which that entry's quote does not state and which the registry's `khronos-dfs-srgb-eotf` entry does. | drift; not load-bearing (a test helper's citation); misleads a reviewer applying the Sources axis, who finds 0.04045 nowhere in the cited quote | `src/render/shader_function_tests.rs:1169` | block below | The comment cites `khronos-dfs-srgb-eotf`, the entry whose quote gives the decode's constants, as `src/bench/mod.rs:284` does for the same formula. | `narrowed`: "The comment on `srgb_decode` (`src/render/shader_function_tests.rs:1169-1170`) cites `khronos-dfs-srgb`, whose quote is the EOTF⁻¹ (`docs/references.toml:218-222`), and describes the function correctly as that encoding's inverse; the decode's own knee 0.04045 and exponent 2.4 are quoted by `khronos-dfs-srgb-eotf` (`:224-228`), which `src/bench/mod.rs:284` cites for the same formula, so the site cites the less direct of two entries rather than a wrong one." Both quotes read today at `registry.khronos.org/DataFormat/specs/1.3/dataformat.1.3.html`, §13.3.1 and §13.3.2, word for word. Not a false citation; a reviewer must invert the quoted formula to reach the constants. |  |
| L7-U01 | The registry quotes a CNIG download url, `busquedaSerie.do?codSerie=MDT05`, as a `gmd:onLine` linkage of the MDT05 CSW record, and the record carries no such linkage. | drift; not load-bearing (a research note's asset table, `docs/research/test-assets.md:27`, is the only citer); misleads a reader following the record to the download. | `docs/references.toml:1007` | block below | The quote names a linkage the record holds: `https://centrodedescargas.cnig.es/CentroDescargas/modelo-digital-terreno-mdt05-primera-cobertura [...] https://servicios.idee.es/wcs-inspire/mdt?request=GetCapabilities&service=WCS&version=2.0.1`. | `confirmed`: `docs/references.toml:1010` quotes `busquedaSerie.do?codSerie=MDT05` as a `gmd:onLine` linkage. The CSW record at the entry's url, fetched today (HTTP 200), holds eight `gmd:URL`s, the distribution linkages being the WMS, WMTS and WCS GetCapabilities urls and `https://centrodedescargas.cnig.es/CentroDescargas/modelo-digital-terreno-mdt05-primera-cobertura`; `grep -c busquedaSerie` is 0. |  |
| L7-W01 | The entry `apple-mtldevice-current-allocated-size` has no `checked` field, which the registry's preamble makes part of every entry's form. | drift; not load-bearing (the quote is confirmed); misleads a reviewer re-checking the registry, who cannot tell when it was read | `docs/references.toml:1945` | block below | The preamble (`docs/references.toml:12`) says `checked = <date> # the date the quote was last read in the source`; 580 of 581 entries carry it. | `confirmed`: `docs/references.toml:1945-1948`: the entry `apple-mtldevice-current-allocated-size` has `url`, `section`, `quote` and no `checked`; the preamble (`:12`) makes `checked` part of the form, `grep -c '^\["'` is 581 and `grep -c '^checked'` 580, and an awk over the file names this entry as the one without. |  |
| L7-W02 | The percentages quoted from the live gpuinfo Windows extensions page no longer match the page at its url: four of five figures have moved. | drift; not load-bearing (a research note's coverage figure); misleads a reader opening the url to confirm the quote | `docs/references.toml:2964` | block below | A quote that a reader can confirm at the url: an archived snapshot of the page as read, or figures given as approximate. | `confirmed`: `docs/references.toml:2964` quotes budget 91.03, priority 78.13, pageable 49.72, NV decompression 31.78, EXT decompression 21.5 %. `vulkan.gpuinfo.org/listextensions.php?platform=windows` today, `Recent (1y)` selected in the fetched HTML: 91.03, 78.32, 49.35, 31.59, 21.31 %. Four of five moved; the rubric's `stale`. |  |
| L7-W03 | The Windows `sparseResidencyImage2D` coverage quoted from the live gpuinfo page no longer matches the page. | drift; not load-bearing; same reader misled | `docs/references.toml:2940` | block below | As L7-W02. | `confirmed`: `docs/references.toml:2940` quotes `sparseResidencyImage2D: 76.64 %: 23.36 %`; `listfeaturescore10.php?platform=windows` today reads `77.01 % 22.99 %`. |  |
| L7-W04 | The Linux `sparseResidencyImage2D` coverage quoted from the live gpuinfo page no longer matches the page. | drift; not load-bearing; same reader misled | `docs/references.toml:2946` | block below | As L7-W02. | `confirmed`: `docs/references.toml:2946` quotes `72.11 %: 27.89 %`; `listfeaturescore10.php?platform=linux` today reads `71.92 % 28.08 %`. |  |
| L7-W05 | The Linux `LAZILY_ALLOCATED_BIT` memory-type coverage quoted from the live gpuinfo page no longer matches the page. | drift; not load-bearing; same reader misled | `docs/references.toml:2958` | block below | As L7-W02. | `confirmed`: `docs/references.toml:2958` quotes `DEVICE_LOCAL_BIT LAZILY_ALLOCATED_BIT: 3.85 %: 96.15 %`; `listmemory.php?platform=linux` today reads `3.88 % 96.12 %` (the Windows twin at `:2952` still reads its quoted `5.05 % 94.95 %`). |  |
| L7-W06 | The percentages quoted from the live gpuinfo Linux extensions page no longer match the page: all five figures have moved. | drift; not load-bearing; same reader misled | `docs/references.toml:2970` | block below | As L7-W02. | `confirmed`: `docs/references.toml:2970` quotes 93.65, 55.56, 27.21, 21.54, 11.34 %; `listextensions.php?platform=linux` today reads 93.61, 55.25, 26.71, 21.69, 11.42 %. All five moved. |  |
| L7-X01 | The `section` field of `apple-tech-talk-110147-busy-waits` dates the Tech Talk 2022; Apple's page publishes it 2021-12-07. | drift, not load-bearing (the quote and the code it backs are right; a reader dating the advice is misled by a year) | `docs/references.toml:3823` | block below | `section = "Apple Tech Talk 110147, Tune CPU job scheduling for Apple silicon games (2021), transcript, on busy waits"` | `confirmed`: `docs/references.toml:3823` dates the Tech Talk "(2022)". `https://developer.apple.com/videos/play/tech-talks/110147/` fetched today carries `datePublished" content="2021-12-07"` and `uploadDate" content="2021-12-07"`; the quote itself is on the page. |  |

### L7-A01

`docs/adr/0001-material-as-id-not-bind-group.md:3`, "Material as ID, not bind group": "Materials are entries in one structured GPU buffer, indexed by the Material ID that each Instance carries; the shader reads factors and Layers from it, so one bind group serves every material."

The Instance's Material ID is an override, `NO_MATERIAL` by default:

```
src/render/instance.rs:28:/// The Material ID of an Instance that overrides none of its Mesh's
src/render/instance.rs:29:/// Materials, and of a Primitive that names none.
src/render/instance.rs:30:pub const NO_MATERIAL: u32 = u32::MAX;
```

The culling pass picks the Meshlet's Primitive's Material ID unless the Instance overrides it, and writes it into the Meshlet record:

```
src/render/shaders/culling.wgsl:381:    let overridden = instance.material_id != NO_MATERIAL;
src/render/shaders/culling.wgsl:385:        let material_id = select(meshlets[meshlet_index].material_id, instance.material_id, overridden);
src/render/shaders/culling.wgsl:386:        meshlet_records[first.x + meshlet_in_mesh] = MeshletRecord(slot, meshlet_index, material_id);
```

The shader indexes the buffer with the record's ID, not the Instance's:

```
src/render/shaders/main.wgsl:113:    out.material_id = record.material_id;
src/render/shaders/main.wgsl:172:    let material = materials[in.material_id];
```

Fix to the text: "indexed by the Material ID the Meshlet record carries, its Primitive's unless the Instance overrides it".

### L7-A02

`docs/adr/0002-one-mega-buffer-for-vertices-one-for-indices.md:3`, "One shared buffer for vertices, one for indices": "Meshes are (vertex offset, index offset, index count) sub-allocations of one shared vertex buffer and one shared index buffer in the Mesh pool." The same sentence later names ADR 0017's refinement ("ADR 0017 replaces growth with fixed arenas") and no other.

The Mesh pool holds five Arenas, and the Mesh table addresses a Mesh by Meshlet ranges:

```
src/assets/pools.rs:57:/// The Meshes, and the GPU tables they are drawn from: Arenas of vertices
src/assets/pools.rs:58:/// and indices, the Meshlet Arenas and the Mesh table (ADRs 0017, 0019);
src/assets/pools.rs:74:    vertex_arena: Arena,
src/assets/pools.rs:75:    index_arena: Arena,
src/assets/pools.rs:76:    meshlet_arena: Arena,
src/assets/pools.rs:77:    meshlet_vertex_arena: Arena,
src/assets/pools.rs:78:    meshlet_triangle_arena: Arena,
src/assets/pools.rs:79:    mesh_table: wgpu::Buffer,
```

```
src/assets/mesh.rs:22:pub struct MeshData {
src/assets/mesh.rs:23:    pub first_meshlet: u32,
src/assets/mesh.rs:24:    pub meshlet_count: u32,
src/assets/mesh.rs:25:    pub first_triangle: u32,
src/assets/mesh.rs:26:    pub triangle_count: u32,
```

The index Arena is a BLAS input, not what the main pass draws from:

```
src/assets/pools.rs:105:            index_arena: arena("Index Arena", wgpu::BufferUsages::BLAS_INPUT, u32_bytes),
```

Fix to the text: add "Refined by ADR 0019: the Mesh table names each Mesh's Meshlet ranges in the Meshlet Arenas, which the main pass draws; the index Arena feeds the BLAS builds."

### L7-A03

`docs/adr/0013-reference-hardware-and-provisional-budgets.md:3`, "Reference hardware and provisional budgets": "Budgets are judged on an Apple M4 Pro until a Windows GPU machine exists; that machine gets its own ADR."

The machine exists, with Baselines on both its backends:

```
bench/baselines/nvidia-geforce-rtx-2070-super/town.json
bench/baselines/nvidia-geforce-rtx-2070-super-dx12/town.json
```

No ADR from 0017 to 0024 is about it (`docs/adr/` listing), and the glossary gives it another role:

```
CONTEXT.md:188:The recorded measurements of a Reference scene on one adapter and backend: the reference machine, where Budgets are judged against it; CI's lavapipe, where only its counters are judged; or the RTX 2070 SUPER, the hardware floor benched for comparison, on Vulkan and on DX12.
```

Fix to the text: "Budgets are judged on an Apple M4 Pro, the reference machine; the RTX 2070 SUPER is the hardware floor, benched for comparison on Vulkan and DX12 (`CONTEXT.md`, Baseline)", and either drop the promised ADR or say when it is due.

### L7-A04

`docs/adr/0013-reference-hardware-and-provisional-budgets.md:3`, "Reference hardware and provisional budgets": "where the 6 ms bounds the wall time of each pipeline stage's critical path, waits on worker threads included, and CPU time summed across threads is measured and reported but not budgeted until then".

Frame stats carry per-stage and per-frame wall time of the thread that ran the frame, and no thread-summed CPU time:

```
src/stats/mod.rs:11:    /// Wall-clock time on the CPU of each ECS system and renderer stage the
src/stats/mod.rs:12:    /// frame ran, in run order.
src/stats/mod.rs:13:    pub cpu_times: Vec<(String, Duration)>,
src/stats/mod.rs:14:    /// Wall-clock time from the start of `update` to the end of `render`.
src/stats/mod.rs:15:    pub cpu_frame_time: Duration,
```

`grep -rn "cpu_time\|getrusage\|process_cpu\|user_time\|across threads\|summed across" src/stats src/bench src/state.rs` finds no such measurement. "Until then" (the first Baseline) has also passed, by the same sentence's later amendment.

Fix to the text: "CPU time summed across threads is not yet measured; it joins Frame stats when a system first uses the pool, and is reported, not budgeted".

### L7-A05

`docs/adr/0014-world-positions-are-f64-the-gpu-works-camera-relative.md:19`, "Consequences": "The vertex shader composes camera-relative positions until draws are GPU-driven; then the culling pass does it once per Instance."

Draws are GPU-driven:

```
src/render/main_pass.rs:262:        depth_prepass.draw_indexed_indirect(frame_draw, 0);
```

The vertex shader still composes every vertex:

```
src/render/shaders/main.wgsl:95:    let position = camera_relative_position(
src/render/shaders/main.wgsl:96:        instance.anchor,
src/render/shaders/main.wgsl:97:        instance.offset,
src/render/shaders/main.wgsl:98:        camera.anchor,
src/render/shaders/main.wgsl:99:        camera.offset,
src/render/shaders/main.wgsl:100:        instance.rotation_scale * vertex.position
src/render/shaders/main.wgsl:101:    );
```

The culling pass composes only the bounds' centre, for its frustum and Hi-Z test; it writes no per-Instance camera-relative transform the vertex shader reads:

```
src/render/shaders/culling.wgsl:123:    let centre = camera_relative_position(
src/render/shaders/culling.wgsl:124:        instance.anchor,
src/render/shaders/culling.wgsl:125:        instance.offset,
src/render/shaders/culling.wgsl:126:        camera.anchor,
src/render/shaders/culling.wgsl:127:        camera.offset,
src/render/shaders/culling.wgsl:128:        instance.rotation_scale * (0.5 * (bounds_min + bounds_max))
src/render/shaders/culling.wgsl:129:    );
```

Fix to the text: "The vertex shader composes each vertex camera-relative from its Instance's and the camera's anchor and offset; the culling pass composes each Instance's bounds centre the same way for its tests." (ADR 0019 line 15 makes the same claim of the culling pass; that ADR is another reader's.)

### L7-A06

`docs/adr/0015-reverse-z-depth-with-an-infinite-far-plane.md:3`, "Reverse-Z depth with an infinite far plane": "the `far-coplanar` Reference scene's Golden turns red under standard depth."

The upper plane is red and the lower green:

```
src/scenes/mod.rs:596:const FAR_COPLANAR_COLORS: [[f32; 4]; 2] = [[0.8, 0.1, 0.1, 1.0], [0.1, 0.8, 0.1, 1.0]];
src/scenes/mod.rs:176:            Self::FarCoplanar => ["upper", "lower"]
src/scenes/mod.rs:177:                .into_iter()
src/scenes/mod.rs:178:                .zip(FAR_COPLANAR_COLORS)
src/scenes/mod.rs:606:    for (height, name) in [(0.0, "upper"), (-FAR_COPLANAR_GAP, "lower")] {
```

Under reversed depth the upper (red) plane covers the lower everywhere; under standard depth they interleave:

```
src/scenes/mod.rs:38:    /// centred 2 km in front of the camera, seen at a grazing angle: the upper plane covers the lower one everywhere
src/scenes/mod.rs:39:    /// under reversed depth, and they interleave under standard depth.
```

The committed Golden `tests/goldens/metal/far-coplanar.png` is a uniform red ground under the sky. So the Golden is red under reversed depth, the engine's depth, and would show red and green interleaving under standard depth.

Fix to the text: "the `far-coplanar` Reference scene's Golden is red, its upper plane covering the lower one everywhere; under standard depth the two interleave."

### L7-A07

`docs/adr/0016-the-main-thread-runs-the-frame.md:18`, "Consequences": "Per-frame data the GPU reads is written through `Queue::write_*`, which stages it and orders the copy before the next submit [...], or into a ring of mapped buffers whose slot is reused only after its `map_async` callback fires".

Per-frame data goes through a ring of mapped staging slots, chosen because `Queue::write_buffer` allocates fresh staging per call:

```
src/render/staging.rs:11:/// Staging memory for the per-frame uploads, reused every frame, where
src/render/staging.rs:12:/// `Queue::write_buffer` creates a fresh staging buffer on every call. Its
src/render/staging.rs:13:/// slots grow on demand, never past the most one frame uploads. Each frame
```

```
src/render/renderer.rs:549:        let uploaded_bytes = self
src/render/renderer.rs:550:            .frame_staging
src/render/renderer.rs:551:            .write(&self.device, &mut encoder, uploads);
```

`grep -rn "write_buffer\|write_texture" src/` finds `Queue::write_buffer` only in test modules (`src/render/depth_pyramid_pass.rs:253`, under `#[cfg(test)]` at `:129`; `src/render/shader_function_tests.rs:2222`, `:2246`). ADR 0017 line 19 states the rule the code follows ("Streaming uploads go through a `StagingBelt`, never `Queue::write_texture`"), and `CLAUDE.md:367-370` extends it to per-frame staging; ADR 0016 names neither.

Fix to the text: "Per-frame data the GPU reads is written through the per-frame staging ring (ADR 0017), a slot copied on the frame's encoder and reused only after its map callback fires; the GPU stats come back through a ring of mapped readback buffers on the same rule".

### L7-A08

`docs/adr/0016-the-main-thread-runs-the-frame.md:20`, "Consequences": "the main pass's draw count is the visibility strategy's question [...], which ADR 0019 answers with one `draw_indexed_indirect` per pipeline."

The depth pre-pass pipeline records one draw per Hi-Z phase:

```
src/render/main_pass.rs:253:        depth_prepass.set_pipeline(&self.depth_prepass_pipeline);
src/render/main_pass.rs:262:        depth_prepass.draw_indexed_indirect(frame_draw, 0);
src/render/main_pass.rs:290:        depth_pass.set_pipeline(&self.depth_prepass_pipeline);
src/render/main_pass.rs:296:        depth_pass.draw_indexed_indirect(frame_draw, PHASE_TWO_DRAW_OFFSET);
```

ADR 0019 says so itself:

```
docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:15: ... drawn with one `draw_indexed_indirect` per pipeline and Hi-Z phase, one native call on every backend.
```

Fix to the text: "which ADR 0019 answers with one `draw_indexed_indirect` per pipeline and Hi-Z phase".

### L7-B01

ADR 0017, Consequences, fourth bullet: "Acceleration structures join the ledger: 94 bytes per BLAS triangle (45.5 compacted) and 288.5 per TLAS instance on the M4 Pro, and on lavapipe (Mesa 25.2.8) 104 per BLAS triangle, which compaction leaves unchanged, and 240 per TLAS instance; `gpu_memory_bytes` counts them as estimates by backend until M4 sets exact sizes".

`src/render/acceleration_structures.rs:62-71`:

```rust
    /// The RTX 2070 SUPER's on Vulkan (NVIDIA 621.01): wgpu's allocator
    /// report around building and compacting BLASes of 2,048 to 131,072
    /// triangles, which compaction shrinks to 41%, and creating TLASes of
    /// 1,024 to 262,144 Instances, taken at the largest of each. On DX12 the
    /// same GPU measures within its 64 KiB placement granularity of these.
    const NVIDIA: Self = Self {
        per_blas_triangle: 70.7,
        per_compacted_blas_triangle: 29.2,
        per_tlas_instance: 238.2,
    };
```

`src/render/acceleration_structures.rs:73-85`:

```rust
    /// The estimate for `adapter`: NVIDIA's on an NVIDIA GPU, on any
    /// backend; lavapipe's on any other Vulkan adapter, software or not;
    /// and the M4 Pro's on any other, as the Budgets' reference machine
    /// (ADR 0013).
    pub fn estimate(adapter: &wgpu::AdapterInfo) -> Self {
        if adapter.vendor == NVIDIA_VENDOR {
            Self::NVIDIA
        } else if adapter.backend == wgpu::Backend::Vulkan {
            Self::LAVAPIPE
        } else {
            Self::M4_PRO
        }
    }
```

Fix to the text: add the RTX 2070 SUPER's 70.7, 29.2 and 238.2 as a third measured estimate and say the estimate is picked by vendor (NVIDIA's on any backend), then by backend (lavapipe's on other Vulkan adapters, the M4 Pro's otherwise).

### L7-B02

ADR 0017, Consequences, last bullet: "The cooked format (ADR 0011) stores each mip as a separately loadable byte range of the Texture's Blob, with the Resident tail as one range, and each Primitive's UV texel density (ADR 0018)."

`src/assets/cooked.rs:405-414`:

```rust
/// One Material's run of a Mesh's indices.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedPrimitive {
    pub first_index: u32,
    pub index_count: u32,
    pub material: Option<AssetId>,
    /// Its triangles in index order, each in exactly one Meshlet.
    pub meshlets: Vec<CookedMeshlet>,
}
```

`src/assets/cooked.rs:522-527` (what the Mesh Blob writes per Primitive):

```rust
        for primitive in &self.primitives {
            payload.extend(primitive.first_index.to_le_bytes());
            payload.extend(primitive.index_count.to_le_bytes());
            payload.extend(primitive.material.map_or(0, |id| id.0).to_be_bytes());
            payload.extend((primitive.meshlets.len() as u32).to_le_bytes());
        }
```

`grep -rn texel_density src/` finds nothing. Fix to the text: split the clause so the mip ranges stand as built and the texel density reads "and, once streaming lands, each Primitive's UV texel density (ADR 0018)".

### L7-B03

ADR 0018, status line: "Partly implemented: `praxis cook` and every dev launch cook Materials, Textures and Meshes into Blobs (…), and the runtime loads Materials and Textures (…) and Meshes (…) from them by Asset ID; the reader rebuilds their Meshlets (…), which the renderer does not keep yet."

`src/assets/pools.rs:74-78`:

```rust
    vertex_arena: Arena,
    index_arena: Arena,
    meshlet_arena: Arena,
    meshlet_vertex_arena: Arena,
    meshlet_triangle_arena: Arena,
```

`src/render/renderer.rs:367-374` binds them for the main pass:

```rust
                ("meshlets", mesh_pool.meshlet_arena().as_entire_binding()),
                (
                    "meshlet_vertices",
                    mesh_pool.meshlet_vertex_arena().as_entire_binding(),
                ),
                (
                    "meshlet_triangles",
                    mesh_pool.meshlet_triangle_arena().as_entire_binding(),
```

ADR 0017's own status line already records the three Meshlet Arenas. Fix to the text: "the reader rebuilds their Meshlets, which the Mesh pool keeps in three Arenas the main pass draws from (ADR 0017)".

### L7-B04

ADR 0018, Consequences, Meshes bullet: "Each Primitive stores its UV texel density, because each Primitive's Material has its own Textures (amends ADR 0017, which said per Mesh)."

Code as in L7-B02: `src/assets/cooked.rs:405-414` (`CookedPrimitive` has `first_index`, `index_count`, `material`, `meshlets`) and `:522-527` (the Blob writes those four). The Mesh Blob is at `MESH_FORMAT_VERSION` 2 (`src/assets/cooked.rs:37`) and holds no density. Fix to the text: "Each Primitive will store its UV texel density when streaming lands, because ..." or move the sentence to ADR 0017's planned streaming clause.

### L7-B05

ADR 0019, Consequences, third bullet: "The depth pre-pass writes 4x MSAA depth only; it seeds the Hi-Z pyramid and the lit pass's depth-equal test, and restores front-to-back cost when the culling pass writes draws out of order."

`src/render/main_pass.rs:114-125`:

```rust
        // With shadow rays on, the depth passes write each sample's surface
        // normals beside its depth, for the shadow mask pass.
        let depth_prepass_pipeline = if switches.shadow_rays {
            depth_pipeline(
                "Depth Pre-pass Pipeline",
                "vs_depth_normals",
                Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_surface_normals"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: targets::SURFACE_NORMALS_FORMAT,
```

`src/render/main_pass.rs:240-246`:

```rust
        let surface_normals = [Some(render_targets.surface_normals_attachment(true))];
        let mut depth_prepass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(DEPTH_PREPASS),
            color_attachments: if self.shadow_rays {
                &surface_normals
            } else {
                &[]
```

Shadow rays are on by default (`src/render/renderer.rs:144`). The summary paragraph's amendment states the normals; this Consequence was not re-read. Fix to the text: "The depth pre-pass writes 4x MSAA depth, and with shadow rays on each sample's surface normals (above); it seeds ...".

### L7-B06

ADR 0019, Consequences, third bullet: "With phase two's added, the shadow mask pass rebuilds each pixel's position and face normal from it, so shadow rays need no G-buffer".

`src/render/shaders/shadow_mask.wgsl:210-218`:

```wgsl
fn sample_surface(pixel: vec2<i32>, sample: u32, depth: f32, size: vec2<f32>) -> SampleSurface {
    let point = vec2<f32>(pixel) + sample_offset(sample);
    let position = position_at(camera.view_proj, point, depth, size);
    let normals = textureLoad(surface_normals, pixel, sample) * 2.0 - 1.0;
    ...
        octahedral_decode(normals.xy),
        octahedral_decode(normals.zw)
```

`src/render/targets.rs:91-93`:

```rust
    /// Each sample's surface normals, which the depth passes write with
    /// shadow rays on and the shadow mask pass reads.
    surface_normals: wgpu::TextureView,
```

The position is rebuilt from depth; the triangle and interpolated normals are read from a 4x `Rgba8Unorm` target the depth passes write. Fix to the text: "the shadow mask pass rebuilds each pixel's position from it and reads each sample's normals from the surface normals the depth passes write, so shadow rays need no G-buffer beyond that one attachment".

### L7-B07

ADR 0019, Consequences, fourth bullet: "Lights are culled into Froxels, each with its own light list built against the Froxel's bounds; today's up to four point lights become a light buffer."

`src/render/uniforms.rs:1-4`:

```rust
/// Lights the light buffer holds, directional and point lights together: a
/// fixed capacity, provisional until ADR 0013's split of the 8 GB Budget
/// sizes it.
pub const LIGHT_CAPACITY: u32 = 4096;
```

`src/render/froxel_pass.rs:1-2`: "The Froxel pass (ADR 0019): the compute pass that builds each Froxel's list of the point lights whose range reaches into its bounds". The summary paragraph already describes "a light buffer of fixed capacity". Fix to the text: drop the "today's up to four point lights" clause, or replace it with "the light buffer holds 4096 lights, directional first, provisional until the Budget split".

### L7-B08

ADR 0020, status line: "Partly implemented: each Mesh gets a compacted BLAS at load, and the static TLAS covers the static Instances ...".

`src/assets/pools.rs:19-27`:

```rust
    /// Drawn alone: a Mesh the GPU bends, whose BLAS, built from its rest
    /// shape, would cast another shape's shadow.
    Drawn,
    /// Traced alone: a shadow proxy, which no pass draws, so it keeps no
    /// Meshlets and needs no Material.
    Traced,
    /// Drawn and traced, a Mesh the GPU bends whose BLAS is updated each
    /// frame from its bent shape: built to allow updates, never compacted.
    Refitted,
```

`src/assets/pools.rs:341-343`:

```rust
            self.refitted_blases.push(mesh_blas);
        } else {
            self.awaiting_compaction.push((handle, blas_triangles));
```

The same paragraph later says the crowd's figures "have no BLAS", so the bent case is qualified four sentences on, but the refitted case is never said to be uncompacted. Fix to the text: "each Mesh rays trace gets a BLAS at load, compacted unless the crowd's refit form updates it each frame".

### L7-C01

Consequences, `docs/adr/0021-capability-floor.md:14`: "and 4x MSAA on every attachment format the main pass uses, read per format from the adapter's texture format features (`MULTISAMPLE_X4`), since no device feature covers it, with its resolve (`MULTISAMPLE_RESOLVE`) on the colour formats, since the lit pass resolves its 4x colour into the target [...] Those formats are `Depth32Float` and the HDR target's `Rgba16Float`; the window's surface and the capture target take only the tonemapping pass's single-sample output (amended 2026-10-22 [...])."

The floor checks two formats, `src/gpu/mod.rs:267-269`:

```rust
/// The attachment formats the main pass uses: depth, and the HDR colour.
const MAIN_PASS_FORMATS: [wgpu::TextureFormat; 2] =
    [crate::render::DEPTH_FORMAT, crate::render::HDR_FORMAT];
```

The main pass has a third, multisampled, colour attachment. `src/render/targets.rs:16`: `pub const SAMPLE_COUNT: u32 = 4;`; `:45`: `pub const SURFACE_NORMALS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;`; `:207-212`:

```rust
            surface_normals: target(
                "Surface Normals",
                SURFACE_NORMALS_FORMAT,
                SAMPLE_COUNT,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ),
```

The depth pre-pass and the phase-two depth pass attach it, `src/render/main_pass.rs:240-244`:

```rust
        let surface_normals = [Some(render_targets.surface_normals_attachment(true))];
        ...
            color_attachments: if self.shadow_rays {
                &surface_normals
```

and `:281`: `color_attachments: &[Some(render_targets.surface_normals_attachment(false))],`.

Fix to the text: list `Rgba8Unorm` beside the two, saying the floor's format check covers depth and the HDR colour while the surface normals target, drawn without a resolve, relies on 4x `Rgba8Unorm` being universal; or amend the sentence to "the formats the floor checks are ..." and name the omission.

### L7-C02

Consequences, `docs/adr/0021-capability-floor.md:14`: "So are `COMPUTE_SHADERS` and `INDIRECT_EXECUTION`: a compute pass builds the frame's draw and the main pass draws it with one `draw_indexed_indirect` (ADR 0019), which wgpu-core refuses without them" and, of `INDIRECT_FIRST_INSTANCE`, "which left the floor, since the one draw spans every Instance through packed indices and draws from first instance 0".

The main pass issues four indirect draws, `src/render/main_pass.rs:262`: `depth_prepass.draw_indexed_indirect(frame_draw, 0);`; `:296`: `depth_pass.draw_indexed_indirect(frame_draw, PHASE_TWO_DRAW_OFFSET);`; `:330`: `lit_pass.draw_indexed_indirect(frame_draw, PHASE_TWO_DRAW_OFFSET);`; `:333`: `lit_pass.draw_indexed_indirect(frame_draw, 0);`. The culling shader describes the same, `src/render/shaders/culling.wgsl:3-4`: "which the main pass draws with one indirect call per phase and depth pipeline, and one of both phases for the lit pipeline (ADR 0019)".

Fix to the text: "the main pass draws it with indirect draws, one per phase and pipeline, each from first instance 0".

### L7-C03

Considered options, `docs/adr/0021-capability-floor.md:10`: "so no carve-out is planned; its limits are not yet measured."

CI requests the floor's device on lavapipe on every run: `.github/workflows/ci.yml:58-90` (job `software-goldens`, Mesa `25.2.8-0ubuntu0.24.04.4` at `:29`, `cargo test` with `PRAXIS_REQUIRE_GOLDENS: "1"`), through `src/gpu/mod.rs:31-35`:

```rust
    pub async fn headless() -> anyhow::Result<Self> {
        let adapter =
            first_backend(async |wgpu_instance| request_adapter(wgpu_instance, None).await).await?;
        Self::request_device(adapter).await
    }
```

which fails below the floor (`:54-66`). The same ADR's Consequences (`:16`) quote lavapipe's `maxStorageBufferRange` and storage-buffer count from Mesa 25.2.8.

Fix to the text: drop "its limits are not yet measured", or replace it with "CI's software Vulkan jobs request the floor's device on lavapipe on every run".

### L7-C04

Consequences, `docs/adr/0022-fixed-64-hz-simulation-with-interpolation.md:18`: "The camera's rotation reads the newest input every frame, outside the fixed loop, so mouse-look keeps the frame's latency; its position follows whatever it is attached to, interpolated."

The camera is a resource, not an entity, and one per-frame system moves it. `src/camera/mod.rs:99`: `pub fn fly_camera(input: Res<Input>, time: Res<Time>, mut camera: ResMut<Camera>) {`; `src/state.rs:320`: `schedule.add_systems(fly_camera);`. No component or system in `src/` attaches the camera to an entity or interpolates its position from one (grep for `attach` in `src/camera`, `src/state.rs`, `src/components.rs` finds nothing).

Fix to the text: "its position is moved by input every frame today; a camera attached to an entity would follow that entity's blended World transform."

### L7-C06

Status, `docs/adr/0024-bench-history-is-git-stored-bench-records-keyed-by-engine-source.md:3`: "keyed by source hash × adapter × backend × scene × Harness version". Consequences, `:15`: "The key is source hash × adapter × backend × scene × Harness version." and `:16`: "A record path is its key, so a duplicate is a file that already exists."

The path has four parts, `src/bench/history.rs:204-212`:

```rust
    /// The record's path under `directory`, `ON_SERIES` or `TAGGED`: its
    /// key. The Harness version it holds is not in the path, as it is
    /// defined under `src/`, so a source hash names one.
    fn path(&self, directory: &str) -> PathBuf {
        Path::new(directory)
            .join(&self.adapter_key)
            .join(&self.subject)
            .join(format!("{}.json", self.source_hash))
    }
```

Fix to the text: "keyed by source hash × adapter × backend × scene; the Harness version is defined under `src/`, so the source hash names it, and the record holds it".

### L7-C07

Consequences, `docs/adr/0024-bench-history-is-git-stored-bench-records-keyed-by-engine-source.md:19`: "The 61 commits that touched `bench/baselines/` before the history existed are imported as records, each with the Harness version in force at its commit."

`src/bench/history/import.rs:19-22`:

```rust
/// The commit that started publishing Bench records: every Baseline
/// committed before it is imported, and every bench since has published
/// its own.
const HISTORY_STARTED: &str = "3c6b9d92";
```

At the frozen commit, `git log --oneline --first-parent 3c6b9d92^ -- bench/baselines | wc -l` prints `64`; the three newest (`cfb84b7c`, `3d4b21c3`, `6aec9278`) are dated 2026-10-31, the ADR's acceptance date, so the count moved after the sentence was written.

Fix to the text: "The commits that touched `bench/baselines/` before the history existed (64 at its first import) are imported as records", or drop the number.

### L7-C08

Consequences, `docs/adr/0024-bench-history-is-git-stored-bench-records-keyed-by-engine-source.md:20`: "Baselines are still chosen by hand, and a re-recorded Baseline's record is marked as one."

Only imported Baselines carry a mark, `src/bench/history.rs:258-260`:

```rust
    /// `baseline-commit` for an imported Baseline; none for a bench's
    /// result.
    source: Option<String>,
```

and the import stops at `HISTORY_STARTED`, `src/bench/history/import.rs:19-22` (quoted under L7-C07): "every bench since has published its own". A Baseline re-recorded after that is a bench's record with no mark; whether a pinned commit is covered by a Baseline is read from git, `src/bench/history/verdicts.rs:424-432`:

```rust
            pinned.covered_by_baseline = !git(
                repository,
                &[
                    "diff",
                    "--name-only",
                    &format!("{commit}^"),
                    commit,
                    "--",
                    &baseline.join("/"),
```

The page's legend agrees (`docs/bench/index.html:168`: "A Baseline record, imported from `main`'s history").

Fix to the text: "Baselines are still chosen by hand; a Baseline imported from before the history is marked as one, and a later one is recognised by the commit that changed its file."

### L7-G01

Term and sentence, `CONTEXT.md:199-200`:

> **Series**:
> One metric's Bench records for one Reference scene on one adapter and backend, taken on `main`'s source by a quiet machine (lavapipe's, which read no GPU time, from any machine), in source order. What its charts draw and what a Regression is found in.

Code. The branch's README, which `praxis publish` writes, `src/bench/history.rs:58-61`:

> - `series/<adapter>-<backend>/<scene>.json`: one row per record in `records/`,
>   in the order their commits were made, every one rebuilt by each publish. A
>   row holds the counter frames' counters by name and the measured frames'
>   under `measured_counters`.

`src/bench/history.rs:420`:

> /// One Series file's row: a record's source and its metrics, without the

`src/bench/history.rs:350-351`:

> // A Series is one adapter's, and lavapipe's times mean nothing, so
> // none of its records is a suspect.

`src/bench/history.rs:386-397` judges two timings over the one Series' rows (`JUDGED_TIMINGS`, `cpu_frame_microseconds` and `gpu_frame_microseconds`), and the Regression issue names a Series as a scene on an adapter, `src/bench/history/verdicts.rs:710`:

> - Series: `{scene}` on `{adapter_key}`\n\

The bench page loads one Series per machine and scene, `docs/bench/index.html:392` and `:399-400` (`seriesPoints(machine, scene)`, `series/${machine}/${scene}.json`). Nothing in the code is a Series of one metric; the per-metric object is the "line" of `suspects.rs`, which the glossary does not define.

What the text should say: a Series is the Bench records of one Reference scene on one adapter and backend, in source order, holding every metric; its charts draw one metric at a time and a Suspect or Regression is found in one metric of it. The bench-history guide's tooltip and its glossary entry (`docs/guides/bench-history.html:319`, `:463`) copy this sentence and move with it.

### L7-G02

Term and sentence, `CONTEXT.md:212`:

> Only the records since the last Harness version, scene content or imported Baseline judge it.

Code. `src/bench/history.rs:356-364` computes each row's `starts_line`:

> let starts_line: Vec<bool> = rows
>     .iter()
>     .enumerate()
>     .map(|(index, row)| {
>         (index > 0 && !rows[index - 1].same_definition(row))
>             || row.source == Some(import::IMPORTED_SOURCE)
>             || line_starts.contains(row.source_hash)
>     })

and `line_starts` is the set of sources a Verdict pinned, `src/bench/history.rs:1025-1027`:

> let pinned =
>     verdicts::pinned_sources(&self.directory.join(verdicts::VERDICTS).join(series))?;
> self.series_lines::<StoredRecord>(directory, &pinned)?

The README states the rule, `src/bench/history.rs:81-83`:

> ... A new Harness
> version or scene content, an imported Baseline, or a commit a Verdict pinned
> starts a line. lavapipe's records are never suspects.

The glossary's own Verdict term agrees ("which then starts its Series' line", `CONTEXT.md:216`); the Suspect term's "Only ... or imported Baseline" leaves the pinned commit out.

What the text should say: "Only the records since the last Harness version, scene content, imported Baseline or commit a Verdict pinned judge it." The guide's Suspect tooltip and its "Line" entry (`docs/guides/bench-history.html:347`, `:465`) carry the same omission.

### L7-G03

Term and sentence, `CONTEXT.md:224`:

> The GPU memory Budget splits into one Budget per kind of GPU memory, such as the geometry Arenas, the texture pool and staging, which caps what that kind may hold (ADR 0017).

Code. The Budgets file holds one GPU memory figure, `bench/budgets.json:1-8`:

> {
>   "cpu_frame_microseconds": 6000,
>   "gpu_frame_microseconds": 12000,
>   "gate_microseconds": 6000,
>   "dynamic_tlas_cpu_microseconds": 1100,
>   "dynamic_tlas_gpu_microseconds": 1000,
>   "gpu_memory_bytes": 8000000000
> }

and `src/bench/mod.rs:375-389` (`pub struct Budgets`) has the same six fields. The capacities the sentence says the split caps are provisional constants whose comments say the split has not happened, `src/assets/pools.rs:7-9`:

> /// Bytes of each Arena: one provisional capacity for the app, tests, bench
> /// and CI until ADR 0013's split of the 8 GB Budget replaces it.
> pub const ARENA_CAPACITY_BYTES: u64 = 16 << 20;

likewise `src/render/instance.rs:1-4` (`INSTANCE_CAPACITY`), `src/render/froxel_pass.rs:24-26` (`FROXEL_LIGHT_CAPACITY`), `src/render/acceleration_structures.rs:12` and `:17-19`. ADR 0017 places the split in the future (`docs/adr/0017-*.md:18`: "The 8 GB of ADR 0013 splits into Budgets for geometry arenas, the texture pool, Resident tails, render targets, acceleration structures and staging; M4 sets their sizes").

What the text should say: that the GPU memory Budget is one number today and is to split into one Budget per kind, whose sizes a later milestone sets (ADR 0017); until then each kind's capacity is provisional.

### L7-G04

Term and sentence, `CONTEXT.md:224`:

> A numeric limit on one metric of a Reference scene (CPU ms, GPU ms, GPU memory, Input latency) that a change must not regress.

Code. `bench/budgets.json:1-8` (quoted under L7-G03) and `src/bench/mod.rs:375-389` hold no Input latency Budget, and `grep -rn -i "input latency\|input_latency" src bench` finds nothing: no Frame stat measures it (`src/stats/mod.rs:8-105`). ADR 0022 defers it (`docs/adr/0022-*.md:21`: "judged at p95 once Input latency is measured"), and the glossary's own Input latency term has no code (verdict `planned` above).

What the text should say: the metrics the Budgets file holds (CPU ms, GPU ms, the gate, the dynamic TLAS's CPU and GPU ceilings, GPU memory), with Input latency named as a planned Budget.

### L7-G05

Term and sentence, `CONTEXT.md:88`:

> ... in a slot of the Instance buffer that the Renderable keeps while it is one.

Code. `src/render/prepare.rs:135-138`:

> /// Each Renderable's slot in the Instance buffer, kept until it stops being a
> /// Renderable or is not drawn; a slot freed in an earlier frame is taken
> /// again before a fresh one, and once all `INSTANCE_CAPACITY` are taken a
> /// Renderable is refused one.

`src/render/prepare.rs:358-372`:

> let Some(mesh) = mesh_pool.get(mesh_ref.0) else {
>     tracing::warn!(
>         handle = mesh_ref.0.0,
>         "MeshHandle not in pool, skipping entity"
>     );
>     instance_slots.free(entity);
>     unwritten.insert(entity);
>     return;
> };
> if material_ref.is_none() && mesh.needs_material_handle {
>     tracing::warn!(
>         ?entity,
>         "Renderable has no Material Handle and a Primitive of its Mesh names no Material, skipping it"
>     );
>     instance_slots.free(entity);

Such an entity still has a Transform and a Mesh Handle, so it is a Renderable by `CONTEXT.md:10`, yet its slot is freed and may be taken by another Renderable the same frame. The code's own summary says so, `src/render/prepare.rs:244-247`: "one whose Mesh is not in `mesh_pool`, or that has no Material Handle where a Primitive names no Material, is skipped and frees its slot."

What the text should say: the Renderable keeps the slot while it is one and is drawn; a Renderable prepare skips frees its slot and takes one again when it draws.

### L7-G06

Term and sentence, `CONTEXT.md:112`:

> Where a directional light's bit at the nearest surface differs from a 4-neighbour's, a shadow edge, or where the nearest sample turns from that light while another of the surface's samples faces it, that light is traced again from each other sample that shows the surface, and the fragment shades it by the share of its samples that see it.

Code. `src/render/shadow_mask_pass.rs:48-52`:

> /// Directional lights, the first in the light buffer, whose shadow edges the
> /// pass traces again from each sample: the shadow mask samples hold each
> /// one's verdict at every sample, SAMPLE_COUNT bits a light, above the
> /// samples that show the second surface. Any other keeps one ray per pixel.
> pub(crate) const SHADOW_EDGE_LIGHTS: u32 = 7;

`src/render/shaders/shadow_mask.wgsl:461-463`:

> let edge_light_count = min(light.directional_light_count, SHADOW_EDGE_LIGHTS);
> let hidden = nearest.word.x & ((1u << edge_light_count) - 1u);

and the per-sample turn test loops `for (var index = 0u; index < SHADOW_EDGE_LIGHTS; index++)` (`src/render/shaders/shadow_mask.wgsl:582`). An eighth directional light keeps one ray per pixel. ADR 0020 states the cap ("one of the first seven directional lights", `docs/adr/0020-*.md:3`); the glossary says "a directional light".

What the text should say: "Where one of the first seven directional lights' bit ... that light is traced again ...; any other directional light keeps the pixel's one verdict."

### L7-G07

Term and sentence, `CONTEXT.md:120`:

> A small group of one Primitive's triangles, bounded in vertex and triangle count, that the GPU draws as a unit and can cull as one.

Code. A Meshlet carries nothing to cull by, `src/assets/mesh.rs:37-44`:

> pub struct MeshletData {
>     pub first_vertex: u32,
>     pub first_triangle: u32,
>     pub triangle_count: u32,
>     pub material_id: u32,
> }

and the culling pass tests the Instance's Mesh bounds, appending every Meshlet of a drawn Instance, `src/render/shaders/culling.wgsl:8-10` ("With the CULLING Render switch on, they draw only the Instances whose Mesh bounds reach into the view frustum") and `:382-386` (a loop over `mesh.meshlet_count` writing one record per Meshlet). ADR 0019 defers the capability (`docs/adr/0019-*.md:15`: "Culling tests Instances; tests per Meshlet wait for a profile that shows the pass raster-bound").

What the text should say: the GPU draws a Meshlet as a unit; culling tests its Instance's Mesh bounds, and a per-Meshlet test is ADR 0019's deferred step.

### L7-G08

Term and sentence, `CONTEXT.md:215-217`:

> **Verdict**:
> What the sweep found of one slower Suspect, kept on `bench-history`: held when a night's interleaved re-bench of it and the record before it still shows half its shift and 3%, else cleared. ...
> _Avoid_: Confirmation, bisect result

Code gives the word two other public meanings. `src/bench/mod.rs:400-403`:

> pub enum Verdict {
>     Warn,
>     Fail,
> }

used by `praxis compare` (`src/bin/praxis.rs:657`: `failed += usize::from(finding.verdict == Verdict::Fail);`) and `src/bench/mod.rs:851-854`; and `src/preflight/mod.rs:59-67`:

> /// Whether a check passed, and whether its result blocks `verify`.
> #[derive(Clone, Copy, Debug, PartialEq)]
> pub enum Verdict {
>     Ok,
>     /// Information, such as work in flight, that passes or fails nothing.
>     Note,
>     /// Worth acting on before a bench, but `verify` still passes.
>     Warn,

used at `src/bin/praxis.rs:362`. The sweep's Verdict is `src/bench/history/verdicts.rs:79-101`, whose definition the glossary matches; the glossary is "the vocabulary of code, tickets, and reviews" (`CLAUDE.md`, Documents), and a reader meeting `Verdict::Fail` in compare's output or `preflight`'s finds the glossary defining the word as something else.

What the text should say: keep the sweep's definition and add, in the term or its `_Avoid_` line, that compare's and preflight's verdicts are one metric's or one check's warn or fail, not this; or define both senses.

### L7-G09

Term and sentence, `CONTEXT.md:239-241`:

> **Finding**:
> One defect at one location, filed by an Audit lane's reader in eight fields: ...
> _Avoid_: Issue (a ticket), observation, note

Code. `src/bench/mod.rs:405-411`:

> /// A metric that regressed against its Baseline or exceeds its Budget.
> #[derive(Debug, PartialEq)]
> pub struct Finding {
>     pub metric: &'static str,
>     pub verdict: Verdict,
>     pub detail: String,
> }

built at `src/bench/mod.rs:647-649` and `:819-821`, printed by `praxis compare` (`src/bench/mod.rs:413-420`, "warn ..." or "FAIL ...") and counted at `src/bin/praxis.rs:657`. `scripts/verify.sh` runs compare, so every verify prints bench Findings that have three fields, not eight, and no Audit lane.

What the text should say: keep the audit definition and add, in the term or its `_Avoid_` line, that compare's Finding is one metric past its Baseline or Budget; or define both senses. With L7-G08 this is the same slip twice: a new glossary term took a name a public item already had.

### L7-R01

`CLAUDE.md:190-194`, Project shape:

```
tests/                         # integration tests at agreed seams
  goldens/<backend>[-cpu]/     # committed Goldens; PRAXIS_UPDATE_GOLDENS=1
                                 cargo test --test goldens rewrites them;
                                 README.md shows the metal ones, and
                                 cargo test checks it lists each scene
```

The tree: `tests/goldens/` holds `metal/`, `vulkan/` and `vulkan-cpu/` only (`ls tests/goldens`); `find tests -name README.md` finds nothing. The test the clause names reads the repository root's README: `tests/goldens.rs:386-388`:

```
fn readme_shows_each_scene_golden() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = std::fs::read_to_string(root.join("README.md")).expect("README.md");
```

and its own doc says so, `tests/goldens.rs:25-26`: "`README.md` shows each scene's Metal Golden by linking it, and a test here fails when its links and the scenes checked disagree." The root `README.md:5,105-112` links `tests/goldens/metal/<scene>.png` for the nine scenes. In a tree-shaped listing a bare `README.md` under `tests/goldens/` reads as that directory's file. The shape should say "the root README.md shows the metal ones", or move the sentence beside `tests/`.

### L7-R02

`CLAUDE.md:277`, Code conventions: "Bevy imports: `use bevy_ecs::prelude::*;` (not piecemeal)."

The tree, shipping code:

- `src/state.rs:4-5`: `use bevy_ecs::prelude::*;` then `use bevy_ecs::system::RunSystemOnce;`
- `src/threading.rs:5-6`: `use bevy_ecs::prelude::*;` then `use bevy_ecs::schedule::ExecutorKind;`
- `src/time/mod.rs:3-4`: `use bevy_ecs::prelude::*;` then `use bevy_ecs::schedule::{LogLevel, ScheduleBuildSettings};`

(and `src/render/renderer.rs:810`, `src/state.rs:349` in test modules). The bevy_ecs 0.18.1 prelude (`~/.cargo/registry/src/*/bevy_ecs-0.18.1/src/lib.rs`, `pub mod prelude`) exports from `schedule` only `common_conditions::*, ApplyDeferred, IntoScheduleConfigs, IntoSystemSet, Schedule, Schedules, SystemCondition, SystemSet` and from `system` a list without `RunSystemOnce`; `ExecutorKind`, `LogLevel`, `ScheduleBuildSettings` and `RunSystemOnce` are not in it. The code is right: the prelude cannot supply these. The convention should read "`use bevy_ecs::prelude::*;` for what the prelude exports; an item outside it by its path, not piecemeal copies of prelude items". No test enforces the rule, and the text claims none.

### L7-R03

`CLAUDE.md:278`, Code conventions: "WGPU types: full path (`wgpu::Buffer`), don't import individual types."

The tree, shipping code: `src/render/blas_refit.rs:174` (inside a function): `use wgpu::hal::{self, CommandEncoder as _, api::Metal};` and `src/render/blas_refit.rs:287` (inside `#[cfg(target_vendor = "apple")] mod metal`): `use wgpu::hal::{self, Device as _, api::Metal};`. `api::Metal` is a type (wgpu-hal's backend marker, used as `device.as_hal::<Metal>()` at line 492 and 497). The `as _` imports are traits brought in for their methods, which the rule's word "types" does not cover but a reviewer may read as covered. Elsewhere `src/render/shader_tests.rs:10` imports the module `wgpu::naga`, and `src/render/coverage_tests.rs:13` the trait `wgpu::util::DeviceExt` (both test-only). `grep -rn "^\s*use wgpu::" src tests` finds no other shipping import. The convention should name the exceptions it tolerates, or narrow itself to wgpu's own resource and descriptor types. No test enforces the rule, and the text claims none.

### L7-R04

`CLAUDE.md:387-388`, Review checklist: "Workflows keep least-privilege permissions and pinned action versions".

Permissions hold: `.github/workflows/ci.yml:20-21` `permissions: contents: read`, with `contents: write` only on `publish-bench` (`ci.yml:220-221`); `.github/workflows/pr-lint.yml:14-16` `contents: read`, `pull-requests: read`.

Versions do not hold in one reading and are inconsistent in every reading. `.github/workflows/ci.yml:37,39,43` (repeated at 63-67, 101-107, 151-155, 223, 225, and `actions/upload-artifact@v4` at 87, 138, 187, 200):

```
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
```

`@v4` and `@v2` are moving major tags; `@stable` is a branch that moves with every toolchain release, not a version at all. `.github/workflows/pr-lint.yml:29` pins the same action by commit: `uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4.4.0`. A reviewer applying the item to `ci.yml` cannot tell whether it passes. The checklist should say what pinned means for this repo (a commit SHA with the version in a comment, as `pr-lint.yml` does), and, if `dtolnay/rust-toolchain@stable` is kept on purpose so CI tracks stable Rust, name that exception.

### L7-S01

Registry quote (`docs/references.toml:218-222`, `khronos-dfs-srgb`):

> The corresponding sRGB EOTF⁻¹ conversion from linear (R, G, B) encoding to non-linear (R′, G′, B′) encoding is: R′ = R × 12.92, R ≤ 0.0031308; 1.055 × R^(1/2.4) − 0.055, R > 0.0031308

Source opened: `https://registry.khronos.org/DataFormat/specs/1.3/dataformat.1.3.html#TRANSFER_SRGB_INVEOTF`, §13.3.2 sRGB EOTF⁻¹, which says exactly that; the decode's constants are in §13.3.1 sRGB EOTF (`#TRANSFER_SRGB_EOTF`), "R = R′ / 12.92, R′ ≤ 0.04045; ((R′ + 0.055) / 1.055)^2.4, R′ > 0.04045", which is the registry's `khronos-dfs-srgb-eotf` entry (`docs/references.toml:224-228`).

Code at `src/render/shader_function_tests.rs:1169-1177`:

```rust
/// The inverse of IEC 61966-2-1's sRGB encoding `khronos-dfs-srgb`, in
/// f64.
fn srgb_decode(encoded: f64) -> f64 {
    if encoded <= 0.04045 {
        encoded / 12.92
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}
```

The comment is true as prose (the function is the inverse of the encoding), but the key it cites quotes only the encoding, so the knee 0.04045 and the exponent 2.4 have no quoted source at this site, while the registry holds an entry that states them. Fix: cite `khronos-dfs-srgb-eotf` here (the same formula at `src/bench/mod.rs:284-291` already does), or widen the comment to say the constants follow from inverting the cited quote.

### L7-U01

Opened: `https://www.idee.es/csw-inspire-idee/srv/spa/csw?service=CSW&version=2.0.2&request=GetRecordById&outputSchema=http://www.isotc211.org/2005/gmd&elementSetName=full&id=spaignMDT05` (HTTP 200, by WebFetch and by curl, 2026-11-02).

Registry quote (`docs/references.toml:1010`):

> `https://centrodedescargas.cnig.es/CentroDescargas/busquedaSerie.do?codSerie=MDT05 [...] https://servicios.idee.es/wcs-inspire/mdt?request=GetCapabilities&service=WCS&version=2.0.1`

Source, every `gmd:URL` under `gmd:distributionInfo` / `gmd:onLine`:

> `https://servicios.idee.es/wms-inspire/mdt?request=GetCapabilities&service=WMS&VERSION=1.3.0`
> `https://servicios.idee.es/wmts/mdt?request=GetCapabilities&service=WMTS&VERSION=1.0.0`
> `https://servicios.idee.es/wcs-inspire/mdt?request=GetCapabilities&service=WCS&version=2.0.1`
> `https://centrodedescargas.cnig.es/CentroDescargas/modelo-digital-terreno-mdt05-primera-cobertura`

The second quoted url matches; the first does not occur anywhere in the record (`grep -c busquedaSerie` is 0). The record's `gmd:dateStamp` is 2026-10-14, twelve days before the entry's `checked = 2026-10-26`, so the record was already in this form when checked unless its publisher edits records without moving the stamp. Fix: replace the first url in the quote with the `modelo-digital-terreno-mdt05-primera-cobertura` linkage the record holds, and re-date `checked`. The citing row `docs/research/test-assets.md:27` links `https://centrodedescargas.cnig.es` without the path, so it needs no change.

### L7-W01

Registry, `docs/references.toml:1945-1949`:

```
["apple-mtldevice-current-allocated-size"]
url = "https://developer.apple.com/documentation/metal/mtldevice/currentallocatedsize"
section = "Apple Metal documentation, MTLDevice.currentAllocatedSize, abstract (macOS 10.13 and later)"
quote = '''The total amount of memory, in bytes, the GPU device is using for all of its resources.'''
```

Source opened: `https://developer.apple.com/tutorials/data/documentation/metal/mtldevice/currentallocatedsize.json` (the data behind the url), abstract: "The total amount of memory, in bytes, the GPU device is using for all of its resources." Platforms: macOS 10.13. The quote is confirmed; what is missing is the line the preamble asks for. `grep -c '^\["'` gives 581 entries and `grep -c '^checked'` 580; this is the one without. No test enforces the field (`tests/references.rs` names no field). Fix: add `checked = 2026-11-02` (or the date it was first read) to the entry.

### L7-W02

Registry quote (`docs/references.toml:2964`), section "Extensions, Windows, Recent (1y) filter, read 2026-10-27":

> VK_EXT_memory_budget: 91.03 % [...] VK_EXT_memory_priority: 78.13 % [...] VK_EXT_pageable_device_local_memory: 49.72 % [...] VK_NV_memory_decompression: 31.78 % [...] VK_EXT_memory_decompression: 21.5 %

Source opened 2026-11-02: `https://vulkan.gpuinfo.org/listextensions.php?platform=windows`, with the page's own age filter showing `<option value="recent" selected>Recent (1y)</option>`:

> VK_EXT_memory_budget 91.03 % [...] VK_EXT_memory_priority 78.32 % [...] VK_EXT_pageable_device_local_memory 49.35 % [...] VK_NV_memory_decompression 31.59 % [...] VK_EXT_memory_decompression 21.31 %

The page is a live database whose percentages move with every report uploaded; the entry dates its reading but its url pins nothing, so the quote cannot be confirmed at the url a week later. The research note that cites it (`docs/research/gpu-memory-residency-streaming.md:40`) repeats the 91.03 % figure. Fix: point the url at a web.archive.org snapshot taken on the day read (as `metal-feature-set-tables-sparse-textures` and `ue-5.6-texture-streaming-pool` do), or quote the figures as approximate ("about 91 %") so the claim survives churn; the sibling entry `gpuinfo-vulkan-ray-query-coverage` (another module's) at least carries `&age=recent` in its url, which this family's urls omit.

### L7-W03

Registry quote (`docs/references.toml:2940`): `sparseResidencyImage2D: 76.64 %: 23.36 %`.
Source opened 2026-11-02, `https://vulkan.gpuinfo.org/listfeaturescore10.php?platform=windows`, Recent (1y) selected: `sparseResidencyImage2D 77.01 % 22.99 %`. Same cause and fix as L7-W02.

### L7-W04

Registry quote (`docs/references.toml:2946`): `sparseResidencyImage2D: 72.11 %: 27.89 %`.
Source opened 2026-11-02, `https://vulkan.gpuinfo.org/listfeaturescore10.php?platform=linux`, Recent (1y) selected: `sparseResidencyImage2D 71.92 % 28.08 %`. Same cause and fix as L7-W02.

### L7-W05

Registry quote (`docs/references.toml:2958`): `DEVICE_LOCAL_BIT LAZILY_ALLOCATED_BIT: 3.85 %: 96.15 %`.
Source opened 2026-11-02, `https://vulkan.gpuinfo.org/listmemory.php?platform=linux`, Recent (1y) selected: `DEVICE_LOCAL_BIT LAZILY_ALLOCATED_BIT 3.88 % 96.12 %`. Same cause and fix as L7-W02. (The Windows twin, `gpuinfo-lazily-allocated-windows`, still reads 5.05 % / 94.95 % today and is confirmed, but will drift the same way.)

### L7-W06

Registry quote (`docs/references.toml:2970`):

> VK_EXT_memory_budget: 93.65 % [...] VK_EXT_memory_priority: 55.56 % [...] VK_EXT_pageable_device_local_memory: 27.21 % [...] VK_NV_memory_decompression: 21.54 % [...] VK_EXT_memory_decompression: 11.34 %

Source opened 2026-11-02, `https://vulkan.gpuinfo.org/listextensions.php?platform=linux`, Recent (1y) selected:

> VK_EXT_memory_budget 93.61 % [...] VK_EXT_memory_priority 55.25 % [...] VK_EXT_pageable_device_local_memory 26.71 % [...] VK_NV_memory_decompression 21.69 % [...] VK_EXT_memory_decompression 11.42 %

Same cause and fix as L7-W02.

### L7-X01

Registry, `docs/references.toml:3823`:

> section = "Apple Tech Talk 110147, Tune CPU job scheduling for Apple silicon games (2022), transcript, on busy waits"

Source, https://developer.apple.com/videos/play/tech-talks/110147/ (page metadata, read with curl; the transcript read with WebFetch):

> `<meta ... "datePublished" content="2021-12-07">` and `"uploadDate" content="2021-12-07"`; the transcript says the M1 was introduced "last year" and the M1 Pro and M1 Max "this year".

The quote itself is verbatim:

> Avoid busy waits. They potentially lock a P core, instead of doing something useful with it. They also prevents the scheduler from promoting a thread from and E to a P core. You are also wasting energy and producing unnecessary heat, eating away your thermal headroom.

and `src/capture/mod.rs:283-286` claims only what it says. Fix: change "(2022)" to "(2021)" in the section field; nothing else in the entry or its code sites moves.

### L7-C05 (killed)

Consequences, `docs/adr/0023-lights-are-photometric-as-gltf-defines-them.md:12`: "Shading runs in f32 and pre-exposes colour before fp16 stores it, and every path clamps it to 65504, since on overflow the M4 Pro saturates while Vulkan may store infinity".

The fragment path clamps, `src/render/shaders/main.wgsl:305`: `let exposed = min(luminance * camera.exposure, vec3<f32>(LARGEST_HALF));`, and the resolve path clamps, `src/render/shaders/hdr.wgsl:31`. The clear path does not: `src/render/prepare.rs:301-307`:

```rust
        let background = environment.background_luminance * exposure;
        frame.background = wgpu::Color {
            r: background.x.into(),
            g: background.y.into(),
            b: background.z.into(),
            a: 1.0,
        };
```

which the lit pass clears the 4x colour target to, `src/render/main_pass.rs:318-321`: `Some(render_targets.color_attachment(background)),`. Today's backgrounds stay far below 65504 (`src/scenes/mod.rs:464`), so no Golden shows it.

Fix to the text: "the lit pass's fragment output and the resolve clamp it to 65504; the clear colour is the scene's background times exposure, bounded by scene data".

## Rule candidates

Each candidate is a slip a reader saw twice, with the ids of every Finding of its kind that survived the verifier and the one-sentence rule that would have caught it; the rules ticket decides, each rule backed by at least two Findings. The verifier read every candidate against its ids and marked each `stands` or `falls`; none fell. The research note on staleness checks (`docs/research/stale-comment-checks.md`) weighs mechanical checks on identifiers and numbers in comments; none of the candidates below is of that kind, since every Finding here is a prose clause that went false, which only a re-read at the PR that changed the code catches.

| rule | finding ids | status |
|---|---|---|
| The PR that lands an ADR which changes how an earlier ADR's clause is done adds "Refined by ADR n" to that clause in the same PR. | L7-A02, L7-A07 | stands (L7-A07 narrowed, still a missing refinement) |
| A PR that lands a pass greps the ADRs for the pass's name and rewrites every clause that predicted what it would do. | L7-A05, L7-A08, L7-C02 | stands; L7-C02, ADR 0021's "one `draw_indexed_indirect`", is a third of the same kind, added by the verifier |
| An "until <event>" clause in an ADR is rewritten by the PR that lands the event, which the PR's body names. | L7-A03, L7-A04 | stands |
| A sentence that says what a Blob holds is checked against the Blob's Rust struct and its format version when either changes, and a field no struct holds is written as planned. | L7-B02, L7-B04 | stands |
| An amendment that changes what a pass writes or reads re-reads every Consequence that names that pass and amends it or dates it. | L7-B05, L7-B06 | stands |
| A PR that lands deferred work greps the ADRs for "today" and "yet" clauses naming it and removes or dates them. | L7-B03, L7-B07 | stands |
| A PR that adds a render attachment, a render target format or an indirect draw re-reads ADR 0021's enumerations in the same PR, and `MAIN_PASS_FORMATS` is asserted against the targets module's attachment formats by a test. | L7-C01, L7-C02 | stands |
| An ADR states a count or a "not yet" only with the date or commit it was true at, or rewords it so it cannot go stale. | L7-C03, L7-C07 | stands |
| Before a glossary term lands, `grep -rn "pub \(struct\|enum\) <Term>\b" src` runs, and a public item of the same name with another meaning is named in the term's `_Avoid_` line or given its own sense. | L7-G08, L7-G09 | stands |
| A glossary sentence that states what an ADR decides for a later milestone writes it as the ADRs do, "Planned; today ...", so the "today" half is what the code holds. | L7-G03, L7-G04 | stands |
| An import or naming convention in `CLAUDE.md` names its exceptions or is enforced by a conventions test; a convention with neither is reworded to what the tree does. | L7-R02, L7-R03 | stands (L7-R02 narrowed to exactly the missing exception) |
| A figure quoted from a live database page is cited through an archived snapshot url of the day it was read, or quoted as approximate. | L7-W02, L7-W03, L7-W04, L7-W05, L7-W06 | stands |

No two readers proposed the same rule, so none merge. The "Refined by ADR n" rule (A02, A07), the "deferred work lands" rule (B03, B07), the "until <event>" rule (A03, A04) and the "count or not-yet" rule (C03, C07) are four forms of one slip, a status clause not moved by the PR that made it false; the rules ticket may fold them into one.

## Killed

The Findings the verifier killed, 1 of 46, each with
its reason; their evidence blocks are above, marked killed.

| id | claim | reason |
|---|---|---|
| L7-C05 | ADR 0023 says "every path clamps" exposed colour to 65504; the lit pass's clear colour, the background luminance times exposure, reaches the colour target unclamped | The code does what the doc says. `docs/adr/0023-lights-are-photometric-as-gltf-defines-them.md:12`: "every path clamps it to 65504". The clear colour the reader named unclamped (`src/render/prepare.rs:301-307`) reaches the colour target through `RenderTargets::color_attachment`, `src/render/targets.rs:336-341`, which clamps each channel: `let largest_half = 65504.0; ... .map(|channel| channel.min(largest_half))` before weighting it for the resolve. The reader cited the call at `src/render/main_pass.rs:318-321` without opening the function. |
