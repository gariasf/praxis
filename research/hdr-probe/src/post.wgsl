// Throwaway probe for #189 (never merged): the final pass the HDR survey's
// options share. Undoes the resolve weighting, applies exposure and bloom, one
// tonemapping operator, the sRGB encoding and a static dither, in one
// full-screen pass. Override constants pick the variant, so each pipeline
// compiles one path.

// 0 clamp (today), 1 Reinhard on luminance, 2 Hable, 3 ACES (Narkowicz fit),
// 4 ACES (Hill fit), 5 AgX, 6 Khronos PBR Neutral, 7 Tony McMapface, 8 GT7.
override OPERATOR: u32 = 0u;
override DITHER: bool = false;
override INVERT_WEIGHTING: bool = false;
override BLOOM: bool = false;
// 1: one input texel per pixel. 4: a 4x4 block of input texels per pixel,
// averaged in scene-linear space (DISPLAY_AVERAGE false) or after the
// operator, in display-linear space (true).
override SUPERSAMPLE: u32 = 1u;
override DISPLAY_AVERAGE: bool = false;

struct Post {
    exposure: f32,
    bloom_strength: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0) var hdr_texture: texture_2d<f32>;
@group(0) @binding(1) var tony_lut: texture_3d<f32>;
@group(0) @binding(2) var linear_clamp: sampler;
@group(0) @binding(3) var<uniform> post: Post;
@group(0) @binding(4) var bloom_texture: texture_2d<f32>;

const REC709_LUMINANCE = vec3<f32>(0.2126, 0.7152, 0.0722);

@vertex
fn vs_fullscreen(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, REC709_LUMINANCE);
}

// The lit pass stores color / (1 + luminance(color)), so the hardware box
// resolve averages bounded values (Karis 2013); with a linear luminance the
// inverse below makes the resolve a 1 / (1 + luminance) weighted average.
fn invert_weighting(weighted: vec3<f32>) -> vec3<f32> {
    return weighted / max(1.0 - luminance(weighted), 1.0 / 65504.0);
}

fn tonemap_clamp(color: vec3<f32>) -> vec3<f32> {
    return min(color, vec3<f32>(1.0));
}

// Reinhard et al. 2002, equation 3, on luminance; the colour keeps its ratios.
fn tonemap_reinhard_luminance(color: vec3<f32>) -> vec3<f32> {
    return min(color / (1.0 + luminance(color)), vec3<f32>(1.0));
}

// Hable 2010 ("Filmic Tonemapping Operators", filmicworlds.com): his
// constants, exposure bias 2 and linear white 11.2.
fn hable_curve(x: vec3<f32>) -> vec3<f32> {
    let shoulder_strength = 0.15;
    let linear_strength = 0.50;
    let linear_angle = 0.10;
    let toe_strength = 0.20;
    let toe_numerator = 0.02;
    let toe_denominator = 0.30;
    return ((x * (shoulder_strength * x + linear_angle * linear_strength) + toe_strength * toe_numerator)
        / (x * (shoulder_strength * x + linear_strength) + toe_strength * toe_denominator))
        - toe_numerator / toe_denominator;
}

fn tonemap_hable(color: vec3<f32>) -> vec3<f32> {
    let exposure_bias = 2.0;
    let linear_white = 11.2;
    return min(hable_curve(exposure_bias * color) / hable_curve(vec3<f32>(linear_white)), vec3<f32>(1.0));
}

// Narkowicz 2016 ("ACES Filmic Tone Mapping Curve", CC0 or MIT), as published,
// without the 0.6 input scale that would restore the original ACES exposure.
fn tonemap_aces_narkowicz(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

// Stephen Hill's fit of ACES RRT + ODT, from BakingLab's ACES.hlsl (MIT);
// matrix columns are the HLSL rows.
fn tonemap_aces_hill(color: vec3<f32>) -> vec3<f32> {
    let input_matrix = mat3x3<f32>(
        vec3<f32>(0.59719, 0.07600, 0.02840),
        vec3<f32>(0.35458, 0.90834, 0.13383),
        vec3<f32>(0.04823, 0.01566, 0.83777),
    );
    let output_matrix = mat3x3<f32>(
        vec3<f32>(1.60475, -0.10208, -0.00327),
        vec3<f32>(-0.53108, 1.10813, -0.07276),
        vec3<f32>(-0.07367, -0.00605, 1.07602),
    );
    let v = input_matrix * color;
    let fitted = (v * (v + 0.0245786) - 0.000090537) / (v * (0.983729 * v + 0.4329510) + 0.238081);
    return clamp(output_matrix * fitted, vec3<f32>(0.0), vec3<f32>(1.0));
}

// AgX as three.js r186 implements it (MIT), after Filament and Blender: a
// Rec. 2020 working space, iolite's polynomial for the default-contrast
// sigmoid, no look.
fn tonemap_agx(color: vec3<f32>) -> vec3<f32> {
    let srgb_to_rec2020 = mat3x3<f32>(
        vec3<f32>(0.6274, 0.0691, 0.0164),
        vec3<f32>(0.3293, 0.9195, 0.0880),
        vec3<f32>(0.0433, 0.0113, 0.8956),
    );
    let rec2020_to_srgb = mat3x3<f32>(
        vec3<f32>(1.6605, -0.1246, -0.0182),
        vec3<f32>(-0.5876, 1.1329, -0.1006),
        vec3<f32>(-0.0728, -0.0083, 1.1187),
    );
    let inset = mat3x3<f32>(
        vec3<f32>(0.856627153315983, 0.137318972929847, 0.11189821299995),
        vec3<f32>(0.0951212405381588, 0.761241990602591, 0.0767994186031903),
        vec3<f32>(0.0482516061458583, 0.101439036467562, 0.811302368396859),
    );
    let outset = mat3x3<f32>(
        vec3<f32>(1.1271005818144368, -0.1413297634984383, -0.14132976349843826),
        vec3<f32>(-0.11060664309660323, 1.157823702216272, -0.11060664309660294),
        vec3<f32>(-0.016493938717834573, -0.016493938717834257, 1.2519364065950405),
    );
    let minimum_ev = -12.47393;
    let maximum_ev = 4.026069;
    var v = inset * (srgb_to_rec2020 * color);
    v = clamp((log2(max(v, vec3<f32>(1e-10))) - minimum_ev) / (maximum_ev - minimum_ev), vec3<f32>(0.0), vec3<f32>(1.0));
    let v2 = v * v;
    let v4 = v2 * v2;
    v = 15.5 * v4 * v2 - 40.14 * v4 * v + 31.96 * v4 - 6.868 * v2 * v + 0.4298 * v2 + 0.1191 * v - 0.00232;
    v = pow(max(outset * v, vec3<f32>(0.0)), vec3<f32>(2.2));
    return clamp(rec2020_to_srgb * v, vec3<f32>(0.0), vec3<f32>(1.0));
}

// Khronos PBR Neutral, from KhronosGroup/ToneMapping's pbrNeutral.glsl
// (Apache-2.0).
fn tonemap_pbr_neutral(input: vec3<f32>) -> vec3<f32> {
    let start_compression = 0.8 - 0.04;
    let desaturation = 0.15;
    let x = min(input.r, min(input.g, input.b));
    let offset = select(0.04, x - 6.25 * x * x, x < 0.08);
    var color = input - offset;
    let peak = max(color.r, max(color.g, color.b));
    if peak < start_compression {
        return color;
    }
    let d = 1.0 - start_compression;
    let new_peak = 1.0 - d * d / (peak + d - start_compression);
    color *= new_peak / peak;
    let g = 1.0 - 1.0 / (desaturation * (peak - new_peak) + 1.0);
    return mix(color, vec3<f32>(new_peak), g);
}

// Tony McMapface (h3r2tic, MIT or Apache-2.0): its 48^3 LUT, sampled at texel
// centres after the x / (x + 1) encoding the LUT was made with.
fn tonemap_tony(color: vec3<f32>) -> vec3<f32> {
    let encoded = color / (color + 1.0);
    let dimension = 48.0;
    let coordinates = encoded * ((dimension - 1.0) / dimension) + 0.5 / dimension;
    return textureSampleLevel(tony_lut, linear_clamp, coordinates, 0.0).rgb;
}

// GT7's SDR path (Yasutomi, Suzuki and Uchimura, SIGGRAPH 2025; Polyphony
// Digital's gt7_tone_mapping.cpp 1.0, MIT): frame-buffer 1.0 is 100 cd/m^2
// and SDR paper white 250 cd/m^2. The probe reads its input's 1.0 as that
// paper white, converts Rec. 709 to Rec. 2020 and back, and clips.
const GT7_PAPER_WHITE: f32 = 2.5;
const PQ_M1: f32 = 0.1593017578125;
const PQ_M2: f32 = 78.84375;
const PQ_C1: f32 = 0.8359375;
const PQ_C2: f32 = 18.8515625;
const PQ_C3: f32 = 18.6875;

fn gt7_curve(x: f32) -> f32 {
    let peak = GT7_PAPER_WHITE;
    let alpha = 0.25;
    let mid_point = 0.538;
    let linear_section = 0.444;
    let toe_strength = 1.280;
    let k = (linear_section - 1.0) / (alpha - 1.0);
    let k_a = peak * linear_section + peak * k;
    let k_b = -peak * k * exp(linear_section / k);
    let k_c = -1.0 / (k * peak);
    if x < 0.0 {
        return 0.0;
    }
    if x < linear_section * peak {
        let weight_linear = smoothstep(0.0, mid_point, x);
        let toe_mapped = mid_point * pow(x / mid_point, toe_strength);
        return (1.0 - weight_linear) * toe_mapped + weight_linear * x;
    }
    return k_a + k_b * exp(x * k_c);
}

fn pq_inverse_eotf(frame_buffer: f32) -> f32 {
    let normalized = pow(frame_buffer * 100.0 / 10000.0, PQ_M1);
    return exp2(PQ_M2 * (log2(PQ_C1 + PQ_C2 * normalized) - log2(1.0 + PQ_C3 * normalized)));
}

fn pq_eotf(signal: f32) -> f32 {
    let power = pow(clamp(signal, 0.0, 1.0), 1.0 / PQ_M2);
    let linear = max(power - PQ_C1, 0.0) / (PQ_C2 - PQ_C3 * power);
    return pow(linear, 1.0 / PQ_M1) * 10000.0 / 100.0;
}

fn rec2020_to_ictcp(rgb: vec3<f32>) -> vec3<f32> {
    let l = pq_inverse_eotf((rgb.r * 1688.0 + rgb.g * 2146.0 + rgb.b * 262.0) / 4096.0);
    let m = pq_inverse_eotf((rgb.r * 683.0 + rgb.g * 2951.0 + rgb.b * 462.0) / 4096.0);
    let s = pq_inverse_eotf((rgb.r * 99.0 + rgb.g * 309.0 + rgb.b * 3688.0) / 4096.0);
    return vec3<f32>(
        (2048.0 * l + 2048.0 * m) / 4096.0,
        (6610.0 * l - 13613.0 * m + 7003.0 * s) / 4096.0,
        (17933.0 * l - 17390.0 * m - 543.0 * s) / 4096.0,
    );
}

fn ictcp_to_rec2020(ictcp: vec3<f32>) -> vec3<f32> {
    let l = pq_eotf(ictcp.x + 0.00860904 * ictcp.y + 0.11103 * ictcp.z);
    let m = pq_eotf(ictcp.x - 0.00860904 * ictcp.y - 0.11103 * ictcp.z);
    let s = pq_eotf(ictcp.x + 0.560031 * ictcp.y - 0.320627 * ictcp.z);
    return max(vec3<f32>(
        3.43661 * l - 2.50645 * m + 0.0698454 * s,
        -0.79133 * l + 1.9836 * m - 0.192271 * s,
        -0.0259499 * l - 0.0989137 * m + 1.12486 * s,
    ), vec3<f32>(0.0));
}

fn tonemap_gt7(color: vec3<f32>) -> vec3<f32> {
    let srgb_to_rec2020 = mat3x3<f32>(
        vec3<f32>(0.6274, 0.0691, 0.0164),
        vec3<f32>(0.3293, 0.9195, 0.0880),
        vec3<f32>(0.0433, 0.0113, 0.8956),
    );
    let rec2020_to_srgb = mat3x3<f32>(
        vec3<f32>(1.6605, -0.1246, -0.0182),
        vec3<f32>(-0.5876, 1.1329, -0.1006),
        vec3<f32>(-0.0728, -0.0083, 1.1187),
    );
    let rgb = srgb_to_rec2020 * color * GT7_PAPER_WHITE;
    let ucs = rec2020_to_ictcp(rgb);
    let skewed = vec3<f32>(gt7_curve(rgb.r), gt7_curve(rgb.g), gt7_curve(rgb.b));
    let skewed_ucs = rec2020_to_ictcp(skewed);
    let target_intensity = rec2020_to_ictcp(vec3<f32>(GT7_PAPER_WHITE)).x;
    let chroma_scale = 1.0 - smoothstep(0.98, 1.16, ucs.x / target_intensity);
    let scaled = ictcp_to_rec2020(vec3<f32>(skewed_ucs.x, ucs.y * chroma_scale, ucs.z * chroma_scale));
    let blended = 0.4 * skewed + 0.6 * scaled;
    let display = min(blended, vec3<f32>(GT7_PAPER_WHITE)) / GT7_PAPER_WHITE;
    return clamp(rec2020_to_srgb * display, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn tonemap(color: vec3<f32>) -> vec3<f32> {
    switch OPERATOR {
        case 8u: { return tonemap_gt7(color); }
        case 1u: { return tonemap_reinhard_luminance(color); }
        case 2u: { return tonemap_hable(color); }
        case 3u: { return tonemap_aces_narkowicz(color); }
        case 4u: { return tonemap_aces_hill(color); }
        case 5u: { return tonemap_agx(color); }
        case 6u: { return tonemap_pbr_neutral(color); }
        case 7u: { return tonemap_tony(color); }
        default: { return tonemap_clamp(color); }
    }
}

// IEC 61966-2-1's sRGB encoding.
fn srgb_encode(display_linear: vec3<f32>) -> vec3<f32> {
    let clamped = clamp(display_linear, vec3<f32>(0.0), vec3<f32>(1.0));
    return select(
        1.055 * pow(clamped, vec3<f32>(1.0 / 2.4)) - 0.055,
        clamped * 12.92,
        clamped <= vec3<f32>(0.0031308),
    );
}

// Jarzynski and Olano 2020, "Hash Functions for GPU Rendering" (JCGT), pcg2d.
fn pcg2d(seed: vec2<u32>) -> vec2<u32> {
    var v = seed * 1664525u + 1013904223u;
    v.x += v.y * 1664525u;
    v.y += v.x * 1664525u;
    v = v ^ (v >> vec2<u32>(16u));
    v.x += v.y * 1664525u;
    v.y += v.x * 1664525u;
    v = v ^ (v >> vec2<u32>(16u));
    return v;
}

// Triangular-PDF noise in (-1, 1) quantisation steps, fixed per pixel: no
// frame index, so the pattern never moves.
fn triangular_noise(pixel: vec2<u32>) -> f32 {
    let unit = vec2<f32>(pcg2d(pixel) >> vec2<u32>(8u)) / 16777216.0;
    return unit.x + unit.y - 1.0;
}

fn scene_radiance(texel: vec2<i32>) -> vec3<f32> {
    var radiance = textureLoad(hdr_texture, texel, 0).rgb;
    if INVERT_WEIGHTING {
        radiance = invert_weighting(radiance);
    }
    return radiance * post.exposure;
}

@fragment
fn fs_post(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(position.xy);
    var display_linear = vec3<f32>(0.0);
    if SUPERSAMPLE == 1u {
        var radiance = scene_radiance(pixel);
        if BLOOM {
            let coordinates = position.xy / vec2<f32>(textureDimensions(hdr_texture));
            let bloom = textureSampleLevel(bloom_texture, linear_clamp, coordinates, 0.0).rgb * post.exposure;
            radiance = mix(radiance, bloom, post.bloom_strength);
        }
        display_linear = tonemap(radiance);
    } else {
        var sum = vec3<f32>(0.0);
        for (var y = 0; y < 4; y++) {
            for (var x = 0; x < 4; x++) {
                let radiance = scene_radiance(pixel * 4 + vec2<i32>(x, y));
                if DISPLAY_AVERAGE {
                    sum += tonemap(radiance);
                } else {
                    sum += radiance;
                }
            }
        }
        if DISPLAY_AVERAGE {
            display_linear = sum / 16.0;
        } else {
            display_linear = tonemap(sum / 16.0);
        }
    }
    var encoded = srgb_encode(display_linear);
    if DITHER {
        encoded += triangular_noise(vec2<u32>(position.xy)) / 255.0;
    }
    return vec4<f32>(encoded, 1.0);
}
