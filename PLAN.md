# Phase 5: Integrate bevy_ecs + handle pools

## Context

Phase 4 done. Three hardcoded `glam::Mat4` transforms plus a loaded
`DamagedHelmet.glb` get rendered by `State::render()` via a nested loop over
`self.transforms` and `self.primitives`. Camera, lights, model uniforms, and
GPU buffers all live directly as fields on `State`.

Replace ad-hoc scene data with `bevy_ecs`. Goal: the three helmets are still
on screen at the end, but their `Transform` + `MeshHandle` + `MaterialHandle`
now come from a `bevy_ecs::World` query instead of hardcoded `Vec`s on
`State`.

## Decision: bevy_ecs over rolling own

Rolling a Pikuma-style sparse-set ECS was the prior plan — generational IDs,
type erasure with `Any`, signature bitsets, query borrow gymnastics. Real
Rust skill, weeks of work, ~1500 lines. Rejected on 2026-04-28 in favor of
shipping rendering progress sooner. Rendering is this project's prize; ECS
is plumbing.

`bevy_ecs` is used as a **sub-crate only** — no transitive dep on
`bevy_render` / `bevy_app` / the full Bevy engine. The crate provides
`World`, `Component`, `Query`, `Schedule`, `Commands`. Same shape a
hand-rolled archetype ECS would expose, but production-grade and free.

What you skip learning: generational IDs, sparse set vs archetype storage,
type-erased component pools, query borrow patterns, system scheduling. What
you still own: handle pools (mesh/texture/material), instance buffers,
renderer pull-shape. The rendering-adjacent concerns survive.

## Architecture decisions

| Decision | Choice | Why |
|---|---|---|
| ECS crate | `bevy_ecs` (current 0.x), sub-crate only | Just World/Component/Query/Schedule/Commands; no full Bevy app/render |
| Components | `#[derive(Component)]` on `Transform`, `MeshRef`, `MaterialRef`, `LightRef` | Authoring shape: `glam::Affine3A` and handles, never GPU bytes |
| Resource pools | `MeshPool`, `TexturePool`, `MaterialPool` as `#[derive(Resource)]` | Pools live in `World` as resources but their internals (handle-indexed `Vec<T>`) are still ours |
| Handles | Plain `MeshHandle(u32)` etc. — no generational, no `Arc` | Phase 5 never unloads. Revisit at Phase 13 (streaming) |
| Systems | Free fns with bevy params (`Query`, `Res`, `ResMut`, `Commands`) | Bevy auto-wires deps from signatures. No manual `&mut World` plumbing |
| Scheduling | One `Schedule` with `chain()` for now | No need for parallel execution at Phase-5 scale |

## Gotchas

### Don't pull full Bevy

`bevy_ecs` is published as a separate crate from the umbrella `bevy` crate.
Cargo `bevy_ecs = "..."`, **not** `bevy = "..."`. Confirm `cargo tree`
shows no `bevy_render`, `bevy_app`, or `bevy_window` after adding it. If
those appear, the wrong crate is in `Cargo.toml`.

### Bevy version churn

`bevy_ecs` is 0.x and breaks across minor releases. Pin the exact version.
Expect ~1–2 hours of migration work per minor bump (renamed types,
schedule API tweaks). Read the bevy migration guide before bumping.

### Resources vs Components

A `MeshPool` is a singleton — one per world. Use `#[derive(Resource)]`.
Components are per-entity data: `MeshRef(handle)` on each helmet entity.
Don't accidentally make pools components or vice versa — bevy will compile
either way but the semantics break.

### Don't import `bevy_transform::Transform`

`bevy_transform` is a separate crate that ships its own `Transform` type
(with parent/child hierarchy support). It's tempting but **don't pull it
in** — that's a step toward the full Bevy stack. Define our own
`Transform` as a `glam::Affine3A` newtype. Hierarchy lands in Phase 7 and
will use our own pattern.

### Query borrow rules

Bevy enforces disjoint mutable access via system parameters. Two systems
that both write the same component must run sequentially (`chain()`) or
read instead of write. At Phase 5 scale this never bites; it's worth
knowing the rule before it does.

### Authoring data, not GPU layout

ROADMAP Design Principle #2 still binds. `Transform` component =
`glam::Affine3A`, not `[[f32;4];4]`. Renderer packs GPU bytes during a
`prepare_renderables` system. The shape lesson here is: components
describe the *world*, the instance buffer describes what the *GPU sees*,
and the prepare system bridges them.

### `vec3<f32>` uniform trap (reminder)

No new uniforms expected here, but if one slips in: no `vec3<f32>` in
WGSL uniform structs. Use `vec4<f32>` or an explicit pad field.

---

## Mental models

Cross-cutting concepts. Worth reading once before Step 1.

### bevy_ecs primer

Five concepts cover almost everything in Phase 5:

```rust
use bevy_ecs::prelude::*;

#[derive(Component)] struct Transform(glam::Affine3A);
#[derive(Component)] struct MeshRef(MeshHandle);

#[derive(Resource, Default)]
struct MeshPool { meshes: Vec<Mesh> }

fn spawn_helmets(mut cmd: Commands) {
    cmd.spawn((Transform(glam::Affine3A::IDENTITY), MeshRef(MeshHandle(0))));
}

fn render(q: Query<(&Transform, &MeshRef)>, pool: Res<MeshPool>) {
    for (xf, m) in &q { /* draw */ }
}

let mut world = World::new();
world.insert_resource(MeshPool::default());
let mut sched = Schedule::default();
sched.add_systems((spawn_helmets, render).chain());
sched.run(&mut world);
```

- **Component** — pure-data marker on a struct. Components attach to
  entities; queries find entities with the requested combination.
- **Resource** — singleton in the world. Use for asset pools, render
  context, frame counters, anything there's exactly one of.
- **Query** — system parameter that iterates matching entities.
  `Query<(&A, &mut B)>` reads `A`, writes `B`. An entity must have *all*
  listed components to appear.
- **Commands** — buffered mutations. `cmd.spawn(...)` and
  `cmd.entity(e).despawn()` queue changes that flush at sync points
  (between systems by default). Lets a system mutate the world while
  iterating queries.
- **Schedule** — list of systems with ordering rules.
  `add_systems((a, b, c).chain())` runs strictly in order. `.before()` /
  `.after()` add finer constraints when needed.

Read `bevy_ecs`'s docs.rs page once, then come back. The crate's
docstrings are excellent and cover the corner cases.

### Handles as lightweight references (still ours)

Pool design is not bevy's job. Handles are plain integers indexing into a
`Vec`. Many entities share one mesh by copying the handle. No `Arc`, no
lifetimes, no borrow-checker pressure on game code.

```rust
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MeshHandle(pub u32);

#[derive(Resource, Default)]
pub struct MeshPool {
    meshes: Vec<Mesh>,
}

impl MeshPool {
    pub fn insert(&mut self, mesh: Mesh) -> MeshHandle {
        let h = MeshHandle(self.meshes.len() as u32);
        self.meshes.push(mesh);
        h
    }
    pub fn get(&self, h: MeshHandle) -> &Mesh {
        &self.meshes[h.0 as usize]
    }
}
```

Generational handles for resources are optional here — plain `u32` is
fine while assets never unload. Revisit at Phase 13 (streaming).

Note: handles are *not* `bevy_ecs::Entity`. Entities are game data and
live in `World`. Resources are engine data — owned by `World` as
`Resource`s but conceptually distinct. Keeping that line clean is Design
Principle #1 in `ROADMAP.md`.

### Instance buffers vs per-entity uniform buffers (Step 3 background)

Step 3 uses a **single instance buffer** rather than per-entity uniform
buffers. Quick tour:

- **Per-entity uniform buffers** — each entity gets its own small
  buffer; one `queue.write_buffer` call per entity per frame. Driver
  overhead scales linearly with entity count. Easy to write but
  architecturally a dead end — GPU-driven rendering, RT TLAS rebuilds,
  and instancing all want their data consolidated. Phase 5 skips this
  trap.
- **Instance buffer** — one structured buffer holds every renderable's
  transform end-to-end. Draws use `base_instance` +
  `@builtin(instance_index)` to index into it. One `queue.write_buffer`
  per frame total. **This is what Step 3 builds.** Same shape extends
  cleanly to GPU-driven rendering, RT, and bindless.
- **Single uniform buffer + dynamic offset** — older variant using
  uniform buffers and per-draw dynamic offsets. Works fine but has
  stricter alignment rules than storage buffers. The instance-buffer
  pattern with `@builtin(instance_index)` is the modern default.
- **GPU-driven rendering** — Phase-11 territory. CPU writes a
  draw-command buffer; GPU consumes it via indirect draws. Per-instance
  data is the same instance buffer Step 3 builds, plus a draw-command
  buffer alongside it. Layering this on top of Step 3's architecture
  is additive, not a rewrite.

Phase 5 ships the instance-buffer pattern with a simple "rewrite the
whole buffer every frame" upload. Incremental updates (dirty flags) are
Phase-7 scene-graph work; indirect draws are Phase 11.

### Pull-renderer and uber-shader (forward look)

Two terms from `ROADMAP.md`'s Design Principles that surface in phase
planning but aren't built yet.

- **Pull-renderer.** Each frame, the renderer queries the world for
  what to draw (`world.query::<(&Transform, &MeshRef, &MaterialRef)>`)
  and pulls the data it needs. The opposite is a push model: ECS code
  calls into the renderer to register draws. Pull keeps sim ignorant of
  wgpu — Design Principle #3.
- **Uber-shader.** One large shader with branches or feature flags for
  every material variant, instead of N specialized shaders. Trades
  runtime branching for build-time and binding simplicity. Phase 5 does
  not write one; the seed is just "do not proliferate per-material
  shaders without thinking".

### GPU-driven rendering (forward look, post-Phase-5)

"Everything on GPU" is the modern AAA pattern Spartan and UE5 advertise.
It is a *renderer* concern, separate from the ECS choice in this phase.
Praxis can adopt it later without revisiting Phase 5 work.

Concretely, GPU-driven rendering replaces these CPU-side patterns:

- **Per-entity uniform buffer** → **one persistent scene buffer**
  holding every transform / material entry, updated incrementally via
  dirty flags.
- **Bind-group-per-material** → **bindless**: one giant descriptor array
  of textures; shaders index into it by material ID. No per-material
  bind-group churn.
- **Submit-draw-per-mesh** → **indirect drawing**: CPU writes a command
  buffer once; GPU reads it and dispatches every draw without further
  CPU involvement.
- **CPU-side frustum / occlusion culling** → **compute-shader culling**:
  GPU tests visibility against camera + depth pyramid, builds the
  visible-instance list the indirect draw consumes.
- **One wgpu buffer per mesh** → **one mega-buffer**: meshes are
  sub-allocations referenced by offset; helps streaming and reduces
  bind churn.

Where the threshold sits, roughly: CPU draw-call submission becomes the
bottleneck around 3,000–5,000 visible draws per frame on modern
hardware. A Skyrim-shape RPG with chunked streaming and aggressive
culling typically lives in the hundreds to low thousands — well below
the threshold.

Phase-5 lock-in for going GPU-driven later: low. The lock-in lives in
the renderer — per-entity uniform buffers, bind-group-per-material,
immediate-mode draw submission. All of those are already on the rewrite
list whenever a rendering-focused phase tackles bindless or indirect.

---

## Steps

Each step ends with a visible or verifiable checkpoint. The rule: no step
advances until its checkpoint holds.

### Step 1: Add bevy_ecs, define components and pools

_Pin a current `bevy_ecs` 0.x version. Define `Transform`, `MeshRef`,
`MaterialRef`, `LightRef` components. Define `MeshPool`, `TexturePool`,
`MaterialPool` as resources._

**Notes.** Sub-crate only — confirm `cargo tree | grep bevy` shows just
`bevy_ecs` (and its small graph of internal deps), not `bevy_render` or
`bevy_app`. Components are pure data: `Transform = glam::Affine3A`, refs
wrap `MeshHandle(u32)`. Pools live in the world as resources, but their
internals are still ours. No render changes yet.

Where to look:

- `Cargo.toml` — add `bevy_ecs = "0.x"` (check `cargo search bevy_ecs`
  for current; pin exact). After `cargo build`, `cargo tree` should
  show only `bevy_ecs` from the bevy family.
- New `src/components.rs` — `#[derive(Component)]` types: `Transform`,
  `MeshRef`, `MaterialRef`, `LightRef`. Wire with `mod components;`
  in `main.rs`.
- New `src/resources.rs` (or `src/render/resources.rs`) — handle
  newtypes (`MeshHandle(pub u32)` etc.) and `#[derive(Resource)]`
  pools. `MeshPool` wraps `Vec<Mesh>`; `Mesh` is the inner type.
- Existing `GpuPrimitive` struct (`src/state.rs:12`) is the candidate
  shape for `Mesh` — but decide first whether one `Mesh` = one
  primitive or one mesh = many primitives (the helmet has several
  primitives, so this is a real design call). Either is defensible;
  note the reasoning.
- Add a `world: bevy_ecs::World` field to `State` (`src/state.rs:19`),
  initialized in `State::new`. Resources inserted right after via
  `world.insert_resource(MeshPool::default())`.

- [x] Done

Checkpoint: `cargo build` passes. `World::new()` runs and resources
register without panicking.

### Step 2: Migrate 3 helmets into ECS

_Replace `State.transforms` and `State.primitives` with ECS entities
carrying `Transform` + `MeshRef` + `MaterialRef`. Old render path stays —
for now the renderer still runs nested loops, but its data source is the
world._

**Notes.** Spawn at startup via `cmd.spawn((Transform(...), MeshRef(h),
MaterialRef(m)))`. The render path can iterate
`world.query::<(&Transform, &MeshRef, &MaterialRef)>()` and look meshes /
materials up in pools. It will look transitional — that's fine. The
milestone is "data flow works", not "render is rewritten". The picture
on screen must be identical.

Where to look:

- `src/state.rs:232` — `load_model("assets/DamagedHelmet.glb")` stays;
  the loaded primitives feed `MeshPool::insert` (and texture data
  feeds `TexturePool::insert`) instead of the local `primitives` Vec.
- `src/state.rs:234-238` — hardcoded `transforms` Vec → 3
  `cmd.spawn(...)` calls at the end of `State::new` (one per helmet
  position). All three share the same `MeshHandle` and
  `MaterialHandle`.
- `src/state.rs:240-310` — per-primitive vertex/index/texture upload
  moves inside `MeshPool::insert` (or a `load_model_into_pool`
  helper). The pool returns one `MeshHandle` shared by all 3 spawns.
- `src/state.rs:33` (`primitives` field) and `src/state.rs:37`
  (`transforms` field) — **deleted from `State`**; their data lives
  in the world now.
- `src/state.rs:466-476` (per-transform model uniform write loop) and
  `src/state.rs:568-578` (nested render loop) — **stay** for this
  step; they read from `world.query` instead of `self.transforms` /
  `self.primitives`. Ugly but transitional. Step 3 cleans them up.
- `src/state.rs:36` (`model_buffers`) — **also stays** this step:
  still one buffer per renderable. Deleted in Step 3.

- [x] Done

Checkpoint: **three helmets still on screen, unchanged**. `State` no
longer owns `Vec<Transform>` or hardcoded primitive lists. Milestone
commit.

### Step 3: Renderer queries world via prepare pass + single instance buffer

_Delete the hardcoded loops and per-index `State.model_buffers`. Add a
`prepare_renderables` system that walks the query, builds
`Vec<InstanceData>`, and uploads to one persistent instance buffer.
Per-entity GPU buffers go away._

**Notes.** Skip the `HashMap<Entity, wgpu::Buffer>` per-entity pattern.
It is a dead end: GPU-driven rendering, RT TLAS rebuilds, and instancing
all want one structured buffer of instance data, not many small uniform
buffers. Bake the right shape now — see *Instance buffers vs per-entity
uniform buffers* in Mental models for the full reasoning.

Concrete shape:

- `State` owns one `wgpu::Buffer` of `InstanceData` (model matrix for
  now; normal matrix + material ID get added in later phases), sized
  for an initial capacity and resized on overflow.
- A `prepare_renderables` system each frame walks
  `world.query::<(&Transform, &MeshRef, &MaterialRef)>()`, builds a
  `Vec<InstanceData>`, and uploads it once via `queue.write_buffer`.
  One write per frame, regardless of entity count.
- Each draw call uses `base_instance` + instance count to span its
  slice of the instance buffer. Vertex shader reads its
  `@builtin(instance_index)` in WGSL and uses it to look up the model
  matrix in the buffer.
- `prepare_renderables` also produces the `(mesh, material,
  instance_idx)` draw list the renderer iterates.

Sub-decision: storage-buffer bind group vs vertex-step instance
attribute for delivering per-instance data. **Recommend storage
buffer** — extends cleanly to indirect draws and bindless later, and
matches Load-bearing decisions #3 / #5 in `ROADMAP.md`.

What this is **not** doing yet: bindless materials (Phase 10), indirect
draws (Phase 11), persistent buffer with incremental dirty-flag updates
(Phase 7 scene-graph work). The instance buffer is fully rewritten each
frame here. Fine at Phase-5 entity counts; the architecture is right
even if the implementation is the simple version.

Where to look:

- New `src/render/instance.rs` (or similar) for `InstanceData`
  (`#[repr(C)]`, `bytemuck::Pod + Zeroable`). Start with
  `model: [[f32; 4]; 4]`; normal matrix joins later (Phase 6 lighting
  work needs it).
- `src/state.rs:36` (`model_buffers` field) — **deleted**. Replaced
  by one `wgpu::Buffer` (the instance buffer) plus its bind group.
- `src/state.rs:158-171` (`model_bind_group_layout`) — **repurpose**
  as a storage-buffer layout. Single binding,
  `BufferBindingType::Storage { read_only: true }`, visibility
  `wgpu::ShaderStages::VERTEX`.
- `src/state.rs:342-362` (`model_buffers` Vec creation) — **deleted**;
  replaced by one `wgpu::Buffer` of size
  `cap * size_of::<InstanceData>()` with `STORAGE | COPY_DST` usage,
  and one bind group built from it. Track `cap` so you can detect
  overflow and recreate.
- `src/state.rs:466-476` (per-transform write loop in `update`) —
  **deleted**; replaced by `prepare_renderables` building a
  `Vec<InstanceData>` from the query and one `queue.write_buffer`.
- `src/state.rs:568-578` (nested render loop) — **collapsed**:
  `prepare_renderables` returns a draw list; render iterates that,
  setting per-draw bindings (texture for the mesh's material) and
  using `draw_indexed(idx_range, 0, instance_range)` where
  `instance_range = instance_idx..instance_idx + 1` for now. (Real
  instancing — multiple entities sharing a mesh collapsed into one
  draw with `instance_count > 1` — is the natural follow-up once
  this works.)
- `src/shader.wgsl` — vertex shader stops reading from a uniform
  `Model` and instead reads from
  `@group(3) @binding(0) var<storage, read> instances:
  array<InstanceData>;`, indexed by `@builtin(instance_index)`. The
  uniform `Model` struct goes away. Match WGSL `InstanceData` layout
  to the Rust side (16-byte alignment; column-major matrices).

- [x] Done

Checkpoint: same 3 helmets, but `State` no longer owns `transforms` or
any per-entity buffers. One instance buffer, one `queue.write_buffer`
per frame, draws indexed into it. Render data flows from the world.

### Step 4: Camera and input become systems

_Extract the camera fly logic out of `State::update()` into a
`fn camera_system(...)` registered in the schedule. Same for input
handling. `State::update()` shrinks to `self.schedule.run(&mut
self.world)`._

**Notes.** Free functions with bevy params:
`fn camera_system(mut q: Query<&mut Transform, With<Camera>>, input:
Res<Input>, time: Res<Time>) { ... }`. Decide whether the camera is an
entity (with `Camera` + `Transform` components) or a `Camera` resource —
both defensible. Pick one and note the reasoning. The systems list
lives in the schedule; for now `chain()` enforces order.

Where to look:

- `src/state.rs:30` (`keys_pressed: HashSet<KeyCode>` field) → `Input`
  resource in the world. The winit event handler in `main.rs` writes
  into it instead of into a `State` field. Keep `Input` minimal: the
  pressed-key set + maybe a per-frame `just_pressed` set for Step 5.
- `src/state.rs:32` (`pub camera: Camera` field) → either a `Camera`
  resource or a camera entity with `Camera` + `Transform` components.
  Resource is simpler; component shape matches what bigger engines
  do. Pick one for now — switching later is local.
- `src/state.rs:400-431` (input + camera fly logic in `update()`) →
  `fn camera_system(...)` in new `src/systems/camera.rs`. Frame `dt`
  comes from a `Time` resource a separate `tick_time` system updates
  each frame.
- `src/state.rs:432-489` (uniform packing for camera + lights) →
  `prepare_camera_uniforms` and `prepare_light_uniforms` systems, or
  fold into one `prepare_frame_uniforms`. They run before
  `prepare_renderables` (chain ordering).
- `src/state.rs:491-586` (`render`) — stays as a method on `State`
  for now; the `prepare_renderables` system feeds it the draw list
  via a `FrameData` resource (Load-bearing decision #4 in
  `ROADMAP.md` — the renderer never reads `World` mid-frame, only
  the snapshot a prepare system produced).
- `State::update()` shrinks to roughly `self.schedule.run(&mut
  self.world);`. Ordering enforced via
  `add_systems((tick_time, camera_system, prepare_frame_uniforms,
  prepare_renderables).chain())`.
- New `src/systems/` module — declare in `main.rs` with
  `mod systems;`, sub-modules per system file (`camera.rs`,
  `prepare.rs`, etc.).

- [x]

Checkpoint: camera controls identical to Phase 4. `State::update()` is
mostly a `Schedule::run` call.

### Step 5: Spawn / despawn at runtime

_Keybind (e.g. `G`) spawns a new helmet at the camera's current position;
another (e.g. `H`) despawns the most recently spawned. Verifies bevy's
`Commands` deferred mutation works under real use, and that the instance
buffer correctly reflects entity churn._

**Notes.** Bevy handles deferred spawn / kill internally — `cmd.spawn`
and `cmd.entity(e).despawn()` queue at the right sync points; you don't
manage `pending_spawn` / `pending_kill` yourself. Test the nasty cases:
spawn during iteration, spawn 100 entities in one frame, rapid spawn +
kill, despawn then verify queries skip the entity. Confirm the instance
buffer doesn't carry stale entries for dead entities.

Where to look:

- `Input` resource (Step 4) gains a `just_pressed: HashSet<KeyCode>`
  set, populated by the winit event handler in `main.rs` on
  `KeyboardInput { state: Pressed, .. }` and cleared at the end of
  each frame (or by a system at the start of the next frame).
- New `src/systems/spawn.rs` —
  `fn spawn_helmet_system(mut cmd: Commands, input: Res<Input>,
  camera: Res<Camera> /* or Query<&Transform, With<Camera>> */,
  helmet_assets: Res<HelmetHandles>, mut spawned:
  ResMut<RuntimeHelmets>)`. Reads `G`, calls `cmd.spawn(...)` with a
  `Transform` at the camera position, pushes the new `Entity` into
  `RuntimeHelmets`.
- Keep mesh + material handles for the helmet in a small resource
  (`HelmetHandles { mesh: MeshHandle, material: MaterialHandle }`)
  inserted at startup, so the spawn system doesn't have to look
  them up.
- Despawn system: reads `H`, pops the last `Entity` from
  `RuntimeHelmets`, calls `cmd.entity(e).despawn()`. Bevy flushes
  between systems; the next frame's `prepare_renderables` query
  won't see the dead entity.
- Instance buffer cleanup is automatic — Step 3 rewrites it whole
  each frame from the live query, so dead entities vanish
  immediately. Confirm by spawning 5, despawning all 5, checking
  `Vec<InstanceData>::len() == 3` (the original startup helmets).
- Register the two new systems in the schedule, chained after
  `camera_system` so they see this frame's input but before
  `prepare_renderables`.

- [x]

Checkpoint: can add and remove helmets live without crashes, leaks, or
stale-entity rendering. Closing the app drops all pools cleanly.
