# Particle System Module

GPU-accelerated particle system with support for multiple emitters, physical forces, and texture atlases.

## Quick Start

```rust
use praxis_graphics::{ParticleSystem, ParticleEmitterConfig, EmitterShape};
use praxis_math::Vec3;

let mut particle_system = ParticleSystem::new(
    memory_allocator,
    command_buffer_allocator,
    queue,
)?;

let config = ParticleEmitterConfig {
    shape: EmitterShape::Sphere { radius: 1.0 },
    emission_rate: 50.0,
    particle_lifetime: 2.0,
    initial_velocity: Vec3::new(0.0, 5.0, 0.0),
    ..Default::default()
};

particle_system.add_emitter("fire", config);

// In game loop
particle_system.update(delta_time);
particle_system.prepare_render()?;
```

## Features

- **Multiple Emitter Shapes**: Point, sphere, box, circle, cone
- **Physical Forces**: Gravity, wind, attraction, radial, drag
- **Color/Size Gradients**: Interpolated over particle lifetime
- **Texture Atlases**: Sprite sheet support
- **GPU Instancing**: Efficient rendering of thousands of particles
- **ECS Integration**: `ParticleEmitter` component

## Documentation

See [docs/particle_system.md](../../docs/particle_system.md) for complete documentation.

## Example

See [examples/particles_demo.rs](../../examples/particles_demo.rs) for a full demonstration.
