// What turns exposed colour into display colour, after GT7's part: the
// conversions around GT7 and its reading of exposure.

// Linear colour from Rec. 709's primaries to Rec. 2020's, and back, each column
// a primary (ITU-R BT.2087-0's matrix [ref:bt2087-rec709-to-rec2020], derived
// again from BT.709-6's [ref:bt709-6-primaries] and BT.2020-2's
// [ref:bt2020-2-primaries] primaries and D65 to seven places).
const REC709_TO_REC2020 = mat3x3<f32>(
    vec3<f32>(0.6274039, 0.0690973, 0.0163914),
    vec3<f32>(0.3292830, 0.9195404, 0.0880133),
    vec3<f32>(0.0433131, 0.0113623, 0.8955953),
);
const REC2020_TO_REC709 = mat3x3<f32>(
    vec3<f32>(1.6604910, -0.1245505, -0.0181508),
    vec3<f32>(-0.5876411, 1.1328999, -0.1005789),
    vec3<f32>(-0.0728499, -0.0083494, 1.1187297),
);

// Display-linear colour in Rec. 709's primaries, in [0, 1], from exposed
// colour: GT7, reading exposed 1 as its SDR paper white, so GT7's linear
// section keeps exposed mid-tones as they are, clipped back into Rec. 709.
fn tonemap(exposed: vec3<f32>) -> vec3<f32> {
    let display = gt7_tonemap(REC709_TO_REC2020 * exposed * GT7_PAPER_WHITE);
    return clamp(REC2020_TO_REC709 * display, vec3<f32>(0.0), vec3<f32>(1.0));
}

// IEC 61966-2-1's sRGB encoding of display-linear colour
// [ref:khronos-dfs-srgb], clamped to [0, 1]. The tonemapping pass encodes here,
// once, into an 8-bit UNORM target, so its dither lands on 8-bit steps.
fn srgb_encode(display: vec3<f32>) -> vec3<f32> {
    let clamped = clamp(display, vec3<f32>(0.0), vec3<f32>(1.0));
    return select(
        1.055 * pow(clamped, vec3<f32>(1.0 / 2.4)) - 0.055,
        12.92 * clamped,
        clamped <= vec3<f32>(0.0031308)
    );
}

// Noise on (−1, 1) of triangular density, in 8-bit steps, from a value of
// the dither mask: one of 256 levels, each standing for its bin's centre in
// (0, 1), turned by the inverse of the triangular distribution's CDF, as
// Peters remaps a blue-noise mask for dithering
// (momentsingraphics.de/BlueNoise.html, "Dithering" [ref:peters-blue-noise]).
// Of the dithers that remove noise modulation, triangular noise of two steps
// peak to peak adds the least noise (Lipshitz, Wannamaker and Vanderkooy
// 1992, p. 372 [ref:lipshitz-1992-triangular-dither]).
fn dither_noise(mask_value: f32) -> f32 {
    let uniform = (round(mask_value * 255.0) + 0.5) / 256.0;
    let centred = 2.0 * uniform - 1.0;
    return sign(centred) * (1.0 - sqrt(1.0 - abs(centred)));
}
