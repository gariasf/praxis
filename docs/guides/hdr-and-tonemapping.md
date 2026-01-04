# HDR and Tone Mapping

High Dynamic Range rendering enables more realistic lighting, better bloom effects, and proper handling of bright light sources.

## Overview

The HDR pipeline consists of:
1. **Render to HDR target** - Use floating-point format (`R16G16B16A16_SFLOAT`)
2. **Calculate exposure** - Automatic or manual
3. **Apply tone mapping** - Convert HDR to displayable LDR range

## Key Components

### HdrRenderTarget

Floating-point render target for HDR scene rendering.

```rust
use praxis_graphics::HdrRenderTarget;

let hdr_render_pass = render_context.create_hdr_render_pass()?;
let hdr_target = HdrRenderTarget::new(
    memory_allocator,
    hdr_render_pass,
    [1920, 1080],
)?;
```

**Format Details:**
- **R16G16B16A16_SFLOAT**: 16-bit floating-point per channel
- **Range**: Approximately -65504 to +65504
- **Memory**: 8 bytes per pixel (vs 4 bytes for RGBA8)

### ToneMapper

Converts HDR values to displayable LDR range.

```rust
use praxis_graphics::{ToneMapper, ToneMappingOperator, ExposureMode};

let mut tone_mapper = ToneMapper::new(
    device,
    memory_allocator,
    vulkano::format::Format::R8G8B8A8_UNORM,
    ToneMappingOperator::ACES,
)?;

// Configure automatic exposure
tone_mapper.set_exposure_mode(ExposureMode::Automatic { speed: 2.0 });

// Apply tone mapping
tone_mapper.apply(
    command_buffer,
    &hdr_target,
    output_framebuffer,
    output_extent,
    average_luminance,
    delta_time,
)?;
```

## Tone Mapping Operators

### Reinhard

**Formula:** `color / (color + 1)`

- Simple and fast
- Good for general use
- Can look flat in very bright scenes

**Best for:** Simple scenes, fast iteration, educational purposes

### ACES Filmic (Recommended)

**Formula:** Academy Color Encoding System curve

- Industry standard (AAA games, film)
- Cinematic look with good color preservation
- Slightly more expensive than Reinhard

**Best for:** Production-quality games, realistic rendering

### Uncharted 2 (Hable)

**Formula:** Custom curve from Uncharted 2

- High contrast, dramatic look
- Strong toe and shoulder
- Popular in games

**Best for:** Games with dramatic lighting

## Exposure Modes

### Manual Exposure

Fixed exposure value for artistic control:

```rust
tone_mapper.set_exposure_mode(ExposureMode::Manual { exposure: 1.0 });
```

### Automatic Exposure

Dynamic exposure based on scene luminance:

```rust
tone_mapper.set_exposure_mode(ExposureMode::Automatic { speed: 2.0 });
```

**Algorithm:**
```
target_exposure = key_value / average_luminance
current_exposure = lerp(current_exposure, target_exposure, adaptation_rate * delta_time)
```

**Parameters (via ExposureCalculator):**
- `key_value`: Target middle gray (default: 0.18)
- `min_exposure` / `max_exposure`: Bounds (default: 0.1 to 10.0)

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

        tone_mapper.set_exposure_mode(ExposureMode::Automatic { speed: 2.0 });

        Ok(Self { hdr_target, tone_mapper })
    }

    fn render_frame(&mut self, ...) -> Result<()> {
        // 1. Render scene to HDR target
        // 2. Calculate average luminance
        // 3. Apply tone mapping
        self.tone_mapper.apply(
            &mut builder,
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

## Integration

### With Bloom

HDR significantly improves bloom quality:

```rust
// Render scene to HDR target
// Apply bloom on HDR data (before tone mapping)
bloom_effect.apply(command_buffer, &hdr_target, &bloom_output)?;

// Tone map the bloomed HDR result
tone_mapper.apply(command_buffer, &bloom_output, output_framebuffer, ...)?;
```

### With Deferred Rendering

```rust
// G-buffer can use HDR format for albedo
// Lighting pass outputs to HDR target
deferred_renderer.render_lighting(hdr_target.framebuffer(), ...)?;

// Tone map the lit HDR result
tone_mapper.apply(...)?;
```

## Performance

### Memory Usage

HDR render targets use 2x the memory of LDR:
- **HDR (R16G16B16A16_SFLOAT)**: 8 bytes per pixel
- **LDR (R8G8B8A8_UNORM)**: 4 bytes per pixel

For 1920x1080: HDR ~16.6 MB, LDR ~8.3 MB

### Rendering Cost

Tone mapping is a full-screen post-process:
- **Cost**: O(pixels)
- **Typical**: <1ms at 1080p on modern GPUs
- All operators have similar performance

## Troubleshooting

| Problem | Cause | Solution |
|---------|-------|----------|
| Scene too dark | Low exposure | Increase exposure or key value |
| Scene too bright | High exposure | Decrease exposure or key value |
| Washed out colors | High exposure or wrong operator | Try ACES, check gamma (2.2) |
| Flickering (auto-exposure) | Fast adaptation | Decrease adaptation speed |

## Example

```bash
cargo run --example hdr_demo
```

## See Also

- [Rendering Guide](rendering.md) - Complete rendering pipeline
- [Post-Processing](post-processing.md) - Bloom and effects
- [Concepts: Vulkan Rendering](../concepts/vulkan-rendering.md) - Pipeline fundamentals
