// The tonemapping pass's fragment entry point, after the HDR, GT7, display
// and full-screen triangle parts; its binding is declared from its bind
// group's definition (src/render/bind_groups.rs).

// A pixel's display colour, from its exposed colour, unweighted from the
// resolve, sRGB-encoded and dithered by up to an 8-bit step either way with
// the mask's triangular noise, fixed to the screen.
@fragment
fn fs_tonemap(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<u32>(position.xy);
    let weighted = textureLoad(hdr_target, pixel, 0).rgb;
    let encoded = srgb_encode(tonemap(resolve_unweighted(weighted)));
    // DITHER_MASK_SIDE, a power of two declared from Rust's
    // (src/render/shaders/mod.rs), tiles the mask with a bit mask.
    let noise = dither_noise(textureLoad(dither_mask, pixel % DITHER_MASK_SIDE, 0).r);
    return vec4<f32>(encoded + noise / 255.0, 1.0);
}
