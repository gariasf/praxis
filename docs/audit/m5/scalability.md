# Scalability check: M4's capacities and static TLAS under M5's far field

Lane 4 of `docs/agents/audit.md` (scalability and reality check), run against
M5's scope only: LOD chains, an HLOD or far proxy per Chunk out to the 2 km
view, far proxies in the TLAS, and millions of Resident Instances. Every
`path:line` is at `main` `d4ac21eb8ca9df5927cb74c50504f57478298efb`. One
reader prepared it from the tree, the Baselines under `bench/baselines/`,
`docs/performance.md` and the M4 exit's recorded figures; it ran no bench, no
capture and nothing on the GPU, and no adversarial verifier has reopened it
yet. Every number is read from a file or derived in the cell that states it.
It charts nothing; the decision tickets on M5's map use it. The flat cull's
figures (S09, the ADR 0019 row that repeats it, must-change item 5 and
not-verified item 6) were corrected on 2026-11-14 from the stage split in
`docs/performance.md`, the culling pass, each stage on its own. The
heightfield rows (H01 to H08) were added on 2026-11-14 for the strategy
review's Finding S1-01 (`docs/audit/strategy/2026-11.md`), their `path:line`
at `main` `99b4141be6459ea12978feb18cba0581adb777f9`. One adversarial
verifier reopened those rows, and what it found is in them.

## Summary

With an HLOD proxy per Chunk past the Load radius (survey 3's F1), M4's
capacities hold M5's far field: the Instance slots keep a 1.28 times margin,
the static TLAS a 1.04 times one, and criterion 7's N = 10 survives with
nothing to spare, as M4's arrival build of 18.7 to 19.6 ms already sits at its
19.6 ms limit; without HLOD, the 2.23 to 2.30 M static Instances within 2 km at
the world's density need 4.3 to 4.5 times the slots and 8.4 to 8.7 times the
static TLAS, and a build of 98 to 170 ms puts N at 42 to 72 frames. The Fast
travel's 50 ms hold survives if it still waits for the Load radius alone, but
its every-frame clause does not: `FrameIo::fits` caps a frame's placed
Instances and written bytes, which one-Placement proxies barely touch, so 10
to 14 proxy loads at a Chunk load's measured cost take the arrival frame's 8.5
to 9.0 ms of CPU past 16,667 µs. Ranked by what they block, the must-change
list is a per-frame cap on load count or main-thread load time (the arrival),
far geometry beyond one proxy per Chunk kept out of the static TLAS (N = 10),
the Material table's 222 entries (F1's proxies), and, only if HLOD is not
built, the Instance slots and the flat per-slot cull.

A heightfield, sized for the milestone that brings terrain (rows H01 to H08:
a regular grid a Chunk at 1 m posts, 131,072 triangles and one BLAS each,
every time derived and none measured), changes no M5 criterion and leaves
M5's shapes standing: one Far proxy per Chunk, LOD 0 in every BLAS, the
static TLAS's entries, three fewer a Chunk, and N = 10. Five things M5 fixes
must change when terrain is charted. "Nothing drawn past 2 km but flat
ground": relief 100 m high stands 87 pixels tall at 2 km. The shadows of that
relief, which the static TLAS does not hold. The load frame: the main thread
waits for each new BLAS's build, about 1.8 to 2.3 ms a Chunk by a one-off
probe's rate, so the four Chunks of a Fast travel's first frame take it to
16.3 to 18.7 ms. The Far proxy's 5,832-triangle cap and its 8-triangle
ground. And the memory: the 24 heightfields in use need the geometry Arenas'
shares re-cut and 1.85 times their Budget with chains and proxies counted,
and 1.5 times what the acceleration structures' Budget has free. At 2 m posts
the load frame and both Budgets hold, and the Arenas' shares still change.

## Scale assumptions

- **Target**, ADR 0013:3: "100k visible Instances, about 5M visible triangles,
  a 10 by 10 km streamed Game world, a 2 km LOD'd view distance, and GPU
  memory at most 8 GB at 1440p"; no CPU or GPU frame past 16,667 µs.
- **Chunks within the 2 km view.** A Chunk is within a radius when its nearest
  point is (`src/streaming/mod.rs:287-295`). From a camera anywhere on a 64 m
  grid, 220 to 227 Chunks lie within 2,000 m (14.42 to 14.88 km²), 8 to 12
  within the 256 m Load radius and 20 to 23 within the 512 m hold radius
  (`docs/guides/chunk-streamer.html:188`; this lane's count gives 20 to 24,
  counting a Chunk exactly at 512 m). Chunks past the Load radius within
  2,000 m: 220 − 12 = 208 to 227 − 8 = 219.
- **Assumption W (the world's density).** `world` places 16,214,556 static
  Instances over 40 × 40 Chunks, 10.24 km a side
  (`docs/guides/reference-scenes.html:783`; `src/scenes/world.rs:23`):
  16,214,556 / 104.8576 km² = 154,634 per km², and 16,214,556 / 64 = 253,352
  per region of one settlement. Over the 2 km disc (12.566 km²), 1,943,189;
  over the Chunk set, 2,229,514 to 2,300,491. Inside a settlement, at most
  253,387 Instances within the Load radius (same guide line) over its
  754 m square (13 blocks of 4 × 12 m lots plus a 10 m street,
  `src/scenes/world.rs:30-35`), 0.5685 km²: about 445,699 per km².
- **Assumption D (town's density)**, M3.5's: 254,256 per km²; 3,195,075 over
  the disc, 3.67 to 3.78 M over the Chunk set.
- **Without HLOD**: every static Instance in the 2 km Chunk set is Resident,
  drawn and traced, with the 10,000 Moving Instances; held Chunks play no part.
- **With HLOD (F1)**: M4's near field unchanged (the Load radius drawn and
  traced, the hold radius held), plus one merged proxy per Chunk past the Load
  radius out to 2,000 m, at most 219, each one Instance, one Mesh and its
  Materials.
- **Assumption G** (M3.5's): unique geometry in units of the cooked helmet,
  14,127 vertices, 23,358 triangles, 278 Meshlets, 844,804 Arena bytes, so
  36.17 bytes per drawn and traced triangle. Survey 3's G1 halves each LOD
  level, so a chain is at most 2 times its first level's triangles (M3.5's 4/3
  assumed quarter levels).
- **Static TLAS build.** The worker's build is fill, record and `finish`
  (`src/render/acceleration_structures.rs:530-558`). Survey 4's probe
  (`docs/research/acceleration-structures-for-streamed-worlds.md:68-78`) puts
  recording at 3.44 to 5.67 ms at 262,144 entries empty or full, 13.1 to 21.6
  ns per entry of capacity, and fill plus `finish` at 3.3 + 27.2 to 27.8 ns per
  filled entry: 44.2 to 52.2 ns per entry of a full TLAS. The M4 exit's
  arrival build, 18.7 to 19.6 ms (`docs/performance.md:262`), over 264,720
  entries is 70.6 to 74.0 ns. Rows use 44 to 74 ns per entry, scaled linearly
  past 264,720, an assumption (Not verified, 1).
- **Frames until traced.** One build runs at a time
  (`src/render/acceleration_structures.rs:521`), so a change waits for the
  build in flight, then its own, and is traced from the frame after its
  build's submit. Fitted to the M4 exit: at `world`'s Baseline GPU frame
  median, 4,891.3 µs (`bench/baselines/apple-m4-pro/world.json`), N ≈ 2 +
  wait / 4.891 ms; the arrival's wait of 18.7 + 17.3 = 36.0 to 19.6 + 19.3 =
  38.9 ms gives 9.4 to 10.0, against the 9 to 10 measured
  (`docs/performance.md:256-268`). N = 10 allows a wait of 8 × 4.891 = 39.1 ms,
  two builds of 19.6 ms: 264,396 entries at 74 ns, 444,667 at 44 ns.

## Capacities, floors and sizes

Each M4 capacity is its Budget share at its bytes per entry
(`bench/budgets.json:9-17`), pinned by
`src/render/renderer.rs:1034-1044` and `src/assets/pools.rs:1200-1213`.
Rows S01 to S07 are those M4 changed from M3.5's R01 to R07.

| id | constant | where (`path:line`) | value at `d4ac21eb` | target at M5's scale | arithmetic | verdict |
|---|---|---|---|---|---|---|
| S01 | Instance slots, `RenderCapacities::instances` | `src/render/renderer.rs:178`, `:226-251`; pinned `:1039` | 30% of 234,881,024 = 70,464,307 bytes, less 8, in 34,856-byte scan blocks of 256 slots (136.16 bytes a slot, `:189-192`): 2,021 blocks, 517,376 slots | Without HLOD: 2.23 to 2.30 M static + 10,000 Moving (W); 3.20 to 3.78 M (D). With HLOD: the hold radius's 394,032 + 219 proxies + 10,000 | Without: 2,239,514 / 517,376 = 4.33 to 2,310,491 / 517,376 = 4.47 times; 304.9 to 314.6 MB of slots, a 30% share of a 1.02 to 1.05 GB Budget; D: 6.2 to 7.3 times. With: 404,251 / 517,376 = 0.78, a 1.28 times margin, 113,125 slots free | survives with HLOD (1.28x); must change without it (4.3 to 4.5x at W) |
| S02 | Static TLAS entries, `static_tlas_instances` | `src/render/renderer.rs:185`, `:245-248`; pinned `:1041`; created `src/render/acceleration_structures.rs:329`; the fit test `src/scenes/world.rs:731-795` | 57% of 268,435,456 = 153,008,209 bytes over 2 × 289 (two TLASes, `:197`): 264,720 entries | Without HLOD: 2.23 to 2.30 M (W), 3.20 to 3.78 M (D). With HLOD: 253,387 near-field + at most 219 proxies | Without: 8.42 to 8.69 times; the two TLASes 1.29 to 1.33 GB against the 268,435,456-byte Budget. With: 253,606 / 264,720, a 1.044 times margin, 11,114 entries free after the proxies (11,333 before) | survives with one proxy per Chunk; must change without HLOD, or with far geometry past 11,114 entries |
| S03 | Static TLAS build on its worker, one at a time | `src/render/acceleration_structures.rs:511-563` (`:521` waits for an idle spare); M4 exit `docs/performance.md:256-268`, `:843-845` | 18.7 to 19.6 ms at the arrival, 5.2 to 18.6 ms for the segment's other builds | N = 10 needs a build within 19.6 ms (Scale assumptions) | With HLOD: + 219 × 44 to 74 ns = 0.010 to 0.016 ms, under 0.1% and inside the arrival's 0.9 ms spread. Without: 2.23 M × 44 ns = 98 ms to 2.30 M × 74 ns = 170 ms (W), 141 to 280 ms (D) | survives with HLOD at zero margin; must change without it |
| S04 | `REANCHOR_CELLS`, re-anchor builds | `src/render/acceleration_structures.rs:195` | 4 cells, 256 m | 2 km view; the camera's 50 m/s (`src/scenes/world.rs:46`) | A re-anchor build every 256 / 50 = 5.12 s of flight without a static change, whatever the far field holds; it rebuilds every entry, so it costs what S03 costs | survives |
| S05 | `DYNAMIC_TLAS_CAPACITY` | `src/render/acceleration_structures.rs:17` | 10,000 | 10,000; M5 adds no Moving Instances | `world` Baseline `dynamic_tlas` CPU median 672.9 µs against the 1,100 µs Budget (`bench/budgets.json:5`) | survives |
| S06 | Acceleration structures Budget, `TLAS_BYTES_PER_INSTANCE` | `bench/budgets.json:14`; `src/render/renderer.rs:197` | 268,435,456 bytes; 289 per entry; `world` peak 155,577,592 (Baseline) | Without HLOD: two static TLASes at S02's count | 2.23 to 2.30 M × 578 = 1,288,940,000 to 1,329,400,000 bytes, 4.8 to 5.0 times the Budget. Every Budget sums to 5,955,911,680 (ADR 0013:3), 2,044,088,320 under 8 GB; S01 and S06 together add 1.37 to 1.42 GB, which fits: memory is not what binds | survives with HLOD; must change without it (a Budget raised by ADR or map comment) |
| S07 | `DrawCapacity::FIXED.packed_indices` | `src/render/culling_pass.rs:70-73` | 2^25 = 33,554,432 indices, 128 MiB, the floor's one binding | Visible triangles × 3 | The frustum's 72.7° of 360° (45° vertical at 16:9) holds 20.2% of the disc. Without HLOD or LOD: 450,451 to 464,791 Instances in view (4.5 to 4.6 times ADR 0013's 100k); at `world`'s 12.33 triangles an Instance (135,150 / 10,961, Baseline) 16.7 to 17.2 M indices, 50 to 51%; at `town`'s 40.30 (5,552,306 / 137,776) 54.5 to 56.2 M, 1.62 to 1.67 times. With HLOD: 219 × 20.2% × P triangles a proxy, 1.3 M indices at P = 10,000, 4% | survives with HLOD; without, decided by content class: survives `world`'s boxes, must change at `town`'s |
| S08 | `DrawCapacity::FIXED.meshlet_records` | `src/render/culling_pass.rs:70-73` | 2^20 = 1,048,576 | Visible Meshlets | 5,000,000 / 10.7 triangles a Meshlet = 467,290, 44.6%; S07's 5.55 to 5.73 M `world`-class triangles 49.5 to 51.1% | survives |
| S09 | Flat per-slot cull, `SCAN_BLOCK_SIZE`, `MAX_WORKGROUPS_PER_DIMENSION` | `src/render/culling_pass.rs:1-13`, `:47`, `:53` | 256 slots a block; every slot in use is tested; `town`'s per-slot stages take 200.6 µs in phase one and 171.7 µs in phase two at 254,256 slots, 1.464 ns a slot for both (`docs/performance.md`, each stage on its own); the Baseline's 1,136.7 µs for phase one, 4.5 ns a slot, includes the triangle appends | Without HLOD: S01's 2.24 to 2.31 M slots in use. With HLOD: M4's plus 219 | Without: 2,239,514 × 1.464 ns = 3.3 ms to 2,310,491 × 1.464 = 3.4 ms of per-slot stages against the 12 ms GPU frame, plus appends that follow the triangles drawn (`world`'s phase one reads 194.2 µs at a slot count no Baseline records). Workgroups: 2.31 M / 256 = 9,026 per dimension, under 65,535 | survives with HLOD; must change without it (ADR 0019's per-Instance culling) |
| S10 | Light buffer, `RenderCapacities::lights` | `src/render/renderer.rs:181`; pinned `:1040`; cut farthest first `src/render/prepare.rs:1017-1029`; drawn Chunks' lights only `:989-993` | 0.1% of 234,881,024 / 48 = 4,893 | `world` cooks no point lights (`src/scenes/world.rs:461-463`). At `town`'s 1,024 lamps per km², the Load radius holds 537 to 805; the 2 km disc 12,868, the Chunk set 14,764 to 15,234 | 12,868 / 4,893 = 2.63 times, if lights past the Load radius are listed; within it, 6.1 to 9.1 times margin | survives while far Chunks contribute no lights; must change if they do |
| S11 | Froxel grid's far slice, `FROXEL_LIGHT_CAPACITY` | `src/render/froxel_pass.rs:19-28` | 32 slices, the last from 1,024 m to infinity; 127 lights a Froxel | Far lights listed at `town`'s density | M3.5 R11 and R12: a far-slice Froxel covers 68,000 m², 107,000 m² with an 18 m light range each side: 70 to 110 lamps, 1.15 times margin. Everything from 1,024 m to the 2,000 m view edge shares one slice | survives; with far lights listed, margin 1.15x |
| S12 | `SHADOW_MASK_LIGHTS` | `src/render/shadow_mask_pass.rs:35` | 59 bits, 58 for point lights after one sun | S11's far lists | 70 to 110 listed past 58 bits: far point lights past the 58th cast no shadow | survives while no far lights are listed; must change if a far point light must shadow |
| S13 | Geometry Arenas, `ARENA_PER_MILLE` | `src/assets/pools.rs:16`, `:54-80`; pinned `:1204-1208` | 83,886,080 bytes as 40,852,520 / 26,591,884 / 503,316 / 7,130,316 / 8,808,036 | Unique Resident geometry: LOD chains and proxies. `world` uses 14,704 Arena bytes (Baseline `arena_bytes_in_use`) | The Meshlet triangle Arena binds first: 2,202,009 triangles, 94.3 helmet-class Meshes, 47.1 with G1's doubled chains (index Arena 2,215,990, vertex 2,412,384, Meshlet vertex 2,343,397, Meshlet 2,643,088 by the helmet's ratios). 219 proxies of P triangles fill it at P = 2,202,009 / 219 = 10,055 with nothing else resident | survives `world`'s content and F1 under 10,055 triangles a proxy; must change under Assumption G with chains past 47 helmet-class Meshes |
| S14 | Vertex Arena under the floor's 128 MiB binding | `src/gpu/mod.rs:310-322`, test `:821-825` | 40,852,520 of 134,217,728 bytes, 0.304 | G2 (unique vertices per level) doubles vertex bytes at the same Budget | The binding is reached only if the geometry Budget grows 134,217,728 / 40,852,520 = 3.29 times | survives |
| S15 | Mesh table, `PoolCapacities::meshes` | `src/assets/pools.rs:19`, `:73`; pinned `:1209` | 83,886,080 / 4,488 = 18,691 | Unique Resident Meshes: the near field's, 219 proxy Meshes, LOD levels if each is its own entry | `world`'s library is a few dozen Meshes; 18,691 / (219 + 4 levels × 100 helmet-class) = 30 times margin | survives |
| S16 | Material table, `PoolCapacities::materials` | `src/assets/pools.rs:24`, `:74`; pinned `:1210` | 4,294,967,296 / 19,267,584 (helmet's Textures past their tails) = 222 | F1's proxies: their settlements' Materials, or one baked Material a proxy | `world` clads each settlement in 24 facade Materials (`src/scenes/world.rs:72`), 3 named ones beside them (`:265`). From a settlement's centre, where the Fast travel lands (`src/scenes/world.rs:183-187`), every settlement of the 3 × 3 regions about it lies within 2,000 m: with strays of up to 640 − 377 − 32 = 231 m (`:38`), a diagonal one's nearest point is at most hypot(1,742 − 377, 1,742 − 377) = 1,930 m away. Each of 2,704 lots picks one of 24 facades, so 9 × 24 + 3 = 219 of 222, and farms clad in further regions' facades (`:555-576`) pass it. Baked: 208 to 219 proxy Materials plus the near field's. A Chunk whose Materials do not fit is refused (five such on M4's first traversal, #672) | must change for F1: the count is set by bytes a full Material holds, and a proxy at its Resident tails holds far fewer |
| S17 | `INSTANCES_PLACED_PER_FRAME` | `src/streaming/io.rs:481`; applied `:891-904` | 131,072 | Proxy loads: one Placement each | 131,072 proxies a frame pass the cap: it bounds nothing proxies do | survives as a number; the cap is S18's must change |
| S18 | `FrameIo::fits`, the per-frame load bytes | `src/streaming/io.rs:891-904`; cap pinned `src/render/renderer.rs:1042` | 22,937,393 bytes a frame; a first load passes either cap | Proxy loads on and after the arrival frame | A proxy of P = 10,000 triangles writes 361,677 Arena bytes: 63 a frame under the bytes cap. At a Chunk's measured sizing plus load, 0.26 to 0.32 + 0.29 to 0.46 ms (`docs/performance.md:61-68`), 63 proxies cost 35 to 49 ms; the arrival frame's 16,667 − 9,000 µs (`:104`) admits 7,667 / 780 = 9.8 to 7,667 / 550 = 13.9 | must change: no cap bounds a frame's load count or main-thread time |
| S19 | Load and hold radii | `src/scenes/mod.rs:514-530`; `src/streaming/mod.rs:270-272`; held Chunks neither drawn nor traced `:59-61`, `src/render/prepare.rs:899`, `:991` | 256 m, 512 m | 2,000 m view, 7.8 times the Load radius | Proxies cover what M4 holds undrawn: 8 to 12 Chunks drawn of 220 to 227 within 2 km | survives as the near field's radius |
| S20 | GPU memory, the seven Budgets | `bench/budgets.json:8-17`; ADR 0013:3 | 5,955,911,680 bytes; M4's traversal peak 1,003,873,440 (`docs/performance.md`, M4 exit rerun) | Without HLOD: S01 + S06's 1.37 to 1.42 GB more | Fits 8,000,000,000 with 0.62 to 0.67 GB to spare | survives |

## ADR clauses M5 touches

| ADR | clause | verdict | arithmetic | where |
|---|---|---|---|---|
| 0003 | "the buffer holds a fixed number of slots ... sized at device creation from the scene and pass buffers' GPU memory Budget ... 517,376 slots" | survives with HLOD; must change without | S01: 1.28x with HLOD; 4.3 to 4.5x short without | `docs/adr/0003-persistent-scene-transform-buffer.md:3` |
| 0003 | "each loaded Chunk owns a contiguous range of slots ... A held Chunk keeps its range, written Instances with no Mesh until it is drawn again" | survives | A proxy is a range of one slot; a held Chunk shows its proxy, not its range | same line |
| 0013 | "a 2 km LOD'd view distance" | survives (M5 builds it) | With HLOD every M4 capacity but the Material table holds (S01 to S20) | `docs/adr/0013-reference-hardware-and-provisional-budgets.md:3` |
| 0013 | "100k visible Instances" | survives with HLOD; must change without | S07: 450,451 to 464,791 in the frustum without HLOD or LOD, before occlusion | same line |
| 0013 | "no CPU or GPU frame lasts longer than 16,667 µs" | must change in the streaming code, not in the clause | S18: about 10 to 14 Chunk-cost proxy loads fill the arrival frame | same line |
| 0013 | the seven Budgets, "5,955,911,680 in all" | survives with HLOD; must change without | S06: 4.8 to 5.0 times the acceleration structures' Budget; total within 8 GB (S20) | same line |
| 0017 | "One vertex arena ... the Capability floor's 128 MiB `max_storage_buffer_binding_size` caps that arena" | survives | S14: 0.304 of the binding | `docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:17` |
| 0017 | "the Material table an entry per 19,267,584 bytes of the texture pool's" | must change for F1 | S16: 222 entries against 219 at 9 regions' facades, more past them | `docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:18` |
| 0017 | "the static TLAS's two TLASes 57% of the acceleration structures' ... the Instance slots 30% of the scene and pass buffers' ... the lights 0.1%" | survives with HLOD; must change without | S01, S02, S06, S10 | same line |
| 0017 | "A frame's Chunk loads also place at most 131,072 Instances, since prepare and the static TLAS take each on the main thread ... a Chunk that alone writes past the cap loads with no other writes in its frame" | must change | S17, S18: neither cap bounds a frame of one-Placement loads | `docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:19` |
| 0019 | "Dense static meshes use discrete LOD" | survives | M5 builds it; survey 3's G and S options | `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:29` |
| 0019 | "Culling tests Instances" | survives with HLOD; must change without | S09: 3.3 to 3.4 ms of per-slot stages at 1.464 ns a slot over 2.24 to 2.31 M slots, plus appends that follow the triangles drawn | `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:35` |
| 0019 | the light buffer, "4,893 lights ... past it the point lights farthest from the camera ... are cut" | survives while only drawn Chunks contribute lights | S10: 2.63x short at `town`'s lamps over 2 km | `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:38` |
| 0019 | far-slice Froxel lists (the last slice to infinity) | survives while no far lights are listed; margin 1.15x if they are | S11, S12 | `src/render/froxel_pass.rs:19-28` |
| 0020 | "The TLAS holds the near field plus merged far proxies, so a 100k-Instance rebuild (6 to 8 ms ...) stays out of the frame; its radius is decided with Chunk streaming (M4)" | survives with one merged proxy per Chunk; must change for per-Instance far entries | S02, S03: 11,114 entries free; the near field is 253,387, not 100k, and its build 18.7 to 19.6 ms | `docs/adr/0020-ray-traced-probe-gi-and-hard-ray-traced-shadows.md:20` |
| 0020 | "rebuilt only when its static Instances change (a Chunk streams in or out, or a static Renderable spawns or despawns) or the anchor moves" | survives; under survey 4's L1 it rebuilds without pause | An HLOD swap falls on its Chunk's own load or unload frame, one batch. Under L1 a discrete LOD change is a BLAS change: at 50 m/s a band edge at 256 m sweeps 2 × 256 × 50 / 60 = 427 m² a frame, about 190 settlement Instances, so a change waits every frame | same line |
| 0020 | "a build lands a frame or more after its change" | survives | S03 and criterion 7 below | same line |

## M4 choices M5 must undo

1. **Held Chunks draw nothing, and nothing past the hold radius is
   Resident.** `src/streaming/mod.rs:59-61`; `src/render/prepare.rs:899`,
   `:991`. Clause: ADR 0013:3's 2 km view. Undone by M5's far field; under F1
   a held or unloaded Chunk past the Load radius shows its proxy.
2. **The static TLAS is sized to the Load radius's peak.** 264,720 entries
   against 253,387 (`src/scenes/world.rs:731-795`), 11,333 free. Clause:
   ADR 0020:20, "near field plus merged far proxies". Proxies fit if they are
   one per Chunk; any other far geometry needs a TLAS of its own (survey 4's
   far-field F1), not this one's capacity, since recording grows with capacity
   filled or not (survey 4, row 1).
3. **The per-frame load caps count Instances and bytes, not loads.**
   `src/streaming/io.rs:481`, `:891-904`. Clause: ADR 0017:19. Undone by a
   cap that bounds a frame's main-thread load time or load count (S18).
4. **The Material table's count is a full Material's texture bytes.**
   `src/assets/pools.rs:24`, `:74`. Clause: ADR 0017:18. Undone by counting
   proxy Materials apart, or by proxies that share Materials (S16).
5. **A Mesh is one Meshlet range and one BLAS.** `src/assets/mesh.rs:34-42`;
   no `LOD` in `src/`. Clause: ADR 0019:29. Undone by M5's chains; which level
   the BLAS holds (survey 4's L1 to L3) decides whether LOD changes rebuild
   the static TLAS (ADR 0020 row above).
6. **Criterion 7's N is counted in bench frames.** Its 10 came from two
   builds waiting at `world`'s 4.89 ms GPU frame; the same wait at a 12 ms
   frame reads 2 + 39.1 / 12 = 5.3 frames, and at 16,667 µs 4.3. A far field
   that lengthens the frame lowers N without any build changing; the
   criterion reads the same across scenes only in milliseconds.

## Criterion 7, N = 10

**With HLOD (F1): survives, at the margin M4 measured.** The arrival's
changes wait for two builds, 36.0 to 38.9 ms, and N = 10 allows 39.1 ms at
`world`'s frame (Scale assumptions). One merged proxy per Chunk adds 219
entries at most, 0.010 to 0.016 ms a build, inside the 0.9 ms spread between
runs. An HLOD swap lands in its Chunk's load or unload batch. Under survey 4's
L1 the worker builds without pause, but a change still waits at most for the
build in flight and its own, so ordinary loads rise toward N = 10 instead of
7, while the arrival stays at 9 to 10. N passes 10 when a build passes 19.6 ms:
past 264,396 entries at 74 ns, or 444,667 at 44 ns.

**Without HLOD: fails.** 2.23 to 2.30 M entries build in 98 to 170 ms, so the
arrival waits 196 to 340 ms: N = 2 + 2 × 98 / 4.891 = 42 to 2 + 2 × 170 /
4.891 = 72 frames at today's frame, 14 to 22 at 16,667 µs. At town's density
(3.20 to 3.78 M), 141 to 280 ms a build.

Not settled without a bench: the build time past 264,720 entries (Not
verified, 1), and the far field's effect on the frame period (item 6 above).

## The Fast travel's arrival

**The hold, within 50 ms: survives if it still waits for the Load radius
alone.** The Fast travel's hold ends when every Chunk within the Load radius
has loaded (`CONTEXT.md:173-175`; `src/streaming/mod.rs:247`), measured at
23.9 to 26.4 ms over 4 to 5 frames (`docs/performance.md:85-90`). A hold that
also waited for 208 to 219 proxies would not: at 9.8 to 13.9 proxy loads a
frame at a Chunk's cost (S18) they take 15 to 23 frames after the near field,
73 to 113 ms at `world`'s 4.89 ms frame. The far field then arrives after the
hold, nearest first, as `FrameIo::next_ready` orders loads
(`src/streaming/io.rs:872-883`).

**Every frame within 16,667 µs: must change.** The arrival frame reads 8.5 to
9.0 ms of CPU after the read-wait fix (`docs/performance.md:101-105`). Its
loads are bounded by 131,072 Instances and 22,937,393 bytes; a proxy is one
Placement and about 362 kB at 10,000 triangles, so 63 fit the bytes cap and
the Instances cap never binds. At a Chunk's measured 0.55 to 0.78 ms of
sizing and load, 10 to 14 proxies fill the frame's 7,667 µs of headroom.
Breakpoint: proxy loads a frame × main-thread cost of one ≤ 16,667 µs less
the arrival frame's own; what a one-Placement load costs is Not verified, 3.
The arrival's static TLAS build gains 0.010 to 0.016 ms (S03); the GPU side of
the arrival frame, `world`'s 12.1 ms maximum (Baseline) plus the far field's
draws, is Not verified, 5.

## A heightfield under M5's decisions

Terrain is a later milestone and M5's ground stays flat. These rows size a
heightfield over the 10.24 km world against what M5 fixes, so that terrain's
charting knows which of M5's decisions it reopens. They choose no terrain
technique: the form sized is the plainest, a regular grid a Chunk as one Mesh
with one BLAS, cooked, loaded and held as any Mesh is at `99b4141b`. Nothing
here was run; every time is derived from the measurements named.

- **Resolution.** 1 m between posts, with 2 m and 0.5 m beside it. A Chunk is
  256 m (`src/assets/cooked.rs:58`): at s metres, 2 × (256 / s)² triangles
  and (256 / s + 1)² vertices, a Chunk's edge posts repeated in its
  neighbours.
- **Arena bytes**, as `MeshPool::written_bytes` counts them
  (`src/assets/pools.rs:292-312`): 28 a vertex (`src/assets/cooked.rs:475`),
  12 a traced triangle's indices, 4 a Meshlet triangle, 16 a Meshlet
  (`src/assets/mesh.rs:51`) and 40 a Mesh (`:34`), with Meshlet vertices and
  Meshlets at the helmet's ratios, 17,768 Meshlet vertices
  (`docs/audit/m3.5/scalability.md:381`) and 278 Meshlets over 23,358
  triangles. A G1 chain at most doubles the three Meshlet Arenas' bytes and
  adds no vertex or index.
- **BLAS bytes.** 94.0 a triangle built and 45.5 compacted on the M4 Pro
  (`src/render/acceleration_structures.rs:49`), measured over 2,117,632
  triangles, a 2 by 2 km heightfield in 64 BLASes (32,768 triangles each) and
  four rocks (`docs/research/global-illumination-and-lighting.md:88`, `:97`);
  70.7 and 29.2 on the RTX 2070 SUPER (`:41`). The ledger charges wgpu-hal's
  size query times 45.5 / 94.0 (`src/assets/pools.rs:587-588`, `:852-857`),
  which may differ (Not verified, 13).
- **BLAS build.** That heightfield's 2,097,152 triangles took 29 to 36 ms of
  GPU, 13.8 to 17.2 ns a triangle, and one BLAS of 5,120 triangles 0.6 to
  0.75 ms (same note, `:98`), which leaves 0.53 to 0.66 ms for a frame's
  builds whatever they hold; ten and a hundred such BLASes, 1.2 to 1.55 and
  7.8 to 9.3 ms, fit the same line (1.24 to 1.54, 7.6 to 9.5). Scaled to one
  BLAS a Chunk (Not verified, 9).
- **In use and Resident.** A Chunk counts as a user of its Meshes until it
  unloads past the hold radius (`src/streaming/io.rs:811-818`), so 20 to 24
  heightfields are in use once the camera has moved (Scale assumptions). A
  Mesh left without users stays Resident, its BLAS with it, until a refused
  load frees it, least recently used first (`src/assets/pools.rs:314-327`,
  `:341-342`, `:389-403`): the Arenas must hold the 24 in use, and fill with unused ones
  beyond them.
- **The arrival.** A Fast travel's arrival loads nine Chunks over three
  frames, four on the first, whose CPU reads 8.5 to 9.0 ms, in a hold of 23.9
  to 26.4 ms (`docs/performance.md:56-59`, `:70-72`, `:85-90`, `:101-104`).
- **A pixel** spans 0.575 mrad (survey 3, Where Praxis stands): 0.1473 m at
  the Load radius, 1.15 m at 2,000 m.
- **Relief.** A grid of spacing g lies within g² / 4R of relief whose radius
  of curvature is R both ways, the sagitta of a triangle's long edge, g√2; a
  simplifier on real relief keeps fewer triangles on flats and more on breaks
  (Not verified, 10).

A Chunk's heightfield:

| posts | triangles | vertices | Arena bytes, LOD 0 | BLAS bytes, M4 Pro, built / compacted | BLAS bytes, RTX 2070 SUPER, built / compacted | BLAS build alone in a frame, M4 Pro |
|---:|---:|---:|---:|---:|---:|---:|
| 2 m | 32,768 | 16,641 | 1,096,220 | 3,080,192 / 1,490,944 | 2,316,698 / 956,826 | 0.98 to 1.22 ms |
| 1 m | 131,072 | 66,049 | 4,370,340 | 12,320,768 / 5,963,776 | 9,266,790 / 3,827,302 | 2.34 to 2.91 ms |
| 0.5 m | 524,288 | 263,169 | 17,452,484 | 49,283,072 / 23,855,104 | 37,067,162 / 15,309,210 | 7.78 to 9.66 ms |

At 1 m: 66,049 × 28 + 131,072 × (12 + 4) + 99,704 × 4 + 1,560 × 16 + 40 =
4,370,340 bytes, 33.3 a triangle, and 5,318,364 with a chain. Its build:
0.53 + 131,072 × 13.8 ns = 2.34 to 0.66 + 131,072 × 17.2 ns = 2.91 ms.

| id | what M5 fixes | where | the heightfield at 1 m posts | arithmetic | verdict |
|---|---|---|---|---|---|
| H01 | The geometry Arenas hold the densest 2 km view: its Far proxies' 630,396 LOD 0 triangles, 1.26 M with chains, of the Meshlet triangle Arena's 2,202,009 (the M5 spec's bake decision, item 1) | `src/assets/pools.rs:16`, `:61-70`; `bench/budgets.json:10`; S13, S14 | 131,072 triangles a Chunk, 3,145,728 over the 24 in use | At today's shares the Meshlet triangle Arena is short first: 3,145,728 / 2,202,009 = 1.43 times at LOD 0, and 2 × (3,145,728 + 630,396) = 7,552,248, 3.43 times, with the proxies and every chain. The Budget cannot grow by that at these shares: 3.43 × 40,852,520 = 140,111,888 bytes of vertex Arena pass the 134,217,728-byte binding (S14). In bytes, 24 heightfields with chains are 24 × 5,318,364 = 127,640,736, 1.52 times the 83,886,080-byte Budget, and 155,000,430 with the proxies' 630,396 triangles at the helmet's 43.4 bytes with a chain, 27,359,694, 1.85 times. At 2 m: 2 × (786,432 + 630,396) = 2,833,656, 1.29 times the Meshlet triangle Arena, in 24 × 1,333,196 + 27,359,694 = 59,356,398 bytes, 0.71 of the Budget | the fit test survives as the judge; **must change**: the Arenas' shares (`ARENA_PER_MILLE`) at either resolution, and at 1 m the geometry Budget, about 1.85 times, out of the 2,044,088,320 bytes no Budget holds (S06) |
| H02 | Every BLAS holds its Mesh's LOD 0, within the acceleration structures' Budget (the M5 map's BLAS decision) | `bench/budgets.json:14`; `acceleration_structures` in `world.json` under `bench/baselines/apple-m4-pro/` and `nvidia-geforce-rtx-2070-super/`; `src/assets/pools.rs:573-640` | 5,963,776 bytes a Chunk compacted, 12,320,768 until its load frame's compaction | `world`'s peak is 155,577,592 of 268,435,456, so 112,857,864 are free. The 24 in use: 143,130,624, 1.27 times that; with the proxies' 630,396 × 45.5 = 28,683,018, 171,813,642, 1.52 times, and more for each unused heightfield the Arenas still hold. At 2 m: 35,782,656 + 28,683,018 = 64,465,674, 0.57, and the 48,392,190 left hold 32 unused heightfields, 56 in all. On the RTX 2070 SUPER at 1 m, 24 × 3,827,302 + 630,396 × 29.2 = 110,262,811 of the 139,898,996 its Baseline leaves, 0.79 | LOD 0 in every BLAS survives; at 1 m on the M4 Pro the Budget **must change**; survives at 2 m while the Arenas hold 56 heightfields or fewer |
| H03 | The static TLAS's 264,720 entries, 11,114 free after 219 Far proxies, and the 517,376 Instance slots (S01, S02); the coarsest LOD under a pixel, picked per Instance (the M5 map's LOD decision) | `src/render/renderer.rs:1039-1041`; `src/scenes/world.rs:25`, `:489-493` | One Instance a Chunk in place of its four 128 m ground tiles | 3 entries fewer a Chunk, 24 to 36 fewer in the near field; a Far proxy stays one entry. Selection picks one LOD for a Chunk-wide Instance, and the Chunk the camera stands in draws LOD 0 whole, 131,072 triangles; 12 Chunks at LOD 0 are 1,572,864, 0.31 of ADR 0013's 5 M visible triangles. Cut into patches of p metres for a LOD each, (256 / p)² a Chunk: 12 Chunks at 32 m take 768 entries, at 16 m 3,072, at 8 m 12,288, past the 11,114; 24 Chunks at 16 m take 6,144 of the 113,125 free slots | both survive, as one Mesh a Chunk and down to 16 m patches |
| H04 | N = 10: a newly loaded Chunk is in the static TLAS within 10 frames (M5 criterion 6) | `src/render/renderer.rs:656-660`; `src/assets/pools.rs:686-716`; S03 | No entry added (H03) | The static TLAS's build follows its entries, 44 to 74 ns each, and none is added. A Chunk's BLAS is built and compacted in its load's own frame, before the static TLAS takes it, so its build lengthens that frame (H05), not the wait N counts. A build moved off the load frame would add its frames to N, which has none to spare | survives, at the zero margin M4 measured, while the BLAS is built in the load's frame |
| H05 | No frame past 16,667 µs and a Fast travel's hold within 50 ms (M5 criteria 1 and 4), under the per-frame load caps, M5's load-count cap among them | `src/render/renderer.rs:660`, `:1042`; `src/assets/pools.rs:695`; `src/streaming/io.rs:891-904` | A new BLAS with every Chunk, where `world`'s Meshes are a library its first Chunks load once | `upload` waits on the main thread for the GPU to finish what is submitted, the frame's new BLASes included: at least 0.53 to 0.66 ms and 1.81 to 2.25 ms a Chunk. The arrival's first frame loads four Chunks, which the bytes cap still admits (2,170,752 + 4 × 4,370,340 = 19,652,112 of 22,937,393): 7.8 to 9.7 ms on its 8.5 to 9.0 ms, 16.3 to 18.7 ms. The hold's nine Chunks in three frames add 17.9 to 22.2 ms: 41.8 to 48.6 ms. At 2 m the first frame reads 10.8 to 11.9 ms and the hold 29.6 to 33.4 ms | **must change** at 1 m: the arrival's first frame passes 16,667 µs except at the low end of both ranges, by a wait that is a floor; the hold survives with 1.4 ms at the slow end; both survive at 2 m. A lower cap moves waits into the hold, and a build off the load frame spends H04's margin |
| H06 | A Far proxy is one merged Mesh under a cap of 5,832 triangles, its Chunk's ground tiles a second Primitive of 8 triangles with the ground's own Material (the M5 spec's bake decision, items 1 and 6) | the decision; `src/scenes/world.rs:25`; `src/scenes/mod.rs:1429-1447` | Its Chunk's relief, to a pixel at the Load radius, 0.1473 m | As a grid of spacing g = √(4 × 0.1473 × R), 2 × (256 / g)² triangles: 2,225 at R = 100 m (g = 7.68 m), 4,450 at R = 50 m (5.43 m), 11,125 at R = 20 m (3.43 m). The cap is the densest Chunk's walls and roofs alone, so the relief comes on top of it. Over 285 Resident proxies, 4,450 each are 1,268,250 LOD 0 triangles; with the proxies' 630,396, 1,898,646, 0.86 of the Meshlet triangle Arena before any Chunk or chain. Traced, the BLAS bytes H02 leaves at 2 m hold at most 48,392,190 / 45.5 / 285 = 3,731 triangles a proxy (g = 5.93 m, within a pixel for R of 59.6 m or more); at 1 m none are left. The ground's own Material costs no texel only while it tiles | one Far proxy per Chunk survives; its cap and its 8-triangle ground **must change** |
| H07 | Past the View distance only flat ground is drawn (the M5 map's far-field decision) | the decision; the M5 spec, story 21 | Relief past 2 km | Relief h metres high at d metres stands h / (0.000575 × d) pixels tall: 100 m at 2 km, 87; 100 m at 5 km, 35; 300 m at 10.24 km, 51; 300 m across the world's diagonal, 14.48 km, 36. A flat plane meets the far field's edge with a step wherever the relief there is more than a pixel, 1.15 m, from it. The whole world as one Mesh is 204,800 triangles at 32 m posts, 0.093 of the Meshlet triangle Arena, within a pixel at 2 km for R of 32² / (4 × 1.15) = 222 m or more, and 819,200 at 16 m, 0.37, for R of 56 m or more; Far Cry 5 keeps its terrain's 2 km root tiles resident (survey 3, row 7) | **must change**: the relief is drawn past 2 km; "nothing past the View distance" holds for what stands on it |
| H08 | Far proxies cast and receive the sun's shadows from static TLAS entries out to 2 km, with no far TLAS (the M5 map's BLAS decision) | `src/scenes/mod.rs:547`; the decision | Relief past 2 km casts shadows the static TLAS does not hold | `world`'s sun, along (0.5, −1.0, −0.3), stands atan(1 / √(0.5² + 0.3²)) = 59.75° up, so relief h high just past the edge leaves h / tan 59.75° = 0.58 h of the far field unshadowed: 58 m for 100 m, 51 pixels at 2 km. A building past the edge does the same today, over 0.58 of its height. The near field is reached when the sun stands lower than atan(h / d): at 2 km, 353 m of relief at 10°, 175 m at 5°, 70 m at 2°; M6's Reference scene is to have a moving sun (`docs/audit/strategy/2026-11.md:94`). H07's 32 m Mesh would be one entry and 204,800 × 45.5 = 9,318,400 bytes, its box over every Instance (survey 4, rows 11 and 13) | **must change**: relief drawn past 2 km (H07) is traced, or its shadows are missing, which reopens the far TLAS |

Must change when terrain is charted, ranked by what it blocks; none changes
an M5 criterion or an M5 ticket, since M5's ground is flat:

1. **Something of the relief is drawn past the View distance** (H07). Blocks
   any view of relief: a hill 100 m high stands 87 pixels tall at the far
   field's edge.
2. **The load frame's wait for a Chunk's BLAS** (H05;
   `src/render/renderer.rs:660`). Blocks the every-frame clause on the
   arrival at 1 m posts; its two ways out cost the hold or N = 10 (H04).
3. **The Far proxy's cap and its ground Primitive** (H06). Block the far
   field's relief between the Load radius and 2 km.
4. **The geometry Arenas' shares, and at 1 m their Budget and the
   acceleration structures'** (H01, H02; ADR 0013:3). A Budget moves up
   through an ADR or a map comment.
5. **Relief past 2 km in a TLAS** (H08). Blocks the shadows of whatever H07
   draws: over 0.58 of its height at `world`'s sun, into the near field under
   about 10°.

## Must change, ranked by what it blocks

1. **A per-frame bound on load count or main-thread load time in
   `FrameIo::fits`** (`src/streaming/io.rs:891-904`; ADR 0017:19). Blocks the
   Fast travel's every-frame clause, and any streaming frame, under any far
   field that streams proxies: the caps today admit 63 proxies a frame at
   10,000 triangles, where 10 to 14 fill the arrival frame.
2. **Far geometry beyond one merged proxy per Chunk stays out of the static
   TLAS** (`src/render/renderer.rs:185`; ADR 0020:20). Blocks criterion 7's
   N = 10, whose build already sits at its 19.6 ms limit: 11,114 entries
   remain after F1's proxies, and any growth in capacity lengthens recording
   whether filled or not. Survey 4's far-field F1, a TLAS of its own, is the
   form that keeps the near field's build as M4 measured it.
3. **The Material table's 222 entries** (`src/assets/pools.rs:24`, `:74`;
   ADR 0017:18). Blocks F1's proxies drawing with their settlements' or their
   own Materials: 219 of 222 at 9 regions' facades, and a refused Chunk past
   it.
4. **Instance slots, 517,376** (S01; ADR 0003). Blocks only the far field
   without HLOD: 4.3 to 4.5 times short at the world's density.
5. **The flat per-slot cull** (S09; ADR 0019:35). Blocks only the far field
   without HLOD: 3.3 to 3.4 ms of per-slot stages against the 12 ms GPU
   frame, plus appends that follow the triangles drawn.
6. **The light buffer, the far Froxel slice and the shadow mask's bits**
   (S10 to S12; ADR 0019:38). Block only lights past the Load radius: 2.63
   times the buffer at `town`'s lamps over 2 km.

## Not verified

1. **The static TLAS's build time past 264,720 entries.** Linear scaling at
   44 to 74 ns an entry is an assumption; survey 4's probe stops at 262,144.
   Settles it: that probe on the M4 Pro at capacities of 0.5, 1 and 2.3 M,
   full and empty.
2. **The world's static Instances and Materials within 2,000 m.** Derived
   from the world's mean density and its regions. Settles it: the loop of
   `the_densest_view_fits_the_capacities_the_budgets_give`
   (`src/scenes/world.rs:731-795`) at 2,000 m, CPU only, not run here as
   other sessions bench on this machine.
3. **What a one-Placement proxy load costs on the main thread** (sizing,
   load, mip planning). Settles it: the per-frame probe of #773 on `world`'s
   bench segment once proxies cook.
4. **What far proxies in the static TLAS cost near-field rays.** Epic
   measured 44% on PS5 (survey 4, row 11); Praxis's shadow rays are
   unmeasured with them. Settles it: shadow-ray time on M5's Reference
   scene with proxies in the static TLAS and in a TLAS of their own.
5. **The far field's GPU frame on the arrival.** Settles it: GPU frame
   maximum on M5's Reference scene bench.
6. **The flat cull's per-slot cost on `world`.** 1.464 ns is `town`'s
   per-slot stages; no Baseline records `world`'s slots in use.
7. **The N model.** 2 + wait / frame period is fitted to three probe runs of
   one segment. Settles it: `frames_until_traced` on M5's Reference scene at
   two frame periods.
8. **Triangles in view with LOD at 2 km** (S07, S08). Settles it: G1's chains
   cooked for the world's Meshes and `triangles_drawn` on M5's Reference
   scene.
9. **What one Chunk's BLAS costs the load frame** (H05). 0.53 to 0.66 ms a
   frame and 13.8 to 17.2 ns a triangle are fitted to a one-off probe that is
   not in the repo, whose largest BLAS held 32,768 triangles; the wait also
   covers whatever else is submitted, compaction's copy and its second wait
   (`src/assets/pools.rs:719`), and the Blob's read and decode is not in it.
   Settles it: the `blas_compaction` stage on a scene that loads a Mesh of
   131,072 triangles.
10. **The triangles a heightfield keeps at a given error** (H06, H07).
    g² / 4R is a uniform grid's bound on relief of one curvature. Settles it:
    G1's chain and a Far proxy cooked from a measured heightfield.
11. **What a Chunk-wide BLAS under every Instance of its Chunk costs a ray.**
    Its box overlaps theirs (survey 4, row 13). Settles it: shadow-ray time
    on a streamed scene with and without the heightfield.
12. **A grid's Meshlets.** The Meshlet and Meshlet vertex Arenas' bytes use
    the helmet's ratios; a grid cut in index order may differ, in the two
    smallest shares of H01.
13. **A heightfield BLAS's bytes in the ledger** (H02). 94.0 and 45.5 a
    triangle are the probe's allocated sizes; the ledger charges wgpu-hal's
    size query. Settles it: `blas_bytes` for 131,072 triangles on each
    adapter.
