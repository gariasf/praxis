//! Joint constraints for connecting rigid bodies.
//!
//! This module provides various types of joint constraints that can be used to
//! connect rigid bodies together with different degrees of freedom.

#![allow(clippy::missing_const_for_fn)]

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use praxis_math::Vec3;

/// Hinge joint that constrains two bodies to rotate around a common axis.
///
/// A hinge joint (also called a revolute joint) allows rotation around one axis
/// while constraining all other degrees of freedom. This is like a door hinge or
/// a wheel axle.
#[derive(Component, Debug, Clone)]
pub struct HingeJoint {
    /// The other entity this joint connects to.
    pub connected_entity: Entity,
    
    /// Local anchor point on this body.
    pub local_anchor1: Vec3,
    
    /// Local anchor point on the connected body.
    pub local_anchor2: Vec3,
    
    /// Rotation axis in local space of the first body.
    pub local_axis1: Vec3,
    
    /// Rotation axis in local space of the second body.
    pub local_axis2: Vec3,
    
    /// Whether the joint has rotation limits.
    pub limits_enabled: bool,
    
    /// Minimum rotation angle in radians (if limits enabled).
    pub min_angle: f32,
    
    /// Maximum rotation angle in radians (if limits enabled).
    pub max_angle: f32,
    
    /// Motor target velocity in radians per second.
    pub motor_velocity: f32,
    
    /// Maximum motor force/torque.
    pub motor_max_force: f32,
}

impl HingeJoint {
    /// Creates a new hinge joint between two entities.
    #[must_use]
    pub fn new(connected_entity: Entity, local_anchor1: Vec3, local_anchor2: Vec3) -> Self {
        Self {
            connected_entity,
            local_anchor1,
            local_anchor2,
            local_axis1: Vec3::Y,
            local_axis2: Vec3::Y,
            limits_enabled: false,
            min_angle: 0.0,
            max_angle: 0.0,
            motor_velocity: 0.0,
            motor_max_force: 0.0,
        }
    }
    
    /// Sets the rotation axis for both bodies.
    #[must_use]
    pub fn with_axis(mut self, axis1: Vec3, axis2: Vec3) -> Self {
        self.local_axis1 = axis1;
        self.local_axis2 = axis2;
        self
    }
    
    /// Sets rotation limits for the hinge.
    #[must_use]
    pub fn with_limits(mut self, min_angle: f32, max_angle: f32) -> Self {
        self.limits_enabled = true;
        self.min_angle = min_angle;
        self.max_angle = max_angle;
        self
    }
    
    /// Sets a motor for the hinge joint.
    #[must_use]
    pub fn with_motor(mut self, velocity: f32, max_force: f32) -> Self {
        self.motor_velocity = velocity;
        self.motor_max_force = max_force;
        self
    }
}

/// Ball-and-socket joint that allows free rotation around a point.
///
/// A ball joint (also called a spherical joint) constrains the position but
/// allows free rotation in all directions. Like a shoulder joint or a joystick.
#[derive(Component, Debug, Clone)]
pub struct BallJoint {
    /// The other entity this joint connects to.
    pub connected_entity: Entity,
    
    /// Local anchor point on this body.
    pub local_anchor1: Vec3,
    
    /// Local anchor point on the connected body.
    pub local_anchor2: Vec3,
}

impl BallJoint {
    /// Creates a new ball joint between two entities.
    #[must_use]
    pub const fn new(connected_entity: Entity, local_anchor1: Vec3, local_anchor2: Vec3) -> Self {
        Self {
            connected_entity,
            local_anchor1,
            local_anchor2,
        }
    }
}

/// Slider joint that constrains two bodies to translate along an axis.
///
/// A slider joint (also called a prismatic joint) allows translation along one
/// axis while constraining all rotation and translation in other directions.
/// Like a drawer or a piston.
#[derive(Component, Debug, Clone)]
pub struct SliderJoint {
    /// The other entity this joint connects to.
    pub connected_entity: Entity,
    
    /// Local anchor point on this body.
    pub local_anchor1: Vec3,
    
    /// Local anchor point on the connected body.
    pub local_anchor2: Vec3,
    
    /// Slide axis in local space of the first body.
    pub local_axis1: Vec3,
    
    /// Slide axis in local space of the second body.
    pub local_axis2: Vec3,
    
    /// Whether the joint has translation limits.
    pub limits_enabled: bool,
    
    /// Minimum translation distance along axis (if limits enabled).
    pub min_distance: f32,
    
    /// Maximum translation distance along axis (if limits enabled).
    pub max_distance: f32,
    
    /// Motor target velocity along the axis.
    pub motor_velocity: f32,
    
    /// Maximum motor force.
    pub motor_max_force: f32,
}

impl SliderJoint {
    /// Creates a new slider joint between two entities.
    #[must_use]
    pub fn new(connected_entity: Entity, local_anchor1: Vec3, local_anchor2: Vec3) -> Self {
        Self {
            connected_entity,
            local_anchor1,
            local_anchor2,
            local_axis1: Vec3::X,
            local_axis2: Vec3::X,
            limits_enabled: false,
            min_distance: 0.0,
            max_distance: 0.0,
            motor_velocity: 0.0,
            motor_max_force: 0.0,
        }
    }
    
    /// Sets the slide axis for both bodies.
    #[must_use]
    pub fn with_axis(mut self, axis1: Vec3, axis2: Vec3) -> Self {
        self.local_axis1 = axis1;
        self.local_axis2 = axis2;
        self
    }
    
    /// Sets translation limits for the slider.
    #[must_use]
    pub fn with_limits(mut self, min_distance: f32, max_distance: f32) -> Self {
        self.limits_enabled = true;
        self.min_distance = min_distance;
        self.max_distance = max_distance;
        self
    }
    
    /// Sets a motor for the slider joint.
    #[must_use]
    pub fn with_motor(mut self, velocity: f32, max_force: f32) -> Self {
        self.motor_velocity = velocity;
        self.motor_max_force = max_force;
        self
    }
}

/// Spring joint that applies spring forces between two bodies.
///
/// A spring joint connects two bodies with a spring-damper system that tries
/// to maintain a target distance or configuration.
#[derive(Component, Debug, Clone)]
pub struct SpringJoint {
    /// The other entity this joint connects to.
    pub connected_entity: Entity,
    
    /// Local anchor point on this body.
    pub local_anchor1: Vec3,
    
    /// Local anchor point on the connected body.
    pub local_anchor2: Vec3,
    
    /// Rest length of the spring (target distance).
    pub rest_length: f32,
    
    /// Spring stiffness coefficient (higher = stiffer).
    pub stiffness: f32,
    
    /// Damping coefficient (higher = more damping).
    pub damping: f32,
}

impl SpringJoint {
    /// Creates a new spring joint between two entities.
    #[must_use]
    pub const fn new(
        connected_entity: Entity,
        local_anchor1: Vec3,
        local_anchor2: Vec3,
        rest_length: f32,
    ) -> Self {
        Self {
            connected_entity,
            local_anchor1,
            local_anchor2,
            rest_length,
            stiffness: 100.0,
            damping: 5.0,
        }
    }
    
    /// Sets the spring stiffness.
    #[must_use]
    pub const fn with_stiffness(mut self, stiffness: f32) -> Self {
        self.stiffness = stiffness;
        self
    }
    
    /// Sets the spring damping.
    #[must_use]
    pub const fn with_damping(mut self, damping: f32) -> Self {
        self.damping = damping;
        self
    }
}

/// Fixed joint that rigidly connects two bodies.
///
/// A fixed joint constrains all degrees of freedom, making the two bodies
/// behave as if they were welded together.
#[derive(Component, Debug, Clone)]
pub struct FixedJoint {
    /// The other entity this joint connects to.
    pub connected_entity: Entity,
    
    /// Local anchor point on this body.
    pub local_anchor1: Vec3,
    
    /// Local anchor point on the connected body.
    pub local_anchor2: Vec3,
}

impl FixedJoint {
    /// Creates a new fixed joint between two entities.
    #[must_use]
    pub const fn new(connected_entity: Entity, local_anchor1: Vec3, local_anchor2: Vec3) -> Self {
        Self {
            connected_entity,
            local_anchor1,
            local_anchor2,
        }
    }
}
