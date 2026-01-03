# Advanced Rendering Techniques

This document covers advanced rendering techniques in Praxis Graphics, including Screen-Space Ambient Occlusion (SSAO), High Dynamic Range (HDR) rendering, and Image-Based Lighting (IBL).

## Table of Contents

1. [Screen-Space Ambient Occlusion (SSAO)](#screen-space-ambient-occlusion-ssao)
2. [High Dynamic Range (HDR) Rendering](#high-dynamic-range-hdr-rendering)
3. [Image-Based Lighting (IBL)](#image-based-lighting-ibl)

---

# Screen-Space Ambient Occlusion (SSAO)

SSAO is a screen-space technique that approximates ambient occlusion by sampling the depth buffer around each pixel. It simulates indirect lighting occlusion by darkening pixels surrounded by nearby geometry (crevices, corners, contact shadows).

## Theory

### Ambient Occlusion Concept

**Ambient Occlusion (AO)** measures how exposed a point on a surface is to ambient lighting:
- **1.0 (white)**: Fully exposed, receives full ambient light
- **0.0 (black)**: Fully occluded, receives no ambient light

**Mathematical Definition**:
```
AO(p) = 1 - (1/π) ∫_Ω V(p, ω) * cos(θ) dω

where:
  p = surface point
  Ω = hemisphere around surface normal
  V(p, ω) = visibility function (0 if occluded, 1 if visible)
  θ = angle between normal and sample direction
```

**Ground Truth AO** requires ray tracing the full hemisphere, which is too expensive for real-time rendering.

### Screen-Space Approximation

SSAO approximates AO using only screen-space data:
1. **Sample hemisphere** around each pixel in view space
2. **Test depth buffer** for each sample
3. **Count occlusion**: How many samples are behind geometry?
4. **Output occlusion factor**: 0.0 = fully occluded, 1.0 = not occluded

**Key Insight**: Instead of ray tracing, we sample the depth buffer at nearby screen-space positions.

## Algorithm Steps

### Step 1: Generate Sample Kernel

Create a hemisphere of sample points distributed in view space:

```rust
fn generate_sample_kernel(count: u32) -> Vec<Vec3> {
    let mut kernel = Vec::with_capacity(count as usize);
    
    for i in 0..count {
        // Random point in hemisphere
        let mut sample = Vec3::new(
            random_range(-1.0, 1.0),  // x: -1 to 1
            random_range(-1.0, 1.0),  // y: -1 to 1
            random_range(0.0, 1.0),   // z: 0 to 1 (hemisphere)
        );
        
        sample = sample.normalize();
        
        // Scale samples: more near origin, fewer far away
        let scale = i as f32 / count as f32;
        scale = lerp(0.1, 1.0, scale * scale);  // Quadratic distribution
        sample *= scale;
        
        kernel.push(sample);
    }
    
    kernel
}
```

**Distribution**: Samples are concentrated near the surface (scale²) for better contact shadow detail.

**Typical kernel sizes**:
- **16 samples**: Fast, noisy, suitable for real-time with heavy blur
- **32 samples**: Balanced quality/performance
- **64 samples**: High quality, smooth results
- **128+ samples**: Overkill, diminishing returns

### Step 2: Generate Noise Texture

Create a small texture of random rotation vectors to reduce banding:

```rust
fn generate_noise_texture(size: u32) -> Vec<Vec3> {
    let mut noise = Vec::with_capacity((size * size) as usize);
    
    for _ in 0..(size * size) {
        // Random tangent-space vector (rotate sample kernel)
        let vec = Vec3::new(
            random_range(-1.0, 1.0),
            random_range(-1.0, 1.0),
            0.0,  // Only rotate in XY plane
        );
        noise.push(vec.normalize());
    }
    
    noise
}
```

**Purpose**: Each pixel gets a different random rotation of the sample kernel, breaking up patterns and reducing banding artifacts.

**Typical size**: 4×4 texture (tiled across screen)

### Step 3: SSAO Pass

For each pixel, sample the depth buffer and accumulate occlusion:

```glsl
#version 450

layout(location = 0) in vec2 v_uv;
layout(location = 0) out float o_occlusion;

// G-buffer inputs
layout(set = 0, binding = 0) uniform sampler2D u_normal;
layout(set = 0, binding = 1) uniform sampler2D u_depth;
layout(set = 0, binding = 2) uniform sampler2D u_noise;

layout(set = 0, binding = 3) uniform SsaoUniforms {
    mat4 projection;
    mat4 view;
    vec4 samples[64];  // Sample kernel in view space
    vec2 noise_scale;  // Screen size / noise texture size
    float radius;      // Sample hemisphere radius
    float bias;        // Depth bias to prevent self-occlusion
    float power;       // Power curve for artistic control
    int kernel_size;   // Number of samples to use
} ssao;

// Reconstruct view-space position from depth
vec3 reconstruct_view_position(vec2 uv, float depth) {
    // Convert to NDC
    vec4 ndc = vec4(
        uv * 2.0 - 1.0,
        depth * 2.0 - 1.0,
        1.0
    );
    
    // Unproject
    vec4 view_pos = inverse(ssao.projection) * ndc;
    return view_pos.xyz / view_pos.w;
}

void main() {
    // Sample G-buffer
    vec3 world_normal = texture(u_normal, v_uv).rgb;
    float depth = texture(u_depth, v_uv).r;
    
    // Skip skybox (depth = 1.0)
    if (depth >= 0.9999) {
        o_occlusion = 1.0;
        return;
    }
    
    // Reconstruct view-space position
    vec3 view_pos = reconstruct_view_position(v_uv, depth);
    
    // Transform normal to view space
    vec3 view_normal = normalize((ssao.view * vec4(world_normal, 0.0)).xyz);
    
    // Sample noise texture for kernel rotation
    vec2 noise_uv = v_uv * ssao.noise_scale;
    vec3 random_vec = normalize(texture(u_noise, noise_uv).rgb * 2.0 - 1.0);
    
    // Construct TBN matrix to orient sample kernel
    vec3 tangent = normalize(random_vec - view_normal * dot(random_vec, view_normal));
    vec3 bitangent = cross(view_normal, tangent);
    mat3 TBN = mat3(tangent, bitangent, view_normal);
    
    // Accumulate occlusion
    float occlusion = 0.0;
    
    for (int i = 0; i < ssao.kernel_size; i++) {
        // Get sample position in view space
        vec3 sample_pos = TBN * ssao.samples[i].xyz;
        sample_pos = view_pos + sample_pos * ssao.radius;
        
        // Project sample to screen space
        vec4 offset = ssao.projection * vec4(sample_pos, 1.0);
        offset.xyz /= offset.w;
        offset.xyz = offset.xyz * 0.5 + 0.5;  // NDC to UV
        
        // Sample depth at offset position
        float sample_depth = texture(u_depth, offset.xy).r;
        vec3 sample_view_pos = reconstruct_view_position(offset.xy, sample_depth);
        
        // Range check: only occlude if sample is close enough
        float range_check = smoothstep(0.0, 1.0, ssao.radius / abs(view_pos.z - sample_view_pos.z));
        
        // Occlusion test: is sample behind geometry?
        float depth_difference = sample_view_pos.z - sample_pos.z;
        occlusion += (depth_difference >= ssao.bias ? 1.0 : 0.0) * range_check;
    }
    
    // Normalize and invert (1.0 = not occluded)
    occlusion = 1.0 - (occlusion / float(ssao.kernel_size));
    
    // Apply power curve for artistic control
    occlusion = pow(occlusion, ssao.power);
    
    o_occlusion = occlusion;
}
```

**Key Parameters**:
- **radius**: Size of the sampling hemisphere (0.5 = small details, 2.0 = large scale AO)
- **bias**: Prevents self-occlusion artifacts (typically 0.025)
- **power**: Contrast control (1.0 = linear, 2.0 = darkens, 0.5 = lightens)

### Step 4: Blur Pass

Apply bilateral blur to reduce noise while preserving edges:

```glsl
#version 450

layout(location = 0) in vec2 v_uv;
layout(location = 0) out float o_occlusion;

layout(set = 0, binding = 0) uniform sampler2D u_occlusion;

layout(push_constant) uniform PushConstants {
    vec2 texel_size;  // 1.0 / resolution
} push;

void main() {
    vec2 texel_size = push.texel_size;
    float result = 0.0;
    
    // 4×4 box blur
    for (int x = -2; x < 2; x++) {
        for (int y = -2; y < 2; y++) {
            vec2 offset = vec2(float(x), float(y)) * texel_size;
            result += texture(u_occlusion, v_uv + offset).r;
        }
    }
    
    o_occlusion = result / 16.0;
}
```

**Blur types**:
1. **Box blur**: Simple average, can blur across edges
2. **Gaussian blur**: Weighted average, smoother results
3. **Bilateral blur**: Edge-aware, preserves geometry boundaries (preferred)

## Integration with Deferred Rendering

SSAO integrates seamlessly with deferred rendering:

```rust
// After G-buffer pass
let ssao_texture = ssao_renderer.render(
    builder,
    &gbuffer,
    projection_matrix,
    view_matrix,
)?;

// In lighting pass
deferred_renderer.render_with_ssao(
    builder,
    output_framebuffer,
    viewport,
    draw_commands,
    view_proj_buffer,
    dynamic_uniform_buffer,
    mesh_manager,
    texture_manager,
    lighting_buffer,
    ssao_texture,  // Applied to ambient lighting
)?;
```

**In lighting shader**:
```glsl
float ao = texture(u_ssao, v_uv).r;
vec3 ambient = lighting.ambient_color.rgb * albedo * ao;
```

## Quality Tuning

### Kernel Size vs. Performance

| Samples | Quality | Performance | Use Case |
|---------|---------|-------------|----------|
| 16      | Low     | ~0.3ms      | Mobile, fast-paced games |
| 32      | Medium  | ~0.6ms      | Balanced |
| 64      | High    | ~1.2ms      | Desktop, quality mode |
| 128     | Very High | ~2.4ms    | Cinematic, offline |

*(Times approximate for 1080p on mid-range GPU)*

### Radius Tuning

- **Small radius (0.3-0.5)**: Contact shadows, fine details
- **Medium radius (0.5-1.0)**: General-purpose AO
- **Large radius (1.0-3.0)**: Large-scale occlusion, can look soft

**Best practice**: Use small radius with more samples for tight, detailed AO.

### Bias Adjustment

- **Too low**: Self-occlusion artifacts (surface darkens incorrectly)
- **Too high**: Loses contact shadows (floaty look)
- **Typical values**: 0.01-0.05

### Power Curve

```
occlusion_final = pow(occlusion, power)

power = 1.0: Linear (realistic)
power = 2.0: More contrast (darker shadows)
power = 0.5: Less contrast (subtle)
```

## Common Artifacts and Solutions

### 1. Banding/Pattern Artifacts

**Cause**: Insufficient noise texture coverage or low sample count

**Solutions**:
- Increase noise texture size (4×4 to 8×8)
- Generate better random rotations
- Add temporal noise (different each frame)
- Increase blur kernel size

### 2. Halo Artifacts

**Cause**: Occlusion bleeding across depth discontinuities

**Solutions**:
- Implement bilateral blur (depth-aware)
- Reduce radius near edges
- Adjust range check in SSAO pass

### 3. Self-Occlusion

**Cause**: Bias too low, surface samples itself

**Solutions**:
- Increase bias parameter
- Use normal-oriented hemisphere (samples only above surface)
- Better depth comparison logic

### 4. Over-Darkening

**Cause**: Too many samples registering as occluded

**Solutions**:
- Reduce radius
- Increase bias
- Adjust power curve (lower value)
- Check range check smoothstep

## Performance Optimization

### 1. Resolution Reduction

Render SSAO at half resolution:
- 1920×1080 → 960×540
- 4× fewer pixels to process
- Upscale with bilateral filter
- **Savings**: 75% of SSAO cost

### 2. Temporal Accumulation

Spread samples across multiple frames:
- Frame 1: Sample kernel offset 0
- Frame 2: Sample kernel offset 1
- Blend results over time
- Requires motion vectors for moving objects

### 3. Hierarchical Depth Buffer

Use mipmapped depth buffer:
- Sample lower mips for distant samples
- Reduces cache misses
- Improves performance with large radius

### 4. Compute Shader Implementation

Use compute shaders instead of fragment shaders:
- Better occupancy
- Shared memory for G-buffer data
- Coalesced memory access

---

# High Dynamic Range (HDR) Rendering

HDR rendering uses floating-point precision to represent colors beyond the [0,1] range, enabling more realistic lighting and better post-processing effects.

## Theory

### Why HDR?

**Low Dynamic Range (LDR)** limits color values to [0, 1]:
- Cannot represent bright lights properly
- Loses information when values exceed 1.0
- Poor bloom and glow effects
- Unrealistic exposure handling

**High Dynamic Range (HDR)** uses floating-point values:
- Colors can exceed 1.0 (bright lights, sun, etc.)
- Preserves lighting information
- Enables realistic tone mapping
- Better bloom and glow effects
- Proper exposure simulation

### Real-World Luminance

Human vision can perceive a dynamic range of ~10,000,000:1 (night to bright sunlight).

**Examples**:
- Starlight: 0.001 lux
- Indoor lighting: 100-500 lux
- Overcast day: 10,000 lux
- Direct sunlight: 100,000 lux
- Sun surface: 1,000,000,000 lux (relative)

**LDR**: All values clamped to [0, 1] – sun is as bright as white paper
**HDR**: Values can represent true brightness ratios – sun is 1000× brighter than indoor lights

## HDR Rendering Pipeline

### Stage 1: HDR Scene Rendering

Render scene to floating-point render target:

```rust
// Create HDR render target
let hdr_render_pass = render_context.create_hdr_render_pass()?;
let hdr_target = HdrRenderTarget::new(
    memory_allocator,
    hdr_render_pass,
    [1920, 1080],
)?;

// Render scene to HDR target
render_scene_to_hdr(render_context, &hdr_target, draw_commands)?;
```

**Format**: `R16G16B16A16_SFLOAT`
- 16-bit floating-point per channel
- Range: -65504 to +65504
- Sufficient precision for HDR
- 8 bytes per pixel (2× LDR size)

**Shader Output**:
```glsl
layout(location = 0) out vec4 o_color;

void main() {
    // Calculate lighting (can exceed 1.0)
    vec3 color = calculate_pbr_lighting(...);
    
    // No clamping! Output raw HDR values
    o_color = vec4(color, 1.0);
}
```

### Stage 2: Luminance Calculation

Calculate average scene luminance for automatic exposure:

```glsl
// Average luminance calculation
float luminance(vec3 color) {
    return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

vec3 hdr_color = texture(u_hdr_texture, uv).rgb;
float luma = luminance(hdr_color);
// Average across all pixels
```

**Methods**:
1. **Compute shader reduction**: Parallel sum on GPU
2. **Mipmapped luminance**: Downsample luminance to 1×1 texture
3. **CPU readback**: Download and compute on CPU (slow)
4. **Approximation**: Use fixed value based on scene knowledge

**Typical values**:
- Dark indoor: 0.05
- Indoor: 0.1-0.3
- Outdoor (overcast): 0.5
- Outdoor (sunny): 1.0-2.0
- Very bright: 5.0+

### Stage 3: Exposure Calculation

**Manual Exposure**:
```rust
let exposure = 1.5;  // Fixed value
```

**Automatic Exposure**:
```rust
struct ExposureCalculator {
    current_exposure: f32,
    target_exposure: f32,
    key_value: f32,        // Target middle gray (0.18)
    min_exposure: f32,      // Lower bound
    max_exposure: f32,      // Upper bound
    adaptation_speed: f32,  // How fast to adapt
}

impl ExposureCalculator {
    fn calculate(&mut self, average_luminance: f32, delta_time: f32) -> f32 {
        // Calculate target exposure
        self.target_exposure = self.key_value / (average_luminance + 0.001);
        self.target_exposure = self.target_exposure.clamp(self.min_exposure, self.max_exposure);
        
        // Smoothly adapt current exposure to target
        let adaptation_rate = 1.0 - f32::exp(-self.adaptation_speed * delta_time);
        self.current_exposure = lerp(self.current_exposure, self.target_exposure, adaptation_rate);
        
        self.current_exposure
    }
}
```

**Parameters**:
- **key_value**: Target brightness for middle tones (0.18 = 18% gray, photographic standard)
- **adaptation_speed**: 1.0 = slow (realistic eye adaptation), 5.0 = fast (responsive gameplay)
- **min/max_exposure**: Prevents extreme values (typically 0.1 to 10.0)

### Stage 4: Tone Mapping

Convert HDR values to displayable LDR range [0, 1]:

```glsl
vec3 tone_map(vec3 hdr_color, float exposure) {
    // Apply exposure
    vec3 exposed = hdr_color * exposure;
    
    // Apply tone mapping operator
    vec3 ldr_color = tone_map_operator(exposed);
    
    // Gamma correction
    ldr_color = pow(ldr_color, vec3(1.0 / 2.2));
    
    return ldr_color;
}
```

## Tone Mapping Operators

### 1. Reinhard

**Formula**:
```glsl
vec3 reinhard(vec3 color) {
    return color / (color + vec3(1.0));
}
```

**Characteristics**:
- Simple and fast
- Smooth compression
- Preserves hue
- Can look flat in very bright scenes

**When to use**: Fast iteration, simple scenes, mobile devices

### 2. ACES Filmic (Recommended)

**Formula** (Approximation of Academy Color Encoding System):
```glsl
vec3 aces_filmic(vec3 x) {
    float a = 2.51;
    float b = 0.03;
    float c = 2.43;
    float d = 0.59;
    float e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}
```

**Characteristics**:
- Industry standard (film and games)
- Cinematic look with good color preservation
- Proper highlights and shadows
- Slightly more expensive than Reinhard

**When to use**: Production quality, AAA games, realistic rendering

**Examples using ACES**: *The Last of Us*, *Uncharted 4*, *Call of Duty*

### 3. Uncharted 2 (Hable)

**Formula**:
```glsl
vec3 uncharted2_partial(vec3 x) {
    float A = 0.15;  // Shoulder strength
    float B = 0.50;  // Linear strength
    float C = 0.10;  // Linear angle
    float D = 0.20;  // Toe strength
    float E = 0.02;  // Toe numerator
    float F = 0.30;  // Toe denominator
    return ((x * (A * x + C * B) + D * E) / (x * (A * x + B) + D * F)) - E / F;
}

vec3 uncharted2(vec3 color) {
    float exposure_bias = 2.0;
    vec3 curr = uncharted2_partial(color * exposure_bias);
    vec3 white_scale = 1.0 / uncharted2_partial(vec3(11.2));
    return curr * white_scale;
}
```

**Characteristics**:
- High contrast
- Dramatic look with strong toe and shoulder
- Popular in games
- Similar cost to ACES

**When to use**: High-contrast scenes, dramatic lighting, stylized games

### Comparison

| Operator | Speed | Contrast | Color | Best For |
|----------|-------|----------|-------|----------|
| Reinhard | Fast | Low | Good | Simple scenes, prototyping |
| ACES | Medium | Medium | Excellent | Production, realism |
| Uncharted 2 | Medium | High | Good | Dramatic, stylized |

## Tone Mapping Workflow

```rust
use praxis_graphics::{ToneMapper, ToneMappingOperator, ExposureMode};

// Create tone mapper
let mut tone_mapper = ToneMapper::new(
    device,
    memory_allocator,
    Format::R8G8B8A8_UNORM,  // Output format
    ToneMappingOperator::ACES,
)?;

// Configure exposure mode
tone_mapper.set_exposure_mode(ExposureMode::Automatic {
    speed: 2.0,  // Adaptation speed
});

// In render loop
tone_mapper.apply(
    command_buffer,
    &hdr_target,           // HDR input
    output_framebuffer,    // LDR output
    output_extent,
    average_luminance,     // From scene
    delta_time,           // For smooth adaptation
)?;
```

### Switching Operators

```rust
// Runtime switching
tone_mapper.set_operator(ToneMappingOperator::Reinhard);
tone_mapper.set_operator(ToneMappingOperator::ACES);
tone_mapper.set_operator(ToneMappingOperator::Uncharted2);
```

## HDR Best Practices

### 1. Light Intensity Values

Set realistic light intensities for HDR:

```rust
// Directional light (sun)
directional_light.intensity = 5.0;  // Much brighter than LDR (1.0)

// Point light (lamp)
point_light.intensity = 50.0;  // Bright indoor light

// Emissive materials
material.emissive = 2.0;  // Self-illuminated surfaces
```

**Rule of thumb**: Think in physical units (candelas, lumens) if possible.

### 2. Color Grading

Adjust tone mapper parameters for artistic control:

```glsl
// Before tone mapping
color = color * color_grading.contrast;
color = color + vec3(color_grading.brightness);
color = color * color_grading.saturation;

// Then tone map
color = tone_map(color, exposure);
```

### 3. Bloom Integration

HDR makes bloom effects look much better:

```rust
// Render scene to HDR
render_scene(&hdr_target)?;

// Extract bright pixels (threshold in HDR space)
bloom.extract_bright(&hdr_target, threshold: 1.0)?;

// Blur bright pixels
bloom.blur()?;

// Combine with HDR scene
bloom.composite(&hdr_target)?;

// Then tone map the bloomed HDR result
tone_mapper.apply(&hdr_target, &output_framebuffer)?;
```

### 4. Monitor Exposure in Debug UI

```rust
// Display current exposure value
ui.label(format!("Exposure: {:.2}", tone_mapper.current_exposure()));
ui.label(format!("Avg Luminance: {:.3}", average_luminance));

// Exposure override slider for testing
ui.slider("Manual Exposure", &mut manual_exposure, 0.1..=10.0);
```

## Performance Considerations

### Memory Usage

HDR doubles memory for render targets:
- **LDR (R8G8B8A8)**: 4 bytes per pixel
- **HDR (R16G16B16A16_SFLOAT)**: 8 bytes per pixel

**1920×1080**: ~8 MB LDR → ~16 MB HDR

### Tone Mapping Cost

Tone mapping is a full-screen pass:
- **Cost**: O(pixels), typically <1ms at 1080p
- **Operator complexity**: All operators have similar cost

### Bandwidth

Reading/writing HDR buffers uses 2× bandwidth:
```
LDR: 1920×1080 × 60fps × 4 bytes = ~475 MB/s
HDR: 1920×1080 × 60fps × 8 bytes = ~950 MB/s
```

---

# Image-Based Lighting (IBL)

Image-Based Lighting uses environment maps to provide realistic reflections and ambient lighting from the surrounding scene.

## Theory

### Why IBL?

**Traditional ambient lighting**:
```glsl
vec3 ambient = ambient_color * albedo;
```
- Uniform color from all directions
- Unrealistic
- No environment reflections
- No indirect lighting

**Image-Based Lighting**:
```glsl
vec3 ambient = sample_irradiance_map(normal) * albedo;
vec3 reflection = sample_prefiltered_map(reflect_dir, roughness);
```
- Lighting varies by direction
- Captures environment appearance
- Realistic reflections on materials
- Approximates indirect lighting

### Physically-Based Rendering with IBL

IBL completes the PBR lighting equation:

```
L_o = ∫_Ω (k_d * albedo/π + k_s * BRDF) * L_i(ω) * cos(θ) dω

where:
  L_o = outgoing radiance (final color)
  Ω = hemisphere around normal
  k_d = diffuse weight (1 - metallic)
  k_s = specular weight (Fresnel)
  L_i(ω) = incoming radiance from direction ω
  θ = angle between normal and light direction
```

**Direct lighting**: L_i from point/directional lights (finite set)
**IBL**: L_i from environment map (infinite directions)

## IBL Data Structures

### 1. Environment Map (Cubemap)

**Purpose**: Captures the surrounding scene in all directions

**Format**: HDR cubemap (6 faces: +X, -X, +Y, -Y, +Z, -Z)
- Resolution: 256×256, 512×512, or 1024×1024 per face
- Format: `R16G16B16A16_SFLOAT` (HDR)
- Content: HDR image of the environment

**Capture**:
```rust
// Position probe at point of interest
let probe_position = Vec3::new(0.0, 2.0, 0.0);

// Render 6 views (90° FOV each)
for face in 0..6 {
    let view_matrix = get_cubemap_face_view(probe_position, face);
    let proj_matrix = perspective_fov(90.0_degrees, 1.0, near, far);
    
    render_scene_to_cubemap_face(
        face,
        view_matrix,
        proj_matrix,
        &environment_map,
    )?;
}
```

**6 Views**:
- +X (right): look right, up = +Y
- -X (left): look left, up = +Y
- +Y (top): look up, up = -Z
- -Y (bottom): look down, up = +Z
- +Z (forward): look forward, up = +Y
- -Z (back): look back, up = +Y

### 2. Irradiance Map (Diffuse IBL)

**Purpose**: Precomputed diffuse lighting for lambertian surfaces

**Format**: Low-res cubemap (32×32 or 64×64)
- Much smaller than environment map
- Contains convolved diffuse lighting
- Averaged over hemisphere for each direction

**Generation** (Convolution):
```glsl
vec3 irradiance = vec3(0.0);
int samples = 0;

// Integrate over hemisphere around normal N
for (float phi = 0.0; phi < 2.0 * PI; phi += sample_delta) {
    for (float theta = 0.0; theta < 0.5 * PI; theta += sample_delta) {
        // Sample direction in tangent space
        vec3 tangent_sample = vec3(
            sin(theta) * cos(phi),
            sin(theta) * sin(phi),
            cos(theta)
        );
        
        // Transform to world space
        vec3 sample_vec = tangent_to_world(tangent_sample, N);
        
        // Sample environment map
        vec3 env_color = texture(environment_map, sample_vec).rgb;
        
        // Accumulate with cosine weighting
        irradiance += env_color * cos(theta) * sin(theta);
        samples++;
    }
}

irradiance = PI * irradiance / float(samples);
```

**Mathematical basis**:
```
Irradiance(N) = ∫_Ω L_i(ω) * cos(θ) dω

Approximation via Monte Carlo integration:
≈ (π / num_samples) * Σ L_i(ω_i) * cos(θ_i) * sin(θ_i)
```

**Usage in shader**:
```glsl
vec3 irradiance = texture(irradiance_map, normal).rgb;
vec3 diffuse = irradiance * albedo * (1.0 - metallic);
```

### 3. Prefiltered Environment Map (Specular IBL)

**Purpose**: Precomputed specular reflections for different roughness levels

**Format**: Cubemap with mipmaps (5-6 levels)
- Mip 0: Sharp reflections (roughness 0.0)
- Mip 1: Slightly blurred (roughness 0.25)
- Mip 2: Medium blur (roughness 0.5)
- Mip 3: Blurred (roughness 0.75)
- Mip 4: Very blurred (roughness 1.0)

**Generation** (Importance Sampling):
```glsl
vec3 prefiltered_color = vec3(0.0);
float total_weight = 0.0;

for (int i = 0; i < SAMPLE_COUNT; i++) {
    // Quasi-random sequence
    vec2 xi = hammersley(i, SAMPLE_COUNT);
    
    // Importance sample GGX distribution
    vec3 H = importance_sample_ggx(xi, N, roughness);
    vec3 L = reflect(-V, H);
    
    float n_dot_l = dot(N, L);
    if (n_dot_l > 0.0) {
        // Sample environment map
        vec3 env_color = texture(environment_map, L).rgb;
        
        // Weight by BRDF and visibility
        prefiltered_color += env_color * n_dot_l;
        total_weight += n_dot_l;
    }
}

prefiltered_color /= total_weight;
```

**GGX Importance Sampling**:
```glsl
vec3 importance_sample_ggx(vec2 xi, vec3 N, float roughness) {
    float a = roughness * roughness;
    
    // Spherical coordinates
    float phi = 2.0 * PI * xi.x;
    float cos_theta = sqrt((1.0 - xi.y) / (1.0 + (a*a - 1.0) * xi.y));
    float sin_theta = sqrt(1.0 - cos_theta * cos_theta);
    
    // Tangent space halfway vector
    vec3 H = vec3(
        sin_theta * cos(phi),
        sin_theta * sin(phi),
        cos_theta
    );
    
    // Transform to world space
    return tangent_to_world(H, N);
}
```

**Hammersley Sequence** (Low-discrepancy sampling):
```glsl
vec2 hammersley(uint i, uint N) {
    uint bits = i;
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    float rdi = float(bits) * 2.3283064365386963e-10;
    return vec2(float(i) / float(N), rdi);
}
```

**Usage in shader**:
```glsl
vec3 R = reflect(-view_dir, normal);
float lod = roughness * MAX_REFLECTION_LOD;  // Typically 4.0
vec3 prefiltered = textureLod(prefiltered_map, R, lod).rgb;
vec3 specular = prefiltered * (F * brdf.x + brdf.y);
```

### 4. BRDF Integration Map (Split-Sum Approximation)

**Purpose**: Lookup table for BRDF integration

**Format**: 2D texture (512×512 or 256×256)
- X-axis: cos(θ) where θ = angle between normal and view (0 to 1)
- Y-axis: roughness (0 to 1)
- RG channels: (scale, bias) for F0 * scale + bias

**Split-Sum Approximation**:
```
∫ f(l,v) * L_i(l) * cos(θ) dl
≈ (∫ L_i(l) * cos(θ) dl) * (∫ f(l,v) * cos(θ) dl)
  [Prefiltered map]          [BRDF LUT]
```

**Generation**:
```glsl
vec2 integrate_brdf(float n_dot_v, float roughness) {
    vec3 V = vec3(sqrt(1.0 - n_dot_v * n_dot_v), 0.0, n_dot_v);
    vec3 N = vec3(0.0, 0.0, 1.0);
    
    float A = 0.0;  // Scale
    float B = 0.0;  // Bias
    
    for (int i = 0; i < SAMPLE_COUNT; i++) {
        vec2 xi = hammersley(i, SAMPLE_COUNT);
        vec3 H = importance_sample_ggx(xi, N, roughness);
        vec3 L = reflect(-V, H);
        
        float n_dot_l = max(L.z, 0.0);
        float n_dot_h = max(H.z, 0.0);
        float v_dot_h = max(dot(V, H), 0.0);
        
        if (n_dot_l > 0.0) {
            float G = geometry_smith_ibl(N, V, L, roughness);
            float G_vis = (G * v_dot_h) / (n_dot_h * n_dot_v);
            float Fc = pow(1.0 - v_dot_h, 5.0);
            
            A += (1.0 - Fc) * G_vis;
            B += Fc * G_vis;
        }
    }
    
    return vec2(A, B) / float(SAMPLE_COUNT);
}
```

**Usage in shader**:
```glsl
vec2 env_brdf = texture(brdf_lut, vec2(max(dot(N, V), 0.0), roughness)).rg;
vec3 specular = prefiltered_color * (F0 * env_brdf.x + env_brdf.y);
```

**Note**: BRDF LUT is shared across all environment probes (computed once).

## IBL in PBR Shader

Complete integration:

```glsl
void main() {
    // Sample G-buffer / material
    vec3 albedo = ...;
    vec3 normal = ...;
    float metallic = ...;
    float roughness = ...;
    float ao = ...;  // From SSAO
    
    vec3 view_dir = normalize(camera_pos - world_pos);
    vec3 reflect_dir = reflect(-view_dir, normal);
    
    // Calculate F0 (surface reflection at zero incidence)
    vec3 F0 = mix(vec3(0.04), albedo, metallic);
    
    // Diffuse IBL
    vec3 irradiance = texture(irradiance_map, normal).rgb;
    vec3 F = fresnel_schlick_roughness(max(dot(normal, view_dir), 0.0), F0, roughness);
    vec3 k_d = (1.0 - F) * (1.0 - metallic);
    vec3 diffuse = irradiance * albedo * k_d;
    
    // Specular IBL
    float lod = roughness * MAX_REFLECTION_LOD;
    vec3 prefiltered = textureLod(prefiltered_map, reflect_dir, lod).rgb;
    vec2 brdf = texture(brdf_lut, vec2(max(dot(normal, view_dir), 0.0), roughness)).rg;
    vec3 specular = prefiltered * (F0 * brdf.x + brdf.y);
    
    // Combine with AO
    vec3 ambient = (diffuse + specular) * ao;
    
    // Add direct lighting
    vec3 direct_lighting = calculate_direct_lights(...);
    
    vec3 final_color = ambient + direct_lighting;
    
    o_color = vec4(final_color, 1.0);
}
```

## Environment Probe System

### Probe Placement

**Static scenes**: Place probes at key locations with distinct lighting
- One probe per room (indoors)
- Grid pattern (outdoors, 10-20 units apart)
- Higher resolution for important areas

**Dynamic scenes**: Update probes when lighting changes
- Manual trigger on significant events
- Periodic updates (every N frames)
- Continuous (expensive, only for hero objects)

### Multiple Probes

Blend between probes based on distance:

```rust
fn get_ibl_data(position: Vec3, probe_manager: &ProbeManager) -> IblData {
    let nearby_probes = probe_manager.get_probes_in_range(position);
    
    let mut total_weight = 0.0;
    let mut blended = IblData::default();
    
    for probe in nearby_probes {
        let distance = (probe.position - position).length();
        
        if distance < probe.influence_radius {
            let weight = 1.0 / (distance + 1.0);
            blended += probe.ibl_data * weight;
            total_weight += weight;
        }
    }
    
    if total_weight > 0.0 {
        blended / total_weight
    } else {
        // Fallback to default sky
        IblData::default_sky()
    }
}
```

### Update Modes

```rust
pub enum ProbeUpdateMode {
    Once,              // Capture once at startup
    EveryNFrames(u32), // Update periodically
    Manual,            // Update when mark_dirty() called
    Continuous,        // Update every frame (expensive)
}
```

**Performance**:
- **Once**: No runtime cost
- **EveryNFrames(60)**: ~1/60th of capture cost per frame
- **Manual**: Cost only when triggered
- **Continuous**: Full capture cost every frame (avoid if possible)

## Memory and Performance

### Memory Usage Per Probe

For a 512×512 environment probe:
- Environment map: 512² × 6 faces × 8 bytes (FP16 RGBA) = 12 MB
- Irradiance map: 32² × 6 × 8 bytes = 25 KB
- Prefiltered map: 512² × 6 × 8 × 1.33 (mipmaps) = 16 MB
- **Total: ~28 MB per probe**

BRDF LUT: 512² × 2 bytes (RG) = 512 KB (shared across all probes)

### Resolution Trade-offs

| Resolution | Memory | Quality | Use Case |
|------------|--------|---------|----------|
| 128×128    | 1.5 MB | Low     | Distant probes, ambient only |
| 256×256    | 6 MB   | Medium  | Standard quality |
| 512×512    | 28 MB  | High    | Close-up reflections |
| 1024×1024  | 112 MB | Very High | Hero assets, showcases |

### Optimization Tips

1. **Use appropriate resolutions**: Start at 256×256
2. **Limit probe count**: 5-10 probes is usually sufficient
3. **Update frequency**: Use Once or EveryNFrames
4. **LOD system**: Lower resolution for distant probes
5. **Compression**: BC6H compression for HDR cubemaps (not yet implemented)

## Advanced Techniques

### Parallax Correction

For indoor environments, project reflection ray to room bounding box:

```glsl
vec3 parallax_correction(vec3 ray_dir, vec3 probe_pos, vec3 world_pos, vec3 box_min, vec3 box_max) {
    // Ray-AABB intersection
    vec3 first_plane_intersect = (box_max - world_pos) / ray_dir;
    vec3 second_plane_intersect = (box_min - world_pos) / ray_dir;
    vec3 furthest_plane = max(first_plane_intersect, second_plane_intersect);
    float distance = min(furthest_plane.x, min(furthest_plane.y, furthest_plane.z));
    
    vec3 intersection = world_pos + ray_dir * distance;
    return normalize(intersection - probe_pos);
}
```

### Probe Blending

Smooth transitions between probe influence regions:

```glsl
float weight = smoothstep(influence_radius, influence_radius * 0.5, distance_to_probe);
```

## Further Reading

- **Real-Time Rendering, 4th Edition** - Chapter 11: Image-Based Effects
- [Valve - Compute Shader Image Effects](https://developer.nvidia.com/gpugems/gpugems3/part-v-image-effects/chapter-31-fast-filter-width-estimates-image-filtering)
- [Epic Games - Real Shading in Unreal Engine 4](https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf)
- [LearnOpenGL - IBL Diffuse/Specular](https://learnopengl.com/PBR/IBL/Diffuse-irradiance)
- [Moving Frostbite to PBR](https://seblagarde.files.wordpress.com/2015/07/course_notes_moving_frostbite_to_pbr_v32.pdf)

## References

- `crates/praxis_graphics/src/ssao.rs` - SSAO implementation
- `crates/praxis_graphics/src/hdr.rs` - HDR system
- `crates/praxis_graphics/src/environment_probe.rs` - IBL implementation
- `examples/ssao_demo.rs` - SSAO example
- `examples/hdr_demo.rs` - HDR example
- `examples/environment_probe_demo.rs` - IBL example
- `docs/ENVIRONMENT_PROBES.md` - Detailed probe documentation
- `crates/praxis_graphics/HDR_RENDERING.md` - HDR technical details
