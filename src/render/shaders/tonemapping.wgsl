// The tonemapping pass's entry points; its binding is declared from its bind
// group's definition (src/render/bind_groups.rs).

// (−1, −1), (3, −1) and (−1, 3): one triangle that covers the target.
@vertex
fn vs_full_screen(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    let corner = vec2<f32>(f32((vertex_index << 1u) & 2u), f32(vertex_index & 2u));
    return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

// A pixel's exposed colour, clamped to display white as the 8-bit target
// clamped it before the HDR target; #200's GT7 replaces the clamp.
@fragment
fn fs_tonemap(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let exposed = textureLoad(hdr_target, vec2<u32>(position.xy), 0).rgb;
    return vec4<f32>(min(exposed, vec3<f32>(1.0)), 1.0);
}
