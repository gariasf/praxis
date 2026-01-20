# Rendering Architecture Patterns

Rendering architecture determines how a game engine processes geometry and lighting to produce final images. The choice fundamentally affects performance, visual quality, and what rendering features are feasible.

## The Core Problem

A renderer must:
1. **Process geometry**: Transform vertices, cull invisible objects
2. **Calculate lighting**: Direct light, shadows, indirect light
3. **Apply materials**: Textures, shaders, transparency
4. **Produce final image**: Combine everything into pixels

Modern scenes have:
- Thousands to millions of triangles
- Dozens to hundreds of lights
- Complex materials (PBR, subsurface scattering)
- Post-processing effects (bloom, tone mapping, anti-aliasing)

The rendering pattern determines how these are organized and when each step happens.

## Pattern Variants

### 1. Forward Rendering

**Concept**: Render each object once, calculating all lighting in a single shader pass per object.

```
for each object in scene:
    bind_object_geometry()
    bind_material()
    
    for each light affecting object:
        accumulate_lighting()
    
    output_lit_pixel()
```

**Visual flow**:
```
3D Scene → For each object:
              Vertex Shader (transform)
              → Fragment Shader (all lighting)
              → Output pixel
```

**Shader pseudo-code**:
```glsl
// Vertex Shader
output.position = projection * view * model * input.position;
output.normal = model * input.normal;
output.uv = input.uv;

// Fragment Shader (does ALL the work)
vec3 final_color = vec3(0.0);

// Accumulate all lights
for (int i = 0; i < num_lights; i++) {
    vec3 light_contribution = calculate_lighting(
        lights[i],
        position,
        normal,
        material
    );
    final_color += light_contribution;
}

// Apply material
final_color *= texture(albedo_map, uv);

output_color = vec4(final_color, 1.0);
```

**Trade-offs**:

✅ **Strengths**:
- Simple to understand and implement
- Transparent objects are natural (render back-to-front)
- MSAA (multisample anti-aliasing) works well
- Low memory bandwidth (no G-buffer)
- Good for scenes with few lights
- VR-friendly (single pass per eye)

❌ **Weaknesses**:
- Performance scales with lights × objects (O(n×m))
- Wasted work on overdraw (shading pixels that get occluded)
- Shader complexity increases with light count
- Limited light count (typically 4-8 lights)
- Difficult to do complex lighting (global illumination, many lights)

**When to use**:
- Mobile games (lower bandwidth requirements)
- VR (need forward rendering for stereo)
- Outdoor scenes with few lights
- Stylized graphics (toon shading, simple lighting)
- Scenes with lots of transparency

**Real-world examples**:
- Mobile games (Unity mobile pipeline)
- VR games (forward renderer required)
- Minecraft (simple lighting model)
- Many Nintendo games (stylized lighting)

**Performance characteristics**:
- **Best case**: Few lights, no overdraw → fast
- **Worst case**: Many lights, high overdraw → very slow
- **Typical**: 60 FPS with 4-8 lights at 1080p

### 2. Deferred Rendering

**Concept**: Split rendering into two phases:
1. **Geometry pass**: Render scene properties to textures (G-buffer)
2. **Lighting pass**: Calculate lighting using G-buffer data

```
# Phase 1: Geometry Pass
for each object in scene:
    render_to_gbuffer(position, normal, albedo, material_properties)

# Phase 2: Lighting Pass (full-screen)
for each light in scene:
    for each pixel in screen:
        read_gbuffer(position, normal, albedo, material)
        calculate_lighting(light, surface_properties)
        accumulate_to_final_image()
```

**Visual flow**:
```
3D Scene → Geometry Pass:
             For each object:
               Output to G-buffer (position, normal, albedo, etc.)
         
         → Lighting Pass:
             For each light:
               Read G-buffer
               Calculate lighting
               Accumulate
         
         → Final Image
```

**G-buffer layout** (typical):
```
Texture 0 (RGB): World Position
Texture 1 (RGB): World Normal
Texture 2 (RGBA): Albedo + Roughness
Texture 3 (RGBA): Metallic + AO + Emission + Flags
Depth Buffer: Scene depth
```

**Shader pseudo-code**:
```glsl
// Geometry Pass Fragment Shader
output_position = world_position;
output_normal = normalize(world_normal);
output_albedo = texture(albedo_map, uv);
output_material = vec4(metallic, roughness, ao, 0.0);

// Lighting Pass Fragment Shader (full-screen quad)
vec3 position = texture(gbuffer_position, screen_uv).xyz;
vec3 normal = texture(gbuffer_normal, screen_uv).xyz;
vec3 albedo = texture(gbuffer_albedo, screen_uv).rgb;
vec4 material = texture(gbuffer_material, screen_uv);

vec3 final_color = vec3(0.0);

// Process each light
for (int i = 0; i < num_lights; i++) {
    final_color += calculate_lighting(
        lights[i],
        position,
        normal,
        albedo,
        material
    );
}

output_color = vec4(final_color, 1.0);
```

**Trade-offs**:

✅ **Strengths**:
- Performance independent of light count (O(n + m) not O(n×m))
- Supports hundreds to thousands of lights
- No wasted shading on overdraw (only visible pixels)
- Complex lighting algorithms feasible
- Screen-space effects easy (SSAO, SSR)
- Consistent performance (every pixel shaded once)

❌ **Weaknesses**:
- High memory bandwidth (read/write multiple G-buffer textures)
- No hardware MSAA (too expensive with G-buffers)
- Transparency is difficult (can't write to G-buffer)
- Material variation limited by G-buffer format
- Higher memory usage (G-buffer textures)
- Older hardware may struggle

**When to use**:
- Indoor scenes with many lights
- AAA games with complex lighting
- Scenes with limited transparency
- PC/Console (enough memory bandwidth)
- Games targeting modern hardware

**Real-world examples**:
- Most AAA games (Unreal Engine default)
- Battlefield series
- The Last of Us
- Modern indoor shooters

**Performance characteristics**:
- **Best case**: Many lights, low overdraw → excellent
- **Worst case**: High resolution, slow G-buffer reads → moderate
- **Typical**: 60 FPS with 100+ lights at 1080p

**G-buffer optimization**: Pack data efficiently
```
# Instead of 4×RGBA16F (32 bytes per pixel)
# Use optimized packing (16 bytes per pixel):
RGBA16F: Position.xyz + Material.Metallic
RGBA16F: Normal.xy (reconstruct z) + Albedo.rg
RGBA16F: Albedo.b + Roughness + AO + Emission
```

### 3. Forward+ (Forward Plus / Tiled Forward)

**Concept**: Combine forward rendering with tile-based light culling. Determine which lights affect which screen tiles, then do forward rendering per-tile.

```
# Phase 1: Light Culling (compute shader)
divide_screen_into_tiles(16×16 pixels)

for each tile:
    frustum = calculate_tile_frustum()
    visible_lights = []
    
    for each light:
        if frustum_intersects_light(frustum, light):
            visible_lights.append(light)
    
    store_light_list(tile, visible_lights)

# Phase 2: Forward Rendering (with per-tile light lists)
for each object:
    bind_geometry()
    
    for each fragment:
        tile = get_tile_for_pixel(fragment.position)
        lights = get_light_list(tile)
        
        final_color = vec3(0.0)
        for each light in lights:
            final_color += calculate_lighting(light, fragment)
        
        output_color = final_color
```

**Visual flow**:
```
3D Scene + Lights → Light Culling Pass (compute shader):
                      Build per-tile light lists
                  
                  → Forward Rendering:
                      For each object:
                        For each pixel:
                          Use tile's light list
                          Calculate lighting
                  
                  → Final Image
```

**Implementation details**:
```
Tile size: 16×16 pixels (common choice)
Screen resolution: 1920×1080
Tile count: 120×68 = 8,160 tiles

Per-tile data structure:
struct TileLightList {
    uint light_count;
    uint light_indices[MAX_LIGHTS_PER_TILE];  // e.g., 256
}

Storage: ~8MB for light lists (16×16 tiles, 256 lights max)
```

**Trade-offs**:

✅ **Strengths**:
- Supports many lights (like deferred)
- Transparent objects work (like forward)
- MSAA works (like forward)
- Lower bandwidth than deferred (no G-buffer)
- Good for both indoor and outdoor scenes
- Scalable light count

❌ **Weaknesses**:
- More complex than forward or deferred
- Requires compute shader support
- Light culling overhead
- Needs careful tuning (tile size, max lights per tile)
- Conservative culling (tiles may include lights that don't affect geometry)

**When to use**:
- Games needing both many lights and transparency
- Modern hardware (compute shaders required)
- AAA games on PC/Console
- VR games needing many lights

**Real-world examples**:
- Doom (2016) - pioneered the approach
- Unreal Engine 4/5 (optional forward+ renderer)
- Many modern AAA games
- VR games with complex lighting

**Performance characteristics**:
- **Best case**: Many small lights, good spatial distribution → excellent
- **Worst case**: Huge lights covering many tiles → overhead
- **Typical**: 60 FPS with 1000+ lights at 1080p

### 4. Clustered Rendering (3D Tiling)

**Concept**: Extend Forward+ from 2D screen tiles to 3D view frustum clusters. Cull lights per 3D cluster instead of 2D tile.

```
# Phase 1: Build 3D Grid
divide_frustum_into_3d_clusters(
    x_subdivisions=16,
    y_subdivisions=9,
    z_subdivisions=24  # Depth slices
)

# Phase 2: Light Culling
for each cluster in frustum:
    bounds = calculate_cluster_bounds_3d()
    visible_lights = []
    
    for each light:
        if bounds_intersect_light_3d(bounds, light):
            visible_lights.append(light)
    
    store_light_list(cluster, visible_lights)

# Phase 3: Rendering
for each fragment:
    cluster = get_cluster_for_position_3d(fragment.position, fragment.depth)
    lights = get_light_list(cluster)
    
    # Same as Forward+
    for each light in lights:
        accumulate_lighting(light, fragment)
```

**Cluster structure**:
```
Screen: 1920×1080
Cluster grid: 16×9×24 = 3,456 clusters

Each cluster:
- 120×120 pixels in X-Y
- Exponential depth slicing in Z

cluster_z = pow(depth / far_plane, 1.0 / slice_scaling)
```

**Trade-offs**:

✅ **Strengths**:
- Tighter light culling than Forward+ (3D vs 2D)
- Handles depth variation better
- Even more lights possible (10,000+)
- Good for volumetric effects (fog, particles)
- Scalable across different scene types

❌ **Weaknesses**:
- Most complex implementation
- Higher memory for 3D grid
- Depth-dependent culling complexity
- Requires compute shaders
- Tuning depth slicing is challenging

**When to use**:
- Extreme light counts (thousands)
- Scenes with high depth complexity
- Modern AAA games
- Research/cutting-edge engines

**Real-world examples**:
- Doom Eternal (evolution of Doom 2016)
- Call of Duty: Advanced Warfare
- Unity HDRP (clustered forward/deferred)
- Unreal Engine 5 (for volumetrics)

**Performance characteristics**:
- **Best case**: Deep scenes, many lights → excellent culling
- **Worst case**: Shallow scenes → overhead vs Forward+
- **Typical**: 60 FPS with 1000s of lights at 1080p

### 5. Hybrid Approaches

Many engines combine multiple techniques:

#### Deferred + Forward for Transparency

```
# Phase 1: Deferred for opaque
render_opaque_to_gbuffer()
calculate_deferred_lighting()

# Phase 2: Forward for transparent
depth_test_against_opaque()
render_transparent_forward()
```

**Used in**: Most deferred engines (Unreal, Unity HDRP)

#### Forward+ with Deferred Decals

```
# Render opaque forward+
forward_plus_rendering()

# G-buffer for decals only
render_gbuffer_for_decal_receivers()
apply_decals_deferred()
```

**Used in**: Games with many decals (bullet holes, blood)

#### Multiple Passes for Different Materials

```
# Simple materials: Forward
render_simple_objects_forward()

# Complex materials: Deferred
render_complex_objects_deferred()

# Subsurface scattering: Separate pass
render_skin_subsurface_pass()
```

**Used in**: Character rendering in AAA games

## Comparison Table

| Pattern | Lights | Transparency | Memory | Bandwidth | Complexity | MSAA |
|---------|--------|--------------|--------|-----------|------------|------|
| **Forward** | Few (4-8) | ✅ Easy | ✅ Low | ✅ Low | ⭐ Simple | ✅ Yes |
| **Deferred** | Many (100s) | ❌ Difficult | ❌ High | ❌ High | ⭐⭐ Moderate | ❌ No |
| **Forward+** | Many (1000s) | ✅ Easy | ✅ Low | ✅ Low | ⭐⭐⭐ Complex | ✅ Yes |
| **Clustered** | Very Many (10k+) | ✅ Easy | 🟡 Moderate | ✅ Low | ⭐⭐⭐⭐ Very Complex | ✅ Yes |

## Advanced Rendering Techniques

### GPU-Driven Rendering

Modern pattern: GPU decides what to render, not CPU.

```
# CPU: Upload all geometry once
upload_all_meshes_to_gpu()
upload_instance_data()

# GPU Compute: Cull and build draw commands
compute_shader_frustum_culling()
compute_shader_occlusion_culling()
build_indirect_draw_commands()

# GPU Rendering: Execute commands
execute_indirect_draw_calls()
```

**Benefits**: Eliminates CPU-GPU sync, scales to millions of objects

**Used in**: Unreal Engine 5 Nanite, Unity DOTS, modern AAA engines

### Virtual Shadow Maps

**Problem**: Traditional shadow maps are limited by resolution

**Solution**: Tile-based shadow map atlas, only allocate tiles for visible regions

```
# Allocate shadow map tiles dynamically
for each light:
    visible_tiles = determine_visible_shadow_regions()
    allocate_shadow_tiles(visible_tiles)
    render_shadow_to_tiles()

# When rendering
shadow_coord = calculate_shadow_coordinate()
tile = lookup_shadow_tile(shadow_coord)
shadow_value = sample_shadow_tile(tile, shadow_coord)
```

**Used in**: Unreal Engine 5, modern AAA games

### Screen-Space Effects

Leverage deferred rendering for post-process effects:

- **SSAO** (Screen-Space Ambient Occlusion): Read depth/normal G-buffer
- **SSR** (Screen-Space Reflections): Ray-march depth buffer
- **SSS** (Screen-Space Shadows): Additional shadowing pass
- **SSGI** (Screen-Space Global Illumination): Bounce light calculation

**Trade-off**: Fast but only sees what's on screen (no off-screen data)

## Choosing a Rendering Pattern

### Decision Matrix

**Choose Forward if**:
- ✅ Target mobile/VR
- ✅ Need transparency
- ✅ Limited light count (<10)
- ✅ Need MSAA
- ✅ Low memory bandwidth

**Choose Deferred if**:
- ✅ Many lights (100+)
- ✅ Target PC/console
- ✅ Complex lighting (GI, many point lights)
- ✅ Little transparency
- ✅ Indoor scenes

**Choose Forward+ if**:
- ✅ Many lights AND transparency
- ✅ Modern hardware (compute shaders)
- ✅ Need MSAA with many lights
- ✅ Mixed indoor/outdoor
- ✅ VR with complex lighting

**Choose Clustered if**:
- ✅ Extreme light counts (1000s)
- ✅ Cutting-edge AAA
- ✅ Deep/complex scenes
- ✅ Volumetric effects
- ✅ Have engineering resources

### Hybrid Strategy

Most production engines use combinations:

```
# Example AAA game rendering pipeline
1. Depth pre-pass (early-z)
2. G-buffer pass (opaque objects)
3. Deferred lighting (point/spot lights)
4. Forward pass (transparent objects)
5. Forward pass (emissive objects)
6. Post-processing (SSAO, SSR, bloom, etc.)
7. UI rendering (forward)
```

## Implementation Considerations

### Bandwidth Optimization

**G-buffer packing** (deferred):
```
# Pack normal + roughness
normal_packed.xy = normal.xy;
normal_packed.z = roughness;
normal.z = sqrt(1.0 - dot(normal.xy, normal.xy));  # Reconstruct
```

**Light list compaction** (forward+/clustered):
```
# Only store light indices, not full light data
uint16_t light_indices[MAX_LIGHTS_PER_TILE];
# Fetch full light data when needed
Light light = global_lights[light_indices[i]];
```

### Anti-Aliasing

**Forward**: MSAA works natively (hardware support)

**Deferred**: Use TAA (Temporal Anti-Aliasing) or FXAA
- MSAA too expensive (G-buffer × sample count)
- TAA: Accumulate subpixel jitter across frames
- FXAA: Post-process edge smoothing

**Forward+/Clustered**: MSAA works (still forward rendering)

### Transparency Handling

**Forward/Forward+/Clustered**: Natural depth-sorted rendering

**Deferred**: Multiple strategies:
1. **Forward pass after deferred**: Standard approach
2. **OIT (Order-Independent Transparency)**: Complex, accumulate layers
3. **Weighted Blended OIT**: Approximation, single pass
4. **Depth peeling**: Multiple passes, precise

### Material Variation

**Forward**: Unlimited material complexity (per-object shaders)

**Deferred**: Limited by G-buffer format
- Solution: Uber-shader with material IDs
- Solution: Multiple rendering passes for different material types

**Forward+**: Same as forward (unlimited)

## Common Pitfalls

### Pitfall 1: Not Profiling

**Problem**: Choosing renderer based on assumptions, not measurements

**Solution**: Profile on target hardware
```
Measure:
- GPU time per pass
- Memory bandwidth (use GPU profilers)
- Overdraw (many tools visualize this)
- Light culling overhead
```

### Pitfall 2: Over-Engineering

**Problem**: Implementing clustered rendering for a game with 5 lights

**Solution**: Match complexity to requirements
- Start simple (forward)
- Profile and identify bottlenecks
- Upgrade only if needed

### Pitfall 3: Ignoring Transparency

**Problem**: Choosing deferred, then realizing game has lots of glass/particles

**Solution**: Consider transparency needs upfront
- Count transparent objects in design
- Prototype transparency rendering early
- Choose hybrid approach if needed

### Pitfall 4: G-buffer Format Lock-in

**Problem**: Fixed G-buffer format can't support new material types

**Solution**: Design extensible G-buffer
```
# Reserve bits for material type
material_id = 2 bits (4 material types)
# Each type interprets remaining G-buffer data differently
```

## Further Reading

### Foundational Papers
- **"Deferred Shading in Tabula Rasa"** by Koonce (first major game)
- **"Tiled Shading"** by Olsson et al. (Forward+)
- **"Clustered Deferred and Forward Shading"** by Olsson et al.
- **"Practical Clustered Shading"** by Emil Persson (Avalanche Studios)

### GDC Talks
- **"The Rendering of DOOM 2016"** by Tiago Sousa (Forward+ showcase)
- **"Advances in Real-Time Rendering"** (annual SIGGRAPH course)
- **"FrameGraph: Extensible Rendering Architecture in Frostbite"** (EA)
- **"Rendering the Hellscape of Doom Eternal"** (Clustered evolution)

### Blog Posts
- **"Forward vs Deferred vs Forward+ Rendering"** by Learn OpenGL
- **"Deferred Rendering for Beginners"** by Learn OpenGL
- **"Real-Time Rendering Resources"** by Ke-Sen Huang (comprehensive paper list)

### Engine Documentation
- **Unreal Engine**: Rendering architecture docs
- **Unity HDRP**: Rendering pipeline documentation
- **Godot 4**: Forward+ implementation details
- **AMD GPUOpen**: Rendering technique samples

### Books
- **Real-Time Rendering** (4th ed.) by Akenine-Möller et al. - Comprehensive rendering theory
- **GPU Gems** series - Many rendering techniques
- **Game Engine Architecture** by Gregory - Rendering pipeline chapters

## Summary

Rendering architecture is a fundamental engine choice affecting performance and capabilities:

- **Forward**: Simple, good for mobile/VR, few lights
- **Deferred**: Many lights, complex lighting, high bandwidth
- **Forward+**: Best of both worlds, requires modern hardware
- **Clustered**: Cutting-edge, extreme light counts, very complex

Most modern engines use **hybrid approaches**, combining techniques:
- Deferred for opaque + Forward for transparent (most common)
- Forward+ with selective deferred passes
- GPU-driven culling + clustered lighting

**Choose based on**:
1. Target hardware capabilities
2. Light count requirements
3. Transparency needs
4. Engineering resources
5. Performance requirements

The trend is toward **Forward+/Clustered** on modern hardware, but **Deferred** remains dominant for current-gen games. **Forward** still essential for mobile/VR.

Always profile on target hardware with representative scenes before committing to an architecture!
