// Throwaway probe for #189 (never merged): what a float16 colour attachment
// stores when a fragment writes more than 65504, and what one such sample
// does to a 4x hardware resolve. Values come from a uniform, so nothing is
// folded at shader compile time.

struct Values {
    per_pixel: array<vec4<f32>, 8>,
}

@group(0) @binding(0) var<uniform> values: Values;

@vertex
fn vs_triangle(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    // Full screen for the 1x test; the 4x test passes a small triangle as
    // instance 1 around sample 0 of a 1x1 target, (0.375, 0.125) in pixel
    // units.
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

@vertex
fn vs_sample_zero(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    var corners = array<vec2<f32>, 3>(vec2<f32>(-0.35, 0.65), vec2<f32>(-0.15, 0.65), vec2<f32>(-0.25, 0.85));
    return vec4<f32>(corners[index], 0.0, 1.0);
}

@fragment
fn fs_value(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return values.per_pixel[u32(position.x) % 8u];
}
