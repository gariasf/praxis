//! Functions of the main shader run on the GPU over test vectors and are
//! compared with oracles written apart from the shader: literals, analytic
//! results and CPU references.

use std::io::Write;

use wgpu::util::DeviceExt;

use crate::gpu::GpuContext;

/// Largest difference allowed between a result and its expectation. Results
/// land within 2e-7 of theirs on the M4 Pro.
const TOLERANCE: f32 = 1e-6;

/// Schlick adds (1 − cos θ)⁵ of the light the base reflectance misses: none
/// at normal incidence, 1/32 at cos θ = 0.5, and all of it at grazing.
#[test]
fn fresnel_schlick_reflects_the_base_reflectance_at_normal_incidence() {
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
/// = 1 for θ from 0 to π/2, summed here by the midpoint rule.
#[test]
fn distribution_ggx_integrates_to_one_over_the_hemisphere() {
    let Some(context) = gpu_context() else {
        return;
    };
    // Smoother surfaces peak D too sharply for f32 and these samples to sum
    // within the tolerance.
    let roughnesses = [0.25, 0.5, 0.75, 1.0];
    let samples = 4096;
    let step = std::f64::consts::FRAC_PI_2 / samples as f64;
    let angles: Vec<f64> = (0..samples)
        .map(|sample| (sample as f64 + 0.5) * step)
        .collect();

    let inputs: Vec<[f32; 4]> = roughnesses
        .iter()
        .flat_map(|&roughness| {
            angles
                .iter()
                .map(move |angle| [angle.cos() as f32, roughness, 0.0, 0.0])
        })
        .collect();
    let results = evaluate(
        &context,
        "vec4<f32>(distribution_ggx(input.x, input.y))",
        &inputs,
    );

    for (roughness, distribution) in roughnesses.iter().zip(results.chunks(samples)) {
        let integral = angles
            .iter()
            .zip(distribution)
            .map(|(angle, result)| f64::from(result[0]) * angle.cos() * angle.sin())
            .sum::<f64>()
            * std::f64::consts::TAU
            * step;
        // The sums land within 5e-6 of 1, nearly all of it the midpoint
        // rule's error.
        assert!(
            (integral - 1.0).abs() <= 1e-5,
            "roughness {roughness}: D integrates to {integral}, expected 1"
        );
    }
}

/// Smith's G is the fraction of microfacets both seen and lit, so it stays in
/// [0, 1] and is 1 along the normal. At roughness 1 the shader's k is 1/2,
/// where Schlick's form equals the exact Smith masking of GGX (Karis 2013).
/// Karis's remap for analytic lights keeps k at 1/8 or more, so unlike exact
/// Smith this G does not reach 1 as roughness falls to 0.
#[test]
fn geometry_smith_stays_in_zero_to_one_and_is_exact_at_roughness_one() {
    let Some(context) = gpu_context() else {
        return;
    };
    let steps: [f32; 11] = std::array::from_fn(|step| step as f32 / 10.0);
    let mut inputs = Vec::new();
    for normal_dot_view in steps {
        for normal_dot_light in steps {
            for roughness in steps {
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
    }
}

/// The exact Smith masking of GGX (Walter et al. 2007, eq. 34) for a
/// direction at `cosine` from the normal, at width `alpha`:
/// 2 / (1 + √(1 + α² tan² θ)).
fn smith_masking_ggx(cosine: f64, alpha: f64) -> f64 {
    let tangent_squared = (1.0 - cosine * cosine) / (cosine * cosine);
    2.0 / (1.0 + (1.0 + alpha * alpha * tangent_squared).sqrt())
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
/// functions the renderer runs. `EXPRESSION` stands for the expression under
/// test. Its bindings are ones the main shader leaves free, so no backend
/// emits a main-shader resource beside the kernel's.
const KERNEL: &str = "
@group(0) @binding(1) var<storage, read> test_vectors: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> test_results: array<vec4<f32>>;

@compute @workgroup_size(64)
fn evaluate_test_vectors(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if index < arrayLength(&test_vectors) {
        let input = test_vectors[index];
        test_results[index] = EXPRESSION;
    }
}
";

/// Invocations per workgroup, as `KERNEL` declares.
const WORKGROUP_SIZE: u32 = 64;

/// Evaluates `expression`, WGSL of type `vec4<f32>` that reads its test
/// vector as `input` and may call any function of the main shader, once per
/// element of `inputs` in one compute dispatch; returns the results in order.
fn evaluate(context: &GpuContext, expression: &str, inputs: &[[f32; 4]]) -> Vec<[f32; 4]> {
    let device = &context.device;
    let source = format!(
        "{}\n{}",
        include_str!("../shader.wgsl"),
        KERNEL.replace("EXPRESSION", expression)
    );
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

    let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Test Vectors"),
        contents: bytemuck::cast_slice(inputs),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let size = input_buffer.size();
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
        pass.dispatch_workgroups((inputs.len() as u32).div_ceil(WORKGROUP_SIZE), 1, 1);
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
