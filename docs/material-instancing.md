# Material Instancing Guide

Material instancing is a powerful technique for efficiently handling multiple objects that share textures but have different material properties.

## Concept

Traditional approach:
```
Object 1 → Material A (textures + properties)
Object 2 → Material B (textures + properties)  // Same textures!
Object 3 → Material C (textures + properties)  // Same textures!
```

With instancing:
```
Object 1 → Instance 1 → Base Material (textures)
Object 2 → Instance 2 → Base Material (textures)
Object 3 → Instance 3 → Base Material (textures)
```

## Benefits

### Memory Efficiency

**Without instancing:**
- 100 objects with similar materials
- Each loads 8MB of textures
- Total: 800MB VRAM

**With instancing:**
- 100 instances of 10 base materials
- Only base materials load textures
- Total: 80MB VRAM (90% savings!)

### GPU Performance

**Descriptor Sets:**
- Traditional: 100 objects = 100 descriptor sets/frame
- Instanced: 100 objects = 10 descriptor sets/frame

**Texture Binds:**
- Only when base material changes
- 90% reduction in GPU state changes

## Usage

### Basic Instancing

```rust
use praxis_graphics::{MaterialManager, MaterialInstance, MaterialProperties};

// Create base material
let mut material_manager = MaterialManager::new();
material_manager.create_material("metal_base", albedo_texture);

// Create instance with overrides
let base = material_manager.get_material("metal_base").unwrap();
let instance = MaterialInstance::new(base)
    .override_properties(
        MaterialProperties::new()
            .with_metallic(0.9)
            .with_roughness(0.2)
    );
```

### Instance Manager

```rust
use praxis_graphics::MaterialInstanceManager;

let mut instance_manager = MaterialInstanceManager::new();

// Create instance
instance_manager.create_instance("shiny_metal", base_material)
    .override_properties(
        MaterialProperties::new().with_roughness(0.1)
    );

// Get instance
if let Some(instance) = instance_manager.get_instance("shiny_metal") {
    let props = instance.properties();
    // Use properties...
}

// Statistics
let stats = instance_manager.compute_stats();
println!("Total instances: {}", stats.total_instances);
println!("Unique bases: {}", stats.unique_base_materials);
println!("Avg instances/base: {:.1}", stats.avg_instances_per_base);
```

## When to Use

### ✅ Good Use Cases

**Character Variation:**
```rust
// Base: Character skin texture
// Instances: Different skin tones, wetness, dirt levels
for i in 0..10 {
    let skin_tone = [0.8 + i as f32 * 0.02, 0.6, 0.5, 1.0];
    instances.push(
        MaterialInstance::new(base_skin)
            .override_properties(
                MaterialProperties::new()
                    .with_base_color(skin_tone)
            )
    );
}
```

**Environment Objects:**
```rust
// Base: Rock texture
// Instances: Different weathering levels
let instances = vec![
    MaterialInstance::new(base_rock)
        .override_properties(MaterialProperties::new().with_roughness(0.3)),
    MaterialInstance::new(base_rock)
        .override_properties(MaterialProperties::new().with_roughness(0.7)),
    MaterialInstance::new(base_rock)
        .override_properties(MaterialProperties::new().with_roughness(0.9)),
];
```

**Vegetation:**
```rust
// Base: Leaf texture
// Instances: Seasonal color variation
let summer = MaterialInstance::new(base_leaf)
    .override_properties(
        MaterialProperties::new()
            .with_base_color([0.3, 0.8, 0.2, 1.0])
    );

let autumn = MaterialInstance::new(base_leaf)
    .override_properties(
        MaterialProperties::new()
            .with_base_color([0.9, 0.5, 0.1, 1.0])
    );
```

### ❌ Bad Use Cases

**Completely Different Materials:**
```rust
// Don't do this - no shared textures!
let wood = create_base("wood", wood_texture);
let metal = MaterialInstance::new(wood)  // Wrong!
    .override_properties(metal_props);

// Do this instead:
let wood = Material::new("wood", wood_texture);
let metal = Material::new("metal", metal_texture);
```

**Unique Textures:**
```rust
// Don't instance if each needs unique textures
// Each instance would override all textures anyway
```

## Performance Tips

### Optimal Instance Count

```rust
// Sweet spot: 10-20 instances per base material
// Too few (2-3): Little benefit
// Too many (100+): Consider splitting into multiple bases
```

### Batching

```rust
// Group draw calls by base material
draw_calls.sort_by(|a, b| {
    a.instance.base_material().id.cmp(&b.instance.base_material().id)
});
```

### Memory Budget

```rust
let stats = instance_manager.compute_stats();

// Rule of thumb: Aim for 10:1 ratio
let efficiency = stats.total_instances as f32 / stats.unique_base_materials as f32;
if efficiency < 5.0 {
    println!("Warning: Low instancing efficiency ({:.1}:1)", efficiency);
}
```

## Advanced Patterns

### Dynamic Property Changes

```rust
// Update instance properties at runtime
if let Some(instance) = instance_manager.get_instance_mut("my_instance") {
    *instance = instance.clone()
        .override_properties(
            MaterialProperties::new()
                .with_emissive_strength(2.0)  // Make it glow
        );
}
```

### Progressive Loading

```rust
// Load base materials first
for base in essential_bases {
    material_manager.add_material(base);
}

// Create instances as needed
spawn_objects()
    .map(|obj| MaterialInstance::new(get_base(obj.type)))
    .collect();
```

### LOD Integration

```rust
// Use instances for LOD levels
struct LodMaterial {
    high: MaterialInstance,  // Full properties
    medium: MaterialInstance,  // Simplified
    low: MaterialInstance,   // Minimal
}

// All share same base textures
```

## Debugging

### Instance Statistics

```rust
let stats = instance_manager.compute_stats();
println!("=== Instancing Stats ===");
println!("Total instances: {}", stats.total_instances);
println!("Unique bases: {}", stats.unique_base_materials);
println!("With overrides: {}", stats.instances_with_overrides);
println!("Avg per base: {:.2}", stats.avg_instances_per_base);

// Calculate memory savings
let without = stats.total_instances * 8; // 8MB per texture
let with = stats.unique_base_materials * 8;
println!("Memory saved: {}MB", without - with);
```

### Property Tracking

```rust
for (id, instance) in instance_manager.iter() {
    println!("Instance: {}", id);
    println!("  Base: {}", instance.base_material().id);
    println!("  Has overrides: {}", instance.has_overrides());
    
    if instance.has_overrides() {
        let props = instance.properties();
        println!("  Metallic: {}", props.metallic);
        println!("  Roughness: {}", props.roughness);
    }
}
```

## Common Pitfalls

### Over-Instancing

```rust
// DON'T: Instance for no benefit
let instance = MaterialInstance::new(base);
// No overrides! Just use base directly.
```

### Texture Overrides

```rust
// DON'T: Override all textures (defeats purpose)
material.set_albedo_texture(new_texture);
material.set_normal_texture(Some(new_normal));
// At this point, should just be separate material
```

### Excessive Property Changes

```rust
// DON'T: Change instance properties every frame
// GPU upload cost negates benefits

// DO: Pre-create instances for common states
let presets = vec![
    MaterialInstance::new(base).override_properties(wet_props),
    MaterialInstance::new(base).override_properties(dry_props),
];
// Switch between presets instead
```

## See Also

- [Material System Overview](advanced_materials.md)
- [Material Layers](MATERIAL_LAYERS.md)
- [Performance Guide](performance_guide.md)
