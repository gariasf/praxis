//! GPU-accelerated particle system with instanced rendering.
//!
//! This module provides a complete particle system implementation with:
//! - Multiple emitter shapes (point, sphere, box)
//! - Per-particle properties (lifetime, velocity, color, size)
//! - Physical forces (gravity, wind, attraction points)
//! - Texture atlas support for particle sprites
//! - Efficient GPU instancing for rendering thousands of particles
//! - Particle-particle and particle-world collision detection using spatial hashing
//! - Soft particles with depth buffer comparison for smooth blending
//! - GPU-based particle sorting for correct alpha blending using bitonic sort
//! - Collision response forces for realistic bouncing particles
//!
//! # Architecture
//!
//! The particle system uses GPU instancing to render large numbers of particles efficiently.
//! Each particle is represented as an instance with per-instance attributes including position,
//! color, size, and rotation. The system updates particles on the CPU and uploads instance
//! data to the GPU each frame.
//!
//! # Collision Detection
//!
//! Spatial hashing is used for efficient particle-particle collision detection. The world space
//! is divided into a grid, and particles are hashed into cells based on their position. Collision
//! checks are then performed only within neighboring cells.
//!
//! # Soft Particles
//!
//! Particles fade out smoothly near geometry by comparing particle depth with scene depth buffer.
//! This prevents hard intersections and creates a more natural look.
//!
//! # GPU Sorting
//!
//! Particles are sorted on the GPU using bitonic sort to ensure correct alpha blending.
//! Particles are sorted by distance from camera to render back-to-front.
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
//!     enable_collisions: true,
//!     collision_radius: 0.5,
//!     restitution: 0.7,
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
    command_buffer::{
        allocator::CommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryCommandBufferAbstract,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, DescriptorSet, WriteDescriptorSet,
    },
    device::{Device, Queue},
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
    pipeline::graphics::vertex_input::Vertex,
    pipeline::{
        compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo,
        ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    sync::GpuFuture,
};

/// Maximum number of particles per emitter.
pub const MAX_PARTICLES_PER_EMITTER: usize = 10000;

/// Size of spatial hash grid cells.
const SPATIAL_HASH_CELL_SIZE: f32 = 2.0;

/// Maximum number of spatial hash buckets.
const SPATIAL_HASH_TABLE_SIZE: usize = 4096;

/// Configuration for soft particle rendering.
#[derive(Debug, Clone, Copy)]
pub struct SoftParticleConfig {
    /// Distance over which particles fade when near geometry.
    pub fade_distance: f32,
    /// Power for the fade curve (higher = sharper transition).
    pub fade_power: f32,
}

impl Default for SoftParticleConfig {
    fn default() -> Self {
        Self {
            fade_distance: 1.0,
            fade_power: 2.0,
        }
    }
}

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
    /// Collision radius for collision detection.
    collision_radius: f32,
    /// Distance from camera (for sorting).
    camera_distance: f32,
}

/// GPU particle data for compute shaders.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuParticle {
    pub position: [f32; 3],
    pub _padding1: f32,
    pub velocity: [f32; 3],
    pub _padding2: f32,
    pub color: [f32; 4],
    pub size: f32,
    pub rotation: f32,
    pub lifetime: f32,
    pub camera_distance: f32,
}

/// Spatial hash entry for collision detection.
#[derive(Debug, Clone)]
struct SpatialHashEntry {
    particle_indices: Vec<usize>,
}

/// Spatial hash grid for efficient collision detection.
struct SpatialHash {
    table: Vec<SpatialHashEntry>,
    cell_size: f32,
}

impl SpatialHash {
    fn new(cell_size: f32) -> Self {
        Self {
            table: vec![
                SpatialHashEntry {
                    particle_indices: Vec::new()
                };
                SPATIAL_HASH_TABLE_SIZE
            ],
            cell_size,
        }
    }

    #[allow(dead_code)]
    fn clear(&mut self) {
        for entry in &mut self.table {
            entry.particle_indices.clear();
        }
    }

    fn hash_position(&self, position: Vec3) -> usize {
        let x = (position.x / self.cell_size).floor() as i32;
        let y = (position.y / self.cell_size).floor() as i32;
        let z = (position.z / self.cell_size).floor() as i32;

        let hash =
            ((x.wrapping_mul(73856093)) ^ (y.wrapping_mul(19349663)) ^ (z.wrapping_mul(83492791)))
                .unsigned_abs() as usize;

        hash % SPATIAL_HASH_TABLE_SIZE
    }

    fn insert(&mut self, index: usize, position: Vec3) {
        let hash = self.hash_position(position);
        self.table[hash].particle_indices.push(index);
    }

    fn query_neighbors(&self, position: Vec3) -> Vec<usize> {
        let mut neighbors = Vec::new();

        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let offset = Vec3::new(
                        dx as f32 * self.cell_size,
                        dy as f32 * self.cell_size,
                        dz as f32 * self.cell_size,
                    );
                    let neighbor_pos = position + offset;
                    let hash = self.hash_position(neighbor_pos);
                    neighbors.extend(&self.table[hash].particle_indices);
                }
            }
        }

        neighbors
    }
}

/// World collision plane for particle-world interactions.
#[derive(Debug, Clone, Copy)]
pub struct CollisionPlane {
    pub point: Vec3,
    pub normal: Vec3,
}

impl CollisionPlane {
    pub fn new(point: Vec3, normal: Vec3) -> Self {
        Self {
            point,
            normal: normal.normalize(),
        }
    }

    fn distance_to_point(&self, point: Vec3) -> f32 {
        (point - self.point).dot(self.normal)
    }
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
            collision_radius: 0.5,
            camera_distance: 0.0,
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
    /// Enable particle-particle collision detection.
    pub enable_collisions: bool,
    /// Collision radius for particles.
    pub collision_radius: f32,
    /// Restitution coefficient for collisions (0 = no bounce, 1 = perfect bounce).
    pub restitution: f32,
    /// Friction coefficient for collisions.
    pub friction: f32,
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
            enable_collisions: false,
            collision_radius: 0.5,
            restitution: 0.5,
            friction: 0.1,
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
        self.update_with_collisions(delta_time, &[]);
    }

    /// Updates the emitter with world collision planes.
    pub fn update_with_collisions(&mut self, delta_time: f32, collision_planes: &[CollisionPlane]) {
        if !self.is_active {
            return;
        }

        self.time_alive += delta_time;

        let should_emit = self.config.looping || self.time_alive < self.config.duration;

        if should_emit {
            self.emission_accumulator += self.config.emission_rate * delta_time;
            let particles_to_emit = self.emission_accumulator.floor() as usize;
            self.emission_accumulator -= particles_to_emit as f32;

            for _ in 0..particles_to_emit {
                self.emit_particle();
            }
        }

        for particle in &mut self.particles {
            if !particle.is_alive() {
                continue;
            }

            particle.lifetime -= delta_time;
            if particle.lifetime <= 0.0 {
                particle.active = false;
                continue;
            }

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

            particle.position += particle.velocity * delta_time;
            particle.rotation += particle.rotation_speed * delta_time;

            if let Some(ref colors) = self.config.color_over_lifetime {
                if !colors.is_empty() {
                    let t = particle.lifetime_t();
                    let index = (t * (colors.len() - 1) as f32).floor() as usize;
                    let next_index = (index + 1).min(colors.len() - 1);
                    let local_t = (t * (colors.len() - 1) as f32) - index as f32;

                    particle.color = lerp_color(colors[index], colors[next_index], local_t);
                }
            }

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

        if self.config.enable_collisions {
            self.resolve_particle_collisions();
        }

        for collision_plane in collision_planes {
            self.resolve_plane_collisions(collision_plane);
        }
    }

    fn resolve_particle_collisions(&mut self) {
        let mut spatial_hash = SpatialHash::new(SPATIAL_HASH_CELL_SIZE);

        for (i, particle) in self.particles.iter().enumerate() {
            if particle.is_alive() {
                spatial_hash.insert(i, particle.position);
            }
        }

        let mut collision_pairs = Vec::new();

        for i in 0..self.particles.len() {
            if !self.particles[i].is_alive() {
                continue;
            }

            let neighbors = spatial_hash.query_neighbors(self.particles[i].position);

            for &j in &neighbors {
                if i >= j || !self.particles[j].is_alive() {
                    continue;
                }

                let delta = self.particles[j].position - self.particles[i].position;
                let distance = delta.length();
                let min_dist =
                    self.particles[i].collision_radius + self.particles[j].collision_radius;

                if distance < min_dist && distance > 0.001 {
                    collision_pairs.push((i, j, delta, distance, min_dist));
                }
            }
        }

        for (i, j, delta, distance, min_dist) in collision_pairs {
            let normal = delta / distance;
            let overlap = min_dist - distance;

            let pos_correction = normal * overlap * 0.5;
            self.particles[i].position -= pos_correction;
            self.particles[j].position += pos_correction;

            let relative_velocity = self.particles[j].velocity - self.particles[i].velocity;
            let velocity_along_normal = relative_velocity.dot(normal);

            if velocity_along_normal < 0.0 {
                let restitution = self.config.restitution;
                let impulse = -(1.0 + restitution) * velocity_along_normal;
                let impulse_vector = normal * impulse * 0.5;

                self.particles[i].velocity -= impulse_vector;
                self.particles[j].velocity += impulse_vector;

                let tangent = relative_velocity - normal * velocity_along_normal;
                let tangent_length = tangent.length();
                if tangent_length > 0.001 {
                    let friction_impulse = tangent / tangent_length * self.config.friction;
                    self.particles[i].velocity += friction_impulse * 0.5;
                    self.particles[j].velocity -= friction_impulse * 0.5;
                }
            }
        }
    }

    fn resolve_plane_collisions(&mut self, plane: &CollisionPlane) {
        for particle in &mut self.particles {
            if !particle.is_alive() {
                continue;
            }

            let distance = plane.distance_to_point(particle.position);

            if distance < particle.collision_radius {
                let penetration = particle.collision_radius - distance;
                particle.position += plane.normal * penetration;

                let velocity_along_normal = particle.velocity.dot(plane.normal);
                if velocity_along_normal < 0.0 {
                    let restitution = self.config.restitution;
                    particle.velocity -= plane.normal * velocity_along_normal * (1.0 + restitution);

                    let tangent_velocity =
                        particle.velocity - plane.normal * particle.velocity.dot(plane.normal);
                    let friction = self.config.friction;
                    particle.velocity -= tangent_velocity * friction;
                }
            }
        }
    }

    /// Emits a new particle.
    fn emit_particle(&mut self) {
        let spawn_position = self.position + self.get_spawn_offset();

        let particle = match self.particles.iter_mut().find(|p| !p.active) {
            Some(p) => p,
            None => return,
        };

        particle.active = true;
        particle.lifetime = self.config.particle_lifetime
            + (rand::random::<f32>() - 0.5) * 2.0 * self.config.lifetime_randomness;
        particle.initial_lifetime = particle.lifetime;
        particle.position = spawn_position;

        let vel_rand = Vec3::new(
            (rand::random::<f32>() - 0.5) * self.config.velocity_randomness,
            (rand::random::<f32>() - 0.5) * self.config.velocity_randomness,
            (rand::random::<f32>() - 0.5) * self.config.velocity_randomness,
        );
        particle.velocity = self.config.initial_velocity + vel_rand;
        particle.color = self.config.initial_color;
        particle.size = self.config.initial_size
            + (rand::random::<f32>() - 0.5) * 2.0 * self.config.size_randomness;
        particle.rotation = self.config.initial_rotation;
        particle.rotation_speed = self.config.rotation_speed
            + (rand::random::<f32>() - 0.5) * 2.0 * self.config.rotation_speed_randomness;
        particle.collision_radius = self.config.collision_radius;
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
    command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
    queue: Arc<Queue>,
    #[allow(dead_code)]
    device: Arc<Device>,
    sort_pipeline: Option<Arc<ComputePipeline>>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    #[allow(dead_code)]
    gpu_particle_buffer: Option<Subbuffer<[GpuParticle]>>,
    enable_gpu_sorting: bool,
    camera_position: Vec3,
    collision_planes: Vec<CollisionPlane>,
    soft_particle_config: SoftParticleConfig,
}

impl ParticleSystem {
    /// Creates a new particle system.
    pub fn new(
        memory_allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
    ) -> Result<Self> {
        debug!("Creating particle system");

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

        let device = queue.device().clone();
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let sort_pipeline = Self::create_sort_pipeline(&device)?;

        Ok(Self {
            emitters: HashMap::new(),
            instance_buffer: None,
            quad_vertices: quad_vertex_buffer,
            quad_indices: quad_index_buffer,
            memory_allocator,
            command_buffer_allocator,
            queue,
            device,
            sort_pipeline: Some(sort_pipeline),
            descriptor_set_allocator,
            gpu_particle_buffer: None,
            enable_gpu_sorting: true,
            camera_position: Vec3::ZERO,
            collision_planes: Vec::new(),
            soft_particle_config: SoftParticleConfig::default(),
        })
    }

    fn create_sort_pipeline(device: &Arc<Device>) -> Result<Arc<ComputePipeline>> {
        mod cs {
            vulkano_shaders::shader! {
                ty: "compute",
                path: "src/shaders/particle_sort.comp"
            }
        }

        let cs_module = cs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load particle sort shader: {}", e))?;

        let cs_entry_point = cs_module
            .entry_point("main")
            .ok_or_else(|| eyre::eyre!("Particle sort shader missing main entry point"))?;

        let stage = PipelineShaderStageCreateInfo::new(cs_entry_point);
        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&[stage.clone()])
                .into_pipeline_layout_create_info(device.clone())
                .map_err(|e| eyre::eyre!("Failed to create pipeline layout: {}", e))?,
        )
        .map_err(|e| eyre::eyre!("Failed to create pipeline layout: {}", e))?;

        ComputePipeline::new(
            device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(stage, layout),
        )
        .map_err(|e| eyre::eyre!("Failed to create compute pipeline: {}", e))
    }

    pub fn set_camera_position(&mut self, position: Vec3) {
        self.camera_position = position;
    }

    pub fn add_collision_plane(&mut self, plane: CollisionPlane) {
        self.collision_planes.push(plane);
    }

    pub fn clear_collision_planes(&mut self) {
        self.collision_planes.clear();
    }

    pub fn set_gpu_sorting_enabled(&mut self, enabled: bool) {
        self.enable_gpu_sorting = enabled;
    }

    pub fn set_soft_particle_config(&mut self, config: SoftParticleConfig) {
        self.soft_particle_config = config;
    }

    pub fn soft_particle_config(&self) -> &SoftParticleConfig {
        &self.soft_particle_config
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
            emitter.update_with_collisions(delta_time, &self.collision_planes);
        }

        for emitter in self.emitters.values_mut() {
            for particle in &mut emitter.particles {
                if particle.is_alive() {
                    let to_camera = self.camera_position - particle.position;
                    particle.camera_distance = to_camera.length();
                }
            }
        }
    }

    /// Prepares particle instance data for rendering.
    pub fn prepare_render(&mut self) -> Result<()> {
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

        if self.enable_gpu_sorting && all_instances.len() > 1 {
            self.sort_particles_gpu(&mut all_instances)?;
        } else {
            all_instances.sort_by(|a, b| {
                let dist_a = (Vec3::from(a.position) - self.camera_position).length_squared();
                let dist_b = (Vec3::from(b.position) - self.camera_position).length_squared();
                dist_b
                    .partial_cmp(&dist_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

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

    fn sort_particles_gpu(&mut self, instances: &mut [ParticleInstance]) -> Result<()> {
        let count = instances.len();
        let padded_count = count.next_power_of_two();

        let mut gpu_data: Vec<GpuParticle> = instances
            .iter()
            .map(|inst| {
                let dist = (Vec3::from(inst.position) - self.camera_position).length();
                GpuParticle {
                    position: inst.position,
                    _padding1: 0.0,
                    velocity: [0.0; 3],
                    _padding2: 0.0,
                    color: inst.color,
                    size: inst.size,
                    rotation: inst.rotation,
                    lifetime: 0.0,
                    camera_distance: dist,
                }
            })
            .collect();

        gpu_data.resize(padded_count, GpuParticle::default());

        let gpu_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            gpu_data.into_iter(),
        )
        .map_err(|e| eyre::eyre!("Failed to create GPU particle buffer: {}", e))?;

        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            self.sort_pipeline.as_ref().unwrap().layout().set_layouts()[0].clone(),
            [WriteDescriptorSet::buffer(0, gpu_buffer.clone())],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create descriptor set: {}", e))?;

        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| eyre::eyre!("Failed to create command buffer: {}", e))?;

        builder
            .bind_pipeline_compute(self.sort_pipeline.as_ref().unwrap().clone())
            .map_err(|e| eyre::eyre!("Failed to bind pipeline: {}", e))?
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.sort_pipeline.as_ref().unwrap().layout().clone(),
                0,
                descriptor_set,
            )
            .map_err(|e| eyre::eyre!("Failed to bind descriptor sets: {}", e))?;

        let num_stages = (padded_count as f32).log2() as u32;
        for stage in 0..num_stages {
            for substage in (0..=stage).rev() {
                #[repr(C)]
                #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
                struct PushConstants {
                    stage: u32,
                    substage: u32,
                    num_particles: u32,
                }

                let push_constants = PushConstants {
                    stage,
                    substage,
                    num_particles: padded_count as u32,
                };

                unsafe {
                    builder
                        .push_constants(
                            self.sort_pipeline.as_ref().unwrap().layout().clone(),
                            0,
                            push_constants,
                        )
                        .map_err(|e| eyre::eyre!("Failed to push constants: {}", e))?
                        .dispatch([(padded_count / 256) as u32 + 1, 1, 1])
                        .map_err(|e| eyre::eyre!("Failed to dispatch: {}", e))?;
                }
            }
        }

        let command_buffer = builder
            .build()
            .map_err(|e| eyre::eyre!("Failed to build command buffer: {}", e))?;

        command_buffer
            .execute(self.queue.clone())
            .map_err(|e| eyre::eyre!("Failed to execute command buffer: {}", e))?
            .then_signal_fence_and_flush()
            .map_err(|e| eyre::eyre!("Failed to flush: {}", e))?
            .wait(None)
            .map_err(|e| eyre::eyre!("Failed to wait: {}", e))?;

        let sorted_data = gpu_buffer
            .read()
            .map_err(|e| eyre::eyre!("Failed to read GPU buffer: {}", e))?;

        for (i, gpu_particle) in sorted_data.iter().take(count).enumerate() {
            instances[i].position = gpu_particle.position;
            instances[i].color = gpu_particle.color;
            instances[i].size = gpu_particle.size;
            instances[i].rotation = gpu_particle.rotation;
        }

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
        assert_eq!(std::mem::size_of::<ParticleInstance>(), 40);
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
