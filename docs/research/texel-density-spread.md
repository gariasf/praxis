# Texel density spread: how far a Primitive's UVs are from its cooked mean

Measured 2026-11-11 on `main` at d454229c, for #760: every Mesh the cook reads, the Reference scenes' generated Meshes and the manifest's two glTF sources.

**Summary.** Leave the mean. Every Primitive that samples a streamed Texture is within one mip of its cooked texel density over all of its area. The world's walls, the only surfaces in the world scene that sample one, are perfectly even: every wall triangle has 0.250 UV units per metre, the same as the mean. So no candidate changes a wall's figure, and the world's traversal requests the same 230 mips under each. Primitives that spread past two mips exist (town's props, the crowd's figure, birds), but their Materials sample no Texture, so mip priority never reads their density. #17's fog line keeps its trigger, an authored or scanned streamed Mesh, and gains these figures.

## Impact

- Nothing changes: no cook, Blob or runtime change follows, and no defect ticket is filed.
- When a streamed Mesh comes from an authored or scanned source (M7's content at the latest), the figures below are the baseline to compare its spread against.

## Method

Mip priority (ADR 0017, `src/streaming/mips.rs`) turns a Primitive's `CookedPrimitive::texel_density`, the square root of its summed UV area over its summed mesh-space area, into the finest mip a Placement needs. A triangle whose own density is r times that mean needs a mip log2(r) finer than the one requested, so r = 2 is one mip too coarse on that triangle and r = 4 is two.

A throwaway test probe ran over the cook's `Geometry` for each Primitive `read_sources` returns for all twelve Reference scenes and both manifest glTF sources. Per triangle it took √(UV area / mesh area). It reported the densest triangle's ratio to the cooked mean, and the ratio at the top 1% and top 5% of area (the density above which that share of the Primitive's area lies). It also reported the area share whose ratio is 2 or more (one mip or more too coarse) and 4 or more (two mips or more). Its oracle was a hand-built Primitive of two triangles: 1 m² at 8 UV units per metre and 3 m² with no UV area. The probe gave a mean of 4, a densest ratio of 2, 25% of the area one mip finer and none two mips finer. The cook's own `texel_density` returned the same mean.

"Streamed Texture" marks a Primitive whose Material, its own or its Placements', samples a Texture. The world's walls take their facade Material from their Placement, and so does the texture wall's quad.

## Figures

Densities in UV units per metre; ratios over the cooked mean.

| Mesh | Primitive | Streamed Texture | Triangles | Cooked | Densest | Densest × | Top 1% × | Top 5% × | ≥ 1 mip | ≥ 2 mips |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
| SciFiHelmet (glTF) | 0 | yes | 23,358 | 0.156 | 0.441 | 2.82 | 1.14 | 1.07 | 0.3% | 0 |
| NormalTangentTest_low (glTF) | 0 | yes | 7,774 | 0.375 | 0.593 | 1.58 | 1.58 | 1.58 | 0 | 0 |
| pbr-sweep sphere | 0 | no | 2,208 | 0.553 | 1.248 | 2.26 | 2.26 | 1.60 | 1.7% | 0 |
| uv-grid plane | 0 | yes | 2 | 0.250 | 0.250 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| far-coplanar plane | 0 | no | 2 | 0.008 | 0.008 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| orrery sphere | 0 | yes | 2,208 | 0.553 | 1.248 | 2.26 | 2.26 | 1.60 | 1.7% | 0 |
| minification ground | 0 | yes | 2 | 1.000 | 1.000 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| texture-wall quad | 0 | yes | 2 | 1.250 | 1.250 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| shadows sphere-over-plane | 0 | no | 2 | 0.062 | 0.062 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| shadows sphere-over-plane | 1 | no | 2,208 | 0.553 | 1.248 | 2.26 | 2.26 | 1.60 | 1.7% | 0 |
| crowd ground | 0 | no | 2 | 0.042 | 0.042 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| crowd figure | 0 | no | 5,120 | 0.656 | 4.097 | 6.24 | 2.52 | 1.56 | 2.1% | 0.2% |
| crowd proxy | 0 | no | 168 | 0.270 | 0.373 | 1.39 | 1.39 | 1.39 | 0 | 0 |
| town ground | 0 | no | 2 | 0.008 | 0.008 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| town box | 0 | no | 12 | 1.000 | 1.000 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| town air-conditioner | 0 | no | 132 | 1.308 | 6.250 | 4.78 | 3.15 | 1.20 | 1.5% | 0.6% |
| town vent | 0 | no | 90 | 1.938 | 4.166 | 2.15 | 2.15 | 2.15 | 8.7% | 0 |
| town water-tank | 0 | no | 138 | 1.016 | 8.333 | 8.20 | 2.59 | 2.59 | 9.0% | 0.4% |
| town dish | 0 | no | 104 | 2.266 | 12.499 | 5.51 | 4.41 | 1.56 | 1.2% | 1.2% |
| town aerial | 0 | no | 56 | 4.865 | 33.332 | 6.85 | 3.43 | 1.53 | 1.0% | 0.8% |
| town solar-panel | 0 | no | 24 | 6.222 | 20.003 | 3.21 | 3.21 | 0.83 | 3.2% | 0 |
| town solar-panel | 1 | no | 12 | 1.327 | 5.000 | 3.77 | 3.77 | 2.98 | 6.1% | 0 |
| town railing-post | 0 | no | 20 | 3.873 | 20.002 | 5.16 | 5.16 | 0.67 | 2.1% | 2.1% |
| town lamp | 0 | no | 52 | 2.014 | 12.911 | 6.41 | 2.76 | 1.85 | 1.3% | 0.6% |
| town utility-pole | 0 | no | 40 | 1.119 | 7.454 | 6.66 | 3.44 | 1.92 | 1.7% | 1.0% |
| town wire | 0 | no | 90 | 0.716 | 0.717 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| town tree | 0 | no | 20 | 1.062 | 4.167 | 3.92 | 3.92 | 0.72 | 4.7% | 0 |
| town tree | 1 | no | 54 | 0.271 | 0.500 | 1.84 | 1.84 | 1.84 | 0 | 0 |
| town bench | 0 | no | 48 | 2.474 | 7.906 | 3.19 | 2.86 | 2.02 | 5.9% | 0 |
| town red-car | 0 | no | 12 | 0.523 | 0.951 | 1.82 | 1.82 | 1.82 | 0 | 0 |
| town red-car | 1 | no | 12 | 0.778 | 1.155 | 1.48 | 1.48 | 1.48 | 0 | 0 |
| town red-car | 2 | no | 144 | 1.543 | 1.563 | 1.01 | 1.01 | 1.01 | 0 | 0 |
| town blue-car | 0 | no | 12 | 0.523 | 0.951 | 1.82 | 1.82 | 1.82 | 0 | 0 |
| town blue-car | 1 | no | 12 | 0.778 | 1.155 | 1.48 | 1.48 | 1.48 | 0 | 0 |
| town blue-car | 2 | no | 144 | 1.543 | 1.563 | 1.01 | 1.01 | 1.01 | 0 | 0 |
| town bollard | 0 | no | 42 | 2.374 | 4.999 | 2.11 | 2.11 | 2.11 | 13.2% | 0 |
| town bird | 0 | no | 24 | 6.279 | 26.728 | 4.26 | 2.30 | 2.06 | 8.0% | 0.9% |
| town bin | 0 | no | 88 | 1.354 | 2.941 | 2.17 | 2.17 | 1.32 | 4.2% | 0 |
| world ground | 0 | no | 2 | 0.008 | 0.008 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| world window | 0 | no | 10 | 1.070 | 2.635 | 2.46 | 2.46 | 2.13 | 12.1% | 0 |
| world bird | 0 | no | 24 | 6.279 | 26.728 | 4.26 | 2.30 | 2.06 | 8.0% | 0.9% |
| world cottage | 0 | yes | 8 | 0.250 | 0.250 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| world cottage roof | 0 | no | 10 | 0.292 | 0.711 | 2.44 | 2.44 | 2.44 | 14.5% | 0 |
| world house | 0 | yes | 8 | 0.250 | 0.250 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| world house roof | 0 | no | 10 | 0.244 | 0.623 | 2.56 | 2.56 | 2.56 | 12.2% | 0 |
| world terrace | 0 | yes | 8 | 0.250 | 0.250 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| world terrace roof | 0 | no | 10 | 0.250 | 0.711 | 2.85 | 2.85 | 2.25 | 12.9% | 0 |
| world hall | 0 | yes | 8 | 0.250 | 0.250 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| world hall roof | 0 | no | 10 | 0.209 | 0.589 | 2.81 | 2.81 | 2.81 | 10.6% | 0 |
| world townhouse | 0 | yes | 8 | 0.250 | 0.250 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| world townhouse roof | 0 | no | 10 | 0.262 | 0.711 | 2.72 | 2.72 | 2.72 | 13.3% | 0 |
| world tower | 0 | yes | 8 | 0.250 | 0.250 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| world tower roof | 0 | no | 10 | 0.244 | 0.623 | 2.56 | 2.56 | 2.56 | 12.2% | 0 |
| world shop | 0 | yes | 8 | 0.250 | 0.250 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| world shop roof | 0 | no | 10 | 0.245 | 0.662 | 2.70 | 2.70 | 2.70 | 12.4% | 0 |
| world block | 0 | yes | 8 | 0.250 | 0.250 | 1.00 | 1.00 | 1.00 | 0 | 0 |
| world block roof | 0 | no | 10 | 0.200 | 0.561 | 2.81 | 2.81 | 2.81 | 10.2% | 0 |

What the figures say:

- **Streamed Primitives.** The flat quads and the world's walls are uniform (1.00 at every statistic). The orrery's sphere is one mip too coarse on 1.7% of its area, the triangles at its poles. SciFiHelmet's densest triangle is 2.82 times its mean, but only 0.3% of its area is one mip off, and its top 1% is 1.14. NormalTangentTest's bumped half is 1.58 times the mean, under one mip. None has any area two mips off.
- **Spread without a Texture.** Town's props reach 8.2 (water tank) and the crowd's figure 6.2, but over at most 2.1% of area two mips off (railing post). The world's roofs and windows are 2.4 to 2.9 times their mean on 10 to 15% of their area: each slab face maps the whole unit square, so its thin sides are as dense as its top is sparse. Were a slab ever textured, those sides would show one mip too coarse.

## Candidates and their cost

The finer mips each candidate asks for, against today's mean, as log2 of its ratio. A shift of s mips multiplies the finest Resident mip's texels by up to 4^s for the Placements that need it.

| Cooked figure, per Primitive | World traversal: mips requested | World walls | Orrery sphere | SciFiHelmet | NormalTangentTest |
|---|---:|---:|---:|---:|---:|
| Mean (today) | 230 | 0 | 0 | 0 | 0 |
| Top 5% of area | 230 | 0 | +0.68 | +0.10 | +0.66 |
| Top 1% of area | 230 | 0 | +1.17 | +0.19 | +0.66 |
| Densest triangle | 230 | 0 | +1.17 | +1.50 | +0.66 |

- **Today's figure** is a probe counter in `MipPriority::plan` over one `praxis traverse` on the M4 Pro: 230 requests of one mip each. The machine was not quiet and on battery, which can move when a request lands but not how many levels it asks for. Every candidate gives the walls the same 0.250, so each would cook the same wall Blobs and request the same mips.
- **Per Meshlet.** Each wall is 8 triangles, one Meshlet, so per-Meshlet figures change nothing in the world either. On a spread Primitive a Meshlet's figure lies between the mean and the densest triangle's, so its cost is bounded by the densest row.
- **What each would buy.** Densest triangle fixes every triangle at the price of the whole Primitive: SciFiHelmet would ask for 1.5 mips finer everywhere to sharpen 0.3% of it. A high percentile per Meshlet confines the cost to the Meshlets that carry the dense UVs. It is the candidate to measure first when the trigger fires, against a cook-time report of each Primitive's densest-over-mean ratio, as this probe printed it.

## Open questions

1. Whether authored or scanned content (M7) spreads past two mips over a visible share of a streamed Primitive. The trigger on #17 says when to measure; owner: whoever takes that content's import ticket.
