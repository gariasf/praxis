# Audit lane 2, image quality

Draft, prepared to the point of judgement. Every Reference scene captured at
2560×1440 at the frozen commit `8bc34eb18fcfb79307193551e96619316aef566c`
by `scripts/gallery.sh` with `PRAXIS_BLENDER`, `town` at its eight frames
and `crowd` in both shadow forms; each exported scene beside its Cycles
render with its difference image, its two distance figures recorded as the
scene's first Cycles Baseline (`bench/cycles/<scene>.json`); the 3x
box-filtered supersampled reference for the scenes where aliasing is the
question (`town`, `minification`, `texture-wall`); and the candidate 2x
crops below, each praxis | Cycles | difference, with the `DIFFERENCES.md`
item that explains it or `none`. **No Finding is filed here.** The owner
judges every crop in the judgement session; a crop the owner reads as a
defect becomes a Finding of class `image` then, with this draft's row as
its evidence.

## Summary

At native resolution every difference between praxis and Cycles that a
candidate crop shows reads as one of the five known differences in
`scripts/cycles-reference/DIFFERENCES.md`, the ambient without sky
visibility first: it is the whole of `town`'s shade gap, the owner's
largest visible gap at the previous milestone's exit, and it reaches every
scene with geometry that hides the sky. Three candidates sit at the edge of
the list rather than inside it and are the ones to look at longest: the
rim of a smooth metal sphere in `pbr-sweep` and the car bodies in `town`,
where Cycles is the brighter one (its sky reflected at grazing, which
praxis's Lambertian ambient has no lobe for), and a bright rim all round
`orrery`'s planet that only praxis draws. Against the supersampled
reference, `town` and `texture-wall` differ only along edges and wires, as
4x MSAA against nine samples predicts; `minification` differs across its
whole floor, where the specular antialiasing and Toksvig widening are
resolution-dependent and the box filter itself leaks the checker's moiré.

## Run

| | |
|---|---|
| Commit | `8bc34eb18fcfb79307193551e96619316aef566c` (praxis built from it; `scripts/gallery.sh` and `scripts/cycles-reference/compare.py` taken from `main` at `88868582`, the two files that differ, for the Baseline recording; no source file differs between the two commits) |
| Machine | Apple M4 Pro, Metal, on the charger (94%, charging), `caffeinate -dims`, no peer benching (asked and confirmed) |
| Load average at start | 18.5 (peers' readers; captures are deterministic, see the cross-check) |
| Blender | 5.2.3 LTS, Cycles on Metal |
| Resolution | 2560×1440; supersampled captures 7680×4320 box-filtered to 2560×1440 in linear light by ImageMagick 7.1.2 |
| Wall time | 7 min 35 s for the gallery with Cycles; about 1 min for the three supersampled captures |
| Cross-check | the figures equal the three galleries run at this commit when the Baseline mechanism landed, to the third decimal (`town` 4.898 against 4.90; `minification` 11.127 against 11.13) |

Everything the run produced is under `pending-artifacts/m3.5-lane-2/`
(git-ignored): `gallery/` (the 55 images and `logs/`), `supersampled/`,
`crops/` (121 ranked candidate tiles), `crops4x/` (the subtle ones at 4x),
`crops-supersampled/` (praxis | supersampled | difference), `sheets/` (one
contact sheet per image), `candidates.md` (every tile with its scene, frame,
pixel box and figures), `gallery-run1.log`, `supersample.log`, and the two
scripts that made the tiles and the supersampled references.

## Distance from Cycles

The gallery's figures, pasted from its log: the mean 8-bit difference of the
display colours and the RMSE of the display-linear light, each the Baseline
recorded for the scene at the Goldens' frame.

| Image | Mean difference, 8-bit steps | Linear RMSE | Baseline |
|---|---:|---:|---|
| town 59 | 4.898 | 0.00750 | recorded |
| town 300 | 4.634 | 0.00983 | |
| town 500 | 3.688 | 0.01039 | |
| town 700 | 4.680 | 0.01376 | |
| town 900 | 4.848 | 0.01373 | |
| town 1200 | 4.925 | 0.01367 | |
| town 1500 | 5.060 | 0.01325 | |
| town 1800 | 5.265 | 0.01314 | |
| helmet 59 | 2.439 | 0.01414 | recorded |
| pbr-sweep 59 | 1.051 | 0.00719 | recorded |
| uv-grid 59 | 0.722 | 0.00342 | recorded |
| normal-tangent 59 | 1.211 | 0.02093 | recorded |
| far-coplanar 59 | 0.902 | 0.00405 | recorded |
| orrery 59 | 0.544 | 0.00227 | recorded |
| minification 59 | 11.127 | 0.08516 | recorded |
| texture-wall 59 | 0.516 | 0.00209 | recorded |
| shadows 59 | 2.230 | 0.01108 | recorded |
| crowd proxies 59, crowd refit 59 | | | not exported: no Cycles render |

## Against the supersampled reference

`praxis difference` of the 1440p capture against the 3x box-filtered
capture of the same frame: pixels past the Goldens' tolerance (one 8-bit
step) and the largest channel delta.

| Scene | Pixels past tolerance | Max channel delta | What they are |
|---|---:|---:|---|
| town 59 | 91,566 | 138 | silhouettes, roof lines, railings and wires: the one-pixel lines 4x MSAA quantises against nine samples; the near cars' bodies at one to two steps across |
| minification 59 | 1,371,831 | 173 | the whole floor: the sheen is wider at 1x than at 3x (the specular antialiasing and Toksvig widening depend on the pixel footprint), and the box filter leaks the checker's moiré where praxis's mips average it away |
| texture-wall 59 | 32,540 | 49 | the tiles' edges only |

The 3x hardware capture is praxis's own sampler at a finer footprint, not
an independent reference: it resolves what MSAA cannot place, not what the
shading gets wrong. It belongs to the silhouette rows below; the Cycles
render remains the reference for shading.

## Candidates

Each row is one crop, praxis | Cycles | difference at 2x (the difference:
praxis dimmed to a quarter, red where praxis is brighter, blue where Cycles
is), at `pending-artifacts/m3.5-lane-2/crops/<scene>-frame-<n>-<x>_<y>_160x90.png`;
`4x` marks a copy in `crops4x/` at point-sampled 4x. The pixel box is
`x, y, width, height` in the 2560×1440 capture. The item is the heading in
`scripts/cycles-reference/DIFFERENCES.md` that explains the crop, with a
note where the match is partial; `none` where no item does. The 121-tile
ranked index these were chosen from is `candidates.md` beside them.

### Terminators

| Id | Scene, frame | Pixel box | What the crop shows | Item |
|---|---|---|---|---|
| C01 | far-coplanar 59 | 0, 0, 160, 90 | the brown sphere's terminator: praxis grades into a lit dark side, Cycles is black past it (the plane below hides the sky) | Ambient light without visibility |
| C02 | normal-tangent 59 | 1600, 180, 160, 90 (4x) | a dome's terminator and its dark side: praxis lit, Cycles black; a one-pixel blue line along the dome's silhouette against the floor | Ambient light without visibility; 4x MSAA against supersampling |
| C03 | pbr-sweep 59 | 1920, 900, 160, 90 | the rough metal sphere's shaded half: praxis brighter by a few steps everywhere only the ambient reaches | Ambient light as an ideal diffuse response |
| C04 | shadows 59 | 1120, 810, 160, 90 (4x) | a checkered sphere's terminator: the same shapes; grid lines brighter in praxis where minified; a one-pixel line at the silhouette | BC compression and mips against source images; 4x MSAA against supersampling |

### Highlights

| Id | Scene, frame | Pixel box | What the crop shows | Item |
|---|---|---|---|---|
| C05 | helmet 59 | 480, 1080, 160, 90 (4x) | the three point lights on the chin's metal: praxis's highlights wider and softer, Cycles' tight with coloured speckle along the tubes | Highlights widened by specular antialiasing and Toksvig; the speckle is Cycles' sampling noise |
| C06 | helmet 59 | 480, 1170, 160, 90 | the same lights lower on the chin; the metal between them grey in praxis, black in Cycles | Highlights widened; Ambient light without visibility (reflections of the helmet itself) |
| C07 | pbr-sweep 59 | 480, 720, 160, 90 (4x) | a smooth metal sphere's highlight: praxis's halo wider; a black blob in Cycles where the sphere reflects its neighbour | Highlights widened; the reference is direct light only (a reflection of another surface is black) |
| C08 | pbr-sweep 59 | 960, 810, 160, 90 (4x) | a rough sphere's red highlight: praxis's core wider, and a faint ring outside it where Cycles is brighter | Highlights widened (the item names a red ring; the blue ring outside it is the same lobe difference seen from the other side) |
| C09 | orrery 59 | 1280, 450, 160, 90 (4x) | the lamps' highlights on the metal planet: wide soft halos in praxis, points in Cycles; a black band in Cycles where the planet reflects its ring | Highlights widened; direct light only |
| C10 | orrery 59 | 800, 720, 160, 90 (4x) | a thin bright rim all round the planet's silhouette in praxis; Cycles has only the sun's crescent at the top | none listed as such. Nearest: Highlights widened, if the rim is the sun's grazing highlight spread by the specular antialiasing where the normal turns fastest. Worth the owner's eye |

### Silhouettes

| Id | Scene, frame | Pixel box | What the crop shows | Item |
|---|---|---|---|---|
| C11 | town 59 | 1120, 630, 160, 90 | the distant street: wires, roof lines and window edges as alternating red and blue one-pixel lines | 4x MSAA against supersampling |
| C12 | town 500 | 1760, 900, 160, 90 (4x) | rooftop railings and clutter: every bar brighter in praxis, the clutter's undersides darker in Cycles | 4x MSAA against supersampling (bars thinner than a pixel); Ambient light without visibility |
| C13 | town 500 | 0, 1260, 160, 90 (4x) | a railing against a shaded wall, with the roof's shadow band | 4x MSAA against supersampling; Ambient light without visibility |
| C14 | pbr-sweep 59 | 640, 720, 160, 90 (4x) | a smooth metal sphere's rim: a band inside the silhouette where Cycles is brighter, and a blue one-pixel edge | none. Nearest: Ambient light as an ideal diffuse response, which predicts praxis brighter towards rims; here Cycles is brighter, its white sky reflected at grazing, a specular lobe praxis's ambient does not have. Worth the owner's eye |
| C15 | far-coplanar 59 | 1280, 630, 160, 90 | the coplanar discs' edges: one-pixel coloured lines, no z-fighting in either | 4x MSAA against supersampling |
| C16 | uv-grid 59 | 1120, 360, 160, 90 | the plane's far edge against the sky and the tile seams | 4x MSAA against supersampling; BC compression and mips against source images |
| S01 | town 59 | 640, 450, 160, 90 (`crops-supersampled/`) | praxis against its 3x capture: a wire and a railing as coloured specks | 4x MSAA against supersampling |
| S02 | town 59 | 1920, 990, 160, 90 (`crops-supersampled/`) | the near car's body against its 3x capture: within a step or two across the whole body | 4x MSAA against supersampling (the specular antialiasing's footprint) |

### Shadow edges

| Id | Scene, frame | Pixel box | What the crop shows | Item |
|---|---|---|---|---|
| C17 | town 59 | 800, 540, 160, 90 | a lamp-lit wall with the tree's shadow and the lamp post: the shadow's edge the same in both; the wall noisy in Cycles; the post grey all over in praxis, dark with one bright line in Cycles | Ambient light without visibility; Ambient light as an ideal diffuse response (a metal post under praxis's ambient); the wall's speckle is Cycles' sampling noise |
| C18 | town 300 | 640, 540, 160, 90 (4x) | the same wall and post from the next camera position | as C17 |
| C19 | town 1800 | 480, 1350, 160, 90 | a lamp at a tree's base on a pink wall: the tree's shadow in lamp light, the wall brighter in praxis | Ambient light without visibility; Cycles' sampling noise |
| C20 | town 700 | 160, 630, 160, 90 | a street between buildings from above: lamp pools with bright specks in Cycles, smooth in praxis; the street walls brighter in praxis | Cycles' sampling noise; Ambient light without visibility |
| C21 | shadows 59 | 1120, 720, 160, 90 | a cast shadow's edge on the floor and the light grid's lines on the sphere | Ambient light without visibility (the penumbra's dark side); BC compression and mips against source images |

### Shade

| Id | Scene, frame | Pixel box | What the crop shows | Item |
|---|---|---|---|---|
| C22 | town 59 | 0, 90, 160, 90 | a shaded wall: praxis uniformly brighter, the largest gap of the scene | Ambient light without visibility |
| C23 | town 59 | 160, 810, 160, 90 | shaded pavement and a wall's foot | Ambient light without visibility |
| C24 | town 59 | 960, 720, 160, 90 (4x) | bollards and parked cars: the bollards lit grey in praxis, dark in Cycles; the blue car's body brighter and noisy in Cycles | Ambient light without visibility (bollards); the car: none. Nearest: Ambient light as an ideal diffuse response, with the opposite sign, the paint reflecting Cycles' sky. Worth the owner's eye |
| C25 | town 300 | 2240, 900, 160, 90 | a blue car beside a shaded wall | as C24 |
| C26 | helmet 59 | 1920, 1170, 160, 90 | the grey housing's diffuse surfaces: praxis a few steps brighter everywhere | Ambient light as an ideal diffuse response |
| C27 | normal-tangent 59 | 1600, 630, 160, 90 | a dome's lit side with its normal map: shading the same, praxis brighter in the shade | Ambient light without visibility |

### Texture filtering

| Id | Scene, frame | Pixel box | What the crop shows | Item |
|---|---|---|---|---|
| C28 | minification 59 | 1120, 630, 160, 90 | the floor at the sun's reflection: praxis's broad sheen against Cycles' streak; moiré beyond it | Highlights widened by specular antialiasing and Toksvig; BC compression and mips against source images |
| C29 | minification 59 | 960, 540, 160, 90 | the far floor: praxis's smooth bands against Cycles' aliased checker | BC compression and mips against source images |
| C30 | texture-wall 59 | 1120, 270, 160, 90 | tiles and their edges: within a step | BC compression and mips against source images |
| C31 | uv-grid 59 | 1280, 540, 160, 90 | the tile seams: the dark lines lighter in praxis where minified | BC compression and mips against source images |
| S03 | minification 59 | 1120, 630, 160, 90 (`crops-supersampled/`) | praxis against its 3x capture: red where the 1x sheen is wider, blue where the 3x capture keeps checker detail | Highlights widened; the box filter's own moiré leak |
| S04 | minification 59 | 960, 540, 160, 90 (`crops-supersampled/`) | the far floor against its 3x capture: a faint red field | as S03 |
| S05 | texture-wall 59 | 1120, 270, 160, 90 (`crops-supersampled/`) | tiles against their 3x capture: nothing past the edges | 4x MSAA against supersampling |
| S06 | town 59 | 160, 540, 160, 90 (`crops-supersampled/`) | a shaded wall against its 3x capture: identical | none needed: no difference |

## For the judgement session

1. The gap to look at first is the shade: C22, C17, C24, C12, with the
   owner's note from the previous exit that Cycles' town shade looks far
   better. The item that explains it names its own trigger (the probes).
   Whether an interim sky-visibility term is worth a ticket before then is
   the owner's call at the triage, not this draft's.
2. The three edge-of-list crops, C10, C14 and C24's car, are where a
   Finding could come from: each is a specular response to the sky or the
   sun at grazing that praxis either lacks (C14, C24) or spreads (C10).
3. Cycles' speckle in lamp-lit `town` surfaces (C17 to C20) is on the
   reference's side; a higher sample count for the point lights would clean
   it, and the figures would move by it, so a change there re-records the
   Baselines.
4. The tolerance the Baselines are judged with is the one accepted when the
   mechanism landed: 0.01 steps of mean difference and 0.0001 of linear
   RMSE, against a measured run-to-run spread of under 1e-6 steps.
