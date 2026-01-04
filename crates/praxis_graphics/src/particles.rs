//! GPU-accelerated particle system with instanced rendering.
//!
//! This module provides a complete particle system implementation with:
//! - Multiple emitter shapes (point, sphere, box)
//! - Per-particle properties (lifetime, velocity, color, size)
//! - Physical forces (gravity, wind, attraction points)
//! - Texture atlas support for particle sprites
//! - Efficient GPU instancing for rendering thousands of particles
//!
//! # Architecture
//!
//! The particle system uses GPU instancing to render large numbers of particles efficiently.
//! Each particle is represented as an instance with per-instance attributes including position,
//! color, size, and rotation. The system updates particles on the CPU and uploads instance
//! data to the GPU each frame.
//!
//! # Example
//!
//! ```rust,no_run
//! use praxis_graphics::particles::{ParticleSystem, ParticleEmitterConfig, EmitterShape};
//! use praxis_math::Vec3;
//!
//! # async fn example() -> praxis_utils::Result<()> {
//! // Create particle system
//! // let mut particle_system = ParticleSystem::new(allocator, command_allocator, queue)?;
//!
//! // Configure an emitter
//! let config = ParticleEmitterConfig {
//!     shape: EmitterShape::Sphere { radius: 1.0 },
//!     emission_rate: 50.0,
//!     particle_lifetime: 2.0,
//!     initial_velocity: Vec3::new(0.0, 5.0, 0.0),
//!     velocity_randomness: 2.0,
//!     initial_color: [1.0, 0.8, 0.2, 1.0],
//!     color_over_lifetime: Some(vec![
//!         [1.0, 0.8, 0.2, 1.0],
//!         [1.0, 0.3, 0.0, 0.5],
//!         [0.5, 0.0, 0.0, 0.0],
//!     ]),
//!     ..Default::default()
//! };
//!
//! // Add emitter to system
//! // particle_system.add_emitter("fire", config);
//! # Ok(())
//! # }
//! ```

use crate::vertex::Vertex3D;
use praxis_math::Vec3;
use praxis_utils::{debug, eyre, trace, Result};
use std::collections::HashMap;
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::allocator::CommandBufferAllocator,
    device::Queue,
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
    pipeline::graphics::vertex_input::Vertex,
};

/// Maximum number of particles per emitter.
pub const MAX_PARTICLES_PER_EMITTER: usize = 10000;

/// A single particle instance.
#[derive(Debug, Clone, Copy)]
struct Particle {
    /// Current position in world space.
    position: Vec3,
    /// Current velocity.
    velocity: Vec3,
    /// Current color (RGBA).
    color: [f32; 4],
    /// Current size/scale.
    size: f32,
    /// Current rotation angle in radians.
    rotation: f32,
    /// Rotation speed in radians per second.
    rotation_speed: f32,
    /// Remaining lifetime in seconds.
    lifetime: f32,
    /// Total lifetime in seconds (for interpolation).
    initial_lifetime: f32,
    /// Is this particle active?
    active: bool,
}

impl Particle {
    fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            color: [1.0, 1.0, 1.0, 1.0],
            size: 1.0,
            rotation: 0.0,
            rotation_speed: 0.0,
            lifetime: 0.0,
            initial_lifetime: 0.0,
            active: false,
        }
    }

    fn is_alive(&self) -> bool {
        self.active && self.lifetime > 0.0
    }

    fn lifetime_t(&self) -> f32 {
        if self.initial_lifetime > 0.0 {
            1.0 - (self.lifetime / self.initial_lifetime)
        } else {
            0.0
        }
    }
}

/// Per-instance data for GPU rendering.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, Vertex)]
pub struct ParticleInstance {
    /// World position of the particle.
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    /// RGBA color of the particle.
    #[format(R32G32B32A32_SFLOAT)]
    pub color: [f32; 4],
    /// Size scale of the particle.
    #[format(R32_SFLOAT)]
    pub size: f32,
    /// Rotation angle in radians.
    #[format(R32_SFLOAT)]
    pub rotation: f32,
    /// Texture atlas index (for multi-texture particles).
    #[format(R32_SFLOAT)]
    pub atlas_index: f32,
}

/// Shape of particle emitter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmitterShape {
    /// Emit from a single point.
    Point,
    /// Emit from the surface of a sphere.
    Sphere { radius: f32 },
    /// Emit from within a box volume.
    Box { extents: Vec3 },
    /// Emit from the edge of a circle.
    Circle { radius: f32 },
    /// Emit from a cone.
    Cone { radius: f32, angle: f32 },
}

impl Default for EmitterShape {
    fn default() -> Self {
        Self::Point
    }
}

/// Force applied to particles.
#[derive(Debug, Clone, Copy)]
pub enum ParticleForce {
    /// Constant gravity force.
    Gravity { strength: Vec3 },
    /// Wind force with turbulence.
    Wind {
        direction: Vec3,
        strength: f32,
        turbulence: f32,
    },
    /// Point attractor.
    Attraction {
        position: Vec3,
        strength: f32,
        radius: f32,
    },
    /// Radial force (positive = push away, negative = pull in).
    Radial { origin: Vec3, strength: f32 },
    /// Drag/air resistance.
    Drag { coefficient: f32 },
}

/// Configuration for a particle emitter.
#[derive(Debug, Clone)]
pub struct ParticleEmitterConfig {
    /// Shape of the emitter.
    pub shape: EmitterShape,
    /// Number of particles emitted per second.
    pub emission_rate: f32,
    /// Maximum number of particles this emitter can have alive at once.
    pub max_particles: usize,
    /// Particle lifetime in seconds.
    pub particle_lifetime: f32,
    /// Random variation in lifetime (±).
    pub lifetime_randomness: f32,
    /// Initial velocity of emitted particles.
    pub initial_velocity: Vec3,
    /// Random variation in velocity.
    pub velocity_randomness: f32,
    /// Initial color (RGBA).
    pub initial_color: [f32; 4],
    /// Color gradient over lifetime (if Some).
    pub color_over_lifetime: Option<Vec<[f32; 4]>>,
    /// Initial size.
    pub initial_size: f32,
    /// Size curve over lifetime (if Some).
    pub size_over_lifetime: Option<Vec<f32>>,
    /// Random variation in size.
    pub size_randomness: f32,
    /// Initial rotation angle.
    pub initial_rotation: f32,
    /// Rotation speed (radians per second).
    pub rotation_speed: f32,
    /// Random variation in rotation speed.
    pub rotation_speed_randomness: f32,
    /// Forces applied to particles.
    pub forces: Vec<ParticleForce>,
    /// Texture atlas cell (row, column) for this emitter.
    pub atlas_cell: Option<(u32, u32)>,
    /// Texture atlas grid size (rows, columns).
    pub atlas_grid: Option<(u32, u32)>,
    /// Whether emitter is looping.
    pub looping: bool,
    /// Duration of emission (if not looping).
    pub duration: f32,
}

impl Default for ParticleEmitterConfig {
    fn default() -> Self {
        Self {
            shape: EmitterShape::Point,
            emission_rate: 10.0,
            max_particles: 1000,
            particle_lifetime: 1.0,
            lifetime_randomness: 0.0,
            initial_velocity: Vec3::ZERO,
            velocity_randomness: 0.0,
            initial_color: [1.0, 1.0, 1.0, 1.0],
            color_over_lifetime: None,
            initial_size: 1.0,
            size_over_lifetime: None,
            size_randomness: 0.0,
            initial_rotation: 0.0,
            rotation_speed: 0.0,
            rotation_speed_randomness: 0.0,
            forces: vec![],
            atlas_cell: None,
            atlas_grid: None,
            looping: true,
            duration: 1.0,
        }
    }
}

/// A particle emitter that spawns and manages particles.
pub struct ParticleEmitter {
    config: ParticleEmitterConfig,
    particles: Vec<Particle>,
    position: Vec3,
    emission_accumulator: f32,
    time_alive: f32,
    is_active: bool,
}

impl ParticleEmitter {
    /// Creates a new particle emitter.
    pub fn new(config: ParticleEmitterConfig) -> Self {
        let max_particles = config.max_particles.min(MAX_PARTICLES_PER_EMITTER);
        let particles = vec![Particle::new(); max_particles];

        Self {
            config,
            particles,
            position: Vec3::ZERO,
            emission_accumulator: 0.0,
            time_alive: 0.0,
            is_active: true,
        }
    }

    /// Sets the emitter position.
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    /// Gets the emitter position.
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Activates the emitter.
    pub fn activate(&mut self) {
        self.is_active = true;
    }

    /// Deactivates the emitter.
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// Returns whether the emitter is active.
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Resets the emitter, killing all particles.
    pub fn reset(&mut self) {
        for particle in &mut self.particles {
            particle.active = false;
        }
        self.emission_accumulator = 0.0;
        self.time_alive = 0.0;
    }

    /// Updates the emitter and all its particles.
    pub fn update(&mut self, delta_time: f32) {
        if !self.is_active {
            return;
        }

        self.time_alive += delta_time;

        // Check if emitter should stop emitting (non-looping)
        let should_emit = self.config.looping || self.time_alive < self.config.duration;

        // Emit new particles
        if should_emit {
            self.emission_accumulator += self.config.emission_rate * delta_time;
            let particles_to_emit = self.emission_accumulator.floor() as usize;
            self.emission_accumulator -= particles_to_emit as f32;

            for _ in 0..particles_to_emit {
                self.emit_particle();
            }
        }

        // Update existing particles
        for particle in &mut self.particles {
            if !particle.is_alive() {
                continue;
            }

            // Update lifetime
            particle.lifetime -= delta_time;
            if particle.lifetime <= 0.0 {
                particle.active = false;
                continue;
            }

            // Apply forces
            for force in &self.config.forces {
                match force {
                    ParticleForce::Gravity { strength } => {
                        particle.velocity += *strength * delta_time;
                    }
                    ParticleForce::Wind {
                        direction,
                        strength,
                        turbulence,
                    } => {
                        let wind = *direction * *strength;
                        let turb = if *turbulence > 0.0 {
                            Vec3::new(
                                (rand::random::<f32>() - 0.5) * turbulence,
                                (rand::random::<f32>() - 0.5) * turbulence,
                                (rand::random::<f32>() - 0.5) * turbulence,
                            )
                        } else {
                            Vec3::ZERO
                        };
                        particle.velocity += (wind + turb) * delta_time;
                    }
                    ParticleForce::Attraction {
                        position,
                        strength,
                        radius,
                    } => {
                        let to_attractor = *position - particle.position;
                        let distance = to_attractor.length();
                        if distance < *radius && distance > 0.001 {
                            let force = to_attractor / distance * *strength
                                / (distance * distance).max(0.1);
                            particle.velocity += force * delta_time;
                        }
                    }
                    ParticleForce::Radial { origin, strength } => {
                        let to_particle = particle.position - *origin;
                        let distance = to_particle.length();
                        if distance > 0.001 {
                            let force = to_particle.normalize() * *strength;
                            particle.velocity += force * delta_time;
                        }
                    }
                    ParticleForce::Drag { coefficient } => {
                        particle.velocity *= 1.0 - (*coefficient * delta_time).min(1.0);
                    }
                }
            }

            // Update position
            particle.position += particle.velocity * delta_time;

            // Update rotation
            particle.rotation += particle.rotation_speed * delta_time;

            // Update color over lifetime
            if let Some(ref colors) = self.config.color_over_lifetime {
                if !colors.is_empty() {
                    let t = particle.lifetime_t();
                    let index = (t * (colors.len() - 1) as f32).floor() as usize;
                    let next_index = (index + 1).min(colors.len() - 1);
                    let local_t = (t * (colors.len() - 1) as f32) - index as f32;

                    particle.color = lerp_color(colors[index], colors[next_index], local_t);
                }
            }

            // Update size over lifetime
            if let Some(ref sizes) = self.config.size_over_lifetime {
                if !sizes.is_empty() {
                    let t = particle.lifetime_t();
                    let index = (t * (sizes.len() - 1) as f32).floor() as usize;
                    let next_index = (index + 1).min(sizes.len() - 1);
                    let local_t = (t * (sizes.len() - 1) as f32) - index as f32;

                    particle.size = lerp(sizes[index], sizes[next_index], local_t);
                }
            }
        }
    }

    /// Emits a new particle.
    fn emit_particle(&mut self) {
        // Calculate spawn position before borrowing particles
        let spawn_position = self.position + self.get_spawn_offset();

        // Find an inactive particle slot
        let particle = match self.particles.iter_mut().find(|p| !p.active) {
            Some(p) => p,
            None => return, // No free slots
        };

        // Initialize particle
        particle.active = true;
        particle.lifetime = self.config.particle_lifetime
            + (rand::random::<f32>() - 0.5) * 2.0 * self.config.lifetime_randomness;
        particle.initial_lifetime = particle.lifetime;

        // Set position based on emitter shape
        particle.position = spawn_position;

        // Set velocity
        let vel_rand = Vec3::new(
            (rand::random::<f32>() - 0.5) * self.config.velocity_randomness,
            (rand::random::<f32>() - 0.5) * self.config.velocity_randomness,
            (rand::random::<f32>() - 0.5) * self.config.velocity_randomness,
        );
        particle.velocity = self.config.initial_velocity + vel_rand;

        // Set color
        particle.color = self.config.initial_color;

        // Set size
        particle.size = self.config.initial_size
            + (rand::random::<f32>() - 0.5) * 2.0 * self.config.size_randomness;

        // Set rotation
        particle.rotation = self.config.initial_rotation;
        particle.rotation_speed = self.config.rotation_speed
            + (rand::random::<f32>() - 0.5) * 2.0 * self.config.rotation_speed_randomness;
    }

    /// Gets spawn position offset based on emitter shape.
    fn get_spawn_offset(&self) -> Vec3 {
        match self.config.shape {
            EmitterShape::Point => Vec3::ZERO,
            EmitterShape::Sphere { radius } => random_point_on_sphere() * radius,
            EmitterShape::Box { extents } => Vec3::new(
                (rand::random::<f32>() - 0.5) * extents.x,
                (rand::random::<f32>() - 0.5) * extents.y,
                (rand::random::<f32>() - 0.5) * extents.z,
            ),
            EmitterShape::Circle { radius } => {
                let angle = rand::random::<f32>() * std::f32::consts::TAU;
                Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius)
            }
            EmitterShape::Cone { radius, angle } => {
                let theta = rand::random::<f32>() * std::f32::consts::TAU;
                let phi = rand::random::<f32>() * angle;
                let r = rand::random::<f32>() * radius;

                Vec3::new(
                    phi.sin() * theta.cos() * r,
                    phi.cos() * r,
                    phi.sin() * theta.sin() * r,
                )
            }
        }
    }

    /// Gets active particle count.
    pub fn active_particle_count(&self) -> usize {
        self.particles.iter().filter(|p| p.is_alive()).count()
    }

    /// Converts active particles to instance data.
    fn to_instances(&self) -> Vec<ParticleInstance> {
        let atlas_index = if let (Some((row, col)), Some((_rows, cols))) =
            (self.config.atlas_cell, self.config.atlas_grid)
        {
            (row * cols + col) as f32
        } else {
            0.0
        };

        self.particles
            .iter()
            .filter(|p| p.is_alive())
            .map(|p| ParticleInstance {
                position: p.position.into(),
                color: p.color,
                size: p.size,
                rotation: p.rotation,
                atlas_index,
            })
            .collect()
    }
}

/// GPU particle system managing multiple emitters.
pub struct ParticleSystem {
    emitters: HashMap<String, ParticleEmitter>,
    instance_buffer: Option<Subbuffer<[ParticleInstance]>>,
    quad_vertices: Subbuffer<[Vertex3D]>,
    quad_indices: Subbuffer<[u16]>,
    memory_allocator: Arc<dyn MemoryAllocator>,
    _command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
    _queue: Arc<Queue>,
}

impl ParticleSystem {
    /// Creates a new particle system.
    pub fn new(
        memory_allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
    ) -> Result<Self> {
        debug!("Creating particle system");

        // Create quad geometry for particle billboard
        let quad_vertices = vec![
            Vertex3D::with_uv([-0.5, -0.5, 0.0], [1.0, 1.0, 1.0], [0.0, 0.0]),
            Vertex3D::with_uv([0.5, -0.5, 0.0], [1.0, 1.0, 1.0], [1.0, 0.0]),
            Vertex3D::with_uv([0.5, 0.5, 0.0], [1.0, 1.0, 1.0], [1.0, 1.0]),
            Vertex3D::with_uv([-0.5, 0.5, 0.0], [1.0, 1.0, 1.0], [0.0, 1.0]),
        ];

        let quad_indices = vec![0, 1, 2, 0, 2, 3];

        let quad_vertex_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            quad_vertices.into_iter(),
        )
        .map_err(|e| eyre::eyre!("Failed to create particle quad vertex buffer: {}", e))?;

        let quad_index_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            quad_indices.into_iter(),
        )
        .map_err(|e| eyre::eyre!("Failed to create particle quad index buffer: {}", e))?;

        Ok(Self {
            emitters: HashMap::new(),
            instance_buffer: None,
            quad_vertices: quad_vertex_buffer,
            quad_indices: quad_index_buffer,
            memory_allocator,
            _command_buffer_allocator: command_buffer_allocator,
            _queue: queue,
        })
    }

    /// Adds a new particle emitter.
    pub fn add_emitter(&mut self, id: impl Into<String>, config: ParticleEmitterConfig) {
        let id = id.into();
        debug!("Adding particle emitter '{}'", id);
        self.emitters.insert(id, ParticleEmitter::new(config));
    }

    /// Removes a particle emitter.
    pub fn remove_emitter(&mut self, id: &str) -> bool {
        self.emitters.remove(id).is_some()
    }

    /// Gets a mutable reference to an emitter.
    pub fn get_emitter_mut(&mut self, id: &str) -> Option<&mut ParticleEmitter> {
        self.emitters.get_mut(id)
    }

    /// Gets a reference to an emitter.
    pub fn get_emitter(&self, id: &str) -> Option<&ParticleEmitter> {
        self.emitters.get(id)
    }

    /// Updates all particle emitters.
    pub fn update(&mut self, delta_time: f32) {
        for emitter in self.emitters.values_mut() {
            emitter.update(delta_time);
        }
    }

    /// Prepares particle instance data for rendering.
    pub fn prepare_render(&mut self) -> Result<()> {
        // Collect all particle instances from all emitters
        let mut all_instances = Vec::new();
        for emitter in self.emitters.values() {
            all_instances.extend(emitter.to_instances());
        }

        if all_instances.is_empty() {
            self.instance_buffer = None;
            return Ok(());
        }

        trace!(
            "Preparing {} particle instances for rendering",
            all_instances.len()
        );

        // Create or update instance buffer
        let instance_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            all_instances.into_iter(),
        )
        .map_err(|e| eyre::eyre!("Failed to create particle instance buffer: {}", e))?;

        self.instance_buffer = Some(instance_buffer);
        Ok(())
    }

    /// Gets the total number of active particles across all emitters.
    pub fn total_active_particles(&self) -> usize {
        self.emitters
            .values()
            .map(|e| e.active_particle_count())
            .sum()
    }

    /// Gets the number of emitters.
    pub fn emitter_count(&self) -> usize {
        self.emitters.len()
    }

    /// Gets the instance buffer for rendering.
    pub fn instance_buffer(&self) -> Option<&Subbuffer<[ParticleInstance]>> {
        self.instance_buffer.as_ref()
    }

    /// Gets the quad vertex buffer.
    pub fn quad_vertex_buffer(&self) -> &Subbuffer<[Vertex3D]> {
        &self.quad_vertices
    }

    /// Gets the quad index buffer.
    pub fn quad_index_buffer(&self) -> &Subbuffer<[u16]> {
        &self.quad_indices
    }
}

/// Linear interpolation between two colors.
fn lerp_color(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
        lerp(a[3], b[3], t),
    ]
}

/// Linear interpolation.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Generates a random point on the surface of a unit sphere.
fn random_point_on_sphere() -> Vec3 {
    let theta = rand::random::<f32>() * std::f32::consts::TAU;
    let phi = (rand::random::<f32>() * 2.0 - 1.0).acos();

    Vec3::new(phi.sin() * theta.cos(), phi.sin() * theta.sin(), phi.cos())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_creation() {
        let particle = Particle::new();
        assert!(!particle.active);
        assert_eq!(particle.lifetime, 0.0);
    }

    #[test]
    fn test_particle_is_alive() {
        let mut particle = Particle::new();
        assert!(!particle.is_alive());

        particle.active = true;
        particle.lifetime = 1.0;
        assert!(particle.is_alive());

        particle.lifetime = 0.0;
        assert!(!particle.is_alive());
    }

    #[test]
    fn test_particle_lifetime_t() {
        let mut particle = Particle::new();
        particle.initial_lifetime = 10.0;
        particle.lifetime = 5.0;

        assert_eq!(particle.lifetime_t(), 0.5);
    }

    #[test]
    fn test_emitter_shape_default() {
        let shape = EmitterShape::default();
        assert_eq!(shape, EmitterShape::Point);
    }

    #[test]
    fn test_emitter_config_default() {
        let config = ParticleEmitterConfig::default();
        assert_eq!(config.emission_rate, 10.0);
        assert_eq!(config.particle_lifetime, 1.0);
        assert_eq!(config.max_particles, 1000);
    }

    #[test]
    fn test_emitter_creation() {
        let config = ParticleEmitterConfig::default();
        let emitter = ParticleEmitter::new(config);
        assert!(emitter.is_active());
        assert_eq!(emitter.active_particle_count(), 0);
    }

    #[test]
    fn test_emitter_position() {
        let config = ParticleEmitterConfig::default();
        let mut emitter = ParticleEmitter::new(config);

        let pos = Vec3::new(1.0, 2.0, 3.0);
        emitter.set_position(pos);
        assert_eq!(emitter.position(), pos);
    }

    #[test]
    fn test_emitter_activation() {
        let config = ParticleEmitterConfig::default();
        let mut emitter = ParticleEmitter::new(config);

        assert!(emitter.is_active());

        emitter.deactivate();
        assert!(!emitter.is_active());

        emitter.activate();
        assert!(emitter.is_active());
    }

    #[test]
    fn test_emitter_reset() {
        let config = ParticleEmitterConfig {
            emission_rate: 100.0,
            ..Default::default()
        };
        let mut emitter = ParticleEmitter::new(config);

        emitter.update(0.1);
        assert!(emitter.active_particle_count() > 0);

        emitter.reset();
        assert_eq!(emitter.active_particle_count(), 0);
    }

    #[test]
    fn test_particle_force_gravity() {
        let force = ParticleForce::Gravity {
            strength: Vec3::new(0.0, -9.8, 0.0),
        };

        match force {
            ParticleForce::Gravity { strength } => {
                assert_eq!(strength.y, -9.8);
            }
            _ => panic!("Wrong force type"),
        }
    }

    #[test]
    fn test_particle_force_wind() {
        let force = ParticleForce::Wind {
            direction: Vec3::new(1.0, 0.0, 0.0),
            strength: 5.0,
            turbulence: 0.5,
        };

        match force {
            ParticleForce::Wind { strength, .. } => {
                assert_eq!(strength, 5.0);
            }
            _ => panic!("Wrong force type"),
        }
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn test_lerp_color() {
        let a = [0.0, 0.0, 0.0, 1.0];
        let b = [1.0, 1.0, 1.0, 1.0];
        let result = lerp_color(a, b, 0.5);

        assert_eq!(result, [0.5, 0.5, 0.5, 1.0]);
    }

    #[test]
    fn test_random_point_on_sphere() {
        for _ in 0..100 {
            let point = random_point_on_sphere();
            let length = point.length();
            assert!(
                (length - 1.0).abs() < 0.001,
                "Point should be on unit sphere"
            );
        }
    }

    #[test]
    fn test_particle_instance_size() {
        // Verify ParticleInstance has expected size
        assert_eq!(std::mem::size_of::<ParticleInstance>(), 36);
    }

    #[test]
    fn test_emitter_max_particles_clamped() {
        let config = ParticleEmitterConfig {
            max_particles: MAX_PARTICLES_PER_EMITTER + 1000,
            ..Default::default()
        };
        let emitter = ParticleEmitter::new(config);
        assert_eq!(emitter.particles.len(), MAX_PARTICLES_PER_EMITTER);
    }
}
