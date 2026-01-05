# Advanced Lighting Features

This document describes the advanced lighting features available in the Praxis engine, including light probes, volumetric fog, god rays, area lights, and light linking.

## Overview

The advanced lighting system provides five major features:

1. **Light Probes** - Dynamic global illumination using spherical harmonics
2. **Volumetric Fog** - Raymarched fog with density functions and light scattering
3. **God Rays (Crepuscular Rays)** - Radial blur from light sources
4. **Area Lights** - Polygon lights using Linearly Transformed Cosines (LTC)
5. **Light Linking** - Selective control over which lights affect which objects

## Light Probes

Light probes capture lighting information at specific points in space and use spherical harmonics to represent diffuse irradiance. This provides efficient real-time global illumination.

### Key Features

- **Spherical Harmonics**: Compact representation using 9 coefficients (L2)
- **Probe Grids**: 3D grids for spatial interpolation
- **Trilinear Interpolation**: Smooth blending between nearby probes
- **Dynamic Updates**: Real-time probe data updates

### Usage

```rust
use praxis_graphics::{LightProbeManager, LightProbeGrid};
use praxis_math::Vec3;

// Create a probe grid covering a room
let grid = LightProbeGrid::new(
    Vec3::new(-10.0, 0.0, -10.0),  // Min bounds
    Vec3::new(10.0, 5.0, 10.0),     // Max bounds
    [5, 3, 5],                      // Grid dimensions (5×3×5 = 75 probes)
);

let mut manager = LightProbeManager::new(device, allocator)?;
manager.add_grid(grid);

// Query lighting at a position
let probe = manager.query_at_position(Vec3::new(0.0, 2.0, 0.0));
```

### Technical Details

- **SH Coefficients**: 9 coefficients × 3 channels (RGB) = 27 values per probe
- **Memory**: ~160 bytes per probe (std140 layout)
- **Interpolation**: Trilinear blending between 8 nearest probes
- **GPU Integration**: Uniform buffer binding for shader access

## Volumetric Fog

Volumetric fog simulates atmospheric scattering through raymarching, creating realistic fog effects with light interaction.

### Key Features

- **Raymarching**: Configurable step count for quality/performance balance
- **Density Functions**: Multiple distribution patterns
  - Uniform: Constant density
  - Exponential: Distance-based falloff
  - Height-based: Ground fog with vertical falloff
  - Noise: Procedural variation
- **Light Scattering**: In-scattering from directional lights
- **Phase Function**: Anisotropic scattering (Henyey-Greenstein)
- **Shadow Integration**: Fog receives shadows

### Usage

```rust
use praxis_graphics::{VolumetricFog, VolumetricFogConfig, FogDensityFunction};
use praxis_math::Vec3;

let config = VolumetricFogConfig {
    density_function: FogDensityFunction::HeightBased {
        base_height: 0.0,
        falloff: 0.15,
    },
    color: Vec3::new(0.7, 0.75, 0.8),
    density: 0.05,
    max_distance: 100.0,
    num_steps: 64,
    light_scattering: 0.3,
    anisotropy: 0.0,        // -1.0 (back-scatter) to 1.0 (forward-scatter)
    shadow_influence: 0.8,
};

let fog = VolumetricFog::new(config);
```

### Performance Considerations

- **Step Count**: Higher = better quality, lower = better performance
  - 32 steps: Fast, suitable for distant fog
  - 64 steps: Balanced quality/performance
  - 128 steps: High quality, may impact performance
- **Max Distance**: Limits raymarching range
- **Early Exit**: Raymarch stops when transmittance < 0.01

### Shader Integration

The volumetric fog shader performs:
1. Depth reconstruction to find ray end point
2. Raymarching along view ray
3. Density sampling at each step
4. Light scattering calculation
5. Transmittance accumulation
6. Final color compositing

## God Rays (Crepuscular Rays)

God rays simulate light shafts through atmospheric particles using radial blur from light source positions.

### Key Features

- **Radial Blur**: Directional blur from light to screen edges
- **Occlusion Pass**: Extracts bright areas
- **Configurable Samples**: Quality control
- **Additive Blending**: Natural light shaft appearance
- **Decay Factor**: Light intensity falloff along rays

### Usage

```rust
use praxis_graphics::{GodRays, GodRaysConfig};

let config = GodRaysConfig {
    num_samples: 64,      // Number of radial samples
    density: 0.5,         // Sample spacing
    weight: 0.3,          // Light contribution per sample
    decay: 0.95,          // Intensity decay per sample
    exposure: 0.8,        // Overall brightness
    threshold: 0.8,       // Brightness threshold for occlusion
};

let god_rays = GodRays::new(config);
```

### Algorithm

1. **Occlusion Pass**: Render scene with bright areas isolated
2. **Radial Blur**: Sample along rays from light source
3. **Accumulation**: Sum weighted samples with decay
4. **Composite**: Add god rays to scene additively

### Parameters Explained

- **num_samples**: More samples = smoother rays, higher cost
- **density**: Controls how far each sample steps along the ray
- **weight**: Base contribution of each sample
- **decay**: Multiplicative falloff (0.95 = 5% reduction per sample)
- **exposure**: Final brightness multiplier
- **threshold**: Only pixels brighter than this contribute

## Area Lights

Area lights provide realistic lighting from polygon light sources using the LTC (Linearly Transformed Cosines) technique.

### Key Features

- **Multiple Shapes**:
  - Rectangle: Most common, efficient
  - Disk: Circular lights
  - Sphere: Omnidirectional area light
  - Tube: Linear lights (experimental)
- **LTC Integration**: Accurate specular reflections
- **Soft Shadows**: Natural penumbra (requires shadow system)
- **Two-Sided**: Optional back-face illumination

### Usage

```rust
use praxis_graphics::{AreaLight, AreaLightType, AreaLightManager};
use praxis_math::Vec3;

// Rectangle light (ceiling panel)
let light = AreaLight {
    light_type: AreaLightType::Rectangle { width: 4.0, height: 2.0 },
    position: Vec3::new(0.0, 5.0, 0.0),
    direction: Vec3::new(0.0, -1.0, 0.0),
    color: Vec3::new(1.0, 0.95, 0.85),
    intensity: 15.0,
    two_sided: false,
};

let mut manager = AreaLightManager::new(device, allocator)?;
manager.add_light(light)?;

// Disk light (lamp)
let disk_light = AreaLight::new_disk(Vec3::new(3.0, 4.0, 0.0), 1.5)
    .with_color(Vec3::new(1.0, 0.8, 0.6))
    .with_intensity(10.0);
manager.add_light(disk_light)?;
```

### Technical Details

- **LTC Matrices**: Pre-computed lookup tables for BRDF approximation
- **Polygon Clipping**: Accurate integration over light area
- **Material Interaction**: Works with PBR roughness/metallic
- **Maximum Lights**: 16 area lights per scene (configurable)

### Performance

- Rectangle lights: ~50-100 instructions per fragment
- Sphere lights: ~30-60 instructions per fragment
- LTC lookup: 2 texture samples per light
- Recommendation: Use for key lights, not fill lights

## Light Linking

Light linking provides fine-grained control over which lights affect which objects using bit masks and channels.

### Key Features

- **Channel System**: 32 channels (bits) for flexible grouping
- **Object Masks**: Each object has a 32-bit receive mask
- **Light Channels**: Each light broadcasts on specific channels
- **Zero Overhead**: GPU bitwise operations
- **Dynamic Updates**: Real-time channel modifications

### Usage

```rust
use praxis_graphics::{LightLinkingManager, LightChannel};

let mut manager = LightLinkingManager::new();

// Define channels
manager.register_channel(0, "hero".to_string());
manager.register_channel(1, "environment".to_string());
manager.register_channel(2, "accent".to_string());

// Configure objects (bit masks)
let hero_lights = 0b0001;          // Channel 0
let environment_lights = 0b0010;    // Channel 1
let accent_lights = 0b0100;         // Channel 2

manager.set_object_mask("hero_character", hero_lights | environment_lights)?;
manager.set_object_mask("background_prop", environment_lights)?;
manager.set_object_mask("special_object", accent_lights | environment_lights)?;

// Configure lights
manager.set_light_channel("key_light", 0)?;        // Hero channel
manager.set_light_channel("ambient_light", 1)?;     // Environment channel
manager.set_light_channel("rim_light", 2)?;         // Accent channel

// Query
let can_affect = manager.can_light_affect_object("key_light", "hero_character");
// Returns: true (hero_character receives channel 0, key_light broadcasts on channel 0)
```

### Shader Integration

In the fragment shader:

```glsl
// Per-object uniform
layout(set = 2, binding = 0) uniform ObjectLinking {
    uint light_mask;  // Which channels this object receives
} object_linking;

// Per-light data
struct Light {
    // ... other fields
    uint channel;  // Which channel this light broadcasts on
};

// Lighting loop
for (int i = 0; i < light_count; i++) {
    Light light = lights[i];
    
    // Check if light affects this object
    if ((object_linking.light_mask & (1u << light.channel)) == 0u) {
        continue;  // Skip this light
    }
    
    // Calculate lighting...
}
```

### Use Cases

1. **Hero Lighting**: Character-specific key lights
2. **Set Extension**: Different lighting for foreground vs. background
3. **VFX Isolation**: Effects lights don't affect environment
4. **Performance**: Disable expensive lights for distant objects
5. **Artistic Control**: Fine-tune lighting per shot/scene

## Integration Example

Combining multiple features:

```rust
use praxis_graphics::*;
use praxis_math::Vec3;

// Setup light probes for GI
let probe_grid = LightProbeGrid::new(
    Vec3::new(-20.0, 0.0, -20.0),
    Vec3::new(20.0, 10.0, 20.0),
    [8, 4, 8],
);
let mut probe_manager = LightProbeManager::new(device, allocator)?;
probe_manager.add_grid(probe_grid);

// Setup volumetric fog
let fog = VolumetricFog::new(VolumetricFogConfig {
    density_function: FogDensityFunction::HeightBased {
        base_height: 0.0,
        falloff: 0.1,
    },
    color: Vec3::new(0.7, 0.75, 0.8),
    density: 0.03,
    max_distance: 100.0,
    num_steps: 64,
    light_scattering: 0.4,
    anisotropy: 0.3,
    shadow_influence: 0.8,
});

// Setup god rays from sun
let god_rays = GodRays::new(GodRaysConfig {
    num_samples: 80,
    density: 0.6,
    weight: 0.4,
    decay: 0.96,
    exposure: 0.9,
    threshold: 0.85,
});

// Setup area lights
let mut area_manager = AreaLightManager::new(device, allocator)?;
area_manager.add_light(
    AreaLight::new_rectangle(Vec3::new(0.0, 8.0, 0.0), 4.0, 4.0)
        .with_color(Vec3::new(1.0, 0.95, 0.85))
        .with_intensity(20.0)
)?;

// Setup light linking
let mut linking = LightLinkingManager::new();
linking.set_object_mask("hero", 0b0011)?;  // Channels 0 and 1
linking.set_light_channel("key_light", 0)?;
```

## Performance Guidelines

### Light Probes
- **Good**: 50-200 probes for a typical scene
- **Maximum**: 64 probes active simultaneously (shader limit)
- **Update**: Static probes = zero runtime cost
- **Dynamic**: Update 1-2 probes per frame for moving lights

### Volumetric Fog
- **Good**: 32-64 steps for full-screen fog
- **Optimize**: Reduce steps for distant fog
- **Resolution**: Half-resolution fog with upsampling for better performance

### God Rays
- **Good**: 64-80 samples for quality rays
- **Optimize**: Quarter-resolution for the effect
- **Update**: Per-frame light position update only

### Area Lights
- **Good**: 4-8 area lights per scene
- **Maximum**: 16 area lights (shader limit)
- **Optimize**: Use for key lights only, not ambient fill

### Light Linking
- **Cost**: Minimal (single bitwise AND per light per object)
- **Good**: Use liberally for artistic control
- **Best Practice**: Plan channel allocation (32 channels available)

## Shader Architecture

All advanced lighting features integrate into the main lighting pipeline:

1. **Base Pass**: Standard lighting (directional, point, spot)
2. **Light Probes**: Add indirect diffuse from probes
3. **Area Lights**: Add area light contributions
4. **Volumetric Fog**: Post-process fog with scattering
5. **God Rays**: Post-process radial blur
6. **Light Linking**: Filter all lights by channel masks

## References

### Light Probes
- [Spherical Harmonics for Lighting](https://www.ppsloan.org/publications/StupidSH36.pdf)
- [GPU Gems: Ambient Occlusion](https://developer.nvidia.com/gpugems/gpugems/part-iii-materials/chapter-13-ambient-occlusion)

### Volumetric Fog
- [Volumetric Fog: Unified compute shader based solution](https://www.guerrilla-games.com/read/the-real-time-volumetric-cloudscapes-of-horizon-zero-dawn)
- [GPU Pro 5: Screen-Space Volumetric Fog](http://advances.realtimerendering.com/s2014/index.html)

### God Rays
- [GPU Gems 3: Volumetric Light Scattering](https://developer.nvidia.com/gpugems/gpugems3/part-ii-light-and-shadows/chapter-13-volumetric-light-scattering-post-process)

### Area Lights (LTC)
- [Real-Time Polygonal-Light Shading with Linearly Transformed Cosines](https://eheitzresearch.wordpress.com/415-2/)
- [LTC Paper](https://labs.unity.com/article/real-time-polygonal-light-shading-linearly-transformed-cosines)

### Light Linking
- Industry standard feature in Maya, Houdini, 3ds Max
