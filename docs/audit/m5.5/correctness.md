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

## render

Scope: `src/render/` outside the shaders and the oracle test modules, and
`src/gpu/`, as far as they changed after `8bc34eb1`: `renderer.rs`
(3,895 lines, 1,874 changed), `prepare.rs` (2,366, 1,545),
`acceleration_structures.rs` (1,861, 1,563), `readback.rs` (1,355, 667),
`staging.rs` (624, 326), `blas_refit.rs` (560, 151),
`shadow_mask_pass.rs` (467, 135), `culling_pass.rs` (490, 267),
`main_pass.rs` (387, 225), `targets.rs` (423, 174), `bind_groups.rs`
(698, 87), `tonemapping_pass.rs` (372, 88), `froxel_pass.rs` (149, 77),
`bend_pass.rs` (303, 74), `depth_pyramid_pass.rs` (421, 35),
`light_count_pass.rs` (122, 28), `uniforms.rs`, `mod.rs`, `instance.rs`
and `src/gpu/mod.rs` (1,000, 139). Four readers at high effort shared the
module by file, each with the lane brief verbatim: the frame's
orchestration (`renderer.rs`, `mod.rs`, `instance.rs`, `uniforms.rs`);
prepare, staging and the GPU stats readback; the ray-tracing files
(`acceleration_structures.rs`, `blas_refit.rs`, `shadow_mask_pass.rs`,
`bend_pass.rs`); and the raster passes with `targets.rs`,
`bind_groups.rs` and `src/gpu/`. Each read the non-test part of its
files in full and their diff since `8bc34eb1` hunk by hunk; test modules
were read where a test is cited. ADRs read whole: 0001, 0003, 0004, 0005,
0007, 0014, 0015, 0016, 0017, 0019, 0020, 0021, 0026 and 0027; read for
the clauses that fall here: 0013, 0018, 0022 and 0023. The `CLAUDE.md`
review checklist was read as the operational form of those clauses. A
WGSL part was opened only to confirm what a Rust-side constant, layout,
binding or format must match; every shader-side clause is marked for the
shaders section. The wgpu, wgpu-hal and wgpu-types 30.0.2 sources that
`Cargo.lock` pins, and the vendored wgpu-core, were read where a registry
quote is load-bearing. Every `where` is `path:line` at the frozen commit,
under `src/render/` unless given in full. `deviations.md` holds no row at
a site in these files; `DIFFERENCES.md` was read for what is not a
Finding. Of the last audit's render Findings, the three that ADR
amendments answered are found fixed (rows marked below), and the slot a
skipped Renderable lost is kept now.

### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0001 | "one bind group serves every material" | holds: one `MATERIALS` group, refreshed once a frame, bound in the depth and lit passes | `renderer.rs:425-431`, `:813`, `:878-882`; `bind_groups.rs:49-66` |
| 0001, 0005 | "indexed by the Material ID that each Meshlet record carries, its Primitive's unless the Instance overrides it"; the Instance slot read from the record | holds, Rust side: a 12-byte record (slot, Meshlet, Material ID) read by the vertex stage; the Instance carries its override or `NO_MATERIAL`; the per-Meshlet pick is the shaders' | `culling_pass.rs:250-255`; `bind_groups.rs:285-296`; `instance.rs:23-30`, `:37`; `prepare.rs:162` |
| 0003 | "one GPU buffer, the Instance buffer, in a slot per Renderable, and are updated incrementally through dirty flags; there are no per-entity uniform buffers" | holds: one `STORAGE` buffer bound in `SCENE_GEOMETRY`; `Changed` filters | `renderer.rs:502-512`, `:541-558`; `prepare.rs:86-93`, `:807-825` |
| 0003 | "each Renderable keeps a slot in the Instance buffer until it stops being a Renderable" | holds: a skipped Renderable keeps its slot, written an Instance with no Mesh once; a slot is freed only for an entity that left the Renderable query (the last audit's slot Finding is fixed) | `prepare.rs:737-744`, `:784-798`, `:425-430` |
| 0003 | "a freed slot is taken again before a fresh one" | holds for a slot a Renderable freed (`free.pop().or(fresh)`); a Renderable never takes a slot of a Chunk's free range: L1-R01 | `prepare.rs:330-331` |
| 0003 | "the buffer holds a fixed number of slots, past which a Renderable is refused, warned about once, counted in Frame stats and retried every frame" | broken where Chunks' free ranges hold slots: a Renderable is refused while the buffer has free slots: L1-R01; warned once, counted and retried hold | `prepare.rs:326-345`, `:752`, `:784-787`, `:818-823`; `src/state.rs:313` |
| 0003 | "sized at device creation from the scene and pass buffers' GPU memory Budget" | holds: created once at `RenderCapacities::instances` slots from the checked-in Budgets, never resized | `renderer.rs:253-261`, `:402`, `:502-512`; `prepare.rs:505-508` |
| 0003 | "30% of it at 136.2 bytes a slot, its Instance, offsets and visibility bit ... 517,376 slots at the Budgets M4 set" | broken: the code takes 144.2 bytes a slot and gives 488,448 slots at the same Budgets: L1-R03 | `renderer.rs:204`, `:215-218`, `:259-261`; test `:1088-1098`; `docs/adr/0003-persistent-scene-transform-buffer.md:3` |
| 0003 | "prepare writes only the slots of new Renderables and of those whose World transform, Mesh or Material changed ... through the per-frame staging path, so camera motion writes no Instance" | holds: the camera is no Instance field and a uniform of its own; Instance writes go through `FrameStaging` only | `prepare.rs:86-93`; `instance.rs:33-38`; `renderer.rs:743-750`, `:765-781`; `uniforms.rs:31-62` |
| 0003 | "A freed slot is written an Instance with no Mesh ... and is taken again from the next frame" | holds: `freed` is written after the visit and joins `free` at the end of `prepare` | `prepare.rs:841-844` |
| 0003, 0014 | "the buffer stores an integer anchor plus an f32 offset per Renderable" | holds | `instance.rs:8-12`, `:36`, `:38`; `prepare.rs:157` |
| 0003, 0020, 0022 | "prepare also rewrites every Moving Instance each frame, changed or not" | holds; the amendment answers the last audit's drift Finding on it | `prepare.rs:91`, `:805` |
| 0003 | "On the frame after the Mesh or Material pool frees an entry, prepare writes again every Renderable's Instance and the Placements of each drawn Chunk one of whose Placements names an entry no longer in its pool ... free ranges, free slots and held Chunks ... are not written again" | holds: free ranges and slots are rewritten only on `rewrite_every_instance`; a held Chunk is skipped | `prepare.rs:670-672`, `:681-685`, `:700-708`, `:836-840`, `:925-937` |
| 0003 | "each loaded Chunk owns a contiguous range of slots, one per Placement, taken on the first frame prepare sees the Chunk, before new Renderables take theirs, and kept until it unloads" | holds | `prepare.rs:351-387`, `:391-402`, `:694-733` before `:734-825` |
| 0003 | "its Placements' Instances are written there in the order the Chunk holds them ... and its point lights join the frame's lights" | holds | `prepare.rs:702`, `:952-970`, `:1020-1024` |
| 0003 | "A Placement whose Handles find nothing, or that lacks a Material Handle its Mesh needs, is written an Instance with no Mesh and ... not retried" | holds | `prepare.rs:952-977` |
| 0003 | "A Chunk the buffer has no room for is refused, warned about once and its Placements counted as refused every frame" | holds | `prepare.rs:371-381`, `:697`, `:713-718`; test `:1310-1354` |
| 0003 | "A range is the first free range long enough, else fresh slots past the highest taken" | holds | `prepare.rs:351-387` |
| 0003 | "an unloaded Chunk's range is written Instances with no Mesh and joins the free ranges the next frame, merged with those it touches" | holds | `prepare.rs:391-402`, `:406-423`, `:677`, `:686-693`; test `:1361-1420` |
| 0003 | "A held Chunk keeps its range, written Instances with no Mesh until it is drawn again, and its point lights stay out of the frame's" | holds | `prepare.rs:700-728`, `:1022` |
| 0004 | "The renderer never reads the ECS World mid-frame: a prepare pass builds Frame data ... it lists no draws" | holds: `renderer.rs` imports `bevy_ecs` only in its test module; `upload` and `render` take `&FrameData`; `FrameData` has no draw list; the culling pass reads `frame.instance_slot_count` alone | `renderer.rs:1-26`, `:685`, `:802-807`; `prepare.rs:23-50`; `culling_pass.rs:340-356` |
| 0004 | "every Frame data must reach the renderer, in order" | holds: `upload` asserts the previous frame's encoder was taken by `render`; a lost upload makes the next `prepare` rewrite every Instance | `renderer.rs:691-694`, `:785-790`, `:809-812`; `src/state.rs:299-312`; `prepare.rs:916-920` |
| 0004, 0016 | "Frame data is an owned, `Send` value with no borrow into the ECS World" | holds: compile-time assertion | `prepare.rs:52-55` |
| 0005 | "Per-Instance data (model matrix, normal matrix, Material ID) sits in a structured buffer"; "there is no bind group per mesh or per instance" | holds: 128 bytes, `STORAGE` | `instance.rs:31-40`; `renderer.rs:503-511`, `:541-558` |
| 0005, 0019 | "each index packs a Meshlet record in its high bits and a vertex within that Meshlet in its low six" | holds, Rust side: the record capacity is asserted below the bits above the vertex | `culling_pass.rs:219-221` |
| 0007 | "using `wgpu-hal` only for a subsystem that exceeds wgpu's safe API" | holds: the refit in `blas_refit.rs` and the acceleration-structure size query are the module's only wgpu-hal use | `blas_refit.rs:190-270`; `acceleration_structures.rs:115-173`; `src/gpu/mod.rs:494-498` |
| 0007 | reopen: "`vendor/` carries more than five upstream commits, which `vendor/README.md` lists, or a fix upstream has declined" | holds: one commit, gfx-rs/wgpu `59209f27`, merged upstream; the vendored `ray_tracing.rs` is the registry's 30.0.1 plus exactly that commit | `vendor/README.md:13-35`; `Cargo.toml` `[patch.crates-io]` |
| 0007 | reopen: the newest wgpu release cannot meet the floor, or the pinned wgpu is more than two majors behind | not judged from code: needs the newest published release, which is the exit's dependency review | `Cargo.toml:8` |
| 0007 | "on 2026-11-14 a copy of wgpu-core 30.0.1 carries one upstream fix" | holds as dated; the copy has been 30.0.2 with the same fix since 2026-11-15 (see *Notes for the verifier*) | `vendor/README.md:13` |
| 0013 | GPU memory "counted as the bytes of the buffers and textures the engine has created"; "the GPU stats readback's buffers are left out" | holds: every buffer and texture the Renderer and its passes create goes through the ledger under its kind; `GpuStatsReadback::new` takes none; `gpu_memory_bytes_by_kind` reads the ledger's running totals | `renderer.rs:394`, `:440-469`, `:503-512`, `:524-527`, `:585`, `:1007-1014`; `readback.rs:193-224`; `culling_pass.rs:227-238`; `froxel_pass.rs:73-95`; `targets.rs:119-136`, `:153-164`, `:177-191`; `tonemapping_pass.rs:87-104`; `shadow_mask_pass.rs:159-191`; `bend_pass.rs:62-71`; test `renderer.rs:1106-1139` |
| 0013, 0017 | a `TRANSIENT_ATTACHMENT` "counts as zero only when `transient_saves_memory` is `Some(true)`" | holds: the ledger is made from the adapter's info, which carries it | `renderer.rs:394`; `src/stats/gpu_memory.rs:128-131`, `:285-289` |
| 0013, 0019 | the culling phases' per-slot stages timed apart from `append_triangles` | holds: each phase is two Timed passes, slots then triangles | `renderer.rs:38-53`, `:863-869`, `:898-904`, `:1017-1028`; `culling_pass.rs:42-63`, `:340-388` |
| 0014 | "The GPU never holds a World position in f32: it reads an integer anchor on a 64 m grid plus an f32 offset" | holds: `anchor_and_offset` splits in f64 and rounds the offset once for the camera, every Instance and Placement pivot and every point light; a directional light's `offset` is a unit direction; `view_proj` is projection times the rotation-only view; both TLASes compose every entry's translation from its Anchor and offset against the TLAS's Anchor in f64, rounded once | `instance.rs:1-12`; `uniforms.rs:33-60`; `prepare.rs:157`, `:606-615`, `:960-964`, `:1007-1009`, `:1028`; `src/camera/mod.rs:84-86`; `acceleration_structures.rs:701-731`, `:869-876`, `:1043-1049`, `:1058-1066` |
| 0014 | "ray-tracing acceleration structures ... convert at their boundary into a local frame they own" | holds | `acceleration_structures.rs:243-252` (`AnchoredTlas`), `:1058-1066` |
| 0014 | "The static TLAS's frame is the corner of the camera's Anchor when its first static Instance was written" | holds: both TLASes follow the camera's Anchor, with no build, until an entry is first filled | `acceleration_structures.rs:394`, `:426-435`; test `:1290` |
| 0014, 0020 | "a build moves it only once the camera's Anchor strays more than a Chunk's edge (4 cells) along an axis"; "a build runs though no static Instance changed and re-anchors it at the camera's" | holds: `REANCHOR_CELLS` is 256 m / 64 m = 4; a strayed camera makes `start` record a build at the camera's Anchor; exactly 4 cells starts none | `acceleration_structures.rs:195`, `:572-581`, `:596-604`; `src/assets/cooked.rs:58`; `instance.rs:3`; test `acceleration_structures.rs:1382-1415` |
| 0014, 0020 | "the camera is again within 320 m ... adds less than 4.9 mm to a shadow ray's offset" | holds, Rust side: 5 cells of 64 m times `SHADOW_RAY_OFFSET_SCALE` 2^-16 is 4.88 mm; the offset itself falls in shaders | `acceleration_structures.rs:190-195`; `shaders/shadow_mask.wgsl:36`, `:169-172` |
| 0014, 0020 | the dynamic TLAS "rebuilt each frame in the bound static TLAS's frame, follows in the frame a new one is bound" | holds: `upload` reads the static TLAS's Anchor after its update, which may have bound the spare, and fills the dynamic TLAS and the camera uniform's `tlas_anchor` with it; the shadow mask pass binds the same index in that frame's `render` | `renderer.rs:711-742`, `:915-921`; `acceleration_structures.rs:407-416`, `:844-887` |
| 0015 | "Depth is cleared to 0"; "compared with `Greater`"; "in `Depth32Float`" | holds for all six main-pass pipelines, the twins included; phase two's depth pass and the lit pass load; the lit pipelines compare `Equal` and write nothing | `targets.rs:12`, `:194-199`, `:328-337`, `:341-350`, `:357-370`, `:405-423`; `main_pass.rs:127`, `:211` |
| 0015 | "a Hi-Z pyramid keeps the minimum depth, which is the farthest" | holds, Rust side: one dispatch per level, the GPU test against a CPU minimum of every sample; the reduction is the shaders' | `depth_pyramid_pass.rs:1-4`, `:97-122`; test `:141-153` |
| 0016 | "Only the main thread submits, presents or polls the wgpu `Queue` and `Device`" | holds: every submit and poll the module reaches runs on the thread calling `upload`, `render` or `flush_gpu_frame_stats`: the poll that retires frames, the frame staging, the load staging, BLAS compaction, the static TLAS's collect, the refit, the dynamic TLAS's build and the frame; the static TLAS's worker submits nothing | `renderer.rs:688`, `:698-700`, `:712-718`, `:832-838`, `:962-964`, `:970`, `:990-992`; `staging.rs:97`, `:358-365`; `readback.rs:319-347`; `acceleration_structures.rs:531-555`, `:594-621`; `blas_refit.rs:133-140`, `:147-153` |
| 0016 | "Workers record `CommandEncoder`s and return `CommandBuffer`s; they push no error scopes" | holds, newly applicable: the static TLAS's worker creates an encoder, records the build, finishes it and hands the command buffer back for the main thread to submit; it pushes no error scope | `acceleration_structures.rs:544-547`, `:592-623` |
| 0016 | the refit's SAFETY premise "only the main thread calls into wgpu meanwhile (ADR 0016)" | broken in the comment, not in the ADR: the static TLAS's worker calls into wgpu while `render` records the refits: L1-R05 | `blas_refit.rs:193-196`; `acceleration_structures.rs:592-622` |
| 0016 | "the compute pool is created with `available_parallelism() − 1` workers (at least 1), one pool per State's `FramePreparer`" | holds | `prepare.rs:487-489`, `:520`; `src/threading.rs:11-19` |
| 0016 | "prepare fills the Renderables' Instances on the pool once there are enough to split among its threads" | holds: two or more tasks of 1,024 Instances, batches of 128 taken dynamically, the main thread joining | `prepare.rs:195-264` |
| 0016 | "`desired_maximum_frame_latency` stays 2" | holds | `mod.rs:41-44` (`FRAMES_IN_FLIGHT = 2`) |
| 0016 | "Per-frame data the GPU reads is written into the per-frame staging ring ..., a ring of mapped buffers whose slot is ... reused only after its `map_async` callback fires"; no `Queue::write_*` | holds: the camera and light uniforms, lights, bends, Instances and the slot count are the frame staging's six buffers; a slot is written only once `is_mapped`, which only the callback sets; no `Queue::write_*` outside tests | `renderer.rs:515-527`, `:765-781`; `staging.rs:90-107`, `:163-172`, `:432-441` |
| 0016 | "the GPU stats come back through a ring of mapped readback buffers on the same rule"; "wgpu completes a map only after the last submission using the buffer finishes ... so the ring decides whether the CPU waits"; callbacks "at a known point of the frame" | holds: a slot is read and reused only at `MAP_DONE` or `MAP_FAILED`; a timestamp copy is recorded only into a slot `Recorded` for a completed frame; `render` never waits on a map; `on_submitted_work_done` on the frame's last encoder fires in a submit or poll on the main thread | `readback.rs:251-268`, `:305-313`, `:388-419`, `:510-522` |
| 0016, checklist | bind groups are snapshots; whatever recreates a target rebuilds the groups holding it in one function called at creation and after every resize | holds: `resize` recreates the targets and calls `bind_render_targets` on the pyramid, culling, main, shadow mask, tonemapping and light count passes, and re-derives the per-height LOD threshold and pixel size; the shadow mask's TLAS groups hold TLASes rebuilt in place | `targets.rs:61-65`; `renderer.rs:629-650`; `culling_pass.rs:292-293`, `:392-399`; `depth_pyramid_pass.rs:56`, `:62-94`; `main_pass.rs:80-85`, `:242-249`; `tonemapping_pass.rs:108`, `:119-122`; `light_count_pass.rs:66`, `:78-81`; `shadow_mask_pass.rs:192-207`, `:232-253`; test `renderer.rs:1165` |
| 0017 | "the Instance slots 30% of the scene and pass buffers', at 136.2 bytes a slot with its offsets and visibility bit" | broken, the same Finding: L1-R03 | `renderer.rs:204`, `:215-218`; `docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:18` |
| 0017 | "the lights 0.1%, at 48 bytes" | holds: `LIGHTS_PER_MILLE = 1`, `LightData` 48 bytes, 4,893 lights pinned | `renderer.rs:207`, `:262-265`; `uniforms.rs:17-29`; test `:1093` |
| 0017 | "the static TLAS's two TLASes 57% of the acceleration structures', at 289 bytes an entry" | holds: `STATIC_TLAS_PER_MILLE = 570`, two TLASes at `TLAS_BYTES_PER_INSTANCE = 289`; the fit test bounds the created TLASes by it | `renderer.rs:211`, `:223`, `:271-274`; test `:1128-1134` |
| 0017 | "the staging Budget the frame staging's slots at their largest"; the Chunk loads take what those slots leave of it, over the frames in flight and the one being written | holds: frame staging slots are created through the ledger as `Staging`, never past `max_slot_size`; `load_bytes_per_frame` is the rest over three; `most_uploaded_bytes` sums the six buffers `FrameStaging` is bounded by | `renderer.rs:244-248`, `:266-275`, `:279-287`, `:515-527`; `staging.rs:81-87`, `:137-153`; test `renderer.rs:1135-1138` |
| 0017 | "What ADR 0013 leaves out, the GPU stats readback and the load staging, bypasses [the ledger]" | holds in code; ADR 0013 does not leave the load staging out: L1-R04 | `staging.rs:179-184`, `:344-353`; `readback.rs:193-200` |
| 0017 | "Streaming uploads go through a `StagingBelt`, never `Queue::write_texture`" | holds: every write, re-pitched mip and Texture class move is recorded on the load encoder, the dither mask included; no runtime `write_texture` or `write_buffer` outside tests | `staging.rs:207-215`, `:225-278`, `:284-334`; `tonemapping_pass.rs:105`; `renderer.rs:403`, `:698` |
| 0017 | "the engine caps the bytes written into it per frame at the staging Budget" | holds, staging side: `written_bytes` counts row padding, `mips_written_bytes` sizes the same; enforcing the cap is the streaming module's | `staging.rs:211`, `:241`, `:368-394` |
| 0017 | "Residency changes are batched so that bind group is rebuilt at most once per frame" | holds, Rust side: `render` calls `refresh_bind_group` once; whether it rebuilds is the Material pool's | `renderer.rs:813` |
| 0017, 0018, checklist | no GPU buffer grows at runtime; only per-frame staging grows, up to the buffers a frame writes, which it asserts | holds: the Instance, light and uniform buffers, both static TLASes and the dynamic TLAS are made at their capacities; the frame staging asserts its bound | `renderer.rs:440-469`, `:502-527`; `staging.rs:79-87`; `acceleration_structures.rs:334-354`, `:776-786` |
| 0018 | "Rows are stored unpadded, and the loader re-pitches them to 256 bytes as it writes into the `StagingBelt`" | holds | `staging.rs:225-256`, `:407-426` |
| 0019 | pass order: culling one, Froxel lists, depth pre-pass, pyramid, culling two, phase two's depth and the shadow mask with rays on, lit, tonemapping | holds; the Froxel pass after the first culling dispatch is the last audit's note, unchanged | `renderer.rs:863-938` |
| 0019 | "a compute pass appends the Meshlets of the LOD selected for each Instance ... in slot order, in two phases" | holds, Rust side: offset scans, then records, then triangles, per phase; each phase is now two wgpu compute passes (see *Notes for the verifier*) | `culling_pass.rs:1-20`, `:340-388`, `:364-374` |
| 0019 | "Phase one appends those visible the frame before, one bit per slot" | holds: `capacities.instances / 32` words, zeroed at creation | `culling_pass.rs:276-281` |
| 0019 | "a depth pre-pass draws them into 4x MSAA reverse-Z `Depth32Float` cleared to 0 ... and stores it" | holds | `main_pass.rs:110-114`, `:291-292`; `targets.rs:328-337` |
| 0019 | "A compute pass reduces that depth into the Hi-Z pyramid, each texel the minimum of every sample below it" | holds, Rust side | `depth_pyramid_pass.rs:97-122`; `targets.rs:172-191`, `:212-222` |
| 0019 | "Phase two appends the others whose bounds lie anywhere nearer than the pyramid's minimum ... and rewrites the bits" | holds, Rust side: the whole pyramid is bound in `FRAME_DRAW` | `culling_pass.rs:463-490`; `bind_groups.rs:126-144` |
| 0019 | "Phase two's Instances draw into depth: with shadow rays off, at the start of the lit pass; with them on, in a pass of their own, which like the pre-pass then writes each sample's surface normals too" | holds, the twins included | `main_pass.rs:283-330`, `:358-363`; `renderer.rs:905-922` |
| 0019 | "a transient 4x colour attachment, memoryless on Apple GPUs, that it resolves into the `Rgba16Float` HDR target, which a tonemapping pass then writes into the target" | holds: `TRANSIENT_ATTACHMENT`, cleared, `Discard`, resolved into `hdr` | `targets.rs:200-205`, `:377-400`; `tonemapping_pass.rs:151-177` |
| 0019, 0027 | "one `draw_indexed_indirect` per phase for the depth pipeline and one of both phases for the lit pipeline, and one more per phase and pipeline for Thin Primitives and small Instances"; "No count draws" | holds: seven indirect draws and the tonemapping triangle, counted as 8 with rays on or off; no `multi_draw_indirect` or count draw | `main_pass.rs:315-368`; `tonemapping_pass.rs:176`; `renderer.rs:943-948`; test `:1156` |
| 0019 | "with shadow rays on, the depth passes also write each sample's triangle normal and the normal interpolated over it, 4 bytes per sample ... the target is allocated either way" | holds | `targets.rs:42-45`, `:247-252`; `main_pass.rs:141-169`, `:300-310` |
| 0019 | "the coarsest LOD whose LOD error times the Instance's scale is under the LOD threshold, one pixel of the target's height"; "It reads the camera and the Instance alone, so both phases draw the same LOD and no earlier frame decides it" | holds, Rust side: `lod_error_per_metre` is one pixel at a metre on the target's height, set at creation and on resize and sent in the camera uniform each frame; the culling pass binds no earlier frame's data for it; the comparison is the shaders' | `renderer.rs:186-201`, `:594-595`, `:648-649`, `:733-742`; `uniforms.rs:43-46`; `culling_pass.rs:1-5`, `:299-312`; test `renderer.rs:3130` |
| 0019 | "Culling tests Instances"; "Render pipelines are vertex pipelines" | holds: six vertex pipelines, no mesh stage | `culling_pass.rs:1-20`; `main_pass.rs:115-222` |
| 0019 | "the light buffer ... of fixed capacity" | holds: made once at `capacities.lights` | `renderer.rs:460-469` |
| 0019 | gate "timed from the first culling dispatch to the end of the lit pass"; "Shadow-ray time is reported beside the gate as its own line" | holds: from culling phase one's start to the latest end of the passes up to the lit pass; the shadow mask pass is a Timed pass | `readback.rs:728-745`; `renderer.rs:38-53` |
| 0019 | "A fragment stage that reads any flat input is never merged ... both read a flat input" | falls in shaders | |
| 0019, 0020 | "a shadow mask ... which holds 59 lights per pixel: a light past them casts no shadow there, and Frame stats count it" | holds: 59, asserted to fill 64 bits with the Froxel slice; the overflow buffer cleared each frame and read back | `shadow_mask_pass.rs:35`, `:66-70`, `:351`, `:406-408`; `renderer.rs:949-954` |
| 0019, 0020 | "for one of the first seven directional lights ... toward that light again from each other sample" | holds, Rust side: `SHADOW_EDGE_LIGHTS` 7, asserted to fit with the samples in 32 bits; the edge list is the fourth, indirect dispatch | `shadow_mask_pass.rs:55`, `:60-64`, `:384-401` |
| 0019 | the shadow mask traced after phase two's depth pass and before the lit pass, at the nearest sample and, at the farthest of the others, from a packed list; 4x MSAA pinned where the pass reads samples | holds, Rust side | `renderer.rs:905-931`; `shadow_mask_pass.rs:44-49`, `:58`, `:352-353`, `:360-383` |
| 0019, 0021 | "`max_storage_buffers_per_shader_stage` at least 9, since the culling pass's compute stage reads and writes nine" | holds, exactly nine: four in `SCENE_GEOMETRY` and five in `FRAME_DRAW`; the shadow mask pass's compute stage now binds nine too (see *Notes for the verifier*) | `bind_groups.rs:20-42`, `:71-102`, `:126-144`, `:212-280` |
| 0020 | "it holds exactly the static Instances of the loaded, drawn Chunks, the drawn Far proxies and the static spawned Renderables, each in an entry of its own whatever its slot" | holds, Rust side: an entry per written static Instance with a BLAS, mapped by slot; a freed slot, a Moving Instance's write or a Mesh without a BLAS frees the entry; which Far proxies are drawn is the streaming module's | `acceleration_structures.rs:448-506`, `:629-666`; tests `:1723`, `:1746` |
| 0020 | "leaves out only those written while it is full, which take the entries it frees" | holds: left out by slot, taken lowest slot first as entries free, counted in Frame stats, warned once per run of leaving out | `acceleration_structures.rs:502-525`, `:693-696`; `renderer.rs:723`; test `:1835` |
| 0020 | "two static TLASes, each with its own bind group, and a worker thread takes the static Instance changes the spare lacks, records and finishes its build, which the main thread submits"; "the spare is bound from the frame after that submit" | holds: `upload` hands Frame data to `StaticTlas::update` on the main thread; a `Submitted` spare is swapped in at the start of the next `update`; the shadow mask pass binds the bound TLAS's group by index | `renderer.rs:711-719`, `:915-921`; `acceleration_structures.rs:217-241`, `:385-424`, `:531-555`, `:563-625`; `shadow_mask_pass.rs:83-84`, `:134-143`, `:361-365`; test `acceleration_structures.rs:1547-1601` |
| 0020 | "a capture waits for the worker so its images never depend on the worker's timing" | holds: with `wait_for_static_tlas`, which every capture path sets, `update` waits for each worker and every change starts a build | `renderer.rs:106-111`, `:615`, `:712-718`; `acceleration_structures.rs:398-403`, `:417-422`, `:531-537`; `src/capture/mod.rs:171-174`, `:277-280`; `src/state.rs:681` |
| 0020 | both static TLASes "built empty at creation, so rays can always query them" | holds | `acceleration_structures.rs:355-361` |
| 0020, 0026 | "a Far proxy's entry change alone ... joins the next build and starts one only once it is 120 frames old" | holds: a Far proxy's write or release does not set `starts_build`; the oldest change the bound TLAS lacks starts one at 120 frames | `acceleration_structures.rs:197-199`, `:466-500`, `:513`, `:566-576`; test `:1611` |
| 0020 | "Moving Instances form the dynamic TLAS, rebuilt each frame ... from at most 10,000 of them, those in the lowest slots" | holds: `DYNAMIC_TLAS_CAPACITY` 10,000; prepare lists the 10,000 in the lowest slots and the fill takes them | `acceleration_structures.rs:13-17`, `:850-855`; `prepare.rs:846-862`; test `prepare.rs:1615`; `renderer.rs:737-738`; `uniforms.rs:51-52` |
| 0020 | "its build's recording and submit, and the build itself on the GPU, are timed in each frame that builds it"; the rebuild form "times neither those rebuilds nor the dynamic TLAS's build, which reads as zero" | holds: recording (`prepare_encoder`, `fill`, `record`) and submit are CPU stages; the build reads its bracket, from no earlier than the frame before's last pass, except under the rebuild form; the amendment answers the last audit's drift Finding on the rebuild form | `renderer.rs:55-57`, `:68-69`, `:604`, `:705-709`, `:728-731`, `:844-857`, `:959-965`; `readback.rs:464-480`, `:668-677`; `acceleration_structures.rs:889-916`, `:942-961` |
| 0020 | "That submit, and each frame's that binds a static TLAS, walks one dependency per distinct BLAS since `vendor/wgpu-core` carries" the upstream fix | holds: the vendored wgpu-core dedups by BLAS tracker index | `vendor/wgpu-core/src/command/ray_tracing.rs:233-248`; `vendor/README.md` |
| 0020 | the crowd's refit form, "refitted to its bent shape each frame through wgpu-hal on Metal ... the dynamic TLAS's build waits for the refits on the GPU through a Metal shared event; the refits are timed on the GPU from the end of the bend pass to a marker after them" | holds: with a refit or rebuild form and a bend, the head is submitted with the updates before the build is recorded, and the submission is split in the timestamp log | `blas_refit.rs:64-70`, `:120-145`, `:225-270`, `:470-544`; `renderer.rs:824-843`; `readback.rs:620-630`, `:731-733` |
| 0020 | "Its rebuild form ... rebuilds each figure's BLAS through wgpu each frame" | holds | `blas_refit.rs:116-118`; `src/assets/pools.rs:710-717` |
| 0020 | "each Mesh rays trace gets a BLAS at load, compacted unless it is loaded to be refitted or rebuilt each frame"; "Every BLAS is built from its Mesh's LOD 0" | falls in cook with assets (`src/assets/pools.rs`); the refit reads the pool's `REFITTED_BLAS_FLAGS` and the geometry the pool built the BLAS from | `src/assets/pools.rs:152-154`, `:691-705`; `blas_refit.rs:168`, `:205-207`, `:256`, `:313-335` |
| 0020 | one hard shadow ray per pixel per light, the edge rays, the origin offset, "a shadow ray the static TLAS lets through is traced against [the dynamic]" | falls in shaders; the Rust side dispatches the four passes, binds both TLASes and passes the TLAS Anchor | `shadow_mask_pass.rs:345-402`; `renderer.rs:733-741` |
| 0021 | required features: the three binding-array features, `TEXTURE_COMPRESSION_BC`, `EXPERIMENTAL_RAY_QUERY`, `TIMESTAMP_QUERY` | holds: requested exactly; no other feature used by `src/render/` or `src/gpu/` outside tests; `sample_mask` needs no naga capability | `src/gpu/mod.rs:68-86`, `:269-280` |
| 0021 | downlevel flags `ANISOTROPIC_FILTERING`, `VERTEX_STORAGE`, `COMPUTE_SHADERS`, `INDIRECT_EXECUTION` | holds | `src/gpu/mod.rs:284-297` |
| 0021 | `TIMESTAMP_QUERY_INSIDE_ENCODERS` and `_INSIDE_PASSES` stay optional | holds: one Timestamp query set, every timestamp a pass's `timestamp_writes`, none inside an encoder or pass | `readback.rs:202-207`, `:582-630`; `culling_pass.rs:438-441`; `main_pass.rs:303-314`, `:346-356` |
| 0021 | "4x MSAA on every attachment format the main pass uses ... Those formats are `Depth32Float`, the HDR target's `Rgba16Float` and the surface normals' `Rgba8Unorm` ... the floor checks no resolve on it" | holds: `MULTISAMPLED_FORMATS` lists the three, the floor reads each and asks the resolve of the HDR format alone, and `RenderTargets::new` asserts no multisampled target of another format; the last audit's floor Finding is fixed | `targets.rs:47-52`, `:114-118`; `src/gpu/mod.rs:57-66`, `:363-374` |
| 0021 | "returns one typed error naming ... each attachment format without 4x MSAA or, on colour, its resolve" | broken in the ADR's text: the floor names a missing resolve on the HDR format only, as the ADR's first Consequence says: L1-R06 | `docs/adr/0021-capability-floor.md:17`; `src/gpu/mod.rs:363-374` |
| 0021 | "the window's surface and the capture target take only the tonemapping pass's single-sample output" | holds | `tonemapping_pass.rs:83` |
| 0021 | "`ExperimentalFeatures::enabled()`, an `unsafe` call; it is the one `unsafe` this ADR adds, and its comment cites ADR 0020" | holds: the other `unsafe` blocks in the module belong to ADR 0020's refit and to the acceleration-structure size query, each with a SAFETY comment | `src/gpu/mod.rs:72-78`; `acceleration_structures.rs:128-130`, `:165-173`; `blas_refit.rs:190-270`, `:298-309`, `:345-367`, `:483-494`, `:526-556` |
| 0021 | limits: defaults with `using_minimum_supported_acceleration_structure_values()`, `max_buffer_size` 2^31 − 1, binding-array elements 65,536, acceleration structures 2, storage buffers 9 | holds; `max_texture_array_layers` is written out at the default 256, raising nothing; five Channels of 13,107 classes take 65,535 elements in the fragment stage; two `SHADOW_RAY_SCENE` groups, one per static TLAS, each binding two TLASes | `src/gpu/mod.rs:303-323`; `bind_groups.rs:49-66`, `:111-118`, `:672-686`; `src/assets/material.rs:19-20` |
| 0021 | the other per-stage counts at wgpu's defaults | holds: storage textures at most 2 of 4, sampled textures at most 2 outside binding arrays of 16, uniforms at most 2 of 12, samplers 2 of 16, colour bytes per sample 8 of 32, bind groups 0 to 3 of 4; storage texture formats `R32Float`, `Rg32Uint`, `R32Uint`, each guaranteed | `bind_groups.rs:20-409`; wgpu-types 30.0.2 `limits.rs:421-445`, `texture/format.rs:969-986` |
| 0021 | "`max_storage_buffer_binding_size` stays at the default 128 MiB ... that Arena stays within it, as a test checks" | holds: the vertex Arena, the Instance buffer and the light buffer are tested against it; the packed indices are 2^25 × 4 bytes, the limit exactly | `src/gpu/mod.rs:821-845`; `culling_pass.rs:93-101`, `:256-260` |
| 0021 | `max_compute_workgroups_per_dimension` default 65,535 | holds: asserted against the device; `append_triangles` is indirect in rows of it; the slots passes dispatch 1,909 workgroups at the checked-in Budgets | `culling_pass.rs:77-81`, `:224-226`, `:364-387` |
| 0021 | "Headless contexts, tests and Goldens included, use the same floor"; "`praxis capabilities` runs without the floor" | holds | `src/gpu/mod.rs:31-93`, `:122-129` |
| 0022 | "prepare blends the previous and current World transform in f64 into Frame data"; a spawned or Teleported entity is drawn at its current transform | holds: Instances, both light kinds and bends blend by `blend_weight`; `drawn` returns the current transform when the previous is `None` | `prepare.rs:215`, `:1004`, `:1017`, `:1099`; `src/transform/mod.rs:26-29` |
| 0023 | "a `DirectionalLight` gives illuminance in lux and a `PointLight` luminous intensity in candela"; "Exposure is an EV100" | holds: carried as colour times intensity; the camera uniform carries `Camera::exposure`; the background pre-exposed | `prepare.rs:617`, `:1010`, `:1033`; `uniforms.rs:1-10`, `:25-28`, `:40` |
| 0023, checklist | HDR colour clamped to 65504; sRGB encoded once, by the tonemapping pass, into 8-bit UNORM | holds: the clear is clamped and weighted; `debug_assert!(!target_format.is_srgb())`; the surface is `Bgra8Unorm` or `Rgba8Unorm`, tested | `targets.rs:377-400`; `tonemapping_pass.rs:52-55`; `src/gpu/mod.rs:458-485`; test `:691-707` |
| 0026 | "Under it the Instance slots, the static TLAS and the flat cull keep M4's capacities" | holds on its purpose: the densest 2 km view fits the capacities, tested; the slots fell from 517,376 to 488,448 when Thin Primitives landed, which L1-R03 records | `renderer.rs:1088-1098`; `src/scenes/world.rs:858`, `:879` |
| 0026 | "The Resident Far proxies are in the World beside the loaded Chunks, each a Chunk of one Placement that takes one Instance slot"; "A Far proxy is held, its slot an Instance with no Mesh, while it is past the View distance or its Chunk is drawn" | holds, prepare side: `ResidentFarProxies` and `FarGround` take ranges after the loaded Chunks'; when one is held is the streaming module's | `prepare.rs:630-642`, `:699`, `:725-728` |
| 0026 | "A Far proxy is a Mesh and the culling pass selects its LOD as any Instance's" | holds, Rust side: the culling pass has no Far proxy case | `culling_pass.rs:1-20` |
| 0026 | "While drawn it holds one static TLAS entry under the mask bit an ordinary Instance has" | holds: a Far proxy's Mesh has no Thin Primitive, so its entry takes `ORDINARY_MASK` | `acceleration_structures.rs:990-994`; `src/assets/pools.rs:862-869` |
| 0026 | "a swap is a static change: the static TLAS takes both sides of it from its own next build, and the draw does not wait"; "a capture, which waits for the static TLAS's worker, builds each at once" | holds: Chunk, Far proxy and far ground writes are never Moving; the Placements written or released in the swap's frame start the build that carries the Far proxy's change too; the draw reads the Instance buffer, not the TLAS | `prepare.rs:727`, `:969`; `acceleration_structures.rs:398-405`, `:466-500` |
| 0026 | "the proxy is placed once that BLAS is ready, as a Chunk is" | falls in streaming; the TLAS asserts it in debug builds only (see *Notes for the verifier*) | `acceleration_structures.rs:454-458` |
| 0026 | the ground past the View distance is "placed by the scene as a Chunk of one Placement that is never held, and traced by no ray" | holds: drawn as a Chunk of one Placement; a `MeshUse::Drawn` Mesh has no BLAS, so its write frees any entry | `prepare.rs:636-642`; `acceleration_structures.rs:452-476`; `src/assets/pools.rs:113-139`; `src/scenes/world.rs:246-252` |
| 0026 | "Thin Primitives carry an instance-mask bit per width class, one class per doubling" | holds, Rust side: bits 1 to 7 for classes from 3.125 mm, each twice the one before; 400 mm and wider take the ordinary bit; declared into WGSL from Rust | `acceleration_structures.rs:969-1008`; `shaders/mod.rs:316-317`; test `acceleration_structures.rs:1441-1470` |
| 0027 | "No pipeline of the main pass enables alpha-to-coverage" | holds: one `MultisampleState`, count 4, `alpha_to_coverage_enabled: false`, shared by all six pipelines | `main_pass.rs:110-114`, `:128`, `:212`, `:216-222` |
| 0027 | "The depth passes write the mask ... The lit pass ... writes none" | holds, Rust side: the depth twins run `fs_thin_depth` or `fs_thin_surface_normals`, which output `sample_mask`; the lit twin runs `fs_thin_main`, which reads it; what they compute falls in shaders | `main_pass.rs:157-186`, `:217-222` |
| 0027 | "Each pipeline of the main pass has a twin ... The twins draw every frame, since any Instance can be small" | holds: three pipelines, three twins, each recorded unconditionally after its pipeline, every frame | `main_pass.rs:39-42`, `:323-329`, `:358-368`; `renderer.rs:883-931` |
| 0027 | "the culling pass lists the geometry drawn through the twins apart, in slot order within each phase"; "the twins' packed indices share the frame's draw's buffer: each phase's fill its end" | holds, Rust side: each phase's Thin draw in `FrameDraw`, its packed indices at the buffer's end; the storage-buffer count stays nine; the order is the shaders' | `culling_pass.rs:6-11`, `:104-122`, `:137-142`, `:256-260` |
| 0027 | a Thin Primitive's and a small Instance's coverage "reads the camera and the Instance alone" | holds, Rust side: `pixel_size_per_metre` is in the camera uniform, set from the target's height at creation and on resize | `renderer.rs:197-201`, `:595`, `:649`, `:733-735`; `uniforms.rs:57-59` |
| 0027 | the five levels, the Thin Primitive and small Instance coverage, the culling drop, the second-surface listing of a split pixel | falls in shaders | |
| 0008, 0020, checklist | the main pass reads no screen-space history, and nothing rules out its 4x MSAA | holds: `MAIN_PASS_INPUTS` binds this frame's records, mask and mask samples | `bind_groups.rs:285-317` |
| checklist | GPU memory through the ledger under its kind; acceleration structures "at the size wgpu-hal's query gives (`blas_bytes`, `tlas_bytes`), a compacted BLAS at its adapter's estimated fraction" | holds: the query is built as wgpu-core builds its own at creation, plus wgpu-core's instance buffer; the fraction is chosen by vendor and backend as its doc says; the refit scratch and both TLAS marker results are counted | `acceleration_structures.rs:29-188`, `:340-351`, `:776-799`; `blas_refit.rs:372-381`, `:413-422`; `src/stats/gpu_memory.rs:188-209`; `vendor/wgpu-core/src/device/ray_tracing.rs:56-117`, `:242-282` |
| checklist | "per-frame GPU writes add to `uploaded_bytes`" | broken for the static TLAS's builds, whose instance records wgpu-core stages and uploads in the frame that submits them, uncounted: L1-R02; holds for the frame staging's six buffers, the dynamic TLAS's records, the streamer's load staging bytes and the Texture class moves | `acceleration_structures.rs:544-552`; `renderer.rs:698`, `:712-723`, `:778-781`, `:851-856`; `staging.rs:132`, `:358-365`; `src/state.rs:287` |
| checklist | the indirect buffer is not read-write bound in an active group | holds: each indirect dispatch runs with the stand-in bound for the buffer it reads; the main pass's groups hold no `frame_draw` | `culling_pass.rs:376-387`, `:466-490`; `main_pass.rs:373-386`; `shadow_mask_pass.rs:99-101`, `:266-270`, `:377-383`, `:395-401` |
| checklist | the GPU builds the frame's draw in slot order within each phase, never in the order atomics land | holds, Rust side: prefix sums by block, then blocks, then records; static TLAS entries are taken on the main thread in write order; the shadow mask's lists are appended by atomics but each listed pixel writes only its own pixel | `culling_pass.rs:364-374`; `acceleration_structures.rs:440-527`, `:629-666` |
| checklist | a binding is written once, in its definition; no WGSL part declares one | holds | `bind_groups.rs:513-541`; `shaders/mod.rs:98-179` |
| checklist | matrices column-major; Rust-only padding starts with `_`; `FrameDraw` matches `culling.wgsl` | holds: `padded_columns` pads each column; the uniforms' padding fields are `_pad*`; `FrameDraw` is 112 bytes, its fields at WGSL's offsets by inspection | `instance.rs:14-18`; `uniforms.rs:9`, `:24`, `:28`, `:61`; `culling_pass.rs:111-128`; `culling.wgsl:28-61` |
| checklist | no `unwrap()` on a GPU operation in the render path; no `unsafe` unless necessary and explained | holds: the poll failure is logged; the new size-query `unsafe` is the only way to the driver's size, since wgpu-core keeps it crate-private [ref:wgpu-30-blas-compacted-size-private]; one premise of the refit's comment is stale, L1-R05 | `renderer.rs:688-690`; `acceleration_structures.rs:29-35`, `:128-130`, `:165` |

### Spec stories

| Story | Verdict | Where |
|---|---|---|
| M4 story 2, Placements as Instances in a slot range their Chunk owns | holds: a Chunk is a range, not entities | `prepare.rs:294-321`, `:351-387`, `:694-733` |
| M4 story 17, Moving Instances spawned by scene systems, outside Chunks | holds, prepare side: a Placement's Instance is never written as Moving | `prepare.rs:727`, `:969` |
| M4 story 20, Chunks leaving the Load radius unloaded | holds, prepare side: the range is written Instances with no Mesh and freed for the next frame | `prepare.rs:391-402`, `:686-693` |
| M4 story 24, the bytes a frame writes into the staging belt capped by the staging Budget | holds, render side: the cap is computed in `from_budgets`, and `written_bytes` and `mips_written_bytes` count the belt's bytes with row padding; enforcing it is the streaming module's | `renderer.rs:244-248`, `:275`; `staging.rs:211`, `:241`, `:382-394` |
| M4 story 28, a stale Handle refused, never drawn as the Asset that reused its index | holds, prepare side: Mesh lookups and Material generations checked for Renderables and Placements, and every Instance that may name a freed index rewritten on the frame after a free | `prepare.rs:670-672`, `:756-783`, `:925-937`, `:953-958` |
| M4 story 32, each Channel a binding array of texture arrays, one per class | holds, Rust side | `bind_groups.rs:44-66`, `:672-686` |
| M4 story 33, the floor kept at 256 layers per texture array | holds: `FLOOR_TEXTURE_ARRAY_LAYERS` is wgpu's default | `src/gpu/mod.rs:303-306`, `:315` |
| M4 story 37, a Texture moved to its new class by copy through the staging belt, safe API only | holds: `copy_texture_to_texture` per mip on the load encoder, its bytes returned at `submit` into `uploaded_bytes` | `staging.rs:284-334`, `:358-365`; `renderer.rs:698` |
| M4 story 43, every fixed capacity sized from its Budget at device creation, not a compile-time constant | holds for the Instance slots, the light buffer, the static TLAS's entries, the load bytes a frame and the culling pass's per-slot buffers; the frame's draw and the Froxel lists are fixed by the floor and held in the scene and pass buffers' Budget; `DYNAMIC_TLAS_CAPACITY` and `BEND_CAPACITY` stay constants (see *Notes for the verifier*) | `renderer.rs:209-211`, `:251-293`, `:402`; `prepare.rs:495-508`; `culling_pass.rs:83-101`, `:244-281`; `froxel_pass.rs:26-31`; `acceleration_structures.rs:13-17`; `bend_pass.rs:22` |
| M4 story 44, GPU memory counted by kind where it is created, "so that no buffer or texture escapes `gpu_memory_bytes`" | holds for every buffer, texture and acceleration structure the module creates; the load staging's belt buffers escape it, as ADR 0017 allows and ADR 0013 does not record: L1-R04 | `renderer.rs:440-469`, `:503-512`, `:1012-1014`; `staging.rs:140-149`, `:344-353` |
| M4 story 45, acceleration-structure bytes counted exactly | holds: the driver's size through wgpu-hal, a compacted BLAS at a measured per-adapter fraction, labelled an estimate | `acceleration_structures.rs:29-188` |
| M4 story 46, the vertex Arena within one 128 MiB binding | holds: tested | `src/gpu/mod.rs:817-825` |
| M4 story 47, a request past any capacity refused, logged and counted | holds for Chunk ranges, lights, the frame's draw, the static and dynamic TLAS and bends; a Renderable is refused early while free ranges hold slots, though counted, not silently: L1-R01 | `prepare.rs:332-338`, `:372-379`, `:661-668`, `:846-862`, `:871-878`; `culling_pass.rs:113-116`, `:337-339`; `acceleration_structures.rs:502-525`, `:693-696`; `renderer.rs:723`; `src/state.rs:313-317` |
| M4 story 48, the static TLAS holds exactly the static Instances of the loaded Chunks | holds, Rust side: held and unloaded Chunks' slots are written Instances with no Mesh and free their entries; Chunk writes are static | `prepare.rs:686-693`, `:725-731`; `acceleration_structures.rs:448-506`; test `:1746` |
| M4 story 49, the static TLAS's anchor moves with the camera | holds | `acceleration_structures.rs:563-604`; test `:1382` |
| M4 story 50, the static TLAS rebuilt off the frame's critical path and swapped in whole | holds | `renderer.rs:711-719`; `acceleration_structures.rs:407-416`, `:563-625`; test `:1547` |
| M4 story 51, frames before a newly loaded Chunk casts shadows measured | holds: `frames_until_traced` into Frame stats; the ceiling falls in tooling | `acceleration_structures.rs:322-328`, `:414-415`; `renderer.rs:722` |
| M4 story 52, the dynamic TLAS follows the static TLAS's anchor in the frame it swaps in | holds | `renderer.rs:724-741` |
| M4 story 53, waits on BLAS compaction measured on frames that load | holds, render side: a `blas_compaction` stage on frames that wait; the compaction is the cook and assets module's | `renderer.rs:71-73`, `:699-704` |
| M4 story 54, lights stream with their Chunks | holds | `prepare.rs:1020-1024` |
| M4 story 55, the light buffer filled by distance from the camera | holds: past the capacity the farthest point lights are cut, directional lights first; tested | `prepare.rs:1038-1061`; test `:2134` |
| M4 story 60, 10,000 Moving Instances | holds, prepare side: the dynamic TLAS takes the 10,000 in the lowest slots | `prepare.rs:846-862` |
| M4 story 65, the same content at the origin and the far corner gives identical images | falls in tests (the capture seam); the TLAS side composes every translation against its Anchor in f64 | `acceleration_structures.rs:1058-1066` |
| M4 story 70, the static TLAS's build time reported apart from the frame's critical path | holds: the worker's time into `static_tlas_build_time`, read in the frame that submits its build, and the main thread's share as the `static_tlas_update` stage | `renderer.rs:64-66`, `:719-721`; `acceleration_structures.rs:319-324`, `:547`, `:616-620` |
| M4 story 71, GPU memory a running total | holds: read from the ledger's totals | `renderer.rs:1012-1014` |
| M5 story 8, every Instance at the coarsest LOD whose error is under a pixel | holds, Rust side; the selection falls in shaders | `renderer.rs:186-201`, `:733-735`; `culling_pass.rs:1-5` |
| M5 story 9, selection reads no earlier frame | holds, Rust side: no history is bound for it; the visibility bits pick a phase only | `renderer.rs:733-742`; `culling_pass.rs:200-202`, `:276-281` |
| M5 story 10, Render switches forcing LOD 0 and a given LOD | holds: `forced_lod`, `None` as it ships, clamped to the coarsest LOD and declared into WGSL | `renderer.rs:112-115`, `:169`; `culling_pass.rs:299-311` |
| M5 story 12, every BLAS from LOD 0, so a LOD switch starts no static build | falls in cook with assets; the TLAS starts builds from Instance writes alone, and selection writes no Instance | `acceleration_structures.rs:448-506` |
| M5 story 18, Chunks outside the Load radius drawn as their Far proxy | holds, prepare side: every Resident Far proxy not held is written | `prepare.rs:630-642`, `:699-731` |
| M5 story 19, the swap whole at the Load radius | holds, prepare side: a Chunk's and its Far proxy's whole ranges are written in the `prepare` that sees their held state change | `prepare.rs:700-731` |
| M5 story 21, flat ground to the horizon past the View distance | holds, prepare side: `FarGround` drawn as a Chunk of one Placement | `prepare.rs:636-642` |
| M5 story 27, Far proxies cast and receive the sun's shadows | holds, Rust side: written static, entries under the ordinary bit | `prepare.rs:727`, `:969`; `acceleration_structures.rs:478-500`, `:990-994` |
| M5 story 28, a swap traced within N = 10 frames, no static TLAS build on the critical path | holds, Rust side: a swap starts a build in its frame, built by the worker; the frames until traced are counted; the reading of N falls in tooling | `renderer.rs:711-722`; `acceleration_structures.rs:466-500`, `:563-625` |
| M5 story 29, shadow-ray time with Far proxies traced and not | falls in tooling: no Render switch untraces Far proxies | `renderer.rs:78-115` |
| M5 story 35, a Thin Primitive drawn a pixel wide | holds, Rust side: the pixel size per metre in the camera uniform, the twins drawn and counted; the five levels fall in shaders | `renderer.rs:733-735`, `:943-948`; `main_pass.rs:157-186`, `:217-222` |
| M5 story 36, masked geometry drawn through pipelines of its own | holds | `main_pass.rs:1-13`, `:39-42`, `:323-368`; `renderer.rs:943-948` |
| M5 story 37, a Thin Primitive's shadow only onto receivers no wider than it | holds, Rust side: the width-class bits; the receiver's ray mask falls in shaders | `acceleration_structures.rs:969-1008` |
| M5 story 39, a sub-pixel Instance scaled to a pixel, dropped at zero coverage | falls in shaders; Rust side it draws through the twins with the camera's pixel size | `culling_pass.rs:6-11`; `renderer.rs:733-735` |
| M5 story 40, the Instance slots in use in the Bench record and the culling phases' per-slot stages judged against 1 ms | holds, Rust side: `instance_slot_count` is one past the highest slot taken, written each frame; the per-slot steps run in passes of their own, timed and read back apart; the judgment falls in tooling | `prepare.rs:845`; `renderer.rs:38-53`, `:782`; `culling_pass.rs:42-63`, `:340-388`; `readback.rs:728-738` |

### Registry keys

Every key cited in the module was read against its registry quote, and
against the wgpu, wgpu-hal and wgpu-types 30.0.2 sources `Cargo.lock`
resolves and the vendored wgpu-core where a quote is load-bearing; each
quote is found verbatim in 30.0.2, though the registry's URLs name
v30.0.1 (see *Notes for the verifier*). All 84 match, by file:

| File | Keys | Verdict |
|---|---|---|
| `renderer.rs:192` | `meshoptimizer-1.3-lod-selection` | matches: a pixel's size a metre from the camera, `2 tan(fov / 2) / height`, is the quote's `tan(camera_fovy / 2) * 2 / screen_height` with the projection's own vertical field of view; the quote's `lod_factor` is `LOD_THRESHOLD_PIXELS`, 1; the comparison's direction is the shaders' and has its `deviations.md` row |
| `renderer.rs:680` | `wgpu-30-poll-retires` | matches: a `PollType::Poll` maintains the device without waiting, and the queue's maintain retires finished submissions (`vendor/wgpu-core/src/device/resource.rs`) |
| `prepare.rs:172-177` | `openmp-5.2-schedule-dynamic`, `async-executor-1.14.0-woken-ticker-notifies-next`, `onetbb-2023.2-grainsize` | matches: each task and the main thread take `INSTANCES_PER_BATCH` until none are left, the last batch shorter (`:281-290`) |
| `prepare.rs:225-226` | `rust-1.96-mutex-boxes-on-first-lock`, `rust-1.96-queue-rwlock-allocation-free` | matches: the batches sit behind a `RwLock`, taken by `write`, no `Mutex` |
| `prepare.rs:258-260` | `bevy-tasks-0.20.1-scope-polls-results-first`, `futures-lite-2.6.1-or-polls-first-future-first`, `async-executor-1.14.0-run-makes-a-runner` | matches: the `spawn_on_scope(async {})` task keeps the scope's results pending at their first poll, so a runner is made in every scope |
| `prepare.rs:487-488` | `concurrent-queue-2.5.0-push-allocates-block` | matches: one pool per `FramePreparer` |
| `prepare.rs:2327-2330` (test) | `futures-lite-2.6.1-block-on-caches-a-parker`, `parking-2.2.1-park-locks-and-waits`, `std-1.96.0-pthread-mutex-boxed-on-first-use`, `std-1.96.0-pthread-condvar-boxed-on-first-use` | matches: the test parks the main thread in `block_on` three times before counting |
| `staging.rs:25`; `readback.rs:35-38` | `wgpu-30-handle-mapping`, `wgpu-30-submit-maintains`, `wgpu-30-map-after-submission` | matches: a map requested once the copy is seen complete is finished by the next submit's maintain; the vendored 30.0.2 lines (`device/life.rs:246`, `:400-404`; `device/queue.rs:1541`) are unchanged |
| `readback.rs:77`, `:704` | `metal-feature-set-tables-sample-buffers`, `apple-metal-fence-per-stage` | matches: one query set of 7 × 28 = 196 timestamps within one sample buffer; a render pass's time counts from the latest end of the passes it waits on (`:715-727`) |
| `culling_pass.rs:79`, `:198` | `wgpu-types-30-default-workgroups-per-dimension`, `wgpu-core-30-dispatch-usage-scope` | matches: 65,535 in 30.0.2's `defaults()`, asserted against the device; the stand-in keeps `frame_draw` out of `append_triangles`'s usage scope |
| `targets.rs:32`, `:57` | `salvi-2012-sbaa-surfaces`, `bt709-6-luma`, `bt2087-rec709-to-xyz` | matches: two surfaces; `[0.2126, 0.7152, 0.0722]` |
| `tonemapping_pass.rs:22` | `peters-blue-noise` | matches, Rust side: the mask compiled in and tested as the asset's red channel; the remap is the shaders' |
| `bind_groups.rs:108-110` | `wgpu-pr-9835`, `wgpu-30-use-tlas-per-flush`, `wgpu-30-submit-tlas-dependencies` | matches: "one per distinct BLAS" holds because the vendored wgpu-core carries the PR's commit (`vendor/wgpu-core/src/command/ray_tracing.rs:235-280`) |
| `shadow_mask_pass.rs:41` | `vulkan-standard-sample-locations` | matches, unchanged |
| `acceleration_structures.rs:23-27`, `:1100-1101` | `pci-vendor-id-nvidia`, `vulkan-vendorid-pci`, `dxgi-adapter-desc2-vendorid`, `wgpu-30-dx12-vendor-id`, `vulkan-vendor-id-mesa`, `lavapipe-vendor-id-mesa` | matches, unchanged: `NVIDIA_VENDOR = 0x10DE` against `AdapterInfo::vendor` |
| `acceleration_structures.rs:29-35` | `wgpu-30-blas-compacted-size-private` | matches: `BlasCompactState::Ready { size }` is `pub(crate)` (`vendor/wgpu-core/src/resource.rs:2519-2525`) |
| `acceleration_structures.rs:53-57`, `:115-121` | `wgpu-hal-30-metal-instances-buffer-unwrap`, `wgpu-30-device-as-hal`, `wgpu-hal-30-build-sizes`, `wgpu-30-blas-size-at-create`, `wgpu-30-tlas-size-at-create`, `wgpu-30-tlas-instance-buffer`, `wgpu-hal-30-metal-instance-bytes`, `wgpu-hal-30-instance-bytes` | matches: the Triangles entry has wgpu-core's fields; the Instances entry has a buffer, as Metal unwraps one (`wgpu-hal-30.0.2/src/metal/conv.rs:390`); the instance buffer is one backend record per entry for at least one entry, as wgpu-core sizes it; Vulkan's record is 64 bytes (`vulkan/adapter.rs:1825`) |
| `acceleration_structures.rs:750`, `:821-828` | `wgpu-30-tlas-instance-staging`, `metal-indirect-instance-descriptor`, `vulkan-acceleration-structure-instance`, `d3d12-raytracing-instance-desc` | matches: wgpu-core stages the filled entries only (`vendor/wgpu-core/src/command/ray_tracing.rs:237-258`), 72 bytes an entry on Metal and 64 elsewhere, so the dynamic TLAS counts `uploaded_entries` |
| `acceleration_structures.rs:908-913` | `wgpu-30-tlas-scratch-per-build`, `gpu-allocator-0.28-chunk-map`, `hashbrown-0.16-rehash-grows` | matches: a scratch sized by the TLAS's `build_scratch_size` per build (`vendor/wgpu-core/src/command/ray_tracing.rs:218-228`, `:323`); `chunks: HashMap`; in-place rehash at half full |
| `acceleration_structures.rs:920`, `:930` | `wgpu-30-command-encoder-pool`, `wgpu-hal-30-metal-command-state` | matches: LIFO `free_encoders.pop()` and `push` (`vendor/wgpu-core/src/command/allocator.rs:38`, `:50`); `CommandState::reset` clears and keeps |
| `acceleration_structures.rs:989`, `:1281` | `wgpu-30-tlas-instance-fields` | matches: an 8-bit mask, hit when it shares a bit with the ray's; the slot as `custom_data` below 2^24, checked for the checked-in Budgets (test `:1283-1287`) |
| `acceleration_structures.rs:1012` | `msl-4.1-sin-fast-accuracy` | matches the quoted 2^-13 within ±π; `BEND_ROUNDING_SHARE` is the last audit's noted estimate, unchanged |
| `blas_refit.rs:4`, `:6`, `:47` | `wgpu-30-metal-refit`, `wgpu-30-blas-rebuild-only`, `wgpu-issue-9215` | matches: `refitAccelerationStructure_...` for `Update` (`wgpu-hal-30.0.2/src/metal/command.rs:1865`); "only rebuild implemented" (`vendor/wgpu-core/src/command/ray_tracing.rs:1116`); Metal's `place_acceleration_structure_barrier` is empty |
| `blas_refit.rs:198-224` | `wgpu-30-as-hal-mut-safety`, `wgpu-30-mixed-encoding-panics`, `wgpu-30-metal-retain-references`, `metal-hazard-tracking-default`, `wgpu-30-metal-buffer-tracked`, `wgpu-30-tlas-blas-build-order`, `wgpu-30-metal-barriers-empty` | matches, each quote found in 30.0.2; the stale premise beside them, L1-R05, is no key's quote |
| `blas_refit.rs:291`, `:306`, `:528`, `:532` | `wgpu-30-metal-scratch-alignment`, `wgpu-30-raw-boxed`, `wgpu-30-metal-event-staging`, `wgpu-30-core-flush-submit` | matches: the static TLAS's worker submits nothing and maps nothing, so the event staging is still consumed by the refit's and the build's submits |
| `src/gpu/mod.rs:5-18` | `wgpu-hal-30-manual-memory-hints`, `gpu-allocator-0.28-personal-block`, `wgpu-hal-30-vulkan-texture-allocation`, `nvidia-vulkan-dos-donts-dedicated-memory` | matches: 16 MiB lies within the `Manual` arm's 4 to 256 MiB clamp (`lib.rs:457-470`); Metal's adapter ignores the hints; a resource above a block gets a personal block; Vulkan textures are `GpuAllocatorManaged`; the comment's 2048² BC7 example is under *Notes for the verifier* |
| `src/gpu/mod.rs:75`, `:288`, `:464`, `:492` | `wgpu-types-30-experimental-features-unsafe`, `wgpu-core-30-anisotropy-clamp`, `wgpu-hal-30-vulkan-downlevel-flags`, `wgpu-core-30-backend-order` | matches |
| `src/gpu/mod.rs:564-574` | `wgpu-types-30-memory-budget-thresholds`, `wgpu-issue-9745`, `wgpu-core-30-lose-if-oom` | matches: DX12 refuses creation past 100% of the reported budget; Vulkan is left without for the quoted fallback; `for_device_loss` unset everywhere, tested `:879-889` |
| `src/gpu/mod.rs:610`, `:819-820` (tests) | `wgpu-hal-30-metal-downlevel-flags`, `vulkan-max-storage-buffer-range`, `mesa-25.2.8-gallivm-max-shader-buffer-size` | matches: indirect execution by GPU family, anisotropy always; 2^27 bytes both |

`mod.rs`, `instance.rs`, `uniforms.rs`, `staging.rs` past `:25`,
`main_pass.rs`, `froxel_pass.rs`, `light_count_pass.rs`,
`depth_pyramid_pass.rs` and `bend_pass.rs` cite no key.

### Findings

| id | claim | severity | where | evidence | expected | verifier | placement |
|---|---|---|---|---|---|---|---|
| L1-R01 | A Renderable that needs a new slot is refused, warned "the Instance buffer is full" and counted once Chunk ranges have carried the highest slot taken to the capacity and no Renderable has freed one, though the free ranges unloaded Chunks gave back hold free slots. | `correctness`; clause "past which a Renderable is refused"; no Reference scene reaches it at shipped capacities: `world`'s timed frames hold 307,263 slots in use (`docs/performance.md:330-331`) of 488,448 | `prepare.rs:330-331` | block below | ADR 0003: "the buffer holds a fixed number of slots, past which a Renderable is refused, warned about once, counted in Frame stats and retried every frame"; the struct's own doc, `prepare.rs:297-298`: "once all `capacity` are taken a Renderable is refused one" | | |
| L1-R02 | A static TLAS build uploads every filled entry's instance record in the frame that submits it, and Frame stats' `uploaded_bytes` counts none of it, while the dynamic TLAS's records, staged the same way, are counted. | `correctness`; one counter misses one kind of work: up to 264,720 × 72 = 19,059,840 bytes per static build on Metal at the checked-in capacity (64 bytes an entry elsewhere), in every frame that submits a build | `acceleration_structures.rs:544-552`; `renderer.rs:712-723` | block below | `CLAUDE.md` checklist: "per-frame GPU writes add to `uploaded_bytes`"; the dynamic TLAS's own rule, `acceleration_structures.rs:748-751`, `:884-885` | | |
| L1-R03 | ADRs 0003 and 0017 give the Instance slots 136.2 bytes a slot, and ADR 0003 517,376 slots at the M4 Budgets, where the code takes 144.2 bytes a slot, four offsets per slot since Thin Primitives' twin draws, and gives 488,448 slots at the same Budgets. | `drift`; not load-bearing for code, which derives the figure; misleads a reader by 28,928 slots (5.6%), which `docs/research/vegetation-and-grass.md` and `docs/audit/m5/scalability.md` repeat | `renderer.rs:215-218`, `:259-261`; `docs/adr/0003-persistent-scene-transform-buffer.md:3`; `docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:18` | block below | ADRs 0003 and 0017 state the bytes a slot and the slot count the code derives: 144.2 and 488,448 at the checked-in Budgets | | |
| L1-R04 | ADR 0017 says ADR 0013 leaves the load staging out of GPU memory, but ADR 0013's list of what `gpu_memory_bytes` leaves out, and the `CLAUDE.md` checklist's, do not name it; the load staging's belt buffers, written in every streaming frame since M4, bypass the ledger with no ADR 0013 clause recording it. | `drift`; load-bearing for the staging Budget's reading: `from_budgets` reserves the staging Budget past the frame staging for the load staging, which the `Staging` kind of `gpu_memory_bytes` can never show | `docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:18`; `staging.rs:179-184` | block below | ADR 0013's exclusions name the load staging, or the load staging is counted under `Staging`; M4 story 44: "so that no buffer or texture escapes `gpu_memory_bytes`" | | |
| L1-R05 | The refit's SAFETY comment rests one premise on "only the main thread calls into wgpu meanwhile (ADR 0016)", which is false since the static TLAS's worker records and finishes builds while `render` records the refits, and which ADR 0016 does not say. | `drift`; one premise of one `unsafe` block's SAFETY comment; the block stays sound, as the worker destroys none of the objects the refit holds unguarded | `blas_refit.rs:193-196` | block below | ADR 0016: "Only the main thread submits, presents or polls ... Workers record `CommandEncoder`s and return `CommandBuffer`s"; the premise states what keeps the unguarded objects alive, that nothing destroys them | | |
| L1-R06 | ADR 0021's fourth Consequence says the floor's error names each attachment format without 4x MSAA "or, on colour, its resolve", but since the floor asks the resolve of the HDR format alone it never names the surface normals' `Rgba8Unorm`, a colour attachment format, for its resolve. | `drift`; one ADR sentence contradicting the ADR's own amended first Consequence; zero adapters affected, as wgpu-types 30.0.2 guarantees `MULTISAMPLE_RESOLVE` on `Rgba8Unorm` | `docs/adr/0021-capability-floor.md:17` | block below | the sentence says "or, on the HDR format, its resolve", as `:14` and `src/gpu/mod.rs:363-374` do | | |
| L1-R07 | The doc comment of `adapter_report` says the capability report checks 4x MSAA on the main pass's attachment formats "resolved on colour ones", but the report asks the resolve of the HDR format alone, so it prints `Rgba8Unorm yes` without checking its resolve. | `drift`; one doc comment; no adapter affected, as L1-R06 | `src/gpu/mod.rs:143` | block below | the comment says "resolved on the HDR format", as `floor_format_features` decides | | |

#### L1-R01

Code, `prepare.rs:326-340`:

```rust
fn get_or_take(&mut self, entity: Entity) -> Option<u32> {
    if let Some(&slot) = self.by_entity.get(&entity) {
        return Some(slot);
    }
    let fresh = (self.end < self.capacity).then_some(self.end);
    let Some(slot) = self.free.pop().or(fresh) else {
        if self.refused.insert(entity) {
            tracing::warn!(
                ?entity,
                capacity = self.capacity,
                "Renderable refused: the Instance buffer is full"
            );
        }
        return None;
    };
```

`free` holds only slots a Renderable gave back (`:425-430` push into
`freed`, which `:844` appends to `free`). A Chunk's range, once it
unloads, goes to `freed_ranges` (`:391-402`) and then `free_ranges`
(`:406-423`), which only `take_range` reads (`:351-366`); `end` never
falls. So once Chunks have taken fresh ranges up to `capacity`
(`take_range`'s fresh path, `:367-386`, sets `end` to the range's end, up
to `capacity` inclusive) and the World holds no slot a Renderable freed,
every new Renderable is refused, warned that the buffer is full and
counted in `renderables_refused` (`:784-787`), while `free_ranges` may
hold every slot of the Chunks unloaded since. It is retried every frame
but takes a slot only when a Renderable frees one; a free range is never
offered to it.

ADR 0003: "each Renderable keeps a slot ... and a freed slot is taken
again before a fresh one; the buffer holds a fixed number of slots, past
which a Renderable is refused, warned about once, counted in Frame stats
and retried every frame", and of ranges: "A range is the first free range
long enough, else fresh slots past the highest taken ... An entity's slot
is still per entity". The amendment that added ranges does not say a
Renderable may not take a slot of a free range, and the struct's doc
(`:294-298`) says a Renderable is refused "once all `capacity` are
taken". No test covers it:
`a_renderable_past_capacity_is_refused_and_warned_about_once_until_a_slot_frees`
(`:1268-1303`) fills the buffer with Renderables alone, and
`a_chunk_past_capacity_is_refused_whole_and_warned_about_once`
(`:1310-1354`) refuses a Chunk, not a Renderable. Not visible on any
Reference scene: no benched run is shown to carry `end` to the capacity
(488,448 slots at the checked-in Budgets, test `renderer.rs:1092`), and a
refused Renderable draws nothing rather than drawing wrong. Either
`get_or_take` falls back to the first free range's first slot, shrinking
it as `take_range` does, or ADR 0003 and the doc say a Renderable never
takes a Chunk's freed slots, which makes this `drift`.

#### L1-R02

Code, `acceleration_structures.rs:544-552`, where the main thread takes
the worker's build:

```rust
self.spare = Some(match work.build {
    Some(build) => {
        queue.submit(std::iter::once(build));
        update.build_time = Some(work.time);
        Spare::Submitted {
            tlas: work.tlas,
            built_from,
        }
    }
```

`StaticTlasUpdate` (`:319-329`) carries the worker's time and
`frames_until_traced`, and `renderer.rs:712-723` copies those and
`static_instances_left_out` into the counters; nothing adds bytes. The
worker's build (`:605-615`) builds the whole TLAS: wgpu-core converts and
stages every filled entry [ref:wgpu-30-tlas-instance-staging], the quote
being `instance_buffer_staging_source.extend(state.device.raw().tlas_instance_to_bytes(`,
over `package.instances.iter().flatten()`
(`vendor/wgpu-core/src/command/ray_tracing.rs:237-258`), into a staging
buffer copied on the build's command buffer (`:386-392`). The dynamic
TLAS counts exactly this for its own build: `entry_upload_bytes`, "Bytes
each entry uploads to the GPU per build: the backend's own instance
record, which wgpu-core converts it to and stages" (`:748-751`), returned
by `record` and added at `renderer.rs:851-856`. The field's doc,
`src/stats/mod.rs:25-31`, names "the dynamic TLAS's instances its build
uploads" and not the static TLAS's. The `CLAUDE.md` checklist: "per-frame
GPU writes add to `uploaded_bytes`".

Magnitude: the static TLAS holds 264,720 entries at the checked-in
Budgets (test `renderer.rs:1094`), so a full build stages 19,059,840
bytes on Metal (72 a record, `acceleration_structures.rs:823`) and
16,942,080 elsewhere, against `load_bytes_per_frame` of 26,640,177; ADR
0020 records 402 builds over `world`'s bench segment before Far proxies'
changes were held back. At the last audit's commit the static build ran
only at load and on a spawn and was not counted either; what landed
since makes it recur with every Chunk streamed, every re-anchor and every
120-frame Far proxy build, in frames the worker's timing picks. That
timing may be why the omission is deliberate, since a counter that moves
with the worker would break counter equality between runs, but no
comment or doc records it. The fix is either to add the bytes in the
frame that submits the build, or to name the exclusion and its reason in
the counter's doc. Not visible: nothing drawn changes.

#### L1-R03

Code, `renderer.rs:213-218`:

```rust
/// Bytes a scan block of Instance slots takes: each slot's Instance, offsets
/// and visibility bit, and the block's offsets.
const INSTANCE_SCAN_BLOCK_BYTES: u64 = SCAN_BLOCK_SIZE as u64
    * (size_of::<InstanceData>() + size_of::<[u32; 4]>()) as u64
    + size_of::<[u32; 4]>() as u64
    + SCAN_BLOCK_SIZE as u64 / u8::BITS as u64;
```

`InstanceData` is 128 bytes (`instance.rs:31-40`), so a block is
256 × (128 + 16) + 16 + 32 = 36,912 bytes, 144.19 a slot. `from_budgets`
(`:259-261`) takes 30% of the scene and pass buffers' Budget less two
16-byte entries in whole blocks, and the test pins the result at the
checked-in Budgets, `renderer.rs:1092`: `instances: 488_448`. The offsets
grew from two `u32`s to four with Thin Primitives' twin draws, which
changed `[u32; 2]` to `[u32; 4]` here and `517_376` to `488_448` in the
test; ADR 0027's Consequences record that the twins' packed indices share
the frame's draw's buffer, but no ADR records what the change cost a
slot.

ADR 0003, `docs/adr/0003-persistent-scene-transform-buffer.md:3`: "the
number is sized at device creation from the scene and pass buffers' GPU
memory Budget ..., 30% of it at 136.2 bytes a slot, its Instance, offsets
and visibility bit, in whole 256-slot scan blocks of the culling pass:
517,376 slots at the Budgets M4 set". 136.2 is 128 + 8 + 1/8 + 16/256,
the two-offset layout. ADR 0017,
`docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:18`:
"the Instance slots 30% of the scene and pass buffers', at 136.2 bytes a
slot with its offsets and visibility bit, in whole scan blocks". The
Budgets are M4's (the share and the Budget are unchanged), so both
figures are false at the frozen commit. The capacity itself was put to
the owner when the offsets grew, so the number is decided; the ADRs were
not amended. The code is consistent with itself (the culling pass's
offset buffer is four `u32`s per slot plus per block plus two,
`culling_pass.rs:244-249`, and `tests/frame_stats.rs:173-176` sums the
same), and no `deviations.md` row covers an ADR's figure. The M5 source
check saw the same sentence and left it to the docs and rules drift lane
(`docs/audit/m5/sources.md:1000-1002`) without filing it; it is filed
here because the clause is a decision clause of ADR 0003 that this lane
gives a verdict on. Not visible: nothing reads the ADR's figure at run
time.

#### L1-R04

ADR 0017, `docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:18`,
amended 2026-11-05: "The Renderer and its Pools create the buffers,
textures and TLASes they count through one ledger ... and
`gpu_memory_bytes` is the sum of those totals. What ADR 0013 leaves out,
the GPU stats readback and the load staging, bypasses it". ADR 0013,
`docs/adr/0013-reference-hardware-and-provisional-budgets.md:3`: "the GPU
stats readback's buffers are left out of that sum, since they measure the
frame rather than hold it ...; so are the target the frame is presented
into, a window's surface textures or capture's stand-in for them, and
capture's read-back buffer". The load staging is not in that list. The
`CLAUDE.md` checklist: "GPU buffers, textures and acceleration structures
are created through the GPU memory ledger (`GpuMemory`) under their kind
...; the GPU stats readback's buffers stay out, as they measure the frame
(ADR 0013)".

Code, `staging.rs:179-184`: "Staging for load-time uploads (ADR 0017):
each write goes through a `StagingBelt` ... The belt goes with it, so its
staging buffers live only until the GPU has run the copies and never
count in `gpu_memory_bytes`", and `:344-353` creates the belt with
`wgpu::util::StagingBelt::new(device.clone(), LOAD_CHUNK_BYTES)`, outside
`GpuMemory`. Since M4 the streamer writes through it in every frame that
loads a Chunk, a Far proxy or a mip, and `RenderCapacities::from_budgets`
sizes its per-frame cap as the staging Budget past the frame staging's
slots, over the frames in flight plus one (`renderer.rs:266-275`): the
staging Budget is set to hold it, while the `Staging` kind the Budget is
compared against counts only the frame staging's slots
(`staging.rs:140-149`). The code follows ADR 0017; the defect is ADR
0017's statement about ADR 0013, which ADR 0013's text does not bear out,
and the checklist that names one exception. The fix is an ADR 0013
amendment naming the load staging among what `gpu_memory_bytes` leaves
out, with the checklist line, or counting the belt's chunks under
`Staging`.

#### L1-R05

Code, `blas_refit.rs:190-196`:

```rust
// SAFETY: wgpu-hal's refit is sound under these invariants, each
// held here:
// - Every handle is Metal's: `new` refuses the refit off Metal.
// - Every wgpu-hal object outlives its use past its guard
//   (`metal::unguarded`): the Mesh pool, this refit and `blases`
//   hold their handles throughout, nothing destroys them, and only
//   the main thread calls into wgpu meanwhile (ADR 0016).
```

Since the static TLAS's worker landed, `StaticTlas::start` hands the
spare to a `std::thread` that calls `device.create_command_encoder`,
`build_acceleration_structures` and `finish`
(`acceleration_structures.rs:592-622`), and replaces TLAS entries,
dropping the `TlasInstance`s it replaces (`:713-716`). `start` runs in
`Renderer::upload` (`renderer.rs:712-718`), and without
`wait_for_static_tlas`, as the window and the bench run, the worker is
collected only by a later frame's `update`
(`acceleration_structures.rs:531-537`), so it runs while
`Renderer::render` records the refits under the comment
(`renderer.rs:826-843`). ADR 0016 does not claim what the comment cites
it for: "Only the main thread submits, presents or polls the wgpu `Queue`
and `Device` ... Workers record `CommandEncoder`s and return
`CommandBuffer`s". The block stays sound: what `unguarded` needs is that
the resource "is neither destroyed nor dropped while the reference lives"
(`blas_refit.rs:298-301`), and the worker destroys none of the Arenas,
refitted BLASes or scratch, since the static TLAS holds no Moving
Instance's BLAS and wgpu-core snatches a raw resource only on `destroy`.
The neighbouring premise in `Order::stage`, "only the main thread submits
(ADR 0016)" (`:526-532`), still holds: the worker submits nothing. Not
visible; the fix is the comment's clause, replaced by one that says no
thread destroys them.

#### L1-R06

Code, `src/gpu/mod.rs:363-374`:

```rust
/// The format features the floor requires of a main-pass attachment format:
/// 4x MSAA, and on the HDR format its resolve, since the lit pass resolves
/// its multisampled colour into the HDR target (ADR 0019). No other target
/// is resolved.
fn floor_format_features(format: wgpu::TextureFormat) -> wgpu::TextureFormatFeatureFlags {
    let msaa_x4 = wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4;
    if format == crate::render::HDR_FORMAT {
        msaa_x4 | wgpu::TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE
    } else {
        msaa_x4
    }
}
```

`check_floor` (`:345-349`) names a format in `missing_format_features`
with what `floor_format_features` asks of it and the adapter lacks, so
the typed error can name a missing resolve on `Rgba16Float` only; the
test `the_surface_normals_format_needs_no_resolve` (`:722-748`) pins it.
ADR 0021, Consequences, fourth bullet
(`docs/adr/0021-capability-floor.md:17`): "`GpuContext` checks the whole
floor before requesting the device and returns one typed error naming
each missing feature and downlevel flag, each attachment format without
4x MSAA or, on colour, its resolve, and each limit with its required and
adapter values". The first bullet (`:14`), amended in scope: "with its
resolve (`MULTISAMPLE_RESOLVE`) on the HDR colour format ... Those
formats are `Depth32Float`, the HDR target's `Rgba16Float` and the
surface normals' `Rgba8Unorm`, which the depth passes write with shadow
rays on and nothing resolves, so the floor checks no resolve on it". At
`8bc34eb1`, `floor_format_features` asked the resolve of every non-depth
format, so "on colour" matched; the change in scope that narrowed it to
the HDR format amended the first bullet and left the fourth, which now
contradicts it within the ADR. Not visible: the code is right and the
first bullet says so; wgpu-types 30.0.2 guarantees `msaa_resolve` on
`Rgba8Unorm` (`texture/format.rs:975`), so no adapter's error would
differ.

#### L1-R07

Comment, `src/gpu/mod.rs:139-144`:

```rust
/// The adapter, its backend, and whether the adapter supports each
/// feature the roadmap plans on, with the related features and limits.
/// One line per entry, followed by two of the Capability floor's limits, the
/// buffer size and the acceleration structures per stage, then 4x MSAA on
/// the main pass's attachment formats, resolved on colour ones, and
/// anisotropic filtering.
```

beside the code it describes, `:230-244`, which prints each of
`MULTISAMPLED_FORMATS` as `yes` when `floor_format_features(format) -
flags` is empty, and `floor_format_features` asks `MULTISAMPLE_RESOLVE`
of the HDR format alone (`:367-374`). `Rgba8Unorm` is a colour format of
`MULTISAMPLED_FORMATS` (`targets.rs:51-52`) whose resolve the report
never checks, so "resolved on colour ones" misdescribes the line it
prints. The comment was last edited before `8bc34eb1`; the changes in
scope that added `Rgba8Unorm` to the multisampled formats and narrowed
the resolve to the HDR format made it false. One fix serves L1-R06 and
L1-R07. Not visible.

### Notes for the verifier

Not filed, since each needs a run, an owner's reading or another lane;
recorded so the next audit does not re-find them.

- The spare static TLAS is rebuilt while frames still in flight may trace
  it: under `wait_for_static_tlas` the build is submitted in the very
  frame that unbinds it (`acceleration_structures.rs:407-422`), and the
  GPU may still be running the frame before's shadow mask pass, which
  read it. Metal's `place_acceleration_structure_barrier` is empty
  (`wgpu-hal-30.0.2/src/metal/command.rs:1877-1881`), so the order rests
  on Metal tracking hazards on the `MTLAccelerationStructure` across
  command buffers, which [ref:wgpu-issue-9215] doubts for builds. Run:
  Metal API validation on a `world` traversal under `wait_for_static_tlas`,
  or a capture that re-anchors every frame, against the same capture with
  a `device.poll(Wait)` before each static build's submit.
- `entry_changes` asserts a static Instance's BLAS is ready in debug
  builds only (`acceleration_structures.rs:454-458`); a release build that
  placed one early would put a BLAS not yet built or compacted into the
  build. Whether streaming can place one early is the streaming reader's.
- M4 story 43 asks for TLAS entries "not a compile-time constant", while
  `DYNAMIC_TLAS_CAPACITY = 10_000` (`acceleration_structures.rs:17`)
  stays one, as ADR 0020 caps it, and `docs/audit/m5/scalability.md` reads
  it as surviving; `BEND_CAPACITY` (`bend_pass.rs:22`) is the same case.
  An owner's reading whether the story meant the static TLAS alone.
- `compacted_blas_fraction` gives a non-NVIDIA Vulkan GPU lavapipe's 1.0
  and a non-NVIDIA DX12 GPU the M4 Pro's 45.5/94.0
  (`acceleration_structures.rs:36-51`), as its doc says; whether the
  checklist's "its adapter's estimated fraction" allows a stand-in
  adapter's figure is an owner's reading.
- A Mesh with Thin Primitives and other Primitives takes the ordinary
  mask (`src/assets/pools.rs:862-869`, `acceleration_structures.rs:990-994`),
  so its thin parts cast at every receiver width, short of M5 story 37.
  Whether the cook allows such a Mesh is the cook reader's.
- The doc of `StaticTlas::update` says that with `wait` "no frame depends
  on the worker's timing or on the frames before it"
  (`acceleration_structures.rs:383-384`); the bound Anchor still depends
  on the camera's path within 4 cells, which a capture fixes. An M4 story
  65 run (the same Chunk at the origin and the far corner) would show
  whether it moves a bit of an image.
- Lost-upload recovery (`renderer.rs:783-790`, `src/state.rs:301-303`,
  `prepare.rs:681-693`, `:836-843`) still has no test that induces a
  failed staging map, as the last audit noted. Two cases are new: with
  Chunk ranges a slot can be written twice in one Frame data, the later
  write landing last (`staging.rs:119-127`, the static TLAS reading writes
  in order, `acceleration_structures.rs:448-450`); and after a re-anchor
  the failed frame keeps the previous camera uniform, its `tlas_anchor`
  included, while the shadow mask binds the newly bound TLAS
  (`renderer.rs:915-921`), one frame of shadows in the wrong frame. Run:
  force `FrameStaging::write` to return 0 for one frame of `world` after a
  Chunk unloads, and again in the frame after a re-anchor, and compare
  the next frame's capture with an unbroken run.
- With the crowd's refit form, a frame whose refits are refused (no
  scratch: "an Arena is not Metal's", `blas_refit.rs:180-185`) still splits
  the submission after the bend pass and brackets the dynamic TLAS's
  build, so `frame_times` takes the build as spanned
  (`readback.rs:667-677`) and may read it as zero though it ran. A run
  that forces `metal::scratch` to return `None` under `crowd` with the
  refit form would show whether `dynamic_tlas_build` reads zero there.
- A Chunk refused a range still has its point lights drawn:
  `prepare_lights` reads every Chunk not held (`prepare.rs:1020-1024`),
  refused or not. ADR 0003 says a loaded Chunk's "point lights join the
  frame's lights" and is silent on a refused one; an owner's reading.
- A GPU stats slot whose map fails is logged and freed but not counted in
  `gpu_readbacks_skipped` (`readback.rs:503-506`); the module doc counts
  skips of a slot "not ready" (`:5-7`), which a failed map is not. No run
  has shown a failed map.
- The camera uniform's `dynamic_tlas_entries` is the Frame data's Moving
  Instance count (`renderer.rs:737-738`), not the dynamic TLAS's filled
  entries: a Moving Instance whose Mesh has no BLAS fills no entry
  (`acceleration_structures.rs:860-879`), so the count can be non-zero
  with an empty TLAS. The shader only tests it against zero
  (`uniforms.rs:51-52`), so a ray then traces an empty TLAS and finds
  nothing; no image or counter changes.
- `uploaded_bytes`'s doc (`src/stats/mod.rs:25-31`) ends "The pools' own
  loads are not counted", while `src/state.rs:287` adds the streamer's
  `written_bytes`, the loads it writes into the pools through the load
  staging, to it. Whether "the pools' own loads" means only those at a
  scene's load is the tooling and streaming readers' to judge; the stats
  doc is the tooling module's.
- The load staging's belt allocates chunks on demand at run time, which
  ADR 0017 permits ("it allocates more on demand"), while the checklist
  says "Only per-frame staging grows"; the checklist's wording lags ADR
  0017. The docs and rules drift lane's.
- ADR 0007's exit says "on 2026-11-14 a copy of wgpu-core 30.0.1 carries
  one upstream fix"; the copy became 30.0.2 on 2026-11-15, which updated
  `vendor/README.md`, `CLAUDE.md` and `docs/agents/project-shape.md` but
  not the ADR. The sentence is dated, so it holds as dated. The README's
  "the only difference is the commit above" could be checked only against
  30.0.1, whose `ray_tracing.rs` is the vendored one less exactly the
  commit; the other seven differing files carry the changes the bump
  names. Run: download `wgpu-core` 30.0.2 from crates.io and `diff -r` it
  with `vendor/wgpu-core`. Every registry URL for a wgpu 30 quote still
  names v30.0.1, though each quote reads the same in 30.0.2.
- ADR 0019 says "a compute pass appends the Meshlets ... in two phases";
  each phase is now two wgpu compute passes, the per-slot steps and
  `append_triangles`, so Frame stats time them apart
  (`culling_pass.rs:16-20`, `:340-388`). Whether "a compute pass" names
  the culling pass as a whole is a reading; `CONTEXT.md` does not define
  the culling pass.
- The shadow mask pass's compute stage now binds nine storage buffers,
  the floor exactly, since `instance_offsets` joined `SHADOW_MASK_BUILD`
  (`bind_groups.rs:227-234`); ADR 0021 names only the culling pass as the
  reason for nine. Not a defect; a tenth buffer in either pass raises the
  floor.
- The second-surface and shadow-edge lists hold (pixels + 1) × 4 bytes
  and are bound whole (`targets.rs:165-171`); at 8192 × 4096 pixels or
  more they pass the floor's 128 MiB `max_storage_buffer_binding_size`,
  so a resize there fails in wgpu's validation rather than being refused.
  7680 × 4320 fits. Reachable only past 8K UHD, like the last audit's
  note on `height <= 1 << 14`. Run: a headless capture at 8192 × 4096.
- `src/gpu/mod.rs:8-9`: "a 2048² BC7 Texture with its mips still shares
  one, so Textures cost few allocations". Textures now live in Texture
  class arrays created at the class's slice count
  (`src/assets/material.rs:430-448`), so whether a slice shares a 16 MiB
  block depends on its class's size, not its own; the conclusion, few
  allocations, still holds. Run: `generate_allocator_report` on Vulkan
  after `town` settles.
- `RenderCapacities`'s doc (`renderer.rs:225-232`) cites ADR 0013 for the
  shares, which ADR 0017 writes (with ADRs 0003 and 0019), while ADR 0013
  records the Budgets themselves. The docs and rules drift lane's.
- `bend_pass.rs:111` dispatches `longest.div_ceil(64)` workgroups in x
  with no check against 65,535, so a bent Mesh past 4,194,240 vertices
  would fail; the line is unchanged since `8bc34eb1`.
- With Far proxies changing each frame and no build due, `start` spawns
  an OS thread per frame to catch the spare up
  (`acceleration_structures.rs:563-625`), and wgpu-core's per-build
  scratch and instance staging live outside the GPU memory ledger: costs
  for the performance and scalability lanes, not clauses.

### Counts

| Severity class | Findings |
|---|---|
| `image` | 0 |
| `budget` | 0 |
| `correctness` | 2 (L1-R01, L1-R02) |
| `drift` | 5 (L1-R03, L1-R04, L1-R05, L1-R06, L1-R07) |
| `health` | 0 |

Before the verifier. Clause verdicts: 128 rows, of which 6 are
`broken` and 1 holds in code with the ADRs disagreeing, each naming its
Finding; 5 fall in shaders or another module and 1 is not judged from
code; none is `superseded`. Spec stories: 40 rows. Registry keys: 84 keys
in 29 rows, all matching their quotes.
