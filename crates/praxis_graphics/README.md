# praxis_graphics

Vulkan-based rendering system for Praxis engine.

## Overview

Modern rendering system built on Vulkan using `vulkano` for safe Rust abstractions.

## Features

- **Forward Rendering**: Single-pass rendering with direct lighting
- **Deferred Rendering**: Multi-pass rendering for complex lighting
- **HDR & Tone Mapping**: ACES and Reinhard tone mapping
- **Shadow Mapping**: Cascaded shadow maps for directional lights
- **GPU-Driven Optimization**:
  - Frustum culling
  - Occlusion culling (Hi-Z)
  - LOD system
  - Mesh streaming
- **Debug Visualization**: Culling, LOD, occlusion overlays

## Architecture

```
RenderContext
    ├── Device & Swapchain
    ├── Pipelines
    ├── Descriptor Sets
    ├── Command Buffers
    └── Synchronization
```

## Example

```rust
use praxis_graphics::{RenderContext, Camera};

let context = RenderContext::new(window)?;
let camera = Camera::new(position, target);

// Render loop
context.begin_frame()?;
context.render_scene(&camera, &meshes)?;
context.end_frame()?;
```

## Dependencies

- `vulkano`: Safe Vulkan bindings
- `vulkano-shaders`: Shader compilation
- `vulkano-util`: Memory management
- `image`: Image loading
- `parking_lot`: Fast mutexes

## Usage

```toml
praxis_graphics = { path = "../praxis_graphics", version = "0.1.0" }
```
