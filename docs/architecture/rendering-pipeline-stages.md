# Rendering Pipeline Stages

This document provides a visual breakdown of the Praxis rendering pipeline, showing the complete flow from scene data to final frame presentation, including both forward and deferred rendering paths.

## Overview

The Praxis graphics subsystem supports two primary rendering pipelines:

1. **Forward Rendering**: Single-pass rendering with immediate lighting
2. **Deferred Rendering**: Multi-pass rendering with G-buffer

Both pipelines share common preprocessing stages and post-processing effects.

## Complete Pipeline Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        FRAME START                                       │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Frame Timing & Synchronization                                │    │
│  │  - Calculate delta time                                        │    │
│  │  - Wait for previous frame GPU completion                      │    │
│  │  - Advance dynamic buffer frame index                          │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
└─────────────────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────────────────┐
│                        CPU PREPROCESSING                                 │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  1. ECS Systems Update                                         │    │
│  │     - Transform propagation (parent → child)                   │    │
│  │     - Animation system (skeletal, blending)                    │    │
│  │     - Physics sync (Rapier → ECS)                             │    │
│  │     - Camera updates (view/projection matrices)                │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  2. Spatial Culling                                            │    │
│  │     - Extract camera frustum                                   │    │
│  │     - Octree/BVH traversal                                     │    │
│  │     - Test entity bounds against frustum                       │    │
│  │     - Mark visible entities                                    │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  3. Lighting Data Collection                                   │    │
│  │     - Query all light components                               │    │
│  │     - Build directional light array (max 4)                    │    │
│  │     - Build point light array (max 16)                         │    │
│  │     - Calculate shadow cascade splits                          │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  4. Draw Command Generation                                    │    │
│  │     - Query visible entities with (Mesh, Material, Transform)  │    │
│  │     - Create DrawCommand for each entity                       │    │
│  │     - Sort by material (minimize state changes)                │    │
│  │     - Sort by depth (front-to-back for early-Z)               │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  5. Uniform Buffer Updates                                     │    │
│  │     - Write view/projection matrices                           │    │
│  │     - Write lighting data                                      │    │
│  │     - Write model matrices to dynamic buffer                   │    │
│  │     - Update material properties                               │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
└─────────────────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────────────────┐
│                        GPU SETUP                                         │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Swapchain Management                                          │    │
│  │     - Acquire next swapchain image                             │    │
│  │     - Handle out-of-date swapchain (resize)                    │    │
│  │     - Get image index and acquire semaphore                    │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Command Buffer Allocation                                     │    │
│  │     - Allocate primary command buffer                          │    │
│  │     - Begin command buffer recording                           │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
└─────────────────────────────────────────────────────────────────────────┘
                                ↓
                    ┌───────────────────────┐
                    │   Pipeline Selection   │
                    └───────────┬───────────┘
                                │
                ┌───────────────┴────────────────┐
                │                                │
                ▼                                ▼
    ┌──────────────────────┐        ┌──────────────────────┐
    │  FORWARD RENDERING   │        │ DEFERRED RENDERING   │
    └──────────┬───────────┘        └──────────┬───────────┘
               │                               │
               ▼                               ▼
```

## Forward Rendering Pipeline

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     FORWARD RENDERING PATH                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Shadow Pass (Optional)                                        │    │
│  │                                                                 │    │
│  │  For each cascade (typically 3-4):                            │    │
│  │    1. Bind shadow framebuffer (2048×2048 depth texture)       │    │
│  │    2. Set viewport and depth bias                             │    │
│  │    3. Bind shadow pipeline (depth-only)                       │    │
│  │    4. For each draw command:                                   │    │
│  │       - Bind vertex/index buffers                             │    │
│  │       - Set cascade light matrix                              │    │
│  │       - Draw indexed (depth write only)                       │    │
│  │    5. Transition shadow map for shader reading                │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Main Color Pass                                               │    │
│  │                                                                 │    │
│  │  Begin Render Pass:                                            │    │
│  │    - Clear color: (0.0, 0.0, 0.0, 1.0)                        │    │
│  │    - Clear depth: 1.0                                          │    │
│  │    - Load shadow maps (from previous pass)                    │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Pipeline Binding                                              │    │
│  │    - Bind forward graphics pipeline                            │    │
│  │    - Set viewport and scissor                                  │    │
│  │    - Bind global descriptor set (set 0):                       │    │
│  │      • View/Projection matrices                                │    │
│  │      • Lighting uniform buffer                                 │    │
│  │      • Shadow maps (sampler2DArray)                            │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Draw Commands (Sorted by Material)                           │    │
│  │                                                                 │    │
│  │  For each draw command:                                        │    │
│  │    1. Bind vertex buffer (positions, normals, UVs, tangents)  │    │
│  │    2. Bind index buffer                                        │    │
│  │    3. Bind per-object descriptor set (set 1):                 │    │
│  │       - Model matrix (dynamic offset)                         │    │
│  │    4. Bind material descriptor set (set 2) if changed:        │    │
│  │       - Albedo texture                                         │    │
│  │       - Normal map                                             │    │
│  │       - Metallic/Roughness texture                            │    │
│  │       - Material properties                                    │    │
│  │    5. Draw indexed (index count, 1 instance)                  │    │
│  │                                                                 │    │
│  │  Vertex Shader:                                                │    │
│  │    - Transform vertex to clip space                           │    │
│  │    - Calculate world position                                  │    │
│  │    - Transform normal to world space                          │    │
│  │    - Calculate TBN matrix for normal mapping                  │    │
│  │    - Pass interpolated data to fragment shader                │    │
│  │                                                                 │    │
│  │  Fragment Shader:                                              │    │
│  │    - Sample all material textures                             │    │
│  │    - Calculate normal from normal map                         │    │
│  │    - Compute view direction                                    │    │
│  │    - Accumulate lighting:                                      │    │
│  │      • Ambient term                                            │    │
│  │      • For each directional light:                            │    │
│  │        - Calculate Cook-Torrance BRDF                         │    │
│  │        - Sample shadow map with PCF                           │    │
│  │        - Apply shadow factor                                   │    │
│  │      • For each point light:                                   │    │
│  │        - Calculate distance attenuation                       │    │
│  │        - Calculate BRDF contribution                          │    │
│  │    - Add emissive term                                         │    │
│  │    - Output final color (HDR, linear space)                   │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Transparent Objects (Optional)                                │    │
│  │    - Sort back-to-front by distance                           │    │
│  │    - Disable depth writes                                      │    │
│  │    - Enable alpha blending                                     │    │
│  │    - Render with same pipeline                                │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  End Render Pass                                                         │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## Deferred Rendering Pipeline

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     DEFERRED RENDERING PATH                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Pass 1: Geometry Buffer (G-Buffer) Pass                      │    │
│  │                                                                 │    │
│  │  Begin Render Pass (MRT - Multiple Render Targets):           │    │
│  │    Attachment 0: Albedo (R8G8B8A8_UNORM)                      │    │
│  │    Attachment 1: Normal (R16G16B16A16_SFLOAT)                 │    │
│  │    Attachment 2: Metallic/Roughness (R8G8B8A8_UNORM)          │    │
│  │    Attachment 3: Depth (D32_SFLOAT)                           │    │
│  │                                                                 │    │
│  │  Clear all attachments                                         │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Pipeline Binding                                              │    │
│  │    - Bind geometry pipeline                                    │    │
│  │    - Set viewport and scissor                                  │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Draw Commands (Sorted by Material)                           │    │
│  │                                                                 │    │
│  │  For each opaque object:                                       │    │
│  │    1. Bind vertex/index buffers                               │    │
│  │    2. Bind descriptor sets (transform, material)              │    │
│  │    3. Draw indexed                                             │    │
│  │                                                                 │    │
│  │  Vertex Shader:                                                │    │
│  │    - Transform to clip space                                   │    │
│  │    - Pass world position and normal                           │    │
│  │    - Pass UVs and TBN matrix                                  │    │
│  │                                                                 │    │
│  │  Fragment Shader:                                              │    │
│  │    - Sample material textures                                  │    │
│  │    - Output to G-Buffer:                                       │    │
│  │      • out_albedo = albedo texture                            │    │
│  │      • out_normal = normalize(normal) * 0.5 + 0.5 (encode)   │    │
│  │      • out_metallic_rough = (metallic, roughness, emissive)   │    │
│  │    - Depth written automatically                              │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  End Render Pass                                                         │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Pass 2: Lighting Pass (Full-Screen)                          │    │
│  │                                                                 │    │
│  │  Begin Render Pass:                                            │    │
│  │    Target: HDR color buffer (R16G16B16A16_SFLOAT)             │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Pipeline Binding                                              │    │
│  │    - Bind lighting pipeline                                    │    │
│  │    - Bind G-Buffer textures as inputs (set 0):                │    │
│  │      • Albedo sampler                                          │    │
│  │      • Normal sampler                                          │    │
│  │      • Metallic/Roughness sampler                             │    │
│  │      • Depth sampler                                           │    │
│  │    - Bind lighting uniforms (set 1):                          │    │
│  │      • View/Projection matrices (for position reconstruction)  │    │
│  │      • Light array (directional + point)                      │    │
│  │      • Shadow maps                                             │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Full-Screen Quad Rendering                                   │    │
│  │                                                                 │    │
│  │  Vertex Shader:                                                │    │
│  │    - Generate full-screen triangle (no vertex buffer)         │    │
│  │    - Pass UV coordinates                                       │    │
│  │                                                                 │    │
│  │  Fragment Shader (runs for every visible pixel):              │    │
│  │    1. Sample G-Buffer at current UV                           │    │
│  │    2. Decode normal (from [0,1] to [-1,1])                    │    │
│  │    3. Reconstruct world position from depth:                  │    │
│  │       - UV + depth → clip space                               │    │
│  │       - inverse(proj) → view space                            │    │
│  │       - inverse(view) → world space                           │    │
│  │    4. Calculate view direction                                 │    │
│  │    5. Accumulate lighting:                                     │    │
│  │       - Start with ambient                                     │    │
│  │       - For each directional light:                           │    │
│  │         • Calculate Cook-Torrance BRDF                        │    │
│  │         • Sample shadow map                                    │    │
│  │         • Accumulate contribution                             │    │
│  │       - For each point light:                                  │    │
│  │         • Calculate attenuation                               │    │
│  │         • Calculate BRDF                                       │    │
│  │         • Accumulate contribution                             │    │
│  │    6. Add emissive                                             │    │
│  │    7. Output lit color (HDR)                                  │    │
│  │                                                                 │    │
│  │  Draw call: Single indexed draw (6 vertices = 2 triangles)    │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  End Render Pass                                                         │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Pass 3: Forward Pass for Transparency (Optional)             │    │
│  │    - Render transparent objects over lit scene                │    │
│  │    - Use forward pipeline                                      │    │
│  │    - Alpha blending enabled                                    │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## Post-Processing Pipeline

Both forward and deferred paths converge at post-processing:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     POST-PROCESSING STAGES                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Input: HDR Color Buffer (R16G16B16A16_SFLOAT)                         │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  1. Bloom (Optional)                                           │    │
│  │                                                                 │    │
│  │  a) Bright Pass:                                               │    │
│  │     - Extract bright pixels (luminance > threshold)            │    │
│  │     - Output to half-resolution buffer                         │    │
│  │                                                                 │    │
│  │  b) Gaussian Blur (Multiple Passes):                          │    │
│  │     - Horizontal blur pass                                     │    │
│  │     - Vertical blur pass                                       │    │
│  │     - Repeat with downsampling (5+ levels)                    │    │
│  │                                                                 │    │
│  │  c) Upsampling & Combination:                                 │    │
│  │     - Upsample blur pyramids                                   │    │
│  │     - Combine with original image                             │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  2. Tone Mapping                                               │    │
│  │                                                                 │    │
│  │  HDR → LDR conversion (full-screen pass):                     │    │
│  │                                                                 │    │
│  │  Operator Selection:                                           │    │
│  │    - ACES (cinematic, default)                                │    │
│  │    - Reinhard (simple, fast)                                  │    │
│  │    - Uncharted 2 (game-style)                                 │    │
│  │                                                                 │    │
│  │  Process:                                                      │    │
│  │    1. Apply exposure adjustment                               │    │
│  │    2. Apply tone mapping curve                                │    │
│  │    3. Output to LDR buffer (R8G8B8A8_UNORM)                  │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  3. Gamma Correction                                           │    │
│  │                                                                 │    │
│  │    - Apply gamma 2.2 (sRGB approximation)                     │    │
│  │    - color_out = pow(color_in, 1.0/2.2)                       │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  4. Additional Effects (Optional)                              │    │
│  │                                                                 │    │
│  │    - Color grading (LUT-based)                                │    │
│  │    - Vignette                                                  │    │
│  │    - Chromatic aberration                                      │    │
│  │    - Film grain                                                │    │
│  │    - Screen-space reflections (SSR)                           │    │
│  │    - Temporal anti-aliasing (TAA)                             │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  5. UI/Debug Overlay                                           │    │
│  │                                                                 │    │
│  │    - Render GUI with egui                                      │    │
│  │    - Render debug lines/gizmos                                │    │
│  │    - Render text overlays                                      │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  Final Output: Swapchain Image (R8G8B8A8_SRGB)                        │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## Frame Finalization

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        FRAME SUBMISSION                                  │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Command Buffer Submission                                     │    │
│  │                                                                 │    │
│  │  1. End command buffer recording                               │    │
│  │  2. Create submission:                                         │    │
│  │     - Wait on: Image acquire semaphore                        │    │
│  │     - Execute: Command buffer                                  │    │
│  │     - Signal: Render complete semaphore                       │    │
│  │  3. Submit to graphics queue                                   │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Presentation                                                  │    │
│  │                                                                 │    │
│  │  1. Create present info:                                       │    │
│  │     - Wait on: Render complete semaphore                      │    │
│  │     - Present: Swapchain image at index                       │    │
│  │  2. Submit to present queue                                    │    │
│  │  3. Handle swapchain status:                                   │    │
│  │     - Success: Frame displayed                                 │    │
│  │     - Suboptimal: Mark for recreation                         │    │
│  │     - Out of date: Recreate immediately                       │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                               ↓                                          │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Frame Cleanup                                                 │    │
│  │                                                                 │    │
│  │  1. Store fence for GPU synchronization                       │    │
│  │  2. Request next frame redraw                                  │    │
│  │  3. Update frame statistics                                    │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## Performance Characteristics

### Forward Rendering

**Time Complexity**: O(lights × drawn_fragments)

**Advantages**:
- Simple pipeline
- Natural transparency support
- Low memory usage
- Good for few lights (< 10)

**Disadvantages**:
- Poor light scaling
- Overdraw waste on lighting
- All lights processed for all fragments

### Deferred Rendering

**Time Complexity**: O(triangles + lights × visible_pixels)

**Advantages**:
- Excellent light scaling (10+ lights)
- No overdraw waste
- Decoupled geometry and lighting
- Easy to add advanced effects

**Disadvantages**:
- High memory usage (G-Buffer ~40-80 MB @ 1080p)
- Transparency requires forward pass
- MSAA expensive
- Higher bandwidth requirements

## Pipeline Selection Guide

### Choose Forward When:
- Few lights (< 5-10)
- Heavy use of transparency
- Limited VRAM
- Mobile/low-end platforms
- MSAA required

### Choose Deferred When:
- Many lights (10+)
- Mostly opaque geometry
- Complex lighting effects
- High-end platforms with VRAM
- Advanced post-processing pipeline

## Optimization Opportunities

### CPU Side
1. **Material Batching**: Sort draw commands by material to minimize state changes
2. **Frustum Culling**: Use spatial structures (octree/BVH) to cull invisible objects
3. **LOD System**: Select appropriate mesh detail based on distance
4. **Parallel Command Recording**: Record secondary command buffers in parallel

### GPU Side
1. **Early-Z Testing**: Sort opaque objects front-to-back
2. **Descriptor Set Reuse**: Cache and reuse descriptor sets for identical materials
3. **Dynamic Uniform Buffers**: Use dynamic offsets to reduce descriptor set allocations
4. **GPU Culling**: Compute-shader based frustum culling on GPU
5. **Indirect Drawing**: Use multi-draw-indirect for reduced CPU overhead

### Memory
1. **Texture Compression**: Use BC/ASTC formats
2. **Mesh Streaming**: Load/unload meshes based on visibility
3. **Texture Atlasing**: Combine small textures to reduce draw calls
4. **Buffer Pooling**: Reuse staging buffers across frames

## Related Documentation

- [Render Pipeline Concepts](render-pipeline.md) - Detailed pipeline explanation
- [Vulkan Rendering](../concepts/vulkan-rendering.md) - Vulkan fundamentals
- [Forward Rendering Guide](../guides/rendering/forward-rendering.md) - Forward pipeline guide
- [Deferred Rendering Guide](../guides/rendering/deferred-rendering.md) - Deferred pipeline guide
- [HDR and Tone Mapping](../guides/rendering/hdr-tonemapping.md) - Post-processing
- [Shadows Guide](../guides/rendering/shadows.md) - Shadow mapping
- [Spatial Optimization](../guides/spatial-optimization.md) - Culling and LOD
- [Rendering Learning Path](../learning-paths/rendering.md) - Progressive learning
