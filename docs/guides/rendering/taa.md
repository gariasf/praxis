# Temporal Anti-Aliasing (TAA)

Temporal Anti-Aliasing (TAA) is a screen-space technique that reduces aliasing artifacts by accumulating samples across multiple frames. Praxis provides a complete TAA implementation with velocity buffers, neighborhood clamping, and Halton sequence jittering.

## Overview

TAA works by blending the current frame with previous frame(s) using motion vectors to reproject history samples. This temporal accumulation effectively increases sample density without the performance cost of traditional MSAA.

**Benefits:**
- Dramatically reduces temporal aliasing (shimmering edges, specular flickering)
- Lower performance cost than MSAA (2-4x faster)
- Produces temporally stable image
- Works well with deferred rendering
- Enables sub-pixel detail recovery with camera jitter

**Trade-offs:**
- Can introduce ghosting on fast-moving objects (mitigated by neighborhood clamping)
- Requires velocity buffer generation
- Needs history buffer storage (~8MB at 1080p)
- Adds 1-2ms frame latency

## Algorithm

TAA consists of four key steps:

### 1. Camera Jitter

Apply sub-pixel offsets to the projection matrix for better sample coverage:

```rust
use praxis_graphics::taa::{HaltonSequence, apply_jitter_to_projection};

let mut halton = HaltonSequence::new();

// In render loop
let jitter = halton.next_jitter();
let jittered_projection = apply_jitter_to_projection(
    projection_matrix,
    jitter,
    1920, // width
    1080  // height
);

// Use jittered_projection for rendering
```

**Halton Sequence**: Low-discrepancy sequence providing optimal sub-pixel coverage across 16 frames.

### 2. Velocity Buffer Generation

Generate per-pixel motion vectors during the geometry pass:

```glsl
// Vertex shader
layout(location = 0) in vec3 a_position;

layout(set = 0, binding = 0) uniform Matrices {
    mat4 current_view_proj;
    mat4 previous_view_proj;
};

out vec4 v_current_pos;
out vec4 v_previous_pos;

void main() {
    v_current_pos = current_view_proj * vec4(a_position, 1.0);
    v_previous_pos = previous_view_proj * vec4(a_position, 1.0);
    gl_Position = v_current_pos;
}
```

```glsl
// Fragment shader
in vec4 v_current_pos;
in vec4 v_previous_pos;

layout(location = 1) out vec2 o_velocity;

void main() {
    // Convert to NDC
    vec2 current_ndc = v_current_pos.xy / v_current_pos.w;
    vec2 previous_ndc = v_previous_pos.xy / v_previous_pos.w;
    
    // Calculate screen-space velocity
    o_velocity = (current_ndc - previous_ndc) * 0.5;
}
```

### 3. Temporal Reprojection

Sample the history buffer using velocity vectors:

```rust
use praxis_graphics::taa::{TaaRenderer, TaaConfig, TaaRenderTarget};

let mut taa = TaaRenderer::new(device, memory_allocator)?;

let mut taa_target = taa.create_render_target(1920, 1080)?;

let config = TaaConfig {
    jitter_offset: halton.next_jitter(),
    blend_factor: 0.1, // 90% history, 10% current
};
```

Shader performs reprojection:

```glsl
// Sample history at reprojected location
vec2 uv = v_uv;
vec2 velocity = texture(u_velocity, uv).xy;
vec2 history_uv = uv - velocity;

// Clamp to valid UV range
history_uv = clamp(history_uv, 0.0, 1.0);

vec3 history_color = texture(u_history, history_uv).rgb;
vec3 current_color = texture(u_current, uv).rgb;
```

### 4. Neighborhood Clamping

Reject disocclusions and prevent ghosting using neighborhood color clamping:

```glsl
// Sample 3x3 neighborhood around current pixel
vec3 color_min = vec3(999999.0);
vec3 color_max = vec3(-999999.0);
vec3 color_avg = vec3(0.0);

const int RADIUS = 1;
for (int y = -RADIUS; y <= RADIUS; y++) {
    for (int x = -RADIUS; x <= RADIUS; x++) {
        vec2 offset = vec2(x, y) * texel_size;
        vec3 neighbor = texture(u_current, uv + offset).rgb;
        
        color_min = min(color_min, neighbor);
        color_max = max(color_max, neighbor);
        color_avg += neighbor;
    }
}
color_avg /= 9.0;

// Clamp history to neighborhood bounds
history_color = clamp(history_color, color_min, color_max);
```

**Variance Clipping (Advanced)**: Use neighborhood variance instead of min/max for better quality:

```glsl
vec3 m1 = color_avg;
vec3 m2 = vec3(0.0);

for (int y = -RADIUS; y <= RADIUS; y++) {
    for (int x = -RADIUS; x <= RADIUS; x++) {
        vec2 offset = vec2(x, y) * texel_size;
        vec3 c = texture(u_current, uv + offset).rgb;
        m2 += c * c;
    }
}
m2 /= 9.0;

vec3 sigma = sqrt(max(vec3(0.0), m2 - m1 * m1));
vec3 box_min = m1 - sigma * 1.5;
vec3 box_max = m1 + sigma * 1.5;

history_color = clamp(history_color, box_min, box_max);
```

### 5. Temporal Blending

Blend current and clamped history:

```glsl
vec3 final_color = mix(history_color, current_color, blend_factor);
```

**Adaptive Blending**: Increase blend factor in high-velocity areas:

```glsl
float velocity_length = length(velocity);
float adaptive_blend = mix(0.05, 0.3, saturate(velocity_length * 10.0));
vec3 final_color = mix(history_color, current_color, adaptive_blend);
```

## Usage

### Basic Setup

```rust
use praxis_graphics::taa::{TaaRenderer, TaaConfig, HaltonSequence, apply_jitter_to_projection};
use std::sync::Arc;

// Create TAA renderer
let taa = TaaRenderer::new(
    device.clone(),
    memory_allocator.clone()
)?;

// Create render target
let mut taa_target = taa.create_render_target(1920, 1080)?;

// Create jitter generator
let mut halton = HaltonSequence::new();
```

### Per-Frame Rendering

```rust
// Generate jitter for this frame
let jitter = halton.next_jitter();

// Apply jitter to projection matrix
let jittered_proj = apply_jitter_to_projection(
    camera.projection_matrix(),
    jitter,
    width,
    height
);

// Render scene with jittered projection
render_scene_with_velocity(
    &mut builder,
    &scene,
    camera.view_matrix(),
    jittered_proj,
    previous_view_proj // For velocity generation
)?;

// Apply TAA
let config = TaaConfig {
    jitter_offset: jitter,
    blend_factor: 0.1,
};

taa.apply(
    &mut builder,
    &taa_target,
    current_frame_view,
    velocity_buffer_view,
    depth_buffer_view,
    config
)?;

// Swap history buffers
taa_target.swap_buffers();

// Store matrices for next frame
previous_view_proj = camera.view_matrix() * jittered_proj;
```

### Configuration

```rust
let config = TaaConfig {
    jitter_offset: [0.0, 0.0], // Will be set per-frame by Halton sequence
    blend_factor: 0.1,         // Lower = more history (smoother but more ghosting)
};
```

**Tuning Guidelines:**
- `blend_factor: 0.05-0.1`: Smooth, good for static/slow-moving scenes
- `blend_factor: 0.1-0.2`: Balanced, good general purpose
- `blend_factor: 0.2-0.3`: Responsive, good for fast-paced games
- `blend_factor: 0.3+`: Minimal ghosting but less effective anti-aliasing

## Integration with Rendering Pipelines

### Forward Rendering

```rust
// 1. Render scene to HDR target with velocity
forward_renderer.render_with_velocity(
    builder,
    scene_target,
    velocity_target,
    objects,
    jittered_view_proj,
    previous_view_proj
)?;

// 2. Apply TAA
taa.apply(
    builder,
    &taa_target,
    scene_target.color_view.clone(),
    velocity_target.color_view.clone(),
    scene_target.depth_view.clone(),
    config
)?;

// 3. Use TAA output for post-processing or display
let anti_aliased = taa_target.color_view.clone();
```

### Deferred Rendering

```rust
// 1. Geometry pass with velocity
deferred_renderer.geometry_pass_with_velocity(
    builder,
    gbuffer,
    velocity_buffer,
    objects,
    jittered_view_proj,
    previous_view_proj
)?;

// 2. Lighting pass
deferred_renderer.lighting_pass(
    builder,
    lit_scene_target,
    gbuffer,
    lights
)?;

// 3. Apply TAA
taa.apply(
    builder,
    &taa_target,
    lit_scene_target.color_view.clone(),
    velocity_buffer.color_view.clone(),
    gbuffer.depth.clone(),
    config
)?;
```

## Performance Considerations

### Cost Analysis

At 1080p:
- **Velocity generation**: ~0.3ms (integrated into geometry pass)
- **TAA resolve**: ~0.8-1.2ms (full-screen pass)
- **Memory**: ~24MB (history + velocity buffers)

Compare to MSAA 4x:
- **Cost**: 3-4ms (4x geometry + resolve)
- **Memory**: ~33MB (4x render targets)

**TAA is 2-4x faster than MSAA 4x.**

### Optimization Tips

1. **Quarter-resolution velocity**: For performance-critical scenarios, generate velocity at half or quarter resolution:
   ```rust
   let velocity_target = create_velocity_target(width / 2, height / 2);
   ```

2. **Skip static objects**: Track which objects moved and only generate velocity for dynamic geometry.

3. **Shared velocity generation**: Compute velocity in the geometry pass to avoid extra rendering.

4. **Mipmap history buffer**: Use mipmaps on history for faster sampling:
   ```rust
   let history_image = Image::new(
       memory_allocator,
       ImageCreateInfo {
           mip_levels: calculate_mip_levels(width, height),
           ..image_create_info
       },
       allocation_info
   )?;
   ```

## Troubleshooting

### Ghosting on Fast Motion

**Symptom**: Visible trails behind moving objects.

**Solutions:**
- Increase `blend_factor` (0.15-0.25)
- Implement adaptive blending based on velocity
- Add edge detection to boost current frame contribution at object boundaries

### Shimmering on Static Geometry

**Symptom**: Edges still shimmer despite TAA.

**Solutions:**
- Decrease `blend_factor` (0.05-0.08)
- Verify jitter pattern is correct (should cover 4x4 grid over 16 frames)
- Check that velocity is correctly zero for static objects

### Smearing/Blur

**Symptom**: Image appears soft or smeared.

**Solutions:**
- Reduce neighborhood clamping radius
- Use variance-based clamping instead of min/max
- Implement sharpening pass after TAA:
  ```glsl
  vec3 sharpen = color + (color - avg_neighborhood) * sharpness;
  ```

### Incorrect Velocity

**Symptom**: Warping, disocclusions, or incorrect reprojection.

**Solutions:**
- Ensure previous frame matrices are correctly stored
- Verify velocity is in screen-space [-1, 1] range
- Check that dynamic objects update their previous transforms

## Advanced Techniques

### Responsive TAA

Increase responsiveness for fast camera movement:

```rust
let camera_velocity = (current_position - previous_position).length();
let responsive_blend = mix(0.1, 0.3, saturate(camera_velocity * 0.1));

let config = TaaConfig {
    jitter_offset: jitter,
    blend_factor: responsive_blend,
};
```

### Multi-Frame Accumulation

For offline rendering or cutscenes, accumulate more frames:

```rust
let accumulation_weight = 1.0 / frame_count as f32;
let config = TaaConfig {
    jitter_offset: jitter,
    blend_factor: accumulation_weight, // Decreases over time
};
```

### YCoCg Color Space

Perform blending in YCoCg for better perceptual results:

```glsl
vec3 rgb_to_ycocg(vec3 rgb) {
    return vec3(
        0.25 * rgb.r + 0.5 * rgb.g + 0.25 * rgb.b,
        0.5 * rgb.r - 0.5 * rgb.b,
        -0.25 * rgb.r + 0.5 * rgb.g - 0.25 * rgb.b
    );
}

vec3 ycocg_to_rgb(vec3 ycocg) {
    return vec3(
        ycocg.x + ycocg.y - ycocg.z,
        ycocg.x + ycocg.z,
        ycocg.x - ycocg.y - ycocg.z
    );
}

// Convert to YCoCg, perform TAA, convert back
vec3 current_ycocg = rgb_to_ycocg(current_color);
vec3 history_ycocg = rgb_to_ycocg(history_color);
vec3 result_ycocg = mix(history_ycocg, current_ycocg, blend_factor);
vec3 result = ycocg_to_rgb(result_ycocg);
```

## Comparison with Other AA Techniques

| Technique | Quality | Performance | Notes |
|-----------|---------|-------------|-------|
| **No AA** | Poor | Free | Severe aliasing |
| **FXAA** | Low-Medium | 0.3-0.5ms | Post-process only, blurry |
| **SMAA** | Medium | 0.8-1.2ms | Better than FXAA, still some aliasing |
| **MSAA 4x** | Medium-High | 3-5ms | Expensive, doesn't work with deferred |
| **TAA** | High | 1-2ms | Best quality/performance, minor ghosting |
| **DLSS/FSR** | High | 0.5-1.5ms | Requires vendor support, includes upscaling |

**Recommendation**: Use TAA for most scenarios. It provides excellent quality at reasonable cost and works with both forward and deferred rendering.

## Example

```rust
use praxis_graphics::taa::{TaaRenderer, TaaConfig, HaltonSequence, apply_jitter_to_projection};
use praxis_math::Mat4;

struct Renderer {
    taa: TaaRenderer,
    taa_target: TaaRenderTarget,
    halton: HaltonSequence,
    previous_view_proj: Mat4,
}

impl Renderer {
    fn render_frame(&mut self, camera: &Camera, scene: &Scene) -> Result<()> {
        // Generate jitter
        let jitter = self.halton.next_jitter();
        
        // Apply to projection
        let proj = camera.projection_matrix();
        let jittered_proj = apply_jitter_to_projection(proj, jitter, 1920, 1080);
        let view_proj = camera.view_matrix() * jittered_proj;
        
        // Render with velocity
        self.render_scene_with_velocity(scene, view_proj, self.previous_view_proj)?;
        
        // Apply TAA
        let config = TaaConfig {
            jitter_offset: jitter,
            blend_factor: 0.1,
        };
        
        self.taa.apply(
            &mut self.command_buffer,
            &self.taa_target,
            self.scene_color_view.clone(),
            self.velocity_view.clone(),
            self.depth_view.clone(),
            config
        )?;
        
        // Swap buffers
        self.taa_target.swap_buffers();
        
        // Store for next frame
        self.previous_view_proj = view_proj;
        
        Ok(())
    }
}
```

## See Also

- [Post-Processing](post-processing.md) - Other post-processing effects
- [Deferred Rendering](deferred-rendering.md) - Integration with G-buffer
- [HDR and Tone Mapping](hdr-tonemapping.md) - HDR pipeline integration
- [Screen-Space Reflections](ssr.md) - Another temporal technique
