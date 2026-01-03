# HDR Module

High Dynamic Range rendering system for Praxis Graphics.

## Overview

This module provides a complete HDR rendering pipeline with:
- Floating-point render targets (R16G16B16A16_SFLOAT)
- Automatic and manual exposure calculation
- Multiple tone mapping operators (ACES, Reinhard, Uncharted 2)

## Quick Example

```rust
use praxis_graphics::hdr::{HdrRenderTarget, ToneMapper, ToneMappingOperator};

// Create HDR render target
let hdr_target = HdrRenderTarget::new(memory_allocator, render_pass, [1920, 1080])?;

// Create tone mapper
let mut tone_mapper = ToneMapper::new(
    device,
    memory_allocator,
    format,
    ToneMappingOperator::ACES,
)?;

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

## Modules

- `render_target`: HDR render targets with floating-point precision
- `exposure`: Automatic and manual exposure calculation
- `tone_mapper`: Tone mapping operators and high-level API

## Key Types

- `HdrRenderTarget`: Floating-point render target
- `ToneMapper`: Complete tone mapping system with exposure
- `ToneMapPass`: Low-level tone mapping pass
- `ExposureCalculator`: Exposure calculation engine
- `ToneMappingOperator`: Operator selection (ACES, Reinhard, Uncharted2)
- `ExposureMode`: Manual or automatic exposure

## Documentation

See the parent directory for detailed guides:
- `HDR_RENDERING.md`: Complete guide
- `HDR_QUICK_START.md`: Quick start guide

## Example

Run the HDR demo:
```bash
cargo run --example hdr_demo
```
