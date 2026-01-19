# Rendering Performance Optimization Guide

Comprehensive guide to tuning Praxis rendering performance through LOD management, mesh streaming, culling optimizations, and hardware-specific configurations.

## Table of Contents

1. [Overview](#overview)
2. [LOD Distance Thresholds](#lod-distance-thresholds)
3. [Mesh Streaming Priorities](#mesh-streaming-priorities)
4. [Descriptor Cache Configuration](#descriptor-cache-configuration)
5. [Hi-Z Occlusion Culling](#hi-z-occlusion-culling)
6. [GPU Culling Optimization](#gpu-culling-optimization)
7. [Profiling Methodology](#profiling-methodology)
8. [Hardware Tier Configurations](#hardware-tier-configurations)
9. [Trade-off Analysis](#trade-off-analysis)

---

## Overview

Praxis provides multiple rendering optimization systems that work together to maximize performance across different hardware tiers. This guide covers configuration parameters, trade-offs, and recommended baselines for common scenarios.

### Key Systems

- **LOD (Level of Detail)**: Distance-based mesh switching to reduce polygon count
- **Mesh Streaming**: Background loading of mesh data based on visibility
- **GPU Culling**: Parallel frustum and occlusion testing on compute shaders
- **Descriptor Caching**: Reuse of descriptor sets to reduce allocation overhead
- **Hi-Z Occlusion**: Hierarchical depth buffer for conservative occlusion culling

### Performance Philosophy

> Optimize for the **common case**, provide fallbacks for the **worst case**, and profile the **actual case**.

---

## LOD Distance Thresholds

### Fundamentals

LOD selection uses **squared distances** to avoid expensive `sqrt()` operations:

```rust
// Efficient distance test (no sqrt)
let delta = object_position - camera_position;
let distance_squared = delta.length_squared();

if distance_squared < lod_threshold_squared {
    // Use high detail mesh
}
```

### Distance Calculation Methods

#### 1. Screen Coverage Based (Recommended)

Calculate thresholds based on desired screen coverage:

```rust
// Target: LOD switches when object occupies N pixels on screen
fn calculate_lod_distance(
    object_radius: f32,
    target_pixels: f32,
    screen_height: f32,
    fov_vertical: f32,
) -> f32 {
    // tan(fov/2) = (screen_height/2) / distance
    // object_screen_height = (object_radius / distance) * (screen_height / tan(fov/2))
    let tan_half_fov = (fov_vertical / 2.0).tan();
    let distance = (object_radius * screen_height) / (target_pixels * tan_half_fov * 2.0);
    distance
}

// Example: Switch to LOD1 when object is < 100 pixels tall
let lod1_distance = calculate_lod_distance(
    5.0,    // 5 unit radius
    100.0,  // 100 pixel target
    1080.0, // 1080p screen
    std::f32::consts::PI / 3.0, // 60° FOV
);
// Result: ~93.5 units
```

#### 2. Polygon Budget Based

Calculate based on target triangle count:

```rust
fn calculate_lod_by_polygon_budget(
    lod_levels: &[(usize, usize)], // (triangle_count, target_on_screen)
    total_budget: usize,
    typical_visible: usize,
) -> Vec<f32> {
    lod_levels
        .iter()
        .map(|(tri_count, target_on_screen)| {
            // Distance where this LOD contributes appropriately to budget
            let budget_per_object = total_budget / typical_visible;
            let distance_factor = (*tri_count as f32 / budget_per_object as f32).sqrt();
            distance_factor * 50.0 // Base distance scale
        })
        .collect()
}

// Example: 2M triangle budget, 100 typical visible objects
let lod_distances = calculate_lod_by_polygon_budget(
    &[
        (10000, 20),  // LOD0: 10k tris, target 20 on screen
        (5000, 40),   // LOD1: 5k tris, target 40 on screen
        (1000, 40),   // LOD2: 1k tris, target 40+ on screen
    ],
    2_000_000, // 2M tri budget
    100,       // 100 typical visible
);
```

### Recommended Thresholds by Object Type

| Object Type | LOD0 (High) | LOD1 (Medium) | LOD2 (Low) | LOD3 (Impostor) |
|-------------|-------------|---------------|------------|-----------------|
| **Hero Character** | 0-20 units | 20-60 units | 60-150 units | 150-300 units |
| **Environment Props** | 0-15 units | 15-50 units | 50-120 units | 120-250 units |
| **Vegetation** | 0-10 units | 10-30 units | 30-80 units | 80-200 units |
| **Background Objects** | 0-30 units | 30-80 units | 80-200 units | 200-500 units |
| **Terrain Chunks** | 0-40 units | 40-100 units | 100-250 units | 250+ units |

### Configuration Examples

#### Aggressive Performance (Lower Quality)

```rust
use praxis_spatial::lod::LodGroup;

let lod_group = LodGroup::new(vec![
    LodLevel::new("mesh_high", 0.0, 12.0),    // Very close only
    LodLevel::new("mesh_medium", 12.0, 35.0), // Medium quickly
    LodLevel::new("mesh_low", 35.0, 100.0),   // Low detail at distance
]);
```

#### Balanced Quality (Default)

```rust
let lod_group = LodGroup::new(vec![
    LodLevel::new("mesh_high", 0.0, 20.0),    // High detail nearby
    LodLevel::new("mesh_medium", 20.0, 60.0), // Medium at moderate distance
    LodLevel::new("mesh_low", 60.0, 150.0),   // Low at far distance
]);
```

#### Quality Focused (Higher Detail)

```rust
let lod_group = LodGroup::new(vec![
    LodLevel::new("mesh_high", 0.0, 35.0),     // Extended high detail
    LodLevel::new("mesh_medium", 35.0, 100.0), // Medium farther out
    LodLevel::new("mesh_low", 100.0, 250.0),   // Low only very far
]);
```

### LOD Bias

Global or per-object adjustment to shift all thresholds:

```rust
// Positive bias: prefer higher detail (multiply distances by factor > 1.0)
lod_group.set_lod_bias(0.3); // Effective 30% distance increase

// Negative bias: prefer lower detail (multiply distances by factor < 1.0)
lod_group.set_lod_bias(-0.3); // Effective 30% distance decrease

// Global bias (affects all LOD groups)
let mut lod_manager = LodManager::new();
lod_manager.set_global_lod_bias(0.2); // 20% quality boost
```

### Hysteresis to Prevent Popping

Add dead zones to prevent rapid LOD switching:

```rust
// Add 10% hysteresis to prevent flickering at boundaries
let lod_group = LodGroup::new(vec![
    LodLevel::new("mesh_high", 0.0, 20.0),
    LodLevel::new("mesh_medium", 22.0, 60.0),  // 2 unit gap (10%)
    LodLevel::new("mesh_low", 66.0, 150.0),    // 6 unit gap (10%)
]);
```

---

## Mesh Streaming Priorities

### Priority Calculation

The mesh streaming system uses a priority queue to load visible meshes based on multiple factors:

```rust
pub fn calculate_streaming_priority(
    is_visible: bool,
    distance_to_camera: f32,
    bounding_radius: f32,
    time_since_request: f32,
) -> f32 {
    if !is_visible {
        return 0.0; // Don't load invisible meshes
    }

    // Visibility priority (closer to camera = higher priority)
    let distance_priority = if distance_to_camera < bounding_radius * 2.0 {
        100.0 // Very close, immediate load
    } else if distance_to_camera < bounding_radius * 10.0 {
        50.0 // Moderate distance
    } else {
        10.0 // Far but visible
    };

    // Distance-based priority (inverse relationship)
    let inverse_distance_priority = 1000.0 / (distance_to_camera + 1.0);

    // Time-based priority (prevent starvation)
    let age_priority = time_since_request * 5.0;

    distance_priority + inverse_distance_priority + age_priority
}
```

### Configuration Parameters

```rust
use praxis_graphics::mesh::MeshStreamingSystem;

let streaming_system = MeshStreamingSystem::new(
    allocator,
    command_buffer_allocator,
    transfer_queue,
);

// Register mesh with bounding sphere for streaming
streaming_system.register_mesh("tree_lod2", mesh_data)?;

// Priority is automatically calculated based on:
// - Frustum visibility (is_visible check)
// - Distance from camera
// - Bounding sphere radius
// - Time in queue
```

### Streaming Budget

Control maximum simultaneous loads to prevent GPU stalls:

```rust
// Recommended: 2-4 meshes per frame for 60 FPS
const MAX_MESHES_PER_FRAME: usize = 3;
const MAX_BYTES_PER_FRAME: usize = 4 * 1024 * 1024; // 4 MB/frame

// Batch loading based on budget
let mut loaded_count = 0;
let mut loaded_bytes = 0;

for pending_mesh in pending_queue.by_priority() {
    if loaded_count >= MAX_MESHES_PER_FRAME {
        break;
    }
    if loaded_bytes + pending_mesh.size_bytes() > MAX_BYTES_PER_FRAME {
        break;
    }

    streaming_system.request_load(&pending_mesh.id, mesh_data, priority);
    loaded_count += 1;
    loaded_bytes += pending_mesh.size_bytes();
}
```

### Preloading Strategies

#### 1. Distance-Based Preloading

Load meshes before they become visible:

```rust
// Preload meshes within preload radius (larger than max render distance)
const PRELOAD_DISTANCE: f32 = 200.0; // Start loading at 200 units
const RENDER_DISTANCE: f32 = 150.0;   // Render up to 150 units

if distance_to_camera < PRELOAD_DISTANCE && !mesh_loaded {
    let preload_priority = calculate_streaming_priority(
        true,
        distance_to_camera,
        bounding_radius,
        time_since_visible,
    ) * 0.5; // Lower priority than visible meshes
    
    streaming_system.request_load(mesh_id, mesh_data, preload_priority);
}
```

#### 2. Camera Velocity Prediction

Predict camera movement to preload ahead:

```rust
// Predict where camera will be in 1 second
let predicted_position = camera_position + camera_velocity * 1.0;

// Calculate priority based on predicted distance
let predicted_distance = (object_position - predicted_position).length();
let prediction_priority = calculate_streaming_priority(
    true,
    predicted_distance,
    bounding_radius,
    0.0,
) * 0.75; // Slightly lower than actual visibility
```

#### 3. Portal-Based Preloading

Preload meshes in adjacent areas:

```rust
// When player is near a portal/doorway, preload destination area
if player_near_portal {
    for mesh_id in portal_destination_meshes {
        streaming_system.request_load(
            mesh_id,
            mesh_data,
            50.0, // Medium priority
        );
    }
}
```

### Memory Management

Set limits to prevent out-of-memory conditions:

```rust
// Track total loaded mesh memory
struct StreamingMemoryManager {
    max_loaded_memory: usize,
    current_loaded_memory: usize,
    loaded_meshes: HashMap<String, usize>, // mesh_id -> size
}

impl StreamingMemoryManager {
    pub fn can_load(&self, mesh_size: usize) -> bool {
        self.current_loaded_memory + mesh_size <= self.max_loaded_memory
    }

    pub fn evict_least_recently_used(&mut self) -> Option<String> {
        // Evict LRU mesh to make space
        // Implementation depends on tracking access times
        todo!()
    }
}

// Typical memory budgets
// - Low-end: 256 MB for streamed meshes
// - Mid-range: 512 MB for streamed meshes
// - High-end: 1 GB+ for streamed meshes
```

---

## Descriptor Cache Configuration

### Background

Vulkan descriptor sets allocate GPU resources for shaders. Creating/destroying them has overhead. The descriptor cache reuses sets across frames.

### Cache Implementation

```rust
use lru::LruCache;
use std::num::NonZeroUsize;

pub struct DescriptorSetCache {
    cache: LruCache<DescriptorSetKey, Arc<DescriptorSet>>,
    hits: usize,
    misses: usize,
}

impl DescriptorSetCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
            hits: 0,
            misses: 0,
        }
    }

    pub fn hit_rate(&self) -> f32 {
        let total = self.hits + self.misses;
        if total == 0 {
            return 0.0;
        }
        self.hits as f32 / total as f32
    }
}
```

### Cache Size Recommendations

| Scene Complexity | Unique Materials | Unique Textures | Recommended Cache Size |
|------------------|------------------|-----------------|------------------------|
| **Simple** | < 20 | < 50 | 128 entries |
| **Moderate** | 20-100 | 50-200 | 256 entries |
| **Complex** | 100-500 | 200-1000 | 512 entries |
| **Very Complex** | 500+ | 1000+ | 1024 entries |

### Eviction Policies

#### LRU (Least Recently Used) - Default

Best for general-purpose rendering with varied scene contents:

```rust
// LRU evicts oldest unused entries
let cache = DescriptorSetCache::new(256); // Evicts LRU when full
```

**Pros:**
- Good general-purpose behavior
- Adapts to access patterns
- Simple to implement

**Cons:**
- May evict descriptors that will be needed soon
- No consideration for creation cost

#### LFU (Least Frequently Used)

Better for scenes with stable, frequently reused materials:

```rust
pub struct LfuDescriptorCache {
    cache: HashMap<DescriptorSetKey, (Arc<DescriptorSet>, usize)>, // value + frequency
    max_size: usize,
}

impl LfuDescriptorCache {
    pub fn get_or_create(&mut self, key: DescriptorSetKey) -> Arc<DescriptorSet> {
        if let Some((descriptor, freq)) = self.cache.get_mut(&key) {
            *freq += 1; // Increment frequency
            return descriptor.clone();
        }

        // Cache miss - create new descriptor
        let descriptor = self.create_descriptor(&key);

        // Evict LFU if at capacity
        if self.cache.len() >= self.max_size {
            let lfu_key = self.cache
                .iter()
                .min_by_key(|(_, (_, freq))| freq)
                .map(|(k, _)| k.clone())
                .unwrap();
            self.cache.remove(&lfu_key);
        }

        self.cache.insert(key, (descriptor.clone(), 1));
        descriptor
    }
}
```

**Pros:**
- Retains frequently used descriptors
- Good for stable scene contents

**Cons:**
- Newly loaded descriptors may be evicted quickly
- Requires frequency tracking overhead

#### Size-Aware Eviction

Evict based on memory cost:

```rust
pub struct SizeAwareCache {
    cache: HashMap<DescriptorSetKey, (Arc<DescriptorSet>, usize)>, // value + size
    total_size: usize,
    max_size: usize,
}

impl SizeAwareCache {
    pub fn evict_to_fit(&mut self, required_size: usize) {
        while self.total_size + required_size > self.max_size {
            // Evict largest descriptor first
            let largest_key = self.cache
                .iter()
                .max_by_key(|(_, (_, size))| size)
                .map(|(k, _)| k.clone())
                .unwrap();
            
            let (_, size) = self.cache.remove(&largest_key).unwrap();
            self.total_size -= size;
        }
    }
}
```

### Monitoring Cache Performance

```rust
// Track cache effectiveness
#[derive(Default)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub current_size: usize,
    pub peak_size: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f32 {
        let total = self.hits + self.misses;
        if total == 0 { return 0.0; }
        self.hits as f32 / total as f32
    }

    pub fn average_occupancy(&self) -> f32 {
        if self.peak_size == 0 { return 0.0; }
        self.current_size as f32 / self.peak_size as f32
    }
}

// Log stats periodically
if frame_count % 300 == 0 {
    println!("Descriptor Cache Stats:");
    println!("  Hit Rate: {:.1}%", cache_stats.hit_rate() * 100.0);
    println!("  Evictions: {}", cache_stats.evictions);
    println!("  Occupancy: {:.1}%", cache_stats.average_occupancy() * 100.0);
}
```

**Interpretation:**

- **Hit Rate > 90%**: Cache is well-sized
- **Hit Rate 70-90%**: Consider increasing cache size
- **Hit Rate < 70%**: Cache too small or eviction policy suboptimal
- **Evictions > 100/frame**: Cache thrashing, increase size

---

## Hi-Z Occlusion Culling

### Hierarchical Z-Buffer Overview

Hi-Z occlusion culling uses a mipmap pyramid of depth values to quickly test if objects are occluded by previously rendered geometry.

### Mip Level Selection

The key to efficient Hi-Z is selecting the appropriate mip level based on object screen size:

```rust
pub fn select_hiz_mip_level(
    screen_space_bbox: [f32; 4], // [min_x, min_y, max_x, max_y] in pixels
    hiz_resolution: [u32; 2],     // [width, height] of base Hi-Z
) -> u32 {
    let bbox_width = screen_space_bbox[2] - screen_space_bbox[0];
    let bbox_height = screen_space_bbox[3] - screen_space_bbox[1];
    let bbox_max_dim = bbox_width.max(bbox_height);

    // Select mip where bbox fits in approximately 1-2 pixels
    let mip_level = (bbox_max_dim.log2()).floor() as u32;

    // Clamp to available mips
    let max_mip = (hiz_resolution[0].min(hiz_resolution[1]) as f32).log2().floor() as u32;
    mip_level.min(max_mip)
}
```

### Recommended Mip Levels

| Screen Resolution | Base Hi-Z | Mip Levels | Total Memory |
|-------------------|-----------|------------|--------------|
| 1920x1080 | 1920x1080 | 11 levels | ~8.3 MB (R32F) |
| 2560x1440 | 2560x1440 | 12 levels | ~14.8 MB (R32F) |
| 3840x2160 | 3840x2160 | 12 levels | ~33.2 MB (R32F) |

**Memory calculation:**
```
Sum of mip sizes = W*H*4 * (1 + 1/4 + 1/16 + ... + 1/4^n)
                 ≈ W*H*4 * 1.33
```

### Conservative Occlusion Testing

Always use conservative tests to avoid false culling:

```rust
pub fn is_occluded_conservative(
    object_depth: f32,
    hiz_depth: f32,
    depth_bias: f32,
) -> bool {
    // Use bias to prevent false culling due to:
    // - Floating-point precision errors
    // - Mip filtering approximations
    // - Camera movement between frames
    object_depth > hiz_depth + depth_bias
}

// Recommended biases:
// - Static camera: 0.001
// - Moving camera: 0.01
// - Fast-moving camera: 0.05
```

### Multi-Sample Testing

Test multiple points to reduce false culling:

```rust
pub fn multi_sample_occlusion_test(
    object_bbox_screen: [f32; 4],
    object_depth: f32,
    hiz_pyramid: &HiZPyramid,
) -> bool {
    // Test 5 points: 4 corners + center
    let test_points = [
        (object_bbox_screen[0], object_bbox_screen[1]), // Top-left
        (object_bbox_screen[2], object_bbox_screen[1]), // Top-right
        (object_bbox_screen[0], object_bbox_screen[3]), // Bottom-left
        (object_bbox_screen[2], object_bbox_screen[3]), // Bottom-right
        (
            (object_bbox_screen[0] + object_bbox_screen[2]) / 2.0,
            (object_bbox_screen[1] + object_bbox_screen[3]) / 2.0,
        ), // Center
    ];

    // Object is occluded only if ALL test points are occluded
    test_points.iter().all(|&(x, y)| {
        let hiz_depth = hiz_pyramid.sample(x, y, mip_level);
        is_occluded_conservative(object_depth, hiz_depth, 0.01)
    })
}
```

### Two-Pass Occlusion

Optimal occlusion culling uses two passes:

```rust
// Pass 1: Render occluders (large, opaque objects)
render_occluders_to_depth_buffer();

// Generate Hi-Z pyramid from depth buffer
generate_hiz_pyramid(depth_buffer);

// Pass 2: Test all other objects against Hi-Z
for object in scene_objects {
    if !is_occluded_conservative(object, hiz_pyramid) {
        render_object(object);
    }
}
```

**Occluder Selection Criteria:**
- Large objects (> 5% screen coverage)
- Opaque geometry only
- Low rendering cost
- Static or slowly moving

### Hi-Z Update Frequency

Update Hi-Z pyramid based on scene dynamics:

```rust
pub enum HiZUpdateStrategy {
    EveryFrame,           // Most accurate, highest cost (1-2ms overhead)
    EveryNFrames(u32),    // Balance accuracy and cost
    OnCameraMove,         // Update only when camera moves
    OnMajorOccluderChange, // Update when large objects move
}

// Recommended strategies:
// - Fast-paced games: EveryFrame or EveryNFrames(2)
// - Third-person: EveryNFrames(3-5)
// - Strategy/RTS: OnCameraMove
// - Static scenes: OnMajorOccluderChange
```

### Performance Characteristics

| Scene Type | Occluders | Culled Objects | Frame Time Savings | Hi-Z Cost |
|------------|-----------|----------------|-------------------|-----------|
| **Outdoor Open** | Few | 10-20% | 0.5-1ms | 1-1.5ms |
| **Outdoor Dense Forest** | Many | 30-50% | 2-4ms | 1-2ms |
| **Indoor Corridors** | Many | 40-60% | 3-6ms | 1-1.5ms |
| **Indoor Large Rooms** | Moderate | 20-40% | 1-3ms | 1-2ms |
| **Urban City** | Many | 50-70% | 5-10ms | 1.5-2ms |

**Rule of Thumb:** Hi-Z is profitable when:
```
(culled_objects * avg_render_cost) > hiz_generation_cost
```

---

## GPU Culling Optimization

### Workgroup Size Tuning

The compute shader processes objects in parallel workgroups:

```glsl
layout(local_size_x = WORKGROUP_SIZE) in;
```

**Hardware-Specific Recommendations:**

| GPU Architecture | Optimal Workgroup Size | Reasoning |
|------------------|------------------------|-----------|
| **NVIDIA (Ampere/Turing)** | 32 or 64 | 32 threads per warp |
| **AMD (RDNA2/RDNA3)** | 64 | 64 threads per wavefront |
| **Intel (Xe)** | 16 or 32 | Variable SIMD width |
| **Mobile (ARM Mali)** | 16 | 16 threads per warp |
| **Mobile (Adreno)** | 64 | 64-128 threads per threadgroup |

```rust
// Configure for target hardware
pub fn create_culling_config_for_hardware(gpu_info: &GpuInfo) -> CullingConfig {
    let workgroup_size = match gpu_info.vendor {
        GpuVendor::Nvidia => 64,
        GpuVendor::Amd => 64,
        GpuVendor::Intel => 32,
        GpuVendor::ArmMali => 16,
        GpuVendor::Qualcomm => 64,
        _ => 64, // Safe default
    };

    CullingConfig {
        workgroup_size,
        ..Default::default()
    }
}
```

### Dispatch Sizing

Calculate optimal workgroup dispatch count:

```rust
pub fn calculate_dispatch_size(
    object_count: u32,
    workgroup_size: u32,
) -> [u32; 3] {
    // Round up to nearest multiple of workgroup size
    let workgroup_count = object_count.div_ceil(workgroup_size);
    
    // Some GPUs prefer certain dispatch dimensions
    // Generally X dimension is fastest
    [workgroup_count, 1, 1]
}

// For very large object counts, can split into 2D or 3D dispatch
pub fn calculate_dispatch_size_2d(
    object_count: u32,
    workgroup_size: u32,
) -> [u32; 3] {
    let total_workgroups = object_count.div_ceil(workgroup_size);
    
    // Split into 2D for better cache behavior
    let workgroups_x = (total_workgroups as f32).sqrt().ceil() as u32;
    let workgroups_y = total_workgroups.div_ceil(workgroups_x);
    
    [workgroups_x, workgroups_y, 1]
}
```

### Early-Out Optimization

Order culling tests from fastest to slowest:

```glsl
void main() {
    uint id = gl_GlobalInvocationID.x;
    if (id >= object_count) return; // Bounds check

    ObjectData obj = objects[id];
    
    // 1. Distance culling (cheapest: 3 subtractions + dot product)
    vec3 to_camera = camera_position - obj.position;
    float dist_sq = dot(to_camera, to_camera);
    if (dist_sq > max_distance_sq) {
        return; // Early out
    }
    
    // 2. Frustum culling (moderate: 6 plane tests)
    if (frustum_cull(obj.bounding_sphere)) {
        return; // Early out
    }
    
    // 3. Occlusion culling (expensive: texture samples + depth compare)
    if (occlusion_enabled && occlusion_cull(obj)) {
        return; // Early out
    }
    
    // Object is visible - write to output
    uint index = atomicAdd(visible_count, 1);
    visible_objects[index] = id;
}
```

### Memory Access Patterns

Optimize buffer layouts for cache coherency:

```rust
// Bad: Structure of Arrays (poor cache usage for culling)
struct SceneDataSoA {
    positions: Vec<Vec3>,       // Buffer 1
    bounding_spheres: Vec<Vec4>,// Buffer 2
    mesh_ids: Vec<u32>,         // Buffer 3
}

// Good: Array of Structures (better cache locality)
#[repr(C, align(16))]
struct ObjectData {
    position: Vec3,
    bounding_radius: f32,
    bounding_center: Vec3,
    mesh_id: u32,
    // Total: 32 bytes, fits in 2 cache lines
}

struct SceneDataAoS {
    objects: Vec<ObjectData>, // Single contiguous buffer
}
```

### Atomic Contention Reduction

Reduce contention on atomic counters:

```glsl
// Shared memory for thread-local accumulation
shared uint local_visible_count;
shared uint local_visible_indices[WORKGROUP_SIZE];

void main() {
    uint local_id = gl_LocalInvocationID.x;
    uint global_id = gl_GlobalInvocationID.x;
    
    // Initialize shared memory
    if (local_id == 0) {
        local_visible_count = 0;
    }
    barrier();
    
    // Test object
    bool visible = test_visibility(objects[global_id]);
    
    // Write to local buffer
    if (visible) {
        uint local_index = atomicAdd(local_visible_count, 1);
        local_visible_indices[local_index] = global_id;
    }
    barrier();
    
    // Single thread writes all visible from workgroup to global
    if (local_id == 0 && local_visible_count > 0) {
        uint base_index = atomicAdd(global_visible_count, local_visible_count);
        for (uint i = 0; i < local_visible_count; i++) {
            visible_objects[base_index + i] = local_visible_indices[i];
        }
    }
}
```

This reduces atomic operations from `visible_objects_per_workgroup` to 1 per workgroup.

---

## Profiling Methodology

### Baseline Measurement

Establish baseline performance before optimization:

```rust
use std::time::Instant;

pub struct FrameTimings {
    pub total_frame: f32,
    pub culling: f32,
    pub lod_selection: f32,
    pub mesh_streaming: f32,
    pub rendering: f32,
}

pub fn measure_frame_timings() -> FrameTimings {
    let frame_start = Instant::now();
    
    let culling_start = Instant::now();
    perform_culling();
    let culling_time = culling_start.elapsed().as_secs_f32() * 1000.0;
    
    let lod_start = Instant::now();
    update_lod_system();
    let lod_time = lod_start.elapsed().as_secs_f32() * 1000.0;
    
    let streaming_start = Instant::now();
    update_mesh_streaming();
    let streaming_time = streaming_start.elapsed().as_secs_f32() * 1000.0;
    
    let render_start = Instant::now();
    render_scene();
    let render_time = render_start.elapsed().as_secs_f32() * 1000.0;
    
    let total_time = frame_start.elapsed().as_secs_f32() * 1000.0;
    
    FrameTimings {
        total_frame: total_time,
        culling: culling_time,
        lod_selection: lod_time,
        mesh_streaming: streaming_time,
        rendering: render_time,
    }
}
```

### Statistical Analysis

Collect data over multiple frames for meaningful results:

```rust
pub struct PerformanceStats {
    samples: Vec<f32>,
    capacity: usize,
}

impl PerformanceStats {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: Vec::with_capacity(capacity),
            capacity,
        }
    }
    
    pub fn record(&mut self, value: f32) {
        if self.samples.len() >= self.capacity {
            self.samples.remove(0);
        }
        self.samples.push(value);
    }
    
    pub fn mean(&self) -> f32 {
        self.samples.iter().sum::<f32>() / self.samples.len() as f32
    }
    
    pub fn median(&self) -> f32 {
        let mut sorted = self.samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted[sorted.len() / 2]
    }
    
    pub fn percentile_95(&self) -> f32 {
        let mut sorted = self.samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted[(sorted.len() as f32 * 0.95) as usize]
    }
    
    pub fn percentile_99(&self) -> f32 {
        let mut sorted = self.samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted[(sorted.len() as f32 * 0.99) as usize]
    }
    
    pub fn min(&self) -> f32 {
        self.samples.iter().cloned().fold(f32::INFINITY, f32::min)
    }
    
    pub fn max(&self) -> f32 {
        self.samples.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
    }
}

// Usage: Track 300 frames (5 seconds at 60 FPS)
let mut frame_times = PerformanceStats::new(300);
for _ in 0..300 {
    let timings = measure_frame_timings();
    frame_times.record(timings.total_frame);
}

println!("Frame Time Statistics:");
println!("  Mean: {:.2} ms", frame_times.mean());
println!("  Median: {:.2} ms", frame_times.median());
println!("  95th percentile: {:.2} ms", frame_times.percentile_95());
println!("  99th percentile: {:.2} ms", frame_times.percentile_99());
println!("  Min: {:.2} ms, Max: {:.2} ms", frame_times.min(), frame_times.max());
```

### A/B Testing Methodology

Compare two configurations scientifically:

```rust
pub fn compare_configurations(
    config_a: OptimizationConfig,
    config_b: OptimizationConfig,
    test_duration_frames: usize,
) -> ComparisonResult {
    // Test configuration A
    apply_config(&config_a);
    let stats_a = collect_stats(test_duration_frames);
    
    // Wait for stabilization
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    // Test configuration B
    apply_config(&config_b);
    let stats_b = collect_stats(test_duration_frames);
    
    ComparisonResult {
        config_a_mean: stats_a.mean(),
        config_b_mean: stats_b.mean(),
        improvement: (stats_a.mean() - stats_b.mean()) / stats_a.mean() * 100.0,
        statistical_significance: t_test(&stats_a.samples, &stats_b.samples),
    }
}

// Results interpretation:
// - improvement > 5%: Significant improvement
// - improvement 2-5%: Moderate improvement
// - improvement 0-2%: Negligible difference
// - improvement < 0%: Configuration B is slower
```

### GPU Profiling Integration

Use vendor-specific tools for detailed GPU metrics:

**NVIDIA (NSight Graphics):**
```bash
# Capture frame with NSight
nsight-sys profile --trace=vulkan cargo run --release --example performance_test
```

**AMD (Radeon GPU Profiler):**
```bash
# Enable profiling markers in code
vkCmdDebugMarkerBeginEXT(command_buffer, &marker_info);
perform_culling();
vkCmdDebugMarkerEndEXT(command_buffer);
```

**RenderDoc (Cross-platform):**
- Capture frames with `Ctrl+F12`
- Analyze pipeline stages, draw calls, GPU time
- Inspect buffer contents and shader execution

### Automated Performance Testing

Create regression tests for performance:

```rust
#[test]
fn performance_regression_test_10k_objects() {
    let config = OptimizationConfig::default();
    let scene = create_test_scene_10k_objects();
    
    let mut stats = PerformanceStats::new(100);
    for _ in 0..100 {
        let timings = measure_frame_timings();
        stats.record(timings.total_frame);
    }
    
    // Assert performance targets
    assert!(stats.mean() < 16.67, "Average frame time exceeds 60 FPS target");
    assert!(stats.percentile_95() < 20.0, "95th percentile exceeds 50 FPS");
    assert!(stats.percentile_99() < 33.33, "99th percentile exceeds 30 FPS");
}
```

---

## Hardware Tier Configurations

### Low-End Hardware (Integrated Graphics)

**Target:** Intel UHD 630, AMD Vega 8, entry-level laptops

```rust
pub fn create_low_end_config() -> OptimizationConfig {
    OptimizationConfig {
        // LOD: Aggressive distance thresholds
        lod_distances: vec![
            (0, 12.0),   // LOD0: Very close only
            (1, 35.0),   // LOD1: Quick transition to medium
            (2, 80.0),   // LOD2: Low detail at moderate distance
        ],
        lod_global_bias: -0.3, // Prefer lower detail
        
        // Culling: CPU-based for small scenes
        use_gpu_culling: false,
        frustum_culling: true,
        occlusion_culling: false, // Skip Hi-Z overhead
        max_render_distance: 100.0,
        
        // Mesh Streaming: Conservative budget
        mesh_streaming_enabled: true,
        max_meshes_per_frame: 2,
        max_streaming_memory_mb: 128,
        preload_distance_multiplier: 1.1,
        
        // Descriptor Cache: Small cache
        descriptor_cache_size: 128,
        descriptor_eviction_policy: EvictionPolicy::Lru,
        
        // Rendering: Reduce quality
        shadow_map_resolution: 1024,
        max_lights: 4,
        max_draw_calls_per_frame: 500,
    }
}
```

**Expected Performance:**
- 30-60 FPS in moderate scenes (1000-3000 objects)
- Frame time budget: 16-33 ms
- Memory footprint: < 512 MB

### Mid-Range Hardware (Dedicated GPU)

**Target:** GTX 1660, RTX 3050, RX 6600, mid-range desktops/laptops

```rust
pub fn create_mid_range_config() -> OptimizationConfig {
    OptimizationConfig {
        // LOD: Balanced thresholds
        lod_distances: vec![
            (0, 20.0),   // LOD0: Good high detail range
            (1, 60.0),   // LOD1: Moderate transition
            (2, 150.0),  // LOD2: Low detail at distance
        ],
        lod_global_bias: 0.0, // Neutral
        
        // Culling: Hybrid approach
        use_gpu_culling: true,
        gpu_culling_threshold: 5000, // Switch to GPU at 5k objects
        frustum_culling: true,
        occlusion_culling: true, // Enable Hi-Z
        hiz_update_frequency: HiZUpdateFrequency::EveryNFrames(3),
        max_render_distance: 200.0,
        
        // Mesh Streaming: Moderate budget
        mesh_streaming_enabled: true,
        max_meshes_per_frame: 3,
        max_streaming_memory_mb: 384,
        preload_distance_multiplier: 1.3,
        
        // Descriptor Cache: Medium cache
        descriptor_cache_size: 256,
        descriptor_eviction_policy: EvictionPolicy::Lru,
        
        // Rendering: Good quality
        shadow_map_resolution: 2048,
        max_lights: 8,
        max_draw_calls_per_frame: 2000,
    }
}
```

**Expected Performance:**
- 60-90 FPS in moderate scenes (5000-10000 objects)
- Frame time budget: 11-16 ms
- Memory footprint: 512-1024 MB

### High-End Hardware (Enthusiast GPU)

**Target:** RTX 3080/4080, RX 6800 XT/7800 XT, high-end desktops

```rust
pub fn create_high_end_config() -> OptimizationConfig {
    OptimizationConfig {
        // LOD: Extended high detail range
        lod_distances: vec![
            (0, 35.0),   // LOD0: Extended high detail
            (1, 100.0),  // LOD1: Farther medium range
            (2, 250.0),  // LOD2: Low detail only very far
        ],
        lod_global_bias: 0.2, // Prefer higher detail
        
        // Culling: GPU-driven for large scenes
        use_gpu_culling: true,
        gpu_culling_threshold: 2000, // Use GPU more aggressively
        frustum_culling: true,
        occlusion_culling: true,
        hiz_update_frequency: HiZUpdateFrequency::EveryFrame,
        max_render_distance: 400.0,
        
        // Mesh Streaming: High budget
        mesh_streaming_enabled: true,
        max_meshes_per_frame: 5,
        max_streaming_memory_mb: 1024,
        preload_distance_multiplier: 1.5,
        
        // Descriptor Cache: Large cache
        descriptor_cache_size: 512,
        descriptor_eviction_policy: EvictionPolicy::Lru,
        
        // Rendering: Maximum quality
        shadow_map_resolution: 4096,
        max_lights: 16,
        max_draw_calls_per_frame: 5000,
    }
}
```

**Expected Performance:**
- 90-144+ FPS in large scenes (10000-50000 objects)
- Frame time budget: 7-11 ms
- Memory footprint: 1-2 GB

### Ultra Hardware (Flagship GPU)

**Target:** RTX 4090, RX 7900 XTX, workstation GPUs

```rust
pub fn create_ultra_config() -> OptimizationConfig {
    OptimizationConfig {
        // LOD: Maximum quality
        lod_distances: vec![
            (0, 50.0),   // LOD0: Very extended high detail
            (1, 150.0),  // LOD1: Far medium range
            (2, 400.0),  // LOD2: Low detail at extreme distance
        ],
        lod_global_bias: 0.5, // Strongly prefer higher detail
        
        // Culling: Full GPU optimization
        use_gpu_culling: true,
        gpu_culling_threshold: 1000,
        frustum_culling: true,
        occlusion_culling: true,
        hiz_update_frequency: HiZUpdateFrequency::EveryFrame,
        max_render_distance: 600.0,
        
        // Mesh Streaming: Maximum budget
        mesh_streaming_enabled: true,
        max_meshes_per_frame: 8,
        max_streaming_memory_mb: 2048,
        preload_distance_multiplier: 2.0,
        
        // Descriptor Cache: Very large cache
        descriptor_cache_size: 1024,
        descriptor_eviction_policy: EvictionPolicy::Lfu, // Keep frequent descriptors
        
        // Rendering: Ultra quality
        shadow_map_resolution: 8192,
        max_lights: 32,
        max_draw_calls_per_frame: 10000,
    }
}
```

**Expected Performance:**
- 144-240+ FPS in massive scenes (50000+ objects)
- Frame time budget: 4-7 ms
- Memory footprint: 2-4 GB

---

## Trade-off Analysis

### LOD Distance vs Visual Quality

| Configuration | Visual Quality | Performance Gain | Pop-in Visibility |
|---------------|----------------|------------------|-------------------|
| **Aggressive (Low-End)** | Lower | +40-60% FPS | Very noticeable |
| **Balanced (Default)** | Good | +20-30% FPS | Slightly noticeable |
| **Quality (High-End)** | High | +10-15% FPS | Barely noticeable |

**Recommendation:** Start with Balanced, adjust based on profiling results.

### GPU Culling vs CPU Culling

| Factor | CPU Culling | GPU Culling |
|--------|-------------|-------------|
| **Best for** | < 5000 objects | >= 5000 objects |
| **Latency** | Immediate | 1-2 frame delay |
| **CPU Load** | 0.5-5ms/frame | < 0.1ms/frame |
| **GPU Load** | None | 0.3-1ms/frame |
| **Scalability** | Linear O(n) | Sub-linear O(n/cores) |

**Recommendation:** Use hybrid approach with automatic switching at 5000 object threshold.

### Occlusion Culling Cost vs Benefit

| Scene Type | Occlusion Benefit | Hi-Z Cost | Net Gain | Recommended? |
|------------|-------------------|-----------|----------|--------------|
| **Open Outdoor** | Low (10-20%) | 1-2ms | -0.5 to +1ms | ❌ No |
| **Dense Forest** | High (40-60%) | 1.5ms | +3-5ms | ✅ Yes |
| **Indoor Corridors** | Very High (60-80%) | 1ms | +5-10ms | ✅ Yes |
| **Urban City** | High (50-70%) | 2ms | +5-8ms | ✅ Yes |

**Recommendation:** Enable for indoor and urban scenes, disable for open outdoor environments.

### Mesh Streaming Memory vs Load Times

| Memory Budget | Load Latency | Visible Pop-in | Preload Effectiveness |
|---------------|--------------|----------------|----------------------|
| **128 MB (Low)** | High (2-5 frames) | Frequent | Limited |
| **384 MB (Med)** | Medium (1-2 frames) | Occasional | Good |
| **1024 MB (High)** | Low (< 1 frame) | Rare | Excellent |

**Recommendation:** 384 MB for mid-range, increase if pop-in is visible.

### Descriptor Cache Size vs Hit Rate

| Cache Size | Typical Hit Rate | Memory Cost | Recommended For |
|------------|------------------|-------------|-----------------|
| **64 entries** | 60-70% | ~4 KB | Very simple scenes |
| **128 entries** | 75-85% | ~8 KB | Simple scenes |
| **256 entries** | 85-92% | ~16 KB | Moderate complexity |
| **512 entries** | 92-96% | ~32 KB | Complex scenes |
| **1024 entries** | 95-98% | ~64 KB | Very complex scenes |

**Recommendation:** Start with 256, monitor hit rate, increase if < 85%.

---

## Conclusion

Performance optimization is an iterative process:

1. **Measure baseline** with default settings
2. **Profile** to identify bottlenecks
3. **Apply targeted optimizations** based on hardware tier
4. **Measure improvements** with statistical rigor
5. **Iterate** on most impactful areas

### Quick Reference Checklist

- [ ] LOD distances configured for target hardware
- [ ] GPU culling enabled for scenes with 5000+ objects
- [ ] Hi-Z occlusion enabled for indoor/urban environments
- [ ] Mesh streaming budget set appropriately
- [ ] Descriptor cache sized for scene complexity
- [ ] Performance stats collected over 300+ frames
- [ ] A/B testing performed for configuration changes
- [ ] Frame time targets met for all hardware tiers

### Further Resources

- [LOD System Guide](lod.md)
- [GPU Culling Guide](gpu-culling.md)
- [Hi-Z Implementation Summary](hiz-implementation-summary.md)
- [Profiling Guide](../profiling/)
- [Render Stats System](../../../crates/praxis_graphics/src/render_stats.rs)
