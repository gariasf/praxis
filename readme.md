# Praxis

A 3D engine in Rust for large open-world games on cutting-edge hardware.

## Reference scenes

Each Reference scene with a Golden, shown by it: the image the scene's render must reproduce, as the M4 Pro renders it on Metal. The images are the committed Goldens in `tests/goldens/metal/`, so the table always shows the current ones.

| Golden | Scene |
| --- | --- |
| <img src="tests/goldens/metal/town.png" alt="town" width="320"> | **town**: a procedural town of 1 × 1 km, with about 250,000 Instances, 10,000 of them pigeons circling in flocks, and 1,024 street lamps. |
| <img src="tests/goldens/metal/shadows.png" alt="shadows" width="320"> | **shadows**: a sphere over a plane under the sun and one point light, its shadows ray-traced per pixel; the sun's matches its analytic shadow. |
| <img src="tests/goldens/metal/pbr-sweep.png" alt="pbr-sweep" width="320"> | **pbr-sweep**: spheres across metallic (rows) and roughness (columns), under the sun and four coloured point lights. |
| <img src="tests/goldens/metal/orrery.png" alt="orrery" width="320"> | **orrery**: five textured spheres, each circling and spinning in its parent's frame, moved per Tick and drawn blended between Ticks. |
| <img src="tests/goldens/metal/minification.png" alt="minification" width="320"> | **minification**: a glossy checker floor 100 m long, seen at a grazing angle under one raking point light: its fine checks and bumps show mips, anisotropic filtering and Toksvig. |
| <img src="tests/goldens/metal/normal-tangent.png" alt="normal-tangent" width="320"> | **normal-tangent**: Khronos NormalTangentTest seen face on, its real bumps beside normal-mapped quads that should shade the same. |
| <img src="tests/goldens/metal/uv-grid.png" alt="uv-grid" width="320"> | **uv-grid**: a ground plane with a generated grid texture, tiled twice. |
| <img src="tests/goldens/metal/texture-wall.png" alt="texture-wall" width="320"> | **texture-wall**: 10,000 Textures of 4 × 4 texels resident at once, and a 10 × 10 grid of quads, each with its own Material. |
| <img src="tests/goldens/metal/far-coplanar.png" alt="far-coplanar" width="320"> | **far-coplanar**: two planes of contrasting colour 1 cm apart, 2 km out, seen at a grazing angle. Under reversed depth the upper one covers the lower everywhere, so the image is one colour; under standard depth they would interleave. |

## License

[MIT](LICENSE)
