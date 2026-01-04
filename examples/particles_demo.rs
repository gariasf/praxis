//! Particle System Demo
//!
//! This example demonstrates the particle system with various effects:
//! - Fire particles with upward velocity and color gradient
//! - Smoke particles with slow upward drift
//! - Explosion particles with radial velocity
//! - Multiple emitter shapes and forces
//! - Particle-particle collisions using spatial hashing
//! - Particle-world collisions with ground plane
//! - GPU-based particle sorting for correct alpha blending
//! - Soft particles that fade near geometry

use praxis_ecs::{ParticleEmitter, Transform, World};
use praxis_graphics::{
    CollisionPlane, EmitterShape, ParticleEmitterConfig, ParticleForce, ParticleSystem,
    RenderContext, SoftParticleConfig,
};
use praxis_math::Vec3;
use praxis_utils::Result;
use praxis_window::WindowManager;
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<()> {
    praxis_utils::init_logging();

    let event_loop = EventLoop::new()?;
    let mut window_manager = WindowManager::new(&event_loop)?;
    let window = Arc::new(window_manager.create_window("Particle System Demo", 1280, 720)?);

    let mut render_context = pollster::block_on(RenderContext::new(window.clone()))?;
    let mut particle_system = ParticleSystem::new(
        render_context.memory_allocator().clone(),
        render_context.command_buffer_allocator().clone(),
        render_context.graphics_queue.clone(),
    )?;

    particle_system.set_camera_position(Vec3::new(0.0, 5.0, 10.0));
    particle_system.set_gpu_sorting_enabled(true);
    particle_system.set_soft_particle_config(SoftParticleConfig {
        fade_distance: 0.5,
        fade_power: 2.0,
    });

    let ground_plane = CollisionPlane::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
    particle_system.add_collision_plane(ground_plane);

    let fire_config = ParticleEmitterConfig {
        shape: EmitterShape::Sphere { radius: 0.5 },
        emission_rate: 50.0,
        max_particles: 500,
        particle_lifetime: 2.0,
        lifetime_randomness: 0.3,
        initial_velocity: Vec3::new(0.0, 3.0, 0.0),
        velocity_randomness: 1.0,
        initial_color: [1.0, 0.8, 0.2, 1.0],
        color_over_lifetime: Some(vec![
            [1.0, 0.8, 0.2, 1.0],
            [1.0, 0.3, 0.0, 0.8],
            [0.5, 0.0, 0.0, 0.3],
            [0.1, 0.0, 0.0, 0.0],
        ]),
        initial_size: 0.3,
        size_over_lifetime: Some(vec![0.1, 0.5, 0.8, 0.4]),
        size_randomness: 0.1,
        rotation_speed: 2.0,
        rotation_speed_randomness: 1.0,
        forces: vec![
            ParticleForce::Gravity {
                strength: Vec3::new(0.0, 1.0, 0.0),
            },
            ParticleForce::Wind {
                direction: Vec3::new(1.0, 0.0, 0.0),
                strength: 0.5,
                turbulence: 0.3,
            },
            ParticleForce::Drag { coefficient: 0.5 },
        ],
        looping: true,
        enable_collisions: false,
        ..Default::default()
    };
    particle_system.add_emitter("fire", fire_config);

    // Create smoke emitter
    let smoke_config = ParticleEmitterConfig {
        shape: EmitterShape::Point,
        emission_rate: 20.0,
        max_particles: 300,
        particle_lifetime: 4.0,
        lifetime_randomness: 0.5,
        initial_velocity: Vec3::new(0.0, 1.0, 0.0),
        velocity_randomness: 0.5,
        initial_color: [0.5, 0.5, 0.5, 0.5],
        color_over_lifetime: Some(vec![
            [0.5, 0.5, 0.5, 0.5], // Gray
            [0.4, 0.4, 0.4, 0.3], // Lighter gray
            [0.3, 0.3, 0.3, 0.1], // Very light gray
            [0.2, 0.2, 0.2, 0.0], // Fade out
        ]),
        initial_size: 0.5,
        size_over_lifetime: Some(vec![0.3, 0.8, 1.2, 1.5]),
        size_randomness: 0.2,
        rotation_speed: 0.5,
        rotation_speed_randomness: 0.5,
        forces: vec![
            ParticleForce::Wind {
                direction: Vec3::new(1.0, 0.5, 0.0),
                strength: 1.0,
                turbulence: 0.8,
            },
            ParticleForce::Drag { coefficient: 0.3 },
        ],
        looping: true,
        ..Default::default()
    };
    particle_system.add_emitter("smoke", smoke_config);

    let explosion_config = ParticleEmitterConfig {
        shape: EmitterShape::Sphere { radius: 0.2 },
        emission_rate: 200.0,
        max_particles: 1000,
        particle_lifetime: 1.5,
        lifetime_randomness: 0.2,
        initial_velocity: Vec3::ZERO,
        velocity_randomness: 5.0,
        initial_color: [1.0, 1.0, 0.5, 1.0],
        color_over_lifetime: Some(vec![
            [1.0, 1.0, 0.5, 1.0],
            [1.0, 0.5, 0.0, 0.8],
            [1.0, 0.0, 0.0, 0.5],
            [0.2, 0.0, 0.0, 0.0],
        ]),
        initial_size: 0.2,
        size_over_lifetime: Some(vec![0.2, 0.5, 0.3, 0.1]),
        size_randomness: 0.1,
        forces: vec![
            ParticleForce::Radial {
                origin: Vec3::ZERO,
                strength: 10.0,
            },
            ParticleForce::Gravity {
                strength: Vec3::new(0.0, -9.8, 0.0),
            },
            ParticleForce::Drag { coefficient: 2.0 },
        ],
        looping: false,
        duration: 0.2,
        enable_collisions: true,
        collision_radius: 0.3,
        restitution: 0.7,
        friction: 0.2,
        ..Default::default()
    };
    particle_system.add_emitter("explosion", explosion_config);

    // Position emitters in world space
    if let Some(fire_emitter) = particle_system.get_emitter_mut("fire") {
        fire_emitter.set_position(Vec3::new(-3.0, 0.0, 0.0));
    }
    if let Some(smoke_emitter) = particle_system.get_emitter_mut("smoke") {
        smoke_emitter.set_position(Vec3::new(-3.0, 2.0, 0.0));
    }
    if let Some(explosion_emitter) = particle_system.get_emitter_mut("explosion") {
        explosion_emitter.set_position(Vec3::new(3.0, 1.0, 0.0));
    }

    // Create ECS world for particle emitter components
    let mut world = World::new();
    world.spawn((
        Transform::from_xyz(-3.0, 0.0, 0.0),
        ParticleEmitter::new("fire"),
    ));
    world.spawn((
        Transform::from_xyz(-3.0, 2.0, 0.0),
        ParticleEmitter::new("smoke"),
    ));
    world.spawn((
        Transform::from_xyz(3.0, 1.0, 0.0),
        ParticleEmitter::new("explosion"),
    ));

    let mut last_frame_time = std::time::Instant::now();
    let mut explosion_cooldown = 0.0f32;

    event_loop.run(move |event, window_target| {
        window_target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("Close button pressed, exiting...");
                window_target.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                render_context.configure_surface(size.width, size.height);
            }
            Event::AboutToWait => {
                // Calculate delta time
                let current_time = std::time::Instant::now();
                let delta_time = (current_time - last_frame_time).as_secs_f32();
                last_frame_time = current_time;

                // Update particle system
                particle_system.update(delta_time);

                // Trigger explosion periodically
                explosion_cooldown -= delta_time;
                if explosion_cooldown <= 0.0 {
                    if let Some(explosion_emitter) = particle_system.get_emitter_mut("explosion") {
                        explosion_emitter.reset();
                        explosion_emitter.activate();
                    }
                    explosion_cooldown = 3.0; // Explode every 3 seconds
                }

                // Prepare particle rendering
                if let Err(e) = particle_system.prepare_render() {
                    eprintln!("Failed to prepare particle rendering: {}", e);
                }

                println!(
                    "Active particles: {} across {} emitters | Soft Particles: enabled",
                    particle_system.total_active_particles(),
                    particle_system.emitter_count()
                );

                // Request redraw
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // In a real application, you would integrate this with the actual rendering pipeline
                // For this demo, we just print statistics
                println!(
                    "Frame rendered with {} particles",
                    particle_system.total_active_particles()
                );
            }
            _ => {}
        }
    })?;

    Ok(())
}
