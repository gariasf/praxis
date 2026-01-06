# Bindless Rendering in Praxis Graphics

This document describes the bindless rendering system implemented using VK_EXT_descriptor_indexing.

## Overview

Bindless rendering eliminates per-material descriptor set binds by using large texture arrays and material indices passed via push constants. This provides dramatic performance improvements for scenes with many materials.

## Architecture

### Traditional Rendering

```
For each unique material (e.g., 100 materials):
  1. Bind material descriptor set (CPU → GPU sync)
  2. For each object using this material (e.g., 10 objects):
     - Push model matrix
     - Draw call

Total: 100 descriptor set binds + 1000 draw calls
```

### Bindless Rendering

```
Bind bindless descriptor set ONCE at start of frame
For each object (e.g., 1000 objects):
  1. Push material index (4 bytes via push constant)
  2. Draw call

Total: 1 descriptor set bind + 1000 draw calls
Result: 100x reduction in descriptor set operations
```

## Components

### 1. BindlessTextureManager

Central manager for bindless textures and materials.

```rust
use praxis_graphics::bindless::BindlessTextureManager;

// Initialize
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

let metal_idx = bindless.register_texture(
    "metal",
    metal_texture.view.clone(),
    metal_texture.sampler.clone(),
)?;

// Register materials
let material_data = BindlessMaterialData {
    base_color: [1.0, 1.0, 1.0, 1.0],
    albedo_texture_index: brick_idx,
    normal_texture_index: 0, // default normal map
    metallic: 0.0,
    roughness: 0.5,
    emissive_strength: 0.0,
    _padding: [0.0; 3],
};

let material_idx = bindless.register_material(material_data)?;

// Get descriptor set for binding
let bindless_descriptor_set = bindless.get_descriptor_set()?;
```

### 2. BindlessMaterialData

GPU-side material structure containing texture indices and material properties.

```rust
#[repr(C)]
pub struct BindlessMaterialData {
    pub base_color: [f32; 4],
    pub albedo_texture_index: u32,
    pub normal_texture_index: u32,
    pub metallic: f32,
    pub roughness: f32,
    pub emissive_strength: f32,
    pub _padding: [f32; 3],
}
```

### 3. Shader Integration

Shaders automatically support both bindless and traditional modes:

```glsl
// Bindless texture array
layout(set = 2, binding = 0) uniform sampler2D bindless_textures[];

// Bindless material data
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
        // Sample from bindless texture array
        BindlessMaterial mat = bindless_materials.materials[push.material_index];
        tex_color = texture(bindless_textures[nonuniformEXT(mat.albedo_texture_index)], v_uv);
        // ... use mat.metallic, mat.roughness, etc.
    } else {
        // Traditional mode: sample from bound textures
        tex_color = texture(albedo_texture, v_uv);
        // ... use material.metallic, material.roughness, etc.
    }
}
```

## RenderContext Integration

### Enabling Bindless Rendering

```rust
// Enable bindless mode
render_context.enable_bindless_rendering()?;

// Access bindless manager
let bindless = render_context.bindless_manager_mut().unwrap();

// Register all textures
for (name, texture) in texture_manager.iter() {
    bindless.register_texture(
        name,
        texture.view.clone(),
        texture.sampler.clone(),
    )?;
}
```

### Rendering with Bindless

When bindless mode is enabled, the render path automatically:

1. Binds the bindless descriptor set once per frame
2. For each draw call:
   - Pushes material index via push constant
   - Draws without rebinding descriptors

Material switches become essentially free (just a push constant write).

### Switching Modes

```rust
// Enable bindless
render_context.enable_bindless_rendering()?;

// Check status
assert!(render_context.is_bindless_enabled());

// Disable (returns to traditional mode)
render_context.disable_bindless_rendering();
```

## Vulkan Requirements

### Extensions

- `VK_EXT_descriptor_indexing` (enabled automatically)

### Features

The following features are enabled in device creation:

- `descriptor_binding_partially_bound`: Allows descriptor sets with unbound entries
- `runtime_descriptor_array`: Enables runtime-sized descriptor arrays
- `descriptor_binding_variable_descriptor_count`: Variable descriptor counts
- `shader_sampled_image_array_non_uniform_indexing`: Non-uniform texture indexing

### Pipeline Configuration

Bindless pipelines include:

- Descriptor set layout for texture arrays (set 2)
- Push constant range for material index (4 bytes)
- Partially bound flag for sparse texture binding

## Performance Characteristics

### CPU Performance

| Scenario | Traditional | Bindless | Improvement |
|----------|-------------|----------|-------------|
| 100 materials, 1000 objects | 100 descriptor binds | 1 descriptor bind | 100x |
| Descriptor set allocation | Per-material, per-frame | Once at initialization | Eliminates allocation |
| CPU→GPU sync | Every material switch | None during rendering | Massive reduction |

### GPU Performance

- **No change in draw call count**: Same number of draws
- **Minimal push constant overhead**: 4 bytes per draw (extremely fast)
- **Texture cache friendly**: All textures in single array improves locality

### Memory Usage

- **Texture array**: O(texture_count) - sparse binding supported
- **Material buffer**: O(material_count * 48 bytes)
- **Capacity**: 4096 textures and 4096 materials maximum

## Limitations

1. **Maximum textures**: 4096 (can be increased via MAX_BINDLESS_TEXTURES)
2. **Maximum materials**: 4096 (can be increased via MAX_BINDLESS_MATERIALS)
3. **Driver support**: Requires VK_EXT_descriptor_indexing
4. **Backward compatibility**: Shaders support both modes, but pipeline must be created with bindless layout

## Best Practices

### 1. Register Textures Early

```rust
// Register during initialization, not per-frame
for texture in all_textures {
    bindless.register_texture(&texture.name, texture.view, texture.sampler)?;
}
```

### 2. Deduplicate Materials

The system automatically deduplicates identical materials:

```rust
// These return the same index if properties match
let idx1 = bindless.register_material(material_data)?;
let idx2 = bindless.register_material(material_data)?;
assert_eq!(idx1, idx2);
```

### 3. Monitor Capacity

```rust
// Check usage
println!("Textures: {}/{}", 
    bindless.texture_count(), 
    bindless::MAX_BINDLESS_TEXTURES
);
println!("Materials: {}/{}", 
    bindless.material_count(), 
    bindless::MAX_BINDLESS_MATERIALS
);
```

### 4. Batch by Material (Optional)

While not required, sorting draws by material index can improve GPU cache coherency:

```rust
draw_commands.sort_by_key(|cmd| cmd.material_index);
```

## Migration Guide

### From Traditional to Bindless

1. **Enable bindless rendering**:
   ```rust
   render_context.enable_bindless_rendering()?;
   ```

2. **Register existing textures**:
   ```rust
   let bindless = render_context.bindless_manager_mut().unwrap();
   for (name, texture) in texture_manager.iter() {
       bindless.register_texture(name, texture.view, texture.sampler)?;
   }
   ```

3. **Create material data**:
   ```rust
   for material in materials {
       let data = BindlessMaterialData {
           base_color: material.base_color.into(),
           albedo_texture_index: bindless.get_texture_index(&material.albedo_texture).unwrap(),
           normal_texture_index: bindless.get_texture_index(&material.normal_map).unwrap(),
           metallic: material.metallic,
           roughness: material.roughness,
           emissive_strength: material.emissive_strength,
           _padding: [0.0; 3],
       };
       bindless.register_material(data)?;
   }
   ```

4. **Render with material indices**:
   The RenderContext automatically handles bindless rendering when enabled.

## Debugging

### Verify Bindless Mode

```rust
if render_context.is_bindless_enabled() {
    println!("✓ Bindless rendering active");
    let bindless = render_context.bindless_manager().unwrap();
    println!("  Textures: {}", bindless.texture_count());
    println!("  Materials: {}", bindless.material_count());
} else {
    println!("✗ Traditional rendering active");
}
```

### Shader Debugging

Use Vulkan validation layers to catch:
- Unbound descriptor array indices
- Out-of-bounds texture access
- Push constant mismatches

### Performance Profiling

Compare descriptor set bind counts:

```
Traditional: vkCmdBindDescriptorSets calls = material_count
Bindless:    vkCmdBindDescriptorSets calls = 1

Reduction = (material_count - 1) / material_count * 100%
For 100 materials: 99% reduction in descriptor binds
```

## Future Enhancements

- **Dynamic texture updates**: Hot-reload textures without recreating descriptor set
- **Texture compression**: Automatic BC7/ASTC compression for bindless textures
- **Multi-frame buffering**: Separate material buffers per frame in flight
- **Hierarchical materials**: Material inheritance and composition
- **Shader permutations**: Compile-time bindless vs traditional shader variants
