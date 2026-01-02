//! Physics resources for managing simulation state.
//!
//! This module provides ECS resources that manage the physics simulation
//! pipeline, configuration, and state.

use bevy_ecs::entity::Entity;
use bevy_ecs::system::Resource;
use praxis_math::Vec3;
use rapier3d::prelude::*;
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
    /// Casts a ray through the physics world and returns the first entity hit,
    /// along with the distance to the hit point.
    ///
    /// # Arguments
    ///
    /// * `origin` - Starting point of the ray in world space
    /// * `direction` - Direction of the ray (should be normalized for accurate distance)
    /// * `max_distance` - Maximum distance to check along the ray
    /// * `solid` - Whether to treat colliders as solid or not
    ///
    /// # Returns
    ///
    /// Returns `Some((entity, distance))` if a hit occurred, `None` otherwise.
    /// The distance is measured in world units from the origin.
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

    /// Performs a shape cast and returns the first hit.
    ///
    /// This is a placeholder for future implementation. Shape casting (sweeping a
    /// shape along a path) requires more complex Rapier API usage.
    ///
    /// # Arguments
    ///
    /// * `shape_pos` - Starting position of the shape
    /// * `shape_rot` - Starting rotation of the shape
    /// * `direction` - Direction to cast (should be normalized)
    /// * `shape` - The shape to cast
    /// * `max_distance` - Maximum distance to check
    ///
    /// # Returns
    ///
    /// Returns `Some((entity, distance))` if a hit occurred, `None` otherwise.
    /// Currently always returns `None`.
    pub fn shape_cast(
        &self,
        _shape_pos: Vec3,
        _shape_rot: praxis_math::Quat,
        _direction: Vec3,
        _shape: &dyn Shape,
        _max_distance: f32,
    ) -> Option<(Entity, f32)> {
        // Shape casting is not exposed in a simple way in rapier 0.22
        // For now, we'll just return None
        // A full implementation would require diving into internal Rapier APIs
        None
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
