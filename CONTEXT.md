# Praxis

A 3D engine in Rust for large open-world games, built on data shapes that a GPU-driven renderer can consume directly.

## Language

### Scene content

**Renderable**:
An entity that has a Transform, a Mesh Handle, and a Material Handle. The renderer pulls Renderables; entities never render themselves.
_Avoid_: Drawable, object, model

**Mesh**:
An ordered list of Primitives that are rendered together.
_Avoid_: Model, geometry

**Primitive**:
One contiguous range of a Mesh's vertices and indices.
_Avoid_: Submesh, surface

**Material**:
The appearance of a Renderable: PBR factors plus one Layer per Channel.
_Avoid_: Shader, texture set, bind group

**Channel**:
One texture role of a Material: albedo, normal, metallic-roughness, occlusion or emissive.
_Avoid_: Map, slot

**Layer**:
One numbered texture within a Channel. Layer 0 of every Channel is a neutral default, so a Material without that Channel has no effect.
_Avoid_: Slice, texture index

**Resident tail**:
The smallest mips of a texture, which stay in GPU memory for as long as the texture is loaded, so a texture whose finer mips are not loaded shows blurred, never missing.
_Avoid_: Mip tail, packed mips

### Resources

**Pool**:
The central store of one kind of resource (Mesh pool, Material pool). It hands out Handles; components never own resources.
_Avoid_: Manager, cache, registry

**Handle**:
A typed index into a Pool, held by components in place of the resource.
_Avoid_: Pointer, resource ID

**Material ID**:
The number that identifies a Material to the GPU. Each Instance carries one.
_Avoid_: Material index

### Frame

**Instance**:
One Renderable's data for one frame: its Transform and Material ID.
_Avoid_: Object

**Frame data**:
The per-frame collection of visible Renderables, lights and camera that the renderer consumes in place of reading the ECS World.
_Avoid_: Scene data, render state

**Froxel**:
A cell of the view frustum, bounded in screen space and depth, that holds the list of lights reaching it.
_Avoid_: Cluster, light cluster, tile

**Meshlet**:
A small fixed-size group of a Mesh's triangles, culled as a unit.
_Avoid_: Cluster, chunk

**Input latency**:
The time from the engine receiving an input event to the present of the first frame whose simulation consumed that event. Time the event spent in the OS before the engine received it, and time in the display queue after present, are not counted.
_Avoid_: Input lag, motion-to-photon

### Worlds

**ECS World**:
The container of entities, components and resources. A code concept, never the playable space.
_Avoid_: World (unqualified), registry

**Game world**:
The playable space, divided into Chunks.
_Avoid_: Level, map, world (unqualified)

**Chunk**:
A fixed-size vertical column of the Game world, addressed by integer coordinates, that loads and unloads as a unit.
_Avoid_: Cell, tile, sector

**World position**:
A position in a Game world, in metres from its origin. Positions relative to the camera, a Chunk or a parent entity derive from it.
_Avoid_: Global position, absolute position

**Scene**:
A serialisable set of entities and their hierarchy that loads into the ECS World. Not the whole Game world.
_Avoid_: Level, prefab

### Verification

**Reference scene**:
A Scene paired with a fixed camera path and injected time, so that every run produces the same frames. Used for Goldens and Baselines.
_Avoid_: Test scene, benchmark

**Golden**:
A committed image that a Reference scene must reproduce within tolerance.
_Avoid_: Snapshot, screenshot

**Baseline**:
The recorded measurements of a Reference scene on the reference machine. Budgets are judged against it.
_Avoid_: Benchmark result

**Budget**:
A numeric limit on one metric of a Reference scene (CPU ms, GPU ms per pass, number of draw calls, bytes uploaded, GPU memory) that a change must not regress.
_Avoid_: Target, threshold
