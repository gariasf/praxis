# Bloom Effect

The bloom effect creates a glowing halo around bright areas of the scene, simulating the way light bleeds in camera lenses and human vision. This document describes the implementation and usage of the bloom post-processing effect in Praxis.

## Overview

The bloom effect is implemented using a multi-pass technique:

1. **Brightness Extraction**: Extract pixels brighter than a threshold
2. **Gaussian Blur**: Apply separable blur for smooth glow (horizontal + vertical passes)
3. **Tone Mapping**: Combine bloom with original scene using HDR tone mapping

## Architecture

### Components

#### `BloomConfig`

Configuration struct for controlling bloom parameters:

```rust
pub struct BloomConfig {
    pub brightness_threshold: f32,  // Minimum brightness for bloom (default: 1.0)
    pub blur_iterations: u32,       // Number of blur passes (default: 5)
    pub exposure: f32,              // HDR exposure (default: 1.0)
    pub bloom_intensity: f32,       // Bloom blend strength (default: 0.3)
}
```

**Parameters:**

- **brightness_threshold**: Only pixels with brightness above this value contribute to bloom. Lower values create more bloom. Range: 0.1-5.0, typical: 0.8-1.5
- **blur_iterations**: Number of times to apply the blur. More iterations = wider, softer glow. Range: 1-10, typical: 3-7
- **exposure**: HDR exposure multiplier applied before tone mapping. Higher values brighten the scene. Range: 0.1-5.0, typical: 0.8-1.5
- **bloom_intensity**: How strongly the bloom is blended with the original scene. Range: 0.0-2.0, typical: 0.2-0.5

#### `BloomEffect`

Main bloom effect manager that orchestrates all passes:

```rust
pub struct BloomEffect {
    // Individual passes
    brightness_pass: BrightnessExtractionPass,
    blur_h_pass: GaussianBlurHorizontalPass,
    blur_v_pass: GaussianBlurVerticalPass,
    tone_map_pass: ToneMapPass,
    
    config: BloomConfig,
    render_pass: Arc<RenderPass>,
}
```

#### Individual Passes

- **`BrightnessExtractionPass`**: Extracts bright pixels using luminance calculation
- **`GaussianBlurHorizontalPass`**: Applies horizontal Gaussian blur with 5-tap kernel
- **`GaussianBlurVerticalPass`**: Applies vertical Gaussian blur with 5-tap kernel
- **`ToneMapPass`**: Combines scene and bloom using Reinhard tone mapping

## Implementation Details

### Brightness Extraction Shader

The brightness extraction pass uses a luminance-based threshold:

```glsl
float brightness = dot(color.rgb, vec3(0.2126, 0.7152, 0.0722));

if (brightness > threshold) {
    out_color = color;
} else {
    out_color = vec4(0.0);
}
```

This uses the standard luminance weights that account for human eye sensitivity.

### Separable Gaussian Blur

Instead of a full 2D blur kernel (which would require N² samples), we use two 1D kernels:

**Horizontal Pass:**
```glsl
float weights[5] = float[](0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);

vec3 result = texture(input, uv).rgb * weights[0];
for (int i = 1; i < 5; i++) {
    float offset = float(i) * texel_size.x;
    result += texture(input, uv + vec2(offset, 0.0)).rgb * weights[i];
    result += texture(input, uv - vec2(offset, 0.0)).rgb * weights[i];
}
```

**Vertical Pass:**
Same as horizontal but samples along Y axis.

This reduces complexity from O(N²) to O(2N) while producing identical results.

### Tone Mapping

The tone mapping pass combines the original scene with the bloom:

```glsl
vec3 hdr_color = scene_color + bloom_color * bloom_intensity;

// Reinhard tone mapping
vec3 tone_mapped = hdr_color * exposure / (hdr_color + vec3(1.0));

// Gamma correction
vec3 final_color = pow(tone_mapped, vec3(1.0 / 2.2));
```

**Reinhard Tone Mapping:** Compresses HDR values to [0,1] range while preserving relative brightness.

**Gamma Correction:** Converts from linear to sRGB color space for correct display.

## Usage

### Basic Usage

```rust
use praxis_graphics::{BloomEffect, BloomConfig, RenderTarget, RenderTargetPool};

// Create bloom effect
let config = BloomConfig::new()
    .with_brightness_threshold(1.0)
    .with_blur_iterations(5)
    .with_exposure(1.0)
    .with_bloom_intensity(0.3);

let mut bloom = BloomEffect::new(
    device,
    memory_allocator,
    Format::R8G8B8A8_UNORM,
    config,
)?;

// Create render target pool
let mut pool = RenderTargetPool::new(
    memory_allocator,
    bloom.render_pass().clone(),
    Format::R8G8B8A8_UNORM,
);

// In render loop:
// 1. Render scene to texture (scene_texture)
// 2. Apply bloom
bloom.apply(&mut builder, &scene_texture, &output_texture, &mut pool)?;

// 3. Present output_texture or blit to swapchain
```

### Runtime Configuration

```rust
// Update bloom parameters at runtime
bloom.config_mut().brightness_threshold = 1.2;
bloom.config_mut().bloom_intensity = 0.4;
bloom.update_config();

// Or replace entire config
let new_config = BloomConfig::new()
    .with_brightness_threshold(0.8)
    .with_blur_iterations(7);
bloom.set_config(new_config);
```

### Integration with Render Loop

The bloom effect requires rendering to offscreen targets:

```rust
// Render scene to texture instead of swapchain
let scene_target = pool.acquire([width, height])?;
render_scene_to_target(&scene_target)?;

// Apply bloom (reads scene_target, writes to output_target)
let output_target = pool.acquire([width, height])?;
bloom.apply(&mut builder, &scene_target, &output_target, &mut pool)?;

// Blit output_target to swapchain
blit_to_swapchain(&output_target)?;

// Release targets back to pool
pool.release_all();
```

## Performance Considerations

### Render Target Pooling

The `RenderTargetPool` reuses render targets to avoid expensive allocations:

```rust
// Pool automatically reuses matching targets
let target1 = pool.acquire([1920, 1080])?;  // Allocates new
let target2 = pool.acquire([1920, 1080])?;  // Allocates new
pool.release(target1);
let target3 = pool.acquire([1920, 1080])?;  // Reuses target1
```

### Blur Iterations

Each blur iteration requires 2 passes (horizontal + vertical):
- 1 iteration = 2 render passes
- 5 iterations = 10 render passes
- 10 iterations = 20 render passes

More iterations = smoother, wider bloom, but higher cost.

### Resolution Scaling

Consider rendering bloom at lower resolution for better performance:

```rust
// Render bloom at half resolution
let bloom_width = scene_width / 2;
let bloom_height = scene_height / 2;

let downsampled = pool.acquire([bloom_width, bloom_height])?;
// ... downsample scene to downsampled ...

bloom.apply(&mut builder, &downsampled, &bloom_output, &mut pool)?;

// ... upsample bloom_output back to full resolution ...
```

This can reduce bloom cost by ~75% with minimal visual difference.

## Visual Quality Tips

### Bright Objects

For objects that should bloom strongly, use emissive materials:

```rust
let material = MaterialProperties::new()
    .with_emissive([2.0, 1.5, 0.5, 1.0])  // Bright emissive color
    .with_metallic(0.0)
    .with_roughness(1.0);
```

### Threshold Tuning

- **Low threshold (0.5-0.8)**: More bloom, dreamlike, can wash out scene
- **Medium threshold (0.8-1.2)**: Balanced, natural looking
- **High threshold (1.2-2.0)**: Less bloom, only very bright objects glow

### Intensity Tuning

- **Low intensity (0.1-0.2)**: Subtle glow, realistic
- **Medium intensity (0.3-0.5)**: Noticeable bloom, stylized
- **High intensity (0.6-1.0+)**: Strong glow, dreamlike/fantasy

### Blur Iterations

- **Few iterations (1-3)**: Sharp, tight glow
- **Medium iterations (4-7)**: Smooth, natural glow
- **Many iterations (8-10)**: Very wide, soft glow

## Example Configurations

### Realistic Sun Glow
```rust
BloomConfig::new()
    .with_brightness_threshold(1.5)
    .with_blur_iterations(5)
    .with_exposure(1.0)
    .with_bloom_intensity(0.3)
```

### Sci-Fi Neon
```rust
BloomConfig::new()
    .with_brightness_threshold(0.8)
    .with_blur_iterations(7)
    .with_exposure(1.2)
    .with_bloom_intensity(0.6)
```

### Subtle Enhancement
```rust
BloomConfig::new()
    .with_brightness_threshold(1.2)
    .with_blur_iterations(3)
    .with_exposure(1.0)
    .with_bloom_intensity(0.2)
```

### Dream Sequence
```rust
BloomConfig::new()
    .with_brightness_threshold(0.5)
    .with_blur_iterations(10)
    .with_exposure(1.5)
    .with_bloom_intensity(0.8)
```

## Technical Notes

### HDR Workflow

The bloom effect expects HDR (high dynamic range) input where colors can exceed 1.0. The tone mapping pass converts this to LDR (low dynamic range) for display.

If your scene doesn't render in HDR:
- Use emissive materials with values > 1.0 for bloom sources
- The tone mapping will still work but won't compress as much dynamic range

### Gamma Correction

The tone mapping pass applies gamma correction (sRGB conversion). If your swapchain is already in sRGB format, you may need to disable gamma correction or adjust the gamma value.

### Memory Usage

Each render target uses GPU memory:
- 1920×1080 RGBA8 = ~8 MB
- Bloom typically needs 3-10 render targets depending on blur iterations
- Pool reuse significantly reduces allocation overhead

### Shader Compilation

Shaders are compiled at build time using `vulkano-shaders`. If you modify the shader files, rebuild the project to see changes.
