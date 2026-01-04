# Rendering

Praxis supports both forward and deferred rendering pipelines with Vulkan via `vulkano`.

## Forward Rendering (Default)

Forward rendering processes each object through the full lighting calculation:

```text
For each object:
    For each light:
        Calculate lighting contribution
```

**Cost**: O(objects × triangles × lights)

Best for: Scenes with few lights, transparency, MSAA.

### Basic Usage

```rust
render_context.render(&RenderCommands {
    view: camera_view,
    proj: camera_proj,
    draw_commands: &objects,
    lighting: Some(&lighting),
})?;
```

### Material System

PBR metallic-roughness workflow:

```rust
let material = MaterialProperties {
    albedo: [1.0, 0.84, 0.0, 1.0],  // Gold color
    metallic: 1.0,                   // Full metal
    roughness: 0.1,                  // Polished
    emissive: 0.0,
    _padding: 0.0,
};
```

### Lighting

Dynamic directional and point lights:

```rust
lighting.directional_lights[0] = DirectionalLightData {
    direction: [0.0, -1.0, -0.5, 0.0],
    color: [1.0, 0.95, 0.8, 0.0],
    intensity: 1.0,
    _padding: [0.0; 3],
};
```

## Deferred Rendering

Separates geometry from lighting using G-buffer:

```text
Pass 1 (Geometry): Write surface properties to G-buffer
Pass 2 (Lighting): Calculate lighting per-pixel
```

**Cost**: O(objects × triangles) + O(pixels × lights)

Best for: Scenes with many lights (5+).

### G-Buffer Layout

| Buffer | Format | Content |
|--------|--------|---------|
| Albedo | R8G8B8A8_UNORM | Base color |
| Normal | R16G16B16A16_SFLOAT | World-space normal |
| Metal-Rough | R8G8B8A8_UNORM | Metallic, roughness, emissive |
| Depth | D32_SFLOAT | Depth values |

Memory: ~20 bytes/pixel (~41 MB at 1080p)

### Usage

```rust
use praxis_graphics::DeferredRenderer;

deferred_renderer.render(
    builder,
    output_framebuffer,
    viewport,
    draw_commands,
    view_proj_buffer,
    dynamic_uniform_buffer,
    mesh_manager,
    texture_manager,
    lighting_buffer,
)?;
```

### Trade-offs

**Benefits:**
- Efficient with many lights
- Only shades visible pixels
- Decoupled materials and lighting

**Limitations:**
- Higher memory bandwidth
- Transparency requires forward pass
- MSAA is expensive

## Choosing a Pipeline

| Scenario | Recommended |
|----------|-------------|
| < 5 lights | Forward |
| 5+ lights | Deferred |
| Transparency needed | Forward (or hybrid) |
| MSAA required | Forward |
| Many small lights | Deferred |

## Examples

```bash
cargo run --example deferred_demo      # Deferred rendering
cargo run --example comprehensive_scene_demo  # Forward with materials
```

## See Also

- [HDR and Tone Mapping](hdr-and-tonemapping.md) - High dynamic range
- [Shadows](shadows.md) - Cascaded shadow maps
- [Post-Processing](post-processing.md) - Screen-space effects
- [docs/deferred_rendering.md](../deferred_rendering.md) - Detailed G-buffer docs
- [docs/RENDERING_EXPLAINED.md](../RENDERING_EXPLAINED.md) - Pipeline deep dive
