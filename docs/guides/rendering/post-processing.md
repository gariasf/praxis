# Post-Processing Effects

Post-processing applies full-screen effects after the main render pass to enhance visual quality. This guide covers the post-processing framework and available effects in Praxis.

## Overview

Post-processing effects are chained together, each reading from the previous result:

```text
Scene → Bloom → DOF → Chromatic Aberration → Vignette → Film Grain → Output
```

## Post-Processing Framework

### Core Components

#### PostProcessPass Trait

```rust
pub trait PostProcessPass: Send + Sync {
    fn execute(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        input: &RenderTarget,
        output: &RenderTarget,
    ) -> Result<()>;

    fn name(&self) -> &str;
}
```

#### RenderTargetPool

Manages render target lifecycle efficiently:

```rust
let mut pool = RenderTargetPool::new(memory_allocator, render_pass);

// Acquire targets (reuses from pool when available)
let target = pool.acquire([width, height])?;

// ... use target ...

// Return to pool for reuse
pool.release(target);
```

#### PostProcessChain

Orchestrates multiple passes:

```rust
let mut chain = PostProcessChain::new(device, queue);
chain.add_pass(Box::new(bloom_pass));
chain.add_pass(Box::new(vignette_pass));
chain.process(&input, &output, &mut pool)?;
```

## Built-in Effects

### Bloom

Creates a glowing halo around bright areas, simulating light bleed in camera lenses and human vision.

#### Algorithm

1. **Brightness Extraction**: Extract pixels brighter than threshold
2. **Gaussian Blur**: Apply separable blur (horizontal + vertical passes)
3. **Composite**: Combine bloom with original scene

#### Configuration

```rust
use praxis_graphics::{BloomEffect, BloomConfig};

let config = BloomConfig {
    brightness_threshold: 1.0,  // Minimum brightness for bloom
    blur_iterations: 5,         // Number of blur passes
    exposure: 1.0,              // HDR exposure
    bloom_intensity: 0.3,       // Bloom blend strength
};

let mut bloom = BloomEffect::new(
    device,
    memory_allocator,
    Format::R8G8B8A8_UNORM,
    config,
)?;

// Apply bloom
bloom.apply(&mut builder, &scene_texture, &output_texture, &mut pool)?;
```

**Parameters**:

| Parameter | Range | Description |
|-----------|-------|-------------|
| `brightness_threshold` | 0.1-5.0 | Only pixels above this brightness bloom |
| `blur_iterations` | 1-10 | More = wider, softer glow |
| `exposure` | 0.1-5.0 | HDR exposure multiplier |
| `bloom_intensity` | 0.0-2.0 | Bloom blend strength |

**Example Configurations**:

```rust
// Realistic sun glow
BloomConfig {
    brightness_threshold: 1.5,
    blur_iterations: 5,
    exposure: 1.0,
    bloom_intensity: 0.3,
}

// Sci-fi neon
BloomConfig {
    brightness_threshold: 0.8,
    blur_iterations: 7,
    exposure: 1.2,
    bloom_intensity: 0.6,
}

// Subtle enhancement
BloomConfig {
    brightness_threshold: 1.2,
    blur_iterations: 3,
    exposure: 1.0,
    bloom_intensity: 0.2,
}
```

### Depth-of-Field (DoF)

Simulates camera lens focus by blurring objects that are out of focus.

#### Features

- Circle of Confusion (CoC) calculation
- Bokeh blur using Poisson disk sampling
- Depth-aware sampling prevents foreground/background bleeding
- Configurable focus distance, range, and blur radius

#### Configuration

```rust
use praxis_graphics::{DepthOfFieldPass, DofConfig};

let mut dof_pass = DepthOfFieldPass::new(
    device.clone(),
    memory_allocator.clone(),
    Format::R8G8B8A8_UNORM,
    DofConfig {
        focus_distance: 10.0,  // Focus at 10 units
        focus_range: 5.0,      // Sharp range ±5 units
        bokeh_radius: 8.0,     // Max blur radius in pixels
        aperture: 2.8,         // f/2.8 aperture
    },
)?;

// Apply DoF (requires depth texture)
dof_pass.execute_with_depth(
    &mut builder,
    &input_target,
    &depth_texture,
    &depth_sampler,
    &output_target,
)?;
```

**Parameters**:

| Parameter | Range | Description |
|-----------|-------|-------------|
| `focus_distance` | 5.0-100.0 | Distance to focal plane (world units) |
| `focus_range` | 1.0-20.0 | Range around focal plane that stays sharp |
| `bokeh_radius` | 2.0-16.0 | Maximum blur radius for out-of-focus areas |
| `aperture` | 1.4-22.0 | Aperture size (f-number) |

### Motion Blur

Creates realistic motion blur based on per-pixel velocity information.

#### Features

- Velocity buffer rendering
- Sample accumulation along motion paths
- Configurable shutter angle
- Adaptive sampling (skips stationary pixels)

#### Configuration

```rust
use praxis_graphics::{MotionBlurPass, MotionBlurConfig, VelocityBufferRenderer};

// Create velocity buffer renderer
let velocity_renderer = VelocityBufferRenderer::new(
    device.clone(),
    memory_allocator.clone(),
)?;

// Create motion blur pass
let mut motion_blur_pass = MotionBlurPass::new(
    device.clone(),
    memory_allocator.clone(),
    Format::R8G8B8A8_UNORM,
    MotionBlurConfig {
        intensity: 1.0,
        sample_count: 16,
        shutter_angle: 180.0,
        max_blur_radius: 32.0,
    },
)?;

// Generate velocity buffer
let velocity_buffer = velocity_renderer.create_buffer(width, height)?;
velocity_renderer.render(
    &mut builder,
    &velocity_buffer,
    current_mvp,
    previous_mvp,
    &meshes,
)?;

// Apply motion blur
motion_blur_pass.execute_with_velocity(
    &mut builder,
    &input_target,
    &velocity_buffer.image_view,
    &velocity_sampler,
    &output_target,
)?;
```

**Parameters**:

| Parameter | Range | Description |
|-----------|-------|-------------|
| `intensity` | 0.5-2.0 | Blur intensity multiplier |
| `sample_count` | 8-32 | Samples along motion vector |
| `shutter_angle` | 90.0-360.0 | Simulated camera shutter angle |
| `max_blur_radius` | 16.0-64.0 | Maximum blur radius in pixels |

### Chromatic Aberration

Simulates lens color fringing, creating rainbow-like distortion at high-contrast edges.

#### Features

- Radial distortion (increases toward edges)
- Separate R/B channel offsets
- Optional directional control
- Distance-based falloff

#### Configuration

```rust
use praxis_graphics::{ChromaticAberrationPass, ChromaticAberrationConfig};

let mut chromatic_aberration_pass = ChromaticAberrationPass::new(
    device.clone(),
    memory_allocator.clone(),
    Format::R8G8B8A8_UNORM,
    ChromaticAberrationConfig {
        intensity: 0.003,
        radial_falloff: 2.0,
        direction: [0.0, 0.0],  // Pure radial
        red_offset: 1.0,
        blue_offset: 1.0,
    },
)?;

chromatic_aberration_pass.execute(&mut builder, &input_target, &output_target)?;
```

**Parameters**:

| Parameter | Range | Description |
|-----------|-------|-------------|
| `intensity` | 0.001-0.01 | Overall aberration strength |
| `radial_falloff` | 1.0-3.0 | Exponent for distance falloff |
| `direction` | [-1,-1]-[1,1] | Optional directional bias |
| `red_offset` | 0.5-1.5 | Red channel offset multiplier |
| `blue_offset` | 0.5-1.5 | Blue channel offset multiplier |

### Vignette

Darkens the edges of the image, drawing attention to the center.

#### Features

- Adjustable shape (rectangular to circular)
- Customizable center point
- Smooth gradient transition
- Intensity control

#### Configuration

```rust
use praxis_graphics::{VignettePass, VignetteConfig};

let mut vignette_pass = VignettePass::new(
    device.clone(),
    memory_allocator.clone(),
    Format::R8G8B8A8_UNORM,
    VignetteConfig {
        intensity: 0.8,
        smoothness: 0.5,
        roundness: 1.0,
        center: [0.5, 0.5],
    },
)?;

vignette_pass.execute(&mut builder, &input_target, &output_target)?;
```

**Parameters**:

| Parameter | Range | Description |
|-----------|-------|-------------|
| `intensity` | 0.3-1.0 | Darkness at edges |
| `smoothness` | 0.2-0.8 | Gradient transition width |
| `roundness` | 0.0-1.0 | Shape (0=rectangular, 1=circular) |
| `center` | [0,0]-[1,1] | Vignette center point (normalized) |

### Film Grain

Adds procedural grain noise to simulate film stock.

#### Features

- Procedural generation (real-time noise)
- Luminance-based intensity
- Animated grain
- Configurable particle size

#### Configuration

```rust
use praxis_graphics::{FilmGrainPass, FilmGrainConfig};

let mut film_grain_pass = FilmGrainPass::new(
    device.clone(),
    memory_allocator.clone(),
    Format::R8G8B8A8_UNORM,
    FilmGrainConfig {
        intensity: 0.05,
        size: 2.0,
        luminance_impact: 0.5,
        time: 0.0,  // Update each frame
    },
)?;

// Update time each frame for animation
film_grain_pass.set_config(FilmGrainConfig {
    time: elapsed_time_ms,
    ..film_grain_pass.config()
});

film_grain_pass.execute(&mut builder, &input_target, &output_target)?;
```

**Parameters**:

| Parameter | Range | Description |
|-----------|-------|-------------|
| `intensity` | 0.01-0.1 | Grain strength |
| `size` | 1.0-4.0 | Grain particle size |
| `luminance_impact` | 0.0-1.0 | How grain varies with brightness |
| `time` | 0.0-∞ | Time for animation (milliseconds) |

## Chaining Effects

Combine multiple effects using `PostProcessChain`:

```rust
use praxis_graphics::PostProcessChain;

let mut chain = PostProcessChain::new();

// Add effects in order (order matters!)
chain.add_pass(Box::new(bloom_pass));
chain.add_pass(Box::new(depth_of_field_pass));
chain.add_pass(Box::new(motion_blur_pass));
chain.add_pass(Box::new(chromatic_aberration_pass));
chain.add_pass(Box::new(vignette_pass));
chain.add_pass(Box::new(film_grain_pass));

// Apply entire chain
let output = chain.process(&input_target, &mut render_target_pool)?;
```

**Effect Ordering Best Practices**:

1. **Bloom** - First, works on bright pixels
2. **Depth-of-Field** - Simulates lens focus
3. **Motion Blur** - Adds motion
4. **Chromatic Aberration** - Lens distortion
5. **Vignette** - Frame darkening
6. **Film Grain** - Final texture overlay

## Performance

### Typical Frame Time Impact (1080p)

| Effect | Cost | Notes |
|--------|------|-------|
| Bloom | 2-4ms | Scales with blur iterations |
| Depth-of-Field | 2-4ms | Scales with bokeh radius |
| Motion Blur | 1-3ms | Scales with sample count |
| Chromatic Aberration | 0.2-0.5ms | Minimal overhead |
| Vignette | 0.1-0.2ms | Simple per-pixel operation |
| Film Grain | 0.2-0.4ms | Procedural noise |

*Performance measured on mid-range GPU (RTX 3060)*

### Optimization Tips

**1. Resolution Scaling**

Apply expensive effects at lower resolution:

```rust
// Render bloom at half resolution
let bloom_width = scene_width / 2;
let bloom_height = scene_height / 2;

let downsampled = pool.acquire([bloom_width, bloom_height])?;
// Downsample, apply bloom, then upsample
```

**2. Conditional Application**

Skip effects when intensity is zero:

```rust
if bloom_config.bloom_intensity > 0.01 {
    bloom.apply(...)?;
}
```

**3. Effect Ordering**

Apply cheaper effects (vignette, grain) last.

**4. Shared Resources**

Reuse render targets through `RenderTargetPool` for 100× fewer allocations.

**5. Sample Count Reduction**

Use fewer samples on slower hardware:

```rust
// Adjust based on performance target
let sample_count = if is_low_end { 8 } else { 16 };
```

## Integration

### With HDR Rendering

Apply post-processing after HDR, before tone mapping:

```rust
// 1. Render scene to HDR
render_scene(&hdr_target)?;

// 2. Apply effects in HDR space
bloom.apply(&hdr_target, &bloomed_hdr)?;

// 3. Tone map to LDR
tone_mapper.apply(&bloomed_hdr, &output)?;
```

See [hdr-tonemapping.md](hdr-tonemapping.md) for details.

### With Deferred Rendering

Post-processing works naturally with deferred:

```rust
// 1. Deferred rendering outputs to target
deferred_renderer.render(output_target, ...)?;

// 2. Apply post-processing chain
post_process_chain.process(&output_target, &final_output, &mut pool)?;
```

See [deferred-rendering.md](deferred-rendering.md) for details.

## Best Practices

### Artistic Guidelines

1. **Subtlety**: Less is more - subtle effects are more realistic
2. **Context**: Match effects to scene mood (DOF for cinematics, motion blur for action)
3. **Consistency**: Maintain consistent effect intensity throughout
4. **Performance**: Test on target hardware and adjust quality

### Technical Guidelines

1. **Depth Access**: DoF requires accurate depth buffer
2. **Velocity Buffer**: Motion blur needs previous frame MVP matrices
3. **HDR Pipeline**: Apply effects before tone mapping when possible
4. **Gamma Correction**: Ensure proper color space handling

## Troubleshooting

### Bloom Issues

**Problem**: No bloom visible
- Verify threshold is appropriate for scene brightness
- Check that emissive materials have values > 0
- Ensure bloom intensity > 0

**Problem**: Too much bloom
- Increase brightness threshold
- Reduce bloom intensity
- Decrease blur iterations

### Depth-of-Field Issues

**Problem**: Wrong objects in focus
- Verify depth buffer format and precision
- Check focus distance matches world scale

**Problem**: Harsh transitions
- Increase focus range
- Reduce bokeh radius

### Motion Blur Issues

**Problem**: No visible blur
- Verify velocity buffer is generated correctly
- Check previous frame matrices are stored

**Problem**: Excessive blur
- Reduce intensity
- Increase max_blur_radius clamp

## Examples

```bash
# Bloom effect demo
cargo run --example bloom_demo

# Cinematic post-processing
cargo run --example cinematic_post_processing_demo

# Complete post-processing chain
cargo run --example post_process_demo
```

## See Also

- [HDR and Tone Mapping](hdr-tonemapping.md) - High dynamic range rendering
- [Forward Rendering](forward-rendering.md) - Basic rendering pipeline
- [Deferred Rendering](deferred-rendering.md) - Multi-pass rendering

## References

- [GPU Gems 3: Chapter 28 - Practical Post-Process Depth of Field](https://developer.nvidia.com/gpugems/gpugems3/part-iv-image-effects/chapter-28-practical-post-process-depth-field)
- [Siggraph 2014: Next Generation Post Processing in Call of Duty](https://www.iryoku.com/next-generation-post-processing-in-call-of-duty-advanced-warfare/)
- [Real-Time Rendering, 4th Edition](http://www.realtimerendering.com/) - Chapter 12: Image-Space Effects
