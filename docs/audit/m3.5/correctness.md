# Audit lane 1, correctness against intent

The lane's report, in the form `docs/agents/audit.md` gives: every ADR's
decision clauses read against the code at the frozen commit
`8bc34eb18fcfb79307193551e96619316aef566c`, a verdict per clause, the
milestone spec's stories and every registry key cited in code checked the
same way. Four readers, one per module, each a section below; the lane's
report session writes the Summary, the counts and the Killed section and
runs the verifier over every section. Finding ids carry the module's
letter so that sections written in parallel never collide: render
`L1-R<n>`, shaders `L1-S<n>`, cook with assets and scenes `L1-C<n>`,
tooling `L1-T<n>`.

## Summary

Written by the report session.

## Counts

Written by the report session.

## render

Scope: `src/render/` outside the shaders and the test modules:
`renderer.rs`, `prepare.rs`, `instance.rs`, `uniforms.rs`, `staging.rs`,
`mod.rs`, `targets.rs`, `bind_groups.rs`, `culling_pass.rs`,
`depth_pyramid_pass.rs`, `main_pass.rs`, `froxel_pass.rs`,
`tonemapping_pass.rs`, `light_count_pass.rs`, `shadow_mask_pass.rs`,
`acceleration_structures.rs`, `blas_refit.rs`, `bend_pass.rs`,
`readback.rs`; 10,677 lines, every file read in full. Three readers at
high effort shared the module by file, each with the lane brief verbatim:
the core data path (renderer, prepare, Instance, uniforms, staging), the
raster passes with their targets and bind groups, and the ray-tracing
passes with the bend pass and the GPU stats readback. ADRs read: 0001 to
0006, 0008, 0014, 0015, 0016, 0019, 0020, 0021, 0022, 0023. The
`CLAUDE.md` review checklist was read as the operational form of those
clauses. A WGSL part was opened only to confirm what a Rust-side constant,
layout, binding or format must match; every shader-side clause is marked
for the shaders section. Every `where` is `path:line` at the frozen
commit, under `src/render/` unless given in full. `deviations.md` holds
no row at this commit; `DIFFERENCES.md` was read for what is not a
Finding. The two open issues this reader weighed, the long-lived Vulkan
device's flipped shadow verdict on the RTX 2070 SUPER and `town`'s
sub-pixel wires aliasing into dots, are under *Open issues read*.

### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0001 | "indexed by the Material ID that each Instance carries ... one bind group serves every material" | holds | `instance.rs:42`; `renderer.rs:278`, `:636` (one `MATERIALS` group) |
| 0001, 0005 | Material ID and Instance slot read from the Meshlet record | holds, Rust side | `culling_pass.rs:191-196` (12-byte record: slot, Meshlet, Material ID); `bind_groups.rs:273-280` read by the vertex stage |
| 0002 | "sub-allocations of one shared vertex buffer and one shared index buffer" | falls in cook with assets (`src/assets/pools.rs`); the renderer binds the one Arena | `renderer.rs:365` |
| 0003 | "a slot per Renderable ... updated incrementally through dirty flags; no per-entity uniform buffers" | holds | `prepare.rs:81-88` (`Changed` filters), `:140-151` |
| 0003 | "each Renderable keeps a slot in the Instance buffer until it stops being a Renderable" | broken for a Renderable skipped for a missing Mesh or Material Handle: L1-R02 | `prepare.rs:358-375` |
| 0003 | "a freed slot is taken again before a fresh one" | holds | `prepare.rs:160-161` (`free.pop().or(fresh)`) |
| 0003 | "the buffer holds a fixed 524,288 slots" | holds | `instance.rs:4` (`1 << 19`); `renderer.rs:333-339`, created once |
| 0003 | "past which a Renderable is refused, warned about once, counted in Frame stats and retried every frame" | holds | refused and warned once `prepare.rs:156-175`; counted `:377` into `src/state.rs:271`; retried through `unwritten` `:354`, `:378`, `:411-417` |
| 0003 | "prepare writes only the slots of new Renderables and of those whose World transform, Mesh or Material changed, found by bevy_ecs change detection ... so camera motion writes no Instance" | holds on its purpose; refined by ADR 0020 and ADR 0022, which the ADR does not record: every Moving Instance is rewritten each frame: L1-R04 | `prepare.rs:81-88`, `:400-418` |
| 0003 | "through the per-frame staging path" | holds | `renderer.rs:514-520`, `:548-550`; `staging.rs:54-119` |
| 0003 | "A freed slot is written an Instance with no Mesh, which the GPU does not draw, and is taken again from the next frame" | holds: freed slots get `NO_MESH` this frame and join `free` after the visits | `prepare.rs:419-434`; test `renderer.rs:1806-1850` |
| 0003, 0014 | "the buffer stores an integer anchor plus an f32 offset per Renderable" | holds | `instance.rs:41`, `:43`; `prepare.rs:385` |
| 0004 | "The renderer never reads the ECS World mid-frame: a prepare pass builds Frame data ... and the renderer consumes only that" | holds: `renderer.rs` imports `bevy_ecs` only under test | `renderer.rs:1-23`, `:163-166`; `culling_pass.rs:284-291` takes `&FrameData` |
| 0004 | "it lists no draws" | holds: `FrameData` has no draw list; the frame's draws are four, fixed | `prepare.rs:22-45`; `renderer.rs:694` |
| 0004 | "every Frame data must reach the renderer, in order" | holds: prepare and upload run only for a frame that is drawn, and render follows | `src/lib.rs:233-253`; `src/capture/mod.rs:260-261`; `src/state.rs:263-282`; `renderer.rs:478-481` asserts no upload without a render |
| 0004, 0016 | "Frame data is an owned, `Send` value with no borrow into the ECS World" | holds: compile-time assertion | `prepare.rs:47-50` |
| 0005 | "Per-Instance data (model matrix, normal matrix, Material ID) sits in a structured buffer" | holds: 128 bytes, `STORAGE` | `instance.rs:36-45`; `renderer.rs:334-339` |
| 0005 | "the lookup key is the Instance slot in the Meshlet record a packed index names" | holds, Rust side; the packing is the shaders section's | `culling_pass.rs:166-168` asserts the record fits above the vertex's six bits |
| 0006 | "Components hold data only: no closures, no `Box<dyn Trait>`, no behaviour methods" | broken in letter for three pure methods on components that prepare calls: L1-R03 | `src/components.rs:20-39`, `:71-77`; `src/transform/mod.rs:11-31` |
| 0008 | "the main pass reads none [of the screen-space history]" | holds: `MAIN_PASS_INPUTS` binds this frame's records and mask only; the culling pass's visibility bits choose a phase, not a pixel | `bind_groups.rs:269-301`; `culling_pass.rs:150-152` |
| 0014 | "The GPU never holds a World position in f32: it reads an integer anchor on a 64 m grid plus an f32 offset" | holds: camera, Instance pivot and point light go through `anchor_and_offset`; a directional light's `offset` is a unit direction; `view_proj` is projection times rotation; TLAS transforms and bounds are anchor-relative | `instance.rs:8`, `:13-17`; `prepare.rs:291`, `:385`, `:535-537`, `:556`; `src/camera/mod.rs:80-82`; `acceleration_structures.rs:479-487`; `renderer.rs:499`, `:506-510` |
| 0014 | "Subsystems that compute in f32 (... ray-tracing acceleration structures) convert at their boundary into a local frame they own" | holds: `anchored_transform` composes the translation from the static anchor in f64 and rounds once to f32; the dynamic TLAS's box is in the same frame | `acceleration_structures.rs:463-474`, `:479-487` |
| 0014 | "the Froxel pass and the lit pass compose [point lights] camera-relative" | falls in shaders (`froxel.wgsl`, `common.wgsl`) | |
| 0015 | "Depth is cleared to 0" | holds; the two later depth attachments load | `targets.rs:292`; `:305`, `:320` |
| 0015 | "compared with `Greater`" | holds; the lit pass compares `Equal` and writes nothing | `targets.rs:387`, `:395-401` |
| 0015 | "in `Depth32Float`" | holds | `targets.rs:12`, `:158-162`, `:385` |
| 0015 | "with no far plane" | falls in the camera module (`projection_matrix`) | |
| 0015 | "a Hi-Z pyramid keeps the minimum depth, which is the farthest" | holds, Rust side: the GPU test reduces every sample with `f32::min` and compares bit-exactly; the reduction is the shaders section's | `depth_pyramid_pass.rs:1-4`, `:145-204` |
| 0016 | "Only the main thread submits, presents or polls" | holds: every `queue.submit` and `device.poll` is called from `Renderer::upload`, `render`, `flush_gpu_frame_stats` or construction, on the calling thread, none from a worker or a callback | `renderer.rs:719`, `:725`; `staging.rs:83`; `acceleration_structures.rs:255`; `blas_refit.rs:120`, `:124`, `:140`; `readback.rs:304`, `:321-325`; `src/lib.rs:248-254` |
| 0016 | "`desired_maximum_frame_latency` stays 2" | holds | `mod.rs:43` (`FRAMES_IN_FLIGHT = 2`); `src/lib.rs:165` |
| 0016 | "Per-frame data ... written through `Queue::write_*` ... or into a ring of mapped buffers whose slot is reused only after its `map_async` callback fires" | holds: staging has three slots, `is_mapped` set only by the callback, an unmapped slot waits on `device.poll`; a readback slot with a map pending is not reused, and its frame is untimed and counted | `staging.rs:9`, `:76-93`, `:150-159`, `:285-294`; `readback.rs:240-244`, `:389-395`, `:487-496` |
| 0016 | "wgpu completes a map only after the last submission using the buffer finishes, and refuses to submit work that uses a mapped buffer, so the ring decides whether the CPU waits" | holds: no copy is recorded into a slot unless it is `Recorded` for this frame; `render` never waits on a map; `flush` waits at the end of a run | `readback.rs:273-287`, `:302-330`, `:364-381` |
| 0016 | "map and work-done callbacks run at a known point of the frame" | holds: `on_submitted_work_done` on the frame's encoder fires at a submit or poll on the main thread | `readback.rs:291-296` |
| 0016 | "Workers record `CommandEncoder`s ... push no error scopes" | not applicable: no worker records at this commit | |
| 0016, checklist | targets recreated on resize with the bind groups that hold them | holds: `RenderTargets::new` runs on resize and `resize` calls `bind_render_targets` on every pass, each building its target-reading groups in one function called at creation and on resize | `targets.rs:54-57`, `:98`; `renderer.rs:444-461`; `culling_pass.rs:233-234`, `:327-334`; `depth_pyramid_pass.rs:60`, `:66-98`; `main_pass.rs:61-66`, `:192-199`; `tonemapping_pass.rs:100`, `:111-114`; `light_count_pass.rs:70`, `:82-85` |
| 0019 | pass order: culling one, Froxel lists, depth pre-pass, pyramid, culling two, phase-two depth and the shadow mask with rays on, lit, tonemapping | holds; the Froxel pass runs after the first culling dispatch so that the gate counts it, where the ADR says "at the start of the frame" (see *Notes for the verifier*) | `renderer.rs:617-685`, `:627-633` |
| 0019 | "a 4x MSAA reverse-Z depth pre-pass" | holds | `targets.rs:16` (`SAMPLE_COUNT = 4`); `main_pass.rs:91-95`, `:108`; `targets.rs:288-297` (Clear 0, Store) |
| 0019 | "then a lit pass at depth equal" | holds | `main_pass.rs:173`, `:332-333` |
| 0019 | "Geometric edges get 4x MSAA (the Apple9 maximum)" | holds; the pyramid and shadow mask passes pin the count at 4 | `targets.rs:16`; `depth_pyramid_pass.rs:18`; `shadow_mask_pass.rs:55` |
| 0019 | "with alpha-to-coverage for masked materials" | deferred by the M3 spec to M6 (masked and double-sided Materials) | `main_pass.rs:94` (`alpha_to_coverage_enabled: false`) |
| 0019 | "a compute pass appends the Meshlets of each Instance ... in slot order, in two phases" | holds, Rust side: offset slots, offset blocks, append records, append triangles per phase; the slot order inside the shader is the shaders section's | `culling_pass.rs:1-13`, `:292-322` |
| 0019 | "Phase one appends those visible the frame before, one bit per slot" | holds | `culling_pass.rs:150-152`, `:218-222` (`INSTANCE_CAPACITY / u32::BITS` words, zeroed at creation) |
| 0019 | "A compute pass reduces that depth into the Hi-Z pyramid, each texel the minimum of every sample below it" | holds, Rust side: one dispatch per level, level 0 from the depth target, each later level from the one below, down to 1x1 | `depth_pyramid_pass.rs:84-96`, `:113-125`; `targets.rs:139-154` |
| 0019 | "Phase two appends the others whose bounds lie anywhere nearer than the pyramid's minimum ... and rewrites the bits" | holds, Rust side: the whole pyramid is bound; the test is the shaders section's | `culling_pass.rs:132-139`, `:400-406` |
| 0019 | "Phase two's Instances draw into depth: with shadow rays off, at the start of the lit pass; with them on, in a pass of their own, which like the pre-pass then writes each sample's surface normals too" | holds | `main_pass.rs:270-297`, `:328-331`; `renderer.rs:658-671` |
| 0019 | "A lit pass draws both phases' at depth equal into a transient 4x colour attachment, memoryless on Apple GPUs, that it resolves into the `Rgba16Float` HDR target" | holds; wgpu-core 30.0.1 asserts the transient attachment's Clear or DontCare load and Discard store at pass begin | `targets.rs:22`, `:163-168`, `:345-357` |
| 0019 | "which a tonemapping pass then writes into the target" | holds | `tonemapping_pass.rs:128-131`, `:148-174` |
| 0019 | "one `draw_indexed_indirect` per phase for the depth pipeline and one of both phases for the lit pipeline"; "No count draws" | holds: three indirect draws, no `multi_draw_indirect` or count draw | `main_pass.rs:262`, `:296`, `:330`, `:333`; `culling_pass.rs:94-95` |
| 0019 | "each index packs a Meshlet record in its high bits and a vertex within that Meshlet in its low six" | holds, Rust side | `culling_pass.rs:166-168` |
| 0019 | "the point lights of each pixel's Froxel, which a compute pass at the start of the frame lists from a light buffer of fixed capacity" | holds, Rust side | `froxel_pass.rs:118-131`; `renderer.rs:298-303` (light buffer capacity) |
| 0019, 0020 | "up to 59 lights per pixel; a light past them casts no shadow there and is counted" | holds: `SHADOW_MASK_LIGHTS = 59`, `59 + log2(FROXEL_SLICES) == 64` asserted; the overflow buffer is cleared per frame, copied into the readback slot, read into Frame stats and warned once | `targets.rs:24-26` (`Rg32Uint`); `shadow_mask_pass.rs:32`, `:67`, `:339`, `:390-392`; `readback.rs:277-286`, `:422-433`, `:467` |
| 0019 | "the depth passes also write each sample's triangle normal and the normal interpolated over it, 4 bytes per sample" | holds | `targets.rs:42-45`, `:207-212`; `main_pass.rs:116-130` |
| 0019 | "A fragment stage that reads any flat input is never merged ... the depth passes' normals and the lit pass ... both read a flat input" | falls in shaders (`main.wgsl`) | |
| 0019 | "a compute pass traces [the shadow mask] from the depth target ... at each pixel's nearest sample and ... the farthest of those too, into the mask's second layer" | holds, Rust side: recorded after phase two's depth pass and before the lit pass; two dispatches, the second indirect from the first's list; the depth target, surface normals, mask and mask samples bound | `renderer.rs:664-680`; `shadow_mask_pass.rs:306-321`, `:348-367` |
| 0019 | "the second surfaces ... are traced from a packed list" | holds in form: `second_surfaces` list and indirect dispatch arguments cleared each frame; the stand-in group is bound where the dispatch buffer is read indirectly | `shadow_mask_pass.rs:94-96`, `:256-260`, `:340-343`, `:367` |
| 0019 | "`max_storage_buffers_per_shader_stage` at least 9, since the culling pass's compute stage reads and writes nine" | holds, exactly nine: five in `FRAME_DRAW`, four compute-visible in `SCENE_GEOMETRY` | `bind_groups.rs:74-78`, `:84-95`, `:127-131` |
| 0019 | "Culling tests Instances; tests per Meshlet wait for a profile" | holds: the pass tests Instance bounds | `culling_pass.rs:292-322` |
| 0020 | "one hard shadow ray per pixel per light ... skipped where the light adds nothing"; "from an origin offset along the surface's normal and composed in the TLAS's frame" | falls in shaders (`shadow_mask.wgsl`); the Rust side dispatches one 8x8 workgroup per tile and passes the TLAS anchor in the camera uniform | `shadow_mask_pass.rs:355-360`; `renderer.rs:499-506` |
| 0020 | "for one of the first seven directional lights ... toward that light again from each other sample" | holds: `SHADOW_EDGE_LIGHTS = 7`, declared into WGSL from Rust, `4 * (1 + 7) <= 32` asserted; the edge list is a fourth, indirect dispatch | `shadow_mask_pass.rs:52`, `:60-61`, `:379-385`; `shaders/mod.rs:260` |
| 0020 | the 4-neighbour verdict difference, the terminator per sample, the point light's surface bit from the first facing sample | falls in shaders (`shadow_mask.wgsl`) | `shadow_mask_pass.rs:368-385` records the dispatches |
| 0020 | "the static TLAS covers the static Instances in a frame anchored at load ... never re-anchored" | holds: the anchor is taken from the first Frame data's camera and never replaced | `acceleration_structures.rs:109`, `:140-141`, `:188-192` |
| 0020 | "rebuilt in the frame its static Instances change, as when one is spawned or despawned" | holds: the first update always builds; a write of a static Instance, or a freed slot that held one, sets `changed`; a Moving Instance's write clears its entry | `acceleration_structures.rs:140`, `:143-169`, `:177-179`; test `:649-725` |
| 0020 | "Moving Instances form the dynamic TLAS, rebuilt each frame in the static TLAS's frame from at most 10,000 of them, those in the lowest slots" | holds | `acceleration_structures.rs:19`, `:312-349`, `:356-371`; `prepare.rs:436-452`; `renderer.rs:499-502` |
| 0020 | "a shadow ray the static TLAS lets through is traced against it" | falls in shaders; both TLAS are bound in `SHADOW_RAY_SCENE` | `shadow_mask_pass.rs:123-131`; `bind_groups.rs:99-114` |
| 0020 | "its build's recording and submit, and the build itself on the GPU, are timed in each frame that builds it" | holds for the proxies and refit forms; broken for the crowd's rebuild form, where no markers are recorded and the build reads as zero: L1-R05 | `renderer.rs:487-503`, `:603-616`, `:716-722`; `acceleration_structures.rs:356-371`, `:394-413`; `readback.rs:588-592` |
| 0020 | "The `crowd` Reference scene's figures ... cast their shadows through rigid per-bone shadow proxies ... the default form" | holds: `CrowdForm::Proxies` is the default; `BlasRefit::new` returns `None` for it | `renderer.rs:146`; `blas_refit.rs:56-58` |
| 0020 | "refit Render switch ... refitted to its bent shape each frame through wgpu-hal on Metal ... the dynamic TLAS's build waits for the refits on the GPU through a Metal shared event" | holds: `AccelerationStructureBuildMode::Update` through `as_hal_mut::<Metal>`, Metal only; the shared event signals from the refit submission and the next submission waits; the CPU wait is a logged fallback | `blas_refit.rs:59-62`, `:107-109`, `:131-143`, `:233-278`, `:473-547`; `renderer.rs:133-136` |
| 0020 | "the refits are timed on the GPU from the end of the bend pass to a marker after them" | holds | `blas_refit.rs:112-122`, `:390-460`; `readback.rs:655-657`; `renderer.rs:594-601` |
| 0020 | "A constant ambient term stands in for indirect light" | falls in shaders | |
| 0021 | "4x MSAA on every attachment format the main pass uses, read per format from the adapter's texture format features" | broken: the surface normals attachment, `Rgba8Unorm` at 4x, is not in the floor's list: L1-R01 | `targets.rs:207-212`; `main_pass.rs:123-127`; `src/gpu/mod.rs:256-258` |
| 0021 | "Those formats are `Depth32Float` and the HDR target's `Rgba16Float`" | drift, the same Finding: a third format since the surface normals landed | `docs/adr/0021-capability-floor.md`, Consequences, first bullet |
| 0021 | "the window's surface and the capture target take only the tonemapping pass's single-sample output" | holds | `tonemapping_pass.rs:80` |
| 0021 | "`max_storage_buffer_binding_size` stays at the default 128 MiB" | holds, at the limit: 2^25 packed indices of 4 bytes | `culling_pass.rs:63-71`, `:197-201` |
| 0021 | `max_compute_workgroups_per_dimension` default 65,535 | holds: asserted against the device; every direct dispatch is below it, `append_triangles` is indirect in rows of 65,535 | `culling_pass.rs:52`, `:169-171`, `:312`, `:317`; `depth_pyramid_pass.rs:120-124`; `light_count_pass.rs:96-100`; `froxel_pass.rs:130` |
| 0021 | four bind groups | holds: every group index is 0 to 3 | `bind_groups.rs:6-8` |
| 0021 | "`EXPERIMENTAL_RAY_QUERY` ... required"; "the one `unsafe` this ADR adds" | holds; `blas_refit.rs` adds its own `unsafe` blocks under ADR 0020's refit, each with a SAFETY comment whose premises match their registry quotes (see *Registry keys*) | `src/gpu/mod.rs:74-77`, `:170`, `:269`; `blas_refit.rs:198-233`, `:304-315`, `:350-352`, `:486-497`, `:529-546`, `:551-560` |
| 0021 | "`max_acceleration_structures_per_shader_stage` at least 2" | holds: the floor requests 2; `SHADOW_RAY_SCENE` binds exactly two | `src/gpu/mod.rs:303`; `shadow_mask_pass.rs:127-130` |
| 0021 | "`TIMESTAMP_QUERY`" | holds: one query set, `timestamp_writes` per pass, no `write_timestamp` inside encoders or passes | `readback.rs:172-177`, `:501-551` |
| 0022 | "prepare blends the previous and current World transform in f64 into Frame data"; a Moving Instance carries `PreviousWorldTransform`; a spawned or Teleported entity is drawn at its current transform | holds | `prepare.rs:86`, `:308`, `:349-352`, `:383`, `:531`, `:555`; `src/transform/mod.rs:21-31`; `src/state.rs:263`; `acceleration_structures.rs:319-322` |
| 0023 | "a `DirectionalLight` gives illuminance in lux and a `PointLight` luminous intensity in candela" | holds: carried as colour times intensity | `prepare.rs:538`, `:562`; `uniforms.rs:34-36` |
| 0023 | "Exposure is an EV100, which scales luminance by 1/(1.2 · 2^EV100)" | holds | `src/camera/mod.rs:48-49`; `prepare.rs:292-296`; `uniforms.rs:49-50` |
| 0023 | "pre-exposes colour before fp16 stores it, and every path clamps it to 65504" | holds for the one colour this module produces, the background clear, pre-exposed in prepare and clamped where it becomes the clear; the lit pass's clamp is the shaders section's | `prepare.rs:300-307`; `targets.rs:337-341` |
| 0023 | "the tonemapping pass then maps it through GT7 for display" | holds, Rust side; the GPU test checks five colours against the sample implementation's outputs within one step | `tonemapping_pass.rs:53-83`, `:213-281` |
| 0023, checklist | the tonemapping pass alone encodes sRGB into 8-bit UNORM targets; no target is an sRGB format; the dither lands on 8-bit steps | holds: `debug_assert!(!target_format.is_srgb())`; the surface is `Bgra8Unorm` or `Rgba8Unorm`; a 64x64 `R8Unorm` mask, the GPU test showing a flat grey over three codes keeping its mean | `tonemapping_pass.rs:14-25`, `:49-52`, `:84-99`, `:290-351`; `src/gpu/mod.rs:443-460` |
| checklist | a binding written once in `bind_groups.rs`, declared into WGSL from Rust | holds: no `@group` in this module's Rust; each shader module lists its groups | `bind_groups.rs:497-524`; `shaders/mod.rs:88-135` |
| checklist | the indirect buffer is not read-write bound in an active group | holds: the stand-in is bound before `dispatch_workgroups_indirect(&frame_draw)`; the main pass's indirect draws run with groups holding no `frame_draw` | `culling_pass.rs:145-149`, `:319-321`; `bind_groups.rs:20-42`, `:48-65`, `:70-97`, `:269-301` |
| checklist | no GPU buffer grows at runtime; a request past a capacity is refused and logged; only per-frame staging grows, bounded and asserted | holds: every buffer sized at creation; draw and Froxel overflows counted and warned; refit scratch made once, later BLASes refused with one error; bends capped at `BEND_CAPACITY`; staging bounded by the six buffers `upload` writes | `culling_pass.rs:85`, `:172-222`, `:277-283`; `froxel_pass.rs:39`, `:67-81`; `readback.rs:401-420`; `blas_refit.rs:188-196`; `bend_pass.rs:19`; `prepare.rs:636-638`; `staging.rs:67-74`; `renderer.rs:342-355`, `:535-547` |
| checklist | Frame stats see new GPU memory; acceleration structures by the measured per-adapter estimate; the readback's buffers stay out | holds: targets, culling, Froxel, tonemapping, shadow mask, bend, refit and dynamic TLAS marker buffers are summed; `AccelerationStructureBytes` is chosen by vendor and backend; `GpuStatsReadback` has no `gpu_memory_bytes` | `targets.rs:362-378`; `culling_pass.rs:358-365`; `froxel_pass.rs:145-147`; `tonemapping_pass.rs:141-143`; `shadow_mask_pass.rs:396-401`; `bend_pass.rs:117-119`; `blas_refit.rs:148-163`, `:462-464`; `acceleration_structures.rs:31-98`, `:431-433`; `renderer.rs:756-797` |
| checklist | per-frame GPU writes add to `uploaded_bytes` | holds: the Instance, light and uniform writes, `instance_slot_count`, and the dynamic TLAS's staged instance records (72 bytes on Metal, 64 elsewhere, per filled entry) | `renderer.rs:521-551`, `:611-613`; `culling_pass.rs:180-184`; `acceleration_structures.rs:288-297`, `:339-347` |
| checklist | whatever changes what an Instance holds makes prepare write its slot; other slots keep what earlier frames wrote | holds; a failed upload is recovered by rewriting every slot, free ones included | `prepare.rs:81-88`, `:347-353`, `:422-426`; `renderer.rs:554-559`, `:744-746`; `src/state.rs:260-262` |
| checklist | no `unwrap()` on a GPU operation in the render path | holds: every `unwrap` and `expect` in the module is under `#[cfg(test)]`; map, poll and `get_mapped_range` failures are logged | `readback.rs:473`, `:477`; `blas_refit.rs:140-142`, `:498-501` |
| checklist | matrices column-major, `mat3x3` and `array<vec3>` padded per column or element | holds for the CPU-written structs, which `shader_tests.rs` compares with their WGSL twins; `FrameDraw` (72 bytes) and `LightCounts` are GPU-written, so the twin check skips them by design, and their layouts match `culling.wgsl` and `common.wgsl` by inspection | `uniforms.rs`; `instance.rs:36-45`; `culling_pass.rs:82-91`; `froxel_pass.rs:38-43`; `bind_groups.rs:418-420` |

### Spec stories

| Story | Verdict | Where |
|---|---|---|
| 1, constant draws per pipeline and phase | holds: three indirect draws and the tonemapping triangle whatever the Instance count | `main_pass.rs:262`, `:296`, `:330`, `:333`; `renderer.rs:690-694`; test `:961-984` |
| 2, frustum and two-phase Hi-Z in compute | holds, Rust side | `culling_pass.rs:292-322`; `depth_pyramid_pass.rs:101-126` |
| 3, 16, 20, 34, culling-off, 1-sample, brute-force and analytic-shadow equivalences | fall in tooling and the tests (the capture seam); the switches reach the pipelines | `main_pass.rs:167-170`; `culling_pass.rs:242-246` |
| 5, packed index: record high, vertex low six | holds, Rust side | `culling_pass.rs:166-168` |
| 9, Instances and triangles drawn read back | holds: `FrameDraw` copied into the readback slot at `end_frame`, read into `GpuFrameStats` | `culling_pass.rs:82-110`; `readback.rs:275-276`, `:399-400`, `:457-471` |
| 10, only changed Instances written; camera motion writes none | holds: the camera is neither an Instance field nor a `Changed` filter | `prepare.rs:81-88`; `instance.rs:38-45` |
| 11, Frame data lists no draws | holds | `prepare.rs:18-45` |
| 12, draw overflow draws what fits, counted and warned | holds | `culling_pass.rs:85`, `:277-283`; `readback.rs:401-408`; test `renderer.rs:994-1052` |
| 13, the 4x MSAA reverse-Z pre-pass seeds Hi-Z and depth equal | holds | `main_pass.rs:108`, `:173`; `targets.rs:288-297` |
| 14, every Golden at 4x MSAA, sub-pixel wires anti-aliased without temporal accumulation | holds in code; the image shortfall is `DIFFERENCES.md`'s "4x MSAA against supersampling" (see *Open issues read*) | `targets.rs:16`; `main_pass.rs:91-95` |
| 15, 4x colour memoryless and resolved in-pass, asserted | holds; the assertion is wgpu-core's at pass begin | `targets.rs:163-168`, `:345-357` |
| 17, directional lights are components, direction from the World transform | holds: the forward axis of the blended transform | `prepare.rs:527-541` |
| 18, the hard-coded light gone | holds | `prepare.rs:519-570`; test `:1161-1168` |
| 19, point lights culled into Froxels, each list against its bounds | holds, Rust side | `froxel_pass.rs:12-29`, `:118-131` |
| 22, 27, inverse square, range window, rays skipped where a light adds nothing | fall in shaders | |
| 25, one constant ambient irradiance and background radiance per scene, in the decided units | holds | `prepare.rs:300-307`, `:317-322`; `targets.rs:331-358` |
| 26, every light that reaches a pixel traces one hard ray | holds, Rust side: one dispatch over every tile, a mask of 59 bits; the per-light loop is the shaders section's | `shadow_mask_pass.rs:355-360` |
| 28, one compacted BLAS per Mesh at load | falls in cook with assets (`src/assets/pools.rs`); a refitted BLAS is built `ALLOW_UPDATE` and never compacted, as the refit needs | `src/assets/pools.rs:44-46`, `:316-323` |
| 29, static TLAS at load, anchored there, never re-anchored, 100 km check | holds; the test places the load anchor 100 km out | `acceleration_structures.rs:140-141`, `:504`, `:649-725` |
| 30, dynamic TLAS rebuilt each frame at a fixed 10,000 | holds | `acceleration_structures.rs:19`, `:305-371` |
| 31, Moving Instance exactly when carrying its previous World transform | holds | `prepare.rs:397`; test `renderer.rs:879-919`; `acceleration_structures.rs:319-322` |
| 32, Moving Instances past 10,000 left out, counted and warned | holds | `prepare.rs:436-452`; `src/state.rs:273`; test `renderer.rs:926-960` |
| 33, a static spawn or despawn after load rebuilds the static TLAS that frame | holds, tested | `acceleration_structures.rs:143-179`, `:712-714` |
| 35, 53, 57, the dynamic TLAS ceilings, the shadow-ray line, the crowd's two forms against their ceilings | fall in tooling (bench); the timings they need exist, the rebuild form's GPU build excepted (L1-R05) | `renderer.rs:52`, `:64-70`, `:594-600`; `readback.rs:588-592`, `:655-657` |
| 36, acceleration-structure bytes from measured per-backend sizes, labelled estimates | holds | `acceleration_structures.rs:73-98`; `renderer.rs:779`, `:788-797`; `src/stats/mod.rs:46-49` |
| 39, the HDR resolve keeps bright edges from aliasing while colour stays memoryless | holds, Rust side: the clear is weighted as the lit pass weights | `targets.rs:1-7`, `:331-358` |
| 42, the Instance buffer fixed at 524,288, one constant | holds | `instance.rs:1-4`; `renderer.rs:333-339`; `culling_pass.rs:187`, `:220` |
| 43, a spawn past capacity refused and logged, nothing written | holds | `prepare.rs:156-175`, `:376-380`; test `renderer.rs:771-804` |
| 44, every M3 GPU buffer at a fixed provisional capacity, overflow counted and warned | holds | `culling_pass.rs:54-71`; `froxel_pass.rs:24-29`; `renderer.rs:298-303`; `prepare.rs:323-330`, `:436-468`, `:542-568`; `readback.rs:401-420` |
| 50, `crowd` with a Baseline and no Golden | falls in cook with assets and scenes; the figures' bounds hold every bend, which the dynamic TLAS's box relies on | `src/scenes/mod.rs:947-958`; `acceleration_structures.rs:308-311` |
| 51, GPU time per pass and a frame total | holds; one deliberate exception: the light count pass, which only the `count_point_lights` switch runs, is untimed and outside the frame's span, so a Baseline recorded with it on would omit it | `readback.rs:576-672`; `light_count_pass.rs:92`; `renderer.rs:686-688` |
| 52, the gate from the first culling dispatch to the end of the lit pass, rays off | holds: untimed passes excluded; the Froxel pass is placed after the first culling dispatch so the gate counts it | `readback.rs:664-669`, `:810-835`; `renderer.rs:627-633` |
| 58, render switches only capture and bench pass | holds: the window passes `RenderSwitches::default()` | `renderer.rs:72-161`, `:359`, `:421`; `src/lib.rs:182`; `src/bin/praxis.rs:454`, `:744` |

### Registry keys

Every key cited in the module was read against its registry quote, and
the wgpu, wgpu-core and wgpu-hal 30.0.1 sources in the cargo registry
where a quote is load-bearing. All 42 match. By file:

| File | Keys | Verdict |
|---|---|---|
| `staging.rs:22`, `readback.rs:36-39` | `wgpu-30-handle-mapping`, `wgpu-30-submit-maintains`, `wgpu-30-map-after-submission` | matches: a map requested once the copy is seen complete is finished by the next submit's maintain, which allocates the callback list, as the `MAP_LAG` reasoning says |
| `readback.rs:78`, `:630` | `metal-feature-set-tables-sample-buffers`, `apple-metal-fence-per-stage` | matches: one query set of 7 x 24 = 168 queries within the 4,096 of one sample buffer; a render pass's start may precede the end of the passes it waits on, so `counted_begin` takes the later, tested at `readback.rs:844-880` |
| `culling_pass.rs:48-52`, `:145-149` | `wgpu-types-30-default-workgroups-per-dimension`, `wgpu-core-30-dispatch-usage-scope` | matches: `MAX_WORKGROUPS_PER_DIMENSION = 65_535` asserted against the device; the stand-in group covers the indirect buffer's per-dispatch usage scope |
| `targets.rs:28-33`, `:47-52` | `salvi-2012-sbaa-surfaces`, `bt709-6-luma`, `bt2087-rec709-to-xyz` | matches: `SHADOW_MASK_SURFACES = 2`; the luma weights `[0.2126, 0.7152, 0.0722]` declared into WGSL at `shaders/mod.rs:248` |
| `tonemapping_pass.rs:20-25` | `peters-blue-noise` | matches, Rust side: the 64x64 mask compiled in, the test at `:187-200` checks it is the asset's red channel with each of 256 levels 16 times; the triangular remap is the shaders section's |
| `bind_groups.rs:99-106` | `wgpu-30-tlas-dependency-per-instance`, `wgpu-30-use-tlas-per-flush`, `wgpu-30-submit-tlas-dependencies` | matches: `SHADOW_RAY_SCENE` is declared by the shadow mask module alone; `DYNAMIC_TLAS_MARKERS` binds the dynamic TLAS only |
| `shadow_mask_pass.rs:38` | `vulkan-standard-sample-locations` | matches: `SAMPLE_OFFSETS` are the four standard locations with a top-left origin; the test checks the backend's depth target against them |
| `acceleration_structures.rs:26-29`, `:508-509` | `pci-vendor-id-nvidia`, `vulkan-vendorid-pci`, `dxgi-adapter-desc2-vendorid`, `wgpu-30-dx12-vendor-id`, `vulkan-vendor-id-mesa`, `lavapipe-vendor-id-mesa` | matches: `NVIDIA_VENDOR = 0x10DE` against `AdapterInfo::vendor`, the PCI vendor ID on both backends; the lavapipe test uses 0x10005 |
| `acceleration_structures.rs:225`, `:290-295` | `wgpu-30-tlas-instance-staging`, `metal-indirect-instance-descriptor`, `vulkan-acceleration-structure-instance`, `d3d12-raytracing-instance-desc` | matches: wgpu-core stages the filled entries only; 72 bytes per entry on Metal, 64 on Vulkan and DX12 |
| `acceleration_structures.rs:375`, `:382` | `wgpu-30-command-encoder-pool`, `wgpu-hal-30-metal-command-state` | matches: the encoder pool is LIFO, so `prepare_encoder`'s swap rests on it; `CommandState::reset` keeps its allocations |
| `acceleration_structures.rs:445` | `msl-4.1-sin-fast-accuracy` | matches the quoted 2^-13 within ±π; the code grows the box by `BEND_ROUNDING_SHARE`, 2^-8 of the largest coordinate, stated as an estimate with no oracle (see *Notes for the verifier*) |
| `blas_refit.rs:4`, `:6`, `:41` | `wgpu-30-metal-refit`, `wgpu-30-blas-rebuild-only`, `wgpu-issue-9215` | matches: `AccelerationStructureBuildMode::Update` reaches Metal's refit; wgpu's own API rebuilds only; the shared event and the CPU-wait fallback exist for the quoted gap |
| `blas_refit.rs:206-232` | `wgpu-30-as-hal-mut-safety`, `wgpu-30-mixed-encoding-panics`, `wgpu-30-metal-retain-references`, `metal-hazard-tracking-default`, `wgpu-30-metal-buffer-tracked`, `wgpu-30-tlas-blas-build-order`, `wgpu-30-metal-barriers-empty` | matches: each premise of the SAFETY comment holds as quoted; the refit encoder takes hal commands only and is finished by wgpu; a refit through wgpu-hal changes no build index, so the TLAS validation still passes |
| `blas_refit.rs:297`, `:312`, `:531`, `:535` | `wgpu-30-metal-scratch-alignment`, `wgpu-30-raw-boxed`, `wgpu-30-metal-event-staging`, `wgpu-30-core-flush-submit` | matches: hal reports alignment 1 and the code spaces regions at 256; the boxed raw handle's address holds while the resource lives; the event signal is consumed by the very next submit, and `render` maps nothing between the refit submit and the build submit |

### Open issues read

| Issue | What the code shows | Needs hardware |
|---|---|---|
| A long-lived Vulkan device flips one pixel's shadow verdict on the RTX 2070 SUPER (`town`, frame 1500) | From code alone: the static TLAS's build input is deterministic per State (slots from prepare, transforms composed in f64 and rounded once, `acceleration_structures.rs:143-169`, `:479-487`); the dynamic TLAS, the second-surface and edge lists are filled by atomics but each listed pixel writes only its own pixel, and the overflow count is a commutative sum, so the Rust side has no order-dependent verdict; the readback ring attributes counts and timestamps by slot and frame number, never by callback order. The one State-dependent branch on the load path, BLAS compaction readiness (`src/assets/pools.rs:403-409`), the issue's own probe ruled out. No ADR clause is shown broken by reading; nothing filed. | Yes: naming the device state needs the RTX, with Vulkan validation's GPU-assisted and synchronisation checks around the static TLAS build and the first shadow mask dispatch |
| `town`'s sub-pixel wires, railings and pigeons alias into dots | ADR 0008 asks for spatial anti-aliasing and ADR 0019 decides 4x MSAA; the code implements 4x on every main pass attachment (`targets.rs:16`, `main_pass.rs:91-95`) and `DIFFERENCES.md` lists the shortfall under "4x MSAA against supersampling". No ADR clause is broken by what the issue describes; nothing filed. Its placement is a technique decision for a triage. | No |

### Findings

| id | claim | severity | where | evidence | expected |
|---|---|---|---|---|---|
| L1-R01 | The surface normals target, `Rgba8Unorm` at 4x MSAA, is a main-pass attachment format the Capability floor never checks, so ADR 0021's "every attachment format the main pass uses" clause is broken and its format list is stale. | `correctness`; one of three attachment formats unchecked; zero adapters affected today, since wgpu-types 30.0.1 guarantees `MULTISAMPLE_X4` on `Rgba8Unorm` | `targets.rs:207-212`; `main_pass.rs:123-127`; `src/gpu/mod.rs:256-258`; `docs/adr/0021-capability-floor.md`, Consequences, first bullet | block below | ADR 0021: "4x MSAA on every attachment format the main pass uses, read per format from the adapter's texture format features ... Those formats are `Depth32Float` and the HDR target's `Rgba16Float`" |
| L1-R02 | A Renderable skipped for a Mesh not in the pool, or for a missing Material Handle, loses its Instance slot while it is still a Renderable. | `correctness`; clause "keeps a slot ... until it stops being a Renderable"; no visible defect, affects only skipped Renderables, which in M3 occur only in tests | `prepare.rs:358-375`, `:135-138` | block below | ADR 0003: "each Renderable keeps a slot in the Instance buffer until it stops being a Renderable, and a freed slot is taken again before a fresh one" |
| L1-R03 | Three components carry methods, which ADR 0006's "no behaviour methods" forbids in letter; each is a pure function of the component's own data. | `correctness`; clause "no behaviour methods"; 3 components, 3 methods, called from prepare at 4 sites | `src/components.rs:20-39` (`Transform::to_affine`), `:71-77` (`Bend::drawn_curvature`); `src/transform/mod.rs:11-31` (`WorldTransform::translation`, `drawn`); callers `prepare.rs:383`, `:531`, `:555`, `:606` | block below | ADR 0006: "Components hold data only: no closures, no `Box<dyn Trait>`, no behaviour methods. Behaviour lives in systems or, later, script handlers." |
| L1-R04 | ADR 0003 says prepare writes only new or changed Renderables' slots; prepare also rewrites every Moving Instance every frame, a refinement by ADRs 0020 and 0022 that ADR 0003 does not record. | `drift`; not load-bearing: `CONTEXT.md` records it and the clause's purpose, that camera motion writes no Instance, holds | `prepare.rs:86`; `docs/adr/0003-persistent-scene-transform-buffer.md` | block below | ADR 0003 names the refinement by ADRs 0020 and 0022 as it names ADR 0014's |
| L1-R05 | With the crowd's rebuild form, the dynamic TLAS's build is not timed on the GPU and reads as zero, though ADR 0020 says the build is timed in each frame that builds it and names no form that is not. | `drift`; not load-bearing: one non-default Render switch, an oracle, which no Baseline uses; a reader of ADR 0020 or of a rebuild-form bench is misled about `dynamic_tlas_build` and the GPU frame total | `renderer.rs:34`, `:421`, `:605-607`; `acceleration_structures.rs:394-397`; `readback.rs:588-592` | block below | ADR 0020: "its build's recording and submit, and the build itself on the GPU, are timed in each frame that builds it"; either the ADR names the rebuild form as untimed or `deviations.md` carries the row |

#### L1-R01

Code, `targets.rs:207-212`:

```rust
surface_normals: target(
    "Surface Normals",
    SURFACE_NORMALS_FORMAT,
    SAMPLE_COUNT,
    wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
),
```

with `SURFACE_NORMALS_FORMAT = wgpu::TextureFormat::Rgba8Unorm` at
`targets.rs:45` and `SAMPLE_COUNT = 4` at `:16`; `main_pass.rs:123-127`
names it as the depth pre-pass's colour target, drawn at
`multisample.count = SAMPLE_COUNT` (`:91-95`). The floor,
`src/gpu/mod.rs:256-258`:

```rust
const MAIN_PASS_FORMATS: [wgpu::TextureFormat; 2] =
    [crate::render::DEPTH_FORMAT, crate::render::HDR_FORMAT];
```

and `check_floor` (`:314-349`) tests `MULTISAMPLE_X4`, and
`MULTISAMPLE_RESOLVE` on colour, for those two only. ADR 0021,
Consequences, first bullet: "4x MSAA on every attachment format the main
pass uses, read per format from the adapter's texture format features
(`MULTISAMPLE_X4`), since no device feature covers it ... Those formats
are `Depth32Float` and the HDR target's `Rgba16Float`". ADR 0019, as
amended on 2026-10-28: "with shadow rays on, the depth passes also write
each sample's triangle normal and the normal interpolated over it, 4 bytes
per sample ... the target is allocated either way". The amendment added a
third 4x attachment format and neither the floor's list nor ADR 0021's
sentence followed. No adapter fails today: wgpu-types 30.0.1's guaranteed
format features give `Rgba8Unorm` 4x sampling and resolve
(`texture/format.rs:975`), which is why no Golden or CI run shows it. The
normals target is never resolved, so it needs `MULTISAMPLE_X4` alone; the
floor's rule that a colour format implies resolve (`src/gpu/mod.rs:352-360`)
would over-ask if the format were simply appended, so a fix states per
format what it needs. Not `image`: nothing visible.

#### L1-R02

Code, `prepare.rs:358-375`:

```rust
let Some(mesh) = mesh_pool.get(mesh_ref.0) else {
    tracing::warn!(handle = mesh_ref.0.0, "MeshHandle not in pool, skipping entity");
    instance_slots.free(entity);
    unwritten.insert(entity);
    return;
};
if material_ref.is_none() && mesh.needs_material_handle {
    tracing::warn!(?entity, "Renderable has no Material Handle and a Primitive of its Mesh names no Material, skipping it");
    instance_slots.free(entity);
    unwritten.insert(entity);
    return;
}
```

and the struct's doc at `:135-138`: "Each Renderable's slot in the
Instance buffer, kept until it stops being a Renderable or is not drawn".
ADR 0003: "each Renderable keeps a slot in the Instance buffer until it
stops being a Renderable, and a freed slot is taken again before a fresh
one". `CONTEXT.md`, Renderable: "An entity that has a Transform and a Mesh
Handle", so an entity whose Mesh is not yet, or no longer, in the pool is
still a Renderable, yet its slot is freed (`:363`, `:372`), written
`NO_MESH` (`:427-433`) and, once the Mesh loads, it takes whichever slot
`free.pop()` gives (`:160-161`): two Instance writes and possibly another
slot where ADR 0003 promises one kept slot. Harmless in M3, where a Mesh
leaves the pool only in tests; the "or is not drawn" clause is the code's
own addition. The fix is either an amendment to ADR 0003, which makes this
`drift`, or keeping the slot and writing `NO_MESH` into it without
freeing it.

#### L1-R03

Code, `src/components.rs:20-39`:

```rust
impl Transform {
    pub const IDENTITY: Self = ...;
    pub const fn from_translation(translation: glam::DVec3) -> Self { ... }
    pub(crate) fn to_affine(self) -> glam::DAffine3 { ... }
}
```

`src/components.rs:71-77`:

```rust
impl Bend {
    pub fn drawn_curvature(&self, blend_weight: f32) -> f32 {
        self.previous_curvature + (self.curvature - self.previous_curvature) * blend_weight
    }
}
```

`src/transform/mod.rs:11-31`: `WorldTransform::translation` and
`WorldTransform::drawn(&self, previous, blend_weight) -> Self`, the ADR
0022 blend. ADR 0006: "Components hold data only: no closures, no
`Box<dyn Trait>`, no behaviour methods. Behaviour lives in systems or,
later, script handlers. Pure-data components serialise cleanly for
save/load and let scripts hot-reload without dangling references inside
ECS storage". The three methods are pure functions of the component's
own fields with no side effect, closure or trait object, so the ADR's
two reasons, serialisation and hot-reload, are untouched; the letter of
the clause is not. Constructors and `IDENTITY` are not behaviour and are
not counted. The fix is either an ADR 0006 amendment admitting pure
derivations of a component's own data, which makes this `drift`, or
moving the three into free functions in `prepare.rs` and
`src/transform/mod.rs`. Severity `correctness` for the verifier to
narrow or kill; magnitude is three methods at four call sites.

#### L1-R04

Code, `prepare.rs:81-88`:

```rust
type ChangedRenderableQuery = RenderableQuery<
    Or<(
        Changed<WorldTransform>,
        Changed<MeshRef>,
        Changed<MaterialRef>,
        With<PreviousWorldTransform>,
    )>,
>;
```

ADR 0003: "prepare writes only the slots of new Renderables and of those
whose World transform, Mesh or Material changed, found by bevy_ecs change
detection, through the per-frame staging path, so camera motion writes
no Instance." The `With<PreviousWorldTransform>` arm writes every Moving
Instance each frame whether or not its World transform changed; a
stationary entity carrying the component is rewritten every frame. The
rewrite is what ADR 0020 needs, since the dynamic TLAS is rebuilt each
frame from `frame.instances` (`prepare.rs:38-41`), and what ADR 0022
needs, since the blended transform changes with the blend weight;
`CONTEXT.md`, Moving Instance, says "It is rewritten each frame". ADR
0003 ends "Refined by ADR 0014" and names no refinement by ADRs 0020 or
0022. The clause holds on its purpose; the Finding is the ADR's text.

#### L1-R05

Code, `renderer.rs:34`: "rebuild form, an oracle, times neither its
rebuild nor its build"; `renderer.rs:421`:

```rust
marks_dynamic_tlas_build: switches.crowd_form != CrowdForm::Rebuild,
```

`renderer.rs:605-607` brackets the build with `DYNAMIC_TLAS_BUILD` only
when that flag holds; `acceleration_structures.rs:394-397`: "Records a
marker into `encoder`, timed into `timestamp_writes`; none without them.
wgpu-core checks the first marker's query against the BLASes of the last
build, so a BLAS rebuilt through wgpu since would fail the whole command
buffer, build included."; `readback.rs:588-592`:

```rust
let build = if shape.dynamic_tlas_built {
    time(ticks[0].max(build_floor), ticks[1])
} else {
    Duration::ZERO
};
```

ADR 0020: "Moving Instances form the dynamic TLAS, rebuilt each frame
... its build's recording and submit, and the build itself on the GPU,
are timed in each frame that builds it." The ADR names the proxies form,
the default, and the refit switch; it does not name the rebuild form, so
the clause has no carve-out for it, and `deviations.md` has no row. The
code is deliberate and documented at `renderer.rs:34`, so this is the ADR
drifting from a documented code decision, not a defect in the timing.
Under the rebuild form the frame's rebuild commands also sit in the head
submission without timestamps, so the GPU frame total spans from the bend
pass across the build (`renderer.rs:694-697`) rather than summing the
build apart.

### Notes for the verifier

Not filed, since each needs a run, an owner's reading or belongs to
another lane; recorded so the next audit does not re-find them.

- The Froxel pass runs after the first culling dispatch so that ADR
  0019's gate counts light culling (`renderer.rs:627-633`), where the
  ADR says the lists are built "at the start of the frame". Whether the
  gate is meant to include light culling is a reading of the ADR, an
  owner's call at the triage; nothing reads the lists before the pass.
- Lost-upload recovery (`renderer.rs:554-559`, `src/state.rs:260-262`,
  `prepare.rs:400-401`, `:422-426`) has no test that induces a failed
  staging map, so that the rewritten frame equals an unbroken one is
  unverified. Run: force `FrameStaging::write` to return 0 for one frame
  under a Golden scene and compare the next frame's capture to the
  Golden. An input for the tests lane.
- `BEND_ROUNDING_SHARE = 2^-8` (`acceleration_structures.rs:451`) grows
  the dynamic TLAS's box for the bend's sine chain as an estimate with no
  oracle; no source names the bound it would break. A run that bends
  `crowd`'s figures at their sway curvature and compares the bent
  vertices to the grown box would give the test constant the lane asks
  for.
- The refit marker's ordering after the refits relies on Metal hazard
  tracking of the scratch across an acceleration-structure encoder and a
  compute encoder (`blas_refit.rs:386-389`); confirming it needs the M4
  Pro: a refit-form `crowd` bench whose `blas_refit` line is never zero
  or negative.
- The `AccelerationStructureBytes` figures (`acceleration_structures.rs:46-71`)
  are measurements the code cites to the research note and to wgpu's
  allocator reports; re-measuring them is the performance lane's.
- Story 51: the light count pass is untimed and outside the frame's GPU
  span by design (`light_count_pass.rs:92`, `renderer.rs:686-688`); a
  Baseline recorded with `count_point_lights` on would omit it. None is.
- `targets.rs:101` asserts `height <= 1 << 14` rather than refusing;
  unreachable under the default `max_texture_dimension_2d` of 8,192.
- The long-lived Vulkan device's flipped verdict needs the RTX 2070
  SUPER; see *Open issues read*.

### Counts

| Severity class | Findings |
|---|---|
| `image` | 0 |
| `budget` | 0 |
| `correctness` | 3 (L1-R01, L1-R02, L1-R03) |
| `drift` | 2 (L1-R04, L1-R05) |
| `health` | 0 |

Clause verdicts: 92 rows; `holds` 78, two of them with a Finding on the
clause's text or on one form (L1-R04, L1-R05); `broken` 3 (L1-R01,
L1-R02, L1-R03); `drift` 1 (the ADR 0021 format list, in L1-R01);
`superseded` 0; deferred by the spec 1; falls in another module 8; not
applicable 1. Spec stories: 39 checked, 28 hold in this module, 11 fall
in shaders, cook with assets and scenes, or tooling. Registry keys: 42,
all match.

## cook, assets and scenes

Scope: `src/cook/`, `src/assets/`, `src/scenes/` with `town.rs`,
`src/components.rs`, `src/transform/`, `src/time/`, `src/threading.rs`
and `assets/manifest.json`; 9,487 lines of code, every file read in full,
with the tests that exercise them. Three readers at high effort shared the
module by file, each with the lane brief verbatim: the cooker with the
manifest and the cook tests; the Cooked cache reader, the pools and the
compute pool; the Reference scenes, the components, the transforms and the
fixed loop. Each reader's scope, clause verdicts, stories, registry keys,
oracles and open issues are the three subsections below; their Findings
are one table, numbered in order of class, then of file, and the two
readers who filed the same drift of ADR 0018's status sentence share one
Finding (L1-C06). ADRs read: 0011, 0017, 0018 and 0022 in full, and the
clauses of 0002, 0003, 0006, 0008, 0009, 0013, 0014, 0016, 0019, 0020,
0021 and 0023 that name cooking, Assets, Arenas, Meshlets, BLAS at load,
Reference scenes, components, transforms or the fixed schedule. The one
open issue this module weighed, a renamed Asset's old ID loading its stale
Blob unwarned, is under the cook reader's and the assets reader's *Open
issues read*. `deviations.md` holds no row at this commit. No `cargo`
command was run: every oracle row gives the test's bound and, where the
arithmetic allows, the analytic value, not a measured run.

### cook, with the manifest and the cook tests

Scope: `src/cook/mod.rs`, `mesh.rs`, `texture.rs`, `fetch.rs`, `export.rs`
(3,297 lines), `tests/cook.rs` and `tests/asset_manifest.rs` (1,040),
`assets/manifest.json` and `assets/ATTRIBUTION.md` (111); 4,448 lines,
every file read in full. `src/assets/cooked.rs` (974 lines) was read in
full as the writer's half of the Blob formats: its `to_blob` functions are
what the cooker writes, and they are cited here for that; its reader's
verdicts are the assets reader's. `docs/research/test-assets.md` was read
for the asset policy, `CONTEXT.md` for Asset ID, Blob, Cook key and
Resident tail. ADRs read: 0011, 0017, 0018 in full; 0008 and 0023 for the
clauses that name the cook (Toksvig as cook-time specular anti-aliasing;
the units `export-scene` writes); 0019 for the six-bit vertex index that
sets the Meshlet limit. The `CLAUDE.md` review checklist's cook lines
(cooked Blobs only, Asset IDs never source paths, the `cook` feature) were
read as the operational form of ADR 0018. Where a claim rests on a crate's
behaviour, the cargo registry sources were opened: `meshopt` 0.6.2's
vendored meshoptimizer (`vendor/src/clusterizer.cpp`, `meshoptimizer.h`,
`vertexcodec.cpp`), `intel_tex_2` 0.5.0 (`src/bc7.rs`, `bc5.rs`, `bc4.rs`)
and `ktx2` 0.5.0. Every `where` is `path:line` at the frozen commit, under
`src/cook/` unless given in full. `deviations.md` holds no row at this
commit; `DIFFERENCES.md` describes images, none of which this module
produces. The one open issue this reader weighed, a renamed Asset's old
ID loading its stale Blob unwarned, is under *Open issues read*. No
`cargo` command was run: the oracle column below gives each test's bound
and, where the arithmetic allows, the analytic value, not a measured run.

#### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0011 | "glTF (from Blender) is the authoring source; the runtime loads Blobs" | holds: the cooker alone opens glTF, through the `cook` feature's `gltf` dependency; the runtime reads the Cooked cache | `mod.rs:375-378`; `Cargo.toml:16`, `:41`; `src/state.rs:159-178` |
| 0011 | "a development launch reaches glTF only through the cooker, which cooks what changed before the runtime loads Blobs" | holds: `cook_scene` runs before `PoolLoader` reads the cache; without `cook` nothing is cooked and the cache is read as is | `src/state.rs:159-178`; `mod.rs:124-133` |
| 0011 | "Textures cook into KTX2, a standard container" | holds: `ktx2`'s header, level index and DFD serialisers; the round-trip test reads the Blob back through `ktx2::Reader` | `src/assets/cooked.rs:261-320`, `:841-852` |
| 0018 | "each source file gets a 128-bit GUID, written once into its `assets/manifest.json` entry" | holds: eight entries, each a 32-hex GUID; the manifest test refuses a missing, short or shared GUID | `assets/manifest.json:4`, `:15`, `:26`, `:37`, `:48`, `:59`, `:70`, `:81`; `tests/asset_manifest.rs:176-186` |
| 0018 | "An Asset ID is the first 128 bits of SHA-256 over (GUID, Asset kind, glTF name, Channel for a Texture)" | holds, in that order, each part tagged and length-prefixed; a `shasum` oracle pins one ID; a Texture's name is its image's, not the glTF `texture`'s (see *Notes for the verifier*) | `src/assets/cooked.rs:83-117`, `:792-800`; `mod.rs:380-384`, `:446`; `mesh.rs:369`, `:379` |
| 0018 | "The glTF index replaces the name when the name is missing or not unique" | holds: `asset_names` per kind (images, materials, meshes); the cook seam names an unnamed Material by its index | `mod.rs:481-492`, `:380`, `:385`; `mesh.rs:338-339`; `tests/cook.rs:113`, `:215-216` |
| 0018 | "A cooked Asset is a Blob named by the SHA-256 of its bytes, found through a cook index that maps Asset ID to Blob, and rebuilt when its Cook key changes" | holds: `blobs/<sha256>` and `index/<id>` holding hash and Cook key; `is_fresh` compares the key and requires the Blob present | `src/assets/cooked.rs:694-710`, `:745-755`; `mod.rs:332-336`, `:142`, `:150`, `:163` |
| 0018 | "A glTF mesh renamed in Blender gets a new Asset ID" | holds by construction: the name is hashed | `src/assets/cooked.rs:100-109`; `mesh.rs:379` |
| 0018 | "The cooker warns when an ID that something references no longer exists" | broken in part: only a Material's Textures and a Primitive's Material are checked, each against the never-pruned cache, and a Reference scene's references by name are not seen: L1-C01 | `mod.rs:190-196`, `:197-207`, `:340-353`; `src/assets/cooked.rs:703-710`, `:745-755`; `src/state.rs:202-203`; `src/scenes/mod.rs:580-582` |
| 0018 | "Cook key: SHA-256 over what the Asset cooks from (a Texture's source image, a Mesh's decoded Primitives and their Material IDs, a Material's own Blob; by content ...)" | holds: Texture over source bytes or generated pixels; Mesh over every `Geometry` field, destructured so a new field fails to compile, and the Material ID; Material over its Blob; the seam re-cooks exactly the changed Material, and a touched embedded image exactly its two Textures | `texture.rs:119-141`; `mesh.rs:41-70`; `mod.rs:323-329`; `tests/cook.rs:232-247`, `:651-665` |
| 0018 | "the Cook keys of its dependencies, every cook setting, the format version and the encoder versions" | holds: the normal Texture's key tagged `N`; `texture_zstd_level`, the one setting; `TEXTURE_FORMAT_VERSION`, `MESH_FORMAT_VERSION`, `MATERIAL_FORMAT_VERSION`; `intel_tex_2 0.5.0 basic`, `bevy_mikktspace 1.0.0`, `meshoptimizer 0.25`, vertex codec 0, zstd's version number, all matching `Cargo.lock` and the vendored header; a setting change re-cooks every Texture and no Material | `texture.rs:21`, `:121-131`; `mesh.rs:29-35`, `:43-49`; `mod.rs:325-326`; `Cargo.lock` (`intel_tex_2` 0.5.0, `bevy_mikktspace` 1.0.0, `meshopt` 0.6.2); `meshoptimizer.h:15`; `tests/cook.rs:387-415` |
| 0018 | "The metallic-roughness Texture depends on the normal Texture (Toksvig)" | holds: `with_normal_texture` for glTF and procedural sources, the first Material's when several share the image; the key covers it | `mod.rs:436-443`, `:280-303`; `texture.rs:87-93`, `:128-131` |
| 0018 | "Each dev launch recomputes the keys of the Assets it references ... and cooks only the stale ones" | holds: `cook_scene` reads the scene's source files by GUID and its procedural Assets; the seam shows a launch cooking its scene's sources and nothing else, then nothing | `mod.rs:124-133`, `:229-262`; `tests/cook.rs:610-649` |
| 0018 | "The Cooked cache lives in git-ignored `cooked/` and is never evicted. Deleting it is always safe." | holds: `/cooked/` ignored; no code removes an index file or Blob; the seam deletes the cache and re-cooks | `src/assets/cooked.rs:653`; `.gitignore:22-23`; `tests/cook.rs:249-269` |
| 0018 | "Each Blob starts with a magic and a format version (a Texture Blob is KTX2, with its format version in a key-value entry)" | holds for what the cooker writes: `PXMA` 2, `PXME` 2, `PXTXVERSION` = 2 as a little-endian u32 | `src/assets/cooked.rs:18-30`, `:154-155`, `:271-275`, `:541-542` |
| 0018 | "A mismatch is refused with a re-cook message; there is no migration code" | falls in assets (the reader); the cook seam shows the message after a deleted cache | `src/assets/cooked.rs:639-650`; `tests/cook.rs:257-262` |
| 0018 | "Engine data that no cook format holds exactly is compiled into the binary ... the 64² blue-noise dither mask ... The runtime still opens no source path" | holds, cook side: the manifest lists the PNG; `read_sources` cooks only `.gltf` and `.glb` entries, so no Texture Blob is made of it; the pinning test is the render reader's | `assets/manifest.json:79-89`; `mod.rs:241-243`; `assets/ATTRIBUTION.md:21` |
| 0018 | "Textures are KTX2 with every level zstd-supercompressed, stored smallest mip first, so the Resident tail ... is one contiguous byte range" | holds: levels written in `rev()` order, each `zstd::bulk::compress`ed, `SupercompressionScheme::Zstandard`; the round-trip test checks the offsets fall with the level | `src/assets/cooked.rs:284-293`, `:303`, `:841-851` |
| 0018 | "Rows are stored unpadded" | holds: `encode` writes `blocks_wide × block_rows × block_bytes`, concatenated in row order; `level_bytes` asserts it | `texture.rs:504-509`, `:546`, `:206`; `src/assets/cooked.rs:237-248` |
| 0018 | "the loader re-pitches them to 256 bytes as it writes into the `StagingBelt`" | falls in assets (`src/assets/pools.rs`) | |
| 0018 | "Formats per Channel, without RDO: albedo and emissive BC7 sRGB, normal BC5 (z rebuilt in the shader), metallic-roughness BC5 linear, occlusion BC4" | holds; `intel_tex_2` has no RDO path; the normal's x and y alone reach the encoder | `texture.rs:162-170`, `:421-423`, `:514-541`; `src/assets/cooked.rs:221-234` |
| 0018 | "BC6H waits for HDR" | not applicable | |
| 0018 | "The encoder is `intel_tex_2` at its basic profile" | holds: `opaque_basic_settings`, or `alpha_basic_settings` when any alpha is below 255; BC5 and BC4 take no profile | `texture.rs:492-503`, `:522`, `:531`, `:540`; `intel_tex_2-0.5.0/src/bc7.rs:114`, `:199` |
| 0018 | "BC7 measured 44.7 to 57.2 dB ... 52.86 dB on SciFiHelmet's base colour" | holds as a floor: the seam asserts at least 44.0 dB on every BC7 Texture and prints each Channel's PSNR (see *Oracles*) | `tests/cook.rs:304-346` |
| 0018 | "Mips: a 2×2 box filter in linear light, with normals renormalised per mip" | holds: sRGB decoded by the IEC 61966-2-1 EOTF into f32, each mip the mean of a 2×2 box, encoded back once; the normal Channel renormalised at every mip; mip 0 of albedo and emissive takes the source bytes without a round trip; the checker test shows 0.5 linear landing at 188 | `texture.rs:172-184`, `:195-204`, `:230-240`, `:301-307`, `:333-358`, `:384-391`, `:396-404`; `tests/cook.rs:348-371` |
| 0018, 0008 | "Toksvig widens each mip's roughness from the normal Texture's variance: a cook-time, non-temporal answer to specular aliasing" | holds: the spread carries the unnormalised mean and the variance 1 − \|mean\|² down the normal Texture's chain (law of total variance, so agreeing normals keep zero); `widened_roughness` applies Toksvig's eq. 1 then the 2σ² widening of alpha²; a pure function of the two images, no history | `texture.rs:173-176`, `:188-194`, `:282-298`, `:314-327`, `:362-381`, `:443-468`; `tests/cook.rs:481-520` |
| 0018 | "The cooker refuses a Texture whose size is not a multiple of 4 or exceeds 8192, rather than resizing it" | holds: `MAX_TEXTURE_SIDE = 8192`; tested at 6×4 and 8196×4 | `texture.rs:152-161`; `src/assets/cooked.rs:33`; `tests/cook.rs:373-385` |
| 0018 | "Meshes: a 28-byte vertex (position `Float32x3`, UV `Float32x2`, normal, tangent and bitangent sign as a QTangent in `Snorm16x4`, its w kept non-zero so the sign survives)" | holds: `CookedVertex` asserted 28 bytes; w nudged up to one Snorm16 step before quantising, then negated for a negative bitangent sign, so it rounds to at least ±1 | `src/assets/cooked.rs:391-403`, `:47`; `mesh.rs:303-327` |
| 0018 | "and u32 indices" | holds | `mesh.rs:146`; `src/assets/cooked.rs:493`, `:513` |
| 0018 | "`meshopt` reorders for vertex cache, overdraw and fetch, then encodes, then zstd compresses" | holds, in that order: cache and overdraw per Primitive, fetch over the whole Mesh, `encode_vertex_buffer` and `encode_index_buffer`, zstd level 19 | `mesh.rs:147`, `:154`, `:97`; `src/assets/cooked.rs:512-513`, `:543`, `:501` |
| 0018, 0019 | "Each Primitive's triangles are cut, in that order, into Meshlets of at most 64 vertices, because a packed index carries the vertex within its Meshlet in six bits, and at most 124 triangles" | holds: `MESHLET_MAX_VERTICES = 64`, `MESHLET_MAX_TRIANGLES = 124`, cut in index order after the fetch pass renumbers vertices without reordering triangles; the read-back test checks both limits on every Reference scene Mesh | `src/assets/cooked.rs:40`, `:44`; `mesh.rs:84`, `:96-103`, `:165-197`; test `mesh.rs:603-608` |
| 0018 | "Each Meshlet is the longest run the limits allow, as meshoptimizer's scan builder cuts them" | holds for the longest run; drift on the second half: on a degenerate triangle the two cuts differ, which a code comment records and `deviations.md` does not: L1-C05 | `mesh.rs:158-164`, `:172-182`; `meshopt-0.6.2/vendor/src/clusterizer.cpp:368-370` |
| 0018 | "A Meshlet is stored as one byte, its triangle count; the reader rebuilds its vertex remap and local triangles from its run of the index stream" | holds for what the cooker writes: a `u8` per Meshlet after the Primitives; the rebuild is the reader's and is exercised by the read-back test | `src/assets/cooked.rs:531-537`, `:600-612`; test `mesh.rs:655-685` |
| 0018 | "SciFiHelmet's Mesh Blob is 322,850 bytes this way, 13.82 per triangle" | holds as a ceiling: 13.83 bytes per triangle over 23,358 triangles, skipped when the helmet is not fetched (322,850 / 23,358 = 13.8218) | `tests/cook.rs:677-681`, `:706-720` |
| 0018 | "The Blob also holds the Mesh's bounding box for culling" | holds: the box around every cooked vertex and any box a Primitive adds (the bent figures' rest-shape envelope), six f32s after the counts; checked against the sources | `mesh.rs:104-116`; `src/assets/cooked.rs:522-524`; test `mesh.rs:632-652` |
| 0018 | "per-Meshlet bounds and cones wait for per-Meshlet tests" | not applicable | |
| 0018, 0017 | "Each Primitive stores its UV texel density" | deferred by the spec: `CookedPrimitive` has no such field and nothing under `src/` computes one; the spec moves ADR 0017's residency half, which the density serves, to M4 | `src/assets/cooked.rs:405-413`; `mesh.rs:86-91` |
| 0018 | "A Material is its own Asset: PBR factors plus one Texture Asset ID per Channel. Primitives reference it by ID." | holds: ten factors and five IDs; each Primitive carries `Option<AssetId>`, none on glTF's default Material | `src/assets/cooked.rs:135-176`; `mod.rs:465-477`; `mesh.rs:366-369`; `src/assets/cooked.rs:410` |
| 0018 | "A Chunk's edge is 256 m ... A v1 Chunk is a list of Placements" | deferred by the spec (M4: cooked Chunks and Placements) | |
| 0018 | "The cooker lives in `src/cook/` behind the `praxis cook` command, with its encoder dependencies behind a default-on `cook` feature that a shipping build turns off" | holds: `default = ["cook"]`; `bevy_mikktspace`, `gltf`, `image`, `intel_tex_2` optional behind it; `pub mod cook` under the feature; CI's no-`cook` build checks none of the four links (tooling) | `Cargo.toml:15-16`, `:20`, `:24`, `:37-41`; `src/lib.rs:23-24`; `.github/workflows/ci.yml:115-122` |
| 0018 | "Procedural Reference scenes cook too, under Asset IDs derived from the scene name" | holds: `AssetSource::Scene(name)` tagged `S`; `read_sources` cooks every scene's Materials, Textures and Meshes | `src/assets/cooked.rs:90-93`; `src/scenes/mod.rs:107-121`; `mod.rs:263-317` |
| 0018 | "so Goldens test the cooked path ... CI cooks before the golden test" | falls in tooling; CI runs `cargo run -- cook` before `cargo test` | `.github/workflows/ci.yml:75-78` |
| 0018 | "New dependencies ... `intel_tex_2`, `ktx2`, `meshopt`, `zstd`; `sha2` moves from dev-dependency to dependency" | holds: all five under `[dependencies]`, `sha2` with `default-features = false` | `Cargo.toml:23-27` |
| 0018 | status: "the reader rebuilds their Meshlets (#192), which the renderer does not keep yet" | drift: the Mesh pool keeps them in three Meshlet Arenas, as ADR 0017's own amendment says: L1-C06 | `src/assets/pools.rs:76-78`, `:106-111`; `docs/adr/0018-...md:3` |
| 0017 | "The cooked format (ADR 0011) stores each mip as a separately loadable byte range of the Texture's Blob, with the Resident tail as one range" | holds: one `LevelIndex` entry per mip with its own offset and lengths; smallest first, so the tail is contiguous | `src/assets/cooked.rs:277-293` |
| 0017 | "and each Primitive's UV texel density (ADR 0018)" | deferred by the spec, as above | |
| 0017 | "Textures ... loaded from cooked Blobs with their full mip chains" | holds, cook side: `mips = ilog2(max side) + 1`, every level encoded; the seam checks each Reference scene Texture has that many | `texture.rs:171`, `:178-208`; `tests/cook.rs:315-316` |
| 0023 | "a `DirectionalLight` gives illuminance in lux and a `PointLight` luminous intensity in candela ... ambient term gives illuminance in lux ... background ... cd/m²" | holds where `export-scene` writes them: the components' values copied under fields named `illuminance`, `intensity`, `ambient_illuminance`, `background_luminance`, documented with the key; the shadows export test reads 100,000 lx and 900,000 cd back | `export.rs:98-103`, `:112-121`, `:164-167`, `:226-244`; `src/components.rs:83-84`, `:99-100`; test `export.rs:1036-1060` |
| 0023 | "Exposure is an EV100" | holds: the camera's `ev100` exported as is; the test expects 16.3466 for 100,000 lx to 1 | `export.rs:80-85`, `:214`; test `:1027-1033` |
| 0023 | "Core glTF caps emissive at 1 cd/m² ... Brighter emissives need KHR_materials_emissive_strength, which the cooker does not read yet" | holds: `emissive_factor` alone is cooked | `mod.rs:465-477` |
| 0023 | "Point lights fall off by inverse square inside KHR_lights_punctual's range window" | falls in shaders; the export writes `range` with the key | `export.rs:238-242` |
| checklist | "The runtime loads cooked Blobs only and references Assets by Asset ID, never by source path" | holds: the two glTF-backed scenes name their Meshes by `AssetId::new` with the file's GUID; `DEFAULT_MODEL` is a message string | `src/state.rs:35`, `:202-205`; `src/scenes/mod.rs:580-582` |
| policy | "One manifest records path, source URL, SPDX licence expression, credit text, upstream version, and SHA-256 ... CI fails on an `assets/` file without an entry. Credits are collected in an attribution file. Files whose expression includes an `NC` term ... are marked dev-only" | holds: `deny_unknown_fields` entry with every field; unlisted, absent, mismatched, uncredited, blank and `-NC-` without `dev_only` each a problem, and each problem kind exercised; the three credits appear verbatim | `tests/asset_manifest.rs:13-34`, `:144-216`, `:46-110`; `assets/ATTRIBUTION.md:7`, `:13`, `:19` |
| policy | "Pinned URL plus SHA-256, into a git-ignored cache; never committed" (Tier 3; SciFiHelmet fetched) | holds: `fetched: true` entries; a download whose SHA-256 differs is refused before anything is written; a file is written beside its place and renamed in; `/assets/SciFiHelmet/` ignored | `assets/manifest.json:11`, `:22`, `:33`, `:44`, `:55`, `:66`; `fetch.rs:102`, `:109-126`; `.gitignore:16-17`; tests `fetch.rs:222-279` |

#### Spec stories

| Story | Verdict | Where |
|---|---|---|
| 4, Meshlets of at most 64 vertices into each Mesh Blob with meshopt | holds in substance: `meshopt` orders and encodes; the cut is the engine's scan in `meshopt_buildMeshletsScan`'s rule, since `meshopt` 0.6.2 binds that builder only as an `unsafe` extern, as ADR 0018 (amended) records | `mesh.rs:97`, `:147`, `:154`, `:165-197`; `meshopt-0.6.2/gen/bindings.rs:588`; `src/clusterize.rs:94`, `:137`, `:181` (the safe wrappers, none a scan) |
| 6, the 28-byte vertex pulled from the Arena against the cooker's encoder | holds, cook side: `CookedVertex` is the layout the shader decodes; the decode oracle is the shaders section's | `src/assets/cooked.rs:391-403`; `mesh.rs:303-327` |
| 7, each Primitive draws with its own Material | holds, cook side: the Material ID per Primitive in the Blob | `mesh.rs:81-91`; `src/assets/cooked.rs:525-530` |
| 8, SciFiHelmet's cooked Mesh Blob measured once and kept as a ceiling | holds: 13.83 bytes per triangle, skipped when not fetched | `tests/cook.rs:677-681`, `:710-719` |
| 46, `town` procedural at cook time, CI needs no fetch | holds, cook side: `Town` names no source file and its Meshes and Materials cook as procedural Assets; the generator is the scenes reader's | `src/scenes/mod.rs:126-139`; `mod.rs:263-317`; `src/scenes/town.rs:907`, `:924` |
| Implementation decision, Meshlet cook: "gains each Meshlet's vertex remap and local triangles" | superseded by ADR 0018 as amended on 2026-10-19: one byte per Meshlet, the remap and local triangles rebuilt by the reader | `src/assets/cooked.rs:531-537`, `:425-484` |
| Implementation decision, Meshlet cook: "plus a bounding volume per Mesh ... The format version bumps" | holds: `bounds` in the Blob; `MESH_FORMAT_VERSION = 2`, version 1 refused | `src/assets/cooked.rs:37`, `:522-524`, `:915-929` |
| Testing decision, Cook: Meshlet validity (every triangle in exactly one Meshlet, at most 64 vertices, every Meshlet triangle mapping back to its Primitive's) | holds: `check_meshlets` walks each Primitive's index stream against its Meshlets' triangles, asserts none left over, the limits, and the corner-wise set equality with the source | `mesh.rs:591-653`; `:659-685` |
| Testing decision, Cook: SciFiHelmet's bytes per triangle, skipped when not fetched | holds | `tests/cook.rs:710` |
| Testing decision, prior art: the QTangent round trip | holds: 4 Snorm16 steps over 264 frames, both signs, and the near-half-turn cases | `mesh.rs:432-471` |

#### Registry keys

Every key cited in the module was read against its registry quote; the
vendored meshoptimizer 0.25 and `intel_tex_2` 0.5.0 sources were opened
where a quote is about a crate. All 13 cited in these files match. The
four `ktx2-*` keys sit in `src/assets/cooked.rs`, on the writer this
module calls, and were checked for that; `hoffman-2013-f0` and
`bevy-lux-constants` are cited in `src/scenes/mod.rs`, not here, and are
the scenes reader's; both were glanced at and match.

| File | Keys | Verdict |
|---|---|---|
| `mesh.rs:27` | `meshoptimizer-0.25-vertex-codec` | matches: `vertexcodec.cpp:125` `static int gEncodeVertexVersion = 0;` and `:1781` passes it from `meshopt_encodeVertexBuffer`; the Cook key names "vertex codec 0" |
| `mesh.rs:162` | `meshoptimizer-0.25-scan-meshlets` | matches: `clusterizer.cpp:368-370` is the quoted `used_extra` and split condition, called per triangle from `meshopt_buildMeshletsScan` at `:1289`; `av`, `bv`, `cv` are references into `used[]`, so a repeated corner is counted twice, as the comment says; the code's cut differs there (L1-C05) |
| `mesh.rs:258-259` | `gltf-2.0-texture-origin`, `gltf-2.0-normal-texture-convention` | matches: `v` is flipped to `1 − v` for MikkTSpace so the bitangent runs up the image, where glTF's normal Texture +Y points; the plane test shows tangent +X, bitangent −Z along decreasing `v` |
| `mesh.rs:278` | `gltf-2.0-mikktspace-tangents` | matches: `bevy_mikktspace::generate_tangents` over positions, normals and `TEXCOORD_0`, always, since glTF `TANGENT` is not read (see *Notes for the verifier*) |
| `mesh.rs:299`, `export.rs:617` | `gltf-2.0-tangent-bitangent` | matches: the frame's columns are the tangent, `normal × tangent`, the normal, with w's sign the bitangent sign; `tangent_frame` decodes w's sign back into the glTF tangent's w |
| `texture.rs:397` | `khronos-dfs-srgb` | matches: `≤ 0.0031308 → × 12.92`, else `1.055 x^(1/2.4) − 0.055`; the decode at `:234-238` is its inverse at the 0.04045 knee |
| `texture.rs:439` | `gltf-2.0-roughness-mapping` | matches: alpha² = roughness⁴ in `widened_roughness`, undone by `powf(0.25)` |
| `texture.rs:448` | `toksvig-2004-variance` | matches: `(1 − \|mean\|)/\|mean\|` written as `variance / ((1 + mean_length) · mean_length)` with `variance = 1 − \|mean\|²`, algebraically eq. 1 and free of the cancellation |
| `texture.rs:457`, `:459` | `kaplanyan-2016-ndf-filtering`, `tokuyoshi-2019-roughness-matrix` | matches: alpha'² = alpha² + 2σ² from eqs. 14 and 15; the 0.18 cap is named and not applied, with the reason, which the second quote's κ confirms is a cap on the derivative-estimated term; only alpha² is capped at 1 |
| `export.rs:230`, `:238` | `khr-lights-punctual-units` | matches: `illuminance` on the directional entry, `intensity` in candela on the point entry, copied from components documented in lux and candela |
| `export.rs:241` | `khr-lights-punctual-range` | matches in what the export writes: `range` in metres, the window itself being the shader's |
| `src/assets/cooked.rs:26`, `:255-259` (writer) | `ktx2-level-order`, `ktx2-supercompression-per-level`, `ktx2-level-alignment`, `ktx2-key-value-alignment` | matches: levels written smallest first; each level compressed on its own; no padding between levels under scheme 2; `PXTXVERSION` plus NUL is 12 bytes, a multiple of the u32 value's 4, asserted at `:857-864` |
| `src/scenes/mod.rs:471`, `:519` (scenes reader) | `bevy-lux-constants`, `hoffman-2013-f0` | match on a glance: `SUN_ILLUMINANCE = 100_000.0`; `SWEEP_BASE_COLOR = [1.0, 0.71, 0.29, 1.0]`, the quoted linear F0 of gold |

#### Oracles

Every cook oracle runs on the CPU in f32 (or f64 in the export test), so
no WGSL accuracy floor applies. No test was run by this reader; the
measured column gives the test's bound and, where the arithmetic allows,
the analytic value the bound was set against.

| Function | Test, constant | Literature number, key | Measured error | Verdict |
|---|---|---|---|---|
| `widened_roughness` (`texture.rs:443-468`) | `toksvig_widens_each_metallic_roughness_mip_by_its_normal_textures_variance`, `tests/cook.rs:481-520`; `ROUGHNESS_TOLERANCE = 0.01` (`:430`) on 8-bit roughness after BC5; red-capable: the widening must exceed 10 × tolerance at every mip above 0 (`:510`) | σ² = (1 − \|Nₐ\|)/\|Nₐ\| [`toksvig-2004-variance`]; α'² = α² + 2σ² [`kaplanyan-2016-ndf-filtering`]; α = roughness² [`gltf-2.0-roughness-mapping`] | within 0.01 at every texel of five mips when the test passes; the expected value is the composed formula with the analytic mean length cos(tilt)^m, so the test checks the implementation of the quotes, not the angle-for-slope step (see *Notes for the verifier*) | matches its sources |
| `qtangent` (`mesh.rs:303-327`) | `qtangents_decode_to_their_frame_within_snorm16_precision`, `mesh.rs:452-471`; `4.0 * STEP`, `STEP = 1/32767 ≈ 3.05e-5`, so 1.22e-4 per component of the decoded normal and tangent, over 260 rotations about four axes and four near-half-turns, both signs | Snorm16 quantisation: half a step per component, two quaternion products per rotated axis, plus the one-step nudge on w (ADR 0018, "its w kept non-zero") | ≤ 1.22e-4 when the test passes; the bitangent sign exact | holds |
| `cook` plane frame (`mesh.rs:122-156`) | `a_plane_has_its_uv_aligned_tangent_frame`, `mesh.rs:501-513`; `4.0 * STEP` | glTF's bitangent = cross(N, T) · w [`gltf-2.0-tangent-bitangent`]; +Y up in normal Textures [`gltf-2.0-normal-texture-convention`] | normal within 1.22e-4 of +Y, tangent of +X, sign +1 | holds |
| `mesh_glb` and `tangent_frame` (`export.rs:616-722`) | `shadows_exports_its_constants_and_its_cooked_mesh`, `export.rs:1000-1161`; `4.0 * STEP` in f64 (`:993`) against the analytic sphere's and plane's normals, unit tangents perpendicular to them | the analytic sphere (normal out from the centre 1 m up) and plane (+Y, +X) | ≤ 1.22e-4 when the test passes | holds |
| mip filter in linear light (`texture.rs:230-358`) | `a_black_and_white_checker_filters_in_linear_light`, `tests/cook.rs:348-371`; `abs_diff(188) <= 2` on mip 1 of an 8×8 checker | sRGB EOTF⁻¹ of 0.5: 1.055 · 0.5^(1/2.4) − 0.055 = 0.7354 → 187.5 of 255 [`khronos-dfs-srgb`]; an average of the encoded values would give 128 | analytic 187.5 against the asserted 188 ± 2: 0.5 of the bound before BC7's error | holds |
| `encode`, BC7 (`texture.rs:479-547`) | `reference_scene_textures_cook_every_mip_and_keep_their_quality`, `tests/cook.rs:304-346`; `decibels >= 44.0` on every BC7 Texture of the seam's scenes, SciFiHelmet's when fetched; BC5 and BC4 PSNRs printed, not asserted | ADR 0018: BC7 44.7 to 57.2 dB on DamagedHelmet, 52.86 dB on SciFiHelmet's base colour; BC5 44.11, 42.71 and BC4 44.06 dB, which is why no BC5 or BC4 floor is asserted | printed to stderr per Texture at test time; not run here | holds as a floor |
| `CookedMesh::to_blob` size (`src/assets/cooked.rs:511-545`) | `every_reference_scene_mesh_cooks_and_reads_back`, `tests/cook.rs:677-720`; `HELMET_MESH_BYTES_PER_TRIANGLE = 13.83` over `HELMET_TRIANGLES = 23_358` | ADR 0018: 322,850 bytes, 13.82 per triangle | ceiling 0.008 above the ADR's 13.8218 | holds |
| `weld` (`mesh.rs:201-222`) | `sci_fi_helmet_welds_to_14085_vertices`, `mesh.rs:690-703`; exact | `docs/research/asset-compression-and-cooked-data.md:35`: "welded on position, UV and normal (14,085 vertices ...)" | exact count | holds |
| `AssetId::new` (`src/assets/cooked.rs:83-117`, the reader's file) | `asset_id_is_truncated_sha256_of_its_encoding`, `:792-800`; exact | SHA-256 of the documented encoding by `shasum -a 256`, truncated to 128 bits | exact | holds |

#### Open issues read

| Issue | What the code shows | Needs hardware |
|---|---|---|
| #221, "A renamed Asset's old ID still loads its stale Blob, unwarned", weighed against ADR 0018's "A glTF mesh renamed in Blender gets a new Asset ID. The cooker warns when an ID that something references no longer exists." and "never evicted" | The first sentence holds: the name is in the hash (`src/assets/cooked.rs:100-109`, `mesh.rs:379`). "Never evicted" holds: `CookedCache::write` adds or replaces an index file and never removes one (`:745-755`), and no other code touches `index/`. The warning clause is implemented for two reference kinds only: a Material's Textures (`mod.rs:190-196`) and a Primitive's Material (`:197-207`, `:340-353`), each tested with `cache.entry(id).is_none()`, that is against the Cooked cache, not against what this cook produced, so a stale index entry satisfies it. The references the issue describes are not seen at all: a Reference scene names a glTF Mesh in code (`src/state.rs:202-203` `AssetName::Name("SciFiHelmet")`; `src/scenes/mod.rs:580-582` `"NormalTangentTest_low"`), `cook_scene` receives only the scene's source GUIDs (`mod.rs:130-131`), and `Sources` carries no list of the IDs a scene will load. After a rename the new Mesh cooks under its new ID, the old ID's index file stays, and `CookedCache::read` (`:713-727`) returns the old Blob to the loader with no staleness check; the cook report says `cooked: 1, unchanged: n` and nothing else. The clause is therefore broken in part, as the issue states; filed as L1-C01 so that the clause table has its exception, with the expectation that it inherits the issue's placement (fog, M4's charting, the owner's). What would close it in M4 is already sketched by the code: a cook knows every ID it wrote or found fresh (`CookReport`), so an index entry it neither wrote nor confirmed is a candidate for a warning or for pruning, once Chunks and Scenes give the cooker the full reference set. | No |

### assets

Scope: `src/assets/` and the compute pool: `src/assets/mod.rs`,
`cooked.rs`, `loader.rs`, `material.rs`, `mesh.rs`, `pools.rs` and
`src/threading.rs`; 2,742 lines, every file read in full, test modules
included, with `tests/compute_pool.rs` (8 lines) and the blocks of
`tests/cook.rs`, `tests/frame_stats.rs`, `tests/reference_scenes.rs` and
`tests/conventions.rs` that exercise the Cooked cache and the pools. ADRs
read in full: 0002, 0009, 0016, 0017, 0018, 0020, 0021; the clauses of
0003, 0006, 0011, 0013, 0014, 0019, 0022 and 0023 that name cooking,
Assets, Arenas, Meshlets, BLAS at load or the compute pool. The `CLAUDE.md`
review checklist was read as the operational form of those clauses.
`src/cook/mod.rs` and `src/cook/mesh.rs` were opened only where the
reader's format or the open issue needed the writer's side;
`src/render/renderer.rs`, `staging.rs`, `bind_groups.rs`, `src/gpu/mod.rs`
and `src/render/shaders/main.wgsl`, `shading.wgsl` only where they consume
what the pools hand over. The crate sources of `meshopt` 0.6.2,
`offset-allocator` 0.2.0, `ktx2` 0.5.0 and `zstd` 0.14.2 were read in the
cargo registry where a clause rests on their behaviour. Every `where` is
`path:line` at the frozen commit, under `src/assets/` unless given in full.
`deviations.md` holds no row at this commit. The one open issue this
reader weighed, the renamed Asset's old ID, is under *Open issues read*.

#### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0002 | "sub-allocations of one shared vertex buffer and one shared index buffer in the Mesh pool" | holds: one vertex Arena, one index Arena, each one `wgpu::Buffer`; the draw's indices are ADR 0019's packed indices and the Meshlet triangle Arena, and the index Arena feeds BLAS builds only (see *Notes for the verifier*) | `pools.rs:74-75`, `:98-105`, `:103-104`, `:433-440` |
| 0002, 0017 | "fixed arenas ... implemented in #145 at a provisional 16 MiB each until the Budget split" | holds | `pools.rs:7-9` (`16 << 20`); `src/render/renderer.rs:340` |
| 0006 | Handles are plain data | holds: `MeshHandle(u32)`, `MaterialHandle(u32)` | `mesh.rs:14-15`; `material.rs:31-39` |
| 0009 | "`bevy_ecs` used on its own, not the Bevy engine" | holds: `bevy_ecs` and `bevy_tasks` 0.18.1 are the only `bevy_*` runtime dependencies | `Cargo.toml:17-18`; `src/threading.rs:5-7` |
| 0011 | "a development launch reaches glTF only through the cooker, which cooks what changed before the runtime loads Blobs" | holds: the loader reads `CookedCache` only; the cook runs under `cfg(feature = "cook")` before it; `gltf` is an optional dependency of that feature | `loader.rs:57`, `:73`, `:84`; `material.rs:266`, `:306`; `src/state.rs:159-180`; `Cargo.toml:16` |
| 0013, checklist | `gpu_memory_bytes` counts every pool buffer and texture; acceleration structures by the per-adapter estimate | holds: the five Arenas and the Mesh table; the SSBO and every Channel's Textures; BLAS bytes split by compacted and uncompacted triangles; all three summed by the Renderer | `pools.rs:471-477`, `:481-489`; `material.rs:228-236`; `src/render/renderer.rs:778-783`, `:793`; `tests/frame_stats.rs:80-83` |
| 0013, checklist | per-frame GPU writes add to `uploaded_bytes` | not applicable to this module: the pools write at load through `LoadStaging`, which is not per-frame (see *Notes for the verifier*) | `pools.rs:265-291`; `material.rs:127-129`, `:250` |
| 0014 | "cooked meshes stay within 128 m of their pivot"; positions f32 in mesh space | holds, reader side: `position: [f32; 3]` mesh-local, bounds mesh-local; the 128 m bound is the cooker's | `cooked.rs:391-403`, `:494-495`; `mesh.rs:27-29` |
| 0016 | "the `ComputeTaskPool` is created with `available_parallelism() − 1` workers (at least 1)" | holds; tested | `src/threading.rs:12-22`; `tests/compute_pool.rs:4-8` |
| 0016 | "the schedule stays `SingleThreaded`" | holds | `src/threading.rs:27-31` |
| 0016 | "bevy_ecs's `multi_threaded` feature is on" | holds | `Cargo.toml:17-18` |
| 0016 | "No QoS class is set" | holds: `TaskPoolBuilder` with threads and a name only | `src/threading.rs:17-20` |
| 0016 | "Asset reads and decoding run on a separate IO pool outside the compute pool ... the main thread copies it into the `StagingBelt`" | deferred by the spec: no IO pool exists; the ADR says "the rest is planned" and the spec moves ADR 0017's residency half to M4; loads decode on the main thread at spawn | `loader.rs:57`, `:84`; `src/state.rs:174-180`; no `IoTaskPool` in `src/` |
| 0016 | "Only the main thread submits, presents or polls" | holds: `compact_blases` submits and polls, called from `Renderer::upload` on the calling thread; the eight-thread test uses a device per thread under `cfg(test)` | `pools.rs:402`, `:421-422`; `src/render/renderer.rs:485-486`; `pools.rs:726-756` |
| 0017 | "Each Channel is one partially bound texture binding array (`TEXTURE_BINDING_ARRAY`, `PARTIALLY_BOUND_BINDING_ARRAY`) in one bind group" | holds: five `TextureViewArray` resources in `MATERIALS`, each with `count` 13,107, bound with the views present | `material.rs:77-84`, `:362-387`; `src/render/bind_groups.rs:48-65`, `:634-647` |
| 0017 | "a Layer is an index into its Channel's array, Layer 0 stays the neutral default" | holds in form: five 1x1 neutral Textures pushed first; the normal Channel's neutral is not a no-op: L1-C03 | `material.rs:156-160`, `:180-199`, `:267`, `:307-309` |
| 0017 | "the fixed 8 Layers per Channel go" | holds: 65,536 / 5 = 13,107 per Channel | `material.rs:15-16`; `src/gpu/mod.rs:293` |
| 0017 | "Residency changes are batched so that bind group is rebuilt at most once per frame" | holds: loads set `bind_group_is_stale`; `refresh_bind_group` rebuilds once, called once per `render` | `material.rs:311`, `:318-340`; `src/render/renderer.rs:582` |
| 0017 | the gate: the rebuild's cost measured at the bound Textures' count | holds: the build is timed and logged with the Texture count | `material.rs:320`, `:329-338` |
| 0017 | "sampled trilinear, the colour Channels at 16x anisotropy and the data Channels at 4x" | holds: two samplers, `Linear` on all three filters, `Repeat`, `anisotropy_clamp` 16 and 4; the shader samples albedo and emissive with `colour_sampler`, the rest with `data_sampler` | `material.rs:28-29`, `:201-202`, `:346-357`; `src/render/shaders/main.wgsl:173-178` |
| 0017 | "Priority is computed on the CPU ... from distance and the Primitive's UV texel density (stored at cook time, per Primitive)" | deferred by the spec (ADR 0017's residency half to M4); no Blob field holds a density | `cooked.rs:406-413`, `:525-530`, `:565-580` |
| 0017 | "Each texture's Resident tail ... never leaves ... evicted only below p − 1" | deferred by the spec: no eviction; every Texture loads whole | `material.rs:117-133` |
| 0017 | "One vertex arena and one index arena, and the Meshlet arenas, each at most 2^31 − 1 bytes" | holds: asserted at creation; the floor's `max_buffer_size` is 2^31 − 1 | `pools.rs:540-545`; `src/gpu/mod.rs:300` |
| 0017 | "sub-allocated with the `offset-allocator` crate" | holds: one `offset_allocator::Allocator` per Arena, in elements; `allocation_size` is the requested count, so `bytes_in_use` is exact (`offset-allocator-0.2.0/src/lib.rs:246`, `:486-491`) | `pools.rs:525-530`, `:553`, `:561-572` |
| 0017 | "the Capability floor's 128 MiB `max_storage_buffer_binding_size` caps that arena" | holds: 16 MiB, pinned by a test against the floor | `pools.rs:9`; `src/gpu/mod.rs:756-760` |
| 0017 | "No growth at runtime: a load that does not fit is refused and logged" | holds: Arena buffers created once; a Mesh past an Arena or the Mesh table returns `None` after an `error!`, every Arena as it was; tested | `pools.rs:188-194`, `:203-229`, `:503-521`, `:547-552`; test `:965-994` |
| 0017 | "The CPU shadow of vertex and index data goes" | holds: a loaded `Mesh` keeps its vertex range and bounds only | `mesh.rs:3-12`; `pools.rs:298-302` |
| 0017 | "Acceleration structures join the ledger: 94 bytes per BLAS triangle (45.5 compacted) ... `gpu_memory_bytes` counts them as estimates" | holds, pool side: triangles move from uncompacted to compacted as each BLAS is compacted; the per-adapter figures are the render section's | `pools.rs:71-73`, `:338-345`, `:416-418`, `:481-489`; test `:696-711` |
| 0017 | "A `TRANSIENT_ATTACHMENT` counts as zero only when `transient_saves_memory` is `Some(true)`" | falls in tooling (`src/stats/`, `texture_bytes`); the pool passes the flag through | `material.rs:228-236` |
| 0017 | "Streaming uploads go through a `StagingBelt`, never `Queue::write_texture`" | holds: every pool write goes through `LoadStaging`, a `StagingBelt` and one encoder; no `queue.write_texture` or `queue.write_buffer` in `src/` outside the per-frame path | `material.rs:128`, `:250`; `pools.rs:575-578`, `:287-291`; `src/render/staging.rs:171-173`, `:186-193`, `:198-251` |
| 0017 | "the engine caps the bytes written into it per frame at the staging Budget, and an upload past the cap waits for a later frame" | deferred by the spec: loads happen at spawn, submitted whole before the first frame; no staging Budget is read | `src/render/staging.rs:274-279`; `src/render/renderer.rs:485` |
| 0017 | "The cooked format stores each mip as a separately loadable byte range ... with the Resident tail as one range" | holds for the layout: the KTX2 level index gives each level its own range, written smallest mip first; reading by range is deferred with streaming | `cooked.rs:277-293`, `:367-381`; test `:841-852` |
| 0018 | "An Asset ID is the first 128 bits of SHA-256 over (GUID, Asset kind, glTF name, Channel for a Texture)" | holds; the oracle recomputed outside the code matches (see *Oracles*) | `cooked.rs:79-117`; test `:792-800` |
| 0018 | "The glTF index replaces the name when the name is missing or not unique" | holds in the encoding (`AssetName::Index`); the choice is the cooker's | `cooked.rs:67-73`, `:104-108` |
| 0018 | "A cooked Asset is a Blob named by the SHA-256 of its bytes, found through a cook index that maps Asset ID to Blob" | holds: `blobs/<sha256>` and `index/<id>` holding hash and Cook key, one file per Asset, written whole and renamed into place | `cooked.rs:655-656`, `:694-700`, `:703-710`, `:744-755`, `:763-783` |
| 0018 | "A glTF mesh renamed in Blender gets a new Asset ID. The cooker warns when an ID that something references no longer exists." | holds as written for an ID absent from the cache; the renamed Asset's old ID keeps its entry: see *Open issues read* | `cooked.rs:703-710`, `:745-755`; `src/cook/mod.rs:190-196`, `:344-352` |
| 0018 | "The Cooked cache lives in git-ignored `cooked/` and is never evicted" | holds: no code removes a Blob or an index file | `cooked.rs:653`; `.gitignore:23`; `cooked.rs:744-755` |
| 0018 | "The runtime loads Blobs only (ADR 0011). Each Blob starts with a magic and a format version (a Texture Blob is KTX2, with its format version in a key-value entry). A mismatch is refused with a re-cook message; there is no migration code." | holds: `PXMA` 2, `PXME` 2, `PXTXVERSION` 2; one `payload` check with no other branch; tested for each kind, M3's first Texture entry included | `cooked.rs:17-37`, `:178-179`, `:324-336`, `:548`, `:639-650`; tests `:804-824`, `:866-900`, `:918-929` |
| 0018 | "Engine data that no cook format holds exactly is compiled into the binary ... the tonemapping pass's 64² blue-noise dither mask" | falls in render (`tonemapping_pass.rs:20-25`, test `:187-200`); no asset module holds it | |
| 0018 | "The runtime still opens no source path" | holds: the only paths the module opens are `cooked/index/<id>` and `cooked/blobs/<hash>`; `ASSETS_DIRECTORY` is reached under `cfg(feature = "cook")` only; `DEFAULT_MODEL` names a path in an error message and an `exists()` check (see *Notes for the verifier*) | `cooked.rs:694-700`, `:704`, `:721`; `src/state.rs:159-172`, `:203-206`; `src/bin/praxis.rs:435` |
| 0018 | "Textures are KTX2 with every level zstd-supercompressed, stored smallest mip first" | holds: the reader requires `Zstandard`, decompresses each level on its own; the writer orders levels `.rev()`, pinned by the test's descending offsets | `cooked.rs:284-292`, `:338-341`, `:367-381`; test `:841-851` |
| 0018 | "Rows are stored unpadded, and the loader re-pitches them to 256 bytes as it writes into the `StagingBelt`" | holds: `levels` are "rows unpadded"; `LoadStaging::write_texture` re-pitches to `COPY_BYTES_PER_ROW_ALIGNMENT` into a belt slice | `cooked.rs:216-217`; `material.rs:127-129`; `src/render/staging.rs:195-251` |
| 0018 | "Formats per Channel, without RDO: albedo and emissive BC7 sRGB, normal BC5, metallic-roughness BC5 linear, occlusion BC4" | holds, reader side: the three formats are the only ones a Texture Blob may hold; the per-Channel assignment falls in cook (the reader does not check a Blob's format against its Channel, see *Notes for the verifier*) | `cooked.rs:221-234`, `:342-350` |
| 0018 | "The cooker refuses a Texture whose size is not a multiple of 4 or exceeds 8192" | falls in cook; the reader mirrors it, and requires the full mip chain | `cooked.rs:33`, `:354-366` |
| 0018 | "a 28-byte vertex (position `Float32x3`, UV `Float32x2`, ... QTangent in `Snorm16x4`, its w kept non-zero)" | holds for the layout, asserted at compile time; the non-zero w is the cooker's, `QTANGENT_SMALLEST_W` declared here | `cooked.rs:45-47`, `:391-403` |
| 0018 | "Meshlets of at most 64 vertices ... and at most 124 triangles" | holds: the reader refuses a run past either limit | `cooked.rs:38-44`, `:444-447`, `:458-461` |
| 0018 | "Each Meshlet is the longest run the limits allow, as meshoptimizer's scan builder cuts them" | falls in cook (`src/cook/mesh.rs:162-181`) | |
| 0018 | "A Meshlet is stored as one byte, its triangle count; the reader rebuilds its vertex remap and local triangles from its run of the index stream, which BLAS builds read" | holds: a byte per Meshlet after the Primitives; `CookedMeshlet::split` numbers vertices by first use with corners sorted, so the index codec's rotation changes nothing; the index Arena is `BLAS_INPUT` | `cooked.rs:503-509`, `:531-537`, `:582-585`, `:600-611`, `:426-484`; `pools.rs:103-105`, `:174-179`, `:307-313` |
| 0018 | "The Blob also holds the Mesh's bounding box for culling" | holds: six f32 after the counts, into `MeshData` | `cooked.rs:494-495`, `:522-524`, `:559-562`; `pools.rs:257-264` |
| 0018 | "the reader rebuilds their Meshlets (#192), which the renderer does not keep yet" | drift: the Mesh pool keeps them in three Meshlet Arenas: L1-C06 | `docs/adr/0018-...md:3`; `pools.rs:106-120`, `:272-286` |
| 0018 | "Each Primitive stores its UV texel density" | deferred by the spec, as the ADR 0017 priority clause above; the Blob has no field for it | `cooked.rs:525-530` |
| 0018 | "A Material is its own Asset: PBR factors plus one Texture Asset ID per Channel. Primitives reference it by ID." | holds: ten factors, five IDs; `CookedPrimitive::material: Option<AssetId>` | `cooked.rs:134-144`, `:147-149`, `:406-410` |
| 0018 | Chunks and Placements | deferred by the spec (M4) | |
| 0018 | "The cooker lives in `src/cook/` ... encoder dependencies behind a default-on `cook` feature" | holds for this module's split: `to_blob` and `write` under `cfg(feature = "cook")`, every `from_blob` and `read` unconditional | `cooked.rs:151`, `:260`, `:510`, `:744` |
| 0018 | "Procedural Reference scenes cook too, under Asset IDs derived from the scene name, so Goldens test the cooked path" | holds, reader side: `AssetSource::Scene(name)` hashed with its own tag; the pbr-sweep sphere is read back from the cache by that ID | `cooked.rs:63-64`, `:89-93`; `tests/frame_stats.rs:156-162`; `src/scenes/mod.rs:113-115` |
| 0018 | "New dependencies ... `ktx2` (Apache-2.0), `meshopt`, `zstd`; `sha2` moves from dev-dependency to dependency" | holds: `ktx2` 0.5.0, `meshopt` 0.6.2, `zstd` 0.14.2, `offset-allocator` 0.2.0, `sha2` 0.11 in `[dependencies]` | `Cargo.toml:23-28` |
| 0019 | "a packed index carries the vertex within its Meshlet in six bits" | holds: `MESHLET_MAX_VERTICES = 64`, enforced on read | `cooked.rs:38-40`, `:458-461` |
| 0019, spec "GPU tables" | "At Mesh load, the Mesh pool writes its Meshlets, each Primitive's Meshlet range and Material ID, and each Mesh's bounds into fixed-capacity GPU buffers. The vertex Arena becomes readable as a storage buffer in the vertex stage, and the Arenas become BLAS build inputs." | holds: `MeshletData` per Meshlet with its Primitive's Material ID or `NO_MATERIAL`; `MeshData` with the Mesh's Meshlet range and bounds at its Handle; vertex Arena `STORAGE | BLAS_INPUT`, index Arena `BLAS_INPUT`; the GPU-side layout read back by a test | `pools.rs:98-105`, `:121-126`, `:233-291`; `mesh.rs:20-44`; test `pools.rs:622-684`; `src/render/renderer.rs:365-374` |
| 0020 | "each Mesh gets a compacted BLAS at load" | holds: `ALLOW_COMPACTION`, built in the load's encoder, compacted once that staging is submitted, the pool keeping the compacted BLAS | `pools.rs:315-329`, `:336`, `:343`, `:393-425`; `src/render/renderer.rs:485-486`; test `:692-718` |
| 0020 | "The `crowd` Reference scene's figures, which the GPU bends each frame, have no BLAS" | holds: `MeshUse::Drawn` pushes `None` | `pools.rs:19-21`, `:303-306`; `loader.rs:64-66` |
| 0020, spec story 28 | the refit form: "gives each figure a BLAS, refitted to its bent shape each frame through wgpu-hal on Metal" | holds, pool side: `REFITTED_BLAS_FLAGS = PREFER_FAST_TRACE | ALLOW_UPDATE`, never queued for compaction, kept with its build descriptor; a wgpu rebuild of the same BLASes exists for the oracle | `pools.rs:43-46`, `:314-323`, `:330-344`, `:349-383` |
| 0020 | the TLAS-after-compaction ordering: "wgpu-core 30.0.1 records no barrier after `compact_blas`'s copy" | holds: `compact_blases` polls to completion before the static TLAS reads the BLASes; tested under GPU load | `pools.rs:385-392`, `:402`, `:421-424`; test `:720-756` |
| 0021 | "Required features: `TEXTURE_BINDING_ARRAY`, `PARTIALLY_BOUND_BINDING_ARRAY` and `SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING` (ADR 0017)" | holds | `src/gpu/mod.rs:263-265` |
| 0021 | "`TEXTURE_COMPRESSION_BC` (ADR 0018 cooks BC only)" | holds: the three Blob formats are BC7, BC5, BC4 | `src/gpu/mod.rs:267`; `cooked.rs:221-234` |
| 0021 | "`EXPERIMENTAL_RAY_QUERY`" covers `create_blas` and `BLAS_INPUT` | holds | `src/gpu/mod.rs:269`; `pools.rs:100`, `:315` |
| 0021 | "The `ANISOTROPIC_FILTERING` downlevel flag is required too ... without the flag wgpu silently clamps every sampler's anisotropy to 1" | holds | `src/gpu/mod.rs:281`; `material.rs:343-357` |
| 0021 | "`max_binding_array_elements_per_shader_stage` at least 65,536" | holds: the five Channels take 5 x 13,107 = 65,535 | `src/gpu/mod.rs:293`, `:301`; `material.rs:15-16` |
| 0021 | "`max_buffer_size` at least 2^31 − 1 (ADR 0017 arenas)" | holds | `src/gpu/mod.rs:300`; `pools.rs:542-545` |
| 0021 | "`VERTEX_STORAGE`: the vertex shader ... pulls its vertices from the vertex Arena through storage buffers" | holds: the vertex Arena is `STORAGE` | `src/gpu/mod.rs:284`; `pools.rs:98-102` |
| 0023 | "Brighter emissives need KHR_materials_emissive_strength, which the cooker does not read yet" | holds: the Material's emissive strength is the constant 1.0 | `material.rs:284` |
| checklist | "Pool lookups return `Option<&T>`" | holds | `pools.rs:429-431`, `:496-498` |
| checklist | no `unwrap()` in `src/render/`'s hot path; `pools.rs` is exempt by the conventions test | holds: the `expect`s in `pools.rs` and `material.rs` outside tests are on constants (`"256 is not zero"` lives in `staging.rs`); `cooked.rs`'s are on slice lengths just checked | `cooked.rs:115`, `:568`, `:633`, `:643`; `tests/conventions.rs:339-342` |
| checklist | "The runtime loads cooked Blobs only and references Assets by Asset ID, never by source path" | holds: `AssetLoader` takes `AssetId` only | `loader.rs:9-33` |
| checklist | "Streaming uploads go through the `StagingBelt` ... no GPU buffer grows at runtime, and a request past a buffer's capacity is refused and logged" | broken for the Material SSBO: a Material past 256 is logged and substituted by Material 0, not refused: L1-C04 | `material.rs:20`, `:241-247` |

#### Spec stories

| Story | Verdict | Where |
|---|---|---|
| 4, the cooker builds Meshlets of at most 64 vertices into each Mesh Blob with meshopt | holds, reader side: the limits are refused on read; the build falls in cook | `cooked.rs:38-44`, `:444-461`; `src/cook/mesh.rs:101`, `:180-181` |
| 6, the vertex shader reads the 28-byte vertex from the vertex Arena | holds, Rust side: `CookedVertex` is 28 bytes, the Arena's element; the decode oracle is the shaders section's | `cooked.rs:391-403`; `pools.rs:98-102`, `:359` |
| 7, each Primitive draws with its own Material unless its Renderable overrides it; the loader's warning goes | holds: each Primitive's Material loaded once per Mesh, written as its Meshlets' `material_id`; a Primitive without one marks the Mesh `needs_material_handle`; the test asserts no `WARN` | `loader.rs:91-113`; `pools.rs:236-243`, `:299`; test `loader.rs:127-203` |
| 8, SciFiHelmet's Mesh Blob bytes per triangle kept as a ceiling | falls in cook: `HELMET_MESH_BYTES_PER_TRIANGLE = 13.83` in `tests/cook.rs:677-681`; the reader's format is what it measures | `cooked.rs:503-545` |
| 28, one compacted BLAS per Mesh built at load | holds; a refitted BLAS is built `ALLOW_UPDATE` and never compacted, as the refit needs | `pools.rs:44-46`, `:315-346`, `:393-425` |
| 36, acceleration-structure bytes from measured per-backend sizes, labelled estimates | holds, pool side: `estimated_blas_bytes` by compacted and uncompacted triangles | `pools.rs:479-489`; `src/render/renderer.rs:788-795` |
| 44, every GPU buffer M3 adds at a fixed provisional capacity, overflow counted and warned | holds for the Mesh table and the three Meshlet Arenas: fixed at creation, a Mesh past them refused with an `error!` and nothing written; the refusal is logged, not counted, as the Arena refusal test that predates M3 decided (see *Notes for the verifier*) | `pools.rs:12`, `:106-126`, `:188-229`; test `:965-994` |
| 50, `crowd` with a Baseline and no Golden | holds, loader side: `bent_mesh`, `refitted_mesh` and `shadow_proxy` load the figure drawn-only, refitted, or traced-only without Materials | `loader.rs:21-32`, `:64-78`; `pools.rs:17-28`; `src/scenes/mod.rs:988-990`, `:1019-1024` |
| 57, the crowd's BLAS refit through wgpu-hal against per-bone proxies | holds, pool side: `refitted_blases()` hands the build descriptors and Arena ranges to `blas_refit`; `rebuild_refitted_blases` is the oracle's wgpu rebuild | `pools.rs:349-383`; `src/render/blas_refit.rs:100-104`, `:178` |
| Implementation decision, GPU tables | holds (clause row above) | `pools.rs:233-291` |
| Testing decision, the Arena refusal test | holds: one vertex past the vertex Arena, then one index past the index Arena while the vertices fit, each refused with `gpu_memory_bytes` and `arena_bytes_in_use` unchanged, and the vertex given back | `pools.rs:964-994` |
| Testing decision, the procedural scenes rendering without `assets/` | falls in tooling and scenes (`tests/reference_scenes.rs`); the reader side is the `AssetSource::Scene` ID | `cooked.rs:63-64`; `tests/reference_scenes.rs:58-61` |

#### Registry keys

Four keys are cited in the module, all in `cooked.rs`, all four KTX 2.0
specification quotes; each was read against its registry entry and the
`ktx2` 0.5.0 reader (`ktx2-0.5.0/src/lib.rs:243-249` reads levels through
the level index; `:290-298` iterates the key-value data). All match.

| File | Keys | Verdict |
|---|---|---|
| `cooked.rs:26`, `:259`, `:856` | `ktx2-key-value-alignment` | matches: "the key length including its terminating NUL must be a multiple of the number of bytes in the number"; `PXTXVERSION` plus NUL is 12 bytes for a u32, and the test asserts both the key length and the value's offset are multiples of 4 (`:857-864`) |
| `cooked.rs:255` | `ktx2-level-order` | matches: "Mip level data is ordered from the level with the smallest size images ... to that with the largest"; the writer appends levels in reverse (`:284-292`) and the test asserts the level index's offsets descend (`:841-851`); the reader takes each level by its index entry, so mip 0 comes first whatever the byte order |
| `cooked.rs:256` | `ktx2-supercompression-per-level` | matches: "applied independently to each mip level"; one `zstd::bulk::compress` per level (`:285`), one `decompress` per level with its expected size as capacity (`:372`) |
| `cooked.rs:257` | `ktx2-level-alignment` | matches: "required_alignment = ... 1 (supercompressionScheme ≠ 0)"; the writer concatenates compressed levels with no padding (`:285-292`) and the reader requires `Zstandard` (`:338-341`) |

#### Oracles

| Function | Test, constant | Literature number, key | Measured error | Verdict |
|---|---|---|---|---|
| `AssetId::new` (`cooked.rs:83-117`) | `asset_id_is_truncated_sha256_of_its_encoding` (`cooked.rs:792-800`), `397412b2d75d55daad25a42ab0cbbb05` | SHA-256 of the 35-byte encoding `G`, GUID `00..0f`, kind `01`, `N`, length 8 as u64 LE, `material`, truncated to 128 bits; no registry key (ADR 0018 names SHA-256; `sha2` 0.11 is the implementation); recomputed with `shasum -a 256` on the same bytes: `397412b2d75d55daad25a42ab0cbbb05` | 0 (bit-exact) | holds |

The other tests in the module (Blob round trips, the Texture layout, the
GPU tables read back, BLAS compaction) compare the code with itself or
with the specification quotes above, not with a literature number, so
they are not oracles in the lane's sense.

#### Open issues read

| Issue | What the code shows | Needs hardware |
|---|---|---|
| #221, a renamed Asset's old ID still loads its stale Blob, unwarned | As the issue says. The cook index is one file per Asset ID, `index/<id>` (`cooked.rs:698-700`), written by `write` and never removed by any code (`:744-755`; no `remove_file` in the module). `entry` resolves any ID whose index file and Blob exist (`:703-710`), and `read` returns that Blob with no check that a source still names the ID (`:713-727`). The cooker's two warnings fire only for an ID absent from the cache: a Material's Texture (`src/cook/mod.rs:190-196`) and a Primitive's Material (`:344-352`). So ADR 0018's "The cooker warns when an ID that something references no longer exists" holds for "exists" read as "has an entry", and after a rename the old ID still has one: the renamed Mesh is re-cooked under its new ID, the old ID's entry stays, and a scene or save still naming the old ID loads the stale Blob in silence; "never evicted" (ADR 0018) is what keeps it. No Finding filed: the owner has placed the issue at M4's charting as `fog`, and a Finding here would restate it. What a fix needs is on the cooker's side (an ID the cook no longer produces, warned about or tombstoned), not the reader's. | No |

### scenes, components, transforms and the fixed loop

Scope: `src/scenes/mod.rs` (1,351 lines), `src/scenes/town.rs` (1,296),
`src/components.rs` (101), `src/transform/mod.rs` (327), `src/time/mod.rs`
(283); 3,358 lines, every file read in full, test modules included. The
tests that exercise them were read in full: `tests/reference_scenes.rs`,
`tests/crowd.rs`, `tests/shadows.rs`, `tests/frame_stats.rs`, and
`tests/goldens.rs` where it lists the scenes, the 100 km shift and `town`'s
frames. `src/state.rs` was read where it builds the two schedules, spawns a
scene, shifts it and propagates it (`:60-130`, `:250-345`) and in its test
module (`:346-563`), which drives the fixed loop against an analytic orbit;
`src/lib.rs:215-250` where it measures the frame delta; `src/capture/mod.rs`
where it feeds a fixed delta (`:176-262`); `src/render/prepare.rs` only where
it consumes `PreviousWorldTransform`, `Bend`, the lights and the blend weight
(`:36-100`, `:296-322`, `:340-400`, `:515-612`), whose verdicts are the render
section's; `src/camera/mod.rs` for `ev100_for_exposure`, `FIELD_OF_VIEW` and
`fly_camera`; `src/environment/mod.rs`; `src/threading.rs:27-31`;
`src/render/bend_pass.rs:187-206` for the `bent` oracle the scene tests call;
`src/assets/loader.rs:24-78` for the three ways the crowd loads a Mesh. ADRs
read in full: 0006, 0014, 0018, 0019, 0020, 0022, 0023; ADR 0003's and 0016's
clauses on Frame data through the render section. The spec's stories 17,
18, 25, 26, 27, 31 and 45 to 50 and its Reference scenes, DirectionalLight,
Froxel and acceleration-structure Implementation decisions were checked.
Every `where` is `path:line` at the frozen commit, from the repository root.
`deviations.md` holds no row at this commit; `DIFFERENCES.md` describes
images, none of which this module's code decides. Three component methods
on `Transform`, `Bend` and `WorldTransform` are already filed as L1-R03 in
the render section; they are cited, not filed again.

#### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0022 | "Simulation advances in fixed Ticks of 1/64 s (15.625 ms)" | holds: `TICK = 15_625_000` ns, exact in binary, `TICK_SECONDS` its f64 | `src/time/mod.rs:20-24` |
| 0022 | "`src/time/` runs the fixed schedule once per Tick, at most 4 per frame" | holds | `src/time/mod.rs:27`, `:53-63`, `:80-87` |
| 0022 | "accumulated time past the cap is dropped, the game slows, and the dropped time is reported in frame stats" | holds: `advance` returns the Ticks past 4 as `dropped`, `run_ticks` hands it up, the recorder keeps it per frame; 250 ms drops 187.5 ms in the test | `src/time/mod.rs:56-62`, `:86`; `src/state.rs:256`; test `src/time/mod.rs:200-210` |
| 0022 | "The frame delta is measured with `Instant` at frame start, before surface acquire ... and clamped to 250 ms" | holds: the window measures before `get_current_texture`; `MAX_FRAME_DELTA` 250 ms; a capture feeds each scene's fixed delta instead | `src/lib.rs:219-224`; `src/time/mod.rs:18`, `:33-35`; `src/scenes/mod.rs:370-387`; `src/capture/mod.rs:236`, `:260` |
| 0022 | "Simulated time is an integer Tick counter; systems read a per-Tick delta derived from it, never an f32 sum of seconds" | holds: `tick: u64`; `spin` and `sway` set the state for the end of the running Tick from `(tick + 1) as f64 * TICK_SECONDS`, so nothing accumulates; `Time::delta_time` is per frame and read by `fly_camera` alone | `src/time/mod.rs:8-14`, `:42`; `src/scenes/mod.rs:662-670`, `:1060-1077`; `src/camera/mod.rs:99` |
| 0022 | "an accumulator resource, a fixed schedule run once per Tick, and the blend weight α as a resource" | holds: `FixedTime { tick, blend_weight, accumulator }` | `src/time/mod.rs:37-47`, `:80-87` |
| 0022 | "ambiguity detection is `Error`" | holds, tested: an unordered conflicting pair fails `initialize` | `src/time/mod.rs:69-76`; test `:113-125` |
| 0022 | "Everything that changes game state runs in the fixed schedule ... Input sampling, the camera, the blend, prepare and presentation-only effects run once per frame" | holds: the fixed chain is record previous transforms, record previous curvatures, helmet spawn and despawn, `spin`, `sway`, `clear_just_pressed`, `reset_teleported`, propagate; the per-frame schedule holds `fly_camera` only; presses wait in `Input` for the next Tick, tested | `src/state.rs:318-342`; test `src/time/mod.rs:264-282` |
| 0022 | "The camera's rotation reads the newest input every frame, outside the fixed loop" | holds: `fly_camera` in the per-frame schedule reads `Input` and `Time`; "its position follows whatever it is attached to, interpolated" is not applicable, no scene attaches the camera | `src/state.rs:320`; `src/camera/mod.rs:99-102` |
| 0022 | "the helmet spawn and despawn run in it" | holds here; the systems' bodies fall in the helmet module | `src/state.rs:332-334` |
| 0022 | "World transforms propagate at the end of every Tick" | holds: `propagate_world_transforms` closes the chain, and runs once at load so frames before the first Tick draw composed transforms | `src/state.rs:339-341`, `:104-108`; `src/transform/mod.rs:87-141` |
| 0022 | "an entity the fixed schedule moves keeps its previous Tick's World transform in `PreviousWorldTransform`", children included | holds: `record_previous_world_transforms` opens each Tick; every orrery body, every town flock centre and bird, every crowd bone carries it; the refit-form figure carries it too (next row) | `src/transform/mod.rs:34-58`; `src/scenes/mod.rs:633-650`; `src/scenes/town.rs:791-807`; `src/scenes/mod.rs:1026-1034` |
| 0022 | "Static Instances store nothing extra" | holds for every static Renderable (the town's 244,256 buildings, props and lamps, the shadows Mesh, the sweeps); the refit-form crowd figure carries `PreviousWorldTransform` though its Transform never changes, which ADR 0020 decides ("its Instance a Moving Instance") and the code says why (see *Notes for the verifier*) | `src/scenes/town.rs:770-788`; `src/scenes/mod.rs:1014-1023` |
| 0022 | "prepare blends previous and current in f64" | holds: `WorldTransform::drawn` lerps `matrix3` and `translation` in f64; `Bend::drawn_curvature` lerps the curvature; consumers in the render section | `src/transform/mod.rs:16-31`; `src/components.rs:71-77`; `src/render/prepare.rs:383`, `:531`, `:555`, `:606` |
| 0022 | "A spawned entity starts with previous equal to current ... so neither sweeps across the Game world" | holds in effect: a spawn carries `PreviousWorldTransform(None)`, drawn at its current transform until the first Tick records it; a `Bend` spawns with `previous_curvature == curvature`; tested at 100 km with a spawn inside a Tick | `src/transform/mod.rs:22-24`, `:37-39`; `src/scenes/mod.rs:637`, `:1001-1008`; `src/scenes/town.rs:797`, `:805`; test `src/state.rs:509-561` |
| 0022 | "a teleport goes through an explicit marker that resets previous to current" | holds: `Teleported` drops the previous transform of the entity and its descendants before `propagate_world_transforms` and removes itself; no production system inserts the marker at this commit, so the State test alone exercises it (see *Notes for the verifier*) | `src/transform/mod.rs:43-48`, `:69-85`; `src/state.rs:337-339`, `:528`, `:540-561` |
| 0022 | "Determinism: ... no system reads the wall clock, randomness comes from a seeded resource advanced per Tick, and no result depends on hash-map iteration order or on reduction order in a parallel iterator" | holds: `Instant::now` appears only in the window, the recorder, the renderer and the Material load, none in the fixed schedule; the fixed schedule has no randomness, so the seeded resource is not exercised: `town`'s choices are a hash of where each thing stands, taken at spawn; the schedule is single-threaded; queries iterate in table order, `placements` sorts its lamp corners stably, and the dirty list is sorted and deduplicated | `src/time/mod.rs:69-70`; `src/threading.rs:27-31`; `src/scenes/town.rs:255-270`, `:357-359`, `:399-403`; `src/transform/mod.rs:105-109` |
| 0022 | "the same binary on the same machine, fed the same per-Tick inputs, reaches a bit-identical simulation state whatever the frame timing" | holds for the loop: 1,280 Ticks under 1 to 40 ms frames and 250 ms stalls bit-compare with steady 60 Hz | test `src/time/mod.rs:168-197` |
| 0022 | "Nothing is guaranteed across platforms or builds: `sin` and its kin are not bit-stable" | not applicable: `spin` and `sway` use `sin` and `from_rotation_y`, no cross-platform claim is made | `src/scenes/mod.rs:667-668`, `:1050-1052` |
| 0022 | "The `orrery` Reference scene shows it" | holds: five spheres, each a child of the last, spun per Tick, each carrying `PreviousWorldTransform`; its capture steps at 144 Hz so frames land between Ticks | `src/scenes/mod.rs:41-43`, `:627-653`, `:370-375` |
| 0022 | "the step model must also stay judder-free at refresh rates that do not divide 64 Hz (120, 144, variable)" | holds in form: the orrery's capture steps at 144 Hz, the State test cycles 144, 60, 30 and 90 Hz frames against the analytic orbit within 10 µm; judder itself is a visual check | `src/scenes/mod.rs:375`; `src/state.rs:436-437`, `:446-459` |
| 0022 | "The blend reads two simulation states, never screen-space history" | holds: `drawn` reads two components | `src/transform/mod.rs:21-31` |
| 0022 | "Input latency ... ADR 0013's budget splits, judged at p95 once Input latency is measured" | falls in tooling (bench); not measured at this commit | |
| 0022 | "A physics engine chosen in M7 steps once per Tick" | deferred by the ADR to M7 | |
| 0006 | "Components hold data only: no closures, no `Box<dyn Trait>`, no behaviour methods" | holds for `MeshRef`, `MaterialRef`, `PointLight`, `DirectionalLight`, `PreviousWorldTransform`, `Teleported`, `Spin`, `Bone`; broken in letter for `Transform`, `Bend` and `WorldTransform`: L1-R03 (render section); `Sway::curvature` is a fourth method of the same kind, for L1-R03's magnitude (see *Notes for the verifier*) | `src/components.rs:43-47`, `:79-101`; `src/transform/mod.rs:40-48`; `src/scenes/mod.rs:655-660`, `:1046-1053`, `:1056-1059` |
| 0006, checklist | "No GPU resources stored on ECS components" | holds: `MeshRef`, `MaterialRef` and `Bend::rest` hold Handles, indices into the Pools | `src/components.rs:43-47`, `:61` |
| 0014 | "`Transform` holds translation as `DVec3` ... with rotation (`Quat`) and scale (`Vec3`) in f32, 64 bytes" | holds, asserted at compile time: 64 bytes, 16 aligned | `src/components.rs:9-18` |
| 0014 | "A child's Transform composes with its ancestors' into a World transform in f64, recomputed only when it or an ancestor moved" | holds: `to_affine` lifts rotation and scale to f64, `propagate_world_transforms` recomputes dirty subtrees and nothing else, tested | `src/components.rs:34-40`; `src/transform/mod.rs:87-141`; tests `:221-239`, `:241-268` |
| 0014 | "right-handed, +Y up (gravity along −Y) ... with cameras looking down −Z" | holds in this module: a `DirectionalLight` shines along its −Z, each scene's sun is `from_rotation_arc(NEG_Z, direction)`, ground planes face +Y, spins and flocks turn about +Y | `src/components.rs:94-95`; `src/scenes/mod.rs:436-439`, `:1193-1214`, `:664-669` |
| 0014 | "Within 100 km of the origin ... simulated positions resolve to 0.1 mm" | holds: the five-deep chain at 100 km is exact to the bit against its closed form (see *Oracles*); the blended orbit at 100 km holds 10 µm | `src/transform/mod.rs:170-171`, `:194-219`; `src/state.rs:366-367`, `:446-459` |
| 0014, spec | "`town` stays within about 2 km of that anchor, an f32 step of at most 0.24 mm" | holds: the town spans ±500 m, its ground ±550 m, flocks stay inside their 100 m cells, the camera starts at (2, 1.7, 445) and orbits at radius 480; the farthest ground corner from the load anchor is about 1.14 km | `src/scenes/town.rs:168-173`, `:240-248`, `:821-825`, `:960-962` |
| 0014 | the 100 km check: "a point 0.3 m from the camera lands within 0.25 px ... of its exact camera-relative position" | holds on the scene side: `State::new` shifts every root Transform and the camera by the shift (children stay parent-relative), and the Goldens test shifts each scene by 1,563 whole 64 m cells; `town` and `shadows` are in its list | `src/state.rs:72`, `:99-103`; `tests/goldens.rs:33-43`, `:50-56`, `:216-223` |
| 0018 | "Procedural Reference scenes cook too, under Asset IDs derived from the scene name, so Goldens test the cooked path" | holds: `material_id`, `mesh_id` and `texture_id` build from `AssetSource::Scene(self.name())`; every spawn loads through the loader by those IDs; a launch from an empty directory cooks into `cooked/index` | `src/scenes/mod.rs:106-122`, `:541`, `:632`; `src/scenes/town.rs:756-765`; `tests/reference_scenes.rs:58-61` |
| 0018 | "The runtime ... references Assets by Asset ID, never by source path" | holds in this module: the glTF scenes name their source by manifest GUID; `source_files` exists only under the `cook` feature | `src/scenes/mod.rs:505-508`, `:580-582`, `:125-140` |
| 0023 | "a `DirectionalLight` gives illuminance in lux and a `PointLight` luminous intensity in candela" | holds: the fields say so; the sun is 100,000 lx, the coloured lights 900,000 cd, the lamps 2,000,000 cd, minification's 3 × 10⁷ cd | `src/components.rs:83-84`, `:99-100`; `src/scenes/mod.rs:473`, `:496-503`, `:686`; `src/scenes/town.rs:224-229` |
| 0023 | "a Reference scene's ambient term gives illuminance in lux"; "emissive and background give luminance in cd/m²" | holds: `ambient_illuminance` 10,000 lx, `background_luminance` (10,000, 20,000, 30,000) cd/m² in every scene | `src/scenes/mod.rs:447-467`; `src/environment/mod.rs:3-12` |
| 0023 | "Exposure is an EV100, which scales luminance by 1/(1.2 · 2^EV100)"; "each scene has an EV100" set by hand | holds: every scene's EV100 is `ev100_for_exposure(1 / 100,000)` = log₂(100,000 / 1.2) = 16.35, as the comment says | `src/scenes/mod.rs:351-368`, `:347`; `src/camera/mod.rs:11-16` |
| 0023 | "every Reference scene's lights, ambient and background are in these units, under a sun of 100,000 lx" | holds: one `SUN_ILLUMINANCE` for every scene's sun and every other light is given against it | `src/scenes/mod.rs:418-445`, `:470-473` |
| 0023, spec | "Point lights carry a range: authored, as in KHR_lights_punctual, or derived" | holds: every range is authored, 20 m, 18 m, 200 m, and the field is KHR_lights_punctual's `range`, here finite | `src/components.rs:85-87`; `src/scenes/mod.rs:500`, `:688`, `:867-870`; `src/scenes/town.rs:227` |
| 0023, spec | "The sun and 1,024 point lights" in `town` | holds: `LAMP_COUNT = 1024` lamps at the lowest-ranked block corners, one light each, below the head and outside the Mesh | `src/scenes/town.rs:218-229`, `:357-364`, `:784-788`; `src/scenes/mod.rs:434` |
| 0020 | "Direct light is one hard shadow ray per pixel per light" | drift: `PointLight::casts_shadow` lets a light take no ray, which no ADR or spec line admits; every Reference scene sets it `true`: L1-C07 | `src/components.rs:88-91`; `src/scenes/mod.rs:399`, `:501`; `src/scenes/town.rs:228` |
| 0020 | "The `crowd` Reference scene's figures, which the GPU bends each frame, have no BLAS and cast their shadows through rigid per-bone shadow proxies, Moving Instances in that TLAS, the default form" | holds: `spawn_crowd` is the proxies form; each of 100 figures gets its own bent Mesh and 16 bone children with the proxy Mesh and `PreviousWorldTransform`; the proxy loads as traced, undrawn | `src/scenes/mod.rs:959-964`, `:989-993`, `:1024-1034`; `src/assets/loader.rs:30-32`, `:72-78` |
| 0020 | "The crowd's refit Render switch instead gives each figure a BLAS ... its Instance a Moving Instance" | holds on the spawn side: `spawn_traced_crowd` loads each figure as `refitted_mesh` with `PreviousWorldTransform` and no proxies | `src/scenes/mod.rs:966-974`, `:1014-1023` |
| 0020 | "a dynamic TLAS of Moving Instances rebuilt each frame and capped at 10k Instances" | holds: `town`'s flocks are asserted at compile time to equal `DYNAMIC_TLAS_CAPACITY`, 100 flocks of 100 birds | `src/scenes/town.rs:231-239` |
| 0019 | "lit by every directional light and by the point lights of each pixel's Froxel ... from a light buffer of fixed capacity" | falls in render and shaders; the scenes supply at most 1,025 lights | `src/scenes/town.rs:220` |
| 0019 | "on a Reference scene at 1440p with 4x MSAA, the opaque main pass takes more than 6 ms ... The clause fired on `town`" | falls in tooling (bench); the scene exists at the decided scale | `src/scenes/mod.rs:63-67` |

#### Spec stories

| Story | Verdict | Where |
|---|---|---|
| 17, directional lights are components with a colour and an intensity, direction from the entity's World transform | holds: `DirectionalLight { color, intensity }`; each scene's sun is a `Transform` whose −Z is turned to the scene's direction, spawned with the light | `src/components.rs:94-101`; `src/scenes/mod.rs:418-445`; `src/state.rs:95` |
| 18, the hard-coded light gone, no scene lit by a light it did not spawn | holds: `State::new` spawns `scene.sun()` and `scene.point_lights()`, `town` spawns its lamps' lights itself; `texture-wall` and `crowd` have no point light on purpose | `src/state.rs:94-98`; `src/scenes/mod.rs:389-416`; `src/scenes/town.rs:784-788` |
| 25, ambient one constant irradiance per scene, background one constant radiance, in the decided units | holds | `src/scenes/mod.rs:447-467`; `src/environment/mod.rs:8-12` |
| 26, 27, every light that reaches a pixel traces one ray; a ray skipped only where its light adds nothing | the trace falls in shaders; on the component side `casts_shadow` can skip a light that adds light, unused by any scene: L1-C07 | `src/components.rs:88-91` |
| 31, an Instance is a Moving Instance exactly when its entity carries its previous World transform | holds on the spawn side: the orrery's bodies, the town's birds and the crowd's bones or refit figures carry it, nothing else does; classification in the render section | `src/scenes/mod.rs:637`, `:1020`, `:1033`; `src/scenes/town.rs:797`, `:805` |
| 45, a dense Reference scene, `town`, at target scale | holds: 254,256 Instances of 19 Meshes and 9,994,190 triangles by the closed-form test; the M4 Pro Baseline's medians draw 137,730 Instances and 5,552,214 triangles after culling, over the spec's 100,000 and 5,000,000 | `src/scenes/town.rs:23-73`; `tests/frame_stats.rs:12-15`, `:313`; `bench/baselines/apple-m4-pro/town.json:80-81` |
| 46, `town` procedural at cook time, counts exact, CI needs no fetch, joins the 100 km check | holds: `source_files` is empty for `town`, its Meshes and Materials come from `generators`, the counts are closed-form, and it is in the Goldens' list with the shift | `src/scenes/mod.rs:130-138`, `:229`, `:313`; `src/scenes/town.rs:869-1133`; `tests/goldens.rs:33-43`, `:216`; `tests/frame_stats.rs:313` |
| 47, `town`'s camera path at street level, then over the rooftops, with no cuts | holds: 420 frames at 1.7 m up the avenue at X = 0 at 4 m/s, a cubic of 360 frames leaving at that speed and arriving at rest at (0, 260, 480) while the gaze turns from far up the avenue to the town's middle, then a circle of radius 480 eased in over 120 frames; position, speed and gaze are continuous at both joins (checked by hand: the street's end equals the cubic at progress 0, the cubic's end equals the circle at angle 0, the eased angle's rate reaches `ORBIT_SPEED` where the linear branch takes over) | `src/scenes/town.rs:812-867` |
| 48, `town`'s Goldens at full density on Metal and software Vulkan, CI bench shortened to counters | holds for the Goldens: `tests/goldens/metal/town.png` and `tests/goldens/vulkan-cpu/town.png` exist and `town` takes the equivalence checks at eight frames; the CI bench's counters fall in tooling | `tests/goldens.rs:33-43`, `:296-307`; `tests/goldens/metal/town.png`; `tests/goldens/vulkan-cpu/town.png` |
| 49, a small shadow scene with Goldens | holds: a sphere of radius 0.5 m, 1 m over a ground plane of ±8 m, one Mesh of two Primitives with their own Materials, under the sun and one white point light; Goldens on both keys; the analytic oracle in `tests/shadows.rs` copies its constants exactly (see *Oracles*) | `src/scenes/mod.rs:52-55`, `:293-301`, `:846-877`; `tests/goldens/metal/shadows.png` |
| 50, `crowd` with a Baseline and no Golden | holds: not in the Goldens' list; `crowd.json` Baselines on all four adapters; 100 figures of 40 × 65 segments, 5,120 triangles each, bent on the GPU; its bent bounds hold every sway, tested | `tests/goldens.rs:33-43`; `bench/baselines/apple-m4-pro/crowd.json`; `src/scenes/mod.rs:882-894`, `:947-958`; test `:1323-1350` |
| Implementation decision, `town`: "about 1 × 1 km with building blocks as occluders, street props of 12 to 200 triangles, and thin poles and wires that go sub-pixel at distance" | holds: 20 × 20 blocks 50 m apart over ±500 m; the box is 12 triangles and the car, the largest prop, 168 (two cuboids and four 10-sided wheels of 36), by the generators' closed forms (a closed cylinder of n sides is 4n − 4 triangles, a cone 2n − 2, a cuboid 12); wires are 15 mm tubes of 3 sides, railing posts 25 mm | `src/scenes/town.rs:171-174`, `:927-947`, `:1071-1075`, `:1046-1048`, `:1186-1198`, `:1203-1259`, `:1264-1295` |
| Implementation decision, `town`: "About 250k Instances of 10 to 20 Meshes"; "10,000 Moving Instances circling in the fixed schedule" | holds: 254,256 Instances of 19 Meshes; 100 flock centres with `Spin` and 100 birds each as their children, all carrying `PreviousWorldTransform` | `src/scenes/town.rs:53-73`, `:231-239`, `:789-808`; `tests/frame_stats.rs:14`, `:313` |
| Implementation decision, `town`: "with the scene reporting average lights per lit pixel" | falls in render (the light count pass) and tooling | |
| Implementation decision, shadow scene: "A sphere over a plane under the sun and one point light" | holds | `src/scenes/mod.rs:293-301`, `:867-877` |
| Implementation decision, `crowd`: "100 meshes of 5,120 triangles deformed on the GPU each frame"; two forms | holds: each figure loads its own Mesh (`bent_mesh` or `refitted_mesh` per figure) from one rest Mesh; the forms are `CrowdForm::Proxies` and `Refit`/`Rebuild`; their timing falls in render and tooling | `src/scenes/mod.rs:988`, `:1019`, `:1024`; `src/state.rs:87-90` |
| Implementation decision, DirectionalLight: "A new component holding a colour and an intensity ... Each Reference scene spawns its own lights" | holds | `src/components.rs:96-101`; `src/state.rs:94-98` |
| Implementation decision, acceleration structures: "An Instance is moving when its entity carries `PreviousWorldTransform`" | holds on the spawn side (story 31) | as story 31 |
| Testing decision, prior art: "the procedural scenes rendering without `assets/`" | holds: four procedural scenes render identically twice from a directory with no `assets/`, and a glTF scene fails naming its manifest GUID | `tests/reference_scenes.rs:9-79` |

#### Registry keys

Two keys are cited in this module's files; both match their quotes. The
light components cite ADR 0023 rather than a key; the ADR's keys are read
against them.

| File | Keys | Verdict |
|---|---|---|
| `src/scenes/mod.rs:471` | `bevy-lux-constants` | matches: the quote gives `DIRECT_SUNLIGHT: f32 = 100_000.`, and `SUN_ILLUMINANCE = 100_000.0` lx at `:473` |
| `src/scenes/mod.rs:519` | `hoffman-2013-f0` | matches: the quote gives gold's linear F0 as `1.00,0.71,0.29`, and `SWEEP_BASE_COLOR = [1.0, 0.71, 0.29, 1.0]` at `:522` |
| `src/components.rs:83`, `:99` (cite ADR 0023) | `khr-lights-punctual-units` | matches: "`point` ... lights use luminous intensity in candela ... `directional` lights use illuminance in lux"; `PointLight::intensity` is documented in candela and `DirectionalLight::intensity` in lux, and every scene's values are in those units |
| `src/components.rs:85-87` (names the extension's `range`) | `khr-lights-punctual-range` | matches in role: the quoted window is a function of `range` and distance; the field is that `range`, here required finite; the window itself is the shaders section's |

#### Oracles

| Function | Test, constant | Literature number, key | Measured error | Verdict |
|---|---|---|---|---|
| `propagate_world_transforms` (`src/transform/mod.rs:95-141`) | `a_five_deep_chain_matches_its_closed_form`, `assert_near` ≤ 1e-9 m at `ROOT` 100 km (`:170-171`, `:194-219`) | ADR 0014's bar: simulated positions resolve to 0.1 mm within 100 km; no registry key | 0 by analysis, not run: `CYCLE` = (0.5, 0.5, 0.5, 0.5) is exact in f32 and `DMat3::from_quat` of it is a permutation matrix, scale 2 keeps every entry dyadic, and each translation is `ROOT` plus a power of two under the same exponent (100000.1 + 8, 0.25 + 18, −3.3 + 4 are exact; −3.3 + 36 rounds once on both sides to the same f64) | holds; 1e-9 m is 10⁵ times inside the bar |
| `WorldTransform::drawn` (`src/transform/mod.rs:21-31`) | `a_moving_entity_is_drawn_at_the_linear_blend_of_its_two_newest_ticks`, ≤ 1e-9 m and exact `matrix3` (`:289-308`) | ADR 0022: "prepare blends previous and current in f64" | 0: weight 0.25 of (4, −2, 8) is exact | holds |
| the State's fixed loop with `record_previous_world_transforms`, `propagate_world_transforms` and `drawn` (`src/state.rs:318-342`) | `assert_drawn_at` ≤ 1e-5 m against the analytic `blend_on_orbit` at `PIVOT` 100 km, over 200 frames of 144, 60, 30 and 90 Hz (`src/state.rs:366-367`, `:433-459`, `:461-468`, `:540-561`); `drawn_curvature` ≤ 1e-4 (`:489-507`) | ADR 0014's 0.1 mm bar; ADR 0022's blend | bounded, not run: the pivot's rotation is an f32 `Quat` from an f64 angle, 2⁻²⁴ × 2π ≈ 3.7e-7 rad, which at `ORBIT_RADIUS` 2.5 m is 9.4e-7 m, a tenth of the constant; the pivot's translation is exact | holds; 10 µm is inside the bar |
| `bone_transform` and `bent_middle` (`src/scenes/mod.rs:1084-1120`) | `each_bones_shadow_proxy_sits_on_the_bent_middle`, centre ≤ 1e-7 m, axes ≤ 1e-6, against the bend pass's `bent`, a 1,024-step midpoint sum (`:1268-1288`; `src/render/bend_pass.rs:187-206`) | the circular arc: chord h·sin(κh/2)/(κh/2) at half the turn; no registry key (geometry) | the oracle's own error bounds the gap: midpoint rule over h = 1.74375 m (bone 15) with |f''| = κ² = 0.1225 and N = 1,024 gives h³κ²/(24N²) = 2.6e-8 m, the comment's "about 3e-8 m", a quarter of the constant; the axes pass through an f32 `Quat`, about 1e-7 | holds |
| `crowd_figure_bounds` (`src/scenes/mod.rs:952-957`) | `a_crowd_figure_bends_within_its_bounds`: containment, no tolerance, eight curvatures up to `CROWD_SWAY_CURVATURE` at two azimuths (`:1323-1350`) | the arc's lateral reach (1 − cos κh)/κ; no key | none to measure: a containment test | holds by its form; the GPU side falls in shaders |
| `bone_transform` (`src/scenes/mod.rs:1084-1098`) | `each_bones_shadow_proxy_lies_inside_its_figure`: strict containment of 33 × 16 points per proxy in the spheroid (`:1293-1315`) | the figure's spheroid, radius 0.25 m, height 1.8 m | none: containment | holds by its form |
| `FixedTime::advance` and `run_ticks` (`src/time/mod.rs:53-87`) | exact equalities: 250 ms drops 187,500 µs with weight 0; 100 ms drops 31,250 µs with weight 0.4; 600 frames of 16,666,667 ns run 640 Ticks; bit-identical positions over 1,280 Ticks under jitter (`:199-255`, `:168-197`) | ADR 0022: Ticks of 1/64 s, at most 4 per frame, 250 ms clamp | 0: integer nanosecond arithmetic; 600 × 16,666,667 ns = 10,000,000,200 ns = 640.00001 Ticks | holds |
| the shadows scene's geometry and lights (`src/scenes/mod.rs:846-877`, `:293-301`) | `tests/shadows.rs`: `TOLERANCE = 1` 8-bit step outside a one-pixel band, each light's shadow over 4,000 judged pixels; its copies of the scene (`SPHERE_CENTRE` (0, 1, 0), radius 0.5, ground ±8, camera (0, 2.4, 3) at (0, 0.45, −0.4), 45° field, sun (0.5, −1, −0.3), point (1.5, 2.5, 1.0)) equal the scene's constants and `FIELD_OF_VIEW` | the test's own bound: the triangulated sphere lies within 2.1 mm of the true one, so outlines lie within 2.5 mm (a quarter pixel) under the sun and 11 mm (three quarters) under the point light | constants match line for line (`tests/shadows.rs:27-50`, `:68` against `src/scenes/mod.rs:849-870`, `src/camera/mod.rs:86`); the pixel verdict needs a GPU run, the tests' or tooling's | holds on the scene side |

### Findings

| id | claim | severity | where | evidence | expected |
|---|---|---|---|---|---|
| L1-C01 | The cooker warns of a dangling reference only for a Material's Textures and a Primitive's Material, checked against the never-pruned Cooked cache, and sees none of a Reference scene's references by name, so a renamed Asset's old ID resolves to its stale Blob without a warning. | `correctness`; one clause of ADR 0018 broken in part; two reference kinds of four (Material→Texture, Primitive→Material, scene→Mesh, scene→Material) covered; no visible defect until a source is renamed | `mod.rs:190-196`, `:197-207`, `:340-353`, `:130-131`; `src/assets/cooked.rs:703-710`, `:713-727`, `:745-755`; `src/state.rs:202-203`; `src/scenes/mod.rs:580-582` | block below | ADR 0018: "The cooker warns when an ID that something references no longer exists." |
| L1-C02 | `CookedMesh::from_blob` sizes the decoded vertex allocation from the Blob's vertex count before anything checks that count, so a corrupt count allocates up to 2^32 x 28 bytes and aborts instead of returning the refusal the index count gets | `correctness`; one count, up to 2^32 vertices of 28 bytes | `cooked.rs:551`, `:586-587` | the code's own rule at `:588-592`: "a count past that is corrupt and must not size an allocation"; `meshopt-0.6.2/src/encoding.rs:82`: `let mut result: Vec<T> = vec![Default::default(); vertex_count];` before the FFI call | the vertex count is bounded by the encoded bytes before `decode_vertex_buffer` is called, as the index count is at `:588-593` |
| L1-C03 | Layer 0 of the normal Channel decodes to a normal tilted 0.32° off the surface, not the "flat normal" the comment and ADR 0017's "neutral default" describe | `correctness`; x and y off by 1/255 (0.0039) each, a 0.318° tilt, on every Material without a normal Texture | `material.rs:187` | `(wgpu::TextureFormat::Rgba8Unorm, [128, 128, 255, 255])` decoded by `shading.wgsl:156-159` as `encoded * 2.0 - 1.0`: 128/255 x 2 − 1 = 1/255; the comment at `material.rs:156-159`: "the flat normal, x and y at 0.5 ... samples a no-op, and its factors pass through unchanged"; ADR 0017: "Layer 0 stays the neutral default" | a Layer 0 normal that decodes to exactly (0, 0, 1): 0.5 has no `Unorm8` code, so a signed format, a 16-bit code or a shader select on `DEFAULT_LAYER` |
| L1-C04 | A Material past the SSBO's 256 is not refused: `insert` logs and returns `MaterialHandle(0)`, so the Renderable draws with Material 0 | `correctness`; one capacity, Materials past 256 | `material.rs:241-247` | `return MaterialHandle(0);` after `tracing::error!(... "material SSBO full; reusing material 0 ...")`; `CLAUDE.md` checklist: "a request past a buffer's capacity is refused and logged"; the Mesh pool's sibling refuses (`pools.rs:188-194`) | `load` and `insert` return an error or `None` and write nothing, as the Mesh pool does, counted as the spec's story 44 asks of capacities |
| L1-C05 | On a degenerate triangle the engine's Meshlet cut counts the repeated corner once where meshoptimizer's scan builder counts it twice, so the two can cut differently, which ADR 0018's "as meshoptimizer's scan builder cuts them" denies and a code comment, not `deviations.md`, records. | `drift`; not load-bearing: any cut within the limits is valid, the Blob self-describes it and the reader checks the limits; affects only Meshes with degenerate triangles | `mesh.rs:158-164`, `:172-182`; `meshopt-0.6.2/vendor/src/clusterizer.cpp:362-370`; `docs/adr/0018-...md`, Consequences, Meshes bullet | block below | ADR 0018: "Each Meshlet is the longest run the limits allow, as meshoptimizer's scan builder cuts them"; `docs/audit/deviations.md`: "A Deviation lives here and nowhere else: not in a code comment" |
| L1-C06 | ADR 0018's status sentence says the renderer does not keep the Meshlets the reader rebuilds, while the Mesh pool keeps them in three Meshlet Arenas, as ADR 0017's amendment of 2026-10-20 and ADR 0019 say. | `drift`; not load-bearing: ADRs 0017 and 0019 and `CONTEXT.md` state the current form; a reader of ADR 0018 alone is misled about M3's draw path | `docs/adr/0018-...md:3` ("which the renderer does not keep yet"); `src/assets/pools.rs:76-78`, `:106-111`; `docs/adr/0017-...md:3` ("three Meshlet arenas ... amended 2026-10-20") | block below | the spec's Implementation Decision, ADR status lines: "Each ticket that changes what an ADR's 'Planned; today …' sentence describes updates that sentence" |
| L1-C07 | `PointLight::casts_shadow` lets a point light cast no shadow, taking no ray and no mask bit, which ADR 0020's "one hard shadow ray per pixel per light", story 27's "skipped only where its light adds nothing" and the spec's placement of "a per-light shadow opt-out" among the fog lines do not admit; no ADR records the decision. | `drift`; not load-bearing: every Reference scene sets it `true`, so every image follows the ADR; a reader of ADR 0020, story 27 or the spec's fog lines is misled about whether a light may opt out | `src/components.rs:88-91`; consumer `src/render/prepare.rs:550-561`; `src/scenes/mod.rs:399`, `:501`; `src/scenes/town.rs:228` | block below | ADR 0020 names the per-light opt-out as it names the 59-light cap, and the spec's fog line is struck; or the field waits for the scene that needs it |

#### L1-C01

Code, `mod.rs:190-196`:

```rust
for (id, _, material) in &sources.materials {
    for texture in material.textures.iter().flatten() {
        if cache.entry(*texture).is_none() {
            tracing::warn!(material = %id, %texture, "Material samples a Texture the Cooked cache lacks");
        }
    }
}
```

and `mod.rs:340-353`, `unresolved_materials`, which filters the
`(Mesh, Material)` references on `cache.entry(*material).is_none()` and
warns per unresolved pair. Both ask the Cooked cache, whose `entry`
(`src/assets/cooked.rs:703-710`) returns any index file whose Blob exists,
and whose `write` (`:745-755`) only ever adds or replaces an index file:

```rust
write_atomically(
    &self.index_path(id),
    format!("{hash} {cook_key}\n").as_bytes(),
)
```

The references a Reference scene holds are in code, by name:
`src/state.rs:202-203`

```rust
let source = AssetSource::File(SCI_FI_HELMET);
let id = AssetId::new(source, AssetKind::Mesh, AssetName::Name("SciFiHelmet"));
```

and `src/scenes/mod.rs:580-582` for `"NormalTangentTest_low"`. The cooker
never sees them: `cook_scene` (`mod.rs:130-131`) passes
`scene.source_files()`, the GUIDs, and `Sources` (`:92-101`) lists what the
files define, not what the scene loads. ADR 0018: "A glTF mesh renamed in
Blender gets a new Asset ID. The cooker warns when an ID that something
references no longer exists." After the rename the Mesh cooks under its
new ID, the old index file stays ("never evicted", which holds), and
`CookedCache::read` (`:713-727`) hands the loader the old Blob. The seam's
rename test (`tests/cook.rs:190-230`) renames the *file* with its GUID
kept, which is the identity criterion, not this case; no test renames an
Asset inside its source. Severity `correctness`: the ADR's promise of a
warning is not kept for the reference kinds the ADR's own example (a mesh
renamed in Blender) implies, and nothing is visible until a rename
happens. This is the open issue's claim read against the code; see *Open
issues read* for its placement.

#### L1-C02

Code, `cooked.rs:551` and `:586-593`:

```rust
let vertex_count = take_u32(rest)? as usize;
...
let vertex_bytes = take_u32(rest)? as usize;
let vertices = meshopt::decode_vertex_buffer(take(rest, vertex_bytes)?, vertex_count)?;
// The index codec spends at least a byte per triangle, so a count
// past that is corrupt and must not size an allocation.
anyhow::ensure!(
    index_count / 3 <= rest.len(),
    "Mesh Blob has more indices than its encoded bytes hold"
);
let indices: Vec<u32> = meshopt::decode_index_buffer(rest, index_count)?;
```

`meshopt-0.6.2/src/encoding.rs:78-91`:

```rust
pub fn decode_vertex_buffer<T: Clone + Default>(
    encoded: &[u8],
    vertex_count: usize,
) -> Result<Vec<T>> {
    let mut result: Vec<T> = vec![Default::default(); vertex_count];
    let result_code = unsafe {
        ffi::meshopt_decodeVertexBuffer(
```

The Rust wrapper allocates `vertex_count` elements before meshoptimizer
sees the encoded bytes, so the C decoder's size check comes too late to
stop the allocation. The index path guards exactly this: its comment
states the rule, and its bound comes from the codec's minimum bytes per
triangle. The vertex path, four lines earlier, applies no bound: a Blob
whose vertex count is corrupt (a torn write that `write_atomically`
prevents for the cooker's own files, a damaged disk, or an edited file)
allocates `vertex_count * 28` bytes, up to 112 GiB, and an allocation
failure aborts the process rather than surfacing as the `anyhow::Error`
the rest of `from_blob` returns. ADR 0018 frames Blob reading as refusal
with a message ("A mismatch is refused with a re-cook message"), and the
code's own comment extends that to corrupt counts; the vertex count is
the one count that escapes it. Severity `correctness`: the code disagrees
with its stated rule and the crate's documented behaviour, with no
visible defect on a healthy cache. Not reproduced by running, as the
brief forbids; the two quotes carry the claim.

#### L1-C03

Code, `material.rs:184-191`:

```rust
MaterialChannel::Albedo | MaterialChannel::Emissive => {
    (wgpu::TextureFormat::Rgba8UnormSrgb, [255; 4])
}
MaterialChannel::Normal => (wgpu::TextureFormat::Rgba8Unorm, [128, 128, 255, 255]),
MaterialChannel::MetallicRoughness | MaterialChannel::Occlusion => {
    (wgpu::TextureFormat::Rgba8Unorm, [255; 4])
}
```

`material.rs:156-159`:

```rust
/// Layer 0 of every Channel holds a 1×1 neutral texture: white for
/// albedo / metallic-roughness / occlusion / emissive, the flat normal,
/// x and y at 0.5, for normal. A material without a channel points here,
/// samples a no-op, and its factors pass through unchanged.
```

`src/render/shaders/shading.wgsl:156-160`:

```wgsl
fn tangent_space_normal(encoded: vec2<f32>, scale: f32) -> vec3<f32> {
    let x_and_y = encoded * 2.0 - vec2<f32>(1.0);
    let z = sqrt(max(1.0 - dot(x_and_y, x_and_y), 0.0));
    return vec3<f32>(x_and_y * scale, select(z, 1.0, scale == 0.0));
}
```

`main.wgsl:178` samples `normal_textures[material.layers.y]` for every
Material, Layer 0 included, and `:188-190` rotates the result into world
space and normalises it. `Unorm8` has no code for 0.5: 128 decodes to
128/255 = 0.50196, so `x_and_y` is (1/255, 1/255) = (0.0039, 0.0039), the
normal is (0.0039, 0.0039, 0.99998), and the shading normal of every
Material without a normal Texture is tilted atan(√2 / 255) = 0.318°
toward the tangent and bitangent, with `normal_scale` 1. The white
neutrals are exact (255/255 = 1), so the other four Channels are the
no-op the comment claims; the normal Channel is not. ADR 0017: "Layer 0
stays the neutral default." The magnitude is one 8-bit step in each of x
and y; its effect on `N·L` is at most sin(0.318°) ≈ 0.0056 of the light,
so under one 8-bit step at most exposures, which is why the class is
`correctness` and not `image`. The scenes without normal Textures
(`pbr-sweep`, `town`, `shadow`, `crowd`, the ground planes) all shade
through this Layer; a verifier with the M4 Pro can capture `pbr-sweep`
with the neutral replaced by an exact (0, 0, 1) and difference the two
to decide whether a pixel moves. Candidates for the fix: `Rgba8Snorm` with
(0, 0, 127), whose 0 is exact and whose decode the shader would take as
`encoded` already signed, or a `select` on `layers.y == DEFAULT_LAYER`
that skips the perturbation; a fix ticket decides.

#### L1-C04

Code, `material.rs:240-254`:

```rust
pub fn insert(&mut self, staging: &mut LoadStaging, data: MaterialData) -> MaterialHandle {
    if self.materials.len() as u64 >= MATERIAL_CAPACITY {
        tracing::error!(
            capacity = MATERIAL_CAPACITY,
            "material SSBO full; reusing material 0 (raise MATERIAL_CAPACITY)"
        );
        return MaterialHandle(0);
    }
```

`CLAUDE.md`, review checklist: "Streaming uploads go through the
`StagingBelt`, never `Queue::write_texture`; no GPU buffer grows at
runtime, and a request past a buffer's capacity is refused and logged."
The SSBO does not grow and the overflow is logged; it is not refused.
`load` (`material.rs:289`) returns `Ok(self.insert(...))`, so the loader
and the scene receive a valid-looking `MaterialHandle(0)` for the 257th
Material and every Renderable that holds it draws with whatever Material
loaded first, a wrong image that nothing but the log reports. The Mesh
pool's sibling path refuses (`pools.rs:188-194` returns `None`, and
`loader.rs:59-61` turns it into an error), and the spec's stories 43 and
44 ask the same of every fixed capacity: "refused and logged with nothing
written", "any overflow counted and warned about". No Reference scene
reaches 256 Materials at this commit (`tests/frame_stats.rs:82` sizes the
SSBO at 256), so there is no visible defect; the class is `correctness`
for the disagreement with the checklist's rule. The Texture Channel's
full path (`material.rs:103-110`, `:307-309`) substitutes Layer 0 the same
way, with its doc comment saying so ("or Layer 0 when the Channel is
full"); it is noted under *Notes for the verifier* rather than filed,
since ADR 0017's eviction replaces it and no ADR clause asks for refusal
there.

#### L1-C05

Code, `mesh.rs:158-164` and `:172-182`:

```rust
/// Each Meshlet's triangle count when `triangles`, indexing `vertex_count`
/// vertices, are cut in order into the longest runs the Meshlet limits
/// allow, like meshoptimizer's `meshopt_buildMeshletsScan`, which `meshopt`
/// binds only as an `unsafe` call and which counts a degenerate triangle's
/// repeated corner twice [ref:meshoptimizer-0.25-scan-meshlets].
...
let new_vertices = (0..3)
    .filter(|&corner| {
        !in_run[corners[corner] as usize]
            && (corner == 0 || corners[corner] != corners[corner - 1])
    })
    .count();
if counts
    .last()
    .is_none_or(|&count| count == MESHLET_MAX_TRIANGLES)
    || run_vertices.len() + new_vertices > MESHLET_MAX_VERTICES
```

meshoptimizer 0.25, `clusterizer.cpp:362-370` (the registry's quote):

```cpp
short& av = used[a];
short& bv = used[b];
short& cv = used[c];
...
int used_extra = (av < 0) + (bv < 0) + (cv < 0);

if (meshlet.vertex_count + used_extra > max_vertices || meshlet.triangle_count >= max_triangles || split)
```

For a triangle `(a, a, c)`, `av` and `bv` alias `used[a]`, so `used_extra`
counts the new vertex `a` twice, and at 62 vertices in the run with `a` and
`c` new meshoptimizer starts a new Meshlet (62 + 3 > 64) where the code
continues the run (62 + 2 ≤ 64). The code's `corners[corner] !=
corners[corner - 1]` on the sorted corners is the deliberate difference.
ADR 0018: "Each Meshlet is the longest run the limits allow, as
meshoptimizer's scan builder cuts them": the first half is what the code
does and the second is not, on this one case. `docs/audit/deviations.md`:
"A Deviation lives here and nowhere else: not in a code comment, where no
number may sit and nobody reads across the tree"; the comment at
`mesh.rs:162` is where it lives today, and the table is empty. Severity
`drift`: the Blob stores the counts the cut produced and the reader
rebuilds from them (`src/assets/cooked.rs:430-484`), so either cut loads
and draws; what is wrong is the ADR's equivalence and the row's absence.
The fix is a `deviations.md` row at `mesh.rs:meshlet_triangle_counts`, or
the ADR's clause narrowed to non-degenerate triangles.

#### L1-C06

`docs/adr/0018-...md:3`: "Partly implemented: ... the runtime loads
Materials and Textures (#150) and Meshes (#151) from them by Asset ID; the
reader rebuilds their Meshlets (#192), which the renderer does not keep
yet." Code, `src/assets/pools.rs:76-78`:

```rust
meshlet_arena: Arena,
meshlet_vertex_arena: Arena,
meshlet_triangle_arena: Arena,
```

created at `:106-120` ("Meshlet Arena", sized by `MeshletData`), written
with each loaded Mesh's Meshlets, Meshlet vertices and packed triangles at
`:272-286`, and bound by `src/render/renderer.rs:365-374` as `meshlets`,
`meshlet_vertices` and `meshlet_triangles` for the culling and main
passes. ADR 0017,
amended 2026-10-20: "and into three Meshlet arenas of the same size, for
the Meshlets, their vertices and their triangles"; ADR 0019: "the cooker
builds Meshlets of at most 64 vertices into each Mesh Blob, and the vertex
shader reads the record for its Instance, Material and vertex base". The
spec's Implementation Decision, ADR status lines: "Each ticket that changes
what an ADR's 'Planned; today …' sentence describes updates that
sentence." The draw-path ticket updated ADRs 0017 and 0019 and not ADR
0018's status. Severity `drift`, not load-bearing: the other two ADRs and
`CONTEXT.md`'s Meshlet entry carry the current form; the stale clause
misleads only a reader of ADR 0018 alone. The fix is one clause: the
Meshlets are kept in the Meshlet Arenas (ADR 0017) and drawn (ADR 0019).

#### L1-C07

Code, `src/components.rs:79-92`:

```rust
/// A point light at its entity's `Transform`.
#[derive(Component, Clone, Copy)]
pub struct PointLight {
    pub color: glam::Vec3,
    /// Luminous intensity, in candela (ADR 0023).
    pub intensity: f32,
    /// Metres from the light past which it adds nothing (KHR_lights_punctual's
    /// `range`, which must be positive and here finite).
    pub range: f32,
    /// Whether it casts a shadow: one that casts none takes no shadow ray
    /// and no place in the shadow mask, which then holds more of the lights
    /// that cast one.
    pub casts_shadow: bool,
}
```

Consumer, `src/render/prepare.rs:550-561`: the point lights are listed
"that cast a shadow, then those that cast none", and a light that casts none
is written with `unshadowed_range: light.range`, which the shaders read as
the range inside which no ray is traced and the light is unshadowed. ADR
0020: "Direct light is one hard shadow ray per pixel per light"; "One hard
shadow ray per pixel is traced toward each directional light ... skipped
where the light adds nothing ... and one toward each point light that
reaches the pixel, ending at the light". Spec story 27: "I want a shadow
ray skipped only where its light adds nothing (N·L ≤ 0, or beyond its
range), so that no ray is wasted and none is dropped". Spec, Out of Scope:
"Fog lines: a per-light shadow opt-out". The field is the opt-out the spec
fogged, landed by a later ticket whose intent the field's own comment
states (more mask bits for the lights that cast a shadow); ADR 0020 names
the 59-light cap and the edge rays as amendments but not this, and
`deviations.md` has no row. Every Reference scene sets `casts_shadow:
true` (`src/scenes/mod.rs:399`, `:501`; `src/scenes/town.rs:228`), so no
Golden, Baseline or Cycles comparison sees the opt-out, and the rendered
images follow the ADR; the Finding is the documents' silence against a
documented code decision, as L1-R04 is for ADR 0003. Not `correctness`:
the engine's behaviour on every scene is the ADR's. Magnitude: one field,
one consumer, zero scenes.

### Notes for the verifier

#### cook, with the manifest and the cook tests

Not filed, since each is a reading without a quoted clause it breaks, or
belongs to another reader; recorded so the next audit does not re-find
them.

- A Texture's Asset ID hashes the *image's* name (`mod.rs:380-384`,
  `image_names[texture.source().index()]`), where ADR 0018 says "glTF
  name". A glTF `texture` is a sampler over an image, so two textures over
  one image collapse to one Asset per Channel, which is the useful
  reading; the ADR does not say which object's name. The function's doc
  at `mod.rs:369-372` states the choice.
- The Toksvig oracle's expected value (`tests/cook.rs:504-508`) is the
  code's own two-quote composition with an analytic mean length, so it
  checks the implementation, not the model. The substitution of Toksvig's
  angular variance for Kaplanyan's slope variance (`texture.rs:459-461`,
  "tan θ ≈ θ") is unchecked by any oracle: at the test's largest tilt
  (four tilts of 10°, 40°) tan θ exceeds θ by 20%. ADR 0018 names Toksvig
  and does not say how the variance widens GGX, so no clause is broken;
  an owner who wants the model checked would set a reference widening
  from a filtered GGX lobe, which is the shaders lane's kind of oracle.
- `linearise`'s sRGB decode (`texture.rs:231-240`) has no direct oracle:
  the checker test exercises the encode side, since 0 and 255 decode
  exactly to 0 and 1. A one-line test of 0.5 encoded (188) against
  the table's decode of 188 (0.5029) would close it.
- The three encoder version strings in the Cook keys (`texture.rs:21`
  `intel_tex_2 0.5.0 basic`; `mesh.rs:31` `bevy_mikktspace 1.0.0,
  meshoptimizer 0.25`) are hand-pinned and match `Cargo.lock` and the
  vendored `meshoptimizer.h:15` today; a dependency bump without the edit
  would leave Blobs encoded by the old version fresh. A test pinning the
  strings to `cargo metadata`'s versions would make the check mechanical.
  The zstd version is in the Mesh key (`mesh.rs:32`) and not the Texture
  key; neither changes decoded texels, so the asymmetry is harmless.
- The glTF import refuses a Primitive without normals (`mesh.rs:354-357`),
  reads `TEXCOORD_0` for every Channel (`:358-361`) whatever a texture's
  `texCoord`, ignores `KHR_texture_transform`, and recomputes tangents
  even where `TANGENT` is authored (`:124-129`). glTF 2.0 asks a client to
  generate flat normals and to use authored tangents; the registry quote
  `gltf-2.0-mikktspace-tangents` is for the case where tangents are
  absent. No ADR clause or Reference asset is touched (every manifest
  model has normals and one UV set), so this is importer scope, not a
  defect; a Finding would need a source the code claims to follow.
- A metallic-roughness image shared by two Materials with different normal
  Textures is widened by the first Material's (`mod.rs:433-435`,
  `:280-282`), since the Texture Asset is per image and Channel. The ADR
  says only that the metallic-roughness Texture depends on the normal
  Texture. The owner may want this in `deviations.md` or the ADR if such
  content arrives.
- `Level::reduce` drops the last row or column of an odd side
  (`texture.rs:331-332`, "a 3-tap filter there if non-power-of-two
  Textures show it"). ADR 0018's "2×2 box filter" is silent on
  non-power-of-two sizes, which the multiple-of-4 rule admits (12 → 6 →
  3 → 1). No Reference Texture is non-power-of-two.
- The manifest test checks `-NC-` for `dev_only` (`tests/asset_manifest.rs:191-193`)
  and not "a restrictive `LicenseRef-`" (`docs/research/test-assets.md`,
  Sourcing strategy 4), which it cannot tell from `LicenseRef-LegalMark-*`
  mechanically. No such entry exists.
- The identity criterion's render half
  (`tests/cook.rs:729-781`) needs a GPU adapter and skips without one; CI
  runs it on lavapipe (tooling).
- The PSNR floor (`tests/cook.rs:341-343`) is 44.0 dB where ADR 0018
  reports 44.7 dB as DamagedHelmet's lowest BC7 Texture and 52.86 dB for
  SciFiHelmet; the gap is the margin, and the BC5 and BC4 figures (42.71
  to 44.11) are printed only. A re-measurement is the performance lane's.
- `cook_scene` cooks every Asset of a referenced glTF file, not only the
  Assets the scene loads (`mod.rs:229-257`), a superset of ADR 0018's
  "the Assets it references"; the ADR's purpose, a quick dev launch,
  holds since a file's Assets are the unit of change detection.

#### assets

Not filed, since each is a reading, needs a run, or belongs to another
lane; recorded so the next audit does not re-find them.

- L1-C03 could be `image`: a `pbr-sweep` or `shadow` capture on the M4
  Pro with Layer 0's normal replaced by an exact (0, 0, 1), differenced
  against the Golden, decides. The 0.318° tilt is arithmetic from the
  quoted lines, not a measurement.
- The Texture Channel's full path (`material.rs:103-110`, `:307-313`)
  substitutes Layer 0 with an `error!` and, since the failed ID is not
  recorded in `texture_layers`, decodes and logs again on each later
  request for the same ID. 13,107 Textures per Channel is beyond every
  scene; ADR 0017's eviction replaces the path. Not filed.
- ADR 0002 says "one shared index buffer" for draws, BLAS and instancing;
  since ADR 0019 the draw reads packed indices and the Meshlet triangle
  Arena, and the index Arena is `BLAS_INPUT` only (`pools.rs:103-105`).
  ADR 0002 records ADR 0017's amendment and not ADR 0019's split. The
  render reader marked the clause as falling here; this reader reads it as holding
  on its purpose (one buffer per kind, addressable by offset) with the
  ADR's text behind, the same shape as the render section's L1-R04. Left
  for the verifier to decide whether a `drift` Finding is wanted; the
  evidence is the two lines above and ADR 0002 line 3.
- `Arena::allocate(0)` is reached for every `MeshUse::Traced` Mesh (no
  Meshlets) and `MeshUse::Drawn` Mesh (no index stream):
  `offset-allocator-0.2.0/src/lib.rs:199-291` serves a zero-size request
  by taking a node and returning the whole remainder, so it costs one of
  the Arena's 131,072 allocation nodes (`:143-144`) and no bytes;
  `MESH_CAPACITY` is 65,536, so the nodes cannot run out first. Not a
  defect; noted because the refusal test covers nonzero counts only.
- Load-time uploads through `LoadStaging` (`pools.rs:265-291`,
  `material.rs:128`, `:250`) are not added to `uploaded_bytes`; the
  checklist names per-frame writes, and these happen at spawn, before the
  first frame. If the verifier reads the checklist as covering every GPU
  write, the counter at `src/render/renderer.rs:548-551` is the place.
- `CookedTexture::from_blob` (`cooked.rs:354-360`) requires `height > 0`
  and both sides multiples of 4, so a width of 0 passes and would reach
  `create_texture` as a validation error, not a refusal. The cooker cannot
  produce it (`image` decodes no zero-width PNG), so it is unreachable
  from a cooked file; a hand-edited Blob is the only path.
- The reader does not check a Texture Blob's format against the Channel
  it is loaded into (`cooked.rs:342-350` accepts any of the three;
  `material.rs:307` pushes it into `channel`). The Asset ID carries the
  Channel, so a cooked cache cannot mix them; ADR 0018's per-Channel
  formats are enforced by the cooker alone (`src/cook/`).
- `DEFAULT_MODEL` (`src/state.rs:35`) is a source path the runtime holds:
  it is formatted into an error message (`:203-206`) and tested with
  `exists()` by the bench's scene list (`src/bin/praxis.rs:435`) to warn
  that the helmet is not fetched. Neither opens the file; "the runtime
  still opens no source path" holds. A stricter reading (no source path
  named at all) is the owner's.
- `src/render/staging.rs:211-215` asserts that a level's bytes match its
  row pitch rather than refusing; unreachable from `material.rs:127-129`
  because `from_blob` checks every level's size first
  (`cooked.rs:371-378`). Render section's file.
- Story 44's "counted": a refused Mesh is logged (`pools.rs:214-228`) and
  not counted in Frame stats; `arena_bytes_in_use` is reported
  (`src/state.rs:306`). The Arena refusal test predates M3 and the spec
  lists it as prior art, so this reader reads the story's counting as applying to
  the per-frame buffers it names.
- `MaterialPool::materials: Vec<MaterialData>` (`material.rs:142`) is a
  CPU copy of the SSBO, read only by a test (`:446`) and for `len()`; ADR
  0017's "CPU shadow ... goes" names vertex and index data only, so it is
  not a clause break. A health lane may want the `Vec` replaced by a
  count.
- `tests/cook.rs:677-681` keeps SciFiHelmet's 13.83 bytes per triangle as
  story 8's ceiling; re-measuring it against the Blob format at this
  commit is the cook reader's.

#### scenes, components, transforms and the fixed loop

Not filed, since each is already filed elsewhere, needs a run or an owner's
reading, or breaks no clause; recorded so the next audit does not re-find
them.

- `Sway::curvature(&self, seconds) -> f32` (`src/scenes/mod.rs:1046-1053`)
  is a method on a component, called by the `sway` system (`:1070`) and the
  crowd's spawn (`:1002`): the same letter of ADR 0006 that L1-R03 files
  for `Transform::to_affine`, `Bend::drawn_curvature` and
  `WorldTransform::{translation, drawn}`. L1-R03's magnitude is then four
  components and four methods at six call sites, not three at four.
- `Teleported` is inserted by no production system at this commit
  (`grep -rn Teleported src` finds `src/transform/mod.rs` and the State
  test at `src/state.rs:528` only), so `reset_teleported` runs every Tick
  over an empty query and the clause is held by
  `a_spawned_or_teleported_entity_is_never_drawn_between_its_old_and_new_place`
  alone (`src/state.rs:540-561`). A mechanism without a user is not a
  defect under this lane; the health lane may weigh it.
- The refit-form crowd figure carries `PreviousWorldTransform` though the
  fixed schedule never changes its Transform (`src/scenes/mod.rs:1014-1023`:
  "A Moving Instance, so it joins the dynamic TLAS, rebuilt each frame over
  its BLAS's bent bounds"). ADR 0020 decides it ("its Instance a Moving
  Instance"); ADR 0022's "Static Instances store nothing extra" is not
  broken, since the figure's BLAS changes each frame; `CONTEXT.md`'s
  Moving Instance, "An Instance whose entity the fixed schedule moves", is
  narrower than the code. A glossary reading for the docs lane.
- The first Tick a scheduled `propagate_world_transforms` runs after
  `State::new`'s `run_system_once` (`src/state.rs:104-108`) sees every
  Transform as changed, since the schedule's system instance has its own
  change ticks, and recomputes every World transform once, which marks
  every `WorldTransform` changed for the next prepare. At 60 Hz the first
  Tick lands in frame 0, whose prepare writes every Instance anyway, so
  `tests/frame_stats.rs:337-343` sees 10,000 writes from frame 1; at a
  frame delta under a Tick the full rewrite would land in a later frame.
  No clause is broken; a cost note for the performance lane on `town`
  (254,256 Instances) at refresh rates above 64 Hz.
- `WorldTransform::drawn` lerps the rotation-scale matrix, so a rotation of
  θ per Tick draws the entity shrunk by up to 1 − cos(θ/2) mid-Tick
  (`src/transform/mod.rs:18-20`: "0.1% at 0.09 rad (5.8 rad/s)"; checked:
  1 − cos(0.045) = 1.01e-3, 0.09 × 64 = 5.76 rad/s). The orrery's fastest
  body, 4.5 rad/s, shrinks by 0.06%; `town`'s flocks, at most 0.6 rad/s,
  by 0.001%. ADR 0022 names "a blend" without its form and cites no source
  for it, so this is not a deviation from a source; whether the number
  belongs in `deviations.md` rather than the comment is a triage reading
  of `docs/audit/deviations.md`'s rule that a number sits in no comment.
- Judder at 120 and 144 Hz (ADR 0022, "judder-free at refresh rates that
  do not divide 64 Hz") is a visual check on the window; the capture path
  proves only the orrery's 144 Hz frames land between Ticks.
- The spec's "scene reporting average lights per lit pixel" for `town` is
  the light count pass under a Render switch, the render and tooling
  sections'.
- `src/scenes/town.rs:222-223` says the lamp light gives "the sun's
  illuminance 4.5 m away": 2,000,000 cd / 4.5² = 98,765 lx, 1.2% under;
  "nearly as much on the ground below", 4.7 m down, is 90,539 lx. Prose
  approximations in a comment, not filed.
- The crowd's proxies must stay inside the figure unbent
  (`each_bones_shadow_proxy_lies_inside_its_figure`) but no test places
  them inside a bent figure; the proxies and the bend pass share
  `bent_middle`'s arc, so a bent proxy follows the middle, while its width
  is the unbent spheroid's. Whether a bent figure's own shadow rays leave
  it clear of its proxies at the sway's most curvature is a GPU run on
  `crowd` (the proxies form against the refit form, which `tests/crowd.rs`
  compares only refit against rebuild).

### Counts

| Severity class | Findings |
|---|---|
| `image` | 0 |
| `budget` | 0 |
| `correctness` | 4 (L1-C01, L1-C02, L1-C03, L1-C04) |
| `drift` | 3 (L1-C05, L1-C06, L1-C07) |
| `health` | 0 |

Clause verdicts: 174 rows over the three readers (54, 74, 46); `holds`
143, four of them with a Finding on the clause's form or on one half of
its text (L1-C03, L1-C05, L1-C06, L1-C07); `broken` 3 (L1-C01 in part,
L1-C04, and ADR 0006's letter as the render section's L1-R03, with a
fourth component method noted for its magnitude); `drift` 3 rows, two of
them the one Finding L1-C06 and one L1-C07; `superseded` 0; deferred by
the spec or the ADR 10; falls in another module or lane 11; not
applicable 4. Spec stories and decisions: 41 rows (10, 12, 19); 35 hold in
this module, 1 is superseded by ADR 0018 as amended, 1 carries L1-C07 on
its component side, 4 fall in render, tooling or the shaders. Registry
keys: 21 rows (13, 4, 4), 19 distinct keys cited in the module's files,
all match. Oracles: 18 rows (9, 1, 8), all hold; every measured error is 0
or bounded by analysis inside the test's constant, none run. Open issues
read: 1, filed as L1-C01 with the expectation that it inherits the
issue's placement.

## shaders

Scope: `src/render/shaders/`: the sixteen WGSL parts (2,643 lines) and
`mod.rs` (292 lines, how the parts assemble into modules and what Rust
declares into them), with `src/render/shader_function_tests.rs` (2,296
lines, the 28 oracle tests and their GPU harness) and
`src/render/shader_tests.rs` (741 lines); every file read in full. Four
readers at high effort shared the module by file, each with the lane
brief verbatim: A, surface shading (`shading.wgsl`, `main.wgsl`); B, HDR
and display (`hdr.wgsl`, `gt7.wgsl`, `display.wgsl`, `tonemapping.wgsl`);
C, geometry, culling, pyramid, Froxels and bend (`common.wgsl`,
`culling.wgsl`, `depth_pyramid.wgsl`, `froxel_grid.wgsl`, `froxel.wgsl`,
`light_count.wgsl`, `bend.wgsl`, the two marker parts, `mod.rs`,
`shader_tests.rs`); D, the shadow mask (`shadow_mask.wgsl`,
`segment_meets_box`). Every reader read `common.wgsl`. ADRs read: 0014,
0015, 0019, 0020 and 0023 clause by clause, with the `CLAUDE.md` review
checklist's shader clauses as their operational form; the clauses the
render section marked "falls in shaders" are answered here. Sources: glTF
2.0 Appendix B and KHR_lights_punctual as `docs/references.toml` quotes
them, and every other key a part cites, opened and read against its
quote. For every oracle the reader derived the WGSL accuracy floor of the
expression chain the test exercises, from the registry's precision quotes
(`higham-3.1`, `vulkan-spirv-precision`, `msl-4.1-fast-math`,
`msl-4.1-rounding`, `d3d11.3-fp32-rules`), and wrote it beside the test's
constant under *Oracles*, with a verdict `explained`, `tight`, `loose` or
`unexplained`; a measured error is the one the test file states, since
readers run nothing. The session ran the 28 oracle tests once on the
frozen tree before the readers started (Apple M4 Pro, Metal, battery at
84% and charging, load average 2.08, peers idle on the GPU): 28 passed in
1.4 s. Every `where` is `path:line` at the frozen commit, under
`src/render/shaders/` unless given in full. `deviations.md` holds no row
at this commit, which two Findings turn on; `DIFFERENCES.md` was read for
what is not a Finding. Two readers filed the same drift on ADR 0014's
culling-pass sentence; the section keeps one Finding, L1-S05, and both
readers' clause rows.

### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0019 | "a lit pass at depth equal" | holds, shader side: every depth entry point computes its position through one `vertex_output` and declares it `@invariant`, so the Equal test (render) meets the pre-pass's depths bit for bit | `main.wgsl:8`, `:17`, `:27`, `:85-102` |
| 0019 | "that loops over its Froxel's lights" | holds | `main.wgsl:269-299` |
| 0019 | "and reads each one's shadow from a shadow mask" | holds: one `textureLoad` of the surface's word per fragment, a bit per light | `main.wgsl:233-239`, `:290` |
| 0019 | "lit by every directional light" | holds: the first shaded unconditionally (its illuminance zero without one), the rest in a loop to `directional_light_count` | `main.wgsl:212-261` |
| 0019 | "and by the point lights of each pixel's Froxel" | holds for the read: `froxel_at` at the fragment's pixel centre and its clip w, the Froxel's list walked in light order; the list's build falls in shaders C | `main.wgsl:272-283`; `froxel_grid.wgsl:34-41` |
| 0019 | "which a compute pass at the start of the frame lists from a light buffer of fixed capacity: those whose range reaches into the Froxel's bounds" | falls in shaders C | |
| 0019 | "each hidden where the shadow mask says so, the directional lights' rays first, then those point lights'" | holds: a directional light's bit is its index, a point light's `directional_light_count + entry` in the Froxel's list, the order the mask writes them | `main.wgsl:240-241`, `:252-253`, `:286-290`; `shadow_mask.wgsl:7-10`, `:270` |
| 0019, 0020 | "up to 59 lights per pixel; a light past them casts no shadow there" | holds: `masked` is false for any bit at or past `SHADOW_MASK_LIGHTS`, `UNMASKED` included, so the light is summed unshadowed | `main.wgsl:118-126`, `:163` |
| 0019 | the mask's word follows the Froxel of the sample it was traced from, its slice in the second half's top bits (the amendment's "each fragment reads the bits traced from the surface it shades") | holds: the fragment reads the slice from `mask.y >> 27`, rebuilds that Froxel in its own screen rectangle and finds each light's bit by binary search; `masked` stops at bit 58, so a slice bit is never read as a light's; the mask's rectangle is the pixel centre's, so only the slice can differ | `main.wgsl:146-164`, `:276`, `:287-289`; `shadow_mask.wgsl:262`, `:351-352` |
| 0019 | "each fragment reads the bits traced from the surface it shades: the nearest sample's, or, where the fragment covers a sample off that sample's plane, the farthest such sample's ... a nearest-surface fragment that runs under a nearer surface at another sample reads that surface's" | holds: `nearest_surface = (covered & samples) == 0`, layer 1 otherwise, from the fragment's `sample_mask` | `main.wgsl:171`, `:229-239` |
| 0019 | "A fragment stage that reads any flat input is never merged ... the depth passes' normals and the lit pass ... both read a flat input" | holds: `fs_main` indexes the Material table by the flat `material_id`; `fs_surface_normals` reads it into a select the compiler cannot drop; `fs_depth` reads none, and with rays off no pass must agree with it on a sample's triangle | `main.wgsl:12`, `:30`, `:69-72`, `:172`; `src/render/main_pass.rs:116-149` |
| 0019 | "the vertex shader reads the record for its Instance, Material and vertex base, then pulls its vertex from the vertex Arena" | holds for the vertex shader's use; the packing, `decode_vertex` and their oracles fall in shaders C | `main.wgsl:85-101` |
| 0019 | "Each Primitive draws with its own Material unless its Instance overrides it" | holds for the read: the record's Material ID, which the culling pass resolves (shaders C) | `main.wgsl:113`, `:172` |
| 0019 | "the depth passes also write each sample's triangle normal and the normal interpolated over it, 4 bytes per sample" | holds: `fs_surface_normals` writes both, octahedral-encoded into two channels each, the triangle's from the position's derivatives turned toward the camera, an edge-on triangle and a zero interpolated normal given a direction | `main.wgsl:44-73` |
| 0019 | "a transient 4x colour attachment ... that it resolves into the `Rgba16Float` HDR target" | holds for the store: the lit pass writes `resolve_weighted(exposed)`; the weighting falls in shaders B | `main.wgsl:306` |
| 0019 | "with alpha-to-coverage for masked materials" | deferred by the M3 spec to M6; the shader writes `surface.opacity` as alpha and nothing reads it as coverage | `main.wgsl:306` |
| 0019 | "a compute pass traces [the mask] from the depth target ... into the mask's second layer" | falls in shaders D | |
| 0019 | "appends ... in slot order, in two phases"; Hi-Z test; index packing | falls in shaders C | |
| 0020 | "one hard shadow ray per pixel per light ... skipped where the light adds nothing" | falls in shaders D; the lit pass's half holds: a light at or below the horizon contributes `max(N·L, 0) = 0` | `shading.wgsl:220` |
| 0020 | "the fragment shading it by the share of its samples that see it" | holds: the hidden verdicts over the covered samples, counted, over the covered samples, for the first `SHADOW_EDGE_LIGHTS` directional lights at the nearest surface; the mask's bit otherwise | `main.wgsl:134-144`, `:240-241`, `:252-253` |
| 0020 | "the surface's one bit for that light is the verdict of a ray from the first such sample" | falls in shaders D; consumed here as one bit | `main.wgsl:290` |
| 0020 | "A constant ambient term stands in for indirect light" | holds: E/π times base colour times occlusion, E the scene's ambient illuminance | `shading.wgsl:260-268`; `main.wgsl:301` |
| 0020 | "Direct light is Temporal class (a)" | holds: the lit pass reads this frame's mask, lists, lights and Textures only; no history | `main.wgsl:170-307` |
| 0020 | "irradiance probes ... shaded from position and normal in ADR 0019's lit pass" | not applicable: probes are M5's | |
| 0019 | "Metal's ray tracing in render pipelines is not compatible with mesh shading, which kept the lit pass from being one until [the shadow mask ticket] moved its rays into the shadow mask pass" | holds: `main.wgsl` has no ray query; the main module enables `wgpu_binding_array` only | `src/render/shaders/mod.rs:87-93` |
| 0023 | "a `DirectionalLight` gives illuminance in lux" | holds: its colour times lux enters `reflected_luminance` as E, the illuminance on a surface facing it, f(L, V) E (N·L)⁺ | `main.wgsl:240-248`; `shading.wgsl:196-221` |
| 0023 | "a `PointLight` luminous intensity in candela" | holds: E = I · window(d) / d² | `shading.wgsl:228-258` |
| 0023 | "emissive and background give luminance in cd/m²" | holds for emissive: the texel times the factor is added to the outgoing luminance before exposure; the background is render's | `main.wgsl:301-302`; `shading.wgsl:88` |
| 0023 | "a Reference scene's ambient term gives illuminance in lux" | holds: divided by π into a Lambertian surface's luminance | `shading.wgsl:267` |
| 0023 | "Exposure is an EV100, which scales luminance by 1/(1.2 · 2^EV100)" | holds for the application: luminance times `camera.exposure`, once; the scale is render's | `main.wgsl:305` |
| 0023 | "Shading runs in f32 and pre-exposes colour before fp16 stores it, and every path clamps it to 65504" | holds for the lit pass: f32 throughout, `min(luminance · exposure, LARGEST_HALF)` before the weighted store, which only lowers it | `main.wgsl:303-306`; `hdr.wgsl:7`, `:23-25` |
| 0023 | "Point lights fall off by inverse square inside KHR_lights_punctual's range window" | holds; one deviation within 1 cm, documented in a comment and not in `deviations.md`: L1-S02 | `shading.wgsl:226`, `:234-238`, `:254-256` |
| 0023 | "Core glTF caps emissive at 1 cd/m² ... Brighter emissives need KHR_materials_emissive_strength, which the cooker does not read yet" | holds: the shader multiplies the texel by `material.emissive.rgb` and ignores `.w`, which the loader sets to 1 | `shading.wgsl:88`; `src/assets/material.rs:284` |
| 0014 | "point lights reach the GPU as an anchor plus an offset, which the Froxel pass and the lit pass compose camera-relative" | holds for the lit pass: `point_light_position` composes through `camera_relative_position`, then the light's offset from the fragment is the difference of two camera-relative positions; the Froxel pass's is shaders C's | `main.wgsl:285`, `:295`; `common.wgsl:27-40`, `:119-133` |
| 0014 | "The GPU never holds a World position in f32 ... shaders compose positions relative to the camera" | holds: the vertex shader subtracts anchors in integers, then offsets, then adds the rotated local position; the fragment's position, view direction and light offsets are camera-relative | `main.wgsl:95-101`, `:206`, `:295` |
| 0014 | "The vertex shader composes camera-relative positions until draws are GPU-driven; then the culling pass does it once per Instance" | broken in letter, `drift`: draws are GPU-driven and the vertex shader still composes per vertex; the culling pass composes only its bounds centre and hands the vertex shader nothing: L1-S05 | `main.wgsl:95-101`; `culling.wgsl:123-126` |
| 0014 | a directional light has no position; "its offset holds the direction it shines along" | holds: `normalize(-offset)` is the direction to the light; no anchor is read | `main.wgsl:214-218`, `:251`; `common.wgsl:157-159` |
| 0015 | "Depth is cleared to 0 and compared with `Greater` ... with no far plane" | not applicable in this letter's files: the lit pass compares Equal (render) and reads no depth; its one depth-derived value is the Froxel slice from clip w, which has no direction | `main.wgsl:273` |
| 0015 | "a Hi-Z pyramid keeps the minimum depth" | falls in shaders C | |
| checklist | "calls no `textureSample` under non-uniform control flow" | holds: the five samples are the first statements of `fs_main`, before any branch; `if SHADOW_RAYS` and `if FROXEL_LIGHTS` are pipeline overrides, uniform; the light loops hold `textureLoad` and storage reads only | `main.wgsl:166-178`, `:233`, `:272`, `:279-299` |
| checklist | "no divide-by-zero or NaN at grazing angles or zero roughness" | holds: roughness floored at 0.04 after widening; D's denominator held at α²; V's denominator and χ⁺ in one select; H from `inverseSqrt(max(…, 1e-30))`; d² floored at 1 cm²; the range enters only the window. The one unguarded `normalize` is of the interpolated normal and tangent (Notes) | `shading.wgsl:86`, `:116-121`, `:144`, `:209`, `:235-237`, `:254`; `main.wgsl:183-190` |
| checklist | "WGSL keeps sRGB and linear apart" | holds: colour Layers are BC7 sRGB formats the sampler decodes, data Layers UNORM; linear factors multiply linear texels; shading encodes nothing | `main.wgsl:173-178`; `src/cook/texture.rs:162-170` |
| checklist | "HDR colour is pre-exposed before fp16 stores it and clamped to 65504" | holds (the ADR 0023 row above) | `main.wgsl:305` |
| checklist | "builds the frame's draw in slot order ... never in the order atomics land" | falls in shaders C | |
| checklist | "Matrices reach WGSL column-major" | holds for the uses: `view_proj * vec4`, `rotation_scale * position`, `normal_matrix * normal`, column-vector products; the Rust side is render's | `main.wgsl:100-109` |
| checklist | "A new or changed shading function in WGSL has an oracle test" | holds for `shading.wgsl`: all eleven functions have one; `masked`, `directional_hidden_share` and `froxel_mask_bit` in `main.wgsl` have none (Notes) | `shader_function_tests.rs:20-652`, `:1298-1456` |
| checklist | "Identifiers are full words, WGSL included" | holds: tested; no abbreviation in either file | `tests/conventions.rs:192-214` |
| checklist | comments cite "only an upstream bug, adapted code's source and licence, a pinned third-party source or an ADR" | holds: Filament's Apache-2.0 at the two adapted sites, glTF's CC BY 4.0, Tokuyoshi "Equations only; no code copied" | `shading.wgsl:3`, `:37-38`, `:107`, `:224` |
| 0023 | "a `DirectionalLight` gives illuminance in lux and a `PointLight` luminous intensity in candela ... emissive and background give luminance in cd/m² ... ambient ... in lux" | falls in shaders A (`shading.wgsl`, `main.wgsl`) and render | |
| 0023 | "Exposure is an EV100, which scales luminance by 1/(1.2 · 2^EV100) so that 1.0 lands near display white" | holds, the display half: `tonemap` multiplies exposed colour by `GT7_PAPER_WHITE`, 2.5, so exposed 1 is GT7's 250 cd/m² paper white; the curve takes 2.5 to 2.0878806 and the division by 2.5 gives 0.83516 display-linear, sRGB code 236 of 255. The 1/(1.2 · 2^EV100) scale is render's | `display.wgsl:22-25`; `gt7.wgsl:32`, `:123`; `shader_function_tests.rs:1002` (`[1.0, 1.0, 1.0]` → 0.8351601) |
| 0023 | "each scene has an EV100, which the lit pass applies before the `Rgba16Float` HDR target stores its colour, clamped to 65504" | holds: `luminance * camera.exposure` is clamped to `LARGEST_HALF` and only then weighted for the resolve, so the stored value is finite whatever the shading produced; the weighting itself caps a channel at 1/0.0722 = 13.85, so the clamp's work is to keep an f32 overflow out of the weighting | `main.wgsl:303-306` (letter A's file; the constant is this module's); `hdr.wgsl:4-7`, `:23-25` |
| 0023 | "the tonemapping pass then maps it through GT7 for display" | holds: unweight, `tonemap`, which is GT7 between the Rec. 709 to Rec. 2020 conversions, then sRGB | `tonemapping.wgsl:18-19`; `display.wgsl:22-25`; `gt7.wgsl:115-124` |
| 0023 | Consequences: "Shading runs in f32 and pre-exposes colour before fp16 stores it, and every path clamps it to 65504, since on overflow the M4 Pro saturates while Vulkan may store infinity" | holds in the shaders: the constant carries WGSL's 65504 and Vulkan's permission to round to infinity; the lit pass clamps before the store; the inverse of the weighting clamps its pole to 65504 too. The clear's clamp is render's | `hdr.wgsl:4-7`, `:27-31`; `main.wgsl:305` |
| 0023 | Consequences: "a dielectric's sun glint at roughness 0.04 peaks at 1.24×10⁸ cd/m², 10.9 stops past fp16's 65504" | holds, arithmetic: log₂(1.24 × 10⁸ / 65504) = 10.89 | `docs/adr/0023-photometric-light-units.md`, Consequences, first bullet |
| 0023 | Consequences: "Core glTF caps emissive at 1 cd/m² ..., which an exposure of EV100 15 maps to 2.5×10⁻⁵ of display white" | holds, arithmetic: 1/(1.2 · 2¹⁵) = 2.54 × 10⁻⁵; `surface.emissive` joins the luminance before the exposure multiply, so it is pre-exposed with the rest | `main.wgsl:302-305` |
| 0023 | Consequences: "Each Reference scene sets its EV100 by hand in M3 ... Histogram adaptation and local tone mapping wait" | holds: the shaders read one exposure scalar from the camera uniform and no history, histogram or neighbourhood | `common.wgsl:13-22` (`exposure: f32`); `tonemapping.wgsl:16-24` reads one pixel of one target |
| 0019 | "A lit pass draws ... into a transient 4x colour attachment ... that it resolves into the `Rgba16Float` HDR target" | holds, the shader half: what the hardware's box resolve averages is colour weighted by 1/(1 + luminance), so the average lands in a display-like space and a bright sample cannot whiten its edge; the weighting is undone in the tonemapping pass | `hdr.wgsl:16-31`; `main.wgsl:306`; `tonemapping.wgsl:18-19` |
| 0019 | "which a tonemapping pass then writes into the target" | holds: one full-screen triangle, one `textureLoad` of the HDR target per pixel | `tonemapping.wgsl:5-10`, `:15-24` |
| 0019 | every other clause: Froxel reads, flat inputs, depth equal, two culling phases in slot order, the Hi-Z test, index packing, the shadow mask's reads | falls in shaders A, C and D | |
| 0014 | "shaders compose positions relative to the camera" and the precision bar | not applicable to these parts, which hold no position; the composition is shaders C's | |
| 0015 | reverse-Z, cleared to 0, `Greater`, the pyramid's minimum | not applicable: the tonemapping module has no depth; the direction is shaders C's and D's | |
| 0020 | one hard ray per pixel per light, the edge rays, the ambient stand-in | falls in shaders A and D | |
| checklist | "sRGB is encoded once: the tonemapping pass alone encodes it, in its shader, into 8-bit UNORM targets, so its dither lands on 8-bit steps" | holds: `srgb_encode` is the one sRGB expression in every WGSL part (the only `12.92`, `1.0 / 2.4` and `0.0031308` in `src/render/shaders/`), called once, and the noise is added as `noise / 255.0` to the encoded value, one code either way, into the 8-bit UNORM the Rust side asserts | `display.wgsl:30-37`; `tonemapping.wgsl:19`, `:22-23` |
| checklist | "WGSL keeps sRGB and linear apart" | holds: every operation before `tonemapping.wgsl:19` is linear (weighting, GT7's frame-buffer units, the primaries conversions); the dither alone is added in the encoded domain, which the HDR survey's row 19 decides on purpose, so that its amplitude is one code everywhere | `tonemapping.wgsl:18-23`; `display.wgsl:39-51` |
| checklist | "no divide-by-zero or NaN" | holds: `1 + luminance ≥ 1`; `complement ≥ 1/65504`; `PQ_C2 − PQ_C3 · power ≥ 0.164` since `power ≤ 1`; `pow` takes `max(.., 0)` or a clamp; the dither's `sqrt(1 − \|centred\|)` has `\|centred\| ≤ 0.998`; the remaining divisors are literals | `hdr.wgsl:24`, `:30-31`; `gt7.wgsl:54-57`, `:63`, `:71`, `:77-79`, `:120`, `:123`; `display.wgsl:31-33`, `:48-50`; `tonemapping.wgsl:23` |
| checklist | "calls no `textureSample` under non-uniform control flow" | holds: the module samples nothing; two `textureLoad`s in uniform control flow | `tonemapping.wgsl:18`, `:22` |
| checklist | "A new or changed shading function in WGSL has an oracle test" | holds: `relative_luminance`, `resolve_weighted`, `resolve_unweighted`, `gt7_curve`, `pq_encode`, `pq_decode`, `rec2020_to_ictcp`, `ictcp_to_rec2020`, `gt7_tonemap`, `tonemap`, `srgb_encode`, `dither_noise` each sit under one of the ten tests; the two entry points are under the pass's GPU tests, render's | `shader_function_tests.rs:657-1180` |
| checklist | "A binding is written once ... no WGSL part declares one" | holds for these parts: no `@group` or `@binding` in the four files; `hdr_target` and `dither_mask` arrive from the bind group's definition | `tonemapping.wgsl:1-3`; `shaders/mod.rs:136-142` |
| checklist | "Matrices reach WGSL column-major" | holds for the WGSL constants: each `vec3` of `REC709_TO_REC2020` is a column, a Rec. 709 primary's Rec. 2020 coordinates, so `M * exposed` applies BT.2087's rows; no CPU writes these | `display.wgsl:8-17`, `:23-24` |
| checklist | slot order never atomic order; depth cleared to 0 and `Greater`; no World position in f32 | falls in shaders C and D; not applicable here | |
| 0014 | "The GPU never holds a World position in f32: it reads an integer anchor on a 64 m grid plus an f32 offset, and shaders compose positions relative to the camera" | holds: one composition function, the anchor difference in i32 then f32 times 64 (exact below 2²⁴ m), every caller passes an anchor and offset | `common.wgsl:31-40`; callers `culling.wgsl:123-129`, `froxel.wgsl:38`, `light_count.wgsl:54`, `common.wgsl:121-133`; `mod.rs:238`, `:263` declares `ANCHOR_CELL_SIZE` 64.0 from `instance.rs:8` |
| 0014 | "point lights reach the GPU as an anchor plus an offset, which the Froxel pass and the lit pass compose camera-relative" | holds for the Froxel pass; the lit pass's call falls in shaders A | `common.wgsl:119-133`; `froxel.wgsl:38`; `light_count.wgsl:54` |
| 0014 | "a shader-function oracle holds the composition to the precision bar in every evaluation order within 100 km, and shows f32 World positions failing it" | holds: the GPU's one order and every CPU order of the four terms, 8³ camera positions to ±100 km, 26 directions, 3 local positions to 128 m; the f32 World composition asserted past the bar | `src/render/shader_function_tests.rs:1769-1862`, `:2054-2140` |
| 0014 | Precision bar: "a point 0.3 m from the camera lands within 0.25 px at 1440p and a 45° vertical field of view (0.043 mm)" | holds: 2 · 0.3 · tan 22.5° / 1440 · 0.25 = 4.31e-5 m; the oracle's constant is 4.3e-5 | `src/render/shader_function_tests.rs:1767-1769` |
| 0014 | "Worst cases ... sum to 0.041 mm, 0.0185 and 0.0225: the composition in every evaluation order ... plus f32 rotation of a vertex 128 m from its pivot" | holds: the composition's any-order floor by arithmetic is 2.6e-5 m (round to nearest), above the 1.85e-5 measured; the culling pass takes 0.041 mm twice as its composition allowance | `culling.wgsl:159-165` (`COMPOSITION_ERROR = 1e-4`); Oracles below |
| 0014 | "The vertex shader composes camera-relative positions until draws are GPU-driven; then the culling pass does it once per Instance" | drift: L1-S05. The culling pass composes only its bounds' centre, for its own test, and writes no transform; the vertex shader composes per vertex | `culling.wgsl:119-138`; `main.wgsl:95-101` |
| 0014 | "f64 on the GPU: Metal has no f64 shaders, and its default fast math may reorder the arithmetic" | holds: no part uses f64 (WGSL has none), and the oracle's every-order check covers reordering | `src/render/shader_function_tests.rs:2091-2112`, `:2121-2140` |
| 0014 | "World-anchored shader effects (noise, wind, triplanar mapping) derive their periodic coordinates from the anchor" | not applicable: no such effect in any part | |
| 0015 | "Depth is cleared to 0 and compared with `Greater`, in `Depth32Float`" | falls in render | |
| 0015 | "with no far plane" | holds where the shaders rely on it: the frustum has five planes, "every point past the near plane lands at z ≥ 0"; the last Froxel slice reaches infinity; `position_at` and the light count read clip z as the near distance over clip w. The matrix itself falls in the camera module (`src/camera/mod.rs:95-97`, glam's infinite reverse) | `culling.wgsl:140-146`, `:105-109`; `froxel_grid.wgsl:6-7`, `:19-22`; `common.wgsl:42-52`; `light_count.wgsl:36-41` |
| 0015 | "shadow comparisons follow the same direction" | falls in shaders D | |
| 0015, checklist | "a Hi-Z pyramid keeps the minimum depth, which is the farthest"; "Hi-Z passes follow the same direction" | holds: every reduction is `min`, the edge value 1.0 is the nearest and so the identity, and the occlusion test asks whether the box's nearest depth is below the pyramid's minimum | `depth_pyramid.wgsl:1-5`, `:29-33`, `:46-52`, `:56-60`, `:110`, `:115-119`, `:146`; `culling.wgsl:166-170`, `:219-223` |
| 0019 | "a compute pass appends the Meshlets of each Instance whose Mesh bounds reach into the view frustum to the compacted index buffer in slot order, in two phases" | holds: each slot's offsets are an exclusive scan within its block, then a scan over blocks, both deterministic; records and indices land at those offsets | `culling.wgsl:79-96`, `:239-284`, `:294-338`, `:342-344`, `:362-388`, `:401-426` |
| 0019, checklist | "The GPU builds the frame's draw in slot order within each culling phase, never in the order atomics land" | holds: the pass's atomics are `atomicMax` on the index count, `atomicAdd` on the two Instance counts and `atomicOr` on the visibility words, each commutative; no offset comes from one | `culling.wgsl:60-64`, `:268-277`, `:370-379` |
| 0019 | "Phase one appends those visible the frame before, one bit per slot" | holds, and phase one also asks the frustum, so an Instance that left the view is not drawn: fewer draws, the same image | `culling.wgsl:226-230`, `:252`, `:259` |
| 0019 | "a depth pre-pass draws them into 4x MSAA reverse-Z `Depth32Float` cleared to 0 and stores it" | falls in render and shaders A | |
| 0019 | "A compute pass reduces that depth into the Hi-Z pyramid, each texel the minimum of every sample below it" | holds: level 0 takes the four samples of each pixel its texel overlaps (two per axis, three where the depth target's side is odd), each later level the two or three texels below whose share of the screen overlaps its own; the overlap rule ⌊t·below/size⌋ to ⌈(t+1)·below/size⌉ − 1 is the one the GPU test's CPU reduction uses | `depth_pyramid.wgsl:19-24`, `:29-34`, `:70-111`, `:127-147`; `src/render/depth_pyramid_pass.rs:284-297` |
| 0019 | "Phase two appends the others whose bounds lie anywhere nearer than the pyramid's minimum over the texels their screen rectangle covers, and rewrites the bits" | holds, conservatively: the eight corners' depth, each corner moved toward the camera by 2⁻¹⁴ of the clip terms summed plus 0.1 mm; the rectangle grown by a pixel and read at the level where it spans at most two texels; `nearest_depth < farthest` occludes; a corner at or behind the camera, or a rectangle wider than two texels after rounding, does not; the bits are gathered and written per block | `culling.wgsl:159-224`, `:253-257`, `:268-277` |
| 0019 | "one `draw_indexed_indirect` per phase for the depth pipeline and one of both phases for the lit pipeline" | holds, shader side: `FrameDraw.draw` grows from index 0 over both phases and `phase_two_draw` starts at phase one's sum; the draw is contiguous, since an Instance that does not fit ends it | `culling.wgsl:37-52`, `:318-337`, `:346-354`, `:370-377` |
| 0019 | "lit by ... the point lights of each pixel's Froxel, which a compute pass ... lists from a light buffer of fixed capacity: those whose range reaches into the Froxel's bounds"; "Lights are culled into Froxels, each with its own light list built against the Froxel's bounds" | holds: a workgroup per screen rectangle, an invocation per slice, each testing every point light's sphere against the rectangle's four planes through the camera and its slice's depth slab, inclusive at every boundary, in light order, counting the lights past the list's capacity | `froxel.wgsl:7-64`; `froxel_grid.wgsl:14-22`; capacity 127 from `src/render/froxel_pass.rs:26` through `mod.rs:255` |
| 0019 | "each index packs a Meshlet record in its high bits and a vertex within that Meshlet in its low six: the cooker builds Meshlets of at most 64 vertices" | holds: `MESHLET_VERTEX_BITS` is log₂ 64 = 6, declared from the cooked limit, which `mod.rs` asserts a power of two; the record capacity is asserted below 2²⁶ in Rust; the triangle's corner is one byte, lowest byte first, as the pool packs it | `common.wgsl:215-230`; `mod.rs:36`, `:239`, `:264`; `culling.wgsl:421-425`; `src/assets/pools.rs:253-257`; `src/render/culling_pass.rs:166-168` |
| 0019 | "the vertex shader reads the record for its Instance, Material and vertex base, then pulls its vertex from the vertex Arena" | the read falls in shaders A (`main.wgsl:90-94`); the decode holds: position and uv from scalar arrays, the QTangent from two Snorm16 pairs, under the cooked-vertex oracle | `common.wgsl:232-289`; `src/render/shader_function_tests.rs:1479-1546` |
| 0019 | "Each Primitive draws with its own Material unless its Instance overrides it" | holds: the record takes the Meshlet's Material ID unless the Instance's is not `NO_MATERIAL` | `culling.wgsl:381-386`; `mod.rs:242` |
| 0019 | "Culling tests Instances; tests per Meshlet wait for a profile" | holds: the one test is of the Mesh's bounds per slot | `culling.wgsl:248-257` |
| 0019 | "One compute culling pass composes each Instance's camera-relative transform (ADR 0014), tests the frustum and a two-phase Hi-Z pyramid" | tests hold; "composes each Instance's camera-relative transform" is the same drift as ADR 0014's sentence: L1-S05 | `culling.wgsl:119-138` |
| 0019 | "Froxels, each with its own list built once against its bounds, serve the opaque and transparent passes alike" | not applicable: no transparent pass at this commit | |
| 0019 | "the depth passes also write each sample's triangle normal and the normal interpolated over it, 4 bytes per sample" | the encode holds: two octahedral coordinates in [−1, 1] per normal, two normals, so four 8-bit channels; the write falls in shaders A | `common.wgsl:291-297`; `src/render/shader_function_tests.rs:1682-1766` |
| 0019 | "the shadow mask pass rebuilds each pixel's position and face normal from [the depth target]" | the two functions hold under oracle: `position_at` to 2⁻²⁰ of the largest coordinate against an f64 inverse, `octahedral_decode` to 4e-6 and, from 8 bits, 0.95°; the pass falls in shaders D | `common.wgsl:49-56`, `:300-305`; `src/render/shader_function_tests.rs:1618-1766` |
| 0019 | "`max_storage_buffers_per_shader_stage` at least 9, since the culling pass's compute stage reads and writes nine" | holds: the culling module's compute-visible storage bindings are `instances`, `meshes`, `meshlets`, `meshlet_triangles` and `instance_offsets`, `meshlet_records`, `packed_indices`, `frame_draw`, `instance_visibility`, nine; the limit itself falls in render | `mod.rs:105-110`; `src/render/bind_groups.rs:70-97`, `:122-134`; `culling.wgsl` reads each |
| 0019 | "A fragment stage that reads any flat input is never merged"; "No count draws"; "Phase two's Instances draw into depth ..." | fall in shaders A and render | |
| 0020 | "The `crowd` Reference scene's figures, which the GPU bends each frame, have no BLAS and cast their shadows through rigid per-bone shadow proxies" | the bend holds: each vertex of the rest Mesh is turned about the horizontal axis across the azimuth by curvature times height, placed on the arc's chord, its QTangent turned and re-stored as the cooker stores one; the BLAS and proxies fall in render and cook | `bend.wgsl:43-81`; `src/cook/mesh.rs:303-327` |
| 0020 | "the dynamic TLAS's build waits for the refits on the GPU ...; the refits are timed on the GPU from the end of the bend pass to a marker after them" | holds in form: the marker's one invocation reads the refits' scratch and keeps it; whether Metal orders it after the refits needs hardware (render's note) | `blas_refit_marker.wgsl:1-7` |
| 0020 | "its build's recording and submit, and the build itself on the GPU, are timed in each frame that builds it" | holds in form: the marker queries the dynamic TLAS with a ray of length 1e-6 and keeps the committed kind, so the GPU orders it around the build; the rebuild form's missing markers are render's L1-R05 | `dynamic_tlas_marker.wgsl:1-17` |
| 0020 | "A constant ambient term stands in for indirect light"; the shadow ray clauses; "Shadow and probe rays query both" | fall in shaders A and D | |
| 0022 | "Everything that changes game state runs in the fixed schedule ... no system reads the wall clock" | holds, nothing shader-side: the bend reads a curvature the CPU blends between Ticks; no part reads time | `bend.wgsl:11-21`; `src/render/bend_pass.rs:28-39` |
| 0023 | photometric units, EV100, 65504, GT7 | not applicable in this letter: fall in shaders A and B; the Froxel pass reads a light's `range`, never its intensity | `froxel.wgsl:51-53` |
| checklist | "WGSL ... has no divide-by-zero or NaN at grazing angles or zero roughness" | holds in these parts: `position_at` divides by a depth its callers guard above 0 and by the squared lengths of projection rows; `octahedral_encode` by \|x\| + \|y\| + \|z\| of a unit vector; `occluded` by clip w after `w <= w_rounding` returns; `sinc` by an angle the series replaces below 0.5; `camera_relative_point` by a projection's determinant | `common.wgsl:49-56`, `:293-294`; `culling.wgsl:191-194`; `bend.wgsl:28-32`; `light_count.wgsl:19-27`, `:39` |
| checklist | "calls no `textureSample` under non-uniform control flow" | holds: these parts call `textureLoad` only | `culling.wgsl:200`, `:220-221`; `depth_pyramid.wgsl:31-32`, `:117-118`; `light_count.wgsl:38` |
| checklist | "A binding is written once, in its bind group's definition, which declares it in WGSL ...; no WGSL part declares one" | holds, tested: a `@group` in a part fails `stage_problems` | `mod.rs:281-283`; `src/render/shader_tests.rs:34-38`, `:68-121`, `:495-536` |
| checklist | "`cargo test` checks each binding's stages against the entry points that use it, and the structs the CPU writes field by field against their Rust twins; Rust-only padding fields start with `_`" | holds: visibility equals use per naga's analysis; nine twins compared by offset, size and scalar; `FrameDraw`, `LightCounts` and `MeshletRecord` are GPU-written and skipped by the `Writer::Gpu` rule, their layouts matching by inspection (72, 16 and 12 bytes) | `src/render/shader_tests.rs:231-312`, `:380-418`, `:655-716`; `culling.wgsl:21-52`; `common.wgsl:112-117`, `:209-213`; `src/render/culling_pass.rs:82-91` |
| checklist | "Shaders are WGSL parts ... assembled into modules by its `mod.rs` after the bindings and constants it declares from Rust; a function under an oracle lives in a shared part" | holds: enables, then 26 constants and the sample offsets, an `override` per Render switch, the groups' declarations, then the parts; every `.wgsl` under `src/` is a part of some module, tested; every oracle function is in `common.wgsl`, `shading.wgsl`, `hdr.wgsl`, `gt7.wgsl` or `display.wgsl`, the `SHARED` set | `mod.rs:57-74`, `:169-176`, `:231-291`; `src/render/shader_tests.rs:123-136` |
| checklist | "Matrices reach WGSL column-major; a `mat3x3` ... padded per column on the Rust side" | holds for the one `mat3x3` pair these parts read: `InstanceData` is 128 bytes both sides, its columns padded | `common.wgsl:175-182`; `src/render/instance.rs:19-23`, `:38-45`; `src/render/shader_tests.rs:268-275` |
| 0020 | "Direct light is one hard shadow ray per pixel per light" | holds: one `shadowed` call per light entry of the surface; `shadowed` continues the same ray from where it leaves its own mesh, two turns at most, and `first_intersection` is the one call that queries | `shadow_mask.wgsl:270-346`, `:343`; `:163-179`; `:96-122` |
| 0020 | "skipped where the light adds nothing ([...] a light the surface's face turns from is hidden without one)" | holds on its purpose: a directional light the interpolated normal turns from by more than sin 2° is hidden with no ray; a point light beyond its shadow range is passed over with no ray, no bit and no count; the word "face" is stale: L1-S06 | `:293-300`; `:299-303`; `:225-228` |
| 0020 | "from an origin offset along the surface's normal" | holds: along the triangle normal by the larger of 2^-16 of the largest coordinate (1 m at least) and 1/32 of a pixel's width, toward the light's side where the triangle faces it, inside the surface otherwise | `:153-162`; `:33`, `:48` |
| 0020 | "and composed in the TLAS's frame" | holds: `camera_relative_position(camera.anchor, camera.offset, camera.tlas_anchor, 0, position)` is the camera's World position less the TLAS anchor's corner plus the camera-relative point, the anchor difference in integers | `:144-152`; `common.wgsl:31-40` |
| 0020 | "toward each directional light against the static TLAS [...] ending at [...] infinity" | holds: `UNBOUNDED_RAY_DISTANCE` is f32's largest finite value, and `segment_meets_box` cuts it before any product | `:23`, `:282`; `common.wgsl:80` |
| 0020 | "one toward each point light that reaches the pixel, ending at the light" | holds: `distance = sqrt(distance_squared)`, the ray's `end_distance` | `:310`, `:343`, `:170` |
| 0020, 0019 | "A compute pass traces them from the depth target, at each pixel's nearest sample" | holds: the greatest of the four depths, reverse-Z, the first such sample | `:397-405`, `:356-362` |
| 0020, 0019 | "where its other samples show another surface, at the farthest of those too [...] into the mask's second layer" | holds: samples off the nearest sample's triangle plane by more than a pixel's width, other than the background's; the least depth among them is listed with its sample and traced into layer 1 | `:413-449`; `:480-499`, `:498` |
| 0019 | "the second surfaces [...] are traced from a packed list, as tile-based deferred renderers schedule per-sample shading" | holds: `atomicAdd` on the list's count, the opening entry of each 64-run grows the indirect dispatch in rows of `MAX_WORKGROUPS_PER_DIMENSION`, the second entry point reads its entry and returns past the count | `:434-447`; `:486-492` |
| 0019 | "a pixel of three surfaces or more gives its middle ones the farthest's" | holds: `second_samples` holds every off-plane sample, one word is traced for them all, from the farthest | `:424-426`, `:429-433`; `:240-243` |
| 0019 | "each fragment reads the bits traced from the surface it shades [...] a nearest-surface fragment that runs under a nearer surface at another sample reads that surface's" | falls in shaders A (`main.wgsl:232-238`); the layout it reads, second-surface sample bits 0-3 and sample verdicts above them, is what `build_shadow_mask` and `trace_shadow_edges` store | `:464-468`; `:676-680`; `main.wgsl:233-238` |
| 0020 | "at a pixel whose verdict for one of the first seven directional lights differs from a 4-neighbour's among the lights both traced" | holds: `differing_lights` masks the hidden-bit difference by both pixels' traced bits and `EDGE_LIGHTS` (7); left, right, up, down with the target's side standing in past it, so a border pixel differs from itself nowhere | `:525-530`, `:536-548`; `:520` |
| 0020 | "or whose nearest sample turns from that light while another sample of its nearest surface faces it" | holds: `turned_lights` is hidden-and-not-traced; `turned_lights_other_samples_face` tests each other nearest-surface sample's stored interpolated normal with the same margin | `:552-554`, `:562-590`; `:607` |
| 0020 | "toward that light again from each other sample that shows its nearest surface, the fragment shading it by the share of its samples that see it" | holds for the tracing: every sample other than the nearest, the second surface's and the background's gets `directional_hidden` per edge light, bit `4 × light + sample`; the share is shaders A's (`directional_hidden_share`) and reads the same bit layout | `:658-675`, `:672`; `main.wgsl:134-144` |
| 0020 | "where the nearest sample turns from a point light while another sample of its nearest surface faces it, the surface's one bit for that light is the verdict of a ray from the first such sample" | holds: the first other sample whose interpolated normal faces the light becomes `traced_surface`; none facing, the bit is set with no ray; a second surface passes `other_samples = 0` | `:311-338`; `:334-337`; `:497` |
| 0020, 0019 | "holds 59 lights per pixel: a light past them casts no shadow there, and Frame stats count it" | holds: bits 0-31 in `x`, 32-58 in `y`, the slice in `y`'s bits 27-31 (`59 - 32`); a light at entry 59 or past it increments `overflowed`, summed once per surface by one atomic | `:194-198`, `:351-352`; `:275-277`, `:304-307`, `:347-349`; `src/render/shadow_mask_pass.rs:32`, `:67` |
| 0020 | "a shadow ray the static TLAS lets through is traced against [the dynamic TLAS]" | holds: the dynamic TLAS is queried unless it is empty, a first hit was wanted and found, or the segment misses the dynamic box grown by 2^-16 of the largest coordinate; a nearest-hit ray queries it to the static hit's distance, so only a nearer dynamic triangle replaces it | `:103-121`; `:107`, `:111-112` |
| 0020 | "rebuilt each frame in the static TLAS's frame" | holds, shader side: both queries take the same origin and direction, and the box is read in the same frame (`CameraUniform`'s comment) | `:99`, `:115`; `common.wgsl:9-12` |
| 0020 | "Direct light is Temporal class (a)" | holds: the pass reads this frame's depth, normals, lights and Froxel lists, no history | `:397-402`, `:213`, `:264`, `:307` |
| 0020 | "A constant ambient term stands in for indirect light" | falls in shaders A | |
| 0020 | "Soft sun shadows [...] a denoiser is class (c)"; penumbrae reopen later | not applicable: one hard ray, no jitter | `:343` |
| 0019 | "a compute pass traces, from each pixel's nearest sample, one shadow ray toward each light that reaches it and keeps whether it is hidden in the shadow mask" | holds | `:388-471`, `:457` |
| 0019 | "the depth passes also write each sample's triangle normal and the normal interpolated over it [...] which the shadow mask pass reads so that a coarse smooth-shaded mesh's terminator follows its interpolated normals rather than its triangles" | holds: the turn tests read `interpolated_normal`; `triangle_normal` decides only which side the origin leaves from and the plane of the second-surface test; the store is `octahedral_encode(...) * 0.5 + 0.5` and the load `* 2.0 - 1.0`, xy the triangle's, zw the interpolated | `:213-218`, `:297`, `:311`, `:158-162`, `:417`; `main.wgsl:67-68` |
| 0019 | "the point lights of each pixel's Froxel, which a compute pass [...] lists" | holds: `froxel_at` at the pixel's centre and the sample's view depth `near / depth`; the list's `light_count` bounds the loop and `lights[entry - directional_light_count]` names the light; with `FROXEL_LIGHTS` off every point light in buffer order, as the lit pass sums them | `:258-265`, `:291-294`; `froxel_grid.wgsl:34-41`; `main.wgsl:270-286` |
| 0019 | the mask's slice bits: "the Froxel at this pixel's rectangle and the slice that the sample its word was traced from shows" | holds: `slice = froxel % FROXEL_SLICES`, stored at bit 27 of `y`, read by `mask.y >> (SHADOW_MASK_LIGHTS - 32u)` | `:263`, `:365-366`; `main.wgsl:276` |
| 0019 | "A fragment stage that reads any flat input is never merged" | falls in shaders A (`main.wgsl:69-72`) | |
| 0019, checklist | the GPU builds in slot order, never in the order atomics land | holds for the mask: the two lists are filled in atomic order, but each listed entry writes only its own pixel's word or samples, and the overflow count is a commutative sum, so no result depends on the order | `:434-435`, `:498`; `:610-611`, `:676-680`; `:348` |
| 0014 | "The GPU never holds a World position in f32: it reads an integer anchor [...] plus an f32 offset, and shaders compose positions relative to the camera" | holds: the TLAS-frame origin is composed from `camera.anchor - camera.tlas_anchor` in `vec3<i32>` times 64 plus `camera.offset` plus the camera-relative point; point lights go through `point_light_position` | `:146-152`, `:296`; `common.wgsl:13-22`, `:38-39`, `:121-133` |
| 0014 | "Subsystems that compute in f32 ([...] ray-tracing acceleration structures) convert at their boundary into a local frame they own" | holds: one composition per ray into the TLAS's frame, and nothing else in the pass leaves the camera-relative frame | `:146-152`, `:162` |
| 0014 | precision bar: "a point 0.3 m from the camera lands within [...] 0.043 mm" | holds by the same arithmetic as the oracle in shaders C: the composition adds two f32 terms at the TLAS frame's magnitude (2 km in `town`, step 0.24 mm, per the M3 spec), and the ray origin's offset floor, 2^-16 m, is 15 µm, far above the composition's 2 × 2^-24 of 2 km (0.24 µm) | `:146-152`; `common.wgsl:38-39` |
| 0015 | "Depth is cleared to 0" | holds: 0 is read as the background everywhere a depth is used | `:249`, `:262`, `:369`, `:383`, `:414`, `:424`, `:678` |
| 0015 | "compared with `Greater`" | holds: the nearest sample is the greatest depth, the farthest second surface the least, and 2 stands past the near plane's 1 | `:403-405`, `:430-432`, `:573`, `:655` |
| 0015 | "with no far plane" | holds: the view depth is `view_proj[3].z / depth`, the near distance over the reverse-Z depth, finite for every drawn sample since depth > 0 is tested first | `:261`; `common.wgsl:49-56` |
| 0015 | "shadow comparisons follow the same direction" | holds: the mask makes no depth comparison other than those above; the rays compare distances along the ray, not depths | `:173`, `:187-188` |
| 0023 | light units, EV100, 65504 clamp | not applicable: the pass reads no intensity, only directions, positions and ranges | `:281`, `:296`, `:300` |
| checklist | "calls no `textureSample` under non-uniform control flow" | holds: the module calls `textureLoad` only | `:213`, `:325`, `:398-401`, `:496`, `:510`, `:568-571`, `:580`, `:650-653` |
| checklist | "no divide-by-zero or NaN at grazing angles" | holds: the point light's facing test precedes the division by its distance (a light at no distance is hidden or handed to another sample); `position_at` divides by a depth tested > 0; directional directions normalise a unit vector; `segment_meets_box` divides by nothing | `:308-311`, `:338`, `:340`; `:249`, `:263`; `common.wgsl:64` |
| checklist | ray flags: "Terminate rays on the first hit when possible", "Monitor the register consumption of the query objects" | holds: `RAY_FLAG_TERMINATE_ON_FIRST_HIT` where the ray leaves its surface, nearest-hit only for the one that starts inside and must read `front_face`; `RAY_FLAG_FORCE_OPAQUE` always; one `ray_query` variable, re-initialised for the second TLAS, in the one function every ray calls | `:166`, `:177`; `:98-99`, `:115`; `:90-95` |
| checklist, Rust | `SAMPLE_OFFSETS` are the standard pattern of four, top-left origin | holds: (0.375, 0.125), (0.875, 0.375), (0.125, 0.625), (0.625, 0.875), the two registry quotes verbatim; `sample_offset`'s select tree returns the indexed one; `position_at` takes pixels from the top left, y down | `src/render/shadow_mask_pass.rs:41-46`; `:186-192`, `:211`; `common.wgsl:50` |

### Spec stories

| Story | Verdict | Where |
|---|---|---|
| 6, the vertex shader reads the 28-byte vertex from the Arena | holds for the use: `decode_vertex(vertices[meshlet_vertices[first_vertex + index.vertex]])`; the decode and its oracle fall in shaders C | `main.wgsl:90-94` |
| 7, each Primitive with its own Material | holds for the read; the resolution falls in shaders C | `main.wgsl:113`, `:172` |
| 13, the pre-pass seeds the lit pass's depth-equal test | holds: one `@invariant` position for every entry point | `main.wgsl:8`, `:17`, `:27`, `:85` |
| 14, 4x MSAA without temporal accumulation | holds, shader side: shading once per pixel with per-pixel derivatives, no history read | `main.wgsl:193-204` |
| 15, 39, the 4x colour resolved in-pass, bright edges kept from aliasing | holds for the store, weighted for the resolve; the weighting falls in shaders B | `main.wgsl:303-306` |
| 17, directional lights' direction from the World transform | holds for the read: `-offset`, which prepare fills with the forward axis (render) | `main.wgsl:214-218`, `:251` |
| 19, point lights culled into Froxels | holds for the read (clause rows) | `main.wgsl:272-283` |
| 20, Froxel lighting matches brute force | holds by construction: both paths sum point lights in light order, the Froxel's list being in light order (shaders C), so the f32 sums are the same where the lists hold the same lights; the capture is tooling's | `main.wgsl:279-283`; `froxel.wgsl:3-4` |
| 21, Appendix B's BRDF exactly (D, V, F and the mix), each with an oracle | holds: D, V and F as the registry quotes them; the mix gathered by lobe is Appendix B's `mix(dielectric_brdf, metal_brdf, metallic)` term by term (the L1-S01 block's algebra) and its oracle compares with the "full code" form | `shading.wgsl:92-194`; `shader_function_tests.rs:20-272`, `:1229-1284` |
| 22, inverse square and the smooth range window, exactly zero at the range | holds; tested exactly 0 at and past the range | `shading.wgsl:234-238`; `shader_function_tests.rs:533-535` |
| 23, tutorial constants gone | holds: every constant carries a source, except the roughness floor's value and the 1e-30 guard, which the source leaves open (Notes) | `shading.wgsl:11-28`, `:226`; `main.wgsl:118` |
| 24, the glossary's names (Layers, the normal Texture) | holds: `material.layers`, `normal_textures`, `Surface` | `main.wgsl:172-178`; `common.wgsl:135-144` |
| 25, ambient one constant irradiance per scene in the decided units | holds: lux in, E/π out | `shading.wgsl:266-268`; `main.wgsl:301` |
| 26, 27, one ray per light reaching the pixel, skipped where N·L ≤ 0 | the rays fall in shaders D; the lit pass adds nothing for such a light and consumes one bit per light | `shading.wgsl:220`; `main.wgsl:290` |
| 40, exposure and tonemapping oracle-tested | falls in shaders B and the camera module | |
| 41, highlights exceed 1.0 before tonemapping | holds: the lit pass clamps at 65504 and nowhere lower | `main.wgsl:305` |
| 37, the HDR survey resolved before any HDR code | holds by the record: `docs/research/hdr-exposure-and-tonemapping.md` exists and the parts follow its decided rows (GT7, reversible weighting, static triangular dither, `Rgba16Float` with the 65504 clamp); not a code mechanism | `docs/research/hdr-exposure-and-tonemapping.md`, rows 8, 15, 18, 19 |
| 38, the light-unit choice recorded as an ADR | holds: ADR 0023; the units themselves are shaders A's and render's | `docs/adr/0023-photometric-light-units.md` |
| 39, the MSAA resolve of HDR keeps bright edges from aliasing while the 4x colour stays memoryless | holds, the shader half: every lit fragment writes `resolve_weighted`, the hardware box-resolves, the tonemapping pass undoes it; one sample of 100 over three of 0.05 resolves to 0.395 where the box gives 25.04 (arithmetic, matching the test's statement) | `hdr.wgsl:23-31`; `main.wgsl:306`; `tonemapping.wgsl:19`; test `:690-751` |
| 40, exposure and tonemapping functions oracle-tested, so that the curve and its units are exact | holds for tonemapping: the curve against the sample's 18 values, PQ against ST 2084 in f64, ICtCp against BT.2100's grey, GT7 against 26 of the sample's outputs, `tonemap` against 14 with the units (exposed 1 = 2.5 = 250 cd/m²) in the oracle, sRGB and the dither against f64. The exposure scale 1/(1.2 · 2^EV100) is Rust-side, render's | `shader_function_tests.rs:758-1049`, `:1058-1083` |
| 41, a Golden whose highlights exceed 1.0 before tonemapping | falls in tooling and scenes; the shader gives such a highlight a finite weighted value (≤ 13.85 per channel) and the curve's shoulder, not a clip | `hdr.wgsl:23-25`; `gt7.wgsl:66` |
| 2, every Instance against the frustum and a two-phase Hi-Z pyramid of minimum reverse-Z depth, in compute | holds | `culling.wgsl:147-157`, `:177-224`, `:239-284` |
| 3, culling conservative, shown by `town` culled and unculled matching at eight frames | holds in mechanism: the frustum test is a separating-plane test on five planes, the occlusion test moves corners toward the camera and grows the rectangle; the equivalence runs in the Goldens test, zero pixels allowed on hardware, one on lavapipe (tooling) | `culling.wgsl:140-157`, `:159-224`; `tests/goldens.rs:58-71`, `:143-175`, `:299-306` |
| 5, each index packs a Meshlet record in its high bits and a vertex within that Meshlet in its low six | holds, under oracle at every bit boundary | `common.wgsl:222-230`; `src/render/shader_function_tests.rs:1548-1611` |
| 6, the vertex shader reads the 28-byte vertex from the Arena, with an oracle for the decode | holds for the decode: 28 bytes of scalar arrays, the QTangent's Snorm16 in two u32s; the oracle lays vertices out as the Arena does and reads them by index; the read itself is shaders A's | `common.wgsl:232-239`, `:277-289`; `src/render/shader_function_tests.rs:1470-1546` |
| 9, Instances and triangles drawn read back from the GPU | holds: `instances_drawn` added per fitting Instance; triangles are the draw's index count over three | `culling.wgsl:370-373`; `src/render/culling_pass.rs:100-104` |
| 12, a frame past the compacted index buffer draws what fits, counts the overflow | holds: `fits` ends the draw at the last Instance that fits, the rest counted as overflowed; records short of the buffer's end are still written so `append_triangles` reads none unwritten | `culling.wgsl:346-354`, `:362-388`, `:415-417` |
| 13, the 4x MSAA reverse-Z pre-pass seeds the Hi-Z pyramid | holds: level 0 loads the four samples of each pixel, asserted four in Rust | `depth_pyramid.wgsl:26-34`; `src/render/depth_pyramid_pass.rs:16-18` |
| 19, point lights culled into Froxels, each list built against its bounds | holds | `froxel.wgsl:7-64` |
| 20, Froxel-culled lighting matches brute force within Golden tolerance | holds in mechanism: every boundary test is inclusive and a light's window is zero at its range, so a light that contributes to a point is listed in the point's Froxel; the equivalence allows zero pixels and runs in the Goldens test and the renderer's test | `froxel.wgsl:48-53`; `tests/goldens.rs:153-183`; `src/render/renderer.rs:1856-1933` |
| 22, inverse square and the range window | falls in shaders A | |
| 26, 27, 34, shadow rays | fall in shaders D | |
| 44, every M3 GPU buffer at a fixed capacity, overflow counted and warned | holds, shader side: Froxel lists past 127 lights counted; draw records and indices past their buffers counted | `froxel.wgsl:54-63`; `culling.wgsl:378-379` |
| 50, `crowd`'s figures deformed on the GPU each frame | holds: the bend pass | `bend.wgsl:43-81` |
| 57, the crowd's acceleration-structure update timed as BLAS refit and as proxies | holds in form for the markers that bound the times; the measurement falls in tooling | `blas_refit_marker.wgsl:4-7`; `dynamic_tlas_marker.wgsl:6-17` |
| 26, every light that reaches a pixel traces one hard ray, without shadow maps | holds: one `shadowed` per light entry, directional lights then the Froxel's point lights | `:270-346` |
| 27, a ray skipped only where its light adds nothing (N·L ≤ 0, or beyond its range), none dropped by a cap | holds on its purpose, conservatively: a directional or point light is hidden with no ray when the interpolated normal turns from it by more than sin 2°, so a light at N·L in (−sin 2°, 0] still gets a ray the lit pass then darkens, by the stored normals' error the comment states; a point light at or past its range gets none; the cap drops no ray, it drops a bit and counts (see Notes) | `:61-74`, `:283`, `:311`; `:300-303`; `:275-277`, `:304-307` |
| 22, the mask's side of the range window: zero at the range | holds: `distance_squared >= shadow_range * shadow_range` skips the light exactly where the lit pass's window reaches zero | `:300-303` |
| 29, shadows keep ADR 0014's precision, the static TLAS's frame anchored at load | holds, shader side: composition from integer anchors into the TLAS's frame | `:146-152` |
| 30, moving geometry casts shadows through the dynamic TLAS | holds: queried after the static TLAS unless empty, found or missed by the box | `:103-121` |
| 19, 20, Froxel lists read per pixel; Froxel lighting matches brute force | holds for the mask's read: the same list, the same order, the same bit positions as the lit pass with `FROXEL_LIGHTS` on or off; the equivalence check is tooling's | `:258-265`, `:291-294`; `main.wgsl:270-290` |
| 34, the analytic sphere-over-plane oracle | falls in tooling (the capture seam) | |
| 58, the `SHADOW_RAYS` and `FROXEL_LIGHTS` switches | holds: `FROXEL_LIGHTS` is an override constant the mask branches on; `SHADOW_RAYS` gates the lit pass's reads (shaders A) | `:260`, `:274`, `:292`; `main.wgsl:233` |

### Registry keys

| File | Keys | Verdict |
|---|---|---|
| `shading.wgsl:7` | `gltf-2.0.1-sample-brdf` | matches: the comment's "separable Smith G and a simplified mix of the two lobes" is the quote's "separable form of the Smith joint masking-shadowing function [...] we can simplify the mix" |
| `shading.wgsl:15`, `:165` | `gltf-2.0-metal-and-dielectric-brdf` | matches: f₀ = 0.04 at ior 1.5 (((1 − 1.5)/(1 + 1.5))² = 0.04); the mix gathered by lobe equals the quoted "full code" term by term, shown under L1-S01's block |
| `shading.wgsl:21` | `gltf-2.0-zero-roughness` | matches: "α should be clamped to a small positive value"; the floor is applied to roughness, so α ≥ 2.56e-6; the value 0.04 is the engine's choice, which the source leaves open |
| `shading.wgsl:25`, `:36` | `tokuyoshi-2019-normal-derivatives` | matches: σ = 0.5 carried as σ² = 0.25; eq. (4) α² + min(2σ²(‖∆nᵤ‖² + ‖∆nᵥ‖²), κ) term by term; Listing 2's `saturate` as `min(…, 1.0)` |
| `shading.wgsl:26` | `tokuyoshi-2019-roughness-matrix` | matches: κ = 0.18 |
| `shading.wgsl:37` | `gltf-2.0-roughness-mapping` | matches: α = roughness² at `:47`, `:174` |
| `shading.wgsl:64` | `gltf-2.0-metallic-roughness` | matches through the cook: "The factor value is a linear multiplier"; glTF's green (roughness) and blue (metalness) are packed as BC5's red and green by `src/cook/texture.rs:263-268`, which the shader reads as `.r` and `.g` |
| `shading.wgsl:93` | `gltf-2.0-fresnel` | matches: F = f₀ + (1 − f₀)(1 − \|V·H\|)⁵, the weight multiplied out |
| `shading.wgsl:103` | `gltf-2.0-specular-brdf` | matches: D = α² χ⁺(N·H)/(π ((N·H)²(α² − 1) + 1)²) with the inner sum as (N·H)²α² + (1 − (N·H)²); V = χ⁺(H·L) χ⁺(H·V)/(2(\|N·V\|√(α² + (1 − α²)(N·L)²) + \|N·L\|√(α² + (1 − α²)(N·V)²))), one χ⁺ since H·L = H·V for the bisector |
| `shading.wgsl:108` | `filament-ggx-lagrange-identity` | matches: `cross(normal, half_vector)` then `dot` of it with itself, as the quote's `NxH` and `oneMinusNoHSquared` |
| `shading.wgsl:152` | `gltf-2.0-normal-texture-scale` | matches: (sample · 2 − 1) ⊙ ⟨scale, scale, 1⟩, normalised last, after an orthonormal frame, so the order is immaterial; z rebuilt as the positive root since BC5 stores x and y |
| `shading.wgsl:225` | `filament-punctual-light-1cm` | matches: `max(distanceSquare, 1e-4)` |
| `shading.wgsl:230` | `khr-lights-punctual-range` | matches: max(min(1 − (d/r)⁴, 1), 0)/d² with (d/r)⁴ as (d²/r²)²; the 1 cm floor beneath d² is the deviation L1-S02 names |
| `shading.wgsl:265` | `gltf-2.0-occlusion` | matches: "Direct lighting is not affected"; occlusion multiplies the ambient term only |
| `main.wgsl:182` | `gltf-2.0-bitangent` | matches: `cross(geometric_normal, tangent) * in.tangent_world.w` |
| `main.wgsl:223` | `salvi-2012-sbaa-surfaces` | matches: "one color sample per surface"; each fragment reads the word of the surface it shades |
| `shader_function_tests.rs:56` | `walter-2007-projected-area` | matches: 2π ∫ D cos θ sin θ dθ = 1 is eq. 4 at v = n |
| `shader_function_tests.rs:116-117` | `walter-2007-ggx`, `gltf-2.0-roughness-mapping` | match: eq. 33 at θ = 0 is 1/(π α²); 16/π at roughness 0.5 |
| `shader_function_tests.rs:288` | `gltf-2.0-coupling` | matches: the dielectric's furnace bound is not tightened past 1 + the black dielectric's, as the quote says `fresnel_mix` "breaks [...] energy conservation" |
| `shader_function_tests.rs:407-415` | `vulkan-spirv-precision`, `d3d11.3-fp32-rules`, `wgpu-hal-30-metal-compile-options`, `msl-4.1-fast-math`, `msl-4.1-rounding`, `higham-3.1` | match: correctly rounded add and multiply on Vulkan; 0.5 ULP on D3D; no math-mode setter in wgpu-hal's Metal compile options; fast math's reassociation and contraction with correctly rounded add, multiply and fma; round toward zero allowed, hence u = 2⁻²³; γ₃ = 3u/(1 − 3u) and the bound γₙ \|x\|ᵀ\|y\| |
| `shader_function_tests.rs:1180` | `hoffman-2013-f0` | matches: gold's linear F₀ (1.00, 0.71, 0.29) |
| `shader_function_tests.rs:1256` | `gltf-2.0-metal-and-dielectric-brdf` | matches: `reference_brdf` is the "full code" line by line |
| `shader_function_tests.rs:1292` | `gltf-2.0-normal-texture-scale` | matches |
| `shader_function_tests.rs:1346-1348` | `tokuyoshi-2019-normal-derivatives`, `tokuyoshi-2019-roughness-matrix` | match |
| `hdr.wgsl:4`, used at `main.wgsl:305` | `wgsl-f16-largest` | matches: 65504.0; shaders B's key, cited here for the clamp alone |
| `hdr.wgsl:4`, `:6` | `wgsl-f16-largest`, `vulkan-fp-conversion` | matches: `LARGEST_HALF = 65504.0` is the quoted "Largest positive finite: 65504.0h"; the clamp answers the quoted "may be rounded ... to the appropriately signed infinity" |
| `hdr.wgsl:21` | `karis-2013-weighted-average` | matches: weight 1/(1 + luma) with BT.709 linear luminance for luma; the inverse `weighted / (1 − luminance(weighted))` is Karis's "inverse tone mapped using T_inverse", exact for linear luma (Σ wᵢcᵢ / Σ wᵢ, derived in the Oracles block) |
| `hdr.wgsl:10` via `targets.rs:47-52`, `shaders/mod.rs:248` | `bt709-6-luma`, `bt2087-rec709-to-xyz` | matches: `[0.2126, 0.7152, 0.0722]`; derived again from BT.709-6's primaries and D65 the Y row is 0.21264, 0.71517, 0.07219 |
| `gt7.wgsl:2`, `:31` | `gt7-slides` | matches: paper white 250 nits = 2.5 in units of 100 nits (`GT7_PAPER_WHITE = 2.5`); "multiply by 1 / 2.5 after SDR tone mapping" is `:123` |
| `gt7.wgsl:5` | `gt7-tone-mapping-cpp` | matches: the quoted `initializeCurve(target, 0.25f, 0.538f, 0.444f, 1.280f)` are `alpha`, `mid_point`, `linear_section`, `toe_strength` at `:48-51`; `REFERENCE_LUMINANCE 100.0f <-> 1.0f` is the `100.0 / 10000.0` at `:71`; the 18-point curve table matches the curve as written to 2.3 × 10⁻⁷ (f32 printing) |
| `gt7.wgsl:35` | `bt2100-pq` | matches coefficient by coefficient: m1 = 0.1593017578125, m2 = 78.84375, c1 = 0.8359375, c2 = 18.8515625, c3 = 18.6875, each a dyadic rational exact in f32; `pq_encode` is ((c1 + c2 Yᵐ¹)/(1 + c3 Yᵐ¹))ᵐ², written as exp2 of a log2 difference; `pq_decode` is Table 4's EOTF with the max at 0 |
| `gt7.wgsl:83` | `bt2100-ictcp` | matches coefficient by coefficient: L, M, S rows 1688/2146/262, 683/2951/462, 99/309/3688 over 4096 (each row sums to 4096, so a grey keeps its value); I = (2048 L′ + 2048 M′)/4096 = 0.5 L′ + 0.5 M′; Cт and Cᴩ rows 6610/−13613/7003 and 17933/−17390/−543 over 4096. The inverse matrices at `:100-107` are the exact inverses to 3.4 × 10⁻⁶ relative, the "six digits" the comment claims |
| `display.wgsl:5-7` | `bt2087-rec709-to-rec2020`, `bt709-6-primaries`, `bt2020-2-primaries` | matches: the nine constants round to BT.2087's four-digit matrix exactly, and derived again from the quoted primaries and D65 (0.3127, 0.3290) they agree to 4.7 × 10⁻⁸, seven places as the comment says; `REC2020_TO_REC709` is that matrix's inverse to 3.9 × 10⁻⁸ |
| `display.wgsl:28` | `khronos-dfs-srgb` | matches: `12.92 R` for `R ≤ 0.0031308`, `1.055 R^(1/2.4) − 0.055` above, the `≤` on the linear side as quoted |
| `display.wgsl:43` | `peters-blue-noise` | matches in form, the quoted `mad(x, 2, −1)` then `sign(x)(1 − sqrt(1 − \|x\|))`; the code feeds it the level's bin centre (k + ½)/256 where the listing feeds the texel value k/255: on purpose, stated in the comment, no `deviations.md` row: L1-S03 |
| `display.wgsl:46` | `lipshitz-1992-triangular-dither` | matches: the noise spans (−1, 1) codes, "2-LSB peak-to-peak", of triangular density |
| ADR 0023 only, not cited by these parts | `frostbite-2014-exposure`, `frostbite-2014-fp16`, `unreal-scene-color-overflow` | matches the parts' behaviour: "Exposure is pre-applied" before the fp16 store (`main.wgsl:305`), and the clamp answers Unreal's overflow "black patches" |
| this section's floors | `higham-3.1`, `vulkan-spirv-precision`, `msl-4.1-fast-math`, `msl-4.1-rounding`, `d3d11.3-fp32-rules` | matches as used; the registry's `vulkan-spirv-precision` quote copies only the add, multiply and dot rows, so the transcendental bounds above are cited to the same table past its quote |
| `culling.wgsl:78` | `hillis-steele-partial-sums` | matches: eight steps for a block of 256, each adding the value `stride` places back behind two barriers, then read one place back for the exclusive sum, "all partial sums ... in the same amount of time it took to compute the single sum" |
| `bend.wgsl:25-26` | `msl-4.1-fast-math`, `msl-4.1-sin-fast-accuracy` | matches: the default is fast; "sin(x): For x in the domain [-pi, pi], the maximum absolute error is <= 2⁻¹³", which `sin(angle) / angle` would divide by a small angle, so the series stands in below 0.5 rad, where its first term left out is 0.5⁸ / 9! = 1.08e-8, as the comment says |
| `light_count.wgsl:9` | `webgpu-standard-sample-pattern` | matches: "4: Sample 0: (0.375, 0.125)" relative to the pixel's top-left corner; `FIRST_SAMPLE` is that, added to the pixel in framebuffer coordinates |
| `common.wgsl`, `depth_pyramid.wgsl`, `froxel_grid.wgsl`, `froxel.wgsl`, the two markers, `mod.rs`, `shader_tests.rs`, the oracles at `:1479-1864` | none cited | |
| relied on uncited: `common.wgsl:44-47`, `light_count.wgsl:36-41`, `culling.wgsl:140-146` | `glam-0.32.1-perspective-infinite-reverse-rh` (cited at `src/camera/mod.rs:94`) | matches: the quoted columns put the near distance in clip z and −z in clip w, so "depth is the near distance over clip w" and z ≥ 0 for every point; `view_proj[3].z` is the near distance |
| relied on uncited: `culling.wgsl:174-175` ("grows the rectangle by a pixel past where rasterization rounds vertices") | `vulkan-subpixel-precision-bits` (cited at `shadow_mask.wgsl:39`, shaders D) | matches: the minimum is 4 bits, a sixteenth of a pixel; the margin is 0.5 level-0 texel = at least one pixel (see Notes) |
| `common.wgsl:252-259` with `main.wgsl:181-189` | `gltf-2.0-bitangent` (cited in shaders A) | matches: the decode gives normal, tangent and w's sign; the lit pass's bitangent is cross(normal, tangent) times that sign, "the cross product of the XYZ components of the normal and tangent vectors ... multiplying that cross product against the W component" |
| `froxel.wgsl:48-53` | `khr-lights-punctual-range` (cited in shaders A and cook) | matches: the window max(min(1 − (d/r)⁴, 1), 0)/d² is zero at and past `range`, so a sphere of radius `range` is the light's reach and the Froxel test's slab and planes use it |
| floors below | `vulkan-spirv-precision`, `d3d11.3-fp32-rules`, `msl-4.1-fast-math`, `msl-4.1-rounding`, `wgpu-hal-30-metal-compile-options`, `higham-3.1` | match as used: add and multiply correctly rounded, 0.5 ULP on D3D, Metal may round toward zero, no math mode set, γₙ for an n-term chain in any order |
| `shadow_mask.wgsl:29` | `wachter-binder-2019-offset-error` | matches: the quote says the intersection's distance to the surface's plane "is proportional to the distance from the origin (0,0,0)" and that "the size of the surface [...] becomes dominant for triangles very close to the origin"; the code offsets by a share of the largest coordinate in either frame, and floors it at 1 m for points near the origin (`:153-157`). The share, 2^-16, is the chapter's `float_scale` (1/65536) but the quote does not carry that constant, so the constant rests on the comment's own 256-ulp argument |
| `shadow_mask.wgsl:38` | `d3d11.3-rasterizer-precision` | matches: "EXACTLY 8 bits" of fraction is the comment's "1/256 of a pixel" |
| `shadow_mask.wgsl:39` | `vulkan-subpixel-precision-bits` | matches: the required minimum is 4 bits, the comment's "as coarse as 1/16" |
| `shadow_mask.wgsl:91` | `nvidia-2022-rtx-best-practices-shadow-rays` | matches: "Terminate rays on the first hit when possible [...] (as for shadow rays)" is `RAY_FLAG_TERMINATE_ON_FIRST_HIT` at `:166`; "the query objects must hold state [...] This consumes registers and complex user code may limit occupancy" is the one-call-site design at `:96` and the normals read moved to `list_shadow_edges` (`:596-598`) |
| `shadow_mask.wgsl:386`, `:596` | `lauritzen-2010-per-sample-scheduling` | matches: "Shade sample 0 for the whole tile [...] Pack a list of pixels that require per-sample shading [...] Redistribute threads to process additional samples" is the nearest surface traced in place, the second surfaces and shadow edges listed with atomics and traced by indirect dispatches (`:434-447`, `:610-621`) |
| `shadow_mask.wgsl:410` | `lauritzen-2010-edge-detection` | matches in form: "Compare sample depths to depth derivatives" is done against the nearest sample's triangle plane, the exact plane its derivatives would give, with a one-pixel-width threshold the code chooses (`:418-426`) |
| `src/render/shadow_mask_pass.rs:38` (declared into WGSL as `SAMPLE_OFFSETS`) | `vulkan-standard-sample-locations`, `webgpu-standard-sample-pattern` | matches: the four locations are the quotes' verbatim, in the quotes' order, with the quotes' top-left origin; the mask rebuilds each sample at them (`:211`) and the Rust test at `shadow_mask_pass.rs:415-466` checks the backend's depth target against them |
| `main.wgsl:223`, `src/render/targets.rs:32` (the mask's two layers) | `salvi-2012-sbaa-surfaces` | matches: "store only a small and fixed (e.g. 2 or 3) number of surfaces per pixel" and "only one color sample per surface" is `SHADOW_MASK_SURFACES = 2`, one word per surface, each sample's surface marked (`:457`, `:498`, `:467`); which surface is the second when there are three or more, the farthest, is the code's choice, recorded in ADR 0019 and not in the quote |

### Oracles

| Function | Test | Constant | WGSL floor | Stated error | Verdict |
|---|---|---|---|---|---|
| `fresnel_weight` | `fresnel_rises_from_the_base_reflectance_to_one_at_grazing`, `:20` | `TOLERANCE` 1e-6 absolute | 7 roundings on values ≤ 1: 7u ≈ 4.2e-7 absolute (8.3e-7 under round toward zero); the 1 + ulp case gives a negative complement whose fifth power underflows to 0 | "within 2e-7 of theirs on the M4 Pro" (`:12-13`) | explained |
| `distribution_ggx`, the hemisphere sum | `distribution_ggx_integrates_to_one_over_the_hemisphere`, `:60`, `:109-110` | 1e-5 absolute on the integral | per sample D within ≈ 1.0e-6 relative (block A); the sum of positive terms inherits it: ≈ 1.0e-6 + the sampling's 1e-7 | "within 2e-7 of 1 on the M4 Pro; the sampling alone is off by under 1e-7" (`:107-108`) | explained, 9× the floor |
| `distribution_ggx`, the peak 16/π | the same test, `:124-125` | `TOLERANCE` 1e-6 absolute on 5.093 | one division and the PI constant: 5.093 × (3.0e-7 + 2.8e-8) ≈ 1.7e-6 at 2.5 ulp; 4.4e-7 if the division is correctly rounded (block A) | 2e-7 | tight under a 2.5 ulp division; explained where division is correctly rounded (Notes) |
| `distribution_ggx`, Appendix B | `distribution_ggx_matches_appendix_b_from_the_roughness_floor_to_one`, `:136` | `RELATIVE_TOLERANCE` 4e-6 relative | ≈ 1.1e-6 relative (block A); no cancellation: both terms of the denominator are positive and 1 − (N·H)² arrives from f64 | "within 2e-6 of theirs on the M4 Pro" (`:1184-1186`) | explained |
| `smith_visibility` | `smith_visibility_matches_appendix_b_and_keeps_g_within_zero_to_one`, `:170` | 4e-6 relative; G ≤ 1 + 1e-6 | ≈ 7.8e-7 relative (block B); the test's cosines are exact inputs, so no grazing cancellation | 2e-6 | explained |
| `metallic_roughness_brdf` | `metallic_roughness_brdf_matches_appendix_b_and_is_reciprocal`, `:210` | 4e-6 relative, match and swap | match ≈ 2.5e-6 relative where the specular lobe dominates (D 1.1e-6 + V 7.8e-7 + F and the mix 5u), ≈ 3e-7 where diffuse does; 5e-6 under round toward zero. Swap: the two terms of V's denominator exchange, so the swap differs only where a fused multiply-add breaks the symmetry, ≈ 2u (block B) | 2e-6 | explained for the match (tight under round toward zero); the reciprocity half is loose by about 30× on the same constant (Notes) |
| `reflected_luminance`, the ten direction cases | `reflected_luminance_follows_the_brdf_and_a_white_metal_reflects_at_most_what_it_receives`, `:290`, `:437-443` | `DIRECTION_TOLERANCE` 5e-5 relative, plus brdf × γ₃ absolute | ≈ 1.1e-4 relative for the grazing-view, floor-roughness metal case (view 90°, light 45°): V's shadowing term is \|N·V\| × 0.707 where N·V should be 0 and the f32 dot errs by up to γ₃ ≈ 1.8e-7, against a masking term of \|N·L\| α = 1.13e-3 (block C); ≈ 5e-6 for the other nine | "up to 1.1e-5 of the result on the M4 Pro" (`:1189-1193`) | tight; the comment names D's steepness, which these cases never approach: L1-S01 |
| `reflected_luminance`, the furnace sums | the same test, `:458-462` | `FURNACE_SUM_TOLERANCE` 1e-5 absolute | per cell ≈ 1e-6 to 2e-6 relative (block C: the L + V cancellation is exact by Sterbenz, the normalisation's 2 ulp scales H alone); coherent over a sum ≤ 1: ≈ 2e-6 absolute | "within 2e-7 of each other on the M4 Pro" (`:485-487`) | explained |
| `reflected_luminance`, the bounds | the same test, `:474-481` | `QUADRATURE_TOLERANCE` 1e-3 | a quadrature bound, not a rounding floor: the cells' refinement difference | "within 7.1e-4 of sums over cells half or a quarter their size" (`:489-492`) | explained |
| `point_light_attenuation` | `point_light_attenuation_falls_by_inverse_square_to_zero_at_the_range`, `:501` | 4e-6 × inverse square | ≈ 8e-7 of the inverse square: the window's absolute error ≈ 4.2e-7 (one division, two products, one difference: the difference's cancellation just inside the range leaves its absolute error unchanged, which is why the test judges against the inverse square) plus the final division's 3.0e-7 (block D) | 2e-6 | explained |
| `point_light_luminance` | `point_light_luminance_attenuates_the_intensity_it_reflects`, `:547` | `DIRECTION_TOLERANCE` 5e-5 × reflected × inverse square, without the window | ≈ 4e-6 relative at roughness 0.5 (block D): the direction's 4e-7 length error through D and V, the BRDF's 2.5e-6, the attenuation's 8e-7; just inside the range the tolerance is 5e-4 of the value, as the window is left out of it | none for this test; the constant's comment speaks of the roughness floor, which this test does not use | loose, 12×; folded into L1-S01 |
| `ambient_luminance` | `ambient_luminance_is_the_base_colour_over_pi_darkened_by_occlusion`, `:625` | 4e-6 relative | ≈ 5e-7: product, division by the same PI constant (3.0e-7), product | 2e-6 | explained |
| `tangent_space_normal` | `tangent_space_normal_scales_x_and_y_and_normalises_as_gltf_writes_it`, `:1298` | `TOLERANCE` 1e-6 absolute per axis | ≈ 6e-7: x, y, their squares, the sum and 1 − sum exact at 1/256 steps; `sqrt` 2 ulp on z; the test's `normalize` 3u + 2 ulp + u (block E); 1.2e-6 under round toward zero | 2e-7 | explained; tight under round toward zero |
| `specular_antialiased_roughness` | `specular_antialiased_roughness_adds_the_normals_variance_to_alpha_squared`, `:1353` | 4e-6 relative | ≈ 3e-7 to 5.5e-7: two dots (3u each) and a sum (u) on the filter term, α⁴ 2u, the sum u, then two square roots quarter the relative error and add 2 × 2 ulp (block E) | 2e-6 | explained, 7 to 12× |
| `decode_surface` | `decode_surface_scales_each_texel_by_its_factor_and_floors_roughness`, `:1404` | exact (`assert_eq!`) | 0 for the products, which are of binary fractions; the widened roughness passes through `sqrt(sqrt(81/4096))`, exact only if `sqrt` returns the exact root of a perfect square (9/64, then 3/8); the second case ends at the floor's `max`, where the roots do not matter (block E) | none; the comment says "each product is exact" and does not mention the roots | tight for the roughness channel of the first case, by Vulkan's `sqrt` allowance (Notes) |
| `relative_luminance` | `:657-677` | `TOLERANCE = 1e-6`, absolute | a 3-term dot: γ₅ Σ\|xᵢyᵢ\| ≤ 5u = 3.0 × 10⁻⁷ for inputs in [0, 1], plus 6 × 10⁻⁸ from the three coefficients' f32 representation: 3.6 × 10⁻⁷ (7 × 10⁻⁷ under round-toward-zero) | "within 2e-7 on the M4 Pro" (`:12-13`) | explained |
| `resolve_weighted`, `resolve_unweighted` | `:690-751` | `RELATIVE_TOLERANCE = 4e-6` × max(expected, 1) | block below: 1.7 × 10⁻⁶ relative on the worst case, 3.4 × 10⁻⁶ under round-toward-zero; the limits are exact | "within 2e-6 on the M4 Pro" (`:1185-1186`) | explained |
| `gt7_curve` | `:758-790` | `GT7_TOLERANCE = 3e-4`, absolute, frame-buffer units | block below: toe ≤ 1 × 10⁻⁷, shoulder ≤ 2.8 × 10⁻⁶; the reference's own f32 chain about the same, so 5 × 10⁻⁶ for the comparison | none for this test; the constant's "7e-5" is GT7's | loose, 60× for this test alone; the constant is shared and its comment attributes its size to PQ, which this chain has none of; not misleading, no Finding |
| `pq_encode`, `pq_decode` | `:796-836` | `GT7_TOLERANCE` absolute on the signal; `GT7_TOLERANCE × max(value, 1)` on the round trip; `1e-3` on 0.508 | block below: encode ≤ 7.8 × 10⁻⁵ relative (4 × 10⁻⁵ absolute at signal 0.5) by the table's worst case, from `log2(a) − log2(b)` × 78.84; the round trip divides that by PQ's log-slope 0.42 to 0.15, so 9 × 10⁻⁵ to 5.4 × 10⁻⁴ relative at frame-buffer 0.001 to 10; exact at 100, where both log2 arguments are the same f32 | none for this test | encode: explained; round trip: tight at frame-buffer 1, 2.5 and 10 by the table's worst case (4.0, 4.5, 5.4 × 10⁻⁴ against 3 × 10⁻⁴); passes where `log2` beats 3 ULP and its two nearby arguments share their error |
| `rec2020_to_ictcp`, `ictcp_to_rec2020` | `:841-890` | `GT7_TOLERANCE` on I against `pq_encode` and on Cт, Cᴩ at 0; `1e-3 × max(expected, 1)` on the round trip | grey I: the two chains share `pq_encode`; the matrix rows sum to 4096 exactly, so the dot differs from the input by ≤ 5u: 3 × 10⁻⁷ through PQ's slope ≤ 1 → ≤ 3 × 10⁻⁷; Cт, Cᴩ: γ₅ × 6.65 p ≤ 2 × 10⁻⁶; round trip: the six-digit inverses' product deviates from identity by 3.3 × 10⁻⁶ (derived), 2 × 10⁻⁵ relative after the row sums, under PQ's own round trip, 1 × 10⁻⁴ typical and 4.5 × 10⁻⁴ by the worst case | none | I and chroma: loose, 100× to 300× on a shared constant, harmless; round trip: explained, though the comment credits the six-digit matrices (4 × 10⁻⁵) for a margin PQ's f32 chain uses |
| `gt7_tonemap` | `:899-920` | `GT7_TOLERANCE`, absolute, display white 1 | 0.4 × curve/2.5 ≤ 1.2 × 10⁻⁶; 0.6 × an ICtCp round trip of values ≤ 2.5 over 2.5: 1 × 10⁻⁴ typical, 3 × 10⁻⁴ worst case; the sample runs in float, so the reference carries a like chain: 2 × 10⁻⁴ typical | "within 7e-5 on the M4 Pro" (`:1014-1015`) | explained |
| `tonemap` | `:931-950` | `GT7_TOLERANCE` | `gt7_tonemap`'s floor times the inverse matrix's largest absolute row sum, 2.32 (red from saturated green: 1.66 r − 0.588 g − 0.073 b): 2.3 × 10⁻⁴ typical; the 7-place matrices against the f64 reference's exact ones add 1.4 × 10⁻⁷ | "within 2.3e-4, where its conversion back to Rec. 709 subtracts near-equal terms for saturated green" (`:1016-1017`) | explained; 1.3× headroom over the stated error |
| `srgb_encode` | `:1025-1046` | `SRGB_TOLERANCE = 4e-6`, absolute | Vulkan: `log2` 3 ULP at \|log2 x\| ≤ 8.3 → 2.9 × 10⁻⁶ absolute, × 1/2.4, into `exp2` → ≤ 8.3 × 10⁻⁷ relative, plus `exp2`'s 10 ULP → 1 × 10⁻⁷ absolute at most after × 1.055; Metal: `pow` 16 ulp → 1.0 × 10⁻⁶ absolute at 1.0; the linear branch 2u × 0.04 | "a thousandth of an 8-bit step" (`:1047-1048`), stated as the unit, not a measurement | explained (4× the Metal bound) |
| `dither_noise` | `:1058-1083` | `TOLERANCE = 1e-6` per level; mean ≤ 1e-6; variance within 1e-4 of 1/6 | per level: `round`, `+ 0.5`, `/ 256`, `2u − 1`, `1 − \|c\|` all exact (9-bit dyadic values); `sqrt` ≤ 4.5 ULP ≤ 2.7 × 10⁻⁷, then `1 − sqrt` exact by Sterbenz: 2.7 × 10⁻⁷ absolute; mean: exactly 0 by the levels' symmetry k ↔ 255 − k; variance: the 256 bin centres give 0.166585, a midpoint bias of −8.2 × 10⁻⁵ against 1/6, derived, so the margin is 1.8 × 10⁻⁵ and the GPU adds ≤ 6 × 10⁻⁷ | "within 2e-7 on the M4 Pro" (the shared `TOLERANCE`) | per level: explained; variance 1e-4: unexplained in the comment, 1.2× the derived bias, no Finding |
| `srgb_encode(tonemap(..))`, the display ramp | `:1091-1165` | `DISPLAY_RAMP_TOLERANCE = 2 × GT7_TOLERANCE`, display-linear | two GPUs each within `tonemap`'s floor of the sample: 2 × 2.3 × 10⁻⁴; the file's 7-decimal encoded values add ≤ 5 × 10⁻⁸ × 2.4; 348 rows = 6 colours × 58 scales, as the file holds | none, derived from `GT7_TOLERANCE` (`:1163-1166`) | explained |
| `decode_vertex`, `unpack_qtangent`, `decode_qtangent`, `rotate` (`common.wgsl:252-289`) | `cooked_vertices_decode_to_their_position_uv_and_tangent_frame`, `:1479-1546` | position and uv exact; normal and tangent within 4 Snorm16 steps per component = 1.22e-4; sign exact | position, uv: storage loads, 0 roundings. Frame: quantisation ≤ 0.5 step per component plus the cooker's w nudge ≤ 1 step; a rotated axis's component is 2(ab + cd) with Σ\|q\| ≤ 2, so ≤ 2 steps from rounding and ≤ 2 from the nudge: 4 steps. f32 chain (unpack 1u, normalize ≈ 7u, two crosses and the sum ≈ 12u) ≈ 20u = 1.2e-6 to 2.4e-6, 4 to 8% of one step (block A) | none stated | explained: the doc at `:1470-1477` derives the 4 steps; f32 adds a few percent of a step, inside the slack of the worst case |
| `pack_index`, `unpack_index` (`common.wgsl:224-230`) | `a_packed_index_holds_its_meshlet_record_above_six_bits_of_vertex`, `:1548-1611` | equality, on 16-bit halves | integer shifts and masks, exact; `f32(u32)` of values ≤ 0xFFFF exact; floor 0 | none | explained (exact) |
| `position_at` (`common.wgsl:49-56`) | `position_at_rebuilds_the_point_a_depth_shows_through_a_pixel`, `:1618-1680` | Euclidean distance ≤ 2⁻²⁰ · max \|coordinate\| = 9.5e-7 · max \|pₖ\| | per component ≈ 19u · w from the add and multiply chain plus three divisions, each 1u on Metal or up to 2.5 ULP on Vulkan (a row the registry does not quote): ≈ 24u · w; over three components ≤ √3 of it ≈ 41u · w = 2.5e-6 · w (u = 2⁻²⁴) or 4.9e-6 · w (u = 2⁻²³); the bound is relative to w, since the sum forward + x·right + y·up cancels where a coordinate is near 0 while its terms are up to 2.15 w (block B) | none stated; the constant is motivated downstream, "sixteen times inside the 2⁻¹⁶ a shadow ray's origin leaves its surface by" | tight: the constant is 5.5e-7 · w to 9.5e-7 · \|p\|, 2.6 to 6.4 times under the worst-case floor; passes where rounding errors do not align |
| `octahedral_encode`, `octahedral_decode` (`common.wgsl:293-305`) | `octahedral_coordinates_decode_to_the_unit_vector_they_encode`, `:1682-1766` | coordinates in [−1, 1] exact; full round trip ≤ 4 · TOLERANCE = 4e-6; from 8 bits ≤ 0.95° | round trip: encode ≈ 4.5u on each coordinate (two adds, one division), fold 1u, decode's z two roundings ≤ 2u and the fold 1u, the pre-normalised vector no shorter than 1/√3, so ≈ 30u on the direction: 1.8e-6 (2⁻²⁴) or 3.6e-6 (2⁻²³). 8 bits: half a step 1/255 per coordinate moves the point ≤ 9.6e-3 on the face plane, over a length ≥ 1/√3: 0.0166 rad = 0.951° (block C) | TOLERANCE's "within 2e-7 ... on the M4 Pro" at `:12-14`; none for the factor 4 | explained: 4e-6 sits 1.1 to 2.2 times over the floor, and 0.95° is the quantisation worst case to three figures; the factor 4 on TOLERANCE is unstated (Notes) |
| `camera_relative_position` (`common.wgsl:31-40`) | `camera_relative_position_meets_the_precision_bar_within_100_km`, `:1781-1862` | PRECISION_BAR = 4.3e-5 m, on the GPU's order and on every CPU order | the anchor term is exact; in the written order the rounding is ≤ u(64 + 128.3 + 0.3) = 192.6u per axis, √3 of it 2.0e-5 m (2⁻²⁴) or 4.0e-5 m (2⁻²³); in the worst other order, local position added first, ≤ 256u per axis, 2.6e-5 m or 5.3e-5 m (block D) | the ADR's measured 0.0185 mm for every order | explained: the constant is the ADR's bar, derivable from a quarter pixel; the written order's floor is under it at both u; the reassociated, toward-zero corner is over it (Notes) |
| `segment_meets_box` (`common.wgsl:69-97`) | `a_segment_meets_a_box_where_some_point_of_it_lies_in_the_box`, `shader_function_tests.rs:1864-2049`, 19 cases | none: a boolean per case, `assert_eq!(result[0] == 1.0, *expected)` at `:2047` | a verdict can flip only where the exact margin from the separating boundary is below about 24 × 2^-24 × (largest coordinate) ≈ 1.5e-6 × M box units, M at most 37.5 among the cases, so below 6e-5; every case's margin is at least 1 box unit, or the case lies exactly on a face and its deciding comparison is exact (block below) | none stated | explained: the doc comment (`:1856-1863`) says what each case exercises and the cases' margins are derivable; it does not say that the two on-face cases pass by exact arithmetic rather than margin (see Notes) |


#### Block A: `distribution_ggx`

`shading.wgsl:114-121`: `alpha_squared = alpha * alpha` (u); `normal_dot_half
* normal_dot_half * alpha_squared` (2u); `+ normal_cross_half_squared` (u,
the input itself rounded from f64 at u); `max` exact; `denominator *
denominator` (doubles the relative error, + u); `PI * …` (u, and PI's own
2.8e-8, since f32(3.14159265) is f32(π)); `alpha_squared / …` (2.5 ulp,
3.0e-7). Denominator: 5u = 3.0e-7; squared: 6.6e-7; times PI: 7.5e-7;
division: 1.05e-6; numerator's u: 1.1e-6 relative. At the floor's peak the
`max` returns α² exactly and the only errors are PI's and the division's.
Peak check: 0.0625 / (PI × 0.00390625), the product exact as a power-of-two
scaling, the quotient 5.0929…: PI's 2.8e-8 relative is 1.4e-7 absolute; a
correctly rounded division adds at most 2⁻²⁴ × 5.09 = 3.0e-7, a 2.5 ulp
division up to 2.5 × 2⁻²¹ = 1.2e-6.

#### Block B: `smith_visibility` and the BRDF mix

`shading.wgsl:136-144`: α² (u); 1 − α² (u, exact at α = 1); c² (u); product
(u); + α² (u): 4u inside each root, halved by the root, + 2 ulp for the root
itself: 3.6e-7; × |c'| (u): 4.2e-7; the two terms summed (u): 4.8e-7; the
reciprocal (2.5 ulp, 3.0e-7): 7.8e-7 relative. Where both cosines are 0 the
denominator is 0 and the select returns 0. Reciprocity: under the swap
`shadowing_term` becomes the old `masking_term` and vice versa, bit for bit,
and IEEE addition commutes, so the denominators are identical unless the
compiler fuses one product into the addition (`msl-4.1-fast-math`'s
contraction), which perturbs one term by u: the swap's floor is about 2u.

The mix, `shading.wgsl:183-193`, against the quoted "full code"
(`gltf-2.0-metal-and-dielectric-brdf`): with w the Fresnel weight, F_d =
0.04 + 0.96 w, F_m = c + (1 − c) w, s the specular lobe and m metallic,
Appendix B's material = (1 − m)[(1 − F_d) c/π + F_d s] + m F_m s =
(1 − m)(1 − F_d) c/π + [(1 − m) F_d + m F_m] s. The bracket is
(1 − m)(0.04 + 0.96 w) + m(c + (1 − c) w) = [(1 − m) 0.04 + m c] +
w[(1 − m) − (1 − m) 0.04 + m − m c] = F₀ + w (1 − F₀) with
F₀ = mix(0.04, c, m): the code's `specular_fresnel`. The diffuse share is the
code's `diffuse_share`. Equal term by term.

#### Block C: `reflected_luminance`

Direction case (view 90°, light 45° at azimuth 180°, metallic 1, roughness
0.04), `shader_function_tests.rs:313`, `:316-318`: N = tilt·Z, V = tilt·X,
L = tilt·(−0.707, 0, 0.707), each rounded to f32 and read in f64 by the
reference, so the inputs are shared and only the shader's arithmetic
differs. In `smith_visibility` with α = 0.0016: `shadowing_term` = |N·V| ×
√(α² + (1 − α²)(N·L)²) = |N·V| × 0.7071, where the exact N·V of the f32
vectors is about 1e-8 and the f32 dot adds up to γ₃ ≈ 1.8e-7 (3.6e-7 at
the test's own u = 2⁻²³); `masking_term` = |N·L| × √(α² + (1 − α²)(N·V)²)
= 0.7071 × 0.0016 = 1.131e-3. V = 1/(2(shadowing + masking)), so δV/V =
δ(shadowing)/(shadowing + masking) = 0.7071 × 1.8e-7 / 1.131e-3 = 1.1e-4
(2.2e-4 at 2⁻²³). The result for a metal is F × D × V × N·L with D and
N·L far from any peak, so the result's relative floor is V's: 1.1e-4
against the constant 5e-5. D is not steep here: N·H = cos 67.5° = 0.383,
where |N × H|² = 0.853 and D's relative error is about 2 × 3u. A fused
dot product on the M4 Pro lands far inside γ₃ (the measured 1.1e-5 implies
a dot error near 1.2e-8), which is why the test passes there; a backend
that evaluates the three products and two sums separately may reach the
bound.

Furnace cells (`:321-348`, `:354-380`): N = Z exactly, V in the x-z plane,
L = 2(V·H)H − V in f64 then f32. In the shader `half_sum = L + V`: for a
cell near the normal L.x ≈ −V.x, and the sum is exact by Sterbenz's lemma
(V.x/2 ≤ |L.x| ≤ 2V.x), so the cancellation adds no error; the
normalisation `inverseSqrt(dot)` (3u + 2 ulp + u) scales every component of
H alike, so N·H, V·H and |N × H|² are off by the same 4e-7 and D's
denominator by 2 × 4e-7: ≈ 1e-6 relative per cell, ≈ 2e-6 with the BRDF
chain. Summed with positive weights over cells whose weights sum to at most
about 1, a coherent worst case is ≈ 2e-6 absolute.

#### Block D: the point light

`point_light_attenuation`, `shading.wgsl:235-237`: `range * range` (u);
`distance_squared / …` (3.0e-7); `ratio_squared * ratio_squared` (u); `1.0
- …` (u): the window's absolute error is the absolute error of the
subtracted term, ≈ 4.2e-7 × ratio⁴ ≤ 4.2e-7, which just inside the range
is a large share of the small window (at 3.999 m of 4 m the window is
1e-3, so 4e-4 relative), and the test rightly judges against the inverse
square; `window / distance_squared` adds 3.0e-7: ≈ 8e-7 of the inverse
square. `point_light_luminance`, `:254-257`: `dot` (3u), `max`,
`inverseSqrt` (2 ulp), product (u): the light direction's length is off by
≈ 4e-7, which moves N·L, N·H and V·H by as much, through D at α = 0.25
(≈ 2 × 4e-7 with the Lagrange sum) and V (≈ 4e-7), plus the BRDF chain's
2.5e-6 and the attenuation's 8e-7: ≈ 4e-6 relative to the reflected
value. The tolerance at `:605` is 5e-5 × reflected × inverse square without
the window: 5e-5 of the value where the window is 1, 5e-4 of it at 3.9 m
of a 4 m range.

#### Block E: the surface decode

`tangent_space_normal`, `shading.wgsl:157-159`: encodings k/256 give x =
(k − 128)/128 exactly, x² = (k − 128)²/16384 exactly (14 bits), the sum
and 1 − sum exact; `sqrt` 2 ulp on z (2.4e-7); × scale exact for 0.5, 1,
2; the test's `normalize`: `dot` 3u, `inverseSqrt` 2 ulp, product u: ≈ 6e-7
absolute per axis of a unit vector. `specular_antialiased_roughness`,
`:44-48`: two dots (3u each) summed (u) on the filter term, × 0.25 and × 2
exact, `min`; `roughness * roughness` (u), `alpha * alpha` (u), the sum
(u), `min`; two roots: a relative error ε inside becomes ε/4 outside plus
2 × 2 ulp. `decode_surface`, `:77-86`: with zero derivatives the filter
term is 0 and roughness returns as (roughness⁴)^(1/4): for 0.375,
0.375⁴ = 81/4096 exactly, √ = 9/64 exactly if `sqrt` is correctly rounded,
√ = 3/8 exactly likewise; the `assert_eq!` at `:1451-1454` needs both
roots exact. For 3/256 the roots are exact too, and the `max` with the
floor hides them.

#### The weighted resolve's floor

The GPU computes T(c) = c/(1 + L(c)) per sample (dot 5 roundings, add,
divide: γ₇ ≈ 4.2 × 10⁻⁷), sums four (3 roundings) and divides by 4
(exact): w carries ≤ 10u = 6 × 10⁻⁷ relative. The inverse computes L(w)
(5 roundings, ≤ 3 × 10⁻⁷ absolute plus w's 6 × 10⁻⁷), then 1 − L(w): in
exact arithmetic L(mean T) = 1 − ¼ Σ wᵢ, so the complement is ¼ Σ wᵢ =
1/(1 + L_c) for one sample, and the subtraction turns L(w)'s absolute error
into relative error amplified by (1 + L_c). For the fourth case,
`[2047, 1000, 3]` with three dim samples, the resolved colour is
(1.1001, 0.7663, 0.3679), L_c = 0.8085, so the floor is 10u + 9u × 1.81 +
2u = 1.7 × 10⁻⁶ relative; the second case's 0.395 has L_c = 0.395 and a
smaller floor. The identity Σ wᵢcᵢ / Σ wᵢ = T⁻¹(mean T(cᵢ)) is exact, so
the f64 oracle is the right number. The three limits are exact: a weighted
luminance at or past 1 makes the complement `1/65504`, and `min` returns
65504.0 bit-exactly.

#### The GT7 curve's floor

With `peak` 2.5, `alpha` 0.25, `mid_point` 0.538, `linear_section` 0.444,
`toe_strength` 1.28: `shoulder_rise` k = 0.741333, `shoulder_limit` A =
2.963333 (1.1853 × peak, the comment's "1.19"), `shoulder_scale` B =
−3.373351, `shoulder_rate` C = −0.539568. The branch is continuous with
slope 1 at 1.11: A + B·exp(1.11 C) = peak(l + k) − peak·k = 1.11. Toe
branch at 0.05: `smoothstep` ≈ 6 roundings on the weight, `pow` ≈ 1.4 ×
10⁻⁶ relative (Vulkan) or 1 × 10⁻⁶ (Metal 16 ulp) on a toe of 0.0263, the
blend 3 roundings: ≤ 1 × 10⁻⁷ absolute. Shoulder at 1.11, its worst: B's
eight roundings (k two, l/k one, `exp(0.599)` 4.2 ULP, two multiplies) give
8u × 3.37 = 1.6 × 10⁻⁶; `exp(1.11 C)` = 0.5494 with 4.2 ULP plus the
argument's 0.6u → 5.3 × 10⁻⁷ on B·exp; A's three roundings 5.3 × 10⁻⁷; the
sum 1.5 × 10⁻⁷: 2.8 × 10⁻⁶ absolute in frame-buffer units, 1.1 × 10⁻⁶ of
display white. The 18 reference values agree with the curve in f64 to 2.3 ×
10⁻⁷, the f32 printing of a float sample, so the comparison's floor is about
5 × 10⁻⁶. Against `GT7_TOLERANCE = 3e-4` that is a factor of 60.

#### PQ's floor and the round trip

`pq_encode` writes `exp2(m2 · (log2(c1 + c2 n) − log2(1 + c3 n)))`. The
two arguments lie between 3.85 and 19.69 for the test's inputs, so each
`log2` result (1.9 to 4.3) is bounded by 3 ULP: 3 × 2⁻²³ = 3.6 × 10⁻⁷
below 2, 3 × 2⁻²² = 7.2 × 10⁻⁷ between 2 and 4, 3 × 2⁻²¹ = 1.4 × 10⁻⁶
above 4. The difference's error, up to 2.9 × 10⁻⁶, is multiplied by m2 =
78.84 into the exponent, 2.3 × 10⁻⁴ at worst, which is a relative error of
2.3 × 10⁻⁴ × ln 2 = 1.6 × 10⁻⁴ on the signal, 7.8 × 10⁻⁵ where both
arguments sit between 2 and 4 or 4 and 8 together, as they do for every
input but 0.001 and 100. The comment's "PQ's exponents amplifying f32
rounding" is this. At signal 0.5 the absolute error is 4 × 10⁻⁵, so the
encode assertion's 3 × 10⁻⁴ is 4× to 8× its floor. At frame-buffer 100 the
arguments are both 19.6875 exactly (c1 + c2 = 1 + c3 in f32), the
difference is 0 and the signal 1.0 bit-exactly, as asserted at `:829-834`.

The round trip divides the encode error by PQ's log-slope d ln E′ / d ln Y
= m1 m2 n (c2/(c1 + c2 n) − c3/(1 + c3 n)), n = Yᵐ¹: 0.423 at
frame-buffer 0.001, 0.285 at 0.05, 0.236 at 0.25, 0.198 at 1, 0.175 at
2.5, 0.145 at 10. The decode's own chain adds less: `pow(s, 1/m2)` is
within 2 × 10⁻⁷, `power − c1` amplifies by 6 to 7 and `c2 − c3·power` by up
to 23, `pow(.., 1/m1)` by 6.3, together about 7 × 10⁻⁵ relative at 0.001.
So the round trip's worst-case floor is 9 × 10⁻⁵ relative at 0.001 (1 ×
10⁻⁷ absolute, against 3 × 10⁻⁴), 2.7 × 10⁻⁴ at 0.05 (1.4 × 10⁻⁵ against
3 × 10⁻⁴), 3.3 × 10⁻⁴ at 0.25 (8 × 10⁻⁵ against 3 × 10⁻⁴), and 4.0, 4.5 and
5.4 × 10⁻⁴ relative at 1, 2.5 and 10, where the tolerance is 3 × 10⁻⁴
relative. The test passes on the M4 Pro and in CI's lavapipe because
neither `log2` is 3 ULP off, and because the two arguments are within 5% of
each other, so one implementation's error on them is nearly the same and
cancels in the difference. A `pow((c1 + c2 n)/(1 + c3 n), m2)` form would
put one `log2` inside [0.5, 2], bounded at 2⁻²¹ absolute, for a worst case
of 4.4 × 10⁻⁵ in the exponent, five times smaller; whether the GPU's error
actually approaches the bound needs a run, so this stays a Note.

#### Floor arithmetic

Block A, the tangent frame. The decode is `normalize(q)` then
`rotate(q, e)`, `common.wgsl:252-266`. Each Snorm16 component is off by
≤ 0.5 step = 1.53e-5 before the nudge (`src/cook/mesh.rs:316-320` sets w
to one step where it would round to zero and shortens the axis to match).
A rotated axis's component is a sum of two products of two quaternion
components, doubled, so its derivative in any one component is ≤ 2 Σ|qᵢ|
≤ 4 (Σ|qᵢ| ≤ 2 for a unit quaternion): ≤ 4 · 0.5 step from rounding and
≤ 4 · 0.5 step more from a one-step nudge shared between w and the axis,
4 steps in all, 1.22e-4, which the constant takes. The f32 chain adds:
`unpack2x16snorm` one division per component (1u, or 2.5 ULP on Vulkan),
`normalize` a four-term dot (γ₄), an inverse square root (≈ 2 ULP) and a
product (1u), `rotate` two cross products of two products and a
subtraction each (γ₂ on unit terms) around a fused `+ w v` and the final
sum: ≈ 20u ≈ 1.2e-6 to 2.4e-6 per component, 4 to 8% of a step.

Block B, `position_at`. With view_proj = P · R, R a rotation, the rows
are (f/a)·R₀, f·R₁, (0, 0, 0, n) and the forward axis, so right = R₀/(f/a),
|right| = a/f = 0.736 at 16:9, |up| = 1/f = 0.414, |forward| = 1, with
f = 1/tan 22.5° = 2.414. Per component k of w · (Fₖ + dₓ Rₖ + d_y Uₖ):
device coordinate dₓ = fl(fl(x/sₓ) · 2 − 1): ≤ 3u absolute; Rₖ: a three-term
dot of positives (γ₃) and a division (1u) then the component's division
(1u): ≤ 5u relative; dₓ Rₖ: ≤ |Rₖ| (6u |dₓ| + 3u) ≤ 9u |R|; likewise d_y Uₖ ≤
9u |U|; two additions ≤ 2u (|F| + |A| + |B|); the product with w ≤ 2u |S|
(w's division 1u, the product 1u). Summed with |S| ≤ 2.15: ≈ 19u · w per
component, plus the divisions' excess over 1u on Vulkan (three at 2.5 ULP,
≈ 4.5u): ≈ 24u · w; the Euclidean distance over three components ≤ √3 of
it, ≈ 41u · w: 2.5e-6 · w at u = 2⁻²⁴, 4.9e-6 · w at 2⁻²³. The bound is in w
because the sum cancels: a coordinate of p can be near 0 while F, dₓR and
d_yU are not. The constant is 2⁻²⁰ · max|pₖ| with max|pₖ| ≥ |p|/√3 ≥ w/√3,
so 5.5e-7 · w at the diagonal and 7.6e-7 · w at this camera's centre pixel
(|F| max component 0.8): 2.6 to 6.4 times under the worst-case floor at
u = 2⁻²⁴. A sum of ten independent roundings lands near √10 u in
practice, 1.9e-7 · w per component, so the test passes where the roundings
do not align, which is the hardware's luck, not the floor's.

Block C, octahedral. Encode: fl(|x| + |y| + |z|) γ₂, the division 1u (2.5
ULP on Vulkan), so ≤ 4.5u on each projected coordinate (|p| ≤ 1); the fold
1 − |p.yx| one rounding, 1u. Decode: z = fl(fl(1 − |x|) − |y|), two
roundings on magnitudes ≤ 1, ≤ 2u absolute (where z ≈ 0 the cancellation
leaves this absolute error, not a relative one); the fold 1u; so the
pre-normalised vector v = (x, y, z) has component errors ≈ 6.5u, 6.5u and
≈ 13u (z carries x's and y's); |v|² = x² + y² + z² with |x| + |y| + |z| = 1
is at least 1/3 at a face centre, so a component error moves the
direction by ≤ √3 times it: ≈ √(6.5² + 6.5² + 13²) u · √3 = 27.5u, plus
`normalize`'s own ≈ 3u: ≈ 30u, 1.8e-6 at 2⁻²⁴, 3.6e-6 at 2⁻²³, against
4e-6. From 8 bits: a coordinate stored as round(255 (c + 1)/2)/255 · 2 − 1
is off by ≤ 1/255 = 3.92e-3; both coordinates off moves the point on the
face plane by ≤ √(2 · 3.92e-3²) in xy and the two together move z by
≤ 2 · 3.92e-3, so ≤ √(5.5e-3² + 7.8e-3²) = 9.6e-3 in all, over |v| ≥ 1/√3:
≤ 0.0166 rad = 0.951°, the constant.

Block D, camera-relative composition, `common.wgsl:38-39`. The anchor
difference is an i32 subtraction, converted to f32 exactly (≤ 3,125 cells
within 200 km), times 64 exactly (a power of two, product under 2²⁴ m):
no rounding. offset − camera_offset, both in [0, 64): ≤ 64u. Plus the
anchor term: the sum is the pivot's position relative to the camera,
≤ 128.3 m in the test (local positions to 128 m, the point at 0.3 m):
≤ 128.3u. Plus the local position: the result is 0.3 m, ≤ 0.3u. Per axis
≤ 192.6u: 1.15e-5 m at 2⁻²⁴, 2.3e-5 m at 2⁻²³; over three axes √3 of it,
2.0e-5 m and 4.0e-5 m, both under 4.3e-5. If a compiler adds the local
position to the offsets first (|sum| ≤ 192 m, ≤ 192u) and the anchor term
last (≤ 0.3u): ≤ 256u per axis, 2.6e-5 m at 2⁻²⁴, which the CPU's
every-order check bounds and the ADR's 1.85e-5 measured lies under, but
5.3e-5 m at 2⁻²³, over the bar: a GPU that both reassociates and rounds
toward zero could miss the bar in a worst case, which the oracle exists
to show.

#### Floor: `segment_meets_box`

Backends: addition, subtraction and multiplication are correctly rounded on all three (`vulkan-spirv-precision`: "OpFAdd: Correctly rounded. [...] OpFMul [...] Correctly rounded"; `msl-4.1-fast-math` Table 8.2: "x + y: Correctly rounded [...] x * y: Correctly rounded [...] fma: Correctly rounded"; `d3d11.3-fp32-rules`: "within 0.5 ULP [...] addition, subtraction and multiplication"), so each is one rounding of at most u = 2^-24 relative (`higham-3.1`, γₙ ≈ nu). `dot` inherits its products and sums (`vulkan-spirv-precision`). `length` adds a square root, which none of the quoted entries bounds; it is taken as a few ulps below. Metal's fast math may contract to fma and reassociate, which changes which roundings happen but not their count's order.

The chain (`common.wgsl:77-96`): `centre`, `half_extent` (one subtraction or addition and one multiplication by 0.5, exact), `from_centre` (1), two lengths (a 3-term dot, γ₃, then a square root), `cut_end_distance` (1 addition, then an exact `min`), `half_length` (1 subtraction, exact halving), `start_distance + half_length` (1), `direction × that` (1), `+ from_centre` (1), `half_segment` (1), `reach` (exact), `half_extent + reach` (1). The three box-axis comparisons each carry about 10 roundings relative to the magnitudes involved, at most |from_centre| + half-diagonal: 10u × 2M. The three cross-axis comparisons multiply those quantities (`cross(midpoint, half_segment)`: 2 products and a subtraction each, with cancellation when the midpoint and the segment are nearly parallel; `crossed_reach`: 2 products and an addition), so their absolute error is about 12u × |midpoint| × |half_segment|; divided by |half_segment| to read it as a distance from the box, about 12u × |midpoint| ≤ 24u × M. Floor, in box units: 24 × 2^-24 × M ≈ 1.4e-6 × M.

Margins of the cases, exact arithmetic, M the largest coordinate in play:

- "through", "ends short", "starts past", "box behind", "parallel outside", "parallel inside", "from inside": margins of 1 to 4 on the deciding axis, M ≤ 10, dyadic inputs; the deciding quantities are exact.
- "box behind, unbounded": the cut is min(f32::MAX, 5 + √3) ≈ 6.732; midpoint x ≈ 8.366 against 1 + 3.366 = 4.366: margin 4.
- "towards, unbounded": M = 1000; the cut ≈ 1001.73; the x axis margin ≈ 2.7, the cross z axis |−250.43| against 500.87: margin 250; floor 1.4e-3.
- "diagonal through", "from inside, unbounded": the midpoint and the segment share one direction with bitwise-equal components, so `cross` is exactly 0 against a positive reach; the axis margins are 2 and 1.
- "clears a corner": cross z = h × (m.x − m.y) with m.x − m.y = −2.5, so |−6.47| against 5.18: margin 1.29; the error of m.x − m.y is two roundings of 2.59, about 3e-7.
- "cuts a corner": |−3.9| against 5.2 on the cross z axis: margin 1.3 (0.5 h).
- "flat box": `half_extent.y = 0`; the y axis compares 0.79 against 0 + 2.21: margin 1.4.
- "far box": M = 37.5 (the origin is 12.5 below the centre, the cut 34.23); the y axis compares 4.87 against 16.5 + 16.87: margin 28.5; the cross axes are exactly 0 (the direction is an axis, the midpoint along it). Floor 5e-5.
- "far box, ends short": the y axis compares |−27.25| against 16.5 + 9.75 = 26.25: margin 1.0, every quantity dyadic (−37.5 + 0.5 + 9.75) and exact. Floor 5e-5.
- "ends on the face": the cut is min(4, 5 + √3) = 4 exactly; the x axis compares 3 against 1 + 2 with `>`: equality, exact, so the face is inside, as the doc comment promises ("its faces included").
- "along a face": the only inexact value is the half-length h ≈ 3.4155 (from the cut 5.099 + 1.732); the y axis compares 1 against 1 + |0 × h| = 1, and the cross z axis compares |(−1.58)(0) − 1 × h| = h against 1 × 0 + 1 × h = h: each side is h multiplied by exactly 1 or 0 and added to exactly 0, so both sides are the same f32 whatever h rounds to, under fma contraction too. Exact by structure, not by luck.

Smallest non-exact margin 1.0 against a floor of at most 6e-5: four orders of magnitude. No case can flip on any backend; the test's boolean form needs no constant.

### Findings

| id | claim | severity | where | evidence | expected |
|---|---|---|---|---|---|
| L1-S01 | `DIRECTION_TOLERANCE`'s comment attributes its 5e-5 to D's steepness near its peak at the roughness floor, but no case it guards approaches that peak; the error that drives it is Smith V's shadowing term at a grazing view, whose worst-case floor (1.1e-4) is above the constant, while the point light test it also guards has a floor of 4e-6 at roughness 0.5. | `drift`; not load-bearing (a test's tolerance rationale); misleads a reader of the oracle and anyone porting it to a backend without fused dot products | `src/render/shader_function_tests.rs:1189-1193`; used at `:440`, `:605` | block below | The comment names the term and the case that set the floor (V's \|N·V\| term against \|N·L\| α at the floor roughness, γ₃ of the dot) and the constant or the case is chosen so the floor is below it, as `reflected_luminance`'s own `rounding` term already does for N·L |
| L1-S02 | `point_light_luminance` holds a point light's distance² at 1 cm² and shortens its direction with the distance, deviating from KHR_lights_punctual's attenuation on purpose with Filament as the source, and `deviations.md`, which says a Deviation lives there and nowhere else, has no row for it; the deviation is recorded only in the code comment the file forbids. | `drift`; load-bearing for audits, which `deviations.md` is read before; one row missing, two code sites | `docs/audit/deviations.md:10-11`, `:20-21`; `src/render/shaders/shading.wgsl:223-226`, `:240-245`, `:254-256` | block below | A row: `shading.wgsl:point_light_luminance` deviates from `khr-lights-punctual-range`'s 1/d² within 1 cm, holding d² at 1e-4 and the direction shortened, because 1/d² is unbounded and the Filament floor keeps a surface at the light finite; revisit when area lights arrive |
| L1-S03 | `dither_noise` feeds Peters's triangular remap the level's bin centre (k + ½)/256 where the cited listing feeds the mask's texel value k/255, a deliberate deviation from the quoted source that `docs/audit/deviations.md` has no row for. | `correctness`; at most 1/16 of an 8-bit code, at the mask's two extreme levels, 1/128 at the middle ones; no visible defect | `display.wgsl:47-51`; `docs/audit/deviations.md`, the empty table | block below | the lane's rule: a deviation the code makes on purpose has a row in `deviations.md` naming its source, why, and when to revisit; or the listing's form |
| L1-S04 | The comment on `GT7_TOLERANCE` says the constant is "on a scale where 1 is display white", but two of its five uses compare on other scales: `gt7_curve` in frame-buffer units, where display white is 2.5, and PQ in signal units, where 1 is 10,000 cd/m². | `drift`; not load-bearing: the constant's size is explained by its tightest use; a reader of the curve test would take 3e-4 for 1/8 of a code where it is 1/20 | `shader_function_tests.rs:1013-1018`; `:787` (`gt7_curve`, frame-buffer units); `:818-826` (PQ signal and its round trip) | block below | a comment that names the unit per use, or per-test constants |
| L1-S05 | ADR 0014 and ADR 0019 say the culling pass composes each Instance's camera-relative transform once per Instance now that draws are GPU-driven; the culling pass composes only the centre of the Instance's bounds, for its own test, and writes nothing the vertex shader reads, which composes every vertex itself. | `drift`; not load-bearing: the precision bar is met by the per-vertex composition the oracle checks, and the Instance buffer keeps anchor plus offset either way; a reader of either ADR expects a per-Instance transform buffer that does not exist | `docs/adr/0014-world-positions-are-f64-the-gpu-works-camera-relative.md`, Consequences, first bullet; `docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md`, Consequences, first bullet; `culling.wgsl:119-138`; `main.wgsl:95-101` | block below | either ADR says what the code does: the culling pass composes the bounds' centre for its test, and the vertex shader composes each vertex from its Instance's anchor and offset, or the pass writes the transform the ADRs describe |
| L1-S06 | ADR 0020 says a light "the surface's face turns from is hidden without" a ray, but the mask decides the turn from the interpolated normal, the face normal choosing only the ray's origin side. | `drift`; not load-bearing: ADR 0019's normals amendment records the interpolated test, so a reader of both is not misled, but a reader of ADR 0020 alone would file the interpolated test, or the terminator rays it enables, as a departure | `shadow_mask.wgsl:278-286`, `:311`, `:225-228`, `:158-162`; `docs/adr/0020-ray-traced-probe-gi-and-hard-ray-traced-shadows.md`, line 3, the implemented-status sentence | block below | ADR 0020's sentence names the interpolated normal, or defers to ADR 0019's amendment, as its later sentences name the surface's samples |


#### L1-S01

The constant and its comment, `src/render/shader_function_tests.rs:1189-1193`:

```rust
/// The same for shading from unit vectors: f32 rounds each direction and
/// each cosine between them by up to about 1e-7, which D, steep near its
/// peak at the roughness floor, turns into up to 1.1e-5 of the result on the
/// M4 Pro.
const DIRECTION_TOLERANCE: f64 = 5e-5;
```

It guards the ten direction cases of `reflected_luminance` (`:309-319`,
asserted at `:437-443`) and every case of `point_light_luminance` (`:605`).
The direction cases' half vectors sit at 67.5° or more from the normal, or
are zero (the opposite light), so D is never near its peak; the point light
test runs at roughness 0.5, not the floor. The error that reaches 1.1e-5 on
the M4 Pro, and the worst-case floor of 1.1e-4, come from
`smith_visibility` (`src/render/shaders/shading.wgsl:139-143`):

```wgsl
let shadowing_term = abs(normal_dot_view)
    * sqrt(alpha_squared + (1.0 - alpha_squared) * normal_dot_light * normal_dot_light);
let masking_term = abs(normal_dot_light)
    * sqrt(alpha_squared + (1.0 - alpha_squared) * normal_dot_view * normal_dot_view);
```

at the case (view 90°, light 45° at azimuth 180°, metallic 1, roughness
0.04): N·V is 0 in exact arithmetic, so `shadowing_term` is whatever the
f32 dot of two rounded unit vectors leaves, up to γ₃ ‖N‖ ‖V‖ ≈ 1.8e-7
(`higham-3.1`: "|xᵀy − fl(xᵀy)| ≤ γₙ |x|ᵀ|y|"), times 0.7071, against
`masking_term` = 0.7071 × 0.0016 = 1.131e-3; the reciprocal's relative error
is the ratio, 1.1e-4, or 2.2e-4 at the u = 2⁻²³ the test takes for Metal's
round toward zero (`msl-4.1-rounding`). The reference,
`reference_visibility` (`:1240-1253`), takes N·V in f64 from the same f32
vectors, so this difference is not shared. The result is F × D × V × N·L
with D at N·H = 0.383 (|N × H|² = 0.853, relative error about 2 × 3u) and
N·L = 0.707 (its own γ₃ covered by the `rounding` term at `:419-426`), so
the result's floor is V's: 1.1e-4 against 5e-5. The test passes because a
fused dot product lands far inside γ₃ (the measured 1.1e-5 implies a dot
error near 1.2e-8); a backend evaluating three products and two sums
separately, each correctly rounded as `vulkan-spirv-precision` allows
("OpDot(x, y): Inherited from Σ xᵢ × yᵢ"), may reach 1.5e-7 and fail. Block
C has the arithmetic; Block D the point light test's 4e-6 floor, 12× under
the constant, with no window in its tolerance.

#### L1-S02

`docs/audit/deviations.md:10-11`:

> A Deviation lives here and nowhere else: not in a code comment, where no
> number may sit and nobody reads across the tree

and its table (`:20-21`) has no row. `src/render/shaders/shading.wgsl:223-226`:

```wgsl
// Point lights hold distance² at 1 cm² or more, as if 1 cm in radius, so a
// surface at one is lit finitely; Filament (Apache-2.0) holds the same
// […the registry citation of filament-punctual-light-1cm…].
const MINIMUM_LIGHT_DISTANCE_SQUARED: f32 = 1e-4;
```

and `:254-256`:

```wgsl
let distance_squared = max(dot(light_offset, light_offset), MINIMUM_LIGHT_DISTANCE_SQUARED);
let illuminance = intensity * point_light_attenuation(distance_squared, range);
let light_direction = light_offset * inverseSqrt(distance_squared);
```

against `khr-lights-punctual-range`: "attenuation = max( min( 1.0 − (
current_distance / range )⁴, 1 ), 0 ) / current_distance²", unbounded at
the light. Within 1 cm the code's illuminance is I × window / 1e-4 rather
than I × window / d², and `light_direction` is shorter than unit, so N·L,
N·H and V·H are scaled by d / 0.01 and H·L no longer equals H·V. The code
is right to differ, its reason is Filament's and its oracle
(`shader_function_tests.rs:599`) bakes the same floor into the reference,
so the deviation is tested but recorded only where `deviations.md` says
none lives. The comment at `:240-245` is the second site. One row is
expected; whether the specular antialiasing widening and the roughness
floor's value join it is the owner's (Notes).

#### L1-S03

The code (`display.wgsl:47-51`):

```
fn dither_noise(mask_value: f32) -> f32 {
    let uniform = (round(mask_value * 255.0) + 0.5) / 256.0;
    let centred = 2.0 * uniform - 1.0;
    return sign(centred) * (1.0 - sqrt(1.0 - abs(centred)));
}
```

The source, `peters-blue-noise`, as the registry quotes it: "// Go from a
uniform distribution on [0,1] to a symmetric triangular distribution on
[-1,1] with maximal density at 0 / BlueNoise=mad(BlueNoise,2.0f,-1.0f);
BlueNoise=sign(BlueNoise)*(1.0f-sqrt(1.0f-abs(BlueNoise)));", where
`BlueNoise` is the mask's texel value, k/255 for an 8-bit texel k.

The code's own comment states the change (`display.wgsl:39-41`): "one of
256 levels, each standing for its bin's centre in (0, 1), turned by the
inverse of the triangular distribution's CDF, as Peters remaps a blue-noise
mask for dithering", and the test (`shader_function_tests.rs:1051-1057`)
checks the bin-centre form.

Arithmetic over the 256 levels: Peters's form maps level 0 to −1.0 and level
255 to +1.0; the code maps them to −0.9375 and +0.9375 (1 − √(1/256)), a
difference of 0.0625 codes; at levels 127 and 128 the forms differ by 7.7 ×
10⁻⁶. Both have mean exactly 0. The code's variance over the levels is
0.166585, Peters's 0.170204, against the triangular distribution's 1/6 =
0.166667: the code's form is the closer discretisation, so the deviation is
an improvement on the listing, which is why it belongs in `deviations.md`
rather than in a code change. The Finding is against the missing row, not
against the function; the triage may also judge that the comment's own
sentence is record enough, in which case `deviations.md`'s rule ("A
Deviation lives here and nowhere else: not in a code comment") is the text
to amend.

#### L1-S04

The comment (`shader_function_tests.rs:1013-1018`): "Largest difference
allowed between GT7 on the GPU and the sample's outputs, on a scale where 1
is display white. On the M4 Pro, GT7 lands within 7e-5 of them, PQ's
exponents amplifying f32 rounding, and `tonemap` within 2.3e-4, where its
conversion back to Rec. 709 subtracts near-equal terms for saturated green:
a fifth of an 8-bit step."

Uses on that scale: `gt7_tonemap` (`:912-917`), `tonemap` (`:944-949`), both
outputs in [0, 1] with 1 display white; `ictcp_holds_a_grey` (`:868-876`)
compares PQ-encoded intensities, which is a signal scale but of the same
order. Uses off it: `gt7_curve_matches_polyphonys_sample_implementation`
(`:787`) compares frame-buffer values up to 2.963, where display white is
`GT7_PAPER_WHITE` = 2.5, so 3e-4 there is 1.2e-4 of display white, a 20th of
a code rather than the comment's fifth; `pq_encodes_as_st_2084_and_decodes_back`
(`:818-826`) compares a PQ signal in [0, 1], where 1 is 10,000 cd/m², and
frame-buffer values on the round trip, relative above 1. The floors above
show the constant is sized by PQ and `tonemap`, not by the curve, whose
chain has no PQ in it: the comment's one reason does not cover its loosest
use.

#### L1-S05

ADR 0014, Consequences, first bullet: "The vertex shader composes
camera-relative positions until draws are GPU-driven; then the culling
pass does it once per Instance." ADR 0019, Consequences, first bullet:
"One compute culling pass composes each Instance's camera-relative
transform (ADR 0014), tests the frustum and a two-phase Hi-Z pyramid of
minimum reverse-Z depth". Draws are GPU-driven at this commit (the frame's
draw is `draw_indexed_indirect` from the culling pass's buffers). The
culling pass's one composition, `culling.wgsl:119-138`:

```wgsl
fn clip_space_bounds(instance: InstanceData, mesh: MeshData) -> ClipBox {
    ...
    let centre = camera_relative_position(
        instance.anchor,
        instance.offset,
        camera.anchor,
        camera.offset,
        instance.rotation_scale * (0.5 * (bounds_min + bounds_max))
    );
    return ClipBox(
        camera.view_proj * vec4<f32>(centre, 1.0),
        ...
```

is a value local to the test; no binding of the culling module receives a
composed transform (`src/render/bind_groups.rs:122-134`: offsets, records,
indices, the draw, visibility, the pyramid). The vertex shader,
`main.wgsl:95-101`:

```wgsl
    let position = camera_relative_position(
        instance.anchor,
        instance.offset,
        camera.anchor,
        camera.offset,
        instance.rotation_scale * vertex.position
    );
```

composes each vertex from the Instance's anchor and offset, as before
draws were GPU-driven. Both ADRs' sentences describe a hand-off that did
not happen. Nothing is wrong in the code: the per-vertex composition is
the one the precision oracle holds to the bar (`shader_function_tests.rs:1781`),
and the ADR's status line ("which the vertex shader composes
camera-relative") says what the code does. The drift is in the two
Consequences sentences, which say what the code does not; a reader
planning a change to the culling pass or the Instance layout would look
for a transform buffer. Severity `drift`, not load-bearing, two readers'
sentences.

#### L1-S06

Code, `shadow_mask.wgsl:278-286`:

```wgsl
// A light the interpolated normal turns from is hidden by the
// surface itself, with no ray.
direction = normalize(-lights[entry].offset);
distance = UNBOUNDED_RAY_DISTANCE;
if !(dot(interpolated_normal, direction) > -INTERPOLATED_TURN_MARGIN) {
    hidden = with_bit(hidden, entry);
    continue;
}
```

the same test for a point light at `:311` (`dot(interpolated_normal, to_light) <= -INTERPOLATED_TURN_MARGIN * distance`) and in `directional_hidden` at `:227`. The triangle normal enters only `shadowed` at `:158-162`, where `facing = dot(triangle_normal, direction)` chooses whether the origin leaves the surface along the normal or starts inside it, and the second-surface plane test at `:417`.

ADR 0020, line 3: "One hard shadow ray per pixel is traced toward each directional light against the static TLAS, skipped where the light adds nothing (since [the shadow mask ticket], a light the surface's face turns from is hidden without one), from an origin offset along the surface's normal". ADR 0019, line 3, as amended 2026-10-28: "the depth passes also write each sample's triangle normal and the normal interpolated over it [...] which the shadow mask pass reads so that a coarse smooth-shaded mesh's terminator follows its interpolated normals rather than its triangles". The change ADR 0019 records moved the turn test from the face to the interpolated normal and left ADR 0020's parenthesis as written. The clause holds on its purpose, a light the surface turns from is hidden with no ray; the Finding is the word "face". Severity `drift`, not load-bearing, since the lit pass shades by the interpolated normal and the ADR 0019 sentence carries the decision; magnitude one sentence, read by anyone auditing the mask against ADR 0020 alone.

### Notes for the verifier

Not filed, since each needs a GPU run, an owner's reading, a registry addition or belongs to another lane; recorded so the next audit does not re-find them. By reader: A surface shading, B HDR and display, C geometry, culling, pyramid, Froxels and bend, D shadow mask.


**A.**

- The registry's `vulkan-spirv-precision` quote covers `OpFAdd`, `OpFMul`
  and `OpDot` only; every oracle chain here divides, and most take a root,
  and the Vulkan table's rows for those (`OpFDiv` 2.5 ULP, `inversesqrt`
  2 ULP, `sqrt` inherited from 1.0/inversesqrt) are not quoted. The floors
  above take them from the table itself, and `msl-4.1-fast-math`'s Table
  8.2 rows for division and roots are likewise unquoted. When the floors
  are written beside the constants, as the lane asks, those rows belong in
  the registry.
- Two assertions are tight only by an allowance the registry does not
  quote: the GGX peak at `TOLERANCE` 1e-6 absolute on 16/π ≈ 5.09, whose
  one division at 2.5 ulp is 1.2e-6; and `decode_surface`'s exact equality
  of 0.375 after `sqrt(sqrt(81/4096))`, which needs exact roots of perfect
  squares. Both pass on the M4 Pro (2e-7) and on lavapipe in CI, so those
  backends round the division and the roots correctly there; a conforming
  Vulkan driver need not. Loosening the peak check to `RELATIVE_TOLERANCE`
  and the roughness equality to one ulp would make them explained; an
  owner's call, not a defect.
- `TOLERANCE` 1e-6 absolute for `tangent_space_normal` (floor ≈ 6e-7) and
  `RELATIVE_TOLERANCE` 4e-6 for the BRDF match (floor ≈ 2.5e-6) are
  explained at round to nearest and tight under the round toward zero
  `msl-4.1-rounding` permits (1.2e-6 and 5e-6). The M4 Pro's measured
  2e-7 and 2e-6 show it rounds to nearest.
- The reciprocity assertion (`:262-269`) reuses `RELATIVE_TOLERANCE` where
  the swap's floor is about 2u (fused multiply-add asymmetry alone): loose
  by about 30×, on a shared constant whose name does not claim otherwise.
- `MINIMUM_ROUGHNESS = 0.04` (`shading.wgsl:22`): the source says "a small
  positive value" and the number is the engine's; ADR 0023 reasons from it
  (the 1.24×10⁸ cd/m² glint checks: D 1/(π α²) = 1.24e5, V 1/4, F₀ 0.04,
  E 1e5). Story 23's "no unsourced numbers" reads either way; the owner's
  call whether the value's reason (which roughness the floor renders as
  mirror-like, or the f32 limit of the Lagrange sum) is written down. The
  `1e-30` at `:209` is a guard against a zero half sum, not a number the
  image depends on.
- Candidates for `deviations.md` beyond L1-S02, if the owner reads them as
  deviations: the specular antialiasing widening (`shading.wgsl:39-49`,
  `:77-81`), whose image half is `DIFFERENCES.md`'s "Highlights widened by
  specular antialiasing and Toksvig", and the summation of D's denominator
  by Lagrange's identity, which is the same quantity in another expression
  and not a deviation. The ambient's form, E/π times base colour for metals
  too, is `DIFFERENCES.md`'s "Ambient light as an ideal diffuse response"
  and ADR 0020's stand-in; a code-side row would say the same.
- `main.wgsl:183-190` normalises the interpolated normal and the
  Gram-Schmidt tangent without a guard: a triangle whose vertex normals
  interpolate to zero, or whose tangent lies along the normal, gives NaN,
  which `min(…, LARGEST_HALF)` need not catch (WGSL leaves `min` with a NaN
  unspecified) and the HDR target stores. `fs_surface_normals` guards the
  same normal (`:61-66`). The cooker's MikkTSpace tangents and the
  Reference scenes' meshes give no such triangle, and no capture shows one;
  a Golden over authored content with hard-edged duplicated normals would
  settle whether a guard is wanted. Not the checklist's grazing-angle or
  zero-roughness case, which the shading terms handle.
- `masked`, `directional_hidden_share` and `froxel_mask_bit`
  (`main.wgsl:123-164`) have no oracle; they are exercised by the shadow
  Goldens and the equivalence captures (tooling). `froxel_mask_bit`'s
  binary search assumes each Froxel's list is in ascending light order,
  which `froxel.wgsl:3-4` promises (shaders C). The 32-bit shifts in
  `directional_hidden_share` for an index past 7 are discarded by the
  `select`, and WGSL masks a runtime shift amount, so no undefined value
  reaches the result.
- The lit pass's Froxel takes the screen size from
  `textureDimensions(shadow_mask)` (`main.wgsl:274`), which render sizes to
  the target and binds with rays off too; the shadow mask's Froxel
  rectangle is also the pixel centre's (`shadow_mask.wgsl:262`), so the
  slice is the only coordinate the two can disagree on, and the slice is
  what `main.wgsl:276` corrects.
- `lights[0]` is read when the light buffer holds no directional light and
  its intensity multiplied by zero (`main.wgsl:213-218`, `:240`); a NaN or
  infinity in a stale slot would survive. The buffer is zeroed at creation
  and prepare writes finite values (render); nothing to file.
- `shading.wgsl:107-110` and `:181-182` quote measurements in comments
  ("1.1% more light", "25 µs (5%)"): reasons for an expression's form, with
  no number the image depends on; `deviations.md`'s "no number may sit"
  concerns Deviations, so nothing filed.
- Story 20's equivalence rests on both paths summing in light order; the
  capture that proves it is tooling's.


**B.**

- PQ round trip, `tight` by the table's worst case (Oracles block): at
  frame-buffer 1, 2.5 and 10 the spec-bound floor is 4.0 to 5.4 × 10⁻⁴
  relative against a 3 × 10⁻⁴ tolerance. The test passes on the M4 Pro and
  lavapipe (CI is green at the frozen commit), so their `log2` is well
  inside 3 ULP or its errors cancel between the two nearby arguments. A
  run on the RTX 2070 SUPER under Vulkan and DX12 would say whether any
  backend approaches the bound; the `pow` of the ratio form would cut the
  worst case fivefold if one does. The performance lane would weigh the
  extra divide.
- `GT7_TOLERANCE` is `loose` by about 60× for `gt7_curve` and by 100× to
  300× for the ICtCp grey and chroma sub-assertions, all on a constant
  shared with PQ-bearing tests that explain its size. Per-test constants
  would make each floor legible; the reader is not misled today, since the
  comment states the measurements. L1-S04 covers the unit the comment
  misnames.
- The dither variance assertion's `1e-4` (`:1081`) has no reason beside
  it; the derived gap is the 256-level midpoint bias of −8.15 × 10⁻⁵, so
  the margin is 1.8 × 10⁻⁵ and the GPU's rounding adds under 10⁻⁶. A
  comment saying "within 1e-4: the 256 bin centres' discretisation bias is
  8.2e-5" would make it `explained`.
- The ICtCp round trip's comment credits "the sample's six-digit inverse
  matrix" for the `1e-3` margin; the six-digit matrices contribute about 4
  × 10⁻⁵ (their products deviate from identity by 3.3 × 10⁻⁶ and 1.7 ×
  10⁻⁶), and PQ's f32 encode-decode chain the rest.
- The weighted resolve's floor doubles to 3.4 × 10⁻⁶ relative under
  Metal's permitted round-toward-zero (`msl-4.1-rounding`), still under 4
  × 10⁻⁶; the stated 2 × 10⁻⁶ on the M4 Pro says it rounds to nearest in
  practice.
- A numerical property of the design, not a clause: the HDR target holds
  weighted colour, whose luminance is L/(1 + L) < 1, in fp16, whose step
  near 1 is 2⁻¹¹; undoing the weighting multiplies that relative step by
  (1 + L_c), so exposed luminance 1 carries 1 × 10⁻³ relative error after
  the resolve, 10 carries 5 × 10⁻³ and 100 carries 5 × 10⁻², all before
  GT7's shoulder compresses them. For greys at or past exposed 2 the curve
  saturates and the error is invisible; for saturated colours the ICtCp
  path carries it into the whitening. This is the survey's row 18 ("whose
  inverse caps at 2047") taken as decided; a capture would show whether it
  is ever a visible step. The weighting also means the fp16 store can never
  overflow (a channel caps at 1/0.0722 = 13.85), so ADR 0023's clamp guards
  the weighting against f32 infinity and the unweighting's pole, which the
  hdr part's comments could say.
- The dither of a clamped encoded value: at pure black and pure white the
  triangular noise exceeds half a code with probability 1/8 (derived over
  the levels), so 12.5% of exactly-black pixels write code 1 and 12.5% of
  exactly-white write 254; the other side clamps in the UNORM store. This
  is non-subtractive dither as Lipshitz describes it and `DIFFERENCES.md`
  lists the dither; whether black letterboxes or a clipped sky should be
  exempt is an owner's visual call.
- `tonemap` clips Rec. 2020 to Rec. 709 after GT7 (`display.wgsl:24`),
  which Polyphony's sample does not do; the survey's limitations list and
  the test's docstring say so, and the code comment says "clipped back into
  Rec. 709". Not a deviation from a quoted claim, so no row; noted so the
  Cycles lane reads a saturated-green hue shift as decided.
- `GT7_PAPER_WHITE`'s comment (`gt7.wgsl:30-31`) and the `tonemap` comment
  (`display.wgsl:19-21`) agree with the slides; the sample's chroma fade
  constants 0.98 and 1.16 and the 40/60 blend at `gt7.wgsl:120-122` are not
  in any registry quote, so their only evidence is the 26-row reference
  table from the compiled sample, which the test holds to 3 × 10⁻⁴.
- The sRGB knee is IEC's: 12.92 × 0.0031308 = 0.0404499 against the power
  branch's 0.0404499 at the same point, continuous to 3 × 10⁻⁸; the
  standard's constants, not a defect.
- The display ramp file has 348 rows, 6 colours × (1 + 57 quarter stops),
  as the test builds them, and its two header lines are the only `#` lines.
- The checklist's "sRGB encoded once" was confirmed by searching every WGSL
  part for `12.92`, `1.0 / 2.4`, `0.0031308` and `0.04045`: only
  `display.wgsl:33-35` holds them; the Rust side's assertion that the
  target is not an sRGB format is render's row.


**C.**

- `position_at`'s oracle constant is `tight` by the arithmetic in block
  B: the worst-case floor is 2.6 to 6.4 times the constant at u = 2⁻²⁴.
  The constant is chosen from downstream need, not the floor, and the
  test states no measured error. A run on lavapipe and on the RTX 2070
  SUPER, printing the largest error, would show the headroom; the floor
  suggests the constant could be loosened to 2⁻¹⁸ of the largest
  coordinate and still sit four times inside the shadow ray's 2⁻¹⁶
  offset. Not a Finding under the lane's rule (only `unexplained` and
  `loose` are).
- The precision-bar oracle (block D): the GPU's written order is under
  the bar at both unit roundoffs; the worst other order is over it only
  if the GPU both reassociates and rounds toward zero (5.3e-5 m against
  4.3e-5). The M4 Pro passes. A failure elsewhere would be the oracle
  doing its job; nothing to file.
- The octahedral round trip's constant is "4 × TOLERANCE" with no reason
  given for the 4 (`shader_function_tests.rs:1754`); block C shows the
  floor warrants it (1.8e-6 to 3.6e-6 against 4e-6). A one-line comment
  would make it `explained` outright.
- `culling.wgsl:174-175` grows the occlusion rectangle "by a pixel past
  where rasterization rounds vertices" without citing the bound it rests
  on; the registry's `vulkan-subpixel-precision-bits` (minimum 4 bits, a
  sixteenth of a pixel) is cited only at `shadow_mask.wgsl:39`. By
  arithmetic the margin holds: `pixel_margin = 0.5 / level_zero_size` in
  level-0 texture coordinates is one pixel where the depth target's side
  is even and slightly more where it is odd, and a sample a triangle
  covers lies inside its pixel, whose whole extent then lies inside the
  grown rectangle; f32's error on the corners' projection (≈ 2⁻²⁰ of a
  normalised coordinate, about a thousandth of a pixel) is far inside it. A citation beside
  the comment is a docs-lane fix.
- Froxel slice boundaries in f32: `froxel_slice` (`froxel_grid.wgsl:25-28`)
  may put a point within an ulp of a slice start into the neighbouring
  slice; the Froxel pass lists a light in both slices its sphere touches
  (`froxel.wgsl:52-53`, inclusive), so the only light a point could lose
  is one whose range exceeds the distance by less than that ulp, ≈ 1e-4 m
  at 1 km, where KHR_lights_punctual's window is ≈ 4e-7 of the light. The
  Golden equivalence allows zero pixels and holds. Not measurable, not
  filed.
- `MESHLET_MAX_TRIANGLES` is declared into every module (`mod.rs:240`)
  and no part reads it (grep over `src/render/shaders/*.wgsl`): a
  vestigial declaration, `health`, one line.
- `light_count.wgsl:19-27` `camera_relative_point` rebuilds a point from
  clip space by Cramer's rule, a second implementation of what
  `position_at` does under oracle; it runs only with the
  `count_point_lights` Render switch and has no oracle. Correct by
  inspection (the inverse of a 3×3 by cross products over its
  determinant). `health`: one function, one call site.
- The Froxel grid's functions (`froxel_grid.wgsl`) are not in the
  oracle harness's `SHARED` parts (`mod.rs:171-176`), so no oracle can
  call `froxel_slice` or `froxel_at`; they are covered by the renderer's
  Froxel test against an f64 assignment of every light to every Froxel
  (`src/render/renderer.rs:2022-2110`) and the brute-force Golden
  equivalence. An input for the tests lane if an oracle is wanted.
- `bend.wgsl:28-32` keeps `sin(angle) / angle` for |angle| ≥ 0.5, where
  the registry's bound (2⁻¹³ absolute on [−π, π]) would allow a chord error
  of 2 m · 2⁻¹³ / 0.6 ≈ 4e-4 m at the bend test's largest half angle, yet
  the bend pass test holds positions to 1e-5 m (`src/render/bend_pass.rs:276-281`,
  curvature −0.6 over 2 m): the M4 Pro's sine is far inside the vendor's
  bound. A GPU at the bound would fail that pass test, not the shader;
  the dynamic TLAS's box grows by 2⁻⁸ of the largest coordinate
  (`src/render/acceleration_structures.rs:444-451`), which covers it.
  Render's note on `BEND_ROUNDING_SHARE` stands.
- `var<workgroup> visible_words` (`culling.wgsl:64`) is never cleared in
  the shader; it relies on WGSL's zero-initialised workgroup memory,
  which wgpu 30's checked shader runtime default provides. Fine as is;
  a GPU run with runtime checks off would show stale bits.
- Phase one draws an Instance visible the frame before only if it is
  also in the frustum now (`culling.wgsl:259`), where ADR 0019 says
  "those visible the frame before": fewer draws, the same image; the
  ADR's sentence is a summary, not misled. Not filed.
- Greene's "finest-level sample ... covers the screen-space bounding
  box" (ADR 0019's citation) is refined here to the finest level where
  the rectangle spans at most two texels, reading up to four
  (`culling.wgsl:206-222`), a common tightening; the code does not cite
  Greene and nothing contradicts the quote. Not filed.
- The culling-off equivalence allows one differing pixel on lavapipe
  (`tests/goldens.rs:58-63`, a driver resolving one sample of `town`'s
  frame 1200 by which triangles the draw holds): a tooling-lane row, not
  a shader defect; on hardware zero pixels are allowed and hold.
- The occlusion test's `COMPOSITION_ERROR = 1e-4` (`culling.wgsl:159-165`)
  takes ADR 0014's 0.041 mm twice, for the box and the vertices, rounded
  up to 0.1 mm; the ADR's 0.041 includes 0.0225 mm of rotation at 128 m
  that the box's composition shares. Conservative; nothing to file.


**D.**

- Story 27 says a ray is skipped where "N·L ≤ 0"; the mask skips where the stored interpolated normal turns from the light by more than sin 2° (`shadow_mask.wgsl:72`, `:283`, `:311`), so a light at N·L in (−0.035, 0] gets a ray the lit pass then darkens anyway. The comment at `:61-71` justifies the margin by the 8-bit octahedral normals' error, "within 0.95° where the store rounds to the nearest step [...] and about twice that where it may round to either". That 0.95° figure cites no source and has no oracle; a CPU sweep of `octahedral_encode` and `octahedral_decode` over the sphere at 8 bits would give the test constant. The direction of the deviation is conservative: extra rays, none dropped. An owner's call whether the story's "no ray is wasted" is meant literally.
- The comment on `SHADOW_RAY_OFFSET_SCALE` (`:29-32`) says rebuilding the point, composing it and intersecting "each round a coordinate by a few times 2^-24 of its size at most". My count of `position_at` (`common.wgsl:49-56`) is two divisions, a 3-term dot, and about six further roundings, so nearer 12 × 2^-24 than "a few", and `segment_meets_box`'s cross axes reach about 24 × 2^-24 (block above); the offset's 256 covers both by an order of magnitude, so nothing turns on it. The same "a few times" wording is at `:54-55`.
- The offset's extent (`:153`) takes the largest coordinate in the TLAS's frame and relative to the camera. The hardware may compute the hit in the BLAS's object frame, whose coordinates reach 128 m from the pivot (ADR 0014), which neither extent includes; for a surface point within a metre of both the camera and the TLAS anchor's corner, on a Mesh whose pivot is far from it, the 1 m floor (15 µm) is the only cover against a hit error proportional to the object-frame coordinate. The Wächter-Binder quote speaks of one origin; which frame a given GPU's intersector rounds in is not in any quoted source. A GPU run on the shadow scene with the camera at the TLAS anchor's corner would show whether self-shadow speckle appears; no Reference scene places the camera there today.
- The oracle's two on-face cases ("ends on the face", "along a face") pass because their deciding comparisons are exact: the inexact half-length enters both sides multiplied by exactly 1 or 0 (block above). A case with a face at a non-dyadic coordinate, say 0.3, would compare two differently rounded values and could pass or fail by the backend's rounding. The doc comment at `shader_function_tests.rs:1856-1863` does not say the on-face cases depend on this; a reader extending the cases could add a flaky one. Not filed: the test as written is exact on every backend, so no reader of the current test is misled.
- `shadow_mask.wgsl:438` writes `abs(off_plane) > vec4<f32>(surface.pixel_width) & depths > vec4<f32>(0.0)`. In the WGSL specification's expression grammar the operands of `&` are unary expressions, so two relational expressions joined by `&` need parentheses; naga's parser accepts the form by precedence and both backends compile it today (the Goldens pass through naga on Metal and lavapipe). A portability note for the health lane, if my reading of the grammar is right; no effect on this commit.
- The second-surface threshold, one pixel's width off the nearest sample's plane (`:424`), and the turn margins sin 2°, cos 3° and sin 10° (`:72-78`) are the code's constants with their reasoning in comments and no source; the Lauritzen quote names the comparison, not a threshold. Judging them needs the image lane (the shadow scene and `crowd`'s silhouettes at native resolution).
- `trace_shadow_edges` includes every light the nearest sample turns from in `edge_lights` (`:647`), where `list_shadow_edges` listed the pixel only if another sample faces one of them (`:607`); the per-sample `directional_hidden` gives a turned sample the same verdict with no ray, so the result is equal and the cost is a few dot products. Health lane, if anything.
- Directional lights at entry 59 or past it are counted whether or not the surface turns from them (`:275-277`); point lights are counted only within range (`:300-307`). Consistent: a turned light needs its hidden bit, which the word cannot hold, while an out-of-range light needs none.
- The Froxel the mask's bits follow is found at the pixel's centre with the sample's view depth (`:261-262`), and the lit pass's at the fragment's own view depth at the same centre (`main.wgsl:273-276`); the slice bits and `froxel_mask_bit` reconcile them. Whether a Froxel list that differs between the two ever lacks a light the fragment sums, giving `UNMASKED`, is a counting question for the performance lane's `count_point_lights` switch.
- The module's line range for this letter, 1865-2139, ends inside shaders C's camera-relative precision helpers (`shader_function_tests.rs:2051-2139`); the one oracle in this letter ends at `:2049`.
- The analytic sphere-over-plane shadow oracle (story 34) and the equivalence of Froxel lighting with brute force (story 20) are tooling's, at the capture seam.

### Counts

| Severity class | Findings |
|---|---|
| `image` | 0 |
| `budget` | 0 |
| `correctness` | 1 (L1-S03) |
| `drift` | 5 (L1-S01, L1-S02, L1-S04, L1-S05, L1-S06) |
| `health` | 0 |

Clause verdicts: 151 rows; `holds` 118, two of them with a Finding on the
clause's record or wording (L1-S02 on the point light's range window,
L1-S06 on ADR 0020's "face"); `drift` 3 rows, one Finding (L1-S05: ADR
0014's sentence, read by readers A and C, and ADR 0019's); `broken` 0;
`superseded` 0; deferred by the spec 1; falls in another letter or in
render 20; not applicable 9. Spec stories: 43 rows, all holding in the
module or falling in tooling at the capture seam. Registry keys: the 32
keys the module's parts cite, all match, one in form with a Finding on
its missing Deviation row (`peters-blue-noise`, L1-S03); the keys opened
for the floors match. Oracles: 28 tests in 31 rows; `explained`
21, `tight` 5 (the GGX peak, the direction cases, the surface decode's
roughness root, PQ's round trip, `position_at`), `loose` 4 (the BRDF's
reciprocity half, the point light test, and two users of the shared
`GT7_TOLERANCE`), `unexplained` 1 (the dither variance's sub-assertion), a
mixed row counted by its sterner verdict; the two oracle Findings are
L1-S01 and L1-S04.

## tooling

Scope: `src/bench/` with `history/` (`mod.rs`, `report.rs`,
`provenance.rs`, `history.rs`, `history/verdicts.rs`, `history/import.rs`,
`history/suspects.rs`), `src/capture/mod.rs`, `src/preflight/mod.rs`,
`src/stats/mod.rs`, `src/gpu/mod.rs`; 9,019 lines of Rust in 11 files,
every file read in full; and `scripts/verify.sh`, `gallery.sh`,
`sweep.sh`, `sweep.plist`, `sweep-task.xml`, `pr-lint.sh`,
`pr-lint-test.sh`, `scripts/cycles-reference/` (`render.py`,
`compare.py`, `calibrate.py`, `DIFFERENCES.md`, `display-ramp.txt`),
`.github/workflows/ci.yml`, `pr-lint.yml` and
`.github/pull_request_template.md`; 2,281 lines of text, every one read.
Three readers at high effort shared the module by file, each with the
lane brief verbatim: bench with its history; capture, preflight, stats and
the GPU context; the scripts, the Cycles reference and the workflows. ADRs
read: 0013 and 0024 clause by clause by every reader, and 0004, 0016,
0021 and 0023 where a file implements one of their clauses. The
`CLAUDE.md` review checklist and `docs/agents/workflow.md`'s Session
start, PRs, Review, Performance and memory and Correctness sections were
read as the operational form of what the scripts and preflight must do.
`bench/budgets.json`, one Baseline per adapter, the `bench`, `compare`,
`report`, `publish`, `history`, `capture` and `preflight` dispatch in
`src/bin/praxis.rs`, and the Goldens test were opened only to confirm what
the module must deliver. Registry quotes from wgpu crates were confirmed
against wgpu-hal, wgpu-core and wgpu-types 30.0.1 and gpu-allocator 0.28.0
in the cargo registry; the repository's milestones were listed read-only.
Every `where` is `path:line` at the frozen commit. The Capability floor's
format list, already filed by the render reader as L1-R01, is marked in
the clause table and not re-filed. `deviations.md` holds no row at this
commit; `DIFFERENCES.md` was read beside the Cycles reference. The
readers' three scope statements precede the Counts, so the verifier knows
which files and sources each opened.

### Clause verdicts

| ADR | Clause (quoted, shortened) | Verdict | Where |
|---|---|---|---|
| 0013 | "60 fps at 1440p native with CPU at most 6 ms and GPU at most 12 ms per frame" | holds (median): `REFERENCE` is 2560x1440, `SIXTY_HERTZ`, Budgets 6000/12000, judged at the median | `src/bench/mod.rs:55-61`, `:454`, `:691-696`, `:716-721`; `bench/budgets.json:2-3` |
| 0013 | CPU judged "at the median and p95" (story 55; ADR re-decision quotes town "5.80 ms at the median and 6.21 ms at p95") | broken: L1-T01 | `src/bench/mod.rs:683-696` |
| 0013 | "the 6 ms bounds the wall time of each pipeline stage's critical path, waits on worker threads included" | falls in stats (`cpu_frame_time` producer); bench reads it at `src/bench/mod.rs:542-547` | — |
| 0013 | "Input latency at most 2 frame periods ... judged at p95" | not implemented: no bench metric, no `budgets.json` key (see Notes) | `bench/budgets.json:1-8` |
| 0013 | "GPU memory at most 8 GB at 1440p, counted as the bytes of the buffers and textures the engine has created, summed by the engine itself" | holds (bench half): `gpu_memory_bytes` median against the Budget in both modes; the sum falls in stats and render | `src/bench/mod.rs:227`, `:654-659`; `bench/budgets.json:7` |
| 0013 | "the GPU stats readback's buffers are left out of that sum" | falls in stats (readback) | — |
| 0013 | "acceleration structures join the count" | falls in render | — |
| 0013 | "the 8 GB splits into Budgets for geometry arenas, the texture pool, ... whose sizes M4 sets" | not applicable yet (one `gpu_memory_bytes` Budget; M4 sets the split) | `bench/budgets.json:7` |
| 0013 | "the dynamic TLAS's ceilings of 1.1 ms of CPU and 1.0 ms of GPU" | holds: 1100/1000, judged at the median | `bench/budgets.json:5-6`; `src/bench/mod.rs:740-753` |
| 0013 | "Budgets are judged on an Apple M4 Pro" | holds on the M4 Pro; `compare` also applies them to every other adapter's Local run (Notes) | `src/bench/mod.rs:634-755` |
| 0013 | "what `town` breaches is ticketed as an offender, not budgeted away" | holds: Budgets unchanged; town's Baseline (GPU 19,935 µs, gate 8,612 µs) fails `compare` by design | `bench/baselines/apple-m4-pro/town.json:18`, `:22`; `src/bench/mod.rs:716-721`, `:734-739` |
| 0013 | "A budget moves down freely and up only through an ADR or a map comment" | not applicable (process) | — |
| 0024 | "keyed by source hash × adapter × backend × scene × Harness version" | holds: path `records/<adapter>-<backend>/<scene>/<source hash>.json`; the Harness version is a constant under `src/`, so a source hash names one (comment says so) | `src/bench/history.rs:162`, `:205-213`; `src/bench/provenance.rs:21` |
| 0024 | "The source hash covers `src/`, `assets/`, `Cargo.toml`, `Cargo.lock` and `.cargo/`, a list held in one place in code, and a test fails when a new top-level path is classified neither way" | holds | `src/bench/provenance.rs:26`, `:225-242`, `:295-329` |
| 0024 | "A commit whose source hash didn't change inherits the record before it" | holds: `mktree` over `ls-tree` of the measured paths; tested | `src/bench/provenance.rs:225-242`, `:358-370` |
| 0024 | "When two records share a key, the quieter one stays: the one with the lower GPU p95/median" | broken (drift): L1-T04; the ratio rule itself holds for like against like | `src/bench/history.rs:952-959`, `:962-981`, `:986-988` |
| 0024 | "A record joins a Series only if its tree was clean, its profile release, the machine quiet (preflight warned nothing, and p95/median below about 1.06), and its source `main`'s" | holds: tags `dirty tree`, `source not main's`, `<profile> profile`, `machine not quiet`, configuration; quiet is `ratio < 1.06` with GPU load ≤ 10 %, no other praxis process, no power or display warning (the assets warning is not read: Notes) | `src/bench/history.rs:808-845`; `src/bench/provenance.rs:30`, `:102-103`, `:130-140` |
| 0024 | "lavapipe's records are the exception to the quiet test ... a record from CI's configuration joins them whether or not the runner was quiet, and the first record on a Series under a key stays" | holds: `!lavapipe && !quiet`, `CI_COUNTERS` as its configuration, `publishes_once`; lavapipe rows never Suspects | `src/bench/history.rs:161`, `:822-831`, `:350-355`, `:966-967` |
| 0024 | "Every other record is kept, tagged with its reasons, and drawn only on request" | holds (kept under `tagged/` with `tags`); drawing falls outside tooling (`docs/bench`) | `src/bench/history.rs:167-169`, `:944-951` |
| 0024 | "An Experiment's records are kept under its name and never join a Series" | holds | `src/bench/history.rs:184-203`, `:944-951`, `:525-541` |
| 0024 | "The Harness version rises whenever a change alters what a metric means, and each record carries its scene's content hash. A chart breaks its line where either changes" | holds: `HARNESS_VERSION = 10` = last imported era (9 breaks + 1); `scene_content_hash` from `render_frames`; Series lines break on `same_definition` (the chart falls outside tooling) | `src/bench/provenance.rs:21`, `:109`; `src/bench/history/import.rs:29-66`, `:97`; `src/bench/history.rs:1144-1148` |
| 0024 | "The 61 commits that touched `bench/baselines/` before the history existed are imported as records, each with the Harness version in force at its commit" | holds in mechanism (first-parent log before `3c6b9d92`, version stepped at each break); the count is 64, not 61: L1-T03 | `src/bench/history/import.rs:23`, `:141-173` |
| 0024 | "Publishing never blocks work. A failed publish warns and the bench still succeeds" | holds | `src/bin/praxis.rs:467-481` |
| 0024 | "`verify` never publishes" | holds: `--no-publish` | `scripts/verify.sh:52`; `src/bin/praxis.rs:190` |
| 0024 | "Baselines are still chosen by hand, and a re-recorded Baseline's record is marked as one" | broken: L1-T02 (only imported Baselines are marked) | `src/bench/history.rs:356-363`; `src/bench/history/import.rs:69`, `:277` |
| 0024 | "The history observes and never gates: Baselines stay in `bench/baselines/` on `main`, and `verify` and `compare` judge against them as before" | holds: `compare` takes the Baseline the binary reads from `bench/baselines/<adapter key>/`; no history type reaches it | `src/bench/mod.rs:32`, `:634-755`; `src/bin/praxis.rs:597`, `:701-704` |
| 0024 | "A timing record far from its Series' recent median is a suspect" | holds: window of 8, judged from the third, ± max(4 MAD, 3 %), CPU and GPU frame medians | `src/bench/history/suspects.rs:7-19`, `:50-104`; `src/bench/history.rs:390-397` |
| 0024 | "The sweep re-benches it, bisects the commits behind it, and files a confirmed Regression that no re-recorded Baseline covers as one `needs-triage` issue" | holds: interleaved ends, `holds` at max(shift/2, 3 %), first-parent bisect by source hash, `covered_by_baseline`, one issue per scene and commit with label `needs-triage` | `src/bench/history/verdicts.rs:222-313`, `:424-436`, `:444`, `:451-498`, `:757-772`, `:795-797` |
| 0024 | "Records publish themselves: ... `praxis bench` by default in sessions, and a nightly sweep ... benches `main` only when its source has no quiet record" | holds (bench half): publish is the default; `unrecorded_scenes` looks under `records/` for `main`'s source hash; the sweep script falls in scripts | `src/bin/praxis.rs:190`, `:467-470`; `src/bench/history.rs:595-621` |
| 0024 | "The branch holds only data and a README, is never merged, and CI writes it only from pushes to `main`" | holds for the README and data (every publish rewrites `README.md`); CI falls in workflows | `src/bench/history.rs:997-998`; `src/bench/history/verdicts.rs:667` |
| 0024 | "A bad publish is undone with `git revert` on `bench-history`" | not applicable (process); every publish is one commit | `src/bench/history.rs:642-681` |
| 0024 | "A page in `docs/` draws the Series" | falls outside tooling (`docs/bench`) | — |
| 0013 | "Budgets are judged on an Apple M4 Pro" | holds (classification); the judging falls in bench | `src/preflight/mod.rs:23-25, 38` |
| 0013 | "CPU at most 6 ms and GPU at most 12 ms per frame ... median and p95" | falls in bench; the measured fields are here | `src/stats/mod.rs:14-15, 77-80` |
| 0013 | "CPU time summed across threads is measured and reported but not budgeted" | not applicable: only the frame's thread is timed, and no system runs on the pool (ADR 0016 line 3) | `src/stats/mod.rs:11-13, 419-422` |
| 0013 | "Input latency at most 2 frame periods ... judged at p95" | falls in bench and input | — |
| 0013 | "GPU memory ... counted as the bytes of the buffers and textures the engine has created, summed by the engine itself on every backend" | holds at the seam; the sum falls in render; drift: L1-T07 (the capture target is left out) | `src/stats/mod.rs:39-45, 182-200`; `src/state.rs:302` |
| 0013 | "the GPU stats readback's buffers are left out of that sum" | holds at the seam (documented); the exclusion falls in render | `src/stats/mod.rs:42-44` |
| 0013 | "wgpu's allocator report and Metal's `currentAllocatedSize` are diagnostics only" | holds: an `Option` printed, never budgeted; `currentAllocatedSize` is read nowhere | `src/stats/mod.rs:53-55, 169-178` |
| 0013 | "acceleration structures join the count (ADR 0017)" | holds at the seam ("Of `gpu_memory_bytes`"); the estimate falls in render | `src/stats/mod.rs:46-49` |
| 0013 | "Budgets were re-decided against that first Baseline ... each stays as written" | falls in bench | — |
| 0021 | "requires a fixed set of wgpu features, downlevel flags and limits ... fails with every gap listed ... nothing in the floor has a fallback" | holds | `src/gpu/mod.rs:55-65, 314-347` |
| 0021 | "The floor holds only what an ADR or shipped code uses, drawn from Tiers 1 and 2" | holds: every feature requested is in matrix Tier 1 or 2 (`docs/research/wgpu-capability-matrix.md:195-196`) | `src/gpu/mod.rs:261-272` |
| 0021 | "`GpuContext` checks the adapter against the floor, then requests exactly its features, limits and experimental opt-in" | holds | `src/gpu/mod.rs:60-78` |
| 0021 | "Required features: `TEXTURE_BINDING_ARRAY`, `PARTIALLY_BOUND_BINDING_ARRAY`, `..._NON_UNIFORM_INDEXING`, `TEXTURE_COMPRESSION_BC`, `EXPERIMENTAL_RAY_QUERY`, `TIMESTAMP_QUERY`" | holds | `src/gpu/mod.rs:263-271` |
| 0021 | "4x MSAA on every attachment format the main pass uses ... with its resolve on the colour formats ... Those formats are `Depth32Float` and ... `Rgba16Float`" | broken: L1-R01 (render reader; the code matches the ADR's two-format list, not the main pass's attachments) | `src/gpu/mod.rs:256-257, 352-359` |
| 0021 | "The `ANISOTROPIC_FILTERING` downlevel flag is required too" | holds | `src/gpu/mod.rs:277-281` |
| 0021 | "So is `VERTEX_STORAGE`" | holds | `src/gpu/mod.rs:282-284` |
| 0021 | "So are `COMPUTE_SHADERS` and `INDIRECT_EXECUTION`" | holds | `src/gpu/mod.rs:285-288` |
| 0021 | "`INDIRECT_FIRST_INSTANCE` ... left the floor" | holds: absent from the floor, reported only | `src/gpu/mod.rs:187-191, 261-272` |
| 0021 | "`TIMESTAMP_QUERY_INSIDE_ENCODERS` and `_INSIDE_PASSES` stay optional" | holds: absent from the floor, reported only | `src/gpu/mod.rs:173-181, 261-272` |
| 0021 | "Ray queries need `ExperimentalFeatures::enabled()`, an `unsafe` call ... the one `unsafe` this ADR adds, and its comment cites ADR 0020" | holds: the file's one `unsafe`, comment names ADR 0020 | `src/gpu/mod.rs:71-77` |
| 0021 | "Required limits are `Limits::default()` with `using_minimum_supported_acceleration_structure_values()` ... `max_buffer_size` at least 2^31 − 1, `max_binding_array_elements_per_shader_stage` at least 65,536, `max_acceleration_structures_per_shader_stage` at least 2, `max_storage_buffers_per_shader_stage` at least 9" | holds | `src/gpu/mod.rs:297-309` |
| 0021 | "`max_storage_buffer_binding_size` stays at the default 128 MiB ... that Arena stays within it, as a test checks" | holds: not overridden (wgpu-types `limits.rs:441` is `128 << 20`); test present | `src/gpu/mod.rs:307, 757-761` |
| 0021 | "returns one typed error naming each missing feature and downlevel flag, each attachment format without 4x MSAA or ... its resolve, and each limit with its required and adapter values" | holds (`fatal: false` collects every limit gap, wgpu-types `limits.rs:875-889`) | `src/gpu/mod.rs:320-346, 364-414` |
| 0021 | "the binary logs it with `tracing::error!` and exits non-zero" | holds | `src/bin/praxis.rs:25-28` |
| 0021 | "`praxis capabilities` runs without the floor" | holds: no device requested | `src/gpu/mod.rs:111-118` |
| 0021 | "Headless contexts, tests and Goldens included, use the same floor" | holds: `headless` goes through `request_device` | `src/gpu/mod.rs:31-35` |
| 0021 | "The full floor ... enforced in M1 ... CI proves it on lavapipe" | falls in workflows (tooling, scripts reader) | — |
| 0016 | "Only the main thread submits, presents or polls the wgpu `Queue` and `Device`" | holds for capture: submit, poll and map on the calling thread; each Goldens capture owns its device (`tests/goldens.rs:266-278, 287`) | `src/capture/mod.rs:246-253, 261, 348-354` |
| 0016 | "runs ... the simulation schedule, prepare and command recording, then submits ... no stage runs a frame behind another" | holds: per drawn frame, `update` then `render` | `src/capture/mod.rs:256-261`; `src/state.rs:254-285` |
| 0016 | "`desired_maximum_frame_latency` stays 2" | holds at capture: in flight bounded by `FRAMES_IN_FLIGHT`; the constant falls in render | `src/capture/mod.rs:228, 239-255` |
| 0016 | "Frame data ... is an owned, `Send` value with no borrow into the ECS World" | falls in render | `src/render/prepare.rs:47-50` |
| 0004 | "every Frame data must reach the renderer, in order" | holds: drawn frames ascending, each prepared then rendered; undrawn frames run the simulation only, and prepare's change window spans from one `prepare` to the next (`clear_trackers` at its end), so no change is skipped | `src/capture/mod.rs:230-261`; `src/render/prepare.rs:259-263, 470-472` |
| 0004 | "prepare ... builds Frame data ... the renderer consumes only that" | falls in render; capture drives `update` then `render` | `src/state.rs:254-285` |
| 0023 | "the tonemapping pass then maps it through GT7 for display" (and the checklist: sRGB encoded once, into 8-bit UNORM, no sRGB target) | holds for capture: `Rgba8Unorm` target, surface refuses sRGB formats, PNG written as raw RGBA8 | `src/capture/mod.rs:9-11, 220`; `src/gpu/mod.rs:450-470`; `src/bin/praxis.rs:768-775` |
| 0023 | "each scene has an EV100, which the lit pass applies before the `Rgba16Float` HDR target stores its colour, clamped to 65504" | falls in shaders | — |
| 0024 | "the machine quiet (preflight warned nothing, and p95/median below about 1.06)" | drift: L1-T06 (the quiet verdict reads four of preflight's five warnings) | `src/preflight/mod.rs:242-262, 349-356`; `src/bench/provenance.rs:126-139` |
| 0024 | "Only clean, release-profile records of `main`'s source taken by a quiet machine form a Series" | falls in bench | — |
| 0024 | "`praxis bench` by default in sessions ... `verify` never publishes ... a nightly sweep" | falls in bench and scripts | — |
| workflow, Session start | "prints ... in ten lines or fewer: the adapter and which machine it is, whether its Goldens, Baselines, fetched files and Cooked cache are present, GPU load and other praxis processes, the open PRs and the live milestone's assigned tickets" | broken: L1-T05 (the tickets line); the other nine clauses hold, 9 lines on macOS and Windows, 8 on Linux | `src/preflight/mod.rs:102-136, 561-618` |
| workflow, Session start | "A `FAIL` line blocks `scripts/verify.sh` and names its fix" | holds for adapter, goldens, baselines; whether each blocks `verify.sh` falls to the scripts reader | `src/preflight/mod.rs:127-131, 184-194, 218-230` |
| 0024 | "Records publish themselves: CI's lavapipe on every push to `main`" | holds (docs-only pushes excepted by `paths-ignore`; their source hash is unchanged, ADR 0024 "inherits the record before it") | `.github/workflows/ci.yml:9-11, 214-217` |
| 0024 | "a nightly sweep on the reference machine and the RTX 2070 SUPER" | holds | `scripts/sweep.plist:17-23`, `scripts/sweep-task.xml:9-15`, `scripts/sweep.sh:18-35` |
| 0024 | "benches `main` only when its source has no quiet record" | holds in the script; the record test falls in `src/bench/history/` | `scripts/sweep.sh:80, 195-199` |
| 0024 | "and the machine is quiet" | holds | `scripts/sweep.sh:85-92, 102-122` |
| 0024 | "Only clean, release-profile records ... form a Series" | holds for the build profile; the clean and quiet stamps fall in `src/bench/provenance.rs` | `.github/workflows/ci.yml:168`, `scripts/sweep.sh:78, 135` |
| 0024 | "`verify` never publishes" | holds | `scripts/verify.sh:52` |
| 0024 | "A failed publish warns and the bench still succeeds" | falls in `src/bench/`; the sweep logs and goes on | `scripts/sweep.sh:183` |
| 0024 | "The sweep re-benches it, bisects the commits behind it" | broken: L1-T08 | `scripts/sweep.sh:154-175` |
| 0024 | "files a confirmed Regression that no re-recorded Baseline covers as one `needs-triage` issue" | holds in the script; the issue falls in `src/bench/history/verdicts.rs:444-553` | `scripts/sweep.sh:191-193` |
| 0024 | "CI writes it only from pushes to `main`, never from a pull request" | holds | `.github/workflows/ci.yml:20-21, 216, 220-221` |
| 0024 | "A local scheduled task runs no code from a pull request" | holds | `scripts/sweep.sh:70-72` |
| 0024 | "The branch holds only data and a README, is never merged" | falls in `src/bench/` (publish) | `.github/workflows/ci.yml:237-238` |
| 0024 | "lavapipe's records are the exception to the quiet test" | falls in `src/bench/history/` | `.github/workflows/ci.yml:210-213` |
| 0024 (spec decision) | "`publish-bench` publishes lavapipe's records and the release build's size and wall time on every push to `main`; PR runs publish nothing" | holds | `.github/workflows/ci.yml:165-169, 198-208, 214-238` |
| 0024 (spec decision) | "`praxis bench` publishes ... unless given `--no-publish`" | falls in `src/bench/`; every script bench passes `--no-publish` and publishes later or never | `scripts/verify.sh:52`, `scripts/sweep.sh:171, 199`, `.github/workflows/ci.yml:178` |
| 0024 (spec decision) | "each run stamped quiet (load average, `pmset -g batt`, no peer benching)" | falls in `src/preflight/` (`mod.rs:358-372` reads `pmset`) | `scripts/sweep.sh:85-92` |
| 0013 | "`verify` and `compare` judge against them [Baselines] as before" | holds, counters only | `scripts/verify.sh:28-53` |
| 0013 | "Budgets are judged on an Apple M4 Pro" | holds: CI judges counters alone, "lavapipe's times mean nothing" | `.github/workflows/ci.yml:171-181` |
| 0013 | "At M3's exit the Budgets were re-decided ... each stays as written" | falls in `src/bench/` (`bench/budgets.json`) | ADR 0013:3 |
| 0023 | "`DirectionalLight` gives illuminance in lux and a `PointLight` luminous intensity in candela" | holds | `scripts/cycles-reference/render.py:28-30, 95-135`, checked by `calibrate.py:67-77` |
| 0023 | "emissive and background give luminance in cd/m²" | holds | `scripts/cycles-reference/render.py:147-148, 342-344` |
| 0023 | "a Reference scene's ambient term gives illuminance in lux" | holds (world radiance E/π) | `scripts/cycles-reference/render.py:138-146`, `calibrate.py:76` |
| 0023 | "Exposure is an EV100, which scales luminance by 1/(1.2 · 2^EV100)" | holds | `scripts/cycles-reference/render.py:381, 397-404, 344` |
| 0023 | "Point lights fall off by inverse square inside KHR_lights_punctual's range window" | holds | `scripts/cycles-reference/render.py:95-123`, `calibrate.py:72-74` |
| 0023 | "the tonemapping pass then maps it through GT7 for display" (the port), sRGB encoded once | holds: the port's constants equal the registry quote and the GPU ramp within an 8-bit step | `scripts/cycles-reference/compare.py:54-56, 84, 97, 104, 126-148` |
| 0023 | "clamped to 65504" | not applicable: the reference is float EXR and f64 numpy | `scripts/cycles-reference/render.py:83-84` |
| 0004, 0016, 0021 | capture's frame drive, who submits and polls, the Capability floor | not applicable: no clause lands in `scripts/` or the workflows | — |

### Spec stories

| Story | Verdict | Where |
|---|---|---|
| 9 (Instances and triangles drawn after culling, read back from the GPU) | holds (bench half): counters read `stats.gpu` | `src/bench/mod.rs:221-222` |
| 35 (dynamic TLAS at 10,000 Moving Instances kept as CPU and GPU ceilings) | holds: 1100/1000 µs ceilings judged at the median | `bench/budgets.json:5-6`; `src/bench/mod.rs:740-753` |
| 45, 48 (town at target scale; CI bench shortened to counters) | holds: `CI_COUNTERS`, `bench --ci-counters`, `compare --ci` reads counters only | `src/bench/mod.rs:87-92`, `:661-663`; `src/bin/praxis.rs:271`, `:303` |
| 50 (crowd with a Baseline, no Golden) | holds (Baseline half) | `bench/baselines/apple-m4-pro/crowd.json`; Golden absence falls in capture |
| 51 (GPU time per pass and a GPU frame total in every Baseline) | holds | `src/bench/mod.rs:164-174`, `:519-528`, `:560-565`; `bench/baselines/apple-m4-pro/town.json:17-78` |
| 52 (gate from first culling dispatch to end of lit pass, rays off, median over the path) | holds (bench half): gate from the rays-off run, median; the span falls in stats | `src/bench/mod.rs:566-571` |
| 53 (shadow-ray time as its own line, rays on minus rays off) | holds | `src/bench/mod.rs:572-580`; `src/bench/report.rs:83-88` |
| 55 (town within 12 ms GPU and 6 ms CPU, at the median and p95 for CPU) | broken: L1-T01 (CPU p95 never judged) | `src/bench/mod.rs:683-696` |
| 56 (Budgets re-decided against town's first Baseline) | holds: Budgets unchanged, ADR amended | `bench/budgets.json:1-8` |
| 57 (crowd's AS update as BLAS refit and as proxies, against 1.0 ms GPU and 0.5 ms CPU) | holds in tooling (`--crowd-form` benches `crowd-<form>`); the 0.5 ms CPU ceiling is not in `budgets.json` (Notes) | `src/bin/praxis.rs:454-461`, `:610-613`; `src/bench/mod.rs:552-556` |
| 58 (render switches that only capture and bench pass) | holds | `src/bench/mod.rs:426-464`; `src/bin/praxis.rs:210` |
| 59 (warm verify under 60 s) | needs a run (Notes) | `scripts/verify.sh` |
| M3.5: `praxis bench` publishes unless `--no-publish`; `verify` never publishes; runs stamped quiet | holds | `src/bin/praxis.rs:190`, `:467-481`; `scripts/verify.sh:52`; `src/bench/provenance.rs:52`, `:113-118` |
| 3, 8, 14, 16, 20, 34, 36, 42 | fall in capture, cook, render (none of these files bears on them) | — |
| 3 (`town` the same with culling off at eight frames) | holds at capture: `capture_frames` renders the eight frames with the frame before each, as the Goldens test asks with culling on and off; 0 differing pixels allowed on hardware, 1 on lavapipe | `src/capture/mod.rs:138-172`; `tests/goldens.rs:58-71, 140-148, 169-175, 299` |
| 9 (Instances and triangles drawn, read back from the GPU) | holds at the seam: fields and their printed line; the readback falls in render | `src/stats/mod.rs:84-86, 132-142` |
| 14 (every Golden with 4x MSAA) | falls in render; the capture target is the single-sample tonemapped output, as ADR 0021 line 14 says | `src/capture/mod.rs:210-223` |
| 16 (the MSAA PR shows a 1-sample render failing) | not in code: evidence is that PR's image; needs the owner's record | — |
| 20 (Froxel-culled lighting matches brute force within Golden tolerance) | holds at capture: `froxel_lights` and `shadow_rays` switches reach `State::new`; the test allows 0 pixels | `src/capture/mod.rs:201-209`; `tests/goldens.rs:150-162, 176-182` |
| 34 (analytic shadow oracle) | falls in render (`tests/shadows.rs`); capture only supplies pixels | — |
| 36 (acceleration-structure bytes labelled as estimates) | holds: field named and printed as estimated | `src/stats/mod.rs:46-49, 172-178` |
| 42 (Instance buffer fixed at 524,288) | falls in render; refusals are counted here | `src/stats/mod.rs:26-27` |
| 51 (GPU time per pass and a GPU frame total) | holds at the seam; the Baseline falls in bench | `src/stats/mod.rs:72-80` |
| 52 (gate from the first culling dispatch to the end of the lit pass) | holds at the seam ("from the start of culling phase one to the latest end of the passes up to the lit pass"); the timing falls in render | `src/stats/mod.rs:81-83` |
| 58 (render switches only capture and bench pass) | holds: `RenderSwitches` is a parameter of `capture`, `capture_frames` and `render_frames`; the binary's capture sets `count_point_lights` and `crowd_form` only, the Goldens test the rest | `src/capture/mod.rs:114, 145, 192`; `src/bin/praxis.rs:744-748`; `tests/goldens.rs:140-162` |
| 3 (`town` the same with culling off at eight frames) | touches: the gallery captures `town` at the Goldens test's eight frames, equal to `tests/goldens.rs:299`; the equivalence check falls in `tests/goldens.rs` | `scripts/gallery.sh:33, 111-113` |
| 45 (`town` a dense Reference scene) | holds: benched by verify and CI as a Baselined scene | `scripts/verify.sh:49`, `.github/workflows/ci.yml:178`, `bench/baselines/*/town.json` |
| 48 (`town` Goldens on software Vulkan, CI bench shortened to counters) | holds | `.github/workflows/ci.yml:57-91, 93-142, 171-181` |
| 50 (`crowd` with a Baseline and no Golden) | holds: Baselined on every adapter, captured in two forms, benched by verify | `bench/baselines/*/crowd.json`, `scripts/gallery.sh:118-120`, `scripts/verify.sh:49` |
| 56 (Budgets re-decided at the exit) | holds in the ADR's text; the values fall in `src/bench/` | ADR 0013:3 |
| 58 (render switches only capture and bench pass) | holds: the gallery uses `--crowd-form` on capture; the limit falls in `src/bin/praxis.rs:319-320` | `scripts/gallery.sh:119` |
| 59 (warm verify under 60 s) | needs a run: `verify.sh` prints its wall time and asserts no bound; the aliasing skip is its one speed lever, and L1-T11 narrows it | `scripts/verify.sh:34-43, 78` |

### Registry keys

Every key cited in the module was read against its registry quote, and the wgpu-hal, wgpu-core and wgpu-types 30.0.1 and gpu-allocator 0.28.0 sources in the cargo registry where a quote is load-bearing. All 21 match. By file:

| File | Keys | Verdict |
|---|---|---|
| `src/bench/mod.rs:284` | `khronos-dfs-srgb-eotf` | matches: registry quote "R = R′ / 12.92, R′ ≤ 0.04045; ((R′ + 0.055) / 1.055)^2.4, R′ > 0.04045" (`docs/references.toml:224-228`); code `encoded <= 0.04045 → encoded / 12.92`, else `((encoded + 0.055) / 1.055).powf(2.4)` (`src/bench/mod.rs:287-291`), knee constant and the ≤ at the knee as quoted; the decode-table test's expected 0.0908 for 85/255 recomputes to 0.0908 |
| `src/capture/mod.rs:286` | `apple-tech-talk-110147-busy-waits` | matches: the quote ("Avoid busy waits ... lock a P core ... thermal headroom") is the reason the spin wait is confined to the bench on a hardware adapter; `capture` and `capture_frames` pass `FrameWait::Block` (`:127, 169`) |
| `src/stats/mod.rs:54` | `wgpu-hal-30-allocator-report-default` | matches: wgpu-hal `lib.rs:1210-1212` returns `None` by default; the field is an `Option` |
| `src/preflight/mod.rs:411` | `microsoft-query-user-notification-state` | matches: values 2 and 3 warn, 1 to 7 pass, else a note (`:412-426`) |
| `src/gpu/mod.rs:5` | `wgpu-hal-30-manual-memory-hints` | matches: wgpu-hal `lib.rs:444-473` read; `Performance` is 128 to 256 MiB, `Manual` clamps to 4 to 256 MiB, and 16 MiB..16 MiB sits inside (`:19, 79-82`) |
| `src/gpu/mod.rs:7` | `gpu-allocator-0.28-personal-block` | matches: `vulkan/mod.rs:484-488` gives a block of its own to `size > memblock_size`; `:667-673` destroys it when empty |
| `src/gpu/mod.rs:15` | `wgpu-hal-30-vulkan-texture-allocation` | matches: `vulkan/device.rs:1192-1205` allocates every texture `GpuAllocatorManaged`, never dedicated |
| `src/gpu/mod.rs:17` | `nvidia-vulkan-dos-donts-dedicated-memory` | matches the comment as quoted in the registry (web source, not re-fetched) |
| `src/gpu/mod.rs:74` | `wgpu-types-30-experimental-features-unsafe` | matches: `tokens.rs:29-37`, `pub const unsafe fn enabled()` |
| `src/gpu/mod.rs:280` | `wgpu-core-30-anisotropy-clamp` | matches: `device/resource.rs:2318-2328` sets 1 without the flag |
| `src/gpu/mod.rs:449` | `wgpu-hal-30-vulkan-downlevel-flags` | matches: `vulkan/adapter.rs:684-707`; `SURFACE_VIEW_FORMATS` needs `swapchain_mutable_format` (or no swapchain extension, moot for a presenting device; see Notes) |
| `src/gpu/mod.rs:477` | `wgpu-core-30-backend-order` | matches: `instance.rs:83-88` adds Vulkan, Metal, Dx12 in that order; `BACKEND_ORDER` (`:479-483`) is the same |
| `src/gpu/mod.rs:579` | `wgpu-hal-30-metal-downlevel-flags` | matches: `metal/adapter.rs:1333-1359` starts from `DownlevelFlags::all()` (wgpu-types `limits.rs:1063-1066`), sets `INDIRECT_EXECUTION` from `indirect_draw_dispatch` and `ANISOTROPIC_FILTERING` true; the test's `M4_PRO_DOWNLEVEL_FLAGS` agrees |
| `src/gpu/mod.rs:754` | `vulkan-max-storage-buffer-range` | matches: spec minimum 2^27; the floor keeps wgpu's default `128 << 20` (`limits.rs:441`) |
| `src/gpu/mod.rs:755` | `mesa-25.2.8-gallivm-max-shader-buffer-size` | matches: `1 << 27` equals the floor's binding size; the test asserts the Arena fits (`:757-761`); Mesa source not in the registry, quote taken as given |
| `scripts/cycles-reference/render.py:99` | `khr-lights-punctual-range` | matches: quote `max( min( 1.0 - ( current_distance / range )⁴, 1 ), 0 ) / current_distance²`; `render.py:108-122` builds `1 − (ray length / range)⁴` clamped to [0, 1] on the light's emission, Cycles' point light supplies `1/d²`; `calibrate.py:72-74` checks `1 − (10/20)⁴ = 0.9375` |
| `scripts/cycles-reference/render.py:304` | `gltf-2.0-metallic-roughness` | matches: green to Roughness, blue to Metallic, each times its factor, `render.py:305-310, 318` |
| `scripts/cycles-reference/render.py:313` | `gltf-2.0-zero-roughness` | matches: the clamp path, `max(roughness, MINIMUM_ROUGHNESS)` read from `shading.wgsl`, `render.py:37-40, 315-319` |
| `scripts/cycles-reference/compare.py:30` | `gt7-tone-mapping-cpp` | matches: quote's 250 cd/m² over 100 cd/m² is `GT7_PAPER_WHITE = 2.5` (`compare.py:54`), `initializeCurve(..., 0.25f, 0.538f, 0.444f, 1.280f)` is `compare.py:84`, the 100 cd/m² reference is the PQ scale `100.0 / 10000.0` (`compare.py:97, 104`); equal to `gt7.wgsl:32, 71, 79` and the curve at `gt7.wgsl:62, 120, 122` |
| `scripts/sweep.sh:19` | `apple-launchd-start-calendar-interval-sleep` | matches: "your job will run when the computer wakes up"; the comment says "or on the next wake when asleep then", `sweep.sh:18-19`, `sweep.plist:17-23` |
| `scripts/sweep.sh:96` | `nvidia-smi-utilization-sample-period` | matches: "between 1 second and 1/6 second"; the comment says "covers the past 1/6 s to 1 s", `sweep.sh:94-96` |

No other `[ref:` citation exists under `src/bench/`.
Also confirmed for ADR 0021's limit clauses though cited only in the ADR: `wgpu-hal-30-vulkan-storage-buffers-halved` (`vulkan/adapter.rs:1534-1553`), `wgpu-types-30-acceleration-structure-minimums` (`limits.rs:774-786`: `max_tlas_instance_count: (1 << 24) - 1`, `max_acceleration_structures_per_shader_stage: 1`, which the floor raises to 2).
No key here quotes a wgpu crate, so no cargo registry source was opened.

### Findings

| id | claim | severity | where | evidence | expected |
|---|---|---|---|---|---|
| L1-T01 | `compare` judges the CPU frame Budget at the median only; the CPU p95 is never compared with 6 ms | correctness: clause broken (today's town Baseline p95 5,088 µs passes either way; the ADR's own exit figure, 6,210 µs at p95, would pass `compare`) | `src/bench/mod.rs:683-696` | block below | story 55: "6 ms of main-thread CPU, at the median and p95 for CPU" |
| L1-T02 | No code marks a Baseline re-recorded after the history started; only imported Baselines carry a Baseline mark and start a Series line | correctness: clause broken (one sweep night per re-recorded scene; 21 Baselines were re-recorded after the history started, commit `732e58c7`) | `src/bench/history.rs:356-363`; `src/bench/history/import.rs:69`, `:277` | block below | ADR 0024: "a re-recorded Baseline's record is marked as one" |
| L1-T03 | The ADR's count of imported Baseline commits, 61, is not the first-parent count before `HISTORY_STARTED`, which is 64 | drift: not load-bearing (misleads a reader checking the import's output) | `docs/adr/0024-bench-history-is-git-stored-bench-records-keyed-by-engine-source.md:19`; `src/bench/history/import.rs:23`, `:146-157` | block below | the ADR's number equals what the import reads |
| L1-T04 | A record on a Series holds its key against a tagged one whatever their spreads, and a Series record displaces a tagged one; the ADR says the quieter stays | drift: not load-bearing (documented in the branch README; a tagged record is never drawn) | `src/bench/history.rs:952-959`, `:986-988`, `:100-101` | block below | ADR 0024: "When two records share a key, the quieter one stays: the one with the lower GPU p95/median" |
| L1-T05 | `praxis preflight` prints no assigned tickets for the live milestone, because `live_milestone` parses the title after `M` as a `u32` and the only open milestone is `M3.5` | correctness: the clause "the live milestone's assigned tickets" broken for every session of the Consolidation milestone; 1 of preflight's 9 lines wrong | `src/preflight/mod.rs:612-618`, `:567-569` | block below | `docs/agents/workflow.md:68`: preflight prints "the open PRs and the live milestone's assigned tickets" |
| L1-T06 | ADR 0024's quiet rule says a record joins a Series when "preflight warned nothing", but the quiet verdict reads four of preflight's five warnings: the assets warning (fetched files absent, Cooked cache empty) never enters it | drift: not load-bearing (timings unaffected); a reader of the ADR expects an empty Cooked cache to keep a record off its Series | `docs/adr/0024-bench-history-is-git-stored-bench-records-keyed-by-engine-source.md:17`; `src/preflight/mod.rs:242-262`; `src/bench/provenance.rs:126-139` | block below | ADR text and code name the same set of warnings |
| L1-T07 | `gpu_memory_bytes` leaves out the capture target, a texture the engine created, where ADR 0013 counts "the bytes of the buffers and textures the engine has created" | drift: documented in the field's comment, not in the ADR or `deviations.md`; 14,745,600 bytes at 2560×1440 (1.9% of `town`'s 764 MB Baseline), plus a 14,745,600-byte read-back buffer alive during each read-back | `src/stats/mod.rs:39-45`; `src/capture/mod.rs:210-223, 305-310` | block below | ADR 0013 line 3: GPU memory "counted as the bytes of the buffers and textures the engine has created, summed by the engine itself on every backend; ... the GPU stats readback's buffers are left out" (the only exclusion named) |
| L1-T08 | The sweep re-benches a bisect's end or commit with `--no-publish`, which the release binary of any commit before `3c6b9d92` refuses, so the first live Suspect on a Harness version 10 Series whose good end is an imported record fails to re-bench every night | correctness: 12 first-parent commits of the live Harness era predate the flag, 3 of them imported Baseline records | `scripts/sweep.sh:171` | block below | ADR 0024: "The sweep re-benches it, bisects the commits behind it" |
| L1-T09 | `ci.yml` pins no action to a commit SHA, and `dtolnay/rust-toolchain@stable` pins no version at all, in a workflow whose `publish-bench` job holds `contents: write` | correctness: 18 `uses:` lines, 4 to a branch, 2 in the write-capable job | `.github/workflows/ci.yml:37-39` | block below | `CLAUDE.md` review checklist: "Workflows keep least-privilege permissions and pinned action versions"; the form `pr-lint.yml:29` uses |
| L1-T10 | `pr-lint.sh` enforces no bound on the Verification section's pasted output, though the PRs section it enforces caps it at 30 lines | correctness: one rule of the section unenforced | `scripts/pr-lint.sh:44-60` | block below | `docs/agents/workflow.md:32`: "how it was verified (command and output, 30 lines at most) ... CI's PR form check (`scripts/pr-lint.sh`) fails a description or commit that breaks this section's rules" |
| L1-T11 | `verify.sh` skips the aliasing measurement when the diff touches only `src/scenes/`, whose generators change what every Reference scene renders | correctness: one directory outside the filter; a scene change's aliasing shift passes verify and lands on the next rendering PR's compare | `scripts/verify.sh:37-43` | block below | `verify.sh:34-36`'s own rule: measured "when the diff against main can change what renders"; `docs/agents/workflow.md:30`: verify judges "the bench counters against Baselines and Budgets" |

#### L1-T01

```rust
// src/bench/mod.rs:683-696
    let cpu = current.cpu_frame_microseconds.median;
    check_regression(
        &mut findings,
        "cpu_frame_microseconds",
        cpu,
        baseline.cpu_frame_microseconds.median,
        CPU_NOISE_FLOOR_MICROSECONDS,
    );
    check_budget(
        &mut findings,
        "cpu_frame_microseconds",
        cpu,
        budgets.cpu_frame_microseconds,
    );
```

```rust
// src/bench/mod.rs:629-630
/// Compares `current` with its `baseline` and the Budgets. Medians count;
/// p95s are reported only, as they move with one slow frame.
```

Spec story 55: "I want `town` within 12 ms of GPU and 6 ms of main-thread CPU, at the median and p95 for CPU, at 1440p with 4x MSAA". ADR 0013 (re-decision): "`town`'s at 2560×1440 with 4x MSAA on the M4 Pro (CPU frame 5.80 ms at the median and 6.21 ms at p95, GPU frame 20.1 ms, GPU memory 764 MB): each stays as written and is no longer provisional ... what `town` breaches is ticketed as an offender", which names the CPU p95 as a judged figure. `Spread.p95` is computed (`src/bench/mod.rs:879-885`) and printed by `report` (`src/bench/report.rs:55`, `:116`) but no `check_budget` reads it. Every test of `compare` moves medians only (`src/bench/mod.rs:1116-1125`). Magnitude: the current M4 Pro town Baseline has `cpu_frame_microseconds.p95` 5,088.292 (`bench/baselines/apple-m4-pro/town.json:11`), under 6,000, so no verdict changes today. Fix: one more `check_budget(&mut findings, "cpu_frame_microseconds p95", current.cpu_frame_microseconds.p95, budgets.cpu_frame_microseconds)` after the median's, the doc comment at `:629-630` reworded, and a test with a p95 over 6,000 and a median under it.

#### L1-T02

```rust
// src/bench/history.rs:356-363
            let starts_line: Vec<bool> = rows
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    (index > 0 && !rows[index - 1].same_definition(row))
                        || row.source == Some(import::IMPORTED_SOURCE)
                        || line_starts.contains(row.source_hash)
                })
```

```rust
// src/bench/history/import.rs:69
pub(super) const IMPORTED_SOURCE: &str = "baseline-commit";
```

ADR 0024: "Baselines are still chosen by hand, and a re-recorded Baseline's record is marked as one." The only Baseline mark is `provenance.source = "baseline-commit"`, written by `imported` (`src/bench/history/import.rs:277`) for commits before `HISTORY_STARTED` alone. `record` (`src/bench/history.rs:685-723`), which builds every record a bench publishes, inserts no such mark, and nothing in `rebuild_series` (`:1020-1034`) reads `bench/baselines/`. `git log --first-parent 3c6b9d92..HEAD -- bench/baselines` shows one commit, `732e58c7`, re-recording 21 Baselines (M4 Pro and lavapipe); their records, if published, start no line and carry no mark. Magnitude: a level change a re-recorded Baseline accepted is a Suspect (`src/bench/history/suspects.rs:50-87`), costs the sweep a night of re-bench and bisect, and is kept from an issue only by `covered_by_baseline` (`src/bench/history/verdicts.rs:424-436`, `:761`), which checks the pinned commit's diff on the Baseline path; the Series line also keeps judging later records against the pre-Baseline window until it rolls off. Fix: in `rebuild_series`, treat as a line start (and mark in the row, as the import does with `source`) any record whose source hash equals that of a `main` commit that changed `bench/baselines/<adapter key>/<scene>.json`, read with `git log --first-parent --format=%H -- <path>` as `baseline_records` already does for the import.

#### L1-T03

```rust
// src/bench/history/import.rs:23
const HISTORY_STARTED: &str = "3c6b9d92";
```

```rust
// src/bench/history/import.rs:146-157
    let changed_baselines = git(
        repository,
        &[
            "log",
            "--first-parent",
            "--format=%H",
            history_started,
            "--",
            BASELINES_DIRECTORY,
        ],
        None,
    )?;
```

ADR 0024 (`docs/adr/0024-...md:19`): "The 61 commits that touched `bench/baselines/` before the history existed are imported as records". The same `git log --first-parent --format=%H 3c6b9d92 -- bench/baselines` the code runs lists 64 commits at the frozen commit (64 also with `3c6b9d92^`, without `--first-parent`, and with `--diff-filter=AM -M100%` on `*.json`). Magnitude: the number is descriptive, nothing in code counts to it; a reader comparing the import's `Publication` line with the ADR is misled. Fix: the ADR line reads "the 64 commits" or drops the count.

#### L1-T04

```rust
// src/bench/history.rs:952-959
            if !on_series && record.experiment.is_none() && series_path.exists() {
                tracing::info!(
                    record = %relative_path.display(),
                    "Bench record dropped: one on a Series holds its key"
                );
                publication.dropped += 1;
                continue;
            }
```

```rust
// src/bench/history.rs:986-988
                if tagged_path.exists() {
                    std::fs::remove_file(&tagged_path)?;
                }
```

ADR 0024: "A record path is its key, so a duplicate is a file that already exists. When two records share a key, the quieter one stays: the one with the lower GPU p95/median." The ratio rule holds only within one directory (`src/bench/history.rs:962-981`); across `records/` and `tagged/` the Series record wins regardless, which the branch README states (`src/bench/history.rs:100-101`: "A record in `records/` holds its key against a tagged one") and a test pins (`:1479-1505`). A tagged record of the same source hash with a lower ratio exists when a run is tagged for the machine (battery, GPU load, another praxis process) or for its configuration or profile with steady frames. Magnitude: not load-bearing; the Series keeps a record that passed every quiet test, and the tagged one was never drawn. Fix: the ADR sentence gains "within a Series or among the tagged; a record on a Series holds its key against a tagged one", or the code applies the ratio across both, whichever the owner meant.

#### L1-T05

```rust
// src/preflight/mod.rs:610-618
/// The live milestone: the open one with the lowest number in its `M<n>`
/// title.
fn live_milestone<'title>(titles: &[&'title str]) -> Option<&'title str> {
    titles
        .iter()
        .filter_map(|&title| Some((title.strip_prefix('M')?.parse::<u32>().ok()?, title)))
        .min()
        .map(|(_, title)| title)
}
```

```rust
// src/preflight/mod.rs:567-569
    let Some(live) = live_milestone(&titles) else {
        return Check::new("tickets", Verdict::Note, "no open milestone named M<n>");
    };
```

`"3.5".parse::<u32>()` is an error, so `M3.5` is filtered out. The repository's milestones, listed read-only on 2026-11-02: one open, number 5, title `M3.5`, 36 open issues; no other open milestone. Preflight therefore prints `note  tickets   no open milestone named M<n>` on this commit. `docs/agents/workflow.md:68`: "It prints the session's starting state in ten lines or fewer: ... the open PRs and the live milestone's assigned tickets." `CONTEXT.md:232` and `docs/agents/workflow.md:12` make `M3.5` a milestone of the ladder, so the `M<n>` assumption in the doc comment and the test (`src/preflight/mod.rs:766-768`, titles `M10`, `M3`, `Backlog`, `M4`) is stale. Magnitude: every session of the Consolidation milestone reads no assigned tickets from preflight. Fix: parse the title after `M` as `<major>[.<minor>]` into `(u32, u32)` and take the minimum; add `M3.5` to the test at `:766-768` so that `["M4", "M3.5"]` yields `M3.5`.

#### L1-T06

ADR 0024, Consequences (line 17): "A record joins a Series only if its tree was clean, its profile release, the machine quiet (preflight warned nothing, and p95/median below about 1.06), and its source `main`'s."

```rust
// src/bench/provenance.rs:126-139
/// Whether a run was quiet: preflight would have warned of neither the GPU
/// load, other praxis processes, nor the power or display at its start, and its GPU frames spread
/// less than `QUIET_P95_OVER_MEDIAN`. ...
fn is_quiet(
    gpu_load_percent: Option<u32>,
    other_processes: Option<&[String]>,
    machine_warnings: &[String],
    gpu_frame_p95_over_median: Option<f64>,
) -> bool {
    gpu_load_percent.is_none_or(|percent| percent <= BUSY_GPU_PERCENT)
        && other_processes.is_none_or(<[String]>::is_empty)
        && machine_warnings.is_empty()
        && gpu_frame_p95_over_median.is_some_and(|ratio| ratio < QUIET_P95_OVER_MEDIAN)
}
```

`machine_warnings` is power and display only (`src/preflight/mod.rs:349-356`). Preflight's fifth `Warn` is the assets check:

```rust
// src/preflight/mod.rs:242-250
    if !absent.is_empty() {
        return Check::new(
            "assets",
            Verdict::Warn,
            format!(
                "{} of {fetched} fetched files absent, so verify's launches download them: run `praxis fetch`",
                absent.len()
            ),
        );
    }
```

and `:254-262` warns the same way on an empty Cooked cache. Neither reaches `is_quiet`, so a record taken on a machine whose preflight warned of absent fetched files joins its Series, against the ADR's words; the code's own comment already states the narrower rule. Magnitude: a doc sentence; no timing changes either way (fetching and cooking precede the measured frames). Fix: amend the ADR's parenthesis to "preflight warned of neither the GPU load, other praxis processes, nor the power or display", or make the assets check a `Note`, since `docs/agents/workflow.md:68` defines `warn` as "holds off a bench" and this one does not.

#### L1-T07

ADR 0013 (line 3): GPU memory is "counted as the bytes of the buffers and textures the engine has created, summed by the engine itself on every backend ...; the GPU stats readback's buffers are left out of that sum, since they measure the frame rather than hold it".

```rust
// src/stats/mod.rs:39-45
    /// Bytes of the GPU buffers and textures the State created, its Pools
    /// included, summed by the engine on every backend (ADR 0013), and of
    /// its acceleration structures, estimated. The render target and
    /// capture's own buffers are the caller's, and the GPU stats
    /// readback's are left out, since they measure the frame rather than
    /// hold it.
    pub gpu_memory_bytes: u64,
```

```rust
// src/capture/mod.rs:210-223
    let target = context.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Capture Target"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TARGET_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
```

The engine creates the capture target and the read-back buffer (`src/capture/mod.rs:305-310`); the ADR names one exclusion, the readback ring, and the field's comment adds a second, "the render target and capture's own buffers". The reason is sound (a window's surface textures are the surface's, so the stand-in stays out to count the same bytes as the windowed app), but it is written beside the field, not in the ADR or `docs/audit/deviations.md`, which has no rows. Magnitude at the reference size: 2560 × 1440 × 4 = 14,745,600 bytes for the target, the same for the read-back buffer (`padded_row_bytes` 10,240 × 1,440), 1.9% of `town`'s 764 MB Baseline, well inside the 8 GB Budget. Fix: one clause in ADR 0013 ("the presentation target, surface or capture, and capture's read-back buffer are left out, since a window does not create its surface textures"), or a `deviations.md` row saying the same.

#### L1-T08

```
scripts/sweep.sh:163-174
    read -r scene commit <<<"$job"
    may_start "the re-bench of $scene at $commit" || break
    tree=$(tree_at "$commit") || {
      log "no release binary of $commit"
      break
    }
    runs=$((runs + 1))
    run="$night/$(printf '%03d' "$runs")"
    (cd "$tree" && ./praxis bench "$run" --no-publish "$scene") || {
      log "re-bench of $scene at $commit failed"
      break
    }
```

`tree_at` builds the binary of the commit the bisect names (`sweep.sh:129-146`) and runs that binary. `--no-publish` entered the CLI at `3c6b9d92` (2026-10-31); the parser before it, `git show 3c6b9d92~1:src/bin/praxis.rs`, lists `bench <output directory> [--ci | --counters | --ci-counters] [--skip-aliasing] [<scene>...]` (line 225) and answers an unknown word with `no scene named {name:?}; scenes are {names:?}` (line 314), so the re-bench exits nonzero. The live Harness version is 10 (`src/bench/provenance.rs:21`); its era began at `a00280dc` (the last of `HARNESS_BREAKS`, `src/bench/history/import.rs:29-63`), 12 first-parent commits before `3c6b9d92`, and the imported Baselines of `cfb84b7c`, `3d4b21c3` and `6aec9278` lie in that gap, enough for `FEWEST_JUDGING_RECORDS = 3` (`suspects.rs:11, 91-94`). An imported record starts a line but is still the window of the record after it (`suspects.rs:25-28`), so the first live record that shifts is a Suspect whose good end is a pre-flag commit, and the bisect's intermediate commits are pre-flag too. The night's ends are benched first (`verdicts.rs:871`), so the night breaks at its first run, and a held Verdict with no pinned commit is resumed each night (`verdicts.rs:379-383`): the sweep never passes that Suspect. ADR 0024:21: "The sweep re-benches it, bisects the commits behind it, and files a confirmed Regression". Fix: pass the flag only to a binary that knows it, `git merge-base --is-ancestor 3c6b9d92 "$commit" && flags=--no-publish` (a pre-flag binary cannot publish anyway), or have `history bisect` name no commit before `HISTORY_STARTED` (`import.rs:23`). The same shape holds for `./praxis fetch` at `sweep.sh:143` for commits before `8d564977` (2026-10-25), outside the live era, so not counted.

#### L1-T09

```
.github/workflows/ci.yml:37-43
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - uses: Swatinem/rust-cache@v2
```

The 18 `uses:` lines: `actions/checkout@v4` at 37, 63, 101, 151, 223; `dtolnay/rust-toolchain@stable` at 39, 65, 103, 153; `Swatinem/rust-cache@v2` at 43, 67, 107, 155; `actions/upload-artifact@v4` at 87, 138, 187, 200; `actions/download-artifact@v4` at 225. `stable` is a branch of that repository that moves with every push; `v4` and `v2` are tags their owners move on each release. The job that can write the repository (`permissions: contents: write`, `ci.yml:220-221`) runs the checkout at 223 and the download at 225 by moving tag. The repository's other workflow pins by SHA with the version in a comment: `.github/workflows/pr-lint.yml:29`: `uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4.4.0`. `CLAUDE.md`, Review checklist: "Workflows keep least-privilege permissions and pinned action versions". Fix: each line to a commit SHA with its version beside it; `dtolnay/rust-toolchain@<sha>` with `with: toolchain: stable`.

#### L1-T10

```
scripts/pr-lint.sh:44-53
verification=$(section '## Verification')
uncounted=()
for kind in found fixed declined deferred; do
  if ! grep -Eiq "(^|[^0-9])[0-9]+ $kind([^[:alpha:]]|$)|(^|[^[:alpha:]])$kind:? [0-9]+" <<<"$verification"; then
    uncounted+=("$kind")
  fi
done
```

The section's checks are the counts (44-53) and the "Sources checked" line (55-60); nothing reads the section's length or its fenced output. `docs/agents/workflow.md:32`: "Description follows `.github/pull_request_template.md`: what changed, how it was verified (command and output, 30 lines at most), metric deltas ... CI's PR form check (`scripts/pr-lint.sh`) fails a description or commit that breaks this section's rules"; `.github/pull_request_template.md:11`: "The command run and its output, 30 lines at most". `pr-lint-test.sh:115-116` even passes a Verification section of 3,000 table rows. Magnitude: one of the section's mechanical rules unenforced; a PR pasting a full log passes the check that claims to fail it. Fix: `pr-lint.sh:21-22` already prefixes fenced lines with a space, so counting Verification's space-prefixed lines and missing above 30 is a few lines beside the counts check, with a case in `pr-lint-test.sh`.

#### L1-T11

```
scripts/verify.sh:34-43
# Aliasing renders each scene twice more, once at three times the Goldens'
# size, so it is measured only when the diff against main can change what
# renders: the renderer and its shaders, the cooker or the assets.
skip_aliasing=--skip-aliasing
base=$(git merge-base HEAD origin/main 2>/dev/null || git merge-base HEAD main 2>/dev/null)
if [ -z "$base" ] ||
  [ -n "$(git diff --name-only "$base" -- src/render src/cook assets)" ] ||
  [ -n "$(git ls-files --others --exclude-standard -- src/render src/cook assets)" ]; then
  skip_aliasing=
fi
```

`src/scenes/` is "Reference scenes: generated content, camera paths in frames, frame deltas, the Spin system ... town.rs, the dense scene's generator" (`CLAUDE.md`, Project shape); `aliasing_rmse` and `aliasing_p99` are "The aliasing of the Goldens' frame ... the engine's own render" (`src/bench/mod.rs:260-271`), a function of that content, and both are Baseline counters compare judges when present. A diff that moves `town`'s buildings or `orrery`'s spin changes the frame and the two counters, yet matches neither path, so `--skip-aliasing` stays set and compare judges no aliasing; the shift surfaces on the next PR that touches `src/render/`, charged to it. Magnitude: one directory (and, by the same reading, `src/camera/` and `src/environment/`, which set the view and the lights). Fix: add `src/scenes` to both path lists at `verify.sh:40-41`, and the comment at 35-36.

### Notes for the verifier

Not filed, since each needs a run, an owner's reading or belongs to another lane; recorded so the next audit does not re-find them.

- ADR 0024's "preflight warned nothing" is read as the machine checks only: `is_quiet` (`src/bench/provenance.rs:130-140`) takes GPU load, other praxis processes and `machine_warnings` (power, display: `src/preflight/mod.rs:347-356`); preflight's `assets` warning (`src/preflight/mod.rs:245`, `:257`) is not in it, and the README states the narrower rule (`src/bench/history.rs:67-69`). An absent fetched file or empty Cooked cache changes no measured frame, so no Finding; the owner says whether the parenthetical is literal.
- ADR 0013's Input latency Budget ("judged at p95") has no measurement anywhere in bench and no key in `bench/budgets.json:1-8`; nothing to quote against, so a ticket rather than a Finding.
- `compare` applies the Budgets to every adapter's Local run (`src/bench/mod.rs:654-659`, `:691-696`, `:716-721`) though ADR 0013 judges them "on an Apple M4 Pro"; the RTX 2070 SUPER's town run fails the GPU Budget as the M4 Pro's does. Owner call, not a defect.
- town's M4 Pro Baseline breaches the GPU frame (19,935 > 12,000) and gate (8,612 > 6,000) Budgets (`bench/baselines/apple-m4-pro/town.json:18`, `:22`), so `praxis compare` of a reference town run fails today by the ADR's own "ticketed as an offender" rule; `verify` compares counters only (`scripts/verify.sh:52`). Not filed.
- `report` pairs times with a Baseline by adapter and frame count, not backend (`src/bench/report.rs:25-27`), while `compare` requires the backend too (`src/bench/mod.rs:673-682`) and the doc says "as `compare` judges them" (`src/bench/report.rs:12-13`); the binary reads the Baseline by adapter and backend (`src/bin/praxis.rs:701-704`), so no wrong pairing reaches it. Health, one line.
- `dynamic_tlas_cpu_microseconds` sums `BLAS_REFIT` too (`src/bench/mod.rs:552-556`), as its field doc says (`:160-162`), but the `Budgets` doc names only "its build's recording and submit" (`:383-386`). On crowd the refit joins the 1,100 µs ceiling; story 57's 0.5 ms CPU figure for the crowd's acceleration-structure update is in no Budget. Owner says whether 0.5 ms was a decision input only.
- Story 59 (warm verify under 60 s) needs a timed run on the M4 Pro.
- Oracle-style constants and their justification: `WARN_RATIO`/`FAIL_RATIO` 1.05/1.10, `CPU_NOISE_FLOOR_MICROSECONDS` 10, `GPU_PASS_NOISE_FLOOR_MICROSECONDS` 20 are justified by measurements quoted in their comments (`src/bench/mod.rs:129-143`: GPU medians within 3.4 %, CPU medians within 8.1 µs, culling pass 136-153 µs); `QUIET_P95_OVER_MEDIAN` 1.06 (`src/bench/provenance.rs:28-30`) by the ADR's "about 1.06" alone; `WINDOW_RECORDS` 8, `FEWEST_JUDGING_RECORDS` 3, `MEDIAN_ABSOLUTE_DEVIATIONS` 4, `SMALLEST_SHIFT_FRACTION` 0.03 (`src/bench/history/suspects.rs:7-19`) by the glossary and the ±1.2 % noise fixture (`:130-135`); `SMALLEST_HELD_SHIFT` 0.03 and `END_RUNS` 2 (`src/bench/history/verdicts.rs:21-27`) by the glossary only. A mean of two runs per end (`:239`) is the thinnest sample that can be interleaved; a performance-lane question, not a correctness one. The aliasing tests' expected values 333,333 and 707,107 (`src/bench/mod.rs:1519-1520`, `:1534-1535`) are exact arithmetic, recomputed here.
- `HARNESS_VERSION` 10 (`src/bench/provenance.rs:21`) equals the import's last era (9 breaks + 1, `src/bench/history/import.rs:29-66`, `:97`); it was last changed at the import commit, and no first-parent commit since `3c6b9d92` on `src/stats`, `src/render/readback.rs` or `src/bench/mod.rs` changes a metric's meaning by its subject. Only a run of `praxis history import` against the branch checks the restamp.
- The nearest-rank percentile (`src/bench/mod.rs:889-893`) takes the lower middle for an even count, so the suspects' window median of 8 is the 4th smallest; the shift test's 0.04-0.06 band holds (recomputed: 530/9,970 = 0.053).
- `check_budget` for `gpu_memory_bytes` reads the measured frames' median (`src/bench/mod.rs:657`) while the counter comparison may read the counter frames' (`:641`, `:794-803`); the counter is constant after frame 8, so the values agree.
- `src/stats/mod.rs:182-200`: `texture_bytes` counts a `TRANSIENT_ATTACHMENT` texture as 0 where the adapter keeps it in tile memory. ADR 0013 line 3 says "the bytes of the ... textures the engine has created"; a memoryless attachment holds no device memory, so 0 is its true size, but no ADR clause or `deviations.md` row says so. Needs the owner: record it or leave it.
- ADR 0013 line 3, "CPU time summed across threads is measured and reported": nothing sums across threads; `src/stats/mod.rs:11-13, 415-422` time the frame's thread only. Moot while no system uses the compute pool (ADR 0016 line 3); re-check when one does.
- `src/preflight/mod.rs:240`: the assets check's `Fail` branch (manifest unreadable) names no fix, against `docs/agents/workflow.md:68` "names its fix". Needs a run to see the message; the other three `Fail` lines name theirs (`:130, 184-189, 218-225`).
- Whether each preflight `Fail` (adapter, Goldens or Baselines absent) actually fails `scripts/verify.sh`: scripts reader.
- The M3.5 spec's decision stamps lanes 2 and 3 quiet by "load average, `pmset -g batt`, no peer benching". The stamp records GPU utilisation from `ioreg` or `nvidia-smi` (`src/preflight/mod.rs:286-303`), battery (`:360-380`) and peer praxis processes (`:430-443`); nothing in `src/` or `scripts/` reads a CPU load average. Needs the owner: is GPU utilisation the intended reading, or should `getloadavg` join the stamp.
- `src/gpu/mod.rs:446-449`: the comment says Vulkan reports `SURFACE_VIEW_FORMATS` "only with `VK_KHR_swapchain_mutable_format`"; wgpu-hal `vulkan/adapter.rs:701-705` also sets it when the swapchain extension is absent, which no presenting device is. Not filed.
- `src/capture/mod.rs:243-251`: `FrameWait::Spin` busy-waits, which the cited Apple talk advises against; confined to the bench on a hardware adapter and documented with its reason (the blocked-thread clock). Whether that reason is still measured on wgpu 30.0.1 is the bench reader's question.
- ADR 0016 "Only the main thread submits, presents or polls": the Goldens test captures on test threads, each with a device of its own (`tests/goldens.rs:266-278, 287, 318`), so each device has one thread. The clause is read as "the device's owning thread"; a sentence in the ADR would remove the doubt. Docs lane.
- `tests/goldens/README.md`, named by the brief, does not exist; the README the test checks is the repository root's (`tests/goldens.rs:336-337, 388`). `CONTEXT.md` has no `Quiet` entry (the word appears only in the Bench record and Series entries, `:192, 200`), though the brief lists it. Docs lane.
- `gpu_memory_bytes`'s sum and the readback exclusion are implemented in render (`src/state.rs:302` calls `renderer.gpu_memory_bytes()`); the render reader owns their correctness.
- `src/gpu/mod.rs:3-19`: the 16 MiB block comment's claims were checked against the sources (`Performance` 128 to 256 MiB, `Manual` clamp, personal blocks, `GpuAllocatorManaged` textures); the 198 µs versus 52 µs measurement it quotes needs the RTX 2070 SUPER to re-confirm. Performance lane.
- `scripts/verify.sh:78` prints the wall time and asserts nothing about spec story 59's 60 s; a warm run on the M4 Pro decides it.
- `scripts/verify.sh:61-63` runs the Goldens test on the GPU beside every other test, some of which open a device too; whether that costs time or flakes needs a run.
- `scripts/pr-lint.sh:87-106` lints the branch's commits; the squash commit written at merge (`workflow.md:48`) is never linted. Process, for the owner.
- `scripts/pr-lint.sh` cannot check "the body says why, not what" (`workflow.md:35`) and does not check that a shading PR (`shading.wgsl`, `main.wgsl`, `shadow_mask.wgsl`, `froxel.wgsl`, `workflow.md:34`) links a Cycles image; the doc's lint claim names only before, after and difference, so not filed. The owner may want the second as a rule.
- `scripts/gallery.sh:32-33` copies `GOLDEN_FRAME` and `TOWN_FRAMES` from `tests/goldens.rs:294, 299`; equal today, no test ties them.
- `scripts/gallery.sh:118-119` captures `proxies` and `refit` of the three `--crowd-form` values (`src/bin/praxis.rs:320` adds `rebuild`); `workflow.md:64` says "both shadow forms". Whether `rebuild` can render differently from `refit` is the render reader's.
- `scripts/gallery.sh:57-59`: a run of spaces before the script path and a literal newline inside a `printf` format; it works, health only.
- `scripts/sweep.sh:183` publishes, with the current binary, result files that binaries of older commits wrote; whether `publish` reads every Harness version 10 record shape falls in `src/bench/`.
- `scripts/sweep.sh:202` ends the night with `fail` when publishing `main`'s records fails, after the records are on disk; ADR 0024's "Publishing never blocks work" concerns the bench, which has succeeded, so not filed.
- `.github/workflows/ci.yml:11`: pushes to `main` touching only `scripts/`, `tests/`, `bench/` or `.github/` run CI and publish a record of an unchanged source hash; harmless under ADR 0024:17 "the first record on a Series under a key stays", not filed.
- `.github/workflows/ci.yml:58` says the software Goldens job is "Not a required check until two green weeks"; the branch protection that decides it is repository settings, not readable here. For the owner.
- `scripts/cycles-reference/calibrate.py:72` names KHR_lights_punctual's window without a `[ref:]` key; the key sits on the builder it calls, `render.py:99`. Not filed.
- The M3.5 decision's quiet stamp (load average, `pmset -g batt`, peers) is `src/preflight/mod.rs:308-372`; the sweep only greps its `warn` lines (`sweep.sh:87, 105`), whose labels match (`mod.rs:94, 308-314`). Falls in preflight.

### Reader scope statements

**Bench with its history.** Read in full at the frozen commit (worktree `praxis-497`, only `docs/audit/` differs): `src/bench/mod.rs` (1,612 lines), `src/bench/report.rs` (419), `src/bench/provenance.rs` (449), `src/bench/history.rs` (1,942), `src/bench/history/verdicts.rs` (1,171), `src/bench/history/import.rs` (607), `src/bench/history/suspects.rs` (208): 6,408 lines. Also `bench/budgets.json`, `bench/baselines/apple-m4-pro/town.json`, the `bench`/`publish`/`history`/`compare`/`report` dispatch in `src/bin/praxis.rs:430-718`, `docs/agents/workflow.md` (Performance and memory), the glossary entries Baseline through Budget in `CONTEXT.md:183-225`, `src/preflight/mod.rs` where the quiet test reads it, and `scripts/verify.sh:52`. ADRs read clause by clause: 0013 and 0024. Git was read, never written (`git log`, `git show --stat`).

**Capture, preflight, stats and the GPU context.** Read in full at the frozen commit, in the worktree: `src/capture/mod.rs` (467 lines), `src/preflight/mod.rs` (777), `src/stats/mod.rs` (464), `src/gpu/mod.rs` (903), 2,611 lines. Read as far as they state what these files must deliver: `tests/goldens.rs:1-75, 130-330, 336-425`, `src/bin/praxis.rs:15-30, 95-131, 352-369, 725-788`, `src/state.rs:250-310`, `src/render/prepare.rs:77-88, 256-270, 468-474`, `src/bench/provenance.rs:24-32, 120-140`, `docs/agents/workflow.md` Session start (line 68) and Correctness, `CONTEXT.md:171-233`. ADRs read whole: 0004, 0013, 0016, 0021, 0023, 0024. Registry keys opened in `docs/references.toml`, and their quoted lines confirmed in `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` for wgpu-hal 30.0.1, wgpu-core 30.0.1, wgpu-types 30.0.1, gpu-allocator 0.28.0 and bevy_ecs 0.18.1. The repository's milestones were listed read-only (`gh api repos/gariasf/praxis/milestones`, 2026-11-02): one open milestone, titled `M3.5`, 36 open issues. Nothing was run, written or benched. Note: `tests/goldens/README.md`, which the brief names, does not exist; the Goldens test checks the repository root `README.md` (`tests/goldens.rs:336-337, 388`).

**Scripts, the Cycles reference and the workflows.** Read in full at the worktree (`HEAD` `1902cf4c`, equal to the frozen commit `8bc34eb1` in every file below; only `docs/audit/m3.5/correctness.md` differs): `scripts/verify.sh` (78 lines), `scripts/gallery.sh` (120), `scripts/sweep.sh` (206), `scripts/sweep.plist` (29), `scripts/sweep-task.xml` (39), `scripts/pr-lint.sh` (108), `scripts/pr-lint-test.sh` (204), `scripts/cycles-reference/render.py` (450), `compare.py` (207), `calibrate.py` (93), `DIFFERENCES.md` (76), `display-ramp.txt` (350, header plus 348 rows of six numbers), `.github/workflows/ci.yml` (238), `.github/workflows/pr-lint.yml` (58), `.github/pull_request_template.md` (25): 2,281 lines. ADRs read: 0024, 0013, 0023 in full; 0004, 0016 and 0021 place no clause in these files (their implementations are `src/capture/`, `src/render/`, `src/gpu/`). Intent read beside them: `docs/agents/workflow.md` sections PRs, Review, Performance and memory, Correctness, Session start (lines 26-72); `CLAUDE.md` review checklist; `docs/agents/triage-labels.md`; `docs/agents/issue-tracker.md`; the six `docs/references.toml` entries the files cite. To check the scripts' contracts I also opened `tests/goldens.rs:294-306` (frames), `src/scenes/mod.rs:71-98` (scene names), `src/bin/praxis.rs:156-320` (the CLI the scripts call), `src/bench/history/import.rs:23-63` (Harness eras), `src/bench/history/suspects.rs:10-29, 89-104` and `verdicts.rs:366-444` (what the sweep's bisect names), `src/preflight/mod.rs:94, 308-314, 358-372` (the labels the sweep greps), `src/render/shaders/gt7.wgsl:32-122` and `display.wgsl:8-13` (the constants `compare.py` ports), and `git show 3c6b9d92~1:src/bin/praxis.rs` for the CLI before the publish flag existed. `calibrate.py` asserts (`calibrate.py:26, 84-86`) that a white Lambertian plane's rendered luminance is E/π within 1% of the closed form for five lights it builds through `render.py`'s own builders: a point of 1 cd at 1 m (E = 1), at 10 m (E = 0.01), at 10 m with range 20 m (E = 0.9375/100, KHR_lights_punctual's window), a sun of 1 lx head-on, and a white world of 1 lx (`calibrate.py:67-77`); its source is the inverse-square law and the registry's range window, not a registry key of its own, and `--wrong-unit` (`calibrate.py:61-62`) makes the check red-capable, which `gallery.sh:56-61` exercises.

### Counts

| Severity class | Findings |
|---|---|
| `image` | 0 |
| `budget` | 0 |
| `correctness` | 7 (L1-T01, L1-T02, L1-T05, L1-T08, L1-T09, L1-T10, L1-T11) |
| `drift` | 4 (L1-T03, L1-T04, L1-T06, L1-T07) |
| `health` | 0 |

Clause verdicts: 99 rows; `holds` 66, two of them with a Finding on the
clause's text (L1-T03, L1-T07); `broken` 6 (L1-T01, L1-T02, L1-T04,
L1-T05, L1-T08, and the Capability floor's format list, L1-R01, filed by
the render reader); `drift` 1 (L1-T06); `superseded` 0; falls in another
module or outside tooling 19; not applicable 6; not implemented 1 (ADR
0013's Input latency Budget, in the Notes). Spec stories: 21 checked (3,
8, 9, 14, 16, 20, 34, 35, 36, 42, 45, 48, 50, 51, 52, 53, 55, 56, 57, 58,
59) and the spec's publishing and quiet decisions; 1 broken in this module
(55, L1-T01), 2 need a run or the owner's record (16, 59), the rest hold
in this module or fall in render, shaders or cook. Registry keys: 21, all
match.

## Killed

Written by the verifier.
