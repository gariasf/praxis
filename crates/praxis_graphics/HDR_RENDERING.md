# HDR Rendering

High Dynamic Range rendering pipeline with tone mapping for realistic lighting and bloom effects.

## Overview

HDR rendering uses floating-point precision for scene rendering and tone mapping to convert to displayable LDR values. This enables:

- More realistic lighting with high-intensity light sources
- Better bloom effects without clamping
- Proper handling of bright-to-dark transitions
- Automatic or manual exposure control

## Quick Start

```rust
use praxis_graphics::{HdrRenderTarget, ToneMapper, ToneMappingOperator, ExposureMode};

// Create HDR render target
let hdr_render_pass = render_context.create_hdr_render_pass()?;
let hdr_target = HdrRenderTarget::new(
    memory_allocator,
    hdr_render_pass,
    [1920, 1080],
)?;

// Create tone mapper with ACES operator
let mut tone_mapper = ToneMapper::new(
    device,
    memory_allocator,
    vulkano::format::Format::R8G8B8A8_UNORM,
    ToneMappingOperator::ACES,
)?;

// Configure automatic exposure
tone_mapper.set_exposure_mode(ExposureMode::Automatic { speed: 2.0 });

// Render to HDR target, then apply tone mapping
tone_mapper.apply(
    command_buffer,
    &hdr_target,
    output_framebuffer,
    output_extent,
    average_luminance,
    delta_time,
)?;
```

## Components

### HdrRenderTarget

Floating-point render target using `R16G16B16A16_SFLOAT` format.

**Format details:**
- 16-bit float per channel
- Range: -65504 to +65504
- 8 bytes per pixel (vs 4 for RGBA8)

### ToneMapper

Converts HDR to displayable LDR using tone mapping operators.

**Operators:**

**1. Reinhard**
- Formula: `color / (color + 1)`
- Simple and fast
- Good for general use
- Best for: Simple scenes, fast iteration

**2. ACES Filmic**
- Industry standard (AAA games, film)
- Cinematic look
- Good color preservation
- Best for: Production quality, realistic rendering

**3. Uncharted 2 (Hable)**
- High contrast, dramatic look
- Strong toe and shoulder
- Popular in games
- Best for: Dramatic lighting, high-contrast scenes

### Exposure Modes

**Manual Exposure:**
```rust
tone_mapper.set_exposure_mode(ExposureMode::Manual {
    exposure: 1.0,  // Fixed value
});
```

**Automatic Exposure:**
```rust
tone_mapper.set_exposure_mode(ExposureMode::Automatic {
    speed: 2.0,  // Adaptation speed
});
```

**Algorithm:**
```
target_exposure = key_value / average_luminance
current_exposure = lerp(current_exposure, target_exposure, speed * delta_time)
```

**Parameters:**
- `key_value`: Target middle gray (default: 0.18)
- `speed`: Adaptation speed (higher = faster)
- `min_exposure`: Lower bound (default: 0.1)
- `max_exposure`: Upper bound (default: 10.0)

## HDR Pipeline

### Stage 1: HDR Scene Rendering

Render scene to HDR render target:

```rust
// Render to hdr_target.framebuffer() instead of swapchain
let render_commands = RenderCommands {
    view: camera_view,
    proj: camera_proj,
    draw_commands: &draw_commands,
    lighting: Some(&lighting),
};

// Note: Actual rendering would target HDR framebuffer
```

### Stage 2: Luminance Calculation

Calculate average scene luminance for auto-exposure:

```rust
// Simple approximation
let average_luminance = 0.5;

// Or calculate from scene
let average_luminance = calculate_scene_luminance(&lighting_data);
```

### Stage 3: Tone Mapping

Convert HDR to LDR:

```rust
// With automatic exposure
tone_mapper.apply(
    command_buffer,
    &hdr_target,
    output_framebuffer,
    output_extent,
    average_luminance,
    delta_time,
)?;

// With manual exposure
tone_mapper.apply_with_exposure(
    command_buffer,
    &hdr_target,
    output_framebuffer,
    output_extent,
    1.5,  // Manual exposure
)?;
```

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
        let hdr_render_pass = render_context.create_hdr_render_pass()?;
        let hdr_target = HdrRenderTarget::new(
            render_context.memory_allocator.clone(),
            hdr_render_pass,
            [1920, 1080],
        )?;
        
        let mut tone_mapper = ToneMapper::new(
            render_context.device.clone(),
            render_context.memory_allocator.clone(),
            vulkano::format::Format::R8G8B8A8_UNORM,
            ToneMappingOperator::ACES,
        )?;
        
        tone_mapper.set_exposure_mode(ExposureMode::Automatic {
            speed: 2.0,
        });
        
        Ok(Self { hdr_target, tone_mapper })
    }
    
    fn render(&mut self, /* ... */) -> Result<()> {
        let average_luminance = 0.5;
        
        // Render scene to HDR target...
        
        // Apply tone mapping
        self.tone_mapper.apply(
            command_buffer,
            &self.hdr_target,
            output_framebuffer,
            output_extent,
            average_luminance,
            delta_time,
        )?;
        
        Ok(())
    }
}
```

## Performance

### Memory Usage

**HDR (R16G16B16A16_SFLOAT)**: 8 bytes per pixel  
**LDR (R8G8B8A8_UNORM)**: 4 bytes per pixel  

**1920×1080:**
- HDR: ~16.6 MB
- LDR: ~8.3 MB

### Rendering Cost

- **Tone mapping**: <1ms at 1080p on modern GPUs
- **Full-screen pass**: O(pixels)
- **Operator complexity**: All operators have similar cost

### Recommendations

1. Use native or slightly lower resolution for HDR target
2. R16G16B16A16_SFLOAT is optimal for quality/performance
3. ACES for production, Reinhard for rapid iteration
4. Fixed value if scene has consistent lighting

## Integration

### With Bloom

HDR significantly improves bloom quality:

```rust
// Render scene to HDR target
// Apply bloom on HDR data
bloom_effect.apply(command_buffer, &hdr_target, &bloom_output)?;

// Tone map the bloomed result
tone_mapper.apply(command_buffer, &bloom_output, output)?;
```

### With Deferred Rendering

```rust
// Lighting pass outputs to HDR target
deferred_renderer.render_lighting(hdr_target.framebuffer())?;

// Tone map result
tone_mapper.apply()?;
```

## Best Practices

1. **Always use HDR with bloom** - combination is far superior to LDR
2. **Test multiple operators** - different scenes look better with different operators
3. **Start with auto-exposure** - manual exposure is hard to tune
4. **Monitor exposure values** - display in debug UI
5. **Clamp light values** - even with HDR, extreme values can cause issues

## Troubleshooting

### Scene too dark
- Increase manual exposure
- Increase auto-exposure key value
- Check light intensities
- Verify HDR target is used

### Scene too bright
- Decrease manual exposure
- Decrease auto-exposure key value
- Check for extremely bright lights
- Verify tone mapping is applied

### Washed out colors
- Try ACES operator (best color preservation)
- Check gamma value (should be 2.2)
- Ensure exposure not too high

### Flickering in auto-exposure
- Decrease adaptation speed
- Smooth luminance calculation
- Clamp exposure min/max more tightly

## See Also

- [Post-Processing](POST_PROCESSING.md)
- [Rendering Guide](../../docs/guides/rendering/hdr-tonemapping.md)
- Implementation: `crates/praxis_graphics/src/hdr.rs`
- ACES reference: https://github.com/TheRealMJP/BakingLab/blob/master/BakingLab/ACES.hlsl
