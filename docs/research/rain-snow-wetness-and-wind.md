# Rain, snow, wetness and wind: state of the art

Researched 2026-11-21 for ticket #951 (parent #944): what falls, what it leaves on surfaces and the wind that moves it, driven by one weather state, under ADRs 0008, 0019, 0020 and 0022. Sky, clouds and fog are decided (`docs/research/volumetrics-without-screen-space-history.md`); open water is #950's, terrain #948's, what bends in the wind #949's. It ends in options and chooses none.

**Summary.** Nothing surveyed needs screen-space history, and no option rebuilds a decision M5 took or M6 is about to take, except two forms that can be avoided: wetness or snow as a new Channel or a lobe, and snow trails that rays must see.

- **Cover with no shadow maps.** Every shipped title that says how it keeps rain from under a roof (Remember Me, Assassin's Creed 4, Far Cry 6) reads a depth map drawn from above; Far Cry 6 and Wicked Engine keep it in the sun's shadow atlas, and Torque3D casts a ray per drop instead. Praxis has no such pass (ADR 0020), so each form is priced as what it would add. *A depth map of its own*: a depth-only view from above through culling and an indirect draw of its own; Remember Me's 256×256 map cost 0.20 ms on Xbox 360 and 0.32 ms on PS3, and nothing is published for either of Praxis's adapters. *A cooked height per Chunk*: no pass at run time, a field in the Chunk's Blob and a re-cook; static geometry only, dry under an overhang that slanted rain reaches; no source ships it. *A ray per drop*, through the TLAS that exists: at the repo's measured 0.96 ns a ray, a lower bound from coherent shadow rays on the helmet's 257,000 triangles, M4 Pro, 2560×1440, 4x MSAA (`docs/performance.md`, Where the shadow mask pass's time goes), far smaller than `town`'s TLAS, 100,000 drops come to at least 0.10 ms and Assassin's Creed 4's 320,000 to at least 0.31 ms (computed). *A ray per pixel* toward the rain, for wet shading, is one more sun: 1.8 to 2.0 ms on the M4 Pro at 2560×1440 in the GI note's open scene (`docs/research/global-illumination-and-lighting.md`, Measured on the M4 Pro). *Authored cover*: Interior volumes, at no run-time cost, which leave eaves and balconies wet. Far Cry 6 splits it: rays for what moves, a map for the rest.
- **Wetness: inputs, not a lobe.** Every shipped source read wets a surface by changing the BRDF's inputs (albedo darkened by porosity, roughness lowered, the normal flattened), and none adds a lobe. In Praxis that is arithmetic on one or two scalars before the BRDF, evaluated always, so it carries no branch; a uniform `if` around one term cost the M4 Pro's main pass 10 to 12% at 2560×1440 (`docs/performance.md`, Measurements behind the code). An authored porosity, puddle or drip mask is a sixth Channel, since ADR 0018's formats leave no data Channel a free component, unless it rides in emissive's BC7 alpha, which glTF leaves unused: then every Material with a mask carries an emissive texture. A lobe is a second specular evaluation per light per sample at 4x MSAA and up to three Channels; OpenPBR's coat itself leaves out the darkening of a porous base. The input form serves ground, props, cloth and hair as they are: Far Cry 6 wets cloth and hair with its props' function. Skin is the exception: drips need a normal of their own, a droplet normal and mask (Far Cry 6) or a specular normal derived from a wetness mask (The Last of Us Part II), so an authored mask, still an input and no second lobe. No source wets cloth's translucency.
- **One weather state** holds, across the sources: rain or snow intensity; a wetness, a puddle and a snow level, each with its own rise and fall rate; wind direction, magnitude and gusts; cloud cover, fog and the sky's parameters; lightning strikes; and where it varies (regional limits, local events of a position, a radius and a blend). Under ADR 0022 it is a resource of the fixed schedule, advanced per Tick from the Tick counter and the seeded generator. Game state reads it there: grip, a horse's gait in snow, people who shelter, wind on a projectile, audio's cues. Prepare reads it once per frame for the sky tables and cloud map, fog, drops, the Materials' wetness and snow, vegetation, water, sun and sky light, and through them the probes and exposure. Drop positions, ripples and a GPU wind volume are presentation and need not replay.
- **Falling rain and snow** ship as simulated GPU particles (320,000 drops updated in under 0.1 ms and drawn in 0.4 to 4.0 ms in Assassin's Creed 4, hardware not named) or as streak textures scrolled on a camera-centred cylinder (1.69 to 1.72 ms on PS3 and Xbox 360 in Remember Me, whatever the intensity). Both are complete each frame and no source says its rain relies on TAA. Rain is Praxis's first particles and the first use of ADR 0019's sorted transparency; no source prices streaks under 4x MSAA.
- **Lying snow** is the input form, by how far the normal faces up. Trails are a height texture written by what touches the snow (under 1.0 ms and 2 MB on PS3 and Xbox 360 in Batman: Arkham Origins), drawn by relief, which no shadow sees, by tessellation, which wgpu 30 lacks, or, in one WGSL demo, by displacing a dense grid in the vertex stage, which is terrain's to offer.
- **Wind** ships as a global vector, scrolled noise and local sources summed, sometimes with a fluid volume around the camera (God of War: about 0.1 ms and 384 KB, hardware not named). Every source evaluates or steps it per frame, except Torque3D's emitter, on a fixed 32 ms tick but seeded from the clock; under ADR 0022 the wind game state reads is a function of the Tick, and a GPU volume is presentation only.
- **Lightning** is a short-lived light in every source but ToyShop, which bakes it into lightmaps. In Praxis its rays are complete in its frame (ADR 0020), while probes that reach 90% in 0.5 s answer after a flash of 4 to 8 frames is over; Red Dead Redemption 2 keeps strikes out of its sky probes, and no source says what a flash should do to exposure.

Sources: Lagarde's articles on Remember Me's rain (2012 to 2013); GDC and SIGGRAPH decks from 2006 to 2022 with speaker notes; studio posts and an interview; OpenPBR and glTF clearcoat; five papers; Unreal 5.9's and Unity 6.6's documentation; and nine register entries' source at the register's commits, never built or run. Each claim relied on is in `docs/references.toml` by key, and every quote there was matched by script against a saved copy of its source on 2026-11-21. Nothing was measured in this repo.

Terms: the **class** column is `CONTEXT.md`'s Temporal class, used of any term. Licence classes are #944's: **A**, adaptable with citation; **E**, evidence only; **P**, Apache-2.0 alone. Unreal, Unity and Flax are class E and were read through documentation only [ref:ue-eula-2026-11-engine-code] [ref:unity-terms-of-service-source]; Spartan Engine was opened only at commit `cb258a1`, its last under MIT [ref:spartan-engine-cb258a1-licence]. A **drop** is one rain or snow particle.

## Comparison

| # | Technique | Buys | Published cost, hardware | Class; TAA in its images | Maturity | Fit: ADRs 0008, 0019, 0020, 0022 | Reference code |
|---|---|---|---|---|---|---|---|
| 1 | Simulated streak particles | Depth, parallax, wind, splashes where drops land | Update under 0.1 ms, draw 0.4 to 4.0 ms, up to 320,000 drops, hardware not named; 126 frames a second for 1,000,000 streaks at 1280×1024 on a GeForce 8800 GTX (vendor sample) | a; not stated for rain (Far Cry 6's frame has TAA) | Shipped: Assassin's Creed 4, Far Cry 6 | First particles, first transparent pass; sub-pixel streaks unpriced at 4x MSAA | Wicked, bevy_hanabi: A |
| 2 | Streak textures on a cylinder | Fixed cost at any intensity | 1.69 ms PS3, 1.72 ms Xbox 360, resolution not stated | a; not stated | Shipped: Remember Me | A blended pass with a depth test per layer | None |
| 3 | Depth map from above | Cover for drops, splashes and wet shading | 0.20 ms Xbox 360, 0.32 ms PS3, 256×256 | a, b where refreshed in sections; no | Shipped: Remember Me, Assassin's Creed 4, Far Cry 6 | A depth-only pass Praxis lacks | Wicked, Godot: A |
| 4 | Cooked top-down height | Cover with no run-time pass | None | static data | Not shipped by any source read | A field per Chunk (ADR 0018); static only | None |
| 5 | A ray toward the rain | Exact cover under slanted rain and moving roofs | None | a per drop; b every few frames per object | Shipped: Far Cry 6 for moving objects; engine: Torque3D per drop on the CPU | The TLAS exists (ADR 0020) | Torque3D: A |
| 6 | Authored cover | No run-time cost | None | static data | Shipped: Remember Me, Far Cry 6 | Interior volumes exist | None |
| 7 | Wetness as Material inputs | Darkening, sheen, flattened normal from one scalar | None | a; no | Shipped: Remember Me, Uncharted 4, The Last of Us Part II, Far Cry 6 | Arithmetic, no branch; ground, cloth, hair | Wicked: A |
| 8 | Puddles and ripples | Standing water that grows and dries; rain rings | 0.14 ms PS3, 0.15 ms Xbox 360 for a 256×256 ripple texture | a; no | Shipped: Remember Me, Far Cry 6 | A mask the cooked Material lacks, or decals | None |
| 9 | Wetness kept per surface | Things that wet and dry at their own rate | None | b, state outside the screen; no | Shipped: Far Cry 6, Call of Duty: Modern Warfare II | A scalar per Instance (ADR 0003) or per vertex | Wicked: A |
| 10 | A water layer as a lobe | Water's reflection over a rough base | None for real time | a | Standards: glTF clearcoat, OpenPBR; no title found | Per-light work at 4x MSAA, up to three Channels | None |
| 11 | Snow cover as inputs | Snow on any Mesh that faces up | None | a | Engine: Torque3D; no shipped title's account | As row 7; under a roof, rows 3 to 6 | Torque3D: A |
| 12 | Trails in a height texture, by relief | Footprints at any triangle density | Under 1.0 ms of GPU, 2 MB, PS3 and Xbox 360 | b, state outside the screen; not stated | Shipped: Batman: Arkham Origins on consoles, The Last of Us Part II, God of War (2018) | Rays see flat snow; depth pre-pass and lit pass must agree | None |
| 13 | Trails as displaced geometry | Silhouettes and shadows that follow the trail | None from a title; a demo's whole base scene 1.64 ms at 2560×1440 on an RTX 5070 Ti, its author's | a; the demo uses TAA, titles do not say | Shipped by tessellation: Batman on PC, God of War Ragnarök, Ghost of Yōtei | No tessellation in wgpu 30; a dense grid is terrain's; BLAS stays flat unless rebuilt | snowflow_demo: A [ref:snowflow-5450397-licence] |
| 14 | Simulated snow or soil | Snow that piles and flows; footprints from force | 68.5 frames a second, 1.33 M particles, four Tesla V100; 65 frames a second, 256×256 grid, laptop CPU | a | Research | Outside a frame's Budgets | footprints: A [ref:alvarado-2022-footprints-code-licence] |
| 15 | Closed-form wind | One answer for every reader, CPU and GPU | None | a; no | Shipped: Ghost of Tsushima, Uncharted 4, Crysis, Far Cry 6 | A function of the Tick | Wicked, Spartan at `cb258a1`, bevy_feronia: A |
| 16 | Wind volume around the camera | Gusts that linger and travel | About 0.1 ms, 384 KB (God of War); about 150 µs (Horizon Zero Dawn); hardware not named | b, state outside the screen; not stated | Shipped: God of War, Horizon Zero Dawn | Presentation only, unless stepped per Tick on the CPU | None |
| 17 | Weather state from presets | Authored weather, regions, local events | None | not rendered | Shipped: Far Cry 6, Horizon Forbidden West, Red Dead Redemption 2 | A resource of the fixed schedule | Wicked: A |
| 18 | Lightning as a short-lived light | A flash that lights the scene | None | a for the light; kept out of cloud history in two titles | Shipped: Far Cry 6, Red Dead Redemption 2, Horizon Forbidden West | Rays class (a); probes lag it | None |

## Falling rain and snow

**1. Simulated particles.** Assassin's Creed 4 simulates rain in compute over a 3×3 grid of clusters around the camera, with wind, a top-down map for cover and collisions against the depth buffer; its geometry-shader draw was "the performance bottleneck", 0.4 to 4.0 ms against under 0.1 ms to update up to 320,000 drops, hardware not named [ref:wronski-2014-ac4-gpu-rain]. Far Cry 6 emits, simulates, sorts and draws on the GPU, recycles streaks in a cylinder around the player so their count holds at any speed, spawns splashes where a streak meets the depth buffer, and lights rain from probes because per-pixel lighting was "Too expensive"; no timing [ref:zhou-weick-2022-fc6-rain-particles]. Its clouds lean on history and the frame's TAA [ref:zhou-weick-2022-fc6-temporal-reliance]; the deck says neither of rain. The foundations are a measured model of a streak's appearance under light and view [ref:garg-nayar-2006-rain-streak-model] and a vendor's sample that draws its textures as billboards [ref:tariq-2007-rain-gpu-particles]. No real-time paper on falling rain or snow since 2019 was found.

**2. Streak textures.** Remember Me's author weighs the two: particles move convincingly but "Stronger precipitation requires increasing the number of particles", while textures cost the same in any rain and lack depth. He draws four layers of scrolled streak texture on a cylinder around the camera, each with a virtual depth tested against the scene's, the nearest two also against the cover map [ref:lagarde-2012-raindrops-textured-layers]. ATI's ToyShop, his major reference, composites rain full-screen and names stretched alpha-blended particles as games' form then [ref:tatarchuk-2006-toyshop-rain].

## Cover

**3. A depth map from above.** Remember Me renders "a depth map view from the top in the rain direction", 256×256 over 20 m, of the meshes artists tag, reads it back to place splashes, and reuses it for drops and the camera's own droplets [ref:lagarde-2012-rain-depth-map-from-above] [ref:lagarde-2012-camera-cover-ray-or-depth-map]. For wet shading its author names its limit, "the limited distance/resolution it covers", and prefers painted vertices [ref:lagarde-2013-wet-only-where-exposed]. Far Cry 6 keeps a rain shadow map of static elements in its sun shadow atlas, "updated a few sections each frame" [ref:zhou-weick-2022-fc6-rain-shadow-map], and a wetness shadow for static surfaces in its deferred shadow pass [ref:zhou-weick-2022-fc6-wetness-static-and-dynamic].

**4. A cooked height.** No source read cooks cover. Godot's height field holds the same data, drawn at run time, and states the limit any top-down height shares: it "cannot represent overhangs" [ref:godot-4.7.2-particle-heightfield]. For straight rain the highest surface in a column is the cover; slanted rain reaches under an edge a height keeps dry.

**5. Rays.** Torque3D casts a ray per drop when the drop is made, which "can be expensive" [ref:torque3d-9521d43-precipitation]. Far Cry 6 gives characters, weapons and vehicles a wetness component that casts a ray "every few frames" and wets or dries gradually [ref:zhou-weick-2022-fc6-wetness-static-and-dynamic]. Remember Me's author names a line check per moving object, "but this can be costly" on that generation's CPUs [ref:lagarde-2013-wet-only-where-exposed]. No source gives a figure.

**6. Authored cover.** Remember Me paints dry vertices [ref:lagarde-2013-wet-only-where-exposed]; Far Cry 6 paints a mask for vehicle interiors and limits weather by zone [ref:zhou-weick-2022-fc6-regions-overrides-replication].

## Wet surfaces

**7. Wetness as inputs.** Lagarde finds that "Almost all games I know" change BRDF parameters and concludes that "simply tweaking BRDF parameters from dry material do the job"; the thin water layer that would need "a dual layer BRDF" he calls expensive and approximates with a normal blended toward the geometric one [ref:lagarde-2013-wet-surfaces-by-brdf-inputs]. Porosity drives darkening: a texture in a free channel or, where none can be afforded, a value derived from roughness, metals left undarkened [ref:lagarde-2013-porosity-input]. Far Cry 6 does the same nine years on, porosity a linear function of smoothness with two factors per Material type [ref:zhou-weick-2022-fc6-apply-wetness-porosity]. Uncharted 4 and The Last of Us Part II drive every BRDF parameter from one wetness value [ref:brinck-2016-u4-wetness-and-porosity] [ref:brinck-2020-tlou2-wetness-one-parameter]. On characters Far Cry 6 reuses the props' function for cloth with a cap on smoothness, one set of factors for hair, and for skin a texture holding a droplet normal and a mask [ref:zhou-weick-2022-fc6-wet-cloth-hair-skin]. Lagarde leaves out "The translucency of material when wet (cloth, paper…)" [ref:lagarde-2013-porosity-input].

**8. Puddles and ripples.** Remember Me floods a painted depth and the Material's height map against two levels the CPU raises with rain and lowers with time, and draws ripples from one 256×256 normal texture regenerated each frame for the whole Game world; it budgets 3 ms for all of rain's core effects at 30 frames a second [ref:lagarde-2013-flood-levels-and-ripple-map]. Far Cry 6's puddles are terrain decals that write only porosity, grown from their centres, under one animated 256×256 normal texture of rings and wind waves [ref:zhou-weick-2022-fc6-puddles-and-ripples]. Red Dead Redemption 2's form in its deformed ground, by observation [ref:df-2018-rdr2-snow-and-mud-trails-persist].

**9. Wetness kept per surface.** Wicked Engine keeps a wetness value per vertex of each object that asks, dries steep faces faster and reads its rain blocker [ref:wicked-engine-fc8385b-wetmap-per-vertex] [ref:wicked-engine-fc8385b-rain-blocker]. Call of Duty: Modern Warfare II dries each object by its material type [ref:activision-2022-mw2-wetness-dries-by-material]; Ghost of Yōtei keeps wet, bloody, muddy and snowy state in a grid deformed around the character [ref:bentley-2025-yotei-directional-grid]. Grand Theft Auto VI's surfaces that "dry at different rates" are a reporter's account [ref:parkin-2026-edge-gta6-weather].

**10. A lobe.** glTF's clearcoat layers a second specular BRDF, with its own roughness and optionally its own normal, over the base [ref:khr-materials-clearcoat-layered-brdf]. OpenPBR's coat darkens the base by light reflected inside the coat, and assumes the other mechanism, water filling a porous base, does not occur [ref:openpbr-1.1.2-coat-darkening-and-porosity]. A 2025 paper models wet porous media as particles in liquid, in an offline renderer [ref:pan-2025-wet-porous-micro-ellipsoid]. The nearest shipped case is not a lobe: wet skin in The Last of Us Part II, where the one specular evaluation reads a normal derived from the wetness mask, in "a different permutation" of its deferred shader, so drips do not read as veins [ref:brinck-2020-tlou2-wet-skin-mask-and-specular-normal].

## Snow lying

**11. Cover.** Torque3D blends one texture over a surface by how far its normal faces up, inside an authored volume, reading no weather value [ref:torque3d-9521d43-accumulation-shader-feature] [ref:torque3d-9521d43-accumulation-volume]. Bevy's decal module lists snow among its uses [ref:bevy-0.20.1-decal-module-names-snow]. No shipped title's account of snow cover on arbitrary Meshes was found.

**12. Trails by relief.** Batman: Arkham Origins renders what touches a snow surface from below through an ankle-high orthographic frustum into a height map per rooftop or street, at most 512 texels a side and two surfaces a frame, filling back in a little each frame; consoles draw it by relief mapping [ref:barre-brisebois-2014-arkham-snow-heightmaps]. The Last of Us Part II draws footprints into a render target per snow mesh and offsets depth in a custom depth pass; the sun's shadow still "is “seeing” the undeformed snow", so a shadow is faked for the sun alone [ref:brinck-2020-tlou2-snow-deformation]. God of War (2018) used screen-space parallax, whose artefacts its successor was built to remove [ref:surricchio-2023-gowr-snow-tessellation].

**13. Trails as geometry.** Batman's deck gives the gain, shadows and occlusion that follow the surface, and the verdict that relief "will never be as thick as tessellation" [ref:barre-brisebois-2014-arkham-snow-limits]. God of War Ragnarök tessellates only the terrain chunks a compute pass finds inside the displaced area, carves with capsules on characters and rigid bodies, and lets particles read the displacement; no figures [ref:surricchio-2023-gowr-snow-tessellation] [ref:surricchio-2023-gowr-snow-carving]. Ghost of Yōtei renders particles and geometry into a displacement buffer over tessellated terrain [ref:bentley-2025-yotei-snow-displacement-buffer]. A WebGPU demo in WGSL does it without tessellation: a clipmap of 8.5 cm cells displaced in the vertex stage from two 2048×2048 trail textures over 80 m, redrawn into its shadow cascades, with TAA in its post chain [ref:snowflow-5450397-deformation-buffer].

**14. Simulation.** A real-time material point method runs 1.33 million snow particles [ref:fei-2021-realtime-mpm-snow]; a 2022 model computes footprints from the force under each foot [ref:alvarado-2022-soft-ground-footprints]. On behaviour, snow depth reaches a horse's animation in Red Dead Redemption 2 [ref:kleanthous-2021-terrain-analysis-blackboard].

How long trails last: Batman frees a height map when its surface is far or unstreamed and lists "Save the heightmaps and reload them?" as future work [ref:barre-brisebois-2014-arkham-snow-limits]; Red Dead Redemption 2's "persist a great distance", by observation [ref:df-2018-rdr2-snow-and-mud-trails-persist]. No source says what holds them.

## Wind

**15. Closed form.** Ghost of Tsushima models wind as a vector, noise and "vorticles" [ref:rockenbeck-2021-got-wind-model]: by a press report of the talk, no fluid dynamics, a magnitude varied by Perlin noise, and up to hundreds of small spheres of wind summed by brute force [ref:rockenbeck-2021-got-wind-press-report]. Particles, trees, grass, cloth and ropes read it, with scrolled gust layers and emitters on sword swings [ref:vainio-2021-got-wind-readers-and-gusts]. Uncharted 4 projects one gust texture top-down [ref:naughtydog-2016-u4-wind-gust-texture]; Crysis sums directional and omnidirectional sources into one vector per instance on the CPU [ref:sousa-2007-crysis-wind-per-instance]; Far Cry 6's weather manager hands one direction and magnitude to clouds, rain, the ocean and trees [ref:zhou-weick-2022-fc6-wind-readers]. Unreal's Dynamic Wind takes a direction, a speed, a noise texture, local force sources, push capsules and velocity volumes, and simulates plants' bones on the GPU [ref:unreal-5.9-dynamic-wind] [ref:unreal-5.9-dynamic-wind-inputs] [ref:unreal-5.9-dynamic-wind-local-forces]; Unity's wind zones are directional or spherical and reach trees and particles [ref:unity-6.6-wind-zone]. What plants do with it is #949's.

**16. A volume around the camera.** God of War adds, between a global vector and each object's motion, a 32×16×32 volume of about a metre a cell that follows the player: diffusion and advection each frame, pressure dropped [ref:renard-2019-gow-wind-three-tiers] [ref:renard-2019-gow-wind-fluid-grid], fed by motors that designers place or attach, which metre cells handle badly when small [ref:renard-2019-gow-wind-motors]. It costs "about one tenth of a millisecond" and 384 KB, hardware not named, run asynchronously [ref:renard-2019-gow-wind-cost]. Hair, leaves, fur and particles read it on the GPU in its frame; audio and cloth, on the CPU, read "the previous frames wind results" [ref:renard-2019-gow-wind-readers]. Scattered writes are summed by integer atomics in 16.16 fixed point, so order does not change the sum [ref:renard-2019-gow-wind-fixed-point-atomic-add]; the solver is Stam's, stable at any step [ref:stam-1999-stable-fluids-unconditionally-stable]. Horizon Zero Dawn's camera-local "Global Wind Force Field" is a compute shader of about 150 µs [ref:sanders-2018-hzd-global-wind-force-field]. Red Dead Redemption 2 scrolls cloud and fog noise by wind and reprojects fog by it, in a frame that relies on TAA [ref:bauer-2019-rdr2-wind-in-volumetrics-and-taa]. No paper since 2019 on real-time wind fields for games was found.

## The weather state

**17. Presets on a schedule.** Far Cry 6's weather manager reads a preset from an hourly forecast, a five-day loop per region, applies regional limits blended over overlapping regions, then mission and cutscene overrides, and integrates wetness, rain and lightning over time; its outputs "drive visuals, audio, gameplay and so on", and it is replicated between players [ref:zhou-weick-2022-fc6-weather-state-flow] [ref:zhou-weick-2022-fc6-presets-and-forecast] [ref:zhou-weick-2022-fc6-regions-overrides-replication]. Horizon Forbidden West selects a cloudscape per weather state and defines a local weather event by "Position, Radius and blend factor" [ref:schneider-2022-nubis-weather-states] [ref:schneider-2022-hfw-local-weather-event]. Red Dead Redemption 2 blends look-up tables across weather transitions and lists localised weather as future work [ref:bauer-2019-time-of-day-weather-cycles-climate-regions]; Ghost of Tsushima relights its probes at run time as weather and time of day change [ref:patry-2021-got-weather-states-and-probes]. Remember Me's control panel ties rain intensity to a splash count, three rise and fall speeds (puddles, cracks, wet level) and fog [ref:lagarde-2013-rain-control-panel]. Wicked Engine's weather component holds wind, rain values, fog and the sky's, clouds' and ocean's parameters, and no snow or wetness value [ref:wicked-engine-fc8385b-weather-component]. Weather simulated from physics exists offline: a 2025 thunderstorm model takes 0.68 s a frame for one scene on an RTX A6000 [ref:hao-2025-thunderscapes-offline-thunderstorm].

What one weather state holds, the union of those sources:

- rain or snow intensity, and a factor for its on-surface effects;
- a wetness level, a puddle level and a snow depth, each with its own rise and fall rate;
- wind direction, magnitude and gust parameters, and the list of local wind sources;
- cloud cover and cloudscape, fog amount, the sky's parameters;
- lightning: whether strikes occur, and each strike's place and Tick;
- where it varies: regional limits, or local events of a position, a radius and a blend.

What reads it, under ADR 0022:

- *In the fixed schedule, as game state:* vehicles' grip (#952; no source read ties grip to weather), a horse's gait in deep snow and people who shelter (`docs/research/living-open-worlds.md`, row 17), wind on whatever the simulation moves, and audio's choice of cues.
- *In prepare, once per frame:* the sky and aerial-perspective tables, which follow a change in their frame, and the cloud map, which lags by its slice count (volumetrics note, Weather); fog density; sun and sky light, and through them the probes (class b, ADR 0020) and exposure; drops (rate, direction, cover); Materials (wetness, puddles, ripples, snow); the wind's renderers: vegetation (#949), drops, cloth and hair, and water (#950), which Far Cry 6 feeds a Beaufort level.

**18. Lightning.** Far Cry 6 places an omni light and lights its clouds in the upsample pass, "so that the lightning doesn’t end up in the history buffer" [ref:zhou-weick-2022-fc6-lightning-omni-light]. Red Dead Redemption 2 adds a point light per strike to its scattering march and leaves strikes out of its sky probes [ref:bauer-2019-rdr2-lightning-point-lights]. Guerrilla animates each flash, several inside the cloud and then one or two to the ground, and masks them because flashes of 4 to 8 frames outrun a 16-frame cloud build [ref:schneider-2022-nubis-lightning-sequence] [ref:schneider-2022-nubis-evolved-lightning-16-frames]. ToyShop baked the flash into lightmaps because shadows for one more light cost too much [ref:tatarchuk-2006-lightning-lightmaps].

## The register's entries

Each read at the register's commit, never built or run; an implementation is evidence, not a template. None holds snow that lies and takes trails; the register names Spartan's tyre tracks and Chrono's deformable soil as the nearest, not read here.

- **Wicked Engine** (MIT: A [ref:wicked-engine-fc8385b-licence]). Rain is one GPU emitter of up to a million particles in a volume ahead of the camera, moved by wind [ref:wicked-engine-fc8385b-rain-emitter]. Its blocker is a 128×128 region of the shadow atlas drawn from straight above whenever it rains [ref:wicked-engine-fc8385b-rain-blocker-draw]; a drop under it becomes a splash, tested against the previous frame's map [ref:wicked-engine-fc8385b-particle-reads-rain-blocker]. Wetness changes inputs only [ref:wicked-engine-fc8385b-wet-shading-inputs]. Wind is a 3D noise texture written each frame from position and time, with no carried state [ref:wicked-engine-fc8385b-wind]. Fit: the nearest whole; its cover lives in the pass Praxis dropped; no snow.
- **Godot** (MIT: A [ref:godot-4.7.2-licence]). GPU particles collide with one depth texture of 256 to 8192 texels a side [ref:godot-4.7.2-height-field-texture], drawn orthographically down a box's axis [ref:godot-4.7.2-height-field-render], one bilinear lookup per particle [ref:godot-4.7.2-particle-height-field-collision]; it can follow the camera, redrawing when the camera leaves the box, and its documentation prices the modes in words only [ref:godot-4.7.2-height-field-follow-camera] [ref:godot-4.7.2-height-field-documented-costs]. Its area wind reaches soft bodies only [ref:godot-4.7.2-wind-soft-body]. Fit: row 3 in a clustered forward renderer.
- **Torque3D** (MIT: A [ref:torque3d-9521d43-licence]). Up to 100,000 drops on the CPU in a box that follows the camera, a ray each [ref:torque3d-9521d43-precipitation-drop-ray-and-count]; wind emitters whose gusts draw on a generator seeded from the real-time clock [ref:torque3d-9521d43-forest-wind-emitter]; accumulation as in row 11. Fit: row 5 on the CPU; the emitter is what a replay cannot match.
- **Spartan Engine** at `cb258a1` (MIT: A). One procedural wind texture a frame for all wind-driven geometry [ref:spartan-engine-cb258a1-wind-field]. Fit: row 15.
- **bevy_hanabi** (MIT OR Apache-2.0: A [ref:bevy-hanabi-738b353-licence]). Compute particles on wgpu 30; collision and the depth buffer are unticked in its feature list [ref:bevy-hanabi-738b353-readme], and its region tests kill particles in a sphere, a box or outside the frustum [ref:bevy-hanabi-738b353-kill-modifiers]. Fit: the stack's nearest particle code; no cover.
- **O3DE** (Apache-2.0 OR MIT: A [ref:o3de-2610.0-licence]). Wind as a query for the wind at a position [ref:o3de-2610.0-wind]. Fit: the interface of a wind the simulation owns.
- **bevy_feronia** (code MIT OR Apache-2.0: A [ref:bevy-feronia-0.8.4-licence]). A global wind resource holding a current and a previous wind [ref:bevy-feronia-0.8.4-global-wind], applied in WGSL after Ghost of Tsushima's grass talk [ref:bevy-feronia-0.8.4-wind], its noise sampled so as not to break motion vectors and TAA [ref:bevy-feronia-0.8.4-wind-noise]. Fit: row 15 in WGSL; the previous wind serves motion vectors Praxis does not write.
- **Bevy** (MIT OR Apache-2.0: A [ref:bevy-0.20.1-licence]). Clustered decals that work in forward rendering [ref:bevy-0.20.1-clustered-decals]. Fit: a puddle or snow decal's form; decals have their own line on the ladder's map.
- **Terrain3D** (MIT: A [ref:terrain3d-1.1.0-licence]). Wetness is a painted roughness offset, darkening painted apart [ref:terrain3d-1.1.0-paint-wetness]. Fit: row 7 with no state.

## What wetness and snow ask of the cooked Material and the lit pass

A cooked Material is five Channels and their factors; normal and metallic-roughness are two-component formats and occlusion one (ADR 0018), so no data Channel has a component to spare; emissive's BC7 alpha is the one component glTF leaves unused. The ladder's map holds, under "Materials past glTF's core", that every new Channel is one more binding array in the Material bind group (ADR 0017) and shrinks the Material table, that every lobe is per-light work at 4x MSAA, that a uniform `if` around one term cost the M4 Pro's main pass 10 to 12% (`docs/performance.md`), and that a pipeline per lobe set adds a `draw_indexed_indirect` per Hi-Z phase.

| Form | Cooked Material | Lit pass | Serves |
|---|---|---|---|
| **An input.** Wetness and snow scalars; porosity from roughness and metallic | Nothing, or two factors per Material as Far Cry 6 has | Arithmetic on albedo, roughness and normal before the BRDF, always evaluated, no branch | Ground, props, cloth, hair; skin without drips |
| **An input with a mask.** An authored porosity, puddle height or drip mask | A sixth Channel: one more binding array, a smaller Material table, one more texture in a Far proxy's baked Material (ADR 0026); or emissive's alpha, with an emissive texture on every masked Material | One more texture read per fragment | The above, with puddles that follow authored relief and skin's drips through a normal derived from the mask |
| **A Render switch.** Either form compiled in or out when the Renderer is made | As the form | Nothing when off; a Game world whose weather changes keeps it on | As the form |
| **A lobe.** A coat with its own roughness and normal | Up to three Channels and factors | A second specular evaluation per light per sample | Standing water over a rough base, lacquer, car paint |

Where the scalars live is a second choice: in the weather state alone (one value for the Game world, masked by cover), or per Instance, written by prepare into the Instance's slot (ADR 0003, 136.2 bytes a slot today) as `docs/research/living-open-worlds.md` asks for characters. Snow cover is the input form with a second albedo and roughness to blend toward; snow under a roof is the cover question again.

## Fit to Praxis

- **ADR 0019, forward 4x MSAA.** Transparency is "a sorted forward pass after the opaque pass, sharing the Froxel lists, MSAA and the opaque depth"; it is not built, and the ladder's map revives particles "when content first needs them". Rain is that content. A streak narrower than a pixel is what a Thin Primitive solves for opaque tubes; for a blended streak the equivalent, width held at a pixel and opacity scaled down, is unmeasured.
- **ADR 0008.** Rows 1 to 9, 11, 12, 15 and 17 are complete each frame or hold state outside the screen. The pre-mortem's line stands: particles lose temporal upsampling and keep soft depth fades.
- **ADR 0020's rays.** The per-ray figure is from coherent rays of neighbouring pixels; rays from scattered drops through `town`'s or `world`'s TLAS are unmeasured, and no figure exists on the RTX 2070 SUPER.
- **ADR 0022.** A replay has to match the weather state and the wind, and whatever game state reads from them. Drop positions, ripple phase and a GPU wind volume need not, unless a Golden holds them, in which case they are functions of the injected time. That favours drops whose position is a function of time and a seed over drops integrated frame by frame. Three entries break the rule as written: Wicked's emitter steps by frame time, Torque3D's wind seeds from the clock, and God of War's CPU readers take a GPU result a frame late, the read-back the register found in five oceans.
- **One queue.** God of War's wind runs asynchronously; wgpu exposes one queue [ref:wgpu-hal-30-metal-one-queue], so its 0.1 ms would land in the frame.
- **No tessellation.** wgpu 30's shader stages are vertex, fragment, compute, task, mesh and the ray tracing stages [ref:wgpu-30-shader-stages]. Geometry trails would be vertex-stage displacement of a grid dense near the camera, terrain's to offer, or a Mesh a compute pass rewrites, as the Bend does.
- **Rays and trails.** A trail drawn by relief or displaced at draw time leaves the BLAS flat: shadow and probe rays see undeformed snow, as The Last of Us Part II's shadow map did. A trail rays see is a BLAS rebuilt as it changes, which the pre-mortem prices at 7.8 to 9.3 ms of M4 Pro GPU for a hundred BLASes of 5,120 triangles.
- **What terrain (#948) would offer snow:** a height query, so a carving shape knows where hard ground lies; a displacement input in its vertex stage and vertices enough near the camera to show it; a world-space mapping for a trail texture that follows the camera; and its answer to rays, flat or rebuilt.
- **ADR 0013.** The GPU frame's existing shares sum to 9 of its 12 ms (`docs/audit/strategy/2026-11.md`, S3-04). The published figures are from 2012 to 2019 consoles, none at 2560×1440 or on Praxis's adapters; no conversion is offered.

## Cost of delay

| Option | Rebuilds | Adds beside |
|---|---|---|
| Drops, either form, and the transparent pass | Nothing | A pass ADR 0019 names; a particle buffer in the GPU memory ledger |
| Cover by a depth map | Nothing | A second view through culling and its draw |
| Cover by a cooked height | Nothing | A field per Chunk; a re-cook |
| Cover by rays | Nothing | Rays from the frame's unallocated share, beside M6's probe rays |
| Wetness or snow as inputs | Nothing, if M6's probe rays shade a hit through the function the lit pass uses for a Material's inputs; otherwise a wet street bounces dry light | Scalars in the weather state or the Instance slot |
| A sixth Channel | The Material bind group (ADR 0017), the Material table's size, what M5's Far proxy bake writes (ADR 0026) | |
| A mask in emissive's alpha | Nothing | An emissive texture on every masked Material; the cook's emissive alpha |
| A lobe | The Material record and cooked format, and per-light work in the lit pass M6 adds probes to | |
| Trails as a texture | Nothing | A texture that follows the camera |
| Trails as geometry rays see | BLASes static per Mesh, and what the static TLAS holds | |
| Wind, closed form or with a volume | Nothing | A resource of the fixed schedule; a small 3D texture |
| The weather state | Nothing | A resource of the fixed schedule |
| Lightning | Nothing, if a light can be kept out of the probe update, which M6 decides | One more light for a few frames |

Two items are cheap before M6 is charted and dearer after: that a probe ray's hit and the lit pass apply a Material's inputs through one function, and that a light can be marked as not feeding probes. Neither moves work ahead of M6.

## Options

Falling:

- **R1. Stateless streaks**: drops whose position is a function of time and a seed, recycled in a volume around the camera, drawn in the transparent pass. Replays and Goldens hold by construction. Open: draw cost and stability at 4x MSAA.
- **R2. Simulated particles** (row 1): collisions, bounces and splashes from state. Open: the same, and a fixed step for Goldens.
- **R3. Streak textures on a cylinder** (row 2): a cost that does not rise with the storm; no parallax between drops.

Cover:

- **C1. A depth map of its own** (row 3): drops, splashes and wet shading from one lookup, moving geometry included if drawn each frame.
- **C2. A cooked height per Chunk** (row 4): the same lookup for static geometry with no pass; no source ships it.
- **C3. Rays** (row 5): a ray per drop, and one every few frames per Moving Instance for its wetness; exact under slanted rain.
- **C4. Authored**: Interior volumes as cover; eaves and balconies stay wet.
- C3 for what moves beside C1 or C2 for surfaces is Far Cry 6's split, with its shadow atlas where C1 or C2 stand.

Wetness and snow cover:

- **W1. Inputs**, porosity from roughness: ground, cloth and hair; skin without drips.
- **W2. Inputs with a mask**, a sixth Channel or emissive's alpha: adds authored puddles and skin's drips.
- **W3. A lobe**, with the survey of material models the ladder's map already asks for.
- Each with scalars in the weather state alone, or per Instance.

Trails:

- **S1. None** until terrain is charted.
- **S2. A trail texture drawn by relief** (row 12): rays see flat snow.
- **S3. A trail texture displacing terrain's vertices** (row 13): needs terrain's grid; rays see flat snow unless a BLAS is rebuilt.

Wind:

- **N1. Closed form**: a vector, scrolled noise and a list of local sources, the same function on the CPU per Tick and on the GPU per frame.
- **N2. N1 with a GPU volume** for presentation (row 16), read by drops, grass, hair and cloth, never by game state.
- **N3. A fluid volume as game state**, stepped per Tick on the CPU.

Weather state:

- **T1. Global scalars** from blended presets.
- **T2. T1 with local events** of a position, a radius and a blend.
- **T3. A field over the Game world.**

## Prototype questions

Costs no source publishes, each to be measured on the M4 Pro and the RTX 2070 SUPER from one engine source (ADR 0013).

1. **Cover three ways.** A 128×128 to 256×256 depth-only draw from above of the Instances within 30 m, a cooked height sampled in its place, and one ray per drop for 100,000 and 320,000 drops through the static and dynamic TLAS: GPU time for each, and what each leaks under an eave and a moving roof. On `town`.
2. **Streaks at 4x MSAA with no history.** 100,000 to 320,000 blended streaks in a forward transparent pass at 2560×1440, lit from one ambient value against lit per vertex, and whether streaks under a pixel wide shimmer as the camera moves. On `town`.
3. **Wetness as inputs.** What the always-evaluated form adds to the lit pass at 2560×1440 and 4x MSAA against the form compiled out, on `pbr-sweep` and `town`; and per Instance on `crowd`, with the living-world note's fifth prototype question.
4. **A flash against probes.** A directional light on for 4 to 8 frames: what the probes show during and after it, with the light feeding them and kept out. On `town`, once M6's probes exist.
5. **Trails.** A camera-centred trail texture drawn by relief against the same texture displacing a grid: lit-pass time, and what a shadow ray shows at a footprint's rim. It waits for the world-content Reference scene #944 has not chosen; `world`'s flat ground stands in.

A wind volume's cost is published (about 0.1 ms and about 150 µs, hardware not named) and small beside the others; no prototype is asked for it.

## Not verified

1. The hardware and resolution of Assassin's Creed 4's rain, God of War's wind and Horizon Zero Dawn's wind timings, and Remember Me's console resolution: the sources do not say.
2. Whether any shipped rain or snow relies on TAA: no source read says; Far Cry 6's deck credits TAA for its clouds only.
3. Not obtained: the GDC 2023 talk on God of War Ragnarök's snow (an interview stands in), Rise of the Tomb Raider's snow chapter in GPU Pro 7, any talk on Horizon Zero Dawn: The Frozen Wilds' snow, the speaker notes of both Ghost of Tsushima wind talks (one known through a press report), Feeley's God of War vegetation talk, Fearing's and Ohlsson and Seipel's papers on fallen snow, and a 2021 Charles University thesis on snow deformation, whose site did not answer.
4. Research since 2019: arXiv was queried for rain, snow, wet materials and wind, and web searches for real-time rain and snow talks found nothing newer; SIGGRAPH, EGSR, HPG, I3D and JCGT were not searched issue by issue. "No paper found" means that and no more.
5. Of snowflow_demo only the readme and licence were read, its figures its author's; of the footprints repository only the licence.
6. Unreal's Dynamic Wind page gives no timing; no Unreal, Unity or Flax page on rain, snow or wetness was found, as the register reported.
7. Every figure computed from the repo's per-ray cost is a lower bound from coherent per-pixel rays on one scene and one adapter.
