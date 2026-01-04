# Particle System

The Praxis particle system provides GPU-accelerated particle rendering with support for multiple emitter shapes, physical forces, color/size gradients, and texture atlases.

## Overview

The particle system is designed for efficient rendering of large numbers of particles using GPU instancing. Each particle emitter manages its own particle pool and updates particles on the CPU, then uploads instance data to the GPU for rendering.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Particle System                      │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   Emitter 1  │  │   Emitter 2  │  │   Emitter 3  │ │
│  │              │  │              │  │              │ │
│  │  Particles[] │  │  Particles[] │  │  Particles[] │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│         │                 │                 │          │
│         └─────────────────┴─────────────────┘          │
│                        │                               │
│                   Instance Buffer                      │
│                        │                               │
└────────────────────────┼───────────────────────────────┘
                         │
                         ▼
                   GPU Rendering
                   (Instanced Draw)
```

## Core Components

### ParticleSystem

The main particle system manages multiple emitters and handles GPU resource allocation:

```rust
use praxis_graphics::ParticleSystem;
use std::sync::Arc;

let mut particle_system = ParticleSystem::new(
    memory_allocator,
    command_buffer_allocator,
    queue,
)?;
```

### ParticleEmitter

Each emitter spawns and manages particles according to its configuration:

```rust
use praxis_graphics::{ParticleEmitterConfig, EmitterShape};
use praxis_math::Vec3;

let config = ParticleEmitterConfig {
    shape: EmitterShape::Sphere { radius: 1.0 },
    emission_rate: 50.0,
    max_particles: 1000,
    particle_lifetime: 2.0,
    initial_velocity: Vec3::new(0.0, 5.0, 0.0),
    ..Default::default()
};

particle_system.add_emitter("fire", config);
```

## Emitter Shapes

Particles can be spawned from various shapes:

### Point

Spawn all particles from a single point:

```rust
EmitterShape::Point
```

### Sphere

Spawn particles from the surface of a sphere:

```rust
EmitterShape::Sphere { radius: 2.0 }
```

### Box

Spawn particles from within a box volume:

```rust
EmitterShape::Box {
    extents: Vec3::new(2.0, 1.0, 2.0)
}
```

### Circle

Spawn particles from the edge of a circle:

```rust
EmitterShape::Circle { radius: 1.5 }
```

### Cone

Spawn particles from within a cone:

```rust
EmitterShape::Cone {
    radius: 1.0,
    angle: std::f32::consts::PI / 4.0, // 45 degrees
}
```

## Particle Properties

### Lifetime

Control how long particles live:

```rust
let config = ParticleEmitterConfig {
    particle_lifetime: 3.0,        // Base lifetime in seconds
    lifetime_randomness: 0.5,      // Random variation ± 0.5 seconds
    ..Default::default()
};
```

### Velocity

Control particle movement:

```rust
let config = ParticleEmitterConfig {
    initial_velocity: Vec3::new(0.0, 10.0, 0.0),  // Upward
    velocity_randomness: 2.0,                      // ± 2.0 units
    ..Default::default()
};
```

### Color Over Lifetime

Define color gradients:

```rust
let config = ParticleEmitterConfig {
    initial_color: [1.0, 1.0, 1.0, 1.0],
    color_over_lifetime: Some(vec![
        [1.0, 1.0, 1.0, 1.0],  // White (start)
        [1.0, 0.5, 0.0, 0.8],  // Orange
        [1.0, 0.0, 0.0, 0.5],  // Red
        [0.5, 0.0, 0.0, 0.0],  // Dark red, transparent (end)
    ]),
    ..Default::default()
};
```

### Size Over Lifetime

Define size curves:

```rust
let config = ParticleEmitterConfig {
    initial_size: 0.5,
    size_over_lifetime: Some(vec![
        0.1,  // Small (start)
        0.5,  // Medium
        1.0,  // Large
        0.3,  // Shrink (end)
    ]),
    size_randomness: 0.1,  // ± 0.1 units
    ..Default::default()
};
```

### Rotation

Control particle rotation:

```rust
let config = ParticleEmitterConfig {
    initial_rotation: 0.0,                         // Starting angle (radians)
    rotation_speed: 2.0,                           // Radians per second
    rotation_speed_randomness: 1.0,                // ± 1.0 rad/s
    ..Default::default()
};
```

## Physical Forces

Apply forces to particles for realistic motion:

### Gravity

Constant downward (or any direction) force:

```rust
use praxis_graphics::ParticleForce;

ParticleForce::Gravity {
    strength: Vec3::new(0.0, -9.8, 0.0)
}
```

### Wind

Directional force with turbulence:

```rust
ParticleForce::Wind {
    direction: Vec3::new(1.0, 0.0, 0.0),  // Blowing to the right
    strength: 5.0,
    turbulence: 0.5,                       // Adds random variation
}
```

### Attraction

Pull particles toward a point:

```rust
ParticleForce::Attraction {
    position: Vec3::new(0.0, 5.0, 0.0),  // Attraction point
    strength: 10.0,
    radius: 20.0,                         // Maximum attraction distance
}
```

### Radial

Push particles away from (or toward) a point:

```rust
ParticleForce::Radial {
    origin: Vec3::new(0.0, 0.0, 0.0),
    strength: 5.0,  // Positive = push away, negative = pull in
}
```

### Drag

Air resistance that slows particles:

```rust
ParticleForce::Drag {
    coefficient: 0.5  // Higher = more resistance
}
```

## Texture Atlases

Use sprite sheets for animated particles:

```rust
let config = ParticleEmitterConfig {
    atlas_cell: Some((0, 0)),      // (row, column) in atlas
    atlas_grid: Some((4, 4)),      // 4x4 grid of sprites
    ..Default::default()
};
```

The atlas index is automatically calculated and passed to the shader:
```
index = row * columns + column
```

## Common Effects

### Fire

```rust
let fire_config = ParticleEmitterConfig {
    shape: EmitterShape::Sphere { radius: 0.5 },
    emission_rate: 50.0,
    particle_lifetime: 2.0,
    initial_velocity: Vec3::new(0.0, 3.0, 0.0),
    velocity_randomness: 1.0,
    initial_color: [1.0, 0.8, 0.2, 1.0],
    color_over_lifetime: Some(vec![
        [1.0, 0.8, 0.2, 1.0],  // Yellow
        [1.0, 0.3, 0.0, 0.8],  // Orange
        [0.5, 0.0, 0.0, 0.3],  // Red
        [0.1, 0.0, 0.0, 0.0],  // Black
    ]),
    size_over_lifetime: Some(vec![0.1, 0.5, 0.8, 0.4]),
    forces: vec![
        ParticleForce::Gravity { strength: Vec3::new(0.0, 1.0, 0.0) },
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
    initial_color: [0.5, 0.5, 0.5, 0.5],
    color_over_lifetime: Some(vec![
        [0.5, 0.5, 0.5, 0.5],
        [0.4, 0.4, 0.4, 0.3],
        [0.3, 0.3, 0.3, 0.1],
        [0.2, 0.2, 0.2, 0.0],
    ]),
    size_over_lifetime: Some(vec![0.3, 0.8, 1.2, 1.5]),
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
```

### Explosion

```rust
let explosion_config = ParticleEmitterConfig {
    shape: EmitterShape::Sphere { radius: 0.2 },
    emission_rate: 200.0,
    particle_lifetime: 1.5,
    initial_velocity: Vec3::ZERO,
    velocity_randomness: 5.0,
    initial_color: [1.0, 1.0, 0.5, 1.0],
    color_over_lifetime: Some(vec![
        [1.0, 1.0, 0.5, 1.0],
        [1.0, 0.5, 0.0, 0.8],
        [1.0, 0.0, 0.0, 0.5],
        [0.2, 0.0, 0.0, 0.0],
    ]),
    forces: vec![
        ParticleForce::Radial {
            origin: Vec3::ZERO,
            strength: 10.0,
        },
        ParticleForce::Gravity {
            strength: Vec3::new(0.0, -9.8, 0.0),
        },
    ],
    looping: false,
    duration: 0.2,
    ..Default::default()
};
```

## ECS Integration

Particle emitters can be attached to entities:

```rust
use praxis_ecs::{World, Transform, ParticleEmitter};
use praxis_math::Vec3;

let mut world = World::new();

// Create a moving fire emitter
world.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    ParticleEmitter::new("fire"),
));

// The particle system can query emitter positions from the ECS
```

## Update Loop

Typical usage in the main loop:

```rust
// In your game loop
let delta_time = /* calculate frame time */;

// Update all particle emitters
particle_system.update(delta_time);

// Prepare instance data for rendering
particle_system.prepare_render()?;

// Render particles (integrate with your rendering pipeline)
// let instance_buffer = particle_system.instance_buffer();
// let quad_vertices = particle_system.quad_vertex_buffer();
// let quad_indices = particle_system.quad_index_buffer();
```

## Performance Considerations

### Particle Limits

Each emitter has a maximum particle count:

```rust
let config = ParticleEmitterConfig {
    max_particles: 1000,  // Will not exceed this
    ..Default::default()
};
```

The global limit per emitter is `MAX_PARTICLES_PER_EMITTER` (10,000).

### Emission Rate vs. Lifetime

Total particles = emission_rate × particle_lifetime

```rust
// 100 particles per second × 2 seconds = ~200 particles at steady state
let config = ParticleEmitterConfig {
    emission_rate: 100.0,
    particle_lifetime: 2.0,
    max_particles: 250,  // Allow some headroom
    ..Default::default()
};
```

### GPU Instancing

The system uses GPU instancing for efficient rendering:
- One quad geometry (4 vertices, 6 indices)
- One draw call per frame for all particles
- Each particle is an instance with per-instance data

### Memory Usage

Per particle instance: 36 bytes
- Position: 12 bytes (3 × f32)
- Color: 16 bytes (4 × f32)
- Size: 4 bytes (f32)
- Rotation: 4 bytes (f32)
- Atlas index: 4 bytes (f32)

With 10,000 particles: ~360 KB

## Troubleshooting

### Particles Not Appearing

1. Check emitter is active: `emitter.is_active()`
2. Verify emission rate > 0
3. Ensure particle lifetime > 0
4. Check max_particles is sufficient

### Performance Issues

1. Reduce `max_particles`
2. Lower `emission_rate`
3. Shorten `particle_lifetime`
4. Use fewer emitters
5. Optimize force calculations

### Unexpected Behavior

1. Check force directions and strengths
2. Verify color/size gradients have valid values
3. Ensure emitter position is correct
4. Check looping vs. one-shot behavior

## Example: Complete Fire Effect

```rust
use praxis_graphics::{ParticleSystem, ParticleEmitterConfig, EmitterShape, ParticleForce};
use praxis_math::Vec3;

// Initialize particle system
let mut particle_system = ParticleSystem::new(
    memory_allocator,
    command_buffer_allocator,
    queue,
)?;

// Configure fire emitter
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
        ParticleForce::Gravity { strength: Vec3::new(0.0, 1.0, 0.0) },
        ParticleForce::Wind {
            direction: Vec3::new(1.0, 0.0, 0.0),
            strength: 0.5,
            turbulence: 0.3,
        },
        ParticleForce::Drag { coefficient: 0.5 },
    ],
    looping: true,
    ..Default::default()
};

particle_system.add_emitter("campfire", fire_config);

// Position the emitter
if let Some(emitter) = particle_system.get_emitter_mut("campfire") {
    emitter.set_position(Vec3::new(0.0, 0.0, 0.0));
}

// In game loop
loop {
    let delta_time = calculate_delta_time();
    
    particle_system.update(delta_time);
    particle_system.prepare_render()?;
    
    // Render...
}
```

## See Also

- [Graphics System Documentation](./rendering.md)
- [ECS Documentation](./ARCHITECTURE.md)
- Example: `examples/particles_demo.rs`
