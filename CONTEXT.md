# Praxis

A 3D engine in Rust for large open-world games, built on data shapes that a GPU-driven renderer can consume directly.

## Language

### Scene content

**Renderable**:
An entity with a Transform, a mesh reference and a material reference. The renderer pulls Renderables from the ECS World; entities never draw themselves.
_Avoid_: Drawable, object, model

**Mesh**:
An ordered list of Primitives that are drawn together, stored in the Mesh pool.
_Avoid_: Model, geometry

**Primitive**:
One contiguous range of a Mesh's vertices and indices in the shared geometry buffers.
_Avoid_: Submesh, surface, draw

**Material**:
The surface description of a Renderable: PBR factors plus one Layer per Channel, stored as one entry in the material buffer and looked up by Material ID.
_Avoid_: Shader, texture set, bind group

**Channel**:
One texture role of a Material: albedo, normal, metallic-roughness, occlusion or emissive. Each Channel has its own texture array.
_Avoid_: Map, slot

**Layer**:
A position in a Channel's texture array. Layer 0 of every Channel is a neutral default, so a Material without that Channel samples a no-op.
_Avoid_: Slice, texture index

### Resources

**Pool**:
The central store of one kind of resource (Mesh pool, Material pool). It owns the GPU buffers and textures and hands out Handles; components never own GPU resources.
_Avoid_: Manager, cache, registry

**Handle**:
A typed index into a Pool, held by components in place of the resource. A plain integer while nothing is unloaded; generational once resources can be.
_Avoid_: Reference, pointer, resource ID

**Material ID**:
The integer form of a Material's Handle that each Instance carries, which the fragment shader uses to read the Material.
_Avoid_: Material index

### Frame

**Instance**:
One Renderable's data for one frame: a model matrix, a normal matrix and a Material ID, one row of the instance buffer. Every Primitive of the Renderable's Mesh is drawn with it.
_Avoid_: Object, draw

**Frame data**:
The per-frame snapshot (visible Renderables, lights, camera) that the renderer consumes instead of reading the ECS World. Planned; the renderer still queries the World today.
_Avoid_: Scene data, render state

### Worlds

**ECS World**:
The `bevy_ecs` container of entities, components and resources. A code concept, never the playable space.
_Avoid_: World (unqualified), registry

**Game world**:
The playable space, streamed in Chunks around the camera.
_Avoid_: Level, map, world (unqualified)

**Chunk**:
A fixed-size region of the Game world that is loaded and unloaded as a unit. Planned.
_Avoid_: Cell, tile, sector

**Scene**:
A serialisable set of entities and their hierarchy that loads into the ECS World. Not the whole Game world.
_Avoid_: Level, prefab

### Verification

**Reference scene**:
A deterministic scene with a fixed camera path, used for Goldens and Budgets.
_Avoid_: Test scene, benchmark

**Golden**:
A committed reference image that a Reference scene must reproduce within tolerance.
_Avoid_: Snapshot, screenshot

**Baseline**:
The recorded measurements of a Reference scene on the reference machine, which Budgets are judged against.
_Avoid_: Benchmark result

**Budget**:
A numeric limit on one metric of a Reference scene (CPU ms, GPU ms per pass, draw calls, bytes uploaded, VRAM) that a change must not regress.
_Avoid_: Target, threshold
