# Cinematic Post-Processing Effects

This document describes the advanced cinematic post-processing effects available in Praxis, including depth-of-field, motion blur, chromatic aberration, vignette, and film grain.

## Overview

Cinematic post-processing effects enhance the visual presentation of rendered scenes by simulating real camera lens characteristics and film properties. These effects are essential for achieving photorealistic or stylized cinematic looks in games and interactive applications.

## Available Effects

### 1. Depth-of-Field (DoF)

Simulates realistic camera lens focus by blurring objects that are out of focus, creating a shallow depth-of-field effect commonly seen in photography and cinematography.

#### Features
- **Circle of Confusion (CoC)**: Physically-based calculation of blur amount based on distance from focal plane
- **Bokeh Blur**: Poisson disk sampling for realistic lens bokeh shapes
- **Depth-aware Sampling**: Prevents bleeding between foreground and background
- **Configurable Parameters**: Focus distance, focus range, bokeh radius, aperture

#### Usage

```rust
use praxis_graphics::{DepthOfFieldPass, DofConfig};

// Create DoF pass
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

// Apply DoF effect (requires depth texture)
dof_pass.execute_with_depth(
    &mut builder,
    &input_target,
    &depth_texture,
    &depth_sampler,
    &output_target,
)?;
```

#### Configuration

| Parameter | Type | Description | Typical Range |
|-----------|------|-------------|---------------|
| `focus_distance` | f32 | Distance to focal plane (world units) | 5.0 - 100.0 |
| `focus_range` | f32 | Range around focal plane that stays sharp | 1.0 - 20.0 |
| `bokeh_radius` | f32 | Maximum blur radius for out-of-focus areas | 2.0 - 16.0 |
| `aperture` | f32 | Aperture size (f-number) | 1.4 - 22.0 |

### 2. Motion Blur

Creates realistic motion blur based on per-pixel velocity information, simulating the way fast-moving objects appear blurred due to camera shutter exposure time.

#### Features
- **Velocity Buffer**: Per-pixel screen-space motion vectors
- **Sample Accumulation**: Accumulates samples along motion paths
- **Shutter Angle Simulation**: Configurable shutter angle (0-360 degrees)
- **Adaptive Sampling**: Skips blur for stationary pixels

#### Usage

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

// Generate velocity buffer (before main rendering)
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

#### Configuration

| Parameter | Type | Description | Typical Range |
|-----------|------|-------------|---------------|
| `intensity` | f32 | Blur intensity multiplier | 0.5 - 2.0 |
| `sample_count` | i32 | Number of samples along motion vector | 8 - 32 |
| `shutter_angle` | f32 | Simulated camera shutter angle (degrees) | 90.0 - 360.0 |
| `max_blur_radius` | f32 | Maximum blur radius in pixels | 16.0 - 64.0 |

### 3. Chromatic Aberration

Simulates lens color fringing caused by imperfect lenses failing to focus all colors at the same convergence point, creating a rainbow-like distortion at high-contrast edges.

#### Features
- **Radial Distortion**: Color fringing increases toward screen edges
- **Separate Channel Offsets**: Independent control of red and blue channels
- **Directional Control**: Optional directional distortion
- **Distance-based Falloff**: Configurable radial falloff exponent

#### Usage

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
        ..Default::default()
    },
)?;

// Apply chromatic aberration
chromatic_aberration_pass.execute(&mut builder, &input_target, &output_target)?;
```

#### Configuration

| Parameter | Type | Description | Typical Range |
|-----------|------|-------------|---------------|
| `intensity` | f32 | Overall aberration strength | 0.001 - 0.01 |
| `radial_falloff` | f32 | Exponent for distance falloff | 1.0 - 3.0 |
| `direction` | [f32; 2] | Optional directional bias (x, y) | [-1.0, -1.0] - [1.0, 1.0] |
| `red_offset` | f32 | Red channel offset multiplier | 0.5 - 1.5 |
| `blue_offset` | f32 | Blue channel offset multiplier | 0.5 - 1.5 |

### 4. Vignette

Darkens the edges of the image, drawing the viewer's attention to the center and creating a cinematic framing effect.

#### Features
- **Shape Control**: Adjustable roundness from rectangular to circular
- **Center Control**: Customizable vignette center point
- **Smooth Transition**: Configurable gradient smoothness
- **Intensity Control**: Adjustable darkness at edges

#### Usage

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
        ..Default::default()
    },
)?;

// Apply vignette
vignette_pass.execute(&mut builder, &input_target, &output_target)?;
```

#### Configuration

| Parameter | Type | Description | Typical Range |
|-----------|------|-------------|---------------|
| `intensity` | f32 | Darkness at edges | 0.3 - 1.0 |
| `smoothness` | f32 | Gradient transition width | 0.2 - 0.8 |
| `roundness` | f32 | Shape (0=rectangular, 1=circular) | 0.0 - 1.0 |
| `center` | [f32; 2] | Vignette center point (normalized) | [0.0, 0.0] - [1.0, 1.0] |

### 5. Film Grain

Adds procedural grain noise to simulate film stock, creating a more organic and cinematic appearance.

#### Features
- **Procedural Generation**: Real-time grain using noise functions
- **Luminance-based Intensity**: Grain varies with image brightness
- **Animated Grain**: Time-based animation for realistic film look
- **Configurable Size**: Adjustable grain particle size

#### Usage

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

// Apply film grain
film_grain_pass.execute(&mut builder, &input_target, &output_target)?;
```

#### Configuration

| Parameter | Type | Description | Typical Range |
|-----------|------|-------------|---------------|
| `intensity` | f32 | Grain strength | 0.01 - 0.1 |
| `size` | f32 | Grain particle size | 1.0 - 4.0 |
| `luminance_impact` | f32 | How much grain varies with brightness | 0.0 - 1.0 |
| `time` | f32 | Time for animation (milliseconds) | 0.0 - ∞ |

## Chaining Effects

Multiple cinematic effects can be chained together using `PostProcessChain` to create sophisticated looks:

```rust
use praxis_graphics::PostProcessChain;

let mut chain = PostProcessChain::new();

// Add effects in order
chain.add_pass(Box::new(depth_of_field_pass));
chain.add_pass(Box::new(motion_blur_pass));
chain.add_pass(Box::new(chromatic_aberration_pass));
chain.add_pass(Box::new(vignette_pass));
chain.add_pass(Box::new(film_grain_pass));

// Apply entire chain
let output = chain.process(&input_target, &mut render_target_pool)?;
```

## Performance Considerations

### Optimization Tips

1. **Resolution Scaling**: Apply expensive effects (DoF, motion blur) at lower resolution and upscale
2. **Conditional Application**: Skip effects when intensity is zero or below threshold
3. **Sample Count**: Use fewer samples for motion blur on slower hardware
4. **Effect Ordering**: Apply cheaper effects (vignette, grain) last
5. **Shared Resources**: Reuse render targets through `RenderTargetPool`

### Typical Performance Impact

| Effect | GPU Cost | Frame Time @ 1080p | Notes |
|--------|----------|-------------------|-------|
| Depth-of-Field | High | 2-4ms | Cost scales with blur radius |
| Motion Blur | Medium-High | 1-3ms | Cost scales with sample count |
| Chromatic Aberration | Low | 0.2-0.5ms | Minimal overhead |
| Vignette | Very Low | 0.1-0.2ms | Simple per-pixel operation |
| Film Grain | Low | 0.2-0.4ms | Procedural noise computation |

*Performance measured on mid-range GPU (RTX 3060). Actual performance varies by hardware.*

## Best Practices

### Artistic Guidelines

1. **Subtlety**: Less is more - subtle effects are more realistic
2. **Context**: Match effects to scene mood (e.g., motion blur for action, DoF for cinematics)
3. **Consistency**: Maintain consistent effect intensity throughout experience
4. **Performance**: Test on target hardware and adjust quality accordingly

### Technical Guidelines

1. **Depth Access**: DoF requires accurate depth buffer - ensure proper depth rendering
2. **Velocity Buffer**: Motion blur requires previous frame transformations - store MVP matrices
3. **HDR Pipeline**: Apply effects before tone mapping when possible for better quality
4. **Gamma Correction**: Ensure proper color space handling throughout pipeline

## Common Issues and Solutions

### Depth-of-Field Issues

**Problem**: Objects at wrong distance are in focus
- **Solution**: Verify depth buffer format and precision
- **Solution**: Check focus distance units match world scale

**Problem**: Harsh transitions between focused and blurred areas
- **Solution**: Increase `focus_range` parameter
- **Solution**: Reduce `bokeh_radius` for subtler effect

### Motion Blur Issues

**Problem**: Excessive blur on small movements
- **Solution**: Reduce `intensity` parameter
- **Solution**: Increase `max_blur_radius` clamp

**Problem**: No visible blur
- **Solution**: Verify velocity buffer is being generated correctly
- **Solution**: Check that previous frame matrices are being stored

### Chromatic Aberration Issues

**Problem**: Effect too strong or distracting
- **Solution**: Reduce `intensity` (typically 0.001-0.005 works well)
- **Solution**: Increase `radial_falloff` to concentrate at edges

### Film Grain Issues

**Problem**: Grain appears static or repetitive
- **Solution**: Update `time` parameter each frame
- **Solution**: Adjust `size` to break up patterns

## References

- [GPU Gems 3: Chapter 28 - Practical Post-Process Depth of Field](https://developer.nvidia.com/gpugems/gpugems3/part-iv-image-effects/chapter-28-practical-post-process-depth-field)
- [Siggraph 2014: Next Generation Post Processing in Call of Duty: Advanced Warfare](https://www.iryoku.com/next-generation-post-processing-in-call-of-duty-advanced-warfare/)
- [A Reconstruction Filter for Plausible Motion Blur](https://www.doc.ic.ac.uk/~dfg/graphics/GraphicsLectureNotesSpring2012/MotionBlur.pdf)

## See Also

- [POST_PROCESSING.md](./POST_PROCESSING.md) - General post-processing framework
- [guides/rendering.md](./guides/rendering.md) - Rendering pipeline overview
- [guides/hdr-and-tonemapping.md](./guides/hdr-and-tonemapping.md) - HDR rendering
