//! Functions of the main shader run on the GPU over test vectors and are
//! compared with oracles written apart from the shader: literals, analytic
//! results and CPU references.

use std::io::Write;

use crate::gpu::GpuContext;
use crate::render::instance::{ANCHOR_CELL_SIZE, anchor_and_offset};

/// Largest difference allowed between a result and its expectation. Results
/// land within 2e-7 of theirs on the M4 Pro.
const TOLERANCE: f32 = 1e-6;

/// Schlick adds (1 − cos θ)⁵ of the light the base reflectance misses: none
/// at normal incidence, 1/32 at cos θ = 0.5, and all of it at grazing.
#[test]
fn fresnel_schlick_rises_from_the_base_reflectance_to_one_at_grazing() {
    let Some(context) = gpu_context() else {
        return;
    };
    // One base reflectance per channel: a dielectric, a midpoint, a mirror.
    let base_reflectance = [0.04, 0.5, 1.0];
    let cases = [
        (1.0, base_reflectance),
        (0.5, [0.07, 0.515625, 1.0]),
        (0.0, [1.0, 1.0, 1.0]),
    ];

    let [red, green, blue] = base_reflectance;
    let inputs = cases.map(|(cosine, _)| [cosine, red, green, blue]);
    let results = evaluate(
        &context,
        "vec4<f32>(fresnel_schlick(input.x, input.yzw), 0.0)",
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
/// the hemisphere (Walter et al. 2007, eq. 4): 2π ∫ D(cos θ) cos θ sin θ dθ
/// = 1 for θ from 0 to π/2. The sum takes θ = π/2 · u² at the midpoints of
/// even steps in u, which crowds samples near the normal, where D peaks.
#[test]
fn distribution_ggx_integrates_to_one_over_the_hemisphere() {
    let Some(context) = gpu_context() else {
        return;
    };
    // Below 0.25, f32 rounding near D's peak costs the sum up to 2e-3, at the
    // renderer's floor of 0.04.
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
        "vec4<f32>(distribution_ggx(input.x, input.y))",
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
    // D is 1/(π α²) with α = roughness² (Walter et al. 2007, eq. 33), which
    // is 16/π at roughness 0.5.
    let peak = evaluate(
        &context,
        "vec4<f32>(distribution_ggx(input.x, input.y))",
        &[[1.0, 0.5, 0.0, 0.0]],
    )[0][0];
    assert!(
        (f64::from(peak) - 16.0 / std::f64::consts::PI).abs() <= f64::from(TOLERANCE),
        "D at n·h = 1, roughness 0.5, is {peak}, expected 16/π"
    );
}

/// Smith's G is the fraction of microfacets both seen and lit, so it stays in
/// [0, 1] and is 1 along the normal. At roughness 1 the shader's k is 1/2,
/// where Schlick's form equals the exact Smith masking of GGX (Karis 2013).
/// Below that, k follows Disney's roughness remap for analytic lights, which
/// Karis adopts, and a literal pins it at roughness 0.5. The remap keeps k at
/// 1/8 or more, so unlike exact Smith this G does not reach 1 as roughness
/// falls to 0.
#[test]
fn geometry_smith_stays_in_zero_to_one_and_is_exact_at_roughness_one() {
    let Some(context) = gpu_context() else {
        return;
    };
    let tenths: [f32; 11] = std::array::from_fn(|tenth| tenth as f32 / 10.0);
    let mut inputs = Vec::new();
    for normal_dot_view in tenths {
        for normal_dot_light in tenths {
            for roughness in tenths {
                inputs.push([normal_dot_view, normal_dot_light, roughness, 0.0]);
            }
        }
    }
    let results = evaluate(
        &context,
        "vec4<f32>(geometry_smith(input.x, input.y, input.z))",
        &inputs,
    );

    for (&[normal_dot_view, normal_dot_light, roughness, _], result) in inputs.iter().zip(results) {
        let geometry = result[0];
        assert!(
            (-TOLERANCE..=1.0 + TOLERANCE).contains(&geometry),
            "G({normal_dot_view}, {normal_dot_light}, {roughness}) = {geometry}, outside [0, 1]"
        );
        if normal_dot_view == 1.0 && normal_dot_light == 1.0 {
            assert!(
                (geometry - 1.0).abs() <= TOLERANCE,
                "G(1, 1, {roughness}) = {geometry}, expected 1"
            );
        }
        if roughness == 1.0 {
            let expected = smith_masking_ggx(normal_dot_view.into(), 1.0)
                * smith_masking_ggx(normal_dot_light.into(), 1.0);
            assert!(
                (f64::from(geometry) - expected).abs() <= f64::from(TOLERANCE),
                "G({normal_dot_view}, {normal_dot_light}, 1) = {geometry}, expected {expected}"
            );
        }
        if [normal_dot_view, normal_dot_light, roughness] == [0.5; 3] {
            // k = (0.5 + 1)² / 8 = 9/32, so each term is
            // 0.5 / (0.5 · 23/32 + 9/32) = 32/41.
            let expected = 1024.0 / 1681.0;
            assert!(
                (f64::from(geometry) - expected).abs() <= f64::from(TOLERANCE),
                "G(0.5, 0.5, 0.5) = {geometry}, expected 1024/1681"
            );
        }
    }
}

/// The exact Smith masking of GGX (Walter et al. 2007, eq. 34) for a
/// direction at `cosine` from the normal, at width `alpha`:
/// 2 / (1 + √(1 + α² tan² θ)).
fn smith_masking_ggx(cosine: f64, alpha: f64) -> f64 {
    let tangent_squared = (1.0 - cosine * cosine) / (cosine * cosine);
    2.0 / (1.0 + (1.0 + alpha * alpha * tangent_squared).sqrt())
}

/// A cooked normal map stores a unit normal's x and y, each mapped to
/// [0, 1], and the shader rebuilds z = √(1 − x² − y²), the cosine between the
/// normal and the surface's, which falls to 0 at grazing slopes. On a lattice
/// of encodings 1/512 apart, whose x, y and squares f32 holds exactly, every
/// result matches that closed form, evaluated in f64, within sqrt's rounding.
/// Past the unit circle, where BC5's error can carry an encoding, z is 0, not
/// NaN.
#[test]
fn tangent_space_normal_rebuilds_z_from_x_and_y() {
    let Some(context) = gpu_context() else {
        return;
    };
    let steps: Vec<f64> = (0..=1024)
        .step_by(2)
        .map(|step| f64::from(step) / 1024.0)
        .collect();
    let inputs: Vec<[f32; 4]> = steps
        .iter()
        .flat_map(|&x| steps.iter().map(move |&y| [x as f32, y as f32, 0.0, 0.0]))
        .collect();
    let results = evaluate(
        &context,
        "vec4<f32>(tangent_space_normal(input.xy), 0.0)",
        &inputs,
    );

    let mut grazing = 0;
    for (input, result) in inputs.iter().zip(results) {
        let [x, y] = [input[0], input[1]].map(|encoded| f64::from(encoded) * 2.0 - 1.0);
        let expected = (1.0 - x * x - y * y).max(0.0).sqrt();
        let [result_x, result_y, result_z, _] = result.map(f64::from);
        assert!(
            (result_x, result_y) == (x, y) && (result_z - expected).abs() <= 1e-6 * expected,
            "encoded ({}, {}) rebuilds {result:?}, expected ({x}, {y}, {expected})",
            input[0],
            input[1]
        );
        grazing += usize::from(expected > 0.0 && expected < 1.0f64.to_radians().sin());
    }
    // Normals within 1° of grazing are among the cases.
    assert!(grazing > 0);
}

/// The cooker's QTangents, unpacked as a `Snorm16x4` vertex attribute is
/// (each component over 32,767, clamped at −1), decode on the GPU to the
/// normal, tangent and bitangent sign they were cooked from, both signs and
/// frames near the half turns whose w rounds towards zero included. A rotated
/// axis is a product of two quaternions, each component off by at most half a
/// Snorm16 step, plus the nudge that keeps w a whole step: within 4 steps.
#[cfg(feature = "cook")]
#[test]
fn qtangents_from_the_cooker_decode_to_their_frame() {
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
    let inputs: Vec<[f32; 4]> = tangent_frames
        .iter()
        .map(|&(normal, tangent, bitangent_sign)| {
            qtangent(normal.into(), tangent.extend(bitangent_sign).into())
                .map(|component| (f32::from(component) * step).max(-1.0))
        })
        .collect();
    let normals_and_signs = evaluate(
        &context,
        "vec4<f32>(decode_qtangent(input).normal, decode_qtangent(input).bitangent_sign)",
        &inputs,
    );
    let tangents = evaluate(
        &context,
        "vec4<f32>(decode_qtangent(input).tangent, 0.0)",
        &inputs,
    );

    for ((frame, normal_and_sign), tangent) in
        tangent_frames.iter().zip(normals_and_signs).zip(tangents)
    {
        let &(normal, tangent_expected, bitangent_sign) = frame;
        let decoded_normal = glam::Vec3::from_slice(&normal_and_sign);
        let decoded_tangent = glam::Vec3::from_slice(&tangent);
        assert!(
            normal_and_sign[3] == bitangent_sign
                && decoded_normal.abs_diff_eq(normal, 4.0 * step)
                && decoded_tangent.abs_diff_eq(tangent_expected, 4.0 * step),
            "normal {normal}, tangent {tangent_expected}, bitangent sign {bitangent_sign} decode \
             to {decoded_normal}, {decoded_tangent}, {}",
            normal_and_sign[3]
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

/// The compute kernel appended to the main shader's source, so it calls the
/// functions the renderer runs. `EXPRESSION`, `VECTORS_PER_CASE` and
/// `WORKGROUP_SIZE` stand for the expression under test, the test vectors
/// each case holds and the constant below. Its bindings are ones
/// the main shader leaves free, so no backend emits a main-shader resource
/// beside the kernel's.
const KERNEL: &str = "
@group(0) @binding(1) var<storage, read> test_vectors: array<array<vec4<f32>, VECTORS_PER_CASE>>;
@group(0) @binding(2) var<storage, read_write> test_results: array<vec4<f32>>;

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
/// vector as `input` and may call any function of the main shader, once per
/// element of `inputs` in one compute dispatch; returns the results in order.
fn evaluate(context: &GpuContext, expression: &str, inputs: &[[f32; 4]]) -> Vec<[f32; 4]> {
    evaluate_cases::<1>(context, expression, bytemuck::cast_slice(inputs))
}

/// As `evaluate`, for cases of `VECTORS` test vectors each, which
/// `expression` reads as `inputs[0]` to `inputs[VECTORS - 1]`.
fn evaluate_cases<const VECTORS: usize>(
    context: &GpuContext,
    expression: &str,
    cases: &[[[f32; 4]; VECTORS]],
) -> Vec<[f32; 4]> {
    let device = &context.device;
    let kernel = KERNEL
        .replace("WORKGROUP_SIZE", &WORKGROUP_SIZE.to_string())
        .replace("VECTORS_PER_CASE", &VECTORS.to_string())
        .replace("EXPRESSION", expression);
    let source = format!("{}\n{kernel}", include_str!("../shader.wgsl"));
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
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Shader Function Harness Bind Group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 1,
                resource: input_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: output_buffer.as_entire_binding(),
            },
        ],
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
