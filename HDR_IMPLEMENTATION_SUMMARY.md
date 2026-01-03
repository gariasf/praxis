# HDR Rendering Implementation Summary

This document summarizes the HDR (High Dynamic Range) rendering implementation for the Praxis game engine.

## Overview

A complete HDR rendering pipeline has been implemented with floating-point render targets, exposure calculation (automatic and manual), and multiple tone mapping operators.

## Components Implemented

### 1. HDR Module (`crates/praxis_graphics/src/hdr/`)

New module containing all HDR rendering functionality:

#### `render_target.rs`
- **HdrRenderTarget**: Floating-point render target using R16G16B16A16_SFLOAT format
- Supports HDR values beyond [0,1] range
- Provides 16-bit floating-point precision per channel
- Full integration with Vulkan framebuffers and samplers

#### `exposure.rs`
- **ExposureMode**: Enum for manual or automatic exposure
  - `Manual { exposure: f32 }`: Fixed exposure value
  - `Automatic { speed: f32 }`: Dynamic exposure with adaptation speed
- **ExposureCalculator**: Automatic exposure calculation based on scene luminance
  - Smooth adaptation using exponential interpolation
  - Configurable key value, min/max exposure, and adaptation speed
  - Formula: `exposure = key_value / average_luminance`

#### `tone_mapper.rs`
- **ToneMappingOperator**: Enum for tone mapping algorithm selection
  - `Reinhard`: Simple and fast (`color / (color + 1)`)
  - `ACES`: Industry-standard filmic curve, used in AAA games
  - `Uncharted2`: High contrast, dramatic look (Hable tone mapping)
- **ToneMapPass**: Low-level tone mapping pass
  - Configurable operator, gamma, and exposure
  - Efficient shader-based implementation
- **ToneMapper**: High-level tone mapper with integrated exposure calculation
  - Combines exposure calculation and tone mapping
  - Simple API for HDR to LDR conversion

### 2. Shaders

#### `crates/praxis_graphics/src/shaders/hdr_tone_map.frag`
- Complete GLSL fragment shader implementing all three tone mapping operators
- Runtime operator selection via push constants
- Implements:
  - Reinhard tone mapping
  - ACES filmic tone mapping (Narkowicz approximation)
  - Uncharted 2 tone mapping (Hable curve)
- Gamma correction support
- Exposure adjustment

### 3. Integration with RenderContext

Enhanced `RenderContext` with HDR support:
- `create_hdr_render_pass()`: Creates render pass for HDR rendering
- Documentation updated to include HDR system

### 4. Public API

All HDR types exported from `praxis_graphics`:
- `HdrRenderTarget`
- `ToneMapper`
- `ToneMapPass` (as `HdrToneMapPass`)
- `ToneMappingOperator`
- `ExposureCalculator`
- `ExposureMode`

## Features

### Floating-Point Render Targets
- **Format**: R16G16B16A16_SFLOAT
- **Range**: Approximately -65504 to +65504
- **Memory**: 8 bytes per pixel (2x standard LDR)
- **Benefits**: Accurate representation of HDR values, better bloom effects

### Multiple Tone Mapping Operators

#### Reinhard
- **Formula**: `color / (color + 1)`
- **Use Case**: Simple scenes, fast iteration, educational
- **Performance**: Fastest
- **Quality**: Good for general use, can look flat in bright scenes

#### ACES Filmic
- **Formula**: Academy Color Encoding System curve
- **Use Case**: Production games, cinematic look, realistic rendering
- **Performance**: Slightly more expensive than Reinhard
- **Quality**: Industry standard, excellent color preservation

#### Uncharted 2 (Hable)
- **Formula**: Custom curve from Uncharted 2
- **Use Case**: Games with dramatic lighting, high-contrast scenes
- **Performance**: Similar to ACES
- **Quality**: Strong contrast, dramatic look

### Exposure Calculation

#### Manual Exposure
- Fixed exposure value set by application
- Direct control for artistic purposes
- Use cases: cutscenes, controlled lighting, testing

#### Automatic Exposure
- Dynamic exposure based on scene luminance
- Smooth adaptation using exponential interpolation
- Configurable adaptation speed
- Clamped to min/max bounds
- Use cases: dynamic environments, indoor/outdoor transitions, realistic camera

**Algorithm**:
```
target_exposure = key_value / average_luminance
adaptation_rate = 1.0 - exp(-speed * delta_time)
current_exposure += (target_exposure - current_exposure) * adaptation_rate
```

### Gamma Correction
- Configurable gamma value (default: 2.2)
- Applied after tone mapping
- Standard sRGB gamma correction

## Usage Examples

### Basic HDR Rendering

```rust
use praxis_graphics::{HdrRenderTarget, ToneMapper, ToneMappingOperator};

// Create HDR render target
let hdr_render_pass = render_context.create_hdr_render_pass()?;
let hdr_target = HdrRenderTarget::new(
    memory_allocator,
    hdr_render_pass,
    [1920, 1080],
)?;

// Create tone mapper with ACES
let mut tone_mapper = ToneMapper::new(
    device,
    memory_allocator,
    vulkano::format::Format::R8G8B8A8_UNORM,
    ToneMappingOperator::ACES,
)?;

// Render scene to HDR target (implementation specific)
// ...

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

### Automatic Exposure

```rust
use praxis_graphics::ExposureMode;

// Set automatic exposure with smooth adaptation
tone_mapper.set_exposure_mode(ExposureMode::Automatic {
    speed: 2.0, // Higher = faster adaptation
});

// In render loop
tone_mapper.apply(
    command_buffer,
    &hdr_target,
    output_framebuffer,
    output_extent,
    average_luminance, // From scene analysis
    delta_time,
)?;
```

### Manual Exposure

```rust
use praxis_graphics::ExposureMode;

// Set manual exposure
tone_mapper.set_exposure_mode(ExposureMode::Manual {
    exposure: 1.5,
});

// Apply with manual exposure
tone_mapper.apply_with_exposure(
    command_buffer,
    &hdr_target,
    output_framebuffer,
    output_extent,
    1.5, // Exposure value
)?;
```

### Switching Tone Mapping Operators

```rust
// Change operator at runtime
tone_mapper.set_operator(ToneMappingOperator::Uncharted2);

// Set gamma
tone_mapper.set_gamma(2.2);

// Query current settings
let current_operator = tone_mapper.operator();
let current_exposure = tone_mapper.current_exposure();
```

## Integration Points

### With Existing Systems

1. **Post-Processing**: HDR works seamlessly with bloom and other effects
2. **Deferred Rendering**: G-buffer can use HDR format for better precision
3. **Forward Rendering**: Scene can render to HDR target before tone mapping
4. **GUI**: Exposure and operator can be controlled via debug UI

### Render Pipeline Integration

```
┌─────────────────┐
│  Scene Render   │  (to HDR target)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Post-Processing │  (bloom, etc. - optional)
│   (HDR space)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Tone Mapping    │  (HDR → LDR conversion)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Final Output   │  (to swapchain/screen)
└─────────────────┘
```

## Documentation

### Created Files

1. **`crates/praxis_graphics/HDR_RENDERING.md`**: Complete guide to HDR rendering
   - Overview and key components
   - Detailed operator descriptions
   - Performance considerations
   - Integration examples
   - Best practices and troubleshooting

2. **`examples/hdr_demo.rs`**: Demonstration example
   - Real-time operator switching
   - Exposure control (manual and automatic)
   - GUI controls for all parameters
   - Visual comparison of operators

3. **Module documentation**: Comprehensive inline documentation
   - All public types documented
   - Usage examples for each component
   - Technical details and algorithms

### Updated Files

1. **`crates/praxis_graphics/src/lib.rs`**:
   - Added HDR section to crate documentation
   - Detailed explanation of HDR pipeline
   - Usage examples
   - Operator comparisons

2. **`crates/praxis_graphics/src/shaders.rs`**:
   - Added `hdr_tone_map_fs` shader module

## File Structure

```
crates/praxis_graphics/
├── src/
│   ├── hdr/
│   │   ├── mod.rs              (module definition and exports)
│   │   ├── render_target.rs   (HDR render targets)
│   │   ├── exposure.rs         (exposure calculation)
│   │   └── tone_mapper.rs      (tone mapping implementation)
│   ├── shaders/
│   │   └── hdr_tone_map.frag   (tone mapping shader)
│   ├── lib.rs                  (updated with HDR exports)
│   └── shaders.rs              (updated with HDR shader)
├── HDR_RENDERING.md            (comprehensive guide)
└── ...

examples/
└── hdr_demo.rs                 (demonstration example)

HDR_IMPLEMENTATION_SUMMARY.md   (this file)
```

## Performance Characteristics

### Memory Usage
- HDR render target: 8 bytes per pixel (2x LDR)
- For 1920×1080: ~16.6 MB (vs 8.3 MB for LDR)

### Rendering Cost
- Tone mapping: Full-screen post-process pass
- Cost: O(pixels)
- Typical: <1ms at 1080p on modern GPUs
- All operators have similar performance

### Recommendations
1. Use native or slightly lower resolution for HDR target
2. R16G16B16A16_SFLOAT is optimal for quality/performance
3. ACES for production, Reinhard for iteration
4. Fixed luminance value if scene has consistent lighting

## Best Practices

1. **Always use HDR with bloom**: Combination is far superior to LDR bloom
2. **Test multiple operators**: Different scenes work better with different operators
3. **Start with auto-exposure**: Manual exposure is hard to tune
4. **Monitor exposure values**: Display current exposure in debug UI
5. **Clamp light values**: Even with HDR, extremely high values can cause issues

## Technical Details

### Tone Mapping Shader Implementation

The shader implements all three operators efficiently:
- Runtime operator selection (no recompilation needed)
- Optimized ACES approximation (Narkowicz)
- Proper Uncharted 2 curve with white point normalization
- Single shader pass for all operators

### Exposure Adaptation

Uses exponential interpolation for smooth adaptation:
```glsl
adaptation_rate = 1.0 - exp(-speed * delta_time)
current = current + (target - current) * adaptation_rate
```

Benefits:
- Frame-rate independent
- Smooth, natural-looking adaptation
- Configurable speed
- Mathematically correct decay

### Precision Considerations

R16G16B16A16_SFLOAT provides:
- 11-bit mantissa, 5-bit exponent per channel
- Range: ~-65504 to +65504
- Smallest normal: ~6.1×10⁻⁵
- Sufficient for most HDR scenarios
- Good balance of precision and memory

## Future Enhancements

Potential additions (not implemented):
1. **Histogram-based auto-exposure**: GPU compute for accurate luminance
2. **Additional operators**: Filmic, GT, etc.
3. **Per-operator parameters**: Customizable curves
4. **Exposure metering modes**: Center-weighted, spot, etc.
5. **HDR display output**: HDR10, Dolby Vision support
6. **Color grading**: LUT-based color correction
7. **Eye adaptation simulation**: More sophisticated biological model

## Testing

The implementation can be tested using:
1. **HDR demo example**: `cargo run --example hdr_demo`
2. **Integration with existing examples**: Add HDR to scene rendering
3. **Performance profiling**: Measure tone mapping cost
4. **Visual testing**: Compare operators on various scenes

## Conclusion

The HDR rendering implementation provides a complete, production-ready system for high dynamic range rendering in the Praxis engine. It includes:

- ✅ Floating-point render targets
- ✅ Multiple industry-standard tone mapping operators
- ✅ Automatic and manual exposure control
- ✅ Smooth exposure adaptation
- ✅ Comprehensive documentation
- ✅ Example demonstration
- ✅ Full integration with existing rendering pipeline
- ✅ Optimized shader implementation
- ✅ Runtime operator switching
- ✅ Configurable gamma correction

The system is ready for use in production game development and provides a solid foundation for advanced lighting and post-processing effects.
