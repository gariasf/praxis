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

## shaders

Scope: `src/render/shaders/`, the 16 WGSL parts (`common.wgsl`,
`shading.wgsl`, `main.wgsl`, `gt7.wgsl`, `hdr.wgsl`, `display.wgsl`,
`tonemapping.wgsl`, `culling.wgsl`, `depth_pyramid.wgsl`,
`froxel_grid.wgsl`, `froxel.wgsl`, `light_count.wgsl`, `bend.wgsl`,
`shadow_mask.wgsl`, `dynamic_tlas_marker.wgsl`, `blas_refit_marker.wgsl`)
and `mod.rs`, which assembles them into the eleven shader modules after
the constants Rust declares to WGSL, the Render switches' overrides and the
bind groups' declarations; with the two shader test modules,
`src/render/shader_function_tests.rs` (the oracle harness and 28 oracle
tests) and `src/render/shader_tests.rs` (module validation, stage
visibility and the CPU-twin layout checks); 5,972 lines, every file read
in full. Four readers at high effort shared the module by file, each with
the lane brief verbatim: shading and the display (the shading chain from
`main.wgsl` through `shading.wgsl` to `hdr.wgsl`, and the tonemapping
chain through `display.wgsl` and `gt7.wgsl`, with `common.wgsl`);
geometry, culling and lights (the culling phases, the depth pyramid, the
Froxel grid, the light count and the bend); the shadow mask and the
assembly (`shadow_mask.wgsl`, the two marker parts and `mod.rs`); and the
shader tests. Each swept its files along the six axes and recorded the
sweep, so what was read and judged clean is on the record beside what was
filed. The cognitive complexity probe,
`cargo clippy --all-targets -- -W clippy::cognitive_complexity`, ran once
at the frozen commit: three functions over the default threshold of 25,
two in the cooker's exporter and one in the binary's command dispatch,
none under `src/render/`; the two shader test modules were in its scope
and are clean, and clippy reads no WGSL, so this section files nothing on
that axis and the longest WGSL function is recorded in the notes for a
probe that can. Every `where` is `path:line` at the frozen commit, under
`src/render/shaders/` unless given in full. `deviations.md` holds no row
at this commit; `DIFFERENCES.md` was read for what is not a Finding, and
its shader differences describe `ambient_luminance` and
`specular_antialiased_roughness` as they are. `tests/conventions.rs` was
read for the rules already mechanical, and none of its catches is
re-filed; the `view_proj` spelling's 23 WGSL sites are counted under the
render section's row and not re-filed here. The worktree was unavailable
for two minutes during the read, so a few of the use counts ran on the
main checkout, whose `src/` and `tests/` are identical to the frozen
commit (`git diff --stat 8bc34eb1 3368786d -- src tests` prints nothing).
Each Finding's evidence block says whether the fix it implies can move a
pixel: none can, so every `health` fix ticket carries the refactor licence
and none is a defect; one, L5-S05, can change a Bench counter by a
last-bit rounding, so its ticket compares the light count pass's three
counters with the Baselines rather than relying on bit-exactness, and
L5-S09 leans on naga accepting storage pointers as function parameters,
with the smaller fallback its block names.

### Axis sweep

| File | Over-engineering | Vestigial | Cognitive complexity | Naming | Module depth | Comments |
|---|---|---|---|---|---|---|
| `common.wgsl` | clean (`point_light_position`'s two camera parameters take `camera.anchor, camera.offset` at all 4 call sites, but the part is assembled into the bend module and the oracle harness, which declare no `camera`, so the parameters are what lets it compile there; `camera_relative_position`'s `local_position` varies, `vec3(0.0)` at one caller and a rotated vertex at the others; `rotate` has 3 callers in 2 parts) | clean (every struct field is read by some part: `unshadowed_range` at `shadow_mask.wgsl:300`, `dynamic_tlas_entries` and the bounds at `shadow_mask.wgsl:103-109`, `tlas_anchor` at `:149`, `exposure` at `main.wgsl:305`, `MeshletData.material_id` at `culling.wgsl:385`; `extra.yzw` and `emissive.w` are the 16-byte layout's padding, see Notes 6) | none (probe ran clean; the probe reads no WGSL) | clean (`view_proj` at lines 14, 44, 47, 49, 51, 52: counted under the render section's `view_proj` row; `uv`, `qtangent`, `tlas` domain words; `bounds_min`, `box_min` and their `max` twins weighed as the builtins' own spelling, Notes 9; `z` a coordinate) | clean (12 functions, 14 structs; the 3 functions one module calls, `position_at`, `segment_meets_box`, `octahedral_decode`, and `pack_index`, `unpack_index`, `decode_vertex`, `octahedral_encode` each sit under an oracle, which the convention puts in a shared part; 11 structs are named by bind group declarations or `shading.wgsl`) | L5-S01 (the header's "every shader module"; the other 24 comments checked against the code, the Rust twins in `uniforms.rs`, `instance.rs`, `material.rs`, `froxel_pass.rs` and ADRs 0014, 0015, 0018, 0019, 0020, 0023) |
| `shading.wgsl` | clean (the 11 functions with one WGSL caller each are under oracles, and `decode_surface`'s 7 parameters all vary at its caller `main.wgsl:196-204`) | clean (every constant and function is called; `Surface`'s 7 fields all read, `opacity` at `main.wgsl:306`, `occlusion` at `:267`) | none (probe ran clean; the probe reads no WGSL) | clean (`alpha` is glTF's own α, a full word; `brdf` a domain word; every dot product spelled out) | clean (19 items; 6 reach `main.wgsl`, the other 13 are internal and read by the oracle harness through `SHARED`) | L5-S02 (the 25 µs figure); the other 17 comments checked against the code and the 14 registry quotes they cite (`gltf-2.0-specular-brdf`'s ν formula, `gltf-2.0-normal-texture-scale`, `khr-lights-punctual-range`, `gltf-2.0-occlusion`, `tokuyoshi-2019-normal-derivatives` among them); the 1.1% figure at line 110 weighed, Notes 3 |
| `main.wgsl` | clean (`vertex_output` has 3 entry-point callers; `masked` and `directional_hidden_share` 2 each; `froxel_mask_bit` 1, 15 lines; `DepthNormalsOutput` beside `VertexOutput` carries 3 interpolants in place of 5; the first directional light hoisted out of its loop is the measured branch cost's workaround, Notes 4) | clean (`fs_depth` is set by `main_pass.rs:141` for the lit pass's depth pipeline; `UNMASKED` returned at `:163` and read by `masked` through its `bit < SHADOW_MASK_LIGHTS` test; `VertexOutput`'s 6 fields all read in `fs_main`) | none (probe ran clean; the probe reads no WGSL) | L5-S04 (`vs_`, `fs_` on 6 entry points); `in`, `out`, `half`, `first`, `low`, `high`, `middle` are words | clean (13 items, none shared: 6 entry points named by `main_pass.rs`, 7 helpers and types local to the pass) | clean (the header weighed, Notes 5; `fs_surface_normals`'s edge-on clause against lines 56-60, the flat-input clause against ADR 0019; the mask comment's "top bits of the word's second half" against `SHADOW_MASK_LIGHTS - 32u` at line 276, 59 - 32 = 27; `fs_depth`'s comment against `main_pass.rs:135-146`); L5-S03 (the 10% figure) |
| `hdr.wgsl` | clean (`relative_luminance` 2 callers in the part plus an oracle) | clean (`LARGEST_HALF` read at `main.wgsl:305` and `hdr.wgsl:30-31`; `resolve_weighted` at `main.wgsl:306`; `resolve_unweighted` at `tonemapping.wgsl:19`) | none (probe ran clean; the probe reads no WGSL) | clean (`color` a full word; `hdr` a domain word) | clean (4 items; 2 read by the main module, 1 by the tonemapping module, 1 internal under an oracle) | clean ("every shader of the lit pass writes it", line 17, weighed, Notes 7; the Karis and WGSL f16 quotes match) |
| `display.wgsl` | clean (`tonemap`, `srgb_encode`, `dither_noise` have one caller each in `tonemapping.wgsl` and sit here because each is under an oracle, with the display ramp over the two together) | clean (both matrices read at lines 23-24) | none (probe ran clean; the probe reads no WGSL) | clean (`uniform`, `centred`, `clamped` words; `srgb` a domain word) | clean (5 items; 3 read by the tonemapping module, 2 internal) | clean (BT.2087's matrix read against lines 8-12 column by column; the sRGB knee against `khronos-dfs-srgb`; Peters and Lipshitz quotes match) |
| `gt7.wgsl` | clean (adapted code under Polyphony's MIT notice; `let peak = GT7_PAPER_WHITE` keeps the sample's parameter as a local, Notes 8) | clean (five PQ constants each read in `pq_encode` and `pq_decode`; six functions each called) | none (probe ran clean; the probe reads no WGSL) | clean (`PQ_M1` to `PQ_C3` are BT.2100's own symbols, which the comment at lines 34-35 says the names follow; `rgb`, `lms`, `ictcp`, `pq` domain words, Notes 10) | clean (12 items; `gt7_tonemap` and `GT7_PAPER_WHITE` read by `display.wgsl`, the other 10 internal under oracles) | clean (the shoulder's 1.19 recomputed from lines 54-55: (0.444 - 1)/(0.25 - 1) = 0.741, 0.444 + 0.741 = 1.185; the PQ constants and the ICtCp rows against the BT.2100 quotes; the paper white against `gt7-slides`) |
| `tonemapping.wgsl` | clean | clean | none (probe ran clean; the probe reads no WGSL) | L5-S04 (`vs_full_screen`, `fs_tonemap`) | clean (2 entry points, each named once in `tonemapping_pass.rs:64, 70`) | clean (the triangle's corners recomputed from line 8: vertex 0 (-1, -1), 1 (3, -1), 2 (-1, 3); "a power of two declared from Rust's" is asserted at `tonemapping_pass.rs:18`) |
| `src/render/shaders/culling.wgsl` | clean (every helper with one caller, `clip_space_bounds`, `in_view_frustum`, `occluded`, `visible_before`, carries a named step of the two phases; `saturating_add` 7 sites, `slot_offsets` 2, `fits` 2, `side_plane_distances` 4, `near_plane_distance` 4; both phases run every branch, as `PHASE_TWO` and `CULLING` are pipeline constants; every constant the module declares from Rust that the part relies on, `SCAN_BLOCK_SIZE`, `MAX_WORKGROUPS_PER_DIMENSION`, `INSTANCE_CAPACITY`, `NO_MESH`, `NO_MATERIAL`, is read) | clean (26 items, each referenced; `FrameDraw.phase_two_offsets` written at `:320`, read at `:307`; `visible_words` written `:270`, read `:276`) | none (probe ran clean; the probe reads no WGSL) | clean (`view_proj` at `:131-132` counted under the render section's `view_proj` row; `.r`, `u` suffixes, `e` in `1e-4`, `w`, `x`, `y`, `z` are WGSL's spellings and components; `num_workgroups` is the builtin's name) | clean (4 entry points and 1 override are the interface; 20 items private to the part) | clean (the header's draw claim checked against `main_pass.rs:262`, `:296`, `:330`, `:333`; "0.041 mm" at `:162` checked against ADR 0014's Consequences; the Hillis and Steele key exists and the scan reads one place back as `:78` says; `fits`'s and `append_records`'s comments checked against the saturating offsets) |
| `src/render/shaders/depth_pyramid.wgsl` | clean (`pixel_minimum`, `shared_block_minimum`, `overlapped_texels` have one caller each and each names a reduction step the `reduce_depth` comment refers to; `OverlappedTexels` is WGSL's form for two return values) | clean (10 items, each referenced; `pixel_minima`'s extra row and column written `:81-90`, read `:102-108`) | none (probe ran clean; the probe reads no WGSL) | clean (`.r` swizzle, `u` suffixes, `x`, `y`; `below_size`, `odd_sides`, `past_tile` are words) | clean (2 entry points; 8 items private to the part) | clean ("Rust asserts it has (SAMPLE_COUNT)" at `:27` matches `depth_pyramid_pass.rs:18`; the formula at `:17` matches `:21-22`; "two along a side that is even and three along one that is odd" at `:67-68` checked for a width of 37 and 18 texels) |
| `src/render/shaders/froxel_grid.wgsl` | clean (`froxel_slice_start` serves one module, the Froxel pass, 2 sites; its inverse `froxel_slice` sits beside it, so the pair defines the slicing; Notes 4) | clean (5 items, each read by another part or by `froxel_at`) | none (probe ran clean; the probe reads no WGSL) | clean (`u` suffixes, `x`, `y`) | clean (5 items; `froxel_at` read by 3 modules, `FROXEL_GRID_SIZE`, `froxel_index`, `froxel_slice_start` by the Froxel pass, `froxel_slice` a helper of `froxel_at`) | clean (the header's slicing matches `froxel_pass.rs:17-22`; "counted from the lowest normalised device coordinates" at `:13`, `:32` matches `1.0 - point.y / size.y` at `:36`) |
| `src/render/shaders/froxel.wgsl` | clean (one entry point, no helper) | clean | none (probe ran clean; the probe reads no WGSL) | clean (`view_proj` at `:20-25` counted under the render section's `view_proj` row; `clip_x`, `clip_y`, `inside_sides`, `normal_lengths` are words) | clean (1 entry point) | clean (the plane derivation at `:20-24` matches `:26-31` and `:42-47`; "in light order" at `:3` matches the loop from `directional_light_count` at `:36`; "counting those its list has no room for" matches `:61-63`) |
| `src/render/shaders/light_count.wgsl` | L5-S05, L5-S06 | clean (every item referenced) | none (probe ran clean; the probe reads no WGSL) | clean (`view_proj` at `:17-20`, `:40` counted under the render section's `view_proj` row; `x_row`, `y_row`, `w_row`, `y_cross_w` name rows and a cross product by their coordinate) | L5-S05, L5-S06 (two private items re-derive what the shared part and the generated constants give the module) | clean (the header matches `light_count_pass.rs:1-7`; "clip z is the near plane's distance" at `:37` matches `common.wgsl:45-47`; the `webgpu-standard-sample-pattern` key exists and quotes sample 0 at (0.375, 0.125)) |
| `src/render/shaders/bend.wgsl` | clean (`sinc` and `quaternion_product` have one caller each and are the two mathematical steps the entry point composes) | clean (every item referenced; `BendData`'s 5 fields read at `:45-56`) | none (probe ran clean; the probe reads no WGSL) | clean (`toward`, `chord`, `middle`, `turned`, `stored`, `squared`, `series`, `axis_length` are words; `w`, `x`, `y`, `z` components) | clean (1 entry point and the CPU twin `BendData`) | clean ("a stand-in for skinning" at `:2` as the glossary's Bend entry says; the Taylor bound at `:24` checked, 0.5^8 / 9! = 1.08e-8; the series factors 6, 20, 42 give 1/6, 1/120, 1/5040; `msl-4.1-fast-math` and `msl-4.1-sin-fast-accuracy` exist and quote what `:25-26` claims; the quaternion's axis `(toward.z, 0, -toward.x)` is +Y x toward as `:52` says; the chord length at `:53-54` checked) |
| `src/render/shaders/shadow_mask.wgsl` | L5-S08, L5-S09, L5-S10 (every function's callers counted across the three modules that could include it: none has none; `sample_offset` and `turned_lights_other_samples_face` have one caller each, weighed in the Notes) | clean (every constant, struct and function read at least once; `PixelList` is the type of two bindings; `listed_count` is shared by two entry points) | none (probe ran clean; the probe reads no WGSL) | clean (`view_proj` at lines 212, 216, 261 and 419 to 422 counted under the render section's `view_proj` row; `hit.t`, `lane`, `own`, `point` weighed in the Notes) | clean (four entry points, four pipelines in `shadow_mask_pass.rs`; no function of the part is called from another part) | L5-S11, L5-S12 (every other comment checked against its lines, the margins' sines and cosines recomputed, `docs/performance.md`'s section on `first_intersection` and the six cited registry keys found) |
| `src/render/shaders/dynamic_tlas_marker.wgsl` | clean (one entry point, one pipeline, `acceleration_structures.rs:272`; the ray's zero origin, up direction, cull mask and `1e-6` reach are the cheapest query that still orders the pass) | clean | none (probe ran clean; the probe reads no WGSL) | clean | clean (its own bind group keeps the static TLAS's dependency walk out of the marker pass, `bind_groups.rs:99` to `106`) | clean (its comment matches `DynamicTlas::record`, `acceleration_structures.rs:355` to `371`: a marker before the build timed at its end, one after timed at its start) |
| `src/render/shaders/blas_refit_marker.wgsl` | clean (one entry point, one pipeline, `blas_refit.rs:404`) | clean (gated to Apple with its bind group and its caller, `mod.rs:153`, `bind_groups.rs:173`, `blas_refit.rs:285`) | none (probe ran clean; the probe reads no WGSL) | clean | clean | clean (matches `RefitMarker::record`, `blas_refit.rs:430` to `459`: one dispatch of one workgroup after the refits, timed at its start) |
| `src/render/shaders/mod.rs` | clean (`Part`, `ShaderModule`, `part!`, `descriptor`, `source`, `locate` and `assemble` each have callers outside the file or serve one that does; `enables` is non-empty on three modules) | L5-S07 | none (probe ran clean) | clean (`starts`, `before`, `line_start`, `switch`, `extension`; `x` and `y` bind the sample offsets' coordinates at line 271) | clean (counts below: every public item has a caller outside the file; `pub` is crate-wide in effect since `lib.rs:30` keeps `render` private) | clean (the `COMMON` doc at lines 57 to 58 is incomplete rather than wrong, weighed in the Notes; the `assemble` doc's order matches lines 232 to 290) |
| `src/render/shader_function_tests.rs` | clean (every helper has 2 or more callers except `srgb_decode`, 1 line with 2 calls, and `Case`, one test's fixture whose 4 methods hold real logic; `evaluate_cases`'s `vertices` is passed by 1 of 9 callers and `&[]` by 8; its `VECTORS` generic has 6 instantiations, 1, 3, 4, 5, 6 and `evaluate`'s 1; Notes 3, 4, 6) | clean (every function, constant and `Case` method has a caller; counts under Interface widths) | none (probe ran clean) | L5-S13 (`m1`, `m2`, `c1`, `c2`, `c3` at :809-817); `view_proj` at :1611, :1614, :1628, :1629, :1657 counted under the render section's view_proj row; `x`, `y`, `z` are coordinates, `xyz` a coordinate triple; `uv`, `uvs`, `gpu`, `pq`, `ictcp`, `gt7` the domain's words | clean (no public item; 13 functions, 1 struct with 4 methods, 14 constants, every caller inside the file) | L5-S14 (the comment at :1630 beside :1631), L5-S15 (the test name at :1298 against its assertion at :1316); every other test doc checked against what its assertions pin, and every number a doc names checked against the tree (Notes 11) |
| `src/render/shader_tests.rs` | clean (`ShaderScalar` has 5 impls, each a type some twin holds, `i16` for `CookedVertex::qtangent`; the `Twin` struct update at :345-351 is the flexibility `layout_check_reports_every_kind_of_mismatch` exercises; `validate` returns the module and its analysis, both read by `stage_problems` and the module alone by `structs_shared_with_the_cpu`) | clean (every function has a caller; `Field::size` and `Field::scalar` are read by `describe` through `twin_problems`) | none (probe ran clean) | clean (`ty` and `span` are naga's field names, `info` a word for naga's `ModuleInfo`; `wgsl` and `rust` name the two sides' sets at :697 and :705; `view_proj` at :242 counted under the render section's view_proj row) | clean (no public item; 10 functions, 2 structs, 1 trait with 5 impls, 1 macro; every caller inside the file) | clean (`wgpu-core 30.0.1` at :40 matches `Cargo.lock`; the header's four claims at :1-5 are each a test, :24, :35, :124, :232; `validate`'s "as `GpuContext` does" at :441 matches `src/gpu/mod.rs:70`, which requests `floor_features()`) |

### Interface widths

| Part or type | Public items | Callers outside the file | Wider than needed | Where |
|---|---|---|---|---|
| `common.wgsl` | 26: 14 structs, 12 functions | 11 structs named by bind group declarations (`bind_groups.rs:26-87, 128-154, 190-193, 208-215, 277, 312-322`) or by `shading.wgsl` (`MaterialData`); 11 functions called from other parts: `camera_relative_position` 3 parts (`culling.wgsl`, `main.wgsl`, `shadow_mask.wgsl`), `point_light_position` 4 (`froxel.wgsl`, `light_count.wgsl`, `main.wgsl`, `shadow_mask.wgsl`), `position_at` 1 (`shadow_mask.wgsl`, 6 sites), `segment_meets_box` 1 (`shadow_mask.wgsl`), `octahedral_decode` 1 (`shadow_mask.wgsl`, 4 sites), `octahedral_encode` 1 (`main.wgsl`, 2 sites), `pack_index` 1 (`culling.wgsl`), `unpack_index` 1 (`main.wgsl`), `decode_vertex` 1 (`main.wgsl`), `rotate` 1 (`bend.wgsl`), `unpack_qtangent` 1 (`bend.wgsl`); internal only: `PackedIndex`, `TangentFrame`, `Vertex`, `decode_qtangent` | no: each function one module calls is under an oracle (`position_at`, `segment_meets_box`, octahedral, packed index, `decode_vertex`, `camera_relative_position`), which the convention keeps in a shared part; 6 of 10 shipping modules include the part | `src/render/shaders/common.wgsl:13-305` |
| `shading.wgsl` | 19: 7 constants, `Surface`, 11 functions | `main.wgsl` reads 6: `Surface`, `decode_surface`, `tangent_space_normal`, `reflected_luminance`, `point_light_luminance`, `ambient_luminance`; the oracle harness reads `PI` and `MINIMUM_ROUGHNESS` and calls 11 functions; 1 module includes the part (`MAIN`) | no: the 13 internal items are the oracles' seams | `src/render/shaders/shading.wgsl:11-268` |
| `main.wgsl` | 13: 6 entry points, 2 structs, `UNMASKED`, 4 functions | `main_pass.rs` names 6 entry points at 7 sites (`:119, 122, 132, 138, 141, 155, 161`); `coverage_tests.rs:286, 292` names `vs_main` and `fs_main` | no: nothing outside the pass reads a helper | `src/render/shaders/main.wgsl:5-307` |
| `hdr.wgsl` | 4: `LARGEST_HALF`, 3 functions | `main.wgsl:305-306` reads `LARGEST_HALF` and `resolve_weighted`; `tonemapping.wgsl:19` reads `resolve_unweighted`; `relative_luminance` internal, under an oracle; 2 modules include the part | no | `src/render/shaders/hdr.wgsl:7-32` |
| `display.wgsl` | 5: 2 matrices, 3 functions | `tonemapping.wgsl:19, 22` calls `tonemap`, `srgb_encode`, `dither_noise`; `scripts/cycles-reference/compare.py` ports all 3 and both matrices; 1 module includes the part | no: the three functions sit apart from `tonemapping.wgsl` for their oracles | `src/render/shaders/display.wgsl:8-51` |
| `gt7.wgsl` | 12: 6 constants, 6 functions | `display.wgsl:23` calls `gt7_tonemap` and reads `GT7_PAPER_WHITE`; `compare.py` ports all 6 functions; 1 module includes the part | no | `src/render/shaders/gt7.wgsl:32-124` |
| `tonemapping.wgsl` | 2 entry points | `tonemapping_pass.rs:64, 70` | no | `src/render/shaders/tonemapping.wgsl:7-24` |
| `culling.wgsl` | 6: 4 entry points (`offset_slots`, `offset_blocks`, `append_records`, `append_triangles`), the `PHASE_TWO` override, `FrameDraw` (read back by the CPU); 20 items private to the part | `culling_pass.rs:261-264` (the 4 entry points), `:245` (`PHASE_TWO`), `readback.rs:399` (`FrameDraw`'s layout through its Rust twin at `culling_pass.rs:82`) | no | `src/render/shaders/culling.wgsl:19`, `:45`, `:240`, `:294`, `:362`, `:401` |
| `depth_pyramid.wgsl` | 2: `reduce_depth`, `reduce_level`; 8 items private to the part | `depth_pyramid_pass.rs:54-55` | no | `src/render/shaders/depth_pyramid.wgsl:71`, `:127` |
| `froxel_grid.wgsl` | 5: `FROXEL_GRID_SIZE`, `froxel_index`, `froxel_slice_start`, `froxel_slice`, `froxel_at` | `froxel.wgsl` (`FROXEL_GRID_SIZE` 2, `froxel_index` 1, `froxel_slice_start` 2); `main.wgsl:275`, `shadow_mask.wgsl:262`, `light_count.wgsl:49` (`froxel_at`); `froxel_slice` only `froxel_at` at `:40` | no (`froxel_slice_start` serves one module; Notes 4) | `src/render/shaders/froxel_grid.wgsl:10-41` |
| `froxel.wgsl` | 1: `build_froxel_lists` | `froxel_pass.rs:103` | no | `src/render/shaders/froxel.wgsl:8` |
| `light_count.wgsl` | 1: `count_point_lights`; 5 items private to the part (`FIRST_SAMPLE`, `camera_relative_point`, 3 workgroup atomics) | `light_count_pass.rs:60` | no; two private items duplicate what the module already holds (L5-S05, L5-S06) | `src/render/shaders/light_count.wgsl:30` |
| `bend.wgsl` | 2: `bend_vertices`, `BendData` (CPU twin); 2 items private to the part | `bend_pass.rs:77` (the entry point), `bend_pass.rs:29`, `prepare.rs` (4 sites), `shader_tests.rs:302` (`BendData`) | no | `src/render/shaders/bend.wgsl:15`, `:44` |
| `shadow_mask.wgsl` | 4 entry points (`build_shadow_mask`, `trace_second_surfaces`, `list_shadow_edges`, `trace_shadow_edges`); 0 functions, structs or constants another part calls (`grep -lw` of each of its 33 names across `src/render/shaders/*.wgsl` lists only itself) | 4 pipelines, `shadow_mask_pass.rs:200` to `203` | no | `src/render/shaders/shadow_mask.wgsl:388`, `:481`, `:600`, `:631` |
| `dynamic_tlas_marker.wgsl` | 1 entry point | 1, `acceleration_structures.rs:280` | no | `src/render/shaders/dynamic_tlas_marker.wgsl:7` |
| `blas_refit_marker.wgsl` | 1 entry point | 1, `blas_refit.rs:412` | no | `src/render/shaders/blas_refit_marker.wgsl:5` |
| `mod.rs` `Part` | struct, 2 public fields (`path`, `source`) | `shader_tests.rs` builds 9 test parts (lines 79, 90, 142, 173, 177, 181, 353, 395, 422) and reads `path` at 128; `path` is read nowhere outside tests (`cfg_attr(not(test), allow(dead_code))` at `mod.rs:42`) | no | `src/render/shaders/mod.rs:40` |
| `mod.rs` `ShaderModule` | struct, 4 public fields, `descriptor`, `source`, `locate` (test) | `descriptor`: 10 passes (`grep -rn 'shaders::' src tests`: `culling_pass.rs:236`, `bend_pass.rs:68`, `froxel_pass.rs:94`, `depth_pyramid_pass.rs:34`, `acceleration_structures.rs:271`, `blas_refit.rs:403`, `tonemapping_pass.rs:53`, `light_count_pass.rs:51`, `shadow_mask_pass.rs:133`, `main_pass.rs:67`); `source`: `shader_tests.rs:445`, `shader_function_tests.rs:2200`; `locate`: `shader_tests.rs:448`; the fields: `shader_tests.rs` builds 8 test modules and reads `name`, `bind_groups` and `parts` (lines 50, 60, 127, 446, 511, 545, 627) | no | `src/render/shaders/mod.rs:79` |
| `mod.rs` module constants | 10 shipping (`MAIN` to `BEND`), `SHARED` and `MODULES` under `cfg(test)` | one pass each for the 10 (the list above); `SHARED`: `shader_function_tests.rs:2200`; `MODULES`: `shader_tests.rs:25`, `36`, `46`, `125`, `233` | no | `src/render/shaders/mod.rs:87` to `192` |
| `mod.rs` constants declared to WGSL | 26 scalar or vector constants plus `SAMPLE_OFFSETS` (lines 238 to 277) and 3 overrides (`CULLING`, `FROXEL_LIGHTS`, `SHADOW_RAYS`, `renderer.rs:154` to `160`) | 25 of the 26 read by some part; `MESHLET_MAX_TRIANGLES` by none (L5-S07); each override read by at least one part (`CULLING` 5 sites in `culling.wgsl`; `FROXEL_LIGHTS` 7 in `main.wgsl` and `shadow_mask.wgsl`; `SHADOW_RAYS` 2 in `main.wgsl`) | one constant | `src/render/shaders/mod.rs:240` |
| The GPU harness: `gpu_context`, `evaluate`, `evaluate_cases`, `KERNEL`, `WORKGROUP_SIZE` | 0 (5 private) | 0 | no | `gpu_context` 28 callers (every test), `evaluate` 27, `evaluate_cases` 9 (8 pass `&[]` as `vertices`, `cooked_vertices_decode_to_their_position_uv_and_tangent_frame` passes the vertices), `KERNEL` and `WORKGROUP_SIZE` read by `evaluate_cases` alone; `src/render/shader_function_tests.rs:2143-2296` |
| The f64 references: `reference_distribution`, `reference_visibility`, `reference_brdf`, `srgb_decode` | 0 (4 private) | 0 | no | 2, 2, 3 and 1 callers (`reference_brdf` calls the first two); `src/render/shader_function_tests.rs:1171-1284` |
| The case builders: `direction`, `cosines`, `surface`, `minimum_roughness`, `assert_close` | 0 (5 private) | 0 | no | 12, 3, 4, 7 and 25 callers; `src/render/shader_function_tests.rs:1197-1227`, `:1460-1467` |
| The tolerances and tables: `TOLERANCE`, `RELATIVE_TOLERANCE`, `DIRECTION_TOLERANCE`, `GT7_TOLERANCE`, `SRGB_TOLERANCE`, `DISPLAY_RAMP_TOLERANCE`, `FURNACE_SUM_TOLERANCE`, `QUADRATURE_TOLERANCE`, `PRECISION_BAR`, `GOLD`, `GT7_REFERENCE`, `TONEMAP_REFERENCE` | 0 (12 private constants) | 0 | no | mentions 8, 9, 3, 13, 2, 2, 2, 3, 4, 6, 3, 3; three constants have one use beside their definition (Notes 4) |
| `Case` (4 methods) with `sums_in_every_order` | 0 (1 struct, 4 methods, 1 function) | 0 | no | one test, `camera_relative_position_meets_the_precision_bar_within_100_km`, calls `Case::new` once, `test_vectors` once, `worst_error_in_any_order` and `world_space_error` once each; `src/render/shader_function_tests.rs:2054-2139` |
| The module validation: `validate`, `stage_problems`, `defined_binding`, `wgsl_files` | 0 (4 private) | 0 | no | 10, 2, 2 and 1 call sites; `src/render/shader_tests.rs:444-575` |
| The layout checker: `Twin`, `Field`, `ShaderScalar` with 5 impls, `scalar_of`, `twin!`, `layout_problems`, `structs_shared_with_the_cpu`, `twin_problems`, `wgsl_scalar`, `describe` | 0 (14 private items) | 0 | no | `twin!` 12 uses, `layout_problems` 3, `structs_shared_with_the_cpu` 1, `twin_problems` 1, `wgsl_scalar` 1 plus its recursion, `describe` 2, `scalar_of` 1 (inside `twin!`); `src/render/shader_tests.rs:213-229`, `:577-741` |
| What the tests consume of `shaders::` (`MODULES`, `Part`, `ShaderModule`, `SHARED.source()`, `locate`) | judged with the assembly, above | `shader_tests.rs` imports 3 names at :20 and calls `locate` at :448; `shader_function_tests.rs` calls `SHARED.source()` at :2200 | judged with the assembly, above | `src/render/shaders/mod.rs:170-176`, `:179-192`, `:211-223` |


For the two test modules, counts are `grep` over the file; a function's count excludes its definition, a constant's includes its definition and doc mentions (`grep -ow`). Neither file has a `pub` item, and `grep -rn 'shader_function_tests\|shader_tests' src tests scripts docs` finds no caller outside them: `src/render/mod.rs:24` and `:26` declare them under `#[cfg(test)]`, `tests/conventions.rs:279-280` names them as the two test-only files its `unwrap` rule skips, and the rest are doc mentions.

### Findings

| id | claim | severity | where | evidence | expected |
|---|---|---|---|---|---|
| L5-S01 | The header says the part is "Shared by every shader module", but 4 of the 10 shipping modules do not include it, and its list of contents omits 3 of the part's 12 functions. | drift; not load-bearing | `src/render/shaders/common.wgsl:1-3` | block below | `CLAUDE.md`, Code conventions: comments say what the code does. "Shared by the modules that read the frame, the geometry or the lights" (or `mod.rs:57-58`'s "The structs more than one pass reads"), listing the ray and box test, the point light's position and the octahedral codec with the rest. |
| L5-S02 | The comment on the BRDF's lobe mix carries a measured cost, "the lit pass took up to 25 µs (5%) longer at 1440p on the M4 Pro", that nothing in the tree re-measures or checks. | drift; not load-bearing | `src/render/shaders/shading.wgsl:181-182` | block below | `CLAUDE.md`, Code conventions: comments say what the code does and why. The why is the line before it, that Schlick's F is linear in F0 so one lobe at the mixed F0 is Appendix B's mix exactly, which the BRDF oracle pins; measured costs live in the Bench history. |
| L5-S03 | The comment on the first directional light carries a measured cost, "a branch around it cost the lit pass about 10% on the M4 Pro", that nothing in the tree re-measures or checks. | drift; not load-bearing | `src/render/shaders/main.wgsl:208-211` | block below | As L5-S02: the why, that a uniform branch around the shading costs more on the M4 Pro than shading a zero light, stands without the figure; the figure belongs to the Bench history. |
| L5-S04 | The 8 render entry points are named with the two-letter stage prefixes `vs_` and `fs_`, abbreviating vertex and fragment, while every compute entry point in the tree is named by what it does. | health; 8 names, 19 sites in 4 files | `src/render/shaders/main.wgsl:17` | block below | `CLAUDE.md`, Code conventions: identifiers are full words, WGSL included. Names that say what the stage does, as the compute parts' do (`build_shadow_mask`, `reduce_level`, `bend_vertices`): `project_vertex`, `project_depth_vertex`, `write_surface_normals`, `write_depth`, `shade_pixel`, `cover_target`, `tonemap_pixel`; or the full words `vertex_main` and `fragment_main`. |
| L5-S05 | `camera_relative_point` and the clip-space position built around it re-derive the point a depth shows through a pixel, which the shared part's `position_at`, under an oracle and included in the module, already computes. | health; about 17 lines, 1 call site | `src/render/shaders/light_count.wgsl:19` | block below | CLAUDE.md, Code conventions: "a function under an oracle lives in a shared part"; `common.wgsl:1-3` lists "the position a depth target's depth places at a pixel" among what the shared part holds. One call, `position_at(camera.view_proj, sample, pixel_depth, vec2<f32>(size))`, as `shadow_mask.wgsl:212` makes it. |
| L5-S06 | `FIRST_SAMPLE` declares the first sample's offset a second time, which `SAMPLE_OFFSETS[0]`, declared from Rust into every module and checked against the backend by a test, already holds. | health; 3 lines, 1 call site | `src/render/shaders/light_count.wgsl:10` | block below | One declaration per constant, in `mod.rs`'s generated prelude; `SAMPLE_OFFSETS[0]`, as `shadow_mask.wgsl:419` reads it. |
| L5-S07 | `assemble` declares `MESHLET_MAX_TRIANGLES` to every WGSL module and no part reads it | health; 2 lines, 0 call sites | `src/render/shaders/mod.rs:240` | block below | A constant declared to WGSL is one some part reads (the header, `mod.rs:3`, "the constants Rust shares with WGSL"); drop the line and the import at `mod.rs:8` |
| L5-S08 | The four-sample depth load and the nearest-sample pick are written out three times, once per dispatch that needs them, where one function would serve | health; 3 call sites, 20 lines | `src/render/shaders/shadow_mask.wgsl:397` | block below | One `pixel_depths(pixel) -> vec4<f32>` (and `nearest_sample(depths) -> u32`) in the part, called at the three sites, as `first_sample_of` already serves its four |
| L5-S09 | The list-append and indirect-dispatch-size block of `build_shadow_mask` is repeated verbatim in `list_shadow_edges`, differing only in the bindings it names | health; 2 call sites, 24 lines | `src/render/shaders/shadow_mask.wgsl:434` | block below | One function that appends an entry to a `PixelList` and grows its dispatch, called from both, with the `SHADOW_MASK_TILE_SIZE * SHADOW_MASK_TILE_SIZE` product (6 sites) a named constant |
| L5-S10 | A directional light's direction and the turn test against `INTERPOLATED_TURN_MARGIN` are spelled out at three sites, and `directional_hidden`'s body is repeated inline in `surface_word`'s directional branch | health; 3 call sites, 9 lines | `src/render/shaders/shadow_mask.wgsl:281` | block below | One `directional_light_direction(index)` and one `turns_from(normal, direction) -> bool`, which `directional_hidden`, `surface_word` and `turned_lights_other_samples_face` call |
| L5-S11 | The part's header describes the first two of its four dispatches: it never mentions that pixels at a directional light's shadow edge, or whose nearest sample turns from a light another sample faces, are listed and traced again per sample into `shadow_mask_samples` | drift; not load-bearing | `src/render/shaders/shadow_mask.wgsl:1` | block below | A header that says what the part does (CLAUDE.md, Code conventions: comments say what the code does and why): the four dispatches, the second layer and the per-sample verdicts, as `shadow_mask_pass.rs:1` to `12` and ADR 0020 state them |
| L5-S12 | The `SHADOW_RAY_PIXEL_OFFSET` comment sizes the constant from a measurement (points within 0.0031 of a pixel's width, 320x180 to 2560x1440, the first distance as little as 0.0033 at 320x180) that nothing in the tree records or reproduces | drift; not load-bearing | `src/render/shaders/shadow_mask.wgsl:41` | block below | A number a shipping comment relies on is checked by a test or recorded with its run in `docs/` (as `docs/performance.md` records `first_intersection`'s A/B, cited from `shadow_mask.wgsl:95`); the comment then cites the record |
| L5-S13 | Five test locals are a letter and a digit, `m1`, `m2`, `c1`, `c2`, `c3`, against the full-word rule, where the shader and the Cycles port spell the same constants `PQ_M1` to `PQ_C3`. | health; 5 identifiers on 4 lines in 1 file | `src/render/shader_function_tests.rs:809` | block below | `CLAUDE.md`, Code conventions: identifiers are full words, a textbook symbol goes in a comment. The spelling the shader (`gt7.wgsl:36-40`) and `scripts/cycles-reference/compare.py:55-56` share, `pq_m1`, `pq_m2`, `pq_c1`, `pq_c2`, `pq_c3`, which names the standard the constants come from. |
| L5-S14 | The comment "The near plane's distance over the depth along the view axis" heads `let near = f64::from(columns[3][2])`, which is the near plane's distance alone; the quotient it describes is computed seven lines below. | drift; not load-bearing | `src/render/shader_function_tests.rs:1630` | block below | `CLAUDE.md`, Code conventions: comments say what the code beside them does. The comment belongs beside `depth` at :1638, or reads "The near plane's distance, which over the depth along the view axis gives the depth"; `common.wgsl:44-47` states the relation correctly. |
| L5-S15 | The test name says `tangent_space_normal` "normalises as glTF writes it", but the function returns an unnormalised vector and the test normalises it in its own WGSL wrapper, so no assertion pins a normalisation by the function. | drift; not load-bearing | `src/render/shader_function_tests.rs:1298` | block below | A test's name claims what its assertions pin. The assertions pin the direction of the function's result against glTF's formula, so a name of that shape serves: `tangent_space_normal_scales_x_and_y_and_points_where_gltf_normalises_to`, or the "normalises" clause dropped. |


#### L5-S01

`src/render/shaders/common.wgsl:1-3`:

```
// Shared by every shader module: the structs the CPU writes, the Anchor
// composition, the position a depth target's depth places at a pixel, the
// packed index and the vertex decode (src/render/shaders/mod.rs).
```

`src/render/shaders/mod.rs` lists the parts of each module. `COMMON` is in `MAIN` (`:92`), `SHADOW_MASK` (`:101`), `CULLING` (`:109`), `FROXEL` (`:117`), `LIGHT_COUNT` (`:125`) and `BEND` (`:166`), and in the test-only `SHARED` (`:175`). It is not in `DEPTH_PYRAMID` (`:133`, `parts: &[part!("depth_pyramid.wgsl")]`), `TONEMAPPING` (`:141`, `parts: &[HDR, GT7, DISPLAY, part!("tonemapping.wgsl")]`), `DYNAMIC_TLAS_MARKER` (`:149`) or `BLAS_REFIT_MARKER` (`:158`): `grep -c COMMON src/render/shaders/mod.rs` counts 8 lines, the declaration plus 7 module lists, against 11 modules. The same file's own doc for the part, `mod.rs:57-58`, says "The structs more than one pass reads", which is the accurate form.

The header's list names the structs, `camera_relative_position`, `position_at`, `pack_index` and `unpack_index`, and the vertex decode. It does not name `segment_meets_box` (`:69-97`, 29 lines, the ray-against-box test the shadow mask pass calls), `point_light_position` (`:121-133`, called from 4 parts) or `octahedral_encode` and `octahedral_decode` (`:293-305`, the surface normals' codec). A reader who trusts the header to say what every module can call, or what the part holds, is misled on both counts, but writing code on either belief fails at shader validation rather than at the image, so the drift is not load-bearing. The fix is a comment: it cannot move a pixel.

#### L5-S02

`src/render/shaders/shading.wgsl:177-182`:

```
    // The mix, gathered by lobe. A dielectric's surface reflects F of the
    // light, from its fixed F0, and the rest enters, scatters and leaves
    // diffuse; a metal's F0 is its base colour. Schlick's F is linear in F0,
    // so metallic's mix of the two specular lobes is one lobe whose F is at
    // the mixed F0, which no light changes. Mixed as Appendix B writes it,
    // the lit pass took up to 25 µs (5%) longer at 1440p on the M4 Pro.
```

`grep -rn "25 µs" docs src` finds the figure in `docs/performance.md:2491` (the culling pass's fixed cost, a different measurement) and in this comment only; `grep -rn "lobe" docs/performance.md` finds one line about Toksvig; `grep -rn "by lobe\|one-lobe" docs src` finds this comment and `docs/research/source-audit.md:17`, which records that the one-lobe mix expands to the source's exactly and names no cost. No Baseline, Bench record or performance note holds the 25 µs or the 5%, and the lit pass's cost has moved many times since (the shadow mask, the Froxel lists, the second surfaces). The sentence before the figure is the why, and `metallic_roughness_brdf_matches_appendix_b_and_is_reciprocal` (`src/render/shader_function_tests.rs:210`) pins that the one-lobe form is Appendix B's to `RELATIVE_TOLERANCE`. The same form as the render section's L5-R13. The fix is a comment: it cannot move a pixel.

#### L5-S03

`src/render/shaders/main.wgsl:208-211`:

```
    // The first directional light is shaded whether or not the light buffer
    // holds one, since a branch around it cost the lit pass about 10% on
    // the M4 Pro: without one, its illuminance is zero and a fixed direction
    // keeps the term finite. Any others follow.
```

`grep -rn "about 10%\|10% on the M4\|branch around" docs src` finds this comment alone; `grep -rn "10%" docs/performance.md docs/research/*.md` filtered to "lit pass", "directional" or "branch" finds two unrelated lines (shadow map research, an HDR pixel share). Nothing re-measures the branch's cost, and the code the comment guards, the hoisted first light at lines 212-248 beside the loop at 249-261, is what a reader needs the why for: that a uniform branch around the shading is dearer than shading a zero light on the M4 Pro. That sentence holds without its figure. The same form as L5-S02 and the render section's L5-R13. The fix is a comment: it cannot move a pixel.

#### L5-S04

The 8 render entry points, `grep -n "^fn vs_\|^fn fs_" src/render/shaders/*.wgsl`:

```
main.wgsl:17:fn vs_depth_prepass(...)
main.wgsl:34:fn vs_depth_normals(...)
main.wgsl:51:fn fs_surface_normals(...)
main.wgsl:78:fn fs_depth() {}
main.wgsl:81:fn vs_main(...)
main.wgsl:171:fn fs_main(...)
tonemapping.wgsl:7:fn vs_full_screen(...)
tonemapping.wgsl:16:fn fs_tonemap(...)
```

`vs` and `fs` stand for vertex stage and fragment stage; neither is on the full-word rule's list of the domain's own words, nor a WGSL, naga, wgpu or glam API name (WGSL names the stages `@vertex` and `@fragment`). The tree's 13 compute entry points take the other form, a verb phrase: `build_shadow_mask`, `trace_second_surfaces`, `list_shadow_edges`, `trace_shadow_edges` (`shadow_mask.wgsl`), `reduce_depth`, `reduce_level` (`depth_pyramid.wgsl`), `build_froxel_lists`, `count_point_lights`, `bend_vertices`, `offset_slots`, `offset_blocks`, `append_records`, `append_triangles`, `mark_dynamic_tlas`, `mark_blas_refit`. Sites: the 8 definitions; `grep -rno 'entry_point: Some("[a-z_]*")' src/render/*.rs` names them at `main_pass.rs:119, 122, 132, 138, 141, 155, 161` (7) and `tonemapping_pass.rs:64, 70` (2); `coverage_tests.rs:286, 292` names `vs_main` and `fs_main` for its own test WGSL (2, which the render section deferred here as the engine's convention). 19 sites in 4 files. `tests/conventions.rs` catches no stage prefix. Weighed against filing: wgpu's examples spell their entry points `vs_main` and `fs_main`, so the form is a familiar idiom; but an idiom from a tutorial is not an API name, and `CLAUDE.md` names the rule for WGSL in so many words. A rename of entry points changes the strings `main_pass.rs` and `tonemapping_pass.rs` pass and nothing the GPU runs: it cannot move a pixel, so the fix needs a bit-exact Golden check only.

#### L5-S05

`src/render/shaders/light_count.wgsl:16-27`:

```wgsl
// The point relative to the camera at clip-space position `clip`: the
// solution of view_proj's x, y and w rows, which hold no translation (ADR
// 0014), by Cramer's rule in cross products.
fn camera_relative_point(clip: vec4<f32>) -> vec3<f32> {
    let rows = transpose(camera.view_proj);
    let x_row = rows[0].xyz;
    let y_row = rows[1].xyz;
    let w_row = rows[3].xyz;
    let y_cross_w = cross(y_row, w_row);
    return (clip.x * y_cross_w + clip.y * cross(w_row, x_row) + clip.w * cross(x_row, y_row))
        / dot(x_row, y_cross_w);
}
```

and its one call site, `:40-48`:

```wgsl
            let near = camera.view_proj[3].z;
            let view_depth = near / pixel_depth;
            let sample = vec2<f32>(pixel.xy) + FIRST_SAMPLE;
            let normalised = vec2<f32>(
                sample.x / f32(size.x) * 2.0 - 1.0,
                1.0 - sample.y / f32(size.y) * 2.0
            );
            let clip = vec4<f32>(normalised * view_depth, near, view_depth);
            let position = camera_relative_point(clip);
```

`src/render/shaders/common.wgsl:49-56` computes the same point from the same inputs:

```wgsl
fn position_at(view_proj: mat4x4<f32>, point: vec2<f32>, depth: f32, size: vec2<f32>) -> vec3<f32> {
    let device = vec2<f32>(point.x / size.x * 2.0 - 1.0, 1.0 - point.y / size.y * 2.0);
    let rows = transpose(view_proj);
    let view_depth = view_proj[3].z / depth;
    let right = rows[0].xyz / dot(rows[0].xyz, rows[0].xyz);
    let up = rows[1].xyz / dot(rows[1].xyz, rows[1].xyz);
    return view_depth * (rows[3].xyz + device.x * right + device.y * up);
}
```

Counts. `grep -rnw camera_relative_point src tests docs scripts`: 2 lines, both in `light_count.wgsl` (`:19` the definition, `:48` the call). `grep -rnw position_at src tests`: `common.wgsl:49` (the definition), `shadow_mask.wgsl` 6 call sites (`:212`, `:216`, `:419-422`), `shader_function_tests.rs:1618` (the oracle `position_at_rebuilds_the_point_a_depth_shows_through_a_pixel`, which holds it within 2^-20 relative of the f64 inverse over 30 pixels and 6 distances at 2560 x 1440). The `LIGHT_COUNT` module's parts are `COMMON, FROXEL_GRID, light_count.wgsl` (`mod.rs:125`), so `position_at` is in scope at `:48`. The two forms agree: `view_proj`'s x, y and w rows are mutually orthogonal (a rigid view under a projection without skew), so Cramer's solution of `rows[0] . p = clip.x`, `rows[1] . p = clip.y`, `rows[3] . p = clip.w` is `clip.x rows[0] / |rows[0]|^2 + clip.y rows[1] / |rows[1]|^2 + clip.w rows[3]`, which with `clip = (normalised * view_depth, near, view_depth)` is `position_at`'s expression term for term. The `normalised` computation at `:43-46` repeats `position_at`'s `device` line. Smaller form: `let view_depth = camera.view_proj[3].z / pixel_depth;` stays for `froxel_at`; `camera_relative_point`, `near`, `normalised` and `clip` go, and `:48` becomes `let position = position_at(camera.view_proj, sample, pixel_depth, vec2<f32>(size));`: about 17 lines fewer, one function fewer in the part.

Pixel movement: the light count pass writes no image, so no Golden moves. It writes the Bench counters `lit_pixels`, `listed_point_lights` and `reaching_point_lights` (`common.wgsl:112-117`, read back at `readback.rs:410`). `lit_pixels` and `listed_point_lights` cannot change (the depth test at `:39` and `froxel_at`'s inputs are unchanged). `reaching_point_lights` compares `dot(to_light, to_light) < range^2` at `:55` from a position whose last-bit rounding differs between the two expressions, so a light at exactly its range at some pixel could flip by one; the fix ticket compares the three counters with the Baselines rather than relying on bit-exactness.

#### L5-S06

`src/render/shaders/light_count.wgsl:8-10`:

```wgsl
// Where a pixel's first sample lies within it: WebGPU's standard 4x
// pattern puts sample 0 at (0.375, 0.125) [webgpu-standard-sample-pattern].
const FIRST_SAMPLE: vec2<f32> = vec2<f32>(0.375, 0.125);
```

`src/render/shaders/mod.rs:269-277` declares, into every module's prelude, `const SAMPLE_OFFSETS: array<vec2<f32>, 4> = array<vec2<f32>, 4>(vec2<f32>(0.375, 0.125), ...)` from `shadow_mask_pass.rs:41-46`, whose doc says "WGSL declares them as `SAMPLE_OFFSETS`, and a test checks the backend's depth target against them": `shadow_mask_pass.rs:416`, `each_depth_sample_lies_where_the_pass_rebuilds_it`. `shadow_mask.wgsl:419` reads the same offset as `SAMPLE_OFFSETS[0]`. Counts: `grep -rnw FIRST_SAMPLE src tests docs scripts`: `light_count.wgsl:10` (the definition), `:42` (the one use), `docs/research/source-audit.md:76` (the source audit's row naming it as the citation's site). `grep -rnw SAMPLE_OFFSETS src/render/shaders`: `mod.rs` 3, `shadow_mask.wgsl` 7. The second declaration carries its own citation and its own literal, which the test checking `SAMPLE_OFFSETS` does not cover. Smaller form: `let sample = vec2<f32>(pixel.xy) + SAMPLE_OFFSETS[0];` at `:42`, and lines 8-10 go; the citation then lives once, at `shadow_mask_pass.rs:38`, and `docs/research/source-audit.md:76` is updated to name it (falls in lane 7's docs).

Pixel movement: none. The value is the same literal (`{:?}` of the f32 0.375 prints `0.375`, of 0.125 prints `0.125`), so the assembled module differs only in the constant's name; Goldens bit-exact, counters equal.

#### L5-S07

`src/render/shaders/mod.rs:8` and `mod.rs:240`:

```rust
use crate::cooked::{MESHLET_MAX_TRIANGLES, MESHLET_MAX_VERTICES, QTANGENT_SMALLEST_W};
...
             const MESHLET_MAX_TRIANGLES: u32 = {MESHLET_MAX_TRIANGLES}u;\n\
```

The grep over every part: `grep -ow MESHLET_MAX_TRIANGLES src/render/shaders/*.wgsl | wc -l` gives 0. Every other name `assemble` declares is read by at least one part: `grep -lw <name> src/render/shaders/*.wgsl` lists `common.wgsl` for `ANCHOR_CELL_SIZE` (2 sites) and `MESHLET_VERTEX_BITS` (4), `culling.wgsl` for `SCAN_BLOCK_SIZE` (18), `INSTANCE_CAPACITY` (6) and `NO_MESH` (2, with `common.wgsl`), `shadow_mask.wgsl` for `SAMPLE_OFFSETS` (9) and `SHADOW_MASK_TILE_SIZE` (16), and so on through the 26; only the Meshlet triangle cap has no reader. Across the crate the constant is read by the cooker and the cooked reader alone: `grep -rn MESHLET_MAX_TRIANGLES src tests scripts docs` lists `src/assets/cooked.rs:44` (its definition) and `:445`, `src/cook/mesh.rs:12`, `:31` and `:180`, and the two `mod.rs` lines. The culling pass reads each Meshlet's `triangle_count` from `MeshletData` rather than the cap, so no part needs it. The fix removes one declaration and one import name; it changes no module's behaviour and cannot move a pixel (a Golden check is a formality; the Goldens are bit-exact).

#### L5-S08

The same block three times. `build_shadow_mask`, `src/render/shaders/shadow_mask.wgsl:397` to `405`:

```wgsl
    let depths = vec4<f32>(
        textureLoad(depth_target, pixel, 0),
        textureLoad(depth_target, pixel, 1),
        textureLoad(depth_target, pixel, 2),
        textureLoad(depth_target, pixel, 3)
    );
    // The nearest sample has the greatest depth (ADR 0015); the first such.
    let depth = max(max(depths.x, depths.y), max(depths.z, depths.w));
    let sample = first_sample_of(depths, depth);
```

`turned_lights_other_samples_face`, `shadow_mask.wgsl:567` to `573`, loads the same `vec4` and picks `first_sample_of(depths, max(max(depths.x, depths.y), max(depths.z, depths.w)))`; `trace_shadow_edges`, `shadow_mask.wgsl:649` to `655`, does both again. `grep -c 'textureLoad(depth_target, pixel, 0)' src/render/shaders/shadow_mask.wgsl` gives 3; `grep -c 'max(max(depths.x, depths.y), max(depths.z, depths.w))'` gives 3. The comment at line 395 to 396 explains why the four loads are spelled out rather than looped (Metal's compiler does not unroll a loop over them); that reason holds inside a function as well as at each site, so `fn pixel_depths(pixel: vec2<i32>) -> vec4<f32>` with the four loads, and `fn nearest_sample(depths: vec4<f32>) -> u32`, serve the three sites and keep the explanation in one place. The part already does this for the four-way pick (`first_sample_of`, 4 callers) and for the other-samples mask (`other_nearest_samples`, 2 callers). Moving the loads and the `max` chain into a function reorders no floating-point operation, so the fix cannot move a pixel; the Goldens stay bit-exact.

#### L5-S09

`build_shadow_mask`, `src/render/shaders/shadow_mask.wgsl:434` to `447`:

```wgsl
            let entry = atomicAdd(&second_surfaces.count, 1u);
            second_surfaces.entries[entry] = invocation.x | (invocation.y << 16u) | (second_sample << 30u);
            // The entry that opens a workgroup's run of the second dispatch
            // adds that workgroup, in rows of MAX_WORKGROUPS_PER_DIMENSION.
            let group_size = SHADOW_MASK_TILE_SIZE * SHADOW_MASK_TILE_SIZE;
            if entry % group_size == 0u {
                let groups = entry / group_size + 1u;
                atomicMax(&second_surface_dispatch[0], min(groups, MAX_WORKGROUPS_PER_DIMENSION));
                atomicMax(
                    &second_surface_dispatch[1],
                    (groups + MAX_WORKGROUPS_PER_DIMENSION - 1u) / MAX_WORKGROUPS_PER_DIMENSION
                );
                atomicStore(&second_surface_dispatch[2], 1u);
            }
```

`list_shadow_edges`, `shadow_mask.wgsl:610` to `621`, is the same twelve lines with `shadow_edges` for `second_surfaces`, `shadow_edge_dispatch` for `second_surface_dispatch`, and no sample in bit 30. `grep -c 'atomicMax(&' src/render/shaders/shadow_mask.wgsl` gives 4, two per site; `grep -c 'atomicStore(&' ` gives 2; `grep -c 'SHADOW_MASK_TILE_SIZE \* SHADOW_MASK_TILE_SIZE'` gives 6 (lines 438, 480, 489, 612, 630, 639), the workgroup's size of the two list-reading dispatches recomputed at each use. One function, `fn list_pixel(list: ptr<storage, PixelList, read_write>, dispatch: ptr<storage, array<atomic<u32>, 3>, read_write>, entry_bits: u32)`, serves both sites, and a `const LIST_GROUP_SIZE: u32 = SHADOW_MASK_TILE_SIZE * SHADOW_MASK_TILE_SIZE;` the six. The fix leans on naga accepting pointers to storage as function parameters, which WGSL's core grammar does not and naga's frontend does; if a future naga refuses them, the two sites stay and the constant alone is extracted. The fix changes no arithmetic and no atomic's order, so it cannot move a pixel; the dispatch counts are the same words.

#### L5-S10

The directional light's direction and its turn test, three times. `directional_hidden`, `src/render/shaders/shadow_mask.wgsl:225` to `229`:

```wgsl
fn directional_hidden(surface: SampleSurface, index: u32) -> bool {
    let direction = normalize(-lights[index].offset);
    return !(dot(surface.interpolated_normal, direction) > -INTERPOLATED_TURN_MARGIN)
        || shadowed(surface.position, surface.pixel_width, surface.triangle_normal, surface.interpolated_normal, direction, UNBOUNDED_RAY_DISTANCE);
}
```

`surface_word`'s directional branch, `shadow_mask.wgsl:281` to `286` and `343`, repeats the body inline:

```wgsl
                direction = normalize(-lights[entry].offset);
                distance = UNBOUNDED_RAY_DISTANCE;
                if !(dot(interpolated_normal, direction) > -INTERPOLATED_TURN_MARGIN) {
                    hidden = with_bit(hidden, entry);
                    continue;
                }
```

and `turned_lights_other_samples_face`, `shadow_mask.wgsl:584`, spells the pair out a third time:

```wgsl
                && dot(interpolated_normal, normalize(-lights[index].offset)) > -INTERPOLATED_TURN_MARGIN {
```

`grep -c 'normalize(-lights\[' src/render/shaders/shadow_mask.wgsl` gives 3; `grep -c '> -INTERPOLATED_TURN_MARGIN' ` gives 5, the three directional sites and the two point-light sites at lines 311 and 328, which scale the margin by the unnormalised distance and so take a different form. `surface_word` cannot call `directional_hidden` whole, since it records which lights it traced a ray toward (`traced |= 1u << entry`, line 288) between the turn test and the ray; but `fn directional_light_direction(index: u32) -> vec3<f32>` and `fn turns_from(normal: vec3<f32>, direction: vec3<f32>) -> bool` serve all three sites and name the test the comments at lines 61 to 71 describe. The extracted expressions are the same operations in the same order, so the fix cannot move a pixel; the Goldens stay bit-exact. (`directional_hidden` itself has one caller, `trace_shadow_edges` at line 673; it is kept as the name of the per-sample verdict, not filed.)

#### L5-S11

The header, `src/render/shaders/shadow_mask.wgsl:1` to `7`:

```wgsl
// The shadow mask pass (src/render/shadow_mask_pass.rs): an invocation per
// pixel traces one hard shadow ray toward each light that reaches the surface
// the pixel's nearest sample shows (ADR 0020), and keeps a bit per light that
// its ray found hidden, for the lit pass to read. Where other samples of the
// pixel show another surface, it does the same from the farthest of them
// into the mask's second layer, and marks which samples show it, so each
// fragment of an edge pixel reads its own surface's bits.
```

The rest of the header (lines 7 to 19) covers the bit order, the overflow count, lights that cast no shadow and the inputs. Nothing in it names the third and fourth dispatches, `list_shadow_edges` (lines 592 to 622) and `trace_shadow_edges` (lines 624 to 681), which together with `directional_verdicts`, `shadow_edges`, `shadow_edge_dispatch` and the `shadow_mask_samples` texture make 90 of the part's 681 lines: a pixel whose verdict for one of the first `SHADOW_EDGE_LIGHTS` directional lights differs from a 4-neighbour's, or whose nearest sample turns from such a light that another sample of its surface faces, is traced again from each other sample, and the lit pass shades that light by the share of its samples that see it. The `shadow_mask_pass.rs` header (lines 8 to 11) and ADR 0020's amendment both state that form; the part's header stops at the form before it. The entry points' own comments (lines 381 to 386, 592 to 598, 624 to 629) are accurate, so a reader who reads past the header is not misled: not load-bearing. A comment fix; it cannot move a pixel.

#### L5-S12

`src/render/shaders/shadow_mask.wgsl:41` to `48`:

```wgsl
// that grid, by estimate, as no specification bounds it. Measured on the
// M4 Pro, whose grid Metal does not document, every Reference scene's
// points lie within 0.0031 of a pixel's width of their triangles, from
// 320x180 to 2560x1440, while the first distance spans as little as 0.0033
// of a pixel's width at 320x180 through the camera's 45° field of view, and
// less through a wider one. This share is ten times the measured error and
// half a step of a 1/16 grid.
const SHADOW_RAY_PIXEL_OFFSET: f32 = 0.03125; // 2^-5
```

The constant's value rests on the measured 0.0031: "ten times the measured error". `grep -rn '0\.0031\b\|0\.0033\|width of their triangles\|SHADOW_RAY_PIXEL_OFFSET' src tests scripts docs` finds the number at this comment alone (the other hits are sRGB's 0.0031308); no test, script, Bench record or page under `docs/` records the run (adapter, commit, scenes, resolutions) or reproduces it, where the part's other measured claim, the one-call ray query at line 95, cites `docs/performance.md`, which holds the A/B at lines 643 to 690. The neighbouring margins' claim, 0.95° for the 8-bit octahedral normals at line 64, is pinned by an oracle (`shader_function_tests.rs:1680`, `octahedral_coordinates_decode_to_the_unit_vector_they_encode`). Not load-bearing: a reader keeps the constant and its reasoning, and nothing says the number is wrong; the Finding is that nothing in the tree can say it is right. The fix is a record in `docs/` (or a test over the Reference scenes' depth against their triangles) that the comment cites, not a code change; it cannot move a pixel.

#### L5-S13

`src/render/shader_function_tests.rs:809-817`:

```rust
    let (m1, m2) = (2610.0 / 16384.0, 2523.0 / 4096.0 * 128.0);
    let (c1, c2, c3) = (
        3424.0 / 4096.0,
        2413.0 / 4096.0 * 32.0,
        2392.0 / 4096.0 * 32.0,
    );
    for ((value, encoded), decoded) in frame_buffer_values.into_iter().zip(&encoded).zip(decoded) {
        let normalized = (f64::from(value) * 100.0 / 10000.0).powf(m1);
        let expected = ((c1 + c2 * normalized) / (1.0 + c3 * normalized)).powf(m2);
```

The names stand for SMPTE ST 2084's exponents m₁, m₂ and its rational coefficients c₁, c₂, c₃, the standard's own symbols. `grep -rnw 'm1\|m2\|c1\|c2\|c3' src --include='*.rs' --include='*.wgsl'` finds these 4 lines and no other site in the crate. The shipping spelling is prefixed: `src/render/shaders/gt7.wgsl:36-40` declares `PQ_M1`, `PQ_M2`, `PQ_C1`, `PQ_C2`, `PQ_C3`, and `scripts/cycles-reference/compare.py:55-56` the same five names, so the test is the one place the constants lose the word that says what they are. The full-word rule allows a textbook symbol in a comment only; `tests/conventions.rs:190-214` catches only the three shading dot spellings, not this. The lane's list of the domain's own words holds `pq` but not a bare symbol-and-digit, so the row is filed; if the triage reads the standard's symbols as the domain's own words, it is a kill. The fix is a rename in a test: it cannot move a pixel.

#### L5-S14

`src/render/shader_function_tests.rs:1630-1638`:

```rust
    // The near plane's distance over the depth along the view axis.
    let near = f64::from(columns[3][2]);
    let points = [0.0, 0.375, 640.5, 1279.875, 2559.625, 2560.0]
        .into_iter()
        .flat_map(|x| [0.0, 0.125, 720.5, 1439.875, 1440.0].map(|y| [x, y]));
    let mut cases = Vec::new();
    for [x, y] in points {
        for distance in [0.1, 0.35, 2.0, 40.0, 900.0, 10_000.0] {
            let depth = (near / distance) as f32;
```

`columns[3][2]` is `view_proj`'s column 3, row 2, the constant that a reverse-Z projection with no far plane puts in clip z: the near plane's distance, as `src/render/shaders/common.wgsl:44-47` says ("puts the near plane's distance in clip z and the depth along the view axis in clip w, so depth is the near distance over clip w"). The comment's noun phrase names a ratio, near over view depth, which is `depth` at :1638, not `near` at :1631, the line it heads. A reader of the local alone takes `near` for a depth value. Not load-bearing: the correct statement sits in the shared part's doc, and the test's own assertion against the f64 inverse at :1657-1673 would catch a wrong use. The fix moves or rewords a comment: it cannot move a pixel.

#### L5-S15

The test, `src/render/shader_function_tests.rs:1298` and its expression at :1314-1318:

```rust
fn tangent_space_normal_scales_x_and_y_and_normalises_as_gltf_writes_it() {
    ...
    let results = evaluate(
        &context,
        "vec4<f32>(normalize(tangent_space_normal(input.xy, input.z)), 0.0)",
        &inputs,
    );
```

The function, `src/render/shaders/shading.wgsl:156-160`, returns `vec3<f32>(x_and_y * scale, select(z, 1.0, scale == 0.0))`, whose length is `sqrt(scale² (x² + y²) + z²)`, not 1 for a scale other than 1; its doc at :152-154 says "which glTF normalises last: the lit pass does, once its orthonormal tangent frame has turned the result into World space". The test's `normalize(...)` is the harness's, and the oracle at :1324-1326 normalises its own f64 vector, so the assertions at :1327-1337 pin the direction of the function's result and nothing about its length: the "normalises" the name claims is a property no assertion pins, which the lane's definition of `drift` names. The test's doc at :1289-1291 does say who normalises, and the function's doc says the same, so a reader has the correct statement two places over: not load-bearing. The fix renames a test: it cannot move a pixel.

### Notes for the verifier

Not filed, since each is a near-miss under the lane's axes, an allowed form, or belongs to another lane; recorded so the next audit does not re-find them. By reader: A shading and the display, B geometry, culling and lights, C the shadow mask and the assembly, D the shader tests.


**A.**

1. `view_proj` in `common.wgsl` (lines 14, 44, 47, 49, 51, 52) is counted under the render section's `view_proj` row (its 23 WGSL sites); not re-filed.
2. `tests/conventions.rs` is green at this commit: no `n_dot_v`-style name, no ticket number in a comment, no `println!`, no `Less` depth and no sRGB format exists in these files; the ticket-number rule explains why `shading.wgsl:182` and `main.wgsl:209` carry a figure and not a reference to where it was measured.
3. `shading.wgsl:108-111`, "summed over the hemisphere gave a white metal seen head-on 1.1% more light than it received", is a measured figure like L5-S02's, but the property it motivates is pinned: `reflected_luminance_follows_the_brdf_and_a_white_metal_reflects_at_most_what_it_receives` (`shader_function_tests.rs:290`) sums the white furnace at the roughness floor from 0°, 60° and 85° and asserts the white metal reflects at most 1 + 1e-3. The figure is stale by design (it describes the form the code no longer takes) and the fix it motivated is under test, so it was weighed and not filed; the verifier may narrow L5-S02 and L5-S03 the same way if the triage wants one row for the three figures.
4. `main.wgsl:212-261`: the first directional light is shaded outside the loop with `select`-ed inputs, so `reflected_luminance` appears twice where one loop from 0 would serve. The duplicate exists for the measured branch cost L5-S03 cites; folding it back into the loop is a cost question (lane 3), and a branch for a `select` can change the image on the M4 Pro, so it is not a health refactor. Not filed.
5. `main.wgsl:1-3`, "The main pass's entry points, the depth pre-pass's and the lit pass's": the surface normals pipeline (`vs_depth_normals`, `fs_surface_normals`) is the depth pre-pass's with shadow rays on (`main_pass.rs:116-130`) and `fs_depth` the lit pass's depth pipeline (`:135-146`), and phase two's depth pass reuses the pre-pass pipeline, so every entry point is one of the two the header names. Checked, clean.
6. `MaterialData.emissive.w`: `src/assets/material.rs:72` documents it as `w=strength` and `material.rs:284` writes `1.0` there, but no part reads it (`grep -n "\.emissive" src/render/shaders/*.wgsl` finds `shading.wgsl:88`, `.rgb`, and `main.wgsl:302`, the `Surface` field). A field written and never read, whose misdescribing comment is in `material.rs`; `common.wgsl:139` carries no comment on it. Falls in assets.
7. `hdr.wgsl:16-17`, "as every shader of the lit pass writes it": the lit pass has one colour-writing fragment entry, `fs_main`, and `fs_depth`, which writes none, so the plural is vacuously true. Not filed.
8. `gt7.wgsl:47`, `let peak = GT7_PAPER_WHITE;`, is an alias with 4 reads in a function with no other caller of the constant; the sample takes the peak as a parameter, and the alias keeps the ported lines readable against it. Adapted code under the MIT notice; not filed.
9. `bounds_min`, `bounds_max`, `box_min`, `box_max`, `dynamic_tlas_bounds_min`, `dynamic_tlas_bounds_max` (`common.wgsl:20-21, 74-75, 192-193`): `min` and `max` are WGSL's and glam's own function names and the render section judged the Rust twins clean (`uniforms.rs:62, 64`); weighed as the domain's spelling, not filed. If the verifier reads them as abbreviations of minimum and maximum, the sites are those 6 plus `culling.wgsl:120-128` and `shadow_mask.wgsl:108-112`, 14 in all.
10. `PQ_M1`, `PQ_M2`, `PQ_C1`, `PQ_C2`, `PQ_C3` (`gt7.wgsl:36-40`) are BT.2100-3's own symbols m1, m2, c1, c2, c3, which the comment at lines 34-35 names as the source of the spelling, and the part is adapted code; `lms` (`:85, 90-93, 99, 105-107`) is the LMS cone space, `rgb` and `ictcp` colour spaces. All weighed as the domain's words.
11. `point_light_position` (`common.wgsl:121-133`): all 4 callers pass `camera.anchor, camera.offset` (`froxel.wgsl:38`, `light_count.wgsl:54`, `shadow_mask.wgsl:296`, `main.wgsl:285`), the shape of "a parameter every caller passes the same value". The part is also assembled into `BEND` (`mod.rs:166`, bind group `BENDS` alone) and the oracle harness's `SHARED` (`mod.rs:175`, no bind groups), where no `camera` is declared, so the function cannot read the binding itself and the parameters are load-bearing. Not filed.
12. `common.wgsl:144`, `};` after `MaterialData`: the one struct among the 17 parts closed with a semicolon (`grep -n "^};" src/render/shaders/*.wgsl`). WGSL accepts the empty declaration. One character; below the bar, recorded so the next audit sees it was read.
13. `shading.wgsl:1-2`, "shared by every pass that lights a surface": one module, `MAIN`, includes the part; true as written. The part is a shared part for the oracle harness, which `mod.rs:5-6` says.
14. The ν formula at `shading.wgsl:124-126` and `smith_visibility` match the quote of `gltf-2.0-specular-brdf` (the height-correlated form), while `gltf-2.0.1-sample-brdf`'s quote names the published 2.0.1's separable form, as the header at lines 4-7 says. Whether the specification source's form is the right one to implement is lane 1's; the comments describe the code they sit beside. Checked, clean.
15. `main.wgsl:273` recomputes the view depth as `(camera.view_proj * vec4(..., 1.0)).w`, a full `mat4x4` product for one row; `culling.wgsl:131` takes the whole clip position and `froxel.wgsl:39-41` dots the rows directly. A cost, not a twin helper: falls in lane 3.
16. `fs_main` (`main.wgsl:170-307`, 137 lines) is the longest function among these parts; the probe reads no WGSL, so nothing is filed on the cognitive complexity axis, and its three sections (textures and tangent frame, the directional lights, the point lights) are each commented. Recorded for the next probe that can read WGSL.
17. `docs/audit/deviations.md` has no rows at this commit; `scripts/cycles-reference/DIFFERENCES.md`'s shader differences (ambient without visibility, ambient as E/π, specular antialiasing and Toksvig, MSAA, BC and mips) describe `ambient_luminance` and `specular_antialiased_roughness` as they are; nothing in them is read as a defect here.
18. The 29 registry keys the seven parts cite each exist in `docs/references.toml` (`grep -c '^\["<key>"\]'` is 1 for every one) and their quotes match the code beside the citation: among them BT.2087's matrix (`display.wgsl:8-12`, columns against the quote's rows), BT.2100's five PQ constants, the ICtCp rows, the sRGB knee at 0.0031308, KHR_lights_punctual's window, glTF's normal scale and occlusion clauses, Tokuyoshi's σ = 0.5 and κ = 0.18.


**B.**

1. `visible_before` (`culling.wgsl:228-230`) has one caller (`:259`) and a one-line body, the lane's canonical over-engineering form. Not filed: the name carries the bit layout the comment explains, and inlining it at `:259` would put `extractBits(instance_visibility[slot / 32u], slot % 32u, 1u) == 1u` inside a three-term boolean. Deep-module rule over line count.
2. `pixel_minimum` (`depth_pyramid.wgsl:29-34`) is called only by `pixel_minimum_within` (`:52`), and `shared_block_minimum` (`:56-61`) only at `:100`. Not filed: merging `pixel_minimum` into `pixel_minimum_within` saves 3 lines and moves the Metal unrolling comment under an edge test; `shared_block_minimum` names the step `reduce_depth`'s comment describes.
3. `OverlappedTexels` (`depth_pyramid.wgsl:10-13`) exists to return two `vec2<u32>` from `overlapped_texels`, which has one caller (`:132`). WGSL's form for two return values; not filed.
4. `froxel_slice_start` (`froxel_grid.wgsl:19-22`) sits in the shared grid part and only `froxel.wgsl` calls it (`:17-18`); `grep -rnw froxel_slice_start src`: 3 lines. Not filed: its inverse `froxel_slice` (`:25-28`), which `froxel_at` calls for three modules, sits beside it, and the pair is the grid's slicing in both directions; moving one to the pass part splits the definition for no caller's gain. The axis's "wider than needed" form is a function only its own part calls; this one another part calls.
5. `froxel_slice` is called only within its own part (`froxel_at`, `:40`): a helper of a shared function, which the axis's rule does not mean to move. Not filed.
6. `view_proj` appears in `culling.wgsl` (2 sites), `froxel.wgsl` (3) and `light_count.wgsl` (3); counted under the render section's `view_proj` row, not re-filed.
7. Single-letter identifiers found by a sweep of the six parts with comments stripped: `.r` (a swizzle, `culling.wgsl:220-221`, `depth_pyramid.wgsl:117-118`), `u` (literal suffixes), `e` (`1e-4` at `culling.wgsl:165`), `w`, `x`, `y`, `z` (components). `num_workgroups` (`culling.wgsl:403`) is the WGSL builtin's spelling. None of the lane's abbreviation list appears. Allowed.
8. `culling.wgsl:390-393`, `APPEND_WORKGROUP_SIZE = 32u`, justifies the size by a measurement ("wider workgroups wrote a dense scene's draw slower") nothing in the tree checks. Not a number in the comment's sense, and a cost reason; falls in lane 3 if re-measured.
9. `culling.wgsl:160-165`, "ADR 0014's worst camera-relative error, 0.041 mm", matches `docs/adr/0014-world-positions-are-f64-the-gpu-works-camera-relative.md:19` ("sum to 0.041 mm"); `COMPOSITION_ERROR = 1e-4` m is twice 0.041 mm rounded up to 0.1 mm as the comment says. Checked, accurate.
10. `bend.wgsl:23-27`: the series `1 - s/6 (1 - s/20 (1 - s/42))` is `1 - s/6 + s^2/120 - s^3/5040`, the terms through angle^6; the next, angle^8 / 9!, at 0.5 rad is 0.5^8 / 362,880 = 1.08e-8, under the 1.1e-8 stated. `msl-4.1-fast-math` (`docs/references.toml:54`) and `msl-4.1-sin-fast-accuracy` (`:419`, "maximum absolute error is <= 2^-13") say what the comment claims. Checked, accurate.
11. `culling.wgsl:1-7`'s draw claim (one indirect call per phase and depth pipeline, one of both phases for the lit pipeline) matches `main_pass.rs:262` (offset 0), `:296` (`PHASE_TWO_DRAW_OFFSET`), `:330` (`PHASE_TWO_DRAW_OFFSET`, into depth) and `:333` (offset 0). `:7-9` breaks "With the CULLING / Render switch on" across three short lines; layout, not an inaccuracy; not filed.
12. With the `CULLING` switch off, phase two's `offset_slots` still gathers and writes visibility bits (`culling.wgsl:268-277`) that phase one never reads (`:259`, `!CULLING || visible_before(slot)`). `CULLING` is a pipeline constant, so no source branch is dead; a cost question, falls in lane 3.
13. `FrameDraw` (`culling.wgsl:45-52`) and `LightCounts` (`common.wgsl:112-117`) are read back by the CPU (`readback.rs:399`, `:410`) through Rust twins (`culling_pass.rs:82-91`, `froxel_pass.rs:38-43`), but the twin test covers only structs the CPU writes (`shader_tests.rs:380-383`, "a struct only the GPU writes needs no Rust twin"), so their layouts are pinned by hand (`culling_pass.rs:89`, the `_padding` comment). Falls in lane 6.
14. `froxel_at`, `froxel_slice` and `froxel_slice_start` sit in the shared `FROXEL_GRID` part, which is not in the `SHARED` module the oracle harness assembles (`mod.rs:175`: `COMMON, SHADING, HDR, GT7, DISPLAY`), so none has an oracle. The convention "a function under an oracle lives in a shared part" is not broken, as none is under one; whether one should be falls in lane 6.
15. `MESHLET_MAX_TRIANGLES` is declared to WGSL at `mod.rs:240` and `grep -rn MESHLET_MAX_TRIANGLES src/render/shaders/*.wgsl` finds no part reading it. Filed as L5-S07.
16. L5-S05's two forms agree only for a `view_proj` whose x, y and w rows are mutually orthogonal, which the engine's rigid view under `perspective_infinite_reverse_rh` gives; `position_at`'s oracle pins that case. Whether either form is right for a sheared or scaled view falls in lane 1; neither is wrong for the camera the engine builds.
17. `DrawIndexedIndirect` and `DispatchIndirect` (`culling.wgsl:22-35`) mirror wgpu's `DrawIndexedIndirectArgs` and `DispatchIndirectArgs`; WGSL cannot import them, so the mirror is the only form. Not duplication of the lane's kind.
18. `offset_slots` reads `instances[slot]` twice (`culling.wgsl:248-249`, once for `.mesh` in the condition, once whole). A load the compiler may merge; a cost question at most, falls in lane 3.


**C.**

1. `sample_offset` (`shadow_mask.wgsl:186` to `192`) has one caller, `sample_surface` at line 211, and selects among `SAMPLE_OFFSETS[0]` to `[3]` with three `select`s where `SAMPLE_OFFSETS[sample]` would read as one expression; `build_shadow_mask` indexes the array by literal at lines 419 to 422. No comment says why the select chain exists; naga's backends copy a `const` array to a local for a runtime index, so the chain is likely a cost choice. That is a GPU cost question (falls in lane 3) and the rewrite could change code generation, so it is not filed here.
2. `position_at`, `octahedral_decode` and `segment_meets_box` live in `common.wgsl` and are called from `shadow_mask.wgsl` alone (`grep -lw` across the parts: `common.wgsl` and `shadow_mask.wgsl` each; 7, 5 and 2 sites with their definitions). Each is under an oracle (`shader_function_tests.rs:1618`, `:1682`, `:1865`), and a function under an oracle lives in a shared part by the module's own rule (`mod.rs:5` to `6`), so their place is right; the width of `common.wgsl` is judged in its own row above.
3. The two marker parts, 17 and 7 lines, keep their own files and `ShaderModule` entries. Each binds a group of its own (`DYNAMIC_TLAS_MARKERS`, `BLAS_REFIT_MARKERS`, both index 3, both with `marker_result`), and the dynamic TLAS's marker binds only that TLAS because wgpu-core walks every bound TLAS's dependencies per Instance at each flush (`bind_groups.rs:99` to `106`, `shader_tests.rs:40` to `63`), so folding it into the shadow mask module, which binds both TLAS in group 2, would make each marker pass pay the static TLAS's walk; the refit marker is gated to Apple with its caller (`blas_refit.rs:285`). One module with two entry points would need one group holding the TLAS and the refit scratch, and the ray-query `enable` for a part that never queries. Not filed.
4. The prologues of `trace_second_surfaces` (`shadow_mask.wgsl:486` to `495`) and `trace_shadow_edges` (`:636` to `644`) share eight lines: lane 0 loads the count, the entry index, `workgroupUniformLoad`, the early return, the entry's load and the pixel's decode. The early `return` cannot move into a helper and the decodes differ (bits 16 to 29 and bit 30 for a second surface, bits 16 and up for an edge), so a helper would serve half the block; weighed and not filed, beside L5-S09.
5. The headers of `shadow_mask_pass.rs:8` to `11` (edges only, not the turned-sample case) and `bind_groups.rs:198` to `203` (`SHADOW_MASK_BUILD`'s doc names neither `directional_verdicts`, `shadow_edges` nor `shadow_edge_dispatch`) are the render module's comments: falls in render.
6. `mod.rs:57` to `58` documents `COMMON` as "the structs more than one pass reads, the Anchor composition, the packed index and the vertex decode"; `common.wgsl:1` to `3` lists one more, "the position a depth target's depth places at a pixel", and the part also holds `segment_meets_box`, the octahedral codec, `point_light_position`, `FroxelList` and `LightCounts`. Incomplete, not wrong; a reader is not misled into wrong code. Not filed; a one-line fix if L5-S07 opens `mod.rs`.
7. `assemble` declares every constant to every module whatever it reads (`SCAN_BLOCK_SIZE` reaches the tonemapping module, `DITHER_MASK_SIDE` the culling module), and every override to every module (`SHADOW_RAYS` is read by `main.wgsl` alone, 2 sites; `CULLING` by `culling.wgsl` alone, 5 sites). One block is the design the header states, every pipeline sets every override (`shadow_mask_pass.rs:142`), and naga drops an unread `const`. Not filed.
8. `view_proj` at `shadow_mask.wgsl:212`, `216`, `261` and `419` to `422` is counted under the render section's `view_proj` row.
9. Naming weighed and allowed: `lane` for `local_invocation_index` (a full word, the SIMD lane); `hit.t` (`RayIntersection`'s own field, wgpu's API name; the comment at line 81 writes "tmin and tmax" as the textbook symbols); `own` for the pixel's own verdict word; `group` and `groups` for the workgroup builtins; `point` at line 295 for a `LightData` that is a point light, a full word that names the light's kind rather than its role (`light` is taken by the uniform); `0xffu` for the cull mask, a literal. In `mod.rs`, `x` and `y` at line 271 bind coordinates.
10. `shadow_mask.wgsl:261`, `let view_depth = camera.view_proj[3].z / depth;`, reads a view depth from the projection's fourth column, which holds for a camera-relative view with no translation (`common.wgsl:5` to `6`) and an infinite reverse-Z projection; whether every caller of `froxel_at` recovers depth the same way is a correctness reading: falls in lane 1.
11. Comment numbers recomputed: sin 2° = 0.03489949670, sin 10° = 0.17364817767, cos 3° = 0.99862953475 (`shadow_mask.wgsl:72` to `78`); 2^-16 and 2^-5 (lines 33, 48, 59); the 256 at lines 32 and 56 is 2^-16 over 2^-24. `UNBOUNDED_RAY_DISTANCE` is f32's largest finite value. The six registry keys the part cites exist in `docs/references.toml` (`wachter-binder-2019-offset-error`, `d3d11.3-rasterizer-precision`, `vulkan-subpixel-precision-bits`, `nvidia-2022-rtx-best-practices-shadow-rays`, `lauritzen-2010-per-sample-scheduling`, `lauritzen-2010-edge-detection`); whether each says what the comment claims is lane 7's reading.
12. `trace_shadow_edges:647` traces `turned_lights(own)`, every light the nearest sample turns from, where `list_shadow_edges:607` lists a pixel only for those another sample faces (`turned_lights_other_samples_face`). The comment at lines 624 to 629 describes the code as written, and a sample that also turns is hidden with no ray (`directional_hidden`), so the wider set costs no ray at the pixels listed. Not a Finding.
13. `Part::path` is read only by `locate`, under `cfg(test)`, and carries `cfg_attr(not(test), allow(dead_code))` with a doc that says what it is for (`mod.rs:41` to `43`). A field kept for error messages in tests is not vestigial; not filed.
14. `directional_hidden` (one caller, `shadow_mask.wgsl:673`) and `turned_lights_other_samples_face` (one caller, `:607`) each name one step of a dispatch in 5 and 29 lines; neither is an abbreviation of a line. Not filed; the former's duplicated body is L5-S10.
15. Nothing in the three WGSL parts reaches `tests/conventions.rs`'s six rules, which are green at this commit; no row above re-finds one.
16. The worktree was unavailable for the last greps; they ran in the main checkout, whose `src/` and `tests/` equal the frozen commit (`git diff --stat 8bc34eb1 HEAD -- src tests` is empty; the only differences are two scripts under `scripts/cycles-reference/` and `scripts/gallery.sh`, outside this module).


**D.**

1. `smith_visibility_matches_appendix_b_and_keeps_g_within_zero_to_one` (`shader_function_tests.rs:170`) asserts G ≤ 1 directly at :200 and G ≥ 0 only through the reference match at :198: `reference_visibility` is 0 or a positive reciprocal and every cosine in the cases is 0 or positive, so G lands within `RELATIVE_TOLERANCE × expected` of a non-negative value. The lower bound is pinned, indirectly; not filed.
2. `type Case = (...)` at `shader_function_tests.rs:1874`, local to the segment test, shadows the file's `struct Case` at :2054, the camera-relative test's fixture. Two things of one name in one file, the alias in scope for 150 lines. Both are full words, so the naming axis as the lane names it does not hold it; weighed as clarity, not filed.
3. `evaluate_cases`'s `vertices` parameter (`shader_function_tests.rs:2193`) is `&[]` in 8 of 9 calls and the cooked vertices in 1 (:1516). One caller exercises it, so it is not flexibility nothing exercises; the alternative, a second harness function, would duplicate 100 lines. Not filed.
4. `SRGB_TOLERANCE` (:1049), `DISPLAY_RAMP_TOLERANCE` (:1167) and `FURNACE_SUM_TOLERANCE` (:487) each have one use. A literal beside its assertion would serve, but the file's convention is a named constant carrying the measured rationale in its doc, which every tolerance in the file follows. Not filed.
5. Literals that a Rust constant could replace: BT.709's 0.2126, 0.7152, 0.0722 at :662-664 and again at :721 while `LUMINANCE_COEFFICIENTS` exists in `src/render/targets.rs`; ST 2084's fractions at :809-814 while `gt7.wgsl:36-40` holds the decimals; 0.04 at :25 and :1278 while `shading.wgsl:16` holds `DIELECTRIC_BASE_REFLECTANCE`. The header at :1-3 says the oracles are "written apart from the shader", so reading the engine's constants would defeat them. The two BT.709 sites could share one test-local constant; below a ticket's magnitude. Not filed.
6. `Case` with its 4 methods and `sums_in_every_order` (:2054-2139, 86 lines) serve one test. They hold the oracle's logic (every f32 summation order of four terms, the f32 World-position control), not an abstraction over it. Not filed.
7. `minimum_roughness` (:1465) reads `MINIMUM_ROUGHNESS` from the shader with a GPU dispatch of one invocation, called in 7 tests. `grep -rn MINIMUM_ROUGHNESS src --include='*.rs'` finds no Rust twin of the constant, so the alternative is a literal `0.04` duplicated from `shading.wgsl:22`. The dispatches' cost falls in lane 3; the design is sound. Not filed.
8. Every oracle test creates its own headless `GpuContext` (`gpu_context`, 28 callers). Whether 28 device creations per run are a cost or a flake source falls in lane 3 or lane 6; not a health matter.
9. The tolerances and the measured numbers their docs carry (`TOLERANCE`'s "within 2e-7 of theirs on the M4 Pro" at :12-13, `GT7_TOLERANCE`'s 7e-5 and 2.3e-4 at :1013-1018, `DIRECTION_TOLERANCE`'s 1.1e-5 at :1189-1193, `QUADRATURE_TOLERANCE`'s 7.1e-4 at :489-492) and whether each oracle is the right oracle fall in lane 1.
10. The comment at `shader_function_tests.rs:1806`, "Vertices up to 128 m from their pivot, the most a cooked mesh holds", restates ADR 0014's consequence ("cooked meshes stay within 128 m of their pivot"). `grep -rn 'pivot' src/cook src/assets` and `grep -rn '\b128\b' src/cook` find no bound the cooker enforces (run on the main checkout, whose `src/cook` and `src/assets` are unchanged since the frozen commit, `git diff --stat` empty). The comment matches the ADR; whether the code matches the ADR falls in lane 7.
11. Comment claims checked against the tree and found matching: `shader_tests.rs:40` "wgpu-core 30.0.1" against `Cargo.lock` (`version = "30.0.1"`); `GOLD` "as the pbr-sweep scene uses it" (:1179-1182) against `src/scenes/mod.rs:522` `SWEEP_BASE_COLOR = [1.0, 0.71, 0.29, 1.0]`; "the 2^-16 a shadow ray's origin leaves its surface by" (:1615) against `shadow_mask.wgsl:33` `SHADOW_RAY_OFFSET_SCALE = 1.52587890625e-5; // 2^-16`; "within 0.95° of it, the bound the shadow mask's margins take" (:1680) against `shadow_mask.wgsl:64`; "which the mask holds 16 times each" (:1055) against `tonemapping_pass.rs:17` `DITHER_MASK_SIDE = 64` (4096 texels over 256 levels) and its test at :185-199; `display-ramp.txt` "which the Cycles reference's port of it is checked against" (:1087-1088) against `scripts/cycles-reference/compare.py`, which reads the file; "top 26 bits ... low six" (:1548-1549) against `src/assets/cooked.rs:40` `MESHLET_MAX_VERTICES = 64` and `mod.rs:264` `ilog2()`; `gt7_curve`'s "towards 2.963, 1.19 times the peak" (:756) against `shoulder_limit` at `gt7.wgsl:55`, 2.5 × 1.18533 = 2.9633, so 1.19 is the two-decimal rounding; the `i16` impl of `ShaderScalar` (`shader_tests.rs:608-611`) against `CookedVertex::qtangent: [i16; 4]` (`src/render/bend_pass.rs:210`); `PRECISION_BAR`'s 0.043 mm (:1767-1769) against ADR 0014's precision bar and the arithmetic (tan 22.5° × 0.3 m × 2 / 1440 × 0.25 = 4.31e-5 m); the resolve doc's 0.395 and 25.04 (:684-685) against the formula at :719-730.
12. `view_proj` at `shader_tests.rs:242` and `shader_function_tests.rs:1611`, :1614, :1628, :1629, :1657 are the 6 sites the render section's `view_proj` row counted for these two files; not re-filed.
13. `point_light_luminance_attenuates_the_intensity_it_reflects`'s doc (:539-545) says the lights are "beside and above the normal" while its cases include a polar angle of 120° (:557), below the horizon. The reference handles it (N·L clamped at 0 at :601) and the doc of `reflected_luminance`'s test names that case. A loose span, not a wrong statement; not filed.
14. Vestigial sweep: every function, constant, struct field and macro in both files has a caller (Interface widths). `Part::path`'s `#[cfg_attr(not(test), allow(dead_code))]` at `shaders/mod.rs:42` exists for `shader_tests.rs`'s error messages and `every_wgsl_file_is_a_part_of_some_module`; weighed in the assembly's notes, not vestigial.
15. Functions of the shared parts with no oracle of their own (`point_light_position`, `rotate`, `decode_qtangent`, `unpack_qtangent`, `resolve_weighted` alone, `LARGEST_HALF`): `decode_vertex`'s oracle exercises the three QTangent functions and the resolve oracle both halves; `point_light_position` is a five-argument call of `camera_relative_position`. Whether any needs its own seam falls in lane 6.
16. The harness kernel (:2160-2174) declares `test_vertices` at binding 2 for every test and `evaluate_cases` binds it only when `vertices` is non-empty (:2237-2262), which rests on wgpu's auto layout dropping a binding the expression never reads. The tests are green at the frozen commit, so the reliance holds; a comment saying so would help a reader who adds a vertex-reading expression without vertices. Below a ticket; not filed.
17. The worktree at the frozen commit was unavailable after every file had been read in full and every count but one taken there. The remaining grep (Note 10) ran on the main checkout at `3368786d`, whose `src/cook`, `src/assets`, `src/render/shaders` and the two test files `git diff --stat` shows unchanged since the frozen commit.

### Counts

| Severity class | Findings |
|---|---|
| `image` | 0 |
| `budget` | 0 |
| `correctness` | 0 |
| `drift` | 7 (L5-S01, L5-S02, L5-S03, L5-S11, L5-S12, L5-S14, L5-S15); load-bearing 0 |
| `health` | 8 (L5-S04, L5-S05, L5-S06, L5-S07, L5-S08, L5-S09, L5-S10, L5-S13) |

By axis: over-engineering 5 (L5-S05, L5-S06, L5-S08, L5-S09, L5-S10);
vestigial 1 (L5-S07); cognitive complexity 0 (the probe ran clean under
`src/render/` and reads no WGSL); naming 2 (L5-S04, L5-S13); module depth
0; comment accuracy 7 (the `drift` rows), of which 3 carry a measured
figure nothing in the tree checks (L5-S02, L5-S03, L5-S12). Of the
`health` rows, 1 is in a test module only (L5-S13) and 7 in shipping
code; of the `drift` rows, 2 are in a test module (L5-S14, L5-S15). Axis
sweep: 19 file rows. Interface widths: 28 parts, types and surfaces, 152
public items counted (81 in the shading and display parts, 17 in the
geometry parts, 35 in the shadow mask and marker parts, 19 in `mod.rs`)
and 51 private items in the two test modules. Files read in full: the 19
of the module; read in full or in part for use counts, dispatch shapes
and the twins the parts rely on: the passes, bind groups, uniforms and
Instance layout of the render module, the material and cooked-vertex
types, the ADRs named above, the registry, the performance notes and this
report's render section.

## cook, assets and scenes

Scope: `src/cook/` (`mod.rs`, `fetch.rs`, `export.rs`, `mesh.rs`,
`texture.rs`), `src/assets/` (`mod.rs`, `mesh.rs`, `material.rs`,
`loader.rs`, `cooked.rs`, `pools.rs`), `src/scenes/` (`mod.rs`, `town.rs`),
`src/components.rs`, `src/transform/mod.rs`, `src/time/mod.rs`,
`src/threading.rs` and `assets/manifest.json`; 9,397 lines of Rust. The
lane's coverage floor outside `src/render` is every fifth file in path
order, a wrong sample opening its module; this section read every file in
full with its test module instead, so the sweep table is the record of
coverage. Five readers at high effort shared the module by file, each
with the lane brief verbatim: the cooker's pipeline (settings, report,
fetch and the scene exporter); the mesh and texture encoders with the
asset types, the material pool and the loader; the Cooked cache, the mesh
pool and the manifest; the Reference scenes with `town`; and the ECS
components, the transform systems, the fixed schedule and the compute
pool. Each swept its files along the six axes and recorded the sweep.
The cognitive complexity probe,
`cargo clippy --all-targets -- -W clippy::cognitive_complexity`, ran once
at the frozen commit and named two functions in this module, both tests
in the scene exporter, filed on that axis below; its third hit falls in
tooling. Every `where` is `path:line` at the frozen commit, given in full.
`deviations.md` holds no row at this commit; `DIFFERENCES.md` was read for
what is not a Finding. `tests/conventions.rs` was read for the rules
already mechanical, and none of its catches is re-filed. The ADRs whose
clauses fall here, 0011, 0017, 0018 and 0022, were read for the
comment-accuracy axis. Each Finding's evidence block says whether the fix
it implies can move a pixel or change a Blob's content hash.

### Axis sweep

| File | Over-engineering | Vestigial | Cognitive complexity | Naming | Module depth | Comments |
|---|---|---|---|---|---|---|
| `src/cook/mod.rs` | clean: `CookSettings` has one field, varied by the cook seam test (`tests/cook.rs:398`); `read_manifest`'s `may_be_absent` takes both values (`mod.rs:236`, `fetch.rs:27,64`, `export.rs:387`); `Sources` has every field read; the data URI branch at `mod.rs:400` is defensive handling of a glTF form, not a dead branch | L5-C04 (`SCENE_FORMAT_VERSION` re-export at `mod.rs:17`); L5-C05 (`CookReport::unresolved_materials`); `ManifestEntry`'s five fields all read (`fetch.rs:35-114`, `mod.rs:244`) | none in this file (probe ran clean) | clean: `pbr` (`mod.rs:357,466`) is the glossary's own word (PBR factors); `info` and `uri` are the `gltf` crate's own names; `guid`, `url`, `json`, `id` are domain words | clean: 18 public items, each with a caller outside the module but `SCENE_FORMAT_VERSION` (L5-C04); `texture_sources` has test callers only, a seam (Notes A.4) | clean: `is_fresh`'s "its Blob present" (`mod.rs:331`) matches `CookedCache::entry` (`cooked.rs:706` tests `blob_path(blob).exists()`); the thread comment at `mod.rs:169-171` matches `chunks(threads)` and `div_ceil`; "read twice" at `mod.rs:456` matches the two `texture_source` calls |
| `src/cook/fetch.rs` | clean: `Fetch`'s two variants are both constructed (`fetch.rs:103,127`) and both matched (`src/bin/praxis.rs:396-397`); `fetch_entries`'s `download` parameter is the test seam three tests use; `fetch_for_launch_with` has one shipping caller and three test callers | clean: `directory_of`, `sha256_hex`, `download` each have shipping callers | none in this file (probe ran clean) | clean: `partial`, `actual`, `fetches`, `scene_files`; the test helper's `test: &str` parameter names its role | clean: 4 public items, `fetch` and `absent_fetched` each with one shipping caller, `fetch_for_launch` `pub(super)` with one | clean: the curl comment at `fetch.rs:132-135` matches the arguments (`--connect-timeout 30`, `--speed-limit 1000`, `--speed-time 60`); the "written beside its place and renamed" doc at `fetch.rs:91-93` matches `fetch.rs:120-126`; `docs/research/test-assets.md` exists |
| `src/cook/export.rs` | L5-C03 (a third QTangent decoder); `MeshSource` and `TextureFile` variants all constructed (`export.rs:381,452` and `368,424,426`), `Embedded` reached by NormalTangentTest's glb (`assets/manifest.json:69`, exported by `every_reference_scene_but_crowd_exports_whole`); `AssetRecorder` is the trait's second implementor | clean: `name_text`, `file_name`, `channel_name`, `Exported::write` each called; `check_references`, `scratch`, `read_scene`, `floats`, `assert_near` each used by two or more tests | L5-C01 (`export.rs:773`, 33 over 25), L5-C02 (`export.rs:1001`, 39 over 25) | clean: `bin` (`export.rs:657`) is glTF's own BIN chunk and the `Glb` struct's field it fills (Notes A.6); `glb` is the format's name; `uvs`, `suns`, `ev100` allowed; `[x, y, z, sign]` at `export.rs:1133` are coordinates | clean: 2 public items; `export_scene` has one shipping caller (`src/bin/praxis.rs:795`) | clean: "As prepare" at `export.rs:96,137` matches `src/render/prepare.rs:535,370`; `tangent_frame`'s doc (`export.rs:616-618`) matches `common.wgsl:252-258,275-278`; `SCENE_FORMAT_VERSION` 1 matches `scripts/cycles-reference/render.py:27,377`; the glTF constants at `export.rs:633-636` are the specification's values; the three `khr-lights-punctual-*` and `gltf-2.0-tangent-bitangent` keys exist in `docs/references.toml:133,139,707` and say what the comments claim |
| `src/cook/mesh.rs` | clean (`SourcePrimitive` is used in 4 files; `TangentGeometry` is the adapter `bevy_mikktspace::Geometry` requires; `Level`-style helpers all have 2 or more callers) | clean (every constant and function has a caller; `tests::sample_rotations` is used by `src/render/shader_function_tests.rs:1480`) | none (probe ran clean) | clean (`u`, `v` at lines 205, 261 and 576 are UV components weighed as coordinates; `T` at line 538 is a generic; `remap` is meshopt's API name) | L5-C07, L5-C08 | clean (meshopt 0.6.2 vendors meshoptimizer 0.25, `bevy_mikktspace` 1.0.0 and overdraw 1.05 checked against `Cargo.lock` and the vendored header; the scan-builder comment weighed, see Notes B.8) |
| `src/cook/texture.rs` | L5-C06 | clean (`SourceImage`'s two variants are both constructed, at `src/cook/mod.rs:277` and `:417`; `TextureSource: Clone` is exercised by `src/cook/mod.rs:294`) | none (probe ran clean) | clean (`x`, `y` are texel coordinates; `data` at line 507 is a word; `bc7_settings` names the API's type) | clean (`decode` is `pub` for `tests/cook.rs:318`; the `pub` fields `id`, `channel`, `name` are read there and in `src/cook/mod.rs`) | clean (`intel_tex_2 0.5.0` matches `Cargo.lock`; "PNG or JPEG" at line 46 holds by feature unification, see Notes B.6) |
| `src/assets/mesh.rs` | clean | clean (all four types and every field have readers: `needs_material_handle` at `src/render/prepare.rs:367`, `vertices` at `prepare.rs:598-610`, `bounds` at `src/render/acceleration_structures.rs:333`, the two GPU structs in `src/render/shader_tests.rs:276-284` and their WGSL twins) | none (probe ran clean) | clean | L5-C12 | clean (`Mesh`'s "as prepare needs it" is narrow, `bounds` is read by the acceleration structures too; not load-bearing, Notes B.10) |
| `src/assets/material.rs` | clean (`insert`, `MaterialData` and `DEFAULT_LAYER` are `pub` for `src/render/renderer.rs`'s tests, Notes B.3) | clean (`materials: Vec<MaterialData>` is read by shipping code for its length only, Notes B.4) | none (probe ran clean) | clean (no abbreviation in an identifier; `SSBO`, `MR` and `AO` appear in comments and one message only) | L5-C10 | L5-C13 (13,107, `material_ref.0.index()`, the 126 µs and `MATERIALS`' binding names checked against their sources) |
| `src/assets/loader.rs` | clean (`AssetLoader` has two implementors, `PoolLoader` at line 45 and `AssetRecorder` at `src/cook/export.rs:294`, and 20 `dyn AssetLoader` sites) | clean (all six trait methods are called from `src/scenes/mod.rs`, `src/scenes/town.rs` or `src/state.rs`) | none (probe ran clean) | clean | L5-C09, L5-C11 | clean (each method's doc matches its `MeshUse` variant's at `src/assets/pools.rs:19-27`) |
| `src/assets/mod.rs` | clean | clean (the `Mesh` and `MeshUse` re-exports are used only inside `assets`, Notes B.11) | none (probe ran clean) | clean | clean (`assets` is a private module of `src/lib.rs:18`, so `pub` and `pub(crate)` reach the same code) | clean (no comments) |
| `src/assets/cooked.rs` | clean (every variant of `AssetKind`, `AssetSource` and `AssetName` is constructed outside the file; `CookedCache::new`'s `impl Into<PathBuf>` takes `&str`, `&PathBuf` and `PathBuf` across its 22 callers; `CookedMeshlet::split`'s `impl IntoIterator` takes arrays and iterators across its 4 callers; the three `to_blob`/`from_blob` pairs share `payload` where their framing agrees and differ where KTX2 and meshopt differ) | clean (all 12 `pub(crate)` constants have at least one reader outside the file; `pub use super::MaterialChannel` is imported through `crate::cooked` by `src/cook/mod.rs:24` and `src/cook/texture.rs:9`) | none in this section (probe ran clean) | L5-C16 (the Channel count as a literal `5`); otherwise clean (`id`, `uv`, `hex`, `fmt`, `dfd`, `kvd`, `sgd` weighed below) | clean (`pub` items are crate-external through `pub use assets::cooked` at `src/lib.rs:38`, and `tests/cook.rs` calls `read`, `material`, `texture`, `mesh`, `write` and `new`) | clean (the layout, order and size claims at `cooked.rs:38-47`, `80-82`, `147-149`, `391`, `503-509`, `829`, `874-877` each checked against the code or the ADR) |
| `src/assets/pools.rs` | L5-C15 (`allocate_all<const N>` with one instantiation); `load` beside `load_for` weighed and not filed | L5-C14 (a test re-implements `gpu_context`); `MESH_CAPACITY` is `pub` with no reader outside the file, weighed below | none in this section (probe ran clean) | clean (`N` and `T` are type parameters; `x`, `y` are coordinates; `blas`, `tlas`, `u32_bytes`, `texels` weighed below) | clean but for `MESH_CAPACITY` (17 methods, each with a caller outside the file; the six test helpers each used by two or more test modules) | L5-C17 ("a BLAS per Mesh"); the figures at `pools.rs:596`, `733-734`, `982-983` recomputed and found right; `wgpu-core 30.0.1` at `pools.rs:388` matches `Cargo.lock` |
| `assets/manifest.json` | clean (`dev_only` false on every entry is policy the manifest test enforces, weighed below) | clean (SciFiHelmet's six files load in `src/state.rs`, NormalTangentTest in `src/scenes/mod.rs:580`, the blue-noise mask is compiled into `src/render/tonemapping_pass.rs` and pinned to the file by its test) | none in this section (probe ran clean) | clean | clean | clean |
| `src/scenes/mod.rs` | L5-C18, L5-C19 (two `match` expressions whose eleven arms are one or-pattern returning one value; the other eight per-scene matches each have two or more distinct arms) | clean (every private constant is read, which the compiler's dead-code lint also guarantees; the three `pub` items the crate exports, `ALL`, `name` and `from_name`, each have callers in `src/bin/praxis.rs`; `Geometry` derives `Clone` that nothing calls, Notes D.1) | none in this section (probe ran clean) | clean (the single letters are `u`/`v` as the components of a UV, `x`/`y`/`z` as coordinates, `i` nowhere; `tau` is the constant's own name; Notes D.2) | L5-C20, L5-C21, L5-C22 (the spawn surface is eleven free functions plus four helpers exported for one caller; `ScenePrimitive` is converted to `SourcePrimitive` the same way at both consumers; four spawns repeat one body) | L5-C23 (one camera angle a moved target left stale); every other figure re-derived from the code beside it and found right, Notes D.7 |
| `src/scenes/town.rs` | clean (`TownRenderable::new` and `cube` are both used at many sites; `Choice` has every variant constructed; `Block`'s and `Flock`'s fields are all read; `TownMesh`/`BoxMaterial` share a shape, Notes D.4) | clean (all 53 private constants are read; the eighteen `MATERIALS` rows are each named by a Mesh or a `BoxMaterial`) | none in this section (probe ran clean) | clean (`min`/`max` in `cuboid` are glam's own spelling; `x`/`z` are wheel coordinates; Notes D.2) | clean (four `pub(crate)` items, `spawn_town`, `camera_at`, `materials`, `meshes`, each with one caller in `src/scenes/mod.rs` or `src/state.rs`) | clean (every count, distance, angle and intensity in a comment re-derived: lamps 1,024, birds 10,000, "about 250,000" against the pinned 254,256, 4.5 m, 18 m, flock height clear of roofs, 4 m/s, Notes D.7) |
| `src/components.rs` | clean (every field of `Transform`, `Bend`, `PointLight` and `DirectionalLight` is read in prepare or the scenes; `Transform::IDENTITY` has 2 callers, `from_translation` 7 files, `to_affine` 2) | clean (no constant or field unread; `MaterialRef` is constructed in state, the scenes and the exporter) | none in this section (probe ran clean) | clean (`rest`, `azimuth`, `curvature`, `casts_shadow`, `intensity`, `range`: full words; `x`, `y`, `z` only in comments) | clean (components hold `MeshHandle`, `MaterialHandle` and scalars, no GPU resource; the three methods are pure functions of their fields, ADR 0006) | clean (the 64-byte claim is asserted at `components.rs:17`; the spawn rule "previous curvature equal to its curvature" holds at `scenes/mod.rs:1006`) |
| `src/transform/mod.rs` | clean (the `Moved` alias has one use, 2 lines, weighed under Notes E.3; `drawn` has 7 shipping callers) | clean, with one note (`Teleported` is inserted by no shipping system, Notes E.1) | none in this section (probe ran clean) | clean (`moved`, `unparented`, `dirty`, `stack`, `composed`, `jumped`; `θ` sits in a comment only) | clean (3 public items on `WorldTransform`; `translation()` beside the public field weighed under Notes E.4; `record_previous_curvatures` acting on `Bend` weighed under Notes E.5) | clean (the shrink figure at `transform/mod.rs:19-20` recomputes: 1 − cos(0.045) = 0.00101, 0.09 rad × 64 Hz = 5.76 rad/s) |
| `src/time/mod.rs` | clean (`clamp_frame_delta` wraps one `min` for one caller, weighed under Notes E.6) | clean (`MAX_FRAME_DELTA` and `MAX_TICKS_PER_FRAME` are read only in their own file, by shipping code, Notes E.7) | none in this section (probe ran clean) | clean (`due`, `ticks`, `remainder`, `dropped`, `accumulator`; `α` in a comment only; test locals `seed`, `frame`, `steady`, `jittery`) | clean (`FixedTime` exposes 2 fields and hides `advance` and the accumulator; `fixed_schedule` and `run_ticks` have one shipping caller each, which is the orchestrator) | clean (`delta_time`'s "Injected through `State::update`" holds through `Simulation::simulate` at `state.rs:124`, called from `State::update` at `state.rs:256`) |
| `src/threading.rs` | L5-C24 | L5-C24 | none in this section (probe ran clean) | clean (`cores`, `workers`) | clean (2 functions; `single_threaded_schedule` has 2 callers) | clean (the doc at `threading.rs:24-26` matches `bevy_ecs` 0.18's default executor under the `multi_threaded` feature, ADR 0016) |

### Interface widths

| Type | Public items | Callers outside the file | Wider than needed | Where |
|---|---|---|---|---|
| `CookSettings` | 1 field (`texture_zstd_level`), `Default` | shipping: `src/state.rs:167`, `src/bin/praxis.rs:378`, both `CookSettings::default()`; `src/cook/texture.rs:119,147,563` read the field; `tests/cook.rs:30,394,398,634`, one varying the field | no: one field, the ADR's hook for settings in the Cook key; weighed in Notes A.1 | `src/cook/mod.rs:30` |
| `CookReport` | 4 fields | `cooked` and `unchanged`: `src/bin/praxis.rs:383-384`, `tests/cook.rs` 8 sites; `absent_sources`: `src/state.rs:170`; `unresolved_materials`: `tests/cook.rs:689` only | yes, one field with no shipping reader (L5-C05) | `src/cook/mod.rs:45` |
| `cook`, `cook_scene`, `texture_sources`, `ASSETS_DIRECTORY` | 4 | `cook`: `src/bin/praxis.rs:374`, `tests/cook.rs:30,395,401`; `cook_scene`: `src/state.rs:163`, `tests/cook.rs:635`; `texture_sources`: `tests/cook.rs:311,405`, no shipping caller; `ASSETS_DIRECTORY`: 5 sites in `state.rs`, `bin/praxis.rs`, `preflight/mod.rs` | no: `texture_sources` is a test seam (Notes A.4) | `src/cook/mod.rs:105-224` |
| `Fetch`, `fetch`, `absent_fetched` | enum of 2 variants, 2 functions | `fetch` and both variants: `src/bin/praxis.rs:394-397`; `absent_fetched`: `src/preflight/mod.rs:238` | no | `src/cook/fetch.rs:15-43` |
| `export_scene`, `SCENE_FORMAT_VERSION` | 2 | `export_scene`: `src/bin/praxis.rs:795`; `SCENE_FORMAT_VERSION`: none outside `export.rs` (its reader is `render.py`'s own constant) | yes, the re-export (L5-C04) | `src/cook/export.rs:31,40` |
| `LoadedImage`, `TextureSource` re-exports | 2 names (`mod.rs:19`) | `LoadedImage`: `src/scenes/mod.rs` 7 sites; `TextureSource`: named outside `src/cook/` nowhere, but it is the return type of `texture_sources`, so it must be `pub` | no | `src/cook/mod.rs:19` |
| `pub(crate) mod mesh` | 1 | `src/render/shader_function_tests.rs:1480` (`qtangent`, `tests::sample_rotations`), `src/cook/export.rs:15,514,1095` | no | `src/cook/mod.rs:7` |
| Mesh encoder (`src/cook/mesh.rs`) | 8: `SourcePrimitive`, `encoder_settings`, `cook_key`, `cook`, `weld`, `qtangent`, `gltf_meshes`, `tests::sample_rotations` | `src/cook/mod.rs` (`cook`, `cook_key` twice, `gltf_meshes`, `SourcePrimitive`), `src/cook/export.rs` (`cook` twice, `SourcePrimitive` three times), `src/render/shader_function_tests.rs` (`qtangent`, `sample_rotations`); `weld` and `encoder_settings` have none | yes: two items visible past the file with no caller past it | L5-C07, L5-C08 |
| Texture encoder (`src/cook/texture.rs`) | 11: `LoadedImage` with 3 fields; `TextureSource` with `id`, `channel`, `name`, `encoded`, `generated`, `with_normal_texture`, `cook_key`, `cook`, `decode` | `src/cook/mod.rs` (every method but `decode`), `src/scenes/mod.rs` (6 `LoadedImage` sites), `src/cook/export.rs` (1), `tests/cook.rs` (`decode`, the three fields) | one parameter: `cook_key`'s `format_version`, which the one shipping caller fixes | L5-C06 |
| `Mesh`, `MeshHandle`, `MeshData`, `MeshletData` (`src/assets/mesh.rs`) | 18: 4 types, 14 fields | `src/assets/pools.rs`, `src/render/prepare.rs`, `src/render/acceleration_structures.rs`, `src/render/bend_pass.rs`, `src/render/shader_tests.rs`, `src/render/bind_groups.rs`, `src/components.rs`; 32 `MeshHandle` mentions outside `assets` | no, but `MeshHandle` is read as `.0.0` at 5 sites while its sibling has a getter | L5-C12 |
| `MaterialPool` and its types (`src/assets/material.rs`) | 18: `MaterialPool` 9 (`new`, `gpu_memory_bytes`, `insert`, `load`, `load_texture`, `refresh_bind_group`, `DEFAULT_LAYER`, `bind_group_layout`, `bind_group`); `MaterialHandle` 2 (`pub u32`, `index`); `MaterialChannel::ALL` 1; `MaterialData` 5 fields; `TEXTURES_PER_CHANNEL` 1 | shipping: `src/render/renderer.rs:261,278,582,636,769` (`new`, `bind_group_layout`, `refresh_bind_group`, `bind_group`, `gpu_memory_bytes`), `src/assets/loader.rs:47,52` (`load`, `load_texture`), `src/render/bind_groups.rs:644` (`TEXTURES_PER_CHANNEL`), `src/cook` (`ALL` twice); `index` 14 sites. Tests only: `insert` 6, `DEFAULT_LAYER` 12, `MaterialData` 6, all in `renderer.rs` | not for shipping callers plus tests; `insert` and `DEFAULT_LAYER` reach outside the file for tests alone (Notes B.3) | none filed |
| `AssetLoader` and `PoolLoader` (`src/assets/loader.rs`) | 11: 6 trait methods, 5 `PoolLoader` fields | `src/state.rs:7,65,77,174,201`, `src/scenes/mod.rs` (13 `dyn AssetLoader` signatures, 15 method calls), `src/scenes/town.rs:756,760,764`, `src/cook/export.rs:294` (second implementor) | no | none filed |
| `assets` module root (`src/assets/mod.rs`) | 12 re-exports over 5 modules | every module but `assets` imports through `crate::assets::`, except 13 test-helper and twin imports that name `assets::pools::` or `assets::material::` directly | no (`Mesh` and `MeshUse` re-exports have no user outside `assets`, Notes B.11) | none filed |
| The 12 `pub(crate)` constants of the Cooked formats | 12 | each 1 to 7 sites: `MATERIAL_MAGIC` 1, `MATERIAL_FORMAT_VERSION` 1, `TEXTURE_VERSION_KEY` 1, `TEXTURE_FORMAT_VERSION` 2 shipping and 3 test, `MAX_TEXTURE_SIDE` 2 shipping and 1 test, `MESH_MAGIC` 1, `MESH_FORMAT_VERSION` 1, `MESHLET_MAX_VERTICES` 7 across 5 files, `MESHLET_MAX_TRIANGLES` 3, `QTANGENT_SMALLEST_W` 4 across 2 files and a WGSL part, `COOKED_CACHE_DIRECTORY` 4, `CookedMesh::ZSTD_LEVEL` 1 | no | `src/assets/cooked.rs:18-47`, `501`, `653` |
| `AssetId`, `AssetKind`, `AssetSource`, `AssetName` | 4 types, 1 constructor, 7 variants | `AssetId::new` 20 sites across 8 files; every variant constructed outside the file (`File` 11, `Scene` 6, `Name` 14, `Index` 4, `Mesh` 8, `Material` 6, `Texture` 6) | no | `src/assets/cooked.rs:52-118` |
| `CookedMaterial` | 6 fields, `to_blob`, `from_blob`, `PAYLOAD_BYTES` (private) | `to_blob` 4 shipping sites in `src/cook/mod.rs` and 3 test sites; `from_blob` only through `CookedCache::material`; fields read by `src/assets/material.rs`, `src/cook/mod.rs`, `src/cook/export.rs` | no | `src/assets/cooked.rs:135-207` |
| `CookedTexture` | 4 fields, `to_blob(zstd_level)`, `from_blob`; `level_bytes` beside it | `to_blob` 1 shipping site (`src/cook/texture.rs:215`) and 1 test site; `from_blob` only through `CookedCache::texture`; `level_bytes` 1 outside site (a `debug_assert_eq!` in `src/cook/texture.rs:206`) | no | `src/assets/cooked.rs:212-389` |
| `CookedVertex`, `CookedPrimitive`, `CookedMeshlet`, `CookedMesh` | 13 fields, `CookedMeshlet::split`, `CookedMesh::to_blob`, `CookedMesh::from_blob` | `split` 4 sites across 3 files; `to_blob` 1 shipping site (`src/cook/mesh.rs`) and 3 test sites; `from_blob` only through `CookedCache::mesh`; the types are built by the cooker, the Reference scenes, the bend pass and 6 test modules | no | `src/assets/cooked.rs:394-620` |
| `CookedCache` and `IndexEntry` | `new`, `content_hash`, `read`, `material`, `texture`, `mesh` (`pub`); `entry`, `write` (`pub(crate)`); `IndexEntry` with 2 fields; `hex` | `new` 22 sites; `content_hash` 1 (`src/state.rs:195`); `read` 2 (`tests/cook.rs:286`, `715`); `material` 9; `texture` 9; `mesh` 7; `entry` 3 (`src/cook/mod.rs:192`, `334`, `347`); `write` 3 shipping and 4 test; `hex` 4 | no | `src/assets/cooked.rs:657-760` |
| `MeshPool` | `new`, `load`, `load_for`, `rebuild_refitted_blases`, `refitted_blases`, `compact_blases`, `blas`, `vertex_arena`, `index_arena`, `mesh_table`, `meshlet_arena`, `meshlet_vertex_arena`, `meshlet_triangle_arena`, `gpu_memory_bytes`, `estimated_blas_bytes`, `arena_bytes_in_use`, `get` (17) | `new` 4; `load` 1 shipping (`src/assets/loader.rs:60`) and 9 test; `load_for` 2 (`loader.rs:76`, `87`); `rebuild_refitted_blases` 1; `refitted_blases` 2; `compact_blases` 1 shipping and 1 test; `blas` 2; `vertex_arena` 5; `index_arena` 2; `mesh_table` 1; `meshlet_arena` 1; `meshlet_vertex_arena` 1; `meshlet_triangle_arena` 1; `gpu_memory_bytes` 1; `estimated_blas_bytes` 1; `arena_bytes_in_use` 1; `get` 6 | no | `src/assets/pools.rs:82-499` |
| `MeshUse`, `MeshBlas`, `REFITTED_BLAS_FLAGS`, `ARENA_CAPACITY_BYTES`, `MESH_CAPACITY` | 4 variants, 4 fields, 3 constants | `Drawn`, `Traced`, `Refitted` 1 each in `src/assets/loader.rs`, `DrawnAndTraced` only through `load`; `MeshBlas`'s 4 fields read in `src/render/blas_refit.rs:324-335`; `REFITTED_BLAS_FLAGS` 2 in `blas_refit.rs`; `ARENA_CAPACITY_BYTES` 6 across 5 files; `MESH_CAPACITY` 0 | `MESH_CAPACITY` is `pub` with no reader outside the file (`grep -rn "MESH_CAPACITY" src tests` finds `pools.rs:12`, `123`, `188`, `190` only); `assets` is a private module, so the spelling reaches no more code than no modifier would | `src/assets/pools.rs:12` |
| `Arena` (private) | `new`, `allocate`, `free`, `write`, `longest_free_range`, 4 fields | none; `free` is reached only by `allocate_all`'s rollback at `pools.rs:516` | no | `src/assets/pools.rs:525-584` |
| The test helpers `pools::tests` | `mesh`, `triangle_about_x`, `trace_down`, `read_storage`, `gpu_context`, `logged` (6, `pub(crate)`) | `mesh` 2 modules (`src/render/prepare.rs`, `src/render/acceleration_structures.rs`); `triangle_about_x` 1 module, 1 site, plus 3 in-file tests; `trace_down` 1 module, 3 sites, plus 1 in-file; `read_storage` 3 modules, 9 sites; `gpu_context` 3 modules (`tonemapping_pass`, `depth_pyramid_pass`, `shadow_mask_pass`); `logged` 4 modules, 11 sites | no | `src/assets/pools.rs:587-1021` |
| `ReferenceScene` (enum, `pub`, re-exported from `src/lib.rs`) | 15: `ALL`, `name`, `from_name` (`pub`); `material_id`, `mesh_id`, `texture_id`, `source_files`, `materials`, `textures`, `meshes`, `camera_at`, `frame_delta`, `point_lights`, `sun`, `environment` (`pub(crate)`); `ev100` private | 11 files: `src/bin/praxis.rs`, `src/state.rs`, `src/capture/mod.rs`, `src/bench/mod.rs`, `src/cook/mod.rs`, `src/cook/export.rs`, `src/cook/mesh.rs`, `tests/cook.rs`, `tests/frame_stats.rs`, `tests/goldens.rs`, `tests/reference_scenes.rs`; every public item has at least one caller outside `src/scenes/` | No for what it has; too narrow for what it lacks: the one per-scene behaviour not behind it is the spawn (L5-C20) | `src/scenes/mod.rs:70-468` |
| The spawn surface: `spawn_pbr_sweep`, `spawn_uv_grid`, `spawn_normal_tangent`, `spawn_far_coplanar`, `spawn_orrery`, `spawn_minification`, `spawn_texture_wall`, `spawn_shadows`, `spawn_crowd`, `spawn_traced_crowd`, `spawn_town` | 11 `pub(crate)` functions of one signature | 1 site: `src/state.rs:78-91`, a `match scene` to a function pointer | Yes: eleven items for one `match` that re-does what the enum's other methods do inside the module (L5-C20) | `src/scenes/mod.rs:537-974`, `src/scenes/town.rs:756` |
| The helmet spawn's helpers: `SCI_FI_HELMET`, `SWEEP_METALLIC`, `SWEEP_ROUGHNESS`, `sweep_material_name` | 4 `pub(crate)` items | 1 site: `src/state.rs:202,214-217` (`State::spawn_helmet_scene`) | Yes: exported so that one scene's spawn can live outside the module (L5-C20) | `src/scenes/mod.rs:506,526-527,533` |
| The fixed-schedule systems: `Spin` (1 `pub(crate)` field), `spin`, `Sway` (private field), `sway`, `Bone` (private field) | 5 `pub(crate)` items | 1 site: `src/state.rs:335-336` adds `spin` and `sway` to the fixed schedule; `Spin` is constructed only inside `src/scenes/` | No | `src/scenes/mod.rs:657-670,1041-1077` |
| The geometry surface: `Geometry` (5 `pub(crate)` fields), `ScenePrimitive`, `ground_plane` | 7 `pub(crate)` items | `src/cook/mesh.rs:15,53,122,201,215,370,393,475,483,503` (`Geometry`, `ground_plane` in its tests); `ScenePrimitive` reaches `src/cook/mod.rs:304-313` and `src/cook/export.rs:371-377` only through `meshes()` | `ScenePrimitive` yes: both consumers map it to `SourcePrimitive` the same way (L5-C21) | `src/scenes/mod.rs:1125-1138,1196` |
| `town.rs`'s surface: `spawn_town`, `camera_at`, `materials`, `meshes` | 4 `pub(crate)` items | `spawn_town`: `src/state.rs:91` through the re-export at `src/scenes/mod.rs:22`; `camera_at`, `materials`, `meshes`: one call each in `src/scenes/mod.rs:229,313,344` | No | `src/scenes/town.rs:756,830,870` |
| `Transform` | 6: 3 fields, `IDENTITY`, `from_translation`, `to_affine` (`pub(crate)`) | `from_translation` in 7 files (`grep -rnF from_translation src tests`, 24 lines); `IDENTITY` in `scenes/mod.rs` and `prepare.rs`; `to_affine` at `transform/mod.rs:134` and `scenes/mod.rs:1307` (a test) | no | `src/components.rs:11-41` |
| `MeshRef`, `MaterialRef` | 1 field each | `MeshRef` in 7 files, 40 lines; `MaterialRef` in 5 files, 19 lines | no | `src/components.rs:44-47` |
| `Bend` | 5: 4 fields, `drawn_curvature` | 5 files (prepare, scenes, state, bend_pass, loader); `drawn_curvature` at `prepare.rs:606` and `state.rs:503` (a test) | no | `src/components.rs:60-77` |
| `PointLight`, `DirectionalLight` | 4 and 2 fields | 4 and 3 files; `casts_shadow` read at 13 lines, `range` at 6 | no | `src/components.rs:81-101` |
| `WorldTransform` | 3: the public `DAffine3`, `translation()`, `drawn()` | `drawn` at 7 shipping sites (`prepare.rs:383,531,555`, `export.rs:95,114,148`) and `state.rs:455` (a test); `translation()` at `prepare.rs:385,556`, `export.rs:115` and 3 test sites; `.0.matrix3` read directly at `prepare.rs:384` | marginal: a getter beside a public field (Notes E.4); removing it is 3 lines against 6 rewritten call sites | `src/transform/mod.rs:9-32` |
| `PreviousWorldTransform` | 1: the public `Option<DAffine3>` | constructed `::default()` at 5 shipping sites (`scenes/mod.rs:637,1020,1032`, `town.rs:797,805`) and 8 test sites; read by prepare and the exporter | no | `src/transform/mod.rs:41` |
| `Teleported` | 0 (a marker) | inserted at `state.rs:528` only, inside a test; read by `reset_teleported` | no shipping writer (Notes E.1) | `src/transform/mod.rs:48` |
| transform systems | 4 functions | all 4 in the fixed schedule at `state.rs:330-339`; `propagate_world_transforms` also at `state.rs:107`, `renderer.rs:1835` and `prepare.rs:694` (tests) | no | `src/transform/mod.rs:52-141` |
| `Time` | 1 field | written at `state.rs:124`, read at `camera/mod.rs:102` | no | `src/time/mod.rs:9-14` |
| time constants | 5: `MAX_FRAME_DELTA`, `TICK`, `TICK_SECONDS`, `MAX_TICKS_PER_FRAME`, `SIXTY_HERTZ` | `TICK_SECONDS` at `scenes/mod.rs:665,1068` and `state.rs:377,393` (tests); `TICK` at `state.rs:442-443` (tests); `SIXTY_HERTZ` at `bench/mod.rs:454` and `scenes/mod.rs:385`; `MAX_FRAME_DELTA` and `MAX_TICKS_PER_FRAME` none (`grep -rn MAX_FRAME_DELTA src tests`: 2 lines, both in the file) | no: two `pub` items with no outside reader sit in a private module (`lib.rs:35`), where `pub` and `pub(crate)` reach the same code | `src/time/mod.rs:18-30` |
| `clamp_frame_delta` | 1 function | `lib.rs:223` | no (Notes E.6) | `src/time/mod.rs:33` |
| `FixedTime` | 2 fields (`tick`, `blend_weight`); `advance` and `accumulator` private | `blend_weight` read at `prepare.rs:308`, `export.rs:77`; `tick` read at `scenes/mod.rs:665,1068`; both written only by `run_ticks` in shipping code, and set directly by tests at `prepare.rs:1071,1316` | no | `src/time/mod.rs:40-64` |
| `fixed_schedule`, `run_ticks` | 2 functions | `state.rs:318` and `state.rs:123`, plus state and export tests | no | `src/time/mod.rs:69-87` |
| `init_compute_pool` | 1 function, re-exported at `lib.rs:47` | `state.rs:144`; `tests/compute_pool.rs:6` | yes: a pool nothing dispatches to, and a crate-level re-export whose only caller outside the crate is the test that counts its threads (L5-C24) | `src/threading.rs:12-22` |
| `single_threaded_schedule` | 1 function (`pub(crate)`) | `state.rs:317`, `time/mod.rs:70` | no | `src/threading.rs:27-31` |

### Findings

| id | claim | severity | where | evidence | expected |
|---|---|---|---|---|---|
| L5-C01 | The test `helmet_exports_the_textured_helmet_its_sweep_and_lights` is over clippy's cognitive complexity threshold. | health; 119 lines, 33 over a threshold of 25 | `src/cook/export.rs:773` | block below | A test the probe passes: three helpers, one shared with L5-C02 |
| L5-C02 | The test `shadows_exports_its_constants_and_its_cooked_mesh` is over clippy's cognitive complexity threshold. | health; 161 lines, 39 over a threshold of 25 | `src/cook/export.rs:1001` | block below | A test the probe passes: the glb read-back and the per-vertex frame check as helpers, the sun and environment asserts shared with L5-C01 |
| L5-C03 | The QTangent is decoded by three Rust functions of the same shape, one in the exporter, when the encoder module that defines the encoding could define its decode once. | health; 3 copies, 24 lines | `src/cook/export.rs:619` | block below | `CLAUDE.md`, prefer deep modules behind small interfaces: `cook/mesh.rs` owns `qtangent` (`mesh.rs:303`) and should own its inverse, which the exporter and both tests call |
| L5-C04 | `SCENE_FORMAT_VERSION` is re-exported from the crate's public `cook` module and read by nothing outside `export.rs`. | health; 1 line, 0 callers | `src/cook/mod.rs:17` | `grep -rn SCENE_FORMAT_VERSION . --exclude-dir=target --exclude-dir=.git`: `src/cook/export.rs:31,160`, `src/cook/mod.rs:17`, `scripts/cycles-reference/render.py:27,377`; `render.py` carries its own constant, so no Rust reader exists | A private `const` in `export.rs`; `pub use export::export_scene` alone at `mod.rs:17` |
| L5-C05 | `CookReport::unresolved_materials` is filled by every cook and read by no shipping code; the warning at `mod.rs:350` already reports each entry. | health; 1 field, 0 shipping readers, 1 test reader | `src/cook/mod.rs:50` | block below | A report field has a caller that acts on it (`absent_sources` has `state.rs:170`); the smaller form is the warning alone, or the test asserting on the cache it already reads |
| L5-C06 | `TextureSource::cook_key` takes a `format_version` parameter that its two shipping callers always pass as `TEXTURE_FORMAT_VERSION`; only the file's own test varies it, while the Mesh key reads its constant inside. | health; 1 parameter, 2 shipping call sites | `src/cook/texture.rs:119` | block below | A parameter every caller passes the same value is flexibility nothing exercises; read `TEXTURE_FORMAT_VERSION` inside `cook_key` as `src/cook/mesh.rs:44` reads `MESH_FORMAT_VERSION`, and let the test pin the key's dependence on the version another way. |
| L5-C07 | `weld` is `pub(crate)` with no caller outside `src/cook/mesh.rs`. | health; 1 line, 0 outside call sites | `src/cook/mesh.rs:201` | block below | A `pub` only its own module uses is a wider interface than its callers need; make it private. |
| L5-C08 | `encoder_settings` is `pub(super)` with no caller outside `src/cook/mesh.rs`. | health; 1 line, 0 outside call sites | `src/cook/mesh.rs:29` | block below | As L5-C07: private. |
| L5-C09 | `PoolLoader::mesh` repeats `mesh_for`'s body with `MeshPool::load` in place of `load_for`, which the pool documents as the same load for a drawn and traced Mesh. | health; 6 lines | `src/assets/loader.rs:56-62` | block below | `bent_mesh` and `refitted_mesh` are one line each through `mesh_for`; `mesh` should be `self.mesh_for(id, MeshUse::DrawnAndTraced)`. |
| L5-C10 | The material pool's test hand-rolls the nine-line headless-context-or-skip match that `pools::tests::gpu_context` already provides to the crate. | health; 9 lines | `src/assets/material.rs:401-409` | block below | Tests in `src/render/` call `crate::assets::pools::tests::gpu_context()` and return on `None`; this one should too. |
| L5-C11 | The loader's test hand-rolls the same nine-line match as L5-C10. | health; 9 lines | `src/assets/loader.rs:128-136` | block below | As L5-C10. |
| L5-C12 | The two Handle newtypes are read two ways: `MaterialHandle` has both a `pub u32` field and an `index()` getter whose doc comment names `.0.0` as the worse spelling, while `MeshHandle` beside it has only the field and is read `.0.0` at five sites. | health; 5 `.0.0` sites against 15 `index()` sites | `src/assets/mesh.rs:15` | block below | One convention for sibling Handles: either both carry `index()` or neither does. |
| L5-C13 | The comment on `MaterialData::emissive` says its `w` is the emissive strength, but `load` writes a constant 1.0 there and no shader reads it. | drift; load-bearing | `src/assets/material.rs:72` | block below | A comment says what the code does; the field is padding, as `metallic_roughness`'s `w` is labelled, or strength is implemented and read. |
| L5-C14 | The test `a_mesh_past_capacity_is_refused_and_logged_and_changes_nothing` re-implements the module's own `gpu_context` helper inline in nine lines, while the module's three other GPU tests call the helper | health; 9 lines, 1 site | `src/assets/pools.rs:966-974` | block below | The test opens with `let Some(context) = gpu_context() else { return; };` as the module's three other GPU tests do (`pools.rs:623`, `693`, `727`) |
| L5-C15 | `allocate_all` is generic over a const `N` that has one instantiation, `N = 5`, at its one call site | health; 1 call site, 21 lines | `src/assets/pools.rs:503-521` | block below | A function over `[&mut Arena; 5]` and `[usize; 5]`, or a method on `MeshPool` over its `arenas()` array; `CLAUDE.md`, How agents work here: prefer deep modules behind small interfaces |
| L5-C16 | The number of Channels is the literal `5` in 12 shipping sites and 2 test sites across the crate, with no named constant; `MaterialChannel::ALL` is itself typed `[Self; 5]` | health; 14 sites across 6 files | `src/assets/cooked.rs:143` | block below | One name for the count (`MaterialChannel::ALL.len()` where a `const` context allows it, or a `MaterialChannel::COUNT`), so that adding a Channel changes one line; `CLAUDE.md`, Code conventions: identifiers are full words, and a number that recurs is an identifier without one |
| L5-C17 | `MeshPool`'s doc comment says the pool holds "a BLAS per Mesh", but `blases` is `Vec<Option<wgpu::Blas>>` and a Mesh loaded as `MeshUse::Drawn` has none | drift; not load-bearing | `src/assets/pools.rs:57-59` | block below | "a BLAS per Mesh that rays trace", as the field's own comment at `pools.rs:63-64` already says |
| L5-C18 | `ReferenceScene::ev100` matches all eleven variants in one or-pattern to one expression, so the `match` and the `self` it takes decide nothing | health; 12 lines | `src/scenes/mod.rs:354-368` | block below | `CLAUDE.md`, How agents work here: no flexibility nothing exercises. A three-line function, or the expression inlined at its one caller (`camera_at`, `mod.rs:347`) |
| L5-C19 | `ReferenceScene::environment` matches all eleven variants in one or-pattern to one `Environment` value, so the `match` decides nothing | health; 12 lines | `src/scenes/mod.rs:450-467` | block below | As L5-C18: a function that returns the value, six lines |
| L5-C20 | The spawn is the one per-scene behaviour not behind `ReferenceScene`: eleven `pub(crate)` spawn functions and four helper items are exported for one `match` in `src/state.rs` that re-dispatches on the enum, and the helmet's spawn lives in that file | health; 15 exported items, 1 call site, 2 wrapper functions | `src/scenes/mod.rs:537-974` and `src/scenes/town.rs:756`; the caller at `src/state.rs:78-91` and `src/state.rs:201-243` falls in tooling | block below | `CLAUDE.md`, How agents work here: deep modules behind small interfaces. One `ReferenceScene::spawn(self, crowd_form, world, loader)` holding the `match`, the spawn functions private, the four helpers private, `spawn_crowd` and `spawn_traced_crowd` deleted |
| L5-C21 | `ScenePrimitive` names a Primitive's Material by `&'static str` where `SourcePrimitive` names it by `AssetId`, and both consumers of `meshes()` convert one to the other with the same six-line `map` | health; 2 call sites, 14 lines | `src/scenes/mod.rs:1125`; the two conversions at `src/cook/mod.rs:304-313` and `src/cook/export.rs:371-377` fall in the cooker's pipeline | block below | One shape: `meshes()` resolves `scene.material_id(name)` itself, since it has `self`, and returns `Vec<(&'static str, Vec<SourcePrimitive>)>`; `ScenePrimitive` goes |
| L5-C22 | Four spawn functions have one body: load one Mesh by its Asset ID and spawn it at `Transform::IDENTITY` | health; 4 functions, 30 lines | `src/scenes/mod.rs:566-570,576-585,705-712,873-877` | block below | One function taking the `AssetId`; with L5-C20 each becomes a one-line `match` arm |
| L5-C23 | The crowd camera's doc comment says it looks down "about 28°" at the crowd's middle; the camera and target it sits beside look down 26.9°, and 28° is the angle to a target at ground level, before the target rose to 0.6 m | drift; not load-bearing | `src/scenes/mod.rs:921-924` | block below | The comment's figure matches the constants beside it: "about 27°", or the figure dropped |
| L5-C24 | `init_compute_pool` spawns a process-global pool of `cores − 1` worker threads at every `State::new`, and no code in the crate dispatches work to it. | health; 31 lines, 1 test file, 1 dependency, 0 users | `src/threading.rs:12-22` | block below | ADR 0016's own rule, "builds nothing before measurement asks for it": create the pool where the first system or scoped task uses it, with the same `cores − 1` count the ADR fixes; until then the orchestrator calls nothing and `tests/compute_pool.rs` goes with it |

#### L5-C01

The probe's line, from `cargo clippy --all-targets -- -W clippy::cognitive_complexity` at the frozen commit:

```
warning: the function has a cognitive complexity of (33/25)
   --> src/cook/export.rs:773:8
773 |     fn helmet_exports_the_textured_helmet_its_sweep_and_lights() {
```

The test body (`export.rs:773-891`) runs four groups of assertions in one function: the renderable count and the textured helmet (`787-799`), the ten sweep copies in a nested `for` over rows and columns with four asserts each (`800-827`), the helmet Mesh and its four Texture files (`829-851`), and the point lights, sun and environment (`853-888`). The nested loops and the `for` over the light table carry most of the branching.

The split: `assert_sweep_copies(&scene, renderables)` for `800-827`; `assert_helmet_textures(&scene)` for `829-851`; `assert_sun_and_environment(&scene)` for `870-888`, whose nineteen lines are repeated verbatim in `shadows_exports_its_constants_and_its_cooked_mesh` at `export.rs:1035-1054` (the same `dvec3(0.5, -1.0, -0.3)` direction, `[1.0; 3]` colour, `100_000.0` illuminance, `[10_000.0; 3]` ambient and `[10_000.0, 20_000.0, 30_000.0]` background, with the same tolerances), so one helper serves both tests.

The fix moves no pixel and changes no Blob: the test writes `scene.json` to a scratch directory under `std::env::temp_dir()` (`export.rs:734-739`) and reads it back; it touches no Golden, no Cooked cache and no cooked format.

#### L5-C02

The probe's line:

```
warning: the function has a cognitive complexity of (39/25)
    --> src/cook/export.rs:1001:8
1001 |     fn shadows_exports_its_constants_and_its_cooked_mesh() {
```

The test body (`export.rs:1001-1161`) runs five groups: the camera (`1006-1033`), the environment and lights (`1035-1061`, the sun and environment part shared with L5-C01), the Renderable (`1063-1069`), the two Materials in a `for` (`1071-1087`), and the glb read-back (`1089-1159`): a `for` over Primitives, inside it five reader collections and two equality asserts, then a `for` over every index with an `if index == 0` for the expected normal, four asserts and a second `if index == 0` for the ground's tangent. The two nested loops with their conditionals are the bulk of the 39.

The split: `assert_sun_and_environment(&scene)` (L5-C01's helper) for `1035-1054`; `assert_primitive_matches_cooked(&reader, &cooked, cooked_primitive)` for `1106-1128`, which pins indices, positions and UVs against the cooked Mesh; `assert_tangent_frame(position, normal, tangent, sign, is_ground)` for `1131-1157`, which checks one vertex's decoded frame against the analytic sphere and plane within `4.0 * STEP`. The outer `for` then holds one call per Primitive and one per vertex.

The fix moves no pixel and changes no Blob: the test exports to a scratch directory and reads the glb back through the `gltf` crate; `cook::mesh::cook` is called, not changed, and nothing writes to the Cooked cache.

#### L5-C03

The three decoders:

`src/cook/export.rs:619-626`, shipping, f32, the frame the exporter writes into each glb:

```rust
fn tangent_frame(qtangent: [i16; 4]) -> ([f32; 3], [f32; 4]) {
    let components = qtangent.map(|component| (f32::from(component) / i16::MAX as f32).max(-1.0));
    let rotation = glam::Quat::from_array(components).normalize();
    let bitangent_sign = if rotation.w < 0.0 { -1.0 } else { 1.0 };
    let normal = rotation * glam::Vec3::Z;
    let tangent = rotation * glam::Vec3::X;
```

`src/cook/mesh.rs:423-428`, under `#[cfg(test)]`, f32, returning `rotation.w.signum()` for the sign:

```rust
    fn decode(qtangent: [i16; 4]) -> (Vec3, Vec3, f32) {
        let rotation =
            Quat::from_array(qtangent.map(|component| (component as f32 * STEP).max(-1.0)))
                .normalize();
        (rotation * Vec3::Z, rotation * Vec3::X, rotation.w.signum())
```

`src/render/bend_pass.rs:210-216`, under `#[cfg(test)]` (the module at `bend_pass.rs:122`), f64:

```rust
    fn tangent_frame(qtangent: [i16; 4]) -> (glam::DVec3, glam::DVec3, f64) {
        let components =
            qtangent.map(|component| (f64::from(component) / f64::from(i16::MAX)).max(-1.0));
        let rotation = glam::DQuat::from_array(components).normalize();
        let sign = if rotation.w < 0.0 { -1.0 } else { 1.0 };
        (rotation * glam::DVec3::Z, rotation * glam::DVec3::X, sign)
```

`grep -rn "fn tangent_frame\|fn decode(qtangent" src`: 3 definitions. `grep -rn "fn qtangent" src`: 1, at `src/cook/mesh.rs:303`, `pub(crate)`, which `src/render/shader_function_tests.rs:1480` already imports from the encoder module. Each copy documents itself as "decoded as the shader will" (`mesh.rs:421-422`), "as the main pass decodes it" (`bend_pass.rs:208-209`) or "as the shaders decode it" (`export.rs:617`): three statements of the same contract with the WGSL `decode_qtangent` at `common.wgsl:252-258`, each of which drifts alone if that contract changes.

The smaller form: one `pub(crate) fn decode_qtangent(qtangent: [i16; 4]) -> ([f32; 3], [f32; 4])` in `src/cook/mesh.rs` beside `qtangent`, with `export.rs`'s exact arithmetic, called from the exporter and from the `mesh.rs` test; the `bend_pass.rs` test converts the result to f64 (its tolerances are Snorm16 steps, `bend_pass.rs:224`, so the f32 decode followed by widening is within them, which a run of that test confirms). The two test copies fall in the encoders and in render; the Finding's site is the shipping copy.

Pixels: no Golden reads the exporter. If the shared function keeps `export.rs:620-625`'s f32 operations in order, every exported glb is byte-identical and the Cycles references in `bench/cycles/` are unchanged; no Blob is written by the exporter, so no content hash moves. A fix that instead adopted the f64 form would move the glb's tangents by at most one f32 rounding and would need the Cycles Baselines re-judged.

#### L5-C04

`src/cook/mod.rs:17`:

```rust
pub use export::{SCENE_FORMAT_VERSION, export_scene};
```

`grep -rn SCENE_FORMAT_VERSION . --exclude-dir=target --exclude-dir=.git` finds the constant at `src/cook/export.rs:31` (its definition) and `export.rs:160` (its one read, into the scene file), the re-export at `src/cook/mod.rs:17`, and `scripts/cycles-reference/render.py:27,377`, which carries its own copy of the number and compares the file's field against it. No Rust file outside `export.rs` names it, so the re-export has no reader. The smaller form drops `SCENE_FORMAT_VERSION` from the `pub use` and leaves `export_scene`. The fix moves no pixel and changes no Blob: the exporter writes the same number and the cooker writes nothing different.

#### L5-C05

The field and its filling:

```rust
    /// Each Mesh and the Material one of its Primitives names that is not
    /// in the Cooked cache.
    pub unresolved_materials: Vec<(AssetId, AssetId)>,
```

(`src/cook/mod.rs:48-50`), filled at `mod.rs:197-207` by collecting every `(Mesh, Material)` reference across `sources.meshes` and at `mod.rs:338-353` by `unresolved_materials`, which filters those whose Material the cache lacks and warns for each (`mod.rs:350`, "a Primitive names a Material the Cooked cache lacks").

Readers: `grep -rn "unresolved_materials" src tests`: `src/cook/mod.rs:50,207,340,504` (the definition, the fill, the function and its unit test) and `tests/cook.rs:689` (`assert_eq!(report.unresolved_materials, [])` in `every_reference_scene_mesh_cooks_and_reads_back`). The binary prints `cooked` and `unchanged` only (`src/bin/praxis.rs:380-385`); the dev launch reads `absent_sources` only (`src/state.rs:163-170`). Of the report's four fields, this is the one with no shipping reader; the Material-samples-a-missing-Texture case beside it (`mod.rs:190-196`) is a warning with no report field, so the two "the cache lacks it" cases are already shaped differently.

The smaller form: the function warns and returns nothing, the collection at `197-207` folds into it, and the field goes (about 12 lines); the seam test at `tests/cook.rs:689` then asserts what it already can, that each Mesh's Materials read back from the cache (`tests/cook.rs:691-700` reads them by ID). Or the field stays, as the one way a test observes the warning without capturing `tracing`; the verifier weighs which. No pixel or Blob moves either way: the report shapes no cooked bytes.

#### L5-C06

`src/cook/texture.rs:119`:

```rust
pub(super) fn cook_key(&self, settings: &CookSettings, format_version: u32) -> String {
    let mut hasher = Sha256::new();
    hasher.update(TEXTURE_VERSION_KEY);
    hasher.update(format_version.to_le_bytes());
```

Callers: `grep -rn "cook_key(" src tests` finds two shipping calls, `src/cook/mod.rs:162`: `let cook_key = texture.cook_key(settings, TEXTURE_FORMAT_VERSION);`, and the normal Texture's recursive call at `src/cook/texture.rs:130`, which forwards the same `format_version`, and the two in the file's own test, `src/cook/texture.rs:565-566`, which pass `TEXTURE_FORMAT_VERSION` and `TEXTURE_FORMAT_VERSION + 1` to assert the keys differ. `grep -rn "TEXTURE_FORMAT_VERSION" src tests` shows no other value is ever passed. The Mesh encoder does the same job without the parameter: `src/cook/mesh.rs:43-44` hashes `MESH_MAGIC` and `MESH_FORMAT_VERSION.to_le_bytes()` directly, and its own key test (`src/cook/mesh.rs:391`) does not vary the version. The parameter exists for one test. Dropping it hashes the same bytes, so every Texture's Cook key stays the same, nothing re-cooks and no pixel moves. The test would then pin the version's part of the key by comparing against a key computed without the version's bytes, or go, since `the_cook_key_covers_every_part_a_mesh_cooks_from` already sets the precedent of not pinning the version.

#### L5-C07

`src/cook/mesh.rs:201`: `pub(crate) fn weld(geometry: &Geometry) -> Geometry {`.

`grep -rnw "weld" src tests | grep -v "^src/cook/mesh.rs"` finds one hit, a prose mention in a comment at `tests/frame_stats.rs:153` ("which the cook's weld drops"). Inside the file it has three callers: `cook_primitive` at line 123 and the tests at lines 494 and 701. `qtangent` beside it is `pub(crate)` for a real outside caller (`src/render/shader_function_tests.rs:1503`), and `cook` for `src/cook/export.rs:514,1095`; `weld` has none. A visibility change moves no pixel and changes no Blob.

#### L5-C08

`src/cook/mesh.rs:29`: `pub(super) fn encoder_settings() -> String {`.

`grep -rn "encoder_settings(" src tests | grep -v "^src/cook/mesh.rs"` finds nothing. Its one caller is `cook_key` at `src/cook/mesh.rs:49`. The string it returns is part of every Mesh Cook key, so the function must stay; only its visibility is wider than its callers. No pixel, no Blob changes.

#### L5-C09

`src/assets/loader.rs:56-62`:

```rust
fn mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
    let mesh = self.cache.mesh(id)?;
    let materials = self.materials(id, &mesh)?;
    self.mesh_pool
        .load(self.staging, &mesh, &materials)
        .with_context(|| format!("the Mesh pool refused Mesh {id}"))
}
```

and `src/assets/loader.rs:83-89`:

```rust
fn mesh_for(&mut self, id: AssetId, mesh_use: MeshUse) -> anyhow::Result<MeshHandle> {
    let mesh = self.cache.mesh(id)?;
    let materials = self.materials(id, &mesh)?;
    self.mesh_pool
        .load_for(self.staging, &mesh, &materials, mesh_use)
        .with_context(|| format!("the Mesh pool refused Mesh {id}"))
}
```

The pool documents `load` at `src/assets/pools.rs:130` as "Loads `mesh` to be drawn and traced, as `load_for` loads it", and `MeshUse::DrawnAndTraced` is the variant at `src/assets/pools.rs:18`. `bent_mesh` (line 64) and `refitted_mesh` (line 68) already go through `mesh_for`; `mesh` is the one sibling that does not. The same pool call runs either way, so Goldens, counters and the Mesh table are unchanged; the fix moves no pixel and changes no Blob. Whether `MeshPool::load` itself should exist beside `load_for` is the Cooked cache and the Pools section's question; this Finding is the loader's copy of the shape.

#### L5-C10

`src/assets/material.rs:401-409`:

```rust
let context = match pollster::block_on(GpuContext::headless()) {
    Ok(context) => context,
    Err(error) if error.is::<wgpu::RequestAdapterError>() => {
        // Not `eprintln!`, which libtest captures from passing tests.
        writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
        return;
    }
    Err(error) => panic!("headless GPU context failed: {error:#}"),
};
```

`src/assets/pools.rs:950-962` defines `pub(crate) fn gpu_context() -> Option<GpuContext>` with this exact match, and `grep -rn "pools::tests::gpu_context" src` shows `src/render/tonemapping_pass.rs:180`, `src/render/shadow_mask_pass.rs:407` and `src/render/depth_pyramid_pass.rs:132` using it. The material pool's test is in the same `assets` module and already imports `crate::gpu::GpuContext` (line 395). A test refactor moves no pixel. `grep -rln "skipped: no GPU adapter" src tests` lists 18 files with the hand-rolled form; the ones outside the encoders and the asset types fall in render, tooling or the Cooked cache and the Pools section (`src/assets/pools.rs:966` itself).

#### L5-C11

`src/assets/loader.rs:128-136` is the same nine lines as L5-C10, in a test that already imports `crate::assets::pools::tests::logged` (line 122), so the helper's module is in reach. Test only; no pixel.

#### L5-C12

`src/assets/mesh.rs:15`: `pub struct MeshHandle(pub u32);` with no methods. `src/assets/material.rs:32-40`:

```rust
pub struct MaterialHandle(pub u32);

impl MaterialHandle {
    /// Index into the material SSBO. Reads better than `.0` at call sites that
    /// unwrap a wrapping newtype (e.g. `material_ref.0.index()`).
    pub fn index(self) -> u32 {
        self.0
    }
}
```

`grep -rn "\.0\.0\b" src` finds the spelling the comment calls worse at five sites, all reads of `MeshHandle` through `MeshRef`: `src/render/prepare.rs:360,395,592,600` and `src/cook/export.rs:131`. `MaterialHandle::index()` has 15 call sites (`src/render/renderer.rs` 10, `src/render/prepare.rs:393`, `src/cook/export.rs:134`, `src/assets/pools.rs:669`, `src/assets/loader.rs:201`; the `.index(, and one test site at `src/assets/material.rs:446`)` at `src/cook/mesh.rs:368` and `src/cook/export.rs:428` are gltf's API). Both newtypes keep the field `pub` because every constructor is a struct literal (`MaterialHandle(5)` at `src/render/prepare.rs:654`, `MeshHandle(...)` in the pool). The getter and the field are both public on one sibling and the field alone on the other, so a reader learns two spellings for one idea. Either spelling is a refactor of reads; no pixel moves.

#### L5-C13

`src/assets/material.rs:69-75`:

```rust
pub struct MaterialData {
    pub base_color: [f32; 4],
    pub metallic_roughness: [f32; 4], // x=metallic, y=roughness, z=normal scale, w=pad
    pub emissive: [f32; 4],           // xyz=color, w=strength
    pub layers: [u32; 4],             // x=albedo, y=normal, z=MR, w=AO
    pub extra: [u32; 4],              // x=emissive Layer, yzw=pad
}
```

The one writer is `load` at `src/assets/material.rs:284`: `emissive: [red, green, blue, 1.0],` with the 1.0 a literal, not a Material property (`CookedMaterial` has `emissive: [f32; 3]` and no strength, as the test at line 425-432 shows). `grep -rn "emissive" src/render/shaders/*.wgsl` finds every read: `src/render/shaders/shading.wgsl:88` `emissive * material.emissive.rgb` and `src/render/shaders/main.wgsl:177`, which samples the emissive Texture's `.rgb`; nothing reads `.w` or `.a`. The WGSL twin at `src/render/shaders/common.wgsl:139` declares `emissive: vec4<f32>` with no comment on `w`, so the Rust comment is the only claim that `w` is a strength. Load-bearing: a reader who sets `w` to scale an emissive Material writes code with no effect, and the glTF emissive strength extension, if it is ever cooked, would look done here. The smaller form is `w=pad` and a 0.0, which changes bytes the shader never reads, so no pixel moves; the Golden captures and counters stay.

#### L5-C14

`src/assets/pools.rs:952-962` defines the helper the whole crate's GPU tests import:

```rust
    pub(crate) fn gpu_context() -> Option<GpuContext> {
        match pollster::block_on(GpuContext::headless()) {
            Ok(context) => Some(context),
            Err(error) if error.is::<wgpu::RequestAdapterError>() => {
                // Not `eprintln!`, which libtest captures from passing tests.
                writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
                None
            }
            Err(error) => panic!("headless GPU context failed: {error:#}"),
        }
    }
```

`src/assets/pools.rs:964-974`, in the same `mod tests`, opens the next test with the same match written out:

```rust
    #[test]
    fn a_mesh_past_capacity_is_refused_and_logged_and_changes_nothing() {
        let context = match pollster::block_on(GpuContext::headless()) {
            Ok(context) => context,
            Err(error) if error.is::<wgpu::RequestAdapterError>() => {
                // Not `eprintln!`, which libtest captures from passing tests.
                writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
                return;
            }
            Err(error) => panic!("headless GPU context failed: {error:#}"),
        };
```

`grep -n "gpu_context()" src/assets/pools.rs` finds the helper called at `pools.rs:623`, `693`, `727` and `731`; this test alone does not call it. `grep -rn "pools::tests::.*gpu_context" src` finds 3 importing modules (`src/render/tonemapping_pass.rs:180`, `src/render/depth_pyramid_pass.rs:132`, `src/render/shadow_mask_pass.rs:407`). The fix replaces lines 966-974 with the `let Some(context) = gpu_context() else { return; };` form of `pools.rs:623-625`. A test's refactor moves no pixel and changes no Blob.

#### L5-C15

`src/assets/pools.rs:503-506`:

```rust
fn allocate_all<const N: usize>(
    mut arenas: [&mut Arena; N],
    counts: [usize; N],
) -> Option<[offset_allocator::Allocation; N]> {
```

`grep -rn "allocate_all" src tests` finds the definition and one call, `src/assets/pools.rs:203`, whose arrays are the pool's five Arenas and five counts (`pools.rs:180-186`, `204-211`). `MeshPool::arenas()` at `pools.rs:460` already returns `[&Arena; 5]`, so the count has a fixed home in the same file. The generic lets the compiler check the two arrays agree in length, which a `5` in both signatures does as well; the `Vec::with_capacity(N)` and `try_into().ok()` at `pools.rs:507` and `520` exist to turn a `Vec` back into an array of generic length and would go with it. Over-engineering by the module's own test: a generic with one instantiation. The fix is a signature change; it allocates the same ranges in the same order, so it moves no pixel and changes no Blob.

#### L5-C16

`src/assets/cooked.rs:143`, `149` and `193`:

```rust
    pub textures: [Option<AssetId>; 5],
    ...
    const PAYLOAD_BYTES: usize = 10 * 4 + 5 * 16;
    ...
        let mut textures = [None; 5];
```

The count is the number of `MaterialChannel` variants, which `src/assets/material.rs:56` spells `pub const ALL: [Self; 5]`. `grep -rn "; 5\]" src | grep -v "ALL"` finds the same literal standing for the Channel count at `src/assets/cooked.rs:143`, `193`, `src/assets/material.rs:144`, `267`, `366`, `src/cook/mod.rs:356`, `422`, `465`, `src/cook/export.rs:414`, `src/scenes/town.rs:917` (shipping) and `src/assets/material.rs:424`, `src/assets/loader.rs:146` (tests); with `cooked.rs:149`'s `5 * 16` and `material.rs:56`'s own `[Self; 5]`, 14 sites in 6 files. `grep -rn "CHANNEL_COUNT\|ALL.len()" src` finds `ALL.len()` once, at `material.rs:16`, as the divisor of `TEXTURES_PER_CHANNEL`; nothing names the count for the arrays. The Material Blob's layout (`cooked.rs:147-149`, "16 bytes per Channel") is sized by the same number, so a sixth Channel would need every site found by hand. The site filed is the Cooked format's, the one the others mirror; the sites in `src/assets/material.rs` and `src/cook/` fall in the encoders and the asset types and in the cooker's pipeline, `town.rs:917` in the Reference scenes. A named constant equal to 5 changes no Blob byte, no Cook key and no pixel.

#### L5-C17

`src/assets/pools.rs:57-59`:

```rust
/// The Meshes, and the GPU tables they are drawn from: Arenas of vertices
/// and indices, the Meshlet Arenas and the Mesh table (ADRs 0017, 0019);
/// and a BLAS per Mesh, which rays trace (ADR 0020).
```

beside `src/assets/pools.rs:63-65`:

```rust
    /// Each Mesh's BLAS, at its Handle's index, built at load from its
    /// ranges of the vertex and index Arenas; none for a Mesh no ray traces.
    blases: Vec<Option<wgpu::Blas>>,
```

and `pools.rs:303-306`, where a Mesh whose `MeshUse` is not traced pushes `None`. `src/assets/loader.rs:65` loads every bent Mesh as `MeshUse::Drawn`, so the `crowd` scene's figures have no BLAS. The type's comment keeps the form the pool had before `MeshUse::Drawn` existed. Not load-bearing: the field's comment four lines down corrects it before a reader writes code against it. A comment change moves no pixel.

#### L5-C18

`src/scenes/mod.rs:354-368`:

```rust
fn ev100(self) -> f32 {
    match self {
        Self::Helmet
        | Self::PbrSweep
        | Self::UvGrid
        | Self::NormalTangent
        | Self::FarCoplanar
        | Self::Orrery
        | Self::Minification
        | Self::TextureWall
        | Self::Shadows
        | Self::Crowd
        | Self::Town => ev100_for_exposure(1.0 / SUN_ILLUMINANCE),
    }
}
```

One or-pattern over every variant is a `match` with one arm: `self` is bound and never read into a decision. The doc comment above it (`mod.rs:351-353`) says the value is the same for every scene on purpose ("Each scene exposes the sun's illuminance to 1"), so the branching carries nothing a reader needs. Callers: one, `camera_at` at `mod.rs:347` (`grep -n 'ev100()' src/scenes/mod.rs`: 1 call). The smaller form is `fn ev100(self) -> f32 { ev100_for_exposure(1.0 / SUN_ILLUMINANCE) }`, or the expression at the one call site; a scene that one day differs adds its arm then. The fix moves no pixel: the value is unchanged. It changes no Cook key or camera path.

#### L5-C19

`src/scenes/mod.rs:450-467`:

```rust
pub(crate) fn environment(self) -> Environment {
    match self {
        Self::Helmet
        | Self::PbrSweep
        ...
        | Self::Town => Environment {
            ambient_illuminance: glam::Vec3::splat(0.1 * SUN_ILLUMINANCE),
            background_luminance: glam::vec3(0.1, 0.2, 0.3) * SUN_ILLUMINANCE,
        },
    }
}
```

The same shape as L5-C18: eleven variants in one arm. Its doc comment (`mod.rs:447-449`) describes one environment for every scene. Caller: one, `src/state.rs:94`. The eight other per-scene methods (`materials`, `textures`, `meshes`, `camera_at`, `frame_delta`, `point_lights`, `sun`, `source_files`) each have two or more distinct arms, so they are judged clean; only these two have none. Fix moves no pixel.

#### L5-C20

`ReferenceScene` holds every per-scene behaviour as a method on the enum (`mod.rs:86-467`: the name, the Asset IDs, the cooked content, the camera path, the frame delta, the lights, the sun, the environment), except the spawn. The spawn is eleven `pub(crate)` free functions of one signature, `fn(&mut World, &mut dyn AssetLoader) -> anyhow::Result<()>`, at `mod.rs:537,566,576,600,631,705,801,873,962,969` and `town.rs:756`, re-exported for exactly one caller, which re-does the dispatch the enum's other methods do inside the module:

`src/state.rs:78-91`:

```rust
let spawn: fn(&mut World, &mut dyn AssetLoader) -> anyhow::Result<()> = match scene {
    ReferenceScene::Helmet => State::spawn_helmet_scene,
    ReferenceScene::PbrSweep => spawn_pbr_sweep,
    ...
    ReferenceScene::Crowd => match crowd_form {
        CrowdForm::Proxies => spawn_crowd,
        CrowdForm::Refit | CrowdForm::Rebuild => spawn_traced_crowd,
    },
    ReferenceScene::Town => spawn_town,
};
```

`grep -rn 'spawn_\(pbr_sweep\|uv_grid\|normal_tangent\|far_coplanar\|orrery\|minification\|texture_wall\|shadows\|crowd\|traced_crowd\|town\)\b' src tests | grep -v '^src/scenes/'`: 14 lines, all in `src/state.rs:18-20` (the `use`) and `src/state.rs:79-91` (the `match`). No test and no other module calls a spawn function.

Because the caller holds the dispatch, the helmet scene's spawn lives in the caller, `State::spawn_helmet_scene` at `src/state.rs:201-243`, and the scenes module exports four items so it can be built there: `SCI_FI_HELMET` (`mod.rs:506`), `SWEEP_METALLIC` and `SWEEP_ROUGHNESS` (`mod.rs:526-527`) and `sweep_material_name` (`mod.rs:533`). `grep -rn 'SWEEP_METALLIC\|SWEEP_ROUGHNESS\|sweep_material_name\|SCI_FI_HELMET' src tests | grep -v '^src/scenes/'`: 6 lines, all in `src/state.rs:18,21,202,214,215,217`.

Because the dispatch needs a function pointer per arm, the crowd needs two wrappers that pass a constant to a third function: `spawn_crowd` (`mod.rs:962-964`, passes `CrowdForm::Proxies`) and `spawn_traced_crowd` (`mod.rs:969-974`, passes `CrowdForm::Refit`) around `spawn_crowd_as` (`mod.rs:978`). With the dispatch inside the module the arm is `Self::Crowd => spawn_crowd_as(world, loader, crowd_form)`, and the two wrappers go.

The smaller form: `pub(crate) fn spawn(self, crowd_form: CrowdForm, world: &mut World, loader: &mut dyn AssetLoader) -> anyhow::Result<()>` on `ReferenceScene`, holding the `match`; the eleven spawn functions become private; `spawn_helmet_scene` moves into the module beside its constants (it inserts the `HelmetAssets` resource, `src/state.rs:242`, so the move adds a dependency from `src/scenes/` on `src/helmet/`, as it already depends on `src/render/` for `CrowdForm` and on `src/camera/`); the four helpers become private. The module's interface shrinks by 15 items to 1 and `src/state.rs` loses 14 lines of `match`. The fix moves no pixel: the same functions run in the same order with the same arguments; no Cook key or camera path changes. Caller `src/state.rs:78-91` falls in tooling.

#### L5-C21

`src/scenes/mod.rs:1125`:

```rust
pub(crate) type ScenePrimitive = (Geometry, Option<&'static str>);
```

`src/cook/mesh.rs:18`:

```rust
pub(crate) type SourcePrimitive = (Geometry, Option<AssetId>);
```

Two tuple types over the same `Geometry`, differing only in whether the Material is a name or its Asset ID. `ReferenceScene::meshes` (`mod.rs:275`) returns the first; both of its consumers immediately turn it into the second with the same code:

`src/cook/mod.rs:304-313`:

```rust
for (name, primitives) in scene.meshes() {
    let id = scene.mesh_id(name);
    let primitives: Vec<_> = primitives
        .into_iter()
        .map(|(geometry, material)| {
            let material = material.map(|material| scene.material_id(material));
            (geometry, material)
        })
        .collect();
```

`src/cook/export.rs:371-377`:

```rust
for (name, primitives) in scene.meshes() {
    let primitives: Vec<SourcePrimitive> = primitives
        .into_iter()
        .map(|(geometry, material)| {
            (geometry, material.map(|name| scene.material_id(name)))
        })
        .collect();
```

`grep -rn 'ScenePrimitive' src tests`: 4 lines, the alias at `mod.rs:1125`, the return types at `mod.rs:275` and `town.rs:924`, and the `use` at `town.rs:882`; no consumer keeps a `ScenePrimitive`. `grep -rn '\.meshes()' src tests | grep 'scene\.'`: 4 call sites (`cook/mod.rs:304`, `cook/export.rs:371,1096`, `cook/mesh.rs:664`), the two above converting, the two in tests reading `Geometry` only. `meshes` has `self` and `material_id` is its own method, so it can resolve the names itself. The smaller form: `meshes` returns `Vec<(&'static str, Vec<SourcePrimitive>)>` with `self.material_id(name)` applied where each `Some("…")` is written (`mod.rs:279,284,287,299,308` and inside `town.rs:926`'s `of` closure, which would take the scene), `ScenePrimitive` goes, and the two consumers drop their `map`. The alias `SourcePrimitive` can live in either module; `src/cook/mesh.rs` already imports `scenes::Geometry` (`mesh.rs:15`). The fix moves no pixel and changes no Blob: the Asset IDs reaching `mesh::cook_key` (`mesh.rs:41`) are the same values from the same `material_id` calls. The two conversion sites fall in the cooker's pipeline.

#### L5-C22

Four spawn functions with one body, load one Mesh and spawn it at the origin:

`src/scenes/mod.rs:566-570`:

```rust
pub(crate) fn spawn_uv_grid(world: &mut World, loader: &mut dyn AssetLoader) -> anyhow::Result<()> {
    let plane = loader.mesh(ReferenceScene::UvGrid.mesh_id("plane"))?;
    world.spawn((Transform::IDENTITY, MeshRef(plane)));
    Ok(())
}
```

`src/scenes/mod.rs:576-585` (`spawn_normal_tangent`: `loader.mesh(AssetId::new(source, AssetKind::Mesh, name))?` then the same `spawn`), `src/scenes/mod.rs:705-712` (`spawn_minification`: `loader.mesh(ReferenceScene::Minification.mesh_id("ground"))?` then the same `spawn`), `src/scenes/mod.rs:873-877` (`spawn_shadows`: `loader.mesh(ReferenceScene::Shadows.mesh_id("sphere-over-plane"))?` then the same `spawn`). The ground in `spawn_crowd_as` (`mod.rs:984-985`) is a fifth site of the two lines. Their doc comments (`mod.rs:565,575,704,872`) also say the same sentence four ways ("The … draws with the Material its Primitive names"). One `fn spawn_mesh(world, loader, id: AssetId) -> anyhow::Result<()>` serves all five; under L5-C20 the four become arms `Self::UvGrid => spawn_mesh(world, loader, self.mesh_id("plane"))`. 30 lines become about 10. The fix moves no pixel: the same Mesh is loaded and spawned with the same components.

#### L5-C23

`src/scenes/mod.rs:921-924`:

```rust
/// Above the crowd and back from it, looking down about 28° at its middle,
/// so every figure stays in view and none hides another whole.
const CROWD_CAMERA: glam::DVec3 = glam::DVec3::new(0.0, 11.0, 20.0);
const CROWD_CAMERA_TARGET: glam::DVec3 = glam::DVec3::new(0.0, 0.6, -0.5);
```

From (0, 11, 20) to (0, 0.6, −0.5) the camera drops 10.4 m over 20.5 m: atan(10.4 / 20.5) = atan(0.507) = 26.9°. To a target at ground level, (0, 0, −0.5), the drop is 11 m: atan(11 / 20.5) = 28.2°, the comment's figure. So the comment was written for a target on the ground and the target's `y` rose to 0.6 (the figures' middles are 0.9 m up) without the figure following. The two other camera comments that quote an angle agree with their constants: `MINIFICATION_CAMERA` (`mod.rs:678-681`, "about 8.5°": atan(1.2 / 8) = 8.5°) and `TEXTURE_WALL_CAMERA` (`mod.rs:786-788`, "about 60°": atan(12 / 7) = 59.7°). Not load-bearing: nothing computes from the angle, and a reader changing the camera would recompute it. The expected form is a figure that matches, "about 27°", or no figure. A comment change moves no pixel.

#### L5-C24

The code, `src/threading.rs:12-22`:

```rust
pub fn init_compute_pool() -> &'static ComputeTaskPool {
    ComputeTaskPool::get_or_init(|| {
        let cores = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
        let workers = cores.saturating_sub(1).max(1);
        tracing::debug!(workers, cores, "compute pool");
        TaskPoolBuilder::new()
            .num_threads(workers)
            .thread_name("Compute Task Pool".to_string())
            .build()
    })
}
```

Called once per State at `src/state.rs:144`, inside `State::new`, so every window, capture, bench run and every integration test that builds a State starts `cores − 1` threads (11 on the M4 Pro) that then idle until the process exits.

Who uses the pool: `grep -rn "ComputeTaskPool\|par_iter\|bevy_tasks" src tests` returns 4 lines, all in `src/threading.rs`; `tests/compute_pool.rs` reaches the pool through the crate's `init_compute_pool` re-export; `grep -rn "\.scope(" src` finds only `recorder.systems.scope` (a stats timer, not a task scope) and tracing spans. No `par_iter`, no `ComputeTaskPool::get`, no scoped task anywhere. The two schedules the pool could serve both run on the single-threaded executor (`src/threading.rs:27-31`, pinned by `simulation_schedules_run_single_threaded` at `src/state.rs:358`), which never touches a task pool.

What the pool costs: the threads themselves, `cores − 1` of them per process, their stacks, and the `bevy_tasks` dependency (`Cargo.toml:18`) that exists for this one function. `bevy_ecs`'s `multi_threaded` feature (`Cargo.toml:17`) is on for `par_iter`, which nothing calls either; the Cargo side falls in tooling.

What reads it: `tests/compute_pool.rs:4-8` asserts `pool.thread_num() == (cores - 1).max(1)`, the one observable property, and the re-export `pub use threading::init_compute_pool` at `src/lib.rs:47` exists so that test can call it. ADR 0016 records the state as built: "the `ComputeTaskPool` is created with `available_parallelism() − 1` workers (at least 1) and the schedule stays `SingleThreaded`; no system uses the pool yet, and the rest is planned". The ADR's reason for the explicit creation is to avoid `TaskPool::default`'s `cores` threads beside the main thread; that reason applies at the moment a pool is first needed, not before.

The smaller form: delete `init_compute_pool`, its call at `state.rs:144`, the re-export at `lib.rs:47` and `tests/compute_pool.rs`, and make `single_threaded_schedule` the file's whole content (or fold it into `time/mod.rs`, its other caller being `state.rs`); when the first `par_iter` or scoped task lands, create the pool beside it with the ADR's count. Alternatively, place it as fog with the trigger "the first system that dispatches to the compute pool", which ADR 0016 itself names as the next step. Either way the fix moves no pixel and changes no Blob: nothing in the frame reads the pool, so Goldens stay bit-exact and counters equal; CPU frame timings stay within noise, since the idle threads do no work in the frame.

### Notes for the verifier

Not filed, since each is a near-miss under the lane's axes, an allowed form, or belongs to another lane; recorded so the next audit does not re-find them. By reader: A the cooker's pipeline, B the encoders and the asset types, C the Cooked cache and the Pools, D the Reference scenes, E the ECS and the schedule.

**A.**

1. `CookSettings` (`src/cook/mod.rs:30`) was weighed as "an option every caller sets the same way" and not filed: both shipping callers pass `CookSettings::default()` (`src/state.rs:167`, `src/bin/praxis.rs:378`), and the only variation is `tests/cook.rs:398-399`, which raises `texture_zstd_level` by one to prove the Cook key re-cooks every Texture and no Material. The struct is one field with a `Default`, ADR 0018's "every cook setting" is hashed through it at `texture.rs:124`, and the level is used at `texture.rs:215`; folding it into a constant would save about 15 lines and remove a parameter from `cook` and `cook_scene` at 7 sites while losing the seam test's lever. Judged minimal already; the next audit need not re-weigh it unless a second field appears that nothing varies.
2. The probe's line for `helmet_exports_the_textured_helmet_its_sweep_and_lights` reads `(33/25)`, and the Finding quotes the probe.
3. `AssetRecorder` (`src/cook/export.rs:289-321`) is the second implementor of `AssetLoader` beside `PoolLoader` (`grep -rn "impl AssetLoader" src`: 2), so the trait is not a trait with one implementor; recorded for the encoders.
4. `texture_sources` (`src/cook/mod.rs:219`) and `TextureSource::decode` (`src/cook/texture.rs:96`, among the encoders) have no shipping caller outside the module: `tests/cook.rs:311,318,405` are their callers. Both are the cook seam's observation points (`docs/agents/workflow.md`, "Tests live at pre-agreed seams"), so neither is filed as vestigial.
5. `Exported::write` (`src/cook/export.rs:597`) has one caller, the glb at `export.rs:515`, while the PNG write at `export.rs:561-571` creates its directory and writes through `image::save_buffer_with_format` itself. Two write shapes for two formats; not filed, as the PNG encoder needs the `image` call and `write` would only replace the `create_dir_all` line.
6. Abbreviations weighed and allowed: `bin` (`export.rs:657,666,704,719`) names glTF's BIN chunk and fills `gltf::binary::Glb`'s `bin` field, a crate's own field name; `glb` and `uri` are the format's own words; `pbr` (`mod.rs:357,466`) is `CONTEXT.md`'s "PBR factors"; `info` (`mod.rs:359,361,365`) is the `gltf` crate's `texture::Info` and a word in its own right; `uvs`, `suns`, `ev100`, `guid`, `sha256`, `url`, `json`.
7. The data URI branch at `src/cook/mod.rs:400-403` is reached by no manifest source at the frozen commit (`assets/manifest.json` lists one `.gltf` with external images and one `.glb` with buffer-view images); it is the cooker's handling of a glTF form the specification allows, so it is defensive, not dead, and not filed.
8. `CookReport`'s doc comment (`src/cook/mod.rs:43`, "The Asset IDs one cook wrote, and those it found up to date") names two of its four fields; the other two carry their own docs at `mod.rs:48-51`. Not filed as drift: it says nothing the code does not do.
9. Lane 1 (correctness) owns, if anyone does, whether `read_sources`' Toksvig dependency at `mod.rs:283-297` (the first Material's normal Texture when several sample the same metallic-roughness image) matches ADR 0018's "the first Material's" clause; the code matches the ADR as read, so nothing is passed on.
10. The test module of `src/cook/mod.rs` pins two things the shipping code does (`unresolved_materials` on an empty cache; every scene's `source_files` in the manifest); `fetch.rs`'s three tests pin `fetch_entries` and `fetch_for_launch_with` through the `download` seam; `export.rs`'s five tests pin the exporter against the scenes and the cooker. None pins nothing.

**B.**

1. `AssetLoader` (`src/assets/loader.rs:9`) is not a trait with one implementor: `PoolLoader` at `src/assets/loader.rs:45` and `AssetRecorder` at `src/cook/export.rs:294` implement it, and `grep -rnw "AssetLoader" src` finds 20 `&mut dyn AssetLoader` sites across `src/state.rs`, `src/scenes/mod.rs` and `src/scenes/town.rs`. Not filed.
2. `SourcePrimitive` (`src/cook/mesh.rs:18`) is used in `src/cook/mod.rs:99` and `src/cook/export.rs:15,326,372` beside its own file; a type alias with four files of readers is not an abstraction with one use. Not filed. Whether it and `src/scenes/mod.rs`'s `ScenePrimitive` are two names for one shape is the Reference scenes section's question.
3. `MaterialPool::insert` (`src/assets/material.rs:240`), `MaterialPool::DEFAULT_LAYER` (line 160) and `MaterialData` (line 69) have shipping callers only inside `src/assets/material.rs` (`load` at line 289 and 267); every outside caller (`insert` 6, `DEFAULT_LAYER` 12, `MaterialData` 6) is in `src/render/renderer.rs`'s test module, which builds a white Material without a Cooked cache. The width serves tests, so not filed; a verifier who counts tests as callers finds it exact, one who does not finds three `pub` items for tests alone.
4. `MaterialPool.materials: Vec<MaterialData>` (`src/assets/material.rs:142`) is read by shipping code for `.len()` only (lines 241, 248); the test at lines 446 and 452 reads entries. A count would serve shipping code, and the Vec holds at most 256 entries of 80 bytes. Not filed as vestigial because the test reads it; near miss.
5. `MaterialPool::gpu_memory_bytes` is called from `src/render/renderer.rs:769`; every method of the pool has a shipping caller.
6. `src/cook/texture.rs:46` says an encoded image is "PNG or JPEG". Praxis's own `image` dependency (`Cargo.toml:15`) enables `png` only, but `gltf` 1.4.1's `import` feature (its `Cargo.toml`, `import = ["base64", "image", "urlencoding"]`, `image` with `jpeg` and `png`) unifies into the same `image` build, and `Cargo.lock` lists `zune-jpeg` under `image`. JPEG decodes, so the comment is true, but it rests on another crate's feature list rather than Praxis's. A dependency note for lane 7 or the tooling, not a comment defect here.
7. `src/assets/material.rs:22-27` quotes a measurement, "saves 126 µs of minification's lit pass", and cites `docs/performance.md`, where line 2768 records "data 4x 126 µs" against the Golden commit it names. The figure matches its record. A number in a shipping comment with its source is weighed and not filed; the verifier may hold otherwise under the axis's "nothing in the tree re-measures" clause.
8. `src/cook/mesh.rs:158-164` describes `meshlet_triangle_counts` as "like meshoptimizer's `meshopt_buildMeshletsScan`, which `meshopt` binds only as an `unsafe` call and which counts a degenerate triangle's repeated corner twice" (registry key `meshoptimizer-0.25-scan-meshlets`, whose quote is `used_extra = (av < 0) + (bv < 0) + (cv < 0)`). The code at lines 170-177 sorts the corners and counts a repeated one once. Read as two reasons not to call meshoptimizer, the comment is accurate; read as "the code does what meshoptimizer does", it is not. Whether the cut positions must match meshoptimizer's on a degenerate triangle is a correctness question (falls in lane 1); the comment is left as a near miss, and a verifier who reads it the second way may file it as `drift`, not load-bearing.
9. `src/assets/material.rs:137-140` says the fragment shader "looks up `materials[instance.material_id]`"; `src/render/shaders/main.wgsl:172` reads `materials[in.material_id]`, the Instance's Material ID carried as a fragment input. A paraphrase, not load-bearing; not filed.
10. `src/assets/mesh.rs:1-2` says `Mesh` is "as prepare needs it"; `bounds` is read by `src/render/acceleration_structures.rs:333` as well as `needs_material_handle` and `vertices` by `src/render/prepare.rs`. Narrow but not misleading; not filed.
11. `src/assets/mod.rs:9-10` re-exports `Mesh` and `MeshUse`, which no module outside `assets` imports (`grep -rn "crate::assets" src tests | grep -v "^src/assets/"` lists neither); their users are `src/assets/pools.rs:2` and `src/assets/loader.rs:3`. Inside a private module a re-export costs one name; not filed.
12. `src/cook/mesh.rs:67` hashes the Material ID with `to_be_bytes` while every other part of the key uses `to_le_bytes` (lines 44, 46). Harmless to the key's stability, but a change would re-cook every Mesh (new Cook keys) while moving no pixel. Not a health axis item; noted for whoever next touches the key.
13. `TextureSource::encoded` and `TextureSource::generated` (`src/cook/texture.rs:53,69`) have one caller each (`src/cook/mod.rs:417,277`) and differ only in the `SourceImage` variant they wrap. One constructor taking a `SourceImage` would serve, but `SourceImage` is private to the file and the two are four lines each; not filed.
14. `TextureSource::decode` (`src/cook/texture.rs:96`) clones a generated image's pixels on every call, and `cook` calls it for the Texture and again for its normal Texture. A cost, not a health item; falls in lane 3.
15. `src/cook/mesh.rs:201-221`, `weld` returns `bounds: None`; `cook` reads the added bounds from the source Primitives (lines 108-111), not the welded ones, so nothing is lost. Correct; noted so no later reader files it.
16. Allowed abbreviations weighed: `u`, `v` as UV components (`src/cook/mesh.rs:205,261,576`), `x`, `y` as texel coordinates throughout `src/cook/texture.rs`, `T` as a generic parameter (`src/cook/mesh.rs:538`), `remap` and `bc7` as meshopt's and intel_tex_2's own names, `uv`, `uvs`, `rgba`, `id`, `guid`, `hex` and `gltf` as the domain's words.
17. `#[link(name = "stdc++")] unsafe extern "C" {}` at `src/cook/texture.rs:16-18` is the one `unsafe` in these files, with its reason stated in the comment above it (a prebuilt library linked without its runtime on Linux). Not a health item.
18. The repeated hand-rolled GPU skip (L5-C10, L5-C11) also appears at `src/assets/pools.rs:966`, in the file that defines the helper (falls in the Cooked cache and the Pools section), and in 15 further files under `src/render/`, `src/capture/`, `src/gpu/` and `tests/` (falls in render and tooling).

**C.**

1. `MESH_CAPACITY` (`src/assets/pools.rs:12`) is `pub` and has no reader outside `pools.rs` (`grep -rn "MESH_CAPACITY" src tests`: 4 hits, all in the file). `assets` is a private module (`src/lib.rs:18`), so `pub` reaches no more code than no modifier would; a mixed `pub` spelling inside a private module is not a Finding on its own, so it is recorded in the interface table and not filed.
2. `MeshPool::load` (`pools.rs:131`) is a one-line wrapper over `load_for` with `MeshUse::DrawnAndTraced`, its one shipping caller `src/assets/loader.rs:60` and nine test callers. Not filed: the wrapper is what lets tests not name the default. The near-miss is in the loader: `PoolLoader::mesh` at `loader.rs:56-61` repeats the body of `PoolLoader::mesh_for` at `loader.rs:83-89` instead of calling `self.mesh_for(id, MeshUse::DrawnAndTraced)`; filed from the loader as L5-C09.
3. `gpu_context` is written out four times in the crate: `src/assets/pools.rs:952` (the `pub(crate)` one three render modules import), `src/render/renderer.rs:2578` (`pub(crate)`, which `renderer.rs`'s own tests call 22 times), `src/render/shader_function_tests.rs:2143` and `tests/crowd.rs:19`, each the same eleven lines. L5-C14 files the inline copy in the Cooked cache and the Pools; the other three fall in render and in tooling (`tests/crowd.rs` cannot reach a `pub(crate)` helper, so it would need a `pub` seam in `src/gpu/` or `src/capture/`).
4. `CookedTexture::to_blob` takes its zstd level as a parameter, which the one shipping caller reads from `CookSettings::texture_zstd_level` (`src/cook/texture.rs:215`, default `zstd::DEFAULT_COMPRESSION_LEVEL` at `src/cook/mod.rs:38`), while `CookedMesh::to_blob` hard-codes `CookedMesh::ZSTD_LEVEL = 19` (`cooked.rs:501`). Two formats, two conventions for the same knob. Whether `texture_zstd_level` is an option every caller sets the same way is `CookSettings`'s question; weighed for the cooker's pipeline and recorded there.
5. The three `to_blob`/`from_blob` pairs were weighed. The Material and Mesh Blobs share the magic-and-version framing through `payload` (`cooked.rs:639`); the Texture Blob is KTX2 and carries its version in a key-value entry, so it cannot share it. The bodies differ in substance (raw f32s; KTX2 with per-level zstd; meshopt codecs inside one zstd frame), so no one function would serve. Not filed.
6. Naming weighed and allowed: `id` (domain word), `uv` (domain word), `hex` (`cooked.rs:758`, the word for a hexadecimal string, used as such by four cook modules), `fmt` (`std::fmt` and the `Display::fmt` method, the API's spelling), `dfd_byte_offset`, `kvd_byte_offset`, `sgd_byte_offset` (`cooked.rs:305-310`, `ktx2::Index`'s own field names), `blas`, `tlas` (domain words), `u32_bytes` (`pools.rs:88`, a unit-carrying name), `texels`, `mips` (domain words), `x` in `let vertex = |x| ...` at `pools.rs:631` and `[x, y]` at `pools.rs:782` (coordinates), the type parameters `N` and `T` (`pools.rs:503`, `884`, `997`; the rule names arguments, parameters, locals, fields and lambda bindings, not type parameters), and the WGSL in `trace_down` and `read_storage` (`tlas`, `tags`, `hit`, `ray`, `copy`, `source`: full words or domain words).
7. Figures in comments recomputed and found right: `pools.rs:596` "four 28-byte vertices, or 28 indices" (112 bytes; `CookedVertex` is 28 by the assert at `cooked.rs:403`); `pools.rs:733-734` "eight pools of full Arenas would take 660 MiB" (five 16 MiB Arenas plus a 65,536 by 40-byte Mesh table is 82.5 MiB; eight is 660); `pools.rs:982-983` and `991` against `CAPACITY`; `cooked.rs:829` the BC4 block counts; `cooked.rs:874-877` the 13-to-16 and 20-byte entry sizes and the `[60..64]` header offset (KTX2's `kvdByteLength`). `pools.rs:388` names `wgpu-core 30.0.1`; `Cargo.lock` pins `wgpu-core 30.0.1`.
8. ADR claims in comments checked: `cooked.rs:31-33` (refused rather than resized, ADR 0018's Mips clause), `cooked.rs:38-44` (64 vertices for six bits, 124 triangles, ADR 0018's Meshes clause and ADR 0019), `cooked.rs:80-82` (source, kind, name, Channel: ADR 0018's order), `cooked.rs:391` (28 bytes, ADR 0018), `pools.rs:7-8` (the split of the 8 GB Budget: `bench/budgets.json` still holds one `gpu_memory_bytes`, and ADR 0017 says M4 sets the per-kind sizes, so "until" is still true), `pools.rs:540-541` (2^31 - 1 on DX12 and non-NVIDIA Linux and Android Vulkan, ADR 0017's Arena clause). All match.
9. `assets/manifest.json`'s `dev_only` is `false` on all eight entries and is read only by `tests/asset_manifest.rs:191`, which requires it on a non-commercial licence; the cooker's `ManifestEntry` (`src/cook/mod.rs:58-69`) does not deserialise it, and nothing keeps a dev-only file out of a build, which `docs/research/test-assets.md` paragraph 4 says is the field's purpose. With no dev-only entry there is nothing unexercised in the code today; the gap between the policy document and the code falls in lane 7. `version` is likewise read only by the test (non-empty check at `tests/asset_manifest.rs:170`), as the policy's provenance record.
10. Cosmetic, not an axis: two doc comments carry an uneven wrap left by an edit, `cooked.rs:81` (a line past the comment column) and `pools.rs:145-146` (a long line then a four-word one). `rustfmt` does not rewrap comments.
11. `CookedCache::read` is `pub` and its only callers outside the file are two integration tests (`tests/cook.rs:286`, `715`), reached through `pub use assets::cooked` at `src/lib.rs:38`; `material`, `texture` and `mesh` are the shipping readers. The `pub` is needed for those tests and was not filed; whether the tests should read raw Blob bytes is lane 6's question.
12. `Arena::free` (`pools.rs:568`) is reached only by `allocate_all`'s rollback; Meshes never unload at this commit. Not vestigial: the rollback runs on every refused Mesh and `a_mesh_past_capacity_is_refused_and_logged_and_changes_nothing` pins it. A capacity's fitness and whether unloading will need it fall in lane 4.
13. `IndexEntry::cook_key` carries `#[cfg_attr(not(feature = "cook"), allow(dead_code))]` with the comment "Read by the cooker only" (`cooked.rs:666-668`); `grep -rn "cook_key" src/cook/mod.rs` confirms the cooker reads it at `src/cook/mod.rs:334-336` and nothing else does. Accurate.

**D.**

1. `Geometry` derives `Clone` (`src/scenes/mod.rs:1129`) and no `.clone()` of a `Geometry` exists in the crate (`grep -rn '\.clone()' src/scenes src/cook`: the hits are on `String`, `TextureSource`, `Vec3` iterators and a glTF blob). Not filed: a derive is a token, not an item, and `Default` beside it is used (`mod.rs:1153`, `town.rs:1142,1190,1204,1265`). The next audit can drop it with L5-C21's edit.
2. Allowed short names weighed and passed: `u`/`v` in `uv_sphere` (`mod.rs:1155,1158`) and `cylinder` (`town.rs:1214-1215`) are the components of a UV, a domain word; `x`/`y` texel coordinates (`mod.rs:731,749-750,1231-1232`), `x`/`z` wheel coordinates (`town.rs:928`) and the `(x, y, z)` axis vectors (`town.rs:1189`) are coordinates; `min`/`max` as `cuboid`'s corners (`town.rs:1187`) are glam's own spelling (`Vec3::min`, `Vec3::max`) and are not on the abbreviation list; `tau` (`mod.rs:1049`) binds `std::f64::consts::TAU` under its own name; `bone.0` (`mod.rs:1073`) reads a newtype's one field; `of` (`town.rs:926`) is a word, though it says little, and is used 21 times in one table. The abbreviation grep over both files (`buf|len|idx|tex|desc|cmd|enc|src|dst|pos|num|cnt|tmp|ret|err|ctx|res|attr|coord|dim|fmt|cfg|prim|mat|tri|vert|img|rot|dir|vel|col|px|rad|deg|ptr|val|arr|param|elem|seg|lat|lon|cam|geo|norm`) hit only `vec!` macros, `.len()`, `cfg` attributes and the `min`/`max` above.
3. Near miss, not filed: `spawn_texture_wall` (`mod.rs:813-821`) and `spawn_crowd_as` (`mod.rs:993-997`) place a centred 10 by 10 grid from an index, a column count and a spacing with the same four lines; `Block::new` (`town.rs:388`) and `flocks` (`town.rs:696`) have the same one-line `offset` closure with a different pitch. Two sites each; a shared helper saves about six lines net. Filed nothing; a fix ticket for L5-C20 or L5-C22 may fold the first pair in.
4. Near miss, not filed: `TownMesh` (`town.rs:25-107`) and `BoxMaterial` (`town.rs:112-158`) have the same shape: an enum, an `ALL` array in discriminant order, a `name` method and a `const _` that asserts the order. Two sites; a macro or trait would cost more than the 50 lines it saves, and the shape is the crate's idiom for an enum that indexes a `Vec` of Handles.
5. Not filed: `point_lights` (`mod.rs:402-407`) keeps `Self::Town => &[]` as its own arm beside `Self::TextureWall | Self::Crowd => &[]` to carry a comment that its doc comment (`mod.rs:390`) already says ("town's stand in its lamps, which its spawn places"). Two lines; a comment repeated, not wrong.
6. Not filed: the `Crowd` variant's doc (`mod.rs:56-62`, "Their Meshes have no BLAS … each casts its shadow through rigid shadow proxies") describes the default `CrowdForm::Proxies` (`src/render/renderer.rs:105-106,146`); under `Refit` or `Rebuild` the figures do have BLASes (`mod.rs:1014-1023`). The doc describes the scene as it ships, and `spawn_traced_crowd`'s doc (`mod.rs:966-968`) says the other form. Judged accurate.
7. Figures in shipping comments re-derived and found right (so the next audit need not): 40 by 65 segments make 2 × 40 × 65 − 2 × 40 = 5,120 triangles (`mod.rs:889-890`); 12 by 8 make 168 (`mod.rs:898-899`); 0.35 rad/m × 1.8 m = 0.63 rad = 36° (`mod.rs:909-910`); 1.8 m / 16 bones = 0.1125 m, half 0.05625 m (`mod.rs:1263-1264`); π(3 − √5) = 2.39996 (`mod.rs:914-915`); the orrery's 60 captured frames (`src/capture/mod.rs:99`, one `update` per frame at `mod.rs:258-260`) at 1/144 s total 416.67 ms = 26.667 Ticks, two thirds of the way (`mod.rs:370-372`); "about 250,000 Instances" (`mod.rs:63-64`) against `TOWN_INSTANCES = 254_256` pinned at `tests/frame_stats.rs:14`; atan(300 / 2000) = 8.5° and 0.01 / tan(8.5°) = 6.7 cm (`mod.rs:589-590`); sqrt(300) = 17.3 m (`mod.rs:685`); the light at (0, 1, 16) to the floor's far corners (±50, 0, −50) is 82.8 m (`mod.rs:687`); the mirror point of a 1 m light seen from 1.2 m over 30 m sits 16.4 m out (`mod.rs:682-683`); 99 × 101 = 9,999 (`mod.rs:778`); 20 × SUN / 4.47² = SUN and 20 / 4.85² = 0.85 on the ground (`town.rs:222-223`); the tallest roof plant tops out at 6 × 3.2 + 3.5 + 0.15 = 22.85 m, under the lowest bird at 30 − 4 = 26 m (`town.rs:234`); 4 / 60 m per frame at 60 Hz is 4 m/s (`town.rs:820-822`); `FLOCK_STRAY` = 50 − 30 = 20 m keeps a 30 m flock in a 100 m cell (`town.rs:246-248`); the lamp's light at y = 4.7 sits under its head at y = 4.85 to 5.05 (`town.rs:218-221,1055-1056`); the uv sphere's `u` starts at +Z and grows towards +X (`mod.rs:1148-1149,1160-1164`); `ORRERY_ORBIT_RADIUS` is in the parent's diameters because the parent's local radius is 0.5 (`mod.rs:622-623`); the model's bounds −1.2 to 0.95 have middle −0.125 (`mod.rs:572-573`). The path the `bent_middle` doc cites, `src/render/shaders/bend.wgsl` (`mod.rs:1102`), exists. The `Helmet` variant's "local runs only" (`mod.rs:28`) holds: neither CI workflow fetches it (`grep -rli 'helmet\|fetch' .github/workflows`: no hit) and `tests/frame_stats.rs:286-290` skips it when absent.
8. Not filed: `name`'s doc comment (`mod.rs:85`, "The name the capture command selects the scene by") names one of its callers; the bench, the cooker, the exporter, the Goldens and the Cooked cache's `AssetSource::Scene` also key on it (`src/bench/mod.rs:530`, `src/cook/mod.rs:276`, `src/cook/export.rs:161`, `tests/goldens.rs:333`, `mod.rs:108`). Narrow, not wrong.
9. Not filed: `spawn_crowd_as`'s `form` parameter (`mod.rs:981`) never carries `CrowdForm::Rebuild`, since `spawn_traced_crowd` passes `Refit` for both (`mod.rs:973`) and `src/state.rs:89` maps `Rebuild` to it; the arm `CrowdForm::Refit | CrowdForm::Rebuild => None` (`mod.rs:991`) is a pattern, not a branch, and the renderer, not the spawn, tells the two apart. L5-C20's form passes `crowd_form` straight through and the wrappers go.
10. Nothing for lane 1 found: `spin` (`mod.rs:664-670`) and `sway` (`mod.rs:1063-1077`) set state from `fixed_time.tick` and `TICK_SECONDS`, read no wall clock, and `spawn_crowd_as` and `spawn_town` give every entity they move a `PreviousWorldTransform` (`mod.rs:1020,1032`; `town.rs:797,805`), as ADR 0022 says; the orrery's `Spin` bodies too (`mod.rs:637`).
11. Lane 6, not filed here: the test module (`mod.rs:1246-1351`) pins `bone_transform` against the bend pass's oracle, each proxy inside its figure, and the figure inside its culling box; no test pins `uv_sphere`'s triangle count or `cylinder`'s winding, which `tests/frame_stats.rs:282-283` and the Goldens cover indirectly.
12. Style, not filed: `mod.rs:38` is a doc line of 118 columns that `rustfmt` leaves; the constants mix `COLORS` (`mod.rs:596,857,918`, after glTF's `base_color`) with `COLOURED_LIGHTS` and `centre` (`mod.rs:477,605`), a spelling choice, not an abbreviation.
13. L5-C20's fix is a judgement for the triage: moving `spawn_helmet_scene` into `src/scenes/` makes the scenes module depend on `src/helmet/` for `HelmetAssets`; the alternative that keeps the dependency direction, a `spawn` method with the helmet arm calling back into `state`, keeps one of the fifteen items exported. Either way the proof of unchanged behaviour is Goldens bit-exact and `tests/frame_stats.rs`'s Instance counts equal.

**E.**

1. `Teleported` (`src/transform/mod.rs:48`) is inserted by no shipping system: `grep -rn Teleported src tests` finds its definition, its reader `reset_teleported` (`transform/mod.rs:73,83`) and one insertion at `state.rs:528`, inside a test. `reset_teleported` runs every Tick over an empty query (`state.rs:338`). Not filed: ADR 0022 names the marker as the teleport mechanism ("a teleport goes through an explicit marker that resets previous to current") and `CLAUDE.md`'s review checklist requires a jump to insert it; the component is the designed hook for the first game-state system that jumps, its cost per Tick is one empty query, and a Finding against it would be placed wontfix on the ADR. Recorded here so the next audit does not re-find it.
2. `Transform` holds no `WorldTransform` author: `#[require(WorldTransform)]` at `components.rs:10` gives a spawned entity the identity until the first propagation, and `Simulation::new` propagates once before the first frame (`state.rs:106-108`). The doc "never authored" at `transform/mod.rs:7` holds for shipping code; tests construct it directly (`transform/mod.rs:291`). Clean.
3. The `Moved` type alias (`transform/mod.rs:88`) has one use, at line 96. Not filed: 2 lines, it names the filter the doc comment above the system refers to, and inlining it lengthens the parameter list clippy already allows past its argument limit (`transform/mod.rs:94`).
4. `WorldTransform::translation()` (`transform/mod.rs:12-14`) is a getter beside a public field, one of the width smells weighed. Not filed: 3 lines; its 6 callers would each become `.0.translation`, which is no smaller and reads worse at `prepare.rs:385` where the result of `drawn` is read.
5. `record_previous_curvatures` (`transform/mod.rs:63-67`) lives in the transform module but acts on `Bend`, a component the components module declares, so `transform` imports `Bend` (`transform/mod.rs:3`) while `components` imports `WorldTransform` (`components.rs:4`). Not filed: 5 lines, and its doc says why it sits beside `record_previous_world_transforms` (the same Tick-start role). A future bend module would take it with `Bend` and `drawn_curvature`.
6. `clamp_frame_delta` (`time/mod.rs:33-35`) is one `min` for one caller (`lib.rs:223`). Not filed: it keeps `MAX_FRAME_DELTA` and the clamp's name in the module that owns the Tick, and exporting the constant instead is the same line count.
7. `MAX_FRAME_DELTA` and `MAX_TICKS_PER_FRAME` (`time/mod.rs:18,27`) are `pub` with no reader outside their file, beside `SIXTY_HERTZ` at `pub(crate)`. Not filed: `mod time;` is private (`lib.rs:35`), so the spellings reach the same code, and a mixed `pub` spelling inside a private module is not a Finding on its own. Both are read by shipping code in the file (`time/mod.rs:34,56`), so neither is vestigial.
8. `bevy_ecs`'s `multi_threaded` feature and the `bevy_tasks` dependency (`Cargo.toml:17-18`) serve only L5-C24's pool; the dependency question falls in tooling and is an owner decision under `CLAUDE.md`'s Stack.
9. The doc of `simulation_schedules` at `state.rs:313-315` reads "The State's one schedule: its systems, timed by `recorder`, on the single-threaded executor. The schedule run once per frame and the fixed schedule run once per Tick." for a function returning two schedules; the first sentence is stale. Falls in tooling (`state.rs` is outside the module); severity would be drift, not load-bearing.
10. `a_press_in_a_frame_that_runs_no_tick_is_consumed_by_the_next_tick` (`time/mod.rs:265-282`) builds its own `(count_presses, clear_just_pressed).chain()` schedule, so it pins the order the test wrote, not the orchestrator's at `state.rs:337`; of the shipping code in `time/mod.rs` it exercises only that zero Ticks run no system, which `a_frame_shorter_than_a_tick_runs_none_and_keeps_its_time` (`time/mod.rs:243`) already pins. Whether the orchestrator's order needs its own seam falls in lane 6.
11. `despawning_a_parent_despawns_its_subtree` (`transform/mod.rs:311-326`) relies on `bevy_ecs`'s linked despawn of `ChildOf` descendants; it does exercise the despawned-entity branch at `transform/mod.rs:128-133` through `RemovedComponents<ChildOf>`, so it pins shipping code. Clean.
12. The components carry no GPU resource (`MeshHandle`, `MaterialHandle` are Pool indices, `CONTEXT.md` Handle) and no behaviour beyond pure functions of their fields (`to_affine`, `drawn_curvature`), matching ADR 0006; `Transform`'s size and alignment are asserted at `components.rs:17-18` against the doc at `components.rs:7`. Clean on the GPU-free rule.
13. `Time` (`time/mod.rs:9-14`) has no `///` doc and its one field is documented with `//`. Not filed: the comment is accurate, and the rule "Don't add comments to code you didn't change" means a fix ticket would not carry it alone.
14. Abbreviations weighed and allowed: `θ` (`transform/mod.rs:19`) and `α` (`time/mod.rs:44`) as textbook symbols in comments, which the full-word rule permits; `uv`-free and `id`-free files; `x`, `y`, `z` as coordinates in comments and test oracles only.
15. Comment figures recomputed and found right: `transform/mod.rs:19-20` (1 − cos(θ/2) at θ = 0.09 rad is 0.00101, and 0.09 rad per 1/64 s is 5.76 rad/s, written 5.8); `time/mod.rs:21` (15,625,000 ns is 1/64 s); the test at `time/mod.rs:206-208` (250 ms is 16 Ticks, 12 dropped is 187,500 µs) and `time/mod.rs:217-222` (100 ms is 6.4 Ticks, 2 dropped is 31,250 µs, 0.4 carried).

### Counts

| Severity class | Findings |
|---|---|
| `image` | 0 |
| `budget` | 0 |
| `correctness` | 0 |
| `drift` | 3 (L5-C13, L5-C17, L5-C23); load-bearing 1 (L5-C13) |
| `health` | 21 (L5-C01 to L5-C12, L5-C14 to L5-C16, L5-C18 to L5-C22, L5-C24) |

By axis: over-engineering 11 (L5-C03, L5-C06, L5-C09, L5-C10, L5-C11,
L5-C14, L5-C15, L5-C18, L5-C19, L5-C21, L5-C22); vestigial 3 (L5-C04,
L5-C05, L5-C24); cognitive complexity 2 (L5-C01, L5-C02, the probe's two
hits in this module, both tests); naming 1 (L5-C16, a count with no
name); module depth 4 (L5-C07, L5-C08, L5-C12, L5-C20); comment accuracy
3 (the `drift` rows). Of the `health` rows, 5 are in test modules only
(L5-C01, L5-C02, L5-C10, L5-C11, L5-C14) and 16 in shipping code. No
implied fix moves a pixel or changes a Blob's content hash, so every
`health` row carries the refactor licence and none is a defect. Axis
sweep: 18 file rows. Interface widths: 44 types and module surfaces, 290
public items counted. Files read in full: the 18 of the module (17 Rust
files, 9,397 lines, and `assets/manifest.json`); read in part for use
counts: `src/lib.rs`, `src/state.rs`, `src/bin/praxis.rs`, `src/helmet/`,
`src/preflight/`, `src/capture/`, `src/render/` (prepare, the renderer,
the acceleration structures, the bend pass, the shader function tests),
`tests/`, `scripts/cycles-reference/render.py`, `scripts/gallery.sh`,
`docs/references.toml` and the ADRs the module cites.

## tooling

Scope: `src/bench/` (`mod.rs`, `provenance.rs`, `report.rs`, `history.rs`
and `history/import.rs`, `history/suspects.rs`, `history/verdicts.rs`),
`src/capture/mod.rs`, `src/stats/mod.rs`, `src/preflight/mod.rs`,
`src/gpu/mod.rs`, `src/bin/praxis.rs` (the thin binary, read here as the
tools' command dispatch and the site of the cognitive complexity probe's
third hit), `scripts/*.sh` with `scripts/sweep.plist` and
`scripts/sweep-task.xml`, `scripts/cycles-reference/` (`calibrate.py`,
`render.py`, `compare.py`, `DIFFERENCES.md`, `display-ramp.txt`) and
`.github/workflows/`; 10,130 lines of Rust and 1,906 of shell, Python,
Markdown, XML and YAML, beside the 350-line display ramp the Cycles
comparison checks itself against. The lane's coverage floor outside `src/render` is
every fifth file in path order, a wrong sample opening its module; this
section read every file in full with its test module instead, so the
sweep table is the record of coverage. Six readers at high effort shared
the module by file, each with the lane brief verbatim: the bench
(configurations, measurements, compare, provenance and the report); the
Bench history (records, Series, the worktree and the Suspects); the
import and the verdicts (the Baselines' import, the sweep's bisect and
the Regression issues); capture, stats and preflight; the GPU context and
the binary; and the scripts and the workflows. Each swept its files along
the six axes and recorded the sweep. The cognitive complexity probe,
`cargo clippy --all-targets -- -W clippy::cognitive_complexity`, ran once
at the frozen commit and named one function in this module, the binary's
command parser, filed on that axis below; its other two hits are tests in
the scene exporter, filed under cook, assets and scenes. Every `where` is
`path:line` at the frozen commit, given in full. `deviations.md` holds no
row at this commit; `DIFFERENCES.md` was read for what is not a Finding.
`tests/conventions.rs` was read for the rules already mechanical, and
none of its catches is re-filed. The ADRs whose clauses fall here, 0013
and 0024, and ADR 0021 for the Capability floor, were read for the
comment-accuracy axis. Each Finding's evidence block says whether the fix
it implies can move a pixel, change what a measurement means (which
raises the Harness version), change a Bench record's key, or change what
the verify command, CI or the sweep does. `src/lib.rs`, `src/state.rs`,
`src/camera/`, `src/input/`, `src/environment/`, `src/helmet/` and
`tests/*.rs` fall in no module of this lane's split; they were read for
use counts, and what was seen in them is under *Notes for the verifier*
as outside every module's scope, for the report session to place.

### Axis sweep

| File | Over-engineering | Vestigial | Cognitive complexity | Naming | Module depth | Comments |
|---|---|---|---|---|---|---|
| `src/bench/mod.rs` | clean (the six `check_budget` calls in `compare` and the five `spread(...)` builds in `measure` weighed, see the notes) | L5-T01 | none in this file (probe ran clean) | L5-T03, L5-T04 (`stats`, `rmse`, `p95`, `p99`, `tlas` weighed as the domain's words) | L5-T09 | L5-T02, L5-T05, L5-T06, L5-T07 (the ADR 0022 citation at `mod.rs:453` checked against the ADR's line 23 and holds) |
| `src/bench/provenance.rs` | L5-T19 | clean (`Checkout.branch` and five `Provenance` fields are written and read by no code, weighed as record data, see the notes) | none in this file (probe ran clean) | clean (`driver_info` is wgpu's `AdapterInfo` field, `stdin` is std's) | L5-T09 | clean (`HARNESS_VERSION` 10 is 1 plus the import's 9 breaks; the 17 tracked top-level paths split 5 measured and 12 unmeasured as the comment says) |
| `src/bench/report.rs` | clean (the test module's `BUDGETS` twin of `mod.rs`'s weighed, see the notes) | clean (`grouped`, `signed`, `ratio` each have 2 or more callers) | none in this file (probe ran clean) | clean | clean (`report` is the file's one public item, called at `src/bin/praxis.rs:681` and `:691`) | L5-T08 (the header's `docs/performance.md` exists and `docs/agents/workflow.md:55` names it) |
| `src/bench/history.rs` | L5-T11, L5-T12 (the in-memory and stored twins, the `SeriesRecord` trait and `MedianOf` were weighed and kept: notes 1 to 4) | clean (every constant at `history.rs:21-40` has a reader: `grep -rn` counts in *Interface widths*; `Default` on `Publication` is used at `verdicts.rs:668`; every test helper is used twice or more) | none in this section (probe ran clean) | clean (`json` at `history.rs:962,1047` is the domain's word; `tip`, `main`, `path`, `left`, `right` are words; `%ct` is git's format) | L5-T10 (the `pub` fields of `Publication`) | L5-T13 (every other doc comment, the `README` constant and each `tracing` message were read against the code beside them: the README's configurations and quiet thresholds are pinned by the test at `history.rs:1601-1620`, its Suspect clause matches `suspects.rs:7-19`, its tagging clauses match `tags` at `history.rs:808-845`, its replacement clause matches `apply` at `history.rs:952-981`) |
| `src/bench/history/suspects.rs` | clean (`Step` is built once and read once, the four constants each read once in `band` or `suspects`; a struct for four fields of one step is the smaller form, not a tuple) | clean (`WINDOW_RECORDS` 2 uses, `FEWEST_JUDGING_RECORDS` 1, `MEDIAN_ABSOLUTE_DEVIATIONS` 1, `SMALLEST_SHIFT_FRACTION` 1, all read) | none in this section (probe ran clean) | clean (`center`, `deviation`, `tolerance`, `slower`, `continues` are words; `NOISE` is a test constant) | clean (`Point` and `suspects` are `pub(super)`, one caller at `history.rs:366-374`) | L5-T14 |
| `src/bench/history/import.rs` | L5-T16, L5-T17 (`BaselineCommit`, `Baseline` and `ImportedProvenance` beside `history.rs`'s `StoredProvenance` weighed, see note 3; `import_up_to`'s parameters, note 4) | L5-T15 (every other constant and function has a reader: `BASELINES_DIRECTORY` 2 uses, `HISTORY_STARTED` 1, `IMPORTED_SOURCE` read in `history.rs:361`, `backend_before_keyed`, `remeasured` and `measured_records` each called) | none in this section (probe ran clean) | clean (`json`, `p95_over_median`, `ci_configuration` weighed; every local, field and parameter a full word) | clean (3 non-private items, see Interface widths) | clean (the `HISTORY_STARTED` doc's "before it" against an inclusive `rev-list`, note 7; the `baseline_records` doc's "the quieter stays", which `apply` in `history.rs:936` does, not this function, read as a reason for the order and left) |
| `src/bench/history/verdicts.rs` | L5-T19 (`Suspect` and `Verdict` sharing five fields, note 5; `Row` beside `SeriesRow`, note 6; `Filing`, `State` and `Issue` each with every variant and field constructed and read) | clean (`NIGHT_VERDICT` 2 uses, `BENCH_PAGE` 2, `TRIAGE_LABEL` 3, `END_RUNS` 4, `SMALLEST_HELD_SHIFT` 5; every private function called; `pinned_sources` and `VERDICTS` read in `history.rs:1026`) | none in this section (probe ran clean) | clean (`gh`, `url`, `tip`, `key`, `cpu`, `gpu`, `fmt` weighed as the domain's and std's own words; `each` in a closure, `so_far` in a test) | 14 of 16 `pub(super)` items have no reader in `history.rs`, see Interface widths and note 1; not filed in a private module | clean (the `next_commit` doc's "the commit that changed the source to the bad end's" against the one-change case, note 8; the `record` doc leaves out that it rewrites `README.md`, read as harmless) |
| `src/capture/mod.rs` | clean (the three layers `capture`, `capture_frames`, `render_frames` each have callers of their own; `FrameWait`'s two variants are both constructed, `Block` in the file and `Spin` at `src/bench/mod.rs:442`; `texture_bytes` is not here) | clean (every `pub` and `pub(crate)` item has a reader outside the file; `ImageDifference` is named nowhere outside but is the return type of `compare_images`) | none in this section (probe ran clean) | clean (`gpu` at `:294` is a domain word; `oldest`, `drawn`, `wait` full words) | clean (14 items, every one with a caller outside the file) | clean (`:9-10` against `src/gpu/mod.rs:457-461`, `:13-14` against `tests/goldens/`, `:98` against `capture`, `:132-137` against `:151-158`, `:282-288` against `src/bench/mod.rs:439-442`) |
| `src/stats/mod.rs` | L5-T18 | clean (every `FrameStats` field is read outside the file or printed by `Display`, which `praxis capture` prints at `src/bin/praxis.rs:785` and `tests/frame_stats.rs:253` reads; `texture_bytes`'s `transient_saves_memory` takes a variable at two call sites and `false` at one) | none in this section (probe ran clean) | clean (`gpu` locals at `:121`, `:133` are a domain word; `formatter`, `pass`, `span`, `index` full words) | clean (the recorder's `pub(crate)` counters are written by `src/state.rs`, `src/render/renderer.rs` and `src/render/readback.rs`; `SystemTimer::scope` has its one caller at `src/state.rs:319`) | L5-T23 |
| `src/preflight/mod.rs` | L5-T19 | clean (`LISTED_NAMES` read at `:213`, `:622`, `:623`; both `assets()` sides built, CI runs `--no-default-features` at `.github/workflows/ci.yml:116` and `:134`; every `Verdict` variant constructed; `Machine::description` read at `:118`) | none in this section (probe ran clean) | L5-T21, L5-T22 | L5-T20 | L5-T24 (`:212` against `src/bin/praxis.rs:434-441` and `:219` against `.github/workflows/ci.yml:178` hold; the goldens `FAIL` holds since `scripts/verify.sh:61` sets `PRAXIS_REQUIRE_GOLDENS=1`) |
| `src/gpu/mod.rs` | L5-T31 (the headless adapter request written three times; `backend_order`, `first_backend`, `first_success`, `instance_descriptor` each have one shipping caller and a test, which is the shape of a seam, not a wrapper) | clean (every private item has a caller in the file: `check_floor` 1, `floor_format_features` 2, `MAIN_PASS_FORMATS` 2, `DEVICE_MEMORY_BLOCK_BYTES` 2, `BACKEND_ORDER` 2; the four `pub(crate)` floor items each have one reader outside the file) | none in this file (probe ran clean) | L5-T32 (`info` at line 134; `msaa_x4`, `wgpu_instance`, `formatter`, `gaps`, `built`, `requested` weighed and allowed) | L5-T30 (`BelowCapabilityFloor` and `LimitGap` public with public fields, no reader outside the crate; `vendor` has one caller in `tests/frame_stats.rs`, `device_type` two in `tests/goldens.rs`, so those stay public) | L5-T28 (the report's doc names the floor's limits, the report prints three of four), L5-T29 (four measured timings in the block-size constant's doc) |
| `src/bin/praxis.rs` | clean (the four `adapter_info` sites and the two `publish` matches weighed, see the notes; every per-command function is called once from `run_command`, which is the dispatch's shape) | clean (`DEFAULT_CAPTURE_SIZE` 1 use at line 241; `take_option` 4 callers, `take_flag` 2; every `Command` variant is constructed in `parse_command` and matched in `run_command`; every test uses the `parse` helper) | L5-T25 (`parse_command`, the probe's 27 over 25) | clean (`ci_run`, `json`, `stdout`, `night`, `mode`, `tables`, `pixels` weighed: `ci` and `json` are domain words, the rest full words) | clean (`Command` is private with 17 variants, one reader in `run_command` and the test module; nothing in the file is `pub`) | L5-T26 (the usage text omits `preflight`), L5-T27 (the usage text lists five `cook`-gated commands in a build that lacks them); the doc comments of `bench`, `publish`, `compare`, `report`, `capture`, `difference` and the comment at line 601 checked against their code and `src/bench/mod.rs:55-87` and found accurate |
| `scripts/verify.sh` | clean (three functions, each called from the step list at `:72-76`; `bench`'s scene list is one pipeline) | clean (`skip_aliasing` set at `:37`, cleared at `:42`, read at `:52` and `:77`) | none in this file (probe ran clean; a shell file, no probe; nesting depth 2) | clean (`logs`, `results`, `step`, `goldens_job`, `others`; `fmt` at `:72` is cargo's subcommand) | clean (3 functions, 0 callers outside; the script is the interface) | L5-T35 (`:56`); `:28-36` against `src/bench/mod.rs:48,64` and `:2-3` against `docs/agents/workflow.md`, PRs, match |
| `scripts/gallery.sh` | L5-T33 (`:57-59`, the Blender command line spelled twice) | clean (every function called: `blender` 4 sites, `reference` 1, `capture` 3) | none in this file (probe ran clean; nesting depth 2) | clean (`against`, `golden_frame`, `town_frames`, `form`; `tmp` at `:5,9,28` is the `target/tmp/` directory's name) | clean (3 functions, 0 callers outside; one positional argument and one environment variable) | clean (`:31-33` equal `tests/goldens.rs:294,299`; `:2-3` "every Reference scene": 11 variants at `src/scenes/mod.rs:88-98`, 11 captured at `:111-120`; `:105` against `src/cook/export.rs:47-48`) |
| `scripts/pr-lint.sh` | clean (`(\.rs$\|/)` at `:67` anticipates a `src/cook/texture/` directory that does not exist; `CLAUDE.md`'s split rule makes the form a convention, note 14) | clean (`miss` 11 sites, `section` 5) | none in this file (probe ran clean; nesting depth 3 at `:65-85`) | clean (`misses`, `heading`, `verification`, `uncounted`, `unlinked`; Perl's `$embedded`, `$text`, `$url`, `$found`) | clean (2 functions; the interface is three positional files, called at `.github/workflows/pr-lint.yml:57` and `scripts/pr-lint-test.sh:60`) | L5-T37 (`:30`); `:2-4`, `:62-64` and `:102` match the rules at `:29-105` |
| `scripts/pr-lint-test.sh` | clean (six helpers, each with 2 to 26 call sites) | clean (every `expect_fail` names a rule `pr-lint.sh` holds at `:29-105`; every `expect_pass` exercises a form the lint handles: fences `:22`, CRLF `:21`, counts kind first `:47`, 64 KB `:36-38`, `perl -CA` `:95`) | none in this file (probe ran clean; one loop at `:177-181`) | clean (`failures`, `cases`, `changed_paths`, `fragment`, `human_commit`, `fence`; `raw` at `:153` is the raw-content host's own word) | clean (6 helpers, 0 callers outside; `pr-lint.yml:36` runs the file) | clean (`:2-3`, `:43`, `:75`, `:89-90`, `:113-114`, `:158` describe the code beside them; `:23` "20 passed" is sample body text, note 13) |
| `scripts/sweep.sh` | clean (`log` 14 sites, `fail` 7, `may_start` 2, `gpu_busy` 1, `tree_at` 1; the two environment knobs at `:44-45` are the operator's, note 6) | clean (`bisect_done` set `:153,160`, read `:186`; `runs` `:152,169,170,177`) | none in this file (probe ran clean; the body is one block of nesting depth 3 at `:154-175`) | clean (`cap_seconds` carries its unit, `wait_start`, `tree`, `build`, `fetched`, `night`, `job`; `dev` is `/dev/null`) | clean (5 functions, 0 callers outside; `sweep.plist:11` and `sweep-task.xml:36` run the file) | clean (the install recipe `:18-43` against `sweep.plist` and `sweep-task.xml`, note 15; `:44-45` `WGPU_BACKEND` is read at `src/gpu/mod.rs:485-506`; `:59-61` dated timings, note 5; `:99-101` against `:103-105`; `:136-139` against `:138-139`) |
| `scripts/sweep.plist` | clean | clean (every key read by launchd) | none in this file (no code) | clean | clean (one program, one schedule) | clean (`:3` and `:13-14` match `:10-11` and `:15-16`) |
| `scripts/sweep-task.xml` | clean | clean | none in this file (no code) | clean (`@SWEEP@`, `@BASH@` are the recipe's placeholders, `sweep.sh:23,32`) | clean (one action, one trigger) | clean (`:1-3` and `:6` match `:10-14`, `:19`, `:28`, `:36`) |
| `scripts/cycles-reference/calibrate.py` | clean (`measured` has one caller, `:83`, and is a function, not an abstraction) | clean (`TOLERANCE` read at `:86-87`) | none in this file (probe ran clean; one loop) | clean (`plane_data`, `camera_data`, `add_light`, `illuminance`, `luminance`; `np` by convention) | clean (3 functions, 1 constant; `gallery.sh:55,57` run the file twice) | clean (`:2-10` against `:59-89`; `:72` 1 − (10 / 20)⁴ = 0.9375 holds; `:21` against `:22-24`) |
| `scripts/cycles-reference/compare.py` | clean (the inverse matrices at `:69-74` are literal, as `gt7.wgsl:98-108`'s, by design under the ramp oracle) | clean (13 constant names, each read; 14 functions, each called; `crop` at `:198` has no caller in the tree but its seven products are committed, note 9) | none in this file (probe ran clean; `__main__` has nesting depth 2) | L5-T34 (`:79`) | clean (interface is three commands; `gallery.sh:62,77` run `ramp` and `compare`) | clean (`:2-14` against `:185-207`; `:27-29`, note 8; `:137-138` against `src/render/shader_function_tests.rs:1084-1089`) |
| `scripts/cycles-reference/render.py` | clean (`Gltf`'s 5 methods, `MaterialBuilder`'s 3, `instancer`, `to_blender`: each called; `point_light`'s `casts_shadow` takes both values, `calibrate.py:69` and `:404`) | clean (9 constants each read; `UNBOUNDED_RANGE` only by `calibrate.py:69,71`, note 11; `SCENE_FORMAT_VERSION = 1` equals `src/cook/export.rs:31`) | none in this file (probe ran clean; `build` is 68 lines at depth 3, `:374-441`) | clean (`x, y, z, w` at `:391,431` are quaternion components; `co`, `bpy`, `bsdf` are Blender's own; `uri`, `uvs` domain words) | clean (9 constants, 11 functions, 2 classes; `calibrate.py` imports 7 functions and 2 constants, `compare.py` 2 functions, `gallery.sh:76` runs the file) | clean (`:2-11` against `:52-86` and `:444-450`; `:24-25`, `:28-29`, `:33-34`, `:37` match the lines beneath; `:37-40` reads `src/render/shaders/shading.wgsl:22`) |
| `scripts/cycles-reference/DIFFERENCES.md` | clean (no code) | clean (5 entries, 7 images named, 7 files in `differences/`) | none in this file (no code) | clean | clean | L5-T38 (`:9-10`) |
| `scripts/cycles-reference/display-ramp.txt` | clean (data) | clean (read by `compare.py:143-144` and written by `src/render/shader_function_tests.rs:1121-1131`) | none in this file (no code) | clean | clean (348 rows, 2 readers) | clean (`:1-2` names the test that writes it, `shader_function_tests.rs:1091`) |
| `.github/workflows/ci.yml` | clean (the setup block repeats four times, `:37-43`, `:63-67`, `:101-107`, `:151-155`, and the lavapipe install three, `:69-73`, `:109-113`, `:157-161`; a composite action would be as long as what it removes and needs a `components` input, note 2) | clean (`benched` output `:149` read at `:216`; `release-build-seconds` `:169` read at `:238`; the three failure artifacts are read by hand, `:183-184`; the `Build` step at `:51-52` repeats what `Test` compiles, note 4) | none in this file (no code) | clean (`tree`, `started`, `ARTIFACT_DIRECTORY`) | clean (5 jobs; one output and one artifact cross jobs) | L5-T36 (`:58`); `:13-15` holds for concurrency, note 3; `:171-173` lists the 8 counters of `src/bench/mod.rs:198-207`; `:25-29` against `tests/goldens.rs:1-5`; action pins are lane 7's L7-R04, note 1 |
| `.github/workflows/pr-lint.yml` | clean | clean (both `gh api` outputs read at `:57-58`) | none in this file (no code) | clean (`BODY`, `REPOSITORY`, `NUMBER`; `sha` at `:52` is git's word) | clean (1 job, 2 steps) | clean (`:3-4` against `:5-8`; `:20` against `:21`; `:38-40` against `:42-58`) |

### Interface widths

| Type | Public items | Callers outside the file | Wider than needed | Where |
|---|---|---|---|---|
| `Configuration` | 5 `pub` fields, 4 associated constants | `src/bin/praxis.rs` (built at `:220`, constants at `:266-275`, `:603`), `src/bench/history.rs:824-826` and tests, `src/gpu/mod.rs:864` (test) | `CI` is built by no script, workflow or Series (L5-T01) | `src/bench/mod.rs:41-93` |
| `Measurements` | 16 `pub` fields, `from_baseline_json` | `src/bin/praxis.rs` (7 sites, `:498` to `:702`), `src/bench/history.rs` (12 sites), `src/bench/history/import.rs:488,520`, `report.rs` | no: the binary reads `scene`, `width`, `height`, `adapter`, `backend`; the history reads the rest | `src/bench/mod.rs:149-193` |
| `Counters` | 8 `pub` fields, `NAMES` (`pub(super)`), `of` and `named` private | `history.rs:16,246,1127`, `import.rs:532` (`NAMES`) | no | `src/bench/mod.rs:198-258` |
| `Aliasing` | 2 `pub` fields, `pub fn between` | none: `between` is called at `mod.rs:351` and in 3 tests of the same file | yes: `between` is `pub` in the `pub mod bench` and nothing outside the crate calls it (L5-T09) | `src/bench/mod.rs:269-330` |
| `CounterFrames`, `Spread` | 2 and 2 `pub` fields | `history.rs` (12 sites), `import.rs:14,240`, `provenance.rs:12,98` | no: both are field types of `Measurements` | `src/bench/mod.rs:360-370` |
| `Budgets` | 6 `pub` fields | `src/bin/praxis.rs:6,592,673`, `report.rs` | no | `src/bench/mod.rs:375-389` |
| `CompareMode`, `Verdict`, `Finding` | 2 variants, 2 variants, 3 `pub` fields plus `Display` | `src/bin/praxis.rs:6,88,301,305,591` (`CompareMode`), `:6,657` (`Verdict`), `:657-659` (`Finding` through its fields and `Display`) | no in width; both names collide with the glossary (L5-T03, L5-T04) | `src/bench/mod.rs:392-421` |
| `run`, `compare` | 2 `pub fn` | `src/bin/praxis.rs:458`, `:645` | no | `src/bench/mod.rs:426`, `:634` |
| `baseline_directory`, `BASELINE_DIRECTORY`, `BUDGETS_PATH` | 1 `pub fn`, 2 `pub const` | `baseline_directory`: `src/bin/praxis.rs:702`, `src/preflight/mod.rs:198`, `src/bench/history/verdicts.rs:13,405`; `BASELINE_DIRECTORY`: `src/preflight/mod.rs:10,202,203,710` only; `BUDGETS_PATH`: `src/bin/praxis.rs:592,673` | `BASELINE_DIRECTORY` is `pub` and read only inside the crate (L5-T09) | `src/bench/mod.rs:32,35,109` |
| `Provenance` | 10 `pub` fields | `src/bench/history.rs:693-717` (reads `checkout`, `quiet`, `gpu_frame_p95_over_median`), `mod.rs:184,478` | the fields are a serialized record's; 5 of them and `Checkout.branch` are read by no code (notes 5) | `src/bench/provenance.rs:34-55` |
| `BenchStart` | 4 `pub` fields, `pub fn read`, `pub(crate) fn provenance` | `src/bin/praxis.rs:450` (`read()`), `src/bench/mod.rs:431,478` (`provenance`); no field is read outside `provenance.rs:101-116` | yes: four `pub` fields nothing outside the file reads (L5-T09) | `src/bench/provenance.rs:61-124` |
| `Checkout` | 4 `pub` fields, `pub fn read` | `history.rs:15,303,312,780,1261` (the type and `read` in a test), `verdicts.rs:898` (test), `mod.rs:1025` (test) | yes: `read` is re-exported `pub` and nothing outside the crate calls it (L5-T09) | `src/bench/provenance.rs:144-188` |
| `source_hash`, `git`, `HARNESS_VERSION`, `QUIET_P95_OVER_MEDIAN`, `MEASURED_PATHS` | `pub fn`, `pub(super) fn`, `pub const`, `pub(super) const`, private | `source_hash`: `history.rs` (4 sites), `import.rs:15,170`, `verdicts.rs:11,616`; `git`: `history.rs` (40 sites), `import.rs` (7), `verdicts.rs` (6); `HARNESS_VERSION`: `import.rs:387,605`; `QUIET_P95_OVER_MEDIAN`: `mod.rs:990`, `history.rs:1612-1615`, `import.rs:15,285` | no: `provenance` is a private module, so its `pub` reaches the crate only | `src/bench/provenance.rs:21-30,225,246` |
| `report` | 1 `pub fn`; `grouped`, `signed`, `ratio` private | `src/bin/praxis.rs:681,691` | no | `src/bench/report.rs:17` |
| `Publication` | 7 `pub` fields (`on_series`, `tagged`, `in_experiment`, `experiment`, `dropped`, `restamped`, `commit`), `Display`, derived `Default` | `src/bin/praxis.rs:475,515,523` print it through `Display` and read no field (`grep -n "\.on_series\|\.tagged\|\.in_experiment\|\.dropped\|\.restamped" src/bin/praxis.rs`: 0); `import.rs:105` and `verdicts.rs:668` construct it, both child modules | yes, L5-T10 | `history.rs:454-470` |
| `ReleaseBuild` | 2 `pub` fields (`binary_bytes`, `build_seconds`) | `src/bin/praxis.rs:505` constructs it | no | `history.rs:292-295` |
| the `publish` family | `publish`, `publish_experiment`, `check_experiment_name`, `unrecorded_scenes` (4 functions) | `praxis.rs:470,513` (`publish`), `praxis.rs:469,512` (`publish_experiment`), `praxis.rs:180` (`check_experiment_name`, also called inside `publish_experiment` at `history.rs:530`), `praxis.rs:534` (`unrecorded_scenes`, which `scripts/sweep.sh:80` reaches through `praxis history unrecorded`) | yes, L5-T12 (two publish functions where one serves) | `history.rs:502-621` |
| `Worktree` | private; `add`, `apply`, `rebuild_every_series`, `rebuild_series`, `series_lines`, `commit`, `Drop`, plus the field `directory` read by `import.rs:362` and `verdicts.rs:667`; `import.rs:319-383` adds `restamp`, `remove_imported`, `measured_records` | `import.rs` (3 method definitions, `directory`), `verdicts.rs:667` (`directory`), both through `push`'s closure (`history.rs:642-646`) | no | `history.rs:880-1098` |
| `SeriesRecord` | trait, 2 methods (`order`, `rows`) | none; 2 implementors (`StoredRecord` at `history.rs:340`, `StoredReleaseBuild` at `history.rs:399`), 1 generic site (`series_lines` at `history.rs:1039`) | no | `history.rs:328-338` |
| `BenchRecord`, `StoredRecord`, `StoredProvenance`, `StoredCounterFrames`, `SeriesRow` | private | `import.rs` uses `BenchRecord` (16 mentions across the two files) and `StoredRecord`, `SeriesRow` in its tests (`import.rs:385`) | no | `history.rs:124-282, 424-450` |
| the constants | `REMOTE` (7 mentions, this file), `BRANCH` (16, this file and `verdicts.rs`), `ON_SERIES` (12, this file and `import.rs`), `TAGGED` (7, same), `EXPERIMENTS` (2: definition and `history.rs:948`), `RELEASE_BUILD` (8, three files), `LAVAPIPE_ADAPTER_PREFIX` (2: definition and `is_lavapipe`), `PUSH_ATTEMPTS` (2: definition and `history.rs:671`), `README` (written at `history.rs:997` and `verdicts.rs:667`, tested at `history.rs:1618`) | as listed | no | `history.rs:21-119` |
| `suspects::Point`, `suspects::suspects` | `pub(super)`: 2 fields, 1 function | `history.rs:366,374` | no | `suspects.rs:22-87` |
| the import (`import_baselines`, `import_up_to`, `IMPORTED_SOURCE`) | 3: one `pub` function re-exported through `history.rs:18` and `bench/mod.rs:7`, one `pub(super)` function, one `pub(super)` constant | `import_baselines`: `src/bin/praxis.rs:522` (1 shipping caller); `import_up_to`: `src/bench/history.rs:1689,1723,1786` (3 test callers, 0 shipping); `IMPORTED_SOURCE`: `src/bench/history.rs:361` (1 shipping) and `:1836` (1 test) | no: `import_up_to` is the test seam for the one-shot command (note 4) | `src/bench/history/import.rs:69-91` |
| `Worktree`'s import methods (`restamp`, `remove_imported`, `measured_records`) | 0 public: three private methods on another file's type | none; all three called from `import_up_to` only (`import.rs:103,104` and `:327,348`) | no, but `restamp` writes nothing any longer (L5-T16) | `src/bench/history/import.rs:319-381` |
| the verdicts' commands (`suspects`, `bisect`, `file_regressions`, `Suspect`) | 4 `pub`, re-exported through `history.rs:19` and `bench/mod.rs:6-7`; `Suspect`'s six fields are `pub(super)`, so a caller outside the crate reads it through `Display` only | `src/bin/praxis.rs:550,563,581` (one caller each); `scripts/sweep.sh:155,193` through the binary (`history bisect`, `history regressions`); `history suspects` has no script caller, only the binary's command and its usage text | no: one function per command, each returning what the binary prints | `src/bench/history/verdicts.rs:123,317,366,451` |
| the verdicts' internals (`VERDICTS`, `pinned_sources`, `Commit`, `State`, `Pinned`, `Measured`, `Ends`, `Verdict`, `Row`, `Run`, `unjudged`, `step`, `Issue`, `issue`, `Filing`, `filing`) | 16 `pub(super)` | `VERDICTS` and `pinned_sources`: `src/bench/history.rs:1026` (1 each); the other 14: 0 callers in `history.rs` or anywhere outside the file (`grep -rn` over the worktree for each name, quoted in note 1) | 14 items are visible one module wider than any reader; inside the private `history` module the spelling alone is not a Finding (note 1) | `src/bench/history/verdicts.rs:16-31,39,46,59,67,81,148,159,169,222,569,678,688,745,757` |
| capture's surface (`capture`, `capture_frames`, `render_frames`, `FrameWait`, `read_back`, `submit_and_map`, `compare_images`, `difference_image`, `golden_key`, `ImageDifference` with 2 fields, `GOLDEN_DIRECTORY`, `GOLDEN_TOLERANCE`, `FRAME_COUNT`, `TARGET_FORMAT`) | 14 items: 8 `pub`, re-exported at `src/lib.rs:39-42`, 6 `pub(crate)` | `src/bin/praxis.rs` 5 (`capture` :764, `capture_frames` :751, `compare_images` :828, `difference_image` :822, `GOLDEN_TOLERANCE` :833); `src/bench/mod.rs` 3 (`capture` :341, `render_frames` :445, `FrameWait` :441-442); `src/preflight/mod.rs` 2 (`GOLDEN_DIRECTORY`, `golden_key`); `src/cook/export.rs:50` (`FRAME_COUNT`); render and assets test modules 3 (`TARGET_FORMAT` and `read_back` in `tonemapping_pass.rs`, `read_back` in `renderer.rs`, `submit_and_map` in `staging.rs` and `pools.rs`); 8 files under `tests/` | no | `src/capture/mod.rs:11-356` |
| `FrameStats` | 18 `pub` fields, `Display` | `src/bench/mod.rs`, `src/bench/report.rs`, `src/bench/history/import.rs`, `src/state.rs`, `src/render/renderer.rs`, `src/capture/mod.rs`, `tests/frame_stats.rs`; three fields (`renderables_refused`, `static_instances_left_out`, `allocator_report_bytes`) are written outside and read only by `Display`, which `src/bin/praxis.rs:785` prints | no | `src/stats/mod.rs:10-65` |
| `GpuFrameStats` | 13 `pub` fields | `src/render/readback.rs` (writes all 13), `src/bench/mod.rs` (reads 12), `src/bench/report.rs`, `tests/frame_stats.rs` | no | `src/stats/mod.rs:69-105` |
| `FrameRecorder` | 11 `pub(crate)` fields, 5 `pub(crate)` methods | `src/state.rs` (10 lines), `src/render/renderer.rs` (14), `src/render/readback.rs` (4), `src/cook/export.rs:58` | no, but the ten counters are listed four times (L5-T18) | `src/stats/mod.rs:204-309` |
| `SystemTimer` | 1 `pub(crate)` method `scope`; `tracing::Subscriber` (8 methods) | `src/state.rs:319`, through `recorder.systems` | no | `src/stats/mod.rs:316-394` |
| `texture_bytes` | 1 `pub(crate)` function | `src/render/targets.rs:373`, `src/render/tonemapping_pass.rs:142`, `src/assets/material.rs:233` | no | `src/stats/mod.rs:185` |
| preflight's surface (`Machine` with `classify`, `Verdict`, `Check` with 3 fields, `run`; `pub(crate)` `BUSY_GPU_PERCENT`, `gpu_load_percent`, `machine_warnings`, `other_praxis_processes`) | 8 `pub` items in a `pub mod`, 4 `pub(crate)` | `src/bin/praxis.rs:355-362` (`run`, `Verdict::Fail`, `check.verdict`, `Display`); `src/render/coverage_tests.rs:17,662` (`Machine`, `classify`); `src/bench/provenance.rs:13-14` (the 4 `pub(crate)`); `src/bench/history.rs:1611` (`BUSY_GPU_PERCENT`); 0 in `tests/` | yes: `Machine`, `classify`, `Check.name`, `Check.detail` have no reader outside the crate (L5-T20) | `src/preflight/mod.rs:22-136` |
| `GpuContext` | 8: the struct, 3 `pub(crate)` fields (`adapter`, `device`, `queue`), `headless` (pub), `with_surface` (`pub(crate)`), `backend`, `vendor`, `device_type` (pub) | `headless` 25 (13 in `src/`, 12 in `tests/`); `with_surface` 1 (`src/lib.rs:154`); `backend` 10 (4 in `src/`, 6 in `tests/`); `vendor` 1 (`tests/frame_stats.rs:63`); `device_type` 2 (`tests/goldens.rs:101-102`); `.adapter` read directly at 9 sites outside `src/gpu/` | no (each getter has a caller outside the crate; the field reads that bypass `device_type` are in other modules, see the notes) | `src/gpu/mod.rs:23-109` |
| The Capability floor | 7: `floor_features`, `floor_downlevel_flags`, `floor_limits`, `FLOOR_BINDING_ARRAY_ELEMENTS_PER_STAGE` (`pub(crate)`); `BelowCapabilityFloor` (pub, 4 pub fields); `LimitGap` (pub, 3 pub fields); `check_floor`, `floor_format_features`, `MAIN_PASS_FORMATS` private | `floor_features` 1 (`src/render/shader_tests.rs:454`), `floor_downlevel_flags` 1 (same line), `floor_limits` 1 (`src/render/shader_tests.rs:460`), `FLOOR_BINDING_ARRAY_ELEMENTS_PER_STAGE` 1 (`src/assets/material.rs:16`); `BelowCapabilityFloor` 0, `LimitGap` 0 (`grep -rn BelowCapabilityFloor src tests` hits `src/gpu/mod.rs` only; `grep -rn downcast src tests` hits nothing) | yes: two `pub` error types with seven `pub` fields in a `pub` module, read by nothing outside the file (L5-T30) | `src/gpu/mod.rs:259-379` |
| The adapter reports | 2: `capability_report`, `adapter_info` (pub); `adapter_report` private | `capability_report` 2 (`src/bin/praxis.rs:347`, `tests/gpu_context.rs:19`); `adapter_info` 5 (`src/bin/praxis.rs:531,548,562,578`, `src/preflight/mod.rs:106`) | no | `src/gpu/mod.rs:111-253` |
| The backend choice | 0 public: `BACKEND_ORDER`, `backend_order`, `first_backend`, `first_success`, `instance_descriptor`, `request_adapter`, `choose_surface_format` all private | none; each has 1 or 2 shipping callers in the file and a test | no | `src/gpu/mod.rs:418-555` |
| `Command` (the binary) | 0 public: a private enum of 17 variants (`Window`, `Capabilities`, `Preflight`, `Cook`, `Fetch`, `Capture`, `ExportScene`, `Difference`, `Bench`, `Publish`, `ImportBaselines`, `UnrecordedScenes`, `Suspects`, `Bisect`, `Regressions`, `Compare`, `Report`), five of them `cook`-gated | none (a binary crate); 1 reader in the file, `run_command` at `src/bin/praxis.rs:95-152`, plus the test module | no | `src/bin/praxis.rs:34-93` |
| The per-command functions | 0 public: `print_capabilities`, `preflight`, `cook`, `fetch`, `bench`, `publish`, `import_baselines`, `unrecorded_scenes`, `suspects`, `bisect`, `regressions`, `compare`, `report`, `capture`, `export_scene`, `difference`, and the helpers `parse_scene`, `baseline_path`, `result_paths`, `read_json`, `take_option`, `take_flag` | none; each command function has 1 caller in `run_command`; `parse_scene` 4, `baseline_path` 2, `result_paths` 3, `read_json` 5 | no | `src/bin/praxis.rs:346-837` |
| `scripts/verify.sh` (`step`, `bench`, `tests`) | 3 functions; no arguments, no environment variable | 0 function callers; the file is named by `CLAUDE.md`, `README.md`, `docs/agents/workflow.md`, `docs/agents/sessions.md`, `src/bin/praxis.rs`, `src/preflight/mod.rs` (6 files) | no | `scripts/verify.sh:14,45,58` |
| `scripts/gallery.sh` (`blender`, `reference`, `capture`) | 3 functions; 1 positional argument, 1 environment variable (`PRAXIS_BLENDER`) | 0 function callers; the file is named by `docs/agents/workflow.md`, `docs/agents/audit.md`, `docs/performance.md`, `.github/pull_request_template.md`, `scripts/pr-lint.sh`, `DIFFERENCES.md` (6 files) | no | `scripts/gallery.sh:43,68,83` |
| `scripts/pr-lint.sh` (`miss`, `section`) | 2 functions; 3 positional file arguments | `.github/workflows/pr-lint.yml:57`, `scripts/pr-lint-test.sh:60` (2 sites) | no | `scripts/pr-lint.sh:13,25` |
| `scripts/pr-lint-test.sh` (`passing_body` 3 sites, `run_lint` 2, `expect_pass` 16, `expect_fail` 26, `edited_body` 19, `images_body` 5) | 6 helpers; no arguments | `.github/workflows/pr-lint.yml:36` (1 site) | no | `scripts/pr-lint-test.sh:13-93,159` |
| `scripts/sweep.sh` (`log` 14 sites, `fail` 7, `gpu_busy` 1, `may_start` 2, `tree_at` 1) | 5 functions; 2 environment knobs (`PRAXIS_SWEEP_CAP_SECONDS`, `WGPU_BACKEND`), 1 re-exec guard (`PRAXIS_SWEEP_UPDATED`) | `scripts/sweep.plist:11`, `scripts/sweep-task.xml:36` (2 sites); `PRAXIS_SWEEP_CAP_SECONDS` is set nowhere in the tree | no (a documented operator knob) | `scripts/sweep.sh:64-65,102,109,129` |
| `scripts/cycles-reference/render.py` | 9 constants, 11 functions, 2 classes with 8 methods; command line of 3 to 4 arguments | `calibrate.py`: `world` 2, `link` 2, `point_light` 3, `sun` 1, `new_scene` 1, `require_blender` 1, `arguments` 1, `POINT_POWER_PER_INTENSITY` 1, `UNBOUNDED_RANGE` 2; `compare.py`: `arguments` 1, `require_blender` 1; `gallery.sh:76` runs it | no | `scripts/cycles-reference/render.py:26-40,43-156,162-366` |
| `scripts/cycles-reference/compare.py` | 13 constant names, 14 functions, 3 commands (`ramp`, `compare`, `crop`) | `gallery.sh:62` (`ramp`), `gallery.sh:77` (`compare`); `crop`: 0 in the tree (note 9) | no (the by-hand `crop` made the seven committed images) | `scripts/cycles-reference/compare.py:54-75,78-182,185-207` |
| `scripts/cycles-reference/calibrate.py` | 1 constant, 3 functions, 1 flag (`--wrong-unit`) | `gallery.sh:55` and `:57` (2 sites) | no | `scripts/cycles-reference/calibrate.py:26,29,50,59` |
| `.github/workflows/ci.yml` (5 jobs) | 5 jobs, 1 job output (`benched`), 4 artifacts | `publish-bench` reads `benched` (`:216`) and `bench-publish` (`:225-228`); the three failure artifacts are read by hand | no | `.github/workflows/ci.yml:33,57,93,144,214` |
| `.github/workflows/pr-lint.yml` (1 job) | 1 job, 2 steps | none; it runs the two scripts | no | `.github/workflows/pr-lint.yml:24` |

### Findings

| id | claim | severity | where | evidence | expected |
|---|---|---|---|---|---|
| L5-T01 | `Configuration::CI` and the `bench --ci` flag that builds it are run by no script, workflow or test of a run, and a lavapipe record of that shape joins no Series; the constant's two remaining readers use it as the Goldens' size, which the tree spells in five places. | health; 7 lines, 1 parse arm, 5 spellings of one size | `src/bench/mod.rs:67-73` | block below | One constant names the Goldens' size (640 by 360), `CI_COUNTERS` carries CI's size from it, and `CI` with its `--ci` arm goes; CLAUDE.md, "No throwaway code" and "Prefer deep modules behind small interfaces" |
| L5-T02 | The doc comments on `CI` and `CI_COUNTERS` say CI benches `CI`'s frames and falls back to `CI_COUNTERS` for a scene too dense, while CI runs `--ci-counters` for every scene and the history calls `CI_COUNTERS` "CI's configuration". | drift; load-bearing (a reader recording a lavapipe Baseline by it runs the wrong command and gets a record tagged off its Series) | `src/bench/mod.rs:63-66, 85-86` | block below | A comment that says what CI runs: `CI_COUNTERS` for every scene (`.github/workflows/ci.yml:178`), with `compare --ci` reading counters only |
| L5-T03 | `bench::Verdict`, the warn-or-fail outcome of `compare`, takes the glossary's term Verdict, which `CONTEXT.md` gives to what the sweep found of a Suspect and `history::verdicts::Verdict` implements, so the crate carries three `Verdict` types and the binary must spell out `praxis::preflight::Verdict::Fail`. | health; 37 sites in `mod.rs`, 2 in `src/bin/praxis.rs` | `src/bench/mod.rs:400-403` | block below | `CONTEXT.md`, Verdict: the glossary's term names one concept; a compare outcome is a `fails: bool` on the result (two variants, printed `warn` or `FAIL`), or a name outside the glossary |
| L5-T04 | `bench::Finding`, a metric that regressed or exceeds its Budget, takes the glossary's term Finding, which `CONTEXT.md` gives to an Audit lane's eight-field record. | health; 6 sites in `mod.rs`, 2 reads in `src/bin/praxis.rs` | `src/bench/mod.rs:405-411` | block below | `CONTEXT.md`, Finding: a name outside the glossary (Regression is also taken), such as `Breach` or `CompareResult`, chosen with L5-T03's |
| L5-T05 | The comment justifying `WARN_RATIO` and `FAIL_RATIO` quotes a noise measurement ("6 runs", "GPU medians within 3.4%, CPU medians of 31–42 µs within 8.1 µs") that nothing in the tree re-measures and that describes CPU frames two orders of magnitude below today's Baselines. | drift; not load-bearing (the ratios themselves are stated correctly beside it) | `src/bench/mod.rs:129-133` | block below | A comment that says what the ratios do and why without a dated figure (CLAUDE.md, Comments; `docs/audit/deviations.md`: "not in a code comment, where no number may sit"); the measurement belongs in `docs/performance.md` if kept |
| L5-T06 | The comment justifying `GPU_PASS_NOISE_FLOOR_MICROSECONDS` quotes a measurement ("over three runs on the M4 Pro, helmet's culling pass read 136 to 153 µs, every other pass within 3%") that nothing in the tree re-measures. | drift; not load-bearing | `src/bench/mod.rs:141-143` | block below | As L5-T05: the floor's purpose without the dated figure |
| L5-T07 | The `Display for Measurements` doc lists the line's fields and omits the dynamic TLAS CPU median the line prints, and the aliasing figures that follow the counters. | drift; not load-bearing | `src/bench/mod.rs:594-596` | block below | A doc that lists what the line holds: the CPU frame, the dynamic TLAS CPU median, the GPU frame, the gate, the shadow-ray time, each pass's median, the counters, then the aliasing figures when measured |
| L5-T08 | `report`'s doc says times compare "with a Baseline of as many frames on the same adapter, as `compare` judges them", but `compare` also refuses another backend, which `report` does not. | drift; not load-bearing (the binary pairs a result with its adapter and backend's Baseline, so no report crosses backends today) | `src/bench/report.rs:12-14` | block below | Either the doc says "on the same adapter, whatever the backend" or `times_baseline` also matches `backend`, as `compare` does at `mod.rs:673-682` |
| L5-T09 | The `pub mod bench`'s surface holds four items nothing outside the crate reads: `Aliasing::between`, `BASELINE_DIRECTORY`, `BenchStart`'s four fields and `Checkout::read`. | health; 7 declaration lines, 0 call sites outside the crate | `src/bench/mod.rs:5-35` (the module's surface) | block below | `pub(crate)` or private, as the module depth axis reads: in a `pub` module a `pub` item nothing outside the crate reads is wider than its callers need |
| L5-T10 | `Publication`'s seven fields are `pub` in a `pub` module, and nothing outside the crate reads one: the binary prints the `Display` only. | health; 7 field lines, 0 call sites outside the crate | `src/bench/history.rs:454-470` | block below | A `pub` item in a `pub` module that nothing outside the crate reads is wider than its callers need; the fields can be private, since the two constructors and the tests are child modules. |
| L5-T11 | `Worktree::apply` spells out a `Publication` of seven zero fields where the derived `Default`, which `verdicts.rs` already uses, serves in one line. | health; 9 lines | `src/bench/history.rs:934-942` | block below | `let mut publication = Publication::default();`, the form `verdicts.rs:668` takes. |
| L5-T12 | `publish_experiment` repeats `publish`'s prelude and body; the two differ only in the Experiment name and the release build, which the binary already keeps exclusive. | health; 20 lines, 2 branching call sites | `src/bench/history.rs:502-541` | block below | One `publish(repository, results, release_build, experiment)`; the binary's two `match experiment` arms at `praxis.rs:468-470` and `praxis.rs:511-513` collapse to one call each. |
| L5-T13 | `PUSH_ATTEMPTS`'s doc comment says it is how many times a publish rebuilds its commit on a new tip, but the loop rebuilds four times and pushes five. | drift; not load-bearing | `src/bench/history.rs:38-40` | block below | The comment names what the constant counts: push attempts, of which all but the first rebuild the commit. |
| L5-T14 | `Point::starts_line`'s doc comment names three reasons a record starts a line and leaves out the fourth the code sets it for: a commit a Verdict pinned. | drift; not load-bearing | `src/bench/history/suspects.rs:25-28` | block below | The comment lists the four cases `history.rs:356-364` computes, as the README at `history.rs:81-83` does. |
| L5-T15 | The second column of `HARNESS_BREAKS`, nine strings saying what each break changed, is read by nothing: the one shipping use maps it away and the four test uses read only the array's length | health; 9 lines, 0 readers | `src/bench/history/import.rs:29-66` | block below | `CLAUDE.md`, Code conventions: comments say what the code does and why. The smaller form is a `[&str; 9]` of commits with the description as a comment beside each, which the same reader reads at the same place |
| L5-T16 | `Worktree::restamp`, which gives records published at Harness version 1 the version in force when the history started, has had nothing to write since the first import: every record on `bench-history` at version 1 is an imported one, and every record the bench publishes carries version 10 | health; 18 lines plus the `restamped` field and its `Display` arm, 0 records reached | `src/bench/history/import.rs:325-342` | block below | ADR 0024: the Baselines before the history "are imported as records, each with the Harness version in force at its commit", a one-time migration; a shipping command carries only what a run can still do. The smaller form drops `restamp`, the `Publication::restamped` field (`history.rs:467`) and its `Display` arm (`history.rs:487-488`), and leaves `remove_imported` and `apply`, which a second import does exercise |
| L5-T17 | The two lines that resolve a repository's top level (`git rev-parse --show-toplevel`, then `Path::new(&root)`) are written four times across the bench | health; 4 sites, 8 lines | `src/bench/history/import.rs:92-93` | block below | one function in `provenance.rs` beside `git`, `toplevel(repository) -> anyhow::Result<PathBuf>`, called at the four sites; the other three sites fall in tooling, the Bench history (`history.rs:507,531`) and tooling, the bench (`provenance.rs:162`) |
| L5-T18 | `FrameRecorder` lists its ten per-frame counters four times (declaration, `new`, `begin`, `stats`) where one `FrameStats` field reset and copied would serve | health; about 40 lines | `src/stats/mod.rs:211-220` | block below | `CLAUDE.md`, How agents work here: deep modules behind small interfaces; a recorder holding `counters: FrameStats`, `begin` setting it to `FrameStats::default()` and `stats` cloning it, lists the counters once |
| L5-T19 | `output` is the fourth process runner of one shape in the crate, beside the verdicts' `gh`, the provenance's `run` and the cooker's `download` | health; 4 sites, about 20 lines | `src/preflight/mod.rs:276-282` | block below | one `run(command) -> anyhow::Result<String>` that reports a non-zero exit with its standard error, the preflight's callers taking `.ok()` on it |
| L5-T20 | Four of preflight's eight `pub` items (`Machine`, `Machine::classify`, `Check.name`, `Check.detail`) have no reader outside the crate | health; 4 declaration lines, 1 call site inside the crate | `src/preflight/mod.rs:22-77` | block below | The module depth axis: in a `pub` module a `pub` item nothing outside the crate reads is `pub(crate)`; `Machine` and `classify` `pub(crate)` for `src/render/coverage_tests.rs`, `name` and `detail` private behind `Display` |
| L5-T21 | `preflight::Verdict` names a check's four-way result with the glossary's term for the sweep's judgement of a Suspect, the third type of that name in the crate | health; 44 lines to rename (the 4 variants' 43 use lines and `src/bin/praxis.rs:362`), 3 types of one name | `src/preflight/mod.rs:61` | block below | `CONTEXT.md`, Verdict: "What the sweep found of one slower Suspect"; the code's names use the glossary's terms, so a check's result takes another name (its `Display` already calls it a verdict only in prose) |
| L5-T22 | The local `info`, bound to a `wgpu::AdapterInfo`, is an abbreviation of "information" used on eight lines of `run` | health; 8 lines in the file, 21 in the crate | `src/preflight/mod.rs:107` | block below | `CLAUDE.md`, Code conventions: identifiers are full words; `adapter` or `adapter_info` |
| L5-T23 | The `uploaded_bytes` doc comment names two of the frame's uploads where the renderer sums six | drift; load-bearing | `src/stats/mod.rs:20-21` | block below | the comment lists what `Renderer::upload` stages (`src/render/renderer.rs:514-551`) and what the dynamic TLAS's recording adds (`:605-613`): the camera and light uniforms, the light buffer, the bends, the Instances written, the slot count and the dynamic TLAS's instances |
| L5-T24 | The module doc promises every `FAIL` "the command that fixes it", and the `assets` check's `FAIL` carries only the error's text | drift; not load-bearing | `src/preflight/mod.rs:1-3` | block below | `docs/agents/workflow.md`, Session start: "A `FAIL` line blocks `scripts/verify.sh` and names its fix"; the `assets` error path names `praxis fetch` or the manifest to fix, or the module doc says a `FAIL` names its fix where one is known |
| L5-T25 | `parse_command` is the one shipping function the cognitive complexity probe names, at 27 over the lint's default threshold of 25, because one function holds the option extraction, eight scope checks and a 24-arm match | health; 2 over the threshold, 171 lines | `src/bin/praxis.rs:154-324` | block below | `CLAUDE.md`, Code conventions: `cargo clippy` passes; the probe's threshold of 25 is the lane's bar. One of the three groups named below moves into a helper and the function reads under 25 |
| L5-T26 | The usage text `parse_command` prints on bad arguments lists every command the `Command` enum has but `preflight` | drift; not load-bearing | `src/bin/praxis.rs:313-322` | block below | The usage names every command: `Command::Preflight` at `src/bin/praxis.rs:37`, parsed at `src/bin/praxis.rs:232`, which `scripts/sweep.sh:85,104` runs and `CLAUDE.md:108` documents as `praxis preflight` |
| L5-T27 | The usage text names `cook`, `fetch`, `capture`, `export-scene` and `difference` in every build, while those five `Command` variants and their match arms exist only under the `cook` feature, which CI builds without | drift; load-bearing in a `--no-default-features` build | `src/bin/praxis.rs:313-322` | block below | The usage text, or the five command names in it, gated as the variants are (`#[cfg(feature = "cook")]` at `src/bin/praxis.rs:38-61` and `233-264`), so a build without `cook` never tells a reader to run a command it then refuses with the same text |
| L5-T28 | `adapter_report`'s doc says the planned-feature lines are "followed by the Capability floor's limits", but the report prints three of the floor's four raised limits: `max_storage_buffers_per_shader_stage` is absent | drift; not load-bearing | `src/gpu/mod.rs:128-132` against `src/gpu/mod.rs:233-251` | block below | Either the report prints the fourth limit `floor_limits` raises (`src/gpu/mod.rs:306`), or the doc says which limits it prints. `tests/gpu_context.rs:46-49` pins only `max buffer size` and `acceleration structures` |
| L5-T29 | `DEVICE_MEMORY_BLOCK_BYTES`'s doc carries four measured pass timings (198, 52, 798 and 285 µs) that nothing in the tree records or re-measures | drift; not load-bearing | `src/gpu/mod.rs:9-13` | block below | `docs/audit/deviations.md`: a code comment holds no number; `docs/research/source-audit.md:170`: measured figures are checked against where they were measured, and `docs/performance.md` has no row for this one (`grep -n -i -e 'memory block' -e suballocat -e '16 MiB' docs/performance.md` finds only the Arena capacity at line 2937). The comment keeps the mechanism (a resource larger than a block gets its own) and the figures move to `docs/performance.md`, which the comment cites |
| L5-T30 | `BelowCapabilityFloor` and `LimitGap` are `pub` types with seven `pub` fields in the `pub mod gpu`, and nothing outside `src/gpu/mod.rs` names either or downcasts to them | health; 19 lines (2 types, 7 fields), 0 call sites | `src/gpu/mod.rs:361-379` | block below | The module depth axis: in a `pub` module, a `pub` item nothing outside the crate reads is wider than needed. `check_floor` is private and `request_device` returns the error inside `anyhow::Result`, so the two types and their fields can be private to the module (the `Display` impl is what every caller sees) |
| L5-T31 | The headless adapter request `first_backend(async \|wgpu_instance\| request_adapter(wgpu_instance, None).await).await?` is written out three times where one private function would serve | health; 3 call sites, 6 lines | `src/gpu/mod.rs:32-33`, `115-116`, `123-124` | block below | `CLAUDE.md`, How agents work here: deep modules behind small interfaces. One `async fn headless_adapter() -> anyhow::Result<wgpu::Adapter>` called from `headless`, `capability_report` and `adapter_info` |
| L5-T32 | The local `info` in `adapter_report` abbreviates "adapter info" | health; 1 definition, 5 uses | `src/gpu/mod.rs:134` | block below | `CLAUDE.md`, Code conventions: identifiers are full words; the same file spells it `adapter_info` at `src/gpu/mod.rs:431`. Rename to `adapter_info` |
| L5-T33 | The wrong-unit check spells the Blender command line a second time instead of reusing `blender`, with a line continuation collapsed into five spaces and a literal newline inside its `printf` format. | health; 5 lines | `scripts/gallery.sh:57-59` | block below | One function runs Blender and reports its status; the inverse check passes the status it expects, or reads the function's return under `if`, so the command line is spelled once (`CLAUDE.md`, "No throwaway code. Clean naming, structure"). |
| L5-T34 | `smoothstep`'s local `t` is a single letter outside the loop-index and coordinate forms. | health; 1 site | `scripts/cycles-reference/compare.py:79` | block below | `CLAUDE.md`, Code conventions: identifiers are full words; `fraction` (or `interpolant`), with `t` as the textbook symbol in a comment if wanted. |
| L5-T35 | The comment says the Goldens test "takes about half the step", a share nothing in the tree measures, records or checks. | drift; not load-bearing | `scripts/verify.sh:56-57` | block below | A comment states the reason without the figure ("the Goldens test is the step's longest, so it runs beside the rest"), or the figure sits where the verify command prints its step times (`:20`), which the comment could cite. |
| L5-T36 | The comment "Not a required check until two green weeks" names a trigger that has passed: the job has run since 2026-10-13, every push to `main` since 2026-10-26 ended green or cancelled, and branch protection still requires only the `check` job. | drift; not load-bearing | `.github/workflows/ci.yml:58` | block below | The comment says what is true now (the job is not a required check, and why, or until what), or goes once the job is made required. |
| L5-T37 | The ticket-line miss message attributes three forms to `docs/agents/workflow.md`, PRs, which names one of them ("Closes #N"); "Part k of #N" appears in no document under `docs/` or `.github/`. | drift; not load-bearing | `scripts/pr-lint.sh:30` | block below | The message cites where each form is written, or the PRs section and the template name the partial-PR form the lint accepts (the second is lane 7's placement; lane 7's docs table read the pair as "matches"). |
| L5-T38 | The provenance line says each crop comes "from `scripts/gallery.sh` with `PRAXIS_BLENDER` set", but `gallery.sh` runs only `compare.py`'s `ramp` and `compare` commands; the crops come from `compare.py crop`, which nothing in the tree runs. | drift; not load-bearing | `scripts/cycles-reference/DIFFERENCES.md:9-10` | block below | "Each crop is praxis \| Cycles \| difference, cut by `compare.py crop` from a gallery (`scripts/gallery.sh` with `PRAXIS_BLENDER` set), at 2x." |


#### L5-T01

The four configurations and what builds them:

```
src/bench/mod.rs:55  pub const REFERENCE    2560x1440, 248 warm-up, 1800 measured, aliasing
src/bench/mod.rs:67  pub const CI            640x360, 248 warm-up,  256 measured, no aliasing
src/bench/mod.rs:77  pub const COUNTERS     2560x1440,   8 warm-up,   64 measured, aliasing
src/bench/mod.rs:87  pub const CI_COUNTERS   640x360,   8 warm-up,   64 measured, no aliasing
```

- `grep -rnE 'Configuration::CI\b' src tests scripts .github docs` gives 4 sites: `src/bench/mod.rs:339` (`.width`, `.height` as "the Goldens' size" for `measure_aliasing`), `src/bin/praxis.rs:266` (the `--ci` parse arm), `src/bin/praxis.rs:603` (`.width`, `.height`: "Both of CI's configurations render at its size"), `src/bin/praxis.rs:892` (a parse test). No test runs it through `run` or `measure`; `mod.rs:1380` and `:1408` run `REFERENCE` and `COUNTERS`.
- `grep -rnE -- '--ci\b' src/bin scripts .github tests docs/agents` gives `bench --ci` only at `src/bin/praxis.rs:265`, `:318` (usage) and `:887` (parse test); `scripts/verify.sh:53` and `.github/workflows/ci.yml:181` are `compare --ci`, and `.github/workflows/ci.yml:178` is `praxis bench target/bench --ci-counters --no-publish` for every scene (no scene list). `scripts/sweep.sh:171,199` run `bench` with no flag (`REFERENCE`).
- `src/bench/history.rs:822-826` tags a lavapipe record off its Series unless its size and frame count are `Configuration::CI_COUNTERS`'s ("CI's configuration"), so a `bench --ci` record (256 frames) never joins a Series.
- `bench/baselines/llvmpipe-llvm-20-1-2-256-bits/*.json`: 9 scenes carry `"measured_frames": 256` (recorded under `CI`), `town.json` carries 64 (`CI_COUNTERS`). `compare` in `CompareMode::Ci` compares the counter frames when the lengths differ (`mod.rs:794-803`), so these Baselines need no re-record for the fix, though a re-record would make them match CI's artifact.
- The Goldens' size, 640 by 360, is spelled at `src/bench/mod.rs:68-69`, `tests/goldens.rs:47-48`, `tests/crowd.rs:11-12`, `tests/shadows.rs:22-23` and `src/bin/praxis.rs:13` (`DEFAULT_CAPTURE_SIZE: &str = "640x360"`): `grep -rnE '\b640\b|640x' src tests scripts` lists them beside seven hits that are other figures or prose (the history's README and test fixtures at `src/bench/history.rs:52,1667,1780`, the import's fixtures at `src/bench/history/import.rs:541,553`, a shader test's sample point and a Tick count). The two readers of `Configuration::CI` want that size, not CI's run.

The fix removes a constant and a parse arm (the arm and the usage string fall in the GPU context and the binary; the four other spellings fall in capture, stats and preflight for a home constant, and outside every module's scope for the tests) and renders nothing, so it can move no pixel. It changes no measurement's meaning, no Bench record key (`CI_COUNTERS` keeps its fields), and no step of the verify command, CI or the sweep, none of which runs `--ci`; it changes the flags the binary accepts.

#### L5-T02

`src/bench/mod.rs:63-66`:

```
/// CI's lavapipe renders on the CPU, so CI benches at the Goldens' size
/// and for fewer frames, and without `Aliasing`, whose render at three
/// times the Goldens' size it is too slow for: `compare --ci` reads
/// counters only.
pub const CI: Self = Self { ... measured_frames: 256 ... };
```

`src/bench/mod.rs:85-86`:

```
/// CI's size, rendering the counter frames and no more: CI's bench of a
/// scene too dense for lavapipe to render `CI`'s frames in its time.
pub const CI_COUNTERS: Self = Self { ... };
```

Against the code: `.github/workflows/ci.yml:170-178` benches every scene with `--ci-counters` ("Each scene renders its counter frames alone, which compare with a Baseline of any length at CI's size"), and `src/bench/history.rs:824` names `CI_COUNTERS` "CI's configuration" and keeps a lavapipe record of any other shape off its Series. Nothing runs `CI`'s 256 frames (L5-T01's grep). The comments describe the arrangement under which 9 of the 10 lavapipe Baselines were recorded (256 frames) and `town` alone took the counter frames; a reader who records a lavapipe Baseline by them runs `bench --ci` and publishes a record tagged "640x360 over 256 frames, not CI's configuration" (`history.rs:838`), which is the wrong command, so the drift is load-bearing. The fix is a comment, and no pixel, measurement or step moves.

#### L5-T03

`src/bench/mod.rs:400-403`:

```
pub enum Verdict {
    Warn,
    Fail,
}
```

`CONTEXT.md`, Verdict: "What the sweep found of one slower Suspect, kept on `bench-history`: held when a night's interleaved re-bench ... else cleared." That is `src/bench/history/verdicts.rs:81` (`pub(super) struct Verdict`). A third, `src/preflight/mod.rs:61` (`pub enum Verdict { Ok, Note, Warn, Fail }`), is a check's state. `grep -rnE '^\s*pub(\([a-z]+\))? (enum|struct) Verdict' src` gives the three. `grep -cE '\bVerdict\b' src/bench/mod.rs` gives 37; in `src/bin/praxis.rs`, 5 lines, of which `:362` spells `praxis::preflight::Verdict::Fail` in full because `:6` imports the bench's. The glossary's rule is that its terms are "the vocabulary of code, tickets, and reviews" (`CLAUDE.md`, Documents), so a term with one meaning there should name one concept in code. The smaller form: a `fails: bool` on the compare result (the `Display` at `mod.rs:413-421` already maps the two variants to `warn` and `FAIL`), which also removes the enum; `check_regression` (`mod.rs:851-855`) sets it from the two ratios. No pixel, measurement, record or step moves; the binary's `compare` output text is unchanged.

#### L5-T04

`src/bench/mod.rs:405-411`:

```
/// A metric that regressed against its Baseline or exceeds its Budget.
#[derive(Debug, PartialEq)]
pub struct Finding {
    pub metric: &'static str,
    pub verdict: Verdict,
    pub detail: String,
}
```

`CONTEXT.md`, Finding: "One defect at one location, filed by an Audit lane's reader in eight fields", _Avoid_ "Issue (a ticket), observation, note". `grep -nE '\bFinding\b' src/bench/mod.rs` gives 10 lines (6 outside tests: `:405-421`, `:647`, `:819`, `:858`, `:871`); `src/bin/praxis.rs:657-659` reads `finding.verdict` and prints it. The term entered the glossary at this commit with the audit skill, so the collision is new; it stands all the same, since this report and every triage will say "Finding" of something else. `Regression` is also a glossary term (a commit a Verdict pinned), so the new name is neither: `Breach` or `CompareResult`, chosen together with L5-T03's. A rename moves nothing but names.

#### L5-T05

`src/bench/mod.rs:129-135`:

```
/// A time warns above this ratio to its Baseline and fails above
/// `FAIL_RATIO`; a counter fails on any increase, as it is the same on every
/// run. Noise floor on the M4 Pro, 6 runs of each scene under the owner's
/// usual load: counters identical, GPU medians within 3.4%, CPU medians of
/// 31–42 µs within 8.1 µs.
const WARN_RATIO: f64 = 1.05;
const FAIL_RATIO: f64 = 1.10;
```

The figures "6 runs", "3.4%", "31–42 µs" and "8.1 µs" are a measurement nothing in the tree re-measures (`grep -rn '8.1 µs\|3.4%' src docs/performance.md bench` finds the comment alone). The CPU medians the Baselines hold today are `bench/baselines/apple-m4-pro/helmet.json` 186.1 µs and `town.json` 4,734.8 µs (`"cpu_frame_microseconds":{"median":...}`), so "31–42 µs" describes scenes of another era. The ratios and the counter rule stated beside the figures are what the code does (`check_regression`, `mod.rs:842-857`; `compare`, `mod.rs:643-653`), so the drift is not load-bearing. `docs/bench/index.html:171` restates the ratios and floors for the page's readers. The fix is a comment.

#### L5-T06

`src/bench/mod.rs:141-143`:

```
/// Likewise for a pass's GPU time: over three runs on the M4 Pro, helmet's
/// culling pass read 136 to 153 µs, every other pass within 3%.
const GPU_PASS_NOISE_FLOOR_MICROSECONDS: f64 = 20.0;
```

"three runs", "136 to 153 µs" and "within 3%" are a measurement nothing re-measures; the pass set has since split (`TIMED_PASSES`, `src/render/renderer.rs`, has no pass named culling pass alone; `culling_phase_one` and others appear in `report.rs:276`'s fixtures). The floor's rule (`check_regression` with `noise_floor` at `mod.rs:710-711`, `:723-732`) is stated correctly in the sentence before (`mod.rs:137-138`). Not load-bearing; the fix is a comment.

#### L5-T07

`src/bench/mod.rs:594-596`:

```
/// One line, `key=value` pairs after the scene name: medians and p95s of
/// the CPU and GPU frame, the gate and shadow-ray time, each pass's median,
/// then the counters.
impl std::fmt::Display for Measurements {
```

The line it documents (`mod.rs:600-624`) writes `cpu_us median= p95=`, then `dynamic_tlas_cpu_us=<median>` (`:602`, `:606`), then `gpu_us`, `gate_us`, `shadow_ray_us`, each pass's median, the counters, and the aliasing figures when measured (`:621-623`). The doc omits the dynamic TLAS CPU median and the aliasing figures. `src/bin/praxis.rs:462` prints this line per scene. Not load-bearing; the fix is a comment.

#### L5-T08

`src/bench/report.rs:10-16`:

```
/// `current`'s times and counters, its aliasing among them when it was
/// measured, as markdown tables, each against its `baseline`, when there is
/// one, and the Budgets. Times compare only with a
/// Baseline of as many frames on the same adapter, as `compare` judges them,
/// each pass with the Baseline's pass of its name; counters compare as
/// `compare` reads them. ...
```

`report.rs:25-27` keeps a times Baseline when `(&baseline.adapter, baseline.measured_frames) == (&current.adapter, current.measured_frames)`. `compare` at `src/bench/mod.rs:673-682` refuses a Baseline whose `(adapter, backend)` differs ("compare times on the Baseline's machine and backend"). So `report` would show time columns against the same adapter's Baseline on another backend, where `compare` errors; "as `compare` judges them" overstates the parity. The binary reads the Baseline from `baseline_directory(&current.adapter, &current.backend)` (`src/bin/praxis.rs:701-703`), so today no report meets such a Baseline, and the drift is not load-bearing. Whether `report` should also match `backend` is a one-line question for the fix; neither form moves a pixel or a measurement.

#### L5-T09

`src/lib.rs:19` declares `pub mod bench;`, so its `pub` items are the crate's public surface, read outside the library by `src/bin/praxis.rs` and `tests/`. Four items nothing outside the crate reads:

- `pub fn between` (`src/bench/mod.rs:278`): `grep -rn 'Aliasing::between' src tests` gives `mod.rs:351` and three tests in the same file (`:1501`, `:1518`, `:1530`).
- `pub const BASELINE_DIRECTORY` (`src/bench/mod.rs:32`): `grep -rn BASELINE_DIRECTORY src tests scripts` gives `mod.rs:113` and `src/preflight/mod.rs:10,202,203,710`, all in the crate; the binary reaches the directory through `baseline_directory` (`src/bin/praxis.rs:702`).
- `pub struct BenchStart` with four `pub` fields (`src/bench/provenance.rs:61-68`): `grep -nE 'start\.' src/bin/praxis.rs` gives none; the binary calls `BenchStart::read()` (`:450`) and passes `&start` (`:458`); the fields are read at `provenance.rs:101-116` only.
- `pub fn read` on `Checkout` (`src/bench/provenance.rs:159`): callers `provenance.rs:75` and its tests, `src/bench/history.rs:1261` (a test); none in the binary or `tests/`.

`Checkout`, `Provenance`, `Spread`, `Counters`, `CounterFrames` and `Aliasing` stay `pub` as field types of `Measurements`, which the binary reads and writes. The fix narrows four visibilities and compiles or does not; it moves no pixel, measurement, record or step.

#### L5-T10

```rust
// src/bench/history.rs:453-470
#[derive(Debug, Default, PartialEq)]
pub struct Publication {
    /// Records written under `records/`, so on a Series.
    pub on_series: usize,
    /// Records written under `tagged/`.
    pub tagged: usize,
    /// Records written under `experiments/`.
    pub in_experiment: usize,
    /// The Experiment published to; none for a publish to the Series.
    pub experiment: Option<String>,
    /// Records dropped for the same record, or one at least as quiet, under
    /// their key.
    pub dropped: usize,
    /// Records already on the branch given the current Harness version.
    pub restamped: usize,
    /// The commit pushed; none when nothing changed.
    pub commit: Option<String>,
}
```

`src/lib.rs:19` declares `pub mod bench`, and `src/bench/mod.rs:6` re-exports `Publication`, so the fields reach the binary. The binary never reads one:

- `grep -n "Publication\|publication\|\.on_series\|\.tagged\|\.in_experiment\|\.dropped\|\.restamped" src/bin/praxis.rs` finds `praxis.rs:475,511,515,522,523`, each a `writeln!(std::io::stdout(), "{publication}")` or the binding it prints: 0 field reads.
- `grep -rn "Publication {" src/bench/` finds the three constructors, `history.rs:537,934` and `import.rs:105`; `verdicts.rs:668` uses `Publication::default()`. `import` and `verdicts` are declared at `history.rs:5-7` as child modules, which read a parent's private fields.
- The tests that read the fields (`history.rs:1273,1275,1318,...`) sit in `mod tests` at `history.rs:1152`, a child module too.

The smaller form drops the seven `pub` keywords. The fix moves no pixel, changes no record's key or path, no measurement's meaning, and nothing the verify command, CI or the sweep does; the `Display` is unchanged.

#### L5-T11

```rust
// src/bench/history.rs:934-942
let mut publication = Publication {
    on_series: 0,
    tagged: 0,
    in_experiment: 0,
    experiment: None,
    dropped: 0,
    restamped: 0,
    commit: None,
};
```

`Publication` derives `Default` at `history.rs:453`, and `grep -rn "Publication::default()" src` finds one use, `verdicts.rs:668`. The literal above is `Default::default()` written out: `usize` defaults to 0 and `Option` to `None`. One line serves nine, and the type's two constructors then take one form. The fix moves no pixel and changes no record, measurement, key or workflow.

#### L5-T12

```rust
// src/bench/history.rs:507-518
let root = git(repository, &["rev-parse", "--show-toplevel"], None)?;
let repository = Path::new(&root);
let main = fetch_main(repository)?;
let main_source_hash = source_hash(repository, &main)?;
let release_build = release_build
    .map(|build| release_build_record(repository, results, build, &main, &main_source_hash))
    .transpose()?;
let records: Vec<BenchRecord> =
    bench_records(repository, results, &main, &main_source_hash, None)
        .chain(release_build)
        .collect();
push_records(repository, &records)

// src/bench/history.rs:530-540
check_experiment_name(name)?;
let root = git(repository, &["rev-parse", "--show-toplevel"], None)?;
let repository = Path::new(&root);
let main = fetch_main(repository)?;
let main_source_hash = source_hash(repository, &main)?;
let records: Vec<BenchRecord> =
    bench_records(repository, results, &main, &main_source_hash, Some(name)).collect();
Ok(Publication {
    experiment: Some(name.to_owned()),
    ..push_records(repository, &records)?
})
```

Lines 507-510 and 531-534 are the same four lines; `bench_records` at `history.rs:558-564` already takes `experiment: Option<&str>`, so the only difference the second function adds is the `release_build` the first takes and the `experiment` field it sets. The binary branches on the same `Option` at both call sites:

- `grep -n "praxis::bench::publish" src/bin/praxis.rs`: `praxis.rs:469-470` (`match experiment { Some(name) => publish_experiment(..), None => publish(.., None) }`) and `praxis.rs:512-513` (the same match with `release_build`).
- `praxis.rs:206-207` already refuses `--experiment` with `--release-build-seconds` ("a release build publishes beside main's results, never an Experiment's"), so a merged `publish(repository, results, release_build: Option<ReleaseBuild>, experiment: Option<&str>)` has no new case to reject; the `anyhow::ensure!` can move into it or stay in the parser.

The smaller form is one public function of about 20 lines in place of two (`history.rs:497-541`, 45 lines), and one call per site in the binary. `check_experiment_name` stays public: the parser calls it at `praxis.rs:180` to fail before the bench runs. The fix moves no pixel, changes no record, key, path or Harness version, and the Experiment records land where they do today; the test at `history.rs:1508-1598` is the proof.

#### L5-T13

```rust
// src/bench/history.rs:38-40
/// How many times a publish rebuilds its commit on the branch's new tip
/// after another publish pushed first.
const PUSH_ATTEMPTS: u32 = 5;
```

```rust
// src/bench/history.rs:647-680
let mut attempt = 1;
let mut tip = remote_tip(repository)?;
loop {
    let worktree = Worktree::add(repository, tip.as_deref())?;
    // ... write, commit, push ...
    let current = remote_tip(repository)?;
    if current == tip || attempt == PUSH_ATTEMPTS {
        return Err(error);
    }
    // ...
    tip = current;
    attempt += 1;
}
```

`attempt` starts at 1 and the loop returns the error when it reaches `PUSH_ATTEMPTS`, so the body runs five times: one first build and four rebuilds on a moved tip. The comment says the constant counts the rebuilds, which would make five rebuilds and six pushes. The name `PUSH_ATTEMPTS` matches the code; the comment does not. Not load-bearing: a reader tuning the retry count would be off by one in either direction, and no record or measurement changes. The fix is the comment.

#### L5-T14

```rust
// src/bench/history/suspects.rs:22-29
pub(super) struct Point {
    /// None when the record did not measure the metric.
    pub median: Option<f64>,
    /// Whether no record before it compares with it: its Harness version or
    /// scene content changed, or it is an imported Baseline, which accepted
    /// its level.
    pub starts_line: bool,
}
```

```rust
// src/bench/history.rs:356-364
let starts_line: Vec<bool> = rows
    .iter()
    .enumerate()
    .map(|(index, row)| {
        (index > 0 && !rows[index - 1].same_definition(row))
            || row.source == Some(import::IMPORTED_SOURCE)
            || line_starts.contains(row.source_hash)
    })
    .collect();
```

`line_starts` is `verdicts::pinned_sources(...)` at `history.rs:1025-1027`: the source hashes of commits a Verdict pinned. The README the same file writes has all four cases (`history.rs:81-83`: "A new Harness version or scene content, an imported Baseline, or a commit a Verdict pinned starts a line"), the glossary's Verdict entry has it ("which then starts its Series' line"), and the test at `history.rs:1839-1843` pins it ("A bisect pinned the step's commit: its line starts there"). The field's own comment, the one a reader of `suspects.rs` sees, stops at three. Not load-bearing: the README and the test carry the full rule. The fix is the comment.

#### L5-T15

The table and its one shipping use:

```
src/bench/history/import.rs:29  const HARNESS_BREAKS: [(&str, &str); 9] = [
src/bench/history/import.rs:30      (
src/bench/history/import.rs:31          "926fcc18",
src/bench/history/import.rs:32          "gpu_main_pass_microseconds spans the culling pass's start to the main pass's end",
src/bench/history/import.rs:33      ),
...
src/bench/history/import.rs:81          &HARNESS_BREAKS.map(|(commit, _)| commit),
```

The grep and its count: `grep -rn 'HARNESS_BREAKS' src tests scripts bench docs .github` finds 7 lines: the definition (`import.rs:29`), the shipping use that discards the string (`import.rs:81`), four test uses of `.len()` (`import.rs:483,507,515,605`), and one mention in `docs/guides/bench-history.html:285` naming the constant in a chapter mark. No line reads the tuple's second element: no `.1`, no destructuring that binds it, no `Display`, no test asserting a description. The doc comment above the table (`import.rs:25-28`) already says what the column is for: "and what changed".

The fix is a rename of the array's type and nine comments; it moves no pixel, raises no Harness version (the breaks' commits and their order are the key's input, the strings are not), changes no record's key or path, and changes nothing the verify command, CI or the sweep do. Proof: `cargo test` green, the import's `Publication` unchanged on the fixture in `history.rs:1689`.

#### L5-T16

The method and its one call:

```
src/bench/history/import.rs:103         let restamped = worktree.restamp(started_harness_version)?;
src/bench/history/import.rs:325     fn restamp(&self, version: u32) -> anyhow::Result<usize> {
src/bench/history/import.rs:326         let mut restamped = 0;
src/bench/history/import.rs:327         for (path, mut file) in self.measured_records()? {
...
src/bench/history/import.rs:332             if provenance.contains_key("source")
src/bench/history/import.rs:333                 || provenance.get("harness_version") != Some(&1.into())
src/bench/history/import.rs:334             {
src/bench/history/import.rs:335                 continue;
src/bench/history/import.rs:336             }
src/bench/history/import.rs:337             provenance.insert("harness_version".to_owned(), version.into());
```

What the branch holds, read from the local `origin/bench-history` ref (fetched 2026-11-02; the verifier can `git fetch` and re-run the same reads):

- `git grep -l '"source": "baseline-commit"' origin/bench-history -- records tagged | wc -l` gives 780: the import has run.
- `git grep -l '"harness_version": 1,\?$' origin/bench-history -- records tagged | wc -l` gives 135, and `git show` of each of the 135 finds `"source"` in every one: no record at version 1 lacks a source, so the `continue` at `import.rs:332-336` is taken for every record on the branch.
- No new record can arrive at version 1: `src/bench/provenance.rs:21` has `pub const HARNESS_VERSION: u32 = 10;` and `BenchStart::provenance` writes it (`provenance.rs:108`); the import writes `commit.harness_version` from the breaks table (`import.rs:281`), 1 to 10, always with a `source`.
- The branch holds no `verdicts/` directory yet (`git ls-tree --name-only origin/bench-history`: `README.md`, `experiments`, `records`, `series`, `tagged`).

So the second and every later run of `praxis history import` walks every record under `records/` and `tagged/` twice (once in `restamp`, once in `remove_imported`), parses each, and `restamp` writes nothing. The doc comment on `import_baselines` (`import.rs:71-76`) describes this correctly ("a second import under the same rules changes nothing"); the code is accurate, and unreachable with the branch as it is.

The one argument for keeping it: ADR 0024 says a bad publish "is undone with `git revert` on `bench-history`". A revert of the import's own commit would bring back the 135 pre-import records at version 1 without a source, and `restamp` would be needed again. That is a revert of the migration itself, not of a bad publish; the verifier may narrow the claim to "keep behind the revert case" if the owner wants the import to be re-runnable from any branch state.

The fix removes `restamp` (`import.rs:320-342`, 23 lines with its doc), the call (`import.rs:103`), the `restamped` field of `Publication` (`history.rs:467`, its zero at `history.rs:940`, its `Display` arm at `history.rs:487-488`) and the two test assertions on it (`history.rs:1695,1730`). It moves no pixel, raises no Harness version, changes no record's key, path or bytes, and changes nothing the verify command, CI or the sweep do; it changes what `praxis history import` prints only by dropping a `, re-stamped N` that is never printed today. Proof: the import fixture test in `history.rs:1660-1795` with its `restamped` assertions removed, and a dry import against the branch producing `(nothing changed)`.

#### L5-T17

The site in the import and its three twins:

```
src/bench/history/import.rs:92      let root = git(repository, &["rev-parse", "--show-toplevel"], None)?;
src/bench/history/import.rs:93      let repository = Path::new(&root);
src/bench/history.rs:507            let root = git(repository, &["rev-parse", "--show-toplevel"], None)?;
src/bench/history.rs:508            let repository = Path::new(&root);
src/bench/history.rs:531            let root = git(repository, &["rev-parse", "--show-toplevel"], None)?;
src/bench/history.rs:532            let repository = Path::new(&root);
src/bench/provenance.rs:162         let root = git(repository, &["rev-parse", "--show-toplevel"], None)?;
```

The grep and its count: `grep -rn 'show-toplevel' src` finds exactly these 4 lines, each followed by the same `Path::new(&root)` shadowing (the `provenance.rs` site continues on its own line 163). Four public entry points (`import_up_to`, `publish`, `publish_experiment`, `Checkout::read`) each normalise the caller's path the same way before passing it to `git`.

The smaller form: `pub(super) fn toplevel(repository: &Path) -> anyhow::Result<PathBuf>` in `provenance.rs` beside `git`, four one-line calls. It moves no pixel, raises no Harness version, changes no record's key or path, and changes nothing the verify command, CI or the sweep do. Proof: `cargo test` green (the fixture tests in `history.rs` call every one of the four entry points from a nested directory or the root).

#### L5-T18

`src/stats/mod.rs:211-220` declares the counters:

```rust
    pub(crate) draw_calls: u32,
    pub(crate) uploaded_bytes: u64,
    pub(crate) instances_written: u32,
    pub(crate) renderables_refused: u32,
    pub(crate) lights_overflowed: u32,
    pub(crate) static_instances_left_out: u32,
    pub(crate) moving_instances_left_out: u32,
    pub(crate) bends_left_out: u32,
    pub(crate) dropped_time: Duration,
    pub(crate) gpu_readbacks_skipped: u32,
```

The same ten names are listed again at `:232-241` (`new`, each set to zero), `:249-258` (`begin`, each set to zero again) and `:295-305` (`stats`, each copied into a `FrameStats`). `grep -c 'gpu_readbacks_skipped' src/stats/mod.rs` gives 7: the `FrameStats` field, the four listings, and two lines of `FrameStats`'s `Display` (`:157`, `:167`). `FrameStats` already declares each of these fields with the same name and type (`:19-38`, `:60-64`) and derives `Default`, so a `counters: FrameStats` field on the recorder, `self.counters = FrameStats::default()` in `begin` and `FrameStats { cpu_times, cpu_frame_time, heap_allocations, ..self.counters.clone() }` in `stats` lists them once; the writers (`src/state.rs:256,271-274`, `src/render/renderer.rs:495,551,557,694`, `src/render/readback.rs:235,242`) change from `recorder.x` to `recorder.counters.x`. `FrameStats::default()` allocates nothing (empty `Vec`), so the doc at `:202-203`, "without allocating once the first frames have sized its lists", still holds. The fix moves no pixel and changes no counter's meaning, so ADR 0024's Harness version stays; the proof is counters equal in `praxis compare` and `tests/frame_stats.rs` green.

#### L5-T19

`grep -rn 'Command::new(' src` gives 4 sites: `src/bench/provenance.rs:255` (`git_command`, run by `run` at `:262-285`), `src/bench/history/verdicts.rs:553` (`gh`), `src/preflight/mod.rs:277` (`output`) and `src/cook/fetch.rs:137` (`download`). The preflight's:

```rust
fn output(program: &str, arguments: &[&str]) -> Option<String> {
    let output = Command::new(program).args(arguments).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}
```

The verdicts' `gh` (`verdicts.rs:552-565`) is the same shape with `anyhow::Result`, a `current_dir` and the standard error in its message; the provenance's `run` (`provenance.rs:262-285`) adds a piped standard input and trims; the cooker's `download` (`fetch.rs:136-158`) is `curl` with fixed flags. Two of the four run `gh`: the preflight's `gh_json` at `:606-608` wraps `output("gh", ...)`, the verdicts' `gh` runs it with a repository. One runner returning `anyhow::Result<String>` with the program, arguments and standard error in its error, taking an optional working directory and standard input, serves all four; the preflight's callers, which want silence on a missing program, call `.ok()`. The git runner's `-C` argument and the `curl` flags stay at their call sites. About 20 lines go. The fix moves no pixel, measures nothing differently and changes no Bench record's key; the proof is the preflight's and the sweep's output unchanged, and the provenance and verdicts tests green. The bench and the import and the verdicts weighed the same shape from the `git` and `gh` sides (the provenance's `run` hardcodes `git` in both its messages, which is what kept `gh` from calling it); this row stands for all four sites.

#### L5-T20

`src/lib.rs:28` declares `pub mod preflight`, so its `pub` items reach `src/bin/praxis.rs` and `tests/`. `grep -rn 'preflight::' src/bin tests` gives 2 lines, `src/bin/praxis.rs:355` and `:362`: `praxis::preflight::run()` and, on one line, `check.verdict == praxis::preflight::Verdict::Fail`; the checks print through `Display` (`writeln!(stdout, "{check}")` at `:358`). `grep -rn 'Machine' src tests --include='*.rs' | grep -v '^src/preflight/'` gives 2 lines, both `src/render/coverage_tests.rs` (`:17` the import, `:662` `Machine::classify(...) == Machine::M4Pro`), inside the crate. `grep -rnE 'check\.(name|detail)' src/bin tests` gives 0. So of the eight `pub` items (`Machine` at `:22`, `classify` at `:36`, `Verdict` at `:61`, `Check` at `:73` with `name`, `verdict`, `detail`, `run` at `:102`), four have no reader outside the crate: `Machine` and `classify` serve one crate-internal test, `name` and `detail` are read only by the `Display` at `:89-99`. `pub(crate)` on `Machine` and `classify`, and private `name` and `detail`, narrow the module's interface to `run`, `Verdict` and `Check.verdict`, which is what the binary reads. No pixel, measurement or script output changes.

#### L5-T21

`grep -rn 'enum Verdict\|struct Verdict' src` gives 3: `src/bench/mod.rs:400` (`pub enum Verdict { Warn, Fail }`, a compare Finding's), `src/bench/history/verdicts.rs:81` (`pub(super) struct Verdict`, the sweep's) and `src/preflight/mod.rs:61`:

```rust
pub enum Verdict {
    Ok,
    /// Information, such as work in flight, that passes or fails nothing.
    Note,
    /// Worth acting on before a bench, but `verify` still passes.
    Warn,
    /// `scripts/verify.sh` fails until it is fixed.
    Fail,
}
```

`CONTEXT.md` defines Verdict once, as the sweep's: "What the sweep found of one slower Suspect, kept on `bench-history`: held ... else cleared", with "Confirmation, bisect result" to avoid. `CLAUDE.md`, Documents, makes the glossary's terms the vocabulary of code, and a glossary term borrowed for an unrelated four-way result (a check's) makes the word mean three things in one crate: `src/bin/praxis.rs:362` and `:657` compare a `preflight::Verdict` and a `bench::Verdict` within 300 lines of each other. A rename (`Outcome`, `Status`, or `Level`) at `:61` and its 4 variants' 43 use lines in the file plus `src/bin/praxis.rs:362` changes no behaviour, no output text (the `Display` at `:91-96` prints `ok`, `note`, `warn`, `FAIL`) and no pixel. The bench's `Verdict` at `src/bench/mod.rs:400` falls in tooling, the bench.

#### L5-T22

`src/preflight/mod.rs:107-125`:

```rust
        Ok(info) => {
            let machine = Machine::classify(&info.name, info.backend, info.device_type);
```

`info` stands for "information" and is bound to the `wgpu::AdapterInfo` that `crate::gpu::adapter_info()` returns; `grep -c '\binfo\b' src/preflight/mod.rs` gives 8 lines (`:107`, `:108`, `:114`, `:115`, `:116`, `:117`, `:122`, `:125`), on which the name appears 12 times. `CLAUDE.md`'s full-word rule allows the domain's own words and wgpu's own API names as the API spells them, which covers `AdapterInfo` and `get_info()` but not a local chosen here. `grep -rnE '\binfo\b' src --include='*.rs' | grep -vE 'tracing::info|info!|//' | wc -l` gives 21 lines in 5 files: 8 here, 4 in `src/gpu/mod.rs:134-211` (falls in tooling, the GPU context and the binary), 2 in `src/render/coverage_tests.rs:661-662` and 4 in `src/render/shader_tests.rs:475-517` (fall in render), 3 in `src/cook/mod.rs:359-365` (fall in cook, assets and scenes, where it is gltf's own `texture info` and so allowed). `adapter` or `adapter_info` serves; the rename changes no output.

#### L5-T23

`src/stats/mod.rs:20-22`:

```rust
    /// Bytes the frame's uploads wrote to GPU buffers: the Instances it wrote
    /// and the uniforms. Pools upload only while a Scene loads.
    pub uploaded_bytes: u64,
```

`grep -rn 'uploaded_bytes +=' src/render` gives 2 writers: `src/render/renderer.rs:551`, which sums the `uploads` list built at `:514-547` (the camera uniform and the light uniform at `:535-541`, `light_upload` (the light buffer, `:526-527`), `bend_upload` (the bend parameters, `:528-534`), `instance_uploads` (`:514-520`) and `slot_count_upload` (the culling pass's slot count, `:521-525`)), and `src/render/renderer.rs:612`, which adds the dynamic TLAS's upload returned by `record` (`src/render/acceleration_structures.rs:361`, "the bytes the build uploads"). The comment names the Instances and the uniforms and omits the light buffer, the bends, the slot count and the dynamic TLAS's instances. Load-bearing: `uploaded_bytes` is a Baseline counter `praxis compare` judges (`src/bench/mod.rs:203`, `:226`, `:239`), so a reader placing a counter Regression from this comment would look at Instances and uniforms and miss the lights, bends and TLAS that `town` and `crowd` upload every frame. The fix is the comment alone.

#### L5-T24

`src/preflight/mod.rs:1-3`:

```rust
//! `praxis preflight`: a session's starting state, one line per check, so
//! the session reads it rather than working it out. A check that would make
//! `scripts/verify.sh` fail is a `FAIL`, with the command that fixes it.
```

`grep -n 'Verdict::Fail' src/preflight/mod.rs` gives 5 lines: the `Display` arm at `:95` and 4 sites that build a `FAIL`. `:192` (goldens) names `PRAXIS_UPDATE_GOLDENS=1 cargo test --test goldens` or CI's artifact; `:228` (baselines) names `praxis bench <directory>`; `:129-130` (adapter) names `praxis capabilities`, which "diagnoses it"; `:240` (assets) is:

```rust
            Err(error) => return Check::new("assets", Verdict::Fail, format!("{error:#}")),
```

the error of `crate::cook::absent_fetched` (`src/cook/fetch.rs:32-43`, a manifest read that failed) with no command. The doc, and `docs/agents/workflow.md`'s session start ("A `FAIL` line blocks `scripts/verify.sh` and names its fix"), say every `FAIL` names its fix; one of four does not. Not load-bearing: the error text names the manifest path, and the reader who meets it has `praxis fetch` two lines above at `:247`. The fix is a fix in the message (`; check assets/manifest.json`) or a doc that says "where one is known".

#### L5-T25

The probe's line, from the lane's measurement stamp:

```
warning: the function has a cognitive complexity of (27/25)
   --> src/bin/praxis.rs:154:4
154 | fn parse_command(arguments: &[String]) -> anyhow::Result<Command> {
```

The function has three groups, any one of which moved into a helper takes it under 25:

1. The option extraction, `src/bin/praxis.rs:155-183`: `--crowd-form`, `--frame`, `--release-build-seconds`, `--experiment`, `--skip-aliasing`, `--no-publish`, each a `take_option` or `take_flag` with its own parse closure.
2. The eight scope checks, `src/bin/praxis.rs:185-216`: eight `anyhow::ensure!` calls saying which command each option belongs to. A `fn check_option_scope(command: Option<&str>, options: &Options) -> anyhow::Result<()>` holds them.
3. The match, `src/bin/praxis.rs:229-323`, 24 arms. The five `history` arms (`src/bin/praxis.rs:282-298`) group into `fn parse_history(rest: &[String]) -> anyhow::Result<Command>`, and the four `bench` arms (`src/bin/praxis.rs:265-276`, one per `Configuration` constant) into `fn parse_bench(output, rest, ...)`, which takes the configuration flag off the front of `rest` and benches the remainder as scenes.

The refactor moves no pixel: `parse_command` builds a `Command` and renders nothing. It changes no measurement's meaning and raises no Harness version, since `Configuration` and the bench are untouched. It can change what the verify command, CI and the sweep do only if a flag stops parsing, and the test module `src/bin/praxis.rs:839-1111` (15 tests) pins every form those callers use: `scripts/verify.sh:52-53` (`bench ... --counters --no-publish [--skip-aliasing]`, `compare ... --ci`), `.github/workflows/ci.yml:178-181,237` (`bench ... --ci-counters --no-publish`, `compare ... --ci`, `publish ... --release-build-seconds`), `scripts/sweep.sh:80-202` (`history unrecorded`, `preflight`, `history bisect`, `bench ... --no-publish <scene>`, `publish`, `history regressions`). Those tests passing is the fix ticket's proof.

#### L5-T26

The usage text, `src/bin/praxis.rs:313-322`:

```
"unexpected arguments {arguments:?}: run `praxis [<scene>]`, `capabilities`, \
 `cook`, `fetch`, `capture <scene> <png path> [<width>x<height>] [--frame <n>]`, \
 `export-scene <scene> <directory> [--frame <n>]`, \
 `difference <expected png> <actual png> <difference png>`, \
 `bench <output directory> [--ci | --counters | --ci-counters] [--skip-aliasing] [--no-publish | --experiment <name>] [<scene>...]`, \
 `publish <bench directory> [--release-build-seconds <seconds> | --experiment <name>]`, `history import`, `history unrecorded`, `history suspects`, `history bisect <night directory>`, `history regressions`, `compare <bench directory> [--ci]`, or `report <bench directory>`; `capture` and `bench` also take \
 `--crowd-form <proxies | refit | rebuild>`; scenes are {:?}"
```

`sed -n 313,322p src/bin/praxis.rs | grep -c preflight` gives 0. The enum has `Preflight` at `src/bin/praxis.rs:37` and the arm `[command] if command == "preflight" => Command::Preflight` at `src/bin/praxis.rs:232`. Every other variant is named: `Window` as `praxis [<scene>]`, and `Capabilities`, `Cook`, `Fetch`, `Capture`, `ExportScene`, `Difference`, `Bench`, `Publish`, `ImportBaselines`, `UnrecordedScenes`, `Suspects`, `Bisect`, `Regressions`, `Compare`, `Report` each by their command word. Not load-bearing: a reader is told too little, not told wrong; `preflight` takes no arguments, so no form can be mistyped, and `scripts/sweep.sh:85` and `CLAUDE.md:108` name it. The fix adds one word to the string and moves no pixel.

#### L5-T27

The same usage text at `src/bin/praxis.rs:313-322` names `cook`, `fetch`, `capture`, `export-scene` and `difference` with no `cfg`. Their variants are gated: `#[cfg(feature = "cook")]` before `Cook`, `Fetch`, `Capture`, `ExportScene` and `Difference` at `src/bin/praxis.rs:38,40,42,50,56`, and before their match arms at `src/bin/praxis.rs:233,235,237,245,253,259`. `Cargo.toml:37-41` makes `cook` default-on and says "a shipping build turns it off", and CI builds that configuration: `.github/workflows/ci.yml:116` (`cargo clippy --all-targets --no-default-features -- -D warnings`) and `.github/workflows/ci.yml:134` (`cargo test --no-default-features --test goldens`). In that build, `praxis cook` falls to the `_` arm and prints a usage that tells the reader to run `cook`. Load-bearing by the lane's definition (a reader acting on it runs the wrong command), though only in the build no session runs today. The fix gates the five names (a `cfg!(feature = "cook")` branch choosing between two strings, or a gated `const` for the five names concatenated into the text) and moves no pixel; the test `a_command_missing_its_arguments_is_refused_with_the_usage` at `src/bin/praxis.rs:873-876` pins the text's presence.

#### L5-T28

The doc at `src/gpu/mod.rs:128-132`:

```
/// The adapter, its backend, and whether the adapter supports each
/// feature the roadmap plans on, with the related features and limits.
/// One line per entry, followed by the Capability floor's limits, 4x MSAA on
/// the main pass's attachment formats, resolved on colour ones, and
/// anisotropic filtering.
```

The floor raises four limits at `src/gpu/mod.rs:297-309`: `max_buffer_size`, `max_binding_array_elements_per_shader_stage`, `max_acceleration_structures_per_shader_stage` and `max_storage_buffers_per_shader_stage`. The report prints `max buffer size` and `acceleration structures` in the block at `src/gpu/mod.rs:233-251`, and `max {} per stage` for binding arrays inside the `binding arrays` line at `src/gpu/mod.rs:151-158`. `grep -rn max_storage_buffers_per_shader_stage src` hits `src/gpu/mod.rs:306` (the floor), `:600` (the M4 Pro test literal) and `:769` (the floor-exact test): no `format!` prints it. Not load-bearing: `praxis capabilities` is a diagnostic, and an adapter short of that limit is named by `check_floor`'s error (`src/gpu/mod.rs:402-407`), so a reader diagnosing a floor failure is not misled, only one reading the report alone. The fix is one more `{:<24} max {} per stage` pair in the `format!` at `src/gpu/mod.rs:233-251`, or two words in the doc; neither moves a pixel. `tests/gpu_context.rs:30` caps the report at 30 lines; it has 14 today (two header lines at `src/gpu/mod.rs:209-212`, eight feature lines, four trailing lines at `:233-251`), so a line fits.

#### L5-T29

The doc at `src/gpu/mod.rs:9-13`:

```
/// ... In blocks of 128
/// to 256 MiB, on the RTX 2070 SUPER under Vulkan, a State whose targets
/// landed in a block an earlier State had filled drew pbr-sweep's depth
/// pre-pass in 198 µs instead of 52 and its lit pass in 798 µs instead of
/// 285.
```

`grep -rn DEVICE_MEMORY_BLOCK_BYTES src tests docs` hits `src/gpu/mod.rs` only (lines 19, 80, 81, 883). `grep -n -i -e 'memory block' -e suballocat -e '16 MiB' -e '198' docs/performance.md` finds no row for this measurement (line 2937 is the Arena capacity; the 198s at lines 894, 2693 and 3171 are other metrics). The test `render_targets_at_the_reference_size_get_memory_of_their_own` at `src/gpu/mod.rs:847-902` re-checks the mechanism (each target alone in its block) and not the timings, and skips on Metal, the reference machine's backend. So four timings sit in a shipping comment that nothing re-measures, the form the comment accuracy axis names. Not load-bearing: the constant's value rests on the mechanism, which is tested, and a reader who kept or changed the value would do so by re-measuring, not by these numbers. The fix moves the four figures into `docs/performance.md` with their run conditions and leaves the comment citing that page; it moves no pixel and changes no measurement.

#### L5-T30

```
pub struct BelowCapabilityFloor {
    pub missing_features: wgpu::Features,
    pub missing_downlevel_flags: wgpu::DownlevelFlags,
    pub missing_format_features: Vec<(wgpu::TextureFormat, wgpu::TextureFormatFeatureFlags)>,
    pub limits: Vec<LimitGap>,
}
pub struct LimitGap {
    pub name: &'static str,
    pub required: u64,
    pub adapter: u64,
}
```

(`src/gpu/mod.rs:364-379`.) `src/lib.rs:26` declares `pub mod gpu`, so these are the crate's public API. `grep -rn BelowCapabilityFloor src tests scripts docs` hits `src/gpu/mod.rs` alone (6 lines: the definition, `check_floor`'s construction at 328, the `Display` and `Error` impls at 381 and 416, and the test at 711); `grep -rn LimitGap src tests` likewise (4 lines, all in the file); `grep -rn downcast src tests` hits nothing. The only producer is the private `check_floor` (`src/gpu/mod.rs:314`), and the only consumer path is `request_device`'s `?` at `src/gpu/mod.rs:60-65`, which wraps it in `anyhow::Error`; the binary logs it through `Display` at `src/bin/praxis.rs:26`, as the ADR's consequence says ("the binary logs it with `tracing::error!`"). ADR 0021 asks for "one typed error naming each missing feature", which a private type with a `Display` impl satisfies. Smaller form: drop `pub` from both structs and their seven fields (the test module is in the file and keeps access). No pixel moves; no measurement changes.

#### L5-T31

The three sites:

```
src/gpu/mod.rs:32:        let adapter =
src/gpu/mod.rs:33:            first_backend(async |wgpu_instance| request_adapter(wgpu_instance, None).await).await?;
src/gpu/mod.rs:115:    let adapter =
src/gpu/mod.rs:116:        first_backend(async |wgpu_instance| request_adapter(wgpu_instance, None).await).await?;
src/gpu/mod.rs:123:    let adapter =
src/gpu/mod.rs:124:        first_backend(async |wgpu_instance| request_adapter(wgpu_instance, None).await).await?;
```

`grep -c 'request_adapter(wgpu_instance, None)' src/gpu/mod.rs` gives 3. `headless`, `capability_report` and `adapter_info` each want the adapter a headless context would use, and `capability_report`'s and `adapter_info`'s docs both say so ("the adapter a headless context would use", `src/gpu/mod.rs:111,120`). One private `async fn headless_adapter() -> anyhow::Result<wgpu::Adapter>` makes each a one-line body and gives the shared doc one home. Six lines become one function of four and three calls. No pixel moves, no measurement changes: the adapter chosen is the same.

#### L5-T32

```
src/gpu/mod.rs:134:    let info = adapter.get_info();
```

Read at `src/gpu/mod.rs:198` (`info.subgroup_min_size`), `:199` (`info.subgroup_max_size`) and `:211` (`info.name`, `info.device_type`, `info.backend`): 5 uses. `info` stands for "adapter info" (wgpu's `AdapterInfo`); the module's abbreviation list names `info`, and the same file spells the full word at `src/gpu/mod.rs:431` (`let adapter_info = adapter.get_info();`). `tests/conventions.rs` checks shading initials only, so this is not already caught. The rename touches one function and moves no pixel.

#### L5-T33

`scripts/gallery.sh:43-52` defines the one way the script runs Blender:

```
blender() {
  local log="$1" script="$2"
  shift 2
  if ! "$PRAXIS_BLENDER" -b --factory-startup --python-exit-code 1 \
    --python "scripts/cycles-reference/$script" -- "$@" >>"$log" 2>&1; then
    printf 'FAIL  %s  full log: %s\n' "$script" "$log"
    tail -n 20 "$log"
    exit 1
  fi
}
```

`scripts/gallery.sh:56-61` spells the same command line again, because `blender` exits on failure and this check needs the inverse (a run that must fail):

```
  # The check itself: point lights built in the wrong unit must fail it.
  if "$PRAXIS_BLENDER" -b --factory-startup --python-exit-code 1 --python     scripts/cycles-reference/calibrate.py -- --wrong-unit >>"$logs/calibrate.log" 2>&1; then
    printf 'FAIL  calibrate.py  passed point lights in the wrong unit: %s
' "$logs/calibrate.log"
    exit 1
  fi
```

`sed -n 57p scripts/gallery.sh | grep -o "python *scripts" | od -c` shows `p y t h o n` followed by five spaces then `s c r i p t s`: the `\` and newline of a continuation like `:46-47`'s collapsed into the spaces. `sed -n 57,59p scripts/gallery.sh | od -c` shows the `printf` format string `'FAIL  calibrate.py  passed point lights in the wrong unit: %s` closed by a real newline byte and then `'` on `:59`: the `\n` escape every other `printf` in the file writes (`:48`, `:63`, `:72`, `:79`, `:90`, `:95`, `:97`, `:102`, `:103`) became a literal line break inside the quotes. The output is the same, so the script works; the second spelling of the Blender command line (two sites: `:46-47` and `:57`) is the health defect, five lines where a status-returning helper would serve both callers (`blender` reporting its exit status, with the exit moved to the callers that want it, or an argument naming the status expected). The fix changes no render: the script only drives `praxis capture`, `praxis export-scene` and Blender; no pixel of a Golden, Baseline or Bench record depends on it, and `verify`, CI and the sweep do not run it.

#### L5-T34

`scripts/cycles-reference/compare.py:78-80`:

```
def smoothstep(edge0, edge1, x):
    t = np.clip((x - edge0) / (edge1 - edge0), 0.0, 1.0)
    return t * t * (3.0 - 2.0 * t)
```

`grep -nE "(^|[ (,])[a-z]{1,2} (=|in )" scripts/cycles-reference/*.py` finds, outside `x, y, z, w` quaternion components at `render.py:391,431`, this one single-letter local, used 3 times on `:80`. `t` is neither a loop index nor a coordinate; it is the interpolant. The parameter `x` is the WGSL builtin `smoothstep(low, high, x)`'s own spelling of its third argument and is weighed as the API's name (note 10); `edge0` and `edge1` are GLSL's names for the same arguments, words with a digit. `tests/conventions.rs` checks `.rs` and `.wgsl` under `src/` and `tests/` only (`:47`, `:81`), so no test reaches this file. The rename changes the port's text and not its values: `check_ramp` at `:142-148` holds the port to the GPU's own ramp within one 8-bit step on every run, so a wrong rename fails the gallery's first Blender step, and no pixel of praxis's own render depends on this file.

#### L5-T35

`scripts/verify.sh:56-57`:

```
# The Goldens test takes about half the step, so it runs beside every other
# test once both are built; its log is printed when it fails.
```

`grep -rn --include='*.md' --include='*.sh' "half the step\|Goldens test takes" .` finds the figure at this one site; `docs/agents/workflow.md:30` gives the whole command "about a minute once built" and no share per step. The script prints each step's time (`:20`, `printf 'ok    %-7s %4ss\n'`) but the `test` step is one line, so the Goldens' share inside it is printed nowhere, and nothing in `tests/`, `bench/` or `docs/` records or re-measures it. The axis names "a number in a shipping comment: a size, a timing, a ratio nothing in the tree re-measures"; this is a ratio of that kind. Not load-bearing: the structure the comment justifies (the Goldens test in the background at `:61`, the rest in the foreground at `:63`) is right whatever the share, and a reader acting on the comment runs no wrong command. The dated timings at `scripts/sweep.sh:59-61` are the same form and are not filed (note 5). A comment change can move no pixel and changes nothing the verify command does.

#### L5-T36

`.github/workflows/ci.yml:57-59`:

```
  software-goldens:
    # Not a required check until two green weeks.
    name: Goldens (software Vulkan)
```

`git log --format='%h %ad %s' --date=short -S'Not a required check until two green weeks' -- .github/workflows/ci.yml` places the comment and the job at 2026-10-13, 20 days before the frozen commit (2026-11-02). `gh api repos/gariasf/praxis/branches/main/protection/required_status_checks --jq '.contexts'` returns `["Check, Clippy & Format"]`: the `check` job alone is required; `software-goldens`, `shipping-goldens`, `software-bench` and the PR form check are not. `gh run list --workflow ci.yml --branch main --event push --limit 60 --json conclusion,createdAt` grouped by day: 2026-10-26 1 success 1 cancelled; 10-27 3 success 2 cancelled; 10-28 9 success 1 cancelled; 10-29 1 success; 10-30 11 success; 10-31 18 success 3 cancelled; 11-01 4 success; 11-02 6 success; no failure in the 60 runs, which reach back one week (the listing's limit, not the job's history). The comment's "why" cites a condition (fewer than two green weeks) that no longer holds, and the comment stays while the setting it describes did not change. Not load-bearing: a reader is misled about what gates a merge, writes no wrong code and runs no wrong command. Whether the job should now be required is the owner's (a process matter, lane 7's rules or the map); the Finding is the comment's stale reason. The comment's change moves no pixel and changes nothing CI runs.

#### L5-T37

`scripts/pr-lint.sh:29-31`:

```
if ! grep -Eq '^(Closes #[0-9]+|Part [0-9]+ of #[0-9]+|Ticket: n/a \(.+\))$' <<<"$body"; then
  miss 'body: no ticket line; one ticket per PR needs "Closes #N", "Part k of #N" or "Ticket: n/a (reason)" on a line of its own (workflow.md, PRs)'
fi
```

`docs/agents/workflow.md:28`, PRs: "One ticket per PR (`Closes #N`)". `.github/pull_request_template.md:1-3`: `Closes #` and "Docs, ADR, and process PRs: replace the line above with `Ticket: n/a (reason)`". `grep -rn --include='*.md' "Part [0-9k] of #\|Part k of" .` (excluding `target/`) finds nothing: no document names the partial-PR form. The form is live: `git log --format='%h %s' --grep='Part [0-9]* of #[0-9]'` on `main` finds 25 commits whose messages carry it, and `scripts/pr-lint-test.sh:102` tests it ("a partial PR"). So the message is the only text that states the three forms, and it cites `workflow.md`, PRs as their source. Lane 7's docs table (`docs/audit/m3.5/docs-drift.md:717`) read `workflow.md:28` against `pr-lint.sh:29-31` and wrote "matches", so the doc gap is not filed there; the Finding here is the usage string's attribution, on the comment accuracy axis. Not load-bearing: a reader using the form gets a pass; a reader of the doc alone does not learn it. If the triage prefers the doc to name the form, the fix is lane 7's. A message or doc change moves no pixel and changes nothing the lint rejects.

#### L5-T38

`scripts/cycles-reference/DIFFERENCES.md:9-10`:

```
Each crop is praxis | Cycles | difference, from `scripts/gallery.sh` with
`PRAXIS_BLENDER` set, at 2x.
```

`grep -nE "^\s*(blender|reference|capture) " scripts/gallery.sh` lists the script's Blender runs: `:55` `calibrate.py`, `:62` `compare.py ramp`, `:76` `render.py`, `:77` `compare.py compare`; the script writes whole images (`<scene>-frame-<n>-cycles.png`, `-cycles-difference.png`, `:66-79`) and no crop. The three-panel 2x crop is `compare.py`'s `crop` command, `scripts/cycles-reference/compare.py:8` and `:198-205` ("writes praxis | Cycles | difference of a region, each pixel doubled"). `grep -rn --include='*.md' --include='*.sh' --include='*.py' --include='*.yml' --include='*.html' "compare.py crop\|-- crop\|\"crop\"" .` finds `crop` called nowhere but defined at `compare.py:198`; the seven images under `differences/` are its products, cut by hand from a gallery's images. The sentence names the gallery as the source of the crops; the gallery is the source of the images the crops are cut from. Not load-bearing: a reader redoing a crop finds the command in `compare.py`'s docstring, one file away, and no measurement or render depends on the sentence. A one-clause change to the entry; no code moves.

### Notes for the verifier

Not filed, since each is a near-miss under the lane's axes, an allowed form, or belongs to another lane; recorded so the next audit does not re-find them. By reader: A the bench, B the Bench history, C the import and the verdicts, D capture, stats and preflight, E the GPU context and the binary, F the scripts and the workflows.


**A.**

1. Near-miss, `compare`'s shape (`src/bench/mod.rs:634-755`, 122 lines): six `check_budget` calls of five to six lines each (`:654-659`, `:691-696`, `:716-721`, `:734-739`, `:740-745`, `:747-752`, about 30 lines) where a `[(metric, current, budget)]` loop like the one at `:724-733` would take ten; and two `ensure!` at `:665-682` that `ensure_comparable` cannot hold because `report` shares it and allows them. Not filed: the helper exists and the calls are its table; a ticket for L5-T03 or L5-T04 may fold it in.
2. Near-miss, `measure` (`src/bench/mod.rs:495-592`): five `spread(measured....map(|...| microseconds(...)).collect())` builds (`:542-580`) that one closure over a field selector would make five lines. Not filed for the same reason.
3. `spread` (`src/bench/mod.rs:879-885`) clones its values and `percentile` sorts the vector twice; a cost nit of lane 3's kind, trivial at 1,800 values; not filed.
4. `Display for Measurements` (`src/bench/mod.rs:597-627`) beside `report` (`src/bench/report.rs:17-167`) are two renderings of the same figures, weighed as asked: the first is the bench's one-line stdout per scene (`src/bin/praxis.rs:462`), the second the markdown tables PRs paste. Different consumers, different forms; not filed (L5-T07 is the first's doc).
5. Written and read by no code: `Provenance.driver`, `.driver_info`, `.gpu_load_percent`, `.other_processes`, `.started_unix_seconds` (`src/bench/provenance.rs:39-54`) and `Checkout.branch` (`:152`). `StoredProvenance` (`src/bench/history.rs:254-265`) reads `commit`, `source_hash`, `harness_version`, `source`, `scene_content_hash`, `gpu_frame_p95_over_median`; `record` (`history.rs:693-717`) reads `checkout`, `quiet`, `gpu_frame_p95_over_median`; `docs/bench/index.html:340-345` reads the same five as `StoredProvenance`. They are a Bench record's data, which ADR 0024 means to be queried "with `git` and `jq`", so weighed as record content and not filed as vestigial.
6. Falls in lane 1 and the Bench history: `quiet` is false on a battery or display warning (`is_quiet`, `provenance.rs:130-140`, through `BenchStart.machine_warnings`, `:67`), but the record carries no field saying which warning, and the only tag is "machine not quiet" (`history.rs:830`); ADR 0024 says a kept record is "tagged with its reasons".
7. Unverifiable, not filed: `REFERENCE`'s doc "With the warm-up, as many frames as one run can time on the GPU" (`src/bench/mod.rs:53-54`): no constant in the tree bounds 2,048 frames (`src/render/readback.rs:30-53` sizes a ring of a few slots per frame in flight, not a run), so nothing pins the claim.
8. Checked and holding: `// ADR 0022 judges Budgets at 60 Hz.` (`src/bench/mod.rs:453`) against ADR 0022's line 23, "Budgets stay judged at 60 Hz"; `HARNESS_VERSION` 10 (`provenance.rs:21`) against `HARNESS_BREAKS`'s 9 entries (`src/bench/history/import.rs:29`) and the test at `import.rs:605`; `MEASURED_PATHS` 5 plus `UNMEASURED_PATHS` 12 against the 17 paths of `git ls-tree --name-only HEAD`, with the test ADR 0024 promises at `provenance.rs:311`; `QUIET_P95_OVER_MEDIAN` 1.06 against the ADR's "about 1.06"; `baseline_directory`'s doc against its test (`mod.rs:1476-1491`) and `bench/baselines/`'s four directory names; `report.rs:1-2`'s `docs/performance.md`, which exists and `docs/agents/workflow.md:55` names; the `khronos-dfs-srgb-eotf` key at `mod.rs:284` exists in `docs/references.toml:224`.
9. Allowed abbreviations weighed: `stats` as a lambda binding (`mod.rs:214-229`, `:504-556`) is the crate's own module name and the glossary's Frame stats; `rmse`, `p95`, `p99` are statistics' own terms, and `aliasing_rmse` is a Baseline and Bench record field, so a rename would change the record form; `tlas`, `blas` are ray tracing's; `driver_info` is wgpu's `AdapterInfo` field; `stdin` is std's; `cpu`, `gpu`, `ci`, `json`, `git`, `gh` are on the allowed list. `tests/conventions.rs` covers no abbreviation in these files, so nothing here re-finds it.
10. Near-miss in the tests: `BUDGETS` is spelled identically at `src/bench/mod.rs:995-1002` and `src/bench/report.rs:223-230` (8 lines), and `report.rs:236-268` builds its own `Measurements` fixture while `mod.rs:931` exposes `pub(super) fn baseline()`, which `src/bench/history/verdicts.rs:897` already reuses. Eight lines; not filed alone.
11. Falls in lane 7: `docs/bench/index.html:171` restates `WARN_RATIO`, `FAIL_RATIO` and the two noise floors as literal text ("past +5%", "past +10% when also past the noise floor (10 µs on the CPU, 20 µs on the GPU)") beside the constants at `src/bench/mod.rs:134-143`: text held twice, a change to one leaves the other.
12. The module split's description of "`history.rs`'s own git runner" does not hold at this commit: `src/bench/history.rs:15` imports `provenance::git` and has no `Command::new`. The three runners are `provenance.rs:262`, `verdicts.rs:552` and `preflight/mod.rs:276` (L5-T19).
13. For L5-T01's fix, the lavapipe Baselines need no re-record: 9 carry 256 frames and `town` 64, and `compare --ci` compares counter frames across lengths (`mod.rs:794-803`); `praxis compare`'s hint for a missing lavapipe Baseline already says "copy <scene>.json from CI's bench-results artifact" (`src/bin/praxis.rs:605-608`), which yields 64 frames.
14. Falls in lane 6 or lane 1: `percentile` on an empty vector panics at `values[rank.clamp(1, 0) - 1]` (`src/bench/mod.rs:892`, `clamp` with `min > max`); unreachable from the four constants since `run` ensures at least 72 frames (`:435-438`) and every constant measures 64 or more, but no test pins it.
15. `Configuration`'s `warm_up_frames` doc, "the counter frames among them are counted" (`mod.rs:44-46`), holds for `REFERENCE` and `CI` (warm-up 248 covers frames 8 to 72) and reads loosely for `COUNTERS` and `CI_COUNTERS`, whose warm-up is 8 and whose counter frames are their measured frames; weighed accurate enough, not filed.
16. The usage string at `src/bin/praxis.rs:318` lists `--ci`; if L5-T01 is fixed it goes with the arm, in the GPU context and the binary.


**B.**

1. The in-memory and stored twins (`BenchRecord` at `history.rs:124-141`, `StoredRecord` at `history.rs:225-250`): weighed and not filed. `BenchRecord` carries the serialized `Measurements` as a JSON map plus the publish's own decisions (`tags`, `publishes_once`, `experiment`); `StoredRecord` reads every record on the branch, imported Baselines included, which lack the fields `Measurements` requires (`history.rs:221-223`, `232-243` are all `Option`), and keeps the counters as a flattened map because a record holds only the counters its bench counted. One type with serde attributes would have to make every `Measurements` timing optional, which `compare` and the Baselines' reader do not want. The split follows the ADR's import clause, not a repeated shape.
2. `ReleaseBuildRecord` (`history.rs:300-306`), `StoredReleaseBuild` (`history.rs:310-315`) and `ReleaseBuildRow` (`history.rs:319-325`): three shapes of one payload around `ReleaseBuild`. Merging the first two into one owned type that both serializes and deserializes would save about 8 lines and cost two `String` clones per publish and a `committed_unix_seconds` field that `BenchRecord::new` inserts into the map after serialization (`history.rs:163-166`), so it would need `skip_serializing`. Weighed and not filed: the saving is below the cost in attributes.
3. `MedianOf` (`history.rs:386`): a type alias with one use, the type of `JUDGED_TIMINGS` at `history.rs:390`. One line that names the closure's role; not filed.
4. `SeriesRecord` (`history.rs:328-338`): two implementors and one generic site (`series_lines`), so not an abstraction with one use. Its `rows` takes `line_starts`, which `StoredReleaseBuild::rows` ignores (`history.rs:406`); a parameter one of two implementors ignores is the trait's shape, not flexibility nothing exercises.
5. The Suspect rule in prose sits in four places: the `README` constant (`history.rs:77-83`), the bench page's legend (`docs/bench/index.html:169`), the field guide `docs/guides/bench-history.html`, and `CONTEXT.md`'s Suspect entry. The test at `history.rs:1601-1620` pins the README's configurations and quiet thresholds against `Configuration`, `BUSY_GPU_PERCENT` and `QUIET_P95_OVER_MEDIAN`, but not the README's "8 records", "4 median absolute deviations" or "3%" against `suspects.rs:7-19`, and nothing pins the page's legend. They agree at this commit, so no drift is filed; that a test misses the seam is lane 6's, and the page's and guide's copies are lane 7's.
6. `BenchRecord::path`'s comment (`history.rs:205-207`) says the Harness version is not in the path "as it is defined under `src/`, so a source hash names one", while the ADR's key clause lists the Harness version as a key component and `import.rs:319-341` restamps records published at version 1. Whether a source hash names one Harness version after a restamp is a correctness question: falls in lane 1.
7. `history.rs` has no git runner of its own: `fetch_main`, `push`, `remote_tip` and `committed_unix_seconds` (`history.rs:624-876`) call `provenance::git` (`provenance.rs:246`). The "three process runners of one shape" are provenance's `git`, `verdicts.rs:552`'s `gh` and `preflight/mod.rs`'s `gh_json`; the first falls in tooling, the bench, the other two in tooling, the import and the verdicts, and tooling, capture, stats and preflight.
8. `import.rs:312` writes `adapter.starts_with("llvmpipe")` where `is_lavapipe` and `LAVAPIPE_ADAPTER_PREFIX` exist (`history.rs:36,286-288`; `grep -rn '"llvmpipe' src --include='*.rs'` outside tests: `import.rs:312`, `preflight/mod.rs:39`). A repeated literal: falls in tooling, the import and the verdicts, and tooling, capture, stats and preflight.
9. `Worktree` (`history.rs:880-1098`) beside `scripts/sweep.sh:123-143`'s trees under `target/sweep/trees`: the one is a temporary detached worktree of `bench-history` that a publish writes and removes; the other a worktree of one of `main`'s commits that holds a release binary for the night's bisect and is kept for a later night. Neither wraps the other. `import.rs:319-383`'s `impl Worktree` adds three methods the import alone calls (`import.rs:103` and inside the impl), reading only `directory`.
10. The serde alias comment at `history.rs:234-236` ("Named `gpu_main_pass_microseconds` until the bench renamed it, the same span under a new name"): whether the old field measured the same span is a question for the import's `HARNESS_BREAKS` and lane 1, not a comment this section can judge against code beside it.
11. Naming forms weighed and allowed: `json` as the bytes of a JSON file (`history.rs:962,1047`), `p95_over_median`, `ci_configuration` (`history.rs:1852`), `_frames` as serde's renamed ignored field (`history.rs:270`), `%ct` as git's format string (`history.rs:851`). `metal` and `lavapipe` as test path variables (`history.rs:1660-1661`) are words.
12. `Run` (`history.rs:794-801`) is built at `history.rs:708` and `import.rs:294` and destructured once at `history.rs:813-820`: two construction sites, not a parameter object with one use.
13. `BenchRecord::p95_over_median` and `StoredRecord::p95_over_median` (`history.rs:216-218,277-281`) are two three-line methods of one shape on two types; a shared helper would be as long. Not filed.
14. The test module's helpers each have two or more uses: `Fixture::commits` 6, `Fixture::read` and `Fixture::files` many, `commit` many, `result` many, `lavapipe_result` 3, `LAVAPIPE_KEY` 3, `LAVAPIPE` 2. No test pins nothing: each asserts on the branch's files, Series rows or `Publication` counts.
15. Fix tickets for L5-T10 to L5-T12 carry the refactor licence with this proof: no Golden changes (the Bench history touches no render), counters equal, no record key, path or Harness version changes, the `Display` of `Publication` unchanged, the history tests at `history.rs:1151-1942` green. L5-T13 and L5-T14 are comment edits.


**C.**

1. **Visibility one module too wide, not filed.** Of the 16 `pub(super)` items in `verdicts.rs`, 14 have no reader outside the file: `grep -rn` over the worktree for `Commit {`, `State::`, `Pinned {`, `Measured {`, `Ends {`, `Verdict::of`, `Row {`, `Run {`, `unjudged`, `step(`, `Issue {`, `fn issue\|issue(`, `Filing`, `filing(` finds each only in `verdicts.rs` (its shipping code and its tests), while `VERDICTS` and `pinned_sources` are read at `history.rs:1026`. `verdicts` and `history` are private modules (`history.rs:5-7`, `bench/mod.rs:1`), so by the lane's rule the spelling alone is not a Finding; a fix ticket on L5-T19 or L5-T16 could tighten them in passing. `Verdict::of` is `pub fn` on a `pub(super)` type, which reaches no further than the type.
2. **Three types named `Verdict`.** `grep -rn 'enum Verdict\|struct Verdict' src` finds `src/bench/mod.rs:400` (`pub enum Verdict`, a compare's outcome per metric), `src/preflight/mod.rs:61` (`pub enum Verdict`, a check's line class) and `src/bench/history/verdicts.rs:81` (the sweep's). `CONTEXT.md` defines Verdict as the sweep's: "What the sweep found of one slower Suspect". The two other uses of the glossary word fall in tooling, the bench, and tooling, capture, stats and preflight; the one here matches the glossary. Likewise two types named `Run` one module apart: `history.rs:794` (`Run<'run>`, what a bench run measured on) and `verdicts.rs:159` (`Run`, a night's run: commit and median); neither is imported into the other's file, so there is no clash, but a reader of `import.rs:294` (`Some(Run { adapter, ... })`) and `verdicts.rs:641` (`Run { commit, median }`) meets two `Run`s. Not an abbreviation, so not on the naming axis; recorded for the next audit.
3. **`ImportedProvenance` as a third provenance shape, weighed and not filed.** `provenance.rs:34` `Provenance` (the bench's writer, 12 fields with `Checkout` flattened), `history.rs:259` `StoredProvenance` (the reader of both writers, 6 fields, three of them `Option`) and `import.rs:246` `ImportedProvenance` (the import's writer, 6 fields, with `baseline` that the reader ignores and without `scene_content_hash`). Two writers of different fields and one loose reader is the smallest arrangement that keeps the stored bytes as they are; deriving `Serialize` on `StoredProvenance` and writing it from the import would add `"scene_content_hash": null` to 780 records, which changes record bytes, so a fix would not carry the unchanged-behaviour proof. `Baseline` (`import.rs:230`) is likewise a loose reader of Baselines older than `Measurements` can parse (its `backend` and `gpu_frame_microseconds` are `Option`), and `BaselineCommit` (`import.rs:113`) is built once (`import.rs:168`) and read by `imported`; neither is a twin of `BenchRecord`.
4. **`import_up_to`'s parameters exist for tests.** Its one shipping caller, `import_baselines` (`import.rs:78-82`), passes the two constants; its three other callers are the fixture tests at `history.rs:1689,1723,1786`, which need a `history_started` and a break of their own. A test seam, not flexibility nothing exercises.
5. **`Suspect` and `Verdict` share five fields.** `Verdict::of` (`verdicts.rs:105-118`) copies `scene`, `metric`, `shift`, `suspect` and `predecessor` from a `Suspect`; `Suspect` adds `committed_unix_seconds` for the sort at `verdicts.rs:355` and is not serialised, `Verdict` is the stored file. Embedding one in the other would change the Verdict file's shape; nothing is on the branch yet (no `verdicts/` directory), so a fix now would carry the proof cheaply, but the gain is five lines. Not filed.
6. **`Row` beside `SeriesRow`.** `verdicts.rs:148` `Row` reads four fields of a Series row; `history.rs:424` `SeriesRow<'record>` writes nineteen, borrowing from a `StoredRecord`. A borrowing writer cannot be the owned reader; the pair is by need. Not filed.
7. **`HISTORY_STARTED`'s doc against the inclusive range, weighed and not filed.** `import.rs:20-22` says "every Baseline committed before it is imported", and `import_up_to` (`:85`) and `baseline_records` (`:120-121`) say "before `history_started`"; the code's `rev-list --reverse --first-parent history_started` (`:143`) and `log --first-parent ... history_started -- bench/baselines` (`:149-156`) include the commit itself. `git show --stat 3c6b9d92 -- bench/baselines` is empty: that commit changed no Baseline, so on `main` the two readings import the same records and no reader is misled today. A later `HISTORY_STARTED` that did change a Baseline would import it beside the record its own bench published, and `apply` would keep the quieter. Recorded so the next audit does not re-weigh it.
8. **`next_commit`'s one-change case.** `verdicts.rs:292-296` says that with no other source between the ends "the commit that changed the source to the bad end's is pinned"; with `changes.len() == 1` (every commit of one source, so the ends share a source) the code pins `changes[0]`, the good end (`verdicts.rs:307-310`). The ends are two distinct Series rows, whose paths are their source hashes (`history.rs:216-221`), and a moved end is always the first commit of a source run, so the case does not arise from `bisect`; a correctness question for lane 1 if it ever does, not a comment defect.
9. **`night/all` and `night_runs`.** `scripts/sweep.sh:178-183` copies a night's results into `$night/all/` as `<run>-<scene>.json` and publishes that directory; `night_runs` (`verdicts.rs:624-648`) reads every directory under the night for `<scene>.json`, so `all/` holds no file it would read and the runs are not double-counted. Checked because the doc at `verdicts.rs:622-623` says "in `night`'s directories" without excluding one. Clean.
10. **`SMALLEST_HELD_SHIFT` beside `SMALLEST_SHIFT_FRACTION`.** `verdicts.rs:23` and `suspects.rs:19` both hold `0.03`; `grep -rn SMALLEST_SHIFT_FRACTION src docs` finds it at `suspects.rs:19,43,102` and in the guide's JavaScript twin at `docs/guides/bench-history.html:486,497`. `CONTEXT.md` names the two 3% floors separately (Suspect: "the larger of 4 median absolute deviations and 3%"; Verdict: "half its shift and 3%"), so they are two values of one magnitude, not one constant; the comparison is tooling, the Bench history's item and is left to it.
11. **`BENCH_PAGE`'s link lands.** `verdicts.rs:711` writes `?machine=<adapter key>&scene=<scene>`; `docs/bench/index.html:1410-1415` reads `parameters.get("machine")` and `parameters.get("scene")`. Clean. `TRIAGE_LABEL` is `needs-triage` (`verdicts.rs:444`), the string `docs/agents/triage-labels.md` maps the role to. Clean.
12. **ADR 0024's "61 commits", falls in lane 7.** `git rev-list --first-parent 3c6b9d92 -- bench/baselines | wc -l` gives 64 first-parent commits that touched `bench/baselines/` before the history started; the ADR's Consequences say 61. The import also skips a commit whose Baselines were only renamed or given fields (`import.rs:189-202`), so the number of commits imported is a third figure. A doc against the code, not a comment in these files.
13. **Test helpers each used by one test, not filed.** In `verdicts.rs`'s tests `next` (1 test, 6 calls), `row_commit` (1 test, 2 calls) and `row` (1 test, 5 calls) serve one test each, and still shorten it; `night`, `runs`, `commits` and `regression` serve several. In `import.rs`'s tests `commit` serves 5 tests and `stored` 3. No helper is unused.
14. **Parsing a Baseline twice.** `import.rs:260-261` parses the same `json` into a `Baseline` and into a `serde_json::Value`; `serde_json::from_value` on a clone of the object would parse once. Two lines; recorded, not filed.
15. **Falls in lane 1 (none found).** No correctness disagreement with ADR 0024's Suspect, bisect or Regression clauses was seen in these two files: `holds` is half the marked shift and 3% (`verdicts.rs:795-797`, the glossary's Verdict); `step` benches the ends interleaved `END_RUNS` times before judging (`:231-238`); `file_regressions` opens one issue per scene and pinned commit and records it (`:451-498`); `unjudged` lists slower Suspects only (`:190`).
16. **Outside every module's scope.** Nothing seen in `src/lib.rs`, `src/state.rs` or `tests/*.rs` while counting uses; `tests/` references no item of these two files.


**D.**

1. `src/preflight/mod.rs:612-618`, `live_milestone`, parses `M<n>` as a `u32` after stripping `M`, so `M3.5` parses as nothing and `praxis preflight` prints `no open milestone named M<n>` while the live milestone is `M3.5`. The doc comment at `:610-611` ("the open one with the lowest number in its `M<n>` title") and the message at `:568` describe the code as written, so nothing here is drift; the disagreement with the live milestone falls in lane 1.
2. `src/capture/mod.rs:282-288`, `FrameWait::Spin`'s doc, carries "about 30% slower", a figure nothing in the tree re-measures. Weighed and not filed: it is the reason the variant exists, is cited with its registry key `apple-tech-talk-110147-busy-waits`, and the bench's choice at `src/bench/mod.rs:439-442` matches the doc's "only the bench spins, on a hardware adapter".
3. `src/capture/mod.rs:27-33`, `GOLDEN_TOLERANCE`'s doc, carries "20 captures of each of the first two Golden scenes, `pbr-sweep` and `uv-grid`" and "the noise is 0". Weighed and not filed: both scenes exist (`src/scenes/mod.rs:31,33`), the figure is the measurement's provenance rather than a parameter, and every Goldens run (`scripts/verify.sh:61`, `.github/workflows/ci.yml:134`) re-checks that the tolerance holds on three adapters.
4. The capture's three layers: `capture` has callers in `src/bin/praxis.rs:764`, `src/bench/mod.rs:341` and seven files under `tests/`; `capture_frames` in `src/bin/praxis.rs:751` and `tests/goldens.rs:317`; `render_frames` has one caller outside the file (`src/bench/mod.rs:445`, which alone reads the returned content hash and chooses `FrameWait`) and the two wrappers inside. The two wrappers return different things (one the last frame's pixels and every frame's stats, the other several frames' pixels), so neither is the other's special case; not filed.
5. `ImageDifference` is re-exported at `src/lib.rs:40` and named nowhere outside `src/capture/mod.rs` (`grep -rn ImageDifference src tests` gives the definition, the re-export and the test); callers read its fields from `compare_images`'s return value. A `pub` function's return type must be `pub`, so the re-export is required; not filed.
6. `src/render/renderer.rs:836` declares a test-module `const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;` that duplicates `src/capture/mod.rs:11`, which `src/render/tonemapping_pass.rs:181` imports instead. Falls in render.
7. `tests/frame_stats.rs:18` declares `const FRAME_COUNT: usize = 60;`, the value of `src/capture/mod.rs:99`, which is `pub(crate)` and not re-exported, so the integration test cannot read it. Outside every module's scope; a `pub` re-export would remove the twin.
8. `texture_bytes`'s `transient_saves_memory` parameter (`src/stats/mod.rs:185`): `src/render/targets.rs:373` and `src/assets/material.rs:233` pass a variable, `src/render/tonemapping_pass.rs:142` passes `false`; exercised both ways, clean.
9. The two `assets()` under `cfg(feature = "cook")` and `cfg(not(feature = "cook"))` (`src/preflight/mod.rs:235-273`): CI builds and tests without the feature (`.github/workflows/ci.yml:116`, `:134`), so neither side is dead; clean.
10. `SystemTimer` implements `tracing::Subscriber` in full (`src/stats/mod.rs:357-394`, eight methods, four of them empty) to time bevy_ecs's system spans; it has one caller (`src/state.rs:319`). A `tracing_subscriber::Layer` would need a `Registry` installed as the default subscriber for the schedule's lifetime, while the doc at `:311-314` explains that a span reports to the subscriber that was the default when it was created, so `scope` captures the systems whatever the global subscriber filters; the width is what the mechanism needs. Not filed.
11. The bench's `Finding { metric, verdict, detail }` with its `Display` "verdict metric detail" (`src/bench/mod.rs:400-421`) and the preflight's `Check { name, verdict, detail }` with its `Display` "verdict name detail" (`src/preflight/mod.rs:73-99`) are one shape twice, with different verdict sets (two against four). Weighed and not filed: one type would carry `Ok` and `Note` into the compare's Findings, which never produce them; L5-T21 covers the name they share.
12. The fourth process runner, `src/cook/fetch.rs:136-158` (`curl`), falls in cook, assets and scenes; it is counted in L5-T19's four sites and needs no Finding of its own.
13. `FrameStats` fields read outside the file only through `Display`: `renderables_refused` (written at `src/state.rs:271`), `static_instances_left_out` (`src/render/renderer.rs:495`), `allocator_report_bytes` (`src/state.rs`, `src/render/renderer.rs`). `praxis capture` prints the `Display` (`src/bin/praxis.rs:785`) and `tests/frame_stats.rs:253` reads it, so none is vestigial.
14. `GpuFrameStats::gate_time` and `phase_two_triangles_drawn` are absent from `FrameStats`'s `Display` (`src/stats/mod.rs:109-179`) while the bench reads both (`src/bench/mod.rs`, 3 and 1 lines). The `Display`'s doc at `:107-108` promises groups of `key=value` pairs, not every field; not filed. If the capture's print is meant as the full record, that is a lane 6 or lane 7 question.
15. Preflight's comments checked against other files and found accurate: `:212` ("With no scene named, bench records every scene whose assets are present") against `src/bin/praxis.rs:434-441`, which takes `ReferenceScene::ALL` less an unfetched helmet; `:219` ("CI fetches no helmet, so benches none") against `.github/workflows/ci.yml:178`, which runs `praxis bench` with no scene and no fetch step; `:67` (`Fail`: "`scripts/verify.sh` fails until it is fixed") for goldens against `scripts/verify.sh:61` (`PRAXIS_REQUIRE_GOLDENS=1`) and for baselines against `scripts/verify.sh:30-33`.
16. Abbreviations weighed and allowed: `gpu` as a local (`src/capture/mod.rs:294`, `src/stats/mod.rs:121,133`), a domain word; `gh`, `ps`, `ioreg`, `pmset`, `tasklist`, `powershell` as program names in `output` calls and test locals (`src/preflight/mod.rs:289,295,361,432,434,758`); `id` (`:466,477`); `pid=,comm=` and `/FO`, `/NH` as `ps`'s and `tasklist`'s own flags; `KEY` (`:334`), a full word.
17. The `Finding` type in `src/bench/mod.rs:407` uses the glossary's term Finding (an audit's) for a compare's result, the same borrowing as L5-T21; falls in tooling, the bench.


**E.**

1. Weighed and not filed: the four history commands in the binary each run `let adapter = pollster::block_on(praxis::gpu::adapter_info())?;` and pass `&adapter.name, adapter.backend.to_str()` (`src/bin/praxis.rs:531-537`, `548-550`, `562-566`, `578-581`). A helper would replace one line with one line; the shape worth questioning is the `(repository: &Path, adapter: &str, backend: &str)` triple that `unrecorded_scenes`, `suspects`, `bisect` and `file_regressions` all take (`src/bench/history.rs:595-599`, `src/bench/history/verdicts.rs:317,366-370,451-454`), where one adapter-key type would serve; that falls in tooling, the Bench history, and the import and the verdicts.
2. Weighed and not filed: the `match experiment { Some(name) => publish_experiment(..), None => publish(..) }` appears twice in the binary (`src/bin/praxis.rs:468-471`, `511-514`), four lines each with a different third argument; a helper would be as long as what it replaces.
3. Weighed and not filed: the Baseline read chain `std::fs::read(&baseline_path).map_err(anyhow::Error::from).and_then(|json| Measurements::from_baseline_json(&json))` appears twice (`src/bin/praxis.rs:629-631`, `678-680`). `src/bench/` has no reader of a results directory (`grep -rn read_dir src/bench` hits only the Bench history's record directories at `src/bench/history.rs:1007,1009,1045`), so `result_paths`, `baseline_path` and `read_json` are not duplicated there.
4. Weighed and not filed: `Path::new(".")` is passed nine times in the binary (`grep -c 'Path::new(".")' src/bin/praxis.rs` gives 9) as the `repository` of every history function. The history tests pass fixture roots, so it is not a parameter every caller passes the same value; it is the binary's working directory, which is the right place for that knowledge.
5. Weighed and not filed: `first_success`'s `let Some(..) = failures.pop() else { anyhow::bail!("no backend to try") }` at `src/gpu/mod.rs:530-532` is unreachable from its two callers (`first_backend` ensures a non-empty list at `src/gpu/mod.rs:504-507`; the test at `:820,829` passes two). Three defensive lines for an empty slice, since `std` has no non-empty slice type.
6. Weighed and not filed: `[scene] if ReferenceScene::from_name(scene).is_some() => Command::Window(parse_scene(scene)?)` at `src/bin/praxis.rs:310-312` parses the name twice and the `?` cannot fail. The guard keeps a one-word non-scene (`bench` alone) on the usage arm, which `a_command_missing_its_arguments_is_refused_with_the_usage` pins; `if let` guards are not stable Rust.
7. Weighed and not filed: the test constant `MSAA_X4` at `src/gpu/mod.rs:562-565` holds `MULTISAMPLE_X2 | MULTISAMPLE_X4 | MULTISAMPLE_RESOLVE`, more than its name says. Test-only and no abbreviation; a rename to `M4_PRO_FORMAT_FEATURES` would read truer.
8. Near-miss on comment accuracy, not filed: `adapter_report`'s doc says the lines are "each feature the roadmap plans on" (`src/gpu/mod.rs:128-129`). `ROADMAP.md` names immediates alone (`ROADMAP.md:52`); ADR 0021 says subgroups are among the features "which nothing uses", mesh shaders are what "ADR 0019 does not use", and `INDIRECT_FIRST_INSTANCE` "left the floor". `tests/gpu_context.rs:3-5` pins the same eight as "the features the renderer plans to use". `docs/research/source-audit.md:170` leaves the matrix's mesh-shader, subgroup and immediate rows to the milestone that implements them, so "plans on" is loose rather than wrong; a doc or ADR question for lane 7 if it is one at all.
9. Weighed and not filed: `BACKEND_ORDER`'s doc carries driver versions (617.42, 621.01) and an exception code (0x87D) at `src/gpu/mod.rs:475-476`. Identifiers of a measured failure, not measurements; `docs/research/source-audit.md:200` reviewed the comment and marked it measured.
10. Falls in tooling, the bench, and capture, stats and preflight: `context.adapter.get_info().device_type` is read through the field at `src/bench/mod.rs:440` and `src/capture/mod.rs:389` where the getter `device_type()` exists (`src/gpu/mod.rs:106`), and `context.adapter.get_info()` whole at `src/bench/mod.rs:465` and `src/render/renderer.rs:405` (the last falls in render). The getter is justified by its callers outside the crate (`tests/goldens.rs:101-102`); the mixed access is a near-miss on module depth at those sites, not in `src/gpu/mod.rs`.
11. Outside every module's scope: `surface_caps` at `src/lib.rs:155`, read at `:162-163` (3 uses), abbreviates "surface capabilities"; the full-word rule, as `src/gpu/mod.rs:451` spells it (`surface_capabilities`).
12. Falls in render: `let info = context.adapter.get_info();` at `src/render/coverage_tests.rs:661`, the same abbreviation as L5-T32.
13. Not filed, the map's fog line: a Vulkan adapter below the floor ends the run at `request_device` (`src/gpu/mod.rs:60-65`) rather than falling through to DX12, since `first_backend` wraps only the adapter request (`src/gpu/mod.rs:33,42-47`) and the floor check runs after it.
14. Falls in lane 6: `render_targets_at_the_reference_size_get_memory_of_their_own` (`src/gpu/mod.rs:847-902`) returns early when the backend reports no allocator, which Metal does (`:870-877`), so on the reference machine it asserts nothing; it runs on CI's lavapipe and the RTX 2070 SUPER. Also lane 6: `m4_pro_limits` (`src/gpu/mod.rs:587-647`) is a 60-line literal of one adapter's limits on one wgpu version, which two tests use; a change in wgpu's `Limits` fields breaks compilation rather than a test.
15. Allowed abbreviations weighed in these two files: `msaa_x4` (`msaa` is a domain word), `ci_run` (`ci`), `json`, `stdout`, `fmt` (std's module and the trait method as the API spells them), `driver_info`, `subgroup_min_size` and every other wgpu field name as wgpu spells it, `T` as a type parameter (not an argument, parameter, local, field or lambda binding), `png` in the usage string.
16. `vendor()` has one caller and it is outside the crate (`tests/frame_stats.rs:63`), so its `pub` stands. `DEFAULT_CAPTURE_SIZE` has one use (`src/bin/praxis.rs:241`) and is a named, documented constant, not a vestige.
17. `src/bin/praxis.rs:601`'s comment "Both of CI's configurations render at its size" holds: `Configuration::CI_COUNTERS` takes `Self::CI.width` and `Self::CI.height` (`src/bench/mod.rs:82-87`).
18. The usage text was checked in the reverse direction too: it names no command the enum lacks in a `cook` build, and its option scopes (`--crowd-form` on `capture` and `bench`, `--frame` on `capture` and `export-scene`, `--skip-aliasing` and `--no-publish` on `bench`, `--release-build-seconds` on `publish`, `--experiment` on `bench` and `publish`) match the `ensure!` calls at `src/bin/praxis.rs:185-216`.


**F.**

1. `.github/workflows/ci.yml` pins mutable refs at 18 sites (`actions/checkout@v4` at `:37,63,101,151,223`; `dtolnay/rust-toolchain@stable` at `:39,65,103,153`; `Swatinem/rust-cache@v2` at `:43,67,107,155`; `actions/upload-artifact@v4` at `:87,138,187,200`; `actions/download-artifact@v4` at `:225`) while `.github/workflows/pr-lint.yml:29` pins a commit SHA with `# v4.4.0`. Lane 7 filed it as L7-R04 against `CLAUDE.md:387-388`; not refiled. Two facts for that ticket: there is no `.github/dependabot.yml`, so SHA pins would not be refreshed by a bot; and `dtolnay/rust-toolchain@stable` reads its ref as the toolchain name, so a SHA pin of it needs `with: toolchain: stable`.
2. `ci.yml`'s setup block (`checkout`, `rust-toolchain`, `rust-cache`) appears four times (`:37-43`, `:63-67`, `:101-107`, `:151-155`) and the lavapipe install three (`:69-73`, `:109-113`, `:157-161`). Weighed and not filed: the blocks differ (`components: clippy, rustfmt` at `:41`, `components: clippy` at `:105`, none elsewhere), so a composite action under `.github/actions/` would need an input and would be about as long as the 27 lines it removes, plus one more file. If the triage wants it, it is a workflow change with no pixel moved and no measurement changed.
3. `ci.yml:13-15` says "Each push to main runs in a group of its own, so none is cancelled and every one publishes its Bench records". The group key `ci-${{ github.event.pull_request.number || github.run_id }}` (`:17`) is the run id on a push, so concurrency cancels no `main` run: the comment holds for what it describes. But `gh run list --workflow ci.yml --branch main --event push --limit 60` shows 8 of 60 `main` push runs since 2026-10-26 ended `cancelled`; the one inspected (`gh run view 41223987799 --json jobs`, 2026-10-31 17:19 UTC, head `7c562597`) had all four jobs cancelled at once about five minutes in, before `publish-bench` could run, so those commits published no lavapipe Bench record. Whatever cancels them (a hand, a tool, a timeout) is not in the tree. Falls in lane 1 (ADR 0024: "CI's lavapipe on every push to `main`"), and lane 6 if a check should notice a missing record.
4. `ci.yml:51-52`, the `check` job's `Build` step (`cargo build`) before `Test` (`cargo test`): `cargo test` compiles the library, the binary (integration tests need it) and the test targets, so the step compiles nothing `Test` would not; it names a compile failure apart from a test failure in the job's log. Weighed as a readability choice, not vestigial. Not filed.
5. `scripts/sweep.sh:57-62` derives the 450 s cap from "219 s on the M4 Pro, 155 s on the RTX 2070 SUPER, both measured 2026-10-31": the same form as L5-T35 (a timing in a shipping comment nothing re-measures) but dated, which stamps it as a measurement, and it is the cap's derivation rather than a justification. Weighed and not filed; the verifier may want both treated alike.
6. `scripts/sweep.sh:62` `PRAXIS_SWEEP_CAP_SECONDS` is set nowhere in the tree (`grep -rn "PRAXIS_SWEEP" .`: the header `:44`, the read `:62`, and `PRAXIS_SWEEP_UPDATED` at `:74`); `WGPU_BACKEND` (`:44-45`) is read by `src/gpu/mod.rs:485-506`, so the header is accurate about it. An operator's knob the header documents, not flexibility nothing exercises. Not filed.
7. `scripts/gallery.sh:31-33` hold `golden_frame=59` and `town_frames=(59 300 500 700 900 1200 1500 1800)` as copies of `tests/goldens.rs:294,299`; equal at this commit, and the comment names the source. No test ties them (`render.py:38-40` shows the tree's precedent of reading a constant from source text with a regular expression; a shell `grep` of `tests/goldens.rs` would do the same). A drift between them would be caught by nothing: lane 6's.
8. `scripts/cycles-reference/compare.py:27-29`: "The constants and matrices of display.wgsl and gt7.wgsl, which cite their sources; each function below is its namesake there." `gt7_curve`, `pq_encode`, `pq_decode`, `rec2020_to_ictcp`, `ictcp_to_rec2020`, `gt7_tonemap` are in `gt7.wgsl:46-115`, `tonemap` and `srgb_encode` in `display.wgsl:22,30`, `GT7_PAPER_WHITE` and the five `PQ_*` constants at `gt7.wgsl:32-40`, `REC709_TO_REC2020` and `REC2020_TO_REC709` at `display.wgsl:8,13`. The four LMS and ICtCp matrices (`compare.py:67-74`) are inline `dot` rows at `gt7.wgsl:84-108` with no names, `EIGHT_BIT_STEP` and `display` (`:136`) have no namesake (the WGSL composition is `srgb_encode(tonemap(...))` in `tonemapping.wgsl:16`'s `fs_tonemap` and `shader_function_tests.rs:1084`), and `smoothstep` is the WGSL builtin. Weighed as a near-miss: `display`'s own docstring two lines below says what it is, and "constants and matrices of" is true of unnamed rows. Not filed.
9. `compare.py:198-205`, the `crop` command: no caller in the tree (grep count 0 outside the file). Not vestigial: its seven products are the committed images `DIFFERENCES.md` shows, its usage is in the docstring at `:8`, and a milestone's visual audit cuts crops by hand. L5-T38 is about the entry that names its source, not about the command.
10. Allowed abbreviations weighed in these files: `tmp` (`target/tmp/`, a directory name, `gallery.sh:5,9,28`), `src` and `dev` in paths, `fmt` (cargo's subcommand, `verify.sh:72`), `np` and `bpy` (numpy's convention, Blender's module), `co` (Blender's vertex attribute, `render.py:237,435`), `bsdf`, `uv`, `uvs`, `uri`, `exr`, `rgb`, `rgba`, `ictcp`, `lms`, `pq`, `sha` (`pr-lint.yml:52`), `raw` (`pr-lint-test.sh:153`, the raw-content host), `x, y, z, w` as quaternion components (`render.py:391,431`), `x, y, width, height` as a crop's coordinates (`compare.py:200`), and `smoothstep(edge0, edge1, x)`'s parameters (`compare.py:78`): WGSL names the builtin's third argument `x`, GLSL names the first two `edge0`, `edge1`.
11. `render.py:32` `UNBOUNDED_RANGE = 1e9` is read only by `calibrate.py:69,71`; `render.py`'s own `build` passes the export's `light["range"]` (`:404`), which `src/cook/export.rs:119,242` writes for every point light. A constant of the calibration living in the module it imports; one line, two uses. Not filed.
12. `tests/references.rs:154` scans `rs`, `wgsl`, `md`, `sh`, `toml`: the registry keys cited at `render.py:99,304,313` and `compare.py:30` are checked by no test (the `.sh` citations at `sweep.sh:19,96` are). All four keys exist in `docs/references.toml` (`:79`, `:109`, `:139`, `:170`) at this commit. Falls in lane 6.
13. `scripts/pr-lint-test.sh:23` "`scripts/pr-lint-test.sh`: 20 passed." is sample PR body text, not a claim about the test, which runs about 47 cases (41 `expect_` lines, one of them in a loop of 5 paths, and 2 inline cases). `:153` is a fixture URL whose last segment names a PR directory on the `pr-images` branch: a data string, not a comment, outside the ticket-number convention test's `.rs`/`.wgsl` scope. Neither filed.
14. `scripts/pr-lint.sh:67`'s `src/cook/(texture|mesh)(\.rs$|/)` accepts `src/cook/texture/` and `src/cook/mesh/` directories that do not exist at this commit (`ls src/cook/`: files only). `CLAUDE.md`, Project shape: "When a feature folder grows past one file, split into `feature/{resource,system,...}.rs`", so the form anticipates a documented convention; `pr-lint-test.sh:177` exercises only the `.rs` forms. Not filed.
15. `scripts/sweep.sh:18-43`'s install recipe against `sweep.plist` and `sweep-task.xml`: the label `com.github.gariasf.praxis.sweep` (`plist:7`, recipe `:24,26,39`), the `@SWEEP@` placeholder (`plist:11,25,27`; `xml:36`), `@BASH@` (`xml:35`), 04:00 (`plist:19-22`; `xml:10`), wake (`xml:28`), logged-on user (`xml:19`) all match. The Mac recipe creates `target/sweep` before launchd opens its log there (`:22`, `plist:25,27`); the Windows recipe does not and need not, since Task Scheduler opens no file and `sweep.sh:54` makes the directory. Clean.
16. `render.py:162-206`, the `Gltf` class, reads glTF itself rather than through Blender's importer: a design choice (the materials are built by hand at `:276-345` to match praxis's model) and not health. Not filed.
17. `ci.yml:148-149` and `:216`: `software-bench`'s `benched` output is `steps.bench.outcome`, and `publish-bench`'s `if` also tests `!cancelled()`; with note 3's cancellations the job's guard behaves as the comment at `:210-213` says. Clean.
18. Only the `check` job is a required status check (note on L5-T36); `shipping-goldens`, `software-bench` and the PR form check are not required either, and no comment says so of them. Whether they should be is the owner's, through lane 7's rules or the map; outside this lane.
19. `DIFFERENCES.md`'s five entries name seven images (`:29,31,43,57,59,67,76`) and `scripts/cycles-reference/differences/` holds seven files with those names: no entry without an image, no image without an entry.

### Counts

| Severity class | Findings |
|---|---|
| `image` | 0 |
| `budget` | 0 |
| `correctness` | 0 |
| `drift` | 17 (L5-T02, L5-T05 to L5-T08, L5-T13, L5-T14, L5-T23, L5-T24, L5-T26 to L5-T29, L5-T35 to L5-T38); load-bearing 3 (L5-T02, L5-T23, and L5-T27 in a build without the `cook` feature) |
| `health` | 21 (L5-T01, L5-T03, L5-T04, L5-T09 to L5-T12, L5-T15 to L5-T22, L5-T25, L5-T30 to L5-T34) |

By axis: over-engineering 8 (L5-T11, L5-T12, L5-T16, L5-T17, L5-T18,
L5-T19, L5-T31, L5-T33); vestigial 2 (L5-T01, L5-T15); cognitive
complexity 1 (L5-T25, the probe's one hit in this module, the binary's
command parser); naming 6 (L5-T03, L5-T04, L5-T21, L5-T22, L5-T32,
L5-T34, of which three are glossary terms borrowed for another concept and
three are abbreviations or a single letter); module depth 4 (L5-T09,
L5-T10, L5-T20, L5-T30, each a `pub` surface in a `pub` module that
nothing outside the crate reads); comment accuracy 17 (the `drift` rows,
of which four are measured figures in shipping comments that nothing in
the tree re-measures, L5-T05, L5-T06, L5-T29 and L5-T35, and one a dated
trigger, L5-T36). The same shape, a process runner, was filed from three
sides and stands once (L5-T19); the glossary's `Verdict` is borrowed
twice, at two sites, and stands twice (L5-T03, L5-T21). Every `health`
row is in shipping code; none is in a test module. No implied fix moves a
pixel; L5-T01, L5-T12 and L5-T16 change what the binary accepts or what a
publish writes, so their proof is the sweep's and CI's output unchanged
beside green tests, and the rest carry the refactor licence with Goldens
bit-exact, counters equal and timings within noise. Axis sweep: 26 file
rows. Interface widths: 49 types and module surfaces, 226 public items
counted where a row gives a number. Files read in full: the 26 of the
module (12 Rust files, 10,130 lines; 13 shell, Python, Markdown, XML,
plist and YAML files, 1,906 lines, beside the display ramp); read in part
for use counts: `src/lib.rs`, `src/state.rs`, `src/render/` (the
renderer and the coverage tests), `src/cook/fetch.rs`, `tests/`,
`docs/bench/index.html`, `docs/agents/workflow.md`, `docs/performance.md`,
`docs/research/source-audit.md`, the ADRs the module cites and the
`bench-history` branch's records through read-only `git` commands.

## Killed

Written by the report session.
