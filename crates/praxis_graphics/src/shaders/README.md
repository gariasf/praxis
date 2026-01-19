# Praxis Graphics Shaders

This directory contains all GLSL shaders used by the Praxis graphics system. Each shader is documented with its purpose, inputs/outputs, and integration points.

## Shader Categories

### Core Rendering

#### `triangle.vert` / `triangle.frag`
**Purpose**: Basic 3D rendering with PBR materials and lighting.

**Inputs**:
- Vertex attributes: Position, normal, color, UV, tangent
- Set 0, Binding 0: View/projection matrices
- Set 0, Binding 1: Model matrices (dynamic uniform buffer)
- Set 0, Binding 2: Albedo texture
- Set 0, Binding 3: Lighting data (directional and point lights)
- Set 0, Binding 4: Shadow data
- Set 0, Bindings 5-8: Shadow map samplers (cascaded)
- Set 0, Binding 9: Normal map texture
- Set 0, Binding 10: Bone matrices (skeletal animation)
- Set 1, Binding 0: Material properties (metallic, roughness, emissive)

**Outputs**:
- Fragment color with lighting, shadows, and material properties applied

**Integration**: Primary shader for forward rendering pipeline. Supports normal mapping with TBN matrix, PBR materials, multiple light sources, cascaded shadow maps, and skeletal animation.

---

### Deferred Rendering

#### `deferred_geometry.vert` / `deferred_geometry.frag`
**Purpose**: Geometry pass for deferred rendering, writes to G-buffer.

**Inputs**:
- Vertex attributes: Position, normal, color, UV, tangent
- Set 0, Binding 0: View/projection matrices
- Set 0, Binding 1: Model matrices
- Set 0, Binding 2: Albedo texture
- Set 0, Binding 9: Normal map
- Set 1, Binding 0: Material properties

**Outputs**:
- Attachment 0: Albedo (RGB) + Alpha
- Attachment 1: World-space normal (RGB)
- Attachment 2: Metallic (R) + Roughness (G)
- Depth attachment: Scene depth

**Integration**: First pass in deferred rendering pipeline. Outputs material and geometry data to G-buffer textures for subsequent lighting pass. Efficient for scenes with many lights.

#### `deferred_lighting.vert` / `deferred_lighting.frag`
**Purpose**: Lighting pass for deferred rendering, reads G-buffer and computes lighting.

**Inputs**:
- Set 0, Binding 0: G-buffer albedo texture
- Set 0, Binding 1: G-buffer normal texture
- Set 0, Binding 2: G-buffer metallic-roughness texture
- Set 0, Binding 3: G-buffer depth texture
- Set 0, Binding 4: Lighting data uniform
- Set 0, Binding 5: Shadow data
- Set 0, Bindings 6-9: Shadow map samplers

**Outputs**:
- Fragment color with accumulated lighting from all light sources

**Integration**: Second pass in deferred rendering pipeline. Runs as full-screen quad, reconstructs world positions from depth, and accumulates lighting contributions.

---

### Shadow Mapping

#### `shadow.vert` / `shadow.frag`
**Purpose**: Generate shadow maps from light's perspective.

**Inputs**:
- Vertex attributes: Position
- Set 0, Binding 0: Light view-projection matrix
- Set 0, Binding 1: Model matrices

**Outputs**:
- Depth only (written to shadow map texture)

**Integration**: Shadow map generation pass. Runs for each cascade in cascaded shadow maps. Fragment shader is minimal (depth-only rendering).

---

### Skybox

#### `skybox.vert` / `skybox.frag`
**Purpose**: Render skybox using cubemap texture.

**Inputs**:
- Vertex attributes: Position
- Set 0, Binding 0: View/projection matrices
- Set 0, Binding 1: Cubemap sampler

**Outputs**:
- Skybox color sampled from cubemap

**Integration**: Rendered last with depth test set to LESS_OR_EQUAL and camera-centered positioning. Uses reversed depth buffer for infinite distance.

---

### Environment Probes / IBL

#### `ibl_irradiance.vert` / `ibl_irradiance.frag`
**Purpose**: Generate diffuse irradiance map from environment cubemap.

**Inputs**:
- Vertex attributes: Position
- Set 0, Binding 0: Projection matrix
- Set 0, Binding 1: Source environment cubemap

**Outputs**:
- Irradiance cubemap (32x32 per face, convolved for diffuse lighting)

**Integration**: Pre-processing step for IBL. Runs once per environment map to generate irradiance map used for ambient diffuse lighting. Uses hemisphere sampling with cosine weighting.

#### `ibl_prefilter.vert` / `ibl_prefilter.frag`
**Purpose**: Generate specular reflection prefiltered environment map.

**Inputs**:
- Vertex attributes: Position
- Set 0, Binding 0: Projection matrix
- Set 0, Binding 1: Source environment cubemap
- Push constant: Roughness level

**Outputs**:
- Prefiltered cubemap with multiple roughness levels (5 mip levels)

**Integration**: Pre-processing step for IBL specular reflections. Runs once per roughness level using importance sampling with GGX distribution. Uses 1024 samples per pixel for quality.

#### `equirect_to_cube.vert` / `equirect_to_cube.frag`
**Purpose**: Convert equirectangular HDR image to cubemap.

**Inputs**:
- Vertex attributes: Position
- Set 0, Binding 0: Projection matrix
- Set 0, Binding 1: Equirectangular texture sampler

**Outputs**:
- Cubemap texture (6 faces)

**Integration**: Asset loading utility. Converts panoramic HDR images to cubemap format for use as environment maps. Uses spherical coordinate mapping.

---

### Screen-Space Ambient Occlusion (SSAO)

#### `ssao.vert` / `ssao.frag`
**Purpose**: Compute screen-space ambient occlusion.

**Inputs**:
- Set 0, Binding 0: Depth texture
- Set 0, Binding 1: Normal texture (view-space)
- Set 0, Binding 2: Noise texture (4x4 tiling)
- Set 0, Binding 3: SSAO parameters (sample kernel, radius, bias)
- Set 0, Binding 4: Projection matrix

**Outputs**:
- Occlusion factor (R channel, 0 = fully occluded, 1 = no occlusion)

**Integration**: Post-geometry pass effect. Samples hemisphere around each pixel in view space using rotating noise pattern. Output is blurred before application.

#### `ssao_blur.vert` / `ssao_blur.frag`
**Purpose**: Blur SSAO output to reduce noise artifacts.

**Inputs**:
- Set 0, Binding 0: Raw SSAO texture
- Set 0, Binding 1: Depth texture (for depth-aware filtering)

**Outputs**:
- Blurred occlusion factor

**Integration**: Applied after SSAO generation. Uses box blur with depth awareness to preserve edges. Output multiplied with lighting in final composition.

---

### Screen-Space Reflections (SSR)

#### `ssr.vert` / `ssr.frag`
**Purpose**: Compute screen-space reflections using hierarchical ray marching.

**Inputs**:
- Set 0, Binding 0: G-buffer normal texture
- Set 0, Binding 1: G-buffer depth texture
- Set 0, Binding 2: G-buffer metallic-roughness texture
- Set 0, Binding 3: Scene color texture
- Set 0, Binding 4: SSR parameters (max steps, step size, thickness, roughness threshold)

**Outputs**:
- RGB: Reflection color, A: Confidence (0-1, based on hit quality)

**Integration**: Post-lighting effect for metallic surfaces. Traces reflection rays through depth buffer with binary search refinement. Low confidence areas fall back to environment probes in composite pass.

#### `ssr_blur.vert` / `ssr_blur.frag`
**Purpose**: Apply roughness-aware blur to SSR reflections.

**Inputs**:
- Set 0, Binding 0: SSR reflection texture
- Set 0, Binding 1: G-buffer metallic-roughness texture
- Push constant: Texel size, blur direction

**Outputs**:
- Blurred reflection texture

**Integration**: Applied after SSR generation in two separable passes (horizontal then vertical). Blur radius scales with surface roughness.

#### `ssr_composite.vert` / `ssr_composite.frag`
**Purpose**: Blend SSR reflections with environment probe fallback.

**Inputs**:
- Set 0, Binding 0: SSR texture (RGB + confidence)
- Set 0, Binding 1: Environment cubemap
- Set 0, Binding 2: G-buffer normal texture
- Set 0, Binding 3: G-buffer metallic-roughness texture

**Outputs**:
- Final reflection color with Fresnel effect applied

**Integration**: Final SSR pass. Blends screen-space reflections with environment probe based on confidence. High-confidence SSR used directly, low-confidence blends with environment map.

---

### Temporal Anti-Aliasing (TAA)

#### `taa.vert` / `taa.frag`
**Purpose**: Temporal anti-aliasing using velocity-based reprojection.

**Inputs**:
- Set 0, Binding 0: Current frame texture
- Set 0, Binding 1: History frame texture
- Set 0, Binding 2: Velocity buffer texture
- Set 0, Binding 3: Depth buffer texture
- Set 0, Binding 4: TAA config (jitter offset, blend factor)

**Outputs**:
- Temporally accumulated anti-aliased image

**Integration**: Applied after main rendering. Uses velocity buffer for reprojection, YCoCg color space for better clamping, and neighborhood AABB clamping for history rejection. Adaptive blend factor based on velocity magnitude.

---

### HDR Tone Mapping

#### `hdr_tone_map.frag`
**Purpose**: Convert HDR rendering to LDR with tone mapping.

**Inputs**:
- Set 0, Binding 0: HDR color texture
- Set 0, Binding 1: Tone mapping parameters (operator, exposure, gamma)

**Outputs**:
- LDR color [0,1] with gamma correction applied

**Integration**: Applied to final HDR render target. Supports multiple operators: ACES, Reinhard, Uncharted 2. Includes automatic and manual exposure control. Converts from linear color space to sRGB with gamma correction (2.2).

---

### Post-Processing Base

#### `post_process.vert`
**Purpose**: Shared vertex shader for full-screen post-processing effects.

**Inputs**:
- Vertex attributes: Position (vec2), UV (vec2)

**Outputs**:
- Clip-space position, UV coordinates

**Integration**: Used by all post-processing fragment shaders. Generates full-screen quad covering [-1,1] NDC space.

---

### Post-Processing Effects

#### `post_process_grayscale.frag`
**Purpose**: Convert image to grayscale.

**Inputs**:
- Set 0, Binding 0: Color texture

**Outputs**:
- Grayscale image using perceptually accurate luminance weights (0.299R, 0.587G, 0.114B)

**Integration**: Simple color conversion effect. Can be chained with other post-processing.

#### `post_process_brightness_extract.frag`
**Purpose**: Extract bright pixels above threshold for bloom.

**Inputs**:
- Set 0, Binding 0: HDR color texture
- Push constant: Brightness threshold

**Outputs**:
- Bright pixels only (others set to black)

**Integration**: First pass in bloom pipeline. Extracts highlights for subsequent blur passes.

#### `post_process_gaussian_blur_h.frag` / `post_process_gaussian_blur_v.frag`
**Purpose**: Separable Gaussian blur (horizontal and vertical passes).

**Inputs**:
- Set 0, Binding 0: Input texture
- Push constant: Blur kernel size, texel size

**Outputs**:
- Blurred image

**Integration**: Used for bloom, depth-of-field, and other blur effects. Two-pass approach (horizontal then vertical) is more efficient than 2D blur. Configurable kernel size.

#### `post_process_blur.frag`
**Purpose**: Simple 9-tap box blur.

**Inputs**:
- Set 0, Binding 0: Input texture
- Push constant: Texel size, blur radius

**Outputs**:
- Blurred image (3x3 kernel)

**Integration**: Fast blur for quick approximations. Less quality than Gaussian but faster.

#### `post_process_copy.frag`
**Purpose**: Direct texture copy/passthrough.

**Inputs**:
- Set 0, Binding 0: Source texture

**Outputs**:
- Exact copy of input texture

**Integration**: Utility shader for render target copies and pipeline transitions.

#### `post_process_tone_map.frag`
**Purpose**: Simple Reinhard tone mapping for post-processing.

**Inputs**:
- Set 0, Binding 0: HDR color texture

**Outputs**:
- Tone-mapped color with gamma correction

**Integration**: Simplified version of main tone mapper. Used in bloom combine pass and other post effects requiring tone mapping.

---

### Cinematic Post-Processing

#### `post_process_dof.frag`
**Purpose**: Depth-of-field with bokeh blur simulation.

**Inputs**:
- Set 0, Binding 0: Color texture
- Set 0, Binding 1: Depth texture
- Set 0, Binding 2: DoF parameters (focus distance, focal length, aperture, sensor size)

**Outputs**:
- Image with depth-based blur applied

**Integration**: Cinematic effect simulating camera focus. Uses circle of confusion calculation based on camera parameters and Poisson disk sampling for bokeh shape.

#### `post_process_motion_blur.frag`
**Purpose**: Per-pixel motion blur using velocity buffer.

**Inputs**:
- Set 0, Binding 0: Color texture
- Set 0, Binding 1: Velocity buffer (screen-space motion vectors)
- Set 0, Binding 2: Motion blur parameters (sample count, shutter angle)

**Outputs**:
- Motion-blurred image

**Integration**: Requires velocity buffer from `velocity_buffer` shader. Samples along motion vectors with configurable sample count. Simulates camera shutter angle.

#### `post_process_chromatic_aberration.frag`
**Purpose**: Lens chromatic aberration (color fringing).

**Inputs**:
- Set 0, Binding 0: Color texture
- Push constant: Aberration intensity, center point

**Outputs**:
- Image with radial color separation

**Integration**: Cinematic lens distortion effect. Separates RGB channels with distance-based falloff from center. Configurable intensity.

#### `post_process_vignette.frag`
**Purpose**: Vignette darkening effect.

**Inputs**:
- Set 0, Binding 0: Color texture
- Push constant: Vignette intensity, shape (inner/outer radius), center point

**Outputs**:
- Image with edge darkening

**Integration**: Cinematic framing effect. Darkens edges with configurable shape and smoothness. Common in film-style rendering.

#### `post_process_film_grain.frag`
**Purpose**: Procedural film grain noise.

**Inputs**:
- Set 0, Binding 0: Color texture
- Push constant: Grain intensity, grain size, time (for animation)

**Outputs**:
- Image with film grain applied

**Integration**: Adds analog film texture. Procedurally generated noise scaled by luminance. Animated grain for realism.

---

### Volumetric Effects

#### `volumetric_fog.vert` / `volumetric_fog.frag`
**Purpose**: Volumetric fog with light scattering.

**Inputs**:
- Set 0, Binding 0: Fog parameters (color, density, max distance, steps, density function)
- Set 0, Binding 1: View/projection matrices
- Set 0, Binding 2: Depth texture
- Set 0, Binding 3: Scene color texture
- Set 0, Binding 4: Lighting data (directional light)

**Outputs**:
- Scene color with volumetric fog applied

**Integration**: Post-lighting effect. Ray-marches through scene using depth buffer. Supports multiple density functions: uniform, exponential distance, height-based, and noise-based. Includes phase function for anisotropic scattering.

#### `god_rays.vert` / `god_rays.frag`
**Purpose**: Volumetric god rays (light shafts) from light source.

**Inputs**:
- Set 0, Binding 0: God rays parameters (light screen position, samples, density, weight, decay, exposure, threshold)
- Set 0, Binding 1: Scene texture
- Set 0, Binding 2: Occlusion texture

**Outputs**:
- Scene color with god rays added

**Integration**: Post-processing effect. Radial blur from light source screen position. Samples along ray from pixel to light, accumulating occluded light with decay. Threshold filters bright areas.

---

### Velocity Buffer

#### `velocity_buffer.vert` / `velocity_buffer.frag`
**Purpose**: Generate per-pixel screen-space motion vectors.

**Inputs**:
- Vertex attributes: Position
- Set 0, Binding 0: Current frame MVP matrix
- Set 0, Binding 1: Previous frame MVP matrix
- Set 0, Binding 2: Model matrices

**Outputs**:
- RG: Screen-space velocity (current position - previous position)

**Integration**: Required for motion blur and TAA. Renders during geometry pass with both current and previous frame transforms. Output is 2D vector representing pixel movement.

---

### Particle System

#### `particle.vert` / `particle.frag`
**Purpose**: Render billboard particles with soft particle blending.

**Inputs**:
- Vertex attributes: Particle position, size, color, rotation, lifetime
- Set 0, Binding 0: View/projection matrices
- Set 0, Binding 1: Particle texture
- Set 0, Binding 2: Depth texture (for soft particles)
- Set 0, Binding 3: Soft particle parameters (fade distance)

**Outputs**:
- Particle color with alpha blending and soft edge fading

**Integration**: Forward rendering with alpha blending. Particles always face camera (billboard). Soft particles fade when intersecting geometry using depth buffer comparison. Per-particle rotation and color modulation.

#### `particle_update.comp`
**Purpose**: GPU-accelerated particle simulation.

**Inputs**:
- Set 0, Binding 0: Particle buffer (position, velocity, lifetime, size, color, distance)
- Set 0, Binding 1: Update parameters (delta time, forces, camera position)

**Outputs**:
- Updated particle buffer (modified in-place)

**Integration**: Compute shader run before particle rendering. Updates positions, velocities, lifetimes on GPU. Applies forces (gravity, wind, drag). Calculates camera distance for sorting. Processes up to 256 particles per workgroup.

#### `particle_emit.comp`
**Purpose**: GPU-based particle emission.

**Inputs**:
- Set 0, Binding 0: Particle buffer
- Set 0, Binding 1: Emitter parameters (position, emission rate, particle properties)
- Set 0, Binding 2: Random seed

**Outputs**:
- Particle buffer with newly emitted particles

**Integration**: Compute shader run before particle update. Uses atomic operations for thread-safe particle slot allocation. Randomizes particle properties (velocity, size, lifetime, color). Supports various emitter shapes.

#### `particle_sort.comp`
**Purpose**: GPU bitonic sort for particle alpha blending order.

**Inputs**:
- Set 0, Binding 0: Particle buffer with camera distances
- Set 0, Binding 1: Sort parameters (particle count, sort step, sort stage)

**Outputs**:
- Particle buffer sorted by camera distance (back-to-front)

**Integration**: Compute shader run after particle update, before rendering. Implements bitonic sort algorithm for parallel execution. Required for correct alpha blending. Works on power-of-two particle counts.

---

### GPU-Driven Rendering

#### `gpu_culling.comp`
**Purpose**: GPU frustum and occlusion culling for indirect rendering.

**Inputs**:
- Set 0, Binding 0: Culling uniforms (view-proj matrix, frustum planes, camera position, enable flags)
- Set 0, Binding 1: Draw commands (model matrix, bounding sphere, mesh ID, material ID)
- Set 0, Binding 2: Mesh data (index count, vertex offset)
- Set 0, Binding 3: Output indirect draw buffer (written)
- Set 0, Binding 4: Visible indices buffer (written)
- Set 0, Binding 5: Draw count atomic counter (written)
- Set 0, Binding 6: Hi-Z pyramid (for occlusion culling)

**Outputs**:
- Indirect draw commands (VkDrawIndexedIndirectCommand format)
- Visible object indices
- Total visible count

**Integration**: Compute shader run before indirect draw calls. Tests bounding spheres against frustum planes. Optional occlusion culling using Hi-Z pyramid. Atomically builds indirect draw buffer. Processes 64 objects per workgroup.

#### `hiz_generate.comp`
**Purpose**: Generate hierarchical depth buffer mipmaps for occlusion culling.

**Inputs**:
- Set 0, Binding 0: Input depth texture (previous mip level)
- Set 0, Binding 1: Output depth image (current mip level, write-only)
- Push constant: Input size, output size, mip level

**Outputs**:
- Depth mipmap with maximum depth of 2x2 block (conservative)

**Integration**: Compute shader run after depth-only pre-pass. Generates mip chain by taking maximum of 2x2 blocks. Used by `gpu_culling` for Hi-Z occlusion queries. Processes 16x16 pixels per workgroup.

#### `lod_selection.comp`
**Purpose**: GPU-driven LOD level selection based on distance.

**Inputs**:
- Set 0, Binding 0: LOD uniforms (camera position, LOD bias, object count, enable flag)
- Set 0, Binding 1: Object data (transform, bounding sphere, mesh ID, LOD metadata)
- Set 0, Binding 2: LOD level definitions (mesh ID, distance ranges)
- Set 0, Binding 3: Output selected LOD per object (written)
- Set 0, Binding 4: Output distance buffer (written)

**Outputs**:
- Selected mesh ID per object
- Camera distance squared per object

**Integration**: Compute shader run before culling. Calculates object-camera distance and selects appropriate LOD level. Supports LOD bias for quality adjustment. Processes 64 objects per workgroup.

---

### Advanced Materials

#### `advanced_material.frag`
**Purpose**: Advanced PBR with parallax occlusion mapping, clearcoat, sheen, and transmission.

**Inputs**:
- Vertex inputs: World position, normal, color, UV, tangent, bitangent
- Set 0, Binding 0: View/projection matrices
- Set 0, Binding 2: Albedo texture
- Set 0, Binding 9: Normal map
- Set 0, Binding 11: Metallic-roughness map
- Set 0, Binding 12: Height map (for parallax)
- Set 0, Binding 13: Ambient occlusion map
- Set 0, Binding 14: Emissive map
- Set 0, Binding 3: Lighting data
- Set 1, Binding 0: Base material properties
- Set 1, Binding 1: Extended PBR properties (clearcoat, sheen, transmission, IOR, anisotropy)
- Set 1, Binding 2: Parallax properties (height scale, sample counts)

**Outputs**:
- Fragment color with advanced material effects

**Integration**: Enhanced material shader for high-quality rendering. Parallax occlusion mapping provides depth perception. Clearcoat adds secondary specular layer (car paint). Sheen simulates fabric-like appearance. Transmission for translucent materials.

#### `material_layer_blend.vert` / `material_layer_blend.frag`
**Purpose**: Blend up to 4 material layers with masks and blend modes.

**Inputs**:
- Set 0, Bindings 0-2: Base layer textures (albedo, normal, metallic-roughness)
- Set 0, Bindings 3-6: Layer 1 textures + mask
- Set 0, Bindings 7-10: Layer 2 textures + mask
- Set 0, Bindings 11-14: Layer 3 textures + mask
- Set 1, Binding 0: Layer parameters (UV scale, opacity, blend mode, enabled flags)

**Outputs**:
- Blended material properties (typically rendered to texture)

**Integration**: Material pre-processing shader. Blends multiple material layers using masks. Supports blend modes: Replace, Add, Multiply, Overlay. Uses RNM (Reoriented Normal Mapping) for normal blending. Output typically feeds into main material shader.

---

### Advanced Lighting

#### `area_lights.vert` / `area_lights.frag`
**Purpose**: Area light rendering using Linearly Transformed Cosines (LTC).

**Inputs**:
- Vertex inputs: World position, normal, color, UV, tangent, bitangent
- Set 0, Binding 0: View/projection matrices
- Set 0, Binding 1: Area lights data (transforms, colors, types, parameters)
- Set 0, Binding 2: LTC matrix lookup table 1
- Set 0, Binding 3: LTC matrix lookup table 2
- Set 0, Binding 4: Albedo texture
- Set 1, Binding 0: Material properties

**Outputs**:
- Fragment color with area light contributions

**Integration**: Specialized lighting shader for rectangular and sphere area lights. Uses LTC technique for physically-based area lighting. Requires pre-computed LTC lookup tables. Supports up to 16 area lights. More realistic than point lights for large light sources.

#### `light_probe.vert` / `light_probe.frag`
**Purpose**: Render objects lit by spherical harmonic light probes.

**Inputs**:
- Vertex inputs: World position, normal, color, UV
- Set 0, Binding 0: View/projection matrices
- Set 0, Binding 1: Light probe data (position, SH coefficients up to L2, intensity, radius)
- Set 0, Binding 2: Albedo texture
- Set 1, Binding 0: Material properties

**Outputs**:
- Fragment color with probe-based lighting

**Integration**: Dynamic global illumination using pre-computed light probes. Evaluates spherical harmonics (L0, L1, L2 bands) for smooth ambient lighting. Blends multiple nearby probes weighted by distance. Supports up to 64 probes.

#### `light_linking.glsl`
**Purpose**: Helper functions for selective light-object interaction.

**Inputs**:
- Set 2, Binding 0: Light linking uniform (light mask bitmask)

**Outputs**:
- Helper functions: `can_light_affect_object(channel)`, `can_light_affect_object_mask(mask)`

**Integration**: Include file for shaders needing light linking. Allows artists to control which lights affect which objects using channel masks. Provides standard channel definitions (HERO, ENVIRONMENT, ACCENT, EFFECTS, UI).

---

### Debug/Utility

#### `line.vert` / `line.frag`
**Purpose**: Simple line rendering for debug visualization.

**Inputs**:
- Vertex attributes: Position (vec3), color (vec3)
- Set 0, Binding 0: View/projection matrices

**Outputs**:
- Solid color lines

**Integration**: Debug rendering system. Used for gizmos, wireframes, bounding boxes, physics debug visualization. No lighting or depth complexity.

---

## Shader Conventions

### Descriptor Set Organization

**Set 0: Per-frame/Per-view data**
- Binding 0: View/projection matrices
- Binding 1: Model/world matrices (may be dynamic uniform buffer)
- Binding 2+: Textures and lighting data

**Set 1: Per-material data**
- Binding 0: Material properties uniform
- Binding 1+: Material-specific parameters

**Set 2: Bindless/Advanced features**
- Binding 0: Texture array (up to 4096 textures) or light linking
- Binding 1: Material data buffer (up to 4096 materials)

### Vertex Attributes

**Standard 3D vertex format:**
- Location 0: Position (vec3)
- Location 1: Normal (vec3)
- Location 2: Color (vec3)
- Location 3: UV coordinates (vec2)
- Location 4: Tangent (vec4, w = handedness)
- Locations 5-6: Used by specific shaders (bitangent, bone indices, bone weights)

**Post-processing vertex format:**
- Location 0: Position (vec2, NDC space)
- Location 1: UV (vec2)

### Coordinate Systems

- **Right-handed coordinate system**
- **Y-up convention**
- **Depth range: [0, 1]** (Vulkan convention, reversed for better precision)
- **Cubemap faces:** +X, -X, +Y, -Y, +Z, -Z order
- **Normal map convention:** OpenGL (Y-up) tangent space
- **UV origin:** Bottom-left (0,0), top-right (1,1)

### Color Spaces

- **Input textures:** sRGB color space (automatically linearized by sampler)
- **Rendering:** Linear RGB color space throughout pipeline
- **Output:** sRGB after tone mapping and gamma correction (gamma 2.2)
- **HDR:** Linear RGB with floating-point precision

### Buffer Layouts

**std140 for uniform buffers**: 16-byte alignment for vec3 (padded to vec4)

**std430 for storage buffers**: Tighter packing, vec3 is 12 bytes

### Naming Conventions

- **Uniforms:** `CamelCase` for struct names, `snake_case` for members
- **Textures:** Descriptive names with `_texture`, `_map`, or `_buffer` suffix
- **Outputs:** Prefix with `f_` (fragment), `v_` (vertex), `out` or descriptive name
- **Functions:** `snake_case`
- **Constants:** `SCREAMING_SNAKE_CASE` or `CamelCase`

---

## Shader Compilation

Shaders are compiled at build time using `vulkano-shaders` macro:

```rust
mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/shader.vert"
    }
}
```

The compilation process:
1. Parse GLSL source files at build time
2. Compile to SPIR-V using `glslang`
3. Generate Rust bindings for layouts and specialization constants
4. Embed SPIR-V in binary

For manual shader debugging/validation:
```bash
glslangValidator -V shader.vert -o shader.vert.spv
spirv-val shader.vert.spv
```

---

## Performance Considerations

### Texture Sampling
- Use mipmaps for distant textures (reduces bandwidth and aliasing)
- Cache texture lookups when used multiple times in shader
- Use explicit LOD when possible (`textureLod`)
- Prefer anisotropic filtering for oblique viewing angles

### Branching
- Minimize dynamic branching in fragment shaders (divergent execution)
- Use `mix()` for interpolation instead of `if` when possible
- Flatten short branches with boolean arithmetic
- Early exit in compute shaders when thread has no work

### Bandwidth Optimization
- Pack data efficiently (smaller formats where possible)
- Minimize render target writes in G-buffer (use efficient formats)
- Use compressed texture formats (BC1, BC3, BC5 for normals, BC6H for HDR)
- Consider half-precision (mediump) for mobile/performance

### GPU Optimization
- Use early-Z testing (render opaque front-to-back)
- Align uniform data to 16-byte boundaries
- Use SIMD-friendly operations (vec4, mat4)
- Leverage compute shaders for parallel work (culling, sorting, simulation)

### Compute Shader Best Practices
- Choose workgroup size based on target hardware (64-256 threads typical)
- Minimize shared memory bank conflicts
- Coalesce global memory accesses
- Use barriers only when necessary
- Balance workload across threads (avoid idle threads)

---

## Adding New Shaders

When adding a new shader to the system:

1. **Create shader files** in this directory (`.vert`, `.frag`, `.comp`)
2. **Add header comment** explaining purpose, inputs, outputs
3. **Follow naming conventions** (descriptive, lowercase with underscores)
4. **Document descriptor sets** with comments in shader
5. **Update this README** with shader documentation in appropriate category
6. **Add shader loading** in corresponding Rust module
7. **Write example usage** in module documentation or examples directory
8. **Test shader** with validation layers enabled
9. **Benchmark** if performance-critical (use profiling tools)

### Shader Template

```glsl
#version 450

// Purpose: Brief description of what this shader does
// Inputs: List descriptor sets, bindings, vertex attributes
// Outputs: List output attachments or return values
// Integration: How this fits into the rendering pipeline

// Descriptor sets
layout(set = 0, binding = 0) uniform MyUniforms {
    // ...
} uniforms;

// Vertex attributes
layout(location = 0) in vec3 position;

// Outputs
layout(location = 0) out vec4 fragColor;

void main() {
    // Implementation
}
```

---

## Debugging Shaders

### Vulkan Validation Layers
Enable validation layers for detailed error messages:
```rust
let instance = Instance::new(
    library,
    InstanceCreateInfo {
        enabled_layers: vec!["VK_LAYER_KHRONOS_validation".to_owned()],
        ..Default::default()
    },
)?;
```

### RenderDoc
Use RenderDoc for frame capture and shader debugging:
1. Launch application through RenderDoc
2. Capture frame (F12 or trigger)
3. Inspect draw calls and pipeline state
4. View shader inputs/outputs per pixel
5. Examine generated SPIR-V assembly

### Shader Printf Debugging
Use `debugPrintfEXT` extension (requires Vulkan 1.3+ and validation layers):
```glsl
#extension GL_EXT_debug_printf : enable

void main() {
    debugPrintfEXT("Value at pixel (%d,%d): %f", gl_FragCoord.x, gl_FragCoord.y, my_value);
}
```

### Common Issues
- **Black screen:** Check if vertex shader properly transforms to clip space
- **Incorrect colors:** Verify color space conversions (linear vs sRGB)
- **Flickering:** Check depth buffer configuration and winding order
- **Performance:** Use GPU profiler (RenderDoc, NSight, PIX) to identify bottlenecks

---

## References

### Specifications
- [Vulkan GLSL Specification](https://www.khronos.org/registry/vulkan/specs/1.3/html/vkspec.html#shaders)
- [GLSL Language Specification 4.60](https://www.khronos.org/registry/OpenGL/specs/gl/GLSLangSpec.4.60.pdf)
- [SPIR-V Specification](https://www.khronos.org/registry/spir-v/)

### PBR and Lighting
- [LearnOpenGL - PBR Theory](https://learnopengl.com/PBR/Theory)
- [Real Shading in Unreal Engine 4](https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf)
- [Physically Based Rendering Book](https://www.pbr-book.org/)

### Advanced Techniques
- [GPU Gems Series](https://developer.nvidia.com/gpugems/gpugems/contributors)
- [Real-Time Rendering 4th Edition](https://www.realtimerendering.com/)
- [LTC Area Lights](https://eheitzresearch.wordpress.com/415-2/)
- [Temporal Anti-Aliasing](https://de45xmedrsdbp.cloudfront.net/Resources/files/TemporalAA_small-59732822.pdf)

### Vulkan Resources
- [Vulkan Tutorial](https://vulkan-tutorial.com/)
- [Vulkano Documentation](https://vulkano.rs/)
- [Sascha Willems Examples](https://github.com/SaschaWillems/Vulkan)
