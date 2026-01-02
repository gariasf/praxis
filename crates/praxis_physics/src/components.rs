//! Physics components for the Praxis ECS.
//!
//! This module provides components that define physics properties for entities.
//! Components are designed to be intuitive and match common physics concepts.

use bevy_ecs::component::Component;
use praxis_math::Vec3;

/// Rigid body component defining the physics behavior type.
///
/// Determines how the entity participates in physics simulation:
/// - **Dynamic**: Fully simulated, affected by forces and collisions
/// - **Static**: Immovable, not affected by forces (e.g., terrain, walls)
/// - **Kinematic**: Moved by animation/code, affects others but not affected by forces
///
/// # Example
///
/// ```rust,no_run
/// use praxis_physics::RigidBody;
/// use praxis_ecs::{World, Transform};
///
/// let mut world = World::new();
///
/// // Dynamic body (e.g., a falling box)
/// world.spawn((
///     Transform::from_xyz(0.0, 10.0, 0.0),
///     RigidBody::Dynamic,
/// ));
///
/// // Static body (e.g., ground)
/// world.spawn((
///     Transform::default(),
///     RigidBody::Static,
/// ));
///
/// // Kinematic body (e.g., moving platform)
/// world.spawn((
///     Transform::default(),
///     RigidBody::Kinematic,
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBody {
    /// Dynamic body affected by forces and collisions.
    Dynamic,
    
    /// Static body that never moves.
    Static,
    
    /// Kinematic body controlled by animation or code.
    Kinematic,
}

impl RigidBody {
    /// Returns true if this is a dynamic body.
    pub fn is_dynamic(&self) -> bool {
        matches!(self, Self::Dynamic)
    }

    /// Returns true if this is a static body.
    pub fn is_static(&self) -> bool {
        matches!(self, Self::Static)
    }

    /// Returns true if this is a kinematic body.
    pub fn is_kinematic(&self) -> bool {
        matches!(self, Self::Kinematic)
    }
}

impl Default for RigidBody {
    fn default() -> Self {
        Self::Dynamic
    }
}

/// Collider component defining collision geometry.
///
/// Specifies the shape used for collision detection and response.
/// Colliders can be attached to rigid bodies or used as sensors.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_physics::Collider;
/// use praxis_ecs::{World, Transform};
///
/// let mut world = World::new();
///
/// // Box collider
/// world.spawn((
///     Transform::default(),
///     Collider::cuboid(1.0, 1.0, 1.0),
/// ));
///
/// // Sphere collider
/// world.spawn((
///     Transform::default(),
///     Collider::sphere(0.5),
/// ));
///
/// // Capsule collider (useful for characters)
/// world.spawn((
///     Transform::default(),
///     Collider::capsule_y(0.5, 1.0),
/// ));
/// ```
#[derive(Component, Debug, Clone)]
pub enum Collider {
    /// Box-shaped collider with half-extents.
    Cuboid {
        /// Half-width (x)
        hx: f32,
        /// Half-height (y)
        hy: f32,
        /// Half-depth (z)
        hz: f32,
    },
    
    /// Sphere collider with radius.
    Sphere {
        /// Radius
        radius: f32,
    },
    
    /// Capsule collider aligned with Y-axis.
    CapsuleY {
        /// Half-height of cylindrical segment
        half_height: f32,
        /// Radius
        radius: f32,
    },
    
    /// Capsule collider aligned with X-axis.
    CapsuleX {
        /// Half-height of cylindrical segment
        half_height: f32,
        /// Radius
        radius: f32,
    },
    
    /// Capsule collider aligned with Z-axis.
    CapsuleZ {
        /// Half-height of cylindrical segment
        half_height: f32,
        /// Radius
        radius: f32,
    },
    
    /// Cylinder collider aligned with Y-axis.
    CylinderY {
        /// Half-height
        half_height: f32,
        /// Radius
        radius: f32,
    },
}

impl Collider {
    /// Creates a cuboid collider with the given half-extents.
    ///
    /// # Arguments
    ///
    /// * `hx` - Half-width (x-axis)
    /// * `hy` - Half-height (y-axis)
    /// * `hz` - Half-depth (z-axis)
    pub fn cuboid(hx: f32, hy: f32, hz: f32) -> Self {
        Self::Cuboid { hx, hy, hz }
    }

    /// Creates a sphere collider with the given radius.
    pub fn sphere(radius: f32) -> Self {
        Self::Sphere { radius }
    }

    /// Creates a Y-aligned capsule collider.
    ///
    /// # Arguments
    ///
    /// * `half_height` - Half-height of the cylindrical segment
    /// * `radius` - Radius of the capsule
    pub fn capsule_y(half_height: f32, radius: f32) -> Self {
        Self::CapsuleY { half_height, radius }
    }

    /// Creates an X-aligned capsule collider.
    pub fn capsule_x(half_height: f32, radius: f32) -> Self {
        Self::CapsuleX { half_height, radius }
    }

    /// Creates a Z-aligned capsule collider.
    pub fn capsule_z(half_height: f32, radius: f32) -> Self {
        Self::CapsuleZ { half_height, radius }
    }

    /// Creates a Y-aligned cylinder collider.
    pub fn cylinder_y(half_height: f32, radius: f32) -> Self {
        Self::CylinderY { half_height, radius }
    }
}

/// Velocity component for dynamic and kinematic bodies.
///
/// Stores linear and angular velocity. For dynamic bodies, velocities are
/// updated by the physics simulation. For kinematic bodies, they can be
/// set directly to create movement.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_physics::{Velocity, RigidBody};
/// use praxis_ecs::{World, Transform};
/// use praxis_math::Vec3;
///
/// let mut world = World::new();
///
/// // Create a body with initial velocity
/// world.spawn((
///     Transform::from_xyz(0.0, 10.0, 0.0),
///     RigidBody::Dynamic,
///     Velocity::linear(Vec3::new(1.0, 0.0, 0.0)),
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Velocity {
    /// Linear velocity in units per second.
    pub linear: Vec3,
    
    /// Angular velocity in radians per second.
    pub angular: Vec3,
}

impl Velocity {
    /// Creates a velocity with only linear component.
    pub fn linear(linear: Vec3) -> Self {
        Self {
            linear,
            angular: Vec3::ZERO,
        }
    }

    /// Creates a velocity with only angular component.
    pub fn angular(angular: Vec3) -> Self {
        Self {
            linear: Vec3::ZERO,
            angular,
        }
    }

    /// Creates a velocity with both linear and angular components.
    pub fn new(linear: Vec3, angular: Vec3) -> Self {
        Self { linear, angular }
    }
}

/// External forces component for accumulating forces and torques.
///
/// Forces and torques added to this component are applied during the
/// physics step and then cleared. This provides a clean way to apply
/// one-time or per-frame forces without directly manipulating velocity.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_physics::{ExternalForces, RigidBody};
/// use praxis_ecs::{World, Query, Transform};
/// use praxis_math::Vec3;
///
/// // System that applies an upward force when space is pressed
/// fn jump_system(mut query: Query<&mut ExternalForces>) {
///     for mut forces in query.iter_mut() {
///         forces.apply_force(Vec3::new(0.0, 500.0, 0.0));
///     }
/// }
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ExternalForces {
    /// Accumulated force in Newtons.
    pub force: Vec3,
    
    /// Accumulated torque in Newton-meters.
    pub torque: Vec3,
}

impl ExternalForces {
    /// Applies a force to the body.
    pub fn apply_force(&mut self, force: Vec3) {
        self.force += force;
    }

    /// Applies a torque to the body.
    pub fn apply_torque(&mut self, torque: Vec3) {
        self.torque += torque;
    }

    /// Applies a force at a specific point relative to the center of mass.
    pub fn apply_force_at_point(&mut self, force: Vec3, point: Vec3) {
        self.force += force;
        self.torque += point.cross(force);
    }

    /// Clears all accumulated forces and torques.
    pub fn clear(&mut self) {
        self.force = Vec3::ZERO;
        self.torque = Vec3::ZERO;
    }
}

/// Mass properties component.
///
/// Defines the mass and inertia properties of a rigid body.
/// For dynamic bodies, mass affects how forces translate to acceleration.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_physics::{Mass, RigidBody};
/// use praxis_ecs::{World, Transform};
///
/// let mut world = World::new();
///
/// // Heavy object
/// world.spawn((
///     Transform::from_xyz(0.0, 10.0, 0.0),
///     RigidBody::Dynamic,
///     Mass::new(100.0),
/// ));
///
/// // Light object with custom inertia
/// world.spawn((
///     Transform::from_xyz(0.0, 5.0, 0.0),
///     RigidBody::Dynamic,
///     Mass::with_inertia(1.0, 0.5),
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy)]
pub struct Mass {
    /// Mass in kilograms.
    pub mass: f32,
    
    /// Angular inertia factor (simplified scalar).
    pub angular_inertia: f32,
}

impl Mass {
    /// Creates a mass component with the given mass value.
    ///
    /// Angular inertia is automatically computed based on mass.
    pub fn new(mass: f32) -> Self {
        Self {
            mass,
            angular_inertia: mass,
        }
    }

    /// Creates a mass component with custom angular inertia.
    pub fn with_inertia(mass: f32, angular_inertia: f32) -> Self {
        Self {
            mass,
            angular_inertia,
        }
    }
}

impl Default for Mass {
    fn default() -> Self {
        Self::new(1.0)
    }
}

/// Friction component defining surface friction coefficient.
///
/// Controls how much friction is applied during contact with other bodies.
/// Higher values create more friction (objects slide less).
///
/// # Example
///
/// ```rust,no_run
/// use praxis_physics::{Friction, RigidBody, Collider};
/// use praxis_ecs::{World, Transform};
///
/// let mut world = World::new();
///
/// // Slippery surface (ice)
/// world.spawn((
///     Transform::default(),
///     RigidBody::Static,
///     Collider::cuboid(10.0, 0.5, 10.0),
///     Friction::new(0.1),
/// ));
///
/// // High friction surface (rubber)
/// world.spawn((
///     Transform::default(),
///     RigidBody::Static,
///     Collider::cuboid(10.0, 0.5, 10.0),
///     Friction::new(1.5),
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy)]
pub struct Friction {
    /// Friction coefficient (typically 0.0 to 2.0).
    pub coefficient: f32,
}

impl Friction {
    /// Creates a friction component with the given coefficient.
    pub fn new(coefficient: f32) -> Self {
        Self { coefficient }
    }
}

impl Default for Friction {
    fn default() -> Self {
        Self::new(0.5)
    }
}

/// Restitution component defining bounciness.
///
/// Controls how much kinetic energy is preserved during collisions.
/// - 0.0 = no bounce (perfectly inelastic)
/// - 1.0 = perfect bounce (perfectly elastic)
/// - >1.0 = gains energy (unusual but possible)
///
/// # Example
///
/// ```rust,no_run
/// use praxis_physics::{Restitution, RigidBody, Collider};
/// use praxis_ecs::{World, Transform};
///
/// let mut world = World::new();
///
/// // Bouncy ball
/// world.spawn((
///     Transform::from_xyz(0.0, 10.0, 0.0),
///     RigidBody::Dynamic,
///     Collider::sphere(0.5),
///     Restitution::new(0.8),
/// ));
///
/// // Non-bouncy box
/// world.spawn((
///     Transform::from_xyz(0.0, 5.0, 0.0),
///     RigidBody::Dynamic,
///     Collider::cuboid(1.0, 1.0, 1.0),
///     Restitution::new(0.0),
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy)]
pub struct Restitution {
    /// Restitution coefficient (typically 0.0 to 1.0).
    pub coefficient: f32,
}

impl Restitution {
    /// Creates a restitution component with the given coefficient.
    pub fn new(coefficient: f32) -> Self {
        Self { coefficient }
    }
}

impl Default for Restitution {
    fn default() -> Self {
        Self::new(0.0)
    }
}

/// Collision groups component for filtering what collides with what.
///
/// Uses bit masks to control which bodies can collide with each other.
/// Each body belongs to one or more groups and can collide with bodies
/// in specified groups.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_physics::{CollisionGroups, RigidBody, Collider};
/// use praxis_ecs::{World, Transform};
///
/// let mut world = World::new();
///
/// // Player (group 0) collides with enemies (group 1) and terrain (group 2)
/// world.spawn((
///     Transform::default(),
///     RigidBody::Dynamic,
///     Collider::capsule_y(0.5, 1.0),
///     CollisionGroups::new(0b001, 0b110),
/// ));
///
/// // Enemy (group 1) collides with player (group 0) and terrain (group 2)
/// world.spawn((
///     Transform::default(),
///     RigidBody::Dynamic,
///     Collider::capsule_y(0.5, 1.0),
///     CollisionGroups::new(0b010, 0b101),
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy)]
pub struct CollisionGroups {
    /// Bit mask of groups this body belongs to.
    pub memberships: u32,
    
    /// Bit mask of groups this body can collide with.
    pub filter: u32,
}

impl CollisionGroups {
    /// Creates collision groups with specified membership and filter.
    pub fn new(memberships: u32, filter: u32) -> Self {
        Self { memberships, filter }
    }

    /// Creates collision groups that collide with everything.
    pub fn all() -> Self {
        Self {
            memberships: u32::MAX,
            filter: u32::MAX,
        }
    }

    /// Creates collision groups for a specific group index (0-31).
    pub fn group(group: u32) -> Self {
        let mask = 1 << group;
        Self {
            memberships: mask,
            filter: u32::MAX,
        }
    }
}

impl Default for CollisionGroups {
    fn default() -> Self {
        Self::all()
    }
}

/// Sleeping component controlling whether a body can sleep.
///
/// Sleeping is a performance optimization where bodies at rest are not
/// simulated until disturbed. This component controls sleep behavior.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_physics::{Sleeping, RigidBody, Collider};
/// use praxis_ecs::{World, Transform};
///
/// let mut world = World::new();
///
/// // Body that can sleep when at rest
/// world.spawn((
///     Transform::from_xyz(0.0, 10.0, 0.0),
///     RigidBody::Dynamic,
///     Collider::cuboid(1.0, 1.0, 1.0),
///     Sleeping::default(),
/// ));
///
/// // Body that never sleeps (always simulated)
/// world.spawn((
///     Transform::from_xyz(0.0, 5.0, 0.0),
///     RigidBody::Dynamic,
///     Collider::sphere(0.5),
///     Sleeping::disabled(),
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy)]
pub struct Sleeping {
    /// Whether sleeping is enabled.
    pub enabled: bool,
    
    /// Linear velocity threshold for sleeping.
    pub linear_threshold: f32,
    
    /// Angular velocity threshold for sleeping.
    pub angular_threshold: f32,
}

impl Sleeping {
    /// Creates a sleeping component with default thresholds.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a sleeping component with sleeping disabled.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Creates a sleeping component with custom thresholds.
    pub fn with_thresholds(linear_threshold: f32, angular_threshold: f32) -> Self {
        Self {
            enabled: true,
            linear_threshold,
            angular_threshold,
        }
    }
}

impl Default for Sleeping {
    fn default() -> Self {
        Self {
            enabled: true,
            linear_threshold: 0.01,
            angular_threshold: 0.01,
        }
    }
}

/// Sensor component marking a collider as a sensor (trigger).
///
/// Sensors detect overlaps but don't generate collision responses.
/// They're useful for trigger volumes, detection zones, etc.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_physics::{Sensor, RigidBody, Collider};
/// use praxis_ecs::{World, Transform};
///
/// let mut world = World::new();
///
/// // Trigger volume that detects when player enters
/// world.spawn((
///     Transform::from_xyz(10.0, 0.0, 0.0),
///     RigidBody::Static,
///     Collider::cuboid(2.0, 2.0, 2.0),
///     Sensor,
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Sensor;

/// Locked axes component for constraining rigid body motion.
///
/// Prevents movement or rotation along specified axes. Useful for
/// creating 2.5D gameplay or restricting physics behavior.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_physics::{LockedAxes, RigidBody, Collider};
/// use praxis_ecs::{World, Transform};
///
/// let mut world = World::new();
///
/// // Character that can't rotate (stays upright)
/// world.spawn((
///     Transform::default(),
///     RigidBody::Dynamic,
///     Collider::capsule_y(0.5, 1.0),
///     LockedAxes::rotation(),
/// ));
///
/// // Object that can only move in XZ plane (2.5D)
/// world.spawn((
///     Transform::default(),
///     RigidBody::Dynamic,
///     Collider::cuboid(1.0, 1.0, 1.0),
///     LockedAxes::new().lock_translation_y().lock_rotation_x().lock_rotation_z(),
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct LockedAxes {
    /// Lock translation along X axis.
    pub lock_translation_x: bool,
    /// Lock translation along Y axis.
    pub lock_translation_y: bool,
    /// Lock translation along Z axis.
    pub lock_translation_z: bool,
    /// Lock rotation around X axis.
    pub lock_rotation_x: bool,
    /// Lock rotation around Y axis.
    pub lock_rotation_y: bool,
    /// Lock rotation around Z axis.
    pub lock_rotation_z: bool,
}

impl LockedAxes {
    /// Creates an unlocked axes configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Locks all translation axes (no linear movement).
    pub fn translation() -> Self {
        Self {
            lock_translation_x: true,
            lock_translation_y: true,
            lock_translation_z: true,
            ..Default::default()
        }
    }

    /// Locks all rotation axes (no angular movement).
    pub fn rotation() -> Self {
        Self {
            lock_rotation_x: true,
            lock_rotation_y: true,
            lock_rotation_z: true,
            ..Default::default()
        }
    }

    /// Locks translation along X axis.
    pub fn lock_translation_x(mut self) -> Self {
        self.lock_translation_x = true;
        self
    }

    /// Locks translation along Y axis.
    pub fn lock_translation_y(mut self) -> Self {
        self.lock_translation_y = true;
        self
    }

    /// Locks translation along Z axis.
    pub fn lock_translation_z(mut self) -> Self {
        self.lock_translation_z = true;
        self
    }

    /// Locks rotation around X axis.
    pub fn lock_rotation_x(mut self) -> Self {
        self.lock_rotation_x = true;
        self
    }

    /// Locks rotation around Y axis.
    pub fn lock_rotation_y(mut self) -> Self {
        self.lock_rotation_y = true;
        self
    }

    /// Locks rotation around Z axis.
    pub fn lock_rotation_z(mut self) -> Self {
        self.lock_rotation_z = true;
        self
    }
}
