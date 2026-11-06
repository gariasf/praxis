# Acceleration structures for streamed worlds: state of the art

Researched 2026-11-05 for ticket #613, survey 4 of map #609. Its M4 options feed the decisions on #609 (the static TLAS off the critical path, re-anchoring, the Load radius's rebuild cost, exact acceleration-structure sizes); its M5 options wait for M5's charting.

**Summary.** On the M4 Pro with wgpu 30.0.1, rebuilding a static TLAS of `town`'s 254,256 Instances costs 17.5 to 18.1 ms of CPU and about 9 to 13 ms of GPU, and an empty TLAS of 262,144 slots already costs 4.3 to 6.9 ms of CPU to rebuild. A worker thread can take the fill, the recording and `finish` (13 to 14 ms of it), but the main thread still pays the submit (4.3 to 4.5 ms at that size, wgpu-core's walk of one dependency per Instance). On the GPU, a build in its own command buffer submitted before the frame's ray work overlaps it, but the two share the GPU: a 244,256-Instance build plus a 12 ms ray pass take 18.2 to 18.4 ms against 20.3 to 20.4 ms one after the other. Shipped open worlds cull their TLAS to tens of thousands of Instances (Epic about 100,000, AC Shadows about 30,000), rebuild it every frame on an async queue, and Unreal writes it relative to the camera at each rebuild; wgpu has one queue, no refit and no partitioned TLAS. At `town`'s density a Load radius of 256 m holds 112,000 to 144,000 static Instances, 384 m up to 256,000 and 512 m up to 368,000. Exact sizes are reachable today through wgpu-hal. The note ends in options and chooses none.

Sources: SIGGRAPH Advances talks (Epic 2022, Frostbite 2024, Ubisoft 2025), GTC 2019 (Battlefield V), NVIDIA, Apple, Microsoft and Khronos documentation and specifications, Unreal Engine 5.8.4 and Unity HDRP source, papers from 2003 to 2024, and wgpu 30.0.1 source and issues; a Rust probe on wgpu 30.0.1, measured on the M4 Pro (Measured on the M4 Pro). Each claim relied on is in `docs/references.toml`, cited by key. Boundaries: Chunk streaming and IO are survey 2's (#612); LOD and HLOD for raster are survey 3's (#627). Ruled out by the ticket: TLAS refit through wgpu-core and ray-tracing pipelines.

## Comparison

Costs on the M4 Pro are this note's probe unless a source is named; "build" is a full TLAS rebuild.

| Technique | Buys | Cost (measured or published) | Maturity | Fit: wgpu 30, Metal, ADRs 0007, 0014, 0016 | Licence |
|---|---|---|---|---|---|
| 1. Rebuild in the frame's encoder (Praxis today) | Nothing to manage | 254,256 Instances: 17.5–18.1 ms CPU in the frame, GPU in the frame | Is Praxis | A rebuild during play breaks ADR 0013's 6 ms CPU at `town`'s size | n/a |
| 2. Its own command buffer, before the frame's passes | GPU overlap with independent passes | GPU span 18.2–18.4 ms against 20.3–20.4 serial (244,256 Instances, 12 ms ray pass) | The dynamic TLAS does it | Safe wgpu; one queue | n/a |
| 3. Fill, record and `finish` on a worker | Takes 13–14 ms of CPU off the main thread | Main still submits: 4.3–4.5 ms at 254,256 | Unreal gathers dynamic instances on workers | Safe wgpu; ADR 0016 keeps the submit | n/a |
| 4. Async compute queue | Build hidden under other work | Epic: 2 ms build, 0.3 ms of frame | Shipped: Matrix Awakens, AC Shadows, Frostbite | Not in wgpu: one queue | n/a |
| 5. Instance buffer written on the GPU | No CPU instance gather | Epic: 0.02 ms for 100K | Shipped: Unreal 5 | Not in wgpu 30; upstream issues open | n/a |
| 6. Spare TLAS built ahead, swapped in | The frame never waits for a build | A second TLAS's memory (75.6 MB at 262,144) | Research, 2007 (CPU) | Safe wgpu: a bind group per TLAS | n/a |
| 7. Several static TLASes | Rebuild only the changed one | Per-ray: one query per TLAS, unmeasured | Shipped: Unreal 5.8's far-field scene layer | Safe wgpu; no TLAS binding arrays on Metal | n/a |
| 8. Re-anchor at every rebuild | Precision at 10 km at no extra build | None beyond the rebuild | Shipped: Unreal 5, Unity HDRP | Safe wgpu | UE EULA, Unity Companion |
| 9. Partition translation (PTLAS) | Re-anchor without a rebuild | D3D12 sample: 0.79 ms against 2.88 ms rebuild | Preview (DXR), NVIDIA Vulkan extension | Not in wgpu or Metal | MIT sample (Vulkan original Apache-2.0) |
| 10. Exact sizes through wgpu-hal | Measured bytes, per adapter | One query per structure at creation | Praxis already queries BLAS refit scratch so | ADR 0007's per-subsystem hal use | MIT OR Apache-2.0 |
| 11. Far proxies, separate far-field TLAS (M5) | Shadows and GI past the near field | Epic: same TLAS +44% near-field traversal | Matrix Awakens shipped one TLAS, far field offset; Unreal 5.8 has its own layer | Safe wgpu, as row 7 | n/a |
| 12. BLAS LOD matched to raster (M5) | No self-shadowing from LOD mismatch | A BLAS per LOD | NVIDIA guidance | Safe wgpu | n/a |
| 13. Per-Chunk merged static BLAS (M5) | Fewer Instances, less overlap | Memory: instancing lost per Chunk; unmeasured on GPU | Vendor guidance only | Safe wgpu | n/a |
| 14. Partitioned and cluster structures (M5) | Partial TLAS rebuilds, cluster BLAS | DXR sample above | Preview on DXR; NVIDIA-only on Vulkan; none on Metal | wgpu-hal per backend, or upstream | n/a |

## Techniques: M4

**1. Rebuilding in the frame's encoder (today).** wgpu-core always rebuilds [ref:wgpu-30-tlas-rebuild-only]. It encodes the build at `finish` [ref:wgpu-30-encode-at-finish], walking every slot up to capacity [ref:wgpu-30-tlas-instances-capacity-vec], converting each filled entry [ref:wgpu-30-tlas-instance-staging] into a staging buffer made for each build [ref:wgpu-30-tlas-staging-per-build], with a scratch buffer sized by capacity [ref:wgpu-30-tlas-scratch-per-build]; the build never uses its record of the lowest unmodified entry, so every entry is converted again [ref:wgpu-30-lowest-unmodified-unused]. At submit, each build and each pass that binds the TLAS walks one dependency per Instance [ref:wgpu-30-tlas-dependency-per-instance] [ref:wgpu-30-validate-tlas-dependencies]; upstream deduplicated them after 30.0.1 [ref:wgpu-pr-9835]. The probe splits the cost: capacity drives recording (3.4 to 5.7 ms at 262,144, empty or full), the filled count drives `finish`, submit and the GPU. An upstream report puts 100,000 Instances at 60 to 80 ms in a debug build, "3-4x faster" in release [ref:wgpu-issue-9946].

**2. Its own command buffer.** wgpu-hal opens one Metal queue per device [ref:wgpu-hal-30-metal-one-queue], so a build in another command buffer still shares the frame's queue. The probe shows the M4 Pro running a build submitted before an independent ray pass alongside it: the pass starts 3.2 to 3.3 ms after the build and the two overlap for 14.2 to 14.7 ms, but the pass takes 14.6 to 14.7 ms (11.7 to 14.7 alone) and the span saves only about 2 ms over running them one after the other. A build submitted after the pass waits for it. NVIDIA suggests considering async compute for builds beside rasterized G-buffer or shadow work, as "potentially beneficial" [ref:nvidia-2022-rtx-best-practices-as-builds]; in Praxis the depth passes are the work a build could hide under, if it is submitted before them.

**3. A worker thread.** `CommandEncoder` and `Tlas` are `Send` and `Sync` on native [ref:wgpu-30-command-encoder-send-sync], so a worker can fill, record and finish and hand the command buffer over; ADR 0016 keeps the submit on the main thread. `finish` holds the device's snatch lock for reading [ref:wgpu-30-encode-snatch-read], and `Buffer::destroy`, `Texture::destroy` and a `Device::poll` that frees what they left take it for writing [ref:wgpu-30-poll-snatch-write], so such a call on the main thread can wait for the worker's whole `finish`. In the probe the worker's builds left the frame's recording unchanged and added 0.4 ms to its submit in two of three rounds; the main thread paid 5.6 to 7.1 ms to submit each build. Unreal gathers dynamic instances on worker tasks [ref:unreal-5.8-rt-parallel-gather].

**4. Async compute.** The Matrix Awakens' TLAS takes "around 2 ms" to build, started early on async compute, which "reduced the actual impact on overall frame time to around 0.3ms" [ref:netzel-costa-2022-rtow-tlas-async]; Unreal 5.8 puts the build on async compute behind a switch [ref:unreal-5.8-rt-async-build]. AC Shadows rebuilds its TLAS every frame "in the async queue" [ref:leblanc-conte-2025-acs-tlas-async], and Frostbite always overlaps its BLAS and TLAS builds with G-buffer laydown [ref:apers-2024-gibs-async-as-build]. wgpu exposes one queue (row 2).

**5. A GPU-written instance buffer.** Epic found that gathering 100,000 instance descriptors on the CPU "can end up taking a significant amount of time" and builds the buffer in a compute shader for 0.02 ms [ref:netzel-costa-2022-rtow-gpu-instance-buffer]. wgpu 30 stages instances on the CPU (row 1); upstream issues propose an instance upload apart from the build [ref:wgpu-issue-10450] and an indirect build from a GPU buffer [ref:wgpu-issue-10512]. Neither is in a release.

**6. A spare TLAS, swapped.** Ize, Wald and Parker rebuild a BVH over several frames and switch it in "with two pointer swaps" [ref:ize-2007-async-bvh-swap]; no source found describes a shipped engine double-buffering its TLAS. In wgpu a bind group captures its TLAS at creation [ref:wgpu-30-tlas-binding-at-creation], so a swap is a second bind group. TLASes carry no tracked usage [ref:wgpu-30-tlas-tracker-stateless]; before a build wgpu-core records one barrier for all acceleration structures [ref:wgpu-30-tlas-barrier-before-build], which is empty on Metal [ref:wgpu-hal-30-metal-as-barrier-empty], so ordering a rebuild after a pass that read the same TLAS rests on Metal's hazard tracking [ref:metal-hazard-tracking-default]. Building the spare, never the bound one, avoids that question. Its memory is a second TLAS: 75.6 MB at 262,144 slots (row 10).

**7. Several static TLASes.** Epic's "better solution is to simply build each representation in a separate TLAS", which "completely avoid[s] the traversal overhead of overlapping instances" and lets the far field rebuild "at a lower rate" [ref:netzel-costa-2022-rtow-far-field-tlas]; Unreal 5.8 keeps a far-field layer of its ray-tracing scene [ref:unreal-5.8-rt-far-field-layer]. Split by Chunk group or ring, a Chunk change rebuilds one small TLAS, whose CPU cost follows its own capacity (0.2 to 0.5 ms empty at 16,384 slots). Each ray then queries every TLAS it may hit. wgpu 30 has TLAS binding arrays on DX12 and Vulkan only [ref:wgpu-types-30-tlas-binding-array], so on Metal each TLAS is its own binding.

**8. Re-anchoring.** Unreal translates world space so "its origin is at ViewOrigin" [ref:unreal-5.8-preview-translation] and writes each TLAS instance relative to it [ref:unreal-5.8-rt-translated-world-instances]; Unity subtracts a relative origin from every instance "If you do camera-relative ray tracing" [ref:unity-6000.6-rtas-relative-origin], and HDRP passes the camera position when it builds [ref:unity-hdrp-camera-relative-rtas]. Both rebuild every frame, so re-anchoring costs no extra build. Praxis rebuilds its static TLAS on a Chunk change, which can move the anchor at no extra cost; its dynamic TLAS is rebuilt each frame in the static TLAS's frame, so it follows when the frame that swaps in the static TLAS also hands its anchor to the camera uniform. A camera that travels without crossing a Chunk boundary keeps the old anchor, and the ray offset grows with its distance from it (`docs/audit/m3.5/scalability.md`, L4-02).

**9. Partition translation.** DXR's partitioned TLAS, in preview, lets the application store instance transforms relative to their partition, and "the position translations can be updated efficiently without triggering a rebuild of the entire Partitioned TLAS" [ref:dxr-2026-ptlas-partition-translation]; Vulkan's NVIDIA extension specifies partitions and a global partition for frequently updated instances [ref:vulkan-1.4.367-partitioned-tlas]. the D3D12 sample on Microsoft's blog rebuilds a TLAS in 2.88 ms where the partitioned one takes 0.79 ms, GPU not stated [ref:microsoft-2026-ptlas-preview-domino]. Neither wgpu 30 nor Metal has it (row 14).

**10. Exact sizes.** wgpu-core sizes a TLAS once, at creation, for its capacity [ref:wgpu-30-tlas-size-at-create], and keeps the figure crate-private [ref:wgpu-30-as-size-info-private]. `Device::as_hal` [ref:wgpu-30-device-as-hal] reaches the same query [ref:wgpu-hal-30-build-sizes]: Metal's `accelerationStructureSizesWithDescriptor` [ref:wgpu-hal-30-metal-build-sizes], Vulkan's `vkGetAccelerationStructureBuildSizesKHR` [ref:wgpu-hal-30-vulkan-build-sizes] and DX12's prebuild info [ref:wgpu-hal-30-dx12-build-sizes]. On Metal a TLAS query must pass some buffer as its instance buffer, or wgpu-hal's unwrap panics [ref:wgpu-hal-30-metal-instances-buffer-unwrap]. Beyond it, wgpu-core keeps an instance buffer of one record per slot [ref:wgpu-30-tlas-instance-buffer], 72 bytes on Metal [ref:metal-indirect-instance-descriptor], and makes scratch for each build. On the M4 Pro the query returns 216.38 bytes per slot plus at most 3.7 KB, and 22.29 bytes of build scratch; with the instance buffer that is 288.38 bytes per slot, against the 288.5 the ledger then estimated from `currentAllocatedSize`. The compacted size of a BLAS stays crate-private [ref:wgpu-30-blas-compacted-size-private], and the driver's allocated size is not reachable [ref:wgpu-hal-30-as-raw-private].

**Published costs against Instance count.** Epic measured 13 ms to build 1.5M instances and 2 ms for 100K, platform not stated [ref:netzel-costa-2022-rtow-build-vs-instances]; early Lumen tests found its per-frame rebuild "limited to 100K instances" [ref:wright-2022-lumen-tlas-instance-limit]; Unreal advises 100,000 instances or fewer after culling for 30 fps on consoles [ref:ue-5.8-rt-perf-tlas-instances-overlap]. Battlefield V's TLAS and BLAS builds fell from 64 ms to 14.5 ms by 4-degree culling (20,000 TLAS instances to 2,800, 5,000 BLAS rebuilds to 400), with "Occasional popping" [ref:deligiannis-2019-bfv-as-culling], and to 1.15 ms with BLAS-side changes, staggered rebuilds and the fast-build flag among them [ref:deligiannis-2019-bfv-as-staggered-cost]. DXR's fast-build flag is meant to take, as a rough rule of thumb, "about 1/2 to 1/3 the build time" [ref:microsoft-dxr-build-flags], and wgpu maps it to Metal's [ref:wgpu-30-prefer-fast-build-metal]; the probe did not try it. The two-level structure is Wald, Benthin and Slusallek's, whose top-level rebuild "only depends on the number of objects" [ref:wald-2003-two-level-toplevel-rebuild]; the 2021 survey warns that overlapping objects cut its culling [ref:meister-2021-bvh-star-two-level]. H-PLOC, a research GPU builder, builds a BVH4 over a million instances in 2.2 ms on a Radeon 7900 XT [ref:benthin-2024-hploc-tlas]; no API lets Praxis supply its own builder for ray queries.

## Techniques: M5

**11. Far proxies.** Epic traced World Partition HLOD meshes, "both simplified and merged together", as a far field, made mutually exclusive with the near field by ray masks [ref:wright-2022-lumen-far-field-hlod]. In one TLAS that cost "a show-stopping 44% penalty to all near-field traversal costs" (1.65 to 2.37 ms on PS5 at 1080p, 1.77 ms with the far field translated away) [ref:wright-2022-lumen-far-field-overlap], and Epic call multiple TLASes the proper fix (row 7). Unreal 5.8's far field, when enabled, starts at 200 m and, by its documentation, ends at 1 km [ref:ue-5.8-lumen-far-field-and-fallback]. AC Shadows describes no far proxies: a coarse, low-LOD scene [ref:leblanc-conte-2025-acs-rt-scene], about 30,000 instances in 320 MB on Xbox Series X [ref:leblanc-conte-2025-acs-bvh-stats], and a 128×128 fallback cubemap, one face a frame, for far geometry [ref:leblanc-conte-2025-acs-far-cubemap]. wgpu's 8-bit instance mask carries a near/far split [ref:wgpu-30-tlas-instance-fields]. No source measures the pop where proxies take over.

**12. Which LOD each BLAS holds.** A secondary ray "must 'see' the same LODs" as the rasterizer, or shadows break [ref:lloyd-2020-stochastic-lod-dxr]; NVIDIA suggests the same LOD for both to avoid self-intersection, or back-face culling where they differ [ref:nvidia-2022-rtx-best-practices-lod-culling]. Epic's fallback meshes self-intersected, fixed by a short back-face-culled ray before the long one [ref:wright-2022-lumen-fallback-self-intersection]; Praxis's shadow rays start from the depth target, the case they describe. Stochastic LOD puts two LODs in the TLAS and picks per ray by mask, at most about 10% overhead on an RTX 2080 Ti [ref:lloyd-2020-stochastic-lod-dxr]; its dithered transition needs filtering without temporal accumulation (ADR 0008). A discrete LOD change is a BLAS change, so a static TLAS rebuild under ADR 0020:20's triggers.

**13. Per-Chunk merged static BLASes.** NVIDIA [ref:nvidia-2022-rtx-best-practices-shadow-rays], Apple [ref:apple-wwdc22-10105-instance-overhead] and AMD [ref:amd-rra-instance-overlap] name overlapping instances as a per-ray cost and merging as the remedy; Epic asks only that overlap be kept to a minimum [ref:ue-5.8-rt-perf-tlas-instances-overlap]. Re-braiding, which opens object BVHs during the top-level build, gave 1.2 to 2.1 times the rendering speed on CPU [ref:benthin-2017-partial-rebraiding], and Frostbite's selective re-braiding with offline BLASes saves 0.35 ms on Xbox Series X [ref:apers-2024-gibs-rebraiding]; both are builder-side. No published GPU measurement merges a region's static instances into one BLAS. Merging a Chunk's static clutter cuts the TLAS's Instances, and so its rebuild (row 1), at the cost of instancing within the Chunk; `docs/performance.md`'s option B for `town`'s shadow rays is this.

**14. Partitioned and cluster structures.** DXR Part 2 (preview) adds partitioned TLASes, whose instances can swap BLAS within explicit bounds for LOD without a rebuild [ref:dxr-2026-ptlas-overview], and clustered geometry for streaming and LOD [ref:dxr-2026-clusters-support]; Microsoft lists "All RTX GPUs", AMD's RX 9000 series and Intel's Arc B-Series, with Qualcomm to follow [ref:microsoft-2026-clas-ptlas-preview]. Vulkan has NVIDIA's partitioned [ref:vulkan-1.4.367-partitioned-tlas] and cluster [ref:vulkan-1.4.367-cluster-acceleration-structure] extensions. Metal has neither; its nearest form is multi-level instancing [ref:apple-wwdc23-10128-multi-level-instancing], up to 16 levels on Apple9 [ref:metal-feature-set-tables-instancing-levels]. wgpu 30's instances reference a BLAS only [ref:wgpu-30-tlas-instance-blas-only]; wgpu's ray-tracing tracking issue lists partitioned and cluster structures as "Nice to have" [ref:wgpu-issue-6762], and a PR merged after 30.0.1 exposes a Vulkan acceleration structure's handle, so a TLAS wgpu binds can be built by hand over cluster BLASes [ref:wgpu-pr-10187]. Intel's per-frame cluster BVH for Nanite-class meshes costs 5.7 to 9 ms on an Arc A770, by writing the hardware layout [ref:benthin-2023-micro-poly-hlod-rt].

## Measured on the M4 Pro

A Rust probe outside the repo, on wgpu 30.0.1's Metal backend with ray queries and timestamps, built 19 box BLASes (12 triangles each, 1 to 17 m on a side) and placed N Instances at `town`'s static density (its 244,256 static Instances over its 1 km square), uniformly over a square, each turned about the vertical. Each rebuild refilled every entry, recorded the build between two one-invocation compute passes that query the TLAS, timed on the GPU from the end of the first to the start of the second, submitted, and waited; timestamps were resolved in a later submission, since a beginning-of-pass timestamp resolved in its own command buffer read the previous frame's value. CPU columns come from a second run without the marker passes, whose bindings add their own submit walk (row 1). Each row: 5 warm-up and 30 measured rebuilds, the median, given as the range over three rounds. Apple M4 Pro, macOS 27.0.1, AC power, one-minute load average 4.1 to 8.8 (other sessions building), no other GPU bench.

Static TLAS of capacity 262,144 (`STATIC_TLAS_CAPACITY`), ms:

| Instances | Fill | Record | `finish` | Submit | CPU total | GPU, idle before |
|---:|---:|---:|---:|---:|---:|---:|
| 0 | 0 | 3.44–5.67 | 0.83–1.25 | 0.02–0.04 | 4.29–6.87 | 0.18–0.23 |
| 10,000 | 0.03–0.06 | 3.50–5.41 | 1.20–1.69 | 0.20–0.24 | 4.93–7.43 | 1.10–2.94 |
| 50,000 | 0.17–0.34 | 3.65–6.33 | 2.36–3.49 | 0.85–0.86 | 7.07–11.95 | 2.48–8.40 |
| 100,000 | 0.33–0.63 | 3.89–5.34 | 3.82–5.50 | 1.64–1.67 | 9.72–15.04 | 4.29–7.86 |
| 150,000 | 0.49–0.50 | 4.03–4.14 | 4.99–5.28 | 2.75–2.76 | 12.24–12.69 | 6.68–10.79 |
| 200,000 | 0.65–0.66 | 4.27–4.33 | 6.38–6.61 | 3.44–3.54 | 14.83–15.16 | 8.82–11.78 |
| 254,256 | 0.84–0.85 | 4.45–4.58 | 7.89–8.17 | 4.33–4.52 | 17.53–18.08 | 11.07–12.73 |

Capacity alone (empty TLAS), CPU total: 0.20–0.48 ms at 16,384 slots, 0.79–1.19 at 65,536, 1.87–2.10 at 131,072, 8.86–13.86 at 524,288. At 524,288 slots, 400,000 Instances cost 29.9–31.4 ms of CPU and 17.4–18.8 of GPU; 524,288 cost 35.6–57.1 and 21.0–23.9. The GPU column is noisy because the GPU idled before each build: the same 244,256-Instance build took 18.3 to 22.1 ms after an idle wait in the overlap runs, and 8.5 to 8.8 ms when it followed other work.

GPU overlap: two TLASes of 244,256 Instances each; the frame is one 2560×1440 compute pass tracing four short shadow-like rays per pixel against the first TLAS while the second is rebuilt. Medians over 30 frames, range over three rounds, ms:

| Order | Frame pass | Build | GPU span | Overlap |
|---|---:|---:|---:|---:|
| Frame pass alone | 11.7–14.7 | | 11.7–14.7 | |
| Build alone, after idle | | 18.3–22.1 | 18.3–22.1 | |
| Frame's command buffer, then the build's | 11.4–11.7 | 8.5–8.8 | 20.3–20.4 | 0 |
| Build's command buffer, then the frame's | 14.6–14.7 | 17.6–18.1 | 18.2–18.4 | 14.2–14.7 |
| Build, then frame pass, one encoder | 12.9–13.0 | 20.2–21.1 | 20.2–21.1 | 12.9–13.0 |

Worker thread: the main thread recorded and submitted the frame pass above, then waited, 120 frames a round; a worker filled, recorded and finished a 244,256-Instance build in a loop and the main thread submitted each one it found. Without the worker the frame's submit took 3.43 to 3.53 ms (one pass binding a 244,256-Instance TLAS, row 1's walk); with it 3.83 to 3.86 ms in two rounds and 6.48 ms in a round at higher load, the frame's recording 37 to 81 µs either way. The worker's CPU per build was 14.5 to 21.1 ms, and the main thread's submit of each build 5.64 to 7.11 ms.

Static Instances per Load radius at `town`'s density (16,008 per 256 m Chunk), counting a Chunk as loaded when any part of it is within the radius of a camera anywhere in its own Chunk, against the nearest rows of the table:

| Load radius | Chunks | Static Instances | Nearest rows (CPU total, ms) |
|---:|---:|---:|---|
| 256 m | 7–9 | 112,053–144,068 | 100,000 to 150,000: 9.7–15.0 |
| 384 m | 11–16 | 176,083–256,121 | 150,000 to 254,256: 12.2–18.1 |
| 512 m | 20–23 | 320,151–368,174 | 254,256 to 400,000 at 524,288 slots: 21.6–31.4 |
| 768 m | 39–45 | 624,295–720,340 | past ADR 0003's 524,288 slots |

## Options for M4 (no recommendation)

- **Where the static TLAS rebuilds.**
  - **A. Its own command buffer on the main thread**, submitted before the depth passes. GPU overlap as measured; the CPU stays in the frame (17.5 to 18.1 ms at 254,256 Instances).
  - **B. A worker fills, records and finishes; the main thread submits.** The main thread keeps the submit (4.3 to 4.5 ms at 254,256 on wgpu 30.0.1, less where the dependency walk is deduplicated upstream) and a buffer or texture destroy, or the poll after it, can wait on `finish`'s lock. A new build lands a frame or more after its Chunk loads.
  - **C. Several static TLASes**, by Chunk group or ring, rebuilt one at a time; each costs its own capacity (0.2 to 0.5 ms empty at 16,384) plus its Instances, and each ray queries all of them, at a cost to measure.
  - **D. Fewer Instances**: cull static Instances by size and distance (Battlefield V, Epic, AC Shadows), or merge per Chunk (row 13). Cuts CPU and GPU together; culled Instances cast no shadow.
- **How it swaps in.**
  - **S1. Two static TLASes and two bind groups**, the spare built while the other is bound, switched by the first frame after its submit; costs a second TLAS (75.6 MB at 262,144 slots).
  - **S2. In place**, as today: the frame that rebuilds traces the new TLAS, and the build sits in that frame's GPU time.
- **When the anchor moves.**
  - **R1. At every static rebuild**, to the camera's Anchor, with the dynamic TLAS following in the same frame through the camera uniform's TLAS anchor.
  - **R2. R1 plus a rebuild when the camera is a set distance from the anchor** without a Chunk change, bounding L4-02's ray offset.
- **Load radius**: the table above at `town`'s density; the world scene's sparse land scales by its own Instance count. A capacity above 262,144 costs CPU whether filled or not.
- **Exact sizes.**
  - **E1. Query wgpu-hal at creation** for each TLAS and BLAS, adding wgpu-core's instance buffer (72 bytes a slot on Metal, 64 elsewhere); compacted BLAS sizes stay estimates unless read through the raw API.
  - **E2. Keep the per-adapter estimates**, re-measured.

## Options for M5 (no recommendation)

- **Far field.** F1: proxies in a separate far TLAS (row 11, Epic's fix); F2: proxies in the static TLAS behind an instance mask (Epic's 44%); F3: a fallback cubemap of far geometry (AC Shadows); F4: nothing past the Load radius, as M4.
- **Which LOD each BLAS holds.** L1: the raster's LOD, rebuilt with the TLAS when it changes; L2: one coarser LOD for rays, with Epic's short-ray fix for self-intersection; L3: stochastic LOD, which uses mask bits and needs filtering without temporal accumulation.
- **Per-Chunk merged static BLASes.** M1: merge each Chunk's small static Instances, prototyped on `town`'s shadow rays first; M2: keep instancing.
- **Partitioned and cluster structures.** P1: wgpu-hal per backend (DXR preview and NVIDIA's Vulkan extensions on the RTX 2070 SUPER, multi-level instancing on Metal); P2: wait for wgpu, where they are "Nice to have".

## Not verified

- Why an isolated build's GPU time ranged from 8.5 to 22 ms: the probe did not pin the GPU's clock state.
- How a static rebuild overlaps Praxis's own depth passes, rather than the probe's synthetic ray pass.
- Ray cost of querying several static TLASes (option C), and of a merged per-Chunk BLAS (row 13).
- Metal's ordering of a pass that reads a TLAS and a later rebuild of it in place, beyond wgpu's empty barrier.
- Whether wgpu 31's deduplicated dependencies (#603's upgrade) change the submit figures here.
- The hardware behind Epic's, Battlefield V's and Microsoft's timings; none states it.
- A shipped engine that double-buffers its TLAS; only CPU research was found.
- The fast-build flag's effect on the M4 Pro.
