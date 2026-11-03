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
| `src/cook/mesh.rs:meshlet_triangle_counts` | meshoptimizer's scan builder counts a degenerate triangle's repeated corner twice toward a Meshlet's vertex limit; the code counts it once, so on a degenerate triangle the two can cut a Meshlet at a different triangle | `meshoptimizer-0.25-scan-meshlets` | The code reimplements the scan builder, which `meshopt` binds only as an `unsafe` call; a repeated corner is one vertex of the Meshlet, so counting it once is the Meshlet's own vertex count and cuts none past the 64-vertex limit | the cook switches to meshoptimizer's builder, or a Golden differs between the two cuts |

- **Where**: `path:item`, the function or constant that deviates.
- **What deviates**: the source's value or form and the code's, in one
  sentence.
- **Source**: the key in `docs/references.toml` the code cites for it.
- **Why**: the reason the code is right to differ: the WGSL accuracy floor
  of its expression chain, a measured cost, an ADR's clause, a backend's
  behaviour.
- **Revisit when**: the trigger that reopens it: a milestone, an ADR, a
  dependency version, a measurement crossing a number.
