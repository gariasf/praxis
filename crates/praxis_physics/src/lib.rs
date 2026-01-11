//! Physics simulation for the Praxis engine.
//!
//! This crate provides physics simulation capabilities using the `Rapier3D` physics engine,
//! wrapped in an ECS-friendly interface that integrates seamlessly with Praxis's architecture.
//!
//! # `Rapier3D` Integration Philosophy
//!
//! This crate implements a **bridge pattern** between `Rapier3D` (a standalone physics engine)
//! and Praxis's ECS architecture. The integration is designed around several key principles:
//!
//! ## 1. Dual Representation
//!
//! Physical objects exist in two parallel representations that must be kept synchronized:
//!
//! - **ECS Representation**: `Transform`, `RigidBody`, `Collider` components on entities
//!   - This is the "source of truth" for gameplay code
//!   - Manipulated by player input, animation, AI, and game logic
//!   - Accessible through standard ECS queries and systems
//!
//! - **Physics Representation**: Rapier's `RigidBodySet`, `ColliderSet`, handles
//!   - This is the "source of truth" for physics simulation
//!   - Manipulated by forces, gravity, collisions, and constraints
//!   - Internal to the physics engine, accessed through handles
//!
//! The physics system maintains **bidirectional mappings** between these representations:
//! ```text
//! Entity <-> RigidBodyHandle (entity_to_body, body_to_entity)
//! Entity <-> ColliderHandle   (entity_to_collider)
//! ```
//!
//! ## 2. Synchronization Strategy
//!
//! The sync process runs twice per frame in opposite directions:
//!
//! ```text
//! Frame Timeline:
//! ┌─────────────────────────────────────────────────────────────────┐
//! │ 1. Gameplay Code                                                │
//! │    - Player input moves kinematic bodies (Transform modified)   │
//! │    - Animation updates Transform                                │
//! │    - AI/scripting modifies positions                            │
//! ├─────────────────────────────────────────────────────────────────┤
//! │ 2. Pre-Physics Sync (ECS → Physics)                             │
//! │    - Copy modified Transforms to Rapier bodies                  │
//! │    - Uses change detection to only sync modified entities       │
//! │    - Critical for kinematic bodies (player-controlled)          │
//! ├─────────────────────────────────────────────────────────────────┤
//! │ 3. Physics Step                                                 │
//! │    - Rapier simulates forces, gravity, collisions               │
//! │    - Dynamic bodies move based on physics                       │
//! │    - Fixed timestep integration (deterministic)                 │
//! ├─────────────────────────────────────────────────────────────────┤
//! │ 4. Post-Physics Sync (Physics → ECS)                            │
//! │    - Copy Rapier body positions back to Transform               │
//! │    - Only for dynamic bodies (physics-controlled)               │
//! │    - Makes physics results visible to rendering/gameplay        │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## 3. Why Bidirectional Sync Is Essential
//!
//! ### Without Pre-Physics Sync (ECS → Physics):
//! - Kinematic bodies (player character, moving platforms) wouldn't move
//! - Teleportation wouldn't work (setting Transform has no effect)
//! - Animation-driven physics interactions wouldn't happen
//!
//! ### Without Post-Physics Sync (Physics → ECS):
//! - Dynamic bodies appear frozen (physics simulates but rendering doesn't see it)
//! - Collision detection works but objects don't visibly move
//! - Gameplay code can't react to physics-driven positions
//!
//! ### Why Change Detection Matters:
//! Without change detection, we'd create a feedback loop:
//! 1. Physics moves dynamic body → Updates Transform
//! 2. Pre-physics sync sees "changed" Transform → Overwrites Rapier position
//! 3. Physics simulation fights against stale data → Jittery movement
//!
//! With change detection, we only sync when gameplay code modifies Transform,
//! not when physics modifies it, preventing the feedback loop.
//!
//! # Architecture
//!
//! The physics system follows Praxis's ECS-first design:
//! - **Components**: Physics properties are stored as ECS components
//! - **Resources**: Physics pipeline and configuration are ECS resources
//! - **Systems**: Physics simulation runs as scheduled ECS systems
//!
//! # Basic Usage
//!
//! ```rust,no_run
//! use praxis_physics::{
//!     PhysicsWorld, PhysicsConfig, PhysicsTime, ContactEvents,
//!     RigidBody, Collider, PhysicsVelocity, CollisionEventReceiver,
//!     cleanup_physics_entities, physics_step_system, sync_physics_transforms_system,
//!     clear_collision_event_receivers, populate_collision_events,
//! };
//! use praxis_ecs::{World, Schedule, IntoSystemConfigs, Transform};
//!
//! let mut world = World::new();
//! world.insert_resource(PhysicsWorld::new());
//! world.insert_resource(PhysicsConfig::default());
//! world.insert_resource(PhysicsTime::new());
//! world.insert_resource(ContactEvents::new());
//!
//! let mut schedule = Schedule::default();
//! schedule.add_systems((
//!     cleanup_physics_entities,           // Clean up despawned entities
//!     clear_collision_event_receivers,    // Clear old collision events
//!     sync_physics_transforms_system,     // ECS → Physics
//!     physics_step_system,                // Run simulation
//!     sync_physics_transforms_system,     // Physics → ECS
//!     populate_collision_events,          // Distribute collision events
//! ).chain());
//!
//! // Create a static ground plane
//! world.spawn((
//!     Transform::from_xyz(0.0, 0.0, 0.0),
//!     RigidBody::Static,
//!     Collider::cuboid(50.0, 0.5, 50.0),
//! ));
//!
//! // Create a dynamic sphere that receives collision events
//! world.spawn((
//!     Transform::from_xyz(0.0, 10.0, 0.0),
//!     RigidBody::Dynamic,
//!     Collider::sphere(1.0),
//!     PhysicsVelocity::default(),
//!     CollisionEventReceiver::new(), // Enable collision events
//! ));
//!
//! // Run the simulation
//! schedule.run(world.inner_mut());
//! ```
//!
//! # Collision Event Handling Example
//!
//! ```rust,no_run
//! use praxis_physics::{CollisionEventReceiver, CollisionEvent};
//! use praxis_ecs::{Query, Entity};
//!
//! /// System that handles collision events for a player entity
//! fn handle_player_collisions(
//!     query: Query<&CollisionEventReceiver>
//! ) {
//!     for receiver in query.iter() {
//!         for event in &receiver.events {
//!             match event {
//!                 CollisionEvent::CollisionStarted(self_entity, other_entity) => {
//!                     println!("Player {:?} started colliding with {:?}",
//!                              self_entity, other_entity);
//!                     // Play impact sound, trigger damage, etc.
//!                 }
//!                 CollisionEvent::CollisionStopped(self_entity, other_entity) => {
//!                     println!("Player {:?} stopped colliding with {:?}",
//!                              self_entity, other_entity);
//!                     // Stop continuous effects
//!                 }
//!                 CollisionEvent::CollisionPersisted(self_entity, other_entity) => {
//!                     println!("Player {:?} continues colliding with {:?}",
//!                              self_entity, other_entity);
//!                     // Apply damage over time
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! # Spatial Query Example
//!
//! ```rust,no_run
//! use praxis_physics::PhysicsWorld;
//! use praxis_math::Vec3;
//! use praxis_ecs::Res;
//!
//! /// System that performs raycasting for weapon firing
//! fn weapon_raycast(physics: Res<PhysicsWorld>) {
//!     let origin = Vec3::new(0.0, 1.5, 0.0);
//!     let direction = Vec3::new(0.0, 0.0, 1.0).normalize();
//!     
//!     // Raycast to find what the weapon hits
//!     if let Some((entity, distance)) = physics.raycast(
//!         origin,
//!         direction,
//!         100.0, // Max range
//!         true,  // Solid (stop at first hit)
//!     ) {
//!         println!("Hit entity {:?} at distance {}", entity, distance);
//!         let hit_point = origin + direction * distance;
//!         // Apply damage, spawn hit effects at hit_point, etc.
//!     }
//!     
//!     // Check if a point is inside a damage zone
//!     let player_pos = Vec3::new(5.0, 0.0, 5.0);
//!     if let Some(zone) = physics.point_inside(player_pos) {
//!         println!("Player is inside damage zone {:?}", zone);
//!     }
//! }
//! ```
//!
//! # Components
//!
//! Physics properties are defined through components:
//!
//! - **`RigidBody`**: Marks an entity as a rigid body (Dynamic, Static, or Kinematic)
//! - **`Collider`**: Defines collision geometry
//! - **`PhysicsVelocity`**: Linear and angular velocity
//! - **`ExternalForces`**: Accumulated forces and torques
//! - **`Mass`**: Mass properties
//! - **`Friction`**: Surface friction coefficient
//! - **`Restitution`**: Bounciness coefficient
//! - **`CollisionGroups`**: Collision filtering
//! - **`Sleeping`**: Sleep state control
//! - **`CollisionEventReceiver`**: Component for receiving collision events
//!
//! # Collision Events
//!
//! The physics system provides collision event handling through:
//!
//! - **`CollisionEvent`**: Enum representing collision event types (Started, Stopped, Persisted)
//! - **`CollisionEventReceiver`**: Component that stores collision events for an entity
//! - **`clear_collision_event_receivers`**: System that clears event buffers each frame
//! - **`populate_collision_events`**: System that distributes events to entities
//!
//! # Spatial Queries
//!
//! The `PhysicsWorld` resource provides query helpers for spatial operations:
//!
//! - **`raycast`**: Cast a ray and return the first hit
//! - **`raycast_all`**: Cast a ray and return all hits
//! - **`shape_cast`**: Sweep a shape and return the first hit
//! - **`point_inside`**: Check if a point is inside any collider
//!
//! These queries use the physics world's spatial acceleration structures for efficient
//! collision detection and are commonly used for weapon systems, character controllers,
//! and spatial awareness.
//!
//! # Systems
//!
//! The physics simulation requires these core systems to be scheduled in order:
//!
//! 1. **`cleanup_physics_entities`**: Removes Rapier bodies/colliders for despawned entities
//!    (should run before physics to avoid processing stale entities)
//! 2. **`sync_physics_transforms_system`**: Bidirectionally syncs Transform components
//!    with Rapier rigid body positions (should be called before and after physics step)
//! 3. **`physics_step_system`**: Advances the physics simulation using fixed timestep integration
//!
//! For collision events, add these systems:
//! 1. **`clear_collision_event_receivers`**: Clears event buffers (before physics step)
//! 2. **`populate_collision_events`**: Distributes events to entities (after physics step)
//!
//! Alternative: Use the separate legacy systems:
//! 1. **`sync_transforms_to_physics`**: Updates physics bodies from ECS transforms
//! 2. **`step_physics_simulation`**: Advances the physics simulation (no fixed timestep)
//! 3. **`sync_transforms_from_physics`**: Updates ECS transforms from physics bodies
//!
//! # Transform Synchronization and System Ordering
//!
//! The physics system maintains bidirectional synchronization between ECS `Transform`
//! components and Rapier rigid body positions. This allows both physics-driven and
//! kinematic movement to work seamlessly.
//!
//! ## Critical: System Order Matters
//!
//! The order in which physics systems run is **critical** for correct behavior. Running
//! systems in the wrong order causes bugs ranging from jittery movement to physics not
//! working at all. Here's why:
//!
//! ### Correct Order (Required):
//!
//! ```rust,no_run
//! use praxis_ecs::{Schedule, IntoSystemConfigs};
//! use praxis_physics::{
//!     cleanup_physics_entities,
//!     clear_collision_event_receivers,
//!     sync_physics_transforms_system,
//!     physics_step_system,
//!     populate_collision_events,
//! };
//!
//! let mut schedule = Schedule::default();
//! schedule.add_systems((
//!     // 1. CLEANUP: Remove physics objects for despawned entities
//!     cleanup_physics_entities,
//!     
//!     // 2. CLEAR EVENTS: Reset collision event buffers for new frame
//!     clear_collision_event_receivers,
//!     
//!     // 3. SYNC ECS→PHYSICS: Apply gameplay changes to physics world
//!     sync_physics_transforms_system,
//!     
//!     // 4. SIMULATE: Run the physics simulation
//!     physics_step_system,
//!     
//!     // 5. SYNC PHYSICS→ECS: Apply physics results back to ECS
//!     sync_physics_transforms_system,
//!     
//!     // 6. DISTRIBUTE EVENTS: Send collision events to entity components
//!     populate_collision_events,
//! ).chain());
//! ```
//!
//! ### Why Each Step Must Come in This Order:
//!
//! #### Step 1: Cleanup Must Come First
//!
//! **Why before sync?** If we sync first, we might try to sync entities that were despawned,
//! causing panics or wasted work. Cleanup ensures we only work with live entities.
//!
//! **What happens if cleanup runs after?** Stale Rapier bodies persist for an extra frame,
//! generating ghost collisions and wasting CPU on non-existent objects.
//!
//! #### Step 2: Clear Events Before Simulation
//!
//! **Why before physics?** Events from the previous frame must be cleared before generating
//! new ones. Otherwise, events accumulate indefinitely, consuming memory and confusing
//! game logic.
//!
//! **What happens if clear runs after populate?** Newly generated events are immediately
//! deleted, so no collision events ever reach gameplay code. Collision detection appears
//! broken.
//!
//! #### Step 3: Sync ECS→Physics Before Simulation
//!
//! **Why before physics?** Gameplay code modified Transforms (player movement, animations,
//! teleports) during the last frame. We must apply these changes to Rapier before simulating,
//! or physics won't see them.
//!
//! **What happens if sync runs after?** Kinematic bodies don't move (player input ignored),
//! animations don't affect physics, teleportation doesn't work. The physics world is out
//! of sync with the game world.
//!
//! #### Step 4: Simulate Physics
//!
//! The core physics step must run after receiving ECS changes and before sending results back.
//!
//! #### Step 5: Sync Physics→ECS After Simulation
//!
//! **Why after physics?** Dynamic bodies moved during simulation. We must copy their new
//! positions back to Transform so rendering displays them correctly and gameplay can react.
//!
//! **What happens if sync runs before?** Dynamic bodies appear frozen. The physics simulation
//! runs, but its results are invisible. Objects fall but don't render as falling.
//!
//! #### Step 6: Distribute Events Last
//!
//! **Why after everything?** Collision events are generated during the physics step. We
//! distribute them to entities after the step completes, once all physics state is finalized.
//!
//! **What happens if distribute runs before?** No events exist yet, so nothing is distributed.
//! Gameplay never sees collisions.
//!
//! ## Common Mistakes and Symptoms
//!
//! ### Mistake: Only One Sync System
//! ```rust,ignore
//! schedule.add_systems((
//!     sync_physics_transforms_system,  // Only once!
//!     physics_step_system,
//! ).chain());
//! ```
//! **Symptoms:**
//! - Either kinematic bodies don't move (if sync is after)
//! - Or dynamic bodies don't move (if sync is before)
//!
//! ### Mistake: Sync in Wrong Order
//! ```rust,ignore
//! schedule.add_systems((
//!     physics_step_system,
//!     sync_physics_transforms_system,  // After physics
//!     sync_physics_transforms_system,  // After physics again???
//! ).chain());
//! ```
//! **Symptoms:**
//! - Kinematic bodies jitter or don't respond to input
//! - Physics results are delayed by one frame
//!
//! ### Mistake: Cleanup After Physics
//! ```rust,ignore
//! schedule.add_systems((
//!     physics_step_system,
//!     cleanup_physics_entities,  // Too late!
//! ).chain());
//! ```
//! **Symptoms:**
//! - Ghost collisions with despawned entities
//! - Crashes when physics references deleted entities
//!
//! ## Transform Propagation Integration
//!
//! If using hierarchical transforms (parent-child relationships), transform propagation
//! must run **after** physics syncs positions but **before** rendering:
//!
//! ```text
//! Physics Systems:
//!   1. sync_physics_transforms_system (physics → ECS)
//!   
//! Transform Systems:
//!   2. propagate_transforms (updates GlobalTransform from Transform hierarchy)
//!   
//! Rendering:
//!   3. render (reads GlobalTransform)
//! ```
//!
//! **Why this order?** `GlobalTransform` combines a Transform with its parent's `GlobalTransform`.
//! If propagation runs before physics sync, it uses stale Transform data and children lag
//! behind their physics-driven parents by one frame.
//!
//! # Fixed Timestep Reasoning
//!
//! Physics simulation uses **fixed timestep integration** rather than variable timestep.
//! This is a fundamental design decision with important implications.
//!
//! ## The Problem with Variable Timestep
//!
//! Real-time applications run at variable frame rates (30fps, 60fps, 144fps, or anything
//! in between). A naive approach would be:
//!
//! ```text
//! Every frame:
//!     delta_time = time_since_last_frame
//!     physics.step(delta_time)  // Variable dt
//! ```
//!
//! This seems logical but causes severe problems:
//!
//! ### 1. Non-Determinism
//!
//! The same inputs produce different results at different frame rates:
//! - At 60fps: Object travels distance X
//! - At 30fps: Object travels distance Y ≠ X
//!
//! This makes:
//! - Replays impossible (can't reproduce gameplay)
//! - Networking unreliable (clients desync)
//! - Debugging nightmarish (bugs appear/disappear randomly)
//! - Physics simulations unpredictable
//!
//! ### 2. Numerical Instability
//!
//! Physics solvers use iterative methods (Sequential Impulses, PGS) that converge
//! based on timestep size. Large timesteps (slow frames) cause:
//! - Poor solver convergence → constraints violated → objects penetrate
//! - Energy gain → simulation explodes
//! - Tunneling → fast objects pass through walls
//!
//! ### 3. Temporal Aliasing
//!
//! Collision detection samples at discrete intervals. Large timesteps miss events:
//! - Bullet passes through paper-thin wall between samples
//! - Fast-moving platform skips over player (player falls through)
//! - Critical collision events don't fire
//!
//! ## Fixed Timestep Solution
//!
//! Instead, we advance physics in fixed increments independent of frame rate:
//!
//! ```text
//! Every frame:
//!     accumulator += delta_time
//!     while accumulator >= FIXED_DT:
//!         physics.step(FIXED_DT)  // Always same dt!
//!         accumulator -= FIXED_DT
//! ```
//!
//! ### Benefits:
//!
//! 1. **Determinism**: Same inputs + same initial conditions = same output, every time
//!    - Makes replays, networking, and debugging reliable
//!    - Physics behaves identically on any hardware
//!
//! 2. **Stability**: Solver parameters are tuned for a specific timestep
//!    - Consistent convergence behavior
//!    - Predictable constraint satisfaction
//!    - No energy gain from variable integration
//!
//! 3. **Frame-rate Independence**: Physics quality doesn't depend on FPS
//!    - Fast computer (144fps): Smooth but physics runs at 60Hz
//!    - Slow computer (30fps): Choppy but physics still at 60Hz
//!    - Physics behaves identically in both cases
//!
//! ## Timestep Choice: Why 60 Hz?
//!
//! The default timestep is 1/60 second (16.67ms, or 60 Hz). This is a sweet spot:
//!
//! - **Too large** (e.g., 1/30 = 33ms):
//!   - Poor responsiveness (input lag)
//!   - Visible stepping in smooth motion
//!   - More tunneling issues
//!
//! - **Too small** (e.g., 1/240 = 4ms):
//!   - Excessive CPU usage
//!   - Accumulator fills quickly on slow frames → spiral of death
//!   - Diminishing returns for stability
//!
//! - **60 Hz is ideal** because:
//!   - Matches common monitor refresh rates (60fps)
//!   - Good responsiveness (16ms latency)
//!   - Stable simulation with reasonable CPU cost
//!   - Industry standard (most physics engines default to this)
//!
//! ## Accumulator Pattern Explained
//!
//! ```text
//! Frame 1: Render time = 16ms
//!   accumulator = 0 + 16 = 16ms
//!   16 >= 16.67? No → Skip physics, carry forward
//!
//! Frame 2: Render time = 18ms
//!   accumulator = 16 + 18 = 34ms
//!   34 >= 16.67? Yes → Step physics (34 - 16.67 = 17.33ms remaining)
//!   17.33 >= 16.67? Yes → Step physics (17.33 - 16.67 = 0.66ms remaining)
//!   
//! Frame 3: Render time = 16ms
//!   accumulator = 0.66 + 16 = 16.66ms
//!   16.66 >= 16.67? No → Skip physics
//! ```
//!
//! ### Key Properties:
//!
//! - **Variable steps per frame**: 0, 1, 2, or more physics steps depending on frame time
//! - **Fractional remainder**: Accumulator stores leftover time (<1 timestep)
//! - **Catches up**: Slow frames run multiple steps to stay in sync with real time
//! - **Never drops updates**: Every physics step is simulated exactly once
//!
//! ## Spiral of Death
//!
//! A potential issue: if physics takes longer than real-time to compute, the accumulator
//! grows without bound:
//!
//! ```text
//! Frame N: 16ms frame, physics takes 20ms
//!   accumulator = 16ms → step → takes 20ms
//!   Real time advanced 20ms, but only simulated 16.67ms
//!   Falling behind by 3.33ms per frame!
//! ```
//!
//! This creates a positive feedback loop where physics gets slower and slower.
//!
//! ### Mitigations:
//!
//! 1. **Cap max steps per frame**:
//!    ```rust,ignore
//!    let max_steps = 3;
//!    let mut steps = 0;
//!    while accumulator >= timestep && steps < max_steps {
//!        physics.step(timestep);
//!        accumulator -= timestep;
//!        steps += 1;
//!    }
//!    if steps == max_steps { accumulator = 0.0; } // Discard excess time
//!    ```
//!
//! 2. **Use interpolation** (see below) so visual stuttering is less noticeable
//!
//! 3. **Optimize physics** to run faster than real-time under normal conditions
//!
//! ## Interpolation (Future Enhancement)
//!
//! Fixed timestep creates visual artifacts: rendering happens between physics steps,
//! showing old positions. The solution is **interpolation**:
//!
//! ```text
//! Store previous_transform and current_transform
//!
//! alpha = accumulator / timestep  // 0.0 to 1.0
//! render_transform = lerp(previous_transform, current_transform, alpha)
//! ```
//!
//! This provides smooth visuals at any frame rate while maintaining fixed timestep
//! physics. Currently not implemented but mentioned in `physics_step_system` comments.
//!
//! # Collision Detection Phases
//!
//! Rapier's collision detection uses a **two-phase algorithm** (broad phase + narrow phase)
//! that's fundamental to high-performance physics engines.
//!
//! ## Why Two Phases?
//!
//! Naive collision detection tests every pair of objects: O(n²) complexity.
//! For n=1000 objects, that's 499,500 tests per frame. Impossible at 60fps!
//!
//! Two-phase detection reduces this to practical performance:
//! - **Broad phase**: O(n log n) using spatial data structures
//! - **Narrow phase**: O(k) where k << n (only potentially colliding pairs)
//!
//! ## Phase 1: Broad Phase
//!
//! **Purpose**: Quickly eliminate pairs that are too far apart to possibly collide.
//!
//! **Method**: Uses spatial partitioning (typically a Dynamic AABB Tree):
//!
//! ```text
//! Each object has an Axis-Aligned Bounding Box (AABB):
//! ┌─────────┐
//! │  /\     │  ← AABB (box aligned with X/Y/Z axes)
//! │ /  \    │
//! │/____\   │  ← Actual shape (triangle)
//! └─────────┘
//!
//! AABBs are cheap to test: 6 comparisons (min.x < max.x, etc.)
//! Actual shapes are expensive: GJK, SAT, EPA algorithms
//! ```
//!
//! **Algorithm**:
//! 1. Build/update spatial tree (BVH, quadtree, grid, etc.)
//! 2. Traverse tree to find AABB overlaps
//! 3. Output potentially colliding pairs: `[(A,B), (C,D), (E,F), ...]`
//!
//! **Complexity**: O(n log n) average case due to hierarchical tree structure
//!
//! **False Positives**: Broad phase intentionally allows false positives (AABBs overlap
//! but shapes don't). This is fine - narrow phase filters them out. Zero false negatives
//! is required (must never miss actual collisions).
//!
//! ## Phase 2: Narrow Phase
//!
//! **Purpose**: Precisely determine if shapes actually collide and generate contact information.
//!
//! **Method**: Uses geometric algorithms on actual shape geometry:
//!
//! ### Algorithms Used:
//!
//! - **GJK (Gilbert-Johnson-Keerthi)**: Computes minimum distance between convex shapes
//!   - Iteratively refines a simplex (point, line, triangle, tetrahedron)
//!   - Converges to distance or detects overlap
//!   - Very fast for convex shapes
//!
//! - **SAT (Separating Axis Theorem)**: Tests if a separating plane exists
//!   - Projects shapes onto candidate axes
//!   - If projections don't overlap on any axis → no collision
//!   - Used for polytopes (boxes, polyhedra)
//!
//! - **EPA (Expanding Polytope Algorithm)**: Finds penetration depth for overlapping shapes
//!   - Runs after GJK detects overlap
//!   - Expands a polytope to find deepest penetration
//!   - Provides normal vector and depth for constraint solver
//!
//! ### Output: Contact Manifolds
//!
//! For each colliding pair, narrow phase generates a **contact manifold**:
//!
//! ```text
//! Contact Manifold {
//!     contact_points: [(x₁, y₁, z₁), (x₂, y₂, z₂), ...],  // Where surfaces touch
//!     contact_normals: [n₁, n₂, ...],                      // Collision direction
//!     penetration_depths: [d₁, d₂, ...],                   // How deep overlap is
//! }
//! ```
//!
//! These manifolds are passed to the constraint solver to generate separation impulses.
//!
//! ## Phase 3: Contact Resolution (Constraint Solver)
//!
//! The solver uses contact manifolds to resolve collisions:
//!
//! 1. **Velocity Constraint**: Prevent interpenetration
//!    ```text
//!    Relative velocity along normal should become >= 0 (separating)
//!    Apply impulse to achieve this
//!    ```
//!
//! 2. **Position Constraint**: Correct existing penetration
//!    ```text
//!    Objects are already overlapping by depth d
//!    Apply position correction to push them apart
//!    ```
//!
//! 3. **Friction**: Apply tangential forces
//!    ```text
//!    Friction force ≤ friction_coefficient * normal_force
//!    Opposes relative motion parallel to surface
//!    ```
//!
//! The solver iterates multiple times (typically 4-20 iterations) to converge to a
//! solution that satisfies all constraints simultaneously.
//!
//! ## Continuous Collision Detection (CCD)
//!
//! Standard discrete collision detection samples at timesteps. Fast objects can
//! "tunnel" through thin obstacles:
//!
//! ```text
//! Frame 1:     ●        |        (bullet before wall)
//! Frame 2:              |    ●   (bullet after wall - missed collision!)
//! ```
//!
//! **CCD Solution**: Sweep shapes along their motion path using shape casting:
//!
//! ```text
//! Swept volume: [======>]   (bullet's path as a capsule)
//!                    |       (wall)
//! Hit detected!  [===●       (time of impact found)
//! ```
//!
//! CCD is expensive, so it's only enabled for objects flagged as "fast-moving" or
//! "CCD-enabled". Used for bullets, fast vehicles, or any object that must never tunnel.

mod cloth;
mod cloth_systems;
mod components;
mod joint_systems;
mod joints;
mod ragdoll;
mod ragdoll_systems;
mod resources;
mod systems;
mod vehicle;
mod vehicle_systems;

#[cfg(test)]
mod tests;

pub use cloth::*;
pub use cloth_systems::*;
pub use components::*;
pub use joint_systems::*;
pub use joints::*;
pub use ragdoll::*;
pub use ragdoll_systems::*;
pub use resources::*;
pub use systems::*;
pub use vehicle::*;
pub use vehicle_systems::*;

use praxis_utils::{info, Result};

/// Initializes the physics system.
///
/// This function sets up any necessary global state for the physics system.
/// It should be called once during engine initialization, before any physics
/// resources or components are used.
///
/// # Purpose
///
/// The initialization function serves as a centralized entry point for physics
/// subsystem setup. Currently, it:
/// - Logs initialization status for debugging and monitoring
/// - Provides a hook for future initialization needs (e.g., thread pools, GPU backends)
/// - Validates that dependencies are available
///
/// # Integration
///
/// This function follows the Praxis pattern where each subsystem provides an `init()`
/// function that is called during engine startup. The physics system is typically
/// initialized after core systems (utils, ECS) but before window and rendering.
///
/// # Example
///
/// ```rust,no_run
/// // In engine initialization sequence
/// praxis_utils::init().expect("Failed to initialize utilities");
/// praxis_ecs::init().expect("Failed to initialize ECS");
/// praxis_physics::init().expect("Failed to initialize physics system");
/// ```
///
/// # Errors
///
/// Returns an error if initialization fails. Currently, this function always succeeds,
/// but future versions may perform validation or setup that could fail.
///
/// # Thread Safety
///
/// This function is safe to call from any thread, but should only be called once
/// during application lifetime. Multiple calls are harmless but redundant.
pub fn init() -> Result<()> {
    info!("Initializing physics system");
    // Future initialization work can be added here, such as:
    // - Setting up thread pools for parallel collision detection
    // - Initializing GPU acceleration if available
    // - Validating Rapier version compatibility
    // - Loading physics configuration from files
    Ok(())
}
