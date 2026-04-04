# Praxis Roadmap

Step-by-step plan for building a 3D engine in Rust.
Each milestone ends with something visible and runnable.

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

## Phase 1 — Window & Triangle

Open a window and render a colored triangle.

- [ ] Set up cargo workspace with a single binary crate
- [ ] Create a window with winit, handle close/resize events
- [ ] Initialize wgpu: instance, adapter, device, surface
- [ ] Write a basic vertex + fragment shader (WGSL)
- [ ] Create a render pipeline and draw a hardcoded triangle
- [ ] Handle window resize (recreate surface config)

References: Learn Wgpu (https://sotrh.github.io/learn-wgpu/) chapters 1-3, wgpu repo examples.

---

## Phase 2 — 3D Fundamentals

Render a rotating 3D cube with a movable camera.

- [ ] Add glam for math (vectors, matrices, quaternions)
- [ ] Implement a perspective camera (view + projection matrices)
- [ ] Create vertex buffers and index buffers for a cube
- [ ] Pass MVP matrix to the shader via a uniform buffer
- [ ] Add basic input handling (keyboard/mouse via winit)
- [ ] Implement a fly camera or orbit camera
- [ ] Add depth buffer

---

## Phase 3 — Textures & Mesh Loading

Load a textured glTF model and display it.

- [ ] Load an image and create a wgpu texture + sampler
- [ ] Sample the texture in the fragment shader (UV coordinates)
- [ ] Integrate the `gltf` crate to parse .glb files
- [ ] Extract vertex positions, normals, UVs, and indices from glTF
- [ ] Render a loaded mesh with its base color texture
- [ ] Support multiple meshes in a single file

Good test models: glTF sample models repo (Box, DamagedHelmet, Sponza).

---

## Phase 4 — Basic Lighting (Phong/Blinn-Phong)

A lit scene with at least one light source.

- [ ] Pass surface normals to the fragment shader
- [ ] Implement directional light in the shader
- [ ] Add ambient + diffuse + specular components
- [ ] Support point lights (with attenuation)
- [ ] Pass light data via uniform buffers
- [ ] Multiple objects in the scene with different positions

---

## Phase 5 — Build the ECS

Replace ad-hoc game objects with a proper ECS. Done here instead of
earlier because by now the data that flows through the engine is clear.

- [ ] Design the core: Entity (ID), Component (data), System (logic)
- [ ] Implement a sparse set or archetype storage for components
- [ ] Basic query API: "give me all entities with Transform + Mesh"
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

## Phase 8 — Shadows

Dynamic shadows from at least directional lights.

- [ ] Shadow mapping: render depth from light's perspective
- [ ] Sample shadow map in the main pass
- [ ] PCF or similar filtering to soften shadow edges
- [ ] Cascaded shadow maps for large outdoor scenes
- [ ] Debug visualization of shadow cascades

---

## Phase 9 — Debug UI & In-Game Tools

An in-game overlay for tweaking and debugging.

- [ ] Integrate egui via egui-wgpu
- [ ] FPS counter, frame time graph
- [ ] Entity inspector (select entity, view/edit components)
- [ ] Light controls (position, color, intensity sliders)
- [ ] Render mode toggles (wireframe, normals, depth)
- [ ] Console for runtime commands

---

## Phase 10 — Deferred Rendering (Optional)

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
