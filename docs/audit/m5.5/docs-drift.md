# Audit lane 7, docs and rules drift

The lane's report at `M5.5`, in the form `docs/agents/audit.md` gives:
`CONTEXT.md`, `CLAUDE.md`, `docs/agents/*` and ADRs 0001 to 0027 read whole
against the code at the frozen commit
`e5c55a8da2a59db2f09c99c8ea4522987d5354ac`, the code winning, with a
verdict per glossary term, per rule and workflow clause, and per ADR status
line, Consequence, count and "today", "until" or "yet" clause read. The
ADRs' decision clauses are lane 1's. The scope is what landed on `main`
after `M3.5`'s frozen commit `8bc34eb18fcfb79307193551e96619316aef566c`: a
Finding is filed where that code made a sentence false, or where `M3.5`'s
lane 7 missed one. The reference half is skipped, as M5's exit ran it on
the same documents (`docs/audit/m5/sources.md`); no registry entry was
opened against its source. Three readers at high effort, each with one
shared brief that quoted the lane brief verbatim: A on the ADRs, G on the
glossary, R on `CLAUDE.md` with `docs/agents/*`. One adversarial verifier
reopened every Finding and 24 `matches` rows whose code landed in scope.
Finding ids carry the reader's letter. Every `where` is a `path:line` at
the frozen commit. Severity is `drift` throughout, as the lane brief sets;
the magnitude says whether the sentence is load-bearing and whom it
misleads. The three leads the bevy_ecs 0.20 bump's review handed to this
lane are judged in the ADRs' Notes: the second is L7-A03; the first and
third hold, in a research note and in test code, outside this lane's
documents.

## Summary

The glossary, the rules and the ADRs say what the code at the frozen commit does almost everywhere: of 304 ADR rows, 79 glossary terms and 195 rule and workflow clauses read, the readers filed 16 Findings, the verifier confirmed 12, narrowed 3 and killed 1, none of the 15 left is load-bearing, and every sentence on which `M3.5`'s lane 7 placed a Finding in these documents now says what the code does. Twelve of the fifteen concern work that landed in scope, eleven of them work that landed after the Review checklist gained its item asking a change to grep the ADRs and `CONTEXT.md` for what it renames, removes or changes (2026-11-04); the other three (L7-G04, L7-A06, L7-A08) are sentences `M3.5`'s lane missed. Five Rule candidates stand, each on two Findings and all forms of `M3.5`'s one slip, a sentence not moved by the change that made it false: a figure derived from a struct's size, a status one ADR borrows from another, work moved to another thread or machine, a member added to a set the glossary lists, and a clause that presumes unbuilt what has landed.

## Counts

Every Finding is `drift`, the lane's class; no reader found cause for
another, and none is load-bearing. The verifier reopened 24 `matches` rows
whose code landed in scope (8 glossary terms, 7 rules, 9 ADR rows); every
one held, so it filed no Finding of its own.

| Read | Rows | Findings filed | Surviving |
|---|---:|---:|---:|
| ADR rows (status lines, Consequences, counts, "today", "until", "yet"), 0001 to 0027 | 304 | 9 | 8 |
| Glossary terms | 79 | 4 | 4 |
| `CLAUDE.md` clauses | 91 | 1 | 1 |
| `docs/agents/*` clauses | 104 | 2 | 2 |

| Severity class | Findings |
|---|---:|
| `drift` | 15 |
| `image`, `budget`, `correctness`, `health` | 0 |

| Verifier verdict | Findings |
|---|---:|
| `confirmed` | 12 |
| `narrowed` | 3 |
| `killed` | 1 |
| filed | 16 |

## ADRs 0001 to 0027

I read the 27 files `docs/adr/0001-*.md` to `docs/adr/0027-*.md` whole in the worktree at the frozen commit `e5c55a8da2a59db2f09c99c8ea4522987d5354ac`, and gave a verdict row to every status line, amendment, Consequence, count and "today", "until", "yet", "still" or "no longer" clause; decision clauses, ADR 0026's *Decision* and *Settled by* sections among them, got a row only where they carry a count. The scope was `git log 8bc34eb1..HEAD` (238 commits, 70 of them touching `docs/adr/`) and `git diff 8bc34eb1 HEAD -- docs/adr/`. Prior art read first: `docs/audit/m3.5/docs-drift.md` (its three ADR sections, its Findings L7-A01 to L7-C08, its Rule candidates and Killed) and `docs/audit/m5/sources.md` (its ADR 0026 and 0027 rows, its amendments rows, its Findings, its Killed and "Left for a full lane 7"); a row whose sentence those reports filed names the Finding and whether its fix holds. Code opened: `src/render/{renderer,prepare,instance,main_pass,staging,targets,uniforms,acceleration_structures,blas_refit,culling_pass,shadow_mask_pass,froxel_pass,depth_pyramid_pass,tonemapping_pass,readback}.rs`, `src/render/shaders/{main,culling,common,shading,shadow_mask}.wgsl`, `src/assets/{pools,material,cooked,loader}.rs`, `src/cook/{mod,mesh,far_proxy,texture}.rs`, `src/cook/far_proxy/bake.rs`, `src/streaming/{mod,io,mips,chunks}.rs`, `src/stats/{mod,gpu_memory}.rs`, `src/bench/{mod,provenance,history}.rs`, `src/bench/history/verdicts.rs`, `src/gpu/mod.rs`, `src/threading.rs`, `src/time/mod.rs`, `src/transform/mod.rs`, `src/components.rs`, `src/camera/mod.rs`, `src/scenes/{mod,world,town}.rs`, `src/state.rs`, `src/lib.rs`, `src/bin/praxis.rs`, `src/capture/mod.rs`, `bench/budgets.json`, `bench/baselines/` (listing), `Cargo.toml`, `Cargo.lock`, `vendor/README.md`, `scripts/sweep.plist`, `.github/workflows/ci.yml` (Mesa pin), `docs/agents/workflow.md:13`, `:59`, `docs/research/job-systems-and-frame-threading.md` (lead 1's rows), and from `~/.cargo/registry/src/` at the pinned versions bevy_ecs 0.20.1 (`query/par_iter.rs`, `schedule/executor/{mod,multi_threaded}.rs`, `schedule/schedule.rs`, `lib.rs`), bevy_tasks 0.20.1 (`usages.rs`) and meshopt 0.7.0 (`Cargo.toml`). Commit bodies read with `git show` for the slot size change, the RTX sweep's stop, the large-class view and the meshopt bump. Greps run over `src/`, `tests/`, `scripts/`: `517_376`/`488_448`/`136.2`/`PER_MILLE`, `par_iter`/`ComputeTaskPool`/`get_or_init`, `Schedule::default()`, `as_hal`/`wgpu::hal`, `queue.write_buffer`/`write_texture`, `error_scope`, `qos`, `std::thread`/`submit` in the acceleration structures, `alpha_to_coverage`, `interpolate(flat)`, `emissive_strength`, `SUN_ILLUMINANCE`, `MESHLET_MAX_*`, `LOD_THRESHOLD_PIXELS`, `SHADOW_MASK_LIGHTS`, `SHADOW_EDGE_LIGHTS`, `DYNAMIC_TLAS_CAPACITY`, `INSTANCES_PLACED_PER_FRAME`, `LOADS_PER_FRAME`, `IO_THREADS`, `MULTISAMPLED_FORMATS`, `FLOOR_*`, `MEASURED_PATHS`, `rerecord`, `busy`, `needs-triage`, and the ADRs for "today", "until", "yet", "still", "no longer" and "provisional". Every registry quote an ADR cites was left alone, as the brief skips the reference half; measurements, history and hardware are marked not checkable.

### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0001 | "indexed by the Material ID that each Meshlet record carries, its Primitive's unless the Instance overrides it" | matches (the m3.5 L7-A01 fix holds) | `src/render/shaders/culling.wgsl:594-600`, `src/render/shaders/common.wgsl:305-309`, `src/render/shaders/main.wgsl:308` |
| 0001 | "Implemented in Phase 6 Step 2 (a8d188e)" | not checkable from the tree (history) | — |
| 0002 | "Implemented in Phase 6 Step 1 (fc98395), growing by CPU re-upload; ADR 0017 replaces growth with fixed arenas ... each at most 2^31 − 1 bytes" | matches (history; no growth today) | `src/assets/pools.rs:240-255` |
| 0002 | "at a provisional 16 MiB each until the Budget split; each Arena is now sized at device creation from the geometry Arenas' GPU memory Budget, in the share ADR 0017 gives it" | matches | `src/assets/pools.rs:15-19`, `:82-83`; `bench/budgets.json:11` |
| 0002 | "Refined by ADR 0019: the Mesh table names each Mesh's Meshlet ranges in three further Arenas ... the index Arena feeds only the BLAS builds and refits" | matches (the m3.5 L7-A02 fix holds; index Arena `BLAS_INPUT` only) | `src/assets/pools.rs:250-254`, `:699` |
| 0003 | "each Renderable keeps a slot ... until it stops being a Renderable, and a freed slot is taken again before a fresh one" | matches (a skipped Renderable keeps its slot) | `src/render/prepare.rs:530-538`, `:735-739` |
| 0003 | "past which a Renderable is refused, warned about once, counted in Frame stats and retried every frame" | matches | `src/render/prepare.rs:536-538` |
| 0003 | "the number is sized at device creation from the scene and pass buffers' GPU memory Budget ... 30% of it ... in whole 256-slot scan blocks" | matches | `src/render/renderer.rs:203-204`, `:259-261`; `src/render/culling_pass.rs:67` |
| 0003 | "at 136.2 bytes a slot, its Instance, offsets and visibility bit ... 517,376 slots at the Budgets M4 set" | drift (L7-A01) | `src/render/renderer.rs:213-218`, `:1092`; `src/render/instance.rs:33-40` |
| 0003 | "prepare writes only the slots of new Renderables and of those whose World transform, Mesh or Material changed, found by bevy_ecs change detection, through the per-frame staging path" | matches | `src/render/prepare.rs:82-91`, `:565-570` |
| 0003 | "A freed slot is written an Instance with no Mesh ... taken again from the next frame" | matches | `src/render/prepare.rs:686-693` |
| 0003 | "Refined by ADR 0014: the buffer stores an integer anchor plus an f32 offset" | matches | `src/render/instance.rs:36-38` |
| 0003 | "Refined by ADRs 0022 and 0020: prepare also rewrites every Moving Instance each frame" | matches (`With<PreviousWorldTransform>` in the changed query) | `src/render/prepare.rs:82-91`, `:539-542` |
| 0003 | "On the frame after the Mesh or Material pool frees an entry, prepare writes again every Renderable's Instance and the Placements of each drawn Chunk one of whose Placements names an entry no longer in its pool ... free ranges, free slots and held Chunks ... are not written again" | matches (checked at M5 too) | `src/render/prepare.rs:670-733` |
| 0003 | "A Placement has no entity: each loaded Chunk owns a contiguous range of slots ... before new Renderables take theirs, and kept until it unloads ... its point lights join the frame's lights" | matches | `src/render/prepare.rs:549-563`, `:694-733`, `:643-654` |
| 0003 | "A Placement whose Handles find nothing ... is written an Instance with no Mesh and ... not retried" | matches | `src/render/prepare.rs:555-556` |
| 0003 | "A Chunk the buffer has no room for is refused, warned about once and its Placements counted as refused every frame. An entity's slot is still per entity" | matches | `src/render/prepare.rs:351-381`, `:713-718` |
| 0003 | "A range is the first free range long enough, else fresh slots past the highest taken; an unloaded Chunk's range ... joins the free ranges the next frame, merged with those it touches" | matches | `src/render/prepare.rs:351-381`, `:406-423` |
| 0003 | "A held Chunk keeps its range, written Instances with no Mesh until it is drawn again, and its point lights stay out of the frame's" | matches | `src/render/prepare.rs:552-554`, `:725-728` |
| 0004 | "a prepare pass builds Frame data (the Instances that changed, lights, camera) ... it lists no draws" | matches | `src/render/prepare.rs:525-572` |
| 0004 | "Frame data is an owned, `Send` value with no borrow into the ECS World" | matches | `src/render/prepare.rs:24` |
| 0004 | "Today prepare (`src/render/prepare.rs`) fills Frame data from the ECS World on the main thread" | drift (L7-A02) | `src/render/prepare.rs:168-170`, `:187-245`, `:254-262`, `:489` |
| 0004 | "the `Renderer`, which owns the Mesh and Material pools, draws it without reading the ECS World" | matches | `src/render/renderer.rs:332-333` |
| 0005 | "Implemented as the instance buffer in Phase 5" | not checkable from the tree (history) | — |
| 0005 | "Refined by ADR 0014: the model matrix is relative to an integer anchor, which the Instance also carries" | matches | `src/render/instance.rs:33-40` |
| 0005 | "the lookup key is the Instance slot in the Meshlet record a packed index names, not `@builtin(instance_index)`" | matches | `src/render/shaders/common.wgsl:305-309`, `src/render/shaders/main.wgsl:118`, `:224` |
| 0006 | "`Transform::to_affine`, `Bend::drawn_curvature` and `WorldTransform::{translation, drawn}` are of this kind" | matches (and the other methods on components, `Transform::from_translation`, `Sway::curvature`, are pure too) | `src/components.rs:27`, `:34`, `:74`; `src/transform/mod.rs:12`, `:26`; `src/scenes/mod.rs:1314-1320` |
| 0006 | "Behaviour lives in systems or, later, script handlers"; "Pure-data components serialise cleanly for save/load" | systems: matches; scripts, save/load: planned | `Cargo.toml` (no `mlua`) |
| 0007 | "using `wgpu-hal` only for a subsystem that exceeds wgpu's safe API; migration to raw Vulkan through `ash` is not planned" | matches (`as_hal` only for BLAS size queries and the Metal refit; `ash` only as wgpu-hal's dependency) | `src/render/acceleration_structures.rs:122-130`, `src/stats/gpu_memory.rs:383-393`, `src/render/blas_refit.rs:166-279`; `Cargo.lock:114` |
| 0007 | "Consoles are out of scope (0010)" | not checkable from the tree (scope) | — |
| 0007 | Consequences: "Each milestone exit's dependency review (`docs/agents/workflow.md`, The ladder) checks three things" | matches (M5's L7-B01 fixed: the clause now names the three conditions) | `docs/agents/workflow.md:13` |
| 0007 | "`vendor/` carries more than five upstream commits, which `vendor/README.md` lists" (a trigger) | matches (one commit listed; trigger not met) | `vendor/README.md:22-31` |
| 0007 | "The exit is an owned fork ... where on 2026-11-14 a copy of wgpu-core 30.0.1 carries one upstream fix" | matches as dated (the copy is 30.0.2 since; M5's L7-B02 killed it as a dated clause, and nothing changed under that reason) | `vendor/README.md:13`; `Cargo.toml:54` |
| 0008 | "Anti-aliasing is spatial (MSAA, SMAA) ... the main pass reads none (ADR 0019)" | matches (4x MSAA; no history target) | `src/render/targets.rs` (no previous-frame target) |
| 0008 | "Lighting is ray-traced probe GI with hard ray-traced shadows" | shadows: matches; probe GI: planned (a constant ambient stands in) | `src/render/shaders/shading.wgsl:261-267` |
| 0008 | "the main pass is clustered forward with 4x MSAA (ADR 0019)" | matches | `src/render/targets.rs:16` |
| 0009 | "The ECS is `bevy_ecs` used on its own, not the Bevy engine ... ADR 0014 replaces that newtype" | matches (`bevy_ecs`, `bevy_tasks`, `bevy_mikktspace` only) | `Cargo.toml:17-20`; `Cargo.lock:190-350`; `src/components.rs:9-17` |
| 0009 | "considered and rejected on 2026-04-28" | not checkable from the tree (history) | — |
| 0010 | "single-player games on Windows, Linux and macOS ... wgpu's desktop backends are enough"; "Accepted 2026-10-12" | matches (dx12, metal, vulkan; no web backend); the date is not checkable | `Cargo.toml:8` |
| 0011 | "the runtime loads Blobs with mip chains, GPU block compression (BC) and byte ranges sized for streaming" | matches | `src/assets/cooked.rs`, `src/streaming/io.rs` |
| 0011 | "since then a development launch reaches glTF only through the cooker, which cooks what changed before the runtime loads Blobs" | matches (a dev launch cooks each Asset it loads that is missing or stale) | `src/state.rs:205`; `src/cook/mod.rs:695-707`; `Cargo.toml:16`, `:41` |
| 0011 | "Textures were uncompressed RGBA8 without mips when this was accepted"; "Accepted 2026-10-12" | not checkable from the tree (history) | — |
| 0011 | "Textures cook into KTX2, a standard container" | matches | `Cargo.toml:25` |
| 0012 | "Planned; sequenced by the milestone ladder" | matches (status; no `mlua` in the lock) | `Cargo.lock` |
| 0013 | "Budgets are judged on an Apple M4 Pro ...; the RTX 2070 SUPER is the hardware floor, benched for comparison on Vulkan and on DX12" | matches (m3.5's L7-A03 fixed); Baselines on both backends | `bench/baselines/nvidia-geforce-rtx-2070-super/`, `bench/baselines/nvidia-geforce-rtx-2070-super-dx12/` |
| 0013 | "it gates regressions, the Budgets being judged on it" | matches (compare judges times on the Baseline's adapter) | `src/bench/mod.rs:1252-1261` |
| 0013 | "On `town` at one source the M4 Pro's GPU frame reads 2.2 times ... (the Bench records of source hash `6a91ca76`)" | not checkable from the tree (Bench records; M5 confirmed them) | — |
| 0013 | "60 fps at 1440p native with CPU at most 6 ms and GPU at most 12 ms" | matches | `bench/budgets.json:2-3` |
| 0013 | "CPU time summed across threads is neither measured nor budgeted yet, Frame stats holding wall-clock times per system, stage and frame only" | matches (m3.5's L7-A04 fixed) | `src/stats/mod.rs:16-20` |
| 0013 | "the CPU frame, the GPU frame and ADR 0019's gate answer to their Budgets at the p99 as at the median, the CPU frame at its p95 too, and no CPU or GPU frame lasts longer than 16,667 µs" | matches | `src/bench/mod.rs:1270-1281`, `:1310-1318`, `:1331-1334`; `bench/budgets.json:8` |
| 0013 | "Input latency at most 2 frame periods ... judged at p95" | planned (no Input latency measure) | `bench/budgets.json` |
| 0013 | "100k visible Instances, about 5M visible triangles, a 10 by 10 km streamed Game world, a 2 km LOD'd view distance (drawn past the Load radius as a Far proxy per Chunk, ADR 0026)" | matches (`world` is 40 Chunks of 256 m a side, 10.24 km) | `src/scenes/world.rs:26-27`; `src/assets/cooked.rs:58` |
| 0013 | "GPU memory at most 8 GB ... summed by the engine itself on every backend, because wgpu has no allocator report on Metal (29.0.1 and 30.0.1 ...)" | matches (the versions are the dated reason; 30.0.2 pinned since) | `bench/budgets.json:9`; `src/stats/gpu_memory.rs:145-205` |
| 0013 | "the GPU stats readback's buffers are left out ...; so are the target the frame is presented into ... and capture's read-back buffer" | matches (created on the device, not through the ledger) | `src/render/readback.rs:194`; `src/capture/mod.rs:332`, `:465` |
| 0013 | "the 8 GB splits into Budgets for geometry arenas, the texture pool, Resident tails, render targets, acceleration structures and staging ...; a seventh Budget covers the scene and pass buffers" | matches | `bench/budgets.json:10-18` |
| 0013 | "83,886,080 bytes for geometry arenas, 4,294,967,296 ..., 234,881,024 for the scene and pass buffers, 5,955,911,680 in all" | matches (each figure, and the sum) | `bench/budgets.json:11-17` |
| 0013 | "each capacity is its share of its kind's Budget at its real bytes per entry, as ADR 0017 gives them" | matches | `src/render/renderer.rs:225-277`; `src/assets/pools.rs:15-19` |
| 0013 | "At M3's exit the Budgets were re-decided ... (CPU frame 5.80 ms ..., GPU memory 764 MB)" | not checkable from the tree (history; m3.5 checked it) | — |
| 0013 | "and so do the dynamic TLAS's ceilings of 1.1 ms of CPU and 1.0 ms of GPU" | matches | `bench/budgets.json:5-6` |
| 0013 | "A budget moves down freely and up only through an ADR or a map comment (`docs/agents/workflow.md`)" | matches | `docs/agents/workflow.md:59` |
| 0013 | guard: "both culling phases' per-slot stages (`offset_slots`, `offset_blocks` and `append_records`, timed apart from `append_triangles`) ... `culling_per_slot_microseconds` ... `praxis compare` fails a scene past it" | matches | `src/render/culling_pass.rs:44-55`, `:163-165`; `bench/budgets.json:7`; `src/bench/mod.rs:1349-1361` |
| 0013 | "the Bench record's counters hold the Instance slots in use" | matches | `src/bench/mod.rs:336`, `:384` |
| 0014 | "World positions are f64 ... `Transform` holds translation as `DVec3` ... 64 bytes" | matches (after the glam 0.34 bump) | `src/components.rs:9-17` |
| 0014 | "Implemented: `Transform` and the camera hold f64 positions ... point lights reach the GPU as an anchor plus an offset ... A child's Transform composes ... Each Instance and the camera reach the GPU as an anchor ... a shader-function oracle holds the composition" | matches (unchanged code; m3.5 checked each) | `src/render/instance.rs:33-40`; `src/transform/mod.rs` |
| 0014 | Consequences: "The vertex shader composes each vertex camera-relative ...; the culling pass composes each Instance's bounds centre the same way for its tests" | matches (m3.5's L7-A05 fixed) | `src/render/shaders/main.wgsl:28-38`; `src/render/shaders/culling.wgsl:133-151` |
| 0014 | "Worst cases, measured separately, sum to 0.041 mm ..." | not checkable from the tree (measurement) | — |
| 0014 | "The physics engine, not yet chosen" | planned (no physics crate) | `Cargo.toml` |
| 0014 | "The static TLAS's frame is the corner of the camera's Anchor when its first static Instance was written, and a build moves it only once the camera's Anchor strays more than a Chunk's edge (4 cells) along an axis" | matches | `src/render/acceleration_structures.rs:190-207`, `:572-576` |
| 0014 | "Lua receives positions as numbers ... Saves store f64" | planned | — |
| 0014 | "A Chunk is a vertical column addressed by integer (x, z), with its origin at (x·edge, 0, z·edge) ... cooked meshes stay within 128 m of their pivot, which the cooker enforces" | matches | `src/assets/cooked.rs:58-69`; `src/cook/mesh.rs:92-94`, `:163-173` |
| 0014 | "A Far proxy's Mesh is exempt ... within a Chunk's edge of it ... placed unrotated, and is drawn from the Load radius outwards unless its Chunk is late" | matches (M5 checked it; unchanged since) | `src/streaming/chunks.rs:118-127` |
| 0014 | "World-anchored shader effects (noise, wind, triplanar mapping) derive their periodic coordinates from the anchor" | planned (no such effect in `src/render/shaders/`) | — |
| 0014 | "One Game world is loaded at a time" | matches | `src/components.rs:9-17` |
| 0015 | "With a 0.1 m near plane and `Depth32Float` ... Depth is cleared to 0 and compared with `Greater` ... with no far plane" | matches | `src/camera/mod.rs:92-103`; `src/render/targets.rs` |
| 0015 | "shadow comparisons follow the same direction" | not checkable from the tree (no shadow map; shadows are rays) | — |
| 0015 | "`projection_matrix` in `src/camera/` is the one projection the window, capture and bench paths use" | matches (the one non-test caller is prepare) | `src/camera/mod.rs:100`; `src/render/prepare.rs:606` |
| 0015 | "the `far-coplanar` Reference scene's Golden pins it: red, the upper plane covering the green lower one everywhere" | matches (m3.5's L7-A06 fixed) | `tests/goldens/metal/far-coplanar.png` |
| 0016 | "The main thread runs winit, input, the simulation schedule, prepare and command recording, then submits and presents" | matches | `src/state.rs:328-333`; `src/render/acceleration_structures.rs:530-546` |
| 0016 | "wgpu-hal 30.0.1's Metal acquire sends `-[NSView window]`" | not checkable from the tree (registry quote; wgpu-hal 30.0.2 is pinned now, the clause names its version) | `Cargo.lock` |
| 0016 | status: "the `multi_threaded` feature is on, the compute pool is created with `available_parallelism() − 1` workers (at least 1), one pool per State's `FramePreparer`, and the schedule stays on the `SingleThreadedExecutor`" | matches | `Cargo.toml:17-18`; `src/threading.rs:11-28`; `src/render/prepare.rs:489`, `:520` |
| 0016 | status: "prepare fills the Renderables' Instances on the pool once there are enough to split among its threads, and no system uses it yet" | matches | `src/render/prepare.rs:168-170`, `:220-224` |
| 0016 | status: "the IO pool is built (below); the rest is planned" | drift (L7-A04) | `src/render/acceleration_structures.rs:204-207`, `:592-614` |
| 0016 | Considered options (render thread, OS thread, job graph) | not checkable from the tree (rejected options) | — |
| 0016 | "Frame data (ADR 0004) is an owned, `Send` value ... Trigger: a Reference scene's main-thread critical path exceeds 6 ms while the pool is saturated" | matches; the trigger is a rule | `src/render/prepare.rs:24` |
| 0016 | "bevy_ecs's `multi_threaded` feature is on, for `par_iter` and scoped pool tasks ..., while the schedule keeps the `SingleThreadedExecutor`" | matches (the feature is on; the stated reason holds though no `par_iter` is in `src/`) | `Cargo.toml:17`; `src/threading.rs:21-28` |
| 0016 | "the multi-threaded executor costs 0.4 to 1.2 µs per system per run" | not checkable from the tree (measurement) | — |
| 0016 | "The compute pool is created explicitly with `available_parallelism() − 1` workers, since the main thread joins scopes ... No QoS class is set" | matches (no `qos` in `src/` or `scripts/`) | `src/threading.rs:11-19`; `src/render/prepare.rs:254-262` |
| 0016 | "The compute pool is a `TaskPool` that each State's `FramePreparer` owns, not the process-global `ComputeTaskPool` ... A State's own pool starts its queue with the State" | matches | `src/render/prepare.rs:487-489`, `:520` |
| 0016 | "`world`'s counter frames read 10,474 heap allocations ..." | not checkable from the tree (measurement; M5 confirmed it from the issue) | — |
| 0016 | "`par_iter` runs on the process-global pool ..., which the engine no longer creates; the first system to use it decides where prepare's tasks then run" | drift (L7-A03; lead 2) | `~/.cargo/registry/src/*/bevy_ecs-0.20.1/src/query/par_iter.rs:103-105`; `bevy_tasks-0.20.1/src/usages.rs:29-38`; `src/threading.rs:24-28` |
| 0016 | "The IO pool is four std threads ... They take Chunk reads first, then mips, then Far proxy reads ... on all threads but one" | matches | `src/streaming/io.rs:38-42`, `:306-316` |
| 0016 | "They decode into CPU memory, and the main thread copies it into the `StagingBelt`" | matches (load staging is a `StagingBelt` at the frozen commit) | `src/render/staging.rs:179-187` |
| 0016 | "Decoding stays on those threads: run as compute-pool tasks, it settled ... 1.1% and 5.4% slower ..." | not checkable from the tree (measurement) | — |
| 0016 | "Only the main thread submits, presents or polls the wgpu `Queue` and `Device` ... Workers record `CommandEncoder`s and return `CommandBuffer`s; they push no error scopes" | matches (the static TLAS worker records and returns its build, the main thread submits it; no `push_error_scope` in `src/`) | `src/render/acceleration_structures.rs:206`, `:530-546`, `:592-614` |
| 0016 | "`desired_maximum_frame_latency` stays 2" | matches | `src/lib.rs:166`; `src/render/mod.rs` (`FRAMES_IN_FLIGHT`) |
| 0016 | "Per-frame data the GPU reads is written into the per-frame staging ring (ADR 0017) ...; `Queue::write_*` ... stages outside the ring" | matches (m3.5's L7-A07 fixed; no shipping `Queue::write_buffer`) | `src/render/staging.rs:8-31`; `src/render/depth_pyramid_pass.rs:250-252` (a test) |
| 0016 | "Lua (ADR 0012) runs in one state on the main thread" | planned | — |
| 0016 | "Recording 100k draws ... 5.1 to 6.9 ms ..." | not checkable from the tree (measurement) | — |
| 0016 | "which ADR 0019 answers with one `draw_indexed_indirect` per pipeline and Hi-Z phase" | matches as m3.5's L7-A08 fix worded it (the lit pipeline's one draw spans both phases; each thin twin draws per phase) | `src/render/main_pass.rs:315-368` |
| 0017 | status: "each Channel is a partially bound binding array of texture arrays, one per Texture class ... loaded from cooked Blobs through a `StagingBelt` and sampled trilinear, the colour Channels at 16x anisotropy and the data Channels at 4x" | matches | `src/assets/material.rs:19-24`; `src/render/staging.rs:179-187` |
| 0017 | status: "a Texture a Chunk loads arriving as its Resident tail, its finer mips streaming in by the priority below through the IO pool, while one a spawn loads keeps its full mip chain ... a Far proxy's atlas arriving with its mips down to the one its distance from the camera needed" | matches | `src/streaming/io.rs`; `src/streaming/mips.rs:302-330` |
| 0017 | status: "a provisional 16 MiB each until #625 sized them from the Budget" | matches (history, dated) | `src/assets/pools.rs:74-99` |
| 0017 | status: "each Arena and table at its share of its kind's Budget below, and each Channel a Texture class per 64 KiB of the texture pool's fifth, up to what the Capability floor binds" | matches | `src/assets/pools.rs:74-99`; `src/stats/gpu_memory.rs:107`; `src/assets/material.rs:19-20` |
| 0017 | Considered options (software VT, sparse, virtual geometry, growable buffers) | not checkable from the tree (rejected designs) | — |
| 0017 | "A class's slice count is fixed when it opens, at most the Capability floor's 256 array layers (ADR 0021) ... an empty slice still holds memory, which `gpu_memory_bytes` counts" | matches (a large class's view spans only the slices placed since the last landed change; its array, and its count, are whole) | `src/assets/material.rs:24`; `src/gpu/mod.rs:305-315` |
| 0017 | "per Texture, 10,005 `useResource` calls cost 12.8 to 13.5 ms ... against texture-wall's 13.8 to 14.3 ms gate" | not checkable from the tree (measurement) | — |
| 0017 | "Residency changes are batched so that bind group is rebuilt at most once per frame" | matches (one `refresh_bind_group` per render) | `src/assets/material.rs:1238-1250`; `src/render/renderer.rs:813` |
| 0017 | "Measured in #153 at the line ... M4's charting chose that survey's option B" | not checkable from the tree (history) | — |
| 0017 | "Priority is computed on the CPU ... from distance and the Primitive's UV texel density (stored at cook time ...); Chunks inside the load radius are requested ahead. No GPU feedback readback" | matches | `src/streaming/mips.rs:302-330`; `src/assets/cooked.rs:510` |
| 0017 | "The visible Instances are the drawn Placements of the loaded Chunks and the Renderables ... frustum aside" | matches | `src/streaming/io.rs:30-35` (`MipRenderableQuery`) |
| 0017 | "judged at the nearest point of the sphere around its Mesh's bounds, at its smallest scale, against the footprint of a pixel at the corner of a 16:9 view 1,440 pixels tall" | matches | `src/streaming/mips.rs:21-31`, `:312-313`, `:419-421` |
| 0017 | "The mips of the Chunks and Far proxies held past their radius ... are requested as the drawn ones' are and are never late ... judged from where the camera ... will be the Chunk streamer's least lead of frames later, at most a Chunk edge nearer ...; eviction ranks a Texture by that same need, while the late mips are still judged from where the camera is" | matches | `src/streaming/mips.rs:49`, `:293-330`; `src/streaming/mod.rs:47` |
| 0017 | "The Materials rays reach off screen ... take the Resident tail as their floor ... texels span 10.4 to 15.6 mm ... A finer floor waits for ... a ray count M6 chooses" | mechanism not re-checked (unchanged since M4's exit); the figures are not checkable; the probe rays are planned | — |
| 0017 | "Each texture's Resident tail (mips of 64 KiB and smaller) never leaves ... (87,408 bytes for a 2,048² BC7 or BC5 Texture, 1.6% of its chain)" | matches (the arithmetic: 65,536 + 16,384 + 4,096 + 1,024 + 256 + 64 + 3 × 16) | `src/stats/gpu_memory.rs:107` |
| 0017 | "a mip loads at priority p and is evicted only below p − 1 ... least recently used breaking ties" | matches | `src/streaming/mips.rs:293-299` |
| 0017 | "each at most 2^31 − 1 bytes ... sub-allocated with the `offset-allocator` crate ... the Capability floor's 128 MiB `max_storage_buffer_binding_size` caps that arena ... No growth at runtime ... The CPU shadow ... goes" | matches (code unchanged in kind since m3.5's check) | `src/assets/pools.rs` |
| 0017 | "the vertex, index, Meshlet, Meshlet vertex and Meshlet triangle Arenas take 65.8%, 13.0%, 1.1%, 13.3% and 6.8%" | matches | `src/assets/pools.rs:19` |
| 0017 | "the 282 within the View distance ... 59,390,720 of the 60,518,460 bytes measured ... 69% to 73% full"; "measured without Far proxies the shares were 40.8% ..."; "with LOD 0's Meshlets alone ... 48.7% ..." | not checkable from the tree (measurements and history) | — |
| 0017 | "the Mesh table an entry per 5,124 bytes"; "the Material table an entry per 19,267,584 bytes ... a second count of 285 entries" | matches | `src/assets/pools.rs:31-43`, `:93-95` |
| 0017 | "the static TLAS's two TLASes 57% of the acceleration structures', at 289 bytes an entry" | matches | `src/render/renderer.rs:209-223`, `:271-274` |
| 0017 | "the Instance slots 30% of the scene and pass buffers', at 136.2 bytes a slot with its offsets and visibility bit, in whole scan blocks" | drift (L7-A05) | `src/render/renderer.rs:213-218`, `:1092` |
| 0017 | "and the lights 0.1%, at 48 bytes" | matches | `src/render/renderer.rs:206-207`; `src/render/uniforms.rs:19-29` |
| 0017 | "The split leaves the vertex Arena at 55,197,040 bytes, under the floor's 128 MiB binding" | matches (83,886,080 × 0.658, down to 4 bytes) | `src/assets/pools.rs:82-84` |
| 0017 | "The Renderer and its Pools create the buffers, textures and TLASes they count through one ledger ... What ADR 0013 leaves out, the GPU stats readback and the load staging, bypasses it" | matches | `src/stats/gpu_memory.rs:145-205`; `src/render/staging.rs:179-184` |
| 0017 | "each TLAS with wgpu-core's buffer of its Instances, 72 bytes a slot on Metal and 64 elsewhere" | matches for the figures (the citation is the registry half, skipped; see Notes) | `src/stats/gpu_memory.rs:340`, `:364`; `src/render/acceleration_structures.rs:823` |
| 0017 | "45.5 of 94 bytes per triangle on the M4 Pro, 29.2 of 70.7 on the RTX 2070 SUPER ... and all of it on lavapipe (Mesa 25.2.8), picked by the adapter's vendor, then its backend" | matches (CI still pins a Mesa 25.2.8 build) | `src/render/acceleration_structures.rs:27`, `:36-51`; `.github/workflows/ci.yml:42` |
| 0017 | "Streaming uploads go through a `StagingBelt`, never `Queue::write_texture`" | matches (no shipping `Queue::write_texture`) | `src/render/staging.rs:179-213` |
| 0017 | "The cap holds each frame's streamed Chunk loads to a third of the staging Budget ... a third of what those slots at their largest leave of it" | matches (`FRAMES_IN_FLIGHT + 1` = 3) | `src/render/renderer.rs:266-275` |
| 0017 | "A capture's Settled wait loads past the cap ... and a spawn's loads are not capped" | matches (M5 checked it) | `src/streaming/io.rs` |
| 0017 | "A frame's Chunk loads also place at most 131,072 Instances" | matches | `src/streaming/io.rs:760` |
| 0017 | "A frame also loads at most 3 Chunks ..." | matches | `src/streaming/io.rs:765`, `:983` |
| 0017 | "A Mesh a streamed load brings ... the GPU builds and compacts its BLAS over the frames after ... the load's Instances are placed in the first frame that finds every Mesh they name ready" | matches (M5 checked it; unchanged) | `src/streaming/io.rs` |
| 0017 | "The count stays 3, as `world`'s own Chunk loads size it: its slowest frame reads about 4 ms longer at 8 or 16" | not checkable from the tree (measurement) | — |
| 0017 | "The cooked format (ADR 0011) stores each mip as a separately loadable byte range ... and the Mesh Blob each Primitive's UV texel density" | matches (m3.5's L7-B02 answered by the code: stored since) | `src/assets/cooked.rs:510`, `:670`, `:722` |
| 0018 | "each source file gets a 128-bit GUID ... An Asset ID is the first 128 bits of SHA-256 over (GUID, Asset kind, glTF name, Channel for a Texture)" | matches (unchanged since m3.5's check) | `src/assets/cooked.rs` |
| 0018 | status: "`praxis cook` and every dev launch cook Materials, Textures and Meshes ... the reader rebuilds their Meshlets, which the Mesh pool keeps in three Arenas the main pass draws from" | matches (m3.5's L7-B03 fixed) | `src/assets/pools.rs:74-99` |
| 0018 | status: "also cook each Reference scene's static content into Chunks ... since 2026-11-08 streams in around its camera through the Chunk streamer and the IO pool" | matches | `src/streaming/io.rs:306-316`; `src/streaming/mod.rs` |
| 0018 | status: "Plain `praxis cook` leaves out the `world` scene, whose library of more than 8 GB cooks only as `praxis cook world` or on a miss" | matches | `src/bin/praxis.rs:485-495` |
| 0018 | status: "A dev launch cooks only what it loads: each Asset as its load finds the Blob missing or stale" | matches | `src/state.rs:205` |
| 0018 | Considered options (paths, sidecars, content hashes, archive, Basis, neural, BLAKE3/XXH3) | not checkable from the tree (rejected designs and measurements) | — |
| 0018 | "The cooker warns when an ID that something references no longer exists" | matches (unchanged) | `src/cook/mod.rs` |
| 0018 | Cook key clause; "Each dev launch recomputes the keys of its scene's Materials and Meshes as it starts, of each Chunk and Texture as it loads it, and, as it loads a Far proxy, of the Textures that proxy bakes from" | matches (the 30 MB / 10 ms figure is not checkable) | `src/cook/mod.rs:695-707`; `src/state.rs:205` |
| 0018 | "The Cooked cache lives in git-ignored `cooked/` and is never evicted" | matches | `.gitignore` |
| 0018 | "Each Blob starts with a magic and a format version ... A mismatch is refused with a re-cook message" | matches (unchanged in kind) | `src/assets/cooked.rs` |
| 0018 | "Engine data ... compiled into the binary ...: the tonemapping pass's 64² blue-noise dither mask" | matches | `src/render/tonemapping_pass.rs:105` |
| 0018 | "Textures are KTX2 with every level zstd-supercompressed, stored smallest mip first ... the loader re-pitches them to 256 bytes as it writes into the `StagingBelt`" | matches | `src/render/staging.rs:213`, `:554` |
| 0018 | "Formats per Channel ... The encoder is `intel_tex_2` at its basic profile ... since `intel_tex_2` 0.5.0 ... A Far proxy's baked albedo alone encodes at its `ultra_fast` profile" | matches (0.5.0 pinned); the dB figures are not checkable | `Cargo.toml:24`; `Cargo.lock:1153-1154` |
| 0018 | "BC6H waits for HDR" | planned | — |
| 0018 | "Mips: a 2×2 box filter in linear light ... Toksvig ... refuses a Texture whose size is not a multiple of 4 or exceeds 8192" | matches | `src/cook/texture.rs:47`, `:171`, `:562-569` |
| 0018 | "a 28-byte vertex ... and u32 indices" | matches | `src/assets/cooked.rs:497` |
| 0018 | "Meshlets of at most 64 vertices ... and at most 124 triangles ... as meshoptimizer's scan builder cuts them" | matches | `src/assets/cooked.rs:45`, `:49`; `src/cook/mesh.rs:47` |
| 0018 | "SciFiHelmet's Mesh Blob was 322,852 bytes this way ..."; "of six LODs, was 356,874 bytes"; "with meshoptimizer 1.3's vertex codec 1 it is 353,120 bytes" | not checkable from the tree (fetched asset, no test pins it); the codec the cook key names matches | `src/cook/mesh.rs:43-47` |
| 0018 | "per-Meshlet bounds and cones wait for per-Meshlet tests" | planned (culling tests Instances) | `src/render/shaders/culling.wgsl:133-151` |
| 0018 | "Each Primitive stores its UV texel density ... the square root of its summed UV area over its summed mesh-space area" | matches (m3.5's L7-B04 answered by the code) | `src/assets/cooked.rs:510`, `:670` |
| 0018 | "The Blob holds the Mesh's LOD chain ... at most 8 LODs and ends before a LOD that would keep more than 85%" | matches (M5 checked it; the meshopt 0.7 bump leaves the rule) | `src/cook/mesh.rs:47` |
| 0018 | "A Thin Primitive's vertices are locked ... Each Primitive stores a Thin Primitive's radius, zero for any other" | matches | `src/assets/cooked.rs:514`, `:671` |
| 0018 | "A Material is its own Asset: PBR factors plus one Texture Asset ID per Channel" | matches (unchanged) | `src/assets/cooked.rs` |
| 0018 | "A Chunk's edge is 256 m ... 40 by 40 Chunks cover 10 km. A v1 Chunk is its own Asset ... holding its Placements, its point lights and its bounds" | matches | `src/assets/cooked.rs:58`; `src/scenes/world.rs:26` |
| 0018 | "`town`'s 16 Chunks are 20.6 MB raw and 0.69 MB at level 9 ..." | not checkable from the tree (measurement) | — |
| 0018 | "The cooker lives in `src/cook/` ... behind a default-on `cook` feature" | matches | `Cargo.toml:38`, `:41` |
| 0018 | "New dependencies ...: `intel_tex_2` (MIT or Apache-2.0 ...), `ktx2` (Apache-2.0), `meshopt` (MIT or Apache-2.0), `zstd` (BSD-3-Clause)" | matches (meshopt 0.7.0's licence field is `MIT OR Apache-2.0`) | `Cargo.toml:23-27`; `~/.cargo/registry/src/*/meshopt-0.7.0/Cargo.toml:71` |
| 0019 | "a 4x MSAA reverse-Z depth pre-pass, then a lit pass at depth equal ... reads each one's shadow from a shadow mask, which a compute pass traces" | matches | `src/render/main_pass.rs:315-368`; `src/render/shadow_mask_pass.rs:35` |
| 0019 | amendment: "the rays cost the M4 Pro's GPU frame 2.4 to 13.4 times ..." | not checkable from the tree (measurement) | — |
| 0019 | "with alpha-to-coverage for masked materials (amended ..., ADR 0027: alpha-to-coverage stays off ...)" | matches (M5 checked it) | `src/render/main_pass.rs` |
| 0019 | "its 4x depth, 58,982,400 bytes at 1440p ..."; amendment: "the depth passes also write each sample's triangle normal and the normal interpolated over it, 4 bytes per sample ... the target is allocated either way" | matches (unchanged since m3.5's check); the ms figure is not checkable | `src/render/targets.rs` |
| 0019 | status: "Accepted 2026-10-12. Planned; today a compute pass appends the Meshlets ..." | drift (L7-A07), killed by the verifier (see Killed) | `src/render/main_pass.rs:315-368`; `src/render/froxel_pass.rs:22-31`; `src/render/shadow_mask_pass.rs:35` |
| 0019 | "today a compute pass appends the Meshlets of the LOD selected for each Instance whose Mesh bounds reach into the view frustum ... in slot order, in two phases" | matches | `src/render/shaders/culling.wgsl:133-151`, `:594-600` |
| 0019 | "Phase one ... Hi-Z pyramid ... Phase two ... Phase two's Instances draw into depth: with shadow rays off, at the start of the lit pass; with them on, in a pass of their own" | matches | `src/render/main_pass.rs:315-362` |
| 0019 | "with one `draw_indexed_indirect` per phase for the depth pipeline and one of both phases for the lit pipeline, and one more per phase and pipeline for Thin Primitives and small Instances (ADR 0027)" | matches | `src/render/main_pass.rs:323`, `:329`, `:360`, `:362`, `:365-368` |
| 0019 | "lit by every directional light and by the point lights of each pixel's Froxel ... up to 59 lights per pixel; a light past them casts no shadow there and is counted" | matches | `src/render/shadow_mask_pass.rs:35`; `src/render/froxel_pass.rs:22-31` |
| 0019 | Considered options B to E and Froxels' rationale | not checkable from the tree (rejected designs, measurements, registry quotes) | — |
| 0019 | "Dense static meshes use discrete LOD" | matches (LOD chains, selected per Instance) | `src/render/renderer.rs:188-194` |
| 0019 | "One compute culling pass composes each Instance's bounds centre camera-relative ... drawn with one `draw_indexed_indirect` per pipeline and Hi-Z phase ... No count draws" | matches (no `multi_draw_indirect` in `src/`) | `src/render/shaders/culling.wgsl:133-151`; `src/render/main_pass.rs:315-368` |
| 0019 | "One draw spans every Instance because each index packs a Meshlet record in its high bits and a vertex within that Meshlet in its low six" | matches (the thin twins split the draw by pipeline, not by Instance) | `src/assets/cooked.rs:45`; `src/render/shaders/common.wgsl:305-309` |
| 0019 | "Culling tests Instances; tests per Meshlet wait for a profile ... it stays a prototype" | matches | `src/render/shaders/culling.wgsl:133-151` |
| 0019 | "A Mesh is a Meshlet range per LOD ... for at most 8 LODs"; "the coarsest LOD whose LOD error times the Instance's scale is under the LOD threshold, one pixel of the target's height" | matches (M5 checked it; unchanged) | `src/render/renderer.rs:188-194`; `src/assets/cooked.rs` (`MESH_MAX_LODS`) |
| 0019 | "On the M4 Pro ... fragments of nearby triangles ... can be merged ... Two passes that must agree on a sample's triangle ... both read a flat input" | hardware: not checkable from the tree; the flat inputs match | `src/render/shaders/main.wgsl:12`, `:30`, `:98-103` |
| 0019 | "The depth pre-pass writes 4x MSAA depth, and with shadow rays on each sample's surface normals ... the shadow mask pass rebuilds each pixel's position from it and reads each sample's normals from the surface normals" | matches (m3.5's L7-B05 and L7-B06 fixed) | `src/render/main_pass.rs`; `src/render/shaders/shadow_mask.wgsl` |
| 0019 | "the mask's second layer ... 16 bytes per pixel, 58,982,404 bytes at 1440p ... 13.7% of its pixels" | matches for the bytes (unchanged since m3.5); the ms, RMSE and percentage are not checkable | `src/render/targets.rs` |
| 0019 | "they are listed from the light buffer, which holds its lights directional first ... 0.1% of it at 48 bytes a light: 4,893 lights at the Budgets M4 set" | matches (m3.5's L7-B07 fixed) | `src/render/renderer.rs:206-207`, `:262-265`, `:1093` |
| 0019 | "past it the point lights farthest from the camera, measured between f64 World positions (ADR 0014), are cut and the rest keep their order" | matches | `src/render/prepare.rs:983`, `:1051-1055` |
| 0019 | "at town's density over a 2 km view, Resident lights number 12,868" | not checkable from the tree (measurement) | — |
| 0019 | "Transparency is a sorted forward pass after the opaque pass ..." | planned (no transparent pass) | `src/render/main_pass.rs` |
| 0019 | "Render pipelines are vertex pipelines ... a mesh-shaded pre-pass returns with Meshlet culling" | matches; the mesh-shaded pre-pass is planned | `src/render/main_pass.rs` |
| 0019 | Reopen clause: "more than 6 ms of GPU ... timed from the first culling dispatch to the end of the lit pass with shadow rays off ... Shadow-ray time is reported beside the gate" | matches | `bench/budgets.json:4`; `src/bench/mod.rs:711-713`, `:1319-1334` |
| 0019 | "The 6 ms is provisional, as ADR 0013's budgets are" | drift (L7-A06) | `docs/adr/0013-reference-hardware-and-provisional-budgets.md:3`; `bench/budgets.json:4`; `src/bench/mod.rs:1331-1334` |
| 0019 | "The clause fired on `town` and is spent there: B ... 1.9 to 3.5 ms ...; D ... 2.3% under A's 8.6 ms ..."; "`prototype/visibility-b` and `prototype/visibility-d` ... where no figure for either exists" | not checkable from the tree (measurements, branches; M5 found both branches on `origin`) | — |
| 0020 | "Indirect diffuse light comes from ray-traced irradiance probes in camera-centred clipmap cascades ... updated a subset per frame" | planned (a constant ambient stands in, as the status says) | `src/render/shaders/shading.wgsl:261-267` |
| 0020 | "Direct light is one hard shadow ray per pixel per light, and, where a directional light's verdict differs from a neighbour's ... one from each other sample" | matches | `src/render/shadow_mask_pass.rs:35`, `:55` |
| 0020 | amendments: "on `town` the extra rays cost ... 2.7 to 3.1 ms ..."; "11 pixels of `crowd`'s frame 59 ... 13.8% of `town`'s pixels ... 0.57 to 0.60 ms" | not checkable from the tree (measurements) | — |
| 0020 | "Direct light is Temporal class (a); indirect light is class (b), and class (c) is used nowhere" | matches for direct light; indirect is planned | `src/render/targets.rs` (no history target) |
| 0020 | status: "each Mesh rays trace gets a BLAS at load, compacted unless it is loaded to be refitted or rebuilt each frame" | matches (m3.5's L7-B08 fixed) | `src/assets/pools.rs:651-683` |
| 0020 | status: "the static TLAS covered ... until it was re-anchored and rebuilt off the frame's critical path on 2026-11-08 ...; it holds exactly the static Instances of the loaded, drawn Chunks, the drawn Far proxies ... and the static spawned Renderables, each in an entry of its own" | matches | `src/render/acceleration_structures.rs:200-210`, `:1742-1743` |
| 0020 | status: "One hard shadow ray per pixel is traced toward each directional light against the static TLAS, skipped where ... the surface's interpolated normal turns from [it]" | matches | `src/render/shaders/shadow_mask.wgsl:70-79`, `:139-155` |
| 0020 | status: "at a pixel whose verdict for one of the first seven directional lights differs from a 4-neighbour's ..." | matches | `src/render/shadow_mask_pass.rs:55` |
| 0020 | status: "into a shadow mask the lit pass reads, which holds 59 lights per pixel ... Frame stats count it" | matches | `src/render/shadow_mask_pass.rs:35` |
| 0020 | status: "Moving Instances form the dynamic TLAS, rebuilt each frame ... from at most 10,000 of them, those in the lowest slots ... timed in each frame that builds it" | matches | `src/render/acceleration_structures.rs:17`; `src/render/prepare.rs:852` |
| 0020 | status: crowd's shadow proxies, the refit switch through wgpu-hal on Metal, the rebuild form "as `DynamicTlas::mark` records" | matches | `src/render/blas_refit.rs:166-279`; `src/render/acceleration_structures.rs:940-960` |
| 0020 | status: "A constant ambient term stands in for indirect light" | matches | `src/render/shaders/shading.wgsl:261-267` |
| 0020 | Considered options A, B, D, E, surfels, soft sun shadows | not checkable from the tree (rejected designs, figures, registry quote) | — |
| 0020 | "Ray queries are a required capability ...; cascaded shadow maps are dropped" | matches | `src/gpu/mod.rs:277` |
| 0020 | "Indirect light converges to 90% ... the probe update takes at most 2 ms of GPU ..." | planned (no probes) | — |
| 0020 | "Both are provisional, as ADR 0013's budgets are" | drift (L7-A08) | `docs/adr/0013-reference-hardware-and-provisional-budgets.md:3`; `bench/budgets.json:2-8` |
| 0020 | "No bake is needed ... The one cooked lighting term is a per-Chunk sky-visibility field"; Interiors; Rough and glossy specular; contact darkening | planned | — |
| 0020 | "The TLAS holds the near field plus merged far proxies ... A static build keeps the static TLAS's Anchor until the camera's is more than a Chunk's edge from it ... two static TLASes ... a worker thread ... which the main thread submits (ADR 0016)" | matches (M5 checked it; unchanged) | `src/render/acceleration_structures.rs:190-210`, `:530-546`, `:572-614` |
| 0020 | "on wgpu 30.0.1 a worker takes 13 to 14 ms of a 254,256-Instance rebuild ... 75.6 MB at 262,144 entries"; "wgpu-core's `Queue::submit` fell from 2.5 to 0.05 ms" | not checkable from the tree (measurements, dated by version) | — |
| 0020 | "walks one dependency per distinct BLAS since `vendor/wgpu-core` carries gfx-rs/wgpu#9835" | matches | `vendor/README.md:22-31`; `Cargo.toml:54` |
| 0020 | "a Far proxy's entry change alone ... starts one only once it is 120 frames old"; "a dynamic TLAS of Moving Instances rebuilt each frame and capped at 10k Instances" | matches | `src/render/acceleration_structures.rs:17`, `:199` |
| 0020 | "Shadow and probe rays query both" | shadow rays: matches; probe rays: planned | `src/render/shaders/shadow_mask.wgsl` |
| 0020 | "Every BLAS is built from its Mesh's LOD 0 ... the index Arena ... holds LOD 0's indices alone" | matches (M5 checked it; unchanged) | `src/assets/pools.rs:15-19`, `:699` |
| 0020 | "Sun penumbrae reopen when area lights arrive or a Golden shows hard sun edges at distance" | matches (hard rays only); the trigger is not checkable | — |
| 0021 | "Device creation requires a fixed set of wgpu features, downlevel flags and limits ... fails with every gap listed ... The floor holds only what an ADR or shipped code uses" | matches | `src/gpu/mod.rs:266-345` |
| 0021 | "a Tier 3 to 6 feature is always optional, probed at runtime, with a path for adapters without it" | matches (vacuously: none is requested) | `src/gpu/mod.rs:268-279` |
| 0021 | "Enforced since #111: `GpuContext` checks the adapter against the floor, then requests exactly its features, limits and experimental opt-in" | matches | `src/gpu/mod.rs:57-78` |
| 0021 | Considered options: Tier 1 in full, Tiers 1 and 2, the adapter's own limits | not checkable from the tree (rejected options; the M4 Pro limits are recorded in a test) | `src/gpu/mod.rs:623` |
| 0021 | "CI's software Vulkan jobs request the floor's device on lavapipe, which shows lavapipe meets it" | matches (m3.5's L7-C03 fixed) | `.github/workflows/ci.yml:42`, `:177` |
| 0021 | "Required features: `TEXTURE_BINDING_ARRAY`, `PARTIALLY_BOUND_BINDING_ARRAY`, `SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING`, `TEXTURE_COMPRESSION_BC`, `EXPERIMENTAL_RAY_QUERY`, `TIMESTAMP_QUERY`" | matches | `src/gpu/mod.rs:268-279` |
| 0021 | "4x MSAA on every attachment format the main pass uses ... with its resolve on the HDR colour format ... Those formats are `Depth32Float`, the HDR target's `Rgba16Float` and the surface normals' `Rgba8Unorm`" | matches (m3.5's L7-C01 fixed) | `src/render/targets.rs:51-52`; `src/gpu/mod.rs:57`, `:363-373` |
| 0021 | "the window's surface and the capture target take only the tonemapping pass's single-sample output" | matches (unchanged) | `src/gpu/mod.rs:458-475` |
| 0021 | "The `ANISOTROPIC_FILTERING` downlevel flag is required too ... So is `VERTEX_STORAGE` ... So are `COMPUTE_SHADERS` and `INDIRECT_EXECUTION`" | matches | `src/gpu/mod.rs:283-296` |
| 0021 | "the main pass draws it with indirect draws, one per pipeline and Hi-Z phase, each from first instance 0" | matches as m3.5's L7-C02 fix worded it (see the ADR 0016 row) | `src/render/main_pass.rs:315-368` |
| 0021 | "which left the floor, since each draw spans every Instance through packed indices"; "`TIMESTAMP_QUERY_INSIDE_ENCODERS` and `_INSIDE_PASSES` stay optional" | matches (neither in the floor) | `src/gpu/mod.rs:268-279` |
| 0021 | "Vulkan, Metal and DX12 report it on every adapter in wgpu-hal 30.0.1"; "the M4 Pro reports neither on wgpu 30.0.1" | not checkable from the tree (registry quotes and hardware, named by version) | — |
| 0021 | "Ray queries need `ExperimentalFeatures::enabled()`, an `unsafe` call; it is the one `unsafe` this ADR adds, and its comment cites ADR 0020" | matches | `src/gpu/mod.rs:75-78` |
| 0021 | "Required limits are `wgpu::Limits::default()` with `using_minimum_supported_acceleration_structure_values()` ... raised only where an ADR needs it: `max_buffer_size` at least 2^31 − 1, `max_binding_array_elements_per_shader_stage` at least 65,536, `max_acceleration_structures_per_shader_stage` at least 2 ... `max_storage_buffers_per_shader_stage` at least 9" | matches (`max_texture_array_layers` is named at its default, not raised) | `src/gpu/mod.rs:310-322` |
| 0021 | "since the culling pass's compute stage reads and writes nine" | matches (M5 counted nine; the stage's groups are unchanged since) | `src/render/bind_groups.rs` |
| 0021 | "`max_storage_buffer_binding_size` stays at the default 128 MiB ... so that Arena stays within it, as a test checks" | matches | `src/gpu/mod.rs:310-322`; `src/assets/pools.rs:82-84` |
| 0021 | "A ticket that needs more raises the floor in its PR" | not checkable from the tree (process) | — |
| 0021 | "`GpuContext` checks the whole floor before requesting the device and returns one typed error ...; the binary logs it with `tracing::error!` and exits non-zero. `praxis capabilities` runs without the floor" | matches | `src/gpu/mod.rs:327-345`; `src/bin/praxis.rs:121`, `:296` |
| 0021 | "Headless contexts, tests and Goldens included, use the same floor" | matches (unchanged) | `src/gpu/mod.rs` |
| 0021 | "Hardware out of scope ... (52% of gpuinfo's ...)" | not checkable from the tree (hardware; registry) | — |
| 0021 | "The full floor, ray queries included, is enforced in M1, before ray queries are first used in M3" | matches (history; the floor requests ray queries) | `src/gpu/mod.rs:277` |
| 0022 | "`src/time/` runs the fixed schedule once per Tick, at most 4 per frame, with ambiguity detection at `Error`, the Tick counter and blend weight α in `FixedTime`, and the dropped time in frame stats; the helmet spawn and despawn run in it" | matches | `src/time/mod.rs:4-45`, `:70`; `src/state.rs:389-417` |
| 0022 | "World transforms propagate at the end of every Tick ... `PreviousWorldTransform`, and prepare blends the two in f64; a spawned entity, or one marked `Teleported`, is drawn at its current World transform until the next Tick" | matches (the blend now slerps a rotation, still in f64) | `src/transform/mod.rs:20-45`; `src/state.rs:403-415` |
| 0022 | "The `orrery` Reference scene shows it" | matches (unchanged) | `src/scenes/mod.rs` |
| 0022 | Considered options A, B, D, E, 60 and 120 Hz | not checkable from the tree (measurements, rejected options) | — |
| 0022 | "The frame delta is measured with `Instant` at frame start, before surface acquire ... clamped to 250 ms. The fixed loop runs at most 4 Ticks per frame; accumulated time past the cap is dropped ... reported in frame stats" | matches | `src/lib.rs:222`; `src/time/mod.rs:4`, `:13`, `:37-45` |
| 0022 | "Simulated time is an integer Tick counter" | matches (unchanged) | `src/time/mod.rs` |
| 0022 | "Everything that changes game state runs in the fixed schedule ... Input sampling, the camera, the blend, prepare and presentation-only effects run once per frame" | matches for what exists (spin, sway, helmet, wrap-around); physics, AI and Lua planned | `src/state.rs:394-417` |
| 0022 | "its position is moved by input every frame too, since the camera is free-flying and attached to nothing" | matches (m3.5's L7-C04 fixed) | `src/camera/mod.rs:107`; `src/state.rs:394` |
| 0022 | "Static Instances store nothing extra. A spawned entity starts with previous equal to current ... a teleport goes through an explicit marker" | matches | `src/transform/mod.rs:26-29`; `src/state.rs:414` |
| 0022 | "Blending 100k Instances costs 0.12 to 0.29 ms more" | not checkable from the tree (measurement) | — |
| 0022 | Determinism: "ambiguity detection is `Error`, no system reads the wall clock, randomness comes from a seeded resource advanced per Tick, and no result depends on hash-map iteration order or on reduction order in a parallel iterator" | matches for the fixed schedule's systems (none reads `Instant`, draws randomness or iterates in parallel) | `src/state.rs:403-415`; `src/threading.rs:24-28` |
| 0022 | "ADR 0013's budget splits, judged at p95 once Input latency is measured" | planned | `bench/budgets.json` |
| 0022 | "The blend reads two simulation states, never screen-space history"; "Budgets stay judged at 60 Hz" | matches | `src/transform/mod.rs:26`; `src/time/mod.rs:16` |
| 0022 | "A physics engine chosen in M8 steps once per Tick" | planned | — |
| 0023 | "a `DirectionalLight` gives illuminance in lux and a `PointLight` luminous intensity in candela ...; emissive and background give luminance in cd/m², and a Reference scene's ambient term gives illuminance in lux. Exposure is an EV100" | matches (unchanged since m3.5's check) | `src/components.rs:80-101`; `src/scenes/mod.rs:629-630` |
| 0023 | "glTF lights then import without a scale" | not checkable from the tree (the cooker imports no glTF light) | `Cargo.toml:16` |
| 0023 | "every Reference scene's lights, ambient and background are in these units, under a sun of 100,000 lx, and each scene has an EV100 ... clamped to 65504; the tonemapping pass then maps it through GT7" | matches (`world` included) | `src/scenes/mod.rs:509-529`, `:586-639`; `src/state.rs:120` |
| 0023 | Considered options U2 and lumens | not checkable from the tree (registry) | — |
| 0023 | "Shading runs in f32 and pre-exposes colour before fp16 stores it, and every path clamps it to 65504" | matches (m3.5's L7-C05 was killed on the clear colour's clamp) | `src/render/targets.rs` (`color_attachment`) |
| 0023 | "Each Reference scene sets its EV100 by hand in M3" | matches | `src/scenes/mod.rs:513-529` |
| 0023 | "Histogram adaptation and local tone mapping wait for ..." | planned | — |
| 0023 | "Brighter emissives need KHR_materials_emissive_strength, which the cooker does not read yet" | matches (no `emissive_strength` in `src/`) | `src/cook/mod.rs` |
| 0023 | "Point lights fall off by inverse square inside KHR_lights_punctual's range window. Spot lights, out of M3, take candela too" | falloff: matches (unchanged); spot lights: planned | `src/render/shaders/shading.wgsl` |
| 0023 | "Blender's glTF exporter ... writes ..." | not checkable from the tree (registry) | — |
| 0024 | "keyed by source hash × adapter × backend × scene; the Harness version is defined under `src/`, so the source hash names it, and the record holds it" | matches (m3.5's L7-C06 fixed) | `src/bench/history.rs` |
| 0024 | "Only clean, release-profile records of `main`'s source taken by a quiet machine form a Series. The history observes and never gates" | matches (unchanged) | `src/bench/history.rs` |
| 0024 | "a nightly sweep on the reference machine ... The RTX 2070 SUPER has no sweep" | matches (only `sweep.plist` and `sweep.sh`; the Windows task is gone) | `scripts/sweep.plist`, `scripts/sweep.sh` |
| 0024 | "its Goldens and Baselines are recorded by hand at each milestone's exit" | not checkable from the tree (process); the RTX Baselines exist | `bench/baselines/nvidia-geforce-rtx-2070-super-dx12/` |
| 0024 | "A page in `docs/` draws the Series" | matches | `docs/bench/` |
| 0024 | Considered options | not checkable from the tree (external) | — |
| 0024 | "The source hash covers `src/`, `assets/`, `Cargo.toml`, `Cargo.lock`, `.cargo/` and `vendor/` ..., a list held in one place in code, and a test fails when a new top-level path is classified neither way" | matches | `src/bench/provenance.rs:27-33`, `:410` |
| 0024 | "A record path is its key ... the quieter one stays: the one with the lower GPU p95/median; a record on a Series holds its key against a tagged one" | matches (unchanged in kind) | `src/bench/history.rs` |
| 0024 | quiet rule: "preflight warned of none of the GPU load, the CPU load, other praxis processes, the power and the display ... p95/median below about 1.06 ... over the median of the 31 frames around it ... no process outside praxis at or over half a core at the scene's start or end" | matches | `src/bench/provenance.rs:14`, `:36-43`, `:63-68`, `:193` |
| 0024 | "lavapipe's records are the exception to the quiet test ...; Every other record is kept, tagged ...; An Experiment's records are kept under its name" | matches (unchanged) | `src/bench/history.rs` |
| 0024 | "The Harness version rises whenever a change alters what a metric means, and each record carries its scene's content hash" | matches (unchanged) | `src/bench/provenance.rs` |
| 0024 | "64 on `main`'s first-parent line before `HISTORY_STARTED`" | matches (m3.5's L7-C07 fixed) | `src/bench/history/import.rs` |
| 0024 | "Publishing never blocks work ...; `verify` never publishes. Baselines are still chosen by hand, and a re-recorded Baseline's record is marked as one" | matches (m3.5's L7-C08 answered by the code: `baseline-rerecorded`) | `src/bench/history.rs:130-134`, `:265-283` |
| 0024 | "A timing record far from its Series' recent median is a suspect. The sweep re-benches it, bisects ... files ... one `needs-triage` issue" | matches | `scripts/sweep.sh:15`; `src/bench/history/verdicts.rs:435` |
| 0024 | "A bad publish is undone with `git revert` on `bench-history`. The branch holds only data and a README ... CI writes it only from pushes to `main`" | matches (unchanged since m3.5's check) | `.github/workflows/ci.yml` |
| 0025 | "everything in it is published under the MIT licence (`LICENSE`, `Cargo.toml`): `main`, every PR's branch, `bench-history` and `pr-images` alike" | matches for `LICENSE` and `Cargo.toml`; the branches are not checkable from the tree | `LICENSE`; `Cargo.toml:5` |
| 0025 | "The owner kept MIT on 2026-10-20 ... kept the repo public on 2026-11-14"; "revisited before the first outside contribution" | not checkable from the tree (decisions, process) | — |
| 0025 | Considered options (private, another licence) | not checkable from the tree | — |
| 0025 | "Every file under `assets/` has a manifest entry and a credit, which `cargo test` checks" | matches | `tests/asset_manifest.rs` |
| 0025 | "a vendored crate keeps its licence files (`vendor/README.md`), and every dependency's licence expression is on the allowlist in `tests/dependency_licences.rs`" | matches | `vendor/wgpu-core/LICENSE.MIT`, `LICENSE.APACHE`; `tests/dependency_licences.rs` |
| 0025 | "CI runs on GitHub's hosted runners, without a GPU, and local scheduled tasks bench the real ones" | drift (L7-A09) | `scripts/sweep.plist:3`; `scripts/` (no `sweep-task.xml`); `docs/adr/0024-bench-history-is-git-stored-bench-records-keyed-by-engine-source.md:3` |
| 0026 | status: "Implemented: the cook of a Far proxy's Mesh ..., its atlas ..., the transfer ... the Blobs of its baked Material ..., its draw, the swap and the ground past the View distance ... and its shadows" | matches (M5 checked each; the code since changed the mip uses, the export and the meshopt version, none of which reaches these) | `src/cook/far_proxy.rs`; `src/render/acceleration_structures.rs:199` |
| 0026 | Decision counts: "14.7 cm at 256 m"; "more than 5,832 triangles ... 324 buildings of 18"; "29.5 cm ... 2 texels of gutter a side on a grid of 4 ... 4 × 4 samples a texel" | matches | `src/cook/far_proxy.rs:21`; `src/cook/far_proxy/bake.rs:15-18` |
| 0026 | Decision: "requested when its Chunk comes within the View distance, and ahead of that while the camera closes on it" | matches (M5's L7-A02 fixed) | `src/streaming/mod.rs:229-249` |
| 0026 | Decision, "Its mesh": "the error is the simplifier's own measure, an approximation of the deviation [ref:meshoptimizer-1.3-simplify-error]" | matches (the meshopt 0.7 bump moved the key and, by its probe, left every Far proxy's geometry as it was) | `Cargo.toml:26`; `src/cook/mesh.rs:47` |
| 0026 | Settled by the cook: "LOD 0 at 5,832 triangles and LOD 1 at 3,240, its roofs flat and 0.3 m lower, which the simplifier reports as an error of 0.27 m"; "a reported error of 2.79 m, where a pixel at the View distance spans 1.15 m" | matches for 5,832, 3,240 and the 0.3 m (pinned by test); 0.27 m and 2.79 m are not checkable from the tree (measurements, unchanged by the bump's same geometry) | `src/cook/far_proxy.rs:735` |
| 0026 | Settled by the cook: "its corners lie 181 m from its centre, and `world`'s farthest proxy vertex 190 m ... f32 resolves 0.015 mm" | matches for the arithmetic (M5); 190 m is not checkable from the tree | — |
| 0026 | Settled by the bake: "past 8,192 texels a side fails its cook. `world`'s densest Chunk charts into 2048²"; "within 45°"; "two pixels of its plane" | matches (M5 checked them; the bake is unchanged since) | `src/cook/far_proxy/bake.rs:15-21` |
| 0026 | Settled by the bake: "7.5 core-hours under `basic` against 0.66 under `ultra_fast` ... 21,790 of 3.7 million pixels" | not checkable from the tree (measurements) | — |
| 0026 | Settled by the draw: "starts one itself once it is 120 frames old, so such a Far proxy can draw unshadowed for two seconds at 60 Hz" | matches | `src/render/acceleration_structures.rs:199` |
| 0026 | Settled by the draw, "Its mips": "A Far proxy held past the View distance has its mips requested too, never late ...; one held while its Chunk is drawn needs none" | matches (a held-for-its-Chunk use needs nothing; a held-ahead one is judged ahead) | `src/streaming/mips.rs:316-323` |
| 0026 | Settled by the draw: "scaled to reach 1,024 km ... laid 0.5 m under ... 62.5 mm ... a tenth of a pixel under the horizon" | matches (M5 checked them; unchanged) | `src/scenes/world.rs:46`, `:51`, `:58` |
| 0026 | Considered options: "at most 219 proxies"; "57 and 41 pixels (clustering by 12 on the Chunk's shell alone ...)"; "3.02 M texels" | not checkable from the tree (measurements; M5's L7-F01 fixed the 12) | — |
| 0026 | Consequences: "ADR 0013's 2 km View distance is drawn this way, and the 'merged far proxies' of ADR 0020 are these" | matches | `docs/adr/0013-reference-hardware-and-provisional-budgets.md:3`; `docs/adr/0020-ray-traced-probe-gi-and-hard-ray-traced-shadows.md:20` |
| 0026 | Consequences: "ADR 0014's 128 m pivot bound exempts a Far proxy's Mesh" | matches | `src/cook/mesh.rs:163-173` |
| 0026 | Consequences: "The Material table counts Far proxies' Materials apart, and a frame's loads are capped by count (ADR 0017)" | matches | `src/assets/pools.rs:43`; `src/streaming/io.rs:765` |
| 0026 | Consequences: "A streamed scene whose Chunk keeps more triangles than the cap at one pixel fails its cook, naming the count" | matches | `src/cook/far_proxy.rs:21` |
| 0026 | Consequences: "A LOD error is what the simplifier reports, which on `world`'s flattened roofs is a tenth under the distance they moved" | matches (M5 checked it) | `src/cook/mesh.rs` |
| 0026 | Consequences: "A dev launch that loads a Far proxy hashes the albedo and metallic-roughness Textures its sources draw with ... and bakes the proxy where a Blob of it is missing or stale" | matches (M5 checked it; unchanged) | `src/cook/mod.rs` |
| 0027 | status: "Implemented for Thin Primitives and small Instances. Not yet: M7's masked Materials" | matches (no alpha mode in the cook or the Material) | `src/render/shaders/main.wgsl:187-207`; `src/cook/mod.rs` |
| 0027 | Decision: "alpha-to-coverage stays off in every pipeline"; "Five levels, rounded ... sample 0, then 3, 1 and 2"; "Each pipeline of the main pass has a twin" | matches (code unchanged since M5's check) | `src/render/main_pass.rs:113`; `src/render/shaders/common.wgsl:224-233` |
| 0027 | Considered: "cost `town`'s GPU frame 0.73 ms with shadow rays on and its gate 0.56 ms" | not checkable from the tree (measurement) | — |
| 0027 | Consequences: "ADR 0019's 'alpha-to-coverage for masked materials' no longer holds" | matches (ADR 0019 carries the amendment) | `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:3` |
| 0027 | Consequences: "Every tube takes three faces' share, 0.75 of its diameter ... from 65 m to 75 m and from 521 m to 602 m ... 361 m" | matches for 0.75 (the distances follow from it as M5 found) | `src/render/shaders/common.wgsl:224-227` |
| 0027 | Consequences: "The culling pass's compute stage already binds the nine storage buffers the Capability floor allows" | matches | `src/gpu/mod.rs:320` |
| 0027 | Consequences: "A fragment stage that writes the mask reads a flat input" | matches | `src/render/shaders/main.wgsl:98-103` |
| 0027 | Consequences: "81% of the dots a frame shows are there in the next, against 64% unscaled" | not checkable from the tree (measurement) | — |
| 0027 | Consequences: "The culling pass tests a small Instance's bounds as it draws them, scaled ..."; "A pixel whose samples a mask splits ... is listed for the shadow mask pass's second surface" | matches (unchanged since M5's check) | `src/render/shaders/culling.wgsl`; `src/render/shaders/shadow_mask.wgsl` |

### Notes

- **Lead 1 holds, outside this reader's documents.** `docs/research/job-systems-and-frame-threading.md:16`, the *ECS-scheduled parallel systems* row: "Praxis runs `SingleThreaded`: bevy_ecs's `multi_threaded` feature is off". `Cargo.toml:17`: `bevy_ecs = { version = "0.20.1", features = ["trace", "debug", "multi_threaded"] }`. bevy_ecs 0.20.1 has no `ExecutorKind` (`grep -rn "enum ExecutorKind"` over its `src/` finds nothing); it has `SingleThreadedExecutor` (`src/schedule/executor/single_threaded.rs:32`), which `src/threading.rs:24-28` sets. The same note's `:51` ("so `Schedule::default()` in src/state.rs runs single-threaded") and `:97` read false the same way. A research note is not in this lane's documents, so no Finding.
- **Lead 2 holds** and is L7-A03.
- **Lead 3 holds, in test code.** `src/render/prepare.rs:1182`, `src/render/acceleration_structures.rs:1776`, `src/streaming/mips.rs:849`, `src/streaming/chunks.rs:316` and `src/transform/mod.rs:179`, each in a `#[cfg(test)]` module, build their propagation schedule with `Schedule::default()`. In bevy_ecs 0.20.1 that is `Schedule::new(DefaultSchedule)` with `executor: default_executor()` (`src/schedule/schedule.rs:410-421`), which is `MultiThreadedExecutor` when `multi_threaded` is on (`src/schedule/executor/mod.rs:48-56`), and its run calls `ComputeTaskPool::get_or_init(TaskPool::default)` (`src/schedule/executor/multi_threaded.rs:274`): `available_parallelism()` threads in the test process, where the engine itself never creates that pool. It bears on L7-A03: in a test process that has run one of those schedules, a `par_iter` would find the pool and run, so a test would not show the panic the engine would meet.
- ADR 0016's clause "the main pass's draw count ... which ADR 0019 answers with one `draw_indexed_indirect` per pipeline and Hi-Z phase", and ADR 0021's "one per pipeline and Hi-Z phase", carry the wording m3.5's L7-A08 and L7-C02 fixes chose; the lit pipeline's one draw spans both phases (`src/render/main_pass.rs:365`), which "per pipeline and Hi-Z phase" bounds rather than counts. Not re-filed. A code comment keeps the older slip: `src/gpu/mod.rs:293-294`, "A compute pass builds the frame's draw, which the main pass draws with one indirect call (ADR 0019)", against up to eight indirect draws (`src/render/main_pass.rs:323-368`); a comment is outside this reader's documents.
- ADR 0017:18's "72 bytes a slot on Metal and 64 elsewhere" matches the code (`src/stats/gpu_memory.rs:364`, `src/render/acceleration_structures.rs:823`); M5 killed on scope a Finding that its three cited keys do not state 72. That is a registry citation, the half this audit skips, so it stays for the next source check.
- ADR 0026:7, a decision clause and so lane 1's: "Under it the Instance slots, the static TLAS and the flat cull keep M4's capacities". The Instance slots fell from M4's 517,376 to 488,448 two days after the ADR was accepted (`src/render/renderer.rs:1092`); the static TLAS keeps 264,720 (`:1094`). Left to lane 1.
- ADR 0019's status word (L7-A07) and the "provisional" clauses (L7-A06, L7-A08) stood at `8bc34eb1`; they are filed as misses of M3.5's lane 7, which the brief allows. L7-A01, L7-A03 and L7-A05 are three of the four leads M5's "Left for a full lane 7" names; the fourth, `docs/performance.md:2154`, is outside the ADRs.
- Doubt: L7-A04 may be read as covered by ADR 0020:20, which states the static TLAS worker; it is filed because ADR 0016's status line claims to list what of ADR 0016 is built and says the rest is planned.
- Doubt: L7-A09 rests on a plural ("tasks", "the real ones"); the clause's rationale holds.

## The glossary

I read every bold term of `CONTEXT.md` at the frozen commit `e5c55a8da2a59db2f09c99c8ea4522987d5354ac` in the worktree `/Users/guillem.arias/Documents/gariasf/praxis-1004` (79 terms, `CONTEXT.md:9` to `CONTEXT.md:331`). For each one I opened the code that holds what the definition claims: the component, resource, constant, struct, pass or shader function, and the doc comment that pins its number. The files opened were `src/components.rs`, `src/assets/{mesh,cooked,material,pools}.rs`, `src/cook/{mesh,far_proxy}.rs`, `src/render/{prepare,instance,renderer,readback,froxel_pass,shadow_mask_pass,targets,main_pass,bend_pass}.rs`, `src/render/shaders/{common,culling,shadow_mask}.wgsl`, `src/streaming/{mod,chunks,io,mips}.rs`, `src/scenes/mod.rs`, `src/stats/mod.rs`, `src/time/mod.rs`, `src/transform/mod.rs`, `src/bench/{mod,provenance}.rs`, `src/bench/history.rs`, `src/bench/history/{suspects,verdicts,night,rerecord,import}.rs`, `src/preflight/mod.rs`, `src/gpu/mod.rs`, `bench/budgets.json`, `bench/cycles/*.json`, `bench/baselines/`, `scripts/gallery.sh`, `scripts/cycles-reference/compare.py`, `scripts/sweep.sh`, `docs/agents/audit.md`, `docs/agents/workflow.md:12` and `docs/audit/deviations.md`. Numbers in definitions were checked against the constants that hold them: 59 and 7 lights (`SHADOW_MASK_LIGHTS`, `SHADOW_EDGE_LIGHTS`), 64 m (`ANCHOR_CELL_SIZE`), 1/64 s (`TICK`), 2 km (`view_distance_metres`), 8 records, 4 median absolute deviations, 3% and 1.25% (`WINDOW_RECORDS`, `MEDIAN_ABSOLUTE_DEVIATIONS`, `CPU_FRAME_SMALLEST_SHIFT`, `GPU_FRAME_SMALLEST_SHIFT`), half a core (`BUSY_PROCESS_PERCENT`), and the Meshlet bounds (`MESHLET_MAX_VERTICES`, `MESHLET_MAX_TRIANGLES`). Terms that name future scope were grepped over `src/`, `bench/` and `tests/`. Public type names were collected with `grep -rhoE "pub(\(crate\)|\(super\))? (struct|enum|type|trait) [A-Z]\w*" src` and matched against every term; the identifiers of the `_Avoid_` words added since M3.5 (HLOD, impostor, pop-in, warp, stutter, spike, bucket, size class, mip tail, LOD bias, screen-space error, LOD distance, streaming radius, draw distance, wire, strand, sector) were grepped over `src/`. Each term's history was read with `git log -L` where a clause looked stale, to place it before or after `8bc34eb1`. The M3.5 glossary Findings L7-G01 to L7-G09 and M5's L7-A01 (LOD error) were each re-read at the frozen commit. All of them now read true. Four Findings result, all `drift`, none load-bearing: an override clause that predates Placements, a Frame stats list that M4's streaming counters never joined, a Baseline clause that the counters-only re-record command now contradicts, and a Suspect window that omits its minimum. Nothing in either checkout was changed.

### Term verdicts

| Term | Verdict | Where |
|---|---|---|
| Renderable | matches | `src/render/prepare.rs:67-74` (`RenderableData`: `WorldTransform`, `MeshRef`, `Option<MaterialRef>`); `src/components.rs:9-11`, `:43-47` |
| Mesh | matches | `src/assets/cooked.rs:591-603` (`CookedMesh`: `primitives`, `coarser_lods`); `src/assets/mesh.rs:30-49` (`MeshData`: every LOD's Meshlets); BLAS from LOD 0, `src/render/shaders/shadow_mask.wgsl:145-146` |
| Primitive | matches | `src/assets/cooked.rs:499-517` (`CookedPrimitive`: `first_index`, `index_count`); `src/assets/cooked.rs:612-615` (each LOD's run per Primitive); `src/cook/mesh.rs:210` (a Primitive's vertices appended as one run) |
| Thin Primitive | matches | `src/assets/cooked.rs:511-514` (`thin_radius_metres`); `src/render/shaders/common.wgsl:223-228` (`thin_drawn_radius`: a pixel wide at the narrowest view); `src/render/shaders/common.wgsl:282-288` (`shadow_ray_mask`: a width class whose upper bound, twice its lower, is narrower than the pixel is passed through) |
| LOD | matches | `src/assets/cooked.rs:605-616` (`CookedLod`); `src/assets/cooked.rs:628-635` (LOD 0 is the source's indices) |
| LOD chain | matches | `src/assets/cooked.rs:599-600` ("LOD 1 and up, each coarser than the one before"); `MESH_MAX_LODS = 8`, `:41-42` |
| LOD error | matches (M5's L7-A01 fixed) | `src/assets/cooked.rs:608-610` ("the simplifier's error from LOD 0 down to it"); `src/render/shaders/shadow_mask.wgsl:127` |
| LOD threshold | matches | `src/render/renderer.rs:186-195` (`LOD_THRESHOLD_PIXELS = 1.0`, `lod_error_per_metre`); `src/render/shaders/common.wgsl:203-221` (`selected_lod`) |
| Material | matches | `src/assets/material.rs:81-91` (`MaterialData`: factors, `layers`, `extra` = emissive Layer) |
| Channel | matches | `src/assets/material.rs:58-79` (five Channels); `src/assets/material.rs:533-552` (colour Channels sRGB, data Channels linear); `src/assets/material.rs:598-599` (`colour_sampler`, `data_sampler`) |
| Layer | matches | `src/assets/material.rs:128-141` (`ClassAndSlice::layer`: class times `SLICES_PER_CLASS` plus slice); `src/assets/material.rs:522-526` (`DEFAULT_LAYER = 0`, a neutral 1x1 per Channel) |
| Texture | matches | `src/assets/material.rs:248-261` (`TexturePlacement`: one Texture's tail slice and chain slice, sampled at one Layer) |
| Texture class | matches | `src/assets/material.rs:93-100` (`TextureShape`: size, mip count, format); `src/assets/material.rs:150-158`, `:168-215` (slice count fixed when the class opens; a full class of a shape opens another) |
| Resident tail | matches (was `planned` at M3.5) | `src/assets/cooked.rs:249-253` (`resident_tail_level`); `src/assets/material.rs:248-261` (the tail's slice stays while the chain's comes and goes); `src/assets/material.rs:274-277` (only a chain gives way) |
| Pool | matches | `src/assets/pools.rs` (`MeshPool`), `src/assets/material.rs:484` (`MaterialPool`); `src/components.rs:43-47` (components hold Handles) |
| Handle | matches | `src/assets/mesh.rs:22-28` (`MeshHandle { index, generation }`); `src/assets/material.rs:42-56`; `src/assets/pools.rs:1028-1032` (`live(index, generation)`); `src/render/prepare.rs:162-164` (the GPU gets `index` alone) |
| Arena | matches | `src/assets/pools.rs:1317-1324` ("A GPU buffer of fixed capacity, never grown, sub-allocated in ranges"); `src/assets/pools.rs:202-206` (vertex, index, Meshlet, Meshlet vertex and Meshlet triangle Arenas); `src/assets/pools.rs:15-18` (every LOD's Meshlets, LOD 0's indices) |
| Resident | matches | `src/streaming/chunks.rs:15-19` (`ResidentFarProxies`: Meshes); no Chunk is called Resident (`grep -rni "resident chunk\|ResidentChunk" src`: none) |
| Material ID | drift (L7-G01) | `src/streaming/chunks.rs:54-59` (`ChunkPlacement::material`); `src/render/prepare.rs:953-963` (a Placement's override reaches its Instance) |
| Asset | matches (Chunk was `planned` at M3.5) | `src/assets/cooked.rs:72-80` (`AssetKind`: `Mesh`, `Material`, `Texture`, `Chunk`) |
| Asset ID | matches | `src/assets/cooked.rs:82-143` (`AssetSource::File(guid)`, `Scene(name)`, `AssetName::Index` "when the name is missing or not unique"); `src/scenes/mod.rs:345-356` (Chunks and Far proxies named under their scene) |
| Blob | matches | `src/assets/cooked.rs:1-4` ("Blobs named by the SHA-256 of their bytes") |
| Cook key | matches | `src/cook/mesh.rs:57`, `src/cook/texture.rs:135`, `src/cook/chunk.rs:101`, `src/cook/mod.rs:882` (`material_cook_key`), `:903` (`is_fresh`) |
| Cooked cache | matches | `src/assets/cooked.rs:1080-1081` (`blobs/<SHA-256>`, `index/<Asset ID>`) |
| Placement | matches (was `planned` at M3.5) | `src/assets/cooked.rs:850-860` (`CookedPlacement`: Mesh, `material` override, translation from the Chunk's origin) |
| Instance | matches (M3.5's L7-G05 fixed) | `src/render/prepare.rs:149-166` (`instance()`: Anchor and offset, Mesh index, override Material ID); `src/render/prepare.rs:294-298` (a Renderable's slot kept until it stops being one, a Chunk's range until it unloads); `src/render/prepare.rs:938-967` (`write_chunk`) |
| Moving Instance | matches | `src/render/prepare.rs:82-93` (`With<PreviousWorldTransform>` rewritten each frame); `src/render/prepare.rs:43-46` (`moving_instances` for the dynamic TLAS); `src/render/prepare.rs:129-130` (the static TLAS leaves them out) |
| Bend | matches (the milestone is consistent with ADR 0020) | `src/components.rs:49-69`; `src/render/bend_pass.rs:1-7` (a compute pass, "a stand-in for skinning"); M8 as `src/render/renderer.rs:119` and `docs/adr/0020-*.md:3` ("M8 builds on") |
| Shadow proxy | matches | `src/assets/pools.rs:118-120` (`MeshUse::Traced`: "a shadow proxy, which no pass draws") |
| Frame data | matches | `src/render/prepare.rs:23-50` (`FrameData`: camera, lights, changed `instances`, `moving_instances`, `bends`; "It lists no draws") |
| Froxel | matches | `src/render/froxel_pass.rs:1-4`, `:14-28` |
| Shadow mask | matches (M3.5's L7-G06 fixed) | `src/render/shadow_mask_pass.rs:30-35` (`SHADOW_MASK_LIGHTS = 59`); `src/render/shadow_mask_pass.rs:52-56` (`SHADOW_EDGE_LIGHTS = 7`) |
| Surface normals | matches | `src/render/targets.rs:42-45`; `src/render/main_pass.rs:141-161` (the depth pipelines' `fs_surface_normals` targets) |
| Meshlet | matches (M3.5's L7-G07 fixed) | `src/assets/mesh.rs:51-64` (`MeshletData`: no bounds); `src/assets/cooked.rs:43-49` (64 vertices, 124 triangles); `src/render/shaders/culling.wgsl:124-129` (Instance's Mesh bounds) |
| Temporal class | matches (no code counterpart) | `docs/adr/0020-*.md:3` ("Direct light is Temporal class (a); indirect light is class (b), and class (c) is used nowhere") |
| Interior volume | planned | `grep -rni "interior.volume" src bench tests`: none |
| Input latency | planned | `grep -rni "input.latency" src bench tests`: none |
| Tick | matches | `src/time/mod.rs:21` (`TICK = 15_625_000 ns`); `src/transform/mod.rs:26-30` (`drawn` blends the two newest Ticks) |
| ECS World | matches | `bevy_ecs::prelude::World`, as `src/render/prepare.rs:985` |
| Game world | matches (was `planned` at M3.5) | `src/streaming/mod.rs:1-4` (the Chunk streamer of a Game world) |
| Chunk | matches (was `planned` at M3.5) | `src/assets/cooked.rs:57-70` (`CHUNK_EDGE_METRES = 256`, `chunk_holding(pivot)`); `src/assets/cooked.rs:872-880` (`CookedChunk`: Placements, lights, bounds that may reach past the edge) |
| Load radius | matches | `src/streaming/mod.rs:311-321` (horizontal `distance_metres`); `src/streaming/mod.rs:59-61`, `:284-288` (a loaded Chunk outside it is held, not drawn) |
| Late Chunk | matches | `src/streaming/mod.rs:255-262` (counted once on entering the Load radius unloaded, arrivals aside); `src/streaming/io.rs:892-897` (placed once its BLASes are built and compacted) |
| Far proxy | matches | `src/cook/far_proxy.rs:1-2`, `:28-37` (static Placements merged, LOD 0 simplified to a pixel at the Load radius); `src/streaming/chunks.rs:15-19` (held past the View distance or while its Chunk is drawn) |
| Far field | matches | `src/scenes/mod.rs:556-580` (`load_radius_metres`, `view_distance_metres`) |
| Late Far proxy | matches | `src/streaming/mod.rs:78-81` ("its Late Chunks are the Late Far proxies"); `src/streaming/mod.rs:84-86`, `:183`, `:273` (a fill's Far proxies are not late) |
| Late mip | matches | `src/streaming/mips.rs:120-131` (late: what drawn Placements and Renderables need from the camera, not Resident); `src/streaming/mips.rs:51-62` (a Chunk held ahead is not late) |
| Fast travel | matches | `src/streaming/io.rs:675-705` (hold from the jump, then the Far field's fill, each timed); `src/stats/mod.rs:113-121` ("so its path pauses"); `src/streaming/mod.rs:182-183`, `:260` (no Late Chunk on arrival) |
| View distance | matches | `src/scenes/mod.rs:575-580` (`World => 2000.0`, else the Load radius) |
| World position | matches | `src/components.rs:6-12` (`translation: DVec3`, f64 metres) |
| World transform | matches | `src/transform/mod.rs:5-9` |
| Anchor | matches | `src/render/instance.rs:1-12` (`ANCHOR_CELL_SIZE = 64.0`, `anchor_and_offset`) |
| Scene | matches (serialisable: planned, as at M3.5) | `src/scenes/mod.rs` spawns into the `World`; no scene serialisation (`grep -rni "serialis\|serde" src` finds only the bench's) |
| Reference scene | matches | `src/scenes/mod.rs:33-36` (`ReferenceScene`, "each with a camera path in frames and a frame delta") |
| Golden | matches | `src/capture/mod.rs:35` (`GOLDEN_TOLERANCE: u8 = 1`), `:60` |
| Settled frame | matches | `src/bench/mod.rs:132-136` (`SETTLED_FRAMES = ..COUNTER_FRAMES.end`); `src/streaming/mod.rs:73-75` (`settled`); `src/streaming/io.rs:900-904` |
| Render switches | matches | `src/render/renderer.rs:75-116` (`RenderSwitches`: `culling`, `froxel_lights`, `shadow_rays`, `count_point_lights`, `crowd_form`, `residency`, `wait_for_static_tlas`, `forced_lod`; "The defaults are what ships") |
| Frame stats | drift (L7-G02) | `src/stats/mod.rs:13-128` (`FrameStats`); `src/stats/mod.rs:89-123` (the streaming fields the list omits); `src/stats/mod.rs:130-174` (`GpuFrameStats`); `src/render/readback.rs:643-653` (the frame total and the gate) |
| Timed pass | matches | `src/render/renderer.rs:38-53` (`TIMED_PASSES`, fixed, the build and refit among them); `src/render/readback.rs:719-728` (a pass not run takes no time) |
| Baseline | drift (L7-G03) | `bench/baselines/` (`apple-m4-pro`, `llvmpipe-*`, `nvidia-geforce-rtx-2070-super`, `-dx12`); `src/bench/history/import.rs:278-282` (`remeasured`); `src/bench/history/rerecord.rs:1-6`, `:176-182` (`praxis history rerecord` re-records counters alone) |
| Cycles Baseline | matches (see Notes) | `bench/cycles/town.json` (frame, size, commit, Blender, `mean_difference_steps`, `linear_rmse`); `scripts/cycles-reference/compare.py:85-92`, `:238-256`; `scripts/gallery.sh:12-22` |
| Bench record | matches | `src/bench/provenance.rs:44-81` (`Provenance`: checkout, driver, `harness_version`, `quiet`) |
| Hitch | matches | `src/bench/mod.rs:1577-1580` (`hitches`: over twice the median); `src/bench/mod.rs:200-213` (one count per kind); no Budget or compare reads them (`grep -n hitches src/bench/mod.rs`) |
| Bench history | matches | `src/bench/history.rs` (records, Series, experiments on `bench-history`) |
| Series | matches (M3.5's L7-G01 fixed) | `src/bench/provenance.rs:215-235` (`is_quiet`: preflight, busy processes, GPU frames against `NEIGHBOUR_FRAMES`); `src/preflight/mod.rs:25` (`BUSY_PROCESS_PERCENT = 50.0`) |
| Harness version | matches | `src/bench/provenance.rs:18-22` (`HARNESS_VERSION = 14`) |
| Experiment | matches | `src/bench/history.rs` (`experiments/<name>`, on no Series) |
| Suspect | drift (L7-G04); M3.5's L7-G02 fixed | `src/bench/history/suspects.rs:6-15` (`WINDOW_RECORDS = 8`, `FEWEST_JUDGING_RECORDS = 3`, `MEDIAN_ABSOLUTE_DEVIATIONS = 4.0`); `src/bench/history/suspects.rs:88-93`; `src/bench/history.rs:432-438` (3%, 1.25%); `src/bench/history/suspects.rs:20-24` (line starts) |
| Verdict | matches (M3.5's L7-G08 fixed by the `_Avoid_` line) | `src/bench/history/verdicts.rs:73-95`; `src/bench/history/verdicts.rs:768-787` (half the shift and 3%); `src/bench/history/verdicts.rs:603-610` (`--first-parent`); `src/bench/mod.rs:750`, `src/preflight/mod.rs:71` (the other `Verdict`s the line names) |
| Night | matches | `src/bench/history/night.rs:1-3`; `scripts/sweep.sh:2-18` (nightly on the M4 Pro) |
| Regression | matches (its "re-recorded" is the Baseline term's sense: see L7-G03) | `src/bench/history/verdicts.rs:501-506` (`file_regressions`: one `needs-triage` issue per scene and commit, recorded on its Verdicts) |
| Budget | matches (M3.5's L7-G03 and L7-G04 fixed) | `bench/budgets.json:1-18`; `src/bench/mod.rs:704-727` (`Budgets`, the guard "not a Budget"); `src/bench/mod.rs:1270-1357` (median, p95, p99, maximum, the guard); `src/render/renderer.rs:226-249` (`RenderCapacities` from the Budgets); `src/assets/pools.rs:37-40` (`FAR_PROXY_MATERIALS = 285`) |
| Capability floor | matches | `src/gpu/mod.rs:333-379` (`BelowCapabilityFloor`, every gap named) |
| Consolidation milestone | matches | `docs/agents/workflow.md:12`; `docs/agents/audit.md:3` (lanes 1, 4 and 7 at `M5.5`), `:12` (the scope since the last frozen commit) |
| Frozen commit | matches | `docs/agents/audit.md:11` |
| Audit lane | matches | `docs/agents/audit.md:3`, `:20-24` (brief, readers, verifier, report at `docs/audit/<milestone>/<lane>.md`) |
| Finding | matches (M3.5's L7-G09 fixed by the `_Avoid_` line) | `docs/agents/audit.md:30-39` (eight fields); `src/bench/mod.rs:755-761` (compare's `Finding`, the one the `_Avoid_` line names) |
| Severity class | matches | `docs/agents/audit.md:41-51` |
| Deviation | matches | `docs/agents/audit.md` (Deviations); `docs/audit/deviations.md:1-18` |

### Notes

- **Cycles Baseline**, `CONTEXT.md:259`: "fails a scene whose run exceeds either figure by its tolerance; each scene has its own, as its known differences differ." Read as "each scene has its own Cycles Baseline", it matches: `bench/cycles/` holds one file per exported scene. Read as "each scene has its own tolerance", it is false. The tolerances are two module constants shared by every scene, `MEAN_DIFFERENCE_TOLERANCE_STEPS = 0.01` and `LINEAR_RMSE_TOLERANCE = 0.0001` (`scripts/cycles-reference/compare.py:91-92`), set from run-to-run noise, not from known differences (`:85-90`). The nearest antecedent of "its own" is "its tolerance", so the sentence can mislead. I filed nothing because the clause's reason ("as its known differences differ") fits only the Baseline reading. The verifier may judge otherwise.
- `TexturePlacement` (`src/assets/material.rs:248-261`), with `ClassTable::release(placement: ClassAndSlice)` (`:224`), uses "placement" for a Texture's slices in its Channel. That is not the glossary's Placement. The item is private, so the CLAUDE.md rule on same-named `pub struct`/`enum` items does not reach it. No Finding.
- **Bend**'s "until M8" cannot be settled from the tree. It agrees with the ladder in `README.md:62` (M8 Gameplay core), `docs/adr/0020-*.md:3` and `src/render/renderer.rs:119`.
- **Load radius** says Chunks "are loaded" within it. The streamer also requests ahead of it and holds Chunks to one Chunk edge past it (`src/streaming/mod.rs:230-232`, `:296-298`). The definition names where Chunks must be loaded and drawn, which the code keeps, so it is not drift.
- Left to reader R: `docs/agents/audit.md` and `workflow.md` as documents. I read them only as the defining text of the audit terms.

## The rules

I read all of these at `e5c55a8da2a59db2f09c99c8ea4522987d5354ac` in `/Users/guillem.arias/Documents/gariasf/praxis-1004`: `CLAUDE.md` (352 lines) and the eight files of `docs/agents/` (`workflow.md`, `project-shape.md`, `machine.md`, `sessions.md`, `audit.md`, `issue-tracker.md`, `domain.md`, `triage-labels.md`). Two of them, `machine.md` and `project-shape.md`, landed in scope. `git diff 8bc34eb1 HEAD` over the same paths and `git log 8bc34eb1..HEAD` (238 commits) set the scope. Every path, file, module, command, flag, number, label, branch and test the text names was checked against the tree:

- the Stack: `Cargo.toml` and `Cargo.lock`, and `vendor/README.md` for the vendored crate;
- the Project shapes and the Documents: `git ls-files` over `src/`, `tests/`, `scripts/`, `bench/`, `docs/` and `.github/`, plus the `//!` header of each file that landed in scope;
- every "tested" clause, traced to a test function by name: `tests/conventions.rs`, `tests/goldens.rs`, `tests/field_guides.rs`, `tests/references.rs`, `tests/asset_manifest.rs`, `tests/dependency_licences.rs`, and the two hash tests in `src/cook/`;
- `workflow.md`: `scripts/verify.sh`, `scripts/pr-lint.sh`, `scripts/gallery.sh`, `scripts/sweep.sh`, `scripts/lavapipe-files.sh`, the three workflows under `.github/workflows/`, `.github/pull_request_template.md`, `src/bin/praxis.rs` (its parser and usage string), `src/preflight/mod.rs`, `src/bench/` (`compare` modes, `history/verdicts.rs`'s Baseline coverage) and `src/bench/provenance.rs`;
- the Review checklist: `grep` over `src/` for every identifier it names, and every `create_buffer` and `create_texture` outside the ledger;
- the import conventions: every non-glob `use bevy_ecs::` and every `use wgpu::` in `src/`, held against the bevy_ecs 0.20.1 prelude under `~/.cargo/registry`;
- `~/.claude/skills/` for every skill named;
- `gh label list`, and `gh issue view` for the two track issues `workflow.md` names.

Each clause M3.5's lane 7 had filed (its L7-R01 to L7-R04) was re-read; all four now read true. Clauses about process, people, chat or GitHub state are `not checkable from the tree`. Nothing in either checkout was changed and no cargo, script or praxis binary was run.

### CLAUDE.md

| Section | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| Preamble | "agents implement, commit, open PRs and merge them once reviewed, without asking ... (`docs/agents/workflow.md`)" | not checkable from the tree (process); the file says the same | `docs/agents/workflow.md:5` |
| Project posture | "Load-bearing decisions (`docs/adr/`)" | matches: 27 ADRs, 0001 to 0027 | `docs/adr/0001-material-as-id-not-bind-group.md` ... `docs/adr/0027-coverage-in-the-main-pass-is-a-written-sample-mask.md` |
| Project posture | "a prototype ticket's branch, which is never merged (`docs/agents/workflow.md`, Tickets)" | matches | `docs/agents/workflow.md:25` |
| Project posture | "Work traces to a ticket in the live milestone"; "a field guide for each session's important new concepts" | not checkable from the tree (process) | `docs/agents/workflow.md:9,32` |
| Documents | "`docs/agents/workflow.md` — how work moves: ladder, tickets, PRs, budgets, session start" | matches: headings The ladder, Tickets, PRs, Review, Performance and memory, Correctness, Session start, Context hygiene | `docs/agents/workflow.md:7,16,27,38,53,63,69,77` |
| Documents | "`docs/agents/sessions.md` — which skill starts which kind of session" | matches | `docs/agents/sessions.md:1-16` |
| Documents | "`docs/agents/machine.md` — how sessions share one machine: claims, worktrees, quiet turns" | matches: headings Claims, Worktrees, Quiet turns, Timed runs | `docs/agents/machine.md:5,11,18,28` |
| Documents | "`CONTEXT.md` — glossary"; "`docs/adr/` — decisions that are hard to reverse" | matches | `CONTEXT.md`; `docs/adr/` (27 files) |
| Documents | "`docs/references.toml` ... code, ADRs and notes cite its keys (`[ref:<key>]`)" | matches: 237 bracketed citations under `src/` and `docs/adr/`; no unbracketed `ref:` key form in `src/` | a grep of `src` and `docs/adr` for the bracketed citation form |
| Documents | "`docs/guides/` — ... a page per mechanism and per concept ... a page describes the commit its *Describes* line names" | matches: 33 pages, every one carries a *Describes* line; hosting not checkable | `grep -L Describes docs/guides/*.html` (none) |
| Documents | "`docs/bench/` — the page that draws the Bench history's Series from `bench-history` (ADR 0024)" | matches | `docs/bench/index.html:166-167` |
| Documents | "`docs/audit/` — `deviations.md` ...; `<milestone>/<lane>.md`; `strategy/<year>-<month>.md`, a strategy review ... with its Decision queue" | matches: `deviations.md` (four rows), `m3.5/`, `m5/`, `strategy/2026-11.md` with "## Decision queue" | `docs/audit/deviations.md:20-25`; `docs/audit/strategy/2026-11.md:46` |
| Documents | "`ROADMAP.md` — long-range vision: target shape, design principles, scripting plan" | matches | `ROADMAP.md:35,67` |
| Agent skills, Issue tracker | "GitHub Issues ... via the `gh` CLI. See `docs/agents/issue-tracker.md`" | matches | `docs/agents/issue-tracker.md:1-3` |
| Agent skills, Triage labels | the five roles and `fog`, "each label string equals its role name" | matches `triage-labels.md`; `gh label list` holds all six | `docs/agents/triage-labels.md:5-12` |
| Agent skills, Audit | "shared brief, parallel read-only readers, adversarial verifier, report, triage. See `docs/agents/audit.md`" | matches | `docs/agents/audit.md:3,18-24` |
| Agent skills, Domain docs | "one `CONTEXT.md` plus `docs/adr/` at the repo root" | matches: no `CONTEXT-MAP.md` | `ls -a` of the root |
| Stack | "Rust (edition 2024)" | matches | `Cargo.toml:4` |
| Stack | "wgpu 30, WGSL shaders" | matches: lock `wgpu` 30.0.2 | `Cargo.toml:8`; `Cargo.lock` |
| Stack | "winit 0.30"; "glam" | matches: winit 0.30.13; glam 0.34 (lock 0.34.3, a transitive 0.33.12 beside it) | `Cargo.toml:9-10` |
| Stack | "`bevy_ecs` (sub-crate only) ... with `bevy_tasks` for its compute pool" | matches: `bevy_ecs` 0.20.1, `bevy_tasks` 0.20.1 | `Cargo.toml:17-18` |
| Stack | "`gltf`, `image`, `bevy_mikktspace`, `intel_tex_2`, all four behind the `cook` feature; Blobs via `ktx2`, `zstd`, `meshopt` and `sha2`, Arenas via `offset-allocator`" | matches: the four `optional = true` and in `cook = [...]`; the five non-optional | `Cargo.toml:15-16,20,23-28,41` |
| Stack | "`tracing` + `tracing-subscriber` (with `env-filter`)"; "Single crate: a library plus a thin binary" | matches | `Cargo.toml:11,19`; `src/lib.rs`, `src/bin/praxis.rs` |
| Stack | "until wgpu 31 ships, a fix ... is carried on a vendored 30.0.2 crate under `vendor/`, behind `[patch.crates-io]`; `vendor/README.md` names each upstream commit applied" | matches: `vendor/wgpu-core` 30.0.2, one commit named | `Cargo.toml:53-54`; `vendor/README.md:12-35` |
| Stack | "Version bumps ... owner decisions"; "Each milestone's exit reviews the releases ... checks ADR 0007's reopen trigger" | not checkable from the tree (process); the clause is in `workflow.md` | `docs/agents/workflow.md:13` |
| Project shape | "lib.rs, state.rs # library root (window shell) + orchestrator"; "bin/praxis.rs # thin binary: command dispatch"; "gpu/ # GPU context, windowed or headless" | matches | `src/lib.rs:18-38`; `src/bin/praxis.rs:294-411` |
| Project shape | "capture/ # offscreen captures, Goldens, image difference"; "preflight/ # `praxis preflight`" | matches | `src/capture/mod.rs`; `src/bin/praxis.rs:297` |
| Project shape | "bench/ # bench, compare, report, traverse, the Bench history" | matches: `traverse` in `bench/mod.rs`, `history/` | `src/bench/mod.rs:836`; `src/bench/history.rs` |
| Project shape | "stats/ # frame stats, GPU memory ledger, counting allocator" | matches | `src/stats/gpu_memory.rs`; `src/stats/mod.rs:492` |
| Project shape | "streaming/ # the Chunk streamer, its IO pool, mip priority" | matches | `src/streaming/{mod,io,mips,chunks}.rs` |
| Project shape | "scenes/"; "components.rs"; "threading.rs # compute pool, schedule executor" | matches | `src/scenes/{mod,town,world}.rs`; `src/threading.rs:6` |
| Project shape | "camera/, time/, input/, helmet/, transform/, environment/ # feature modules: resource (+ systems) in one mod.rs" | matches: each holds only `mod.rs` | `git ls-files src` |
| Project shape | "assets/ # asset types, Pools, the Cooked cache reader, loader"; "cook/ # the cooker (`cook` feature): cook, fetch, export-scene" | matches | `src/assets/*.rs`; `src/cook/{mod,fetch,export}.rs`; `Cargo.toml:41` |
| Project shape | "render/ # Renderer, a module per pass, targets, prepare, bind_groups, readback, shaders/" | matches | `src/render/*.rs` |
| Project shape | "tests/ # integration tests at agreed seams; goldens/" | matches: 13 test files, `goldens/{metal,vulkan,vulkan-cpu}` | `tests/` |
| Project shape | "bench/ # budgets.json, baselines/<adapter>/, cycles/" | matches: four adapter directories, eleven Cycles Baselines | `bench/` |
| Project shape | "scripts/ # verify, gallery, pr-lint, sweep, lavapipe-files, cycles-reference/" | matches (also `pr-lint-test.sh`, `sweep.plist`, named in `project-shape.md`) | `scripts/` |
| Project shape | "vendor/ # crates.io crates carrying upstream fixes" | matches | `vendor/wgpu-core` |
| Project shape | "What each holds, file by file and pass by pass: `docs/agents/project-shape.md`" | see the `project-shape.md` rows | `docs/agents/project-shape.md` |
| Project shape | "Rendering pulls from `World` (Design Principle #3) ... the Renderer draws only that, reading no `World` (ADR 0004). ECS types never hold GPU buffers (Design Principle #1)" | matches: Principles 1 and 3; `World` in `renderer.rs` only in its test module; no `wgpu::` in `components.rs` or the feature modules | `ROADMAP.md:39,49`; `src/render/renderer.rs:1040,2361` (tests) |
| Project shape | "When a feature folder grows past one file, split into `feature/{resource,system,...}.rs`. Don't pre-split" | matches: the six feature folders are one file each | `git ls-files src` |
| How agents work here | "A PR whose gallery reports a difference, that moves a Golden or changes a `.wgsl` file, or that needs a visual check, shows before and after images (`docs/agents/workflow.md`, PRs)" | matches `workflow.md`'s rule | `docs/agents/workflow.md:35` |
| How agents work here | "Work the live milestone's frontier"; "A bug-fix is not a refactor ..."; "Don't add comments ... to code you didn't change" | not checkable from the tree (process) | — |
| How agents work here | "`/implement` closes with a local `/code-review`" | matches: the skill's step calls `code-review` | `~/.claude/skills/implement/SKILL.md:14` |
| How agents work here | "The one licence to refactor is an audit's code-health fix ticket ... Goldens bit-exact, counters equal, timings within noise" | matches `audit.md`'s Lane 5 | `docs/agents/audit.md:105` |
| How agents work here | "`cargo fmt` and `clippy` own style"; "skills: `codebase-design`, `tdd`" | matches: both skills installed | `~/.claude/skills/{codebase-design,tdd}` |
| Code conventions | "`cargo fmt` and `cargo clippy` must pass" | matches: verify's first two steps | `scripts/verify.sh` (steps `fmt`, `clippy`) |
| Code conventions | "never `println!` or `log` (tested)" | matches: rule "print or `log` instead of `tracing`", function `printing` | `tests/conventions.rs:18,158` |
| Code conventions | "Prefer structured fields" | not checkable from the tree (judgement) | — |
| Code conventions | "Default log filter is `praxis=debug,wgpu=warn,naga=warn` via `.cargo/config.toml`" | matches | `.cargo/config.toml:3` |
| Code conventions | "Bevy imports: `use bevy_ecs::prelude::*;` for what the prelude exports; items outside it by path" (M3.5's L7-R02, reworded since) | matches: the four by-path imports (`RunSystemOnce`, `SingleThreadedExecutor`, `LogLevel`, `ScheduleBuildSettings`) are outside the 0.20.1 prelude | `src/state.rs:7`; `src/threading.rs:6`; `src/time/mod.rs:4`; `bevy_ecs-0.20.1/src/lib.rs` (`pub mod prelude`) |
| Code conventions | "WGPU types: wgpu's own types by full path; wgpu-hal's `api::Metal` and traits imported `as _` excepted" (M3.5's L7-R03, reworded since) | matches: shipping imports are the `hal` module, `api::Metal` and `as _` traits; the rest sit in test modules | `src/render/blas_refit.rs:166,279`; `src/render/acceleration_structures.rs:128`; `src/stats/gpu_memory.rs:388` |
| Code conventions | "Identifiers are full words, WGSL included (... shading names tested)" | matches: rule "abbreviated shading name" | `tests/conventions.rs:26,326` |
| Code conventions | "Pool lookups return `Option<&T>` ... never `.unwrap()` in a hot loop" | matches: `get`, `blas`, `handle_at` return `Option`; `render_unwraps` | `src/assets/pools.rs:857,881,1012,1028`; `tests/conventions.rs:38,464` |
| Code conventions | "Shaders are WGSL parts in `src/render/shaders/`, assembled into modules by its `mod.rs` ...; a function under an oracle lives in a shared part" | matches | `src/render/shaders/mod.rs` |
| Code conventions | "No `unsafe` unless absolutely necessary and clearly explained" | matches: every `unsafe` block in shipping code sits under a `SAFETY` comment, the new ones included | `src/gpu/mod.rs:72-78`; `src/render/shaders/mod.rs:242-246`; `src/render/acceleration_structures.rs:129-130` |
| Code conventions | "Use feature flags (`image = { version = "0.25", default-features = false, features = ["png"] }`)" | matches | `Cargo.toml:15,32` |
| Code conventions | "Tests use the built-in harness ... Test-framework crates are an owner decision" | matches: dev-dependencies `image`, `texture2ddecoder`, `wgpu-naga-bridge` | `Cargo.toml:30-35` |
| Code conventions | "Every file under `assets/` has an entry in `assets/manifest.json` and its credit in `assets/ATTRIBUTION.md`; `cargo test` checks both. Policy: `docs/research/test-assets.md`" | matches | `tests/asset_manifest.rs:37` |
| Code conventions | "Every dependency's licence expression is on the allowlist in `tests/dependency_licences.rs`" | matches | `tests/dependency_licences.rs:37` |
| Code conventions | "Docs, tickets and code comments name no patents" | not checkable mechanically | — |
| Code conventions | "Comments ... citing no ticket, PR or decision ... (ticket numbers tested)" | matches: rule "ticket number in a comment" | `tests/conventions.rs:22,182` |
| Review checklist | "A size-dependent target lives in the targets module (`src/render/targets.rs`), whose `RenderTargets::new` creates every target and runs again on resize ... a pass builds the bind groups that read targets in one function, called at creation and after every resize" | matches: `resize` rebuilds the targets then calls each pass's `bind_render_targets` | `src/render/renderer.rs:412,629-645` |
| Review checklist | "A binding is written once, in its bind group's definition (`src/render/bind_groups.rs`) ...; no WGSL part declares one ... Rust-only padding fields start with `_`" | matches: no `@group(` in any `.wgsl` part | `git grep '@group(' -- 'src/render/shaders/*.wgsl'` (nothing) |
| Review checklist | "No World position reaches the GPU in f32: only an anchor plus an f32 offset (ADR 0014)" | matches | `src/render/instance.rs:8` |
| Review checklist | "Depth has no far plane, and WGSL that compares or reduces depth keeps reversed depth's direction (ADR 0015)" | matches as far as the tree shows: rule "depth not cleared to 0 or compared with `Less`"; the WGSL half is judgement | `tests/conventions.rs:34,415` |
| Review checklist | "Light values are photometric ... HDR colour is pre-exposed before fp16 stores it and clamped to 65504 (ADR 0023)" | matches | `src/render/shaders/hdr.wgsl:7`; `src/render/targets.rs:377-380` |
| Review checklist | "Matrices reach WGSL column-major (glam `to_cols_array_2d`)" | matches | `src/render/prepare.rs:610` |
| Review checklist | "A new or changed shading function in WGSL has an oracle test in `src/render/shader_function_tests.rs`" | matches: the file exists | `src/render/shader_function_tests.rs` |
| Review checklist | "sRGB is encoded once: the tonemapping pass alone encodes it, in its shader, into 8-bit UNORM targets" | matches: `srgb_encode` called only from `tonemapping.wgsl`; the surface test | `src/render/shaders/tonemapping.wgsl:19`; `src/gpu/mod.rs:692` |
| Review checklist | "Outside `src/render/`, no `unwrap()` on a GPU operation that could fail at runtime" | matches: no shipping `.unwrap()` on a device, queue, buffer, map, texture or surface call outside `src/render/` | grep over `git ls-files 'src/*.rs'` above each file's first `#[cfg(test)]` |
| Review checklist | "No GPU resources stored on ECS components" | matches | `src/components.rs:11-93` |
| Review checklist | "Code adapted from elsewhere cites its source and licence ... (ADR 0025)" | matches as far as the tree shows: `license = "MIT"`, `LICENSE`; the rest is judgement | `Cargo.toml:5`; `LICENSE:1` |
| Review checklist | "A cost that rejects a technique ... final only with the figure on the M4 Pro and the RTX 2070 SUPER (ADR 0013)" | not checkable from the tree (process) | — |
| Review checklist | "Every reference ... (`docs/agents/workflow.md`, Review: Sources)" | matches: the heading exists | `docs/agents/workflow.md:43` |
| Review checklist | "Every name the change renames ... is grepped in comments, ADRs, `CONTEXT.md` and `docs/research/` ... a new glossary term is grepped against `pub struct`/`enum` names" | not checkable from the tree (process); the directories exist | `docs/research/` |
| Review checklist | "Frame data holds no borrow into the ECS World, and only the main thread submits, presents or polls (ADR 0016)" | matches: `FrameData` has no lifetime; the IO pool holds no `wgpu` handle | `src/render/prepare.rs:27`; `src/streaming/io.rs` |
| Review checklist | "Whatever changes what an Instance holds makes prepare write its slot; other slots keep what earlier frames wrote (ADR 0003)" | matches | `src/render/prepare.rs:39-40` |
| Review checklist | "The GPU builds the frame's draw in slot order within each culling phase" | matches | `src/render/culling_pass.rs:6,146` |
| Review checklist | "GPU buffers, textures and acceleration structures are created through the GPU memory ledger (`GpuMemory`) under their kind, ... (`blas_bytes`, `tlas_bytes`), a compacted BLAS at its adapter's estimated fraction; the GPU stats readback's buffers stay out ...; per-frame GPU writes add to `uploaded_bytes`, and systems are added inside `SystemTimer::scope`" | drift (L7-R02): the load staging's belt buffers also stay out of the ledger, by design, and the checklist names only the readback's | `src/render/staging.rs:179-187`; names: `src/stats/gpu_memory.rs:113`, `src/render/acceleration_structures.rs:36,58,70`, `src/render/renderer.rs:698,781`, `src/stats/mod.rs:361` |
| Review checklist | "Streaming uploads go through the `StagingBelt`, never `Queue::write_texture`; no GPU buffer grows at runtime ... Only per-frame staging grows, up to the buffers a frame writes, which it asserts (ADRs 0003, 0017)" | matches: `LoadStaging` writes through a `StagingBelt`; no `queue.write_texture` in `src/`; `FrameStaging` asserts its ceiling | `src/render/staging.rs:13-16,81,185-187,344-348` |
| Review checklist | "The runtime loads cooked Blobs only and references Assets by Asset ID (ADR 0018)" | matches | `src/assets/loader.rs` (`AssetId`) |
| Review checklist | "the two hash tests (`src/cook/mesh.rs`, `src/cook/far_proxy.rs`) ... a C++ build that fuses a multiply and an add (`.cargo/config.toml`)" | matches | `src/cook/mesh.rs:993`; `src/cook/far_proxy.rs:816`; `.cargo/config.toml:4-6` |
| Review checklist | "no change rules out its 4x MSAA (ADRs 0008, 0019, 0020)" | matches: `SAMPLE_COUNT = 4` | `src/render/targets.rs:16` |
| Review checklist | "No main-pass pipeline enables alpha-to-coverage. Partial coverage is a `sample_mask` the depth passes write, through pipelines of their own ... (ADR 0027)" | matches: `alpha_to_coverage_enabled: false`; `fs_thin_depth` returns the mask | `src/render/main_pass.rs:113`; `src/render/shaders/main.wgsl:187` |
| Review checklist | "A newly used wgpu feature or limit joins the Capability floor ... (ADR 0021)" | matches: the floor functions and the matrix exist | `src/gpu/mod.rs:269,310`; `docs/research/wgpu-capability-matrix.md` |
| Review checklist | "Game-state changes run in the fixed schedule, which reads no wall clock and has no ambiguous system order (ADR 0022); ... `PreviousWorldTransform`, ... `Teleported` before `reset_teleported` runs" | matches: `ambiguity_detection: LogLevel::Error` | `src/time/mod.rs:69-75`; `src/transform/mod.rs:59,66,89` |
| Review checklist | "WGSL keeps sRGB and linear apart ... no `textureSample` under non-uniform control flow" | not checkable mechanically (judgement; five `textureSample` sites) | `git grep textureSample -- 'src/render/shaders/*.wgsl'` |
| Review checklist | "Workflows keep least-privilege permissions and pin each action to a commit SHA with its version in a comment, as `pr-lint.yml` does" (M3.5's L7-R04, reworded since) | matches: all three workflows pin by SHA with a comment, `contents: write` only on `publish-bench`; see Notes on `# stable` | `.github/workflows/ci.yml:37-38,335-336`; `.github/workflows/lavapipe-files.yml:9-10,26-30`; `.github/workflows/pr-lint.yml:14-16,29` |
| Maintenance | "Update this file in the same PR when: Folder layout changes: a directory here, a file in `docs/agents/project-shape.md` ..." | not checkable from the tree (process); see L7-R01 for files that landed without a line | `docs/agents/project-shape.md` |
| Agent fleet | "No `AGENTS.md`, `.cursor/rules/`, `.junie/guidelines.md`, or similar" | matches | `ls -a` of the root |

### docs/agents

| File, heading | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| workflow.md, intro | "A decision that lives only in chat is not made: it lands as an ADR (`docs/adr/`), a comment on the map issue, or a ticket comment" | not checkable from the tree (GitHub) | `docs/adr/` |
| workflow.md, The ladder | "Exactly one is **live**: the lowest-numbered open milestone" | not checkable from the tree (GitHub); preflight reads the milestones through `gh` | `src/preflight/mod.rs:700` |
| workflow.md, The ladder | "`track:world-systems`, #944, open since 2026-11-21 ... the day `track:observability`, #427, closed"; "label `wayfinder:map`" | matches GitHub: the world-systems track's map open with `wayfinder:map` and `track:world-systems`; the observability track's map closed 2026-11-21; the labels exist | `gh issue view` on the two issues the clause names, `gh label list` |
| workflow.md, The ladder | "A **Consolidation milestone** ... along the Audit lanes of `docs/agents/audit.md` that the decision opening it names (all eight at `M3.5`)" | matches: eight lane briefs; `audit.md` names lanes 1, 4 and 7 at `M5.5` | `docs/agents/audit.md:3,71-126` |
| workflow.md, The ladder | "the verify command (`scripts/verify.sh`) exits 0 within 30 lines" | matches: one line per step, seven at most on success | `scripts/verify.sh:74-80` |
| workflow.md, The ladder | "Goldens committed on Metal and software Vulkan"; "a profile ... in `docs/performance.md`" | matches | `tests/goldens/{metal,vulkan-cpu}/`; `docs/performance.md` |
| workflow.md, The ladder | "it checks ADR 0007's three reopen conditions: ... Capability floor, `vendor/` carries more than five upstream commits or a fix upstream declined, or the pinned wgpu is more than two major releases behind" | matches ADR 0007's three | `docs/adr/0007-wgpu-is-the-permanent-graphics-api.md:9` |
| workflow.md, The ladder | "The audit skill, `docs/agents/audit.md`, is the standing form of the visual audit and the source check: its image and drift lanes" | matches: lanes 2 and 7 | `docs/agents/audit.md:79,115` |
| workflow.md, The ladder | "Reordering is a decision comment on the map issue" | not checkable from the tree | — |
| workflow.md, Tickets | "Size ceiling: 800 changed lines of code, which CI's PR form check counts: every changed line outside `docs/`, the root's `.md` files, Goldens, the lockfile, `bench/baselines/`, `bench/cycles/` and `vendor/`" | matches: `code_line_limit=800` and the uncounted pattern | `scripts/pr-lint.sh:108-113` |
| workflow.md, Tickets | "A ticket is written at about 400, where three PRs in four have landed" | not checkable from the tree (history) | — |
| workflow.md, Tickets | "verification seam agreed before coding (`/tdd`)"; "the form is in `docs/agents/issue-tracker.md`" | matches: the skill is installed; the form is there | `~/.claude/skills/tdd`; `docs/agents/issue-tracker.md:16-47` |
| workflow.md, Tickets | "blocked by a `wayfinder:research` ticket ... (`docs/research/wgpu-capability-matrix.md`, ADR 0013 ..., ADR 0008 ...)"; "the **prototype ticket** (label `wayfinder:prototype`)" | matches as far as the tree shows: the files and both labels exist | `docs/research/wgpu-capability-matrix.md`; `gh label list` |
| workflow.md, PRs | "One ticket per PR (`Closes #N`)" | matches: pr-lint's ticket line | `scripts/pr-lint.sh:36-38` |
| workflow.md, PRs | "`scripts/lavapipe-files.sh` has CI record them in a run that judges nothing and copies into the tree the Goldens and Baselines CI would fail, and the Baselines a judged counter fell below ... a Baseline's counter that only fell, which CI passes" | matches: the script dispatches `lavapipe-files.yml` on a branch rebased onto `origin/main` and copies its artifacts; `compare --recording` fails a fallen counter the gate passes | `scripts/lavapipe-files.sh:16-30,51-62`; `src/bench/mod.rs:745,1194,1989` |
| workflow.md, PRs | "The verify command ... formatting, lint, build, tests (Goldens required) and the bench counters against Baselines and Budgets, one line per step" | matches; "about a minute" not checkable | `scripts/verify.sh:40-80` |
| workflow.md, PRs | the field guides' page shape (*Describes*, *Interface* with *Files*, *How it is tested*, ... *History*) | matches as a target: every page carries *Describes*; 27 of 33 an *Interface* box, the rest concept pages and the hub | `docs/guides/*.html` |
| workflow.md, PRs | "`cargo test --test field_guides` lists each stale page ...; the list fails nothing" and "checks every link and image ..., and of every page, the hub included, that its *Describes* commit is an ancestor of `main`, that every identifier it names in code font exists on `main`, and that its prose names no ticket or PR; of every page with an Interface box, that its *Files* row lists files that exist" | matches: `guides()` takes every `.html`, the hub included; `VOLUMES` is empty | `tests/field_guides.rs:1-18,327-340,378-436` |
| workflow.md, PRs | "Description follows `.github/pull_request_template.md` ... CI's PR form check (`scripts/pr-lint.sh`) fails a title, description or commit that breaks this section's rules or Review's counts and "Sources checked" line, and a PR over the size ceiling" | matches | `.github/pull_request_template.md:5,9,15,19,23`; `scripts/pr-lint.sh:36-73,108-144`; `.github/workflows/pr-lint.yml:42-60` |
| workflow.md, PRs | "`praxis capture <scene> <png> [<width>x<height>] [--frame <n>[,<n>...]]` renders a scene headless, ... in one run" | matches the usage (which adds `[--lod <n>]`) | `src/bin/praxis.rs:419` |
| workflow.md, PRs | "The images go on the `pr-images` branch" | matches: the branch exists locally and on origin | `git branch -a` |
| workflow.md, PRs | "A PR that changes `src/render/` ..., `src/cook/texture.rs`, `src/cook/mesh.rs` or ADR 0019 or 0020 runs `scripts/gallery.sh` twice ... prints for each image how many pixels differ ... and the largest channel delta ... `22 of 22 gallery images identical to the base's` ... CI's PR form check ... asks ... for the three links when a Golden moves or a `.wgsl` file changes, and otherwise takes either the three or that line with its two counts equal" | matches | `scripts/pr-lint.sh:93-107`; `src/bin/praxis.rs:1052`; `scripts/gallery.sh:41-44,135-139` |
| workflow.md, PRs | "also carries the Cycles images ... with `PRAXIS_BLENDER` naming Blender 5.2 (`scripts/cycles-reference/`) ... `scripts/cycles-reference/DIFFERENCES.md`" | matches | `scripts/gallery.sh:12-27`; `scripts/cycles-reference/DIFFERENCES.md` |
| workflow.md, PRs | "Commit subject 72 characters at most ... No commit names an AI co-author or carries a "Generated with Claude" line ... the title is 72 characters at most with that number" | matches: `check_message` over the squash commit and each commit | `scripts/pr-lint.sh:116-144` |
| workflow.md, Review | "No external reviewer runs (CodeRabbit was removed, 2026-10-14)" | matches as far as the tree shows: no CodeRabbit configuration; three workflows | `.github/workflows/` |
| workflow.md, Review | "run `/code-review` ...: a Standards axis ... and a Spec axis ..., plus a third sub-agent beside them for a Sources axis" | not checkable from the tree: the installed skill runs two axes; the third is the session's own (as at M3.5) | `~/.claude/skills/code-review/SKILL.md:3,6` |
| workflow.md, Review | "The Standards sub-agent runs at medium effort, the Spec and Sources sub-agents at high (`docs/agents/sessions.md`, Effort)" | matches | `docs/agents/sessions.md:25-26` |
| workflow.md, Review | "`cargo test --test references` fails on a dangling key or an unused entry" | matches | `tests/references.rs:15,32` |
| workflow.md, Review | "Fill the PR template, with the review's counts ... and a "Sources checked" line"; "`gh pr merge <n> --squash --subject ... --body-file <file>`" | matches for the counts and line (pr-lint); the merge is process | `scripts/pr-lint.sh:51-73` |
| workflow.md, Review | "A declined finding class that recurs becomes a line in the `CLAUDE.md` review checklist or a `CODING_STANDARDS.md` rule" | not checkable from the tree (process); no `CODING_STANDARDS.md`, which Correctness allows | `docs/agents/workflow.md:66` |
| workflow.md, Performance and memory | "Budgets and Baselines live in the repo beside the benches (`bench/`)" | matches | `bench/budgets.json`; `bench/baselines/` |
| workflow.md, Performance and memory | "The sweep files a Regression only for a commit that did not re-record its scene's Baseline for the Series' adapter and backend" | matches: `covered_by_baseline` reads the scene's file in the adapter-and-backend directory at the pinned commit and its parent; no issue when covered | `src/bench/history/verdicts.rs:415-425,736`; `src/bench/mod.rs:146-152` |
| workflow.md, Performance and memory | "pasted from `praxis report <results directory>`" | matches | `src/bin/praxis.rs:408` |
| workflow.md, Performance and memory | "the two files' `provenance.source_hash` are equal" | matches: the field exists | `src/bench/provenance.rs:241` |
| workflow.md, Performance and memory | "`praxis bench` publishes ... unless given `--no-publish`, and `praxis publish <results directory>` publishes them later. With `--experiment <name>`, either publishes ..." | matches: flags limited to `bench` and `publish` | `src/bin/praxis.rs:226-251,362` |
| workflow.md, Performance and memory | "`verify` never publishes. CI's `publish-bench` job publishes lavapipe's records and the release build's size and wall time on every push to `main`; PR runs publish nothing" | matches | `scripts/verify.sh:54`; `.github/workflows/ci.yml:284,293,329-353` |
| workflow.md, Correctness | "Mechanical rules belong in CI. `CODING_STANDARDS.md`, when it exists" | matches: absent | `ls` of the root |
| workflow.md, Correctness | "`scripts/gallery.sh [<earlier gallery>]` captures every Reference scene at 2560×1440 into `target/tmp/gallery/` (`town` at the Goldens test's eight frames, `world` at its four Goldens' frames, `crowd` in both shadow forms)" | matches: `town_frames` and `world_frames` equal `TOWN_FRAMES` and `WORLD_FRAMES` | `scripts/gallery.sh:41-44,147-158`; `tests/goldens.rs:319,373` |
| workflow.md, Correctness | "with `PRAXIS_BLENDER` set ... where `export-scene` exports the scene" | matches: crowd skipped | `scripts/gallery.sh:141-143` |
| workflow.md, Session start | "`cargo run --release -- preflight` ... ten lines or fewer: the adapter and which machine it is, ... Goldens, Baselines, fetched files and Cooked cache ..., GPU load, the one-minute load average, other praxis processes and any other process at half a core or more, the open PRs and the live milestone's assigned tickets" | matches: ten checks at most (macOS: gpu load, cpu load, processes, power, adapter, goldens, baselines, assets, prs, tickets); `BUSY_PROCESS_PERCENT = 50.0` | `src/preflight/mod.rs:25,112-147,662-688` |
| workflow.md, Session start | "A `FAIL` line blocks `scripts/verify.sh` and names its fix; a `warn` line holds off a bench" | matches preflight's own statement | `src/preflight/mod.rs:1-3` |
| workflow.md, Session start | the `gh` orient and frontier commands; "(`docs/agents/machine.md`, Claims)" | not checkable from the tree (GitHub); the heading exists | `docs/agents/machine.md:5` |
| workflow.md, Session start | "`cargo run --release -- fetch` ... the failing step's log is in `target/verify/`" | matches | `src/bin/praxis.rs:305`; `scripts/verify.sh:9` |
| workflow.md, Context hygiene | "Validation commands print 30 lines at most on success"; "Research results are cached in `docs/research/`" | matches for verify and preflight; 32 notes | `scripts/verify.sh:74-80`; `docs/research/` |
| project-shape.md, intro | "What each directory and file of the tree holds ... `CLAUDE.md` keeps a line per directory. A change to the layout changes this file in the same PR" | drift (L7-R01): two files that landed in scope, `src/cook/chunk.rs` and `src/bench/history/night.rs`, have no entry while their siblings do | `docs/agents/project-shape.md:3,15-27,77-93` |
| project-shape.md, src/ | "lib.rs, state.rs"; "bin/praxis.rs"; "gpu/" | matches | `src/lib.rs`, `src/state.rs`, `src/bin/praxis.rs`, `src/gpu/mod.rs` |
| project-shape.md, capture/ | "how two images, or two shadow masks, differ" | matches | `src/capture/mod.rs:91-105`; `src/bin/praxis.rs:142` |
| project-shape.md, preflight/ | "a session's starting state, a line per check" | matches | `src/preflight/mod.rs:1-3` |
| project-shape.md, bench/ | "`praxis traverse`, the world's traversal; history/import.rs, `praxis history import` ...; history/suspects.rs ...; history/rerecord.rs, `praxis history rerecord`: ... re-record the counters of the Baselines they moved, timings kept, with the commits between" | matches | `src/bench/mod.rs:836`; `src/bench/history/rerecord.rs:1-6,23-24`; `src/bin/praxis.rs:367-392` |
| project-shape.md, stats/ | "the GPU memory ledger by kind, counting allocator" | matches | `src/stats/gpu_memory.rs:9,113`; `src/stats/mod.rs:492` |
| project-shape.md, streaming/ | "the Chunk streamer ... reaching IO through a narrow trait; io.rs, the IO pool (ADR 0016) ... ChunkStreaming ... a second Chunk streamer whose radius is the View distance; mips.rs, mip priority ...; chunks.rs, ... LoadedChunks ... ResidentFarProxies ... FarGround" | matches | `src/streaming/mod.rs:23,82`; `src/streaming/io.rs:1,662,710-714`; `src/streaming/chunks.rs:13,19,25` |
| project-shape.md, scenes/ | "the Spin system (the orrery, town's and world's flocks), the wrap that keeps world's flocks about its camera and the crowd's Sway system; town.rs ...; world.rs" | matches | `src/scenes/mod.rs:898-904,921,1331`; `src/scenes/{town,world}.rs` |
| project-shape.md, components.rs, threading.rs, feature modules | "(Transform, MeshRef)"; "(ADR 0016)"; "per domain in one mod.rs" | matches | `src/components.rs:11,44`; `src/threading.rs` |
| project-shape.md, assets/ | "asset types, pools, the Cooked cache reader (Asset IDs, Blobs), and the loader" | matches | `src/assets/{mod,pools,cooked,loader,material,mesh}.rs` |
| project-shape.md, cook/ | "the cooker behind the default-on `cook` feature ...; launch.rs ...; far_proxy.rs ..., far_proxy/bake.rs its atlas and the Material baked into it (ADR 0026); `praxis fetch` ...; `praxis export-scene`" | matches as far as it goes: `default = ["cook"]`; see L7-R01 for `chunk.rs` | `Cargo.toml:38`; `src/cook/launch.rs:1-2`; `src/cook/far_proxy/bake.rs:1-3` |
| project-shape.md, render/ | the passes (bend_pass, blas_refit, froxel_pass, culling_pass, main_pass, shadow_mask_pass, depth_pyramid_pass, tonemapping_pass, light_count_pass), targets, prepare, bind_groups, readback, acceleration_structures, instance, uniforms, staging, shaders/ | matches: each module exists; main_pass's Thin pipelines (`fs_thin_depth`, `fs_thin_main`) belong to the passes it names | `src/render/*.rs`; `src/render/shaders/main.wgsl:187,207` |
| project-shape.md, tests/goldens | "a `<golden>-lod-0.png` beside each that LOD selection changes; the root README.md shows the metal ones, and cargo test checks it lists each scene" (M3.5's L7-R01, fixed) | matches | `tests/goldens.rs:17,31,408-460`; `tests/goldens/metal/*-lod-0.png` |
| project-shape.md, bench/ | "budgets.json (ADR 0013) and baselines/<adapter>/<scene>.json: the M4 Pro's, CI's lavapipe (`--ci-counters`) and the RTX 2070 SUPER's, a `-dx12` directory ...; cycles/<scene>.json, each exported scene's Cycles Baseline, which scripts/gallery.sh with PRAXIS_BLENDER judges a run against" | matches | `bench/baselines/*`; `.github/workflows/ci.yml:293`; `scripts/gallery.sh:19-25,95-100` |
| project-shape.md, scripts | pr-lint.sh "title and body as the squash commit, size, commits and a rendering PR's gallery images ... tested by scripts/pr-lint-test.sh" | matches | `scripts/pr-lint.sh:1-5,108-135`; `.github/workflows/pr-lint.yml:35-36` |
| project-shape.md, scripts | gallery.sh; cycles-reference/ "calibrate.py ..., render.py ..., compare.py ...; DIFFERENCES.md" | matches (also `display-ramp.txt`, `differences/`) | `scripts/gallery.sh:1-27`; `git ls-files scripts/cycles-reference` |
| project-shape.md, scripts | sweep.sh "re-benches a slower Suspect ... bisects it (`praxis history bisect`), opens a needs-triage issue per pinned Regression (`praxis history regressions`), then benches main's scenes lacking a record on a Series here, on a quiet M4 Pro; beside it its launchd definition (sweep.plist)" | matches: the subcommands parse; `sweep-task.xml` is gone, as the text no longer names it | `scripts/sweep.sh:2-17`; `src/bin/praxis.rs:376-384`; `scripts/sweep.plist` |
| project-shape.md, scripts | verify.sh "fmt, clippy, build, tests with Goldens, bench counters against Baselines and Budgets"; lavapipe-files.sh "has CI record a pushed branch's lavapipe Goldens and Baselines, or takes a run's, and copies the changed ones into the tree" | matches | `scripts/verify.sh:74-78`; `scripts/lavapipe-files.sh:1-5` |
| project-shape.md, vendor/ | "until their release (README.md): wgpu-core 30.0.2 with the TLAS dependency dedup" | matches | `vendor/README.md:12-35` |
| machine.md, Claims | "A ticket's state is on the tracker ... A message or a comment names its session (`praxis-b4`)" | not checkable from the tree (process, GitHub) | — |
| machine.md, Worktrees | "One worktree per ticket, `../praxis-<ticket>`"; "After the merge the session removes its worktree and branch" | not checkable from the tree (process) | — |
| machine.md, Worktrees | "Each worktree has its own `target/` ...: a test binary holds its worktree's path" | matches: tests locate the tree through the compile-time `CARGO_MANIFEST_DIR` | `tests/goldens.rs:460`; `tests/field_guides.rs:328` |
| machine.md, Worktrees | "A worktree's `cooked/` is an APFS clone ..., not a link: a link shares one cache across Blob formats, and `git add -A` commits it" | matches: `.gitignore` ignores the directory form `/cooked/`, which a symlink does not match | `.gitignore:23` |
| machine.md, Quiet turns | "Sessions take turns by message ... `<ticket> quiet` ... `<ticket> done`"; "(`workflow.md`, PRs)" | not checkable from the tree (process); `workflow.md`, PRs carries the no-other-load rule for Baselines | `docs/agents/workflow.md:29` |
| machine.md, Timed runs | "`cargo run --release -- preflight` comes first; a `warn` line holds off a bench (`workflow.md`, Session start)" | matches | `docs/agents/workflow.md:71`; `src/preflight/mod.rs:388-395,438-442` |
| machine.md, Timed runs | "`pmset -g batt` reads `AC Power`"; "`mds_stores` nor `BDLDaemon`" | matches for `pmset` (preflight parses the same report); the process names are the host's, not checkable from the tree | `src/preflight/mod.rs:427-447` |
| machine.md, Timed runs | "An A/B interleaves its variants over at least three rounds" | not checkable from the tree (process) | — |
| machine.md, Timed runs | "A script never runs `praxis` with no arguments: that opens the window ... `praxis help` prints the usage; `praxis bench --help` benches every scene into a directory named `--help`" | matches: no arguments opens the window on `helmet`; `help` falls to the error that prints the usage; `bench <output>` with no scenes takes `ReferenceScene::ALL` | `src/bin/praxis.rs:295,359-361,415-434`; `src/bin/praxis.rs:564` |
| sessions.md, intro | "The agent invokes `tdd`, `diagnosing-bugs`, `research`, `prototype`, `domain-modeling`, `codebase-design`, and `code-review` itself" | matches: all seven installed | `~/.claude/skills/` |
| sessions.md, table | `/implement`, `/wayfinder`, `/grill-with-docs`, `/to-spec`, `/to-tickets`, `/handoff`, `/prototype`, `/triage`, `/improve-codebase-architecture`, `/ask-matt` | matches: all installed | `~/.claude/skills/` |
| sessions.md, table | "`/audit <lane>`, or "Audit lane <n> per docs/agents/audit.md" ... report PR at `docs/audit/<milestone>/<lane>.md`" | matches with a note: no `audit` skill installed; the prompt form is the alternative both files give | `docs/agents/audit.md:3` |
| sessions.md, table | "cited note under `docs/research/`" | matches | `docs/research/` |
| sessions.md, Effort | "low: `cargo run --release -- preflight`, `cargo run --release -- fetch`, `scripts/verify.sh`, bench runs ..."; "the Agent tool's `effort`" | matches: the commands exist; the tool parameter is the harness's | `src/bin/praxis.rs:297,305`; `scripts/verify.sh` |
| sessions.md, Rules of thumb | "(workflow.md, Review)", "(workflow.md, The ladder)", "(`docs/agents/machine.md`)", "(workflow.md, PRs)" | matches: the headings and file exist | `docs/agents/workflow.md:7,27,38`; `docs/agents/machine.md` |
| triage-labels.md | the five roles and `fog`, each label string equal to its role | matches: all six in `gh label list` | `docs/agents/triage-labels.md:7-12` |
| triage-labels.md | "Charting a milestone starts from `gh issue list --label fog --state open`" | not checkable from the tree | — |
| issue-tracker.md, Conventions | the `gh issue` commands | not checkable from the tree (CLI usage) | — |
| issue-tracker.md, Ticket form | "A ticket body has these sections, in this order (`docs/agents/workflow.md`, Tickets)" | matches `workflow.md`'s Start here, seam and ceiling clauses | `docs/agents/workflow.md:20-23` |
| issue-tracker.md, Pull requests | "`/triage` reads this flag" | matches: the skill is installed | `~/.claude/skills/triage` |
| issue-tracker.md, Wayfinding | "Used by `/wayfinder`"; labels `wayfinder:map`, `wayfinder:<type>` (`research`/`prototype`/`grilling`/`task`) | matches: the skill and all five labels exist | `~/.claude/skills/wayfinder`; `gh label list` |
| domain.md | "`CONTEXT.md` at the repo root, or `CONTEXT-MAP.md` ... if it exists"; "`docs/adr/`" | matches: no `CONTEXT-MAP.md` | `ls -a` of the root |
| domain.md | "The `/domain-modeling` skill (reached via `/grill-with-docs` and `/improve-codebase-architecture`)" | matches: all three installed | `~/.claude/skills/` |
| domain.md, File structure | "docs/adr/0001-material-as-id-not-bind-group.md" | matches | `docs/adr/0001-material-as-id-not-bind-group.md` |
| domain.md, Flag ADR conflicts | "Contradicts ADR 0007 (wgpu is the permanent graphics API)" | matches the ADR's file name | `docs/adr/0007-wgpu-is-the-permanent-graphics-api.md` |
| audit.md, intro | "Run it for each lane a Consolidation milestone's opening decision names (all eight at `M3.5`; lanes 1, 4 and 7 at `M5.5`) ... `/audit <lane>` where that skill is installed" | not checkable from the tree (GitHub); hedged on the skill, which is not installed | `~/.claude/skills/` |
| audit.md, intro | "A lane brief names the referents ... of the audit that last ran it: lanes 1, 4 and 7 `M5.5`'s, the others `M3.5`'s" | matches the briefs: lane 1 and 4 name `M5.5`, lane 7 M5's exit, lane 3 "27 of them at `M3.5`" | `docs/agents/audit.md:73,88,95,117` |
| audit.md, Inputs | "(for `M5.5`, since `8bc34eb18fcfb79307193551e96619316aef566c`; everything, at `M3.5`)"; `deviations.md`; `DIFFERENCES.md`; `docs/references.toml` | matches: the commit exists; the three files exist | `git log -1 8bc34eb1`; the paths |
| audit.md, The pipeline | "at high effort (`docs/agents/sessions.md`, Effort)"; "`docs/agents/workflow.md`, Performance and memory, says how a figure is quoted" | matches: the headings exist | `docs/agents/sessions.md:18`; `docs/agents/workflow.md:57` |
| audit.md, The pipeline | "Report. `docs/audit/<milestone>/<lane>.md` on `main`, by PR" | matches: `m3.5/` and `m5/` hold lane reports | `docs/audit/m3.5/`, `docs/audit/m5/` |
| audit.md, Deviations | "`docs/audit/deviations.md` holds, in one table ...: what deviates, the source key, why, and the trigger" | matches: columns Where, What deviates, Source, Why, Revisit when | `docs/audit/deviations.md:20` |
| audit.md, Lane 1 | "the stories of the specs ... (for `M5.5`, #611 and #804)"; "every function under an oracle" | not checkable from the tree (GitHub); the oracle file exists | `src/render/shader_function_tests.rs` |
| audit.md, Lane 2 | "`scripts/gallery.sh` with `PRAXIS_BLENDER` at 2560×1440, and the 3x box-filtered supersampled reference" | matches | `scripts/gallery.sh:39` |
| audit.md, Lane 2 | "against Cycles (recorded as that scene's first Cycles Baseline; the Baseline mechanism itself is a fix ticket the spec charts)" | drift (L7-R03): the mechanism landed in scope; eleven Cycles Baselines exist and the gallery judges against them | `bench/cycles/*.json`; `scripts/gallery.sh:19-25,95-100` |
| audit.md, Lane 2 | "Each `DIFFERENCES.md` item is tagged in that file `deliberate until <trigger>` or `ticketed #n`" | matches: nine items, nine tags | `scripts/cycles-reference/DIFFERENCES.md:24,40,74,102,122,139,163,176,195` |
| audit.md, Lane 3 | "the ladder's map issue (#17; 27 of them at `M3.5`) ... the archive comment its body links"; "`pmset -g batt`" | not checkable from the tree (GitHub; a host command preflight also reads) | `src/preflight/mod.rs:427-429` |
| audit.md, Lane 4 | "(for `M5.5`, M6's: probe GI, Interior volumes and reflection cubemaps ...)"; "on the M4 Pro and the RTX 2070 SUPER from one engine source (ADR 0013)" | not checkable from the tree (roadmap) | — |
| audit.md, Lane 5 | "clippy's `cognitive_complexity` lint run as a probe"; "Fix tickets carry the refactor licence (`CLAUDE.md`, How agents work here)" | matches the CLAUDE.md clause; the lint is clippy's | `CLAUDE.md:165-168` |
| audit.md, Lane 6 | "Golden tolerances kept at their values (1 of 255 and 0 pixels)"; "`src/render/shader_function_tests.rs`" | matches: `GOLDEN_TOLERANCE: u8 = 1`, `HARDWARE_UNCULLED_PIXELS = 0` (software 1, its own named constant) | `src/capture/mod.rs:35`; `tests/goldens.rs:71,79` |
| audit.md, Lane 7 | "the ADRs, `CONTEXT.md`, `CLAUDE.md`, `docs/agents/*` and `docs/references.toml` ... as M5's did for `M5.5` (`docs/audit/m5/sources.md`)" | matches: the paths exist | `docs/audit/m5/sources.md` |
| audit.md, Lane 8 | "per volume, the mechanism it describes, the page in the guide's shape (`docs/agents/workflow.md`, PRs) that absorbs it" | matches the heading cited; see Notes: no volume is left | `docs/agents/workflow.md:32`; `tests/field_guides.rs:17` |

Notes:

- `ci.yml` and `lavapipe-files.yml` pin `dtolnay/rust-toolchain` to a commit with the comment `# stable`, the branch it was taken from rather than a version. The checklist item reads "with its version in a comment"; the action has no version tags, so this is the closest form and is not filed.
- Two shipping GPU objects besides the load staging's belt are made outside the ledger: the capture's offscreen target and read-back buffer (`src/capture/mod.rs:332,465`). The target stands where the surface would be, which the ledger does not count either; they are named in L7-R02's evidence, not filed apart.
- `tests/conventions.rs:16` holds seven rules; the seventh, "measured timing or ratio in a shipping comment" (`:42`), has no line in `CLAUDE.md`'s Code conventions. The file does not claim to list every tested rule, so this is no Finding.
- `audit.md`'s Lane 8 inventories volumes; `tests/field_guides.rs:17` lists none (`VOLUMES: &[&str] = &[]`), so the next run of that lane has nothing to inventory. `audit.md:5` keeps a brief's referents until the audit that runs it next replaces them, and the guide rewrite is docs, not code, so no Finding is filed.
- `workflow.md`, Review: the installed `code-review` skill runs two axes; the Sources sub-agent is the session's own, as at M3.5. Not checkable from the tree.
- `src/threading.rs:6` imports `SingleThreadedExecutor`; the three leads on ADR 0016 and `multi_threaded` are reader A's.
- At the frozen commit `CLAUDE.md:289` names the `StagingBelt` for streaming uploads, which holds (`LoadStaging` wraps one); `main` has since reworded the item.

## Findings

The 15 Findings the verifier confirmed or narrowed, in id order; the
`placement` column is the triage's. A narrowed Finding keeps the reader's
claim in its `claim` cell and carries the rewritten claim in its `verifier`
cell. The evidence blocks follow the table; the killed Finding's block
follows them, kept so that the next audit does not re-find it.

| id | claim | severity | where | evidence | expected | verifier | placement |
|---|---|---|---|---|---|---|---|
| L7-A01 | ADR 0003 sizes an Instance slot at 136.2 bytes and the Budgets at 517,376 slots; since the culling pass's third sum a slot's offsets are 16 bytes, a slot 144.2 bytes, and the checked-in Budgets give 488,448 slots. | drift; not load-bearing (the 30% share and the scan-block rule hold); misleads a reader sizing the Instance buffer | `docs/adr/0003-persistent-scene-transform-buffer.md:3` | block below | "at 144.2 bytes a slot ... 488,448 slots at the checked-in Budgets", or the figures dated | `confirmed`: `docs/adr/0003-persistent-scene-transform-buffer.md:3`: "30% of it at 136.2 bytes a slot, its Instance, offsets and visibility bit, in whole 256-slot scan blocks of the culling pass: 517,376 slots at the Budgets M4 set". `src/render/renderer.rs:213-218` sizes a block as `SCAN_BLOCK_SIZE * (size_of::<InstanceData>() + size_of::<[u32; 4]>()) + size_of::<[u32; 4]>() + SCAN_BLOCK_SIZE / 8`: with `InstanceData` 128 bytes (`src/render/instance.rs:33-40`) and `SCAN_BLOCK_SIZE = 256` (`src/render/culling_pass.rs:67`), 36,912 bytes, 144.19 a slot; `:1092` pins `instances: 488_448`. The offsets grew in `66390f70` (2026-11-16, a descendant of `8bc34eb1`). Not a dated clause: "(amended 2026-11-09)" tags the text, and M5's verifier itself left the sentence as a lead (`docs/audit/m5/sources.md:1000-1002`). Not load-bearing. |  |
| L7-A02 | ADR 0004 says prepare fills Frame data from the ECS World "on the main thread"; since prepare's Instance fill moved to the compute pool, compute pool tasks read the World's Renderables and write their Instances, the main thread joining them. | drift; not load-bearing (the decision, no World read by the renderer, holds); misleads a reader about which threads read the World | `docs/adr/0004-frame-data-snapshot.md:3` | block below | "Today prepare fills Frame data from the ECS World, driven from the main thread, its Instance fill split across the compute pool's tasks (ADR 0016)" | `confirmed`: `docs/adr/0004-frame-data-snapshot.md:3`: "Today prepare (`src/render/prepare.rs`) fills Frame data from the ECS World on the main thread"; `src/render/prepare.rs:208`, inside the fill closure, queries the World (`renderables.get(entity)`), and `:250-265` (`run_on_pool_and_main_thread`) runs that closure in `compute_pool.scope` tasks beside the main thread. `8bc34eb1`'s `prepare.rs` has no `compute_pool`, and M3.5's row (`docs/audit/m3.5/docs-drift.md:94`) matched the clause then. Not load-bearing. |  |
| L7-A03 | ADR 0016 says `par_iter` "runs on the process-global pool, which the engine no longer creates; the first system to use it decides where prepare's tasks then run"; at bevy_ecs 0.20.1 `par_iter` calls `ComputeTaskPool::get()`, which panics on a pool nothing created, nothing in the engine creates it under the `SingleThreadedExecutor`, and prepare's tasks run on the `FramePreparer`'s own pool whatever a system does. | drift; not load-bearing (no `par_iter` in `src/`), but it misleads the author of the first parallel system into a panic and a reader into thinking prepare's pool follows it | `docs/adr/0016-the-main-thread-runs-the-frame.md:16` | block below | "`par_iter` reads the process-global `ComputeTaskPool` and panics while nothing has created it, which the engine no longer does; the first system to use it creates that pool or moves to the State's own, and decides then whether prepare's tasks share it" | `narrowed`: `docs/adr/0016-the-main-thread-runs-the-frame.md:16` says `par_iter` "runs on the process-global pool ..., which the engine no longer creates", but at bevy_ecs 0.20.1 with `multi_threaded` on a `par_iter` panics while nothing has created that pool, and nothing in `src/` does. `~/.cargo/registry/.../bevy_ecs-0.20.1/src/query/par_iter.rs:103-105` calls `bevy_tasks::ComputeTaskPool::get()`, documented "# Panics If the [`ComputeTaskPool`] is not initialized" (`:36-39`); `bevy_tasks-0.20.1/src/usages.rs:29-38` `get()` `.expect(...)`; `Cargo.toml:17` enables `multi_threaded`; `git grep "ComputeTaskPool\|par_iter\|get_or_init" -- src` finds nothing, and `src/threading.rs:24-27` sets the `SingleThreadedExecutor`, so the executor's `get_or_init` never runs. The reader's second half, "prepare's tasks run on the `FramePreparer`'s own pool whatever a system does", does not refute the clause: "the first system to use it decides where prepare's tasks then run" reads as a deferred decision, as M5's source check read it (`docs/audit/m5/sources.md:188`, "reads as a deferred decision, not a fallback"), and the same bullet's first sentence already says prepare's pool is its own. |  |
| L7-A04 | ADR 0016's status line lists what is built (the feature, the compute pool, the executor, prepare's fill, the IO pool) and says "the rest is planned", but its Consequence that workers record `CommandEncoder`s and return `CommandBuffer`s the main thread submits is built: a static TLAS worker thread does exactly that. | drift; not load-bearing; misleads a reader of the status line about which threads exist and record GPU work | `docs/adr/0016-the-main-thread-runs-the-frame.md:3` | block below | The status line adds "a worker thread records the static TLAS's build, which the main thread submits (ADR 0020)" before "the rest is planned" | `confirmed`: `docs/adr/0016-the-main-thread-runs-the-frame.md:3`: "...; the IO pool is built (below); the rest is planned", the rest including `:18`, "Workers record `CommandEncoder`s and return `CommandBuffer`s". `src/render/acceleration_structures.rs:592-614` spawns `"Static TLAS Worker"`, which calls `device.create_command_encoder` and returns `encoder.finish()`, and `:546` submits it on the main thread (`queue.submit(std::iter::once(build))`). Landed `78727a00` (2026-11-08), in scope. ADR 0020:20 states the worker, but ADR 0016's status line, which enumerates the threads built (the IO pool among them), does not. Not load-bearing. |  |
| L7-A05 | ADR 0017 gives an Instance slot's cost as 136.2 bytes with its offsets and visibility bit; a slot's offsets are 16 bytes since the culling pass's third sum, and a slot costs 144.2 bytes. | drift; not load-bearing (the 30% share holds); misleads a reader sizing the scene and pass buffers (the same slip as L7-A01, in a second ADR) | `docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:18` | block below | "the Instance slots 30% of the scene and pass buffers', at 144.2 bytes a slot with its three offsets and visibility bit" | `confirmed`: `docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:18`: "the Instance slots 30% of the scene and pass buffers', at 136.2 bytes a slot with its offsets and visibility bit"; the same arithmetic as L7-A01 from `src/render/renderer.rs:213-218` gives 144.19, the offsets having grown in `66390f70`. One defect at a second location, named by M5's verifier beside ADR 0003's (`docs/audit/m5/sources.md:1000-1002`). Not load-bearing. |  |
| L7-A06 | ADR 0019's reopen clause says "The 6 ms is provisional, as ADR 0013's budgets are", but ADR 0013 made the Budgets no longer provisional at M3's exit and the gate is one of them, judged and failed by `praxis compare` like the others. | drift; not load-bearing (the figure is the same either way); misleads a reader weighing whether the gate can move without an ADR | `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:21` | block below | "The 6 ms is a Budget like ADR 0013's, which M3's exit made final" or the sentence cut | `confirmed`: `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:21` (Consequences): "The 6 ms is provisional, as ADR 0013's budgets are."; `docs/adr/0013-reference-hardware-and-provisional-budgets.md:3`: "At M3's exit the Budgets were re-decided ...: each stays as written and is no longer provisional", the gate among those judged ("the CPU frame, the GPU frame and ADR 0019's gate answer to their Budgets"); `bench/budgets.json` holds `gate_microseconds` and `src/bench/mod.rs:1331-1334` fails a run past it. The sentence stands at `8bc34eb1` and M3.5 gave it no verdict: a miss of M3.5's lane. Not load-bearing. |  |
| L7-A08 | ADR 0020 says its probe convergence and 2 ms update targets "are provisional, as ADR 0013's budgets are", but ADR 0013 made its Budgets no longer provisional at M3's exit. | drift; not load-bearing (the targets are unbuilt; their own provisional status may stand); misleads a reader about ADR 0013's Budgets (the same slip as L7-A06) | `docs/adr/0020-ray-traced-probe-gi-and-hard-ray-traced-shadows.md:17` | block below | "Both are provisional until probes are built and measured" (no comparison with ADR 0013's Budgets) | `confirmed`: `docs/adr/0020-ray-traced-probe-gi-and-hard-ray-traced-shadows.md:17` (Consequences): "Both are provisional, as ADR 0013's budgets are."; ADR 0013:3: "each stays as written and is no longer provisional". The comparison asserts a present fact about ADR 0013's Budgets that has been false since M3's exit; the sentence stands at `8bc34eb1` and M3.5's row (`docs/audit/m3.5/docs-drift.md:357`) judged the bullet `planned` without it. Not load-bearing; the probe targets' own provisional status may stand. |  |
| L7-A09 | ADR 0025 says "local scheduled tasks bench the real ones", the GPUs CI lacks; since the RTX 2070 SUPER's sweep stopped, one scheduled task benches one GPU, the M4 Pro's, and the RTX 2070 SUPER is benched by hand at each exit, as ADR 0024 now says. | drift; not load-bearing (the clause's point, no self-hosted runner, holds); misleads a reader about which machines bench unattended | `docs/adr/0025-the-repo-is-public-under-mit.md:17` | block below | "and a local scheduled task benches the reference machine, the RTX 2070 SUPER being benched by hand (ADR 0024)" | `confirmed`: `docs/adr/0025-the-repo-is-public-under-mit.md:17`: "CI runs on GitHub's hosted runners, without a GPU, and local scheduled tasks bench the real ones."; `ls scripts` holds `sweep.plist` and `sweep.sh` only, `scripts/sweep.sh:16-17` "It runs on the M4 Pro", and ADR 0024:3: "The RTX 2070 SUPER has no sweep ... its Goldens and Baselines are recorded by hand at each milestone's exit", both from `372ce4a0` (2026-11-21), after the clause (`300116fb`, 2026-11-14). The plural names two machines; one scheduled task on one GPU remains. Not load-bearing. |  |
| L7-G01 | The glossary's Material ID says an Instance carries one only when its Renderable overrides its Mesh's Materials, but a Placement's Instance carries one too, when the Placement names an overriding Material. | drift; not load-bearing (the Instance and Placement terms state the override); misleads a reader of the term about Chunk Instances | `CONTEXT.md:83` | block below | "an Instance carries one only when its Renderable or Placement overrides its Mesh's Materials" | `confirmed`: `CONTEXT.md:83`: "an Instance carries one only when its Renderable overrides its Mesh's Materials"; `src/render/prepare.rs:953-963` passes `placement.material` to `instance()`, which writes `material_id: material.map_or(NO_MATERIAL, MaterialHandle::index)` (`:162`). `src/streaming/chunks.rs` is absent at `8bc34eb1`, and the sentence was last touched 2026-10-19 (`28294f45`), so code landed since made it false. Not load-bearing: Instance (`CONTEXT.md:115`) and Placement (`:109`) state the override. |  |
| L7-G02 | The glossary's Frame stats lists what one frame measures, but leaves out every streaming field M4 and M5 added to `FrameStats`: Late Chunks, Late Far proxies, late mips, Chunks loaded, unloaded and not loaded, bytes read, pool entries freed, Meshes and Materials refused, and the Fast travel's hold and Far field fill. | drift; not load-bearing (the Late terms say they are counted); misleads a reader who takes the list as what Frame stats hold, and one looking for where a Late Chunk is counted | `CONTEXT.md:247` | block below | The list names the streaming fields, or closes with "and the streaming counters (Late Chunks, Late Far proxies, late mips, loads, unloads, refusals, bytes read, Fast travel's hold and fill)" | `confirmed`: `CONTEXT.md:247` lists Frame stats down to "dropped time and the readbacks of GPU stats it skipped", counters included ("Renderables and Placements refused"), yet `src/stats/mod.rs:89-123` holds `late_chunks`, `late_far_proxies`, `chunks_loaded`, `chunks_unloaded`, `bytes_read`, `pool_entries_freed`, `meshes_refused`, `materials_refused`, `chunks_not_loaded`, `late_mips`, `holding`, `fast_travel_hold`, `far_field_filling`, `far_field_fill`, none named. All landed after `8bc34eb1`; M3.5's row (`docs/audit/m3.5/docs-drift.md:562`) matched the shorter struct. Not load-bearing. |  |
| L7-G03 | The glossary's Baseline says a commit that changes only a Baseline's counters does not re-record it. But `praxis history rerecord`, which landed after that sentence, writes exactly such commits and calls each one a re-recorded Baseline. | drift; not load-bearing (both say such a commit starts no line on its Series); misleads a reader of Suspect ("imported or re-recorded Baseline") and Regression ("no re-recorded Baseline covers"), who meets the command's other sense in the code and in its reports | `CONTEXT.md:255` | block below | The term names both: a Baseline is re-measured when a commit changes a timing it held or its Harness version, and only that starts a line or covers a Regression; a commit that re-records its counters alone, as `praxis history rerecord` does, does neither. Or the command's name and doc stop calling it a re-record | `confirmed`: `CONTEXT.md:255`: "a commit that changes only its counters does not [re-record one]"; `src/bench/history/rerecord.rs:1-4` (landed `605f40b7`, 2026-11-20): "runs of `main` re-record the counters of this machine's Baselines where they moved. A re-recorded Baseline keeps its timings", and `:33` names the outcome `CounterRerecord::Rerecorded`. The glossary's sense is `remeasured` (`src/bench/history/import.rs:278-279`, "Counters alone changing ... measured nothing again"), which `src/bench/history/verdicts.rs:730` still calls "no re-recorded Baseline covers", so the code now uses the word in both senses and the glossary excludes one. Not load-bearing: both say such a commit starts no line. |  |
| L7-G04 | The glossary's Suspect judges a record against "up to 8 records before it", but a record with fewer than 3 records before it on its line is never judged. | drift; not load-bearing (it only affects the first records after a line starts); misleads a reader of the bench page or the Series who expects the second record after a line start to be flagged. Missed by M3.5's lane 7 (the minimum predates `8bc34eb1`) | `CONTEXT.md:287` | block below | "the median of the 3 to 8 records before it on its line" | `confirmed`: `CONTEXT.md:287`: "outside the median of up to 8 records before it"; `src/bench/history/suspects.rs:9-11`: "with fewer, as at the start of a line, it is not judged. `const FEWEST_JUDGING_RECORDS: usize = 3;`", and `band` returns `None` below it (`:91-93`). The same constant and the same "up to 8" text stand at `8bc34eb1` (`git show 8bc34eb1:src/bench/history/suspects.rs`, line 11; `git show 8bc34eb1:CONTEXT.md`, line 212), and M3.5's Suspect row (`docs/audit/m3.5/docs-drift.md:569`) and its L7-G02 omit it: a miss of M3.5's lane, so in scope. Not load-bearing. |  |
| L7-R01 | `docs/agents/project-shape.md` says it holds what each directory and file of the tree holds, and `CLAUDE.md` points to it "file by file", but `src/cook/chunk.rs` (the Chunk cook) and `src/bench/history/night.rs` (a Night's directory), both landed in scope before the file was written, have no entry while their siblings do. | drift; not load-bearing; a session looking for where Chunks are cooked or where a Night's directory is kept is misled into the sibling files | `docs/agents/project-shape.md:3` | block below | The `cook/` entry names `chunk.rs` beside `launch.rs` and `far_proxy.rs`, and the `bench/` entry names `history/night.rs` beside `import.rs`, `suspects.rs` and `rerecord.rs`; or the intro says it names the files a reader would not find from the directory's line. | `narrowed`: `docs/agents/project-shape.md:3` says it holds "What each directory and file of the tree holds", but `src/bench/history/night.rs` (a Night's directory, added in scope, `7a4429bc`) has no line, by name or by what it holds, in the `bench/` entry (`:15-27`), which names its siblings `history/import.rs`, `history/suspects.rs` and `history/rerecord.rs`. Of the source files added since `8bc34eb1` (`git diff --name-status --diff-filter=A 8bc34eb1 HEAD -- src`), the shape names every one by name or by what it holds except `night.rs`. `src/cook/chunk.rs` ("a Reference scene's static content cut into Chunks", `:1-2`) is covered by what it holds: the `cook/` entry says "`praxis cook` turns manifest sources and Reference scenes into Blobs" (`:78-80`), as `stats/gpu_memory.rs` is by "the GPU memory ledger by kind" (`:29`). Nothing in the shape names a Night or its directory (`grep -n night` finds only "the nightly sweep"). Not load-bearing. |  |
| L7-R02 | The Review checklist says GPU buffers are created through the GPU memory ledger with only the GPU stats readback's buffers kept out, but the load staging's `StagingBelt` buffers are kept out of `gpu_memory_bytes` by design and the checklist does not name them. | drift; not load-bearing; a reviewer applying the item to a change in load staging would ask for ledger accounting the design refuses | `CLAUDE.md:281-288` | block below | The item names the load staging's belt beside the readback as kept out (counted in `uploaded_bytes` instead), or says which transient buffers stay out. | `confirmed`: `CLAUDE.md:281-286`: "GPU buffers ... are created through the GPU memory ledger (`GpuMemory`) ...; the GPU stats readback's buffers stay out"; `src/render/staging.rs:179-184`: the load staging's belt "staging buffers live only until the GPU has run the copies and never count in `gpu_memory_bytes`", the belt made by `wgpu::util::StagingBelt::new(device.clone(), LOAD_CHUNK_BYTES)` (`:348`). ADR 0017 names the same exception ("the GPU stats readback and the load staging, bypasses it"), so only the checklist omits it. The belt's exception is at `8bc34eb1` too (`staging.rs:170`), and M3.5's wording ("GPU buffers and textures join their owner's `gpu_memory_bytes`") was already false for it: the item rewritten in scope (`f99166b5`) and M3.5's row both missed it. Not load-bearing. |  |
| L7-R03 | `audit.md`'s Lane 2 says a scene's Cycles distance is "recorded as that scene's first Cycles Baseline; the Baseline mechanism itself is a fix ticket the spec charts", but the mechanism landed in scope: eleven Cycles Baselines are committed and the gallery judges each run against them. | drift; not load-bearing until Lane 2 next runs; its readers are misled into recording first Baselines over existing ones | `docs/agents/audit.md:81` | block below | Lane 2 judges each scene's distance against its Cycles Baseline in `bench/cycles/` (`scripts/gallery.sh` with `PRAXIS_BLENDER`), recording one only for a scene that has none. | `narrowed`: `docs/agents/audit.md:81`, Lane 2's Method, has readers record each scene's Cycles distance "as that scene's first Cycles Baseline", but every exported scene has one (`bench/cycles/`, eleven files, landed `88868582`) and `scripts/gallery.sh:19-25` judges a run against it, recording only under `PRAXIS_RECORD_CYCLES_BASELINES`. The quote is at `:81`, not `:82`. The parenthesis's second half, "the Baseline mechanism itself is a fix ticket the spec charts", is a referent of the audit that last ran Lane 2, which `audit.md:5` keeps by design ("A lane brief names the referents (spec issues, fog lines, the next milestone's scope) of the audit that last ran it: ... the others `M3.5`'s. A later audit replaces those referents with its own in the PR that opens it"); only the method clause is drift. `bench/cycles/` is absent at `8bc34eb1`, so code landed since made it false. Not load-bearing until Lane 2 next runs. |  |

### L7-A01

`docs/adr/0003-persistent-scene-transform-buffer.md:3`, "Persistent Instance buffer": "30% of it at 136.2 bytes a slot, its Instance, offsets and visibility bit, in whole 256-slot scan blocks of the culling pass: 517,376 slots at the Budgets M4 set".

`src/render/renderer.rs:213-218`:

```
const INSTANCE_SCAN_BLOCK_BYTES: u64 = SCAN_BLOCK_SIZE as u64
    * (size_of::<InstanceData>() + size_of::<[u32; 4]>()) as u64
    + size_of::<[u32; 4]>() as u64
    + SCAN_BLOCK_SIZE as u64 / u8::BITS as u64;
```

`InstanceData` is 128 bytes (`src/render/instance.rs:33-40`: two `[[f32; 4]; 3]`, `[i32; 3]`, `u32`, `[f32; 3]`, `u32`), so a block is 256 × 144 + 16 + 32 = 36,912 bytes, 144.19 a slot. `src/render/renderer.rs:1092`, the test `the_checked_in_budgets_give_these_render_capacities`: `instances: 488_448,`. The offsets went from 8 to 16 bytes with the culling pass's third sum in the commit that drew Thin Primitives (2026-11-16), whose body records "28,928 fewer Instance slots"; the ADR was not touched for it. M5's source check saw this and left it to a full lane 7 ("Left for a full lane 7").

Fix to the text: "30% of it at 144.2 bytes a slot, its Instance, three offsets and visibility bit, in whole 256-slot scan blocks of the culling pass: 488,448 slots at the checked-in Budgets".

### L7-A02

`docs/adr/0004-frame-data-snapshot.md:3`, "Frame data snapshot": "Today prepare (`src/render/prepare.rs`) fills Frame data from the ECS World on the main thread, and the `Renderer` [...] draws it without reading the ECS World."

`src/render/prepare.rs:168-170`: "The fill spawns a compute pool task per this many drawn Instances, and fills fewer than two tasks' worth on the main thread. `const INSTANCES_PER_TASK: usize = 1024;`". `:208`, inside the fill each task runs: `let Ok((_, current, previous, mesh_ref, material_ref)) = renderables.get(entity) else {` (a World query); `:254-262` `compute_pool.scope(|scope| { for _ in 0..task_count { scope.spawn(async move { work() }); } ... work(); });`. ADR 0016's own status line says so ("prepare fills its Instances on a `TaskPool` that each State's `FramePreparer` owns").

Fix to the text: "Today prepare (`src/render/prepare.rs`) fills Frame data from the ECS World, on the main thread and, for its Instances, on the compute pool's tasks the main thread joins (ADR 0016), and the `Renderer` [...]".

### L7-A03

`docs/adr/0016-the-main-thread-runs-the-frame.md:16`, Consequences: "`par_iter` runs on the process-global pool [ref:bevy-ecs-0.20.1-par-iter-multi-threaded], which the engine no longer creates; the first system to use it decides where prepare's tasks then run".

bevy_ecs 0.20.1, the version `Cargo.lock:190-191` pins, `src/query/par_iter.rs:103-105`: `#[cfg(all(not(target_arch = "wasm32"), feature = "multi_threaded"))] { let thread_count = bevy_tasks::ComputeTaskPool::get().thread_num();`, its doc saying "# Panics If the [`ComputeTaskPool`] is not initialized" (`:36-39`). bevy_tasks 0.20.1 `src/usages.rs:29-38`: `pub fn get() -> &'static Self { $static.get().expect(concat!("The ", stringify!($type), " has not been initialized yet. [...]"))`. The one creator in bevy_ecs is the multi-threaded executor (`src/schedule/executor/multi_threaded.rs:274`, `ComputeTaskPool::get_or_init(TaskPool::default)`), which the engine replaces: `src/threading.rs:24-28` `schedule.set_executor(SingleThreadedExecutor::new());`, used by `src/state.rs:392` and `src/time/mod.rs:70`; `grep -rn "ComputeTaskPool\|par_iter" src` finds nothing. Prepare's tasks run on its own pool, `src/render/prepare.rs:489` `compute_pool: bevy_tasks::TaskPool` and `:520` `compute_pool: compute_pool(),`, which no system's use of `par_iter` reaches. M5's source check confirmed the clause with a note that `get()` panics and listed it under "Left for a full lane 7"; the clause entered with the per-State pool (2026-11-13) and was re-cited at the bevy 0.20.1 bump, both after `8bc34eb1`. This is the brief's lead 2, and it holds.

Fix to the text: "`par_iter` reads the process-global `ComputeTaskPool` and panics while nothing has created it, which the engine no longer does; the first system to use it creates that pool, or takes the State's own, and decides then whether prepare's tasks share it".

### L7-A04

`docs/adr/0016-the-main-thread-runs-the-frame.md:3`, status: "prepare fills the Renderables' Instances on the pool once there are enough to split among its threads, and no system uses it yet [...]; the IO pool is built (below); the rest is planned." The rest includes `:18`, "Workers record `CommandEncoder`s and return `CommandBuffer`s; they push no error scopes".

`src/render/acceleration_structures.rs:204-206`: "Two TLASes hold it: rays trace the bound one while a worker thread brings the spare up to date and records its build, which the main thread submits (ADR 0016)"; `:592-614`: `std::thread::Builder::new().name("Static TLAS Worker".to_string()).spawn(move || { [...] let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Static TLAS Encoder"), }); [...] encoder.finish() });`, and `:546` `queue.submit(std::iter::once(build));` on the main thread. ADR 0020:20 states the worker ("a worker thread takes the static Instance changes the spare lacks, records and finishes its build, which the main thread submits (ADR 0016)"); ADR 0016's status line, last amended for the IO pool (2026-11-08, the same day the worker landed), does not.

Fix to the text: "[...]; the IO pool is built (below), and a worker thread records the static TLAS's build, which the main thread submits (ADR 0020); the rest is planned."

### L7-A05

`docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:18`, Consequences: "the Instance slots 30% of the scene and pass buffers', at 136.2 bytes a slot with its offsets and visibility bit, in whole scan blocks".

`src/render/renderer.rs:213-218` (quoted under L7-A01): a scan block is `SCAN_BLOCK_SIZE × (size_of::<InstanceData>() + size_of::<[u32; 4]>()) + size_of::<[u32; 4]>() + SCAN_BLOCK_SIZE / 8`, 36,912 bytes for 256 slots, 144.19 a slot. The 136.2 is the figure from before the offsets grew from two to four `u32` for the Thin Primitives' third sum (2026-11-16). M5's verifier named this sentence beside ADR 0003's under "Left for a full lane 7".

Fix to the text: "the Instance slots 30% of the scene and pass buffers', at 144.2 bytes a slot with its offsets and visibility bit, in whole scan blocks".

### L7-A06

`docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:21`, Consequences, the reopen clause: "Shadow-ray time is reported beside the gate as its own line [...]. The 6 ms is provisional, as ADR 0013's budgets are."

`docs/adr/0013-reference-hardware-and-provisional-budgets.md:3`: "At M3's exit the Budgets were re-decided against that first Baseline [...]: each stays as written and is no longer provisional", and the same ADR counts the gate among them: "the CPU frame, the GPU frame and ADR 0019's gate answer to their Budgets at the p99 as at the median". The code judges it so: `bench/budgets.json:4` `"gate_microseconds": 6000,`; `src/bench/mod.rs:1331-1334` `let (metric, budget) = ("gate_microseconds", budgets.gate_microseconds); check_budget(&mut findings, metric, gate, budget); [...] check_tail_budget(&mut findings, metric, "p99", gate_p99, budget);`. The sentence was true when written (2026-10-19) and false from M3's exit (2026-10-30); it stood unchanged at `8bc34eb1` (`git show 8bc34eb1:docs/adr/0019-*.md`), and M3.5's lane 7 gave the reopen clause a verdict without this sentence, so it missed it.

Fix to the text: "The 6 ms is a Budget, final since M3's exit as ADR 0013's are."

### L7-A08

`docs/adr/0020-ray-traced-probe-gi-and-hard-ray-traced-shadows.md:17`, Consequences: "the probe update takes at most 2 ms of GPU on the dense Reference scene, judged on the M4 Pro [...]. Both are provisional, as ADR 0013's budgets are."

`docs/adr/0013-reference-hardware-and-provisional-budgets.md:3`: "At M3's exit the Budgets were re-decided against that first Baseline [...]: each stays as written and is no longer provisional". The Budgets the code judges are final figures with no provisional mark: `bench/budgets.json:2-8` and `src/bench/mod.rs:1270-1361`, which fails a run past any of them. The sentence stood at `8bc34eb1` (`git show 8bc34eb1:docs/adr/0020-*.md`) and M3.5's lane 7 did not give it a verdict; the 2026-11-14 amendment rewrote the same bullet's adapter clause and left it.

Fix to the text: "Both are provisional until the probes are built and measured."

### L7-A09

`docs/adr/0025-the-repo-is-public-under-mit.md:17`, Consequences: "Self-hosted Actions runners on the owner's machines, which a pull request from anyone could run code on (ADR 0024). CI runs on GitHub's hosted runners, without a GPU, and local scheduled tasks bench the real ones."

The commit that stopped the RTX 2070 SUPER's sweep (2026-11-21) deleted `scripts/sweep-task.xml`, the Windows scheduled task, and cut `scripts/sweep.sh` to the M4 Pro; what remains is `scripts/sweep.plist:3`: "<!-- The nightly sweep on the M4 Pro, installed as scripts/sweep.sh says. -->". That commit rewrote ADR 0024 (`:3`: "The RTX 2070 SUPER has no sweep: it was rarely on, so its sweep published on few days, and its Goldens and Baselines are recorded by hand at each milestone's exit") and left ADR 0025's clause, written a week before while both tasks ran.

Fix to the text: "CI runs on GitHub's hosted runners, without a GPU, and a local scheduled task benches the reference machine's; the RTX 2070 SUPER is benched by hand (ADR 0024)."

### L7-G01

`CONTEXT.md:82-83`, Language, Resources, **Material ID**: "A Primitive's Material has one; an Instance carries one only when its Renderable overrides its Mesh's Materials."

The code:
- `src/assets/cooked.rs:855-856`, `CookedPlacement`: "/// The Material that overrides every Primitive's, if any." / `pub material: Option<AssetId>,`.
- `src/streaming/chunks.rs:54-59`, `ChunkPlacement`: "A Placement as prepare draws it: its World transform, its Mesh, and the Material that overrides its Mesh's, if any".
- `src/render/prepare.rs:953-963` (`write_chunk`) passes `placement.material` to `instance()`, and `instance()` writes `material_id: material.map_or(NO_MATERIAL, MaterialHandle::index)` (`src/render/prepare.rs:162`).

The glossary's own **Instance** (`CONTEXT.md:115`) and **Placement** (`CONTEXT.md:109`) state the override. `git log -L82,84:CONTEXT.md` last touches the sentence on 2026-10-19, before Placements landed (`95eef204`, `3f7e098a`, both after `8bc34eb1`).

Fix to the text: "an Instance carries one only when its Renderable or Placement overrides its Mesh's Materials."

### L7-G02

`CONTEXT.md:246-247`, Language, Verification, **Frame stats**: "What one frame cost, measured by the engine itself: CPU time per system and stage, GPU time per pass, [...] bends the bend pass had no room for, bytes uploaded, heap allocations and of them the dynamic TLAS build's, GPU memory, Arena bytes in use, dropped time and the readbacks of GPU stats it skipped."

The code, `src/stats/mod.rs:89-123`, holds these `FrameStats` fields, none of which the list names:
- `late_chunks` ("Chunks the camera came within the Load radius of before they loaded")
- `late_far_proxies`
- `chunks_loaded`, `chunks_unloaded`, `chunks_not_loaded`
- `bytes_read`
- `pool_entries_freed`
- `meshes_refused`, `materials_refused`
- `late_mips` ("Late mips: mips the frame's drawn Chunks or Renderables need [...] that are not Resident")
- `holding`, `fast_travel_hold`, `far_field_filling`, `far_field_fill`

`GpuFrameStats` adds `phase_two_triangles_drawn` (`:156-158`). Each landed after `8bc34eb1`: `e9019be8` (late Chunks, bytes read), `92643f28` (late mips), `cfa49567` (Fast travel's hold), `2e6ba738` (refusals), `308e7c67` (Late Far proxies). The term itself was rewritten in the same window (`3f7e098a`, `78727a00`, `88941ee2`, `31fae9d1`), and none of those rewrites added these fields.

Fix to the text: after "Arena bytes in use", add "the streaming counters: Late Chunks and Late Far proxies, late mips, Chunks loaded, unloaded and not loaded, bytes read, pool entries freed and Meshes and Materials refused, and a Fast travel's hold and Far field fill".

### L7-G03

`CONTEXT.md:254-255`, Language, Verification, **Baseline**: "A commit re-records one when it changes a timing the Baseline held or its Harness version; a commit that changes only its counters does not, and starts no line on its Series." That sentence landed on 2026-11-08 (`e251e21e`).

The code that landed since (`605f40b7`, 2026-11-20):
- `src/bench/history/rerecord.rs:1-6`: "`praxis history rerecord`: runs of `main` re-record the counters of this machine's Baselines where they moved. A re-recorded Baseline keeps its timings and Harness version, which a person chooses, so its record starts no line on its Series".
- `:30-35`: `CounterRerecord::Rerecorded { baseline, moved }`.
- `:176-182`: `rerecord_baselines` "writes the Baselines whose counters moved [...] with the commits each re-recorded Baseline's counters moved between".

The rule the glossary means is `remeasured` (`src/bench/history/import.rs:278-282`): "Counters alone changing, or fields added, measured nothing again". So the code calls a counters-only commit a re-record, and the glossary says such a commit is not one. **Suspect** (`CONTEXT.md:287`, "imported or re-recorded Baseline") and **Regression** (`CONTEXT.md:299`, "no re-recorded Baseline of its scene covers") use the glossary's sense. A reader who reaches them from the command or its report gets the other sense.

Fix to the text: "A commit re-measures one when it changes a timing the Baseline held or its Harness version, and only that starts a line on its Series or covers a Regression; a commit that re-records its counters alone, as `praxis history rerecord` does, does neither." Also rename "re-recorded" to "re-measured" in Suspect and Regression.

### L7-G04

`CONTEXT.md:286-287`, Language, Verification, **Suspect**: "A Bench record on a Series whose CPU or GPU frame median sits outside the median of up to 8 records before it, ± the larger of 4 median absolute deviations and 3% [...]".

The code, `src/bench/history/suspects.rs:9-11`: "/// The fewest records before a record that judge it; with fewer, as at the start of a line, it is not judged. `const FEWEST_JUDGING_RECORDS: usize = 3;`". At `:91-93`, `band` does `if window.len() < FEWEST_JUDGING_RECORDS { return None; }`, and `suspects` (`:59-60`) returns no shift for that record. The constant is at `8bc34eb1` too (`git show 8bc34eb1:src/bench/history/suspects.rs`, line 11), so M3.5's lane 7, which re-read this term for its L7-G02, missed it.

Fix to the text: "sits outside the median of the 3 to 8 records before it on its line".

### L7-R01

`docs/agents/project-shape.md:3`, intro: "What each directory and file of the tree holds. Read when looking for where something lives, or when a change adds, moves or renames a file; `CLAUDE.md` keeps a line per directory. A change to the layout changes this file in the same PR." `CLAUDE.md:139`, Project shape: "What each holds, file by file and pass by pass: `docs/agents/project-shape.md`." Both lines landed in scope (`git blame`: 5c4e8955).

The tree has two files the shape does not name, each landed in scope before the shape was written:

- `src/cook/chunk.rs:1-2` (added 95eef204, 2026-11-06): "//! The Chunk cook (ADR 0018): a Reference scene's static content cut into Chunks, each Placement and light in the Chunk that holds its pivot." The `cook/` entry (`docs/agents/project-shape.md:77-93`) names `launch.rs`, `far_proxy.rs` and `far_proxy/bake.rs` and not `chunk.rs`.
- `src/bench/history/night.rs:1-3` (added 7a4429bc, 2026-11-06): "//! A Night's directory: the Verdict it started from, then one directory per run, numbered in the order benched, each holding the commit it benched and the results its bench wrote." The `bench/` entry (`:15-27`) names `history/import.rs`, `history/suspects.rs` and `history/rerecord.rs` and not `night.rs`.

`grep -n 'chunk.rs\|night' docs/agents/project-shape.md` finds only `chunks.rs` (streaming, `:52`) and "the nightly sweep" (`:178`). Older files the shape leaves to their directory's line (`history/verdicts.rs`, `provenance.rs`, `report.rs`, `cook/mesh.rs`, `cook/texture.rs`), and the root's `assets/`, `docs/` and `.github/`, are outside the scope's rule and not filed; they bear on the same "each directory and file" claim.

Fix to the text: add `chunk.rs, the Chunk cook: a scene's static content cut into Chunks` to the `cook/` entry and `history/night.rs, a Night's directory: its Verdict and one directory per run` to the `bench/` entry, or reword the intro to "what each directory holds, and the files a reader would not find from it".

### L7-R02

`CLAUDE.md:281-288`, Review checklist: "Frame stats still see new work: GPU buffers, textures and acceleration structures are created through the GPU memory ledger (`GpuMemory`) under their kind, ...; the GPU stats readback's buffers stay out, as they measure the frame (ADR 0013); per-frame GPU writes add to `uploaded_bytes`, ..." (lines 281-283 and 286 landed in scope, f99166b5).

The code: `src/render/staging.rs:179-187`:

```
/// Staging for load-time uploads (ADR 0017): each write goes through a
/// `StagingBelt` into a copy on one encoder, with the BLAS builds that read
/// them and the copies that move Textures between Texture classes, which
/// `submit` sends ahead of the next frame's. The belt goes with it, so its
/// staging buffers live only until the GPU has run the copies and never
/// count in `gpu_memory_bytes`.
pub struct LoadStaging {
    device: wgpu::Device,
    pending: Option<(wgpu::util::StagingBelt, wgpu::CommandEncoder)>,
```

The belt is created at `src/render/staging.rs:348` (`wgpu::util::StagingBelt::new(device.clone(), LOAD_CHUNK_BYTES)`), outside `GpuMemory`; its bytes reach Frame stats only as `uploaded_bytes` (`src/render/renderer.rs:698`). The capture's offscreen target and read-back buffer are likewise made by `context.device` directly (`src/capture/mod.rs:332,465`). The checklist's only named exception is the readback's.

Fix to the text: "...; the GPU stats readback's buffers and the load staging's belt stay out, the one measuring the frame (ADR 0013), the other counted in `uploaded_bytes`; ..."

### L7-R03

`docs/agents/audit.md:81`, Lane 2, image quality: "Method: readers run AFK and prepare the gallery, each scene's mean 8-bit difference and linear-light RMSE against Cycles (recorded as that scene's first Cycles Baseline; the Baseline mechanism itself is a fix ticket the spec charts), ..." The line is unchanged since `8bc34eb1` (`git blame`).

The mechanism landed in scope, 88868582 "Cycles Baselines: judge each exported scene's distance from its render", 2026-11-02, a descendant of `8bc34eb1`. At the frozen commit `bench/cycles/` holds eleven Baselines (`far-coplanar`, `helmet`, `minification`, `normal-tangent`, `orrery`, `pbr-sweep`, `shadows`, `texture-wall`, `town`, `uv-grid`, `world`), and `scripts/gallery.sh:19-25`: "a scene with a Cycles Baseline (bench/cycles/<scene>.json) is judged against it at the Goldens' frame: a figure past its Baseline by compare.py's tolerance prints a FAIL line ... With PRAXIS_RECORD_CYCLES_BASELINES set, it records each exported scene's Baseline from this run instead." `docs/agents/project-shape.md:157-160` describes the same Baselines.

Fix to the text: "... each scene's mean 8-bit difference and linear-light RMSE against Cycles, judged against its Cycles Baseline in `bench/cycles/` (recorded only for a scene that has none, with `PRAXIS_RECORD_CYCLES_BASELINES`), ..."

### L7-A07 (killed)

`docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:3`, status: "Accepted 2026-10-12 ([#22]). Planned; today a compute pass appends the Meshlets of the LOD selected for each Instance [...]".

What the decision names and the tree ships: the depth pre-pass and the lit pass at depth equal with one or more indirect draws per pipeline (`src/render/main_pass.rs:315-368`), the Froxel lists from the light buffer (`src/render/froxel_pass.rs:22-31`), the shadow mask of up to 59 lights (`src/render/shadow_mask_pass.rs:35`), the two-phase Hi-Z culling (`src/render/shaders/culling.wgsl:133-151`). What stays unbuilt: the transparent pass (`:19`) and a mesh-shaded pre-pass (`:20`). ADRs 0016 (`:3`), 0017 (`:3`) and 0018 (`:3`) call the same state "Partly built" or "Partly implemented". The word was in the status at `8bc34eb1`, when the pass was already built, and M3.5's lane 7 did not give it a verdict.

Fix to the text: "Accepted 2026-10-12. Partly implemented, the transparent pass and a mesh-shaded pre-pass planned; today a compute pass appends [...]".

## Rule candidates

Each candidate is a slip seen twice, with the ids of every surviving Finding
of its kind and the one-sentence rule that would have caught it; the rules
ticket decides, each rule backed by at least two Findings. The readers
proposed five; the verifier read each against its ids and marked it
`stands` or `falls`, and added one across readers. All five that stand are
forms of the slip `M3.5`'s candidates named, a sentence not moved by the
change that made it false. Since 2026-11-04, the Review checklist has
asked that every name a change renames, removes or changes the behaviour of
(a pass, buffer, format, Blob field or constant) be grepped in the ADRs and
`CONTEXT.md`. One candidate is a slip that item names and that happened
after it: the culling pass's offsets grew from 8 to 16 bytes a slot, and
ADRs 0003 and 0017 kept the old figure (L7-A01, L7-A05). One predates it,
from M3's exit (L7-A06, L7-A08). The other three are changes its words do
not name: a member added to a set the glossary lists, work moved to
another thread or machine, and work landed that a doc presumed unbuilt.
The rules ticket may weigh the item's wording for those three apart from
the one slip past it.

| rule | finding ids | status |
|---|---|---|
| A PR that adds a member to a set a glossary entry lists (a `FrameStats` field, a new source of an Instance) extends that entry in the same PR, found by grepping `CONTEXT.md` for the term whose siblings it lists. | L7-G01, L7-G02 | stands (both confirmed; the checklist's grep rule covers names a change "renames, removes or changes the behaviour of", not ones it adds) |
| A figure an ADR derives from a struct's size or a capacity test cites the test that pins it, and the PR that moves the pin greps the ADRs for the old figure. | L7-A01, L7-A05 | stands (both confirmed) |
| An amendment that changes a status word of one ADR (provisional, planned, final) greps the other ADRs for clauses that borrow it ("as ADR n's ... are"). | L7-A06, L7-A08 | stands (both confirmed) |
| An ADR's status line says, per Consequence, built or planned; the PR that builds one moves the status line. | L7-A04, L7-A07 | falls: L7-A07 killed, one Finding left |
| A PR that moves work to another thread or machine, or stops it on one (a pool, a worker, a scheduled task), greps the ADRs for present-tense clauses naming the place it leaves. | L7-A02, L7-A09 | stands (both confirmed) |
| Across readers: a PR that lands work a doc presumes unbuilt (a "the rest is planned", a "record the first one") rewrites that clause in the same PR, in `docs/agents/*` as in the ADRs. The same slip as M3.5's stood "deferred work lands" rule (its L7-B03, L7-B07), here past the ADRs. | L7-R03, L7-A04 | stands (R03 narrowed, A04 confirmed) |

## Killed

The Findings the verifier killed, 1 of 16, each with its reason; its
evidence block is above, marked killed.

| id | claim | reason |
|---|---|---|
| L7-A07 | ADR 0019's status line reads "Planned; today ...", but its decision is built save the transparent pass and a mesh-shaded pre-pass: the 4x MSAA depth pre-pass, the lit pass at depth equal, the Froxel lists, the shadow mask and the two-phase culling all ship. | The "Planned; today ..." status is the ADRs' own form, its "today" half holding what the code ships, and no sentence of it is false against the code. M3.5's stood rule candidate (`docs/audit/m3.5/docs-drift.md:2178`) names it as the form "the ADRs do, 'Planned; today ...', so the 'today' half is what the code holds"; ADR 0012 uses it too. ADR 0019:3's "today" half states the built pass in full (two-phase culling, the pre-pass, the Hi-Z pyramid, the shadow mask, the lit pass at depth equal, the Froxel lists, up to 59 lights), and the decision is not wholly built (the transparent pass, `:19`, is not), so "Planned" for the decision as written holds. The reader's evidence is a label that differs from ADRs 0016-0018 and 0020 ("Partly built", "Partly implemented"), not a clause the code contradicts; no reader of the status is misled about what ships. |
