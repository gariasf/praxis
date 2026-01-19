# Velocity Buffer and TAA Integration

This document provides a comprehensive guide to the velocity buffer and Temporal Anti-Aliasing (TAA) implementation in Praxis.

## Overview

The Praxis graphics system includes a fully integrated velocity buffer and TAA pipeline that works seamlessly with the deferred renderer. This system provides:

- **Per-pixel velocity buffer generation** during the geometry pass
- **Temporal reprojection** with neighborhood clamping
- **Halton sequence jitter** for optimal sub-pixel sampling
- **Adaptive blending** based on motion magnitude
- **Motion blur support** using velocity data

## Architecture

### Velocity Buffer Generation

The velocity buffer is automatically populated during the deferred rendering geometry pass:

```glsl
// Vertex Shader (deferred_geometry.vert)
v_current_pos = view_proj.proj * view_proj.view * model * vec4(position, 1.0);
v_previous_pos = prev_view_proj.proj * prev_view_proj.view * prev_model * vec4(position, 1.0);

// Fragment Shader (deferred_geometry.frag)
vec2 current_ndc = v_current_pos.xy / v_current_pos.w;
vec2 previous_ndc = v_previous_pos.xy / v_previous_pos.w;
out_velocity = current_ndc - previous_ndc;
```

### TAA Pipeline

The TAA pass uses the velocity buffer for temporal reprojection:

1. **Temporal Reprojection**: Sample history using velocity-based UV offset
2. **Neighborhood Clamping**: Reject invalid history using AABB clamping in YCoCg color space
3. **Adaptive Blending**: Blend current and history based on motion magnitude
4. **Jitter Application**: Apply Halton sequence jitter for sub-pixel accumulation

## Usage

### Basic Setup

```rust
use praxis_graphics::{
    deferred::{DeferredRenderParams, DeferredRenderer},
    taa::{TaaRenderer, TaaApplyParams, TaaConfig, HaltonSequence, apply_jitter_to_projection},
    velocity_buffer::VelocityBufferRenderer,
};

// Create renderers
let deferred_renderer = DeferredRenderer::new(
    device.clone(),
    memory_allocator.clone(),
    descriptor_set_allocator.clone(),
    width,
    height,
)?;

let taa_renderer = TaaRenderer::new(device.clone(), memory_allocator.clone())?;
let taa_target = taa_renderer.create_render_target(width, height)?;

// Create Halton sequence for jitter
let mut halton_sequence = HaltonSequence::new();
```

### Per-Frame Rendering

```rust
// 1. Generate jitter for current frame
let jitter = halton_sequence.next_jitter();
let jittered_proj = apply_jitter_to_projection(proj, jitter, width, height);

// 2. Track previous frame state
let previous_view_proj = // ... stored from last frame
let previous_transforms = // ... stored from last frame

// 3. Render with deferred renderer (generates velocity buffer)
let params = DeferredRenderParams {
    output_framebuffer,
    viewport,
    draw_commands: &draw_commands,
    view_proj_buffer,
    dynamic_uniform_buffer: &current_dynamic_buffer,
    mesh_manager,
    texture_manager,
    lighting_buffer,
    previous_view_proj_buffer,      // Previous frame matrices
    previous_dynamic_uniform_buffer, // Previous frame transforms
};

deferred_renderer.render(&mut cmd_buffer, &params)?;

// 4. Access velocity buffer
let velocity_buffer = deferred_renderer.velocity_buffer().unwrap();

// 5. Apply TAA
let taa_config = TaaConfig {
    jitter_offset: jitter,
    blend_factor: 0.1,
};

let taa_params = TaaApplyParams {
    taa_target: &taa_target,
    current_frame: current_frame_view,
    velocity_buffer: velocity_buffer.clone(),
    depth_buffer: depth_buffer_view,
    config: taa_config,
};

taa_renderer.apply(&mut cmd_buffer, &taa_params)?;

// 6. Swap TAA history buffers
taa_target.swap_buffers();

// 7. Store state for next frame
previous_view_proj = current_view_proj;
previous_transforms = current_transforms;
```

## Halton Sequence

The Halton sequence provides optimal sub-pixel jitter patterns for TAA:

```rust
let mut sequence = HaltonSequence::new();

// Generates values in range [-0.5, 0.5] over 16 frames
let jitter = sequence.next_jitter();  // [f32; 2]

// Apply to projection matrix
let jittered_proj = apply_jitter_to_projection(proj, jitter, width, height);
```

The Halton(2,3) sequence ensures even distribution of sample points across the pixel:

```text
Frame 0: (+0.000, +0.000)
Frame 1: (+0.500, +0.333)
Frame 2: (+0.250, +0.667)
Frame 3: (+0.750, +0.111)
...
Frame 15: Back to frame 0 pattern
```

## TAA Configuration

### Blend Factor

Controls the ratio between current and history frames:

```rust
TaaConfig {
    blend_factor: 0.05,  // 95% history, 5% current (more temporal stability)
    // or
    blend_factor: 0.2,   // 80% history, 20% current (more responsive)
}
```

- **Lower values (0.05-0.1)**: More temporal accumulation, better quality, potential ghosting
- **Higher values (0.2-0.3)**: Less ghosting, more responsive, less accumulation

### Adaptive Blending

The TAA shader automatically adjusts blend factor based on velocity:

```glsl
float velocity_length = length(velocity);
float adaptive_blend = mix(config.blend_factor, 0.5, clamp(velocity_length * 10.0, 0.0, 1.0));
```

- Static pixels: Use configured blend factor (more history)
- Fast-moving pixels: Blend towards 0.5 (less history to reduce ghosting)

## Velocity Buffer Format

The velocity buffer uses `R16G16_SFLOAT` format:

- **R channel**: Horizontal screen-space velocity
- **G channel**: Vertical screen-space velocity
- **Range**: Typically [-1, 1] in NDC space, but can exceed for fast motion
- **Precision**: 16-bit float provides sub-pixel accuracy

### Reading Velocity Data

```rust
// Extract velocity from deferred renderer
let velocity_buffer = deferred_renderer.velocity_buffer().unwrap();

// Use for motion blur
let motion_blur_params = MotionBlurParams {
    input: current_frame,
    velocity_buffer: velocity_buffer.clone(),
    config: MotionBlurConfig::default(),
};
```

## Motion Blur Integration

The velocity buffer can be used for motion blur effects:

```rust
use praxis_graphics::post_process::{MotionBlurPass, MotionBlurConfig};

let motion_blur = MotionBlurPass::new(
    device.clone(),
    memory_allocator.clone(),
    format,
    MotionBlurConfig {
        intensity: 1.0,
        sample_count: 16,
        shutter_angle: 180.0,
        max_blur_radius: 32.0,
    },
)?;

// Apply motion blur using velocity buffer
motion_blur.execute_with_velocity(
    &mut cmd_buffer,
    input,
    velocity_buffer,
    velocity_sampler,
    output,
)?;
```

## Common Issues and Solutions

### Ghosting Artifacts

**Symptom**: Visible trails behind moving objects

**Solutions**:
- Increase `blend_factor` (e.g., from 0.05 to 0.15)
- Check that neighborhood clamping is working (should be automatic)
- Verify velocity buffer accuracy
- Consider tighter AABB clamping bounds

### Excessive Blur

**Symptom**: Image appears blurry or soft

**Solutions**:
- Decrease `blend_factor` (e.g., from 0.2 to 0.08)
- Verify jitter magnitude is in correct range
- Check that jitter is being applied correctly
- Ensure velocity buffer has correct values

### Temporal Instability

**Symptom**: Flickering or unstable image

**Solutions**:
- Verify Halton sequence is incrementing correctly
- Check that history buffer is preserved between frames
- Ensure `swap_buffers()` is called after TAA
- Verify previous frame matrices are stored correctly

### Incorrect Motion Vectors

**Symptom**: Weird artifacts, incorrect reprojection

**Solutions**:
- Verify current and previous matrices are correct
- Check perspective division in velocity calculation
- Ensure dynamic offset indexing is correct
- Validate that model matrices match draw order

## Performance Considerations

### Cost Breakdown

At 1080p (1920×1080):

- **Velocity Buffer Generation**: ~0.1ms (part of geometry pass)
- **TAA Pass**: ~2-3ms (full-screen pass with neighborhood sampling)
- **Total Overhead**: ~2-3ms per frame

### Optimization Tips

1. **Use appropriate jitter patterns**: Halton(2,3) is optimal for 16-frame cycles
2. **Minimize history samples**: 3×3 neighborhood is sufficient for clamping
3. **Consider resolution**: TAA cost scales with pixel count
4. **Profile neighborhood sampling**: This is the main cost in TAA shader

## Visual Verification

When implementing or debugging TAA, verify:

### Velocity Buffer
- [ ] Moving objects generate non-zero velocity
- [ ] Static objects have near-zero velocity
- [ ] Camera motion produces consistent patterns
- [ ] Velocity magnitude matches object speed

### TAA Quality
- [ ] Edges are smooth without excessive blur
- [ ] No significant ghosting on moving objects
- [ ] Temporal stability across frames
- [ ] Sub-pixel details preserved

### Motion Blur
- [ ] Blur direction matches motion
- [ ] Blur magnitude scales with velocity
- [ ] Static objects remain sharp
- [ ] No artifacts on fast motion

## Examples

See `examples/deferred_taa_demo.rs` for a complete working example demonstrating:

- Deferred rendering with velocity buffer generation
- TAA with Halton jitter
- Motion blur effects
- Interactive camera controls
- Toggle switches for effect comparison

## Technical Details

### Shader Pipeline

1. **Geometry Pass** (`deferred_geometry.vert/frag`)
   - Outputs: Albedo, Normal, Metallic-Roughness, **Velocity**, Depth
   - Velocity computed from current and previous clip-space positions

2. **TAA Pass** (`taa.vert/frag`)
   - Inputs: Current frame, History frame, Velocity buffer, Depth buffer
   - Performs temporal reprojection and neighborhood clamping
   - Outputs: Anti-aliased frame

3. **Motion Blur Pass** (Optional) (`post_process_motion_blur.frag`)
   - Inputs: Current frame, Velocity buffer
   - Samples along motion vectors for blur
   - Outputs: Motion-blurred frame

### Color Space

TAA uses YCoCg color space for better clamping:

```glsl
vec3 rgb_to_ycocg(vec3 rgb) {
    float Y  = dot(rgb, vec3(0.25, 0.5, 0.25));
    float Co = dot(rgb, vec3(0.5, 0.0, -0.5));
    float Cg = dot(rgb, vec3(-0.25, 0.5, -0.25));
    return vec3(Y, Co, Cg);
}
```

This provides better perceptual clamping than RGB space.

## References

- [High Quality Temporal Supersampling](https://de45xmedrsdbp.cloudfront.net/Resources/files/TemporalAA_small-59732822.pdf) - Brian Karis, Epic Games
- [Temporal Anti-Aliasing in Uncharted 4](https://www.gdcvault.com/play/1023521/Temporal-Supersampling-and-Anti-Aliasing) - GDC 2016
- [A Survey of Temporal Antialiasing Techniques](https://www.elopezr.com/temporal-aa-and-the-quest-for-the-holy-trail/) - Eduardo López

## Testing

The integration test suite (`tests/velocity_buffer_taa_test.rs`) validates:

- Velocity buffer generation with moving objects
- Motion vector magnitude accuracy across frames
- Temporal reprojection UV calculations
- History buffer sampling and blending
- Out-of-bounds detection

Run tests with:

```bash
cargo test --test velocity_buffer_taa_test
```
