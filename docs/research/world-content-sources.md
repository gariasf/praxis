# World content: sources, licences, fit

Researched 2026-11-21 for ticket #947 (parent #944), against each publisher's own pages. It extends `docs/research/test-assets.md` from test models to the content a world is built from: terrain, vegetation, rocks and ground, water, vehicles, and the tools that generate them. Licences are quoted as the publisher states them, in `docs/references.toml`. This is engineering due diligence, not legal advice. Nothing was downloaded, chosen or added to the manifest: only licence and product pages were read, and two records of Poly Haven's API.

**Summary.** A public MIT repo can fetch real terrain and CC0 rocks, ground and plants in CI today; trees built for real time, grass and vehicles are the gaps.

- **Terrain is covered.** Public elevation data is free to redistribute, for a credit at most: Spain at 2 m and 5 m under CC BY 4.0, the United States at 1 m to 10 m in the public domain, Switzerland at 0.5 m under terms that ask a source line. It arrives as GeoTIFF rasters, which ADR 0011 does not name as a source format.
- **Rocks, ground and plants are covered by CC0 scans.** Poly Haven lists 110 nature models and ships glTF; ambientCG adds CC0 surfaces. Both are fetchable from the publisher.
- **Trees exist under CC0 but are not real-time assets.** A Poly Haven pine is 17.4 million polygons in a 949 MB buffer. The trees made with the modeller games use (SpeedTree's ORCA pack) are noncommercial, and Fab's licence bars sharing a file outside a private repository: both dev-only.
- **Generators split in two.** Blender's output belongs to its author, and EZ-Tree is MIT with CC0 and MIT textures: what they produce can be committed. Every free edition of a commercial terrain tool read here (Gaea, World Machine, Houdini) limits its output to noncommercial use, and The Grove forbids distributing its trees at all.
- **Vehicles are the widest gap.** The usable ones found are a toy car, a concept car carrying a trademark, and two OBJ cars with no rig.
- **Of the rows `test-assets.md` left unverified,** five are resolved (the McGuire archive, Fab, Sketchfab, Blender's demo files, glTF-Validator), as is MDT02's licence, and one stays open: Intel still names no version of its Creative Commons licence.

The note feeds one line of #944's *Not yet specified*, the Reference scene for world content, and ends in options for it. It decides nothing.

## Classes

As `docs/research/test-assets.md` and ADR 0025 set them: **committed** (a small file in `assets/`, with a manifest entry and a credit), **fetched** (by `praxis fetch` from a pinned URL with its hash, never committed) and **dev-only** (never committed, never fetched in CI, and no committed Golden rendered from it). A source whose licence allows redistribution and commercial use is committed or fetched by size; the table says *open* for those. A source under a noncommercial term, a bar on redistribution, or a licence the page does not confirm is dev-only.

## Sources

| Source | Offers | Licence as stated | Class | Formats | Notes |
|---|---|---|---|---|---|
| [IGN / CNIG MDT05 and MDT02](https://centrodedescargas.cnig.es) | Spain's terrain model at a 5 m and a 2 m grid | CC BY 4.0 for both: "MDT02-cob2 2015-2021 CC-BY 4.0 scne.es" [ref:ign-mdt02-metadata], as MDT05's record states [ref:ign-mdt05-metadata] | open: fetched | File format not read | MDT02's licence was unverified in `test-assets.md`; resolved. Whether the download centre serves a tile from a URL a script can pin was not checked |
| [USGS 3DEP](https://www.usgs.gov/3d-elevation-program/about-3dep-products-services) | US elevation: a seamless 1 m model in production since mid-2025, 1/3 arc-second (about 10 m) with full coverage, lidar point clouds | US public domain; credit asked, not required [ref:usgs-public-domain]. "free of charge and without use restrictions" [ref:usgs-3dep-products] | open: fetched | Cloud-optimised GeoTIFF, 10 km tiles | The least encumbered source of terrain |
| [swisstopo swissALTI3D](https://www.swisstopo.admin.ch/en/height-model-swissalti3d) | Switzerland without vegetation or buildings, 0.5 m or 2 m grid, 1 km² tiles | May be "used, distributed and made accessible", "enriched and processed and also used commercially"; "A reference to the source is mandatory" [ref:swisstopo-ogd-terms] | open: fetched | Cloud-optimised GeoTIFF | The finest grid found: alpine relief at 0.5 m [ref:swisstopo-swissalti3d] |
| [Copernicus DEM GLO-30](https://dataspace.copernicus.eu/explore-data/data-collections/copernicus-contributing-missions/collections-description/COP-DEM) | The globe at 30 m, as a surface model: buildings and vegetation are in it [ref:copernicus-dem-collection] | Its own licence: reproduction, distribution and adaptation, with a notice ending "all rights reserved" and the duty to bind whoever redistributes [ref:copernicus-dem-glo30-licence] | fetched; a committed derivative is the owner's call | GeoTIFF, DTED | Not CC: a committed heightfield cut from it would carry conditions MIT does not state. The Data Space asks for a registered user; the copy on AWS needs no account [ref:copernicus-dem-aws-registry] |
| [NASA Earth science data](https://www.earthdata.nasa.gov/engage/open-data-services-software-policies/data-use-guidance) | Data of NASA-led missions | CC0 unless a product is marked otherwise; citation urged [ref:nasa-esdis-data-use] | open | Not read | Which elevation products, their formats and whether a download needs a login were not read |
| [GEBCO grid](https://www.gebco.net/data-products/gridded-bathymetry-data) | Ocean floor and land on a 15 arc-second grid | "placed in the public domain" [ref:gebco-grid-terms] | open: fetched | netCDF, Esri ASCII, GeoTIFF | About 460 m at the equator: a seabed for an ocean's extent, not terrain |
| [Poly Haven](https://polyhaven.com/models) | 110 nature models (20 trees, 37 rocks, 54 plants, 32 ground cover, 4 grass; categories overlap), PBR textures, HDRIs [ref:polyhaven-api-nature-models] | CC0 [ref:polyhaven-asset-license] | open: fetched | glTF, Blender, FBX, USD [ref:polyhaven-api-pine-tree-files] | Rocks and nature models start from photogrammetry and are retopologised [ref:polyhaven-faq-photogrammetry]. 72 of the 110 set a `lods` flag. See *Quality*. The terms bar scraping [ref:polyhaven-terms-scraping] and the API asks a credit and a `User-Agent` [ref:polyhaven-api-conditions] |
| [ambientCG](https://ambientcg.com) | Materials, atlases, terrains, 3D models, HDRIs, by its type filter [ref:ambientcg-asset-types] | CC0 1.0 [ref:ambientcg-license] | open | Not read | What its Terrain and Atlas types hold, and at what resolution, was not read |
| [Fab](https://www.fab.com/eula), Quixel Megascans included | Marketplace content; what Megascans holds was not read | Fab Standard License: no standalone distribution "except to your collaborators", for example "in a private online repository" [ref:fab-eula-sharing]; not "in world- or level-editing tools or templates", not combined with GPL or CC BY-SA content [ref:fab-eula-restrictions] | dev-only | Per listing, not read | Rendered images may be distributed freely [ref:fab-eula-linear-media], yet ADR 0025 still keeps a committed Golden off a dev-only file. A listing offered under CC BY follows that licence instead. Tiers: see *Tiers and dates* |
| [Sketchfab](https://sketchfab.com/terms) | Uploaded models; "Cars & Vehicles" and "Nature & Plants" are among its categories | Per model. The uploader picks a Creative Commons licence or the store's, and "is solely responsible for choosing" it [ref:sketchfab-terms-free-downloads]. The store licence bars use "as a stand-alone file" [ref:sketchfab-store-licence] | per model: CC0 and CC BY open, by re-hosting; the rest dev-only | glTF, GLB, USDZ [ref:sketchfab-download-api] | A download needs a login, so CI cannot fetch from it. The licence that binds is the one "at the time of download": record it then. No model's licence was checked |
| [Khronos glTF-Sample-Assets](https://github.com/KhronosGroup/glTF-Sample-Assets) | `ToyCar`, `CarConcept`, `CesiumMilkTruck`, `DiffuseTransmissionPlant` | `CC0-1.0`; `CC-BY-4.0 AND LicenseRef-LegalMark-Khronos`; `CC-BY-4.0 AND LicenseRef-LegalMark-Cesium`; `CC-BY-4.0 AND CC0-1.0` [ref:khronos-sample-assets-reuse-vehicles] | open: fetched | glTF | Showcase models, read at the commit the manifest pins. The glob caveat of `test-assets.md` applies |
| [Smithsonian Open Access](https://www.si.edu/openaccess/faq) | Scans of museum objects | CC0 where an item is so designated; otherwise "subject to usage conditions" [ref:smithsonian-open-access-faq] | open, per item | glTF, GLB, OBJ | Which vehicles it holds was not surveyed |
| [McGuire Computer Graphics Archive](https://casual-effects.com/data) | Research meshes: trees and cars among them | Per model. White Oak: CC BY-NC-SA 3.0, © SpeedTree [ref:mcguire-white-oak]. Scrub Pine: CC0 [ref:mcguire-scrub-pine]. Sports Car: CC BY 4.0 [ref:mcguire-sports-car]. BMW: CC0 [ref:mcguire-bmw] | per model: the oak dev-only, the other three open | OBJ and PNG [ref:mcguire-archive] | Unverified in `test-assets.md`; resolved. Needs conversion to glTF. White Oak 36,760 triangles, Sports Car 300,957, BMW 385,079 |
| NVIDIA ORCA: [SpeedTree](https://developer.nvidia.com/orca/speedtree), [Emerald Square](https://developer.nvidia.com/orca/nvidia-emerald-square) | Eight SpeedTree plants (trees, a fern, a hedge, grass); a city park of 10,046,405 triangles that uses them | CC BY-NC-SA 3.0 for both [ref:orca-speedtree] [ref:orca-emerald-square] | dev-only | FBX | Trees from the modeller games use, and noncommercial. ORCA is marked a legacy archive |
| [Blender demo files](https://www.blender.org/download/demo-files/) | Scenes: splash artwork, Cycles demos, asset libraries | Per file: CC0, CC-BY or CC-BY-SA [ref:blender-demo-files-licences] | CC0 and CC-BY open; CC-BY-SA dev-only | `.blend` | Unverified in `test-assets.md`; resolved. Scenes, not kits; the forest scene read, Red Autumn Forest, is CC-BY-SA 3.0. Whether a Golden rendered from a share-alike file must be shared alike is not settled here, hence dev-only |
| [Intel samples](https://www.intel.com/content/www/us/en/developer/topic-technology/graphics-research/samples.html): Jungle Ruins, Sponza's trees | A rainforest scene (4.0 GB); cypresses whose leaves are alpha cards | "the Creative Commons Attribution license": still no version and no licence text [ref:intel-samples-jungle-ruins] | dev-only until Intel names the version | Blender and USD; the Sponza add-ons glTF [ref:intel-samples-sponza] | Stays unresolved, as `test-assets.md` left Sponza |
| [Disney Moana Island Scene](https://disneyanimation.com/resources/moana-island-scene/) | A film island: instanced vegetation, volumes | "may only be used for research or software development (including benchmarking) purposes" [ref:disney-moana-island-licence] | dev-only | USD and pbrt beside a 45 GB base package [ref:disney-moana-island-packages] | No glTF |
| [ShareTextures](https://www.sharetextures.com/p/license) | Textures and models | "a custom CC0-based license": no redistribution, "No automated downloads" [ref:sharetextures-licence] | dev-only | Not read | Called CC0 and is not: a fetch in CI is what it forbids |
| **Generators** | | *Tool; then its output* | | | |
| [Blender](https://www.blender.org/about/license/) | Modelling, Geometry Nodes, glTF export | Tool: GNU GPL. Output: "What you create with Blender is your sole property" [ref:blender-license-artwork] | output open, as the author's own work | glTF by export | Already the tool behind `scripts/cycles-reference/` |
| [EZ-Tree](https://github.com/dgreenheck/ez-tree) | A procedural tree generator, as a library and a web app | Tool: MIT [ref:ez-tree-licence]. Output: no statement. Its bark Textures are CC0 from ambientCG and its leaf Textures MIT [ref:ez-tree-texture-licence] | output open; the leaf Textures keep MIT's notice | GLB | One bark and one leaf Material across its LOD levels; leaves are quads [ref:ez-tree-readme] |
| [Infinigen](https://github.com/princeton-vl/infinigen) | Procedural worlds, built on Blender | Tool: BSD 3-Clause, © Princeton University [ref:infinigen-licence]. Output: no statement | output unverified | OBJ, OpenUSD [ref:infinigen-readme] | What it generates and at what cost was not read |
| [SpeedTree](https://unity.com/products/speedtree) | A vegetation modeller and a tree library | Tool: paid plans by revenue, from Indie under 100k; a 30-day trial with limited export [ref:speedtree-plans]. Output: terms not read | dev-only until read | OBJ, FBX, USD, with atlases and LODs | The licence that governs redistributing an exported tree was not found on the product page |
| [The Grove](https://www.thegrove3d.com/buy/) | A tree growth simulator for Blender | Tool: paid. Output: "Do not sell or distribute 3D models of trees grown with The Grove" [ref:the-grove-licence] | dev-only | via Blender | A tool whose output cannot be published as a file |
| [Gaea](https://quadspinner.com/Order) | Terrain with erosion | Tool: Community edition free, "for non-commercial purposes, including but not limited to Assets"; paid editions by revenue. Output of a paid edition: "You may sell or distribute Assets you create", with a credit line [ref:gaea-eula] | Community: dev-only. Paid: open, a purchase the owner decides | Meshes, displacement and colour maps | Community builds at most 1024 × 1024 [ref:gaea-editions] |
| [World Machine Basic](https://www.world-machine.com/download.php) | Terrain with erosion | "Free for non-commercial use"; a commercial work "containing World Machine output" needs a full licence [ref:world-machine-basic] | dev-only | Not read | Output at most 1025 × 1025 |
| [Houdini Apprentice](https://www.sidefx.com/get/try-houdini/) | Procedural modelling, terrain included | "free for personal, non-commercial projects" [ref:houdini-apprentice] | dev-only | Not read | |
| [Tree It](http://www.evolved-software.com/treeit/treeit) | A tree generator with adjustable LOD | Tool: "Free". Output: no statement on the page [ref:treeit-overview] | dev-only until stated | FBX, OBJ | |

## Quality

The surveys of #944 had not landed when this was written, so quality is given in the terms the ticket names, from what the publishers state.

- **Scan resolution.** Poly Haven's nature models carry Textures of 4096², 8192² or 16384², its trees 8192². ambientCG's and Megascans' resolutions were not read.
- **LOD chains.** No open source states one built for real time. Poly Haven sets a `lods` flag on 72 of 110 nature models and does not say how many levels it means. Its `pine_tree_01` is 17,427,094 polygons with a 948,849,556-byte glTF buffer [ref:polyhaven-api-pine-tree-files]: a film asset, which the cooker would have to simplify into a chain before any budget holds. EZ-Tree generates its levels from one skeleton, with fewer and larger leaves further out. SpeedTree exports LODs and atlases; the Sponza cypresses and EZ-Tree use leaf cards.
- **Leaves and bark as separate Materials.** Poly Haven's pine keeps bark, trunk and twig Texture sets apart. EZ-Tree has one bark and one leaf Material. The others were not read to this depth, since that needs the files.
- **Texel density.** Poly Haven publishes a figure per model (`texel_density`), without its unit; across the 20 trees it spans 392 to 230,007, so the library is not uniform and a scene would have to normalise it.

## Tiers and dates

- **Fab.** "Both pricing tiers (Personal and Professional) grant you the same scope of rights"; the tiers differ in who may buy them, by revenue under or over 100,000 USD in the last twelve months [ref:fab-eula-tiers]. The agreement read is dated October 1st, 2024. Content acquired from Quixel before Fab, under the terms of that time, was not read.
- **Gaea.** Editions are tied to revenue (Indie to 100,000 USD, Professional to 1,000,000), [ref:gaea-eula], and a licence update binds only versions released after it: an installed version keeps the agreement it shipped with [ref:gaea-eula-versions].
- **Sketchfab.** The licence that binds is the one in force when the file is downloaded [ref:sketchfab-terms-free-downloads], so the record to keep is the licence the page shows on that day.

## The rows `test-assets.md` left unverified

- **McGuire Computer Graphics Archive**: resolved. The page frames `g3d/data10/index.html`, whose models each carry a licence in an `info.js` record; the four read are in the table.
- **Fab and Megascans**: the licence is resolved (Fab Standard License, dev-only). Whether Megascans listings are free of charge was not confirmed: Quixel's page rendered empty on the date read.
- **Sketchfab**: resolved as a per-model licence chosen by the uploader, behind a login.
- **Blender demo files**: resolved as per-file CC0, CC-BY or CC-BY-SA.
- **glTF-Validator**: Apache-2.0 [ref:gltf-validator-licence]: usable as a tool or dependency under the legal filter of #944.
- **MDT02**: CC BY 4.0, as MDT05.
- **Intel's licence version**: still unstated on 2026-11-15's snapshot of the page. Stays marked.

## Options for a world-content Reference scene

Each uses only sources CI can fetch or the repo can hold. They combine: A is the floor under any of the others.

- **A. Procedural, in the repo.** A heightfield from noise and erosion written here, trees from a generator written here. No licence, deterministic, tiny (Tier 1 of `test-assets.md`). It exercises every system and shows nothing of how scanned content behaves.
- **B. Real terrain, CC0 scans.** A crop of public elevation (USGS 3DEP in the public domain, or MDT02 under CC BY 4.0 for a credit), dressed with Poly Haven rocks, ground cover and ground Materials. All fetched. It needs a heightfield import that does not exist, and the fetch is large: one tree alone is about 1 GB. The Poly Haven URL read carries no version, so a re-upload would fail the hash until the manifest moves.
- **C. Real terrain, generated trees.** B's terrain and rocks, with trees authored once in Blender or EZ-Tree and kept as small glTF files, committed or fetched from the repo's own hosting. The LOD chain, leaf cards and Material split are then under the engine's control. The trees look generated, not scanned.
- **D. A dev-only realism set beside any of them.** Megascans, the ORCA SpeedTree pack or Emerald Square on the owner's machine, for the visual audit and local benches. No Golden, no CI, no file in the repo.

B and C differ in where the trees come from; the terrain survey decides whether real elevation or a generated heightfield is the better test.

## Gaps

- **Vehicles.** No source read offers a photoreal vehicle as glTF under CC0 or CC BY that is known to be built for driving: wheels as separate nodes, a collision Mesh. `ToyCar` is a toy; `CarConcept` carries a trademark notice; the McGuire cars are OBJ files with no rig stated, the BMW "a single car with wheels in a default pose". Whether any of them separates its wheels was not checked, since that needs the file.
- **Trees for real time.** CC0 trees exist and are film assets. A chain down to cards or an impostor, wind data and a tested Material split come only with SpeedTree-class content, which is noncommercial (ORCA) or sold under terms not read.
- **Grass.** Poly Haven lists four grass models. No CC0 library of grass cards or atlases was confirmed; ambientCG's Atlas type was not read.
- **Water.** A water surface needs no asset. No source was read for flow maps, foam Textures or shoreline data, and bathymetry stops at GEBCO's 15 arc-seconds.
- **Terrain as a source format.** Every elevation source delivers GeoTIFF, and ADR 0011 names glTF. What a heightfield is authored and cooked from is a question for the terrain survey.
- **Terrain colour.** Orthophotos and land-cover data, which a terrain's Materials would be painted from, were not surveyed.

## Not verified

- Megascans' price, the terms of content acquired before Fab, and that its listings sit under the Fab Standard License and no terms of their own: no Megascans listing was read.
- SpeedTree's terms for redistributing an exported model; Tree It's, EZ-Tree's and Infinigen's for their output, which their pages do not state.
- Intel's Creative Commons version.
- Whether NASA's and CNIG's downloads can be pinned to a URL without a login or a form; MDT02's file format.
- What ambientCG's Terrain and Atlas types and Smithsonian's collection hold.
- What Poly Haven's `lods` flag and `texel_density` unit mean, and whether its glTF packages carry the LODs.
- The licence of any single Sketchfab or Fab listing.
- blender.org, intel.com and si.edu refused a direct fetch; their pages were read from web.archive.org snapshots of 2026-11-15 to 2026-11-18.
