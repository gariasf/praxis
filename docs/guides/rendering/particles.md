# Particle Effects Guide

This guide provides practical examples for creating common particle effects in Praxis. For comprehensive documentation on the particle system architecture and advanced features, see [crates/praxis_graphics/PARTICLES.md](../../crates/praxis_graphics/PARTICLES.md).

## Basic Setup

```rust
use praxis_graphics::{ParticleSystem, ParticleEmitterConfig, EmitterShape, ParticleForce};
use praxis_math::Vec3;

// Create particle system
let mut particle_system = ParticleSystem::new(
    memory_allocator,
    command_buffer_allocator,
    queue,
)?;

// Add emitter with configuration
let config = ParticleEmitterConfig {
    shape: EmitterShape::Point,
    emission_rate: 50.0,
    particle_lifetime: 2.0,
    initial_velocity: Vec3::new(0.0, 3.0, 0.0),
    ..Default::default()
};

particle_system.add_emitter("effect_name", config);

// Update each frame
particle_system.update(delta_time);
particle_system.prepare_render()?;
```

## Common Effects

### Fire

```rust
let fire_config = ParticleEmitterConfig {
    shape: EmitterShape::Sphere { radius: 0.5 },
    emission_rate: 50.0,
    max_particles: 500,
    particle_lifetime: 2.0,
    lifetime_randomness: 0.3,
    
    initial_velocity: Vec3::new(0.0, 3.0, 0.0),
    velocity_randomness: 1.0,
    
    initial_color: [1.0, 0.8, 0.2, 1.0],  // Bright yellow
    color_over_lifetime: Some(vec![
        [1.0, 0.8, 0.2, 1.0],  // Yellow
        [1.0, 0.3, 0.0, 0.8],  // Orange
        [0.5, 0.0, 0.0, 0.3],  // Dark red
        [0.1, 0.0, 0.0, 0.0],  // Fade out
    ]),
    
    initial_size: 0.3,
    size_over_lifetime: Some(vec![0.1, 0.5, 0.8, 0.4]),
    
    forces: vec![
        ParticleForce::Gravity {
            strength: Vec3::new(0.0, 1.0, 0.0),  // Upward (hot air)
        },
        ParticleForce::Drag { coefficient: 0.5 },
    ],
    
    looping: true,
    ..Default::default()
};
```

### Smoke

```rust
let smoke_config = ParticleEmitterConfig {
    shape: EmitterShape::Point,
    emission_rate: 20.0,
    particle_lifetime: 4.0,
    
    initial_velocity: Vec3::new(0.0, 1.0, 0.0),
    velocity_randomness: 0.5,
    
    initial_color: [0.5, 0.5, 0.5, 0.5],  // Gray
    color_over_lifetime: Some(vec![
        [0.5, 0.5, 0.5, 0.5],
        [0.3, 0.3, 0.3, 0.1],
        [0.2, 0.2, 0.2, 0.0],  // Fade to transparent
    ]),
    
    initial_size: 0.3,
    size_over_lifetime: Some(vec![0.3, 1.0, 1.5]),  // Grows over time
    
    forces: vec![
        ParticleForce::Wind {
            direction: Vec3::new(1.0, 0.5, 0.0),
            strength: 1.0,
            turbulence: 0.8,  // Random wobble
        },
    ],
    
    looping: true,
    ..Default::default()
};
```

### Explosion

```rust
let explosion_config = ParticleEmitterConfig {
    shape: EmitterShape::Sphere { radius: 0.2 },
    emission_rate: 200.0,
    max_particles: 300,
    particle_lifetime: 1.5,
    
    initial_velocity: Vec3::ZERO,
    velocity_randomness: 5.0,  // High randomness = radial burst
    
    initial_color: [1.0, 1.0, 0.5, 1.0],  // Bright yellow
    color_over_lifetime: Some(vec![
        [1.0, 1.0, 0.5, 1.0],
        [1.0, 0.3, 0.0, 0.7],  // Orange
        [0.2, 0.0, 0.0, 0.0],  // Fade out
    ]),
    
    initial_size: 0.3,
    size_over_lifetime: Some(vec![0.3, 0.5, 0.2]),
    
    forces: vec![
        ParticleForce::Radial {
            origin: Vec3::ZERO,
            strength: 10.0,  // Push outward
        },
        ParticleForce::Gravity {
            strength: Vec3::new(0.0, -9.8, 0.0),
        },
    ],
    
    looping: false,     // One-shot effect
    duration: 0.2,      // Emit for 0.2 seconds
    ..Default::default()
};
```

### Sparks

```rust
let sparks_config = ParticleEmitterConfig {
    shape: EmitterShape::Point,
    emission_rate: 100.0,
    particle_lifetime: 0.5,
    
    initial_velocity: Vec3::ZERO,
    velocity_randomness: 3.0,
    
    initial_color: [1.0, 1.0, 0.5, 1.0],  // Bright yellow-white
    color_over_lifetime: Some(vec![
        [1.0, 1.0, 0.5, 1.0],
        [1.0, 0.5, 0.0, 0.5],
        [0.5, 0.0, 0.0, 0.0],
    ]),
    
    initial_size: 0.05,
    size_over_lifetime: Some(vec![0.05, 0.03, 0.01]),
    
    forces: vec![
        ParticleForce::Gravity {
            strength: Vec3::new(0.0, -9.8, 0.0),
        },
        ParticleForce::Drag { coefficient: 2.0 },  // High drag
    ],
    
    looping: false,
    duration: 0.1,
    ..Default::default()
};
```

### Magic/Energy Effect

```rust
let magic_config = ParticleEmitterConfig {
    shape: EmitterShape::Sphere { radius: 1.0 },
    emission_rate: 30.0,
    particle_lifetime: 1.0,
    
    initial_velocity: Vec3::ZERO,
    velocity_randomness: 0.5,
    
    initial_color: [0.5, 0.5, 1.0, 1.0],  // Blue
    color_over_lifetime: Some(vec![
        [0.5, 0.5, 1.0, 1.0],
        [0.3, 0.8, 1.0, 0.8],
        [0.2, 0.5, 1.0, 0.0],
    ]),
    
    initial_size: 0.1,
    size_over_lifetime: Some(vec![0.1, 0.3, 0.1]),
    
    rotation_speed: 5.0,
    
    forces: vec![
        ParticleForce::Attraction {
            position: Vec3::ZERO,
            strength: 2.0,
            radius: 5.0,
        },
    ],
    
    looping: true,
    ..Default::default()
};
```

### Rain

```rust
let rain_config = ParticleEmitterConfig {
    shape: EmitterShape::Box {
        half_extents: Vec3::new(10.0, 0.1, 10.0),
    },
    emission_rate: 500.0,
    max_particles: 2000,
    particle_lifetime: 2.0,
    
    initial_velocity: Vec3::new(0.0, -10.0, 0.0),
    velocity_randomness: 1.0,
    
    initial_color: [0.7, 0.7, 1.0, 0.3],  // Light blue, semi-transparent
    
    initial_size: 0.02,
    size_randomness: 0.01,
    
    forces: vec![
        ParticleForce::Gravity {
            strength: Vec3::new(0.0, -9.8, 0.0),
        },
    ],
    
    looping: true,
    ..Default::default()
};
```

## ECS Integration

Attach particle emitters to entities for dynamic positioning:

```rust
use praxis_ecs::{World, Transform};
use praxis_graphics::ParticleEmitter;

let mut world = World::new();

// Torch with fire effect
world.spawn((
    Transform::from_xyz(5.0, 1.0, 0.0),
    ParticleEmitter::new("torch_fire"),
));

// Moving projectile with trail
world.spawn((
    Transform::from_xyz(0.0, 2.0, 0.0),
    ParticleEmitter::new("magic_trail"),
    Velocity(Vec3::new(5.0, 0.0, 0.0)),
));
```

## Performance Tips

1. **Emission Rate × Lifetime = Active Particle Count**
   - 50/sec × 2s = ~100 particles
   - Plan your particle budgets accordingly

2. **Use looping wisely**
   - Continuous: `looping: true` (fire, smoke, rain)
   - One-shot: `looping: false` (explosions, impacts)

3. **Cull distant effects**
   - Don't update particles far from camera
   - Disable emitters when not visible

4. **Target particle counts**
   - 100 particles: negligible performance impact
   - 1,000 particles: good for main effects
   - 5,000 particles: acceptable for heavy effects
   - 10,000+ particles: use sparingly

## Advanced Features

The particle system also supports:

- **Collision Detection**: Particle-particle and particle-plane collisions with spatial hashing
- **GPU Sorting**: Correct alpha blending via bitonic sort
- **Soft Particles**: Smooth blending with scene geometry using depth buffer

See [crates/praxis_graphics/PARTICLES.md](../../crates/praxis_graphics/PARTICLES.md) for complete documentation on these features.

## Example Demo

Run the particles demo to see these effects in action:

```bash
cargo run --example particles_demo
```
