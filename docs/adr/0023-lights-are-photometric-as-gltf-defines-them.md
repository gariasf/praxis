# Lights are photometric, as glTF defines them

Light values are photometric, in glTF's units: a `DirectionalLight` gives illuminance in lux and a `PointLight` luminous intensity in candela, as KHR_lights_punctual defines them; emissive and background give luminance in cd/m², and a Reference scene's ambient term gives illuminance in lux. Exposure is an EV100, which scales luminance by 1/(1.2 · 2^EV100) so that 1.0 lands near display white. glTF lights then import without a scale, and ADR 0020's ray-traced shadows and probe GI work on real ratios between sun, sky and interiors, 29.9 stops from a moonless night to direct sun (`docs/research/hdr-exposure-and-tonemapping.md`, option U1, rows 11, 14 and 15). Accepted 2026-10-20 ([#189](https://github.com/gariasf/praxis/issues/189)). Partly implemented: every Reference scene's lights, ambient and background are in these units, under a sun of 100,000 lx, and each scene has an EV100, which the lit pass applies before the `Rgba16Float` HDR target stores its colour, clamped to 65504 (#200).

## Considered options

- Relative units (option U2): a sun near 1 and unitless point lights, so fewer stops and an exposure near 1. glTF lights would need a scale at import, values tuned in one scene do not carry to another, and Frostbite found converting relative lights to physical units "extremely hard" once content existed (row 14).
- Point lights in lumens, as Frostbite, Filament, Bevy and Godot author them: a bulb's familiar rating, but glTF stores candela, so every import would convert (Bevy's glTF loader multiplies by 4π).

## Consequences

- fp16 cannot hold the range unexposed: under 100,000 lx, a dielectric's sun glint at roughness 0.04 peaks at 1.24×10⁸ cd/m², 10.9 stops past fp16's 65504. Shading runs in f32 and pre-exposes colour before fp16 stores it, and every path clamps it to 65504, since on overflow the M4 Pro saturates while Vulkan may store infinity, which shows as black patches.
- Each Reference scene sets its EV100 by hand in M3, which keeps Goldens exact. Histogram adaptation and local tone mapping wait for the first scene that joins an interior and sunlight or changes the time of day (map issue, *Not yet specified*).
- Core glTF caps emissive at 1 cd/m² (texture and factor each at most 1), which an exposure of EV100 15 maps to 2.5×10⁻⁵ of display white. Brighter emissives need KHR_materials_emissive_strength, which the cooker does not read yet.
- Point lights fall off by inverse square inside KHR_lights_punctual's range window (#199). Spot lights, out of M3, take candela too.
- Blender's glTF exporter, in its physically based lighting mode (`SPEC`), writes a point light's watts as W/(4π)·683 candela and a sun's W/m² as W/m²·683 lux, so lights authored in Blender arrive in these units.
