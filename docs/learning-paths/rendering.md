# Rendering Learning Path

Master the Praxis graphics pipeline from basic rendering to advanced custom pipelines.

## Path Overview

**Time Investment**: 3-6 weeks depending on prior graphics experience  
**Prerequisites**: Basic understanding of 3D math (vectors, matrices)  
**Final Goal**: Build custom rendering pipelines with advanced effects

## Progression Map

```
Beginner (1-2 weeks)
├── Vulkan basics
├── Forward rendering
├── Materials & lighting
└── Basic optimization
    ↓
Intermediate (2-3 weeks)
├── Deferred rendering
├── HDR & tone mapping
├── Shadow mapping
├── Environment probes
└── Post-processing
    ↓
Advanced (2-3 weeks)
├── Custom shaders
├── Pipeline optimization
├── GPU-driven rendering
└── Advanced techniques
```

---

## Beginner: Forward Rendering Fundamentals

**Goal**: Understand and use the forward rendering pipeline effectively.

### Prerequisites

- ✓ Installed Praxis engine
- ✓ Basic 3D math knowledge (or willing to learn)
- ✓ Completed [Getting Started](../getting-started/README.md)

### Step 1: Understand the Pipeline

**Theory** (2-3 hours):
1. Read [Vulkan Rendering Concepts](../concepts/vulkan-rendering.md)
   - Pipeline stages (vertex → fragment)
   - Render passes and framebuffers
   - Descriptor sets and bindings

2. Read [Beginner's Guide: Rendering Pipeline Flow](../beginners-guide.md#rendering-pipeline-flow)
   - Application initialization
   - Per-frame rendering
   - Command buffer recording

**Key Concepts to Master**:
- What is a vertex shader?
- What is a fragment shader?
- How does data flow from CPU to GPU?
- What are descriptor sets?

### Step 2: Basic Rendering

**Practice** (3-4 hours):
1. Read [Rendering Guide](../guides/rendering.md) - Forward rendering section
2. Run basic example:
   ```bash
   cargo run --example scene_demo
   ```
3. Study the code in `examples/scene_demo.rs`

**Exercises**:
1. Modify mesh positions
2. Change mesh colors
3. Add a second mesh
4. Experiment with camera angles

**Expected Output**: Rendered 3D scene with meshes

### Step 3: Materials and PBR

**Theory** (2 hours):
1. Read [PBR Materials Concepts](../concepts/pbr-materials.md)
   - Albedo, metallic, roughness
   - Energy conservation
   - Physically-based workflow

**Practice** (2-3 hours):
1. Continue [Rendering Guide: Material System](../guides/rendering.md#material-system)
2. Run material example:
   ```bash
   cargo run --example material_demo
   ```

**Exercises**:
1. Create a gold material (high metallic, low roughness)
2. Create a rubber material (low metallic, high roughness)
3. Create a glowing material (high emissive)
4. Mix different material properties

### Step 4: Lighting

**Theory** (2 hours):
1. Read [Lighting Concepts](../concepts/lighting.md)
   - Directional lights (sun)
   - Point lights (bulbs)
   - Attenuation and intensity

2. Read [Beginner's Guide: Lighting System](../beginners-guide.md#lighting-system-architecture)

**Practice** (3-4 hours):
1. Continue [Rendering Guide: Lighting](../guides/rendering.md#lighting)
2. Experiment with light placement

**Exercises**:
1. Add a directional light (sun)
2. Add multiple point lights
3. Animate light positions
4. Adjust light colors and intensities

### Step 5: Optimization Basics

**Theory** (2 hours):
1. Read [Spatial Optimization](../guides/spatial-optimization.md) - Introduction
   - Frustum culling
   - Basic LOD concepts

**Practice** (2 hours):
1. Run optimization example:
   ```bash
   cargo run --example spatial_optimization_demo
   ```

**Exercises**:
1. Enable frustum culling
2. Observe performance differences
3. Use built-in profiler

### Beginner Checkpoint

**Self-Assessment**:
- [ ] Can render basic 3D scenes
- [ ] Understand material properties (albedo, metallic, roughness)
- [ ] Can add and configure lights
- [ ] Know when objects are/aren't rendered (culling)
- [ ] Comfortable with the rendering API

**Capstone Project**: Create a simple scene with:
- 5+ different meshes
- 3+ different materials
- 2+ light sources
- Working frustum culling

**Time to Complete**: 15-20 hours

---

## Intermediate: Advanced Rendering Techniques

**Goal**: Master advanced rendering features and optimization techniques.

### Prerequisites

- ✓ Completed Beginner section
- ✓ Comfortable with forward rendering
- ✓ Understanding of GPU concepts

### Step 1: Deferred Rendering

**Theory** (3-4 hours):
1. Read [Deferred Rendering Guide](../guides/deferred-rendering.md)
   - G-buffer architecture
   - Geometry pass vs lighting pass
   - Forward vs deferred tradeoffs

2. Study comparison:
   - Forward: O(objects × triangles × lights)
   - Deferred: O(objects × triangles) + O(pixels × lights)

**Practice** (4-5 hours):
1. Convert forward renderer to deferred
2. Examine G-buffer contents
3. Run example:
   ```bash
   cargo run --example advanced_lighting_demo
   ```

**Exercises**:
1. Switch between forward and deferred
2. Compare performance with many lights (1 vs 10 vs 50)
3. Visualize G-buffer components
4. Measure memory usage

**When to Use**:
- Deferred: 5+ lights, complex lighting
- Forward: Few lights, transparency needed

### Step 2: HDR and Tone Mapping

**Theory** (2-3 hours):
1. Read [HDR and Tone Mapping Guide](../guides/hdr-and-tonemapping.md)
   - What is HDR?
   - Why exceed [0,1] range?
   - Tone mapping operators (ACES, Reinhard)
   - Exposure control

**Practice** (3-4 hours):
1. Enable HDR rendering
2. Implement tone mapping
3. Compare tone mapping algorithms

**Exercises**:
1. Create over-bright lights (intensity > 1.0)
2. Switch between ACES and Reinhard
3. Adjust exposure values
4. Observe differences in bright/dark areas

**Visual Test**: Bright explosion in dark room should look realistic

### Step 3: Shadow Mapping

**Theory** (3-4 hours):
1. Read [Shadow Mapping Guide](../guides/shadows.md)
   - Shadow map generation
   - Cascaded shadow maps (CSM)
   - Percentage Closer Filtering (PCF)
   - Shadow bias and artifacts

2. Read [Beginner's Guide: Shadow Mapping](../beginners-guide.md#shadow-mapping-system)

**Practice** (5-6 hours):
1. Enable shadow mapping
2. Configure cascade splits
3. Adjust PCF kernel size
4. Tune shadow bias

**Exercises**:
1. Add shadows to directional light
2. Visualize cascade splits
3. Adjust shadow quality vs performance
4. Fix shadow artifacts (acne, peter-panning)

**Common Issues**:
- Shadow acne: Increase bias
- Peter-panning: Decrease bias
- Cascade seams: Blend between cascades

### Step 4: Environment Probes (IBL)

**Theory** (3 hours):
1. Read [Environment Probes Guide](../guides/environment-probes.md)
   - Image-based lighting (IBL)
   - Reflection captures
   - Ambient lighting from environment
   - Pre-filtered environment maps

**Practice** (4-5 hours):
1. Load environment maps
2. Enable IBL reflections
3. Run example:
   ```bash
   cargo run --example environment_probe_demo
   ```

**Exercises**:
1. Try different environment maps (indoor, outdoor)
2. Adjust reflection intensity
3. Compare with/without IBL
4. Use IBL for realistic metallic materials

**Visual Impact**: Metallic materials should reflect environment

### Step 5: Post-Processing

**Theory** (2-3 hours):
1. Read [Post-Processing Guide](../guides/post-processing.md)
   - Bloom effect
   - Color grading
   - Vignette, chromatic aberration
   - Effect composition

**Practice** (4-5 hours):
1. Implement bloom
2. Add color grading LUT
3. Chain multiple effects

**Exercises**:
1. Add bloom to bright lights
2. Create different color grades (warm, cool, noir)
3. Combine multiple effects
4. Adjust effect intensity

### Intermediate Checkpoint

**Self-Assessment**:
- [ ] Can choose between forward and deferred rendering
- [ ] Understand HDR and tone mapping
- [ ] Can implement realistic shadows
- [ ] Know how to use environment probes
- [ ] Can apply post-processing effects
- [ ] Comfortable with performance tradeoffs

**Capstone Project**: Create a visually rich scene with:
- Deferred rendering with 10+ lights
- HDR with tone mapping
- Cascaded shadow maps
- IBL reflections
- Post-processing stack (bloom + color grade)

**Time to Complete**: 30-40 hours

---

## Advanced: Custom Pipeline Development

**Goal**: Create custom rendering pipelines and optimize for production.

### Prerequisites

- ✓ Completed Intermediate section
- ✓ Strong understanding of Vulkan concepts
- ✓ Proficient with shader programming

### Step 1: Pipeline Architecture

**Theory** (4-5 hours):
1. Read [Architecture: Render Pipeline](../architecture/render-pipeline.md)
2. Read [Beginner's Guide: Vulkano Abstractions](../beginners-guide.md#vulkanvulkano-abstractions)
3. Study `praxis_graphics` crate internals

**Key Concepts**:
- Pipeline state objects
- Descriptor set layouts
- Render pass compatibility
- Dynamic state

**Practice** (5-6 hours):
1. Trace through render pass creation
2. Study descriptor set allocation
3. Examine shader compilation

**Understanding Goal**: How RenderContext works internally

### Step 2: Custom Shaders

**Theory** (3 hours):
1. Read [Shaders Reference](../reference/shaders.md)
2. Review GLSL/SPIR-V compilation
3. Study existing shaders in `praxis_graphics/src/shaders/`

**Practice** (8-10 hours):
1. Create custom vertex shader
2. Create custom fragment shader
3. Add custom uniform data
4. Integrate into render pipeline

**Exercises**:
1. Implement custom vertex deformation
2. Add custom lighting model
3. Create stylized shading (toon, cel)
4. Implement vertex colors

**Challenge**: Create a complete custom material system

### Step 3: Descriptor Set Optimization

**Theory** (3-4 hours):
1. Read [Beginner's Guide: Dynamic Uniform Buffers](../beginners-guide.md#dynamic-uniform-buffer-ring-system)
2. Study descriptor set pooling
3. Understand descriptor set caching

**Practice** (6-8 hours):
1. Implement dynamic uniform buffers
2. Profile descriptor set allocations
3. Optimize descriptor set reuse
4. Batch similar materials

**Optimization Goals**:
- Minimize descriptor set allocations
- Reuse descriptor sets across frames
- Batch by material to reduce bindings

**Measurement**: Use [Profiling](../profiling.md) to verify improvements

### Step 4: GPU-Driven Rendering

**Theory** (4 hours):
1. Study GPU culling techniques
2. Research indirect drawing
3. Understand compute shaders

**Practice** (10-12 hours):
1. Implement GPU frustum culling
2. Use indirect draw calls
3. Compute shader occlusion culling

**Advanced Techniques**:
- Indirect drawing
- GPU occlusion queries
- Hi-Z occlusion culling
- Meshlet rendering (future)

### Step 5: Advanced Effects

**Specialized Topics** (choose based on interest):

**Option A: Particle Systems**
1. Read [Particles Guide](../guides/particles.md)
2. Study `crates/praxis_graphics/PARTICLES.md`
3. Implement GPU particle simulation
4. Run example:
   ```bash
   cargo run --example particles_demo
   ```

**Option B: Procedural Generation**
1. Read [Procedural Textures](../procedural-textures.md)
2. Implement runtime texture generation
3. Create texture graphs
4. Run example:
   ```bash
   cargo run --example procedural_texture_demo
   ```

**Option C: Terrain Rendering**
1. Read [Terrain System](../terrain-system.md)
2. Implement heightmap-based terrain
3. Add terrain LOD
4. Run example:
   ```bash
   cargo run --example terrain_demo
   ```

### Advanced Checkpoint

**Self-Assessment**:
- [ ] Understand Vulkano abstraction layers
- [ ] Can create custom shaders and pipelines
- [ ] Know how to optimize descriptor sets
- [ ] Familiar with GPU-driven techniques
- [ ] Can implement advanced effects
- [ ] Can profile and optimize rendering

**Capstone Project**: Choose one:

1. **Custom Renderer**: Build a specialized renderer (e.g., stylized, retro)
2. **Visual Effect**: Implement a complex effect (volumetric fog, water)
3. **Optimization**: Profile and optimize existing renderer (2x FPS improvement)

**Time to Complete**: 40-60 hours

---

## Cross-References

### Performance Optimization
- [Profiling](../profiling.md) - Measure rendering performance
- [Spatial Optimization](../guides/spatial-optimization.md) - Culling techniques
- [LOD System](../lod-system.md) - Level of detail

### Related Systems
- [Camera System](../camera-system.md) - Camera controls
- [Mesh System](../mesh-system.md) - Geometry management
- [Assets Guide](../guides/assets.md) - Load models and textures

### Engine Internals
- [Architecture](../architecture.md) - Overall design
- [ECS Patterns](../architecture/ecs-patterns.md) - Data organization
- [Crates: praxis_graphics](../reference/crates.md) - Graphics crate details

---

## Practice Resources

### Examples to Study
```bash
# Beginner
cargo run --example scene_demo
cargo run --example material_demo

# Intermediate  
cargo run --example advanced_lighting_demo
cargo run --example environment_probe_demo

# Advanced
cargo run --example advanced_material_demo
cargo run --example advanced_rendering_demo
cargo run --example particles_demo
cargo run --example procedural_texture_demo
```

### External Resources
- Vulkan Tutorial: https://vulkan-tutorial.com/
- Learn OpenGL (concepts apply): https://learnopengl.com/
- Real-Time Rendering book
- GPU Gems series

---

## Next Steps

After completing this path:

1. **Specialize**: Deep dive into specific areas (particles, terrain, etc.)
2. **Integrate**: Combine with [Animation Path](animation.md) for animated models
3. **Optimize**: Focus on [Performance Path](performance.md)
4. **Create**: Build a complete game or demo scene

## Getting Help

- Check `examples/` for working code
- Review `praxis_graphics` crate documentation
- Study shader code in `praxis_graphics/src/shaders/`
- Profile with built-in tools

---

[← Back to Learning Paths](README.md) | [Next: Animation Path →](animation.md)
