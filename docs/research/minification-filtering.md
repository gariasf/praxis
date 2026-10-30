# Filtering minified texture detail wider than the pixel's box: state of the art

Researched 2026-10-30 for ticket #388, from #359's finding that `minification`'s far checker moiré is in the exact box-filtered reference (`docs/performance.md`, "minification's far moiré is in the box-filtered reference").

**Summary.** The residue is the box's own: a box's mean of a sinusoid falls only as its sinc and flips sign, and hardware anisotropic filtering weights its probes equally, a box along the major axis. Only a filter without the box's sidelobes removes it, and every such filter blurs the checks the box resolves. Without temporal history the published way to put one in the texture lookup is Gaussian-weighted probes along the major axis (Feline 1999, Mavridis & Papaioannou 2011); on the M4 Pro a shader version at σ 0.75 px takes the far band's moiré from 5.31 to 1.27 steps, lands 1.11 steps from an exact Gaussian reference, and adds about 0.3 ms to `minification`'s gate and 0.6 to 0.7 ms to `town`'s. A wide resolve over 4x MSAA needs per-sample shading to see texture detail inside the pixel, which adds about 10 to 12 ms to `town`'s gate; without it, it blurs the whole shaded image. The one post-2019 line of work, stochastic texture filtering, resolves its noise temporally in real time, or with more samples per pixel, and its temporal form is out under ADR 0008. Options close the note; none is chosen.

Sources: papers (SIGGRAPH, IEEE CG&A, I3D, TOG, PACMCGIT, JCGT; 1986 to 2025) by author or publisher copy; PBRT 4th edition online; the WebGPU, WGSL and Vulkan specifications; SIGGRAPH 2015 Advances; blog posts by Pettineo, Golus and Wronski; source at the commits linked; and probes measured on the M4 Pro (Measured on the M4 Pro). Each claim the note relies on is in `docs/references.toml`, cited by key. Moiré and RMSE figures are in 8-bit steps over the far band's outer 800 pixels each side, as #359 scored them.

## Comparison

| Technique | Buys against the box (far band) | Blur it costs | Published cost (context as stated) | Maturity | Fit: no history (ADR 0008), 4x MSAA lit pass shading once per pixel (ADR 0019), wgpu core | Licence |
|---|---|---|---|---|---|---|
| 1. Box footprint, hardware anisotropic (today) | Is the box: moiré 5.31, 1.68 from the exact box | None beyond the box | Free | Shipped everywhere | Is Praxis | n/a |
| 2. Gaussian, Mitchell or windowed-sinc pixel filter over supersamples | Gaussian σ 0.75 px: moiré 1.18 (exact 3x); Mitchell's notch "strongly suppresses postaliasing patterns" | 9.05 from the exact box; notch "quite blurry"; sinc rings | Offline default (pbrt-v4 Gaussian); needs samples inside the pixel | Offline standard | Needs per-sample shading: +10.3 to 11.5 ms `town` gate (measured) | n/a |
| 3. Wide MSAA resolve, shading once per pixel | Blurs the shaded image: moiré 1.40 (σ 0.75 px blur of the capture) | Every pixel blurred, geometry and all textures: 8.37 from the exact box | The Order, PS4 1920×800: 4x custom resolve 0.63 ms against hardware 0.56 ms without TAA; shipped 2 fragments, 4 coverage samples, with TAA, 1.02 against 0.50 ms | Shipped: The Order: 1886 (with TAA) | Fits: a compute resolve; filters what the box already aliased | MIT (Pettineo's samples) |
| 4. Exact EWA | The reference the probe methods approximate | Gaussian's blur | 93 MTexel/s, 5.5% of hardware 16x, GTX 460 900² | Research, offline | Fits; cost unbounded without an eccentricity clamp | Pseudocode |
| 5. Gaussian-weighted probes along the major axis (Feline, Mavridis & Papaioannou) | σ 0.75 px: moiré 1.27, 1.11 from the exact Gaussian; σ 0.5: 2.96 | σ 0.75: 9.13 from the exact box; σ 0.5: 6.09 | +0.28 ms `minification` gate, +0.60 to 0.67 ms `town` (measured); 658 MTexel/s, 38.7% of hardware 16x, MSE 30.0 against 126.3 | Research; shader drop-in | Fits: `textureSampleLevel` in the lit pass, colour Channels only | MIT (author's demo, archived) |
| 6. Footprint widened past the pixel (scaled gradients) | 1.5x: moiré 3.16; 2x: 2.05 | 1.5x: 7.98 from the exact box; 2x: 10.25, more than the Gaussian, with more residue | 1.5x: +0.12 ms `minification` gate (measured); 2x not timed | Hardware path | Fits: `textureSampleGrad` | n/a |
| 7. LOD bias | +1: moiré 5.00 | 2.14 from the exact box | Free | Hardware path | Fits: `textureSampleBias` (wgpu samplers have no bias) | n/a |
| 8. Wider mip prefilters (Kaiser, Lanczos) | Changes each mip texel, not the probes' equal weights | "weird dots or lines" | Cook time only | Shipped in tools | Fits the cooker | n/a |
| 9. Shader supersampling of the lookup (RGSS) | Finer sampling of the same box: per-sample, half footprint: moiré 4.85 | Sharper than anisotropic alone | +0.15 ms (4 taps), +1 ms (8), GTX 970 1080p | Blog practice | Fits; still a box | Not stated |
| 10. Stochastic texture filtering | Any kernel at a cost independent of its size (one lookup; two with negative lobes) | Per kernel | Real-time results resolved with DLSS, 4K RTX 4090 | Research (I3D 2024), SDK | Out: temporal integration | NVIDIA RTX SDK licence |

## Techniques

**1. The box today.** A pixel that averages its footprint applies a box, whose frequency response is a sinc that neither isolates the signal nor rejects frequencies past Nyquist ([PBRT §8.1.2][pbrt-sampling] [ref:pharr-2023-pbrt-box-spectrum]); PBRT calls it "just about the worst filter possible" ([PBRT §8.8.3][pbrt-filters] [ref:pharr-2023-pbrt-box-filter]), Heckbert its response "far from ideal, with high side lobes" ([Heckbert 1989][heckbert] [ref:heckbert-1989-box-side-lobes]), and Mitchell & Netravali name the checkerboard among the periodic patterns whose postaliasing shows as moiré ([Mitchell & Netravali][mitchell] [ref:mitchell-1988-postaliasing-moire]). The texture path builds the same box: Vulkan leaves anisotropic filtering to the implementation, and its one example averages N probes with weight 1/N ([Vulkan][vk-aniso] [ref:vulkan-anisotropic-filtering-implementation-dependent]); the M4 Pro matched a filter built that way within 0.23 steps (#359). Feline names the result: equal weights give "poor high-frequency rejection along the major axis" ([Feline §2.3][feline] [ref:mccormack-1999-equal-weight-probes]).

**2. Wider pixel filters.** Offline renderers trade the box for a Gaussian, pbrt-v4's default, whose slight blur masks remaining aliasing ([PBRT §8.8.3][pbrt-filters] [ref:pharr-2023-pbrt-gaussian]); Mitchell & Netravali's cubics trade blur against ringing, (1/3, 1/3) recommended ([Mitchell & Netravali][mitchell] [ref:mitchell-1988-blur-ringing], [ref:pharr-2023-pbrt-mitchell]), and their notch filter suppresses postaliasing at the cost of being "quite blurry" ([ref:mitchell-1988-notch]); windowed sinc still rings on steps ([ref:pharr-2023-pbrt-windowed-sinc]). Each filters samples inside and around the pixel, and in Praxis's lit pass a pixel has one texture lookup: WebGPU lets a fragment shader run once per pixel and broadcast its result to the covered samples, and requires it to run per sample only when `sample_index` or `sample` interpolation feeds its output ([WebGPU §23.2.10][webgpu] [ref:webgpu-2026-per-sample-shading]), and inputs default to the pixel centre ([WGSL][wgsl-interp] [ref:wgsl-2026-interpolation-sampling], [ref:wgsl-2026-sample-index]). Per-sample shading on the M4 Pro costs `town`'s gate 10.3 to 11.5 ms (Measured), and with each sample still filtering a whole pixel's footprint it blurs (moiré 3.58, 5.60 from the exact box) rather than reproducing the supersampled reference; 2x2 supersamples resized with the Gaussian land 0.97 steps from the exact Gaussian.

**3. A wide MSAA resolve.** Pettineo found cubics without negative lobes 2 to 3 pixels wide the best balance for a custom resolve, and windowed sinc "almost totally unappealing" ([Pettineo 2012][mjp-resolve] [ref:pettineo-2012-msaa-resolve-filters]); The Order: 1886 shipped a compute resolve with a 2-pixel cubic and no negative lobes ([Advances 2015][order] [ref:pettineo-2015-order-resolve]); on PS4 at 1920×800 a 4x custom resolve took 0.63 ms against the hardware's 0.56 ms without TAA, and the game shipped 2 colour fragments and 4 coverage samples with TAA in its resolve, 1.02 ms against 0.50 ms ([slide 38][order] [ref:pettineo-2015-order-aa-timings]); its samples are MIT ([MSAAFilter][msaafilter] [ref:pettineo-2015-msaafilter-licence], [DX11Samples][dx11samples] [ref:pettineo-2021-dx11samples-licence]). With shading once per pixel the samples of an interior pixel hold one colour, so the resolve filters the shaded 1x image: a Gaussian of σ 0.75 px over the capture takes the moiré to 1.40, but blurs every edge, highlight and texture alike (8.37 from the exact box), and filters residue the box already aliased into each pixel rather than the texture.

**4. EWA.** Greene & Heckbert map the pixel's circular filter through the local affine map to an ellipse in texture space and weight it with a Gaussian ([Greene & Heckbert][omnimax] [ref:greene-1986-ewa-kernel-and-cost]); filters over squares or rectangles, mip maps and summed-area tables among them, "cannot filter elongated diagonal areas accurately", giving aliasing or blur ([ref:greene-1986-square-filters-alias-or-blur]). With a Gaussian reconstruction and prefilter the whole resampling filter is an elliptical Gaussian ([Heckbert 1989][heckbert] [ref:heckbert-1989-elliptical-gaussian-resampling]); its cost follows the ellipse's area ([ref:heckbert-1989-ewa-jacobian-cost]) and is unbounded until eccentricity is clamped, where Heckbert prefers fattening the ellipse (blur) to shortening it (aliasing) ([ref:heckbert-1989-pyramid-cost-eccentricity-clamp]). Exact EWA ran at 5.5% of hardware 16x's throughput on a GTX 460 ([Mavridis Table 1][mavridis] [ref:mavridis-2011-table1-cost]), and Feline notes its Gaussian blurs, sharper kernels costing 4 to 9 times the work ([Feline][feline] [ref:mccormack-1999-ewa-blur-and-setup]).

**5. Gaussian-weighted probes.** Feline places isotropic probes along the ellipse's major axis, about 2·major/minor − 1 of them, and weights them by a Gaussian, comparing well with EWA at far fewer texel fetches ([Feline §3][feline] [ref:mccormack-1999-feline-probes]), and, when degraded for speed, can favour blur on regular textures to avoid moiré ([ref:mccormack-1999-feline-conclusions]). Mavridis & Papaioannou do it in a shader with the hardware's anisotropic probes ([I3D 2011][mavridis] [ref:mavridis-2011-ellipse-approximation]): five probes reach MSE 30.0 against reference EWA where hardware 16x reaches 126.3, at 38.7% of its throughput, GTX 460, 900² ([ref:mavridis-2011-table1-cost]); a wider window trades the remaining aliasing for blur ([ref:mavridis-2011-results-window-blur]); its demo and source, MIT-licensed, survive on the author's archived page ([demo][mavridis-demo] [ref:mavridis-2011-code-mit]); its fastest variant splits probes over two frames, which ADR 0008 rules out. Manson & Schaefer replace each trilinear probe with their fixed-cost approximation of an isotropic Gaussian at the same bandwidth, having shown Lánczos 2 for isotropic filtering and a checkerboard plane ([TOG 2013][manson] [ref:manson-2013-cardinality]). The probe measured here is trilinear taps along the major derivative, each as wide as the Gaussian is across, Gaussian-weighted over ±2σ: at σ 0.75 px it matches the exact Gaussian reference within 1.11 steps; at σ 0.5 px it keeps more detail and half the moiré.

**6. A widened footprint.** Scaling the derivatives passed to `textureSampleGrad` widens the hardware's footprint along both of them ([WGSL][wgsl-tex] [ref:wgsl-texture-sample-bias-grad]); its probes keep their equal weights, so it trades the box for a wider box: at 2x the moiré falls to 2.05 but the band lies 10.25 steps from the exact box, blurrier than the Gaussian probes with more residue.

**7. LOD bias.** A bias adds to the level the derivatives choose ([Vulkan][vk-lod] [ref:vulkan-lod-bias]); wgpu 30.0.1's sampler descriptor has no bias field, so it would be `textureSampleBias` in the shader ([wgpu-types][wgpu-sampler] [ref:wgpu-types-30-sampler-descriptor-no-bias]). It widens each probe across the view, not the footprint's length along it, where the moiré lives: +1 moves the moiré from 5.31 to 5.00.

**8. Wider mip prefilters.** Kaiser and other sinc-based mip kernels keep shapes better than a box, but can add "weird dots or lines" ([Golus 2019][golus] [ref:golus-2019-kaiser-mips]); a box 2x downsample both blurs and leaves aliasing ([Wronski 2021][wronski] [ref:wronski-2021-box-downsample]). A better kernel changes what each mip texel holds, not the equal weights over the probes, and #359 found the cooked box mips within 0.49 steps of the exact mean.

**9. Shader supersampling.** Golus's four rotated-grid taps at mip bias −1 over 8x anisotropic come "extremely close" to a ground truth, for 0.15 ms on a GTX 970 at 1080p, 1 ms at eight taps, and still show "some degree of moire" ([Golus 2019][golus] [ref:golus-2019-rgss]). Equal-weight taps over the pixel sample the box more finely; they do not change its sidelobes, as per-sample shading at half footprint shows here (moiré 4.85).

**10. Since 2019.** Stochastic texture filtering applies any kernel at a cost independent of its size, one lookup, or two for kernels with negative lobes such as Mitchell's ([Pharr et al. 2024][stf] [ref:pharr-2024-stf-filter-cost]), but for minification it jitters within the pixel's own footprint ([ref:pharr-2024-stf-anisotropy]), and its noise is resolved by spatiotemporal denoising or by more samples per pixel: every real-time result uses DLSS, and lowpass filtering "needs to average significantly more samples" ([ref:pharr-2024-stf-minification-dlss], [ref:pharr-2024-stf-noise], [ref:pharr-2024-stf-discussion]); its SDK resolves the noise with DLSS, TAA where DLSS is missing, under NVIDIA's RTX SDK licence ([RTXTF][rtxtf] [ref:nvidia-rtxtf-readme]). The 2025 sample reuse helps magnification only ([I3D 2025][stfreuse] [ref:wronski-2025-stf-reuse]); neural texture compression filters stochastically and resolves temporally ([NTC][ntc] [ref:vaidyanathan-2023-ntc-filtering]); ray cones choose the footprint for ray hits and keep the hardware's probes ([JCGT 2021][raycones] [ref:akenine-moller-2021-ray-cones]). No paper found since 2019 removes minification's box residue without history.

## Measured on the M4 Pro

#359's measurement patch, extended and never merged (`pending-artifacts/388/probe-388.patch` in the owner's main checkout, and `probe-388-per-sample.patch` for the per-sample build), adds variants of the lit pass's albedo sample chosen by `PRAXIS_PROBE` and `PRAXIS_PROBE_PARAM`: `textureSampleBias`, `textureSampleGrad` with scaled derivatives, and Gaussian-weighted trilinear probes along the major axis. A second build adds `@builtin(sample_index)` to `fs_main` and `sample` interpolation to its UV, which makes the lit pass shade each sample. Each image is `praxis capture minification` at frame 59, 2560×1440, on the M4 Pro (macOS 27.0.1, wgpu 30.0.1 on Metal, release build of `81b7e022`); the references are #359's exact checker mean over 3×3 subpixels, resized to 2560×1440 in linear light with a box or with `magick -filter gaussian -define filter:sigma=0.75`. RMSE is #359's (the capture reads 1.68 from the exact box, the Gaussian 9.05, as there); moiré here is the standard deviation of the band's grey minus its Gaussian blur of σ 6 px taken over the whole image, which reads 5.25 for the exact box where #359's table reads 6.01, so the moiré column compares within this table only.

| Image | Moiré | RMSE against exact, box | RMSE against exact, Gaussian σ 0.75 px |
|---|---:|---:|---:|
| Capture, `81b7e022` | 5.31 | 1.68 | 9.23 |
| Exact, box (the reference) | 5.25 | 0 | 9.05 |
| Exact, Gaussian σ 0.75 px | 1.18 | 9.05 | 0 |
| Gaussian probes, σ 0.75 px | 1.27 | 9.13 | 1.11 |
| Gaussian probes, σ 0.5 px | 2.96 | 6.09 | 4.13 |
| Footprint 1.5x (`textureSampleGrad`) | 3.16 | 7.98 | 4.68 |
| Footprint 2x | 2.05 | 10.25 | 3.00 |
| LOD bias +0.5 | 5.23 | 1.72 | 9.10 |
| LOD bias +1 | 5.00 | 2.14 | 8.67 |
| Capture blurred, Gaussian σ 0.75 px in linear light (a wide resolve, shading per pixel) | 1.40 | 8.37 | 1.90 |
| Per-sample shading, box resolve | 3.58 | 5.60 | 5.46 |
| Per-sample shading, footprint 0.5x | 4.85 | 3.61 | 8.30 |
| 2x2 supersamples of the capture, box | 5.29 | 0.99 | 9.15 |
| 2x2 supersamples, Gaussian σ 0.75 px | 1.20 | 8.99 | 0.97 |

Cost, from `praxis bench <dir> --skip-aliasing <scene>` runs interleaved with the unchanged sample (control), medians in ms, on 2026-10-30. Preflight read 42% to 90% GPU load from outside Praxis during the runs, so the deltas are indicative; the probe also keeps the hardware sample, which a real filter would replace.

| Variant | `minification` gate (3 rounds) | Δ | `town` gate (2 rounds) | Δ |
|---|---:|---:|---:|---:|
| Control | 0.886 to 0.904 | — | 8.75, 8.82 | — |
| Gaussian probes, σ 0.75 px | 1.166 to 1.174 | +0.28 | 9.42, 9.41 | +0.60 to 0.67 |
| Gaussian probes, σ 0.5 px | 1.164 to 1.279 | +0.28 to 0.38 | not run | |
| Footprint 1.5x | 1.006 to 1.024 | +0.12 | not run | |
| LOD bias +1 | 0.863 to 0.870 | −0.02 | not run | |
| Per-sample shading | 1.910 to 1.917 | +1.02 | 19.07, 20.35 | +10.3 to 11.5 |

`town`'s gate already reads past ADR 0013's 6 ms (#297). Crops of the band's left side at 2x, on the `pr-images` branch at `85f2ee3`: [references](https://raw.githubusercontent.com/gariasf/praxis/85f2ee3915d3babd843ca382856c82855970cab2/388/far-band-references-2x.png) (capture, exact box, exact Gaussian), [texture filters](https://raw.githubusercontent.com/gariasf/praxis/85f2ee3915d3babd843ca382856c82855970cab2/388/far-band-texture-filters-2x.png) (Gaussian probes σ 0.75 and 0.5, footprint 2x, bias +1), [resolves](https://raw.githubusercontent.com/gariasf/praxis/85f2ee3915d3babd843ca382856c82855970cab2/388/far-band-resolves-2x.png) (blurred capture, per-sample shading, exact Gaussian).

## Options (no recommendation)

Each reads no history, needs no new wgpu feature and leaves ADR 0019's 4x MSAA in place.

- **A. Keep the box.** No cost. The far band keeps its moiré, matching the exact box reference; nothing in the texture path is wrong.
- **B. Gaussian-weighted probes, σ 0.75 px, for the colour Channels.** About +0.3 ms on `minification`, +0.6 to 0.7 ms on `town`'s gate as probed. Moiré 5.31 to 1.27, matching the exact Gaussian; minified checks blur as much as the Gaussian blurs them (9 steps from the box reference). Restricting the probes to pixels past an anisotropy threshold would cut the cost, unmeasured.
- **C. The same at σ 0.5 px.** About the same cost. Moiré to 2.96, keeping more of the detail the box resolves (6.09 from the box reference).
- **D. A footprint widened 1.5 to 2x through `textureSampleGrad`.** +0.12 ms on `minification` at 1.5x; 2x not timed. At 1.5x moiré 3.16, 2.5 times B's, blurring less than B (7.98 from the box reference); at 2x moiré 2.05, blurring more than B (10.25) for more residue.
- **E. Per-sample shading with a wide custom resolve.** +10.3 to 11.5 ms on `town`'s gate, plus the resolve. The offline answer, reaching the exact Gaussian within about a step at 2x2; it filters geometry edges too, and changes ADR 0019's shading rate.

## Not verified

The GPU Pro 2 chapter on elliptical filtering (paywalled); Feline's extended reports; Real-Time Rendering 4th ed. §6.2.2 (no excerpt opened); AMD's documentation of its tent CFAA modes (only press found); the licence of the stochastic texture filtering paper's supplementary code; the 2021 appearance-driven mip generation paper. None is relied on.

[pbrt-sampling]: https://pbr-book.org/4ed/Sampling_and_Reconstruction/Sampling_Theory.html#IdealSamplingandReconstruction
[pbrt-filters]: https://pbr-book.org/4ed/Sampling_and_Reconstruction/Image_Reconstruction.html#BoxFilter
[heckbert]: https://www2.eecs.berkeley.edu/Pubs/TechRpts/1989/CSD-89-516.pdf
[mitchell]: https://www.cs.utexas.edu/~fussell/courses/cs384g-fall2013/lectures/mitchell/Mitchell.pdf#page=4
[vk-aniso]: https://github.com/KhronosGroup/Vulkan-Docs/blob/b94f8bd2d004d2949469d8a5071ee10eb7d23566/chapters/textures.adoc#L2769-L2826
[feline]: https://graphicon.ru/oldgr/library/siggraph/99/papers/mccormac/mccormac.pdf
[webgpu]: https://www.w3.org/TR/2026/CRD-webgpu-20260915/#per-sample-shading
[wgsl-interp]: https://www.w3.org/TR/2026/CRD-WGSL-20261026/#interpolation-sampling
[mjp-resolve]: https://therealmjp.github.io/posts/msaa-resolve-filters/
[order]: https://advances.realtimerendering.com/s2015/rad_siggraph_advances_2015.pptx
[msaafilter]: https://github.com/TheRealMJP/MSAAFilter/blob/cfc1c19411e24dc07f74272c17f423330ac96941/License.txt
[dx11samples]: https://github.com/TheRealMJP/DX11Samples/blob/900422d42ca173af31e96436e26df1f87ad5a89d/LICENSE
[omnimax]: https://www.cs.cmu.edu/~ph/omni.ps.gz
[mavridis]: https://graphics.cs.aueb.gr/graphics/docs/papers/I3D11_EllipticalFiltering_draft.pdf
[manson]: https://people.engr.tamu.edu/schaefer/research/cardinality.pdf
[wgsl-tex]: https://www.w3.org/TR/2026/CRD-WGSL-20261026/#texturesamplebias
[vk-lod]: https://github.com/KhronosGroup/Vulkan-Docs/blob/b94f8bd2d004d2949469d8a5071ee10eb7d23566/chapters/textures.adoc#L1688-L1717
[wgpu-sampler]: https://github.com/gfx-rs/wgpu/blob/40f4a34ebaf56f9a046231f54125ad046239d3f3/wgpu-types/src/texture.rs#L658-L683
[golus]: https://web.archive.org/web/20241214090822/https://bgolus.medium.com/sharper-mipmapping-using-shader-based-supersampling-ed7aadb47bec
[wronski]: https://bartwronski.com/2021/02/15/bilinear-down-upsampling-pixel-grids-and-that-half-pixel-offset/
[stf]: https://arxiv.org/pdf/2407.06107v1
[rtxtf]: https://github.com/NVIDIA-RTX/RTXTF/blob/992678902cbe95ef520e84153d60ad97eb124e54/README.md
[mavridis-demo]: http://web.archive.org/web/20200216143853/http://www.pmavridis.com:80/ewa.html
[stfreuse]: https://arxiv.org/pdf/2504.05562v1
[ntc]: https://arxiv.org/pdf/2305.17105v1#page=7
[raycones]: https://jcgt.org/published/0010/01/01/paper.pdf
