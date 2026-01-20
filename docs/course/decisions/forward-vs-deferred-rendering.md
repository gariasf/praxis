# Decision Tree: Forward vs Deferred Rendering

```
┌──────────────────────────────────────────────────┐
│ Should I use Forward or Deferred Rendering?      │
└──────────────────────────────────────────────────┘
                        │
                        ▼
        ┌───────────────────────────────────┐
        │ How many dynamic lights do you    │
        │ need in a scene?                  │
        └───────────────────────────────────┘
                /               \
               /                 \
         < 10 lights         > 20 lights
              │                    │
              ▼                    ▼
    ┌──────────────────┐    ┌──────────┐
    │ More questions → │    │ Deferred │
    └──────────────────┘    │ (strong  │
              │             │  rec.)   │
              ▼             └──────────┘
    ┌──────────────────────┐
    │ What's your target   │
    │ platform?            │
    └──────────────────────┘
          /          \
         /            \
    Desktop/        Mobile/
    Console          Web
       │               │
       ▼               ▼
┌──────────────┐  ┌─────────┐
│ Either works │  │ Forward │
│ More Qs →    │  │ (strong │
└──────────────┘  │  rec.)  │
       │          └─────────┘
       ▼
┌──────────────────────┐
│ Do you need many     │
│ transparent objects? │
└──────────────────────┘
      /         \
     /           \
   Yes            No
    │             │
    ▼             ▼
┌─────────┐  ┌──────────┐
│ Forward │  │ Deferred │
└─────────┘  └──────────┘
```

## Quick Decision Matrix

| Factor | Forward | Deferred |
|--------|---------|----------|
| **Few lights (< 10)** | ✅ Simpler | ⚠️ Overkill |
| **Many lights (> 20)** | ❌ Poor perf | ✅ Excellent |
| **Transparency** | ✅ Native support | ❌ Complex workarounds |
| **MSAA** | ✅ Hardware MSAA | ❌ Manual resolve |
| **Memory bandwidth** | ✅ Low | ❌ High (G-buffer) |
| **Mobile/Web** | ✅ Better fit | ❌ Bandwidth issues |
| **Material variety** | ✅ Flexible | ⚠️ Shader explosion |
| **Draw call count** | ⚠️ Matters | ✅ Less critical |
| **Implementation** | ✅ Simpler | ⚠️ More complex |

## Detailed Analysis

### Forward Rendering

**How it works:**
```
For each object:
    For each light:
        Accumulate lighting
    Apply material
    Write to framebuffer
```

**Pipeline:**
```
Vertex Shader → Fragment Shader → Blend/Depth Test → Framebuffer
                  ↓
            [Light loop happens here]
            Iterate all lights affecting fragment
```

#### Choose Forward If:

**✅ High Priority:**
- **Few dynamic lights** (< 10 per scene)
- **Mobile/Web** target (limited bandwidth)
- **Heavy transparency** (particles, glass, water)
- Need **MSAA** (multi-sample anti-aliasing)
- **Simple lighting** requirements
- **Memory constrained** devices

**Example Use Cases:**
- Mobile games with simple lighting
- Stylized/cartoon rendering (few lights)
- Particle-heavy games
- VR applications (bandwidth sensitive)
- Web games (WebGL constraints)

**Pros:**
- **Simplicity**: Straightforward pipeline, easier to implement
- **Transparency**: Native alpha blending support
- **MSAA**: Hardware MSAA works out of the box
- **Memory**: No G-buffer overhead (~50% less memory)
- **Bandwidth**: Lower memory bandwidth usage
- **Flexibility**: Different materials per object easy
- **Debugging**: Easier to debug (fewer passes)

**Cons:**
- **Light scaling**: O(lights × objects) complexity
- **Overdraw**: Shading wasted on occluded pixels
- **Light limits**: Performance degrades with many lights
- **Shader variants**: Need variants for different light counts
- **Forward+**: Need advanced techniques for many lights

**Praxis Implementation:**
```rust
// Forward rendering in Praxis
let forward_pass = RenderPass {
    attachments: vec![
        color_attachment,
        depth_attachment,
    ],
};

// In fragment shader:
// - Iterate all lights
// - Accumulate lighting
// - Apply material
// - Output final color
```

**Performance Profile:**
```
Scene: 1000 objects, 5 lights
Forward:  ~3ms per frame
Deferred: ~4ms per frame (overhead not worth it)

Scene: 1000 objects, 50 lights
Forward:  ~45ms per frame (unplayable)
Deferred: ~8ms per frame (smooth)
```

### Deferred Rendering

**How it works:**
```
Pass 1 (Geometry): Render all objects to G-buffer
Pass 2 (Lighting): For each light, compute lighting from G-buffer
Pass 3 (Composite): Combine results
```

**Pipeline:**
```
Geometry Pass:
    Vertex Shader → Fragment Shader → G-Buffer
                       ↓
                Write: Position, Normal, Albedo, Material

Lighting Pass:
    For each light:
        Read G-Buffer
        Compute lighting
        Accumulate to light buffer

Composite Pass:
    Combine light + albedo + effects → Final framebuffer
```

#### Choose Deferred If:

**✅ High Priority:**
- **Many dynamic lights** (> 20 per scene)
- **Desktop/Console** target (good bandwidth)
- Target is **high-end** (3GB+ VRAM)
- Need **screen-space effects** (SSAO, SSR)
- **Consistent lighting** model across objects
- **Complex lighting** (area lights, GI approximations)

**Example Use Cases:**
- Open-world games with many lights
- Urban environments (street lights, neon signs)
- Horror games (many flashlights/torches)
- AAA desktop/console titles
- Architectural visualization

**Pros:**
- **Light scaling**: O(lights + objects) complexity
- **Many lights**: Handles hundreds of lights efficiently
- **Decoupled**: Geometry separate from lighting
- **Screen-space effects**: Easy to add SSAO, SSR, etc.
- **Culling**: Light volumes cull per-pixel
- **Consistency**: Single lighting model for all objects

**Cons:**
- **Memory**: G-buffer requires significant VRAM (3-4 render targets)
- **Bandwidth**: High memory bandwidth usage
- **Transparency**: Requires separate forward pass
- **MSAA**: Manual resolve needed (expensive)
- **Material variants**: Limited material types (shader explosion)
- **Mobile**: Often prohibitive on mobile GPUs
- **Complexity**: More passes, more debugging

**Praxis Implementation:**
```rust
// Deferred rendering in Praxis
// Pass 1: Geometry
let g_buffer = GBuffer {
    position: create_image(format::R16G16B16A16_SFLOAT),
    normal: create_image(format::R16G16B16A16_SFLOAT),
    albedo: create_image(format::R8G8B8A8_UNORM),
    material: create_image(format::R8G8B8A8_UNORM),
};

// Pass 2: Lighting
for light in lights {
    render_light_volume(&light, &g_buffer);
}

// Pass 3: Transparent objects (forward)
render_transparent_objects();
```

**Performance Profile:**
```
Scene: 1000 objects, 5 lights
Forward:  ~3ms per frame (simpler wins)
Deferred: ~4ms per frame

Scene: 1000 objects, 50 lights
Forward:  ~45ms per frame
Deferred: ~8ms per frame (scales much better)

Scene: 1000 objects, 200 lights
Forward:  ~180ms per frame (unplayable)
Deferred: ~15ms per frame (playable)
```

## Hybrid Approaches

### Forward+ (Tiled Forward Rendering)

**How it works:**
```
1. Depth prepass
2. Tile frustum culling → Build light lists per tile
3. Forward render with per-tile light lists
```

**When to use:**
- Need many lights AND transparency
- Want forward simplicity with deferred scaling
- Compute shader support available
- Modern GPU (compute shaders)

**Pros:**
- Scales well with light count (like deferred)
- Native transparency support (like forward)
- No G-buffer overhead

**Cons:**
- Complex implementation
- Requires compute shaders
- Still does some redundant shading

**Example:**
```rust
// Forward+ pseudo-code
// Pass 1: Depth prepass
render_depth_only();

// Pass 2: Light culling (compute shader)
let light_grid = cull_lights_per_tile(depth_buffer);

// Pass 3: Forward render with light grid
for object in objects {
    let tile = get_tile(object.screen_pos);
    let lights = light_grid[tile];
    shade(object, lights); // Only relevant lights
}
```

### Clustered Forward/Deferred

**How it works:**
```
Divide frustum into 3D clusters (not just 2D tiles)
Cull lights per cluster
Apply in forward or deferred pass
```

**When to use:**
- Need better light culling than Forward+
- Depth varies significantly in scene
- AAA production with engineering resources

**Pros:**
- Best light culling (3D clusters vs 2D tiles)
- Scales to thousands of lights
- Works with forward or deferred

**Cons:**
- Most complex implementation
- Higher overhead for few lights
- Requires significant engineering

### Deferred + Forward (Hybrid)

**How it works:**
```
1. Deferred pass for opaque objects
2. Forward pass for transparent objects
```

**When to use:**
- Most common real-world approach
- Need many lights AND transparency
- Desktop/console target

**Praxis uses this:**
```rust
// 1. Deferred for opaque
deferred_renderer.render_opaque(opaque_objects);

// 2. Forward for transparent
forward_renderer.render_transparent(transparent_objects);
```

## Platform-Specific Guidance

### Desktop (High-End PC)
**Recommendation: Deferred or Forward+**

Desktop GPUs have:
- High memory bandwidth
- Large VRAM (8GB+)
- Fast compute shaders

**Choose deferred if:**
- Many lights (typical in modern games)
- Using PBR with complex lighting
- Building AAA-style visuals

**Choose forward if:**
- Stylized art style (few lights)
- Heavy transparency (effects-focused game)

### Console (PS5, Xbox Series X)
**Recommendation: Deferred or Clustered**

Similar to desktop but:
- Fixed hardware (can optimize heavily)
- Good bandwidth management
- Unified memory architecture

Most AAA console games use deferred or clustered.

### Mobile (iOS, Android)
**Strong recommendation: Forward**

Mobile GPUs are:
- Bandwidth constrained
- Tile-based (TBDR architecture)
- Limited VRAM

Deferred's G-buffer thrashes the tile cache.

**Exception:** High-end mobile (iPhone 15 Pro) can handle lightweight deferred.

### VR
**Strong recommendation: Forward or Forward+**

VR requires:
- Extremely high frame rates (90+ FPS)
- Low latency
- Bandwidth efficiency (render 2 eyes)

Deferred's bandwidth cost is multiplied by 2 (stereo rendering).

**Technique:** Use Forward+ with foveated rendering.

### Web (WebGL/WebGPU)
**Strong recommendation: Forward**

Web constraints:
- Limited bandwidth
- Varied hardware
- Shader compilation overhead

Deferred adds too much complexity and bandwidth for web.

## Material System Implications

### Forward: Flexible Materials

Each object can have completely different material:
```rust
// Easy in forward rendering
struct MetalMaterial { roughness, metallic }
struct ClothMaterial { fuzz, subsurface }
struct GlassMaterial { IOR, thickness }

// Each has own shader
```

**Benefit:** Artistic flexibility

**Cost:** Many shader permutations

### Deferred: Unified Materials

All objects must fit same G-buffer layout:
```rust
// G-buffer layout (fixed for all objects)
struct GBuffer {
    position: vec3,
    normal: vec3,
    albedo: vec3,
    roughness: f32,
    metallic: f32,
    // Can't easily add per-material data
}
```

**Benefit:** Consistent lighting

**Cost:** Limited material variety

**Solution:** Use material IDs and lookup textures or switch to Forward+

## Transparency Handling

### Forward: Native Transparency
```rust
// Just blend transparents after opaques
render_opaque_objects();
render_transparent_objects(); // Alpha blending works
```

**Simple and correct.**

### Deferred: Transparent Problem

G-buffer stores only one depth layer. Transparents need special handling:

**Option 1:** Forward pass for transparents (Praxis approach)
```rust
render_deferred_opaques();
render_forward_transparents(); // Separate pass
```

**Option 2:** Depth peeling (expensive)
```rust
for layer in 0..MAX_LAYERS {
    render_layer_to_g_buffer(layer);
}
```

**Option 3:** Approximate with screen-space blending
```rust
// Render transparents to separate buffer
// Blend with deferred result
```

## Anti-Aliasing

### Forward: Easy MSAA

Hardware MSAA works naturally:
```rust
let color_image = Image {
    samples: SampleCount::Sample4, // 4x MSAA
    // Hardware handles everything
};
```

**Clean and fast.**

### Deferred: MSAA Problems

G-buffer with MSAA requires 4x memory (per sample):
- Position: 4x samples
- Normal: 4x samples  
- Albedo: 4x samples
- Material: 4x samples

**Memory explodes!**

**Solutions:**
1. **Post-process AA** (FXAA, TAA): Praxis uses this
2. **Hybrid MSAA**: Only some G-buffer targets
3. **Compute-based resolve**: Manual sample resolution

```rust
// Praxis approach: TAA (Temporal Anti-Aliasing)
// Cheaper than G-buffer MSAA
let taa = TemporalAntiAliasing::new();
taa.resolve(current_frame, previous_frame);
```

## Memory & Bandwidth Analysis

### Forward Rendering

**Memory:**
```
Color buffer:  1920x1080 × 4 bytes = 8.3 MB
Depth buffer:  1920x1080 × 4 bytes = 8.3 MB
Total: ~16 MB
```

**Bandwidth (per frame):**
```
Write color: 8.3 MB
Write depth: 8.3 MB  
Read depth (for depth test): 8.3 MB
Total: ~25 MB per frame
```

### Deferred Rendering

**Memory:**
```
G-Buffer:
  Position:  1920x1080 × 16 bytes = 33 MB
  Normal:    1920x1080 × 16 bytes = 33 MB
  Albedo:    1920x1080 × 4 bytes  = 8.3 MB
  Material:  1920x1080 × 4 bytes  = 8.3 MB
Depth:       1920x1080 × 4 bytes  = 8.3 MB
Light:       1920x1080 × 8 bytes  = 16.6 MB
Total: ~107 MB (6.5x more than forward)
```

**Bandwidth (per frame):**
```
Write G-Buffer: 82.6 MB
Read G-Buffer (lighting): 82.6 MB
Write light buffer: 16.6 MB
Read for composite: 16.6 MB
Total: ~198 MB per frame (8x more than forward)
```

**Impact:**
- Desktop GPUs: Fine (200+ GB/s bandwidth)
- Mobile GPUs: Problem (10-20 GB/s bandwidth)

## Implementation Complexity

### Forward Rendering (Simpler)

**Code estimate:** ~1000 lines

```rust
// Pseudo-code structure
struct ForwardRenderer {
    pipeline: GraphicsPipeline,
    lights: Vec<Light>,
}

impl ForwardRenderer {
    fn render(&self, objects: &[Object]) {
        for object in objects {
            bind_object_data(object);
            bind_lights(&self.lights);
            draw(object);
        }
    }
}
```

**Challenges:**
- Shader permutations (different light counts)
- Light culling (if optimizing)

### Deferred Rendering (More Complex)

**Code estimate:** ~2500 lines

```rust
// Pseudo-code structure
struct DeferredRenderer {
    geometry_pass: GeometryPass,
    lighting_pass: LightingPass,
    composite_pass: CompositePass,
    g_buffer: GBuffer,
}

impl DeferredRenderer {
    fn render(&self, objects: &[Object], lights: &[Light]) {
        // Pass 1: Geometry
        self.geometry_pass.render(objects, &self.g_buffer);
        
        // Pass 2: Lighting
        for light in lights {
            self.lighting_pass.render_light(light, &self.g_buffer);
        }
        
        // Pass 3: Composite
        self.composite_pass.render(&self.g_buffer);
        
        // Pass 4: Forward transparents
        render_forward_transparents(objects);
    }
}
```

**Challenges:**
- Multiple render passes
- G-buffer management
- Transparency handling
- Manual anti-aliasing
- More shader code

## Performance Benchmarks

### Scenario 1: Simple Scene (Few Lights)
```
Scene: 1000 objects, 5 point lights
Target: 60 FPS (16.6ms budget)

Forward:
  Geometry: 2ms
  Lighting: 1ms (5 lights)
  Total: 3ms ✅

Deferred:
  Geometry pass: 1.5ms
  Lighting pass: 1ms
  Composite: 0.5ms
  Overhead: 1ms
  Total: 4ms ✅

Winner: Forward (simpler and slightly faster)
```

### Scenario 2: Many Lights
```
Scene: 1000 objects, 100 point lights
Target: 60 FPS (16.6ms budget)

Forward:
  Geometry: 2ms
  Lighting: 80ms (100 lights × many pixels)
  Total: 82ms ❌ (unplayable)

Deferred:
  Geometry pass: 1.5ms
  Lighting pass: 5ms (light volumes culled)
  Composite: 0.5ms
  Total: 7ms ✅

Winner: Deferred (scales much better)
```

### Scenario 3: Heavy Transparency
```
Scene: 100 objects, 20 lights, 5000 particles
Target: 60 FPS (16.6ms budget)

Forward:
  Opaques: 2ms
  Transparents: 8ms (native blending)
  Total: 10ms ✅

Deferred:
  Deferred opaques: 3ms
  Forward transparents: 10ms (separate pass overhead)
  Total: 13ms ⚠️

Winner: Forward (simpler transparency)
```

## Decision Checklist

Mark your answers:

| Question | Forward | Deferred |
|----------|---------|----------|
| Target is mobile/web? | ✓ | |
| Target is desktop/console? | | ✓ |
| < 10 dynamic lights? | ✓ | |
| > 20 dynamic lights? | | ✓ |
| Heavy transparency? | ✓ | |
| Mostly opaque objects? | | ✓ |
| Need hardware MSAA? | ✓ | |
| Want screen-space effects? | | ✓ |
| Limited memory bandwidth? | ✓ | |
| High-end GPU target? | | ✓ |
| Simple implementation priority? | ✓ | |
| Scalability priority? | | ✓ |

**Score:**
- **Mostly Forward**: Use forward rendering
- **Mostly Deferred**: Use deferred rendering
- **Tied**: Consider hybrid (deferred + forward for transparents)

## Migration Path

### Starting with Forward, Moving to Deferred

**When to migrate:**
- Light count growing unmanageable
- Performance degrades with complex scenes
- Adding screen-space effects

**Migration steps:**
1. **Add G-buffer output** to existing shaders
2. **Separate lighting** into dedicated pass
3. **Keep forward path** for transparents
4. **Benchmark** to verify improvement

### Starting with Deferred, Moving to Forward+

**When to migrate:**
- Need better transparency handling
- Want reduced memory footprint
- Targeting wider range of hardware

**Migration steps:**
1. **Add depth prepass**
2. **Implement tile-based light culling** (compute shader)
3. **Replace deferred pass** with forward pass using light lists
4. **Remove G-buffer**

## Recommended Reading

- **Forward Rendering:**
  - Real-Time Rendering, 4th Edition - Chapter 7
  - [Learn OpenGL - Lighting](https://learnopengl.com/Lighting/Colors)

- **Deferred Rendering:**
  - [Deferred Shading in Tabula Rasa](https://developer.nvidia.com/gpugems/gpugems2/part-ii-shading-lighting-and-shadows/chapter-9-deferred-shading-tabula-rasa)
  - [Deferred Rendering in Killzone 2](https://www.guerrilla-games.com/read/deferred-rendering-in-killzone-2)

- **Forward+:**
  - [Forward+ Rendering](https://takahiroharada.files.wordpress.com/2015/04/forward_plus.pdf)

- **Clustered:**
  - [Clustered Deferred and Forward Shading](https://www.humus.name/Articles/PracticalClusteredShading.pdf)

- **Praxis Documentation:**
  - `docs/guides/rendering.md`
  - `docs/guides/rendering/hdr-tonemapping.md`
  - `crates/praxis_graphics/README.md`

## Conclusion

**TL;DR:**
- **Few lights, mobile, transparency? → Forward**
- **Many lights, desktop, opaque-heavy? → Deferred**
- **Many lights + transparency? → Forward+ or Hybrid**
- **Learning project? → Start with Forward, add Deferred later**

**Praxis Choice:** Deferred + Forward hybrid
- Deferred for opaque geometry (scales to many lights)
- Forward for transparent objects (particles, effects)
- TAA instead of G-buffer MSAA (memory efficient)

This hybrid approach provides:
- Good light scalability (deferred)
- Correct transparency (forward)
- Reasonable memory footprint (no MSAA G-buffer)
- Educational value (demonstrates both techniques)

**Your choice depends on:**
1. Target platform capabilities
2. Lighting complexity needs
3. Transparency requirements
4. Team experience and timeline

Neither is universally better - match the technique to your constraints.
