# Praxis Roadmap

Step-by-step plan for building a 3D engine in Rust.
Each milestone ends with something visible and runnable.

**Target shape:** a big-world, lore-heavy RPG (Skyrim-style). This anchors which
rendering features are core (streaming, LOD, shadow cascades, scene hierarchy,
long view distances) vs. polish (HDR10, upscaling, ray tracing). The engine may
grow to cover persistence, scripting, and AI later; the foundation is rendering.

## Tech Stack

- **Language:** Rust
- **Graphics:** wgpu (Vulkan on Windows/Linux, Metal on macOS)
- **Windowing:** winit
- **Math:** glam
- **ECS:** Custom (built as part of the project)
- **Asset format:** glTF (via `gltf` crate)
- **Debug UI:** egui + egui-wgpu (later)
- **Logging:** tracing + tracing-subscriber

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

## Phase 5 — Build the ECS

Replace ad-hoc game objects with a proper ECS. Done here instead of
earlier because by now the data that flows through the engine is clear.

- [ ] Design the core: Entity (ID), Component (data), System (logic)
- [ ] Implement a sparse set or archetype storage for components
- [ ] Basic query API: "give me all entities with Transform + Mesh"
- [ ] Central resource pools: MeshPool, TexturePool, MaterialPool — hand out handles
- [ ] Components hold handles (MeshHandle, MaterialHandle), not owned GPU resources
- [ ] Keep component data in authoring form (`glam::Affine3A`, etc.), not GPU bytes
- [ ] Refactor the renderer to query ECS for renderable entities
- [ ] Refactor input/camera to be systems
- [ ] Add/remove entities at runtime

---

## Phase 6 — PBR Materials

Physically-based rendering with metallic-roughness workflow.

- [ ] Read PBR material properties from glTF (metallic, roughness, etc.)
- [ ] Implement Cook-Torrance BRDF in the fragment shader
- [ ] Support albedo, normal, metallic-roughness, AO, and emissive maps
- [ ] Add an environment map / IBL for ambient lighting
- [ ] HDR rendering with tone mapping (ACES or similar)
- [ ] Gamma correction

References: LearnOpenGL PBR chapters, Google Filament design doc.

---

## Phase 7 — Scene Graph & Transforms

Parent-child relationships and a proper scene hierarchy.

- [ ] Implement transform hierarchy (local + world transforms)
- [ ] Parent-child entity relationships in the ECS
- [ ] Propagate transforms down the hierarchy
- [ ] Scene serialization (load/save scenes from files, RON or JSON)

---

## Phase 8 — Streaming & World Chunks

A big-world RPG can't fit in memory. Load and unload the world as the player
moves.

- [ ] Divide the world into chunks (fixed-size cells, like Skyrim's exterior cells)
- [ ] Async load meshes/textures from disk without stalling the render loop
- [ ] Stream in a radius around the camera, stream out beyond
- [ ] Smooth transition across chunk boundaries (no pop-in, no frame hitch)
- [ ] Persistent entity state across load/unload (edits to the world survive)
- [ ] Debug visualization of loaded/unloaded chunks

---

## Phase 9 — Shadows

Dynamic shadows from at least directional lights.

- [ ] Shadow mapping: render depth from light's perspective
- [ ] Sample shadow map in the main pass
- [ ] PCF or similar filtering to soften shadow edges
- [ ] Cascaded shadow maps for large outdoor scenes
- [ ] Debug visualization of shadow cascades

---

## Phase 10 — LOD & Imposters

Big view distances need cheaper versions of far-away geometry.

- [ ] Discrete mesh LODs (glTF supports via extras or separate meshes)
- [ ] Distance-based LOD selection per renderable
- [ ] Imposters for distant vegetation / props (billboards with baked lighting)
- [ ] LOD bias and smooth crossfade to hide pops
- [ ] Per-chunk LOD (whole chunks degrade at distance, not just individual objects)

---

## Phase 11 — Debug UI & In-Game Tools

An in-game overlay for tweaking and debugging.

- [ ] Integrate egui via egui-wgpu
- [ ] FPS counter, frame time graph
- [ ] Entity inspector (select entity, view/edit components)
- [ ] Light controls (position, color, intensity sliders)
- [ ] Render mode toggles (wireframe, normals, depth)
- [ ] Console for runtime commands

---

## Phase 12 — Deferred Rendering (Optional)

Switch from forward to deferred rendering for many lights.

- [ ] Render to G-buffer (albedo, normals, depth, PBR params)
- [ ] Lighting pass reads G-buffer, computes lighting in screen space
- [ ] Support dozens/hundreds of point lights efficiently
- [ ] Forward pass for transparent objects

---

## Beyond

Not planned in detail. Tackle when the foundation is solid:

- Frustum culling and occlusion culling
- LOD (level of detail)
- Skeletal animation
- Audio (kira or rodio)
- Terrain rendering
- Vegetation and foliage
- Particle systems
- Post-processing (bloom, SSAO, motion blur, TAA)
- Multithreaded job system
- Asset cooking pipeline (glTF -> optimized binary)
- Networking (way down the road)

---

## Principles

1. **One thing at a time.** Each phase builds on the last. Don't skip ahead.
2. **Run it, see it.** Every session should end with something visible.
3. **Understand before moving on.** If I can't explain what a uniform buffer does after Phase 1, stay in Phase 1.
4. **Split crates when it hurts, not before.** 800 lines doing three things? Split.
5. **No dead code.** If it doesn't run, it doesn't exist.
6. **Small commits, clear messages.** If it compiles and does something new, commit it.
