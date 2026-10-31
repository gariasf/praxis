# Praxis

A 3D engine in Rust for large open-world games, built on data shapes that a GPU-driven renderer can consume directly.

## Language

### Scene content

**Renderable**:
An entity that has a Transform and a Mesh Handle, and optionally a Material Handle that overrides the Materials of its Mesh's Primitives. The renderer pulls Renderables; entities never render themselves.
_Avoid_: Drawable, object, model

**Mesh**:
An ordered list of Primitives that are rendered together.
_Avoid_: Model, geometry

**Primitive**:
One contiguous range of a Mesh's vertices and indices.
_Avoid_: Submesh, surface

**Material**:
The appearance of a Primitive: PBR factors plus one Layer per Channel.
_Avoid_: Shader, texture set, bind group

**Channel**:
One texture role of a Material: albedo, normal, metallic-roughness, occlusion or emissive. Albedo and emissive are the colour Channels, encoded sRGB; normal, metallic-roughness and occlusion are the data Channels, encoded linear. Each kind has its own sampler.
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

**Arena**:
A GPU buffer of fixed capacity, created at startup and never grown, from which a Pool sub-allocates ranges; the Mesh pool's hold vertices, indices, and Meshlets with their vertices and triangles.
_Avoid_: Mega-buffer, heap

**Material ID**:
The number that identifies a Material to the GPU. A Primitive's Material has one; an Instance carries one only when its Renderable overrides its Mesh's Materials.
_Avoid_: Material index

### Assets

**Asset**:
A unit the cooker produces and the runtime loads: a Mesh, Material, Texture or Chunk.
_Avoid_: Resource, file

**Asset ID**:
The stable name of an Asset, derived from its source file's GUID, or a procedural Reference scene's name. It survives moving and editing the source, except an edit that renames the Asset inside it, gives another item its name, or moves it while its name is missing or shared (the glTF index then names it).
_Avoid_: Asset path, GUID (the GUID names a source file)

**Blob**:
One cooked Asset's bytes, named by their content hash.
_Avoid_: Package, cooked file

**Cook key**:
The hash of everything that determines a Blob: what it cooks from, dependencies, settings and cooker versions. A changed Cook key means the Asset is cooked again.
_Avoid_: Cache key, version

**Cooked cache**:
The store of Blobs plus the index from Asset ID to Blob. Disposable: it can always be rebuilt from sources.
_Avoid_: Build output, derived data

**Placement**:
One Mesh Asset positioned inside a Chunk.
_Avoid_: Spawn, instance (an Instance is a Renderable's data on the GPU)

### Frame

**Instance**:
One Renderable's data on the GPU: its World transform, as an Anchor plus an f32 offset, its Mesh, and the Material ID that overrides its Mesh's Materials, if any, in a slot of the Instance buffer that the Renderable keeps while it is one.
_Avoid_: Object

**Moving Instance**:
An Instance whose entity the fixed schedule moves, so the entity carries its previous World transform. It is rewritten each frame and rebuilt into the dynamic TLAS; every other Instance is static.
_Avoid_: Dynamic Instance, movable

**Bend**:
The GPU's stand-in for skinning until M7: each frame a compute pass rewrites a Renderable's own Mesh, the bent Mesh, from a rest Mesh of the same vertices, its middle curved into an arc by a curvature the fixed schedule sets each Tick.
_Avoid_: Deformation, skinning

**Shadow proxy**:
A Mesh that rays trace and no pass draws, standing in for part of a Renderable whose own BLAS would not hold the shape it is drawn in, such as a bent figure's bone.
_Avoid_: Collider, impostor

**Frame data**:
The per-frame collection of changed Instances, lights, camera and the Meshes the GPU bends that the renderer consumes in place of reading the ECS World. It lists no draws: the GPU culls Instances and builds its own.
_Avoid_: Scene data, render state

**Froxel**:
A cell of the view frustum, bounded in screen space and depth, that holds the list of lights reaching it.
_Avoid_: Cluster, light cluster, tile

**Shadow mask**:
A bit per light that casts a shadow at each pixel, set where that light is hidden from the surface the pixel's nearest sample shows, by the surface itself where its interpolated normal turns from that light or by what its shadow ray finds, in the order the lit pass sums the pixel's lights, up to 59 of them; point lights that cast none follow them in the light buffer and take no bit. Where the pixel's other samples show another surface, a second set of bits holds the farthest one's, and each fragment reads the set of the surface it shades. Where a directional light's bit at the nearest surface differs from a 4-neighbour's, a shadow edge, or where the nearest sample turns from that light while another of the surface's samples faces it, that light is traced again from each other sample that shows the surface, and the fragment shades it by the share of its samples that see it. Where the nearest sample turns from a point light that another of the surface's samples faces, the surface's bit is that sample's ray's verdict. A compute pass traces it from the depth target and the Surface normals; the lit pass reads it.
_Avoid_: Shadow map, shadow buffer, visibility buffer

**Surface normals**:
Two normals per sample of the depth target, its triangle's own and the one interpolated over the triangle, which the depth passes write with shadow rays on and the shadow mask pass reads. Where they disagree near a light's terminator, the triangle can turn from a light the interpolated normal faces.
_Avoid_: G-buffer, normal buffer

**Meshlet**:
A small group of one Primitive's triangles, bounded in vertex and triangle count, that the GPU draws as a unit and can cull as one.
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

**Anchor**:
The integer coordinates of the 64 m grid cell that holds a World position. The GPU receives the position as its Anchor plus an f32 offset from the cell's corner, never as one f32 position.
_Avoid_: Origin

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

**Render switches**:
The variants a Renderer is created with and keeps: whether Instances are culled, whether lights are culled into Froxels or all reach every pixel, whether shadow rays are traced, whether a pass counts each lit pixel's point lights, and the form the `crowd` scene's deforming meshes take in the TLAS. Their defaults are what ships; scene content such as ambient and background is not a switch.
_Avoid_: Render settings, config, options, debug flags

**Frame stats**:
What one frame cost, measured by the engine itself: CPU time per system and stage, GPU time per pass, of the dynamic TLAS's build and of the BLAS refit, the GPU frame total (that build, counted from the end of the frame before's last pass, which hides the rest of it, plus, for each submission of the frame's passes, the span from the earliest start of its passes to their latest end), ADR 0019's gate (from the start of culling phase one to the latest end of the passes up to the lit pass), draw calls, Instances and triangles drawn, Instances the draw had no room for, Instances written, Renderables refused, lights the light buffer had no room for, lights the Froxel lists had no room for and lights the shadow mask had no bit for, the lit pixels and the point lights their Froxels list and whose range reaches them (where counted), static Instances left out of the static TLAS and Moving Instances left out of the dynamic TLAS, bends the bend pass had no room for, bytes uploaded, heap allocations, GPU memory, Arena bytes in use, dropped time and the readbacks of GPU stats it skipped.
_Avoid_: Metrics, telemetry

**Baseline**:
The recorded measurements of a Reference scene on one adapter and backend: the reference machine, where Budgets are judged against it; CI's lavapipe, where only its counters are judged; or the RTX 2070 SUPER, the hardware floor benched for comparison, on Vulkan and on DX12.
_Avoid_: Benchmark result

**Bench record**:
One bench run's measurements of one Reference scene, with where and when it ran: the source it measured, the adapter, backend and driver, the harness version, and whether the machine was quiet. A Baseline is a Bench record chosen to judge changes against.
_Avoid_: Benchmark result, sample, data point

**Bench history**:
Every Bench record kept, from sessions, CI and the machines' own runs, including those kept off a Series.
_Avoid_: Metrics database, telemetry

**Series**:
One metric's Bench records for one Reference scene on one adapter and backend, taken on `main`'s source by a quiet machine, in source order. What its charts draw and what a Regression is found in.
_Avoid_: Trend, timeline

**Harness version**:
A number that names what the bench's metrics mean. It rises whenever a change alters how a metric is measured, so that two Bench records compare only when they share it.
_Avoid_: Schema version, bench version

**Experiment**:
Bench records taken from a branch under a name, kept beside the Series and compared with them, never part of one.
_Avoid_: Branch run, trial

**Budget**:
A numeric limit on one metric of a Reference scene (CPU ms, GPU ms, GPU memory, Input latency) that a change must not regress. The GPU memory Budget splits into one Budget per kind of GPU memory, such as the geometry Arenas, the texture pool and staging, which caps what that kind may hold (ADR 0017). Counters such as draw calls and bytes uploaded are held to their Baselines instead.
_Avoid_: Target, threshold

**Capability floor**:
The GPU features and limits the engine requires to start. An adapter below it is refused with every gap named; nothing in it has a fallback.
_Avoid_: Minimum spec, requirements, baseline
