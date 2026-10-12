# Test models and resources: sources, licenses, fit

Researched 2026-10-12 against each source's own pages. Licenses are quoted as the source states them. "Unverified" marks claims the source page did not confirm.

## What the repo ships today

| File | Provenance | License | Referenced in `src/` |
|---|---|---|---|
| `assets/DamagedHelmet.glb` | SHA-256 `a1e3b04de97b11de564ce6e53b95f02954a297f0008183ac63a4f5974f6b32d8` equals Khronos `glTF-Sample-Assets` `main` | CC BY 4.0, © 2018 ctxwing (rebuild + glTF conversion). The 2016 original by theblueturtle_ was CC BY-NC 4.0 ([README](https://github.com/KhronosGroup/glTF-Sample-Assets/blob/main/Models/DamagedHelmet/README.md)) | yes |
| `assets/BoxTextured.gltf`, `BoxTextured0.bin`, `CesiumLogoFlat.png` | Khronos BoxTextured (Cesium), added in #10 | CC BY 4.0; the logo carries a Cesium trademark mark ([README](https://github.com/KhronosGroup/glTF-Sample-Assets/blob/main/Models/BoxTextured/README.md)) | no |
| `assets/pikuma.png` | unknown, added in #9 | unknown | no |

No attribution file exists. CC BY requires attribution wherever the file is redistributed, and this repo is public.

## Sources

| Source | Offers | License as stated | Notes |
|---|---|---|---|
| [Khronos glTF-Sample-Assets](https://github.com/KhronosGroup/glTF-Sample-Assets) | Curated feature and test models; lists `Models-pbrtest.md`, `Models-testing.md` | Per model, in each model's README. "Assets without licenses or licenses that were not suitable ... were removed." | Best conformance set. Verified below. |
| [Poly Haven](https://polyhaven.com/license) | HDRIs, PBR textures, models | CC0. Commercial use and redistribution allowed, no credit needed | ToS bars "web scraping or data mining without express permission". Using the live [API](https://github.com/Poly-Haven/Public-API) requires a "Powered by Poly Haven" credit |
| [ambientCG](https://docs.ambientcg.com/license/) | PBR texture sets | CC0 1.0. "You can include the raw files in your project, for example a video game." | |
| [Kenney](https://kenney.nl/support) | Low-poly props, kits | CC0, commercial use allowed | Stylised; useful for instancing volume, not for PBR realism |
| [NVIDIA ORCA: Amazon Lumberyard Bistro](https://developer.nvidia.com/orca/amazon-lumberyard-bistro) | Interior 1,046,609 tris (1,293,691 with wine), exterior 2,832,120 tris | CC BY 4.0; cite "Amazon Lumberyard Bistro, Open Research Content Archive (ORCA)", Amazon Lumberyard, 2017 | FBX and Falcor scene only; glTF needs a conversion step |
| [Intel Sponza](https://www.intel.com/content/www/us/en/developer/topic-technology/graphics-research/samples.html) | Base scene 3.71 GB, 4K textures, glTF/USD/FBX; add-ons curtains 750 MB, ivy 1.29 GB, trees 907 MB, emissive candles 2.97 GB | "Creative Commons Attribution license" (version not stated on the page) | Large; fetch-only |
| [IGN / CNIG MDT05](https://inspire-geoportal.ec.europa.eu/srv/api/records/spaignMDT05) | Spain terrain model, 5 m grid; download at [CNIG Centro de Descargas](https://centrodedescargas.cnig.es); WCS at `servicios.idee.es/wcs-inspire/mdt` | CC BY 4.0 SCNE (INSPIRE record's use constraint) | Real terrain for streaming and terrain tests. MDT02 licence unverified |

### Khronos models by licence (read from each model's README, 2026-10-12)

- CC0: `MetalRoughSpheresNoTextures`, `NormalTangentTest`, `TextureTransformTest`, `TextureCoordinateTest`, `SimpleInstancing`, and the `Compare*` PBR sweeps (`CompareMetallic`, `CompareRoughness`, `CompareNormal`, `CompareBaseColor`, `CompareAmbientOcclusion`; each also carries a Khronos trademark mark on its logo).
- CC BY 4.0: `DamagedHelmet`, `MetalRoughSpheres`, `NormalTangentMirrorTest`, `TextureSettingsTest`, `EmissiveStrengthTest`, `OrientationTest`, `MultiUVTest`, `BoxTextured`.
- **Not CC:** `Sponza` is © 2016 Crytek under the "Cryengine Limited License Agreement" ([README](https://github.com/KhronosGroup/glTF-Sample-Assets/blob/main/Models/Sponza/README.md)). `ROADMAP.md` lists Sponza as a good test model; this copy is not freely redistributable.

## Reference tools (independent oracles)

- [glTF-IBL-Sampler](https://github.com/KhronosGroup/glTF-IBL-Sampler), Apache-2.0: HDR panorama in; Lambertian diffuse cube map, GGX specular cube map with mips, and BRDF LUT out. Its KTX2 output can be basis-compressed (lossy); compare against an uncompressed format and match sample count, resolution, and LOD bias.
- [glTF-Sample-Renderer](https://github.com/KhronosGroup/glTF-Sample-Renderer), Apache-2.0: Khronos' official WebGL2 renderer, used in Google's Render Fidelity Tools. No headless mode documented; a headless browser would drive it (unverified).
- [glTF-Asset-Generator](https://github.com/KhronosGroup/glTF-Asset-Generator), MIT: C# project generating importer conformance models (Accessor, Animation, Buffer, Material, Mesh, Node, Texture sampler). Whether it emits expected-result images is unverified.

## Not verified

McGuire Computer Graphics Archive (page content not retrievable), Fab/Quixel Megascans, Sketchfab, Blender demo files, glTF-Validator.

## Proposed sourcing strategy (owner decision)

1. **Tier 1, procedural and in-repo.** Analytic spheres, UV grids, normal-map patterns, instancing grids, seeded scatter, heightfields. No licence, deterministic, tiny. These are the CI-gated golden and scale-ladder scenes.
2. **Tier 2, small curated set, committed.** The CC0/CC BY Khronos models above, one or two CC0 HDRIs and PBR texture sets (Poly Haven, ambientCG), each with a manifest entry. Per-model size is checked in the ticket that adds it.
3. **Tier 3, large realism and stress scenes, fetched.** Intel Sponza, ORCA Bistro (after glTF conversion), IGN terrain tiles. Pinned URL plus SHA-256, into a git-ignored cache; never committed. Used by local benches and milestone-exit visual checks.
4. **Manifest and hygiene.** One manifest records path, source URL, SPDX licence id, credit text, upstream version, and SHA-256. CI fails on an `assets/` file without an entry. Credits are collected in an attribution file.
5. **Cleanup.** Remove unreferenced `pikuma.png`, `BoxTextured*`, and `CesiumLogoFlat.png` unless a test adopts them.
