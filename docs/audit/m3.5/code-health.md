# Audit lane 5, code health

The lane's report, in the form `docs/agents/audit.md` gives: `src/render`
and the shaders read in full at the frozen commit
`8bc34eb18fcfb79307193551e96619316aef566c`, and every fifth other file in
path order, along six axes: over-engineering, vestigial code, cognitive
complexity, naming against the full-word rule, module depth and comment
accuracy. Four readers, one per module, each a section below; the lane's
report session writes the Summary, the counts and the Killed section and
runs the verifier over every section. Finding ids carry the module's
letter so that sections written in parallel never collide: render
`L5-R<n>`, shaders `L5-S<n>`, cook with assets and scenes `L5-C<n>`,
tooling `L5-T<n>`.

## Summary

Written by the report session.

## Counts

Written by the report session.

## render

Scope: `src/render/` outside the shaders and the shader test modules:
`renderer.rs`, `prepare.rs`, `instance.rs`, `uniforms.rs`, `staging.rs`,
`mod.rs`, `targets.rs`, `bind_groups.rs`, `culling_pass.rs`,
`depth_pyramid_pass.rs`, `main_pass.rs`, `froxel_pass.rs`,
`tonemapping_pass.rs`, `light_count_pass.rs`, `shadow_mask_pass.rs`,
`acceleration_structures.rs`, `blas_refit.rs`, `bend_pass.rs`,
`readback.rs`, `coverage_tests.rs`; 11,505 lines, every file read in full
with its test module. Four readers at high effort shared the module by
file, each with the lane brief verbatim: the Renderer with the Instance
and uniform layouts; the data path (prepare, staging, targets, bind
groups); ray tracing and the GPU stats readback with the coverage tests;
and the eight passes. Each swept its files along the six axes and
recorded the sweep, so what was read and judged clean is on the record
beside what was filed. The cognitive complexity probe,
`cargo clippy --all-targets -- -W clippy::cognitive_complexity`, ran once
at the frozen commit: three functions over the default threshold of 25,
two in the cooker's exporter and one in the binary's command dispatch,
none under `src/render/`, so this section files nothing on that axis.
Every `where` is `path:line` at the frozen commit, under `src/render/`
unless given in full. `deviations.md` holds no row at this commit;
`DIFFERENCES.md` was read for what is not a Finding. `tests/conventions.rs`
was read for the rules already mechanical, and none of its catches is
re-filed. Two readers filed the `view_proj` spelling from different files;
one row stands, counting the crate-wide sites. Each Finding's evidence
block says whether the fix it implies can move a pixel: none can, so every
`health` fix ticket carries the refactor licence and none is a defect.

### Axis sweep

| File | Over-engineering | Vestigial | Cognitive complexity | Naming | Module depth | Comments |
|---|---|---|---|---|---|---|
| `src/render/renderer.rs` | clean (checked `with_draw_capacity`, one test caller beside `new`; the `frame_encoder` fallback; the derived `shadow_rays` and `marks_dynamic_tlas_build` fields; see Notes) | L5-R07 (checked every Render switch against the binary, bench, scenes, tests and scripts: each is set somewhere; every `CrowdForm` method has a caller; every test pins shipping behaviour) | none in this letter (probe ran clean) | clean (checked every local, parameter and field; `gpu`, `ev100`, `uv` weighed as domain words; `_pad*` as the padding form CLAUDE.md names) | L5-R01 (`device`, `queue` crate-visible with no outside reader); `aspect_ratio` weighed, see Notes | L5-R04, L5-R05, L5-R06 (checked `TIMED_PASSES`'s doc and the `draw_calls += 4` comment against `readback.rs` and `main_pass.rs`: consistent) |
| `src/render/renderer.rs`, test module | L5-R08, L5-R09 (literals a helper would hold) | clean (`Occluder`'s four variants are all constructed; `shows`, `shadowed_floor`, `floor_drawn_with`, `TRIANGLE_BOUNDS`, `SUN` all used) | none in this letter (probe ran clean) | clean (`x`, `y` are coordinates; `gpu` a domain word) | clean (`gpu_context` has 9 outside callers, `render_frame` 1) | clean (the `doubling_the_instances` doc's four draws match `main_pass.rs:329-333` with shadow rays on) |
| `src/render/mod.rs` | clean | clean (every re-export has at least one consumer, the module's own files included) | none in this letter (probe ran clean) | clean | L5-R02 (12 re-exports with no consumer outside the module; two import paths in use) | clean (`FRAMES_IN_FLIGHT`'s doc matches `readback.rs` and `staging.rs` sizing by it) |
| `src/render/instance.rs` | clean (every item has 2 or more consumers) | clean | none in this letter (probe ran clean) | clean | clean (7 items, each with consumers in 2 or more other files) | clean (64 MiB checked: 2^19 Instances of 128 bytes; `ANCHOR_CELL_SIZE` declared from here by `shaders/mod.rs`, 3 sites) |
| `src/render/uniforms.rs` | clean | clean (every field is read by a WGSL part) | none in this letter (probe ran clean) | L5-R03 (`view_proj`) | clean (3 structs, 1 constant; consumers in `bind_groups.rs`, `prepare.rs`, `acceleration_structures.rs`, `shader_tests.rs`) | clean (`tlas_anchor`'s doc matches `renderer.rs:499-506`; `LightData`'s order is prepare's, render B) |
| `src/render/prepare.rs` | clean (the generic alias `LightQuery<Light>` has two instantiations, `RenderableQuery<Filter>` two; `rewrite_every_instance` is set by `state.rs:261`; `FrameData::background` and `InstanceWrite::moving` are read in the Renderer and the TLAS; `prepare`'s `aspect_ratio` varies between `state.rs:263` and the tests) | clean (every field of `FramePreparer` is read in `prepare`; every `FrameData` field is read in `renderer.rs` or `acceleration_structures.rs`; every test helper has two or more callers) | none in this letter (probe ran clean) | L5-R03 | clean (8 public functions, each called from `state.rs`; `frame()` has one caller, needed because `prepare`'s borrow cannot be held across the stats writes) | clean (the `prepare` doc, the `ChangedRenderableQuery` doc and the `moving_instances` doc each checked against the code paths they describe) |
| `src/render/staging.rs` | clean (`max_slot_size` has one caller but is computed from the Renderer's buffers at `renderer.rs:343`, so it is not a constant every caller repeats; `write`'s `Clone` bound serves the size pass before the copy pass) | clean (`SLOT_COUNT`, `LOAD_CHUNK_BYTES`, `mark_mapped` each used; `is_mapped`, `map_requested` both read and written) | none in this letter (probe ran clean) | clean (`max_slot_size` weighed, see Notes) | clean (3 and 5 public functions, every one with a caller outside the file) | clean (the slot-turn doc at lines 11 to 22 checked against lines 76 to 78: the slot mapped next was last filled `FRAMES_IN_FLIGHT` writes before; the `wgpu-30-handle-mapping` key exists) |
| `src/render/targets.rs` | clean (`surface_normals_attachment(clear)` is called with `true` and `false`; `final_depth_attachment(kept)` takes `self.keeps_depth`, which the `count_point_lights` switch sets at `main_pass.rs:186`) | L5-R10 | none in this letter (probe ran clean) | clean (`r`, `g`, `b`, `a`, `ops` are wgpu's field names; the locals are `red`, `green`, `blue`, `weight`) | L5-R10 | L5-R12, L5-R13 (the `color_attachment` doc checked against `main.wgsl:305-306` and `hdr.wgsl:23-25`; the 14-bit row against `shadow_mask.wgsl:373-375`; `octahedral_encode` at `common.wgsl:293`; `LUMINANCE_COEFFICIENTS` declared at `shaders/mod.rs:248`; the four registry keys exist) |
| `src/render/bind_groups.rs` | clean (`layout_of` has two callers, `depth_pyramid_pass.rs:36-37`, each a different subset, which is the flexibility it exists for; `visible_to` three callers in `main_pass.rs`; `Binding::count` set by one helper, `channel`, for the five Channel arrays) | clean (each of the 14 definitions is bound and set in a pass; `BLAS_REFIT_MARKERS` carries the same `cfg` as its one user in `blas_refit.rs`) | none in this letter (probe ran clean) | clean (`ty` is wgpu's field name; `wgsl_type`, `buffer_type` full words) | clean (`uniform` and `read_only_storage` are `pub(crate)` for `shader_tests.rs` alone, weighed in Notes) | L5-R11 (the `SHADOW_RAY_SCENE` doc's `wgpu-core 30.0.1` matches `Cargo.lock:2996-2997`; the `MATERIALS` doc's Channel order matches `assets/material.rs:47-53`; the three wgpu keys exist) |
| `src/render/acceleration_structures.rs` | clean; `StaticTlas::new`'s `capacity` parameter has one shipping caller passing `STATIC_TLAS_CAPACITY` (`src/render/renderer.rs:357`) but the tests pass 8 and 2 (lines 655, 664, 795), so it is exercised; `fill`'s `.take(DYNAMIC_TLAS_CAPACITY)` (line 322) guards what `src/render/prepare.rs:437-438` already truncates, weighed as a render-path guard against an index panic, not filed | clean; `tlas_instance`, `placed_bounds`, `anchored_transform`, `create_encoder` and `BEND_ROUNDING_SHARE` each have 2 or more uses in the file; `NVIDIA_VENDOR` 1 use plus the test's literal; `DYNAMIC_TLAS_BUILD` used by `src/bench/mod.rs:746`, `src/bench/report.rs:94` and `src/render/renderer.rs:36` | none in this letter (probe ran clean) | clean; `x_axis`, `y_axis`, `z_axis`, `cells`, `reach`, `centre` are full words; `blas`, `tlas` are domain words | clean; 6 and 7 public functions, every one called from `renderer.rs` or `shadow_mask_pass.rs` (table below); `AccelerationStructureBytes`' three `pub` fields are read outside the file by one test literal only (`src/assets/pools.rs:696-700`), weighed, not filed | clean; checked the "250k Instances" claim (line 10) against `town`'s 254,256 Instances (`docs/performance.md:966`; 244,256 static, under the 262,144 capacity), the GI research doc's 2.1M-triangle and 100,064-Instance figures (`docs/research/global-illumination-and-lighting.md:95`), "group 3" (line 232) against `src/render/bind_groups.rs:162`, `DYNAMIC_TLAS_BUILD`'s span (lines 21-22) against `src/render/readback.rs:589` and `bracket` (`readback.rs:527-541`), and every cited registry key |
| `src/render/blas_refit.rs` | clean; `Order`, `RefitMarker`, `unguarded`, `triangles` and `scratch` each have 2 or more uses inside the module; the `metal` module is `cfg`-gated to Apple with one consumer because the mechanism is Metal's alone | clean; the non-Apple arms (lines 67-68, 71-72, 126-130, 161-162) are the `Rebuild` and `Proxies` path on Vulkan and DX12, which `tests/crowd.rs:49` and the RTX Baselines run | none in this letter (probe ran clean) | clean; `hal` is wgpu's module name, `raw_blases` and `raw_shared_event` follow wgpu-hal's spelling | clean; 3 public functions, 3 call sites in `renderer.rs` (359, 598, 774) | L5-R16; checked the module doc's "on Metal only" against `CrowdForm::unavailable_on` (`src/render/renderer.rs:133-134`), the `scratch` field doc's "one loaded later is not refitted, as logged" against lines 195-197 and the `zip(offsets)` at 256-259, "group 3" (line 395) against `src/render/bind_groups.rs:175`, and `BLAS_REFIT_PASS`'s span (lines 16-17) against `readback.rs:656` and `marker_after` (`readback.rs:545-551`) |
| `src/render/readback.rs` | clean; `FrameTimestamps`' four constructors each serve a distinct timestamp shape (6, 4, 1 and 1 callers); `SlotState`'s four variants are each constructed and matched | clean; `MAP_FAILED` (line 72) is stored at line 492 and matched only by the wildcard arm at line 477, weighed as a named value that documents the store, not filed | none in this letter (probe ran clean) | clean; `ty` is wgpu's field name; the `gpu` lambda binding (line 958) is a domain word | L5-R15 | clean; checked `COMPLETION_LAG`, `MAP_LAG` and `SLOT_COUNT` (3, 2, 7) against the ring test at lines 962-990, "the first of `TIMED_PASSES`" (line 118) against `src/render/renderer.rs:36`, the Metal stale-timestamp claim (line 7-9) against `docs/research/wgpu-capability-matrix.md:188`, the 32 sample buffers (line 76) against that matrix's query-set row, and every cited registry key |
| `src/render/coverage_tests.rs` | L5-R14; `Raster.draws: Vec<Vec<Triangle>>`, `Shading`, seven of `IdSource`'s eight variants and `Indexing::SharedRotated` exist for the ignored test alone | L5-R14; `flat_quads`, `sequence`, `sample_positions`, `analytic` and the 2x MSAA device feature have no caller outside the ignored test | none in this letter (probe ran clean) | L5-R17; weighed and allowed `(sin, cos)` (line 138, std's `sin_cos` spelling), `u`, `v` (lines 139-141, the quad plane's coordinates), `w` (the clip coordinate) and `in` in the test WGSL (the engine's `main.wgsl` spells its fragment input the same) | clean; a private test module with no `pub` item | clean; the performance doc's heading date and command (lines 671-673) match `docs/performance.md:1906` and `1932-1934`; "7,871 are in view" (line 156) needs a run to confirm, noted below |
| `src/render/culling_pass.rs` | L5-R18 (the compute pipeline descriptor shape, first site) | clean: every `pub(crate)` constant is declared to WGSL by `src/render/shaders/mod.rs` and read by `culling.wgsl` or `shadow_mask.wgsl`; `FrameDraw`'s four `pub(crate)` fields and both methods are read by `readback.rs`; `DrawCapacity`'s fields are set outside `PROVISIONAL` once, by a Renderer test (`renderer.rs:999`) | none in this letter (probe ran clean) | clean: `slots`, `pipelines`, `layouts`, `definition`, `phase_two` are words; `_padding` is a Rust-only padding field | clean: all 8 methods have a Renderer caller; 4 getters hand out buffers the Renderer binds, uploads or reads back | L5-R21 |
| `src/render/depth_pyramid_pass.rs` | L5-R18 (second site) | clean: `DEPTH_PYRAMID_TILE_SIZE` is declared to WGSL and read 4 times in `depth_pyramid.wgsl`; the three `pub(crate)` test helpers are used by `shadow_mask_pass.rs` tests and `renderer.rs` tests | none in this letter (probe ran clean) | clean: `x`, `y`, `below_x`, `below_y` are coordinates; `across`, `up` are words; `reduced.0` and `reduced.1` are tuple field accesses, not names | clean: 3 methods, each with one Renderer caller | clean: "each texel the minimum depth, the farthest" matches reversed depth; the `SAMPLE_COUNT == 4` assertion's comment names the unrolling reason the shader states |
| `src/render/main_pass.rs` | L5-R19 (two depth record functions of one shape) | clean: `lit_pass_depth_pipeline` is created with shadow rays on and set only with them off, a reachable switch (see Notes) | none in this letter (probe ran clean) | clean | L5-R20 (two buffers passed per call that construction could hold) | clean: the three wgpu-template comments at 84 to 89 and 319 are true of wgpu; weighed, not filed (see Notes) |
| `src/render/froxel_pass.rs` | clean: constructor takes the three Renderer buffers by name; no abstraction with one use | clean: all 8 constants are declared to WGSL and read in `froxel_grid.wgsl`, `froxel.wgsl`, `common.wgsl` or Renderer tests; `LightCounts` is read by `readback.rs` | none in this letter (probe ran clean) | clean | clean: 5 methods, each with a caller in the Renderer or the light count pass | clean: "80 pixels square at 2560 × 1440" (2560 / 32, 1440 / 18), "a list is 512 bytes" ((127 + 1) × 4), "18,432 lists, 9 MiB" (32 × 18 × 32 × 512 B) and "the last from 1,024 m" (1 m × 2^(30 / 3)) all check |
| `src/render/shadow_mask_pass.rs` | L5-R18 (fourth site); constructor's name-tagged buffer array weighed, not filed (see Notes) | clean: all 5 constants are declared to WGSL and read by `shadow_mask.wgsl` or `main.wgsl`; every `const _` assertion pins a layout the shader relies on | none in this letter (probe ran clean) | clean: `own` names the pass's own bindings; test locals `plane`, `wrong`, `across`, `down` are words | clean: 5 methods, each with one Renderer caller | clean: "fill the mask's 8 bytes per pixel" is asserted at 67; the three bind groups' doc at 101 to 103 matches `record`'s use of them at 348 to 385 |
| `src/render/tonemapping_pass.rs` | clean: one pipeline, one bind group, one texture | clean: `DITHER_MASK_SIDE` is declared to WGSL and read by `tonemapping.wgsl`; `DITHER_MASK` is read by `new` and both tests | none in this letter (probe ran clean) | clean: `r`, `g`, `b`, `a` in tests are `wgpu::Color`'s own field names | clean: 4 methods, each with one Renderer caller | clean on accuracy; the "about 38 µs at 1440p on the M4 Pro" figure at 16 is a number in a shipping comment that only a GPU run can check (see Notes) |
| `src/render/bend_pass.rs` | L5-R18 (fifth site) | clean: `BEND_CAPACITY` is read 4 times by `prepare.rs`, `BEND_WORKGROUP_SIZE` by WGSL, `BendData` by `prepare.rs`, `bind_groups.rs` and `shader_tests.rs`; the test helper `bent` is used by `scenes/mod.rs` tests | none in this letter (probe ran clean) | clean: `uv`, `qtangent`, `gpu` are the domain's words | clean: 4 methods, each with a Renderer caller; `record`'s `bool` is read as `bent` at `renderer.rs:587` | clean |
| `src/render/light_count_pass.rs` | L5-R18 (sixth site) | clean: the `count_point_lights` switch is set by `src/bin/praxis.rs:745` and a Renderer test (`renderer.rs:2002`), so the pass is not vestigial; `LIGHT_COUNT_TILE_SIZE` is declared to WGSL and read by `light_count.wgsl:29` | none in this letter (probe ran clean) | clean | clean: 3 methods, each with one Renderer caller | clean: the module doc's "reads the depth the main pass keeps then" matches `MainPass::keeps_depth` |

### Interface widths

| Type | Public items | Callers outside the file | Wider than needed | Where |
|---|---|---|---|---|
| `Renderer` | 16: 11 `pub fn` (`new`, `resize`, `aspect_ratio`, `upload`, `render`, `gpu_frame_stats`, `flush_gpu_frame_stats`, `take_lost_instance_writes`, `allocator_report_bytes`, `gpu_memory_bytes`, `estimated_acceleration_structure_bytes`), 5 `pub(crate)` fields (`device`, `queue`, `mesh_pool`, `material_pool`, `load_staging`) | `src/state.rs` (14 sites: every `pub fn` once each, `mesh_pool` twice, the destructure at `state.rs:150-154` taking `mesh_pool`, `material_pool`, `load_staging`); `src/render/bend_pass.rs` tests (`new`, `load_staging`, `mesh_pool`) | yes: `device` and `queue` have no reader outside the file; `aspect_ratio` is held only to be handed back (Notes) | L5-R01, `src/render/renderer.rs:168-169` |
| `RenderSwitches` | 7: 5 `pub` fields, `Default`, `pub(crate) constants` | fields set in `src/bin/praxis.rs:745-746`, `src/bench/mod.rs:462`, `tests/goldens.rs:145,151,159`, `tests/frame_stats.rs:292`, `tests/shadows.rs:153`, `tests/crowd.rs:34`; `constants` in `culling_pass.rs`, `main_pass.rs`, `shadow_mask_pass.rs`, `shaders/mod.rs` | no | |
| `CrowdForm` | 5: `ALL`, `name`, `from_name`, `unavailable_on`, `Default` | `src/bin/praxis.rs:158-159` (`from_name`, `ALL`, `name`), `src/state.rs:145` and `src/render/blas_refit.rs:59` (`unavailable_on`), `src/scenes/mod.rs`, `src/cook/export.rs`, `tests/crowd.rs` | no | |
| the render module (`mod.rs`) | 19 `pub mod`, 26 re-exported names, `FRAMES_IN_FLIGHT` | 11 files outside `src/render` import from it (`assets/loader.rs`, `assets/material.rs`, `assets/pools.rs`, `bench/mod.rs`, `bench/report.rs`, `capture/mod.rs`, `cook/export.rs`, `gpu/mod.rs`, `scenes/mod.rs`, `scenes/town.rs`, `state.rs`, `stats/mod.rs`); `lib.rs:30` keeps `mod render` private and re-exports `CrowdForm`, `RenderSwitches` | yes: 12 of the 26 re-exported names have no consumer outside the module | L5-R02, `src/render/mod.rs:28-38` |
| `InstanceData` and `instance.rs`'s items | 13: 6 `pub` fields, 4 constants, 2 functions, the struct | `acceleration_structures.rs`, `bind_groups.rs`, `prepare.rs`, `shader_tests.rs`, `shader_function_tests.rs`, `shaders/mod.rs`, `assets/pools.rs` | no | |
| `CameraUniform`, `LightUniform`, `LightData`, `LIGHT_CAPACITY` | 25: 21 `pub` fields (5 of them `_pad*`), 3 structs, 1 constant | `bind_groups.rs`, `prepare.rs`, `acceleration_structures.rs`, `shader_tests.rs`; every field read by a WGSL part in `shaders/` | no | |
| `FrameData` (`prepare.rs:22`) | 9 public fields, 0 public functions | `renderer.rs`, `acceleration_structures.rs`, `state.rs`: 29 sites name the type; every field read outside (`.instance_writes` 3, `.moving_instances` 8, `.instance_slot_count` 8, `.background` 2, `.bends` 11) | no | |
| `FramePreparer` (`prepare.rs:188`) | 8 public functions | `state.rs`: `new` 1, `prepare` 1, `frame` 1, the four counters 1 each, `rewrite_all_instances` 1; `renderer.rs` tests: `new`, `prepare` | no | |
| `InstanceWrite`, `MovingInstance` (`prepare.rs:120`, `:130`) | 3 and 2 public fields | `acceleration_structures.rs:147` reads `.moving`, `:320` the list; constructed in the Renderer's and the TLAS's tests | no | |
| `FrameStaging` (`staging.rs:23`) | 3 public functions | `renderer.rs`: `new` 1 (`:343`), `write` 1 (`:549`), `gpu_memory_bytes` 2 | no | |
| `LoadStaging` (`staging.rs:171`) | 5 public functions | `pools.rs` (`write` 2, `build_blas` 1, `submit` 3), `material.rs` (`write` 1, `write_texture` 1), `tonemapping_pass.rs` (`write_texture` 1, `submit` 2), `acceleration_structures.rs` (`submit` 1), `renderer.rs` (`submit` 1), `loader.rs`, `prepare.rs` tests (`new`) | no | |
| `RenderTargets` (`targets.rs:58`) and the module's constants | 17 public functions, 9 public constants, 2 public free functions | `new` 8 sites; getters: `.depth()` 7, `.shadow_mask()` 5, `.second_surfaces()` 3, `.shadow_edges()` 3, `.color_attachment(` 3, `.depth_attachment()` 2, `.depth_pyramid_levels()` 2, `.shadow_mask_samples()` 2, `.surface_normals_attachment(` 2, `.surface_normals()` 1, `.hdr()` 1, `.depth_pyramid()` 1, `.directional_verdicts()` 1, `.loaded_depth_attachment()` 1, `.final_depth_attachment(` 1, `gpu_memory_bytes` 1 (`renderer.rs:769`); constants: `SAMPLE_COUNT` 25, `LUMINANCE_COEFFICIENTS` 7, `HDR_FORMAT` 4, the five other formats 2 each, `SHADOW_MASK_SURFACES` 0; `depth_stencil_state` 2, `depth_equal_state` 1 | yes: `SHADOW_MASK_SURFACES` is `pub` with no reader outside the file | L5-R10 |
| `BindGroupDefinition` (`bind_groups.rs:396`) with `Binding`, `Writer` and the free functions | 14 public definitions; 3 public fields and 5 public functions on the definition; 6 public fields on `Binding`; 2 variants; `pipeline_layout`; `uniform` and `read_only_storage` `pub(crate)` | each definition 5 to 8 sites in its pass; `.bind_group(` 14, `.layout(` 13, `pipeline_layout(` 11, `.visible_to(` 3 (all `main_pass.rs`), `.layout_of(` 2 (both `depth_pyramid_pass.rs`), `.declarations()` 1 (`shaders/mod.rs`); `uniform` 4 and `read_only_storage` 2, all in `shader_tests.rs`; `Writer` read at `shader_tests.rs:668` only | marginally: `pub(crate)` on two helpers that only a sibling test module reads (`pub(super)` would serve); not filed, see Notes | |
| `AccelerationStructureBytes` | 6: fields `per_blas_triangle`, `per_compacted_blas_triangle`, `per_tlas_instance`; functions `estimate`, `blases`, `tlas` | `estimate` 1 (`renderer.rs:433`); `blases` 1 (`pools.rs:485`); `tlas` 0 direct, reached through the two `estimated_bytes`; the fields 1, a test literal (`pools.rs:696-700`) | by the three `pub` fields, which only a test outside the file reads; not filed, 3 fields, 1 site | `src/render/acceleration_structures.rs:35-40` |
| `StaticTlas` | 6: `new`, `update`, `binding`, `anchor`, `instances_left_out`, `estimated_bytes` | 6 sites: `renderer.rs:357, 493, 495, 499, 794`; `shadow_mask_pass.rs:128` | no; each function has a caller | `src/render/acceleration_structures.rs:107-207` |
| `DynamicTlas` | 7: `new`, `fill`, `record`, `prepare_encoder`, `binding`, `estimated_bytes`, `gpu_memory_bytes` | 7 sites: `renderer.rs:358, 490, 502, 610, 780, 795`; `shadow_mask_pass.rs:129` | no; each function has a caller | `src/render/acceleration_structures.rs:216-434` |
| `BlasRefit` | 3: `new`, `submit`, `gpu_memory_bytes` (plus the constant `BLAS_REFIT_PASS`) | 3 sites: `renderer.rs:359, 598, 774`; `BLAS_REFIT_PASS` at `renderer.rs:38, 597` and `readback.rs:20` | no | `src/render/blas_refit.rs:22-163` |
| `GpuStatsReadback` | 6: `new`, `begin_frame`, `shape`, `end_frame`, `flush`, `read_back` | 6 sites: `renderer.rs:395, 584, 698, 705, 733, 739` | yes, by `shape`: one caller, always immediately before `end_frame` (L5-R15) | `src/render/readback.rs:75-482` |
| `FrameTimestamps` | 4: `compute_pass`, `render_pass`, `bracket`, `marker_after` | 12 sites in `renderer.rs`: `compute_pass` 6 (586, 626, 632, 649, 656, 669), `render_pass` 4 (645, 665, 679, 684), `bracket` 1 (607), `marker_after` 1 (597) | no; the two single-caller constructors produce timestamp shapes the other two cannot | `src/render/readback.rs:154-556` |
| `FrameShape` | 3 fields: `dynamic_tlas_built`, `bent`, `blas_refitted` | 1 site constructs it (`renderer.rs:698-704`); read in `frame_times` | no | `src/render/readback.rs:117-130` |
| `CullingPass` | 8 methods (`new`, `record`, `bind_render_targets`, `instance_slot_count`, `meshlet_records`, `packed_indices`, `frame_draw`, `gpu_memory_bytes`); beside it `CullingPhase` and `label`, `SCAN_BLOCK_SIZE`, `MAX_WORKGROUPS_PER_DIMENSION`, `PHASE_TWO_DRAW_OFFSET`, `DrawCapacity` with 2 fields and `PROVISIONAL`, `FrameDraw` with 4 fields and 2 methods: 24 in the file | `renderer.rs`: `new` 1, `record` 2, `bind_render_targets` 1, `instance_slot_count` 2, `meshlet_records` 1 (+2), `packed_indices` 3 (+1), `frame_draw` 4 (+3), `gpu_memory_bytes` 1 | no: each getter hands the Renderer a buffer it binds, uploads to or reads back; the three getters the main pass's record calls carry are the subject of L5-R20, filed against the main pass | `src/render/culling_pass.rs:123` |
| `DepthPyramidPass` | 3 methods (`new`, `bind_render_targets`, `record`); `DEPTH_PYRAMID_PASS`, `DEPTH_PYRAMID_TILE_SIZE`: 5 in the file; 3 `pub(crate)` test helpers | `renderer.rs`: `new` 1, `bind_render_targets` 1, `record` 1 | no | `src/render/depth_pyramid_pass.rs:22` |
| `MainPass` | 5 methods (`new`, `bind_render_targets`, `record_depth_prepass`, `record_phase_two_depth`, `record_lit_pass`); `DEPTH_PREPASS`, `PHASE_TWO_DEPTH_PASS`, `LIT_PASS`: 8 in the file | `renderer.rs`: each method 1 | yes: `record_depth_prepass`, `record_phase_two_depth` and `record_lit_pass` each take `packed_indices` and `frame_draw`, and the one caller passes `self.culling_pass.packed_indices()` and `self.culling_pass.frame_draw()` all three times, while `meshlet_records` from the same pass is cloned at construction (L5-R20); two record methods of one shape (L5-R19) | `src/render/main_pass.rs:28` |
| `FroxelPass` | 5 methods (`new`, `record`, `froxel_lists`, `light_counts`, `gpu_memory_bytes`); 8 constants; `LightCounts` with 4 fields: 18 in the file | `renderer.rs`: `new` 1, `record` 1, `froxel_lists` 2 (+1), `light_counts` 1, `gpu_memory_bytes` 1; `light_count_pass.rs`: `froxel_lists` 1, `light_counts` 1 | no | `src/render/froxel_pass.rs:46` |
| `ShadowMaskPass` | 5 methods (`new`, `bind_render_targets`, `record`, `shadow_mask_lights_overflowed`, `gpu_memory_bytes`); `SHADOW_MASK_PASS`, `SHADOW_MASK_TILE_SIZE`, `SHADOW_MASK_LIGHTS`, `SAMPLE_OFFSETS`, `SHADOW_EDGE_LIGHTS`: 10 in the file | `renderer.rs`: each method 1 | no on width; `new`'s `buffers: [(&'static str, &wgpu::Buffer); 4]` makes the Renderer spell four of the pass's binding names (`renderer.rs:383` to `388`), weighed in Notes | `src/render/shadow_mask_pass.rs:71` |
| `TonemappingPass` | 4 methods (`new`, `bind_render_targets`, `gpu_memory_bytes`, `record`); `TONEMAPPING_PASS`, `DITHER_MASK_SIDE`: 6 in the file | `renderer.rs`: each method 1 | no | `src/render/tonemapping_pass.rs:29` |
| `BendPass` | 4 methods (`new`, `record`, `bends`, `gpu_memory_bytes`); `BEND_PASS`, `BEND_CAPACITY`, `BEND_WORKGROUP_SIZE`; `BendData` with 5 fields: 13 in the file | `renderer.rs`: `new` 1, `record` 1, `bends` 2, `gpu_memory_bytes` 1; `prepare.rs`: `BEND_CAPACITY` 4, `BendData` 3 | no | `src/render/bend_pass.rs:41` |
| `LightCountPass` | 3 methods (`new`, `bind_render_targets`, `record`); `LIGHT_COUNT_TILE_SIZE`: 4 in the file | `renderer.rs`: each method 1 | no | `src/render/light_count_pass.rs:19` |



`src/lib.rs:30` declares `mod render;` privately, so `pub` and `pub(crate)` inside the module reach the same code; the mixed spellings (`pub struct DrawCapacity`, `pub const BEND_CAPACITY` beside `pub(crate) const SCAN_BLOCK_SIZE`) change nothing and are not filed.

### Findings

| id | claim | severity | where | evidence | expected |
|---|---|---|---|---|---|
| L5-R01 | `Renderer::device` and `Renderer::queue` are `pub(crate)` though nothing outside `renderer.rs` reads them. | health; 2 fields, 0 outside call sites | `src/render/renderer.rs:168-169` | block below | Private fields: the Renderer's interface is what its callers use (CLAUDE.md, deep modules behind small interfaces). |
| L5-R02 | `mod.rs` re-exports 12 names that no file outside `src/render` imports, while the module's own files reach the same items by two paths. | health; 12 names, 2 import paths for 1 item | `src/render/mod.rs:28-38` | block below | A re-export list that is the module's interface to the rest of the crate; the module's files import from the submodule, so one path per item. |
| L5-R03 | `view_proj` abbreviates `projection`, against the full-word rule. | health; 50 sites in 12 files (27 Rust, 23 WGSL) | `src/render/uniforms.rs:45` | block below | `view_projection` (CLAUDE.md, Code conventions: identifiers are full words, WGSL included; `tests/conventions.rs` catches only the three shading spellings). |
| L5-R04 | The `gpu_frame_stats` doc names `upload` as what reads stats back, but `render` does, through `GpuStatsReadback::begin_frame`. | drift; not load-bearing | `src/render/renderer.rs:730-731` | block below | "The GPU stats of earlier frames that the last `render` or `flush_gpu_frame_stats` read back", as `readback.rs:332` says of `begin_frame` and `flush`. |
| L5-R05 | The `RENDER_PASSES` doc says the other timed passes are compute passes, but two of the eight are the dynamic TLAS's build and the BLAS refit, timed by markers. | drift; not load-bearing | `src/render/renderer.rs:54` | block below | "The render passes of `TIMED_PASSES`; the others are compute passes, but the dynamic TLAS's build and the BLAS refit, which markers time." |
| L5-R06 | The doc on `DYNAMIC_TLAS_RECORDING` describes two stages, recording and submit, while the constant is the recording alone and `DYNAMIC_TLAS_SUBMIT` carries no doc. | drift; not load-bearing | `src/render/renderer.rs:62-70` | block below | One doc per constant, or the two constants adjacent under one doc that names both. |
| L5-R07 | The comment "submit will accept anything that implements IntoIter" describes wgpu's API, not this code. | health; 1 line | `src/render/renderer.rs:724` | block below | Comments say what the code does and why (CLAUDE.md, Code conventions); nothing here needs saying, so no comment. |
| L5-R08 | The test module builds the same white matte `MaterialData` literal five times where a helper beside `emissive_material` would. | health; 50 lines, 5 sites, about 40 removable | `src/render/renderer.rs:1232-1241` | block below | One `white_material(renderer)` helper, as `emissive_material` (`renderer.rs:2603-2615`) already is for the emissive case. |
| L5-R09 | The test module builds the camera at the origin looking down -Z as a `CameraUniform` literal four times, and the same matrix twice more as a local. | health; 6 sites, about 24 lines | `src/render/renderer.rs:1069-1073` | block below | One helper, or a `frame_of` variant that takes the camera, as `frame_of` (`renderer.rs:2709-2735`) already builds the identity camera. |
| L5-R10 | `SHADOW_MASK_SURFACES` is `pub` yet nothing outside `targets.rs` reads it; its one use is the layer count of the shadow mask texture in the same file. | health; 1 line, 0 callers outside the file | `src/render/targets.rs:33` | block below | A constant with one reader in its own file is private, or, if the shader is to share it as it shares `SAMPLE_COUNT`, it is declared through `shaders/mod.rs`. |
| L5-R11 | The `SHADOW_MASK_BUILD` doc lists what the group held before the surface normals and the shadow edges joined it: 4 of its 14 bindings are not described. | drift; not load-bearing | `src/render/bind_groups.rs:198-203` | block below | `CLAUDE.md`, Code conventions: comments say what the code does. The doc names every binding the group holds, as the other group docs in the file do. |
| L5-R12 | The comment on the depth target's creation names one of its three readers. | drift; not load-bearing | `src/render/targets.rs:156` | block below | The comment that justifies `TEXTURE_BINDING` names the passes that bind the depth target, or defers to the getter's doc at `targets.rs:239`, "as passes read it". |
| L5-R13 | The `final_depth_attachment` doc carries a measured cost, "about 50 µs at 1440p on the M4 Pro", that nothing in the tree re-measures or checks. | drift; not load-bearing | `src/render/targets.rs:314-315` | block below | `CLAUDE.md`, Code conventions: comments say what the code does and why; the why here is "a stored depth writes the whole target back", which holds without the figure. Measured costs live in the Bench history and Baselines. |
| L5-R14 | About 330 of `coverage_tests.rs`' 828 lines exist only for `coverage_experiments`, a test marked `#[ignore = "prints a table and asserts nothing"]` that nothing runs | health; about 330 lines, 0 automated callers | `src/render/coverage_tests.rs:677` | block below | `CLAUDE.md`, Project posture: "No throwaway code"; the appendix's axis "tests that pin nothing". The smaller form keeps the one asserting test and the helpers it reaches: `tilted_quads`, `every_triangle` with `Indexing::Shared`, `vertices` as positions, `rasterise` with `IdSource::Primitive`, `same_quad`, `compaction_changes`, `coverage_context` without the 2x feature |
| L5-R15 | `GpuStatsReadback::shape` is a public method with one caller that always calls it immediately before `end_frame`, and both repeat the same `Recorded { frame } if frame == self.frame` guard | health; 1 method, 1 call site, 1 duplicated guard | `src/render/readback.rs:255` | block below | `CLAUDE.md`, How agents work here: "deep modules behind small interfaces". `end_frame` takes the `FrameShape` as a parameter and sets it under its existing guard; the readback's interface drops to 5 functions |
| L5-R16 | The doc comment on `BlasRefit::submit` describes the `Refit` form only: for `CrowdForm::Rebuild` the update is recorded into `head`, not "after" it, and one command buffer is submitted, not "both" | drift; not load-bearing | `src/render/blas_refit.rs:90` | block below | `CLAUDE.md`, Code conventions: comments say what the code does. The comment names both forms: the rebuild recorded into `head` and submitted with it, the refit recorded after `head` and submitted beside it |
| L5-R17 | Four locals are named by a letter and a digit, `q0`, `q1`, `q2`, `q3`, in a function whose body spans 50 lines | health; 4 bindings, 1 site | `src/render/coverage_tests.rs:518` | block below | `CLAUDE.md`, Code conventions: "Identifiers are full words"; the appendix allows single letters only as a short loop's index or a coordinate. Index the quad's array directly, or name the bindings `first_corner`, `second_corner`, `third_corner`, `fourth_corner` |
| L5-R18 | Six passes each spell out the same eleven-line `wgpu::ComputePipelineDescriptor` (label from the entry point, one pipeline layout, one module, the entry point, compilation options, `cache: None`) that one shared function would build. | health; about 70 lines over 6 sites in this letter, 8 crate-wide | `src/render/culling_pass.rs:248` (first site) | block below | `CLAUDE.md`, How agents work here: "Prefer deep modules behind small interfaces"; the appendix names pipeline creation as a shape repeated across passes. One `compute_pipeline(device, layout, module, entry_point, constants)` beside `pipeline_layout` in `src/render/bind_groups.rs:529`, called once per pipeline. |
| L5-R19 | `MainPass::record_depth_prepass` and `MainPass::record_phase_two_depth` are one function written twice: same pipeline, same bind-group filter, same index buffer, differing only in label, attachments and the draw's offset. | health; 67 lines where about 35 serve, 2 call sites | `src/render/main_pass.rs:231` and `src/render/main_pass.rs:270` | block below | One `record_depth_pass(encoder, render_targets, bind_groups, ..., phase: CullingPhase, timestamp_writes)` choosing label, attachments and offset by the phase, as `CullingPass::record` already does with `CullingPhase`. |
| L5-R20 | The three `MainPass::record_*` methods take `packed_indices` and `frame_draw` on every call although their one caller always passes the culling pass's two fixed buffers, while the third culling buffer, `meshlet_records`, is already cloned into the pass at construction. | health; 6 parameter sites, 3 call sites, one `#[expect(clippy::too_many_arguments)]` | `src/render/main_pass.rs:236` | block below | `MainPass::new` takes the culling pass's `packed_indices` and `frame_draw` beside `meshlet_records` (`src/render/main_pass.rs:55`) and clones them as it clones that one (`src/render/main_pass.rs:60`); the record methods drop two parameters each and `record_lit_pass` loses the lint expectation at `src/render/main_pass.rs:305`. |
| L5-R21 | The doc comment of `CullingPass::gpu_memory_bytes` lists five buffers, but the sum beneath it also counts the Meshlet record and packed index buffers, 140 MiB at `PROVISIONAL`. | drift; load-bearing | `src/render/culling_pass.rs:356` | block below | A comment that says what the code does: "Bytes of every buffer of the frame's draw: the Meshlet records, packed indices, indirect arguments and their stand-in, and the Instance slot count, offsets and visibility." |


#### L5-R01

`src/render/renderer.rs:168-169`:

```rust
pub(crate) device: wgpu::Device,
pub(crate) queue: wgpu::Queue,
```

`grep -rn 'renderer\.device\|renderer\.queue' src tests` finds 0 lines. The only destructure of a `Renderer` outside the file, `src/state.rs:150-154`, takes `mesh_pool`, `material_pool` and `load_staging` and leaves the rest with `..`. Inside the file both fields are used only through `self.device` and `self.queue`. The tests in `bend_pass.rs` that hold a `Renderer` read `context.device`, not the Renderer's. The other three `pub(crate)` fields are read: `mesh_pool` by `src/state.rs:265,306` and `src/render/bend_pass.rs:234-235,258`, `material_pool` and `load_staging` by the `state.rs:150` destructure and `bend_pass.rs:233`. Fix: drop `pub(crate)` from the two fields. Cannot move a pixel.

#### L5-R02

`src/render/mod.rs:28-38` re-exports 26 names. For each, `grep -rlw <name> src tests --include='*.rs' | grep -v '^src/render/'` finds no file for these 12: `STATIC_TLAS_CAPACITY`, `DynamicTlas`, `StaticTlas`, `INSTANCE_CAPACITY`, `InstanceData`, `NO_MESH`, `FrameData`, `FrameStaging`, `CameraUniform`, `LIGHT_CAPACITY`, `LightData`, `LightUniform`. Their only consumers are the module's own files, which import them through the re-export (`renderer.rs:18-22` imports `CameraUniform`, `DynamicTlas`, `FrameData`, `FrameStaging`, `INSTANCE_CAPACITY`, `InstanceData`, `LIGHT_CAPACITY`, `LightData`, `LightUniform`, `LoadStaging`, `STATIC_TLAS_CAPACITY`, `StaticTlas` from `crate::render::{...}`) while other files of the module import the same kinds of item by their submodule path (`shadow_mask_pass.rs:14` takes `DynamicTlas`, `StaticTlas` from `crate::render::acceleration_structures::`; `culling_pass.rs:19`, `main_pass.rs:14`, `shadow_mask_pass.rs:17` and `shaders/mod.rs:25` take `RenderSwitches` from `crate::render::renderer::`, where `state.rs:16`, `capture/mod.rs:4` and `bench/mod.rs:21` take it from `crate::render::`). So `RenderSwitches` is imported by two paths, 4 sites each way, and the re-export list does not tell a reader which items the rest of the crate depends on. The 14 names with an outside consumer: `AccelerationStructureBytes`, `NO_MATERIAL` and `LoadStaging` (`assets/pools.rs:4`, `assets/material.rs:8`, `assets/loader.rs:5`), `DYNAMIC_TLAS_CAPACITY` (`scenes/town.rs:20`), `FramePreparer`, `Renderer`, `CrowdForm`, `RenderSwitches` (`state.rs:16`, `lib.rs:43`), `DEPTH_FORMAT`, `HDR_FORMAT` (`gpu/mod.rs:257`), `TIMED_PASSES`, `BLAS_REFIT`, `DYNAMIC_TLAS_RECORDING`, `DYNAMIC_TLAS_SUBMIT` (`bench/mod.rs:21`, `stats/mod.rs:6`), and `FRAMES_IN_FLIGHT` (`capture/mod.rs:4`, `lib.rs:165`). `lib.rs:30` keeps `mod render` private, so `pub` here is crate-wide visibility. Fix: drop the 12 re-exports and import them in `renderer.rs` from their submodules; pick one path for `RenderSwitches` inside the module. Cannot move a pixel.

#### L5-R03

`src/render/uniforms.rs:45`:

```rust
pub view_proj: [[f32; 4]; 4],
```

`proj` stands for projection; the doc two lines above spells it out ("Projection times the view rotation"). `grep -rnw view_proj src tests scripts` counts 50 lines: `renderer.rs` 16, `shader_function_tests.rs` 5, `coverage_tests.rs` 2, `prepare.rs` 2, `shader_tests.rs` 1, `uniforms.rs` 1 (27 Rust); `shaders/common.wgsl` 6, `shadow_mask.wgsl` 7, `froxel.wgsl` 3, `light_count.wgsl` 3, `culling.wgsl` 2, `main.wgsl` 2 (23 WGSL, which fall in shaders). `shader_tests.rs:214-225` matches Rust fields to WGSL fields by `stringify!($field)`, so the two halves rename together or the test fails. `tests/conventions.rs:190-214` catches only `n_dot_v`, `NdotV` and `NoV` spellings, not this. The brief's allowed list (`url`, `id`, `cpu`, `gpu`, `wgpu`, `glam`, `blas`, `tlas`, `msaa`, `hdr`, `srgb`, `ev100`, API names as spelled) does not hold `proj`; glam's own name for the matrix is `perspective_infinite_reverse_rh`, no `proj`. Fix: rename to `view_projection` in Rust and WGSL. Cannot move a pixel (a rename; the shader tests pin the layout).

#### L5-R04

`src/render/renderer.rs:730-731`:

```rust
/// The GPU stats of earlier frames that the last `upload` or
/// `flush_gpu_frame_stats` read back, by frame number, counted from 0.
pub fn gpu_frame_stats(&self) -> &[GpuFrameStats] {
```

`upload` (`renderer.rs:476-562`) never touches `self.gpu_stats`. `render` does, at `renderer.rs:584`, `let timestamps = self.gpu_stats.begin_frame(&mut encoder, recorder);`, and `readback.rs:212-215` documents `begin_frame` as what "reads back the slot it takes"; `readback.rs:332` documents `read_back` as "The stats the last `begin_frame` or `flush` read back". Not load-bearing: every caller renders right after uploading (`state.rs:270` then `state.rs:280-282`; `lib.rs:248-253`; `capture/mod.rs:260-261,432-433`) and reads stats after rendering, so the stale frame the doc would let a reader expect never shows. `state.rs:287-288` repeats the error one level up ("the last `update` or `flush_gpu_frame_stats`"); that file falls outside the render module (the orchestrator, lane 5's sample or lane 7).

#### L5-R05

`src/render/renderer.rs:54`:

```rust
/// The render passes of `TIMED_PASSES`; the others are compute passes.
```

`TIMED_PASSES` (`renderer.rs:35-48`) holds 12 names; 4 are in `RENDER_PASSES`. Of the other 8, `DYNAMIC_TLAS_BUILD` is the dynamic TLAS's build, timed by `timestamps.bracket(DYNAMIC_TLAS_BUILD)` at `renderer.rs:605-607`, and `BLAS_REFIT_PASS` is the frame's BLAS updates, timed by `timestamps.marker_after(BLAS_REFIT_PASS)` at `renderer.rs:597`; neither goes through `FrameTimestamps::compute_pass` (`readback.rs:500-508`, "a compute pass of `TIMED_PASSES`"), and `readback.rs:654-658` times both apart from the compute passes. Not load-bearing: `readback.rs:643` uses `RENDER_PASSES` only to pick which passes count from an earlier end, which is right for both.

#### L5-R06

`src/render/renderer.rs:62-70`:

```rust
/// The CPU stages of the dynamic TLAS's build: its recording and its
/// submit, the CPU cost ADR 0020 caps.
pub(crate) const DYNAMIC_TLAS_RECORDING: &str = "dynamic_tlas";

/// The CPU stage that records the frame's BLAS updates, with the crowd's
/// refit or rebuild form, and submits them with the frame's commands before
/// them.
pub(crate) const BLAS_REFIT: &str = "blas_refit";
pub(crate) const DYNAMIC_TLAS_SUBMIT: &str = "dynamic_tlas_submit";
```

The doc at 62-63 is a doc comment on `DYNAMIC_TLAS_RECORDING` alone, which `renderer.rs:491,503,615` use as the recording stage; the submit stage is `DYNAMIC_TLAS_SUBMIT` (`renderer.rs:721`), which has no doc and sits after `BLAS_REFIT`'s. A reader of `DYNAMIC_TLAS_RECORDING`'s doc is told it is two stages. Not load-bearing: `bench/mod.rs` reads all three by name and sums what it needs.

#### L5-R07

`src/render/renderer.rs:724-725`:

```rust
// submit will accept anything that implements IntoIter
let submission = self.queue.submit(std::iter::once(command_buffer));
```

The comment states a fact about `wgpu::Queue::submit`'s signature, not about this code or why it does what it does; it is the wording of the wgpu beginner tutorial that `main_pass.rs:319` also carries ("This is what @location(0) in the fragment shader targets"). `grep -rn 'implements IntoIter' src` finds this one line. Fix: delete the line. Cannot move a pixel.

#### L5-R08

`grep -n 'layers: \[MaterialPool::DEFAULT_LAYER; 4\]' src/render/renderer.rs` finds 6 sites: `2611` inside `emissive_material`, and 5 copies of this literal:

```rust
let white = renderer.material_pool.insert(
    &mut renderer.load_staging,
    MaterialData {
        base_color: [1.0; 4],
        metallic_roughness: [0.0, 1.0, 0.0, 0.0],
        emissive: [0.0, 0.0, 0.0, 1.0],
        layers: [MaterialPool::DEFAULT_LAYER; 4],
        extra: [MaterialPool::DEFAULT_LAYER, 0, 0, 0],
    },
);
```

at `renderer.rs:1232-1241`, `1370-1379`, `1446-1455`, `1871-1880` and `1953-1962`, byte for byte the same but for indentation. `emissive_material` (`renderer.rs:2601-2615`) is the helper form for the black emissive Material; the white matte one has none. A `white_material(renderer: &mut Renderer) -> MaterialHandle` helper replaces 50 lines with 5 calls. Tests only: cannot move a pixel.

#### L5-R09

`grep -n 'projection_matrix(1.0) \* Camera::new().view_matrix()' src/render/renderer.rs` finds 6 sites. Four are the literal

```rust
let camera = CameraUniform {
    view_proj: (projection_matrix(1.0) * Camera::new().view_matrix()).to_cols_array_2d(),
    exposure: 1.0,
    ..CameraUniform::default()
};
```

at `renderer.rs:1069-1073`, `1208-1212`, `1344-1348` and `1480-1485`; two bind the matrix as `view_proj` (`1863`, `1950`) and build the same `CameraUniform` from it at `1899-1903` and `1974-1978`. `frame_of` (`renderer.rs:2709-2735`) already builds the identity camera at exposure 1, and every one of the six frames then overrides it with `FrameData { camera, ..frame_of(...) }`. A helper, or `frame_of` taking the camera, replaces about 24 lines. Tests only: cannot move a pixel.

#### L5-R10

`src/render/targets.rs:33`:

```rust
pub const SHADOW_MASK_SURFACES: u32 = 2;
```

Its uses:

```
grep -rn "SHADOW_MASK_SURFACES" . | grep -v '^./target' | grep -v '^./.git/'
docs/guides/edge-shadows.html:347   (a quoted copy of the declaration in a field guide)
src/render/targets.rs:33            (the declaration)
src/render/targets.rs:190           (the layer count of the shadow mask texture)
```

Zero readers outside the file in `src/`, `tests/`, `scripts/` or the shaders. The shaders write and read layer 1 as the second surface without a constant named from Rust: `shaders/mod.rs:248-267` declares `LUMINANCE_COEFFICIENTS` and `SAMPLE_COUNT` from `targets.rs` into every module but not this one. The `pub` serves no caller, so the interface is one item wider than its callers need. The smaller form is a private `const`; the deeper form, if the shader's hardcoded layer index is to follow it, is a declaration through `shaders/mod.rs` as `SAMPLE_COUNT` has. Either is a visibility change or a constant plumbing with the same value; neither moves a pixel.

#### L5-R11

`src/render/bind_groups.rs:198-203`, the doc:

```
/// Group 3 of the shadow mask pass, which builds the mask: the camera, the
/// lights and the Froxel lists, which `FRAME` holds, the depth target, and
/// what the pass writes: the shadow mask, which samples show its second
/// surface, how many lights it had no bit for, which the CPU reads back, and
/// the list of second surfaces its first dispatch writes for its second,
/// with that dispatch's size.
```

The bindings at `bind_groups.rs:207-263`: `camera`, `light`, `lights`, `froxel_lists`, `depth_target`, `surface_normals` (`:226-232`), `shadow_mask`, `shadow_mask_samples`, `shadow_mask_lights_overflowed`, `second_surfaces`, `second_surface_dispatch`, `directional_verdicts` (`:260`), `shadow_edges` (`:261`), `shadow_edge_dispatch` (`:262`). The doc describes ten; `surface_normals`, `directional_verdicts`, `shadow_edges` and `shadow_edge_dispatch` are absent, and "its first dispatch writes for its second" describes a two-dispatch pass where the bindings show a third list and dispatch for shadow edges. The `RenderTargets` field docs at `targets.rs:81-93` describe each of the four, so the facts exist one file over; the group doc is the stale copy. Not load-bearing: a reader binding the group goes by the binding names, which `bind_group` resolves by name. A doc edit; no pixel moves.

#### L5-R12

`src/render/targets.rs:156-162`:

```rust
// Read by the depth pyramid pass.
depth: target(
    "Depth Texture",
    DEPTH_FORMAT,
    SAMPLE_COUNT,
    wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
),
```

The readers that bind the depth target:

```
grep -rn --include='*.rs' -F ".depth()" src | grep -v '^src/render/targets.rs'
src/render/depth_pyramid_pass.rs:87:   ("depth", render_targets.depth()),
src/render/shadow_mask_pass.rs:308:    wgpu::BindingResource::TextureView(render_targets.depth()),
src/render/light_count_pass.rs:122:    wgpu::BindingResource::TextureView(render_targets.depth()),
```

plus the test read-backs at `depth_pyramid_pass.rs:185`, `shadow_mask_pass.rs:429` and the size reads at `light_count_pass.rs:76`, `:84`. The comment explains `TEXTURE_BINDING` by one reader; the `SHADOW_MASK_BUILD` and `LIGHT_COUNT_INPUT` groups (`bind_groups.rs:219-225`, `:323-329`) bind it too. The getter's own doc at `targets.rs:239`, "The depth target, as passes read it", is the accurate form. Not load-bearing: a reader removing the binding usage on this comment's word would hit a validation error at the first shadow mask bind group, not a wrong image. A comment edit; no pixel moves.

#### L5-R13

`src/render/targets.rs:312-315`:

```
/// The depth target as the attachment of the frame's last pass that
/// draws depth: loaded, and discarded unless `kept` for a later pass to
/// read. A pass that writes depth and stores it writes the whole target
/// back, about 50 µs at 1440p on the M4 Pro.
```

The first sentence says what the code does (`targets.rs:316-329`: `Load`, then `Store` or `Discard` by `kept`); the second says why `Discard` is the default. The figure is a measurement from one adapter at one resolution at one commit, with no Baseline or test pinning it, so it ages silently as the pass, the driver or the resolution changes; the lane's comment-accuracy axis names "a number in a shipping comment" as this kind. The why stands without it: "A pass that writes depth and stores it writes the whole target back." Not load-bearing: the number steers no decision a reader would make differently at 40 or 80 µs. A comment edit; no pixel moves.

#### L5-R14

`src/render/coverage_tests.rs:676-678`:

```rust
#[test]
#[ignore = "prints a table and asserts nothing"]
fn coverage_experiments() {
```

The file holds two tests. `a_draw_of_whole_index_sharing_groups_keeps_every_sample` (lines 654-669) asserts, and guards the ADR 0019 line on the M4 Pro's fragment merging. `coverage_experiments` (lines 671-828, 158 lines) prints the table that `docs/performance.md:1906-1962` already records, and asserts nothing, as its own attribute says. Nothing runs it: `grep -rn 'coverage_experiments' src tests scripts docs` finds its definition (line 678), its doc comment (line 672) and the command in `docs/performance.md:1933`, which a person types.

Items with no use outside the ignored test, by `grep -n` over the file (line numbers are every occurrence; the first is the definition):

- `Shading` (lines 178-182) and the `sample_input` match (223-226): `Shading::Sample` at 225 (the match arm) and 709 (an experiment row). The asserting test passes `Shading::Pixel` (line 616).
- Seven `IdSource` variants (lines 190-205): `Flat` 229, 756; `Interpolated` 230, 716; `Smooth` 231, 722; `PrimitiveBesideFlat` 232, 369, 734; `PrimitiveBesideSmooth` 233, 370, 728; `BothKeepingPrimitive` 236, 371, 503, 740; `BothKeepingWeights` 236, 372, 504, 746. Lines 369-372 and 503-504 are arms that exist for them. The asserting test passes `IdSource::Primitive` (line 617).
- `Indexing::SharedRotated` (lines 41-42, arm at 57): 750, 754. `Indexing::Own` (lines 43-46, arm at 58): 700, 825. The asserting test passes `Indexing::Shared` (lines 659, 663).
- `vertices`' `weights`, `leads` and `quad` columns (lines 70-73, 76-78, 81, 87-90) and the WGSL `Varyings` fields `weights`, `leads`, `quad` (lines 246-251, 258): read only by the `Flat`, `Interpolated`, `Smooth`, `PrimitiveBeside*` and `BothKeeping*` ID sources.
- `sample_positions` (lines 97-111): called at 514, inside `analytic`.
- `sequence` (lines 113-122): called at 127, inside `flat_quads`.
- `flat_quads` (lines 124-152): called at 682, inside `coverage_experiments`.
- `analytic` (lines 510-566, 57 lines): called at 799 and 812, inside `coverage_experiments`.
- The 2x MSAA device feature: `TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES` requested at 580, read at 781 as `has_2x`, used at 786 for the table's 2x column. The asserting test rasterises at 4x only (line 615).
- `Raster.draws: Vec<Vec<Triangle>>` (line 210): more than one draw only in the rows at 771 and 775; the asserting test passes `vec![draw]` (line 614).

Counted: 3 + 10 + 15 + 10 + 29 + 9 + 45 + 57 + 158, about 336 lines, 40% of the file, leaving about 490 lines for the asserting test and its path.

The fix cannot move a pixel: the file is `#[cfg(test)]` (`src/render/mod.rs:21-22`) and ships nothing. The counter-argument the triage should weigh: the experiment is the reproducible form of the measurement that amended ADR 0019 (`docs/adr/0019-*.md:16`, "measured, `docs/performance.md`"), and a reader on new hardware or a new wgpu could re-run it; the asserting test guards the ADR's line on the M4 Pro alone. If the owner wants the experiment kept, `wontfix` with that reason, and the finding records that the file carries it on purpose.

#### L5-R15

`src/render/readback.rs:253-260` and `266-274`:

```rust
    /// Records which of the timed work that runs in some frames only the
    /// frame `begin_frame` started ran.
    pub fn shape(&mut self, shape: FrameShape) {
        let slot = &mut self.slots[slot_index(self.frame)];
        if matches!(slot.state, SlotState::Recorded { frame } if frame == self.frame) {
            slot.shape = shape;
        }
    }
    ...
    pub fn end_frame(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame_draw: &wgpu::Buffer,
        light_counts: &wgpu::Buffer,
        shadow_mask_lights_overflowed: &wgpu::Buffer,
    ) {
        let slot = &self.slots[slot_index(self.frame)];
        if matches!(slot.state, SlotState::Recorded { frame } if frame == self.frame) {
```

`grep -rn '\.shape(' src tests` finds one caller, `src/render/renderer.rs:698`, and `grep -rn '\.end_frame(' src tests` one, `src/render/renderer.rs:705`; the renderer calls them back to back (lines 698-710) with only the `FrameShape` literal between. Every value the shape needs (`dynamic_tlas_commands`, `bent`, `blas_refitted`) is known before `end_frame` runs. The same guard, that the slot is `Recorded` for this frame, is evaluated twice. The readback test at lines 942-960 calls `end_frame` without `shape`, so the test path already treats the shape as optional, which a `FrameShape` parameter (passing `FrameShape::default()`) serves.

The fix cannot move a pixel or a counter: the same shape value reaches the same slot in the same frame; `frame_times` reads it unchanged. Timings are a CPU method call fewer per frame, within noise.

#### L5-R16

`src/render/blas_refit.rs:90-94`, the comment:

```rust
    /// Records the update of `mesh_pool`'s refitted BLASes after `head`,
    /// the frame's commands up to its bend pass, submits both through
    /// `queue`, and orders what `queue` submits next after them. A refit
    /// is followed by a marker that writes `refit_end`; returns whether it
    /// was.
```

beside `src/render/blas_refit.rs:103-106` and `123-130`, the code:

```rust
        if self.form == CrowdForm::Rebuild {
            mesh_pool.rebuild_refitted_blases(&mut head);
        }
        let head = head.finish();
        ...
        } else {
            (queue.submit(std::iter::once(head)), false)
        };
        #[cfg(not(target_vendor = "apple"))]
        let (submission, refit_timed) = {
            let _ = refit_end;
            (queue.submit(std::iter::once(head)), false)
        };
```

For `CrowdForm::Rebuild`, the only form off Metal (`src/render/renderer.rs:133-134` refuses `Refit` elsewhere), the rebuild is recorded into `head` before it is finished, and `head` alone is submitted; "after `head`" and "submits both" hold for `CrowdForm::Refit` alone (lines 112-122). The last clause, "orders what `queue` submits next after them", holds for both forms on Metal, since `Order` is made for the backend, not the form (lines 64-66). The struct's own doc (lines 19-21) and the module doc (lines 1-7) are right about both forms.

Not load-bearing: a reader acting on it would place the rebuild in a second command buffer, which costs a submission but renders the same. No pixel moves either way.

#### L5-R17

`src/render/coverage_tests.rs:518-523`:

```rust
        let [q0, q1, q2, q3] = quads[id as usize / 2];
        let corners = if id % 2 == 0 {
            [q0, q1, q2]
        } else {
            [q0, q2, q3]
        }
```

`grep -n '\bq[0-3]\b' src/render/coverage_tests.rs` finds the bindings at line 518 and their uses at 520 and 522, plus the notation in comments at lines 26-27, 31 and 72-73, where the rule allows a textbook symbol. The function `analytic` (lines 513-566) runs 54 lines, past the few-line body the full-word rule allows a letter for. The quad is `[[f32; 4]; 4]`, so the same selection reads `[quad[0], quad[1], quad[2]]` and `[quad[0], quad[2], quad[3]]` with no bindings, or names them by their role as the doc at line 26 describes them, the quad's four corners in draw order.

The fix cannot move a pixel: a test-only file, and a rename or an index rewrite changes no value.

#### L5-R18

The six shipping sites among the passes, each creating a compute pipeline from a `wgpu::ComputePipelineDescriptor` with the same six fields:

```
$ grep -rn 'create_compute_pipeline' src --include='*.rs' | grep -v 'tests\|coverage_tests\|shader_function_tests\|pools.rs'
src/render/culling_pass.rs:248:                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
src/render/bend_pass.rs:69:        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
src/render/froxel_pass.rs:95:        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
src/render/acceleration_structures.rs:272:        let marker_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
src/render/depth_pyramid_pass.rs:40:            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
src/render/shadow_mask_pass.rs:144:            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
src/render/blas_refit.rs:404:            let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
src/render/light_count_pass.rs:52:        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
```

Eight sites crate-wide: six among the passes, and two in the ray-tracing files (`acceleration_structures.rs:272` and `blas_refit.rs:404`). Three of the six wrap the descriptor in a local closure named `pipeline` to stamp out several entry points (`culling_pass.rs:247` to `259`, `depth_pyramid_pass.rs:38` to `52`, `shadow_mask_pass.rs:143` to `155`), which is the shared function written three times locally; the other three write it inline once (`froxel_pass.rs:95` to `106`, `bend_pass.rs:69` to `80`, `light_count_pass.rs:52` to `63`). The descriptor at `src/render/froxel_pass.rs:95` to `106`:

```rust
let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
    label: Some("build_froxel_lists"),
    layout: Some(&pipeline_layout(
        device,
        "Froxel Pass Pipeline Layout",
        &[(&FROXEL_LISTS, &layout)],
    )),
    module: &shader,
    entry_point: Some("build_froxel_lists"),
    compilation_options: wgpu::PipelineCompilationOptions::default(),
    cache: None,
});
```

The closure form at `src/render/shadow_mask_pass.rs:143` to `155`:

```rust
let pipeline = |entry_point| {
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(entry_point),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some(entry_point),
        compilation_options: wgpu::PipelineCompilationOptions {
            constants: &constants,
            ..wgpu::PipelineCompilationOptions::default()
        },
        cache: None,
    })
};
```

Every site labels the pipeline with its entry point (or a name equal to it), passes one layout and one module, and sets `cache: None`; the only variation is whether `constants` is empty. Line count of the six descriptors: 11 + 12 + 12 + 11 + 12 + 12 = 70. A shared `compute_pipeline(device, layout, module, entry_point, constants: &[(&str, f64)])` of about 15 lines beside `pipeline_layout` (`src/render/bind_groups.rs:529`), which already plays this role for layouts, replaces them with six one-line calls and removes the three local closures. The pipelines created are identical, so no pixel can move; Goldens stay bit-exact and counters equal.

#### L5-R19

`src/render/main_pass.rs:231` to `263` and `270` to `297`, side by side:

```rust
pub fn record_depth_prepass(&self, encoder, render_targets, bind_groups, packed_indices, frame_draw, timestamp_writes) {
    let surface_normals = [Some(render_targets.surface_normals_attachment(true))];
    let mut depth_prepass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(DEPTH_PREPASS),
        color_attachments: if self.shadow_rays { &surface_normals } else { &[] },
        depth_stencil_attachment: Some(render_targets.depth_attachment()),
        ...
    });
    depth_prepass.set_pipeline(&self.depth_prepass_pipeline);
    self.bind(&mut depth_prepass, bind_groups.iter().copied()
        .filter(|&(definition, _)| definition.visible_to(wgpu::ShaderStages::VERTEX)), packed_indices);
    depth_prepass.draw_indexed_indirect(frame_draw, 0);
}

pub fn record_phase_two_depth(&self, encoder, render_targets, bind_groups, packed_indices, frame_draw, timestamp_writes) {
    let mut depth_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(PHASE_TWO_DEPTH_PASS),
        color_attachments: &[Some(render_targets.surface_normals_attachment(false))],
        depth_stencil_attachment: Some(render_targets.loaded_depth_attachment()),
        ...
    });
    depth_pass.set_pipeline(&self.depth_prepass_pipeline);
    self.bind(&mut depth_pass, bind_groups.iter().copied()
        .filter(|&(definition, _)| definition.visible_to(wgpu::ShaderStages::VERTEX)), packed_indices);
    depth_pass.draw_indexed_indirect(frame_draw, PHASE_TWO_DRAW_OFFSET);
}
```

Same signature (six parameters), same pipeline (`self.depth_prepass_pipeline` at 253 and 287), the same seven-line bind-group filter written twice (254 to 261 and 288 to 295), same `occlusion_query_set`, `multiview_mask` and `timestamp_writes`. The differences are the label, the colour attachment (phase one's clears the surface normals and binds them only with shadow rays on; phase two's loads them and always binds them, which `record_phase_two_depth` may do because the Renderer calls it only with shadow rays on, `renderer.rs:658`), the depth attachment (cleared or loaded) and the draw offset (0 or `PHASE_TWO_DRAW_OFFSET`). Callers: `renderer.rs:639` and `renderer.rs:659`, one each.

```
$ grep -rn 'record_depth_prepass\|record_phase_two_depth' src tests | grep -v 'fn record_'
src/render/renderer.rs:639:        self.main_pass.record_depth_prepass(
src/render/renderer.rs:659:            self.main_pass.record_phase_two_depth(
```

One `record_depth_pass` taking a `CullingPhase` (already imported into this module's neighbour, `src/render/culling_pass.rs:25`) and matching on it for the four differing values is about 35 lines in place of 67 and writes the filter once. The commands recorded are identical, so no pixel can move.

#### L5-R20

The one caller, `src/render/renderer.rs:639` to `677`, passes the same two culling buffers to all three record methods:

```rust
self.main_pass.record_depth_prepass(
    &mut encoder,
    &self.render_targets,
    &bind_groups,
    self.culling_pass.packed_indices(),
    self.culling_pass.frame_draw(),
    timestamps.map(|timestamps| timestamps.render_pass(DEPTH_PREPASS)),
);
...
    self.main_pass.record_phase_two_depth(
        &mut encoder,
        &self.render_targets,
        &bind_groups,
        self.culling_pass.packed_indices(),
        self.culling_pass.frame_draw(),
        ...
    );
...
self.main_pass.record_lit_pass(
    &mut encoder,
    &self.render_targets,
    &bind_groups,
    self.culling_pass.packed_indices(),
    self.culling_pass.frame_draw(),
    frame.background,
    ...
);
```

```
$ grep -rn 'record_depth_prepass\|record_phase_two_depth\|record_lit_pass' src tests | grep -v 'fn record_'
src/render/renderer.rs:639:        self.main_pass.record_depth_prepass(
src/render/renderer.rs:659:            self.main_pass.record_phase_two_depth(
src/render/renderer.rs:672:        self.main_pass.record_lit_pass(
```

Three callers, every one passing `self.culling_pass.packed_indices()` and `self.culling_pass.frame_draw()`. The culling pass creates both buffers once at a fixed capacity (`src/render/culling_pass.rs:197` to `211`) and never recreates them: `CullingPass::bind_render_targets` rebuilds only bind groups (`src/render/culling_pass.rs:327` to `334`), so a handle cloned at construction stays valid for the Renderer's life, which is exactly how the main pass already holds the third culling buffer:

```rust
// src/render/main_pass.rs:55 and 60
meshlet_records: &wgpu::Buffer,
...
let meshlet_records_buffer = meshlet_records.clone();
```

The parameters at `src/render/main_pass.rs:236` to `237`, `275` to `276` and `311` to `312` are the six sites; the eighth argument of `record_lit_pass` is what `#[expect(clippy::too_many_arguments)]` at `src/render/main_pass.rs:305` excuses, and dropping two brings it to six, under the lint's threshold of seven, so the attribute goes too. `render_targets` must stay a parameter: the targets are recreated on resize. The commands recorded are identical, so no pixel can move. Fixed together with L5-R19 the main pass's record interface shrinks from three six-to-eight-parameter methods to two of four to five.

#### L5-R21

The comment at `src/render/culling_pass.rs:356` to `357` and the code it heads at `358` to `365`:

```rust
/// Bytes of the frame's draw and its stand-in, and of the Instance slot
/// count, offsets and visibility.
pub fn gpu_memory_bytes(&self) -> u64 {
    self.buffers
        .named()
        .iter()
        .map(|(_, buffer)| buffer.size())
        .sum::<u64>()
        + self.buffers.frame_draw_stand_in.size()
}
```

`named()` at `src/render/culling_pass.rs:370` to `379` returns six buffers:

```rust
fn named(&self) -> [(&'static str, &wgpu::Buffer); 6] {
    [
        ("instance_slot_count", &self.instance_slot_count),
        ("instance_offsets", &self.instance_offsets),
        ("meshlet_records", &self.meshlet_records),
        ("packed_indices", &self.packed_indices),
        ("frame_draw", &self.frame_draw),
        ("instance_visibility", &self.instance_visibility),
    ]
}
```

The comment names five buffers (the frame's draw, its stand-in, the slot count, the offsets, the visibility) and leaves out `meshlet_records` and `packed_indices`, which the sum includes: at `DrawCapacity::PROVISIONAL` (`src/render/culling_pass.rs:68` to `71`) those two are 2^20 × 12 B = 12 MiB and 2^25 × 4 B = 128 MiB, the bulk of what the function returns; the five it names total under 1.3 MiB. Load-bearing: the review checklist (`CLAUDE.md`, "Frame stats still see new work: GPU buffers and textures join their owner's `gpu_memory_bytes`") sends a reader to this comment to learn which buffers are counted; one who trusts it would add the two to another owner's sum and double count 140 MiB against the GPU memory Budget, or report them as uncounted in a Budget split. No other owner counts them now:

```
$ grep -rn 'meshlet_records\|packed_indices' src/render/renderer.rs src/assets/pools.rs | grep -i 'size()\|memory'
(no output)
```

The fix is a comment; it moves no pixel and no counter.

### Notes for the verifier

Not filed, since each is a near-miss under the lane's axes, an allowed form, or belongs to another lane; recorded so the next audit does not re-find them. By reader: A the Renderer, B the data path, C ray tracing and stats, D the passes.


**A.**

- Render switches, vestigial axis: every switch is set by something. `culling` by `tests/goldens.rs:145` and `tests/frame_stats.rs:292`; `froxel_lights` by `tests/goldens.rs:159`; `shadow_rays` by `src/bench/mod.rs:462`, `tests/shadows.rs:153`, `tests/goldens.rs:151`; `count_point_lights` by `src/bin/praxis.rs:745` (the `capture` command, when no frame is named) and the test at `renderer.rs:2001-2003`; `crowd_form` by `src/bin/praxis.rs:158` and `tests/crowd.rs:34`. `light_count_pass.rs` and the switch's shipping cost are render D's.
- `Renderer::with_draw_capacity` (`renderer.rs:242-249`) is private, called by `new` with `DrawCapacity::PROVISIONAL` and by one test (`renderer.rs:1010`) with a small capacity to reach the draw's overflow. A parameter one caller varies, but it is the test seam for the overflow path; not filed.
- `Renderer` stores `shadow_rays: bool` (`renderer.rs:179`) and `marks_dynamic_tlas_build: bool` (`renderer.rs:185`), both derived from `switches` at `renderer.rs:419,421`, rather than the `RenderSwitches` value. Two fields for one; a judgement call, not filed.
- `Renderer::aspect_ratio` (`renderer.rs:170,412,461,465-467`) is set in `new` and `resize` and returned by the getter; the Renderer never reads it. Its one reader is `src/state.rs:266`, which hands it to `FramePreparer::prepare`. The Renderer carries a value only its caller uses, from a width and height the caller gave it; 5 lines. Below the threshold I filed at; the verifier may fold it into L5-R01 as "interface wider than the Renderer needs".
- `render`'s fallback `unwrap_or_else(|| create_frame_encoder(&self.device))` (`renderer.rs:578-581`) is never reached: every caller uploads first (`state.rs:270` then `280-282`; `lib.rs:248,253`; `capture/mod.rs:260-261,432-433`), and `upload`'s `debug_assert!` at `renderer.rs:478-481` guards the other order. Flexibility nothing exercises, but it is the graceful form `CLAUDE.md` asks for in place of `unwrap` in `src/render/` (`tests/conventions.rs:274-286`); not filed.
- The stage names `"upload"`, `"encode"` and `"submit"` (`renderer.rs:561,713,726`) are literals where `DYNAMIC_TLAS_RECORDING`, `BLAS_REFIT` and `DYNAMIC_TLAS_SUBMIT` are constants; each literal has one other site, `tests/frame_stats.rs`. Two sites each; not filed.
- `renderer.rs:1220-1224` builds a `sun` equal to the `SUN` constant at `renderer.rs:1506-1513` (offset `[0, 0, -1]`, intensity `[1; 3]`, the rest default); 5 lines, one site. Below threshold; it would ride along with L5-R09's fix.
- `draw_calls += 4` (`renderer.rs:694`) with shadow rays off: checked against `main_pass.rs:329-333`, where the lit pass draws twice (phase two into depth, then both phases) when `shadow_rays` is false, so the count holds either way. Clean.
- `TIMED_PASSES`'s doc (`renderer.rs:25-34`) checked against `readback.rs:625-660`: a render pass counts from the latest earlier end, a compute pass from its own start, the build from the marker before, the refit from the bend pass's end. Consistent.
- `_pad`, `_pad_offset`, `_pad_intensity`, `_pad_dynamic_tlas_bounds_min`, `_pad_dynamic_tlas_bounds_max` (`uniforms.rs:14,37,52,63,65`): weighed `pad` as the word padding, which `CLAUDE.md`'s review checklist names ("Rust-only padding fields start with `_`"); allowed.
- Domain words weighed and allowed: `gpu` as a local for `GpuFrameStats` in tests, `ev100`, `uv`, `qtangent` (the cooked vertex's own name), `rgb` in a comment, `SSBO` in the comment at `renderer.rs:257` (a comment, not an identifier; the comment is accurate: the Material pool is created at `261`, the main pass at `274`).
- `instance.rs:1-3` says 64 MiB: 2^19 Instances times 128 bytes (`rotation_scale` 48, `normal_matrix` 48, `anchor` 12, `material_id` 4, `offset` 12, `mesh` 4) is 64 MiB. Clean.
- Among the passes: `main_pass.rs:319`, the tutorial comment twin of L5-R07.
- Falls in the orchestrator (`src/state.rs:287-288`), lane 5's every-fifth-file sample or lane 7: the `gpu_frame_stats` doc repeats L5-R04 with `update` for `upload`.
- Falls in shaders: the 23 WGSL sites of `view_proj` (L5-R03).
- In the data path: whether `prepare.rs` orders lights as `uniforms.rs:17-21` says (directional, then shadowing point lights, then the rest); not checked here.
- Lane 1: nothing found in these files that disagrees with an ADR.
- `docs/audit/deviations.md` has no rows at this commit; nothing in `scripts/cycles-reference/DIFFERENCES.md` bears on these files.


**B.**

1. `max_slot_size` (`staging.rs:27`, `:41`, `:68-73`): `max` abbreviates "maximum", but it is Rust's own vocabulary (`Ord::max`, `u64::MAX`, `Iterator::max`), used as a word across the crate (`.max(1)` at `targets.rs:140`), and the full-word rule's examples do not name it. Weighed as the API's spelling; a stricter reader may file it (3 sites in this file).
2. `mesh_ref`, `material_ref` (`prepare.rs:355`, `:358`, `:392-395`): locals named after the component types `MeshRef` and `MaterialRef` (`src/components.rs`, outside `src/render/`). `Ref` is Rust's own spelling (`std::cell::Ref`, `AsRef`, the `ref` keyword) and `CLAUDE.md`'s project shape names `MeshRef` as the component. If the type is judged an abbreviation, it falls in components, not render; the locals follow the type.
3. `RenderableItem<'world>` (`prepare.rs:72`): a type alias with one use, the `visit` closure's parameter at `:355`. It names a `QueryItem<'_, '_, RenderableData>` that would otherwise be spelled inline once, at the same length; neither form is smaller. Not over-engineering by the lane's definition (a parameter, not an abstraction with flexibility).
4. Three warn-once latches in `prepare` with the same shape: `lights_overflowed` with `warned_of_overflowed_lights` (`prepare.rs:323-330`), `moving_instances_left_out` with `warned_of_moving_instances_left_out` (`:439-452`), `bends_left_out` with `warned_of_bends_left_out` (`:461-468`). Each is "warn when the count is first above zero, re-arm when it returns to zero", 6 fields and about 24 lines where one `WarnOnce { count, warned }` type with a `set(count, message)` would serve. Duplication inside one file is not one of the lane's six axes (the repeated-shape reading was done across the passes), so recorded here, not filed. A fix would move no pixel.
5. `uniform` and `read_only_storage` are `pub(crate)` (`bind_groups.rs:548`, `:557`) and their only callers outside the file are in `src/render/shader_tests.rs` (6 sites: `:73-76`, `:388`, `:391`). `pub(super)` is the narrowest form that still serves the sibling test module. One token of visibility on two helpers; below the magnitude worth a ticket.
6. `Binding::writer` is read only by tests (`shader_tests.rs:668`) and carries `#[cfg_attr(not(test), allow(dead_code))]` (`bind_groups.rs:414-415`). The field's doc at `:417-420` says why it exists: to tell `shader_tests` which structs have a Rust twin to compare. A test-driving field in shipping data, deliberate and documented; not vestigial by the lane's definition (it is read).
7. `MAIN_PASS_INPUTS` doc (`bind_groups.rs:266-268`) describes `shadow_mask_samples` as "which samples show its second surface"; the texture also carries the first directional lights' verdicts per sample (`targets.rs:35-40`, `:77-80`) and `main.wgsl:235` loads it. Incomplete rather than wrong, and the same staleness as L5-R11's; recorded here so the next audit sees it was read.
8. The `depth` field at `targets.rs:59` has no doc comment where every other field has one; the getter at `:239-242` documents it. Style, not an axis.
9. `targets.rs:1-7`, the module doc, says the Hi-Z pyramid "keeps the minimum, the farthest" and the HDR target "holds exposed colour weighted by 1/(1 + luminance)"; both checked: `hdr.wgsl:23-25` divides by `1 + relative_luminance`, and reverse-Z makes the minimum the farthest. Clean.
10. `bind_groups.rs:5-8` says groups sit "within the Capability floor's four (ADR 0021)"; the ADR's clause is lane 7's to check, not this lane's. The highest index in the file is 3, consistent with four.
11. `FrameStaging::write` takes `impl Iterator + Clone` and walks the iterator twice (`staging.rs:60-63` for the size, `:105` for the copies); the Renderer's one caller (`renderer.rs:536-549`) builds a chain of slices, cheap to clone. A design choice that avoids collecting; not over-engineering.
12. Nothing in these four files matches a row of `docs/audit/deviations.md` (it has none at this commit) or a section of `scripts/cycles-reference/DIFFERENCES.md`. Nothing here is a correctness or ADR question for lane 1; the `prepare_bends` race-drop loop at `prepare.rs:618-629` removes while it searches the same list, which reads as correct for the cases traced, but a lane 1 reader may want to confirm it against the bend pass's write ordering.


**C.**

1. `AccelerationStructureBytes`' three `pub` fields (`src/render/acceleration_structures.rs:36-39`): the only reader outside the file is a test literal in `src/assets/pools.rs:696-700`; the shipping callers use `estimate`, `blases` and `tlas`. Three fields, one site; `pub(crate)` would change nothing a caller sees. Below the magnitude worth a ticket.
2. `DynamicTlas::fill`'s `.take(DYNAMIC_TLAS_CAPACITY as usize)` (`acceleration_structures.rs:319-322`) under the comment "Prepare lists no more than its capacity": `src/render/prepare.rs:437-438` truncates to the same capacity, so the branch is never reached. Kept because it guards `self.tlas[index]` from an index panic in the render path if prepare regressed; a guard is not flexibility.
3. `MAP_FAILED` (`readback.rs:72`): stored at line 492, matched only by the wildcard at line 477. Referenced, so not vestigial; a named store is clearer than a bare `4`.
4. `std::ptr::eq(pipeline, &id_pipeline)` (`coverage_tests.rs:374`): the `prepare` closure decides whether to bind the ID buffer by comparing its argument's address with a captured local, where the `writes_ids: bool` that built the pipeline already says so. Clever where boring would do; one site in a test file, no axis of this lane names it.
5. `StaticTlas::new` takes a `capacity` while `DynamicTlas::new` reads `DYNAMIC_TLAS_CAPACITY` itself (`acceleration_structures.rs:116, 244-247`): an asymmetry, but the static one's parameter is what lets the left-out test use a capacity of 2 (line 795). Not over-engineering.
6. `FrameTimestamps::render_pass` carries a `debug_assert!` that the pass is in `RENDER_PASSES` (`readback.rs:512-515`); `compute_pass` has no twin asserting the opposite (line 501). An asymmetry in a test-only check, not a defect.
7. `pass_position` panics on an unknown pass (`readback.rs:675-680`) and runs in the render path from `frame_times` (line 587, 640, 664), with constant inputs only, so the panic is unreachable; `tests/conventions.rs` catches `.unwrap()` and not `panic!`. If judged at all it is lane 1's (a render-path panic rule), not health.
8. "7,871 are in view" (`coverage_tests.rs:155-156`) is a number in a test file's comment that only a run of `tilted_quads` confirms; `docs/performance.md:1983` area carries the same experiment. Needs a GPU-less run; not filed.
9. `DynamicTlas::mark`'s doc (`acceleration_structures.rs:394-397`) says wgpu-core checks the first marker's query against the BLASes of the last build; the claim cites no registry key here (the related `wgpu-30-tlas-blas-build-order` is cited at `blas_refit.rs:230`). A source check, falls in lane 7.
10. `prepare_encoder`'s comment (`acceleration_structures.rs:373-387`) describes an encoder-swap mechanism on Metal and the allocation it avoids; a timing and allocation claim that needs a GPU run, falls in lane 3.
11. `entry_upload_bytes` of 72 on Metal and 64 elsewhere (`acceleration_structures.rs:288-297`) are source figures with keys `metal-indirect-instance-descriptor`, `vulkan-acceleration-structure-instance`, `d3d12-raytracing-instance-desc`; whether they match the cited structs falls in lane 7.
12. `vs_main`, `fs_main`, `cs_read` (`coverage_tests.rs:256, 261, 267`): stage abbreviations as entry point names; `src/render/shaders/main.wgsl` and `src/render/main_pass.rs` use `vs_main` and `fs_main` too, so a Finding would be against the engine's convention, falls in shaders.
13. `STATIC_TLAS_CAPACITY`'s comment "Sized for the 250k Instances M3 plans `town` with" (`acceleration_structures.rs:9-10`): `town` is built now, with 254,256 Instances (`docs/performance.md:966`), 10,000 of them Moving (`src/scenes/town.rs:236-239`), so 244,256 static under 262,144. "plans" is past tense; the number still holds. Not drift.
14. `DYNAMIC_TLAS_CAPACITY`'s comment "counted and warned about" (`acceleration_structures.rs:15-18`): the counting and the warning are in `src/render/prepare.rs:438-452`, not in this file; a statement about the system, true as written. Not drift.
15. The test doc at `acceleration_structures.rs:727-730` has one line past the comment width ("and a point at the origin once it holds none. A"); `rustfmt` leaves comments alone. Formatting, not an axis.
16. Every `ref` key cited in the four files resolves to a quoted table in `docs/references.toml` (33 keys checked by grep against the `["key"]` form). `cargo test --test references` guards the same.
17. Outside the lane: the worktree's `CLAUDE.md` and `main`'s differ only in the project-shape tree (the `cycles/<scene>.json` Baselines line under `bench/`), a later commit's addition; the conventions judged against are identical.
18. `docs/audit/deviations.md` has no rows at this commit; `scripts/cycles-reference/DIFFERENCES.md` describes shader differences, none of which touch these four files.


**D.**

1. **`count_point_lights` is not vestigial.** `src/bin/praxis.rs:745` sets `count_point_lights: frame.is_none()` and the Renderer test at `src/render/renderer.rs:2002` sets it `true`; the pass behind it (`light_count_pass.rs`) is created only then (`renderer.rs:310`). No scene, bench Baseline or script names the switch (`grep -rn count_point_lights docs scripts bench` returns nothing), so it is a command's switch, not a scene's; weighed and it holds.
2. **Three tile-size constants, one value.** `DEPTH_PYRAMID_TILE_SIZE` (`depth_pyramid_pass.rs:14`), `SHADOW_MASK_TILE_SIZE` (`shadow_mask_pass.rs:25`) and `LIGHT_COUNT_TILE_SIZE` (`light_count_pass.rs:15`) are all 8, each declared to WGSL on its own line of `src/render/shaders/mod.rs:246`, `247` and `258`, and each drives the same `width.div_ceil(TILE_SIZE), height.div_ceil(TILE_SIZE), 1` dispatch (4 sites). Not filed: each pass's workgroup shape is its own tuning knob (the shadow mask's tile also sizes its list entries, `shadow_mask.wgsl:438`, `489`), and merging them couples three passes for a saving of two lines. A later audit may file it if a fourth joins.
3. **`MainPass::lit_pass_depth_pipeline` is compiled in every configuration and set only with shadow rays off** (`main_pass.rs:136` to `149`, used at `328` to `331`); `RenderSwitches::default()` has `shadow_rays: true` (`renderer.rs:143`). The off branch is reached (lavapipe CI, the `shadow_rays` A/B), so not vestigial; an `Option<wgpu::RenderPipeline>` created only when `!switches.shadow_rays`, as the Renderer holds `light_count_pass: Option<LightCountPass>`, would spare one pipeline compile at startup. Below the bar; needs a startup-time measurement to matter, which is lane 3's.
4. **`ShadowMaskPass::new` takes `buffers: [(&'static str, &wgpu::Buffer); 4]`** (`shadow_mask_pass.rs:119`), so the Renderer spells four of the pass's binding names (`renderer.rs:383` to `388`: `"camera"`, `"light"`, `"lights"`, `"froxel_lists"`) that `SHADOW_MASK_BUILD` also spells (`bind_groups.rs:208` to `214`); `FroxelPass::new` (`froxel_pass.rs:61` to `66`) and `LightCountPass::new` (`light_count_pass.rs:43` to `49`) take the same buffers as named parameters and keep the names inside. One call site, four names; a divergence of shape between sibling passes rather than width, so weighed and not filed. A fix ticket for L5-R18 to L5-R20 could take it along.
5. **A number in a shipping comment**: `tonemapping_pass.rs:14` to `16`, "a modulo by the texture's size cost about 38 µs at 1440p on the M4 Pro". The axis lists numbers in shipping comments, but the evidence a comment Finding needs is the comment beside code it misdescribes, and whether 38 µs still holds needs a GPU A/B on the M4 Pro, which a reader does not run. Left for lane 3 or the owner; the comment's claim about the mask side being a power of two is asserted at `tonemapping_pass.rs:18`.
6. **wgpu-template comments** at `main_pass.rs:84` ("Setting this to anything other than Fill requires Features::NON_FILL_POLYGON"), `86`, `88` and `319` ("This is what @location(0) in the fragment shader targets") are the wgpu tutorial's stock comments; they are true of wgpu and say nothing about this pass's why. Not drift (not wrong), and four comment lines are below a health Finding's bar. Nothing else in the crate carries them (`grep -rn 'Requires Features::'` finds only these).
7. **`light_count_pass.rs:91` labels its pass with the literal `"light_count_pass"`** where every other pass has a `pub(crate) const` that `TIMED_PASSES` (`renderer.rs:35` to `48`) reads. The light count pass is untimed by design (`renderer.rs:686`, "Untimed, after the passes the frame's GPU time spans"; `record` takes no `timestamp_writes`), so nothing reads a constant; not filed.
8. **`DrawCapacity`'s public fields** (`culling_pass.rs:57` to `60`) are set outside `PROVISIONAL` only by the test at `renderer.rs:999` through the private `Renderer::with_draw_capacity` (`renderer.rs:242`). A test that shrinks the draw to prove overflow handling is the flexibility's one legitimate exerciser; not filed.
9. **Buffer creation shape**: `culling_pass.rs:172` to `179` wraps `create_buffer` in a local `buffer(label, size, usage)` closure and `shadow_mask_pass.rs:165` to `172` in a `dispatch(label, usage)` closure; `froxel_pass.rs:68`, `74`, `bend_pass.rs:53` and `shadow_mask_pass.rs:157` write the descriptor inline, as do 14 sites outside the passes. Every descriptor sets `mapped_at_creation: false`. Not filed: the closures are local and the inline form is five lines; a crate-wide helper would be the kind of abstraction that saves one line a site.
10. **`FrameDrawBuffers::named()` substitution by string** (`culling_pass.rs:393`, `if name == "frame_draw"`) picks the stand-in by the binding's name. Correct and tested through the Goldens; a typo would surface as a validation error at the first frame. Not filed.
11. **Naming**: nothing filed in eight files. Weighed and allowed: `uv`, `qtangent`, `gpu`, `hdr`, `tlas`, `blas` (domain words); `r`, `g`, `b`, `a` (`wgpu::Color`'s fields, `tonemapping_pass.rs:254` to `257`, `318` to `321`); `x`, `y` and `below_x`, `below_y` as coordinates (`depth_pyramid_pass.rs:176`, `301` to `305`; `shadow_mask_pass.rs:433`); `reduced.0`, `reduced.1` as tuple fields (`depth_pyramid_pass.rs:75`). `tests/conventions.rs` covers the shading initials, `println!`, ticket numbers, sRGB formats, forward depth and `unwrap` in `src/render/`; none re-found.
12. **Long doc lines** at `culling_pass.rs:8` and `shadow_mask_pass.rs:112` run past 80 columns where `rustfmt` does not wrap comments. Style, not an axis; `cargo fmt` owns it.
13. **Lane 1 note**: `main_pass.rs:71` to `73` says "The depth pipelines' fragment stages read nothing, so they read only the groups their vertex stage sees"; with shadow rays on the pre-pass fragment stage is `fs_surface_normals`, which writes a colour target from interpolated inputs and reads no binding, so the layout filter at `73` holds by construction (a fragment binding outside the vertex-visible groups would fail pipeline creation). Consistent; recorded so lane 1 need not re-read it.
14. `docs/audit/deviations.md` has no rows at this commit; `scripts/cycles-reference/DIFFERENCES.md` describes image differences, none of which a pass's Rust touches.

### Counts

| Severity class | Findings |
|---|---|
| `image` | 0 |
| `budget` | 0 |
| `correctness` | 0 |
| `drift` | 8 (L5-R04, L5-R05, L5-R06, L5-R11, L5-R12, L5-R13, L5-R16, L5-R21); load-bearing 1 (L5-R21) |
| `health` | 13 (L5-R01, L5-R02, L5-R03, L5-R07, L5-R08, L5-R09, L5-R10, L5-R14, L5-R15, L5-R17, L5-R18, L5-R19, L5-R20) |

By axis: over-engineering 7 (L5-R08, L5-R09, L5-R14, L5-R15, L5-R18,
L5-R19, L5-R20); vestigial 1 (L5-R07); cognitive complexity 0 (the probe
ran clean under `src/render/`); naming 2 (L5-R03, L5-R17); module depth 3
(L5-R01, L5-R02, L5-R10); comment accuracy 8 (the `drift` rows). Of the
`health` rows, 4 are in test modules only (L5-R08, L5-R09, L5-R14,
L5-R17) and 9 in shipping code. Axis sweep: 21 file rows. Interface
widths: 28 types and module surfaces, 313 public items counted. Files
read in full: the 20 of the module; read in part for use counts: 25
others across `src/`, `tests/`, `docs/` and `bench/`.

## Killed

Written by the report session.
