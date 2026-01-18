# Bindless Rendering

High-performance material system using VK_EXT_descriptor_indexing for dramatically reduced CPU overhead.

## Overview

Bindless rendering eliminates per-material descriptor set binds by using large texture arrays and material indices passed via push constants.

**Benefits:**
- 100× reduction in descriptor set operations
- Single bind for entire frame
- Eliminates CPU-GPU sync overhead
- Efficient for scenes with many materials

## Architecture

### Traditional vs Bindless

**Traditional Rendering:**
```
For each unique material (100 materials):
  1. Bind material descriptor set    ← CPU-GPU sync
  2. For each object (10 objects):
     - Push model matrix
     - Draw call

Total: 100 descriptor binds + 1000 draws
```

**Bindless Rendering:**
```
Bind bindless descriptor set ONCE
For each object (1000 objects):
  1. Push material index (4 bytes)  ← Fast
  2. Draw call

Total: 1 descriptor bind + 1000 draws
Result: 99% reduction in descriptor operations
```

## Quick Start

### Initialization

```rust
use praxis_graphics::bindless::BindlessTextureManager;

// Create bindless manager
let mut bindless = BindlessTextureManager::new(
    device,
    memory_allocator,
    descriptor_set_allocator,
)?;

// Register textures
let brick_idx = bindless.register_texture(
    "brick",
    brick_texture.view.clone(),
    brick_texture.sampler.clone(),
)?;

// Register materials
let material_data = BindlessMaterialData {
    base_color: [1.0, 1.0, 1.0, 1.0],
    albedo_texture_index: brick_idx,
    normal_texture_index: 0,
    metallic: 0.0,
    roughness: 0.5,
    emissive_strength: 0.0,
    _padding: [0.0; 3],
};
let material_idx = bindless.register_material(material_data)?;
```

### Rendering

```rust
// Enable bindless mode
render_context.enable_bindless_rendering()?;

// Bindless manager handles descriptor binding automatically
// Just push material index per draw:
command_buffer.push_constants(
    pipeline.layout().clone(),
    0,
    &material_idx,
)?;
```

## Shader Integration

Shaders automatically support both bindless and traditional modes:

```glsl
// Set 2: Bindless resources
layout(set = 2, binding = 0) uniform sampler2D bindless_textures[];
layout(set = 2, binding = 1, std140) uniform BindlessMaterialData {
    BindlessMaterial materials[4096];
} bindless_materials;

// Push constant for material index
layout(push_constant) uniform PushConstants {
    uint material_index;
} push;

void main() {
    // Automatic mode detection
    bool use_bindless = (push.material_index != 0xFFFFFFFF);
    
    if (use_bindless) {
        // Bindless path
        BindlessMaterial mat = bindless_materials.materials[push.material_index];
        tex_color = texture(
            bindless_textures[nonuniformEXT(mat.albedo_texture_index)], 
            v_uv
        );
        metallic = mat.metallic;
        roughness = mat.roughness;
    } else {
        // Traditional path
        tex_color = texture(albedo_texture, v_uv);
        metallic = material.metallic;
        roughness = material.roughness;
    }
}
```

## Data Structures

### BindlessMaterialData

```rust
#[repr(C)]
pub struct BindlessMaterialData {
    pub base_color: [f32; 4],
    pub albedo_texture_index: u32,
    pub normal_texture_index: u32,
    pub metallic: f32,
    pub roughness: f32,
    pub emissive_strength: f32,
    pub _padding: [f32; 3],  // std140 alignment
}
```

### Capacity

- **Max textures**: 4096 (configurable via `MAX_BINDLESS_TEXTURES`)
- **Max materials**: 4096 (configurable via `MAX_BINDLESS_MATERIALS`)

## RenderContext Integration

### Enable Bindless

```rust
// Enable bindless mode
render_context.enable_bindless_rendering()?;

// Register textures
let bindless = render_context.bindless_manager_mut().unwrap();
for (name, texture) in texture_manager.iter() {
    bindless.register_texture(
        name,
        texture.view.clone(),
        texture.sampler.clone(),
    )?;
}

// Create materials
let material_data = BindlessMaterialData { /* ... */ };
let material_idx = bindless.register_material(material_data)?;
```

### Automatic Deduplication

Identical materials are automatically deduplicated:

```rust
let idx1 = bindless.register_material(material_data)?;
let idx2 = bindless.register_material(material_data)?;
assert_eq!(idx1, idx2);  // Same index returned
```

### Check Status

```rust
if render_context.is_bindless_enabled() {
    println!("Bindless rendering active");
    let bindless = render_context.bindless_manager().unwrap();
    println!("  Textures: {}", bindless.texture_count());
    println!("  Materials: {}", bindless.material_count());
}
```

## Performance

### CPU Performance

| Scenario | Traditional | Bindless | Improvement |
|----------|-------------|----------|-------------|
| 100 materials, 1000 objects | 100 binds | 1 bind | 100× |
| Descriptor allocation | Per-material | Once | Eliminates |
| CPU→GPU sync | Every material | Once | Massive reduction |

### GPU Performance

- **No draw call overhead**: Same number of draws
- **Push constant**: 4 bytes per draw (extremely fast)
- **Texture cache**: All textures in array improves locality

### Memory Usage

- **Texture array**: Sparse binding supported (only used slots allocated)
- **Material buffer**: 48 bytes × material count
- **Overhead**: Minimal compared to traditional approach

## Vulkan Requirements

### Extensions

- `VK_EXT_descriptor_indexing` (automatically enabled)

### Features

Required features enabled automatically:
- `descriptor_binding_partially_bound`
- `runtime_descriptor_array`
- `descriptor_binding_variable_descriptor_count`
- `shader_sampled_image_array_non_uniform_indexing`

### Pipeline Configuration

- Set 2 layout for texture arrays
- Push constant range (4 bytes)
- Partially bound flag for sparse binding

## Best Practices

### 1. Register Textures Early

```rust
// At initialization, not per-frame
for texture in all_textures {
    bindless.register_texture(&texture.name, texture.view, texture.sampler)?;
}
```

### 2. Monitor Capacity

```rust
println!("Textures: {}/{}", 
    bindless.texture_count(), 
    MAX_BINDLESS_TEXTURES
);
println!("Materials: {}/{}", 
    bindless.material_count(), 
    MAX_BINDLESS_MATERIALS
);
```

### 3. Batch by Material (Optional)

While not required, sorting can improve GPU cache:

```rust
draw_commands.sort_by_key(|cmd| cmd.material_index);
```

### 4. Fallback Support

Shaders support both modes for compatibility:

```glsl
// 0xFFFFFFFF = use traditional path
push.material_index = 0xFFFFFFFF;
```

## Migration from Traditional

### 1. Enable Bindless

```rust
render_context.enable_bindless_rendering()?;
```

### 2. Register Textures

```rust
let bindless = render_context.bindless_manager_mut().unwrap();
for (name, texture) in texture_manager.iter() {
    bindless.register_texture(name, texture.view, texture.sampler)?;
}
```

### 3. Create Materials

```rust
for material in materials {
    let data = BindlessMaterialData {
        base_color: material.base_color.into(),
        albedo_texture_index: bindless.get_texture_index(&material.albedo_texture)?,
        normal_texture_index: bindless.get_texture_index(&material.normal_map)?,
        metallic: material.metallic,
        roughness: material.roughness,
        emissive_strength: material.emissive_strength,
        _padding: [0.0; 3],
    };
    let idx = bindless.register_material(data)?;
    // Store idx for rendering
}
```

### 4. Render

RenderContext automatically handles bindless rendering when enabled.

## Limitations

1. **Texture limit**: 4096 textures (hardware dependent)
2. **Material limit**: 4096 materials (configurable)
3. **Driver support**: Requires VK_EXT_descriptor_indexing
4. **Fallback required**: Must support traditional path for compatibility

## Future Enhancements

- Dynamic texture updates (hot-reload)
- Automatic texture compression (BC7/ASTC)
- Multi-frame buffering for material buffers
- Hierarchical material system
- Compile-time shader variants

## See Also

- [Material System](MATERIAL_SYSTEM.md)
- [Descriptor Sets Reference](DESCRIPTOR_SETS_REFERENCE.md)
- Implementation: `crates/praxis_graphics/src/bindless.rs`
