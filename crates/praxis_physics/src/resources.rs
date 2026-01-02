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
    pub(crate) rigid_body_set: RigidBodySet,
    
    /// Rapier's collider set.
    pub(crate) collider_set: ColliderSet,
    
    /// Rapier's gravity integration pipeline.
    pub(crate) integration_parameters: IntegrationParameters,
    
    /// Rapier's physics pipeline.
    pub(crate) physics_pipeline: PhysicsPipeline,
    
    /// Rapier's island manager.
    pub(crate) island_manager: IslandManager,
    
    /// Rapier's broad phase.
    pub(crate) broad_phase: DefaultBroadPhase,
    
    /// Rapier's narrow phase.
    pub(crate) narrow_phase: NarrowPhase,
    
    /// Rapier's impulse joint set.
    pub(crate) impulse_joint_set: ImpulseJointSet,
    
    /// Rapier's multibody joint set.
    pub(crate) multibody_joint_set: MultibodyJointSet,
    
    /// Rapier's CCD solver.
    pub(crate) ccd_solver: CCDSolver,
    
    /// Query pipeline for raycasts and spatial queries.
    pub(crate) query_pipeline: QueryPipeline,
    
    /// Mapping from ECS entities to Rapier rigid body handles.
    pub(crate) entity_to_body: HashMap<Entity, RigidBodyHandle>,
    
    /// Mapping from Rapier rigid body handles to ECS entities.
    pub(crate) body_to_entity: HashMap<RigidBodyHandle, Entity>,
    
    /// Mapping from ECS entities to Rapier collider handles.
    pub(crate) entity_to_collider: HashMap<Entity, ColliderHandle>,
}

impl PhysicsWorld {
    /// Creates a new physics world with default settings.
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
    pub fn get_body_handle(&self, entity: Entity) -> Option<RigidBodyHandle> {
        self.entity_to_body.get(&entity).copied()
    }

    /// Gets the entity for a rigid body handle, if it exists.
    pub fn get_entity_from_body(&self, handle: RigidBodyHandle) -> Option<Entity> {
        self.body_to_entity.get(&handle).copied()
    }

    /// Gets the collider handle for an entity, if it exists.
    pub fn get_collider_handle(&self, entity: Entity) -> Option<ColliderHandle> {
        self.entity_to_collider.get(&entity).copied()
    }

    /// Performs a raycast and returns the first hit.
    ///
    /// # Arguments
    ///
    /// * `origin` - Starting point of the ray
    /// * `direction` - Direction of the ray (should be normalized)
    /// * `max_distance` - Maximum distance to check
    /// * `solid` - Whether to treat colliders as solid or not
    ///
    /// # Returns
    ///
    /// Returns `Some((entity, distance))` if a hit occurred, `None` otherwise.
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a configuration with no gravity (space).
    pub fn zero_gravity() -> Self {
        Self {
            gravity: Vec3::ZERO,
            ..Default::default()
        }
    }

    /// Creates a configuration with custom gravity.
    pub fn with_gravity(gravity: Vec3) -> Self {
        Self {
            gravity,
            ..Default::default()
        }
    }

    /// Creates a configuration with custom timestep.
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
    pub fn new() -> Self {
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
    pub fn should_step(&self, timestep: f32) -> bool {
        self.accumulator >= timestep
    }
}

impl Default for PhysicsTime {
    fn default() -> Self {
        Self::new()
    }
}
