# Particle System

The Praxis particle system provides GPU-accelerated particle rendering with advanced features including collision detection, GPU sorting, and soft particles.

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

## Core Features

- **Multiple Emitter Shapes**: Point, sphere, box, circle, cone
- **Physical Forces**: Gravity, wind, attraction, radial, drag
- **Color/Size Gradients**: Interpolated over particle lifetime
- **Texture Atlases**: Sprite sheet support
- **GPU Instancing**: Efficient rendering of thousands of particles
- **ECS Integration**: `ParticleEmitter` component

## Advanced Features

### Spatial Hashing for Collision Detection

Particle-particle collisions are efficiently detected using spatial hashing:

- World space is divided into a grid
- Particles are hashed into cells based on position
- Collision checks only performed within neighboring cells
- O(n) complexity instead of O(n²)

```rust
let config = ParticleEmitterConfig {
    enable_collisions: true,
    collision_radius: 0.5,
    restitution: 0.7,  // 0 = no bounce, 1 = perfect bounce
    friction: 0.2,
    ..Default::default()
};
```

### World Collision Planes

Particles can collide with infinite planes for ground, walls, etc:

```rust
// Create a ground plane at y=0
let ground = CollisionPlane::new(
    Vec3::new(0.0, 0.0, 0.0),  // Point on plane
    Vec3::new(0.0, 1.0, 0.0)    // Normal vector
);
particle_system.add_collision_plane(ground);
```

### GPU-Based Particle Sorting

Particles are sorted on the GPU using bitonic sort for correct alpha blending:

```rust
// Enable GPU sorting (enabled by default)
particle_system.set_gpu_sorting_enabled(true);

// Set camera position for depth sorting
particle_system.set_camera_position(camera_pos);
```

**How it works:**
1. Particles are uploaded to GPU buffer
2. Bitonic sort compute shader sorts by camera distance
3. Sorted particles rendered back-to-front
4. Correct alpha blending achieved

### Soft Particles

Particles fade smoothly near geometry using depth buffer comparison:

```rust
particle_system.set_soft_particle_config(SoftParticleConfig {
    fade_distance: 1.0,  // Distance over which to fade
    fade_power: 2.0,     // Power for fade curve (higher = sharper)
});
```

**Implementation:**
- Fragment shader compares particle depth with scene depth
- Particles fade out within `fade_distance` of geometry
- Eliminates hard intersections
- Creates natural-looking effects

## Collision Response

### Particle-Particle Collisions

When two particles collide:

1. **Separation**: Particles pushed apart to resolve overlap
2. **Impulse**: Velocity adjusted based on restitution coefficient
3. **Friction**: Tangential velocity reduced based on friction

```rust
// Elastic collision (bouncy)
restitution: 0.9,
friction: 0.1,

// Inelastic collision (soft)
restitution: 0.3,
friction: 0.5,
```

### Particle-Plane Collisions

When a particle hits a plane:

1. **Reflection**: Velocity reflected across plane normal
2. **Damping**: Velocity scaled by restitution
3. **Friction**: Tangent velocity reduced

## Performance Considerations

### Spatial Hashing

- Cell size: 2.0 units (configurable via `SPATIAL_HASH_CELL_SIZE`)
- Hash table size: 4096 buckets
- Collision checks: ~27 cells per particle (3×3×3 neighborhood)

### GPU Sorting

- Uses bitonic sort algorithm
- Pads to power-of-two count
- Workgroup size: 256 threads
- Efficient for 100-10,000 particles

### Optimization Tips

1. **Disable collisions** for non-interactive effects (fire, smoke)
2. **Use CPU sorting** for small particle counts (<100)
3. **Limit collision radius** to reduce spatial hash queries
4. **Batch emitters** that share properties

## Complete Example: Bouncing Particles

```rust
use praxis_graphics::{
    ParticleSystem, ParticleEmitterConfig, EmitterShape,
    CollisionPlane, SoftParticleConfig,
};

// Create system
let mut particle_system = ParticleSystem::new(
    memory_allocator,
    command_buffer_allocator,
    queue,
)?;

// Configure collision emitter
let config = ParticleEmitterConfig {
    shape: EmitterShape::Sphere { radius: 1.0 },
    emission_rate: 100.0,
    max_particles: 1000,
    particle_lifetime: 5.0,
    initial_velocity: Vec3::new(0.0, 10.0, 0.0),
    velocity_randomness: 3.0,
    enable_collisions: true,
    collision_radius: 0.3,
    restitution: 0.8,
    friction: 0.2,
    forces: vec![
        ParticleForce::Gravity {
            strength: Vec3::new(0.0, -9.8, 0.0)
        }
    ],
    ..Default::default()
};

particle_system.add_emitter("bouncing", config);

// Add ground plane
let ground = CollisionPlane::new(
    Vec3::ZERO,
    Vec3::new(0.0, 1.0, 0.0)
);
particle_system.add_collision_plane(ground);

// Enable soft particles
particle_system.set_soft_particle_config(SoftParticleConfig {
    fade_distance: 0.5,
    fade_power: 2.0,
});

// Update loop
loop {
    particle_system.set_camera_position(camera_pos);
    particle_system.update(delta_time);
    particle_system.prepare_render()?;
}
```

## Shader Integration

The particle system uses three shaders:

### `particle.vert`
- Billboard particles (always face camera)
- Per-particle rotation
- Instanced rendering

### `particle.frag`
- Soft particle depth comparison
- Edge fade for smooth appearance
- Texture sampling with color modulation

### `particle_sort.comp`
- Bitonic sort algorithm
- Sorts by camera distance
- Runs on GPU compute queue

## Limitations

1. **Collision accuracy**: Spatial hashing uses discrete cells
2. **Sort power-of-two**: GPU sort pads to nearest power of two
3. **Soft particles require depth**: Need scene depth buffer
4. **Transparent only**: Particles always use alpha blending

## Future Enhancements

- Particle-mesh collisions using BVH
- Fluid simulation with SPH
- GPU particle simulation (compute shader)
- Particle trails and ribbons
- Particle attractors and repellers

## Example Demos

See the following examples for complete demonstrations:
- `examples/particles_demo.rs` - Basic particle system usage
- `examples/comprehensive_scene_demo.rs` - Particles integrated with full scene
