# Open-world streaming and IO: state of the art

Researched 2026-11-05 for ticket #612, survey 2 of M4's map (#609). Its options feed the owner's decisions there on (a) to (e) and, with survey 4, on the Load radius.

**Summary.** The published answers converge on four points.
- Where a shipped engine publishes how it holds static world content, it is flat instance data per tile, with no entity per placement. Horizon Zero Dawn builds immutable tiles on its streaming thread, splitting any above 24K instances and pooling groups under 1K into an orphan tile, and keeps the main thread to activating them.
- Speed is limited by per-object setup, not by disk. Sunset Overdrive's runtime initialisation was its limit, not file IO. On the M4 Pro, one Chunk at `town`'s density (16,663 Renderables) costs 1.3 to 1.6 ms of main-thread work to spawn as entities and give slots, and 0.6 to 0.7 ms to despawn. Writing its Instances straight from Placement bytes costs 0.14 to 0.15 ms. One thread reads and decodes large cooked Blobs at 1.5 GB/s of output, while small Blobs cost 43 to 70 µs each to open and read.
- Data no longer used is kept until memory runs short, then evicted least recently used first. This holds from Funkhouser's 1996 walkthrough system to Titanfall 2 and CesiumJS; freeing at once (Bevy) is what Unity warns against as churn.
- Reads run on a small dedicated thread or two and decoding runs on worker threads at low priority. Bevy's IO pool does the reverse.

wgpu 30 exposes no memory-budget query on Metal. The note ends in options for (a) to (e) and the Load radius and chooses none.

Sources: GDC, GCAP and studio slides with speaker notes (Guerrilla 2017 and 2023, Insomniac 2015, Respawn 2017, CD Projekt Red 2024); papers (I3D 1992, Graphics Interface 1996, IEEE Visualization 2002, IEEE PVG 2003, GraphiCon 2021, MobiCom 2025, arXiv 2025 and 2026); Epic's documentation through web.archive.org snapshots, Unity's and OpenMW's documentation; Apple, Microsoft and Khronos documentation; source at the tags and commits linked: wgpu 30.0.1, Bevy 0.18.1, smol-rs blocking 1.6.2 and async-fs 2.2.0, zstd 1.5.7, liburing, libdispatch, xnu, OGRE 14.3.0, CesiumJS 1.123 and DirectStorage. It also reports two probes measured on the M4 Pro (Measured on the M4 Pro). Each claim the note relies on is in `docs/references.toml`, cited by key.

Boundaries: mip streaming, residency, page sizes and GPU decompression are [#46's note](gpu-memory-residency-streaming.md); job systems and upload queues are [#47's](job-systems-and-frame-threading.md); acceleration structures for streamed worlds are [survey 4's](acceleration-structures-for-streamed-worlds.md); LOD, HLOD and the far field are survey 3's. Ruled out by the ticket: virtual geometry pages (ADR 0019), archive packaging, simulating unloaded Chunks. ADR 0008 is not engaged: nothing here accumulates across frames.

## Comparison

"Main thread" is the thread that owns the ECS World (ADR 0016). Town density is 254,256 Renderables per km², 16,663 per 256 m Chunk (computed).

| Technique | Buys | Cost (measured or published) | Maturity | Fit: ADRs 0013, 0016, 0017, 0018 | Licence |
|---|---|---|---|---|---|
| 1. An entity per Placement (Praxis today) | Every system sees static content; one spawn path | Per Chunk on the main thread: spawn 0.72–0.75 ms, slots and Instances 0.53–0.81 ms, despawn 0.50–0.58 ms (probe); Sunset Overdrive 180 ms per hex before time-slicing | Shipped: Sunset Overdrive; Unity Entities stages them on a worker | Structural changes on the main thread, inside the 6 ms | n/a |
| 2. Instances in a slot range the Chunk owns | No entity; Instances written from cooked bytes, off the main thread | 0.14–0.15 ms per Chunk (probe); Decima: 500K to 1.5M instances potentially visible | Shipped: Horizon Zero Dawn | ADR 0003's slots become ranges; culling and the TLAS get a Chunk grouping | n/a |
| 3. Load inside a radius, unload outside it | Simple; memory follows the radius | Not published in the sources read | Shipped: Sunset Overdrive, Unreal World Partition (256 m cells, 768 m range in its example) | Is the Load radius as `CONTEXT.md` defines it | n/a |
| 4. Hysteresis: a hold radius, an activation zone or a delay | No thrash at the border | OGRE: hold 3000 against load 2000; OpenMW: 5 s cache delay | Shipped: OGRE, OpenMW; Decima's hulls for encounters | Held Chunks count against capacities | OGRE MIT |
| 5. Request ahead from speed | Chunks loaded before they are reached | Funkhouser 1996: worst frame 5.78 s to 0.263 s with lookahead; 4 ms per frame | Research since 1992; OpenMW 1 s, L3GS 1 s | 50 m/s for 1 s is 50 m, a fifth of a Chunk | n/a |
| 6. Fast travel: preload or hold | No hole at the destination | Sunset Overdrive: all seven zones at once | Shipped: Sunset Overdrive, CesiumJS, OpenMW (off by default) | Map decision 10: hold until loaded | CesiumJS Apache-2.0 |
| 7. Free when the last user goes | Most free space | Churn when a user returns (Unity) | Shipped: Bevy | Pools count users (map decision 4) | Bevy MIT OR Apache-2.0 |
| 8. Keep until short, evict least recently used | No reload on return | Titanfall 2: at most 16 MiB dropped per frame | Shipped: Titanfall 2, Unreal texture streaming, CesiumJS; research since 1996 | Eviction runs inside a refused allocation (ADR 0017) | CesiumJS Apache-2.0 |
| 9. Dedicated IO thread, decoding on workers | Reads never wait behind CPU work | Titanfall 2: 2 requests in flight; Cyberpunk 2077: decoding at the lowest job priority | Shipped: Titanfall 2, Cyberpunk 2077, Unreal, DirectStorage | std threads; no new dependency | n/a |
| 10. Async IO pool with blocking reads (Bevy) | Async API over files | One more thread hop and an 8 MB pipe per reader | Shipped: Bevy | Needs `blocking` or `async-fs`, absent from `Cargo.lock` | MIT OR Apache-2.0 |
| 11. OS asynchronous file IO | Reads queued without a thread each | Not published in the sources read | io_uring, Dispatch I/O, IOCP, DirectStorage | Three APIs, `unsafe`, none in the stack | liburing LGPL and MIT; libdispatch Apache-2.0 |
| 12. zstd decoding per Blob | Decode speed about level-independent (level 1 and up) | 1580 MB/s on a Core i7-9700K; 1.75–2.00 GB/s decoded here, one thread (probe) | Shipped everywhere zstd is | One context per thread; parallel across Blobs only | BSD OR GPLv2 |
| 13. Budget from the adapter | Follows the OS and other apps | DXGI: over budget may stutter | D3D12, Vulkan, Metal | wgpu 30: thresholds on D3D12 and Vulkan only, none on Metal | n/a |
| 14. Fixed pools by tier | Predictable; tested | Unreal 200 to 1000 MB; Titanfall 2 0 to 5860 MiB | Shipped: Unreal, Titanfall 2 | Is ADR 0013's 8 GB | n/a |

## Techniques

**1. An entity per Placement.** Today `spawn_town` `spawn_batch`es every Renderable, and prepare gives each a slot in an entity map (`src/render/prepare.rs`). Sunset Overdrive found that "The limiting factor on how fast we could stream the world was runtime initialization, not file IO": a typical zone held about 2000 actors, and initialising a hex's actors at once made a 180 ms hitch per hex, time-sliced to 3 ms a frame; destruction was never time-sliced [ref:ruskin-2015-sunset-overdrive-init-cost]. Unity Entities builds entities on a worker in a staging World and moves them in, so the main thread pays one sync point [ref:unity-entities-1.3-exclusive-entity-transaction]. The probe's figures are in Measured on the M4 Pro.

**2. Instances in a Chunk's slot range.** Horizon Zero Dawn's static world is StaticTiles, "defined by the streaming system", "created immutable" [ref:decima-2017-static-tiles]: "mostly flat buffers of GPU data", split above 24K instances and merged under 1K, built on the streaming thread while "the main thread just has to make tiles active" [ref:decima-2017-static-tile-build]; it handles "500K up to 1.5 million instances" potentially visible from one position [ref:decima-2017-static-scene-scale]. Unreal 5.6's experimental Fast Geometry Streaming plugin targets "immutable static geometry" that loads "faster with constant frame rates" [ref:ue-5.6-fast-geometry-streaming]; Epic published no figures, nor how it holds instances. In Praxis a Chunk would own a contiguous range of Instance slots, written from its cooked Placements by the IO pool into CPU memory and copied into the `StagingBelt` by the main thread (ADR 0017). At town density a Chunk's Instances are 2.13 MB (16,663 × 128 bytes, computed). A static Placement then has no entity, so a system that wants one (a script, physics) gets it another way.

**3. Load inside a radius, unload outside it.** Sunset Overdrive's region was "a point, a radius, and a list of zones": load on entering the radius, unload on leaving it [ref:ruskin-2015-sunset-overdrive-region-radius]. Unreal World Partition loads grid cells within a Loading Range of each streaming source; its documentation's example is a 256 m cell and a 768 m range, and a Block on Slow Streaming option "Blocks loading in situations where grid cells are not loading fast enough" [ref:ue-5.7-world-partition-grid]. Horizon Zero Dawn loads tiles around the player at four resolutions: 9 high, 9 medium, 9 low, and 12 very low that are always loaded [ref:fahner-2023-hzd-tile-rings].

**4. Hysteresis.** OGRE pages hold to a larger radius than they load to, "to minimise thrashing if the camera goes backwards and forwards at the loading border", 3000 against 2000 by default [ref:ogre-14.3-hold-radius]. Horizon Zero Dawn's area-based streaming, for quests, scenes and dynamic encounters, uses a primary hull as a loading hint and a secondary hull as an activation hint [ref:fahner-2023-hzd-area-hulls]. OpenMW keeps an asset 5 s after its last cell unloads [ref:openmw-0.49-cell-preload]. No paper studying hysteresis was found.

**5. Requesting ahead from speed.** The foundation is Funkhouser's walkthrough system, which loads whole cells and predicts "which objects might be rendered in future frames" because disk reads are slow [ref:funkhouser-1992-walkthrough-prefetch]. Its 1996 successor bounds the range reachable in N frames by "a sphere [...] with radius Nvp", weighted toward the direction of travel [ref:funkhouser-1996-observer-range], and loads per frame only what the disk reads in one frame [ref:funkhouser-1996-read-and-release]. Against OS paging alone, prefetching up to 16 frames ahead cut the worst frame from 5.78 s to 0.263 s, and misses grew with speed [ref:funkhouser-1996-results]. Later systems extrapolate the camera's linear and angular speed [ref:correa-2003-lookahead-extrapolation], widen the prefetch frustum with panning speed [ref:varadhan-2002-expanded-frustum], and state that speed "has to be adjusted to the disk bandwidth" [ref:correa-2003-speed-vs-bandwidth]. Since 2019 the disk literature is thin: one paper left prefetching out after it degraded performance when the cache was full [ref:semenov-2021-prefetch-degradation], and network streaming of Gaussian splats predicts the viewport 1 s ahead from 0.5 s of history [ref:tsai-2025-l3gs-viewport-prediction]. In shipped code, OpenMW predicts the player 1 s ahead [ref:openmw-0.49-cell-preload], and CesiumJS skips a request when 60 frames of the current movement exceed the tile's size [ref:cesium-1.123-cull-requests-while-moving]. Sunset Overdrive sized hexes of about 110 m for 14 m/s [ref:ruskin-2015-sunset-overdrive-hex-speed] to stream in 1.6 s, and found disk IO "was not the limiting factor" [ref:ruskin-2015-sunset-overdrive-hex-deadline]. At 50 m/s a whole second of lead is 50 m; the probe's IO and decode time for a Chunk's Placements is milliseconds (Measured), so the lead is set by frame-paced work, not by the disk.

**6. Fast travel.** A teleport made Sunset Overdrive load "*all seven* of the surrounding zones at once rather than [...] two or three" [ref:ruskin-2015-sunset-overdrive-teleport]. CesiumJS preloads tiles at a flight's destination during the flight [ref:cesium-1.123-tile-cache-budget]; OpenMW can preload every fast-travel destination, off by default for its memory [ref:openmw-0.49-cell-preload]. The map's decision 10 already holds until the radius is loaded.

**7. Free when the last user goes.** In Bevy, dropping the last strong handle frees the asset [ref:bevy-0.18.1-strong-handle-drop], processed at the next PreUpdate by `track_assets` [ref:bevy-0.18.1-track-assets]. Unity counts references but frees only whole bundles [ref:unity-addressables-2.5-refcount-unload], and names the cost: releasing the last asset of a bundle and reloading it at once is what it calls asset churn [ref:unity-addressables-2.5-asset-churn].

**8. Keep until short, least recently used first.** Funkhouser's replacement "closely resembles a least recently used (LRU) policy", freeing before loading so memory "is never overburdened" [ref:funkhouser-1996-read-and-release]; Varadhan and Manocha protect what is in use or just fetched [ref:varadhan-2002-replacement]. Titanfall 2 puts mips unused this frame on a drop list ordered by `-metric + -frames_unused` [ref:barb-2017-tf2-drop-list] and drops while over its memory target, when told to drop all unused mips, or to make room for a better-ranked load, at most 16 MiB a frame on the first path [ref:barb-2017-tf2-add-drop]. Unreal keeps cached mips while the streaming pool has room; only `r.Streaming.DropMips`, a debug option, drops them "even if the streaming pool allows it" [ref:ue-5.6-texture-streaming-retention]. CesiumJS trims unused tiles only above `cacheBytes`, 512 MiB by default [ref:cesium-1.123-tile-cache-budget]; an out-of-core Gaussian-splat trainer and renderer (arXiv, 2025; v4 2026) writes its VRAM cache back to RAM least recently used first past a memory threshold [ref:windisch-2026-lod-gaussians-lru]. In Praxis's fixed Arenas (ADR 0017) eviction would run when an allocation is refused, and freed ranges fragment the Arena.

**9. A dedicated IO thread, decoding on workers.** Titanfall 2 used one "Async thread", "2 requests in flight", with priority queues and 64 KiB reads [ref:barb-2017-tf2-async-io]. Cyberpunk 2077 lists Async IO beside, not among, its job workers [ref:tremblay-2024-cyberpunk-threads] and runs "Deserialization & Decompression on Jobs • At the lowest job priority", throttled, with no frame job allowed to wait on streaming [ref:tremblay-2024-cyberpunk-streaming-jobs]. Unreal has an optional separate async loading thread [ref:ue-5.6-async-loading-thread]. DirectStorage has one submission thread by default and lets a title switch its decompression threads off to decode on its own jobs [ref:directstorage-developer-guidance-queues]. ADR 0016 already puts reads and decoding on an IO pool outside the compute pool; the compute pool takes 11 of the M4 Pro's 12 cores (8 performance, 4 efficiency), so decode threads beside it oversubscribe.

**10. Bevy's async IO pool.** bevy_tasks' IoTaskPool expects tasks that "should just await receiving data from somewhere (i.e. disk)" [ref:bevy-0.18.1-io-task-pool]; Bevy sizes it at 25 % of cores, 1 to 4 [ref:bevy-0.18.1-io-pool-threads]. Its asset server runs each load, decode included, as one IoTaskPool task [ref:bevy-0.18.1-asset-load-on-io-pool], reading through `async_fs`, which waits up to 500 ms for one of 128 open-file slots on macOS and then opens anyway [ref:bevy-0.18.1-asset-file-reader]. `async-fs` is `std::fs` on `blocking`'s pool [ref:async-fs-2.2.0-blocking], which exists because "blocking is not allowed inside futures" and grows to 500 threads by default [ref:blocking-1.6.2-thread-pool] and hands bytes over through an 8 MB pipe [ref:blocking-1.6.2-unblock-pipe]. Licences: Bevy MIT OR Apache-2.0 [ref:bevy-0.18.1-licence]; blocking Apache-2.0 OR MIT [ref:blocking-1.6.2-licence].

**11. OS asynchronous file IO.** io_uring queues reads "without blocking the calling process" on Linux [ref:liburing-2.15-io-uring]; Dispatch I/O gives random-access channels whose reads "may be performed concurrently" on macOS [ref:libdispatch-1542-dispatch-io], and `fcntl` offers `F_RDADVISE` and `F_NOCACHE` [ref:xnu-12377-fcntl-rdadvise-nocache]. On Windows, overlapped reads turn synchronous on cache hits, NTFS-compressed files or a saturated cache manager, and `FILE_FLAG_NO_BUFFERING` "is the best way to guarantee" asynchrony [ref:ms-kb156932-async-disk-io-synchronous]; DirectStorage's `Enqueue` blocks when its queue is full, "easily [...] several milliseconds" [ref:directstorage-developer-guidance-queues]. Metal's IO queue is in #46's note (row 11). None is reachable from the stack without `unsafe` per platform.

**12. zstd.** zstd 1.5.6 decompresses the Silesia corpus at 1580 MB/s at level 1 on a Core i7-9700K [ref:zstd-1.5.7-benchmarks], and across its positive levels decompression speed "remains roughly the same at all settings" [ref:zstd-1.5.7-decompression-speed-by-level]. Its worker threads compress only [ref:zstd-1.5.7-multithreading-compression-only]; parallel decoding takes "one context per thread" [ref:zstd-1.5.7-decompression-context-per-thread]. A Blob is one zstd frame per Mesh or per mip level (`src/assets/cooked.rs`), so one Blob decodes on one thread. Licence: BSD OR GPLv2 [ref:zstd-1.5.7-licence].

**13. A budget from the adapter.** Metal's `recommendedMaxWorkingSetSize` is an approximation of what the device can allocate "without affecting its runtime performance" [ref:apple-mtldevice-recommended-max-working-set-size], and `currentAllocatedSize` the bytes in use [ref:apple-mtldevice-current-allocated-size]; on the M4 Pro the first reads 19,069,665,280 bytes, above 8 GB ([#46's note](gpu-memory-residency-streaming.md), row 8). Vulkan's `heapBudget` "may change based on other activity in the system", to be polled "once per frame, per few seconds" [ref:vulkan-ext-memory-budget-interval] [ref:vulkan-ext-memory-budget]. DXGI's Budget is what "the application should target", and above it "the application may incur stuttering" [ref:ms-dxgi-query-video-memory-info-budget]; an event can replace polling [ref:ms-dxgi-budget-change-notification]; D3D12's budget fluctuates "dramatically when the user switches away" [ref:ms-d3d12-residency-budget-fluctuates]. wgpu 30 exposes only `MemoryBudgetThresholds`, which fail creation or lose the device at a percentage of the budget, "Currently only the D3D12 and (optionally) Vulkan backends support these options" [ref:wgpu-types-30-memory-budget-thresholds]; on Metal the check is a `TODO` returning `Ok(())` [ref:wgpu-hal-30-metal-check-if-oom-todo], and the open issue would refuse creation above 90 % of the budget [ref:wgpu-issue-7460-oom-detection]. The figure itself is reachable only through `as_hal`: the `MTLDevice` from `raw_device` [ref:wgpu-hal-30-metal-raw-device], which needs `objc2-metal` as a direct dependency, and DXGI's `query_video_memory_info` on the DX12 adapter [ref:wgpu-hal-30-dx12-query-video-memory-info].

**14. Fixed pools by tier.** Unreal's scalability levels set `r.Streaming.PoolSize`, a size in MB [ref:ue-5.6-texture-streaming-pool], to 200, 400, 700 and 1000 [ref:ue-5.7-scalability-texture-pool], optionally capped to available GPU memory [ref:ue-5.6-texture-streaming-pool-vram]; Titanfall 2's PC settings were 0, 375, 750, 1250 and 5860 MiB, with 928 MiB on console [ref:barb-2017-tf2-pool-tiers]. Neither tracks the OS budget live in the sources read.

## Measured on the M4 Pro

Two Rust probes outside the repo, built in release against this tree's `praxis` library (its public `cooked` module), bevy_ecs 0.18.1 with Praxis's features, glam 0.32.1 and zstd 0.14.2. Apple M4 Pro, macOS 27.0.1, AC power, on 2026-11-05; the one-minute load average was 4.9 to 17.7, from other sessions' tests; an on-access antivirus scanner was running, which may add to each file open.

**Read and decode.** The Blobs the Cooked cache's index names were copied with `F_NOCACHE` writes and `fsync` before each variant, so a first read comes from the SSD: a second read was 1.6 to 7 times faster, and a read with `F_NOCACHE` matched the first. Each thread takes the next file, `File::open` and `read_to_end`, then, in the decode variants, `CookedTexture::from_blob` or `CookedMesh::from_blob` (zstd, then meshopt for Meshes). Decoded bytes are the levels' and the vertex, index and Meshlet arrays' sizes. Large Blobs: the 7 at or above 100 KB (two Meshes, five Textures; 18.4 MB read, 28.8 MB decoded), three rounds, ranges of the three. All Blobs: 10,192 (21.7 MB read, mean 2.1 KB, 47.0 MB decoded), one round, as each variant re-copies 10,192 files.

| Variant, large Blobs | 1 thread | 2 | 4 | 8 |
|---|---|---|---|---|
| Read, cold (MB/s read) | 3,821–3,882 | 5,097–5,187 | 4,839–5,345 | 4,479–5,265 |
| Decode, in memory (MB/s decoded) | 1,753–2,002 | 3,300–3,455 | 4,775–5,217 | 6,579–6,929 |
| Read and decode, cold (MB/s decoded) | 1,463–1,554 | 2,357–2,473 | 2,839–3,294 | 3,490–3,704 |

| Variant, all 10,192 Blobs | 1 thread | 2 | 4 | 8 |
|---|---|---|---|---|
| Read, cold (ms; µs per file) | 713; 70 | 1,547; 152 | 457; 45 | 438; 43 |
| Read, warm (ms; µs per file) | 440; 43 | 980; 96 | 159; 16 | 122; 12 |
| Decode, in memory (ms) | 22 | 14 | 26 | 38 |
| Read and decode, cold (ms) | 915 | 679 | 648 | 401 |

Large Blobs read at SSD speed and decode at about half the read speed on one thread, so a decode thread, not the disk, sets the pace below four threads. Small Blobs cost 43 to 70 µs each to open and read on one thread, 20 to 32 times their 2.2 µs decode; the two-thread row is slower than one thread in both read variants and was not explained. A Chunk's Placements at town density, in the probe's 68-byte form, are 1.13 MB (computed); at one thread's 937 to 995 MB/s of compressed input, reading and decoding them is about a millisecond (computed).

**Spawning one Chunk.** 16,663 Placements (random translations in a 256 m square, rotation about the vertical, uniform scale), spawned as `Transform` (requiring `WorldTransform`), `MeshRef` and `MaterialRef` as in `src/components.rs`, into a World already holding 0, 12 or 30 such Chunks. "Slots and Instances" iterates an `Added<Transform>` query, inserts each entity into an `EntityHashMap` slot map and writes a 128-byte `InstanceData`, as prepare does for a new Renderable; "despawn" calls `World::despawn` per entity and "slots freed" removes them from the map. The slot-range row writes the same `InstanceData` from the Placement bytes alone. Medians of 50 repeats; the range is over three runs.

| Resident entities | `spawn_batch` | Slots and Instances | Despawn | Slots freed | Slot range: Instances from Placements |
|---|---|---|---|---|---|
| 0 | 0.731 ms | 0.530–0.542 ms | 0.496–0.501 ms | 0.100–0.101 ms | 0.144 ms |
| 199,956 | 0.724–0.742 ms | 0.645–0.654 ms | 0.498–0.507 ms | 0.100–0.103 ms | 0.144–0.149 ms |
| 499,890 | 0.738–0.749 ms | 0.808 ms | 0.547–0.579 ms | 0.102–0.103 ms | 0.148–0.149 ms |

A Chunk in and out as entities costs 1.9 to 2.2 ms of main-thread work at town density, 1.3 to 1.6 ms of it in the frame it arrives unless time-sliced; the `Added` query's cost grows with the World. The slot-range write is a fifth of the spawn alone and needs no World.

## Load radius candidates

Chunks whose nearest point lies within the radius of the camera, over 256 camera positions in a Chunk; arrivals per second at 50 m/s over 60 s, with the most in any one second in brackets (computed). Instances assume town density everywhere, the worst case; the world scene is sparse between settlements (map decision 9). Praxis's capacities today: 524,288 Instance slots, 262,144 static TLAS Instances. [Survey 4](acceleration-structures-for-streamed-worlds.md) counts static Instances only, 16,008 per Chunk, with the same Chunk counts, and gives each radius's TLAS rebuild cost.

| Load radius | Chunks loaded | Instances at town density, at most | Arrivals per second along an axis | Along the diagonal |
|---|---|---|---|---|
| 256 m | 7–9 | 149,967 | 0.57 (2) | 0.68 (4) |
| 512 m | 20–23 | 383,249 | 0.97 (3) | 0.97 (4) |
| 768 m | 39–45 | 749,835 | 1.35 (4) | 1.52 (4) |
| 1,024 m | 64–69 | 1,149,747 | 1.77 (5) | 1.85 (5) |
| 1,536 m | 135–141 | 2,349,483 | 2.52 (5) | 2.63 (7) |
| 2,048 m | 228–241 | 4,015,783 | 3.28 (6) | 3.43 (10) |

A hold radius one Chunk wider than the Load radius holds about the next row's count. At 2,048 m and town density, entity streaming would cost about 7 ms of main-thread work per second of travel (3.4 Chunks in and as many out a second, 1.9 to 2.2 ms per Chunk in and out, computed).

## Options (no recommendation)

**(a) How static Placements are held.**
- A1. Entities, as today (row 1). Each Chunk spawns its Placements with `spawn_batch` and despawns them on unload. Every system sees them, and nothing in prepare changes. It costs 1.9 to 2.2 ms of main-thread work per Chunk at town density, which needs time-slicing across frames to stay inside 6 ms, and the World grows to the loaded Instance count.
- A2. Slot range (row 2, the charting's lean). The IO pool writes a Chunk's Instances from its cooked bytes into a contiguous range of slots, and the main thread copies them into the `StagingBelt`. It costs 0.14 to 0.15 ms per Chunk, off the main thread. Two things change: ADR 0003's per-entity slots become per-Chunk ranges, which need a range allocator and defragmentation or fixed-size ranges, and static Placements have no entity for scripts or physics until one is made on demand.
- A3. Entities built off the main thread (row 1, Unity's form). Spawn into a staging World on the IO pool, then move them into the main World in one step. bevy_ecs 0.18.1's support for such a move was not checked.

**(b) When an unused Mesh, Material or Texture is freed.**
- B1. At once (row 7). The pool entry and its Arena range are freed in the frame the count reaches zero. This is the simplest and leaves the most room. Content crossing the hysteresis border churns.
- B2. Kept until short, least recently used first (row 8). Unused entries stay Resident. When an allocation is refused, the least recently used are freed, a bounded amount per frame as in Titanfall 2, and the allocation is retried. This avoids reloads on return. It needs a recency list per pool and eviction inside the load path.
- B3. B1 after a delay (OpenMW's 5 s). It bounds churn without a pressure path.

**(c) Hysteresis and lead.**
- C1. A hold radius one Chunk (256 m) beyond the Load radius (OGRE's form). It needs capacities for the next row of the table.
- C2. A time-based hold, unloading N seconds after leaving.
- C3. None (Sunset Overdrive's form), with reloads at the border.

Each takes a lead for requests: the Load radius plus 50 m/s times the measured time to Resident, of which IO and decode are milliseconds.

**(d) The IO pool's shape.**
- D1. A few dedicated std threads issue blocking reads. Decoding runs on the same threads or at low priority on the compute pool (row 9). No new dependency.
- D2. bevy_tasks' IoTaskPool with blocking reads inside its tasks (row 10 without `blocking`). This stalls its executor threads, against its own guidance.
- D3. IoTaskPool with `async-fs` (row 10). Bevy's form, and a new dependency.
- D4. OS asynchronous IO per platform (row 11). Unsafe on three APIs.

Measured: one thread reads and decodes large Blobs at 1.5 GB/s of output and four at 2.8 to 3.3 GB/s. Small Blobs are bound by the per-file open, 43 to 70 µs, whatever the decoder.

**(e) The memory Budget.**
- E1. The fixed 8 GB (row 14, ADR 0013 as written). It sits below the M4 Pro's `recommendedMaxWorkingSetSize`, 19,069,665,280 bytes.
- E2. The adapter's report caps the 8 GB (row 13). DXGI and Vulkan are reachable through `as_hal` or wgpu's thresholds. Metal needs `objc2-metal` as a direct dependency (an owner decision) and `unsafe`, until wgpu's issue lands.
- E3. Wgpu's `MemoryBudgetThresholds` alone, on D3D12 and Vulkan, as a backstop beside E1. Nothing on Metal.

**The Load radius (with survey 4).** The candidates are the table's rows. At town density only 256 m stays under the static TLAS's 262,144 Instances and only 512 m under the 524,288 slots. Every radius from 768 m up needs the world scene's sparser land, LOD, or larger capacities, whose rebuild time is survey 4's.

## Not verified

- **Spider-Man and Ghost of Tsushima.** Marvel's Spider-Man (GDC 2019) and Ghost of Tsushima's streaming talk (GDC 2021) are video only; neither was watched. Their streaming speeds and deadlines are not in this note.
- **Unreal source.** Fast Geometry Streaming's costs, and whether World Partition predicts from velocity or unloads with hysteresis, are not documented. The source needs an Epic login.
- **Prepare's real per-Renderable cost.** The probe imitates prepare's slot map and Instance write; it did not run `FramePreparer::prepare`.
- **Cooked Blob mix.** The Blob mix is the current Cooked cache's: texture-wall's 10,005 tiny Textures dominate the small-Blob set. No Chunk Blob exists yet.
- **Probe conditions.** The two-thread anomaly in the small-Blob rows, and how much the antivirus scanner adds per open, were not isolated.
- **`recommendedMaxWorkingSetSize` at run time.** Whether it changes during a run is not documented by Apple.
- **Arena fragmentation.** Fragmentation of `offset-allocator` Arenas under long streaming was not measured.
- **Research since 2019.** Disk-backed prefetching for large scenes yielded one paper (GraphiCon 2021). The rest stream RAM to VRAM or over networks.
