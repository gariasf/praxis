//! Vehicle physics simulation using wheel colliders and suspension.
//!
//! This module provides components and systems for realistic vehicle simulation
//! with wheel physics, suspension, and traction.

#![allow(clippy::missing_const_for_fn)]

use bevy_ecs::component::Component;
use praxis_math::Vec3;

/// Vehicle component that marks an entity as a vehicle chassis.
///
/// The vehicle is controlled through acceleration, steering, and braking inputs.
#[derive(Component, Debug, Clone)]
pub struct Vehicle {
    /// Current steering angle in radians (-1.0 to 1.0, normalized).
    pub steering: f32,
    
    /// Current throttle input (0.0 to 1.0).
    pub throttle: f32,
    
    /// Current brake input (0.0 to 1.0).
    pub brake: f32,
    
    /// Maximum steering angle in radians.
    pub max_steering_angle: f32,
    
    /// Engine torque in Newton-meters.
    pub engine_torque: f32,
    
    /// Brake force in Newtons.
    pub brake_force: f32,
    
    /// Center of mass offset from the vehicle's transform origin.
    pub center_of_mass: Vec3,
}

impl Vehicle {
    /// Creates a new vehicle with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            steering: 0.0,
            throttle: 0.0,
            brake: 0.0,
            max_steering_angle: 0.5,
            engine_torque: 500.0,
            brake_force: 1500.0,
            center_of_mass: Vec3::new(0.0, -0.5, 0.0),
        }
    }
    
    /// Sets the steering input (-1.0 to 1.0).
    pub fn set_steering(&mut self, steering: f32) {
        self.steering = steering.clamp(-1.0, 1.0);
    }
    
    /// Sets the throttle input (0.0 to 1.0).
    pub fn set_throttle(&mut self, throttle: f32) {
        self.throttle = throttle.clamp(0.0, 1.0);
    }
    
    /// Sets the brake input (0.0 to 1.0).
    pub fn set_brake(&mut self, brake: f32) {
        self.brake = brake.clamp(0.0, 1.0);
    }
}

impl Default for Vehicle {
    fn default() -> Self {
        Self::new()
    }
}

/// Wheel collider component for vehicle wheels.
///
/// Each wheel has its own suspension, collision detection, and traction simulation.
#[derive(Component, Debug, Clone)]
pub struct WheelCollider {
    /// Wheel radius in meters.
    pub radius: f32,
    
    /// Wheel width in meters (for collision detection).
    pub width: f32,
    
    /// Mass of the wheel in kilograms.
    pub mass: f32,
    
    /// Local position of the wheel relative to the vehicle chassis.
    pub local_position: Vec3,
    
    /// Whether this wheel receives steering input.
    pub steerable: bool,
    
    /// Whether this wheel receives engine torque.
    pub powered: bool,
    
    /// Suspension configuration.
    pub suspension: WheelSuspension,
    
    /// Current suspension compression (0.0 = fully extended, 1.0 = fully compressed).
    pub suspension_compression: f32,
    
    /// Current wheel rotation angle in radians.
    pub rotation_angle: f32,
    
    /// Current wheel angular velocity in radians per second.
    pub angular_velocity: f32,
    
    /// Whether the wheel is currently grounded.
    pub is_grounded: bool,
    
    /// Ground contact normal (if grounded).
    pub ground_normal: Vec3,
    
    /// Ground contact point in world space (if grounded).
    pub ground_point: Vec3,
}

impl WheelCollider {
    /// Creates a new wheel collider.
    #[must_use]
    pub fn new(local_position: Vec3, radius: f32) -> Self {
        Self {
            radius,
            width: 0.2,
            mass: 20.0,
            local_position,
            steerable: false,
            powered: false,
            suspension: WheelSuspension::default(),
            suspension_compression: 0.0,
            rotation_angle: 0.0,
            angular_velocity: 0.0,
            is_grounded: false,
            ground_normal: Vec3::Y,
            ground_point: Vec3::ZERO,
        }
    }
    
    /// Marks this wheel as steerable (receives steering input).
    #[must_use]
    pub const fn steerable(mut self) -> Self {
        self.steerable = true;
        self
    }
    
    /// Marks this wheel as powered (receives engine torque).
    #[must_use]
    pub const fn powered(mut self) -> Self {
        self.powered = true;
        self
    }
    
    /// Sets the wheel width.
    #[must_use]
    pub const fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    
    /// Sets the wheel mass.
    #[must_use]
    pub const fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }
    
    /// Sets the suspension configuration.
    #[must_use]
    pub const fn with_suspension(mut self, suspension: WheelSuspension) -> Self {
        self.suspension = suspension;
        self
    }
}

/// Wheel suspension configuration.
///
/// Simulates spring-damper suspension system for realistic vehicle handling.
#[derive(Debug, Clone, Copy)]
pub struct WheelSuspension {
    /// Maximum suspension travel distance in meters.
    pub travel: f32,
    
    /// Spring stiffness in Newtons per meter.
    pub spring_stiffness: f32,
    
    /// Damping coefficient in Newton-seconds per meter.
    pub damping: f32,
    
    /// Rest position of the suspension (0.0 = fully extended, 1.0 = fully compressed).
    pub rest_position: f32,
}

impl WheelSuspension {
    /// Creates suspension with given parameters.
    #[must_use]
    pub const fn new(travel: f32, spring_stiffness: f32, damping: f32) -> Self {
        Self {
            travel,
            spring_stiffness,
            damping,
            rest_position: 0.5,
        }
    }
}

impl Default for WheelSuspension {
    fn default() -> Self {
        Self {
            travel: 0.3,
            spring_stiffness: 35000.0,
            damping: 4500.0,
            rest_position: 0.5,
        }
    }
}

/// Wheel friction configuration.
///
/// Defines how much grip the wheel has in different directions.
#[derive(Component, Debug, Clone, Copy)]
pub struct WheelFriction {
    /// Forward friction coefficient (for acceleration/braking).
    pub forward_grip: f32,
    
    /// Sideways friction coefficient (for cornering).
    pub sideways_grip: f32,
    
    /// Forward slip stiffness (how quickly grip is applied).
    pub forward_stiffness: f32,
    
    /// Sideways slip stiffness.
    pub sideways_stiffness: f32,
}

impl WheelFriction {
    /// Creates default friction settings for asphalt/tarmac.
    #[must_use]
    pub const fn asphalt() -> Self {
        Self {
            forward_grip: 1.0,
            sideways_grip: 1.0,
            forward_stiffness: 2.0,
            sideways_stiffness: 2.0,
        }
    }
    
    /// Creates friction settings for dirt/gravel.
    #[must_use]
    pub const fn dirt() -> Self {
        Self {
            forward_grip: 0.7,
            sideways_grip: 0.8,
            forward_stiffness: 1.5,
            sideways_stiffness: 1.5,
        }
    }
    
    /// Creates friction settings for ice.
    #[must_use]
    pub const fn ice() -> Self {
        Self {
            forward_grip: 0.2,
            sideways_grip: 0.2,
            forward_stiffness: 0.5,
            sideways_stiffness: 0.5,
        }
    }
}

impl Default for WheelFriction {
    fn default() -> Self {
        Self::asphalt()
    }
}

/// Anti-roll bar (stabilizer bar) for vehicle stability.
///
/// Connects left and right wheels to reduce body roll during cornering.
#[derive(Component, Debug, Clone, Copy)]
pub struct AntiRollBar {
    /// Stiffness of the anti-roll bar in Newton-meters per radian.
    pub stiffness: f32,
}

impl AntiRollBar {
    /// Creates a new anti-roll bar with specified stiffness.
    #[must_use]
    pub const fn new(stiffness: f32) -> Self {
        Self { stiffness }
    }
}

impl Default for AntiRollBar {
    fn default() -> Self {
        Self { stiffness: 5000.0 }
    }
}
