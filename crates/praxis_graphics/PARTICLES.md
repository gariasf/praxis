# Particle System

GPU-accelerated particle rendering with collision detection, GPU sorting, and soft particles.

## Features

- **Multiple Emitter Shapes**: Point, sphere, box, circle, cone
- **Physical Forces**: Gravity, wind, attraction, radial, drag
- **Color/Size Gradients**: Interpolated over lifetime
- **Texture Atlases**: Sprite sheet support
- **GPU Instancing**: Efficient rendering of thousands of particles
- **Spatial Hashing**: O(n) particle-particle collisions
- **World Collisions**: Infinite plane colliders
- **GPU Sorting**: Bitonic sort for correct alpha blending
- **Soft Particles**: Depth-aware fade near geometry

## Quick Start

```rust
use praxis_graphics::{ParticleRenderer, ParticleEmitterConfig, EmitterShape};

let mut particle_renderer = ParticleRenderer::new(
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

particle_renderer.add_emitter("fire", config);

// In game loop
particle_renderer.update(delta_time);
particle_renderer.prepare_render()?;
```

## Emitter Configuration

### Shapes

```rust
use praxis_graphics::EmitterShape;

// Point source
EmitterShape::Point

// Sphere volume
EmitterShape::Sphere { radius: 1.0 }

// Box volume
EmitterShape::Box { half_extents: Vec3::splat(1.0) }

// Circle on XZ plane
EmitterShape::Circle { radius: 2.0 }

// Cone along Y axis
EmitterShape::Cone { 
    radius: 1.0, 
    height: 2.0 
}
```

### Forces

```rust
use praxis_graphics::ParticleForce;

// Gravity
ParticleForce::Gravity {
    strength: Vec3::new(0.0, -9.8, 0.0)
}

// Constant wind
ParticleForce::Wind {
    direction: Vec3::new(1.0, 0.0, 0.0),
    strength: 2.0,
}

// Attract to point
ParticleForce::Attraction {
    position: Vec3::ZERO,
    strength: 5.0,
}

// Radial force (explosion/implosion)
ParticleForce::Radial {
    center: Vec3::new(0.0, 2.0, 0.0),
    strength: 10.0,
}

// Velocity damping
ParticleForce::Drag {
    coefficient: 0.1,
}
```

### Color and Size Gradients

```rust
use praxis_graphics::ParticleEmitterConfig;

let config = ParticleEmitterConfig {
    // Color over lifetime
    color_over_lifetime: vec![
        (0.0, Vec3::new(1.0, 1.0, 0.0)),  // Start: Yellow
        (0.5, Vec3::new(1.0, 0.5, 0.0)),  // Mid: Orange
        (1.0, Vec3::new(1.0, 0.0, 0.0)),  // End: Red
    ],
    
    // Size over lifetime
    size_over_lifetime: vec![
        (0.0, 0.1),   // Start: Small
        (0.3, 0.5),   // Grow
        (1.0, 0.1),   // Shrink at end
    ],
    
    ..Default::default()
};
```

## Collision Detection

### Particle-Particle Collisions

Spatial hashing enables efficient O(n) collision detection:

```rust
let config = ParticleEmitterConfig {
    enable_collisions: true,
    collision_radius: 0.5,
    restitution: 0.7,  // 0 = no bounce, 1 = perfect bounce
    friction: 0.2,
    ..Default::default()
};
```

**Algorithm:**
1. World divided into grid cells
2. Particles hashed to cells by position
3. Collisions checked only within neighboring cells (3×3×3)
4. Result: O(n) instead of O(n²)

**Collision Response:**
- **Separation**: Particles pushed apart to resolve overlap
- **Impulse**: Velocity adjusted based on restitution
- **Friction**: Tangential velocity reduced

### World Collision Planes

Particles can collide with infinite planes:

```rust
use praxis_graphics::CollisionPlane;

// Ground plane at y=0
let ground = CollisionPlane::new(
    Vec3::new(0.0, 0.0, 0.0),  // Point on plane
    Vec3::new(0.0, 1.0, 0.0)    // Normal vector (up)
);
particle_renderer.add_collision_plane(ground);

// Wall plane
let wall = CollisionPlane::new(
    Vec3::new(5.0, 0.0, 0.0),   // Point at x=5
    Vec3::new(-1.0, 0.0, 0.0)   // Normal (pointing left)
);
particle_renderer.add_collision_plane(wall);
```

**Collision Response:**
1. Velocity reflected across plane normal
2. Velocity scaled by restitution
3. Tangent velocity reduced by friction

## GPU Sorting

Particles are sorted on GPU using bitonic sort for correct alpha blending:

```rust
// Enable GPU sorting (enabled by default)
particle_renderer.set_gpu_sorting_enabled(true);

// Set camera position for depth sorting
particle_renderer.set_camera_position(camera_pos);
```

**Process:**
1. Particles uploaded to GPU buffer
2. Bitonic sort compute shader sorts by camera distance
3. Sorted particles rendered back-to-front
4. Correct alpha blending achieved

**Performance:**
- Workgroup size: 256 threads
- Efficient for 100-10,000 particles
- Pads to power-of-two count

## Soft Particles

Particles fade smoothly near geometry using depth buffer comparison:

```rust
use praxis_graphics::SoftParticleConfig;

particle_renderer.set_soft_particle_config(SoftParticleConfig {
    fade_distance: 1.0,  // Distance over which to fade
    fade_power: 2.0,     // Power for fade curve (higher = sharper)
});
```

**Implementation:**
- Fragment shader compares particle depth with scene depth
- Particles fade within `fade_distance` of geometry
- Eliminates hard intersections
- Creates natural-looking effects

**Shader code:**
```glsl
float scene_depth = texture(depth_texture, screen_uv).r;
float particle_depth = gl_FragCoord.z;
float depth_diff = scene_depth - particle_depth;
float fade = smoothstep(0.0, fade_distance, depth_diff);
alpha *= pow(fade, fade_power);
```

## Complete Example

```rust
use praxis_graphics::{
    ParticleRenderer, ParticleEmitterConfig, EmitterShape,
    CollisionPlane, SoftParticleConfig, ParticleForce,
};

// Create renderer
let mut particle_renderer = ParticleRenderer::new(
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
    
    // Collisions
    enable_collisions: true,
    collision_radius: 0.3,
    restitution: 0.8,
    friction: 0.2,
    
    // Forces
    forces: vec![
        ParticleForce::Gravity {
            strength: Vec3::new(0.0, -9.8, 0.0)
        }
    ],
    
    // Appearance
    color_over_lifetime: vec![
        (0.0, Vec3::new(1.0, 1.0, 1.0)),
        (1.0, Vec3::new(0.5, 0.5, 0.5)),
    ],
    
    ..Default::default()
};

particle_renderer.add_emitter("bouncing", config);

// Add ground plane
let ground = CollisionPlane::new(
    Vec3::ZERO,
    Vec3::new(0.0, 1.0, 0.0)
);
particle_renderer.add_collision_plane(ground);

// Enable soft particles
particle_renderer.set_soft_particle_config(SoftParticleConfig {
    fade_distance: 0.5,
    fade_power: 2.0,
});

// Update loop
loop {
    particle_renderer.set_camera_position(camera_pos);
    particle_renderer.update(delta_time);
    particle_renderer.prepare_render()?;
}
```

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
5. **Reduce max_particles** for better performance

## Shaders

### `particle.vert`

- Billboard particles (always face camera)
- Per-particle rotation
- Instanced rendering

```glsl
// Transform to world space
vec3 world_pos = particle_position;

// Billboard to face camera
vec3 right = normalize(cross(camera_up, camera_forward));
vec3 up = cross(camera_forward, right);

// Expand quad
vec3 vertex_pos = world_pos 
    + right * position.x * particle_size
    + up * position.y * particle_size;
```

### `particle.frag`

- Soft particle depth comparison
- Edge fade for smooth appearance
- Texture sampling with color modulation

```glsl
// Sample texture
vec4 tex_color = texture(particle_texture, uv);

// Soft particle fade
float depth_fade = calculate_depth_fade(screen_uv);

// Apply color and alpha
vec4 final_color = tex_color * particle_color;
final_color.a *= depth_fade * particle_alpha;
```

### `particle_sort.comp`

- Bitonic sort algorithm
- Sorts by camera distance
- Runs on GPU compute queue

```glsl
layout(local_size_x = 256) in;

void main() {
    uint i = gl_GlobalInvocationID.x;
    // Bitonic sort implementation...
}
```

## Limitations

1. **Collision accuracy**: Spatial hashing uses discrete cells
2. **Sort power-of-two**: GPU sort pads to nearest power of two
3. **Soft particles require depth**: Need scene depth buffer
4. **Transparent only**: Particles always use alpha blending
5. **Single texture per emitter**: No per-particle texture variation

## See Also

- Example: `examples/particles_demo.rs`
- Example: `examples/comprehensive_scene_demo.rs`
- [Rendering Guide](../../docs/guides/rendering/particles.md)
- Implementation: `crates/praxis_graphics/src/particles.rs`
