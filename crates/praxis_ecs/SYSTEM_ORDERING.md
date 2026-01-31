# System Ordering and Scheduling Guide

Complete guide to understanding and controlling system execution order in the Praxis ECS.

## Table of Contents

1. [Why System Ordering Matters](#why-system-ordering-matters)
2. [Automatic Ordering](#automatic-ordering)
3. [Explicit Ordering](#explicit-ordering)
4. [System Sets](#system-sets)
5. [Common Ordering Patterns](#common-ordering-patterns)
6. [Debugging Order Issues](#debugging-order-issues)

## Why System Ordering Matters

System execution order affects correctness and performance:

```rust
// WRONG ORDER:
fn render_system() { }      // Renders old positions
fn movement_system() { }    // Updates positions

// CORRECT ORDER:
fn movement_system() { }    // Updates positions first
fn render_system() { }      // Renders new positions
```

### Data Dependencies

Systems that read/write the same data have dependencies:

```rust
// System A writes Transform
fn physics_system(mut query: Query<&mut Transform>) { }

// System B reads Transform
fn render_system(query: Query<&Transform>) { }

// physics_system MUST run before render_system
```

## Automatic Ordering

### Parallel Execution

Systems with no conflicts run in parallel:

```rust
// These can run simultaneously (both read-only)
fn render_system(query: Query<&Transform>) { }
fn audio_system(query: Query<&Transform>) { }
```

### Sequential for Conflicts

Systems with overlapping mutable access run sequentially:

```rust
// These MUST run sequentially
fn physics_system(mut query: Query<&mut Transform>) { }
fn animation_system(mut query: Query<&mut Transform>) { }

// ECS automatically serializes these
```

### Access Pattern Analysis

The scheduler analyzes system parameters:

```rust
fn system_a(
    mut query: Query<&mut Transform>,  // Writes Transform
    health: Query<&Health>,            // Reads Health
) { }

fn system_b(
    query: Query<&Transform>,          // Reads Transform
    mut health: Query<&mut Health>,    // Writes Health
) { }

// system_a writes Transform, system_b reads it → must run in order
// system_a reads Health, system_b writes it → must run in order
// Therefore: system_a → system_b
```

## Explicit Ordering

### Using chain()

Force systems to run in sequence:

```rust
use praxis_ecs::{Schedule, IntoSystemConfigs};

let mut schedule = Schedule::default();

schedule.add_systems((
    input_system,
    physics_system,
    animation_system,
    render_system,
).chain());  // Runs in this exact order
```

### Using before() and after()

Specify relative ordering:

```rust
schedule.add_systems(
    physics_system
        .after(input_system)
        .before(render_system)
);

schedule.add_systems(
    animation_system
        .after(physics_system)
        .before(render_system)
);
```

### Multiple Dependencies

Systems can have multiple ordering constraints:

```rust
schedule.add_systems(
    update_ai
        .after(input_system)
        .after(physics_system)
        .before(animation_system)
);
```

## System Sets

### Defining Sets

Group related systems together:

```rust
use praxis_ecs::SystemSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameSystemSet {
    Input,
    Logic,
    Physics,
    Animation,
    Rendering,
}
```

### Adding Systems to Sets

```rust
schedule.add_systems(
    input_system.in_set(GameSystemSet::Input)
);

schedule.add_systems(
    (physics_system, collision_system)
        .in_set(GameSystemSet::Physics)
);

schedule.add_systems(
    render_system.in_set(GameSystemSet::Rendering)
);
```

### Ordering Sets

Order entire groups at once:

```rust
schedule.configure_sets((
    GameSystemSet::Input,
    GameSystemSet::Logic,
    GameSystemSet::Physics,
    GameSystemSet::Animation,
    GameSystemSet::Rendering,
).chain());
```

### Nested Sets

Sets can contain other sets:

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum PhysicsSet {
    Integration,
    Collision,
    Resolution,
}

schedule.configure_sets((
    PhysicsSet::Integration,
    PhysicsSet::Collision,
    PhysicsSet::Resolution,
).chain().in_set(GameSystemSet::Physics));
```

## Common Ordering Patterns

### Game Loop Pattern

Standard game update order:

```rust
schedule.add_systems((
    // 1. Input
    read_keyboard,
    read_mouse,
    read_gamepad,
    
    // 2. Pre-update
    update_time,
    clear_events,
    
    // 3. Update
    player_input,
    ai_update,
    spawner_system,
    
    // 4. Physics
    physics_step,
    collision_detection,
    collision_response,
    
    // 5. Transform propagation
    sync_parent_child_relationships,
    propagate_transforms,
    
    // 6. Animation
    update_animations,
    apply_animation_transforms,
    
    // 7. Post-update
    update_camera,
    update_audio_listener,
    
    // 8. Rendering
    gather_visible_entities,
    update_render_resources,
    render_frame,
).chain());
```

### Transform Hierarchy Pattern

Proper order for hierarchical transforms:

```rust
use praxis_ecs::systems::*;

schedule.add_systems((
    // 1. Update parent-child relationships
    sync_parent_child_relationships,
    cleanup_removed_parents,
    
    // 2. Propagate root transforms
    propagate_transforms,
    
    // 3. Handle reparented entities
    propagate_transforms_for_reparented,
    
    // 4. Handle changed children
    propagate_transforms_for_changed_children,
).chain());
```

### Physics Pattern

Physics simulation order:

```rust
schedule.add_systems((
    // 1. Apply forces
    apply_gravity,
    apply_impulses,
    
    // 2. Integrate
    integrate_velocity,
    
    // 3. Collision detection
    broad_phase_collision,
    narrow_phase_collision,
    
    // 4. Constraint solving
    solve_constraints,
    
    // 5. Integration
    integrate_position,
    
    // 6. Sync with ECS
    sync_physics_to_transform,
).chain());
```

### Rendering Pattern

```rust
schedule.add_systems((
    // 1. Update camera
    update_camera_matrices,
    
    // 2. Culling
    frustum_culling,
    occlusion_culling,
    
    // 3. LOD selection
    update_lod_levels,
    
    // 4. Gather rendering data
    gather_lighting,
    gather_visible_meshes,
    
    // 5. Render
    render_shadows,
    render_main_pass,
    render_post_processing,
).chain());
```

### Event Processing Pattern

```rust
use praxis_ecs::{EventReader, EventWriter};

#[derive(Event)]
struct CollisionEvent;

// Event producers run first
fn collision_detection(mut events: EventWriter<CollisionEvent>) {
    // Generate events
}

// Event consumers run after
fn damage_system(mut events: EventReader<CollisionEvent>) {
    for event in events.read() {
        // Process events
    }
}

schedule.add_systems((
    collision_detection,  // Produces events
    damage_system,        // Consumes events
).chain());
```

## Debugging Order Issues

### Symptom: Flickering or Frame-Delay

**Cause:** Systems running in wrong order

```rust
// BAD: Render uses old transform
schedule.add_systems((
    render_system,      // Uses transform from last frame
    movement_system,    // Updates transform
));

// GOOD: Movement before render
schedule.add_systems((
    movement_system,    // Updates transform
    render_system,      // Uses current transform
).chain());
```

### Symptom: Inconsistent Behavior

**Cause:** Non-deterministic execution order

```rust
// BAD: No explicit ordering (race condition)
schedule.add_systems(physics_system);
schedule.add_systems(animation_system);
// Order is undefined!

// GOOD: Explicit ordering
schedule.add_systems((
    physics_system,
    animation_system,
).chain());
```

### Symptom: Transform Not Updating

**Cause:** Missing transform propagation systems

```rust
// BAD: Child transform never updates
let parent = world.spawn(Transform::from_xyz(10.0, 0.0, 0.0));
let child = world.spawn((
    Transform::from_xyz(5.0, 0.0, 0.0),
    Parent(parent),
));
// GlobalTransform stays at identity!

// GOOD: Add transform systems
schedule.add_systems((
    sync_parent_child_relationships,
    propagate_transforms,
).chain());
```

### Symptom: Events Not Firing

**Cause:** Event consumers before producers

```rust
// BAD: Consumer runs before producer
schedule.add_systems((
    damage_system,          // Reads events (none yet!)
    collision_detection,    // Writes events
).chain());

// GOOD: Producer before consumer
schedule.add_systems((
    collision_detection,    // Writes events first
    damage_system,          // Reads events
).chain());
```

## Best Practices

### 1. Document System Dependencies

```rust
/// Updates entity positions based on velocity.
///
/// **Dependencies:**
/// - Must run AFTER `apply_forces` (which updates velocity)
/// - Must run BEFORE `collision_detection` (which uses position)
fn integrate_velocity(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<DeltaTime>,
) { }
```

### 2. Use System Sets for Organization

```rust
// Good: Organized into logical stages
schedule.add_systems(
    player_input.in_set(GameSystemSet::Input)
);
schedule.add_systems(
    enemy_ai.in_set(GameSystemSet::Logic)
);

// Bad: Everything in default set (harder to reason about)
schedule.add_systems(player_input);
schedule.add_systems(enemy_ai);
```

### 3. Explicit is Better Than Implicit

```rust
// Good: Clear and explicit
schedule.add_systems((
    physics_system,
    animation_system,
).chain());

// Unclear: Relies on automatic ordering
schedule.add_systems(physics_system);
schedule.add_systems(animation_system);
```

### 4. Group Related Systems

```rust
// Good: Related systems together
schedule.add_systems((
    physics_step,
    collision_detection,
    collision_response,
).chain().in_set(PhysicsSet));

// Bad: Scattered across schedule
schedule.add_systems(physics_step);
// ... other systems ...
schedule.add_systems(collision_detection);
// ... other systems ...
schedule.add_systems(collision_response);
```

### 5. Minimize Sequential Dependencies

```rust
// Good: Enable parallelism
fn render_meshes(query: Query<&Transform>) { }
fn render_particles(query: Query<&Transform>) { }
fn update_audio(query: Query<&Transform>) { }
// All read-only, can run in parallel

// Bad: Unnecessary sequential execution
fn mega_system(
    meshes: Query<&Transform>,
    particles: Query<&Transform>,
    audio: Query<&Transform>,
) {
    // Everything in one system prevents parallelism
}
```

## Advanced Patterns

### Conditional System Execution

Run systems only when conditions are met:

```rust
#[derive(Resource)]
struct GameState {
    paused: bool,
}

fn should_run_gameplay(state: Res<GameState>) -> bool {
    !state.paused
}

schedule.add_systems(
    gameplay_system.run_if(should_run_gameplay)
);
```

### Multiple Schedules

Separate schedules for different stages:

```rust
use praxis_ecs::ScheduleLabel;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct FixedUpdate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Update;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Render;

// Physics runs at fixed timestep
let mut fixed_schedule = Schedule::new(FixedUpdate);
fixed_schedule.add_systems(physics_system);

// Logic runs every frame
let mut update_schedule = Schedule::new(Update);
update_schedule.add_systems(gameplay_system);

// Rendering runs after everything
let mut render_schedule = Schedule::new(Render);
render_schedule.add_systems(render_system);
```

### Ambiguity Detection

Check for potential ordering issues:

```rust
// Systems with ambiguous order (both write Transform)
schedule.add_systems(system_a);
schedule.add_systems(system_b);

// Explicitly allow ambiguity if intentional
schedule.add_systems(
    (system_a, system_b)
        .ambiguous_with_all()  // Suppress warning
);
```

## Summary

**Key Principles:**

1. **Systems with data conflicts run sequentially** - Automatic
2. **Systems without conflicts run in parallel** - Automatic
3. **Use `.chain()` for explicit ordering** - Manual control
4. **Use System Sets for organization** - Scalability
5. **Document dependencies** - Maintainability
6. **Test execution order** - Correctness

**Common Order:**
1. Input
2. Logic/AI
3. Physics
4. Transform propagation
5. Animation
6. Camera update
7. Culling
8. Rendering

Following these guidelines ensures your systems execute correctly and efficiently.
