# Performance Optimization Learning Path

Master performance analysis and optimization techniques across all engine systems.

## Path Overview

**Time Investment**: 1-2 weeks  
**Prerequisites**: Completed at least one other path  
**Final Goal**: 60+ FPS in production scenarios

## Progression Map

```
Beginner (4-5 days)
├── Profiling basics
├── Identifying bottlenecks
├── Common optimizations
└── Measurement techniques
    ↓
Intermediate (4-5 days)
├── Rendering optimization
├── ECS optimization
├── Physics optimization
└── Memory optimization
    ↓
Advanced (4-5 days)
├── GPU profiling
├── Multi-threading
├── LOD systems
└── Production deployment
```

---

## Beginner: Profiling Basics

**Theory** (2-3 hours):
1. Read [Profiling Guide](../profiling.md)
2. Understand performance metrics
3. Learn measurement tools

**Practice** (4-6 hours):
1. Run profiling examples
2. Identify bottlenecks
3. Measure baselines

**Basic Profiling**:
```rust
use praxis_profiling::{Profiler, profile_scope};

// Enable profiling
Profiler::enable();

// Profile function
fn expensive_function() {
    profile_scope!("expensive_function");
    // ... work
}

// Get results
let stats = Profiler::get_stats();
for (name, duration) in stats {
    println!("{}: {:.2}ms", name, duration);
}
```

**Run Examples**:
```bash
cargo run --example profiling_demo
cargo run --example profiling_advanced_demo
```

**Key Metrics**:
- Frame time (target: < 16.67ms for 60 FPS)
- Draw calls (target: < 1000)
- Memory usage
- CPU/GPU time split

### Checkpoint
- [ ] Can profile code
- [ ] Identify slow systems
- [ ] Understand metrics

**Time**: 6-10 hours

---

## Intermediate: System-Specific Optimization

### Rendering Optimization (8-10 hours)

**Techniques**:
1. Read [Spatial Optimization](../guides/spatial-optimization.md)
2. Implement frustum culling
3. Add LOD system
4. Batch draw calls

**Frustum Culling**:
```rust
fn frustum_culling_system(
    query: Query<(Entity, &GlobalTransform, &BoundingBox)>,
    camera: Query<&FrustumPlanes, With<Camera>>,
    mut visible: ResMut<VisibleEntities>,
) {
    visible.clear();
    let frustum = camera.single();
    
    for (entity, transform, bounds) in query.iter() {
        if frustum.intersects_aabb(bounds, transform) {
            visible.push(entity);
        }
    }
}
```

**LOD System**:
```rust
#[derive(Component)]
struct LodMesh {
    high: String,    // < 10m
    medium: String,  // 10-50m
    low: String,     // 50-100m
}

fn lod_system(
    query: Query<(&GlobalTransform, &LodMesh, &mut MeshHandle)>,
    camera: Query<&Transform, With<Camera>>,
) {
    let camera_pos = camera.single().translation;
    
    for (transform, lod, mut mesh) in query.iter_mut() {
        let distance = transform.translation().distance(camera_pos);
        mesh.id = if distance < 10.0 {
            lod.high.clone()
        } else if distance < 50.0 {
            lod.medium.clone()
        } else {
            lod.low.clone()
        };
    }
}
```

**Run Example**:
```bash
cargo run --example lod_demo
cargo run --example spatial_optimization_demo
```

### ECS Optimization (4-6 hours)

**Techniques**:
- Query optimization
- Component organization
- System ordering
- Change detection

**Optimized Queries**:
```rust
// BAD: Iterates all entities every frame
fn bad_system(query: Query<&Transform>) {
    for transform in query.iter() {
        // Process everything
    }
}

// GOOD: Only process changed entities
fn good_system(query: Query<&Transform, Changed<Transform>>) {
    for transform in query.iter() {
        // Only process changed
    }
}

// GOOD: Cache results
#[derive(Resource)]
struct CachedData {
    entities: Vec<Entity>,
    last_update: f32,
}

fn cached_system(
    query: Query<Entity, With<Enemy>>,
    mut cache: ResMut<CachedData>,
    time: Res<Time>,
) {
    // Update cache every 0.5s instead of every frame
    if time.elapsed() - cache.last_update > 0.5 {
        cache.entities = query.iter().collect();
        cache.last_update = time.elapsed();
    }
    
    // Use cached data
    for entity in &cache.entities {
        // Process
    }
}
```

### Physics Optimization (4-6 hours)

**Techniques**:
- Simplify colliders
- Collision groups
- Sleeping bodies
- Solver tuning

**Collision Groups**:
```rust
// Reduce collision checks with groups
const PLAYER: u32 = 0b0001;
const ENEMY: u32 = 0b0010;
const PROJECTILE: u32 = 0b0100;
const STATIC: u32 = 0b1000;

// Projectiles don't collide with each other
world.spawn((
    Collider::sphere(0.1),
    CollisionGroups::new(
        PROJECTILE,
        ENEMY | STATIC,  // Only these
    ),
));
```

**Simplified Colliders**:
```rust
// BAD: Complex mesh collider for simple object
Collider::trimesh(vertices, indices)

// GOOD: Simple primitive
Collider::sphere(0.5)  // Much faster

// GOOD: Compound for complex shapes
Collider::compound(vec![
    Collider::cuboid(1.0, 1.0, 1.0),
    Collider::sphere(0.5),
])
```

### Memory Optimization (4-6 hours)

**Techniques**:
- Object pooling
- Memory profiling
- Reduce allocations
- Cache-friendly data

**Object Pooling**:
```rust
#[derive(Resource)]
struct ProjectilePool {
    inactive: Vec<Entity>,
}

impl ProjectilePool {
    fn spawn(&mut self, world: &mut World, position: Vec3) -> Entity {
        if let Some(entity) = self.inactive.pop() {
            // Reuse existing entity
            let mut transform = world.get_mut::<Transform>(entity).unwrap();
            transform.translation = position;
            entity
        } else {
            // Create new
            world.spawn((
                Transform::from_translation(position),
                Projectile,
            )).id()
        }
    }
    
    fn despawn(&mut self, entity: Entity) {
        self.inactive.push(entity);
    }
}
```

### Checkpoint
- [ ] Rendering optimized (culling, LOD)
- [ ] ECS queries efficient
- [ ] Physics simplified
- [ ] Memory usage reduced

**Time**: 20-25 hours

---

## Advanced: Production Optimization

### GPU Profiling (4-6 hours)

**Tools**:
- RenderDoc
- Nvidia Nsight
- AMD Radeon GPU Profiler

**Techniques**:
```rust
// Measure GPU time
let query = device.create_timestamp_query()?;

// Record work
query.begin();
render_context.render(&commands)?;
query.end();

// Get results
let gpu_time_ms = query.get_results()?;
```

### Multi-Threading (6-8 hours)

**Parallel ECS**:
```rust
// Process entities in parallel
fn parallel_update_system(
    query: Query<&mut Transform>,
) {
    query.par_iter_mut().for_each(|mut transform| {
        // Parallel processing
        transform.translation += Vec3::Y * 0.1;
    });
}
```

### LOD Systems (4-6 hours)

**Mesh LOD**: Distance-based mesh swapping  
**Animation LOD**: Reduce update rate for distant characters  
**Audio LOD**: Limit simultaneous sounds  
**Physics LOD**: Simplify colliders at distance

### Production Deployment (4-6 hours)

**Release Optimizations**:
```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
```

**Performance Targets**:
- 60 FPS minimum (16.67ms per frame)
- < 2GB memory usage
- < 100ms load times
- < 1000 draw calls per frame

### Checkpoint
- [ ] GPU profiled and optimized
- [ ] Multi-threading utilized
- [ ] Complete LOD system
- [ ] Production-ready build

**Time**: 20-25 hours

---

## Optimization Checklist

### Rendering
- [ ] Frustum culling enabled
- [ ] Occlusion culling (if needed)
- [ ] Mesh LOD system
- [ ] Texture compression
- [ ] Draw call batching
- [ ] Shader optimization

### ECS
- [ ] Changed<T> for infrequent updates
- [ ] Cached queries where appropriate
- [ ] Efficient component layout
- [ ] System ordering optimized

### Physics
- [ ] Simple colliders used
- [ ] Collision groups configured
- [ ] Sleeping bodies enabled
- [ ] Solver iterations tuned

### Memory
- [ ] Object pooling for frequent spawns
- [ ] Asset streaming
- [ ] Memory profiling done
- [ ] Leaks identified and fixed

### Other
- [ ] Animation LOD
- [ ] Audio LOD
- [ ] Profiling in release mode
- [ ] Tested on target hardware

---

## Cross-References

- All other learning paths (apply optimizations)
- [Profiling Guide](../profiling.md)
- [Spatial Optimization](../guides/spatial-optimization.md)
- [LOD System](../lod-system.md)

---

## Resources

```bash
# Profile examples
cargo run --release --example profiling_advanced_demo
cargo run --release --example spatial_optimization_demo
cargo run --release --example lod_demo
```

**External Tools**:
- RenderDoc (GPU profiling)
- cargo-flamegraph (CPU profiling)
- valgrind/massif (memory profiling)

---

[← Back to Learning Paths](README.md)
