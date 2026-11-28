# Deviations from the literature

Where the code should not match its source, and why, so that an audit's
reader or a review files no Finding against code that is right on purpose.
One row per Deviation, each at a code site. The image half of the same
list is `scripts/cycles-reference/DIFFERENCES.md`: what a Cycles
difference image shows that is not a defect. The audit skill
(`docs/agents/audit.md`) hands both files to every reader.

A Deviation lives here and nowhere else: not in a code comment, where no
number may sit and nobody reads across the tree, and not in
`docs/references.toml`, where a source has many uses and a Deviation
belongs to one site. A reader who finds the code matching a row files no
Finding; one who finds a row wrong files a Finding against the row, not
the code. A row is added when the owner accepts, at a triage, a deviation
an audit found past its tolerance, or when a PR deviates on purpose; it
goes when its trigger fires and the code changes, or when the source it
deviates from is superseded.

| Where | What deviates | Source | Why | Revisit when |
|---|---|---|---|---|
| `src/render/shaders/shading.wgsl:point_light_luminance` | KHR_lights_punctual attenuates a point light by 1/d²; the code holds d² at 1 cm² or more (`MINIMUM_LIGHT_DISTANCE_SQUARED`), so within 1 cm of the light the direction it shades with, the offset over the held distance, is shorter than unit | `khr-lights-punctual-range`, `filament-punctual-light-1cm` | 1/d² is unbounded at the light; Filament holds the same floor, a light 1 cm in radius, so a surface at a point light is lit finitely | area lights arrive |
| `src/render/shaders/display.wgsl:dither_noise` | Peters's listing feeds the triangular remap the mask's texel value, k/255; the code feeds it the level's bin centre, (k + ½)/256, so level 0 maps to −0.9375 where the listing gives −1.0, at most 1/16 of an 8-bit step apart | `peters-blue-noise` | Each of the 256 levels stands for its bin's centre in (0, 1), so no level lands on the remap's endpoints, where the triangular distribution's inverse CDF is steepest | the dither mask's bit depth or the target's changes |
| `src/cook/mesh.rs:meshlet_triangle_counts` | meshoptimizer's scan builder counts a degenerate triangle's repeated corner twice toward a Meshlet's vertex limit; the code counts it once, so on a degenerate triangle the two can cut a Meshlet at a different triangle | `meshoptimizer-1.3-scan-meshlets` | The code reimplements the scan builder, which `meshopt` binds only as an `unsafe` call; a repeated corner is one vertex of the Meshlet, so counting it once is the Meshlet's own vertex count and cuts none past the 64-vertex limit | the cook switches to meshoptimizer's builder, or a Golden differs between the two cuts |
| `src/render/shaders/common.wgsl:selected_lod` | meshoptimizer's example draws a LOD whose error is at most a pixel's size at the distance to the bounding sphere (`>=`); the code draws it only where the error is under it (`<`), so from inside the bounding sphere, where the distance is 0, it draws LOD 0 even beside a LOD of error 0 | `meshoptimizer-1.3-lod-selection` | The LOD threshold is the error a LOD must stay under, and a camera inside an Instance's bounds is as near it as a camera gets, where the source is what is drawn | A cooked chain holds a coarser LOD of error 0 |
| `src/streaming/mips.rs:Sight::Ahead` | Unreal's streamer prestreams a hidden component's textures one mip coarser than it would need; the code requests a held Chunk's or Far proxy's mips at its full need | `ue-5.8-texture-streaming-hidden-prefetch` | A Far proxy held past the View distance needs, from there, the mip it needs once drawn at the View distance, so one coarser would leave each late by a mip on the frame it is drawn, the late mips the prefetch removes | the Texture pool's bytes pass 90% of its Budget |
| `src/render/shaders/shadow_mask.wgsl:shadowed` | Tabellion and Lamorlette ignore a hit near a ray's origin within an offset larger than the largest between the coarse and fine tessellations of the surface the ray leaves; the code passes the back face of whichever Instance a ray leaving the surface meets first, the receiver's or another's, within `LOD_ERROR_MARGIN` times that Instance's drawn LOD error, so an open Mesh drawn past LOD 0 stops shadowing a surface its back face lies that close above | `tabellion-lamorlette-2004-ray-origin-offset` | The pass rebuilds the receiver from depth and normals and does not know its Instance, so it cannot tell the receiver's own drawn LOD from another's; by the LOD threshold, the passed height is under 3 pixels' width at the occluder's distance, and no capture shows it | a capture shows light under the back face of an open Mesh drawn past LOD 0, or a pass that knows the receiver's Instance traces these rays |
| `src/assets/cooked.rs:CookedPrimitive::texel_density` | Unity's UV distribution metric is one figure per Mesh, the average over its triangles of triangle area over UV area; the code takes one per Primitive, the square root of its summed UV area over its summed mesh-space area, as the cook's `texel_density` (`src/cook/mesh.rs`) computes it | `unity-6000-uv-distribution-metric` | Each Primitive's Material has its own Textures, so mip priority needs a density per Primitive (ADR 0017); a ratio of sums weights each triangle by its area where an average of ratios lets a sliver count as much as a large triangle, and the square root makes it UV units per metre, the length mip priority compares with a pixel's footprint (ADR 0018) | mip priority judges texel density per triangle or per Meshlet |
| `src/render/probe_update_pass.rs:probe_ray_directions` | DDGI rotates its Fibonacci spiral of ray directions stochastically; the code traces the same 128 directions every update | `majercik-2019-ddgi-ray-directions` | A fixed set converges each Probe to one estimate, so a still scene does not flicker, and its error is a fixed bias of the spiral rather than noise | the flicker and convergence tests choose the ray set |
| `src/render/shaders/probe_blend.wgsl:blend_irradiance` | DDGI's Equation (1) sums each ray's radiance weighted by its cosine to the texel, and skips a texel whose weights fall below 0.001; the code divides that sum by the weights' and multiplies by π, and skips no texel | `majercik-2019-ddgi-probe-update` | Divided by the weights, a texel holds irradiance in lux whatever the ray count, so the ADR 0023 scale and a capture read it directly; 128 fixed rays leave every texel's weights well above the threshold | the ray count or the ray set changes so a texel can be left with no ray in its lobe |
| `src/render/probes.rs:IRRADIANCE_FORMAT` | DDGI stores a Probe's irradiance in R11G11B10F, 4 bytes a texel; the code stores it in RGBA16F, 8 bytes a texel, its alpha unused, so a Cascade's irradiance takes twice the bytes | `majercik-2019-ddgi-probe-encoding` | The Capability floor does not request `RG11B10UFLOAT_RENDERABLE`, without which `Rg11b10Ufloat` is neither a render attachment nor storage, so no pass could write it; `Rgba16Float` is both blendable and storage (`wgpu-types-30-guaranteed-format-features`) | the Capability floor takes `RG11B10UFLOAT_RENDERABLE`, or the `probes` kind nears its Budget |

- **Where**: `path:item`, the function or constant that deviates.
- **What deviates**: the source's value or form and the code's, in one
  sentence.
- **Source**: the key in `docs/references.toml` the code cites for it.
- **Why**: the reason the code is right to differ: the WGSL accuracy floor
  of its expression chain, a measured cost, an ADR's clause, a backend's
  behaviour.
- **Revisit when**: the trigger that reopens it: a milestone, an ADR, a
  dependency version, a measurement crossing a number.
