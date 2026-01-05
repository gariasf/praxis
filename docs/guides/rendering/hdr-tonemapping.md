# HDR and Tone Mapping

High Dynamic Range (HDR) rendering enables more realistic lighting by using floating-point precision to represent colors beyond the [0,1] range. This guide covers HDR rendering and tone mapping in Praxis.

## Overview

The HDR pipeline consists of three stages:

1. **Render to HDR target** - Use floating-point format for lighting calculations
2. **Calculate exposure** - Determine appropriate brightness level (automatic or manual)
3. **Apply tone mapping** - Convert HDR to displayable LDR range [0,1]

## Why HDR?

### LDR Limitations

**Low Dynamic Range (LDR)** limits color values to [0, 1]:
- Cannot represent bright lights properly (sun gets clamped to 1.0)
- Loses information when values exceed 1.0
- Poor bloom and glow effects
- Unrealistic exposure handling
- No proper overbright emissive materials

### HDR Advantages

**High Dynamic Range (HDR)** uses floating-point values:
- Colors can exceed 1.0 (bright lights, sun, emissive surfaces)
- Preserves lighting information across wide range
- Enables realistic tone mapping and exposure
- Better bloom and glow effects
- Proper representation of real-world luminance

### Real-World Luminance

Human vision can perceive a dynamic range of ~10,000,000:1 (from starlight to bright sunlight).

**Examples**:
- Starlight: 0.001 lux
- Indoor lighting: 100-500 lux
- Overcast day: 10,000 lux
- Direct sunlight: 100,000 lux
- Sun surface (relative): 1,000,000,000 lux

**LDR**: All values clamped to [0, 1] – sun is as bright as white paper  
**HDR**: Values represent true brightness ratios – sun is 1000× brighter than indoor lights

## HDR Rendering Pipeline

### Stage 1: HDR Scene Rendering

Render scene to floating-point render target:

```rust
use praxis_graphics::HdrRenderTarget;

// Create HDR render pass
let hdr_render_pass = render_context.create_hdr_render_pass()?;

// Create HDR render target
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
- Range: Approximately -65504 to +65504
- Sufficient precision for HDR
- Memory: 8 bytes per pixel (vs 4 bytes for RGBA8)

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
// Calculate luminance from RGB color
float luminance(vec3 color) {
    return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

vec3 hdr_color = texture(u_hdr_texture, uv).rgb;
float luma = luminance(hdr_color);
// Average across all pixels for scene luminance
```

**Methods**:
1. **Compute shader reduction**: Parallel sum on GPU (recommended)
2. **Mipmapped luminance**: Downsample luminance to 1×1 texture
3. **CPU readback**: Download and compute on CPU (slow, avoid)
4. **Approximation**: Use fixed value based on scene knowledge

**Typical values**:
- Dark indoor: 0.05
- Indoor: 0.1-0.3
- Outdoor (overcast): 0.5
- Outdoor (sunny): 1.0-2.0
- Very bright: 5.0+

### Stage 3: Exposure Calculation

#### Manual Exposure

Fixed exposure value for artistic control:

```rust
use praxis_graphics::ExposureMode;

tone_mapper.set_exposure_mode(ExposureMode::Manual { 
    exposure: 1.0  // Adjust as needed
});
```

**Use cases**:
- Cinematic sequences with controlled lighting
- Scenes where automatic adaptation would be distracting
- Fine-tuned artistic look

#### Automatic Exposure

Dynamic exposure based on scene luminance (simulates eye/camera adaptation):

```rust
tone_mapper.set_exposure_mode(ExposureMode::Automatic { 
    speed: 2.0  // Adaptation speed
});
```

**Algorithm**:
```rust
struct ExposureCalculator {
    current_exposure: f32,
    target_exposure: f32,
    key_value: f32,        // Target middle gray (0.18)
    min_exposure: f32,      // Lower bound (0.1)
    max_exposure: f32,      // Upper bound (10.0)
    adaptation_speed: f32,  // How fast to adapt
}

impl ExposureCalculator {
    fn calculate(&mut self, average_luminance: f32, delta_time: f32) -> f32 {
        // Calculate target exposure from scene luminance
        self.target_exposure = self.key_value / (average_luminance + 0.001);
        self.target_exposure = self.target_exposure.clamp(
            self.min_exposure, 
            self.max_exposure
        );
        
        // Smoothly adapt current exposure to target
        let adaptation_rate = 1.0 - f32::exp(-self.adaptation_speed * delta_time);
        self.current_exposure = lerp(
            self.current_exposure, 
            self.target_exposure, 
            adaptation_rate
        );
        
        self.current_exposure
    }
}
```

**Parameters**:
- **key_value**: Target brightness for middle tones (default: 0.18 = 18% gray, photographic standard)
- **adaptation_speed**: 
  - 1.0 = slow, realistic eye adaptation
  - 2.0 = medium, responsive (recommended)
  - 5.0 = fast, game-like
- **min/max_exposure**: Prevents extreme values (typically 0.1 to 10.0)

### Stage 4: Tone Mapping

Convert HDR values [0, ∞) to displayable LDR range [0, 1]:

```glsl
vec3 tone_map(vec3 hdr_color, float exposure) {
    // Apply exposure
    vec3 exposed = hdr_color * exposure;
    
    // Apply tone mapping operator
    vec3 ldr_color = tone_map_operator(exposed);
    
    // Gamma correction (linear to sRGB)
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
- Smooth compression of HDR values
- Preserves hue
- Can look flat in very bright scenes

**When to use**: 
- Fast iteration and prototyping
- Simple scenes without extreme brightness
- Educational purposes
- Mobile devices

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
- Industry standard (film and AAA games)
- Cinematic look with excellent color preservation
- Proper contrast in highlights and shadows
- Slightly more expensive than Reinhard (negligible)

**When to use**: 
- Production-quality games
- Realistic rendering
- Scenes with high dynamic range
- Default choice for most projects

**Games using ACES**: *The Last of Us*, *Uncharted 4*, *Call of Duty*, *Red Dead Redemption 2*

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
- High contrast with strong toe and shoulder
- Dramatic look
- Popular in action games
- Similar cost to ACES

**When to use**: 
- High-contrast scenes
- Dramatic lighting
- Stylized games
- Action-oriented titles

### Comparison

| Operator | Speed | Contrast | Color | Best For |
|----------|-------|----------|-------|----------|
| Reinhard | Fast | Low | Good | Prototyping, mobile |
| ACES | Medium | Medium | Excellent | Production, realism |
| Uncharted 2 | Medium | High | Good | Dramatic, stylized |

**Visual Comparison**:
- **Reinhard**: Even, balanced, can wash out bright areas
- **ACES**: Cinematic, natural color transitions, industry standard
- **Uncharted 2**: Punchy, high contrast, dramatic highlights

## Complete Example

```rust
use praxis_graphics::{
    HdrRenderTarget, ToneMapper, ToneMappingOperator, ExposureMode,
};

struct HdrRenderer {
    hdr_target: HdrRenderTarget,
    tone_mapper: ToneMapper,
}

impl HdrRenderer {
    fn new(render_context: &RenderContext) -> Result<Self> {
        // Create HDR render pass and target
        let hdr_render_pass = render_context.create_hdr_render_pass()?;
        let hdr_target = HdrRenderTarget::new(
            render_context.memory_allocator.clone(),
            hdr_render_pass,
            [1920, 1080],
        )?;

        // Create tone mapper with ACES operator
        let mut tone_mapper = ToneMapper::new(
            render_context.device.clone(),
            render_context.memory_allocator.clone(),
            vulkano::format::Format::R8G8B8A8_UNORM,
            ToneMappingOperator::ACES,
        )?;

        // Enable automatic exposure
        tone_mapper.set_exposure_mode(ExposureMode::Automatic { 
            speed: 2.0 
        });

        Ok(Self { hdr_target, tone_mapper })
    }

    fn render_frame(
        &mut self, 
        command_buffer: &mut AutoCommandBufferBuilder,
        output_framebuffer: Arc<Framebuffer>,
        delta_time: f32,
        average_luminance: f32,
    ) -> Result<()> {
        // 1. Render scene to HDR target
        // ... scene rendering code ...

        // 2. Apply tone mapping (HDR to LDR conversion)
        self.tone_mapper.apply(
            command_buffer,
            &self.hdr_target,
            output_framebuffer,
            [1920, 1080],
            average_luminance,
            delta_time,
        )?;
        
        Ok(())
    }
}
```

### Runtime Operator Switching

```rust
// Allow users to choose tone mapping operator
match user_preference {
    "reinhard" => tone_mapper.set_operator(ToneMappingOperator::Reinhard),
    "aces" => tone_mapper.set_operator(ToneMappingOperator::ACES),
    "uncharted2" => tone_mapper.set_operator(ToneMappingOperator::Uncharted2),
    _ => {},
}
```

## Integration with Other Systems

### With Bloom

HDR significantly improves bloom quality:

```rust
// 1. Render scene to HDR target
render_scene(&hdr_target)?;

// 2. Extract bright pixels (threshold in HDR space)
bloom.extract_bright(&hdr_target, threshold: 1.0)?;

// 3. Blur bright pixels
bloom.blur()?;

// 4. Combine bloom with HDR scene
bloom.composite(&hdr_target)?;

// 5. Tone map the bloomed HDR result
tone_mapper.apply(&hdr_target, &output_framebuffer)?;
```

See [post-processing.md](post-processing.md) for bloom details.

### With Deferred Rendering

```rust
// Lighting pass outputs to HDR target
deferred_renderer.render_lighting(hdr_target.framebuffer(), ...)?;

// Tone map the lit HDR result
tone_mapper.apply(
    command_buffer,
    &hdr_target,
    output_framebuffer,
    ...
)?;
```

See [deferred-rendering.md](deferred-rendering.md) for details.

### With Environment Probes

IBL works naturally with HDR:

```rust
// Environment probes capture HDR data
probe.capture_hdr(&scene)?;

// IBL contributes HDR lighting
vec3 ibl_color = sample_environment_probe(...); // Can exceed 1.0

// Tone mapping converts final result to LDR
```

See [environment-probes.md](environment-probes.md) for details.

## Best Practices

### 1. Light Intensity Values

Set realistic light intensities for HDR:

```rust
// Directional light (sun) - much brighter than LDR
directional_light.intensity = 5.0;

// Point light (lamp)
point_light.intensity = 50.0;

// Emissive materials
material.emissive = 2.0;  // Can exceed 1.0 for glow
```

**Rule of thumb**: Think in physical units (candelas, lumens) if possible.

### 2. Color Grading

Apply color grading before tone mapping for better control:

```glsl
// Before tone mapping
color = color * color_grading.contrast;
color = color + vec3(color_grading.brightness);
color = mix(vec3(dot(color, vec3(0.333))), color, color_grading.saturation);

// Then tone map
color = tone_map(color, exposure);
```

### 3. Monitor Exposure in Debug UI

```rust
// Display current exposure value for debugging
ui.label(format!("Exposure: {:.2}", tone_mapper.current_exposure()));
ui.label(format!("Avg Luminance: {:.3}", average_luminance));

// Exposure override slider for testing
if ui.button("Manual Exposure") {
    ui.slider("Exposure", &mut manual_exposure, 0.1..=10.0);
}
```

### 4. Adaptation Speed

Tune adaptation speed for different scenarios:

```rust
// Slow adaptation (realistic, cinematic)
tone_mapper.set_exposure_mode(ExposureMode::Automatic { speed: 1.0 });

// Fast adaptation (responsive gameplay)
tone_mapper.set_exposure_mode(ExposureMode::Automatic { speed: 5.0 });

// Disable adaptation (instant adjustment)
tone_mapper.set_exposure_mode(ExposureMode::Automatic { speed: 100.0 });
```

## Performance

### Memory Usage

HDR render targets use 2× the memory of LDR:
- **LDR (R8G8B8A8)**: 4 bytes per pixel
- **HDR (R16G16B16A16_SFLOAT)**: 8 bytes per pixel

**Examples**:
- 1920×1080: LDR ~8.3 MB, HDR ~16.6 MB
- 3840×2160: LDR ~33 MB, HDR ~66 MB

### Rendering Cost

Tone mapping is a full-screen post-process pass:
- **Complexity**: O(pixels)
- **Typical cost**: <1ms at 1080p on modern GPUs
- **Operator cost**: All operators have similar performance (negligible difference)

### Bandwidth

Reading/writing HDR buffers uses 2× bandwidth:
```
LDR: 1920×1080 × 60fps × 4 bytes = ~475 MB/s
HDR: 1920×1080 × 60fps × 8 bytes = ~950 MB/s
```

**Mitigation**: Modern GPUs handle this easily; bandwidth is rarely the bottleneck.

## Troubleshooting

| Problem | Cause | Solution |
|---------|-------|----------|
| Scene too dark | Low exposure | Increase exposure or key value |
| Scene too bright | High exposure | Decrease exposure or key value |
| Washed out colors | Wrong operator or high exposure | Try ACES, check gamma (2.2) |
| Flickering (auto) | Fast adaptation | Decrease adaptation speed |
| No bloom | LDR pipeline | Use HDR before bloom |
| Banding artifacts | 8-bit output | Ensure sRGB framebuffer |

## Examples

```bash
# HDR rendering demo
cargo run --example hdr_demo

# Advanced lighting with HDR
cargo run --example advanced_lighting_demo

# Environment probes with HDR
cargo run --example environment_probe_demo
```

## See Also

- [Forward Rendering](forward-rendering.md) - Basic rendering pipeline
- [Deferred Rendering](deferred-rendering.md) - Multi-pass rendering
- [Post-Processing](post-processing.md) - Bloom and other effects
- [Environment Probes](environment-probes.md) - Image-based lighting

## References

- [Reinhard et al. (2002). "Photographic Tone Reproduction for Digital Images"](http://www.cmap.polytechnique.fr/~peyre/cours/x2005signal/hdr_photographic.pdf)
- [Academy Color Encoding System (ACES)](https://www.oscars.org/science-technology/sci-tech-projects/aces)
- [Hable, John (2010). "Uncharted 2: HDR Lighting"](http://filmicworlds.com/blog/filmic-tonemapping-operators/)
- [Karis, Brian (2013). "Real Shading in Unreal Engine 4"](https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf)
