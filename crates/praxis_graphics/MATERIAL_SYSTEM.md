# Material System

Comprehensive PBR material system with advanced rendering features.

## Features

**Core PBR**
- Albedo (base color) with texture support
- Metallic-roughness workflow
- Normal mapping
- Ambient occlusion
- Emissive materials
- Height maps for parallax

**Material Instancing**
- Share textures across instances
- Per-instance property overrides
- Efficient memory usage
- GPU-friendly batching
- See [Material Instancing](MATERIAL_INSTANCING.md)

**Advanced PBR**
- Clearcoat: Secondary specular layer
- Sheen: Fabric-like reflectance
- Transmission: Light transmission
- Anisotropy: Directional roughness

**Material Layers**
- Blend up to 4 materials
- Multiple blend modes
- Per-layer UV scaling
- Dynamic or pre-computed blending

## Quick Start

### Basic Material

```rust
use praxis_graphics::{Material, MaterialProperties};

let material = Material::new("my_material", albedo_texture);

material.set_properties(
    MaterialProperties::new()
        .with_metallic(0.8)
        .with_roughness(0.3)
        .with_base_color([1.0, 1.0, 1.0, 1.0])
);
```

### Material Instance

```rust
use praxis_graphics::MaterialInstance;

// Create instance with property overrides
let instance = MaterialInstance::new(base_material)
    .override_properties(
        MaterialProperties::new()
            .with_metallic(0.5)
            .with_roughness(0.6)
    );
```

### Extended PBR

```rust
use praxis_graphics::ExtendedPbrProperties;

material.set_extended_properties(
    ExtendedPbrProperties::new()
        .with_clearcoat(1.0)
        .with_clearcoat_roughness(0.05)
);
```

## Material Properties

### Base Properties

```rust
pub struct MaterialProperties {
    pub base_color: [f32; 4],    // RGBA tint
    pub metallic: f32,            // 0.0 = dielectric, 1.0 = metal
    pub roughness: f32,           // 0.0 = smooth, 1.0 = rough
    pub emissive_strength: f32,   // Self-illumination
}
```

**Metallic-Roughness Workflow**
- Metallic 0.0: Dielectric (plastic, wood, stone)
- Metallic 1.0: Conductor (gold, iron, copper)
- Roughness 0.0: Mirror-smooth specular
- Roughness 1.0: Completely diffuse

### Extended Properties

```rust
pub struct ExtendedPbrProperties {
    pub clearcoat: f32,              // 0.0-1.0 coating strength
    pub clearcoat_roughness: f32,    // Coating roughness
    pub sheen: f32,                  // Fabric sheen
    pub sheen_tint: [f32; 3],        // Sheen color
    pub transmission: f32,            // Light transmission
    pub transmission_roughness: f32,  // Transmission blur
    pub anisotropy: f32,             // -1.0 to 1.0
    pub anisotropy_rotation: f32,    // Tangent rotation
}
```

**Use Cases:**
- **Clearcoat**: Car paint, varnished wood
- **Sheen**: Velvet, satin fabrics
- **Transmission**: Glass, water, translucent materials
- **Anisotropy**: Brushed metal, hair, fur

### Parallax Properties

```rust
pub struct ParallaxProperties {
    pub height_scale: f32,      // Displacement strength
    pub min_samples: u32,       // Min raymarching samples
    pub max_samples: u32,       // Max raymarching samples
}
```

## Textures

### Standard Texture Slots

| Slot | Usage | Format |
|------|-------|--------|
| Albedo | Base color | RGBA8 or SRGB8 |
| Normal | Tangent-space normals | RGB8 |
| Metallic-Roughness | PBR properties | RG8 (metallic=R, roughness=G) |
| Height | Parallax displacement | R8 or R16 |
| AO | Ambient occlusion | R8 |
| Emissive | Self-illumination | RGB8 or RGB16F |

### Texture Loading

```rust
use praxis_graphics::TextureManager;

let texture_manager = TextureManager::new(device, allocator);

// Load texture
texture_manager.load_texture("brick_albedo", "textures/brick_color.png")?;

// Use in material
let material = Material::new("brick", "brick_albedo");
material.set_normal_map("brick_normal");
material.set_metallic_roughness_map("brick_mr");
```

## Material Instancing

Create many material variants without duplicating textures:

```rust
// Base material (shared textures)
let base = Arc::new(Material::new("metal_base", metal_texture));
material_manager.add_material(base.clone());

// Color variants (property overrides only)
for i in 0..100 {
    let color = generate_color(i);
    render_context.create_material_instance(
        format!("metal_{}", i),
        "metal_base"
    )?.override_properties(
        MaterialProperties::new().with_base_color(color)
    );
}
```

**Memory savings**: 100 instances = 1× texture memory (not 100×)

See [Material Instancing](MATERIAL_INSTANCING.md) for details.

## Material Layers

Blend multiple materials with masks:

```rust
use praxis_graphics::{MaterialLayer, BlendMode};

// Add rust weathering layer
material.add_layer(
    MaterialLayer::new("rust", "rust_material")
        .with_mask(rust_mask_texture)
        .with_blend_mode(BlendMode::Multiply)
        .with_opacity(0.7)
);
```

**Blend Modes:**
- `Replace`: Replace base material
- `Add`: Additive blending
- `Multiply`: Multiplicative darkening
- `Overlay`: Preserve highlights and shadows

**Layer Order:**
1. Base material
2. Layer 0 (blended on top)
3. Layer 1 (blended on Layer 0)
4. Layer 2 (blended on Layer 1)

## Rendering Pipeline Integration

### Forward Rendering

```rust
// Set material properties descriptor set
command_buffer.bind_descriptor_sets(
    PipelineBindPoint::Graphics,
    pipeline.layout().clone(),
    1, // Set 1
    material_descriptor_set,
    [],
);
```

### Deferred Rendering

```rust
// G-buffer pass writes material properties
layout(location = 0) out vec4 gbuffer_albedo;
layout(location = 1) out vec4 gbuffer_normal;
layout(location = 2) out vec4 gbuffer_metallic_roughness;

void main() {
    gbuffer_albedo = texture(albedo_texture, uv);
    gbuffer_normal = pack_normal(normal);
    gbuffer_metallic_roughness = vec4(metallic, roughness, 0, 0);
}
```

### Bindless Rendering

```rust
// Register material with bindless system
let bindless = render_context.bindless_manager_mut()?;
let material_data = BindlessMaterialData {
    base_color: material.base_color,
    albedo_texture_index: bindless.get_texture_index("albedo")?,
    normal_texture_index: bindless.get_texture_index("normal")?,
    metallic: material.metallic,
    roughness: material.roughness,
    emissive_strength: material.emissive_strength,
    _padding: [0.0; 3],
};
let material_idx = bindless.register_material(material_data)?;
```

See [Bindless Rendering](BINDLESS_RENDERING.md) for details.

## Shader Integration

### Standard Material Shader

```glsl
// Set 1: Material properties
layout(set = 1, binding = 0) uniform MaterialProperties {
    vec4 base_color;
    float metallic;
    float roughness;
    float emissive_strength;
} material;

void main() {
    vec4 albedo = texture(albedo_texture, uv) * material.base_color;
    float metallic = texture(metallic_roughness, uv).r * material.metallic;
    float roughness = texture(metallic_roughness, uv).g * material.roughness;
    
    // PBR lighting calculation...
}
```

### Clearcoat Shader

```glsl
// Extended PBR properties
layout(set = 1, binding = 1) uniform ExtendedPbr {
    float clearcoat;
    float clearcoat_roughness;
    // ...
} extended;

// Dual-lobe BRDF
vec3 base_specular = cook_torrance(normal, roughness, metallic);
vec3 coat_specular = cook_torrance(normal, extended.clearcoat_roughness, 0.04);
vec3 final_specular = mix(base_specular, coat_specular, extended.clearcoat);
```

## Performance Considerations

### Memory Usage

**Traditional (100 materials):**
- 100 materials × 8MB textures = 800MB

**With Instancing (100 instances of 10 bases):**
- 10 base materials × 8MB textures = 80MB
- 90% memory reduction

### Descriptor Sets

**Traditional:**
- 1 descriptor set per material per frame
- 100 materials = 100 descriptor set allocations

**Instanced + Pooled:**
- Descriptor sets cached and reused
- 100 instances = ~10 unique descriptor sets
- 90% reduction in allocations

### GPU State Changes

**Material Sorting:**
```rust
// Sort draws by material to minimize state changes
draw_commands.sort_by_key(|cmd| {
    (cmd.material_id, cmd.texture_id)
});
```

**Batching:**
- Group objects with same material
- Minimize descriptor set binds
- Use dynamic offsets for per-object data

## Best Practices

### Texture Resolution

- Albedo: 1024×1024 to 2048×2048
- Normal: 1024×1024 (high detail) or 512×512
- Metallic-Roughness: 512×512 to 1024×1024
- Height: 512×512 to 1024×1024
- AO: 512×512 (can be lower)

### Material Authoring

**Good PBR values:**
- Metallic: 0.0 (dielectric) or 1.0 (metal), avoid in-between
- Roughness: 0.1 minimum (nothing is perfectly smooth)
- Base color: Use measured albedo values (avoid pure black/white)

**Texture packing:**
- Pack metallic + roughness into single texture (RG channels)
- Pack AO into unused channel if needed
- Use BC compression (BC7 for color, BC5 for normals)

### Performance Optimization

1. **Use instancing** for material variants
2. **Enable descriptor set pooling** for automatic reuse
3. **Sort by material** to reduce state changes
4. **Share textures** across materials when possible
5. **Use mipmaps** for all textures

## Debugging

### Visualize Material Properties

```glsl
// Debug mode constants
const uint DEBUG_ALBEDO = 0;
const uint DEBUG_NORMAL = 1;
const uint DEBUG_METALLIC = 2;
const uint DEBUG_ROUGHNESS = 3;

if (debug_mode == DEBUG_ALBEDO) {
    f_color = vec4(albedo, 1.0);
} else if (debug_mode == DEBUG_METALLIC) {
    f_color = vec4(vec3(metallic), 1.0);
}
```

### Monitor Material Statistics

```rust
let stats = render_context.material_instance_stats();
println!("Total instances: {}", stats.total_instances);
println!("Unique base materials: {}", stats.unique_base_materials);
println!("Avg instances per base: {:.2}", stats.avg_instances_per_base);
```

## See Also

- [Material Instancing](MATERIAL_INSTANCING.md) - Detailed instancing guide
- [Bindless Rendering](BINDLESS_RENDERING.md) - Bindless material system
- [Descriptor Sets Reference](DESCRIPTOR_SETS_REFERENCE.md) - Shader layouts
- Example: `examples/material_demo.rs`
- Example: `examples/material_instancing_demo.rs`
