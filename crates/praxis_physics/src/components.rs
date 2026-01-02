//! Physics components for the Praxis ECS.
//!
//! This module provides components that define physics properties for entities.
//! Components are designed to be intuitive and match common physics concepts.

use bevy_ecs::component::Component;
use praxis_math::Vec3;

/// Rigid body component defining the physics behavior type.
///
/// A rigid body is a solid object that maintains its shape during physics simulation.
/// This component determines how the entity participates in physics simulation:
///
/// ## Variants
///
/// - **Dynamic**: Fully simulated rigid body affected by forces, gravity, and collisions.
///   Dynamic bodies have mass and respond to all physical interactions. Use this for
///   objects that should fall, bounce, and react to physics naturally (e.g., balls,
///   boxes, ragdolls, vehicles).
///
/// - **Static**: Immovable rigid body that never moves during simulation. Static bodies
///   have infinite mass and are unaffected by any forces or collisions, but they can
///   affect other bodies through collisions. Use this for terrain, walls, buildings,
///   and other permanent level geometry that should never move.
///
/// - **Kinematic**: Rigid body moved by animation or code rather than physics forces.
///   Kinematic bodies affect dynamic bodies through collisions but are not affected
///   by forces, gravity, or collisions with other bodies. Use this for moving platforms,
///   elevators, doors, and other objects with scripted or animated movement that should
///   push dynamic objects but not be pushed back.
///
/// # Physical Meaning
///
/// In classical mechanics, a rigid body is an idealization where the distance between
/// any two points on the object remains constant regardless of external forces. This
/// component controls whether the body's motion is computed by the physics engine
/// (Dynamic), fixed in place (Static), or controlled externally (Kinematic).
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
    ///
    /// Dynamic bodies are fully simulated and respond to:
    /// - Gravity and external forces
    /// - Collisions with all body types
    /// - Constraints and joints
    ///
    /// Physical properties like mass, velocity, and acceleration apply.
    Dynamic,
    
    /// Static body that never moves.
    ///
    /// Static bodies:
    /// - Have infinite mass (immovable)
    /// - Do not respond to any forces
    /// - Can collide with dynamic and kinematic bodies
    /// - Are optimized for stationary objects
    ///
    /// Most efficient body type for non-moving geometry.
    Static,
    
    /// Kinematic body controlled by animation or code.
    ///
    /// Kinematic bodies:
    /// - Are moved by setting their position/velocity directly
    /// - Push dynamic bodies but are not pushed back
    /// - Ignore forces and gravity
    /// - Useful for player-controlled characters and scripted objects
    Kinematic,
}

impl RigidBody {
    /// Returns true if this is a dynamic body.
    #[must_use]
    pub const fn is_dynamic(&self) -> bool {
        matches!(self, Self::Dynamic)
    }

    /// Returns true if this is a static body.
    #[must_use]
    pub const fn is_static(&self) -> bool {
        matches!(self, Self::Static)
    }

    /// Returns true if this is a kinematic body.
    #[must_use]
    pub const fn is_kinematic(&self) -> bool {
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
/// A collider defines the shape used for collision detection and physical response.
/// The shape determines how the object interacts with other physical objects in the
/// simulation. Colliders can be attached to rigid bodies for physics simulation or
/// used as sensors (triggers) that detect overlaps without physical response.
///
/// # Physical Meaning
///
/// In physics simulation, collision detection requires computational geometry to
/// determine when and where objects intersect. Different shapes have different
/// performance characteristics and accuracy trade-offs:
///
/// - **Primitive shapes** (sphere, box, capsule) are fast and numerically stable
/// - **Spheres** are the fastest but least accurate for non-spherical objects
/// - **Boxes** (cuboids) are good general-purpose shapes
/// - **Capsules** are excellent for characters (smooth collision, upright stability)
/// - **Cylinders** are useful for wheels and cylindrical objects
///
/// The dimensions are specified as "half-extents" (half the total size) because
/// physics engines typically work with distance from center, making the math
/// simpler and more symmetric.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_physics::Collider;
/// use praxis_ecs::{World, Transform};
///
/// let mut world = World::new();
///
/// // Box collider (2x2x2 total size)
/// world.spawn((
///     Transform::default(),
///     Collider::cuboid(1.0, 1.0, 1.0),
/// ));
///
/// // Sphere collider (radius 0.5)
/// world.spawn((
///     Transform::default(),
///     Collider::sphere(0.5),
/// ));
///
/// // Capsule collider for character (1.0 radius, 2.0 height cylindrical segment)
/// world.spawn((
///     Transform::default(),
///     Collider::capsule_y(1.0, 0.5),
/// ));
/// ```
#[derive(Component, Debug, Clone)]
pub enum Collider {
    /// Box-shaped collider with half-extents.
    ///
    /// A rectangular box (cuboid) is defined by three half-extents representing
    /// half the width, height, and depth. The total dimensions are double these values.
    Cuboid {
        /// Half-width (x-axis) in world units.
        ///
        /// The full width of the box is 2 * hx. This is the distance from the
        /// center to the left/right faces.
        hx: f32,
        
        /// Half-height (y-axis) in world units.
        ///
        /// The full height of the box is 2 * hy. This is the distance from the
        /// center to the top/bottom faces.
        hy: f32,
        
        /// Half-depth (z-axis) in world units.
        ///
        /// The full depth of the box is 2 * hz. This is the distance from the
        /// center to the front/back faces.
        hz: f32,
    },
    
    /// Sphere collider with radius.
    ///
    /// A perfect sphere defined by its radius. Spheres are the most efficient
    /// collision shape and are ideal for balls, projectiles, and approximately
    /// spherical objects.
    Sphere {
        /// Radius of the sphere in world units.
        ///
        /// This is the distance from the center to the surface. The diameter
        /// is 2 * radius. Spheres have constant radius in all directions,
        /// making collision detection very fast.
        radius: f32,
    },
    
    /// Capsule collider aligned with Y-axis (vertical).
    ///
    /// A capsule is a cylinder with hemispherical caps on both ends. It's
    /// excellent for character controllers because it provides smooth collision
    /// response and naturally stays upright. The Y-axis alignment is standard
    /// for standing characters.
    CapsuleY {
        /// Half-height of the cylindrical segment in world units.
        ///
        /// This is half the height of the cylinder portion only, not including
        /// the hemispherical caps. Total capsule height is:
        /// 2 * (`half_height` + radius).
        half_height: f32,
        
        /// Radius of the cylindrical segment and hemispherical caps in world units.
        ///
        /// This defines the thickness of the capsule. The radius is constant
        /// along the entire length and applies to both the cylinder and the caps.
        radius: f32,
    },
    
    /// Capsule collider aligned with X-axis (horizontal).
    ///
    /// Same as `CapsuleY` but oriented along the X-axis. Useful for lying objects
    /// or horizontal movement constraints.
    CapsuleX {
        /// Half-height of the cylindrical segment in world units.
        ///
        /// This is half the length along the X-axis, not including the caps.
        half_height: f32,
        
        /// Radius of the cylindrical segment and hemispherical caps in world units.
        radius: f32,
    },
    
    /// Capsule collider aligned with Z-axis (horizontal).
    ///
    /// Same as `CapsuleY` but oriented along the Z-axis. Useful for depth-aligned
    /// objects or forward-facing constraints.
    CapsuleZ {
        /// Half-height of the cylindrical segment in world units.
        ///
        /// This is half the length along the Z-axis, not including the caps.
        half_height: f32,
        
        /// Radius of the cylindrical segment and hemispherical caps in world units.
        radius: f32,
    },
    
    /// Cylinder collider aligned with Y-axis (vertical).
    ///
    /// A cylinder is a round shape with flat top and bottom faces. Unlike a
    /// capsule, the edges are sharp, which can cause collision artifacts. Use
    /// cylinders for wheels, pillars, and objects where flat ends are important.
    CylinderY {
        /// Half-height of the cylinder in world units.
        ///
        /// This is the distance from the center to the top or bottom flat face.
        /// Total cylinder height is 2 * `half_height`.
        half_height: f32,
        
        /// Radius of the circular cross-section in world units.
        ///
        /// This defines the width of the cylinder. The diameter is 2 * radius.
        radius: f32,
    },
}

impl Collider {
    /// Creates a cuboid collider with the given half-extents.
    ///
    /// # Arguments
    ///
    /// * `hx` - Half-width (x-axis) - actual width will be 2 * hx
    /// * `hy` - Half-height (y-axis) - actual height will be 2 * hy
    /// * `hz` - Half-depth (z-axis) - actual depth will be 2 * hz
    #[must_use]
    pub const fn cuboid(hx: f32, hy: f32, hz: f32) -> Self {
        Self::Cuboid { hx, hy, hz }
    }

    /// Creates a sphere collider with the given radius.
    ///
    /// # Arguments
    ///
    /// * `radius` - Distance from center to surface
    #[must_use]
    pub const fn sphere(radius: f32) -> Self {
        Self::Sphere { radius }
    }

    /// Creates a Y-aligned capsule collider.
    ///
    /// # Arguments
    ///
    /// * `half_height` - Half-height of the cylindrical segment (not including caps)
    /// * `radius` - Radius of the capsule (applies to cylinder and caps)
    #[must_use]
    pub const fn capsule_y(half_height: f32, radius: f32) -> Self {
        Self::CapsuleY { half_height, radius }
    }

    /// Creates an X-aligned capsule collider.
    ///
    /// # Arguments
    ///
    /// * `half_height` - Half-length of the cylindrical segment along X-axis
    /// * `radius` - Radius of the capsule
    #[must_use]
    pub const fn capsule_x(half_height: f32, radius: f32) -> Self {
        Self::CapsuleX { half_height, radius }
    }

    /// Creates a Z-aligned capsule collider.
    ///
    /// # Arguments
    ///
    /// * `half_height` - Half-length of the cylindrical segment along Z-axis
    /// * `radius` - Radius of the capsule
    #[must_use]
    pub const fn capsule_z(half_height: f32, radius: f32) -> Self {
        Self::CapsuleZ { half_height, radius }
    }

    /// Creates a Y-aligned cylinder collider.
    ///
    /// # Arguments
    ///
    /// * `half_height` - Half-height of the cylinder
    /// * `radius` - Radius of the circular cross-section
    #[must_use]
    pub const fn cylinder_y(half_height: f32, radius: f32) -> Self {
        Self::CylinderY { half_height, radius }
    }
}

/// Physics velocity component for dynamic and kinematic bodies.
///
/// Velocity describes the rate of change of position (linear velocity) and
/// orientation (angular velocity) of a rigid body. This is a fundamental
/// concept in classical mechanics.
///
/// # Physical Meaning
///
/// ## Linear Velocity
/// Linear velocity is the rate of change of position, measured in units per second.
/// It describes how fast and in what direction the object's center of mass is moving
/// through space. A velocity of Vec3(1.0, 0.0, 0.0) means the object moves 1 unit
/// in the positive X direction each second.
///
/// ## Angular Velocity
/// Angular velocity is the rate of change of orientation, measured in radians per second.
/// It describes how fast and around which axis the object is rotating. The direction
/// of the vector indicates the rotation axis (using the right-hand rule), and the
/// magnitude indicates the rotation speed. For example, Vec3(0.0, 1.0, 0.0) means
/// the object rotates around the Y-axis at 1 radian per second (about 57.3 degrees
/// per second).
///
/// ## Usage
///
/// For **dynamic bodies**, velocities are computed and updated by the physics engine
/// based on forces, collisions, and constraints. You can read the velocity to know
/// how the object is moving, or set it to give the object an initial velocity or
/// change its motion directly.
///
/// For **kinematic bodies**, velocities can be set directly to create movement.
/// The physics engine uses the velocity to move the body and detect collisions,
/// but does not modify the velocity based on forces or collisions.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_physics::{PhysicsVelocity, RigidBody};
/// use praxis_ecs::{World, Transform};
/// use praxis_math::Vec3;
///
/// let mut world = World::new();
///
/// // Create a body with initial velocity
/// world.spawn((
///     Transform::from_xyz(0.0, 10.0, 0.0),
///     RigidBody::Dynamic,
///     PhysicsVelocity::linear(Vec3::new(1.0, 0.0, 0.0)),
/// ));
///
/// // Create a spinning object
/// world.spawn((
///     Transform::default(),
///     RigidBody::Dynamic,
///     PhysicsVelocity::angular(Vec3::new(0.0, 3.14, 0.0)), // Spinning around Y-axis
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PhysicsVelocity {
    /// Linear velocity vector in units per second.
    ///
    /// This is the rate of change of the body's position in world space.
    /// Each component represents velocity along one axis:
    /// - `x`: Velocity in the X direction (left/right)
    /// - `y`: Velocity in the Y direction (up/down)
    /// - `z`: Velocity in the Z direction (forward/back)
    ///
    /// The magnitude (length) of the vector is the speed, and the direction
    /// is the direction of movement. For example:
    /// - Vec3(5.0, 0.0, 0.0) = moving right at 5 units/sec
    /// - Vec3(0.0, -9.8, 0.0) = falling at 9.8 units/sec
    /// - Vec3(3.0, 4.0, 0.0) = moving at 5 units/sec at an angle
    pub linear: Vec3,
    
    /// Angular velocity vector in radians per second.
    ///
    /// This is the rate of change of the body's orientation. The vector
    /// represents rotation using axis-angle representation:
    /// - The direction of the vector is the axis of rotation
    /// - The magnitude is the rotation speed in radians per second
    ///
    /// Uses the right-hand rule: point your right thumb along the vector,
    /// your fingers curl in the direction of rotation.
    ///
    /// For example:
    /// - Vec3(0.0, 1.0, 0.0) = rotating counterclockwise around Y-axis at 1 rad/sec
    /// - Vec3(3.14, 0.0, 0.0) = rotating around X-axis at π rad/sec (180°/sec)
    /// - Vec3(0.0, 0.0, 0.0) = not rotating
    pub angular: Vec3,
}

impl PhysicsVelocity {
    /// Creates a velocity with only linear component.
    ///
    /// # Arguments
    ///
    /// * `linear` - Linear velocity in units per second
    #[must_use]
    pub const fn linear(linear: Vec3) -> Self {
        Self {
            linear,
            angular: Vec3::ZERO,
        }
    }

    /// Creates a velocity with only angular component.
    ///
    /// # Arguments
    ///
    /// * `angular` - Angular velocity in radians per second
    #[must_use]
    pub const fn angular(angular: Vec3) -> Self {
        Self {
            linear: Vec3::ZERO,
            angular,
        }
    }

    /// Creates a velocity with both linear and angular components.
    ///
    /// # Arguments
    ///
    /// * `linear` - Linear velocity in units per second
    /// * `angular` - Angular velocity in radians per second
    #[must_use]
    pub const fn new(linear: Vec3, angular: Vec3) -> Self {
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
    pub const fn clear(&mut self) {
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
    #[must_use]
    pub const fn new(mass: f32) -> Self {
        Self {
            mass,
            angular_inertia: mass,
        }
    }

    /// Creates a mass component with custom angular inertia.
    #[must_use]
    pub const fn with_inertia(mass: f32, angular_inertia: f32) -> Self {
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
    #[must_use]
    pub const fn new(coefficient: f32) -> Self {
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
    #[must_use]
    pub const fn new(coefficient: f32) -> Self {
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
    #[must_use]
    pub const fn new(memberships: u32, filter: u32) -> Self {
        Self { memberships, filter }
    }

    /// Creates collision groups that collide with everything.
    #[must_use]
    pub const fn all() -> Self {
        Self {
            memberships: u32::MAX,
            filter: u32::MAX,
        }
    }

    /// Creates collision groups for a specific group index (0-31).
    #[must_use]
    pub const fn group(group: u32) -> Self {
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a sleeping component with sleeping disabled.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Creates a sleeping component with custom thresholds.
    #[must_use]
    pub const fn with_thresholds(linear_threshold: f32, angular_threshold: f32) -> Self {
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
#[allow(clippy::struct_excessive_bools)]
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Locks all translation axes (no linear movement).
    #[must_use]
    pub fn translation() -> Self {
        Self {
            lock_translation_x: true,
            lock_translation_y: true,
            lock_translation_z: true,
            ..Default::default()
        }
    }

    /// Locks all rotation axes (no angular movement).
    #[must_use]
    pub fn rotation() -> Self {
        Self {
            lock_rotation_x: true,
            lock_rotation_y: true,
            lock_rotation_z: true,
            ..Default::default()
        }
    }

    /// Locks translation along X axis.
    #[must_use]
    pub const fn lock_translation_x(mut self) -> Self {
        self.lock_translation_x = true;
        self
    }

    /// Locks translation along Y axis.
    #[must_use]
    pub const fn lock_translation_y(mut self) -> Self {
        self.lock_translation_y = true;
        self
    }

    /// Locks translation along Z axis.
    #[must_use]
    pub const fn lock_translation_z(mut self) -> Self {
        self.lock_translation_z = true;
        self
    }

    /// Locks rotation around X axis.
    #[must_use]
    pub const fn lock_rotation_x(mut self) -> Self {
        self.lock_rotation_x = true;
        self
    }

    /// Locks rotation around Y axis.
    #[must_use]
    pub const fn lock_rotation_y(mut self) -> Self {
        self.lock_rotation_y = true;
        self
    }

    /// Locks rotation around Z axis.
    #[must_use]
    pub const fn lock_rotation_z(mut self) -> Self {
        self.lock_rotation_z = true;
        self
    }
}
