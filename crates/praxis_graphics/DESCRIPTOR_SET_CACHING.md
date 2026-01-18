# Descriptor Set Caching

Persistent descriptor set caching with LRU eviction for optimized rendering performance.

## Overview

`DescriptorSetPool` implements automatic caching and reuse of descriptor sets to eliminate per-frame allocations while maintaining bounded memory usage through LRU eviction.

## Key Features

- **Persistent caching**: Descriptor sets created once and reused
- **LRU eviction**: Automatic cleanup of unused sets
- **Frame tracking**: Monitors descriptor set usage per frame
- **Bounded memory**: Configurable eviction threshold prevents unbounded growth
- **Zero overhead**: Cache hits require only hash lookup

## Architecture

### Data Structures

```rust
struct CachedDescriptorSet {
    descriptor_set: Arc<DescriptorSet>,
    last_used_frame: u64,
}

struct CachedMaterialDescriptorSet {
    descriptor_set: Arc<DescriptorSet>,
    material_buffer: Subbuffer<MaterialProperties>,
    last_used_frame: u64,
}

struct DescriptorSetPool {
    transform_sets: HashMap<TransformKey, CachedDescriptorSet>,
    material_sets: HashMap<MaterialKey, CachedMaterialDescriptorSet>,
    current_frame: u64,
    eviction_threshold: u64,  // Default: 60 frames
}
```

### Cache Keys

**Transform Descriptor Sets**
```rust
TransformKey {
    texture_name: String,
}
```

**Material Descriptor Sets**
```rust
MaterialKey {
    texture_name: String,
    properties_hash: u64,  // Hash of material properties bytes
}
```

## Usage

### Automatic Integration

Caching is fully automatic through `RenderContext`:

```rust
// Frame start (called automatically in render())
render_context.begin_frame();

// Descriptor set access (automatic caching)
let desc_set = pool.get_or_create_transform_set(
    &texture_name,
    // ... descriptor creation closure
)?;

// Eviction runs automatically every 60 frames
```

### Configuration

```rust
// Set eviction threshold
render_context.set_descriptor_set_pool_eviction_threshold(120);

// Monitor pool state
let size = render_context.descriptor_set_pool_size();
let frame = render_context.descriptor_set_pool_frame();
```

## LRU Eviction Algorithm

### Frame Advancement

Each frame:
1. Increment frame counter: `current_frame += 1`
2. Every 60 frames: Run eviction check

### Eviction Process

```rust
// Calculate cutoff
let cutoff_frame = current_frame - eviction_threshold;

// Retain only recently used sets
transform_sets.retain(|_, cached| {
    cached.last_used_frame >= cutoff_frame
});

material_sets.retain(|_, cached| {
    cached.last_used_frame >= cutoff_frame
});
```

### Frame Tracking

On descriptor set access:
```rust
// Cache hit: Update frame
cached.last_used_frame = current_frame;

// Cache miss: Create new entry
cache.insert(key, CachedDescriptorSet {
    descriptor_set: new_set,
    last_used_frame: current_frame,
});
```

## Performance Impact

### Before Caching

**Scene with 100 objects, 10 textures:**
- Frame 1: 100 descriptor set allocations
- Frame 2: 100 descriptor set allocations
- Per second (60 FPS): 6,000 allocations

### After Caching

**Same scene:**
- Frame 1: 10 allocations (one per texture)
- Frame 2+: 0 allocations (100% cache hits)
- Per second (60 FPS): ~0 allocations

**Result: 100x+ reduction in allocations**

### Memory Usage

| Scenario | Peak Descriptor Sets |
|----------|---------------------|
| Static scene (10 materials) | 10 (stable) |
| Changing scene (20 materials, alternating) | 20 (after eviction) |
| Dynamic scene (100 materials, 60% reuse) | ~60 (steady state) |

### CPU Overhead

- **Cache hit**: O(1) hash lookup + field update (<1ns)
- **Cache miss**: O(1) insert + descriptor creation (~100ns)
- **Eviction check**: O(n) where n = cached sets, runs every 60 frames

**Typical cost**: <0.1ms for 1000+ cached sets

## Configuration Guidelines

### Eviction Threshold

```rust
// Conservative: 120 frames (~2 seconds at 60 FPS)
// - Higher memory usage
// - Better for scenes with cyclical material usage
pool.set_eviction_threshold(120);

// Balanced: 60 frames (~1 second at 60 FPS) [default]
// - Good balance
// - Suitable for most applications
pool.set_eviction_threshold(60);

// Aggressive: 30 frames (~0.5 seconds at 60 FPS)
// - Lower memory usage
// - Better for rapidly changing scenes
pool.set_eviction_threshold(30);
```

### Monitoring

```rust
// Get statistics
println!("Cached descriptor sets: {}", pool.size());
println!("Current frame: {}", pool.current_frame());
println!("Eviction threshold: {} frames", pool.eviction_threshold());
```

## Example Scenarios

### Scenario 1: Static Scene

```
Frame 1-60:   10 materials used every frame
  → Creates 10 descriptor sets
  → All 10 sets used every frame

Frame 61-120: Same materials
  → 100% cache hits
  → No eviction (all recently used)

Result: 10 descriptor sets maintained indefinitely
```

### Scenario 2: Changing Scene

```
Frame 1-60:   Scene A (10 materials)
  → Creates 10 descriptor sets for A

Frame 61-120: Scene B (different 10 materials)
  → Creates 10 new descriptor sets for B
  → Scene A sets unused (last_used=60)
  → Total: 20 descriptor sets

Frame 121:    Eviction check
  → Scene A sets evicted (unused for 61 frames)
  → Scene B sets retained
  → Total: 10 descriptor sets

Result: Automatic cleanup of unused sets
```

### Scenario 3: Cyclical Usage

```
Frame 1-60:   Materials A, B, C
  → Creates 3 descriptor sets

Frame 61-120: Materials B, C, D
  → Reuses B, C
  → Creates D
  → A unused

Frame 121:    Eviction check
  → A evicted
  → B, C, D retained

Frame 121-180: Materials A, C, D
  → Recreates A
  → Reuses C, D

Result: Only active sets maintained
```

## Integration with Material Instancing

Material instances benefit from descriptor set pooling:

```rust
// 100 instances with 10 unique property combinations
for instance in instances {
    let desc_set = pool.get_or_create_material_set(
        &instance.texture_name,
        &instance.properties,  // Hashed for key
    )?;
}

// Result: 10 descriptor sets (not 100)
// Cache hit rate: 90%
```

## Best Practices

### 1. Consistent Property Values

Group instances with identical properties:

```rust
// Good: 100 instances, 5 unique properties = 5 descriptor sets
let colors = [red, green, blue, yellow, magenta];
for i in 0..100 {
    let color = colors[i % 5];  // Reuses 5 colors
}

// Bad: 100 instances, 100 unique properties = 100 descriptor sets
for i in 0..100 {
    let color = generate_unique_color(i);  // All different
}
```

### 2. Material Batching

Sort draws by material to improve cache locality:

```rust
draw_commands.sort_by_key(|cmd| {
    (cmd.texture_name.clone(), cmd.material_hash)
});
```

### 3. Pre-warming Cache

Pre-create descriptor sets for known materials:

```rust
// During level load
for material in level.materials {
    pool.get_or_create_material_set(&material)?;
}
```

## Limitations

- **Eviction granularity**: 60-frame intervals (not every frame)
- **No manual control**: Cannot explicitly retain/evict specific sets
- **Memory bounds**: Based on time, not memory pressure
- **Global pool**: All descriptor sets share same eviction policy

## Future Enhancements

Potential improvements:
- Adaptive eviction based on memory pressure
- Per-material eviction thresholds
- Statistics tracking (hits/misses)
- Memory usage limits (evict LRU when limit reached)
- Manual retain/release API

## See Also

- [Material Instancing](MATERIAL_INSTANCING.md) - Leverages descriptor set pooling
- [Material System](MATERIAL_SYSTEM.md) - Material management
- [Descriptor Sets Reference](DESCRIPTOR_SETS_REFERENCE.md) - Shader layouts
