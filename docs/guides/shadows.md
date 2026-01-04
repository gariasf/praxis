# Shadow Mapping

Cascaded shadow maps (CSM) with percentage closer filtering (PCF) for realistic, soft shadows from directional lights.

## Overview

Shadow mapping is a two-pass technique:

1. **Shadow Pass**: Render scene from light's perspective to depth texture
2. **Main Pass**: Sample shadow map to determine if fragments are in shadow

## Cascaded Shadow Maps

CSM divides the view frustum into multiple cascades at different distances:

```
Camera                                           Far Plane
  │                                                   │
  ├──────────┬────────────────┬───────────────────────┤
  │Cascade 0 │   Cascade 1    │      Cascade 2        │
  │ (0-20m)  │   (20-100m)    │     (100-500m)        │
  │ High Res │  Medium Res    │      Lower Res        │
  └──────────┴────────────────┴───────────────────────┘
```

This prevents shadow aliasing near the camera while maintaining performance.

## Configuration

```rust
use praxis_graphics::shadow::ShadowConfig;

// High quality for close-up scenes
let high_quality = ShadowConfig {
    shadow_map_size: 2048,
    cascade_count: 4,
    cascade_distances: [10.0, 30.0, 100.0, 300.0],
    pcf_samples: 9,  // 3x3 filter
    bias: 0.005,
};

// Performance-focused for open worlds
let performance = ShadowConfig {
    shadow_map_size: 1024,
    cascade_count: 2,
    cascade_distances: [30.0, 150.0, 500.0, 1000.0],
    pcf_samples: 4,  // 2x2 filter
    bias: 0.01,
};

// Default (balanced)
let default = ShadowConfig::default();
```

### Parameters

| Parameter | Description | Typical Values |
|-----------|-------------|----------------|
| `shadow_map_size` | Resolution per cascade | 512, 1024, 2048, 4096 |
| `cascade_count` | Number of cascades | 2-4 |
| `cascade_distances` | Split distances (meters) | Scene-dependent |
| `pcf_samples` | Soft shadow quality | 1, 4, 9, 16 |
| `bias` | Shadow acne prevention | 0.001-0.01 |

## PCF Filtering

Percentage Closer Filtering softens shadow edges:

| Samples | Pattern | Quality | Cost |
|---------|---------|---------|------|
| 1 | Single point | Hard shadows | Fastest |
| 4 | 2x2 grid | Soft shadows | Low |
| 9 | 3x3 grid | Softer shadows | Medium |
| 16 | 4x4 grid | Softest shadows | Higher |

## Usage

```rust
use praxis_graphics::shadow::{ShadowMapManager, ShadowConfig};
use praxis_math::Vec3;

// Create shadow manager
let shadow_manager = ShadowMapManager::new(
    memory_allocator.clone(),
    ShadowConfig::default(),
)?;

// Update each frame with light direction
let light_direction = Vec3::new(0.3, -0.8, 0.5).normalize();
shadow_manager.update(
    light_direction,
    camera_view_matrix,
    camera_projection_matrix,
)?;

// Shadow pass: render to shadow maps
for cascade_idx in 0..shadow_manager.cascade_count() {
    let framebuffer = shadow_manager.shadow_framebuffers()[cascade_idx];
    // Begin render pass, render geometry, end render pass
}

// Main pass: shadows applied automatically via descriptor bindings
```

### Descriptor Set Layout

Shadow data is bound at:
- **Binding 4**: Shadow uniform buffer (matrices, config)
- **Bindings 5-8**: Shadow map cascades 0-3

## Performance

### Memory Usage

Formula: `cascade_count × shadow_map_size² × 4 bytes`

| Configuration | Memory |
|---------------|--------|
| 2 × 512² | 2 MB |
| 3 × 1024² | 12 MB |
| 4 × 2048² | 64 MB |

### Recommended Settings

| Target | Resolution | Cascades | PCF | Cost |
|--------|------------|----------|-----|------|
| Mobile | 512 | 2 | 1 | ~0.5-1ms |
| Low Desktop | 1024 | 2 | 4 | ~1-2ms |
| Mid Desktop | 1024 | 3 | 4 | ~2-3ms |
| High Desktop | 2048 | 3 | 9 | ~4-6ms |

## Troubleshooting

### Shadow Acne

**Symptoms:** Striped/moiré patterns, self-shadowing artifacts

**Solutions:**
1. Increase `bias` in config
2. Increase shadow map resolution
3. Slope-scale bias is already enabled

### Peter Panning

**Symptoms:** Shadows detached from objects, floating shadows

**Solutions:**
1. Decrease `bias` (opposite of acne fix)
2. Find balance between acne and peter panning

### Blocky Shadows

**Symptoms:** Jagged, pixelated edges

**Solutions:**
1. Increase shadow map resolution
2. Add more cascades
3. Increase PCF sample count

### Poor Performance

**Solutions:**
1. Reduce shadow map resolution
2. Decrease cascade count
3. Reduce PCF samples
4. Implement shadow caster culling

## Example

```bash
cargo run --example shadow_demo
```

## See Also

- [Rendering Guide](rendering.md) - Complete rendering pipeline
- [Concepts: Vulkan Rendering](../concepts/vulkan-rendering.md) - Pipeline fundamentals
