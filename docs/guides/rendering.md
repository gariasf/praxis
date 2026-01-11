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

## Design Rationale and Tradeoffs

### Why Two Rendering Pipelines?

Praxis provides both forward and deferred rendering because no single approach is optimal for all scenarios. This dual-pipeline architecture acknowledges that different game types have fundamentally different rendering characteristics.

**Key Design Decision**: Rather than forcing a single rendering path, we provide both and make switching between them trivial. This allows developers to:
- Profile their specific scene characteristics
- Use the optimal renderer for each frame or scene
- Implement hybrid approaches (deferred for opaque, forward for transparent)

### Forward Rendering Architecture Decisions

**Decision**: Use per-object lighting calculations in fragment shader

**Rationale**:
- Simplest implementation with fewest render passes (single pass)
- Natural support for transparency through alpha blending
- Hardware MSAA support without additional complexity
- Lower memory bandwidth requirements (no G-buffer)
- Optimal for mobile and low-end hardware

**Alternatives Considered**:
1. **Light pre-pass**: Rejected due to still requiring multiple passes and geometry buffer
2. **Clustered forward**: Deferred for future optimization; adds significant complexity
3. **Forward+**: Complexity doesn't justify benefits for target use cases (indie games, prototypes)

**Performance Characteristics**:
- Complexity: O(objects × triangles × lights)
- Best case: 1-3 lights, simple geometry, transparency heavy
- Worst case: 10+ lights, dense geometry, no transparency
- Memory: Minimal (no G-buffer, ~8MB for 1080p framebuffer)
- Bandwidth: Low (single render target write)

**Tradeoffs**:
- ✓ Simple, predictable performance
- ✓ Full transparency support
- ✓ MSAA works seamlessly
- ✗ Scales poorly with light count
- ✗ Redundant shading of hidden pixels
- ✗ Light-geometry coupling limits flexibility

### Deferred Rendering Architecture Decisions

**Decision**: Two-pass rendering with G-buffer for geometry data

**Rationale**:
- Decouples geometry complexity from lighting complexity
- Only shades visible pixels (passes depth test)
- Enables efficient multi-light scenarios (10+ lights with minimal overhead)
- Provides foundation for advanced effects (SSAO, SSR, decals)

**G-Buffer Layout Decisions**:

| Choice | Rationale | Alternatives Rejected |
|--------|-----------|----------------------|
| R8G8B8A8 for albedo | Sufficient color precision, compact | R16G16B16A16 (wasteful), BC1 compression (quality loss) |
| R16G16B16A16_SFLOAT for normals | High precision needed for lighting | R8G8B8A8 (banding), octahedral encoding (complexity vs quality) |
| R8G8B8A8 for material | Metallic/roughness don't need high precision | R16G16B16A16 (overkill), R16G16 (insufficient channels) |
| Depth-based position reconstruction | Saves 16 bytes/pixel (~33MB at 1080p) | Store world position (memory waste) |

**Total G-buffer cost**: ~20 bytes/pixel = 41MB at 1080p vs 33MB if storing explicit position

**Why Not Store Position?**
- Mathematical reconstruction from depth is cheap (2-3 ALU ops)
- Saved bandwidth (one fewer render target) improves performance more than reconstruction cost
- Modern GPUs have excellent depth buffer compression

**Alternatives Considered**:
1. **Light pre-pass (deferred lighting)**: 
   - Rejected: Still requires second geometry pass for materials, no bandwidth savings
2. **Tiled/clustered deferred**:
   - Deferred: Excellent for 100+ lights but adds complexity; will implement if needed
3. **Inferred rendering**:
   - Rejected: Requires edge detection and reconstruction, too complex for benefits

**Performance Characteristics**:
- Complexity: O(objects × triangles) + O(pixels × lights)
- Best case: 5+ lights, limited transparency, complex geometry
- Worst case: <3 lights, heavy transparency, simple geometry
- Memory: ~41MB for 1080p G-buffer
- Bandwidth: High (multiple render target writes + reads)

**Tradeoffs**:
- ✓ Scales excellently with light count
- ✓ Only shades visible pixels
- ✓ Material-lighting decoupling enables flexibility
- ✓ Foundation for advanced post-processing
- ✗ Transparency requires separate pass
- ✗ MSAA extremely expensive (multiple render targets)
- ✗ Higher memory bandwidth usage
- ✗ Two render passes increase CPU overhead

### Lighting Model: Why Blinn-Phong with PBR Parameters?

**Decision**: Use Blinn-Phong lighting extended with metallic-roughness workflow

**Rationale**:
- Blinn-Phong is computationally cheap and well-understood
- PBR parameters (metallic/roughness) provide artist-friendly material authoring
- Hybrid approach: PBR authoring simplicity + Blinn-Phong performance
- Sufficient visual quality for most indie games and prototypes

**Why Not Full PBR (Cook-Torrance)?**
- Full PBR with GGX/Schlick is 3-4x more expensive per light
- Target hardware (indie games) often doesn't need physically accurate rendering
- Can upgrade to full PBR in future without changing material pipeline

**Alternatives Considered**:
1. **Pure Blinn-Phong**: Rejected; specular power authoring is unintuitive
2. **Full Cook-Torrance PBR**: Too expensive for target use cases
3. **Simplified PBR**: Current approach IS simplified PBR
4. **Disney principled BRDF**: Overkill for real-time rendering

### Shadow Implementation: Cascaded Shadow Maps

**Decision**: Use cascaded shadow maps for directional lights

**Why Cascaded?**
- Directional lights cover entire scene; single shadow map wastes resolution
- Cascades allocate resolution based on distance (more detail near camera)
- Industry standard approach with proven results

**Why Not Alternative Techniques?**
- **Perspective shadow maps**: Complex, still has artifacts
- **Variance shadow maps**: Require 16-bit textures, light bleeding artifacts
- **Ray-traced shadows**: GPU compute requirements too high for target hardware

**See**: [Shadows Guide](rendering/shadows.md) for detailed shadow implementation

### HDR and Tone Mapping Integration

**Decision**: Optional HDR pipeline with multiple tone mapping operators

**Rationale**:
- HDR allows realistic light intensity ranges
- Tone mapping is essential for displaying HDR on LDR displays
- Multiple operators (ACES, Reinhard, etc.) suit different art styles

**See**: [HDR and Tone Mapping](rendering/hdr-tonemapping.md) for detailed rationale

### Hybrid Rendering Strategy

**Key Insight**: Most scenes benefit from combining both pipelines:
- Deferred for opaque geometry with complex lighting
- Forward for transparent particles, UI, skybox
- Best of both worlds at minimal integration cost

**Implementation Pattern**:
```rust
// 1. Deferred pass: opaque objects with many lights
deferred_renderer.render(opaque_objects, lighting);

// 2. Forward pass: transparent objects
forward_renderer.render(transparent_objects, lighting);
```

**Why This Works**:
- Depth buffer shared between passes prevents overdraw
- Transparent objects naturally render after opaque (depth sorted)
- CPU overhead minimal (pipeline bind cost is low)

### Future Architectural Considerations

**Planned Enhancements**:
1. **Clustered forward/deferred**: For 100+ dynamic lights
2. **Temporal anti-aliasing (TAA)**: Better than MSAA for deferred
3. **Virtual texturing**: For large material variety
4. **GPU-driven rendering**: Reduce CPU overhead

**Why Not Now?**
- Current implementation handles target use cases (indie games)
- Premature optimization would complicate codebase
- These features require significant engineering investment
- Can add incrementally based on user needs

### When to Choose Forward vs Deferred

**Choose Forward when**:
- Scene has <5 dynamic lights
- Heavy use of transparency (particles, water, glass)
- Targeting mobile or low-end hardware
- MSAA is required for art style
- Memory bandwidth is limited

**Choose Deferred when**:
- Scene has 5+ dynamic lights
- Mostly opaque geometry
- Targeting PC/console hardware
- Need advanced post-processing (SSAO, SSR)
- Want to decouple lighting from material complexity

**Choose Hybrid when**:
- Scene has both many lights AND transparency
- Want optimal performance for varied content
- Have development time to integrate both

## Examples

```bash
cargo run --example deferred_demo      # Deferred rendering
cargo run --example comprehensive_scene_demo  # Forward with materials
```

## See Also

- [HDR and Tone Mapping](rendering/hdr-tonemapping.md) - High dynamic range
- [Shadows](rendering/shadows.md) - Cascaded shadow maps
- [Post-Processing](rendering/post-processing.md) - Screen-space effects
- [Deferred Rendering](rendering/deferred-rendering.md) - Detailed G-buffer docs
- [Advanced Rendering](rendering/advanced-rendering.md) - Pipeline deep dive
