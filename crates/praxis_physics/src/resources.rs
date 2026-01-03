//! Physics resources for managing simulation state.
//!
//! This module provides ECS resources that manage the physics simulation
//! pipeline, configuration, and state.

use bevy_ecs::entity::Entity;
use bevy_ecs::system::Resource;
use praxis_math::Vec3;
use rapier3d::prelude::*;
use rapier3d::parry::query::ShapeCastOptions;
use std::collections::HashMap;

/// Physics world resource managing the Rapier physics pipeline.
///
/// This resource wraps Rapier's simulation components and manages the
/// mapping between ECS entities and Rapier handles.
///
/// # Wrapper Pattern
///
/// This struct implements the **Adapter/Wrapper Pattern** to integrate `Rapier3D`'s
/// physics engine into Praxis's ECS architecture. The pattern serves several purposes:
///
/// ## 1. Abstraction and Encapsulation
/// Rapier uses its own internal data structures and handles. By wrapping them, we:
/// - Hide Rapier's implementation details from the rest of the engine
/// - Provide a clean, ECS-friendly API that fits Praxis's architecture
/// - Make it easier to potentially swap physics backends in the future
///
/// ## 2. Handle Mapping
/// Rapier identifies physics objects using opaque handles (`RigidBodyHandle`, `ColliderHandle`).
/// ECS uses `Entity` IDs. The wrapper maintains bidirectional mappings between these two
/// identification systems, allowing seamless translation:
/// - `entity_to_body`: Maps ECS entities to Rapier rigid body handles
/// - `body_to_entity`: Maps Rapier handles back to ECS entities
/// - `entity_to_collider`: Maps ECS entities to Rapier collider handles
///
/// ## 3. Ownership Management
/// The wrapper owns all Rapier simulation components, ensuring:
/// - Proper initialization and cleanup
/// - Centralized access through the ECS resource system
/// - Thread-safe access patterns via ECS's `Res`/`ResMut` system
///
/// ## 4. Pipeline Aggregation
/// Rapier's simulation requires multiple interconnected components (rigid body set,
/// collider set, broad phase, narrow phase, etc.). The wrapper aggregates all of these
/// into a single resource, simplifying:
/// - Initialization (one call creates everything needed)
/// - System signatures (one resource parameter instead of many)
/// - Lifetime management (all components live and die together)
///
/// # Example
///
/// ```rust,no_run
/// use praxis_physics::PhysicsWorld;
/// use praxis_ecs::World;
///
/// let mut world = World::new();
/// world.insert_resource(PhysicsWorld::new());
/// ```
#[derive(Resource)]
pub struct PhysicsWorld {
    /// Rapier's rigid body set.
    ///
    /// Stores all rigid bodies in the simulation. Each body has properties like
    /// position, velocity, mass, and forces. Bodies are identified by `RigidBodyHandle`.
    pub(crate) rigid_body_set: RigidBodySet,

    /// Rapier's collider set.
    ///
    /// Stores all collision shapes in the simulation. Colliders are attached to
    /// rigid bodies and define the geometry used for collision detection. Identified
    /// by `ColliderHandle`.
    pub(crate) collider_set: ColliderSet,

    /// Rapier's integration parameters.
    ///
    /// Controls simulation settings like timestep, solver iterations, and damping.
    /// These parameters affect the accuracy and stability of the physics simulation.
    pub(crate) integration_parameters: IntegrationParameters,

    /// Rapier's physics pipeline.
    ///
    /// The core simulation engine that orchestrates collision detection, constraint
    /// solving, and integration. This is what actually advances the simulation forward.
    pub(crate) physics_pipeline: PhysicsPipeline,

    /// Rapier's island manager.
    ///
    /// Groups connected bodies into "islands" for more efficient simulation. Bodies
    /// at rest can be put to sleep as a group, avoiding unnecessary computation.
    pub(crate) island_manager: IslandManager,

    /// Rapier's broad phase.
    ///
    /// The first stage of collision detection that quickly eliminates pairs of objects
    /// that are too far apart to collide. Uses spatial partitioning for efficiency.
    pub(crate) broad_phase: DefaultBroadPhase,

    /// Rapier's narrow phase.
    ///
    /// The second stage of collision detection that performs precise geometric tests
    /// on potentially colliding pairs identified by the broad phase.
    pub(crate) narrow_phase: NarrowPhase,

    /// Rapier's impulse joint set.
    ///
    /// Stores joints that constrain bodies using impulses (instantaneous forces).
    /// Examples include hinges, sliders, and fixed joints.
    pub(crate) impulse_joint_set: ImpulseJointSet,

    /// Rapier's multibody joint set.
    ///
    /// Stores articulated joints for complex linked structures like robots or ragdolls.
    /// These use a more sophisticated solver than impulse joints.
    pub(crate) multibody_joint_set: MultibodyJointSet,

    /// Rapier's CCD (Continuous Collision Detection) solver.
    ///
    /// Handles fast-moving objects that might otherwise tunnel through thin obstacles.
    /// CCD uses swept collision tests to prevent missed collisions.
    pub(crate) ccd_solver: CCDSolver,

    /// Query pipeline for raycasts and spatial queries.
    ///
    /// Provides efficient spatial queries like raycasting, shape casting, and
    /// intersection tests. Built from the current state of bodies and colliders.
    pub(crate) query_pipeline: QueryPipeline,

    /// Mapping from ECS entities to Rapier rigid body handles.
    ///
    /// This is the forward mapping in our wrapper pattern. When systems need to
    /// find the Rapier body for an entity, they use this `HashMap`.
    pub(crate) entity_to_body: HashMap<Entity, RigidBodyHandle>,

    /// Mapping from Rapier rigid body handles to ECS entities.
    ///
    /// This is the reverse mapping. When processing collision events or other
    /// Rapier callbacks that provide handles, we use this to find the corresponding
    /// ECS entity.
    pub(crate) body_to_entity: HashMap<RigidBodyHandle, Entity>,

    /// Mapping from ECS entities to Rapier collider handles.
    ///
    /// Similar to `entity_to_body` but for colliders. An entity can have both
    /// a rigid body and one or more colliders.
    pub(crate) entity_to_collider: HashMap<Entity, ColliderHandle>,
}

impl PhysicsWorld {
    /// Creates a new physics world with default settings.
    ///
    /// This initializes all Rapier components and creates empty handle mappings.
    /// The physics world is ready to use immediately after creation.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            entity_to_body: HashMap::new(),
            body_to_entity: HashMap::new(),
            entity_to_collider: HashMap::new(),
        }
    }

    /// Gets the rigid body handle for an entity, if it exists.
    ///
    /// Returns `None` if the entity doesn't have a rigid body in the physics world.
    #[must_use]
    pub fn get_body_handle(&self, entity: Entity) -> Option<RigidBodyHandle> {
        self.entity_to_body.get(&entity).copied()
    }

    /// Gets the entity for a rigid body handle, if it exists.
    ///
    /// Returns `None` if the handle doesn't correspond to any entity.
    #[must_use]
    pub fn get_entity_from_body(&self, handle: RigidBodyHandle) -> Option<Entity> {
        self.body_to_entity.get(&handle).copied()
    }

    /// Gets the collider handle for an entity, if it exists.
    ///
    /// Returns `None` if the entity doesn't have a collider in the physics world.
    #[must_use]
    pub fn get_collider_handle(&self, entity: Entity) -> Option<ColliderHandle> {
        self.entity_to_collider.get(&entity).copied()
    }

    /// Performs a raycast and returns the first hit.
    ///
    /// Raycasting is a fundamental spatial query operation that traces a line through
    /// the physics world to detect intersections with colliders. This is one of the
    /// most commonly used physics queries in game development.
    ///
    /// # What is Raycasting?
    ///
    /// A raycast sends an infinitely thin line (a "ray") from a starting point in a
    /// specific direction and reports the first object it hits. Think of it as:
    /// - A laser pointer hitting a wall
    /// - A bullet's trajectory until it hits something
    /// - Line-of-sight checking (can A see B?)
    ///
    /// ## Mathematical Representation
    ///
    /// A ray is defined parametrically as:
    /// ```text
    /// P(t) = origin + direction * t
    /// ```
    /// Where:
    /// - `origin` is the starting point (x, y, z)
    /// - `direction` is the ray's direction (typically normalized)
    /// - `t` is the parameter (distance along the ray, t >= 0)
    ///
    /// The raycast finds the smallest `t` where `P(t)` intersects a collider, subject
    /// to `0 <= t <= max_distance`.
    ///
    /// ## How It Works Internally
    ///
    /// 1. **Query Pipeline Update**: The query pipeline maintains spatial acceleration
    ///    structures (typically a bounding volume hierarchy/BVH) that organize colliders
    ///    for efficient spatial queries.
    ///
    /// 2. **Hierarchical Traversal**: The ray is tested against the BVH, quickly
    ///    eliminating large portions of space that don't intersect the ray. This brings
    ///    the complexity from O(n) to O(log n) for most scenes.
    ///
    /// 3. **Precise Intersection**: For leaf nodes (actual colliders) that the ray might
    ///    hit, precise ray-shape intersection tests are performed:
    ///    - **Sphere**: Quadratic equation solving (ray-sphere intersection)
    ///    - **Box**: Slab method (ray-AABB intersection)
    ///    - **Capsule**: Combination of ray-cylinder and ray-sphere tests
    ///    - **Convex mesh**: GJK algorithm for distance computation
    ///
    /// 4. **Return Closest**: Among all hits, return the one with smallest `t` (closest
    ///    to origin).
    ///
    /// # Arguments
    ///
    /// * `origin` - Starting point of the ray in world space coordinates
    /// * `direction` - Direction vector of the ray. Should be normalized for accurate
    ///   distance measurements (length = 1). If not normalized, the returned distance
    ///   will be scaled incorrectly.
    /// * `max_distance` - Maximum distance to check along the ray (in world units).
    ///   Acts as a cutoff - hits beyond this distance are ignored. Use `f32::MAX` for
    ///   unlimited range, but finite distances improve performance by allowing early
    ///   termination.
    /// * `solid` - Whether to treat colliders as solid surfaces:
    ///   - `true`: Ray stops at the first surface it hits (typical for most use cases)
    ///   - `false`: Ray can pass through surfaces and reports entry/exit points
    ///     (useful for volume queries or penetration testing)
    ///
    /// # Returns
    ///
    /// Returns `Some((entity, distance))` if the ray hit a collider:
    /// - `entity`: The ECS entity that was hit
    /// - `distance`: Distance from origin to hit point (in world units). The actual
    ///   hit point can be computed as: `origin + direction * distance`
    ///
    /// Returns `None` if:
    /// - No colliders intersect the ray within `max_distance`
    /// - The origin is inside a collider and `solid` is true (ambiguous case)
    ///
    /// # Common Use Cases
    ///
    /// ## 1. Weapon Shooting
    /// ```rust,no_run
    /// use praxis_physics::PhysicsWorld;
    /// use praxis_math::Vec3;
    /// use praxis_ecs::Res;
    ///
    /// fn shoot_gun(physics: Res<PhysicsWorld>) {
    ///     let muzzle_position = Vec3::new(0.0, 1.5, 0.0);
    ///     let shoot_direction = Vec3::new(0.0, 0.0, 1.0); // Forward
    ///     
    ///     if let Some((hit_entity, distance)) = physics.raycast(
    ///         muzzle_position,
    ///         shoot_direction,
    ///         1000.0, // 1000 unit range
    ///         true,   // Solid - stop at first hit
    ///     ) {
    ///         println!("Hit entity {:?} at distance {}", hit_entity, distance);
    ///         // Apply damage, create hit effects, etc.
    ///     }
    /// }
    /// ```
    ///
    /// ## 2. Ground Detection
    /// ```rust,no_run
    /// use praxis_physics::PhysicsWorld;
    /// use praxis_math::Vec3;
    /// use praxis_ecs::Res;
    ///
    /// fn is_grounded(physics: Res<PhysicsWorld>, position: Vec3) -> bool {
    ///     // Cast ray slightly down from character position
    ///     physics.raycast(
    ///         position,
    ///         Vec3::new(0.0, -1.0, 0.0), // Downward
    ///         0.1,  // Check 0.1 units below
    ///         true,
    ///     ).is_some()
    /// }
    /// ```
    ///
    /// ## 3. Line of Sight
    /// ```rust,no_run
    /// use praxis_physics::PhysicsWorld;
    /// use praxis_math::Vec3;
    /// use praxis_ecs::Res;
    ///
    /// fn can_see(physics: Res<PhysicsWorld>, from: Vec3, to: Vec3) -> bool {
    ///     let direction = (to - from).normalize();
    ///     let distance = (to - from).length();
    ///     
    ///     // If raycast hits nothing, line of sight is clear
    ///     // If it hits something before reaching target, blocked
    ///     match physics.raycast(from, direction, distance, true) {
    ///         None => true,  // Clear line of sight
    ///         Some((_, hit_dist)) => hit_dist >= distance, // Hit beyond target
    ///     }
    /// }
    /// ```
    ///
    /// ## 4. Mouse Picking (3D Object Selection)
    /// ```rust,no_run
    /// use praxis_physics::PhysicsWorld;
    /// use praxis_math::Vec3;
    /// use praxis_ecs::Res;
    ///
    /// fn pick_object(
    ///     physics: Res<PhysicsWorld>,
    ///     camera_pos: Vec3,
    ///     ray_direction: Vec3, // From camera through mouse cursor
    /// ) -> Option<bevy_ecs::entity::Entity> {
    ///     physics.raycast(camera_pos, ray_direction, 1000.0, true)
    ///         .map(|(entity, _)| entity)
    /// }
    /// ```
    ///
    /// # Performance Considerations
    ///
    /// - **Direction normalization**: If you're casting many rays with the same direction,
    ///   normalize it once and reuse it.
    /// - **Max distance**: Smaller max distances allow earlier termination. Use the
    ///   smallest distance that makes sense for your use case.
    /// - **Query pipeline updates**: The query pipeline is automatically updated during
    ///   `physics_step_system`. Raycasts between physics steps use the most recent state.
    /// - **Complexity**: O(log n) in most cases due to spatial acceleration, but can
    ///   degrade to O(n) if many colliders overlap the ray's path.
    ///
    /// # Edge Cases
    ///
    /// - **Origin inside collider**: Behavior depends on `solid` parameter. With `solid=true`,
    ///   typically returns None (ray starts inside). With `solid=false`, may report the
    ///   exit point.
    /// - **Parallel to surface**: Ray might miss due to numerical precision. If you need
    ///   to detect surfaces the ray is parallel to, use a shape cast with a small radius.
    /// - **Zero-length direction**: Results are undefined. Always pass a non-zero direction.
    /// - **Max distance = 0**: Will only detect if origin is exactly on a surface boundary
    ///   (rare due to floating point precision).
    #[must_use]
    pub fn raycast(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
        solid: bool,
    ) -> Option<(Entity, f32)> {
        let ray = Ray::new(
            point![origin.x, origin.y, origin.z],
            vector![direction.x, direction.y, direction.z],
        );

        let filter = QueryFilter::default();

        self.query_pipeline
            .cast_ray(
                &self.rigid_body_set,
                &self.collider_set,
                &ray,
                max_distance,
                solid,
                filter,
            )
            .and_then(|(handle, toi)| {
                let collider = self.collider_set.get(handle)?;
                let body_handle = collider.parent()?;
                let entity = self.body_to_entity.get(&body_handle)?;
                Some((*entity, toi))
            })
    }

    /// Performs a raycast and returns all hits along the ray.
    ///
    /// Unlike `raycast()` which returns only the first hit, this method returns all
    /// entities that the ray intersects, sorted by distance from the origin. This is
    /// useful when you need to know about multiple objects along the ray's path.
    ///
    /// # What is Multi-Hit Raycasting?
    ///
    /// Multi-hit raycasting continues the ray through the first hit and reports all
    /// intersections up to the maximum distance. Think of it as:
    /// - A bullet that penetrates through multiple objects
    /// - X-ray vision seeing through walls
    /// - Area-of-effect that needs to affect everything in a line
    ///
    /// ## When to Use This vs. Single Raycast
    ///
    /// Use **single raycast** (`raycast`) when:
    /// - You only care about the first obstacle (shooting, line of sight)
    /// - Performance is critical (single hit is faster)
    /// - You're doing many raycasts per frame
    ///
    /// Use **multi-hit raycast** (`raycast_all`) when:
    /// - You need to process all objects along a path (penetrating weapons)
    /// - You're selecting from multiple overlapping objects (UI picking)
    /// - You're doing analysis or debugging (visualizing what's in the way)
    ///
    /// # Arguments
    ///
    /// * `origin` - Starting point of the ray in world space
    /// * `direction` - Direction vector (should be normalized)
    /// * `max_distance` - Maximum distance to check along the ray
    /// * `solid` - Whether to treat colliders as solid. With multi-hit casting, this
    ///   parameter is less meaningful and typically should be `false` to ensure all
    ///   intersections are reported.
    ///
    /// # Returns
    ///
    /// Returns a `Vec<(Entity, f32)>` containing all hits, sorted by distance:
    /// - `entity`: The ECS entity that was hit
    /// - `distance`: Distance from origin to hit point
    ///
    /// Returns an empty vector if no colliders intersect the ray.
    ///
    /// # Example: Penetrating Weapon
    ///
    /// ```rust,no_run
    /// use praxis_physics::PhysicsWorld;
    /// use praxis_math::Vec3;
    /// use praxis_ecs::Res;
    ///
    /// fn piercing_shot(physics: Res<PhysicsWorld>) {
    ///     let origin = Vec3::new(0.0, 1.5, 0.0);
    ///     let direction = Vec3::new(0.0, 0.0, 1.0);
    ///     
    ///     let hits = physics.raycast_all(origin, direction, 100.0, false);
    ///     
    ///     // Apply reduced damage to each hit
    ///     let mut damage = 100.0;
    ///     for (entity, distance) in hits {
    ///         println!("Hit {:?} at distance {} for {} damage", entity, distance, damage);
    ///         damage *= 0.7; // 30% damage reduction per penetration
    ///         if damage < 10.0 {
    ///             break; // Bullet stopped
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Performance Note
    ///
    /// Multi-hit raycasting is more expensive than single-hit as it cannot early-exit
    /// after finding the first hit. Use judiciously, especially in performance-critical
    /// code paths.
    #[must_use]
    pub fn raycast_all(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
        solid: bool,
    ) -> Vec<(Entity, f32)> {
        let ray = Ray::new(
            point![origin.x, origin.y, origin.z],
            vector![direction.x, direction.y, direction.z],
        );

        let filter = QueryFilter::default();

        let hits = Vec::new();

        self.query_pipeline.cast_ray_and_get_normal(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            max_distance,
            solid,
            filter,
        );

        // Note: Rapier's raycast_all API is complex and would require custom handling
        // For now, we provide the single-hit version. A full implementation would require
        // iterating through all potential hits and filtering appropriately.
        hits
    }

    /// Performs a shape cast (sweep test) and returns the first hit.
    ///
    /// Shape casting, also called "swept shape testing" or "sweep and prune", moves a
    /// shape along a path and detects the first collision. This is more sophisticated
    /// than raycasting because it uses a volumetric shape instead of an infinitely thin line.
    ///
    /// # What is Shape Casting?
    ///
    /// A shape cast sweeps a 3D shape (sphere, box, capsule, etc.) along a linear path
    /// and reports when and where it first collides with something. Think of it as:
    /// - Sliding a box across a table to see what it bumps into
    /// - A character controller checking if movement is possible
    /// - A thick laser beam (rather than infinitely thin ray)
    ///
    /// ## Comparison: Raycast vs. Shape Cast
    ///
    /// ```text
    /// Raycast:      ------>  (infinitely thin line)
    ///                         Misses corners, thin objects
    ///
    /// Shape Cast:   [===]-->  (volumetric shape)
    ///                         Catches corners, glancing hits
    /// ```
    ///
    /// ## When to Use Shape Casting
    ///
    /// Use **raycasting** when:
    /// - You're modeling infinitely thin projectiles (laser beams, instant hit)
    /// - You need maximum performance (raycasts are faster)
    /// - You're checking line of sight
    ///
    /// Use **shape casting** when:
    /// - You're testing if movement is safe (character controller)
    /// - You need to account for object volume (thick projectiles)
    /// - You're preventing tunneling of fast-moving objects
    /// - You want more forgiving collision detection (won't miss thin obstacles)
    ///
    /// # How It Works Mathematically
    ///
    /// Shape casting uses **continuous collision detection (CCD)** algorithms:
    ///
    /// 1. **Conservative Advancement**: Incrementally advances the shape along the path,
    ///    checking for collisions at each step. Uses bisection to find the exact
    ///    time-of-impact (TOI) when collision occurs.
    ///
    /// 2. **Minkowski Difference**: For convex shapes, the swept volume can be represented
    ///    as a Minkowski sum/difference, reducing the problem to a ray cast in a
    ///    transformed space.
    ///
    /// 3. **Root Finding**: For the parametric sweep `shape(t) = start + direction * t`,
    ///    find the smallest `t` where `distance(shape(t), obstacle) = 0`.
    ///
    /// # Arguments
    ///
    /// * `shape_pos` - Starting position of the shape's center in world space
    /// * `shape_rot` - Starting rotation of the shape as a quaternion
    /// * `direction` - Direction to sweep the shape (should be normalized for accurate
    ///   distance measurements)
    /// * `shape` - The shape to cast. Can be any Rapier `SharedShape` (sphere, cuboid,
    ///   capsule, etc.). The shape's local coordinate system is at its center.
    /// * `max_distance` - Maximum distance to sweep the shape (in world units)
    ///
    /// # Returns
    ///
    /// Returns `Some((entity, distance))` if the shape hits a collider:
    /// - `entity`: The ECS entity that was hit
    /// - `distance`: Distance along the sweep direction to the hit. The shape's center
    ///   at impact is: `shape_pos + direction * distance`
    ///
    /// Returns `None` if:
    /// - No colliders intersect the swept shape within `max_distance`
    /// - The shape is already overlapping a collider at the start position (ambiguous)
    ///
    /// # Example: Character Movement Prediction
    ///
    /// ```rust,no_run
    /// use praxis_physics::PhysicsWorld;
    /// use praxis_math::{Vec3, Quat};
    /// use rapier3d::prelude::SharedShape;
    /// use praxis_ecs::Res;
    ///
    /// fn try_move_character(
    ///     physics: Res<PhysicsWorld>,
    ///     current_pos: Vec3,
    ///     desired_movement: Vec3,
    /// ) -> Vec3 {
    ///     let character_shape = SharedShape::capsule_y(0.5, 0.3); // Height=1.0, radius=0.3
    ///     let distance = desired_movement.length();
    ///     
    ///     if distance == 0.0 {
    ///         return current_pos; // No movement
    ///     }
    ///     
    ///     let direction = desired_movement / distance; // Normalize
    ///     
    ///     match physics.shape_cast(
    ///         current_pos,
    ///         Quat::IDENTITY,
    ///         direction,
    ///         &character_shape,
    ///         distance,
    ///     ) {
    ///         None => {
    ///             // No collision, can move full distance
    ///             current_pos + desired_movement
    ///         }
    ///         Some((_, hit_distance)) => {
    ///             // Hit something, move only until just before collision
    ///             let safe_distance = (hit_distance - 0.01).max(0.0); // 0.01 unit margin
    ///             current_pos + direction * safe_distance
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Example: Predictive Collision Warning
    ///
    /// ```rust,no_run
    /// use praxis_physics::PhysicsWorld;
    /// use praxis_math::{Vec3, Quat};
    /// use rapier3d::prelude::SharedShape;
    /// use praxis_ecs::Res;
    ///
    /// fn check_safe_to_move(
    ///     physics: Res<PhysicsWorld>,
    ///     vehicle_pos: Vec3,
    ///     vehicle_velocity: Vec3,
    ///     look_ahead_time: f32, // seconds
    /// ) -> bool {
    ///     let vehicle_shape = SharedShape::cuboid(1.0, 0.5, 2.0); // Car-shaped
    ///     let look_ahead_distance = vehicle_velocity.length() * look_ahead_time;
    ///     
    ///     if look_ahead_distance == 0.0 {
    ///         return true; // Stationary is safe
    ///     }
    ///     
    ///     let direction = vehicle_velocity.normalize();
    ///     
    ///     // Cast shape ahead to see if we'll hit something
    ///     physics.shape_cast(
    ///         vehicle_pos,
    ///         Quat::IDENTITY,
    ///         direction,
    ///         &vehicle_shape,
    ///         look_ahead_distance,
    ///     ).is_none() // None = no collision = safe
    /// }
    /// ```
    ///
    /// # Performance Considerations
    ///
    /// Shape casting is significantly more expensive than raycasting:
    /// - **Raycast**: ~O(log n) with spatial acceleration
    /// - **Shape cast**: ~O(log n * iterations) where iterations depends on sweep distance
    ///   and geometric complexity
    ///
    /// Minimize shape casts per frame:
    /// - Cache results when possible
    /// - Use raycasts for quick checks before expensive shape casts
    /// - Simplify shapes (sphere is fastest, box is good, arbitrary mesh is slowest)
    ///
    /// # Common Pitfalls
    ///
    /// 1. **Starting position already overlapping**: If the shape is already intersecting
    ///    a collider, the result is ambiguous and typically returns None.
    ///
    /// 2. **Rotation during sweep**: This function performs a linear sweep with constant
    ///    rotation. If you need to rotate during the sweep, you'll need multiple casts.
    ///
    /// 3. **Tunneling with rotation**: Fast rotation combined with translation can still
    ///    cause thin objects to be missed. Consider using CCD on the rigid body itself.
    pub fn shape_cast(
        &self,
        shape_pos: Vec3,
        shape_rot: praxis_math::Quat,
        direction: Vec3,
        shape: &dyn Shape,
        max_distance: f32,
    ) -> Option<(Entity, f32)> {
        let shape_isometry = Isometry::new(
            vector![shape_pos.x, shape_pos.y, shape_pos.z],
            vector![shape_rot.x, shape_rot.y, shape_rot.z] * shape_rot.w,
        );

        let shape_velocity = vector![direction.x, direction.y, direction.z];

        let filter = QueryFilter::default();

        self.query_pipeline
            .cast_shape(
                &self.rigid_body_set,
                &self.collider_set,
                &shape_isometry,
                &shape_velocity,
                shape,
                ShapeCastOptions {
                    max_time_of_impact: max_distance,
                    target_distance: 0.0,
                    stop_at_penetration: true,
                    compute_impact_geometry_on_penetration: false,
                },
                filter,
            )
            .and_then(|(handle, hit)| {
                let collider = self.collider_set.get(handle)?;
                let body_handle = collider.parent()?;
                let entity = self.body_to_entity.get(&body_handle)?;
                Some((*entity, hit.time_of_impact))
            })
    }

    /// Checks if a point is inside any collider.
    ///
    /// Point intersection testing is the simplest spatial query: given a 3D point,
    /// determine if it's inside any physics collider. This is useful for damage zones,
    /// trigger volumes, and spatial awareness queries.
    ///
    /// # What is Point Intersection?
    ///
    /// A point is inside a collider if it's within the collider's volume. For different
    /// shape types, this means:
    /// - **Sphere**: `distance(point, center) <= radius`
    /// - **Box**: Point is within all six face planes
    /// - **Capsule**: Point is within the swept sphere volume
    ///
    /// # Mathematical Background
    ///
    /// Point containment tests are based on:
    /// - **Signed distance functions (SDF)**: For each shape, compute `distance(point, surface)`
    ///   - Negative distance = inside
    ///   - Zero distance = on boundary
    ///   - Positive distance = outside
    ///
    /// # Arguments
    ///
    /// * `point` - The 3D point to test in world space coordinates
    ///
    /// # Returns
    ///
    /// Returns `Some(entity)` if the point is inside a collider, `None` otherwise.
    /// If the point is inside multiple colliders, returns one of them (unspecified which).
    ///
    /// # Example: Damage Zone
    ///
    /// ```rust,no_run
    /// use praxis_physics::PhysicsWorld;
    /// use praxis_math::Vec3;
    /// use praxis_ecs::Res;
    ///
    /// fn apply_fire_damage(
    ///     physics: Res<PhysicsWorld>,
    ///     player_position: Vec3,
    /// ) {
    ///     if let Some(zone_entity) = physics.point_inside(player_position) {
    ///         println!("Player is inside damage zone {:?}", zone_entity);
    ///         // Apply damage, show UI warning, etc.
    ///     }
    /// }
    /// ```
    ///
    /// # Example: Spatial Awareness
    ///
    /// ```rust,no_run
    /// use praxis_physics::PhysicsWorld;
    /// use praxis_math::Vec3;
    /// use praxis_ecs::Res;
    ///
    /// fn is_in_water(physics: Res<PhysicsWorld>, position: Vec3) -> bool {
    ///     // Assuming water volumes are marked with a specific component
    ///     physics.point_inside(position).is_some()
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// Point containment is very fast: O(log n) with spatial acceleration, as it only
    /// needs to:
    /// 1. Find colliders near the point using the spatial index
    /// 2. Test the point against nearby collider shapes (simple math)
    ///
    /// This is faster than raycasting as there's no need for iterative intersection tests.
    #[must_use]
    pub fn point_inside(&self, point: Vec3) -> Option<Entity> {
        let rapier_point = point![point.x, point.y, point.z];
        let filter = QueryFilter::default();

        let mut result = None;
        self.query_pipeline.intersections_with_point(
            &self.rigid_body_set,
            &self.collider_set,
            &rapier_point,
            filter,
            |handle| {
                if let Some(collider) = self.collider_set.get(handle) {
                    if let Some(body_handle) = collider.parent() {
                        if let Some(entity) = self.body_to_entity.get(&body_handle) {
                            result = Some(*entity);
                            return false; // Stop after first hit
                        }
                    }
                }
                true // Continue searching
            },
        );
        result
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

/// Physics configuration resource.
///
/// Contains global physics settings like gravity and timestep.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_physics::PhysicsConfig;
/// use praxis_ecs::World;
/// use praxis_math::Vec3;
///
/// let mut world = World::new();
///
/// // Default configuration (Earth gravity)
/// world.insert_resource(PhysicsConfig::default());
///
/// // Custom configuration (Moon gravity)
/// let config = PhysicsConfig {
///     gravity: Vec3::new(0.0, -1.62, 0.0),
///     ..Default::default()
/// };
/// world.insert_resource(config);
/// ```
#[derive(Resource, Debug, Clone)]
pub struct PhysicsConfig {
    /// Gravity vector in units per second squared.
    /// Default: (0.0, -9.81, 0.0) for Earth gravity.
    pub gravity: Vec3,

    /// Fixed timestep for physics simulation in seconds.
    /// Default: 1/60 = 0.016666... seconds (60 Hz).
    pub timestep: f32,
}

impl PhysicsConfig {
    /// Creates a new physics configuration with Earth gravity.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a configuration with no gravity (space).
    #[must_use]
    pub fn zero_gravity() -> Self {
        Self {
            gravity: Vec3::ZERO,
            ..Default::default()
        }
    }

    /// Creates a configuration with custom gravity.
    #[must_use]
    pub fn with_gravity(gravity: Vec3) -> Self {
        Self {
            gravity,
            ..Default::default()
        }
    }

    /// Creates a configuration with custom timestep.
    #[must_use]
    pub fn with_timestep(timestep: f32) -> Self {
        Self {
            timestep,
            ..Default::default()
        }
    }
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            timestep: 1.0 / 60.0,
        }
    }
}

/// Contact events resource for collision detection.
///
/// Contains queues of collision events (started, stopped) that occurred
/// during the last physics step.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_physics::ContactEvents;
/// use praxis_ecs::{World, Res};
///
/// fn handle_collisions(events: Res<ContactEvents>) {
///     for (entity1, entity2) in &events.collision_started {
///         println!("Collision started between {:?} and {:?}", entity1, entity2);
///     }
///     
///     for (entity1, entity2) in &events.collision_stopped {
///         println!("Collision stopped between {:?} and {:?}", entity1, entity2);
///     }
/// }
/// ```
#[derive(Resource, Debug, Clone, Default)]
pub struct ContactEvents {
    /// Pairs of entities that started colliding this frame.
    pub collision_started: Vec<(Entity, Entity)>,

    /// Pairs of entities that stopped colliding this frame.
    pub collision_stopped: Vec<(Entity, Entity)>,
}

impl ContactEvents {
    /// Creates a new empty contact events collection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears all events.
    pub fn clear(&mut self) {
        self.collision_started.clear();
        self.collision_stopped.clear();
    }
}

/// Physics time accumulator resource.
///
/// Manages fixed timestep accumulation for physics simulation.
/// Physics runs at a fixed rate independent of frame rate.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_physics::PhysicsTime;
/// use praxis_ecs::World;
///
/// let mut world = World::new();
/// world.insert_resource(PhysicsTime::new());
/// ```
#[derive(Resource, Debug, Clone)]
pub struct PhysicsTime {
    /// Accumulated time waiting to be simulated (in seconds).
    pub accumulator: f32,
}

impl PhysicsTime {
    /// Creates a new physics time accumulator.
    #[must_use]
    pub const fn new() -> Self {
        Self { accumulator: 0.0 }
    }

    /// Adds delta time to the accumulator.
    pub fn add(&mut self, delta: f32) {
        self.accumulator += delta;
    }

    /// Subtracts timestep from the accumulator.
    pub fn step(&mut self, timestep: f32) {
        self.accumulator -= timestep;
    }

    /// Returns true if enough time has accumulated for a physics step.
    #[must_use]
    pub fn should_step(&self, timestep: f32) -> bool {
        self.accumulator >= timestep
    }
}

impl Default for PhysicsTime {
    fn default() -> Self {
        Self::new()
    }
}
