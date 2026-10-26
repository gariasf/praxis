# Praxis

A 3D engine in Rust for large open-world games on cutting-edge hardware.

## Reference scenes

Each scene's Golden on Metal, rendered on the M4 Pro at 640×360 with
`praxis capture <scene>`. The images are the committed Goldens in
`tests/goldens/metal/`, so re-recording one updates this page;
`cargo test --test goldens` checks this list against the scenes it checks.

| Golden | Scene |
| --- | --- |
| <img src="tests/goldens/metal/pbr-sweep.png" width="320" alt="pbr-sweep"> | **pbr-sweep**: spheres across metallic (rows) and roughness (columns), the lit pass's BRDF at a glance. |
| <img src="tests/goldens/metal/uv-grid.png" width="320" alt="uv-grid"> | **uv-grid**: a ground plane with a generated grid texture, tiled twice. |
| <img src="tests/goldens/metal/normal-tangent.png" width="320" alt="normal-tangent"> | **normal-tangent**: Khronos NormalTangentTest face on, real bumps beside normal-mapped quads that should shade the same. |
| <img src="tests/goldens/metal/far-coplanar.png" width="320" alt="far-coplanar"> | **far-coplanar**: two planes 1 cm apart, 2 km away at a grazing angle; under reversed depth the upper one covers the lower everywhere. |
| <img src="tests/goldens/metal/orrery.png" width="320" alt="orrery"> | **orrery**: five textured spheres circling and spinning in their parents' frames, moved per Tick and drawn blended between Ticks. |
| <img src="tests/goldens/metal/minification.png" width="320" alt="minification"> | **minification**: a glossy checker floor 100 m long under one raking point light, showing mips, anisotropic filtering and Toksvig. |
| <img src="tests/goldens/metal/texture-wall.png" width="320" alt="texture-wall"> | **texture-wall**: 10,000 resident Textures, each one colour encoding its index, sampled by 100 quads across the whole Layer range. |
| <img src="tests/goldens/metal/shadows.png" width="320" alt="shadows"> | **shadows**: a sphere over a plane under the sun and one point light, its ray-traced shadow matching the analytic one. |
| <img src="tests/goldens/metal/town.png" width="320" alt="town"> | **town**: a procedural 1 km town of about 250,000 Instances, with flocks of pigeons and 1,024 street lamps, seen from street level. |

## License

[MIT](LICENSE)
