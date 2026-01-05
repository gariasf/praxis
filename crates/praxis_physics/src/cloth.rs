//! Cloth simulation using distance constraints and collision response.
//!
//! This module provides position-based dynamics cloth simulation with
//! structural, shear, and bend constraints.

use bevy_ecs::component::Component;
use praxis_math::Vec3;

/// Cloth simulation component.
///
/// Represents a cloth mesh simulated using position-based dynamics with
/// distance constraints for structural integrity.
#[derive(Component, Debug, Clone)]
pub struct Cloth {
    /// Cloth dimensions in particles (width x height).
    pub resolution: (usize, usize),

    /// Spacing between particles in meters.
    pub particle_spacing: f32,

    /// Mass per particle in kilograms.
    pub particle_mass: f32,

    /// Global damping factor (0.0 to 1.0).
    pub damping: f32,

    /// Air resistance coefficient.
    pub air_resistance: f32,

    /// Whether to use self-collision detection.
    pub self_collision: bool,

    /// Particles and their properties.
    pub particles: Vec<ClothParticle>,

    /// Distance constraints between particles.
    pub constraints: Vec<DistanceConstraint>,

    /// Fixed particles (indices that don't move).
    pub fixed_particles: Vec<usize>,
}

impl Cloth {
    /// Creates a new cloth with the specified resolution.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn new(width: usize, height: usize, particle_spacing: f32) -> Self {
        let particle_count = width * height;
        let mut particles = Vec::with_capacity(particle_count);

        for y in 0..height {
            for x in 0..width {
                particles.push(ClothParticle {
                    position: Vec3::new(
                        x as f32 * particle_spacing,
                        y as f32 * particle_spacing,
                        0.0,
                    ),
                    previous_position: Vec3::new(
                        x as f32 * particle_spacing,
                        y as f32 * particle_spacing,
                        0.0,
                    ),
                    velocity: Vec3::ZERO,
                    acceleration: Vec3::ZERO,
                    inverse_mass: 1.0,
                });
            }
        }

        let mut constraints = Vec::new();

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;

                if x < width - 1 {
                    let right_idx = y * width + (x + 1);
                    constraints.push(DistanceConstraint::structural(
                        idx,
                        right_idx,
                        particle_spacing,
                    ));
                }

                if y < height - 1 {
                    let down_idx = (y + 1) * width + x;
                    constraints.push(DistanceConstraint::structural(
                        idx,
                        down_idx,
                        particle_spacing,
                    ));
                }

                if x < width - 1 && y < height - 1 {
                    let diag_idx = (y + 1) * width + (x + 1);
                    constraints.push(DistanceConstraint::shear(
                        idx,
                        diag_idx,
                        particle_spacing * 1.414,
                    ));
                }

                if x > 0 && y < height - 1 {
                    let diag_idx = (y + 1) * width + (x - 1);
                    constraints.push(DistanceConstraint::shear(
                        idx,
                        diag_idx,
                        particle_spacing * 1.414,
                    ));
                }
            }
        }

        Self {
            resolution: (width, height),
            particle_spacing,
            particle_mass: 0.1,
            damping: 0.99,
            air_resistance: 0.01,
            self_collision: false,
            particles,
            constraints,
            fixed_particles: Vec::new(),
        }
    }

    /// Pins a particle at the specified grid position.
    pub fn pin_particle(&mut self, x: usize, y: usize) {
        let idx = y * self.resolution.0 + x;
        if idx < self.particles.len() {
            self.particles[idx].inverse_mass = 0.0;
            self.fixed_particles.push(idx);
        }
    }

    /// Pins the top edge of the cloth.
    pub fn pin_top_edge(&mut self) {
        for x in 0..self.resolution.0 {
            self.pin_particle(x, 0);
        }
    }

    /// Pins all corners of the cloth.
    pub fn pin_corners(&mut self) {
        self.pin_particle(0, 0);
        self.pin_particle(self.resolution.0 - 1, 0);
        self.pin_particle(0, self.resolution.1 - 1);
        self.pin_particle(self.resolution.0 - 1, self.resolution.1 - 1);
    }
}

/// Individual cloth particle.
#[derive(Debug, Clone, Copy)]
pub struct ClothParticle {
    /// Current position in world space.
    pub position: Vec3,

    /// Previous position (for Verlet integration).
    pub previous_position: Vec3,

    /// Current velocity.
    pub velocity: Vec3,

    /// Accumulated acceleration.
    pub acceleration: Vec3,

    /// Inverse mass (0.0 = infinite mass/fixed particle).
    pub inverse_mass: f32,
}

/// Distance constraint between two particles.
#[derive(Debug, Clone, Copy)]
pub struct DistanceConstraint {
    /// Index of first particle.
    pub particle_a: usize,

    /// Index of second particle.
    pub particle_b: usize,

    /// Rest length of the constraint.
    pub rest_length: f32,

    /// Constraint stiffness (0.0 to 1.0).
    pub stiffness: f32,

    /// Type of constraint.
    pub constraint_type: ConstraintType,
}

impl DistanceConstraint {
    /// Creates a structural constraint (edge of cloth).
    #[must_use]
    pub const fn structural(particle_a: usize, particle_b: usize, rest_length: f32) -> Self {
        Self {
            particle_a,
            particle_b,
            rest_length,
            stiffness: 1.0,
            constraint_type: ConstraintType::Structural,
        }
    }

    /// Creates a shear constraint (diagonal).
    #[must_use]
    pub const fn shear(particle_a: usize, particle_b: usize, rest_length: f32) -> Self {
        Self {
            particle_a,
            particle_b,
            rest_length,
            stiffness: 0.8,
            constraint_type: ConstraintType::Shear,
        }
    }

    /// Creates a bend constraint (skip one particle).
    #[must_use]
    pub const fn bend(particle_a: usize, particle_b: usize, rest_length: f32) -> Self {
        Self {
            particle_a,
            particle_b,
            rest_length,
            stiffness: 0.5,
            constraint_type: ConstraintType::Bend,
        }
    }
}

/// Type of cloth constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintType {
    /// Structural constraint (maintains shape).
    Structural,
    /// Shear constraint (prevents diagonal stretching).
    Shear,
    /// Bend constraint (prevents folding).
    Bend,
}

/// Wind force affecting cloth simulation.
#[derive(Component, Debug, Clone, Copy)]
pub struct ClothWind {
    /// Wind direction and strength.
    pub direction: Vec3,

    /// Wind turbulence amount (0.0 to 1.0).
    pub turbulence: f32,

    /// Time offset for turbulence variation.
    pub time_offset: f32,
}

impl ClothWind {
    /// Creates a new wind force.
    #[must_use]
    pub const fn new(direction: Vec3) -> Self {
        Self {
            direction,
            turbulence: 0.0,
            time_offset: 0.0,
        }
    }

    /// Sets the turbulence amount.
    #[must_use]
    pub const fn with_turbulence(mut self, turbulence: f32) -> Self {
        self.turbulence = turbulence;
        self
    }
}

impl Default for ClothWind {
    fn default() -> Self {
        Self::new(Vec3::ZERO)
    }
}

/// Cloth collision settings.
#[derive(Component, Debug, Clone, Copy)]
pub struct ClothCollision {
    /// Collision radius for each particle.
    pub particle_radius: f32,

    /// Collision friction coefficient.
    pub friction: f32,

    /// Collision response stiffness.
    pub response_stiffness: f32,
}

impl ClothCollision {
    /// Creates default collision settings.
    #[must_use]
    pub const fn new(particle_radius: f32) -> Self {
        Self {
            particle_radius,
            friction: 0.4,
            response_stiffness: 1.0,
        }
    }
}

impl Default for ClothCollision {
    fn default() -> Self {
        Self::new(0.02)
    }
}

/// Cloth tearing settings.
#[derive(Component, Debug, Clone, Copy)]
pub struct ClothTearing {
    /// Maximum stretch distance before tearing (multiple of rest length).
    pub tear_distance: f32,

    /// Whether tearing is enabled.
    pub enabled: bool,
}

impl ClothTearing {
    /// Creates tearing settings with specified threshold.
    #[must_use]
    pub const fn new(tear_distance: f32) -> Self {
        Self {
            tear_distance,
            enabled: true,
        }
    }

    /// Disables tearing.
    #[must_use]
    pub const fn disabled() -> Self {
        Self {
            tear_distance: 0.0,
            enabled: false,
        }
    }
}

impl Default for ClothTearing {
    fn default() -> Self {
        Self::new(2.0)
    }
}
