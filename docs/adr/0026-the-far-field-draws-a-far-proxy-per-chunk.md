# The far field draws a Far proxy per Chunk

Between the Load radius and the View distance each Chunk draws one Far proxy in place of its Placements: a Mesh the cooker merges from the Chunk's static Placements, with a Material baked for it and, on a second Primitive, the Chunk's ground tiles under the ground's own Material. A Chunk swaps against its Far proxy at the Load radius, whole and hard; past the View distance only flat ground is drawn. Accepted 2026-11-14 ([decision](https://github.com/gariasf/praxis/issues/795#issuecomment-6874889765), items 1, 3, 8, 9 and 11; its bake as the [bake decision](https://github.com/gariasf/praxis/issues/804#issuecomment-6878696975) amends items 3 and 5, on the survey `docs/research/far-proxy-baking.md`). Implemented: the cook of a Far proxy's Mesh (#812). Not yet: its baked Material, its streaming, its draw and the swap.

## Decision

- **One Far proxy per Chunk.** Every Chunk outside the Load radius and within the View distance draws its Far proxy, one Instance, in place of its Placements. Under it the Instance slots, the static TLAS and the flat cull keep M4's capacities (`docs/audit/m5/scalability.md`).
- **What it holds.** A first Primitive of merged, simplified geometry under one Material baked for it, windows included, and a second Primitive of the Chunk's ground tiles, unsimplified, under the ground's own Material. Only a scene with a Far field cooks proxies; it names its ground's Mesh.
- **Its Mesh.** The cooker merges the Chunk's static Placements about the Chunk's centre on the ground, the proxy's pivot. It drops a Placement when every vertex of it lies within one pixel of one other Placement's surface, that Placement the Chunk's own or one of its eight neighbours', since a Placement belongs to the Chunk that holds its pivot. One pixel is its size at the Load radius at 1440p and a 45° field of view, 14.7 cm at 256 m. It welds what is left on position and simplifies it by edge collapse to one pixel of absolute error, with no triangle count asked [ref:meshoptimizer-0.25-simplify-error], and refuses a proxy of more than 5,832 triangles outside its ground: the walls and roofs of `world`'s densest Chunk, 324 buildings of 18.
- **Its LOD chain.** A Far proxy is a Mesh and the culling pass selects its LOD as any Instance's. Its chain is cut on the welded positions, each coarser LOD simplified from LOD 0 to twice the error of the one before, and ends at eight LODs or at one that keeps more than 85% of the one before. A Mesh's own chain cannot serve: a face's vertices carry its normal and its UVs, so every corner is an attribute seam the simplifier locks [ref:meshoptimizer-0.25-vertex-kinds], and a collapse on welded positions leaves faces that need vertices of their own, since the simplifier only picks among the vertices it is given [ref:meshoptimizer-0.25-simplify-original-vertices]. Each LOD's faces are therefore given flat normals and their own vertices, shared with a finer LOD where they are equal. Halving the triangles, as a Mesh's chain does, is not asked either: on `world`'s densest Chunk the first halving must break buildings, at 2.79 m of error, where a pixel at the View distance spans 1.15 m. That Chunk's chain is LOD 0 at 5,832 triangles and LOD 1 at 3,240, its roofs flat, 0.27 m off.
- **Its Blob.** One Mesh Blob per Chunk, under an Asset ID made of the scene and the Chunk's coordinates, cooked by `praxis cook <scene>` and by a dev launch on a miss. Its Cook key covers its Chunk's and its neighbours' Cook keys and the Cook keys of the Meshes they place.
- **The swap.** A Far proxy shows exactly when its Chunk is not drawn, so the swap is at the Load radius, a whole Chunk at once, with no fade ([prototype](https://github.com/gariasf/praxis/issues/800#issuecomment-6875787005)). A Late Chunk keeps its Far proxy until it loads: it shows coarse, never as a hole.
- **Which are Resident.** A Far proxy is requested when its Chunk comes within the View distance and released past the View distance plus one Chunk's edge. It stays Resident while its Chunk is drawn, so leaving a Chunk never waits on a read.
- **Past the View distance.** Nothing is drawn but flat ground reaching the horizon, one Resident quad.

## Considered options

- Group proxies, one Mesh for several Chunks: at most 219 proxies are drawn, and their LOD chains already cut the triangles drawn with distance.
- Octahedral impostors: no streamed scene has trees or clutter, and one Instance per far object would bring the culling fallback with it. They wait for vegetation.
- Every Chunk's proxy Resident for the whole Game world: `world`'s buildings as bare boxes pass the geometry Budget, and the figure grows with content.
- The source Materials reused with small parts dropped, or vertex colours: `world`'s windows are 8 by 11 pixels at the swap and would vanish there.
- Vertex clustering and pruning of small components under a cap of 3,863 triangles: measured on `world`'s densest Chunk, they move an outline by 12 and 41 pixels. The same cap with the error let rise puts every roofline 2 pixels low at a hard swap.
- A drop rule by bounding size, what shipped engines use: a threshold that drops a window also drops a free-standing part 11 pixels tall, which no bake can carry.
- Voxel remeshing on a newer `meshopt`: nothing measured calls for it.
- The ground left out of the proxy and drawn by the flat ground past the View distance: it lies in the plane of loaded Chunks' ground tiles and would have to be kept from drawing under them, Chunk by Chunk. The ground baked into the proxy's atlas costs 3.02 M texels a Chunk of one colour.

## Consequences

- ADR 0013's 2 km View distance is drawn this way, and the "merged far proxies" of ADR 0020 are these.
- ADR 0014 keeps cooked Meshes within 128 m of their pivot, and no pivot is within 128 m of a whole Chunk: its corners lie 181 m from its centre, and `world`'s farthest proxy vertex 190 m. A Far proxy's Mesh is exempt. Its Instance is never rotated, which is the term the 128 m bound was set by, and f32 resolves 0.015 mm between 128 and 256 m from the pivot (derived, not measured through the shader), against a quarter of a pixel of 37 mm at the Load radius, the nearest a Far proxy is drawn unless its Chunk is late.
- The Material table counts Far proxies' Materials apart, and a frame's loads are capped by count (ADR 0017).
- A streamed scene whose Chunk keeps more triangles than the cap at one pixel fails its cook, naming the count. The cap moves with the Arenas' fit test of the densest 2 km view, not per scene.
- Two Placements each within a pixel of the other's surface are both dropped, as the rule reads. `world` places none.
- A Chunk with nothing but ground cooks a proxy whose first Primitive is empty.
- A Far proxy's first Primitive has no UVs and names no Material until the bake gives it both.
