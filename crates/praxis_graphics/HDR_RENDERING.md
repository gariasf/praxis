# HDR Rendering System

This document describes the HDR (High Dynamic Range) rendering system in Praxis Graphics.

## Overview

The HDR rendering system provides a complete pipeline for rendering with floating-point precision and tone mapping to displayable LDR (Low Dynamic Range) values. This enables more realistic lighting, better bloom effects, and proper handling of bright light sources.

## Key Components

### HdrRenderTarget

Floating-point render target using `R16G16B16A16_SFLOAT` format.

```rust
use praxis_graphics::HdrRenderTarget;

// Create HDR render pass
let hdr_render_pass = render_context.create_hdr_render_pass()?;

// Create HDR render target
let hdr_target = HdrRenderTarget::new(
    memory_allocator,
    hdr_render_pass,
    [1920, 1080], // Resolution
)?;
```

**Format Details:**
- **R16G16B16A16_SFLOAT**: 16-bit floating-point per channel
- **Range**: Approximately -65504 to +65504
- **Precision**: Sufficient for most HDR scenarios
- **Memory**: 8 bytes per pixel (vs 4 bytes for RGBA8)

### ToneMapper

Converts HDR values to displayable LDR range using various algorithms.

```rust
use praxis_graphics::{ToneMapper, ToneMappingOperator};

// Create tone mapper with ACES operator
let mut tone_mapper = ToneMapper::new(
    device,
    memory_allocator,
    vulkano::format::Format::R8G8B8A8_UNORM,
    ToneMappingOperator::ACES,
)?;

// Apply tone mapping
tone_mapper.apply(
    command_buffer,
    &hdr_target,          // HDR input
    output_framebuffer,   // LDR output
    output_extent,
    average_luminance,    // For auto-exposure
    delta_time,          // For smooth adaptation
)?;
```

### Tone Mapping Operators

Three tone mapping operators are provided:

#### 1. Reinhard

**Formula:** `color / (color + 1)`

**Characteristics:**
- Simple and fast
- Good for general use
- Smooth falloff
- Can look flat in very bright scenes

**Best For:** Simple scenes, fast iteration, educational purposes

#### 2. ACES Filmic

**Formula:** Academy Color Encoding System curve

**Characteristics:**
- Industry standard
- Used in AAA games and film production
- Cinematic look
- Good color preservation
- Slightly more expensive than Reinhard

**Best For:** Production-quality games, cinematic look, realistic rendering

#### 3. Uncharted 2 (Hable)

**Formula:** Custom curve designed for Uncharted 2

**Characteristics:**
- High contrast
- Dramatic look
- Strong toe and shoulder
- Popular in games
- Similar cost to ACES

**Best For:** Games with dramatic lighting, high-contrast scenes

### Exposure Calculation

Two exposure modes are supported:

#### Manual Exposure

Fixed exposure value set by the application.

```rust
use praxis_graphics::ExposureMode;

tone_mapper.set_exposure_mode(ExposureMode::Manual {
    exposure: 1.0, // Fixed value
});
```

**Use Cases:**
- Artistic control
- Testing specific exposure values
- Cutscenes with controlled lighting

#### Automatic Exposure

Dynamic exposure based on scene luminance.

```rust
use praxis_graphics::ExposureMode;

tone_mapper.set_exposure_mode(ExposureMode::Automatic {
    speed: 2.0, // Adaptation speed (higher = faster)
});
```

**Algorithm:**
```
target_exposure = key_value / average_luminance
current_exposure = lerp(current_exposure, target_exposure, adaptation_rate * delta_time)
```

**Parameters:**
- **key_value**: Target middle gray (default: 0.18)
- **speed**: How fast exposure adapts to changes
- **min_exposure**: Lower bound (default: 0.1)
- **max_exposure**: Upper bound (default: 10.0)

**Use Cases:**
- Dynamic lighting environments
- Indoor/outdoor transitions
- Realistic camera simulation

### ExposureCalculator

Low-level exposure calculation for custom implementations.

```rust
use praxis_graphics::{ExposureCalculator, ExposureMode};

let mut calculator = ExposureCalculator::new(
    ExposureMode::Automatic { speed: 2.0 }
);

// In render loop
let exposure = calculator.calculate(average_luminance, delta_time);

// Advanced configuration
calculator.set_key_value(0.18);      // Middle gray target
calculator.set_min_exposure(0.1);    // Minimum exposure
calculator.set_max_exposure(10.0);   // Maximum exposure
```

## HDR Rendering Pipeline

### Stage 1: HDR Scene Rendering

Render your scene to the HDR render target using standard rendering commands.

```rust
// Render scene to HDR target
// Note: You would render to hdr_target.framebuffer() instead of swapchain
let render_commands = RenderCommands {
    view: camera_view,
    proj: camera_proj,
    draw_commands: &draw_commands,
    lighting: Some(&lighting),
};

// This renders to swapchain, but you would render to HDR target
render_context.render(&render_commands)?;
```

### Stage 2: Luminance Calculation (Optional for Auto-Exposure)

Calculate average scene luminance. This can be done via:

1. **CPU-side calculation**: Sample HDR texture and compute average
2. **GPU compute shader**: Use histogram or reduction
3. **Approximation**: Use a fixed value based on scene knowledge

```rust
// Simple approximation for demo
let average_luminance = 0.5;

// Or calculate from scene parameters
let average_luminance = calculate_scene_luminance(&lighting_data);
```

### Stage 3: Tone Mapping

Apply tone mapping to convert HDR to LDR.

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
    1.5, // Manual exposure value
)?;
```

## Complete Example

```rust
use praxis_graphics::{
    HdrRenderTarget, ToneMapper, ToneMappingOperator, ExposureMode,
    RenderContext, RenderCommands,
};
use praxis_utils::Result;
use std::sync::Arc;

struct HdrRenderer {
    hdr_target: HdrRenderTarget,
    tone_mapper: ToneMapper,
}

impl HdrRenderer {
    fn new(render_context: &RenderContext) -> Result<Self> {
        // Create HDR render pass
        let hdr_render_pass = render_context.create_hdr_render_pass()?;
        
        // Create HDR render target
        let hdr_target = HdrRenderTarget::new(
            render_context.memory_allocator.clone(),
            hdr_render_pass,
            [1920, 1080],
        )?;
        
        // Create tone mapper with ACES
        let mut tone_mapper = ToneMapper::new(
            render_context.device.clone(),
            render_context.memory_allocator.clone(),
            vulkano::format::Format::R8G8B8A8_UNORM,
            ToneMappingOperator::ACES,
        )?;
        
        // Configure automatic exposure
        tone_mapper.set_exposure_mode(ExposureMode::Automatic {
            speed: 2.0,
        });
        
        Ok(Self {
            hdr_target,
            tone_mapper,
        })
    }
    
    fn render_frame(
        &mut self,
        render_context: &mut RenderContext,
        render_commands: &RenderCommands,
        output_framebuffer: &Arc<vulkano::render_pass::Framebuffer>,
        delta_time: f32,
    ) -> Result<()> {
        // Note: This example shows the general structure.
        // In a real implementation, you would render to hdr_target.framebuffer()
        // instead of directly to the swapchain.
        
        // Calculate or approximate average luminance
        let average_luminance = 0.5;
        
        // Create command buffer
        let mut builder = AutoCommandBufferBuilder::primary(
            render_context.command_buffer_allocator.clone(),
            render_context.graphics_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )?;
        
        // Apply tone mapping
        self.tone_mapper.apply(
            &mut builder,
            &self.hdr_target,
            output_framebuffer,
            [1920, 1080],
            average_luminance,
            delta_time,
        )?;
        
        // Execute command buffer...
        
        Ok(())
    }
    
    fn switch_operator(&mut self, operator: ToneMappingOperator) {
        self.tone_mapper.set_operator(operator);
    }
    
    fn set_manual_exposure(&mut self, exposure: f32) {
        self.tone_mapper.set_exposure_mode(ExposureMode::Manual { exposure });
    }
}
```

## Performance Considerations

### Memory Usage

HDR render targets use 2x the memory of standard LDR targets:
- **HDR (R16G16B16A16_SFLOAT)**: 8 bytes per pixel
- **LDR (R8G8B8A8_UNORM)**: 4 bytes per pixel

For 1920×1080:
- HDR: ~16.6 MB
- LDR: ~8.3 MB

### Rendering Cost

Tone mapping is a full-screen post-process pass:
- **Cost**: O(pixels)
- **Typical performance**: <1ms at 1080p on modern GPUs
- **Operator complexity**: All operators have similar performance

### Recommendations

1. **Resolution**: Use native or slightly lower resolution for HDR target
2. **Format**: R16G16B16A16_SFLOAT is the sweet spot for quality/performance
3. **Operator**: ACES for production, Reinhard for rapid iteration
4. **Auto-exposure**: Use fixed value if scene has consistent lighting

## Integration with Existing Systems

### Bloom Effect

HDR rendering significantly improves bloom quality:

```rust
// Render scene to HDR target
// Apply bloom on HDR data
bloom_effect.apply(command_buffer, &hdr_target, &bloom_output, &mut pool)?;

// Tone map the bloomed HDR result
tone_mapper.apply(command_buffer, &bloom_output, output_framebuffer, ...)?;
```

### Deferred Rendering

HDR works naturally with deferred rendering:

```rust
// G-buffer can use HDR format for albedo
// Lighting pass outputs to HDR target
deferred_renderer.render_lighting(hdr_target.framebuffer(), ...)?;

// Tone map the lit HDR result
tone_mapper.apply(...)?;
```

## Best Practices

1. **Always use HDR with bloom**: The combination is far superior to LDR bloom
2. **Test multiple operators**: Different scenes look better with different operators
3. **Start with auto-exposure**: Manual exposure is hard to tune
4. **Monitor exposure values**: Display current exposure in debug UI
5. **Clamp light values**: Even with HDR, extremely high values can cause issues

## Troubleshooting

### Scene too dark
- Increase manual exposure
- Increase auto-exposure key value
- Check light intensities are reasonable
- Verify HDR target is being used

### Scene too bright
- Decrease manual exposure
- Decrease auto-exposure key value
- Check for extremely bright light sources
- Verify tone mapping is being applied

### Washed out colors
- Try ACES operator (best color preservation)
- Check gamma value (should be 2.2)
- Ensure exposure is not too high

### Flickering in auto-exposure
- Decrease adaptation speed
- Smooth luminance calculation
- Clamp exposure min/max more tightly

## References

- ACES: https://github.com/TheRealMJP/BakingLab/blob/master/BakingLab/ACES.hlsl
- Uncharted 2: http://filmicworlds.com/blog/filmic-tonemapping-operators/
- HDR Theory: https://learnopengl.com/Advanced-Lighting/HDR
