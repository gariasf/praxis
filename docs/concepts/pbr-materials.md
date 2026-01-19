# PBR Materials

Physically-Based Rendering (PBR) in Praxis uses the metallic-roughness workflow for realistic material appearance.

## Core Parameters

### Albedo (Base Color)
The intrinsic color of the surface without lighting effects.

- **RGB**: Surface color
- **Range**: 0.0 - 1.0

For metals, albedo represents the reflectance color (gold is yellow-orange). For dielectrics, it's the diffuse color.

### Metallic
Whether the surface is a metal or non-metal (dielectric).

- **0.0**: Non-metal (plastic, wood, stone, skin)
- **1.0**: Metal (iron, gold, copper, aluminum)

Physical difference:
- Metals have no diffuse reflection (light doesn't penetrate)
- Metals have colored specular highlights
- Dielectrics have white specular (Fresnel) and colored diffuse

### Roughness
How rough or smooth the surface is at a microscopic level.

- **0.0**: Perfectly smooth (mirror, polished metal)
- **1.0**: Completely rough (matte paper, chalk)

Roughness affects the specular highlight:
- Smooth surfaces: tight, bright specular
- Rough surfaces: wide, dim specular

### Emissive
Self-illumination intensity. Added to final color regardless of lighting.

- **0.0**: No emission
- **1.0+**: Glowing (screens, neon, lava)

## Material Properties in Praxis

```rust
#[repr(C)]
pub struct MaterialProperties {
    pub albedo: [f32; 4],     // RGBA base color
    pub metallic: f32,        // 0 = dielectric, 1 = metal
    pub roughness: f32,       // 0 = smooth, 1 = rough
    pub emissive: f32,        // Self-illumination
    pub _padding: f32,
}
```

## Common Material Examples

### Polished Gold
```rust
MaterialProperties {
    albedo: [1.0, 0.84, 0.0, 1.0],  // Gold color
    metallic: 1.0,
    roughness: 0.1,
    emissive: 0.0,
    ..Default::default()
}
```

### Rough Stone
```rust
MaterialProperties {
    albedo: [0.5, 0.5, 0.5, 1.0],  // Gray
    metallic: 0.0,
    roughness: 0.9,
    emissive: 0.0,
    ..Default::default()
}
```

### Glowing Screen
```rust
MaterialProperties {
    albedo: [0.2, 0.6, 1.0, 1.0],  // Blue
    metallic: 0.0,
    roughness: 0.5,
    emissive: 2.0,  // Strong glow
    ..Default::default()
}
```

### Plastic
```rust
MaterialProperties {
    albedo: [0.8, 0.2, 0.2, 1.0],  // Red
    metallic: 0.0,
    roughness: 0.4,
    emissive: 0.0,
    ..Default::default()
}
```

## PBR Lighting Model

Praxis uses the Cook-Torrance BRDF:

```
f = k_d * f_lambert + k_s * f_cook_torrance

where:
  k_d = diffuse contribution (1 - metallic)
  k_s = specular contribution (Fresnel)
  f_lambert = albedo / π
  f_cook_torrance = (D * F * G) / (4 * n·l * n·v)
```

### Distribution (D) - GGX
Models microfacet normal distribution based on roughness.

### Fresnel (F) - Schlick
Reflection increases at grazing angles.

### Geometry (G) - Smith
Accounts for microfacet self-shadowing.

## Texture Maps

Beyond base parameters, PBR often uses texture maps:

- **Albedo Map**: Per-pixel base color
- **Normal Map**: Surface detail without geometry
- **Metallic Map**: Per-pixel metallic values
- **Roughness Map**: Per-pixel roughness
- **Ambient Occlusion Map**: Pre-baked shadowing in crevices

## Material Batching

Praxis automatically sorts draw commands by material to minimize GPU state changes. Objects with identical materials share descriptor sets.

## See Also

- [Beginner's Guide: Material System](../beginners-guide.md#material-system) - Material data flow and batching
- [Rendering Learning Path](../learning-paths/rendering.md) - Learn PBR rendering step-by-step
- [Rendering Guide](../guides/rendering.md) - Using materials in practice
- [Lighting Concepts](lighting.md) - How lights interact with materials
