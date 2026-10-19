// Throwaway probe for #189 (never merged): a mip-chain bloom after Jimenez
// 2014 (SIGGRAPH Advances, "Next Generation Post Processing in Call of Duty:
// Advanced Warfare"): a 13-tap downsample, its first level weighted per 2x2
// box by 1 / (1 + luminance) against fireflies, and a 3x3 tent upsample added
// into each larger level.

override KARIS_AVERAGE: bool = false;

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var linear_clamp: sampler;

const REC709_LUMINANCE = vec3<f32>(0.2126, 0.7152, 0.0722);

@vertex
fn vs_fullscreen(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

fn tap(center: vec2<f32>, texel: vec2<f32>, x: f32, y: f32) -> vec3<f32> {
    return textureSampleLevel(source_texture, linear_clamp, center + vec2<f32>(x, y) * texel, 0.0).rgb;
}

fn box_weight(color: vec3<f32>) -> f32 {
    if KARIS_AVERAGE {
        return 1.0 / (1.0 + dot(color, REC709_LUMINANCE));
    }
    return 1.0;
}

@fragment
fn fs_downsample(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let source_size = vec2<f32>(textureDimensions(source_texture));
    let texel = 1.0 / source_size;
    let center = position.xy * 2.0 * texel;
    let a = tap(center, texel, -2.0, -2.0);
    let b = tap(center, texel, 0.0, -2.0);
    let c = tap(center, texel, 2.0, -2.0);
    let d = tap(center, texel, -1.0, -1.0);
    let e = tap(center, texel, 1.0, -1.0);
    let f = tap(center, texel, -2.0, 0.0);
    let g = tap(center, texel, 0.0, 0.0);
    let h = tap(center, texel, 2.0, 0.0);
    let i = tap(center, texel, -1.0, 1.0);
    let j = tap(center, texel, 1.0, 1.0);
    let k = tap(center, texel, -2.0, 2.0);
    let l = tap(center, texel, 0.0, 2.0);
    let m = tap(center, texel, 2.0, 2.0);
    let inner = (d + e + i + j) * 0.25;
    let top_left = (a + b + f + g) * 0.25;
    let top_right = (b + c + g + h) * 0.25;
    let bottom_left = (f + g + k + l) * 0.25;
    let bottom_right = (g + h + l + m) * 0.25;
    let inner_weight = 0.5 * box_weight(inner);
    let top_left_weight = 0.125 * box_weight(top_left);
    let top_right_weight = 0.125 * box_weight(top_right);
    let bottom_left_weight = 0.125 * box_weight(bottom_left);
    let bottom_right_weight = 0.125 * box_weight(bottom_right);
    let sum = inner * inner_weight + top_left * top_left_weight + top_right * top_right_weight
        + bottom_left * bottom_left_weight + bottom_right * bottom_right_weight;
    let weight = inner_weight + top_left_weight + top_right_weight + bottom_left_weight + bottom_right_weight;
    return vec4<f32>(sum / weight, 1.0);
}

@fragment
fn fs_upsample(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    // The source is the smaller level; the target pixel centre maps into it.
    let source_size = vec2<f32>(textureDimensions(source_texture));
    let texel = 1.0 / source_size;
    let center = position.xy * 0.5 * texel;
    let sum = tap(center, texel, -1.0, -1.0) + 2.0 * tap(center, texel, 0.0, -1.0) + tap(center, texel, 1.0, -1.0)
        + 2.0 * tap(center, texel, -1.0, 0.0) + 4.0 * tap(center, texel, 0.0, 0.0) + 2.0 * tap(center, texel, 1.0, 0.0)
        + tap(center, texel, -1.0, 1.0) + 2.0 * tap(center, texel, 0.0, 1.0) + tap(center, texel, 1.0, 1.0);
    return vec4<f32>(sum / 16.0, 1.0);
}
