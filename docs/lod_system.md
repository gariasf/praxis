# LOD (Level of Detail) System

The Praxis LOD system provides automatic Level of Detail management for 3D models, optimizing rendering performance by displaying different mesh variants based on distance from the camera.

## Overview

The LOD system consists of several key components:

- **`LodLevel`**: Defines a single LOD level with a mesh reference and distance thresholds
- **`LodGroup`**: Manages multiple LOD levels for a single entity
- **`LodGroupComponent`**: ECS component for attaching LOD groups to entities
- **`LodManager`**: System-wide LOD manager with global settings
- **`update_lod_system`**: ECS system that updates LOD groups each frame

## Key Features

### Distance-Based Selection

LOD selection uses **squared distance** to avoid expensive sqrt operations:

```rust
// Efficient distance comparison (no sqrt needed)
let delta = object_position - camera_position;
let distance_squared = delta.length_squared();

if distance_squared < threshold_squared {
    // Use high detail mesh
}
```

### Smooth Transitions

The system supports alpha-blended transitions between LOD levels to eliminate popping artifacts:

- During transition, both old and new meshes are rendered
- Alpha values are interpolated linearly over the transition duration
- Configurable transition duration per LOD group

### Flexible Configuration

- Per-entity LOD settings (transition duration, bias)
- Global LOD bias for quality scaling
- Optional screen-space LOD (future feature)
- Enable/disable transitions for immediate switching

## Usage

### Basic Setup

```rust
use praxis_ecs::{World, Transform, LodGroupComponent};
use praxis_graphics::lod::{LodGroup, LodLevel};

// Create LOD group with 3 detail levels
let lod_group = LodGroup::new(vec![
    LodLevel::new("tree_high", 0.0, 20.0),    // High detail: 0-20 units
    LodLevel::new("tree_medium", 20.0, 50.0), // Medium: 20-50 units
    LodLevel::new("tree_low", 50.0, 100.0),   // Low detail: 50-100 units
]);

// Spawn entity with LOD
world.spawn((
    Transform::from_xyz(10.0, 0.0, 10.0),
    LodGroupComponent::new(lod_group),
));
```

### Configuring Transitions

```rust
let mut lod_group = LodGroup::new(levels);

// Set transition duration (in seconds)
lod_group.set_transition_duration(0.5);

// Enable smooth transitions
lod_group.enable_transitions(true);

// Optionally disable blending during transition
lod_group.set_blend_during_transition(false);
```

### LOD Bias

LOD bias allows forcing higher or lower detail levels:

```rust
// Positive bias: prefer higher detail (objects appear closer)
lod_group.set_lod_bias(0.5);

// Negative bias: prefer lower detail (performance mode)
lod_group.set_lod_bias(-0.5);

// Global LOD bias via manager
let mut lod_manager = LodManager::new();
lod_manager.set_global_lod_bias(0.3);
```

### System Integration

Add the LOD update system to your ECS schedule:

```rust
use praxis_ecs::{Schedule, IntoSystemConfigs};
use praxis_ecs::systems::{update_lod_system, CoreSystemSet};

let mut schedule = Schedule::default();

schedule.add_systems(
    update_lod_system.in_set(CoreSystemSet::PostUpdate)
);
```

Don't forget to update the `DeltaTime` resource each frame:

```rust
use praxis_ecs::systems::DeltaTime;

world.insert_resource(DeltaTime(delta_time_seconds));
```

### Rendering with LOD

The LOD system integrates with the rendering pipeline:

```rust
use praxis_graphics::{DrawCommand, RenderCommands};

// Collect draw commands based on active LOD levels
let mut draw_commands = Vec::new();

for (lod_group, transform) in lod_query.iter(&world) {
    // Get meshes to render (may be multiple during transition)
    let render_meshes = lod_group.get_render_meshes();
    
    for (mesh_id, alpha) in render_meshes {
        draw_commands.push(DrawCommand {
            mesh_id: mesh_id.to_string(),
            model: transform.compute_matrix(),
            texture_name: None,
            material_properties: None,
        });
    }
}

// Render with collected commands
let render_commands = RenderCommands {
    view,
    proj,
    draw_commands: &draw_commands,
    lighting: None,
};

render_context.render(&render_commands)?;
```

## Performance Characteristics

### CPU Cost

- **LOD Selection**: O(n) where n is number of LOD groups
- **Distance Calculation**: Uses squared distance (no sqrt)
- **Transition Updates**: Only active during transitions

### Memory

- **Per LOD Group**: ~200 bytes + mesh references
- **Mesh Storage**: Shared between entities
- **No Runtime Allocations**: During steady state

### GPU Cost

- **Transitions**: Renders 2 meshes during transition
- **Steady State**: Renders 1 mesh per entity
- **Optimal**: Reduces triangle count for distant objects

## Best Practices

### Distance Thresholds

Choose distance thresholds based on:
- Screen coverage (how many pixels the object occupies)
- Visual importance (player characters need higher detail)
- Triangle count reduction (aim for 50-75% reduction per level)

```rust
// Good example for a character:
LodLevel::new("character_high", 0.0, 15.0),    // ~10k triangles
LodLevel::new("character_medium", 15.0, 40.0), // ~5k triangles
LodLevel::new("character_low", 40.0, 100.0),   // ~1k triangles
LodLevel::new("character_impostor", 100.0, 200.0), // 2 triangles (billboard)
```

### Transition Duration

Balance smoothness vs performance:
- **0.2-0.3s**: Good default, barely noticeable
- **0.1s**: Faster, slight pop visible
- **0.5s**: Very smooth, higher GPU cost

```rust
// Recommended values
lod_group.set_transition_duration(0.25); // Default
```

### LOD Mesh Creation

Create LOD meshes by:
1. **Manual Reduction**: Use Blender's Decimate modifier
2. **Automatic**: Use LOD generation tools
3. **Progressive**: Build mesh hierarchy from simplest to most detailed

```rust
// Example: sphere with different subdivision counts
let high = sphere_mesh(10);   // 1200 triangles
let medium = sphere_mesh(5);  // 300 triangles
let low = sphere_mesh(2);     // 48 triangles
```

## Example: Complete LOD Setup

```rust
use praxis::{
    praxis_core::Engine,
    praxis_ecs::{
        systems::{update_lod_system, DeltaTime},
        LodGroupComponent, Transform, World,
    },
    praxis_graphics::lod::{LodGroup, LodLevel, LodManager},
};

async fn setup_lod_demo() -> Result<()> {
    let mut engine = Engine::new(window_config).await?;
    let mut world = World::new();
    let mut lod_manager = LodManager::new();

    // Load mesh variants
    let render_context = engine.render_context_mut();
    render_context.mesh_manager_mut().load_mesh("model_lod0", high_detail)?;
    render_context.mesh_manager_mut().load_mesh("model_lod1", medium_detail)?;
    render_context.mesh_manager_mut().load_mesh("model_lod2", low_detail)?;

    // Create LOD group
    let lod_group = LodGroup::new(vec![
        LodLevel::new("model_lod0", 0.0, 20.0),
        LodLevel::new("model_lod1", 20.0, 60.0),
        LodLevel::new("model_lod2", 60.0, 150.0),
    ]);

    // Spawn entity
    world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        LodGroupComponent::new(lod_group),
    ));

    // Initialize delta time
    world.insert_resource(DeltaTime(0.016));

    // Main loop
    engine.run(move |events, window, render_context| {
        // Update delta time
        world.insert_resource(DeltaTime(delta_time));

        // Update LOD system (happens automatically in schedule)
        // Or call manually:
        update_lod_system(
            world.query::<(&mut LodGroupComponent, &GlobalTransform)>(),
            world.query::<(&Camera, &GlobalTransform)>(),
            world.resource::<DeltaTime>().copied(),
        );

        // Render...
        Ok(true)
    })
}
```

## Advanced Features

### Force LOD Level

For debugging or cutscenes:

```rust
lod_group.force_lod_level(0); // Force highest detail
lod_group.force_lod_level(2); // Force lowest detail
```

### Query LOD State

```rust
// Check current level
let level = lod_group.current_level();

// Check if transitioning
if lod_group.is_transitioning() {
    let progress = lod_group.transition_progress(); // 0.0 to 1.0
}

// Get alpha values during transition
let current_alpha = lod_group.current_alpha();
let target_alpha = lod_group.target_alpha();
```

### Global LOD Control

```rust
let mut lod_manager = LodManager::new();

// Enable/disable globally
lod_manager.set_enabled(false); // Freeze LOD updates

// Adjust quality globally
lod_manager.set_global_lod_bias(0.5); // Higher detail everywhere

// Get statistics
let stats = lod_manager.statistics();
println!("Active groups: {}", stats.active_groups);
println!("Transitioning: {}", stats.transitioning_groups);
```

## Future Enhancements

Potential future additions:

- **Screen-space LOD**: Select level based on screen coverage
- **Hysteresis**: Prevent flickering at LOD boundaries
- **LOD Fading**: Dither pattern transitions
- **Mesh Morphing**: Smooth vertex transitions
- **Automatic LOD Generation**: Runtime mesh simplification

## See Also

- [Mesh System](mesh_system.md)
- [Rendering Pipeline](RENDERING_EXPLAINED.md)
- [Spatial Optimization](spatial_optimization.md)
- [Frustum Culling](frustum_culling.md)
