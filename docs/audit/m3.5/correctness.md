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
