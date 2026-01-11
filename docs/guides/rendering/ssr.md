# Screen-Space Reflections (SSR)

Screen-Space Reflections (SSR) provide realistic reflections by ray marching through the depth buffer. Praxis implements hierarchical SSR with roughness-aware blur and environment probe fallback for comprehensive reflection coverage.

## Overview

SSR generates reflections by tracing rays through screen-space depth data. It's efficient because it only considers visible geometry, but requires fallback for off-screen objects.

**Benefits:**
- High-quality reflections on smooth surfaces (water, metal, glass)
- Accurate reflections of visible geometry
- Integrates seamlessly with deferred rendering
- Reasonable performance cost (2-4ms)
- Physically-based roughness support

**Limitations:**
- Cannot reflect off-screen objects (mitigated by environment probe fallback)
- Misses geometry behind the camera
- Artifacts at screen edges (mitigated by edge fade)
- Stretching at grazing angles

## Algorithm

SSR in Praxis consists of three main passes:

### 1. Ray Marching Pass

Traces reflection rays through the depth buffer:

```glsl
// Reconstruct view-space position from depth
vec3 view_pos = reconstruct_position(uv, depth);
vec3 view_dir = normalize(view_pos);

// Get surface normal from G-buffer
vec3 normal = texture(u_gbuffer_normal, uv).xyz;

// Calculate reflection direction
vec3 reflect_dir = reflect(view_dir, normal);

// Ray march through depth buffer
vec2 hit_uv;
float hit_confidence;
bool hit = ray_march(view_pos, reflect_dir, hit_uv, hit_confidence);

if (hit) {
    vec3 reflection = texture(u_scene_color, hit_uv).rgb;
    o_color = vec4(reflection, hit_confidence);
} else {
    o_color = vec4(0.0, 0.0, 0.0, 0.0); // Mark for probe fallback
}
```

### 2. Roughness Blur Pass

Applies variable blur based on surface roughness:

```glsl
float roughness = texture(u_gbuffer_roughness, uv).g;

// Roughness determines blur kernel size
float blur_radius = roughness * max_blur_radius;

vec3 blurred_reflection = vec3(0.0);
float total_weight = 0.0;

for (int i = 0; i < sample_count; i++) {
    vec2 offset = poisson_disk[i] * blur_radius * texel_size;
    vec3 sample_color = texture(u_ssr_reflection, uv + offset).rgb;
    float weight = 1.0; // Could use depth-aware weighting
    
    blurred_reflection += sample_color * weight;
    total_weight += weight;
}

blurred_reflection /= total_weight;
```

### 3. Composite Pass

Blends SSR with environment probe fallback:

```glsl
vec4 ssr_data = texture(u_ssr_blurred, uv);
float hit_confidence = ssr_data.a;

if (hit_confidence > min_confidence) {
    // Use SSR
    vec3 reflection = ssr_data.rgb;
    
    // Edge fade
    vec2 edge_factor = smoothstep(0.0, edge_fade, uv) * 
                       smoothstep(0.0, edge_fade, 1.0 - uv);
    float edge_weight = edge_factor.x * edge_factor.y;
    
    reflection_color = mix(probe_reflection, reflection, edge_weight);
} else {
    // Fallback to environment probe
    vec3 reflect_world = mat3(u_inv_view) * reflect_dir;
    reflection_color = texture(u_environment_probe, reflect_world).rgb;
}
```

## Hierarchical Ray Marching

Uses depth buffer mipmap pyramid for efficient ray tracing:

```glsl
vec3 ray_march_hierarchical(vec3 ray_start, vec3 ray_dir) {
    float step_size = initial_step_size;
    vec3 current_pos = ray_start;
    int mip_level = 0;
    
    for (int i = 0; i < max_steps; i++) {
        // Advance ray
        current_pos += ray_dir * step_size;
        
        // Project to screen space
        vec4 proj_pos = projection * vec4(current_pos, 1.0);
        vec2 screen_uv = proj_pos.xy / proj_pos.w * 0.5 + 0.5;
        
        // Sample depth at current mip level
        float depth = textureLod(u_depth, screen_uv, mip_level).r;
        float ray_depth = proj_pos.z / proj_pos.w;
        
        // Check intersection
        if (ray_depth > depth && ray_depth - depth < thickness) {
            // Hit! Refine with binary search
            return binary_search_refinement(current_pos, ray_dir, step_size);
        }
        
        // Adaptive mip level based on distance
        float distance_traveled = length(current_pos - ray_start);
        mip_level = int(log2(distance_traveled / min_step_distance));
        mip_level = clamp(mip_level, 0, max_mip_level);
        
        // Adjust step size based on mip level
        step_size = initial_step_size * pow(2.0, float(mip_level));
    }
    
    return vec3(-1.0); // Miss
}

vec3 binary_search_refinement(vec3 pos, vec3 dir, float step) {
    // Binary search to find precise intersection
    for (int i = 0; i < max_binary_steps; i++) {
        step *= 0.5;
        
        vec4 proj = projection * vec4(pos, 1.0);
        vec2 uv = proj.xy / proj.w * 0.5 + 0.5;
        float depth = texture(u_depth, uv).r;
        float ray_depth = proj.z / proj.w;
        
        if (ray_depth > depth) {
            pos -= dir * step;
        } else {
            pos += dir * step;
        }
    }
    
    return pos;
}
```

## Usage

### Basic Setup

```rust
use praxis_graphics::ssr::{SsrRenderer, SsrConfig};

let config = SsrConfig {
    max_steps: 64,                    // Ray marching iterations
    max_binary_search_steps: 8,      // Refinement iterations
    step_size: 1.0,                  // Ray step multiplier
    thickness: 0.1,                  // Surface thickness for hits
    max_roughness: 0.8,              // Max roughness to compute SSR
    min_hit_confidence: 0.5,         // Min confidence to use SSR
    edge_fade_factor: 0.1,           // Fade out near screen edges
    blur_passes: 2,                  // Number of blur passes
};

let mut ssr = SsrRenderer::new(
    device.clone(),
    memory_allocator.clone(),
    1920, // width
    1080, // height
    config
)?;
```

### Per-Frame Rendering

```rust
// After G-buffer and lighting passes
let reflection_texture = ssr.render(
    &mut builder,
    &gbuffer,                        // G-buffer with normals, depth, roughness
    scene_color.clone(),             // Lit scene color
    camera.projection_matrix(),
    camera.view_matrix(),
    camera.position(),
    Some(&ibl_data)                  // Optional: environment probe fallback
)?;

// Use reflection_texture in final composite
```

### Configuration Tuning

```rust
let config = SsrConfig::default()
    .with_max_steps(128)             // Higher quality, slower (64-128 typical)
    .with_max_binary_search_steps(8) // Precision (4-12 typical)
    .with_thickness(0.15)            // Thicker surfaces (0.05-0.2)
    .with_max_roughness(0.7)         // Skip rough surfaces (0.6-0.9)
    .with_min_hit_confidence(0.6)    // Stricter quality filter
    .with_edge_fade_factor(0.15)     // Wider edge fade
    .with_blur_passes(3);            // More blur iterations
```

**Performance vs. Quality Guidelines:**

| Setting | Low (Fast) | Medium | High (Quality) | Ultra |
|---------|-----------|--------|---------------|-------|
| `max_steps` | 32 | 64 | 96 | 128 |
| `max_binary_search_steps` | 4 | 6 | 8 | 12 |
| `step_size` | 1.5 | 1.0 | 0.8 | 0.5 |
| `blur_passes` | 1 | 2 | 3 | 4 |
| **Cost (1080p)** | ~1.5ms | ~2.5ms | ~4ms | ~6ms |

## Integration Patterns

### With Deferred Rendering

```rust
// 1. G-buffer pass
deferred_renderer.geometry_pass(
    builder,
    &gbuffer,
    objects,
    view_proj
)?;

// 2. Lighting pass
let lit_scene = deferred_renderer.lighting_pass(
    builder,
    &gbuffer,
    lights
)?;

// 3. SSR pass
let reflections = ssr.render(
    builder,
    &gbuffer,
    lit_scene.clone(),
    projection,
    view,
    camera_pos,
    Some(&ibl)
)?;

// 4. Composite reflections
composite_pass(builder, lit_scene, reflections, &gbuffer)?;
```

### With Forward Rendering

For forward rendering, you need to generate a simplified G-buffer:

```rust
// 1. Pre-pass to generate depth/normals
forward_renderer.depth_prepass(
    builder,
    &depth_normal_target,
    objects,
    view_proj
)?;

// 2. Main rendering
let scene_color = forward_renderer.render(
    builder,
    objects,
    lights,
    view_proj
)?;

// 3. Create pseudo-G-buffer for SSR
let pseudo_gbuffer = GBuffer {
    normal: depth_normal_target.normal_view,
    depth: depth_normal_target.depth_view,
    metallic_roughness: default_material_buffer, // From material pass
    albedo: scene_color.clone(),
};

// 4. SSR
let reflections = ssr.render(
    builder,
    &pseudo_gbuffer,
    scene_color,
    projection,
    view,
    camera_pos,
    None // No probe fallback in this example
)?;
```

### Selective SSR (Material-Based)

Only compute SSR for reflective materials:

```rust
// Mark reflective surfaces in G-buffer
if material.metallic > 0.9 && material.roughness < 0.3 {
    gbuffer.metallic_roughness.a = 1.0; // SSR enable flag
}

// In SSR shader
float ssr_enabled = texture(u_gbuffer_material, uv).a;
if (ssr_enabled < 0.5) {
    // Skip SSR for this pixel
    discard;
}
```

## Performance Optimization

### 1. Half-Resolution SSR

Render SSR at half resolution for 4x performance improvement:

```rust
let ssr = SsrRenderer::new(
    device.clone(),
    memory_allocator.clone(),
    width / 2,  // Half resolution
    height / 2,
    config
)?;
```

Upsample with bilateral filter:

```glsl
vec3 upsample_bilateral(sampler2D half_res, vec2 uv, float depth) {
    vec3 result = vec3(0.0);
    float total_weight = 0.0;
    
    for (int y = -1; y <= 1; y++) {
        for (int x = -1; x <= 1; x++) {
            vec2 offset = vec2(x, y) * half_res_texel_size;
            vec3 sample_color = texture(half_res, uv + offset).rgb;
            float sample_depth = texture(half_res_depth, uv + offset).r;
            
            // Depth-aware weight
            float depth_diff = abs(depth - sample_depth);
            float weight = exp(-depth_diff * 100.0);
            
            result += sample_color * weight;
            total_weight += weight;
        }
    }
    
    return result / total_weight;
}
```

### 2. Stencil Masking

Only compute SSR for reflective pixels:

```rust
// During G-buffer pass, mark reflective surfaces in stencil
if material.is_reflective() {
    stencil_value = 1;
}

// SSR pass uses stencil test
render_pass_begin_info.stencil_ops = StencilOps {
    test: CompareOp::Equal,
    reference: 1,
    ..default()
};
```

### 3. Adaptive Ray Marching

Adjust step count based on surface properties:

```glsl
float roughness = texture(u_roughness, uv).r;
int adaptive_steps = int(mix(float(max_steps), float(min_steps), roughness));
```

### 4. Temporal Stability

Accumulate SSR across frames with TAA:

```rust
// After SSR
taa.apply(
    builder,
    &taa_target,
    ssr_reflection.clone(),
    velocity_buffer.clone(),
    depth_buffer.clone(),
    TaaConfig { blend_factor: 0.15, ..default() }
)?;
```

## Troubleshooting

### Ray Marching Artifacts

**Symptom**: Banding, gaps, or incorrect hits.

**Solutions:**
- Increase `max_steps` (64 → 96)
- Decrease `step_size` (1.0 → 0.8)
- Tune `thickness` based on scene scale
- Enable binary search refinement

### Screen-Space Leaking

**Symptom**: Reflections "leak" behind surfaces.

**Solutions:**
- Adjust `thickness` parameter (typically 0.05-0.2)
- Improve depth precision
- Add depth fade near depth discontinuities:
  ```glsl
  float depth_gradient = length(fwidth(depth));
  hit_confidence *= 1.0 - saturate(depth_gradient * 10.0);
  ```

### Edge Artifacts

**Symptom**: Harsh cutoffs at screen borders.

**Solutions:**
- Increase `edge_fade_factor` (0.1 → 0.2)
- Clamp reflection UVs before edge fade
- Blend with environment probe more aggressively near edges

### Stretching at Grazing Angles

**Symptom**: Reflections stretch/smear at low view angles.

**Solutions:**
- Fade out SSR at grazing angles:
  ```glsl
  float NdotV = dot(normal, -view_dir);
  hit_confidence *= saturate(NdotV * 4.0);
  ```
- Increase blend to environment probe at grazing angles

### Performance Issues

**Symptom**: SSR takes too much frame time.

**Solutions:**
- Reduce `max_steps` (128 → 64)
- Use half-resolution rendering
- Implement stencil masking for reflective surfaces only
- Skip SSR for rough surfaces (`max_roughness: 0.6`)

## Advanced Techniques

### Contact Hardening

Sharpen reflections near contact points:

```glsl
float contact_distance = distance_to_contact(hit_pos);
float blur_size = mix(min_blur, max_blur, saturate(contact_distance / max_distance));
```

### Depth Rejection

Reject hits behind surfaces:

```glsl
float hit_depth = texture(u_depth, hit_uv).r;
float surface_depth = linearize_depth(depth);
float hit_surface_depth = linearize_depth(hit_depth);

if (hit_surface_depth < surface_depth - thickness) {
    discard; // Hit is behind surface
}
```

### Importance Sampling for Roughness

For rough surfaces, importance-sample the BRDF:

```glsl
vec3 sample_ggx(vec3 N, float roughness, vec2 random) {
    float a = roughness * roughness;
    float phi = 2.0 * PI * random.x;
    float cos_theta = sqrt((1.0 - random.y) / (1.0 + (a * a - 1.0) * random.y));
    float sin_theta = sqrt(1.0 - cos_theta * cos_theta);
    
    vec3 H = vec3(cos(phi) * sin_theta, sin(phi) * sin_theta, cos_theta);
    // Transform H to world space and reflect
    return reflect(view_dir, N, H);
}

// Multi-sample rough reflections
vec3 rough_reflection = vec3(0.0);
for (int i = 0; i < rough_sample_count; i++) {
    vec2 xi = hammersley(i, rough_sample_count);
    vec3 sample_dir = sample_ggx(normal, roughness, xi);
    rough_reflection += ray_march(view_pos, sample_dir);
}
rough_reflection /= float(rough_sample_count);
```

### Planar Reflections Fallback

For important reflective surfaces (water, mirrors), combine SSR with planar reflections:

```rust
if is_water_surface {
    let planar = render_planar_reflection(water_plane);
    let ssr = render_ssr(gbuffer);
    
    // Blend based on distance from plane
    let blend_factor = compute_plane_distance_fade(pixel_pos, water_plane);
    final_reflection = mix(ssr, planar, blend_factor);
}
```

## Comparison with Other Reflection Techniques

| Technique | Quality | Performance | Limitations |
|-----------|---------|-------------|------------|
| **Cubemaps** | Low-Medium | Fast (0.1ms) | Static only, no local reflections |
| **Planar Reflections** | High | Expensive (re-render scene) | Only for flat surfaces |
| **SSR** | High | Moderate (2-4ms) | Screen-space only, edge artifacts |
| **Ray-traced Reflections** | Highest | Very Expensive (5-15ms) | Requires RT hardware |
| **SSR + Probe Fallback** | High | Moderate (2-4ms) | Best practical solution |

**Recommendation**: Use SSR with environment probe fallback for best quality/performance balance in games.

## Example

```rust
use praxis_graphics::ssr::{SsrRenderer, SsrConfig};
use praxis_graphics::deferred::DeferredRenderer;

struct ReflectiveRenderer {
    deferred: DeferredRenderer,
    ssr: SsrRenderer,
    ssr_enabled: bool,
}

impl ReflectiveRenderer {
    fn render(&mut self, scene: &Scene, camera: &Camera) -> Result<Arc<ImageView>> {
        // Geometry pass
        self.deferred.geometry_pass(&mut self.builder, &self.gbuffer, scene)?;
        
        // Lighting pass
        let lit_scene = self.deferred.lighting_pass(
            &mut self.builder,
            &self.gbuffer,
            &scene.lights
        )?;
        
        if self.ssr_enabled {
            // SSR pass
            let reflections = self.ssr.render(
                &mut self.builder,
                &self.gbuffer,
                lit_scene.clone(),
                camera.projection_matrix(),
                camera.view_matrix(),
                camera.position(),
                Some(&self.environment_probe)
            )?;
            
            // Composite
            self.composite_with_reflections(lit_scene, reflections)
        } else {
            Ok(lit_scene)
        }
    }
}
```

## See Also

- [Deferred Rendering](deferred-rendering.md) - G-buffer generation
- [Environment Probes](environment-probes.md) - Fallback reflections
- [HDR and Tone Mapping](hdr-tonemapping.md) - HDR reflection handling
- [Temporal Anti-Aliasing](taa.md) - Stabilizing SSR with TAA
- [Advanced Materials](../advanced-materials.md) - Reflective material setup
