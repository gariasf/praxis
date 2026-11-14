# Baking a Far proxy's Mesh and Material: state of the art

Researched 2026-11-14 for ticket #805, under the M5 spec (#804) and the [far-field decision](https://github.com/gariasf/praxis/issues/795#issuecomment-6874889765); measured on `main` at `d4ac21eb`.

**Summary.** The bake comes down to six choices (the simplifier and its triangle cap, the rule that drops windows from the geometry, the material transfer, the charts, their gutters and the ground), and a measurement on `world`'s densest settlement Chunk separates the first two. At one pixel of error (14.7 cm) no `meshopt` 0.6.2 call removes a window or brings that Chunk under the decision's 3,863-triangle cap; at the cap `simplify_sloppy` moves outlines by up to 12 pixels and `Prune` deletes whole roofs, while `simplify` on position-welded vertices only flattens roof slabs, so the choice is the cap at the Chunk's 5,832 triangles with one pixel held, or 3,240 triangles with rooflines 2 pixels low. Only a transfer that reads the sources from outside along the proxy's normal (rays from a cage, or the sources rasterised into each chart with a depth test) returns a window standing 6 cm proud of its wall; nearest-point transfer returns the wall. One rectangle a face fits a 4096² atlas with gutters safe to mip 3 (79%) but not to mip 4, where the Resident tail starts (114%), and the ground's 3.0 M texels fit beside them only with no gutter at all, which every engine read avoids by giving distant ground a coarse texture of its own.

Sources: papers from 1992 to 2023 (Rossignac and Borrel, Cohen, Cignoni, Sander, Lévy, Carr and Hart, Ray, Liu, Gao, Chen, Knodt), GDC and SIGGRAPH course material (Frostbite 2007 and 2012, Far Cry 5, Ghost Recon Wildlands, id Software), NVIDIA's atlas white paper, meshoptimizer 0.25 and the `meshopt` 0.6.2 crate, Unreal Engine 5.8.4, Blender 5.2.3, Simplygon 10.4, Unity's HLODSystem and xatlas. Each claim relied on is in `docs/references.toml`, cited by key. Ruled out by the ticket and not surveyed: impostors, group proxies, a normal or occlusion Channel.

## Impact

- **#812, the proxy's Mesh.** Its criterion, under the cap and within 14.7 cm, cannot hold at 3,863 triangles on the densest Chunk. `simplify_sloppy` and `Prune`, which the decision's item 5 and #804 name, give way to a drop rule and `simplify` on welded positions in every option but E1. Item 6 stands: nothing measured calls for `meshopt` 0.7.0.
- **#813, the proxy's Material.** Its test "the source surface nearest them" would pass a bake that loses the windows. Its test "down to the Resident tail's mips" cannot pass at 14.7 cm a texel on the densest Chunk with one rectangle a face. The bake must read Placements of neighbouring Chunks.
- **The decision's derived figures**, replaced by measured ones: 18 triangles a building, not 20; up to 324 buildings a Chunk, not about 312; 191,029 m² of walls and roofs, not 175,800; 8.81 M texels, not 8.1 M.
- **Budgets and the runtime.** No option adds a wgpu feature or runtime code, and none reads history (ADR 0008): the bake is offline and the proxy samples ordinary Textures. The texture pool figure in the decision stands, one 4096² atlas a settlement Chunk. Only A2 needs a new dependency.

## Where Praxis stands

`world` places 16,214,556 static Placements in 1,600 Chunks. Its densest Chunk, (−19, −3), holds 29,941: 4 ground tiles, 324 buildings (a walls Mesh and a roof Mesh each) and 29,289 windows; 227 Chunks hold 300 buildings or more. A building's walls are 8 triangles and its roof slab 10, so the Chunk's walls and roofs, its *shell*, are 5,832 triangles, and its windows, 10 triangles each, are 292,890 of the merged source's 298,722 (98%). The shell's area is 191,029 m² (walls 162,195, roofs 28,834), 8.81 M texels at 14.7 cm; the ground's 65,536 m² is 3.02 M more.

A window stands 6 cm proud of its wall, 0.4 pixels at 256 m, and the wall runs unbroken behind it, clad in a Texture of 512 texels a metre that repeats every 4 m: a bake texel spans 75 of its texels a side. A Placement belongs to the Chunk that holds its pivot, so 645 of the 29,886 windows on this Chunk's walls are held by a neighbour, and 48 of the windows it holds stand on a neighbour's wall.

The cooker creates no GPU device. Albedo cooks to BC7 and metallic-roughness to BC5, a byte a texel as the decision assumed, up to 8,192 texels a side, and a 4096² Texture's Resident tail starts at its 256² mip, mip 4 (`resident_tail_level`). A texel a pixel at 256 m reaches mip 3 at 2,048 m, past the View distance.

## Comparison

| Technique | Buys | Cost (published, or measured here) | Maturity | Fit: `world`, ADRs 0008 and 0013, wgpu 30 | Licence of reference code |
|---|---|---|---|---|---|
| 1. Rays from the proxy along its normal | A source point on any proxy; the outer layer when cast from outside | No time published; "two orders of magnitude faster" than optimisation | Shipped: Blender, Unreal ProxyLOD | Keeps windows from a cage; one ray a texel aliases the brick | GPL and Apache-2.0 (Blender), Epic EULA: read only |
| 2. Nearest-point resampling | A source point for every texel, no misses | 15 s, 128 s at 3 × 3 samples (SGI O2, 1999) | Research tool | Returns the wall behind each window (derived) | None found |
| 3. Sources rasterised or captured into the atlas | The outer layer by depth; no search on planar charts | None published | Shipped: Unreal Approximated Mesh | Fits planar faces; a CPU rasteriser, as the cooker has no GPU | Epic EULA: read only |
| 4. Reuse of the sources' charts | No resampling | None published | Shipped: Unreal merged mesh, Unity SimpleBatcher, Simplygon aggregator | Ruled out: windows keep texels only as geometry | Unity Companion: read only |
| 5. Optimisation by differentiable rendering | Texture, UVs and shape fitted to images | 607 ms × 4,504 iterations (Titan XP); hours (V100) | Research | None: a GPU, and 46 minutes (derived) to hours an asset | Noncommercial (nvdiffmodeling) |
| 6. A rectangle per planar face, rectangle packing | Packing at 0.82 to 0.91 | Measured: 54% of 4096² unpadded | xatlas's planar charts; Liu 2019 | `world`'s shell is 2,916 planar quads | Public domain by its readme (RectangleBinPack) |
| 7. General charting and packing | Any Mesh | Packing under 1 s (2002); 0.55 to 0.60 efficiency | Open implementation: xatlas | A new dependency; needed for non-planar proxies | MIT (xatlas) |
| 8. Gutter on a power-of-two grid, sized for a mip | No bleed down to that mip | Measured: 64% to mip 2, 79% to mip 3, 114% to mip 4 | NVIDIA 2004; id Tech 5 borders | Mip 3 covers the View distance; the tail's mip 4 fits at half density | n/a |
| 9. Small gutter and dilation | Coarse mips blend plausible colours | 4 texels; 10 pixels of dilation | Shipped: Unreal, Simplygon | Bleed below the gutter's mip is accepted | n/a |
| 10. Tiled ground kept at distance | No texels | None | None found | Today's ground is one colour; coplanar with loaded ground tiles | n/a |
| 11. Ground baked to a coarse texture of its own | No tiling far off, small | 132 × 132 BC1 a 64 m tile (Far Cry 5) | Shipped: Unreal, Far Cry 5, Wildlands, Unity | Terrain is out of M5 | Epic EULA, Unity Companion: read only |
| 12. Ground baked into the proxy's atlas | One Material | Measured: 3.02 M texels a Chunk | None found | Fits the densest Chunk only with no gutter | n/a |
| 13. Vertex clustering (`simplify_sloppy`) | Merges disjoint parts; reaches any count | Measured: 12 px at the cap | meshoptimizer, experimental; gltfpack's fallback | Finest cell 25 cm on a 256 m Chunk | MIT |
| 14. Edge collapse (`simplify`) | Error-bounded simplification | Measured: 4 px at the cap, on welded positions | Shipped: gltfpack | Per-face vertices lock; welded ones collapse | MIT |
| 15. `Prune` | Drops isolated components by radius | Measured: 41 px at the cap | gltfpack's default | Drops nothing at one pixel | MIT |
| 16. Small parts removed by size | Fewer parts before simplifying | None published | Shipped: Unreal, Simplygon, Unity | A window is 11 pixels: a pixel-sized threshold keeps it | Epic EULA, Unity Companion: read only |
| 17. Remeshing | Closes gaps, absorbs surface detail | 45 s a model (Gao), about 7 min (Chen), on 32 cores | Shipped: Simplygon, Unreal; research | `meshopt` 0.7.0 (declined) or a new dependency; not called for | Executable only (Gao) |

## Material transfer

**1. Rays from the proxy along its normal.** The usual bake "casts a ray for each surface texel of the low-poly along its normal direction" and copies the attribute at the hit, two orders of magnitude faster than the optimisation methods compared with it [ref:knodt-2023-joint-uv-texture-baking]. Blender casts inward from the low-poly or from a cage, without which it "produces glitches on the edges" [ref:blender-5.2-selected-to-active]. Unreal's ProxyLOD fires rays from texel centres [ref:ue-5.8-proxylod-ray-correspondence], forward and backward along the normal, and keeps the closer hit [ref:ue-5.8-proxylod-hit-selection], with a supersampling constant of 16 and rays ended after six voxels [ref:ue-5.8-proxylod-supersampling]. Antialiasing is by supersampling alone: Cycles jitters inside the texel but cannot move to a neighbouring triangle [ref:blender-5.2.3-bake-jitter]. On `world` (derived, nothing baked) a ray started outside the glass and cast inward returns the window, and a ray started on the wall's plane and cast both ways finds the wall at distance zero.

**2. Nearest-point resampling.** Cignoni et al. resample a detail texture by taking, for each sample of the simplified surface, the source point "at a minimal euclidean distance"; they report aliasing where the mesh has "a very thin solid section" and say that for an impostor "a standard projective technique is more adequate" [ref:cignoni-1999-resampling-detail-textures]. Their tool averages regular oversamples, 128 s against 15 s for 3 × 3 at its finest step on an SGI O2 [ref:cignoni-1999-oversampling]. On `world` (derived) the wall is the nearest surface to every texel of a proxy face lying in its plane, so the windows are lost.

**3. Sources rasterised or captured into the atlas.** Cohen et al. store colour in a map by scan-converting each source triangle at its texture coordinates and leave filtering to run-time mip-mapping [ref:cohen-1998-appearance-preserving]. Unreal's Approximated Mesh layer [ref:ue-5.8-hlod-layer-types] renders the sources from several directions and reprojects: each texel takes the photo pixel that projects onto its position and normal, the photo chosen by the dot product of view and normal [ref:ue-5.8-approximate-photo-set-sample]. Its captures default to 2,048 pixels with supersampling off [ref:ue-5.8-approximate-bake-settings], and Unreal can size a proxy's texture from the World Partition grid's loading range [ref:ue-5.8-material-proxy-texture-sizing], as Praxis's texel a pixel at the Load radius does. Simplygon records per atlas texel the source triangle and barycentric coordinate [ref:simplygon-10.4-mapping-image] and, behind its remesher, accumulates colour through layers by opacity [ref:simplygon-10.4-caster-layers]. For a planar chart, rasterising the sources along the chart's normal with a depth test is technique 1's inward ray for every sample at once (derived): the outermost surface wins, and n × n samples a texel average the brick.

**4. Reuse of the sources' charts.** Unreal's merged mesh generates unique UVs and bakes every material through them [ref:ue-5.8-merge-unique-uv-bake]; Unity's SimpleBatcher combines the source textures and remaps UVs [ref:unity-hlodsystem-04be7e8-simple-batcher]; Simplygon's aggregator repositions the input charts unchanged, and names tiled textures and a building whose windows are instances as its case [ref:simplygon-10.4-aggregation-tiled]. All three keep the sources' triangles, so a window has texels only while its geometry stays, the form the decision rejected.

**5. Optimisation, and research since 2019.** Knodt et al. optimise UVs and texture jointly by differentiable rendering, 607 ms an iteration over 4,504 iterations at 512² on a Titan XP [ref:knodt-2023-joint-uv-texture-baking]; Hasselgren et al. fit shape and material together in hours on a V100 [ref:hasselgren-2021-appearance-driven-simplification], with noncommercial code [ref:nvdiffmodeling-licence]. The low-poly meshing papers produce geometry and leave "UV-generation and texture baking" to follow on the result [ref:chen-2023-robust-low-poly]. No paper since 2019 was found that replaces techniques 1 to 3 in a CPU cook.

## Atlas charting and packing

**6. A rectangle per planar face.** Sander et al. enclose each chart in its least-area rectangle and pack the rectangles at 0.82 to 0.91 efficiency [ref:sander-2001-tmpm-packing-efficiency]. Liu et al. refine charts towards axis-aligned ones because rectangle packing "is easier to achieve high packing efficiency" [ref:liu-2019-bounded-packing-efficiency], and xatlas precomputes regions of coplanar faces and makes charts of them [ref:xatlas-f700c77-planar-charts]. Jylänki's survey of rectangle packers finds MAXRECTS best and shelf packers worth choosing only for simplicity [ref:jylanki-2010-maxrects]. `world`'s shell is 2,916 planar quads; one rectangle a quad fills 54% of a 4096² atlas before gutters.

**7. General charting and packing.** LSCM, the base of automatic atlases, packs chart outlines rather than their bounding rectangles, at ratios of 0.55 to 0.60 against 0.37 to 0.51, in under a second [ref:levy-2002-lscm-tetris-packing]; an atlas of packed complex polygons covered under half its texels in Carr and Hart's tests [ref:carr-2002-mip-mappable-atlas]. xatlas is the open implementation, with options for padding, room for bilinear taps, alignment to 4 × 4 blocks and brute-force placement [ref:xatlas-f700c77-pack-options]. It would be a new dependency, needed once a proxy's faces are not planar.

**8. Gutters sized for a mip.** Packed charts bleed in coarse mips, since "coarser levels will average together spatially disjoint charts" [ref:sander-2001-tmpm-gutter-pull-push], and bilinear filtering of every lower mip reads neighbouring texels [ref:nvidia-2004-atlas-lower-mip-bleed]. Box-filtered mips stay clean only while sub-textures are powers of two that do not "cross power-of-two lines" [ref:nvidia-2004-atlas-power-of-two-lines], and a border halves with each mip, 4 texels becoming 2, a 4-texel border matching a compression block [ref:vanwaveren-2012-svt-border-per-mip]. So a chart on a 2^N grid with 2^(N−1) texels of gutter a side is clean under bilinear filtering at mip N (derived from those two; no source states it so). Ray et al. keep "the three or four finest levels" continuous and argue that coarser ones are seen from far [ref:ray-2010-invisible-seams-mip-levels]; Carr and Hart's quadtree of rectangular charts stays coherent at every level [ref:carr-2002-mip-mappable-atlas]. Measured on the densest Chunk: the padded charts take 64% of the atlas to mip 2, 79% to mip 3 and 114% to mip 4.

**9. Small gutter and dilation.** Shipped tools default to a 4-texel gutter (Unreal's proxy material [ref:ue-5.8-material-proxy-defaults], Simplygon [ref:simplygon-10.4-gutter-space], id Tech 5's pages [ref:vanwaveren-2012-svt-border]) and fill the empty texels: Simplygon dilates 10 pixels and then fills the rest [ref:simplygon-10.4-dilation], Sander et al. pull-push [ref:sander-2001-tmpm-gutter-pull-push]. Coarser mips then blend neighbouring charts. No source read clamps or corrects that at run time; NVIDIA calls its per-mip clamp "only theory" [ref:nvidia-2004-atlas-lower-mip-bleed].

## The ground

**10. Tiled ground kept at distance.** No engine read does this. Frostbite names as a common approach "low-resolution unique color maps" with tiled detail maps "blended in at certain distance to the camera" [ref:andersson-2007-colour-map-detail-maps], and Battlefield 3 found shader splatting "not scalable in view distance" [ref:widmark-2012-bf3-terrain-virtual-texture]. `world`'s ground is one Material with no Texture, so a ground left out of the proxies and drawn by the flat ground the decision already draws past the View distance costs no texel and shows no tiling yet. That ground lies in the plane of loaded Chunks' ground tiles, so it cannot draw under them.

**11. Ground baked to a coarse texture of its own.** Unreal bakes each landscape proxy's material into textures for its HLOD, in a builder of its own [ref:ue-5.8-landscape-hlod-bake]. Far Cry 5 gives each terrain tile, 64 m at the finest, a baked 132 × 132 BC1 albedo for far shading [ref:moore-2018-fc5-baked-albedo], about half a metre a texel (derived). Ghost Recon Wildlands renders distant patches with a "Vista" albedo [ref:werle-2017-wildlands-vista-texture], and Unity's TerrainHLOD bakes the splats into Albedo, Normal and Mask [ref:unity-hlodsystem-04be7e8-terrain-hlod].

**12. Ground in the proxy's atlas.** No engine read does this. A Chunk's ground is one square of 1,739 texels a side. Beside the densest Chunk's shell it fits a 4096² atlas only with no gutter at all (3,962 of 4,096 rows); at half the density shell and ground fit with gutters to mip 4 (56%).

## Geometry

**13. Vertex clustering (`simplify_sloppy`).** Rossignac and Borrel divide a box around the object into cells and replace the vertices in each by one, with an error bounded by a chosen accuracy and results that may turn a solid into a curve [ref:rossignac-1993-vertex-clustering]. meshoptimizer's sloppy simplifier is that algorithm, a quadric choosing each cell's vertex [ref:meshoptimizer-0.25-sloppy-output], and so it "can merge mesh features that are topologically disjoint but spatially close" [ref:meshoptimizer-0.25-simplify-sloppy]. Its grid stops at 1,024 points an axis and its error limit is clamped to 0.1% of the mesh's extent [ref:meshoptimizer-0.25-sloppy-grid]: on a 256 m Chunk the finest cell is 25 cm, so 14.7 cm cannot be asked of it. It takes no options, so not `Prune` either [ref:meshoptimizer-0.25-sloppy-declaration]. gltfpack calls it only when its aggressive mode's edge collapse is stuck, at 10% error [ref:meshoptimizer-0.25-gltfpack-simplify].

**14. Edge collapse (`simplify`, `simplify_with_attributes_and_locks`).** Garland and Heckbert contract arbitrary vertex pairs, which joins unconnected regions [ref:garland-1997-qem]; meshoptimizer joins vertices of equal position and collapses only edges of existing triangles [ref:meshoptimizer-0.25-edge-collapse-topology], so two buildings never merge. It classifies each vertex, and locks one where more than two attribute wedges meet or a seam does not close [ref:meshoptimizer-0.25-vertex-kinds]: every corner of a box built from per-face quads, as `world`'s Meshes and a merge of them are. The crate exposes six options, `Prune` and `Permissive` among them [ref:meshopt-0.6.2-simplify-options]. Measured: on per-face vertices neither call collapses an edge at any error unless `Permissive` unlocks the corners; on positions welded, attributes dropped, `simplify` reaches the cap. A proxy takes new UVs and flat normals in any case.

**15. `Prune`.** The flag removes a component, found over welded positions, once the error reached passes the radius of a sphere about its vertices' mean, smallest first [ref:meshoptimizer-0.25-prune-criterion]. A window's radius is 1.0 m (derived), so at 14.7 cm it removes nothing; under a count it removes whatever radius the count needs. gltfpack turns it on unless borders are locked [ref:meshoptimizer-0.25-gltfpack-simplify]. 0.25 also has a stand-alone `meshopt_simplifyPrune` with a radius cutoff [ref:meshoptimizer-0.25-simplify-prune-function], which the crate binds only in its `ffi` module.

**16. Small parts removed by size.** Shipped tools drop parts in a step of their own, by bounding size. Unreal's Approximated Mesh leaves out a part whose largest bounding-box dimension is under 0.05 times its accuracy [ref:ue-5.8-approximate-tiny-parts]; Simplygon's part remover deletes objects whose bounding-box diagonal is under a threshold, or "smaller than 1 pixel" at a chosen on-screen size [ref:simplygon-10.4-part-remover]; Unity's HLODSystem skips a renderer whose largest bounds dimension is under a minimum [ref:unity-hlodsystem-04be7e8-min-object-size]. Unreal's Simplified Mesh layer sizes the whole proxy in pixels at the nearest distance it is shown [ref:ue-5.8-hlod-simplify-screen-size], as Simplygon does [ref:simplygon-10.4-on-screen-size] and as Praxis's pixel at the Load radius does. A `world` window is 8 by 11 pixels at the swap, so no pixel-sized threshold drops it; what absorbs such detail in shipped pipelines is remeshing.

**17. Remeshing.** Simplygon's remesher fills gaps under a pixel and removes interiors [ref:simplygon-10.1-remeshing]. Unreal's Approximated Mesh voxelises at 1 m by default [ref:ue-5.8-hlod-approximate-voxel] and simplifies to a 0.1 m deviation, or to a count whose default is 2,000 triangles [ref:ue-5.8-approximate-simplify-policy]. Chen et al. extract an iso-surface, remove invisible components and judge the result by silhouette and normal differences from a view where the model spans a given number of pixels, about 7 minutes on average on 32 cores [ref:chen-2023-pixel-criterion]. Gao et al. carve a visual hull of a building and measure silhouettes as the XOR of binary masks, 45 s a model on average [ref:gao-2022-silhouette-metric], released as an executable [ref:gao-2022-low-poly-building]. In `meshopt` it is 0.7.0's voxel remesher [ref:meshoptimizer-1.3-voxel-remesh], which the decision declined; nothing measured below calls for it.

**Licences.** meshoptimizer 0.25 is MIT [ref:meshoptimizer-0.25-licence], and so is xatlas by its LICENSE at `f700c77`. RectangleBinPack's readme releases its code to the public domain, with no licence file. Unreal's source is under Epic's EULA and Unity's HLODSystem under the Unity Companion License [ref:unity-hlodsystem-04be7e8-readme]: both are read as evidence and never copied. Blender's baker is GPL-2.0-or-later and its Cycles kernel Apache-2.0, by the files' SPDX lines at v5.2.3. No code was found for Cignoni's tool or Knodt's method.

## Measurement: outlines from 256 m

A throwaway probe, kept out of the tree, merged the densest Chunk's static Placements less its ground, simplified the merge with each `meshopt` 0.6.2 call, and drew source and result with a CPU rasteriser at 2560 × 1440 and a 45° field of view, a sample at each pixel's centre. Eight views: from each side of the Chunk at 256 m from its nearest edge, from 60 m up (the bench camera's height) and from 1.7 m. A pixel is *lost* where the reference covers it and the result does not, and *gained* in the reverse case; its distance is to the nearest pixel the other covers (chamfer). A pixel shows *another building* where both cover it and the nearest surface belongs to a different building, which is how an outline between two overlapping buildings moves. Gao et al. measure silhouettes the same way, by the XOR of binary masks [ref:gao-2022-silhouette-metric]. Figures are sums over the eight views; the reference covers 2.49 M pixels. The [probe's log](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/probe.log) holds every call and view.

At one pixel of error (0.1473 m), against the merged source, windows included:

| Call | Input | Triangles | Reported error | Lost px | Lost beyond 1 px | Farthest |
|---|---|---:|---:|---:|---:|---:|
| `simplify`, with or without `Prune`; `simplify_with_attributes_and_locks` | merged source, per-face vertices | 298,722 | 0 | 0 | 0 | 0 |
| `simplify_sloppy` | merged source | 199,128 | 0.134 m | 426 | 0 | 1.0 px |
| `simplify`, with or without `Prune` | merged source, positions welded | 123,006 | 0.147 m | 740 | 0 | 1.0 px |
| none: windows dropped by hand | shell | 5,832 | 0 | 2,983 | 1,631 | 28.5 px |
| every call tried | shell, either form | 5,832 | 0 to 0.080 m | 2,983 | 1,631 | 28.5 px |

No call removes a window: the clustering and the welded collapse thin them (to 4 triangles each in the welded case) and leave the shell whole. The 1,631 pixels lost beyond a pixel when the windows are dropped all belong to the 48 windows standing on a neighbour's walls; against the source without those windows, dropping the rest loses 1,114 pixels, none beyond a pixel.

Under the cap of 3,863 triangles, with no error limit, against the Chunk's shell (2,486,433 pixels):

| Call | Input | Triangles | Reported error | Lost / gained px | Beyond 1 px | Farthest lost / gained | Another building, px |
|---|---|---:|---:|---:|---:|---:|---:|
| `simplify_sloppy` | merged source | 2,948 | 5.97 m | 712,249 / 16,300 | 665,778 / 12,403 | 56.6 / 16.0 px | 878,564 |
| `simplify_sloppy` | shell | 3,568 | 2.00 m | 14,121 / 11,624 | 6,157 / 7,505 | 8.0 / 12.4 px | 163,929 |
| `simplify` or `simplify_with_attributes_and_locks`, no `Prune` | shell, per-face vertices | 5,832 | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 |
| the same two with `Prune` | shell or merged source, per-face vertices | 3,644 | 6.12 m | 30,249 / 0 | 20,950 / 0 | 41.4 px / 0 | 67,061 |
| `simplify_with_attributes_and_locks`, `Permissive` | shell, per-face vertices | 3,862 | 6.14 m | 13,936 / 0 | 3,393 / 0 | 3.8 px / 0 | 34,368 |
| `simplify`, with or without `Prune` | shell, positions welded | 3,862 | 0.269 m | 16,593 / 0 | 4,948 / 0 | 3.8 px / 0 | 35,194 |
| `simplify`, with or without `Prune` | merged source, positions welded | 3,240 | 0.688 m | 22,848 / 0 | 8,206 / 0 | 4.2 px / 0 | 47,951 |
| `simplify` with `Prune`, error limit two pixels (0.2946 m), no count | shell, positions welded | 3,240 | 0.270 m | 22,848 / 0 | 8,206 / 0 | 4.2 px / 0 | 47,951 |

What the rows show:

- **Clustering fails at the cap.** With the windows in, they take the whole count. On the shell alone, walls lean and neighbours fuse: 7.6% of the reference's pixels change.
- **`Prune` fails at the cap.** It deletes the components of smallest radius, here many roofs and the smallest buildings' walls.
- **Edge collapse on welded positions holds the outline.** It flattens roof slabs, 0.3 m thick, onto their lower edges and touches nothing else: rooflines sit 0.3 m low, 2 pixels at 256 m and under a pixel past 520 m, and the farthest lost pixel is 3.8 to 4.2 pixels from the result. With every roof flat the shell is 10 triangles a building, 3,240.
- **A count removes the windows, an error limit does not.** On welded positions the cap takes every window out, at a reported error of 0.69 m, but only on a Chunk where the cap binds; at an error limit of one or two pixels the windows stay (123,006 and 120,396 triangles). They need a rule of their own.
- **The cap, not the simplifier, is what one pixel cannot meet.** A box under an overhanging slab is 18 triangles at 14.7 cm, and 324 of them are 5,832.

The densest view's proxies, by the building counts of the Chunks within 2,256 m of a camera on a 64 m grid, come to 35,022 buildings: 630,396 triangles at 18 a building, and 1.26 M with LOD chains of twice their LOD 0, the decision's allowance, against the Meshlet triangle Arena's 2,202,009 (derived, nothing cooked).

Images, on the `pr-images` branch at `1c8bd5e2`; grey is covered by both, red by the reference only, blue by the result only, and the reference is the merged source:

- The source from 60 m and from 1.7 m: [60 m](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/source-south-60m.png), [1.7 m](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/source-south-2m.png). The blank walls in the front row are those whose windows a neighbour holds.
- `simplify_sloppy` on the shell at the cap: [60 m](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/sloppy-cap-shell-south-60m-difference.png), [4x crop](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/sloppy-cap-shell-south-60m-difference-zoom.png), [1.7 m](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/sloppy-cap-shell-south-2m-difference.png), [4x crop](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/sloppy-cap-shell-south-2m-difference-zoom.png); on the merged source: [60 m](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/sloppy-cap-all-south-60m-difference.png).
- `Prune` at the cap: [60 m](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/collapse-cap-prune-shell-south-60m-difference.png), [4x crop](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/collapse-cap-prune-shell-south-60m-difference-zoom.png), [1.7 m](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/collapse-cap-prune-shell-south-2m-difference.png).
- `simplify` on welded positions at the cap: [60 m](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/collapse-cap-shell-welded-south-60m-difference.png), [4x crop](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/collapse-cap-shell-welded-south-60m-difference-zoom.png), [1.7 m](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/collapse-cap-shell-welded-south-2m-difference.png), [4x crop](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/collapse-cap-shell-welded-south-2m-difference-zoom.png); at two pixels: [60 m](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/collapse-two-pixels-prune-shell-welded-south-60m-difference.png), [4x crop](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/collapse-two-pixels-prune-shell-welded-south-60m-difference-zoom.png), [1.7 m, 4x crop](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/collapse-two-pixels-prune-shell-welded-south-2m-difference-zoom.png).
- At one pixel, windows kept: [`simplify_sloppy`](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/sloppy-error-all-south-60m-difference.png), [4x crop](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/sloppy-error-all-south-60m-difference-zoom.png), [`simplify` on welded positions](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/collapse-error-all-welded-south-60m-difference.png).
- Windows dropped, nothing simplified: [60 m](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/windows-dropped-shell-south-60m-difference.png), [4x crop](https://raw.githubusercontent.com/gariasf/praxis/1c8bd5e2a93188c6d8ae0c576e419d6c5756e945/805/windows-dropped-shell-south-60m-difference-zoom.png): the red patch is a window on a neighbour's wall.

Shelf packing of the shell's 2,916 quads into 4,096 columns, one rectangle a quad, tallest first, each side rounded up to whole texels:

| Texel | Ground chart | Gutter a side, grid | Clean to mip | Padded texels | Of 4096² | Rows used | Fits |
|---|---|---|---:|---:|---:|---:|---|
| 14.7 cm | no | none | | 8,992,836 | 54% | 2,271 | yes |
| 14.7 cm | no | 2, on 4 | 2 | 10,716,112 | 64% | 2,708 | yes |
| 14.7 cm | no | 4, on 8 | 3 | 13,204,224 | 79% | 3,296 | yes |
| 14.7 cm | no | 8, on 16 | 4 | 19,149,824 | 114% | 4,752 | no |
| 14.7 cm | yes | none | | 12,016,957 | 72% | 3,962 | yes |
| 14.7 cm | yes | 2, on 4 | 2 | 13,757,648 | 82% | 4,388 | no |
| 29.5 cm | no | 8, on 16 | 4 | 8,607,232 | 51% | 2,128 | yes |
| 29.5 cm | yes | 8, on 16 | 4 | 9,410,048 | 56% | 3,008 | yes |

Four charts in nine are roof-slab edges 3 texels high, which a grid of 16 rounds up to 32; the roofs are 15% of the shell's area and carry one flat colour.

## Options (no recommendation)

- **Geometry and its cap.**
  - E1: as the decision words it, `simplify_sloppy` and `Prune` under 3,863 triangles. Measured: outlines move by 12 and 41 pixels.
  - E2: drop Placements by a rule (D1 or D2), weld positions, `simplify` to one pixel, and set the cap where the densest Chunk's shell fits, 5,832 triangles. One pixel holds at the swap; the fit test of the densest 2 km view decides whether the Arena does.
  - E3: E2's calls with the cap kept at 3,863 and the error let rise on a Chunk over it: 3,240 triangles on the densest, its rooflines 0.3 m low.
  - E4: voxel remeshing on `meshopt` 0.7.0, the declined G3. Nothing measured calls for it.
- **The drop rule.**
  - D1: by bounding size, the shipped rule. To drop a window the threshold must pass 1.6 m, 11 pixels at the swap.
  - D2: by closeness: a Placement goes when every vertex of it lies within a pixel of another Placement's surface, which is what remeshing does implicitly. No engine read does it as a rule of its own.
- **Material transfer.**
  - X1: rasterise the sources into each chart along its normal with a depth test, n × n samples a texel.
  - X2: rays from a cage along the proxy's normal, nearest hit, n × n samples a texel. The same texels as X1 on a planar chart, and the one that still works when a proxy's faces leave the sources' planes.
  - X3: nearest point. Loses `world`'s windows.
- **Charts.** A1: one rectangle a planar face, packed by the cooker; A2: a general charter such as xatlas, a new dependency.
- **Gutters.**
  - B1: 4 texels a side on a grid of 8, dilated: clean to mip 3, which covers the View distance; 79% of the atlas on the densest Chunk.
  - B2: 8 texels a side on a grid of 16: clean to mip 4, the Resident tail's first mip. It needs half the density on the densest Chunk.
  - B3: the shipped default, 4 texels and dilation with no grid, coarse mips left to blend.
- **The ground.**
  - R1: no ground in a Far proxy; the flat ground of the decision's item 11 also draws under the far field, from the ground's own Material.
  - R2: the proxy carries its Chunk's ground tiles as a second Primitive with the ground's own Material, which makes two Materials of a proxy's one.
  - R3: the ground baked into the proxy's atlas, 3.02 M texels a Chunk; half the density on a dense Chunk.
  - R4: the ground baked to a coarse texture of its own, the shipped form, a terrain concern outside M5.

## Not verified

- Nothing was baked. What each transfer does to a window is derived from `world`'s geometry: the wall unbroken behind the glass, the glass 6 cm out.
- No bake or cook time: none is published for techniques 1 to 4 on current hardware, and none was measured.
- The probe's rasteriser samples pixel centres, draws no ground and shades nothing; the swap's look under 4x MSAA with a baked Material is still to be judged on the gallery.
- The outlines are one Chunk's, from eight views.
- The shelf packer is the simplest one; MAXRECTS would pack tighter and was not run. Block alignment is included only where the grid is 4 or more.
- The 1.26 M triangles of the densest view are building counts times 18, doubled for chains; nothing was cooked.
- That mip 3 suffices to 2 km assumes a surface facing the camera; what anisotropic filtering reads on a roof at a grazing angle was not measured.
- Liu et al. 2019 was read as its abstract on the authors' page, and Rossignac and Borrel as page images of the authors' scan.
- How Simplygon finds a texel's source triangle, and which Unreal HLOD layer calls ProxyLOD, are not documented in what was read.
- No studio's slides state how a distant building proxy's texture is baked: Insomniac's 2024 talk was read as its session overview alone [ref:kircher-2024-spider-man-2-distant-lods], and nothing first-party was found for Ubisoft's, Guerrilla's, Rockstar's or CD Projekt's cities.
- Chen et al.'s code and licence were not confirmed.

## Open questions

1. Does the cap rise to the densest Chunk's shell (E2), or does the proxy's error rise above a pixel on Chunks over the cap (E3)? Owner, on #804.
2. Which rule drops a Placement from the proxy's geometry, D1 or D2? Owner, on #804; #812 builds it.
3. Is #813's bleed test bound at mip 3 (B1) or at the Resident tail's first mip (B2, half the density on dense Chunks)? Owner, on #804.
4. Where does the ground between the Load radius and the View distance come from: R1, R2 or R3? Owner, on #804; R2 amends the decision's "one Material of its own".
5. Does the bake read the Placements of the eight neighbouring Chunks, for the 645 windows on this Chunk's walls that a neighbour holds? #813's builder; no option avoids it.
6. A general charter (A2) is a new dependency. It is not needed for `world`; the owner decides when a streamed scene has proxies whose faces are not planar.
