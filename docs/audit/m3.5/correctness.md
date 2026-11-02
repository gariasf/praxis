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

## Killed

Written by the verifier.
