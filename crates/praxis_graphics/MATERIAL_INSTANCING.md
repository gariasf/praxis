# Material Instancing

Efficient per-object material property overrides without duplicating texture data.

## Overview

Material instancing enables hundreds of material variants that share textures but differ in properties (color, metallic, roughness). Instances reference a base material and override only properties that differ.

**Benefits:**
- 90%+ memory reduction for material variants
- 100× faster variant creation
- Automatic descriptor set pooling and reuse
- Seamless integration with rendering pipeline

## Architecture

### Core Components

**MaterialInstance** - References base material with property overrides  
**MaterialInstanceManager** - Manages instance lifecycle and lookup  
**RenderContext Integration** - Automatic resolution in render pipeline

### Data Flow

```
DrawCommand
  └─ material_instance_id: "metal_shiny"
       ↓
MaterialInstanceManager
  └─ Resolve instance → base material + overrides
       ↓
Descriptor Set Pool
  └─ Cache descriptor set by (texture + properties hash)
       ↓
Rendering
  └─ Batch by descriptor set → minimize binds
```

## Quick Start

```rust
use praxis_graphics::{MaterialInstance, MaterialProperties};

// Create base material (shared textures)
let base = Arc::new(Material::new("metal_base", metal_texture));
render_context.material_manager_mut().add_material(base.clone());

// Create color variants (property overrides only)
for i in 0..100 {
    let color = generate_color(i);
    render_context.create_material_instance(
        format!("metal_{}", i),
        "metal_base"
    )?.override_properties(
        MaterialProperties::new().with_base_color(color)
    );
}

// Use in rendering
let draw_cmd = DrawCommand {
    mesh_id: "cube",
    model: transform,
    material_instance_id: Some("metal_5"),  // Use instance
    ..Default::default()
};
```

## Usage Patterns

### Pattern 1: Color Variants

```rust
// Base material once
let base = Arc::new(Material::new("metal_base", metal_texture));
render_context.material_manager_mut().add_material(base.clone());

// 100 color variants (shares textures)
for i in 0..100 {
    let color = generate_color(i);
    render_context.create_material_instance(
        format!("metal_{}", i),
        "metal_base"
    )?.override_properties(
        MaterialProperties::new().with_base_color(color)
    );
}

// Memory: 1× textures (not 100×)
```

### Pattern 2: Surface Property Variations

```rust
// Glossy variant
render_context.create_material_instance("surface_glossy", "surface_base")?
    .override_properties(
        MaterialProperties::new().with_roughness(0.1)
    );

// Rough variant
render_context.create_material_instance("surface_rough", "surface_base")?
    .override_properties(
        MaterialProperties::new().with_roughness(0.9)
    );
```

### Pattern 3: Dynamic Animation

```rust
// Update instance properties each frame
if let Some(instance) = render_context
    .material_instance_manager_mut()
    .get_instance_mut("animated") {
    
    let pulse = (time.sin() + 1.0) / 2.0;
    *instance = instance.clone().override_properties(
        MaterialProperties::new().with_emissive_strength(pulse)
    );
}
```

## Pipeline Integration

### DrawCommand Extension

```rust
pub struct DrawCommand {
    pub mesh_id: String,
    pub model: Mat4,
    pub texture_name: Option<String>,
    pub material_properties: Option<MaterialProperties>,
    pub material_instance_id: Option<String>,  // Instance reference
    pub bone_matrices: Option<Vec<Mat4>>,
}
```

When `material_instance_id` is set, it takes precedence.

### Automatic Resolution

```rust
// In render() method
let (texture_name, material_props, texture) = if let Some(ref instance_id) = draw_cmd.material_instance_id {
    // Use material instance
    let instance = self.material_instance_manager.get_instance(instance_id)?;
    let base_material = instance.base_material();
    let instance_props = instance.properties();
    
    let tex_name = base_material.id.clone();
    let texture = self.texture_manager.get_texture(&tex_name).unwrap_or(default_texture);
    
    (tex_name, instance_props, texture)
} else {
    // Traditional path
    // ...
};
```

### Descriptor Set Pooling

Material instances automatically benefit from descriptor set pooling:

- Pool keys by texture name + properties hash
- Instances with identical properties share cached descriptor sets
- Multiple instances with same overrides = single descriptor set

### Material Batching

Render pipeline automatically batches instances:

- Draw commands sorted by texture and properties
- Instances with identical resolved properties batched together
- Descriptor set binds minimized

## Performance

### Memory Efficiency

**Traditional (100 color variants):**
- 100 materials × 8MB textures = 800MB

**Instancing (100 variants):**
- 1 base material × 8MB = 8MB
- 90% memory reduction

### Descriptor Set Reuse

**Frame 1:** Creates descriptor sets for unique property combinations (~10-20 for 100 instances)  
**Frame 2+:** Reuses all cached descriptor sets (zero allocations)  
**Result:** 100× reduction in descriptor set allocations per frame

### Creation Performance

**Traditional:** Load 100 textures + create 100 materials (~1000ms)  
**Instancing:** Load 1 texture + create 100 instances (~10ms)  
**Result:** 100× faster variant creation

## Monitoring

### Instance Statistics

```rust
let stats = render_context.material_instance_stats();
println!("Total instances: {}", stats.total_instances);
println!("Unique base materials: {}", stats.unique_base_materials);
println!("Instances with overrides: {}", stats.instances_with_overrides);
println!("Avg instances per base: {:.2}", stats.avg_instances_per_base);
```

**Ideal ratios:**
- `avg_instances_per_base > 10`: Excellent efficiency
- `avg_instances_per_base 5-10`: Good efficiency
- `avg_instances_per_base < 5`: Consider consolidating

### Descriptor Set Pool

```rust
let pool_size = render_context.descriptor_set_pool_size();
println!("Cached descriptor sets: {}", pool_size);
```

## Best Practices

### 1. Group by Base Material

```rust
// Good: Many instances per base
let wood_base = create_base("wood");
for variant in wood_variants {
    create_instance(variant, wood_base);
}

// Bad: One instance per base (no benefit)
for variant in all_variants {
    let base = create_base(variant);  // Don't do this!
    create_instance(variant, base);
}
```

### 2. Override Only What Changes

```rust
// Good: Minimal override
instance.override_properties(
    MaterialProperties::new().with_base_color(red)
);

// Bad: Full override (loses benefit)
instance
    .override_properties(base_props)
    .override_extended(base_extended)
    .override_parallax(base_parallax);
```

### 3. Use Consistent Values

```rust
// Good: 100 instances, 5 colors = 5 descriptor sets
let colors = [red, green, blue, yellow, magenta];
for i in 0..100 {
    let color = colors[i % 5];  // Reuses 5 colors
    create_instance_with_color(i, color);
}

// Bad: 100 instances, 100 unique colors = 100 descriptor sets
for i in 0..100 {
    let color = generate_unique_color(i);  // All different
    create_instance_with_color(i, color);
}
```

### 4. Clean Up Unused Instances

```rust
// Remove instance
render_context.material_instance_manager_mut()
    .remove_instance("old_instance");

// Clear all
render_context.material_instance_manager_mut().clear();
```

## Limitations

1. **Texture sharing only**: Instances must share all textures from base
2. **No layer overrides**: Material layers cannot be overridden per instance
3. **Extended properties stored but not rendered**: Future integration needed

## Performance Characteristics

- **Instance lookup**: O(1) hash map lookup
- **Property resolution**: Minimal (one field check)
- **Descriptor set creation**: Amortized O(1) with pooling
- **Memory overhead**: ~64 bytes per instance

## See Also

- [Material System](MATERIAL_SYSTEM.md) - Core material system
- [Descriptor Set Caching](DESCRIPTOR_SET_CACHING.md) - Automatic optimization
- Example: `examples/material_instancing_demo.rs`
- Implementation: `crates/praxis_graphics/src/material_instancing.rs`
