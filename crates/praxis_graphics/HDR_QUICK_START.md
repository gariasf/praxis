# HDR Rendering Quick Start Guide

Get up and running with HDR rendering in minutes.

## Basic Setup (5 minutes)

### Step 1: Create HDR Render Target

```rust
use praxis_graphics::HdrRenderTarget;

// Create HDR render pass
let hdr_render_pass = render_context.create_hdr_render_pass()?;

// Create HDR render target
let hdr_target = HdrRenderTarget::new(
    memory_allocator.clone(),
    hdr_render_pass,
    [1920, 1080], // Your resolution
)?;
```

### Step 2: Create Tone Mapper

```rust
use praxis_graphics::{ToneMapper, ToneMappingOperator};

let mut tone_mapper = ToneMapper::new(
    device.clone(),
    memory_allocator.clone(),
    vulkano::format::Format::R8G8B8A8_UNORM,
    ToneMappingOperator::ACES, // Best default choice
)?;
```

### Step 3: Render and Tone Map

```rust
// 1. Render your scene to hdr_target.framebuffer()
// (Implementation depends on your rendering setup)

// 2. Apply tone mapping
let average_luminance = 0.5; // Or calculate from scene
let delta_time = 0.016; // Your frame delta time

tone_mapper.apply(
    &mut command_buffer,
    &hdr_target,
    output_framebuffer,
    [1920, 1080],
    average_luminance,
    delta_time,
)?;
```

Done! You now have HDR rendering with automatic exposure.

## Common Use Cases

### Gaming with Dynamic Lighting

```rust
// Use automatic exposure for realistic adaptation
tone_mapper.set_exposure_mode(ExposureMode::Automatic {
    speed: 2.0, // Adjust to taste (1.0-5.0 typical)
});

// Use ACES for film-like look
tone_mapper.set_operator(ToneMappingOperator::ACES);
```

### Artistic/Fixed Lighting

```rust
// Use manual exposure for full control
tone_mapper.set_exposure_mode(ExposureMode::Manual {
    exposure: 1.5, // Adjust for desired brightness
});

// Try different operators for different looks
tone_mapper.set_operator(ToneMappingOperator::Uncharted2);
```

### High Contrast Scenes

```rust
// Uncharted 2 operator works great for dramatic lighting
tone_mapper.set_operator(ToneMappingOperator::Uncharted2);

// Use slower auto-exposure to avoid flickering
tone_mapper.set_exposure_mode(ExposureMode::Automatic {
    speed: 1.0,
});
```

## Operator Cheat Sheet

| Operator | Best For | Look | Speed |
|----------|----------|------|-------|
| **ACES** | Most games, production | Cinematic, balanced | Medium |
| **Reinhard** | Testing, simple scenes | Flat, neutral | Fast |
| **Uncharted2** | Dramatic scenes | High contrast | Medium |

**Quick Rule**: Start with ACES, switch to Uncharted 2 if you want more contrast.

## Exposure Cheat Sheet

### Automatic Exposure

```rust
ExposureMode::Automatic {
    speed: 2.0, // How fast to adapt
}
```

**Speed Values:**
- `0.5-1.0`: Slow, realistic camera-like adaptation
- `2.0-3.0`: Medium, good for most games
- `5.0+`: Fast, instant adaptation

### Manual Exposure

```rust
ExposureMode::Manual {
    exposure: 1.0, // Brightness multiplier
}
```

**Exposure Values:**
- `0.5`: Darker scene
- `1.0`: Neutral
- `2.0`: Brighter scene

## Debug/Testing

### Add Debug UI

```rust
// In your GUI update
ui.label(format!("Exposure: {:.2}", tone_mapper.current_exposure()));
ui.add(egui::Slider::new(&mut gamma, 1.0..=3.0).text("Gamma"));
tone_mapper.set_gamma(gamma);
```

### Test Different Operators

```rust
if ui.button("ACES").clicked() {
    tone_mapper.set_operator(ToneMappingOperator::ACES);
}
if ui.button("Reinhard").clicked() {
    tone_mapper.set_operator(ToneMappingOperator::Reinhard);
}
if ui.button("Uncharted 2").clicked() {
    tone_mapper.set_operator(ToneMappingOperator::Uncharted2);
}
```

## Troubleshooting

### Scene Too Dark
```rust
// Increase exposure
tone_mapper.set_exposure_mode(ExposureMode::Manual { exposure: 2.0 });
```

### Scene Too Bright
```rust
// Decrease exposure
tone_mapper.set_exposure_mode(ExposureMode::Manual { exposure: 0.5 });
```

### Colors Look Wrong
```rust
// Use ACES (best color preservation)
tone_mapper.set_operator(ToneMappingOperator::ACES);

// Check gamma
tone_mapper.set_gamma(2.2);
```

### Flickering with Auto-Exposure
```rust
// Slow down adaptation
tone_mapper.set_exposure_mode(ExposureMode::Automatic { speed: 0.5 });
```

## Performance Tips

1. **HDR target resolution**: Can use lower resolution than final output
2. **Luminance calculation**: Use fixed value (0.5) for constant scenes
3. **Operator cost**: All operators have similar performance (~<1ms at 1080p)

## Next Steps

- Read `HDR_RENDERING.md` for detailed explanations
- Run `cargo run --example hdr_demo` to see it in action
- Integrate with bloom for best visual quality
- Experiment with different operators and exposure settings

## Common Patterns

### Indoor/Outdoor Transition

```rust
// Auto-exposure handles this naturally
tone_mapper.set_exposure_mode(ExposureMode::Automatic {
    speed: 1.5, // Smooth adaptation
});
```

### Cutscene with Specific Look

```rust
// Manual exposure for consistent brightness
tone_mapper.set_exposure_mode(ExposureMode::Manual { exposure: 1.2 });

// Cinematic operator
tone_mapper.set_operator(ToneMappingOperator::ACES);
```

### High-Speed Action

```rust
// Fast adaptation to keep up with rapid changes
tone_mapper.set_exposure_mode(ExposureMode::Automatic {
    speed: 3.0,
});

// High contrast for clarity
tone_mapper.set_operator(ToneMappingOperator::Uncharted2);
```

## Complete Minimal Example

```rust
use praxis_graphics::{
    HdrRenderTarget, ToneMapper, ToneMappingOperator, ExposureMode,
};
use praxis_utils::Result;

fn setup_hdr(render_context: &RenderContext) -> Result<(HdrRenderTarget, ToneMapper)> {
    // Create HDR target
    let hdr_render_pass = render_context.create_hdr_render_pass()?;
    let hdr_target = HdrRenderTarget::new(
        render_context.memory_allocator.clone(),
        hdr_render_pass,
        [1920, 1080],
    )?;
    
    // Create tone mapper
    let mut tone_mapper = ToneMapper::new(
        render_context.device.clone(),
        render_context.memory_allocator.clone(),
        vulkano::format::Format::R8G8B8A8_UNORM,
        ToneMappingOperator::ACES,
    )?;
    
    // Configure auto-exposure
    tone_mapper.set_exposure_mode(ExposureMode::Automatic { speed: 2.0 });
    
    Ok((hdr_target, tone_mapper))
}

fn render_frame(
    hdr_target: &HdrRenderTarget,
    tone_mapper: &mut ToneMapper,
    command_buffer: &mut AutoCommandBufferBuilder,
    output_framebuffer: &Arc<Framebuffer>,
    delta_time: f32,
) -> Result<()> {
    // 1. Render scene to HDR target
    // (Your scene rendering code here)
    
    // 2. Apply tone mapping
    tone_mapper.apply(
        command_buffer,
        hdr_target,
        output_framebuffer,
        [1920, 1080],
        0.5, // Average luminance
        delta_time,
    )?;
    
    Ok(())
}
```

That's it! You're now rendering in HDR.
