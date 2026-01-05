# Advanced Material System

The Praxis material system provides a comprehensive PBR (Physically Based Rendering) pipeline with advanced features for modern game and graphics applications.

## Features

### 🎨 Core PBR
- Albedo (base color) with texture support
- Metallic-roughness workflow
- Normal mapping
- Ambient occlusion
- Emissive materials
- Height maps for parallax

### 🔄 Material Instancing
- Share texture data across multiple instances
- Per-instance property overrides
- Efficient memory usage
- GPU-friendly batching

### 📚 Material Layers
- Blend up to 4 materials with masks
- Multiple blend modes (Replace, Add, Multiply, Overlay)
- Per-layer UV scaling
- Dynamic or pre-computed blending

### 🏔️ Parallax Occlusion Mapping
- Height-based surface displacement
- Self-occlusion support
- Adaptive sampling for performance
- View-angle dependent quality

### ✨ Extended PBR Features
- **Clearcoat**: Secondary specular layer (car paint, varnish)
- **Sheen**: Fabric-like grazing angle reflectance
- **Transmission**: Light transmission for glass/water
- **Anisotropy**: Directional roughness for brushed metals

## Quick Start

### Basic Material

```rust
use praxis_graphics::{Material, MaterialProperties};

// Create a material
let material = Material::new("my_material", albedo_texture);

// Set properties
material.set_properties(
    MaterialProperties::new()
        .with_metallic(0.8)
        .with_roughness(0.3)
);
```

### Material Instance

```rust
use praxis_graphics::{MaterialInstance, MaterialProperties};

// Create an instance with overrides
let instance = MaterialInstance::new(base_material)
    .override_properties(
        MaterialProperties::new()
            .with_metallic(0.5)
            .with_roughness(0.6)
    );
```

### Material Layers

```rust
use praxis_graphics::{MaterialLayer, BlendMode};

// Add a weathering layer
material.add_layer(
    MaterialLayer::new("rust", "rust_material")
        .with_mask(rust_mask_texture)
        .with_blend_mode(BlendMode::Multiply)
        .with_opacity(0.7)
);
```

### Extended PBR

```rust
use praxis_graphics::ExtendedPbrProperties;

// Add clearcoat
material.set_extended_properties(
    ExtendedPbrProperties::new()
        .with_clearcoat(1.0)
        .with_clearcoat_roughness(0.05)
);
```

## Architecture

### Material Structure

```
Material
├── ID (for instancing)
├── Base Material ID (if instance)
├── Textures
│   ├── Albedo (required)
│   ├── Normal (optional)
│   ├── Metallic-Roughness (optional)
│   ├── Height (optional)
│   ├── Ambient Occlusion (optional)
│   └── Emissive (optional)
├── Properties
│   ├── Base Color
│   ├── Metallic
│   ├── Roughness
│   └── Emissive Strength
├── Extended Properties
│   ├── Clearcoat
│   ├── Sheen
│   ├── Transmission
│   └── Anisotropy
├── Parallax Properties
│   ├── Height Scale
│   └── Sample Count
└── Layers (up to 3)
    ├── Material Reference
    ├── Blend Mask
    ├── Blend Mode
    └── Opacity
```

### Instancing System

```
MaterialManager
├── Base Materials (shared textures)
│   └── "metal_base"
│       ├── Albedo Texture (2048x2048)
│       └── Normal Map (2048x2048)
└── Instances (property overrides only)
    ├── "metal_shiny" → overrides: roughness=0.1
    ├── "metal_rough" → overrides: roughness=0.8
    └── "metal_gold"  → overrides: base_color=[1,0.8,0]

Memory Savings: 3 instances × 8MB textures = 8MB (not 24MB)
```

## Performance

### Memory Usage

**Traditional Approach:**
- 100 materials with unique properties = 100 × 8MB textures = 800MB

**With Instancing:**
- 100 instances of 10 base materials = 10 × 8MB textures = 80MB
- 90% memory reduction!

### GPU Performance

**Descriptor Sets:**
- Traditional: 1 per object (100 objects = 100 descriptor sets)
- Instanced: 1 per base material (100 objects = 10 descriptor sets)
- 90% reduction in descriptor set allocations

**Texture Binds:**
- Traditional: Change textures for every material
- Instanced: Change textures only when base material changes
- Significant reduction in GPU state changes

## See Also

- [Advanced Materials Documentation](../../docs/advanced-materials.md)
- [Examples](../../examples/material_demo.rs)
