# Scalability check: M4's capacities and static TLAS under M5's far field

Lane 4 of `docs/agents/audit.md` (scalability and reality check), run against
M5's scope only: LOD chains, an HLOD or far proxy per Chunk out to the 2 km
view, far proxies in the TLAS, and millions of Resident Instances. Every
`path:line` is at `main` `d4ac21eb8ca9df5927cb74c50504f57478298efb`. One
reader prepared it from the tree, the Baselines under `bench/baselines/`,
`docs/performance.md` and the M4 exit's recorded figures; it ran no bench, no
capture and nothing on the GPU, and no adversarial verifier has reopened it
yet. Every number is read from a file or derived in the cell that states it.
It charts nothing; the decision tickets on M5's map use it.

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
| S09 | Flat per-slot cull, `SCAN_BLOCK_SIZE`, `MAX_WORKGROUPS_PER_DIMENSION` | `src/render/culling_pass.rs:1-13`, `:47`, `:53` | 256 slots a block; every slot in use is tested; `town` 1,136.7 µs at 254,256 slots, 4.5 ns a slot including appends (Baseline) | Without HLOD: S01's 2.24 to 2.31 M slots in use. With HLOD: M4's plus 219 | Without: 2,239,514 × 4.5 ns = 10.1 ms to 2,310,491 × 4.5 = 10.4 ms against the 12 ms GPU frame (an upper bound: `world`'s phase one reads 194.2 µs at a slot count no Baseline records). Workgroups: 2.31 M / 256 = 9,026 per dimension, under 65,535 | survives with HLOD; must change without it (ADR 0019's per-Instance culling) |
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
| 0019 | "Culling tests Instances" | survives with HLOD; must change without | S09: 10.1 to 10.4 ms at 4.5 ns a slot over 2.24 to 2.31 M slots | `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md:35` |
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
   without HLOD: 10.1 to 10.4 ms against the 12 ms GPU frame.
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
6. **The flat cull's per-slot cost on `world`.** 4.5 ns is `town`'s, appends
   included; no Baseline records `world`'s slots in use.
7. **The N model.** 2 + wait / frame period is fitted to three probe runs of
   one segment. Settles it: `frames_until_traced` on M5's Reference scene at
   two frame periods.
8. **Triangles in view with LOD at 2 km** (S07, S08). Settles it: G1's chains
   cooked for the world's Meshes and `triangles_drawn` on M5's Reference
   scene.
