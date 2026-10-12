# Test models and resources: sources, licenses, fit

Researched 2026-10-12 against each source's own pages. Licenses are quoted as the source states them. "Unverified" marks claims the source page did not confirm. For Khronos models the authority is the repo's machine-readable [`REUSE.toml`](https://github.com/KhronosGroup/glTF-Sample-Assets/blob/main/REUSE.toml) (`precedence = "aggregate"`), which the per-model READMEs agree with.

## What the repo ships today

| File | Provenance | License | Referenced in `src/` |
|---|---|---|---|
| `assets/DamagedHelmet.glb` | SHA-256 `a1e3b04de97b11de564ce6e53b95f02954a297f0008183ac63a4f5974f6b32d8` equals Khronos `glTF-Sample-Assets` `main` | `CC-BY-4.0 AND CC-BY-NC-4.0`: both apply. © 2018 ctxwing (rebuild + glTF conversion) is CC BY 4.0; © 2016 theblueturtle_ (the original) is CC BY-NC 4.0 ([README](https://github.com/KhronosGroup/glTF-Sample-Assets/blob/main/Models/DamagedHelmet/README.md)). Treat as non-commercial | yes |
| `assets/BoxTextured.gltf`, `BoxTextured0.bin`, `CesiumLogoFlat.png` | Khronos BoxTextured (Cesium), added in #10 | `CC-BY-4.0 AND LicenseRef-LegalMark-Cesium`: the logo carries a Cesium trademark mark ([README](https://github.com/KhronosGroup/glTF-Sample-Assets/blob/main/Models/BoxTextured/README.md)) | no |
| `assets/pikuma.png` | unknown, added in #9 | unknown | no |

No attribution file exists. CC BY requires attribution wherever the file is redistributed, and this repo is public. The helmet is also non-commercial: fine as a local reference, never in a shipped build, and not a basis for CI goldens in a repo whose licence allows commercial use.

## Sources

| Source | Offers | License as stated | Notes |
|---|---|---|---|
| [Khronos glTF-Sample-Assets](https://github.com/KhronosGroup/glTF-Sample-Assets) | Curated feature and test models; lists `Models-pbrtest.md`, `Models-testing.md` | Per model, in `REUSE.toml` and each model's README. "Assets without licenses or licenses that were not suitable ... were removed." | Best conformance set. Verified below. |
| [Poly Haven](https://polyhaven.com/license) | HDRIs, PBR textures, models | CC0. Commercial use and redistribution allowed, no credit needed | ToS bars "web scraping or data mining without express permission". The live [API](https://polyhaven.com/our-api) needs a clear credit naming Poly Haven ("Powered by Poly Haven" is an example, and the condition is of the API service, not of the assets) and a unique `User-Agent` on every request; no rate limit stated |
| [ambientCG](https://docs.ambientcg.com/license/) | PBR texture sets | CC0 1.0. "You can include the raw files in your project, for example a video game." | |
| [Kenney](https://kenney.nl/support) | Low-poly props, kits | CC0, commercial use allowed | Stylised; useful for instancing volume, not for PBR realism |
| [NVIDIA ORCA: Amazon Lumberyard Bistro](https://developer.nvidia.com/orca/amazon-lumberyard-bistro) | Interior 1,046,609 tris (1,293,691 with wine), exterior 2,832,120 tris | CC BY 4.0; cite "Amazon Lumberyard Bistro, Open Research Content Archive (ORCA)", Amazon Lumberyard, 2017 | FBX and Falcor scene only; glTF needs a conversion step |
| [Intel Sponza](https://www.intel.com/content/www/us/en/developer/topic-technology/graphics-research/samples.html) | Base scene 3.71 GB, 4K textures, glTF/USD/FBX; add-ons curtains 750 MB, ivy 1.29 GB, trees 907 MB, emissive candles 2.97 GB | "Downloads are available under the Creative Commons Attribution license." No version is named and the page links no licence text | **Unresolved licence version.** Large; fetch-only |
| [IGN / CNIG MDT05](https://inspire-geoportal.ec.europa.eu/srv/api/records/spaignMDT05) | Spain terrain model, 5 m grid; download at [CNIG Centro de Descargas](https://centrodedescargas.cnig.es); WCS at `servicios.idee.es/wcs-inspire/mdt` | CC BY 4.0 SCNE (INSPIRE record's use constraint) | Real terrain for streaming and terrain tests. MDT02 licence unverified |

### Khronos models by SPDX id (`REUSE.toml`, 2026-10-12)

- `CC0-1.0`: `WaterBottle`, `BoomBox`, `FlightHelmet`, `Lantern`, `SciFiHelmet`, `MetalRoughSpheresNoTextures`, `NormalTangentTest`, `TextureTransformTest`, `SimpleInstancing`. The `Compare*` PBR sweeps are `CC0-1.0` with a Khronos trademark mark on the logo (`CompareMetallic` checked in `REUSE.toml`; Roughness, Normal, BaseColor and AmbientOcclusion in their READMEs).
- `CC-BY-4.0`: `MetalRoughSpheres`, `NormalTangentMirrorTest`, `EmissiveStrengthTest`; `TextureSettingsTest`, `OrientationTest` and `MultiUVTest` per their READMEs.
- **Encumbered:** `DamagedHelmet` is `CC-BY-4.0 AND CC-BY-NC-4.0`. `Sponza` is `LicenseRef-CRYENGINE-Agreement`: © 2016 Crytek, "Cryengine Limited License Agreement" ([README](https://github.com/KhronosGroup/glTF-Sample-Assets/blob/main/Models/Sponza/README.md)). `ROADMAP.md` lists Sponza as a good test model; this copy is not freely redistributable.
- **Sizes** (GLB, from the GitHub contents API): `WaterBottle` 8.97 MB, `BoomBox` 10.6 MB, `Lantern` 9.56 MB, `MetalRoughSpheres` 11.2 MB, `NormalTangentTest` 1.80 MB. `SciFiHelmet` and `FlightHelmet` have no binary variant in the repo; the current helmet is 3.77 MB.

## Reference tools (independent oracles)

- [glTF-IBL-Sampler](https://github.com/KhronosGroup/glTF-IBL-Sampler), Apache-2.0: HDR panorama in; Lambertian diffuse cube map, GGX specular cube map with mips, and BRDF LUT out. Its KTX2 output can be basis-compressed (lossy); compare against an uncompressed format and match sample count, resolution, and LOD bias.
- [glTF-Sample-Renderer](https://github.com/KhronosGroup/glTF-Sample-Renderer), Apache-2.0: Khronos' official WebGL2 renderer, used in Google's Render Fidelity Tools. No headless mode documented; a headless browser would drive it (unverified).
- [glTF-Asset-Generator](https://github.com/KhronosGroup/glTF-Asset-Generator), MIT: C# project generating importer conformance models (Accessor, Animation, Buffer, Material, Mesh, Node, Texture sampler). Whether it emits expected-result images is unverified.

## Not verified

McGuire Computer Graphics Archive (page content not retrievable), Fab/Quixel Megascans, Sketchfab, Blender demo files, glTF-Validator.

## Proposed sourcing strategy (owner decision)

1. **Tier 1, procedural and in-repo.** Analytic spheres, UV grids, normal-map patterns, instancing grids, seeded scatter, heightfields. No licence, deterministic, tiny. These are the CI-gated golden and scale-ladder scenes.
2. **Tier 2, small curated set.** `CC0-1.0` and `CC-BY-4.0` Khronos models, one or two CC0 HDRIs and PBR texture sets (Poly Haven, ambientCG), each with a manifest entry. Commit a file only when it is about 2 MB or less (`NormalTangentTest` qualifies); the roughly 10 MB PBR models are fetched like Tier 3, so clones stay light. Per-model size is checked in the ticket that adds it.
3. **Tier 3, large realism and stress scenes, fetched.** ORCA Bistro (after glTF conversion), IGN terrain tiles, and the roughly 10 MB PBR models. Pinned URL plus SHA-256, into a git-ignored cache; never committed. Used by local benches and milestone-exit visual checks. Intel Sponza stays unresolved: usable locally for experiments, never added to the manifest or redistributed until the package or Intel names the licence version.
4. **Manifest and hygiene.** One manifest records path, source URL, SPDX licence expression, credit text, upstream version, and SHA-256, for committed and fetched files alike. CI fails on an `assets/` file without an entry. Credits are collected in an attribution file. Files whose expression includes an `NC` or `LicenseRef-` term are marked dev-only.
5. **Cleanup.** Remove unreferenced `pikuma.png`, `BoxTextured*`, and `CesiumLogoFlat.png` unless a test adopts them.
6. **Decide: the default test model.** `DamagedHelmet.glb` is the only mesh the app loads and it is non-commercial. A `CC0-1.0` helmet-class replacement (`SciFiHelmet`, `FlightHelmet`) would make the goldens redistributable. Changing it moves the Step 4 checkpoint basis, so it is a ticket with the owner's decision.
