//! Ragdoll physics for character death animations and procedural movement.
//!
//! This module provides components for creating articulated ragdoll characters
//! with joint constraints and collision bodies.

#![allow(clippy::missing_const_for_fn, clippy::approx_constant)]

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use praxis_math::Vec3;

/// Ragdoll component that marks an entity as a ragdoll root.
///
/// A ragdoll consists of multiple rigid bodies connected by joints to simulate
/// a character's body with realistic physics.
#[derive(Component, Debug, Clone)]
pub struct Ragdoll {
    /// Whether the ragdoll is currently active (physics-driven).
    pub active: bool,
    
    /// Bone entities that make up this ragdoll.
    pub bones: Vec<RagdollBone>,
    
    /// Blend factor between animated and physics-driven poses (0.0 = animation, 1.0 = physics).
    pub physics_blend: f32,
    
    /// Time until ragdoll becomes fully active after activation.
    pub activation_time: f32,
    
    /// Current activation timer.
    pub activation_timer: f32,
}

impl Ragdoll {
    /// Creates a new inactive ragdoll.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active: false,
            bones: Vec::new(),
            physics_blend: 0.0,
            activation_time: 0.2,
            activation_timer: 0.0,
        }
    }
    
    /// Activates the ragdoll physics.
    pub fn activate(&mut self) {
        self.active = true;
        self.activation_timer = 0.0;
    }
    
    /// Deactivates the ragdoll physics.
    pub fn deactivate(&mut self) {
        self.active = false;
        self.physics_blend = 0.0;
        self.activation_timer = 0.0;
    }
    
    /// Adds a bone to the ragdoll.
    pub fn add_bone(&mut self, bone: RagdollBone) {
        self.bones.push(bone);
    }
}

impl Default for Ragdoll {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual bone in a ragdoll.
#[derive(Debug, Clone)]
pub struct RagdollBone {
    /// Entity representing this bone's rigid body.
    pub entity: Entity,
    
    /// Bone name/identifier.
    pub name: String,
    
    /// Parent bone index (None for root bone).
    pub parent: Option<usize>,
    
    /// Local offset from parent bone.
    pub local_offset: Vec3,
    
    /// Bone configuration.
    pub config: RagdollBoneConfig,
}

impl RagdollBone {
    /// Creates a new ragdoll bone.
    #[must_use]
    pub fn new(entity: Entity, name: String) -> Self {
        Self {
            entity,
            name,
            parent: None,
            local_offset: Vec3::ZERO,
            config: RagdollBoneConfig::default(),
        }
    }
    
    /// Sets the parent bone.
    #[must_use]
    pub const fn with_parent(mut self, parent: usize) -> Self {
        self.parent = Some(parent);
        self
    }
    
    /// Sets the local offset.
    #[must_use]
    pub const fn with_offset(mut self, offset: Vec3) -> Self {
        self.local_offset = offset;
        self
    }
    
    /// Sets the bone configuration.
    #[must_use]
    pub const fn with_config(mut self, config: RagdollBoneConfig) -> Self {
        self.config = config;
        self
    }
}

/// Configuration for a ragdoll bone.
#[derive(Debug, Clone, Copy)]
pub struct RagdollBoneConfig {
    /// Mass of the bone in kilograms.
    pub mass: f32,
    
    /// Angular damping (resistance to rotation).
    pub angular_damping: f32,
    
    /// Linear damping (resistance to translation).
    pub linear_damping: f32,
    
    /// Collision group for this bone.
    pub collision_group: u32,
}

impl RagdollBoneConfig {
    /// Creates configuration for a head bone.
    #[must_use]
    pub const fn head() -> Self {
        Self {
            mass: 5.0,
            angular_damping: 0.1,
            linear_damping: 0.05,
            collision_group: 1,
        }
    }
    
    /// Creates configuration for a torso/chest bone.
    #[must_use]
    pub const fn torso() -> Self {
        Self {
            mass: 15.0,
            angular_damping: 0.05,
            linear_damping: 0.05,
            collision_group: 1,
        }
    }
    
    /// Creates configuration for an upper arm bone.
    #[must_use]
    pub const fn upper_arm() -> Self {
        Self {
            mass: 3.0,
            angular_damping: 0.1,
            linear_damping: 0.05,
            collision_group: 2,
        }
    }
    
    /// Creates configuration for a lower arm bone.
    #[must_use]
    pub const fn lower_arm() -> Self {
        Self {
            mass: 2.0,
            angular_damping: 0.1,
            linear_damping: 0.05,
            collision_group: 2,
        }
    }
    
    /// Creates configuration for an upper leg bone.
    #[must_use]
    pub const fn upper_leg() -> Self {
        Self {
            mass: 8.0,
            angular_damping: 0.05,
            linear_damping: 0.05,
            collision_group: 3,
        }
    }
    
    /// Creates configuration for a lower leg bone.
    #[must_use]
    pub const fn lower_leg() -> Self {
        Self {
            mass: 5.0,
            angular_damping: 0.1,
            linear_damping: 0.05,
            collision_group: 3,
        }
    }
}

impl Default for RagdollBoneConfig {
    fn default() -> Self {
        Self {
            mass: 5.0,
            angular_damping: 0.1,
            linear_damping: 0.05,
            collision_group: 0,
        }
    }
}

/// Ragdoll joint constraint between bones.
#[derive(Component, Debug, Clone)]
pub struct RagdollJoint {
    /// Entity of the connected bone.
    pub connected_bone: Entity,
    
    /// Type of joint.
    pub joint_type: RagdollJointType,
    
    /// Local anchor on this bone.
    pub local_anchor: Vec3,
    
    /// Local anchor on the connected bone.
    pub connected_anchor: Vec3,
    
    /// Joint limits.
    pub limits: RagdollJointLimits,
}

impl RagdollJoint {
    /// Creates a new ragdoll joint.
    #[must_use]
    pub fn new(
        connected_bone: Entity,
        joint_type: RagdollJointType,
        local_anchor: Vec3,
        connected_anchor: Vec3,
    ) -> Self {
        Self {
            connected_bone,
            joint_type,
            local_anchor,
            connected_anchor,
            limits: RagdollJointLimits::default(),
        }
    }
    
    /// Sets the joint limits.
    #[must_use]
    pub const fn with_limits(mut self, limits: RagdollJointLimits) -> Self {
        self.limits = limits;
        self
    }
}

/// Type of ragdoll joint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RagdollJointType {
    /// Ball-and-socket joint (like shoulder, hip).
    Ball,
    /// Hinge joint (like elbow, knee).
    Hinge,
    /// Twist joint (like wrist).
    Twist,
}

/// Joint angle limits for ragdoll joints.
#[derive(Debug, Clone, Copy)]
pub struct RagdollJointLimits {
    /// Minimum angle/twist in radians.
    pub min_angle: f32,
    
    /// Maximum angle/twist in radians.
    pub max_angle: f32,
    
    /// Swing1 limit for ball joints (radians).
    pub swing1_limit: f32,
    
    /// Swing2 limit for ball joints (radians).
    pub swing2_limit: f32,
    
    /// Joint stiffness (0.0 to 1.0).
    pub stiffness: f32,
    
    /// Joint damping.
    pub damping: f32,
}

impl RagdollJointLimits {
    /// Creates shoulder joint limits (wide range of motion).
    #[must_use]
    pub const fn shoulder() -> Self {
        Self {
            min_angle: -3.14,
            max_angle: 3.14,
            swing1_limit: 1.57,
            swing2_limit: 1.57,
            stiffness: 0.8,
            damping: 0.1,
        }
    }
    
    /// Creates elbow joint limits (hinge with limited range).
    #[must_use]
    pub const fn elbow() -> Self {
        Self {
            min_angle: 0.0,
            max_angle: 2.35,
            swing1_limit: 0.0,
            swing2_limit: 0.0,
            stiffness: 0.9,
            damping: 0.05,
        }
    }
    
    /// Creates hip joint limits.
    #[must_use]
    pub const fn hip() -> Self {
        Self {
            min_angle: -3.14,
            max_angle: 3.14,
            swing1_limit: 1.0,
            swing2_limit: 0.78,
            stiffness: 0.85,
            damping: 0.1,
        }
    }
    
    /// Creates knee joint limits.
    #[must_use]
    pub const fn knee() -> Self {
        Self {
            min_angle: 0.0,
            max_angle: 2.35,
            swing1_limit: 0.0,
            swing2_limit: 0.0,
            stiffness: 0.9,
            damping: 0.05,
        }
    }
    
    /// Creates neck joint limits.
    #[must_use]
    pub const fn neck() -> Self {
        Self {
            min_angle: -0.78,
            max_angle: 0.78,
            swing1_limit: 0.52,
            swing2_limit: 0.52,
            stiffness: 0.95,
            damping: 0.2,
        }
    }
}

impl Default for RagdollJointLimits {
    fn default() -> Self {
        Self {
            min_angle: -1.57,
            max_angle: 1.57,
            swing1_limit: 0.78,
            swing2_limit: 0.78,
            stiffness: 0.8,
            damping: 0.1,
        }
    }
}

/// Ragdoll builder for common humanoid configurations.
pub struct RagdollBuilder {
    bones: Vec<RagdollBone>,
    scale: f32,
}

impl RagdollBuilder {
    /// Creates a new ragdoll builder with default scale.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bones: Vec::new(),
            scale: 1.0,
        }
    }
    
    /// Sets the scale factor for the ragdoll.
    #[must_use]
    pub const fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
    
    /// Builds the ragdoll component.
    #[must_use]
    pub fn build(self) -> Ragdoll {
        let mut ragdoll = Ragdoll::new();
        ragdoll.bones = self.bones;
        ragdoll
    }
}

impl Default for RagdollBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Motor settings for powered ragdoll joints.
#[derive(Component, Debug, Clone, Copy)]
pub struct RagdollMotor {
    /// Target angle or position for the motor.
    pub target: f32,
    
    /// Maximum force/torque the motor can apply.
    pub max_force: f32,
    
    /// Whether the motor is active.
    pub enabled: bool,
}

impl RagdollMotor {
    /// Creates a new ragdoll motor.
    #[must_use]
    pub const fn new(target: f32, max_force: f32) -> Self {
        Self {
            target,
            max_force,
            enabled: true,
        }
    }
    
    /// Disables the motor.
    #[must_use]
    pub const fn disabled() -> Self {
        Self {
            target: 0.0,
            max_force: 0.0,
            enabled: false,
        }
    }
}
