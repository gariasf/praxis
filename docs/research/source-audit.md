# Source audit

The Review's Sources axis (`docs/agents/workflow.md`, Review) applied to what merged before every PR's review checked its sources, one table per area. Each reference gets four verdicts: it **exists** (opens at the cited URL, section, commit or version), it **says that** (a primary or trusted source stating what the tree claims), it is **cited right** (the right place, quoted or paraphrased faithfully), and it is **implemented right** (formula, constants, units, edge cases). ✓ passes, ✗ failed and is fixed in the audit's PR, – does not apply. Evidence is the registry entry (`docs/references.toml`), which holds each source's pinned URL, section and verbatim quote; every source was opened for this audit, none vouched from memory.

## Shading, lighting and colour

Audited 2026-10-26 (#305): the shading functions in `src/render/shaders/` and their oracles in `src/render/shader_function_tests.rs`, photometric units and exposure (ADR 0023, `src/camera/mod.rs`, `src/scenes/mod.rs`), the HDR, GT7 and display parts of the tonemapping pass, normal mapping in the lit pass, and ADR 0023's own claims. The environment module cites no source. ADR 0008 and ADR 0020 cite only decisions on this repo's issues; ADR 0020's ray-traced shadows and their measurements are #306's and #308's. The remaining links of the two lighting notes, their survey rows and the shipped-practice links of the HDR note's rows 11, 14 and 15, split off along the ticket size ceiling into #329.

One mismatch changes rendered output and is ticketed: #327, glTF's normal texture scale.

| Reference | Cited in | Exists | Says that | Cited right | Implemented right | Evidence, and the fix where one failed |
|---|---|---|---|---|---|---|
| glTF 2.0 source, Appendix B, Specular BRDF | `shading.wgsl` D and V; oracles | ✓ | ✓ | ✓ | ✓ | [ref:gltf-2.0-specular-brdf]; D's denominator regrouped exactly; V's 0 at a zero denominator is the shader's own, and says so |
| glTF 2.0 source, α = roughness² | oracle of D's peak | ✓ | ✓ | ✗ | ✓ | [ref:gltf-2.0-roughness-mapping]; the mapping was attributed to Walter et al., which has none: now cited to Appendix B |
| glTF 2.0 source, α = 0 note | `MINIMUM_ROUGHNESS` | ✓ | ✗ | ✓ | ✓ | [ref:gltf-2.0-zero-roughness]; it forbids α = 0 rather than "dividing by zero": reworded to say so |
| glTF 2.0 source, Fresnel | `fresnel_weight` | ✓ | ✓ | ✓ | ✓ | [ref:gltf-2.0-fresnel]; multiplied out in place of `pow`, with `abs` as the source |
| glTF 2.0 source, Metal BRDF and Dielectric BRDF | `metallic_roughness_brdf`, F0 0.04; `reference_brdf` | ✓ | ✓ | ✓ | ✓ | [ref:gltf-2.0-metal-and-dielectric-brdf]; the shader's one-lobe mix expands to the source's exactly, Schlick's F being linear in F0 |
| glTF 2.0 source, Coupling Diffuse and Specular Reflection | white-furnace test | ✓ | ✓ | ✓ | ✓ | [ref:gltf-2.0-coupling]; exempts the dielectric for the reason the source gives |
| glTF 2.0.1 (published), B.3.2 and B.3.5 | `shading.wgsl` header | ✓ | ✓ | ✓ | – | [ref:gltf-2.0.1-sample-brdf]: separable Smith G and the simplified mix, as claimed |
| glTF 2.0 source, Metallic-Roughness Material | `decode_surface` | ✓ | ✓ | ✓ | ✓ | [ref:gltf-2.0-metallic-roughness]; factors multiply texels; the BC5 swizzle is ADR 0018's, and the Rust side matches it |
| glTF 2.0 source, occlusion | `ambient_luminance` | ✓ | ✓ | ✓ | ✓ | [ref:gltf-2.0-occlusion]; occlusion darkens the ambient term only |
| glTF 2.0 source, emissive units; schema's `emissiveFactor` | ADR 0023 | ✓ | ✓ | ✓ | – | [ref:gltf-2.0-emissive-units], [ref:gltf-2.0-emissive-factor-schema]: the cap of 1 is the schema's, not the prose's |
| KHR_lights_punctual, intensity units | ADR 0023, `uniforms.rs` | ✓ | ✓ | ✓ | ✓ | [ref:khr-lights-punctual-units] |
| KHR_lights_punctual, range window | `point_light_attenuation`; its oracle | ✓ | ✓ | ✓ | ✓ | [ref:khr-lights-punctual-range]; (d²/r²)² is (d/r)⁴ |
| Walter et al. 2007, eq. 4 | normalisation oracle | ✓ | ✓ | ✓ | ✓ | [ref:walter-2007-projected-area]; the oracle's integral is eq. 4 in spherical coordinates |
| Walter et al. 2007, eq. 33 | D's peak oracle | ✓ | ✓ | ✓ | ✓ | [ref:walter-2007-ggx]; 1/(π α²) at θm = 0 |
| glTF 2.0 source, Meshes (bitangent) and Normal Texture | tangent frame in `main.wgsl`; `tangent_space_normal` | ✓ | ✓ | ✓ | ✗ | [ref:gltf-2.0-bitangent]: the bitangent is cross(normal, tangent.xyz) · tangent.w, as glTF writes it; z is rebuilt from x and y, which BC5 alone stores (ADR 0018). glTF multiplies x and y by `normalTexture.scale` (L2769-L2776), which the cooker never reads: #327, as the fix changes rendered output |
| Filament, 1 cm punctual light | `MINIMUM_LIGHT_DISTANCE_SQUARED` | ✓ | ✓ | ✓ | ✓ | [ref:filament-punctual-light-1cm], Apache-2.0; Filament takes its range window from the unclamped d², the shader from the clamped one, which differ only within 1 cm, where the window is 1 |
| Hoffman 2013, Table 1 (gold) | `SWEEP_BASE_COLOR`, `GOLD` | ✓ | ✓ | ✗ | ✓ | [ref:hoffman-2013-f0]; gold's (1.00, 0.71, 0.29) had no source: now cited |
| GT7, `gt7_tone_mapping.cpp` 1.0 | `gt7.wgsl`; GT7 oracles | ✓ | ✓ | ✓ | ✓ | [ref:gt7-tone-mapping-cpp]; MIT notice verbatim; every constant, matrix and step matches; SHA-256 matches the oracle's |
| GT7 slides, SIGGRAPH 2025 | `gt7.wgsl` paper white | ✓ | ✓ | ✓ | ✓ | [ref:gt7-slides], slide 153: 250 nits, 1.0 = 100 nits, ×1/2.5 after |
| ITU-R BT.2100-3, Table 4 (PQ) | `gt7.wgsl` PQ; PQ oracle | ✓ | ✓ | ✓ | ✓ | [ref:bt2100-pq]; ST 2084's constants as BT.2100 reproduces them; 100 cd/m² encodes to 0.5081 |
| ICtCp, cited as ITU-T T.302 | `rec2020_to_ictcp`; ICtCp oracle | ✓ | ✗ | ✗ | ✓ | T.302 could not be shown to define ICtCp (inherited from the sample's comment); the matrices are BT.2100-3's Table 7, [ref:bt2100-ictcp]: citation replaced; the inverse is the sample's six-digit one, now said |
| ITU-R BT.2087-0, M2 | `REC709_TO_REC2020` | ✓ | ✓ | ✓ | ✓ | [ref:bt2087-rec709-to-rec2020]; both matrices recomputed exactly from [ref:bt709-6-primaries] and [ref:bt2020-2-primaries], matching to 7 places |
| ITU-R BT.709-6, item 3.2 | `LUMINANCE_COEFFICIENTS`; its oracle | ✓ | ✗ | ✓ | ✓ | [ref:bt709-6-luma]; item 3.2 weighs non-linear E′ (luma); the same weights are the Y row of the linear RGB-to-XYZ matrix BT.2087-0 prints, [ref:bt2087-rec709-to-xyz], which the comment now names |
| IEC 61966-2-1 via Khronos Data Format 1.3, §13.3.2 | `srgb_encode`; its oracle | ✓ | ✓ | ✓ | ✓ | [ref:khronos-dfs-srgb]; IEC's text is paywalled, the Data Format Specification restates it; ≤ at the knee as there |
| WGSL, f16 extreme values | `LARGEST_HALF` | ✓ | ✓ | ✓ | ✓ | [ref:wgsl-f16-largest] |
| Vulkan, Floating-Point Format Conversions | `hdr.wgsl`, ADR 0023 | ✓ | ✓ | ✓ | ✓ | [ref:vulkan-fp-conversion]: overflow may round to infinity |
| Frostbite 2014, §5.1.2 exposure | `camera/mod.rs`; ADR 0023 | ✓ | ✓ | ✗ | ✓ | [ref:frostbite-2014-exposure]; q is "lens and vignetting attenuation" (p. 85); the listing ends on p. 87: "p. 86" became "pp. 86-87"; Sunny 16 at 1/125 s is EV100 14.97 |
| Frostbite 2014, p. 95; Table 8, p. 29; p. 91 | ADR 0023 | ✓ | ✓ | ✓ | – | [ref:frostbite-2014-relative-lights], [ref:frostbite-2014-light-units], [ref:frostbite-2014-fp16] |
| Unreal, Using Physical Lighting Units | ADR 0023 | ✓ | ✗ | ✓ | – | [ref:unreal-scene-color-overflow]; it blames black patches on an overflowing colour buffer, not on infinity: ADR 0023 and the HDR note's row 15 now say that |
| Filament, Bevy and Godot light units | ADR 0023 | ✓ | ✓ | ✗ | – | [ref:filament-light-units], [ref:bevy-point-light-lumens], [ref:godot-light-units]; Godot's are opt-in, which the ADR left out: added |
| Bevy, glTF loader ×4π | ADR 0023 | ✓ | ✓ | ✓ | – | [ref:bevy-gltf-point-candela] |
| Bevy, `lux` constants | ADR 0023, `SUN_ILLUMINANCE` | ✓ | ✓ | ✓ | ✓ | [ref:bevy-lux-constants]; log₂(10⁵ / 10⁻⁴) = 29.9 stops; the note's link now spans the whole module |
| glTF-Blender-IO, light export | ADR 0023 | ✓ | ✓ | ✗ | – | [ref:blender-gltf-light-export]; spots convert as points and both take 2^exposure, which the ADR left out: added |
| Karis 2013, Tone mapping | `resolve_weighted`; resolve oracle | ✓ | ✓ | ✗ | ✓ | [ref:karis-2013-weighted-average]; any luminance of the colour; linear BT.709 luminance keeps the inverse exact; the note's link moved to https |
| Peters, Free blue noise textures | `dither_noise`, `DITHER_MASK` | ✓ | ✓ | ✓ | ✓ | [ref:peters-blue-noise], CC0; `LDR_LLL1_0.png` byte-identical to the archive's; bin centres (k + 0.5)/256 keep the noise inside (−1, 1), as the comment says |
| Lipshitz, Wannamaker and Vanderkooy 1992, p. 372 | `dither_noise` | ✓ | ✗ | ✓ | ✓ | [ref:lipshitz-1992-triangular-dither]; "least" is the least added noise among dithers that remove noise modulation, not the least amplitude: reworded |
| Higham 2002, §3.1; Vulkan precision table | horizon tolerance in the white-furnace test | ✓ | ✓ | ✓ | ✗ | Deferred from #314: the bound took u = 2⁻²⁴, round-to-nearest. D3D rounds add and multiply to 0.5 ULP, [ref:d3d11.3-fp32-rules]; Metal's fast math, the default wgpu-hal leaves, [ref:wgpu-hal-30-metal-compile-options], rounds them and fused multiply-adds correctly but may reorder them, [ref:msl-4.1-fast-math], which Higham's any-order bound covers, and may round toward zero, [ref:msl-4.1-rounding]: u is now 2⁻²³ |
| Computed: peak glint 1.24×10⁸ cd/m², 10.9 stops past 65504; EV100 16.35; 2.5×10⁻⁵ of display white | ADR 0023, `scenes/mod.rs` | – | – | – | ✓ | 0.04 · 1/(π · 0.0016²) · ¼ · 10⁵; log₂(10⁵ / 1.2); 1/(1.2 · 2¹⁵) |
