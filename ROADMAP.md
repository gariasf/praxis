# Praxis Roadmap

Step-by-step plan for building a 3D engine in Rust.
Each milestone ends with something visible and runnable.

**Target shape:** a big-world, lore-heavy RPG (Skyrim-style) targeting
**cutting-edge hardware**, not mass-market scalability. The engine prioritizes
image quality and rendering correctness over breadth of compatibility — no
PS4-class baseline, no aggressive low-spec fallbacks. This anchors which
rendering features are core: PBR, hybrid baked + ray-traced lighting,
GPU-driven rendering, streaming, LOD, scene hierarchy, long view distances.
"Polish" is reserved for genuinely optional things (HDR10 metadata, color
grading) — not RT, not bindless, not GI.

**Lighting model:** hybrid. Baked GI for static environments (sharp, stable,
no temporal smear, no convergence delay). Ray-traced direct shadows + RT GI
for dynamic objects on the high preset. Cascaded shadow maps as a baseline
fallback. Forward+ rendering with stable AA (MSAA / SMAA) — no mandatory TAA,
no upscaling stack, no frame generation. Native resolution is the default.

**Graphics API:** wgpu, long-term. wgpu's experimental ray tracing (inline
ray queries) is sufficient for the planned RT features — no full ray
pipelines required. If a specific subsystem ever exceeds wgpu's safe API,
drop into `wgpu-hal` for that subsystem only. Migrating to raw Vulkan via
`ash` is not on the roadmap and should not influence design decisions.

The engine may grow to cover persistence, scripting, and AI later; the
foundation is rendering.

## Tech Stack

- **Language:** Rust (edition 2024)
- **Graphics:** wgpu (Vulkan on Windows/Linux, DX12 on Windows, Metal on macOS) — long-term home; see "Target shape" above for the wgpu-permanent commitment
- **Windowing:** winit
- **Math:** glam
- **ECS:** bevy_ecs sub-crate (no full Bevy engine dep) — adopted Phase 5
- **Asset format:** glTF (via `gltf` crate)
- **Debug UI:** egui + egui-wgpu (Phase 8)
- **Logging:** tracing + tracing-subscriber
- **Scripting:** Lua via [mlua](https://github.com/mlua-rs/mlua), LuaJIT backend on supported platforms (Phase 14); see "Scripting plan" below

## Architecture

Start with a minimal workspace. Split crates when a module gets big enough
to justify it, not before.

```
praxis/
  crates/
    praxis_core/     # math, common types, ECS
    praxis_render/   # wgpu renderer, shaders, pipelines
  src/
    main.rs          # application entry point, doubles as the "game"
```

Add crates like `praxis_assets`, `praxis_scene`, `praxis_ui` only when
they're actually needed. No empty scaffolding.

---

## Design Principles

How new systems are built. Distinct from the coding principles at the bottom
(those are about process: small commits, run-it-see-it, etc.).

1. **Handle-based resources.** Meshes, textures, materials, lights live in
   central pools and are referenced by `u32` or generational handles. ECS
   components carry handles, never owned GPU buffers. Lets the renderer batch,
   instance, or move to bindless later without rewriting components.

2. **Authoring data in components, not GPU byte layout.** `Transform` is
   `glam::Affine3A`, not `[[f32; 4]; 4]`. The renderer packs GPU bytes on
   upload. Keeps ECS readable and independent of wgpu specifics.

3. **Renderer pulls, entities don't push.** No `entity.draw()`. The renderer
   queries the ECS for renderables, sorts and batches, then issues draws.
   Prerequisite for culling, instancing, and (eventually) GPU-driven rendering.

4. **One uber-shader, features behind flags.** Resist the urge to spawn a new
   pipeline per effect. Add shader branches, push constants, or uniforms.
   Fewer PSOs, fewer state changes, less to reason about.

Inspired by modern AAA renderers (Spartan Engine, id Tech). Scoped to what a
learning engine on wgpu can actually benefit from. Even if bindless and
GPU-driven draws never happen here, designing around these shapes keeps those
doors open.

---

## Load-bearing decisions

Six architectural choices that, if locked in early, let RT, GI, GPU-driven
rendering, bindless resources, and Lua scripting layer in cleanly later.
These are the "get them right early or rewrite later" decisions. Everything
else is either swappable or local enough to refactor without pain.

1. **Material as ID, not bind group.** Materials are entries in a
   structured GPU buffer indexed by `MaterialId`. Shaders read material
   params + texture indices from the buffer. Even pre-bindless, the data
   flow is "lookup by ID" — the bindless transition then swaps the
   texture-sampling implementation, not the surrounding architecture.

2. **One mega-buffer for vertex data, one for index data.** Meshes are
   `(vertex_offset, index_offset, index_count)` sub-allocations in
   `MeshPool`. Indirect draws need it; BLAS construction wants it;
   instancing wants it. Going from buffer-per-mesh to mega-buffer later
   is expensive because every renderer codepath bakes in the smaller
   shape.

3. **Persistent scene transform buffer.** Transform-propagation results
   live in one GPU buffer indexed by renderable ID, updated incrementally
   via dirty flags. No per-entity uniform buffers. This is what
   GPU-driven rendering, RT TLAS rebuilds, and instancing all want.

4. **Frame-data snapshot pattern.** The renderer never reads from
   `World` mid-frame. Each frame, a `prepare` pass builds a `FrameData`
   struct (visible renderables, lights, camera) and the renderer
   consumes that. Render-thread split becomes trivial later (pass
   `FrameData` across a channel); renderer testing becomes possible
   without spinning up a `World`.

5. **Instance data in a structured buffer; draws look up by index.**
   No bind-group-per-mesh, no bind-group-per-instance. The draw's
   `@builtin(instance_index)` is the lookup key into per-instance data
   (transform index, material ID, etc.).

6. **Components stay pure data.** Components contain only data — no
   closures (`Box<dyn Fn(...)>`), no behavior methods smuggled in via
   traits, no `Box<dyn Trait>` for polymorphism. Behavior lives in
   systems (Rust) or, eventually, in script handlers (Lua). This rule
   is load-bearing for two reasons: components serialize cleanly
   (save/load works), and scripted behavior can hot-reload without
   leaving dangling references inside ECS storage. Violating this rule
   means every offending component must be refactored before save/load
   or Lua scripting can ship.

Together, decisions 1–5 are roughly the data architecture of GPU-driven
rendering; decision 6 extends the same shape to scripting and save/load.
Built this way, the later phases (Phase 11 GPU-driven, the RT phases,
Lua scripting) are mostly additive — wire new pipelines onto stable
data flows — rather than re-architectures.

---

## Realistic milestones

Solo, weekend-paced, ~4–5 productive hours per weekend ≈ 200–250 hours
per year. Honest calibration so the architecture isn't sized for a
fantasy timeline:

| Time | Realistic landing zone |
|---|---|
| End of year 1 | Phase 7–8: PBR + scene graph + debug UI. Helmets lit by IBL in a navigable scene with live tweaking and frame-time graphs. |
| End of year 2 | Phase 10–12: bindless + early GPU-driven rendering + cascaded shadows. Frame time roughly stable as entity counts grow into the thousands. |
| End of year 3 | Phase 13–14: initial streaming + Lua scripting foundation. Engine becomes scriptable; gameplay prototyping starts without recompiling Rust. |
| End of year 4 | Phase 15–16: LOD + baked lighting pipeline. Hybrid GI's static half complete. |
| End of year 5–6 | Phase 17–18: RT foundations + RT shadows on the high preset. |
| End of year 7+ | Phase 19–20: RT reflections + GI for dynamic objects. Hybrid lighting model complete. |
| Year 8–10 | First shipped game using the engine, smaller in scope than original Skyrim ambition. Engine mature enough for a second, more ambitious project. |

Reference points: id Tech 4 took Carmack ~4 years full-time with team
support. Casey Muratori's Handmade Hero is 10+ years ongoing. Sebastian
Lague's voxel/RT series spans years per major feature. Solo from-scratch
is genuinely slow — that's the territory, not a skill issue.

The right response isn't lower ambition; it's accepting that the
shipped-game milestone is years out and the first shipped game will be
smaller in scope than the engine could theoretically support. That is
normal. id Software's first 3D game wasn't *Doom*; it was
*Hovertank 3D*.

---

## Scripting plan

The engine is in Rust; gameplay logic — quests, dialogue, NPC behaviors,
encounter scripts, item effects, UI flow, content tuning — lives in
Lua. Same boundary Supergiant uses for Hades: a thin engine API, with
the content shape and reactive logic above it. Designers and modders
iterate on game content without recompiling.

**Choice: Lua via [mlua](https://github.com/mlua-rs/mlua), with the
LuaJIT backend on supported platforms.** Lua has a long track record in
shipped games as the gameplay/content layer above a native engine —
Hades and Hades II (Supergiant), World of Warcraft (UI and addons),
Roblox (the Luau dialect), Factorio, the Civilization series, the
classic Crytek and Far Cry titles. mlua is the most actively
maintained Rust binding and supports both PUC-Rio Lua and LuaJIT.
Rhai was considered (cleaner Rust integration, type-safe) but loses
on ecosystem size, designer familiarity, and runtime speed. WASM was
considered and rejected as overkill for gameplay scripting.

**Boundary shape:**

- **Engine in Rust:** ECS storage, simulation, rendering, physics
  (later), audio (later), asset loading. Per-frame hot paths.
- **Lua on top:** content data (entities, scenes, dialogue, items),
  reactive logic (event handlers), sequenced gameplay (coroutines for
  encounters, cutscenes, timed effects).
- **Lua does not author per-frame systems.** It runs in response to
  events the engine fires. Per-frame logic stays in Rust. This rule is
  what makes the boundary scale.

**Required engine subsystems** (built up over multiple phases, anchored
by Phase 14):

- **Event bus.** Engine systems fire events; Lua handlers subscribe
  and react. pcall-protected dispatch; tracing-integrated error
  logging so a broken script never crashes the engine.
- **Coroutine task scheduler.** Lua coroutines yield on engine
  primitives (timers, events, predicates); engine resumes them when
  conditions are satisfied. Frame-budget aware so a Lua loop can't
  starve the renderer.
- **Hot-reload.** File watcher reloads changed `.lua` modules; Lua VM
  is recreated; engine state (ECS components, GPU resources, scene)
  survives. All persistent Lua state lives in well-defined tables that
  serialize cleanly — engine never tries to snapshot arbitrary VM
  state.
- **Component bindings.** Core components (Transform, Health, etc.)
  get typed Rust → Lua accessors. A `LuaComponent` Rust type wraps an
  opaque Lua table for game-content components defined in Lua.
- **Lua REPL inside debug UI** (Phase 8 surface) for live inspection.

**Hot-reload contract — the load-bearing rule:** components stay pure
data (Load-bearing decision 6). No closures, no behavior methods on
components. Behavior is always in systems or Lua handlers, never
captured inside component storage. Without this discipline, hot-reloading
a Lua script leaves dangling references inside ECS components and the
engine crashes.

**Save/load discipline:** all persistent Lua state is in well-defined
tables with stable shapes. Save files serialize the known set; the
engine never tries to snapshot the whole VM. Same approach Hades uses.

---

## Phase 1 — Window & Triangle (done)

- [x] Set up cargo workspace with a single binary crate
- [x] Create a window with winit, handle close/resize events
- [x] Initialize wgpu: instance, adapter, device, surface
- [x] Write a basic vertex + fragment shader (WGSL)
- [x] Create a render pipeline and draw a hardcoded triangle
- [x] Handle window resize (recreate surface config)

---

## Phase 2 — 3D Fundamentals (done)

- [x] Add glam for math (vectors, matrices, quaternions)
- [x] Implement a perspective camera (view + projection matrices)
- [x] Create vertex buffers and index buffers for a cube
- [x] Pass MVP matrix to the shader via a uniform buffer
- [x] Add basic input handling (keyboard/mouse via winit)
- [x] Implement a fly camera
- [x] Add depth buffer

---

## Phase 3 — Textures & Mesh Loading (done)

### Part A — Texture pipeline

Texture a cube with an image loaded from disk.

- [x] Add `image` crate for decoding PNG/JPG
- [x] Load image bytes and decode to RGBA
- [x] Create a wgpu::Texture and write pixel data with queue.write_texture()
- [x] Create a TextureView and Sampler
- [x] Add a bind group (group 1) with texture view + sampler
- [x] Add UV coordinates to the Vertex struct
- [x] Update shader: sample texture with textureSample()

### Part B — glTF mesh loading

Load and render a glTF model with its textures.

- [x] Add `gltf` crate for parsing .glb/.gltf files
- [x] Extract vertex positions, normals, UVs, and indices
- [x] Render a loaded mesh with its base color texture
- [x] Support multiple meshes in a single file

Good test models: glTF sample models repo (Box, DamagedHelmet, Sponza).

---

## Phase 4 — Basic Lighting (Blinn-Phong) (done)

A lit scene with at least one light source.

- [x] Pass surface normals to the fragment shader
- [x] Implement directional light in the shader
- [x] Add ambient + diffuse + specular components
- [x] Support point lights (with attenuation)
- [x] Pass light data via uniform buffers
- [x] Multiple objects in the scene with different positions

---

## Phase 5 — Integrate bevy_ecs + handle pools

Replace ad-hoc scene data with `bevy_ecs` (sub-crate only — no full Bevy
engine dep). Rolling our own sparse-set ECS was considered and rejected
on 2026-04-28: rendering is the project's prize, ECS is plumbing for it.
Skipping the entity-layer learning trades weeks of foundational work for
faster progress on rendering phases. See `PLAN.md` for the full
phase-scratch plan.

- [x] Pin a current `bevy_ecs` 0.x version; confirm `cargo tree` shows
      no transitive dep on `bevy_render` / `bevy_app` / `bevy_window`
- [x] Define components: `Transform` (`glam::Affine3A` newtype) and
      `MeshRef` — pure data, no GPU bytes. (`MaterialRef` deferred to
      Phase 6 Step 2; `LightRef` not built — lights are a uniform, not a
      pooled handle yet.)
- [x] Don't import `bevy_transform::Transform`; keep our own
- [x] Central resource pool: `MeshPool` as `#[derive(Resource)]`; pool
      internals (handle-indexed `Vec`s) are ours. (`MaterialPool` landed
      in Phase 6 Step 2; no separate `TexturePool` — textures live in
      `MaterialPool`'s per-channel arrays.)
- [x] Components hold handles (`MeshHandle`); not owned GPU resources.
      (`MaterialHandle` added in Phase 6 Step 2.)
- [x] Migrate 3 helmets via `commands.spawn((Transform(...), MeshRef(h)))`
      (`MaterialRef` attached from Phase 6 Step 2 onward)
- [x] Renderer reads via `Query<(&Transform, &MeshRef)>`;
      `prepare_renderables` system builds one persistent instance buffer
      per frame (no per-entity uniform buffers)
- [x] Refactor input/camera to be bevy systems registered in a
      `Schedule`
- [x] Add/remove entities at runtime via `Commands::spawn` /
      `entity.despawn`

---

## Phase 6 — PBR Materials & load-bearing data shapes

Physically-based rendering with metallic-roughness workflow. The PBR
features themselves are well-trodden ground — the real work is wiring
them up using the data shapes the rest of the engine will need:
material-as-ID and mega-buffer mesh storage. See **Load-bearing
decisions** above for why.

- [ ] Read PBR material properties from glTF (metallic, roughness, etc.)
- [ ] Implement Cook-Torrance BRDF in the fragment shader
- [ ] Support albedo, normal, metallic-roughness, AO, and emissive maps
- [ ] Add an environment map / IBL for ambient lighting
- [ ] HDR rendering with tone mapping (ACES or similar)
- [ ] Gamma correction
- [ ] **Materials live in a structured GPU buffer indexed by `MaterialId`**;
      shaders read material params + texture indices from the buffer
      (not from per-material bind groups)
- [ ] **All mesh vertex data in one `wgpu::Buffer`, all index data in
      another**; meshes are `(vertex_offset, index_offset, index_count)`
      sub-allocations in `MeshPool`

References: LearnOpenGL PBR chapters, Google Filament design doc.

---

## Phase 7 — Scene Graph & Persistent Scene Buffers

Parent-child relationships, transform hierarchy, and the persistent GPU
scene buffer that downstream phases (GPU-driven, RT) build on.

- [ ] Implement transform hierarchy (local + world transforms)
- [ ] Parent-child entity relationships in the ECS
- [ ] Propagate transforms down the hierarchy each frame (or on dirty)
- [ ] **Persistent GPU instance buffer** holding every renderable's
      world transform + per-instance data, indexed by renderable ID;
      updated incrementally via dirty flags
- [ ] Scene serialization (load/save scenes from files, RON or JSON)

---

## Phase 8 — Debug UI & In-Game Tools

Moved earlier than originally planned: every phase below benefits from
having an in-game inspector, live tweaking, and frame-time visibility.

- [ ] Integrate egui via egui-wgpu
- [ ] FPS counter, frame time graph
- [ ] GPU profile timings (per-pass GPU duration via wgpu timestamp queries)
- [ ] Entity inspector (select entity, view/edit components)
- [ ] Light controls (position, color, intensity sliders)
- [ ] Render mode toggles (wireframe, normals, depth, individual lighting passes)
- [ ] Console for runtime commands

---

## Phase 9 — Frustum Culling & Frame-Data Snapshot

CPU-side frustum culling, plus the architectural pattern that lets the
renderer run without ever reading the ECS directly mid-frame.

- [ ] Per-entity AABB component (computed on mesh load)
- [ ] Frustum extraction from camera matrix
- [ ] CPU-side frustum culling pass; produces visible-renderables list
- [ ] **`prepare_renderables` pass each frame builds a `FrameData`
      struct** (visible renderables, camera, lights) that the renderer
      consumes; renderer never queries `World` directly
- [ ] Debug visualization of culled vs visible entities (toggle in
      Phase-8 UI)

---

## Phase 10 — Bindless Resources

Replace per-material bind groups with descriptor arrays. Foundation for
GPU-driven rendering and ray tracing.

- [ ] Texture descriptor array (bindless textures); materials reference
      textures by index into the global array
- [ ] Update PBR shader to sample by index from the global texture array
- [ ] Materials and instance data both indexed by ID — no bind-group
      churn between draws
- [ ] Verify required wgpu features on target backends
      (`TEXTURE_BINDING_ARRAY`,
      `SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING`)

---

## Phase 11 — GPU-Driven Rendering

The big architectural win. CPU stops issuing draws per object; GPU does.

- [ ] CPU writes one indirect-draw command buffer per frame
- [ ] `draw_indexed_indirect` consumes it; one CPU call covers many GPU
      draws
- [ ] Compute-shader frustum culling: GPU tests visibility, builds the
      visible-instance + indirect-draw lists itself
- [ ] CPU culling kept around as fallback / debug toggle
- [ ] Verify perf: frame time should hold roughly stable as entity count
      climbs into the thousands

---

## Phase 12 — Cascaded Shadow Maps

Dynamic shadows from directional lights. Baseline shadow tech; later
augmented or replaced by RT shadows on the high preset (Phase 18).

- [ ] Shadow mapping: render depth from the light's perspective
- [ ] Sample shadow map in the main pass
- [ ] PCF or similar filtering to soften shadow edges
- [ ] Cascaded shadow maps for large outdoor scenes
- [ ] Debug visualization of shadow cascades (Phase-8 UI)

---

## Phase 13 — Streaming & World Chunks

A big-world RPG can't fit in memory. The renderer is now ready for the
entity counts streaming brings (GPU-driven, persistent scene buffers,
bindless materials all in place).

- [ ] Divide the world into chunks (fixed-size cells, like Skyrim's
      exterior cells)
- [ ] Async load meshes/textures from disk without stalling the render
      loop
- [ ] Stream in a radius around the camera, stream out beyond
- [ ] Smooth transition across chunk boundaries (no pop-in, no frame
      hitch)
- [ ] Persistent entity state across load/unload (edits to the world
      survive)
- [ ] Chunk-aware mesh-buffer allocation/deallocation in the
      mega-buffer
- [ ] Debug visualization of loaded/unloaded chunks (Phase-8 UI)

---

## Phase 14 — Lua Scripting Foundation

Engine becomes scriptable. mlua VM, entity bindings, event bus,
coroutine scheduler, hot-reload. The point at which gameplay
prototyping becomes possible without recompiling Rust. See
**Scripting plan** above for boundary shape and rationale.

- [ ] mlua integration (LuaJIT backend on platforms that support it,
      PUC-Rio Lua otherwise)
- [ ] Lua VM lifecycle: one VM per game session, recreated on
      hot-reload
- [ ] EntityId bindings: Lua holds opaque handles, every read/write
      goes through engine API (no raw pointer smuggling)
- [ ] Component access API: typed accessors for core components;
      `LuaComponent` Rust type wraps opaque Lua tables for
      game-content components defined in Lua. From Rust's perspective
      `LuaComponent` is still pure data (no `dyn Fn`) — it stores a
      handle into the Lua VM. Lua references inside it must be
      re-bound on hot-reload (see hot-reload contract below).
- [ ] Event bus: engine fires events, Lua subscribes via
      `world:on_event(name, handler)`; pcall-protected dispatch with
      tracing-integrated error logging
- [ ] Coroutine task scheduler: Lua coroutines yield on engine
      primitives (timers, events, predicates); engine resumes when
      conditions satisfied; frame-budget aware
- [ ] File watcher hot-reload: changed `.lua` files trigger VM
      recreation; ECS components and GPU resources survive intact
- [ ] Lua REPL panel in Phase-8 debug UI for live inspection
- [ ] Sample script: spawn an entity, wait, change its color, despawn
      — all in Lua, hot-reloadable

---

## Phase 15 — LOD & Imposters

Big view distances need cheaper versions of far-away geometry. With
GPU-driven rendering in place, LOD selection can run on the GPU.

- [ ] Discrete mesh LODs (glTF supports via extras or separate meshes)
- [ ] Distance-based LOD selection (CPU or GPU side)
- [ ] Imposters for distant vegetation / props (billboards with baked
      lighting)
- [ ] LOD bias and smooth crossfade to hide pops
- [ ] Per-chunk LOD (whole chunks degrade at distance)

---

## Phase 16 — Baked Lighting Pipeline

The static side of hybrid GI. Offline tool bakes lightmaps for static
geometry; runtime samples them. Cheap, sharp, stable — the right answer
for environments the player doesn't change.

- [ ] Lightmap UV unwrapping (or import from authoring tool)
- [ ] Offline baker: cast rays from each lightmap texel, gather direct
      + indirect light, write to lightmap textures
- [ ] Runtime sampling: PBR shader adds lightmap contribution for
      static objects
- [ ] Light probe grid for dynamic objects in baked scenes
- [ ] Probe interpolation in PBR shader
- [ ] Per-chunk lightmap storage, streamed alongside chunk geometry

---

## Phase 17 — Hardware Ray Tracing Foundation

Acceleration structures and inline ray queries via wgpu's experimental
RT features. Hooked into the existing mesh + scene buffers, so RT data
isn't a parallel pile.

- [ ] Enable wgpu's experimental ray-tracing feature flags
      (currently `EXPERIMENTAL_RAY_TRACING_ACCELERATION_STRUCTURE` +
      `EXPERIMENTAL_RAY_QUERY` in recent versions; verify exact names
      against the wgpu release in use, since the API is in flux)
- [ ] BLAS construction per mesh (built once at load, references
      mega-buffer ranges)
- [ ] TLAS construction per frame (or incrementally, referencing the
      persistent instance buffer)
- [ ] Inline `rayQueryInitialize` / `rayQueryProceed` working in a
      compute shader (verify with a simple "are we occluded?" test)
- [ ] Per-instance custom index for material lookup from hit shaders

---

## Phase 18 — Ray-Traced Shadows

Replace cascaded shadow maps for the high preset. CSM stays as the
baseline preset.

- [ ] G-buffer pass produces world position + normal
- [ ] Compute pass: per pixel, trace a single shadow ray toward the sun;
      hit/miss writes shadow factor
- [ ] Spatial denoiser (a-trous or similar) — temporal denoising
      explicitly avoided to preserve image stability
- [ ] Soft shadows via cone sampling (multiple rays per pixel for the
      sun, budgeted)
- [ ] Toggle CSM vs RT shadows per quality preset

---

## Phase 19 — Ray-Traced Reflections

Hybrid SSR + RT fallback. SSR for in-screen reflections (cheap); RT
picks up where SSR fails (glancing angles, off-screen content).

- [ ] Screen-space reflection pass first (using existing g-buffer)
- [ ] Where SSR fails (no on-screen hit), trace an RT reflection ray
- [ ] RT hit shader samples lit color via material lookup + simple
      shading (or stored radiance if probes available)
- [ ] Roughness-aware ray spread (mirror reflections = single ray;
      rough surfaces = cone of rays or BRDF importance sampling)
- [ ] Spatial denoise; minimal temporal accumulation if any

---

## Phase 20 — Real-Time GI for Dynamic Objects

The dynamic side of hybrid GI. Static scenes use baked lightmaps from
Phase 16; dynamic objects (player, NPCs, particles) get RT-based GI
contribution.

- [ ] Probe-based GI: world-space probe grid stores incoming radiance
- [ ] Probes update incrementally — N probes per frame, M rays per
      probe, round-robin
- [ ] Dynamic objects sample interpolated probe data in PBR shader
- [ ] Per-object screen-space GI ray budget for higher-quality coverage
      on hero objects
- [ ] Spatial denoising on probe data; minimal temporal blending
- [ ] DDGI or ReSTIR-style sampling if budget allows (research phase)

---

## Beyond

Not planned in detail. Tackle when the foundation is solid:

- Skeletal animation
- Audio (kira or rodio)
- Terrain rendering
- Vegetation and foliage
- Particle systems
- Post-processing (bloom, SSAO alternatives, motion blur if wanted, color grading)
- Render-thread / game-thread split
- Job system for embarrassingly parallel work (skinning, particles, frustum culling)
- Asset cooking pipeline (glTF → optimized engine-native binary)
- Forward+ vs deferred decision (revisit with real scene data)
- HDR10 output
- Networking (way down the road)

---

## Principles

1. **One thing at a time.** Each phase builds on the last. Don't skip ahead.
2. **Run it, see it.** Every session should end with something visible.
3. **Understand before moving on.** If I can't explain what a uniform buffer does after Phase 1, stay in Phase 1.
4. **Split crates when it hurts, not before.** 800 lines doing three things? Split.
5. **No dead code.** If it doesn't run, it doesn't exist.
6. **Small commits, clear messages.** If it compiles and does something new, commit it.
