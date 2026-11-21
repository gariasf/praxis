# M5 sources: Audit lane 7 on M5's ADRs and their sources

The M5 exit's source check, in the form `docs/agents/audit.md` gives for
Audit lane 7 (docs and rules drift), narrowed to the ADRs M5 added or
amended and the research notes they rest on: ADRs 0026 and 0027 in full;
the M5 amendments to ADRs 0003, 0007, 0013, 0014, 0016, 0017, 0018, 0019,
0020, 0024 and 0025 (`git diff 6ebb45fb..62052fe8 -- docs/adr`, from the
last commit before M4's exit on 2026-11-11); and
`docs/research/far-proxy-baking.md`,
`docs/research/lod-hlod-and-far-field.md` and
`docs/research/acceleration-structures-for-streamed-worlds.md`. Every
`where` is a `path:line` at the frozen commit
`62052fe809eb856c0e0376bb7bcccb84c7f6afcd`, read in a checkout detached
at it. Prose claims were checked by the source-check rubric (load-bearing
claims in full, every fifth of the rest in document order, a wrong sample
opening its section); the 261 `docs/references.toml` entries the scoped
documents cite were each opened in full, with every citing sentence in
the scoped documents and every code site that cites them. Seven readers
at high effort, each with one shared brief that quoted the lane brief
verbatim: A on ADRs 0026 and 0027, B on the amendments, F on the
far-proxy note, L on the LOD note's prose and half its entries, M on the
other half, S on the acceleration-structures note's prose and half its
entries, T on the other half. One adversarial verifier reopened every
Finding. Finding ids carry the reader's letter. Severity is `drift`
throughout, as the lane brief sets; the magnitude says whether the claim
is load-bearing and whom it misleads. `docs/audit/deviations.md` holds no
row that touches the scope.

## Summary

M5's ADRs hold against their sources and the code: all 261 registry
entries the scoped documents cite were opened and every quote was found
at its source, one with a line range off by one and four with a slip of
form, and of 229 prose claims read, the decisions of ADRs 0026 and 0027
and the clauses the amendments added match the code they describe. Of 32
Findings filed, 28 survive the verifier as rows (one filed Finding split
into three locations), all `drift`, and one is load-bearing: the
glossary's LOD error is a distance from LOD 0, where the code and ADR
0026 store the simplifier's approximate measure, which falls short of
that distance by up to three times (L7-A01). Thirteen of the rest are one
slip, a research note's present-tense account of Praxis or its open
question that a later M5 change made false without rewriting or dating
it, so the Rule candidate that names it, extending the checklist's grep
for a changed name to `docs/research/`, is the one the rules ticket
should weigh first.

## Counts

| Reader | Prose claims checked | Registry entries opened | Findings filed | Surviving rows |
|---|---:|---:|---:|---:|
| A, ADRs 0026 and 0027 (40 load-bearing, 9 sampled) | 49 | 4 | 2 | 2 |
| B, the amendments (33 load-bearing, 8 sampled, 3 opened by a Finding) | 44 | 30 | 6 | 3 |
| F, `far-proxy-baking.md` (20 load-bearing, 10 sampled) | 30 | 69 | 2 | 2 |
| L, `lod-hlod-and-far-field.md` (16 load-bearing, 34 sampled, 14 opened by two wrong samples) | 64 | 42 | 10 | 10 |
| M, the LOD note's other entries | — | 41 | 1 | 0 |
| S, `acceleration-structures-for-streamed-worlds.md` (15 load-bearing, 3 in the M5 diff, 11 on the code, 13 sampled) | 42 | 38 | 5 | 7 |
| T, the acceleration-structures note's other entries | — | 37 | 6 | 4 |
| All | 229 | 261 | 32 | 28 |

Each reader of entries also read every sentence that cites them, so a
reader's Findings may fall in another reader's document: L7-B04 and
L7-T05 are against the acceleration-structures note, L7-T06 against a
code comment, and L7-B03, L7-L01, L7-L02, L7-L03 and L7-T01 against the
registry itself.

| Registry verdict | Entries |
|---|---:|
| `confirmed` | 260 |
| `wrong` (the section's line range only: L7-B03) | 1 |
| `unsupported`, `stale`, `dead`, `unopened` | 0 |

Four confirmed entries carry a Finding of form against the entry itself
(L7-L01, L7-L02, L7-L03, L7-T01) and keep their `checked` date, as does
the `wrong` one; the other 256 carry the date of this audit, 2026-11-21,
in the change that lands this report.

| Severity class | Surviving rows |
|---|---:|
| `drift`, load-bearing | 1 |
| `drift`, not load-bearing | 27 |
| `image`, `budget`, `correctness`, `health` | 0 |

| Verifier verdict | Findings |
|---|---:|
| `confirmed` | 23, as 25 rows after L7-S03's split |
| `narrowed` | 3 |
| `killed` | 6 |
| filed | 32 |

The verifier moved five Findings from load-bearing to not (L7-B01,
L7-L09, L7-L10, L7-S01, L7-S02): in each, the ADR states its own figure
or clause correctly and rests on that, not on the stale line.

## ADRs 0026 and 0027

I read `docs/adr/0026-the-far-field-draws-a-far-proxy-per-chunk.md` and `docs/adr/0027-coverage-in-the-main-pass-is-a-written-sample-mask.md` in full in the frozen checkout (HEAD `62052fe809eb856c0e0376bb7bcccb84c7f6afcd`). I checked every decision clause and every number a decision, a Budget or a term rests on as load-bearing, and every fifth remaining claim in document order as a sample. The code I read was: `src/cook/far_proxy.rs`, `src/cook/far_proxy/bake.rs`, `src/cook/mesh.rs` (pivot bound, `lod_0`, `push_lod`, LOD loop), `src/cook/mod.rs` (Far proxy Cook key, freshness, neighbours), `src/cook/texture.rs` (BC7 profiles), `src/cook/export.rs:60`, `src/assets/cooked.rs` (`MESH_MAX_LODS`, `CHUNK_EDGE_METRES`, `MAX_TEXTURE_SIDE`), `src/camera/mod.rs:90`, `src/scenes/mod.rs` (radii, Far proxy Asset IDs, `ground_plane`), `src/scenes/world.rs` (far ground, buildings, windows, roofs), `src/streaming/mod.rs` (the Chunk streamer's request, hold and lead), `src/render/acceleration_structures.rs` (Far proxy build wait, entry changes, mask), `src/state.rs` (the swap test), `src/render/main_pass.rs` (every main-pass pipeline and its `MultisampleState`), `src/render/shaders/{main,common,culling,shadow_mask}.wgsl`, `src/render/bind_groups.rs`, `src/render/targets.rs:16` and `src/gpu/mod.rs:318-320`. I also read `docs/performance.md:35`, `docs/audit/m5/scalability.md:66-131` and `CONTEXT.md` (Thin Primitive, LOD, LOD chain, LOD error, Resident, Cook key, Placement, Chunk, Load radius, Late Chunk, Far proxy, Far field, Late Far proxy, Fast travel, View distance). For the sources I downloaded meshoptimizer's `README.md`, `src/meshoptimizer.h` and `src/simplifier.cpp` raw at tag `v0.25` into `dl-A/`. Apart from CRLF line endings, the header and the simplifier are byte-identical to the copy vendored by `meshopt` 0.6.2, the version `Cargo.lock` pins, under `~/.cargo/registry/.../meshopt-0.6.2/vendor/src/`. The crate vendors no README. I also downloaded the WebGPU spec source `spec/index.bs` raw at commit `10be35c70d7bfc37e0557f2d0e38ec42742aba63`. Each registry url's `github.com/.../blob/...` form returned HTTP 200. I grepped every citing site of my four keys across the scoped documents and `src`, `tests` and `scripts`. Sites in `docs/guides/cook-far-proxy.html:378-380` also cite three of the keys. The guides are outside this audit's scope, but I read those three lines and each matches its quote.

### Claim verdicts

| doc | claim (quoted, shortened) | load-bearing | verdict | where / evidence |
|---|---|---|---|---|
| 0026:7 | "Every Chunk outside the Load radius and within the View distance draws its Far proxy, one Instance" | yes | confirmed | `src/cook/far_proxy.rs:212-354` cooks one Mesh per Chunk; ADR 0026:41 and `src/state.rs:907-950` place it as a Chunk of one Placement |
| 0026:7 | "the Instance slots, the static TLAS and the flat cull keep M4's capacities (`docs/audit/m5/scalability.md`)" | yes | confirmed | `docs/audit/m5/scalability.md:117-118`, `:125`: S01, S02 and S09 each "survives with HLOD" |
| 0026:8 | "first Primitive of merged, simplified geometry under one Material baked for it ... second Primitive of the Chunk's ground tiles, unsimplified, under the ground's own Material" | yes | confirmed | `src/cook/far_proxy.rs:281-321` (ground tiles copied unsimplified with the tile Mesh's Material; `lod_0(&[(shell, material), (ground_geometry, ground_material)])`); test `:763` |
| 0026:8 | "Only a scene with a Far field cooks proxies; it names its ground's Mesh" | sample (1st) | confirmed | `src/cook/export.rs:60`; `src/scenes/mod.rs:371-374` `far_field_ground` |
| 0026:9 | "drops a Placement when every vertex of it lies within one pixel of one other Placement's surface, ... the Chunk's own or one of its eight neighbours'" | yes | confirmed | `src/cook/far_proxy.rs:123-159` (`within_another`: every corner within `reach` of a triangle of one `other != index`; candidates only this Chunk's parts); `:240-260` neighbours' surfaces; `src/cook/mod.rs:195-200` eight neighbours |
| 0026:9 | "One pixel is its size at the Load radius at 1440p and a 45° field of view, 14.7 cm at 256 m" | yes | confirmed | `src/cook/far_proxy.rs:24`, `:28-30`; `src/camera/mod.rs:90` (π/4); `src/scenes/mod.rs:553` (`World => 256.0`); 256 × 2 × tan 22.5° / 1440 = 0.1473 m; test `:734` |
| 0026:9 | "welds what is left on position and simplifies it by edge collapse to one pixel of error, with no triangle count asked; the error is the simplifier's own measure, an approximation of the deviation" | yes | confirmed | `src/cook/far_proxy.rs:268-273` (position remap), `:161-178` (`meshopt::simplify(.., 0, error_metres, ErrorAbsolute, ..)`); registry row below |
| 0026:9 | "refuses a proxy of more than 5,832 triangles outside its ground: ... 324 buildings of 18" | yes | confirmed | `src/cook/far_proxy.rs:21`, `:275-279`; `src/scenes/world.rs:406-430` (walls: 4 quads, 8 triangles), `:433-470` (roof slab without bottom: 5 quads, 10); 324 × 18 = 5,832 |
| 0026:10 | "Two Channels ... a texel two pixels at the Load radius, 29.5 cm ... 2 texels of gutter a side on a grid of 4 ... at 4 × 4 samples a texel, on the CPU" | yes | confirmed | `src/cook/far_proxy.rs:22-23`; `src/cook/far_proxy/bake.rs:15-19` (`TEXEL_PIXELS = 2`, `SAMPLES_PER_SIDE = 4`, gutter `4 / 2`, grid `8 / 2`); 2 × 0.1473 = 0.2946 m |
| 0026:10 | amendment: "the gutter and the grid keep their metres ... the packer ... orders charts by their rows in texels" | yes | confirmed | `src/cook/far_proxy/bake.rs:18-19` (both are a fixed number of pixels divided by `TEXEL_PIXELS`), `:199` (sort by `size_texels.y`, descending) |
| 0026:11 | "A Mesh Blob per Chunk and, where its first Primitive has a triangle, a Material Blob and two Texture Blobs, each under an Asset ID made of the scene and the Chunk's coordinates" | yes | confirmed | `src/scenes/mod.rs:353-369`; `src/cook/far_proxy.rs:315-316` (Material only with an atlas); `src/cook/far_proxy/bake.rs` `chart_shell` returns `None` for an empty shell |
| 0026:11 | Cook key "covers the Chunk's and its neighbours' Cook keys, the Cook keys of the Meshes they place, of the Materials ... and of those Materials' albedo and metallic-roughness Textures, and the format and settings of each Blob" | yes | confirmed | `src/cook/mod.rs:285-366` |
| 0026:11 | "fresh when its Mesh is in the Cooked cache under that key and, unless that Mesh names no Material, the three baked Blobs are too" | yes | confirmed | `src/cook/mod.rs:182-192` |
| 0026:11 | "A proxy whose sources name a Material or a Texture the cook did not read is kept as it is" | yes | confirmed | `src/cook/mod.rs:344-358` (`kept = true`) |
| 0026:12 | "the swap is at the Load radius, a whole Chunk at once, with no fade ... A Late Chunk keeps its Far proxy until it loads" | yes | confirmed | `src/state.rs:907-950` (test `a_far_proxy_draws_exactly_while_its_chunk_does_not`) |
| 0026:13 | "A Far proxy is requested when its Chunk comes within the View distance and released past the View distance plus one Chunk's edge" | yes | drift (L7-A02) | `src/streaming/mod.rs:229-247` requests up to `closing × lead_frames` metres past the View distance; `:296-298` release at View distance + edge is as stated |
| 0026:14 | "Nothing is drawn but flat ground reaching the horizon, one Resident quad" | yes | confirmed | `src/scenes/world.rs:242-260` (one ground tile scaled); `src/scenes/mod.rs:1479-1497` (`ground_plane`: 4 vertices, 2 triangles) |
| 0026:20 | "cut on the welded positions, each coarser LOD simplified from LOD 0 to twice the error limit of the one before, and ends at eight LODs or at one that keeps more than 85% of the one before" | yes | confirmed | `src/cook/far_proxy.rs:330-345`; `src/assets/cooked.rs:42` (`MESH_MAX_LODS = 8`); `src/cook/mesh.rs:31` (`0.85`) |
| 0026:20 | "the corners of a box built face by face are vertices the simplifier locks" | yes | confirmed | registry row `meshoptimizer-0.25-vertex-kinds`; position remap `simplifier.cpp:201-213` gives a corner of three faces a three-wedge cycle, which `:463-466` locks |
| 0026:20 | "each LOD's faces get flat normals and their own vertices, shared with a finer LOD where they are equal" | yes | confirmed | `src/cook/far_proxy.rs:180-211` (`faceted`); `src/cook/mesh.rs:248-267` (`push_lod` reuses a finer LOD's identical vertex) |
| 0026:20 | "a pixel at the View distance spans 1.15 m" | sample (2nd) | confirmed | 2000 × 2 × tan 22.5° / 1440 = 1.1506 m; `src/scenes/mod.rs:561` |
| 0026:20 | "LOD 0 at 5,832 triangles and LOD 1 at 3,240, its roofs flat and 0.3 m lower" | yes | confirmed (0.27 m is a measurement, not pinned) | test `src/cook/far_proxy.rs:739-760` (LOD 1 error ≤ 2 px, 324 × 6 lowered vertices 0.3 m under their roof tops); `src/scenes/world.rs:97` (`ROOF_THICKNESS_METRES = 0.3`); 324 × (18 − 8) = 3,240 |
| 0026:21 | "pivot is its Chunk's centre on the ground, and the cooker refuses a proxy with a vertex more than a Chunk's edge from it" | yes | confirmed | `src/cook/far_proxy.rs:220`, `:322-328` |
| 0026:21 | "ADR 0014 keeps every other cooked Mesh within 128 m ... corners lie 181 m ... f32 resolves 0.015 mm between 128 and 256 m ... a quarter of a pixel of 37 mm" | yes | confirmed | `src/cook/mesh.rs:94`, `:164-176`; 128√2 = 181.0 m; 2⁷ × 2⁻²³ = 2⁻¹⁶ m = 0.0153 mm; 147.3 / 4 = 36.8 mm |
| 0026:22 | "A Placement that lies within a pixel of a ground tile is kept" | yes | confirmed | `src/cook/far_proxy.rs:225-226` (ground tiles are partitioned out of the surfaces); test `:430-499` (plate at 0.05 m kept) |
| 0026:24 | "A Chunk with nothing but ground cooks a proxy whose first Primitive is empty" | sample (3rd) | confirmed | `src/cook/far_proxy.rs:161-163` (`simplified` of no indices), `bake.rs` `chart_shell` → `None` |
| 0026:30 | "packed on shelves, tallest first, ... into the smallest square atlas of a power-of-two side ... past 8,192 texels a side fails its cook. `world`'s densest Chunk charts into 2048²" | yes | confirmed | `src/cook/far_proxy/bake.rs:197-224`; `src/assets/cooked.rs:36`; test `src/cook/far_proxy.rs:736-737` (density × 2048 = one texel per `texel_metres`) |
| 0026:31 | "takes the chart nearest it among those facing within 45° of it, or among all of them where none does" | yes | confirmed | `src/cook/far_proxy/bake.rs:21` (cos 45°), `:283-296` |
| 0026:32 | "facing along the chart's normal within two pixels of its plane" | yes | confirmed | `src/cook/far_proxy/bake.rs:17` (`REACH_PIXELS = 2.0`) |
| 0026:34 | "The albedo atlas encodes at the BC7 encoder's `ultra_fast` profile where the Texture cook's is `basic`" | yes | confirmed | `src/cook/texture.rs:21`, `:25`, `:623-626` |
| 0026:41 | "A Far proxy is held, its slot an Instance with no Mesh, while it is past the View distance or its Chunk is drawn" | sample (4th) | confirmed | `src/state.rs:907-950`; `src/render/acceleration_structures.rs:463-476` (a held proxy's freed entry) |
| 0026:42 | "starts no build for it ... starts one itself once it is 120 frames old" | yes | confirmed | `src/render/acceleration_structures.rs:199`, `:446-526` (`starts_build` false for Far proxy changes), `:569-576` |
| 0026:45 | "scaled to reach 1,024 km ... laid 0.5 m under ... never held, and traced by no ray ... 25 times the shallowest depth ... 62.5 mm ... a tenth of a pixel under the horizon" | yes | confirmed | `src/scenes/world.rs:42-51`, `:244-260` (`untraced_mesh`, `FarGround`); `docs/performance.md:35` (hidden at 0.02 m, through at 0.005 m: 0.5 / 0.02 = 25); ulp at 2¹⁹–2²⁰ m = 2⁻⁴ m = 62.5 mm; atan(60 / 1,024,000) / (2 tan 22.5° / 1440) = 0.10 px |
| 0026:49 | "at most 219 proxies are drawn" | sample (5th) | confirmed | `docs/audit/m5/scalability.md:66`, `:82` |
| 0026:56 | "3.02 M texels a Chunk of one colour at a texel a pixel, a quarter of that at two" | sample (6th) | confirmed | (256 / 0.1473)² = 3.02 M |
| 0026:61 | "ADR 0014's 128 m pivot bound exempts a Far proxy's Mesh" | yes | confirmed | `src/cook/mesh.rs:164-176` checks in `cook`, which the proxy bypasses through `lod_0` (`src/cook/far_proxy.rs:316`) |
| 0026:64 | "A LOD error is what the simplifier reports, which on `world`'s flattened roofs is a tenth under the distance they moved" | yes | confirmed against the code; contradicts `CONTEXT.md` (L7-A01) | `src/cook/far_proxy.rs:342` stores `reached_metres` from `meshopt::simplify` |
| 0027:3, :7 | "alpha-to-coverage stays off in every pipeline of the main pass" | yes | confirmed | `src/render/main_pass.rs:110-114`: one `MultisampleState`, `alpha_to_coverage_enabled: false`, passed to every depth and lit pipeline (`:115-222`); the only `alpha_to_coverage` in `src` |
| 0027:8 | "The depth passes write the mask ... The lit pass draws the same geometry at depth equal and writes none ... A fragment of its twin that keeps no sample is discarded" | yes | confirmed | `src/render/shaders/main.wgsl:186-215`; `src/render/main_pass.rs:211` (`depth_equal_state`) |
| 0027:9 | "a count ... equal to its coverage times four, rounded to the nearest, and none below an eighth ... sample 0, then 3, 1 and 2" | yes | confirmed | `src/render/shaders/common.wgsl:231-234`: `0xfb910` gives count 0–4 the masks 0x0, 0x1, 0x9, 0xb, 0xf; `src/render/targets.rs:16` (`SAMPLE_COUNT = 4`) |
| 0027:10 | "Each pipeline of the main pass has a twin ... the culling pass lists the geometry drawn through the twins apart, in slot order within each phase" | yes | confirmed | `src/render/main_pass.rs:147-222` (four twins); `src/render/shaders/culling.wgsl:59-60`, `:505-516`, `:557-561` |
| 0027:11 | Thin coverage "the tube's true width over the width it draws ... drops an Instance of a Mesh that is all Thin Primitives once its widest tube keeps no sample" | yes | confirmed | `src/render/shaders/main.wgsl:181-184`; `src/render/shaders/culling.wgsl:328-343`, `:425-426` |
| 0027:11 | "A Far proxy holds no Thin Primitive" | sample (7th) | confirmed | `src/cook/far_proxy.rs:180-211` (`Geometry::default()`, no `thin_radius_metres`) |
| 0027:12 | small Instance: "scaled ... until they cover a pixel's ... drops it once that rounds to no sample ... four pixels wide ... scaled no further ... A Mesh with a Thin Primitive keeps that rule alone" | yes | confirmed | `src/render/shaders/common.wgsl:236-280`; `src/render/shaders/culling.wgsl:399-424`; `src/render/shaders/main.wgsl:147-159` |
| 0027:16 | "Which samples a given alpha keeps is the platform's choice ... a pipeline that enables it may not also write `sample_mask`" | yes | confirmed | registry row below; first clause, uncited, matches spec `index.bs:17018` "The algorithm of producing the extra mask is platform-dependent and can vary for different pixels." |
| 0027:24 | "ADR 0019's 'alpha-to-coverage for masked materials' no longer holds" | sample (8th) | confirmed | `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:3` carries the amendment |
| 0027:25 | "a third wider for three faces ... a sixth of a pixel wide ... three faces' share, 0.75 of its diameter ... from 65 m to 75 m and from 521 m to 602 m" | yes | confirmed | `src/render/shaders/common.wgsl:224-228` (`THIN_NARROWEST_VIEW = 0.75`); 1 / 0.75 = 1.33; 0.125 / 0.75 = 1/6; 75/65 ≈ 602/521 ≈ 1.155 = cos 30° / 0.75 |
| 0027:26 | "The culling pass's compute stage already binds the nine storage buffers the Capability floor allows" | yes | confirmed | `src/gpu/mod.rs:318-320`; `src/render/bind_groups.rs:72-100` (instances, meshes, meshlets, meshlet_triangles to compute) + `:131-135` (five) = 9 |
| 0027:27 | "A fragment stage that writes the mask reads a flat input" | sample (9th) | confirmed | `src/render/shaders/main.wgsl:100`, `:103` (`@interpolate(flat)` radius and coverage) |

49 claims checked: 40 as load-bearing and 9 as fifth samples. None of the samples was wrong.

### Registry verdicts

| key | registry line | verdict | what was opened, what it says, and whether each citing site holds |
|---|---|---|---|
| `meshoptimizer-0.25-simplify-error` | `docs/references.toml:5782` | confirmed | Opened: `https://raw.githubusercontent.com/zeux/meshoptimizer/v0.25/README.md`, a primary source at the pinned tag; the blob url returns 200. The quote is verbatim from line 446 ("Target error is an approximate measure of the deviation from the original mesh using distance normalized to `[0..1]` range ...") and line 448 ("To disable the error limit, `target_error` can be set to `FLT_MAX`. ... setting `target_index_count` to 0 will simplify the input mesh as much as possible within the specified error limit; ..."). Both lines lie inside the section's 446-452. The range is two lines wider than the quote, and line 452 is the `meshopt_simplifySloppy` paragraph. That is not a defect. Citing sites: ADR 0026:9, "the error is the simplifier's own measure, an approximation of the deviation", holds. The cook passes `ErrorAbsolute` (`src/cook/far_proxy.rs:174`), which `meshoptimizer.h:411` documents as "Treat error limit and resulting error as absolute instead of relative to mesh extents". The ADR does not claim normalisation, so the quote's normalised wording does not contradict it. `docs/research/lod-hlod-and-far-field.md:37`, "targets an error normalized to the mesh's extent", holds. `src/cook/mesh.rs:365`, "No error limit" before `f32::MAX`, holds (FLT_MAX). `src/render/shaders/shadow_mask.wgsl:127-128`, "A cooked LOD error is an approximate measure, no bound", holds. |
| `meshoptimizer-0.25-simplify-original-vertices` | `docs/references.toml:5788` | confirmed | Opened: `meshoptimizer.h` raw at `v0.25`, identical to the vendored copy. Line 433 is "* Mesh simplifier", line 434 is "Reduces the number of triangles in the mesh, attempting to preserve mesh appearance as much as possible", and line 439 is "The resulting index buffer references vertices from the original vertex buffer." The quote is verbatim and the section is right. Its `checked` is 2026-11-06, older than ADR 0026's citation; this check dates it 2026-11-21. Citing sites: ADR 0026:20, "the simplifier only picks among the vertices it is given", holds for every call the `meshopt` 0.6.2 crate exposes (`src/simplify.rs` has no binding for the experimental `meshopt_simplifyWithUpdate`, which moves vertices in place, header `:476-495`). `docs/research/lod-hlod-and-far-field.md:49`, "levels index the original vertices and give no ancestor map", holds: `meshopt_simplify` (`:448`) returns only indices and `result_error`. |
| `meshoptimizer-0.25-vertex-kinds` | `docs/references.toml:6559` | confirmed | Opened: `simplifier.cpp` raw at `v0.25`, identical to the vendored copy. `Kind_Border` through `Kind_Locked` are at lines 296-299, inside the url's L292-L300; the enum itself spans 293-302. Line 443 is "// seam should have one open half-edge for each vertex, ...", and lines 465-466 are "// more than one vertex maps to this one; we don't have classification available" / "result[i] = Kind_Locked;", inside the stated 436-467. I compared the tabs byte by byte and the quote is verbatim. Citing sites: ADR 0026:20, "the corners of a box built face by face are vertices the simplifier locks", holds. `buildPositionRemap` (`:201-213`) joins equal positions, so a corner shared by three faces has a wedge cycle of three, `wedge[wedge[i]] != i`, which `:463-466` locks. `docs/research/far-proxy-baking.md:80`, "locks one where more than two attribute wedges meet or a seam does not close", holds (`:436-461`, `:463-466`). |
| `webgpu-alpha-to-coverage-sample-mask` | `docs/references.toml:681` | confirmed | Opened: `https://raw.githubusercontent.com/gpuweb/gpuweb/10be35c70d7bfc37e0557f2d0e38ec42742aba63/spec/index.bs`; the blob url returns 200. Lines 8792-8794 are under `### Render Pipeline Creation` (line 8570), in the algorithm "validating GPURenderPipelineDescriptor" (line 8776): "If the [=builtin/sample_mask=] builtin is a [=shader stage output=] of \|descriptor\|.{{...fragment}}: - \|descriptor\|.{{...multisample}}.{{GPUMultisampleState/alphaToCoverageEnabled}} is `false`." With the Bikeshed markup stripped, this is the quote verbatim, and the section is right. Citing site: ADR 0027:16, "a pipeline that enables it may not also write `sample_mask`", holds. No code site cites the key. |

Entries: 4 confirmed, 0 wrong, 0 unsupported, 0 stale, 0 dead, 0 unopened.

## M5 amendments to ADRs 0003, 0007, 0013, 0014, 0016, 0017, 0018, 0019, 0020, 0024 and 0025, and their 30 registry entries

I read the amendment diff (`git diff --word-diff 6ebb45fb..62052fe8` over the eleven ADRs, 51 insertions) at the frozen commit `62052fe809eb856c0e0376bb7bcccb84c7f6afcd`, split it into 76 claims in document order, checked the 33 load-bearing ones in full and every fifth of the other 43 (8 samples; none was wrong, so no section opened), and checked a few more where a registry citing site or a Finding needed them. The code at the frozen checkout was the referent: `src/render/prepare.rs`, `src/threading.rs`, `src/render/acceleration_structures.rs`, `src/streaming/{io,mips,chunks}.rs`, `src/assets/{pools,cooked}.rs`, `src/cook/{mesh,far_proxy}.rs`, `src/render/shaders/{common,culling}.wgsl`, `src/render/renderer.rs`, `src/bench/{mod,provenance}.rs`, `bench/budgets.json`, `vendor/`, `LICENSE`, `Cargo.toml`, `.github/workflows`. Measured figures were checked against the Bench records on `origin/bench-history` (the three `town` records of source `6a91ca76`) and, for figures with no record, against the issue or PR body the amendment links (`gh`). For the 30 registry entries I opened every source: crate sources locally at the cited version under `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` (wgpu, wgpu-core, wgpu-hal, wgpu-types 30.0.1 and 30.0.2, async-executor 1.14.0, concurrent-queue 2.5.0, bevy_ecs and bevy_tasks 0.18.1, meshopt 0.6.2's vendored meshoptimizer), each also checked at the pinned 30.0.2 or `vendor/wgpu-core` for staleness, with GitHub raw files at the tags (`v30.0.1`, `v30.0.2`, meshoptimizer `v0.25`) where line numbers mattered; Apple's documentation through its JSON endpoints; the Unity page live; the Unreal page through its archived snapshot (`id_` form, decompressed); the four PDFs with `pdftotext` at the cited page (Greene p. 3, Karis p. 19, Jimenez p. 7, Metal Feature Set Tables p. 9 and its 21 May 2026 date); gfx-rs/wgpu pull 9835 and the wgpu release dates through `gh api`. Every citing site of each key in the scoped documents (`grep -n "ref:<key>]"`) and in `src`, `tests`, `scripts` was read. Terms were checked against `CONTEXT.md`.

### Claim verdicts

| doc | claim (quoted, shortened) | load-bearing | verdict | where / evidence |
|---|---|---|---|---|
| ADR 0003 | "prepare writes again every Renderable's Instance and the Placements of each drawn Chunk one of whose Placements names an entry no longer in its pool; free ranges, free slots and held Chunks [...] are not written again" | yes | confirmed | `src/render/prepare.rs:671-705` (`pools_freed`, `draws_a_freed_entry` at :925 returns false for a held Chunk), :681-690 and :836-842 rewrite free ranges and slots only under `rewrite_every_instance` |
| ADR 0007 | "Each milestone exit's dependency review (`docs/agents/workflow.md`, The ladder) checks three things" | yes | wrong | L7-B01: `docs/agents/workflow.md:13` names no such check |
| ADR 0007 | ray queries "wgpu calls experimental and 'expected to be subject to breaking changes'" | yes | confirmed | registry `wgpu-types-30-ray-query-experimental`, features.rs L1049-1050 in 30.0.1 and 30.0.2 |
| ADR 0007 | "`vendor/` carries more than five upstream commits, which `vendor/README.md` lists" (trigger) | yes | confirmed | `vendor/README.md:23-26` lists one commit, so the trigger is not met |
| ADR 0007 | exit: "on 2026-11-14 a copy of wgpu-core 30.0.1 carries one upstream fix" | no (checked for a Finding) | stale | L7-B02: the copy is 30.0.2 since 2026-11-15 |
| ADR 0013 | "final only with the figure on both adapters [...] their `provenance.source_hash` equal (`docs/agents/workflow.md`, Performance and memory)" | yes | confirmed | `docs/agents/workflow.md:58` |
| ADR 0013 | reference machine "gates regressions, the Budgets being judged on it" | no, fifth sample | confirmed | `src/bench/mod.rs:1184-1360` judges Budgets on the Baseline's adapter |
| ADR 0013 | "On `town` at one source the M4 Pro's GPU frame reads 2.2 times [...] shadow rays 2.7 to 3.1 times and ADR 0019's gate 1.7 to 1.8 times [...] source hash `6a91ca76`" | yes | confirmed | `origin/bench-history:records/{apple-m4-pro-metal,nvidia-geforce-rtx-2070-super-dx12,nvidia-geforce-rtx-2070-super-vulkan}/town/6a91ca76....json`: GPU frame median 20,000.0 / 9,157.2 and 9,248.8 µs (2.18, 2.16); shadow rays 10,916.5 / 3,542.1 and 4,009.1 (3.08, 2.72); gate 8,573.1 / 5,136.4 and 4,821.2 (1.67, 1.78) |
| ADR 0013 | guard: per-slot stages "at or under 1,000 µs [...] `culling_per_slot_microseconds` in `bench/budgets.json`, and `praxis compare` fails a scene past it" | yes | confirmed | `bench/budgets.json:7`; `src/bench/mod.rs:1010-1020` (sum of both phases' `slots_label` passes), :1349-1360 (Fail); `src/render/culling_pass.rs:44-47` names the three stages |
| ADR 0013 | "the Bench record's counters hold the Instance slots in use" | no | confirmed | `src/bench/mod.rs:336` `instance_slots_in_use` |
| ADR 0014 | static TLAS frame "the corner of the camera's Anchor when its first static Instance was written, and a build moves it only once the camera's Anchor strays more than a Chunk's edge (4 cells)" | yes | confirmed | `src/render/acceleration_structures.rs:195` (256 m / 64 m), :428-435 (follows the camera while empty), :563-597 |
| ADR 0014 | Far proxy Mesh "within a Chunk's edge of [its centre], which the cooker enforces instead; it is cooked to be placed unrotated [...] drawn from the Load radius outwards unless its Chunk is late" | yes | confirmed | `src/cook/far_proxy.rs:220,320-328`; `src/streaming/chunks.rs:118-127` (`Quat::IDENTITY`); `src/state.rs:912` |
| ADR 0016 | "one pool per State's `FramePreparer`"; "a `TaskPool` that each State's `FramePreparer` owns, not the process-global `ComputeTaskPool`", `available_parallelism() − 1` (at least 1) | yes | confirmed | `src/render/prepare.rs:487-489,519`; `src/threading.rs:11-19`; `src/state.rs:230`; no `ComputeTaskPool` in `src` |
| ADR 0016 | queue "allocates its next block on the thread that pushes a block's last task, once every 31" | yes | confirmed | registry `concurrent-queue-2.5.0-push-allocates-block`; unbounded.rs L23-25 (`BLOCK_CAP = LAP - 1 = 31`), L192-196 |
| ADR 0016 | "`world`'s counter frames read 10,474 heap allocations after `town`'s run in one process and 10,473 alone" | no, fifth sample | confirmed | the issue the amendment links (ADR 0016:16), its triage table: `town world` 10,474, `world` 10,473 |
| ADR 0016 | "`par_iter` runs on the process-global pool [...] which the engine no longer creates" | yes | confirmed | par_iter.rs L104 `ComputeTaskPool::get()`; no `par_iter` or `ComputeTaskPool` in `src`. Note: `get()` panics while uninitialised (bevy_tasks usages.rs L29-34), so "the first system to use it decides" reads as a deferred decision, not a fallback |
| ADR 0016 | IO threads "take Chunk reads first, then mips, then Far proxy reads [...] a Far proxy's only while no Chunk read is taken and on all threads but one [...] 5.3 MiB for a 2,048² BC7 atlas's chain" | yes | confirmed | `src/streaming/io.rs:29-31,161-203`; 2,048² × 1 B = 4 MiB, × 4/3 = 5.33 MiB |
| ADR 0017 | held Chunks' and Far proxies' mips "are requested as the drawn ones' are and are never late, as Unreal's streamer prestreams [...] there one mip coarser (`docs/audit/deviations.md`)" | yes | confirmed | `src/streaming/mips.rs:49-61` `Sight::Ahead`; `docs/audit/deviations.md:26`; registry `ue-5.8-texture-streaming-hidden-prefetch` |
| ADR 0017 | Arenas "take 65.8%, 13.0%, 1.1%, 13.3% and 6.8%" | yes | confirmed | `src/assets/pools.rs:19` `[658, 130, 11, 133, 68]` |
| ADR 0017 | "with LOD 0's Meshlets alone the Arenas took 48.7%, 31.7%, 0.6%, 8.5% and 10.5%, and a Mesh 4,488 bytes" | no, fifth sample | confirmed | `git show 6ebb45fb:src/assets/pools.rs:16,19`; replaced by `d3f7d0a7` (2026-11-14) |
| ADR 0017 | "the Mesh table an entry per 5,124 bytes" | yes | confirmed | `src/assets/pools.rs:33` |
| ADR 0017 | "a second count of 285 entries, which only a Far proxy's Material takes [...] a load past either is refused, logged and counted" | yes | confirmed | `src/assets/pools.rs:43,96`; `src/assets/material.rs:1920-1938` |
| ADR 0017 | "vertex Arena at 55,197,040 bytes, under the floor's 128 MiB binding" (and 40,852,520 and 34,225,520 before) | yes | confirmed | 83,886,080 × 0.658 = 55,197,040.6, rounded down to 4 bytes; `src/assets/pools.rs:1416`; 0.487 and 0.408 give the earlier two |
| ADR 0017 | "A frame also loads at most 3 Chunks" | yes | confirmed | `src/streaming/io.rs:749-752,1298` |
| ADR 0017 | "a frame's 3 loads are its Chunks' first, then its Far proxies', nearest first" | no, fifth sample | confirmed | `src/streaming/io.rs:996,1261-1290` |
| ADR 0017 | "a Settled wait loads past it" | no (term) | drift | L7-B06 |
| ADR 0018 | Blob "holds the Mesh's LOD chain: the vertices once [...] per LOD its error in metres, each Primitive's run of its indices and its Meshlets, with every LOD's indices in one encoded stream" | yes | confirmed | `src/assets/cooked.rs:636-697` |
| ADR 0018 | "simplified from the one before to half its triangles [...] the whole Mesh in one call, normals and UVs weighted [...] at most 8 LODs and ends before a LOD that would keep more than 85%" | yes | confirmed | `src/cook/mesh.rs:31,291-372`; `src/assets/cooked.rs:42` |
| ADR 0018 | "A Thin Primitive's vertices are locked in the simplifier [...] so every LOD keeps its LOD 0 triangles" | yes | confirmed | `src/cook/mesh.rs:341-357` (`locked` from `thin_radius_metres`); registry `meshoptimizer-0.25-vertex-lock` |
| ADR 0018 | "Each Primitive stores a Thin Primitive's radius, zero for any other" | no, fifth sample | confirmed | `src/assets/cooked.rs:671,723-731` |
| ADR 0019 | "the Mesh table gives the culling pass each LOD's Meshlet range, triangle range and LOD error, for at most 8 LODs" | yes | confirmed | `src/render/shaders/common.wgsl:190-200` |
| ADR 0019 | "the coarsest LOD whose LOD error times the Instance's scale is under the LOD threshold, one pixel of the target's height, at the distance to the Instance's bounding sphere" | yes | confirmed | `src/render/shaders/common.wgsl:203-219`; `src/render/shaders/culling.wgsl:285-299`; `src/render/renderer.rs:185-200` (`LOD_THRESHOLD_PIXELS = 1.0`) |
| ADR 0019 | "It reads the camera and the Instance alone, so both phases draw the same LOD" | no, fifth sample | confirmed | `src/render/shaders/culling.wgsl:280-299` |
| ADR 0019 | "kept branches, `prototype/visibility-b` and `prototype/visibility-d`" | no (extra) | confirmed | both on `origin` |
| ADR 0020 | static TLAS holds "the drawn Far proxies" | yes | confirmed | `src/render/acceleration_structures.rs:238-240,383-439` |
| ADR 0020 | "one cooked lighting term is a per-Chunk sky-visibility field [...] for what is drawn beyond the probe cascades" | no, fifth sample | confirmed as planned | no such field or probes in `src`; the clause is a plan |
| ADR 0020 | "A static build keeps the static TLAS's Anchor until the camera's is more than a Chunk's edge from it [...] where a build runs though no static Instance changed and re-anchors it" | yes | confirmed | `src/render/acceleration_structures.rs:190-195,563-597` |
| ADR 0020 | a Far proxy's entry change "starts one only once it is 120 frames old [...] 402 builds [...] against 59" | yes | confirmed | `src/render/acceleration_structures.rs:199`; the measurement in the body of the PR that brought it ("402 builds against 59 untraced") |
| ADR 0020 | "Every BLAS is built from its Mesh's LOD 0 [...] the index Arena [...] holds LOD 0's indices alone" | yes | confirmed | `src/assets/pools.rs:465,1694`; `CONTEXT.md` Arena |
| ADR 0024 | "Self-hosted Actions runners on the M4 Pro and the RTX 2070 SUPER" (considered option) | no, fifth sample | confirmed | a rename of "the two machines"; `docs/adr/0024-...md:11` |
| ADR 0024 | "the p95 being that of each GPU frame over the median of the 31 frames around it" | yes | confirmed | `src/bench/provenance.rs:36-43` (`NEIGHBOUR_FRAMES = 31`, `QUIET_P95_OVER_MEDIAN = 1.06`) |
| ADR 0025 | "published under the MIT licence (`LICENSE`, `Cargo.toml`)" | yes | confirmed | `LICENSE:1`; `Cargo.toml:5` |
| ADR 0025 | "Every file under `assets/` has a manifest entry and a credit, which `cargo test` checks" | yes | confirmed | `tests/asset_manifest.rs` (reads `ATTRIBUTION.md`) |
| ADR 0025 | "a vendored crate keeps its licence files [...] the allowlist in `tests/dependency_licences.rs`" | yes | confirmed | `vendor/wgpu-core/LICENSE.{MIT,APACHE}`; `tests/dependency_licences.rs` |
| ADR 0025 | "CI runs on GitHub's hosted runners, without a GPU" | no (extra) | confirmed | every `runs-on` in `.github/workflows` is `ubuntu-*` or `windows-2025` |

Claims checked: 33 load-bearing, 8 fifth samples, 3 extra (45 rows above; 44 of them claims, the ADR 0007 exit row being the Finding check).

### Registry verdicts

| key | registry line | verdict | what was opened, what it says, and whether each citing site holds |
|---|---|---|---|
| `apple-mtldevice-current-allocated-size` | `docs/references.toml:2092` | confirmed | Apple JSON `metal/mtldevice/currentallocatedsize`: abstract verbatim, macOS 10.13. ADR 0013:3 ("diagnostics only") holds |
| `apple-nsview-main-actor` | :1882 | confirmed | Apple JSON `appkit/nsview`: declaration `@MainActor class NSView`. ADR 0016:3 holds |
| `async-executor-1.14.0-global-queue` | :1994 | confirmed | lib.rs L357-364 verbatim, State::new L677 `ConcurrentQueue::unbounded()`; bevy_tasks task_pool.rs L646-650, executor.rs L19/L38 wrapper. ADR 0016:16 holds (a Scope's own `spawned` queue is made per scope, task_pool.rs L387-388, so "the pool's one" queue is the executor's) |
| `bevy-ecs-0.18.1-par-iter-multi-threaded` | :1922 | confirmed | par_iter.rs L86-104 verbatim. ADR 0016:14 and :16 hold |
| `bevy-tasks-0.18.1-default-threads` | :1908 | confirmed | task_pool.rs L165-167 verbatim; Default L603; lib.rs L167. ADR 0016:15 holds |
| `concurrent-queue-2.5.0-push-allocates-block` | :2004 | confirmed | unbounded.rs L192-196 verbatim; L23-25 LAP and BLOCK_CAP. ADR 0016:16 and `src/render/prepare.rs:488` hold |
| `greene-hiz` | :633 | confirmed | course PDF p. 3, §3.2: both fragments verbatim. ADR 0019:15 (minimum reverse-Z is the farthest Z) holds |
| `meshoptimizer-0.25-vertex-lock` | :6569 | wrong (section) | v0.25 `src/meshoptimizer.h` (raw at tag, and meshopt 0.6.2's vendored copy): quote verbatim, but `meshopt_SimplifyVertex_Lock = 1 << 0` is L427, outside the section's "L421-L426" (L7-B03). ADR 0018:24 and `src/cook/mesh.rs:295` hold |
| `metal-feature-set-tables-msaa-max` | :445 | confirmed | archived PDF, dated 21 May 2026, p. 9 row "Maximum sample count in render passes with MSAA 4 4 4 4 4 4 4 4 8". ADR 0019:3 (Apple9 maximum 4) holds |
| `nanite-two-pass-occlusion` | :597 | confirmed | Karis PDF p. 19, slide text verbatim. ADR 0019:15 holds |
| `smaa-subpixel-grid` | :585 | confirmed | SMAA PDF p. 7, Figure 10 caption verbatim. ADR 0019:3 holds |
| `ue-5.8-texture-streaming-hidden-prefetch` | :3269 | confirmed | snapshot 20261015152713: Hidden Mips row and the 4.15 Performance paragraph verbatim ("althought" as the page spells it). ADR 0017:15 and `src/streaming/mips.rs:56` hold, with `docs/audit/deviations.md:26` |
| `unity-6000-uv-distribution-metric` | :6112 | confirmed | live page: description, return value and example comments verbatim. ADR 0018:24 and `src/assets/cooked.rs:507` hold |
| `wgpu-30-blas-compacted-size-private` | :4827 | confirmed | wgpu-core 30.0.1 resource.rs L2510-2516, queue.rs L2036-2042; same in `vendor/wgpu-core` L2519. ADR 0017:18, note :50, `src/render/acceleration_structures.rs:31` hold |
| `wgpu-30-blas-size-at-create` | :4821 | confirmed | device/ray_tracing.rs L100-118, L60-69, L159-164 (tag `v30.0.1` raw matches the local crate). ADR 0017:18 and `src/render/acceleration_structures.rs:118` hold |
| `wgpu-30-metal-count-draws-empty` | :547 | confirmed | wgpu-hal 30.0.1 metal/command.rs L1651-1670 (both `//TODO`); still so in 30.0.2. ADR 0019:15 holds |
| `wgpu-30-metal-indirect-loop` | :541 | confirmed | L1608-1649, `for _ in 0..draw_count`. ADR 0019:15 holds |
| `wgpu-30-metal-memoryless` | :553 | confirmed | metal/device.rs L561-564. ADR 0019:3 holds |
| `wgpu-30-staging-belt-chunk-size` | :973 | confirmed | wgpu 30.0.1 util/belt.rs L56-57, L132-133. ADR 0017:19 holds |
| `wgpu-30-tlas-instance-buffer` | :4898 | confirmed | device/ray_tracing.rs L268-275. Note :50 and `src/render/acceleration_structures.rs:120` hold; ADR 0017:18's "72 bytes a slot on Metal" is not in this or its two sibling keys (L7-B05) |
| `wgpu-30-tlas-rebuild-only` | :469 | confirmed | command/ray_tracing.rs L336-341 (L346 in `vendor/`). ADR 0020:20 and note :32 hold |
| `wgpu-30-tlas-size-at-create` | :4916 | confirmed | device/ray_tracing.rs L242-266. ADR 0017:18, note :50 and code :118 hold |
| `wgpu-core-30-write-texture-staging` | :967 | confirmed | queue.rs L853, L994-1013. ADR 0017:19 holds |
| `wgpu-hal-30-allocator-report-default` | :1788 | confirmed | lib.rs L1210-1212; overrides only dx12 and vulkan; 29.0.1 L1164-1166; 30.0.2 the same. ADR 0013:3 and `src/stats/mod.rs:77` hold |
| `wgpu-hal-30-instance-bytes` | :4958 | confirmed | vulkan/adapter.rs L1825 `raw_tlas_instance_size: 64`; dx12 L1037-1040. ADR 0017:18 ("64 elsewhere") and code :121 hold |
| `wgpu-hal-30-metal-acquire-window` | :1900 | confirmed | metal/surface.rs L19-35, L354-375 verbatim. ADR 0016:3 holds |
| `wgpu-hal-30-metal-instance-bytes` | :4964 | confirmed | metal/device.rs L2123-2124, L2158; adapter.rs L1503-1506. Code :121 holds; ADR 0017:18 cites it for "72 bytes", which it does not state (L7-B05) |
| `wgpu-pr-9835` | :3457 | confirmed | `gh api repos/gfx-rs/wgpu/pulls/9835`: title and description verbatim, merged 2026-07-17, merge commit `59209f27`; absent at tag `v30.0.2`, present in `vendor/`. ADR 0020:20 and `src/render/bind_groups.rs:108` hold; the note's "after 30.0.1" (:32) does not (L7-B04) |
| `wgpu-types-30-ray-query-experimental` | :1513 | confirmed | wgpu-types 30.0.1 features.rs L1049-1060 verbatim; same in 30.0.2. ADR 0007:3 and :9 hold |
| `wgpu-types-30-transient-saves-memory` | :981 | confirmed | adapter.rs L212-214. ADR 0017:18 and `src/stats/gpu_memory.rs:130` hold |

Entries: 29 confirmed, 1 wrong (a section's line range). Every entry has a `checked` date.

## `docs/research/far-proxy-baking.md`, and the 69 registry entries it cites that no ADR reader holds

I read `docs/research/far-proxy-baking.md` in full at the frozen commit `62052fe809eb856c0e0376bb7bcccb84c7f6afcd` (in the frozen checkout), with ADR 0026 beside it to tell load-bearing claims from the rest, and checked its claims about this repo against the code there: `src/scenes/world.rs` (walls, roof slab, window and slab geometry, `WINDOW_SIZE_METRES`, `ROOF_THICKNESS_METRES`, `FACADE_TEXTURE_SIDE`/`FACADE_METRES`, `CHUNKS_PER_SIDE`, `CAMERA_HEIGHT_METRES`, the pivot rule at :501, window placement at :642-660), `src/scenes/mod.rs:557-564` (View distance), `src/assets/cooked.rs` (`MAX_TEXTURE_SIDE`, `resident_tail_level`, `CHUNK_EDGE_METRES`), `src/stats/gpu_memory.rs:107` (`RESIDENT_TAIL_LEVEL_BYTES`), `src/cook/texture.rs` (BC7, BC5), `src/cook/` (no GPU device), `src/assets/pools.rs` (`ARENA_PER_MILLE`, at the frozen commit and at the note's `d4ac21eb`), `src/assets/mesh.rs:50-54`, `bench/budgets.json`, `Cargo.toml`. The note's measured figures I checked against the probe's log at its pinned commit (`raw.githubusercontent.com/gariasf/praxis/1c8bd5e2.../805/probe.log`, 469 lines), and its derived ones by arithmetic. For the registry I opened every one of the 69 assigned entries, none sampled: meshoptimizer 0.25 sources as raw files at tag `v0.25` (`src/simplifier.cpp`, `src/meshoptimizer.h`, `README.md`, `gltf/mesh.cpp`, `LICENSE.md`), byte-identical but for CRLF to the `meshopt` 0.6.2 crate's vendored copy in `~/.cargo/registry/.../meshopt-0.6.2/vendor/src`; that crate's `src/simplify.rs`, `src/lib.rs` and `gen/bindings.rs`; meshoptimizer's `v1.3` README (and the `v1.0`, `v1.1`, `v1.2` READMEs, to date the remesher), and the `meshopt` 0.7.0 crate from crates.io (inspected, not built); every PDF at its cited page with `pdftotext` (`-layout`, `-table` where a table's columns ran together), the Rossignac and Borrel scan by extracting its embedded page JPEGs and reading pages 2 and 3 as images; the Simplygon, Blender, Kircher (archived, `id_`) and Liu pages as HTML; Unreal Engine 5.8.4 sources through `gh api` (the account is Epic-linked); Unity HLODSystem, xatlas, nvdiffmodeling, RectangleBinPack and Blender sources as raw files at their pinned commits or tags. Every citing site of each key was found by `grep -rnF "ref:<key>]"` over the scoped ADRs and notes and `grep -rnF "<key>"` over `src`, `tests` and `scripts`: the sites are lines of `far-proxy-baking.md` and of `docs/research/lod-hlod-and-far-field.md` (:37, :39, :53, :55); no code site cites any of the 69. Downloads are in `a scratch directory/dl-F/`.

### Claim verdicts

Load-bearing claims are those ADR 0026's decision, its Settled clauses or its Considered options rest on; the rest were sampled every fifth in document order, counting only claims not already checked through a registry key (each key's citing sentences are checked in full under Registry verdicts).

| doc | claim (quoted, shortened) | load-bearing | verdict | where / evidence |
|---|---|---|---|---|
| far-proxy-baking | "One pixel is ... 14.7 cm at 256 m" (1440 rows, 45°) | yes (ADR 0026, *Its Mesh*) | confirmed | 2·tan(22.5°)/1440·256 = 0.14728 m; probe.log:10 "0.14728 m" |
| far-proxy-baking | "A building's walls are 8 triangles and its roof slab 10" | yes (5,832 cap) | confirmed | `src/scenes/world.rs:406-430` four `quad`s; `:433-444` `slab`, five quads, no bottom (`:455-472`) |
| far-proxy-baking | "324 buildings ... its shell ... 5,832 triangles" | yes (the cap) | confirmed (measured at `d4ac21eb`; not recomputable without running the generator) | probe.log:1,5-6 "most 324", Walls 2592 + Roof 3240 triangles; 324 × 18 = 5,832 |
| far-proxy-baking | "A window stands 6 cm proud of its wall ... the wall runs unbroken behind it" | yes (transfer choice X1/X2 over X3) | confirmed | `world.rs:94` window depth 0.12 centred (`:447-453`), placed on the wall plane (`:642-660`); walls are single quads |
| far-proxy-baking | "a world window is 8 by 11 pixels at the swap" | yes (ADR 0026 Considered options) | confirmed | `world.rs:94` 1.2 × 1.6 m; 1.2/0.14728 = 8.1, 1.6/0.14728 = 10.9 |
| far-proxy-baking | "D1 ... the threshold must pass 1.6 m, 11 pixels at the swap" | yes (drop by size rejected) | confirmed | largest window dimension 1.6 m, `world.rs:94` |
| far-proxy-baking | "at the cap `simplify_sloppy` moves outlines by up to 12 pixels" (also :40, :156) | yes (ADR 0026:53 repeats it) | wrong (L7-F01) | probe.log:20 merged source at the cap: farthest lost 57.3 px vs source, :64 56.6 px vs shell; 12.4 px is the shell-only row (:78) |
| far-proxy-baking | "`Prune` ... 41 px at the cap" | yes (ADR 0026:53) | confirmed | probe.log:69, :83 "41.4" on shell and merged source alike |
| far-proxy-baking | "rooflines sit 0.3 m low, 2 pixels at 256 m"; "3,240 triangles" | yes (ADR 0026 *The LOD chain*, Considered options) | confirmed | `world.rs:97` `ROOF_THICKNESS_METRES = 0.3`; 0.3/0.14728 = 2.04; probe.log:95, :101 (3,240 triangles, 0.2702 m; 4.2 px) |
| far-proxy-baking | "the finest cell is 25 cm, so 14.7 cm cannot be asked of it" | yes (rejects clustering at one pixel) | confirmed | simplifier.cpp:2023-2024, :2607 (grid ≤ 1024, error ≥ 1e-3); probe.log:11 "sloppy's finest cell 0.2504 m" |
| far-proxy-baking | "No call removes a window" at one pixel; "123,006 and 120,396 triangles" | yes (D rule needed) | confirmed | probe.log:91, :95 |
| far-proxy-baking | "the ground's 65,536 m² is 3.02 M more"; "a quarter of that at two" (ADR) | yes (ADR 0026 Considered options) | confirmed | 256² / 0.14728² = 3.021 M; probe.log:438, :446 12,016,957 − 8,992,836 = 3,024,121 |
| far-proxy-baking | "a chart on a 2^N grid with 2^(N−1) texels of gutter a side is clean ... at mip N (derived)" | yes (ADR 0026 *Its Material*: 2 on 4, mip 2) | confirmed (derivation holds: at mip N each grid cell is one texel, the gutter half a texel, so a bilinear tap inside the chart reads only its cell) | far-proxy-baking.md:64 |
| far-proxy-baking | "a 4096² Texture's Resident tail starts at its 256² mip, mip 4" | yes (B2) | confirmed | `src/assets/cooked.rs:253-268`, `src/stats/gpu_memory.rs:107` 64 KiB; 256² BC7 = 65,536 B |
| far-proxy-baking | "Albedo cooks to BC7 and metallic-roughness to BC5 ... up to 8,192 texels a side" | yes (ADR 0026 *The baked Material*) | confirmed | `src/cook/texture.rs:181,184`; `src/assets/cooked.rs:36` |
| far-proxy-baking | "The cooker creates no GPU device" | yes (CPU rasteriser, X1) | confirmed | no `Device`/`request_adapter` under `src/cook/` |
| far-proxy-baking | "A Placement belongs to the Chunk that holds its pivot, so 645 ... held by a neighbour, and 48 ..." | yes (ADR 0026 reads neighbours) | confirmed | `world.rs:501`; probe.log:8-9 |
| far-proxy-baking | padded charts "64% to mip 2, 79% to mip 3 and 114% to mip 4"; table rows | yes (B1/B2, ADR's grid of 4) | confirmed | probe.log:443-445; 10,716,112, 13,204,224, 19,149,824 of 16,777,216 |
| far-proxy-baking | "8.81 M texels at 14.7 cm"; "191,029 m²" | yes (texture pool figure) | confirmed | probe.log:5-6 162,195.2 + 28,834.1; 191,029/0.14728² = 8.81 M |
| far-proxy-baking | "against the Meshlet triangle Arena's 2,202,009 (derived, nothing cooked)" | yes (E2's fit test) | stale (L7-F02) | `src/assets/pools.rs:19` share 68‰ → 1,426,063 at the frozen commit; 2,202,009 is `d4ac21eb`'s 105‰ |
| far-proxy-baking | "227 Chunks hold 300 buildings or more" | fifth sample (5) | confirmed | probe.log:1 |
| far-proxy-baking | "a bake texel spans 75 of its texels a side" | fifth sample (10) | confirmed | `world.rs:86,88` 2048 texels over 4 m = 512/m; 0.14728 × 512 = 75.4 |
| far-proxy-baking | "GPL and Apache-2.0 (Blender)" (Comparison row 1; also :88) | fifth sample (15) | confirmed | v5.2.3 `source/blender/render/intern/bake.cc` and `editors/object/object_bake_api.cc` "SPDX-License-Identifier: GPL-2.0-or-later"; `init_from_bake.h` Apache-2.0 |
| far-proxy-baking | planar-chart rasterising with a depth test "is technique 1's inward ray for every sample at once (derived)" | fifth sample (20) | confirmed (follows from orthographic projection along the chart normal) | far-proxy-baking.md:52 |
| far-proxy-baking | "No source read clamps or corrects that at run time" | fifth sample (25) | confirmed as far as the sources read go (NVIDIA calls its clamp "only theory", p. 13) | — |
| far-proxy-baking | "3,962 of 4,096 rows" | fifth sample (30) | confirmed | probe.log:446 |
| far-proxy-baking | "xatlas ... would be a new dependency" | fifth sample (35) | confirmed | no `xatlas` in `Cargo.toml` |
| far-proxy-baking | "the reference covers 2.49 M pixels" | fifth sample (40) | confirmed | probe.log:20 2,489,416 |
| far-proxy-baking | "A box under an overhanging slab is 18 triangles" | fifth sample (45) | confirmed | `world.rs:406-472` |
| far-proxy-baking | "mip 3 suffices to 2 km"; "reaches mip 3 at 2,048 m, past the View distance" | fifth sample (50) | confirmed | 256 × 2³ = 2,048; `src/scenes/mod.rs:561` `Self::World => 2000.0` |

### Registry verdicts

Every entry has a `checked` date; every url opened (Unreal's through `gh api`, since the repository needs an Epic-linked account, which the `section` says). "Sites hold" means every citing sentence says no more than the quote and the source.

| key | registry line | verdict | what was opened, what it says, and whether each citing site holds |
|---|---|---|---|
| andersson-2007-colour-map-detail-maps | 6474 | confirmed | PDF p. 1, §5.1: quote verbatim. Site far-proxy:70 holds. |
| blender-5.2-selected-to-active | 6264 | confirmed | Manual 5.2 page, Selected to Active: all three sentences verbatim. Site :48 holds. |
| blender-5.2.3-bake-jitter | 6288 | confirmed | `init_from_bake.h` at v5.2.3 L19-L29, Apache-2.0 SPDX at L3: verbatim. Site :48 holds. |
| carr-2002-mip-mappable-atlas | 6399 | confirmed | UMBC PDF pp. 10, 13-14, 20: verbatim. Sites :62 (under half) and :64 (the quadtree, p. 14 "area-balanced quadtree") hold. |
| chen-2023-pixel-criterion | 6624 | confirmed | PDF pp. 4-5, 11, 12: quote and cost verbatim (gaps are the paper's symbols). Site :86 holds. |
| chen-2023-robust-low-poly | 6361 | confirmed | Author PDF p. 2, pp. 11-12: verbatim. Site :56 holds. |
| cignoni-1999-oversampling | 6300 | confirmed | Archived PDF p. 8 (§3.3), Table 3 p. 11 (read with `-table`: no multisampling 14.718, 3 × 3 128.441), p. 18 SGI O2: verbatim. Site :50 holds. |
| cignoni-1999-resampling-detail-textures | 6294 | confirmed | Archived PDF pp. 4-5: verbatim. Site :50 holds. |
| cohen-1998-appearance-preserving | 6307 | confirmed | PDF p. 1 (Abstract, §1.1), p. 4 (§4.2): verbatim. Site :52 holds. |
| gao-2022-low-poly-building | 6638 | confirmed | PDF pp. 1, 7, 8: verbatim. Site :86 holds. |
| gao-2022-silhouette-metric | 6631 | confirmed | PDF pp. 5-8: verbatim ("sten- cil" rejoined). Sites :86 and :92 hold. |
| garland-1997-qem | 5652 | confirmed | PDF p. 1, footnote 4 p. 5: verbatim. Sites far-proxy:80 and lod-hlod:37 (10 floats a vertex, in the cost field) hold. |
| hasselgren-2021-appearance-driven-simplification | 5695 | confirmed | arXiv v1 pp. 1, 11: verbatim. Sites far-proxy:56 and lod-hlod:37 hold. |
| jylanki-2010-maxrects | 6386 | confirmed | PDF p. 33 §5; Readme.txt at 83e7e11 "released to Public Domain", no licence file in the tree: verbatim ("asymptotically" under the gap). Sites :60, :88 hold. |
| kircher-2024-spider-man-2-distant-lods | 5752 | confirmed | Archived GDC Vault page (gzip): verbatim. Sites far-proxy:189 and lod-hlod:55 hold. |
| knodt-2023-joint-uv-texture-baking | 6257 | confirmed | PDF p. 2, p. 7, p. 12, Table 2 p. 13 ("OursUV,SH (512) 607 4504"): verbatim. Sites :48, :56 hold; 607 ms × 4,504 = 45.6 min (Comparison row 5's "46 minutes"). |
| levy-2002-lscm-tetris-packing | 6392 | confirmed | PDF p. 7 (§4, Table 2 read with `-table`), p. 8: verbatim. Site :62 holds. |
| liu-2019-bounded-packing-efficiency | 6374 | confirmed | Project page abstract: verbatim. Site :60 holds. |
| meshopt-0.6.2-simplify-options | 6576 | confirmed | Crate `src/simplify.rs` L13-L26, L112, L282, L361 (docs.rs 200): verbatim. Site :80 "six options" holds (six flags beside `None`). |
| meshoptimizer-0.25-edge-collapse-topology | 6553 | confirmed | simplifier.cpp L209-L210, L1293-L1296, L2437: verbatim. Site :80 holds. |
| meshoptimizer-0.25-gltfpack-simplify | 6547 | confirmed | gltf/mesh.cpp L911, L914-L917, L947, L950: verbatim. Sites :78 (10% error) and :82 hold. |
| meshoptimizer-0.25-licence | 6645 | confirmed | LICENSE.md L1-L3: verbatim. Site :88 holds. |
| meshoptimizer-0.25-prune-criterion | 6582 | confirmed | simplifier.cpp L1869-L1959, L2475-L2489: verbatim. Site :82 holds (components over the position remap, `buildComponents` L1813). |
| meshoptimizer-0.25-simplify-prune-function | 6588 | confirmed | README L550: verbatim; the crate binds `meshopt_simplifyPrune` only in `gen/bindings.rs:439` (`pub mod ffi`). Site :82 holds. |
| meshoptimizer-0.25-simplify-sloppy | 5796 | confirmed | README L452, L544, L716 (experimental list): verbatim. Sites far-proxy:78 and lod-hlod:55 hold. |
| meshoptimizer-0.25-sloppy-declaration | 6541 | confirmed | meshoptimizer.h L498-L511: verbatim. Site :78 ("takes no options") holds. |
| meshoptimizer-0.25-sloppy-grid | 6535 | confirmed | simplifier.cpp L2022-L2040, L2589-L2590, L2607-L2609, L2625-L2626: verbatim. Site :78 holds. |
| meshoptimizer-0.25-sloppy-output | 6529 | confirmed | simplifier.cpp L2172-L2187, L2217-L2260, L2695, L2706-L2707: verbatim. Site :78 holds. |
| meshoptimizer-1.3-voxel-remesh | 5820 | confirmed | v1.3 README L691, L704, L1003: verbatim; absent from the v1.0, v1.1, v1.2 READMEs, so it "arrives in 1.3". Sites lod-hlod:55 and far-proxy:86 hold; ":86" calls it "0.7.0's", which the quote does not state but `meshopt` 0.7.0 bears out (`vendor/src/meshoptimizer.h:15` version 1030, `src/remesh.rs:30`). |
| moore-2018-fc5-baked-albedo | 6500 | confirmed | PDF pp. 14, 17, 80: verbatim. Site :72 holds (64/132 = 0.48 m). |
| nvdiffmodeling-licence | 5869 | confirmed | LICENSE.txt at 9b2ba2e §3.3: verbatim. Sites far-proxy:56 and lod-hlod:37 hold. |
| nvidia-2004-atlas-lower-mip-bleed | 6425 | confirmed | PDF pp. 13-14: verbatim. Sites :64, :66 hold. |
| nvidia-2004-atlas-power-of-two-lines | 6431 | confirmed | PDF p. 5: verbatim (the gap takes "box-" of "box-filter"). Site :64 holds; its "only while" reads NVIDIA's sufficient condition as a necessary one, within reason. The paper's own footers carry both WP-01386 and WP-01387. |
| ray-2010-invisible-seams-mip-levels | 6444 | confirmed | EG diglib PDF pp. 6-7: verbatim. Site :64 holds. |
| rossignac-1993-vertex-clustering | 6523 | confirmed | Scan, page images 2-3 read: "an elongated solid may be approximated by a curve segment. However, the error introduced by the simplification is bounded (in the Hausdorf distance sense) by a user-controlled accuracy factor"; §3.1 and §3.3 sentences verbatim. Site :78 holds. |
| sander-2001-tmpm-gutter-pull-push | 6419 | confirmed | PDF pp. 6-7: verbatim. Sites :64, :66 hold. |
| sander-2001-tmpm-packing-efficiency | 6368 | confirmed | PDF p. 5, Table 1 p. 6, p. 7: verbatim. Site :60 holds. |
| simplygon-10.1-remeshing | 5899 | confirmed | Archived 10.1.400.0 page: verbatim. Sites far-proxy:86 and lod-hlod:55 hold. |
| simplygon-10.4-aggregation-tiled | 6355 | confirmed | 10.4.490.0 page: verbatim. Site :54 holds. |
| simplygon-10.4-caster-layers | 6337 | confirmed | 10.4.490.0 page: verbatim. Site :52 holds. |
| simplygon-10.4-dilation | 6468 | confirmed | 10.4.199.0 ColorCasterSettings: verbatim, "Default: 10". Site :66 holds. |
| simplygon-10.4-gutter-space | 6462 | confirmed | 10.4.199.0 MappingImageOutputMaterialSettings: verbatim, "Default: 4". Site :66 holds. |
| simplygon-10.4-mapping-image | 6331 | confirmed | 10.4.490.0 page: verbatim. Site :52 holds. |
| simplygon-10.4-on-screen-size | 5905 | confirmed | 10.4.117.0 ReductionSettings: verbatim. Sites far-proxy:84 and lod-hlod:39 hold. |
| simplygon-10.4-part-remover | 6600 | confirmed | 10.4.117.0 Part remover: verbatim. Site :84 holds. |
| ue-5.8-approximate-bake-settings | 6319 | confirmed | MeshApproximationSettings.h L222-L228: verbatim. Site :52 holds. |
| ue-5.8-approximate-photo-set-sample | 6313 | confirmed | SpatialPhotoSet.h L100-L105, L222: verbatim. Site :52 holds. |
| ue-5.8-approximate-simplify-policy | 6618 | confirmed | MeshApproximationSettings.h L128-L142: verbatim. Site :86 holds. |
| ue-5.8-approximate-tiny-parts | 6594 | confirmed | MeshApproximationSettings.h L89-L95: verbatim. Site :84 holds. |
| ue-5.8-hlod-approximate-voxel | 5974 | confirmed | MeshApproximationSettings.h L73-L79: verbatim. Sites far-proxy:86 and lod-hlod:55 hold. |
| ue-5.8-hlod-layer-types | 5988 | confirmed | HLODLayer.h L31-L39: verbatim. Sites far-proxy:52 and lod-hlod:53 hold (the latter names four of the six, leaving out the two custom builders). |
| ue-5.8-hlod-simplify-screen-size | 6612 | confirmed | HLODBuilderMeshSimplify.cpp L82-L104: verbatim. Site :84 holds. |
| ue-5.8-landscape-hlod-bake | 6487 | confirmed | LandscapeHLODBuilder.cpp L243-L294, L297, L379-L394: verbatim. Site :72 holds. |
| ue-5.8-material-proxy-defaults | 6450 | confirmed | MaterialMerging.cpp L15-L21: verbatim. Site :66 holds. |
| ue-5.8-material-proxy-texture-sizing | 6325 | confirmed | MaterialMerging.h L21: verbatim. Site :52 holds. |
| ue-5.8-merge-unique-uv-bake | 6343 | confirmed | MeshMergeUtilities.cpp L2847-L2848, L3117-L3118: verbatim. Site :54 holds. |
| ue-5.8-proxylod-hit-selection | 6276 | confirmed | ProxyLODMaterialTransferUtilities.cpp L242, L288, L291: verbatim. Site :48 holds. |
| ue-5.8-proxylod-ray-correspondence | 6270 | confirmed | ProxyLODMaterialTransferUtilities.h L30-L32, L102-L103: verbatim. Site :48 holds. |
| ue-5.8-proxylod-supersampling | 6282 | confirmed | ProxyLODPlugin.cpp L990, L1055: verbatim. Site :48 holds. |
| unity-hlodsystem-04be7e8-min-object-size | 6606 | confirmed | GetMeshRenderer.cs L77-L79: verbatim. Site :84 holds. |
| unity-hlodsystem-04be7e8-readme | 6059 | confirmed | README L8, L18-L22, L65: verbatim. Sites far-proxy:88 and lod-hlod:53 hold. |
| unity-hlodsystem-04be7e8-simple-batcher | 6349 | confirmed | Batcher.md `## SimpleBatcher` L11, L15-L19: verbatim. Site :54 holds. |
| unity-hlodsystem-04be7e8-terrain-hlod | 6512 | confirmed | TerrainHLOD.md L3, L29-L45: verbatim. Site :72 holds. |
| vanwaveren-2012-svt-border | 2714 | confirmed | PDF p. 10 §3.2: verbatim. Site :66 holds. |
| vanwaveren-2012-svt-border-per-mip | 6437 | confirmed | PDF pp. 9-10, p. 18: verbatim. Site :64 holds. |
| werle-2017-wildlands-vista-texture | 6506 | confirmed | PDF pp. 33-34: verbatim (guillemets). Site :72 holds. |
| widmark-2012-bf3-terrain-virtual-texture | 6480 | confirmed | PDF pp. 31, 32, 38: verbatim. Site :70 holds. |
| xatlas-f700c77-pack-options | 6405 | confirmed | xatlas.h L197-L232 at f700c77; LICENSE "MIT License / Copyright (c) 2018-2020 Jonathan Young": verbatim. Sites :62, :88 hold. |
| xatlas-f700c77-planar-charts | 6380 | confirmed | xatlas.cpp L5174, L5248: verbatim. Site :60 holds. |

## `docs/research/lod-hlod-and-far-field.md`

I read the note in full at the frozen commit `62052fe809eb856c0e0376bb7bcccb84c7f6afcd` in the frozen checkout, with M5's diff of it (`git diff 6ebb45fb..62052fe8`, word-level: the culling figures, RedLynx's attribution, Horizon's tile rings, Unreal's voxel accuracy, Frostbite's hair figures, option C1 and two Not verified lines), and ADRs 0026 and 0027 and ADR 0013's amended guard to tell which of its claims are load-bearing. No scoped ADR cites the note by path; I took as load-bearing the claims ADR 0026's pixel and Chunk figures, ADR 0027's carrier and five levels, ADR 0013's per-slot guard and ADR 0021's floor rest on. Repo claims were checked against `src/camera/mod.rs`, `src/assets/cooked.rs`, `src/render/{targets,main_pass}.rs`, `src/render/shaders/{main,culling,common}.wgsl`, `src/gpu/mod.rs`, `src/cook/mesh.rs`, `src/render/renderer.rs`, `src/scenes/town.rs`, `Cargo.lock`, `bench/baselines/apple-m4-pro/world.json`, `docs/performance.md` (the 2026-10-30 culling section, "Each stage on its own" and "Against the sources"), `docs/audit/m3.5/performance.md` (option I) and `docs/audit/m3.5/scalability.md` (line 90 and R09). Each of my 42 registry entries was opened at its source: GitHub files as raw files at the pinned tag or commit (Bevy v0.20.1, Godot 4.4-stable, godot-docs `db50fcf`, meshoptimizer v0.25, ImpostorBaker `c75d365`, Unity Graphics `0efc28e`, gpuweb `688f043`, Unreal 5.8.4 through `gh api`), issues through `gh api` (cesium-unreal 1104, gpuweb 2229), PDFs with `pdftotext` at the cited page (Decima GCAP 2017, Haar and Aaltonen 2015, Karis 2021, Frostbite 2019, Fahner 2023, Fatahalian 2010, Hoppe 1996 and 1999, Meister 2021, Leblanc and Conte 2025, Moore 2018 from its archive, MSL 4.1 from its archive, Décoret 2003 from a 2024 archive of the cited HAL url, whose live page serves curl a bot challenge), the Activision deck as its slide notes unzipped from the `.pptx`, image-only slides (Decima p. 46, the colour legend of Haar and Aaltonen p. 13) by extracting the embedded image or reading the page's fill colours from its content stream, and web pages live or at the pinned `web.archive.org` snapshot (Shader Bits, Cesium blog, Golus, Misiak, NVIDIA blog, Godot 4.5 docs); Erikson's title, venue and abstract came from Crossref and OpenAlex. All downloads sit in `a scratch directory/dl-L/`. Every citing site of each key in the scoped documents (the note, `docs/research/acceleration-structures-for-streamed-worlds.md:52`, `:56`, `:58`) and in code (`src/cook/mesh.rs:299`, `src/render/renderer.rs:192`, `src/render/shaders/culling.wgsl:282`) was read against the quote and the source. No other scoped document cites any of the 42.

### Claim verdicts

Remaining claims were numbered R1 to R174 in document order, clause by clause, a table row counting as one or two; the fifth samples are R5, R10, ... R170. Two samples were stale or narrower than their source (R25, R70), which opened full checks of "Where Praxis stands" and of item 4; their extra rows are marked "section check".

| doc | claim (quoted, shortened) | load-bearing | verdict | where / evidence |
|---|---|---|---|---|
| note | "alpha draws its samples from one nested family, so the two levels' masks overlap instead of complementing" | yes (the note's case against alpha-to-coverage, ADR 0027's carrier) | unsupported (L7-L04) | `:5`; the WebGPU spec says the mask "is platform-dependent and can vary for different pixels" and does not even guarantee a monotonic count; the note's own `:104` leaves it unverified |
| note | "Praxis's flat cull tests a slot in 0.79 ns in its first phase, about 0.7 to 6.5 times the published first-phase figures" | yes (ADR 0013's guard) | confirmed | `:5`; 200.6 µs / 254,256 = 0.789 ns; against 108 µs / 896,322 = 0.1205 ns, 6.55; against 0.28 ms / 250,000 = 1.12 ns, 0.70 |
| note | "`meshopt` 0.6.2, already a dependency, provides the simplifier, the error and the selection formula" | yes (ADR 0026's cook) | confirmed | `:5`; `Cargo.lock:1352-1353`; meshopt 0.6.2 vendors meshoptimizer 0.25 (`vendor/src/meshoptimizer.h:15`, `MESHOPTIMIZER_VERSION 250`) |
| note | per-slot stages "200.6 µs in phase one and 171.7 µs in phase two over `town`'s 254,256 slots in use, 0.79 and 0.68 ns a slot, 1.46 ns for both" | yes (ADR 0013's guard) | confirmed | `:11`; `docs/performance.md:1710-1717` (145.2 + 7.5 + 47.9 = 200.6; 151.6 + 7.7 + 12.4 = 171.7), `:1744` (254,256) |
| note | "the 3.14 M Instances a 2 km view holds at `town`'s density would take 4.6 ms of them" | yes | confirmed | `:11`; `docs/audit/m3.5/scalability.md:90` ("**3.14 M**"); 3.14 M × 1.464 ns = 4.6 ms |
| note | "The 4.5 ns a slot and 14.1 ms of ... R09 divide phase one's whole line" | yes | confirmed | `:11`; `scalability.md:119` ("1,145.25 µs for town's 254,256 slots is 4.50 ns per slot ... about 14.1 ms") |
| note | "At 1440p with Praxis's 45° vertical field of view a pixel spans 0.575 mrad" | yes (ADR 0026:9's 14.7 cm at 256 m) | confirmed | `:11`; `src/camera/mod.rs:90` (`FRAC_PI_4`); 2 tan 22.5° / 1440 = 0.5753 mrad; × 256 m = 14.73 cm |
| note | "The main pass runs 4x MSAA with alpha-to-coverage off" | yes (ADR 0027) | confirmed | `:11`; `src/render/targets.rs:16`, `src/render/main_pass.rs:113` |
| note | "With 4x MSAA, alpha-to-coverage gives five levels" | yes (ADR 0027's five levels) | confirmed | `:47`; golus-2017-alpha-to-coverage, "5 levels of opacity you can play with, 4/4 through 0/4 samples" |
| note | "256 m is the size of Praxis's Chunks" | yes (ADR 0026) | confirmed | `:53`; `src/assets/cooked.rs:58` (`CHUNK_EDGE_METRES: f64 = 256.0`) |
| note | "both phases' 372 µs compare with RedLynx's 670 µs of object and cluster culling" | yes | confirmed | `:63`; 200.6 + 171.7 = 372.3; 0.28 + 0.26 + 0.09 + 0.04 = 0.67 ms (haar-aaltonen-benchmark p. 54) |
| note | "about 0.12 ns an Instance"; "1.2 ns each"; "about 1.1 ns an object" | yes | confirmed | `:63`; 108/896,322, 125/102,804, 0.28 ms/250,000; the CullRasterize lines sum to 2,237 µs (Karis p. 110) |
| note | "Indirect dispatch is on the Capability floor"; "Praxis's indirect buffers already rest on the Capability floor" | yes (ADR 0021) | confirmed | `:29`, `:67`; `src/gpu/mod.rs:294-296` (`DownlevelFlags::INDIRECT_EXECUTION` in the floor) |
| note | C1: "the per-slot stages cost 1.46 ns a slot for both phases, and the 4.5 ns ... is phase one's whole line" | yes (ADR 0013:5) | confirmed | `:94`; as the rows above |
| note | Not verified: "What a `sample_mask` output in the depth pre-pass costs on Apple's tile-based GPU" | yes (ADR 0027) | stale (L7-L09) | `:105`; ADR 0027 records the measure on the M4 Pro |
| note | Not verified: "their cost on `world`, whose slots in use no Baseline records" | yes (ADR 0013's guard) | stale (L7-L10) | `:108`; `bench/baselines/apple-m4-pro/world.json` records `instance_slots_in_use` 307,263 and `culling_per_slot_microseconds` |
| note | R5 "Godot says its noise shows less with TAA" | no, fifth sample | confirmed | `:5`; godot-4.5-dithered-lod, "This may not be as noticeable ... when temporal antialiasing is enabled" |
| note | R10 "octahedral impostors blend their views within the frame and need no history" | no, fifth sample | confirmed | `:5`; brucks-2018-octahedral-impostors, "identify the 3 nearest frames" and blend |
| note | R15 "The note ends in options for M5 and chooses none" | no, fifth sample | confirmed | `:79` "Options for M5 (no recommendation)" |
| note | R19 "drawing half the indices takes about 2.8 ms off its 8.6 ms gate (1.1 ms depth pre-pass, 1.4 ms lit pass, 0.3 ms culling...)" | no, section check | confirmed | `:11`; `docs/audit/m3.5/performance.md:504` (option I), `:443` (8.6 ms) |
| note | R20 "The culling pass tests every Instance slot" | no, fifth sample | confirmed | `:11`; `scalability.md:119` (`offset_slots` dispatches slots in use / 256); the flat cull stands |
| note | R21, R22, R23: wire "one pixel wide at 52 m and a quarter pixel at 209 m"; "a 1 m object at 2 km covers 0.87 px" | no, section check | confirmed | `:11`; `src/scenes/town.rs:1180` (radius 0.015 m, 3 cm); 0.03/0.0005753 = 52.1 m; /4 → 208.6 m; 1/(2000 × 0.0005753) = 0.869 |
| note | R24 "with them off it has no fragment stage" | no, section check | stale (L7-L05) | `:11`; the depth pre-pass's twin runs `fs_thin_depth` with rays off (`src/render/main_pass.rs:163-167`) |
| note | R25 "phase two's depth draw in the lit pass runs the empty `fs_depth`" | no, fifth sample | stale (L7-L05) | `:11`; true of the ordinary pipeline (`main_pass.rs:177-181`, `main.wgsl:82`), but its twin runs `fs_thin_depth`, which writes `sample_mask` (`main_pass.rs:182-186`, `main.wgsl:187`) |
| note | R26 "The lit pass already reads `@builtin(sample_mask)` as input" | no, section check | confirmed | `:11`; `src/render/shaders/main.wgsl:299` |
| note | R30 row 2 fit: "One more test in the culling pass" | no, fifth sample | confirmed | `:18`; `src/render/shaders/culling.wgsl:282-300` (`instance_lod`) |
| note | R35 row 6: "meshoptimizer 0.25 gives no ancestor map" | no, fifth sample | confirmed | `:22`; meshoptimizer v0.25 `src/meshoptimizer.h:438`, "The resulting index buffer references vertices from the original vertex buffer" |
| note | R40 row 11: "Praxis 0.79 ns ... 1.46 ns ...; Nanite about 0.12 ns and RedLynx about 1.1 ns (computed)"; "4.6 ms ... at 3.14 M" | no, fifth sample | confirmed | `:27`; computed above |
| note | R45 row 16 licences: "MIT (TressFX); Persson's demo needs approval" | no, fifth sample | confirmed | `:32`; `gh api repos/GPUOpen-Effects/TressFX` licence MIT; `PhoneWireAA/App.cpp:17` (archive), "For use in anything commercial, please request my approval" |
| note | R50 "meshoptimizer targets an error normalized to the mesh's extent" | no, fifth sample | confirmed | `:37`; meshoptimizer v0.25 README:446, "distance normalized to `[0..1]` range ... 1% of the mesh extents" |
| note | R55 "Godot simplifies with meshoptimizer bottom-up, stopping at 75% of the source" | no, fifth sample | confirmed | `:37`; Godot 4.4-stable `importer_mesh.cpp:450`, `:482` |
| note | R60 "Godot's default threshold of one pixel is 'perceptually lossless'" | no, fifth sample | confirmed | `:39`; godot-docs `mesh_lod.rst:153-154` |
| note | R65 "Godot's instant mode uses hysteresis so a camera at the boundary does not flip levels back and forth" | no, fifth sample | confirmed | `:41`; godot-docs `visibility_ranges.rst:111-113` |
| note | R70 "Both levels draw during the fade, weighted 1 to 0 and 0 to 1, usually through screen-space dithering" | no, fifth sample | unsupported (L7-L06) | `:45`; the Unity manual's "usually" covers "either screen-space dithering or transparency" |
| note | R71 "Unity's URP offers a Bayer matrix, blue noise or a 2×2 stencil" | no, section check | confirmed | `:45`; URP 17.0 manual, Universal Render Pipeline Asset, LOD Cross Fade Dithering Type |
| note | R72 "its core library keeps the pattern static because a moving one 'confuses the TAA'" | no, section check | confirmed | `:45`; Unity Graphics `0efc28e` `Common.hlsl:1679-1680` |
| note | R73, R75, R76, R77 Bevy, Cesium (both), Godot | no, section check | confirmed | `:45`; see the registry rows |
| note | R74 "Unreal's dithered transition is a material switch" | no, section check | confirmed | `:45`; UE 5.8 Material Properties, "Dithered LOD Transition Whether meshes rendered with this material should support dithered LOD transitions" |
| note | R78 "No source publishes its GPU time" | no, section check | not checkable (a search claim) | `:45` |
| note | R80 "the mask it derives from alpha is platform-dependent and may vary by pixel" | no, fifth sample | confirmed | `:47`; gpuweb `688f043` `spec/index.bs:17018` |
| note | R85 "varied per pixel over the standard sample positions ... leaves no hole and no double cover" | no, fifth sample | confirmed (reasoning on the cited pattern) | `:47`; gpuweb `688f043` `spec/index.bs:16576-16597`, the 4-sample positions |
| note | R90 "CDLOD morphs terrain grids in the vertex shader with no seams, between two levels per node" | no, fifth sample | confirmed | `:49`; CDLOD paper p. 7 ("no seams or T-junctions"), p. 9 ("only support transition between two LOD layers") |
| note | R95 "its layer types are instancing, merged, simplified and approximated (voxel) meshes" | no, fifth sample | confirmed | `:53`; UE 5.8.4 `HLODLayer.h:31-39` via `gh api` |
| note | R100 HZD tiles "9 high, 9 medium and 9 low around the player and 12 very low always loaded" | no, fifth sample | confirmed | `:53`; Fahner p. 38 |
| note | R105 "Insomniac automated Spider-Man 2's distant-city LODs when its manual pipeline was no longer viable" | no, fifth sample | confirmed | `:55`; GDC Vault overview (archive 20260818170518) |
| note | R110 billboard Dither, "not an option for all platforms", "look smeary" | no, fifth sample | confirmed | `:57`; Shader Bits archive |
| note | R115 "No shipped engine was found using them" | no, fifth sample | not checkable (a search claim) | `:59` |
| note | R120 "instances are the new triangles" | no, fifth sample | confirmed | `:63`; Karis p. 95 notes |
| note | R125 Activision "quickly became the bottleneck", AABB hierarchies of model collections | no, fifth sample | confirmed | `:65`; deck notes, slide 19 |
| note | R130 empty `ExecuteIndirect` "about 90 ns on average, spiking to 500 ns or worse on a cache miss" | no, fifth sample | confirmed | `:67`; Wihlidal GDC 2024 p. 114 |
| note | R135 Vulkan and Metal "say almost nothing" about forward progress | no, fifth sample | confirmed | `:69`; arXiv 2109.06132v1 p. 1 |
| note | R140 "Unreal maps object size to a cull distance and warns of popping" | no, fifth sample | confirmed | `:73`; UE 5.6 Cull Distance Volumes page |
| note | R145 phone-wire AA clamps to a pixel, fades by the lost width, suggests railings and antennas | no, fifth sample | confirmed | `:75`; Humus page (archive 20260515045315) |
| note | R150 "Its width depends on the distance, so the vertex shader needs the wire's axis" | no, fifth sample | confirmed (reasoning; the code widens in `vs_thin` about the centre line) | `:75`; ADR 0027:11 |
| note | R155 "Far proxies in the near field's TLAS cost Epic 44% of near-field traversal" | no, fifth sample | confirmed | `:77`; Wright 2022 p. 123, "a show-stopping 44% penalty to all near-field traversal costs" |
| note | R160 T1 to T4 | no, fifth sample | confirmed (options; T3's "prototyped first" holds) | `:84-87` |
| note | R165 W1 to W3 | no, fifth sample | confirmed (options) | `:98` |
| note | R170 "Flat against hierarchical culling on one scene; no source compares them" | no, fifth sample | not checkable (a search claim) | `:106` |

Further claims checked through the registry rows below (every sentence citing one of my 42 keys): `:37` (4 sentences), `:39` (2), `:41` (2), `:45` (4), `:47` (2), `:49` (1), `:53` (4), `:57` (6), `:59` (1), `:63` (4), `:65` (6), `:67` (1), `:69` (2), `:73` (1), `:75` (1), `:77` (2); all hold but `:65`'s Meister sentence (L7-L08). Not separately counted: `:47`'s "at a cost unmeasured" (L7-L07), found reading item 5 for its registry keys.

### Registry verdicts

| key | registry line | verdict | what was opened, what it says, and whether each citing site holds |
|---|---|---|---|
| activision-2021-collection-hierarchy | `docs/references.toml:5552` | confirmed | The `.pptx` (56 MB) unzipped: `notesSlide10.xml` (page 10) "And finally, divide all work units into a hierarchy that allows progressive workload expansion, with interstage culling."; `notesSlide19.xml` (page 19) holds the rest verbatim. Note `:65` holds. |
| bevy-0.20-visibility-range-dither | `:5558` | confirmed | Raw `pbr_functions.wesl` at v0.20.1, lines 66-91 verbatim ("The `dither` value ranges from [-16, 16]", `% 4u`); "the standard 4x4 ordered dithering pattern" at line 50 backs the note's "4×4 ordered". Note `:45` holds. |
| brucks-2018-billboard-dither-taa | `:5570` | confirmed | Archive 20180322112218 of the Shader Bits post (dated 2018-03-18 in its metadata), Traditional Billboards paragraphs verbatim. Note `:57` holds. |
| brucks-2018-impostor-hlod-cost | `:5576` | confirmed | Same archive, HLOD section: quote and cost verbatim ("~270,000 verts", "around 600 draw calls", "around 500k triangles", "6.8mb of textures", "341kb each"). Note `:57` holds. |
| brucks-2018-octahedral-impostors | `:5583` | confirmed | Same archive: Hemi-Octahedron, "the 3 nearest frames", "12x12 layout", "only an 8x8 layout" verbatim. Note `:57` holds. |
| brucks-impostorbaker-c75d365 | `:5589` | confirmed | Raw `ImpostorBaker.uplugin` at `c75d365`: Description and CreatedBy verbatim; `gh api` repo licence null, tree at the commit has no LICENSE file, README one line. Note `:57` holds. |
| cesium-2022-dithered-lod-transitions | `:5595` | confirmed | Archive 20260909235459 (gzip): post by Nithin Pranesh, October 20, 2022; the quoted sentences verbatim (the source's curly quotes about "Dither Temporal AA" are straight in the registry). Note `:45` holds. |
| cesium-unreal-1104-dither-msaa | `:5601` | confirmed | `gh api` issue 1104: kring, 2023-05-23, title as cited, sentence verbatim. Note `:45` holds. |
| decima-2017-cluster-sort-key | `:5607` | confirmed | GCAP 2017 PDF p. 26 notes verbatim, the source's "maxmimum" and "degreee" kept. Note `:65` holds. |
| decima-2017-collapsed-lods | `:5613` | confirmed | PDF pp. 19, 20 and 30 notes verbatim. Note `:53` holds. |
| decima-2017-cpu-hierarchy-gpu-jobs | `:5619` | confirmed | PDF p. 28 notes verbatim. Note `:65` holds. |
| decima-2017-query-time | `:5625` | confirmed | PDF p. 46 is a slide image (printed 45); extracted and read: "Typically have 500K – 1.5M StaticInstances / StaticScene GPU data uses ~10-30MB / Main static query time is ~1-2ms / Some of that time the CPU busy-waits for GPU / Shadow query times generally less", verbatim; p. 18 notes name "the PS4's asynchronous compute". Note `:65` holds ("including CPU waits"). |
| decoret-2003-billboard-clouds | `:5632` | confirmed | The live HAL url serves curl a bot challenge page; its archive 20240416120959 opened: abstract p. 2, Figure 10 caption p. 6 ("2 GHz Pentium IV processor with NVidia GeForce 4 Ti4600 with 128MB"), Section 6 p. 7 ("limited by fill rates"), Table 1 p. 8 (31: 0.068, 60: 0.072, 416: 0.220, 552: 0.208; "about 128 128") verbatim. Note `:59` holds. |
| erikson-2001-hlod | `:3344` | confirmed (form Finding L7-L01) | Crossref: title, "Proceedings of the 2001 symposium on Interactive 3D graphics", pp. 111-120, Erikson, Manocha, Baxter. The quote is the title and venue only; note `:53` "replace groups of objects with merged simplifications" is backed by the abstract (OpenAlex: HLODs "serve as higher fidelity drastic simplifications of entire branches of the scene graph"), not by the quote. |
| fahner-2023-hzd-tile-rings | `:5073` | confirmed | PDF p. 38 verbatim; author in the PDF metadata janjaap.fahner@guerrilla-games.com, created 2023-03-22 (the registry's 2023-03-21 publication date not checkable: the Guerrilla page returned 504). Note `:53` holds. |
| fatahalian-2010-small-triangle-overshade | `:5639` | confirmed | PDF p. 1: abstract and Figure 1 caption verbatim. Note `:73` holds. |
| frostbite-2019-strand-pixel-width | `:5645` | confirmed (form Finding L7-L03) | PDF pp. 28, 29, 30, 34 notes verbatim; author Sebastian Tafuri (p. 1); p. 34 table "Long hair 10 000 240 000 ... Physics 4.7ms 0.4ms Render 4.5ms 1.9ms". The cost inserts "[strands]", "[points]", "[physics]", "[render]", which are not in the source. Note `:75` and table row 16 hold. |
| godot-4.4-lod-generation | `:5659` | confirmed | Raw `importer_mesh.cpp` at 4.4-stable lines 450, 453, 482, 503, 506 verbatim; normal weights 2.0 (`:442-445`) and `SIMPLIFY_LOCK_BORDER` (`:461`) as the section says. Note `:37` holds. |
| godot-4.4-mesh-lod-threshold | `:5665` | confirmed | Raw `mesh_lod.rst` at `db50fcf` (the 4.4 branch head), lines 153-156, 172, 185-190 verbatim. Note `:39` holds. |
| godot-4.4-visibility-range-hysteresis | `:5671` | confirmed | Raw `visibility_ranges.rst` at `db50fcf`, lines 111-116 verbatim. Note `:41` holds. |
| godot-4.5-dithered-lod | `:5677` | confirmed | Live 4.5 docs page, "Use dithering for LOD transitions": verbatim. Note `:45` holds. |
| golus-2017-alpha-to-coverage | `:5683` | confirmed | Archive 20261016230034, dated Aug 12, 2017: the three passages verbatim (the source's "it’s" curly). Note `:47` holds. |
| haar-aaltonen-2015-coarse-cpu-cull | `:5689` | confirmed | PDF p. 13, Haar's Assassin's Creed Unity section (pp. 3-38): the stage labels verbatim; the legend's colours read from the page's content stream: light blue (0.627 0.702 0.847) beside "- CPU", dark red (0.439 0 0.114) beside "- GPU"; COARSE FRUSTUM CULLING is light blue, INSTANCE CULLING dark red. Note `:65` holds. |
| haar-aaltonen-benchmark | `:4025` | confirmed | PDF pp. 39, 51, 53, 54 verbatim, cost table as "cell: cell". Sites hold: note `:18` (row 2), `:39` ("0.28 ms ... first phase only"), `:63` (1.12 ns, and 670 µs from the four object and cluster cells). |
| hoppe-1996-geomorph | `:5702` | confirmed | `pm.pdf` p. 3, §3.2 verbatim (the ancestor sentence in parentheses). Note `:49` holds. |
| hoppe-1996-progressive-meshes | `:5708` | confirmed | `pm.pdf` p. 1, §1 verbatim. Note `:41` holds. |
| hoppe-1999-attribute-quadrics | `:5714` | confirmed | `newqem.pdf` p. 1 abstract verbatim. Note `:37` holds. |
| karis-2021-hierarchy-dependent-dispatches | `:5720` | confirmed | Nanite PDF p. 71 notes verbatim. Note `:67` holds. |
| karis-2021-instance-cull-cost | `:5726` | confirmed (form Finding L7-L02) | PDF p. 109 table (896322, 3668; 102804, 365) and p. 110 ("InstanceCull 108ms", "ClusterCull 406us", "Post Pass: InstanceCull 125us", "~2.5ms"); no hardware on either slide. The quote gives the p. 109 table as running text, not as "cell: cell". Note `:63` holds (lines sum to 2,237 µs). |
| karis-2021-instance-cull-single-dispatch | `:5733` | confirmed | PDF p. 15 slide and notes verbatim. Note `:63` holds. |
| karis-2021-instances-new-triangles | `:5739` | confirmed | PDF p. 95 notes verbatim. Note `:63` holds. |
| karis-2021-persistent-threads-scheduling | `:5745` | confirmed | PDF p. 73 bullets and "10-60% saving (typically ~25%)" verbatim. Note `:69` and row 14 hold. |
| leblanc-conte-2025-acs-far-cubemap | `:4616` | confirmed | PDF p. 72 (printed 71) notes verbatim. Note `:77` and `acceleration-structures-for-streamed-worlds.md:56` hold. |
| levien-2021-gpuweb-no-progress-guarantee | `:5758` | confirmed | `gh api` issue 2229: raphlinus, 2021-10-30, body line 33 verbatim. Note `:69` holds. |
| lloyd-2020-stochastic-lod-dxr | `:4634` | confirmed | Live NVIDIA blog, Jun 15, 2020: quote and cost verbatim ("RTX 2080Ti with 64×60 instances", "0.018 ms", "0.22 ms", "about 10%", "1.6%"). Note `:77`, survey 4 `:58` (both citations) hold. |
| meister-2021-bvh-star-two-level | `:4647` | confirmed; note `:65` unsupported (L7-L08) | PDF p. 11, §5.3.3 verbatim. Survey 4 `:52` holds (ray tracing). Note `:65` applies it to cell culling for raster without saying the source is about rays. |
| meshoptimizer-0.25-attribute-simplify | `:5764` | confirmed | Raw README at v0.25, lines 467 (Attribute-aware) and 514 (Permissive) verbatim. Note `:37` holds. |
| meshoptimizer-0.25-lod-chain | `:5770` | confirmed | README lines 521, 533, 554 verbatim (521 and 533 sit under "Simplification with vertex update"). Note `:37` (twice) and `src/cook/mesh.rs:299` (errors summed down the chain, `:375`) hold. |
| meshoptimizer-0.25-lod-selection | `:5776` | confirmed | README lines 454-460 verbatim. Note `:37`, `:39`, `src/render/renderer.rs:192` (2 tan(fov/2)/height, `:199-200`) and `culling.wgsl:282` (distance to the bounds' sphere, `common.wgsl:216`) hold. |
| misiak-2021-xr-impostors | `:5826` | confirmed | Archive 20260715231257 of the group page: abstract verbatim, VRST 2021, doi 10.1145/3489849.3489865. Note `:57` holds. |
| moore-2018-fc5-terrain-quadtree | `:5832` | confirmed | Archive 20260513233252 PDF pp. 4, 14, 15, 22 verbatim. Note `:53` holds. |
| msl-4.1-sample-mask-output | `:5839` | confirmed | Archive 20260925145438 PDF, Version 4.1, 2026-06-04; p. 151 §5.2.3.5 Table 5.7, "sample_mask uint Coverage mask. All OS: Metal 1 and later." Note `:47` holds. |

## Registry entries cited by the LOD, HLOD and far-field note (reader M)

I opened all 41 assigned entries in full at frozen commit `62052fe8`, none sampled, and checked each one's url, section, quote and `checked` date. To find the citing sites I ran `grep -rn "ref:<key>]"` over `docs/`, `CONTEXT.md`, `CLAUDE.md` and `ROADMAP.md`, plus a bare-key grep over `src`, `tests` and `scripts`, then kept the sites in the scoped documents. They are `docs/research/lod-hlod-and-far-field.md` L37-L77 (43 citations), `docs/research/acceleration-structures-for-streamed-worlds.md` L44, L56 and L58 (4 citations), and one code site, `src/render/shaders/light_count.wgsl:9`. None of the 41 keys is cited by ADR 0026, ADR 0027, any M5 amendment (`git diff -U0 6ebb45fb..62052fe8 -- docs/adr`), or `docs/research/far-proxy-baking.md`. ADR 0019 L9 cites `nanite-taa-lod` in an unamended clause, and ADR 0021 L14 cites `wgpu-core-30-indirect-required`; both are out of scope. Sources were opened as follows. GitHub sources (niagara, gpuweb, wgpu, TressFX, OpenSceneGraph, Unity Graphics, vk_lod_clusters, CDLOD) were read as raw files at the pinned commit or tag. The wgpu types and wgpu-core entries were also compared with the locked 30.0.2 crates in `~/.cargo/registry` and `vendor/wgpu-core`. The Unreal Engine 5.8.4 sources came through `gh api repos/EpicGames/UnrealEngine/contents/...?ref=5.8.4-release`. PDFs were read with `pdftotext` at the cited page: the MSL 4.1 spec, Nanite 2021, Lumen 2022, Persson 2012, Sorensen 2021, CDLOD and Wihlidal 2024. The Netzel-Costa .pptx was unzipped, and its slide order was checked in `presentation.xml`. Persson's PhoneWireAA.zip was read from its archive snapshot. The vendor and engine documentation pages (Unreal 5.6/5.8, Unity 6000.0, URP 17, SpeedTree, NVIDIA blog, Khronos registry) were fetched live with curl and converted to text, and `ue-5.6-world-partition-hlod` came from its archive snapshot. All downloads are in `dl-M/`. No prose claims were in this module: reader L checks the note's prose.

### Registry verdicts

| key | registry line | verdict | what was opened, what it says, and whether each citing site holds |
|---|---|---|---|
| msl-4.1-simdgroup-forward-progress | docs/references.toml:5845 | confirmed | `web.archive.org/web/20260925145438id_/.../Metal-Shading-Language-Specification.pdf`: Version 4.1, footer 2026-06-04, p. 157 of 383 holds the sentence verbatim; a full-text search of the 383 pages finds no other "progress" clause. Citing site `lod-hlod-and-far-field.md:69` "Metal promises progress only between SIMD-groups of one threadgroup" holds (the "only" rests on that search; the note drops the source's "in the absence of threadgroup barrier operations", which narrows rather than widens). |
| nanite-lod-no-geomorphing | docs/references.toml:2768 | confirmed | `advances.realtimerendering.com/s2021/Karis_Nanite_SIGGRAPH_Advances_2021_final.pdf` p. 67, slide Seamless LOD, speaker notes: verbatim (line breaks only). Citing site `:41` "judging geomorphs and cross-fades too expensive" holds ("expensive at render time or require significant additional data or both"). |
| nanite-taa-lod | docs/references.toml:591 | confirmed | same page, verbatim. Citing site `:41` "Nanite avoids both by keeping error under a pixel and letting TAA smooth the change" holds. (ADR 0019 L9 also cites it, in a clause M5 did not amend: out of scope.) |
| netzel-costa-2022-rtow-far-field-tlas | docs/references.toml:4686 | confirmed | the .pptx (159.8 MB) unzipped: `presentation.xml` lists `slide33.xml` 33rd; its title "How to setup TLAS to trace against each field separately?"; `notesSlide33.xml` holds the quote verbatim (including "completely avoid"), the gap marking the skipped "This is ok since ..." line. Citing sites: `lod-hlod-and-far-field.md:77` "which a separate TLAS avoids" holds; `acceleration-structures-for-streamed-worlds.md:44` (three quoted fragments, "avoid[s]" bracketed) holds. |
| niagara-567ebe3-gpu-lod-select | docs/references.toml:5851 | confirmed | raw `src/shaders/drawcull.comp.glsl` at 567ebe3: L110-L118 hold the quote verbatim (the `if (cullData.lodEnabled == 1) {` opener sits outside it, the `}` it keeps is real); `niagara.cpp` L1526 sets `cullData.lodTarget = ... // 1px`; licence file MIT. Citing site `docs/research/lod-hlod-and-far-field.md:39` "niagara picks the level per draw in its two-phase culling compute shader": the file's `LATE` specialization constant and `drawVisibility` give the two phases; holds. Section line range 106-118 covers the quote (it starts at L110). |
| niagara-567ebe3-lod-chain | docs/references.toml:5857 | confirmed | raw `src/scene.cpp` at 567ebe3: L246, L256, L260, L266-L268 hold the four fragments verbatim; `scene.h` L92 `MeshLod lods[8];`; `kShadowLodError = 3e-3f` L22; `scenert.cpp` L47 and `scene.cpp` L960/L1030 build BLAS from `mesh.lods[mesh.lodRT]`. Citing sites: `lod-hlod-and-far-field.md:37` "up to 8 levels at 0.6x, stopping when a level keeps 85% of the previous one's indices" holds; `:77` "builds each BLAS from the coarsest level whose error stays under a shadow threshold" holds (the loop keeps the last, coarsest, qualifying level; `lodRT` feeds the BLAS builds). |
| nv-alpha-to-coverage-dither-control | docs/references.toml:5863 | confirmed | `registry.khronos.org/.../NV_alpha_to_coverage_dither_control.txt` live: Revision 1.01, Last Modified May 31, 2017; *Modification to Chapter 17.3.3* holds the quote verbatim. Citing site `:47` "NVIDIA exposes its dither as a switch" holds (DEFAULT/ENABLE/DISABLE modes). |
| nvidia-2022-rtx-best-practices-lod-culling | docs/references.toml:4712 | confirmed | url live (200): Jul 25, 2022, By Juha Sjoholm; under *Acceleration structures* > *General tips* the three bullets verbatim. Citing sites: `lod-hlod-and-far-field.md:77` "the same LOD for both, or back-face culling where they differ" holds; `acceleration-structures-for-streamed-worlds.md:58` "the same LOD for both to avoid self-intersection, or back-face culling where they differ" holds. |
| osg-3.6.5-small-feature-culling | docs/references.toml:5875 | confirmed | raw `include/osg/CullingSet` at tag OpenSceneGraph-3.6.5 L252-L255; `include/osg/CullSettings` L220-L222 (the doc comment "Default is 2.0") and L186-L187 (`DEFAULT_CULLING` includes `SMALL_FEATURE_CULLING`); `src/osg/CullSettings.cpp:36` sets `_smallFeatureCullingPixelSize = 2.0f`. Verbatim. Citing site `:73` "culls bounds under 2 pixels by default" holds, in the doc comment's own words. |
| persson-2012-landmarks-distant-forest | docs/references.toml:5881 | confirmed | `web.archive.org/web/20261004131416id_/.../Persson_CreatingVastGameWorlds.pdf` (42 pp.): p. 5 the landmark sentences, p. 28 the billboard-tree passage, verbatim. Citing site `lod-hlod-and-far-field.md:53` holds. |
| persson-2012-phone-wire-aa | docs/references.toml:5887 | confirmed | `web.archive.org/web/20260515045315id_/...ID=89`: "Phone-wire AA", Tuesday, June 26, 2012; the three passages verbatim (the source's "helpes"). Citing site `:75` "clamps a wire's width to one pixel and fades its alpha by the width it lost, and suggests the same for railings and antennas" holds. |
| persson-2012-phone-wire-aa-licence | docs/references.toml:5893 | confirmed | `web.archive.org/web/20221218100843id_/.../PhoneWireAA.zip` (2.3 MB), `PhoneWireAA/App.cpp` (dated 2012-06-26) Author's note: the quote verbatim (box-drawing asterisks dropped). Citing site `:75` "the demo's code needs the author's approval for commercial use" holds. The archive was read in memory and deleted. |
| sorensen-2021-apple-obe-violation | docs/references.toml:5911 | confirmed | `arxiv.org/pdf/2109.06132v1` p. 25 verbatim; p. 2 defines Occupancy-Bound Execution (OBE). Citing site `:69` "Apple supplied a test that breaks the occupancy-bound model on A12 and A14" holds. |
| sorensen-2021-progress-spec-gap | docs/references.toml:5917 | confirmed | same PDF, p. 1 abstract verbatim; "Proc. ACM Program. Lang. 5, OOPSLA, Article 131 (October 2021)". Citing site `:69` "Vulkan and Metal 'say almost nothing' about forward progress between workgroups" holds. |
| speedtree-runtime-billboard-a2c | docs/references.toml:5923 | confirmed | `docs.unity3d.com/speedtree-runtime-sdk/manual/level-of-detail.html` live (200) on 2026-11-21; both quoted passages verbatim. Citing site `:57` "SpeedTree cross-fades its 3D tree into billboards through alpha-to-coverage" holds. |
| strugar-2010-cdlod-morph | docs/references.toml:5929 | confirmed | raw `cdlod_paper_latest.pdf` at ea8195b: p. 1 "Filip Strugar, 11 July 2010"; p. 7 *Morph implementation* and p. 9 the limitation sentence, verbatim. Citing site `:49` "morphs terrain grids in the vertex shader with no seams, between two levels per node" holds. |
| tressfx-4.1-strand-expand | docs/references.toml:5935 | confirmed | raw `src/Shaders/TressFXStrands.hlsl` at v4.1.0: L100, L101, L118 verbatim (including the source's "to to" typo); `license.txt` MIT. `ComputeCoverage` in `TressFXRendering.hlsl` starts at L104 but ends at L130, not L115 (the section's range is a prefix; nothing cites the rest). Citing site `:75` "widens hair strands by 0.71 pixels of the window's height, split over both edges": each edge vertex moves outward (`fDirIndex` ±1) by 0.71 / `winSize.y` in NDC, half a pixel per NDC unit over the height, so 0.355 px an edge and 0.71 px in all; holds. |
| ue-5.6-auto-lod-groups | docs/references.toml:5944 | confirmed | url live (200), page titled Unreal Engine 5.6 Documentation; the intro sentences and the `SmallProp=` and `HighDetail=` lines of the `[StaticMeshLODSettings]` snippet verbatim. Citing site `:37` "keep 50% of the triangles a level; Small Prop has 4 levels at 10 pixels of error, High Detail 6 at 6" holds on the snippet's `NumLODs`, `LODPercentTriangles=50`, `PixelError` fields (the page does not define `LODPercentTriangles`; "a level" is the field name's reading). |
| ue-5.6-cull-distance-volumes | docs/references.toml:5950 | confirmed | url live (200), 5.6; intro and the example's "popping" sentence verbatim. Citing site `:73` "Unreal maps object size to a cull distance and warns of popping" holds. |
| ue-5.6-lod-screen-size | docs/references.toml:5956 | confirmed | url live (200), 5.6; the sentence verbatim. Citing site `:39` "Unreal and Unity switch on screen size, not distance" holds for Unreal. |
| ue-5.6-skeletal-lod-hysteresis | docs/references.toml:5962 | confirmed | url live (200), 5.6; *LOD Hysteresis* row verbatim. Citing site `:41` "Unreal's skeletal meshes delay only the switch to a simpler level" holds. |
| ue-5.6-world-partition-hlod | docs/references.toml:3275 | confirmed | `web.archive.org/web/20250825185139id_/...` (gzip body, decompressed) titled Unreal Engine 5.6 Documentation: the overview sentence and the *Cell Size* and *Loading Range* rows verbatim. Citing site `:53` "draws unloaded cells by HLOD layers with their own cell size and loading range" holds. |
| ue-5.8-dithered-lod-transition | docs/references.toml:5968 | confirmed | url live (200), Unreal Engine 5.8 Documentation, under *Advanced Material Properties*: both rows verbatim. Citing site `:45` "Unreal's dithered transition is a material switch" holds. |
| ue-5.8-hlod-layer-defaults | docs/references.toml:5980 | confirmed | `gh api repos/EpicGames/UnrealEngine/contents/...` at 5.8.4-release: `Private/WorldPartition/HLOD/HLODLayer.cpp` L37-L39 (constructor L34) and L268 (`GetRuntimeGridName` L266) verbatim; `Public/WorldPartition/HLOD/HLODLayer.h` L80-L90 carry the `UE_DEPRECATED(5.7, ...)` lines (the section names the header without its `Public/` path). Citing site `:53` "the layer's own defaults 256 m cells loaded within 512 m, settings that moved to the partition's settings in 5.7" holds (centimetres, `* 0.01f` to metres); "256 m is the size of Praxis's Chunks" matches `src/assets/cooked.rs:58` `CHUNK_EDGE_METRES: f64 = 256.0`. |
| ue-5.8-hlod-swap-warmup | docs/references.toml:6002 | confirmed | same repo and tag, `HLODRuntimeSubsystem.cpp` L47-L50 and L215 verbatim; `IsWarmupEnabled` L194-L221; `OnCellShown` (L623) hides the cell's HLODs and `OnCellHidden` shows them; `wp.Runtime.HLOD.WarmupEnabled`, `WarmupVT`, `WarmupNanite` default to 1 (L32-L45). Citing site `:53` "a hard toggle; with Nanite or virtual texturing on, the cell's unload waits five frames by default" holds. |
| unity-6000-speedtree-geomorph | docs/references.toml:6010 | confirmed | url answers 301 to `docs.unity.com/en-us/engine/6000.0/manual/.../class-lodgroup` (the same page as the next entry); *Working with SpeedTree Models* and *Blending for SpeedTree Models* hold both passages verbatim. Citing site `:49` "SpeedTree stores each vertex's next-level position" holds. |
| unity-6000.0-layer-cull-distances | docs/references.toml:6016 | confirmed | url live (200); *Remarks* holds the quote verbatim. Citing site `:73` "Unity culls layers early" holds (per-Layer cull distances, "cull small objects early on"). |
| unity-6000.0-lodgroup-transition-height | docs/references.toml:6022 | confirmed; citing site `:45` drifts (L7-M01) | url live (200); *LOD Group selection bar* (the percentage passage) and *Transitioning between LOD levels* (the cross-fade passages) verbatim. Citing site `:39` "Unreal and Unity switch on screen size, not distance" holds for Unity. Citing site `:45` "Both levels draw during the fade, weighted 1 to 0 and 0 to 1, usually through screen-space dithering" says more than the source: "usually" in the source governs "either screen-space dithering or transparency", not dithering alone (L7-M01). |
| unity-graphics-6000.0-cpu-instance-culling-job | docs/references.toml:6028 | confirmed | raw `InstanceCuller.cs` at 9808614: L135-L138 and L1613 verbatim; the package LICENSE.md is the Unity Companion License; `InstanceCuller.Init` (L1538) takes `GPUResidentDrawerResources`. Citing site `:65` "culls flat on the CPU, in a Burst parallel-for job over every Instance, 32 to a batch" holds (`Schedule(instanceData.instancesLength, k_BatchSize)`). |
| unity-graphics-6000.0-small-mesh-culling | docs/references.toml:6039 | confirmed | same file, L296-L309 (section says L296-L310; L310 is the closing brace) and L1601, verbatim. Citing site `:73` "fades small meshes out over a range rather than dropping them abruptly" holds (`lodPercent = math.saturate((maxDist - cameraDist) / fadeOutRange)`). |
| unity-graphics-core-lod-dithering-taa | docs/references.toml:6050 | confirmed | raw `ShaderLibrary/Common.hlsl` at 0efc28e, L1673-L1679 verbatim inside `LODDitheringTransition` (L1676-L1686). Citing site `:45` "its core library keeps the pattern static because a moving one 'confuses the TAA'" holds. |
| unity-urp-17-lod-crossfade-dither | docs/references.toml:6066 | confirmed | url answers 301 to the Unity 6000.0 manual's URP Asset page (URP 17.0 ships with 6000.0); *LOD Cross Fade Dithering Type* holds the quote verbatim (the registry's " ," and " :" spacing is a rendering artefact). Citing site `:45` "Unity's URP offers a Bayer matrix, blue noise or a 2×2 stencil" holds. |
| vk-lod-clusters-persistent-nvidia-only | docs/references.toml:6072 | confirmed | raw README.md at 20352f0, L229 under `## Limitations` (L224), verbatim; LICENSE at that commit is Apache 2.0. Citing site `:69` "NVIDIA's sample turns its persistent kernel off on other vendors' hardware" holds. |
| webgpu-alpha-to-coverage-mask | docs/references.toml:6078 | confirmed | raw `spec/index.bs` at 688f043: L17012 `### Alpha to Coverage`, the quote's sentences at L17014-L17025 verbatim (Bikeshed markup `<dfn>`, `|alpha|` dropped). Citing site `lod-hlod-and-far-field.md:47` "the mask it derives from alpha is platform-dependent and may vary by pixel" holds. |
| webgpu-sample-mask-vs-alpha-to-coverage | docs/references.toml:6084 | confirmed | same file, L8792-L8794, verbatim, inside *validating GPURenderPipelineDescriptor*. Citing site `:47` "WebGPU forbids that output together with alpha-to-coverage" holds. |
| webgpu-standard-sample-pattern | docs/references.toml:675 | confirmed | same file, L16579-L16597: the 4x row Sample 0: (0.375, 0.125) ... Sample 3: (0.625, 0.875) and "Implementations must use the standard sample pattern ..." verbatim. Citing sites: `lod-hlod-and-far-field.md:47` (positions to vary k-of-4 over) holds; code site `src/render/shaders/light_count.wgsl:9` "WebGPU's standard 4x pattern puts sample 0 at (0.375, 0.125)" holds, and `FIRST_SAMPLE` at L10 equals it. |
| wgpu-30.0.1-alpha-to-coverage | docs/references.toml:6090 | confirmed | raw `wgpu-types/src/render.rs` at tag v30.0.1, L433-L442 verbatim; the locked wgpu-types 30.0.2 (`~/.cargo/registry/.../wgpu-types-30.0.2/src/render.rs`) has the same lines at the same numbers, so the v30.0.1 pin is not stale. Citing site `:47` "wgpu's own field gives no pattern guarantee beyond alpha 0 and 1" holds. |
| wgpu-core-30-indirect-required | docs/references.toml:1598 | confirmed | raw `wgpu-core/src/device/resource.rs` at v30.0.1 L1056-L1057 verbatim inside `create_buffer` (L1005); the vendored 30.0.2 `vendor/wgpu-core/src/device/resource.rs:1057` is the same. Citing site `lod-hlod-and-far-field.md:67` "Praxis's indirect buffers already rest on the Capability floor" holds: ADR 0021 L14 puts `INDIRECT_EXECUTION` on the floor citing this key, and `src/gpu/mod.rs:296`/`:615` require it. (ADR 0021 is outside the scope; its site was not re-judged.) |
| wihlidal-2024-empty-execute-indirect | docs/references.toml:6105 | confirmed | `media.gdcvault.com/.../Nanite+GPU+Driven+Materials.pdf` (168 pp.): p. 114 the bullets verbatim; p. 113 the empty-bin trace. `cost` matches. Citing site `:67` "about 90 ns on average, spiking to 500 ns or worse on a cache miss" holds. |
| wright-2022-lumen-fallback-self-intersection | docs/references.toml:5031 | confirmed | `advances.realtimerendering.com/s2022/SIGGRAPH2022-Advances-Lumen-Wright%20et%20al.pdf`: p. 109 *Tracing with Nanite* and p. 112 *Avoiding Self-Intersection* notes, verbatim. Citing sites: `lod-hlod-and-far-field.md:77` holds; `acceleration-structures-for-streamed-worlds.md:58` "fixed by a short back-face-culled ray before the long one" holds ("Tracing directly from the GBuffer" is the case the note likens to its depth-target rays). |
| wright-2022-lumen-far-field-overlap | docs/references.toml:5043 | confirmed | same PDF: p. 123 notes (first four sentences) and p. 124 notes (last sentence) verbatim; p. 124 table Baseline/Overlapping/Translated = 1.65/2.37/1.77 ms near, N/A/1.04/0.60 ms far, matching `cost`; 2.37/1.65 = 1.436. Citing sites: `lod-hlod-and-far-field.md:77` "cost Epic 44% of near-field traversal" holds; `acceleration-structures-for-streamed-worlds.md:56` (quote, the three timings, "the proper fix") holds. |

## `docs/research/acceleration-structures-for-streamed-worlds.md`

Read the whole note at the frozen commit `62052fe8` (137 lines) and its M5 diff (`6ebb45fb..62052fe8`, lines 56, 60 and 62, from the 2026-11-14 survey correction), against the M5-amended ADR 0020:20 and ADR 0014, ADR 0003:3, ADR 0026, and the code (`src/render/acceleration_structures.rs`, `src/render/renderer.rs`, `src/render/blas_refit.rs`, `src/assets/pools.rs`, `vendor/README.md`). Load-bearing claims are those ADR 0020's M5 amendments (LOD 0 BLASes; the R2 re-anchor build without R1's move) and ADR 0026's Far proxy shadows (one static TLAS entry under the ordinary mask bit) rest on, plus the probe figures ADR 0020:20 quotes from this note; every claim about this repo's code was checked; the rest sampled every fifth in document order. Sources were opened, never vouched from memory, into `dl-S/`: PDFs at the cited page with `pdftotext` (Apers 2024, Benthin 2017, 2023 and 2024, Battlefield V GTC 2019, Ize 2007, AC Shadows 2025, Metal Feature Set Tables archive, Lumen 2022); the Epic 2022 `.pptx` read as XML slide and notes text, in presentation order; HTML pages stripped to text (AMD GPUOpen, NVIDIA blog, Microsoft devblog, both UE 5.8 archived docs, Unity 6000.6, Apple WWDC22 and WWDC23 transcripts, Lloyd 2020); Apple's documentation through its JSON data endpoint; GitHub sources raw at the pinned commit or tag (DirectX-Specs `2808d21`, Vulkan-Docs `v1.4.367`, Unity Graphics `0efc28e`) and Unreal Engine `5.8.4-release` through `gh api`; wgpu crates locally at 30.0.1 and 30.0.2 under `~/.cargo/registry`. Every one of the 38 entries opened.

### Claim verdicts

| doc | claim (quoted, shortened) | load-bearing | verdict | where / evidence |
|---|---|---|---|---|
| note | "A worker thread can take the fill, the recording and `finish` (13 to 14 ms of it)" | yes (ADR 0020:20 quotes it) | confirmed | `:5`; table `:78`: CPU total 17.53–18.08 less submit 4.33–4.52 = 13.2–13.6 ms |
| note | "the main thread still pays the submit (4.3 to 4.5 ms at that size" | yes (ADR 0020:20) | confirmed | `:5`, `:78` submit 4.33–4.52 |
| note | "the main thread paid 5.6 to 7.1 ms to submit each build" | yes (ADR 0020:20) | confirmed | `:36`, `:92` "5.64 to 7.11 ms" |
| note | "Its memory is a second TLAS: 75.6 MB at 262,144 slots" | yes (ADR 0020:20) | confirmed | `:42`, `:50`: 288.38 B a slot × 262,144 = 75.60 MB |
| note | "Praxis rebuilds its static TLAS on a Chunk change, which can move the anchor [...] A camera that travels without crossing a Chunk boundary keeps the old anchor, and the ray offset grows" | yes (ADR 0020:20, ADR 0014 M5 amendments on R1/R2) | stale | `:46`; L7-S02 |
| note | R1 "At every static rebuild, to the camera's Anchor"; R2 "R1 plus a rebuild when the camera is a set distance from the anchor" | yes (ADR 0020:20 "R2's build without R1's move") | confirmed | `:114-115` match ADR 0020:20's wording of the decision |
| note | Epic's far field: HLOD "both simplified and merged together", exclusive by ray masks | yes (ADR 0026, Its shadows) | confirmed | `:56`; Lumen 2022 PDF p. 119 notes: "both simplified and merged together [...] force mutual exclusion during ray traversal with a ray-mask" |
| note | one TLAS cost "a show-stopping 44% penalty" (1.65 to 2.37 ms PS5 1080p, 1.77 ms translated); Epic call multiple TLASes the fix | yes (ADR 0026, the Far proxy in the static TLAS) | confirmed | `:56`; Lumen 2022 PDF pp. 123-124: "Near-Field 1.65 ms 2.37 ms 1.77 ms", "A proper solution would be to support multiple top-level acceleration structures" |
| note | UE 5.8 far field starts at the Max Trace Distance (200 m) and ends at `r.LumenScene.FarField.MaxtraceDistance` (1 km) | yes (M5 diff) | confirmed | `:56`; registry row `ue-5.8-lumen-far-field-and-fallback` |
| note | "wgpu's 8-bit instance mask carries a near/far split" | yes (ADR 0026, mask bit) | confirmed | `:56`; registry `wgpu-30-tlas-instance-fields` quote `pub mask: u8` (entry opened by reader T) |
| note | a secondary ray "must 'see' the same LODs" as the rasterizer | yes (ADR 0020:20 LOD 0) | confirmed | `:58`; Lloyd 2020 blog: "Any secondary ray starting from a primary hit must “see” the same LODs" |
| note | NVIDIA: the same LOD for both to avoid self-intersection, or back-face culling where they differ | yes (ADR 0020:20 LOD 0) | confirmed | `:58`; NVIDIA blog, General tips: "using the same LOD for rasterization and ray tracing can be considered [...] enabling back face culling is often needed" |
| note | Epic's fallback meshes self-intersected, fixed by a short back-face-culled ray | yes (ADR 0020:20 LOD 0) | confirmed | `:58`; Lumen 2022 PDF p. 112: "first cast a short ray, to some defined epsilon-distance, ignoring back faces" |
| note | "Praxis's shadow rays start from the depth target, the case they describe" | yes (code claim) | confirmed | `:58`; ADR 0020:3 "A compute pass traces them from the depth target" |
| note | "A discrete LOD change is a BLAS change, so a static TLAS rebuild under ADR 0020:20's triggers" | yes (ADR 0020:20 LOD 0) | wrong | `:58`; L7-S01 |
| note | AMD splits a terrain BLAS into chunks and a two-statue BLAS into one instanced twice | M5 diff | confirmed | `:60`; registry row `amd-rra-instance-overlap` |
| note | Microsoft lists "All RTX GPUs", RX 9000, Arc B-Series, Core Ultra 200V and Series 3, Qualcomm to follow | M5 diff | confirmed | `:62`; registry row `microsoft-2026-clas-ptlas-preview` |
| note | "a PR merged on 2026-08-30, after 30.0.1 and not in 30.0.2, exposes a Vulkan acceleration structure's handle" | M5 diff | confirmed | `:62`; `gh api repos/gfx-rs/wgpu/pulls/10187`: merged_at 2026-08-30T17:30:56Z, "Add raw_handle method on vulkan::AccelerationStructure"; wgpu-hal-30.0.2 `src/vulkan/mod.rs` has `raw_handle` on Buffer, Image, ImageView and CommandBuffer only |
| note | Row 1 "Rebuild in the frame's encoder (Praxis today)"; §1 "(today)"; S2 "In place, as today" | code claim | stale | `:15`, `:32`, `:112`; L7-S03 |
| note | Row 1 fit "breaks ADR 0013's 6 ms CPU" | code claim | confirmed | `:15`; ADR 0013:3 "CPU at most 6 ms" |
| note | Row 2 "The dynamic TLAS does it" (own command buffer) | code claim | confirmed | `:16`; `src/render/acceleration_structures.rs:737-738` "its build goes in a command buffer of its own" |
| note | Row 10 "Praxis already queries BLAS refit scratch so" | code claim | confirmed | `:24`; `src/render/blas_refit.rs:356-363` `get_acceleration_structure_build_sizes(...).update_scratch_size` |
| note | "against the 288.5 the ledger then estimated from `currentAllocatedSize`" | code claim | confirmed | `:50`; dated by "then"; `src/render/acceleration_structures.rs:70` `tlas_bytes` now queries wgpu-hal |
| note | "`docs/performance.md`'s option B for `town`'s shadow rays is this" | code claim | confirmed | `:60`; `docs/performance.md:2154` "B. Fewer, non-overlapping static Instances: merge each cell's static Instances into one BLAS" |
| note | "Static TLAS of capacity 262,144 (`STATIC_TLAS_CAPACITY`)" | code claim | stale | `:68`; L7-S04 |
| note | "past ADR 0003's 524,288 slots" | code claim | stale | `:101`; L7-S05 |
| note | `town`'s density "16,008 per 256 m Chunk" | code claim | confirmed | `:94`; 244,256 × 65,536 m² / 1 km² = 16,007.6 |
| note | "with gfx-rs/wgpu#9835 carried in `vendor/wgpu-core`" | code claim | confirmed | `:134`; `vendor/README.md:25` |
| note | `docs/audit/m3.5/scalability.md`, L4-02 | code claim | confirmed | `:46`; `docs/audit/m3.5/scalability.md:48` lists L4-02 |
| note | build in its own command buffer: "18.2 to 18.4 ms against 20.3 to 20.4" | fifth sample | confirmed | `:5` against table `:88-89` |
| note | "a Load radius of 256 m holds 112,000 to 144,000 [...] 384 m up to 256,000 and 512 m up to 368,000" | fifth sample | confirmed | `:5` against table `:98-100` |
| note | Row 7 "no TLAS binding arrays on Metal" | fifth sample | confirmed | `:21`; wgpu-types-30.0.1 `src/features.rs:1456-1462` `ACCELERATION_STRUCTURE_BINDING_ARRAY` "Supported platforms: - DX12 - Vulkan" |
| note | "wgpu-core always rebuilds" | fifth sample | confirmed | `:32`; wgpu-core-30.0.1 `src/command/ray_tracing.rs:341`, `:1126` `mode: hal::AccelerationStructureBuildMode::Build` |
| note | "At submit, each build [...] walks one dependency per Instance" | fifth sample | confirmed | `:32`; wgpu-core-30.0.1 `src/command/ray_tracing.rs:271` `dependencies.push(blas.clone())` per instance, walked at `:499` |
| note | "the pass starts 3.2 to 3.3 ms after the build and the two overlap for 14.2 to 14.7 ms [...] saves only about 2 ms" | fifth sample | confirmed | `:34` against `:86-89`: span 18.2–18.4 against 20.3–20.4; a start 3.4 ms in is what the per-round medians give, within their spread |
| note | "`Buffer::destroy`, `Texture::destroy` and a `Device::poll` that frees what they left take it for writing" | fifth sample | confirmed | `:36`; wgpu-core-30.0.1 `src/resource.rs:1078`, `:1638`, `src/device/resource.rs:736`, `:754` `snatchable_lock.write()` |
| note | AC Shadows rebuilds its TLAS every frame "in the async queue" | fifth sample | confirmed | `:38`; registry row `leblanc-conte-2025-acs-tlas-async` |
| note | "no source found describes a shipped engine double-buffering its TLAS" | fifth sample | confirmed (as a negative: none of the 38 sources opened, nor Lumen 2022, describes one; Epic, Ubisoft and Frostbite rebuild in place each frame) | `:42` |
| note | ordering a rebuild after a pass that read the TLAS "rests on Metal's hazard tracking" | fifth sample | confirmed | `:42`; registry row `metal-hazard-tracking-default` |
| note | Unreal translates world space so "its origin is at ViewOrigin" | fifth sample | confirmed | `:46`; registry row `unreal-5.8-preview-translation` |
| note | "On Metal a TLAS query must pass some buffer as its instance buffer, or wgpu-hal's unwrap panics" | fifth sample | confirmed | `:50`; wgpu-hal-30.0.1 `src/metal/conv.rs:390` `descriptor.setInstanceDescriptorBuffer(Some(&instances.buffer.unwrap().raw))`, reached from `get_acceleration_structure_build_sizes` (`src/metal/device.rs:2080-2081`) |
| note | Battlefield V "to 1.15 ms with BLAS-side changes, staggered rebuilds and the fast-build flag among them" | fifth sample | confirmed | `:52`; GTC 2019 PDF p. 64 "Stagger full and incremental BLAS rebuild [...] PREFER_FAST_BUILD [...] Avoid redundant rebuilds", p. 65 "14.5 ms -> 1.15 ms" |

Claims checked: 42 (15 load-bearing, 3 in the M5 diff beyond those, 11 about the repo's code, 13 fifth samples). No sample was wrong; the wrong and stale rows are load-bearing or code claims, checked in full.

### Registry verdicts

| key | registry line | verdict | what was opened, what it says, and whether each citing site holds |
|---|---|---|---|
| `amd-rra-instance-overlap` | `docs/references.toml:4096` | confirmed | gpuopen.com/learn/improving-rt-perf-with-rra/ (published 2022-09-15): every quoted sentence verbatim and in order, under "Split terrain into chunks" and "Avoid instance overlap". Site `:60` (M5 wording: AMD splits the terrain BLAS and the two-angel BLAS so bounds stop overlapping) holds. Outside scope: `docs/performance.md:2154` cites it for "AMD warns a merge can also slow traversal"; the page warns that *splitting* a BLAS "often results in overlapping instances", which is the opposite direction, the slip the note's M5 edit corrected in its own sentence. |
| `apers-2024-gibs-async-as-build` | `:4533` | confirmed | Apers 2024 PDF p. 33 notes, quote verbatim. Site `:38` "always overlaps its BLAS and TLAS builds with G-buffer laydown" holds. |
| `apers-2024-gibs-rebraiding` | `:4539` | confirmed | PDF p. 30 notes verbatim; cost p. 55 "Offline BLAS and selective rebraiding shave off 0.35ms together", p. 53 "Skate, which we ran on XBSX at 1440p". Site `:60` holds. |
| `apple-wwdc22-10105-instance-overhead` | `:4084` | confirmed | developer.apple.com/videos/play/wwdc2022/10105/ transcript: both quoted passages verbatim, in order. Site `:60` holds. |
| `apple-wwdc23-10128-multi-level-instancing` | `:4546` | confirmed | wwdc2023/10128 transcript: quote verbatim. Site `:62` holds. |
| `benthin-2017-partial-rebraiding` | `:4552` | confirmed | rebraiding.pdf p. 1, Figure 1 caption ("1.2 × -2.1× higher rendering performance") and Abstract verbatim. Site `:60`'s "on CPU" is the paper's (p. 5 "dual-socket Intel Xeon E52699 v3"; p. 7 "our reference implementation was written for CPUs"), not the quote's; holds. |
| `benthin-2023-micro-poly-hlod-rt` | `:4558` | confirmed | author's PDF pp. 1-2: the three quoted sentences verbatim; cost's 1920 × 1080, 5.7–9 ms on p. 1; "Intel Arc A770" is stated in the per-frame and evaluation sections (pdftotext lines 185, 210), which `section` does not name (H-PLOC's entry names its hardware page; a remark, not filed). Site `:62` holds. |
| `benthin-2024-hploc-tlas` | `:4565` | confirmed | HPLOC.pdf p. 12 Section 4.1 verbatim, "2.21 ms vs 1.96 ms"; p. 9 "AMD Radeon 7900 XT GPU", "ROCm 5.7.3". Site `:52` holds. |
| `deligiannis-2019-bfv-as-culling` | `:4572` | confirmed | GTC 2019 PDF p. 63 verbatim; p. 57 "20200 TLAS instances", "5000 BLAS rebuilds", "GPU rebuild 64 ms". Site `:52` holds. |
| `deligiannis-2019-bfv-as-staggered-cost` | `:4579` | confirmed | PDF p. 65, printed 78: "TLAS + BLAS build (GPU): 14.5 ms -> 1.15 ms", "RayGen (GPU): 0.71 ms -> 0.81 ms (staggered refit + flags)". Site `:52`'s list of what bought it is p. 64's, outside `section`; holds. |
| `dxr-2026-clusters-support` | `:4586` | confirmed | DirectX-Specs `2808d21` `d3d/Raytracing2.md` lines 112-153 (raw, CRLF stripped): quote verbatim, lines 112, 114, 126, 153. Site `:62` holds. |
| `dxr-2026-ptlas-overview` | `:4592` | confirmed | same file lines 390-418, quote verbatim (390, 413, 414, 418). Site `:62` holds. |
| `dxr-2026-ptlas-partition-translation` | `:4598` | confirmed | lines 447-455 verbatim (449, 453, 455); design properties 407-416. Sites `:48` and row 9 `:23` hold. |
| `ize-2007-async-bvh-swap` | `:4604` | confirmed | async_bvh_build.pdf p. 3, §3.1: quote verbatim. Site `:42` and row 6 "Research, 2007 (CPU)" hold. |
| `leblanc-conte-2025-acs-bvh-stats` | `:4610` | confirmed | AC Shadows PDF p. 126 (printed 129): slide bullets and notes verbatim, "Those numbers were taking on an Xbox Series X." Sites `:5`, `:56` hold. |
| `leblanc-conte-2025-acs-rt-scene` | `:4622` | confirmed | PDF p. 28 notes and p. 34 notes verbatim. Site `:56` holds. |
| `leblanc-conte-2025-acs-tlas-async` | `:4628` | confirmed | PDF p. 34 slide and notes verbatim, "We rebuilt every frame the TLAS in the async queue". Sites `:38`, row 4 hold. |
| `metal-feature-set-tables-instancing-levels` | `:4653` | confirmed | web.archive.org `20260924114533id_` Metal-Feature-Set-Tables.pdf (2026-05-21): p. 9 intersection-query row "Not available ×4, 16 ×5" over Apple2–Apple10 (the intersector row reads 32); p. 10 note 12 verbatim; p. 2 "M4-series Metal 3 & 4 Apple9". Site `:62` "up to 16 levels on Apple9" holds for ray queries. |
| `metal-hazard-tracking-default` | `:657` | confirmed | Apple docs JSON for MTLHazardTrackingMode.default, Discussion: "The default tracking mode for a type that inherits MTLResource is MTLHazardTrackingMode.tracked because individual resources benefit from automatic hazard tracking." Sites: `:42` holds; `src/render/blas_refit.rs:211-214` "Metal tracks a resource by default" holds. |
| `metal-indirect-instance-descriptor` | `:427` | confirmed | Apple docs JSON for the initializer: Swift declaration verbatim as quoted. Sites: `:50` "72 bytes on Metal", `:118`, and `src/render/acceleration_structures.rs:821-823` (`Metal => 72`) hold. |
| `microsoft-2026-clas-ptlas-preview` | `:4659` | confirmed | devblogs.microsoft.com/directx/clas-ptlas-preview/ (Amar Patel, 28 October 2026, Agility SDK 1.722.0-preview): every quoted passage verbatim, support lists as quoted; cost table "788 ms 3.11 ms ~254x", "72.6 ms 9.27 ms ~7.8x" on "an AMD RX 9070". Site `:62` (M5 list) holds. |
| `microsoft-2026-ptlas-preview-domino` | `:4666` | confirmed | same page, domino section verbatim; no GPU named for the 2.88 ms figure. Licences: microsoft/DirectX-Graphics-Samples MIT (`gh api` spdx), the sample's `LICENSES/MIT.txt`; nvpro-samples/vk_partitioned_tlas sources `SPDX-License-Identifier: Apache-2.0`; its README "170000 dominoes on a board made of 1.2 million static objects". Sites `:48`, row 9 hold. |
| `microsoft-dxr-build-flags` | `:4673` | confirmed | DirectX-Specs `2808d21` `d3d/Raytracing.md` lines 4211 (ALLOW_UPDATE), 4213 (PREFER_FAST_TRACE), 4214 (PREFER_FAST_BUILD), verbatim. Site `:52` holds. |
| `netzel-costa-2022-rtow-build-vs-instances` | `:4679` | confirmed | the `.pptx`, slide 34 in presentation order: table text and notes verbatim; no platform on the slide or notes (the deck names PS5 and Xbox Series only for the demo and other slides). Site `:52` holds. |
| `netzel-costa-2022-rtow-gpu-instance-buffer` | `:4692` | confirmed | slides 41-42 notes verbatim; slide text "Build Instance Buffer - 0.02ms". Sites `:40`, row 5 hold. |
| `netzel-costa-2022-rtow-tlas-async` | `:4699` | confirmed | slide 43 notes verbatim; "Rebuild TLAS - 2 ms On async queue - 0.3 ms". Sites `:38`, row 4 hold. |
| `nvidia-2022-rtx-best-practices-as-builds` | `:4706` | confirmed | developer.nvidia.com blog (Juha Sjoholm, Jul 25, 2022): the four quoted tips verbatim, under General tips, Build preference flags and Dynamic BLASes. Site `:34` holds. |
| `nvidia-2022-rtx-best-practices-shadow-rays` | `:4060` | confirmed | same page: the merging tip (Organizing geometries into BLASes), first-hit tip (Hit shading, General tips) and the register tips (Inline ray tracing), verbatim. Sites: `:60` holds; `src/render/shaders/shadow_mask.wgsl:93-94` "A shader's live ray queries hold registers that lower its occupancy" holds. |
| `ue-5.8-lumen-far-field-and-fallback` | `:4718` | confirmed | web.archive.org `20261015100109id_` Lumen Technical Details (UE 5.8): every quoted sentence verbatim, including the spelling `r.LumenScene.FarField.MaxtraceDistance`. Site `:56` (M5 wording) holds. |
| `ue-5.8-rt-perf-tlas-instances-overlap` | `:4724` | confirmed | web.archive.org `20260913071353id_` Ray Tracing Performance Guide (UE 5.8): quote verbatim, the second sentence cut at its parenthesis with `[...]`. Sites `:52`, `:60` hold. |
| `unity-6000.6-rtas-relative-origin` | `:4730` | confirmed | docs.unity.com 6000.6 page, Remarks: verbatim. Site `:46` holds. |
| `unity-hdrp-camera-relative-rtas` | `:4736` | confirmed | Unity Graphics `0efc28e` `HDRaytracingManager.HDRTASManager.cs` lines 366-372 verbatim; `HDRaytracingManager.cs:640` `BuildRayTracingAccelerationStructure`, `:690` `m_RTASManager.Build(hdCamera)`, called per camera from `HDRenderPipeline.cs:2962`; licence file "Unity Companion License". Sites `:46`, row 8 hold. |
| `unreal-5.8-preview-translation` | `:4748` | confirmed | `gh api` UnrealEngine `5.8.4-release` `SceneView.cpp` lines 694-696 verbatim; `RayTracingScene.cpp:217-221` `SetViewParams` stores `GetPreViewTranslation()`. Site `:46` holds. |
| `unreal-5.8-rt-async-build` | `:4756` | confirmed | `DeferredShadingRenderer.cpp` lines 1104-1137 contain the quoted lines verbatim; cvar at 314-319, default `0`. Site `:38` "behind a switch" holds. |
| `unreal-5.8-rt-far-field-layer` | `:4766` | confirmed | `RayTracingScene.h` lines 33-40 verbatim; `GetRHIRayTracingScene(ERayTracingSceneLayer Layer, FViewHandle ViewHandle)` (`:181`) gives each layer its own scene per view. Sites `:44`, rows 7 and 11 hold. |
| `unreal-5.8-rt-parallel-gather` | `:4779` | confirmed | `RayTracing.cpp` lines 90-94 verbatim (the entry's L90-L96 covers the cvar to its close). Sites `:36`, row 3 hold. |
| `unreal-5.8-rt-translated-world-instances` | `:4789` | confirmed | `RayTracingInstanceBufferUtil.usf` lines 145 and 153 verbatim; descriptor built from `LocalToTranslatedWorld` at 245-250. Site `:46` holds. |
| `vulkan-1.4.367-cluster-acceleration-structure` | `:4797` | confirmed | Vulkan-Docs `v1.4.367` `chapters/accelstructures.adoc` lines 3424-3455: heading at 3424, quoted sentences at 3426-3430, 3449-3454, verbatim, inside `ifdef::VK_NV_cluster_acceleration_structure[]`. Site `:62` holds. |

Entries: 38 confirmed, 0 wrong, 0 unsupported, 0 stale, 0 dead, 0 unopened. Every entry carries a `checked` date (the oldest 2026-11-05); none needs a Finding, so each takes a new check date from the report.

## Registry entries cited by the acceleration-structures note (reader T)

I read the 37 assigned entries in `docs/references.toml` at `62052fe8`, all of them in full. I grepped `ref:<key>` across the scoped documents and grepped every key across `src`, `tests` and `scripts`. The only scoped document that cites any of them is `docs/research/acceleration-structures-for-streamed-worlds.md`, at lines 32, 34, 36, 40, 42, 44, 48, 50, 52, 56 and 62. No ADR in scope cites them, and neither does the far-proxy note or the LOD note. Six code comments cite them, all in `src/render/acceleration_structures.rs`, at lines 57, 116, 750, 909, 989 and 1281. Out-of-scope citers I did not judge: the GI note and `source-audit.md`, which cite `wgpu-30-tlas-instance-fields`, and the volumetrics note, which cites `wgpu-hal-30-metal-one-queue`.

**wgpu sources.** The entries' urls pin `v30.0.1`. I read each cited range in the local crates `wgpu-30.0.1`, `wgpu-core-30.0.1`, `wgpu-hal-30.0.1` and `wgpu-types-30.0.1` under `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`. Then I fetched all 26 cited files from `raw.githubusercontent.com/gfx-rs/wgpu/v30.0.1/` and diffed them against those crates; all 26 are identical. I also diffed the 30.0.1 crates against the versions `Cargo.lock` pins (wgpu, wgpu-core, wgpu-hal and wgpu-types 30.0.2) and against `vendor/wgpu-core`, which is 30.0.2 plus gfx-rs/wgpu `59209f27`. Line numbers shift in 30.0.2's `command/mod.rs` and `resource.rs`, but the behaviour these entries cite is unchanged. The one exception is the TLAS dependency list, which the vendored copy deduplicates (L7-T05).

**Other sources.**
- Issues 6762, 9946, 10450 and 10512, and PR 10187: read through `gh api` (body, state, dates and timeline). For PR 10187 I also compared each tag against merge commit `786ecb9c`.
- The Vulkan chapter: fetched raw at tag `v1.4.367`.
- Wald 2003: fetched from the author's site and read with `pdftotext` at p. 3. The PDF does not name its venue, so I confirmed it through Crossref (DOI 10.1109/pvgs.2003.1249045).
- The Lumen deck: fetched, 27 MB, and read with `pdftotext` at pp. 118–120 and p. 1 for the authors.

No source was left unopened.

### Registry verdicts

| key | registry line | verdict | what was opened, what it says, and whether each citing site holds |
|---|---|---|---|
| `vulkan-1.4.367-partitioned-tlas` | docs/references.toml:4803 | confirmed | Raw `chapters/accelstructures.adoc` at `v1.4.367`, lines 4845–4893. Each fragment is verbatim, `can:` markup included: L4845-4847, L4851-4854, L4866-4867, L4878-4879, L4888-4889. Note:48 ("specifies partitions and a global partition for frequently updated instances") holds. Note:62 ("Vulkan has NVIDIA's partitioned … extensions") holds. |
| `wald-2003-two-level-toplevel-rebuild` | docs/references.toml:4809 | confirmed; citing site unsupported (L7-T03) | `dynamic.pdf` p. 3, Section 3 "Our Approach": the quote is verbatim. The venue is IEEE PVG 2003 per Crossref. Note:52's quoted clause holds, but its attribution "The two-level structure is Wald, Benthin and Slusallek's" goes beyond the source. The paper credits Lext and Akenine-Möller [2001] with a closely related method, developed independently. |
| `wgpu-30-as-size-info-private` | docs/references.toml:4815 | confirmed | wgpu-core 30.0.1 `resource.rs` L2676-2685 has `pub(crate)` raw, size_info, max_instance_count and instance_buffer, and Blas L2527-2531 likewise. wgpu `api/tlas.rs` and `api/blas.rs` have no size getter (searched for `pub fn`). Note:50 ("keeps the figure crate-private") holds. |
| `wgpu-30-command-encoder-send-sync` | docs/references.toml:4833 | confirmed | wgpu 30.0.1 `api/command_encoder.rs` L27 (`assert_impl_all!(CommandEncoder: Send, Sync)` under `cfg(send_sync)`), `api/tlas.rs` L28 (WasmNotSendSync) and `api/queue.rs` L19. Note:36 ("`Send` and `Sync` on native") holds. |
| `wgpu-30-device-as-hal` | docs/references.toml:4839 | confirmed | wgpu 30.0.1 `api/device.rs` L667-673: the quote is verbatim at L668-670. Note:50 and code acceleration_structures.rs:116 hold. |
| `wgpu-30-encode-at-finish` | docs/references.toml:4845 | confirmed | wgpu-core 30.0.1 `command/mod.rs` L1283-1285 is verbatim. `finish` at L1320-1328 calls `encode_commands`. Note:32 ("encodes the build at `finish`") holds. |
| `wgpu-30-encode-snatch-read` | docs/references.toml:4851 | confirmed | `command/mod.rs` L1093-1098 is verbatim, and the guard is local to the whole of `encode_commands`. Note:36 holds. |
| `wgpu-30-lowest-unmodified-unused` | docs/references.toml:4857 | confirmed | wgpu 30.0.1 `api/tlas.rs` L97-98 is verbatim. In wgpu-core, `lowest_unmodified` appears only in `command/ray_tracing.rs` L184 (a copy), `ray_tracing.rs` L332/L428 (fields) and the trace recorder; it is never read in the build. Note:32 holds. |
| `wgpu-30-poll-snatch-write` | docs/references.toml:4863 | confirmed | `device/resource.rs` L719-736 is verbatim (doc comment at L721, `snatch(&mut self.snatchable_lock.write())` at L736). `poll_and_return_closures` calls it at L804. `Buffer::destroy` takes the lock at `resource.rs` L1078 and `Texture::destroy` at L1638. Both push the deferred lists (L1198, L1712). Note:36 holds. |
| `wgpu-30-prefer-fast-build-metal` | docs/references.toml:4869 | confirmed | wgpu-hal 30.0.1 `metal/conv.rs` L486-491 is verbatim, and wgpu-types `ray_tracing.rs` L140-142 defines the flag. Note:52 ("wgpu maps it to Metal's") holds. |
| `wgpu-30-tlas-barrier-before-build` | docs/references.toml:4880 | confirmed | `command/ray_tracing.rs` L1194-1205 is verbatim. `build_blas` is called at L367, before the TLAS staging at L377, and places one global barrier. Note:42 holds. |
| `wgpu-30-tlas-binding-at-creation` | docs/references.toml:4886 | confirmed | `device/resource.rs` L3219-3252: `insert_single` at L3229 and `try_raw` at L3252. Note:42 holds. |
| `wgpu-30-tlas-dependency-per-instance` | docs/references.toml:3392 | confirmed at v30.0.1; citing site stale (L7-T05) | `command/ray_tracing.rs` L269-271 is verbatim at 30.0.1. The engine builds `vendor/wgpu-core`, which pushes a dependency only for a new BLAS (`vendor/wgpu-core/src/command/ray_tracing.rs`, the dedup hunk at 30.0.1's L271). Note:32, under "(today)", still says each submit walks one dependency per Instance. |
| `wgpu-30-tlas-instance-blas-only` | docs/references.toml:4892 | confirmed | wgpu 30.0.1 `api/blas.rs` L82 is verbatim. wgpu-types `features.rs` has no "partition" or "cluster" (grep). Note:62 holds. |
| `wgpu-30-tlas-instance-fields` | docs/references.toml:487 | confirmed; code site unsupported (L7-T06) | wgpu 30.0.1 `api/blas.rs` L56-67 is verbatim, with gaps. Note:56 ("8-bit instance mask carries a near/far split") holds, and so does code :989 ("a mask that shares a bit"). Code :1281 says wgpu "keeps the lower 24 bits". The source says higher bits make the instance invalid, and wgpu-core rejects them at `command/ray_tracing.rs` L236-239. |
| `wgpu-30-tlas-instance-staging` | docs/references.toml:481 | confirmed | `command/ray_tracing.rs` L244 is verbatim, inside the L235 loop over `iter().flatten()`, which skips empty slots. Note:32 and code :750 hold. |
| `wgpu-30-tlas-instances-capacity-vec` | docs/references.toml:4904 | confirmed | wgpu 30.0.1 `api/device.rs` L777-784 is verbatim. The backend maps every slot (`backend/wgpu_core.rs` L2874-2887), and core keeps the `Option`s (`command/ray_tracing.rs` L163-180), so the build walks every slot. Note:32 holds. |
| `wgpu-30-tlas-scratch-per-build` | docs/references.toml:4910 | confirmed | `command/ray_tracing.rs` L225-226 and L314 are verbatim. `scratch.rs` L16-27 creates the buffer through wgpu-hal, and it is freed via temp_resources at L460-462. Note:32 and code :909 ("a scratch buffer for each build as the encoder finishes") hold. The vendored copy is unchanged here. |
| `wgpu-30-tlas-staging-per-build` | docs/references.toml:4922 | confirmed | `command/ray_tracing.rs` L377-384 is verbatim, and the copy is at L432-436. Note:32 holds. |
| `wgpu-30-tlas-tracker-stateless` | docs/references.toml:4928 | confirmed | wgpu-core 30.0.1 `track/mod.rs` L635-636 is verbatim. Note:42 holds. |
| `wgpu-30-validate-tlas-dependencies` | docs/references.toml:3422 | confirmed | `command/ray_tracing.rs` L515-523 is verbatim, with a gap. Note:32 holds for 30.0.1; on the vendored copy, see L7-T05. |
| `wgpu-hal-30-as-raw-private` | docs/references.toml:4934 | confirmed | wgpu-hal 30.0.1 `metal/mod.rs` L1389-1392: the field is private and its `fn as_raw` at L1395 has no `pub`. `vulkan/mod.rs` L795-800: raw, buffer and allocation are private. Note:50 holds. |
| `wgpu-hal-30-build-sizes` | docs/references.toml:4940 | confirmed | `lib.rs` L1194-1197 is verbatim. Note:50 and code :116 hold. |
| `wgpu-hal-30-dx12-build-sizes` | docs/references.toml:4946 | confirmed | `dx12/device.rs`: fn at L2461, quote verbatim at L2575-2583. Note:50 holds. |
| `wgpu-hal-30-metal-as-barrier-empty` | docs/references.toml:4952 | confirmed | `metal/command.rs` L1877-1881 has an empty body. DX12 places a null UAV barrier (`dx12/command.rs` L1793-1815) and Vulkan a global `MemoryBarrier` (`vulkan/command.rs` L747-772). Note:42 holds. |
| `wgpu-hal-30-metal-build-sizes` | docs/references.toml:4970 | confirmed | `metal/device.rs` L2076-2091 is verbatim. It calls `conv::map_acceleration_structure_descriptor` at L2081. Note:50 holds. |
| `wgpu-hal-30-metal-instances-buffer-unwrap` | docs/references.toml:4976 | confirmed | `metal/conv.rs` L384-390 is verbatim, with `instances.buffer.unwrap()` at L390. The size query uses this function. Note:50 and code :57 hold. |
| `wgpu-hal-30-metal-one-queue` | docs/references.toml:4982 | confirmed | `metal/adapter.rs` L75-79 is verbatim, and `Queue::as_raw` is at `metal/mod.rs` L492-494, inside `impl Queue` from L475. Note:34 holds. |
| `wgpu-hal-30-vulkan-build-sizes` | docs/references.toml:4988 | confirmed | `vulkan/device.rs`: fn at L2629, quote verbatim at L2762-2773. Note:50 holds. |
| `wgpu-issue-10450` | docs/references.toml:4994 | confirmed | `gh api`: open, created 2026-09-24. The quote is verbatim from the body. The timeline shows no linked fix. Note:40 holds. |
| `wgpu-issue-10512` | docs/references.toml:5000 | confirmed | `gh api`: open, created 2026-10-01. The quote is verbatim. Note:40 holds. |
| `wgpu-issue-6762` | docs/references.toml:5006 | confirmed | `gh api`: open, updated 2026-10-03T03:16:17Z. The quote is verbatim from "### Features" and "### Nice to have", including the double space after "PTLAS -". Note:62 holds. |
| `wgpu-issue-9946` | docs/references.toml:5012 | confirmed | `gh api`: open, created 2026-07-27. The quote and the cost (60-80 ms at 100,000 instances, debug; "3-4x faster" in release) are verbatim. Note:32 holds. |
| `wgpu-pr-10187` | docs/references.toml:5019 | confirmed; citing site overstates (L7-T04) | `gh api`: merged 2026-08-30 as `786ecb9c`. Comparing `v30.0.1` (tagged 2026-08-22) and `v30.0.2` with that commit reads "diverged" both times, so neither tag contains it. The quote is verbatim. The body ends "Still working on the full implementation on top of this." Note:62 states the hand-built TLAS as a capability. |
| `wgpu-types-30-tlas-binding-array` | docs/references.toml:5025 | confirmed | wgpu-types 30.0.1 `features.rs` L1450-1462 is verbatim. The feature is set at `dx12/adapter.rs` L627 and `vulkan/adapter.rs` L979, with no Metal site (grep). Note:44 holds. |
| `wright-2022-lumen-far-field-hlod` | docs/references.toml:5037 | confirmed | Lumen PDF pp. 119-120 speaker notes: all three fragments are verbatim. The authors on p. 1 are Wright, Narkowicz and Kelly. Note:56 holds. |
| `wright-2022-lumen-tlas-instance-limit` | docs/references.toml:5050 | confirmed (quote); form: no `checked` (L7-T01); citing site unsupported (L7-T02) | Lumen PDF p. 118: the slide bullets and notes are verbatim. The entry has no `checked` line, the only one of the registry's 1,099 entries without one. Note:52 says early tests "found" the limit, where the source says early experiments "suggested" it. |

Totals: 37 entries opened in full, 37 with quotes confirmed verbatim and sections pointing to the right place, none dead, none unopened. Six Findings: one is the form of an entry (L7-T01), five are citing sites (four in the note, one in code).

## Scope rulings

The verifier's rulings on questions the readers left open.

- L7-L06 and L7-M01 name the same sentence; L7-L06 survives and carries
  L7-M01's expected wording, and L7-M01 is killed as its duplicate.
- L7-S03 named three lines with one slip; one Finding is one defect at one
  location, so it is three rows, L7-S03 (`:15`), L7-S06 (`:32`) and
  L7-S07 (`:112`), with the evidence under L7-S03.
- L7-B05 sits on a line M5 changed but in a sentence M5 left word for
  word, an unchanged clause the brief rules out; killed on scope, its
  facts holding (see Left for a full lane 7).
- L7-T06 is a code comment that cites a key a scoped note cites, so it is
  in scope by the brief.
- A clause that carries its own date and was true at it (L7-B02) is not a
  defect; a note whose header dates only its research is not dated
  clause by clause, so its stale present-tense sentences stand (L7-F02
  and the S Findings).

## Findings

The 28 rows that survive the verifier, in id order; the `placement`
column is the triage's. A narrowed Finding keeps the reader's claim in
its `claim` cell and carries the rewritten claim in its `verifier` cell.
The evidence blocks follow the table; the killed Findings' blocks follow
them, kept so that the next audit does not re-find them.

| id | claim | severity | where | evidence | expected | verifier | placement |
|---|---|---|---|---|---|---|---|
| L7-A01 | The glossary defines a LOD error as how far a LOD strays from LOD 0. The code stores, and ADR 0026 says it stores, the simplifier's own approximate measure, which can fall short of the true deviation by a tenth on `world`'s roofs and by up to three times on `orrery`'s spheres. | drift; load-bearing for the term (the LOD threshold is defined on it); misleads a reader who takes the stored error as the deviation or as a bound | `CONTEXT.md:32-33` | block below | "LOD error: the simplifier's estimate of how far a LOD strays from LOD 0, in metres: an approximate measure, not a bound" (or wording to that effect), matching ADR 0026:64, `src/render/shaders/shadow_mask.wgsl:127-130` and `[ref:meshoptimizer-0.25-simplify-error]` | `confirmed`: `CONTEXT.md:32` reads "How far a LOD strays from LOD 0, in metres". ADR 0026:64 says "A LOD error is what the simplifier reports, which on `world`'s flattened roofs is a tenth under the distance they moved", and `src/render/shaders/shadow_mask.wgsl:127` says "A cooked LOD error is an approximate measure, no bound". The term entered in M5 (`cbbad8c6`, 2026-11-14). It is in scope through ADR 0026:64, which states the term against the glossary. Load-bearing after: yes (a `CONTEXT.md` term, unchanged). |  |
| L7-A02 | ADR 0026 says a Far proxy is requested when its Chunk comes within the View distance. The streamer requests it earlier while the camera closes on it: up to the Chunk's closing speed times the streamer's lead of frames, capped at the View distance plus a Chunk's edge. | drift; not load-bearing (the release radius and the residency bound hold); misleads a reader working out when a proxy is requested or why one is rarely Late | `docs/adr/0026-the-far-field-draws-a-far-proxy-per-chunk.md:13` | block below | "requested ahead of its Chunk's coming within the View distance, as a Chunk is ahead of the Load radius, and released past the View distance plus one Chunk's edge" | `confirmed`: ADR 0026:13 says "requested when its Chunk comes within the View distance". `src/streaming/mod.rs:229-247` widens requests to `(load_radius_metres + lead_metres).min(hold_radius_metres)`, with `lead_metres = closing_metres.max(0.0) * lead_frames`, and `of_far_proxies` (`:147-157`) is that streamer at the View distance. Load-bearing after: no. |  |
| L7-B01 | ADR 0007 says the milestone exit's dependency review in `docs/agents/workflow.md`, The ladder, checks its three reopen conditions, but that section's dependency-review clause names none of them. | drift; load-bearing (a reopen trigger); misleads whoever runs an exit from `workflow.md` | `docs/adr/0007-wgpu-is-the-permanent-graphics-api.md:9` | see block | `workflow.md`, The ladder, names ADR 0007's three checks, as `CLAUDE.md:104` already does, or the ADR points where they are written | `narrowed`: Narrowed claim: `docs/agents/workflow.md:13`'s dependency-review clause ("lists the releases each dependency pinned ... has published since the last exit") omits the three reopen checks that ADR 0007:9 says that review runs. `CLAUDE.md:102-105` does carry the pointer ("checks ADR 0007's reopen trigger there"). The ADR clause itself is not false. The gap is in the doc it points to, and no number or fact rests on it. Load-bearing after: no (moved from yes). |  |
| L7-B03 | The `meshoptimizer-0.25-vertex-lock` entry's section puts the vertex lock flags at L421-L426, but the flag its quote includes is at L427. | drift; not load-bearing (form); misleads a reader opening the cited lines | `docs/references.toml:6571` | see block | "the vertex lock flags (L421-L427)" | `confirmed`: `docs/references.toml:6571` gives the section as "the vertex lock flags (L421-L426)". `meshopt-0.6.2/vendor/src/meshoptimizer.h:427` is "meshopt_SimplifyVertex_Lock = 1 << 0,", which the quote includes. Load-bearing after: no (form). |  |
| L7-B04 | The streamed-worlds note says upstream deduplicated the TLAS dependencies "after 30.0.1", but the change merged on 2026-07-17, before 30.0.1 was published on 2026-08-22. | drift; not load-bearing; misleads on which releases lack the fix | `docs/research/acceleration-structures-for-streamed-worlds.md:32` | see block | as the registry's section says: merged into trunk after 30.0.0, and in neither 30.0.1 nor 30.0.2 | `confirmed`: Note :32 says "upstream deduplicated them after 30.0.1 [ref:wgpu-pr-9835]". `gh api` gives `merged_at` 2026-07-17T07:29:35Z and v30.0.1 `published_at` 2026-08-22T02:57:01Z. `docs/references.toml:3459` agrees: "after v30.0.0; not in v30.0.1 or v30.0.2". The same sentence has a second defect, filed separately as L7-T05. Load-bearing after: no. |  |
| L7-F01 | The note says `simplify_sloppy` under the 3,863-triangle cap moves outlines "by up to 12 pixels", but on the input the decision words (the merged Chunk, windows in) the probe measured 57 pixels; 12.4 is the shell-only row, windows dropped by hand first. | drift; not load-bearing (it understates a rejected option, so the rejection stands), but ADR 0026's Considered options repeat it, and it misleads whoever reopens clustering | `docs/research/far-proxy-baking.md:5` (also `:40`, `:156`; ADR 0026 `docs/adr/0026-the-far-field-draws-a-far-proxy-per-chunk.md:53`) | see block | "up to 57 pixels on the merged Chunk (12 on its shell alone)", or E1's and the summary's figure tied to the shell row it comes from | `confirmed`: `docs/research/far-proxy-baking.md:5` says "moves outlines by up to 12 pixels", and `:156` and ADR 0026:53 repeat 12. The note's own `:110` row reads "`simplify_sloppy` \| merged source \| 2,948 ... \| 56.6 / 16.0 px"; 12.4 is the shell row at `:111`. `1c8bd5e2:805/probe.log` line 64 reads "sloppy-cap \| all \| 2948 ... \| 56.6 \| 16.0". Load-bearing after: no. |  |
| L7-F02 | The note sets the densest view's 1.26 M proxy triangles "against the Meshlet triangle Arena's 2,202,009", but at the frozen commit that Arena holds 1,426,063 triangles. | drift; not load-bearing (ADR 0017's 2026-11-16 amendment re-derived the shares from cooked Far proxies), but the note's headroom reads 57% where the code gives 88% | `docs/research/far-proxy-baking.md:127` | see block | the figure dated to `d4ac21eb`'s shares, or 1,426,063 with ADR 0017's measured fill (69% to 73%) | `confirmed`: `far-proxy-baking.md:127` says "against the Meshlet triangle Arena's 2,202,009". `src/assets/pools.rs:19` reads `[658, 130, 11, 133, 68]`; 83,886,080 × 68/1000/4 = 1,426,063. The figure was right at the note's last edit (`638a8631`, 2026-11-15) and moved in `308e7c67` (2026-11-16), which left the note unchanged. The note's "measured at `d4ac21eb`" header dates the probe, not every clause after M5 edits, so this is not killed as B02 was. Load-bearing after: no. |  |
| L7-L01 | erikson-2001-hlod's quote is the paper's title and venue, which do not hold the claim the note cites it for | `drift`, not load-bearing, form; misleads a reviewer checking the citation against the quote | `docs/references.toml:3347`; cited at `docs/research/lod-hlod-and-far-field.md:53` | see block | a quote from the abstract that says what an HLOD is | `confirmed`: `docs/references.toml:3347`'s quote is the title and venue, which Crossref confirms verbatim. The note at :53 cites it for "replace groups of objects with merged simplifications". The OpenAlex abstract supports the claim ("HLODs ... serve as higher fidelity drastic simplifications of entire branches of the scene graph"), but the quote does not hold it. Load-bearing after: no (form). |  |
| L7-L02 | karis-2021-instance-cull-cost's quote gives a table as running text, not as "cell: cell" rows | `drift`, not load-bearing, form; the pairing of counts to labels is lost | `docs/references.toml:5729` | see block | "Main pass Instances pre-cull: 896322 [...] Instances post-cull: 3668 [...]" | `confirmed`: `docs/references.toml:5729` reads "Main pass Instances pre-cull 896322 Instances post-cull 3668 [...]". Karis p. 109 (`pdftotext`) is a label/value table. The header, `:10`, asks for "a table's rows as "cell: cell"". Load-bearing after: no (form). |  |
| L7-L03 | frostbite-2019-strand-pixel-width's cost inserts four bracketed words the source does not have | `drift`, not load-bearing, form (quote not verbatim) | `docs/references.toml:5649` | see block | the p. 34 table as "cell: cell" rows, gaps only as `[...]` | `confirmed`: `docs/references.toml:5649` reads "Long hair 10 000 [strands] 240 000 [points] 4.7ms [physics] 4.5ms [render]". Frostbite p. 34 has a header row "Asset # Strands # Points Physics Render" and no such words inline. The header allows only `[...]`. Load-bearing after: no (form). |  |
| L7-L04 | The summary says alpha-to-coverage draws its samples from "one nested family", which no source says and the WebGPU spec rules out as a guarantee | `drift`, not load-bearing (ADR 0027 rests on platform choice and the WebGPU exclusion, not on nesting); misleads a reader of the note on why alpha-to-coverage fails | `docs/research/lod-hlod-and-far-field.md:5` | see block | "because the mask a given alpha draws is the platform's choice, so the two levels' masks need not complement" | `confirmed`: `lod-hlod-and-far-field.md:5` says "alpha draws its samples from one nested family". WebGPU `688f043` `index.bs:17018` says "platform-dependent and can vary for different pixels", and `:17024` says "Not all platforms guarantee that the number of bits ... monotonically increases". The note's own `:104` leaves the M4 Pro's pattern open. Load-bearing after: no. |  |
| L7-L05 | "Where Praxis stands" says the depth pre-pass has no fragment stage with rays off and that the lit pass's phase-two depth draw runs the empty `fs_depth`; at the frozen commit both also draw twins whose fragment stage writes `sample_mask` | `drift`, not load-bearing; the note's state of the code predates ADR 0027's twins and misleads a reader of the main pass | `docs/research/lod-hlod-and-far-field.md:11` | see block | the paragraph dated to its research, or naming the twins and `fs_thin_depth` | `confirmed`: Note :11 says "with them off it has no fragment stage, and phase two's depth draw in the lit pass runs the empty `fs_depth`". `src/render/main_pass.rs:157-168` and `:182-186` build the thin twins with `depth_fragment("fs_thin_depth", ...)`, and `main.wgsl:187` reads `fn fs_thin_depth(...) -> @builtin(sample_mask) u32`. Load-bearing after: no. |  |
| L7-L06 | The note says the two-level fade runs "usually through screen-space dithering", citing a source whose "usually" covers dithering or transparency | `drift`, not load-bearing; small overreach | `docs/research/lod-hlod-and-far-field.md:45` | see block | "through screen-space dithering or transparency" | `confirmed`: Note :45 says "usually through screen-space dithering [ref:unity-6000.0-lodgroup-transition-height]". The live Unity page reads "Unity usually implements the cross-fading by using either screen-space dithering or transparency". Take M01's expected wording as the fix. Load-bearing after: no. |  |
| L7-L07 | Item 5 says the mask in the depth passes comes "at a cost unmeasured"; ADR 0027 records the cost on the M4 Pro | `drift`, not load-bearing; a clause the M5 measure made false | `docs/research/lod-hlod-and-far-field.md:47` | see block | the cost as ADR 0027 states it, or the clause dated | `confirmed`: Note :47 ends "at a cost unmeasured". ADR 0027:20 reads "a `sample_mask` output in both depth passes and the lit pass cost `town`'s GPU frame 0.73 ms with shadow rays on". The prototype comment it cites was posted 2026-11-14T10:17Z, before the note's last edit (`c42712e5`, 17:15 +0100). Load-bearing after: no. |  |
| L7-L08 | The note applies Meister et al.'s ray-traversal finding on overlapping top-level BVH bounds to raster cell culling without saying so | `drift`, not load-bearing; misleads on what the source measured | `docs/research/lod-hlod-and-far-field.md:65` | see block | "In a two-level BVH, overlapping top-level bounds cut ray culling efficiency", or the sentence moved to item 17 | `confirmed`: Note :65, in item 12 (CPU cells and GPU Instance culling for the raster), says "Overlapping top-level bounds cut culling efficiency". Meister p. 11 reads "can quickly reduce the culling efficiency of the top-level BVH, as a ray intersecting the overlapping region will need to sequentially intersect all overlapping objects", a finding about rays. Load-bearing after: no. |  |
| L7-L09 | Not verified lists the cost of a `sample_mask` output in the depth pre-pass on Apple's GPU, which ADR 0027 records as measured | `drift`, load-bearing for ADR 0027's "the mask in the pipelines every Instance draws through"; a reader takes the cost as open | `docs/research/lod-hlod-and-far-field.md:105` | see block | the line removed or answered from ADR 0027 | `confirmed`: Note :105 lists "What a `sample_mask` output in the depth pre-pass costs on Apple's tile-based GPU" as Not verified. ADR 0027:20 records it on the M4 Pro. The ADR rests on its own figure, which is correct, not on the note's line. Load-bearing after: no (moved from yes). |  |
| L7-L10 | Not verified says no Baseline records `world`'s slots in use; `world`'s M4 Pro Baseline records them and the per-slot time | `drift`, load-bearing for ADR 0013's guard ("the cost a slot can be read from a Baseline"); a reader takes `world`'s per-slot cost as unknown | `docs/research/lod-hlod-and-far-field.md:108` | see block | `world`: 394.3 µs median over 307,263 slots in use, about 1.28 ns a slot (M4 Pro Baseline) | `confirmed`: Note :108 says "`world`, whose slots in use no Baseline records". `bench/baselines/apple-m4-pro/world.json` holds `instance_slots_in_use` 307263, `culling_per_slot_microseconds` median 394.291 and `counter_frames/instance_slots_in_use` 164559. The expected figure should name which count it divides by. ADR 0013:5's guard holds regardless. Load-bearing after: no (moved from yes). |  |
| L7-S01 | The note says a discrete LOD change is a BLAS change and so a static TLAS rebuild under ADR 0020:20's triggers, where ADR 0020:20, amended in M5, and the code build every BLAS from LOD 0, so a LOD switch changes no BLAS and starts no build. | drift; load-bearing (ADR 0020's M5 LOD 0 amendment); misleads a reader weighing L1 against what was decided, and cites the very clause it contradicts | `docs/research/acceleration-structures-for-streamed-worlds.md:58` | block below | The sentence dated to the survey, or rewritten: since 2026-11-14 every BLAS holds its Mesh's LOD 0, so a LOD switch starts no static TLAS build (ADR 0020:20). | `narrowed`: Narrowed claim: note :58 says without condition "A discrete LOD change is a BLAS change, so a static TLAS rebuild under ADR 0020:20's triggers". That holds only for a BLAS that follows the drawn LOD. ADR 0020:20 (amended 2026-11-14) says "Every BLAS is built from its Mesh's LOD 0 ... so a LOD switch changes no BLAS and starts no static TLAS build", and `src/assets/pools.rs:463-465` loads "LOD 0's index stream and its BLAS's build". The sentence is the premise of the decision, true as a conditional, and false only as a description of today's code. Load-bearing after: no (moved from yes). |  |
| L7-S02 | Technique 8 says Praxis's static rebuild on a Chunk change can move the anchor and that a camera which crosses no Chunk boundary keeps the old anchor while the ray offset grows with distance, where the code keeps the anchor through Chunk-change builds and starts a re-anchoring build once the camera's Anchor is more than a Chunk's edge away, bounding the offset under 4.9 mm. | drift; load-bearing (the R1/R2 options ADR 0020:20's and ADR 0014's M5 amendments decide between); misleads a reader about the problem R2 was chosen to bound, which is now bounded | `docs/research/acceleration-structures-for-streamed-worlds.md:46` | block below | Dated to the survey, or rewritten: a static build keeps the Anchor until the camera's is more than a Chunk's edge from it, where a build re-anchors it (ADR 0020:20, ADR 0014). | `confirmed`: Note :46 says "Praxis rebuilds its static TLAS on a Chunk change, which can move the anchor ... the ray offset grows with its distance from it". `src/render/acceleration_structures.rs:190-195` reads "past which a build re-anchors it ... less than 4.9 mm to its offset", and ADR 0020:20 reads "A static build keeps the static TLAS's Anchor until the camera's is more than a Chunk's edge from it". Load-bearing after: no (moved from yes). |  |
| L7-S03 | The note labels the in-frame, in-place static TLAS rebuild as Praxis "today", where since 2026-11-08 a worker builds a spare static TLAS that is bound from the frame after its submit. | drift; not load-bearing; misleads a reader comparing options with the current code | `docs/research/acceleration-structures-for-streamed-worlds.md:15` | block below | "Praxis at the survey (2026-11-05)", or "Praxis until 2026-11-08", in row 1, §1's heading and option S2. | `confirmed`: `:15` reads "(Praxis today) \| ... \| Is Praxis", `:32` reads "(today)" and `:112` reads "**S2. In place**, as today". `src/render/acceleration_structures.rs:204-207` reads "Two TLASes hold it: rays trace the bound one while a worker thread brings the spare up to date". ADR 0020:20 reads "The static rebuild is off the frame's critical path: two static TLASes". L7-S03 keeps `:15`, and the report files `:32` and `:112` as their own rows. Load-bearing after: no. |  |
| L7-S04 | The probe table names its capacity `STATIC_TLAS_CAPACITY`, a constant the code no longer has; the static TLAS's capacity has been derived from the acceleration structures' Budget since 2026-11-06, 264,720 entries at the checked-in Budgets. | drift; not load-bearing (the probe's figures stand); misleads a reader who greps the code for the constant | `docs/research/acceleration-structures-for-streamed-worlds.md:68` | block below | "the static TLAS's capacity at the survey" without the constant's name, or the name dated. | `confirmed`: Note :68 reads "(`STATIC_TLAS_CAPACITY`)". `grep -rn STATIC_TLAS_CAPACITY src tests scripts` finds nothing, and the last code touch is `3654003f` (2026-11-06). `src/render/renderer.rs:211` reads `STATIC_TLAS_PER_MILLE: u64 = 570`, and `:1094` reads `static_tlas_instances: 264_720`. Load-bearing after: no. |  |
| L7-S05 | The Load radius table says 768 m falls "past ADR 0003's 524,288 slots", where ADR 0003:3 has said 517,376 slots at the M4 Budgets since 2026-11-09. | drift; not load-bearing (624,295–720,340 is past either figure); misleads a reader taking the slot count from the note | `docs/research/acceleration-structures-for-streamed-worlds.md:101` | block below | "past ADR 0003's 517,376 slots", or the figure dated. | `narrowed`: Narrowed claim: note :101 attributes "524,288 slots" to ADR 0003, which says "517,376 slots at the Budgets M4 set" (ADR 0003:3). The code's count is 488,448 (`src/render/renderer.rs:1093`, from `66390f70`, 2026-11-16: "Instance slots: 488,448, down from 517,376"). The fix is the code's figure or a dated one, not ADR 0003's. ADR 0003:3's 517,376 and 136.2 bytes a slot, and ADR 0017:18's same 136.2, were made false by M5 but sit in unchanged clauses outside this audit. Load-bearing after: no. |  |
| L7-S06 | The note labels the in-frame, in-place static TLAS rebuild as Praxis "today", where since 2026-11-08 a worker builds a spare static TLAS that is bound from the frame after its submit. | drift; not load-bearing; misleads a reader comparing options with the current code | `docs/research/acceleration-structures-for-streamed-worlds.md:32` | block of L7-S03 | "Praxis at the survey (2026-11-05)", or "Praxis until 2026-11-08", in row 1, §1's heading and option S2. | `confirmed`: Split from L7-S03, one Finding per location; the verifier's reason there covers `:32`. Load-bearing after: no. |  |
| L7-S07 | The note labels the in-frame, in-place static TLAS rebuild as Praxis "today", where since 2026-11-08 a worker builds a spare static TLAS that is bound from the frame after its submit. | drift; not load-bearing; misleads a reader comparing options with the current code | `docs/research/acceleration-structures-for-streamed-worlds.md:112` | block of L7-S03 | "Praxis at the survey (2026-11-05)", or "Praxis until 2026-11-08", in row 1, §1's heading and option S2. | `confirmed`: Split from L7-S03, one Finding per location; the verifier's reason there covers `:112`. Load-bearing after: no. |  |
| L7-T01 | Registry entry `wright-2022-lumen-tlas-instance-limit` has no `checked` date. | drift; form, not load-bearing. It misleads a reader or test that dates each entry. | docs/references.toml:5050 | Entry runs from L5050 to L5053 and has url, section and quote but no `checked`. A scan of all 1,099 entries finds it is the only one. It had `checked = 2026-11-05` when added in `7668f983` and lost it in the squash merge of `9493ec8d`. | `checked = 2026-11-21`: today's date, since the quote reads verbatim on p. 118. | `confirmed`: `docs/references.toml:5050-5053` has no `checked` line. A scan of all 1,099 entries finds it is the only one without. `git show 7668f983:docs/references.toml` had `checked = 2026-11-05`. Load-bearing after: no (form). |  |
| L7-T02 | Note's claim that early Lumen tests "found" a 100K limit drops the source's hedge, "suggested". | drift; not load-bearing (Published costs paragraph). It misleads a reader weighing Lumen's limit as a measurement. | docs/research/acceleration-structures-for-streamed-worlds.md:52 | "early Lumen tests found its per-frame rebuild "limited to 100K instances" [ref:wright-2022-lumen-tlas-instance-limit]" | The source says "early experiments suggested that our model would be limited to 100K instances". | `confirmed`: Note :52 reads "early Lumen tests found its per-frame rebuild "limited to 100K instances"". The Lumen PDF p. 118 notes read "early experiments suggested that our model would be limited to 100K instances". The quoted phrase is lifted from the hedged sentence. Load-bearing after: no. |  |
| L7-T05 | The note's "(today)" paragraph says each submit walks one dependency per Instance, but Praxis's vendored wgpu-core records one per distinct BLAS. | drift; stale. Not load-bearing for an ADR, but it misleads a reader sizing the main thread's submit cost. | docs/research/acceleration-structures-for-streamed-worlds.md:32 | "**1. Rebuilding in the frame's encoder (today).** … At submit, each build and each pass that binds the TLAS walks one dependency per Instance [ref:wgpu-30-tlas-dependency-per-instance] [ref:wgpu-30-validate-tlas-dependencies]; upstream deduplicated them after 30.0.1" | Date the paragraph to wgpu 30.0.1, or say Praxis carries the dedup in `vendor/wgpu-core` (gfx-rs/wgpu `59209f27`). | `confirmed`: Note :32, under "(today)", reads "walks one dependency per Instance". `vendor/wgpu-core/src/command/ray_tracing.rs:245` reads `let is_new_dependency = seen_dependencies.insert(blas.tracker_index());`, patched in by `[patch.crates-io]` (`Cargo.toml:53`). The note's own :134 records the dedup carried. Load-bearing after: no. |  |
| L7-T06 | A code comment says wgpu "keeps the lower 24 bits" of an entry's custom data; wgpu actually rejects any value with a higher bit set. | drift; not load-bearing (the test's bound is right either way). It misleads a maintainer about truncation. | src/render/acceleration_structures.rs:1280-1281 | "/// An entry's custom data is its slot, of which wgpu keeps the lower 24 /// bits [ref:wgpu-30-tlas-instance-fields]." | "of which wgpu accepts only the lower 24 bits". Per the source, a higher bit makes the instance invalid and the build fails validation. | `confirmed`: `src/render/acceleration_structures.rs:1280` reads "of which wgpu keeps the lower 24 bits [ref:wgpu-30-tlas-instance-fields]". `wgpu-30.0.2/src/api/blas.rs` reads "if any bits are outside that range ... the TlasInstance becomes invalid and generates a validation error when built", and `vendor/wgpu-core/src/command/ray_tracing.rs:239` returns `TlasInvalidCustomIndex`. The key is cited at note :56, so its code site is in scope. Load-bearing after: no. |  |

### L7-A01

`CONTEXT.md:32-33`:

```
**LOD error**:
How far a LOD strays from LOD 0, in metres.
```

`docs/adr/0026-the-far-field-draws-a-far-proxy-per-chunk.md:64`, Consequences: "A LOD error is what the simplifier reports, which on `world`'s flattened roofs is a tenth under the distance they moved." ADR 0026:20: "its roofs flat and 0.3 m lower, which the simplifier reports as an error of 0.27 m."

The code stores the simplifier's report. For a Far proxy it stores the absolute error `meshopt::simplify` returns:

```
src/cook/far_proxy.rs:332:            simplified(&positions, &finest, pixel_metres * 2f32.powi(doublings));
src/cook/far_proxy.rs:342:        mesh::push_lod(&mut proxy, error_metres, &[&shell, &ground_geometry])?;
```

For any other Mesh it accumulates the relative error times the extent: `src/cook/mesh.rs:375` `error_metres += relative_error * extent_metres;`.

The shadow rays already assume it is no bound:

```
src/render/shaders/shadow_mask.wgsl:127: // A cooked LOD error is an approximate measure, no bound
src/render/shaders/shadow_mask.wgsl:128: // [ref:meshoptimizer-0.25-simplify-error]: `orrery`'s spheres meet LOD 0 up to
src/render/shaders/shadow_mask.wgsl:129: // three times theirs high (docs/performance.md, Measurements behind the code).
src/render/shaders/shadow_mask.wgsl:130: const LOD_ERROR_MARGIN: f32 = 3.0;
```

Source, `meshoptimizer-0.25-simplify-error`, README line 446 at `v0.25`: "Target error is an approximate measure of the deviation from the original mesh". The term entered the glossary in M5 (`git log -L32,34:CONTEXT.md` names one commit, `cbbad8c6`).

### L7-A02

`docs/adr/0026-the-far-field-draws-a-far-proxy-per-chunk.md:13`, Decision, "Which are Resident": "A Far proxy is requested when its Chunk comes within the View distance and released past the View distance plus one Chunk's edge."

The Far proxies' streamer is the Chunk streamer with the View distance as its load radius (`src/streaming/mod.rs:147-157`, `of_far_proxies` → `Self::new(chunks, view_distance_metres)`). Its request loop runs out past that radius:

```
src/streaming/mod.rs:229:        let widest_request_radius_metres =
src/streaming/mod.rs:230:            (load_radius_metres + metres_per_frame * lead_frames).min(hold_radius_metres);
src/streaming/mod.rs:242:            let lead_metres = closing_metres.max(0.0) * lead_frames;
src/streaming/mod.rs:243:            if metres > (load_radius_metres + lead_metres).min(hold_radius_metres) {
src/streaming/mod.rs:244:                continue;
src/streaming/mod.rs:245:            }
...
src/streaming/mod.rs:268:            if !loaded && !self.in_flight.contains_key(&coordinates) && !waits_to_retry {
src/streaming/mod.rs:269:                self.requests.push((metres, coordinates));
```

`lead_frames` grows to the longest load seen (`:200`), and `far_proxies` changes only the fill flag (`:273`). Release matches the ADR: `:296-298` `hold_radius_metres = load_radius_metres + CHUNK_EDGE_METRES`, and `:213` keeps a loaded Chunk only within it. ADR 0017:15 says the same of Chunks, "Chunks inside the load radius are requested ahead", but ADR 0026 does not carry it over.

### L7-B01

`docs/adr/0007-wgpu-is-the-permanent-graphics-api.md:9`, *Consequences*: "Each milestone exit's dependency review (`docs/agents/workflow.md`, The ladder) checks three things, and this decision reopens when any holds. The newest wgpu release cannot meet the Capability floor (ADR 0021) [...]. `vendor/` carries more than five upstream commits [...]. Or the wgpu that `Cargo.toml` pins is more than two major releases behind the newest published."

`docs/agents/workflow.md:13`, *The ladder*, the exit's only dependency-review clause: "a dependency review lists the releases each dependency pinned in `Cargo.toml` has published since the last exit, and each is taken, as a version bump the owner decides like any other, or skipped with a dated reason on the ladder's map". `grep -rn "0007" docs/agents/` finds only `docs/agents/domain.md:51`, an example ADR quote. `CLAUDE.md:102-105` does say it: "Each milestone's exit reviews the releases [...] (`docs/agents/workflow.md`, The ladder), and checks ADR 0007's reopen trigger there."

### L7-B03

`docs/references.toml:6569-6573`, `["meshoptimizer-0.25-vertex-lock"]`: section "src/meshoptimizer.h, the vertex lock flags (L421-L426) and the documentation of meshopt_simplifyWithAttributes (L467)"; quote includes "/* Do not move this vertex. */ meshopt_SimplifyVertex_Lock = 1 << 0".

Opened `https://raw.githubusercontent.com/zeux/meshoptimizer/v0.25/src/meshoptimizer.h` and meshopt 0.6.2's `vendor/src/meshoptimizer.h`: L422 "Experimental: Simplification vertex flags/locks [...]", L426 "/* Do not move this vertex. */", L427 "meshopt_SimplifyVertex_Lock = 1 << 0,", L467 "vertex_lock can be NULL; [...] 1 denotes vertices that can't be moved" (in `meshopt_simplifyWithAttributes`'s comment, as stated). The url's anchor `#L421-L467` covers L427; the section's own range does not.

### L7-B04

`docs/research/acceleration-structures-for-streamed-worlds.md:32`, *1. Rebuilding in the frame's encoder (today)*: "At submit, each build and each pass that binds the TLAS walks one dependency per Instance [...]; upstream deduplicated them after 30.0.1 [ref:wgpu-pr-9835]."

`gh api repos/gfx-rs/wgpu/pulls/9835`: `merged_at` 2026-07-17T07:29:35Z. `gh api repos/gfx-rs/wgpu/releases/tags/v30.0.1`: published 2026-08-22T02:57:01Z (v30.0.0: 2026-07-01). Registry section, `docs/references.toml:3459`: "merged into trunk 2026-07-17, after v30.0.0; not in v30.0.1 or v30.0.2". The fix is absent from 30.0.1 because that patch release did not take it, not because it came later.

### L7-F01

- Doc: `docs/research/far-proxy-baking.md:5`, **Summary**: "at the cap `simplify_sloppy` moves outlines by up to 12 pixels and `Prune` deletes whole roofs". The same figure at `:40` (Comparison row 13, "Measured: 12 px at the cap") and `:156` (Options, "E1: as the decision words it, `simplify_sloppy` and `Prune` under 3,863 triangles. Measured: outlines move by 12 and 41 pixels."); ADR 0026 `:53`, Considered options: "Vertex clustering and pruning of small components under a cap of 3,863 triangles: measured on `world`'s densest Chunk, they move an outline by 12 and 41 pixels."
- The note's own table, `:110`: "`simplify_sloppy` | merged source | 2,948 | 5.97 m | 712,249 / 16,300 | ... | 56.6 / 16.0 px", and `:111`: "`simplify_sloppy` | shell | 3,568 | ... | 8.0 / 12.4 px".
- Probe log at `1c8bd5e2` (`805/probe.log`): line 20, against the merged source, "sloppy-cap | all | 2948 | 5.9696 | 2489416 | 715059 | 16127 | 668520 | 12267 | 57.3 | 16.0"; line 64, against the shell, "sloppy-cap | all | 2948 | ... | 56.6 | 16.0"; line 78, "sloppy-cap | shell | 3568 | 2.0001 | ... | 8.0 | 12.4". The decision merges "the Chunk's static Placements" (windows included); the shell input is the windows dropped by hand, which E1 does not do. For `Prune` the merged and shell rows agree (41.4, lines 69 and 83), so only the clustering figure is affected.

### L7-F02

- Doc: `docs/research/far-proxy-baking.md:127`, **Measurement: outlines from 256 m**: "The densest view's proxies ... come to 35,022 buildings: 630,396 triangles at 18 a building, and 1.26 M with LOD chains of twice their LOD 0, the decision's allowance, against the Meshlet triangle Arena's 2,202,009 (derived, nothing cooked)."
- Code at the frozen commit: `src/assets/pools.rs:19` `const ARENA_PER_MILLE: [u64; 5] = [658, 130, 11, 133, 68];`, applied at `:76-84` to `bench/budgets.json:11` `"geometry_arenas": 83886080`, aligned to 4 bytes, with one triangle a `u32` (`src/assets/mesh.rs:52-54`, "its triangles in the Meshlet triangle Arena, each its three corners as indices ..., a byte each"): 83,886,080 × 68 / 1000 → 5,704,252 bytes, 1,426,063 triangles.
- At the note's `d4ac21eb`: `git show d4ac21eb:src/assets/pools.rs` line 16, `[487, 317, 6, 85, 105]` → 8,808,036 bytes, 2,202,009 triangles, the note's figure. The share moved in `src/assets/pools.rs` after the note; ADR 0017 (amended 2026-11-16) records the new shares and "leaves each Arena 69% to 73% full there". The note's header dates its measurements to `d4ac21eb`, but this line states the Arena's capacity as a present fact.

### L7-L01

`docs/references.toml:3344-3348`, `["erikson-2001-hlod"]`: quote `'''HLODs for faster display of large static and dynamic environments [...] Proceedings of the 2001 symposium on Interactive 3D graphics'''`. Crossref (`api.crossref.org/works/10.1145/364338.364376`) gives exactly that title and container. The note, heading "The far field", `:53`: "Erikson's HLODs replace groups of objects with merged simplifications [ref:erikson-2001-hlod]." The title says nothing of what an HLOD replaces. The abstract (OpenAlex record of the same DOI) does: "We augment the LODs with automatically-generated hierarchical levels of detail (HLODs) that serve as higher fidelity drastic simplifications of entire branches of the scene graph." The registry's header asks the quote to be "the claim relied on, verbatim".

### L7-L02

`docs/references.toml:5729`: `quote = '''Main pass Instances pre-cull 896322 Instances post-cull 3668 [...] Post pass Instances pre-cull 102804 Instances post-cull 365'''`. Karis 2021 p. 109 is a two-column table, labels (Instances pre-cull, Instances post-cull, Cluster node visits, Cluster candidates, Visible clusters SW, Visible clusters HW) against values. The registry header (`docs/references.toml:9-10`): "a table's rows as "cell: cell"". The entry at `:4029` (haar-aaltonen-benchmark) shows the form.

### L7-L03

`docs/references.toml:5649`: `cost = '''Regular PS4, 900P, no MSAA, all strands rendered [...] Long hair 10 000 [strands] 240 000 [points] 4.7ms [physics] 4.5ms [render] Short hair 15 000 75 000 0.4ms 1.9ms'''`. Frostbite 2019 PDF p. 34 (`pdftotext`): "Asset # Strands # Points Long hair 10 000 240 000 Short hair 15 000 75 000 Physics 4.7ms 0.4ms Render 4.5ms 1.9ms". The bracketed words are editorial, and the registry marks only gaps, as `[...]`, and gives a table's rows as "cell: cell" (`docs/references.toml:9-10`). One other entry carries an editorial bracket (`:6109`, outside my assignment).

### L7-L04

Note, **Summary**, `:5`: "Plain alpha-to-coverage cannot cross-fade two LODs, because alpha draws its samples from one nested family, so the two levels' masks overlap instead of complementing". Golus (golus-2017-alpha-to-coverage) says only that at one alpha "it's going to be the same two coverage samples for everything that renders to that pixel with 50% A2C". The WebGPU spec (gpuweb `688f043` `spec/index.bs:17018-17025`): "The algorithm of producing the extra mask is platform-dependent and can vary for different pixels. [...] Not all platforms guarantee that the number of bits set to 1 in the mask monotonically increases as alpha increases for a given pixel." A nested family needs at least monotonic growth. The note itself, **Not verified**, `:104`: "Whether the M4 Pro's alpha-to-coverage pattern dithers by pixel or grows monotonically; Metal's documentation does not say." ADR 0027:16 gives the sound reason: "Which samples a given alpha keeps is the platform's choice". M5 left the sentence unchanged.

### L7-L05

Note, **Where Praxis stands**, `:11`: "With shadow rays on, its depth pre-pass writes each sample's surface normal in `fs_surface_normals`; with them off it has no fragment stage, and phase two's depth draw in the lit pass runs the empty `fs_depth`." At the frozen commit, `src/render/main_pass.rs:157-168` builds "Thin Depth Pre-pass Pipeline" with `depth_fragment("fs_thin_depth", &[])` when shadow rays are off, and `:182-186` "Thin Lit Pass Depth Pipeline" with `fs_thin_depth`; `src/render/shaders/main.wgsl:187`: `fn fs_thin_depth(in: ThinVertexOutput) -> @builtin(sample_mask) u32`. ADR 0027:10: "Each pipeline of the main pass has a twin whose fragment stage writes the mask". The ordinary pipelines match the note (`main_pass.rs:148-155`, `:177-181`; `main.wgsl:75`, `:82`). The note's M5 edit rewrote the culling figures in the same paragraph and left these clauses.

### L7-L06

Note, **4. Dithered cross-fade per pixel**, `:45`: "Both levels draw during the fade, weighted 1 to 0 and 0 to 1, usually through screen-space dithering [ref:unity-6000.0-lodgroup-transition-height]". The Unity 6000.0 manual, LOD Group component reference (live page opened): "Cross-fading is the technique of rendering two levels at the same time, with a weighting of 1 to 0 for the current LOD level and 0 to 1 for the next LOD level [...] Unity usually implements the cross-fading by using either screen-space dithering or transparency." The sample opened item 4 in full: its other claims hold (rows R71 to R77 above).

### L7-L07

Note, **5. Cross-fade per sample**, `:47`: "In Praxis both depth passes and the depth-equal lit pass would write the same mask, which means adding it to `fs_surface_normals`, or a fragment stage to a pre-pass that has none, at a cost unmeasured." ADR 0027:20, Considered options: "On the M4 Pro at 2560×1440, a `sample_mask` output in both depth passes and the lit pass cost `town`'s GPU frame 0.73 ms with shadow rays on and its gate 0.56 ms, as much as a `discard` there [...] Measured on the M4 Pro alone." The note was last edited 2026-11-14 (`c42712e5`), the day ADR 0027 was accepted.

### L7-L08

Note, **12. Cells on the CPU, Instances on the GPU**, `:65`: "Overlapping top-level bounds cut culling efficiency [ref:meister-2021-bvh-star-two-level]." Meister et al. 2021, p. 11, §5.3.3 Two-level Hierarchy: "Largely overlapping objects (in world space) [...] can quickly reduce the culling efficiency of the top-level BVH, as a ray intersecting the overlapping region will need to sequentially intersect all overlapping objects." The source is a ray-tracing survey; item 12 is about frustum and occlusion culling of cells for the raster. Survey 4 cites the same entry in its ray-tracing context (`acceleration-structures-for-streamed-worlds.md:52`), where it holds.

### L7-L09

Note, **Not verified**, `:105`: "What a `sample_mask` output in the depth pre-pass costs on Apple's tile-based GPU." ADR 0027:20 (quoted under L7-L07) records that cost on the M4 Pro, Apple's GPU, at 2560×1440, and ADR 0027's Decision rests on it ("so no other geometry pays for it", `:3`). The line stands unchanged since the note was written.

### L7-L10

Note, **Not verified**, `:108`: "That the flat cull's per-slot stages scale linearly past `town`'s 254,256 slots, and their cost on `world`, whose slots in use no Baseline records." `bench/baselines/apple-m4-pro/world.json` at the frozen commit: `"instance_slots_in_use": 307263`, `culling_per_slot_microseconds` median 394.291, `gpu_pass_microseconds.culling_phase_one_slots` median 204.5 and `culling_phase_two_slots` median 187.042. ADR 0013:5 (amended 2026-11-14): "the Bench record's counters hold the Instance slots in use, so the cost a slot can be read from a Baseline". The counters arrived in `31fae9d1` (2026-11-14 23:18), after the note's last edit `c42712e5` (2026-11-14 17:15). The linear-scaling half of the line still stands; `world` itself gives a second point (394.3 µs over 307,263 slots, about 1.28 ns a slot, against `town`'s 1.46 ns).

### L7-S01

Note, *Techniques: M5*, 12. Which LOD each BLAS holds, `:58`: "A discrete LOD change is a BLAS change, so a static TLAS rebuild under ADR 0020:20's triggers."

ADR 0020, Consequences, `docs/adr/0020-ray-traced-probe-gi-and-hard-ray-traced-shadows.md:20`, the M5 amendment: "Every BLAS is built from its Mesh's LOD 0, a refitted Mesh's included, whichever LOD the main pass draws, so a LOD switch changes no BLAS and starts no static TLAS build; the index Arena, which only BLAS builds and refits read, holds LOD 0's indices alone (amended 2026-11-14, [...])". Code: `src/assets/pools.rs:463-465` loads "where `mesh_use` has rays trace it, LOD 0's index stream and its BLAS's build"; `src/assets/pools.rs:863` "the LOD 0 its BLAS holds". The note's M5 correction of 2026-11-14 (`c42712e5`) edited `:56`, `:60` and `:62` and left this sentence. The note's sources hold (Lloyd 2020; NVIDIA 2022, General tips; Lumen 2022 p. 112): the slip is the code claim, not the literature.

### L7-S02

Note, *Techniques: M4*, 8. Re-anchoring, `:46`: "Praxis rebuilds its static TLAS on a Chunk change, which can move the anchor at no extra cost; [...] A camera that travels without crossing a Chunk boundary keeps the old anchor, and the ray offset grows with its distance from it (`docs/audit/m3.5/scalability.md`, L4-02)."

Code, `src/render/acceleration_structures.rs:190-195`: "Anchor cells, along any axis, between the camera's Anchor and the bound static TLAS's past which a build re-anchors it, and runs though no static Instance changed: a Chunk's edge. Within it, the camera adds less than 320 m to what a shadow ray composes in the TLAS's frame, and less than 4.9 mm to its offset." `:201-209`: the frame's origin is the corner of "the camera's [Anchor] when the first static Instance took an entry, or when a build last re-anchored it", and "a frame whose camera's Anchor is more than `REANCHOR_CELLS` from the bound TLAS's along an axis" starts a build. ADR 0020:20 (amended 2026-11-16): "A static build keeps the static TLAS's Anchor until the camera's is more than a Chunk's edge from it along an axis, where a build runs though no static Instance changed and re-anchors it at the camera's, so the camera adds less than 4.9 mm [...] (R2's build without R1's move at every static build)". ADR 0014 (M5 diff): "a build moves it only once the camera's Anchor strays more than a Chunk's edge (4 cells)". A Chunk-change build no longer moves the anchor, and the offset no longer grows without bound.

### L7-S03

Note, *Comparison*, row 1, `:15`: "1. Rebuild in the frame's encoder (Praxis today) | Nothing to manage | 254,256 Instances: 17.5–18.1 ms CPU in the frame, GPU in the frame | Is Praxis". The same slip, same sentence kind: *Techniques: M4*, `:32` "**1. Rebuilding in the frame's encoder (today).**"; *Options for M4*, `:112` "**S2. In place**, as today: the frame that rebuilds traces the new TLAS, and the build sits in that frame's GPU time."

Code, `src/render/acceleration_structures.rs:204-207`: "Two TLASes hold it: rays trace the bound one while a worker thread brings the spare up to date and records its build, which the main thread submits (ADR 0016), and the spare is bound from the frame after that submit." ADR 0020:20: "The static rebuild is off the frame's critical path: two static TLASes [...] (amended 2026-11-08, from [this note]'s options B and S1". The note was edited after that (2026-11-10 `9bcee573`, 2026-11-14 `c42712e5`) without these labels changing. The verifier may split this into three Findings, one a location.

### L7-S04

Note, *Measured on the M4 Pro*, `:68`: "Static TLAS of capacity 262,144 (`STATIC_TLAS_CAPACITY`), ms:".

`grep -rn STATIC_TLAS_CAPACITY src` finds nothing at the frozen commit; `git log -S STATIC_TLAS_CAPACITY -- src` last touches it in `3654003f` "Size the Instance slots, lights and static TLAS from the Budgets" (2026-11-06). Now `src/render/renderer.rs:209-211` `STATIC_TLAS_PER_MILLE: u64 = 570` and `:269-272` `static_tlas_instances: entries(share(budgets.acceleration_structures, STATIC_TLAS_PER_MILLE), 2 * TLAS_BYTES_PER_INSTANCE)`; with `bench/budgets.json:15` `"acceleration_structures": 268435456` that is 153,008,209 / 578 = 264,720, the figure `src/render/renderer.rs:1094` asserts.

### L7-S05

Note, *Measured on the M4 Pro*, Load radius table, `:101`: "| 768 m | 39–45 | 624,295–720,340 | past ADR 0003's 524,288 slots |".

ADR 0003, `docs/adr/0003-persistent-scene-transform-buffer.md:3`: "30% of it at 136.2 bytes a slot, its Instance, offsets and visibility bit, in whole 256-slot scan blocks of the culling pass: 517,376 slots at the Budgets M4 set (amended 2026-11-09, [...])". The 524,288 slots at `:100` are the probe's TLAS capacity and stand; `:101` names ADR 0003's count.

### L7-T01

`docs/references.toml` L5050-5053 at `62052fe8`:

```
["wright-2022-lumen-tlas-instance-limit"]
url = "...Lumen-Wright%20et%20al.pdf#page=118"
section = "Wright et al. (Epic Games), Lumen: ..., PDF p. 118, ..."
quote = '''TLAS complexity ○ Rebuilt every frame ○ 100K instance limit, ...'''
```

The next line is the next entry, `["decima-2017-static-tiles"]`. `git show 7668f983:docs/references.toml` has `checked = 2026-11-05` after the quote. In `9493ec8d` (the squash merge that added it) the entry appears without it, and nothing restored it. Scanning every entry for a `checked` line finds this one alone missing.

### L7-T02

The source is the Lumen PDF, p. 118, speaker notes. They read: "Our system rebuilds the top-level acceleration structure every frame and early experiments suggested that our model would be limited to 100K instances." The slide bullet reads "100K instance limit, but 1M expected instances". The note turns the hedged early result into a finding.

### L7-T05

`vendor/wgpu-core/src/command/ray_tracing.rs`, against 30.0.1's L242 and L271:

```
+            let is_new_dependency = seen_dependencies.insert(blas.tracker_index());
...
-            dependencies.push(blas.clone());
+            if is_new_dependency {
+                dependencies.push(blas.clone());
+            }
```

`vendor/README.md` records the commit: "A TLAS build records one dependency per distinct BLAS rather than per Instance". The note's "Not verified" section (L134) records the answer, but the "(today)" paragraph, the Summary (L5) and row 1 ("Praxis today", L15) still describe 30.0.1.

### L7-T06

wgpu 30.0.1 `api/blas.rs` L62-63 reads: "This must only use the lower 24 bits, if any bits are outside that range (byte 4 does not equal 0) the TlasInstance becomes invalid and generates a validation error when built". The registry quote elides the words after "becomes". wgpu-core `command/ray_tracing.rs` L236-239 reads: `if instance.custom_data >= (1u32 << 24u32) { return Err(BuildAccelerationStructureError::TlasInvalidCustomIndex(...`.

### L7-B02 (killed)

`docs/adr/0007-wgpu-is-the-permanent-graphics-api.md:10`, *Consequences*: "The exit is an owned fork of wgpu's crates, built from `vendor/`, where on 2026-11-14 a copy of wgpu-core 30.0.1 carries one upstream fix".

`vendor/README.md:13`: "## wgpu-core 30.0.2"; `Cargo.lock`: wgpu-core 30.0.2. `git log -- vendor/README.md`: `94273c1f 2026-11-15 Move wgpu to 30.0.2 and re-base the vendored wgpu-core`, whose body lists what it renamed ("`vendor/README.md`, `CLAUDE.md`'s Stack clause and the tree in `docs/agents/project-shape.md` name 30.0.2; the registry entry [...]"), not ADR 0007. `vendor/README.md:8-11`: "the wgpu 31 bump [...] rewrites what names the copy: CLAUDE.md's Stack, the tree in `docs/agents/project-shape.md` and ADR 0020's static TLAS clause" — ADR 0007, which also names the copy, is missing. The clause is dated, so it was true when written; it is stale at the frozen commit.

### L7-B05 (killed)

`docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:18`: "each TLAS with wgpu-core's buffer of its Instances, 72 bytes a slot on Metal and 64 elsewhere [ref:wgpu-30-tlas-instance-buffer] [ref:wgpu-hal-30-metal-instance-bytes] [ref:wgpu-hal-30-instance-bytes]".

The three quotes: `raw_tlas_instance_size .checked_mul(desc.max_instances.max(1))` (no size); `fn tlas_instance_to_bytes(&self, instance: TlasInstance) -> Vec<u8> { let temp = MTLIndirectAccelerationStructureInstanceDescriptor {` (the type, no size); `raw_tlas_instance_size: 64,` (Vulkan). Opened wgpu-hal 30.0.1 metal/adapter.rs L1503-1506: `raw_tlas_instance_size: u32::try_from(size_of::<MTLIndirectAccelerationStructureInstanceDescriptor>())`, so the 72 comes from Apple's struct. The registry's `["metal-indirect-instance-descriptor"]` (`docs/references.toml:427-431`) carries it ("a 48-byte MTLPackedFloat4x3, four UInt32 and an 8-byte MTLResourceID, 72 bytes"), and the note cites it for the same figure (`docs/research/acceleration-structures-for-streamed-worlds.md:50`: "72 bytes on Metal [ref:metal-indirect-instance-descriptor]"). The sentence predates M5; line 18 is in M5's diff for the Arena shares, which is why it is checked here.

### L7-B06 (killed)

`docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:19`, added 2026-11-14: "a load past the count waits for a later frame, nearest the camera first, and a Settled wait loads past it as it does past the bytes"; the same amendment, added 2026-11-15: "a Settled frame and a spawn's loads still wait for theirs". `CONTEXT.md:238`: "**Settled frame**: A frame rendered only after every load it requested is Resident and placed [...]"; no **Settled wait** entry. The line carried one "Settled wait" before M5 (`git show 6ebb45fb:docs/adr/0017-...md`, "A capture's Settled wait loads past the cap"); M5 added the second.

### L7-M01 (killed)

- Doc: `docs/research/lod-hlod-and-far-field.md:45`, heading *Transitions without TAA*, row **4. Dithered cross-fade per pixel.** It reads: "Both levels draw during the fade, weighted 1 to 0 and 0 to 1, usually through screen-space dithering [ref:unity-6000.0-lodgroup-transition-height]".
- Source: registry key `unity-6000.0-lodgroup-transition-height` (`docs/references.toml:6022`). The url `https://docs.unity.com/en-us/engine/6000.0/manual/analysis/graphics-performance-profiling/simplifying-distant-meshes-with-level-of-detail-lod/class-lodgroup` was fetched on 2026-11-21. Under *Transitioning between LOD levels* it says: "Cross-fading is the technique of rendering two levels at the same time, with a weighting of 1 to 0 for the current LOD level and 0 to 1 for the next LOD level [...] Unity usually implements the cross-fading by using either screen-space dithering or transparency." The registry quote has the same words.
- The note moves "usually" from the either/or onto dithering alone, and states it as a general fact rather than Unity's practice. The source does not rank dithering above transparency.
- The fix is a one-clause rewrite. The registry entry stays as it is: its quote is verbatim and correctly placed.

### L7-T03 (killed)

The source is `dynamic.pdf`, p. 2, Section 2 "Previous Work". It reads: "Recently, Lext and Akenine-Moeller [2001] proposed a way for quickly reconstructing an acceleration structure in a hierarchically animated scene. Though their method was developed independently, it is closely related to the way that our method handles hierarchical animation." Figure 3 captions it "Two-level hierarchy as used in our approach". The registry quote supports only the top-level rebuild cost.

### L7-T04 (killed)

`gh api repos/gfx-rs/wgpu/pulls/10187`. The body reads: "What I can't do is build the TLAS over those cluster BLASes. […] Acceleration structures are the one gap. This adds that accessor." Under **Testing** it reads: "Still working on the full implementation on top of this."


## Rule candidates

Each candidate is a slip a reader saw twice, with the ids of the Findings
of its kind; the verifier marked each `stands` (two or more surviving
Findings of its kind) or `falls`. The milestone's rules ticket decides.

| rule | finding ids | status |
|---|---|---|
| A change that makes false what a research note says Praxis does, in the present tense, rewrites or dates that sentence: the checklist's grep for a changed name extends to `docs/research/`. | L7-S01, L7-S02, L7-S03, L7-S04, L7-S05, L7-S06, L7-S07; of the same kind L7-F02, L7-L05, L7-T05 | stands |
| A research note's clause that a later measure answers ("unmeasured", a *Not verified* line) is rewritten or dated by the change that lands the measure. | L7-L07, L7-L09, L7-L10 | stands |
| A registry quote of a table gives its rows as "cell: cell" and adds no bracket but `[...]`; the registry's header states it, and `cargo test --test references` checks neither half. | L7-L02, L7-L03 | stands |
| A statement of which wgpu release carries an upstream change, or which version the vendored copy is, follows the registry's section and is listed in `vendor/README.md`'s bump list. | L7-B02, L7-B04 | falls: L7-B02 is killed; reworded as "a doc that says which wgpu Praxis runs, the vendored copy included, is rewritten by the change that moves it", it could stand on L7-B04 and L7-T05 |
| A sentence that cites a source keeps the source's hedge. | L7-T02, L7-T04 | falls: L7-T04 is killed, leaving L7-T02 alone of its kind |

The first two are forms of one slip, a research note's sentence about
Praxis not moved by the change that made it false; the rules ticket may
fold them into one, which would rest on thirteen rows.

## Left for a full lane 7

Defects the readers or the verifier saw outside this audit's scope, in
clauses M5 did not change or files it did not read. None is a Finding;
each is a lead for the next full lane 7, with who saw it.

- ADR 0003:3 gives "517,376 slots" and "136.2 bytes a slot", and ADR
  0017:18 the same 136.2, both made false by the Instance slots' move to
  488,448 (`66390f70`), in clauses M5 left unchanged (the verifier).
- ADR 0017:18 gives a TLAS's Instance buffer "72 bytes a slot on Metal"
  citing three keys, none of whose quotes states 72 (L7-B05, killed on
  scope; the verifier confirmed the facts).
- `docs/performance.md:2154` cites `amd-rra-instance-overlap` for a merge
  slowing traversal, where AMD's page warns of overlap from splitting a
  BLAS (reader S; not verified).
- ADR 0016 says `par_iter` runs on the global compute pool, which the
  engine no longer creates (reader B, who read the clause as a deferred
  decision; not verified).

## Killed

The Findings the verifier killed, 6 of 32, each with its reason; their
evidence blocks are above, marked killed.

| id | claim | reason |
|---|---|---|
| L7-B02 | ADR 0007's exit names the vendored copy as wgpu-core 30.0.1, while since 2026-11-15 it is the 30.0.2 crate, and `vendor/README.md`'s list of what a wgpu bump rewrites omits ADR 0007. | ADR 0007:10 says "on 2026-11-14 a copy of wgpu-core 30.0.1". `git show 94273c1f^:vendor/README.md` has "## wgpu-core 30.0.1", and the move to 30.0.2 landed 2026-11-15 10:39. A dated clause true at its date is the remedy `CLAUDE.md`'s checklist prescribes ("rewritten or dated"), not a defect. `vendor/README.md` lies outside the narrowed scope, and a wgpu bump makes no dated clause false. |
| L7-B05 | ADR 0017 gives a TLAS's Instance buffer "72 bytes a slot on Metal" citing three keys, none of which states 72; the source of the figure, `metal-indirect-instance-descriptor`, is not cited there. | The brief rules it out of scope: "the unchanged clauses of amended ADRs". `git show 6ebb45fb:` ADR 0017 already holds "72 bytes a slot on Metal and 64 elsewhere [ref:wgpu-30-tlas-instance-buffer] [ref:wgpu-hal-30-metal-instance-bytes] [ref:wgpu-hal-30-instance-bytes]" unchanged. The facts do hold: none of the three quotes, `docs/references.toml:4901`, `:4967` and `:4961`, states 72. It is worth refiling in a full lane 7. |
| L7-B06 | ADR 0017's amendment uses "a Settled wait", which is not a `CONTEXT.md` term, beside "a Settled frame" in the same amendment. | ADR 0017:19's "a Settled wait" applies the glossary's adjective (`CONTEXT.md:238`, **Settled frame**) to the wait for one. Pre-M5 text already had "A capture's Settled wait", and the code says "report Settled" (`src/bench/mod.rs:133`). The phrase is in no `_Avoid_` line and says nothing the code does not, so it is not drift. |
| L7-M01 | The LOD note attributes "usually" to screen-space dithering alone, but Unity's manual says Unity usually cross-fades by either screen-space dithering or transparency. | Same sentence and same defect as L7-L06 (`lod-hlod-and-far-field.md:45`). L7-L06 survives, and M01's expected rewrite is carried into it. |
| L7-T03 | Note's claim "The two-level structure is Wald, Benthin and Slusallek's" credits the structure to them alone; neither the registry quote nor the paper supports that. | The attribution is supported by the survey the same sentence cites. Meister p. 11 (`pdftotext`) reads "Wald et al. [WBS03] proposed a two-level hierarchy". Wald's mention of Lext and Akenine-Möller is a method "developed independently" and "closely related", not the same structure. |
| L7-T04 | Note's claim that PR 10187 lets "a TLAS wgpu binds … be built by hand over cluster BLASes" states as done what the PR's author is still implementing. | The PR body (`gh api repos/gfx-rs/wgpu/pulls/10187`) says what the note says: "What I can't do is build the TLAS over those cluster BLASes. [...] Acceleration structures are the one gap. This adds that accessor", with `mark_acceleration_structures_built` covering the bind. "Still working on the full implementation" is the author's testing status, not a hedge on what the accessor enables. |
