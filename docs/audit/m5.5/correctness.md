# Audit lane 1, correctness against intent

The lane's report, in the form `docs/agents/audit.md` gives: every ADR's
decision clauses read against the code at the frozen commit
`e5c55a8da2a59db2f09c99c8ea4522987d5354ac`, a verdict per clause, the
stories of the M4 and M5 specs, the oracles added since `M3.5`'s frozen
commit `8bc34eb18fcfb79307193551e96619316aef566c` and every registry key
cited in code that landed since checked the same way. Five readers, one per
module, each a section below, appended in the order they were written; the
lane's report session writes the Summary, the counts and the Killed section
and runs the verifier over every section. Finding ids carry the module's
letter so that sections written in parallel never collide: render
`L1-R<n>`, shaders `L1-S<n>`, streaming with the frame `L1-F<n>`, cook with
assets and scenes `L1-C<n>`, tooling `L1-T<n>`.

## streaming

Scope: `src/streaming/` (4,439 lines, all new since `M3.5`'s frozen
commit), `src/state.rs`, `src/threading.rs`, `src/components.rs`,
`src/lib.rs` and the feature modules as far as they changed after
`8bc34eb1`, read at the frozen commit `e5c55a8d`. Two readers at high
effort shared the module, each with the lane brief verbatim: the Chunk
streamer, the IO pool and mip priority (`src/streaming/`); and the frame,
`State` with the compute pool, the components, the transforms and the
feature modules. Each reader's scope, clause verdicts, stories, registry
keys, oracles and open issues are the two subsections below; their
Findings are one table, the streaming reader's numbered from `L1-F01`, the
frame reader's from `L1-F41`. ADRs read whole: 0004, 0006, 0009, 0010,
0012, 0014, 0015, 0016, 0017, 0022 and 0026; read for the clauses this
module governs: 0003, 0013, 0018 and 0023. ADR 0017's capacity clause,
handed to the lane as known broken, carries the verdict `broken, known`
and no Finding: the capacity lives in the assets module. `deviations.md`
holds one row in these files, `src/streaming/mips.rs:Sight::Ahead`, which
the code matches. No `cargo` command was run: every timing, Budget and
image question is under *Notes for the verifier*.

### the Chunk streamer, the IO pool and mip priority

Scope: `src/streaming/`: `mod.rs` (796 lines), `io.rs` (2,339), `chunks.rs`
(415), `mips.rs` (889); 4,439 lines, every file read in full, tests
included, all of it new since the last audit's frozen commit. Opened only as
far as they state what `src/streaming/` delivers or consumes:
`src/assets/material.rs` (`TextureShape`, `ClassTable::has_room`,
`EvictionRank`, `StreamedMips`, `texture_residency`, `stream_mips`,
`evict_for`, `evictable`, `full_class_for_mips`,
`coarsest_evictable_needs`, `evict_finest_mip`, `write_layer`),
`src/assets/cooked.rs` (`resident_tail_level`, the Texture Blob readers,
`CookedCache::read`), `src/render/staging.rs` (`mips_written_bytes`),
`src/state.rs` (construction, `advance`, the shift), `src/stats/mod.rs`
(the streaming counters' docs), `src/camera/mod.rs` (`FIELD_OF_VIEW`),
`src/scenes/mod.rs` (the radii), `CONTEXT.md`'s streaming terms. ADRs read
whole, amendments included: 0016, 0017, 0026; read for the clauses
`src/streaming/` governs: 0003, 0013, 0014, 0018, 0022. Specs read: the M4
spec (Streamed world) and the M5 spec (Far field), stories, implementation
decisions and testing decisions. `docs/audit/deviations.md` holds one row in
these files, `src/streaming/mips.rs:Sight::Ahead`, which the code matches.
Not checked: anything needing a run (no build, test or bench was run), so
every timing, Budget and image claim is under *Notes for the verifier*; the
pools' own room, eviction and refusal logic beyond what `src/streaming/`
calls is the assets reader's. Every `where` is `path:line` at
`e5c55a8d`, under `src/streaming/` unless given in full.

#### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0017 | "Textures are ordinary mip chains streamed whole mip at a time in wgpu's safe API, chosen by CPU heuristics" | holds | `mips.rs:205-222`; `io.rs:1155-1162` (through `LoadStaging`) |
| 0017 | "a Texture a Chunk loads arriving as its Resident tail, its finer mips streaming in by the priority below through the IO pool" | holds | `io.rs:525`, `:615-625`, `:1432`, `:1480`; mips through the IO pool `io.rs:331-334`, `:1121-1123` |
| 0017 | "while one a spawn loads keeps its full mip chain" | holds | `io.rs:878-885` (`reads_resident_tails: false`), `:621-623` |
| 0017 | "a Far proxy's atlas arriving with its mips down to the one its distance from the camera needed when it was requested, as the priority below judges a drawn use" | holds (the read also applies to the ground Material's Textures; see *Notes*) | `io.rs:508-526`, `:1530`; `mips.rs:390-403` |
| 0017 | "drawn from its tail only where its Channel has no room for that chain or the camera has closed on it since" | falls in assets (the Material pool's room on load) | |
| 0017 | geometry through the same belt into fixed Arenas that refuse a Mesh that does not fit; capacities derived at device creation | falls in assets | |
| 0017 | Channels as binding arrays of Texture classes; residency changes batched, one rebuild a frame | falls in assets and render | |
| 0017 | "Priority is computed on the CPU: the finest mip any visible Instance needs, from distance and the Primitive's UV texel density ... against the 1440p pixel footprint" | holds | `mips.rs:21-44`, `:414-416` |
| 0017 | "Chunks inside the load radius are requested ahead" | holds | `mod.rs:230-251` |
| 0017 | "No GPU feedback readback until a measured miss rate asks for it" | holds: no readback in the module | `mips.rs:133-249` |
| 0017 | "The visible Instances are the drawn Placements of the loaded Chunks and the Renderables, the latter at their World transforms on each plan as they move, frustum aside" | holds | `mips.rs:162-202`; `io.rs:1109-1119` |
| 0017 | "judged at the nearest point of the sphere around its Mesh's bounds, at its smallest scale" | holds | `mips.rs:374-381`, `:407-427` |
| 0017 | "against the footprint of a pixel at the corner of a 16:9 view 1,440 pixels tall, where a pixel's angle is narrowest" | holds: 2 tan(fov/2) / 1440 / (1 + tan²θ_corner), the radial footprint | `mips.rs:21-32` |
| 0017 | "a Texture's mips from the finest needed to its finest Resident are requested in one read, finest-needed first" | holds | `mips.rs:213-221`, `:243-244`; `io.rs:564-572` |
| 0017 | "The mips of the Chunks and Far proxies held past their radius ... are requested as the drawn ones' are and are never late, as Unreal's streamer prestreams ... there one mip coarser" | holds; matches the `deviations.md` row | `mips.rs:56-59`, `:320-323`; `io.rs:1093-1108` |
| 0017 | "each request is judged from where the camera, closing ... at its last frame's rate, will be the Chunk streamer's least lead of frames later, at most a Chunk edge nearer" | holds | `mips.rs:49`, `:312-319` |
| 0017 | "eviction ranks a Texture by that same need, while the late mips are still judged from where the camera is" | holds | `mips.rs:209`, `:293`; `io.rs:1152-1154` |
| 0017 | "The Materials rays reach off screen ... take the Resident tail as their floor" | falls in assets (the tail never leaves); nothing in the module plans a ray's use | |
| 0017 | "Each texture's Resident tail ... never leaves ... charged twice in `resident_tails`" | falls in assets | |
| 0017 | "When a request would exceed the texture pool's capacity, which is set from its Budget" | broken, known (see *Open issues read*) | `src/assets/material.rs:231-245` |
| 0017 | "the finest mip of the lowest-priority texture goes first, least recently used breaking ties" | holds | `mips.rs:286-300`; `src/assets/material.rs:1101-1118` |
| 0017 | "a mip loads at priority p and is evicted only below p − 1, so it does not thrash" | holds; test `mips.rs:454-482` | `mips.rs:294-295` |
| 0017 | "the candidates are the Textures whose chain lies in the very class the request needs, whose slice it takes" | holds: the plan's room check keys evictable needs by Channel and class shape | `mips.rs:236-241`; `src/assets/material.rs:1143-1171` |
| 0017 | "the one evicted moves to the class of its mips from the next, or falls back to its tail only when it needs no finer mip than the tail's" | falls in assets | `src/assets/material.rs:1125-1141`, `:1174-1212` |
| 0017 | "a request is made only when its move finds room, free or by an eviction" | broken: L1-F01 | `mips.rs:233-242` |
| 0017 | "Streaming uploads go through a `StagingBelt`, never `Queue::write_texture`" | holds: the module writes only through `LoadStaging` | `io.rs:1155-1162`, `:1489-1495` |
| 0017 | "the engine caps the bytes written into it per frame ... an upload past the cap waits for a later frame" | holds | `io.rs:1313-1325` (loads), `:1139-1144` (mips) |
| 0017 | "the IO pool (ADR 0016) decodes into CPU memory and the main thread copies it into the belt" | holds | `io.rs:437-541`, `:1475-1506` |
| 0017 | "The cap holds each frame's streamed Chunk loads to a third of the staging Budget ..." | falls in render (the value comes from the Renderer's capacities, `src/state.rs:179`) | |
| 0017 | "each Chunk is sized from its decoded Assets before it loads, nearest the camera first, and one that alone writes past the cap loads with no other writes in its frame rather than never" | holds | `io.rs:1298-1308`, `:1320-1324`, `:1421-1469` |
| 0017 | "A capture's Settled wait loads past the cap ... and a spawn's loads are not capped" | holds | `io.rs:1314-1316`, `:1142`; spawns load through `reader()`, `io.rs:878-885` |
| 0017 | "A frame's Chunk loads also place at most 131,072 Instances ... a Chunk that alone places more is placed alone (the cap counts at placement)" | holds | `io.rs:760`, `:1378-1388` |
| 0017 | "a Texture read on the main thread for a Chunk, one a load still in flight claimed, is its Resident tail alone" | holds | `io.rs:615-625`, `:1432`, `:1480`; test `:2161` |
| 0017 | "a Chunk that needs an Asset a Chunk read in flight claimed waits for the reads in flight when its own came back instead, so the main thread reads one only after a failed read or a refused load left it claimed" | holds for the wait; broken after a failed read or a refused load: L1-F03 | `io.rs:404-413`, `:1395-1416`; `:54-70`, `:597-604` |
| 0017 | "A frame also loads at most 3 Chunks ... a load past the count waits for a later frame, nearest the camera first, and a Settled wait loads past it" | holds; tests `io.rs:1794`, `:1809`, `:1824` | `io.rs:765`, `:1317-1319` |
| 0017 | "A Mesh a streamed load brings that the pools lack does not hold its frame ... placed in the first frame that finds every Mesh they name ready ... a Settled frame and a spawn's loads still wait for theirs" | holds; test `io.rs:1762` | `io.rs:941-945`, `:1378-1380`, `:1496`, `:1508-1511` |
| 0017 | "A Far proxy's load counts as a Chunk's does: a frame's 3 loads are its Chunks' first, then its Far proxies', nearest first" | holds: `loads` is not reset between the two streamers | `io.rs:1011`, `:1284-1289`, `:1255` |
| 0017 | "The cooked format ... stores each mip as a separately loadable byte range of the Texture's Blob" | falls in cook with assets (the runtime reads the whole Blob; see *Notes*) | |
| 0017, 0013 | the ledger: buffers, textures and acceleration structures created through `GpuMemory` | holds: the module creates no GPU object outside its tests | `mips.rs:616` and the other `submit` calls are test-only |
| 0026 | "Every Chunk outside the Load radius and within the View distance draws its Far proxy, one Instance, in place of its Placements" | holds, streaming side; the draw falls in render | `chunks.rs:132-151`; `io.rs:1005-1022`, `:1210-1215` |
| 0026 | What it holds, its Mesh, its Material, its Blobs | falls in cook | |
| 0026 | "A Far proxy shows exactly when its Chunk is not drawn, so the swap is at the Load radius, a whole Chunk at once, with no fade" | holds | `io.rs:1006-1010`, `:1020` |
| 0026 | "A Late Chunk keeps its Far proxy until it loads: it shows coarse, never as a hole" | holds: only placed, unheld Chunks hold their proxy | `io.rs:1008-1009` |
| 0026 | "A Far proxy entering at the View distance appears the same way, whole and at once" | holds: one Placement, placed whole | `chunks.rs:137-150`; `io.rs:1378-1388` |
| 0026 | "requested when its Chunk comes within the View distance, and ahead of that while the camera closes on it ... released past the View distance plus one Chunk's edge" | holds; tests `io.rs:1921`, `mod.rs:706` | `mod.rs:149-157`, `:211-251`, `:296-298` |
| 0026 | "It stays Resident while its Chunk is drawn, so leaving a Chunk never waits on a read" | holds: the View distance is never less than the Load radius, and a proxy unloads only past the View distance plus an edge | `mod.rs:212-215`; `io.rs:783-784` |
| 0026 | "Past the View distance ... flat ground ... one Resident quad" | falls in scenes and render (`chunks.rs:21-25` holds the resource) | |
| 0026 | Which draws: "each a Chunk of one Placement that takes one Instance slot ... held ... while it is past the View distance or its Chunk is drawn" | holds | `chunks.rs:15-19`; `mod.rs:284-288`; `io.rs:1210-1215` |
| 0026 | "So the swap falls in the frame the Chunk's Placements are placed, once their BLASes are ready, and in the frame the Chunk is held or unloaded" | holds: the Far proxies' held set is taken from the Chunks kept that same frame | `io.rs:999-1010` |
| 0026 | "the proxy is placed once that BLAS is ready, as a Chunk is" | holds | `io.rs:1508-1511` (`MeshUse::FarProxy`) |
| 0026 | static TLAS entries, mask bit, the 120-frame build | falls in render | |
| 0026 | "Mip priority reads the drawn Far proxies beside the drawn Chunks' Placements" | holds | `io.rs:1109-1115` |
| 0026 | "A Far proxy held past the View distance has its mips requested too, never late" | holds | `io.rs:1105`; `mips.rs:320-323` |
| 0026 | "one held while its Chunk is drawn needs none" | holds (see *Notes* on what eviction then does to it) | `io.rs:1106`; `mips.rs:167-169` |
| 0026 | "A Far proxy is read with its atlas's mips down to the one it needs from where the camera was when it was requested ... one within the Load radius holds the mips it needs where it takes over from its Chunk" | holds at the read; test `io.rs:2256` | `io.rs:508-526`, `:1530` |
| 0026 | Open land: "A Primitive without triangles needs no Material" | falls in render and assets | |
| 0016 | "Asset reads and decoding run on a separate IO pool outside the compute pool" | holds: std threads, not the compute pool | `io.rs:306-316` |
| 0016 | "They decode into CPU memory, and the main thread copies it into the `StagingBelt`" | holds | `io.rs:437-541`, `:1475-1506` |
| 0016 | "The IO pool is four std threads that take the Chunk streamer's requests and mip streaming's" | holds | `io.rs:38`, `:154-157` |
| 0016 | "each reads a Chunk and the Meshes, Materials and Textures it names that no other load claimed, and decodes them where it read them, touching no GPU object" | holds | `io.rs:437-541` (claims `:452-458`), `:564-572` |
| 0016 | "the main thread loads them into the pools" | holds | `io.rs:1475-1506` |
| 0016 | "They take Chunk reads first, then mips, then Far proxy reads, each in the order requested" | holds; test `io.rs:2054` | `io.rs:171-176`, `:188`, `:200-213`, `:224-226` |
| 0016 | "a Far proxy's only while no Chunk read is taken and on all threads but one" | holds | `io.rs:42`, `:206-210`, `:257-266` |
| 0016 | "a Far proxy read with its mips decodes them" | holds | `io.rs:508-526` |
| 0016 | "Decoding stays on those threads" | holds | `io.rs:319-335` |
| 0016 | "Only the main thread submits, presents or polls the wgpu `Queue` and `Device`" | holds: IO threads hold no GPU object; BLAS polling runs on the calling thread | `io.rs:941-945` |
| 0016 | compute pool, `FramePreparer`'s pool, workers recording encoders | falls in threading and render | |
| 0003 | "no GPU buffer grows at runtime, and a request past a buffer's capacity is refused and logged" | holds, streaming side: a refused load is logged and counted, then retried; the pools' refusal falls in assets | `io.rs:1550-1563`; `mod.rs:190-197`; `chunks.rs:88-94` |
| 0014 | no World position in f32 | holds: Placements, lights, proxy pivots and texture-use spheres are f64 | `chunks.rs:33-34`, `:44-51`, `:104`, `:120-128`; `mips.rs:68-73`, `:420-427` |
| 0018 | "the runtime loads cooked Blobs ... from them by Asset ID" | holds | `io.rs:446-450`, `:565`, `:1520-1523` |
| 0018 | "A dev launch cooks only what it loads: each Asset as its load finds the Blob missing or stale" | holds | `io.rs:825-847`, `:1524-1529` |
| 0013, checklist | "per-frame GPU writes add to `uploaded_bytes`" | holds | `io.rs:1163`, `:1341`; `src/state.rs:287` |
| 0022 | "Game-state changes run in the fixed schedule, which reads no wall clock" | holds: streaming runs after the Ticks, outside the fixed schedule; its `Instant` reads time Fast travel stats only | `src/state.rs:276-281`; `io.rs:915-918`, `:926` |

#### Spec stories

| Story | Verdict | Where |
|---|---|---|
| M4 spec, 17 (Moving Instances stay spawned by scene systems) | falls in scenes; mip priority plans them as Renderables | `mips.rs:174-202` |
| M4 spec, 18 (captures render Settled frames) | holds | `io.rs:1038-1045`, `:1302-1304`, `:1540-1542` |
| M4 spec, 19 (Chunks within the Load radius loaded as I move) | holds; test `mod.rs:462` | `mod.rs:240-282` |
| M4 spec, 20 (Chunks that leave unloaded) | holds, past the hold radius (the hysteresis); test `mod.rs:462` | `mod.rs:211-228`, `:296-298` |
| M4 spec, 21 (requested ahead, no Late Chunk at 50 m/s) | holds; tests `mod.rs:525`, `:567`, `:574` | `mod.rs:178-181`, `:230-251` |
| M4 spec, 22 (a Late Chunk counted in Frame stats) | holds; tests `mod.rs:545`, `:686` | `mod.rs:255-263`; `io.rs:994` |
| M4 spec, 23 (reads and decoding on an IO pool) | holds | `io.rs:285-355`, `:437-572` |
| M4 spec, 24 (bytes into the staging belt capped, the rest waiting) | holds | `io.rs:1313-1325`, `:1139-1144` |
| M4 spec, 25 (fast travel holds until the Load radius is loaded) | holds; every frame of the hold within Budget is a bench measure (tooling); test `mod.rs:582` | `mod.rs:272`; `io.rs:993` |
| M4 spec, 26 (time from a fast travel until its Chunks load, measured) | holds; the ceiling falls in tooling | `io.rs:675-709` |
| M4 spec, 29 (each pool entry counted by its users) | holds, streaming side: one user per Chunk, given back on unload or a dropped load | `chunks.rs:155-196`; `io.rs:1197-1209`, `:1219-1226` |
| M4 spec, 31 (when an unused entry is freed) | falls in assets | |
| M4 spec, 35 (mips by priority from distance and texel density against the 1440p footprint) | holds | `mips.rs:21-44`, `:305-330` |
| M4 spec, 36 (ray Materials given a floor) | falls in assets (the tail) | |
| M4 spec, 37 (a Texture moved to its new class by copy) | falls in assets; triggered from `io.rs:1155-1162` | |
| M4 spec, 38 (eviction: lowest priority first, LRU ties, evicted only below load priority minus one) | holds; test `mips.rs:454` | `mips.rs:286-300` |
| M4 spec, 39 (late mips counted) | holds; tests `mips.rs:587`, `:708`, `:756` | `mips.rs:209`; `io.rs:1120` |
| M4 spec, 47 (a request past any capacity refused, logged and counted in Frame stats) | broken for a mip move the Channel refuses: L1-F02 | `io.rs:1155-1162` |
| M4 spec, 54 (lights stream with their Chunks) | holds | `chunks.rs:95-106` |
| M4 spec, 64 (a dev launch cooks a Chunk on a miss) | holds | `io.rs:1524-1529`, `:825-847` |
| M4 spec, 69 (Frame stats count Late Chunks, late mips, loads, unloads, bytes read, entries freed) | holds | `io.rs:645-673`, `:919-927`, `:993-997` |
| M4 spec, implementation decision "The Chunk streamer": camera position and teleport each frame, the cooked Chunk index, Load radius and hysteresis, requests ahead, Late Chunks, the hold, Settled, a narrow IO interface, no GPU state | holds | `mod.rs:22-32`, `:118-145`, `:162-292` |
| M4 spec, implementation decision "The IO pool" | holds | `io.rs:285-355`, `:1313-1325` |
| M4 spec, testing seam 4 (the Chunk streamer without a GPU: requests, loads and unloads against the radius and hysteresis; 0 Late Chunks at 50 m/s and an exact count when delayed; the hold; Settled; last users in order) | holds: each item has its test | `mod.rs:462`, `:525`, `:545`, `:582`, `:634`, `:650` |
| M4 spec, 1-16, 27-28, 30, 32-34, 40-46, 48-53, 55-63, 65-68, 70-71 | fall in cook, assets, scenes, render or tooling | |
| M5 spec, 18 (every Chunk outside the Load radius and within the View distance drawn as its Far proxy) | holds, streaming side; the draw falls in render | `io.rs:1005-1022` |
| M5 spec, 19 (swap at the Load radius, whole) | holds | `io.rs:1006-1010`, `:1210-1215` |
| M5 spec, 20 (a Late Chunk keeps its Far proxy) | holds | `io.rs:1008-1009` |
| M5 spec, 21 (flat ground past the View distance) | falls in scenes and render | |
| M5 spec, 22 (requested within the View distance, released an edge past it, nearest first after the Load radius's Chunks, at most 285 Resident) | holds; test `mod.rs:706` | `mod.rs:149-157`, `:274-282`, `:296-298`; `io.rs:1005-1012` |
| M5 spec, 23 (a Late Far proxy counted) | holds; test `mod.rs:613` | `io.rs:1013` |
| M5 spec, 24 (loads capped by count as well as bytes) | holds; the 16,667 µs is a bench measure | `io.rs:765`, `:1317-1319` |
| M5 spec, 25 (Far proxies' Materials counted apart) | falls in assets; the streamer's test holds Resident plus in-flight proxies to that count | `mod.rs:706-737` |
| M5 spec, 27, 28 (Far proxies' shadows, swap traced within N frames) | fall in render; the proxy is placed only once its BLAS is ready | `io.rs:1508-1511` |
| M5 spec, 30 (a Fast travel's hold waits for the Load radius alone; the far field fills after it, nearest first) | holds; the 50 ms is a bench measure (see *Notes* on queued reads) | `io.rs:993`, `:200-213`; `mod.rs:273` |
| M5 spec, 31 (the far field's fill after a Fast travel measured) | holds; test `io.rs:1967` | `io.rs:675-709` |
| M5 spec, 32 (traversal reports Late Far proxies) | holds, streaming side; the report falls in tooling | `io.rs:648`, `:1013` |
| M5 spec, implementation decision "Requested within the View distance, released past it plus a Chunk's edge. `FrameIo::fits` gains a load-count cap" | holds | `mod.rs:296-298`; `io.rs:1317-1319` |
| M5 spec, testing decisions "The streamer's Far proxy requests are tested without a GPU" and "Settled waits for the Far proxies within the View distance" | holds | `mod.rs:613`, `:706`; `io.rs:1921`, `:1015` |
| M5 spec, 1-17, 26, 29, 33-42 | fall in cook, render, scenes or tooling | |

#### Registry keys

| File | Keys | Verdict |
|---|---|---|
| `mips.rs` | `vulkan-anisotropy-lod` (`:40`) | holds: with η = ρmax/ρmin unclamped, λ = log₂(ρmax/η) = log₂ ρmin, the footprint's minor axis, which a face-on surface's texels per pixel bounds, and trilinear's finest mip is ⌊λ⌋ (`mips.rs:41-44`). The quote's "An implementation may use the value of N [= ⌈η⌉] as an approximation of η" is excluded by the comment's own condition; see *Notes* |
| `mips.rs` | `ue-5.8-texture-streaming-hidden-prefetch` (`:58`) | holds as a cited deviation: the quote prestreams hidden components "with one fewer mips"; `Sight::Ahead` requests the full need (`mips.rs:320-323`), which the `deviations.md` row records, and the row's text matches the code |
| `io.rs`, `chunks.rs`, `mod.rs` | none | |

#### Oracles

No function in `src/streaming/` is tested against a literature number. The
one numeric test of a formula, `mips.rs:438-448`, checks the code's own
closed form: at 45° and 1,440 pixels the corner pixel spans
2 tan(22.5°)/1440 = 0.000575296 m per metre over 1 + tan²θ = 1.713834, so
2,979.05 texels per metre at 1 m; the test's 1.7138 / 0.0005753 = 2,978.97,
0.003% low, against the test's own ±1% margins (1.99 and 2.01, 0.99 and
1.01).

#### Open issues read

| Issue | What the code shows | Needs hardware |
|---|---|---|
| The texture pool's Channels are capped in Texture classes, not bytes (ADR 0017's capacity clause, broken, known) | `MipPriority::plan` asks the Material pool for room only by class: `full_class_for_mips` (`src/assets/material.rs:1143-1154`) answers from `ClassTable::has_room`, which counts classes against the Channel's class capacity (`:231-245`), and when the class is full the plan finds room only by an evictable Texture of that very class (`mips.rs:233-242`). A move that opens a new class, a whole texture array of that shape, is requested whenever the class count allows, whatever the class's bytes; no byte figure enters the plan. Nothing in `src/streaming/` adds a byte cap or should; the cap belongs to the pool. Nothing filed. | No |

### the frame: State, the compute pool, components, transforms and the feature modules

Scope: read in full at `e5c55a8d`, tests included: `src/state.rs` (1,673
lines), `src/threading.rs` (39), `src/components.rs` (97), `src/lib.rs`
(280), `src/transform/mod.rs` (387) and `src/time/mod.rs` (283); 2,759
lines. `src/camera/mod.rs` (185) read in full, since its three in-scope
hunks sit in the registry keys' functions; `src/environment/mod.rs`'s one
in-scope hunk. `src/input/` and `src/helmet/` have no in-scope change;
`src/helmet/mod.rs:20-43` was opened because `spawn_helmet` runs in the
fixed schedule and reads the camera. Opened only as far as these files
deliver to or consume them: `src/scenes/mod.rs:859-934` (orrery, `Spin`,
`spin`, `WrapsAroundCamera`, `wrap_around_camera`), `:1265-1349` (crowd
figures, bones, `sway`), `:1619-1690` (the wrap test); `src/scenes/town.rs:247-273`,
`:783-857` (flocks); `src/scenes/world.rs:26-80`, `:240-277` (`spawn_world`);
`src/streaming/io.rs:887-1047` (`ChunkStreaming::update`, its error paths,
`Camera::teleported`'s clearing); `src/render/prepare.rs:19`, `:489-520`
(the compute pool's owner); `src/render/renderer.rs:658-700`;
`src/render/staging.rs:356-365`; `src/stats/mod.rs:267`, `:291-298`,
`:343-361`; `src/capture/mod.rs:340-422` (how capture drives `State`).
Crate sources opened: glam 0.34.3 `src/camera/camera_impl.rs:206-237`,
`src/camera/rh/proj.rs:138-143`, `:180-244`, `src/camera/rh/view.rs:27-34`,
`src/f64/dquat.rs:687-711`; bevy_ecs 0.20.1
`src/schedule/executor/multi_threaded.rs:265-280`, `:655-670`; bevy_tasks
0.20.1 `src/task_pool.rs:425-470`. ADRs read whole, amendments included:
0004, 0006, 0009, 0010, 0012, 0014, 0015, 0016, 0022, and the Decision of
0026; 0013, 0017 and 0023 for the clauses these files govern. Both specs read
whole (M4: Streamed world; M5: Far field). `docs/audit/deviations.md` has no
row on these files; `scripts/cycles-reference/DIFFERENCES.md` describes no
image these files decide. The last audit's L1-R03 (methods on components) is
closed by ADR 0006's 2026-11-04 amendment and its L1-C07
(`PointLight::casts_shadow`) by the field's removal in scope
(`git diff 8bc34eb1..e5c55a8d -- src/components.rs`); neither is filed again.
Not checked: anything that needs a run (no `cargo`), the GPU-side
behaviour the State tests drive (render and streaming sections), and the
Chunk streamer's internals (the other reader's).

#### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0006 | "Components hold data only: no closures, no `Box<dyn Trait>`, no behaviour methods" | holds: no component in these files holds a closure or trait object; the boxed cook closure is a `ChunkStreaming` field, not a component | `src/components.rs:9-97`; `src/transform/mod.rs:8-9`, `:58-66`; `src/state.rs:229` |
| 0006 | "A method that is a pure function of the component's own data and its arguments ... is allowed: `Transform::to_affine`, `Bend::drawn_curvature` and `WorldTransform::{translation, drawn}`" | holds: `drawn`, rewritten in scope to slerp the rotation, still reads only `self`, `previous` and the weight; the last audit's L1-R03 is closed by this amendment | `src/transform/mod.rs:26-49`; `src/components.rs:34-40`, `:74-76` |
| 0006 | "Behaviour lives in systems or, later, script handlers" | holds: the wrap, spin, sway and propagation are systems | `src/state.rs:403-418` |
| 0009 | "`bevy_ecs` used on its own, not the Bevy engine" | holds: `bevy_ecs` and `bevy_tasks` 0.20.1 are the only `bevy_*` dependencies (`bevy_mikktspace` is the cooker's tangent crate); the lockfile's other `bevy_*` crates are `bevy_ecs`'s own | `Cargo.toml:17-18`; `src/threading.rs:5-7` |
| 0009 | "the engine keeps its own `Transform` ... instead of `bevy_transform`"; "ADR 0014 replaces that newtype" | holds: `Transform` is the engine's, `WorldTransform` its derived f64 form; only `ChildOf`/`Children` come from `bevy_ecs` | `src/components.rs:9-15`; `src/transform/mod.rs:8-9`, `:113-159` |
| 0010 | "single-player games on Windows, Linux and macOS ... nothing in the architecture carries replication, prediction or sandbox constraints" | holds: the frame has one `Input`, one camera, no replication; no `wasm32` cfg anywhere in `src/` | `src/lib.rs:97-127`; `src/state.rs:91-95` |
| 0012 | "Gameplay logic ... lives in Lua through mlua"; "Lua ... never authors per-frame systems" | not applicable: nothing implements it at this commit (no `mlua` in `Cargo.toml`); nothing in these files contradicts it, the per-frame schedule holds `fly_camera` alone | `src/state.rs:395` |
| 0016 | "The main thread runs winit, input, the simulation schedule, prepare and command recording, then submits and presents" | holds: one `redraw` measures, acquires, runs `update` (Ticks, frame schedule, Chunk streaming, prepare, upload), `render`, then `present`, all on the event-loop thread | `src/lib.rs:212-265`; `src/state.rs:274-319`, `:328-334` |
| 0016 | "no stage runs a frame behind another" | holds: `render` draws the Frame data the same frame's `advance` prepared; in a non-settling frame `ChunkStreaming::update` returns `Ok` before any error path, so `advance` never skips prepare there | `src/state.rs:304-312`, `:328-331`; `src/streaming/io.rs:1038-1039` |
| 0016 | "the `multi_threaded` feature is on" | holds | `Cargo.toml:17-18` |
| 0016 | "the compute pool is created with `available_parallelism() − 1` workers (at least 1)" | holds; tested in place of the old `tests/compute_pool.rs` | `src/threading.rs:11-19`; test `:34-38` |
| 0016 | "one pool per State's `FramePreparer`"; "a `TaskPool` that each State's `FramePreparer` owns, not the process-global `ComputeTaskPool`" | holds: `compute_pool()` builds a fresh `TaskPool`, `FramePreparer::new` stores it; no `ComputeTaskPool` or `par_iter` left in `src/` | `src/threading.rs:11-19`; `src/render/prepare.rs:489`, `:520` |
| 0016 | "the schedule stays on the `SingleThreadedExecutor`" | holds for both schedules: the frame schedule and the fixed one are built by `single_threaded_schedule`; tested by thread identity, which tells the executors apart (registry key below) | `src/threading.rs:24-28`; `src/time/mod.rs:69-70`; `src/state.rs:392-393`; test `:440-458` |
| 0016 | "No QoS class is set" | holds: the builder sets a thread count and a name only | `src/threading.rs:15-18` |
| 0016 | "`par_iter` runs on the process-global pool, which the engine no longer creates" | holds: no `ComputeTaskPool::get_or_init` or `par_iter` in `src/` (the old `init_compute_pool` is gone in scope) | `src/threading.rs:1-28`; `src/lib.rs:47-48` |
| 0016 | "Asset reads and decoding run on a separate IO pool ... the main thread copies it into the `StagingBelt`" | falls in streaming; the State hands the IO pool's loader the Renderer's load staging on the main thread | `src/state.rs:190-204` |
| 0016 | "Only the main thread submits, presents or polls the wgpu `Queue` and `Device`" | holds in these files: `present` and `render` (which submits) run in `redraw`; the Renderer's internals fall in render | `src/lib.rs:254-255` |
| 0016 | "`desired_maximum_frame_latency` stays 2" | holds | `src/lib.rs:166`; `src/render/mod.rs:44` |
| 0016 | "Frame data (ADR 0004) is an owned, `Send` value with no borrow into the ECS World" | falls in render; the State passes the owned value from `prepare` to `upload` | `src/state.rs:304-312` |
| 0016 | "Lua (ADR 0012) runs in one state on the main thread" | not applicable: nothing implements it at this commit | |
| 0004 | "The renderer never reads the ECS World mid-frame: a prepare pass builds Frame data ... the renderer consumes only that" | holds in the State: `prepare` takes the World, `upload` and `render` take only Frame data and the pools | `src/state.rs:304-312`, `:328-331` |
| 0004 | "every Frame data must reach the renderer, in order" | holds: each `advance` that prepares also uploads; a settling frame whose load fails returns before `prepare`, so no Frame data is built and lost, and the caller fails the run | `src/state.rs:279-281`, `:304-312`; `src/capture/mod.rs:383-384` |
| 0013, checklist | "systems are added inside `SystemTimer::scope`" | holds: both schedules' systems are added inside `recorder.systems.scope` | `src/state.rs:394-419`; `src/stats/mod.rs:267`, `:361` |
| 0013, checklist | "per-frame GPU writes add to `uploaded_bytes`" | holds: the Chunk streamer's staged bytes are added to what `upload` adds; `LoadStaging::submit` returns moved bytes only, so nothing counts twice | `src/state.rs:287`; `src/render/renderer.rs:698`; `src/render/staging.rs:356-365` |
| 0014 | "World positions are f64 ... `Transform` holds translation as `DVec3` ... rotation (`Quat`) and scale (`Vec3`) in f32, 64 bytes" | holds, asserted at compile time | `src/components.rs:9-18` |
| 0014 | the camera holds an f64 position; "shaders compose positions relative to the camera" | holds on the camera side: `position` is a `DVec3`, `view_matrix` is the rotation alone at the origin | `src/camera/mod.rs:20-21`, `:84-86` |
| 0014 | "A child's Transform composes with its ancestors' into a World transform in f64, recomputed only when it or an ancestor moved" | holds; unchanged in scope, tested | `src/transform/mod.rs:113-159`; tests `:220-237`, `:240-257` |
| 0014 | "Within 100 km ... simulated positions resolve to 0.1 mm" | holds for the blend: the rewritten `drawn` keeps translation linear in f64; the orbit test checks 10 µm at 100 km | `src/transform/mod.rs:31-32`; `src/state.rs:461`, `:541-554` |
| 0014 | a scene moved by a shift draws alike (the State's half): roots, the camera and the Far ground move by the shift, Chunks by the streamer | holds: roots only, so children stay parent-relative; `FarGround` shifted in scope; tested at 10,240 m and at the world's half size | `src/state.rs:94`, `:121-130`, `:190-196`; tests `:666-752`, `:1131-1175` |
| 0015 | "Depth is cleared to 0 and compared with `Greater`, in `Depth32Float`, with no far plane"; "`projection_matrix` in `src/camera/` is the one projection" | holds: the in-scope switch to `camera::rh::proj::directx::perspective_infinite_reverse` builds the same matrix (column 2 `(0, 0, 0, −1)`, column 3 `(0, 0, near, 0)`), so depth is `near / distance`, 1 at the near plane, towards 0 at infinity; tested | `src/camera/mod.rs:91-105`; test `:164-184` |
| 0017 | "the engine caps the bytes written into it per frame at the staging Budget, and an upload past the cap waits for a later frame" | falls in streaming; the State passes the cap from the Renderer's capacities and a test drives it at 3,000 bytes | `src/state.rs:179`, `:194`; test `:1180-1244` |
| 0022 | "Simulation advances in fixed Ticks of 1/64 s (15.625 ms)" | holds | `src/time/mod.rs:21`, `:24` |
| 0022 | "runs the fixed schedule once per Tick, at most 4 per frame, with ambiguity detection at `Error`" | holds: the cap, and the build fails at spawn, before any frame, on an unordered conflicting pair | `src/time/mod.rs:27`, `:53-63`, `:69-76`, `:80-87`; `src/state.rs:136-138`; test `src/time/mod.rs:113-125` |
| 0022 | "the Tick counter and blend weight α in `FixedTime`, and the dropped time in frame stats" | holds | `src/time/mod.rs:39-47`; `src/state.rs:276` |
| 0022 | "the helmet spawn and despawn run in it" | holds | `src/state.rs:407-409` |
| 0022 | "World transforms propagate at the end of every Tick" | holds, and once at spawn so frames before the first Tick draw composed transforms | `src/state.rs:131-135`, `:415` |
| 0022 | "an entity the fixed schedule moves keeps its previous Tick's World transform in `PreviousWorldTransform`" (children included) | holds: every entity a fixed-schedule system moves carries it: orrery bodies, flock centres and birds (`town` and `world` share `spawn_flocks`), crowd bones and refit figures; `sway` changes a proxies-form figure's `Bend` only, kept by `record_previous_curvatures` | `src/state.rs:405-406`; `src/scenes/mod.rs:873-878`, `:1276-1301`; `src/scenes/town.rs:837-853`; `src/scenes/world.rs:271-277` |
| 0022 | "a spawned entity, or one marked `Teleported`, is drawn at its current World transform until the next Tick" | holds; a production system now inserts the marker (`wrap_around_camera`), which the last audit found none did; tested | `src/transform/mod.rs:26-29`, `:89-103`; `src/scenes/mod.rs:931`; tests `src/state.rs:635-658`, `src/scenes/mod.rs:1619-1690` |
| 0022 | "The frame delta is measured with `Instant` at frame start, before surface acquire ... and clamped to 250 ms" | holds | `src/lib.rs:220-228`; `src/time/mod.rs:18`, `:33-35` |
| 0022 | "The fixed loop runs at most 4 Ticks per frame; accumulated time past the cap is dropped, the game slows, and the dropped time is reported in frame stats" | holds; tested at 250 ms and 100 ms | `src/time/mod.rs:53-63`; `src/state.rs:276`; tests `src/time/mod.rs:199-223` |
| 0022 | "Simulated time is an integer Tick counter; systems read a per-Tick delta derived from it" | holds: `spin` and `sway` read `(tick + 1) * TICK_SECONDS`; `Time::delta_time` is read by `fly_camera` alone | `src/time/mod.rs:42`; `src/scenes/mod.rs:904-910`, `:1336`; `src/camera/mod.rs:110` |
| 0022 | "an accumulator resource, a fixed schedule run once per Tick, and the blend weight α as a resource" | holds | `src/time/mod.rs:39-47`, `:80-87` |
| 0022 | "Everything that changes game state runs in the fixed schedule ... Input sampling, the camera, the blend, prepare and presentation-only effects run once per frame" | holds for placement: the fixed chain holds every system that moves an entity, the frame schedule `fly_camera` alone; but the fixed chain now reads the camera every Tick, next rows | `src/state.rs:395`, `:403-418` |
| 0022 | "The camera's rotation reads the newest input every frame, outside the fixed loop ...; its position is moved by input every frame too" | holds: mouse motion writes yaw and pitch as it arrives, `fly_camera` moves the position in the frame schedule | `src/lib.rs:120-127`; `src/camera/mod.rs:107-130`; `src/state.rs:395` |
| 0022 | "Entities the fixed schedule moves keep their previous Tick's World transform; prepare blends previous and current in f64 ... Static Instances store nothing extra" | holds on my side: `drawn` blends in f64, slerping the rotation since scope; static Chunk Placements are not entities; prepare's use falls in render | `src/transform/mod.rs:26-49` |
| 0022 | "Determinism: ... fed the same per-Tick inputs, reaches a bit-identical simulation state whatever the frame timing" | broken for `world`'s flocks: `wrap_around_camera` runs in the fixed schedule and reads the camera, which the frame schedule moves by the wall-clock frame delta: L1-F41 | `src/state.rs:410`; `src/scenes/mod.rs:923`, `:929-932`; `src/camera/mod.rs:110` |
| 0022 | "Inside the fixed schedule, bevy_ecs ambiguity detection is `Error`, no system reads the wall clock, randomness comes from a seeded resource advanced per Tick, and no result depends on hash-map iteration order" | holds in letter: no fixed-schedule system calls `Instant`, none draws randomness, the chain is single-threaded; L1-F41 reads wall-clock-driven state, not the clock | `src/state.rs:403-418`; `src/time/mod.rs:69-76` |
| 0022 | "The blend reads two simulation states, never screen-space history" | holds | `src/transform/mod.rs:26-49` |
| 0022 | "Budgets stay judged at 60 Hz ...; the step model must also stay judder-free at refresh rates that do not divide 64 Hz" | holds in form: the State's tests cycle 144, 60, 30 and 90 Hz frames; judder is a visual check | `src/state.rs:521-539` |
| 0022 | "Input latency ... budget splits", "A physics engine chosen in M8 steps once per Tick" | falls in tooling (not measured); deferred to M8 | |
| 0023 | "Exposure is an EV100, which scales luminance by 1/(1.2 · 2^EV100)" | holds; unchanged in scope | `src/camera/mod.rs:7-16`, `:52-54` |
| 0026 | "past the View distance only flat ground is drawn" | falls in render and streaming; the State moves the Far ground with the scene | `src/state.rs:126-130` |

#### Spec stories

| Story | Verdict | Where |
|---|---|---|
| M4 spec, 16, every Reference scene's static content loads through Chunks | holds in the State: the spawn dispatch builds no static content (six scenes spawn nothing, `texture_wall` loads Textures only, the rest spawn Moving Instances); point lights no longer spawn from the State | `src/state.rs:99-116`; `src/scenes/mod.rs:1072-1084` |
| M4 spec, 17, Moving Instances stay spawned by scene systems | holds: orrery, crowd, `town`'s and `world`'s flocks, and the sun | `src/state.rs:107-114`, `:120` |
| M4 spec, 18, captures and Goldens render Settled frames | holds on the State's side: `update_settled` waits and fails on a failed load; capture's use falls in tooling | `src/state.rs:267-272`; `src/capture/mod.rs:383-384` |
| M4 spec, 19 and 20, Chunks within the Load radius load, those outside unload | falls in streaming; the State's test flies past a Chunk and checks load, hold and unload | test `src/state.rs:754-833` |
| M4 spec, 22, a Late Chunk counted in Frame stats | holds: copied per frame; tested (2 Late Chunks in the flight) | `src/state.rs:282`; test `:830` |
| M4 spec, 23 and 24, IO off the main thread; bytes staged per frame capped | falls in streaming; the cap reaches the streamer from the Renderer's capacities, tested | `src/state.rs:179`, `:194`; test `:1180-1244` |
| M4 spec, 25 and 26, Fast travel holds; its time measured | falls in streaming; the State forwards `holding` and `fast_travel_hold` | `src/state.rs:293-294` |
| M4 spec, 44, GPU memory counted by kind | holds on the State's side: `frame_stats` reports the per-kind ledger and its sum | `src/state.rs:368-377` |
| M4 spec, 47, a request past any capacity refused, logged and counted in Frame stats | holds on the State's side: every refusal counter is copied per frame | `src/state.rs:289-291`, `:313-317` |
| M4 spec, 51 and 53, frames before a loaded Chunk is traced kept to a ceiling; BLAS waits on streaming frames | falls in render; the State's test places a Chunk only once its BLAS is ready, sees no compaction wait, and traces it within 10 frames | test `src/state.rs:1308-1393` |
| M4 spec, 60, 10,000 Moving Instances around the camera | holds in placement: `spawn_world` lays out 100 flocks of 100 birds and the fixed schedule wraps them about the camera; determinism of that wrap is L1-F41 | `src/state.rs:114`, `:410`; `src/scenes/world.rs:240-277`; `src/scenes/town.rs:249-253` |
| M4 spec, 64, a dev launch cooks a Chunk and its Assets when its load misses | holds: the spawn loads through `CookingLoader`, whose cook is then handed to the streamer for each Chunk | `src/state.rs:205-230` |
| M4 spec, 65, the same Chunk content at the world's origin and at its far corner gives identical images | holds: two State tests compare bit for bit, one Chunk 10,240 m out and the whole `world` moved by its half size | tests `src/state.rs:660-752`, `:1131-1175` |
| M4 spec, 69, Frame stats count Late Chunks, late mips, Chunks loaded and unloaded, bytes read, pool entries freed | holds: all copied per frame | `src/state.rs:282-292` |
| M4 spec, Implementation decision, the Chunk streamer "Each frame ... takes the camera's World position and whether the camera teleported" | holds: `Camera::teleported`, set by whoever jumps the camera and cleared by the streamer | `src/camera/mod.rs:29-31`; `src/streaming/io.rs:915` |
| M4 spec, Implementation decision, "the bench never waits" | holds: `update` never settles | `src/state.rs:261-265` |
| M4 spec, Testing decision, prior art: "The compute pool test, which checks the pool leaves the main thread a core" | holds | test `src/threading.rs:34-38` |
| M5 spec, 18 and 20, Chunks outside the Load radius drawn as Far proxies; a Late Chunk keeps its Far proxy | falls in streaming and render; the State's test checks both | test `src/state.rs:926-998` |
| M5 spec, 21, flat ground to the horizon | falls in render; the State shifts the Far ground with the scene | `src/state.rs:126-130` |
| M5 spec, 23, a Late Far proxy counted in Frame stats | holds | `src/state.rs:283` |
| M5 spec, 27 and 28, Far proxies cast and receive the sun's shadows; a swap traced within 10 frames | falls in render; the State's test checks both | test `src/state.rs:1000-1129` |
| M5 spec, 30 and 31, the Fast travel's hold waits for the Load radius alone; the far field's fill measured | falls in streaming; the State forwards `far_field_filling` and `far_field_fill` | `src/state.rs:295-296` |
| M5 spec, Testing decision, "Captures and Goldens render Settled frames; Settled waits for the Far proxies within the View distance" | falls in streaming; the State passes `settle` through | `src/state.rs:274-281`; `src/streaming/io.rs:1015` |
| M5 spec, other stories (LOD, thin features, Far proxy cook, bench) | fall in render, shaders, cook and tooling; nothing in these files implements them | |

#### Registry keys

| File | Keys | Verdict |
|---|---|---|
| `src/camera/mod.rs:10`, `:51`, `:138` | `frostbite-2014-exposure` | matches: the quote gives `maxLuminance = 1.2f * pow(2.0f, EV100); return 1.0f / maxLuminance;` and 78 / (100 · 0.65) = 1.2; `exposure` is `1 / (1.2 · 2^EV100)` (`:52-54`), `ev100_for_exposure` its inverse (`:14-16`), the constant 1.2 (`:11`) |
| `src/camera/mod.rs:98` | `glam-0.34.3-perspective-infinite-reverse` | matches: the code calls `camera::rh::proj::directx::perspective_infinite_reverse`, which is `camera_impl::perspective_infinite_reverse::<true, false>` (`glam-0.34.3/src/camera/rh/proj.rs:242-243`), RH true and YFLIP false as the key's section says; the quoted columns give clip z = near and clip w = −z_view, so depth = near / distance, as the doc comment says |
| `src/state.rs:453` | `bevy-ecs-0.20.1-multi-threaded-executor-spawns-on-the-pool` | matches: `multi_threaded.rs:667-668` spawn a `Send` system with `context.scope.spawn(task)` on the global pool's scope, opened with `tick_task_pool_executor` false (`:274-275`), and `task_pool.rs:436` ticks the pool's executor on the caller only when asked or when the pool has no thread; so a system recorded on the caller's thread proves the single-threaded executor, as the test asserts |

#### Oracles

No function in these files under a literature oracle changed in scope: the
exposure pair and its test (`src/camera/mod.rs:14-16`, `:52-54`, `:141-158`)
are unchanged. Two functions changed in scope have analytic tests, not
literature numbers:

| Function | Test, constant | Reference | Measured error | Verdict |
|---|---|---|---|---|
| `projection_matrix` (`src/camera/mod.rs:99-105`), new glam path | `reversed_depth_is_near_over_distance_with_no_far_plane`, `4.0 * f32::EPSILON * expected` relative (`:173`) | ADR 0015's `near / distance` | by analysis, not run: clip z is `near` exactly and clip w is the f32 distance exactly (products with 0 and −1), so the only rounding is the f32 divide, at most 2⁻²⁴ relative, an eighth of the tolerance's 2⁻²¹ | holds |
| `WorldTransform::drawn` (`src/transform/mod.rs:26-49`), now slerping | `a_spinning_entity_is_drawn_at_its_size_and_at_the_rotation_between_its_ticks`, columns within 1e-12 (`:357`, `:362`) | rotation about +Y by the blended angle | by analysis: f64 slerp of two unit quaternions 0.195 rad apart; its comment's figures check: 12.5 rad/s is the sum of `ORRERY_SPIN_RATES` (`src/scenes/mod.rs:859`), 12.5/64 = 0.1953 rad a Tick, and 1 − cos(0.0977) = 0.48% | holds |

### Findings

| id | claim | severity | where | evidence | expected | verifier | placement |
|---|---|---|---|---|---|---|---|
| L1-F01 | `MipPriority::plan` checks each request's room against the Channel as it stands, without counting the room the same plan's earlier requests will take, so N requests into one class with room, free or evictable, for fewer than N moves are all made, and the excess reads are discarded when applied. | `correctness`; clause "a request is made only when its move finds room" broken whenever more requests in one plan target a class than it has room for; cost a whole Texture Blob read per excess request (5.3 MiB for a 2,048² BC7 chain), nothing visible | `mips.rs:233-242` | block below | ADR 0017: "a request is made only when its move finds room, free or by an eviction" | | |
| L1-F02 | A mip move whose Channel has no room when its read is applied is refused by the Material pool and the refusal is dropped: `MipStreaming::apply` ignores `stream_mips`'s `StreamedMips::Refused`, so it is neither logged nor counted in Frame stats. | `correctness`; M4 spec story 47 broken for one kind of refusal, mip moves; the other refusals (Meshes, Materials, Chunk loads) are logged and counted | `io.rs:1155-1162`; `src/assets/material.rs:1061-1063` | block below | M4 spec, story 47: "a request past any capacity refused, logged and counted in Frame stats, so that an overflow is visible, never silent" | | |
| L1-F03 | An Asset the main thread read from the Cooked cache after a failed read or a refused load left it claimed stays in `read_from_cache` forever, since no decoded copy of it ever comes; once the pools free it and a later IO read decodes it afresh, that copy is dropped and the main thread reads the Blob itself. | `correctness`; clause "the main thread reads one only after a failed read or a refused load left it claimed" broken once per such Asset after its first refusal or failed read; cost one main-thread Blob read and a needless wait | `io.rs:54-70`, `:597-604`, `:636-641` | block below | ADR 0017: "so the main thread reads one only after a failed read or a refused load left it claimed" | | |
| L1-F41 | The fixed schedule runs `wrap_around_camera`, which reads the camera that the per-frame schedule moves by the wall-clock frame delta, so the Tick on which a `world` flock jumps a 320 m span, and so the simulation state after that Tick, depends on frame timing. | `correctness`; clause broken: ADR 0022's determinism ("whatever the frame timing"); 100 flock centres and their 10,000 birds in `world`, in the window only | `src/state.rs:410`; system at `src/scenes/mod.rs:921-934` | block below | ADR 0022: "the same binary on the same machine, fed the same per-Tick inputs, reaches a bit-identical simulation state whatever the frame timing"; "Input sampling, the camera, the blend, prepare and presentation-only effects run once per frame" | | |

#### L1-F01

Code, `mips.rs:233-242`:

```rust
let mut evictable_needs = None;
self.requests.retain(|request| {
    let needed = request.levels.start;
    let Some(class) = materials.full_class_for_mips(request.texture, needed) else {
        return true;
    };
    let needs = evictable_needs
        .get_or_insert_with(|| materials.coarsest_evictable_needs(&eviction_rank));
    needs.get(&class).is_some_and(|&coarsest| coarsest > needed)
});
```

`full_class_for_mips` (`src/assets/material.rs:1143-1154`) returns `None`,
so the request is kept, whenever `ClassTable::has_room(&[shape])` holds for
the one shape (`:231-245`): one free slice, or one class to spare, admits
every request of that shape in the plan. When the class is full, the map of
coarsest evictable needs is built once per plan and never decremented, so one
evictable Texture admits every request of that class whose need is finer than
its own. ADR 0017, third Consequence (amended 2026-11-08): "a request is made
only when its move finds room, free or by an eviction". At apply, the first
move takes the room; each later one finds `has_room` false, evicts another
Texture if one is evictable, and otherwise is refused
(`src/assets/material.rs:1047-1063`), its read discarded (L1-F02). The plan's
next pass sees the room gone and does not request it again, so the waste is
bounded by one read per excess request, but each read is a whole Texture
Blob (`io.rs:564-572`, `src/assets/cooked.rs:1138-1152`). Textures of one
shape needing the same move arrive together whenever a Chunk loads, so the
case needs only a full Channel, which `world`'s eviction regime is built to
reach. Not `budget`: no measurement shows a Budget passed. Not `drift`: the
doc comment at `mips.rs:128-132` states the clause the code breaks. A test
seam: a plan with two Textures of one shape, room for one move, and the
check that one request is made.

#### L1-F02

Code, `io.rs:1151-1164`:

```rust
let priority = &*self.priority;
let eviction_rank = |victim, resident_level| {
    priority.eviction_rank(victim, resident_level, levels.start)
};
material_pool.stream_mips(
    device,
    load_staging,
    texture,
    levels.start,
    &decoded.levels,
    &eviction_rank,
);
streamed.written_bytes += load_staging.written_bytes() - written_before;
false
```

`stream_mips` returns `StreamedMips` (`src/assets/material.rs:1032-1039`),
whose `Refused` means "Its Channel had no room for that class"
(`:289-290`), returned at `:1061-1063` without a log or a count:

```rust
let Some(target) = channel.place(device, target_shape) else {
    return StreamedMips::Refused;
};
```

This call is its only caller outside the pool's tests, and the value is
dropped; `priority.finished(texture, false)` then ends the request as if it
had moved. The Frame stats count Materials refused through
`take_refusals` (`io.rs:925`), which `stream_mips` never increments
(`src/assets/material.rs:679-683` is the only increment). M4 spec, story 47:
"As the owner, I want a request past any capacity refused, logged and
counted in Frame stats, so that an overflow is visible, never silent." A
Channel's class capacity is a capacity, and L1-F01 shows the refusal is
reachable. The defect sits at the call that discards the outcome; where the
log and the count go (here or in the pool) is the fix's choice. Not
`image`: a refused move leaves the Texture at its Resident mips, counted as
late mips if drawn, which no capture here shows.

#### L1-F03

Code, `io.rs:54-56` and `:61-63`:

```rust
/// Assets a load in flight claimed that the main thread read from the
/// Cooked cache first, whose decoded copies are dropped when they come.
read_from_cache: HashSet<AssetId>,
```

```rust
let read_from_cache = &mut self.read_from_cache;
let mut wanted = |id: &AssetId| !read_from_cache.remove(id);
```

and the insert, `io.rs:597-604`:

```rust
let newly_claimed = self
    .claimed
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .insert(id);
if !newly_claimed && !decoded_on_the_io_pool {
    self.decoded.borrow_mut().read_from_cache.insert(id);
}
```

`extend` is the only place an entry leaves (`grep read_from_cache` finds
`:56`, `:62-63`, `:603` and two tests), and `forget` (`:636-641`) clears the
claim but not the entry. The two cases ADR 0017 names as the main thread's
licence to read are exactly those where no copy comes: a read that failed
after claiming the Asset returns `Err`, whose assets `take_finished` skips
(`io.rs:1537-1539`, `.flatten()`), and a refused load consumed the decoded
copy (`io.rs:611-616` remove it from `decoded`) while the claim stays. In
both, the next Chunk that needs the Asset reads it on the main thread and
inserts an entry that nothing will remove. Sequence: Chunk A's read claims
Texture T and fails on a later Asset; Chunk B, waiting on A
(`io.rs:1395-1416`), reads T on the main thread, entry inserted; T loads,
its users leave, the pools free it and `forget` drops the claim
(`src/assets/material.rs:769`); Chunk C's IO read claims T afresh and
decodes it; `extend` finds the stale entry and drops C's copy; C's load then
lacks T (`io.rs:1407-1415`), waits on any earlier reads in flight, and reads
T's Blob on the main thread (`io.rs:615-625`). ADR 0017, sixth Consequence
(amended 2026-11-12): "a Chunk that needs an Asset a Chunk read in flight
claimed waits for the reads in flight when its own came back instead, so
the main thread reads one only after a failed read or a refused load left
it claimed". C's read neither failed nor was refused. The doc comment at
`:54-55` says the set holds Assets "a load in flight claimed", which a
failed or refused load is not. Not `budget`: one Blob read per stale entry,
reachable only after a failure or a refusal. A test seam: the
`DecodedReader` tests at `io.rs:2127` and `:2161`, with a claim and no copy
coming, then `forget`, then a decoded copy that must be kept.

#### L1-F41

The fixed chain, built once for every State:

```rust
// src/state.rs:403-418
        fixed_schedule.add_systems(
            (
                record_previous_world_transforms,
                record_previous_curvatures,
                (spawn_helmet, despawn_helmet)
                    .chain()
                    .run_if(resource_exists::<HelmetAssets>),
                wrap_around_camera.run_if(any_with_component::<WrapsAroundCamera>),
                spin,
                ...
```

The system reads the camera's position each Tick and moves the flock by
whole spans from it:

```rust
// src/scenes/mod.rs:921-932
pub(crate) fn wrap_around_camera(
    mut commands: Commands,
    camera: Res<Camera>,
    mut wrapping: Query<(Entity, &WrapsAroundCamera, &mut Transform)>,
) {
    for (entity, wraps, mut transform) in &mut wrapping {
        let spans = ((camera.position - transform.translation) / wraps.span_metres).round();
        let jump = glam::dvec3(spans.x, 0.0, spans.z) * wraps.span_metres;
        if jump != glam::DVec3::ZERO {
            transform.translation += jump;
            commands.entity(entity).insert(Teleported);
```

The camera's position is per-frame state driven by the wall clock: the
window measures the frame delta (`src/lib.rs:222-225`), `simulate` writes it
to `Time` after the Ticks and then runs the frame schedule
(`src/state.rs:150-152`), and `fly_camera` moves the camera by it:

```rust
// src/camera/mod.rs:110
    let speed = f64::from(camera.speed * time.delta_time);
```

ADR 0022 (`docs/adr/0022-fixed-64-hz-simulation-with-interpolation.md:20`):
"Determinism: the same binary on the same machine, fed the same per-Tick
inputs, reaches a bit-identical simulation state whatever the frame
timing." And (`:17`): "Everything that changes game state runs in the fixed
schedule ... Input sampling, the camera, the blend, prepare and
presentation-only effects run once per frame."

Reasoning. The Ticks of a frame run before that frame's `fly_camera`, so
Tick k sees the camera where the last frame before it left it. Two runs fed
the same key presses at 60 Hz and at 144 Hz sample the camera's path at
different instants, so at a given Tick the camera has crossed a flock's
half-span boundary (160 m, `src/scenes/world.rs:78-80`;
`FLOCK_PITCH_METRES` 32 × `FLOCKS_PER_SIDE` 10, `src/scenes/town.rs:251`)
in one run and not in the other. That flock centre's `Transform` after Tick
k then differs by 320 m between the runs, its birds' World transforms with
it: simulation state at a Tick depends on frame timing. The camera is not a
per-Tick input in ADR 0022's sense: input reaches the fixed schedule through
`Input`, held until a Tick consumes it (`src/state.rs:400-402`); the camera
is named among what runs once per frame. The wrap itself is a
presentation-only effect by the ADR's list (it keeps decorative Moving
Instances near the viewer, the M4 spec's story 60), which the ADR places
once per frame; it sits in the fixed schedule because the blend's jump
marker, `Teleported`, is only honoured there.

Why `correctness`, not higher or lower: nothing visible follows (a wrap is
drawn as a jump at either Tick), no Budget moves, and captures and benches
stay reproducible because they place the camera per frame index and fix
the frame delta (`src/capture/mod.rs:355-360`); the break needs wall-clock
frames, the window. It is not `drift`: the ADR's clause is the intent and
the code departs from it. Counter-reading for the verifier:
`spawn_helmet` (`src/helmet/mod.rs:23`, `:31`), unchanged in scope, also
reads the camera in the fixed schedule and the last audit passed this
clause; it places a spawn once per key press, which a press already ties to
frame timing, whereas the wrap reads the camera every Tick for every flock.
Either the wrap moves to a per-frame form, the camera's position reaches
the fixed schedule as a per-Tick input, or the owner records the camera as
an accepted input of the fixed schedule (an ADR 0022 amendment or a
`deviations.md` row).

### Notes for the verifier

#### the Chunk streamer, the IO pool and mip priority

- `vulkan-anisotropy-lod`: the comment at `mips.rs:36-40` holds the need
  to implementations that do not round the ratio of anisotropy up; the
  quote allows N = ⌈η⌉ in its place, which gives λ = log₂(ρmax/N), less
  than one level finer than log₂ ρmin, so such hardware can sample up to a
  mip finer than requested on an oblique surface and the frame counts no
  late mip. The nearest point of the bounding sphere usually overstates the
  need by more than that. Not filed: no source says which adapters round,
  and `deviations.md` has no row. A probe on the M4 Pro with distinct-colour
  mips on a plane tilted to a non-integer ratio would settle it.
- Far proxy reads apply the drawn need to every Texture of every Material
  the proxy's Mesh names (`io.rs:476-526`), its ground Primitive's Material
  included, while ADRs 0017 and 0026 name only the atlas. In practice a
  Chunk read claims the ground's Textures first (Chunk reads are taken
  first, `io.rs:206-210`, and claims stand while any proxy holds the ground
  Material), so the ground arrives as its tail. Not filed: no path in
  `world` shown to reach it.
- A Far proxy whose Chunk is drawn is `Sight::Replaced`, needs nothing
  (`mips.rs:167-169`), and so ranks `u32::MAX`, first in line for eviction
  (`mips.rs:293`). Under eviction pressure its atlas can lose the mip it
  needs at the Load radius while replaced, and leaving the Chunk then draws
  it late, against ADR 0026's "one within the Load radius holds the mips it
  needs where it takes over from its Chunk". The code matches the 2026-11-20
  amendment's "needs none"; the two sentences of the ADR pull apart only
  under eviction. A `world` traversal reading `late_mips` on the frames a
  Chunk turns held would settle whether it happens.
- No queued read is ever cancelled. After a Fast travel the old location's
  queued Chunk reads, first in the Chunk lane's order (`io.rs:224`), are
  taken before the new location's, and their loads still enter the pools,
  build BLASes and are then dropped (`mod.rs:201-203`; `io.rs:1197-1199`).
  That lengthens the hold against M5 story 30's 50 ms; a `world` bench
  reading `fast_travel_hold` with the queue's length at the jump would
  settle it (lane 3).
- Both decoders read the whole Texture Blob (`io.rs:447`, `:565`;
  `src/assets/cooked.rs:1146`) to decode a tail or a mip range, though the
  cooked format stores each mip as its own range; `bytes_read` counts the
  whole Blob. No clause says the runtime reads only the range. Lane 3.
- A settling frame runs the streamer and the mip plan once per wait
  (`io.rs:946-1045`): from the second pass on, `camera_before` is the same
  camera, so the mip plan's lead is zero, and the streamer's `frame_number`
  counts passes, not frames, so `lead_frames` (`mod.rs:200`) reads in
  passes. Deterministic, so no Golden moves; not filed.
- An IO thread that panics in a decode never calls `requests.finished`
  nor sends: its lane's taken count stays raised, so no Far proxy read is
  taken again after a Chunk read panics, and a Settled wait blocks in
  `recv` (`io.rs:317-339`, `:394-399`). Whether any Blob parser on that path
  can panic on a malformed Blob would settle it.
- The mips' byte cap sizes a move by its mips alone (`io.rs:1139-1141`),
  while `stream_mips` also writes each sampling Material's Layer through the
  load staging (`src/assets/material.rs:1234`): tens of bytes a Material
  past the cap. Not filed: below any Budget.
- `counted_late` loses an entry only on a successful load (`mod.rs:199`):
  a Chunk counted late whose load then fails, left behind by the camera,
  is not counted again on a later late arrival. Edge case; not filed.
- `mod.rs:1-3` says the streamer decides "from the camera's World
  position"; `ChunkStreaming` passes the camera less the State's shift
  (`io.rs:937`, `:1241`). Not load-bearing: the shift is fixed per State
  and only tests set it.
- The per-Chunk texture uses are cached by Chunk Asset ID for the Chunk's
  life (`mips.rs:163-166`); nothing moves a loaded Chunk after its load
  (`src/state.rs:73`, `io.rs:1504`), so the cached spheres stay true.
  Checked; nothing filed.
- Outside the module, `src/assets/material.rs:1061-1063` returns `Refused`
  without a log; L1-F02 names it beside its caller.

#### the frame

- `src/state.rs:388-390`: the doc comment on `simulation_schedules` reads
  "The State's one schedule: its systems, timed by `recorder`, on the
  single-threaded executor." above "The schedule run once per frame and the
  fixed schedule run once per Tick.", while the function returns two
  schedules. Not filed: the first line predates scope (the fixed schedule
  and this comment both stand at `8bc34eb1`), and the lane files a doc only
  when what landed since made it false. `drift`, not load-bearing, if a
  later lane wants it.
- `src/lib.rs:231-236`: a redraw that acquires `Timeout`, `Occluded` or
  `Validation` has already measured its delta (`:222-226`), so that frame's
  time is dropped without reaching `dropped_time`, which ADR 0022 says
  frame stats report. The comment at `:220-221` says the drop is intended,
  and `lib.rs`'s redraw did not change in scope; not filed. A run with an
  occluded window and a frame-stats dump would show the uncounted time.
- `src/state.rs:261-265`: `update` logs a streaming error and carries on
  to `render`, which then draws the previous frame's Frame data. Not filed:
  `ChunkStreaming::update` returns `Ok` on every non-settling frame
  (`src/streaming/io.rs:1038-1039`), so the branch is unreachable at this
  commit; if a later change lets a non-settling frame fail, ADR 0016's "no
  stage runs a frame behind another" would need the branch to skip the
  render.
- Capture's hold loop (`src/capture/mod.rs:361-392`) calls `update` with
  the scene's frame delta on every held frame, so Ticks keep running while
  the camera path pauses for a Fast travel; and its undrawn frames call
  `simulate` directly (`:357`), whose dropped time is discarded. Both
  tooling's; neither contradicts a clause this reader can quote.
- `src/transform/mod.rs:16-25`: the rewritten `drawn` decomposes each
  matrix as a rotation times a scale and says a shear "would be drawn wrong
  even at a Tick". Every moving hierarchy at this commit has an unscaled or
  uniformly scaled parent (orrery `ORRERY_SCALE`, flock centres and crowd
  figures unscaled), so no shear exists; a future non-uniformly scaled
  moving parent would be drawn wrong at every weight, not only between
  Ticks. Not filed: the comment states it.

### Counts

| Severity class | Findings |
|---|---|
| `image` | 0 |
| `budget` | 0 |
| `correctness` | 4 (L1-F01, L1-F02, L1-F03, L1-F41) |
| `drift` | 0 |
| `health` | 0 |

Clause verdicts: 125 rows over the two readers (73, 52). The streaming
reader's: `holds` 57; `broken` 3 (L1-F01; L1-F03, whose wait half holds;
ADR 0017's capacity clause, known); falls in another module 13. The frame
reader's: `holds` 44; `broken` 1 (L1-F41); falls in another module 4,
deferred to tooling 1; not applicable 2 (ADR 0012's Lua, in ADRs 0012 and
0016). `superseded` 0 in both. Spec stories: 64 rows (40, 24) covering all
71 stories of the M4 spec and all 42 of the M5 spec; in this module, the
M4 spec's story 47 is broken (L1-F02) and its story 60 carries L1-F41; the
rest hold here or fall in cook, assets, scenes, render or tooling.
Registry keys: 5 keys at 7 citations, all match, one
(`ue-5.8-texture-streaming-hidden-prefetch`) as its `deviations.md` row.
Oracles: no function in the module is tested against a literature number;
two analytic tests in the frame's files hold by arithmetic.
