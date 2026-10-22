// What turns exposed colour into display colour, after GT7's part: the
// conversions around GT7 and its reading of exposure.

// Linear colour from Rec. 709's primaries to Rec. 2020's, and back, each
// column a primary (ITU-R BT.2087-0's matrix, derived again from BT.709-6's
// and BT.2020-2's primaries and D65 to seven places).
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
