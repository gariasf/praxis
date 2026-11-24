# Audit lane 1, correctness against intent

The lane's report, in the form `docs/agents/audit.md` gives: every ADR's
decision clauses read against the code at the frozen commit
`e5c55a8da2a59db2f09c99c8ea4522987d5354ac`, a verdict per clause, the
stories of the M4 and M5 specs, the oracles added since `M3.5`'s frozen
commit `8bc34eb18fcfb79307193551e96619316aef566c` and every registry key
cited in code that landed since checked the same way. Five readers, one per
module, each a section below, appended in the order they were written; the
lane's report session wrote the Summary, the counts, the merged table of
decision clauses and the Killed section, and ran the verifier over every
section. The verifier's verdicts fill each Findings table's `verifier`
column; each section's clause rows and Counts stay as its readers wrote
them, before the verifier. Finding ids carry the module's
letter so that sections written in parallel never collide: render
`L1-R<n>`, shaders `L1-S<n>`, streaming with the frame `L1-F<n>`, cook with
assets and scenes `L1-C<n>`, tooling `L1-T<n>`.

## Summary

Five readers, one per module, read every ADR's decision clauses, the stories of the M4 and M5 specs, the oracles added since `M3.5` and every registry key cited in code that landed since against the code at the frozen commit, and filed 50 Findings; the verifier reopened every one and the 22 clause rows they break, confirmed 45 (L1-C14 after a run the session made at the frozen commit), narrowed 2 and killed 3, and each of the 538 decision clauses of ADRs 0001 to 0027 has a verdict in one merged table, 19 of them naming a break in the clause or in a comment or document on it. Seventeen are `correctness`, code that breaks a clause or a story with nothing visible: among them a mip plan that sends more requests into a Texture class than it has room for, a stale entry that makes the IO pool drop an Asset's next fresh copy, a fixed-schedule system that reads the camera, a Renderable refused while unloaded Chunks' slots stand free, a static TLAS upload `uploaded_bytes` never counts, small Instances that keep less than their mean coverage off the view axis, re-recorded Baselines imported untagged and without their scene's content hash, and a mip move that can open a Texture class wgpu refuses with a panic. Twenty-nine are `drift` and one is `health`; no Finding shows in an image, no `image` or `budget` Finding was filed, none repeats one of `M3.5`'s, and every registry key cited in scope matches its quote but `unity-6000-uv-distribution-metric`, whose per-Mesh average the density takes per Primitive with no Deviation row (L1-C10).

## Counts

After the verifier and the session's run for L1-C14. Each module
section's own Counts are its readers', before the verifier; no Finding
changed class, and the three killed are counted only among the verdicts.

| Severity class | Findings |
|---|---|
| `image` | 0 |
| `budget` | 0 |
| `correctness` | 17 (L1-F01, L1-F02, L1-F03, L1-F41, L1-R01, L1-R02, L1-S02, L1-S04, L1-S10, L1-T01, L1-T02, L1-T03, L1-T04, L1-C04, L1-C05, L1-C07, L1-C08) |
| `drift` | 29 (L1-R03, L1-R04, L1-R05, L1-R06, L1-R07, L1-S01, L1-S03, L1-S06, L1-S08, L1-S09, L1-S11, L1-S12, L1-S13, L1-T05, L1-T06, L1-T07, L1-T08, L1-T09, L1-T10, L1-T11, L1-C01, L1-C02, L1-C03, L1-C09, L1-C10, L1-C11, L1-C12, L1-C13, L1-C14) |
| `health` | 1 (L1-C15) |

| Verifier verdict | Findings |
|---|---|
| `confirmed` | 45 (L1-C14 after the run the verifier asked for: frame 0 lies 39.6 m inside settlement [4, 5]) |
| `narrowed` | 2 (L1-F02: the refused mip move is logged at `error`, only uncounted; L1-R03: takes L1-S05's research-note location and its account of how the offsets widened) |
| `killed` | 3 (L1-S05, a duplicate of L1-R03; L1-S07; L1-C06) |

The verifier also reopened the 22 clause rows whose verdict says
`broken`: every one stands, two reworded to name L1-R03 for the killed
L1-S05. The merged clause table below has 538 rows over the 27 ADRs: 504
hold; 19 name a break in the clause or in a comment or document on it,
carried by 15 Findings and one known open issue; 12 are not applicable (a
plan not yet built, or a clause no code carries); 2 are not judged from
code (ADR 0007's two reopen triggers); 1 is not implemented (ADR 0013's
Input latency Budget); none is superseded. The session read the 36 clauses
no reader took: 30 hold and 6 are not applicable. Spec story rows: 271
over the five sections (streaming 64, render 40, shaders 34, tooling 34,
cook 99), covering all 71 stories of the M4 spec and all 42 of the M5
spec. Oracle rows: 55 (shaders 16, cook 39), none `unexplained`.
The triage with the owner on 2026-11-24 filled `placement` on every row:
43 Findings in fix tickets, 2 fog, 2 accepted Deviations and the 3 killed
wontfix.

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
| L1-F01 | `MipPriority::plan` checks each request's room against the Channel as it stands, without counting the room the same plan's earlier requests will take, so N requests into one class with room, free or evictable, for fewer than N moves are all made, and the excess reads are discarded when applied. | `correctness`; clause "a request is made only when its move finds room" broken whenever more requests in one plan target a class than it has room for; cost a whole Texture Blob read per excess request (5.3 MiB for a 2,048² BC7 chain), nothing visible | `mips.rs:233-242` | block below | ADR 0017: "a request is made only when its move finds room, free or by an eviction" | `confirmed`: `src/streaming/mips.rs:233-242` keeps every request whose `full_class_for_mips` is `None`, and that is `!channel.table.has_room(&[shape])` for the one shape (`src/assets/material.rs:1150-1151`), never decremented within the plan; the evictable-needs map is built once (`.get_or_insert_with(\|\| materials.coarsest_evictable_needs(...))`) and every kept request goes to the IO pool (`src/streaming/io.rs:1121-1123`), against ADR 0017's "a request is made only when its move finds room, free or by an eviction" (`docs/adr/0017-...md:16`). | fix #1028 |
| L1-F02 | A mip move whose Channel has no room when its read is applied is refused by the Material pool and the refusal is dropped: `MipStreaming::apply` ignores `stream_mips`'s `StreamedMips::Refused`, so it is neither logged nor counted in Frame stats. | `correctness`; M4 spec story 47 broken for one kind of refusal, mip moves; the other refusals (Meshes, Materials, Chunk loads) are logged and counted | `io.rs:1155-1162`; `src/assets/material.rs:1061-1063` | block below | M4 spec, story 47: "a request past any capacity refused, logged and counted in Frame stats, so that an overflow is visible, never silent" | `narrowed`: Rewritten: a mip move refused at apply is logged but not counted in Frame stats: `MipStreaming::apply` drops `stream_mips`'s `StreamedMips::Refused` (`src/streaming/io.rs:1155-1162`) and the pool's only refusal count is the Material table's (`src/assets/material.rs:682`, read by `take_refusals` at `src/streaming/io.rs:925`). "Neither logged" is false: the refusal comes from `channel.place` returning `None` (`src/assets/material.rs:1061`), and `ChannelTextures::place` logs it at `error`, "channel has no room for another Texture class" (`src/assets/material.rs:419-425`); M4 story 47's "counted in Frame stats" alone is broken. A count where `place` refuses would also serve `L1-C07`'s count. | fix #1028 |
| L1-F03 | An Asset the main thread read from the Cooked cache after a failed read or a refused load left it claimed stays in `read_from_cache` forever, since no decoded copy of it ever comes; once the pools free it and a later IO read decodes it afresh, that copy is dropped and the main thread reads the Blob itself. | `correctness`; clause "the main thread reads one only after a failed read or a refused load left it claimed" broken once per such Asset after its first refusal or failed read; cost one main-thread Blob read and a needless wait | `io.rs:54-70`, `:597-604`, `:636-641` | block below | ADR 0017: "so the main thread reads one only after a failed read or a refused load left it claimed" | `confirmed`: `DecodedReader::claimed` inserts into `read_from_cache` whenever the main thread reads an Asset another read claimed (`src/streaming/io.rs:602-604`), the only removal is `extend`'s `!read_from_cache.remove(id)` (`:62-63`), and `forget` clears the claim alone (`:636-641`); `decode_chunk` claims each Asset before reading it (`:452-457`), so a read that fails later leaves a claim with no copy, and the stale entry drops the next fresh IO copy, against ADR 0017's "the main thread reads one only after a failed read or a refused load left it claimed" (`docs/adr/0017-...md:19`). The stale entry also re-arms itself: the next main-thread read under that load's claim inserts it again. | fix #1028 |
| L1-F41 | The fixed schedule runs `wrap_around_camera`, which reads the camera that the per-frame schedule moves by the wall-clock frame delta, so the Tick on which a `world` flock jumps a 320 m span, and so the simulation state after that Tick, depends on frame timing. | `correctness`; clause broken: ADR 0022's determinism ("whatever the frame timing"); 100 flock centres and their 10,000 birds in `world`, in the window only | `src/state.rs:410`; system at `src/scenes/mod.rs:921-934` | block below | ADR 0022: "the same binary on the same machine, fed the same per-Tick inputs, reaches a bit-identical simulation state whatever the frame timing"; "Input sampling, the camera, the blend, prepare and presentation-only effects run once per frame" | `confirmed`: `wrap_around_camera` sits in the fixed chain (`src/state.rs:410`) and reads `Res<Camera>` every Tick (`src/scenes/mod.rs:921-934`), while `simulate` runs the Ticks and then the frame schedule that moves the camera by the wall-clock delta (`src/state.rs:148-151`); ADR 0022 lists "the camera" among what runs once per frame (`docs/adr/0022-...md:17`) and promises a bit-identical state "whatever the frame timing" (`:20`). The system landed after `8bc34eb1`, so `spawn_helmet`'s older camera read does not shield it. | fix #1027 |

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
| L1-R01 | A Renderable that needs a new slot is refused, warned "the Instance buffer is full" and counted once Chunk ranges have carried the highest slot taken to the capacity and no Renderable has freed one, though the free ranges unloaded Chunks gave back hold free slots. | `correctness`; clause "past which a Renderable is refused"; no Reference scene reaches it at shipped capacities: `world`'s timed frames hold 307,263 slots in use (`docs/performance.md:330-331`) of 488,448 | `prepare.rs:330-331` | block below | ADR 0003: "the buffer holds a fixed number of slots, past which a Renderable is refused, warned about once, counted in Frame stats and retried every frame"; the struct's own doc, `prepare.rs:297-298`: "once all `capacity` are taken a Renderable is refused one" | `confirmed`: `get_or_take` offers a Renderable only `self.free.pop().or(fresh)` with `fresh = (self.end < self.capacity).then_some(self.end)` (`src/render/prepare.rs:330-331`), and `free_ranges` is read by `take_range` alone (`:351-366`), so with `end` at capacity a Renderable is refused while unloaded Chunks' ranges stand free, against ADR 0003's "the buffer holds a fixed number of slots, past which a Renderable is refused" and the struct's own "once all `capacity` are taken a Renderable is refused one" (`:296-297`); ranges landed after `8bc34eb1`. | fix #1033 |
| L1-R02 | A static TLAS build uploads every filled entry's instance record in the frame that submits it, and Frame stats' `uploaded_bytes` counts none of it, while the dynamic TLAS's records, staged the same way, are counted. | `correctness`; one counter misses one kind of work: up to 264,720 × 72 = 19,059,840 bytes per static build on Metal at the checked-in capacity (64 bytes an entry elsewhere), in every frame that submits a build | `acceleration_structures.rs:544-552`; `renderer.rs:712-723` | block below | `CLAUDE.md` checklist: "per-frame GPU writes add to `uploaded_bytes`"; the dynamic TLAS's own rule, `acceleration_structures.rs:748-751`, `:884-885` | `confirmed`: The static build's `queue.submit(std::iter::once(build))` (`src/render/acceleration_structures.rs:546`) adds nothing to the counters (`src/render/renderer.rs:719-723` copy its time, frames and left-out count only), while the dynamic TLAS adds `uploaded_bytes` for the same wgpu-core staging (`renderer.rs:851-853`; `vendor/wgpu-core/src/command/ray_tracing.rs:252` stages every filled entry); the intent is the `CLAUDE.md` checklist's "per-frame GPU writes add to `uploaded_bytes`", which the last audit accepted as intent for its `L1-T09`. Magnitude recomputed: 264,720 × 72 = 19,059,840 bytes (`renderer.rs:1094`, `acceleration_structures.rs:823`). | fix #1029 |
| L1-R03 | ADRs 0003 and 0017 give the Instance slots 136.2 bytes a slot, and ADR 0003 517,376 slots at the M4 Budgets, where the code takes 144.2 bytes a slot, four offsets per slot since Thin Primitives' twin draws, and gives 488,448 slots at the same Budgets. | `drift`; not load-bearing for code, which derives the figure; misleads a reader by 28,928 slots (5.6%), which `docs/research/vegetation-and-grass.md` and `docs/audit/m5/scalability.md` repeat | `renderer.rs:215-218`, `:259-261`; `docs/adr/0003-persistent-scene-transform-buffer.md:3`; `docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:18` | block below | ADRs 0003 and 0017 state the bytes a slot and the slot count the code derives: 144.2 and 488,448 at the checked-in Budgets | `narrowed`: Rewritten to absorb `L1-S05`: ADR 0003 (`docs/adr/0003-...md:3`, "136.2 bytes a slot ... 517,376 slots") and ADR 0017 (`docs/adr/0017-...md:18`, "at 136.2 bytes a slot") and `docs/research/rain-snow-wetness-and-wind.md:131` ("136.2 bytes a slot today") give the two-offset layout, where `INSTANCE_SCAN_BLOCK_BYTES` takes `size_of::<[u32; 4]>()` a slot (`src/render/renderer.rs:215-218`), 144.2 bytes, and the test pins `instances: 488_448` (`:1092`); `drift`. The cause is narrowed too: the capacity's offsets went from `[u32; 2]` to `[u32; 4]` with Thin Primitives, whose third sum made the binding `array<vec3<u32>>` (16-byte stride), and the fourth word later took the LOD error (`src/render/bind_groups.rs:131`, `array<vec4<u32>>`; `culling.wgsl:453-456`). | fix #1034 |
| L1-R04 | ADR 0017 says ADR 0013 leaves the load staging out of GPU memory, but ADR 0013's list of what `gpu_memory_bytes` leaves out, and the `CLAUDE.md` checklist's, do not name it; the load staging's belt buffers, written in every streaming frame since M4, bypass the ledger with no ADR 0013 clause recording it. | `drift`; load-bearing for the staging Budget's reading: `from_budgets` reserves the staging Budget past the frame staging for the load staging, which the `Staging` kind of `gpu_memory_bytes` can never show | `docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:18`; `staging.rs:179-184` | block below | ADR 0013's exclusions name the load staging, or the load staging is counted under `Staging`; M4 story 44: "so that no buffer or texture escapes `gpu_memory_bytes`" | `confirmed`: ADR 0017 says "What ADR 0013 leaves out, the GPU stats readback and the load staging, bypasses it" (`docs/adr/0017-...md:18`), while ADR 0013's exclusions (`docs/adr/0013-...md:3`) name the GPU stats readback, the presented target and capture's read-back buffer and not the load staging; `LoadStaging`'s belt is outside `GpuMemory` (`src/render/staging.rs:179-184`); `drift`. | fix #1034 |
| L1-R05 | The refit's SAFETY comment rests one premise on "only the main thread calls into wgpu meanwhile (ADR 0016)", which is false since the static TLAS's worker records and finishes builds while `render` records the refits, and which ADR 0016 does not say. | `drift`; one premise of one `unsafe` block's SAFETY comment; the block stays sound, as the worker destroys none of the objects the refit holds unguarded | `blas_refit.rs:193-196` | block below | ADR 0016: "Only the main thread submits, presents or polls ... Workers record `CommandEncoder`s and return `CommandBuffer`s"; the premise states what keeps the unguarded objects alive, that nothing destroys them | `confirmed`: The SAFETY premise "only the main thread calls into wgpu meanwhile (ADR 0016)" (`src/render/blas_refit.rs:195-196`) is false since the static TLAS worker thread calls `device.create_command_encoder`, `build_acceleration_structures` and `finish` (`src/render/acceleration_structures.rs:605-615`), and ADR 0016 says only "Only the main thread submits, presents or polls ... Workers record `CommandEncoder`s" (`docs/adr/0016-...md:18`); `drift`, the block stays sound. | fix #1034 |
| L1-R06 | ADR 0021's fourth Consequence says the floor's error names each attachment format without 4x MSAA "or, on colour, its resolve", but since the floor asks the resolve of the HDR format alone it never names the surface normals' `Rgba8Unorm`, a colour attachment format, for its resolve. | `drift`; one ADR sentence contradicting the ADR's own amended first Consequence; zero adapters affected, as wgpu-types 30.0.2 guarantees `MULTISAMPLE_RESOLVE` on `Rgba8Unorm` | `docs/adr/0021-capability-floor.md:17` | block below | the sentence says "or, on the HDR format, its resolve", as `:14` and `src/gpu/mod.rs:363-374` do | `confirmed`: ADR 0021's fourth Consequence says "each attachment format without 4x MSAA or, on colour, its resolve" (`docs/adr/0021-capability-floor.md:17`), while its first, amended, asks the resolve "on the HDR colour format" (`:14`) and `floor_format_features` asks `MULTISAMPLE_RESOLVE` only `if format == crate::render::HDR_FORMAT` (`src/gpu/mod.rs:367-374`), so the typed error can never name `Rgba8Unorm`'s resolve; `drift`, one fix with `L1-R07`. | fix #1034 |
| L1-R07 | The doc comment of `adapter_report` says the capability report checks 4x MSAA on the main pass's attachment formats "resolved on colour ones", but the report asks the resolve of the HDR format alone, so it prints `Rgba8Unorm yes` without checking its resolve. | `drift`; one doc comment; no adapter affected, as L1-R06 | `src/gpu/mod.rs:143` | block below | the comment says "resolved on the HDR format", as `floor_format_features` decides | `confirmed`: `adapter_report`'s doc says "4x MSAA on the main pass's attachment formats, resolved on colour ones" (`src/gpu/mod.rs:142-143`), while the report prints each of `MULTISAMPLED_FORMATS` (`src/render/targets.rs:51-52`, `Rgba8Unorm` among them) as `yes` against `floor_format_features` (`src/gpu/mod.rs:235`), which asks the resolve of the HDR format alone; `drift`, a separate location from `L1-R06`'s ADR sentence. | fix #1034 |

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

## shaders

Scope: `src/render/shaders/`, the WGSL parts and `mod.rs`, with the oracles
in `src/render/shader_function_tests.rs`, `src/render/shader_tests.rs` and
`src/render/coverage_tests.rs`, as far as they changed after `8bc34eb1`:
932 changed lines of the parts and `mod.rs` (`culling.wgsl` 388,
`main.wgsl` 196, `common.wgsl` 148, `mod.rs` 116, `shadow_mask.wgsl` 75,
`shading.wgsl` 7, `dynamic_tlas_marker.wgsl` 2), 548 of the oracle file
and 317 of `shader_tests.rs`; `coverage_tests.rs` did not change and was
read for what it pins. Three readers at high effort shared the module by
file, each with the lane brief verbatim: A, surface shading, coverage,
Thin Primitives, small Instances and Texture classes (`main.wgsl`,
`shading.wgsl`, the coverage and Layer functions of `common.wgsl`,
`coverage_tests.rs`); B, culling, LOD selection and module assembly
(`culling.wgsl`, `mod.rs`, `shader_tests.rs`, the composition and LOD
functions of `common.wgsl`); C, the shadow mask and its rays
(`shadow_mask.wgsl`, `shadow_ray_mask`, `segment_meets_box`, `position_at`
and the octahedral codec). Every reader read `common.wgsl` whole. Each
reader's scope, clause verdicts, stories, registry keys and oracles are a
subsection below, its floors in blocks numbered by its letter (A1, B1,
C1, ...); a block named "M3.5's" is one of that report's. Their Findings
are one table, numbered from `L1-S01` in the readers' order; "M3.5's
L1-S0n" names that report's Finding. ADRs read whole and judged for their
WGSL clauses: 0008, 0014, 0015, 0017, 0019, 0020, 0021, 0026 and 0027,
with 0003 for one figure and the `CLAUDE.md` review checklist's shader
clauses as their operational form. `deviations.md` holds two rows in
these files, `shading.wgsl:point_light_luminance` and
`common.wgsl:selected_lod`, which the code matches. ADRs 0008 and 0023
are unchanged since `8bc34eb1`, as are `hdr.wgsl`, `gt7.wgsl`,
`display.wgsl`, `tonemapping.wgsl`, `froxel.wgsl`, `froxel_grid.wgsl`,
`light_count.wgsl`, `depth_pyramid.wgsl`, `bend.wgsl` and
`blas_refit_marker.wgsl`: M3.5's verdicts on the clauses that fall in
those parts stand, and the rows below give them again only where code in
scope calls them. Every `where` is `path:line` at the frozen commit, under
`src/render/shaders/` unless given in full.

Oracles: 36 tests at the frozen commit against 28 at `M3.5`'s. The eight
added since each have a row below, as do five older tests whose bodies or
comments changed in scope; of the eight, three state their WGSL floor
beside their tolerance and five do not (L1-S03, L1-S08, L1-S13). The
session ran the 36 once on the frozen tree before assembling the section,
`cargo test --lib shader_function_tests` at `e5c55a8d` on the Apple M4
Pro's Metal adapter, battery charged at 100%, load average 4.17 at the
start, no peer session benching or holding a quiet turn: 36 passed in
0.27 s, none skipped for want of an adapter. The readers ran nothing.

### surface shading, coverage, Thin Primitives, small Instances and Texture classes

Read in full at the frozen commit `e5c55a8d`: `src/render/shaders/main.wgsl`
(465 lines; in scope `surface_normals` `:50-72`, `vs_thin` `:93-173`,
`thin_fragment_samples` `:181-184`, `fs_thin_depth` `:186-189`,
`fs_thin_surface_normals` `:191-202`, `fs_thin_main` `:204-215`, `fs_main`
`:298-301`, `lit` `:307-465` and the `froxel_mask_bit` midpoint `:287`),
`src/render/shaders/shading.wgsl` (269 lines; in scope only three comments,
`:110`, `:182-183`, `:262`), `src/render/shaders/common.wgsl` (408 lines;
mine: `CameraUniform` `:17-28`, `MeshData` `:191-201`,
`THIN_NARROWEST_VIEW`, `thin_drawn_radius`, `thin_kept_samples`
`:223-234`, `SMALL_INSTANCE_WIDEST_PIXELS`, `small_instance_coverage`,
`small_instance_scale` `:236-280`, `MeshletData.thin_radius_metres`
`:295-301`, `layer_class_and_slice` `:403-407`), and
`src/render/coverage_tests.rs` (828 lines, unchanged since M3.5, read for
what it pins of ADR 0019's merging clause). Diffed against M3.5's frozen
commit `8bc34eb1` for each file. ADRs read whole, status lines and
amendments included, and given a row per clause that falls in these
files: 0027, 0019, 0008, 0023, 0014, 0015, 0017 and 0026; the `CLAUDE.md`
review checklist's shader clauses as their operational form. Spec stories
read: every story of the M4 spec (`spec-611.md`) and the M5 spec
(`spec-804.md`); rows below for those these files implement. Opened
outside these files, to judge a clause and not audited:
`src/render/shaders/mod.rs:290-330` (the constants these parts read),
`src/render/shaders/culling.wgsl:110-175`, `:300-345`, `:390-430` (how
the culling pass decides which Instances reach `vs_thin`, reader B's),
`src/render/main_pass.rs:90-225` (which entry point each pipeline runs and
its multisample state), `src/render/renderer.rs:185-200`
(`pixel_size_per_metre`), `src/assets/material.rs:1-30`, `:120-240`,
`:520-560` (the Layer encoding and the neutral texels), `src/gpu/mod.rs:298-306`
(the floor's array layers), `src/scenes/town.rs:976-1180` (which Thin
Primitives exist). Sources judged by their entries in
`docs/references.toml`; `docs/audit/deviations.md` and
`scripts/cycles-reference/DIFFERENCES.md` read for what is not a
Finding. Oracles read in `src/render/shader_function_tests.rs`: the five
named in the brief, the three tolerance constants' comments the M3.5 exit
rewrote (`:15-23`, `:1227-1247`) and the `decode_surface` comment
(`:1530-1533`), all against M3.5's derived floors. Nothing was run. Every
`where` is `path:line` at the frozen commit, under `src/render/shaders/`
unless given in full.

#### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0027 | "Partial coverage is a `sample_mask` output of the fragment stage" | holds: the two depth twins return `@builtin(sample_mask)`, `fs_thin_depth` as its only output and `fs_thin_surface_normals` beside the normals | `main.wgsl:186-189`, `:191-202` |
| 0027 | "No pipeline of the main pass enables alpha-to-coverage" | holds: every main-pass pipeline is built with the one `MultisampleState`, `alpha_to_coverage_enabled: false`; no WGSL reads alpha as coverage, `surface.opacity` is written as the colour's alpha and nothing reads it as coverage | `src/render/main_pass.rs:110-114`; `main.wgsl:464` |
| 0027 | "The depth passes write the mask, so only the kept samples take the fragment's depth" | holds: the pre-pass twin runs `fs_thin_surface_normals` with shadow rays on, `fs_thin_depth` with them off, and the lit pass's own depth draw runs `fs_thin_depth`; each writes `thin_fragment_samples`, which the raster ANDs with coverage | `main.wgsl:181-202`; `src/render/main_pass.rs:156-186` |
| 0027 | "The lit pass draws the same geometry at depth equal and writes none: it meets the depths the depth passes kept" | holds: `fs_thin_main` has no `sample_mask` output; every twin runs `vs_thin`, whose `clip_position` is `@invariant`, so the lit twin computes the depth pass's positions | `main.wgsl:94`, `:206-215`; `src/render/main_pass.rs:218-221` |
| 0027 | "A fragment of its twin that keeps no sample is discarded there, since another fragment's depth can equal its own" (amended 2026-11-16) | holds: `kept_samples = covered_samples & thin_fragment_samples(in)`, `discard` at 0 | `main.wgsl:204-213` |
| 0027 | "A fragment keeps a count of the four samples equal to its coverage times four, rounded to the nearest, and none below an eighth" | holds: `u32(saturate(coverage) * 4.0 + 0.5)`, round half up, truncation toward zero; 0 exactly below 0.125, and each threshold 0.125, 0.375, 0.625, 0.875 is exact in f32 (Oracles, Block A2) | `common.wgsl:231-233` |
| 0027 | "The samples kept follow from the count alone, sample 0, then 3, 1 and 2, so every fragment of a pixel keeps the same ones and a tube's faces share them" | holds for the order: `0xfb910 >> 4·count & 15` gives 0b0000, 0b0001, 0b1001, 0b1011, 0b1111, nested; the same samples in every fragment of a pixel hold where the fragments' counts agree, which per-fragment inputs can split at a rounding threshold (Notes) | `common.wgsl:230-234` |
| 0027 | "No dither, no pattern across pixels, no state across frames" | holds: the mask reads the count alone; `vs_thin` and `thin_fragment_samples` read this frame's camera, Instance, Mesh and vertex only | `common.wgsl:231-234`; `main.wgsl:113-184` |
| 0027 | "Each pipeline of the main pass has a twin whose fragment stage writes the mask, or for the lit pass meets it" | holds, shader side: four twins, each running `vs_thin`: `fs_thin_surface_normals` twins the pre-pass's `vs_depth_normals` and `fs_surface_normals` (shadow rays on); `fs_thin_depth` twins the pre-pass's `vs_depth_prepass` alone (rays off) and the lit pass's depth draw, `vs_depth_prepass` and `fs_depth`; `fs_thin_main` twins `vs_main` and `fs_main`. The lists and draws fall in shaders B and render | `main.wgsl:113`, `:187`, `:197`, `:207`; `src/render/main_pass.rs:146-221` |
| 0027 | "The twins draw every frame, since any Instance can be small" | falls in render | |
| 0027 | "Its vertices move out from its tube's centre line until the tube draws a pixel wide" | holds: the centre is the vertex less the radial vector, `instance.rotation_scale * (normal * radius)`; the vertex moves to `centre + radial · drawn / radius` with `drawn = max(radius, 0.5 · pixel / 0.75)`, so the tube's narrowest view, 0.75 of its drawn diameter, spans a pixel | `main.wgsl:122-134`; `common.wgsl:223-228` |
| 0027 | "its coverage is the tube's true width over the width it draws" | holds: `radius / thin_drawn_radius(radius, pixel)`, the radius flat and the pixel's span interpolated at the centroid; both widths scale with the same view factor | `main.wgsl:182` |
| 0027 | "It reads the camera and the Instance alone" (Thin Primitive) | holds: the camera uniform, the Instance's rotation-scale, its Meshlet's radius and the vertex's normal | `main.wgsl:115-128` |
| 0027 | "With culling on, the culling pass drops an Instance of a Mesh that is all Thin Primitives once its widest tube keeps no sample at the bounds' nearest depth" | falls in shaders B (`culling.wgsl:328-344`, `:425-427`) | |
| 0027 | "A Far proxy holds no Thin Primitive, so the rule ends at the Load radius" | falls in cook | |
| 0027 | "An Instance whose Mesh bounds cover under a pixel's area is scaled about their centre, across the view, until they cover a pixel's, each vertex keeping its depth" | broken off the view axis, `correctness`, L1-S02: each vertex's offset from the bounds' centre keeps its component along the view axis, the clip-w row's `xyz`, and the rest is scaled by `inverseSqrt(coverage)`, so each vertex keeps its w and its reverse-Z depth near/w (ADR 0015) as the clause asks; but the footprint the bounds' depth extent draws through perspective is not scaled by s², so the scaled bounds cover a pixel on the view axis only (0.84 px at coverage 0.5 at the view's corner) | `main.wgsl:136-160`; `common.wgsl:277-280` |
| 0027 | "its coverage is the area they truly cover over a pixel's" | holds where the bounds' area share is at least (sphere diameter / 4 px)²; elsewhere the coverage is that sphere term, larger than the area share: `drift`, L1-S01 | `common.wgsl:253-274` |
| 0027 | "With culling on, the culling pass drops it once that rounds to no sample, under an eighth" | falls in shaders B for the drop (`culling.wgsl:424`); the shader half holds: coverage under 0.125 keeps mask 0, and the scale stops at 0.125 | `common.wgsl:231-233`, `:277-280` |
| 0027 | "An Instance that draws four pixels wide across the sphere about its bounds is scaled no further" | holds: `sphere_share = diameter² / (4² · pixel²)` is 1 at four pixels across, so `max(area_share, sphere_share) ≥ 1` and the select returns 1, scale 1; below four pixels the sphere term holds the scale under 1/√(area share) (L1-S01) | `common.wgsl:267-274` |
| 0027 | "A Mesh with a Thin Primitive keeps that rule alone" | holds in `vs_thin`: a Meshlet with a radius takes the tube's rule whatever its Instance; that such a Mesh's other Meshlets never reach `vs_thin` is the culling pass's (`culling.wgsl:400-411`: coverage stays 1, so the Instance is not small), shaders B | `main.wgsl:122`, `:135` |
| 0027 | "It reads the camera and the Instance alone" (small Instance) | holds: the Instance's anchor, offset and rotation-scale, its Mesh's bounds and the camera | `main.wgsl:136-160` |
| 0027, Consequences | "A tube of few faces draws wider than a pixel so that its narrowest view spans one, a third wider for three faces, so `town`'s wire ends where it is a sixth of a pixel wide" | holds, arithmetic: drawn = pixel / (2 · 0.75), so a three-faced tube draws 1/0.75 = 4/3 of a pixel across a face; coverage = r / drawn = 0.75 · d / pixel, under 1/8 where d < pixel / 6 | `common.wgsl:224-228`, `:231-233` |
| 0027, Consequences | "Every tube takes three faces' share, 0.75 of its diameter, whatever its faces" | holds: one constant, `THIN_NARROWEST_VIEW = 0.75`, no face count read; an equilateral triangle of circumradius R is 1.5 R = 0.75 · 2R wide at its narrowest, and the stored radius is the circumradius (the vertex's distance from the centre line) | `common.wgsl:223-228`; `main.wgsl:125-127` |
| 0027, Consequences | "A tapered tube keeps its taper as it widens, so the coverage at its ends is off by up to the quarter the cook tolerates" | holds in form: one stored radius per Primitive, each vertex moved by the same factor from a centre that is off the true axis by the taper; `town`'s `cylinder(0.03, 0.02, ...)` at radius 0.025 is such a tube; the tolerance is the cook's | `main.wgsl:125-132`; `src/scenes/town.rs:1078` |
| 0027, Consequences | "A fragment stage that writes the mask reads a flat input, so the M4 Pro merges none of its fragments" | holds: `thin_fragment_samples` reads `radius_metres` and `instance_coverage`, both `@interpolate(flat)`, into its result, so no compiler can drop them | `main.wgsl:100`, `:103`, `:181-184` |
| 0027, Consequences | "A small Instance keeps its mean coverage whatever its Mesh, since the samples it keeps fall by the area it gains" | broken off the view axis, L1-S02: on the axis the scale 1/√c multiplies the drawn area by 1/c and the mask keeps c of the samples, so the product is the Mesh's own area share whichever term set c; off it the drawn area grows by f/c + (1 − f)/√c, so the mean coverage falls by up to 35% at the view's corner | `common.wgsl:273`, `:279`; `main.wgsl:157-159` |
| 0027, Consequences | "The culling pass tests a small Instance's bounds as it draws them, scaled"; "the twins' packed indices share the frame's draw's buffer" | falls in shaders B | |
| 0027, Consequences | "A pixel whose samples a mask splits between two surfaces is listed for the shadow mask pass's second surface" | falls in shaders C; the lit twin's half holds: `lit` takes `covered & kept` as its coverage, so a thin fragment counts as the nearest surface only where none of its kept samples shows the second surface | `main.wgsl:210`, `:214`, `:390-395` |
| 0019 | "a lit pass at depth equal" (the twins) | holds: as for the ordinary pipelines, one vertex function per twin set, `vs_thin`, with an `@invariant` position, so the lit twin's depths meet the depth twins' bit for bit | `main.wgsl:94`, `:113` |
| 0019 | "one more per phase and pipeline for Thin Primitives and small Instances (ADR 0027)" | holds, shader side: an entry point per twin (rows above); the draws are render's | `main.wgsl:113-215` |
| 0019 | "with alpha-to-coverage for masked materials (amended 2026-11-15, ADR 0027: alpha-to-coverage stays off, and partial coverage ... is a written `sample_mask`)" | holds: the ADR 0027 rows above; masked Materials are M7's | `src/render/main_pass.rs:113` |
| 0019 | "A fragment stage that reads any flat input is never merged"; "Two passes that must agree on a sample's triangle, as the depth passes' normals and the lit pass must, both read a flat input" | holds for the twins: `fs_thin_surface_normals` reads `material_id` through `surface_normals`' select and the radius and coverage through `thin_fragment_samples`; `fs_thin_main` reads all three; `fs_thin_depth` reads the radius and coverage. The ordinary entry points are unchanged (M3.5's row); `coverage_tests.rs` still pins the merging the clause describes (`:648-669`) and nothing in scope changed it | `main.wgsl:71`, `:181-184`, `:197-202`, `:207-214`; `src/render/coverage_tests.rs:648-669` |
| 0019 | "the depth passes also write each sample's triangle normal and the normal interpolated over it, 4 bytes per sample" | holds for the twin: `surface_normals` is the former body of `fs_surface_normals` moved into a function, its arguments the varyings it read, called by both depth stages that write normals; a widened tube's normals are those of the triangles drawn | `main.wgsl:50-77`, `:196-202` |
| 0019 | "each fragment reads the bits traced from the surface it shades: the nearest sample's, or, where the fragment covers a sample off that sample's plane, the farthest such sample's ... a nearest-surface fragment that runs under a nearer surface at another sample reads that surface's" | holds, unchanged in `lit`, and for the twin with its kept samples as the coverage | `main.wgsl:387-397`, `:210`, `:214` |
| 0019 | "the vertex shader reads the record for its Instance, Material and vertex base, then pulls its vertex from the vertex Arena" | holds: `vertex_output`, and `vs_thin` reads the same record, Meshlet and Instance again for the radius, the bounds and the normal | `main.wgsl:217-247`, `:114-124` |
| 0019 | "Each Primitive draws with its own Material unless its Instance overrides it" | holds for the read: `record.material_id`, resolved by the culling pass | `main.wgsl:245` |
| 0019 | "One compute culling pass composes each Instance's bounds centre camera-relative (ADR 0014; amended 2026-11-04: the vertex shader still composes each vertex)" | holds: `vertex_output` composes each vertex; `vs_thin` composes the bounds' centre once more for the small-Instance rule, by the same function and the same arguments as `culling.wgsl:133-141`, so both see one centre | `main.wgsl:227-233`, `:140-146`; `culling.wgsl:133-141` |
| 0019 | "lit by every directional light and by the point lights of each pixel's Froxel ... each hidden where the shadow mask says so ... up to 59 lights per pixel" | holds, unchanged since M3.5's rows: the body of `fs_main` moved into `lit` without change past the Texture reads; the twin shades through the same `lit` | `main.wgsl:365-457`, `:298-301`, `:214` |
| 0019 | "they are listed from the light buffer, which holds its lights directional first ... since the lit pass sums lights and the shadow mask gives them its bits in buffer order" (amended 2026-11-04, 2026-11-06) | holds: directional lights 0 to `directional_light_count`, then the Froxel's entries in light order; the struct comment no longer names the shadow-casting split the amendment removed | `main.wgsl:398-457`; `common.wgsl:161-171` |
| 0019 | `froxel_mask_bit`'s binary search, which finds a light's bit in another slice's list | holds: the midpoint changed from `(low + high) / 2` to `low + (high - low) / 2`, the same value for any count at most `FROXEL_LIGHT_CAPACITY` (127), so the change cannot move a bit | `main.wgsl:282-296`; `src/render/froxel_pass.rs:28` |
| 0008 | "Anti-aliasing is spatial (MSAA, SMAA)" | holds: coverage below a pixel is four samples' worth, rounded; no jitter, no accumulation | `common.wgsl:231-234` |
| 0008 | "Nothing the image depends on reads screen-space history ...: ... the main pass reads none (ADR 0019)" | holds: `vs_thin`, `thin_fragment_samples` and `lit` read this frame's uniforms, buffers, Textures and shadow mask only | `main.wgsl:113-215`, `:307-465` |
| 0023 | "a `DirectionalLight` gives illuminance in lux and a `PointLight` luminous intensity in candela ... emissive ... luminance in cd/m² ... ambient ... in lux" | holds, unchanged since M3.5's rows; the twin shades through the same `lit` | `main.wgsl:398-460`; `shading.wgsl` unchanged in code |
| 0023 | "Shading runs in f32 and pre-exposes colour before fp16 stores it, and every path clamps it to 65504" | holds for both lit entry points: one `min(luminance * camera.exposure, LARGEST_HALF)` in `lit` | `main.wgsl:463-464` |
| 0014 | "The GPU never holds a World position in f32 ... shaders compose positions relative to the camera" | holds in the scope's code: `vs_thin` composes the bounds' centre by `camera_relative_position` from anchors and offsets, and the widening and the scaling work on camera-relative positions and offsets from them | `main.wgsl:127-133`, `:140-160`; `common.wgsl:37-46` |
| 0014 | "point lights reach the GPU as an anchor plus an offset, which ... the lit pass compose camera-relative" | holds: `point_light_position` in `lit` | `main.wgsl:443`; `common.wgsl:127-139` |
| 0014 | the precision bar, "a point 0.3 m from the camera lands within 0.25 px ... (0.043 mm)" | holds for the twins: an unwidened tube's vertex is recomposed as `(p − radial) + radial · (r / r)`, which returns p within two roundings of \|p\| (2 × 2⁻²⁴ × \|p\|: 0.12 µm at 1 m, 12 µm at 100 m, where a quarter pixel at 1440p and 45° is 100 × 2 tan 22.5° / 1440 / 4 = 14 mm), and the depth and lit twins compute it alike | `main.wgsl:127-133` |
| 0015 | "with no far plane"; clip w is the depth along the view axis | holds where the twins rely on it: a pixel's span at the centre line is `pixel_size_per_metre · w`, and a vertex at or behind the camera's plane (w ≤ 0) stays and keeps all its samples; the small-Instance scaling keeps w, hence depth near/w | `main.wgsl:128-130`, `:154-160`, `:182` |
| 0015 | "Depth is cleared to 0 and compared with `Greater`"; "Hi-Z passes follow the same direction" | not applicable to these parts, which compare no depth; the twins' depth state is render's | |
| 0017 | "A Layer names a class and a slice of its array, Layer 0 stays the neutral default" | holds: `layer / SLICES_PER_CLASS`, `layer % SLICES_PER_CLASS`, the inverse of Rust's `class * SLICES_PER_CLASS + slice` with the same constant, 256, declared from Rust; Layer 0 decodes to class 0's slice 0, where each Channel's table places its neutral texture first (oracle below) | `common.wgsl:403-407`; `mod.rs:315`; `src/assets/material.rs:22-24`, `:136-140`, `:520-526` |
| 0017 | "Each Channel is one partially bound binding array ... of texture arrays ..., an array per Texture class" | holds for the reads: each of the five Channel reads indexes its binding array by the class and samples the slice as the array layer | `main.wgsl:309-335` |
| 0017 | "sampled trilinear, the colour Channels at 16x anisotropy and the data Channels at 4x" | holds for which sampler each Channel takes: `colour_sampler` for albedo and emissive, `data_sampler` for normal, metallic-roughness and occlusion; the samplers' descriptors are render's | `main.wgsl:314-335` |
| 0017 | "a class's slice count is fixed when it opens, at most the Capability floor's 256 array layers" | holds as WGSL reads it: `SLICES_PER_CLASS` is `FLOOR_TEXTURE_ARRAY_LAYERS`, wgpu's default `max_texture_array_layers`, 256 | `mod.rs:315`; `src/gpu/mod.rs:305-306` |
| 0026 | "The baked Material. Its factors are 1 for base colour, roughness and metallic, and it names no normal, occlusion or emissive Texture" | holds in the lit pass: a Channel without a Texture reads Layer 0, its neutral texel: the flat normal, decoded exactly (oracle below), white occlusion, and white emissive times the Material's emissive factor | `main.wgsl:309-335`; `src/assets/material.rs:520-560` |
| checklist | "No main-pass pipeline enables alpha-to-coverage. Partial coverage is a `sample_mask` the depth passes write, through pipelines of their own, the same samples in every fragment of a pixel, from a stage that reads a flat input (ADR 0027)" | holds (the ADR 0027 rows above; same samples per pixel where the counts agree, Notes) | `main.wgsl:181-202`; `common.wgsl:230-234` |
| checklist | "calls no `textureSample` under non-uniform control flow" | holds by WGSL's rules: `lit` samples its five Textures first; in `fs_thin_main` a non-uniform `discard` precedes the call to `lit`, which WGSL defines as demoting the invocation to a helper, so the samples and derivatives after it stay in uniform control flow; whether Metal's lowering keeps the discarded quad members alive is a Note | `main.wgsl:207-214`, `:303-335` |
| checklist | "no divide-by-zero or NaN at grazing angles or zero roughness" | holds in the scope's code: `drawn / radius` only under `radius_metres > 0`; `thin_fragment_samples` floors the pixel at 1e-9, so its divisor is at least 6.7e-10; `along_view` divides by at least 1e-30; `small_instance_coverage` divides by `view_depth · pixel_area`, 0 at w = 0, where the select returns 1, and a NaN fails `coverage < 1.0` and returns 1 too; `small_instance_scale` clamps at 0.125 before `inverseSqrt` | `main.wgsl:130-132`, `:157`, `:182`; `common.wgsl:256`, `:274`, `:279` |
| checklist | "WGSL keeps sRGB and linear apart" | holds: unchanged reads; the neutral colour texels are `Rgba8UnormSrgb` and the neutral normal `Rgba16Float`, linear, as their Channels' Textures are | `main.wgsl:314-335`; `src/assets/material.rs:536-553` |
| checklist | "No World position reaches the GPU in f32" | holds (the ADR 0014 rows) | |
| checklist | "Matrices reach WGSL column-major" | holds for the uses: `view_proj[0].w`, `[1].w`, `[2].w` are the clip-w row's x, y and z, the view axis; `rotation_scale[i] * half_extent` scales column i, the Mesh's axis i | `main.wgsl:155`; `common.wgsl:246-250` |
| checklist | "A new or changed shading function in WGSL has an oracle test"; "a function under an oracle lives in a shared part" | holds: `thin_drawn_radius`, `thin_kept_samples`, `small_instance_coverage`, `small_instance_scale` and `layer_class_and_slice` sit in `common.wgsl`, each under an oracle; `thin_fragment_samples`, a select between two of them, and `surface_normals`, the former entry point's body, have none | `shader_function_tests.rs:2305-2465`, `:2572-2609` |
| checklist | "A binding is written once ...; no WGSL part declares one" | holds: no `@group` or `@binding` in the three parts | |
| checklist | comments cite "only an upstream bug, adapted code's source and licence, a pinned third-party source or an ADR" | holds: no ticket number in the three parts; the two timing figures the scope removed from comments now point to `docs/performance.md`'s Measurements behind the code, whose rows 27 and 28 hold them | `main.wgsl:365-369`; `shading.wgsl:182-183`; `docs/performance.md:27-28` |
| checklist | "Identifiers are full words, WGSL included" | holds: no abbreviation in the scope's code | |

#### Spec stories

| Story | Verdict | Where |
|---|---|---|
| M4 story 32 ("each Channel bound as a binding array of texture arrays, one array per shape") | holds, shader side: each Channel's read indexes its binding array by the Layer's class and samples its slice as the array layer; the class table that opens one array per shape is the assets module's | `main.wgsl:309-335`; `common.wgsl:403-407` |
| M4 story 33 ("the Capability floor kept at 256 layers per texture array") | holds as WGSL reads it: `SLICES_PER_CLASS` is declared from `FLOOR_TEXTURE_ARRAY_LAYERS`, wgpu's default limit, 256, so no Layer names a slice past it | `mod.rs:315`; `src/gpu/mod.rs:305-306`; `common.wgsl:405-407` |
| M5 story 17 ("a Far proxy's Material baked ..., albedo and metallic-roughness") | holds in the lit pass: the proxy's other three Channels read Layer 0, whose normal decodes to the flat normal exactly (oracle below) and whose occlusion and emissive texels are white, the emissive multiplied by its factor | `main.wgsl:309-335`; `shader_function_tests.rs:1398-1421` |
| M5 story 35 ("a Thin Primitive narrower than a pixel drawn a pixel wide with its coverage rounded to the nearest of five levels") | holds: widened in `vs_thin` to a pixel at its narrowest view, its true width's share kept as a count of four samples rounded to the nearest, none under an eighth | `main.wgsl:122-134`, `:181-184`; `common.wgsl:223-234` |
| M5 story 36 ("masked geometry drawn through pipelines of its own") | holds, shader side: the mask is written only by the twins' fragment stages, `fs_thin_depth` and `fs_thin_surface_normals`; no ordinary entry point writes or reads it beyond its coverage input; which indices reach the twins is shaders B's | `main.wgsl:186-202`, `:74-82`, `:298-301` |
| M5 story 37 ("a Thin Primitive to cast a shadow only onto receivers whose pixel is no wider than it") | falls in shaders C (`shadow_ray_mask`) | |
| M5 story 39 ("an Instance smaller than a pixel scaled to a pixel with its coverage by area, and dropped at zero coverage") | holds on the view axis: scaled across the view by 1/√coverage, its samples kept by coverage, mask 0 under an eighth (the drop itself is shaders B's); off the axis the scaled Instance covers less than a pixel and fades by up to 35% more than its area (L1-S02); "coverage by area" holds where the area share exceeds the four-pixel sphere term, which sets it for slivers (L1-S01) | `main.wgsl:136-160`; `common.wgsl:239-280` |
| M5 story 9 ("selection to read no earlier frame"), as it bears on the twins | holds for the widening, the scaling and the mask: they read this frame's camera and Instance only (ADR 0008 row) | `main.wgsl:113-184` |

#### Registry keys

| File | Keys | Verdict |
|---|---|---|
| `main.wgsl:339` | `gltf-2.0-bitangent` | matches, unchanged since M3.5 (moved into `lit`): `cross(geometric_normal, tangent) * in.tangent_world.w` (`:345`) is the quote's "cross product of the XYZ components of the normal and tangent vectors and multiplying that cross product against the W component of the tangent" |
| `main.wgsl:381` | `salvi-2012-sbaa-surfaces` | matches, unchanged: one word of the shadow mask per surface a fragment shades, the quote's "computing only one color sample per surface"; the twin passes its kept samples as the coverage that picks the surface (`:210`, `:390-395`) |
| `shading.wgsl:7`, `:15`, `:21`, `:25`, `:26`, `:36`, `:37`, `:64`, `:93`, `:103`, `:108`, `:152`, `:165`, `:226`, `:231`, `:266` | the sixteen keys M3.5's report matched | match: no cited line's code changed in scope (the file's diff is three comments), and none of these entries changed in `docs/references.toml` since `8bc34eb1` (`git diff 8bc34eb1..e5c55a8d -- docs/references.toml` names none of them); the `khr-lights-punctual-range` floor is `deviations.md`'s first row |
| `common.wgsl` (scope's functions) | none cited | the coverage functions cite no source, and none needs one: the five levels, the sample order and the widening are ADR 0027's own decisions, and the projected-area form `a \|n · centre\| / depth³` is derived in the oracle's doc comment (`shader_function_tests.rs:2369-2374`) |
| `shader_function_tests.rs:1417-1420`, `:1530-1533` | `vulkan-glsl-std-450-precision` | matches: the quote's "sqrt(): Inherited from 1.0 / inversesqrt(). inversesqrt(): 2 ULP." carries the comments' claim that sqrt of a perfect square need not come back exact; `msl-4.1-fast-math`'s "sqrt(x): Implemented as x * rsqrt(x)" with "rsqrt: <= 2 ulp" allows the same on Metal, which neither comment names (Notes) |
| `shader_function_tests.rs:15-23` | `wgsl-2026-float-accuracy`, `msl-4.1-rounding` | match: the quote's "x / y: 2.5 ULP" gives the GGX peak's 1.3e-6 (2.5 × 2⁻²¹ = 1.19e-6 plus f32(π)'s 1.4e-7 relative to 5.09); "round toward zero ... may be supported" is the u = 2⁻²³ of the parenthesised figures; each figure equals M3.5's derived floor |
| `shader_function_tests.rs:1238-1246` | `higham-3.1` | matches: γ₃ = 3u/(1 − 3u) at u = 2⁻²³ is 3.58e-7, the comment's 3.6e-7; × 0.71 over the masking term's 1.13e-3 is 2.24e-4, its "up to 2.2e-4"; M3.5's L1-S01 is answered by this comment |

#### Oracles

u = 2⁻²⁴ for round to nearest and 2⁻²³ where Metal may round toward zero
(`msl-4.1-rounding`); division 2.5 ULP (`wgsl-2026-float-accuracy`,
`vulkan-spirv-precision`, `msl-4.1-fast-math`); `inverseSqrt` 2 ULP and
`sqrt` inherited from 1/`inverseSqrt` (`vulkan-glsl-std-450-precision`,
`wgsl-2026-float-accuracy`), on Metal x · rsqrt(x) (`msl-4.1-fast-math`);
a dot of n terms within γₙ Σ\|xᵢyᵢ\| (`higham-3.1`).

| Function | Test | Constant | WGSL floor | Stated error | Stated floor | Verdict |
|---|---|---|---|---|---|---|
| `tangent_space_normal` at the normal Channel's Layer 0 | `the_neutral_normal_layer_decodes_to_the_flat_normal_exactly`, `shader_function_tests.rs:1404` | exact (`assert_eq!`, `:1421`) | 0 for x, y and the scale-0 case; z at scales 0.5, 1 and 2 is `sqrt(1)`, which the allowances let land within 4.2e-7 of 1 on WGSL and Vulkan and 2.4e-7 on Metal (Block A1) | none | yes, `:1417-1420` (Vulkan's allowance; Metal's x · rsqrt(x) unnamed) | tight: passes where `sqrt(1)` returns 1, as a correctly rounded root does; the expected (0, 0, 1) is glTF's flat normal and 0x3800 is binary16 0.5 exactly |
| `thin_drawn_radius` | `a_thin_primitive_draws_a_pixel_wide_where_it_is_narrower`, `:2305` | 1e-6 absolute, an unnamed literal (`:2327`) | ≤ 2.5e-8 absolute: the halving exact, one 2.5 ULP division at most 0.08 (ULP 2⁻²⁷), `max` exact, and the f32 inputs' and expectations' own rounding (Block A2) | none | no | loose, about 40×; every expected value recomputed (Block A2), the boundary case 0.0225 m equal to the radius by construction |
| `thin_kept_samples` | `a_thin_primitive_keeps_the_samples_nearest_its_coverage`, `:2335` | exact (`assert_eq!` on the mask, `:2360`) | 0: `saturate` and × 4 exact, + 0.5 exact at the four thresholds 0.125, 0.375, 0.625, 0.875 (sums 1, 2, 3, 4), the off-threshold inputs 0.124, 0.374 and 0.874 sit 0.016 of a count below one, about 2¹⁷ ULP, so neither a fused multiply-add nor round toward zero can flip a count; `u32()` truncates; shift and mask are integer (Block A2) | none | no | exact; the twelve masks recomputed from `0xfb910` |
| `small_instance_coverage` | `a_small_instance_keeps_its_bounds_share_of_a_pixel_and_scales_to_one`, `:2375` | 2e-6 absolute, unnamed literal (`:2427`) | ≤ 8.3e-7 × coverage from about 14 roundings (1.7e-6 at 2⁻²³), plus the literals' own: 0.353_553 is 3.9e-7 under 0.5 · f32(1/√2), 0.5625 is 1.3e-7 under the sheared case's value from its f32 inputs: worst ≈ 1.1e-6 (Block A3) | none | no | explained, 1.8× at round toward zero, about 3× at round to nearest; all sixteen expected coverages recomputed by hand (Block A3) |
| `small_instance_scale` | the same test | 1e-5 absolute, unnamed literal (`:2430`) | `inverseSqrt` 2 ULP at most 2.83 (4.8e-7), plus half the coverage's relative error times the scale (≤ 1.2e-6, 2.4e-6 at 2⁻²³), plus the literals' rounding (2.777_778: 2.2e-7): ≤ 1.9e-6, 3.1e-6 at 2⁻²³ (Block A3) | none | no | explained, 3 to 5× |
| `thin_kept_samples` of the coverage | the same test | exact (`:2431`) | 0 in effect: the nearest case to a threshold is 0.1296, 0.0184 of a count past 1, four orders past the coverage's floor | none | no | exact |
| `layer_class_and_slice` | `every_layer_of_texture_wall_decodes_to_the_class_and_slice_it_was_placed_in`, `:2572` | exact (`assert_eq!`, `:2601`) | 0: `/` and `%` by the constant 256 are exact in u32; the Layer reaches the kernel as f32 and `u32(input.x)` is exact below 2²⁴; texture-wall's largest Layer is 47 · 256 + 16 = 12,048 (Block A4) | none | yes, `:2588` ("Layers stay below 2^24, so f32 carries them exactly") | exact; the expectation is the Rust twin's own placement, so the test pins the round trip through `ClassAndSlice::layer`, and its first assertion that the neutral texture is Layer 0 (`:2599`) |

##### Block A1: the neutral normal

`shading.wgsl:157-159` with `encoded = unpack2x16float(0x38003800) =
(0.5, 0.5)`, exact: `encoded * 2.0 - 1.0` is 0 in every rounding mode
(1.0 − 1.0); `dot` of zeros is 0; `1.0 - 0.0` is 1; `max` exact;
`sqrt(1)`. On WGSL and Vulkan, `sqrt` is inherited from `1.0 /
inverseSqrt(x)`: `inverseSqrt(1)` lies within 2 ULP, in [1 − 2 · 2⁻²⁴,
1 + 2 · 2⁻²³], and the division adds 2.5 ULP: from 1 − 2⁻²³ the quotient
is about 1 + 2⁻²³ + 2.5 · 2⁻²³ = 1 + 4.2e-7, from 1 + 2⁻²² about
1 − 2⁻²² − 2.5 · 2⁻²⁴ = 1 − 3.9e-7. On Metal `sqrt(x)` is `x * rsqrt(x)`,
so z is `rsqrt(1)` within 2 ULP, \|z − 1\| ≤ 2.4e-7. `x_and_y * scale` is 0
for every finite scale, and at scale 0 the `select` returns 1.0 exactly.
The test compares with `assert_eq!`, so it allows none of this: it passes
where `sqrt(1)` is correctly rounded.

##### Block A2: the thin pair

`thin_drawn_radius`, `common.wgsl:227`: `0.5 * pixel_metres` exact (a power
of two, no underflow at these sizes); `/ THIN_NARROWEST_VIEW` one division,
2.5 ULP, or under fast math's "allow reciprocal" a product with f32(4/3)
(relative 3.0e-8) and one rounding; `max` exact. The largest result is
0.08 (pixel 0.12 m), whose ULP is 2⁻²⁷ = 7.45e-9: 2.5 ULP = 1.9e-8. The
inputs and expectations are f32 literals, each within half an ULP of the
decimal: 0.12 within 3.7e-9, which the division scales to 2.5e-9; 0.08
within 3.7e-9. Sum ≤ 2.5e-8; the test's f32 subtraction of two values
this close is exact (Sterbenz). Expected values: 0.5 · 0.012 / 0.75 = 0.008
< 0.015 → 0.015; 0.5 · 0.0225 / 0.75 = 0.015 = the radius → 0.015 either
way; 0.03 → 0.02; 0.06 → 0.04; 0.12 → 0.08; radius 0.025 with 0.03 →
0.02 < 0.025 → 0.025; with 0.06 → 0.04. All seven match `:2310-2316`.

`thin_kept_samples`, `common.wgsl:231-233`: `0xfb910 >> (4 · count) & 15`
gives, for counts 0 to 4, 0x0, 0x1, 0x9 (0b1001), 0xb (0b1011), 0xf: sample
0, then 3, then 1, then 2, ADR 0027's order. Count = ⌊saturate(c) · 4 +
0.5⌋: 0 → 0; 0.124 → ⌊0.996⌋ = 0; 0.125 → 1; 0.25 → ⌊1.5⌋ = 1; 0.374 →
⌊1.996⌋ = 1; 0.375 → 2; 0.5 → ⌊2.5⌋ = 2; 0.625 → 3; 0.874 → ⌊3.996⌋ = 3;
0.875 → 4; 1.0 → ⌊4.5⌋ = 4; 3.0 → saturate 1 → 4. All twelve match
`:2340-2351`. f32(0.124) · 4 = 0.49599999 exactly (power of two); + 0.5
lies in [0.5, 1), ULP 2⁻²⁴, so its rounding moves it by at most 6e-8 from
0.996: no count flips. The thresholds' sums are integers, exact in any
rounding.

##### Block A3: the small Instance

Expected coverages, from `common.wgsl:246-274` with k = 0.001 and w the
negated centre z:

- AHEAD (0, 0, −1000), half 0.5: a × b = 0.25 ẑ, |c · (a × b)| = 250, the
  other two 0; facing 4 · 250 = 1000; pixel = 0.001 · 1000 = 1 m, area 1;
  share 1000 / (1000 · 1) = 1. Sphere 4 · 0.75 / 16 = 0.1875. max 1 →
  select 1; scale 1; count 4 → 0b1111. Matches.
- half 0.25: facing 4 · 62.5 = 250 → 0.25; sphere 0.047; scale 2; count
  ⌊1.5⌋ = 1 → 0b0001. Matches.
- half 0.18: 4 · 0.0324 · 1000 / 1000 = 0.1296; scale 1 / 0.36 = 2.777…;
  count ⌊1.018⌋ = 1. Matches.
- half 0.175: 0.1225; clamped to 0.125, scale 2√2; count ⌊0.99⌋ = 0.
  Matches.
- half 0: 0; scale 2√2; count 0. Matches.
- ASIDE (1000, 0, −1000), half 0.25: two faces of 0.0625 · 1000 = 62.5
  each, facing 500, share 0.5; scale √2; count ⌊2.5⌋ = 2 → 0b1001.
  Matches. Cross-check against the exact projection: the cube's projected
  area across the ray is 2 · 0.25 · cos 45° = 0.354 m² at 1414 m, a solid
  angle of 1.77e-7 sr, over cos³ 45° = 0.354 on the image plane a metre
  out, 5e-7 m², over a pixel's 1e-6 m²: 0.5. The first-order form is exact
  for a flat face.
- half (1, 0.01, 0.01): facing 4 · 0.01 · 1000 = 40 → area share 0.04;
  sphere 4 · (1 + 1e-4 + 1e-4) / 16 = 0.25005; coverage 0.25005, the
  sphere's; scale 1.9998; count 1. Matches; this case, and the sheared
  one, pin the sphere term L1-S01 is about.
- half (2, 0.01, 0.01): sphere 16.0008 / 16 > 1 → 1. Matches.
- (0, 0, −10), half 0.5: pixel 0.01, area 1e-4, facing 10, share 10 /
  (10 · 1e-4) = 1e4 → 1. Matches.
- (0, 0, 1000): w = −1000, select 1. (1, 0, 0): w = 0, area 0, the share
  1/0 or 0/0, select 1. Both match.
- TURNED (columns (s, 0, −s), (0, 1, 0), (s, 0, s), s = f32(1/√2)), half
  0.25: a × b = 0.0625 (s, 0, s), b × c = 0.0625 (s, 0, −s), c × a =
  0.0625 (0, 2s², 0); against (0, 0, −1000): 44.19 + 44.19 + 0; facing
  353.55, share 0.353553; scale 1.681793; count ⌊1.914⌋ = 1. Matches.
- k doubled, half 0.5: pixel 2 m, area 4: 1000 / 4000 = 0.25. Matches.
- SCALED (2, 1, 0.5) on half (0.125, 0.25, 0.5): the 0.25 cube. Matches.
- SHEARED (columns (1, 0, 0), (1, 1e-3, 0), (1, 0, 1e-3)), half 0.5:
  facing 4 · (0.25 + 0.25 + 0) = 2 → area share 0.002; skews all 0.25,
  the farthest 0.75; diameter² 4 · (0.25 + 0.25 + 0.25 + 5e-7 + 1.5) =
  9.000002, / 16 = 0.5625001; coverage 0.5625; scale 4/3; count ⌊2.75⌋ =
  2 → 0b1001. Matches.

Floor of the coverage chain: `half_axes` one product per component (exact
for the identity, the power-of-two scales and 0.25 · s); a cross
component, two products and a difference (exact where one product is 0, as
in every case but the shear); `dot` with the centre, γ₃; three `abs`
terms summed, 2u; × 4 exact; `pixel_metres` one product, its square one
more and doubled, 3u; × `view_depth` u; the division 2.5 ULP, at most 5u
relative at a binade's foot. About 14u relative: 8.3e-7 (2⁻²⁴) or 1.7e-6
(2⁻²³) of a coverage at most 1, so at most that absolute; in the cases
here most steps are exact and the bound is far from reached. The sphere
term: three dots γ₃ each, the skew sums 2u, the four-term sum 3u, the same
pixel area and division: about 14u as well. Against the literals: the
turned case's 0.353_553 is 3.85e-7 under 0.5 · f32(1/√2) = 0.35355339,
and its computed share carries 0.354 × 14u = 2.9e-7 (5.9e-7): ≤ 9.8e-7;
the sheared case's 0.5625 is 1.3e-7 under 0.5625001, plus 0.5625 × 14u =
4.7e-7 (9.4e-7): ≤ 1.1e-6. Against 2e-6: explained.

Scale: `inverseSqrt` 2 ULP, ULP 2⁻²² = 2.4e-7 on [2, 4): 4.8e-7; a relative
error ε in the coverage is ε/2 in its inverse root: ≤ 7u × 2.83 = 1.2e-6
(2.4e-6); the literal 2.777_778 is 2.2e-7 off 1/0.36: ≤ 1.9e-6 (3.1e-6)
against 1e-5.

##### Block A4: the Layer decode

`common.wgsl:405-406`: `layer / 256u` and `layer % 256u`, integer, exact.
The kernel takes the Layer as f32 (`:2589-2592`) and converts with `u32()`,
exact for integers below 2²⁴. texture-wall's albedo Channel: the neutral
texture opens class 0 with 1 slice (Layer 0); the wall's shape opens
classes of 1, 2, 4, ..., 128 slices (255 Textures in classes 1 to 8), then
of 256 (`src/assets/material.rs:201-205`): 10,000 − 255 = 9,745 = 38 ·
256 + 17, so classes 9 to 46 fill and class 47 takes 17, the last at Layer
47 · 256 + 16 = 12,048, far below 2²⁴ and its class below
`CLASSES_PER_CHANNEL`, 13,107. The class and slice return through
`f32(...)`, exact.

### culling, LOD selection and module assembly

Read in full at the frozen commit `e5c55a8d`: `src/render/shaders/culling.wgsl`
(652 lines), `src/render/shaders/common.wgsl` (407 lines), `src/render/shaders/mod.rs`
(348 lines), `src/render/shaders/dynamic_tlas_marker.wgsl` (17 lines) and
`src/render/shader_tests.rs` (1,020 lines); in `src/render/shader_function_tests.rs`,
lines 1630-1700 (`pack_index`), 1862-2020 (the two precision oracles and
`PRECISION_COORDINATES`), 2213-2302 (`Case`, `sums_in_every_order`) and 2503-2569
(the LOD oracle), with the scope diff of the whole file to find the older tests whose
comments changed. `git diff 8bc34eb1..e5c55a8d` read for every one of those files.
ADRs read whole, status lines and amendments included, and judged clause by clause
where the clause lands in these files: 0019, 0014, 0015, 0027, 0008, 0021, 0026,
0017, and 0003's status line for one figure. Spec stories read: every story of
`spec-611.md` (M4) and `spec-804.md` (M5) with its Implementation Decisions. Opened
outside these files, only to judge a clause: `src/render/culling_pass.rs:40-330` (the
constants the WGSL is assembled with, the buffer sizes, the asserts, the dispatches),
`src/render/renderer.rs:186-290` (`RenderCapacities`, the LOD threshold) and its
capacity test at `:1087-1098`, `src/render/bind_groups.rs:17-138`, `:566-584`,
`src/render/instance.rs:1-22`, `src/assets/cooked.rs:35-57`, `src/assets/pools.rs:505-575`
(how the Mesh table orders a LOD's Meshlets), `src/cook/mesh.rs:260-420` (the LOD
chain and its locked Thin Primitives), `src/transform/mod.rs:1-40` (whether an
Instance can shear), `src/gpu/mod.rs:310-323` (the floor's limits),
`src/render/shaders/main.wgsl:276-296` and `:400-445` (the loops of the `MAIN` module,
which `mod.rs` compiles without loop bounding), `bench/budgets.json:17`, and the
registry entries of every key cited in these files and of the precision keys.
`deviations.md`, `DIFFERENCES.md`, `CONTEXT.md` and the M3.5 report's shaders section
(`docs/audit/m3.5/correctness.md:1505-2613`) read for what is not a Finding. Nothing run.

#### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0019 | line 3: "a compute pass appends the Meshlets of the LOD selected for each Instance whose Mesh bounds reach into the view frustum to the compacted index buffer in slot order, in two phases" | holds: each slot's three counts are summed by an exclusive scan within its block of 256 slots, then across blocks by one workgroup, so every Meshlet record and packed index is written at its slot's prefix sum; atomics only take a maximum or a count | `culling.wgsl:92-109`, `:441-460`, `:473-496`, `:578-601`, `:638-650` |
| 0019 | line 3: "Phase one appends those visible the frame before, one bit per slot" | holds: `visible_before` reads one bit of `instance_visibility` per slot; with CULLING off phase one draws every Instance in use | `culling.wgsl:363-367`, `:418-420` |
| 0019 | line 3: "A compute pass reduces that depth into the Hi-Z pyramid, each texel the minimum of every sample below it" | holds: `depth_pyramid.wgsl` has no change in scope, so M3.5's row stands | `depth_pyramid.wgsl` (unchanged since `8bc34eb1`) |
| 0019 | line 3: "Phase two appends the others whose bounds lie anywhere nearer than the pyramid's minimum over the texels their screen rectangle covers, and rewrites the bits" | holds: `occluded` is unchanged since M3.5 and now receives bounds grown for a Thin Primitive's widening or widened for a small Instance, which only enlarge the box, so the test stays conservative; phase two draws only a slot phase one left `NOT_DRAWN`, and writes every bit of its block from the workgroup's gathered words | `culling.wgsl:220-267`, `:398-417`, `:436-445` |
| 0019 | line 3: "one `draw_indexed_indirect` per phase for the depth pipeline and one of both phases for the lit pipeline, and one more per phase and pipeline for Thin Primitives and small Instances (ADR 0027)" | holds for the arguments the pass writes: `draw` holds both phases' ordinary indices from index 0, `phase_two_draw` phase two's, and each phase has a thin draw, each with `instance_count` 1; which pass issues which call is render's | `culling.wgsl:43-61`, `:497-526`, `:579-590` |
| 0019 | line 15: "One compute culling pass composes each Instance's bounds centre camera-relative (ADR 0014; amended 2026-11-04 [...]: the vertex shader still composes each vertex)" | holds: `bounds_centre` composes the Mesh bounds' centre through `camera_relative_position`, and the frustum, Hi-Z, LOD and coverage tests all take it | `culling.wgsl:133-142`, `:151`, `:292`, `:320` |
| 0019 | line 15: "tests the frustum and a two-phase Hi-Z pyramid of minimum reverse-Z depth" | holds (rows above and the ADR 0015 rows) | `culling.wgsl:190-200`, `:220-267` |
| 0019 | line 15: "each index packs a Meshlet record in its high bits and a vertex within that Meshlet in its low six: the cooker builds Meshlets of at most 64 vertices" | holds: `pack_index(record_index, vertex)` shifts by `MESHLET_VERTEX_BITS`, declared as log2 of `MESHLET_MAX_VERTICES` (64), which a compile-time assert keeps a power of two; the record capacity is asserted under 2^26 | `common.wgsl:320-326`; `culling.wgsl:650`; `mod.rs:43`, `:289`, `:319`; `src/render/culling_pass.rs:221` |
| 0019 | line 15: "Each Primitive draws with its own Material unless its Instance overrides it" | holds: each Meshlet record takes the Meshlet's Material ID unless the Instance's is not `NO_MATERIAL` | `culling.wgsl:594-600` |
| 0019 | line 15: "Culling tests Instances; tests per Meshlet wait for a profile" | holds: every test runs once per slot in `offset_slots`; no Meshlet is tested | `culling.wgsl:384-435` |
| 0019 | line 15: "the Mesh table gives the culling pass each LOD's Meshlet range, triangle range and LOD error, for at most 8 LODs" | holds: `MeshData` holds `first_meshlets`, `meshlet_counts`, `first_triangles`, `triangle_counts`, `thin_triangle_counts` and `lod_errors_metres` as arrays of `MESH_MAX_LODS`, declared from `cooked.rs:42` as 8 | `common.wgsl:191-201`; `mod.rs:291`; `src/assets/cooked.rs:42` |
| 0019 | line 15: "The pass draws, of each Instance, the coarsest LOD whose LOD error times the Instance's scale is under the LOD threshold, one pixel of the target's height, at the distance to the Instance's bounding sphere" | holds for an Instance whose rotation-scale columns are orthogonal: the scale is the largest column length and the sphere's radius the Mesh bounds' half-diagonal times it, which bounds a sheared Instance's box only up to √3 short, so there the distance can be overstated: L1-S04. The threshold is `LOD_THRESHOLD_PIXELS` (1) times a pixel's size a metre away, as the camera uniform carries it | `culling.wgsl:269-300`; `common.wgsl:207-221`; `src/render/renderer.rs:188-200` |
| 0019 | line 15: "It reads the camera and the Instance alone, so both phases draw the same LOD and no earlier frame decides it" | holds: `instance_lod` reads the camera uniform, the Instance, its Mesh table entry and the `FORCED_LOD` override; phase two recomputes it with the same function on the same inputs; no visibility bit enters it | `culling.wgsl:284-300`, `:428-431` |
| 0019 | line 15: "the switch is hard, and no Instance draws two LODs" | holds: one LOD per slot, stored in the slot word's top bits and read back by `append_records` and `append_triangles` through `slot_lod` | `culling.wgsl:447-449`, `:576`, `:630` |
| 0014 | line 3: "The GPU never holds a World position in f32: it reads an integer anchor on a 64 m grid plus an f32 offset, and shaders compose positions relative to the camera" | holds in these files: `InstanceData` and `CameraUniform` carry a `vec3<i32>` anchor and an f32 offset; `camera_relative_position` subtracts the anchors in i32 and scales by `ANCHOR_CELL_SIZE`, declared from Rust's 64 | `common.wgsl:17-28`, `:37-46`, `:177-184`; `mod.rs:288`, `:318` |
| 0014 | line 3: "point lights reach the GPU as an anchor plus an offset, which the Froxel pass and the lit pass compose camera-relative" | holds for the shared function: `point_light_position` composes the light's anchor and offset through `camera_relative_position` with no local position; its callers are other readers' | `common.wgsl:127-139`, `:166-171` |
| 0014 | line 3: "a shader-function oracle holds the composition to the precision bar in every evaluation order within 100 km, and shows f32 World positions failing it" | holds: the GPU's order and every order on the CPU over 8³ camera positions to ±100 km, 26 directions and 3 local positions, with an assertion that f32 World positions miss the bar; the floor the test states leaves out the rounding of its inputs: L1-S06 | `src/render/shader_function_tests.rs:1897-1965`, `:2254-2302` |
| 0014 | line 7: "a point 0.3 m from the camera lands within 0.25 px at 1440p and a 45° vertical field of view (0.043 mm)" | holds: 2 × 0.3 × tan 22.5° / 1440 × 0.25 = 4.31e-5 m; `PRECISION_BAR` is 4.3e-5 | `src/render/shader_function_tests.rs:1872` |
| 0014 | line 19: "the culling pass composes each Instance's bounds centre the same way for its tests (amended 2026-11-04 [...])" | holds: `bounds_centre`; M3.5's L1-S05 no longer applies | `culling.wgsl:133-142` |
| 0014 | line 19: "Worst cases [...] sum to 0.041 mm" | holds where the culling pass uses it: `COMPOSITION_ERROR` takes 0.041 mm for the box's centre and for the vertices, 0.082 mm rounded up to 1e-4 m, unchanged since M3.5; a Far proxy's vertices lie past 128 m (Notes) | `culling.wgsl:202-208`, `:222-227` |
| 0015 | line 3: "Depth is cleared to 0 and compared with `Greater` [...] with no far plane" | holds in the culling pass: `in_view_frustum` tests the four side planes and the near plane, z ≤ w, and no far plane | `culling.wgsl:183-200` |
| 0015 | line 3: "a Hi-Z pyramid keeps the minimum depth, which is the farthest" | holds: `occluded` takes the minimum of four texels as the farthest depth and calls a box occluded only where its nearest depth, the largest z/w over its corners with each corner moved towards the camera, is under it; equality is not occluded | `culling.wgsl:228-266` |
| 0015 | line 3: "shadow comparisons follow the same direction" | falls in shaders C | |
| 0027 | line 3: "Implemented for Thin Primitives [...] and small Instances" | holds for the culling pass's half (rows below) | `culling.wgsl:302-344`, `:398-427` |
| 0027 | line 9: "A fragment keeps a count of the four samples equal to its coverage times four, rounded to the nearest, and none below an eighth" | holds for the call: the culling pass drops on `thin_kept_samples(coverage) == 0`, the same function the fragment stage keeps samples by; the function falls in shaders A | `culling.wgsl:424`, `:426` |
| 0027 | line 9: "No dither, no pattern across pixels, no state across frames" | holds for the culling half: coverage and the drop read the camera, the Instance and the Mesh and Meshlet tables only | `culling.wgsl:316-344` |
| 0027 | line 10: "the culling pass lists the geometry drawn through the twins apart, in slot order within each phase" | holds: a third scan component sums each slot's Thin Primitive indices, every index of a small Instance's; each phase's fill the packed index buffer's end back from the phases' before, and `thin_index` places a slot's at the buffer's end less the phase's end sum plus its own sum less the phase's start, so in slot order | `culling.wgsl:306-314`, `:502-517`, `:556-562`, `:640-646` |
| 0027 | line 10: "The twins draw every frame, since any Instance can be small" | holds for the arguments: `offset_blocks` writes each phase's thin draw, `instance_count` 1 and its `first_index`, every frame, at zero indices too | `culling.wgsl:508-517` |
| 0027 | line 11: "With culling on, the culling pass drops an Instance of a Mesh that is all Thin Primitives once its widest tube keeps no sample at the bounds' nearest depth" | holds: a Mesh whose LOD 0 is all Thin Primitive triangles is dropped where the widest Meshlet radius of LOD 0, times the Instance's largest scale, keeps no sample at the grown box's nearest view depth; conservative, as the grown box is nearer than the Mesh's and the largest scale the widest; LOD 0's widest tube is every LOD's, since the cook locks a Thin Primitive's vertices | `culling.wgsl:328-344`, `:425-427`; `src/cook/mesh.rs:293-297`, `:340-347` |
| 0027 | line 11: "It reads the camera and the Instance alone" (a Thin Primitive's coverage) | holds for the culling pass's widening of the bounds and its drop | `culling.wgsl:162-170`, `:330-344` |
| 0027 | line 12: "An Instance whose Mesh bounds cover under a pixel's area is scaled about their centre, across the view, until they cover a pixel's, each vertex keeping its depth" | holds for the bounds the culling pass tests: the clip-space half axes are scaled by `small_instance_scale(coverage)` in x, y and z with w kept; under the reverse-Z projection with no far plane a direction's clip z is 0, so the depth is kept; the vertex shader's scaling falls in shaders A | `culling.wgsl:401-410` |
| 0027 | line 12: "With culling on, the culling pass drops it once that rounds to no sample, under an eighth" | holds | `culling.wgsl:424` |
| 0027 | line 12: "A Mesh with a Thin Primitive keeps that rule alone" | holds: coverage is worked out only where LOD 0 has no Thin Primitive triangle, so `small` is false otherwise | `culling.wgsl:399-411` |
| 0027 | line 26: "The culling pass's compute stage already binds the nine storage buffers the Capability floor allows (ADR 0021), so the twins' packed indices share the frame's draw's buffer: each phase's fill its end, back from the phases' before, in slot order" | holds: four compute storage buffers in `SCENE_GEOMETRY` (`instances`, `meshes`, `meshlets`, `meshlet_triangles`), five in `FRAME_DRAW`, and `FRAME`'s one compute binding is the camera uniform; the placement is the line 10 row's | `src/render/bind_groups.rs:20-40`, `:71-102`, `:122-138`; `culling.wgsl:556-562` |
| 0027 | line 29: "The culling pass tests a small Instance's bounds as it draws them, scaled, against the view frustum and the depth pyramid [...]; it works out every Instance's coverage for that, culled or not" | holds: coverage is worked out for every slot with a Mesh, before the frustum test and whatever CULLING says, and both tests take the widened bounds; which phase draws it still follows the frame before's bit, as for every Instance (ADR 0019) | `culling.wgsl:398-417` |
| 0027 | line 8: "Who writes it" | falls in shaders A | |
| 0008 | line 3: "Nothing the image depends on reads screen-space history" | holds for the culling pass's choices that change the image: the LOD, a small Instance's coverage and the thin drop read no earlier frame; the visibility bit chooses only the phase | `culling.wgsl:284-344`, `:418-420` |
| 0021 | line 16: "`max_storage_buffers_per_shader_stage` at least 9, since the culling pass's compute stage reads and writes nine" | holds: nine, counted in the ADR 0027 line 26 row; the floor requests 9 | `src/gpu/mod.rs:318-320` |
| 0021 | line 14: "indirect draws, one per pipeline and Hi-Z phase, each from first instance 0" | holds: no WGSL writes `first_instance`, which the clear before phase one leaves 0 | `culling.wgsl:497-526`; `src/render/culling_pass.rs:351` |
| 0021 | line 16: "Required limits are `wgpu::Limits::default()` [...]" as the culling module relies on them | holds: 256 invocations a workgroup, the default's most; 4,128 bytes of workgroup storage (`scan`, 256 × 16, and `visible_words`, 8 × 4) against 16,384; `append_triangles` dispatched in rows of at most 65,535 workgroups | `culling.wgsl:73-77`, `:383`, `:506-507`; `src/render/culling_pass.rs:81` |
| 0017 | line 18: "the Instance slots 30% of the scene and pass buffers', at 136.2 bytes a slot with its offsets and visibility bit, in whole scan blocks" | broken in its figure, `drift`: a slot's offsets are now four u32, the three scan components and the LOD error word, so a slot is 144.2 bytes and the checked-in Budgets give 488,448 slots, not 517,376: L1-S05 | `culling.wgsl:446-457`; `src/render/bind_groups.rs:131`; `src/render/renderer.rs:213-217`, `:1092` |
| 0003 | line 3: "30% of it at 136.2 bytes a slot [...]: 517,376 slots at the Budgets M4 set" | broken in its figures, `drift`: L1-S05 | as above |
| 0017 | line 18, and the checklist: "a load past either is refused, logged and counted"; "no GPU buffer grows at runtime, and a request past a buffer's capacity is refused and logged" | holds for the culling pass's buffers: `fits` refuses an Instance whose records or indices pass a buffer's end, and every Instance after it, and `append_records` counts it in `instances_overflowed`; every capacity is `arrayLength` or `INSTANCE_CAPACITY`; the log is render's | `culling.wgsl:540-554`, `:579-593` |
| 0026 | line 20: "A Far proxy is a Mesh and the culling pass selects its LOD as any Instance's" | holds: `culling.wgsl` has no case for a Far proxy | `culling.wgsl:284-300` |
| 0026 | line 41: "A Far proxy is held, its slot an Instance with no Mesh" | holds: a `NO_MESH` slot is neither drawn nor tested, written `NOT_DRAWN` with a LOD error of 0 | `culling.wgsl:395`, `:446-457` |
| 0026 | line 7: "Under it the Instance slots, the static TLAS and the flat cull keep M4's capacities" | holds for the culling pass: one `INSTANCE_CAPACITY` from `RenderCapacities`, one flat pass over the slots | `mod.rs:297`, `:323` |
| 0026 | line 21: "f32 resolves 0.015 mm between 128 and 256 m" | holds, arithmetic: an f32's step in [128, 256) is 2^-16 m = 1.53e-5 m | |
| checklist | "No World position reaches the GPU in f32: only an anchor plus an f32 offset, composed camera-relative (ADR 0014)" | holds (ADR 0014 rows) | `common.wgsl:37-46` |
| checklist | "Depth has no far plane, and WGSL that compares or reduces depth (shadow, Hi-Z) keeps reversed depth's direction (ADR 0015)" | holds for the Hi-Z test (ADR 0015 rows); the shadow side falls in shaders C | `culling.wgsl:190-266` |
| checklist | "Matrices reach WGSL column-major" | holds for the uses: `rotation_scale[i]` is the Instance's axis i, `view_proj * vec4` and `view_proj * mat3x4` are column-vector products | `culling.wgsl:140`, `:151-161`, `:271-273` |
| checklist | "The GPU builds the frame's draw in slot order within each culling phase, never in the order atomics land" | holds: atomics are `atomicMax` on index counts, `atomicAdd` on the drawn and overflowed counts, `atomicOr` on a workgroup's visibility words; every address written comes from prefix sums | `culling.wgsl:438`, `:581-592` |
| checklist | "A binding is written once, in its bind group's definition [...] no WGSL part declares one" | holds: `culling.wgsl` and `common.wgsl` declare no `@group` or `@binding`; `assemble` appends each bind group's declarations | `mod.rs:337-339` |
| checklist | "Streaming uploads [...] no GPU buffer grows at runtime, and a request past a buffer's capacity is refused and logged" | holds for the culling pass (ADR 0017 row) | `culling.wgsl:540-554` |
| checklist | "A newly used wgpu feature or limit joins the Capability floor" | holds: the nine storage buffers are in the floor; `create_shader_module_trusted` and `ShaderRuntimeChecks` are API, not a feature or limit | `src/gpu/mod.rs:318-320`; `mod.rs:226-247` |
| checklist | "A new or changed shading function in WGSL has an oracle test" | holds for `selected_lod` and `camera_relative_position`; the culling pass's own functions (`instance_lod`, `lod_counts`, `fits`, the coverage calls) are not shading functions and sit under the Renderer's GPU tests | `src/render/shader_function_tests.rs:2511`; `src/render/renderer.rs:1344`, `:3130`, `:3470` |
| checklist | "WGSL [...] has no divide-by-zero or NaN" | holds in the culling pass: the growth divides by `max(half_lengths, 1e-30)`; `occluded` divides by `clip.w` only after `clip.w > w_rounding > 0`; `widest_tube_coverage` divides by `thin_drawn_radius`, at least the radius, after returning where it is 0 | `culling.wgsl:172`, `:234-239`, `:340-343` |
| checklist | "calls no `textureSample` under non-uniform control flow" | holds: the culling pass samples nothing; four `textureLoad`s | `culling.wgsl:263-264` |
| checklist | "No `unsafe` unless absolutely necessary and clearly explained" | holds: the one `unsafe` call carries a SAFETY comment citing the registry's quote of what turning loop bounding off risks; the check it relies on has a gap: L1-S07 | `mod.rs:242-246` |
| checklist | "Identifiers are full words, WGSL included"; comments cite no ticket | holds: no abbreviation and no ticket number in these files | |

#### Spec stories

| Story | Verdict | Where |
|---|---|---|
| M4 story 43, every fixed capacity, Instance slots among them, sized from its Budget at device creation, not a compile-time constant | holds for the WGSL: `INSTANCE_CAPACITY` was imported from a Rust constant at M3.5 and is now `capacities.instances`, which `RenderCapacities::from_budgets` derives and `assemble` writes into the culling module at device creation; the frame's draw stays fixed by design (`DrawCapacity::FIXED`, which the story's list does not name) | `mod.rs:297`, `:323`; `src/render/renderer.rs:252-276`; `src/render/culling_pass.rs:83-100` |
| M4 story 47, a request past any capacity refused, logged and counted in Frame stats | holds for the culling pass's buffers: an Instance past the Meshlet records' or packed indices' end is not drawn, nor any after it, and is counted in `instances_overflowed`, which the GPU stats readback reads; the log is render's | `culling.wgsl:540-554`, `:579-593` |
| M4 story 65, the same Chunk content at the world's origin and at its far corner gives identical images | holds for the shader half: positions reach the GPU as anchor plus offset and are composed with the anchor difference in i32, so the camera-relative position depends only on the cell difference and the two offsets; the capture is tooling's | `common.wgsl:37-46` |
| M5 story 4, a LOD chain of at most 8 | holds for the Mesh table the culling pass reads: arrays of `MESH_MAX_LODS` = 8, and `selected_lod` loops to `min(lod_count, MESH_MAX_LODS)` | `common.wgsl:191-201`, `:215`; `mod.rs:291` |
| M5 story 6, each LOD's error stored in metres, so selection compares it with a pixel's size without source data | holds: `lod_errors_metres[lod] * scale` against `error_per_metre * distance_metres` | `common.wgsl:216` |
| M5 story 8, every Instance drawn at the coarsest LOD whose error is under a pixel | holds, with L1-S04 for a sheared Instance | `culling.wgsl:284-300`; `common.wgsl:207-221` |
| M5 story 9, selection reads no earlier frame | holds: the camera, the Instance and its Mesh only | `culling.wgsl:284-300` |
| M5 story 10, a Render switch that forces every Instance to LOD 0 and one that forces a given LOD | holds: `forced_lod` becomes the `FORCED_LOD` override, `MESH_MAX_LODS` when unset, clamped to LOD 7 on the CPU and to each Mesh's coarsest in WGSL, so `Some(0)` forces LOD 0 | `culling.wgsl:22-25`, `:285-287`; `src/render/culling_pass.rs:299-310` |
| M5 story 13, a shadow ray ignores a back face within a pixel's width of its origin | partly, the culling pass's half holds: it writes each slot's drawn LOD's error, in metres at the Instance's largest scale, into the slot's fourth word, in phase one for every slot with a Mesh whether drawn or not, and phase two keeps it; the ray test falls in shaders C | `culling.wgsl:374-377`, `:428-431`, `:450-456`; `shadow_mask.wgsl:192` |
| M5 stories 18, 19 and 20, Chunks past the Load radius drawn as their Far proxy, swapped whole, a Late Chunk keeping it | partly, the culling pass's half holds: a Far proxy's Instance is culled and its LOD selected as any other's, and a held proxy's slot, with no Mesh, draws nothing; which slots hold what is streaming's | `culling.wgsl:395`, `:284-300` |
| M5 story 35, a Thin Primitive narrower than a pixel drawn a pixel wide, coverage rounded to five levels | partly, the culling pass's half holds: the bounds grow by the widening so the frustum and Hi-Z tests keep the widened tube, and its Meshlets are listed for the twin draw; the widening and rounding fall in shaders A | `culling.wgsl:144-181`, `:640-646` |
| M5 story 36, masked geometry drawn through pipelines of its own | holds for the listing: Thin Primitives' and small Instances' indices fill their own region of the packed index buffer, with each phase's draw arguments | `culling.wgsl:502-517`, `:556-562` |
| M5 story 37, a Thin Primitive shadows only receivers whose pixel is no wider than it | partly: `mod.rs` declares `NARROWEST_SHADOW_CLASS_BOUNDS_PER_METRE` (320.0) and `SHADOW_WIDTH_CLASSES` (7) from Rust; `shadow_ray_mask` and the rays fall in shaders C | `mod.rs:316-317`; `src/render/acceleration_structures.rs:971-974` |
| M5 story 39, an Instance smaller than a pixel scaled to a pixel with its coverage by area, dropped at zero coverage | holds for the culling pass's half: coverage worked out for every Instance without a Thin Primitive, the bounds widened by the same scale before the tests, and the drop under CULLING at zero kept samples; the scaling and coverage functions fall in shaders A | `culling.wgsl:399-411`, `:424` |
| M5 story 40, the culling phases' per-slot stages judged against 1 ms | partly: the per-slot stages, `offset_slots`, `offset_blocks` and `append_records`, run in one timed compute pass apart from `append_triangles`; the judging is tooling's | `src/render/culling_pass.rs:42-62`, `:356-390` |

#### Registry keys

| File | Keys | Verdict |
|---|---|---|
| `culling.wgsl:91` | `hillis-steele-partial-sums` | matches: the quote's "compute all partial sums of the array" is the stride-doubling inclusive sum at `:94-102`, read one place back for the exclusive sum at `:104-108`. The addition saturates; min(a + b, 2³² − 1) over unsigned values is associative, so the scan's partial sums are the saturated true sums. The quote carries the result, not the doubling form |
| `culling.wgsl:282` | `meshoptimizer-1.3-lod-selection` | matches the Deviation: the quote's `d = max(0, distance(camera_position, mesh_center) - mesh_radius)` is `max(length(bounds_centre(..)) - radius, 0.0)` with the camera at the origin (`:292`); `e = d * (tan(camera_fovy / 2) * 2 / screen_height)` is `error_per_metre * distance_metres`, with `lod_error_per_metre` = 1 pixel × 2 tan(fov/2)/height (`src/render/renderer.rs:188-200`), so `lod_factor` 1; `lod_ok = e * lod_factor >= lod_error` is `<` in `common.wgsl:216`, the `deviations.md` row, whose wording matches the code (from inside the sphere the distance is 0 and LOD 0 is drawn beside a LOD of error 0). The code multiplies the error by the Instance's scale where the quote's is "in mesh space", the same comparison for an unscaled Instance; the scale's reading falls under L1-S04 |
| `mod.rs:243` | `wgpu-types-30-loop-bounding` | matches: the quote's "backend compilers MAY treat such a loop as unreachable code" is the comment's "a backend compiler may take an infinite loop for unreachable code"; the quote's "the caller MUST ensure that all passed shaders do not contain any infinite loops" is what the comment rests on `shader_tests` for, whose check has a gap (L1-S07) |
| `src/render/culling_pass.rs:79`, relied on by `culling.wgsl:506-507` | `wgpu-types-30-default-workgroups-per-dimension` | matches: `max_compute_workgroups_per_dimension: 65535` is `MAX_WORKGROUPS_PER_DIMENSION`, declared into WGSL at `mod.rs:296`; `append_dispatch` keeps x at ⌈records/rows⌉ ≤ 65,535 |
| `culling.wgsl:210-267`, relied on uncited (ADR 0019 line 15 cites it) | `greene-hiz` | matches in substance, unchanged since M3.5: "choosing the farthest Z" is the minimum in reverse-Z; where the quote finds "the finest-level sample of the pyramid whose corresponding image region covers the screen-space bounding box", the code reads up to 2 × 2 texels at the level where the box spans at most two, which together cover it |
| `culling.wgsl:7-10`, `:378-380`, relied on uncited (ADR 0019 line 15 cites it) | `nanite-two-pass-occlusion` | matches: "Draw what was visible in the previous frame [...] Build HZB [...] Draw what is visible now but wasn't in the last frame" is phase one, the depth pyramid between, and phase two drawing only a slot phase one left `NOT_DRAWN` |
| `src/render/shader_function_tests.rs:1864-1871`, the floors of the precision oracles, uncited | `higham-3.1`, `msl-4.1-rounding`, `msl-4.1-fast-math`, `wgsl-2026-float-accuracy` | match as used in Block B2: additions correctly rounded (`wgsl-2026-float-accuracy`), the u·\|x\| model of each (`higham-3.1`), round toward zero allowed on Metal (`msl-4.1-rounding`), reassociation allowed under fast math (`msl-4.1-fast-math`), which the test answers by checking every order on the CPU |

#### Oracles

| Function | Test | Constant | WGSL floor | Stated error | Stated floor | Verdict |
|---|---|---|---|---|---|---|
| `selected_lod` (`common.wgsl:207-221`) | `the_selected_lod_is_the_coarsest_whose_scaled_error_is_under_the_threshold`, `shader_function_tests.rs:2511` | exact (`assert_eq!` on the LOD, `:2560`) | 0 (Block B1): per LOD two f32 products and one comparison; the four cases on a boundary (`:2531-2534`) multiply dyadic values, so both products are exact and equal, and stay so under a fused multiply-add; every other case's margin from its boundary is at least 13% (0.01 against 0.0115), against a few u | none | no: the doc comment (`:2503-2509`) says what the cases hold, no floor: L1-S08 | exact; all 15 expected LODs recomputed by hand and right; the test fails if `<` became `<=` (cases at `:2531`, `:2533`), if the scale or `lod_count` were ignored (`:2523-2524`, `:2527`) |
| `camera_relative_position` (`common.wgsl:37-46`) | `camera_relative_position_meets_the_precision_bar_within_100_km`, `:1897` | `PRECISION_BAR` 4.3e-5 m, Euclidean, on the GPU's result and on every CPU order (`:1872`) | Block B2: the chain alone, as the file states it, 192.6u per axis in the written order, 2.0e-5 m (4.0e-5 at u = 2⁻²³), 384.9u in the worst other order, 3.97e-5 m; the comparison against the f64 reference also measures the two f32 offsets' rounding, up to 2⁻¹⁹ m each per axis (64u), giving 2.65e-5 m and 4.63e-5 m by the file's own model; by the half-step of the magnitudes the cases reach, at most 320.25u per axis in any order with the inputs, 3.31e-5 m at √3 of it | none in the file; ADR 0014 line 19 states 0.0185 mm measured for every order | yes, `:1864-1871`, without the inputs' rounding: L1-S06 | explained in the written order (1.6× by the stated model with the inputs); in any order tight by that model (4.63e-5 over 4.3e-5) and explained by the cases' magnitudes (1.30×); under round toward zero with reordering over, as the file says |
| `point_light_position` (`common.wgsl:127-139`) | `point_light_position_meets_the_precision_bar_within_100_km`, `:1968` | `PRECISION_BAR` 4.3e-5 m (`:1872`), with `PRECISION_COORDINATES` (`:1876-1885`) shared | Block B2: 64.3u per axis written, 64.6u in any order, as stated; with the offsets' rounding 128.3u, 1.33e-5 m (1.99e-5 with the chain at u = 2⁻²³) | none | yes, `:1869-1871`, without the inputs' rounding (L1-S06) | explained, 3.2× (2.2× under round toward zero) |
| `pack_index`, `unpack_index` (`common.wgsl:320-326`), called by `culling.wgsl:650`; comment changed in scope | `a_packed_index_holds_its_meshlet_record_above_six_bits_of_vertex`, `:1639` | exact (`assert_eq!`, `:1681-1690`) | 0: shifts and masks on u32; each 16-bit half crosses the harness as an f32, exact below 2²⁴; the record's top case, `0x03FF_FFFF`, is the 26 bits above six of vertex | none | yes, `:1677` ("WGSL floor 0: integer shifts and masks, on halves f32 holds exactly") | exact |

##### Block B1: `selected_lod`

`common.wgsl:216`: `lod_errors_metres[lod] * scale` (one rounding) against
`error_per_metre * distance_metres` (one rounding), compared exactly. With
the 1440-row pixel 0.000575 (2 × tan 22.5° / 1440 = 0.82843 / 1440 =
5.753e-4, which the doc's "0.575 mm" rounds) and the doubling chain 0,
0.01, 0.02, ..., 0.64: 10 m gives 0.00575, under no error but LOD 0's, so
0; 20 m gives 0.0115, over 0.01 and under 0.02, so 1; 100 m gives 0.0575,
over 0.04 and under 0.08, so 3; scale 2 turns 0.04 into 0.08, so 2; scale
0.5 turns 0.08 into 0.04, so 4; twice the pixel at 10 m is 0.0115, so 1;
10 km gives 5.75, over 0.64, so 7, or the coarsest a chain of 3 or 1 or 0
LODs holds, 2, 0, 0; 0 m gives 0, so 0. The whole-metre chain with a
threshold of 0.5: 2 m gives 1.0, equal to LOD 1's 1.0 and not under it, so
0; 2.5 m gives 1.25, so 1; scale 2 at 4 m gives 2.0 against 2.0, so 0; at
5 m 2.5 against 2.0 and 4.0, so 1. Every expected value at `:2520-2534`
matches. The smallest margin off a boundary is 0.0115 against 0.01, 13% of
the larger; one rounding of each product is at most 2⁻²⁴ relative, so no
rounding flips a case. The four boundary cases multiply 1.0 × 1.0 against
0.5 × 2.0, 0.5 × 2.5, 2.0 × 1.0 against 0.5 × 4.0, and 2.0 × 1.0 against
0.5 × 5.0: dyadic, exact in f32, so equality is exact, and a fused
multiply-add of an exact product changes nothing. Not exercised: a
`lod_count` above `MESH_MAX_LODS`, which the loop's `min` bounds.

##### Block B2: the precision bar's floor

`common.wgsl:44-45`: `vec3<f32>(anchor - camera_anchor) * ANCHOR_CELL_SIZE`
is exact (an i32 difference of at most 3,125 cells for 200 km, times 64,
under 2²⁴). The written order then rounds three times per axis:
`offset - camera_offset`, |x| < 64; `anchor_position + (...)`, the
pivot's position relative to the camera, |x| ≤ 0.3 + 127.9 = 128.2 for
the local position (−127.9, 3.3, 64.7) at `:1913`; `+ local_position`,
|x| ≤ 0.3. In the u·|x| model (`higham-3.1`'s δ ≤ u per operation, additions
correctly rounded by `wgsl-2026-float-accuracy`): u(64 + 128.3 + 0.3) =
192.6u per axis, √3 × 192.6 × 2⁻²⁴ = 1.99e-5 m, as `:1865-1866` states.
The worst other order, (offset + local) − camera offset with the anchor
term last, sums |x| ≤ 192.3, 192.3 and 0.3: 384.9u, √3 × 384.9 × 2⁻²⁴ =
3.97e-5 m, as `:1867-1868` states.

What the assertion measures is more. `Case::test_vectors` (`:2237-2251`)
takes `anchor_and_offset` (`src/render/instance.rs:8-12`), whose offsets are
the f64 remainder in [0, 64] rounded to f32 by `as_vec3()`; the reference is
the f64 `camera_relative` (`:1951`). Each offset rounds by up to half its
step, 2⁻¹⁹ m = 1.91e-6 m for a value in [32, 64), which is 32u; the
instance's and the camera's together 64u per axis. Added: written order
256.6u, √3 × 256.6 × 2⁻²⁴ = 2.65e-5 m; worst other order 448.9u, √3 ×
448.9 × 2⁻²⁴ = 4.63e-5 m, over the 4.3e-5 bar. The model overstates: by
the half-step of each partial sum's actual magnitude, on the x axis with
the local position −127.9 m the anchor term is 64, 128 or 192, and the
worst order, (anchor + offset) − camera offset then + local, holds two
partial sums in [128, 256), each rounded by at most 2⁻¹⁷ (128u), before the
final 0.3 (0.25u): 256.25u, with the inputs' 64u 320.25u per axis; the z
axis (local 64.7 m, anchor term −128) reaches the same through (offset +
local) − camera offset, and the y axis (local 3.3 m) 192.25u. At √3 ×
320.25u, 3.31e-5 m, under the bar by 1.30×; with the axes' own worsts,
√(320.25² + 192.25² + 320.25²) = 492u, 2.93e-5 m, 1.47×; consistent with
ADR 0014's measured 0.0185 mm. At u = 2⁻²³ on the GPU and reordering, the
file's own 7.9e-5 stands.

`point_light_position` (`common.wgsl:132-138`): local position 0, so the
chain is `offset - camera_offset` (|x| < 64) and `+ anchor_position`
(|x| ≤ 0.3) and an exact `+ 0.0`: 64.3u, √3 × 64.3 × 2⁻²⁴ = 6.64e-6 m; with
the inputs' 64u, 128.3u, 1.33e-5 m, 3.2× under the bar; at u = 2⁻²³ the
chain doubles to 128.6u, with the inputs 192.6u, 1.99e-5 m, 2.2× under.

### the shadow mask and its rays

Read in full at the frozen commit `e5c55a8d`: `src/render/shaders/shadow_mask.wgsl` (702 lines), `src/render/shaders/common.wgsl` (408 lines; this reader owns `CameraUniform`'s TLAS fields, `camera_relative_position` as the shadow ray uses it, `position_at`, `segment_meets_box`, `shadow_ray_mask` and the octahedral functions), `src/render/shaders/dynamic_tlas_marker.wgsl` and `blas_refit_marker.wgsl`, and in `src/render/shader_function_tests.rs` the oracles `a_shadow_ray_includes_the_width_classes_whose_upper_bound_its_receivers_pixel_is_within` (`:2463-2502`), `a_segment_meets_a_box_where_some_point_of_it_lies_in_the_box` (`:2013-2212`), `position_at_rebuilds_the_point_a_depth_shows_through_a_pixel` (`:1696-1765`) and `octahedral_coordinates_decode_to_the_unit_vector_they_encode` (`:1767-1862`), the last three because their bodies changed in scope (floor comments, a glam call) and they exercise this reader's functions. The scope's diff (`git diff 8bc34eb1..e5c55a8d`) of `shadow_mask.wgsl` is 75 lines: `first_intersection`'s `mask` argument, `LOD_ERROR_MARGIN`, the back-face pass by the hit Instance's drawn LOD error, the receiver's width classes, and the point light's `unshadowed_range` test removed; `common.wgsl` gained `CameraUniform.lod_error_per_metre` and `pixel_size_per_metre`, `shadow_ray_mask`, and lost `LightData.unshadowed_range`. ADRs read whole, status lines and amendments included: 0008, 0014, 0015, 0019, 0020, 0026, 0027. Spec stories read: every story of `spec-611.md` (M4) and `spec-804.md` (M5); those this reader's files implement are under *Spec stories*. Opened outside the files, to judge a clause and not audited: `src/render/acceleration_structures.rs` (`instance_mask`, `tlas_instance`, the static TLAS's anchor and re-anchor, `NARROWEST_SHADOW_CLASS_BOUNDS_PER_METRE`, `SHADOW_WIDTH_CLASSES`), `src/render/renderer.rs:705-745` (the frame's TLAS anchor and the dynamic TLAS's fill), `src/render/uniforms.rs:31-64` (`CameraUniform`), `src/render/shaders/culling.wgsl:270-300`, `:345-462` (where each slot's fourth word, the drawn LOD error, is written), `src/cook/mesh.rs:290-380` (the LOD error's units), `src/render/shaders/mod.rs:97-104`, `:186-245`, `:290-320` (the module's enables, which modules run without loop bounding, the declared constants), `src/render/shadow_mask_pass.rs:25-70`, `docs/performance.md:34`, and the commit messages of the scope's three shadow-ray changes for the decisions they record. `docs/audit/deviations.md`, `scripts/cycles-reference/DIFFERENCES.md`, `docs/references.toml` and `CONTEXT.md` were read for what is not a Finding. M3.5's shaders section (`docs/audit/m3.5/correctness.md:1505-2613`, its reader D) was read for what was filed; rows below that only restate an M3.5 verdict on unchanged code give the frozen commit's lines.

#### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0020 | "Direct light is one hard shadow ray per pixel per light" | holds: one `shadowed` call per light entry of a surface; inside it the one ray runs in at most two segments, the nearest hit, then, past a back face it passes, any hit, through `first_intersection`, the one call site that queries | `shadow_mask.wgsl:293-366`, `:364`; `:185-202`; `:99-125` |
| 0020 | "skipped where the light adds nothing (since [...], a light the surface's interpolated normal turns from is hidden without one; amended 2026-11-04 [...]: the test reads the interpolated normal ADR 0019's surface normals carry, not the face's)" | holds: a directional light the interpolated normal turns from by more than sin 2° is hidden with no ray, a point light likewise unless another sample of the surface faces it; the amendment's wording now matches the code, so M3.5's drift on "face" no longer stands | `:306-309`, `:332-358`, `:248-252` |
| 0020 | "from an origin offset along the surface's normal" | holds: along the stored triangle normal by the larger of 2^-16 of the largest coordinate in either frame (1 m at least) and 1/32 of a pixel's width, to the light's side where the triangle faces it, inside the surface otherwise | `:168-177`; `:36`, `:51` |
| 0020 | "and composed in the TLAS's frame" | holds: `camera_relative_position(camera.anchor, camera.offset, camera.tlas_anchor, 0, position)` is (camera Anchor − TLAS Anchor) × 64 in integers, plus the camera's offset, plus the camera-relative point | `:161-167`; `common.wgsl:37-46` |
| 0020 | "one toward each point light that reaches the pixel, ending at the light" | holds: every point light whose range reaches the surface gets a ray ending at `sqrt(distance_squared)`; since the scope removed `LightData.unshadowed_range`, no point light in range is passed over for casting no shadow | `:318-331`, `:364`; `common.wgsl:166-171` |
| 0020, 0019 | "A compute pass traces them from the depth target, at each pixel's nearest sample and, where its other samples show another surface, at the farthest of those too" | holds, unchanged in scope: the greatest of the four depths, first such sample; samples off its triangle's plane by more than a pixel's width, the least depth among them listed and traced into layer 1 | `:418-426`; `:434-470`; `:501-520` |
| 0020 | "at a pixel whose verdict for one of the first seven directional lights differs from a 4-neighbour's among the lights both traced, or whose nearest sample turns from that light while another sample of its nearest surface faces it, toward that light again from each other sample that shows its nearest surface" | holds, unchanged in scope | `:546-551`, `:557-569`, `:573-611`, `:651-702` |
| 0020 | "where the nearest sample turns from a point light while another sample of its nearest surface faces it, the surface's one bit for that light is the verdict of a ray from the first such sample" | holds, unchanged in scope | `:332-359` |
| 0020, 0019 | "holds 59 lights per pixel: a light past them casts no shadow there, and Frame stats count it" | holds: bit = the light's entry, bits 0-31 in `x` and 32-58 in `y`, the Froxel slice from bit 27 of `y` (59 − 32); an entry at 59 or past it, in range, is counted once per surface by one `atomicAdd`; `SHADOW_MASK_LIGHTS + log2 FROXEL_SLICES == 64` is asserted in Rust | `:298-301`, `:325-328`, `:368-373`; `src/render/shadow_mask_pass.rs:35`, `:68-70` |
| 0020 | "Moving Instances form the dynamic TLAS [...] and a shadow ray the static TLAS lets through is traced against it" | holds: queried unless empty, unless a first hit was asked for and found, or unless the segment misses the dynamic box grown by 2^-16 of the largest coordinate; a nearest-hit query runs only to the static hit; both queries take the same flags and the same `mask` | `:102`, `:106-124` |
| 0020 | Consequences: "A static build keeps the static TLAS's Anchor until the camera's is more than a Chunk's edge from it along an axis, [...] so the camera adds less than 4.9 mm to a shadow ray's offset once that build is bound" | holds, shader side: the offset is 2^-16 times the largest coordinate of `tlas_position` and `position`, and `tlas_position` exceeds `position` by the camera's place in the TLAS's frame, under 4 cells of 64 m plus its offset within a cell, so under 320 m per axis: 320 × 2^-16 = 4.88 mm | `:161-172`; `src/render/acceleration_structures.rs:195` (`REANCHOR_CELLS`) |
| 0020 | Consequences: "the dynamic TLAS, rebuilt each frame in the bound static TLAS's frame, follows in the frame a new one is bound" | holds, shader side: one `camera.tlas_anchor` frames both queries' origin and the dynamic box; the Renderer reads the bound TLAS's Anchor after the frame's update and fills the dynamic TLAS and the uniform from that one value | `:111-118`, `:161-167`; `src/render/renderer.rs:724-737` |
| 0020 | Consequences: "Every BLAS is built from its Mesh's LOD 0, [...] whichever LOD the main pass draws" | the build falls in render; the mask's half holds for a ray from a surface onto its own Instance: a back face met no higher above the surface than 3 times the drawn LOD's error is passed, so a coarser LOD that lies inside LOD 0 does not shadow itself; the rule also passes other Instances' back faces: L1-S10 | `:127-130`, `:186-201` |
| 0020 | Consequences: "Shadow and probe rays query both" | holds for shadow rays; no probe ray exists | `:99-125` |
| 0020 | "The `crowd` Reference scene's figures [...] cast their shadows through rigid per-bone shadow proxies, Moving Instances in that TLAS" | holds, shader side: the proxies' entries carry the ordinary bit every ray's mask holds, and the drawn LOD error their slot carries is their Mesh's LOD 0 error, 0, so their back faces always shadow | `common.wgsl:287`; `culling.wgsl:428-431`; `src/render/acceleration_structures.rs:990-1008` |
| 0020 | "Direct light is Temporal class (a)" | holds: the pass reads this frame's depth, normals, lights, Froxel lists and the culling pass's per-slot words; the static TLAS's lag is the decided one (Notes on a stale slot word) | `:192`, `:236`, `:418-423` |
| 0020 | "Soft sun shadows [...] a denoiser is class (c)"; "Sun penumbrae reopen when area lights arrive" | not applicable: one hard ray, no jitter | `:364` |
| 0020 | "A constant ambient term stands in for indirect light" | falls in shaders A | |
| 0020 | Considered option, soft shadows: "contact shadows stay sharp either way" | holds in kind, with a gap: the back-face pass can drop a contact shadow under an open Mesh drawn at a coarser LOD: L1-S10 | `:193-195` |
| 0019 | "a compute pass traces, from each pixel's nearest sample, one shadow ray toward each light that reaches it and keeps whether it is hidden in the shadow mask, and, where the pixel's other samples show another surface, traces from the farthest of those too, into the mask's second layer" | holds | `:408-493`, `:478`; `:501-520`, `:519` |
| 0019 | "each sample's triangle normal and the normal interpolated over it [...] which the shadow mask pass reads so that a coarse smooth-shaded mesh's terminator follows its interpolated normals rather than its triangles" | holds: the turn tests read `interpolated_normal`; `triangle_normal` decides the origin's side, the height the back-face pass measures, and the second-surface plane | `:233-243`, `:306`, `:332`, `:173-177`, `:190`, `:438` |
| 0019 | "the second surfaces of 13.7% of its pixels are traced from a packed list, as tile-based deferred renderers schedule per-sample shading" | holds, unchanged in scope: `atomicAdd` on the count, the opening entry of each 64-entry run grows the indirect dispatch by `atomicMax` in rows of `MAX_WORKGROUPS_PER_DIMENSION`; the second entry point returns past the count read once per workgroup | `:455-468`; `:501-513` |
| 0019 | "a pixel of three surfaces or more gives its middle ones the farthest's" | holds: one word for every off-plane sample, traced from the least depth among them | `:445-454` |
| 0019 | "each fragment reads the bits traced from the surface it shades [...]" | falls in shaders A; the layout it reads is what this pass stores: second-surface sample bits 0-3, each edge light's four sample verdicts above them | `:478`, `:485-489`, `:697-701` |
| 0019 | "the lit pass sums lights and the shadow mask gives them its bits in buffer order" | holds: a light's bit is its entry, the directional lights first, then the Froxel's list in light order (every point light in buffer order with Froxels off); a point light out of range keeps its entry's bit unused, as the lit pass indexes it (`main.wgsl:444-446`) | `:282-293`, `:316`, `:322-324` |
| 0019 | the mask's word follows the Froxel of the sample it was traced from, its slice in the second half's top bits | holds, unchanged in scope | `:283-287`, `:372-373` |
| 0019, checklist | "The GPU builds the frame's draw in slot order [...] never in the order atomics land" | holds for the mask: list entries land in atomic order, but each writes only its own pixel's words; the dispatch sizes are `atomicMax`, the overflow an `atomicAdd`, both commutative; no result depends on which lands first | `:455-468`, `:631-642`, `:369`, `:519`, `:697-701` |
| 0019 | "the shadow mask pass rebuilds each pixel's position from it and reads each sample's normals from the surface normals the depth passes write, so shadow rays need no G-buffer" | holds | `:233-243`; `common.wgsl:55-62` |
| 0015 | "Depth is cleared to 0" | holds: depth 0 is the background at every use | `:272`, `:390`, `:435`, `:445`, `:685` |
| 0015 | "compared with `Greater`" | holds: the nearest sample is the greatest depth, the farthest second surface the least, the sentinel 2 lies past the near plane's 1 | `:425`, `:452-454`, `:594`, `:676` |
| 0015 | "with no far plane" | holds: the view depth is the near distance over the depth, finite for every depth the tests above let through | `:284`; `common.wgsl:58` |
| 0015 | "shadow comparisons follow the same direction" | holds: the ray code compares no depth; the back-face pass compares a height above the surface with an error and a distance along the ray with the light's | `:190-196` |
| 0014 | "The GPU never holds a World position in f32: it reads an integer anchor [...] plus an f32 offset, and shaders compose positions relative to the camera" | holds: the TLAS-frame origin is composed from the Anchors' difference in `vec3<i32>`; point lights through `point_light_position` | `:161-167`, `:319`; `common.wgsl:37-46`, `:127-139` |
| 0014 | "Subsystems that compute in f32 ([...] ray-tracing acceleration structures) convert at their boundary into a local frame they own" | holds: one composition per ray into the TLAS's frame; the dynamic box arrives in it | `:161-167`, `:111-113` |
| 0014 | amended: "a build moves it only once the camera's Anchor strays more than a Chunk's edge (4 cells) along an axis, so, once that build is bound, the camera is again within 320 m of the frame's origin along each axis and a shadow ray composes and lifts its origin 10 km from where it started as at the origin" | holds, shader side: the lift is 2^-16 of the larger of the point's camera-relative and TLAS-frame coordinates, both of which stay near the camera wherever it is in the World; the re-anchor itself is render's | `:161-172` |
| 0027 | Consequences: "A pixel whose samples a mask splits between two surfaces is listed for the shadow mask pass's second surface, as any edge pixel is" | holds: a sample a fragment's mask did not keep holds the depth of the surface behind it, and is listed when that lies off the nearest sample's triangle plane by more than a pixel's width, the rule of every edge; the pass reads depths only, no coverage or sample mask | `:439-447` |
| 0027 | "The depth passes write the mask, so only the kept samples take the fragment's depth" | holds as the mask needs it: the nearest sample of a pixel a Thin Primitive or small Instance keeps one sample of is that sample, traced from the widened surface the depth shows | `:425-426`, `:471-477` |
| 0008 | "Lighting is ray-traced probe GI with hard ray-traced shadows (amended 2026-10-13, ADR 0020)"; no temporal tricks | holds: the mask reads no earlier frame's screen data and accumulates nothing; the one word read per hit is the current frame's for every slot in use (Notes: a static TLAS entry past the slots in use) | `:192`; `culling.wgsl:428-431`, `:446-456` |
| 0026 | Settled by the draw, "Its shadows": "While drawn it holds one static TLAS entry under the mask bit an ordinary Instance has, so it casts and takes the sun's shadows" | holds, shader side: bit 0 is in every ray's mask, the sun's and each point light's, so no width class can hide a Far proxy; the entry's bit is Rust's (`instance_mask` gives a Mesh without a thin radius bit 0) | `common.wgsl:287`; `src/render/acceleration_structures.rs:990-994` |
| 0026 | "A Far proxy's Mesh builds a BLAS from its LOD 0 at load" | the build falls in render; a receiver on a Far proxy drawn at a coarser LOD of its own chain passes its own LOD 0's back faces by the same rule as any Instance | `:186-201` |
| 0026 | "The ground past the View distance [...] traced by no ray" | falls in render (it takes no TLAS entry) | |
| checklist | "No World position reaches the GPU in f32" | holds (ADR 0014 rows) | `:161-167` |
| checklist | "WGSL that compares or reduces depth (shadow, Hi-Z) keeps reversed depth's direction" | holds (ADR 0015 rows) | |
| checklist | "WGSL [...] has no divide-by-zero or NaN at grazing angles" | holds in the scope's code: `height` and `shadow_ray_mask` divide by nothing; `shadow_ray_mask` maps a NaN, infinite or zero pixel width to a mask that still holds bit 0 (Oracles block); a miss's `instance_custom_data` indexes a bounds-checked buffer and its result is discarded (Notes) | `:190-197`; `common.wgsl:283-288` |
| checklist | "calls no `textureSample` under non-uniform control flow" | holds: `textureLoad` only | `:236`, `:346`, `:418-423`, `:517`, `:588-601`, `:670-675` |
| checklist | "A newly used wgpu feature or limit joins the Capability floor" | holds in the WGSL: a run-time cull mask and a hit's custom data are parts of the ray query the floor already requires; the pass's ninth storage buffer is render's | `:102`, `:118`, `:192` |
| checklist | "A new or changed shading function in WGSL has an oracle test" | holds: `shadow_ray_mask` has one (Oracles); `shadowed`'s back-face pass, in a part no oracle harness can load (it needs a TLAS), is tested by the Renderer's GPU test of a back face at several heights | `shader_function_tests.rs:2463-2502` |
| checklist | "Identifiers are full words, WGSL included" | holds for the scope's names (`pixel_bits`, `narrower_classes`, `drawn_lod_error`, `any_hit`, `height`) | `common.wgsl:283-287`; `:188-195` |
| checklist | comments cite "only an upstream bug, adapted code's source and licence, a pinned third-party source or an ADR" | holds: no ticket in the scope's comments; registry keys and ADRs only | `:127-129`, `:141-148` |
| checklist | "Every name the change renames, removes or changes the behaviour of [...] is grepped" | holds for `unshadowed_range` (no stale mention in ADRs, `CONTEXT.md` or code) and `first_intersection`'s arguments (one caller); the doc comments of `first_intersection` and `CONTEXT.md`'s Thin Primitive lag their code: L1-S11, L1-S12 | `:83-99`; `CONTEXT.md:21-22` |

ADR 0020 at the frozen commit has no clause on Thin Primitives' shadows by width class or on a back face passed within the drawn LOD's error, and no other ADR has one: those rules are recorded in the M5 spec's design notes ("Thin Primitives carry an instance-mask bit per width class, one class per doubling"; "Near back faces ignored within a pixel's width times the threshold") and in `CONTEXT.md`'s Thin Primitive entry, so they are judged under *Spec stories*. ADR 0026's "the mask bit an ordinary Instance has" is the one ADR sentence that presumes the instance mask.

#### Spec stories

| Story | Verdict | Where |
|---|---|---|
| M4 story 49, "the static TLAS's anchor to move with the camera, so that a shadow ray's origin lifts by no more than its distance from the camera allows, at 10 km as at the origin" | holds, shader side: the origin is composed in the bound TLAS's frame from the uniform's `tlas_anchor`, and its lift is 2^-16 of the larger of its camera-relative and TLAS-frame coordinates, so it follows the camera, not the World origin; the move itself is render's | `shadow_mask.wgsl:161-172` |
| M4 story 52, "the dynamic TLAS to follow the static TLAS's anchor in the frame the new static TLAS swaps in, so that both answer rays in one frame" | holds, shader side: both queries take the one origin composed from `camera.tlas_anchor`, and the dynamic box arrives in that frame; the Renderer fills both from the bound TLAS's Anchor after the frame's update | `:102`, `:111-118`; `src/render/renderer.rs:724-737` |
| M4 stories 48, 50, 51, 60, 70 (static TLAS membership, the off-frame rebuild, its frames-until-traced ceiling, 10,000 Moving Instances, the build time reported) | falls in render | |
| M4 story 36, "the Materials that rays reach off screen given a floor priority [...] so that shadows [...] never sample a missing mip" | not applicable to the shadow mask: its rays are `RAY_FLAG_FORCE_OPAQUE` and sample no Material | `:102`, `:118` |
| M5 story 12, "every BLAS built from its Mesh's LOD 0 [...] so that a LOD switch changes no BLAS and starts no static TLAS build" | the build falls in render; the mask holds its consequence: a ray from a coarser LOD passes its own LOD 0's back faces by the drawn LOD's error, so the BLAS need not follow the LOD | `:186-201` |
| M5 story 13, "a shadow ray to ignore a back face within a pixel's width of its origin, so that a drawn LOD sitting just inside LOD 0 does not shadow itself" | holds in intent, not in letter: the code passes a back face met no higher above the surface than `LOD_ERROR_MARGIN` (3) times the LOD error the hit Instance draws, a height bounded by the LOD threshold, rather than one within a pixel's width along the ray, which `docs/performance.md:34` measured worse (`orrery` frame 59: 4,100 pixels off LOD 0 forced with a pixel's width along the ray, 500 at 3.0); the rule also passes the back faces of other Instances, which the story's purpose does not need: L1-S10 | `:127-130`, `:186-201`; `docs/performance.md:34` |
| M5 story 14, "the shadow mask pixels that differ from LOD 0 counted and limited" | falls in tooling (the count is `praxis shadow-mask-difference`) | |
| M5 story 27, "Far proxies to cast and receive the sun's shadows" | holds, shader side: every ray's mask holds the ordinary bit a Far proxy's entry carries, and a pixel on a Far proxy is traced as any surface | `common.wgsl:287`; `:293-366` |
| M5 story 29, "shadow-ray time reported with Far proxies traced and not" | falls in render and bench | |
| M5 story 35, "a Thin Primitive narrower than a pixel drawn a pixel wide" | falls in shaders A; the mask traces the widened surface the depth target shows, as any surface | `:233-243` |
| M5 story 37, "a Thin Primitive to cast a shadow only onto receivers whose pixel is no wider than it, so that its shadow ends where it would break into dots" | holds in the reading `CONTEXT.md` records ("whose shadow ends where it would fall between a pixel and half a pixel wide"), not in the story's letter: a ray includes a class whose upper bound, twice its lower, is no narrower than the receiver's pixel, so a receiver whose pixel is up to twice a tube's width still takes its shadow; the rule's range ends short of both readings for a tube under 3.125 mm, one of 400 mm or more, and a Mesh that mixes thin and ordinary Primitives: L1-S12 | `common.wgsl:282-288`; `src/render/acceleration_structures.rs:990-1008` |

#### Registry keys

| File | Keys | Verdict |
|---|---|---|
| `shadow_mask.wgsl:32` | `wachter-binder-2019-offset-error` | matches, unchanged in scope: the quote's "the distance of the intersection to the plane of the surface is proportional to the distance from the origin" and "the size of the surface [...] becomes dominant for triangles very close to the origin" are the offset's share of the largest coordinate and its 1 m floor (`:168-172`); the share 2^-16 rests on the comment's own 256-ulp argument, as M3.5 found |
| `shadow_mask.wgsl:41`, `:42` | `d3d11.3-rasterizer-precision`, `vulkan-subpixel-precision-bits` | matches, unchanged: "EXACTLY 8 bits" of fraction is the comment's 1/256 of a pixel, the required minimum of 4 bits its "as coarse as 1/16" |
| `shadow_mask.wgsl:94` | `nvidia-2022-rtx-best-practices-shadow-rays` | matches: "the query objects must hold state [...] This consumes registers and complex user code may limit occupancy" is the one call site every segment and both TLAS go through (`:99-125`, `:186`), now with the mask a run-time value too; "Terminate rays on the first hit when possible. When resolving the correct closest hit is not required (as for shadow rays)" is the second segment's `RAY_FLAG_TERMINATE_ON_FIRST_HIT` (`:200`); since the scope the first segment of every ray asks for the nearest hit (`:182`), which the back-face pass needs to know which triangle comes first, so the closest hit is required there and the quote's "when possible" holds |
| `shadow_mask.wgsl:128` | `meshoptimizer-1.3-simplify-error` | matches in substance: the quote's "Target error is an approximate measure of the deviation from the original mesh using distance normalized to `[0..1]` range" is the comment's "an approximate measure"; the quote speaks of the target error, and the cooked error is the simplifier's reported error on the same normalised distance, summed down the chain and times `simplify_scale` (`src/cook/mesh.rs:299-310`, `:375`), so the comment applies the quote's word to the reported error; "no bound" is the comment's inference, which the quote's "approximate" supports. The second clause, "`orrery`'s spheres meet LOD 0 up to three times theirs high", rests on `docs/performance.md:34`, which reports pixel counts per margin (3.0 and 1,000.0 equal on all 33 frames), not a measured height (Notes) |
| `shadow_mask.wgsl:148` | `tabellion-lamorlette-2004-ray-origin-offset` | matches what it is cited for: "the offset distance used in this algorithm needs to be bigger than the maximum offset between the coarse and micro-polygon geometric tessellations" is the comment's "a tolerance for hits near the origin has to exceed the largest offset between the two tessellations", with the roles mirrored (here the ray starts on the coarse tessellation and traces the fine one), and `LOD_ERROR_MARGIN` × the drawn LOD error stands in for that largest offset. The quote's first sentence, "Traditional biasing techniques that ignore intersections near the ray origin cannot be applied here, since they would create significant light or shadow leak problems", is answered for a surface's own Mesh by passing back faces only, and not for another Instance's open Mesh: L1-S10 |
| `shadow_mask.wgsl:407`, `:617` | `lauritzen-2010-per-sample-scheduling` | matches, unchanged: "Shade sample 0 for the whole tile [...] Pack a list of pixels that require per-sample shading [...] Redistribute threads" is the nearest surface traced in place and the two lists traced by indirect dispatches (`:455-468`, `:631-642`) |
| `shadow_mask.wgsl:431` | `lauritzen-2010-edge-detection` | matches in form, unchanged: "Compare sample depths to depth derivatives" is done against the nearest sample's triangle plane, with a one-pixel-width threshold the code chooses (`:439-447`) |
| relied on uncited: `common.wgsl:282-287` (the ray's mask meets an entry's where they share a bit; the entry's mask is 8 bits) | `wgpu-30-tlas-instance-fields`, cited at `instance_mask` (`src/render/acceleration_structures.rs:989`), which `shadow_ray_mask`'s comment names | matches: "Reports hit only if `(shader_cull_mask & tlas_instance.mask) != 0u`" and "`mask: u8`" are the ordinary bit 0 and the seven class bits 1 to 7 that both sides use; the ray's mask always holds bit 0, so it is never 0 and never misses an ordinary entry |
| relied on uncited: `shadow_mask.wgsl:191-192` (a hit's custom data is its Instance's slot) | `wgpu-30-tlas-instance-fields` | matches: "This must only use the lower 24 bits" holds for every slot the Budgets give, which `src/render/acceleration_structures.rs:1283` tests |
| relied on uncited: `common.wgsl:284-286` (a float of 1 or more holds the floor of its base-2 logarithm in its exponent, bias 127) | none in the registry | the WGSL specification's binary32 format and `bitcast` are the source; the registry holds no key for them. Correct as read: for a normal f32 x ≥ 1, bits 23-30 hold ⌊log₂ x⌋ + 127 and bits 0-22 are 0 only at a power of two |
| relied on, not applicable: loop bounding | `wgpu-types-30-loop-bounding` | not applicable: the shadow mask module is created with `create_shader_module` and keeps loop bounding (`src/render/shadow_mask_pass.rs:145`; `LOOPS_UNBOUNDED` is `[MAIN, CULLING]`, `src/render/shaders/mod.rs:193`); its back-face loop ends after two turns regardless (`:185-202`) |

#### Oracles

| Function | Test | Constant | WGSL floor | Stated error | Stated floor | Verdict |
|---|---|---|---|---|---|---|
| `shadow_ray_mask` (`common.wgsl:283-288`) | `a_shadow_ray_includes_the_width_classes_whose_upper_bound_its_receivers_pixel_is_within`, `shader_function_tests.rs:2463-2502`, 17 cases | exact: `assert_eq!` of each mask against a literal (`:2495-2500`) | 0 in the integer chain; one rounded product, `(0.5 · p) · 320`, decides the class, and flips a case only where 160 p lies within one rounding of a power of two from 2⁰ to 2⁶; seven cases (0.00625, 0.0125, 0.025, 0.05, 0.1, 0.2, 0.4 m) sit 0.25u above one, and every quoted backend's rounding, to nearest or toward zero, lands them on it (block C1) | none | no: the doc comment (`:2463-2464`) gives the classes' bounds, not that seven cases sit on them and pass only because no backend rounds a product up: L1-S13 | exact; the seven bound cases depend on the rounding direction (block C1) |
| `segment_meets_box` (`common.wgsl:75-103`) | `a_segment_meets_a_box_where_some_point_of_it_lies_in_the_box`, `:2013-2212`, 19 cases, unchanged in scope but for the floor comment | none: a boolean per case (`:2210`) | a verdict flips only within about 24u × M of a separating boundary, M the largest coordinate from the box's centre: ≤ 6e-5 box units for M ≤ 37.5 and 1.4e-3 for the ray from 1000 away at u = 2^-24; 1.1e-4 and 2.9e-3 at u = 2^-23 (block C2) | none | yes, `:2204-2209`, at u = 2^-24 only | exact: the smallest non-exact margin is 1.0 box unit, four orders over the floor in either rounding; the two on-face cases are exact by structure |
| `position_at` (`common.wgsl:55-62`), unchanged in scope | `position_at_rebuilds_the_point_a_depth_shows_through_a_pixel`, `:1696-1765`; body changed in scope (a glam call renamed, comments) | Euclidean distance ≤ 2^-20 × the largest \|coordinate\| (`:1759`) | ≈ 41u · w over the three components, 2.5e-6 · w, 4.9e-6 · w at u = 2^-23, w the view depth (block C3) | none; the doc comment (`:1696-1702`) motivates the bound as "sixteen times inside the 2^-16 a shadow ray's origin leaves its surface by" | yes, `:1756-1758`, both roundings, and it says "tight" | tight, as the comment states: the floor is 2.6 to 6.4 times the constant, which passes where roundings do not align |
| `octahedral_encode`, `octahedral_decode` (`common.wgsl:389-401`), round trip, unchanged in scope | `octahedral_coordinates_decode_to_the_unit_vector_they_encode`, `:1767-1862`; body changed in scope (comments) | 4 × `TOLERANCE` = 4e-6 on the direction (`:1851`) | ≈ 30u: 1.8e-6 at u = 2^-24, 3.6e-6 at 2^-23 (block C4) | `TOLERANCE`'s "Results land within 2e-7 of theirs on the M4 Pro" (`:15-16`), for every test that uses it | yes, `:1848-1849` | explained: the constant is 1.1 to 2.2 times the floor |
| the same, from 8 bits (the store simulated on the CPU, `:1810-1818`) | the same test | 0.95° (`:1834`) | the quantisation's worst case: half an 8-bit step, 1/255, on each coordinate near a face's centre moves the vector by √6/255 = 9.606e-3 across a length of 1/√3, atan(√18/255) = 0.016638 rad = 0.9532°; f32 adds about 2e-6 rad (block C4) | none | stated, wrong: `:1831-1833` says "0.951°, which this takes to three figures" | tight: the worst case lies 0.0032° above the constant, and the direction normalize(84, 84, 87) reaches 0.9532° or 0.9527° whichever way its half code rounds: L1-S09 |

##### Block C1: `shadow_ray_mask`

The chain, `common.wgsl:284-287`, at `p` = `pixel_metres`:

1. `0.5 * pixel_metres`: a multiplication by a power of two, exact for every p above 2^-125 (no subnormal result).
2. `* NARROWEST_SHADOW_CLASS_BOUNDS_PER_METRE` (320.0, declared from `src/render/acceleration_structures.rs:971` by `src/render/shaders/mod.rs:316`): one product, correctly rounded on every backend the registry quotes (`wgsl-2026-float-accuracy`: "x * y: Correctly rounded"; `vulkan-spirv-precision`: "OpFMul [...] Correctly rounded"; `d3d11.3-fp32-rules`: "within 0.5 ULP [...] multiplication"; `msl-4.1-fast-math` Table 8.2: "x * y: Correctly rounded"), or rounded toward zero where Metal uses that mode (`msl-4.1-rounding`: "Either round ties to even or round toward zero rounding mode may be supported"). Metal's fast math may reassociate (`msl-4.1-fast-math`: "allow reassociation"), to `p · 160` (the constants folded) or `(p · 320) · 0.5`: each is still one rounding of the same exact value 160 p, since the halving, wherever it falls, is exact. No contraction applies: there is no addition.
3. `max(…, 1.0)`: exact.
4. `bitcast<u32>`, `>> 23u`, `- 127u`, `& 0x7fffffu`, `!= 0u`, `u32(…)`, `+`, `min(…, 7u)` (`:286`), then `0xfeu << n`, `& 0xffu`, `1u |` (`:287`): integer operations, exact. For x ≥ 1 the exponent field is ⌊log₂ x⌋ + 127, so `- 127u` cannot wrap, and n = ⌈log₂ x⌉ capped at 7; the shift is at most 7.

So x = max(160 p, 1) and n = min(⌈log₂ x⌉, 7); the mask clears the bits of classes 0 to n − 1. Class k's upper bound is U_k = 2^(k+1)/320 m (`instance_mask`'s class is ⌊log₂ max(320 w, 1)⌋, `src/render/acceleration_structures.rs:1001-1002`), and k ≥ ⌈log₂(160 p)⌉ ⇔ 2^k ≥ 160 p ⇔ U_k ≥ p: a class is included exactly when its upper bound is no narrower than the pixel, as the comment at `:282` says. Edge inputs: p = 0 gives x = 1, n = 0, mask 0xff; p = +∞ gives exponent field 255, n = min(128, 7) = 7, mask 0x01; a NaN p gives 1.0 or the NaN from `max` (its exponent field 255), so 0xff or 0x01; every result holds bit 0.

The cases against that, by hand (n, then the expected literal):

| p (m) | 160 p, exact in the f32 input | n | mask |
|---|---|---|---|
| 0.0005, 0.003125, 0.004 | 0.08, 0.5, 0.64 → x = 1 | 0 | 1111_1111 |
| 0.00625 | 1.0000000149 → 1.0 | 0 | 1111_1111 |
| 0.007 | 1.12 | 1 | 1111_1101 |
| 0.0125 | 2.0000000298 → 2.0 | 1 | 1111_1101 |
| 0.025 | 4.0000000596 → 4.0 | 2 | 1111_1001 |
| 0.03 | 4.8 | 3 | 1111_0001 |
| 0.05 | 8.000000119 → 8.0 | 3 | 1111_0001 |
| 0.0501 | 8.016 | 4 | 1110_0001 |
| 0.1 | 16.00000024 → 16.0 | 4 | 1110_0001 |
| 0.147 | 23.52 | 5 | 1100_0001 |
| 0.2 | 32.00000048 → 32.0 | 5 | 1100_0001 |
| 0.21 | 33.6 | 6 | 1000_0001 |
| 0.4 | 64.00000095 → 64.0 | 6 | 1000_0001 |
| 0.41 | 65.6 | 7 | 0000_0001 |
| 5.0 | 800 | 7 (10 capped) | 0000_0001 |

All seventeen literals match. The seven bound cases are 1.6 × 2^-k decimals whose f32 is 1.600000023841858 × 2^-k (1.6's f32 rounds up), 1.49e-8 = 0.25u above the decimal, so the exact product is 2^j (1 + 0.25u). The f32 neighbours are 2^j and 2^j (1 + 2u): to nearest it rounds to 2^j (0.25u < u), toward zero to 2^j. A product rounded up, which no quoted specification allows, would give 2^j (1 + 2u), a nonzero mantissa, n one higher, and each of the seven masks one class bit short. The other ten cases lie at least 2.0e-3 relative from a power of two (8.016 against 8), about 3.4 × 10^4 u: no rounding reaches them. Floor 0 for the verdicts; the seven bound cases are exact by the rounding direction, not by margin.

##### Block C2: `segment_meets_box`

The function is unchanged since M3.5, and its test changed only by the comment at `:2204-2209`; M3.5's block ("Floor: `segment_meets_box`", `docs/audit/m3.5/correctness.md`) derives the chain at `common.wgsl:83-102`: about 10 roundings on each box-axis comparison and about 12u × |midpoint| on each cross-axis one, so a verdict flips only within about 24u × M of its boundary. At u = 2^-24: 24 × 5.96e-8 × 37.5 = 5.4e-5 (the comment's "below 6e-5") and 24 × 5.96e-8 × 1000 = 1.43e-3 (the comment's 1.4e-3). At u = 2^-23, which `msl-4.1-rounding` allows on Metal: 1.07e-4 and 2.86e-3, which the comment omits; the smallest non-exact margin, 1.0 box unit ("far box, ends short", every quantity dyadic), stays four orders above either. The cases are those M3.5 counted, 19; the scope added none.

##### Block C3: `position_at`

Unchanged since M3.5 (`common.wgsl:55-62`); M3.5's Oracles row and its block B derive ≈ 24u · w per component from the add-and-multiply chain and three divisions (2.5 ULP each on Vulkan per `wgsl-2026-float-accuracy`, "x / y: 2.5 ULP"), √3 of it over three components, ≈ 41u · w: 2.44e-6 · w at u = 2^-24 and 4.9e-6 · w at 2^-23. The constant is 2^-20 = 9.54e-7 times the largest coordinate, which on the view axis is 0.8/0.990 = 0.808 of w (the view direction is (0.3, −0.5, −0.8) normalised, `:1710-1711`) and larger toward the target's corners. 2.44e-6 / 9.54e-7 = 2.6 for a largest coordinate of w at u = 2^-24, and 4.9e-6 / (9.54e-7 × 0.808) = 6.4 on the view axis at u = 2^-23: the comment's "2.6 to 6.4 times this bound", which this reader takes as stated and did not re-derive case by case. The test's change in scope, `glam::camera::rh::view::look_to_mat4` for `glam::Mat4::look_to_rh`, builds the same right-handed view.

##### Block C4: the octahedral codec

Round trip: M3.5's block C, unchanged: ≈ 30u on the direction, 1.8e-6 and 3.6e-6, against 4e-6.

From 8 bits (`:1815-1816`): a coordinate c is stored as round(255 (c + 1)/2)/255 · 2 − 1, off by at most 1/255 = 3.922e-3, the error reached where 255 (c + 1)/2 lies on a half code. Near a face's centre, where the pre-normalised vector v = (x, y, 1 − |x| − |y|) is shortest (|v| ≥ 1/√3 = 0.57735), both coordinates off by +1/255 move v by e = (1, 1, −2)/255, |e| = √6/255 = 9.606e-3, and e is nearly perpendicular to v there (e · v ∝ x + y − 2z ≈ 0), so the angle is atan(|e| / |v|) = atan(√18/255) = atan(0.016638) = 0.016636 rad = 0.9532°. Half codes lie at c = (2k + 1)/255 − 1; c = 84/255 = 0.32941 is one (255 × (c + 1)/2 = 169.5) beside the face's centre 1/3, and v = (84, 84, 87)/255. Decoded from the stored (85, 85)/255 it is 0.95319° off, and from (83, 83)/255 0.95266° off, both computed in f64; a brute force over every pair of half codes (k, j from 100 to 254) gives the same maximum, 0.953188°. The constant is 0.95° = 0.016581 rad. M3.5's block C reached "≤ 0.0166 rad = 0.951°" by rounding 0.016638 rad to 0.0166 before converting (0.0166 rad = 0.9511°); the scope's comment at `:1831-1833` carries that 0.951°.

### Findings

| id | claim | severity | where | evidence | expected | verifier | placement |
|---|---|---|---|---|---|---|---|
| L1-S01 | ADR 0027 says a small Instance's coverage "is the area they truly cover over a pixel's" and that it is scaled "until they cover a pixel's", and `vs_thin`'s comment and the oracle's doc comment say the same, but `small_instance_coverage` returns the larger of the bounds' area share and (the sphere's diameter in pixels / 4)², so a sliver keeps samples, and escapes the drop under an eighth, by its length: the oracle's own case of half extent (1, 0.01, 0.01) has an area share of 0.04 and is kept at coverage 0.25005, one sample, scaled to 0.16 of a pixel. | `drift`; load-bearing: a reader predicting which Instances are dropped or how far one is scaled is misled for every Instance whose sphere spans 1.41 px or more while its area share is under 1/8; the mean coverage is unaffected | `docs/adr/0027-coverage-in-the-main-pass-is-a-written-sample-mask.md`, Decision, "A small Instance's coverage"; `main.wgsl:108-110`; `src/render/shader_function_tests.rs:2368-2370`; code at `common.wgsl:267-274` | block below | The ADR, the comment and the doc comment say the coverage is the larger of the bounds' area share and the square of the sphere's width over four pixels, so that the scale 1/√coverage reaches 1 at four pixels across and a sliver's samples follow its length; or the code keeps the area share and caps the scale at four pixels across the sphere | `confirmed`: `small_instance_coverage` returns `max(area_share, sphere_share)` (`src/render/shaders/common.wgsl:273`), and the oracle's own case `(AHEAD, [1.0, 0.01, 0.01], 0.250_05, 1.999_8, 0b0001)` (`src/render/shader_function_tests.rs:2444`) has area share 0.04 (a 2 × 0.02 m face under a 1 m pixel) yet keeps a sample, where ADR 0027 says "its coverage is the area they truly cover over a pixel's" and the drop is "under an eighth" (`docs/adr/0027-...md:12`); `drift`, as the mean coverage holds. | fix #1026 |
| L1-S02 | Off the view axis, scaling a small Instance across the view by 1/√coverage does not make its bounds cover a pixel, so its mean coverage falls below the area it truly covers: the parts of the footprint that the bounds' depth extent draws through perspective keep their width, so at the 2560×1440 view's corner a cube's bounds at coverage 0.5 draw 0.84 of a pixel and at 0.125 draw 0.65, and keep 16% and 35% less coverage than their true area, against ADR 0027's "scaled ... until they cover a pixel's" and its Consequence that a small Instance "keeps its mean coverage whatever its Mesh". | `correctness`; mean coverage low by 1 − (f + (1 − f)√c), f the front faces' share of the footprint: 0 on the view axis, 16% at c = 0.5 and 35% at c = 0.125 at the corner of the 45° 16:9 view, 15% at c = 0.5 for the oracle's own off-axis case; suspected visible as Instances at the screen's edges fading faster than at its centre (`town`'s far flocks) | `main.wgsl:154-160`; `common.wgsl:253-256`, `:278-280`; `docs/adr/0027-coverage-in-the-main-pass-is-a-written-sample-mask.md`, Decision, "A small Instance's coverage", and Consequences, fifth bullet | block below | Either the scale solves f s² + (1 − f) s = 1/c from the same face terms `small_instance_coverage` already sums, so the scaled bounds cover a pixel at any angle, or the ADR states that the scaling restores a pixel on the view axis only and records the loss off it | `confirmed`: The scale is applied across the camera's view axis only, `from_centre - along_view` times `small_instance_scale(instance_coverage)` (`src/render/shaders/main.wgsl:155-159`), so the faces the footprint shows through perspective (the `abs(dot(centre, cross(...)))` terms, `common.wgsl:253-256`) grow by s, not s²; rechecked by the projection: the oracle's ASIDE case (half extent 0.25 at (1000, 0, −1000), c = 0.5) draws 0.5 + 0.5 × 0.707 = 0.854 of a pixel, against ADR 0027's "until they cover a pixel's" and "keeps its mean coverage whatever its Mesh" (`docs/adr/0027-...md:12`, `:28`). | fix #1026 |
| L1-S03 | Three oracle tests added since M3.5 for the coverage functions state no WGSL accuracy floor beside their tolerance or exact comparison: `thin_drawn_radius`'s 1e-6, `small_instance_coverage`'s 2e-6 and `small_instance_scale`'s 1e-5 are unnamed literals with no comment, and the `thin_kept_samples` test says nothing of why an exact comparison holds at its thresholds, where the two other oracles added beside them do (`:1417-1420`, `:2588`). | `drift`; not load-bearing: each constant sits over its derived floor (Oracles: 40×, 1.8 to 3×, 3 to 5×, exact); the lane brief's requirement, three tests, four comparisons | `src/render/shader_function_tests.rs:2327`, `:2360`, `:2427`, `:2430` | block below | A comment beside each constant naming its floor and its chain, as Blocks A2 and A3 give them: the halving exact and one 2.5 ULP division (≤ 2.5e-8 at 0.08); exact sums at the four thresholds; about 14u on the coverage (≤ 1.1e-6 with the literals' rounding at 2⁻²³) and 2 ULP of `inverseSqrt` plus half that on the scale | `confirmed`: `(result[0] - expected).abs() < 1e-6` (`src/render/shader_function_tests.rs:2327`), `assert_eq!` for `thin_kept_samples` (`:2360`) and `2e-6`, `1e-5` (`:2427`, `:2430`) carry no floor comment, while `docs/agents/audit.md:73` asks for "the WGSL accuracy floor for its expression chain written beside the test's constant"; the three tests landed with Thin Primitives and small Instances, after `8bc34eb1`; `drift`, one fix with `L1-S08` and `L1-S13`. | fix #1026 |
| L1-S04 | `instance_lod` takes the sphere about an Instance's Mesh bounds with the half-diagonal times the Instance's largest column length as its radius, and scales the LOD error by that length, which bounds the Instance and its error only while its rotation-scale columns are orthogonal; for a sheared Instance a corner lies up to √3 times the radius from the centre, so the distance, and with it the LOD, can be coarser than the threshold allows, while the comment says no nearer point strays further than the threshold allows. | `correctness`; ADR 0019 line 15's "distance to the Instance's bounding sphere" broken for a sheared Instance: a corner 21% past the sphere for two unit axes 45° apart, at most √3 times the radius; none on an Instance whose axes are orthogonal, which `WorldTransform::drawn` says every hierarchy here composes | `culling.wgsl:269-292`; `common.wgsl:216` | block below | the comment and ADR 0019 name the limit as `clip_space_bounds` does ("only while the axes are orthogonal"), or the radius and the error's scale use a bound that holds under shear, such as the sum of the three half axes' lengths, as `small_instance_coverage` bounds its farthest corner | `confirmed`: `instance_lod` takes `radius = 0.5 * length(bounds_max - bounds_min) * scale` with `scale` the largest column length (`src/render/shaders/culling.wgsl:269-292`), which bounds the box only for orthogonal columns: recomputed, columns (1,0,0), (0.7071,0.7071,0), (0,0,1) put the unit cube's corner at length 2.101 against a radius of 1.732; ADR 0019 asks "the distance to the Instance's bounding sphere" (`docs/adr/0019-...md:15`), and `clip_space_bounds` states this very limit ("only while the axes are orthogonal", `culling.wgsl:145-146`) where `instance_lod`'s comment claims "no nearer point of it ... strays further"; no Reference scene shears an Instance. | fog (#17): revive when an Instance is sheared, in the line on a sheared Thin Primitive's culling bounds |
| L1-S05 | ADR 0003 and ADR 0017 give an Instance slot as 136.2 bytes with its offsets and visibility bit, and ADR 0003 517,376 slots at the checked-in Budgets, but a slot's offsets are four u32 since the culling pass's scan gained its third component and the LOD error word: 144.2 bytes a slot and 488,448 slots, as `RenderCapacities` computes and its test pins. | `drift`; not load-bearing (the code derives the capacity itself); a reader of either ADR overstates the slots by 28,928, 5.9%, and the bytes a slot by 8; a research note repeats "136.2 bytes a slot today" | `docs/adr/0003-persistent-scene-transform-buffer.md:3`; `docs/adr/0017-textures-stream-by-mip-geometry-lives-in-fixed-arenas.md:18`; `docs/research/rain-snow-wetness-and-wind.md:131`; made false by `culling.wgsl:453-456` and `src/render/bind_groups.rs:131` | block below | the ADRs say 144.2 bytes a slot and 488,448 slots, or the arithmetic without the figure | `killed`: duplicate of L1-R03: the same two ADR sentences (`docs/adr/0003-...md:3`, `docs/adr/0017-...md:18`) and the same 136.2 against 144.2 bytes a slot; `L1-R03` keeps it, narrowed to take this Finding's research-note location and its account of the offsets' widening. | wontfix: killed by the verifier, its reason under Killed |
| L1-S06 | The WGSL floors stated beside `PRECISION_BAR` count the composition's own roundings only, but the precision oracles compare the GPU's result with the f64 reference from offsets rounded to f32, up to 2⁻¹⁹ m each per axis; added to the comment's figures the worst other order is 4.63e-5 m, over the 4.3e-5 bar the comment presents its 3.97e-5 as under. | `drift`; not load-bearing: the test's verdict holds, as the cases' actual magnitudes bound it at 3.31e-5 m (Block B2); misleads a reader judging the any-order margin, which is 1.30×, not the 1.08× the comment implies, and under the comment's own model negative | `src/render/shader_function_tests.rs:1864-1871`; inputs at `:2237-2251` and `src/render/instance.rs:8-12`; comparisons at `:1943`, `:2005` | block below | the comment states the floor of what the assertion measures: the chain plus the two offsets' rounding (64u per axis), and the bound that keeps the any-order case under the bar | `confirmed`: The reference is the exact f64 `camera_relative` (`src/render/shader_function_tests.rs:2225-2229`, compared at `:1943`) while the inputs are `offset.as_vec3()` (`src/render/instance.rs:11`), up to 2⁻¹⁹ m off each in [32, 64), so the comment's floors at `:1864-1871` ("384.9u in the worst other order ... 3.97e-5 m") leave out 64u per axis: 448.9u × √3 × 2⁻²⁴ = 4.63e-5 m, over `PRECISION_BAR` 4.3e-5; the comment landed after `8bc34eb1`; `drift`, the verdict holds on the cases' magnitudes. | fix #1034 |
| L1-S07 | The test that `mod.rs` rests its `unsafe` call on passes a counted loop whose body writes the loop's bound, so a loop that never ends, which the registry's quote says a backend may take for unreachable code, can join `MAIN` or `CULLING` with every test green; no loop in either module does so today. | `correctness`; the guarantee behind `create_shader_module_trusted` with loop bounding off is incomplete; no defect in the code at this commit (every bound is a `let`, a parameter, a constant, a uniform field or a `var` its body does not write) | `src/render/shader_tests.rs:682-746`, `:754-765`; `mod.rs:191-193`, `:242-246` | block below | `loop_problems` also refuses a counted loop whose bound names a variable the body writes, or requires bounds that are not `var`s, with a case in `a_loop_whose_text_does_not_end_it_fails` | `killed`: No loop that passes the check can run forever: `for_counter` admits only `name < bound` stepped by `name++` (`src/render/shader_tests.rs:757-764`) with a body that never writes `name` (`:735-743`), and WGSL's `++` applies to integers alone, so the counter steps only after `name < bound` held, never wraps, and the loop ends within 2³² iterations whatever the body writes to the bound; the quote `wgpu-types-30-loop-bounding` speaks of "infinite loops", which this is not. | wontfix: killed by the verifier, its reason under Killed |
| L1-S08 | The LOD oracle, added since M3.5, asserts its LODs exactly and states no WGSL floor beside the assertion, as M5.5's exit asks of every oracle test added since M3.5; its boundary cases pass only because their products are exact. | `drift`; not load-bearing (the floor is 0 and the test is right, Block B1); one oracle without the stated floor the exit requires | `src/render/shader_function_tests.rs:2503-2569`, assertion at `:2560` | block below | a line beside the assertion: floor 0, two correctly rounded products and an exact comparison; the four boundary cases multiply dyadic values, exact, so equality holds bit for bit, fused or not; every other case lies at least 13% from its boundary | `confirmed`: The LOD oracle's `assert_eq!(result[0], expected, ...)` (`src/render/shader_function_tests.rs:2560-2561`) has no floor beside it, and its boundary cases (`(8.0, whole_metres, 1.0, 2.0, 0.5, 0.0)` and kin, `:2531-2534`) hold only because 1 × 1 against 0.5 × 2 is exact; the test landed after `8bc34eb1`; `drift`, one fix with `L1-S03`. | fix #1034 |
| L1-S09 | The octahedral oracle's comment gives the 8-bit store's worst case as 0.951° and the test takes 0.95° as its bound, but the worst case is atan(√18/255) = 0.9532°, which the direction normalize(84, 84, 87) reaches (0.9532° or 0.9527°, whichever way its half code rounds), so the constant sits under the bound it claims to carry and the test passes only because its 4,110 directions miss such a one. | `drift`; load-bearing for the oracle (a valid input fails it by 0.0032°), not for the engine: the margins it backs, sin 2° (`shadow_mask.wgsl:64-77`), cover twice the true bound, 1.906° | `src/render/shader_function_tests.rs:1831-1834` | block below | the comment states 0.9532° (atan(√18/255)), and the constant is at or above it, as is the "within 0.95°" the margins' comment repeats at `shadow_mask.wgsl:67-68` | `confirmed`: The comment says the bound "is 0.951°" and the test takes `0.95f64.to_radians()` (`src/render/shader_function_tests.rs:1831-1834`), but recomputed, normalize(84, 84, 87) encodes to a half code (169.5) and decodes to (1, 1, 1) at sin⁻¹√(18/65043) = 0.9532° or to (83, 83, 89) at 0.9526°, both over 0.95°; the "is 0.951°" comment landed in scope (the M3.5 exit's oracle-floor pass); `drift`. | fix #1034 |
| L1-S10 | `shadowed` passes the back face of whichever Instance a ray leaving its surface meets first, the receiver's own or another's, when it lies no higher above the surface than `LOD_ERROR_MARGIN` times that Instance's drawn LOD error, so an open (single-sided) Mesh drawn at a coarser LOD stops shadowing another Instance's surface whose ray meets its back within that height: the leak `tabellion-lamorlette-2004-ray-origin-offset` warns that ignoring near intersections creates, which the rule's purpose, a drawn LOD lying inside its own LOD 0, does not need. | `correctness`; the source's warning broken in a band up to 3 drawn LOD errors high, under 3 pixels' width at the Instance's distance by the LOD threshold; no capture shows it, and `world`'s frames 59, 300 and 599 change by no pixel from no rule (`docs/performance.md:34`) | `src/render/shaders/shadow_mask.wgsl:193-195` | block below | a back face is passed only where it can be the receiver's own drawn LOD's LOD 0, or the leak under another Instance's open Mesh is accepted and recorded as a Deviation of `tabellion-lamorlette-2004-ray-origin-offset` | `confirmed`: `shadows = any_hit \|\| hit.front_face \|\| (leaves_surface && height > LOD_ERROR_MARGIN * drawn_lod_error)` (`src/render/shaders/shadow_mask.wgsl:193-195`) reads the hit Instance's own LOD error and never compares that Instance with the receiver, so another Instance's single-sided back face within 3 of its drawn LOD errors is passed, where `tabellion-lamorlette-2004-ray-origin-offset` says biasing that ignores "intersections near the ray origin ... would create significant light or shadow leak problems" and sizes the tolerance by "the coarse and micro-polygon geometric tessellations" of the surface the ray leaves; no capture, so `correctness`. | wontfix: a Deviation the owner accepted, its row in `docs/audit/deviations.md` |
| L1-S11 | `first_intersection`'s doc comment says it returns "The first triangle either TLAS holds" along the segment, naming every argument but `mask`, which the scope added; it returns the first triangle of an entry whose instance mask shares a bit with `mask`, so a Thin Primitive of a width class the mask leaves out is passed though a TLAS holds it. | `drift`; not load-bearing: `shadowed`'s comment (`:149-150`) says the ray passes such a Thin Primitive; a reader of `first_intersection` alone misses the filter | `src/render/shaders/shadow_mask.wgsl:83-87` (signature `:99`) | block below | the comment names `mask` and the entries it admits | `confirmed`: The doc says "The first triangle either TLAS holds from `start_distance` to `end_distance`" (`src/render/shaders/shadow_mask.wgsl:83-87`) above `fn first_intersection(flags: u32, mask: u32, ...)` (`:99`), whose `mask` both `RayDesc`s pass as the cull mask (`:102`, `:118`); `drift`. | fix #1034 |
| L1-S12 | `CONTEXT.md` defines a Thin Primitive as one "whose shadow ends where it would fall between a pixel and half a pixel wide", but the width classes give that end only to a Mesh of Thin Primitives alone whose widest tube is 3.125 mm to under 400 mm wide: a narrower tube takes the narrowest class and keeps its shadow until a 6.25 mm pixel, where a 1 mm tube's shadow is 0.16 of a pixel wide, and a tube of 400 mm or more, or any tube of a Mesh with an ordinary Primitive, takes the ordinary bit and casts onto every pixel. | `drift`; not load-bearing: no Reference scene's tube falls outside the range that this reader found (`town`'s wire is 30 mm); a reader of the glossary expects every Thin Primitive's shadow to end | `CONTEXT.md:22` | block below | the glossary's sentence states the range the classes cover, 3.125 mm to 400 mm, in a Mesh of Thin Primitives alone, as the shaders do | `confirmed`: `CONTEXT.md:22` says a Thin Primitive's shadow "ends where it would fall between a pixel and half a pixel wide", while `instance_mask` gives a mixed Mesh `ORDINARY_MASK` (`src/assets/pools.rs:862-863`: a radius only "where every Primitive ... is a Thin Primitive"), puts every width under 3.125 mm in class 0 by `.max(1.0)` and every width of 400 mm or more in the ordinary bit (`src/render/acceleration_structures.rs:1001-1007`); `drift`. | fix #1034 |
| L1-S13 | The width-class oracle, added in scope, states no WGSL floor, though seven of its 17 cases (0.00625 to 0.4 m) sit on a class bound, where the expected mask holds only because each f32 literal lies 0.25u above the bound and every quoted backend rounds the product to nearest or toward zero; the neighbouring oracles state theirs beside their assertions. | `drift`; not load-bearing: the test is right on every quoted backend; the M5.5 exit asks each oracle added since M3.5 for its floor, and a backend that rounds a product up would fail the seven cases with no comment saying why | `src/render/shader_function_tests.rs:2463-2500` | block C1 (Oracles) and below | a floor comment beside the assertion: one rounded product, floor 0, the seven bound cases exact under round to nearest and round toward zero | `confirmed`: The width-class oracle's only comment is "Each case's mask is a literal" (`src/render/shader_function_tests.rs:2463-2464`), and seven cases (0.00625 to 0.4) are 0.1 × 2ᵏ, whose f32 literals sit 0.25u above a bound that `0.5 * pixel_metres * 320.0` (`common.wgsl:284`) reaches only under round-to-nearest or toward zero; landed after `8bc34eb1`; `drift`, one fix with `L1-S03`. | fix #1034 |

#### L1-S01

`common.wgsl:267-274`:

```wgsl
    let sphere_diameter_squared = 4.0 * (dot(half_axes[0], half_axes[0])
        + dot(half_axes[1], half_axes[1])
        + dot(half_axes[2], half_axes[2])
        + 2.0 * farthest_skew);
    let sphere_share = sphere_diameter_squared
        / (SMALL_INSTANCE_WIDEST_PIXELS * SMALL_INSTANCE_WIDEST_PIXELS * pixel_area);
    let coverage = max(area_share, sphere_share);
    return select(1.0, coverage, view_depth > 0.0 && coverage < 1.0);
```

ADR 0027, Decision, "A small Instance's coverage": "An Instance whose Mesh
bounds cover under a pixel's area is scaled about their centre, across the
view, until they cover a pixel's, each vertex keeping its depth, and its
coverage is the area they truly cover over a pixel's. With culling on, the
culling pass drops it once that rounds to no sample, under an eighth. An
Instance that draws four pixels wide across the sphere about its bounds is
scaled no further". `main.wgsl:108-110`: "any other vertex, of a small
Instance, moved out from the centre of its Mesh bounds across the view
until they cover a pixel's area". `shader_function_tests.rs:2368-2370`: "An
Instance whose bounds cover under a pixel's area keeps that share of its
samples and is scaled until they cover a pixel's, and keeps none under an
eighth". M5 story 39 asks for "its coverage by area".

The sphere term is (d / 4p)², d the sphere's diameter and p a pixel, so it
reaches 1/8, the drop, at d = 4p/√8 = 1.41 p, and 1 at d = 4p, where it
stops the scaling as the ADR's last sentence asks. Between, the coverage
is the sphere term wherever it exceeds the area share, which a long thin
box does: the oracle's case at `shader_function_tests.rs:2444`, half
extent (1, 0.01, 0.01) at 1,000 m under a pixel of 1 m, has an area share
of 4 · 0.01 · 1000 / 1000 = 0.04, under the eighth at which the ADR drops
it, and the test asserts coverage 0.250_05, scale 1.999_8 and one sample
kept. Its scaled bounds cover 0.04 · 1.9998² = 0.16 of a pixel, not one.
The mean coverage is right either way (0.16 × 1/4 = 0.04), which is why
this is drift: the code's rule is a coherent one, the documents name
another. The sheared case (`:2458-2460`) is the same: area share 0.002,
coverage 0.5625, two samples.

#### L1-S02

`main.wgsl:154-160`:

```wgsl
        // Across the view only: each vertex keeps its depth, as the culling pass's box does.
        let view_axis = vec3<f32>(camera.view_proj[0].w, camera.view_proj[1].w, camera.view_proj[2].w);
        let from_centre = out.camera_relative_position - bounds_centre;
        let along_view = view_axis * (dot(from_centre, view_axis) / max(dot(view_axis, view_axis), 1e-30));
        out.camera_relative_position = bounds_centre + along_view
            + (from_centre - along_view) * small_instance_scale(instance_coverage);
```

`common.wgsl:253-256` measures the footprint as the sum over the box's
three face pairs of 4 \|centre · (aᵢ × aⱼ)\| / w³, in pixels after
dividing by the pixel's area. For a box of half extent h at (X, Y, −W)
in camera space, axis-aligned, the three terms are 4h²W (the face across
the view axis), 4h²\|X\| and 4h²\|Y\| (the faces along it, which the
footprint shows through perspective). Scaling every vertex's offset across
the view by s about the centre and keeping its component along the view
axis multiplies the first face's two edges by s and each of the other two
faces' one cross-view edge by s, its edge along the view axis unchanged:
the footprint becomes 4h²(s²W + s\|X\| + s\|Y\|) / W³. With f =
W / (W + \|X\| + \|Y\|) and s² = 1/c, the scaled footprint is c (f s² +
(1 − f) s) = f + (1 − f)√c pixels, one pixel only where f = 1, on the
view axis. The mask keeps c of the samples, so the mean coverage is c (f +
(1 − f)√c), against c.

Numbers: at the corner of the 2560×1440 view with a 45° vertical field
of view, X/W = tan 22.5° × 16/9 = 0.736 and Y/W = 0.414, so f = 1 / 2.150
= 0.465. At c = 0.5: 0.465 + 0.535 × 0.707 = 0.843, 16% low. At c =
0.125: 0.465 + 0.535 × 0.354 = 0.654, 35% low. The oracle's own off-axis
case, ASIDE at (1000, 0, −1000) with half extent 0.25 (`:2443`): f = 0.5,
c = 0.5, scale √2, footprint 0.5 + 0.5 × 0.707 = 0.854, 15% low; checked
against the exact projection: the face across the view becomes 0.354 m
square, 0.5 px at 1,000 m under a pixel of 1 m, and the side face keeps
its 0.5 px of perspective width at 0.707 px of height, 0.354 px, sum
0.854. The test asserts the coverage and the scale, not the drawn
footprint, so it passes.

The clauses: ADR 0027, Decision: "scaled about their centre, across the
view, until they cover a pixel's, each vertex keeping its depth";
Consequences, fifth bullet: "A small Instance keeps its mean coverage
whatever its Mesh, since the samples it keeps fall by the area it gains."
The area it gains is 1/c on the axis only. For the verifier: no capture
can isolate this at native resolution; the arithmetic above can be
checked by drawing one small box through the small-Instance twin at the
view's centre and at its corner at equal coverage and counting the
samples its depth writes (a sample-count readback, as `coverage_tests.rs`
does for IDs), or by the projection above. A visible effect, if any, is
`town`'s far flocks thinning toward the screen's edges in a pan.

#### L1-S03

`shader_function_tests.rs:2325-2328`:

```rust
    for ([radius, pixel, expected], result) in cases.into_iter().zip(results) {
        assert!(
            (result[0] - expected).abs() < 1e-6,
```

`:2424-2430`:

```rust
            assert_close(
                result[0],
                coverage,
                2e-6,
                format_args!("the coverage of {case}"),
            );
            assert_close(result[1], scale, 1e-5, format_args!("the scale of {case}"));
```

and the `thin_kept_samples` test's `assert_eq!` at `:2360` with no
comment on its thresholds. The lane brief, `docs/agents/audit.md:73`:
"every function under an oracle that is in the audit's scope against its
literature number, with the WGSL accuracy floor for its expression chain
written beside the test's constant". The two other tests added in scope
that this reader owns state theirs: `:1417-1420` ("WGSL floor 0: binary16
widens exactly [...] but z is sqrt(1), exact only where sqrt rounds
correctly") and `:2588` ("Layers stay below 2^24, so f32 carries them
exactly"). The floors themselves are in Oracles, Blocks A2 and A3.

#### L1-S04

`culling.wgsl:269-292`:

```wgsl
// An Instance's largest scale along an axis.
fn instance_scale(instance: InstanceData) -> f32 {
    return max(
        max(length(instance.rotation_scale[0]), length(instance.rotation_scale[1])),
        length(instance.rotation_scale[2])
    );
}

// The LOD an Instance draws of its Mesh: the one forced, or the one selected
// at the distance to the sphere about its Mesh bounds, under the Instance's
// largest scale along an axis, so no nearer point of it and no axis strays
// further than the LOD threshold allows: [...]
    let scale = instance_scale(instance);
    let radius = 0.5 * length(bounds_max - bounds_min) * scale;
    let distance_metres = max(length(bounds_centre(instance, bounds_min, bounds_max)) - radius, 0.0);
```

ADR 0019 line 15: "The pass draws, of each Instance, the coarsest LOD whose
LOD error times the Instance's scale is under the LOD threshold, one pixel
of the target's height, at the distance to the Instance's bounding sphere."
`meshoptimizer-1.3-lod-selection`: "float d = max(0, distance(camera_position,
mesh_center) - mesh_radius)".

A box corner at half-extent v from the centre lands at `rotation_scale * v`,
whose length is at most Σ|vᵢ| ‖cᵢ‖ ≤ √3 ‖v‖ maxᵢ ‖cᵢ‖, and equals at most
‖v‖ maxᵢ ‖cᵢ‖ only when the columns cᵢ are orthogonal. Example: columns
(1, 0, 0), (0.7071, 0.7071, 0), (0, 0, 1), each of length 1, over the unit
cube's bounds: radius 0.5 × ‖(2, 2, 2)‖ × 1 = 1.732; the corner (1, 1, 1)
lands at (1.7071, 0.7071, 1), of length √4.4142 = 2.101, 0.369 past the
sphere, 21% of its radius. A camera 0.369 m from that corner reads a
distance of up to 0.369 m where the Instance's nearest point is at 0; an
error along that corner's direction is scaled by the matrix's largest
singular value, √1.7071 = 1.307, not 1. Shear exists in the World transform:
`src/transform/mod.rs:21-23` ("a shear, a non-uniformly scaled parent's
turned child, has no such reading"), and two other culling functions guard
it: `small_instance_coverage` ("The farthest corner's, which an Instance
that shears moves past the axes' summed squares", `common.wgsl:257`) and
`clip_space_bounds` ("holds every vertex only while the axes are
orthogonal", `culling.wgsl:145-146`). `instance_lod` states neither the
limit nor a bound that holds past it. The verifier checks whether any
Reference scene places a non-uniformly scaled parent with a turned child;
none known, so no capture is named.

#### L1-S05

`docs/adr/0003-persistent-scene-transform-buffer.md:3`: "30% of it at
136.2 bytes a slot, its Instance, offsets and visibility bit, in whole
256-slot scan blocks of the culling pass: 517,376 slots at the Budgets M4
set". `docs/adr/0017-[...].md:18`: "the Instance slots 30% of the scene and
pass buffers', at 136.2 bytes a slot with its offsets and visibility bit, in
whole scan blocks". `docs/research/rain-snow-wetness-and-wind.md:131`: "(ADR
0003, 136.2 bytes a slot today)".

`culling.wgsl:453-456` writes four words a slot:

```wgsl
        instance_offsets[slot] = vec4<u32>(
            select(NOT_DRAWN, vec3<u32>(record_small_and_lod, offset.yz), drawn),
            bitcast<u32>(lod_error_metres)
        );
```

and `src/render/renderer.rs:213-218` sizes the slots by them:

```rust
const INSTANCE_SCAN_BLOCK_BYTES: u64 = SCAN_BLOCK_SIZE as u64
    * (size_of::<InstanceData>() + size_of::<[u32; 4]>()) as u64
    + size_of::<[u32; 4]>() as u64
    + SCAN_BLOCK_SIZE as u64 / u8::BITS as u64;
```

`InstanceData` is 128 bytes (two `mat3x3<f32>` of 48, a `vec3<i32>` and a
`u32`, a `vec3<f32>` and a `u32`; `common.wgsl:177-184`, pinned by the twin
test): 256 × 144 + 16 + 32 = 36,912 bytes a block, 144.19 a slot. The
checked-in scene and pass buffers' Budget, 234,881,024 bytes
(`bench/budgets.json:17`), gives 234,881,024 × 300 / 1000 = 70,464,307, less
the phase's two entries' 32 = 70,464,275; / 36,912 = 1,908.98, so 1,908
blocks, 488,448 slots, as `src/render/renderer.rs:1092` pins. With two-word
offsets a block is 256 × 136 + 8 + 32 = 34,856 bytes, 136.16 a slot, and
70,464,275 / 34,856 = 2,021.6, so 2,021 blocks, 517,376 slots: the ADRs'
figures, which date from before the offsets widened.

#### L1-S06

`src/render/shader_function_tests.rs:1864-1871`:

```rust
/// [...] WGSL floors:
/// `camera_relative_position`, 192.6u per axis in its written order, 2.0e-5 m
/// (4.0e-5 under round toward zero), and 384.9u in the worst other order,
/// (offset + local position) − camera offset and the anchor term last,
/// 3.97e-5 m (7.9e-5, over this, if a GPU both reorders and rounds toward
/// zero); `point_light_position`, with no local position, 64.3u per axis,
/// 6.6e-6 m (1.3e-5), and 64.6u in the worst other order, 6.7e-6 m (1.3e-5).
const PRECISION_BAR: f64 = 4.3e-5;
```

The inputs: `Case::test_vectors` (`:2237-2239`) calls `anchor_and_offset`,
which returns `offset.as_vec3()` (`src/render/instance.rs:11`), the f64
remainder in [0, 64] rounded to f32; the reference is the exact f64
`camera_relative`, and the assertions at `:1943` and `:2005` measure
`gpu.distance(case.camera_relative)`. Each offset's rounding is up to 2⁻¹⁹ m
= 1.91e-6 m in [32, 64), 32u with u = 2⁻²⁴ m-units; two per axis, 64u. With
them: written order 256.6u, √3 × 256.6 × 2⁻²⁴ = 2.65e-5 m; worst other order
448.9u, √3 × 448.9 × 2⁻²⁴ = 4.63e-5 m, over 4.3e-5; point light 128.3u,
1.33e-5 m. The CPU's any-order check (`:1948-1952`) and the measured
0.0185 mm (ADR 0014 line 19) are under the bar because the cases' partial
sums are smaller than the u·|x| model takes (Block B2: 3.31e-5 m), which the
comment does not say. The numbers it does state are right for the chain
alone (192.6u × √3 × 2⁻²⁴ = 1.99e-5; 384.9u → 3.97e-5; 64.3u → 6.64e-6;
64.6u → 6.67e-6).

#### L1-S07

`mod.rs:242-246`:

```rust
        // SAFETY: without loop bounding a backend compiler may take an
        // infinite loop for unreachable code [ref:wgpu-types-30-loop-bounding],
        // so the module holds none: each loop of its parts ends whatever its
        // buffers hold, which `shader_tests` checks.
        unsafe { device.create_shader_module_trusted(self.descriptor(capacities), runtime_checks) }
```

`wgpu-types-30-loop-bounding`: "If false, the caller MUST ensure that all
passed shaders do not contain any infinite loops. If it does, backend
compilers MAY treat such a loop as unreachable code".

`src/render/shader_tests.rs:682-686` documents the check: a loop must be
pinned or "counted, by a counter its header declares, compares with `<`
alone and steps by `++`, which reaches any bound; a `for` loop whose body
writes its counter [...]" is refused. `for_counter` (`:754-765`) accepts
`name < <anything>` without writes of `name`, and the body scan (`:735-743`)
looks for writes of the counter alone:

```rust
                if writes(body_line, name) {
                    problems.push(format!("{location}: its body writes `{name}`"));
```

So `for (var entry = 0u; entry < point_light_count; entry++) {
point_light_count += 1u; }` passes: `++` reaches a fixed bound, not one the
body raises each step. `main.wgsl:437`'s bound is a `var`
(`var point_light_count = light.point_light_count;`, `:429`), written only
before the loop (`:435`); every other bound in the two modules is a `let`
(`culling.wgsl:478`, `:489`, `:597`), a field of a `let` copy (`:335`), a
parameter (`common.wgsl:215`), a uniform field (`main.wgsl:407`) or a
constant (`culling.wgsl:94`, `:231`, `:647`). `a_loop_whose_text_does_not_end_it_fails`
(`:174-229`) holds no case of a bound the body writes.

#### L1-S08

`src/render/shader_function_tests.rs:2503-2509` (the doc) and `:2557-2568`:

```rust
    for ((lod_count, errors, scale, distance, threshold, expected), result) in
        cases.into_iter().zip(results)
    {
        assert_eq!(
            result[0], expected,
```

No comment beside it states a floor, where the precision oracles
(`:1864-1871`), `pack_index`'s (`:1677`, "WGSL floor 0: integer shifts and
masks, on halves f32 holds exactly"), `position_at`'s and the octahedral
codec's each gained one in scope. Block B1 gives the floor and the margins.

#### L1-S09

Test, `src/render/shader_function_tests.rs:1831-1834`:

```rust
// The 8-bit rounding's bound, half a step per coordinate over a vector no
// shorter than 1/√3, is 0.951°, which this takes to three figures; f32
// adds about 2e-6 rad.
let largest_8_bit_error = 0.95f64.to_radians();
```

and the store it bounds, `:1815-1816`: `(((coordinate * 0.5 + 0.5) * 255.0).round() / 255.0) * 2.0 - 1.0`. Half a step per coordinate is 1/255 in [−1, 1]. With both coordinates off by +1/255, the pre-normalised vector (x, y, 1 − |x| − |y|) moves by (1, 1, −2)/255, of length √6/255 = 9.606e-3; near a face's centre that move is nearly perpendicular to a vector of length 1/√3, so the angle is atan(√6/255 × √3) = atan(√18/255) = atan(0.016638) = 0.016636 rad = 0.9532°. The comment's 0.951° is 0.0166 rad converted (0.9511°), the same quotient rounded to three figures before conversion; the M3.5 report's Block C made the same step ("≤ 0.0166 rad = 0.951°, the constant") and gave the row `explained`. The direction normalize(84, 84, 87) = (0.57048, 0.57048, 0.59085) encodes to (84/255, 84/255), each 169.5 codes up the 8-bit range, a half code: stored as 170 it decodes to normalize(1, 1, 1), 0.95319° away; stored as 169, to normalize(83, 83, 89), 0.95266° away; both over 0.95°. Computed in f64; a brute force over every pair of half codes from 100 to 254 gives the maximum 0.953188°. The test's directions (6 axes, 8 edges, a 4,096-point Fibonacci lattice, `:1776-1800`) contain no such point, so it passes. The shadow mask's margins' comment, `shadow_mask.wgsl:66-69`, says the stored normals hold "each within 0.95° where the store rounds to the nearest step [...] and about twice that where it may round to either", which sin 2° (`:75-76`) covers at the true 0.9532° and twice it, 1.906°.

#### L1-S10

Code, `src/render/shaders/shadow_mask.wgsl:189-195`:

```wgsl
// The stored normal errs by as much as the margin, so the height is its upper bound.
let height = offset + hit.t * (facing + AGREEING_TRIANGLE_TURN_MARGIN);
// A TLAS entry's custom data is its Instance's slot, whose fourth word the culling pass wrote.
let drawn_lod_error = bitcast<f32>(instance_offsets[hit.instance_custom_data].w);
let shadows = any_hit
    || hit.front_face
    || (leaves_surface && height > LOD_ERROR_MARGIN * drawn_lod_error);
```

The hit's Instance is whatever the nearest-hit query found (`:182-186`); nothing compares it with the receiver's, which the pass does not know (it rebuilds the receiver from depth and normals, `:233-243`). The comment's case is the receiver's own: "every BLAS holds its Mesh's LOD 0 (ADR 0020), and the LOD drawn can lie inside it" (`:144-146`), and its closed-mesh premise is "A ray that enters a closed mesh meets a front face first, so whatever stands beyond still shadows the point" (`:141-143`). A single-sided Mesh seen from behind has no front face for the ray to meet first: its back face is the occluder. The source the comment cites, `tabellion-lamorlette-2004-ray-origin-offset`: "Traditional biasing techniques that ignore intersections near the ray origin cannot be applied here, since they would create significant light or shadow leak problems. [...] the offset distance used in this algorithm needs to be bigger than the maximum offset between the coarse and micro-polygon geometric tessellations": the tolerance it justifies is the offset between one surface's two tessellations, the receiver's. M5 story 13 states the same purpose, "so that a drawn LOD sitting just inside LOD 0 does not shadow itself". Open Meshes that shadow with their backs exist: the message of commit `481abfca` (2026-11-17), which added the rule, says "the shadows that open Meshes cast with their backs in `world` stay" at LOD 0, where the error is 0, and that "No source found bounds the tolerance by the drawn LOD's own error; that part is this change's". Magnitude: the culling pass draws a LOD whose error times the Instance's scale is under the LOD threshold times the distance to its bounds (`common.wgsl:216`; `culling.wgsl:284-299`), so 3 × the error is under 3 pixels' width at that distance; a receiver under an open Mesh's back within that height, such as a wall top under a single-sided eave, takes the light the eave should hide. For the verifier: `praxis shadow-mask-difference world <png> 2560x1440 --frame <n>` at frames where buildings with single-sided roofs or awnings stand within the Load radius at a LOD past 0, checking whether the differing pixels lie under an open Mesh's back.

#### L1-S11

`src/render/shaders/shadow_mask.wgsl:83-87`: "The first triangle either TLAS holds from `start_distance` to `end_distance` (the ray's tmin and tmax) along the ray from `origin`, in the TLAS's frame, along `direction`, a unit vector: with RAY_FLAG_TERMINATE_ON_FIRST_HIT in `flags`, any one, and otherwise the nearest." The signature at `:99` is `fn first_intersection(flags: u32, mask: u32, origin: vec3<f32>, direction: vec3<f32>, start_distance: f32, end_distance: f32)`, and both queries pass `mask` as the ray's cull mask (`:102`, `:118`). By `wgpu-30-tlas-instance-fields`, a query "Reports hit only if `(shader_cull_mask & tlas_instance.mask) != 0u`", and `shadowed` passes `shadow_ray_mask(pixel_width)` (`:181`), which leaves out the narrower width classes (`common.wgsl:282-287`). The scope's diff added the argument and changed no word of the comment.

#### L1-S12

`CONTEXT.md:22`: "A Primitive declared a tube of a given radius, drawn a pixel wide where it would be narrower, whose shadow ends where it would fall between a pixel and half a pixel wide." The entry's mask, `src/render/acceleration_structures.rs:990-1008`: with no thin radius (a Mesh with any ordinary Primitive) it returns `ORDINARY_MASK`, 1; otherwise `narrowest_bounds = (width_metres * 320.0).max(1.0)`, `class = (narrowest_bounds as u32).ilog2()`, bit `2 << class` while `class < 7`, else 1. A width under 1/320 m = 3.125 mm gives `narrowest_bounds` 1 and class 0, whose upper bound is 6.25 mm; a width of 128/320 = 0.4 m or more gives class 7 or more and the ordinary bit. The ray's mask always holds bit 0 (`common.wgsl:287`). So a 1 mm tube's shadow ends where the receiver's pixel passes 6.25 mm, 1/6.25 = 0.16 of a pixel wide, short of half a pixel, and a 0.5 m tube, or a tube in a lamp Mesh with an ordinary head, never ends. The classes' two ends and the mixed Mesh's ordinary bit are calls the implementing commit (`0c133356`, 2026-11-16) records ("A width under 3.125 mm takes the narrowest class, and one of 400 mm or more the ordinary bit"; "A Mesh that mixes thin and ordinary Primitives takes the ordinary bit"); the glossary entry did not take them.

#### L1-S13

`src/render/shader_function_tests.rs:2463-2464`: "Each case's mask is a literal: the classes' upper bounds are 6.25, 12.5, 25, 50, 100, 200 and 400 mm, bits 1 to 7." No other comment in the test (`:2465-2502`) speaks of rounding. Beside it, the scope's other oracle comments state theirs, e.g. `:1677` "WGSL floor 0: integer shifts and masks, on halves f32 holds exactly", `:1756-1758`, `:1848-1849`, `:2204-2209`. Block C1 under *Oracles* gives the floor: one product rounded once, so a case's verdict can move only within one rounding of a class bound; the seven bound cases are 0.25u above one, exact under the rounding to nearest that `wgsl-2026-float-accuracy` ("x * y: Correctly rounded") requires and the rounding toward zero that `msl-4.1-rounding` permits.

### Notes for the verifier

Two Findings are drift in documents lane 7 also reads: L1-S05, ADR 0003's
and ADR 0017's slot figures, which the render section files as L1-R03,
and L1-S12, `CONTEXT.md`'s Thin Primitive entry; where another section or
lane files the same sentence, the triage keeps one. Two
leads the render reader passed on, the `unsafe` creation of the main and
culling modules with wgpu's loop bounding off and the culling pass's 256
invocations a workgroup, were judged by reader B and filed as neither;
the gap in the loop check that `unsafe` rests on is L1-S07.

#### surface shading, coverage, Thin Primitives, small Instances and Texture classes

- **The same samples in every fragment of a pixel.** ADR 0027 and
  `common.wgsl:230` say every fragment of a pixel keeps the same samples.
  The mask depends on the count alone, and the masks of the five counts
  are nested (0b0001 ⊂ 0b1001 ⊂ 0b1011 ⊂ 0b1111), but a Thin Primitive's
  count is computed per fragment from `pixel_metres`, interpolated at the
  centroid, and from the provoking vertex's flat `radius_metres`. Two
  fragments of one pixel, from two faces of a tube, can therefore round to
  different counts where the coverage sits at a threshold. With a uniform
  Instance scale every vertex of a ring has the same centre and radius, and
  the faces' interpolated spans differ by about the radius over the
  distance (1e-4 relative for a 1 cm tube at 100 m), so the disagreement is
  confined to a band that narrow at each threshold, and nested masks keep
  the pixel's total monotone. Under a non-uniform Instance scale the radius
  `length(rotation_scale * normal * r)` differs per vertex, so faces of one
  tube can disagree everywhere; no Reference scene scales a Thin Primitive
  non-uniformly that this reader found (`src/scenes/town.rs:976-1180` places them),
  so nothing is filed. To settle: a GPU readback of the depth twin's sample
  mask for a tube under a scale of (2, 1, 1).
- **`discard` before `lit` in `fs_thin_main`.** WGSL defines `discard` as
  demoting the invocation to a helper, so the five `textureSample` calls and
  `dpdx(normal)` after it stay in uniform control flow and the module
  validates. Whether Metal's lowering (`discard_fragment()`) keeps a
  discarded member of a quad computing for its neighbours' derivatives is a
  backend behaviour the registry does not quote. A thin fragment discards
  where its kept samples are 0, which near a tube's end varies inside a
  quad. To settle: dump the lit module's MSL (`RUST_LOG=wgpu_hal=debug`)
  and look at a tube's end in `town` at 8x for a mip or roughness seam on
  the last pixels.
- **Culling and `vs_thin` agree on the coverage.** The culling pass
  (`culling.wgsl:316-326`, `:401-411`) calls the same
  `small_instance_coverage` with the same composed centre and the same w,
  in another module. Where its compiled rounding differs and the coverage
  lies at 1 or at an eighth, the two can disagree by one rounding: a
  culling verdict of "small" over a `vs_thin` coverage of 1 draws the
  Instance unscaled through the twin, and a drop decided at 0.1249999 loses
  an Instance `vs_thin` would have kept one sample of. Both are one
  rounding wide; no Finding.
- **Metal's fast math and the NaN guards.** `msl-4.1-fast-math` lets the
  compiler assume no NaNs and no infinities. `small_instance_coverage`
  divides by `view_depth · pixel_area`, 0 at w = 0, and relies on
  `view_depth > 0.0` in the select, which is evaluated on the input, not
  on the quotient, so the assumption cannot lose it; `coverage < 1.0` on a
  NaN is never reached with w > 0 and a camera whose
  `pixel_size_per_metre` is positive (`src/render/renderer.rs:199-201`).
  `surface_normals`' `crossed * inverseSqrt(0)` is M3.5's.
- **The neutral normal test emulates the sampler.** It decodes the texel
  with `unpack2x16float`, where the lit pass samples a 1×1 `Rgba16Float`
  slice through `data_sampler`, trilinear at 4x anisotropy. Filtering one
  repeated texel returns it where the filter's weights sum to exactly 1,
  which the registry does not promise. A capture of a Material without a
  normal Texture (`pbr-sweep`) at the lit pass is the end-to-end check.
  The test's floor comment names Vulkan's `sqrt` allowance only; Metal's
  "sqrt(x): Implemented as x * rsqrt(x)" (`msl-4.1-fast-math`) allows the
  same, and the M4 Pro passing says its `rsqrt(1)` is 1.
- **Far proxies and point lights.** The M5 spec's Far proxy decision says a
  proxy is "Lit by the sun and the ambient term only"; it is not a story
  and ADR 0026 has no such clause. `lit` does not tell a Far proxy from a
  Placement, so a point light within the Load radius whose range reaches a
  proxy's pixel, which the Froxel lists allow, lights it. Whether `world`
  has a light near enough to its Load radius for this to show is the
  verifier's to check (a night frame at the Load radius), or the owner's
  to rule a non-issue.
- **Unwidened tubes are recomposed.** `vs_thin` rewrites every Thin
  Primitive vertex in front of the camera as `centre + radial · (drawn /
  radius)`, factor 1 where the tube is wider than a pixel, so its position
  differs from `vertex_output`'s by up to two roundings (and a 2.5 ULP
  quotient where `radius / radius` is not exactly 1). The depth and lit
  twins compute it alike, so depth-equal holds; a Thin Primitive sharing
  an edge with an ordinary Primitive of its Mesh could open a crack of
  that size, 12 µm at 100 m, under the precision bar.
- **Checked and sound.** `THIN_NARROWEST_VIEW`'s comment, an equilateral
  triangle's narrowest width 1.5 R = 0.75 · 2R; the farthest-corner skew
  (the four sign patterns of ±a ± b ± c give the four expressions at
  `common.wgsl:263-266`); the face-pair projected area 4 \|c · (aᵢ × aⱼ)\|
  / w³, checked against the exact projection for the oracle's ASIDE case
  (Block A3); `froxel_mask_bit`'s new midpoint, equal to the old for every
  count; the M3.5 exit's rewritten tolerance comments (`:15-23`,
  `:1227-1247`), whose figures equal M3.5's derived floors, the GGX peak's
  1.3e-6 being 2.5 × 2⁻²¹ + f32(π)'s 1.4e-7 at 5.09; `shading.wgsl:262`'s
  "(M6)" for the probes, as `docs/agents/audit.md:95` names M6's scope;
  the two timing comments moved to `docs/performance.md:27-28`, whose rows
  hold the removed figures (10 to 12%, 25 µs and 5%);
  `pixel_size_per_metre` is 2 tan(fov/2) / height (`renderer.rs:199-201`),
  a pixel of the target's height, as ADR 0027's "a pixel wide" reads it.
- **Untested functions.** `thin_fragment_samples` (a select between two
  oracled functions) and `surface_normals` (the former entry point's body)
  have no oracle; neither is a shading function, so the checklist does not
  ask for one.

#### culling, LOD selection and module assembly

Checked and sound, but subtle:

- **The three-component scan and the frame's draw.** Offsets are exclusive
  prefix sums in slot order, within a block by a Hillis and Steele scan
  between barriers in uniform control flow (`culling.wgsl:92-109`; every
  invocation calls it, none returns early), across blocks by one workgroup
  whose loops have the same bound in every invocation (`:474-496`).
  Saturating addition, min(a + b, 2³² − 1) on unsigned values, is
  associative, so the scan's order does not change a saturated sum, and a
  saturated sum is past every capacity (`NOT_DRAWN`, `:79-81`). Atomics take
  only maxima and counts (`:581-592`) and the workgroup's visibility words
  (`:438`, read after the scan's barriers at `:442-445`, the words
  zero-initialised by wgpu's default `zero_initialize_workgroup_memory`,
  `src/render/bind_groups.rs:578-581`).
- **The slot word.** `SLOT_LOD_SHIFT` 28 leaves the LOD bits 28-31, the
  small bit 27 and 27 bits of in-block record offset. `MESH_MAX_LODS` 8 < 16
  is asserted (`src/render/culling_pass.rs:75`), so a drawn slot's x word is
  at most `0x7FFF_FFFF` and never `NOT_DRAWN`; the record capacity, 2²⁰, is
  asserted under 2²⁷ − 1 (`:223`), so a saturated in-block offset, 2²⁷ − 1,
  plus any block offset is past the records and `fits` refuses it. No legal
  capacity overflows the field: `DrawCapacity::FIXED` is a constant, not a
  Budget's.
- **Overlap of the two phases' regions.** Phase two's Thin Primitive indices
  fill [capacity − P₁ − P₂, capacity − P₁) while phase one's ordinary
  indices, already drawn, end at E₁ ≤ capacity − P₁. A phase-two Instance
  writes only where it fits, which needs its first ordinary index ≤
  capacity − P₁ − P₂ (`:552-553`), and that index is at least phase one's
  ordinary sum S₁ ≥ E₁; so no written index of phase two lands on phase
  one's. If phase one overflowed, S₁ (or its record sum) is already past the
  capacity and phase two draws nothing.
- **Thin indices reserved for Instances that do not fit.** P, a phase's
  Thin Primitive index sum, counts every drawn slot's, and `fits` takes all
  of it off the ordinary indices' capacity (`:547-553`): a later slot's Thin
  Primitive indices can push an earlier Instance out, and past the buffer's
  size no Instance of the phase draws. The set drawn stays a prefix in slot
  order and every refused Instance is counted, so the comment at `:540-545`
  holds in its letter; the Renderer test of overflow
  (`src/render/renderer.rs:1344`) exercises ordinary indices, not Thin
  Primitives'. Not filed.
- **Loops of the unbounded modules.** Every loop of `CULLING` and `MAIN`
  ends at this commit: `exclusive_scan`'s stride takes powers of two up to
  2³¹ and the bound is at most 2³¹ (`mod.rs:47`); `append_triangles`' index
  starts under 32 and steps 32 to at most 3 × 124 = 372, under 2³¹;
  `froxel_mask_bit`'s binary search shrinks [low, high) each step whatever
  the list holds (`main.wgsl:286-293`); the rest are counted loops with
  fixed bounds (L1-S07 lists them). No counter or bound wraps.
- **`instance_lod` edge cases.** Distance 0 or the camera inside the sphere
  gives LOD 0, the Deviation row's case. A zero scale gives every error 0,
  so the coarsest LOD at any distance above 0, of an Instance that draws
  nothing visible, and `small_instance_coverage` gives it coverage 0, which
  CULLING drops. `FORCED_LOD` is clamped to each Mesh's coarsest and to 7 on
  the CPU. A Mesh with `lod_count` 0 selects LOD 0. The LOD error word is
  written in phase one for every slot with a Mesh, shadow proxies included.
- **The widest tube.** `widest_tube_coverage` reads LOD 0's Meshlets while
  the Instance may draw a coarser LOD; the cook locks a Thin Primitive's
  vertices in every LOD (`src/cook/mesh.rs:293-297`, `:340-347`) and the Mesh
  table keeps each Meshlet's radius, so the widest is the same.
- **The bounds' growth.** The thin widening is taken at the box's farthest
  w before growth; no vertex lies past it, so the radius bounds every
  vertex's move. A degenerate axis (`half_lengths` 0) gets a growth near
  1e27 times a zero clip vector, which stays 0: the widening along a flat
  axis is lost, as the comment's orthogonality limit already admits.
- **`COMPOSITION_ERROR` and Far proxies.** ADR 0014's 0.041 mm assumes
  vertices within 128 m of the pivot; a Far proxy's reach 190 m (ADR 0026
  line 21), unrotated. Its rounding then exceeds 0.041 mm only for a vertex
  past 128 m drawn within a few metres of the camera, and there
  `OCCLUSION_ROUNDING` × w, 6.1e-5 of the depth, already passes 1e-4 m
  beyond 1.6 m. A Late Chunk's proxy is the one drawn that near. Not filed.
- **The cooked order comment** at `culling.wgsl:636-637` ("so the draw
  rasterizes as the cooked index stream does") reads within each draw: the
  Mesh table puts a LOD's Thin Primitives' Meshlets after its others
  (`src/assets/pools.rs:539-543`) and they draw apart. Not filed.
- **ADR 0027 line 29**, "so whether it is drawn does not follow the frame
  before": which phase draws a small Instance still follows the frame
  before's bit, as every Instance's does; whether it shows does not. Not
  filed.

Leads passed on by the render reader, judged and not filed:

- **The `unsafe` of loop bounding and ADR 0021's "the one `unsafe`".** The
  call is `unsafe`: wgpu 30.0.2 (the version `Cargo.lock` pins) declares
  `pub unsafe fn create_shader_module_trusted` with a `# Safety` section
  making the caller answer for what the disabled checks would catch
  (`wgpu-30.0.2/src/api/device.rs:164-184`). No ADR records it: the only
  `unsafe` an ADR mentions is ADR 0021 line 15, "Ray queries need
  `ExperimentalFeatures::enabled()`, an `unsafe` call [...]; it is the one
  `unsafe` this ADR adds". That sentence counts what ADR 0021 adds, not the
  codebase's `unsafe`, and stays true (`src/gpu/mod.rs:78`); the source has
  `unsafe` in eight other files as well. No ADR clause asks for an `unsafe`
  to be recorded in an ADR. The project's rule is `CLAUDE.md`'s "No
  `unsafe` unless absolutely necessary and clearly explained", which the
  SAFETY comment at `mod.rs:242-245` meets in form, citing the registry's
  quote of what the caller must ensure; the gap in what it relies on is L1-S07.
  Whether turning a runtime check off is a decision for an ADR is the
  owner's call; it is reversible by one field. Not filed.
- **256 invocations a workgroup against the floor.** The floor requests and
  checks the limit: `floor_limits()` is `..wgpu::Limits::default()` past
  four raised fields (`src/gpu/mod.rs:310-323`), so it carries the
  default's `max_compute_invocations_per_workgroup` 256,
  `max_compute_workgroup_size_x` 256 and `max_compute_workgroup_storage_size`
  16,384; the device is requested with exactly those limits
  (`src/gpu/mod.rs:79`), and `check_floor` compares every field of them
  with the adapter's before that (`:335`). The culling module's workgroups
  are 256 × 1 × 1 (`culling.wgsl:383`, `:472`, `:569`, `SCAN_BLOCK_SIZE` =
  256 at `src/render/culling_pass.rs:67`) and 32 × 1 × 1 (`:616`), with
  4,128 bytes of workgroup storage, all within the floor; wgpu-core checks a
  pipeline's workgroup size against the device's limits at creation, so a
  larger `SCAN_BLOCK_SIZE` would fail at device creation, not at run time.
  At the floor's edge with no margin, and no compile-time assert ties the
  constant to it (the unasserted constraints below). Not filed.

Could not settle without running code:

- **The LOD in the two phases.** Phase two recomputes `instance_lod` in a
  pipeline specialised with other overrides; under Metal's default fast
  math ("FP contract to be fast", `msl-4.1-fast-math`) the two pipelines
  could round `selected_lod`'s products differently, so at a boundary a
  phase-two Instance might draw a LOD whose error differs from the one phase
  one wrote into the fourth word, which the shadow mask reads. To settle: a
  GPU test placing Instances at distances straddling a LOD boundary within
  an ulp, hidden in phase one and drawn in phase two, comparing `slot_lod`
  with the error word.
- **Shear in a Reference scene** (L1-S04): whether any scene places a
  non-uniformly scaled parent with a turned child; grep the scene builders
  for a parent `Transform` with unequal scale components and a child with a
  rotation.
- **Unasserted constraints.** `SCAN_BLOCK_SIZE` must be a multiple of 32
  (`visible_words`, `culling.wgsl:77`, `:438`, `:442-444`; the visibility
  buffer of `instances / 32` words, `src/render/culling_pass.rs:279`) and at
  most the floor's 256 invocations a workgroup; neither is asserted, and 256
  meets both. Pipeline creation would fail on the second; the first would
  index past `visible_words` silently clamped.

#### the shadow mask and its rays

Checked and sound, but subtle:

- The back-face loop ends. `shadowed`'s loop (`shadow_mask.wgsl:185-202`) runs at most twice: the second turn sets `RAY_FLAG_TERMINATE_ON_FIRST_HIT` (`:200`), so `any_hit` is true and the `if` at `:196` breaks whatever the hit. The module also keeps wgpu's loop bounding (`src/render/shadow_mask_pass.rs:145` uses `create_shader_module`; `LOOPS_UNBOUNDED` is `[MAIN, CULLING]`, `src/render/shaders/mod.rs:193`).
- A ray past a passed back face cannot pass a real occluder beyond it: the second segment starts at `hit.t + offset` (`:201`) and any triangle of either TLAS shadows it; a front face nearer than the back face is the nearest hit and shadows at once. Only a triangle within `offset` (at least 2^-16 of the coordinate, 1/32 of a pixel) past the back face is skipped.
- On a miss, `hit.instance_custom_data` is read and used as an index (`:192`) before `found` is tested. Its value is undefined without a committed hit, but the module is created with wgpu's default runtime checks, bounds checks included, and the result enters only `shadows`, which `hidden = found && shadows` (`:197`) discards. No defect; a reordering would also skip the load on a miss.
- The height is an upper bound as its comment says: the stored triangle normal is within 0.9532° of the true one (L1-S09's figure) under rounding to nearest, so |d · (n_true − n_stored)| ≤ 2 sin(0.4766°) = 0.0166, under `AGREEING_TRIANGLE_TURN_MARGIN` = sin 2° = 0.0349; under rounding to either, 2 sin(0.953°) = 0.03327, still under it.
- Units: the cooked LOD error is meshoptimizer's relative error times `simplify_scale`, summed down the chain, in metres of the Mesh (`src/cook/mesh.rs:375`); the culling pass multiplies it by the Instance's largest axis scale (`culling.wgsl:270-275`, `:428-431`), so the word the mask reads is in World metres, the TLAS frame's, as `height` is. An unevenly scaled Instance passes back faces by its largest axis, which overstates the error along a smaller one; the implementing commit (`481abfca`) lists this as known.
- Phase one writes every slot's drawn LOD error, drawn or not, and phase two copies it back (`culling.wgsl:428-431`, `:446-456`), so the mask does not depend on culling. Two stale cases the implementing commit lists as "known and left": a static TLAS entry whose slot was reused before its rebuild reads the new Instance's error, and an entry whose slot lies past `instance_slot_count`, which `offset_slots` does not write (`culling.wgsl:446`), reads an earlier frame's word. The ADR 0008 row reads this as the static TLAS's decided lag, not screen-space history; a stricter reading would make the second case a read of history.
- `shadow_ray_mask` never returns 0 and always meets ordinary geometry and Far proxies (bit 0); its widest class, class 6, ends at 400 mm, past which a tube takes the ordinary bit; a pixel width of 0, or any up to 6.25 mm, gives every class, an infinite one none, a NaN one or the other, every result with bit 0 (Oracles, block C1).
- The width classes' Rust and WGSL sides agree at the bounds: a tube exactly on a lower bound 2^k/320 takes class k (`ilog2` of an exact power), and a receiver pixel exactly on an upper bound includes the class (n = k at x = 2^k), the "no narrower than" of the comment.
- The re-anchored static TLAS and the dynamic TLAS share the frame each frame: the Renderer takes `static_tlas.anchor()` after `update` swaps the spare in and passes the same value to `dynamic_tlas.fill` and the uniform (`src/render/renderer.rs:724-737`).

Could not settle without running or a source the registry lacks:

- L1-S10's visible extent. `world`'s frames 59, 300 and 599 show nothing (`docs/performance.md:34`). To settle: `praxis shadow-mask-difference world <png> 2560x1440 --frame <n>` at frames with buildings at a LOD past 0 inside the Load radius, and a probe that marks, per differing pixel, whether the passed back face belongs to another Instance than the receiver's (the receiver's slot is not in the mask's inputs, so this needs a probe that reads the depth pass's Instance or compares against a build where the pass is limited to `hit.t` below the first front face of the hit Instance).
- Whether meshoptimizer's `result_error` from `simplify_with_attributes_and_locks` (`src/cook/mesh.rs:357-369`) includes the weighted attribute error. If it does, the cooked "LOD error" can exceed the geometric deviation, and `LOD_ERROR_MARGIN` passes higher back faces than the geometry needs (more of L1-S10's band). The registry's `meshoptimizer-1.3-simplify-error` quote speaks of the target error only; meshoptimizer 1.3's `meshopt_simplifyWithAttributes` documentation would settle it.
- `LOD_ERROR_MARGIN`'s comment says "`orrery`'s spheres meet LOD 0 up to three times theirs high". `docs/performance.md:34` reports pixel counts per margin, 3.0 and 1,000.0 equal on all 33 frames of the moving view, not a measured height. The equality is consistent with every passed back face lying within 3 errors, not a measurement of it.
- Mirrored Instances. The back-face pass trusts `hit.front_face`. If a backend decides facing in object space, as some ray-tracing APIs do for Instances whose transform mirrors, a mirrored Instance's faces read the other way round, and a leaving ray would pass its true front faces within the band. The registry holds no key on facing under a mirroring transform; whether any Placement has a negative determinant was not checked. The rule existed for rays starting inside the surface before the scope; the scope extends it to leaving rays.
- Small Instances scaled up to a pixel (ADR 0027) and Thin Primitives widened: the receiver point lies on the drawn, enlarged surface, outside the true LOD 0 the BLAS holds. For a widened tube, convex about its centre line, a leaving ray cannot meet the true tube; for a non-convex small Instance scaled up to eight times, a leaving ray could meet its own true front faces. Sub-pixel either way; `town`'s far flocks are where it would show.
- The figure "the camera adds less than 4.9 mm to a shadow ray's offset once that build is bound" holds for the anchor the build took; the build re-anchors at the camera's Anchor when the worker starts (`src/render/acceleration_structures.rs:572-581`), and the camera moves in the frames until it is bound. The ADR words it "once that build is bound"; the strict bound is render's to state.
- Story 13's letter (a pixel's width along the ray) and the code's rule (3 drawn LOD errors of height) differ; the change that made the rule measured both (`docs/performance.md:34`) and its message says the owner decided under the change's Decide items. If the spec's letter is meant to bind, that is a Finding; this reader read the spec's purpose clause as the bar.

Suspicions without evidence of the lane's kind:

- Cost, for the budget lane: every hit now reads a slot's word through `instance_custom_data` (`:192`), which the implementing commit measured at 0.44 ms of `town`'s shadow-ray time (3.9%) on the M4 Pro, and every leaving ray's first segment is now a nearest-hit query where it was an any-hit one; both were accepted in that change.
- `max(NaN, 1.0)` in `shadow_ray_mask`: WGSL leaves which operand a NaN `max` returns to the backend; either result holds bit 0, so no ray misses ordinary geometry. No pixel width the pass computes is NaN for a depth above 0.

### Counts

| Severity class | Findings |
|---|---|
| `image` | 0 |
| `budget` | 0 |
| `correctness` | 4 (L1-S02, L1-S04, L1-S07, L1-S10) |
| `drift` | 9 (L1-S01, L1-S03, L1-S05, L1-S06, L1-S08, L1-S09, L1-S11, L1-S12, L1-S13) |
| `health` | 0 |

Clause verdicts: 173 rows (A 64, B 59, C 50); `holds` 154, seven of them
with a Finding on the clause, its wording or its record (L1-S01 twice,
L1-S04, L1-S06, L1-S07, L1-S10, and one row carrying L1-S11 and L1-S12);
`broken` 4 (L1-S02 on ADR 0027's scaling and its mean-coverage
Consequence, L1-S05 on ADR 0003's and ADR 0017's slot figures);
`superseded` 0; split between this module and render 2, the shader half
holding; falls in another reader or module 11; not applicable 2. Spec
stories: 34 rows (A 8, B 15, C 11); 22 hold, M5 story 8 with L1-S04, M5
story 13 in intent and not in letter with L1-S10, M5 story 37 in the
glossary's reading with L1-S12, M5 story 39 on the view axis only with
L1-S01 and L1-S02; 5 hold in part, the rest falling in another reader;
6 fall in another reader or module; 1 not applicable. Registry keys: 25
rows (A 7, B 7, C 11), every cited key matching its quote:
`meshoptimizer-1.3-lod-selection` as its Deviation row says, `greene-hiz`
and `meshoptimizer-1.3-simplify-error` in substance,
`tabellion-lamorlette-2004-ray-origin-offset` for what it is cited for
with L1-S10 on its warning; one rule relied on with no key (the binary32
exponent `shadow_ray_mask` reads) and one key not applicable (loop
bounding, which the shadow mask module keeps). Oracles: 16 rows over 13
tests; `exact` 7, `explained` 5, `tight` 3 (the neutral normal's
`sqrt(1)`, `position_at` as its comment says, the octahedral 8-bit bound,
L1-S09), `loose` 1 (`thin_drawn_radius`, about 40×), `unexplained` 0;
the 36 tests of the file pass on the M4 Pro.

## tooling

Scope: `src/bench/` with `history/`, `src/stats/`, `src/bin/`,
`src/capture/`, `src/preflight/`, `scripts/` with the Cycles reference,
`tests/` but the Goldens' PNGs, and `.github/` for the clauses of ADR 0025
that scripts and workflows carry, as far as they changed after `8bc34eb1`:
4,952 changed lines under `src/bench/`, 716 under `src/stats/`, 583 in
`src/bin/praxis.rs`, 463 in `src/capture/mod.rs`, 266 in
`src/preflight/mod.rs`, 896 under `scripts/`, 1,810 under `tests/` and 293
under `.github/`. Four readers at high effort shared the module by file,
each with the lane brief verbatim: the bench's run, report and provenance;
the Bench history with the night's sweep; Frame stats, the GPU memory
ledger, capture, preflight and the CLI with their tests; and the other
scripts, the Cycles reference, the workflows and the tests that enforce
written rules. Each read its files' diff since `8bc34eb1` hunk by hunk and
the code around every hunk, most files in full (their statements precede
the Counts). ADRs read clause by clause: 0013 and 0024 by every reader,
0025 by the history and scripts readers; read for the clauses that fall
here: 0004, 0014, 0015, 0016, 0017, 0018, 0019, 0021, 0022 and 0023. The
`CLAUDE.md` review checklist and conventions, and `docs/agents/workflow.md`'s
Session start, Tickets, PRs, Review, Performance and memory and Correctness
sections, were read as the operational form of what bench, preflight, the
scripts and the workflows must do. Every `where` is `path:line` at the
frozen commit. Two Findings count their magnitude on the published `bench-history`
branch as read on 2026-11-23, after the frozen commit, as the record of
what the frozen import code wrote. `deviations.md` holds no row at a site in these
files; `DIFFERENCES.md` was read beside the Cycles reference. Of M3.5's
eleven tooling Findings, all are found fixed: L1-T01, L1-T02, L1-T05 and
L1-T08 in code, L1-T03, L1-T04, L1-T06 and L1-T07 by an ADR amendment,
and L1-T09 to L1-T11 in the workflows and scripts (rows marked below).
The `uploaded_bytes` lead the render section handed to tooling is settled as L1-T05.

### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0004 | "every Frame data must reach the renderer, in order" | holds for capture: drawn and held frames ascending, each prepared then rendered; undrawn frames run the simulation alone and build no Frame data | `src/capture/mod.rs:354-394` |
| 0013 | "Budgets are judged on an Apple M4 Pro ... the RTX 2070 SUPER is the hardware floor, benched for comparison on Vulkan and on DX12" | holds (bench half): Baselines are keyed by adapter, and by backend unless Vulkan or Metal, so the RTX's DX12 runs have a directory of their own; CI's lavapipe judges counters and memory only; `compare` still applies the Budgets to any adapter's Local run (the last audit's note stands) | `src/bench/mod.rs:146-164`, test `:3118`; `bench/baselines/nvidia-geforce-rtx-2070-super-dx12/`; `.github/workflows/ci.yml:286-296` |
| 0013 | "the reference machine ... gates regressions, the Budgets being judged on it" | holds in the history's half: Suspects are marked on every non-lavapipe Series, but only the M4 Pro runs a sweep, so only its Regressions are filed; the history judges no Budget | `src/bench/history.rs:391-394`; `scripts/sweep.sh:16-18` |
| 0013 | "a technique rejected for its cost ... is final only with the figure on both adapters ... from one engine source, their `provenance.source_hash` equal" and "(the Bench records of source hash `6a91ca76`)" | holds: every result carries `source_hash` in its Provenance, and a record holds the run's whole result keyed by source hash; `records/<adapter>/town/6a91ca76….json` exists on `bench-history` for the M4 Pro, the RTX 2070 SUPER on Vulkan and on DX12, and lavapipe; the judging is process | `src/bench/provenance.rs:238-248`, `:276-281`; `src/bench/history.rs:755-794` |
| 0013 | "When the owner owns target-class hardware, the Baselines are re-recorded on it" | not applicable yet | — |
| 0013 | "60 fps at 1440p native with CPU at most 6 ms and GPU at most 12 ms per frame" | holds: `REFERENCE` is 2560x1440, frames step at `SIXTY_HERTZ`, Budgets 6000 and 12000 | `src/bench/mod.rs:61-67`, `:893-894`; `bench/budgets.json:2-3` |
| 0013 | "CPU time summed across threads is neither measured nor budgeted yet, Frame stats holding wall-clock times per system, stage and frame only" | holds: `cpu_times` are system spans and named stages on the frame's thread, `cpu_frame_time` the frame's wall time, `static_tlas_build_time` the worker's wall time on one build (a field ADR 0020 asks for); the bench keeps each system's and stage's time per frame and sums nothing across threads | `src/stats/mod.rs:16-20`, `:47-50`, `:303-318`, `:410-420`; `src/bench/mod.rs:240-244`, `:1022-1035`, test `:2845` |
| 0013 | "the 6 ms bounds the wall time of each pipeline stage's critical path, waits on worker threads included" | holds at the seam: a stage's time is the wall-clock span on the main thread from its start, so a wait on the compute pool inside it counts | `src/stats/mod.rs:300-310`; `src/state.rs:278-300` |
| 0013 | "the CPU frame, the GPU frame and ADR 0019's gate answer to their Budgets at the p99 as at the median, the CPU frame at its p95 too" | holds (M3.5's L1-T01 fixed): median, p95 and p99 of the CPU frame, median and p99 of the GPU frame and the gate; the report's note on it is L1-T06 | `src/bench/mod.rs:1270-1273`, `:1301-1311`, `:1331-1334`; tests `:2036`, `:2198` |
| 0013 | "no CPU or GPU frame lasts longer than 16,667 µs, one 60 Hz period" | holds over the measured frames: `maximum_frame_microseconds` 16667 against both maximums; warm-up frames are not judged | `bench/budgets.json:8`; `src/bench/mod.rs:1274-1281`, `:1312-1318`; test `:2072` |
| 0013 | "Input latency at most 2 frame periods ... judged at p95" | not implemented: no metric and no `budgets.json` key; the glossary's Budget entry says one "is planned, once it is measured" | `bench/budgets.json:1-19`; `CONTEXT.md:303` |
| 0013 | "a 2 km LOD'd view distance (drawn past the Load radius as a Far proxy per Chunk, ADR 0026)" | falls in render and streaming | — |
| 0013 | "GPU memory at most 8 GB at 1440p, counted as the bytes of the buffers and textures the engine has created, summed by the engine itself on every backend" | holds: each buffer, texture and TLAS created through `GpuMemory` adds its bytes to its kind's running total until dropped, and `gpu_memory_bytes` is the sum of the seven totals; `compare` judges it against 8,000,000,000 at the measured frames' median in every mode, which verify and CI run with `--ci`; the load staging's belt, which bypasses the ledger, is L1-R04, and `gpu_memory_bytes`'s doc omits it the same way (Notes) | `src/stats/gpu_memory.rs:109-164`, `:211-249`, tests `:315-337`; `src/state.rs:369-372`; `src/bench/mod.rs:1227-1232`; `bench/budgets.json:9`; `scripts/verify.sh:55`; `.github/workflows/ci.yml:296` |
| 0013 | "because wgpu has no allocator report on Metal"; "wgpu's allocator report and Metal's `currentAllocatedSize` are diagnostics only" | holds: an `Option`, printed only, never budgeted, and in no `Measurements` field; `currentAllocatedSize` is read nowhere | `src/stats/mod.rs:76-78`, `:249-259`; `src/bench/mod.rs:189-274` |
| 0013 | "the GPU stats readback's buffers are left out of that sum" | holds at the seam (documented); the exclusion falls in render | `src/stats/mod.rs:65-67` |
| 0013 | "so are the target the frame is presented into, a window's surface textures or capture's stand-in for them, and capture's read-back buffer" | holds: capture creates its target and every read-back buffer on the device directly, outside the ledger, and the field's doc says so; M3.5's L1-T07 is closed by this amendment | `src/capture/mod.rs:332-345`, `:465-470`; `src/stats/mod.rs:64-65` |
| 0013 | "the 8 GB splits into Budgets for geometry arenas, the texture pool, Resident tails, render targets, acceleration structures and staging ... a seventh Budget covers the scene and pass buffers" | holds: seven `GpuMemoryKind`s, each named as `bench/budgets.json` keys it, `GpuMemoryBudgets` denying unknown fields; Frame stats carry and print each kind; `compare` judges each at its measured frames' median in every mode | `src/stats/gpu_memory.rs:9-66`; `src/stats/mod.rs:69-72`, `:252-255`; `bench/budgets.json:10-18`; `src/bench/mod.rs:1036-1047`, `:1233-1238`, test `:2167` |
| 0013 | "83,886,080 bytes for geometry arenas, 4,294,967,296 ..., 234,881,024 for the scene and pass buffers, 5,955,911,680 in all" | holds: the seven values equal the ADR's and, recomputed, sum to 5,955,911,680, under the 8,000,000,000 the test checks | `bench/budgets.json:11-17`; test `src/bench/mod.rs:2182` |
| 0013 | "acceleration structures join the count (amended 2026-10-12, ADR 0017)" | holds: the `AccelerationStructures` kind, `create_tlas`, and `charge` for BLASes | `src/stats/gpu_memory.rs:17-20`, `:188-220` |
| 0013 | "each capacity is its share of its kind's Budget at its real bytes per entry" | falls in render and assets (`PoolCapacities::from_budgets`, `RenderCapacities::from_budgets`), which read `bench/budgets.json` compiled in | `src/bench/mod.rs:731-736`; `src/assets/pools.rs:75-107`; `src/render/renderer.rs:259-291` |
| 0013 | "Budgets were re-decided ... each stays as written ... and so do the dynamic TLAS's ceilings of 1.1 ms of CPU and 1.0 ms of GPU" | holds: 6000, 12000, 6000, 1100 and 1000 unchanged since `8bc34eb1` (the diff adds keys only); the TLAS ceilings judged at the median | `bench/budgets.json:2-6`; `src/bench/mod.rs:1335-1348`; test `:2324` |
| 0013 | "what `town` breaches is ticketed as an offender, not budgeted away" | holds: no Budget rose | `bench/budgets.json` (diff since `8bc34eb1`) |
| 0013 | "A budget moves down freely and up only through an ADR or a map comment" | holds: the three keys added (guard, maximum, kinds) each came with an ADR 0013 amendment | `bench/budgets.json:7-18` |
| 0013 | the guard: "the median time of both culling phases' per-slot stages (`offset_slots`, `offset_blocks` and `append_records`, timed apart from `append_triangles`) stays at or under 1,000 µs" | holds: each frame sums the two phases' `_slots` passes, which run the three stages and nothing else (pinned by Frame stats' pass list test); failed only when the median is over 1000 | `src/bench/mod.rs:1010-1020`, `:1349-1361`, tests `:2272`, `:2735`; `src/render/culling_pass.rs:47-52`, `:359-371`; `tests/frame_stats.rs:283-300`; `bench/budgets.json:7` |
| 0013 | "`praxis compare` fails a scene past it, on the adapter its Baseline is from, as it does one past a Budget" | holds: Local mode only, after the adapter and backend check; `--ci` passes it, so neither verify nor CI judges it | `src/bench/mod.rs:1239-1261`, `:1349-1361`; test `:2315-2318` |
| 0013 | "the Bench record's counters hold the Instance slots in use" | holds: `instance_slots_in_use` per frame in Frame stats, the median of its frames in the record, judged as any counter, and in `Counters::NAMES`, which the Series row's `measured_counters` reads | `src/stats/mod.rs:32-34`; `src/bench/mod.rs:334-336`, `:384`, `:411-436`; `src/bench/history.rs:1222-1228` |
| 0014, checklist | "No World position reaches the GPU in f32" | holds as tested: every Golden scene, `world` included, must match its Golden shifted 100,032 m, 1,563 whole 64 m anchor cells | `tests/goldens.rs:58-64`, `:197`, `:225-231` |
| 0015, checklist | "Depth has no far plane" | holds (test): a depth clear passes only as a zero literal, a `Less` comparison never | `tests/conventions.rs:410-446`; tests `:448-461` |
| 0016 | "Only the main thread submits, presents or polls the wgpu `Queue` and `Device`" | holds for the bench (`run` and `traverse` render through `render_frames` on the calling thread) and for capture (submits, polls and maps on the calling thread); each Goldens capture owns its device, read as the device's owning thread, as the last audit noted | `src/bench/mod.rs:836-900`; `src/capture/mod.rs:366-388`, `:506-519`; `tests/goldens.rs:285-297` |
| 0016 | "no stage runs a frame behind another" | holds for capture: each drawn or held frame runs `update` then `render` | `src/capture/mod.rs:383-388` |
| 0016 | "`desired_maximum_frame_latency` stays 2" | holds at capture: at most `FRAMES_IN_FLIGHT` (2) in flight, the window's swapchain latency | `src/capture/mod.rs:352`, `:366-382`; `src/render/mod.rs:44`; `src/lib.rs:166` |
| 0017 | "The Renderer and its Pools create the buffers, textures and TLASes they count through one ledger, and charge it their BLASes' bytes; it keeps a running total per kind from creation until drop, and `gpu_memory_bytes` is the sum of those totals" | holds | `src/stats/gpu_memory.rs:104-117`, `:211-249`; `src/state.rs:369-372`; tests `src/stats/gpu_memory.rs:315-337`, `:428-460` |
| 0017 | "What ADR 0013 leaves out, the GPU stats readback and the load staging, bypasses it" | holds in code: neither is created through the ledger; that ADR 0013 does not name the load staging is L1-R04 | `src/render/staging.rs:344-353` |
| 0017, 0013 | "A `TRANSIENT_ATTACHMENT` counts as zero only when `transient_saves_memory` is `Some(true)`" | holds; the pbr-sweep test pins the 4x colour at 0 on Metal and full elsewhere | `src/stats/gpu_memory.rs:128-131`, `:282-294`; `tests/frame_stats.rs:134-137` |
| 0017, checklist | "each BLAS and TLAS at the size its driver gives through wgpu-hal's build-sizes query ... and each TLAS with wgpu-core's buffer of its Instances, 72 bytes a slot on Metal and 64 elsewhere" | holds for the TLAS through the ledger: `create_tlas` charges `tlas_bytes`, and the test asks the driver through wgpu-hal itself and adds 72 or 64 a slot; the BLAS's size falls in render | `src/stats/gpu_memory.rs:188-209`; test `:339-426` |
| 0017, checklist | "A compacted BLAS's size stays an estimate ... picked by the adapter's vendor, then its backend" | falls in render (`compacted_blas_fraction`); the ledger takes it per adapter | `src/stats/gpu_memory.rs:128-138`; `src/render/acceleration_structures.rs:36-51` |
| 0017 | "an empty slice still holds memory, which `gpu_memory_bytes` counts" | holds: a Texture class's array is charged for every slice | `src/stats/gpu_memory.rs:166-186` |
| 0017 | "Each texture's Resident tail (mips of 64 KiB and smaller)" | holds in the ledger's split: a level of at most `RESIDENT_TAIL_LEVEL_BYTES` (64 KiB) a slice goes to `resident_tails`; the test's 128² RGBA8 level, exactly 64 KiB, counts as tail | `src/stats/gpu_memory.rs:106-107`, `:175-181`; test `:451-457` |
| 0017 | "A capture's Settled wait loads past the cap, so its load order stays fixed" | holds on capture's side: `capture` and `capture_frames` settle every drawn frame; the cap falls in streaming | `src/capture/mod.rs:160-177`, `:266-283`, `:383-387` |
| 0018 | "The runtime loads cooked Blobs only and references Assets by Asset ID, never by source path" | holds (CI's proof): a build without `cook` links none of the four decoders and renders every CI Golden with `assets/` removed | `.github/workflows/ci.yml:197-247`; `Cargo.toml:41` |
| 0019 | "the opaque main pass takes more than 6 ms of GPU ... as the median over the scene's camera path, timed from the first culling dispatch to the end of the lit pass with shadow rays off" | holds (bench half): `gate_microseconds` from the rays-off run's measured frames, judged at its median and p99 against 6000; the span from `culling_phase_one_slots`' start falls in render | `src/bench/mod.rs:793-801`, `:991-996`, `:1331-1334`; `bench/budgets.json:4`; `src/render/readback.rs:739-745` |
| 0019 | "Shadow-ray time is reported beside the gate as its own line" | holds: each path frame's GPU frame with rays on less the same path frame's with them off, judged against its Baseline, its own row in the report | `src/bench/mod.rs:997-1002`, `:1076-1083`, `:1321-1330`; `src/bench/report.rs:91-96`; test `src/bench/mod.rs:2907` |
| 0021 | "the binary logs it with `tracing::error!` and exits non-zero" | holds | `src/bin/praxis.rs:24-29` |
| 0021 | "`praxis capabilities` runs without the floor" | holds; preflight's adapter line also reads the adapter without requesting the device | `src/bin/praxis.rs:121`, `:459-463`; `src/preflight/mod.rs:118-144` |
| 0021 | "Headless contexts, tests and Goldens included, use the same floor" | holds: every capture in the Goldens and Frame stats tests goes through `GpuContext::headless` | `tests/goldens.rs:294-297`; `tests/frame_stats.rs:30` |
| 0021 | "CI's software Vulkan jobs request the floor's device on lavapipe"; "CI proves it on lavapipe" | holds: the Goldens run on lavapipe with `PRAXIS_REQUIRE_GOLDENS=1`, under which the test fails rather than skips without an adapter | `.github/workflows/ci.yml:173-187`; `tests/goldens.rs:88-97`, `:101-108` |
| 0022 | "Budgets stay judged at 60 Hz, the display the measurements ran on" | holds: `render_run` and `traverse` step at `SIXTY_HERTZ` | `src/bench/mod.rs:848`, `:893-894` |
| 0023, checklist | sRGB "encoded once: the tonemapping pass alone ... into 8-bit UNORM targets" | holds for capture (`Rgba8Unorm` target, PNGs written as the raw RGBA8 bytes) and as tested (no `*Srgb` format outside assets and the cooker) | `src/capture/mod.rs:11-13`, `:332-345`; `src/bin/praxis.rs:936-943`; `tests/conventions.rs:384-399` |
| 0023 | "`DirectionalLight` gives illuminance in lux and a `PointLight` luminous intensity in candela" | holds in the Cycles reference; its point light now always casts a shadow, as every praxis point light traces one | `scripts/cycles-reference/render.py:30`, `:95-122`, `:125-135`, `:403`; `scripts/cycles-reference/calibrate.py:67-77` |
| 0023 | "a Reference scene's ambient term gives illuminance in lux" | holds: world radiance E/π | `scripts/cycles-reference/render.py:137-146` |
| 0023 | "Exposure is an EV100, which scales luminance by 1/(1.2 · 2^EV100)" | holds | `scripts/cycles-reference/render.py:343`, `:380`, `:396-403` |
| 0023 | "Point lights fall off by inverse square inside KHR_lights_punctual's range window" | holds | `scripts/cycles-reference/render.py:95-122`; `scripts/cycles-reference/calibrate.py:72-74` |
| 0023 | "the tonemapping pass then maps it through GT7 for display", sRGB encoded once | holds: `compare.py` now splits the display transform into `tonemap` then `srgb_encode`, equal to `display`, and decodes praxis's capture with the sRGB EOTF for the linear RMSE | `scripts/cycles-reference/compare.py:143-163`, `:209-216`, `:275-276` |
| 0023 | "clamped to 65504" | not applicable to the Cycles reference: float EXR and f64 arrays | — |
| 0024 | "Every bench run is kept as a Bench record. A record is a file on an orphan branch, `bench-history`" | holds: with no remote tip a publish adds an empty worktree and commits with no parent | `src/bench/history.rs:962-999`, `:1150-1167` |
| 0024 | "keyed by source hash × adapter × backend × scene; the Harness version is defined under `src/`, so the source hash names it, and the record holds it" | holds: `HARNESS_VERSION` in `src/bench/provenance.rs`, written into every Provenance; the path is `<dir>/<adapter>-<backend>/<scene>/<source hash>.json`; a re-recorded Baseline's record is keyed by the commit that landed it, not the source the Baseline measured (Notes) | `src/bench/provenance.rs:22`, `:54`, `:138`; `src/bench/history.rs:221-229`, `:280-282` |
| 0024 | "The source hash covers `src/`, `assets/`, `Cargo.toml`, `Cargo.lock`, `.cargo/` and `vendor/`, a list held in one place in code, and a test fails when a new top-level path is classified neither way" | holds: `MEASURED_PATHS` is the ADR's six, and the test lists the other twelve tracked top-level paths at the frozen commit (Notes on `bench/budgets.json`); no path CI skips lies under a measured one | `src/bench/provenance.rs:27-34`, `:410-444`; `.github/workflows/ci.yml:13-28` |
| 0024 | "The source hash is one git hash over the paths that can change a measurement" | holds in practice (Notes: `bench/budgets.json` is compiled in, and the pinned capacity tests force a `src/` change with any Budget change that moves a capacity) | `src/bench/provenance.rs:320-337` |
| 0024 | "A commit whose source hash didn't change inherits the record before it" | holds: `mktree` over `ls-tree` of the measured paths, tested; `unrecorded_scenes` looks up `main`'s source hash, not its commit; pushes to `main` that change only skipped paths run nothing, and none of those is measured | `src/bench/provenance.rs:320-337`, tests `:481`, `:496`; `src/bench/history.rs:671-690`; `.github/workflows/ci.yml:11-28` |
| 0024 | "Only clean, release-profile records of `main`'s source taken by a quiet machine form a Series" | holds for published records: the bench's record carries `dirty` (untracked measured files included), `branch`, `profile` and `quiet`, and the history tags on them; broken for the import of re-recorded Baselines: L1-T02 | `src/bench/provenance.rs:143-150`, `:238-282`, test `:520`; `src/bench/mod.rs:966-971`; `src/bench/history.rs:875-920`; `src/bench/history/import.rs:374-394` |
| 0024 | "the machine quiet (preflight warned of none of the GPU load, the CPU load, other praxis processes, the power and the display, its assets warning not counting" | holds for published records, the history's tag reading the bench's `provenance.quiet` and the branch README stating the same rule and thresholds; broken for the import: L1-T02. `is_quiet` reads the GPU load against `BUSY_GPU_PERCENT`, the one-minute load average against `BUSY_LOAD_AVERAGE`, other praxis processes and `machine_warnings`, with preflight's own readers, thresholds and directions (warn above 10 % GPU, above a 2.0 load average, on any other praxis process, on battery, on a full-screen application), and the assets check feeds none of them; M3.5's L1-T06 is closed by this amendment | `src/bench/provenance.rs:221-235`, test `:548`; `src/preflight/mod.rs:13-25`, `:304-337`, `:367-399`, `:416-447`, `:479-510` |
| 0024 | "and p95/median below about 1.06 ... the p95 being that of each GPU frame over the median of the 31 frames around it, not over the run's" | holds: `NEIGHBOUR_FRAMES` 31, 15 a side, fewer at the ends, quiet only when `< 1.06`; a run with no ratio is not quiet | `src/bench/provenance.rs:36-43`, `:171-191`, `:234`; test `:631` |
| 0024 | "and no process outside praxis at or over half a core at the scene's start or end, read per scene ... where the process listing gives CPU shares, which Windows' does not" | holds: `busy_processes` at `BUSY_PROCESS_PERCENT` 50 of a core, at or over, `None` on Windows, read before the scene's first render and after its aliasing captures | `src/bench/mod.rs:786`, `:825`; `src/bench/provenance.rs:193-213`, `:232`, test `:669`; `src/preflight/mod.rs:23-25`, `:512-527`, `:600-616` |
| 0024 | "The Harness version rises whenever a change alters what a metric means" | holds for the times: four bumps since `8bc34eb1`, at the p99, maximum and refusal counters (10 to 11), the per-slot passes' split (12), Late Far proxies (13) and the far field's fill (14), and the history breaks a line on each; two counter changes kept their version (Notes) | `src/bench/provenance.rs:18-22`; `src/bench/history.rs:399`, `:1238-1243` |
| 0024 | "each record carries its scene's content hash" | holds for published records: `render_frames` returns the Cooked cache's content hash of the Blobs the run loaded, read after its last frame, into the Provenance; broken for re-recorded Baselines' records: L1-T03 | `src/capture/mod.rs:294`, `:409`; `src/state.rs:321-324`; `src/bench/mod.rs:792`, `:820-826`; `src/bench/provenance.rs:55-56`; `src/bench/history/import.rs:323-335`, `:362-373` |
| 0024 | "A chart breaks its line where either changes, so a new definition never reads as a Regression" | holds for Suspects: `same_definition` starts a line; the chart falls outside tooling; a re-recorded record without a content hash joins either neighbour (L1-T03) | `src/bench/history.rs:395-404`, `:1235-1243` |
| 0024 | "When two records share a key on a Series, or both tagged, the quieter one stays: the one with the lower GPU p95/median; a record on a Series holds its key against a tagged one" (amended 2026-11-04) | holds; M3.5's L1-T04 is settled by the amendment | `src/bench/history.rs:1027-1034`, `:1050-1056`, `:1074-1077` |
| 0024 | "lavapipe's records are the exception to the quiet test ... a record from CI's configuration joins them whether or not the runner was quiet, and the first record on a Series under a key stays" | holds: `CI_COUNTERS` as lavapipe's configuration, no quiet tag, `publishes_once`; lavapipe rows are never Suspects | `src/bench/history.rs:177`, `:391-394`, `:895-905`, `:1051` |
| 0024 | "Every other record is kept, tagged with its reasons, and drawn only on request" | holds: under `tagged/` with `tags` | `src/bench/history.rs:183-185`, `:1022-1026` |
| 0024 | "An Experiment's records are kept under its name and never join a Series" | holds | `src/bench/history.rs:200-219`, `:1019-1026`, `:1079-1080` |
| 0024 | "The commits that touched `bench/baselines/` before the history existed, 64 on `main`'s first-parent line before `HISTORY_STARTED` ... each with the Harness version in force at its commit" (amended 2026-11-04) | holds: `git log --first-parent --format=%H 3c6b9d92 -- bench/baselines` lists 64, recounted; the version steps at each of the nine breaks and the import checks it ends at 10; M3.5's L1-T03 is settled by the amendment | `src/bench/history/import.rs:24`, `:30-67`, `:156-193` |
| 0024 | "Baselines are still chosen by hand, and a re-recorded Baseline's record is marked as one" | holds: commits after `HISTORY_STARTED` give records with `provenance.source` `baseline-rerecorded`; a quieter record that holds the key takes `rerecorded_baseline`; either starts the line; tested; M3.5's L1-T02 fixed (a tagged one's mark can be lost: Notes) | `src/bench/history/import.rs:72-80`, `:194-220`; `src/bench/history.rs:313-321`, `:400-401`, `:1042-1060`, `:1982-2022` |
| 0024 | "A timing record far from its Series' recent median is a suspect" | holds: a window of 8, from 3, ± max(4 MAD, smallest shift), CPU frame 3 %, GPU frame 1.25 %, as the glossary's Suspect | `src/bench/history/suspects.rs:7-15`, `:47-101`; `src/bench/history.rs:429-453` |
| 0024 | "The sweep re-benches it, bisects the commits behind it" | holds: two interleaved runs per end, good first, `holds` at max(shift/2, smallest shift), bisect by source hash; `--no-publish` only for a commit descended from `3c6b9d92`, so M3.5's L1-T08 is fixed | `src/bench/history/verdicts.rs:216-308`, `:360-432`, `:766-772`; `src/bench/history/night.rs:21-23`, `:56-91`; `scripts/sweep.sh:151-190` |
| 0024 | "and files a confirmed Regression that no re-recorded Baseline covers as one `needs-triage` issue" | holds: `covered_by_baseline` from `remeasured` on the adapter and backend's Baseline at the pinned commit and its parent; one issue per scene and commit, an existing title reused, label `needs-triage`; run every night | `src/bench/history/verdicts.rs:394-427`, `:435`, `:452-554`, `:732-747`; `scripts/sweep.sh:198-200` |
| 0024 | "A bad publish is undone with `git revert` on `bench-history`" | not applicable (process); every write is one commit on the tip | `src/bench/history.rs:712-751`, `:1150-1167` |
| 0024 | "A page in `docs/` draws the Series" | falls outside tooling (`docs/bench/`) | — |
| 0024 | "The history observes and never gates: Baselines stay in `bench/baselines/` on `main`, and `verify` and `compare` judge against them as before" | holds: `compare` reads only a Baseline and the Budgets, and no history type reaches it; `history rerecord` writes a Baseline's counters only when a session runs it, and the session commits it; verify and CI compare counters against `bench/baselines/` with `--ci` | `src/bench/mod.rs:33-40`, `:1184-1189`; `src/bench/history/rerecord.rs:182-244`; `src/bin/praxis.rs:738-749`; `scripts/verify.sh:54-55`; `.github/workflows/ci.yml:293-296` |
| 0024 | "a nightly sweep on the reference machine that benches `main` only when its source has no quiet record and the machine is quiet" | holds: import, then `history unrecorded` (the scenes without a Series record of `main`'s source hash), then a wait until preflight warns of nothing, then each bench after a wait for the GPU and CPU load; `world` is never benched by the sweep (Notes) | `scripts/sweep.sh:80-81`, `:109-113`, `:123-128`, `:204-207`; `src/bench/history.rs:665-691`; `src/bin/praxis.rs:656-675` |
| 0024 | "The RTX 2070 SUPER has no sweep ... its Goldens and Baselines are recorded by hand at each milestone's exit" (amended 2026-11-21) | holds in the scripts: `scripts/sweep-task.xml` is gone and `sweep.sh` installs only as a launchd job on the M4 Pro; two documents still say otherwise: L1-T08, L1-T09 | `scripts/sweep.sh:16-34`; `scripts/sweep.plist:1-29` |
| 0024 | Considered options: "A local scheduled task runs no code from a pull request" | holds: the clone moves to `origin/main` only; a bisect benches `main`'s first-parent commits between the Verdict's ends | `scripts/sweep.sh:67-72`; `src/bench/history/verdicts.rs:603-625` |
| 0024 | "Records publish themselves: ... `praxis bench` by default in sessions"; "Publishing never blocks work. A failed publish warns and the bench still succeeds" | holds; the night logs a failed `publish-night` and goes on | `src/bin/praxis.rs:231`, `:291`, `:596-610`; `scripts/sweep.sh:191` |
| 0024 | "Records publish themselves: CI's lavapipe on every push to `main`" | holds | `.github/workflows/ci.yml:11-28`, `:329-353` |
| 0024 | "The branch holds only data and a README, is never merged, and CI writes it only from pushes to `main`, never from a pull request" | holds: every publish, import and Verdict rewrites `README.md` beside the data; `github.event_name == 'push'` on a `main`-only push trigger; the new recording workflow holds `contents: read` | `src/bench/history.rs:1086`; `src/bench/history/verdicts.rs:644`; `.github/workflows/ci.yml:3-12`, `:331`, `:335-336`; `.github/workflows/lavapipe-files.yml:9-10` |
| 0024 | "lavapipe's records ... build time, which a shared runner's load and cache hits move" | holds (CI half): the release build is timed and published beside the records | `.github/workflows/ci.yml:280-284`, `:352-353` |
| 0024 | "`verify` never publishes" | holds | `scripts/verify.sh:54` |
| 0025 | "everything in it is published under the MIT licence (`LICENSE`, `Cargo.toml`)" | holds | `LICENSE:1`; `Cargo.toml:5` |
| 0025 | "Anything the owner would not publish, on any branch, issue or PR: a credential, a token" | holds for the workflows: no `secrets.` is read anywhere; the one token, `github.token`, reaches `gh` through the environment only | `.github/workflows/pr-lint.yml:43-61` |
| 0025 | "A Bench record and a PR's images are published as a commit is" | holds: every record is pushed to the public `bench-history`, and in CI only `publish-bench` writes, only on a push to `main`; what a record names is a note for the owner (Notes) | `src/bench/history.rs:726-735`; `.github/workflows/ci.yml:329-353` |
| 0025 | "Every file under `assets/` has a manifest entry and a credit, which `cargo test` checks" | holds, with planted cases | `tests/asset_manifest.rs:97-106`, `:194-196`, `:209-214` |
| 0025 | "a larger source is fetched from its publisher into a git-ignored cache" | holds: the helmet's six files are `fetched: true` and `/assets/SciFiHelmet/` is ignored | `assets/manifest.json:3-66`; `.gitignore:17` |
| 0025 | "a file under a noncommercial or restrictive term is dev-only: never committed, and no committed Golden is rendered from it" | holds today: every manifest entry is `CC0-1.0`, none `dev_only`; the test enforces only `-NC-` → `dev_only` (Notes) | `tests/asset_manifest.rs:190-193` |
| 0025, checklist | "Adapted code cites its source and licence" | holds for the Cycles reference: the GT7 port names its source, key and the MIT notice | `scripts/cycles-reference/compare.py:36-60` |
| 0025 | "a vendored crate keeps its licence files (`vendor/README.md`)" | holds: `LICENSE.MIT` and `LICENSE.APACHE` beside `vendor/wgpu-core` | `vendor/README.md:22-23` |
| 0025 | "every dependency's licence expression is on the allowlist in `tests/dependency_licences.rs`" | holds: `cargo metadata --locked` over every package; the one expression added since, "BSD-3-Clause AND BSD-2-Clause", is permissive | `tests/dependency_licences.rs:8-34`, `:37-55`, `:74-89` |
| 0025 | "Self-hosted Actions runners on the owner's machines ... CI runs on GitHub's hosted runners, without a GPU, and local scheduled tasks bench the real ones" | holds for the runners: every `runs-on` is `ubuntu-24.04`, `ubuntu-latest` or `windows-2025`, the new workflow included, and the sweep is a launchd job; the plural "scheduled tasks" is L1-T09 | `.github/workflows/ci.yml:60`, `:103`, `:133`, `:160`, `:204`, `:261`, `:333`; `.github/workflows/lavapipe-files.yml:23`, `:69`; `.github/workflows/pr-lint.yml:26` |
| checklist | "Workflows keep least-privilege permissions and pin each action to a commit SHA with its version in a comment, as `pr-lint.yml` does" | holds: every workflow defaults to `contents: read` (with `pull-requests: read` for the lint), `contents: write` only in `publish-bench`; all 31 `uses:` lines are SHA-pinned, the four versioned pins resolve to their commented tags, and the seven `dtolnay/rust-toolchain` pins carry `# stable`, a branch (Notes) | `.github/workflows/ci.yml:37-38`, `:335-336`; `.github/workflows/lavapipe-files.yml:9-10`; `.github/workflows/pr-lint.yml:14-16`, `:29` |
| checklist | "Frame stats still see new work: GPU buffers, textures and acceleration structures are created through the GPU memory ledger (`GpuMemory`) under their kind ...; the GPU stats readback's buffers stay out ...; per-frame GPU writes add to `uploaded_bytes`, and systems are added inside `SystemTimer::scope`" | holds for the ledger and `SystemTimer` these files define, and every Frame stats CPU stage reaches the bench's `cpu_time_microseconds` by name; `uploaded_bytes` misses the static TLAS's staged records (L1-R02), and its doc is L1-T05 | `src/stats/gpu_memory.rs:109-226`; `src/stats/mod.rs:338-363`; `src/state.rs:287`; `src/bench/mod.rs:1022-1035` |
| checklist | "Every name the change renames, removes or changes the behaviour of ... is grepped in comments, ADRs, `CONTEXT.md` ...; each clause the change makes false ... is rewritten or dated in the same PR" | broken for the RTX 2070 SUPER sweep's removal: L1-T08, L1-T09; the glossary's Night, ADR 0024 and the bench-history guide (stale by rule) were updated | `docs/agents/project-shape.md:24-27`; `docs/adr/0025-the-repo-is-public-under-mit.md:17` |
| checklist | "Outside `src/render/`, no `unwrap()` on a GPU operation that could fail at runtime" | holds: capture's polls, map and mapped range go through `?`; the ledger and the CLI unwrap no GPU operation | `src/capture/mod.rs:373-380`, `:492-500`, `:511-518` |
| conventions | "never `println!` or `log` (tested)" | holds | `tests/conventions.rs:156-168` |
| conventions | "Identifiers are full words, WGSL included (... shading names tested)" | holds: now over `src/` and `tests/`, with suffixed spellings and tangent and bitangent initials; red-capable | `tests/conventions.rs:322-367`; tests `:370-383` |
| conventions | "Comments ... citing no ticket, PR or decision ... (ticket numbers tested)" | holds: `#n`, `PR n`, `issue n`, `issues/n` and `pull/n` links outside upstream repositories, now in scripts, workflows, `Cargo.toml` and `.gitignore` with `#` comments; red-capable | `tests/conventions.rs:48-58`, `:76-97`, `:176-232`, `:272-294`; tests `:296-321` |
| workflow, Session start | preflight "prints ... in ten lines or fewer: the adapter and which machine it is, whether its Goldens, Baselines, fetched files and Cooked cache are present, GPU load, the one-minute load average, other praxis processes and any other process at half a core or more, the open PRs and the live milestone's assigned tickets" | holds: 10 lines on macOS, 9 on Linux and on Windows; the live milestone now parses `M<n>.<m>` (M3.5's L1-T05 fixed, tested with `M3.5`, `M3.9`, `M3.10`) | `src/preflight/mod.rs:112-148`, `:699-767`; test `:964-971` |
| workflow, Session start | "A `FAIL` line blocks `scripts/verify.sh` and names its fix" | holds: each `Fail` names its fix, the assets check's included now (the last audit's note), and the binary exits non-zero on any; each `FAIL` (no adapter, Goldens or Baselines absent) fails verify through the Goldens test under `PRAXIS_REQUIRE_GOLDENS=1` or through `compare` | `src/preflight/mod.rs:139-143`, `:196-206`, `:230-242`, `:253-257`; `src/bin/praxis.rs:467-482`; `scripts/verify.sh:55`, `:63` |
| workflow, PRs | lavapipe's files: "`scripts/lavapipe-files.sh` has CI record them in a run that judges nothing and copies into the tree the Goldens and Baselines CI would fail, and the Baselines a judged counter fell below" | broken: L1-T01 (a LOD 0 Golden the recording removes fails the run); the Baselines half holds (`compare --recording` fails on any judged counter that differs) | `.github/workflows/lavapipe-files.yml:42-65`, `:93-105`; `src/bench/mod.rs:1193-1196`; `scripts/lavapipe-files.sh:44-63` |
| workflow, PRs | "CI never commits them: a PR that moves one without committing it fails, but for a Baseline's counter that only fell, which CI passes" | holds: `compare --ci` fails only `current > baseline`; no workflow commits to the PR | `src/bench/mod.rs:1195`; `.github/workflows/lavapipe-files.yml:9-10` |
| workflow, PRs | verify: "formatting, lint, build, tests (Goldens required) and the bench counters against Baselines and Budgets, one line per step" | holds | `scripts/verify.sh:47-80` |
| workflow, ladder | "the verify command (`scripts/verify.sh`) exits 0 within 30 lines" | holds: 7 lines on success | `scripts/verify.sh:74-80` |
| workflow, PRs | "CI's PR form check (`scripts/pr-lint.sh`) fails a title, description or commit that breaks this section's rules or Review's counts and 'Sources checked' line, and a PR over the size ceiling" | holds: ticket line, template sections, counts, 30 pasted lines, Sources checked, images, size, subject length and attribution, the squash commit included | `scripts/pr-lint.sh:36-73`, `:88-114`, `:116-144`; `.github/workflows/pr-lint.yml:5-8`, `:42-61` |
| workflow, Tickets | "Size ceiling: 800 changed lines of code ... outside `docs/`, the root's `.md` files, Goldens, the lockfile, `bench/baselines/`, `bench/cycles/` and `vendor/`" | holds: the uncounted pattern lists the same seven; additions plus deletions from the API's `changes` | `scripts/pr-lint.sh:108-113`; `.github/workflows/pr-lint.yml:57-59` |
| workflow, PRs | images: "asks a PR that changes those paths for the three links when a Golden moves or a `.wgsl` file changes, and otherwise takes either the three or that line with its two counts equal" | holds; its example count is stale: L1-T11 | `scripts/pr-lint.sh:93-106` |
| workflow, PRs | "Commit subject 72 characters at most ... the title is 72 characters at most with that number, and the description carries no such line" | holds | `scripts/pr-lint.sh:116-135` |
| workflow, PRs | field guides: a stale list that fails nothing; links and images; the Describes commit an ancestor of `main`; code-font identifiers on `main`; no ticket or PR in prose; the Files row's files exist | holds clause by clause (Notes on the identifier set) | `tests/field_guides.rs:33-99`, `:205-236`, `:261-269`, `:275-300`, `:351-376`, `:387-436` |
| workflow, Review | "`cargo test --test references` fails on a dangling key or an unused entry" | broken: L1-T04 (the Python scripts are not read) | `tests/references.rs:235-237` |
| workflow, PRs and Correctness | gallery: "prints for each image how many pixels differ from the base's and the largest channel delta"; "`town` at the Goldens test's eight frames, `world` at its four Goldens' frames, `crowd` in both shadow forms" | holds: the frames equal `TOWN_FRAMES` and `WORLD_FRAMES` | `scripts/gallery.sh:42-44`, `:136-139`, `:147-159`; `tests/goldens.rs:313`, `:319`, `:373` |
| workflow, Performance | "The sweep files a Regression only for a commit that did not re-record its scene's Baseline for the Series' adapter and backend" | holds | `src/bench/history/verdicts.rs:396-427`; `src/bench/mod.rs:146-152` |
| `ci.yml`'s own rule | "Each entry is one of `changes`' `skipped_paths`, in the same order" | holds, now tested | `.github/workflows/ci.yml:10`, `:13-28`, `:77-93`; `tests/conventions.rs:595-612` |
| `ci.yml`'s own rule | "A path may skip only if no engine job reads it, or only tests `check` runs" | holds: of the skipped paths only `check`'s tests read any (conventions, references, field guides); the GPU test's `display-ramp.txt` is not skipped | `.github/workflows/ci.yml:50-56`; `src/render/shader_function_tests.rs:1163` |
| glossary | "Cycles Baseline ... The gallery with `PRAXIS_BLENDER` fails a scene whose run exceeds either figure by its tolerance" | holds: judged at the Goldens' frame, scene, frame and size checked, a FAIL line and exit 1 at the end; the tolerances are global, not per scene (Notes) | `scripts/gallery.sh:94-115`, `:160`; `scripts/cycles-reference/compare.py:85-92`, `:238-258` |

### Spec stories

| Story | Verdict | Where |
|---|---|---|
| M4 story 18 (captures and Goldens render Settled frames) | holds on capture's side: every `capture` and `capture_frames` frame is settled, and a failed load fails the render | `src/capture/mod.rs:160-177`, `:266-283`, `:383-384` |
| M4 story 22 (a Late Chunk counted in Frame stats) | holds: a Frame stats field, summed over the bench's frames; on `world` compared on the Settled counter frames, so its measured frames' Late Chunks are reported, not judged | `src/stats/mod.rs:89-90`; `src/bench/mod.rs:319`, `:376`, `:1448-1465`; `src/bench/report.rs:239` |
| M4 story 25 (fast travel holds until the Load radius is loaded, every frame of the hold within Budget) | holds on capture's side: the camera path pauses while the hold lasts, held frames drawn and counted, bounded at 3,600 frames, tested on a jump away and back and on a jump in an undrawn frame; in the bench the hold's frames are measured frames, whose maximum `compare` judges against 16,667 µs, the 6 ms Budget being judged at the median, p95 and p99, not per frame, a per-frame clause the M4 exit accepted failing | `src/capture/mod.rs:354-394`, `:412-413`, tests `:660-751`; `src/bench/mod.rs:1274-1281`, `:1312-1318` |
| M4 story 26 (the time from a Fast travel until its Chunks load measured and kept as a ceiling) | holds as a measurement: `fast_travel_hold` in Frame stats, the Baseline and the report; the 50 ms ceiling is a milestone criterion no command judges (Notes) | `src/stats/mod.rs:116-117`; `src/bench/mod.rs:584-592`, `:1055`; `src/bench/report.rs:247-256` |
| M4 story 39 (late mips counted and reported by the bench, not gated) | holds | `src/stats/mod.rs:110-112`; `src/bench/mod.rs:331`, `:382`, `:395`, test `:1961`; `src/bench/report.rs:245` |
| M4 stories 41 and 42 (the 8 GB split into Budgets per kind, their sizes decided) | holds | `src/stats/gpu_memory.rs:9-66`; `bench/budgets.json:10-18` |
| M4 story 44 (GPU memory counted by kind where it is created, so that no buffer or texture escapes `gpu_memory_bytes`, each kind compared against its Budget) | holds for everything created through the ledger, and each kind is judged; capture's target and read-back buffer stay out as ADR 0013 now records; the load staging escapes (L1-R04) | `src/stats/gpu_memory.rs:140-186`; `src/capture/mod.rs:332-345`, `:465-470`; `src/bench/mod.rs:1036-1047`, `:1233-1238`, test `:2167` |
| M4 story 45 (acceleration-structure bytes counted exactly once M4 can read their sizes) | holds for TLASes (the driver's size plus wgpu-core's Instance buffer); a compacted BLAS stays an estimate, as ADR 0017 records | `src/stats/gpu_memory.rs:188-209`; test `:339-374` |
| M4 story 47 (a request past any capacity refused, logged and counted in Frame stats) | holds at the seam, a Frame stats field per refusal kind, and in the bench for its eight refusal counters, failed on any rise; Meshes and Materials refused and Chunks not loaded reach the traversal only (Notes); the mip move's refusal is L1-F02 | `src/stats/mod.rs:39-46`, `:56-61`, `:104-109`; `src/bench/mod.rs:397-408`, `:1208-1226`, tests `:1975`, `:3094` |
| M4 story 51 (the frames until a newly loaded Chunk casts shadows measured and kept as a ceiling) | holds as a measurement, tested (pbr-sweep's Chunk traced from frame 1), and `frames_until_traced` reports the most over the measured frames; the ceiling N = 10 is a milestone criterion (Notes) | `src/stats/mod.rs:51-55`, `:231-233`; `tests/frame_stats.rs:577-612`; `src/bench/mod.rs:1057-1059`; `src/bench/report.rs:263-267` |
| M4 story 53 (waits on BLAS compaction measured) | holds as a measurement: a CPU stage of its own, spread over the frames that waited | `src/bench/mod.rs:1060-1067`; `src/bench/report.rs:116-120`, `:273-276` |
| M4 story 61 (a camera path of about 60 s for the bench) | holds (bench half): the reference configuration runs the world's whole `BENCH_SEGMENT_FRAMES`; the path falls in scenes | `src/bench/mod.rs:100-123`; test `:1612` |
| M4 story 62 (a full traversal with p99, maximum, Late Chunks and peak GPU memory pasted) | holds: `praxis traverse` measures and prints them; the route falls in scenes | `src/bin/praxis.rs:411`, `:859-866`; `src/bench/mod.rs:614-688`, `:836-863`, test `:2945`; `src/bench/report.rs:321-397` |
| M4 story 63 (CI cooks only what its frames load, runs the world's Settled Goldens at a few frames and its counters on a short stretch) | holds: four `world` Golden frames; `praxis cook` cooks every scene but `world`, the dev Goldens cook `world` on a miss, the shipping job cooks it through the Goldens test's two capture routes; counters on `CI_COUNTERS`' Settled frames through `--ci-counters` | `tests/goldens.rs:315-328`; `.github/workflows/ci.yml:181-187`, `:231-247`, `:291-296`; `src/bin/praxis.rs:492-497`; `src/bench/mod.rs:93-98`, `:131-137` |
| M4 story 65 (the same Chunk content at the origin and at the far corner gives identical images) | holds through the 100 km check, which shifts every Golden scene, `world` included, by whole anchor cells and allows 0 pixels | `tests/goldens.rs:58-64`, `:197`, `:225-231` |
| M4 story 66 (nothing drawn past the Load radius, the pop recorded as a known difference until M5) | superseded by M5 story 18: `DIFFERENCES.md` holds the Far proxies' difference instead of the pop | `scripts/cycles-reference/DIFFERENCES.md:174-191` |
| M4 story 67 (p99 and the maximum beside the median and p95, with a Harness version bump) | holds: nearest rank, 10 to 11 at that change; a Series row carries each Spread whole for the CPU frame, GPU frame, gate and CPU times | `src/bench/mod.rs:690-701`, `:1560-1575`; `src/bench/history.rs:498-504`, `:1200-1212` |
| M4 story 68 (`compare` judges the p99 against the CPU, GPU and gate Budgets and the maximum against 16.7 ms on the world scene) | holds, on every scene, not only `world` | `src/bench/mod.rs:1270-1334` |
| M4 story 69 (Late Chunks, late mips, Chunks loaded and unloaded, bytes read and pool entries freed in Frame stats and every Bench record) | holds: Frame stats fields, bench counters, and the Series row names them through `Counters::NAMES` | `src/stats/mod.rs:89-112`; `src/bench/mod.rs:319-331`, `:376-382`, `:427-433`; `src/bench/history.rs:1215-1228` |
| M4 story 70 (the static TLAS's build time reported apart from the frame's critical path) | holds in Frame stats, printed only when non-zero and tested; the bench's `Measurements` do not carry it (Notes) | `src/stats/mod.rs:47-50`, `:224-230`; `tests/frame_stats.rs:602-605` |
| M4 story 71 (GPU memory a running total updated at load and eviction, costing nothing per frame) | holds: seven relaxed atomic loads per read | `src/stats/gpu_memory.rs:222-225`, `:234-249` |
| M4 spec, testing decision: "every scene's Goldens on Metal and software Vulkan through the Chunk path" | holds (CI half) | `.github/workflows/ci.yml:156-195`; `tests/goldens/vulkan-cpu/` (23 files) |
| M5 story 10 (a Render switch forcing LOD 0 and one forcing a given LOD) | holds in the CLI and tests: `--lod <n>` on `capture` and on a bench with `--no-publish`; the Goldens check every Golden frame at LOD 0 forced | `src/bin/praxis.rs:213-218`, `:269-276`; `tests/goldens.rs:198`, `:232-241`, `:356-362` |
| M5 story 14 (the shadow mask pixels that differ from LOD 0 counted and limited) | holds for the count (`praxis shadow-mask-difference`, tested on four words); no code holds a limit (Notes) | `src/capture/mod.rs:91-130`, `:208-239`, test `:773-795`; `src/bin/praxis.rs:975-1009` |
| M5 story 15 (the weights and threshold tuned on the gallery) | holds (tooling half): the gallery captures every Reference scene at 2560x1440, `world` at its four Golden frames | `scripts/gallery.sh:147-159` |
| M5 story 23 (a Late Far proxy counted) | holds | `src/stats/mod.rs:91-93`; `src/bench/mod.rs:321`, `:377`; `src/bench/report.rs:240-244` |
| M5 story 29 (shadow-ray time reported with Far proxies traced and not) | not in tooling as a standing measure: it was measured once with binaries built apart, in the change that traced Far proxies; no Render switch or CLI mode untraces them (Notes) | `src/render/renderer.rs:78-115` |
| M5 story 31 (the far field's fill after a Fast travel measured and kept as a ceiling) | holds as a measurement: `far_field_filling` and `far_field_fill`, with a Harness version bump; the ceiling (80 frames, 650 ms) is a milestone criterion (Notes) | `src/stats/mod.rs:118-123`; `src/bench/mod.rs:594-599`, `:1056`; `src/bench/report.rs:257-262` |
| M5 story 32 (the full traversal reports Late Far proxies beside Late Chunks) | holds | `src/bench/mod.rs:624`, `:665`; `src/bench/report.rs:336-337` |
| M5 story 40 (the Instance slots in use in the Bench record, the per-slot stages judged against 1 ms) | holds; `culling_per_slot_microseconds` stays in the record without a Series column | `src/stats/mod.rs:32-34`; `src/bench/mod.rs:334-336`, `:1010-1020`, `:1349-1361`; `src/bench/history.rs:1222-1228`; `bench/budgets.json:7` |
| M5 story 42 (the RTX 2070 SUPER's Baselines re-recorded at the exit, reported and not gated) | holds: its two directories exist, a re-recorded RTX Baseline becomes a record on Vulkan and DX12 keys, and no sweep runs there; `compare` would judge Budgets on it if run Locally (the last audit's note) | `bench/baselines/nvidia-geforce-rtx-2070-super/`, `bench/baselines/nvidia-geforce-rtx-2070-super-dx12/`; `src/bench/history/import.rs:194-220`, `:399-409` |
| M5 spec, testing decisions: "LOD 0 stays bit-identical ...: Goldens bit-exact"; "Captures and Goldens render Settled frames" | holds in the test: each Golden frame is compared at LOD 0 forced with its Golden, or its `-lod-0` twin where selection changes the image, at 0 pixels; capture settles; the Far proxies' wait falls in streaming | `tests/goldens.rs:198-242`, `:499-515` |
| M5 spec decision: "each PR carries gallery before, after and difference images" for selection, the swap and widening | holds: such a PR moves Goldens or a `.wgsl` file, and the lint then asks for all three | `scripts/pr-lint.sh:101-102` |
| M4 stories 24, 35 to 38, 48 to 50, 52, 54 and 55; M5 stories 18 to 22, 24 to 28 and 30 | fall in streaming, render, assets or cook; nothing in tooling implements them | — |

### Registry keys

| File | Keys | Verdict |
|---|---|---|
| `src/bench/mod.rs:493` | `khronos-dfs-srgb-eotf` | matches: quote "R = R′ / 12.92, R′ ≤ 0.04045; ((R′ + 0.055) / 1.055)^2.4, R′ > 0.04045"; `encoded <= 0.04045` gives `encoded / 12.92`, else `((encoded + 0.055) / 1.055).powf(2.4)` (`:494-501`), the knee's ≤ as quoted |
| `src/bench/history/night.rs:94` | `git-merge-base-is-ancestor` | matches: "exit with status 0 if true, or with status 1 if not. Errors are signaled by a non-zero status that is not 1"; `is_ancestor` maps 0 to true, 1 to false and anything else to an error (`:104-108`) |
| `src/stats/mod.rs:77` | `wgpu-hal-30-allocator-report-default` | matches: the quote's default returns `None`; wgpu-hal 30.0.2, which `Cargo.lock` pins, has the same lines at `lib.rs:1210-1212`; the field is an `Option` printed as `none` |
| `src/capture/mod.rs:441` | `apple-tech-talk-110147-busy-waits` | matches: the quote ("Avoid busy waits ... lock a P core ... thermal headroom") is why only the bench spins; `capture` and `read_frames` pass `FrameWait::Block` (`:175`, `:281`) |
| `src/preflight/mod.rs:478` | `microsoft-query-user-notification-state` | matches: 2 (`QUNS_BUSY`) and 3 (`QUNS_RUNNING_D3D_FULL_SCREEN`) warn, 1 to 7 pass, anything else is a note (`:480-492`), tested (`:895-903`) |
| `scripts/sweep.sh:24` | `apple-launchd-start-calendar-interval-sleep` | matches: "your job will run when the computer wakes up"; the comment says "04:00 local, or on the next wake when asleep then", and the plist sets `StartCalendarInterval` 4:00 (`scripts/sweep.plist:17-23`) |
| `scripts/sweep.sh:148` | `gnu-bash-news-4.0-mapfile` | matches: mapfile is new in 4.0, so the `/bin/bash` 3.2 the plist runs (`scripts/sweep.plist:10`) lacks it; the job's three lines are read with `read -r` (`:160-164`), the comment sitting above the night's directory instead (Notes) |
| `scripts/cycles-reference/compare.py:39` | `gt7-tone-mapping-cpp` | matches: the quote's 250 over 100 cd/m² is `GT7_PAPER_WHITE = 2.5` (`:63`); `initializeCurve(..., 0.25f, 0.538f, 0.444f, 1.280f)` is `:101`; the 100 cd/m² reference is `100.0 / 10000.0` (`:114`) |
| `scripts/cycles-reference/compare.py:155` (new) | `khronos-dfs-srgb-eotf` | matches: `np.where(clamped <= 0.04045, clamped / 12.92, np.power((clamped + 0.055) / 1.055, 2.4))` (`:157`), the knee's ≤ as quoted |
| `scripts/cycles-reference/render.py:99` | `khr-lights-punctual-range` | matches: quote `max( min( 1.0 - ( current_distance / range )⁴, 1 ), 0 ) / current_distance²`; the node tree is `1 − (Ray Length / range)⁴` clamped (`:108-121`), Cycles' point light giving `1/d²`; `calibrate.py:72-74` checks 0.9375 |
| `scripts/cycles-reference/render.py:303` | `gltf-2.0-metallic-roughness` | matches: green to Roughness, blue to Metallic, each times its factor (`:304-308`, `:315`) |
| `scripts/cycles-reference/render.py:312` | `gltf-2.0-zero-roughness` | matches: the clamp path, `MAXIMUM` with `MINIMUM_ROUGHNESS` (`:313-317`) |

No other file in tooling cites a key: `src/bench/report.rs`, `src/bench/provenance.rs`, the rest of `src/bench/history/`, `src/stats/gpu_memory.rs`, `src/bin/praxis.rs`, the other scripts, the workflows and the tests cite none (`tests/references.rs` holds only planted fixtures). The wgpu-hal quote was opened in the 30.0.2 sources.

### Findings

| id | claim | severity | where | evidence | expected | verifier | placement |
|---|---|---|---|---|---|---|---|
| L1-T01 | The recording workflow fails, and copies nothing, whenever the Goldens update removes a lavapipe LOD 0 Golden: `git ls-files --modified` lists the deleted file, `praxis difference` cannot open it, and `cp --parents` of the absent path fails the step | correctness: the documented route for lavapipe Goldens fails on every change that makes one of the 9 `vulkan-cpu` LOD 0 Goldens redundant, and the removal is left to be found and made by hand | `.github/workflows/lavapipe-files.yml:49-58` | block below | `docs/agents/workflow.md:29`: `scripts/lavapipe-files.sh` "has CI record them in a run that judges nothing and copies into the tree the Goldens and Baselines CI would fail" | `confirmed`: The step pipes `git ls-files --modified --others --exclude-standard tests/goldens` into a loop that ends in `cp --parents "$golden" target/lavapipe-files` (`.github/workflows/lavapipe-files.yml:49-58`) under `shell: bash` (`:15-17`, run as `bash -eo pipefail`), and `update_golden` removes a LOD 0 Golden that no longer differs (`tests/goldens.rs:511-512`, `:521-523`); a deleted tracked file is listed as modified, so `cp` of the absent path fails the step and `lavapipe-files.sh`'s `gh run watch ... --exit-status` (`scripts/lavapipe-files.sh:48`) stops before any download, against `docs/agents/workflow.md:29`'s "copies into the tree the Goldens and Baselines CI would fail". | fix #1031 |
| L1-T02 | The import makes a re-recorded Baseline's record clean and judges it quiet by its GPU frame spread alone, ignoring the `dirty` and `quiet` stamps the Baseline's own provenance holds, so the M3.5 exit's M4 Pro Baselines, measured from a dirty tree, sit on their Series as clean records of a source they did not measure | correctness: clause broken; 10 M4 Pro Series records on `bench-history` (`daaba8ce`'s Baselines but texture-wall, which its spread tags); no record today whose Baseline stamped `quiet: false` with a spread under 1.06 | `src/bench/history/import.rs:374-394` | block below | ADR 0024:17: "A record joins a Series only if its tree was clean, its profile release, the machine quiet (preflight warned of none of ...)" | `confirmed`: `imported` calls `tags(false, true, ...)` (`src/bench/history/import.rs:381-383`), `false` being `dirty` (`src/bench/history.rs:880`), and replaces the Baseline's provenance with `ImportedProvenance` (`import.rs:362-373`), so `git show daaba8ce:bench/baselines/apple-m4-pro/town.json`'s `"dirty": true` (line 136) never reaches a tag; `origin/bench-history` at `472a27c6` holds the 10 untagged records under `records/apple-m4-pro-metal/*/4da82f0c….json`, texture-wall's under `tagged/`; against ADR 0024's "A record joins a Series only if its tree was clean" (`docs/adr/0024-...md:17`). Not a re-find: the last audit's `L1-T02` asked that re-recorded Baselines be marked at all. | fix #1030 |
| L1-T03 | A re-recorded Baseline's record drops the scene content hash its Baseline holds, since the import replaces the Baseline's provenance with one that has no such field | correctness: clause broken for 164 records on `bench-history` (128 of them the M4 Pro's) and 306 import candidates at the frozen commit, every one holding a hash; a content change next to a re-recorded Baseline draws without a line break | `src/bench/history/import.rs:323-335` | block below | ADR 0024:18: "each record carries its scene's content hash. A chart breaks its line where either changes" | `confirmed`: `ImportedProvenance` (`src/bench/history/import.rs:325-335`) has no `scene_content_hash`, so a re-recorded Baseline that holds one (`daaba8ce`'s `town.json` line 141) is stored with `scene_content_hash: None` (`src/bench/history.rs:286-287`), which `same_definition` lets join either neighbour (`:1238-1243`); the `town` record at `472a27c6` holds no such key; against ADR 0024's "each record carries its scene's content hash. A chart breaks its line where either changes" (`docs/adr/0024-...md:18`). | fix #1030 |
| L1-T04 | `tests/references.rs` reads no Python file, so a dangling key cited in the Cycles reference scripts passes the test that the workflow says fails on one | correctness: a check not red-capable for 5 citations in 2 files, one of them added since the last audit; none dangles today | `tests/references.rs:235-237` | block below | `docs/agents/workflow.md:43`: "`cargo test --test references` fails on a dangling key or an unused entry"; `tests/references.rs:215`: "The text files that may cite the registry: code, shaders, scripts and Markdown" | `confirmed`: `let is_text = ["rs", "wgsl", "md", "sh", "toml"]` (`tests/references.rs:235-237`) leaves out `.py`, and a `git grep` of `*.py` files for the bracketed citation form lists five citations in `scripts/cycles-reference/compare.py` and `render.py`, so a dangling key there passes the test that `docs/agents/workflow.md:43` says "fails on a dangling key"; none dangles today. | fix #1031 |
| L1-T05 | `uploaded_bytes`'s doc says "The pools' own loads are not counted" and lists what is counted without them, while every frame adds the bytes its streamed Chunk loads and mips write into the pools through the load staging, which the Frame stats test pins | drift: load-bearing for reading the counter, which `compare` judges against each Baseline and the Bench history publishes; in a streaming frame the undescribed part is up to `load_bytes_per_frame`, 26,640,177 bytes at the checked-in Budgets, and uncapped in a Settled frame | `src/stats/mod.rs:25-31` | block below | the doc names the streamed Chunk loads and mips among what is counted, and says only loads outside the Chunk streamer (a spawn's) are not | `confirmed`: The doc ends "The pools' own loads are not counted" (`src/stats/mod.rs:30-31`) while `State` adds `self.recorder.counters.uploaded_bytes += streamed.written_bytes` (`src/state.rs:287`), "Bytes the frame's loads wrote into the load staging" (`src/streaming/io.rs:652-653`), the Chunk loads' and mips' writes into the pools; `drift`. | fix #1029 |
| L1-T06 | The note `praxis report` prints under its times table says the Budget column bounds the median and p99, but `compare` judges the two dynamic TLAS rows' Budgets at the median alone | drift: a sentence printed in every report pasted into a PR misstates the rule for 2 of the 5 rows that show a Budget (`Dynamic TLAS, CPU` at 1,100 µs, `dynamic_tlas_build` at 1,000 µs) | `src/bench/report.rs:149-153` | block below | the note names the statistics `compare` judges per row: median and p99 for the CPU frame, GPU frame and gate (the CPU frame's p95 too), the median alone for the dynamic TLAS's ceilings (`src/bench/mod.rs:1335-1348`) | `confirmed`: The report prints "The Budget column bounds the median and p99" (`src/bench/report.rs:151`) over rows that include `Dynamic TLAS, CPU` and `dynamic_tlas_build` with a Budget, while `compare` judges both at `.median` alone (`src/bench/mod.rs:1335-1348`) and its own doc limits the p99 to the CPU frame, GPU frame and gate (`:1172-1174`); `drift`. | fix #1034 |
| L1-T07 | `compare`'s doc says every median counts against its Baseline but the dynamic TLAS's CPU median, yet the lit pass's wait, the gate's passes, each system's and stage's CPU time and the BLAS compaction waits are compared with no Baseline | drift: not load-bearing; a reader expects a per-system CPU or lit-pass-wait regression to fail `compare` | `src/bench/mod.rs:1169-1173` | block below | the doc names the medians `compare` judges against Baselines: the CPU frame, the GPU frame, each pass, the gate, shadow-ray time (and aliasing) | `confirmed`: `compare`'s doc says "Medians count against Baselines, the GPU frame's and each pass's included, except the dynamic TLAS's CPU median" (`src/bench/mod.rs:1169-1171`), but nothing in `compare` (`:1184-1447`) reads `lit_pass_wait_microseconds`, `gate_pass_microseconds`, `cpu_time_microseconds` or `blas_compaction_microseconds` (`:225`, `:231`, `:244`, `:265`); `drift`. | fix #1034 |
| L1-T08 | `docs/agents/project-shape.md` says `history rerecord` is run by "a sweep's runs", but no sweep runs it since the RTX 2070 SUPER's sweep stopped | drift: not load-bearing; misleads a reader of the project map about who re-records counters | `docs/agents/project-shape.md:24-27` | block below | the map names what the module does: a session runs `praxis history rerecord` on runs of `main` (`src/bench/history/rerecord.rs:1-6`) | `confirmed`: The map says "`praxis history rerecord`: a sweep's runs re-record the counters" (`docs/agents/project-shape.md:24-27`), while no script names `rerecord` (`grep rerecord scripts/*.sh scripts/*.plist` finds none) and the module says "runs of `main` re-record" (`src/bench/history/rerecord.rs:1-2`); `drift`. | fix #1034 |
| L1-T09 | ADR 0025 says local scheduled tasks bench the real GPUs, but since 2026-11-21 one scheduled task benches one machine and the RTX 2070 SUPER is benched by hand | drift: not load-bearing; one plural | `docs/adr/0025-the-repo-is-public-under-mit.md:17` | block below | ADR 0024:3 (amended 2026-11-21): "The RTX 2070 SUPER has no sweep ... its Goldens and Baselines are recorded by hand at each milestone's exit" | `confirmed`: ADR 0025 says "local scheduled tasks bench the real ones" (`docs/adr/0025-the-repo-is-public-under-mit.md:17`), while ADR 0024 since 2026-11-21 says "The RTX 2070 SUPER has no sweep ... its Goldens and Baselines are recorded by hand at each milestone's exit" (`docs/adr/0024-...md:3`); `drift`, one plural. | fix #1034 |
| L1-T10 | The Frame stats test spells the geometry Arenas' bytes as five summands of the shares before 2026-11-16 (34,225,520 for the vertex Arena), while the code sizes them from `[658, 130, 11, 133, 68]` thousandths (55,197,040 for the vertex Arena); the kind's total is 83,886,072 either way, so the test passes on summands that name no Arena the code builds | drift: not load-bearing (the assertion is the kind's total, which holds); each summand is off by up to 20,971,520 bytes | `tests/frame_stats.rs:117-122` | block below | the summands are the shares `ARENA_PER_MILLE` gives (`src/assets/pools.rs:19`), or the test states the kind's total alone | `confirmed`: The test sums `34_225_520 + 22_229_808 + 838_860 + 12_163_480 + 14_428_404` (`tests/frame_stats.rs:122`), while `ARENA_PER_MILLE` is `[658, 130, 11, 133, 68]` (`src/assets/pools.rs:19`), which gives 55,197,040 for the vertex Arena from 83,886,080 (`bench/budgets.json:11`, `pools.rs:82-85`); 34,225,520 is the 408 thousandths share; `drift`, the total holds. | fix #1034 |
| L1-T11 | The lint's example of the identical-images line, "22 of 22", no longer matches the gallery, which writes 23 images since `world` gained its fourth frame | drift: not load-bearing (the lint takes any equal pair); one example in three places | `scripts/pr-lint.sh:104` | block below | the example count equals what `scripts/gallery.sh` writes | `confirmed`: The lint's example says "22 of 22 gallery images" (`scripts/pr-lint.sh:104`), while `scripts/gallery.sh:147-159` captures 8 `town` frames, 9 scenes, 2 crowd forms and 4 `world` frames (`world_frames=(59 300 599 1500)`, `:44`): 23; `drift`. | fix #1034 |

#### L1-T01

```yaml
# .github/workflows/lavapipe-files.yml:42-59
      - name: Record the Goldens CI would fail
        env:
          PRAXIS_UPDATE_GOLDENS: "1"
          PRAXIS_REQUIRE_GOLDENS: "1"
        run: |
          cargo test --test goldens reference_scenes_match_their_goldens
          mkdir -p target/lavapipe-files
          git ls-files --modified --others --exclude-standard tests/goldens |
            while read -r golden; do
              if git show "HEAD:$golden" > target/committed-golden.png 2>/dev/null; then
                difference=$(RUST_LOG=error target/debug/praxis difference \
                  target/committed-golden.png "$golden" target/golden-difference.png || true)
                if [[ $difference == "0 pixels differ"* ]]; then
                  continue
                fi
              fi
              cp --parents "$golden" target/lavapipe-files
            done
```

```rust
// tests/goldens.rs:511-512 (update_goldens)
            let differs = lod_0.as_raw() != actual.as_raw();
            update_golden(directory, &lod_0_label(&label), differs.then_some(lod_0));
// tests/goldens.rs:519-523 (update_golden)
fn update_golden(directory: &Path, label: &str, actual: Option<&image::RgbaImage>) {
    let path = directory.join(format!("{label}.png"));
    let Some(actual) = actual else {
        if std::fs::remove_file(&path).is_ok() {
            writeln!(std::io::stderr(), "removed Golden {label}").ok();
```

`docs/agents/workflow.md:29`: "on a pushed branch at `main`'s head, `scripts/lavapipe-files.sh` has CI record them in a run that judges nothing and copies into the tree the Goldens and Baselines CI would fail". When a change makes a scene's LOD 0 capture equal its capture (LOD selection no longer changes that frame), the update removes `<label>-lod-0.png`. `git ls-files --modified` counts an unstaged deletion as a modification (confirmed in a scratch repository outside the tree on git 2.54.0). For that path `git show HEAD:` succeeds, `praxis difference` fails on the absent file (`src/bin/praxis.rs:1028-1033`), so `$difference` is empty, and `cp --parents` of the absent file exits non-zero. The step runs under the workflow's `shell: bash` (`.github/workflows/lavapipe-files.yml:15-17`), which GitHub runs as `bash --noprofile --norc -eo pipefail`, so the piped loop's failure ends the step; `upload-artifact` (`:61-65`) has no `if:` and is skipped, and `scripts/lavapipe-files.sh:49` (`gh run watch "$run_id" --exit-status`) exits under `set -e` before downloading either artifact, the Baselines' included. Even with the copy guarded, the script only copies (`scripts/lavapipe-files.sh:58-61`), so a removal never reaches the tree, while CI fails the stale LOD 0 Golden whenever it differs from the new LOD 0 capture (`tests/goldens.rs:232-241`). Magnitude: `tests/goldens/vulkan-cpu/` holds 9 LOD 0 Goldens among its 23 files. Not visible in images; it costs a failed recording run and a deletion by hand. Fix: in the loop, `[ -e "$golden" ] || { echo "$golden" >> target/lavapipe-files/removed; continue; }`, uploaded with the Goldens, and `lavapipe-files.sh` removing each path that list names.

#### L1-T02

```rust
// src/bench/history/import.rs:337-342
/// The Baseline `json` that `commit` committed at `path`, as a Bench
/// record of that commit's source: the file as it was, with the backend it
/// ran on and an imported provenance. Baselines were recorded on a quiet
/// machine, so only their own GPU frame spread can tag one as not quiet:
/// against each frame's neighbours where the Baseline recorded that, else
/// against the run's median.
```

```rust
// src/bench/history/import.rs:374-394
    let quiet = p95_over_neighbours
        .or(p95_over_median)
        .is_none_or(|ratio| ratio < QUIET_P95_OVER_MEDIAN);
    BenchRecord::new(
        file,
        &baseline.scene,
        &commit.source_hash,
        commit.committed_unix_seconds,
        tags(
            false,
            true,
            Some(Run {
                scene: &baseline.scene,
                adapter: &baseline.adapter,
                profile: &baseline.profile,
                quiet,
```

`tags`' first argument is `dirty` (`src/bench/history.rs:880-883`). The rule fits the Baselines imported from before the history, which carry no provenance. A Baseline re-recorded since carries the bench's own stamps (`checkout.dirty`, `quiet`), and `imported` replaces that provenance wholesale (`src/bench/history/import.rs:362-373`), so neither reaches `tags`. The 11 M4 Pro Baselines committed by `daaba8ce` (the M3.5 exit) hold `"dirty": true`, `"commit": "8a1bfc2c…"`, `"source_hash": "919be01e…"` (`git show daaba8ce:bench/baselines/apple-m4-pro/town.json`, lines 134-136). The import keys their records by `daaba8ce`'s source hash, `4da82f0c…`; `8a1bfc2c`'s own is `a51401c4…`, and the two commits differ in 10 files under `src/` (the three source hashes recomputed from `git ls-tree` without writing objects; the method reproduces the `3d64bb23…` that `99f46267`'s Baseline records). All 11 have GPU frame spreads under 1.06 but texture-wall. On `bench-history` (the local `origin/bench-history` at `472a27c6`, 2026-11-23, read as the record of what the import wrote), `records/apple-m4-pro-metal/<scene>/4da82f0c….json` exists for the other 10 scenes, each with `provenance.source` `baseline-rerecorded`, no `tags` and no trace of `dirty`; texture-wall's is under `tagged/` as "machine not quiet". ADR 0024:17 keeps a record off its Series unless "its tree was clean". The same path also drops a Baseline's `quiet: false` (load, busy processes, power) when its spread is under 1.06; every hardware Baseline committed since `3c6b9d92` with `quiet: false` (`6ebb45fb`, `f806b46c`, `d3f7d0a7`, `61acdc8d`, `fe39acef`, on `world` or `helmet`) also has a spread of 1.07 to 1.16, so it is tagged anyway and none is counted. Magnitude: 10 Series records claim a clean measurement of a source other than the one measured; each starts its line as a Baseline, so the error is in that record's level, not in a Suspect. Fix: in `imported`, read `provenance.checkout.dirty` and `provenance.quiet` when the Baseline holds them and pass them to `tags`, keeping the spread rule for Baselines without provenance, with a test of a dirty re-recorded Baseline expecting the `dirty tree` tag.

#### L1-T03

```rust
// src/bench/history/import.rs:323-335
#[derive(serde::Serialize)]
struct ImportedProvenance<'baseline> {
    source: &'static str,
    baseline: &'baseline str,
    commit: &'baseline str,
    source_hash: &'baseline str,
    harness_version: u32,
    gpu_frame_p95_over_median: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gpu_frame_p95_over_neighbours: Option<f64>,
}
```

ADR 0024:18: "The Harness version rises whenever a change alters what a metric means, and each record carries its scene's content hash. A chart breaks its line where either changes, so a new definition never reads as a Regression." Every Baseline file `main` committed since `3c6b9d92` that the import turns into a record (306 by `remeasured`, 137 of them the M4 Pro's) holds `provenance.scene_content_hash` (`daaba8ce`'s `town.json`, line 141). The import overwrites `provenance` with `ImportedProvenance` (`src/bench/history/import.rs:362-373`), which has no such field, so `StoredProvenance.scene_content_hash` reads `None` (`src/bench/history.rs:286-287`) and the Series row writes null. On `bench-history` at `472a27c6`, all 164 records whose `provenance.source` is `baseline-rerecorded` lack the hash, and every one of the 1,781 records a bench published holds it. The branch README says a row holds null for "a time, Aliasing figure or scene content hash it lacks" (`src/bench/history.rs:128-129`); these records do not lack it, the import drops it. `same_definition` lets a null hash join either neighbour (`src/bench/history.rs:1238-1243`), so a cooker or scene change that lands next to a re-recorded Baseline draws on one line, and the record after the Baseline does not start one. Suspects change little, since the Baseline starts its own line (`:400-401`). The test of a current Baseline's record checks eight metrics and not the hash (`src/bench/history/import.rs:576-599`). Fix: add `scene_content_hash: Option<&str>` to `ImportedProvenance`, taken from the Baseline's provenance when present, and assert it in that test.

#### L1-T04

```rust
// tests/references.rs:215-218
/// The text files that may cite the registry: code, shaders, scripts and
/// Markdown, outside hidden directories, build output and cooked data. The
/// registry itself and this file, whose planted citations are its tests,
/// are left out.
// tests/references.rs:235-237
        let is_text = ["rs", "wgsl", "md", "sh", "toml"]
            .iter()
            .any(|extension| path.extension().is_some_and(|found| found == *extension));
```

`docs/agents/workflow.md:43` (Review, Sources): "a new source joins the registry in the same diff (`cargo test --test references` fails on a dangling key or an unused entry)". The Cycles reference's Python scripts cite five keys: `scripts/cycles-reference/compare.py:39` `[ref:gt7-tone-mapping-cpp]`, `compare.py:155` `[ref:khronos-dfs-srgb-eotf]` (added since `8bc34eb1`), `render.py:99` `[ref:khr-lights-punctual-range]`, `render.py:303` `[ref:gltf-2.0-metallic-roughness]` and `render.py:312` `[ref:gltf-2.0-zero-roughness]`; a `git grep -n` for the bracketed citation form outside `.rs`, `.wgsl`, `.md`, `.sh`, `.toml` and `docs/guides/` lists exactly these. The `py` extension is not on the list, so a misspelled or removed key there passes. All five exist today, and each is also cited from a scanned file (`gt7.wgsl`, `src/bench/mod.rs`, `shading.wgsl`, `src/cook/export.rs`, `docs/research/source-audit.md`), which is why no entry reads as unused. The extension list is older than the last audit; what landed since is a new citation in a file the test does not read. Fix: add `"py"` to the list.

#### L1-T05

```rust
// src/stats/mod.rs:25-31
    /// Bytes the frame's uploads wrote to GPU buffers: the camera and light
    /// uniforms, the light buffer, the bends, the Instances it wrote, the
    /// Instance slot count when it changed, and the dynamic TLAS's instances
    /// its build uploads; and the bytes that the moves of Textures between
    /// Texture classes staged since the frame before copy. The pools' own
    /// loads are not counted.
    pub uploaded_bytes: u64,
```

```rust
// src/state.rs:287
        self.recorder.counters.uploaded_bytes += streamed.written_bytes;
```

`streamed.written_bytes` is "Bytes the frame's loads wrote into the load staging" (`src/streaming/io.rs:652-653`): each Chunk load adds what `load_into_pools` wrote (`src/streaming/io.rs:1330-1341`) and each streamed mip what `material_pool.stream_mips` wrote (`:1150-1163`), both loads into the Mesh and Material pools. The test pins it (`tests/frame_stats.rs:62-72`): "only the first also writes the ten 128-byte Instances and the 4-byte count of Instance slots in use, and streams the Chunks: the sphere Mesh's bytes in the Arenas, ..., its 220-byte Mesh table entry and ten 80-byte Materials", with `frames[0].uploaded_bytes` asserted equal to the uploads plus `streamed_bytes`. The doc's last sentence was written on 2026-11-06 (`2b207766`); the line in `src/state.rs` landed on 2026-11-08 (`02fcac89`, "uploaded_bytes counts what streamed loads write"), after `8bc34eb1`, and did not touch the doc. What the sentence still describes truly is a spawn's loads, which go through the pools outside the streamer and are added nowhere (the orrery test's frame 0, `tests/frame_stats.rs:502-516`, counts no Mesh bytes for its spheres). The counter is one of the bench's (`src/bench/mod.rs:357`, `:416`), and `compare` fails a scene whose counter moves (test `src/bench/mod.rs:2135-2145`). Magnitude: `load_bytes_per_frame` is 26,640,177 bytes at the checked-in Budgets (`src/render/renderer.rs:1095`), the most a frame's streamed loads write that is not Settled, by ADR 0017's cap; a Settled frame writes past it. Not visible in an image. The render section handed this lead to tooling. Fix: replace the last sentence with "and the bytes the frame's streamed Chunk loads and mips wrote into the load staging; a spawn's loads are not counted".

#### L1-T06

```rust
// src/bench/report.rs:149-153
    writeln!(
        report,
        "\nThe Budget column bounds the median and p99; the CPU and GPU frames' maximum answers to {} µs. The culling phases' per-slot stages answer to a guard, at the median alone.",
        grouped(budgets.maximum_frame_microseconds, 1)
    )?;
```

```rust
// src/bench/mod.rs:1335-1348
    check_budget(
        &mut findings,
        "dynamic_tlas_cpu_microseconds",
        current.dynamic_tlas_cpu_microseconds.median,
        budgets.dynamic_tlas_cpu_microseconds,
    );
    if let Some(build) = passes.get(DYNAMIC_TLAS_BUILD) {
        check_budget(
            &mut findings,
            "dynamic_tlas_gpu_microseconds",
            build.median,
            budgets.dynamic_tlas_gpu_microseconds,
        );
    }
```

The rows that show a Budget are built at `src/bench/report.rs:66-97` (`CPU frame`, `Dynamic TLAS, CPU`, `GPU frame`, `Gate, rays off`) and `:109-114` (the `dynamic_tlas_build` pass row with `budgets.dynamic_tlas_gpu_microseconds`). ADR 0013 names the p99 for three metrics only: "the CPU frame, the GPU frame and ADR 0019's gate answer to their Budgets at the p99 as at the median, the CPU frame at its p95 too"; the glossary's Budget entry likewise lists "the dynamic TLAS's CPU and GPU ms" with no statistic beside the three that carry "each at its median and p99" (`CONTEXT.md:303`). The report's test pins the sentence (`src/bench/report.rs:574`). `docs/agents/workflow.md` (Performance and memory) makes the report the one source of figures quoted in PRs: "Figures quoted from bench results, in a PR or `docs/performance.md`, are pasted from `praxis report <results directory>`". Visible in every pasted report: a reader of a crowd run whose dynamic TLAS p99 is over 1,100 µs would expect a failure `compare` does not give. Fix: "bounds the median, and the p99 of the CPU frame, GPU frame and gate (the CPU frame's p95 too)".

#### L1-T07

```rust
// src/bench/mod.rs:1169-1173
/// Compares `current` with its `baseline` and the Budgets. Medians count
/// against Baselines, the GPU frame's and each pass's included, except the
/// dynamic TLAS's CPU median; counters count too, but the late mips, which
/// depend on when the IO pool's reads land, and the TLAS build's
/// allocations, which the heap allocations already hold, are reported only.
```

`compare` calls `check_regression` on the CPU frame median (`src/bench/mod.rs:1263-1269`), each `gpu_pass_microseconds` median (`:1291-1300`), the GPU frame median (`:1303-1309`), the gate and shadow-ray medians (`:1321-1330`) and aliasing (`:1239`, `:1488-1491`). It never reads `lit_pass_wait_microseconds` (`:225`), `gate_pass_microseconds` (`:231`), `cpu_time_microseconds` (`:244`) or `blas_compaction_microseconds` (`:265`), each a median-bearing `Spread` the record holds; `culling_per_slot_microseconds` meets its guard only (`:1349-1361`), as the doc's later sentence says. The change that added `cpu_time_microseconds` says in its message "Nothing new is judged by compare", so the code is as intended and the doc overstates. Visible only to a reader of the doc. Fix: "The CPU frame's, the GPU frame's, each pass's, the gate's and shadow-ray time's medians count against Baselines".

#### L1-T08

```
docs/agents/project-shape.md:24-27
                                 history/rerecord.rs, `praxis history
                                 rerecord`: a sweep's runs re-record the
                                 counters of the Baselines they moved,
                                 timings kept, with the commits between
```

```rust
// src/bench/history/rerecord.rs:1-6
//! `praxis history rerecord`: runs of `main` re-record the
//! counters of this machine's Baselines where they moved. A re-recorded
//! Baseline keeps its timings and Harness version, which a person chooses,
//! so its record starts no line on its Series; where those timings no
//! longer compare with the run, the Baseline is left for a person to
//! re-record whole.
```

The project map's line was written when the RTX 2070 SUPER's sweep ran the re-record and opened a pull request with it (`605f40b7`). The commit that stopped that sweep (`372ce4a0`) removed the re-record from `scripts/sweep.sh`, says in its body "Its module doc no longer says a sweep runs it", and changed the map's `sweep.sh` entry (`docs/agents/project-shape.md:178-185`) but not this one. `scripts/sweep.sh` at the frozen commit calls `history import`, `unrecorded`, `bisect`, `publish-night`, `regressions` and `publish`, never `rerecord` (`:80-81`, `:152`, `:191`, `:200`, `:209`). The bench-history guide already says "A session runs `praxis history rerecord <directory>` on runs of `main` whose counters moved" (`docs/guides/bench-history.html:124`). Fix: "`praxis history rerecord`: runs of `main` re-record the counters of the Baselines they moved, timings kept, with the commits between".

#### L1-T09

```
docs/adr/0025-the-repo-is-public-under-mit.md:17
- Self-hosted Actions runners on the owner's machines, which a pull request from anyone could run code on (ADR 0024). CI runs on GitHub's hosted runners, without a GPU, and local scheduled tasks bench the real ones.
```

ADR 0025 was accepted on 2026-11-14, when the M4 Pro ran `sweep.plist` and the RTX 2070 SUPER ran `sweep-task.xml`. `372ce4a0` (2026-11-21) deleted `scripts/sweep-task.xml` and amended ADR 0024:3: "The RTX 2070 SUPER has no sweep: it was rarely on, so its sweep published on few days, and its Goldens and Baselines are recorded by hand at each milestone's exit". ADR 0025 was not touched (its last commit is `300116fb`, 2026-11-14). Magnitude: one plural; the clause's point, that no self-hosted runner exists, holds. Fix: "and a local scheduled task benches the M4 Pro; the RTX 2070 SUPER is benched by hand".

#### L1-T10

```rust
// tests/frame_stats.rs:117-122
    // GPU memory by kind. The geometry Arenas: the Mesh pool's five, each its
    // share of the 80 MiB Budget in whole 4-byte words. ...
    let geometry_arenas = 34_225_520 + 22_229_808 + 838_860 + 12_163_480 + 14_428_404;
```

```rust
// src/assets/pools.rs:19
const ARENA_PER_MILLE: [u64; 5] = [658, 130, 11, 133, 68];
```

`src/assets/pools.rs:82-85` sizes each Arena as `budgets.geometry_arenas * per_mille / 1000` rounded down to 4 bytes. Recomputed from 83,886,080: 55,197,040, 10,905,188, 922,744, 11,156,848 and 5,704,252, summing to 83,886,072. The test's summands are those of 408, 265, 10, 145 and 172 thousandths, each rounded down to 4 bytes, the shares ADR 0017 records as in force from 2026-11-14 to 2026-11-16; they also sum to 83,886,072. The summands were written in `d3f7d0a7` (2026-11-14) and the shares changed in `308e7c67` (2026-11-16), both after `8bc34eb1`. The assertion reads the kind's total (`tests/frame_stats.rs:202-209`), so the test passes and cannot tell the Arenas apart; a reader takes the vertex Arena for 34,225,520 bytes, which ADR 0017 says it was "until those of 2026-11-16". Not visible. Fix: write the summands from the current shares, or state the total 83,886,072, the comment saying that the five Arenas' rounding loses 8 bytes of the Budget.

#### L1-T11

```bash
# scripts/pr-lint.sh:104
        miss "$linking, or gives the gallery's count on one line when no image differs, as \"22 of 22 gallery images identical to the base's\" (workflow.md, PRs)"
```

The gallery captures `town` at 8 frames (`scripts/gallery.sh:147-149`), 9 scenes at the Goldens' frame (`:150-153`), `crowd` in 2 forms (`:154-156`) and `world` at 4 frames (`:44`, `:157-159`): 23 images. The example was written at `888b0152` (2026-11-14), when `world_frames` was `(59 300 599)` and the gallery wrote 22; `b434f5e1` (2026-11-18) added frame 1500. The same "22 of 22" stands in `docs/agents/workflow.md:35` and the fixture `scripts/pr-lint-test.sh:210`. The lint takes any equal pair (`scripts/pr-lint.sh:103`), so no verdict changes; a reader copying the example writes a count the gallery never prints. Fix: 23 in the three places, or an example without a figure.

### Notes for the verifier

Not filed, since each needs a run, an owner's reading or another lane, or has no instance; recorded so the next audit does not find them again.

- Two counter changes kept their Harness version. `02fcac89` (2026-11-08) made `uploaded_bytes` count streamed loads (L1-T05's evidence) and did not raise `HARNESS_VERSION`, which next rose at `31fae9d1` (2026-11-14) for another reason; no Baseline changed in `02fcac89`, and the bench reads the median frame's counter (`src/bench/mod.rs:357`), so the Baselined scenes' medians did not move then. `4c65546d` moved acceleration-structure bytes from an estimate to wgpu-hal's exact size, re-recorded only that counter (−28,698 to −34,377 bytes on the M4 Pro) and kept version 11; the counter's meaning did not change, only its accuracy. Counters are judged by no Suspect. Run: on `bench-history`, compare `uploaded_bytes` across version 11 records whose sources straddle `02fcac89`; the owner says whether a counter's change of meaning or accuracy bumps the version.
- `bench/budgets.json` is compiled in (`src/bench/mod.rs:731-736`) and sizes the pools and the Renderer's capacities, yet `bench` is in `UNMEASURED_PATHS` under the comment "the bench reads none of them" (`src/bench/provenance.rs:408-419`). Not filed: a Budget change that moves a capacity breaks the pinned tests `the_checked_in_budgets_give_these_pool_capacities` (`src/assets/pools.rs:1412`) or `the_checked_in_budgets_give_these_render_capacities` (`src/render/renderer.rs:1088`), so the same commit edits `src/`; all four commits since `8bc34eb1` that touched the file also changed `src/`. The owner may want it listed as a measured path.
- A re-recorded Baseline's record is keyed by, and takes the Harness version of, the commit that landed it (`src/bench/history/import.rs:207-213`), as the branch README says (`src/bench/history.rs:130-132`), not the source and version its Baseline measured; about half the commits that re-recorded a Baseline since `3c6b9d92` landed another source than the one measured. One record's version differs: `0c133356` committed a `town` Baseline measured on `1cb7bb5b` under version 12, and its record says 13; `308e7c67`'s bump changed `world` only, so no line is wrong. The owner says whether a re-recorded Baseline should be keyed by its measured source; L1-T02 is the part that contradicts a clause.
- A tagged re-recorded Baseline loses its mark when a Series record holds its key (`src/bench/history.rs:1027-1034`, `:1074-1077`; the mark is carried only between records in one directory, `:1042-1060`), against the README's promise to "a quieter record of its source that holds the key in its place" (`:133-134`). No instance on `bench-history` at `472a27c6`.
- A held Verdict whose ends no longer show the shift judges nothing and stays held (`src/bench/history/verdicts.rs:236-241`), and every night resumes a held unpinned Verdict before any new Suspect (`:368-383`), so one shift that never reproduces would block every later Suspect on that machine. Liveness is the owner's call.
- The night's runs feed a Verdict whatever their own `provenance.quiet`: `may_start` waits only for preflight's `gpu load` and `cpu load` warnings (`scripts/sweep.sh:123-128`), and `night::runs` reads medians without the stamp (`src/bench/history/night.rs:130-151`). The owner says whether `step` should skip runs stamped not quiet.
- A Series record whose commit is not on `main` (a branch bench whose source hash equals `main`'s, `src/bench/history.rs:847-862`) would make `first_parent` and the sweep's worktree fail every night if it became a Suspect's end (`scripts/sweep.sh:67`, `:138`). Nine RTX 2070 SUPER rows name the vanished `109e1c85`; no M4 Pro row does, and the RTX has no sweep.
- Re-benches of commits older than a change of the quiet rule (`ce265e4c`, `4abdfd44`, `3c908961`) carry that binary's quiet verdict into their records; the version 14 era began after the last change, so only Suspects on older lines are affected. `publish-night` parses an old binary's results as today's `Measurements`, every field added since `3c6b9d92` being an `Option` or defaulted (`src/bench/mod.rs:190-273`), which settles the last audit's note on record shapes.
- The sweep never benches `world`: `history unrecorded` lists every Reference scene but it (`src/bin/praxis.rs:656-675`), so `world`'s M4 Pro Series grows only from sessions. ADR 0024's "benches `main` only when its source has no quiet record" is a condition, not a promise to bench every scene; `scripts/sweep.sh:52-53` records the timing behind it, `:2-3` does not say so.
- Records name, on the public `bench-history`, other processes at or over half a core with their ids (`busy_processes`) and the bench's branch. ADR 0025 rules out "private data"; whether process names are that is the owner's reading.
- `scripts/sweep.sh:192-196` removes the trees whenever `history bisect` prints nothing, also when a held Verdict's ends did not hold, so the comment "No bisect waits for a later night" is then false and the next night rebuilds them; `:148`'s mapfile comment sits above `night=`, not the `read -r` lines it explains (`:160-164`). Health. The cap comment (`:49-55`, 299 s plus 495 s under 900 s) needs a night's log to re-confirm.
- The ceilings of M4 stories 26 and 51 and M5 story 31 (the Fast travel hold 50 ms, N = 10 frames, the far field's fill 80 frames and 650 ms) are milestone criteria. The bench measures each and the Baselines hold them (`bench/baselines/apple-m4-pro/world.json`: hold 28.1 ms over 4 frames, fill 602.8 ms over 79 frames, 7 frames until traced), but `compare` reads none, and the report's Streamed table has no Baseline column (`src/bench/report.rs:231-277`); after the milestones closed a regression of any fails no command. M5 story 14's limit on shadow mask pixels that differ from LOD 0 is likewise in no code (`docs/performance.md:34` records the counts by hand). The owner says whether "kept as a ceiling" and "limited" meant the exit only.
- M5 story 29's shadow-ray time with Far proxies traced and not was a report in the change that traced them; no Render switch or CLI mode untraces them (`src/render/renderer.rs:78-115`), so it cannot be measured again without a patch. The render section marked the story for tooling; nothing in tooling implements it.
- M4 story 70: `static_tlas_build_time` is in Frame stats and printed, but not in `Measurements`, so no Bench record carries it.
- Meshes refused, Materials refused and Chunks not loaded are in Frame stats (`src/stats/mod.rs:104-109`) and the traversal (`src/bench/mod.rs:630-636`, `:679-683`) but not bench `Counters`, while `Counters::REFUSALS`' doc says "one refusal on any frame is a defect" (`:397-398`). The M4 spec's decision lists only the refusals Frame stats held then, all present; a run would show whether a refused Mesh also counts as a refused Renderable.
- `compare` judges `gpu_memory_bytes` and each kind at the measured frames' median (`src/bench/mod.rs:358`, `:1227-1238`); ADR 0013's "at most 8 GB" reads as every frame, which the M4 and M5 exits checked on the traversal's peak (`:667-678`). No frame shown over. The maximum frame is likewise judged over the measured frames only, the warm-up's 248 frames by design not.
- `gpu_memory_bytes`'s doc (`src/stats/mod.rs:62-67`) lists what the sum leaves out and omits the load staging's belt, the omission L1-R04 records in ADR 0013; fix it with L1-R04 rather than file it twice.
- `Budgets::dynamic_tlas_cpu_microseconds`' doc names "its build's recording and submit on the CPU" (`src/bench/mod.rs:714-717`) while the metric also sums `BLAS_REFIT` (`:981-985`); the last audit's note stands. The `Display` doc of `Measurements` (`:1085-1092`) and the report's doc (`src/bench/report.rs:20-23`) omit the far field's fill, which both print: omissions, not filed.
- `path_frame_times` pairs the two runs' frames from the first measured frame (`src/bench/mod.rs:1076-1083`); a hold inside the warm-up would offset the pairing. No Reference scene holds before frame 248 (the world's Fast travel is at frame 1800).
- `compare` of a `--lod` bench (`<scene>-lod-<n>.json`) finds no Baseline and names `praxis bench <dir> <scene>-lod-<n>` as the fix (`src/bin/praxis.rs:770-777`), a scene name the parser refuses. Health.
- `State::frame_stats` calls `generate_allocator_report` on every read (`src/state.rs:374`), which walks the allocator on Vulkan and DX12, after the measured frame. Performance lane.
- `FrameStats::cpu_times` is documented "in run order" (`src/stats/mod.rs:16-18`) but built as every system span, then every stage (`:303-310`, `:321-335`); every system runs before the first stage today. The Goldens test compares `capture_frames` renders with Goldens that `update_goldens` records through `praxis::capture` (`tests/goldens.rs:200-224`, `:333-346`), so it also requires the two paths to agree at frame 59, which its header does not state. Neither is a defect.
- `scripts/verify.sh:38` measures aliasing only for a diff under `src/render`, `src/cook`, `src/scenes` or `assets`; `vendor/` and `src/streaming/`, both new since `8bc34eb1`, are not on the list, while `src/bench/provenance.rs` counts `vendor/` among the measured paths. The same class as M3.5's L1-T11, whose fix chose a list; a diff touching only `vendor/` that moves a Golden would settle it.
- The seven `dtolnay/rust-toolchain@b725eda5… # stable` pins name a branch, which `gh api` reports as diverged from the pinned commit; it still resolves and installs whatever stable is current. The owner says whether a branch name satisfies "its version in a comment". The new workflow's claim that its job ids let rust-cache restore CI's caches holds (rust-cache v2.9.3 keys on `GITHUB_JOB`, `src/config.ts:73-95` at that tag).
- `tests/asset_manifest.rs:190-193` requires `dev_only` for an `-NC-` licence only; nothing requires a `dev_only` entry to be fetched, and an `ND` licence or a `LicenseRef-` is not caught. No such entry exists, and ADR 0025 claims a test only for the manifest entry and the credit.
- `tests/field_guides.rs:351-376` counts as "on `main`" every token of every tracked file outside `docs/` and Markdown, now `vendor/wgpu-core`'s source, JSON Baselines and `Cargo.lock` included, so a removed engine name found there still passes. The owner's reading.
- `tests/references.rs:219-243` walks the file system, not `git ls-files`, so an untracked `pending-artifacts/` is read: a local false failure, never CI's. `docs/guides/prepare.html:678` cites the key `bevy-tasks-0.18.1-scope-polls-results-first` in the bracketed form, absent from the registry; the page quotes code at its Describes commit, and HTML is not read by the test. Docs lane.
- `tests/conventions.rs:42` names its rule "measured timing or ratio", but `has_measured_figure` (`:562-580`) catches ratios only as `%`; no shipping comment says "N times" or "Nx faster".
- `scripts/pr-lint.sh:93` asks a PR that moves a Golden from outside the rendering paths (a scene generator, a transform change) for no images, though `CLAUDE.md` says a PR that moves a Golden shows before and after; the workflow's lint claim covers only the rendering paths. Owner.
- `compare --recording` judges the Budgets too (`src/bin/praxis.rs:755`), so a lavapipe run past a memory Budget would be copied as a Baseline at every recording; lavapipe is far under the Budgets.
- `.github/workflows/ci.yml:234-235` says the shipping job's capture runs "must be the Goldens test's: the Golden's frame alone, then the later WORLD_FRAMES together"; the test also renders `world`'s four frames in one `capture_frames` route (`tests/goldens.rs:142-198`). The job is green; a run from a cold cache would confirm. Pushes to `main` that change only skipped paths skip `check` too (`ci.yml:8-28`), so a docs-only PR not up to date with `main` merges without the field guides and references tests running on the merged commit; documented in the comment.
- `CONTEXT.md:259` says of the Cycles Baseline's tolerance "each scene has its own", while `compare.py:91-92` gives every scene the same two; the tolerances (`:85-92`) are ten times the Baseline's rounding, justified by a three-run spread only a run with Blender re-measures. `src/cook/mod.rs:585` says "`praxis cook` cooks every Reference scene's" while `src/bin/praxis.rs:494-496` leaves `world` out: the cook reader's. `.gitignore:19-20` ignores `/bench/baselines/helmet.json` as "local-only" while helmet Baselines are committed per adapter. Docs lane.
- `tests/compute_pool.rs` was deleted with the pool's move into each `FramePreparer`; ADR 0016's `available_parallelism() − 1` clause is tested in `src/threading.rs:36`.

### Reader scope statements

**The bench: run, report and provenance.** Read at the frozen commit in the worktree: `src/bench/mod.rs` (3,270 lines) in full from line 1 to 1581, its test module by test name and wherever a row cites a test; `src/bench/report.rs` (759) from 1 to 399 with its tests by name; `src/bench/provenance.rs` (705) in full. `git diff 8bc34eb1..HEAD` for the three files, `bench/budgets.json` and ADRs 0013 and 0024, and the first-parent log of every commit since `8bc34eb1` touching them, `src/stats`, `src/render/readback.rs` or `src/capture/mod.rs`, with `HARNESS_VERSION` at each. Opened to confirm what they read: `bench/budgets.json`, the M4 Pro's `town`, `helmet` and `world` Baselines, and the render, stats, pools and preflight lines each row cites. ADRs read whole: 0013 and 0024 clause by clause; 0019, 0020, 0022, 0026 and 0027 searched for bench clauses. The closed M4 and M5 milestones' exit criteria were read with `gh api`.

**The Bench history and the night.** Read in full, non-test code: `src/bench/history.rs` (lines 1-1245 of 2,253), `src/bench/history/import.rs` (1-477 of 751), `night.rs` (1-189 of 345), `rerecord.rs` (1-349 of 556), `suspects.rs` (1-102 of 210), `verdicts.rs` (1-773 of 1,513), `scripts/sweep.sh` (213) and `scripts/sweep.plist` (29); each file's diff since `8bc34eb1`; test modules where cited. Opened to confirm: `src/bench/provenance.rs:1-350`, the parts of `src/bench/mod.rs` the history reads, `src/bin/praxis.rs:367-392`, `:620-749`, preflight's labels and thresholds, `docs/agents/project-shape.md`, `docs/guides/bench-history.html`, `CONTEXT.md:254-304`. ADRs read whole: 0024, 0013 and 0025. Git read only: the Baselines at every commit since `3c6b9d92`, the commits each Finding names, and the local `origin/bench-history` at `472a27c6` (2026-11-23, after the frozen commit, read as the record of what the import wrote, not as the frozen code). Source hashes were recomputed from `git ls-tree` in Python, never with `git mktree`.

**Frame stats, the GPU memory ledger, capture, preflight and the CLI.** Read in full: `src/stats/mod.rs` (543), `src/stats/gpu_memory.rs` (461), `src/capture/mod.rs` (796), `src/preflight/mod.rs` (979), `src/bin/praxis.rs` (1,472), `tests/frame_stats.rs` (612) and `tests/goldens.rs` (539): 5,402 lines, every hunk since `8bc34eb1` among them. Opened to confirm: `src/state.rs:250-330`, `:360-378`, the streaming, staging, renderer, pools and provenance lines each row cites, `bench/budgets.json`, and `git show 02fcac89`. ADRs read whole with their diffs: 0013, 0016, 0017, 0021, 0024; 0004, 0014, 0023 and 0026 where a clause falls here. Registry quotes were confirmed in wgpu-hal 30.0.1 and 30.0.2 and wgpu-types 30.0.2 under `~/.cargo/registry/src/`.

**The scripts, the Cycles reference, the workflows and the rule tests.** Read in full: `scripts/verify.sh` (80), `gallery.sh` (160), `pr-lint.sh` (146), `pr-lint-test.sh` (271), `lavapipe-files.sh` (63), the three workflows (353, 111, 61), `.github/pull_request_template.md` (25), `tests/field_guides.rs` (631), `tests/references.rs` (243), `tests/dependency_licences.rs` (89) and `scripts/cycles-reference/DIFFERENCES.md` (203). By changed hunk with the code around them: `tests/conventions.rs` (664), `tests/cook.rs` (918), `compare.py` (299), `render.py` (449), `calibrate.py` (93). Opened to confirm a script's contract: `tests/goldens.rs`, the CLI's parsing and subcommands, `src/bench/mod.rs`'s Baseline paths and modes, `src/scenes/mod.rs`, `tests/asset_manifest.rs`, `assets/manifest.json`, `.gitignore`, `Cargo.toml`, `vendor/README.md`, the `bench/baselines/` and `bench/cycles/` listings and the history of `scripts/gallery.sh`. ADRs read whole: 0013, 0024 and 0025 with their diffs, and ADR 0016's compute pool clause. Outside the tree, read-only: a scratch git repository to confirm `ls-files --modified` on a deletion, and `gh api` reads of the actions' tags and rust-cache's source.

No reader ran `cargo`, `praxis` or a script, or wrote in either checkout. The other lane 1 sections (streaming, render, shaders) were grepped before filing: L1-R02, L1-R04 and L1-F02 touch tooling clauses and are cited, not filed again.

### Counts

| Severity class | Findings |
|---|---|
| `image` | 0 |
| `budget` | 0 |
| `correctness` | 4 (L1-T01, L1-T02, L1-T03, L1-T04) |
| `drift` | 7 (L1-T05, L1-T06, L1-T07, L1-T08, L1-T09, L1-T10, L1-T11) |
| `health` | 0 |

Before the verifier. Clause verdicts: 120 rows; 6 are broken, each naming
its Finding (three of them holding for published records and broken for the
import of re-recorded Baselines); 106 hold, 5 of them with a `drift`
Finding on a document's text (L1-T05, L1-T06, L1-T08, L1-T09, L1-T11); 4
fall in another module; 3 are not applicable; 1 is not implemented (ADR
0013's Input latency Budget, as M3.5 recorded); none is `superseded`. Spec
stories: 34 rows, one of them superseded by an M5 story and one (M5 story
29) not implemented in tooling as a standing measure. Registry keys: 12
citations of 11 keys, all matching their quotes.

## cook

Scope: `src/cook/`, `src/assets/` and `src/scenes/`, as far as they
changed after `8bc34eb1`: 11,797 changed lines at the frozen commit
(`git diff --numstat 8bc34eb1..e5c55a8d`). `src/cook/` 4,755 (`mod.rs`
937, `far_proxy.rs` 833, `mesh.rs` 740, `far_proxy/bake.rs` 638,
`launch.rs` 557, `chunk.rs` 405, `export.rs` 351, `texture.rs` 294),
`src/assets/` 4,983 (`material.rs` 1,884, `pools.rs` 1,746, `cooked.rs`
712, `loader.rs` 587, `mesh.rs` 46, `mod.rs` 8) and `src/scenes/` 2,059
(`world.rs` 998, `mod.rs` 647, `town.rs` 414), with the cook's seam tests
in `tests/cook.rs` (183). `src/cook/fetch.rs` did not change and was
opened only to judge a clause. Four readers at high effort shared the
module by file, each with the lane brief verbatim: A, the cooker's Meshes,
Textures, scene cook and export (`src/cook/mod.rs`, `mesh.rs`,
`texture.rs`, `export.rs`, `tests/cook.rs` and the writer's half of
`src/assets/cooked.rs`); B, the world's cook (`src/cook/chunk.rs`,
`launch.rs`, `far_proxy.rs`, `far_proxy/bake.rs`); C, assets (all of
`src/assets/`, `cooked.rs` for its reader's half); D, the Reference scenes
(all of `src/scenes/`). Each reader's scope, clause verdicts, stories,
registry keys and oracles are a subsection below, its oracle floors in
blocks numbered by its letter (A1, B1, ...). Their Findings are one table,
numbered from `L1-C01` in the readers' order; "M3.5's L1-C0n" names that
report's Finding. Readers A and D filed the same drift of ADR 0018's
sentence on a Thin Primitive's cap normals, which is one Finding here,
L1-C01, under reader A, whose clause it is. ADRs read whole and judged
clause by clause: 0002, 0011 and 0018, and the cook and Arena clauses of
0017, 0025 and 0026; and, where a clause's implementation lies in these
files, 0001, 0003, 0006, 0008, 0013, 0014, 0016, 0019, 0020, 0021, 0022,
0023 and 0027, with the `CLAUDE.md` review checklist's cook, pool and
scene lines as their operational form. One clause handed to the module as
known broken, ADR 0017's capacity clause (the texture pool's Channels are
capped in Texture classes, not bytes), carries `broken, known` and is
under reader C's *Open issues read*; ADR 0022's determinism, which the
streaming section filed as L1-F41, carries `broken, known` in reader D's
rows and is not filed again. `deviations.md` holds one row in these files,
`src/cook/mesh.rs:meshlet_triangle_counts`, which the code matches. Every
`where` is `path:line` from the repository root at the frozen commit.

Oracles: 39 rows over the tests added or changed in scope (A 7, B 11, C
16, D 5); one states its floor beside its constant (A's LOD chain test).
The session ran the module's tests on the frozen tree before assembling
the section, at `e5c55a8d` on the Apple M4 Pro, battery charged at 100%,
no peer session benching or holding a quiet turn (each asked beforehand).
A first run without SciFiHelmet in the worktree (load average 5.22 at the
start) passed; with SciFiHelmet fetched in (load average 2.10 at the
start), `cargo test --lib -- cook:: assets:: scenes::` passed 95 of 95 in
14.51 s (cook 49, assets 36, scenes 10) and `cargo test --test cook`
passed 16 of 16 in 8.52 s. One measurement was taken for a Finding, with
a temporary print, reverted after the run: SciFiHelmet's Mesh Blob is
353,120 bytes (L1-C02). The readers ran no project code; reader D
recomputed the world generator's `random` in a standalone script and
reader B one bake read in Python.

### the cooker's Meshes, Textures, scene cook and export

Read in full at the frozen commit `e5c55a8d`: `src/cook/mod.rs` (1,324
lines), `src/cook/mesh.rs` (1,349), `src/cook/texture.rs` (753) and
`src/cook/export.rs` (1,403), each with its `#[cfg(test)]` module, and
`tests/cook.rs` (918) as the cook's seam tests; 5,747 lines. The writer
half of `src/assets/cooked.rs` was read for what the cooker writes: the
constants and Asset ID derivation (`:1-154`), `CookedMaterial::to_blob`
(`:171-201`), `CookedTexture::to_blob` and `resident_tail_level`
(`:249-372`), `CookedVertex`, `CookedPrimitive`, `CookedMeshlet::split`,
`CookedMesh`, `CookedLod` and `CookedMesh::to_blob` (`:485-702`); its
reader's verdicts are reader C's. The scope's diff
(`git diff 8bc34eb1..e5c55a8d`) changed 937 lines of `mod.rs`, 740 of
`mesh.rs`, 294 of `texture.rs`, 351 of `export.rs` and 183 of
`tests/cook.rs`; rows on unchanged code give the frozen commit's lines.
ADRs read whole, status lines and amendments included: 0002, 0008, 0011,
0014, 0017, 0018, 0019, 0023, 0025, 0026 and 0027. Spec stories read:
every story and decision of `spec-611.md` (M4) and `spec-804.md` (M5);
those these files implement are under *Spec stories*. Opened outside the
files, to judge a clause and not audited: `src/cook/launch.rs:160-241`
(the dev launch's use of `kept_chunks` and of a Far proxy's references),
`src/cook/far_proxy.rs:22-38` and `src/cook/far_proxy/bake.rs:11-26`
(what the Far proxy's Cook key hashes), `src/assets/pools.rs:510-513`
and `src/render/prepare.rs:530-535`, `:774-781` (when prepare skips a
Renderable without a Material Handle), `src/streaming/mod.rs:313-321`
(`distance_metres`), `src/scenes/mod.rs:85-98`, `:553-575`,
`:1432-1514` (`ReferenceScene::ALL`, the Load radius and View distance,
`uv_sphere`, `ground_plane`), `src/bin/praxis.rs:488-500` (which scenes
plain `praxis cook` cooks), `.cargo/config.toml`, `.github/workflows/ci.yml`
(which jobs run `cargo test`), `Cargo.lock`, and the messages of commits
`c47a441d` (cap normals) and `e5c55a8d` (the `meshopt` 0.7 bump) for the
decisions they record. Crate sources opened, each named where used:
`meshopt-0.7.0` (`src/simplify.rs`, `src/encoding.rs`, and its vendored
meshoptimizer 1.3: `vendor/src/meshoptimizer.h`, `simplifier.cpp`,
`clusterizer.cpp`, `vertexcodec.cpp`), `bevy_mikktspace-1.0.0`
(`src/lib.rs`, `src/mikktspace.rs`) and `glam-0.34.3`
(`src/f32/neon/quat.rs`). `docs/audit/deviations.md`,
`scripts/cycles-reference/DIFFERENCES.md`, `docs/references.toml` and
`CONTEXT.md` were read for what is not a Finding; M3.5's cook section
(`docs/audit/m3.5/correctness.md:494-1504`) and the M5.5 report so far
were read for what was filed. No open issue was named for these files.
No `cargo` command was run: every oracle row gives the test's constant
and the floor derived by hand, not a measured run.

#### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0011 | "glTF (from Blender) is the authoring source; the runtime loads Blobs with mip chains, GPU block compression (BC) and byte ranges sized for streaming" | holds: the cooker alone parses glTF (`gltf` behind the `cook` feature), and writes a full mip chain of BC blocks per Texture with a KTX2 level index giving each mip its byte range | `src/cook/mod.rs:956-1052`; `src/cook/texture.rs:162-218`; `src/assets/cooked.rs:329-345`; `Cargo.toml:41` |
| 0011 | "a development launch reaches glTF only through the cooker, which cooks what changed before the runtime loads Blobs" | falls in reader B (`launch.rs`); `praxis cook <scene>`'s whole-scene cook, the other path, is `cook_scene` | `src/cook/mod.rs:603-613` |
| 0011 | "Textures cook into KTX2, a standard container" | holds: `ktx2` 0.5.0's header, level index and DFD serialisers, one key-value entry | `src/assets/cooked.rs:312-372` |
| 0018 | "An Asset ID is the first 128 bits of SHA-256 over (GUID, Asset kind, glTF name, Channel for a Texture)" | holds, in that order, each part tagged and each string length-prefixed; a Texture is named by its image's name or index, as M3.5 recorded | `src/assets/cooked.rs:103-143`; `src/cook/mod.rs:965-969`, `:1033`; `src/cook/mesh.rs:709-712`, `:736` |
| 0018 | "The glTF index replaces the name when the name is missing or not unique" | holds: `asset_names` gives `Index` to a missing name or one another item shares, for images, Materials and Meshes alike | `src/cook/mod.rs:1071-1082`, `:965`, `:970`; `src/cook/mesh.rs:681-682` |
| 0018 | "A cooked Asset is a Blob named by the SHA-256 of its bytes, found through a cook index ... rebuilt when its Cook key changes" | holds: every write goes through `CookedCache::write` under the Asset's Cook key, and `is_fresh` compares the key with the Blob present | `src/cook/mod.rs:645-694`, `:902-907`; `src/assets/cooked.rs:1174-1185` |
| 0018 | "A glTF mesh renamed in Blender gets a new Asset ID" | holds by construction: the Mesh's name is hashed; the seam renames NormalTangentTest's Mesh and reads the old ID's Blob still there | `src/cook/mesh.rs:736`; `tests/cook.rs:743-799` |
| 0018 | "The cooker warns when an ID that something references no longer exists" | holds, with a ceiling the code states: M3.5's L1-C01 is fixed, as Material to Texture, Mesh to Material, Chunk to Mesh and Material and Reference scene to every Asset its spawn loads are checked against what this cook wrote or found fresh, not against the never-pruned cache; while any source of the referring scene is absent, every ID the cache holds passes, a stale one included (the `ponytail:` comment), and a Far proxy's references are checked only by a dev launch | `src/cook/mod.rs:615-636`, `:731-751`, `:909-934`, `:919-921`, `:514-525`; `src/cook/launch.rs:193-195`; tests `src/cook/mod.rs:1105-1186`, `tests/cook.rs:743-799` |
| 0018 | "Cook key: SHA-256 over what the Asset cooks from (a Texture's source image, a Mesh's decoded Primitives and their Material IDs, a Material's own Blob; by content ...)" | holds: a Texture over its source bytes, or a generated image's pixels and size; a Mesh over every `Geometry` field, destructured so a new field fails to compile, and its Material ID, each part length-prefixed; a Material over its Blob | `src/cook/texture.rs:135-158`; `src/cook/mesh.rs:57-90`; `src/cook/mod.rs:882-888` |
| 0018 | "the Cook keys of its dependencies, every cook setting, the format version and the encoder versions" | holds: the normal Texture's key, tagged `N`; `texture_zstd_level`; the three format versions; `intel_tex_2 0.5.0 basic`, `bevy_mikktspace 1.0.0`, `meshoptimizer 1.3, vertex codec 1` and zstd's runtime version, which match `Cargo.lock` (`intel_tex_2` 0.5.0, `bevy_mikktspace` 1.0.0, `meshopt` 0.7.0), `meshoptimizer.h:15` (`MESHOPTIMIZER_VERSION 1030`) and `vertexcodec.cpp:125` (`gEncodeVertexVersion = 1`) | `src/cook/texture.rs:21`, `:144-147`, `:277-286`; `src/cook/mesh.rs:45-51`, `:59-60`; `src/cook/mod.rs:884-885` |
| 0018 | "The metallic-roughness Texture depends on the normal Texture (Toksvig)" | holds for glTF and generated sources, the first Material's normal Texture where several share the image | `src/cook/mod.rs:1019-1030`, `:830-853`; `src/cook/texture.rs:98-104`, `:144-147` |
| 0018 | "Each dev launch recomputes the keys of its scene's Materials and Meshes as it starts, of each Chunk and Texture as it loads it ..." | falls in reader B (`launch.rs`) | |
| 0018 | "The Cooked cache lives in git-ignored `cooked/` and is never evicted. Deleting it is always safe." | holds, cook side: no cooker code removes an index file or a Blob; the seam deletes the cache and cooks again | `src/cook/mod.rs:638-759`; `tests/cook.rs:254-274` |
| 0018 | "Each Blob starts with a magic and a format version (a Texture Blob is KTX2, with its format version in a key-value entry)" | holds for what the cooker writes: `PXMA` 2, `PXME` 8, and `PXTXVERSION` = 2 as a little-endian u32 | `src/assets/cooked.rs:21-40`, `:179-180`, `:322-327`, `:698-699` |
| 0018 | "Engine data that no cook format holds exactly is compiled into the binary instead ... The runtime still opens no source path" | holds, cook side: `read_sources` reads only `.gltf` and `.glb` entries, so the dither mask is never cooked | `src/cook/mod.rs:784-787` |
| 0018 | "Textures are KTX2 with every level zstd-supercompressed, stored smallest mip first, so the Resident tail ... is one contiguous byte range ... Rows are stored unpadded" | holds: levels written in `rev()` order, each compressed on its own, scheme Zstandard, each level the encoder's tightly packed blocks | `src/assets/cooked.rs:336-345`, `:355`; `src/cook/texture.rs:592-597`, `:628` |
| 0018 | "Formats per Channel, without RDO: albedo and emissive BC7 sRGB, normal BC5 (z rebuilt in the shader), metallic-roughness BC5 linear, occlusion BC4 ... `intel_tex_2` at its basic profile ... A Far proxy's baked albedo alone encodes at its `ultra_fast` profile" | holds: the format per Channel; the basic BC7 profiles, alpha or opaque by the level's texels; `with_fastest_bc7` selects `ultra_fast` and enters the Cook key (it is also set on the baked metallic-roughness Texture, where BC5 ignores it) | `src/cook/texture.rs:179-187`, `:622-627`, `:139-141`; `src/cook/mod.rs:450` |
| 0018 | "Mips: a 2×2 box filter in linear light, with normals renormalised per mip. Toksvig widens each mip's roughness ... The cooker refuses a Texture whose size is not a multiple of 4 or exceeds 8192" | holds: sRGB decoded by table, the box mean, `renormalise` per normal mip, `widened_roughness` from the normal Texture's carried variance, both refusals tested | `src/cook/texture.rs:222-253`, `:351-398`, `:421-448`, `:504-512`, `:565-590`, `:169-178`; `tests/cook.rs:378-390` |
| 0018 | "Meshes: a 28-byte vertex (position `Float32x3`, UV `Float32x2`, ... QTangent in `Snorm16x4`, its w kept non-zero ...), and u32 indices" | holds | `src/assets/cooked.rs:485-497`; `src/cook/mesh.rs:642-669` |
| 0018 | "`meshopt` reorders for vertex cache, overdraw and fetch, then encodes, then zstd compresses" | holds | `src/cook/mesh.rs:485-492`, `:213`; `src/assets/cooked.rs:650-700` |
| 0018 | "Each Primitive's triangles are cut, in that order, into Meshlets of at most 64 vertices ... and at most 124 triangles ... as meshoptimizer's scan builder cuts them" | holds, and the cut on a degenerate triangle matches the Deviation `src/cook/mesh.rs:meshlet_triangle_counts` (`clusterizer.cpp:245-261` in 1.3 still aliases `av`, `bv`, `cv` into `used[]`) | `src/cook/mesh.rs:503-535`; `src/assets/cooked.rs:534-588` |
| 0018 | "A Meshlet is stored as one byte, its triangle count; the reader rebuilds its vertex remap and local triangles" | holds, writer side | `src/assets/cooked.rs:688-694` |
| 0018 | "The Blob also holds the Mesh's bounding box for culling" | holds: the box of LOD 0's vertices and any box a Primitive adds | `src/cook/mesh.rs:220-232`; `src/assets/cooked.rs:665-667` |
| 0018 | "Each Primitive stores its UV texel density ...: the square root of its summed UV area over its summed mesh-space area, in UV units per metre, ... taken per Primitive, as a ratio of summed areas and as a length" | holds: summed in f64 over the source triangles, `sqrt(uv_area / mesh_area)`, 0 where the mesh area is 0; a degenerate triangle adds 0 to both sums, a Primitive without UVs reads 0 (its UVs default to zeros) | `src/cook/mesh.rs:435-453`, `:701-704`; `src/assets/cooked.rs:505-510`, `:670` |
| 0018 | "The Blob holds the Mesh's LOD chain: the vertices once, which every LOD indexes, then per LOD its error in metres, each Primitive's run of its indices and its Meshlets, with every LOD's indices in one encoded stream" | holds; the error stored is the simplifier's, of position, normal and UV together, times the Mesh's extent (see *Notes*) | `src/assets/cooked.rs:637-702`, `:605-616` |
| 0018 | "Each LOD is simplified from the one before to half its triangles by meshoptimizer's attribute-aware simplifier, the whole Mesh in one call, normals and UVs weighted, ... a chain holds at most 8 LODs and ends before a LOD that would keep more than 85% of the indices before it" | holds: `simplify_with_attributes_and_locks` on the finer LOD with a target of `finer.len() / 6 * 3`, no error limit, weights 0.5 and 1.0, the loop bounded by `MESH_MAX_LODS` = 8, a LOD over 0.85 of the finer dropped; each LOD ordered and cut per Primitive | `src/cook/mesh.rs:302-420` |
| 0018 | "A Thin Primitive's vertices are locked in the simplifier [ref:meshoptimizer-1.3-vertex-lock], so every LOD keeps its LOD 0 triangles and a Mesh of Thin Primitives alone has no coarser LOD" | holds: a `true` lock per vertex of a thin Primitive, which `meshopt` 0.7 passes as the Lock bit (`simplify.rs:60`) and which meshoptimizer extends to every wedge at the same position (`simplifier.cpp:510-517`); with no vertex free, a Mesh of thin Primitives alone keeps every index and its chain ends at the 85% rule | `src/cook/mesh.rs:344-351`; test `:780-812` |
| 0018 | "Each Primitive stores a Thin Primitive's radius, zero for any other" | holds | `src/cook/mesh.rs:205`; `src/assets/cooked.rs:671` |
| 0018 | "The cook refuses one unless each triangle is the side of a tube whose radius is within a quarter of the one declared, or a cap closing such a tube, so a side vertex's position less its normal times the radius lies within a quarter of the radius of the centre line and no vertex stores it" | holds: an edge whose normals differ lies on one ring when its centres, position less normal times the declared radius, are within 0.25 of the radius times the normals' difference, which on one ring holds a true radius R to within r/4 of the declared r; a triangle passes as a side with such an edge, or as a cap whose edges are all within 2.5 radii, its vertices at ring positions; tested by Mesh and Primitive | `src/cook/mesh.rs:98`, `:103-150`; test `src/cook/mod.rs:1088-1099` |
| 0018 | "a cap's vertices keep their own normals (amended 2026-11-14)" | holds in intent, wording broken: since 2026-11-16 each cap vertex takes its ring's normal so that a cap widens with its side, which the ADR does not say: L1-C01 | `src/cook/mesh.rs:100-102`, `:151-158`; `src/assets/cooked.rs:511-514` |
| 0018 | "A Material is its own Asset: PBR factors plus one Texture Asset ID per Channel. Primitives reference it by ID" | holds | `src/assets/cooked.rs:156-201`; `src/cook/mod.rs:1054-1067`; `src/cook/mesh.rs:709-712` |
| 0018 | "A Chunk's edge is 256 m ... A Placement holds a Mesh Asset ID ..." | falls in reader B (`chunk.rs`) | |
| 0018 | "The cooker lives in `src/cook/` behind the `praxis cook` command ... a default-on `cook` feature ... Procedural Reference scenes cook too, under Asset IDs derived from the scene name" | holds | `src/cook/mod.rs:813-867`; `src/assets/cooked.rs:114-117`; `Cargo.toml:41` |
| 0018 | status line: "Plain `praxis cook` leaves out the `world` scene, whose library of more than 8 GB cooks only as `praxis cook world` or on a miss" | holds in the command; `cook`'s doc comment still says plain `praxis cook` cooks every scene: L1-C03 | `src/bin/praxis.rs:493-498`; `src/cook/mod.rs:584-586` |
| 0018 | status line: "`praxis cook <scene>` cooks a scene whole"; "A dev launch cooks only what it loads" | the first holds (`cook_scene` reads the scene's sources and cooks every Asset, Chunk and Far proxy of them); the second falls in reader B | `src/cook/mod.rs:603-613`, `:638-759` |
| 0002 | "Meshes are (vertex offset, index offset, index count) sub-allocations of one shared vertex buffer and one shared index buffer" (as the Mesh Blob serves it) | holds, writer side: a Mesh's vertices are one run every LOD indexes from 0, and every LOD's indices are u32 into it, so a loader adds one vertex offset; the Arenas are reader C's | `src/assets/cooked.rs:591-616`, `:649-656` |
| 0017 | "The cooked format (ADR 0011) stores each mip as a separately loadable byte range of the Texture's Blob, with the Resident tail as one range, and the Mesh Blob each Primitive's UV texel density" | holds, writer side: one level index entry and one zstd frame per mip, smallest first; the density per Primitive | `src/assets/cooked.rs:333-345`, `:668-672` |
| 0017 | "Each texture's Resident tail (mips of 64 KiB and smaller) never leaves ... 87,408 bytes for a 2,048² BC7 or BC5 Texture" (as cooked) | holds as the cook lays it out: the 2,048² chain's mips of 64 KiB or less, 256² down to 1², take 65,536 + 16,384 + 4,096 + 1,024 + 256 + 64 + 3 × 16 = 87,408 bytes; `resident_tail_level` and the pool are reader C's | `src/assets/cooked.rs:253-269`, `:288-299` |
| 0017 | "Priority ... from distance and the Primitive's UV texel density (stored at cook time, per Primitive ...)" | holds, cook side (above); the priority falls in streaming | `src/cook/mesh.rs:204`, `:435-453` |
| 0008 | "Nothing the image depends on reads screen-space history"; ADR 0018's "Toksvig ... a cook-time, non-temporal answer to specular aliasing (ADR 0008)" | holds: the widening reads the normal Texture's texels alone, at cook time | `src/cook/texture.rs:243-250`, `:565-590` |
| 0014 | "cooked meshes stay within 128 m of their pivot, which the cooker enforces by refusing a mesh with any vertex farther out" | holds: every source vertex's length against `MAX_PIVOT_DISTANCE_METRES`, the Mesh named by `mesh_blob`'s context; the seam cooks 128.0 m and refuses 128.1 m | `src/cook/mesh.rs:94`, `:166-174`; `src/cook/mod.rs:892-900`; `tests/cook.rs:427-442` |
| 0014 | "A Far proxy's Mesh is exempt ... which the cooker enforces instead" | falls in reader B (`far_proxy.rs` calls `lod_0`, not `cook`) | |
| 0014 | World positions are f64 (as `export-scene` writes them) | holds: Renderables' and Placements' translations are taken from the f64 `DAffine3` of their drawn World transform and written as `[f64; 3]`; lights' and the camera's positions as f64; vertex positions stay mesh-local f32 | `src/cook/export.rs:155-156`, `:172`, `:183`, `:204-208`, `:110`, `:287-288`, `:356-357` |
| 0019 | "the cooker builds Meshlets of at most 64 vertices into each Mesh Blob" | holds: `MESHLET_MAX_VERTICES` = 64, which the cut and `split` enforce | `src/assets/cooked.rs:45`; `src/cook/mesh.rs:519` |
| 0019 | "A Mesh is a Meshlet range per LOD of its LOD chain ...: each LOD's Meshlets are cut from its own indices over the Mesh's one vertex range, ... for at most 8 LODs" | holds, cook side | `src/cook/mesh.rs:402-413`; `src/assets/cooked.rs:42` |
| 0023 | light units (as `export-scene` writes them): lux, candela, cd/m², an ambient term in lux, an EV100 | holds: `illuminance` and `intensity` copied from components documented in lux and candela, `range` in metres, the Environment's lux and cd/m², the camera's EV100 | `src/cook/export.rs:127-131`, `:161-166`, `:218-221`, `:293`, `:296-322` |
| 0023 | "Brighter emissives need KHR_materials_emissive_strength, which the cooker does not read yet" | holds: the cooker reads no extension and now warns, naming the file and each extension used | `src/cook/mod.rs:958-961`; test `:1290-1323` |
| 0025 | "a larger source is fetched from its publisher into a git-ignored cache" (as `praxis cook <scene>` reaches it) | holds: `cook_scene` fetches a scene's absent fetched sources first; `fetch.rs` is unchanged since M3.5 | `src/cook/mod.rs:609` |
| 0025 | "a file under a noncommercial or restrictive term is dev-only: never committed, and no committed Golden is rendered from it" | not applicable to these files: the cooker writes only into the git-ignored Cooked cache, and `export-scene` names a manifest source by its absolute path without copying it | `src/cook/export.rs:511`, `:638`, `:692-693` |
| 0026 | "Its Blobs. A Mesh Blob per Chunk and, where its first Primitive has a triangle, a Material Blob and two Texture Blobs, each under an Asset ID made of the scene and the Chunk's coordinates, cooked together by `praxis cook <scene>` ..." | holds: `cook_far_proxy` writes the two baked Textures, the baked Material and the Mesh, the Mesh last, all under the proxy's one Cook key; Asset IDs from `far_proxy_id` and `far_proxy_baked_ids` | `src/cook/mod.rs:425-468`, `:676-694` |
| 0026 | "They share one Cook key, which covers the Chunk's and its neighbours' Cook keys, the Cook keys of the Meshes they place, of the Materials those draw with and of those Materials' albedo and metallic-roughness Textures, and the format and settings of each Blob" | holds: the key hashes the Mesh and Material magics and versions, `encoder_settings`, the proxy's settings (with the bake's and `ultra_fast`), the Texture settings and version, the ground's ID, each of the nine Chunks' keys or a 0 for an absent one, then the placed Meshes', drawn Materials' and their two baked Channels' Textures' keys; a test changes a drawn and an undrawn Material's key | `src/cook/mod.rs:289-369`; `src/cook/far_proxy.rs:32-38`; test `src/cook/mod.rs:1188-1219` |
| 0026 | "A proxy is fresh when its Mesh is in the Cooked cache under that key and, unless that Mesh names no Material, the three baked Blobs are too. A proxy whose sources name a Material or a Texture the cook did not read is kept as it is, as a Chunk is." | holds | `src/cook/mod.rs:182-193`, `:320`, `:344-346`, `:355-357`, `:681-683` |
| 0026 | "The albedo atlas encodes at the BC7 encoder's `ultra_fast` profile where the Texture cook's is `basic`"; "at the finest mip whose texels are no smaller than the sample, as the Texture cook filters it, so a roughness its normal Texture widens is read widened" | holds: `with_fastest_bc7`; `Mips` built through the same `filtered_levels` the cook encodes, the level `ceil(log2(texels per sample))` | `src/cook/mod.rs:450`; `src/cook/texture.rs:222-273`, `:297-313` |
| 0026 | the rest (the merge, the drop rule, the pivot, the chart, the bake, the swap) | falls in reader B (`far_proxy.rs`, `far_proxy/bake.rs`) and streaming | |
| 0027 | "A tapered tube keeps its taper as it widens, so the coverage at its ends is off by up to the quarter the cook tolerates" (the cook's quarter) | holds: `THIN_RADIUS_TOLERANCE` = 0.25 bounds a ring's radius to within a quarter of the declared one | `src/cook/mesh.rs:96-98`, `:111-117` |
| 0027 | "A Far proxy holds no Thin Primitive" | falls in reader B | |
| `CLAUDE.md` | "The runtime loads cooked Blobs only and references Assets by Asset ID, never by source path (ADR 0018)" | holds for these files: the cooker writes Blobs by Asset ID; `export-scene` is a `cook`-feature tool that writes source paths into `scene.json` for a reference renderer, which no runtime path reads | `src/cook/mod.rs:645-730`; `src/cook/export.rs:268-284` |
| `CLAUDE.md` | "A cooked Mesh Blob is the same bytes on every machine: the two hash tests ... pass on each" | holds where the test runs; see oracle A1 for what it can and cannot detect | `src/cook/mesh.rs:992-1008`; `.cargo/config.toml:4-6` |
| `CLAUDE.md` | no GPU buffer grows and a refused request is logged; uploads through the load staging; GPU memory through the ledger; no GPU resources on ECS components; the slot order of prepare's writes | not applicable: these files create no GPU resource and write no Instance; `export-scene` reads components only | |

#### Spec stories

| Story | Verdict | Where |
|---|---|---|
| M4 story 9, the cooker refuses a Mesh more than 128 m from its pivot, naming the Mesh and the distance | holds: the error names the distance to 0.1 m and the context names the Mesh; the seam's boundary cooks at exactly 128.0 m (128 × 128 = 16,384, whose square root is exact) | `src/cook/mesh.rs:166-174`; `src/cook/mod.rs:892-900`; `tests/cook.rs:427-442` |
| M4 story 12, the cook warns when a Placement names an Asset ID no longer defined | holds: a Chunk's Placement references join the check against what the cook made, each warning naming the Chunk and the ID | `src/cook/mod.rs:661-664`, `:739-751`, `:930-932`; tests `src/cook/mod.rs:1136-1186`, `tests/cook.rs:743-799` |
| M4 story 13, the Mesh Blob stores each Primitive's UV texel density | holds | `src/cook/mesh.rs:204`, `:435-453`; `src/assets/cooked.rs:670` |
| M4 story 56, a generated world of 40 × 40 Chunks cooked by `praxis cook` | holds as ADR 0018 amended it, by `praxis cook world`: `read_sources` lists 1,600 Chunks for `world` | `src/cook/mod.rs:871-876`; test `:1235-1269` |
| M4 story 58, `world`'s unique Textures total more than 8 GB | holds: the test asserts distinct IDs, four sampled Textures distinct, and one Texture's chain bytes times the count over 8,000,000,000 | `src/cook/mod.rs:1235-1269` |
| M4 decision, the cooker: "Texture Blobs record their shape (size, mip count, format), which names their class" | holds: the KTX2 header's pixel width and height, level count and format | `src/assets/cooked.rs:346-355` |
| M4 stories 6-8, 10, 11, 63, 64 | fall in reader B (Chunks, the dev launch) and reader D (ground tiles) | |
| M5 story 4, each Mesh cooked with a LOD chain sharing its source's vertices, each LOD about half the one before, at most 8 | holds | `src/cook/mesh.rs:302-420` |
| M5 story 5, LOD 0 cooked bit-identical to the Mesh before chains | holds in form: `lod_0` is the pre-chain cook, and nothing in `coarser_lods` writes back into LOD 0's vertices, indices or Meshlets; the bit-identity was the chain ticket's Golden check | `src/cook/mesh.rs:175-176`, `:180-234` |
| M5 story 6, each LOD's error stored in metres | holds: the simplifier's relative error times `simplify_scale`, summed down the chain, as an f32 per LOD; it includes the weighted normal and UV error (see *Notes*) | `src/cook/mesh.rs:354`, `:375`; `src/assets/cooked.rs:680` |
| M5 story 7, a cook test that a chain's LODs have no cracks along Primitive borders | holds: every edge of every LOD of a two-Primitive sphere is matched by its reverse, each Primitive keeps to its half and the equator loses vertices | `src/cook/mesh.rs:1247-1329` |
| M5 story 15, the simplifier's weights and the threshold tuned on the gallery once thin features are widened | not settled from code: the weights are the values the chain's first commit set (`cbbad8c6`, normals 0.5, UVs 1.0) and no later commit in scope changes them | `src/cook/mesh.rs:33-39` |
| M5 story 16, each Chunk cooked a Far proxy as one Blob under its own Asset ID | holds for the Blobs and their Cook key (above); the merge, drop and simplification fall in reader B | `src/cook/mod.rs:289-369`, `:425-468` |
| M5 story 17, a Far proxy's Material baked from its sources' Materials | falls in reader B; its encode at `ultra_fast` and its sources' mips are these files' and hold (above) | `src/cook/mod.rs:384-420`, `:441-453` |
| M5 story 33, a Primitive declared thin with a radius, in a generator or by glTF `extras` | holds: `extras.thin_radius_metres` read as a number, refused as anything else | `src/cook/mesh.rs:713-724`; test `:814-851` |
| M5 story 34, the cooker refuses a Thin Primitive without radial normals | holds | `src/cook/mesh.rs:103-150`; test `src/cook/mod.rs:1088-1099` |
| M5 decision, LOD chain: "`simplify_with_attributes_and_locks` on the whole Mesh, each LOD from the one before, stopping when a LOD keeps more than 85% ... One Mesh Blob at format version 4" | holds, the format version since bumped to 8: 5 by the Thin Primitive's radius (`dcb1d06d`), 6 by the cook made machine-independent (`8ac44220`), 7 by the cap rule (`c47a441d`), 8 by the locked tube (`55d110eb`) | `src/cook/mesh.rs:357-374`; `src/assets/cooked.rs:40` |
| M5 decision, thin features: "The radius is stored per Primitive; the centre line is the position less the normal times the radius" | holds, a cap's normal its ring's | `src/assets/cooked.rs:511-514`; `src/cook/mesh.rs:151-158` |
| M5 stories 8-14, 18-32, 35-42 | fall in render, shaders, streaming, reader B or tooling | |

#### Registry keys

Every key cited in these files was read against its registry entry; the
vendored meshoptimizer of `meshopt` 0.7.0 was opened where a quote is
about it, and its version, 1.3 (`meshoptimizer.h:15`,
`MESHOPTIMIZER_VERSION 1030`), is the one every `meshoptimizer-1.3-*` key
names and `Cargo.lock` pins (`meshopt` 0.7.0). The keys on code unchanged
since M3.5 (`gltf-2.0-*`, `khronos-dfs-srgb`, `toksvig-2004-variance`,
`kaplanyan-2016-ndf-filtering`, `tokuyoshi-2019-roughness-matrix`, the
four `ktx2-*`) keep M3.5's verdicts, at the lines below.

| File | Keys | Verdict |
|---|---|---|
| `src/cook/mesh.rs:34` | `meshoptimizer-1.3-attribute-weights` | matches: the quote's `const float nrm_weight = 0.5f;` is `LOD_NORMAL_WEIGHT` = 0.5 (`:35`); the UVs, scaled to metres of Texture over the Mesh's extent (`:325-333`), take 1.0, which the quote's "a change of `1/weight` in attribute value over a distance `d` is approximately equivalent to a change of `d` in position" reads as one metre of Texture for one metre of position |
| `src/cook/mesh.rs:43` | `meshoptimizer-1.3-vertex-codec` | matches: `vertexcodec.cpp:125` `static int gEncodeVertexVersion = 1;` and `:1750` passes it from `meshopt_encodeVertexBuffer`, which `meshopt::encode_vertex_buffer` calls (`meshopt-0.7.0/src/encoding.rs:120-125`) from `CookedMesh::to_blob` (`src/assets/cooked.rs:650`); `encoder_settings` names "vertex codec 1" |
| `src/cook/mesh.rs:295` | `meshoptimizer-1.3-vertex-lock` | matches: "`meshopt_SimplifyVertex_Lock = 1 << 0`", which a Rust `true` is when `meshopt` 0.7 casts the `&[bool]` to the C byte array (`meshopt-0.7.0/src/simplify.rs:60`, `:260`) |
| `src/cook/mesh.rs:299` | `meshoptimizer-1.3-lod-chain` | matches: each LOD is simplified from the one before ("use the previous LOD as the starting point") and the errors are summed (`:375`, "resulting error values from previous levels should be accumulated for LOD selection"); every LOD indexes the source's vertices ("reuse the original vertex buffer") |
| `src/cook/mesh.rs:301` | `meshoptimizer-1.3-simplify-scale` | matches: "Relative error returned by meshopt_simplify via result_error must be *multiplied* by the scaling factor to get absolute error" is `relative_error * extent_metres` (`:375`), the factor from `simplify_scale` over every vertex (`:309`) |
| `src/cook/mesh.rs:365` | `meshoptimizer-1.3-simplify-error` | matches: "To disable the error limit, `target_error` can be set to `FLT_MAX`" is `f32::MAX` (`:366`) |
| `src/cook/mesh.rs:383` | `meshoptimizer-1.3-simplify-materials` | matches: "If the source mesh is already split into subsets with non-overlapping vertex indices, and permissive simplification is not used, ... simplify the entire mesh at once; the result can be split back into subsets": no two Primitives share a vertex (`:310`, `:199-210`), `SimplifyOptions::None` (`:367`), and each triangle is assigned by its first vertex's Primitive with the other two checked (`:384-397`) |
| `src/cook/mesh.rs:500` | `meshoptimizer-1.3-scan-meshlets` | matches the Deviation `src/cook/mesh.rs:meshlet_triangle_counts`: `clusterizer.cpp:245-247` binds `av`, `bv`, `cv` as references into `used[]` and `:259-261` is the quoted `used_extra` and split; the code counts a repeated corner once (`:510-515`) |
| `src/cook/mesh.rs:597-598` | `gltf-2.0-texture-origin`, `gltf-2.0-normal-texture-convention` | matches, as M3.5 found; code unchanged |
| `src/cook/mesh.rs:617` | `gltf-2.0-mikktspace-tangents` | matches, as M3.5 found, with its Note (tangents always generated, from `TEXCOORD_0`); code unchanged |
| `src/cook/mesh.rs:638`, `src/cook/export.rs:735` | `gltf-2.0-tangent-bitangent` | matches, as M3.5 found; code unchanged |
| `src/cook/texture.rs:551` | `khronos-dfs-srgb` | matches: encode knee 0.0031308 (`:553`), decode knee 0.04045 (`:355`) |
| `src/cook/texture.rs:561`, `:570`, `:579`, `:581` | `gltf-2.0-roughness-mapping`, `toksvig-2004-variance`, `kaplanyan-2016-ndf-filtering`, `tokuyoshi-2019-roughness-matrix` | match, as M3.5 found; `widened_roughness` (`:565-590`) unchanged in scope |
| `src/cook/export.rs:309`, `:317`, `:320` | `khr-lights-punctual-units`, `khr-lights-punctual-range` | match: illuminance in lux on the directional entry, intensity in candela and `range` in metres on the point entry, copied from the components |
| `src/assets/cooked.rs:29`, `:306-310` (writer) | `ktx2-key-value-alignment`, `ktx2-level-order`, `ktx2-supercompression-per-level`, `ktx2-level-alignment` | match, as M3.5 found; the writer is unchanged in substance |
| `src/assets/cooked.rs:507` | `unity-6000-uv-distribution-metric` | matches in what the doc comment claims: the quote's "Average of triangle area / uv area" inverted (UV over mesh area) and taken as a length (`sqrt`), which ADR 0018 also words as "taken per Primitive, as a ratio of summed areas"; whether Unity's "average" is of summed areas or of per-triangle ratios the quote does not say (see *Notes*) |
| `src/assets/cooked.rs:620` (writer's layout, reader C's guard) | `meshoptimizer-1.3-vertex-block` | matches: 8,192 / 28 = 292, rounded down to a multiple of 16 is 288, capped at 256 |

#### Oracles

Every oracle below runs on the CPU, in f32 or f64; no WGSL floor applies.
Behaviour tests judged and set aside: the Cook key coverage tests
(`src/cook/mesh.rs:750-778`, `src/cook/texture.rs:678-726`), the glTF
`extras` test (`src/cook/mesh.rs:814-851`), the crack test
(`:1284-1329`, topological), the Meshlet read-back
(`:1139-1158`), the Far proxy Cook key test (`src/cook/mod.rs:1188-1219`),
the dangling-reference tests (`:1105-1186`, `tests/cook.rs:743-799`), the
scene-reference test (`src/cook/mod.rs:1223-1229`), the extension warning
(`:1290-1323`), the 128 m refusal (`tests/cook.rs:427-442`, whose 128.0 m
boundary is exact: 128² = 16,384 and its square root is exact in f32), the
scene cook test (`tests/cook.rs:667-736`) and the export tests other than
`world`'s (`src/cook/export.rs:1163-1231`); the Toksvig, checker, QTangent,
PSNR and `shadows` export oracles are unchanged since M3.5 and keep its
rows.

| Function | Test | Expected value and its source | Constant | Floor | Stated floor | Verdict |
|---|---|---|---|---|---|---|
| `cook` and `CookedMesh::to_blob`, every Reference scene Mesh and NormalTangentTest's (`src/cook/mesh.rs:165-178`; `src/assets/cooked.rs:649-702`) | `the_reference_scenes_meshes_cook_to_the_same_blobs_on_every_machine`, `src/cook/mesh.rs:992-1008` | SHA-256 `e0b71ece…c486` of the concatenated Blobs, pinned across the machines that ran it (`8ac44220`, then `e5c55a8d`) | exact | 0 on the IEEE-only steps; unbounded for the platform `sin`, `cos` and `acos` calls in the chain (Block A1) | no: the message says what a disagreement means, not which steps can cause one | exact; detects a disagreement only between machines that run `cargo test` (Block A1) |
| `coarser_lods` (`src/cook/mesh.rs:302-420`) | `a_lod_chain_halves_until_its_stop_rule_within_eight_lods_and_its_errors_never_decrease`, `:1195-1245` | triangle counts by hand from `uv_sphere`: 65,024 = 2·256·128 − 2·256; 11,040 = 2,208 + 4 × 2,208; 22,080 = 2,208 + 4 × 4,968; the bounds 8,832 to 9,383 and the drop of a 90% LOD from the 85% rule; errors bounded 0.01 to 1.0 m and a ratio of 5 to 20 at ten times the size | counts exact; errors `(0.01..1.0)`, ratio `(5.0..20.0)` | counts 0; the error is meshoptimizer's quadric error, whose order of equal-cost collapses the comment names as the reason for a factor of two | yes for the ratio (`:1222-1223`: "within a factor of two"); no for 0.01 to 1.0 | counts exact; the error bounds are behaviour bounds, a factor of 2 either way of the 10x scaling (Block A2) |
| `texel_density` (`src/cook/mesh.rs:435-453`) | `a_square_metre_quad_has_the_uv_units_per_metre_its_uvs_span`, `:934-940` | 1 and 2 UV units per metre for a 1 m² quad whose UVs span 1 and 2 | exact (`assert_eq!`) | 0: every product, sum and square root in the chain is exact (Block A3) | no | exact |
| `Mips::level_of`, `Mips::level` (`src/cook/texture.rs:297-313`) | `a_sample_reads_the_finest_mip_whose_texels_are_no_smaller_than_it`, `:728-752` | levels `[0, 0, 0, 1, 2, 2, 4, 4]` for 0, 0.5, 1, 1.5, 3, 4, 100 and ∞ texels a sample on a 16 × 4 image, `ceil(log2(texels))` clamped; ADR 0026's "the finest mip whose texels are no smaller than the sample" | exact | 0, provided the platform's `log2f` returns 0 and 2 exactly at 1 and 4, two inputs that sit on a level boundary (Block A4) | no | exact, two boundary inputs resting on the platform's `log2f` |
| thin Primitives through `cook` (`src/cook/mesh.rs:189-207`, `:274-279`, `:403-408`) | `towns_wires_rails_aerials_lamps_and_poles_cook_as_thin_primitives`, `:780-812` | each Primitive's radius as `town`'s generator declares it (0.015 to 0.115 m) at every LOD, and each thin Primitive's index count equal to LOD 0's | exact | 0: the f32 radius is copied, never computed | no | exact |
| SciFiHelmet's Mesh Blob size (`src/assets/cooked.rs:649-702`) | `every_reference_scene_mesh_cooks_and_reads_back`, `tests/cook.rs:801-857` | ADR 0018: "with meshoptimizer 1.3's vertex codec 1 it is 353,120 bytes", 353,120 / 23,358 = 15.1177 bytes per triangle | `HELMET_MESH_BYTES_PER_TRIANGLE` = 15.29, a ceiling | 0: the Blob's size is a deterministic integer for one machine | no | loose: the ceiling admits 15.29 × 23,358 − 353,120 = 4,023 bytes (1.14%) of growth, the margin the codec 1 change took off and the comment says re-records the constant: L1-C02 |
| `export_scene` for `world` (`src/cook/export.rs:49-266`) | `world_exports_the_buildings_within_the_view_distance_whole`, `:1040-1158` | 225 Chunks within 2 km, 900 ground tiles (4 × 225), 23,542 buildings and roofs each, 10,000 birds, one far ground, every window matched to its walls | counts exact; camera within 1e-9 m | 0 for the counts; the camera is the same f64 `camera_at` on both sides, so its floor is 0 and 1e-9 is never reached | no | exact (Block A5) |

##### Block A1: the Mesh Blob hash

What the hash covers: the Blob of every Mesh `ReferenceScene::ALL` (12
scenes, `src/scenes/mod.rs:85-98`) generates, each Primitive given the
Material `AssetId(1)` (`src/cook/mesh.rs:980-990`), and of each Mesh of
the committed `NormalTangentTest.glb` (`:996-998`), through `cook` (LOD 0
and the whole LOD chain) and `to_blob` (meshoptimizer's vertex and index
codecs, then zstd at level 19, `src/assets/cooked.rs:626`, `:700`). It
does not cover SciFiHelmet (fetched), Far proxies (reader B's hash test),
Material or Texture Blobs, or the Cook key, which is not in the Blob.

The chain, step by step:

1. Scene generators (reader D's): `uv_sphere` takes `f64::sin_cos` and
   rounds to f32 (`src/scenes/mod.rs:1441-1447`); `town.rs` still takes f32
   `sin` and `cos` (`src/scenes/town.rs:1320`, `:1384`). Neither is
   correctly rounded by any standard; the f64 form disagrees after
   narrowing only where the f64 results differ and straddle an f32
   rounding boundary, roughly 2⁻²⁹ of a one-ulp f64 disagreement, as f64
   carries 29 more significand bits than f32.
2. `weld` (`src/cook/mesh.rs:539-561`): bitwise keys through
   `generate_vertex_remap`; exact.
3. `corner_tangents` (`:618-635`): `bevy_mikktspace` 1.0.0 in f32, whose
   default `StdOps::acos` is `(x as f64).acos() as f32`
   (`bevy_mikktspace-1.0.0/src/lib.rs:229-234`), called once per corner to
   weight a face's tangent by its angle (`src/mikktspace.rs:736-737`): a
   platform f64 `acos`, with the same 2⁻²⁹ narrowing argument as step 1.
   The `CLAUDE.md` checklist names glam's SIMD, a platform's f32 sine or
   cosine and a fused C++ build; this f64 `acos` is a fourth platform path
   it does not name (see *Notes*).
4. `qtangent` (`:642-669`): glam 0.34's `Vec3` is scalar; `Quat::from_mat3`
   reaches `from_rotation_axes`, scalar subtractions, one `sqrt` and
   products on NEON as on SSE2 (`glam-0.34.3/src/f32/neon/quat.rs:217-262`);
   the normalisation is written by hand in scalar f32 (`:652-653`); `round`
   is exact. IEEE f32 throughout, each step correctly rounded: floor 0
   between machines.
5. `texel_density` (`:435-453`): f64 cross products, `perp_dot`, `abs`,
   sums, one division and one `sqrt`, then `as f32`; IEEE: floor 0.
6. `qtangent_normal` (`:422-431`): scalar f32 for the simplifier's normal
   attribute, written so because glam's SIMD normalise rounds by
   architecture (`8ac44220`); floor 0.
7. meshoptimizer 1.3's C++ (vertex cache, overdraw with float area sums,
   vertex fetch, `simplifyScale`, the quadric simplifier, both codecs):
   built with `-ffp-contract=off` for `aarch64-apple-darwin` and
   `x86_64-unknown-linux-gnu` (`.cargo/config.toml:4-6`), so no multiply
   and add fuse there; floor 0 on those two targets. No flag is set for
   any other target, and `8ac44220` records the RTX 2070 SUPER's MSVC
   build as not covered.
8. zstd 1.5.7 at level 19 (`Cargo.lock`, `zstd-sys 2.1.3+zstd.1.5.7`):
   deterministic for one version and level.

Floor: 0 wherever the chain is IEEE arithmetic; unbounded for steps 1 and
3. Where it can detect: the test fails on a machine whose bytes differ
from the pin, so it compares the machines that run `cargo test`: CI's
`check` job on `ubuntu-latest` (`.github/workflows/ci.yml:101-127`), its
software-Goldens job on `ubuntu-24.04` (`:157-187`) and the M4 Pro. The
Windows job runs `cargo check` and `clippy` only (`:130-156`), so the RTX
2070 SUPER's build, the one machine of ADR 0013's two that runs Windows,
is compared only when its owner runs `cargo test` there by hand.

##### Block A2: the LOD chain's counts

`uv_sphere(radius, 256, 128)` emits two triangles per quad but one at each
pole row (`src/scenes/mod.rs:1455-1471`): 2 × 256 × 128 − 2 × 256 =
65,536 − 512 = 65,024 triangles, the test's LOD 0. `uv_sphere(1.0, 48, 24)`
gives 2 × 48 × 24 − 96 = 2,208. With 2,208 faceted tetrahedra of 4
triangles each, LOD 0 holds 2,208 + 8,832 = 11,040; the tetrahedra cannot
collapse (each corner is three vertices of three normals), so LOD 1, asked
for 5,520, keeps at least 8,832 and, to pass the 85% rule, at most
0.85 × 11,040 = 9,384, the test's `8_832..8_832 + 552`, 552 being a
quarter of the sphere's 2,208. LOD 2 would keep the 8,832 again, over 85%
of LOD 1, so the chain ends at 2 LODs. With 4,968 tetrahedra LOD 0 holds
2,208 + 19,872 = 22,080 and LOD 1 at least 19,872 / 22,080 = 0.90 of it,
past 0.85, so it is dropped and the chain is `[22_080]`. The first loop's
lower bound `finer / 2 − 1` follows from `finer.len() / 6 * 3`
(`src/cook/mesh.rs:364`) and a collapse removing two triangles. All
integer: floor 0. The comparison `simplified.len() as f32 > finer.len()
as f32 * 0.85` (`:371`) rounds: 0.85 in f32 is 0.85000002384, so a LOD of
exactly 85% is kept; no case of the test sits on that boundary (9,384 is
the bound's excluded end).

##### Block A3: texel density of the quad

`ground_plane(0.5, r)` (`src/scenes/mod.rs:1495-1514`) has corners at
±0.5, UVs at 0 and r, indices `0, 1, 2, 0, 2, 3`. Each triangle's cross
product has length 1 (two edges of 1 m at a right angle) and its UV
`perp_dot` magnitude r², all exact in f64 for r = 1, 2. The sums are 2 and
2r², the ratio r², whose `sqrt` is exactly r (`f64::sqrt` is correctly
rounded), and `as f32` keeps 1.0 and 2.0. Floor 0.

##### Block A4: the mip a sample reads

`level_of(16, t / 16)` computes `(t / 16) * 16`, exact for each t of the
test (all are dyadic or ∞), then `log2`, `ceil`, `max(0)`. t = 0 gives
`log2(0) = −∞`, `max` 0; t = 0.5 gives −1, then 0; t = 1.5 gives 0.585,
then 1; t = 3 gives 1.585, then 2; t = 100 gives 6.64, then 7, clamped to
4 (the 16-texel side has mips 0 to 4); t = ∞ saturates and clamps to 4.
t = 1 and t = 4 sit on a boundary: they need `log2f(1.0) = 0` and
`log2f(4.0) = 2` exactly, else `ceil` lands one level coarser. No standard
requires `log2f` to be exact at powers of two, though the common libms
are; the test does not say it rests on that. `from_the_third` at
t = 3 / 16 gives `first_level` 2 and clamps every finer level to it.

##### Block A5: the 225 Chunks

The Chunks whose nearest point lies within 2,000 m of the camera fill the
Minkowski sum of a 2,000 m disc and a 256 m square: 256² + 4 × 256 ×
2,000 + π × 2,000² = 65,536 + 2,048,000 + 12,566,371 = 14,679,907 m², or
224.0 Chunks of 65,536 m². The test's 225 agrees within the one Chunk the
grid's alignment to the camera moves. The other counts are integers the
generator fixes: floor 0.

### the world's cook: Chunks, the dev launch's cook and Far proxies

Scope: `src/cook/chunk.rs` (405 lines), `src/cook/launch.rs` (557),
`src/cook/far_proxy.rs` (833) and `src/cook/far_proxy/bake.rs` (638),
2,433 lines, every line read in full at the frozen commit `e5c55a8d`, their
`#[cfg(test)]` modules included; all four files are new since M3.5's
frozen commit `8bc34eb1`. Opened where these files call into them or read
their constants: `src/cook/mod.rs:130-534` (`Sources`, `GeneratedFarProxy`,
`Sources::chunk`, `far_proxy`, `source_material`, `cook_far_proxy`,
`far_proxy_mesh`, `may_be_kept`) and `:615-934` (`cook_sources`,
`read_sources`, `is_fresh`, `dangling_references`), which are reader A's
file, cited here only where a clause of these four files runs through them;
`src/cook/mesh.rs:25-60`, `:160-290`, `:435-453` (`LOD_KEPT_INDICES_CEILING`,
`encoder_settings`, `cook`, `lod_0`, `push_lod`, `texel_density`);
`src/cook/texture.rs:15-30`, `:95-337`, `:545-560` (`ENCODER`,
`FASTEST_BC7_PROFILE`, `cook_key`, `mips`, `hash_cook_settings`, `Mips`,
`linear_to_srgb`); `src/assets/cooked.rs:36-70` and `:883-960` (the Chunk
constants, `chunk_origin`, `chunk_holding`, the Chunk Blob's payload);
`src/scenes/mod.rs:142-157`, `:346-376`, `:405-440`, `:556-580`, `:726-744`
(source files, the Chunk and Far proxy Asset IDs, static content, Load
radius and View distance); `src/scenes/world.rs:645-700` (the quarter-turn
rotations the hash test's Chunk holds) and `:860-960` (the far field's fit
test, reader D's); `src/camera/` for `FIELD_OF_VIEW`;
`src/streaming/io.rs:814-847`, `:1425-1470`, `:1515-1530` and
`src/streaming/chunks.rs:155-196` (where a streamed load reads Blobs without
the launch's cook); `src/state.rs:83-143`, `:190-230`. ADRs read whole,
status lines included: 0026, 0018, 0011, 0014, 0027; 0017, 0013, 0016,
0019, 0020, 0002 and 0025 for the clauses that name Chunks, Far proxies, the
dev launch, the cook or the far field. Spec stories read: M4 (`spec-611.md`)
6 to 18, 56 to 66; M5 (`spec-804.md`) 16 to 32 and the Implementation and
Testing Decisions. Crate sources opened: `meshopt-0.7.0/src/simplify.rs:77-102`
and its vendored `vendor/src/simplifier.cpp:2489-2490`, `:2621-2622` (what
`result_error` means under `ErrorAbsolute`), `vendor/src/meshoptimizer.h:15`
(`MESHOPTIMIZER_VERSION 1030`); `glam-0.34.3/src/f32/neon/quat.rs:884-899`,
`sse2/quat.rs:881-897`, `neon/vec3a.rs:255-258`, `neon.rs:50-56` (whether
`Quat * Vec3` rounds alike on the M4 Pro and on CI's x86-64). `Cargo.lock`
pins `meshopt` 0.7.0, `glam` 0.34.3 and `intel_tex_2` 0.5.0. Where a number
needed arithmetic past a hand calculation, a Python script recomputed it in
IEEE double and single precision (no project code was run); each such number
says so. `deviations.md` has no row at a site in these files.
`DIFFERENCES.md`'s "Far proxies against the Chunks' own Meshes" (a roof a
pixel off past the Load radius) is a known difference, not a Finding. No
`cargo` command was run.

#### Clause verdicts

Paths are from the repo root; `far_proxy.rs` is `src/cook/far_proxy.rs`,
`bake.rs` `src/cook/far_proxy/bake.rs`, `launch.rs` `src/cook/launch.rs`,
`chunk.rs` `src/cook/chunk.rs`, `mod.rs` `src/cook/mod.rs`.

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0026 | Decision, "One Far proxy per Chunk ... draws its Far proxy, one Instance, in place of its Placements. Under it the Instance slots, the static TLAS and the flat cull keep M4's capacities" | falls in streaming and render: the cook makes one Mesh per Chunk coordinate (`far_proxy_id`), which streaming places | `mod.rs:676-694`; `src/scenes/mod.rs:353-357` |
| 0026 | "A first Primitive of merged, simplified geometry under one Material baked for it, windows included, and a second Primitive of the Chunk's ground tiles, unsimplified, under the ground's own Material" | holds: `cook` splits the Chunk's Placements by `placement.mesh == ground`, merges and simplifies the rest into the shell under the baked Material, and appends each ground tile's vertices unsimplified under its Mesh's own Material, refusing a tile whose Placement overrides it | `far_proxy.rs:225-226`, `:274`, `:281-302`, `:316-319` |
| 0026 | "Only a scene with a Far field cooks proxies; it names its ground's Mesh" | holds: a launch maps Far proxy IDs only for a scene whose `far_field_ground` is `Some`, `cook_sources` skips the rest, and `far_proxy_mesh` refuses a scene without one; the naming falls in reader D | `launch.rs:78-84`; `mod.rs:677-679`, `:477-479`; `src/scenes/mod.rs:373-376` |
| 0026 | "drops a Placement when every vertex of it lies within one pixel of one other Placement's surface, that Placement the Chunk's own or one of its eight neighbours'" | holds: `within_another` tests each of the Chunk's own Placements (the first `parts.len()` surfaces) against each single other surface, every corner within `reach_metres` of some triangle of it; the neighbours' Placements that can reach the Chunk join as surfaces, never as candidates | `far_proxy.rs:123-159`, `:240-262` |
| 0026 | "One pixel is its size at the Load radius at 1440p and a 45° field of view, 14.7 cm at 256 m" | holds: 256 × 2 × tan(22.5°) / 1440 = 0.147276 m, `FIELD_OF_VIEW` = π/4, `world`'s Load radius 256 m; Block B2 | `far_proxy.rs:24`, `:28-30`; `src/camera/mod.rs:90`; `src/scenes/mod.rs:569` |
| 0026 | "It welds what is left on position and simplifies it by edge collapse to one pixel of error, with no triangle count asked; the error is the simplifier's own measure" | holds: `generate_vertex_remap` on positions alone, then `meshopt::simplify` with target count 0, the pixel as target error and `ErrorAbsolute`, whose reported error is in metres (`simplifier.cpp:2489`, `:2621-2622`) | `far_proxy.rs:161-178`, `:267-274` |
| 0026 | "It refuses a proxy of more than 5,832 triangles outside its ground" | holds: `TRIANGLE_CAP` = 5832 = 324 × 18, checked on the simplified shell before the ground joins, the message naming the count | `far_proxy.rs:21`, `:275-279` |
| 0026 | "Two Channels, albedo and metallic-roughness, a texel two pixels at the Load radius, 29.5 cm, in one atlas: one rectangle a planar face ... with 2 texels of gutter a side on a grid of 4, dilated, so no chart bleeds into another down to mip 2" (as amended 2026-11-18) | holds: `TEXEL_PIXELS` 2 (0.29455 m), `GUTTER_TEXELS` 4 / 2 = 2, `GRID_TEXELS` 8 / 2 = 4; each chart's padded block starts on the grid, so a mip-2 texel never spans two blocks and a UV held inside its chart samples mip 2 no farther out than its block's outer texel centres (Notes) | `bake.rs:15-20`, `:93-95`, `:197-223`, `:304-307` |
| 0026 | "The Chunk's Placements and its neighbours', those dropped from the geometry included, are rasterised into each chart along its normal with a depth test, the outermost surface winning, at 4 × 4 samples a texel, on the CPU" | holds: `Transfer::surfaces` is every surface, dropped ones included; `tile` keeps the greatest height along the chart's normal per sample, `SAMPLES_PER_SIDE` 4 | `far_proxy.rs:346-353`; `bake.rs:16`, `:384-452` |
| 0026 | "A Mesh Blob per Chunk and, where its first Primitive has a triangle, a Material Blob and two Texture Blobs, each under an Asset ID made of the scene and the Chunk's coordinates, cooked together by `praxis cook <scene>` and by a dev launch that loads any of them while one is missing or stale" | holds: `cook_far_proxy` writes the two atlases and the Material only when the shell charted (`transfer` is `Some`), then the Mesh; a launch maps all four IDs to one `FarProxy` entry, so loading any of them cooks the set; the IDs fall in reader D | `mod.rs:425-468`, `:676-694`; `launch.rs:78-84`, `:184-201`; `src/scenes/mod.rs:353-370` |
| 0026 | "They share one Cook key, which covers the Chunk's and its neighbours' Cook keys, the Cook keys of the Meshes they place, of the Materials those draw with and of those Materials' albedo and metallic-roughness Textures, and the format and settings of each Blob" | holds: the key hashes the Mesh, Material and Texture formats and settings (the encoder strings and `far_proxy::settings`, which carries every bake constant and the albedo's BC7 profile), the ground's ID, then each of the nine Chunks' keys or a 0 byte, each placed Mesh's key, each Material's key (overrides and every Primitive's) and each of `BAKED_CHANNELS`' Texture keys; the metallic-roughness Texture's key covers its normal Texture's | `mod.rs:289-369`; `far_proxy.rs:32-38`; `bake.rs:23-27`; `texture.rs:135-147` |
| 0026 | "A proxy is fresh when its Mesh is in the Cooked cache under that key and, unless that Mesh names no Material, the three baked Blobs are too" | holds | `mod.rs:185-192` |
| 0026 | "A proxy whose sources name a Material or a Texture the cook did not read is kept as it is, as a Chunk is" | holds: `kept` is set by an unread Material, an unread Texture or a kept Chunk among the nine, and both cooks then write nothing | `mod.rs:320`, `:344-346`, `:355-358`; `launch.rs:188-190`; `mod.rs:681-683` |
| 0026 | The swap, Which are Resident, Past the View distance | falls in streaming and render | |
| 0026 | Settled by the cook, "The LOD chain ... cut on the welded positions, each coarser LOD simplified from LOD 0 to twice the error limit of the one before, and ends at eight LODs or at one that keeps more than 85% of the one before" | holds: each LOD is `simplified(&positions, &finest, pixel × 2^d)` for d = 1 to 7, and the chain stops at an empty LOD or one whose index count passes `LOD_KEPT_INDICES_CEILING` (0.85) of the last kept LOD's | `far_proxy.rs:329-344`; `mesh.rs:31`; `src/assets/cooked.rs:42` |
| 0026 | "each LOD's faces get flat normals and their own vertices, shared with a finer LOD where they are equal" | holds: `faceted` gives each LOD a vertex per position and quantised face normal; `push_lod` reuses a finer LOD's vertex of identical bytes in the same Primitive (reader A's) | `far_proxy.rs:180-206`, `:338-342`; `mesh.rs:238-289` |
| 0026 | "A LOD error is what the simplifier reports" (with Consequence line 64) | holds: the reported error of each LOD's simplification from LOD 0 is stored as its error, in metres | `far_proxy.rs:331-332`, `:342` |
| 0026 | "The pivot. A proxy's pivot is its Chunk's centre on the ground, and the cooker refuses a proxy with a vertex more than a Chunk's edge from it" | holds: the pivot is (128, 0, 128) m from the Chunk's origin; the bound is checked on LOD 0's vertices, and every coarser LOD indexes a subset of the same welded positions | `far_proxy.rs:220`, `:320-328` |
| 0026 | "The ground is no surface to drop against" | holds: ground tiles are partitioned out before the surfaces are built, and neighbours' ground tiles are filtered out | `far_proxy.rs:225-232`, `:242` |
| 0026 | "Dropped against the dropped. A Placement is dropped against any other Placement, one that is itself dropped included" | holds: `within_another` never removes a dropped surface from the candidates' targets | `far_proxy.rs:151-158` |
| 0026 | "Open land. A Chunk with nothing but ground cooks a proxy whose first Primitive is empty" | holds: no parts give no corners, an empty simplification, no atlas and no Material; the test loads `world`'s Chunk (0, 0) and asserts an empty first Primitive without a Material | `far_proxy.rs:238-240`, `:313-319`; `bake.rs:238-240`; `launch.rs:458-459` |
| 0026 | Settled by the bake, "A planar face is the triangles of the first Primitive that share vertices, which carry a position and a face normal" | holds: `faces` joins triangles by shared vertex indices, and `faceted` keys each vertex on its position's bits and its normal in steps of 1/512 | `bake.rs:168-195`; `far_proxy.rs:189-196` |
| 0026 | "Its chart is the least-area rectangle about it along one of its edges, a texel two pixels at the Load radius, its rows no longer than its columns" | holds: `Chart::fit` takes the least-area rectangle over every edge of the face's triangles, swapping axes so that rows ≤ columns | `bake.rs:108-142` |
| 0026 | "Charts are packed on shelves, tallest first, each with 2 texels of gutter a side on a grid of 4, into the smallest square atlas of a power-of-two side that holds them; a proxy whose charts pass 8,192 texels a side fails its cook" | holds: a stable sort by rows, shelves, sides doubling from 4 to `MAX_TEXTURE_SIDE` (8192), then an error; "the smallest that holds them" is the smallest this shelf packer fills | `bake.rs:197-223`; `src/assets/cooked.rs:36` |
| 0026 | "Coarser LODs. Each face of a coarser LOD takes the chart nearest it among those facing within 45° of it, or among all of them where none does, its UVs held inside that chart" | holds: `LIKE_NORMAL_COSINE` = 1/√2, inclusive; nearest by the summed distance of the face's vertices to the chart's rectangle; UVs clamped into it | `bake.rs:21`, `:157-165`, `:278-302` |
| 0026 | "What a chart reads. A source is a triangle of a Placement of the Chunk or of a neighbour that reaches it, one dropped from the geometry included, facing along the chart's normal within two pixels of its plane ... The outermost wins" | holds: a positive projected area is a triangle facing along the normal; `REACH_PIXELS` 2; the per-sample height must lie within the reach and above the sample's last | `bake.rs:17`, `:404-417`, `:437-448` |
| 0026 | "A Placement the runtime does not draw, one without a Material whose Mesh has a Primitive with triangles that names none, is no source" | holds for the bake: `placed` gives such a Placement's triangles no Material and `tile` skips them; its geometry still joins the shell (Notes) | `far_proxy.rs:65-69`; `bake.rs:401-403` |
| 0026 | "A sample reads its source's Material as its factors times its albedo and metallic-roughness Textures, at the finest mip whose texels are no smaller than the sample, as the Texture cook filters it, so a roughness its normal Texture widens is read widened" | holds: `SourceMaterial::shade` multiplies the linear factors by the bilinear sample; `Mips::level_of` is ceil(log2(UV per sample × side)); `TextureSource::mips` runs the Texture cook's own `filtered_levels`, Toksvig included | `bake.rs:69-90`, `:418-419`; `texture.rs:222-272`, `:298-313`; `mod.rs:384-420` |
| 0026 | "A texel is the mean of its covered samples in linear light; one no source covers, a gutter's included, is dilated from its neighbours. A chart no source covers at all, and the texels outside every chart, hold the mean of the baked texels as stored, which mips past mip 2 blend in" | holds: `mean` over covered samples in linear light; a breadth-first dilation over eight neighbours inside the padded block; the fill is the integer mean of every tile's stored sRGB bytes (truncated, Notes) | `bake.rs:52-59`, `:454-498`, `:355-364` |
| 0026 | "A source Texture's repeat is baked. Where a face ends within half a texel ... of the repeat's edge, its last texels blend in the far edge, as the source's own sampler does" | holds: `Mips::sample` filters bilinearly with `rem_euclid` wrapping (reader A's function, called by the bake) | `texture.rs:315-336`; `bake.rs:76-79` |
| 0026 | "The baked Material. Its factors are 1 for base colour, roughness and metallic, and it names no normal, occlusion or emissive Texture. Its Textures cook through the Texture cook, BC7 and BC5 with every mip ... The albedo atlas encodes at the BC7 encoder's `ultra_fast` profile" | holds: `CookedMaterial` of unit factors naming only the two atlases; each atlas a generated `TextureSource` cooked through `TextureSource::cook` with `with_fastest_bc7`, which changes only the BC7 (albedo) encode; the launch test asserts BC7 sRGB and BC5 with every mip | `mod.rs:440-462`; `texture.rs:106-113`, `:162-218`; `launch.rs:438-446` |
| 0026 | "No area, no face. A triangle without area has no normal to chart by and is left out of the proxy" | holds | `far_proxy.rs:185-188`; `bake.rs:631-636` |
| 0026 | Settled by the draw, "Which draws", "Its shadows", "Its mips", "The ground past the View distance" | falls in streaming and render | |
| 0026 | Settled by the draw, "Open land. A Primitive without triangles needs no Material" | holds, cook side: the empty shell names `None`; the draw falls in render | `far_proxy.rs:315` |
| 0026 | Consequence, "ADR 0014's 128 m pivot bound exempts a Far proxy's Mesh" | holds: `far_proxy::cook` builds through `mesh::lod_0` and `push_lod`, never `mesh::cook`, whose 128 m check (`mesh.rs:165-174`) it bypasses, and checks a Chunk's edge instead | `far_proxy.rs:316`, `:320-328` |
| 0026 | Consequence, "A streamed scene whose Chunk keeps more triangles than the cap at one pixel fails its cook, naming the count" | holds: `praxis cook` returns the error; a launch logs it per request (`io.rs:1524-1529`) | `far_proxy.rs:275-279`; `mod.rs:691` |
| 0026 | Consequence, "A dev launch that loads a Far proxy hashes the albedo and metallic-roughness Textures its sources draw with, loaded or not, to find its Cook key, and bakes the proxy where a Blob of it is missing or stale" | holds | `launch.rs:184-200`; `mod.rs:341-361`, `:373-380` |
| 0026 | Consequences, the Material table's count apart and loads capped by count; ADR 0013's and ADR 0020's names | falls in reader C, streaming and render | |
| 0018 | status, "A dev launch cooks only what it loads: each Asset as its load finds the Blob missing or stale, with the Assets it names" | broken in part: a Chunk the launch keeps (a Placement names a Mesh it did not read) returns before cooking any Asset it names, and a streamed load reads those Blobs from the cache unchecked: L1-C05 | `launch.rs:172-178`; `src/streaming/io.rs:1437-1466` |
| 0018 | status, "`praxis cook` and every dev launch also cook each Reference scene's static content into Chunks" | holds: `cook_sources` cooks every Chunk of the scenes read; a launch cooks each Chunk it loads | `mod.rs:661-675`; `launch.rs:172-183` |
| 0018 | Cook key, "SHA-256 over what the Asset cooks from ..., the Cook keys of its dependencies, every cook setting, the format version and the encoder versions", for a Chunk | holds: the Chunk's key hashes `CHUNK_MAGIC`, `CHUNK_FORMAT_VERSION`, zstd's version number and level, and the payload, which is all the Chunk cooks from (Placements, lights, and the bounds its Meshes' boxes give); a Mesh edit that leaves its box unchanged changes no Chunk byte, so no Mesh key is needed | `chunk.rs:98-109`; `src/assets/cooked.rs:890-953` |
| 0018 | Cook key, "Each dev launch recomputes the keys of its scene's Materials and Meshes as it starts, of each Chunk and Texture as it loads it, and, as it loads a Far proxy, of the Textures that proxy bakes from" | holds: `CookingLoader::new` reads the sources, whose Material and Mesh keys `read_sources` computes; a Chunk's key on `Sources::chunk`; a Texture's on first need, cached | `launch.rs:49-59`, `:172-183`, `:202-206`; `mod.rs:813-866`, `:373-380` |
| 0018 | Consequence, "The cooker warns when an ID that something references no longer exists", for the launch | holds for the launch: an ID absent from `entry_by_id` warns even when the cache holds a Blob for it, unless `may_be_kept` (a source absent) excuses it, which `dangling_references`' own comment records as its ceiling; this narrows M3.5's L1-C01 for the launch | `launch.rs:146-148`, `:236-241`; `mod.rs:913-934`; test `launch.rs:524-556` |
| 0018 | "The runtime loads Blobs only (ADR 0011). Each Blob starts with a magic and a format version", for Chunk and Far proxy Blobs | holds: a Chunk Blob is `CHUNK_MAGIC`, `CHUNK_FORMAT_VERSION` and one zstd frame; a Far proxy is an ordinary Mesh Blob and its atlases ordinary KTX2 Texture Blobs | `src/assets/cooked.rs:54-56`, `:955-960`; `mod.rs:451`, `:465` |
| 0018 | "A Far proxy's baked albedo alone encodes at its `ultra_fast` profile" | holds: both atlases are flagged, and the flag reaches only the BC7 encode; the metallic-roughness atlas is BC5 | `mod.rs:450`; `texture.rs:179-206` |
| 0018 | "A Chunk's edge is 256 m (4 by 4 cells of ADR 0014's 64 m grid; 40 by 40 Chunks cover 10 km)" | holds for the edge (`CHUNK_EDGE_METRES`); the 40 by 40 falls in reader D | `src/assets/cooked.rs:57-58` |
| 0018 | "A v1 Chunk is its own Asset, named from its scene and its integer coordinates, holding its Placements, its point lights and its bounds" | holds: `CHUNK_FORMAT_VERSION` 1; the name falls in reader D | `src/assets/cooked.rs:56`; `chunk.rs:67-78`; `src/scenes/mod.rs:346-350` |
| 0018 | "A Placement holds a Mesh Asset ID, an optional Material Asset ID ..., an f64 translation from the Chunk origin, and f32 rotation and scale"; "A light holds an f64 translation, a colour, an intensity in candela (ADR 0023) and a range in metres" | holds: `split` moves each translation from the World's origin to its Chunk's in f64 and keeps every other field as given | `chunk.rs:48-64`; `src/assets/cooked.rs:912-951` |
| 0018 | "Each Placement and light belongs to the Chunk that holds its pivot, and the bounds grow to hold every Placement's Mesh, which may reach past the Chunk's edge" | holds: `chunk_holding` floors x / 256 and z / 256 (a power of two, so exact; a pivot on an edge belongs to the Chunk at its +x, +z side); `chunks` grows the bounds by the eight placed corners of each Placement's source box, rotated in f64 | `src/assets/cooked.rs:66-70`; `chunk.rs:19-42`, `:66-78` |
| 0018 | "The Blob stores each field for all Placements, then for all lights, before zstd" | holds; level 9 | `src/assets/cooked.rs:890-953` |
| 0011 | "a development launch reaches glTF only through the cooker, which cooks what changed before the runtime loads Blobs" | holds per Blob: each Blob a launch loads through `CookingLoader`, or requests through the IO pool, is cooked first where stale (L1-C05 aside); drift on the wording, which reads as a cook ahead of every load: L1-C06 | `docs/adr/0011-gltf-authoring-cooked-runtime-formats.md:3`; `launch.rs:1-2`, `:244-283`; `src/streaming/io.rs:814-819`, `:1524-1529` |
| 0011 | "the runtime loads Blobs with mip chains, GPU block compression (BC) and byte ranges sized for streaming", for a Far proxy's atlas | holds: BC7 and BC5 with every mip through the Texture cook | `mod.rs:449-451`; `launch.rs:443-446` |
| 0014 | "cooked meshes stay within 128 m of their pivot ... A Far proxy's Mesh is exempt: it holds a whole Chunk about its centre, within a Chunk's edge of it, which the cooker enforces instead; it is cooked to be placed unrotated" | holds: see ADR 0026's pivot row; the unrotated placement falls in streaming | `far_proxy.rs:220`, `:320-328` |
| 0014 | World positions in f64, narrowed to f32 only in a local frame | holds: `from_pivot` rotates a source vertex in f32 (within its Mesh's 128 m) and composes the Chunk offset, the translation and the rotated offset in f64 before narrowing; Chunk bounds are composed in f64 | `far_proxy.rs:51-54`, `:251`; `chunk.rs:30-37` |
| 0026 | Settled by the cook, "f32 resolves 0.015 mm between 128 and 256 m" | holds: 2^7 × 2^-23 = 2^-16 m = 0.0153 mm; a quarter pixel at 256 m is 0.147276 / 4 = 36.8 mm | `far_proxy.rs:28-30` |
| 0027 | "A Far proxy holds no Thin Primitive, so the rule ends at the Load radius" | holds: a Thin Primitive's triangles merge into the shell as ordinary triangles, and `faceted` builds `Geometry::default()`, whose `thin_radius_metres` is `None` | `far_proxy.rs:68-94`, `:180-181`; `mesh.rs:190-205` |
| 0017 | Arena shares "as `helmet`'s, `town`'s and the world scene's Meshes fill them ... with the Far proxies of its densest far field: the 282 within the View distance and a Chunk's edge of a camera on a 64 m grid" | falls in reader D: the fit test is `src/scenes/world.rs:860-960`, fed by `far_field_contents` (`mod.rs:541-582`, reader A's) | |
| 0017 | "a second count of 285 entries, which only a Far proxy's Material takes ... and the baked Textures' bytes stay in the texture pool's Budget" | falls in reader C; the cook's share is that each atlas is a square power-of-two side, BC7 or BC5 with every mip, a shape the Texture classes take as any Texture's | `bake.rs:197-223`; `mod.rs:449-451` |
| 0017 | "the Primitive's UV texel density (stored at cook time, per Primitive ...)", for a Far proxy | holds: `lod_0` computes the shell's density from its atlas UVs in f64, and each coarser LOD carries LOD 0's (reader A's code); the launch test reads it back as a texel of two pixels | `mesh.rs:204`, `:278`, `:435-453`; `launch.rs:447-448` |
| 0013 | "2 km LOD'd View distance"; world size and Chunk grid | falls in reader D and streaming; the cook reads only the Load radius, for the pixel | `far_proxy.rs:28-30`; `mod.rs:296` |
| 0016 | the IO pool reads, the main thread loads | not applicable to the cook's own code; the launch's cook runs on the main thread before each request reaches the IO pool (streaming's code) | `src/streaming/io.rs:814-819`, `:1524-1529` |
| 0020 | "a per-Chunk sky-visibility field" | not applicable: planned, nothing in these files cooks it | |
| 0002, 0025 | one vertex and one index buffer; MIT, nothing unpublishable | not applicable: these files allocate no GPU buffer and adapt no outside code; none cites a source and licence | |
| checklist | "The runtime loads cooked Blobs only and references Assets by Asset ID, never by source path" | holds: Placements, Far proxies and baked Blobs are named by Asset ID; `CookingLoader` reads `assets/` only under the `cook` feature, as ADR 0011 allows a dev launch | `chunk.rs:48-64`; `launch.rs:49-59`; `src/state.rs:207-216` |
| checklist | "A cooked Mesh Blob is the same bytes on every machine: the two hash tests ... pass on each" | holds as far as this reader can see: the Far proxy's hash test pins one Chunk's Mesh Blob; whether it passes on each machine is the verifier's (Block B1 for what it can and cannot see) | `far_proxy.rs:815-825` |
| checklist | GPU buffers that never grow, the load staging, the GPU memory ledger, no GPU resources on ECS components, the slot order of prepare | not applicable: none of these files creates, uploads to or writes a GPU resource; the cook's own refusals (the triangle cap, the 8,192-texel atlas) are cook-time | `far_proxy.rs:275-279`; `bake.rs:222` |

#### Spec stories

| Story | Verdict | Where |
|---|---|---|
| M4 story 6, each Chunk cooked as one Blob holding its Placements and point lights | holds | `chunk.rs:19-64`; `src/assets/cooked.rs:900-960` |
| M4 story 7, a Placement names its Mesh by Asset ID, f64 translation from the Chunk's origin, f32 rotation and scale | holds: `split` rebases the translation in f64 and keeps the rest | `chunk.rs:53-57` |
| M4 story 8, a point light with an f64 translation and photometric values | holds: rebased in f64, colour, candela and range kept as the generator gives them | `chunk.rs:58-62` |
| M4 story 11, a Placement belongs to the Chunk that holds its pivot, the bounds grown to hold its Meshes | holds; test `chunk.rs:168-328` checks every cooked Mesh box of `town` inside its Chunk's bounds | `chunk.rs:19-42`, `:66-78` |
| M4 story 12, the cook warns when a Placement names an Asset ID no longer defined | holds for the launch, against what this cook makes, not the cache; a scene with an absent source passes every cached ID (`mod.rs:919-921`, reader A's ceiling) | `launch.rs:146-148`, `:236-241`; test `launch.rs:524-556` |
| M4 stories 15 and 16, `town` and every Reference scene load their static content through Chunks | holds, cook side: the tests load every scene's Chunks and find each Placement and light once, no static entity spawned; the Goldens fall in tooling | tests `chunk.rs:168-328`, `:363-404` |
| M4 story 56, a generated 40 × 40 world cooked by `praxis cook` | falls in reader D and reader A (`cook world` in `mod.rs`); `chunk.rs:333-357` checks each of `world`'s 1,600 Chunks generated alone has its cut's Cook key | test `chunk.rs:333-357` |
| M4 story 63, CI cooks only what its frames load | holds, mechanism side: the launch cooks each Blob on its first load and nothing before; the CI workflow falls in tooling | `launch.rs:117-131`; test `launch.rs:346-404` |
| M4 story 64, a dev launch cooks a Chunk and the Assets it names when its load misses or finds them stale | broken in part: a kept Chunk's named Assets are not cooked: L1-C05 | `launch.rs:172-183` |
| M5 story 16, each Chunk cooked a Far proxy: Placements merged, small parts dropped, simplified to a pixel at the Load radius under a triangle cap, one Blob under its own Asset ID | holds | `far_proxy.rs:212-355`; `mod.rs:425-468` |
| M5 story 17, a Far proxy's Material baked from its sources', albedo and metallic-roughness, "one texel a pixel at the Load radius" | superseded by ADR 0026 as amended 2026-11-18: a texel two pixels, `TEXEL_PIXELS` 2; the two Channels hold | `bake.rs:15`, `:329-380` |
| M5 story 26, the fit test extended to the Far proxies of the densest 2 km view | falls in reader D (`src/scenes/world.rs:860-960`); these files supply each proxy's Arena elements and atlas side through `far_field_contents` | `mod.rs:541-582` |
| M5 stories 18 to 25, 27 to 32 (the draw, the swap, residency, shadows, the fill) | fall in streaming and render; nothing in these files implements them | |
| M5 Implementation Decision, Far proxy: "`meshopt` 0.6.2's `simplify` runs to one pixel under a cap of 5,832 triangles ... with gutters clean to mip 3" | superseded: `meshopt` 0.7.0 since the frozen commit itself, and ADR 0026's amendment of 2026-11-18 (texel two pixels, gutters clean to mip 2); the cap and the pixel hold | `Cargo.lock` (`meshopt` 0.7.0); `far_proxy.rs:21`, `:161-178`; `bake.rs:15-19` |
| M5 Testing Decision, "Captures and Goldens render Settled frames; Settled waits for the Far proxies within the View distance" | falls in streaming and tooling | |

#### Registry keys

No `[ref:<key>]` is cited in these four files. The rows below are the keys
the code relies on uncited, through ADR 0026, ADR 0018 or the function it
calls.

| File | Keys | Verdict |
|---|---|---|
| `far_proxy.rs:161-178` | `meshoptimizer-1.3-simplify-error` (ADR 0026:9) | matches: the quote's "setting `target_index_count` to 0 will simplify the input mesh as much as possible within the specified error limit" is `simplify(indices, &adapter, 0, error_metres, ...)`, and "an approximate measure of the deviation" is ADR 0026's own caveat; the quote's "normalized to `[0..1]` range" is the default mode, which `SimplifyOptions::ErrorAbsolute` replaces with metres, as `simplifier.cpp:2489` (`error_scale = ... ? vertex_scale : 1.f`) and `:2622` (`*out_result_error = sqrtf(result_error) * error_scale`) show; the registry has that flag only as `meshopt-0.6.2-simplify-options`, a version `Cargo.lock` no longer pins (Notes). Version: key names meshoptimizer 1.3 vendored by `meshopt` 0.7.0; `Cargo.lock` pins `meshopt` 0.7.0 and `meshoptimizer.h:15` reads `MESHOPTIMIZER_VERSION 1030 /* 1.3 */` |
| `far_proxy.rs:180-206`, `:338` | `meshoptimizer-1.3-simplify-original-vertices` (ADR 0026:20) | matches: "The resulting index buffer references vertices from the original vertex buffer" is why each LOD indexes the welded `positions` and is re-faceted |
| `far_proxy.rs:271-273` | `meshoptimizer-1.3-vertex-kinds` (ADR 0026:20) | matches: "more than one vertex maps to this one ... result[i] = Kind_Locked" is the locking a face-by-face box meets in a Mesh's own chain; welding on position alone leaves every position one vertex, so no corner locks |
| `bake.rs:43-47` | `unity-6000-uv-distribution-metric` (ADR 0018, texel density) | matches the form ADR 0018 records: the square root of UV area over triangle area, the inverse of Unity's "triangle area / uv area" taken as a length; per triangle here, where the stored density is per Primitive, which the function's doc names as its unit, not its extent |
| `bake.rs:499-503` via `texture.rs:552-558` | `khronos-dfs-srgb` | matches: "R′ = R × 12.92, R ≤ 0.0031308; 1.055 × R^(1/2.4) − 0.055, R > 0.0031308" is `linear_to_srgb`, applied once to the linear mean before the byte |
| `bake.rs:75-90`, `:364` | `gltf-2.0-metallic-roughness` | matches: "The factor value is a linear multiplier for the corresponding texture components" is `red * base_red` and `roughness * self.roughness` on linear samples; "Its green channel contains roughness values and its blue channel contains metalness values" is the atlas pixel `[0, roughness, metallic, 255]` |
| `bake.rs:304-307`, `:369-372` | `gltf-2.0-texture-origin` | matches: "(0, 0) corresponds to the upper left corner" is a UV of the chart's texel over the side with row 0 written first (`LoadedImage`, "top row first") |

#### Oracles

Every test in these four files is new since `8bc34eb1`. All run on the CPU,
so no WGSL floor applies; the floors are f32 (u = 2^-24) and f64 (u = 2^-53)
chains, platform `tan`, `sin`, `cos` and `powf`, glam's SIMD paths and
meshoptimizer's C++.

| Function | Test | Expected value and its source | Constant | Floor | Stated floor | Verdict |
|---|---|---|---|---|---|---|
| `far_proxy::cook` through `far_proxy_mesh` and `CookedMesh::to_blob`, the densest Chunk (-19, -3) | `the_densest_chunks_proxy_cooks_to_the_same_blob_on_every_machine`, `far_proxy.rs:815-825` | SHA-256 `5341731a...c3e4` of the Mesh Blob, pinned across machines (CI's x86-64 and the M4 Pro) | exact | 0 where every operation is deterministic across the two; Block B1 lists what can move it and what it cannot see | no: the message says only that a machine that alone disagrees cooks other bytes | exact |
| `pixel_metres` | `the_densest_chunks_proxy_keeps_the_cap_...`, `far_proxy.rs:734` | 0.1473 m, ADR 0026:9's "14.7 cm at 256 m" to four digits; derived 0.147275938 m | `1e-4` | f32 rounding of the f64 chain, 1.7e-9 m; libm `tan` cannot move the f32 result (Block B2) | no | explained: measured error 2.41e-5 is the four-digit rounding of the expected value; the constant is 2× the largest such rounding (5e-5) |
| `Atlas::chart_shell` and `lod_0`'s `texel_density`, the densest Chunk | same test, `far_proxy.rs:736-737` | a 2048² atlas at a texel of `texel_metres(0.1473)` = 0.2946 m: ADR 0026:30, "charts into 2048²" | `1e-3` on texels per metre × texel size − 1 | 1.63e-4 from the test's four-digit pixel (0.2946 / 0.29455188 − 1), plus at most ~1e-5 from f32 UVs summed in f64 (Block B3) | no | explained, 6× margin |
| LOD 1 of the densest Chunk | same test, `far_proxy.rs:738-741`, `:754` | ADR 0026:20: LOD 1 within twice the pixel, ≤ 85% of LOD 0's indices, 324 buildings' flattened roof tops of 6 index vertices each, the ground's 8 triangles | `2.0 * PIXEL_METRES` = 0.2946; integer counts exact | 0.29455188 is the simplifier's own limit, which `result_error` meets to sqrt-rounding (~3e-8); the counts are integers | no | explained (slack 4.8e-5 over the limit, ADR's reported 0.27 is 0.025 under); counts exact |
| `far_proxy::cook`, LOD 0 against its sources | same test, `far_proxy.rs:797-806` | every seventh source corner within one pixel of the shell: ADR 0026:9 | `PIXEL_METRES` = 0.1473 | the simplifier's limit is 0.14727594, 2.4e-5 under the constant; a distance computed in f32 at 128 to 190 m from the pivot rounds by several ulps of 1.53e-5 (Block B4); and the simplifier's error is approximate, a tenth under the true distance on `world`'s roofs (ADR 0026:64) | no | tight: a corner the simplifier moved by its full limit could read past the constant; it passes because LOD 0 of this Chunk moves no corner that far |
| `distance_to_triangle` | `a_point_is_as_far_from_a_triangle_as_from_its_nearest_face_edge_or_corner`, `far_proxy.rs:419-427` | hand derivation: 0.25 over the face, 3.0 to an edge, 5.0 to a corner, √2 to the hypotenuse | exact for three; `1e-6` for √2 | 0: every intermediate is exact in f32 and both sides round `sqrt(2.0)` alike (Block B5) | no | exact for three; loose for the fourth (constant 1e-6 over a floor of 0), harmless |
| `within_another` and the shell's merge | `a_placement_within_a_pixel_of_anothers_surface_is_left_out_and_no_other_is`, `far_proxy.rs:429-501` | hand derivation of the kept corners in millimetres: ADR 0026:9, :22, :23 (Block B6) | exact (millimetres) | 0: no kept or dropped case lies within f32 rounding of the pixel; the nearest margin is 0.06 m against 0.1473 | no | exact |
| `Transfer::bake` and `Atlas` UVs | `a_baked_proxy_reads_the_outermost_source_along_its_faces_normal_a_neighbours_too`, `far_proxy.rs:609-673` | sRGB EOTF of bytes 200, 40, 30 (`khronos-dfs-srgb-eotf`): 0.5776, 0.0212, 0.0130, times the factors; roughness 0.8 × 128/255, metallic 0.5 × 64/255 | `0.01` per component | 8-bit sRGB storage of the baked texel, 0.0014 at most on these points; and at (2.3, 2.6) the source's own bilinear transition at the brick's red-blue edge, which spills into the atlas texel the point reads: 0.0089 in blue (Block B7) | no | tight at (2.3, 2.6): 0.0089 of 0.01; the other points within 0.0045 |
| `Transfer::bake`, gutters and fill | `no_chart_bleeds_into_another_down_to_mip_2`, `far_proxy.rs:675-716` | each plate's flat colour and roughness at its corners and centre at mips 0, 1, 2: ADR 0026:10 | `0.01` | 8-bit storage: albedo 0.8 → byte 231 → 0.7991 (8.8e-4); metallic 0.5 → byte 128 → 0.5020 (1.96e-3); roughness 0.2 to 0.8 exact; mips of identical texels in one block are exact | no | explained, 5× margin; a bleed would read tenths off |
| `Chart::fit` and `pack` | `each_face_has_a_chart_of_its_own_on_the_grid_with_its_gutter`, `bake.rs:555-594` | hand derivation: a 256-texel atlas; each chart ceil(side / 0.1473) by its longer side first; the UV diagonal the quad's metres (Block B8) | exact for the side, the sizes and the grid; `1e-3` m on the diagonal | 0 for the integers (no side within rounding of a whole texel: nearest 0.3 / 0.1473 = 2.037); ~3e-6 m on the diagonal (f32 UV at 1/256 of a 256-texel side, a 0.3 rad turn in f32) | no | exact; loose on the diagonal (~300×) |
| baked texel density of `world`'s farm Chunk (-20, -11) | `loading_a_far_proxy_cooks_it_...`, `launch.rs:447-448` | a texel of two pixels: texels per metre × 0.1473 = 0.5 (ADR 0026:10) | `1e-3` | 8.2e-5 from the four-digit pixel (0.1473 / 0.29455188 − 0.5), the density's f64 sum negligible | no | explained, 12× margin |

Set aside as behaviour tests, judged and not oracles: in `chunk.rs`, the
three tests that compare the Chunk cut with the generator, the cut with each
Chunk generated alone, and every scene's spawn with its Chunks (their
bounds tolerance of 1e-9 m compares the code's formula with itself); in
`launch.rs`, the four tests of what a launch cooks and warns of; in
`bake.rs`, `a_coarser_lods_face_reads_the_nearest_chart_facing_its_way_held_inside_it`
(chart indices) and
`a_face_longer_than_the_largest_atlas_fails_and_a_triangle_without_area_has_no_face`;
in `far_proxy.rs`, `the_same_chunk_cooks_the_same_blob_twice`.

##### Block B1: the Far proxy's hash test

What the bytes cover: the Mesh Blob of one Chunk's proxy (`far_proxy.rs:718-725`):
the shell's welded and simplified positions, its flat normals (as
QTangents), its atlas UVs, so the charting, the packer's order and each
chart's position; the ground's tiles; the LOD chain's indices, errors and
Meshlets; the bounds; the zstd frame. It does not cover the bake: no atlas
texel, no Texture Blob, no Material Blob; and it covers one Chunk of 1,600.

The chain and what can move it across machines:

1. The Placements: `world::static_content` turns buildings by
   `Quat::from_rotation_y(quarter_turns × π/2)` (`src/scenes/world.rs:659`)
   and windows by `rotation_arc` products (`:682`, `:688`). `from_rotation_y`
   takes a platform f32 `sin_cos`, not correctly rounded across libms; a
   one-ulp difference in sin(π/4) moves a rotated vertex. The test can see
   this, since the densest Chunk holds rotated buildings.
2. `from_pivot` (`far_proxy.rs:52`): `Quat * Vec3` is `mul_vec3a` in glam
   0.34.3. On aarch64 (`neon/quat.rs:884-899`) and on x86-64
   (`sse2/quat.rs:881-897`) both evaluate `rhs·(w² − |b|²) + b·(2 rhs·b) +
   (b × rhs)·2w` with the same operation order: `length_squared` and `dot`
   are scalar `(x·x + y·y) + z·z` on NEON (`neon/vec3a.rs:255-258`) and
   `dot3_into_m128`'s pairwise sum on SSE2, each `(x + y) + z`; rustc does
   not contract. So the two targets round alike here, and the test could
   not tell them apart if they did not: it is the scalar path, which no
   target here builds, that rounds differently. The f64 sum and the
   narrowing are IEEE-exact operations.
3. `generate_vertex_remap`, `simplify` (`far_proxy.rs:271-274`, `:331-332`):
   meshoptimizer's C++, built with `-ffp-contract=off` for
   `aarch64-apple-darwin` and `x86_64-unknown-linux-gnu`
   (`.cargo/config.toml:5-6`). The test sees a contraction on any target
   without that flag.
4. `faceted`, `Chart::fit`, `nearest_texel` (`far_proxy.rs:184-196`;
   `bake.rs:108-165`): scalar `Vec3` cross, dot, `normalize` (IEEE `sqrt`)
   and `ceil`; deterministic.
5. `pixel_metres` (`far_proxy.rs:29`): f64 `tan`, which cannot move the f32
   result (Block B2).
6. `HashMap` use: `faceted`'s `vertex_of`, `within_another`'s grid and
   `reach_of_mesh` are lookups only; output order follows the triangle and
   Placement order. No thread order enters the Mesh.
7. `mesh::lod_0` and `push_lod`: reader A's chain, under `mesh.rs`'s own
   hash test.

Floor: 0 across the two machines for steps 2, 4, 5 and 6; the test detects
steps 1 and 3 and reader A's chain on this one Chunk. The bake's
cross-machine determinism (its `powf` in `linear_to_srgb`, `texture.rs:556`,
and the BC7 encoder) is outside it, as nothing claims the Texture Blobs are
the same bytes on every machine.

##### Block B2: the pixel

`pixel_metres(256.0)`, `far_proxy.rs:29`:
`FIELD_OF_VIEW` = f32(π/4) = 0.785398185253143 (`src/camera/mod.rs:90`),
widened to f64 and halved: 0.392699092626572. `tan` of it: 0.414213575175865
(libm on this host, 1.9e-4 ulp from the 60-digit value 0.41421357517586482...,
recomputed). 256 × 2 = 512 (exact), × tan = 212.077350, / 1440 =
0.147275937840307 (f64), narrowed to f32: 0.14727593958377838 (`0x3e16cf81`).
The f64 value lies 1.74e-9 below the f32 value, 0.117 of an f32 ulp
(2^-26 = 1.49e-8), so 0.38 ulp from a rounding midpoint, against an f64
error of about 1e-17: the same f32 for `tan` one ulp either way (recomputed
for both neighbours). Against the test's 0.1473: 2.41e-5.

##### Block B3: the atlas's texel density

`texel_density` (`mesh.rs:435-453`) sums mesh and UV areas in f64 from f32
positions and UVs. A UV is `(position_texels + held) / 2048` (`bake.rs:306`),
an f32 below 1 with ulp ≤ 6e-8, so ≤ 1.2e-4 texel; over a chart of tens of
texels the relative error of a triangle's UV area is ≤ ~1e-5, and the sum
averages it. The UV map is isometric within each chart (orthonormal
`across` and `down`, `bake.rs:115-122`), so the density is
1 / (0.29455188 × 2048). The test multiplies by 2048 × 0.2946:
0.2946 / 0.29455188 = 1.000163.

##### Block B4: LOD 0 within a pixel

`distance_to_triangle` (`far_proxy.rs:106-121`) on corners up to 190 m from
the pivot (ADR 0026:21): differences of coordinates near 128 m carry an
ulp of 1.53e-5 m, then a cross product, a dot and a division by a length,
so a distance at the limit rounds by a few times 1.5e-5. The test's
`PIXEL_METRES` (0.1473) sits 2.4e-5 above the simplifier's limit
(0.14727594), about 1.6 ulps; and ADR 0026:64 says the simplifier's
reported error runs a tenth under the distance moved. A corner moved to the
limit can therefore read past the constant. The test passes because the
densest Chunk's 5,832 triangles are the cap itself (324 buildings of 18
triangles: four wall quads and a five-quad roof slab,
`src/scenes/world.rs:407-445`), so the merge keeps every triangle and moves
no corner.

##### Block B5: `distance_to_triangle`'s four points

Triangle (0,0,0), (4,0,0), (0,0,4); its normal (4,0,0) × (0,0,4) = (0, −16, 0).
(1, 0.25, 1): over the face; |(1, 0.25, 1)·(0, −16, 0)| / 16 = 4 / 16 = 0.25,
exact. (2, 0, −3): nearest edge z = 0, along = 8 / 16 = 0.5, point (2,0,0),
distance 3, exact. (7, 4, 0): edge (4,0,0) to (0,0,4), along = −12/32,
clamped to 0, so the corner (4,0,0); distance √(9 + 16) = 5, exact.
(3, 0, 3): outside, hypotenuse along = 16/32 = 0.5, point (2,0,2), distance
√2, computed as `sqrt(1 + 0 + 1)`, the same f32 operation as `2f32.sqrt()`:
difference 0.

##### Block B6: the drop rule's kept set

Pixel 0.1473 m; the pane is `upright_quad(1, 1, 0.06)`, so a pane placed at
z stands at z + 0.06. Wall: x 98 to 102, z = 100. Pane at (100, 1, 100):
z = 100.06, 0.06 from the wall, every corner over it: dropped. Pane at
(100, 1, 100.2): 0.26 off: kept, `quad([99.5,1],[100.5,2],100.26)`. Pane at
(110, 1, 100): 7.5 m past the wall's edge: kept. Pane at (101.8, 1, 100):
corners at x = 102.3 lie √(0.3² + 0.06²) = 0.306 from the wall's edge: kept.
Pane at (0.5, 1, 50): x 0 to 1, z = 50.06; the west neighbour's wall at
x = 255 − 256 = −1 spans x −3 to 1 at z = 50: 0.06 off, dropped against a
neighbour. Wall turned π/4 at (60, 0, 60): its normal (sin π/4, 0, cos π/4);
the pane at (60.5, 1, 59.94) stands at z = 60, x 60 to 61; its corner
(61, 1, 60) lies (1, 0)·(0.707, 0.707) = 0.707 off: kept, and the turned
wall's corners (60 ± 2·0.7071, y, 60 ∓ 2·0.7071) are its expected
`turned_wall`. Plate 0.05 above the ground: kept, the ground no surface.
In millimetres from the pivot (x − 128, z − 128), each expected corner is an
f32 within 1e-5 of a whole millimetre; no rounding flips one.

##### Block B7: the bake's outermost read

Computed by this reader in a 1-D model (rows are identical across the
brick), in IEEE double with the f32 sRGB table, recomputed in Python. The
wall spans x 0 to 4 m with UVs 0 to 1, the brick 16 texels wide, red texels
0 to 7, blue 8 to 15. The chart's texel is 2 × 0.1473 = 0.2946 m from x = 0
(the first edge, +x, is the rectangle's `across`, `bake.rs:114-125`), four
samples a side. The source sampler (`texture.rs:317-336`) blends red into
blue between texel centres 7.5/16 and 8.5/16, x 1.875 to 2.125 m. Atlas
texel 7 spans x 2.0622 to 2.3568; its first sample column, at x = 2.0990,
reads 0.104 red; the texel's mean blue is 0.5629 linear, stored as byte
198, read back 0.5647. The point x = 2.3 sits at texel coordinate 7.807, so
bilinear weights 0.693 on texel 7 and 0.307 on texel 8 (0.5776): blue
0.5687, 0.0089 under 0.5776. The other points: (1.0, 0.5), (1.2, 1.5),
(0.7, 2.6) red 0.0014 under (byte storage); (3.7, 0.5) blue 0.0045 under
(texel 13 holds the wall's last 0.17 m, the repeat's wrap to red within
it); (3.0, 0.5) within 1e-4. So the constant 0.01 holds at every point, at
(2.3, 2.6) by 0.0011.

##### Block B8: the chart test's atlas

At 0.1473 m a texel: chart sizes ceil(4/0.1473) × ceil(3/0.1473) = 28 × 21;
(9 m, 2 m) turned to rows ≤ columns, 62 × 14; 204 × 3; 7 × 7; 48 × 17; 34 × 14.
Padded (+ 2 × 2 gutter, up to a multiple of 4): 32 × 28, 68 × 20, 208 × 8,
12 × 12, 52 × 24, 40 × 20. Tallest first, ties in index order: 28, 24, 20
(quad 1), 20 (quad 5), 12, 8. At side 128 the 208-texel chart does not fit;
at 256 the first shelf takes 32 + 52 + 68 + 40 + 12 = 204 columns, the
208-wide chart opens a second shelf at row 28, ending at row 36 ≤ 256. Side
256, as asserted.

### assets: the Cooked cache reader, the loader, the pools and their Arenas

Scope: `src/assets/pools.rs` (2,277 lines), `material.rs` (2,054),
`cooked.rs` (1,556; the reader's half: `AssetId::new`, every `from_blob`
and its helpers, `resident_tail_level`, `level_bytes`, the Texture
decoders, `CookedCache`; the `to_blob` writers are reader A's and were read
only as the format the reader parses), `loader.rs` (655), `mesh.rs` (64)
and `mod.rs` (12); 6,618 lines, every file read in full at `e5c55a8d`,
`#[cfg(test)]` modules included. ADRs read whole, status lines and
amendments included: 0002, 0011, 0017, 0018, 0026; read for the clauses
these files implement: 0001, 0003, 0013, 0020, 0021, 0025. Specs read whole,
stories, implementation and testing decisions: the M4 spec and the M5
spec; rows are given for the 26 M4 and 9 M5 stories these files implement
or partly implement. Opened outside the
files, only as far as a clause needs them: `bench/budgets.json` (the
Budgets the capacities derive from), `src/stats/gpu_memory.rs:107`
(`RESIDENT_TAIL_LEVEL_BYTES`), `src/streaming/io.rs:915-925` (which pool
counters reach Frame stats), `src/render/acceleration_structures.rs:454`,
`:861` (the callers of `handle_at`), `src/render/staging.rs` (the load
staging's `write`, `write_texture`, `copy_texture`, `build_blas`). Crate
sources read in the cargo registry: `meshopt` 0.7.0 (`src/encoding.rs:171-186`,
`vendor/src/vertexcodec.cpp:125-146`, `:505-535`, `vendor/src/meshoptimizer.h:15`,
version 1030, "1.3"), and `offset-allocator` 0.2.0 where an Arena claim
rests on it. `docs/audit/deviations.md` holds no row in these files;
`DIFFERENCES.md` lists no image these files produce. The one known open
issue in these files, the texture pool's Channels capped in Texture
classes rather than bytes, is under *Open issues read*. No `cargo` command
was run. Every `where` is `path:line` at `e5c55a8d`, under `src/assets/`
unless given in full.

#### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0002 | "Meshes are (vertex offset, index offset, index count) sub-allocations of one shared vertex buffer and one shared index buffer in the Mesh pool" | holds: one vertex Arena and one index Arena, each one `wgpu::Buffer`; a Mesh keeps its vertex range and its index range, and its BLAS build names both offsets and LOD 0's index count | `pools.rs:242-255`, `:514`, `:607-613`, `:664-669`, `:691-705` |
| 0002 | "ADR 0017 replaces growth with fixed arenas sized from the Budget, sub-allocated by an offset allocator, each at most 2^31 − 1 bytes" | holds: each Arena's buffer is created once, an `offset_allocator::Allocator` of its elements sub-allocates it, and creation asserts fewer than 2^31 bytes | `pools.rs:1327-1356`, `:1337-1340` |
| 0002 | "each Arena is now sized at device creation from the geometry Arenas' GPU memory Budget, in the share ADR 0017 gives it" | holds: `from_budgets` takes each Arena's per-mille of `geometry_arenas`, rounded down to `COPY_BUFFER_ALIGNMENT`; 658, 130, 11, 133 and 68 per mille are ADR 0017's 65.8%, 13.0%, 1.1%, 13.3% and 6.8% | `pools.rs:19`, `:75-93`; `bench/budgets.json:11` |
| 0002 | "Refined by ADR 0019: the Mesh table names each Mesh's Meshlet ranges in three further Arenas ...; the vertex Arena still holds the vertices, and the index Arena feeds only the BLAS builds and refits" | holds: three Meshlet Arenas, a `MeshData` per Mesh with each LOD's Meshlet and triangle ranges; the index Arena is `BLAS_INPUT` only and no pass binds it | `pools.rs:248-273`, `:529-573`, `:596-600`; `mesh.rs:36-49` |
| 0017 | status: "each Channel is a partially bound binding array of texture arrays, one per Texture class of Textures sized to their source" | holds: a `TextureViewArray` of one `D2Array` view per class per Channel; a class's array is created at the Texture's own width, height, mip count and format | `material.rs:418-475`, `:1296-1303` |
| 0017 | status: "loaded from cooked Blobs through a `StagingBelt` and sampled trilinear, the colour Channels at 16x anisotropy and the data Channels at 4x" | holds: every level through `LoadStaging::write_texture`; two samplers, `Linear` on all three filters, `anisotropy_clamp` 16 and 4 | `material.rs:39-40`, `:362-365`, `:374-376`, `:598-599`, `:1272-1283` |
| 0017 | status: "a Texture a Chunk loads arriving as its Resident tail, its finer mips streaming in ..., while one a spawn loads keeps its full mip chain" | holds, reader side: `resident_tail_from_blob` decodes the tail alone, `from_blob` every mip, and `push` places whatever the reader gave; which reader a load uses falls in streaming | `cooked.rs:376-384`; `material.rs:348-391` |
| 0017 | status: "a Far proxy's atlas arriving with its mips down to the one its distance from the camera needed ..., drawn from its tail only where its Channel has no room for that chain" | holds: `down_to_from_blob` decodes from the finer of the picked mip and the tail's first; `push` places the tail, then the chain, and keeps the tail alone when the chain finds no room | `cooked.rs:389-397`; `material.rs:361`, `:370-372` |
| 0017 | status: "with the eviction below making room for a move into a full Channel" | holds: `stream_mips` calls `evict_for` when the target class has no room | `material.rs:1048-1059` |
| 0017 | status: "geometry loads through the same belt into one vertex and one index arena ..., which refuse a Mesh that does not fit ..., and into three Meshlet arenas" | holds: every Arena write is `LoadStaging::write`; a short Arena returns `MeshRefusal::ArenaShort` with every Arena as it was | `pools.rs:495-509`, `:574-600`, `:1297-1315`, `:1374-1377` |
| 0017 | status: "at device creation the pools derive their capacities from them, each Arena and table at its share ..., and each Channel a Texture class per 64 KiB of the texture pool's fifth, up to what the Capability floor binds" | holds: `texture_pool / (5 × RESIDENT_TAIL_LEVEL_BYTES)` = 13,107, then `.min(CLASSES_PER_CHANNEL)` = 65,536 / 5 = 13,107 (Block C1) | `pools.rs:97-100`; `material.rs:19-20`; `src/stats/gpu_memory.rs:107`; `src/gpu/mod.rs:301` |
| 0017 | "an array per Texture class: the Textures of one shape, size, mip count and format alike, none padded or resampled to share one" | holds: `TextureShape` is width, height, mip count and format; `place` matches on the whole shape | `material.rs:94-110`, `:173-215` |
| 0017 | "A Layer names a class and a slice of its array, Layer 0 stays the neutral default" | holds: `class * SLICES_PER_CLASS + slice`; each Channel's neutral 1×1 Texture is placed first, class 0 slice 0, never released; the normal Channel's neutral is now `Rgba16Float` (0.5, 0.5, 1, 1), exact, which closes M3.5's L1-C03 | `material.rs:136-141`, `:526-559`, `:592-596` |
| 0017 | "A class's slice count is fixed when it opens, at most the Capability floor's 256 array layers, and a full class's shape opens another" | holds: the first class of a shape has one slice, each later one twice the newest's, capped at `SLICES_PER_CLASS` = `FLOOR_TEXTURE_ARRAY_LAYERS`; the array is created at that count once (Block C7) | `material.rs:24`, `:199-214`, `:430-449`; `src/gpu/mod.rs:305` |
| 0017 | "an empty slice still holds memory, which `gpu_memory_bytes` counts" | holds: the array is charged at creation for all its slices through `create_material_texture` | `material.rs:430-448`; `src/stats/gpu_memory.rs:169-186` |
| 0017 | "A Texture whose resident mips change moves to the class of its new shape by copy" | broken in part: `stream_mips` writes the new mips into the target slice and copies the Resident ones after them, and `evict_finest_mip` copies the chain from its next mip into the coarser class, but a new shape whose mip 0 is not whole blocks opens a class wgpu refuses, a panic: L1-C08 | `material.rs:1064-1081`, `:1192-1201` |
| 0017 | "Metal's `useResource` ... and wgpu-core's tracking then follow the classes, not the Textures" | holds, pool side: one view per class is bound, however many slices it holds | `material.rs:451-473`, `:1296-1303` |
| 0017 | "Residency changes are batched so that bind group is rebuilt at most once per frame" | holds: placements set `views_changed`; `refresh_bind_group` rebuilds once if any did, called once at the start of `render` | `material.rs:472`, `:1241-1266`; `src/render/renderer.rs:813` |
| 0017 | the gate: "the rebuild ... measured at 10k textures" | holds, pool side: the rebuild is timed and logged with the bound class count | `material.rs:1246`, `:1255-1264` |
| 0017 | "Priority is computed on the CPU ... from distance and the Primitive's UV texel density (stored at cook time, per Primitive ...)" | holds, pool side: the reader reads each Primitive's density, and the Mesh pool keeps it per Primitive for mip priority; the priority falls in streaming | `cooked.rs:722`, `:759-766`; `pools.rs:516-520`; `mesh.rs:15-16` |
| 0017 | "The Materials rays reach off screen ... take the Resident tail as their floor, which the next consequence keeps loaded" | holds, pool side: a Texture's tail slice is released only when the Texture itself is freed | `material.rs:407-413`, `:787-801`, `:1082-1084`, `:1202` |
| 0017 | "Each texture's Resident tail (mips of 64 KiB and smaller) never leaves" | holds for the tail `resident_tail_level` picks; drift on the definition: a Texture whose first mip of 64 KiB or less is not whole blocks takes its whole chain as its tail: L1-C09 | `cooked.rs:253-269`; `material.rs:356-366` |
| 0017 | "When finer mips arrive, the Texture moves to the class of its mips from its finest Resident one and its tail's slice stays taken, charged twice in `resident_tails` (87,408 bytes for a 2,048² BC7 or BC5 Texture, 1.6% of its chain)" | holds: the chain's class repeats the tail's mips; the ledger charges every level of 64 KiB or less a slice to `ResidentTails`; 87,408 and 1.56% recomputed (Block C2) | `material.rs:370-390`; `src/stats/gpu_memory.rs:175-181` |
| 0017 | "When a request would exceed the texture pool's capacity, which is set from its Budget" | broken, known (see *Open issues read*) | `material.rs:199-201`, `:231-245`; `pools.rs:97-100` |
| 0017 | "the finest mip of the lowest-priority texture goes first, least recently used breaking ties" | holds: the victim minimises (`Reverse(finest_mip_needed)`, `last_used_plan`, Asset ID); `u32::MAX`, needed by nothing, goes first | `material.rs:1102-1118`, `:274-282` |
| 0017 | "a mip loads at priority p and is evicted only below p − 1" | falls in streaming: `rank` returns no rank unless the need is two mips past the finest Resident one (`mips.rs:291-293`), and `evictable` takes no Texture without a rank | `material.rs:1132` |
| 0017 | "the candidates are the Textures whose chain lies in the very class the request needs, whose slice it takes" | holds: `evict_for` keeps Textures of the Channel whose `mips_from(resident_level)` is the target shape; the released slice is the one `place` gives the move | `material.rs:1105-1107`, `:173-187`, `:1061` |
| 0017 | "the one evicted moves to the class of its mips from the next, or falls back to its tail only when it needs no finer mip than the tail's, so it never re-requests what it lost" | holds: `evictable` admits a Texture only when its need is at or past its tail's first mip or the next class has room; `evict_finest_mip` moves it a mip coarser when that class has room, else to its tail (Block C11) | `material.rs:1125-1139`, `:1183-1208` |
| 0017 | "a request is made only when its move finds room, free or by an eviction" | falls in streaming (the M5.5 report's L1-F01); the pool answers `full_class_for_mips` and `coarsest_evictable_needs` | `material.rs:1143-1168` |
| 0017 | "One vertex arena and one index arena, and the Meshlet arenas, each at most 2^31 − 1 bytes" | holds: asserted at creation; the checked-in largest is 55,197,040 bytes | `pools.rs:1337-1340`, test `:1411-1427` |
| 0017 | "sub-allocated with the `offset-allocator` crate" | holds: an `Allocator` per Arena, in elements; `allocation_size` is the requested count (`offset-allocator-0.2.0/src/lib.rs:246`), so `bytes_in_use` is exact; a request is refused when no free range's bin reaches its rounded-up size (see *Notes*) | `pools.rs:1352`, `:1360-1371` |
| 0017 | "the Capability floor's 128 MiB `max_storage_buffer_binding_size` caps that arena" | holds: 55,197,040 bytes against 134,217,728, pinned by a test | `pools.rs:1416`; `src/gpu/mod.rs:824` |
| 0017 | "No growth at runtime: a load that does not fit is refused and logged" | broken in part: the Arenas, the Mesh table and the Material table never grow and refuse with a log (`error!`, `warn!`); a Texture whose Channel has no room for its tail is logged and given Layer 0, not refused, and not counted: L1-C07 | `pools.rs:490-509`, `:1114-1143`; `material.rs:653-655`, `:679-690`, `:418-426`, `:985-989` |
| 0017 | "The CPU shadow of vertex and index data goes" | holds: a loaded `Mesh` keeps its vertex range, bounds, texel densities and Material Handles | `mesh.rs:6-20`; `pools.rs:510-522` |
| 0017 | "the vertex, index, Meshlet, Meshlet vertex and Meshlet triangle Arenas take 65.8%, 13.0%, 1.1%, 13.3% and 6.8% of the geometry Budget" | holds (Block C1) | `pools.rs:19`, `:82-85` |
| 0017 | "the Mesh table an entry per 5,124 bytes of it, `town`'s mean Mesh" | holds: 83,886,080 / 5,124 = 16,371 | `pools.rs:33`, `:94` |
| 0017 | "the Material table an entry per 19,267,584 bytes of the texture pool's, the largest Material of the Reference scenes, `helmet`'s, past its Resident tails" | holds: 3 × 5,505,024 + 2,752,512 = 19,267,584 for three 2,048² BC7 or BC5 chains and one BC4 chain past their tails; 222 entries (Block C2) | `pools.rs:38`, `:95` |
| 0017 | "a second count of 285 entries, which only a Far proxy's Material takes ...: neither count takes an entry of the other's, a load past either is refused, logged and counted" | holds: `count_capacities` and `entries_taken` per `MaterialUse`; `refused_as_full` warns and counts; a full count frees only its own unused Materials; one SSBO of 222 + 285 entries | `pools.rs:43`, `:96`; `material.rs:570-580`, `:616-617`, `:674-690`, `:752-757`, `:891-896` |
| 0017 | "the baked Textures' bytes stay in the texture pool's Budget" | holds: a Far proxy's Textures load into the Channels as any Texture | `material.rs:900-916`; `loader.rs:230-235` |
| 0017 | "The Renderer and its Pools create the buffers, textures and TLASes they count through one ledger, and charge it their BLASes' bytes" | holds: Arenas under `GeometryArenas`, the Mesh table and the SSBO under `SceneAndPassBuffers`, class arrays through `create_material_texture`, BLASes through one `Charge` under `AccelerationStructures` | `pools.rs:239`, `:274-283`, `:1342-1351`, `:999-1005`; `material.rs:570-580`, `:430` |
| 0017 | "What ADR 0013 leaves out, the GPU stats readback and the load staging, bypasses it" | holds, pool side: the pools write through `LoadStaging`, uncounted (the M5.5 report's L1-R04 is the drift on this clause) | `pools.rs:1374-1377`; `material.rs:659` |
| 0017 | "Acceleration structures join the ledger ..., each BLAS ... at the size its driver gives through wgpu-hal's build-sizes query" | holds, pool side: `blas_bytes` at load with the BLAS's own flags and geometry; the query is the render section's | `pools.rs:621-625`, `:683-684` |
| 0017 | "A compacted BLAS's size stays an estimate ...: the fraction of its built size compaction kept where measured" | holds: compacted bytes move from the uncompacted total as each BLAS is compacted and are charged at `compacted_blas_fraction`, rounded (Block C6) | `pools.rs:828-834`, `:999-1005` |
| 0017 | "Streaming uploads go through a `StagingBelt`, never `Queue::write_texture`" | holds: every pool write is `LoadStaging::write`, `write_texture`, `copy_texture` or `build_blas`; no `queue.write_*` in `src/assets/` | `pools.rs:574-600`, `:670`; `material.rs:364`, `:375`, `:378-385`, `:659`, `:1066`, `:1074-1081`, `:1193-1200`, `:1234` |
| 0017 | "A Mesh a streamed load brings that the pools lack does not hold its frame: the GPU builds and compacts its BLAS over the frames after, which the main thread finds by polling ...; a Settled frame and a spawn's loads still wait for theirs" | holds, pool side: `defer_blases`, `poll_blases`, `blas_ready`, `asset_blas_ready`, and `wait_for_blases` for a wait; when to place falls in streaming | `pools.rs:727-763`, `:775-798` |
| 0017 | "The cooked format ... stores each mip as a separately loadable byte range ..., and the Mesh Blob each Primitive's UV texel density" | holds, reader side: each level through its KTX2 level index entry, decompressed on its own; the density read per Primitive; the Blob is read whole (the M5.5 streaming section's note) | `cooked.rs:459-474`, `:722`; `:1146` |
| 0018 | status: "the runtime loads Materials and Textures ... and Meshes ... from them by Asset ID; the reader rebuilds their Meshlets ..., which the Mesh pool keeps in three Arenas the main pass draws from" | holds: `AssetLoader` and `AssetReader` take `AssetId` only; `CookedMeshlet::split` rebuilds each Meshlet; the status sentence now names the Meshlet Arenas, which closes M3.5's L1-C06 | `loader.rs:11-69`; `cooked.rs:534-588`, `:815-832`; `pools.rs:256-273` |
| 0018 | "An Asset ID is the first 128 bits of SHA-256 over (GUID, Asset kind, glTF name, Channel for a Texture)" | holds: unchanged since M3.5 but for the Chunk kind (3); the `shasum` oracle is not changed in scope, so it gets no row | `cooked.rs:107-142`, test `:1221-1229` |
| 0018 | "A cooked Asset is a Blob named by the SHA-256 of its bytes, found through a cook index that maps Asset ID to Blob" | holds, reader side: `index/<id>` holds the hash and Cook key, `blobs/<hash>` the bytes; the reader does not rehash what it reads (see *Notes*) | `cooked.rs:1119-1152` |
| 0018 | "The Cooked cache lives in git-ignored `cooked/` and is never evicted" | holds: no code in the module removes a Blob or an index file | `cooked.rs:1078`, `:1082-1185` |
| 0018 | "The runtime loads Blobs only (ADR 0011). Each Blob starts with a magic and a format version (a Texture Blob is KTX2, with its format version in a key-value entry). A mismatch is refused with a re-cook message; there is no migration code." | holds: `payload` for `PXMA` 2, `PXME` 8, `PXCH` 1; the KTX2 entry `PXTXVERSION` 2; each refusal says "re-cook with `praxis cook`"; no branch reads an older version | `cooked.rs:21-56`, `:1064-1075`, `:416-427`, `:204`, `:705`, `:965` |
| 0018 | "The runtime still opens no source path" | holds: `CookedCache` opens `cooked/index/<id>` and `cooked/blobs/<hash>` only | `cooked.rs:1119-1125`, `:1129`, `:1146` |
| 0018 | "Textures are KTX2 with every level zstd-supercompressed, stored smallest mip first" | holds, reader side: the reader requires `Zstandard` and takes each level by its index entry, so the byte order does not matter to it | `cooked.rs:429-432`, `:459-466` |
| 0018 | "Rows are stored unpadded, and the loader re-pitches them to 256 bytes as it writes into the `StagingBelt`" | holds: `levels` hold unpadded rows; `write_texture` re-pitches to `COPY_BYTES_PER_ROW_ALIGNMENT` | `cooked.rs:244-246`; `src/render/staging.rs:222-246` |
| 0018 | "Formats per Channel ...: albedo and emissive BC7 sRGB, normal BC5, metallic-roughness BC5 linear, occlusion BC4" | holds, reader side: a Texture Blob holds one of the three formats; which Channel takes which is the cooker's | `cooked.rs:272-285`, `:433-441` |
| 0018 | "The cooker refuses a Texture whose size is not a multiple of 4 or exceeds 8192" | falls in reader A; the reader mirrors it and requires the full mip chain | `cooked.rs:444-457` |
| 0018 | "a 28-byte vertex ... and u32 indices" | holds: asserted at compile time; indices decoded as `u32` | `cooked.rs:486-497`, `:802` |
| 0018 | "Meshlets of at most 64 vertices ... and at most 124 triangles" | holds: `split` refuses a run past either | `cooked.rs:548-551`, `:562-565` |
| 0018 | "A Meshlet is stored as one byte, its triangle count; the reader rebuilds its vertex remap and local triangles from its run of the index stream, which BLAS builds read" | holds | `cooked.rs:778-781`, `:815-832`, `:534-588`; `pools.rs:579-580` |
| 0018 | "The Blob also holds the Mesh's bounding box for culling" | holds: six f32 after the counts, into `MeshData` | `cooked.rs:715-718`; `pools.rs:529-533` |
| 0018 | "Each Primitive stores its UV texel density ... the inverse of Unity's per-Mesh average of triangle area over UV area [ref], taken per Primitive, as a ratio of summed areas and as a length" | holds, reader side: one f32 per Primitive; the registry difference this sentence records is L1-C10 | `cooked.rs:505-510`, `:722` |
| 0018 | "The Blob holds the Mesh's LOD chain: the vertices once, which every LOD indexes, then per LOD its error in metres, each Primitive's run of its indices and its Meshlets, with every LOD's indices in one encoded stream" | holds: one vertex stream, per LOD an error, index count and per-Primitive runs, one index stream split coarsest first; a LOD error must be finite and no less than the finer one's | `cooked.rs:734-776`, `:802-814` |
| 0018 | "a chain holds at most 8 LODs" | holds: a count outside 1 to 8 is refused | `cooked.rs:711-714`; `pools.rs:483-486` |
| 0018 | "Each Primitive stores a Thin Primitive's radius, zero for any other" | holds: zero reads as `None`; a radius not finite or below zero is refused | `cooked.rs:723-731` |
| 0018 | "A Material is its own Asset: PBR factors plus one Texture Asset ID per Channel. Primitives reference it by ID." | holds | `cooked.rs:160-169`, `:203-231`, `:504` |
| 0018 | "A Placement holds a Mesh Asset ID, an optional Material Asset ID ..., an f64 translation from the Chunk origin, and f32 rotation and scale ... A light holds an f64 translation, a colour, an intensity in candela ... and a range in metres ... The Blob stores each field for all Placements, then for all lights, before zstd" | holds, reader side: the column layout, its byte count checked against the counts before any column is cut | `cooked.rs:850-881`, `:964-1029` |
| 0018 | "A Chunk's edge is 256 m" | holds | `cooked.rs:58` |
| 0018 | Cook key, dev-launch cooking, the cooker's warning | falls in reader A and reader B | |
| 0011 | "the runtime loads Blobs with mip chains, GPU block compression (BC) and byte ranges sized for streaming" | holds, reader side: a Texture Blob must hold its full chain, in a BC format, with a level index entry per mip | `cooked.rs:433-457`, `:459-474` |
| 0011 | "a development launch reaches glTF only through the cooker, which cooks what changed before the runtime loads Blobs" | holds, reader side: nothing in `src/assets/` reads glTF; the cooking falls in reader B | `loader.rs:71-87` |
| 0011 | "Textures cook into KTX2, a standard container" | holds, reader side: parsed by `ktx2::Reader` | `cooked.rs:416-418` |
| 0026 | Consequence: "The Material table counts Far proxies' Materials apart" | holds: the first Primitive's Material of a Far proxy's Mesh takes an entry of the Far proxies' count, the ground's an ordinary one; a full Far proxies' count frees only unused Far proxies' Meshes | `loader.rs:151-153`, `:164-172`, `:230-234`; `material.rs:891-893` |
| 0026 | "A Far proxy's Mesh builds a BLAS from its LOD 0 at load, and the proxy is placed once that BLAS is ready, as a Chunk is" | holds, pool side: `MeshUse::FarProxy` is traced, its BLAS built from LOD 0's indices, compacted, and `asset_blas_ready` answers for it; the placing falls in streaming | `pools.rs:124-126`, `:137-139`, `:607-613`, `:759-763` |
| 0026 | "While drawn it holds one static TLAS entry under the mask bit an ordinary Instance has" | falls in render; the pool's part is `thin_radius_metres`, `None` for a Mesh with any Primitive that is not thin | `pools.rs:636-641`, `:864-869` |
| 0026 | "A Far proxy is read with its atlas's mips down to the one it needs from where the camera was when it was requested" | holds, reader and pool side (the ADR 0017 status row above) | `cooked.rs:389-397`; `material.rs:370-372` |
| 0026 | Open land: "A Primitive without triangles needs no Material, so the proxy of a Chunk with nothing but ground draws its ground" | holds: `needs_material_handle` counts only Primitives with triangles; tested | `pools.rs:511-513`; test `loader.rs:359-364` |
| 0026 | What it holds, its Mesh, its Material, its Blobs, the swap, which are Resident | falls in reader B (cook) and streaming | |
| 0020 | "each Mesh rays trace gets a BLAS at load, compacted unless it is loaded to be refitted or rebuilt each frame" | holds: `ALLOW_COMPACTION` unless `MeshUse::Refitted`, whose BLAS is `REFITTED_BLAS_FLAGS`, never queued for compaction, and is what the rebuild form rebuilds | `pools.rs:152-154`, `:614-620`, `:673-682`, `:710-717` |
| 0020 | "The `crowd` ... figures ... have no BLAS" | holds: `MeshUse::Drawn` is not traced: no index range, no BLAS | `pools.rs:115-117`, `:137-139`, `:651-653`, `:1271-1275` |
| 0020 | "Every BLAS is built from its Mesh's LOD 0, a refitted Mesh's included ...; the index Arena ... holds LOD 0's indices alone" | holds: `CookedMesh::indices` is LOD 0's; the BLAS's index count and the index Arena range are its length; every LOD's Meshlets load beside it; tested | `cooked.rs:591-597`; `pools.rs:607-613`, `:1270-1275`, `:534`; test `:1767-1770` |
| 0020 | "wgpu-core 30.0.2 records no barrier after `compact_blas`'s copy" (the compaction ordering) | holds: `compact_blases` waits through both stages, and a deferred BLAS is ready only once its compacted copy's submission is done | `pools.rs:765-798`, `:840-852` |
| 0001 | "Materials are entries in one structured GPU buffer, indexed by the Material ID that each Meshlet record carries ...; one bind group serves every material" | holds: one `materials_buffer` of `MaterialData`; `MeshletData::material_id` is the Handle's index or `NO_MATERIAL`; one `MATERIALS` bind group | `material.rs:478-484`, `:570-580`; `pools.rs:552`; `mesh.rs:58-64` |
| 0003 | "Whatever changes what an Instance holds makes prepare write its slot" | falls in render: the pools count their frees (`LoadedMeshes::frees`, `Generations::frees`), which prepare reads | `pools.rs:1034-1038`, `:1256-1259`; `src/render/prepare.rs:670` |
| 0013 | "each capacity is its share of its kind's Budget at its real bytes per entry, as ADR 0017 gives them"; the seven Budgets | holds: `from_budgets` reads `bench/budgets.json`'s figures, which are ADR 0013's | `pools.rs:75-107`; `bench/budgets.json:10-18` |
| 0021 | "`max_texture_array_layers` 256" (the floor a class's slices may reach) | holds: `SLICES_PER_CLASS = FLOOR_TEXTURE_ARRAY_LAYERS` | `material.rs:24`; `src/gpu/mod.rs:305`, `:315` |
| 0021 | "`max_binding_array_elements_per_shader_stage` at least 65,536" | holds: five Channels of 13,107 classes, 65,535 elements | `material.rs:19-20`; `src/gpu/mod.rs:301` |
| 0021 | "`max_buffer_size` at least 2^31 − 1 (ADR 0017 arenas)" | holds | `pools.rs:1337-1340`; `src/gpu/mod.rs:313` |
| 0025 | the repo is public under MIT; adapted code cites its source and licence | not applicable: no code in these files is adapted from elsewhere | |
| checklist | "The runtime loads cooked Blobs only and references Assets by Asset ID, never by source path" | holds | `loader.rs:11-69`; `cooked.rs:1138-1168` |
| checklist | "A cooked Mesh Blob is the same bytes on every machine" | falls in reader A (the writer and its hash tests) | |
| checklist | "no GPU buffer grows at runtime, and a request past a buffer's capacity is refused and logged" | broken in part: holds for the Arenas, the Mesh table and the SSBO; for a Channel's classes a Texture is substituted by Layer 0, logged, not refused or counted: L1-C07 | `pools.rs:490-509`; `material.rs:653-655`, `:985-989` |
| checklist | "Streaming uploads go through the load staging (`LoadStaging`...), never `Queue::write_texture`" | holds (the ADR 0017 row above) | |
| checklist | "GPU buffers, textures and acceleration structures are created through the GPU memory ledger (`GpuMemory`) under their kind" | holds (the ADR 0017 ledger row above); samplers and views hold no memory of their own | |
| checklist | "No GPU resources stored on ECS components" | holds: `MeshHandle` and `MaterialHandle` are an index and a generation | `mesh.rs:24-28`; `material.rs:45-49` |
| checklist | "Pool lookups return `Option<&T>`" | holds | `pools.rs:1012-1014`, `:1028-1032`; `material.rs:704-708` |

#### Spec stories

| Story | Verdict | Where |
|---|---|---|
| M4 story 6, each Chunk cooked as one Blob holding its Placements and point lights | holds, reader side: `CookedChunk::from_blob` reads both, refusing a count its bytes do not hold; the cook falls in reader B | `cooked.rs:964-1029`; test `:1446-1489` |
| M4 story 7, a Placement names its Mesh by Asset ID, f64 translation, f32 rotation and scale | holds, reader side | `cooked.rs:853-860`, `:986-1006` |
| M4 story 8, a point light in a Chunk carries an f64 translation and photometric values | holds, reader side: f64 translation, colour, candela intensity, range | `cooked.rs:865-870`, `:1007-1020` |
| M4 story 12, the cook warns when a Placement names an Asset ID no longer defined | falls in reader A and reader B | |
| M4 story 13, the Mesh Blob stores each Primitive's UV texel density | holds, reader side: read per Primitive and kept on the loaded `Mesh` | `cooked.rs:722`; `pools.rs:516-520` |
| M4 story 27, the Mesh, Material and Texture pools keyed by Asset ID, one entry per Asset however many Chunks load it | holds: `asset_handles` keyed by (Asset ID, `MeshUse`), `loaded_materials` and `loaded_textures` by Asset ID; a Mesh loaded for two uses takes an entry per use, and a bent Mesh one per Renderable, by design (tested) | `pools.rs:174`, `:141-148`, `:381-383`, `:402-411`; `material.rs:500-502`, `:884-890`, `:975-978`; test `pools.rs:1484-1524` |
| M4 story 28, generational Handles, so a stale Handle is refused | holds: `Generations` moves an index's generation on at each free; `get` and `free` check it; tested for Meshes and Materials (Block C10) | `pools.rs:1198-1260`, `:1028-1032`; `material.rs:694-708`; tests `pools.rs:1619-1688`, `material.rs:1542-1595` |
| M4 story 29, each pool entry counted by the loaded Chunks and entities that use it, freed only when nothing uses it | holds: users per Mesh and per Material; a loaded Mesh counts a user of each Material its Primitives name, and a Material of each Texture it samples; only an unused entry is freed | `pools.rs:305-353`, `:402-411`; `material.rs:823-834`, `:787-801`; test `loader.rs:487-564` |
| M4 story 30, a freed Mesh's Arena ranges returned to the offset allocator | holds | `pools.rs:922-945`, `:1367-1371`; test `:1808-1809` |
| M4 story 31, the moment an unused entry is freed set by survey 2's decision | holds: unused entries stay loaded, least recently used first, until a refused load needs room, at most `ENTRIES_FREED_PER_FRAME` = 64 a frame beside a full Material count's one-for-one frees | `pools.rs:45-51`, `:389-401`, `:419-442`; `material.rs:739-772`, `:891-893` |
| M4 story 32, each Channel a binding array of texture arrays, one array per shape | holds (the ADR 0017 rows) | `material.rs:94-110`, `:418-475` |
| M4 story 33, the floor kept at 256 layers per texture array | holds | `material.rs:24`, `:202-204` |
| M4 story 34, each Texture's Resident tail kept for as long as the Texture is loaded | holds; the tail's definition is L1-C09 | `material.rs:407-413`, `:1082-1084`, `:1202-1208` |
| M4 story 35, mips streamed by priority | falls in streaming | |
| M4 story 36, the Materials rays reach off screen given a floor priority | holds, pool side (the tail never leaves); the floor's measure falls in streaming | |
| M4 story 37, a Texture whose Resident mips change moved to its new class by copy through the staging belt | broken in part (L1-C08, a new shape not whole blocks); otherwise holds: new mips through the belt, Resident mips by `copy_texture` on the load staging's encoder, the Material Layers rewritten through the belt | `material.rs:1064-1086`, `:1216-1236` |
| M4 story 38, eviction drops the finest mip of the lowest-priority Texture first, least recently used breaking ties, a mip evicted only below its load priority minus one | holds: the order is the pool's; the hysteresis is streaming's rank (`mips.rs:291-293`) | `material.rs:1094-1139` |
| M4 story 41, the 8 GB split into Budgets per kind | holds, pool side: the pools read the geometry and texture pool Budgets | `pools.rs:75-102` |
| M4 story 43, every fixed capacity sized from its Budget at device creation | holds for the Arenas, the Mesh table, the Material table and the Texture classes; the Far proxies' count is ADR 0017's fixed 285, which no Budget sizes | `pools.rs:75-102`, `:43` |
| M4 story 44, GPU memory counted by kind where it is created | holds | `pools.rs:239`, `:274-283`, `:1342-1351`; `material.rs:430`, `:570-580` |
| M4 story 45, acceleration-structure bytes counted exactly | holds, pool side: built sizes by query, compacted ones by ADR 0017's measured fraction | `pools.rs:621-625`, `:999-1005` |
| M4 story 46, the vertex Arena's binding split only above 128 MiB | holds: 55,197,040 bytes, whole binding | `pools.rs:1416` |
| M4 story 47, a request past any capacity refused, logged and counted in Frame stats | broken in part: holds for Meshes and Materials (`take_refusals`, into `meshes_refused` and `materials_refused`); broken for a Texture whose Channel has no room for its tail: given Layer 0, logged, not counted: L1-C07. A mip move's refusal is the M5.5 report's L1-F02 | `pools.rs:393-397`, `:457`; `material.rs:679-690`, `:985-989`; `src/streaming/io.rs:924-925` |
| M4 story 53, the waits on BLAS compaction measured on streaming frames | falls in streaming and tooling; the pool's deferral is above | `pools.rs:727-798` |
| M4 story 69, Frame stats count pool entries freed | holds, pool side: `take_frame_frees` of both pools, the Material pool's including the Textures freed with a Material | `pools.rs:452-454`; `material.rs:768`, `:805-808`; `src/streaming/io.rs:920-923` |
| M4 story 71, GPU memory kept as a running total updated at load and eviction | holds, pool side: charges at creation and drop; an eviction moves slices within arrays already charged | `pools.rs:999-1005`; `material.rs:430` |
| M5 story 4, a LOD chain sharing its source's vertices, at most 8 LODs | holds, reader side: one vertex stream every LOD indexes, a count past 8 refused; the cook falls in reader A | `cooked.rs:711-714`, `:794-814` |
| M5 story 6, each LOD's error stored in metres | holds, reader side: read, checked finite and non-decreasing, written to `MeshData::lod_errors_metres` | `cooked.rs:738-743`; `pools.rs:572` |
| M5 story 12, every BLAS built from its Mesh's LOD 0, and the index Arena to hold LOD 0 only | holds; tested | `pools.rs:607-613`, `:1270-1275`; test `:1767-1770` |
| M5 story 16, a Far proxy cooked as one Blob under its own Asset ID | falls in reader B; the pool loads it as `MeshUse::FarProxy` | `loader.rs:151-153` |
| M5 story 22, at most 285 Far proxies Resident | falls in streaming; the Material table's Far proxies' count is 285 | `pools.rs:43` |
| M5 story 25, Far proxies' Materials counted apart from the Material table's other entries | holds; tested | `material.rs:296-302`, `:674-690`; `loader.rs:230-234`; tests `material.rs:1959-2053`, `loader.rs:287-381` |
| M5 story 27, Far proxies cast and receive the sun's shadows | holds, pool side: a Far proxy's Mesh gets a compacted BLAS from LOD 0 | `pools.rs:137-139`, `:607-620` |
| M5 story 33, a Primitive declared thin with a radius, without a byte per vertex | holds, reader side: one f32 per Primitive, refused when not finite or negative | `cooked.rs:723-731` |
| M5 story 35, a Thin Primitive drawn a pixel wide | holds, pool side: each LOD's Thin Primitives' Meshlets follow its others', with their radius in `MeshletData` and their count in `thin_triangle_counts`; tested | `pools.rs:537-571`; test `:1531-1614` |

#### Registry keys

| File | Keys | Verdict |
|---|---|---|
| `material.rs:29` | `wgpu-core-30-bound-view-needs-initialised` | matches: the quote pushes `NeedsInitializedMemory` over the view's `mip_range` and `layer_range`; `WHOLE_VIEW_ARRAY_BYTES`' comment says a bound view has the slices it spans initialised, which is why a class past 32 MiB is viewed only over the slices placed (`:451-458`) |
| `material.rs:30` | `wgpu-core-30-submit-initialises-textures` | matches: "the mips and layers a command buffer needs initialised and nothing wrote are cleared in that submit" is the comment's "the first submit that draws with a view bound has wgpu zero-fill the slices it spans that nothing wrote" |
| `cooked.rs:29`, `:310`, `:1302` | `ktx2-key-value-alignment` | matches: `PXTXVERSION` and its NUL are 12 bytes, a multiple of the u32 value's 4; the test checks the key length and the value's offset (`:1303-1310`); `:310` is the writer's comment, reader A's |
| `cooked.rs:306` | `ktx2-level-order` | falls in reader A (the writer's comment); the reader takes each level by its index entry (`:459-462`), so it relies on no order |
| `cooked.rs:307` | `ktx2-supercompression-per-level` | matches, reader side: "applied independently to each mip level"; one `zstd::bulk::decompress` per level, its capacity the level's expected size (`:465`) |
| `cooked.rs:308` | `ktx2-level-alignment` | falls in reader A (the writer packs levels unpadded); the reader requires `Zstandard` (`:429-432`), the scheme under which the quote's alignment is 1 |
| `cooked.rs:507` | `unity-6000-uv-distribution-metric` | differs: the quote is Unity's per-Mesh "Average of triangle area / uv area"; the code's density is per Primitive, the square root of summed UV area over summed mesh-space area; ADR 0018:24 records the difference and `deviations.md` has no row: L1-C10 |
| `cooked.rs:620`, `:786` | `meshoptimizer-1.3-vertex-block` | matches: `(8192 / 28) & ~15` = 288, capped at 256, is `VERTEX_CODEC_BLOCK_VERTICES`; `control_size = vertex_size / 4` = 7 bytes a block for codec 1, so `vertex_count.div_ceil(256) * 7` bounds the encoded bytes from below and admits every valid stream (codec 0 spends a header byte per byte of vertex, more). Version: `Cargo.lock:1369-1370` pins `meshopt` 0.7.0, whose `vendor/src/meshoptimizer.h:15` is version 1030, "1.3", and whose `vertexcodec.cpp:140-146`, `:719-721` hold the quoted lines |
| `pools.rs:2261` | `tracing-issue-3611` | matches: the quote's `has_just_one` short path takes the first registering thread's default; `logged` registers a standing `NoSubscriber` dispatch first so the capture is never the only one, and the test at `:2234-2246` warns from a thread without a subscriber first |
| uncited, `pools.rs:1317-1383` (cites ADR 0017) | `offset-allocator-tlsf`, `offset-allocator-0.2.0-port` (ADR 0017's) | matches: a TLSF-like allocator under MIT, `Cargo.lock:1816-1817` 0.2.0; the Arena's claims rest on `allocate`, `free`, `allocation_size` and `storage_report` (`offset-allocator-0.2.0/src/lib.rs:199-291`, `:495-517`) |
| uncited, `material.rs:24` (through `FLOOR_TEXTURE_ARRAY_LAYERS`) | `wgpu-types-30-max-texture-array-layers-default` (ADR 0017's) | matches: 256 |

#### Oracles

| Function | Test | Expected value and its source | Constant | Floor | Stated floor | Verdict |
|---|---|---|---|---|---|---|
| `PoolCapacities::from_budgets` (`pools.rs:75-102`) | `the_checked_in_budgets_give_these_pool_capacities` (`pools.rs:1411-1427`) | 55,197,040 / 10,905,188 / 922,744 / 11,156,848 / 5,704,252 bytes, 16,371 Meshes, 222 Materials, 285, 13,107 classes: hand derivation from `bench/budgets.json:11-12` and ADR 0017's shares, recomputed (Block C1) | exact | 0 (u64 divisions) | no | exact |
| `PoolCapacities::from_budgets`, `Arena::new` | `a_mesh_past_the_mesh_table_its_budget_sizes_is_refused` (`pools.rs:1432-1461`) | 10,240 Arena bytes and a 2-entry Mesh table from a 10,248-byte Budget (Block C3) | exact | 0 | no | exact; 10,248 / 5,124 = 2 sits on the division's boundary, exact in integers |
| `MeshPool::written_bytes` (`pools.rs:321-338`) | `a_mesh_loads_every_lods_meshlets_and_frees_them_with_it` (`pools.rs:1742-1750`) | 464 bytes, counted element by element by hand (Block C4) | exact | 0 | no | exact |
| `MeshPool::load_for`, index Arena | same test (`pools.rs:1767-1777`) | 24 index bytes (LOD 0's six indices), `meshlet_counts` [2, 1, 0, ...], `triangle_counts` [2, 1, 0, ...], `lod_errors_metres` [0.0, 0.5]: ADR 0020's "LOD 0's indices alone" and the test Mesh's own counts | exact | 0 (0.5 is exact in f32) | no | exact |
| `Arena::allocate`, `allocate_all` | `a_mesh_past_capacity_is_refused_and_logged_and_changes_nothing` (`pools.rs:2201-2232`) | refusal at one vertex past 4 and one index past 28, the vertex given back: `CAPACITY` = 112 bytes, offset-allocator's bins (Block C5) | exact | 0 | no | exact; the index Arena's 25 free elements sit in the bin of 24, so the refusal holds for any request of 25 or more, one lower than "one index past" says (see *Notes*) |
| `MeshPool::charge_blases` (`pools.rs:999-1005`) | `a_meshs_blas_holds_its_triangle_from_its_arena_ranges_and_is_compacted` (`pools.rs:1881-1923`) | `(2 × triangle_bytes × 0.25).round()`: the ledger's fraction applied to two built sizes from the driver's query (Block C6) | exact | 0 (a dyadic fraction of an integer under 2^53) | no | exact; the expected expression repeats the code's, so it pins the bookkeeping, not the fraction |
| `ClassTable::place` (`material.rs:173-215`) | `a_shapes_classes_double_to_the_floors_slices_and_a_full_channel_refuses` (`material.rs:1330-1381`) | slices 1, 2, 4, ..., 256, 256; placements (0,0) to (4,0); refusal once 13,107 classes are open and the shape's newest is full: ADR 0017's doubling to the floor's 256 (Block C7) | exact | 0 | no | exact; the capacity is reached exactly by the loop's condition |
| `ChannelTextures::place`, the view span | `a_large_classes_view_spans_only_the_slices_placed` (`material.rs:1422-1455`) | arrays of 1, 2, 4 slices; views of 1, 2, 1 then 2 slices; `views_changed` [true, true, false, true]: 16 MiB a 4,096² BC7 slice against `WHOLE_VIEW_ARRAY_BYTES` (Block C8) | exact | 0 | no | exact; the two-slice class is 33,554,432 bytes, exactly the constant, and is viewed whole by `<=` |
| `MaterialPool::load_texture`, the ledger | `a_texture_two_materials_share_loads_once` (`material.rs:1459-1537`) | 24 bytes for one 4×4 BC4 Texture of three one-block mips; Layers 256, 256, 0 (Block C9) | exact | 0 | no | exact |
| `MaterialPool::new`, `PoolCapacities::from_budgets` | `a_material_past_capacity_is_refused_and_logged` (`material.rs:1921-1954`) | SSBO of (4 + 285) × 80 = 23,120 bytes; the fifth Material refused (Block C9) | exact | 0 | no | exact; 4 × 19,267,584 / 19,267,584 = 4 sits on the division's boundary |
| `MaterialPool::new`, the two counts | `far_proxies_materials_and_the_others_fill_their_own_counts` (`material.rs:1959-2012`) | SSBO of (222 + 285) × 80 = 40,560 bytes; the 286th and 223rd refused and counted (Block C9) | exact | 0 | no | exact |
| `Generations::take`, `free`, `live` (`pools.rs:1218-1254`) | `a_freed_meshs_index_takes_the_next_mesh_and_refuses_the_old_handle` (`pools.rs:1619-1688`), `a_freed_materials_index_takes_the_next_material_and_refuses_the_old_handle` (`material.rs:1542-1595`) | the freed index reused at generation 1, the generation-0 Handle finding nothing (Block C10) | exact | 0 | no | exact |
| `Unused`, `free_least_recently_used` (`pools.rs:1148-1181`, `:419-442`) | `a_load_into_full_arenas_frees_the_least_recently_used_unused_mesh` (`pools.rs:1827-1874`) | the Mesh whose last user left first (Mesh 2) freed, its vertex range taken by Mesh 3, Material 102 released (Block C10) | exact | 0 | no | exact |
| `evict_for`, `evictable`, `evict_finest_mip` (`material.rs:1094-1212`) | `a_full_channel_evicts_the_lowest_priority_finest_mip_and_keeps_tails` (`material.rs:1707-1795`), `a_texture_gives_way_only_without_losing_a_mip_it_needs` (`material.rs:1802-1855`) | the victim and where it goes, hand-derived from seven classes and the ranks (Block C11) | exact | 0 | no | exact; need 2 against tail 3, and need 3 against tail 3, sit either side of `>=` |
| `resident_tail_level` (`cooked.rs:253-269`) | `texture_blob_round_trips_smallest_mip_first_and_refuses_another_version` (`cooked.rs:1257-1359`); the tail asserts at `material.rs:1724`, `:1820` | tail at mip 1 for 512² BC4, at mip 2 for 1,024² BC7, at mip 3 for 2,048² BC7 (Block C2) | exact | 0 | no | exact; the BC7 cases' tail mips are 65,536 bytes, exactly `RESIDENT_TAIL_LEVEL_BYTES`, taken by `<=` |
| `CookedMesh::from_blob`, the vertex bound (`cooked.rs:789-793`) | `mesh_blob_with_more_vertices_than_its_bytes_hold_is_refused` (`cooked.rs:1513-1530`) | refused at 2^32 − 1 vertices: meshoptimizer 1.3's 7 control bytes a 256-vertex block (`meshoptimizer-1.3-vertex-block`) (Block C12) | exact | 0 | no | exact |

The other tests in these files compare the code with itself or check that a
thing loads, refuses or logs, not a computed value against an independent
source, and get no row: the Blob round trips (Material, Mesh with its LOD
chain and thin radius, Chunk), the partial triangle and wrong count
refusals, the content hash's order, the GPU tables read back
(`pools.rs:1531-1614`), the shared-entry tests (`pools.rs:1484-1524`,
`loader.rs:383-481`), the two TLAS-after-compaction tests on eight devices,
the released slice zero, the shared Texture's slices, the 65 one-for-one
frees, a full count's own frees, the Far proxy load, the Material freed after
its Mesh, the Texture that frees a Mesh, and `logged`'s own test.

##### Block C1: the checked-in capacities

Inputs (`bench/budgets.json:11-12`): `geometry_arenas` G = 83,886,080,
`texture_pool` T = 4,294,967,296. Each Arena (`pools.rs:82-85`) is
⌊⌊G × per-mille / 1000⌋ / 4⌋ × 4:

- vertex: 83,886,080 × 658 = 55,197,040,640; / 1000 → 55,197,040; / 4 = 13,799,260 exactly → 55,197,040.
- index: × 130 = 10,905,190,400; / 1000 → 10,905,190; / 4 → 2,726,297 → 10,905,188.
- Meshlet: × 11 = 922,746,880; / 1000 → 922,746; / 4 → 230,686 → 922,744.
- Meshlet vertex: × 133 = 11,156,848,640; / 1000 → 11,156,848; / 4 = 2,789,212 → 11,156,848.
- Meshlet triangle: × 68 = 5,704,253,440; / 1000 → 5,704,253; / 4 → 1,426,063 → 5,704,252.
- Meshes (`:94`): 83,886,080 / 5,124 → 16,371 (5,124 × 16,371 = 83,885,004; × 16,372 = 83,890,128).
- Materials (`:95`): T / 19,267,584 → 222 (× 222 = 4,277,403,648; × 223 = 4,296,671,232).
- Classes (`:97-100`): T / (5 × 65,536) = T / 327,680 → 13,107 (13,107.2); `CLASSES_PER_CHANNEL` = 65,536 / 5 → 13,107 (`material.rs:19-20`); the minimum is 13,107.

Every step is a u64 or u32 integer division; no input sits on a rounding
boundary (fractions .64, .4, .88, .64, .44 before the first rounding). The
shares sum to 1,000 per mille. The Arenas in elements: 55,197,040 / 28 →
1,971,322 vertices (24 bytes unused), 922,744 / 20 → 46,137 Meshlets
(4 bytes unused; `MeshletData` is five 4-byte fields, `mesh.rs:58-64`).

##### Block C2: the Material's bytes and the Resident tail

`level_bytes` (`cooked.rs:288-299`) is ⌈max(w >> l, 1) / 4⌉ × ⌈max(h >> l, 1) / 4⌉
× block bytes (16 for BC7 and BC5, 8 for BC4). For 2,048² BC7: mip 0
512² blocks × 16 = 4,194,304; mip 1 1,048,576; mip 2 262,144; mip 3
65,536 ≤ 65,536 = `RESIDENT_TAIL_LEVEL_BYTES` (`src/stats/gpu_memory.rs:107`),
so the tail starts at mip 3 by `<=` (`cooked.rs:262`), 256 a multiple of 4
(`:265`). Tail: 65,536 + 16,384 + 4,096 + 1,024 + 256 + 64 + 16 + 16 + 16 =
87,408 (mips 9 to 11 are one block each), ADR 0017's figure; the chain is
4,194,304 + 1,048,576 + 262,144 + 87,408 = 5,592,432, and 87,408 / 5,592,432
= 1.563%, ADR 0017's "1.6%". Past the tail, a BC7 or BC5 chain is
4,194,304 + 1,048,576 + 262,144 = 5,505,024. For 2,048² BC4: mip 0
2,097,152, mip 1 524,288, mip 2 131,072, mip 3 32,768 ≤ 65,536, so past the
tail 2,752,512. Three of the first and one of the second: 16,515,072 +
2,752,512 = 19,267,584 = `TEXTURE_POOL_BYTES_PER_MATERIAL` (`pools.rs:38`).
The tests' tails: 512² BC4, mip 0 131,072 > 65,536, mip 1 32,768 → 1
(`cooked.rs:1284`); 1,024² BC7, mip 1 262,144, mip 2 65,536 → 2
(`material.rs:1724`); 2,048² BC7 → 3 (`material.rs:1820`). The two BC7
cases sit on the `<=` boundary, exact in integers.

##### Block C3: a two-Mesh Budget

G = 2 × 5,124 = 10,248. Arenas: 10,248 × 658 / 1000 → 6,743 → 6,740;
× 130 → 1,332 → 1,332; × 11 → 112 → 112; × 133 → 1,362 → 1,360; × 68 → 696
→ 696. Sum 6,740 + 1,332 + 112 + 1,360 + 696 = 10,240 (`pools.rs:1445`).
Meshes: 10,248 / 5,124 = 2 exactly; the Mesh table is 2 × 220 bytes
(`MeshData`: 4 + 6 × 8 × 4 + 2 × 12 = 220, `mesh.rs:36-49`). The third
triangle is refused by `full` (`pools.rs:490`): two indices taken of two.

##### Block C4: a two-LOD Mesh's written bytes

The test Mesh (`pools.rs:1724-1741`): 4 vertices × 28 = 112; LOD 0's 6
indices × 4 = 24 (traced); Meshlets: LOD 0 two of one triangle and 3
vertices each, LOD 1 one of one triangle and 3 vertices, so 3 × 20 = 60
bytes of `MeshletData`, 9 Meshlet vertices × 4 = 36 and 3 triangles × 4 =
12; `MeshData` 220. Sum 112 + 24 + 60 + 36 + 12 + 220 = 464, which
`written_bytes` (`:334-337`) gives as 112 + (6 + 9 + 3) × 4 + 60 + 220.

##### Block C5: one past each Arena

`CAPACITY` = 112 bytes: 4 vertices, 28 indices, 5 Meshlets, 28 Meshlet
vertices, 28 triangles. offset-allocator 0.2.0 bins a free node by
`uint_to_float_round_down` and searches from `uint_to_float_round_up` of
the request (`offset-allocator-0.2.0/src/lib.rs:207`, `:385`;
`small_float.rs`, three mantissa bits). After the first triangle: vertices
1 free (exact, under 8); indices 25 free, binned at 24 (25 = 11001b:
mantissa (25 >> 1) & 7 = 4, value (8 + 4) << 1 = 24). `mesh(2, ...)` asks 2
vertices of 1: refused. `mesh(1, [0; 26])` takes its vertex, asks 26
indices: `round_up(26)` = 26 (13 & 7 = 5, no low bit), bin 21 above the free
node's bin 20: refused, and `allocate_all` frees the vertex
(`pools.rs:1308-1312`). `mesh(1, [0, 0, 0])` asks 3 indices (bin 3): fits.
Integer throughout. A request of 25 would round up to 26 and be refused
too, though 25 elements are free.

##### Block C6: the compacted BLAS charge

Two BLASes of one triangle each (`pools.rs:1891-1895`, the first
degenerate, same size descriptor): uncompacted 2T; after `compact_blases`
both move to `compacted_blas_bytes` (`:833-834`) and the charge is
`(2T as f64 × 0.25).round()` (`:1001`). 0.25 is 2^-2, 2T is an integer far
under 2^53, so the product is exact and `round` changes nothing unless 2T
is not a multiple of 4, where it rounds half away from zero, as the
expected value's own `round()` does. Floor 0.

##### Block C7: a shape's classes

`place` (`material.rs:173-215`): wall shape × 7: class 0 of 1 slice (0,0);
class 1 of 2 (1,0), (1,1); class 2 of 4 (2,0) to (2,3); the other shape
opens class 3 of 1 (3,0); the next wall opens class 4 at twice class 2's,
8 (4,0); `slices(4)` = 8. 2,000 more walls: classes of 8 (one slice taken),
16, 32, 64, 128, 256, then 256 each: the first ten wall classes are 1, 2,
4, 8, 16, 32, 64, 128, 256, 256 (`:1369`). Opening shapes of widths 16, 20,
... until `classes.len()` = 13,107 leaves the next new shape and the full
`other` class refused (`:199-201`), a wall placed in its newest class's
free slices, and width 16, whose one class is full, refused.

##### Block C8: the view span

4,096² BC7, one mip: 1,024² blocks × 16 = 16,777,216 bytes a slice. Class
0: 1 slice, 16 MiB ≤ 32 MiB, viewed whole (1). Class 1: 2 slices,
33,554,432 = `WHOLE_VIEW_ARRAY_BYTES` (`material.rs:31`, 32 << 20), viewed
whole by `<=` (`:454`): 2. Class 2: 4 slices, 64 MiB, viewed over the
slices placed: 1, then 2 after the next placement. `views_changed`: class
0 opens (true), class 1 opens (true), class 1 slice 1 within its whole view
(false), class 2 opens (true); class 1 slice 0 released and placed again
(false); class 2 slice 1 widens (true). The ledger's `level_bytes`
(`src/stats/gpu_memory.rs:297-302`) multiplies by the array layers through
`theoretical_memory_footprint`.

##### Block C9: the ledger's Texture and SSBO bytes

4×4 BC4 with 3 mips (⌊log2 4⌋ + 1): each mip one 8-byte block; its tail is
mip 0 (8 ≤ 65,536), so one tail class of one slice: 3 × 8 = 24 bytes
(`material.rs:1522`). Its Layer: class 1 (class 0 the neutral), slice 0,
1 × 256 + 0 = 256 = `SLICES_PER_CLASS` (`:1526-1527`); the third Material
has no occlusion Texture: Layer 0. `MaterialData` is five `[_; 4]` of
4-byte words, 80 bytes (`material.rs:83-91`). Budget of 4 × 19,267,584 →
4 Materials, plus the fixed 285: 289 × 80 = 23,120 (`:1936`); checked-in:
507 × 80 = 40,560 (`:1971`).

##### Block C10: generations and least recently used

`Generations` (`pools.rs:1198-1260`): an index taken fresh is generation 0;
`free` sets it not live and moves it to 1; `take` pops it from `free`; a
generation-0 Handle fails `live`. The Mesh test reuses the index at
generation 1 (`:1673-1677`). LRU (`pools.rs:1827-1874`): `remove_user` of
Mesh 2, then Mesh 1, pushes ticks 0 and 1; `pop_least_recent_where` takes
tick 0, Mesh 2; its ranges, freed first, are the ones Mesh 3 takes (the
vertex Arena's 4 elements hold two Meshes of 2). Integer and order only.

##### Block C11: eviction order

`a_full_channel_evicts_...`: 1,024² BC7 read from mip 2 (tail level 2,
Block C2). Seven classes: neutral; three tail classes of 1, 2 and 4 slices
for six Textures; one mip-1 class (Texture 10); mip-0 classes of 1 and 2
slices (10, then 11 and 12, Texture 10's mip-1 slice released). Texture 13
to mip 0: no released slice and its newest mip-0 class full, so `has_room`
counts one new class, 8 > 7: eviction. Candidates 10 (need 5, plan 7), 11
(5, 3), 12 (4, 1); all evictable (needs ≥ tail 2). The minimum of
(`Reverse(need)`, plan, ID) is 11; its next class, mip 1, has the released
slice, so it moves to mip 1 (resident 1) and 13 takes its slice. Then 14
against 12 alone: 12's mip-1 class has no slice left and no new class fits,
so it falls back to its tail (resident 2). 15 finds no candidate:
`Refused`. `a_texture_gives_way_...`: 2,048² BC7 at mip 0 in three classes
(neutral, tail, chain), tail level 3; `evictable` with need 2: 2 ≥ 3 false
and no room for a mip-1 class, so `None`; with need 3: 3 ≥ 3 true. Integer
comparisons; the two needs sit either side of `>=`.

##### Block C12: the vertex count bound

`cooked.rs:789-793`: refused unless ⌈n / 256⌉ × 7 ≤ encoded bytes. At n =
2^32 − 1 = 4,294,967,295: ⌈n / 256⌉ = 16,777,216, × 7 = 117,440,512 bytes,
far past the test's few encoded bytes. The comment's figures: 256 × 28 / 7
= 1,024 decoded bytes per encoded byte, and (2^32 − 1) × 28 =
120,259,084,260 bytes = 112.0 GiB (/ 2^30). A valid 3-vertex stream holds
at least one block's 7 control bytes, so the round trips pass. usize
arithmetic, no overflow on 64-bit.

#### Open issues read

| Issue | What the code shows | Needs hardware |
|---|---|---|
| The texture pool's Channels are capped in Texture classes, not bytes (ADR 0017's capacity clause, broken, known) | As the issue says, from the pool's side. A Channel's capacity is `texture_classes_per_channel`, 13,107, which `from_budgets` derives from bytes on the premise of one class per 64 KiB (`pools.rs:97-100`), but `ClassTable::has_room` and `place` count classes only (`material.rs:199-201`, `:231-245`), and a class opens at 1, 2, 4, ... 256 slices of its shape whatever a slice's bytes (`:202-204`), its whole array charged at creation (`:430-448`). One 2,048² BC7 chain is 5,592,432 bytes a slice (Block C2), so its class of 256 slices is 1,431,662,592 bytes, a third of the 4 GiB texture pool Budget, and a shape's first nine classes hold 511 slices. No byte figure enters a placement, and nothing caps the `resident_tails` Budget either: no pool capacity reads it (`pools.rs:75-102`). The streaming section's row reads the same from the plan's side. Nothing filed. | No |

### the Reference scenes

Scope: read in full at the frozen commit `e5c55a8d`, test modules included: `src/scenes/world.rs` (998 lines, all new since `8bc34eb1`), `src/scenes/town.rs` (1,472 lines, 414 changed in scope) and `src/scenes/mod.rs` (1,726 lines, 647 changed in scope); 4,196 lines. `git diff 8bc34eb1..e5c55a8d -- src/scenes/` and `git log -S` were read to place each changed line and each doc comment in scope. ADRs read whole, status lines and amendments included: 0018, 0026, 0013, 0027, 0006, 0011, 0025; read for the clauses these files implement: 0002, 0003, 0014, 0017, 0019, 0022, 0023. Spec stories read: every story of the M4 spec (`spec-611.md`) and the M5 spec (`spec-804.md`), with their Implementation and Testing decisions. Opened outside these files only as far as a clause or a constant needs them: `src/assets/cooked.rs:50-80` (`CHUNK_EDGE_METRES`, `chunk_origin`, `chunk_holding`); `src/streaming/mod.rs:311-321` (`distance_metres`, the nearest point of a Chunk); `src/state.rs:55-135`, `:215-232`, `:388-420` (the spawn dispatch, the root shift, the fixed chain); `src/cook/mesh.rs:98-150`, `:422-430`, `:975-1008` (the thin tube rule and its tolerance, the QTangent normal decode, the Reference scenes' Mesh hash test); `src/cook/far_proxy.rs:814-824` (the densest Chunk's proxy hash test); `src/cook/export.rs:60-80`; `src/assets/material.rs:171-205` (`ClassTable::place`, which the fit test copies); `src/assets/loader.rs:34-46`; `src/bench/mod.rs:52-117` (warm-up frames); `tests/goldens.rs:1-20`, `:305-330` (`world`'s Golden frames). No crate source was opened. `CONTEXT.md` was read for Thin Primitive, Placement, Chunk, Moving Instance, Fast travel, Load radius and View distance. Several figures below depend on the world generator's hash; this reader recomputed `random` (`town.rs:293-304`, SplitMix64's output function folded over four parts) in a standalone script outside the worktree, and every such figure names the draws it used. `docs/audit/deviations.md` has no row on these files; `scripts/cycles-reference/DIFFERENCES.md` lists image differences of `town` and `world` that no line here decides. The streaming section's frame reader filed L1-F41 at `src/scenes/mod.rs:921-934` (`wrap_around_camera` reading the per-frame camera in the fixed schedule); it is cited, not filed again. No `cargo` command was run.

#### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0018 | "Procedural Reference scenes cook too, under Asset IDs derived from the scene name, so Goldens test the cooked path" | holds: every procedural Mesh, Material, Texture, Chunk and Far proxy ID is built from `AssetSource::Scene(self.name())`; `world`'s and `town`'s spawns load the bird and the ground by those IDs | `src/scenes/mod.rs:122-138`, `:345-369`; `src/scenes/world.rs:245`, `:265`; `src/scenes/town.rs:825` |
| 0018, checklist | "The runtime still opens no source path"; "references Assets by Asset ID, never by source path" | holds: the two glTF scenes name their source by manifest GUID (`SCI_FI_HELMET`, `NORMAL_TANGENT_TEST`) and their Meshes by glTF name under it; `source_files` exists only under the `cook` feature | `src/scenes/mod.rs:140-157`, `:707-718`, `:806-814` |
| 0018, 0011 | "The runtime loads Blobs only"; "glTF ... is the authoring source; the runtime loads Blobs" | holds on the scene side: every generator (Meshes, Materials, Textures, static content) is compiled only with the `cook` feature, and the runtime spawns load by ID through the `AssetLoader` | `src/scenes/world.rs:281-287`; `src/scenes/town.rs:916-924`; `src/scenes/mod.rs:159-161`, `:251-254`, `:299-301`, `:412-415` |
| 0018 | "A Chunk's edge is 256 m (4 by 4 cells of ADR 0014's 64 m grid; 40 by 40 Chunks cover 10 km)" | holds: `CHUNKS_PER_SIDE` 40 over `CHUNK_EDGE_METRES` 256, 10,240 m a side; `chunk_square(-20..20)` lists 1,600 Chunks | `src/scenes/world.rs:26-27`; `src/scenes/mod.rs:397`; `src/assets/cooked.rs:57-58` |
| 0018 | "A v1 Chunk is its own Asset, named from its scene and its integer coordinates" | holds | `src/scenes/mod.rs:345-350` |
| 0018 | "A Placement holds a Mesh Asset ID, an optional Material Asset ID that overrides its Mesh's Materials ..., an f64 translation ..., and f32 rotation and scale" | holds on the scene side: both generators build `CookedPlacement` with a `DVec3` translation from the World's origin, a `Quat` and a `Vec3` scale, the walls' facade Material as the override; the cut to the Chunk's origin falls in reader B | `src/scenes/world.rs:661-672`, `:695-708`; `src/scenes/town.rs:409-419` |
| 0018 | "A light holds an f64 translation, a colour, an intensity in candela (ADR 0023) and a range in metres" | holds on the scene side: `town`'s lamp lights are `CookedLight` from `LAMP_LIGHT`, 2,000,000 cd and 18 m, at f64 positions; `world` cooks no point light | `src/scenes/town.rs:241-245`, `:397`, `:421-429`; `src/scenes/world.rs:498`, `:508`, `:520` |
| 0018, 0026 | "Each Placement and light belongs to the Chunk that holds its pivot"; "a Placement belongs to the Chunk that holds its pivot" | holds: `chunk_content` keeps exactly the Placements and lights whose pivot `chunk_holding` puts in the Chunk; the generator's `reaching` over-selects by each square's reach (a lot's 6 m covers its windows' pivots, at most 5 m off on either axis; a farm tile's 64 m covers its farm, at most 48 + 5 m off) and never under-selects | `src/scenes/mod.rs:449-460`; `src/scenes/world.rs:525-536`, `:555`, `:561`, `:594`, `:631` |
| 0018 | "Plain `praxis cook` leaves out the `world` scene ... cooks only as `praxis cook world` or on a miss" | falls in reader A | |
| 0018 | "The cooker warns when an ID that something references no longer exists" | falls in reader A (M3.5's L1-C01, which reader A finds fixed but for a scene with an absent source); the scenes add references by name (facade Textures by facade name) that only the cook sees | `src/scenes/world.rs:329-333` |
| 0018 | Meshes: "Each Primitive stores a Thin Primitive's radius ... The cook refuses one unless each triangle is the side of a tube whose radius is within a quarter of the one declared, or a cap closing such a tube" | holds, and `town`'s declarations meet it: every declared radius is within a quarter of its tube's (the mast 0.03 to 0.02 m declared 0.025, ratios 1.2 and 0.8; the lamp pole 0.09 to 0.06 declared 0.075, 1.2 and 0.8; the utility pole 0.13 to 0.10 declared 0.115, 1.13 and 0.87; the rest exact); the rule itself falls in reader A | `src/scenes/town.rs:1076-1081`, `:1102-1186`; `src/cook/mesh.rs:98`, `:111-117` |
| 0018 | Meshes: "a cap's vertices keep their own normals" (amended 2026-11-14) | falls in reader A for the rule; the clause is false since the cook gives each cap vertex its ring's normal, which a test in `town.rs` pins: L1-C01 | `src/scenes/town.rs:1452-1470`; `src/cook/mesh.rs:100-102`, `:126-131` |
| 0026 | "Only a scene with a Far field cooks proxies; it names its ground's Mesh" | holds: `far_field_ground` names `world`'s `ground` and no other scene's; only `world`'s View distance exceeds its Load radius | `src/scenes/mod.rs:371-376`, `:573-580` |
| 0026 | Its Blobs: "each under an Asset ID made of the scene and the Chunk's coordinates" | holds: the proxy's Mesh and its baked Material and two Textures are named `far proxy {x} {z}` under the scene | `src/scenes/mod.rs:352-369` |
| 0026 | "It refuses a proxy of more than 5,832 triangles outside its ground: the walls and roofs of `world`'s densest Chunk, 324 buildings of 18" | holds on the scene side: walls are 4 quads (8 triangles) and a roof is a bottomless slab of 5 quads (10); the densest Chunk holds 324 lots by this reader's recomputation of every settlement's lots (`settlement_centre` from the hash, 52 × 52 lots a settlement); 324 × 18 = 5,832; the refusal falls in reader B | `src/scenes/world.rs:408-432`, `:435-446`, `:459-474`, `:583-593` |
| 0026 | "A first Primitive ... windows included, and a second Primitive of the Chunk's ground tiles" | holds on the scene side: `chunk_standing_content` hands the bake each building whole with its windows and the Chunk's 128 m ground tiles; the merge falls in reader B | `src/scenes/mod.rs:462-476`; `src/scenes/world.rs:511-521` |
| 0026 | "Past the View distance. Nothing is drawn but flat ground reaching the horizon, one Resident quad"; Settled by the draw: "One of the ground's tiles, scaled to reach 1,024 km from the world's middle and laid 0.5 m under the tiles, placed by the scene as a Chunk of one Placement that is never held, and traced by no ray" | holds: `spawn_world` loads the ground tile through `untraced_mesh` (no BLAS), scales it by 2 × 1,024,000 / 128 = 16,000 so its 64 m half-side reaches 1,024,000 m, at 0.5 m under the origin, as a `FarGround` Chunk of one Placement with `held: false` | `src/scenes/world.rs:43-51`, `:244-264`; `src/assets/loader.rs:34-36` |
| 0026 | "f32 places a corner to 62.5 mm"; "From the camera's 60 m it ends a tenth of a pixel under the horizon at 1440p" | holds: 2^19 ≤ 1,024,000 < 2^20, so an f32 spacing of 2^-4 m = 62.5 mm; atan(60 / 1,024,000) = 5.86e-5 rad against 2 tan(22.5°) / 1440 = 5.75e-4 rad a pixel, 0.102 pixel (0.1024 from the traversal's corner, 1,019,008 m from the edge) | `src/scenes/world.rs:43-46`, `:58` |
| 0026, 0013 | the View distance, "2 km at target scale (ADR 0013), never less than the Load radius" (`CONTEXT.md`); the Load radius 256 m | holds: `world` 2,000 m and 256 m; every other scene's View distance is its Load radius | `src/scenes/mod.rs:553-580` |
| 0026 | "Which are Resident ... released past the View distance plus one Chunk's edge" (M5 story 22's "at most 285") | falls in streaming; on the scene's constants the most Chunks within 2,256 m of a camera on a 64 m grid is 285, as the story says (this reader's count over 161 × 161 cameras with `distance_metres`) | `src/scenes/mod.rs:573-580`; `src/scenes/world.rs:901-905` |
| 0027 | "A Far proxy holds no Thin Primitive, so the rule ends at the Load radius" | holds on the scene side: no `world` Mesh is declared thin | `src/scenes/world.rs:389-404` |
| 0027 | "Its vertices move out from its tube's centre line ... the tube's true width over the width it draws"; Consequences: "`town`'s wire", "`town`'s six-faced post" | holds on the scene side: the wire is a 3-faced tube of 15 mm, the post a 6-faced tube of 25 mm, both declared at their radius; the post's figures check (0.75 × 0.05 / 5.753e-4 = 65 m, 0.866 × 0.05 / 5.753e-4 = 75 m, 8 × 0.75 × 0.05 / 5.753e-4 = 521 m, 8 × 0.866 × 0.05 / 5.753e-4 = 602 m); the widening falls in shaders | `src/scenes/town.rs:1102-1116`, `:1177-1186` |
| 0019 | the main pass's twins for Thin Primitives | falls in render and shaders; no clause of 0019 names a scene's declaration | |
| 0013 | "a 10 by 10 km streamed Game world, a 2 km LOD'd view distance (drawn past the Load radius as a Far proxy per Chunk, ADR 0026)" | holds: 10.24 km a side, 2 km View distance | `src/scenes/world.rs:1-2`, `:26-27`; `src/scenes/mod.rs:573-580` |
| 0013 | "M4 set the seven above `town`'s and the world scene's peaks on the M4 Pro" | falls in tooling (bench); the scenes exist at the scale the peaks were read from | |
| 0014 | "World positions are f64"; checklist "No World position reaches the GPU in f32" | holds on the scene side: settlement centres, lots, farms, cameras, flock centres, birds' offsets, Placements and lights are `DVec3`; a window's offset from its building (at most 30 m) is rotated in f32 and added in f64 | `src/scenes/world.rs:171-183`, `:191-239`, `:586-593`, `:688`; `src/scenes/town.rs:397`, `:770-806` |
| 0014 | the 128 m pivot bound (ADR 0026's Consequence exempts a Far proxy) | holds: `world`'s farthest vertex from a pivot is a tower roof's top corner, √(4.3² + 29.1² + 4.3²) = 29.7 m (9 storeys of 3.2 m, a 0.3 m slab, 4 m plus 0.3 m of overhang); ground tiles reach 64√2 = 90.5 m in both scenes; `far_coplanar`'s 125 m tiles 88.4 m; the far ground passes the bound and ADR 0026 records why | `src/scenes/world.rs:28`, `:118`, `:435-446`; `src/scenes/town.rs:178-182`; `src/scenes/mod.rs:824-831` |
| 0022 | "Everything that changes game state runs in the fixed schedule"; determinism "whatever the frame timing" | broken, known: L1-F41 (streaming section) files `wrap_around_camera`; `spin` and `sway` read the Tick count alone | `src/scenes/mod.rs:902-910`, `:919-934`, `:1328-1345` |
| 0022, checklist | "every entity whose World transform it changes, children included, carries `PreviousWorldTransform`" | holds: flock centres and birds (`town` and `world` share `spawn_flocks`), orrery bodies, crowd bones and refit figures | `src/scenes/town.rs:834-857`; `src/scenes/mod.rs:873-878`, `:1286-1301` |
| 0022, checklist | "a jump inserts `Teleported` before `reset_teleported` runs" | holds: `wrap_around_camera` inserts it on each flock centre it moves, ahead of `reset_teleported` in the chain; tested to drop the bird's previous transform on the jump; a Fast travel moves only the camera, which is not blended, and the wrap then marks each flock it moves | `src/scenes/mod.rs:929-932`, `:1624-1690`; `src/state.rs:410`, `:414`; `src/scenes/world.rs:207-210` |
| 0022 | "randomness comes from a seeded resource ...; no result depends on hash-map iteration order" | holds: no fixed-schedule system in these files draws randomness; every generator choice is `random` of where the thing stands, iterated in fixed loops; `town` sorts its lamp corners with the stable `sort_by` | `src/scenes/town.rs:301-304`, `:391`; `src/scenes/world.rs:552-564` |
| 0023 | "a `DirectionalLight` gives illuminance in lux and a `PointLight` luminous intensity in candela"; "every Reference scene's lights ... under a sun of 100,000 lx" | holds: `SUN_ILLUMINANCE` 100,000 lx for every scene's sun; `town`'s lamps 20 × 100,000 = 2,000,000 cd, the coloured lights 900,000 cd, minification's 3 × 10^7 cd | `src/scenes/mod.rs:582-610`, `:636-670`, `:950-965`; `src/scenes/town.rs:239-245` |
| 0023 | ambient in lux, background in cd/m², an EV100 per scene set by hand | holds: 10,000 lx ambient, (10,000, 20,000, 30,000) cd/m² background, EV100 log₂(100,000 / 1.2) = 16.35 for all twelve scenes; the traversal sets `world`'s EV100 itself, as `ReferenceScene::camera_at` does | `src/scenes/mod.rs:513-531`, `:612-633`; `src/scenes/world.rs:236` |
| 0006, checklist | "Components hold data only ... A method that is a pure function of the component's own data and its arguments ... is allowed"; "No GPU resources stored on ECS components" | holds: `Spin`, `WrapsAroundCamera` and `Bone` are data; `Sway::curvature` reads only `self.phase` and its argument, the amendment's kind; `MeshRef` holds a Handle | `src/scenes/mod.rs:895-917`, `:1307-1326`; `src/scenes/town.rs:846-853` |
| 0003 | "prepare writes only the slots ... whose World transform, Mesh or Material changed ..., so camera motion writes no Instance" | holds on the scene side: `wrap_around_camera` dereferences `Transform` mutably only when a flock jumps, so a flock the camera does not push past half a span is not marked changed | `src/scenes/mod.rs:926-932` |
| 0003 (refined), M4 story 2 | static Placements as Instances in a slot range their Chunk owns, no entity per Placement | holds on the scene side: no spawn in these files spawns static content; `spawn_world` and `spawn_town` spawn flocks and the far ground's Chunk only; the slot ranges fall in render | `src/scenes/world.rs:244-279`; `src/scenes/town.rs:824-828` |
| 0017 | the geometry Arenas' shares "as `helmet`'s, `town`'s and the world scene's Meshes fill them ..., the world scene's with the Far proxies of its densest far field: the 282 within the View distance and a Chunk's edge of a camera on a 64 m grid" | holds in form: the fit test walks a camera over the same 64 m grid with the same 2,256 m and sums each Arena's most; the 282 is the densest far field by bytes, which needs a run (see *Notes*) | `src/scenes/world.rs:844-997` |
| 0017, 0002 | the Mesh table, the Material table's two counts, the texture pool's bytes and classes sized from the Budgets | holds in form: the fit test asserts each count's most against `PoolCapacities::checked_in()`, Far proxies' baked Materials counted apart (ADR 0026's Consequence); ADR 0002's buffers fall in reader C | `src/scenes/world.rs:977-996` |
| 0011, 0025 | a generated library's sources; "everything in it is published under the MIT licence" | holds: `world`'s and `town`'s content is generated from code; the one adapted source, SplitMix64's output function, cites its paper and Vigna's public-domain `splitmix64.c` | `src/scenes/town.rs:289-298`; `src/scenes/world.rs:356-387` |
| checklist | "A cooked Mesh Blob is the same bytes on every machine: the two hash tests pass on each" | holds as tested: every Reference scene Mesh feeds the hash at `src/cook/mesh.rs:993-1008`; `town`'s `cylinder` and `wire` take platform f32 `cos` and `sin`, which the checklist names as a cause, and agree on the machines CI and the M4 Pro run (see *Notes*) | `src/scenes/town.rs:1318-1321`, `:1383-1384`; `src/scenes/mod.rs:1440-1442` |
| checklist | the slot order of prepare's writes; uploads through the load staging; GPU memory through the ledger; no buffer grows | not applicable: no code in these files creates, writes or uploads a GPU resource | |

#### Spec stories

| Story | Verdict | Where |
|---|---|---|
| M4 story 8, a point light in a Chunk carries an f64 translation and photometric values | holds on the scene side: `town`'s 1,024 lamp lights cook as `CookedLight` in candela at f64 positions | `src/scenes/town.rs:393-398`, `:421-429` |
| M4 story 10, ground cooked as 128 m tiles, four per Chunk | holds: both streamed scenes lay 128 m tiles whose edges fall on multiples of 128 m (`world`: 80 a side from −5,120 m; `town`: 8 a side from −512 m), so each 256 m Chunk holds four whole tiles | `src/scenes/world.rs:28-29`, `:548-551`; `src/scenes/town.rs:178-182`, `:379-381`; `src/scenes/mod.rs:1476-1490` |
| M4 story 11, a Placement belongs to the Chunk that holds its pivot | holds on the scene side (`chunk_content`); the bounds fall in reader B | `src/scenes/mod.rs:449-460` |
| M4 story 14, `town`'s ground re-tiled into 128 m tiles | holds: 8 × 8 tiles, 1,024 m, the 4 × 4 Chunks the blocks stand on, trimmed from 1.1 km as the spec's decision allowed | `src/scenes/town.rs:178-182` |
| M4 story 15, `town` cooked into the Chunks it covers and loaded through the Chunk path | holds on the scene side: `chunk_square(-2..2)` and `static_content` place every block, lamp, pole and wire pivot within ±500 m; Goldens fall in tooling | `src/scenes/mod.rs:396`, `:438`; `src/scenes/town.rs:403-431`, `:709-716` |
| M4 story 16, every Reference scene's static content loads through Chunks | holds on the scene side: `static_content` gives every scene's Placements and lights, and no spawn in these files spawns static content | `src/scenes/mod.rs:412-441`, `:1072-1084` |
| M4 story 17, Moving Instances spawned by scene systems | holds: the flocks, the orrery and the crowd | `src/scenes/town.rs:834-857`; `src/scenes/world.rs:265-277`; `src/scenes/mod.rs:871-893`, `:1248-1305` |
| M4 story 54, lights stream with their Chunks | holds on the scene side: `town`'s and the small scenes' point lights are cooked into Chunks, none spawned | `src/scenes/mod.rs:412-441`; `src/scenes/town.rs:421-429` |
| M4 story 56, a generated world of 40 × 40 Chunks cooked by `praxis cook` | holds: 1,600 Chunks, each generated alone by `static_content_reaching`; the command falls in reader A | `src/scenes/world.rs:26`, `:504-509`; `src/scenes/mod.rs:397`, `:449-460` |
| M4 story 57, settlements as dense as the Budgets allow, sparse land between | holds in form: 64 settlements of 52 × 52 lots, farms on a quarter of the open 128 m tiles, and the fit test asserts the densest view against the Budgets' capacities; whether denser would still fit is the ticket's measurement | `src/scenes/world.rs:31-41`, `:99-100`, `:857-997` |
| M4 story 58, unique Meshes and Textures over 8 GB | holds: 64 settlements × 24 facade Textures of 2048² BC7 with every mip, 4,194,304 × 4/3 bytes each, are 8 GiB (8.59 GB) before the Far proxies' atlases; whether eviction then runs is under *Notes* | `src/scenes/world.rs:84-86`, `:337-352` |
| M4 story 59, flat ground tiles until terrain | holds | `src/scenes/world.rs:389-394` |
| M4 story 60, 10,000 Moving Instances around the camera | holds in placement: 100 flocks of 100 birds, `FLOCKS_PER_SIDE² × BIRDS_PER_FLOCK` asserted equal to `DYNAMIC_TLAS_CAPACITY`, wrapped to within 160 m of the camera per axis; the wrap's determinism is L1-F41 | `src/scenes/town.rs:251-253`; `src/scenes/world.rs:77-82`, `:265-277` |
| M4 story 61, a bench path of about 60 s: into a settlement at 50 m/s, across sparse land, one Fast travel | holds: 3,600 frames at 50/60 m a frame, over settlement [4, 4] at frame 599, over open land at frame 1,799, a 4,948 m jump at frame 1,800 into settlement [1, 6], tested; its comment's start "from farmland" is L1-C14 | `src/scenes/world.rs:53-68`, `:189-211`, `:756-769` |
| M4 story 62, a full traversal along a side, the diagonal and one Fast travel | holds: 11,980 frames west to east along the south side half a Chunk in, a jump to the north-east corner, 16,942 frames down the diagonal, tested | `src/scenes/world.rs:69-75`, `:213-239`, `:771-796` |
| M4 story 63, CI cooks only what its frames load, Settled Goldens at a few frames | falls in tooling; the scene gives four Golden frames (59, 300, 599, 1,500) whose descriptions in `tests/goldens.rs` this reader checked against the path: 250.8 m and 50 m short of settlement [4, 4], 199 m inside it, and 196 m past it with settlement [4, 3]'s Chunks 444 to 1,325 m away, all in the Far field | `src/scenes/world.rs:189-211` |
| M4 story 65, the same Chunk content at the origin and the far corner, identical images | falls in the frame (State tests) | |
| M4 story 66, nothing drawn past the Load radius until M5 | superseded by ADR 0026 | |
| M5 story 16, each Chunk cooked a Far proxy | falls in reader B; the scene supplies `chunk_standing_content` and the ground's Mesh | `src/scenes/mod.rs:371-376`, `:462-476` |
| M5 story 21, flat ground drawn to the horizon | holds on the scene side: the far ground's Chunk; the draw falls in render | `src/scenes/world.rs:244-264` |
| M5 story 22, at most 285 Far proxies Resident | falls in streaming; 285 checks on the scene's constants (see the clause row) | `src/scenes/mod.rs:573-580` |
| M5 story 26, the fit test extended to the densest 2 km view's Far proxies | holds: Far proxies within 2,256 m join the static TLAS, Instance, Arena, Mesh table, baked Material and texture pool counts | `src/scenes/world.rs:844-997` |
| M5 story 30, a Fast travel's hold waits for the Load radius alone | falls in streaming; the scene marks the jump frame `teleported` | `src/scenes/world.rs:207-210`, `:234-235` |
| M5 story 32, the traversal reports Late Far proxies | falls in tooling; the scene gives the path | `src/scenes/world.rs:213-239` |
| M5 story 33, a Primitive declared thin with a radius in a generator or by glTF `extras` | holds for the generator (`Geometry::thin`); `extras` falls in reader A | `src/scenes/mod.rs:1406-1418`; `src/scenes/town.rs:976-978` |
| M5 story 34, the cooker refuses a Thin Primitive without radial normals | falls in reader A; `town.rs`'s test drives the refusal at its seam (a cuboid bar, a bar merged with a post, radius 0 and NaN) | `src/scenes/town.rs:1416-1433` |
| M5 story 38, `town`'s top rails made tubes, tubes of few faces settled | holds: the top rail is a 6-faced tube of the posts' 25 mm radius resting on the 1 m posts (centre 1.025 m), its own Mesh, declared thin | `src/scenes/town.rs:561-566`, `:1117-1135` |
| M5 story 39, `town`'s pigeons scaled to a pixel and dropped at zero coverage | falls in render and shaders; the bird is an ordinary Mesh of two cuboids | `src/scenes/town.rs:1243-1249` |
| M5 stories 1 to 15, 17 to 20, 23 to 25, 27 to 29, 31, 35 to 37, 40 to 42 | fall in cook, render, shaders, streaming or tooling; nothing in these files implements them | |
| M4 stories 1 to 7, 9, 12, 13, 18 to 53, 55, 64, 67 to 71 | fall in cook, assets, streaming, render or tooling | |

#### Registry keys

Two keys are cited in these files, both on lines unchanged since `8bc34eb1`; both match their quotes. No key is cited in `world.rs` or `town.rs`.

| File | Keys | Verdict |
|---|---|---|
| `src/scenes/mod.rs:637` | `bevy-lux-constants` | matches: the quote gives `pub const DIRECT_SUNLIGHT: f32 = 100_000.;`, and `SUN_ILLUMINANCE = 100_000.0` lx at `:639` |
| `src/scenes/mod.rs:755` | `hoffman-2013-f0` | matches: the quote gives gold's linear F0 as `1.00,0.71,0.29`, and `SWEEP_BASE_COLOR = [1.0, 0.71, 0.29, 1.0]` at `:758` |
| `src/scenes/town.rs:289-292` (uncited) | none: SplitMix64's output function, cited to Steele, Lea and Flood (OOPSLA 2014) and Vigna's `splitmix64.c` in prose | no key; the lines are unchanged in scope (see *Notes*) |

#### Oracles

| Function | Test | Expected value and its source | Constant | Floor | Stated floor | Verdict |
|---|---|---|---|---|---|---|
| `camera_at` (`src/scenes/world.rs:191-211`) | `the_bench_path_flies_into_a_settlement_over_farmland_then_fast_travels_into_another` through `jumps` (`:728-744`, `:756-769`) | each non-jump frame moves 50/60 m, M4 story 61's 50 m/s at 60 Hz; one jump, past 1,024 m (twice the 512 m hold radius); recomputed: the jump is 4,948.25 m | `1e-6` m on the step (`:738`) | 1.0e-12 m (Block D1) | no | loose, by about 10^6 |
| `traversal_camera_at` (`src/scenes/world.rs:214-239`) | `the_traversal_flies_a_side_then_the_diagonal_through_one_fast_travel` (`:771-796`) | 50/60 m a frame; the side leg's z unchanged; the diagonal's \|x\| equal to its \|z\|; both legs 9,900 to 10,240 m long; every frame within 5,120 m; recomputed: 9,982.5 m and 9,983.0 m | `1e-6` m on the step and on \|Δx ± Δz\|, `1e-9` m on the side's z (`:779`, `:784`) | 4e-12 m on the step; 0 on the diagonal's Δx + Δz and on the side's z (Block D2) | no | loose on the step, by about 2.5 × 10^5; the diagonal and the side's z exact, under tolerances of 1e-6 m and 1e-9 m |
| `wrap_around_camera` (`src/scenes/mod.rs:921-934`) | `a_wrapping_entity_jumps_whole_spans_to_within_half_a_span_of_the_camera` (`:1624-1690`) | hand derivation: round((340 − 100) / 500, −249 / 500) = (0, −0), no jump; round(260 / 500, −1,240 / 500) = (1, −2), a jump of (500, 0, −1,000) to (600, 40, −1,000), the bird to (610, 40, −1,000), its previous transform dropped; round(−0.48, −0.48) = (−0, −0), no jump | exact (`assert_eq!`) | 0 (Block D3) | no | exact |
| the cook's thin tube rule, through `cylinder`, `wire` and `cuboid` (`src/scenes/town.rs:1314-1406`; rule `src/cook/mesh.rs:103-150`) | `the_cook_takes_a_thin_primitive_only_as_a_tube_near_its_declared_radius` (`:1416-1433`) | ADR 0018: a tube's radius "within a quarter of the one declared": accepted at \|R − r\| / r of 0.1667 (0.05 at 0.06) and 0.200 (the tapered pole at 0.075), refused at 0.2857 (0.05 at 0.07), 0.672 and 9.04, and for a cuboid, a mixed Primitive, 0 and NaN | exact (booleans) against `THIN_RADIUS_TOLERANCE` 0.25 | 0 (Block D4) | no | exact |
| the cook's ring normals for a thin Primitive's caps (`src/cook/mesh.rs:100-131`), through `cylinder` | `a_thin_primitives_cap_vertices_cook_with_their_rings_normals` (`:1452-1470`) | hand derivation: every cooked vertex less its normal times the declared radius lies on the axis (post) or within the taper of it (tapered pole: 0.0150012 m at its base, 0.0149988 m at its top) | `0.01 × radius_metres` past the taper: 2.5e-4 m (post), 7.5e-4 m (pole) (`:1464`) | 5.3e-6 m (post), 1.6e-5 m (pole), worst case of the Snorm16 QTangent's normal (Block D5) | no | loose, by about 47 on the worst-case floor |

Set aside as behaviour tests, not oracles: `the_densest_view_fits_the_capacities_the_budgets_give` (counts against capacities, `src/scenes/world.rs:857-997`), `a_thin_primitive_keeps_every_triangle_at_each_lod_of_a_mesh_that_simplifies` (equal index counts, `src/scenes/town.rs:1435-1450`) and `a_crowd_figure_bends_within_its_bounds` (containment). The crowd's two proxy oracles changed in scope only in how the test builds a `MeshHandle` (`src/scenes/mod.rs:1553-1563`); M3.5's rows stand for them.

##### Block D1: the bench path's step

- `:204`: z = fl(start_z − fl(k · f)), k = fl(50 / 60) = 0.83333333333333337, |k − 5/6| = 3.7e-17. Before the jump |z| ≤ 1,438.41 m and |k f| ≤ 1,500 m, so each rounding is at most half an ulp of 2^-42 m: 1.14e-13 m. After the jump |z| ≤ 3,201.82 m: 2.27e-13 m.
- `:194`: start_z = fl(fl(centre.z + 377) + 300): a constant of the run, common to both frames, so it cancels in the step.
- `:733`: `distance` of two positions whose x and y are equal bit for bit, so it is |Δz| exactly (√ of an exact square of a difference that Sterbenz's lemma makes exact).
- Step error: two roundings of k f and two of the subtraction, 4 × 2.27e-13 = 9.1e-13 m, plus k's 3.7e-17: under 1.0e-12 m. The test compares with fl(50.0 / 60.0) = k, so k's own error does not enter.

##### Block D2: the traversal's step, side and diagonal

- `:230`: heading = (end − start).normalize(), glam's `self * (1 / length)`. Side leg: (9,984, 0, 0) × fl(1 / 9,984): x within 1 ulp of 1, z = 0 × r = 0 exactly. Diagonal: (−9,984, 0, 9,984) × fl(1 / 14,119.4): x and z the same magnitude a, bit for bit, opposite signs, a within 2u of 1/√2.
- `:231-232`: position = start + heading · fl(k f) + 60 Y. |k f| ≤ 14,118 m, so the product's rounding is at most 9.1e-13 m a coordinate and the sum's (|coordinate| ≤ 4,992 m) 4.5e-13 m.
- Step: per coordinate two frames × (9.1e-13 + 4.5e-13) = 2.7e-12 m, as a norm over two coordinates under 4e-12 m; |heading| − 1 adds 2.8e-16 m over a step's 0.833 m.
- Side's z: start.z + 0 + 0 = corner, exact: floor 0, against the test's 1e-9 m.
- Diagonal: start = (c, 60, −c) and heading = (−a, 0, a) with one magnitude a, so x = fl(c − fl(a D)) and z = fl(−c + fl(a D)) = −x exactly, D = fl(k f), since rounding to nearest is symmetric in sign. x + z is 0 at every frame and Δx + Δz is 0: floor 0, against the test's 1e-6 m.
- The leg lengths, 11,979 × k = 9,982.5 m and 16,941 × k = 14,117.5 m along the diagonal (9,982.5 m on each axis), sit 82 m and 257 m inside the range's ends.

##### Block D3: the wrap

- `:927`: spans = ((camera − translation) / 500).round(). The quotients 0.48, −0.498, 0.52, −2.48 and −0.48 lie at least 0.002 from a half, past any f64 rounding of an integer-valued difference divided by 500 (relative 2^-53).
- `:928-929`: jump = (spans.x, 0, spans.z) × 500, and `jump != DVec3::ZERO`. The first and third calls give a z span of −0.0; IEEE equality has −0.0 == 0.0, so no jump, as the test expects: the one boundary in the chain, and it falls the right way.
- `:930`: 100 + 500, 0 − 1,000 and 600 + 10 are integers, exact in f64.

##### Block D4: the tube rule's boundary cases

- `src/cook/mesh.rs:111-117`: on a ring edge of a straight tube of radius R with radial vertex normals, centres_apart = (R − r)(n₁ − n₂), so the test is |R − r| ≤ 0.25 r. On a tapered tube the normal is c·radial + s·Y with c = h / √(h² + (b − t)²): the test becomes |R − r c| ≤ 0.25 r c; c = 0.9999834 for (0.09, 0.06, 5.2), so 0.0150012 / 0.0749988 = 0.20002 at the base and 0.19998 at the top.
- Margins to 0.25: 0.083, 0.050, 0.036, 0.42 and 8.8, each over 10^4 times the f32 chain's relative error (a few 2^-24).
- The cuboid has per-face vertices whose normals are equal along every edge, so no edge is on a ring and every triangle is refused; radius 0 and NaN fail the `ensure!` at `:104-107`.

##### Block D5: the cap vertices' centre line

- Expected: side and cap vertices share positions bit for bit (`around(side)` in both loops, `src/scenes/town.rs:1318-1321`, `:1350-1353`), so a cap vertex takes its ring's normal by position. The post's normal is radial, so position − normal × 0.025 is on the axis. The pole's base vertex is at 0.09 m with c = 0.9999834, so 0.09 − 0.075 c = 0.0150012 m off the axis, against a bound of 0.015 + 0.00075 m; its top, 0.0149988 m.
- Floor: the QTangent stores each quaternion component as Snorm16, a step of 1/32,767, rounded by at most 1.53e-5; `qtangent_normal` (`src/cook/mesh.rs:422-430`) is quadratic in the components, each partial derivative at most 2 in magnitude, so a component of the normal moves by at most 4 × 2 × 1.53e-5 = 1.2e-4 and the normal by at most 2.1e-4; times the radius, 5.3e-6 m (post) and 1.6e-5 m (pole). f32 positions add under 1e-8 m.
- Margin: 2.5e-4 m over 5.3e-6 m and 7.5e-4 − 1.2e-6 m over 1.6e-5 m, both about 47.

### Findings

| id | claim | severity | where | evidence | expected | verifier | placement |
|---|---|---|---|---|---|---|---|
| L1-C01 | ADR 0018 says a Thin Primitive's cap vertices "keep their own normals", but since 2026-11-16 the cook gives each cap vertex its ring's normal, so that the vertex stage widens a cap with its tube's side; the commit that made the change amended ADR 0027 and not ADR 0018. | `drift`; not load-bearing: `CookedPrimitive`'s doc comment and the function's doc state the cap rule; a reader of ADR 0018 alone is misled about what a thin Primitive's normals hold; filed by readers A and D | `docs/adr/0018-cooked-assets-are-named-by-stable-ids-and-stored-by-content-hash.md:24` | block below | ADR 0018's Meshes bullet says a cap's vertices take their ring's normal, as `src/assets/cooked.rs:513` does | `confirmed`: ADR 0018 says "a cap's vertices keep their own normals" (`docs/adr/0018-...md:24`), while `thin_primitive` sets `*normal = *ring_normal` for every non-ring vertex (`src/cook/mesh.rs:151-158`, doc "each cap vertex's normal its ring's", `:100-101`) and `CookedPrimitive`'s doc says "a cap's normal is its ring's" (`src/assets/cooked.rs:513`); `drift`. | fix #1034 |
| L1-C02 | The SciFiHelmet Mesh Blob ceiling stays at 15.29 bytes per triangle though its comment says "A lower figure re-records it" and the `meshopt` 0.7 bump measured the Blob at 353,120 bytes, 15.12 bytes per triangle, which ADR 0018 records; the ceiling admits 4,023 bytes (1.14%) of growth. | `drift`; not load-bearing: the ceiling still catches a regression past 1.14%; a reader of the comment is told the ceiling is the measurement | `tests/cook.rs:804` | block below | `HELMET_MESH_BYTES_PER_TRIANGLE` re-recorded at 15.12 (353,120 / 23,358 = 15.1177, rounded up), or the comment's rule dropped | `confirmed`: `HELMET_MESH_BYTES_PER_TRIANGLE: f64 = 15.29` under "the ceiling the LOD chain cook's measurement sets. A lower figure re-records it" (`tests/cook.rs:801-804`), while ADR 0018 records "with meshoptimizer 1.3's vertex codec 1 it is 353,120 bytes" (`docs/adr/0018-...md:24`): 353,120 / 23,358 = 15.118, so the ceiling sits 1.14% over the measurement it claims to be; `drift`. | fix #1034 |
| L1-C03 | `cook`'s doc comment says plain `praxis cook` cooks every Reference scene's procedural Assets, but since `world` was generated (`2dbd1ec1`) plain `praxis cook` passes every scene but `world`, as ADR 0018's status line says. | `drift`; not load-bearing: the binary and ADR 0018 say it; a reader of the function's doc is misled about `world` | `src/cook/mod.rs:585` | block below | the doc names the exception: every Reference scene's but `world`'s, which `praxis cook world` cooks | `confirmed`: `cook`'s doc says "`praxis cook` cooks every Reference scene's" (`src/cook/mod.rs:584-586`), while the binary's plain `cook` filters `scene != ReferenceScene::World` (`src/bin/praxis.rs:494-496`), as ADR 0018's status line says; `drift`. The tooling section's note on the same line is not a Finding, so nothing is killed for it. | fix #1034 |
| L1-C04 | `export-scene` skips a Renderable without a Material Handle whenever a Primitive of its Mesh names no Material, where prepare skips it only when such a Primitive has triangles, so an export can leave out a Renderable praxis draws, against the module's "a Reference scene as prepare reads it" and the line's own comment "As prepare". | `correctness`; one condition of one skip; no Mesh `export-scene` writes at this commit has an empty Primitive that names no Material, so no export differs today | `src/cook/export.rs:195` | block below | the export skips such a Renderable only when a Primitive with triangles names no Material, as `src/assets/pools.rs:511-513` decides for prepare | `confirmed`: The export skips on `exported.meshes[&mesh].materials.contains(&None)` under the comment "As prepare skips one whose Primitives all have triangles" (`src/cook/export.rs:194-195`), while the Mesh pool sets `needs_material_handle` only where `material.is_none() && primitive.index_count > 0` (`src/assets/pools.rs:511-513`, changed after `8bc34eb1`), so the two disagree on a Mesh with an empty Primitive naming no Material; no exported Mesh has one today. | fog (#17): revive when content has a Primitive on glTF's default Material, in the line on glTF's default Material |
| L1-C05 | A dev launch that keeps a Chunk, because one of its Placements names a Mesh the launch did not read, returns before cooking any Mesh, Material or Texture the Chunk names, and the IO pool then reads those Blobs from the Cooked cache unchecked, so a stale one loads without a cook or a warning. | `correctness`; one clause of ADR 0018 broken in part (the dev launch's "with the Assets it names") and M4 story 64 with it; reachable only while one of a scene's sources is absent: `helmet` with SciFiHelmet unfetched, whose Chunk names 10 procedural sweep Materials (`src/scenes/mod.rs:726-744`); no visible defect unless such an Asset changed since its Blob was cooked | `src/cook/launch.rs:175` | block below | before a kept Chunk's Blob is read, the launch cooks each Asset the Chunk names that it did read, as `praxis cook <scene>` does (`src/cook/mod.rs:645-660`), and keeps only the Chunk's own Blob | `confirmed`: A kept Chunk returns at `if generated.kept { self.sources.kept_chunks.push(id); return Ok(()); }` (`src/cook/launch.rs:175-178`) before `cook_after_named` cooks what it names (`:179-182`), and streaming cooks only the Chunk's own ID (`cook_chunk(id)`, `src/streaming/io.rs:1524-1525`) before the IO pool reads the named Blobs from the cache, against ADR 0018's "each Asset as its load finds the Blob missing or stale, with the Assets it names" and M4 story 64; reachable only with a scene source absent. | fix #1032 |
| L1-C06 | ADR 0011's sentence that a development launch's cooker "cooks what changed before the runtime loads Blobs" still describes a cook ahead of the loads, while a launch cooks each Asset only as its own load reaches it, and never one it does not load, as ADR 0018's amendment of 2026-11-08 and the launch's code say. | `drift`; not load-bearing: ADR 0018's status line states the current behaviour; a reader of ADR 0011 alone expects a launch to cook every changed Asset up front | `docs/adr/0011-gltf-authoring-cooked-runtime-formats.md:3` | block below | ADR 0011's sentence says the launch cooks each Blob it loads, as it loads it, missing or stale, dated with the change that made it so, or points at ADR 0018's amendment | `killed`: ADR 0011's clause is that "a development launch reaches glTF only through the cooker, which cooks what changed before the runtime loads Blobs" (`docs/adr/0011-...md:3`), and that holds of every Blob a launch loads: `src/cook/launch.rs:1-2` cooks "each Asset it loads whose Blob is missing or stale, as it loads it", before the load; the reader concedes the per-Blob reading holds, ADR 0018's amendment states the detail, and no sentence of ADR 0011 is false. | wontfix: killed by the verifier, its reason under Killed |
| L1-C07 | A Texture whose Channel has no room for its Resident tail is neither refused nor counted: `load_texture` returns Layer 0 and the Material loads sampling the neutral default, the refusal logged by `ChannelTextures::place` but missing from Frame stats, whose pool refusals are `meshes_refused` and `materials_refused` alone. | `correctness`; M4 spec story 47 broken for one capacity, a Channel's Texture classes (13,107 each); no Reference scene shown to reach it | `src/assets/material.rs:985` | block below | M4 spec, story 47: "a request past any capacity refused, logged and counted in Frame stats, so that an overflow is visible, never silent"; `CLAUDE.md` checklist: "a request past a buffer's capacity is refused and logged" | `confirmed`: `load_texture` answers a failed `push` with `return Ok(Self::DEFAULT_LAYER)` (`src/assets/material.rs:984-989`), so the Material loads sampling the neutral default instead of being refused, logged only by `ChannelTextures::place` (`:419-425`) and counted nowhere (`refused_as_full`, `:679-689`, counts the Material table alone), against M4 story 47 and ADR 0017's "a load that does not fit is refused and logged"; the last audit confirmed the same shape for Materials (its `L1-C04`, a default Handle in place of a refusal). Not a duplicate of `L1-F02`: a refused move is refused, this load is not; the count half shares one fix with it. | fix #1028 |
| L1-C08 | A mip move or an eviction can open a Texture class whose mip 0 is not whole blocks, which wgpu refuses with a validation error that its default handler turns into a panic: `stream_mips` and `evict_finest_mip` take the class shape `mips_from(level)` for any level the plan asks, while only the Resident tail's first mip is checked for whole blocks, so a Texture whose sides are multiples of 4 but not of 4 × 2^level, such as 2,052², tailed at mip 3, panics on its first move to mip 1 or 2. | `correctness`; a panic on the first such move; every Texture the cooker accepts whose side at a mip between 1 and its tail's first is not a multiple of 4 (2,052², 1,028²); no Reference scene shown to hold one | `src/assets/material.rs:1047` | block below | ADR 0017: "A Texture whose resident mips change moves to the class of its new shape by copy"; ADR 0018: "The cooker refuses a Texture whose size is not a multiple of 4 or exceeds 8192, rather than resizing it", so every such size is one the runtime must stream; the code's own rule at `cooked.rs:249-252`: a mip "not whole blocks, which a Texture of its own cannot start with" | `confirmed`: `stream_mips` takes `loaded.shape.mips_from(first_level)` (`src/assets/material.rs:1047`), and `mips_from` is `width >> level` with no block check (`:118-125`), while only the tail's mip is checked for whole blocks (`src/assets/cooked.rs:264-267`) and wgpu-core refuses a compressed texture "NotMultipleOfBlockWidth" (`vendor/wgpu-core/src/device/resource.rs:1438-1449`) with no error handler installed in `src/`; recomputed, a 2,052² BC7 Texture tails at mip 3 (64² blocks, 65,536 bytes), and its mip 1 class would be 1,026 wide. The reader accepts any multiple of 4 (`cooked.rs:447-448`). | fix #1025 |
| L1-C09 | ADR 0017 and ADR 0018 say a Texture's Resident tail is its mips of 64 KiB and smaller, but `resident_tail_level` makes the whole chain the tail when the first such mip is not whole blocks, so such a Texture never streams and keeps its mips past 64 KiB Resident for its life: a 1,000² BC7 Texture holds 1,335,360 bytes as its tail where the ADRs' tail is 85,360. | `drift`; load-bearing for a reader sizing Resident bytes or predicting which Textures stream; every Texture whose side at its first mip of 64 KiB or less is not a multiple of 4; no Reference scene shown to hold one | `src/assets/cooked.rs:264` | block below | ADR 0017 and ADR 0018 state the exception (a Texture whose tail mip is not whole blocks keeps its whole chain), or the cooker refuses such a size | `confirmed`: `resident_tail_level` filters the first mip of 64 KiB or less on whole blocks and falls back with `.unwrap_or(0)` (`src/assets/cooked.rs:260-268`), so recomputed, a 1,000² BC7 Texture's first such mip is mip 2 (250 texels, 63² blocks, 63,504 bytes), not whole blocks, and its tail is the whole chain, while ADR 0017 says "Each texture's Resident tail (mips of 64 KiB and smaller)" (`docs/adr/0017-...md:16`); `drift`. | fix #1025 |
| L1-C10 | `CookedPrimitive::texel_density` cites Unity's UV distribution metric as what it inverts, but Unity's metric is a per-Mesh average of triangle area over UV area, and the density is taken per Primitive as the square root of summed UV area over summed mesh-space area; ADR 0018 records the difference and `deviations.md` has no row for it. | `drift`; not load-bearing: mip priority reads the density the cooker writes, whatever the citation; one registry citation | `src/assets/cooked.rs:507` | block below | a `deviations.md` row at `src/assets/cooked.rs:CookedPrimitive::texel_density` (and the cook's `texel_density`), source `unity-6000-uv-distribution-metric`, giving the per-Primitive ratio of summed areas and why; `deviations.md`: "A Deviation lives here and nowhere else" | `confirmed`: `texel_density` cites `[ref:unity-6000-uv-distribution-metric]` as what it inverts (`src/assets/cooked.rs:505-508`), whose quote is a per-Mesh "Average of triangle area / uv area", while the density is per Primitive, a ratio of summed areas, as ADR 0018 says ("taken per Primitive, as a ratio of summed areas and as a length"), and `docs/audit/deviations.md`, where "A Deviation lives here and nowhere else" (`:10`), has no row for the key; `drift`. | wontfix: a Deviation the owner accepted, its row in `docs/audit/deviations.md` |
| L1-C11 | The error `ChannelTextures::place` logs for every refused placement says "the Texture keeps its Resident tail, or Layer 0 without one", but a refused mip move in `stream_mips` leaves the Texture at its earlier chain, its finer mips kept, not at its tail. | `drift`; not load-bearing: one log line misleads a reader of a refused move's log; it also shows the refusal the M5.5 report's L1-F02 calls unlogged is logged by the pool | `src/assets/material.rs:423` | block below | the line names what the caller keeps, or `stream_mips` logs its own refusal | `confirmed`: `ChannelTextures::place` logs "the Texture keeps its Resident tail, or Layer 0 without one" (`src/assets/material.rs:419-425`) for every refusal, while `stream_mips` returns `StreamedMips::Refused` on it (`:1061-1062`) before the placement changes, so a refused move keeps its earlier chain; `drift`. It also settles the cross-section point: `L1-F02`'s refusal is logged. | fix #1028 |
| L1-C12 | `ReferenceScene::World`'s doc says the world is "seen within its 256 m Load radius", while since the far field landed every Chunk out to the 2 km View distance draws as its Far proxy and flat ground reaches the horizon. | `drift`; not load-bearing: `view_distance_metres` and ADR 0026 state the true reach; misleads a reader of the scene registry about how far `world` draws, by a factor of 7.8 | `src/scenes/mod.rs:80` | block below | the doc says the world is seen out to its 2 km View distance, its Chunks past the 256 m Load radius as Far proxies, with flat ground beyond | `confirmed`: `ReferenceScene::World`'s doc says "seen within its 256 m Load radius" (`src/scenes/mod.rs:78-80`), while `view_distance_metres` returns `2000.0` for it (`:573-578`) and ADR 0026 draws "Every Chunk outside the Load radius and within the View distance" as its Far proxy; `drift`. | fix #1034 |
| L1-C13 | `chunk_coordinates`'s doc calls its list "the Chunks its spawn loads", while since the Chunk streamer landed the State spawns a scene with no Chunk loaded and streams those within the Load radius; the list is the streamer's index of the Chunks that exist. | `drift`; not load-bearing: no caller depends on the sentence; misleads a reader into taking `world`'s spawn to load all 1,600 Chunks, which only `Simulation::new` (cook tests, and `export` of a scene without a Far field) does | `src/scenes/mod.rs:379` | block below | "the Chunks that exist, which the Chunk streamer streams within the Load radius" | `confirmed`: `chunk_coordinates`' doc says "the Chunks its spawn loads" (`src/scenes/mod.rs:378-379`), while the State's spawn ends "and no Chunk loaded" (`src/state.rs:79-82`) and only `Simulation::new` loads every Chunk, called from the cook's code (`src/cook/chunk.rs:185`, `:377`); `drift`. | fix #1034 |
| L1-C14 | The bench path's doc says the camera starts "from farmland `APPROACH_METRES` south of its settlement's edge", but its frame 0 lies 39.6 m inside settlement [4, 5]'s square, over that settlement's northern row of blocks, until frame 47: the open land between settlements [4, 4] and [4, 5] is 260.4 m deep, under `APPROACH_METRES`' 300 m. | `drift`; not load-bearing: the bench's 248 warm-up frames and the first Golden (frame 59) both fall past frame 47; frames 0 to 47 of 3,600 | `src/scenes/world.rs:56` | block below | the doc says the path starts 300 m short of the settlement, wherever that falls, or `APPROACH_METRES` is no deeper than the open land in front of `CAMERA_SETTLEMENT` (at most 260 m), with a test that frame 0 is over no settlement | `confirmed`: the verifier left frame 0's position to a run, and the session ran it at the frozen commit: a temporary assertion over every settlement's square in `the_bench_path_flies_into_a_settlement_over_farmland_then_fast_travels_into_another` (`src/scenes/world.rs:764`) failed with "frame 0 inside settlement [4, 5] by 39.6001867736386 m"; `APPROACH_METRES` is 300 (`:59`) while `SETTLEMENT_STRAY_METRES` keeps only "64 m of open land between neighbouring settlements" (`:39-41`). | fix #1034 |
| L1-C15 | `TownMesh::ALL` and `BoxMaterial::ALL` say their order "indexes its Handle when loaded", but nothing has indexed a Handle by them since `town` stopped spawning its static content: each array is read only by its own compile-time assertion. | `health`; two arrays and two `const` assertion blocks, 51 lines, 0 call sites; their doc comments describe a load that no longer exists | `src/scenes/town.rs:57-59` | block below | both arrays and their assertions deleted, or a caller that needs them | `confirmed`: `TownMesh::ALL` and `BoxMaterial::ALL` say their order "indexes its Handle when loaded" (`src/scenes/town.rs:57-59`, `:134-136`), and `grep -rn "TownMesh::ALL\|BoxMaterial::ALL" src` finds only their own assertion lines (`:113-114`, `:164-165`); `health`. | fix #1034 |

#### L1-C01

Code, `src/cook/mesh.rs:100-102` and `:151-158`:

```rust
/// A Primitive declared thin, each cap vertex's normal its ring's, so a cap
/// widens with its tube's side; or a refusal, where a triangle is neither the
/// side of a tube of `radius_metres` nor a cap that closes one.
[...]
let mut tube = geometry.clone();
for (vertex, normal) in (0..).zip(&mut tube.normals) {
    if !ring_vertices.contains(&vertex)
        && let Some(ring_normal) = ring_normals.get(&position_bits(vertex))
    {
        *normal = *ring_normal;
    }
}
Ok(tube)
```

`lod_0` cooks the returned geometry (`:190-196`), so a cap vertex's
QTangent carries its ring's normal. `src/assets/cooked.rs:511-514`:
"a vertex's position less its normal times this is on its centre line,
or off it by what its ring's radius differs by on a tapered tube; a cap's
normal is its ring's." ADR 0018, Consequences, Meshes bullet: "The cook
refuses one unless each triangle is the side of a tube [...] or a cap
closing such a tube, so a side vertex's position less its normal times
the radius lies within a quarter of the radius of the centre line and no
vertex stores it; a cap's vertices keep their own normals (amended
2026-11-14 [...])." The rule changed in commit `c47a441d` (2026-11-16),
whose message records it ("Caps: The cook gives each cap vertex its
ring's normal [...] A cap now shades with its ring's normals") and whose
file list amends ADR 0027 alone; `git log 8bc34eb1..e5c55a8d --
docs/adr/0018*` shows no later change to the clause. `MESH_FORMAT_VERSION`
went from 6 to 7 for it. The clause's first half, the refusal, holds; its
last clause is now false.

Reader D filed the same sentence from the scenes' side, and the two are
one Finding here. The test `a_thin_primitives_cap_vertices_cook_with_their_rings_normals`
(`src/scenes/town.rs:1452-1470`) asserts that every cooked vertex, the
caps' included, has a normal that puts its centre on the tube's axis
(`from_axis_metres < taper_metres + 0.01 * radius_metres`), which a cap
vertex's own ±Y normal would not: a cap vertex at radius 0.025 m with
normal +Y gives a centre 0.025 m off the axis, a hundred times the test's
2.5e-4 m. `git log -S` puts both the doc comment and the test in the commit
that made `town`'s rails, aerial bars and pole arms Thin Primitives
(2026-11-16), after the amendment the ADR names. The `CLAUDE.md` checklist:
"each clause the change makes false (a doc comment, [...] a Consequence
[...]) is rewritten or dated in the same PR".

#### L1-C02

Code, `tests/cook.rs:801-805` and `:852-856`:

```rust
/// SciFiHelmet's cooked Mesh Blob bytes per triangle of LOD 0, Meshlets and
/// its LOD chain included, at most: the ceiling the LOD chain cook's
/// measurement sets. A lower figure re-records it.
const HELMET_MESH_BYTES_PER_TRIANGLE: f64 = 15.29;
const HELMET_TRIANGLES: usize = 23_358;
[...]
let bytes_per_triangle = cache.read(helmet).unwrap().len() as f64 / HELMET_TRIANGLES as f64;
assert!(bytes_per_triangle <= HELMET_MESH_BYTES_PER_TRIANGLE, [...]);
```

ADR 0018, Meshes bullet: "SciFiHelmet's Blob, of six LODs, was 356,874
bytes [...]; with meshoptimizer 1.3's vertex codec 1 it is 353,120
bytes, against 356,958 with codec 0 before it". 356,958 / 23,358 =
15.2821, which the ceiling of 15.29 was set over (commit `8ac44220` raised
it from 15.28 for a Blob of 356,958 bytes); 353,120 / 23,358 = 15.1177.
The frozen commit `e5c55a8d` measured the lower figure and amended the
ADR with it, and did not touch `tests/cook.rs`. The ceiling now passes
any Blob up to 15.29 × 23,358 = 357,143 bytes, 4,023 bytes (1.14%) over
the measured one, a margin 19 times the 0.06% M3.5's ceiling kept over
its measurement (13.83 against 13.8218). The test is skipped where
SciFiHelmet is not fetched, as on CI, so only the M4 Pro runs it.

Measured by the session, not the reader: with SciFiHelmet fetched into the
frozen worktree and a temporary print beside the assertion (reverted after
the run), `cargo test --test cook every_reference_scene_mesh_cooks_and_reads_back -- --nocapture`
at `e5c55a8d` on the Apple M4 Pro printed "353120 bytes, 15.117733 bytes
per triangle", the figure ADR 0018 records, and passed against the ceiling
of 15.29.

#### L1-C03

Code, `src/cook/mod.rs:584-596`:

```rust
/// Cooks the sources `assets/manifest.json` lists that are present, and the
/// procedural Assets of `scenes`, into `cache`; `praxis cook` cooks every
/// Reference scene's.
pub fn cook(
```

`src/bin/praxis.rs:493-497`:

```rust
None => {
    let scenes: Vec<_> = (ReferenceScene::ALL.into_iter())
        .filter(|&scene| scene != ReferenceScene::World)
        .collect();
    praxis::cook::cook(assets, &scenes, &cache, &settings)?
```

ADR 0018, status line: "Plain `praxis cook` leaves out the `world` scene,
whose library of more than 8 GB cooks only as `praxis cook world` or on a
miss (amended 2026-11-08 [...])". `git blame` gives the doc comment
`81f25800c` (2026-10-19) and the filter `2dbd1ec19` (2026-11-08), inside
the scope, which made the comment false without editing it. The
`CLAUDE.md` checklist: "each clause the change makes false (a doc
comment, [...]) is rewritten or dated in the same PR".

#### L1-C04

Code, `src/cook/export.rs:187-203`:

```rust
for (transform, mesh_handle, material_handle) in entity_renderables.chain(placements) {
    let mesh = exported.mesh(recorder.meshes[mesh_handle.index as usize])?;
    let material = match material_handle {
        [...]
        // As prepare skips one whose Primitives all have triangles.
        None if exported.meshes[&mesh].materials.contains(&None) => {
            tracing::warn!(mesh, "Renderable has no Material Handle and a Primitive of its Mesh names no Material, skipping it");
            continue;
        }
        None => None,
    };
```

Prepare's rule, `src/render/prepare.rs:775` with
`mesh.needs_material_handle`, which the Mesh pool sets at
`src/assets/pools.rs:511-513`:

```rust
needs_material_handle: mesh_use.drawn()
    && (mesh.primitives.iter().zip(materials))
        .any(|(primitive, material)| material.is_none() && primitive.index_count > 0),
```

and prepare's doc, `src/render/prepare.rs:533-535`: "or that has no
Material Handle where a Primitive with triangles names no Material, is
skipped with a warning". ADR 0026, *Settled by the draw*: "A Primitive
without triangles needs no Material". The export's test reads only
whether a Primitive's Material is `None` (`MeshEntry::materials`,
`:640-648`), not whether that Primitive has triangles, and its comment
claims prepare's rule. The module's doc (`:1-3`): "a Reference scene as
prepare reads it from its World". A Renderable without a Material Handle
whose Mesh has an empty Primitive naming no Material, as a Far proxy of
open land is (ADR 0026, *Open land*), is drawn by praxis and left out of
`scene.json`. `export-scene` writes no Far proxy, and no generated Mesh at
this commit was found with such a Primitive, so no export differs today;
the class is `correctness` for the disagreement with the rule the line
cites. The verifier can confirm by listing, per scene, the Meshes with a
Primitive of `index_count` 0 and `material` `None`.

#### L1-C05

`src/cook/launch.rs:172-178`:

```rust
            SourceEntry::Chunk(index) => {
                let (scene, coordinates) = sources.chunks[index];
                let generated = self.sources.chunk(scene, coordinates)?;
                if generated.kept {
                    self.sources.kept_chunks.push(id);
                    return Ok(());
                }
```

The named Assets are cooked only on the next line's path, which a kept
Chunk never reaches (`:179-182`: `let named = generated.references().collect();
self.cook_after_named(cache, id, &generated.cook_key, named, ...)`).

A streamed load does not pass through the launch's cook for anything but
the Chunk's own ID: `FrameIo::request` runs `cook_chunk(id)` with the Chunk's
ID alone (`src/streaming/io.rs:1518-1529`), and the IO pool then reads each
Mesh, Material and Texture the Chunk names straight from the cache
(`:1437-1466`: `let mesh = reader.mesh(id)?;` ... `let material =
reader.material(id)?;` ... `let texture = reader.texture(id)?;`). No Chunk
loads at spawn (`src/state.rs:81-82`, "and no Chunk loaded"), so every
Chunk takes this path. `Sources::chunk` warns only of the unread Mesh
(`src/cook/mod.rs:254-258`: "a Placement's Mesh is not among the sources
this cook reads; the Chunk's Blob, if any, stays as it is"), not of the
Assets left uncooked.

Reachable: a scene's Chunk is kept when a Placement names a Mesh of a source
the launch could not read (`mod.rs:794-798` marks an absent file kept and
reads none of its Meshes). `helmet`'s only source is SciFiHelmet
(`src/scenes/mod.rs:144`), and its Chunk places SciFiHelmet's Mesh 11 times,
10 of them under procedural sweep Materials, 2 metallic by 5 roughness
values (`src/scenes/mod.rs:726-744`, `:763-765`).
With SciFiHelmet absent but its Blobs cached, a launch of `helmet` keeps the
Chunk, and a sweep Material whose factors changed in code loads its old
Blob. `praxis cook helmet` in the same state cooks every Material it reads
whatever the Chunks (`mod.rs:645-660`), so the two cooks disagree.

The clause, ADR 0018 status line: "A dev launch cooks only what it loads:
each Asset as its load finds the Blob missing or stale, with the Assets it
names, while `praxis cook <scene>` cooks a scene whole (amended 2026-11-08,
[...])". M4 story 64: "I want a dev launch to cook a Chunk and the
Assets it names when its load misses or finds them stale".

The Far proxy branch has the same shape (`launch.rs:188-190`, a kept proxy
returns before its ground's Materials are cooked); no scene can reach it,
as `world`, the one scene with a Far field, reads no source file
(`src/scenes/mod.rs:155`), so it is not filed apart.

For the verifier: a test beside `a_chunk_placing_a_removed_mesh_warns_and_loads_its_stale_blob`
(`launch.rs:524-556`) whose `Sources` hold a Chunk placing one read Mesh
under a read Material and one unread Mesh, with the Material's Blob cooked
under an old key, then `cook_for_scene` called with the Chunk's ID alone,
as streaming calls it, finds the Material's old key still in the index.

#### L1-C06

`docs/adr/0011-gltf-authoring-cooked-runtime-formats.md:3`: "Loading glTF
at runtime was a development convenience until [...]; since then a
development launch reaches glTF only through the cooker, which cooks what
changed before the runtime loads Blobs (amended 2026-10-19, [...])."

The code since the dev launch's cook on a miss:
`src/cook/launch.rs:1-2`: "What a dev launch cooks (ADR 0018): each Asset it
loads whose Blob is missing or stale, as it loads it, with every Asset that
one names." `src/state.rs:204-206`: "A dev launch cooks each Asset it loads
that is missing or stale, as it loads it (ADR 0018)". `CookingLoader::new`
"reads the scene's sources and cooks nothing yet" (`launch.rs:46-48`), and
the test `loading_one_chunk_cooks_that_chunk_and_the_assets_it_names`
asserts an empty index before the first load (`launch.rs:363`). Streaming
cooks a Chunk "on the main thread, before the IO pool reads it"
(`src/streaming/io.rs:814-815`).

ADR 0018 was amended for the change (status line: "A dev launch cooks only
what it loads ... (amended 2026-11-08, [...])"); ADR 0011's sentence
was not. Read per Blob ("cooks what changed before the runtime loads [it]")
it holds but for L1-C05; read as written, a cook of what changed ahead of the
runtime's loads, it describes the launch before the amendment. The rule it
falls under, `CLAUDE.md`'s review checklist: "each clause the change makes
false ... is rewritten or dated in the same PR".

#### L1-C07

`src/assets/material.rs:984-989`:

```rust
        let channel_textures = &mut self.channels[channel as usize];
        let Some((placement, resident_level)) =
            channel_textures.push(device, staging, &texture, holds_back_finer_mips)
        else {
            return Ok(Self::DEFAULT_LAYER);
        };
```

`push` returns `None` only when `place` finds no room for the tail
(`:361`), after `place` logs `tracing::error!(... "channel has no room for
another Texture class; the Texture keeps its Resident tail, or Layer 0
without one")` (`:419-425`). `load_making_room` writes that Layer 0 into
the Material (`:902-905`, `:927-928`) and the Material loads. The pool's
refusal counter counts Materials only: `refused_as_full` is its one
increment (`:679-689`), and `take_refusals` hands it to
`src/streaming/io.rs:925` as `materials_refused`, beside `meshes_refused`
(`:924`); no counter takes a Texture's. The Texture is also not entered in
`loaded_textures`, so each later Material naming it reads and decodes its
Blob again and logs again (M3.5's note, still true). The chain-refused case
at `:370-372` (tail kept) is likewise uncounted, but ADR 0017's 2026-11-20
amendment admits "drawn from its tail only where its Channel has no room
for that chain", so it is named here and not filed. The M3.5 report filed
the same shape for Materials (M3.5's L1-C04, a Material past capacity drawn
as Material 0), which `insert_as` now refuses (`:653-655`). Run: a
`MaterialPool` with `texture_classes_per_channel: 2` (the neutral class and
one), two Textures of different shapes, and `take_refusals` read after the
second.

#### L1-C08

`src/assets/material.rs:1047`, `:1060-1063`:

```rust
        let (texture_channel, target_shape) = (loaded.channel, loaded.shape.mips_from(first_level));
        [...]
        let channel = &mut self.channels[texture_channel as usize];
        let Some(target) = channel.place(device, target_shape) else {
```

`mips_from` (`:118-125`) is `width >> level`, `height >> level`. A class
that opens is created at that size (`:430-448`). wgpu-core refuses a
compressed texture whose width or height is not a multiple of its block
(`vendor/wgpu-core/src/device/resource.rs:1438-1449`,
`TextureDimensionError::NotMultipleOfBlockWidth`), and the `wgpu` crate's
default handler panics on it (`wgpu-30.0.2/src/backend/wgpu_core.rs:692-694`:
`panic!("wgpu error: {err}\n")`); nothing in `src/` installs another
handler. The only whole-block check is the tail's
(`src/assets/cooked.rs:264-267`), whose comment states the rule: a mip "not
whole blocks, which a Texture of its own cannot start with"
(`cooked.rs:249-252`). The reader accepts any side that is a multiple of 4
up to 8192 (`cooked.rs:445-451`), as the cooker does
(`src/cook/texture.rs:170`), and the cook's mip filter expects
non-power-of-two Textures (`src/cook/texture.rs:452-453`). Worked case,
2,052² BC7 (`level_bytes`, `cooked.rs:288-299`): mip 0 513² blocks ×
16 = 4,210,704; mip 1, 1,026 texels, 257² × 16 = 1,056,784; mip 2, 513
texels, 129² × 16 = 266,256; mip 3, 256 texels, 64² × 16 = 65,536 ≤ 65,536,
and 256 is a multiple of 4: tail at mip 3. A Chunk loads it as its tail;
the plan asks any mip finer (`src/streaming/mips.rs:213-220`,
`levels: need.ahead..resident`); a move to mip 1 opens a 1,026 × 1,026 class
and a move to mip 2 a 513 × 513 one, neither a multiple of 4. The same holds
for `evict_finest_mip`'s coarser class (`:1185`, `:1187-1188`) and for
`push`'s chain from a `down_to_from_blob` read (`:370`), whose picked mip is
not checked either (`cooked.rs:393-396`). A spawn's full chain is safe: its
mip 0 is checked. Run: load a 2,052² BC7 Texture through `TailReader` and
call `stream_mips` to mip 1, as `a_full_channel_evicts_...` does for
1,024².

#### L1-C09

`src/assets/cooked.rs:260-268`:

```rust
    (0..mip_level_count)
        .find(|&level| {
            level_bytes(format, width, height, level as usize) as u64 <= RESIDENT_TAIL_LEVEL_BYTES
        })
        .filter(|&level| {
            (width >> level).is_multiple_of(block_width)
                && (height >> level).is_multiple_of(block_height)
        })
        .unwrap_or(0)
```

ADR 0017 (line 16): "Each texture's Resident tail (mips of 64 KiB and
smaller) never leaves"; ADR 0018 (line 21): "the Resident tail (mips of 64
KiB and smaller, ADR 0017)". With level 0 the tail, `push` writes every mip
into the tail's class and places no chain (`material.rs:356-368`), and
`evictable` never takes it (`:1131`). Worked case, 1,000² BC7 (10 mips):
mips 0 to 9 are 1,000,000, 250,000, 63,504, 16,384, 4,096, 1,024, 256, 64,
16, 16 bytes; mip 2 (250 texels) is the first of 64 KiB or less and 250 is
not a multiple of 4, so the tail is level 0: 1,335,360 bytes, where the
ADRs' tail, mips 2 to 9, is 85,360. `CONTEXT.md`'s Resident tail ("the
smallest mips of a texture") states no size and is not contradicted. The
mechanism differs from L1-C08, where the tail's mip is whole blocks and a finer
one is not.

#### L1-C10

`src/assets/cooked.rs:505-510`:

```rust
    /// UV units per mesh-space metre: the square root of its triangles'
    /// summed UV area over their summed mesh-space area, zero when either sum
    /// is, as without UVs; the inverse of [ref:unity-6000-uv-distribution-metric]
    /// as a length. [...]
    pub texel_density: f32,
```

`docs/references.toml`, `unity-6000-uv-distribution-metric`: "The UV
distribution metric can be used to calculate the desired mipmap level based
on the position of the camera. [...] Average of triangle area / uv area.
[...] // uvDistributionMetric is the average of triangle area / uv area".
Unity's is one figure per Mesh, an average; the code's is one per
Primitive, a ratio of sums (`src/cook/mesh.rs:435-452`). ADR 0018 line 24
says so ("taken per Primitive, as a ratio of summed areas and as a
length"), which is a record in an ADR, not a row: `deviations.md` lines
10-11, "A Deviation lives here and nowhere else", and its table has no row
for this key. The M5 source check (`docs/audit/m5/sources.md:237`) confirmed
the quote, not the difference. The cook's function is reader A's; this
Finding is the citation in this file.

#### L1-C11

`src/assets/material.rs:419-425`:

```rust
        let Some(placement) = self.table.place(shape) else {
            tracing::error!(
                channel = self.label,
                capacity = self.table.capacity,
                "channel has no room for another Texture class; the Texture keeps its Resident tail, or Layer 0 without one"
            );
            return None;
        };
```

`stream_mips` calls this `place` for the move's class (`:1061`) and returns
`StreamedMips::Refused` on `None` (`:1062`), with the Texture's placement
and `resident_level` untouched (`:1082-1085` are not reached), so it keeps
its earlier chain, which may hold mips finer than its tail. The message is
true for `push`'s tail and chain (`:361`, `:370-372`). The M5.5 report's
L1-F02 says the refused move is "neither logged nor counted" and its note
says `material.rs:1061-1063` "returns `Refused` without a log"; the refusal
is logged here at `error`, though not counted (see *Notes*).

#### L1-C12

Code, `src/scenes/mod.rs:78-81`:

```rust
    /// A generated Game world of 40 × 40 Chunks, settlements with sparse
    /// farms between them, whose library of Textures outgrows the texture
    /// pool, seen within its 256 m Load radius (`world.rs`).
    World,
```

and `:573-578`:

```rust
    /// The scene's View distance, in metres: the world's 2 km, or the Load
    /// radius of the others, which have no Far field.
    pub(crate) fn view_distance_metres(self) -> f64 {
        match self {
            Self::World => 2000.0,
```

ADR 0026, Decision: "Every Chunk outside the Load radius and within the View distance draws its Far proxy, one Instance, in place of its Placements." and "Past the View distance. Nothing is drawn but flat ground reaching the horizon, one Resident quad." `spawn_world` places that ground (`src/scenes/world.rs:244-264`). `git log -S` puts the doc line in the commit that generated the world (2026-11-08, M4, when nothing drew past the Load radius, as M4 story 66 asked) and the far field's draw in a later one (2026-11-18). 2,000 / 256 = 7.8.

#### L1-C13

Code, `src/scenes/mod.rs:378-380`:

```rust
    /// The coordinates of the Chunks the scene's static content lies in, x
    /// then z, in coordinate order: the Chunks its spawn loads.
    pub(crate) fn chunk_coordinates(self) -> Vec<[i32; 2]> {
```

The State builds its World with `Simulation::spawned` (`src/state.rs:223`), whose doc ends "and no Chunk loaded" (`src/state.rs:79-82`), and hands `chunk_coordinates()` to the Chunk streamer as the index of existing Chunks (`src/streaming/io.rs:787`, `:794`), which loads only those within the Load radius. `Simulation::new`, which calls `load_chunks` over the whole list (`src/state.rs:60-77`), is called only from `src/cook/chunk.rs:185`, `:377` (tests) and `src/cook/export.rs:72-76` for a scene without a Far field. The line was written in the commit that loaded every scene's static content from its Chunks (2026-11-07); the State already used `Simulation::spawned` at the commit that streamed every scene's Chunks through the IO pool (2026-11-08), which made it false. M4 spec, Implementation decision: "The spawn dispatch no longer builds static content: the Chunk streamer loads it."

#### L1-C14

Code, `src/scenes/world.rs:53-60` and `:191-195`:

```rust
/// The settlement whose region the camera flies into, column then row.
const CAMERA_SETTLEMENT: [u32; 2] = [4, 4];
/// The camera flies north along a street, over the roofs and the birds, at
/// 50 m/s, from farmland `APPROACH_METRES` south of its settlement's edge,
/// looking at the ground `GAZE_METRES` ahead.
const CAMERA_HEIGHT_METRES: f64 = 60.0;
const CAMERA_METRES_PER_FRAME: f64 = 50.0 / 60.0;
const APPROACH_METRES: f64 = 300.0;
...
    let (street_x, start_z, frames_flown) = if frame < FAST_TRAVEL_FRAME {
        let centre = settlement_centre(CAMERA_SETTLEMENT);
        let start_z = centre.z + SETTLEMENT_HALF_SIZE_METRES + APPROACH_METRES;
        (centre.x - BLOCK_PITCH_METRES / 2.0, start_z, frame)
```

Arithmetic, from `settlement_centre` (`:171-183`) with `REGION_METRES` 1,280, `SETTLEMENT_STRAY_METRES` 640 − 377 − 32 = 231 and `SETTLEMENT_HALF_SIZE_METRES` 13 × 58 / 2 = 377, and `random` (`src/scenes/town.rs:293-304`) recomputed by this reader:

- Settlement [4, 4]: `random([4, 4, 0, 0])` = 0.237102, `random([4, 4, 1, 0])` = 0.762786; centre x = 640 + 231 × (2 × 0.237102 − 1) = 518.54, z = 640 + 231 × 0.525571 = 761.41; south edge z = 761.41 + 377 = 1,138.41.
- Frame 0: x = 518.54 − 29 = 489.54, z = 1,138.41 + 300 = 1,438.41.
- Settlement [4, 5]: `random([4, 5, 0, 0])` = 0.036441, `random([4, 5, 1, 0])` = 0.187893; centre x = 640 − 231 × 0.927117 = 425.84, z = 1,920 − 231 × 0.624213 = 1,775.81; its square spans x 48.84 to 802.84 and z 1,398.81 to 2,152.81.
- Frame 0 lies in that square, 1,438.41 − 1,398.81 = 39.6 m south of its north edge; at 50/60 m a frame the camera leaves it at frame 48 (z = 1,398.41). The open land between the two settlements is 1,398.81 − 1,138.41 = 260.4 m deep.

The test module's own helper settles it on a run: `over_a_settlement(camera_at(0).position)` (`:746-754`) returns true; the bench path's test checks frames 599, 1,799 and 1,800 only (`:765-768`). `SETTLEMENT_STRAY_METRES`' own doc guarantees only 64 m of open land between neighbours (`:39-41`), so a 300 m approach can start over the neighbour for some seeds; for this one it does. The bench measures from frame 248 (`src/bench/mod.rs:64`, `:76`, `:117`), and the first Golden is frame 59, 250.8 m short of settlement [4, 4] over open land, as `tests/goldens.rs:315-318` says.

#### L1-C15

Code, `src/scenes/town.rs:56-59` and `:111-117`:

```rust
impl TownMesh {
    /// Every Mesh, in the order of its discriminant, which indexes its
    /// Handle when loaded.
    const ALL: [TownMesh; 21] = [
...
const _: () = {
    let mut index = 0;
    while index < TownMesh::ALL.len() {
        assert!(TownMesh::ALL[index] as usize == index);
        index += 1;
    }
};
```

and the same for `BoxMaterial::ALL` (`:133-145`, "Every Material, in the order of its discriminant, which indexes its Handle when loaded.", asserted at `:162-168`). `grep -rn "TownMesh::ALL\|BoxMaterial::ALL" src` at the frozen commit finds only those four assertion lines. At `8bc34eb1`, `spawn_town` loaded every Mesh and Material through these arrays and indexed the Handles by discriminant (`let meshes: Vec<MeshHandle> = TownMesh::ALL ...`, then `MeshRef(meshes[renderable.mesh as usize])`); `git log -S` puts the removal of that load in the commit where `town` spawned its static content from its cooked Chunks (2026-11-07). The cook now names Meshes and Materials by `name()` alone (`:409-415`, `:1230-1233`). Lines: `:57-81` (25 with the doc), `:111-117` (7), `:134-145` (12 with the doc), `:162-168` (7): 51; the enums' `name()` methods stay in use.

### Notes for the verifier

Four of the drift Findings are in documents lane 7 also reads: L1-C01
(ADR 0018's cap normals), L1-C06 (ADR 0011's dev launch), L1-C09 (ADR
0017's and ADR 0018's Resident tail) and L1-C10 (a `deviations.md` row
missing for the texel density); where lane 7 files the same sentence, the
triage keeps one. The tooling section's notes point at `src/cook/mod.rs:585`,
which is L1-C03 here. Reader C reads the streaming section's L1-F02 from
the pool's side: the refused mip move it calls "neither logged nor
counted" is logged at `error` by `ChannelTextures::place`
(`src/assets/material.rs:419-425`) and is only uncounted, so the verifier
may narrow L1-F02; the log line itself misdescribes the move, which is
L1-C11. M3.5's L1-C01, the dangling-reference warning, is fixed in scope
but for a scene with an absent source, which reader A records with its
ceiling and reader B for the launch. Reader A also settles a question the
shaders section's L1-S10 leaves: meshoptimizer 1.3's reported error
includes the weighted attribute error, unclamped, so the "metres" that
ADR 0018 and `CONTEXT.md` give it is not a geometric distance alone.

#### the cooker's Meshes, Textures, scene cook and export

Not filed, since each is a reading without a quoted clause it breaks, a
question only a run can settle, or another reader's; recorded so the next
audit does not re-find them.

- **The LOD error includes the attribute error.** This settles the
  question the shaders section's notes leave open (whether
  `simplify_with_attributes_and_locks`'s `result_error` includes the
  weighted attribute error): it does. In meshoptimizer 1.3,
  `rankEdgeCollapses` adds each collapse's attribute quadric error to its
  position error when attributes are given (`simplifier.cpp:1430-1433`,
  with seam and complex wedges at `:1435-1469`, the collapse's error set
  at `:1477`), `performEdgeCollapses` keeps the largest collapse error
  (`:1643`), and the result is returned
  as `sqrtf(result_error) * error_scale` (`:2620-2622`). The cook asks for
  normals at weight 0.5 and UVs at weight 1.0 and no
  `meshopt_SimplifyErrorClamped`, whose documentation is "Clamp attribute
  error to match position error scale and avoid extreme error values in
  areas with high attribute variance" (`meshoptimizer.h:484-485`); the
  `meshopt` 0.7 commit's review deferred that option as a technique
  decision. `CookedLod::error_metres`'s doc says so ("of position, normal
  and UV together", `src/assets/cooked.rs:608-610`); `CONTEXT.md`'s
  **LOD error** ("how far a LOD strays from LOD 0, in metres") and ADR
  0018's "per LOD its error in metres" read as a geometric distance. The
  README quote under `meshoptimizer-1.3-attribute-weights` makes an
  attribute change "approximately equivalent" to a distance, so "metres"
  is defensible; the consequence is that selection draws a finer LOD and
  the shadow mask's `LOD_ERROR_MARGIN` passes higher back faces where
  normals or UVs vary, more of the shaders section's L1-S10 band. To size
  it, run the chain of `town`'s Meshes once more with
  `simplify_with_locks` (positions only) on the same LODs and compare the
  errors. Not filed: no clause quotes a positions-only error.
- **A fourth platform path in the Mesh cook.** `bevy_mikktspace` 1.0.0
  weights each corner's tangent by an angle from `(x as f64).acos() as
  f32` (`bevy_mikktspace-1.0.0/src/lib.rs:229-234`,
  `src/mikktspace.rs:737`), the platform's f64 `acos`. The `CLAUDE.md`
  checklist line on the Mesh Blob names glam's SIMD, a platform's f32 sine
  or cosine and a fused C++ build; this path is of the same kind, made
  rare by the narrowing (Block A1, step 3), and the hash agrees on the
  machines that run it today. `uv_sphere`'s comment
  (`src/scenes/mod.rs:1440`, reader D's) also presents f64 `sin_cos` as
  machine-independent, which the same narrowing argument makes rare, not
  impossible. A line in the checklist naming any platform `libm` call
  would cover both.
- **What the hash test cannot see.** The RTX 2070 SUPER's Windows build
  runs neither hash test in CI, and `.cargo/config.toml` sets
  `-ffp-contract=off` for no Windows target; commit `8ac44220` records
  both as not covered. SciFiHelmet's Mesh is outside the test.
- **The dangling-reference warning's ceiling.** While any source of the
  referring scene is absent, every ID the Cooked cache holds passes for
  that scene's Scene and Chunk references, a stale one included, even an
  ID of another, present source of the scene (`src/cook/mod.rs:514-525`,
  `:919-921`); the `ponytail:` comment states it. A Far proxy's
  references, the ground's Materials, are checked by a dev launch
  (`src/cook/launch.rs:193-195`) and not by `praxis cook`, where the
  ground Mesh's own Mesh-to-Material reference covers them whenever the
  ground is read. In `cook_sources` no reference names a Chunk, so
  `may_be_kept`'s `kept_chunks.contains(id)` (`:515`) never matches
  there; it matters for the dev launch, which looks Chunks up by ID
  (`src/cook/launch.rs:176`, `:239-240`).
- **Unity's "average".** `unity-6000-uv-distribution-metric`'s quote,
  "Average of triangle area / uv area", does not say whether Unity
  averages per-triangle ratios or divides summed areas. ADR 0018 states
  the code's form ("as a ratio of summed areas"), so a reader is not
  misled; a `deviations.md` row is due only if Unity's metric is the mean
  of ratios, which the registry's quote cannot settle.
- **The image decoder is outside the Texture Cook key.** A Texture's key
  hashes its encoded source bytes and the encoder (`intel_tex_2 0.5.0
  basic`), not the `image` crate that decodes PNG and JPEG
  (`src/cook/texture.rs:116-130`, `:135-158`). A decoder release that
  decoded a JPEG differently would leave Blobs fresh that a fresh cook
  would write otherwise. ADR 0018 lists "the encoder versions", so no
  clause is broken.
- **`with_fastest_bc7` on a baked metallic-roughness Texture.**
  `cook_far_proxy` sets it on both baked Textures (`src/cook/mod.rs:450`);
  BC5 never reads the BC7 settings (`src/cook/texture.rs:648-656`), so the
  flag changes nothing there. Harmless.
- **The world test's "each of one size".** The doc at
  `src/cook/mod.rs:1231-1234` says each of `world`'s Textures is of one
  size; the test asserts it of four sampled Textures and multiplies one
  chain by the count.
- **DIFFERENCES.md and ADR 0026 on a Far proxy past the View distance.**
  The *Far proxies against the Chunks' own Meshes* entry says praxis
  "keeps a Far proxy Resident, and draws it, out to a Chunk's edge past
  the View distance"; ADR 0026's *Settled by the draw* holds a Far proxy,
  its slot an Instance with no Mesh, while it is past the View distance.
  `export-scene` writes the Chunks within the View distance
  (`src/cook/export.rs:254-257`, by `distance_metres`), as the entry
  says. Which of the two describes what praxis draws is streaming's to
  judge.
- **Small Instances in a Cycles rebuild.** `export-scene` writes each
  Renderable at its true transform, as prepare reads it; ADR 0027 then
  scales a small Instance to a pixel in the culling and vertex stages.
  `DIFFERENCES.md` has an entry for Thin Primitives and one for coverage
  quantised by 4x MSAA, and none naming small Instances; whether the MSAA
  entry covers `town`'s far pigeons is the tooling reader's to decide.
- **M5 story 15** (weights tuned on the gallery) cannot be settled from
  code; the weights are the first chain commit's.
- **Sound but subtle, checked:** the cut computed before
  `optimize_vertex_fetch` holds after it, since renumbering is a
  bijection (`src/cook/mesh.rs:212-219`); meshoptimizer extends a lock on
  any wedge to every vertex at its position (`simplifier.cpp:510-517`), so
  a thin tube's triangles survive even where another Primitive touches
  it; the 85% comparison keeps a LOD of exactly 85%, since 0.85 in f32 is
  0.85000002384 (`:371`); `export-scene` composes every translation in
  f64 and narrows only rotation and scale; the Far proxy's Cook key
  hashes each drawn Material once in sorted order, the placed Meshes in
  the order the sources list them and a Texture once per Material that
  samples it (`src/cook/mod.rs:324-361`), deterministic for one source
  list and each part a fixed-length hex key, so no two inputs share an
  encoding.

#### the world's cook: Chunks, the dev launch's cook and Far proxies

- **The atlas's gutters.** A chart's padded block starts at a column and row
  that are multiples of 4 (`bake.rs:206-212`: `column` advances by padded
  widths, `row` by shelf heights, both multiples of `GRID_TEXELS`), the chart
  2 texels in. At mip 2 the chart's first edge is at (col + 2) / 4 = a texel
  centre of the block's first mip-2 texel, and its far edge at
  (col + 2 + size) / 4 ≤ (col + padded) / 4 − 0.5, the block's last centre,
  since padded ≥ size + 4. A UV held inside the chart, bilinear at mip 2,
  therefore weights no texel outside its block. At mip 3 a block of 4 may
  share a mip-3 texel with its neighbour, which ADR 0026:32 admits.
- **BC blocks across charts.** The analysis above is of filtering. At mip 1
  and coarser, a BC7 or BC5 block of 4 × 4 texels spans 8 × 8 or more mip-0
  texels, so up to four charts' blocks; the encoder's endpoints then mix
  their colours. `no_chart_bleeds_into_another_down_to_mip_2` samples the
  bake's uncompressed `Mips`, not the encoded Blob. ADR 0026:34's reopen
  trigger ("a proxy whose capture shows the encoder's blocks") covers what
  a capture would show; a crop of `world`'s densest Chunk's proxy at the
  Load radius, at mip 1 and 2, would settle whether it shows.
- **A Placement the runtime does not draw still joins the shell.** `placed`
  keeps such a Placement's triangles with no Material (`far_proxy.rs:65-94`);
  they are a drop surface and are merged and charted, and their chart reads
  no source, so the proxy draws them in the atlas's mean colour, which
  `no_chart_bleeds_into_another_down_to_mip_2` asserts on purpose
  (`far_proxy.rs:692-703`). ADR 0026 says only that such a Placement is no
  source of the bake. Whether a Far proxy should draw what its Chunk does
  not is the owner's question; no Reference scene places such a Placement
  that this reader found.
- **The fill is a truncated mean.** `bake.rs:360-361` divides integer byte
  sums, `(component / count.max(1)) as u8`, so the fill is the mean rounded
  down, up to one code under it. ADR 0026:32 says "the mean of the baked
  texels as stored". Below the lane's resolution; not filed.
- **A pivot a hair under a Chunk edge.** `chunk_holding` floors exactly, but
  `split`'s `placement.translation -= origin` rounds to 256.0 for a pivot
  within 2.8e-14 m under a Chunk's lower edge (half an f64 ulp at 256), so
  such a Placement sits at 256.0 from its Chunk's origin, not in [0, 256).
  The World position stays exact to that rounding; the town test's
  `(0.0..256.0).contains` (`chunk.rs:248-251`) would catch one on `town`.
  No generator places one.
- **Thread and hash map order.** Chunk order is a `BTreeMap`'s; Placements
  keep the generator's order; every `HashMap` in these files is a lookup.
  The bake's workers take charts from an atomic counter (`bake.rs:332-353`),
  but each tile is written to its own rectangle and the fill sums integers,
  so the atlas does not depend on thread order.
- **What a stale Far proxy key misses.** Checked and found complete: the
  key covers every constant of the drop, the merge, the LOD chain, the
  chart, the pack and the bake (`far_proxy.rs:32-38`, `bake.rs:23-27`),
  the pixel through its f32 `Display`, which round-trips; the Mesh encoder
  string, which names meshoptimizer 1.3 and vertex codec 1 (`mesh.rs:45-47`,
  matching `Cargo.lock` and `meshoptimizer.h:15`); the Texture encoder and
  format (`texture.rs:277-286`). Only the spatial grid's 16 m cell
  (`GRID_CELL_METRES`) and the atlas ceiling are left out, neither of which
  changes a cooked byte.
- **`ErrorAbsolute`'s key.** The registry carries the flag only as
  `meshopt-0.6.2-simplify-options`, whose quote names `const ErrorAbsolute = 4`
  in 0.6.2; `Cargo.lock` pins 0.7.0. Nothing in these files cites the key,
  and the vendored `simplifier.cpp:2489`, `:2621-2622` show the absolute
  error under 0.7.0, so no Finding; the registry's maintainer may want a
  0.7.0 entry.
- **The launch's warnings for an absent source.** `may_be_kept` excuses every
  cached ID a scene with an absent source names (`mod.rs:514-525`, `:919-921`,
  a `ponytail:` ceiling); for the launch this is the remaining part of
  M3.5's L1-C01, reader A's to weigh.
- **The densest Chunk's LOD 0.** Block B4's tightness rests on the shell
  keeping all 5,832 triangles. If the verifier wants it confirmed, the test's
  `shell.index_count / 3` printed for Chunk (-19, -3) settles it.
- **A cook on a miss inside a frame.** `FrameIo::request` cooks on the main
  thread before a read (`src/streaming/io.rs:1524-1529`); `cook_ahead`
  (`:825-847`) moves the cooks ahead for the cameras it is given. A Far
  proxy's bake measured 7.5 core-hours for `world` at a texel a pixel
  (ADR 0026:34), so a launch whose camera meets an unbaked proxy outside
  `cook_ahead` stalls that frame. Streaming's and tooling's lane, if any.
- **Repeated warnings.** `far_proxy_mesh` calls `Sources::chunk` for the
  Chunk and each neighbour again after `far_proxy` did (`mod.rs:480-486`), so
  a kept Chunk warns up to once per proxy that reads it. Noise, not a defect.
- **Not re-found.** `DIFFERENCES.md`'s Far proxy roofs a pixel off, and the
  shaders section's notes on Far proxies under point lights and
  `COMPOSITION_ERROR` past 128 m, are the same mechanisms seen from the
  draw; nothing in these files changes them.

#### assets: the Cooked cache reader, the loader, the pools and their Arenas

- M3.5's assets Findings are closed at this commit: L1-C02, the vertex
  count now bounded by the encoded bytes before `decode_vertex_buffer`
  allocates (`cooked.rs:784-793`, test `:1513-1530`; `meshopt-0.7.0/src/encoding.rs:175`
  still allocates first); L1-C03, the normal Channel's Layer 0 now
  `Rgba16Float` 0.5, 0.5, 1, exact (`material.rs:528-547`); L1-C04, a
  Material past its count now gets no Handle (`material.rs:653-655`, test
  `:1921-1954`); L1-C06, ADR 0018's status sentence now names the Meshlet
  Arenas.
- The M5.5 report's L1-F02 says a refused mip move is "neither logged nor
  counted", and its streaming note says `material.rs:1061-1063` returns
  `Refused` "without a log". `ChannelTextures::place` logs every refused
  placement at `error` before returning `None` (`material.rs:419-425`), the
  move's included. The verifier may want L1-F02's claim narrowed to
  "counted"; L1-C11 is the misleading wording of that log line.
- offset-allocator 0.2.0 bins a free node by rounding its size down and
  searches from the request rounded up, with three mantissa bits
  (`offset-allocator-0.2.0/src/lib.rs:207`, `:385`; `small_float.rs`). So a
  request is refused whenever no bin value lies between it and the largest
  free node, which happens with the node up to about 12.5% larger than the
  request: the refusal test's index Arena holds 25 free elements in the bin
  of 24 and would refuse 25 (Block C5). `Arena::longest_free_range` reports
  that bin value (`lib.rs:495-517`), which is what `allocate` guarantees,
  so the doc comment at `pools.rs:1379` and the refusal log are right; but
  a refused load then frees unused Meshes (`pools.rs:389-401`) for room
  that was there. Not filed: ADR 0017 chooses the crate, and the clause
  says "does not fit" without a granularity. A test that fills an Arena to
  leave a free node of 2^k + 2^(k-3) + 1 elements and loads one of that
  many would show it.
- A zero-element request still needs a free node: `allocate(0)` takes the
  lowest non-empty bin (`lib.rs:207-238`) and returns `None` when none is
  free. A `MeshUse::Traced` load takes no Meshlet element and a
  `MeshUse::Drawn` load no index (`pools.rs:1264-1283`), yet either is
  refused as "an Arena's longest free range is too short" when the Arena it
  takes nothing from is filled exactly or out of nodes (131,072 per Arena,
  above the Mesh table's 16,371). Not reached by any test; noted, not
  filed.
- `CookedCache::read` returns the Blob the index names without hashing it
  (`cooked.rs:1138-1152`), and `content_hash` hashes the index's names, not
  the bytes read (`:1106-1117`). A Blob changed in place under its name
  loads, if it parses, and moves no content hash. ADR 0018 names Blobs by
  their hash and requires no check on read, so nothing is filed; the
  verifier may judge whether the Bench history's content hash should rest
  on the bytes.
- `zstd::decode_all` decodes a Mesh or Chunk Blob's payload with no bound
  (`cooked.rs:705`, `:965`); a Texture's levels are bounded by their
  expected size (`:465`). A corrupt frame can inflate before any count is
  checked. A local, disposable cache; not filed.
- Still as M3.5 noted: the Texture reader requires `height > 0` but not
  `width > 0` (`cooked.rs:445-447`), so a hand-edited Blob of width 0
  reaches `create_texture` and wgpu's panic; the reader does not check a
  Texture Blob's format against its Channel (`:433-441`).
- `texel_density` and the bounds are read without a finiteness check
  (`cooked.rs:715-718`, `:722`), where the thin radius and the LOD error are
  checked (`:724-727`, `:739-742`). The cooker writes finite values.
- `mips_from_blob` truncates to `end - first_level` in u32 (`cooked.rs:404-406`):
  a range ending below the level it starts from would underflow; its one
  caller passes `need.ahead..resident` with `need.ahead < resident`
  (`src/streaming/mips.rs:213-219`).
- L1-C08 and L1-C09 share a root, the whole-block check made at the tail's mip
  alone, but are two defects: L1-C09 Textures never stream, L1-C08 Textures stream
  into an invalid class. A Texture whose side is a power of two times a
  multiple of 4 at every mip down to its tail meets neither: `world`'s
  facades are 2,048 (`src/scenes/world.rs:86`). The helmet sources are not
  in the worktree; listing every cooked Texture's sides (`praxis cook` then
  the KTX2 headers) would say whether any Reference scene has one.
- Sound and subtle, checked: the two Material counts share one
  `Generations`, and since each count's live entries stay within its
  capacity, every index stays under 222 + 285 and inside the SSBO
  (`material.rs:570-580`, `:653-667`); `written_bytes` counts exactly what
  `load_for` writes (Block C4); the copies' mip arithmetic in `push`,
  `stream_mips` and `evict_finest_mip` fills the target from its first mip
  to its last; the neutral class 0 slice 0 is never released, as the
  neutral Textures are not in `loaded_textures`; a tail mip of a BC Texture
  is never 0 texels tall, as the mip before it held more than 64 KiB in at
  most 2,048 blocks a row; `evictable` with streaming's `rank` (need at
  least two past the finest Resident mip, `mips.rs:291-293`) never evicts a
  mip a Texture needs.
- `compact_blases` drains whatever is left after its two waits with the
  warning "BLAS not compacted: not ready after its build"
  (`pools.rs:791-796`), also for an entry already compacted and only
  waiting for its copy's submission; after two blocking polls none should
  be left. Wording only.
- Textures a spawn loads by `AssetLoader::texture` (texture-wall's 10,000,
  `src/scenes/mod.rs:1071-1083`) count a user no Material holds and that no
  code removes, so they are never freed; by design for a spawn.
- The ledger charges a class's levels by size a slice (`src/stats/gpu_memory.rs:175-181`),
  not by role: a L1-C09 Texture's mips past 64 KiB, Resident for life, go to
  `TexturePool`, its smaller ones to `ResidentTails`.
- `GEOMETRY_BYTES_PER_MESH`'s "the mean of `town`'s Meshes when it had 19"
  (`pools.rs:31-33`) and `ARENA_PER_MILLE`'s measured shares need a cook
  and a fill to recheck; the arithmetic from them is Block C1's.

#### the Reference scenes

Not filed, since each is already filed elsewhere, predates the scope, needs a run, or breaks no clause; recorded so the next audit does not re-find them.

- L1-F41 (streaming section) is the one defect this reader found in the fixed-schedule systems of these files (`wrap_around_camera`, `src/scenes/mod.rs:921-934`); its clause rows above cite it.
- Mesh Blobs across machines. `uv_sphere` computes its trigonometry in f64 because "a platform's f32 sine and cosine round by machine, and a Blob's bytes must not" (`src/scenes/mod.rs:1440`), but `town`'s `cylinder` (`src/scenes/town.rs:1318-1321`) and `wire` (`:1383-1384`) call f32 `cos` and `sin`, and `moved` (`:1266-1275`) rotates by glam's SIMD `Quat`, both of which the `CLAUDE.md` checklist names as causes of a hash that moves on one machine alone. Fifteen of `town`'s 21 Meshes are built from them. The Reference scenes' Mesh hash (`src/cook/mesh.rs:993-1008`) pins the bytes on the machines that run it, so no clause is broken on the evidence here. `world`'s walls take their rotation from f32 `sin_cos` (`Quat::from_rotation_y`, `src/scenes/world.rs:659`) and its windows from `from_rotation_arc` and SIMD products (`:682`, `:688-689`); those reach the densest Chunk's Far proxy hash (`src/cook/far_proxy.rs:814-824`) through the walls and roofs it merges. To settle it on a machine no CI job is: run both hash tests on the RTX 2070 SUPER's Windows build, ADR 0013's hardware floor, with its own C runtime's `cosf` and `sinf`.
- Eviction on `world`. `world.rs:5` says "the scene's library outgrows the texture pool and freeing and eviction run", as M4 story 58 asks ("so that eviction must run"). The library is 8 GiB against a 4 GiB pool, but the fit test asserts that every camera's view, with every mip of every facade and atlas Resident, fits the pool (`src/scenes/world.rs:995`); freeing runs (the M4 Pro Baseline counts 1,126 `pool_entries_freed`, `bench/baselines/apple-m4-pro/world.json:334`), and Frame stats hold no eviction counter (`grep -i evict src/stats/mod.rs` finds none). Whether a Texture is ever evicted over the bench path or the traversal needs a run with a counter or a probe on `evict_finest_mip`.
- ADR 0017's "the 282 within the View distance and a Chunk's edge of a camera on a 64 m grid": the most Chunks within 2,256 m of such a camera is 285 (M5 story 22's figure, this reader's count), so the 282 must be the camera of most Arena bytes, which the fit test does not print (it prints each count's most over all cameras, `src/scenes/world.rs:964-971`). Settled by a run that prints the proxies at the camera of most vertex Arena bytes.
- ADR 0026's Considered options say "at most 219 proxies are drawn"; on the fit test's 64 m grid this reader counts at most 218 Chunks with a nearest distance in (256, 2,000] m, and 220 in [256, 2,000]. Not a decision clause and not in these files; for reader B if it weighs the sentence.
- `town`'s `placements` doc, "Every Instance the town places, block by block, then its lamps, poles and wires" (`src/scenes/town.rs:375-376`), omits the ground, which opens the list: 64 tiles now, one Placement at `8bc34eb1` under the same doc. The omission predates the scope, so it is not filed.
- Window and farm reach (checked, sound): a window's pivot lies at most 5 m from its building's on either axis (half a 10 m wall, or 4 m along it for six columns at 1.6 m), inside a lot's 6 m reach; a farm's building lies at most 48 + 5.3 m from its tile's centre, inside the tile's 64 m; a farm keeps clear of its own settlement (excluded under 377 + 64 = 441 m on both axes, its building at most 53.3 m from the tile's centre, so at least 10.7 m clear) and of the neighbour's (a settlement's edge is at least 32 m inside its region, and ground tiles align with regions, 10 to a region).
- Determinism of the generator (checked, sound within one machine): every choice is `random` of where the thing stands, in f64; farms seed by the tile centre's f64 bits; loops run in fixed order. One coincidence: a lot's pick `[settlement, 9, lot_row, 2..4]` equals a brick shade's `[facade, 9, course, brick]` when the global facade index equals the settlement index; it correlates two decorative values and breaks no clause.
- The far ground's depth (checked): `FAR_GROUND_DEPTH_METRES`' comment ties the depth to f32's 62.5 mm at 1,024 km, a horizontal spacing; `docs/performance.md:35` shows the depth it needs falls tenfold with a tenfold shorter reach (through at 0.005 m at 1,024 km and at 0.0005 m at 102.4 km), consistent with the comment, and 0.5 m is 25 times the shallowest depth that hid it (0.02 m), as ADR 0026 says.
- The far ground's `LoadedChunk` takes the ground Mesh's Asset ID as its Chunk ID (`src/scenes/world.rs:253`); no cooked Chunk shares it, so per-Chunk caches keyed by Chunk ID see a distinct key. Checked, sound.
- `town`'s lamp after its head became its own Mesh (checked): the light at 4.7 m lies under the head (4.85 to 5.05 m) and the arm (4.96 to 5.04 m), outside both Meshes, as the comment says (`src/scenes/town.rs:235-238`); the Lamp's Mesh is now Thin Primitives alone, so it takes its widest tube's width class, which the shaders section's L1-S12 reads; the head, an ordinary cuboid of steel with no emission, casts at every distance.
- Load radii (checked): `town`'s 1,024 m holds every Chunk from every frame of its path (the farthest Chunk's nearest point is about 842 m from the orbit at 45°); `far_coplanar`'s 4,352 m holds its farthest Chunk's nearest point at 4,237.6 m.
- The bench path test's "Past twice the 512 m hold radius, so no Chunk stays loaded" (`src/scenes/world.rs:763`) is right: two cameras over 1,024 m apart share no Chunk within 512 m of both.

### Counts

| Severity class | Findings |
|---|---|
| `image` | 0 |
| `budget` | 0 |
| `correctness` | 4 (L1-C04, L1-C05, L1-C07, L1-C08) |
| `drift` | 10 (L1-C01, L1-C02, L1-C03, L1-C06, L1-C09, L1-C10, L1-C11, L1-C12, L1-C13, L1-C14) |
| `health` | 1 (L1-C15) |

Clause verdicts: 261 rows (A 59, B 66, C 95, D 41); `holds` 220, four of
them with a `drift` Finding on the clause's wording or its doc (L1-C01,
L1-C03, L1-C06, L1-C09); `broken` in part 4 (L1-C05 on ADR 0018's dev
launch; L1-C07 on ADR 0017's no-growth clause and its checklist line;
L1-C08 on ADR 0017's move by copy); `broken, known` 2 (ADR 0017's
texture pool capacity in Texture classes, ADR 0022's determinism as the
streaming section's L1-F41); `superseded` 0; falls in another reader or
module 27; not applicable 8. Spec stories: 99 rows (A 19, B 15, C 35, D
30), a story read by more than one reader counted once by each; 70 hold;
broken in part 3 (M4 story 64 with L1-C05, M4 story 47 with L1-C07, M4
story 37 with L1-C08); superseded 3 (M5 story 17 and the M5 spec's Far
proxy Implementation Decision by ADR 0026's 2026-11-18 amendment, M4
story 66 by ADR 0026); falls in another reader or module 22; not settled
from code 1 (M5 story 15, the simplifier's weights tuned on the gallery).
Registry keys: 38 rows (A 17, B 7, C 11, D 3); 34 match, one of them as
the Deviation `src/cook/mesh.rs:meshlet_triangle_counts`; 1 differs
(`unity-6000-uv-distribution-metric`, L1-C10); 2 fall in reader A; 1 an
uncited source unchanged in scope (SplitMix64). Oracles: 39 rows (A 7, B
11, C 16, D 5); `exact` 26; `exact` with a `loose` part 2; `explained` 5;
`tight` 2 (the Far proxy's LOD 0 within a pixel, the bake's outermost read
at (2.3, 2.6)); `loose` 4 (SciFiHelmet's Blob ceiling, L1-C02, by 1.14%
over its measurement; the two camera paths' steps, by about 10^6 and
2.5 × 10^5; the cap vertices' centre line, by about 47); `unexplained` 0;
one states its floor beside its constant. The module's 95 unit tests and
the 16 of `tests/cook.rs` pass on the M4 Pro.

## Decision clauses, ADRs 0001 to 0027

Every decision clause of the 27 ADRs, one row each, in the order of each
ADR's text: its Decision's binding sentences, its amendments, and each
Consequence that says what the code must do; the status lines, counts and
"today", "until" and "yet" clauses are lane 7's. Each section judged the
clauses its module governs, so one clause can carry several sections'
readings, merged here into one verdict; `Read by` names them, and
`report` marks a reading of the report session's own at the frozen
commit, alone where no reader took the clause. Where this table and a
section's row differ, this table carries the verifier's verdict: L1-R03
in place of the killed L1-S05, the killed L1-C06's clause holding, ADR
0019's LOD distance broken for a sheared Instance as the confirmed L1-S04
says, where the shaders row read it as holding for orthogonal columns, and
two rows on which sections disagreed, settled by the Findings the
verifier confirmed (ADR 0018's dev launch, broken in part by L1-C05; ADR
0022's fixed schedule, holding, with L1-F41 breaking its determinism
clause instead).

| ADR | Clause (quoted, shortened) | Verdict | Where | Read by |
|---|---|---|---|---|
| 0001 | "Materials are entries in one structured GPU buffer" | holds: one `materials_buffer` of `MaterialData` | `material.rs:478-484`, `:570-580` | cook |
| 0001 | "indexed by the Material ID that each Meshlet record carries, its Primitive's unless the Instance overrides it" | holds: a 12-byte record (slot, Meshlet, Material ID) read by the vertex stage, the Instance carrying its override or `NO_MATERIAL` (render, Rust side); `MeshletData::material_id` is the Handle's index or `NO_MATERIAL` (cook); the culling pass writes the Instance's ID into the record unless it is `NO_MATERIAL`, else the Meshlet's (report, shader side) | `culling_pass.rs:250-255`; `instance.rs:23-30`; `mesh.rs:58-64`; `src/render/shaders/culling.wgsl:594-600` | render, cook, report |
| 0001 | "the shader reads factors and Layers from it, so one bind group serves every material" | holds: one `MATERIALS` group, refreshed once a frame, bound in the depth and lit passes (render, cook); the lit pass reads `materials[in.material_id]` and its Layers (report) | `renderer.rs:425-431`; `bind_groups.rs:49-66`; `pools.rs:552`; `src/render/shaders/main.wgsl:308-313` | render, cook, report |
| 0002 | "Meshes are (vertex offset, index offset, index count) sub-allocations of one shared vertex buffer and one shared index buffer in the Mesh pool" | holds: one vertex Arena and one index Arena, each one `wgpu::Buffer`; a Mesh keeps its vertex range and its index range, and its BLAS build names both offsets and LOD 0's index count (assets); writer side, a Mesh's vertices are one run every LOD indexes from 0 with u32 indices, so a loader adds one vertex offset (cooker) | `pools.rs:242-255`, `:607-613`, `:691-705`; `src/assets/cooked.rs:591-616` | cook |
| 0002 | "ADR 0017 replaces growth with fixed arenas sized from the Budget, sub-allocated by an offset allocator, each at most 2^31 − 1 bytes" | holds: each Arena's buffer is created once, an `offset_allocator::Allocator` of its elements sub-allocates it, and creation asserts fewer than 2^31 bytes | `pools.rs:1327-1356`, `:1337-1340` | cook |
| 0002 | "each Arena is now sized at device creation from the geometry Arenas' GPU memory Budget ..., in the share ADR 0017 gives it" | holds: `from_budgets` takes each Arena's per-mille of `geometry_arenas`, rounded down to `COPY_BUFFER_ALIGNMENT`; 658, 130, 11, 133 and 68 per mille are ADR 0017's 65.8%, 13.0%, 1.1%, 13.3% and 6.8% | `pools.rs:19`, `:75-93`; `bench/budgets.json:11` | cook |
| 0002 | "Refined by ADR 0019: the Mesh table names each Mesh's Meshlet ranges in three further Arenas ...; the vertex Arena still holds the vertices, and the index Arena feeds only the BLAS builds and refits" | holds: three Meshlet Arenas, a `MeshData` per Mesh with each LOD's Meshlet and triangle ranges; the index Arena is `BLAS_INPUT` only and no pass binds it | `pools.rs:248-273`, `:529-573`, `:596-600`; `mesh.rs:36-49` | cook |
| 0003 | "Transform-propagation results live in one GPU buffer, the Instance buffer, in a slot per Renderable, and are updated incrementally through dirty flags; there are no per-entity uniform buffers" | holds: one `STORAGE` buffer bound in `SCENE_GEOMETRY`; `Changed` filters | `renderer.rs:502-512`, `:541-558`; `prepare.rs:86-93`, `:807-825` | render |
| 0003 | "each Renderable keeps a slot in the Instance buffer until it stops being a Renderable" | holds: a skipped Renderable keeps its slot, written an Instance with no Mesh once; a slot is freed only for an entity that left the Renderable query (the last audit's slot Finding is fixed) | `prepare.rs:737-744`, `:784-798`, `:425-430` | render |
| 0003 | "a freed slot is taken again before a fresh one" | holds for a slot a Renderable freed (`free.pop().or(fresh)`); a Renderable never takes a slot of a Chunk's free range: L1-R01 | `prepare.rs:330-331` | render |
| 0003 | "the buffer holds a fixed number of slots, past which a Renderable is refused, warned about once, counted in Frame stats and retried every frame" | broken where Chunks' free ranges hold slots: a Renderable is refused while the buffer has free slots: L1-R01; warned once, counted and retried hold (render); holds in streaming: a refused load is logged and counted, then retried, the pools' refusal falling in assets | `prepare.rs:326-345`, `:784-787`; `io.rs:1550-1563`; `mod.rs:190-197` | render, streaming |
| 0003 | "the number is sized at device creation from the scene and pass buffers' GPU memory Budget" | holds: created once at `RenderCapacities::instances` slots from the checked-in Budgets, never resized | `renderer.rs:253-261`, `:402`, `:502-512`; `prepare.rs:505-508` | render |
| 0003 | "30% of it at 136.2 bytes a slot, its Instance, offsets and visibility bit, in whole 256-slot scan blocks ...: 517,376 slots at the Budgets M4 set" | broken in its figures, `drift`: the code takes 144.2 bytes a slot and gives 488,448 slots at the same Budgets: L1-R03 (shaders' L1-S05 killed as its duplicate) | `renderer.rs:204`, `:215-218`, `:259-261`; test `:1088-1098` | render, shaders |
| 0003 | "prepare writes only the slots of new Renderables and of those whose World transform, Mesh or Material changed ... through the per-frame staging path, so camera motion writes no Instance" | holds: the camera is no Instance field and a uniform of its own; Instance writes go through `FrameStaging` only (render); on the scene side `wrap_around_camera` dereferences `Transform` mutably only when a flock jumps (scenes) | `prepare.rs:86-93`; `instance.rs:33-38`; `renderer.rs:765-781`; `src/scenes/mod.rs:926-932` | render, cook |
| 0003 | "A freed slot is written an Instance with no Mesh, which the GPU does not draw, and is taken again from the next frame" | holds: `freed` is written after the visit and joins `free` at the end of `prepare` | `prepare.rs:841-844` | render |
| 0003 | "Refined by ADR 0014: the buffer stores an integer anchor plus an f32 offset per Renderable" | holds | `instance.rs:8-12`, `:36`, `:38`; `prepare.rs:157` | render |
| 0003 | "Refined by ADRs 0022 and 0020: prepare also rewrites every Moving Instance each frame, changed or not" | holds; the amendment answers the last audit's drift Finding on it | `prepare.rs:91`, `:805` | render |
| 0003 | "On the frame after the Mesh or Material pool frees an entry, prepare writes again every Renderable's Instance and the Placements of each drawn Chunk one of whose Placements names an entry no longer in its pool ...; free ranges, free slots and held Chunks ... are not written again" | holds: free ranges and slots are rewritten only on `rewrite_every_instance`; a held Chunk is skipped | `prepare.rs:670-672`, `:700-708`, `:836-840`, `:925-937` | render |
| 0003 | "A Placement has no entity: each loaded Chunk owns a contiguous range of slots, one per Placement, taken on the first frame prepare sees the Chunk, before new Renderables take theirs, and kept until it unloads" | holds (render); on the scene side no spawn spawns static content, `spawn_world` and `spawn_town` spawning flocks and the far ground's Chunk only (scenes) | `prepare.rs:351-387`, `:694-733` before `:734-825`; `src/scenes/world.rs:244-279` | render, cook |
| 0003 | "its Placements' Instances are written there in the order the Chunk holds them, on that frame and again whenever every Instance is rewritten, and its point lights join the frame's lights" | holds | `prepare.rs:702`, `:952-970`, `:1020-1024` | render |
| 0003 | "A Placement whose Handles find nothing, or that lacks a Material Handle its Mesh needs, is written an Instance with no Mesh and, unlike a Renderable, not retried" | holds | `prepare.rs:952-977` | render |
| 0003 | "A Chunk the buffer has no room for is refused, warned about once and its Placements counted as refused every frame" | holds | `prepare.rs:371-381`, `:697`, `:713-718`; test `:1310-1354` | render |
| 0003 | "An entity's slot is still per entity" | holds: `InstanceSlots` keeps one slot per entity in `by_entity`, apart from the Chunks' ranges in `by_chunk` | `src/render/prepare.rs:300-303` | report |
| 0003 | "A range is the first free range long enough, else fresh slots past the highest taken" | holds | `prepare.rs:351-387` | render |
| 0003 | "an unloaded Chunk's range is written Instances with no Mesh and joins the free ranges the next frame, merged with those it touches, so a refused Chunk takes a range once one fits" | holds | `prepare.rs:391-402`, `:406-423`, `:686-693`; test `:1361-1420` | render |
| 0003 | "A held Chunk keeps its range, written Instances with no Mesh until it is drawn again, and its point lights stay out of the frame's" | holds | `prepare.rs:700-728`, `:1022` | render |
| 0004 | "The renderer never reads the ECS World mid-frame: a prepare pass builds Frame data (the Instances that changed, lights, camera) and the renderer consumes only that; it lists no draws" | holds: `prepare` takes the World, `upload` and `render` take only Frame data and the pools (State); `renderer.rs` imports `bevy_ecs` only in its test module, `FrameData` has no draw list, and the culling pass reads `frame.instance_slot_count` alone (render) | `src/state.rs:304-312`, `:328-331`; `renderer.rs:1-26`; `prepare.rs:23-50` | streaming, render |
| 0004 | "every Frame data must reach the renderer, in order" | holds: each `advance` that prepares also uploads, and a settling frame whose load fails returns before `prepare`, so no Frame data is built and lost (State); `upload` asserts the previous frame's encoder was taken by `render`, and a lost upload makes the next `prepare` rewrite every Instance (render); capture prepares then renders each drawn and held frame, ascending, and undrawn frames build no Frame data (tooling) | `src/state.rs:279-281`, `:304-312`; `renderer.rs:691-694`; `src/capture/mod.rs:354-394` | streaming, render, tooling |
| 0004 | "Frame data is an owned, `Send` value with no borrow into the ECS World" | holds: compile-time assertion | `prepare.rs:52-55` | render |
| 0005 | "Per-Instance data (model matrix, normal matrix, Material ID) sits in a structured buffer ...; there is no bind group per mesh or per instance" | holds: 128 bytes, `STORAGE` | `instance.rs:31-40`; `renderer.rs:503-511`, `:541-558` | render |
| 0005 | "Refined by ADR 0014: the model matrix is relative to an integer anchor, which the Instance also carries" | holds: `InstanceData` carries `rotation_scale` beside its pivot's `anchor` and `offset`, and the vertex stage composes each rotated vertex from them camera-relative | `src/render/instance.rs:33-40`; `src/render/shaders/main.wgsl:227-233` | report |
| 0005 | "the lookup key is the Instance slot in the Meshlet record a packed index names, not `@builtin(instance_index)`" | holds: a 12-byte record (slot, Meshlet, Material ID), its capacity asserted below the bits above a packed index's six vertex bits (render, Rust side); the vertex stage unpacks the index and reads `instances[record.instance_slot]`, and no WGSL reads `instance_index` (report) | `culling_pass.rs:219-221`, `:250-255`; `bind_groups.rs:285-296`; `src/render/shaders/main.wgsl:222-224` | render, report |
| 0006 | "Components hold data only: no closures, no `Box<dyn Trait>`, no behaviour methods" | holds: no component in these files holds a closure or trait object; the boxed cook closure is a `ChunkStreaming` field, not a component (State); `Spin`, `WrapsAroundCamera` and `Bone` are data and `MeshRef` holds a Handle, no GPU resource (scenes) | `src/components.rs:9-97`; `src/transform/mod.rs:8-9`; `src/state.rs:229`; `src/scenes/mod.rs:895-917` | streaming, cook |
| 0006 | "Behaviour lives in systems or, later, script handlers" | holds: the wrap, spin, sway and propagation are systems | `src/state.rs:403-418` | streaming |
| 0006 | "A method that is a pure function of the component's own data and its arguments ... is allowed: `Transform::to_affine`, `Bend::drawn_curvature` and `WorldTransform::{translation, drawn}`" | holds: `drawn`, rewritten in scope to slerp the rotation, still reads only `self`, `previous` and the weight; the last audit's (M3.5) L1-R03 is closed by this amendment (State); `Sway::curvature` reads only `self.phase` and its argument, the amendment's kind (scenes) | `src/transform/mod.rs:26-49`; `src/components.rs:34-40`, `:74-76`; `src/scenes/mod.rs:1307-1326` | streaming, cook |
| 0007 | "The engine stays on wgpu (Vulkan, DX12, Metal) for its lifetime, using `wgpu-hal` only for a subsystem that exceeds wgpu's safe API" | holds: the refit in `blas_refit.rs` and the acceleration-structure size query are the module's only wgpu-hal use | `blas_refit.rs:190-270`; `acceleration_structures.rs:115-173`; `src/gpu/mod.rs:494-498` | render |
| 0007 | "migration to raw Vulkan through `ash` is not planned and must not influence design"; "Raw Vulkan through `ash` stays unplanned" | holds: `Cargo.toml` names no `ash` dependency and nothing in `src/` calls `ash::` | `Cargo.toml:7-28` | report |
| 0007 | reopen: "The newest wgpu release cannot meet the Capability floor (ADR 0021) on Metal, Vulkan or DX12" | not judged from code: needs the newest published release, which is the exit's dependency review | `Cargo.toml:8` | render |
| 0007 | reopen: "`vendor/` carries more than five upstream commits, which `vendor/README.md` lists, or a fix upstream has declined" | holds: one commit, gfx-rs/wgpu `59209f27`, merged upstream; the vendored `ray_tracing.rs` is the registry's 30.0.1 plus exactly that commit | `vendor/README.md:13-35`; `Cargo.toml` `[patch.crates-io]` | render |
| 0007 | reopen: "the wgpu that `Cargo.toml` pins is more than two major releases behind the newest published" | not judged from code: needs the newest published release, which is the exit's dependency review | `Cargo.toml:8` | render |
| 0007 | exit: "an owned fork of wgpu's crates, built from `vendor/`, where on 2026-11-14 a copy of wgpu-core 30.0.1 carries one upstream fix" | holds as dated; the copy has been 30.0.2 with the same fix since 2026-11-15 (render's Notes for the verifier) | `vendor/README.md:13` | render |
| 0008 | "Anti-aliasing is spatial (MSAA, SMAA); TAA, upscaling stacks and frame generation are not mandatory" | holds: coverage below a pixel is four samples' worth, rounded; no jitter, no accumulation | `common.wgsl:231-234` | shaders |
| 0008 | "native resolution is the default" | holds: the render targets are made at the surface's size at creation and on every resize, with no render scale | `src/lib.rs:198-205`; `src/state.rs:254-255`; `src/render/renderer.rs:412`, `:630-633` | report |
| 0008 | "Nothing the image depends on reads screen-space history (Temporal class (c)): lighting terms are class (a) or (b) (ADR 0020) and the main pass reads none (ADR 0019)" | holds: `vs_thin`, `thin_fragment_samples` and `lit` read this frame's uniforms, buffers, Textures and shadow mask only (surface shading); the culling pass's LOD, small-Instance coverage and thin drop read no earlier frame, the visibility bit choosing only the phase (culling); ADR 0018's Toksvig widening reads the normal Texture's texels alone, at cook time (cooker); for the checklist's main-pass line, `MAIN_PASS_INPUTS` binds this frame's records, mask and mask samples (render) | `main.wgsl:113-215`; `culling.wgsl:284-344`; `src/cook/texture.rs:565-590`; `bind_groups.rs:285-317` | shaders, cook, render |
| 0008 | "Lighting is ray-traced probe GI with hard ray-traced shadows (amended 2026-10-13, ADR 0020)" | holds for the shadow mask: it reads no earlier frame's screen data and accumulates nothing; the one word read per hit is the current frame's for every slot in use (shaders' Notes: a static TLAS entry past the slots in use) | `shadow_mask.wgsl:192`; `culling.wgsl:428-431`, `:446-456` | shaders |
| 0008 | "the main pass is clustered forward with 4x MSAA (amended 2026-10-12, ADR 0019)" | holds: the Froxel pass lists each Froxel's point lights for the lit pass, and the depth target and multisampled colour take `SAMPLE_COUNT` 4 | `src/render/froxel_pass.rs:1-4`; `src/render/targets.rs:16`, `:194-205` | report |
| 0009 | "The ECS is `bevy_ecs` used on its own, not the Bevy engine" | holds: `bevy_ecs` and `bevy_tasks` 0.20.1 are the only `bevy_*` dependencies (`bevy_mikktspace` is the cooker's tangent crate); the lockfile's other `bevy_*` crates are `bevy_ecs`'s own | `Cargo.toml:17-18`; `src/threading.rs:5-7` | streaming |
| 0009 | "the engine keeps its own `Transform` ... instead of `bevy_transform`; ADR 0014 replaces that newtype with an f64 translation and f32 rotation and scale" | holds: `Transform` is the engine's, `WorldTransform` its derived f64 form; only `ChildOf`/`Children` come from `bevy_ecs` | `src/components.rs:9-15`; `src/transform/mod.rs:8-9`, `:113-159` | streaming |
| 0010 | "Praxis targets single-player games on Windows, Linux and macOS ... nothing in the architecture carries replication, prediction or sandbox constraints" | holds: the frame has one `Input`, one camera, no replication; no `wasm32` cfg anywhere in `src/` | `src/lib.rs:97-127`; `src/state.rs:91-95` | streaming |
| 0011 | "glTF (from Blender) is the authoring source; the runtime loads Blobs with mip chains, GPU block compression (BC) and byte ranges sized for streaming" | holds: the cooker alone parses glTF (`gltf` behind the `cook` feature) and writes a full mip chain of BC blocks per Texture with a KTX2 level index giving each mip its byte range (cooker); a Far proxy's atlas is BC7 and BC5 with every mip (world's cook); a Texture Blob must hold its full chain, in a BC format, with a level index entry per mip (reader side); every scene generator compiles only with `cook`, the runtime spawns load by ID through the `AssetLoader`, and `world`'s and `town`'s content is generated from code (scenes) | `src/cook/texture.rs:162-218`; `cooked.rs:433-457`; `src/scenes/mod.rs:159-161`; `Cargo.toml:41` | cook |
| 0011 | "since then a development launch reaches glTF only through the cooker, which cooks what changed before the runtime loads Blobs" | holds per Blob: each Blob a launch loads through `CookingLoader`, or requests through the IO pool, is cooked first where stale (L1-C05 aside); the wording's drift, filed as L1-C06, was killed, as no sentence of the ADR is false; reader side, nothing in `src/assets/` reads glTF (assets) | `launch.rs:244-283`; `src/streaming/io.rs:814-819`, `:1524-1529`; `loader.rs:71-87` | cook |
| 0011 | "Textures cook into KTX2, a standard container" | holds: `ktx2` 0.5.0's header, level index and DFD serialisers, one key-value entry (cooker); parsed by `ktx2::Reader` (reader side) | `src/assets/cooked.rs:312-372`; `cooked.rs:416-418` | cook |
| 0012 | "Gameplay logic ... lives in Lua through mlua"; "Lua ... never authors per-frame systems" | not applicable: nothing implements it at this commit (no `mlua` in `Cargo.toml`); nothing in these files contradicts it, the per-frame schedule holds `fly_camera` alone | `src/state.rs:395` | streaming |
| 0013 | "Budgets are judged on an Apple M4 Pro, the reference machine; the RTX 2070 SUPER is the hardware floor, benched for comparison on Vulkan and on DX12" | holds (bench half): Baselines are keyed by adapter, and by backend unless Vulkan or Metal, so the RTX's DX12 runs have a directory of their own; CI's lavapipe judges counters and memory only; `compare` still applies the Budgets to any adapter's Local run (the last audit's note stands) | `src/bench/mod.rs:146-164`, test `:3118`; `bench/baselines/nvidia-geforce-rtx-2070-super-dx12/`; `.github/workflows/ci.yml:286-296` | tooling |
| 0013 | "The reference machine ... gates regressions, the Budgets being judged on it" | holds in the history's half: Suspects are marked on every non-lavapipe Series, but only the M4 Pro runs a sweep, so only its Regressions are filed; the history judges no Budget | `src/bench/history.rs:391-394`; `scripts/sweep.sh:16-18` | tooling |
| 0013 | "a technique rejected for its cost, or a reopen trigger's verdict, is final only with the figure on both adapters ... from one engine source, their `provenance.source_hash` equal"; "(the Bench records of source hash `6a91ca76`)" | holds: every result carries `source_hash` in its Provenance, and a record holds the run's whole result keyed by source hash; `records/<adapter>/town/6a91ca76….json` exists on `bench-history` for the M4 Pro, the RTX 2070 SUPER on Vulkan and on DX12, and lavapipe; the judging is process | `src/bench/provenance.rs:238-248`, `:276-281`; `src/bench/history.rs:755-794` | tooling |
| 0013 | "When the owner owns target-class hardware, the Baselines are re-recorded on it" | not applicable yet | — | tooling |
| 0013 | "60 fps at 1440p native with CPU at most 6 ms and GPU at most 12 ms per frame" | holds: `REFERENCE` is 2560x1440, frames step at `SIXTY_HERTZ`, Budgets 6000 and 12000 | `src/bench/mod.rs:61-67`, `:893-894`; `bench/budgets.json:2-3` | tooling |
| 0013 | "the 6 ms bounds the wall time of each pipeline stage's critical path, waits on worker threads included" | holds at the seam: a stage's time is the wall-clock span on the main thread from its start, so a wait on the compute pool inside it counts | `src/stats/mod.rs:300-310`; `src/state.rs:278-300` | tooling |
| 0013 | "CPU time summed across threads is neither measured nor budgeted yet, Frame stats holding wall-clock times per system, stage and frame only" | holds: `cpu_times` are system spans and named stages on the frame's thread, `cpu_frame_time` the frame's wall time, `static_tlas_build_time` the worker's wall time on one build (a field ADR 0020 asks for); the bench keeps each system's and stage's time per frame and sums nothing across threads | `src/stats/mod.rs:16-20`, `:47-50`, `:303-318`; `src/bench/mod.rs:1022-1035` | tooling |
| 0013 | "the CPU frame, the GPU frame and ADR 0019's gate answer to their Budgets at the p99 as at the median, the CPU frame at its p95 too" | holds (M3.5's L1-T01 fixed): median, p95 and p99 of the CPU frame, median and p99 of the GPU frame and the gate; the report's note on it is L1-T06 | `src/bench/mod.rs:1270-1273`, `:1301-1311`, `:1331-1334`; test `:2036` | tooling |
| 0013 | "no CPU or GPU frame lasts longer than 16,667 µs, one 60 Hz period" | holds over the measured frames: `maximum_frame_microseconds` 16667 against both maximums; warm-up frames are not judged | `bench/budgets.json:8`; `src/bench/mod.rs:1274-1281`, `:1312-1318`; test `:2072` | tooling |
| 0013 | "Input latency at most 2 frame periods (33.3 ms at 60 fps) for input that drives the camera and 2 frame periods plus one Tick (49.0 ms) for input the simulation consumes, judged at p95" | not implemented: no metric and no `budgets.json` key; the glossary's Budget entry says one "is planned, once it is measured" | `bench/budgets.json:1-19`; `CONTEXT.md:303` | tooling |
| 0013 | "100k visible Instances, about 5M visible triangles" | holds in capacity: the Instance buffer holds 488,448 slots at the checked-in Budgets, as a test pins, and the frame's draw 2^25 packed indices, 11.2 million triangles; whether the Budgets hold at that scale is the bench's | `src/render/renderer.rs:253-277`, test `:1086-1097`; `src/render/culling_pass.rs:92-101` | report |
| 0013 | "a 10 by 10 km streamed Game world" | holds: 10.24 km a side | `src/scenes/world.rs:1-2`, `:26-27` | cook |
| 0013 | "a 2 km LOD'd view distance (drawn past the Load radius as a Far proxy per Chunk, ADR 0026)" | holds: `world`'s View distance is 2,000 m and its Load radius 256 m, every other scene's View distance its Load radius (scenes); the drawing past the Load radius falls in render and streaming (tooling), with no row there under this ADR: a Far proxy is held exactly while its Chunk is drawn (ADR 0014's Far proxy row, `report`) | `src/scenes/mod.rs:553-580`; `src/scenes/world.rs:26-27`; `src/streaming/io.rs:1005-1020` | cook, tooling, report |
| 0013 | "GPU memory at most 8 GB at 1440p, counted as the bytes of the buffers and textures the engine has created, summed by the engine itself on every backend" | holds: each buffer, texture and TLAS created through `GpuMemory` adds its bytes to its kind's running total until dropped, `gpu_memory_bytes` sums the seven totals, and `compare` judges it against 8,000,000,000 at the measured frames' median in every mode; the load staging's belt, which bypasses the ledger, is L1-R04, and `gpu_memory_bytes`'s doc omits it the same way (tooling); every buffer and texture the Renderer and its passes create goes through the ledger under its kind (render); the streaming module creates no GPU object outside its tests (streaming); a `TRANSIENT_ATTACHMENT` counts as zero only when `transient_saves_memory` is `Some(true)`, the pbr-sweep test pinning the 4x colour at 0 on Metal and full elsewhere (render, tooling) | `src/stats/gpu_memory.rs:109-164`; `src/bench/mod.rs:1227-1232`; `renderer.rs:440-469`; `tests/frame_stats.rs:134-137` | tooling, render, streaming |
| 0013 | "the GPU stats readback's buffers are left out of that sum" | holds: `GpuStatsReadback::new` takes no ledger (render); documented at the seam (tooling) | `readback.rs:193-224`; `src/stats/mod.rs:65-67` | render, tooling |
| 0013 | "so are the target the frame is presented into, a window's surface textures or capture's stand-in for them, and capture's read-back buffer" | holds: capture creates its target and every read-back buffer on the device directly, outside the ledger, and the field's doc says so; M3.5's L1-T07 is closed by this amendment | `src/capture/mod.rs:332-345`, `:465-470`; `src/stats/mod.rs:64-65` | tooling |
| 0013 | "because wgpu has no allocator report on Metal"; "wgpu's allocator report and Metal's `currentAllocatedSize` are diagnostics only" | holds: an `Option`, printed only, never budgeted, and in no `Measurements` field; `currentAllocatedSize` is read nowhere | `src/stats/mod.rs:76-78`, `:249-259`; `src/bench/mod.rs:189-274` | tooling |
| 0013 | "the 8 GB splits into Budgets for geometry arenas, the texture pool, Resident tails, render targets, acceleration structures and staging ...; a seventh Budget covers the scene and pass buffers" | holds: seven `GpuMemoryKind`s, each named as `bench/budgets.json` keys it, `GpuMemoryBudgets` denying unknown fields; Frame stats carry and print each kind; `compare` judges each at its measured frames' median in every mode | `src/stats/gpu_memory.rs:9-66`; `bench/budgets.json:10-18`; `src/bench/mod.rs:1036-1047`, `:1233-1238` | tooling |
| 0013 | "acceleration structures join the count (amended 2026-10-12, ADR 0017)" | holds: the `AccelerationStructures` kind, `create_tlas`, and `charge` for BLASes | `src/stats/gpu_memory.rs:17-20`, `:188-220` | tooling |
| 0013 | "M4 set the seven above `town`'s and the world scene's peaks on the M4 Pro: 83,886,080 bytes for geometry arenas, 4,294,967,296 ..., 234,881,024 for the scene and pass buffers, 5,955,911,680 in all" | holds: the seven values equal the ADR's and, recomputed, sum to 5,955,911,680, under the 8,000,000,000 the test checks | `bench/budgets.json:11-17`; test `src/bench/mod.rs:2182` | tooling |
| 0013 | "each capacity is its share of its kind's Budget at its real bytes per entry, as ADR 0017 gives them" | holds: `PoolCapacities::from_budgets` reads `bench/budgets.json`'s figures, which are ADR 0013's (assets); `RenderCapacities::from_budgets` takes each of its capacities as a per-mille share of its kind's Budget at `size_of` its entries, the Instance slots in whole scan blocks (report) | `pools.rs:75-107`; `bench/budgets.json:10-18`; `src/render/renderer.rs:215-218`, `:253-277` | cook, report |
| 0013 | "At M3's exit the Budgets were re-decided ...: each stays as written and is no longer provisional, and so do the dynamic TLAS's ceilings of 1.1 ms of CPU and 1.0 ms of GPU" | holds: 6000, 12000, 6000, 1100 and 1000 unchanged since `8bc34eb1` (the diff adds keys only); the TLAS ceilings judged at the median | `bench/budgets.json:2-6`; `src/bench/mod.rs:1335-1348`; test `:2324` | tooling |
| 0013 | "what `town` breaches is ticketed as an offender, not budgeted away" | holds: no Budget rose | `bench/budgets.json` (diff since `8bc34eb1`) | tooling |
| 0013 | "A budget moves down freely and up only through an ADR or a map comment" | holds: the three keys added (guard, maximum, kinds) each came with an ADR 0013 amendment | `bench/budgets.json:7-18` | tooling |
| 0013 | the guard: "the median time of both culling phases' per-slot stages (`offset_slots`, `offset_blocks` and `append_records`, timed apart from `append_triangles`) stays at or under 1,000 µs on the M4 Pro, on every Reference scene. It is `culling_per_slot_microseconds`" | holds: each frame sums the two phases' `_slots` passes, which run the three stages and nothing else (pinned by Frame stats' pass list test), failed only when the median is over 1000 (tooling); each phase is two Timed passes, slots then triangles (render) | `src/bench/mod.rs:1010-1020`, `:1349-1361`; `src/render/culling_pass.rs:47-52`; `tests/frame_stats.rs:283-300` | tooling, render |
| 0013 | "`praxis compare` fails a scene past it, on the adapter its Baseline is from, as it does one past a Budget" | holds: Local mode only, after the adapter and backend check; `--ci` passes it, so neither verify nor CI judges it | `src/bench/mod.rs:1239-1261`, `:1349-1361`; test `:2315-2318` | tooling |
| 0013 | "the Bench record's counters hold the Instance slots in use" | holds: `instance_slots_in_use` per frame in Frame stats, the median of its frames in the record, judged as any counter, and in `Counters::NAMES`, which the Series row's `measured_counters` reads | `src/stats/mod.rs:32-34`; `src/bench/mod.rs:334-336`, `:411-436`; `src/bench/history.rs:1222-1228` | tooling |
| 0013 | checklist: "per-frame GPU writes add to `uploaded_bytes`, and systems are added inside `SystemTimer::scope`" | holds: the Chunk streamer's staged bytes are added to what `upload` adds, and `LoadStaging::submit` returns moved bytes only, so nothing counts twice; both schedules' systems are added inside `recorder.systems.scope` | `io.rs:1163`; `src/state.rs:287`, `:394-419`; `src/render/staging.rs:356-365` | streaming |
| 0014 | "World positions are f64, right-handed, +Y up ..., in metres, with cameras looking down −Z: `Transform` holds translation as `DVec3` (relative to the parent for a child) with rotation (`Quat`) and scale (`Vec3`) in f32, 64 bytes" | holds, asserted at compile time (State); Placements, lights, proxy pivots and texture-use spheres are f64 (streaming); `export-scene` takes Renderables' and Placements' translations from the f64 `DAffine3` and writes them, lights' and the camera's positions as f64 (cooker); a Far proxy's vertices are composed in f64 before narrowing, Chunk bounds in f64 (world's cook); the scenes' settlement centres, lots, cameras, flocks, Placements and lights are `DVec3` (scenes) | `src/components.rs:9-18`; `chunks.rs:33-34`; `src/cook/export.rs:155-156`; `src/scenes/world.rs:171-183` | streaming, cook |
| 0014 | "The GPU never holds a World position in f32: it reads an integer anchor on a 64 m grid plus an f32 offset, and shaders compose positions relative to the camera" | holds: `anchor_and_offset` splits in f64 and rounds the offset once for the camera, every Instance and Placement pivot and every point light, `view_proj` is projection times the rotation-only view, and both TLASes compose each entry's translation from Anchors in f64 (render); `InstanceData` and `CameraUniform` carry a `vec3<i32>` anchor and an f32 offset and `camera_relative_position` subtracts the anchors in i32 (culling); `vs_thin` composes the bounds' centre by `camera_relative_position` (surface shading); the shadow mask composes the TLAS-frame origin from the Anchors' difference in `vec3<i32>` (shadow mask); as tested, every Golden scene, `world` included, must match its Golden shifted 100,032 m, 1,563 whole 64 m anchor cells (tooling) | `instance.rs:1-12`; `common.wgsl:37-46`; `acceleration_structures.rs:701-731`; `tests/goldens.rs:58-64` | render, shaders, tooling |
| 0014 | "Implemented: `Transform` and the camera hold f64 positions"; "shaders compose positions relative to the camera" (the camera's side) | holds on the camera side: `position` is a `DVec3`, `view_matrix` is the rotation alone at the origin | `src/camera/mod.rs:20-21`, `:84-86` | streaming |
| 0014 | "point lights reach the GPU as an anchor plus an offset, which the Froxel pass and the lit pass compose camera-relative" | holds: `point_light_position` composes the light's anchor and offset through `camera_relative_position` with no local position (culling); `lit` calls it (surface shading), and so does the shadow mask (shadow mask); the Froxel pass calls it for each light's centre (report) | `common.wgsl:127-139`; `main.wgsl:443`; `shadow_mask.wgsl:319`; `src/render/shaders/froxel.wgsl:38` | shaders, report |
| 0014 | "A child's Transform composes with its ancestors' into a World transform in f64, recomputed only when it or an ancestor moved" | holds; unchanged in scope, tested | `src/transform/mod.rs:113-159`; tests `:220-237`, `:240-257` | streaming |
| 0014 | "Each Instance and the camera reach the GPU as an anchor on the 64 m grid plus an f32 offset, which the vertex shader composes camera-relative" | holds: the vertex stage composes each vertex by `camera_relative_position` from its Instance's and the camera's anchor and offset | `src/render/shaders/main.wgsl:222-233`; `src/render/shaders/common.wgsl:37-46` | report |
| 0014 | "a shader-function oracle holds the composition to the precision bar in every evaluation order within 100 km, and shows f32 World positions failing it" | holds: the GPU's order and every order on the CPU over 8³ camera positions to ±100 km, 26 directions and 3 local positions, with an assertion that f32 World positions miss the bar; the floor the test states leaves out the rounding of its inputs: L1-S06 | `src/render/shader_function_tests.rs:1897-1965`, `:2254-2302` | shaders |
| 0014 | precision bar: "a point 0.3 m from the camera lands within 0.25 px at 1440p and a 45° vertical field of view (0.043 mm) of its exact camera-relative position" | holds: 2 × 0.3 × tan 22.5° / 1440 × 0.25 = 4.31e-5 m, and `PRECISION_BAR` is 4.3e-5 (culling); an unwidened tube's vertex, recomposed by the twins, returns within two roundings of \|p\| (surface shading); a scene moved by a shift draws alike, roots, the camera and `FarGround` shifting, tested at 10,240 m and at the world's half size (State) | `src/render/shader_function_tests.rs:1872`; `main.wgsl:127-133`; `src/state.rs:121-130`; tests `:666-752` | shaders, streaming |
| 0014 | precision bar: "simulated positions resolve to 0.1 mm" | holds for the blend: the rewritten `drawn` keeps translation linear in f64; the orbit test checks 10 µm at 100 km | `src/transform/mod.rs:31-32`; `src/state.rs:461`, `:541-554` | streaming |
| 0014 | precision bar: "saves round-trip positions bit-exactly" | holds: nothing saves at this commit; `src/` has no save code | — | report |
| 0014 | "The Instance buffer (ADR 0003) and the per-Instance data (ADR 0005) store anchor plus offset, so camera motion dirties no Instance" | holds | `instance.rs:8-12`, `:36`, `:38`; `prepare.rs:157` | render |
| 0014 | "the culling pass composes each Instance's bounds centre the same way for its tests (amended 2026-11-04 ...)" | holds: `bounds_centre`; M3.5's L1-S05 no longer applies | `culling.wgsl:133-142` | shaders |
| 0014 | "Worst cases ... sum to 0.041 mm, 0.0185 and 0.0225: the composition in every evaluation order ..., plus f32 rotation of a vertex 128 m from its pivot. A 128 m grid sums to 0.045 mm and fails." | holds where the culling pass uses it: `COMPOSITION_ERROR` takes 0.041 mm for the box's centre and for the vertices, 0.082 mm rounded up to 1e-4 m, unchanged since M3.5; a Far proxy's vertices lie past 128 m (shaders' Notes) | `culling.wgsl:202-208`, `:222-227` | shaders |
| 0014 | "Subsystems that compute in f32 (physics, audio, particles, ray-tracing acceleration structures) convert at their boundary into a local frame they own" | holds: `AnchoredTlas` (render); one composition per ray into the TLAS's frame, the dynamic box arriving in it (shadow mask) | `acceleration_structures.rs:243-252`, `:1058-1066`; `shadow_mask.wgsl:161-167`, `:111-113` | render, shaders |
| 0014 | "The static TLAS's frame is the corner of the camera's Anchor when its first static Instance was written" | holds: both TLASes follow the camera's Anchor, with no build, until an entry is first filled | `acceleration_structures.rs:394`, `:426-435`; test `:1290` | render |
| 0014 | "a build moves it only once the camera's Anchor strays more than a Chunk's edge (4 cells) along an axis" | holds: `REANCHOR_CELLS` is 256 m / 64 m = 4; a strayed camera makes `start` record a build at the camera's Anchor, through ADR 0020's build with no static Instance changed; exactly 4 cells starts none | `acceleration_structures.rs:195`, `:572-581`, `:596-604`; test `:1382-1415` | render |
| 0014 | "so, once that build is bound, the camera is again within 320 m of the frame's origin along each axis and a shadow ray composes and lifts its origin 10 km from where it started as at the origin" | holds: 5 cells of 64 m times `SHADOW_RAY_OFFSET_SCALE` 2^-16 is 4.88 mm, and `upload` fills the dynamic TLAS and the camera uniform's `tlas_anchor` with the bound static TLAS's Anchor in the frame it binds (render, Rust side); the lift is 2^-16 of the larger of the point's camera-relative and TLAS-frame coordinates, both near the camera wherever it is in the World (shadow mask) | `acceleration_structures.rs:190-195`; `renderer.rs:711-742`; `shadow_mask.wgsl:161-172`; `shaders/shadow_mask.wgsl:36` | render, shaders |
| 0014 | "Lua receives positions as numbers, which are doubles. Saves store f64." | holds: nothing hands Lua a position or saves one at this commit (no `mlua` in `Cargo.toml`, no save code in `src/`) | `Cargo.toml:7-28` | report |
| 0014 | "A Chunk is a vertical column addressed by integer (x, z), with its origin at (x·edge, 0, z·edge). The cooked format picks the edge and encodes positions inside a Chunk so they meet the bar once promoted to f64" | holds: `chunk_origin([x, z])` is (x, 0, z) × 256 m, an edge of 4 × 4 of the 64 m cells, and a Chunk Blob stores each Placement's translation as f64 | `src/assets/cooked.rs:57-64`, `:921`, `:989` | report |
| 0014 | "cooked meshes stay within 128 m of their pivot, which the cooker enforces by refusing a mesh with any vertex farther out" | holds: every source vertex's length is checked against `MAX_PIVOT_DISTANCE_METRES`, the seam cooking 128.0 m and refusing 128.1 m (cooker); `world`'s farthest vertex from a pivot is 29.7 m, ground tiles reach 90.5 m in both scenes, `far_coplanar`'s 88.4 m, and the far ground passes the bound as ADR 0026 records (scenes) | `src/cook/mesh.rs:94`, `:166-174`; `tests/cook.rs:427-442`; `src/scenes/world.rs:435-446` | cook |
| 0014 | "A Far proxy's Mesh is exempt: it holds a whole Chunk about its centre, within a Chunk's edge of it, which the cooker enforces instead" | holds: see ADR 0026's pivot row (world's cook) | `far_proxy.rs:220`, `:320-328` | cook |
| 0014 | "it is cooked to be placed unrotated, and is drawn from the Load radius outwards unless its Chunk is late (ADR 0026)" | holds: the cook's side as ADR 0026's pivot row (world's cook); `far_proxy_transform` places it unrotated at its Chunk's centre, and a Resident Far proxy is held exactly while its Chunk is drawn, so it draws past the Load radius and in place of a late Chunk (report) | `far_proxy.rs:220`; `src/streaming/chunks.rs:118-128`; `src/streaming/io.rs:1005-1020`, `:1209-1215` | cook, report |
| 0014 | "World-anchored shader effects (noise, wind, triplanar mapping) derive their periodic coordinates from the anchor" | holds: no shader at this commit derives a periodic coordinate from a World position; `bend.wgsl` bends by per-Instance angles, and the tonemapping dither reads its mask by pixel | `src/render/shaders/bend.wgsl:56-62`; `src/render/shaders/tonemapping.wgsl:22` | report |
| 0014 | "One Game world is loaded at a time, so a World position carries no Game world id" | holds: `Transform` and `WorldTransform` hold the position alone, with no world id | `src/components.rs:11-15`; `src/transform/mod.rs:9` | report |
| 0015 | "Depth is cleared to 0" | holds for all six main-pass pipelines, the twins included, phase two's depth pass and the lit pass loading (render); depth 0 is the background at every use in the shadow mask (shadow mask); the conventions test passes a depth clear only as a zero literal (tooling) | `targets.rs:328-337`, `:341-350`; `shadow_mask.wgsl:272`; `tests/conventions.rs:410-446` | render, shaders, tooling |
| 0015 | "compared with `Greater`" | holds for all six main-pass pipelines, the lit pipelines comparing `Equal` and writing nothing (render); the nearest sample is the greatest depth, the farthest second surface the least, the sentinel 2 past the near plane's 1 (shadow mask); the conventions test passes no `Less` comparison (tooling) | `targets.rs:405-423`; `main_pass.rs:127`; `shadow_mask.wgsl:425`; `tests/conventions.rs:410-446` | render, shaders, tooling |
| 0015 | "in `Depth32Float`" | holds | `targets.rs:12`, `:194-199` | render |
| 0015 | "with no far plane" | holds: the in-scope `perspective_infinite_reverse` builds column 2 `(0, 0, 0, −1)` and column 3 `(0, 0, near, 0)`, so depth is `near / distance`, 1 at the near plane, towards 0 at infinity, tested (State); a vertex at or behind the camera's plane stays and keeps all its samples, and the small-Instance scaling keeps w (surface shading); `in_view_frustum` tests the four side planes and the near plane and no far plane (culling); the view depth is the near distance over the depth, finite for every depth let through (shadow mask) | `src/camera/mod.rs:91-105`; `main.wgsl:154-160`; `culling.wgsl:183-200`; `common.wgsl:58` | streaming, shaders |
| 0015 | "shadow comparisons follow the same direction" | holds: the ray code compares no depth; the back-face pass compares a height above the surface with an error and a distance along the ray with the light's | `shadow_mask.wgsl:190-196` | shaders |
| 0015 | "a Hi-Z pyramid keeps the minimum depth, which is the farthest" | holds: one dispatch per level, the GPU test against a CPU minimum of every sample (render, Rust side); `occluded` takes the minimum of four texels as the farthest depth and calls a box occluded only where its nearest depth is under it, equality not occluded (culling) | `depth_pyramid_pass.rs:97-122`; test `:141-153`; `culling.wgsl:228-266` | render, shaders |
| 0015 | "`projection_matrix` in `src/camera/` is the one projection the window, capture and bench paths use" | holds: the in-scope switch to `camera::rh::proj::directx::perspective_infinite_reverse` builds the same matrix, tested (State); prepare, through which the window, capture and bench all draw, is its one caller outside tests (report) | `src/camera/mod.rs:91-105`; test `:164-184`; `src/render/prepare.rs:606` | streaming, report |
| 0015 | "the `far-coplanar` Reference scene's Golden pins it: red, the upper plane covering the green lower one everywhere" | holds: `FarCoplanar` is in the Goldens' scenes with a Golden per backend, and the Metal Golden shows the red plane with no green | `tests/goldens.rs:44`; `tests/goldens/metal/far-coplanar.png`; `tests/goldens/vulkan/far-coplanar.png` | report |
| 0016 | "The main thread runs winit, input, the simulation schedule, prepare and command recording, then submits and presents" | holds: one `redraw` measures, acquires, runs `update` (Ticks, frame schedule, Chunk streaming, prepare, upload), `render`, then `present`, all on the event-loop thread | `src/lib.rs:212-265`; `src/state.rs:274-319`, `:328-334` | streaming |
| 0016 | "no stage runs a frame behind another" | holds: `render` draws the Frame data the same frame's `advance` prepared, and in a non-settling frame `ChunkStreaming::update` returns `Ok` before any error path, so `advance` never skips prepare (State); capture runs `update` then `render` for each drawn or held frame (tooling) | `src/state.rs:304-312`, `:328-331`; `src/streaming/io.rs:1038-1039`; `src/capture/mod.rs:383-388` | streaming, tooling |
| 0016 | "the `multi_threaded` feature is on" | holds | `Cargo.toml:17-18` | streaming |
| 0016 | "the compute pool is created with `available_parallelism() − 1` workers (at least 1), one pool per State's `FramePreparer`"; "a `TaskPool` that each State's `FramePreparer` owns, not the process-global `ComputeTaskPool`" | holds: `compute_pool()` builds a fresh `TaskPool` of that many workers, tested in place of the old `tests/compute_pool.rs`, and `FramePreparer::new` stores it; no `ComputeTaskPool` or `par_iter` left in `src/` | `src/threading.rs:11-19`; test `:34-38`; `src/render/prepare.rs:489`, `:520` | streaming, render |
| 0016 | "the schedule stays on the `SingleThreadedExecutor`" | holds for both schedules: the frame schedule and the fixed one are built by `single_threaded_schedule`; tested by thread identity, which tells the executors apart | `src/threading.rs:24-28`; `src/time/mod.rs:69-70`; `src/state.rs:392-393`; test `:440-458` | streaming |
| 0016 | "prepare fills the Renderables' Instances on the pool once there are enough to split among its threads" | holds: two or more tasks of 1,024 Instances, batches of 128 taken dynamically, the main thread joining | `prepare.rs:195-264` | render |
| 0016 | "Frame data (ADR 0004) is an owned, `Send` value with no borrow into the ECS World" | holds: compile-time assertion | `prepare.rs:52-55` | render |
| 0016 | "No QoS class is set" | holds: the builder sets a thread count and a name only | `src/threading.rs:15-18` | streaming |
| 0016 | "`par_iter` runs on the process-global pool, which the engine no longer creates" | holds: no `ComputeTaskPool::get_or_init` or `par_iter` in `src/` (the old `init_compute_pool` is gone in scope) | `src/threading.rs:1-28`; `src/lib.rs:47-48` | streaming |
| 0016 | "Asset reads and decoding run on a separate IO pool outside the compute pool" | holds: std threads, not the compute pool | `io.rs:306-316` | streaming |
| 0016 | "They decode into CPU memory, and the main thread copies it into the `StagingBelt`" | holds | `io.rs:437-541`, `:1475-1506` | streaming |
| 0016 | "The IO pool is four std threads that take the Chunk streamer's requests and mip streaming's" | holds | `io.rs:38`, `:154-157` | streaming |
| 0016 | "each reads a Chunk and the Meshes, Materials and Textures it names that no other load claimed, and decodes them where it read them, touching no GPU object" | holds | `io.rs:437-541`, `:452-458`, `:564-572` | streaming |
| 0016 | "the main thread loads them into the pools" | holds | `io.rs:1475-1506` | streaming |
| 0016 | "They take Chunk reads first, then mips, then Far proxy reads, each in the order requested" | holds; test `io.rs:2054` | `io.rs:171-176`, `:188`, `:200-213`, `:224-226` | streaming |
| 0016 | "a Far proxy's only while no Chunk read is taken and on all threads but one" | holds | `io.rs:42`, `:206-210`, `:257-266` | streaming |
| 0016 | "as a Far proxy read with its mips decodes them" | holds | `io.rs:508-526` | streaming |
| 0016 | "Decoding stays on those threads" | holds | `io.rs:319-335` | streaming |
| 0016 | "Only the main thread submits, presents or polls the wgpu `Queue` and `Device`, so map and work-done callbacks run at a known point of the frame" | holds: IO threads hold no GPU object and BLAS polling runs on the calling thread (streaming); `present` and `render`, which submits, run in `redraw` (State); every submit and poll the Renderer reaches runs on the thread calling `upload`, `render` or `flush_gpu_frame_stats`, and the static TLAS's worker submits nothing (render); the bench and capture submit, poll and map on the calling thread, each Goldens capture owning its device (tooling) | `io.rs:941-945`; `src/lib.rs:254-255`; `acceleration_structures.rs:594-621`; `src/capture/mod.rs:366-388` | streaming, render, tooling |
| 0016 | "Workers record `CommandEncoder`s and return `CommandBuffer`s; they push no error scopes" | broken in the comment, not in the ADR: the static TLAS's worker calls into wgpu while `render` records the refits: L1-R05 (the refit's SAFETY premise "only the main thread calls into wgpu meanwhile (ADR 0016)"); holds in code, newly applicable: the static TLAS's worker creates an encoder, records the build, finishes it and hands the command buffer back for the main thread to submit, pushing no error scope | `blas_refit.rs:193-196`; `acceleration_structures.rs:544-547`, `:592-623` | render |
| 0016 | "`desired_maximum_frame_latency` stays 2" | holds: the window's surface takes 2, `FRAMES_IN_FLIGHT` (State, render); capture keeps at most `FRAMES_IN_FLIGHT` frames in flight (tooling) | `src/lib.rs:166`; `src/render/mod.rs:41-44`; `src/capture/mod.rs:352`, `:366-382` | streaming, render, tooling |
| 0016 | "Per-frame data the GPU reads is written into the per-frame staging ring ..., a ring of mapped buffers whose slot is copied on the frame's encoder and reused only after its `map_async` callback fires"; `Queue::write_*` "stages outside the ring, against ADR 0017's rule" | holds: the camera and light uniforms, lights, bends, Instances and the slot count are the frame staging's six buffers; a slot is written only once `is_mapped`, which only the callback sets; no `Queue::write_*` outside tests | `renderer.rs:515-527`, `:765-781`; `staging.rs:90-107`, `:163-172` | render |
| 0016 | "the GPU stats come back through a ring of mapped readback buffers on the same rule"; "wgpu completes a map only after the last submission using the buffer finishes ... so the ring decides whether the CPU waits" | holds: a slot is read and reused only at `MAP_DONE` or `MAP_FAILED`; a timestamp copy is recorded only into a slot `Recorded` for a completed frame; `render` never waits on a map; `on_submitted_work_done` on the frame's last encoder fires in a submit or poll on the main thread | `readback.rs:251-268`, `:305-313`, `:388-419`, `:510-522` | render |
| 0016 | "Lua (ADR 0012) runs in one state on the main thread, with mlua's `send` feature off ...; Lua handler time counts against the 6 ms critical path" | not applicable: nothing implements it at this commit | — | streaming |
| 0016 | "the main pass's draw count is the visibility strategy's question, which ADR 0019 answers with one `draw_indexed_indirect` per pipeline and Hi-Z phase" | holds: the main pass draws only through `draw_indexed_indirect` from the frame's draw buffer, one per pipeline and phase in the depth passes and for the thin pipelines; the lit pipeline draws both phases in one, from offset 0 | `src/render/main_pass.rs:323`, `:329`, `:358-368` | report |
| 0016 | checklist: "bind groups are snapshots; whatever recreates a target rebuilds the groups holding it in one function called at creation and after every resize" | holds: `resize` recreates the targets and calls `bind_render_targets` on the pyramid, culling, main, shadow mask, tonemapping and light count passes, and re-derives the per-height LOD threshold and pixel size; the shadow mask's TLAS groups hold TLASes rebuilt in place | `targets.rs:61-65`; `renderer.rs:629-650`; `shadow_mask_pass.rs:192-207`; test `renderer.rs:1165` | render |
| 0017 | "Textures are ordinary mip chains streamed whole mip at a time in wgpu's safe API, chosen by CPU heuristics" | holds | `mips.rs:205-222`; `io.rs:1155-1162` | streaming |
| 0017 | "vertices and indices live in fixed arenas sized from the Budget at device creation and sub-allocated by an offset allocator" | holds: `PoolCapacities::from_budgets` sizes each Arena from the geometry Budget; each Arena is one buffer at that capacity with an `offset_allocator::Allocator` | `src/assets/pools.rs:73-85`, `:1336-1352` | report |
| 0017 | status: "each Channel is a partially bound binding array of texture arrays, one per Texture class of Textures sized to their source" | holds: a `TextureViewArray` of one `D2Array` view per class per Channel, each array at the Texture's own width, height, mip count and format | `material.rs:418-475`, `:1296-1303` | cook |
| 0017 | status: "loaded from cooked Blobs through a `StagingBelt` and sampled trilinear, the colour Channels at 16x anisotropy and the data Channels at 4x" | holds: every level through `LoadStaging::write_texture`; two samplers, `Linear` on all three filters, `anisotropy_clamp` 16 and 4; albedo and emissive read `colour_sampler`, the other three `data_sampler` | `material.rs:362-365`, `:1272-1283`; `main.wgsl:314-335` | shaders, cook |
| 0017 | status: "a Texture a Chunk loads arriving as its Resident tail, its finer mips streaming in by the priority below through the IO pool" | holds; reader side, `resident_tail_from_blob` decodes the tail alone | `io.rs:615-625`, `:331-334`, `:1121-1123`; `cooked.rs:376-384` | streaming, cook |
| 0017 | status: "while one a spawn loads keeps its full mip chain" | holds: spawns read with `reads_resident_tails: false`; `from_blob` decodes every mip | `io.rs:878-885`, `:621-623`; `material.rs:348-391` | streaming, cook |
| 0017 | status: "a Far proxy's atlas arriving with its mips down to the one its distance from the camera needed when it was requested, as the priority below judges a drawn use" | holds (the streaming read also applies to the ground Material's Textures); `down_to_from_blob` decodes from the finer of the picked mip and the tail's first | `io.rs:508-526`; `mips.rs:390-403`; `cooked.rs:389-397` | streaming, cook |
| 0017 | status: "drawn from its tail only where its Channel has no room for that chain or the camera has closed on it since" | holds: `push` places the tail, then the chain, and keeps the tail alone when the chain finds no room | `material.rs:361`, `:370-372` | cook |
| 0017 | status: "with the eviction below making room for a move into a full Channel" | holds: `stream_mips` calls `evict_for` when the target class has no room | `material.rs:1048-1059` | cook |
| 0017 | status: "geometry loads through the same belt into one vertex and one index arena ..., which refuse a Mesh that does not fit ..., and into three Meshlet arenas" | holds: every Arena write is `LoadStaging::write`; a short Arena returns `MeshRefusal::ArenaShort` with every Arena as it was | `pools.rs:495-509`, `:574-600`, `:1297-1315`, `:1374-1377` | cook |
| 0017 | status: "at device creation the pools derive their capacities from them, each Arena and table at its share ..., and each Channel a Texture class per 64 KiB of the texture pool's fifth, up to what the Capability floor binds" | holds: 13,107 classes a Channel (Block C1); holds in form in the world fit test, which asserts each count's most against `PoolCapacities::checked_in()` | `pools.rs:97-100`; `material.rs:19-20`; `src/gpu/mod.rs:301`; `src/scenes/world.rs:977-996` | cook |
| 0017 | "Each Channel is one partially bound binding array (...) of texture arrays in one bind group, an array per Texture class" | holds for the reads: each of the five Channel reads indexes its binding array by the class and samples the slice as the array layer | `main.wgsl:309-335` | shaders |
| 0017 | "the Textures of one shape, size, mip count and format alike, none padded or resampled to share one" | holds: `TextureShape` is width, height, mip count and format; `place` matches on the whole shape | `material.rs:94-110`, `:173-215` | cook |
| 0017 | "A Layer names a class and a slice of its array, Layer 0 stays the neutral default" | holds: WGSL's `layer / SLICES_PER_CLASS`, `layer % SLICES_PER_CLASS` inverts Rust's `class * SLICES_PER_CLASS + slice`; each Channel's neutral 1×1 Texture is class 0 slice 0, never released; the normal Channel's neutral is now exact, which closes M3.5's L1-C03 | `common.wgsl:403-407`; `mod.rs:315`; `material.rs:136-141`, `:526-559` | shaders, cook |
| 0017 | "the fixed 8 Layers per Channel go" | holds: a Layer is `class * SLICES_PER_CLASS + slice` over up to `CLASSES_PER_CHANNEL` classes; no fixed Layer count remains | `src/assets/material.rs:18-23`, `:138-140` | report |
| 0017 | "A class's slice count is fixed when it opens, at most the Capability floor's 256 array layers ..., and a full class's shape opens another" | holds: the first class of a shape has one slice, each later one twice the newest's, capped at `SLICES_PER_CLASS` = `FLOOR_TEXTURE_ARRAY_LAYERS`, 256, which WGSL reads too; the array is created at that count once (Block C7) | `material.rs:24`, `:199-214`, `:430-449`; `src/gpu/mod.rs:305-306` | shaders, cook |
| 0017 | "an empty slice still holds memory, which `gpu_memory_bytes` counts" | holds: a class's array is charged at creation for every slice | `src/stats/gpu_memory.rs:166-186`; `material.rs:430-448` | tooling, cook |
| 0017 | "A Texture whose resident mips change moves to the class of its new shape by copy" | broken in part: `stream_mips` writes the new mips into the target slice and copies the Resident ones after them, and `evict_finest_mip` copies the chain from its next mip into the coarser class, but a new shape whose mip 0 is not whole blocks opens a class wgpu refuses, a panic: L1-C08 | `material.rs:1064-1081`, `:1192-1201` | cook |
| 0017 | "Metal's `useResource` ... and wgpu-core's tracking then follow the classes, not the Textures" | holds, pool side: one view per class is bound, however many slices it holds | `material.rs:451-473`, `:1296-1303` | cook |
| 0017 | "Residency changes are batched so that bind group is rebuilt at most once per frame" | holds: placements set `views_changed`; `refresh_bind_group` rebuilds once if any did, called once at the start of `render` | `material.rs:472`, `:1241-1266`; `renderer.rs:813` | render, cook |
| 0017 | gate: "the rebuild plus Metal's per-texture `useResource` ..., measured at 10k textures on the M4 Pro, fits ADR 0013's 6 ms critical path" | holds, pool side: the rebuild is timed and logged with the bound class count | `material.rs:1246`, `:1255-1264` | cook |
| 0017 | "Priority is computed on the CPU: the finest mip any visible Instance needs, from distance and the Primitive's UV texel density (stored at cook time, per Primitive ...) against the 1440p pixel footprint" | holds: streaming computes it; the cook stores the density per Primitive, a Far proxy's from its atlas UVs with each coarser LOD carrying LOD 0's; the reader reads it and the Mesh pool keeps it per Primitive | `mips.rs:21-44`, `:414-416`; `src/cook/mesh.rs:435-453`; `pools.rs:516-520` | streaming, cook |
| 0017 | "Chunks inside the load radius are requested ahead" | holds | `mod.rs:230-251` | streaming |
| 0017 | "No GPU feedback readback until a measured miss rate asks for it" | holds: no readback in the module | `mips.rs:133-249` | streaming |
| 0017 | "The visible Instances are the drawn Placements of the loaded Chunks and the Renderables, the latter at their World transforms on each plan as they move, frustum aside" | holds | `mips.rs:162-202`; `io.rs:1109-1119` | streaming |
| 0017 | "each is judged at the nearest point of the sphere around its Mesh's bounds, at its smallest scale" | holds | `mips.rs:374-381`, `:407-427` | streaming |
| 0017 | "against the footprint of a pixel at the corner of a 16:9 view 1,440 pixels tall, where a pixel's angle is narrowest" | holds: 2 tan(fov/2) / 1440 / (1 + tan²θ_corner), the radial footprint | `mips.rs:21-32` | streaming |
| 0017 | "a Texture's mips from the finest needed to its finest Resident are requested in one read, finest-needed first" | holds | `mips.rs:213-221`, `:243-244`; `io.rs:564-572` | streaming |
| 0017 | "The mips of the Chunks and Far proxies held past their radius ... are requested as the drawn ones' are and are never late, as Unreal's streamer prestreams ... there one mip coarser" | holds; matches the `deviations.md` row | `mips.rs:56-59`, `:320-323`; `io.rs:1093-1108` | streaming |
| 0017 | "each request is judged from where the camera, closing ... at its last frame's rate, will be the Chunk streamer's least lead of frames later, at most a Chunk edge nearer" | holds | `mips.rs:49`, `:312-319` | streaming |
| 0017 | "eviction ranks a Texture by that same need, while the late mips are still judged from where the camera is" | holds | `mips.rs:209`, `:293`; `io.rs:1152-1154` | streaming |
| 0017 | "The Materials rays reach off screen ... take the Resident tail as their floor, which the next consequence keeps loaded" | holds, pool side: a Texture's tail slice is released only when the Texture itself is freed | `material.rs:407-413`, `:787-801`, `:1082-1084`, `:1202` | cook |
| 0017 | "Each texture's Resident tail (mips of 64 KiB and smaller) never leaves" | holds for the tail `resident_tail_level` picks; drift on the definition: a Texture whose first mip of 64 KiB or less is not whole blocks takes its whole chain as its tail: L1-C09 (cook, the pools); holds in the ledger's split (tooling) and as the cook lays out a 2,048² chain (cook, the cooker) | `cooked.rs:253-269`; `material.rs:356-366`; `src/stats/gpu_memory.rs:106-107`, `:175-181` | tooling, cook |
| 0017 | "When finer mips arrive, the Texture moves to the class of its mips from its finest Resident one and its tail's slice stays taken, charged twice in `resident_tails` (87,408 bytes for a 2,048² BC7 or BC5 Texture, 1.6% of its chain)" | holds: the chain's class repeats the tail's mips; the ledger charges every level of 64 KiB or less a slice to `ResidentTails`; 87,408 and 1.56% recomputed (Block C2) | `material.rs:370-390`; `src/stats/gpu_memory.rs:175-181` | cook |
| 0017 | "When a request would exceed the texture pool's capacity, which is set from its Budget" | broken, known: the pool's Channels are capped in Texture classes, not bytes, so no byte figure of its Budget enters a request; the open issue both sections' *Open issues read* name covers it, and no Finding does | `src/assets/material.rs:231-245`; `material.rs:199-201`; `pools.rs:97-100` | streaming, cook |
| 0017 | "the finest mip of the lowest-priority texture goes first, least recently used breaking ties" | holds: the victim minimises (`Reverse(finest_mip_needed)`, `last_used_plan`, Asset ID) | `mips.rs:286-300`; `material.rs:1102-1118`, `:274-282` | streaming, cook |
| 0017 | "a mip loads at priority p and is evicted only below p − 1, so it does not thrash" | holds; test `mips.rs:454-482` | `mips.rs:294-295`, `:454-482` | streaming |
| 0017 | "the candidates are the Textures whose chain lies in the very class the request needs, whose slice it takes" | holds: the plan's room check keys evictable needs by Channel and class shape; `evict_for` keeps Textures whose `mips_from(resident_level)` is the target shape | `mips.rs:236-241`; `material.rs:1105-1107`, `:1143-1171`, `:173-187` | streaming, cook |
| 0017 | "the one evicted moves to the class of its mips from the next, or falls back to its tail only when it needs no finer mip than the tail's, so it never re-requests what it lost" | holds (Block C11) | `material.rs:1125-1139`, `:1183-1208` | cook |
| 0017 | "a request is made only when its move finds room, free or by an eviction" | broken: L1-F01 | `mips.rs:233-242` | streaming |
| 0017 | "One vertex arena and one index arena, and the Meshlet arenas, each at most 2^31 − 1 bytes" | holds: asserted at creation; the checked-in largest is 55,197,040 bytes | `pools.rs:1337-1340`, `:1411-1427` | cook |
| 0017 | "sub-allocated with the `offset-allocator` crate" | holds: an `Allocator` per Arena, in elements; `bytes_in_use` is exact (see *Notes*) | `pools.rs:1352`, `:1360-1371` | cook |
| 0017 | "the Capability floor's 128 MiB `max_storage_buffer_binding_size` caps that arena" | holds: 55,197,040 bytes against 134,217,728, pinned by a test | `pools.rs:1416`; `src/gpu/mod.rs:824` | cook |
| 0017 | "No growth at runtime: a load that does not fit is refused and logged" | broken in part: the Arenas, the Mesh table and the Material table never grow and refuse with a log (`error!`, `warn!`); a Texture whose Channel has no room for its tail is logged and given Layer 0, not refused, and not counted: L1-C07 (cook); holds for the culling pass's buffers (shaders) and the Renderer's Instance, light, uniform and TLAS buffers (render) | `material.rs:653-655`, `:679-690`; `culling.wgsl:540-554`; `renderer.rs:440-469` | render, shaders, cook |
| 0017 | "The CPU shadow of vertex and index data goes" | holds: a loaded `Mesh` keeps its vertex range, bounds, texel densities and Material Handles | `mesh.rs:6-20`; `pools.rs:510-522` | cook |
| 0017 | "The 8 GB of ADR 0013 splits into Budgets for geometry arenas, the texture pool, Resident tails, render targets, acceleration structures and staging" | holds: `bench/budgets.json` holds those six kinds beside the seventh, 5,955,911,680 bytes in all under 8,000,000,000, and the ledger counts by the same kinds | `bench/budgets.json:9-17`; `src/stats/gpu_memory.rs:9-24` | report |
| 0017 | "the vertex, index, Meshlet, Meshlet vertex and Meshlet triangle Arenas take 65.8%, 13.0%, 1.1%, 13.3% and 6.8% of the geometry Budget" | holds (Block C1) | `pools.rs:19`, `:82-85` | cook |
| 0017 | "as `helmet`'s, `town`'s and the world scene's Meshes fill them ..., the world scene's with the Far proxies of its densest far field: the 282 within the View distance and a Chunk's edge of a camera on a 64 m grid" | holds in form: the fit test walks a camera over the same 64 m grid with the same 2,256 m and sums each Arena's most; the 282 is the densest far field by bytes, which needs a run (see *Notes*) | `src/scenes/world.rs:844-997` | cook |
| 0017 | "the Mesh table an entry per 5,124 bytes of it, `town`'s mean Mesh" | holds: 83,886,080 / 5,124 = 16,371 | `pools.rs:33`, `:94` | cook |
| 0017 | "the Material table an entry per 19,267,584 bytes of the texture pool's, the largest Material of the Reference scenes, `helmet`'s, past its Resident tails" | holds: 3 × 5,505,024 + 2,752,512 = 19,267,584; 222 entries (Block C2) | `pools.rs:38`, `:95` | cook |
| 0017 | "a second count of 285 entries, which only a Far proxy's Material takes ...: neither count takes an entry of the other's, a load past either is refused, logged and counted" | holds: `count_capacities` and `entries_taken` per `MaterialUse`; `refused_as_full` warns and counts; a full count frees only its own unused Materials; one SSBO of 222 + 285 entries | `pools.rs:43`, `:96`; `material.rs:674-690`, `:752-757` | cook |
| 0017 | "the baked Textures' bytes stay in the texture pool's Budget" | holds: a Far proxy's Textures load into the Channels as any Texture | `material.rs:900-916`; `loader.rs:230-235` | cook |
| 0017 | "the static TLAS's two TLASes 57% of the acceleration structures', at 289 bytes an entry" | holds: `STATIC_TLAS_PER_MILLE = 570`, two TLASes at `TLAS_BYTES_PER_INSTANCE = 289`; the fit test bounds the created TLASes by it | `renderer.rs:211`, `:223`, `:271-274`, `:1128-1134` | render |
| 0017 | "the Instance slots 30% of the scene and pass buffers', at 136.2 bytes a slot with its offsets and visibility bit, in whole scan blocks" | broken in its figure, `drift`: a slot's offsets are now four u32, the three scan components and the LOD error word, so a slot is 144.2 bytes and the checked-in Budgets give 488,448 slots, not 517,376: L1-R03 (shaders' L1-S05 killed as its duplicate) | `renderer.rs:204`, `:215-218`; `culling.wgsl:446-457`; `src/render/bind_groups.rs:131` | render, shaders |
| 0017 | "and the lights 0.1%, at 48 bytes" | holds: `LIGHTS_PER_MILLE = 1`, `LightData` 48 bytes, 4,893 lights pinned | `renderer.rs:207`, `:262-265`; `uniforms.rs:17-29` | render |
| 0017 | "the scene and pass buffers' Budget also holds the two tables and the passes' buffers of fixed size" | holds: the Mesh table, the Material SSBO and the passes' buffers are created under `SceneAndPassBuffers`, and a test bounds that kind's created bytes by its checked-in Budget | `src/render/renderer.rs:229-231`, `:1112-1129`; `src/assets/pools.rs:276`; `src/assets/material.rs:572` | report |
| 0017 | "and the staging Budget the frame staging's slots at their largest" | holds: frame staging slots are created through the ledger as `Staging`, never past `max_slot_size`; `most_uploaded_bytes` sums the six buffers `FrameStaging` is bounded by | `renderer.rs:279-287`, `:515-527`; `staging.rs:81-87`; `renderer.rs:1135-1138` | render |
| 0017 | "A seventh kind holds what none of those six does: the scene and pass buffers, the Instance and light buffers, the Mesh and Material tables and the passes' own buffers" | holds: `GpuMemoryKind::SceneAndPassBuffers`, under which the Renderer creates the Instance and light buffers, the pools the Mesh table and Material SSBO, and the passes their own | `src/stats/gpu_memory.rs:24`; `src/render/renderer.rs:460-469`, `:503-512`; `src/render/culling_pass.rs:230` | report |
| 0017 | "The Renderer and its Pools create the buffers, textures and TLASes they count through one ledger, and charge it their BLASes' bytes; it keeps a running total per kind from creation until drop, and `gpu_memory_bytes` is the sum of those totals" | holds; the streaming module creates no GPU object outside its tests | `src/stats/gpu_memory.rs:104-117`, `:211-249`; `pools.rs:1342-1351`; `material.rs:430` | streaming, tooling, cook |
| 0017 | "What ADR 0013 leaves out, the GPU stats readback and the load staging, bypasses it" | holds in code; ADR 0013 does not leave the load staging out: L1-R04 (render); holds in code, naming L1-R04 for ADR 0013's side (tooling); holds, pool side, naming L1-R04 as the drift on this clause (cook) | `staging.rs:179-184`, `:344-353`; `readback.rs:193-200`; `pools.rs:1374-1377` | render, tooling, cook |
| 0017 | "A `TRANSIENT_ATTACHMENT` counts as zero only when `transient_saves_memory` is `Some(true)`" | holds: the ledger is made from the adapter's info; the pbr-sweep test pins the 4x colour at 0 on Metal and full elsewhere | `renderer.rs:394`; `src/stats/gpu_memory.rs:128-131`, `:282-294`; `tests/frame_stats.rs:134-137` | render, tooling |
| 0017 | "Acceleration structures join the ledger ..., each BLAS and TLAS at the size its driver gives through wgpu-hal's build-sizes query ... and each TLAS with wgpu-core's buffer of its Instances, 72 bytes a slot on Metal and 64 elsewhere" | holds: the pools charge `blas_bytes` at load with each BLAS's own flags and geometry (cook); `create_tlas` charges `tlas_bytes`, and the test asks the driver through wgpu-hal and adds 72 or 64 a slot (tooling); the query is built as wgpu-core builds its own, plus wgpu-core's instance buffer (render, checklist row) | `pools.rs:621-625`, `:683-684`; `src/stats/gpu_memory.rs:188-209`, `:339-426` | render, tooling, cook |
| 0017 | "A compacted BLAS's size stays an estimate ...: the fraction of its built size compaction kept where measured, ... picked by the adapter's vendor, then its backend" | holds: compacted bytes move from the uncompacted total as each BLAS is compacted and are charged at `compacted_blas_fraction`, rounded (Block C6); the fraction is chosen by vendor and backend (render, checklist row) | `pools.rs:828-834`, `:999-1005`; `src/render/acceleration_structures.rs:36-51` | render, cook |
| 0017 | "Streaming uploads go through a `StagingBelt`, never `Queue::write_texture`" | holds: the streaming module writes only through `LoadStaging`; every write, re-pitched mip and Texture class move is recorded on the load encoder, the dither mask included; no runtime `write_texture` or `write_buffer` outside tests, none in `src/assets/` | `io.rs:1155-1162`; `staging.rs:207-215`, `:225-278`; `pools.rs:574-600` | streaming, render, cook |
| 0017 | "the engine caps the bytes written into it per frame at the staging Budget, and an upload past the cap waits for a later frame" | holds: the streaming module enforces it, the State passes the cap from the Renderer's capacities (a test drives it at 3,000 bytes), and the staging side counts row padding | `io.rs:1313-1325`, `:1139-1144`; `src/state.rs:179`; `staging.rs:368-394` | streaming, render |
| 0017 | "the IO pool (ADR 0016) decodes into CPU memory and the main thread copies it into the belt" | holds | `io.rs:437-541`, `:1475-1506` | streaming |
| 0017 | "The cap holds each frame's streamed Chunk loads to a third of the staging Budget, ... a third of what those slots at their largest leave of it" | holds: `load_bytes_per_frame` is what the frame staging's slots at their largest leave of the staging Budget, over three | `renderer.rs:244-248`, `:266-275`, `:1135-1138` | render |
| 0017 | "each Chunk is sized from its decoded Assets before it loads, nearest the camera first, and one that alone writes past the cap loads with no other writes in its frame rather than never" | holds | `io.rs:1298-1308`, `:1320-1324`, `:1421-1469` | streaming |
| 0017 | "A capture's Settled wait loads past the cap, so its load order stays fixed, and a spawn's loads are not capped" | holds; on capture's side `capture` and `capture_frames` settle every drawn frame | `io.rs:1314-1316`, `:1142`, `:878-885`; `src/capture/mod.rs:160-177` | streaming, tooling |
| 0017 | "A frame's Chunk loads also place at most 131,072 Instances ...; a Chunk that alone places more is placed alone (the cap counts at placement ...)" | holds | `io.rs:760`, `:1378-1388` | streaming |
| 0017 | "a Texture read on the main thread for a Chunk, one a load still in flight claimed, is its Resident tail alone, as the IO pool reads it" | holds; test `io.rs:2161` | `io.rs:615-625`, `:1432`, `:1480`, `:2161` | streaming |
| 0017 | "a Chunk that needs an Asset a Chunk read in flight claimed waits for the reads in flight when its own came back instead, so the main thread reads one only after a failed read or a refused load left it claimed" | holds for the wait; broken after a failed read or a refused load: L1-F03 | `io.rs:404-413`, `:1395-1416`, `:54-70`, `:597-604` | streaming |
| 0017 | "A frame also loads at most 3 Chunks, whatever they place or write: ... a load past the count waits for a later frame, nearest the camera first, and a Settled wait loads past it" | holds; tests `io.rs:1794`, `:1809`, `:1824` | `io.rs:765`, `:1317-1319` | streaming |
| 0017 | "A Mesh a streamed load brings that the pools lack does not hold its frame ...; a Settled frame and a spawn's loads still wait for theirs" | holds; test `io.rs:1762`; pool side `defer_blases`, `poll_blases`, `blas_ready` and `wait_for_blases` | `io.rs:941-945`, `:1378-1380`, `:1508-1511`; `pools.rs:727-763` | streaming, cook |
| 0017 | "A Far proxy's load counts as a Chunk's does: a frame's 3 loads are its Chunks' first, then its Far proxies', nearest first" | holds: `loads` is not reset between the two streamers | `io.rs:1011`, `:1284-1289`, `:1255` | streaming |
| 0017 | "The cooked format (ADR 0011) stores each mip as a separately loadable byte range of the Texture's Blob, with the Resident tail as one range, and the Mesh Blob each Primitive's UV texel density" | holds: writer side, one level index entry and one zstd frame per mip, smallest first, and the density per Primitive; reader side, each level through its KTX2 index entry, decompressed on its own (the Blob is read whole, see the streaming section's *Notes*) | `src/assets/cooked.rs:333-345`, `:668-672`; `cooked.rs:459-474`, `:722` | cook |
| 0018 | "each source file gets a 128-bit GUID, written once into its `assets/manifest.json` entry" | holds: every manifest entry carries a 32-hex-digit `guid`, and `cargo test` refuses one missing, malformed or shared with another entry | `assets/manifest.json:4`; `tests/asset_manifest.rs:176-184` | report |
| 0018 | "An Asset ID is the first 128 bits of SHA-256 over (GUID, Asset kind, glTF name, Channel for a Texture)" | holds, in that order, each part tagged and each string length-prefixed; a Texture is named by its image's name or index; unchanged since M3.5 but for the Chunk kind (3) | `src/assets/cooked.rs:103-143`; `src/cook/mod.rs:965-969`; `src/cook/mesh.rs:736`; `cooked.rs:1221-1229` | cook |
| 0018 | "The glTF index replaces the name when the name is missing or not unique" | holds: `asset_names` gives `Index` to a missing name or one another item shares, for images, Materials and Meshes alike | `src/cook/mod.rs:1071-1082`, `:965`, `:970`; `src/cook/mesh.rs:681-682` | cook |
| 0018 | "A cooked Asset is a Blob named by the SHA-256 of its bytes, found through a cook index that maps Asset ID to Blob, and rebuilt when its Cook key changes" | holds: every write goes through `CookedCache::write` under the Asset's Cook key and `is_fresh` compares it; the reader reads `index/<id>` then `blobs/<hash>` and does not rehash what it reads (see *Notes*) | `src/cook/mod.rs:645-694`, `:902-907`; `src/assets/cooked.rs:1174-1185`; `cooked.rs:1119-1152` | cook |
| 0018 | status: "`praxis cook` and every dev launch cook Materials, Textures and Meshes into Blobs ..., and the runtime loads Materials and Textures ... and Meshes ... from them by Asset ID; the reader rebuilds their Meshlets ..., which the Mesh pool keeps in three Arenas" | holds: `AssetLoader` and `AssetReader` take `AssetId` only; `CookedMeshlet::split` rebuilds each Meshlet; the status sentence now names the Meshlet Arenas, which closes M3.5's L1-C06 | `io.rs:446-450`, `:1520-1523`; `loader.rs:11-69`; `cooked.rs:534-588` | streaming, cook |
| 0018 | status: "`praxis cook` and every dev launch also cook each Reference scene's static content into Chunks" | holds: `cook_sources` cooks every Chunk of the scenes read; a launch cooks each Chunk it loads | `mod.rs:661-675`; `launch.rs:172-183` | cook |
| 0018 | status: "Plain `praxis cook` leaves out the `world` scene, whose library of more than 8 GB cooks only as `praxis cook world` or on a miss" | holds in the command; `cook`'s doc comment still says plain `praxis cook` cooks every scene: L1-C03 | `src/bin/praxis.rs:493-498`; `src/cook/mod.rs:584-586` | cook |
| 0018 | status: "A dev launch cooks only what it loads: each Asset as its load finds the Blob missing or stale, with the Assets it names" | broken in part: a Chunk the launch keeps (a Placement names a Mesh it did not read) returns before cooking any Asset it names, and a streamed load reads those Blobs from the cache unchecked: L1-C05, which the verifier confirmed against the streaming row's `holds`; every other load cooks its Asset first | `io.rs:825-847`, `:1524-1529`; `launch.rs:172-178`; `src/streaming/io.rs:1437-1466` | streaming, cook |
| 0018 | status: "while `praxis cook <scene>` cooks a scene whole" | holds: `cook_scene` reads the scene's sources and cooks every Asset, Chunk and Far proxy of them | `src/cook/mod.rs:603-613`, `:638-759` | cook |
| 0018 | "A glTF mesh renamed in Blender gets a new Asset ID" | holds by construction: the Mesh's name is hashed; the seam renames NormalTangentTest's Mesh and reads the old ID's Blob still there | `src/cook/mesh.rs:736`; `tests/cook.rs:743-799` | cook |
| 0018 | "The cooker warns when an ID that something references no longer exists" | holds, with a ceiling the code states: M3.5's L1-C01 is fixed, as each reference is checked against what this cook wrote or found fresh, not against the never-pruned cache; while any source of the referring scene is absent, every ID the cache holds passes, a stale one included (the `ponytail:` comment); a Far proxy's references are checked only by a dev launch, which warns on an ID absent from `entry_by_id` unless `may_be_kept` excuses it | `src/cook/mod.rs:615-636`, `:909-934`; `src/cook/launch.rs:193-195`; `launch.rs:236-241` | cook |
| 0018 | "Cook key: SHA-256 over what the Asset cooks from (a Texture's source image, a Mesh's decoded Primitives and their Material IDs, a Material's own Blob; by content ...)" | holds: a Texture over its source bytes or a generated image's pixels and size; a Mesh over every `Geometry` field, destructured so a new field fails to compile, and its Material ID; a Material over its Blob; a Chunk over its payload, all it cooks from | `src/cook/texture.rs:135-158`; `src/cook/mesh.rs:57-90`; `src/cook/mod.rs:882-888`; `chunk.rs:98-109` | cook |
| 0018 | "the Cook keys of its dependencies, every cook setting, the format version and the encoder versions" | holds: the normal Texture's key, tagged `N`; `texture_zstd_level`; the three format versions; the encoder versions match `Cargo.lock`, `meshoptimizer.h:15` and `vertexcodec.cpp:125`; a Chunk's key hashes its magic, format version and zstd's version and level | `src/cook/texture.rs:21`, `:277-286`; `src/cook/mesh.rs:45-51`; `chunk.rs:98-109` | cook |
| 0018 | "The metallic-roughness Texture depends on the normal Texture (Toksvig, below)" | holds for glTF and generated sources, the first Material's normal Texture where several share the image | `src/cook/mod.rs:1019-1030`, `:830-853`; `src/cook/texture.rs:98-104`, `:144-147` | cook |
| 0018 | "Each dev launch recomputes the keys of its scene's Materials and Meshes as it starts, of each Chunk and Texture as it loads it, and, as it loads a Far proxy, of the Textures that proxy bakes from ..., and cooks only the stale Assets it loads" | holds: `CookingLoader::new` reads the sources, whose Material and Mesh keys `read_sources` computes; a Chunk's key on `Sources::chunk`; a Texture's on first need, cached | `launch.rs:49-59`, `:172-183`, `:202-206`; `mod.rs:813-866` | cook |
| 0018 | "The Cooked cache lives in git-ignored `cooked/` and is never evicted. Deleting it is always safe." | holds: no cooker or reader code removes an index file or a Blob; the seam deletes the cache and cooks again | `src/cook/mod.rs:638-759`; `tests/cook.rs:254-274`; `cooked.rs:1082-1185` | cook |
| 0018 | "The runtime loads Blobs only (ADR 0011)" | holds: a build without `cook` links none of the four decoders and renders every CI Golden with `assets/` removed; every scene generator compiles only with `cook`, and the spawns load by ID through the `AssetLoader` | `.github/workflows/ci.yml:197-247`; `Cargo.toml:41`; `src/scenes/world.rs:281-287`; `src/scenes/mod.rs:159-161` | tooling, cook |
| 0018 | "Each Blob starts with a magic and a format version (a Texture Blob is KTX2, with its format version in a key-value entry)" | holds: `PXMA` 2, `PXME` 8, `PXCH` 1 with one zstd frame after it, and `PXTXVERSION` = 2 as a little-endian u32; a Far proxy is an ordinary Mesh Blob and its atlases ordinary KTX2 Texture Blobs | `src/assets/cooked.rs:21-40`, `:54-56`, `:179-180`, `:322-327` | cook |
| 0018 | "A mismatch is refused with a re-cook message; there is no migration code" | holds: each refusal says "re-cook with `praxis cook`"; no branch reads an older version | `cooked.rs:1064-1075`, `:416-427`, `:204`, `:705` | cook |
| 0018 | "Engine data that no cook format holds exactly is compiled into the binary instead, from a copy a test pins to its source in `assets/`: the tonemapping pass's 64² blue-noise dither mask" | holds, cook side: `read_sources` reads only `.gltf` and `.glb` entries, so the dither mask is never cooked; read here: the mask is `include_bytes!` in the tonemapping pass, and a test asserts it is the asset's | `src/cook/mod.rs:784-787`; `src/render/tonemapping_pass.rs:25-26`, `:197` | cook, report |
| 0018 | "The runtime still opens no source path" | holds: `CookedCache` opens `cooked/index/<id>` and `cooked/blobs/<hash>` only; the two glTF scenes name their source by manifest GUID and their Meshes by glTF name, and `source_files` exists only under `cook` | `cooked.rs:1119-1125`, `:1146`; `src/scenes/mod.rs:140-157`, `:806-814` | cook |
| 0018 | "Textures are KTX2 with every level zstd-supercompressed, stored smallest mip first, so the Resident tail ... is one contiguous byte range" | holds: levels written in `rev()` order, each compressed on its own, scheme Zstandard; the reader requires `Zstandard` and takes each level by its index entry | `src/assets/cooked.rs:336-345`, `:355`; `cooked.rs:429-432`, `:459-466` | cook |
| 0018 | "Rows are stored unpadded, and the loader re-pitches them to 256 bytes as it writes into the `StagingBelt`" | holds: `levels` hold unpadded rows; `write_texture` re-pitches to `COPY_BYTES_PER_ROW_ALIGNMENT` | `staging.rs:225-256`, `:407-426`; `cooked.rs:244-246` | render, cook |
| 0018 | "Formats per Channel, without RDO: albedo and emissive BC7 sRGB, normal BC5 (z rebuilt in the shader), metallic-roughness BC5 linear, occlusion BC4 ... The encoder is `intel_tex_2` at its basic profile" | holds: the format per Channel; the basic BC7 profiles, alpha or opaque by the level's texels; a Texture Blob holds one of the three formats | `src/cook/texture.rs:179-187`, `:622-627`; `cooked.rs:272-285`, `:433-441` | cook |
| 0018 | "A Far proxy's baked albedo alone encodes at its `ultra_fast` profile" | holds: `with_fastest_bc7` selects `ultra_fast` and enters the Cook key; both atlases are flagged, and the flag reaches only the BC7 encode, the metallic-roughness atlas being BC5 | `src/cook/texture.rs:139-141`, `:179-206`; `src/cook/mod.rs:450` | cook |
| 0018 | "Mips: a 2×2 box filter in linear light, with normals renormalised per mip" | holds: sRGB decoded by table, the box mean, `renormalise` per normal mip | `src/cook/texture.rs:222-253`, `:351-398`, `:421-448` | cook |
| 0018 | "Toksvig widens each mip's roughness from the normal Texture's variance" | holds: `widened_roughness` from the normal Texture's carried variance | `src/cook/texture.rs:504-512`, `:565-590` | cook |
| 0018 | "The cooker refuses a Texture whose size is not a multiple of 4 or exceeds 8192, rather than resizing it" | holds: both refusals tested; the reader mirrors it and requires the full mip chain | `src/cook/texture.rs:169-178`; `tests/cook.rs:378-390` | cook |
| 0018 | "Meshes: a 28-byte vertex (position `Float32x3`, UV `Float32x2`, ... QTangent in `Snorm16x4`, its w kept non-zero ...), and u32 indices" | holds: asserted at compile time; indices decoded as `u32` | `src/assets/cooked.rs:485-497`; `src/cook/mesh.rs:642-669`; `cooked.rs:802` | cook |
| 0018 | "`meshopt` reorders for vertex cache, overdraw and fetch, then encodes, then zstd compresses" | holds | `src/cook/mesh.rs:485-492`, `:213`; `src/assets/cooked.rs:650-700` | cook |
| 0018 | "Each Primitive's triangles are cut, in that order, into Meshlets of at most 64 vertices ... and at most 124 triangles" | holds: the reader's `split` refuses a run past either | `src/cook/mesh.rs:503-535`; `cooked.rs:548-551`, `:562-565` | cook |
| 0018 | "Each Meshlet is the longest run the limits allow, as meshoptimizer's scan builder cuts them" | holds, and the cut on a degenerate triangle matches the Deviation `src/cook/mesh.rs:meshlet_triangle_counts` | `src/cook/mesh.rs:503-535`; `src/assets/cooked.rs:534-588` | cook |
| 0018 | "A Meshlet is stored as one byte, its triangle count; the reader rebuilds its vertex remap and local triangles from its run of the index stream, which BLAS builds read" | holds | `src/assets/cooked.rs:688-694`; `cooked.rs:815-832`, `:534-588`; `pools.rs:579-580` | cook |
| 0018 | "The Blob also holds the Mesh's bounding box for culling" | holds: the box of LOD 0's vertices and any box a Primitive adds, six f32 after the counts, into `MeshData` | `src/cook/mesh.rs:220-232`; `src/assets/cooked.rs:665-667`; `cooked.rs:715-718` | cook |
| 0018 | "Each Primitive stores its UV texel density ...: the square root of its summed UV area over its summed mesh-space area, in UV units per metre, the inverse of Unity's per-Mesh average ..., taken per Primitive, as a ratio of summed areas and as a length" | holds: summed in f64 over the source triangles, `sqrt(uv_area / mesh_area)`, 0 where the mesh area is 0; one f32 per Primitive; the registry difference this sentence records is L1-C10 | `src/cook/mesh.rs:435-453`, `:701-704`; `src/assets/cooked.rs:505-510`, `:670` | cook |
| 0018 | "The Blob holds the Mesh's LOD chain: the vertices once, which every LOD indexes, then per LOD its error in metres, each Primitive's run of its indices and its Meshlets, with every LOD's indices in one encoded stream" | holds; the error stored is the simplifier's, of position, normal and UV together, times the Mesh's extent (see *Notes*); the reader refuses a LOD error not finite or below the finer one's | `src/assets/cooked.rs:637-702`, `:605-616`; `cooked.rs:734-776`, `:802-814` | cook |
| 0018 | "Each LOD is simplified from the one before to half its triangles by meshoptimizer's attribute-aware simplifier, the whole Mesh in one call, normals and UVs weighted, ...; a chain holds at most 8 LODs and ends before a LOD that would keep more than 85% of the indices before it" | holds: `simplify_with_attributes_and_locks` on the finer LOD with a target of `finer.len() / 6 * 3`, weights 0.5 and 1.0, `MESH_MAX_LODS` = 8, a LOD over 0.85 of the finer dropped; the reader refuses a count outside 1 to 8 | `src/cook/mesh.rs:302-420`; `cooked.rs:711-714`; `pools.rs:483-486` | cook |
| 0018 | "A Thin Primitive's vertices are locked in the simplifier [ref:meshoptimizer-1.3-vertex-lock], so every LOD keeps its LOD 0 triangles and a Mesh of Thin Primitives alone has no coarser LOD" | holds: a `true` lock per vertex of a thin Primitive, passed as the Lock bit and extended by meshoptimizer to every wedge at the same position; with no vertex free, the chain ends at the 85% rule | `src/cook/mesh.rs:344-351`, `:780-812` | cook |
| 0018 | "Each Primitive stores a Thin Primitive's radius, zero for any other" | holds: zero reads as `None`; a radius not finite or below zero is refused | `src/cook/mesh.rs:205`; `src/assets/cooked.rs:671`; `cooked.rs:723-731` | cook |
| 0018 | "The cook refuses one unless each triangle is the side of a tube whose radius is within a quarter of the one declared, or a cap closing such a tube, ... and no vertex stores it" | holds: an edge whose normals differ lies on one ring when its centres are within 0.25 of the radius times the normals' difference; a triangle passes as a side with such an edge, or as a cap; tested by Mesh and Primitive; `town`'s declared radii are each within a quarter of their tube's | `src/cook/mesh.rs:98`, `:103-150`; `src/cook/mod.rs:1088-1099`; `src/scenes/town.rs:1076-1081` | cook |
| 0018 | "a cap's vertices keep their own normals (amended 2026-11-14)" | holds in intent, wording broken: since 2026-11-16 each cap vertex takes its ring's normal so that a cap widens with its side, which the ADR does not say: L1-C01 (cook, the cooker); the clause is false since the cook gives each cap vertex its ring's normal, which a test in `town.rs` pins: L1-C01 (cook, the Reference scenes) | `src/cook/mesh.rs:100-102`, `:151-158`; `src/assets/cooked.rs:511-514`; `src/scenes/town.rs:1452-1470` | cook |
| 0018 | "A Material is its own Asset: PBR factors plus one Texture Asset ID per Channel. Primitives reference it by ID" | holds | `src/assets/cooked.rs:156-201`; `src/cook/mod.rs:1054-1067`; `src/cook/mesh.rs:709-712`; `cooked.rs:203-231` | cook |
| 0018 | "A Chunk's edge is 256 m (4 by 4 cells of ADR 0014's 64 m grid; 40 by 40 Chunks cover 10 km)" | holds: `CHUNK_EDGE_METRES` 256; `CHUNKS_PER_SIDE` 40, 10,240 m a side; `chunk_square(-20..20)` lists 1,600 Chunks | `src/assets/cooked.rs:57-58`; `src/scenes/world.rs:26-27`; `src/scenes/mod.rs:397` | cook |
| 0018 | "A v1 Chunk is its own Asset, named from its scene and its integer coordinates, holding its Placements, its point lights and its bounds" | holds: `CHUNK_FORMAT_VERSION` 1 | `src/assets/cooked.rs:56`; `chunk.rs:67-78`; `src/scenes/mod.rs:345-350` | cook |
| 0018 | "A Placement holds a Mesh Asset ID, an optional Material Asset ID that overrides its Mesh's Materials ..., an f64 translation from the Chunk origin, and f32 rotation and scale" | holds: the generators build `CookedPlacement` with a `DVec3` translation from the World's origin, a `Quat` and a `Vec3` scale; `split` moves each translation to its Chunk's origin in f64; the reader checks the column layout's byte count before cutting | `chunk.rs:48-64`; `src/assets/cooked.rs:912-951`; `cooked.rs:850-881`; `src/scenes/world.rs:661-672` | cook |
| 0018 | "A light holds an f64 translation, a colour, an intensity in candela (ADR 0023) and a range in metres" | holds: `town`'s lamp lights are `CookedLight` from `LAMP_LIGHT`, 2,000,000 cd and 18 m, at f64 positions; `world` cooks no point light | `chunk.rs:48-64`; `src/assets/cooked.rs:912-951`; `src/scenes/town.rs:241-245`, `:421-429` | cook |
| 0018 | "Each Placement and light belongs to the Chunk that holds its pivot, and the bounds grow to hold every Placement's Mesh, which may reach past the Chunk's edge" | holds: `chunk_holding` floors x / 256 and z / 256; `chunks` grows the bounds by the eight placed corners of each Placement's source box; `chunk_content` keeps exactly the Placements and lights whose pivot is in the Chunk, and the generator's `reaching` never under-selects | `src/assets/cooked.rs:66-70`; `chunk.rs:19-42`, `:66-78`; `src/scenes/mod.rs:449-460` | cook |
| 0018 | "The Blob stores each field for all Placements, then for all lights, before zstd" | holds; level 9 | `src/assets/cooked.rs:890-953`; `cooked.rs:964-1029` | cook |
| 0018 | "The cooker lives in `src/cook/` behind the `praxis cook` command, with its encoder dependencies behind a default-on `cook` feature that a shipping build turns off" | holds | `src/cook/mod.rs:813-867`; `Cargo.toml:41` | cook |
| 0018 | "Procedural Reference scenes cook too, under Asset IDs derived from the scene name, so Goldens test the cooked path" | holds: every procedural Mesh, Material, Texture, Chunk and Far proxy ID is built from `AssetSource::Scene(self.name())`; `world`'s and `town`'s spawns load the bird and the ground by those IDs | `src/assets/cooked.rs:114-117`; `src/scenes/mod.rs:122-138`, `:345-369`; `src/scenes/town.rs:825` | cook |
| 0018 | "CI cooks before the golden test" | holds: the software Goldens job runs `cargo run -- cook` before `cargo test`, and the shipping Goldens job cooks before its golden test | `.github/workflows/ci.yml:181-187`, `:232`, `:247` | report |
| 0018 | "New dependencies ...: `intel_tex_2` (MIT or Apache-2.0 ...), `ktx2` (Apache-2.0), `meshopt` (MIT or Apache-2.0), `zstd` (BSD-3-Clause); `sha2` moves from dev-dependency to dependency" | holds: all five are under `[dependencies]`, `intel_tex_2` behind `cook`; each crate's own licence field matches the one named | `Cargo.toml:23-27`, `:41` | report |
| 0019 | "The opaque main pass is clustered forward: a 4x MSAA reverse-Z depth pre-pass, then a lit pass at depth equal that ... reads each one's shadow from a shadow mask, which a compute pass traces from the depth target before it, one ray per pixel and light" | holds: pass order culling one, Froxel lists, depth pre-pass, pyramid, culling two, phase two's depth and the shadow mask with rays on, lit, tonemapping; the Froxel pass after the first culling dispatch is the last audit's note, unchanged | `renderer.rs:863-938`, `:905-931` | render |
| 0019 | "Geometric edges get 4x MSAA ..., with alpha-to-coverage for masked materials (amended 2026-11-15, ADR 0027: alpha-to-coverage stays off, and partial coverage, a masked Material's included, is a written `sample_mask`)" | holds: as the ADR 0027 rows; masked Materials are M7's | `src/render/main_pass.rs:110-114`, `:113` | shaders |
| 0019 | "with shadow rays on, the depth passes also write each sample's triangle normal and the normal interpolated over it, 4 bytes per sample ... which the shadow mask pass reads so that a coarse smooth-shaded mesh's terminator follows its interpolated normals ...; the target is allocated either way" | holds: `surface_normals` called by both depth stages that write normals, the twin's included; the turn tests read `interpolated_normal`, `triangle_normal` decides the origin's side, the back-face height and the second-surface plane | `targets.rs:42-45`, `:247-252`; `main.wgsl:50-77`; `shadow_mask.wgsl:233-243` | render, shaders |
| 0019 | "a compute pass appends the Meshlets of the LOD selected for each Instance whose Mesh bounds reach into the view frustum to the compacted index buffer in slot order, in two phases" | holds: offset scans, then records, then triangles, per phase, each phase two Timed wgpu compute passes (slots, then triangles); every record and packed index written at its slot's prefix sum, atomics only taking a maximum or a count | `culling_pass.rs:340-388`; `renderer.rs:38-53`; `culling.wgsl:441-460`, `:578-601` | render, shaders |
| 0019 | "Phase one appends those visible the frame before, one bit per slot" | holds: `capacities.instances / 32` words, zeroed at creation; `visible_before` reads one bit per slot; with CULLING off phase one draws every Instance in use | `culling_pass.rs:276-281`; `culling.wgsl:363-367`, `:418-420` | render, shaders |
| 0019 | "a depth pre-pass draws them into 4x MSAA reverse-Z `Depth32Float` cleared to 0 (ADR 0015) and stores it" | holds | `main_pass.rs:110-114`, `:291-292`; `targets.rs:328-337` | render |
| 0019 | "A compute pass reduces that depth into the Hi-Z pyramid, each texel the minimum of every sample below it" | holds: Rust side; `depth_pyramid.wgsl` unchanged since `8bc34eb1`, so M3.5's row stands | `depth_pyramid_pass.rs:97-122`; `targets.rs:172-191`, `:212-222` | render, shaders |
| 0019 | "Phase two appends the others whose bounds lie anywhere nearer than the pyramid's minimum over the texels their screen rectangle covers, and rewrites the bits" | holds: the whole pyramid bound in `FRAME_DRAW`; `occluded` unchanged since M3.5 and fed bounds that a Thin Primitive's widening or a small Instance's scale only enlarges; phase two draws only a slot phase one left `NOT_DRAWN` and writes every bit of its block | `culling_pass.rs:463-490`; `bind_groups.rs:126-144`; `culling.wgsl:220-267`, `:398-417` | render, shaders |
| 0019 | "Phase two's Instances draw into depth: with shadow rays off, at the start of the lit pass; with them on, in a pass of their own, which like the pre-pass then writes each sample's surface normals too" | holds, the twins included | `main_pass.rs:283-330`, `:358-363`; `renderer.rs:905-922` | render |
| 0019 | "after which a compute pass traces, from each pixel's nearest sample, one shadow ray toward each light that reaches it ... and, where the pixel's other samples show another surface, traces from the farthest of those too, into the mask's second layer" | holds: traced after phase two's depth pass and before the lit pass; the greatest of the four depths, first such sample; samples off its triangle's plane by more than a pixel's width, the least depth among them, traced into layer 1; 4x MSAA pinned where the pass reads samples | `renderer.rs:905-931`; `shadow_mask_pass.rs:360-383`; `shadow_mask.wgsl:408-493`, `:501-520` | render, shaders |
| 0019 | "A lit pass draws both phases' at depth equal into a transient 4x colour attachment, memoryless on Apple GPUs, that it resolves into the `Rgba16Float` HDR target, which a tonemapping pass then writes into the target" | holds: `TRANSIENT_ATTACHMENT`, cleared, `Discard`, resolved into `hdr`; the twins' `vs_thin` has an `@invariant` position, so the lit twin's depths meet the depth twins' bit for bit | `targets.rs:200-205`, `:377-400`; `tonemapping_pass.rs:151-177`; `main.wgsl:94` | render, shaders |
| 0019 | "with one `draw_indexed_indirect` per phase for the depth pipeline and one of both phases for the lit pipeline, and one more per phase and pipeline for Thin Primitives and small Instances (ADR 0027)" | holds: seven indirect draws and the tonemapping triangle, counted as 8 with rays on or off; the culling pass writes `draw`, `phase_two_draw` and each phase's thin draw, each with `instance_count` 1; an entry point per twin | `main_pass.rs:315-368`; `renderer.rs:943-948`; `culling.wgsl:43-61`, `:497-526` | render, shaders |
| 0019 | "lit by every directional light and by the point lights of each pixel's Froxel ... each hidden where the shadow mask says so, the directional lights' rays first ..., then those point lights'" | holds, unchanged since M3.5's rows: the body of `fs_main` moved into `lit` without change past the Texture reads; the twin shades through the same `lit` | `main.wgsl:365-457`, `:298-301`, `:214` | shaders |
| 0019 | "which a compute pass at the start of the frame lists from a light buffer of fixed capacity: those whose range reaches into the Froxel's bounds" | holds: the light buffer made once at `capacities.lights`; the Froxel pass runs after the first culling dispatch (the last audit's note, unchanged); a light is listed unless its sphere lies wholly outside a side or beyond the slice | `renderer.rs:460-469`, `:863-938`; `froxel.wgsl:48-56` | render, report |
| 0019 | "up to 59 lights per pixel; a light past them casts no shadow there and is counted" | holds: 59, asserted to fill 64 bits with the Froxel slice; an entry at 59 or past it, in range, counted once per surface by one `atomicAdd`; the overflow buffer cleared each frame and read back | `shadow_mask_pass.rs:35`; `shadow_mask.wgsl:325-328`, `:368-373`; `renderer.rs:949-954` | render, shaders |
| 0019 | "One compute culling pass composes each Instance's bounds centre camera-relative (ADR 0014; amended 2026-11-04 ...: the vertex shader still composes each vertex)" | holds: `bounds_centre` composes through `camera_relative_position` and the frustum, Hi-Z, LOD and coverage tests take it; `vertex_output` composes each vertex; `vs_thin` composes the centre again by the same function and arguments | `culling.wgsl:133-142`; `main.wgsl:227-233`, `:140-146` | shaders |
| 0019 | "tests the frustum and a two-phase Hi-Z pyramid of minimum reverse-Z depth (ADR 0015)" | holds | `culling.wgsl:190-200`, `:220-267` | shaders |
| 0019 | "writes a compacted index buffer drawn with one `draw_indexed_indirect` per pipeline and Hi-Z phase ... No count draws" | holds: no `multi_draw_indirect` or count draw | `main_pass.rs:315-368`; `renderer.rs:943-948` | render |
| 0019 | "each index packs a Meshlet record in its high bits and a vertex within that Meshlet in its low six" | holds: `pack_index` shifts by `MESHLET_VERTEX_BITS`, log2 of `MESHLET_MAX_VERTICES` (64), kept a power of two by a compile-time assert; the record capacity asserted under 2^26 | `culling_pass.rs:219-221`; `common.wgsl:320-326`; `culling.wgsl:650`; `mod.rs:43` | render, shaders |
| 0019 | "the cooker builds Meshlets of at most 64 vertices into each Mesh Blob" | holds: `MESHLET_MAX_VERTICES` = 64, which the cut and `split` enforce | `src/assets/cooked.rs:45`; `src/cook/mesh.rs:519` | cook, shaders |
| 0019 | "the vertex shader reads the record for its Instance, Material and vertex base, then pulls its vertex from the vertex Arena" | holds: `vertex_output`; `vs_thin` reads the same record, Meshlet and Instance again for the radius, the bounds and the normal | `main.wgsl:217-247`, `:114-124` | shaders |
| 0019 | "Each Primitive draws with its own Material unless its Instance overrides it" | holds: each Meshlet record takes the Meshlet's Material ID unless the Instance's is not `NO_MATERIAL`; the vertex shader reads `record.material_id` | `culling.wgsl:594-600`; `main.wgsl:245` | shaders |
| 0019 | "Culling tests Instances; tests per Meshlet wait for a profile" | holds: every test runs once per slot in `offset_slots`; no Meshlet is tested | `culling.wgsl:384-435`; `culling_pass.rs:1-20` | render, shaders |
| 0019 | "A Mesh is a Meshlet range per LOD of its LOD chain ...: each LOD's Meshlets are cut from its own indices over the Mesh's one vertex range, ... and the Mesh table gives the culling pass each LOD's Meshlet range, triangle range and LOD error, for at most 8 LODs" | holds: cook side; `MeshData` holds the ranges and `lod_errors_metres` as arrays of `MESH_MAX_LODS`, declared from `cooked.rs:42` as 8 | `src/cook/mesh.rs:402-413`; `common.wgsl:191-201`; `mod.rs:291`; `src/assets/cooked.rs:42` | cook, shaders |
| 0019 | "The pass draws, of each Instance, the coarsest LOD whose LOD error times the Instance's scale is under the LOD threshold, one pixel of the target's height, at the distance to the Instance's bounding sphere" | broken for a sheared Instance, `correctness`: the scale is the largest column length and the sphere's radius the Mesh bounds' half-diagonal times it, which bounds a sheared Instance's box only up to √3 short, so there the distance can be overstated: L1-S04, which the verifier confirmed; holds for an Instance whose rotation-scale columns are orthogonal (shaders); render: holds, Rust side, `lod_error_per_metre` one pixel at a metre on the target's height, set at creation and on resize | `culling.wgsl:269-300`; `common.wgsl:207-221`; `renderer.rs:186-201` | render, shaders |
| 0019 | "It reads the camera and the Instance alone, so both phases draw the same LOD and no earlier frame decides it" | holds: `instance_lod` reads the camera uniform, the Instance, its Mesh table entry and `FORCED_LOD`; phase two recomputes it on the same inputs; no earlier frame's data is bound for it | `culling.wgsl:284-300`, `:428-431`; `culling_pass.rs:299-312`; test `renderer.rs:3130` | render, shaders |
| 0019 | "the switch is hard, and no Instance draws two LODs" | holds: one LOD per slot, stored in the slot word's top bits and read back through `slot_lod` | `culling.wgsl:447-449`, `:576`, `:630` | shaders |
| 0019 | "A fragment stage that reads any flat input is never merged"; "Two passes that must agree on a sample's triangle, as the depth passes' normals and the lit pass must, both read a flat input" | holds for the twins: `fs_thin_surface_normals`, `fs_thin_main` and `fs_thin_depth` each read a flat input; the ordinary entry points unchanged (M3.5's row); `coverage_tests.rs` still pins the merging | `main.wgsl:71`, `:181-184`, `:197-202`; `src/render/coverage_tests.rs:648-669` | shaders |
| 0019 | "the shadow mask pass rebuilds each pixel's position from it and reads each sample's normals from the surface normals the depth passes write, so shadow rays need no G-buffer" | holds | `shadow_mask.wgsl:233-243`; `common.wgsl:55-62` | shaders |
| 0019 | "each fragment reads the bits traced from the surface it shades: the nearest sample's, or, where the fragment covers a sample off that sample's plane, the farthest such sample's, so a pixel of three surfaces or more gives its middle ones the farthest's, and ... a nearest-surface fragment that runs under a nearer surface at another sample reads that surface's" | holds, unchanged in `lit`, and for the twin with its kept samples as the coverage; one word for every off-plane sample, traced from the least depth among them | `main.wgsl:387-397`, `:210`; `shadow_mask.wgsl:445-454`, `:478` | shaders |
| 0019 | "the second surfaces ... are traced from a packed list, as tile-based deferred renderers schedule per-sample shading" | holds, unchanged in scope: `atomicAdd` on the count, each 64-entry run's opening entry grows the indirect dispatch by `atomicMax`; the second entry point returns past the count; against the checklist's slot-order item, list entries land in atomic order but each writes only its own pixel's words, the dispatch sizes `atomicMax` and the overflow an `atomicAdd`, so no result depends on which lands first | `shadow_mask.wgsl:455-468`, `:501-513`, `:631-642`; `shadow_mask_pass.rs:360-383` | render, shaders |
| 0019 | "Lights are culled into Froxels ...; they are listed from the light buffer, which holds its lights directional first (amended 2026-11-04) ..., since the lit pass sums lights and the shadow mask gives them its bits in buffer order" | holds: directional lights 0 to `directional_light_count`, then the Froxel's entries in light order; a light's bit is its entry, the mask's word following the Froxel of the sample it was traced from; `froxel_mask_bit`'s midpoint change cannot move a bit | `main.wgsl:398-457`, `:282-296`; `shadow_mask.wgsl:282-293`; `common.wgsl:161-171` | shaders |
| 0019 | "as many as it is sized for at device creation from the scene and pass buffers' GPU memory Budget (amended 2026-11-06), 0.1% of it at 48 bytes a light" | holds: `LIGHTS_PER_MILLE` 1 of `scene_and_pass_buffers`, divided by `size_of::<LightData>()`, three 16-byte rows | `renderer.rs:207`, `:262-265`; `uniforms.rs:19-29` | report |
| 0019 | "past it the point lights farthest from the camera, measured between f64 World positions (ADR 0014), are cut and the rest keep their order, which is the buffer's order under capacity too" | holds: `prepare_lights` keeps every directional light, keeps the `room` nearest point lights by f64 squared distance (ties by index), restores their query order and counts the rest as overflowed; tested | `src/render/prepare.rs:1036`, `:1049-1061`; test `src/render/prepare.rs:2134` | report |
| 0019 | "Transparency is a sorted forward pass after the opaque pass, sharing the Froxel lists, MSAA and the opaque depth" | not applicable: no transparent pass exists at the frozen commit; the frame ends with the lit pass and tonemapping | `renderer.rs:863-938` | report |
| 0019 | "Render pipelines are vertex pipelines" | holds: six vertex pipelines, no mesh stage | `main_pass.rs:115-222` | render |
| 0019 | Reopen: "the opaque main pass takes more than 6 ms of GPU ... as the median over the scene's camera path, timed from the first culling dispatch to the end of the lit pass with shadow rays off" | holds: the span from culling phase one's start to the latest end of the passes up to the lit pass; `gate_microseconds` from the rays-off run's measured frames, judged at its median and p99 against 6000 | `readback.rs:728-745`; `src/bench/mod.rs:793-801`, `:991-996`; `bench/budgets.json:4` | render, tooling |
| 0019 | "Shadow-ray time is reported beside the gate as its own line" | holds: the shadow mask pass is a Timed pass; each path frame's GPU frame with rays on less the same frame's with them off, judged against its Baseline, its own row in the report | `renderer.rs:38-53`; `src/bench/mod.rs:997-1002`; `src/bench/report.rs:91-96`; test `src/bench/mod.rs:2907` | render, tooling |
| 0020 | "Indirect diffuse light comes from ray-traced irradiance probes in camera-centred clipmap cascades ..., updated a subset per frame and shaded from position and normal in ADR 0019's lit pass"; "every light bounces within the probe cascades ... because the probe rays read the same TLAS as the shadow rays" | not applicable: planned; no probe exists at the frozen commit, and the constant ambient term below stands in | `src/environment/mod.rs:3-12` | report |
| 0020 | "Direct light is one hard shadow ray per pixel per light, and, where a directional light's verdict differs from a neighbour's, or where the surface's nearest sample turns from it and another of its samples faces it, one from each other sample that shows the same surface" | holds: one `shadowed` call per light entry of a surface; inside it the one ray runs in at most two segments, the nearest hit, then, past a back face it passes, any hit, through `first_intersection`, the one call site that queries; the edge rays as the row below | `shadow_mask.wgsl:293-366`, `:185-202`, `:99-125` | shaders |
| 0020 | "Direct light is Temporal class (a); indirect light is class (b), and class (c) is used nowhere" | holds: the pass reads this frame's depth, normals, lights, Froxel lists and the culling pass's per-slot words; the static TLAS's lag is the decided one; the main pass reads no screen-space history, `MAIN_PASS_INPUTS` binding this frame's records, mask and mask samples (the checklist's item) | `shadow_mask.wgsl:192`, `:236`, `:418-423`; `bind_groups.rs:285-317` | render, shaders |
| 0020 | "each Mesh rays trace gets a BLAS at load, compacted unless it is loaded to be refitted or rebuilt each frame" | holds: `ALLOW_COMPACTION` unless `MeshUse::Refitted`, whose BLAS is `REFITTED_BLAS_FLAGS`, never queued for compaction, and is what the rebuild form rebuilds | `pools.rs:152-154`, `:614-620`, `:673-682`, `:710-717` | cook |
| 0020 | "it holds exactly the static Instances of the loaded, drawn Chunks, the drawn Far proxies ... and the static spawned Renderables, each in an entry of its own whatever its slot" | holds, Rust side: an entry per written static Instance with a BLAS, mapped by slot; a freed slot, a Moving Instance's write or a Mesh without a BLAS frees the entry; which Far proxies are drawn is the streaming module's | `acceleration_structures.rs:448-506`, `:629-666`; tests `acceleration_structures.rs:1723`, `:1746` | render |
| 0020 | "and leaves out only those written while it is full, which take the entries it frees" | holds: left out by slot, taken lowest slot first as entries free, counted in Frame stats, warned once per run of leaving out | `acceleration_structures.rs:502-525`, `:693-696`; `renderer.rs:723`; test `acceleration_structures.rs:1835` | render |
| 0020 | "One hard shadow ray per pixel is traced toward each directional light against the static TLAS, skipped where the light adds nothing (... amended 2026-11-04 ...: the test reads the interpolated normal ADR 0019's surface normals carry, not the face's)" | holds: a directional light the interpolated normal turns from by more than sin 2° is hidden with no ray, a point light likewise unless another sample of the surface faces it; the amendment's wording now matches the code, so M3.5's drift on "face" no longer stands | `shadow_mask.wgsl:306-309`, `:332-358`, `:248-252` | shaders |
| 0020 | "from an origin offset along the surface's normal" | holds: along the stored triangle normal by the larger of 2^-16 of the largest coordinate in either frame (1 m at least) and 1/32 of a pixel's width, to the light's side where the triangle faces it, inside the surface otherwise | `shadow_mask.wgsl:168-177`, `:36`, `:51` | shaders |
| 0020 | "and composed in the TLAS's frame" | holds: `camera_relative_position(camera.anchor, camera.offset, camera.tlas_anchor, 0, position)` | `shadow_mask.wgsl:161-167`; `common.wgsl:37-46` | shaders |
| 0020 | "and one toward each point light that reaches the pixel, ending at the light" | holds: every point light whose range reaches the surface gets a ray ending at `sqrt(distance_squared)`; with `LightData.unshadowed_range` removed in scope, no point light in range is passed over | `shadow_mask.wgsl:318-331`, `:364`; `common.wgsl:166-171` | shaders |
| 0020 | "A compute pass traces them from the depth target, at each pixel's nearest sample and, where its other samples show another surface, at the farthest of those too, each fragment then reading its own surface's" | holds, unchanged in scope: the greatest of the four depths, first such sample; samples off its triangle's plane by more than a pixel's width, the least depth among them listed and traced into layer 1; each fragment's read as ADR 0019's row | `shadow_mask.wgsl:418-426`, `:434-470`, `:501-520` | shaders |
| 0020 | "at a pixel whose verdict for one of the first seven directional lights differs from a 4-neighbour's among the lights both traced, or whose nearest sample turns from that light while another sample of its nearest surface faces it, toward that light again from each other sample that shows its nearest surface, the fragment shading it by the share of its samples that see it" | holds, unchanged in scope; Rust side `SHADOW_EDGE_LIGHTS` 7, asserted to fit with the samples in 32 bits, the edge list the fourth, indirect dispatch; the lit pass's `directional_hidden_share` takes the share of the fragment's covered samples each edge light's verdicts hide | `shadow_mask.wgsl:546-551`, `:651-702`; `shadow_mask_pass.rs:55`, `:384-401`; `main.wgsl:266-276` | render, shaders, report |
| 0020 | "where the nearest sample turns from a point light while another sample of its nearest surface faces it, the surface's one bit for that light is the verdict of a ray from the first such sample" | holds, unchanged in scope | `shadow_mask.wgsl:332-359` | shaders |
| 0020 | "into a shadow mask the lit pass reads, which holds 59 lights per pixel: a light past them casts no shadow there, and Frame stats count it" | holds: bit = the light's entry, bits 0-31 in `x` and 32-58 in `y`, the Froxel slice from bit 27 of `y`; an entry at 59 or past it, in range, counted once per surface by one `atomicAdd`; `SHADOW_MASK_LIGHTS + log2 FROXEL_SLICES == 64` asserted; the overflow buffer cleared each frame and read back | `shadow_mask_pass.rs:35`; `shadow_mask.wgsl:298-301`, `:368-373`; `renderer.rs:949-954` | render, shaders |
| 0020 | "Moving Instances form the dynamic TLAS, rebuilt each frame in the static TLAS's frame from at most 10,000 of them, those in the lowest slots" (and Consequences: "capped at 10k Instances") | holds: `DYNAMIC_TLAS_CAPACITY` 10,000; prepare lists the 10,000 in the lowest slots and the fill takes them | `acceleration_structures.rs:13-17`, `:850-855`; `prepare.rs:846-862`; test `prepare.rs:1615` | render |
| 0020 | "and a shadow ray the static TLAS lets through is traced against it" | holds: queried unless empty, unless a first hit was asked for and found, or unless the segment misses the dynamic box grown by 2^-16 of the largest coordinate; a nearest-hit query runs only to the static hit; both queries take the same flags and `mask` | `shadow_mask.wgsl:102`, `:106-124` | shaders |
| 0020 | "its build's recording and submit, and the build itself on the GPU, are timed in each frame that builds it" | holds: recording (`prepare_encoder`, `fill`, `record`) and submit are CPU stages; the build reads its bracket, from no earlier than the frame before's last pass, except under the rebuild form | `renderer.rs:705-709`, `:844-857`; `readback.rs:464-480`; `acceleration_structures.rs:889-916` | render |
| 0020 | "The `crowd` Reference scene's figures, which the GPU bends each frame, have no BLAS and cast their shadows through rigid per-bone shadow proxies, Moving Instances in that TLAS, the default form" | holds: `MeshUse::Drawn` is not traced, no index range, no BLAS; the proxies' entries carry the ordinary bit every ray's mask holds, and their slot's drawn LOD error is their Mesh's LOD 0 error, 0, so their back faces always shadow | `pools.rs:115-117`, `:651-653`; `common.wgsl:287`; `src/render/acceleration_structures.rs:990-1008` | cook, shaders |
| 0020 | "The crowd's refit Render switch instead gives each figure a BLAS, refitted to its bent shape each frame through wgpu-hal on Metal ...; the dynamic TLAS's build waits for the refits on the GPU through a Metal shared event; the refits are timed on the GPU from the end of the bend pass to a marker after them" | holds: with a refit or rebuild form and a bend, the head is submitted with the updates before the build is recorded, and the submission is split in the timestamp log | `blas_refit.rs:120-145`, `:470-544`; `renderer.rs:824-843`; `readback.rs:620-630` | render |
| 0020 | "Its rebuild form ... rebuilds each figure's BLAS through wgpu each frame instead, and times neither those rebuilds nor the dynamic TLAS's build, which reads as zero" (amended 2026-11-04) | holds; the amendment answers the last audit's drift Finding on the rebuild form | `blas_refit.rs:116-118`; `src/assets/pools.rs:710-717`; `acceleration_structures.rs:942-961`; `readback.rs:668-677` | render |
| 0020 | "A constant ambient term stands in for indirect light" | holds: `ambient_luminance`, E/π times the base colour darkened by occlusion, from the scene's constant `Environment::ambient_illuminance`, added to every lit fragment's luminance | `src/render/shaders/shading.wgsl:267-269`; `main.wgsl:459`; `src/environment/mod.rs:3-12`; `src/render/prepare.rs:656` | report |
| 0020 | Considered options, "Soft sun shadows: ... a denoiser is class (c)"; Consequences, "Sun penumbrae reopen when area lights arrive or a Golden shows hard sun edges at distance" | not applicable: one hard ray, no jitter | `shadow_mask.wgsl:364` | shaders |
| 0020 | Considered options, soft shadows: "contact shadows stay sharp either way" | holds in kind, with a gap: the back-face pass can drop a contact shadow under an open Mesh drawn at a coarser LOD: L1-S10 | `shadow_mask.wgsl:193-195` | shaders |
| 0020 | "Ray queries are a required capability, not a high-preset path; cascaded shadow maps are dropped" | holds: `floor_features`, which every device requests, holds `EXPERIMENTAL_RAY_QUERY`; no shadow map exists in `src/` | `src/gpu/mod.rs:71`, `:268-280` | report |
| 0020 | "Indirect light converges to 90% of its value within 0.5 s of a step change ...; the probe update takes at most 2 ms of GPU on the dense Reference scene" | not applicable: no probe update exists | — | report |
| 0020 | "No bake is needed for correct lighting. The one cooked lighting term is a per-Chunk sky-visibility field ..." | not applicable: planned, nothing cooks it; the one bake, the Far proxy's, writes albedo and metallic-roughness alone, no lighting term | `src/cook/mod.rs:440-462` | cook |
| 0020 | "Interiors are entered without a load. ... authored Interior volumes mask probes to their inside or outside" | not applicable: planned; no probe or Interior volume exists | — | report |
| 0020 | "The TLAS holds the near field plus merged far proxies (ADR 0026's Far proxies), so a 100k-Instance rebuild ... stays out of the frame" | holds: as the static TLAS's entries row above; the rebuild runs on a worker (the two-TLAS row below) | `acceleration_structures.rs:448-506`, `:629-666` | render |
| 0020 | "A static build keeps the static TLAS's Anchor until the camera's is more than a Chunk's edge from it along an axis, where a build runs though no static Instance changed and re-anchors it at the camera's" (amended 2026-11-16) | holds: `REANCHOR_CELLS` is 256 m / 64 m = 4; a strayed camera makes `start` record a build at the camera's Anchor; exactly 4 cells starts none | `acceleration_structures.rs:195`, `:572-581`, `:596-604`; test `acceleration_structures.rs:1382-1415` | render |
| 0020 | "so the camera adds less than 4.9 mm to a shadow ray's offset once that build is bound, and the frames drawn in between do not move the frame rays trace in" | holds: 5 cells of 64 m times `SHADOW_RAY_OFFSET_SCALE` 2^-16 is 4.88 mm; the offset is 2^-16 times the largest coordinate of `tlas_position` and `position`, under 320 m per axis | `acceleration_structures.rs:190-195`; `shadow_mask.wgsl:36`, `:161-172` | render, shaders |
| 0020 | "the dynamic TLAS, rebuilt each frame in the bound static TLAS's frame, follows in the frame a new one is bound" (amended 2026-11-08) | holds: `upload` reads the static TLAS's Anchor after its update and fills the dynamic TLAS and the uniform's `tlas_anchor` from it; one `camera.tlas_anchor` frames both queries' origin and the dynamic box; the shadow mask pass binds the same index that frame | `renderer.rs:711-742`, `:915-921`; `acceleration_structures.rs:844-887`; `shadow_mask.wgsl:111-118` | render, shaders |
| 0020 | "two static TLASes, each with its own bind group, and a worker thread takes the static Instance changes the spare lacks, records and finishes its build, which the main thread submits (ADR 0016), and the spare is bound from the frame after that submit" | holds: `upload` hands Frame data to `StaticTlas::update` on the main thread; a `Submitted` spare is swapped in at the start of the next `update`; the shadow mask pass binds the bound TLAS's group by index | `acceleration_structures.rs:385-424`, `:531-555`; `shadow_mask_pass.rs:134-143`; test `acceleration_structures.rs:1547-1601` | render |
| 0020 | "a capture waits for the worker so its images never depend on the worker's timing" | holds: with `wait_for_static_tlas`, which every capture path sets, `update` waits for each worker and every change starts a build | `renderer.rs:106-111`; `acceleration_structures.rs:398-403`; `src/capture/mod.rs:171-174`, `:277-280` | render |
| 0020 | "That submit, and each frame's that binds a static TLAS, walks one dependency per distinct BLAS since `vendor/wgpu-core` carries" the upstream fix | holds: the vendored wgpu-core dedups by BLAS tracker index | `vendor/wgpu-core/src/command/ray_tracing.rs:233-248`; `vendor/README.md` | render |
| 0020 | "a static TLAS in an anchor-relative frame (ADR 0014), rebuilt only when its static Instances change (a Chunk streams in or out, a Far proxy swaps with its Chunk, or a static Renderable spawns or despawns ...) or the anchor moves" | holds: `start` records a build only for a lacking change that starts one, a lacking change 120 frames old, or a camera strayed past `REANCHOR_CELLS` | `acceleration_structures.rs:563-581`, `:446-526` | report |
| 0020 | "a Far proxy's entry change alone, as when it crosses the View distance, joins the next build and starts one only once it is 120 frames old" (amended 2026-11-18) | holds: a Far proxy's write or release does not set `starts_build`; the oldest change the bound TLAS lacks starts one at 120 frames | `acceleration_structures.rs:197-199`, `:466-500`, `:566-576`; test `acceleration_structures.rs:1611` | render |
| 0020 | "Shadow and probe rays query both" | holds for shadow rays; no probe ray exists | `shadow_mask.wgsl:99-125` | shaders |
| 0020 | "Every BLAS is built from its Mesh's LOD 0, a refitted Mesh's included, whichever LOD the main pass draws, so a LOD switch changes no BLAS ...; the index Arena, which only BLAS builds and refits read, holds LOD 0's indices alone" (amended 2026-11-14) | cook: holds, `CookedMesh::indices` is LOD 0's, the BLAS's index count and the index Arena range are its length, every LOD's Meshlets load beside it, tested; shaders: the build falls in render; the mask's half holds for a ray from a surface onto its own Instance: a back face met no higher above the surface than 3 times the drawn LOD's error is passed, so a coarser LOD that lies inside LOD 0 does not shadow itself; the rule also passes other Instances' back faces: L1-S10 | `cooked.rs:591-597`; `pools.rs:607-613`, `:1270-1275`; `shadow_mask.wgsl:186-201` | cook, shaders |
| 0020 | "Rough specular reads the irradiance probes; glossy specular reads reflection cubemaps captured at runtime by ray tracing ...; Mirror reflections ... return with water" | not applicable: planned; no probe or cubemap exists, and the ambient term stands in for both lobes | `src/render/shaders/shading.wgsl:261-269` | report |
| 0020 | "contact darkening is missing beyond what a Material's occlusion Channel authors (ADR 0018); a class (a) contact term is added when a Golden shows floating objects" | holds: the occlusion Channel is sampled and darkens the ambient term alone; no contact term exists | `src/render/shaders/shading.wgsl:267-269`; `main.wgsl:322-326` | report |
| 0021 | "Device creation requires a fixed set of wgpu features, downlevel flags and limits, the Capability floor, and fails with every gap listed when the adapter falls short; nothing in the floor has a fallback" | holds: `request_device` runs `check_floor` before requesting any device and returns `BelowCapabilityFloor`, which collects every missing feature, downlevel flag, format feature and limit | `src/gpu/mod.rs:56-66`, `:328-361`, `:376-385` | report |
| 0021 | "The floor holds only what an ADR or shipped code uses ...; a Tier 3 to 6 feature is always optional, probed at runtime, with a path for adapters without it" | holds: the floor's features are requested exactly, and no other feature is used by `src/render/` or `src/gpu/` outside tests | `src/gpu/mod.rs:68-86`, `:269-280` | render |
| 0021 | "`GpuContext` checks the adapter against the floor, then requests exactly its features, limits and experimental opt-in" | holds: requested exactly, limits and the experimental opt-in included | `src/gpu/mod.rs:68-86`, `:72-78`, `:303-323` | render |
| 0021 | "CI's software Vulkan jobs request the floor's device on lavapipe, which shows lavapipe meets it"; "so CI proves it on lavapipe early" | holds: the Goldens run on lavapipe with `PRAXIS_REQUIRE_GOLDENS=1`, under which the test fails rather than skips without an adapter | `.github/workflows/ci.yml:173-187`; `tests/goldens.rs:88-97`, `:101-108` | tooling |
| 0021 | required features: "`TEXTURE_BINDING_ARRAY`, `PARTIALLY_BOUND_BINDING_ARRAY` and `SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING` ..., `TEXTURE_COMPRESSION_BC` ..., `EXPERIMENTAL_RAY_QUERY` ..., `TIMESTAMP_QUERY`" | holds: requested exactly; no other feature used by `src/render/` or `src/gpu/` outside tests; `sample_mask` needs no naga capability | `src/gpu/mod.rs:68-86`, `:269-280` | render |
| 0021 | "4x MSAA on every attachment format the main pass uses ..., with its resolve (`MULTISAMPLE_RESOLVE`) on the HDR colour format ... Those formats are `Depth32Float`, the HDR target's `Rgba16Float` and the surface normals' `Rgba8Unorm` ..., so the floor checks no resolve on it" | holds: `MULTISAMPLED_FORMATS` lists the three, the floor reads each and asks the resolve of the HDR format alone, and `RenderTargets::new` asserts no multisampled target of another format; the last audit's floor Finding is fixed | `targets.rs:47-52`, `:114-118`; `src/gpu/mod.rs:57-66`, `:363-374` | render |
| 0021 | "the window's surface and the capture target take only the tonemapping pass's single-sample output" | holds | `tonemapping_pass.rs:83` | render |
| 0021 | "The `ANISOTROPIC_FILTERING` downlevel flag is required too: ... Material Channels are sampled at up to 16x, the colour Channels at 16x and the data Channels at 4x" | holds: the flag is required (render); the colour Channel sampler is made at 16x and the data Channel sampler at 4x (report) | `src/gpu/mod.rs:284-297`; `src/assets/material.rs:39-40`, `:598-599` | render, report |
| 0021 | "So is `VERTEX_STORAGE`" | holds | `src/gpu/mod.rs:284-297` | render |
| 0021 | "So are `COMPUTE_SHADERS` and `INDIRECT_EXECUTION`: a compute pass builds the frame's draw and the main pass draws it with indirect draws, one per pipeline and Hi-Z phase, each from first instance 0"; `INDIRECT_FIRST_INSTANCE` "left the floor" | holds: both flags required and `INDIRECT_FIRST_INSTANCE` not requested (render); no WGSL writes `first_instance`, which the clear before phase one leaves 0 (culling) | `src/gpu/mod.rs:284-297`; `culling.wgsl:497-526`; `src/render/culling_pass.rs:351` | render, shaders |
| 0021 | "`TIMESTAMP_QUERY_INSIDE_ENCODERS` and `_INSIDE_PASSES` stay optional" | holds: one Timestamp query set, every timestamp a pass's `timestamp_writes`, none inside an encoder or pass | `readback.rs:202-207`, `:582-630`; `culling_pass.rs:438-441`; `main_pass.rs:303-314` | render |
| 0021 | "Ray queries need `ExperimentalFeatures::enabled()`, an `unsafe` call ...; it is the one `unsafe` this ADR adds, and its comment cites ADR 0020" | holds: the other `unsafe` blocks in the module belong to ADR 0020's refit and to the acceleration-structure size query, each with a SAFETY comment | `src/gpu/mod.rs:72-78`; `acceleration_structures.rs:128-130`, `:165-173`; `blas_refit.rs:190-270` | render |
| 0021 | "Required limits are `wgpu::Limits::default()` with `using_minimum_supported_acceleration_structure_values()` ..., raised only where an ADR needs it" | holds: `max_texture_array_layers` is written out at the default 256, raising nothing, and a class's slices are `FLOOR_TEXTURE_ARRAY_LAYERS`; the other per-stage counts stay within wgpu's defaults (storage textures 2 of 4, uniforms 2 of 12, samplers 2 of 16, bind groups 0 to 3 of 4); `max_compute_workgroups_per_dimension` 65,535 is asserted against the device, `append_triangles` dispatched in rows of it (render, cook); the culling module uses 256 invocations a workgroup and 4,128 bytes of workgroup storage against 16,384 (culling) | `src/gpu/mod.rs:303-323`; `material.rs:24`; `culling_pass.rs:77-81`; `culling.wgsl:73-77` | render, shaders, cook |
| 0021 | "`max_buffer_size` at least 2^31 − 1 (ADR 0017 arenas ...)" | holds | `pools.rs:1337-1340`; `src/gpu/mod.rs:313` | render, cook |
| 0021 | "`max_binding_array_elements_per_shader_stage` at least 65,536" | holds: five Channels of 13,107 classes take 65,535 elements in the fragment stage | `material.rs:19-20`; `src/gpu/mod.rs:301`; `bind_groups.rs:49-66` | render, cook |
| 0021 | "`max_acceleration_structures_per_shader_stage` at least 2 (ADR 0020's static and dynamic TLAS ...)" | holds: two `SHADOW_RAY_SCENE` groups, one per static TLAS, each binding two TLASes | `src/gpu/mod.rs:303-323`; `bind_groups.rs:672-686` | render |
| 0021 | "`max_storage_buffers_per_shader_stage` at least 9, since the culling pass's compute stage reads and writes nine" | holds, exactly nine: four in `SCENE_GEOMETRY` and five in `FRAME_DRAW`, the floor requesting 9; the shadow mask pass's compute stage now binds nine too (render's Notes for the verifier) | `bind_groups.rs:20-42`, `:71-102`, `:212-280`; `src/gpu/mod.rs:318-320` | render, shaders |
| 0021 | "`max_storage_buffer_binding_size` stays at the default 128 MiB ...; the vertex shader reads the whole vertex Arena through one storage binding ..., so that Arena stays within it, as a test checks" | holds: the vertex Arena, the Instance buffer and the light buffer are tested against it; the packed indices are 2^25 × 4 bytes, the limit exactly | `src/gpu/mod.rs:821-845`; `culling_pass.rs:93-101`, `:256-260` | render |
| 0021 | "`GpuContext` checks the whole floor before requesting the device and returns one typed error naming each missing feature and downlevel flag, each attachment format without 4x MSAA or, on colour, its resolve, and each limit with its required and adapter values" | broken in the ADR's text: the floor names a missing resolve on the HDR format only, as the ADR's first Consequence says: L1-R06 (render); holds for the rest: `BelowCapabilityFloor` names each missing feature, downlevel flag and format feature, and each limit with its required and adapter values (report) | `docs/adr/0021-capability-floor.md:17`; `src/gpu/mod.rs:363-374`, `:376-425` | render, report |
| 0021 | "the binary logs it with `tracing::error!` and exits non-zero" | holds | `src/bin/praxis.rs:24-29` | tooling |
| 0021 | "`praxis capabilities` runs without the floor, since it diagnoses that failure" | holds; preflight's adapter line also reads the adapter without requesting the device | `src/gpu/mod.rs:122-129`; `src/bin/praxis.rs:121`, `:459-463`; `src/preflight/mod.rs:118-144` | render, tooling |
| 0021 | "Headless contexts, tests and Goldens included, use the same floor" | holds: every capture in the Goldens and Frame stats tests goes through `GpuContext::headless` | `src/gpu/mod.rs:31-93`; `tests/goldens.rs:294-297`; `tests/frame_stats.rs:30` | render, tooling |
| 0022 | "Simulation advances in fixed Ticks of 1/64 s (15.625 ms), decoupled from rendered frames" | holds | `src/time/mod.rs:21`, `:24` | streaming |
| 0022 | "each frame draws a blend of the two newest Tick states"; status: "prepare blends the two in f64" | holds: Instances, both light kinds and bends blend by `blend_weight` | `prepare.rs:215`, `:1004`, `:1017`, `:1099` | render |
| 0022 | status: "`src/time/` runs the fixed schedule once per Tick, at most 4 per frame, with ambiguity detection at `Error`" | holds: the cap, and the build fails at spawn, before any frame, on an unordered conflicting pair | `src/time/mod.rs:27`, `:53-63`, `:69-76`, `:113-125` | streaming |
| 0022 | status: "the Tick counter and blend weight α in `FixedTime`, and the dropped time in frame stats" | holds | `src/time/mod.rs:39-47`; `src/state.rs:276` | streaming |
| 0022 | status: "the helmet spawn and despawn run in it" | holds | `src/state.rs:407-409` | streaming |
| 0022 | status: "World transforms propagate at the end of every Tick" | holds, and once at spawn so frames before the first Tick draw composed transforms | `src/state.rs:131-135`, `:415` | streaming |
| 0022 | status: "an entity the fixed schedule moves keeps its previous Tick's World transform in `PreviousWorldTransform`" (children included) | holds: orrery bodies, flock centres and birds (`town` and `world` share `spawn_flocks`), crowd bones and refit figures carry it; `sway` changes a proxies-form figure's `Bend` only, kept by `record_previous_curvatures` | `src/state.rs:405-406`; `src/scenes/mod.rs:873-878`, `:1276-1301`; `src/scenes/town.rs:834-857` | streaming, cook |
| 0022 | status: "a spawned entity, or one marked `Teleported`, is drawn at its current World transform until the next Tick" | holds; a production system now inserts the marker (`wrap_around_camera`), which the last audit found none did; `drawn` returns the current transform when the previous is `None`; tested | `src/transform/mod.rs:26-29`, `:89-103`; `src/scenes/mod.rs:931`; `src/state.rs:635-658` | streaming, render |
| 0022 | "The frame delta is measured with `Instant` at frame start, before surface acquire ..., and clamped to 250 ms" | holds | `src/lib.rs:220-228`; `src/time/mod.rs:18`, `:33-35` | streaming |
| 0022 | "The fixed loop runs at most 4 Ticks per frame; accumulated time past the cap is dropped, the game slows, and the dropped time is reported in frame stats" | holds; tested at 250 ms and 100 ms | `src/time/mod.rs:53-63`, `:199-223`; `src/state.rs:276` | streaming |
| 0022 | "Simulated time is an integer Tick counter; systems read a per-Tick delta derived from it, never an f32 sum of seconds" | holds: `spin` and `sway` read `(tick + 1) * TICK_SECONDS`; `Time::delta_time` is read by `fly_camera` alone | `src/time/mod.rs:42`; `src/scenes/mod.rs:904-910`, `:1336`; `src/camera/mod.rs:110` | streaming |
| 0022 | "an accumulator resource, a fixed schedule run once per Tick, and the blend weight α as a resource" | holds | `src/time/mod.rs:39-47`, `:80-87` | streaming |
| 0022 | "Everything that changes game state runs in the fixed schedule: gameplay, physics, AI and Lua event reactions" | holds: the fixed chain holds every system that changes game state, `wrap_around_camera` included; that system's camera read breaks the determinism clause below (L1-F41), not this one, as the verifier found; `spin` and `sway` read the Tick count alone | `src/state.rs:403-418`; `src/scenes/mod.rs:902-910`, `:919-934` | streaming, cook |
| 0022 | "Input sampling, the camera, the blend, prepare and presentation-only effects run once per frame" | holds: the frame schedule holds `fly_camera` alone | `src/state.rs:395` | streaming |
| 0022 | "The camera's rotation reads the newest input every frame, outside the fixed loop ...; its position is moved by input every frame too" | holds: mouse motion writes yaw and pitch as it arrives, `fly_camera` moves the position in the frame schedule | `src/lib.rs:120-127`; `src/camera/mod.rs:107-130`; `src/state.rs:395` | streaming |
| 0022 | "Entities the fixed schedule moves keep their previous Tick's World transform; prepare blends previous and current in f64 into Frame data ... Static Instances store nothing extra" | holds: `drawn` blends in f64, slerping the rotation; static Chunk Placements are not entities; prepare blends by `blend_weight` and rewrites every Moving Instance each frame, changed or not | `src/transform/mod.rs:26-49`; `prepare.rs:215`, `:91`, `:805` | streaming, render |
| 0022 | "A spawned entity starts with previous equal to current, and a teleport goes through an explicit marker that resets previous to current" | holds: `wrap_around_camera` inserts `Teleported` on each flock centre it moves, ahead of `reset_teleported` in the chain; tested to drop the bird's previous transform on the jump; a Fast travel moves only the camera, which is not blended | `src/scenes/mod.rs:929-932`, `:1624-1690`; `src/state.rs:410`, `:414` | cook |
| 0022 | "Determinism: the same binary on the same machine, fed the same per-Tick inputs, reaches a bit-identical simulation state whatever the frame timing" | broken for `world`'s flocks: `wrap_around_camera` runs in the fixed schedule and reads the camera, which the frame schedule moves by the wall-clock frame delta: L1-F41 (streaming); broken, known: L1-F41 (cook) | `src/state.rs:410`; `src/scenes/mod.rs:923`, `:929-932`; `src/camera/mod.rs:110` | streaming, cook |
| 0022 | "Inside the fixed schedule, bevy_ecs ambiguity detection is `Error`, no system reads the wall clock, randomness comes from a seeded resource advanced per Tick, and no result depends on hash-map iteration order ..." | holds in letter: no fixed-schedule system calls `Instant` or draws randomness, the chain is single-threaded; streaming runs after the Ticks, its `Instant` reading Fast travel stats only; the scenes' generators choose by position in fixed loops and sort stably; L1-F41 reads wall-clock-driven state, not the clock | `src/state.rs:403-418`; `src/time/mod.rs:69-76`; `io.rs:915-918`; `src/scenes/town.rs:301-304` | streaming, cook |
| 0022 | "ADR 0013's budget splits, judged at p95 once Input latency is measured" | holds: nothing measures Input latency and the Budgets hold no latency entry, so no split is judged yet; the streaming row placed the clause in tooling, which has no row on it | `bench/budgets.json:2-8` | streaming, report |
| 0022 | "The blend reads two simulation states, never screen-space history" | holds | `src/transform/mod.rs:26-49` | streaming |
| 0022 | "Budgets stay judged at 60 Hz, the display the measurements ran on" | holds: `render_run` and `traverse` step at `SIXTY_HERTZ` | `src/bench/mod.rs:848`, `:893-894` | tooling |
| 0022 | "the step model must also stay judder-free at refresh rates that do not divide 64 Hz (120, 144, variable)" | holds in form: the State's tests cycle 144, 60, 30 and 90 Hz frames; judder is a visual check | `src/state.rs:521-539` | streaming |
| 0022 | "A physics engine chosen in M8 steps once per Tick, substepping inside it if it needs to" | holds: no physics engine in the code; deferred to M8 | — | streaming |
| 0023 | "a `DirectionalLight` gives illuminance in lux and a `PointLight` luminous intensity in candela, as KHR_lights_punctual defines them" | holds: prepare carries colour times intensity; the shading reads them as M3.5's rows found; `export-scene` copies them from components documented in lux and candela; every scene's sun is 100,000 lx and its lights in candela; the Cycles reference takes the same units, its point light now always casting a shadow | `prepare.rs:1010`, `:1033`; `main.wgsl:398-460`; `src/cook/export.rs:127-131`; `scripts/cycles-reference/render.py:95-122` | render, shaders, tooling, cook |
| 0023 | "emissive and background give luminance in cd/m²" | holds: emissive shaded as luminance; the background pre-exposed; `export-scene` writes the Environment's cd/m²; every scene's background (10,000, 20,000, 30,000) cd/m² | `main.wgsl:398-460`; `prepare.rs:617`; `src/cook/export.rs:296-322`; `src/scenes/mod.rs:612-633` | render, shaders, cook |
| 0023 | "a Reference scene's ambient term gives illuminance in lux" | holds: 10,000 lx in every scene, shaded as lux and written so by `export-scene`; the Cycles reference gives it as world radiance E/π | `src/scenes/mod.rs:513-531`; `main.wgsl:398-460`; `src/cook/export.rs:296-322`; `scripts/cycles-reference/render.py:137-146` | shaders, tooling, cook |
| 0023 | "Exposure is an EV100, which scales luminance by 1/(1.2 · 2^EV100)" | holds; unchanged in scope; the camera uniform carries `Camera::exposure`; `export-scene` writes the camera's EV100; the Cycles reference applies the same scale | `src/camera/mod.rs:7-16`, `:52-54`; `uniforms.rs:1-10`; `scripts/cycles-reference/render.py:396-403` | streaming, render, tooling, cook |
| 0023 | status: "every Reference scene's lights, ambient and background are in these units, under a sun of 100,000 lx" | holds: `SUN_ILLUMINANCE` 100,000 lx for every scene's sun; `town`'s lamps 2,000,000 cd, the coloured lights 900,000 cd, minification's 3 × 10^7 cd | `src/scenes/mod.rs:582-610`, `:636-670`, `:950-965`; `src/scenes/town.rs:239-245` | cook |
| 0023 | status: "each scene has an EV100, which the lit pass applies before the `Rgba16Float` HDR target stores its colour, clamped to 65504" | holds: EV100 log₂(100,000 / 1.2) = 16.35 for all twelve scenes, `world`'s set by the traversal as `ReferenceScene::camera_at` sets it; one `min(luminance * camera.exposure, LARGEST_HALF)` in `lit` | `src/scenes/mod.rs:612-633`; `src/scenes/world.rs:236`; `main.wgsl:463-464` | shaders, cook |
| 0023 | status: "the tonemapping pass then maps it through GT7 for display" | holds: read here, the display transform calls `gt7_tonemap`; the tonemapping pass alone encodes sRGB, into 8-bit UNORM targets only (`debug_assert!(!target_format.is_srgb())`, no `*Srgb` format outside assets and the cooker); capture writes the `Rgba8Unorm` target's raw bytes; the Cycles comparison splits its display transform into `tonemap` then `srgb_encode` | `src/render/shaders/display.wgsl:21-24`; `tonemapping_pass.rs:52-55`; `tests/conventions.rs:384-399`; `scripts/cycles-reference/compare.py:143-163` | render, tooling, report |
| 0023 | "Shading runs in f32 and pre-exposes colour before fp16 stores it, and every path clamps it to 65504" | holds for both lit entry points, one `min(luminance * camera.exposure, LARGEST_HALF)` in `lit`; the clear is clamped and weighted | `main.wgsl:463-464`; `targets.rs:377-400` | render, shaders |
| 0023 | "Each Reference scene sets its EV100 by hand in M3, which keeps Goldens exact" | holds: one EV100 set in code for all twelve scenes; the traversal sets `world`'s itself | `src/scenes/mod.rs:612-633`; `src/scenes/world.rs:236` | cook |
| 0023 | "Brighter emissives need KHR_materials_emissive_strength, which the cooker does not read yet" | holds: the cooker reads no extension and now warns, naming the file and each extension used | `src/cook/mod.rs:958-961`, `:1290-1323` | cook |
| 0023 | "Point lights fall off by inverse square inside KHR_lights_punctual's range window" | holds: read here, the engine's `point_light_attenuation` is max(min(1 − (d/r)⁴, 1), 0) / d², under an oracle test; the Cycles reference falls off the same | `src/render/shaders/shading.wgsl:235-239`; `src/render/shader_function_tests.rs:514`; `scripts/cycles-reference/render.py:95-122`; `scripts/cycles-reference/calibrate.py:72-74` | tooling, report |
| 0024 | "Every bench run is kept as a Bench record. A record is a file on an orphan branch, `bench-history`" | holds: with no remote tip a publish adds an empty worktree and commits with no parent | `src/bench/history.rs:962-999`, `:1150-1167` | tooling |
| 0024 | "keyed by source hash × adapter × backend × scene; the Harness version is defined under `src/`, so the source hash names it, and the record holds it" (amended 2026-11-04; restated in Consequences) | holds: `HARNESS_VERSION` in `src/bench/provenance.rs`, written into every Provenance; the path is `<dir>/<adapter>-<backend>/<scene>/<source hash>.json`; a re-recorded Baseline's record is keyed by the commit that landed it, not the source the Baseline measured (tooling's Notes) | `src/bench/provenance.rs:22`, `:54`; `src/bench/history.rs:221-229`, `:280-282` | tooling |
| 0024 | "The source hash is one git hash over the paths that can change a measurement" | holds in practice (tooling's Notes: `bench/budgets.json` is compiled in, and the pinned capacity tests force a `src/` change with any Budget change that moves a capacity) | `src/bench/provenance.rs:320-337` | tooling |
| 0024 | "Only clean, release-profile records of `main`'s source taken by a quiet machine form a Series" | holds for published records: the bench's record carries `dirty` (untracked measured files included), `branch`, `profile` and `quiet`, and the history tags on them; broken for the import of re-recorded Baselines: L1-T02 | `src/bench/provenance.rs:143-150`, `:238-282`; `src/bench/history.rs:875-920`; `src/bench/history/import.rs:374-394` | tooling |
| 0024 | "The history observes and never gates: Baselines stay in `bench/baselines/` on `main`, and `verify` and `compare` judge against them as before" | holds: `compare` reads only a Baseline and the Budgets, no history type reaching it; `history rerecord` writes a Baseline's counters only when a session runs it; verify and CI compare counters against `bench/baselines/` | `src/bench/mod.rs:33-40`, `:1184-1189`; `src/bench/history/rerecord.rs:182-244`; `scripts/verify.sh:54-55` | tooling |
| 0024 | "Records publish themselves: CI's lavapipe on every push to `main`" | holds | `.github/workflows/ci.yml:11-28`, `:329-353` | tooling |
| 0024 | "`praxis bench` by default in sessions" | holds | `src/bin/praxis.rs:231`, `:291`, `:596-610` | tooling |
| 0024 | "a nightly sweep on the reference machine that benches `main` only when its source has no quiet record and the machine is quiet" | holds: import, then `history unrecorded` (the scenes without a Series record of `main`'s source hash), then a wait until preflight warns of nothing, then each bench after a wait for the GPU and CPU load; `world` is never benched by the sweep (tooling's Notes) | `scripts/sweep.sh:80-81`, `:109-113`, `:123-128`; `src/bench/history.rs:665-691` | tooling |
| 0024 | "The RTX 2070 SUPER has no sweep ...: its Goldens and Baselines are recorded by hand at each milestone's exit" (amended 2026-11-21) | holds in the scripts: `scripts/sweep-task.xml` is gone and `sweep.sh` installs only as a launchd job on the M4 Pro; two documents still say otherwise: L1-T08, L1-T09 | `scripts/sweep.sh:16-34`; `scripts/sweep.plist:1-29` | tooling |
| 0024 | "A page in `docs/` draws the Series" | holds: `docs/bench/index.html` reads the records from the `bench-history` branch and draws each machine's Series per scene in source order | `docs/bench/index.html:251`, `:416-429` | report |
| 0024 | Considered options: "A local scheduled task runs no code from a pull request" | holds: the clone moves to `origin/main` only; a bisect benches `main`'s first-parent commits between the Verdict's ends | `scripts/sweep.sh:67-72`; `src/bench/history/verdicts.rs:603-625` | tooling |
| 0024 | "The source hash covers `src/`, `assets/`, `Cargo.toml`, `Cargo.lock`, `.cargo/` and `vendor/` (amended 2026-11-09 ...), a list held in one place in code, and a test fails when a new top-level path is classified neither way" | holds: `MEASURED_PATHS` is the ADR's six, and the test lists the other twelve tracked top-level paths at the frozen commit; no path CI skips lies under a measured one | `src/bench/provenance.rs:27-34`, `:410-444`; `.github/workflows/ci.yml:13-28` | tooling |
| 0024 | "A commit whose source hash didn't change inherits the record before it" | holds: `mktree` over `ls-tree` of the measured paths, tested; `unrecorded_scenes` looks up `main`'s source hash, not its commit; pushes to `main` that change only skipped paths run nothing | `src/bench/provenance.rs:320-337`; `src/bench/history.rs:671-690`; `.github/workflows/ci.yml:11-28` | tooling |
| 0024 | "A record path is its key, so a duplicate is a file that already exists" | holds: the record's path is its key's four parts | `src/bench/history.rs:221-229`, `:280-282` | tooling |
| 0024 | "When two records share a key on a Series, or both tagged, the quieter one stays: the one with the lower GPU p95/median; a record on a Series holds its key against a tagged one" (amended 2026-11-04) | holds; M3.5's L1-T04 is settled by the amendment | `src/bench/history.rs:1027-1034`, `:1050-1056`, `:1074-1077` | tooling |
| 0024 | "A record joins a Series only if its tree was clean, its profile release, the machine quiet (preflight warned of none of the GPU load, the CPU load, other praxis processes, the power and the display, its assets warning not counting ...), and its source `main`'s" | holds for published records, the history's tag reading the bench's `provenance.quiet` and the branch README stating the same rule and thresholds; broken for the import: L1-T02; `is_quiet` reads preflight's own readers, thresholds and directions, the assets check feeding none; M3.5's L1-T06 is closed by this amendment | `src/bench/provenance.rs:221-235`; `src/preflight/mod.rs:13-25`, `:304-337`; `src/bench/history/import.rs:374-394` | tooling |
| 0024 | "and p95/median below about 1.06 ...; the p95 being that of each GPU frame over the median of the 31 frames around it, not over the run's" (amended 2026-11-11) | holds: `NEIGHBOUR_FRAMES` 31, 15 a side, fewer at the ends, quiet only when `< 1.06`; a run with no ratio is not quiet | `src/bench/provenance.rs:36-43`, `:171-191`, `:234`; test `src/bench/provenance.rs:631` | tooling |
| 0024 | "and no process outside praxis at or over half a core at the scene's start or end, read per scene ..., where the process listing gives CPU shares, which Windows' does not" (amended 2026-11-08) | holds: `busy_processes` at `BUSY_PROCESS_PERCENT` 50 of a core, at or over, `None` on Windows, read before the scene's first render and after its aliasing captures | `src/bench/mod.rs:786`, `:825`; `src/bench/provenance.rs:193-213`; `src/preflight/mod.rs:512-527` | tooling |
| 0024 | "lavapipe's records are the exception to the quiet test ...: its Series read ... only counters and binary size ..., and build time ...; So a record from CI's configuration joins them whether or not the runner was quiet, and the first record on a Series under a key stays" | holds: `CI_COUNTERS` as lavapipe's configuration, no quiet tag, `publishes_once`; lavapipe rows are never Suspects; the release build is timed and published beside the records | `src/bench/history.rs:177`, `:391-394`, `:895-905`; `.github/workflows/ci.yml:280-284` | tooling |
| 0024 | "Every other record is kept, tagged with its reasons, and drawn only on request" | holds: under `tagged/` with `tags`; the page draws them behind its Tagged records switch | `src/bench/history.rs:183-185`, `:1022-1026`; `docs/bench/index.html:156`, `:433` | tooling, report |
| 0024 | "An Experiment's records are kept under its name and never join a Series" | holds | `src/bench/history.rs:200-219`, `:1019-1026`, `:1079-1080` | tooling |
| 0024 | "The Harness version rises whenever a change alters what a metric means" | holds for the times: four bumps since `8bc34eb1`, at the p99, maximum and refusal counters (10 to 11), the per-slot passes' split (12), Late Far proxies (13) and the far field's fill (14), and the history breaks a line on each; two counter changes kept their version (tooling's Notes) | `src/bench/provenance.rs:18-22`; `src/bench/history.rs:399`, `:1238-1243` | tooling |
| 0024 | "and each record carries its scene's content hash" | holds for published records: `render_frames` returns the Cooked cache's content hash of the Blobs the run loaded, read after its last frame, into the Provenance; broken for re-recorded Baselines' records: L1-T03 | `src/capture/mod.rs:294`, `:409`; `src/bench/mod.rs:820-826`; `src/bench/history/import.rs:323-335` | tooling |
| 0024 | "A chart breaks its line where either changes, so a new definition never reads as a Regression" | tooling: holds for Suspects: `same_definition` starts a line; the chart falls outside tooling; a re-recorded record without a content hash joins either neighbour (L1-T03); report: the page's `sameDefinition`, the Rust function's twin, breaks a line on a Harness version or a content hash change, a missing hash joining either neighbour | `src/bench/history.rs:395-404`, `:1235-1243`; `docs/bench/index.html:402-404`, `:416-429` | tooling, report |
| 0024 | "The commits that touched `bench/baselines/` before the history existed, 64 on `main`'s first-parent line before `HISTORY_STARTED` (amended 2026-11-04), are imported as records, each with the Harness version in force at its commit" | holds: `git log --first-parent --format=%H 3c6b9d92 -- bench/baselines` lists 64, recounted; the version steps at each of the nine breaks and the import checks it ends at 10; M3.5's L1-T03 is settled by the amendment | `src/bench/history/import.rs:24`, `:30-67`, `:156-193` | tooling |
| 0024 | "Publishing never blocks work. A failed publish warns and the bench still succeeds" | holds; the night logs a failed `publish-night` and goes on | `src/bin/praxis.rs:596-610`; `scripts/sweep.sh:191` | tooling |
| 0024 | "`verify` never publishes" | holds | `scripts/verify.sh:54` | tooling |
| 0024 | "Baselines are still chosen by hand, and a re-recorded Baseline's record is marked as one" | holds: commits after `HISTORY_STARTED` give records with `provenance.source` `baseline-rerecorded`; a quieter record that holds the key takes `rerecorded_baseline`; either starts the line; tested; M3.5's L1-T02 fixed (a tagged one's mark can be lost: tooling's Notes) | `src/bench/history/import.rs:72-80`, `:194-220`; `src/bench/history.rs:313-321`, `:1042-1060` | tooling |
| 0024 | "A timing record far from its Series' recent median is a suspect" | holds: a window of 8, from 3, ± max(4 MAD, smallest shift), CPU frame 3 %, GPU frame 1.25 %, as the glossary's Suspect | `src/bench/history/suspects.rs:7-15`, `:47-101`; `src/bench/history.rs:429-453` | tooling |
| 0024 | "The sweep re-benches it, bisects the commits behind it" | holds: two interleaved runs per end, good first, `holds` at max(shift/2, smallest shift), bisect by source hash; `--no-publish` only for a commit descended from `3c6b9d92`, so M3.5's L1-T08 is fixed | `src/bench/history/verdicts.rs:216-308`, `:360-432`; `src/bench/history/night.rs:56-91`; `scripts/sweep.sh:151-190` | tooling |
| 0024 | "and files a confirmed Regression that no re-recorded Baseline covers as one `needs-triage` issue" | holds: `covered_by_baseline` from `remeasured` on the adapter and backend's Baseline at the pinned commit and its parent; one issue per scene and commit, an existing title reused, label `needs-triage`; run every night | `src/bench/history/verdicts.rs:394-427`, `:452-554`, `:732-747`; `scripts/sweep.sh:198-200` | tooling |
| 0024 | "A bad publish is undone with `git revert` on `bench-history`" | not applicable (process); every write is one commit on the tip | `src/bench/history.rs:712-751`, `:1150-1167` | tooling |
| 0024 | "The branch holds only data and a README, is never merged, and CI writes it only from pushes to `main`, never from a pull request" | holds: every publish, import and Verdict rewrites `README.md` beside the data; `github.event_name == 'push'` on a `main`-only push trigger; the recording workflow holds `contents: read` | `src/bench/history.rs:1086`; `src/bench/history/verdicts.rs:644`; `.github/workflows/ci.yml:3-12`, `:331` | tooling |
| 0025 | "The repository is public on GitHub and everything in it is published under the MIT licence (`LICENSE`, `Cargo.toml`): `main`, every PR's branch, `bench-history` and `pr-images` alike" | holds | `LICENSE:1`; `Cargo.toml:5` | tooling |
| 0025 | "Anything the owner would not publish, on any branch, issue or PR: a credential, a token, private data" | holds for the workflows: no `secrets.` is read anywhere; the one token, `github.token`, reaches `gh` through the environment only | `.github/workflows/pr-lint.yml:43-61` | tooling |
| 0025 | "A Bench record and a PR's images are published as a commit is" | holds: every record is pushed to the public `bench-history`, and in CI only `publish-bench` writes, only on a push to `main`; what a record names is a note for the owner (tooling's *Notes*) | `src/bench/history.rs:726-735`; `.github/workflows/ci.yml:329-353` | tooling |
| 0025 | "Every file under `assets/` has a manifest entry and a credit, which `cargo test` checks" | holds, with planted cases | `tests/asset_manifest.rs:97-106`, `:194-196`, `:209-214` | tooling |
| 0025 | "a larger source is fetched from its publisher into a git-ignored cache" | holds: the helmet's six files are `fetched: true` and `/assets/SciFiHelmet/` is ignored; `cook_scene` fetches a scene's absent fetched sources first, and `fetch.rs` is unchanged since M3.5 | `assets/manifest.json:3-66`; `.gitignore:17`; `src/cook/mod.rs:609` | tooling, cook |
| 0025 | "a file under a noncommercial or restrictive term is dev-only: never committed, and no committed Golden is rendered from it" | holds today: every manifest entry is `CC0-1.0`, none `dev_only`; the test enforces only `-NC-` → `dev_only` (tooling's *Notes*) | `tests/asset_manifest.rs:190-193` | tooling |
| 0025 | "Code copied from a noncommercial or unclear licence ... Adapted code cites its source and licence" | holds where read: the GT7 port in the Cycles reference names its source, key and the MIT notice; `world`'s and `town`'s content is generated from code, and its one adapted source, SplitMix64's output function, cites its paper and Vigna's public-domain `splitmix64.c` | `scripts/cycles-reference/compare.py:36-60`; `src/scenes/town.rs:289-298`; `src/scenes/world.rs:356-387` | tooling, cook |
| 0025 | "a vendored crate keeps its licence files (`vendor/README.md`)" | holds: `LICENSE.MIT` and `LICENSE.APACHE` beside `vendor/wgpu-core` | `vendor/README.md:22-23` | tooling |
| 0025 | "every dependency's licence expression is on the allowlist in `tests/dependency_licences.rs`" | holds: `cargo metadata --locked` over every package; the one expression added since, "BSD-3-Clause AND BSD-2-Clause", is permissive | `tests/dependency_licences.rs:8-34`, `:37-55`, `:74-89` | tooling |
| 0025 | "Self-hosted Actions runners on the owner's machines ... CI runs on GitHub's hosted runners, without a GPU, and local scheduled tasks bench the real ones" | holds for the runners: every `runs-on` is `ubuntu-24.04`, `ubuntu-latest` or `windows-2025`, the new workflow included, and the sweep is a launchd job; the plural "scheduled tasks" is L1-T09 | `.github/workflows/ci.yml:60`; `.github/workflows/lavapipe-files.yml:23`, `:69`; `.github/workflows/pr-lint.yml:26` | tooling |
| 0026 | "Every Chunk outside the Load radius and within the View distance draws its Far proxy, one Instance, in place of its Placements" | holds, streaming side; the draw as the *Which draws* rows | `chunks.rs:132-151`; `io.rs:1005-1022`, `:1210-1215` | streaming |
| 0026 | "Under it the Instance slots, the static TLAS and the flat cull keep M4's capacities" | render: holds on its purpose: the densest 2 km view fits the capacities, tested; the slots fell from 517,376 to 488,448 when Thin Primitives landed, which L1-R03 records; shaders: holds for the culling pass, one `INSTANCE_CAPACITY` from `RenderCapacities`, one flat pass over the slots | `renderer.rs:1088-1098`; `src/scenes/world.rs:858`, `:879`; `mod.rs:297` | render, shaders |
| 0026 | "What it holds. A first Primitive of merged, simplified geometry under one Material baked for it, windows included, and a second Primitive of the Chunk's ground tiles, unsimplified, under the ground's own Material" | holds: `cook` splits the Chunk's Placements by `placement.mesh == ground`, merges and simplifies the rest into the shell under the baked Material, and appends each ground tile unsimplified under its Mesh's own Material, refusing a tile whose Placement overrides it; the scene hands the bake each building whole with its windows and the Chunk's ground tiles | `far_proxy.rs:225-226`, `:281-302`, `:316-319`; `src/scenes/mod.rs:462-476` | cook |
| 0026 | "Only a scene with a Far field cooks proxies; it names its ground's Mesh" | holds: a launch maps Far proxy IDs only for a scene whose `far_field_ground` is `Some`, `cook_sources` skips the rest, and `far_proxy_mesh` refuses a scene without one; `far_field_ground` names `world`'s `ground` and no other scene's | `launch.rs:78-84`; `mod.rs:677-679`; `src/scenes/mod.rs:371-376`, `:573-580` | cook |
| 0026 | "The cooker merges the Chunk's static Placements and drops a Placement when every vertex of it lies within one pixel of one other Placement's surface, that Placement the Chunk's own or one of its eight neighbours', since a Placement belongs to the Chunk that holds its pivot" | holds: `within_another` tests each of the Chunk's own Placements against each single other surface, every corner within `reach_metres` of some triangle of it, the neighbours' Placements joining as surfaces, never as candidates; `chunk_content` keeps exactly the Placements whose pivot `chunk_holding` puts in the Chunk | `far_proxy.rs:123-159`, `:240-262`; `src/scenes/mod.rs:449-460` | cook |
| 0026 | "One pixel is its size at the Load radius at 1440p and a 45° field of view, 14.7 cm at 256 m" | holds: 256 × 2 × tan(22.5°) / 1440 = 0.147276 m, `FIELD_OF_VIEW` = π/4, `world`'s Load radius 256 m | `far_proxy.rs:24`, `:28-30`; `src/camera/mod.rs:90`; `src/scenes/mod.rs:569` | cook |
| 0026 | "It welds what is left on position and simplifies it by edge collapse to one pixel of error, with no triangle count asked; the error is the simplifier's own measure" | holds: `generate_vertex_remap` on positions alone, then `meshopt::simplify` with target count 0, the pixel as target error and `ErrorAbsolute` | `far_proxy.rs:161-178`, `:267-274` | cook |
| 0026 | "It refuses a proxy of more than 5,832 triangles outside its ground: the walls and roofs of `world`'s densest Chunk, 324 buildings of 18" | holds: `TRIANGLE_CAP` = 5832, checked on the simplified shell before the ground joins, the message naming the count; on the scene side walls are 8 triangles and a roof 10, and the densest Chunk holds 324 lots by the cook reader's recomputation | `far_proxy.rs:21`, `:275-279`; `src/scenes/world.rs:408-432`, `:583-593` | cook |
| 0026 | "Its Material. Two Channels, albedo and metallic-roughness, a texel two pixels at the Load radius, 29.5 cm, in one atlas: one rectangle a planar face ..., with 2 texels of gutter a side on a grid of 4, dilated, so no chart bleeds into another down to mip 2" (amended 2026-11-18) | holds: `TEXEL_PIXELS` 2 (0.29455 m), `GUTTER_TEXELS` 4 / 2 = 2, `GRID_TEXELS` 8 / 2 = 4; each chart's padded block starts on the grid, so a mip-2 texel never spans two blocks (cook's Notes) | `bake.rs:15-20`, `:93-95`, `:197-223`, `:304-307` | cook |
| 0026 | "The Chunk's Placements and its neighbours', those dropped from the geometry included, are rasterised into each chart along its normal with a depth test, the outermost surface winning, at 4 × 4 samples a texel, on the CPU" | holds: `Transfer::surfaces` is every surface, dropped ones included; `tile` keeps the greatest height along the chart's normal per sample, `SAMPLES_PER_SIDE` 4 | `far_proxy.rs:346-353`; `bake.rs:16`, `:384-452` | cook |
| 0026 | "Its Blobs. A Mesh Blob per Chunk and, where its first Primitive has a triangle, a Material Blob and two Texture Blobs, each under an Asset ID made of the scene and the Chunk's coordinates, cooked together by `praxis cook <scene>` and by a dev launch that loads any of them while one is missing or stale" | holds: `cook_far_proxy` writes the two atlases and the Material only when the shell charted, then the Mesh, all under one Cook key; a launch maps all four IDs to one `FarProxy` entry; the Blobs are named `far proxy {x} {z}` under the scene | `src/cook/mod.rs:425-468`, `:676-694`; `launch.rs:184-201`; `src/scenes/mod.rs:352-369` | cook |
| 0026 | "They share one Cook key, which covers the Chunk's and its neighbours' Cook keys, the Cook keys of the Meshes they place, of the Materials those draw with and of those Materials' albedo and metallic-roughness Textures, and the format and settings of each Blob" | holds: the key hashes the Mesh, Material and Texture formats and settings (with `far_proxy::settings`, every bake constant and the albedo's BC7 profile), the ground's ID, each of the nine Chunks' keys or a 0 byte, each placed Mesh's, each Material's and each of `BAKED_CHANNELS`' Texture keys; a test changes a drawn and an undrawn Material's key | `src/cook/mod.rs:289-369`; `src/cook/far_proxy.rs:32-38`; `bake.rs:23-27`; test `src/cook/mod.rs:1188-1219` | cook |
| 0026 | "A proxy is fresh when its Mesh is in the Cooked cache under that key and, unless that Mesh names no Material, the three baked Blobs are too" | holds | `src/cook/mod.rs:182-193` | cook |
| 0026 | "A proxy whose sources name a Material or a Texture the cook did not read is kept as it is, as a Chunk is" | holds: `kept` is set by an unread Material, an unread Texture or a kept Chunk among the nine, and both cooks then write nothing | `mod.rs:320`, `:344-346`, `:355-358`; `launch.rs:188-190` | cook |
| 0026 | "The swap. A Far proxy shows exactly when its Chunk is not drawn, so the swap is at the Load radius, a whole Chunk at once, with no fade" | holds | `io.rs:1006-1010`, `:1020` | streaming |
| 0026 | "A Late Chunk keeps its Far proxy until it loads: it shows coarse, never as a hole" | holds: only placed, unheld Chunks hold their proxy | `io.rs:1008-1009` | streaming |
| 0026 | "A Far proxy entering at the View distance appears the same way, whole and at once" | holds: one Placement, placed whole | `chunks.rs:137-150`; `io.rs:1378-1388` | streaming |
| 0026 | "Which are Resident. A Far proxy is requested when its Chunk comes within the View distance, and ahead of that while the camera closes on it ..., and released past the View distance plus one Chunk's edge" | holds; tested | `mod.rs:149-157`, `:211-251`, `:296-298`; test `io.rs:1921` | streaming |
| 0026 | "It stays Resident while its Chunk is drawn, so leaving a Chunk never waits on a read" | holds: the View distance is never less than the Load radius, and a proxy unloads only past the View distance plus an edge | `mod.rs:212-215`; `io.rs:783-784` | streaming |
| 0026 | "Past the View distance. Nothing is drawn but flat ground reaching the horizon, one Resident quad" | holds: `spawn_world` loads the ground tile through `untraced_mesh` and places it as a `FarGround` Chunk of one Placement with `held: false` | `src/scenes/world.rs:43-51`, `:244-264`; `src/assets/loader.rs:34-36` | cook |
| 0026 | "The LOD chain. A Far proxy is a Mesh and the culling pass selects its LOD as any Instance's" | holds: neither the culling pass's Rust side nor `culling.wgsl` has a Far proxy case | `culling_pass.rs:1-20`; `culling.wgsl:284-300` | render, shaders |
| 0026 | "Its chain is cut on the welded positions, each coarser LOD simplified from LOD 0 to twice the error limit of the one before, and ends at eight LODs or at one that keeps more than 85% of the one before" | holds: each LOD is `simplified(&positions, &finest, pixel × 2^d)` for d = 1 to 7, stopping at an empty LOD or one past `LOD_KEPT_INDICES_CEILING` (0.85) of the last kept LOD's indices | `far_proxy.rs:329-344`; `mesh.rs:31`; `src/assets/cooked.rs:42` | cook |
| 0026 | "so each LOD's faces get flat normals and their own vertices, shared with a finer LOD where they are equal" | holds: `faceted` gives each LOD a vertex per position and quantised face normal; `push_lod` reuses a finer LOD's vertex of identical bytes in the same Primitive | `far_proxy.rs:180-206`, `:338-342`; `mesh.rs:238-289` | cook |
| 0026 | "The pivot. A proxy's pivot is its Chunk's centre on the ground, and the cooker refuses a proxy with a vertex more than a Chunk's edge from it" | holds: the pivot is (128, 0, 128) m from the Chunk's origin; the bound is checked on LOD 0's vertices, which every coarser LOD indexes a subset of | `far_proxy.rs:220`, `:320-328` | cook |
| 0026 | "f32 resolves 0.015 mm between 128 and 256 m ..., against a quarter of a pixel of 37 mm at the Load radius" | holds, arithmetic: 2^7 × 2^-23 = 2^-16 m = 0.0153 mm; a quarter pixel at 256 m is 36.8 mm | `far_proxy.rs:28-30` | shaders, cook |
| 0026 | "The ground is no surface to drop against" | holds: ground tiles are partitioned out before the surfaces are built, and neighbours' ground tiles are filtered out | `far_proxy.rs:225-232`, `:242` | cook |
| 0026 | "Dropped against the dropped. A Placement is dropped against any other Placement, one that is itself dropped included" | holds: `within_another` never removes a dropped surface from the candidates' targets | `far_proxy.rs:151-158` | cook |
| 0026 | "Open land. A Chunk with nothing but ground cooks a proxy whose first Primitive is empty" | holds: no parts give no corners, an empty simplification, no atlas and no Material; a test asserts it on `world`'s Chunk (0, 0) | `far_proxy.rs:238-240`, `:313-319`; `bake.rs:238-240`; `launch.rs:458-459` | cook |
| 0026 | "A chart. A planar face is the triangles of the first Primitive that share vertices, which carry a position and a face normal" | holds: `faces` joins triangles by shared vertex indices, and `faceted` keys each vertex on its position's bits and its normal in steps of 1/512 | `bake.rs:168-195`; `far_proxy.rs:189-196` | cook |
| 0026 | "Its chart is the least-area rectangle about it along one of its edges, a texel two pixels at the Load radius, its rows no longer than its columns" | holds: `Chart::fit` takes the least-area rectangle over every edge of the face's triangles, swapping axes so that rows ≤ columns | `bake.rs:108-142` | cook |
| 0026 | "Charts are packed on shelves, tallest first, each with 2 texels of gutter a side on a grid of 4, into the smallest square atlas of a power-of-two side that holds them; a proxy whose charts pass 8,192 texels a side fails its cook" | holds: a stable sort by rows, shelves, sides doubling from 4 to `MAX_TEXTURE_SIDE` (8192), then an error; "the smallest that holds them" is the smallest this shelf packer fills | `bake.rs:197-223`; `src/assets/cooked.rs:36` | cook |
| 0026 | "Coarser LODs. Each face of a coarser LOD takes the chart nearest it among those facing within 45° of it, or among all of them where none does, its UVs held inside that chart" | holds: `LIKE_NORMAL_COSINE` = 1/√2, inclusive; nearest by the summed distance of the face's vertices to the chart's rectangle; UVs clamped into it | `bake.rs:21`, `:157-165`, `:278-302` | cook |
| 0026 | "What a chart reads. A source is a triangle of a Placement of the Chunk or of a neighbour that reaches it, one dropped from the geometry included, facing along the chart's normal within two pixels of its plane ... The outermost wins, at 4 × 4 samples a texel" | holds: a positive projected area is a triangle facing along the normal; `REACH_PIXELS` 2; the per-sample height must lie within the reach and above the sample's last | `bake.rs:17`, `:404-417`, `:437-448` | cook |
| 0026 | "A Placement the runtime does not draw, one without a Material whose Mesh has a Primitive with triangles that names none, is no source" | holds for the bake: `placed` gives such a Placement's triangles no Material and `tile` skips them; its geometry still joins the shell (cook's Notes) | `far_proxy.rs:65-69`; `bake.rs:401-403` | cook |
| 0026 | "A sample reads its source's Material as its factors times its albedo and metallic-roughness Textures, at the finest mip whose texels are no smaller than the sample, as the Texture cook filters it, so a roughness its normal Texture widens is read widened" | holds: `SourceMaterial::shade` multiplies the linear factors by the bilinear sample; `Mips::level_of` is ceil(log2(UV per sample × side)); `TextureSource::mips` runs the Texture cook's own `filtered_levels`, Toksvig included | `bake.rs:69-90`, `:418-419`; `texture.rs:222-273`, `:297-313` | cook |
| 0026 | "A texel is the mean of its covered samples in linear light; one no source covers, a gutter's included, is dilated from its neighbours. A chart no source covers at all, and the texels outside every chart, hold the mean of the baked texels as stored, which mips past mip 2 blend in" | holds: `mean` over covered samples in linear light; a breadth-first dilation over eight neighbours inside the padded block; the fill is the integer mean of every tile's stored sRGB bytes (truncated, cook's Notes) | `bake.rs:52-59`, `:454-498`, `:355-364` | cook |
| 0026 | "A source Texture's repeat is baked. Where a face ends within half a texel ... of the repeat's edge, its last texels blend in the far edge, as the source's own sampler does" | holds: `Mips::sample` filters bilinearly with `rem_euclid` wrapping | `texture.rs:315-336`; `bake.rs:76-79` | cook |
| 0026 | "The baked Material. Its factors are 1 for base colour, roughness and metallic, and it names no normal, occlusion or emissive Texture. Its Textures cook through the Texture cook, BC7 and BC5 with every mip ... The albedo atlas encodes at the BC7 encoder's `ultra_fast` profile where the Texture cook's is `basic`" | holds: `CookedMaterial` of unit factors naming only the two atlases, each cooked through `TextureSource::cook` with `with_fastest_bc7`, which changes only the BC7 (albedo) encode; the launch test asserts BC7 sRGB and BC5 with every mip; in the lit pass a Channel without a Texture reads Layer 0's neutral texel | `src/cook/mod.rs:440-462`; `texture.rs:106-113`; `launch.rs:438-446`; `main.wgsl:309-335` | cook, shaders |
| 0026 | "No area, no face. A triangle without area has no normal to chart by and is left out of the proxy" | holds | `far_proxy.rs:185-188`; `bake.rs:631-636` | cook |
| 0026 | "Which draws. The Resident Far proxies are in the World beside the loaded Chunks, each a Chunk of one Placement that takes one Instance slot" | holds: `ResidentFarProxies` and `FarGround` take ranges after the loaded Chunks' | `chunks.rs:15-19`; `mod.rs:284-288`; `prepare.rs:630-642`, `:699` | streaming, render |
| 0026 | "A Far proxy is held, its slot an Instance with no Mesh, while it is past the View distance or its Chunk is drawn: loaded and within the Load radius" | holds: held on that rule in streaming and written an Instance with no Mesh by prepare; a `NO_MESH` slot is neither drawn nor tested, written `NOT_DRAWN` with a LOD error of 0 | `io.rs:1210-1215`; `prepare.rs:725-728`; `culling.wgsl:395`, `:446-457` | streaming, render, shaders |
| 0026 | "So the swap falls in the frame the Chunk's Placements are placed, once their BLASes are ready, and in the frame the Chunk is held or unloaded" | holds: the Far proxies' held set is taken from the Chunks kept that same frame | `io.rs:999-1010` | streaming |
| 0026 | "Its shadows. A Far proxy's Mesh builds a BLAS from its LOD 0 at load" | holds, pool side: `MeshUse::FarProxy` is traced, its BLAS built from LOD 0's indices and compacted; a receiver on a Far proxy drawn at a coarser LOD of its own chain passes its own LOD 0's back faces by the same rule as any Instance | `pools.rs:124-126`, `:137-139`, `:607-613`; `shadow_mask.wgsl:186-201` | cook, shaders |
| 0026 | "and the proxy is placed once that BLAS is ready, as a Chunk is" | holds: streaming places a `MeshUse::FarProxy` once `asset_blas_ready` answers for it; the TLAS asserts it in debug builds only (render's Notes) | `io.rs:1508-1511`; `pools.rs:759-763`; `acceleration_structures.rs:454-458` | streaming, cook, render |
| 0026 | "While drawn it holds one static TLAS entry under the mask bit an ordinary Instance has, so it casts and takes the sun's shadows" | holds: a Far proxy's Mesh has no Thin Primitive, so its entry takes `ORDINARY_MASK`; bit 0 is in every ray's mask, the sun's and each point light's | `acceleration_structures.rs:990-994`; `src/assets/pools.rs:862-869`; `common.wgsl:287` | render, shaders |
| 0026 | "and a swap is a static change: the static TLAS takes both sides of it from its own next build, and the draw does not wait (ADR 0020)" | holds: Chunk, Far proxy and far ground writes are never Moving; the Placements written or released in the swap's frame start the build that carries the Far proxy's change too; the draw reads the Instance buffer, not the TLAS | `prepare.rs:727`, `:969`; `acceleration_structures.rs:466-500` | render |
| 0026 | "A Far proxy whose entry changes with no other static change in its frame starts no build for it ... The change joins the next build a Chunk or a re-anchor starts, and starts one itself once it is 120 frames old" | holds: a Far proxy's write or release does not set `starts_build`; the oldest change the bound TLAS lacks starts one at 120 frames | `acceleration_structures.rs:197-199`, `:466-500`, `:566-576`; test `acceleration_structures.rs:1611` | render |
| 0026 | "a capture, which waits for the static TLAS's worker, builds each at once" | holds: under `wait_for_static_tlas` every change starts a build | `acceleration_structures.rs:398-405` | render |
| 0026 | "Its mips. Mip priority reads the drawn Far proxies beside the drawn Chunks' Placements" | holds | `io.rs:1109-1115` | streaming |
| 0026 | "A Far proxy held past the View distance has its mips requested too, never late" (amended 2026-11-20) | holds | `io.rs:1105`; `mips.rs:320-323` | streaming |
| 0026 | "one held while its Chunk is drawn needs none" | holds (streaming's Notes on what eviction then does to it) | `io.rs:1106`; `mips.rs:167-169` | streaming |
| 0026 | "A Far proxy is read with its atlas's mips down to the one it needs from where the camera was when it was requested ..., and one within the Load radius holds the mips it needs where it takes over from its Chunk" (amended 2026-11-20) | holds at the read, reader and pool side included; tested | `io.rs:508-526`, `:1530`; `cooked.rs:389-397`; `material.rs:370-372` | streaming, cook |
| 0026 | "Open land. A Primitive without triangles needs no Material, so the proxy of a Chunk with nothing but ground draws its ground" | holds: `needs_material_handle` counts only Primitives with triangles, tested; the empty shell names `None` | `pools.rs:511-513`; test `loader.rs:359-364`; `far_proxy.rs:315` | cook |
| 0026 | "The ground past the View distance. One of the ground's tiles, scaled to reach 1,024 km from the world's middle and laid 0.5 m under the tiles, placed by the scene as a Chunk of one Placement that is never held, and traced by no ray" | holds: scaled by 16,000 so its 64 m half-side reaches 1,024,000 m, 0.5 m under the origin, a `FarGround` Chunk of one Placement with `held: false`; a `MeshUse::Drawn` Mesh has no BLAS, so its write frees any entry | `prepare.rs:636-642`; `acceleration_structures.rs:452-476`; `src/scenes/world.rs:43-51`, `:244-264` | render, cook |
| 0026 | "f32 places a corner to 62.5 mm"; "From the camera's 60 m it ends a tenth of a pixel under the horizon at 1440p" | holds: 2^19 ≤ 1,024,000 < 2^20, an f32 spacing of 2^-4 m; atan(60 / 1,024,000) against 2 tan(22.5°) / 1440 a pixel is 0.102 pixel | `src/scenes/world.rs:43-46`, `:58` | cook |
| 0026 | "ADR 0013's 2 km View distance is drawn this way, and the "merged far proxies" of ADR 0020 are these" | holds: `world` 2,000 m and 256 m; every other scene's View distance is its Load radius | `src/scenes/mod.rs:553-580` | cook |
| 0026 | "ADR 0014's 128 m pivot bound exempts a Far proxy's Mesh, as *The pivot* above has it" | holds: `far_proxy::cook` builds through `mesh::lod_0` and `push_lod`, never `mesh::cook`, whose 128 m check it bypasses, and checks a Chunk's edge instead | `far_proxy.rs:316`, `:320-328` | cook |
| 0026 | "The Material table counts Far proxies' Materials apart, and a frame's loads are capped by count (ADR 0017)" | holds: the first Primitive's Material of a Far proxy's Mesh takes an entry of the Far proxies' count, the ground's an ordinary one; `LOADS_PER_FRAME` 3 counts Chunks and Far proxies together, and `fits` refuses a load past it unless settling | `loader.rs:151-153`, `:164-172`; `src/streaming/io.rs:762-765`, `:1313-1322` | cook, report |
| 0026 | "A streamed scene whose Chunk keeps more triangles than the cap at one pixel fails its cook, naming the count" | holds: `praxis cook` returns the error; a launch logs it per request | `far_proxy.rs:275-279`; `mod.rs:691`; `io.rs:1524-1529` | cook |
| 0026 | "A LOD error is what the simplifier reports" | holds: the reported error of each LOD's simplification from LOD 0 is stored as its error, in metres | `far_proxy.rs:331-332`, `:342` | cook |
| 0026 | "A dev launch that loads a Far proxy hashes the albedo and metallic-roughness Textures its sources draw with, loaded or not, to find its Cook key, and bakes the proxy where a Blob of it is missing or stale" | holds | `launch.rs:184-200`; `mod.rs:341-361`, `:373-380` | cook |
| 0027 | "Implemented for Thin Primitives ... and small Instances" | holds for the culling pass's half (its rows below) | `culling.wgsl:302-344`, `:398-427` | shaders |
| 0027 | "The carrier. Partial coverage is a `sample_mask` output of the fragment stage" | holds: the two depth twins return `@builtin(sample_mask)`, `fs_thin_depth` as its only output and `fs_thin_surface_normals` beside the normals | `main.wgsl:186-189`, `:191-202` | shaders |
| 0027 | "No pipeline of the main pass enables alpha-to-coverage" | holds: one `MultisampleState`, count 4, `alpha_to_coverage_enabled: false`, shared by all six pipelines; no WGSL reads alpha as coverage, `surface.opacity` written as the colour's alpha alone | `main_pass.rs:110-114`, `:216-222`; `main.wgsl:464` | render, shaders |
| 0027 | "Who writes it. The depth passes write the mask, so only the kept samples take the fragment's depth" | holds: the pre-pass twin runs `fs_thin_surface_normals` with shadow rays on, `fs_thin_depth` with them off, and the lit pass's own depth draw runs `fs_thin_depth`, each writing `thin_fragment_samples`, which the raster ANDs with coverage; the shadow mask traces the nearest sample a kept sample shows | `main_pass.rs:157-186`; `main.wgsl:181-202`; `shadow_mask.wgsl:425-426`, `:471-477` | render, shaders |
| 0027 | "The lit pass draws the same geometry at depth equal and writes none: it meets the depths the depth passes kept" | holds: `fs_thin_main` has no `sample_mask` output and reads the covered samples; every twin runs `vs_thin`, whose `clip_position` is `@invariant` | `main.wgsl:94`, `:206-215`; `src/render/main_pass.rs:217-222` | render, shaders |
| 0027 | "A fragment of its twin that keeps no sample is discarded there, since another fragment's depth can equal its own" (amended 2026-11-16) | holds: `kept_samples = covered_samples & thin_fragment_samples(in)`, `discard` at 0 | `main.wgsl:204-213` | shaders |
| 0027 | "A fragment keeps a count of the four samples equal to its coverage times four, rounded to the nearest, and none below an eighth" | holds: `u32(saturate(coverage) * 4.0 + 0.5)`, 0 exactly below 0.125, each threshold exact in f32; the culling pass drops on the same `thin_kept_samples` | `common.wgsl:231-233`; `culling.wgsl:424`, `:426` | shaders |
| 0027 | "The samples kept follow from the count alone, sample 0, then 3, 1 and 2, so every fragment of a pixel keeps the same ones and a tube's faces share them" | holds for the order: `0xfb910 >> 4·count & 15` gives 0b0000, 0b0001, 0b1001, 0b1011, 0b1111, nested; the same samples in every fragment of a pixel hold where the fragments' counts agree, which per-fragment inputs can split at a rounding threshold (Notes) | `common.wgsl:230-234` | shaders |
| 0027 | "No dither, no pattern across pixels, no state across frames" | holds: the mask reads the count alone; `vs_thin`, `thin_fragment_samples` and the culling pass's coverage and drop read this frame's camera, Instance, Mesh and Meshlet tables and vertex only | `common.wgsl:231-234`; `main.wgsl:113-184`; `culling.wgsl:316-344` | shaders |
| 0027 | "Pipelines of their own. Each pipeline of the main pass has a twin whose fragment stage writes the mask, or for the lit pass meets it" | holds: three pipelines, three twins, each recorded after its pipeline; each twin runs `vs_thin`, with `fs_thin_surface_normals` (pre-pass, rays on), `fs_thin_depth` (pre-pass rays off, and the lit pass's depth draw) or `fs_thin_main` | `main_pass.rs:39-42`, `:146-221`; `main.wgsl:113`, `:187-207` | render, shaders |
| 0027 | "and the culling pass lists the geometry drawn through the twins apart, in slot order within each phase" | holds: a third scan component sums each slot's Thin Primitive indices, every index of a small Instance's; `thin_index` places a slot's at the buffer's end less the phase's end sum plus its own sum less the phase's start, so in slot order; each phase's Thin draw in `FrameDraw` | `culling_pass.rs:6-11`, `:104-122`; `culling.wgsl:306-314`, `:640-646` | render, shaders |
| 0027 | "The twins draw every frame, since any Instance can be small" | holds: each twin recorded unconditionally after its pipeline, every frame, seven indirect draws with rays on or off; `offset_blocks` writes each phase's thin draw, `instance_count` 1 and its `first_index`, every frame, at zero indices too | `main_pass.rs:323-329`, `:358-368`; `renderer.rs:883-931`; `culling.wgsl:508-517` | render, shaders |
| 0027 | "A Thin Primitive's coverage. Its vertices move out from its tube's centre line until the tube draws a pixel wide" | holds: the vertex moves to `centre + radial · drawn / radius` with `drawn = max(radius, 0.5 · pixel / 0.75)`; `town`'s wire and post are declared at their radius | `main.wgsl:122-134`; `common.wgsl:223-228`; `src/scenes/town.rs:1102-1116` | shaders, cook |
| 0027 | "and its coverage is the tube's true width over the width it draws" | holds: `radius / thin_drawn_radius(radius, pixel)`, the radius flat and the pixel's span interpolated at the centroid | `main.wgsl:182` | shaders |
| 0027 | "It reads the camera and the Instance alone" (a Thin Primitive) | holds: the camera uniform, the Instance's rotation-scale, its Meshlet's radius and the vertex's normal; the culling pass's widening and drop likewise; `pixel_size_per_metre` in the camera uniform, set from the target's height at creation and on resize | `main.wgsl:115-128`; `culling.wgsl:162-170`, `:330-344`; `renderer.rs:197-201` | render, shaders |
| 0027 | "With culling on, the culling pass drops an Instance of a Mesh that is all Thin Primitives once its widest tube keeps no sample at the bounds' nearest depth" | holds: dropped where LOD 0's widest Meshlet radius times the Instance's largest scale keeps no sample at the grown box's nearest view depth, conservatively; LOD 0's widest tube is every LOD's, since the cook locks a Thin Primitive's vertices | `culling.wgsl:328-344`, `:425-427`; `src/cook/mesh.rs:293-297`, `:340-347` | shaders |
| 0027 | "A Far proxy holds no Thin Primitive, so the rule ends at the Load radius" | holds: a Thin Primitive's triangles merge into the shell as ordinary triangles and `faceted` builds `Geometry::default()`, whose `thin_radius_metres` is `None`; no `world` Mesh is declared thin | `far_proxy.rs:68-94`, `:180-181`; `mesh.rs:190-205`; `src/scenes/world.rs:389-404` | cook |
| 0027 | "A small Instance's coverage. An Instance whose Mesh bounds cover under a pixel's area is scaled about their centre, across the view, until they cover a pixel's, each vertex keeping its depth" | shaders (vertex shader): broken off the view axis, `correctness`, L1-S02: each vertex's offset from the bounds' centre keeps its component along the view axis, the clip-w row's `xyz`, and the rest is scaled by `inverseSqrt(coverage)`, so each vertex keeps its w and its reverse-Z depth near/w (ADR 0015) as the clause asks; but the footprint the bounds' depth extent draws through perspective is not scaled by s², so the scaled bounds cover a pixel on the view axis only (0.84 px at coverage 0.5 at the view's corner); holds for the bounds the culling pass tests | `main.wgsl:136-160`; `common.wgsl:277-280`; `culling.wgsl:401-410` | shaders |
| 0027 | "and its coverage is the area they truly cover over a pixel's" | holds where the bounds' area share is at least (sphere diameter / 4 px)²; elsewhere the coverage is that sphere term, larger than the area share: `drift`, L1-S01 | `common.wgsl:253-274` | shaders |
| 0027 | "With culling on, the culling pass drops it once that rounds to no sample, under an eighth" | holds: the culling pass drops on `thin_kept_samples(coverage) == 0`; coverage under 0.125 keeps mask 0, and the scale stops at 0.125 | `culling.wgsl:424`; `common.wgsl:231-233`, `:277-280` | shaders |
| 0027 | "An Instance that draws four pixels wide across the sphere about its bounds is scaled no further, so a long sliver is not stretched" | holds: `sphere_share = diameter² / (4² · pixel²)` is 1 at four pixels across, so `max(area_share, sphere_share) ≥ 1` and the select returns 1, scale 1; below four pixels the sphere term holds the scale under 1/√(area share) (L1-S01) | `common.wgsl:267-274` | shaders |
| 0027 | "A Mesh with a Thin Primitive keeps that rule alone" | holds: coverage is worked out only where LOD 0 has no Thin Primitive triangle, so `small` is false otherwise; in `vs_thin` a Meshlet with a radius takes the tube's rule whatever its Instance | `culling.wgsl:399-411`; `main.wgsl:122`, `:135` | shaders |
| 0027 | "It reads the camera and the Instance alone" (a small Instance) | holds: the Instance's anchor, offset and rotation-scale, its Mesh's bounds and the camera; `pixel_size_per_metre` in the camera uniform | `main.wgsl:136-160`; `renderer.rs:197-201`; `uniforms.rs:57-59` | render, shaders |
| 0027 | "a masked Material derives its mask from alpha in the shader and writes it as `sample_mask`, through the twins" | not applicable: masked Materials are M7's; no Material derives a mask from alpha at the frozen commit, and nothing reads alpha as coverage | `main.wgsl:464` | report |
| 0027 | "A tube of few faces draws wider than a pixel so that its narrowest view spans one, a third wider for three faces, so `town`'s wire ends where it is a sixth of a pixel wide" | holds, arithmetic: drawn = pixel / (2 · 0.75), so a three-faced tube draws 1/0.75 = 4/3 of a pixel across a face; coverage = r / drawn = 0.75 · d / pixel, under 1/8 where d < pixel / 6 | `common.wgsl:224-228`, `:231-233` | shaders |
| 0027 | "Every tube takes three faces' share, 0.75 of its diameter, whatever its faces"; "`town`'s wire", "`town`'s six-faced post" | holds: one constant, `THIN_NARROWEST_VIEW = 0.75`, no face count read, `drawn = max(radius, 0.5 · pixel / 0.75)` for every tube, the stored radius the circumradius; on the scene side the wire is a 3-faced tube of 15 mm and the post a 6-faced tube of 25 mm, both declared at their radius, and the post's figures check | `common.wgsl:223-228`; `main.wgsl:125-127`; `src/scenes/town.rs:1102-1116`, `:1177-1186` | shaders, cook |
| 0027 | "A tapered tube keeps its taper as it widens, so the coverage at its ends is off by up to the quarter the cook tolerates" | holds: `THIN_RADIUS_TOLERANCE` = 0.25 bounds a ring's radius to within a quarter of the declared one; in form, one stored radius per Primitive, each vertex moved by the same factor from a centre off the true axis by the taper, `town`'s `cylinder(0.03, 0.02, ...)` at radius 0.025 such a tube | `src/cook/mesh.rs:96-98`, `:111-117`; `main.wgsl:125-132`; `src/scenes/town.rs:1078` | cook, shaders |
| 0027 | "The culling pass's compute stage already binds the nine storage buffers the Capability floor allows (ADR 0021), so the twins' packed indices share the frame's draw's buffer: each phase's fill its end, back from the phases' before, in slot order" | holds: four compute storage buffers in `SCENE_GEOMETRY`, five in `FRAME_DRAW`, and `FRAME`'s one compute binding the camera uniform; each phase's packed indices at the buffer's end | `src/render/bind_groups.rs:20-40`, `:71-102`; `culling.wgsl:556-562`; `culling_pass.rs:256-260` | render, shaders |
| 0027 | "A fragment stage that writes the mask reads a flat input, so the M4 Pro merges none of its fragments (ADR 0019)" | holds: `thin_fragment_samples`, which every stage that writes the mask calls, reads `radius_metres` and `instance_coverage`, both `@interpolate(flat)`, into its result, so no compiler can drop them | `main.wgsl:100`, `:103`, `:181-184`, `:197-202` | shaders |
| 0027 | "A small Instance keeps its mean coverage whatever its Mesh, since the samples it keeps fall by the area it gains" | broken off the view axis, L1-S02: on the axis the scale 1/√c multiplies the drawn area by 1/c and the mask keeps c of the samples, so the product is the Mesh's own area share whichever term set c; off it the drawn area grows by f/c + (1 − f)/√c, so the mean coverage falls by up to 35% at the view's corner | `common.wgsl:273`, `:279`; `main.wgsl:157-159` | shaders |
| 0027 | "The culling pass tests a small Instance's bounds as it draws them, scaled, against the view frustum and the depth pyramid, so whether it is drawn does not follow the frame before; it works out every Instance's coverage for that, culled or not" | holds: coverage is worked out for every slot with a Mesh, before the frustum test and whatever CULLING says, and both tests take the widened bounds; which phase draws it still follows the frame before's bit, as for every Instance (ADR 0019) | `culling.wgsl:398-417` | shaders |
| 0027 | "A pixel whose samples a mask splits between two surfaces is listed for the shadow mask pass's second surface, as any edge pixel is" | holds: a sample a fragment's mask did not keep holds the depth of the surface behind it, listed when that lies off the nearest sample's triangle plane by more than a pixel's width; the pass reads depths only; the lit twin's half holds, `lit` taking `covered & kept` as its coverage, so a thin fragment counts as the nearest surface only where none of its kept samples shows the second surface | `shadow_mask.wgsl:439-447`; `main.wgsl:210`, `:214`, `:390-395` | shaders |

## Killed

Three Findings, each kept in its section's table with the verifier's
reason, so the next audit does not file them again.

| id | claim | verifier's reason |
|---|---|---|
| L1-S05 | ADR 0003 and ADR 0017 give an Instance slot as 136.2 bytes with its offsets and visibility bit, and ADR 0003 517,376 slots at the checked-in Budgets, but a slot's offsets are four u32: 144.2 bytes a slot and 488,448 slots. | A duplicate of L1-R03: the same two ADR sentences (`docs/adr/0003-...md:3`, `docs/adr/0017-...md:18`) and the same 136.2 against 144.2 bytes a slot. L1-R03 keeps the defect, narrowed to take this Finding's research-note location and its account of how the offsets widened. |
| L1-S07 | The test that `mod.rs` rests its `unsafe` call on passes a counted loop whose body writes the loop's bound, so a loop that never ends can join `MAIN` or `CULLING` with every test green. | No loop that passes the check can run forever. `for_counter` admits only `name < bound` stepped by `name++` (`src/render/shader_tests.rs:757-764`), with a body that never writes `name` (`:735-743`). WGSL's `++` applies to integers alone, so the counter steps only after `name < bound` held and never wraps, and the loop ends within 2³² iterations whatever the body writes to the bound. The quote `wgpu-types-30-loop-bounding` speaks of "infinite loops", which this is not. |
| L1-C06 | ADR 0011's sentence that a development launch's cooker "cooks what changed before the runtime loads Blobs" still describes a cook ahead of the loads, while a launch cooks each Asset only as its own load reaches it. | The clause holds of every Blob a launch loads: `src/cook/launch.rs:1-2` cooks "each Asset it loads whose Blob is missing or stale, as it loads it", before the load. The reader concedes that the per-Blob reading holds, ADR 0018's amendment states the detail, and no sentence of ADR 0011 is false. |
