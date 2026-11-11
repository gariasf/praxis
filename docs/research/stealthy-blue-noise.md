# Stealthy blue noise: what it could buy Praxis

Read 2026-11-11 at the owner's request: Moran, Donnelly and Wolfe, "Stealthy Blue Noise", ACM TOG 45(6), article 258, December 2026, open access ([doi:10.1145/3842562](https://doi.org/10.1145/3842562)). It is one paper, not a survey; it seeds #758, the prototype that waits for M5's charting, and the fog line on #17 that names it.

**Summary.** The paper generates blue-noise point sets offline by driving the structure factor S(k) to zero on every Fourier mode below a cutoff. It gets a sharp spectral step with no crystalline order, where earlier optimizers roll off smoothly. A frame never runs it, so the runtime cost is a texture or buffer read, as the dither mask's is today. It extends the construction to masks Praxis has a use for: one point per pixel, stored as a texture; one set whose every prefix is blue noise, so a per-pixel threshold gives a blue-noise pattern at any density; points on the sphere; and tiles joined by ghost points. Without screen-space history (ADR 0008), a pattern's error stays visible in every frame, so its spectrum matters more to Praxis than to an engine with TAA. The gains it measures over a jittered grid are modest, though: at most 17% lower MSE in its one-sample-per-pixel renders. It never compares a threshold mask against void-and-cluster, which made the mask Praxis ships. Its generator code is not released. Worth a prototype at M5's charting for the LOD fade's dither, and a line each on the M6, contact-term and terrain fog lines. Nothing to change today.

## Where Praxis stands

- **The one blue-noise use today** is the tonemapping pass's 8-bit dither. `display.wgsl` reads Peters' 64×64 mask [ref:peters-blue-noise], made by Ulichney's void-and-cluster method [ref:peters-blue-noise-void-and-cluster], remapped to a triangular distribution. Its noise is under one 8-bit step, so a better mask there would not show.
- **Every lighting term is per frame or world-space.** Shadows are one hard ray per light per pixel (ADR 0020), and nothing samples stochastically per pixel. ADR 0008 allows Temporal classes (a), complete each frame, and (b), converging in world or probe space, never screen-space history.
- **The next consumers are on the roadmap.**
  - M5's LOD transitions: T2 dithers per pixel over a set time with "its pattern visible for the fade", and T3 cross-fades per sample through complementary `sample_mask` writes ([LOD note](lod-hlod-and-far-field.md), Options). W1 and W2's coverage fades for thin and small Instances need the same kind of mask.
  - M6's DDGI probes trace each update along "a stochastically-rotated Fibonacci spiral pattern" [ref:majercik-2019-ddgi-ray-directions], 64 rays a probe in its largest scenes (Table 1, p. 19).
  - The contact term, class (a) occlusion from short rays or spatial-only GTAO, is a fog line on #17 (ADR 0020).
  - Terrain and vegetation scatter, a fog line on #17. The world generator places each settlement at its region's centre plus a hashed stray (`settlement_centre`, `src/scenes/world.rs`), which makes a jittered grid.

## What the paper does

- **The loss.** For N points on the unit torus, minimise Σ|ρ(k)|² over the M lowest modes, with M = ⌊χ·d·(N − 1)⌋. Up to χ ≈ 1/2 the ground state stays disordered; past it, the set crystallises. The energy and its forces cost O(NM) per step, and the authors use L-BFGS in PyTorch Lightning [ref:moran-2026-stealthy-cost]. From random starts, S(k) typically falls below 10⁻¹⁰ on every constrained mode "within tens of iterations".
- **Against prior work.** Gaussian Blue Noise trades suppression depth against disorder through its kernel width, with no setting that gives "both a deep floor and a sharp edge" (Fig. 4). Step blue noise averages S(k) = 8.3 × 10⁻² inside the same cutoff, against 1.5 × 10⁻²² (Fig. 5) [ref:moran-2026-stealthy-prior-work]. The paper scores order with the bond-orientational parameters ψ4 and ψ6 from 2D melting, and maps the uniformity–disorder trade as a Pareto frontier (Fig. 6).
- **One point per pixel** (§5.3). Hungarian-algorithm reprojection after each step keeps exactly one point per cell, "stored directly as a texture of per-pixel sub-pixel offsets". The imposed grid leaves "slight axial spikes near the cutoff", up to "about one third of the corresponding pure-grid peak" [ref:moran-2026-stealthy-grid-texture].
- **One-sample-per-pixel renders** (Fig. 11, Fig. A.20). On a natural image, RMSE is 0.033 for the regular grid, 0.033 for the jittered grid and 0.028 to 0.032 for the four blue-noise variants. On a perspective checkerboard under a σ = 2 px Gaussian, MSE is 0.00156 for the grid, 0.00100 jittered and 0.00083 to 0.00099 blue. The regular grid's moiré is one that "neither filtering nor accumulation removes" [ref:moran-2026-stealthy-grid-error].
- **Every prefix blue** (§5.2). A weighted sum of stealthy losses over all prefixes orders the points, and the local density is "selected at sample time by a per-point threshold against the importance field". Its mean S(k) beats Mitchell's best-candidate "typically by a factor of two". The total N is fixed beforehand, and "the sequence cannot be extended afterwards" [ref:moran-2026-stealthy-progressive].
- **Matching a reconstruction filter** (Fig. A.18). Taking, at equal M, the modes where the filter's |H(k)|² is largest gives MSE 0.0262, "better than either norm by 1.87× at identical cost" [ref:moran-2026-stealthy-filter-matched].
- **The sphere** (§5.4). Spherical harmonics replace Fourier modes: 136 points with every mode to ℓ = 15 suppressed, "without the polar axis and winding direction that betray structured distributions such as Fibonacci lattices" [ref:moran-2026-stealthy-sphere].
- **Tiles and density** (§6). Tiles exchange read-only ghost points across their edges, which "supports Wang tiling". A per-point mode budget gives variable density without crystallising the sparse regions [ref:moran-2026-stealthy-tiles].
- **Limits** (§7). Generation is O(N²) in the point count, "thousands-to-tens-of-thousands" per domain; a non-uniform FFT would cut that to O(N log N), which the authors call "the natural next implementation step". N is fixed, and quality is "mildly sensitive to N" through partly filled mode shells [ref:moran-2026-stealthy-cost]. Time is not a dimension: spatiotemporal sampling is listed as future work [ref:moran-2026-stealthy-progressive].

## Where it could apply

| Use in Praxis | What the paper offers | Today's alternative | Evidence for the gain | Fit |
|---|---|---|---|---|
| LOD fade dither, T2 (M5) | A set whose every prefix is blue; the fade level thresholds it | Peters' void-and-cluster mask, also a ranked threshold mask, or a 4×4 ordered pattern (Bevy) | Prefixes beat best-candidate by about 2× in S(k); no comparison with void-and-cluster; prefix order and one point per pixel are never combined | Offline mask compiled in like `dither_mask.bin`; no history needed |
| Per-sample fade, T3 (M5) | A per-pixel rank of which of 4 MSAA samples go to each level | Fixed or hashed sample choice | None: the paper leaves joint losses over class unions to future work [ref:moran-2026-stealthy-progressive] | The rank could come from one mask's levels |
| Coverage fade, W1 and W2 (M5) | As T2 | As T2 | As T2 | As T2 |
| Probe ray directions (M6) | An isotropic sphere set at DDGI's 64 to 256 rays a probe, rotated per update as DDGI rotates its spiral | Spherical Fibonacci, randomly rotated (DDGI) | Spectra only (Fig. A.21); no probe-irradiance variance measured | Class (b) already; flicker against lag is M6's check |
| Contact term rays, class (a) | Per-pixel offsets matched to the spatial filter that follows | A blue-noise texture driving the directions | 1.87× lower MSE from filter matching, on stippled images | Spatial-only, so filter matching applies |
| Terrain and vegetation scatter | Variable density with local χ, tiles seamless through ghost points | Jittered grid from a hash, as `settlement_centre` does | Stippling figures only (Figs. A.22 to A.24) | Cook-time, per Chunk; O(N²) per tile |
| 8-bit dither (today) | A better threshold mask | Peters' mask | Below an 8-bit step | No visible gain; leave it |

## What would decide it

Each comparison runs against the pattern the engine would use otherwise, with the generator written from the paper. No code is released, and the paper's licence covers its text, not the method.

1. **Dissolve mask, for M5's charting.** Generate a 64×64 one-point-per-pixel set ranked so every prefix is blue: an untested combination of §5.2 and §5.3. Then, per fade level from 1/64 to 63/64, compare it with Peters' mask on mean S(k) inside the level's own cutoff and on ψ4 and ψ6. Capture `town`'s LOD fade at 25%, 50% and 75%, still camera, 2560×1440, and judge it at 100% and in 2x crops (`scripts/gallery.sh`'s practice). The outcome decides T2's mask and W1 and W2's fade.
2. **Probe rays, at M6.** At the ray count M6 picks, compare a stealthy sphere set and a spherical Fibonacci set, each randomly rotated per update. Measure the frame-to-frame variance of probe irradiance against a converged reference on a Reference scene, at the hysteresis M6 sets. A lower variance buys a lower hysteresis, so less lag, for the same flicker.
3. **Contact rays, when the contact term opens.** Filter-matched offsets against a plain blue-noise texture, each through the spatial filter the term uses. Measure error against a supersampled reference in linear light.
4. **Scatter, when terrain and vegetation open.** Per-Chunk sets with ghost points across the 256 m seams, against the hashed jittered grid. Judge it visually and by S(k) across a seam, and time it in the cooker.

## Licence

The paper is CC BY-NC-ND 4.0 [ref:moran-2026-stealthy-licence]: its text may be quoted with attribution, not adapted. The method is described in full: loss, gradient, optimiser, reprojection and the spherical form. A generator written from that description is Praxis's own, and so are the masks it produces. No code was found to copy.

## A different blue noise: the subsurface article

The owner also passed Maximilian Tarpini's "What Happens When SSS Meets Blue Noise: Super-Exponential Light Transport" (LinkedIn, 2026-09-20). There, blue noise is how a material's scatterers are arranged, not a sampling pattern.

- **The claim.** Scatterers placed independently give Beer–Lambert's exponential free flight. Clustered ones leave open corridors, so light gets further. Scatterers held apart by a minimum separation close those corridors, so light dies "super-exponentially" within a short depth, which gives glazed ceramics a shallow, uniform scattering layer and a glow at thin edges.
- **The math.** Its note ([Statistical Foundation](https://github.com/RomboDev/Miscellaneous/blob/master/MathematicaNotebooks/RandomWalks/Super-Exponential_Light_Transport__%20Statistical_Foundation.pdf)) makes two arguments. A continuous random extinction field can only raise mean transmittance above the classical value, by Jensen's inequality. A renewal process with a hard minimum gap ŝ and an exponential remainder falls below it. Seen from an arbitrary entry point, transmittance is linear, 1 − s/μ, up to ŝ.
- **Its reading list.** Shaw et al. 2002, Davis et al. 2014, Jarabo et al. 2018, Bitterli et al. 2018, d'Eon 2018 and Guo et al. 2019 are listed as the article gives them; none was opened here.
- **Its relation to the paper.** It shares only the physics root. The paper takes stealthy hyperuniform point sets from the materials science of disordered media, and such sets are an extreme case of the negatively correlated layouts the article describes. Neither cites the other.
- **Its fit.**
  - The article renders with offline random walks. Without history (ADR 0008), a per-pixel random walk at one sample a pixel would be noise.
  - For Praxis it is a medium model to weigh where media first appear: characters' subsurface scattering, a profile baked offline from such walks; and atmosphere, clouds and canopies, where clustering raises transmittance instead.
  - A non-exponential medium has no memoryless free flight, so transmittance through a column is no longer the product of its segments' transmittances. That product is what a per-Froxel or per-step volume integration multiplies.
  - Cycles' random walk is exponential, so the Cycles reference would differ there by construction.
  - The note's repository states no licence: its math may guide a survey; its code and notebooks are never copied.

## Not verified

- Stealthy masks against void-and-cluster threshold masks, at any level: the paper compares against Gaussian Blue Noise, step blue noise, best-candidate, jittered and regular grids only.
- The combination of every-prefix-blue with one point per pixel, which a dissolve mask needs; the paper does each alone.
- Any rendered gain on a sphere: Fig. A.21 shows spectra, not integration error.
- Generation time for any N: the paper gives iteration counts, not seconds or hardware.
- Behaviour across frames: every pattern in the paper is static, and moving one without history would shimmer.
- The subsurface article's sources and its renders: it was read as published, its math note skimmed for its argument, and no claim of it is relied on here.
