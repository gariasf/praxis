//! Physics systems for the Praxis ECS.
//!
//! This module provides systems that integrate Rapier physics simulation
//! with the Praxis ECS architecture.

use praxis_ecs::{Commands, Entity, Query, Res, ResMut, Transform, With, Changed};
use praxis_math::{Quat, Vec3};
use rapier3d::prelude::{
    ColliderBuilder, Isometry, RigidBodyBuilder, RigidBodyType, SharedShape, vector, nalgebra,
};

use crate::components::{
    Collider as PraxisCollider, ExternalForces, Friction, Restitution,
    RigidBody as PraxisRigidBody, Sensor, PhysicsVelocity,
};
use crate::resources::{PhysicsWorld, PhysicsConfig, ContactEvents, PhysicsTime};

/// Advances the Rapier physics simulation by one fixed timestep.
///
/// This system implements **fixed timestep integration**, a fundamental technique in
/// physics simulation where the simulation always advances by a constant time interval
/// regardless of the actual frame rate. This is critical for deterministic, stable physics.
///
/// # Fixed Timestep Integration Strategy
///
/// ## Why Fixed Timesteps?
///
/// Real-time applications run at variable frame rates depending on hardware performance,
/// scene complexity, and system load. However, physics simulations are highly sensitive
/// to timestep size:
///
/// - **Large timesteps** lead to instability, tunneling (fast objects passing through walls),
///   and energy gain in the simulation
/// - **Variable timesteps** make the simulation non-deterministic, meaning the same inputs
///   can produce different results depending on frame rate
/// - **Small, fixed timesteps** provide stability, determinism, and consistent behavior
///
/// ## Implementation Approach
///
/// This system uses the **accumulator pattern**:
///
/// 1. **Accumulation**: Each frame's actual delta time is added to an accumulator
/// 2. **Fixed Steps**: While the accumulator has enough time (>= fixed timestep),
///    we run one physics step and subtract the timestep from the accumulator
/// 3. **Remainder**: Any remaining time less than one timestep stays in the accumulator
///    for the next frame
///
/// ### Example Timeline:
///
/// ```text
/// Frame 1: dt = 16ms, accumulator = 16ms
///   → Run 1 step (16.67ms fixed), accumulator = -0.67ms (carried over as 0)
///
/// Frame 2: dt = 20ms, accumulator = 20ms
///   → Run 1 step (16.67ms fixed), accumulator = 3.33ms (carry forward)
///
/// Frame 3: dt = 16ms, accumulator = 19.33ms (3.33 + 16)
///   → Run 1 step (16.67ms fixed), accumulator = 2.66ms (carry forward)
///
/// Frame 4: dt = 50ms, accumulator = 52.66ms (2.66 + 50)
///   → Run 3 steps (50ms total), accumulator = 2.66ms (carry forward)
/// ```
///
/// ## Benefits
///
/// - **Determinism**: Same initial conditions + same inputs = same results every time
/// - **Stability**: Solver convergence and constraint satisfaction work properly
/// - **Frame-rate independence**: Physics behaves identically at 30fps, 60fps, or 144fps
/// - **Temporal coherence**: Objects don't suddenly jump or jitter due to variable dt
///
/// ## Trade-offs
///
/// - **Performance spikes**: Slow frames may run multiple physics steps to catch up
/// - **Spiral of death**: If physics can't keep up with real-time, accumulator grows unbounded
///   (mitigated by clamping max steps per frame or using interpolation)
///
/// # Rapier Pipeline
///
/// The physics step executes these stages (handled internally by Rapier):
///
/// 1. **Broad Phase**: Spatial partitioning to find potentially colliding pairs (AABB tests)
/// 2. **Narrow Phase**: Precise collision detection for candidate pairs (GJK/SAT algorithms)
/// 3. **Island Management**: Group connected bodies for efficient solving
/// 4. **Solver**: Iteratively resolve constraints (contacts, joints) using Sequential Impulses
/// 5. **Integration**: Update positions/velocities based on forces, using symplectic Euler
/// 6. **CCD**: Continuous collision detection for fast-moving objects to prevent tunneling
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{Schedule, IntoSystemConfigs};
/// use praxis_physics::{physics_step_system, sync_physics_transforms_system};
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems((
///     // Sync ECS transforms to Rapier before physics (user input, animations)
///     sync_physics_transforms_system,
///     // Advance the simulation
///     physics_step_system,
///     // Sync Rapier results back to ECS transforms
///     sync_physics_transforms_system,
/// ).chain());
/// ```
///
/// # System Requirements
///
/// - **Resources**: `PhysicsWorld` (`ResMut`), `PhysicsConfig` (`Res`), `PhysicsTime` (`ResMut`),
///   `ContactEvents` (`ResMut`)
/// - **Ordering**: Should run after user input and before rendering
#[allow(clippy::needless_pass_by_value)]
pub fn physics_step_system(
    mut physics_world: ResMut<PhysicsWorld>,
    config: Res<PhysicsConfig>,
    mut physics_time: ResMut<PhysicsTime>,
    mut contact_events: ResMut<ContactEvents>,
) {
    // ========================================================================
    // FIXED TIMESTEP INTEGRATION LOOP
    // ========================================================================
    //
    // The accumulator pattern ensures physics runs at a fixed rate independent
    // of frame rate. This is essential for stability and determinism.
    //
    // The loop may run 0, 1, or multiple times per frame depending on how much
    // time has accumulated since the last frame:
    //
    // - Fast frames (< timestep): No steps, just accumulate time
    // - Normal frames (~timestep): Usually 1 step
    // - Slow frames (>> timestep): Multiple steps to catch up
    //
    // Note: In production, you might want to clamp the number of steps to
    // prevent the "spiral of death" where physics can't keep up with real-time.
    // For example: `let max_steps = 3;` and break after max_steps iterations.
    
    while physics_time.should_step(config.timestep) {
        // Clear previous frame's collision events at the start of each physics step.
        // These events are generated during the narrow phase and should be consumed
        // by gameplay systems before the next step.
        contact_events.clear();

        // ====================================================================
        // CONFIGURE INTEGRATION PARAMETERS
        // ====================================================================
        //
        // Update Rapier's integration parameters with the current timestep.
        // This dt value controls:
        // - How far ahead in time to integrate (position += velocity * dt)
        // - Constraint solver time budget (impulses are scaled by dt)
        // - Damping application (exponential decay over dt)
        
        physics_world.integration_parameters.dt = config.timestep;

        // Convert gravity from Praxis Vec3 to Rapier's nalgebra vector type.
        // Gravity is an acceleration (units/second²) applied to all dynamic bodies.
        // Common values:
        // - Earth: (0, -9.81, 0)
        // - Moon: (0, -1.62, 0)
        // - Space: (0, 0, 0)
        let gravity = vector![config.gravity.x, config.gravity.y, config.gravity.z];

        // Event handler for collision/contact events.
        // Currently using unit type () as we're not processing events yet.
        // In a full implementation, this would capture collision started/ended events
        // and populate the ContactEvents resource.
        let event_handler = ();

        // ====================================================================
        // EXECUTE PHYSICS PIPELINE
        // ====================================================================
        //
        // The physics pipeline is the heart of the simulation. It orchestrates
        // all the complex stages needed to simulate rigid body dynamics:
        //
        // **Broad Phase**: Uses spatial partitioning (typically a dynamic AABB tree)
        //   to quickly identify pairs of objects whose AABBs overlap. This culls
        //   the vast majority of non-colliding pairs in O(n log n) time.
        //
        // **Narrow Phase**: For AABB-overlapping pairs, performs precise collision
        //   detection using algorithms like GJK (Gilbert-Johnson-Keerthi) for
        //   distance queries and SAT (Separating Axis Theorem) for polytopes.
        //   Generates contact manifolds (points, normals, penetration depths).
        //
        // **Island Management**: Groups bodies connected by contacts or joints into
        //   "islands" that can be solved independently. Sleeping islands (at rest)
        //   are skipped entirely for performance.
        //
        // **Constraint Solver**: The Sequential Impulse solver iteratively resolves
        //   all constraints (contacts and joints) by applying impulses that push
        //   bodies apart and enforce joint limits. Runs for a configurable number
        //   of iterations (more = more accurate but slower).
        //
        // **Integration**: Uses symplectic Euler integration to update positions
        //   and velocities based on forces and impulses. Symplectic integration
        //   preserves energy better than standard Euler, reducing drift.
        //
        // **CCD**: Continuous Collision Detection uses swept shape tests to detect
        //   collisions for fast-moving objects that might tunnel through thin
        //   obstacles at discrete timesteps.
        //
        // The pipeline step is destructured to avoid borrowing issues with the
        // PhysicsWorld resource (we need mutable references to multiple fields).

        let PhysicsWorld {
            ref mut rigid_body_set,
            ref mut collider_set,
            ref integration_parameters,
            ref mut physics_pipeline,
            ref mut island_manager,
            ref mut broad_phase,
            ref mut narrow_phase,
            ref mut impulse_joint_set,
            ref mut multibody_joint_set,
            ref mut ccd_solver,
            ref mut query_pipeline,
            ..
        } = *physics_world;
        
        // Execute one complete physics step advancing the simulation by dt.
        // This call performs all the stages described above.
        physics_pipeline.step(
            &gravity,                    // Constant acceleration applied to all dynamic bodies
            integration_parameters,      // Timestep, solver iterations, damping, etc.
            island_manager,              // Groups connected bodies for efficient solving
            broad_phase,                 // Spatial partitioning for collision culling
            narrow_phase,                // Precise collision detection
            rigid_body_set,              // All rigid bodies in the simulation
            collider_set,                // All collision shapes
            impulse_joint_set,           // Constraints connecting bodies
            multibody_joint_set,         // Articulated joint hierarchies
            ccd_solver,                  // Continuous collision detection
            Some(query_pipeline),        // Optional: update spatial queries (raycasts, etc.)
            &(),                         // Physics hooks (custom collision filtering)
            &event_handler,              // Collision event callbacks
        );

        // ====================================================================
        // UPDATE TIME ACCUMULATOR
        // ====================================================================
        //
        // Consume one fixed timestep from the accumulator. This ensures that
        // we only step the physics simulation when enough time has accumulated.
        //
        // If there's still time remaining in the accumulator after this step,
        // the loop will run again. If not enough time remains, we exit and
        // carry the remainder forward to the next frame.
        //
        // This is what makes the system fixed timestep: we always step by
        // exactly config.timestep seconds, never more or less.
        
        physics_time.step(config.timestep);
    }

    // ========================================================================
    // INTERPOLATION NOTE (Not Implemented)
    // ========================================================================
    //
    // The fixed timestep approach has a subtle problem: visual stuttering.
    // Because we only update physics at discrete intervals, rendering happens
    // at times that don't align with physics steps:
    //
    // ```text
    // Physics:  |----P----|----|----P----|----|----P----|
    // Render:   |-R--|-R--|-R--|-R--|-R--|-R--|-R--|-R--|
    //              ↑ Render uses stale physics state!
    // ```
    //
    // **Solution: Interpolation**
    //
    // Calculate an interpolation factor (alpha) from the accumulator:
    // ```rust
    // let alpha = physics_time.accumulator / config.timestep;
    // ```
    //
    // Then in rendering, interpolate between the previous and current physics
    // states:
    // ```rust
    // render_position = previous_position.lerp(current_position, alpha);
    // render_rotation = previous_rotation.slerp(current_rotation, alpha);
    // ```
    //
    // This provides smooth visuals at any frame rate while maintaining fixed
    // timestep physics. Requires storing previous transforms, which we've
    // omitted for simplicity.
}

/// Bidirectionally synchronizes Transform components with Rapier rigid body states.
///
/// This system handles the **critical coupling** between the ECS representation (Transform)
/// and the physics simulation representation (Rapier rigid bodies). It must run twice per
/// frame with different responsibilities each time.
///
/// # Why Bidirectional Synchronization?
///
/// Physics engines and ECS systems maintain separate representations of object state:
///
/// - **ECS**: Stores transforms in `Transform` components, manipulated by gameplay code,
///   animation systems, and player input
/// - **Physics**: Stores transforms in Rapier rigid bodies, manipulated by physical forces,
///   collisions, and constraints
///
/// These representations must be kept in sync, but the synchronization direction depends
/// on the body type and the phase of the frame:
///
/// ## ECS → Physics (Before Physics Step)
///
/// **Purpose**: Apply non-physics changes to the simulation
///
/// - **Dynamic bodies**: Initial position/velocity for newly spawned objects
/// - **Kinematic bodies**: User-controlled movement (character controllers, moving platforms)
/// - **Static bodies**: Level geometry repositioning (rare, but possible)
///
/// Without this sync, kinematic bodies wouldn't move and dynamic bodies couldn't be
/// teleported or have initial velocities set from gameplay code.
///
/// ## Physics → ECS (After Physics Step)
///
/// **Purpose**: Propagate physics simulation results to the game world
///
/// - **Dynamic bodies**: Updated positions from forces, gravity, and collisions
/// - **Kinematic bodies**: Don't update (they're controlled by ECS)
/// - **Static bodies**: Don't update (they never move)
///
/// Without this sync, dynamic objects would appear frozen even though the physics
/// simulation is running.
///
/// # Change Detection Strategy
///
/// Using Bevy ECS's change detection (`Changed<Transform>`) is crucial for performance:
///
/// ## Problem Without Change Detection
///
/// If we naively sync all transforms every frame:
/// 1. Physics updates dynamic body transform in Rapier
/// 2. We copy it to ECS Transform (marks it as "changed")
/// 3. Next frame's pre-physics sync sees "changed" and copies it back to Rapier
/// 4. This overwrites the physics state with slightly stale data
/// 5. Physics simulation jitters or behaves incorrectly
///
/// ## Solution With Change Detection
///
/// By querying `Changed<Transform>`, we only sync transforms that were modified by
/// non-physics systems (gameplay, animation, input):
///
/// 1. Physics updates dynamic body transform in Rapier
/// 2. We copy it to ECS Transform (marks it as "changed")
/// 3. Next frame: Physics step hasn't run yet, so physics results aren't visible
/// 4. Pre-physics sync: `Changed<Transform>` returns nothing for physics-updated entities
///    (because we haven't read them yet, and change detection is frame-based)
/// 5. Physics step runs and updates positions
/// 6. Post-physics sync: We copy results back, marking as changed
/// 7. Rendering sees the updated transforms
///
/// **Key insight**: We only sync ECS→Physics for transforms that were modified by
/// gameplay code, not by the previous physics step.
///
/// # Implementation Details
///
/// ## Rigid Body Creation
///
/// When an entity has a `RigidBody` component but no corresponding Rapier body exists,
/// we create one with the entity's current transform. This handles:
/// - Newly spawned entities
/// - Entities that just had a `RigidBody` component added
///
/// ## Kinematic Body Positioning
///
/// Kinematic bodies use `set_position()` with `wake_up=true` to:
/// - Update their position in the simulation
/// - Wake up any dynamic bodies in contact with them
/// - Trigger proper collision resolution
///
/// This is the correct way to move kinematic bodies - don't set velocity unless
/// you're using velocity-based kinematic mode.
///
/// ## Dynamic Body Updates
///
/// After the physics step, we read the Rapier body's position and update the ECS
/// Transform. We only do this for dynamic bodies because:
/// - Static bodies never move
/// - Kinematic bodies are controlled by ECS, not physics
///
/// ## Rotation Conversion
///
/// Rapier uses unit quaternions internally but stores them as axis-angle vectors
/// (scaled axis where magnitude = angle). We convert these to `glam::Quat` for ECS:
///
/// ```text
/// Rapier rotation: scaled_axis = axis * angle (Vec3 where |v| = angle)
/// Praxis rotation: quaternion (w, x, y, z)
/// ```
///
/// # Example Integration
///
/// ```rust,no_run
/// use praxis_ecs::{Schedule, IntoSystemConfigs};
/// use praxis_physics::{physics_step_system, sync_physics_transforms_system};
///
/// let mut schedule = Schedule::default();
///
/// // Run the sync system twice with the physics step in between
/// schedule.add_systems((
///     // 1. Sync gameplay changes to physics (kinematic movement, teleports)
///     sync_physics_transforms_system,
///     
///     // 2. Run the physics simulation
///     physics_step_system,
///     
///     // 3. Sync physics results back to ECS (dynamic body movement)
///     sync_physics_transforms_system,
/// ).chain());
/// ```
///
/// # Performance Considerations
///
/// - **Change detection**: Only syncs modified transforms, not all of them
/// - **Batch creation**: Creates Rapier bodies on-demand as entities are spawned
/// - **No cloning**: Transforms are copied by value (they're small)
/// - **Cache-friendly**: Sequential iteration over query results
///
/// # System Requirements
///
/// - **Resources**: `PhysicsWorld` (`ResMut`)
/// - **Query**: Entities with `Transform` and `RigidBody` components
/// - **Change Detection**: Uses `Changed<Transform>` for ECS→Physics sync
#[allow(clippy::needless_pass_by_value)]
pub fn sync_physics_transforms_system(
    mut physics_world: ResMut<PhysicsWorld>,
    // Query for entities that have their Transform changed (by gameplay code)
    // This is used for ECS → Physics synchronization
    changed_query: Query<(Entity, &Transform, &PraxisRigidBody), Changed<Transform>>,
    // Query for ALL entities with RigidBody components
    // This is used for Physics → ECS synchronization (for dynamic bodies)
    mut all_query: Query<(Entity, &mut Transform, &PraxisRigidBody), With<PraxisRigidBody>>,
) {
    // ========================================================================
    // PHASE 1: ECS → PHYSICS SYNCHRONIZATION
    // ========================================================================
    //
    // Update Rapier rigid body positions for entities whose Transforms were
    // modified by non-physics systems (gameplay code, animations, input).
    //
    // This phase runs BEFORE the physics step to ensure that kinematic bodies
    // move correctly and dynamic bodies can be teleported or have initial
    // velocities set.
    //
    // **Why use Changed<Transform>?**
    //
    // Without change detection, we'd sync ALL transforms to physics every frame,
    // which would overwrite the physics simulation's results for dynamic bodies.
    // With change detection, we only sync transforms that were actually modified
    // by gameplay code, allowing physics to control dynamic bodies.
    //
    // **What counts as "changed"?**
    //
    // Bevy tracks changes at the component level. A Transform is marked as
    // changed when:
    // - The entity was just spawned with a Transform
    // - Any system with mutable access (&mut Transform) wrote to it
    // - This includes: player input, animation, AI, procedural movement, etc.
    //
    // **Change detection lifecycle:**
    //
    // 1. Frame N: Physics updates dynamic body Transform
    // 2. Frame N: Transform marked as "changed" (by physics system)
    // 3. Frame N+1: Changed<Transform> sees it as changed
    // 4. Frame N+1: We sync it to physics (overwriting with current value)
    // 
    // Wait, that sounds like it would cause a feedback loop! Why doesn't it?
    //
    // Because we run this system TWICE per frame:
    // - First time (before physics): Changed includes gameplay changes
    // - Second time (after physics): Changed includes nothing new (same frame)
    //
    // The actual flow is:
    // 1. Frame N: User moves kinematic body → Transform marked changed
    // 2. Frame N: Sync #1 (before physics) → Copies changed Transform to Rapier
    // 3. Frame N: Physics step → Dynamic bodies move
    // 4. Frame N: Sync #2 (after physics) → Copies dynamic positions to Transform
    // 5. Frame N: Transforms marked changed by sync #2
    // 6. Frame N+1: Sync #1 (before physics) → Changed<Transform> is empty!
    //               (because no gameplay systems ran between sync #2 and sync #1)
    //
    // This works because:
    // - We run sync before AND after physics in the same frame
    // - Change detection is cleared at the end of each frame
    // - Gameplay code runs before the first sync or after the second sync

    for (entity, transform, rigid_body) in changed_query.iter() {
        // --------------------------------------------------------------------
        // GET OR CREATE RAPIER RIGID BODY HANDLE
        // --------------------------------------------------------------------
        //
        // Each ECS entity with a RigidBody component needs a corresponding
        // Rapier rigid body. We maintain a mapping between Entity IDs and
        // RigidBodyHandles in the PhysicsWorld resource.
        
        let body_handle = if let Some(handle) = physics_world.get_body_handle(entity) {
            // Body already exists, use it
            handle
        } else {
            // ------------------------------------------------------------
            // CREATE NEW RAPIER RIGID BODY
            // ------------------------------------------------------------
            //
            // This entity has a RigidBody component but no Rapier body yet.
            // This happens when:
            // - The entity was just spawned this frame
            // - A RigidBody component was just added to an existing entity
            //
            // We create the appropriate Rapier body type based on the
            // component's variant.
            
            let rapier_body_type = match rigid_body {
                PraxisRigidBody::Dynamic => RigidBodyType::Dynamic,
                PraxisRigidBody::Static => RigidBodyType::Fixed,
                PraxisRigidBody::Kinematic => RigidBodyType::KinematicPositionBased,
            };

            // Convert ECS transform to Rapier's position representation.
            //
            // **Rotation conversion:** 
            // Rapier internally uses unit quaternions but the builder expects
            // an axis-angle vector. We convert glam::Quat to nalgebra AxisAngle:
            //
            // 1. Extract the quaternion's imaginary part (x, y, z) = axis * sin(angle/2)
            // 2. Extract the real part w = cos(angle/2)
            // 3. Convert to axis-angle by scaling: scaled_axis = (x,y,z) * angle
            //
            // This is equivalent to: angle = 2 * acos(w), axis = (x,y,z)/sin(angle/2)
            //
            // For small angles: axis ≈ (x,y,z) and angle ≈ 2*sqrt(x²+y²+z²)
            //
            // **Why this works:**
            // Rapier's rotation() method takes a scaled axis vector where the
            // magnitude is the rotation angle and the direction is the axis.
            // Multiplying the imaginary part by w gives us this representation.

            let rapier_body = RigidBodyBuilder::new(rapier_body_type)
                .translation(vector![
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z
                ])
                .rotation(vector![
                    transform.rotation.x,
                    transform.rotation.y,
                    transform.rotation.z
                ] * transform.rotation.w)
                .build();

            // Insert the body into Rapier's rigid body set and create bidirectional
            // mappings between Entity and RigidBodyHandle for fast lookups.
            let handle = physics_world.rigid_body_set.insert(rapier_body);
            physics_world.entity_to_body.insert(entity, handle);
            physics_world.body_to_entity.insert(handle, entity);
            handle
        };

        // --------------------------------------------------------------------
        // UPDATE KINEMATIC BODY POSITIONS
        // --------------------------------------------------------------------
        //
        // Kinematic bodies are moved by setting their position directly rather
        // than applying forces or velocities. They're used for:
        // - Player-controlled characters (responds to input, not physics)
        // - Moving platforms (scripted or animated movement)
        // - Doors, elevators, etc. (controlled by game logic)
        //
        // **Why only kinematic bodies?**
        //
        // - **Dynamic bodies**: Controlled by physics simulation. Setting their
        //   position directly would fight against the solver and cause jittering.
        //   If you want to teleport a dynamic body, you should still use
        //   set_position, but that's a rare operation, not every frame.
        //
        // - **Static bodies**: Never move by definition. Setting their position
        //   is technically possible but extremely expensive (requires rebuilding
        //   spatial partitioning) and should only be done for level modifications,
        //   not regular gameplay.
        //
        // **The wake_up parameter (true):**
        //
        // When we move a kinematic body, we want any dynamic bodies it's in
        // contact with to wake up and respond to the movement. For example:
        // - A platform moves up → boxes on it should be pushed up
        // - A door closes → objects in the doorway should be pushed aside
        //
        // Without waking up, sleeping dynamic bodies would ignore the kinematic
        // motion and appear to float or penetrate.

        if rigid_body.is_kinematic() {
            if let Some(body) = physics_world.rigid_body_set.get_mut(body_handle) {
                // Set the position and rotation of the kinematic body to match
                // the ECS Transform. This makes the physics simulation aware of
                // the body's new position for collision detection and resolution.
                
                body.set_position(
                    Isometry::new(
                        vector![
                            transform.translation.x,
                            transform.translation.y,
                            transform.translation.z
                        ],
                        vector![
                            transform.rotation.x,
                            transform.rotation.y,
                            transform.rotation.z
                        ] * transform.rotation.w,
                    ),
                    true, // wake_up: Wake bodies in contact with this kinematic body
                );
            }
        }
        
        // Note: We don't update dynamic body positions here because they're
        // controlled by the physics simulation. If gameplay code wants to
        // teleport a dynamic body, it should still modify the Transform,
        // and this system will apply it. But this should be rare - most of
        // the time, dynamic bodies move themselves through physics.
        //
        // Note: We don't update static body positions at all. Static bodies
        // are meant to be fixed in place. If you need to move level geometry,
        // consider making it kinematic instead.
    }

    // ========================================================================
    // PHASE 2: PHYSICS → ECS SYNCHRONIZATION
    // ========================================================================
    //
    // After the physics simulation has run, we need to copy the results back
    // to the ECS Transform components so that:
    // - Rendering displays objects in their correct physical positions
    // - Other gameplay systems can react to physics-driven movement
    // - The game state remains consistent between physics and logic
    //
    // **Why iterate over all_query instead of changed_query?**
    //
    // Because we want to update ALL dynamic bodies, not just the ones whose
    // Transforms changed. The physics simulation has moved dynamic bodies,
    // and we need to reflect those changes in the ECS.
    //
    // **Why not update kinematic and static bodies?**
    //
    // - **Kinematic bodies**: Their Transform is the source of truth (set by
    //   gameplay code). Physics follows their motion, not the other way around.
    //   If we copied physics state back, we'd overwrite gameplay-driven movement.
    //
    // - **Static bodies**: They never move in the physics simulation (by design),
    //   so there's nothing to copy back. Their Transform remains constant.

    for (entity, mut transform, rigid_body) in &mut all_query {
        // Skip non-dynamic bodies. Only dynamic bodies are controlled by physics.
        if !rigid_body.is_dynamic() {
            continue;
        }

        // Look up the Rapier rigid body handle for this entity.
        // If the body doesn't exist yet, skip it (it will be created on the
        // next Changed<Transform> iteration in Phase 1).
        if let Some(body_handle) = physics_world.get_body_handle(entity) {
            if let Some(body) = physics_world.rigid_body_set.get(body_handle) {
                // --------------------------------------------------------
                // EXTRACT POSITION FROM RAPIER
                // --------------------------------------------------------
                //
                // Rapier stores rigid body pose as an Isometry, which combines:
                // - Translation: A 3D point representing the body's position
                // - Rotation: A unit quaternion representing the body's orientation
                //
                // We extract these and convert them to Praxis/glam types.
                
                let position = body.position();
                let translation = position.translation;
                let rotation = position.rotation;

                // Update the ECS Transform's translation by copying the vector
                // components from Rapier's nalgebra Vector3 to glam's Vec3.
                transform.translation = Vec3::new(translation.x, translation.y, translation.z);

                // --------------------------------------------------------
                // CONVERT ROTATION: RAPIER → PRAXIS
                // --------------------------------------------------------
                //
                // **Rotation representations:**
                //
                // Rapier uses UnitQuaternion from nalgebra, which stores rotation
                // as (w, x, y, z) where w² + x² + y² + z² = 1.
                //
                // However, the API provides scaled_axis() which returns an axis-angle
                // representation: a vector whose direction is the rotation axis and
                // whose magnitude is the rotation angle in radians.
                //
                // Praxis uses glam::Quat which stores (x, y, z, w).
                //
                // **Conversion strategy:**
                //
                // 1. Extract the scaled axis vector (axis * angle)
                // 2. Compute the angle as the magnitude of the vector
                // 3. If angle > 0, normalize the axis
                // 4. Create a Quat from axis-angle using Quat::from_axis_angle
                //
                // **Special case: angle ≈ 0**
                //
                // When the angle is very small (or zero), the scaled axis is close
                // to zero, and normalizing it would be numerically unstable. In this
                // case, the rotation is effectively identity, so we skip the update.
                // The Transform's rotation is already correct (either identity or
                // close enough that the difference doesn't matter).
                //
                // **Why this matters:**
                //
                // - Prevents NaN/inf from dividing by ~0
                // - Avoids unnecessary quaternion updates for minimal rotations
                // - Maintains numerical stability over long simulations

                let axis = rotation.scaled_axis();
                let angle = axis.norm(); // sqrt(x² + y² + z²) = magnitude
                
                if angle > 0.0 {
                    // Significant rotation detected, normalize the axis and create quaternion
                    let axis_normalized = axis / angle;
                    transform.rotation = Quat::from_axis_angle(
                        Vec3::new(axis_normalized.x, axis_normalized.y, axis_normalized.z),
                        angle,
                    );
                }
                // else: angle ≈ 0, rotation is effectively identity, keep current rotation

                // Note: This mutable access to Transform will mark it as "changed"
                // for change detection. This is fine - it allows other systems to
                // react to physics-driven movement (e.g., animation, audio, effects).
            }
        }
    }

    // ========================================================================
    // TRANSFORM PROPAGATION NOTE
    // ========================================================================
    //
    // If your game uses hierarchical transforms (parent-child relationships),
    // you'll need to run the transform propagation system AFTER this system
    // to update GlobalTransform components based on the new Transform values.
    //
    // The typical system ordering is:
    // 1. sync_physics_transforms_system (updates Transform from physics)
    // 2. propagate_transforms (updates GlobalTransform from Transform hierarchy)
    // 3. render (uses GlobalTransform for rendering)
    //
    // Without propagation, child objects won't follow their parents' physics-
    // driven movement.
}

/// Synchronizes ECS transforms to Rapier rigid bodies.
///
/// This system runs before the physics step and updates Rapier body positions
/// based on ECS Transform components. This allows kinematic bodies to be moved
/// via Transform changes and ensures dynamic bodies start at the correct position.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{Schedule, IntoSystemConfigs};
/// use praxis_physics::sync_transforms_to_physics;
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(sync_transforms_to_physics);
/// ```
#[allow(clippy::needless_pass_by_value)]
pub fn sync_transforms_to_physics(
    mut physics_world: ResMut<PhysicsWorld>,
    query: Query<(Entity, &Transform, &PraxisRigidBody), With<PraxisRigidBody>>,
) {
    for (entity, transform, rigid_body) in query.iter() {
        // Get or create rigid body handle
        let body_handle = if let Some(handle) = physics_world.get_body_handle(entity) {
            handle
        } else {
            // Create new rigid body
            let rapier_body_type = match rigid_body {
                PraxisRigidBody::Dynamic => RigidBodyType::Dynamic,
                PraxisRigidBody::Static => RigidBodyType::Fixed,
                PraxisRigidBody::Kinematic => RigidBodyType::KinematicPositionBased,
            };

            let rapier_body = RigidBodyBuilder::new(rapier_body_type)
                .translation(vector![
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z
                ])
                .rotation(vector![
                    transform.rotation.x,
                    transform.rotation.y,
                    transform.rotation.z
                ] * transform.rotation.w)
                .build();

            let handle = physics_world.rigid_body_set.insert(rapier_body);
            physics_world.entity_to_body.insert(entity, handle);
            physics_world.body_to_entity.insert(handle, entity);
            handle
        };

        // Update position for kinematic bodies
        if rigid_body.is_kinematic() {
            if let Some(body) = physics_world.rigid_body_set.get_mut(body_handle) {
                body.set_position(
                    Isometry::new(
                        vector![
                            transform.translation.x,
                            transform.translation.y,
                            transform.translation.z
                        ],
                        vector![
                            transform.rotation.x,
                            transform.rotation.y,
                            transform.rotation.z
                        ] * transform.rotation.w,
                    ),
                    true,
                );
            }
        }
    }
}

/// Steps the physics simulation forward by the configured timestep.
///
/// This system runs the Rapier physics pipeline, advancing the simulation
/// by one timestep. It should run after `sync_transforms_to_physics` and
/// before `sync_transforms_from_physics`.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{Schedule, IntoSystemConfigs};
/// use praxis_physics::{sync_transforms_to_physics, step_physics_simulation, sync_transforms_from_physics};
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems((
///     sync_transforms_to_physics,
///     step_physics_simulation,
///     sync_transforms_from_physics,
/// ).chain());
/// ```
#[allow(clippy::needless_pass_by_value)]
pub fn step_physics_simulation(
    mut physics_world: ResMut<PhysicsWorld>,
    config: Res<PhysicsConfig>,
    mut contact_events: ResMut<ContactEvents>,
) {
    // Clear previous contact events
    contact_events.clear();

    // Update integration parameters from config
    physics_world.integration_parameters.dt = config.timestep;

    // Set gravity
    let gravity = vector![config.gravity.x, config.gravity.y, config.gravity.z];

    // Create event handler for collision events
    let event_handler = ();

    // Step the physics simulation
    // Need to destructure to avoid borrowing issues
    let PhysicsWorld {
        ref mut rigid_body_set,
        ref mut collider_set,
        ref integration_parameters,
        ref mut physics_pipeline,
        ref mut island_manager,
        ref mut broad_phase,
        ref mut narrow_phase,
        ref mut impulse_joint_set,
        ref mut multibody_joint_set,
        ref mut ccd_solver,
        ref mut query_pipeline,
        ..
    } = *physics_world;
    
    physics_pipeline.step(
        &gravity,
        integration_parameters,
        island_manager,
        broad_phase,
        narrow_phase,
        rigid_body_set,
        collider_set,
        impulse_joint_set,
        multibody_joint_set,
        ccd_solver,
        Some(query_pipeline),
        &(),
        &event_handler,
    );
}

/// Synchronizes Rapier rigid body positions back to ECS transforms.
///
/// This system runs after the physics step and updates ECS Transform components
/// based on Rapier body positions. This allows dynamic bodies to move based on
/// physics simulation.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{Schedule, IntoSystemConfigs};
/// use praxis_physics::{sync_transforms_to_physics, step_physics_simulation, sync_transforms_from_physics};
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems((
///     sync_transforms_to_physics,
///     step_physics_simulation,
///     sync_transforms_from_physics,
/// ).chain());
/// ```
#[allow(clippy::needless_pass_by_value)]
pub fn sync_transforms_from_physics(
    physics_world: Res<PhysicsWorld>,
    mut query: Query<(Entity, &mut Transform, &PraxisRigidBody), With<PraxisRigidBody>>,
) {
    for (entity, mut transform, rigid_body) in &mut query {
        // Only update transforms for dynamic bodies
        if !rigid_body.is_dynamic() {
            continue;
        }

        if let Some(body_handle) = physics_world.get_body_handle(entity) {
            if let Some(body) = physics_world.rigid_body_set.get(body_handle) {
                let position = body.position();
                let translation = position.translation;
                let rotation = position.rotation;

                transform.translation = Vec3::new(translation.x, translation.y, translation.z);

                // Convert rotation to quaternion
                let axis = rotation.scaled_axis();
                let angle = axis.norm();
                if angle > 0.0 {
                    let axis_normalized = axis / angle;
                    transform.rotation = Quat::from_axis_angle(
                        Vec3::new(axis_normalized.x, axis_normalized.y, axis_normalized.z),
                        angle,
                    );
                }
            }
        }
    }
}

/// Applies external forces to rigid bodies.
///
/// This system takes forces and torques from `ExternalForces` components
/// and applies them to the corresponding Rapier rigid bodies, then clears
/// the accumulators.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{Schedule, IntoSystemConfigs};
/// use praxis_physics::apply_external_forces;
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(apply_external_forces);
/// ```
#[allow(clippy::needless_pass_by_value)]
pub fn apply_external_forces(
    mut physics_world: ResMut<PhysicsWorld>,
    mut query: Query<(Entity, &mut ExternalForces), With<PraxisRigidBody>>,
) {
    for (entity, mut forces) in &mut query {
        if let Some(body_handle) = physics_world.get_body_handle(entity) {
            if let Some(body) = physics_world.rigid_body_set.get_mut(body_handle) {
                // Apply force
                if forces.force.length_squared() > 0.0 {
                    body.add_force(
                        vector![forces.force.x, forces.force.y, forces.force.z],
                        true,
                    );
                }

                // Apply torque
                if forces.torque.length_squared() > 0.0 {
                    body.add_torque(
                        vector![forces.torque.x, forces.torque.y, forces.torque.z],
                        true,
                    );
                }

                // Clear forces after applying
                forces.clear();
            }
        }
    }
}

/// Creates or updates colliders for entities with Collider components.
///
/// This system checks for entities with Collider components and ensures they
/// have corresponding Rapier colliders attached to their rigid bodies.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{Schedule, IntoSystemConfigs};
/// use praxis_physics::sync_colliders;
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(sync_colliders);
/// ```
#[allow(clippy::needless_pass_by_value)]
pub fn sync_colliders(
    mut physics_world: ResMut<PhysicsWorld>,
    query: Query<(Entity, &PraxisCollider, Option<&Sensor>), With<PraxisRigidBody>>,
) {
    for (entity, collider, sensor) in &query {
        // Skip if collider already exists
        if physics_world.get_collider_handle(entity).is_some() {
            continue;
        }

        // Get rigid body handle
        let Some(body_handle) = physics_world.get_body_handle(entity) else { continue };

        // Create Rapier collider shape
        let shape: SharedShape = match collider {
            PraxisCollider::Cuboid { hx, hy, hz } => SharedShape::cuboid(*hx, *hy, *hz),
            PraxisCollider::Sphere { radius } => SharedShape::ball(*radius),
            PraxisCollider::CapsuleY {
                half_height,
                radius,
            } => SharedShape::capsule_y(*half_height, *radius),
            PraxisCollider::CapsuleX {
                half_height,
                radius,
            } => SharedShape::capsule_x(*half_height, *radius),
            PraxisCollider::CapsuleZ {
                half_height,
                radius,
            } => SharedShape::capsule_z(*half_height, *radius),
            PraxisCollider::CylinderY {
                half_height,
                radius,
            } => SharedShape::cylinder(*half_height, *radius),
        };

        // Build collider
        let mut collider_builder = ColliderBuilder::new(shape);

        // Set as sensor if component present
        if sensor.is_some() {
            collider_builder = collider_builder.sensor(true);
        }

        let rapier_collider = collider_builder.build();

        // Insert collider - need separate borrows to avoid conflict
        let PhysicsWorld {
            ref mut collider_set,
            ref mut rigid_body_set,
            ref mut entity_to_collider,
            ..
        } = *physics_world;
        
        let collider_handle = collider_set.insert_with_parent(
            rapier_collider,
            body_handle,
            rigid_body_set,
        );

        entity_to_collider.insert(entity, collider_handle);
    }
}

/// Updates rigid body properties from components.
///
/// This system synchronizes physics properties (mass, velocity, friction, etc.)
/// from ECS components to Rapier rigid bodies.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{Schedule, IntoSystemConfigs};
/// use praxis_physics::sync_physics_properties;
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(sync_physics_properties);
/// ```
#[allow(clippy::needless_pass_by_value)]
pub fn sync_physics_properties(
    mut physics_world: ResMut<PhysicsWorld>,
    velocity_query: Query<(Entity, &PhysicsVelocity), With<PraxisRigidBody>>,
    friction_query: Query<(Entity, &Friction), With<PraxisCollider>>,
    restitution_query: Query<(Entity, &Restitution), With<PraxisCollider>>,
) {
    // Sync velocities
    for (entity, velocity) in &velocity_query {
        if let Some(body_handle) = physics_world.get_body_handle(entity) {
            if let Some(body) = physics_world.rigid_body_set.get_mut(body_handle) {
                body.set_linvel(
                    vector![velocity.linear.x, velocity.linear.y, velocity.linear.z],
                    true,
                );
                body.set_angvel(
                    vector![velocity.angular.x, velocity.angular.y, velocity.angular.z],
                    true,
                );
            }
        }
    }

    // Sync friction
    for (entity, friction) in &friction_query {
        if let Some(collider_handle) = physics_world.get_collider_handle(entity) {
            if let Some(collider) = physics_world.collider_set.get_mut(collider_handle) {
                collider.set_friction(friction.coefficient);
            }
        }
    }

    // Sync restitution
    for (entity, restitution) in &restitution_query {
        if let Some(collider_handle) = physics_world.get_collider_handle(entity) {
            if let Some(collider) = physics_world.collider_set.get_mut(collider_handle) {
                collider.set_restitution(restitution.coefficient);
            }
        }
    }
}

/// Removes physics bodies and colliders for despawned entities.
///
/// This system should be run to clean up Rapier resources when ECS entities
/// with physics components are despawned.
///
/// Note: This is a placeholder. Proper cleanup requires tracking entity
/// despawn events, which may need integration with `bevy_ecs` removal detection.
pub const fn cleanup_physics_entities(_commands: Commands, _physics_world: ResMut<PhysicsWorld>) {
    // Placeholder for cleanup logic
    // In a full implementation, this would detect removed RigidBody components
    // and clean up the corresponding Rapier handles
}
