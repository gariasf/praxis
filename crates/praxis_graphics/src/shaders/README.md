# Praxis Graphics Shaders

This directory contains all GLSL shaders used by the Praxis graphics system.

## Shader Categories

### Core Rendering

#### `triangle.vert` / `triangle.frag`
Basic 3D rendering shaders with PBR support:
- Vertex transformations (model, view, projection)
- Normal mapping with TBN matrix
- Material properties (metallic, roughness, emissive)
- Multiple light sources (directional, point)
- Shadow mapping with cascades

### Deferred Rendering

#### `deferred_geometry.vert` / `deferred_geometry.frag`
Geometry pass for deferred rendering:
- Outputs to G-buffer (albedo, normal, metallic-roughness, depth)
- Efficient for scenes with many lights
- Supports normal mapping and PBR materials

#### `deferred_lighting.vert` / `deferred_lighting.frag`
Lighting pass for deferred rendering:
- Full-screen quad processing
- Reads from G-buffer
- Accumulates lighting from all sources
- Supports ambient, directional, and point lights

### Shadow Mapping

#### `shadow.vert` / `shadow.frag`
Shadow map generation:
- Depth-only rendering from light's perspective
- Supports cascaded shadow maps (CSM)
- Used for directional light shadows

### Skybox

#### `skybox.vert` / `skybox.frag`
Skybox rendering with cubemap:
- Camera-centered rendering
- Reversed depth for infinite distance
- Samples cubemap based on view direction

### Environment Probes / IBL

#### `ibl_irradiance.vert` / `ibl_irradiance.frag`
Diffuse irradiance convolution:
- Converts environment map to irradiance map
- Hemisphere sampling over surface normals
- Cosine-weighted integration
- Output: 32x32 cubemap for ambient lighting

#### `ibl_prefilter.vert` / `ibl_prefilter.frag`
Specular reflection prefiltering:
- Importance sampling using GGX distribution
- Generates multiple roughness levels (mipmaps)
- 1024 samples per pixel for quality
- Output: Full-resolution cubemap with 5 mip levels

#### `equirect_to_cube.vert` / `equirect_to_cube.frag`
Equirectangular to cubemap conversion:
- Converts panoramic HDR images to cubemaps
- Spherical coordinate mapping
- Used for loading environment maps

### Screen-Space Ambient Occlusion (SSAO)

#### `ssao.vert` / `ssao.frag`
SSAO computation:
- Hemisphere sampling in screen space
- Depth-aware occlusion detection
- Noise texture for sample rotation
- Output: Occlusion factors

#### `ssao_blur.vert` / `ssao_blur.frag`
SSAO blur pass:
- Box blur to reduce noise artifacts
- Depth-aware filtering
- Preserves edges

### HDR Tone Mapping

#### `hdr_tone_map.frag`
HDR tone mapping:
- Multiple tone mapping operators (ACES, Reinhard, Uncharted 2)
- Exposure adjustment (manual and automatic)
- Gamma correction
- Converts HDR to LDR [0,1]

### Post-Processing

#### `post_process.vert`
Shared vertex shader for post-processing:
- Full-screen quad generation
- UV coordinate passthrough

#### `post_process_grayscale.frag`
Grayscale conversion:
- Luminance-based conversion
- Perceptually accurate weights

#### `post_process_brightness_extract.frag`
Bloom brightness extraction:
- Extracts pixels above brightness threshold
- Used as first pass for bloom effect

#### `post_process_gaussian_blur_h.frag` / `post_process_gaussian_blur_v.frag`
Separable Gaussian blur:
- Horizontal and vertical passes
- Configurable kernel size
- Used for bloom and other effects

#### `post_process_copy.frag`
Simple texture copy:
- Direct texture sampling
- Used for render target copies

#### `post_process_tone_map.frag`
Post-process tone mapping:
- Simple Reinhard tone mapping
- Gamma correction
- Used in bloom combine pass

### Cinematic Post-Processing Effects

#### `post_process_dof.frag`
Depth-of-Field with bokeh blur:
- Circle of confusion calculation
- Poisson disk sampling for bokeh
- Depth-aware blur
- Configurable focus distance and aperture

#### `post_process_motion_blur.frag`
Motion blur using velocity buffer:
- Per-pixel velocity sampling
- Sample accumulation along motion vectors
- Shutter angle simulation
- Configurable sample count

#### `post_process_chromatic_aberration.frag`
Chromatic aberration lens distortion:
- Radial color fringing
- Separate R/G/B channel offsets
- Distance-based falloff
- Configurable intensity

#### `post_process_vignette.frag`
Vignette darkening effect:
- Edge darkening for cinematic framing
- Configurable shape and smoothness
- Center point control
- Intensity adjustment

#### `post_process_film_grain.frag`
Film grain noise:
- Procedural grain generation
- Luminance-based intensity
- Animated grain for realism
- Configurable grain size

### Velocity Buffer Generation

#### `velocity_buffer.vert` / `velocity_buffer.frag`
Velocity buffer for motion blur:
- Current and previous frame MVP matrices
- Per-pixel screen-space motion vectors
- Used by motion blur effect

### Particle System

#### `particle.vert` / `particle.frag`
Particle rendering with soft particles:
- Billboard particle rendering facing camera
- Soft particles with depth buffer comparison
- Smooth fade-out near geometry
- Per-particle rotation and coloring
- Alpha blending for transparency

#### `particle_sort.comp`
GPU-based bitonic sort for particles:
- Sorts particles by camera distance for correct alpha blending
- Efficient parallel sorting algorithm
- Works on power-of-two particle counts
- Uses bitonic sort algorithm for parallel execution

## Shader Conventions

### Binding Locations

**Set 0: Per-frame uniforms**
- Binding 0: View/Projection matrices
- Binding 1: Model matrices (dynamic uniform buffer)
- Binding 2: Albedo texture
- Binding 3: Lighting data
- Binding 4: Shadow data
- Bindings 5-8: Shadow map samplers (one per cascade)
- Binding 9: Normal map texture

**Set 1: Per-material uniforms**
- Binding 0: Material properties (metallic, roughness, emissive)

**Set 2: IBL uniforms (when used)**
- Binding 0: Irradiance cubemap
- Binding 1: Prefiltered cubemap
- Binding 2: BRDF LUT

### Vertex Attributes

**Standard 3D vertex format:**
- Location 0: Position (vec3)
- Location 1: Normal (vec3)
- Location 2: Color (vec3)
- Location 3: UV coordinates (vec2)
- Location 4: Tangent (vec4, w = handedness)

### Coordinate Systems

- **Right-handed coordinate system**
- **Y-up convention**
- **Depth range: [0, 1]** (Vulkan convention)
- **Cubemap faces:** +X, -X, +Y, -Y, +Z, -Z order

### Color Spaces

- **Input textures:** sRGB color space (automatically linearized)
- **Rendering:** Linear color space
- **Output:** sRGB after gamma correction (2.2)

## Shader Compilation

Shaders are compiled at runtime using `vulkano-shaders` macro. The compilation process:

1. Parse GLSL source files
2. Compile to SPIR-V
3. Generate Rust bindings
4. Embed in binary

For development, manual compilation can be done with:
```bash
glslangValidator -V shader.vert -o shader.vert.spv
glslangValidator -V shader.frag -o shader.frag.spv
```

## Performance Tips

### Minimize Texture Samples
- Use mipmaps for distant textures
- Cache texture lookups when used multiple times
- Use texture LOD explicitly when possible

### Optimize Branching
- Prefer arithmetic over conditionals
- Use `mix()` for interpolation instead of `if`
- Flatten short branches with boolean arithmetic

### Reduce Bandwidth
- Pack data efficiently (use smaller formats)
- Minimize render target writes in deferred rendering
- Use compressed texture formats (BC1, BC3, BC6H)

### Leverage Hardware
- Use early-Z testing (render front-to-back)
- Align data to 16-byte boundaries
- Use SIMD-friendly operations (vec4, mat4)

## Adding New Shaders

When adding a new shader to the system:

1. Create `.vert` and `.frag` files in this directory
2. Follow naming conventions (descriptive, lowercase with underscores)
3. Document inputs, outputs, and purpose in header comments
4. Add binding locations using consistent conventions
5. Update this README with shader description
6. Add shader loading in corresponding Rust module
7. Write example usage in module documentation

## Debugging Shaders

### Validation Layers
Enable Vulkan validation layers for detailed error messages:
```rust
let instance = Instance::new(/* ... with validation layers */);
```

### RenderDoc
Use RenderDoc for frame capture and shader debugging:
1. Launch application through RenderDoc
2. Capture frame
3. Inspect shader inputs/outputs
4. View generated SPIR-V assembly

### Printf Debugging
Use `debugPrintfEXT` (Vulkan 1.3+):
```glsl
#extension GL_EXT_debug_printf : enable
debugPrintfEXT("Value: %f", my_value);
```

## References

- [Vulkan GLSL Specification](https://www.khronos.org/registry/vulkan/specs/1.3/html/vkspec.html#shaders)
- [GLSL Language Specification](https://www.khronos.org/registry/OpenGL/specs/gl/GLSLangSpec.4.60.pdf)
- [LearnOpenGL - PBR Theory](https://learnopengl.com/PBR/Theory)
- [Real Shading in Unreal Engine 4](https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf)
