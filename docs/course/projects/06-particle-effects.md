# Project 06: Particle Effects System

**Difficulty**: Intermediate  
**Estimated Time**: 2-3 weeks  
**Core Learning**: Particle systems, GPU compute, instancing, visual effects composition

## Overview

Build a flexible particle effects system that creates fire, smoke, sparks, and other visual effects. This project teaches GPU-accelerated particle simulation, billboard rendering, texture atlases, and effect composition techniques used in modern games.

### Learning Objectives

- Implement CPU and GPU particle simulation
- Use instanced rendering for thousands of particles
- Create particle emitters with various behaviors
- Apply particle textures and blending modes
- Build effect composition system (combine emitters)
- Optimize for performance (100k+ particles)

## Feature Requirements

### Core Features (Minimum Viable)

1. **Basic Particle System**
   - Particle struct (position, velocity, lifetime, size, color)
   - Emitter that spawns particles
   - Update loop (integrate velocity, apply gravity)
   - Particle death/recycling
   - Render particles as billboards (face camera)

2. **Emitter Properties**
   - Spawn rate (particles per second)
   - Initial velocity (direction + spread)
   - Lifetime range (min/max)
   - Size range
   - Color gradient (over lifetime)
   - Gravity and drag

3. **Rendering**
   - Billboard particles (always face camera)
   - Additive blending (for fire/sparks)
   - Alpha blending (for smoke)
   - Textured particles
   - Instanced rendering

4. **Simple Effects**
   - Fire
   - Smoke
   - Sparks
   - Explosion (burst emit)

### Extended Features (Recommended)

5. **Advanced Emitter Shapes**
   - Point emitter
   - Sphere/hemisphere emitter
   - Cone emitter
   - Box/volume emitter
   - Mesh surface emitter

6. **Forces and Behaviors**
   - Attractor/repulsor forces (point gravity)
   - Vortex (spiral motion)
   - Turbulence (noise-based motion)
   - Collision with planes/spheres
   - Velocity over lifetime curves

7. **GPU Particle Simulation**
   - Compute shader-based update
   - 100k+ particles in real-time
   - Parallel particle updates
   - GPU sorting (back-to-front for alpha blending)

### Stretch Goals

8. **Advanced Rendering**
   - Soft particles (depth buffer fade)
   - Distortion particles (heat haze, ripples)
   - Mesh particles (3D models as particles)
   - Particle trails (motion blur effect)
   - Lit particles (interact with scene lighting)

9. **Effect Composition**
   - Multiple emitters per effect
   - Sub-emitters (particles spawn particles)
   - Effect presets library
   - Visual effect editor
   - Trigger events (sound on spawn/death)

## Architecture Guidance

### System Components

```
ParticleEffectSystem
├── ParticleEmitter
│   ├── SpawnController
│   ├── InitialStateGenerator
│   └── EmitterShape
├── ParticleSimulator
│   ├── CPUSimulator
│   ├── GPUSimulator (compute shader)
│   └── ForceRegistry
├── ParticleRenderer
│   ├── BillboardRenderer
│   ├── InstancedRenderer
│   └── SortingSystem
├── EffectComposer
│   ├── EffectDefinition
│   ├── EmitterGroup
│   └── SubEmitterController
└── ResourceManager
    ├── ParticleTextureAtlas
    ├── MaterialLibrary
    └── EffectPresetLibrary
```

### Data Structures

**Particle**
```
Particle:
  - position: vec3
  - velocity: vec3
  - acceleration: vec3
  - lifetime: float (seconds remaining)
  - max_lifetime: float (total lifetime)
  - size: float or vec2 (width, height)
  - rotation: float (billboard rotation)
  - color: vec4 (RGBA)
  - texture_index: int (for texture atlas)
  - is_alive: bool

Derived Properties:
  - age: float = max_lifetime - lifetime
  - age_normalized: float = age / max_lifetime (0-1)
```

**Particle Emitter**
```
ParticleEmitter:
  - position: vec3
  - rotation: quaternion
  - shape: EmitterShape (Point | Sphere | Cone | Box)
  - spawn_rate: float (particles/second)
  - burst_count: int (0 = continuous)
  - max_particles: int
  - duration: float (-1 = infinite)
  
  # Initial particle properties
  - initial_velocity: vec3
  - velocity_spread: float (randomness)
  - lifetime_range: (min, max)
  - size_range: (min, max)
  - color_gradient: array of (time, color)
  - size_over_lifetime: curve or array
  
  # Simulation properties
  - gravity: vec3
  - drag: float (air resistance)
  - forces: array of Force
  
  # Rendering properties
  - material: ParticleMaterial
  - blend_mode: Additive | Alpha | Opaque
  - sort_mode: None | BackToFront | OldestFirst

Methods:
  - emit(count)
  - update(delta_time)
  - get_live_particles() -> array
```

**Emitter Shape**
```
EmitterShape:
  - type: Point | Sphere | Hemisphere | Cone | Box | Mesh
  - parameters: varies by type
  
  # Sphere
  - radius: float
  - emit_from_shell: bool (surface only vs volume)
  
  # Cone
  - radius: float
  - angle: float (cone angle in degrees)
  - height: float
  
  # Box
  - extents: vec3 (half-dimensions)

Methods:
  - sample_position() -> vec3
  - sample_direction() -> vec3
```

**Particle Material**
```
ParticleMaterial:
  - texture: Texture (or atlas)
  - blend_mode: Additive | Alpha | Multiply
  - color_tint: vec4
  - use_vertex_color: bool
  - soft_particle_distance: float (for depth fade)
  - distortion_strength: float (for heat haze)
```

### Simulation Update Loop

**CPU Simulation**
```
update_particles(delta_time):
  for particle in alive_particles:
    # Apply forces
    particle.acceleration = gravity
    for force in emitter.forces:
      particle.acceleration += force.calculate(particle)
    
    # Integrate velocity
    particle.velocity += particle.acceleration * delta_time
    particle.velocity *= (1.0 - drag * delta_time)  # Apply drag
    
    # Integrate position
    particle.position += particle.velocity * delta_time
    
    # Update lifetime
    particle.lifetime -= delta_time
    if particle.lifetime <= 0:
      kill_particle(particle)
    
    # Update visual properties
    age_norm = particle.age_normalized
    particle.color = sample_gradient(color_gradient, age_norm)
    particle.size = sample_curve(size_over_lifetime, age_norm)
```

**GPU Simulation (Compute Shader)**
```glsl
layout(local_size_x = 256) in;

struct Particle {
  vec3 position;
  vec3 velocity;
  float lifetime;
  float max_lifetime;
  vec4 color;
  float size;
  // ... other properties
};

layout(std430, binding = 0) buffer ParticleBuffer {
  Particle particles[];
};

uniform float delta_time;
uniform vec3 gravity;
uniform float drag;

void main() {
  uint id = gl_GlobalInvocationID.x;
  if (id >= particle_count) return;
  
  Particle p = particles[id];
  if (p.lifetime <= 0.0) return;
  
  // Update velocity
  p.velocity += gravity * delta_time;
  p.velocity *= (1.0 - drag * delta_time);
  
  // Update position
  p.position += p.velocity * delta_time;
  
  // Update lifetime
  p.lifetime -= delta_time;
  
  // Update visual properties
  float age_norm = 1.0 - (p.lifetime / p.max_lifetime);
  p.color = sample_color_gradient(age_norm);
  p.size = sample_size_curve(age_norm);
  
  particles[id] = p;
}
```

### Billboard Rendering

**Vertex Shader**
```glsl
// Input: particle center position + instance data
layout(location = 0) in vec3 particle_position;
layout(location = 1) in vec4 particle_color;
layout(location = 2) in float particle_size;
layout(location = 3) in float particle_rotation;

// Per-vertex offset (quad corners: -1,-1 to 1,1)
layout(location = 4) in vec2 vertex_offset;

uniform mat4 view;
uniform mat4 projection;

out vec4 frag_color;
out vec2 frag_uv;

void main() {
  // Get camera right and up vectors from view matrix
  vec3 camera_right = vec3(view[0][0], view[1][0], view[2][0]);
  vec3 camera_up = vec3(view[0][1], view[1][1], view[2][1]);
  
  // Apply rotation (optional)
  vec2 rotated_offset = rotate_2d(vertex_offset, particle_rotation);
  
  // Construct billboard
  vec3 world_pos = particle_position +
    camera_right * rotated_offset.x * particle_size +
    camera_up * rotated_offset.y * particle_size;
  
  gl_Position = projection * view * vec4(world_pos, 1.0);
  
  frag_color = particle_color;
  frag_uv = vertex_offset * 0.5 + 0.5;  // Map -1,1 to 0,1
}
```

**Fragment Shader**
```glsl
in vec4 frag_color;
in vec2 frag_uv;

uniform sampler2D particle_texture;

out vec4 out_color;

void main() {
  vec4 tex_color = texture(particle_texture, frag_uv);
  out_color = tex_color * frag_color;
  
  // Discard fully transparent pixels (optimization)
  if (out_color.a < 0.01) discard;
}
```

## Milestone Plan

### Milestone 1: Basic CPU Particle System (Week 1, Days 1-3)

**Goal**: Render and update simple particles

**Tasks**:
- Define Particle struct
- Create particle pool (fixed array)
- Implement spawn logic (emit N particles)
- Update loop (integrate velocity, decrement lifetime)
- Render as point sprites or simple quads
- Add gravity

**Deliverable**: Falling particles that die after lifetime

### Milestone 2: Billboard Rendering (Week 1, Days 4-5)

**Goal**: Particles always face camera

**Tasks**:
- Implement billboard vertex shader
- Create quad geometry for billboards
- Apply particle texture
- Implement alpha blending
- Add color gradient over lifetime
- Tune visual appearance

**Deliverable**: Textured, camera-facing particles

### Milestone 3: Emitter System (Week 1, Days 6-7)

**Goal**: Configurable particle emitters

**Tasks**:
- Create Emitter class/struct
- Implement spawn rate (particles per second)
- Add initial velocity with spread
- Implement burst mode
- Add emitter shapes (point, sphere, cone)
- UI for emitter parameters

**Deliverable**: Interactive emitter with adjustable properties

### Milestone 4: Effect Presets (Week 2, Days 1-3)

**Goal**: Create common effects (fire, smoke, sparks)

**Tasks**:
- Design fire effect (upward velocity, red/yellow gradient, additive blend)
- Design smoke effect (slow rise, fade to black, alpha blend)
- Design spark effect (fast particles, short lifetime, trails)
- Design explosion (burst emit, radial velocity)
- Create preset library
- Quick-spawn buttons in UI

**Deliverable**: Multiple convincing visual effects

### Milestone 5: Instanced Rendering (Week 2, Days 4-5)

**Goal**: Render thousands of particles efficiently

**Tasks**:
- Implement instanced rendering
- Pack particle data into instance buffer
- Update buffer each frame (streaming or persistent)
- Profile performance (aim for 10k+ particles at 60 FPS)
- Add particle count display

**Deliverable**: High-performance rendering

### Milestone 6: GPU Simulation (Week 2-3, Days 6-7+)

**Goal**: Compute shader-based particle update

**Tasks**:
- Write compute shader for particle update
- Upload particle data to GPU storage buffer
- Dispatch compute shader each frame
- Synchronize compute → render pipeline
- Compare CPU vs GPU performance
- Aim for 100k+ particles

**Deliverable**: GPU-accelerated simulation

### Optional Milestone 7: Advanced Features

**Goal**: Soft particles, sub-emitters, forces

**Tasks**:
- Implement soft particles (depth buffer read)
- Add attractor/repulsor forces
- Add turbulence (noise-based force)
- Implement sub-emitters (particles spawn particles)
- Add collision detection (ground plane)

**Deliverable**: Advanced, production-quality system

## Technical Challenges

### Challenge 1: Particle Spawning Rate

**Problem**: Maintaining consistent spawn rate across varying frame rates

**Approach**:
- Use accumulator for fractional particles
- Spawn integer count each frame, carry over remainder
- Handle variable delta time correctly

**Implementation**:
```
spawn_accumulator = 0.0

update(delta_time):
  spawn_accumulator += spawn_rate * delta_time
  spawn_count = floor(spawn_accumulator)
  spawn_accumulator -= spawn_count
  
  for i in 0..spawn_count:
    emit_particle()
```

### Challenge 2: Particle Sorting (Alpha Blending)

**Problem**: Alpha-blended particles need back-to-front rendering

**Approach**:
- Calculate particle distance to camera
- Sort by distance (CPU or GPU)
- Render in sorted order
- Trade-off: sorting cost vs visual quality

**GPU Sorting**:
- Use bitonic sort or radix sort compute shader
- Only sort when camera moves significantly
- Consider approximate sorting (bucketing)

**Alternative**: Use additive blending (order-independent)

### Challenge 3: Texture Atlas for Variety

**Problem**: Different particle types need different textures

**Approach**:
- Pack multiple textures into atlas (e.g., 4x4 grid)
- Store texture index per particle
- Calculate UV offset in shader
- Reduces texture switches, improves performance

**Shader Code**:
```glsl
uniform vec2 atlas_size;  // e.g., (4, 4) for 4x4 atlas
uniform int texture_index;

void main() {
  vec2 tile_size = vec2(1.0) / atlas_size;
  vec2 tile_offset = vec2(
    float(texture_index % int(atlas_size.x)),
    float(texture_index / int(atlas_size.x))
  ) * tile_size;
  
  vec2 uv = tile_offset + frag_uv * tile_size;
  vec4 tex_color = texture(particle_atlas, uv);
}
```

### Challenge 4: GPU Buffer Management

**Problem**: Efficiently update GPU particle buffers each frame

**Approach**:
- Use persistent mapped buffers (if available)
- Triple buffering to avoid stalls
- Use storage buffers (SSBO) for large data
- Write-only mapping when possible

**Buffer Update Pattern**:
```
# Modern approach (persistent mapping)
buffer = create_buffer(size, PERSISTENT | COHERENT)
ptr = map_buffer(buffer)

update_loop():
  # Wait for previous frame's GPU work
  wait_fence(frame_fence)
  
  # Write directly to mapped memory
  memcpy(ptr, particle_data, size)
  
  # Issue GPU commands
  dispatch_compute()
  render()
  
  # Signal completion
  signal_fence(frame_fence)
```

### Challenge 5: Soft Particles (Depth Fade)

**Problem**: Hard edges when particles intersect geometry

**Approach**:
- Read depth buffer in particle fragment shader
- Compare particle depth to scene depth
- Fade alpha based on depth difference
- Requires depth buffer as shader input

**Shader Code**:
```glsl
uniform sampler2D depth_texture;
uniform float soft_distance;

void main() {
  // Sample scene depth
  vec2 screen_uv = gl_FragCoord.xy / screen_size;
  float scene_depth = texture(depth_texture, screen_uv).r;
  float particle_depth = gl_FragCoord.z;
  
  // Calculate depth difference
  float depth_diff = scene_depth - particle_depth;
  float fade = saturate(depth_diff / soft_distance);
  
  // Apply to alpha
  vec4 color = texture(particle_texture, frag_uv) * frag_color;
  color.a *= fade;
  out_color = color;
}
```

## Reference Implementations

### Praxis Engine (Rust)
- **File**: `examples/particles_demo.rs`
- **Crates**: `praxis_graphics` (particle rendering)
- **Concepts**: GPU compute, instanced rendering

### Other Engines/Frameworks

**Unity (C#)**
- Tutorial: "Particle System" (official docs)
- System: Shuriken particle system
- Key APIs: `ParticleSystem`, `ParticleSystemRenderer`, `EmissionModule`

**Unreal Engine (C++)**
- Tutorial: "Niagara VFX System"
- Key APIs: `UNiagaraSystem`, `UNiagaraComponent`, modules

**Godot (GDScript)**
- Node: `CPUParticles3D`, `GPUParticles3D`
- Tutorial: Particle systems documentation

**Three.js (JavaScript)**
- Library: `three-gpu-particle-system` or custom implementation
- Example: Three.js particle examples

**WebGPU (JavaScript/TypeScript)**
- Example: WebGPU compute shader particles
- Pattern: Compute → vertex buffer for rendering

**Bevy (Rust)**
- Plugin: `bevy_hanabi` (GPU particle system)
- Pattern: ECS-based particle effects

## Extension Ideas

### Beginner Extensions
- Particle LOD (fewer particles when distant)
- Particle pooling visualization
- Effect presets save/load
- Color picker for gradients

### Intermediate Extensions
- Ribbon trails (connect particles in sequence)
- Particle collision with scene geometry
- Wind zones (directional force fields)
- Texture animation (flipbook in atlas)

### Advanced Extensions
- Fluid simulation particles (SPH, PBD)
- GPU marching cubes for metaballs
- Vector field forces (baked 3D texture)
- Screen-space particle lighting

## Success Criteria

Your particle effects system should:

1. ✅ Render 10k+ particles at 60 FPS (GPU simulation: 100k+)
2. ✅ Support multiple simultaneous emitters
3. ✅ Provide intuitive parameter tuning (spawn rate, lifetime, etc.)
4. ✅ Create convincing fire, smoke, spark, and explosion effects
5. ✅ Handle particle lifecycle efficiently (spawn, update, death)
6. ✅ Use appropriate blend modes (additive for fire, alpha for smoke)
7. ✅ Feel responsive and visually appealing

## Assessment Rubric

| Category | Beginner | Intermediate | Advanced |
|----------|----------|--------------|----------|
| **Performance** | 1k particles at 30 FPS | 10k particles at 60 FPS | 100k+ with GPU compute |
| **Features** | Basic emitter, lifetime, velocity | Emitter shapes, gradients, presets | Forces, sub-emitters, GPU sorting |
| **Rendering** | Simple billboards, alpha blend | Instancing, textures, additive blend | Soft particles, trails, distortion |
| **Effects** | 1-2 basic effects | Fire, smoke, sparks, explosion | Complex multi-emitter effects |

## Common Pitfalls

1. **Sorting Every Frame**: Only sort when necessary (alpha blending, camera moves)
2. **Memory Leaks**: Properly release GPU buffers when emitters destroyed
3. **Too Many Draw Calls**: Batch all particles in single instanced draw
4. **Incorrect Blending**: Enable depth writes for opaque, disable for transparent
5. **Expensive Shaders**: Minimize fragment shader complexity (no loops)
6. **Not Using Instancing**: Essential for performance with many particles
7. **Fixed Array Size**: Use dynamic allocation or large fixed pool

## Next Steps

After completing this project, you're ready for:
- **Project 05**: Procedural Terrain Generator (add particle effects to terrain)
- **Project 08**: Scene Editor (place and edit particle effects)
- **Project 10**: Mini Game Engine (integrate as VFX subsystem)
