// The buffer view pass's fragment entry points, one per buffer
// (src/render/buffer_view_pass.rs).

// Which one a view shows of a sample's two normals, as stored, of the depth
// pyramid's levels, or of the Shadow mask's bits.
override BUFFER_VIEW_INDEX: u32 = 0u;

const HALF_GREY_VIEW_DEPTH: f32 = 16.0;
const HOTTEST_POINT_LIGHTS: f32 = 8.0;

// Reversed depth is the near plane's distance over the view depth
// [ref:glam-0.34.3-perspective-infinite-reverse] (ADR 0015), so this is
// 1 / (1 + view depth / HALF_GREY_VIEW_DEPTH), and 0 where the frame drew
// nothing.
fn depth_grey(reversed_depth: f32) -> vec4<f32> {
    let near = camera.view_proj[3].z;
    let grey = reversed_depth / (reversed_depth + near / HALF_GREY_VIEW_DEPTH);
    return vec4<f32>(vec3<f32>(grey), 1.0);
}

struct NearestSample {
    index: u32,
    depth: f32,
}

// The nearest sample has the greatest depth (ADR 0015).
fn nearest_sample(pixel: vec2<u32>) -> NearestSample {
    var nearest = NearestSample(0u, textureLoad(depth, pixel, 0u));
    for (var sample = 1u; sample < SAMPLE_COUNT; sample++) {
        let sample_depth = textureLoad(depth, pixel, sample);
        if sample_depth > nearest.depth {
            nearest = NearestSample(sample, sample_depth);
        }
    }
    return nearest;
}

@fragment
fn fs_depth_view(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return depth_grey(nearest_sample(vec2<u32>(position.xy)).depth);
}

@fragment
fn fs_surface_normals_view(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<u32>(position.xy);
    let nearest = nearest_sample(pixel);
    let encoded = textureLoad(surface_normals, pixel, nearest.index) * 2.0 - 1.0;
    let normal = octahedral_decode(select(encoded.xy, encoded.zw, BUFFER_VIEW_INDEX == 1u));
    return vec4<f32>(select(vec3<f32>(0.0), normal * 0.5 + 0.5, nearest.depth > 0.0), 1.0);
}

@fragment
fn fs_shadow_mask_view(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let word = textureLoad(shadow_mask, vec2<u32>(position.xy), 0u, 0).rg;
    let half = select(word.x, word.y, BUFFER_VIEW_INDEX >= 32u);
    let hidden = BUFFER_VIEW_INDEX < SHADOW_MASK_LIGHTS
        && ((half >> (BUFFER_VIEW_INDEX & 31u)) & 1u) == 1u;
    return vec4<f32>(vec3<f32>(select(192.0, 64.0, hidden) / 255.0), 1.0);
}

@fragment
fn fs_hdr_view(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let weighted = textureLoad(hdr_target, vec2<u32>(position.xy), 0).rgb;
    return vec4<f32>(srgb_encode(resolve_unweighted(weighted)), 1.0);
}

@fragment
fn fs_depth_pyramid_view(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let level = min(BUFFER_VIEW_INDEX, textureNumLevels(depth_pyramid) - 1u);
    let level_size = vec2<f32>(textureDimensions(depth_pyramid, level));
    let texel = vec2<u32>(position.xy / vec2<f32>(textureDimensions(depth)) * level_size);
    return depth_grey(textureLoad(depth_pyramid, texel, level).r);
}

// The Froxel of the pixel's first sample, as the light count pass finds it
// (light_count.wgsl).
@fragment
fn fs_point_lights_view(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<u32>(position.xy);
    let first_depth = textureLoad(depth, pixel, 0u);
    if first_depth == 0.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    let view_depth = camera.view_proj[3].z / first_depth;
    let size = vec2<f32>(textureDimensions(depth));
    let froxel = froxel_at(vec2<f32>(pixel) + SAMPLE_OFFSETS[0], size, view_depth);
    let heat = 3.0 * f32(froxel_lists[froxel].light_count) / HOTTEST_POINT_LIGHTS;
    return vec4<f32>(saturate(vec3<f32>(heat, heat - 1.0, heat - 2.0)), 1.0);
}
