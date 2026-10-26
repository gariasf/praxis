// Exposed colour as the lit pass stores it in the HDR target and the
// tonemapping pass reads it back (ADR 0023).

// fp16's largest finite value. Exposed colour is clamped to it, so no
// backend stores infinity, which Vulkan allows on overflow (ADR 0023)
// [ref:wgsl-f16-largest] [ref:vulkan-fp-conversion].
const LARGEST_HALF: f32 = 65504.0;

// The relative luminance of a linear colour in Rec. 709's primaries, the
// scene's, from LUMINANCE_COEFFICIENTS, ITU-R BT.709-6's, which is declared
// from Rust's (src/render/shaders/mod.rs).
fn relative_luminance(color: vec3<f32>) -> f32 {
    return dot(color, LUMINANCE_COEFFICIENTS);
}

// Exposed colour weighted by 1 / (1 + its luminance), as every shader of the
// lit pass writes it: the hardware's box resolve of 4x samples, undone by
// `resolve_unweighted`, then averages them weighted alike, so a bright
// sample cannot whiten an edge (Karis 2013, "Tone mapping",
// graphicrants.blogspot.com/2013/12/tone-mapping.html
// [ref:karis-2013-weighted-average]). Weighted luminance
// stays below 1.
fn resolve_weighted(color: vec3<f32>) -> vec3<f32> {
    return color / (1.0 + relative_luminance(color));
}

// The inverse of `resolve_weighted`, weighted / (1 − luminance(weighted)),
// clamped to LARGEST_HALF, which a resolved luminance at or past 1 gives.
fn resolve_unweighted(weighted: vec3<f32>) -> vec3<f32> {
    let complement = max(1.0 - relative_luminance(weighted), 1.0 / LARGEST_HALF);
    return min(weighted / complement, vec3<f32>(LARGEST_HALF));
}
