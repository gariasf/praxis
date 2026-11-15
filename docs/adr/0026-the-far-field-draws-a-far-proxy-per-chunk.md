# The far field draws a Far proxy per Chunk

Between the Load radius and the View distance each Chunk draws one Far proxy in place of its Placements: a Mesh the cooker merges from the Chunk's static Placements, with a Material baked for it and, on a second Primitive, the Chunk's ground tiles under the ground's own Material. A Chunk swaps against its Far proxy at the Load radius, whole and hard; past the View distance only flat ground is drawn. Accepted 2026-11-14 ([decision](https://github.com/gariasf/praxis/issues/795#issuecomment-6874889765), items 1, 3, 8, 9 and 11; its bake as the [bake decision](https://github.com/gariasf/praxis/issues/804#issuecomment-6878696975) amends items 3 and 5, on the survey `docs/research/far-proxy-baking.md`). Implemented: the cook of a Far proxy's Mesh (#812), which also settled what the decisions left to the build: the LOD chain's cut, the pivot, and the three cases under *Settled by the cook*; and its atlas (#867), with what *Settled by the bake* lists. Not yet: the Material baked into the atlas, its streaming, its draw and the swap.

## Decision

- **One Far proxy per Chunk.** Every Chunk outside the Load radius and within the View distance draws its Far proxy, one Instance, in place of its Placements. Under it the Instance slots, the static TLAS and the flat cull keep M4's capacities (`docs/audit/m5/scalability.md`).
- **What it holds.** A first Primitive of merged, simplified geometry under one Material baked for it, windows included, and a second Primitive of the Chunk's ground tiles, unsimplified, under the ground's own Material. Only a scene with a Far field cooks proxies; it names its ground's Mesh.
- **Its Mesh.** The cooker merges the Chunk's static Placements and drops a Placement when every vertex of it lies within one pixel of one other Placement's surface, that Placement the Chunk's own or one of its eight neighbours', since a Placement belongs to the Chunk that holds its pivot. One pixel is its size at the Load radius at 1440p and a 45° field of view, 14.7 cm at 256 m. It welds what is left on position and simplifies it by edge collapse to one pixel of error, with no triangle count asked; the error is the simplifier's own measure, an approximation of the deviation [ref:meshoptimizer-0.25-simplify-error]. It refuses a proxy of more than 5,832 triangles outside its ground: the walls and roofs of `world`'s densest Chunk, 324 buildings of 18.
- **Its Blob.** One Mesh Blob per Chunk, under an Asset ID made of the scene and the Chunk's coordinates, cooked by `praxis cook <scene>` and by a dev launch on a miss. Its Cook key covers its Chunk's and its neighbours' Cook keys and the Cook keys of the Meshes they place.
- **The swap.** A Far proxy shows exactly when its Chunk is not drawn, so the swap is at the Load radius, a whole Chunk at once, with no fade; a swap the owner sees on the first cooked proxies reopens the fade ([prototype](https://github.com/gariasf/praxis/issues/800#issuecomment-6875787005), item 3). A Late Chunk keeps its Far proxy until it loads: it shows coarse, never as a hole. A Far proxy entering at the View distance appears the same way, whole and at once.
- **Which are Resident.** A Far proxy is requested when its Chunk comes within the View distance and released past the View distance plus one Chunk's edge. It stays Resident while its Chunk is drawn, so leaving a Chunk never waits on a read.
- **Past the View distance.** Nothing is drawn but flat ground reaching the horizon, one Resident quad.

## Settled by the cook

The decisions left these to the build of the cook; the owner accepts them with this ADR.

- **The LOD chain.** A Far proxy is a Mesh and the culling pass selects its LOD as any Instance's. Its chain is cut on the welded positions, each coarser LOD simplified from LOD 0 to twice the error limit of the one before, and ends at eight LODs or at one that keeps more than 85% of the one before. A Mesh's own chain cannot serve. A face's vertices carry its normal and its UVs, and the corners of a box built face by face are vertices the simplifier locks [ref:meshoptimizer-0.25-vertex-kinds]. A collapse on welded positions leaves faces that need vertices of their own, since the simplifier only picks among the vertices it is given [ref:meshoptimizer-0.25-simplify-original-vertices], so each LOD's faces get flat normals and their own vertices, shared with a finer LOD where they are equal. Halving the triangles, as a Mesh's chain does, is not asked either: on `world`'s densest Chunk the first halving must break buildings, at a reported error of 2.79 m, where a pixel at the View distance spans 1.15 m. That Chunk's chain is LOD 0 at 5,832 triangles and LOD 1 at 3,240, its roofs flat and 0.3 m lower, which the simplifier reports as an error of 0.27 m.
- **The pivot.** A proxy's pivot is its Chunk's centre on the ground, and the cooker refuses a proxy with a vertex more than a Chunk's edge from it. ADR 0014 keeps every other cooked Mesh within 128 m of its pivot, and no pivot is within 128 m of a whole Chunk: its corners lie 181 m from its centre, and `world`'s farthest proxy vertex 190 m. The 128 m bound was set by the error of an f32 rotation, and a proxy is cooked to be placed unrotated. f32 resolves 0.015 mm between 128 and 256 m (derived, not measured through the shader), against a quarter of a pixel of 37 mm at the Load radius, the nearest a Far proxy is drawn unless its Chunk is late.
- **The ground is no surface to drop against.** A Placement that lies within a pixel of a ground tile is kept: the ground is not baked, so nothing would carry what was dropped onto it.
- **Dropped against the dropped.** A Placement is dropped against any other Placement, one that is itself dropped included, so two Placements each within a pixel of the other's surface both go. `world` places neither case.
- **Open land.** A Chunk with nothing but ground cooks a proxy whose first Primitive is empty.

## Settled by the bake

The bake decision left these to the build of the bake.

- **A chart.** A planar face is the triangles of the first Primitive that share vertices, which carry a position and a face normal. Its chart is the least-area rectangle about it along one of its edges, a texel a pixel at the Load radius, its rows no longer than its columns. Charts are packed on shelves, tallest first, each with 4 texels of gutter a side on a grid of 8, into the smallest square atlas of a power-of-two side that holds them; a proxy whose charts pass 8,192 texels a side fails its cook. `world`'s densest Chunk charts into 4096².
- **Coarser LODs.** Each face of a coarser LOD takes the chart nearest it among those facing within 45° of it, its UVs held inside that chart. On `world`, LOD 1's roof tops read LOD 0's.

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
- ADR 0014's 128 m pivot bound exempts a Far proxy's Mesh, as *The pivot* above has it.
- The Material table counts Far proxies' Materials apart, and a frame's loads are capped by count (ADR 0017).
- A streamed scene whose Chunk keeps more triangles than the cap at one pixel fails its cook, naming the count.
- A LOD error is what the simplifier reports, which on `world`'s flattened roofs is a tenth under the distance they moved.
- A Far proxy's first Primitive names no Material until the bake gives it one.
