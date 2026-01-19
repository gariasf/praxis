# ECS System Execution Order

This document provides a comprehensive visual guide to the order in which ECS systems execute during a frame in the Praxis engine, showing dependencies, parallel execution opportunities, and data flow between systems.

## Overview

The Praxis engine uses `bevy_ecs` for its Entity-Component-System implementation. Systems are organized into **system sets** that execute in a defined order, with automatic parallelization within sets when systems have no data dependencies.

## Frame Execution Overview

Each frame executes systems in the following high-level order:

```
Frame N Start
    ↓
Input Processing
    ↓
Pre-Update (Transform Prep)
    ↓
Update (Game Logic)
    ↓
Post-Update (Finalization)
    ↓
Render
    ↓
Frame N End → Frame N+1 Start
```

## System Sets

The engine defines several standard system sets:

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoreSystemSet {
    Input,                  // Input event processing
    PreUpdate,              // Pre-update preparation
    TransformPropagate,     // Transform hierarchy updates
    Update,                 // Main game logic
    PostUpdate,             // Post-update cleanup
    Render,                 // Rendering preparation
}
```

## Detailed System Execution Flow

### Complete Frame Pipeline

```
┌──────────────────────────────────────────────────────────────────────────┐
│                              FRAME N                                      │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ SET: Input                                                       │    │
│  │ Purpose: Collect and process input events                       │    │
│  ├─────────────────────────────────────────────────────────────────┤    │
│  │                                                                  │    │
│  │  ┌──────────────────────┐  ┌──────────────────────┐            │    │
│  │  │ keyboard_input       │  │ mouse_input          │ (Parallel) │    │
│  │  │                      │  │                      │            │    │
│  │  │ - Read winit events  │  │ - Read winit events  │            │    │
│  │  │ - Update key states  │  │ - Update button states│           │    │
│  │  │ - Track press/release│  │ - Track motion       │            │    │
│  │  └──────────────────────┘  └──────────────────────┘            │    │
│  │                  ↓                  ↓                           │    │
│  │                  └──────────┬───────┘                           │    │
│  │                             ↓                                    │    │
│  │  ┌──────────────────────────────────────────────┐               │    │
│  │  │ gamepad_input                                │               │    │
│  │  │                                              │               │    │
│  │  │ - Poll gamepad state (gilrs)                │               │    │
│  │  │ - Update button/axis states                 │               │    │
│  │  │ - Handle connection/disconnection            │               │    │
│  │  └──────────────────────────────────────────────┘               │    │
│  │                             ↓                                    │    │
│  │  ┌──────────────────────────────────────────────┐               │    │
│  │  │ input_action_mapping                         │               │    │
│  │  │                                              │               │    │
│  │  │ - Map raw inputs to game actions            │               │    │
│  │  │ - Handle composite actions                   │               │    │
│  │  │ - Update action state resource               │               │    │
│  │  └──────────────────────────────────────────────┘               │    │
│  │                                                                  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                               ↓                                           │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ SET: PreUpdate                                                   │    │
│  │ Purpose: Prepare data before main update                        │    │
│  ├─────────────────────────────────────────────────────────────────┤    │
│  │                                                                  │    │
│  │  ┌──────────────────────────────────────────────┐               │    │
│  │  │ sync_parent_child_relationships              │               │    │
│  │  │                                              │               │    │
│  │  │ - Detect new Parent components               │               │    │
│  │  │ - Update Children on parent entities        │               │    │
│  │  │ - Handle reparenting                         │               │    │
│  │  │                                              │               │    │
│  │  │ Components:                                  │               │    │
│  │  │   Read: Parent                               │               │    │
│  │  │   Write: Children                            │               │    │
│  │  └──────────────────────────────────────────────┘               │    │
│  │                             ↓                                    │    │
│  │  ┌──────────────────────────────────────────────┐               │    │
│  │  │ cleanup_removed_parents                      │               │    │
│  │  │                                              │               │    │
│  │  │ - Remove orphaned children references       │               │    │
│  │  │ - Clear invalid Parent components           │               │    │
│  │  │                                              │               │    │
│  │  │ Components:                                  │               │    │
│  │  │   Write: Parent, Children                   │               │    │
│  │  └──────────────────────────────────────────────┘               │    │
│  │                             ↓                                    │    │
│  │  ┌──────────────────────────────────────────────┐               │    │
│  │  │ update_delta_time_resource                   │               │    │
│  │  │                                              │               │    │
│  │  │ - Calculate time since last frame           │               │    │
│  │  │ - Update DeltaTime resource                 │               │    │
│  │  │ - Update total elapsed time                  │               │    │
│  │  └──────────────────────────────────────────────┘               │    │
│  │                                                                  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                               ↓                                           │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ SET: TransformPropagate                                          │    │
│  │ Purpose: Update transform hierarchy                              │    │
│  ├─────────────────────────────────────────────────────────────────┤    │
│  │                                                                  │    │
│  │  ┌──────────────────────────────────────────────┐               │    │
│  │  │ propagate_transforms                         │               │    │
│  │  │                                              │               │    │
│  │  │ - Update GlobalTransform from local Transform│               │    │
│  │  │ - Propagate through Parent→Children hierarchy│              │    │
│  │  │ - Breadth-first traversal                    │               │    │
│  │  │                                              │               │    │
│  │  │ Components:                                  │               │    │
│  │  │   Read: Transform, Parent, Children          │               │    │
│  │  │   Write: GlobalTransform                     │               │    │
│  │  └──────────────────────────────────────────────┘               │    │
│  │                             ↓                                    │    │
│  │  ┌──────────────────────────────────────────────┐               │    │
│  │  │ propagate_transforms_for_reparented          │               │    │
│  │  │                                              │               │    │
│  │  │ - Handle entities that changed parents       │               │    │
│  │  │ - Recalculate transform chain                │               │    │
│  │  │                                              │               │    │
│  │  │ Components:                                  │               │    │
│  │  │   Read: Transform, Parent (Changed)          │               │    │
│  │  │   Write: GlobalTransform                     │               │    │
│  │  └──────────────────────────────────────────────┘               │    │
│  │                             ↓                                    │    │
│  │  ┌──────────────────────────────────────────────┐               │    │
│  │  │ propagate_transforms_for_changed_children    │               │    │
│  │  │                                              │               │    │
│  │  │ - Update when Children component changes     │               │    │
│  │  │ - Recalculate affected subtrees              │               │    │
│  │  │                                              │               │    │
│  │  │ Components:                                  │               │    │
│  │  │   Read: Transform, Children (Changed)        │               │    │
│  │  │   Write: GlobalTransform                     │               │    │
│  │  └──────────────────────────────────────────────┘               │    │
│  │                                                                  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                               ↓                                           │
└──────────────────────────────────────────────────────────────────────────┘
                                ↓
```

```
┌──────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ SET: Update                                                      │    │
│  │ Purpose: Main game logic and simulation                         │    │
│  ├─────────────────────────────────────────────────────────────────┤    │
│  │                                                                  │    │
│  │  ┌─────────────────────────────────────────────────────┐        │    │
│  │  │ SUBSYSTEM: Physics (Sequential)                     │        │    │
│  │  ├─────────────────────────────────────────────────────┤        │    │
│  │  │                                                      │        │    │
│  │  │  1. sync_transforms_to_physics                      │        │    │
│  │  │     - Copy Transform → RigidBody positions          │        │    │
│  │  │     - Only for entities changed on ECS side         │        │    │
│  │  │     Components: Read Transform (Changed), Write RigidBody    │    │
│  │  │                                                      │        │    │
│  │  │  2. physics_step                                    │        │    │
│  │  │     - Step Rapier simulation (dt)                   │        │    │
│  │  │     - Detect collisions                             │        │    │
│  │  │     - Resolve constraints                            │        │    │
│  │  │     Resource: Write PhysicsWorld                    │        │    │
│  │  │                                                      │        │    │
│  │  │  3. sync_physics_to_transforms                      │        │    │
│  │  │     - Copy RigidBody positions → Transform          │        │    │
│  │  │     - Update velocity components                     │        │    │
│  │  │     Components: Read RigidBody, Write Transform     │        │    │
│  │  │                                                      │        │    │
│  │  │  4. handle_collision_events                         │        │    │
│  │  │     - Query PhysicsWorld for collisions             │        │    │
│  │  │     - Spawn collision event components              │        │    │
│  │  │     Resource: Read PhysicsWorld                     │        │    │
│  │  │                                                      │        │    │
│  │  └─────────────────────────────────────────────────────┘        │    │
│  │                             ↓                                    │    │
│  │  ┌─────────────────────────────────────────────────────┐        │    │
│  │  │ SUBSYSTEM: Animation (Sequential)                   │        │    │
│  │  ├─────────────────────────────────────────────────────┤        │    │
│  │  │                                                      │        │    │
│  │  │  1. animation_update                                │        │    │
│  │  │     - Advance AnimationPlayer time                  │        │    │
│  │  │     - Sample animation clips                        │        │    │
│  │  │     - Handle looping/completion                      │        │    │
│  │  │     Components: Write AnimationPlayer, Read AnimationClip   │    │
│  │  │                                                      │        │    │
│  │  │  2. animation_blending                              │        │    │
│  │  │     - Blend between animation states                │        │    │
│  │  │     - Apply blend trees                             │        │    │
│  │  │     - Calculate final pose                          │        │    │
│  │  │     Components: Read AnimationPlayer, Write Skeleton│        │    │
│  │  │                                                      │        │    │
│  │  │  3. skeleton_update                                 │        │    │
│  │  │     - Apply pose to bone transforms                 │        │    │
│  │  │     - Calculate bone matrices                        │        │    │
│  │  │     Components: Read Skeleton, Write BoneTransforms │        │    │
│  │  │                                                      │        │    │
│  │  └─────────────────────────────────────────────────────┘        │    │
│  │                                                                  │    │
│  │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────┐  │    │
│  │  │ scripting_update │  │ ai_update        │  │ game_logic   │  │    │
│  │  │                  │  │                  │  │              │  │    │
│  │  │ - Execute Lua    │  │ - State machines │  │ - Custom     │  │    │
│  │  │ - Hot reload     │  │ - Pathfinding    │  │ - Gameplay   │  │    │
│  │  │ - ECS access     │  │ - Behavior trees │  │ - Rules      │  │    │
│  │  └──────────────────┘  └──────────────────┘  └──────────────┘  │    │
│  │          ↓                      ↓                    ↓           │    │
│  │          └──────────────────────┴────────────────────┘           │    │
│  │                             ↓                                    │    │
│  │  ┌─────────────────────────────────────────────────────┐        │    │
│  │  │ camera_update                                       │        │    │
│  │  │                                                      │        │    │
│  │  │ - Update view matrices                              │        │    │
│  │  │ - Update projection matrices                        │        │    │
│  │  │ - Handle camera controllers                         │        │    │
│  │  │                                                      │        │    │
│  │  │ Components:                                         │        │    │
│  │  │   Read: Transform, GlobalTransform                  │        │    │
│  │  │   Write: Camera, ViewMatrix, ProjectionMatrix       │        │    │
│  │  └─────────────────────────────────────────────────────┘        │    │
│  │                             ↓                                    │    │
│  │  ┌─────────────────────────────────────────────────────┐        │    │
│  │  │ audio_update                                        │        │    │
│  │  │                                                      │        │    │
│  │  │ - Update 3D audio source positions                  │        │    │
│  │  │ - Calculate attenuation                             │        │    │
│  │  │ - Apply doppler effect                              │        │    │
│  │  │                                                      │        │    │
│  │  │ Components: Read GlobalTransform, AudioSource       │        │    │
│  │  │ Resource: Write AudioManager                        │        │    │
│  │  └─────────────────────────────────────────────────────┘        │    │
│  │                                                                  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                               ↓                                           │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ SET: PostUpdate                                                  │    │
│  │ Purpose: Prepare data for rendering                             │    │
│  ├─────────────────────────────────────────────────────────────────┤    │
│  │                                                                  │    │
│  │  ┌─────────────────────────────────────────────────────┐        │    │
│  │  │ frustum_culling                                     │        │    │
│  │  │                                                      │        │    │
│  │  │ - Extract camera frustum planes                     │        │    │
│  │  │ - Test entity bounds vs frustum                     │        │    │
│  │  │ - Set Visible component                             │        │    │
│  │  │                                                      │        │    │
│  │  │ Components:                                         │        │    │
│  │  │   Read: GlobalTransform, BoundingBox, Camera        │        │    │
│  │  │   Write: Visible                                    │        │    │
│  │  └─────────────────────────────────────────────────────┘        │    │
│  │                             ↓                                    │    │
│  │  ┌─────────────────────────────────────────────────────┐        │    │
│  │  │ lod_selection                                       │        │    │
│  │  │                                                      │        │    │
│  │  │ - Calculate distance to camera                      │        │    │
│  │  │ - Select appropriate LOD level                      │        │    │
│  │  │ - Update ActiveLOD component                        │        │    │
│  │  │                                                      │        │    │
│  │  │ Components:                                         │        │    │
│  │  │   Read: GlobalTransform, LODLevels, Camera          │        │    │
│  │  │   Write: ActiveLOD                                  │        │    │
│  │  └─────────────────────────────────────────────────────┘        │    │
│  │                             ↓                                    │    │
│  │  ┌─────────────────────────────────────────────────────┐        │    │
│  │  │ gather_lighting                                     │        │    │
│  │  │                                                      │        │    │
│  │  │ - Query all DirectionalLight components            │        │    │
│  │  │ - Query all PointLight components                  │        │    │
│  │  │ - Build LightingData resource                       │        │    │
│  │  │ - Calculate shadow cascade splits                   │        │    │
│  │  │                                                      │        │    │
│  │  │ Components:                                         │        │    │
│  │  │   Read: DirectionalLight, PointLight, GlobalTransform│       │    │
│  │  │ Resource: Write LightingData                        │        │    │
│  │  └─────────────────────────────────────────────────────┘        │    │
│  │                             ↓                                    │    │
│  │  ┌─────────────────────────────────────────────────────┐        │    │
│  │  │ particle_update                                     │        │    │
│  │  │                                                      │        │    │
│  │  │ - Update particle lifetimes                         │        │    │
│  │  │ - Simulate particle physics                         │        │    │
│  │  │ - Remove dead particles                             │        │    │
│  │  │                                                      │        │    │
│  │  │ Components: Write ParticleEmitter                   │        │    │
│  │  └─────────────────────────────────────────────────────┘        │    │
│  │                             ↓                                    │    │
│  │  ┌─────────────────────────────────────────────────────┐        │    │
│  │  │ network_replication (Optional)                      │        │    │
│  │  │                                                      │        │    │
│  │  │ - Collect replicated component changes              │        │    │
│  │  │ - Serialize and send to clients/server              │        │    │
│  │  │ - Apply received updates                            │        │    │
│  │  │                                                      │        │    │
│  │  │ Components: Read Replicated, NetworkId              │        │    │
│  │  │ Resource: Write NetworkServer/NetworkClient         │        │    │
│  │  └─────────────────────────────────────────────────────┘        │    │
│  │                                                                  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                               ↓                                           │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ SET: Render                                                      │    │
│  │ Purpose: Generate render commands                               │    │
│  ├─────────────────────────────────────────────────────────────────┤    │
│  │                                                                  │    │
│  │  ┌─────────────────────────────────────────────────────┐        │    │
│  │  │ generate_render_commands                            │        │    │
│  │  │                                                      │        │    │
│  │  │ - Query visible entities with Mesh + Material       │        │    │
│  │  │ - Create DrawCommand for each                       │        │    │
│  │  │ - Sort by material (batching)                       │        │    │
│  │  │ - Sort by depth (early-Z)                           │        │    │
│  │  │                                                      │        │    │
│  │  │ Components:                                         │        │    │
│  │  │   Read: Visible, MeshHandle, MaterialHandle,        │        │    │
│  │  │         GlobalTransform, ActiveLOD                   │        │    │
│  │  │ Resource: Write RenderCommands                      │        │    │
│  │  └─────────────────────────────────────────────────────┘        │    │
│  │                             ↓                                    │    │
│  │  ┌─────────────────────────────────────────────────────┐        │    │
│  │  │ update_uniform_buffers                              │        │    │
│  │  │                                                      │        │    │
│  │  │ - Write view/projection matrices                    │        │    │
│  │  │ - Write lighting data                               │        │    │
│  │  │ - Write model matrices                              │        │    │
│  │  │                                                      │        │    │
│  │  │ Resource: Read Camera, LightingData                 │        │    │
│  │  │          Write UniformBuffers                       │        │    │
│  │  └─────────────────────────────────────────────────────┘        │    │
│  │                             ↓                                    │    │
│  │  ┌─────────────────────────────────────────────────────┐        │    │
│  │  │ GUI Rendering (egui)                                │        │    │
│  │  │                                                      │        │    │
│  │  │ - Layout and render GUI                             │        │    │
│  │  │ - Generate GUI primitives                           │        │    │
│  │  │                                                      │        │    │
│  │  │ Resource: Write GuiContext                          │        │    │
│  │  └─────────────────────────────────────────────────────┘        │    │
│  │                                                                  │    │
│  │  NOTE: Actual GPU rendering happens outside ECS                │    │
│  │        in RenderContext::render() on the main thread             │    │
│  │                                                                  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

## Parallel Execution

Systems within the same set run in **parallel** when they have no data conflicts:

### Parallelizable Systems (Update Set)

```
┌─────────────────────────────────────────────────────────────┐
│                    Parallel Execution                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Thread 1                Thread 2              Thread 3     │
│     ↓                        ↓                      ↓        │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────┐  │
│  │ ai_update    │      │ scripting    │      │ game     │  │
│  │              │      │              │      │ logic    │  │
│  │ Components:  │      │ Components:  │      │          │  │
│  │  - AIState   │      │  - Script    │      │ Custom   │  │
│  │  - NavAgent  │      │  - Behavior  │      │ systems  │  │
│  └──────────────┘      └──────────────┘      └──────────┘  │
│                                                              │
│  All run in parallel - no shared component access            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Sequential Dependencies

```
┌─────────────────────────────────────────────────────────────┐
│              Sequential (Data Dependencies)                  │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  sync_transforms_to_physics                                 │
│         ↓                                                    │
│  physics_step              ← Must wait for transforms        │
│         ↓                                                    │
│  sync_physics_to_transforms ← Must wait for simulation       │
│         ↓                                                    │
│  handle_collision_events   ← Must wait for sync              │
│                                                              │
│  These systems share Transform and RigidBody components      │
│  Sequential execution ensures data consistency               │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Data Flow Between Systems

### Transform Data Flow

```
Input System
    ↓ (Modify Transform based on input)
sync_parent_child_relationships
    ↓ (Update Children components)
propagate_transforms
    ↓ (Calculate GlobalTransform)
physics_step
    ↓ (Update Transform from physics)
propagate_transforms (again)
    ↓ (Recalculate GlobalTransform)
camera_update
    ↓ (Use GlobalTransform for view matrix)
frustum_culling
    ↓ (Use GlobalTransform for bounds)
generate_render_commands
    ↓ (Use GlobalTransform for model matrix)
Rendering
```

### Lighting Data Flow

```
Light Components
(DirectionalLight, PointLight)
    ↓
gather_lighting_system
    ↓ (Collects into LightingData resource)
update_uniform_buffers
    ↓ (Writes to GPU buffer)
Rendering
    ↓ (Shader reads lighting data)
Fragment Shading
```

## System Configuration

### Defining System Order

```rust
use bevy_ecs::prelude::*;

// Explicit ordering with .before() and .after()
schedule.add_systems((
    system_a.before(system_b),
    system_b.after(system_a),
    system_c,  // Runs in parallel with a and b if no conflicts
));

// Chain for strict sequential order
schedule.add_systems((
    system_a,
    system_b,
    system_c,
).chain());

// System sets
schedule.add_systems(
    (system_a, system_b, system_c)
        .in_set(CoreSystemSet::Update)
);

// Set ordering
schedule.configure_sets((
    CoreSystemSet::Input,
    CoreSystemSet::PreUpdate,
    CoreSystemSet::Update,
    CoreSystemSet::PostUpdate,
).chain());
```

### Component Access Specification

```rust
// Read-only access (multiple systems can read in parallel)
fn read_only_system(query: Query<&Transform>) {
    // Can run in parallel with other readers
}

// Mutable access (exclusive)
fn write_system(mut query: Query<&mut Transform>) {
    // Cannot run in parallel with other Transform access
}

// Mixed access
fn mixed_system(
    positions: Query<&Transform>,           // Read
    mut velocities: Query<&mut Velocity>,   // Write
) {
    // Can run in parallel with other Transform readers
    // Cannot run in parallel with other Velocity writers
}
```

## Common System Patterns

### Fixed Timestep Physics

```rust
fn physics_system(
    mut physics_world: ResMut<PhysicsWorld>,
    time: Res<Time>,
    mut accumulator: Local<f32>,
) {
    const TIMESTEP: f32 = 1.0 / 60.0;  // 60 Hz
    
    *accumulator += time.delta_seconds();
    
    while *accumulator >= TIMESTEP {
        physics_world.step(TIMESTEP);
        *accumulator -= TIMESTEP;
    }
}
```

### Change Detection

```rust
fn on_transform_changed(
    query: Query<(&Transform, &RigidBody), Changed<Transform>>,
) {
    // Only processes entities whose Transform changed this frame
    for (transform, rigid_body) in query.iter() {
        // Sync to physics
    }
}
```

### Entity Lifecycle

```rust
fn cleanup_dead_entities(
    mut commands: Commands,
    query: Query<(Entity, &Health)>,
) {
    for (entity, health) in query.iter() {
        if health.current <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
```

## Performance Considerations

### System Scheduling

1. **Group related systems**: Systems that access similar components benefit from cache locality
2. **Minimize exclusive access**: Use `&T` instead of `&mut T` when possible
3. **Use change detection**: Process only entities with modified components
4. **Avoid global resources**: Exclusive resource access prevents parallelization

### Query Optimization

```rust
// GOOD: Specific filter
fn render_visible(
    query: Query<(&Mesh, &Transform), With<Visible>>
) {
    // Iterates only visible entities
}

// BAD: Manual filtering
fn render_all(
    query: Query<(&Mesh, &Transform, &Visible)>
) {
    for (mesh, transform, visible) in query.iter() {
        if visible.enabled {  // Manual check is slower
            // ...
        }
    }
}
```

### Archetype Awareness

```rust
// Entities with same component set are stored together
// Adding/removing components moves entity to different archetype

// Prefer marker components over option components
#[derive(Component)]
struct Visible;  // GOOD: Add/remove as needed

#[derive(Component)]
struct Visibility(bool);  // BAD: Entity always has it, wastes space
```

## Debug and Profiling

### System Profiling

```rust
use praxis_profiling::*;

fn expensive_system(
    query: Query<&Transform>,
    mut profiler: ResMut<SystemProfiler>,
) {
    let _guard = profiler.scope("expensive_system");
    
    // System logic
    for transform in query.iter() {
        // ...
    }
    
    // Profiler automatically records execution time
}
```

### System Inspection

```bash
# Enable tracing for system execution
RUST_LOG=bevy_ecs::schedule=debug cargo run

# Output shows system execution order and timing
[DEBUG] Running system: sync_parent_child_relationships (0.05ms)
[DEBUG] Running system: propagate_transforms (0.15ms)
[DEBUG] Running systems in parallel: [ai_update, scripting_update] (2.3ms)
```

## Related Documentation

- [ECS Patterns](ecs-patterns.md) - Component and system design patterns
- [ECS Architecture](../concepts/ecs-architecture.md) - Core ECS concepts
- [Engine Lifecycle](engine-lifecycle.md) - Overall engine execution flow
- [Transform Hierarchy](../concepts/transform-hierarchy.md) - Transform system details
- [Physics Guide](../guides/physics.md) - Physics system integration
- [Animation Guide](../guides/animation.md) - Animation system integration
- [Performance Learning Path](../learning-paths/performance.md) - Optimization techniques
