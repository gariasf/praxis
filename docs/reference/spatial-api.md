# Spatial Optimization API Reference

API reference for spatial data structures, culling, and LOD systems.

## Spatial Partitioning

### SpatialManager

Unified manager for spatial queries and updates.

```rust
pub struct SpatialManager { /* ... */ }
```

**Methods:**
- `new_octree(config: SpatialConfig) -> Self`
- `new_bvh(config: SpatialConfig) -> Self`
- `insert(entity: Entity, bounds: Aabb) -> Result<()>`
- `update(entity: Entity, new_bounds: Aabb) -> Result<()>`
- `remove(entity: Entity) -> Result<()>`
- `flush_updates()` - Apply pending updates
- `query_point(point: Vec3) -> Vec<Entity>`
- `query_aabb(aabb: Aabb) -> Vec<Entity>`
- `query_sphere(center: Vec3, radius: f32) -> Vec<Entity>`
- `query_radius(center: Vec3, radius: f32) -> Vec<Entity>`
- `query_ray(origin: Vec3, direction: Vec3, max_distance: f32) -> Vec<Entity>`
- `query_ray_sorted(origin: Vec3, direction: Vec3, max_distance: f32) -> Vec<(Entity, f32)>`
- `clear()`
- `entity_count() -> usize`

### SpatialConfig

Configuration for spatial partitioning.

```rust
pub struct SpatialConfig {
    pub center: Vec3,                    // World center
    pub size: f32,                       // World size
    pub max_entities_per_node: usize,    // Split threshold
    pub max_depth: usize,                // Maximum tree depth
    pub movement_threshold: f32,         // Min movement to trigger update
}
```

**Methods:**
- `default()` - Standard configuration
- `with_size(size: f32)` - Set world size
- `with_max_entities(count: usize)` - Set split threshold

### Aabb

Axis-aligned bounding box.

```rust
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}
```

**Methods:**
- `new(min: Vec3, max: Vec3) -> Self`
- `from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self`
- `from_transform(transform: &Transform, mesh_bounds: &Aabb) -> Self`
- `center() -> Vec3`
- `half_extents() -> Vec3`
- `size() -> Vec3`
- `volume() -> f32`
- `contains_point(point: Vec3) -> bool`
- `intersects(other: &Aabb) -> bool`
- `merge(other: &Aabb) -> Aabb`
- `expand(amount: f32) -> Aabb`

### BoundingSphere

Bounding sphere for simpler culling tests.

```rust
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: f32,
}
```

**Methods:**
- `new(center: Vec3, radius: f32) -> Self`
- `from_aabb(aabb: &Aabb) -> Self`
- `from_transform(transform: &Transform, radius: f32) -> Self`
- `contains_point(point: Vec3) -> bool`
- `intersects(other: &BoundingSphere) -> bool`

## Frustum Culling

### FrustumCuller

Efficient view frustum culling.

```rust
pub struct FrustumCuller { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `update(view_proj: Mat4)` - Update frustum from VP matrix
- `is_visible(bounds: &Aabb) -> bool`
- `is_sphere_visible(sphere: &BoundingSphere) -> bool`
- `cull_entities(entities: &[(Entity, Aabb)]) -> Vec<Entity>`

### Frustum

View frustum representation.

```rust
pub struct Frustum {
    pub planes: [Plane; 6],  // Left, Right, Bottom, Top, Near, Far
}
```

**Methods:**
- `from_view_projection(view_proj: Mat4) -> Self`
- `contains_point(point: Vec3) -> bool`
- `intersects_aabb(aabb: &Aabb) -> bool`
- `intersects_sphere(sphere: &BoundingSphere) -> bool`

## LOD System

### LodManager

Distance-based level of detail management.

```rust
pub struct LodManager { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `register_lod_group(group: LodGroup)`
- `unregister_lod_group(name: &str)`
- `assign_entity(entity: Entity, group_name: &str)`
- `unassign_entity(entity: Entity)`
- `select_lod(entity: Entity, camera_pos: Vec3, entity_pos: Vec3) -> Option<usize>`
- `update_all(camera_pos: Vec3, query: &Query<(Entity, &Transform)>)`

### LodGroup

Defines LOD levels for a mesh type.

```rust
pub struct LodGroup {
    pub name: String,
    pub levels: Vec<LodLevel>,
}
```

**Methods:**
- `new(name: &str, levels: Vec<LodLevel>) -> Self`
- `add_level(level: LodLevel)`
- `select_level(distance: f32) -> usize`

### LodLevel

Individual LOD level.

```rust
pub struct LodLevel {
    pub min_distance: f32,
    pub mesh_name: String,
}
```

**Methods:**
- `new(min_distance: f32, mesh_name: &str) -> Self`

### LodComponent

Component marking entity as using LOD.

```rust
#[derive(Component)]
pub struct LodComponent {
    pub group_name: String,
    pub current_level: usize,
}
```

## Occlusion Culling

### OcclusionCuller

GPU-based occlusion queries (Vulkan).

```rust
pub struct OcclusionCuller { /* ... */ }
```

**Methods:**
- `new(device: Arc<Device>, queue: Arc<Queue>) -> Result<Self>`
- `begin_frame()`
- `issue_query(entity: Entity, bounds: &Aabb) -> QueryId`
- `get_result(query_id: QueryId) -> Option<bool>` - true if visible
- `end_frame()`

### OcclusionQuery

Individual occlusion query.

```rust
pub struct OcclusionQuery {
    pub entity: Entity,
    pub query_id: QueryId,
    pub pending: bool,
}
```

## Components

### SpatialComponent

Marks entity as part of spatial partitioning.

```rust
#[derive(Component)]
pub struct SpatialComponent {
    pub bounds: Aabb,
    pub dirty: bool,
}
```

### Visible

Marker component for entities passing frustum culling.

```rust
#[derive(Component)]
pub struct Visible;
```

## Systems

### update_spatial_partitioning_system

Updates spatial structure for moved entities.

```rust
fn update_spatial_partitioning_system(
    mut manager: ResMut<SpatialManager>,
    query: Query<(Entity, &Transform, &SpatialComponent), Changed<Transform>>,
)
```

### frustum_culling_system

Culls entities outside view frustum.

```rust
fn frustum_culling_system(
    culler: Res<FrustumCuller>,
    camera: Query<&CameraMatrices, With<Camera>>,
    mut query: Query<(Entity, &SpatialComponent, Option<&mut Visible>)>,
)
```

### lod_selection_system

Selects appropriate LOD level per entity.

```rust
fn lod_selection_system(
    manager: Res<LodManager>,
    camera: Query<&Transform, With<Camera>>,
    mut query: Query<(Entity, &Transform, &mut LodComponent)>,
)
```

## Common Patterns

### Basic Spatial Setup

```rust
use praxis_spatial::{SpatialManager, SpatialConfig, Aabb};

let config = SpatialConfig {
    center: Vec3::ZERO,
    size: 1000.0,
    max_entities_per_node: 8,
    max_depth: 8,
    movement_threshold: 0.5,
};

let mut manager = SpatialManager::new_octree(config);
world.insert_resource(manager);
```

### Adding Entities to Spatial Structure

```rust
// Spawn entity with bounds
let bounds = Aabb::from_center_half_extents(
    Vec3::ZERO,
    Vec3::new(1.0, 1.0, 1.0),
);

world.spawn((
    Transform::default(),
    SpatialComponent { bounds, dirty: false },
));
```

### Spatial Queries

```rust
fn find_nearby_system(
    manager: Res<SpatialManager>,
    player: Query<&Transform, With<Player>>,
) {
    if let Ok(player_transform) = player.get_single() {
        let pos = player_transform.translation;
        
        // Find all entities within 10 units
        let nearby = manager.query_radius(pos, 10.0);
        
        for entity in nearby {
            // Process nearby entity
        }
    }
}
```

### Raycast Queries

```rust
fn raycast_system(
    manager: Res<SpatialManager>,
    camera: Query<&Transform, With<Camera>>,
) {
    if let Ok(camera_transform) = camera.get_single() {
        let origin = camera_transform.translation;
        let direction = camera_transform.forward();
        
        let hits = manager.query_ray_sorted(origin, direction, 100.0);
        
        if let Some((entity, distance)) = hits.first() {
            info!("Hit entity {:?} at distance {}", entity, distance);
        }
    }
}
```

### Frustum Culling Setup

```rust
use praxis_spatial::FrustumCuller;

// Create culler
let culler = FrustumCuller::new();
world.insert_resource(culler);

// In system
fn frustum_cull_system(
    mut culler: ResMut<FrustumCuller>,
    camera: Query<&CameraMatrices>,
    mut entities: Query<(Entity, &SpatialComponent, &mut Visible)>,
) {
    if let Ok(matrices) = camera.get_single() {
        culler.update(matrices.view_projection);
        
        for (entity, spatial, mut visible) in &mut entities {
            if culler.is_visible(&spatial.bounds) {
                visible.set_if_neq(Visible);
            } else {
                commands.entity(entity).remove::<Visible>();
            }
        }
    }
}
```

### LOD Setup

```rust
use praxis_spatial::{LodManager, LodGroup, LodLevel};

let mut lod_manager = LodManager::new();

// Register LOD group for trees
lod_manager.register_lod_group(LodGroup::new(
    "tree",
    vec![
        LodLevel::new(0.0, "tree_high"),     // 0-50 units
        LodLevel::new(50.0, "tree_medium"),  // 50-100 units
        LodLevel::new(100.0, "tree_low"),    // 100+ units
    ],
));

world.insert_resource(lod_manager);

// Assign entity to LOD group
world.spawn((
    Transform::default(),
    LodComponent {
        group_name: "tree".to_string(),
        current_level: 0,
    },
));
```

### Combined Optimization

```rust
// Complete optimization pipeline
schedule.add_systems((
    update_spatial_partitioning_system,
    frustum_culling_system,
    lod_selection_system,
).chain());
```

## Performance Tips

### Spatial Partitioning

**Octree**: Best for static/slow-moving objects, uniform distribution
```rust
let manager = SpatialManager::new_octree(config);
```

**BVH**: Best for dynamic objects, non-uniform distribution
```rust
let manager = SpatialManager::new_bvh(config);
```

### Update Frequency

```rust
// Only update when entities move significantly
let config = SpatialConfig {
    movement_threshold: 0.5,  // Only rebuild if moved >0.5 units
    ..Default::default()
};
```

### Query Optimization

```rust
// Batch queries when possible
let entities_to_query: Vec<Entity> = /* ... */;
let results = manager.query_multiple(&entities_to_query);

// Use appropriate query type
manager.query_point(pos);      // Exact point (fastest)
manager.query_sphere(pos, r);   // Sphere (fast)
manager.query_aabb(aabb);       // AABB (medium)
manager.query_ray(o, d, max);   // Ray (slowest)
```

### LOD Configuration

```rust
// More LOD levels = smoother transitions but more complexity
LodGroup::new("character", vec![
    LodLevel::new(0.0, "char_ultra"),
    LodLevel::new(20.0, "char_high"),
    LodLevel::new(40.0, "char_medium"),
    LodLevel::new(80.0, "char_low"),
    LodLevel::new(150.0, "char_impostor"),
]);
```

## See Also

- [Spatial Optimization Guide](../guides/spatial-optimization.md) - Comprehensive guide
- [praxis_spatial crate](../../crates/praxis_spatial/README.md) - Crate documentation
