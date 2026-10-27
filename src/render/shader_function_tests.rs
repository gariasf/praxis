//! Functions of the shared shader parts run on the GPU over test vectors and
//! are compared with oracles written apart from the shader: literals, analytic
//! results and CPU references.

use std::io::Write;

use crate::cooked::CookedVertex;
use crate::gpu::GpuContext;
use crate::render::instance::{ANCHOR_CELL_SIZE, anchor_and_offset};
use crate::render::shaders;

/// Largest difference allowed between a result and its expectation. Results
/// land within 2e-7 of theirs on the M4 Pro.
const TOLERANCE: f32 = 1e-6;

/// Schlick adds (1 − |cos θ|)⁵ of the light the base reflectance misses:
/// none at normal incidence, or where a dot product of unit vectors rounds
/// just past 1, 1/32 at cos θ = ±0.5, and all of it at grazing.
#[test]
fn fresnel_rises_from_the_base_reflectance_to_one_at_grazing() {
    let Some(context) = gpu_context() else {
        return;
    };
    // One base reflectance per channel: a dielectric, a midpoint, a mirror.
    let base_reflectance = [0.04, 0.5, 1.0];
    let cases = [
        (1.0, base_reflectance),
        (f32::from_bits(1.0f32.to_bits() + 1), base_reflectance),
        (0.5, [0.07, 0.515625, 1.0]),
        (-0.5, [0.07, 0.515625, 1.0]),
        (0.0, [1.0, 1.0, 1.0]),
    ];

    let [red, green, blue] = base_reflectance;
    let inputs = cases.map(|(cosine, _)| [cosine, red, green, blue]);
    let results = evaluate(
        &context,
        "vec4<f32>(input.yzw + (vec3<f32>(1.0) - input.yzw) * fresnel_weight(input.x), 0.0)",
        &inputs,
    );

    for ((cosine, expected), result) in cases.into_iter().zip(results) {
        assert!(
            result
                .iter()
                .zip(expected)
                .all(|(actual, expected)| (actual - expected).abs() <= TOLERANCE),
            "F at cos θ = {cosine} is {:?}, expected {expected:?}",
            &result[..3]
        );
    }
}

/// A microfacet distribution weighted by projected area integrates to 1 over
/// the hemisphere (Walter et al. 2007, eq. 4
/// [ref:walter-2007-projected-area]): 2π ∫ D(cos θ) cos θ sin θ dθ
/// = 1 for θ from 0 to π/2. The sum takes θ = π/2 · u² at the midpoints of
/// even steps in u, which crowds samples near the normal, where D peaks.
#[test]
fn distribution_ggx_integrates_to_one_over_the_hemisphere() {
    let Some(context) = gpu_context() else {
        return;
    };
    // Below 0.25, f32 holds cos θ too coarsely near D's peak: at the
    // renderer's floor of 0.04 the sum lands 9e-4 short, so the floor is
    // checked against D's f64 reference instead.
    let roughnesses = [0.25, 0.5, 0.75, 1.0];
    let sample_count = 4096;
    // Each sample's angle θ and the width dθ = π u du it stands for.
    let samples: Vec<(f64, f64)> = (0..sample_count)
        .map(|sample| {
            let position = (sample as f64 + 0.5) / sample_count as f64;
            (
                std::f64::consts::FRAC_PI_2 * position * position,
                std::f64::consts::PI * position / sample_count as f64,
            )
        })
        .collect();

    let inputs: Vec<[f32; 4]> = roughnesses
        .iter()
        .flat_map(|&roughness| {
            samples
                .iter()
                .map(move |(angle, _)| [angle.cos() as f32, roughness, 0.0, 0.0])
        })
        .collect();
    let results = evaluate(
        &context,
        "vec4<f32>(distribution_ggx(input.x, input.y * input.y))",
        &inputs,
    );

    for (roughness, distribution) in roughnesses.iter().zip(results.chunks(sample_count)) {
        let integral = samples
            .iter()
            .zip(distribution)
            .map(|((angle, width), result)| {
                f64::from(result[0]) * angle.cos() * angle.sin() * width
            })
            .sum::<f64>()
            * std::f64::consts::TAU;
        // The sums land within 2e-7 of 1 on the M4 Pro; the sampling alone
        // is off by under 1e-7.
        assert!(
            (integral - 1.0).abs() <= 1e-5,
            "roughness {roughness}: D integrates to {integral}, expected 1"
        );
    }

    // Any normalised D passes the sum, so the peak pins the shape: at n·h = 1,
    // D is 1/(π α²) (Walter et al. 2007, eq. 33 [ref:walter-2007-ggx]), with
    // α = roughness² (glTF 2.0, Appendix B [ref:gltf-2.0-roughness-mapping]),
    // which is 16/π at roughness 0.5.
    let peak = evaluate(
        &context,
        "vec4<f32>(distribution_ggx(input.x, input.y * input.y))",
        &[[1.0, 0.5, 0.0, 0.0]],
    )[0][0];
    assert!(
        (f64::from(peak) - 16.0 / std::f64::consts::PI).abs() <= f64::from(TOLERANCE),
        "D at n·h = 1, roughness 0.5, is {peak}, expected 16/π"
    );
}

/// D matches glTF 2.0 Appendix B's Trowbridge-Reitz formula, evaluated in
/// f64, from below the horizon, where χ⁺ makes it 0, to the normal, and from
/// the roughness floor to 1. At the floor's peak the formula's
/// (N·H)² (α² − 1) + 1 falls to α² = 2.56e-6, which f32 keeps only as the
/// shader sums it.
#[test]
fn distribution_ggx_matches_appendix_b_from_the_roughness_floor_to_one() {
    let Some(context) = gpu_context() else {
        return;
    };
    let cosines = [-0.5, 0.0, 1e-3, 0.5, 0.9, 0.99, 0.999, 0.9999, 0.99999, 1.0];
    let inputs: Vec<[f32; 4]> = [minimum_roughness(&context), 0.25, 0.5, 1.0]
        .into_iter()
        .flat_map(|roughness| cosines.map(|cosine| [cosine, roughness * roughness, 0.0, 0.0]))
        .collect();
    let results = evaluate(
        &context,
        "vec4<f32>(distribution_ggx(input.x, input.y))",
        &inputs,
    );

    for (&[normal_dot_half, alpha, ..], result) in inputs.iter().zip(results) {
        let expected = reference_distribution(normal_dot_half.into(), alpha.into());
        let case = format_args!("D(N·H = {normal_dot_half}, α = {alpha})");
        assert_close(result[0], expected, RELATIVE_TOLERANCE * expected, case);
    }
}

/// V, Smith's height-correlated masking-shadowing over 4 |N·L| |N·V|,
/// matches Appendix B's formula, evaluated in f64, at cosines from grazing to
/// the normal and widths from the roughness floor to 1, and is 0, not
/// infinite, where both cosines are 0, and where χ⁺(H·V) is 0. G, which is
/// 4 |N·L| |N·V| V, the share of microfacets both seen and lit, stays in
/// [0, 1].
#[test]
fn smith_visibility_matches_appendix_b_and_keeps_g_within_zero_to_one() {
    let Some(context) = gpu_context() else {
        return;
    };
    let cosines = [0.0, 1e-4, 0.01, 0.3, 0.7, 1.0];
    let mut inputs = Vec::new();
    for roughness in [minimum_roughness(&context), 0.5, 1.0] {
        for normal_dot_light in cosines {
            for normal_dot_view in cosines {
                for view_dot_half in [0.0, 0.5] {
                    let alpha = roughness * roughness;
                    inputs.push([normal_dot_light, normal_dot_view, view_dot_half, alpha]);
                }
            }
        }
    }
    let results = evaluate(
        &context,
        "vec4<f32>(smith_visibility(input.x, input.y, input.z, input.w))",
        &inputs,
    );

    for (&[normal_dot_light, normal_dot_view, view_dot_half, alpha], result) in
        inputs.iter().zip(results)
    {
        let input = [normal_dot_light, normal_dot_view, view_dot_half, alpha].map(f64::from);
        let expected = reference_visibility(input);
        let case = format_args!("V{input:?}");
        assert_close(result[0], expected, RELATIVE_TOLERANCE * expected, case);
        let geometry = 4.0 * normal_dot_light * normal_dot_view * result[0];
        assert!(geometry <= 1.0 + TOLERANCE, "{case}: G is {geometry}");
    }
}

/// The BRDF, mix(dielectric_brdf, metal_brdf, metallic), matches Appendix
/// B's "full code" evaluated in f64, from the cosines of lights and views
/// from the normal to grazing, at the roughness floor and above, for a
/// dielectric, a metal and half of each, and is reciprocal: the light and
/// view swapped give the same value.
#[test]
fn metallic_roughness_brdf_matches_appendix_b_and_is_reciprocal() {
    let Some(context) = gpu_context() else {
        return;
    };
    let polar_angles = [0.0, 30.0, 60.0, 85.0, 90.0];
    let mut cases = Vec::new();
    for roughness in [minimum_roughness(&context), 0.5, 1.0] {
        for metallic in [0.0, 0.5, 1.0] {
            for view_angle in polar_angles {
                for light_angle in polar_angles {
                    for azimuth in [0.0, 90.0, 180.0] {
                        let view = direction(view_angle, 0.0);
                        let light = direction(light_angle, azimuth);
                        let cosines =
                            cosines(glam::DVec3::Z, view, light).map(|cosine| cosine as f32);
                        let [red, green, blue] = GOLD;
                        cases.push([cosines, [red, green, blue, metallic], [roughness; 4]]);
                    }
                }
            }
        }
    }
    let surface = surface("inputs[1].rgb", "inputs[1].w", "inputs[2].x", "1.0");
    let brdf = |normal_dot_light: &str, normal_dot_view: &str| {
        format!(
            "vec4<f32>(metallic_roughness_brdf({surface}, {normal_dot_light}, \
             {normal_dot_view}, inputs[0].z, inputs[0].w), 0.0)"
        )
    };
    let results = evaluate_cases(&context, &brdf("inputs[0].x", "inputs[0].y"), &cases, &[]);
    let swapped = evaluate_cases(&context, &brdf("inputs[0].y", "inputs[0].x"), &cases, &[]);

    for ((&[cosines, material, [roughness, ..]], result), swapped) in
        cases.iter().zip(results).zip(swapped)
    {
        let [red, green, blue, metallic] = material.map(f64::from);
        let cosines = cosines.map(f64::from);
        let expected = reference_brdf(cosines, [red, green, blue], metallic, roughness.into());
        for channel in 0..3 {
            let case = format_args!(
                "BRDF{cosines:?} metallic {metallic} roughness {roughness}, channel {channel}"
            );
            let tolerance = RELATIVE_TOLERANCE * expected[channel];
            assert_close(result[channel], expected[channel], tolerance, case);
            let value = f64::from(result[channel]);
            let tolerance = RELATIVE_TOLERANCE * value;
            assert_close(
                swapped[channel],
                value,
                tolerance,
                format_args!("{case}, swapped"),
            );
        }
    }
}

/// What a point reflects from a punctual light, f(L, V) E (N·L)⁺, from unit
/// vectors around a normal off every axis, matches the f64 reference: 0 for a
/// light at or below the horizon, finite where the light is opposite the
/// view, even at the roughness floor. A white metal reflects at most what it
/// receives: under illuminance 1 from each direction of the hemisphere in
/// turn, what it reflects sums to at most 1 (a white furnace). Appendix B's
/// dielectric is not held to that, since its fresnel_mix is not energy
/// conserving (Appendix B, "Coupling Diffuse and Specular Reflection"
/// [ref:gltf-2.0-coupling]).
#[test]
fn reflected_luminance_follows_the_brdf_and_a_white_metal_reflects_at_most_what_it_receives() {
    let Some(context) = gpu_context() else {
        return;
    };
    let tilt = glam::DQuat::from_euler(glam::EulerRot::XYZ, 0.3, -0.5, 0.7);
    let case = |view_angle: f64, light: glam::DVec3, metallic: f32, roughness: f32| {
        let [normal, view, light] = [glam::DVec3::Z, direction(view_angle, 0.0), light]
            .map(|vector| (tilt * vector).as_vec3().extend(0.0).to_array());
        let [red, green, blue] = GOLD;
        [
            normal,
            view,
            light,
            [red, green, blue, metallic],
            [roughness; 4],
        ]
    };
    let floor = minimum_roughness(&context);
    let mut cases = Vec::new();
    for (view_angle, light) in [
        (60.0, direction(90.0, 180.0)),
        (60.0, direction(120.0, 180.0)),
        (90.0, direction(90.0, 180.0)),
        (90.0, direction(45.0, 180.0)),
        (120.0, -direction(120.0, 0.0)),
    ] {
        for metallic in [0.0, 1.0] {
            cases.push(case(view_angle, light, metallic, floor));
        }
    }
    // The hemisphere in midpoints of 1° by 2° cells, each the solid angle
    // sin θ dθ dφ of its cell.
    let cells: Vec<(glam::DVec3, f64)> = (0..90)
        .flat_map(|polar| (0..180).map(move |azimuth| (polar, azimuth)))
        .map(|(polar, azimuth)| {
            let polar_angle = f64::from(polar) + 0.5;
            let solid_angle = polar_angle.to_radians().sin() * 2f64.to_radians().powi(2) / 2.0;
            (
                direction(polar_angle, f64::from(azimuth) * 2.0 + 1.0),
                solid_angle,
            )
        })
        .collect();
    let furnaces = [
        (0.0, 0.5),
        (60.0, 0.5),
        (85.0, 0.5),
        (0.0, 1.0),
        (85.0, 1.0),
    ];
    for (view_angle, roughness) in furnaces {
        for &(light, _) in &cells {
            let mut white_metal = case(view_angle, light, 1.0, roughness);
            white_metal[3] = [1.0; 4];
            cases.push(white_metal);
        }
    }
    let surface = surface("inputs[3].rgb", "inputs[3].w", "inputs[4].x", "1.0");
    let results = evaluate_cases(
        &context,
        &format!(
            "vec4<f32>(reflected_luminance({surface}, inputs[0].xyz, inputs[1].xyz, \
             inputs[2].xyz, vec3<f32>(1.0)), 0.0)"
        ),
        &cases,
        &[],
    );

    for (case, result) in cases.iter().zip(&results) {
        let [normal, view, light, material, roughness] =
            case.map(|vector| glam::Vec4::from_array(vector).as_dvec4());
        let [normal, view, light] = [normal, view, light].map(glam::DVec4::truncate);
        let brdf = reference_brdf(
            cosines(normal, view, light),
            material.truncate().to_array(),
            material.w,
            roughness.x,
        );
        let normal_dot_light = normal.dot(light).max(0.0);
        // f32's N·L is three products and two sums, each rounded once: Vulkan
        // rounds them correctly (its precision table
        // [ref:vulkan-spirv-precision]) and D3D to 0.5 ULP
        // [ref:d3d11.3-fp32-rules]. Metal's fast math, which wgpu-hal leaves
        // at its default [ref:wgpu-hal-30-metal-compile-options], rounds
        // them and fused multiply-adds correctly but may reorder them
        // [ref:msl-4.1-fast-math], and Metal may round toward zero
        // [ref:msl-4.1-rounding]. In any order and with any fusing, the dot
        // product errs by at most γ₃ |N|ᵀ|L| ≤ γ₃ ‖N‖ ‖L‖ with
        // γ₃ = 3u / (1 − 3u) (Higham, Accuracy and Stability of Numerical
        // Algorithms, §3.1 [ref:higham-3.1]), where u is 2⁻²³ to cover
        // rounding toward zero, not round-to-nearest's 2⁻²⁴. Against
        // N·L ≈ 0, for a light on the horizon, that error is all there is,
        // which a tolerance relative to the result rules out.
        let unit_roundoff = f64::from(f32::EPSILON);
        let normal_dot_light_rounding =
            3.0 * unit_roundoff / (1.0 - 3.0 * unit_roundoff) * normal.length() * light.length();
        for channel in 0..3 {
            let expected = brdf[channel] * normal_dot_light;
            let case = format_args!("{case:?}, channel {channel}");
            assert_close(
                result[channel],
                expected,
                DIRECTION_TOLERANCE * expected + brdf[channel] * normal_dot_light_rounding,
                case,
            );
        }
    }
    let furnace_results = results[cases.len() - furnaces.len() * cells.len()..].chunks(cells.len());
    for ((view_angle, roughness), results) in furnaces.into_iter().zip(furnace_results) {
        let albedo: f64 = results
            .iter()
            .zip(&cells)
            .map(|(result, (_, solid_angle))| f64::from(result[0]) * solid_angle)
            .sum();
        assert!(
            albedo <= 1.0,
            "a white metal of roughness {roughness}, seen {view_angle}° off its normal, \
             reflects {albedo} of what it receives"
        );
    }
}

/// A point light's attenuation is KHR_lights_punctual's
/// max(min(1 − (d / r)⁴, 1), 0) / d², evaluated in f64 from the same d² the
/// shader takes: inverse square near the light, then windowed smoothly to
/// exactly 0 at the range and past it. Errors are judged against the
/// inverse square alone, since f32 cancels to a few ulps of it in
/// 1 − (d / r)⁴ just inside the range.
#[test]
fn point_light_attenuation_falls_by_inverse_square_to_zero_at_the_range() {
    let Some(context) = gpu_context() else {
        return;
    };
    let cases = [
        (0.02, 4.0),
        (0.5, 1.0),
        (1.0, 10.0),
        (2.0, 4.0),
        (3.0, 4.0),
        (3.999, 4.0),
        (4.0, 4.0),
        (5.0, 4.0),
        (20.0, 20.0),
    ];
    let inputs = cases.map(|(distance, range): (f32, f32)| [distance * distance, range, 0.0, 0.0]);
    let results = evaluate(
        &context,
        "vec4<f32>(point_light_attenuation(input.x, input.y))",
        &inputs,
    );

    for ([distance_squared, range, ..], result) in inputs.into_iter().zip(results) {
        let (distance_squared, range) = (f64::from(distance_squared), f64::from(range));
        let inverse_square = 1.0 / distance_squared;
        let window = (1.0 - (distance_squared / (range * range)).powi(2)).clamp(0.0, 1.0);
        let case = format_args!(
            "attenuation at {} m of a {range} m range",
            distance_squared.sqrt()
        );
        let tolerance = RELATIVE_TOLERANCE * inverse_square;
        assert_close(result[0], window * inverse_square, tolerance, case);
        if distance_squared >= range * range {
            assert_eq!(result[0], 0.0, "{case}");
        }
    }
}

/// What a surface point reflects from a point light matches the f64
/// reference: the light's colour times its intensity, attenuated over the
/// light's distance within its range, distance² held at 1 cm² or more, then
/// reflected from its direction as `reflected_luminance` reflects it, for
/// lights near and far, beside and above the normal, up to and past the
/// range, where it is exactly 0; at the light itself, which has no
/// direction, it is 0, not NaN.
#[test]
fn point_light_luminance_attenuates_the_intensity_it_reflects() {
    let Some(context) = gpu_context() else {
        return;
    };
    let range = 4.0;
    let intensity = [2.0, 1.0, 0.5, 0.0];
    let normal = [0.0, 0.0, 1.0, 0.0];
    let view = direction(30.0, 0.0).as_vec3().extend(0.0).to_array();
    let [red, green, blue] = GOLD;
    let mut offsets = vec![glam::DVec3::ZERO];
    for (polar, azimuth) in [(0.0, 0.0), (45.0, 180.0), (80.0, 90.0), (120.0, 0.0)] {
        for distance in [0.5, 2.0, 3.9, 4.0, 5.0] {
            offsets.push(direction(polar, azimuth) * distance);
        }
    }
    let cases: Vec<[[f32; 4]; 6]> = offsets
        .iter()
        .map(|offset| {
            let offset = offset.as_vec3().extend(0.0).to_array();
            let material = [red, green, blue, 0.5];
            [
                normal,
                view,
                offset,
                material,
                [0.5, range, 0.0, 0.0],
                intensity,
            ]
        })
        .collect();
    let surface = surface("inputs[3].rgb", "inputs[3].w", "inputs[4].x", "1.0");
    let results = evaluate_cases(
        &context,
        &format!(
            "vec4<f32>(point_light_luminance({surface}, inputs[0].xyz, inputs[1].xyz, \
             inputs[2].xyz, inputs[5].rgb, inputs[4].y), 0.0)"
        ),
        &cases,
        &[],
    );

    for (case, result) in cases.iter().zip(results) {
        let [normal, view, offset] = [case[0], case[1], case[2]]
            .map(|vector| glam::Vec4::from_array(vector).truncate().as_dvec3());
        let distance = offset.length();
        let light = offset.try_normalize().unwrap_or(glam::DVec3::ZERO);
        let brdf = reference_brdf(
            cosines(normal, view, light),
            [red, green, blue].map(f64::from),
            0.5,
            0.5,
        );
        let inverse_square = 1.0 / (distance * distance).max(1e-4);
        let window = (1.0 - (distance / f64::from(range)).powi(4)).clamp(0.0, 1.0);
        let normal_dot_light = normal.dot(light).max(0.0);
        for channel in 0..3 {
            let reflected = brdf[channel] * f64::from(intensity[channel]) * normal_dot_light;
            let case = format_args!("a light {distance} m off along {light}, channel {channel}");
            let tolerance = DIRECTION_TOLERANCE * reflected * inverse_square;
            assert_close(
                result[channel],
                reflected * window * inverse_square,
                tolerance,
                case,
            );
            if distance >= f64::from(range) || distance == 0.0 {
                assert_eq!(result[channel], 0.0, "{case}");
            }
        }
    }
}

/// Ambient light reaches every surface as a stand-in for ADR 0020's
/// irradiance probes, which light diffuse and rough specular reflection
/// alike, and leaves as from a Lambertian surface of its base colour, E/π,
/// darkened by occlusion: under an illuminance of π, the base colour times
/// occlusion, whatever the metallic.
#[test]
fn ambient_luminance_is_the_base_colour_over_pi_darkened_by_occlusion() {
    let Some(context) = gpu_context() else {
        return;
    };
    let cases = [(0.0, 1.0), (0.5, 1.0), (1.0, 1.0), (0.0, 0.25)];
    let inputs = cases.map(|(metallic, occlusion)| [metallic, occlusion, 0.0, 0.0]);
    let [red, green, blue] = GOLD;
    let base_color = format!("vec3<f32>({red}, {green}, {blue})");
    let surface = surface(&base_color, "input.x", "0.5", "input.y");
    let results = evaluate(
        &context,
        &format!("vec4<f32>(ambient_luminance({surface}, vec3<f32>(PI)), 0.0)"),
        &inputs,
    );

    for ((metallic, occlusion), result) in cases.into_iter().zip(results) {
        for (channel, base_color) in GOLD.into_iter().enumerate() {
            let expected = f64::from(base_color * occlusion);
            let case = format_args!("metallic {metallic}, occlusion {occlusion}");
            assert_close(
                result[channel],
                expected,
                RELATIVE_TOLERANCE * expected,
                case,
            );
        }
    }
}

/// Rec. 709's relative luminance weighs each linear primary by ITU-R BT.709-6's
/// coefficients [ref:bt709-6-luma], which sum to 1, so white's is 1.
#[test]
fn relative_luminance_weighs_the_primaries_by_bt_709() {
    let Some(context) = gpu_context() else {
        return;
    };
    let cases = [
        ([1.0, 0.0, 0.0], 0.2126),
        ([0.0, 1.0, 0.0], 0.7152),
        ([0.0, 0.0, 1.0], 0.0722),
        ([1.0, 1.0, 1.0], 1.0),
    ];
    let inputs = cases.map(|([red, green, blue], _)| [red, green, blue, 0.0]);
    let results = evaluate(
        &context,
        "vec4<f32>(relative_luminance(input.xyz))",
        &inputs,
    );

    for ((color, expected), result) in cases.into_iter().zip(results) {
        let case = format_args!("the luminance of {color:?}");
        assert_close(result[0], expected, TOLERANCE.into(), case);
    }
}

/// The lit pass weights exposed colour by 1/(1 + luminance), the hardware's box
/// resolve averages four samples so weighted, and the tonemapping pass undoes
/// the weighting: the result is the samples' average weighted by 1/(1 +
/// luminance) (Karis 2013 [ref:karis-2013-weighted-average]), Σ w c / Σ w in
/// f64. One sample of 100 over three of 0.05 resolves to 0.395 where the box
/// gives 25.04, so a bright sub-pixel highlight no longer whitens its edge, and
/// four equal samples resolve to their colour. A weighted luminance at or past
/// 1, which only rounding reaches, undoes to fp16's largest finite value, not
/// infinity.
#[test]
fn the_weighted_resolve_undone_averages_samples_by_one_over_one_plus_luminance() {
    let Some(context) = gpu_context() else {
        return;
    };
    let grey = |value: f32| [value; 3];
    let sample_sets = [
        [[0.5, 0.25, 0.125]; 4],
        [grey(100.0), grey(0.05), grey(0.05), grey(0.05)],
        [[10.0, 0.0, 0.0], [0.0, 10.0, 0.0], grey(0.0), grey(0.0)],
        [
            [2047.0, 1000.0, 3.0],
            [0.0, 0.1, 0.0],
            grey(1.0),
            [0.2, 0.3, 0.4],
        ],
        [grey(0.0); 4],
    ];
    let cases =
        sample_sets.map(|samples| samples.map(|[red, green, blue]| [red, green, blue, 0.0]));
    let results = evaluate_cases::<4>(
        &context,
        "vec4<f32>(resolve_unweighted((resolve_weighted(inputs[0].xyz) \
             + resolve_weighted(inputs[1].xyz) + resolve_weighted(inputs[2].xyz) \
             + resolve_weighted(inputs[3].xyz)) / 4.0), 0.0)",
        &cases,
        &[],
    );

    for (samples, result) in sample_sets.into_iter().zip(results) {
        let weights = samples.map(|sample| {
            let [red, green, blue] = sample.map(f64::from);
            1.0 / (1.0 + 0.2126 * red + 0.7152 * green + 0.0722 * blue)
        });
        let weight_sum: f64 = weights.iter().sum();
        for channel in 0..3 {
            let weighted_sum: f64 = samples
                .iter()
                .zip(weights)
                .map(|(sample, weight)| weight * f64::from(sample[channel]))
                .sum();
            let expected = weighted_sum / weight_sum;
            let case = format_args!("channel {channel} of {samples:?}");
            let tolerance = RELATIVE_TOLERANCE * expected.max(1.0);
            assert_close(result[channel], expected, tolerance, case);
        }
    }

    let limits = evaluate(
        &context,
        "vec4<f32>(resolve_unweighted(input.xyz), 0.0)",
        &[
            [1.0, 1.0, 1.0, 0.0],
            [2.0, 2.0, 2.0, 0.0],
            [0.0, 1.5, 0.0, 0.0],
        ],
    );
    let limits: Vec<[f32; 3]> = limits
        .iter()
        .map(|result| [result[0], result[1], result[2]])
        .collect();
    assert_eq!(limits, [[65504.0; 3], [65504.0; 3], [0.0, 65504.0, 0.0]]);
}

/// The GT tone curve at GT7's parameters matches Polyphony's sample
/// implementation, compiled unmodified: 0 below 0, a toe below the mid
/// point 0.538, the identity from there to 0.444 of the 2.5 peak, 1.11, then
/// a shoulder towards 2.963, 1.19 times the peak.
#[test]
fn gt7_curve_matches_polyphonys_sample_implementation() {
    let Some(context) = gpu_context() else {
        return;
    };
    let reference = [
        (-1.0, 0.0),
        (0.0, 0.0),
        (0.05, 0.026298122),
        (0.2, 0.1666931),
        (0.4, 0.39478737),
        (0.538, 0.538),
        (0.8, 0.8),
        (1.0, 1.0),
        (1.1, 1.1),
        (1.11, 1.1099999),
        (1.2, 1.1978496),
        (1.5, 1.4616987),
        (2.0, 1.8167689),
        (2.5, 2.0878806),
        (3.0, 2.2948864),
        (4.0, 2.5736287),
        (8.0, 2.9183128),
        (100.0, 2.9633331),
    ];
    let inputs = reference.map(|(frame_buffer, _)| [frame_buffer, 0.0, 0.0, 0.0]);
    let results = evaluate(&context, "vec4<f32>(gt7_curve(input.x))", &inputs);

    for ((frame_buffer, expected), result) in reference.into_iter().zip(results) {
        let case = format_args!("the GT curve at {frame_buffer}");
        assert_close(result[0], expected, GT7_TOLERANCE, case);
    }
}

/// PQ's encoding is SMPTE ST 2084's inverse EOTF, evaluated in f64, of
/// frame-buffer values of 100 cd/m² each: 100 cd/m² encodes to 0.508 and
/// 10,000 to 1. Decoding returns each value, PQ's 10,000 cd/m² range
/// spanning the curve's dark end to past GT7's paper white.
#[test]
fn pq_encodes_as_st_2084_and_decodes_back() {
    let Some(context) = gpu_context() else {
        return;
    };
    let frame_buffer_values = [0.0, 0.001, 0.05, 0.25, 1.0, 2.5, 10.0, 100.0];
    let inputs = frame_buffer_values.map(|value| [value, value, value, 0.0]);
    let encoded = evaluate(&context, "vec4<f32>(pq_encode(input.xyz), 0.0)", &inputs);
    let decoded = evaluate(
        &context,
        "vec4<f32>(pq_decode(pq_encode(input.xyz)), 0.0)",
        &inputs,
    );

    let (m1, m2) = (2610.0 / 16384.0, 2523.0 / 4096.0 * 128.0);
    let (c1, c2, c3) = (
        3424.0 / 4096.0,
        2413.0 / 4096.0 * 32.0,
        2392.0 / 4096.0 * 32.0,
    );
    for ((value, encoded), decoded) in frame_buffer_values.into_iter().zip(&encoded).zip(decoded) {
        let normalized = (f64::from(value) * 100.0 / 10000.0).powf(m1);
        let expected = ((c1 + c2 * normalized) / (1.0 + c3 * normalized)).powf(m2);
        let case = format_args!("PQ of {value}");
        assert_close(encoded[0], expected, GT7_TOLERANCE, case);
        let case = format_args!("PQ decoding {value}");
        assert_close(
            decoded[0],
            value.into(),
            GT7_TOLERANCE * f64::from(value).max(1.0),
            case,
        );
    }
    assert_close(encoded[4][0], 0.508, 1e-3, format_args!("PQ of 100 cd/m²"));
    assert_close(
        encoded[7][0],
        1.0,
        GT7_TOLERANCE,
        format_args!("PQ of 10,000 cd/m²"),
    );
}

/// ICtCp (ITU-R BT.2100-3, Table 7 [ref:bt2100-ictcp]) puts a grey's PQ-encoded
/// value in I and none in Ct and Cp, its chroma; converting a colour there and
/// back returns it, within the sample's six-digit inverse matrix.
#[test]
fn ictcp_holds_a_grey_in_its_intensity_and_converts_back() {
    let Some(context) = gpu_context() else {
        return;
    };
    let colors = [
        [0.5, 0.5, 0.5],
        [2.5, 2.5, 2.5],
        [1.0, 0.25, 0.05],
        [0.02, 0.6, 0.3],
        [3.0, 2.0, 8.0],
    ];
    let inputs = colors.map(|[red, green, blue]| [red, green, blue, 0.0]);
    let ictcp = evaluate(
        &context,
        "vec4<f32>(rec2020_to_ictcp(input.xyz), 0.0)",
        &inputs,
    );
    let encoded = evaluate(&context, "vec4<f32>(pq_encode(input.xyz), 0.0)", &inputs);
    let round_trip = evaluate(
        &context,
        "vec4<f32>(ictcp_to_rec2020(rec2020_to_ictcp(input.xyz)), 0.0)",
        &inputs,
    );

    for (index, color) in colors.into_iter().enumerate() {
        if color[0] == color[1] && color[1] == color[2] {
            let case = format_args!("ICtCp of the grey {color:?}");
            assert_close(
                ictcp[index][0],
                encoded[index][0].into(),
                GT7_TOLERANCE,
                case,
            );
            assert_close(ictcp[index][1], 0.0, GT7_TOLERANCE, case);
            assert_close(ictcp[index][2], 0.0, GT7_TOLERANCE, case);
        }
        for channel in 0..3 {
            let expected = f64::from(color[channel]);
            let case = format_args!("channel {channel} of {color:?} through ICtCp");
            assert_close(
                round_trip[index][channel],
                expected,
                1e-3 * expected.max(1.0),
                case,
            );
        }
    }
}

/// GT7's SDR tone mapping matches outputs of Polyphony's sample
/// implementation, gt7_tone_mapping.cpp 1.0 (MIT; SHA-256 94df7e7e…9822),
/// compiled unmodified with clang++ -O0 -ffp-contract=off on the M4 Pro and
/// run on linear Rec. 2020 frame-buffer inputs: the sample's own three;
/// greys through the toe, the linear section from 0.538, the shoulder from
/// 1.11 and past the paper white of 2.5, which saturates; and saturated
/// colours from dim to far past it, which whiten through the ICtCp blend,
/// red and green but not blue.
#[test]
fn gt7_tonemap_matches_polyphonys_sample_implementation() {
    let Some(context) = gpu_context() else {
        return;
    };
    let inputs: Vec<[f32; 4]> = GT7_REFERENCE
        .iter()
        .map(|&([red, green, blue], _)| [red, green, blue, 0.0])
        .collect();
    let results = evaluate(&context, "vec4<f32>(gt7_tonemap(input.xyz), 0.0)", &inputs);

    for ((input, expected), result) in GT7_REFERENCE.iter().zip(results) {
        for channel in 0..3 {
            let case = format_args!("channel {channel} of GT7{input:?}");
            assert_close(
                result[channel],
                expected[channel].into(),
                GT7_TOLERANCE,
                case,
            );
        }
    }
}

/// `tonemap` reads exposed 1 as GT7's paper white, and converts Rec. 709 to
/// Rec. 2020 and back around GT7, clipping into [0, 1]. Against the sample
/// implementation's GT7 between the conversions in f64, from BT.709-6's and
/// BT.2020-2's primaries [ref:bt709-6-primaries] [ref:bt2020-2-primaries]:
/// greys, which both conversions keep grey, pass the linear section, exposed
/// 0.2152 to 0.444, unchanged, so mid-tones keep their exposure; Rec. 709's
/// primaries stay pure but green, which picks up red; past the paper white,
/// colours whiten.
#[test]
fn tonemap_is_gt7_between_rec_709_and_rec_2020_with_exposed_1_at_paper_white() {
    let Some(context) = gpu_context() else {
        return;
    };
    let inputs: Vec<[f32; 4]> = TONEMAP_REFERENCE
        .iter()
        .map(|&([red, green, blue], _)| [red, green, blue, 0.0])
        .collect();
    let results = evaluate(&context, "vec4<f32>(tonemap(input.xyz), 0.0)", &inputs);

    for ((input, expected), result) in TONEMAP_REFERENCE.iter().zip(results) {
        for channel in 0..3 {
            let case = format_args!("channel {channel} of tonemap{input:?}");
            assert_close(
                result[channel],
                expected[channel].into(),
                GT7_TOLERANCE,
                case,
            );
        }
    }
}

/// Linear Rec. 2020 frame-buffer inputs and their outputs from Polyphony's
/// gt7_tone_mapping.cpp 1.0, initialised for SDR, printed to nine digits by
/// a harness that includes the file unmodified.
const GT7_REFERENCE: [([f32; 3], [f32; 3]); 26] = [
    ([0.5, 1.23, 0.75], [0.1996225, 0.49070296, 0.29956776]),
    ([12.3, 34.3, 56.9], [1.0, 1.0, 1.0]),
    ([1504.7, 64.51, 0.5], [1.0, 1.0, 0.7387512]),
    ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
    (
        [0.01, 0.01, 0.01],
        [0.0013133113, 0.0013132402, 0.0013132499],
    ),
    ([0.1, 0.1, 0.1], [0.02633636, 0.026336258, 0.026336217]),
    ([0.3, 0.3, 0.3], [0.11250466, 0.11250422, 0.11250404]),
    ([0.538, 0.538, 0.538], [0.2152061, 0.2152053, 0.21520495]),
    ([0.8, 0.8, 0.8], [0.3200031, 0.3200019, 0.32000136]),
    ([1.11, 1.11, 1.11], [0.4440012, 0.4439995, 0.44399878]),
    ([1.5, 1.5, 1.5], [0.58467156, 0.5846693, 0.5846684]),
    ([2.5, 2.5, 2.5], [0.8351579, 0.8351547, 0.8351534]),
    ([5.0, 5.0, 5.0], [1.0, 1.0, 1.0]),
    ([50.0, 50.0, 50.0], [1.0, 1.0, 1.0]),
    ([0.2, 0.0, 0.0], [0.06769287, 0.0, 0.0]),
    ([2.0, 0.0, 0.0], [0.7297, 0.0, 0.0]),
    ([20.0, 0.0, 0.0], [0.6966279, 0.17642365, 0.15613371]),
    ([0.0, 0.2, 0.0], [0.0, 0.067029335, 0.0]),
    ([0.0, 2.0, 0.0], [0.0, 0.72763175, 0.0]),
    ([0.0, 20.0, 0.0], [0.4372026, 0.91132194, 0.4371986]),
    ([0.0, 0.0, 0.2], [0.0, 0.0, 0.069220796]),
    ([0.0, 0.0, 2.0], [0.0, 0.0, 0.73478824]),
    ([0.0, 0.0, 20.0], [0.0, 0.0, 1.0]),
    ([1.0, 0.6, 0.1], [0.3988926, 0.23934059, 0.034391593]),
    ([3.0, 2.4, 0.2], [0.9566267, 0.79841584, 0.06546863]),
    ([0.05, 0.3, 0.9], [0.014997541, 0.11209259, 0.3466799]),
];

/// Exposed Rec. 709 inputs and the display colours the sample's GT7 gives
/// them between Rec. 709 and Rec. 2020, with the conversions in f64 from
/// the standards' primaries, exposed 1 read as 2.5, and the result clipped
/// to [0, 1].
const TONEMAP_REFERENCE: [([f32; 3], [f32; 3]); 14] = [
    ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
    (
        [0.0864, 0.0864, 0.0864],
        [0.073816635, 0.073816106, 0.073816],
    ),
    ([0.2152, 0.2152, 0.2152], [0.21520665, 0.2152052, 0.2152049]),
    ([0.32, 0.32, 0.32], [0.32000393, 0.32000175, 0.32000127]),
    ([0.6, 0.6, 0.6], [0.5846731, 0.584669, 0.5846683]),
    ([1.0, 1.0, 1.0], [0.8351601, 0.83515435, 0.83515316]),
    ([4.0, 4.0, 4.0], [1.0, 1.0, 1.0]),
    ([0.1, 0.2, 0.3], [0.09670808, 0.19929703, 0.29879954]),
    ([1.0, 0.0, 0.0], [0.9479998, 0.0, 0.0]),
    ([0.0, 1.0, 0.0], [0.027424455, 0.8751596, 0.0]),
    ([0.0, 0.0, 1.0], [0.0, 0.0, 0.85535264]),
    ([8.0, 0.0, 0.0], [1.0, 0.50304955, 0.23320603]),
    ([0.5, 0.25, 0.125], [0.499791, 0.24980466, 0.1230321]),
    ([3.0, 2.0, 0.5], [1.0, 1.0, 0.83223754]),
];

/// Largest difference allowed between GT7 on the GPU and the sample's
/// outputs, on a scale where 1 is display white. On the M4 Pro, GT7 lands
/// within 7e-5 of them, PQ's exponents amplifying f32 rounding, and
/// `tonemap` within 2.3e-4, where its conversion back to Rec. 709 subtracts
/// near-equal terms for saturated green: a fifth of an 8-bit step.
const GT7_TOLERANCE: f64 = 3e-4;

/// IEC 61966-2-1's sRGB encoding, as the Khronos Data Format Specification
/// restates it [ref:khronos-dfs-srgb], evaluated in f64: 12.92 x up to 0.0031308,
/// then 1.055 x^(1/2.4) − 0.055, so 0 encodes to 0 and 1 to 1, across the
/// knee and through the mid-tones; values outside [0, 1] clamp first.
#[test]
fn srgb_encode_follows_iec_61966_2_1() {
    let Some(context) = gpu_context() else {
        return;
    };
    let values = [
        -0.5, 0.0, 0.001, 0.0031308, 0.0032, 0.05, 0.18, 0.5, 0.9, 1.0, 1.5,
    ];
    let inputs = values.map(|value| [value, value, value, 0.0]);
    let results = evaluate(&context, "vec4<f32>(srgb_encode(input.xyz), 0.0)", &inputs);

    for (value, result) in values.into_iter().zip(results) {
        let linear = f64::from(value).clamp(0.0, 1.0);
        let expected = if linear <= 0.0031308 {
            12.92 * linear
        } else {
            1.055 * linear.powf(1.0 / 2.4) - 0.055
        };
        let case = format_args!("the sRGB encoding of {value}");
        assert_close(result[0], expected, SRGB_TOLERANCE, case);
    }
}

/// Largest difference allowed between the shader's sRGB encoding and the
/// standard's in f64: a thousandth of an 8-bit step.
const SRGB_TOLERANCE: f64 = 4e-6;

/// The dither's noise from each of the mask's 256 levels, k/255 as the
/// texture holds it, is the inverse of the CDF of the triangular
/// distribution on (−1, 1) at the level's bin centre u = (k + ½)/256,
/// −1 + √(2u) below ½ and 1 − √(2 − 2u) from it, in f64: odd about ½, and
/// over the 256 levels, which the mask holds 16 times each, its mean is 0
/// and its variance within 1e-4 of the distribution's 1/6.
#[test]
fn dither_noise_is_the_triangular_inverse_cdf_of_each_mask_level() {
    let Some(context) = gpu_context() else {
        return;
    };
    let inputs: Vec<[f32; 4]> = (0..256)
        .map(|level| [level as f32 / 255.0, 0.0, 0.0, 0.0])
        .collect();
    let results = evaluate(&context, "vec4<f32>(dither_noise(input.x))", &inputs);

    let noise: Vec<f64> = results.iter().map(|result| f64::from(result[0])).collect();
    for (level, &noise) in noise.iter().enumerate() {
        let uniform = (level as f64 + 0.5) / 256.0;
        let expected = if uniform < 0.5 {
            -1.0 + (2.0 * uniform).sqrt()
        } else {
            1.0 - (2.0 - 2.0 * uniform).sqrt()
        };
        let case = format_args!("the dither noise of level {level}");
        assert_close(noise as f32, expected, TOLERANCE.into(), case);
    }
    let mean = noise.iter().sum::<f64>() / noise.len() as f64;
    let variance = noise.iter().map(|value| value * value).sum::<f64>() / noise.len() as f64;
    assert!(mean.abs() <= 1e-6, "mean {mean}");
    assert!((variance - 1.0 / 6.0).abs() <= 1e-4, "variance {variance}");
}

/// Linear sRGB reflectance of gold at normal incidence, F0 (Hoffman 2013,
/// Table 1 [ref:hoffman-2013-f0]), as the pbr-sweep scene uses it: each
/// channel a different base colour.
const GOLD: [f32; 3] = [1.0, 0.71, 0.29];

/// Largest error allowed between a result and its f64 reference, relative to
/// the reference, or to a scale the test names. Results land within 2e-6 of
/// theirs on the M4 Pro.
const RELATIVE_TOLERANCE: f64 = 4e-6;

/// The same for shading from unit vectors: f32 rounds each direction and
/// each cosine between them by up to about 1e-7, which D, steep near its
/// peak at the roughness floor, turns into up to 1.1e-5 of the result on the
/// M4 Pro.
const DIRECTION_TOLERANCE: f64 = 5e-5;

/// Asserts that `actual` is finite and within `tolerance` of `expected`, so
/// exactly where `tolerance` is 0.
fn assert_close(actual: f32, expected: f64, tolerance: f64, case: std::fmt::Arguments) {
    let error = (f64::from(actual) - expected).abs();
    assert!(
        actual.is_finite() && error <= tolerance.abs(),
        "{case} is {actual}, expected {expected}"
    );
}

/// The unit direction `polar` degrees from +Z, turned `azimuth` degrees about
/// it from +X.
fn direction(polar: f64, azimuth: f64) -> glam::DVec3 {
    let (polar, azimuth) = (polar.to_radians(), azimuth.to_radians());
    glam::DVec3::new(
        polar.sin() * azimuth.cos(),
        polar.sin() * azimuth.sin(),
        polar.cos(),
    )
}

/// N·L, N·V, N·H and V·H for unit `normal`, `view` and `light` directions,
/// H halfway between L and V, or zero where they are opposite, as the shader
/// leaves it.
fn cosines(normal: glam::DVec3, view: glam::DVec3, light: glam::DVec3) -> [f64; 4] {
    let half = (light + view).try_normalize().unwrap_or(glam::DVec3::ZERO);
    [
        normal.dot(light),
        normal.dot(view),
        normal.dot(half),
        view.dot(half),
    ]
}

/// Appendix B's D, α² χ⁺(N·H) / (π ((N·H)² (α² − 1) + 1)²), as it writes it
/// (glTF 2.0, Khronos, CC BY 4.0).
fn reference_distribution(normal_dot_half: f64, alpha: f64) -> f64 {
    let alpha_squared = alpha * alpha;
    let denominator = normal_dot_half * normal_dot_half * (alpha_squared - 1.0) + 1.0;
    let heaviside = if normal_dot_half > 0.0 { 1.0 } else { 0.0 };
    alpha_squared * heaviside / (std::f64::consts::PI * denominator * denominator)
}

/// Appendix B's V from N·L, N·V, V·H (for H·L and H·V) and α, 0 where its
/// denominator is, as the shader makes it.
fn reference_visibility(
    [normal_dot_light, normal_dot_view, view_dot_half, alpha]: [f64; 4],
) -> f64 {
    let alpha_squared = alpha * alpha;
    let root = |cosine: f64| (alpha_squared + (1.0 - alpha_squared) * cosine * cosine).sqrt();
    let denominator = 2.0
        * (normal_dot_view.abs() * root(normal_dot_light)
            + normal_dot_light.abs() * root(normal_dot_view));
    if view_dot_half > 0.0 && denominator > 0.0 {
        1.0 / denominator
    } else {
        0.0
    }
}

/// Appendix B's "full code" (glTF 2.0, Khronos, CC BY 4.0
/// [ref:gltf-2.0-metal-and-dielectric-brdf]): the BRDF of a
/// surface of `base_color`, `metallic` and `roughness`, from the cosines
/// N·L, N·V, N·H and V·H.
fn reference_brdf(
    cosines: [f64; 4],
    base_color: [f64; 3],
    metallic: f64,
    roughness: f64,
) -> [f64; 3] {
    let [
        normal_dot_light,
        normal_dot_view,
        normal_dot_half,
        view_dot_half,
    ] = cosines;
    let alpha = roughness * roughness;
    let specular_brdf = reference_distribution(normal_dot_half, alpha)
        * reference_visibility([normal_dot_light, normal_dot_view, view_dot_half, alpha]);
    let fresnel_weight = (1.0 - view_dot_half.abs()).powi(5);
    let mix = |base: f64, layer: f64, weight: f64| base * (1.0 - weight) + layer * weight;
    base_color.map(|base_color| {
        let diffuse_brdf = base_color / std::f64::consts::PI;
        let dielectric_fresnel = 0.04 + (1.0 - 0.04) * fresnel_weight;
        let dielectric_brdf = mix(diffuse_brdf, specular_brdf, dielectric_fresnel);
        let metal_fresnel = base_color + (1.0 - base_color) * fresnel_weight;
        let metal_brdf = metal_fresnel * specular_brdf;
        mix(dielectric_brdf, metal_brdf, metallic)
    })
}

/// A cooked normal Texture stores a unit normal's x and y, each mapped to
/// [0, 1]; the shader rebuilds z = √(1 − x² − y²), the cosine between the
/// normal and the surface's, which falls to 0 at grazing slopes, multiplies
/// x and y by the Material's normal scale, and normalises once the tangent
/// frame has turned the result into World space: for a texture of unit
/// normals, glTF 2.0's normalize((sample · 2 − 1) ⊙ ⟨scale, scale, 1⟩)
/// ("Normal Texture" [ref:gltf-2.0-normal-texture-scale]), evaluated here in
/// f64 on a lattice of encodings 1/256 apart, whose x, y and squares f32
/// holds exactly, at scales of 1/2, 1 and 2. Past the unit circle, where
/// BC5's error can carry an encoding, z is 0, not NaN, and a scale of 0
/// gives the surface's normal at every texel, those included.
#[test]
fn tangent_space_normal_scales_x_and_y_and_normalises_as_gltf_writes_it() {
    let Some(context) = gpu_context() else {
        return;
    };
    let steps: Vec<f64> = (0..=1024)
        .step_by(4)
        .map(|step| f64::from(step) / 1024.0)
        .collect();
    let lattice: Vec<[f32; 2]> = steps
        .iter()
        .flat_map(|&x| steps.iter().map(move |&y| [x as f32, y as f32]))
        .collect();
    let inputs: Vec<[f32; 4]> = [0.0, 0.5, 1.0, 2.0]
        .into_iter()
        .flat_map(|scale| lattice.iter().map(move |&[x, y]| [x, y, scale, 0.0]))
        .collect();
    let results = evaluate(
        &context,
        "vec4<f32>(normalize(tangent_space_normal(input.xy, input.z)), 0.0)",
        &inputs,
    );

    let mut without_z = 0;
    for (input, result) in inputs.iter().zip(results) {
        let [x, y] = [input[0], input[1]].map(|encoded| f64::from(encoded) * 2.0 - 1.0);
        let scale = f64::from(input[2]);
        let z = (1.0 - x * x - y * y).max(0.0).sqrt();
        let scaled = glam::DVec3::new(x * scale, y * scale, z);
        let expected = scaled.try_normalize().unwrap_or(glam::DVec3::Z);
        for (axis, (actual, expected)) in result.iter().zip(expected.to_array()).enumerate() {
            assert_close(
                *actual,
                expected,
                f64::from(TOLERANCE),
                format_args!(
                    "encoded ({}, {}) at scale {scale}, axis {axis}: {result:?}, expected {expected}",
                    input[0], input[1]
                ),
            );
        }
        without_z += usize::from(z == 0.0);
    }
    // Encodings at or past the unit circle are among the cases.
    assert!(without_z > 0);
}

/// A surface decodes as its Material's factors times its texels (glTF's
/// metallic-roughness Material): base colour and its alpha, metallic from the
/// cooked metallic-roughness texel's green and roughness from its red,
/// occlusion and emissive; a roughness below the floor rises to it. The
/// values are binary fractions, so each product is exact.
#[test]
fn decode_surface_scales_each_texel_by_its_factor_and_floors_roughness() {
    let Some(context) = gpu_context() else {
        return;
    };
    // The factors (base colour, metallic and roughness, emissive), then the
    // texels (albedo; roughness, metallic and occlusion; emissive).
    let material = [
        [0.5, 0.25, 1.0, 0.75],
        [0.5, 0.75, 0.0, 0.0],
        [2.0, 1.0, 0.5, 1.0],
    ];
    let texels = |roughness| {
        [
            [0.5, 1.0, 0.25, 0.5],
            [roughness, 0.5, 0.75, 0.0],
            [0.25, 0.5, 1.0, 0.0],
        ]
    };
    let cases = [0.5, 1.0 / 64.0].map(|roughness| {
        let [albedo, metallic_roughness, emissive] = texels(roughness);
        [
            material[0],
            material[1],
            material[2],
            albedo,
            metallic_roughness,
            emissive,
        ]
    });
    let decode = |expression: &str| {
        let surface = "decode_surface(MaterialData(inputs[0], inputs[1], inputs[2], vec4<u32>(0u), \
                       vec4<u32>(0u)), inputs[3], inputs[4].xy, inputs[4].z, inputs[5].xyz)";
        evaluate_cases(
            &context,
            &expression.replace("{surface}", surface),
            &cases,
            &[],
        )
    };
    let colours = decode("vec4<f32>({surface}.base_color, {surface}.opacity)");
    let scalars =
        decode("vec4<f32>({surface}.metallic, {surface}.roughness, {surface}.occlusion, 0.0)");
    let emissives = decode("vec4<f32>({surface}.emissive, 0.0)");

    let floor = minimum_roughness(&context);
    assert_eq!(colours, [[0.25, 0.25, 0.25, 0.375]; 2]);
    assert_eq!(
        scalars,
        [[0.25, 0.375, 0.75, 0.0], [0.25, floor, 0.75, 0.0]]
    );
    assert_eq!(emissives, [[0.5, 0.5, 0.5, 0.0]; 2]);
}

/// WGSL for an opaque Surface with no emissive, from WGSL expressions for
/// its base colour, metallic, roughness and occlusion.
fn surface(base_color: &str, metallic: &str, roughness: &str, occlusion: &str) -> String {
    format!("Surface({base_color}, 1.0, {metallic}, {roughness}, {occlusion}, vec3<f32>(0.0))")
}

/// The shader's roughness floor, `MINIMUM_ROUGHNESS` (shading.wgsl).
fn minimum_roughness(context: &GpuContext) -> f32 {
    evaluate(context, "vec4<f32>(MINIMUM_ROUGHNESS)", &[[0.0; 4]])[0][0]
}

/// Vertices as the cooker writes them, each QTangent from its encoder, laid
/// out as the vertex Arena holds them and read on the GPU by index: each
/// position and UV decodes exact, and each QTangent, unpacked from Snorm16
/// (each component over 32,767, clamped at −1), to the normal, tangent and
/// bitangent sign it was cooked from, both signs and frames near the half
/// turns whose w rounds towards zero included. A rotated axis is a product
/// of two quaternions, each component off by at most half a Snorm16 step,
/// plus the nudge that keeps w a whole step: within 4 steps.
#[cfg(feature = "cook")]
#[test]
fn cooked_vertices_decode_to_their_position_uv_and_tangent_frame() {
    use crate::cook::mesh::{qtangent, tests::sample_rotations};

    let Some(context) = gpu_context() else {
        return;
    };
    let step = 1.0 / f32::from(i16::MAX);
    let mut tangent_frames = Vec::new();
    for rotation in sample_rotations() {
        for bitangent_sign in [1.0, -1.0] {
            let (normal, tangent) = (rotation * glam::Vec3::Z, rotation * glam::Vec3::X);
            tangent_frames.push((normal, tangent, bitangent_sign));
        }
    }
    // Every position and UV component a different value, so a read at the
    // wrong offset or stride returns another.
    let vertices: Vec<CookedVertex> = tangent_frames
        .iter()
        .enumerate()
        .map(|(index, &(normal, tangent, bitangent_sign))| {
            let first = 8.0 * index as f32;
            CookedVertex {
                position: [first, first + 1.0, first + 2.0],
                uv: [first + 3.0, first + 4.0],
                qtangent: qtangent(normal.into(), tangent.extend(bitangent_sign).into()),
            }
        })
        .collect();
    // Read last to first, so no case reads the vertex at its own index.
    let inputs: Vec<[[f32; 4]; 1]> = (0..vertices.len())
        .rev()
        .map(|index| [[index as f32, 0.0, 0.0, 0.0]])
        .collect();
    // `{vertex}` in an expression stands for the decoded vertex.
    let decode = |expression: &str| {
        let expression =
            expression.replace("{vertex}", "decode_vertex(test_vertices[u32(input.x)])");
        evaluate_cases(&context, &expression, &inputs, &vertices)
    };
    let positions = decode("vec4<f32>({vertex}.position, 0.0)");
    let uvs = decode("vec4<f32>({vertex}.uv, 0.0, 0.0)");
    let normals_and_signs =
        decode("vec4<f32>({vertex}.tangent_frame.normal, {vertex}.tangent_frame.bitangent_sign)");
    let tangents = decode("vec4<f32>({vertex}.tangent_frame.tangent, 0.0)");

    let decoded = positions
        .into_iter()
        .zip(uvs)
        .zip(normals_and_signs.into_iter().zip(tangents));
    for ([input], ((position, uv), (normal_and_sign, tangent))) in inputs.iter().zip(decoded) {
        let vertex_index = input[0] as usize;
        let vertex = &vertices[vertex_index];
        let (normal, tangent_expected, bitangent_sign) = tangent_frames[vertex_index];
        let decoded_normal = glam::Vec3::from_slice(&normal_and_sign);
        let decoded_tangent = glam::Vec3::from_slice(&tangent);
        assert!(
            position[..3] == vertex.position
                && uv[..2] == vertex.uv
                && normal_and_sign[3] == bitangent_sign
                && decoded_normal.abs_diff_eq(normal, 4.0 * step)
                && decoded_tangent.abs_diff_eq(tangent_expected, 4.0 * step),
            "{vertex:?}: normal {normal}, tangent {tangent_expected}, bitangent sign \
             {bitangent_sign} decode to {position:?}, {uv:?}, {decoded_normal}, \
             {decoded_tangent}, {}",
            normal_and_sign[3]
        );
    }
}

/// A packed index holds its Meshlet record in its top 26 bits and the vertex
/// within that record's Meshlet in its low six (ADR 0019): packing and
/// unpacking keep both at each bit boundary, the top bit and alternating
/// bits included, against indices written out by hand. f32 holds a 16-bit
/// half exactly, so each index crosses the harness as two.
#[test]
fn a_packed_index_holds_its_meshlet_record_above_six_bits_of_vertex() {
    let Some(context) = gpu_context() else {
        return;
    };
    let cases: [(u32, u32, u32); 9] = [
        (0, 0, 0x0000_0000),
        (0, 63, 0x0000_003F),
        (1, 0, 0x0000_0040),
        (1, 63, 0x0000_007F),
        (0x0200_0000, 0, 0x8000_0000),
        (0x03FF_FFFF, 0, 0xFFFF_FFC0),
        (0x03FF_FFFF, 63, 0xFFFF_FFFF),
        (0x02AA_AAAA, 0x2A, 0xAAAA_AAAA),
        (0x0155_5555, 0x15, 0x5555_5555),
    ];
    let halves = |value: u32| [(value >> 16) as f32, (value & 0xFFFF) as f32];
    // `{index}` in an expression stands for the index the case packs.
    let index = "((u32(input.x) << 16u) | u32(input.y))";
    let unpacked = evaluate(
        &context,
        &"vec4<f32>(f32(unpack_index({index}).record >> 16u), \
          f32(unpack_index({index}).record & 0xFFFFu), f32(unpack_index({index}).vertex), 0.0)"
            .replace("{index}", index),
        &cases.map(|(_, _, packed)| {
            let [high, low] = halves(packed);
            [high, low, 0.0, 0.0]
        }),
    );
    let packed = evaluate(
        &context,
        "vec4<f32>(f32(pack_index((u32(input.x) << 16u) | u32(input.y), u32(input.z)) >> 16u), \
         f32(pack_index((u32(input.x) << 16u) | u32(input.y), u32(input.z)) & 0xFFFFu), 0.0, 0.0)",
        &cases.map(|(record, vertex, _)| {
            let [high, low] = halves(record);
            [high, low, vertex as f32, 0.0]
        }),
    );

    for ((record, vertex, index), (unpacked, packed)) in
        cases.into_iter().zip(unpacked.into_iter().zip(packed))
    {
        let [high, low] = halves(record);
        let [index_high, index_low] = halves(index);
        assert_eq!(
            unpacked,
            [high, low, vertex as f32, 0.0],
            "{index:#010x} unpacks to the wrong record or vertex"
        );
        assert_eq!(
            packed,
            [index_high, index_low, 0.0, 0.0],
            "record {record:#x}, vertex {vertex} packs to the wrong index"
        );
    }
}

/// The shadow mask pass rebuilds the point a depth target shows through a
/// pixel from its depth and the camera's view_proj. Through a camera turned
/// off every axis, at points across a 1440p target, its corners and sample
/// offsets included, and depths from the near plane to 10 km, each lands
/// where the f64 inverse of that view_proj puts it, within 2^-20 of its
/// largest coordinate: sixteen times inside the 2^-16 a shadow ray's origin
/// leaves its surface by (`shadow_mask.wgsl`).
#[test]
fn position_at_rebuilds_the_point_a_depth_shows_through_a_pixel() {
    let Some(context) = gpu_context() else {
        return;
    };
    let size = [2560.0_f32, 1440.0];
    let view = glam::Mat4::look_to_rh(
        glam::Vec3::ZERO,
        glam::vec3(0.3, -0.5, -0.8).normalize(),
        glam::Vec3::Y,
    );
    let view_proj = crate::camera::projection_matrix(size[0] / size[1]) * view;
    let columns = view_proj.to_cols_array_2d();
    // The near plane's distance over the depth along the view axis.
    let near = f64::from(columns[3][2]);
    let points = [0.0, 0.375, 640.5, 1279.875, 2559.625, 2560.0]
        .into_iter()
        .flat_map(|x| [0.0, 0.125, 720.5, 1439.875, 1440.0].map(|y| [x, y]));
    let mut cases = Vec::new();
    for [x, y] in points {
        for distance in [0.1, 0.35, 2.0, 40.0, 900.0, 10_000.0] {
            let depth = (near / distance) as f32;
            cases.push([
                columns[0],
                columns[1],
                columns[2],
                columns[3],
                [x, y, depth, 0.0],
                [size[0], size[1], 0.0, 0.0],
            ]);
        }
    }
    let results = evaluate_cases(
        &context,
        "vec4<f32>(position_at(mat4x4<f32>(inputs[0], inputs[1], inputs[2], inputs[3]), \
         inputs[4].xy, inputs[4].z, inputs[5].xy), 0.0)",
        &cases,
        &[],
    );

    let inverse = view_proj.as_dmat4().inverse();
    for (case, result) in cases.iter().zip(results) {
        let [x, y, depth, _] = case[4].map(f64::from);
        let device = glam::dvec4(
            x / f64::from(size[0]) * 2.0 - 1.0,
            1.0 - y / f64::from(size[1]) * 2.0,
            depth,
            1.0,
        );
        let homogeneous = inverse * device;
        let expected = homogeneous.truncate() / homogeneous.w;
        let gpu = glam::Vec3::from_slice(&result).as_dvec3();
        let bound = expected.abs().max_element() * 2f64.powi(-20);
        assert!(
            gpu.distance(expected) <= bound,
            "({x}, {y}) at depth {depth}: rebuilt {gpu}, {expected} by the f64 inverse"
        );
    }
}

/// ADR 0014's precision bar: 0.25 px at 1440p and a 45° vertical field of
/// view, for a point 0.3 m from the camera, is 0.043 mm.
const PRECISION_BAR: f64 = 4.3e-5;

/// A point 0.3 m from the camera, anywhere within 100 km of the origin, is
/// composed from its Instance's anchor and offset, the camera's, and its f32
/// position relative to the Instance; it lands within the precision bar of
/// its f64 position relative to the camera. The GPU runs one evaluation
/// order; every other order the compiler may choose is summed on the CPU.
/// The point's position relative to its pivot arrives rotated and rounded,
/// so error from rotating it, fused into the sum or not, is outside this
/// oracle (ADR 0014 measures it separately).
/// The same points composed from f32 World positions miss the bar.
#[test]
fn camera_relative_position_meets_the_precision_bar_within_100_km() {
    let Some(context) = gpu_context() else {
        return;
    };
    // Cell corners, points either side of them, and the 100 km edge.
    let coordinates = [
        -100_000.0,
        -54_321.987_654,
        -64.000_01,
        0.0,
        0.3,
        63.999_99,
        12_345.678_9,
        99_999.999,
    ];
    let mut directions = Vec::new();
    for x in [-1.0, 0.0, 1.0] {
        for y in [-1.0, 0.0, 1.0] {
            for z in [-1.0, 0.0, 1.0] {
                if [x, y, z] != [0.0; 3] {
                    directions.push(glam::DVec3::new(x, y, z).normalize());
                }
            }
        }
    }
    // Vertices up to 128 m from their pivot, the most a cooked mesh holds.
    let local_positions = [
        glam::Vec3::ZERO,
        glam::Vec3::new(100.0, -50.25, 20.125),
        glam::Vec3::new(-127.9, 3.3, 64.7),
    ];

    let mut cases = Vec::new();
    for x in coordinates {
        for y in coordinates {
            for z in coordinates {
                let camera = glam::DVec3::new(x, y, z);
                for direction in &directions {
                    for local_position in local_positions {
                        cases.push(Case::new(camera, direction * 0.3, local_position));
                    }
                }
            }
        }
    }
    let inputs: Vec<[[f32; 4]; 5]> = cases.iter().map(Case::test_vectors).collect();
    let results = evaluate_cases(
        &context,
        "vec4<f32>(camera_relative_position(vec3<i32>(inputs[0].xyz), inputs[1].xyz, \
         vec3<i32>(inputs[2].xyz), inputs[3].xyz, inputs[4].xyz), 0.0)",
        &inputs,
        &[],
    );

    let mut world_space_error: f64 = 0.0;
    for (case, result) in cases.iter().zip(results) {
        let gpu = glam::Vec3::from_slice(&result).as_dvec3();
        let error = gpu.distance(case.camera_relative);
        assert!(
            error <= PRECISION_BAR,
            "{case:?}: GPU composes {gpu}, {error} m from the f64 reference"
        );
        let error = case.worst_error_in_any_order();
        assert!(
            error <= PRECISION_BAR,
            "{case:?}: some evaluation order lands {error} m from the f64 reference"
        );
        world_space_error = world_space_error.max(case.world_space_error());
    }
    assert!(
        world_space_error > PRECISION_BAR,
        "f32 World positions met the bar ({world_space_error} m), so the oracle cannot fail"
    );
}

/// A point at `camera + camera_relative`, drawn at `local_position` from its
/// Instance's pivot, both placed as the renderer places them.
#[derive(Debug)]
struct Case {
    camera: glam::DVec3,
    camera_relative: glam::DVec3,
    local_position: glam::Vec3,
    pivot: glam::DVec3,
}

impl Case {
    fn new(camera: glam::DVec3, camera_relative: glam::DVec3, local_position: glam::Vec3) -> Self {
        let pivot = camera + camera_relative - local_position.as_dvec3();
        Self {
            camera,
            camera_relative,
            local_position,
            pivot,
        }
    }

    /// Instance anchor, Instance offset, camera anchor, camera offset, local
    /// position; anchors are small integers, exact in f32.
    fn test_vectors(&self) -> [[f32; 4]; 5] {
        let (anchor, offset) = anchor_and_offset(self.pivot);
        let (camera_anchor, camera_offset) = anchor_and_offset(self.camera);
        let anchor = anchor.map(|cell| cell as f32);
        let camera_anchor = camera_anchor.map(|cell| cell as f32);
        [
            anchor,
            offset,
            camera_anchor,
            camera_offset,
            self.local_position.to_array(),
        ]
        .map(|vector| glam::Vec3::from_array(vector).extend(0.0).to_array())
    }

    /// The largest distance from the f64 reference of any order and grouping
    /// in which f32 can sum the composition's four terms, per axis.
    fn worst_error_in_any_order(&self) -> f64 {
        let (anchor, offset) = anchor_and_offset(self.pivot);
        let (camera_anchor, camera_offset) = anchor_and_offset(self.camera);
        let worst_per_axis = glam::DVec3::from_array(std::array::from_fn(|axis| {
            // Exact: the anchor difference times 64 is a small integer.
            let anchor_term = (anchor[axis] - camera_anchor[axis]) as f32 * ANCHOR_CELL_SIZE as f32;
            let terms = [
                anchor_term,
                offset[axis],
                -camera_offset[axis],
                self.local_position[axis],
            ];
            sums_in_every_order(&terms)
                .into_iter()
                .map(|sum| (f64::from(sum) - self.camera_relative[axis]).abs())
                .fold(0.0, f64::max)
        }));
        worst_per_axis.length()
    }

    /// The distance from the f64 reference of the point composed from f32
    /// World positions, as before anchors.
    fn world_space_error(&self) -> f64 {
        let composed = self.pivot.as_vec3() + self.local_position - self.camera.as_vec3();
        composed.as_dvec3().distance(self.camera_relative)
    }
}

/// Every f32 sum of `terms` a compiler free to reorder and regroup can
/// produce: each step adds any two remaining terms.
fn sums_in_every_order(terms: &[f32]) -> Vec<f32> {
    if terms.len() == 1 {
        return terms.to_vec();
    }
    let mut sums = Vec::new();
    for first in 0..terms.len() {
        for second in first + 1..terms.len() {
            let mut remaining: Vec<f32> = terms
                .iter()
                .enumerate()
                .filter(|&(index, _)| index != first && index != second)
                .map(|(_, &term)| term)
                .collect();
            remaining.push(terms[first] + terms[second]);
            sums.extend(sums_in_every_order(&remaining));
        }
    }
    sums
}

/// A headless context, or `None` after saying so on a machine without a GPU
/// adapter, where these tests skip.
fn gpu_context() -> Option<GpuContext> {
    match pollster::block_on(GpuContext::headless()) {
        Ok(context) => Some(context),
        Err(error) if error.is::<wgpu::RequestAdapterError>() => {
            // Not `eprintln!`, which libtest captures from passing tests.
            writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
            None
        }
        Err(error) => panic!("headless GPU context failed: {error:#}"),
    }
}

/// The compute kernel appended to the shared shader parts, so it calls the
/// functions the passes run. `EXPRESSION`, `VECTORS_PER_CASE` and
/// `WORKGROUP_SIZE` stand for the expression under test, the test vectors
/// each case holds and the constant below; `test_vertices` stands in for the
/// vertex Arena.
const KERNEL: &str = "
@group(0) @binding(0) var<storage, read> test_vectors: array<array<vec4<f32>, VECTORS_PER_CASE>>;
@group(0) @binding(1) var<storage, read_write> test_results: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> test_vertices: array<CookedVertex>;

@compute @workgroup_size(WORKGROUP_SIZE)
fn evaluate_test_vectors(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if index < arrayLength(&test_vectors) {
        let inputs = test_vectors[index];
        let input = inputs[0];
        test_results[index] = EXPRESSION;
    }
}
";

/// Invocations per workgroup.
const WORKGROUP_SIZE: u32 = 64;

/// Evaluates `expression`, WGSL of type `vec4<f32>` that reads its test
/// vector as `input` and may call any function of the shared parts, once per
/// element of `inputs` in one compute dispatch; returns the results in order.
fn evaluate(context: &GpuContext, expression: &str, inputs: &[[f32; 4]]) -> Vec<[f32; 4]> {
    evaluate_cases::<1>(context, expression, bytemuck::cast_slice(inputs), &[])
}

/// As `evaluate`, for cases of `VECTORS` test vectors each, which
/// `expression` reads as `inputs[0]` to `inputs[VECTORS - 1]`, and
/// `vertices`, if any, which it reads as `test_vertices`.
fn evaluate_cases<const VECTORS: usize>(
    context: &GpuContext,
    expression: &str,
    cases: &[[[f32; 4]; VECTORS]],
    vertices: &[CookedVertex],
) -> Vec<[f32; 4]> {
    let device = &context.device;
    let kernel = KERNEL
        .replace("WORKGROUP_SIZE", &WORKGROUP_SIZE.to_string())
        .replace("VECTORS_PER_CASE", &VECTORS.to_string())
        .replace("EXPRESSION", expression);
    let source = format!("{}\n{kernel}", shaders::SHARED.source());
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader Function Harness"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Shader Function Harness Pipeline"),
        layout: None,
        module: &module,
        entry_point: Some("evaluate_test_vectors"),
        compilation_options: Default::default(),
        cache: None,
    });

    let input_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Test Vectors"),
        size: size_of_val(cases) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    context
        .queue
        .write_buffer(&input_buffer, 0, bytemuck::cast_slice(cases));
    let size = (cases.len() * size_of::<[f32; 4]>()) as wgpu::BufferAddress;
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Test Results"),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let read_back_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Test Results Read-back"),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    // As the vertex Arena holds them.
    let test_vertices = (!vertices.is_empty()).then(|| {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Test Vertices"),
            size: size_of_val(vertices) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context
            .queue
            .write_buffer(&buffer, 0, bytemuck::cast_slice(vertices));
        buffer
    });
    let mut entries = vec![
        wgpu::BindGroupEntry {
            binding: 0,
            resource: input_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
            binding: 1,
            resource: output_buffer.as_entire_binding(),
        },
    ];
    entries.extend(test_vertices.as_ref().map(|buffer| wgpu::BindGroupEntry {
        binding: 2,
        resource: buffer.as_entire_binding(),
    }));
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Shader Function Harness Bind Group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &entries,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Shader Function Harness Encoder"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups((cases.len() as u32).div_ceil(WORKGROUP_SIZE), 1, 1);
    }
    encoder.copy_buffer_to_buffer(&output_buffer, 0, &read_back_buffer, 0, size);
    context.queue.submit(std::iter::once(encoder.finish()));

    let (sender, receiver) = std::sync::mpsc::channel();
    read_back_buffer.map_async(wgpu::MapMode::Read, .., move |result| {
        sender.send(result).ok();
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("GPU poll failed");
    receiver
        .recv()
        .expect("the map callback ran")
        .expect("map the read-back buffer");
    let results = read_back_buffer
        .get_mapped_range(..)
        .expect("read the mapped results");
    bytemuck::cast_slice(&results).to_vec()
}
