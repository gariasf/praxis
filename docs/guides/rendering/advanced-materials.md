# Advanced Material System

The Praxis advanced material system provides comprehensive support for PBR rendering with efficient material instancing, multi-material blending, and advanced visual effects.

## Overview

The advanced material system includes:

- **Material Instancing**: Share texture data across instances with per-object property overrides
- **Material Layers**: Blend multiple materials with mask textures and various blend modes
- **Parallax Occlusion Mapping**: Enhanced depth perception through height map displacement
- **Extended PBR**: Clearcoat, sheen, transmission, and anisotropy support

## Material Instancing

Material instancing allows efficient per-object parameter overrides without duplicating texture data.

### Benefits

- **Memory Efficiency**: 100 objects sharing 1 base material = 1 set of textures loaded
- **GPU Performance**: Reduced descriptor set allocations
- **Artist Workflow**: Create variations quickly by overriding properties

### Usage

```rust
use praxis_graphics::{Material, MaterialInstance, MaterialProperties, Texture};

// Create a base material
let base_material = Material::new("metal_base", albedo_texture);
material_manager.add_material(base_material);

// Create instances with property overrides
let instance1 = MaterialInstance::new(material_manager.get_material("metal_base").unwrap())
    .override_properties(
        MaterialProperties::new()
            .with_metallic(0.9)
            .with_roughness(0.1)
    );

let instance2 = MaterialInstance::new(material_manager.get_material("metal_base").unwrap())
    .override_properties(
        MaterialProperties::new()
            .with_metallic(0.5)
            .with_roughness(0.6)
    );
```

### Performance

- **Texture Sharing**: All instances reference the same GPU textures
- **Property Overrides**: Small per-instance uniform buffers (32-64 bytes)
- **Batching**: Instances can be batched together for efficient rendering

## Material Layers

Material layers enable blending multiple materials with mask textures for complex surfaces.

### Use Cases

- **Terrain**: Blend rock, grass, sand, and dirt based on height/slope
- **Weathering**: Add rust, scratches, or dirt to base materials
- **Detail Mapping**: Overlay fine detail on base materials
- **Decals**: Apply stickers, logos, or damage to surfaces

### Blend Modes

- **Replace**: Mask controls opacity (default for terrain blending)
- **Add**: Additive blend (good for emissive overlays)
- **Multiply**: Multiplicative blend (darkening effects)
- **Overlay**: Screen/multiply based on luminance (contrast enhancement)

### Usage

```rust
use praxis_graphics::{Material, MaterialLayer, BlendMode};

let mut material = Material::new("terrain", base_texture);

// Add a rock layer
material.add_layer(
    MaterialLayer::new("rock", "rock_material")
        .with_mask(rock_mask_texture)
        .with_blend_mode(BlendMode::Replace)
        .with_opacity(1.0)
        .with_uv_scale([2.0, 2.0])
);

// Add a dirt overlay
material.add_layer(
    MaterialLayer::new("dirt", "dirt_material")
        .with_mask(dirt_mask_texture)
        .with_blend_mode(BlendMode::Multiply)
        .with_opacity(0.7)
);
```

### Performance Considerations

- **Layer Count**: Up to 4 layers supported (base + 3 overlays)
- **Mask Resolution**: Lower resolution masks (512x512) often sufficient
- **UV Scaling**: Different layers can use different UV scales
- **Caching**: Blended results can be pre-computed and cached

## Parallax Occlusion Mapping

Parallax occlusion mapping (POM) adds depth perception to surfaces using height maps.

### Benefits

- **Enhanced Depth**: Flat surfaces appear to have depth without additional geometry
- **Self-Occlusion**: Steep angles show proper occlusion
- **Detail**: Brick walls, cobblestones, and rough surfaces look more realistic

### Configuration

```rust
use praxis_graphics::ParallaxProperties;

let parallax = ParallaxProperties::new()
    .enabled(true)
    .with_height_scale(0.05)      // Depth of effect
    .with_min_samples(8)           // Samples at steep angles
    .with_max_samples(32);         // Samples at shallow angles

material.set_parallax_properties(parallax);
material.set_height_texture(Some(height_map));
```

### Parameters

- **Height Scale** (0.0 - 0.1): Controls depth magnitude
  - 0.02: Subtle depth (wood grain)
  - 0.05: Moderate depth (bricks)
  - 0.08: Strong depth (cobblestones)

- **Sample Count** (8 - 64): Quality vs performance trade-off
  - Min samples: Used at steep viewing angles (fast)
  - Max samples: Used at shallow angles (quality)

### Best Practices

- Use power-of-two texture resolutions
- Height maps should be grayscale (R channel used)
- Black = lowest, White = highest
- Combine with normal maps for best results

## Extended PBR Features

### Clearcoat

A second specular layer on top of the base material, useful for:
- Car paint
- Lacquered wood
- Coated metals
- Wet surfaces

```rust
use praxis_graphics::ExtendedPbrProperties;

let extended = ExtendedPbrProperties::new()
    .with_clearcoat(1.0)              // Full clearcoat
    .with_clearcoat_roughness(0.03);  // Glossy coating

material.set_extended_properties(extended);
```

**Parameters:**
- **Clearcoat** (0.0 - 1.0): Strength of the coating layer
- **Clearcoat Roughness** (0.0 - 1.0): Roughness of the coating

### Sheen

Fabric-like reflectance at grazing angles, useful for:
- Cloth and velvet
- Carpet
- Brushed materials

```rust
let extended = ExtendedPbrProperties::new()
    .with_sheen(0.8)         // Strong sheen effect
    .with_sheen_tint(0.5);   // Partially tinted

material.set_extended_properties(extended);
```

**Parameters:**
- **Sheen** (0.0 - 1.0): Strength of the sheen effect
- **Sheen Tint** (0.0 - 1.0): 0 = white sheen, 1 = colored by albedo

### Transmission

Light transmission through materials, useful for:
- Glass
- Water
- Transparent plastics
- Gemstones

```rust
let extended = ExtendedPbrProperties::new()
    .with_transmission(0.9)   // Highly transparent
    .with_ior(1.5);           // Glass IOR

material.set_extended_properties(extended);
```

**Parameters:**
- **Transmission** (0.0 - 1.0): Amount of light transmitted
- **IOR** (1.0 - 3.0): Index of refraction
  - 1.0: Air
  - 1.33: Water
  - 1.5: Glass
  - 2.4: Diamond

### Anisotropy

Directional roughness for brushed or fibrous materials:
- Brushed metal
- Hair
- Scratched surfaces

```rust
let extended = ExtendedPbrProperties::new()
    .with_anisotropy(0.7)            // Strong directional effect
    .with_anisotropy_rotation(0.25); // Rotate direction

material.set_extended_properties(extended);
```

**Parameters:**
- **Anisotropy** (-1.0 - 1.0): Directional roughness strength
- **Anisotropy Rotation** (0.0 - 1.0): Direction in UV space

## Complete Example

```rust
use praxis_graphics::{
    Material, MaterialProperties, ExtendedPbrProperties, 
    ParallaxProperties, MaterialLayer, BlendMode
};

// Create base material
let mut material = Material::new("advanced_surface", albedo_texture);

// Set base PBR properties
material.set_properties(
    MaterialProperties::new()
        .with_base_color([0.8, 0.8, 0.8, 1.0])
        .with_metallic(0.0)
        .with_roughness(0.5)
);

// Add extended PBR features
material.set_extended_properties(
    ExtendedPbrProperties::new()
        .with_clearcoat(0.5)
        .with_clearcoat_roughness(0.1)
);

// Enable parallax occlusion mapping
material.set_parallax_properties(
    ParallaxProperties::new()
        .enabled(true)
        .with_height_scale(0.05)
);

// Set textures
material.set_normal_texture(Some(normal_map));
material.set_height_texture(Some(height_map));
material.set_metallic_roughness_texture(Some(mr_map));

// Add a weathering layer
material.add_layer(
    MaterialLayer::new("rust", "rust_material")
        .with_mask(rust_mask)
        .with_blend_mode(BlendMode::Multiply)
        .with_opacity(0.6)
);
```

## Performance Tips

### Material Instancing
- Create instances for objects that share textures but differ in properties
- Group instances by base material for better batching
- Use the `MaterialInstanceManager` to track and manage instances

### Material Layers
- Limit to 3-4 layers per material
- Use lower resolution masks where possible
- Pre-compute and cache blended results when layers don't change
- Consider runtime blending vs pre-baked for static vs dynamic content

### Parallax Occlusion Mapping
- Disable for distant objects (use LOD system)
- Reduce sample counts on lower-end hardware
- Use with mipmapped height maps
- Consider screen-space derivatives for better quality

### Extended PBR
- Only enable features you need (zeroed parameters have minimal cost)
- Clearcoat and sheen are relatively cheap
- Transmission is more expensive (requires additional passes)
- Test on target hardware to profile performance

## Shader Integration

The advanced material system uses specialized shaders:

- **advanced_material.frag**: Full PBR with parallax and extended features
- **material_layer_blend.frag**: Multi-material blending

These shaders support:
- Cook-Torrance BRDF
- GGX normal distribution
- Fresnel-Schlick approximation
- Height-based parallax with occlusion
- Multiple blend modes
- Dynamic layer composition

## See Also

- [Material System Architecture](material_system.md)
- [PBR Theory](pbr_theory.md)
- [Texture Management](texture_system.md)
- [Shader Guide](shader_guide.md)
