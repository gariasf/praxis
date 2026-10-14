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

**Texture**:
One Channel's image with its mips. A Layer refers to one Texture.
_Avoid_: Map, image

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

### Assets

**Asset**:
A unit the cooker produces and the runtime loads: a Mesh, Material, Texture or Chunk.
_Avoid_: Resource, file

**Asset ID**:
The stable name of an Asset, derived from its source file's GUID. It survives moving and editing the source, except an edit that renames the Asset inside it.
_Avoid_: Asset path, GUID (the GUID names a source file)

**Blob**:
One cooked Asset's bytes, named by their content hash.
_Avoid_: Package, cooked file

**Cook key**:
The hash of everything that determines a Blob: source bytes, dependencies, settings and cooker versions. A changed Cook key means the Asset is cooked again.
_Avoid_: Cache key, version

**Cooked cache**:
The store of Blobs plus the index from Asset ID to Blob. Disposable: it can always be rebuilt from sources.
_Avoid_: Build output, derived data

**Placement**:
One Mesh Asset positioned inside a Chunk.
_Avoid_: Spawn, instance (an Instance is a Renderable's data on the GPU)

### Frame

**Instance**:
One Renderable's data on the GPU: its World transform and Material ID, in a slot of the Instance buffer that the Renderable keeps while it is one.
_Avoid_: Object

**Frame data**:
The per-frame collection of draws, changed Instances, lights and camera that the renderer consumes in place of reading the ECS World.
_Avoid_: Scene data, render state

**Froxel**:
A cell of the view frustum, bounded in screen space and depth, that holds the list of lights reaching it.
_Avoid_: Cluster, light cluster, tile

**Meshlet**:
A small fixed-size group of a Mesh's triangles, culled as a unit.
_Avoid_: Cluster, chunk

**Temporal class**:
How a lighting term reaches its value over time: (a) complete each frame; (b) converging over frames in a world-space or probe-space cache, so it lags a change without smearing on screen; (c) reusing screen-space history (reprojection, temporal denoising).
_Avoid_: Temporal stability, temporal tier

**Interior volume**:
An authored region of a Game world marking a space as inside, so lighting sampled inside it ignores light gathered outside its walls, and the reverse.
_Avoid_: Room, portal, light blocker

**Input latency**:
The time from the engine receiving an input event to the present of the first frame that shows the state consuming that event at full blend weight. Time the event spent in the OS before the engine received it, and time in the display queue after present, are not counted.
_Avoid_: Input lag, motion-to-photon

**Tick**:
One fixed step of the simulation, 1/64 s; rendered frames draw a blend of the two newest Ticks.
_Avoid_: Physics frame, fixed update, step (unqualified)

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

**World transform**:
Where an entity sits in a Game world: its Transform composed with every ancestor's, in f64. Derived from the hierarchy, never authored.
_Avoid_: Global transform, absolute transform

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
The recorded measurements of a Reference scene on one adapter: the reference machine, where Budgets are judged against it, or CI's lavapipe, whose Baselines hold its counters only.
_Avoid_: Benchmark result

**Budget**:
A numeric limit on one metric of a Reference scene (CPU ms, GPU ms per pass, number of draw calls, bytes uploaded, GPU memory) that a change must not regress.
_Avoid_: Target, threshold

**Capability floor**:
The GPU features and limits the engine requires to start. An adapter below it is refused with every gap named; nothing in it has a fallback.
_Avoid_: Minimum spec, requirements, baseline
