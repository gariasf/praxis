# Rendering Learning Path

Master the Praxis graphics pipeline from basic rendering to advanced custom pipelines.

## Path Overview

**Time Investment**: 4-8 weeks depending on prior graphics experience  
**Prerequisites**: Basic understanding of 3D math (vectors, matrices)  
**Final Goal**: Build modern GPU-driven rendering pipelines with TAA, SSR, GPU culling, and bindless materials

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
Advanced (3-4 weeks)
├── Custom shaders & pipelines
├── Descriptor set optimization
├── Temporal Anti-Aliasing (TAA)
├── Screen-Space Reflections (SSR)
├── GPU culling & indirect drawing
├── Bindless rendering
└── Complete integration
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

### Step 4: Temporal Anti-Aliasing (TAA)

**Goal**: Implement temporal anti-aliasing for high-quality edge smoothing with minimal performance cost.

**Theory** (3-4 hours):
1. Study TAA concepts:
   - Temporal sampling and history reprojection
   - Jitter patterns and sub-pixel offsets
   - Velocity buffers and motion vectors
   - History rejection and clamping
   - Ghosting artifacts and mitigation

2. Read resources:
   - [High Quality Temporal Supersampling (Karis)](http://advances.realtimerendering.com/s2014/epic/TemporalAA.pptx)
   - Understanding neighborhood clamping
   - TAA vs MSAA tradeoffs

**Key Concepts**:
- **Jittering**: Offset projection matrix per frame to sample different sub-pixel positions
- **History Buffer**: Store previous frame's result for temporal accumulation
- **Motion Vectors**: Per-pixel velocity for accurate reprojection
- **Neighborhood Clamping**: Reject history using current frame's color neighborhood (YCoCg color space)

**Practice** (6-8 hours):
1. Implement jitter pattern generation (Halton sequence)
2. Create velocity buffer from camera/object motion
3. Reproject previous frame using motion vectors
4. Implement neighborhood clamping in YCoCg space
5. Blend current and history frames with adaptive weights
6. Run example:
   ```bash
   cargo run --example complete_features_demo
   ```
7. Toggle TAA with '1' key and observe quality improvements

**Exercises**:
1. Implement different jitter patterns (2x2, 4x4, 8x8 Halton)
2. Tune history rejection threshold to minimize ghosting
3. Adjust blend factor (0.9-0.95 typically works well)
4. Add sharpening pass to reduce TAA blur
5. Compare TAA vs no AA on thin geometry edges
6. Measure performance impact (typically 1-2ms)

**Common Issues & Solutions**:
- **Ghosting**: Increase rejection threshold, use tighter neighborhood clamp
- **Excessive blur**: Add sharpening, reduce history weight
- **Flickering**: Ensure consistent jitter pattern, check motion vector precision
- **Disocclusion artifacts**: Improve history rejection at object boundaries

**Visual Test**: Camera movement should show smooth edges without temporal artifacts

**Performance Goal**: < 2ms overhead at 1080p, high-quality AA on all edges

### Step 5: Screen-Space Reflections (SSR)

**Goal**: Add realistic reflections for wet and metallic surfaces using screen-space data.

**Theory** (3-4 hours):
1. Study SSR fundamentals:
   - Ray marching in screen space
   - Hierarchical depth buffer (Hi-Z pyramid)
   - Linear vs binary search refinement
   - Edge fade-out and fresnel falloff
   - Temporal filtering for noise reduction

2. Understand limitations:
   - Can only reflect visible objects
   - Poor for grazing angles
   - Screen-edge artifacts
   - Complement with environment probes

**Key Concepts**:
- **Ray Marching**: Step through depth buffer along reflection vector
- **Hi-Z Acceleration**: Use mipmap chain for large steps, refine with smaller mips
- **Intersection Refinement**: Binary search for precise hit point
- **Confidence Metric**: Weight reflection based on ray quality

**Practice** (8-10 hours):
1. Build depth buffer mipmap chain (Hi-Z pyramid)
2. Implement screen-space ray marching:
   - Transform reflection ray to screen space
   - Step through Hi-Z levels
   - Refine intersection with binary search
3. Add fresnel falloff and edge fade
4. Implement temporal reprojection for stability
5. Integrate with deferred pipeline
6. Run example:
   ```bash
   cargo run --example complete_features_demo
   ```
7. Toggle SSR with '2' key

**Exercises**:
1. Tune ray step count vs quality (32-64 steps typical)
2. Implement adaptive step size based on ray direction
3. Add importance sampling for rough surfaces
4. Compare performance: Hi-Z vs uniform stepping
5. Implement stochastic sampling for rough reflections
6. Blend with environment probe fallback
7. Handle dynamic objects correctly

**Advanced Optimizations**:
- Quarter-resolution ray marching + upsampling
- Tile-based dispatch (only metallic/wet pixels)
- Importance sampling for rough surfaces
- Async compute for ray marching

**Visual Test**: Wet floor should reflect scene accurately, rough materials show blurred reflections

**Performance Goal**: 3-5ms at 1080p with Hi-Z optimization

### Step 6: GPU Culling and Indirect Drawing

**Goal**: Move culling to GPU for massive scene support and reduced CPU bottleneck.

**Theory** (4-5 hours):
1. Study GPU-driven rendering:
   - CPU vs GPU culling tradeoffs
   - Indirect draw buffers (VkDrawIndirectCommand)
   - Compute shader culling passes
   - Frustum and occlusion culling on GPU
   - Multi-draw indirect (MDI) API

2. Read documentation:
   - [GPU-Driven Rendering Pipelines (Wihlidal)](http://advances.realtimerendering.com/s2015/aaltonenhaar_siggraph2015_combined_final_footer_220dpi.pdf)
   - Vulkan indirect drawing specification
   - Study `praxis_graphics/src/gpu_culling.rs`

**Key Concepts**:
- **Indirect Draw Buffer**: GPU-writable buffer containing draw commands
- **Compute Culling**: Parallel GPU culling using compute shaders
- **Bounding Volumes**: Spheres or AABBs for frustum tests
- **Atomic Counters**: Track visible object count on GPU
- **Persistent Buffers**: Reuse buffers across frames

**Practice** (10-12 hours):
1. Study existing implementation in `praxis_graphics/src/gpu_culling.rs`
2. Examine the GPU culling compute shader
3. Understand the culling manager API
4. Run the GPU culling demo:
   ```bash
   cargo run --example gpu_culling_demo
   ```
5. Observe performance with 1000+ objects
6. Examine indirect draw buffer generation
7. Profile CPU vs GPU culling overhead

**Implementation Steps**:
1. Create compute shader for frustum culling:
   ```glsl
   // Extract frustum planes from view-projection matrix
   // Test each object's bounding sphere against planes
   // Write visible objects to indirect draw buffer
   ```
2. Set up storage buffers:
   - Input: Draw commands with bounding volumes
   - Output: Compacted indirect draw buffer
   - Atomic counter for visible count
3. Dispatch compute shader (one thread per object)
4. Execute multi-draw indirect using output buffer
5. Measure CPU time savings

**Exercises**:
1. Implement frustum plane extraction from view-projection matrix
2. Add hierarchical culling (scene graph traversal on GPU)
3. Implement occlusion culling using previous frame's depth
4. Add Hi-Z occlusion culling
5. Measure scaling: 100 vs 1000 vs 10,000 objects
6. Compare CPU frustum culling vs GPU culling overhead
7. Implement two-pass culling (coarse + fine)

**Advanced Techniques**:
- **Hi-Z Occlusion**: Render depth buffer pyramid, test bounding boxes
- **Cluster Culling**: Group objects, cull clusters first
- **Persistent Mapped Buffers**: Minimize CPU-GPU sync
- **Meshlet Rendering**: Per-triangle cluster culling (future)
- **Task/Mesh Shaders**: Next-gen GPU-driven geometry (Vulkan 1.3+)

**Code Reference**:
```rust
// From gpu_culling_demo.rs
let mut gpu_culling = GpuCullingManager::new(
    render_context.device.clone(),
    render_context.memory_allocator().clone(),
    descriptor_allocator,
)?;

// Prepare frame with draw commands and bounding volumes
gpu_culling.prepare_frame(&draw_commands, &mesh_data)?;

// Dispatch culling compute shader
// Execute indirect draw with compacted buffer
```

**Visual Test**: Large scene (1000+ objects) renders smoothly with low CPU usage

**Performance Goal**: Support 10,000+ objects with <1ms CPU overhead

### Step 7: Bindless Rendering (Descriptor Indexing)

**Goal**: Eliminate descriptor set switching for massive performance in material-heavy scenes.

**Theory** (4-5 hours):
1. Study bindless concepts:
   - Descriptor indexing (Vulkan 1.2+)
   - Large descriptor arrays (unbounded arrays)
   - Push constants for material indices
   - Bindless textures and buffers
   - Compatibility requirements

2. Understand benefits:
   - Eliminate per-draw descriptor set bindings
   - Support thousands of unique materials
   - Reduce draw call overhead
   - Enable GPU-driven material selection

3. Read resources:
   - Vulkan descriptor indexing extension
   - [A Comparison of Modern Graphics APIs (Wihlidal)](https://www.youtube.com/watch?v=qx20xFQXClE)
   - Study `VK_EXT_descriptor_indexing` requirements

**Key Concepts**:
- **Descriptor Array**: Single large array of textures/buffers
- **Dynamic Indexing**: Select descriptor using shader variable
- **Push Constants**: Pass material/texture indices per draw
- **Partially Bound**: Not all descriptors need valid resources
- **Update After Bind**: Modify descriptors without rebuilding sets

**Theory vs Traditional**:
```
Traditional:
  For each material:
    Bind descriptor set (material textures)
    Draw meshes with this material
    
Bindless:
  Bind single descriptor set (all textures)
  For each mesh:
    Push material index
    Draw (GPU selects textures from index)
```

**Practice** (10-14 hours):
1. Check device support for descriptor indexing
2. Create large descriptor array (sampler2D textures[1024])
3. Modify shader to use dynamic indexing:
   ```glsl
   layout(push_constant) uniform PushConstants {
       uint material_index;
   };
   
   layout(set = 0, binding = 0) uniform sampler2D textures[];
   
   void main() {
       MaterialData mat = materials[material_index];
       vec4 albedo = texture(textures[mat.albedo_index], uv);
   }
   ```
4. Update render loop to use push constants instead of rebinding
5. Profile draw call overhead reduction
6. Integrate with GPU culling for fully GPU-driven pipeline

**Implementation Steps**:
1. Enable Vulkan features:
   ```rust
   VkPhysicalDeviceDescriptorIndexingFeatures {
       descriptorBindingPartiallyBound: true,
       runtimeDescriptorArray: true,
       descriptorBindingVariableDescriptorCount: true,
       shaderSampledImageArrayNonUniformIndexing: true,
   }
   ```
2. Create descriptor set layout with variable count
3. Allocate large descriptor pool
4. Populate texture array with all loaded textures
5. Modify shaders to use indexed texture access
6. Replace descriptor set bindings with push constants

**Exercises**:
1. Convert material system to use texture indices
2. Implement material data buffer (properties array)
3. Add automatic texture index allocation
4. Profile: traditional vs bindless (measure vkCmdBindDescriptorSets calls)
5. Support texture atlas with bindless + array layers
6. Implement bindless uniform buffers (materials buffer)
7. Add hot-reloading of bindless textures

**Advanced Integration**:
- Combine with GPU culling for fully GPU-driven pipeline
- Use for massive open-world streaming
- Implement virtual texture system with bindless
- Support ray tracing with bindless resources

**Compatibility Notes**:
- Requires Vulkan 1.2 or VK_EXT_descriptor_indexing
- Check for `shaderSampledImageArrayNonUniformIndexing`
- Fallback path for older hardware
- Praxis currently uses traditional descriptors (future feature)

**Code Example** (conceptual):
```rust
// Traditional (current Praxis)
for material in materials {
    cmd_buffer.bind_descriptor_set(material.descriptor_set);
    for mesh in material.meshes {
        cmd_buffer.draw(mesh);
    }
}

// Bindless (future)
cmd_buffer.bind_descriptor_set(global_bindless_set);
for mesh in visible_meshes {
    cmd_buffer.push_constants(mesh.material_index);
    cmd_buffer.draw(mesh);
}
```

**Visual Test**: Scene with 100+ unique materials renders without constant rebinding

**Performance Goal**: Eliminate descriptor set binding overhead, support 1000+ materials

### Step 8: Complete Features Integration

**Goal**: Combine all advanced techniques into a production-ready renderer.

**Practice** (12-16 hours):
1. Run the complete demo:
   ```bash
   cargo run --example complete_features_demo
   ```
2. Study how systems integrate:
   - TAA + SSR: Temporal stability for reflections
   - SSR + Environment Probes: Fallback for off-screen reflections
   - GPU Culling + Bindless: Fully GPU-driven pipeline
   - Deferred + TAA + SSR + SSAO: Full G-buffer utilization

**Integration Exercises**:
1. **TAA + SSR Integration**:
   - Apply TAA to SSR output to reduce noise
   - Share motion vectors between systems
   - Implement consistent jitter across passes

2. **GPU Culling + Bindless**:
   - Cull objects on GPU
   - Draw with bindless materials (zero descriptor switching)
   - Measure combined performance improvement

3. **Complete Pipeline**:
   - Implement full frame breakdown:
     1. GPU culling compute pass
     2. Deferred G-buffer pass (TAA jittered)
     3. SSAO pass
     4. Lighting pass
     5. SSR pass
     6. TAA resolve pass
     7. Post-processing
   - Profile each pass
   - Optimize bottlenecks

4. **Feature Toggles**:
   - Implement runtime toggles (like complete_features_demo)
   - Measure individual feature costs
   - Test combinations for quality/performance tradeoffs

**Performance Profiling**:
```bash
cargo run --example complete_features_demo
# Press 1-4 to toggle features
# Press T for terrain stats
# Press N for network stats
```

**Expected Performance Targets** (1080p, RTX 3070):
- TAA: ~1.5ms
- SSR: ~4ms (with Hi-Z)
- GPU Culling: <0.5ms CPU, ~0.3ms GPU (1000 objects)
- Bindless: 2-3x draw call throughput
- **Total**: 60+ FPS with all features enabled

**Optimization Checklist**:
- [ ] TAA uses efficient neighborhood clamping
- [ ] SSR uses Hi-Z mipmap acceleration
- [ ] GPU culling runs in async compute (if available)
- [ ] Bindless eliminates descriptor set switching
- [ ] All passes use optimal memory layouts
- [ ] Temporal buffers reused efficiently
- [ ] No GPU-CPU sync points (besides present)

**Visual Quality Checklist**:
- [ ] TAA eliminates aliasing without excessive blur
- [ ] SSR shows accurate reflections with smooth falloff
- [ ] No visible culling artifacts
- [ ] Materials render correctly with bindless indexing
- [ ] Temporal stability across all effects

### Step 9: Advanced Effects

**Specialized Topics** (choose based on interest):

**Option A: Particle Systems**
1. Read [Particles Guide](../guides/particles.md)
2. Study `crates/praxis_graphics/PARTICLES.md`
3. Implement GPU particle simulation
4. Run example:
   ```bash
   cargo run --example particles_demo
   ```

**Option B: Terrain Rendering**
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
- [ ] Implemented temporal anti-aliasing (TAA)
- [ ] Implemented screen-space reflections (SSR)
- [ ] Understand GPU culling and indirect drawing
- [ ] Familiar with bindless rendering concepts
- [ ] Can integrate multiple advanced techniques
- [ ] Can profile and optimize rendering
- [ ] Understand modern GPU-driven pipelines

**Capstone Project**: Choose one:

1. **Modern Renderer**: Build a complete modern renderer with TAA, SSR, GPU culling, and bindless materials
2. **GPU-Driven Pipeline**: Implement fully GPU-driven rendering (culling + indirect + bindless)
3. **Custom Renderer**: Build a specialized renderer (e.g., stylized, retro)
4. **Visual Effect**: Implement a complex effect (volumetric fog, water, etc.)
5. **Optimization**: Profile and optimize existing renderer (2x FPS improvement)

**Time to Complete**: 60-80 hours

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
cargo run --example complete_features_demo    # TAA, SSR, GPU culling showcase
cargo run --example gpu_culling_demo         # GPU culling and indirect drawing
cargo run --example particles_demo
cargo run --example procedural_texture_demo
```

### External Resources

**General Graphics**:
- Vulkan Tutorial: https://vulkan-tutorial.com/
- Learn OpenGL (concepts apply): https://learnopengl.com/
- Real-Time Rendering book (4th edition)
- GPU Gems series

**Advanced Techniques**:
- [High Quality Temporal Supersampling (Epic Games)](http://advances.realtimerendering.com/s2014/epic/TemporalAA.pptx) - TAA implementation
- [GPU-Driven Rendering Pipelines (Siggraph 2015)](http://advances.realtimerendering.com/s2015/aaltonenhaar_siggraph2015_combined_final_footer_220dpi.pdf) - GPU culling
- [Stochastic Screen-Space Reflections (Siggraph 2015)](http://advances.realtimerendering.com/s2015/Stochastic%20Screen-Space%20Reflections.pptx) - SSR techniques
- [Bindless Texturing for Deferred Rendering](https://www.gdcvault.com/play/1020791/) - Descriptor indexing
- [A Comparison of Modern Graphics APIs](https://www.youtube.com/watch?v=qx20xFQXClE) - Vulkan features overview

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
