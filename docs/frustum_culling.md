# Frustum Culling System

The Praxis engine provides a complete frustum culling system that automatically removes objects outside the camera's view from rendering, significantly improving performance.

## Overview

Frustum culling works by testing each entity's bounding volume against the camera's view frustum (the 3D volume visible to the camera). Only entities that intersect the frustum are marked as visible and rendered.

## Architecture

The frustum culling system consists of several components:

1. **ECS Components**:
   - `BoundingBox`: Defines an entity's spatial extent (AABB)
   - `Visible`: Marker component for entities inside the frustum
   - `Culled`: Marker component for entities outside the frustum

2. **ECS Systems**:
   - `update_frustum_from_camera`: Extracts frustum from active camera
   - `frustum_culling_system`: Tests entities against frustum and updates visibility

3. **Spatial Module**:
   - `Frustum`: 6-plane representation of camera frustum
   - `Aabb`: Axis-aligned bounding box with intersection tests

4. **ECS Culling Helpers** (`praxis_ecs::culling`):
   - `count_visible_entities()`: Returns count of visible entities
   - `is_entity_visible()`: Checks if a specific entity is visible

## Setup

### 1. Add Systems to Schedule

```rust
use praxis_ecs::{Schedule, IntoSystemConfigs};
use praxis_ecs::systems::{
    propagate_transforms,
    update_perspective_cameras,
    update_frustum_from_camera,
    frustum_culling_system,
};

let mut schedule = Schedule::default();

schedule.add_systems((
    // Transform systems must run first to compute GlobalTransform
    propagate_transforms,
    // Camera system computes view-projection matrix
    update_perspective_cameras,
    // Extract frustum from camera
    update_frustum_from_camera,
    // Perform frustum culling
    frustum_culling_system,
).chain());
```

### 2. Initialize Resources

```rust
use praxis_ecs::{World, systems::CameraFrustum};

let mut world = World::new();
world.insert_resource(CameraFrustum::default());
```

### 3. Add Bounding Boxes to Entities

Every entity that should be culled needs a `BoundingBox` component:

```rust
use praxis_ecs::{World, Transform, GlobalTransform, MeshHandle, BoundingBox};
use praxis_math::Vec3;

world.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    GlobalTransform::default(),
    MeshHandle::new("cube"),
    BoundingBox::from_min_max(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0)),
));
```

## Usage Patterns

### Pattern 1: Build Draw Commands from Visible Entities

The simplest approach is to query only visible entities when building draw commands:

```rust
use praxis_ecs::{Query, With, Visible, MeshHandle, GlobalTransform, MaterialPropertiesComponent, TextureHandle};
use praxis_graphics::{DrawCommand, RenderCommands};

fn render_system(
    visible_entities: Query<
        (&MeshHandle, &GlobalTransform, Option<&TextureHandle>, Option<&MaterialPropertiesComponent>),
        With<Visible>
    >,
) {
    // Build draw commands only from visible entities
    let draw_commands: Vec<DrawCommand> = visible_entities
        .iter()
        .map(|(mesh, transform, texture, material)| DrawCommand {
            mesh_id: mesh.id().to_string(),
            model: transform.matrix,
            texture_name: texture.map(|t| t.id().to_string()),
            material_properties: material.map(|m| m.0),
        })
        .collect();
    
    // Pass to renderer
    // render_context.render(&RenderCommands {
    //     view, proj, draw_commands: &draw_commands, lighting: None
    // });
}
```

### Pattern 2: Query Visibility Directly

For custom rendering logic:

```rust
use praxis_ecs::{Query, With, Without, Visible, Culled};

fn custom_render(
    visible: Query<(&MeshHandle, &Transform), With<Visible>>,
    culled: Query<&MeshHandle, With<Culled>>,
) {
    println!("Visible entities: {}", visible.iter().count());
    println!("Culled entities: {}", culled.iter().count());
    
    for (mesh, transform) in visible.iter() {
        // Render visible entity
    }
}
```

## Computing Bounding Boxes

### From Mesh Vertices

For procedural or loaded meshes, compute the AABB from vertices:

```rust
use praxis_ecs::BoundingBox;
use praxis_math::Vec3;

fn compute_aabb_from_vertices(vertices: &[[f32; 3]]) -> BoundingBox {
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    
    for &[x, y, z] in vertices {
        let v = Vec3::new(x, y, z);
        min = min.min(v);
        max = max.max(v);
    }
    
    BoundingBox::from_min_max(min, max)
}
```

### Predefined Bounds for Primitives

For known primitive shapes:

```rust
// Cube with unit size
let cube_bounds = BoundingBox::from_center_half_extents(
    Vec3::ZERO,
    Vec3::splat(0.5),
);

// Sphere with radius 1.0
let sphere_bounds = BoundingBox::from_center_half_extents(
    Vec3::ZERO,
    Vec3::splat(1.0), // Circumscribed cube
);
```

## Transform Handling

The bounding box stored in the `BoundingBox` component is in **local space**. The frustum culling system automatically transforms it to world space using the entity's `GlobalTransform` before testing against the frustum.

```rust
// Local space bounds
let local_bounds = BoundingBox::from_min_max(Vec3::NEG_ONE, Vec3::ONE);

world.spawn((
    Transform::from_xyz(10.0, 5.0, 0.0).with_scale(Vec3::splat(2.0)),
    GlobalTransform::default(),
    local_bounds, // Will be transformed to world space automatically
));
```

## Performance Considerations

### Optimal Setup

1. **Bounding Box Precision**: Use tight-fitting bounding boxes. Oversized boxes cause unnecessary rendering.

2. **Hierarchy**: Parent-child hierarchies work correctly. Children inherit parent transforms, and their world-space bounds are computed automatically.

3. **Static vs Dynamic**: 
   - Static geometry: Compute bounds once at spawn
   - Dynamic geometry: Update bounds when vertices change

### Performance Metrics

Query the number of visible/culled entities:

```rust
use praxis_ecs::culling::count_visible_entities;

let visible_count = count_visible_entities(&visible_entities);
let cull_rate = 1.0 - (visible_count as f32 / total_entities as f32);
println!("Cull rate: {:.1}%", cull_rate * 100.0);
```

### Typical Performance

- **Cost**: ~50-100 μs for 10,000 entities (depends on CPU)
- **Benefit**: Skips rendering completely for culled objects
- **Best Case**: Dense scenes with many objects outside view (cities, forests)
- **Worst Case**: Objects always visible (small enclosed spaces)

## Integration with Deferred Rendering

The system works seamlessly with both forward and deferred rendering:

```rust
use praxis_graphics::DeferredRenderer;

// With deferred renderer
deferred_renderer.render(
    &mut builder,
    output_framebuffer,
    viewport,
    &visible_draw_commands, // Pre-filtered for visible entities
    view_proj_buffer,
    dynamic_uniform_buffer,
    mesh_manager,
    texture_manager,
    lighting_buffer,
)?;
```

## Debugging

### Visualizing Culling

To see which objects are culled:

```rust
fn debug_culling(
    visible: Query<(&Name, &Transform), With<Visible>>,
    culled: Query<(&Name, &Transform), With<Culled>>,
) {
    for (name, transform) in visible.iter() {
        println!("VISIBLE: {} at {:?}", name.as_str(), transform.translation);
    }
    
    for (name, transform) in culled.iter() {
        println!("CULLED: {} at {:?}", name.as_str(), transform.translation);
    }
}
```

### Drawing Bounding Boxes

Use the line renderer to visualize bounding boxes:

```rust
use praxis_graphics::visual_feedback::create_bounding_box;

fn draw_debug_bounds(
    entities: Query<(&BoundingBox, &GlobalTransform)>,
    line_renderer: &mut LineRenderer,
) {
    for (bbox, transform) in entities.iter() {
        let world_bbox = /* transform bbox to world space */;
        let lines = create_bounding_box(world_bbox.min, world_bbox.max, [1.0, 0.0, 0.0]);
        line_renderer.draw_lines(&lines);
    }
}
```

## Advanced Features

### Custom Culling Criteria

Extend the system with additional culling:

```rust
fn distance_culling_system(
    mut commands: Commands,
    camera_frustum: Res<CameraFrustum>,
    entities: Query<(Entity, &GlobalTransform), With<Visible>>,
) {
    let max_distance = 100.0;
    
    for (entity, transform) in entities.iter() {
        let distance = camera_frustum.position.distance(transform.translation());
        if distance > max_distance {
            commands.entity(entity).insert(Culled);
            commands.entity(entity).remove::<Visible>();
        }
    }
}
```

### LOD Integration

Combine with the LOD system for distance-based detail:

```rust
use praxis_spatial::LodManager;

fn lod_selection_system(
    camera_frustum: Res<CameraFrustum>,
    lod_manager: Res<LodManager>,
    entities: Query<(Entity, &GlobalTransform, &LodComponent), With<Visible>>,
) {
    for (entity, transform, lod_comp) in entities.iter() {
        let distance = camera_frustum.position.distance(transform.translation());
        let selected_lod = lod_manager.select_lod(entity, camera_frustum.position, transform.translation());
        // Use selected LOD level
    }
}
```

## Complete Example

Here's a complete example showing all components working together:

```rust
use praxis_ecs::{World, Schedule, IntoSystemConfigs, systems::*, Query, With, Visible};
use praxis_graphics::{RenderContext, RenderCommands, DrawCommand};
use praxis_math::{Vec3, Mat4};

fn main() -> praxis_utils::Result<()> {
    let mut world = World::new();
    
    // Initialize resources
    world.insert_resource(CameraFrustum::default());
    
    // Create schedule
    let mut schedule = Schedule::default();
    schedule.add_systems((
        propagate_transforms,
        update_perspective_cameras,
        update_frustum_from_camera,
        frustum_culling_system,
    ).chain());
    
    // Spawn camera
    world.spawn(PerspectiveCameraBundle::new(
        Vec3::new(0.0, 5.0, 10.0),
        70.0_f32.to_radians(),
        16.0 / 9.0,
    ));
    
    // Spawn renderable entities
    for x in -10..10 {
        for z in -10..10 {
            world.spawn((
                Transform::from_xyz(x as f32 * 2.0, 0.0, z as f32 * 2.0),
                GlobalTransform::default(),
                MeshHandle::new("cube"),
                BoundingBox::from_center_half_extents(Vec3::ZERO, Vec3::splat(0.5)),
            ));
        }
    }
    
    // Game loop
    loop {
        // Update ECS (includes frustum culling)
        schedule.run(world.inner_mut());
        
        // Build draw commands from visible entities only
        let mut visible_query = world.query_filtered::<(
            &MeshHandle,
            &GlobalTransform,
            Option<&TextureHandle>,
            Option<&MaterialPropertiesComponent>,
        ), With<Visible>>();
        
        let draw_commands: Vec<DrawCommand> = visible_query
            .iter(world.inner())
            .map(|(mesh, transform, texture, material)| DrawCommand {
                mesh_id: mesh.id().to_string(),
                model: transform.matrix,
                texture_name: texture.map(|t| t.id().to_string()),
                material_properties: material.map(|m| m.0),
            })
            .collect();
        
        println!("Rendering {} / {} objects", 
                 draw_commands.len(), 
                 world.query::<&MeshHandle>().iter(world.inner()).count());
        
        // Render
        // render_context.render(&RenderCommands {
        //     view: camera_view,
        //     proj: camera_proj,
        //     draw_commands: &draw_commands,
        //     lighting: Some(&lighting),
        // })?;
    }
}
```

## See Also

- [Spatial Optimization System](../crates/praxis_spatial/README.md) - Full spatial optimization documentation
- [LOD System](lod_system.md) - Level-of-detail mesh switching  
- [Occlusion Culling](occlusion_culling.md) - Hardware occlusion queries
- [BVH and Octrees](spatial_structures.md) - Hierarchical spatial partitioning
