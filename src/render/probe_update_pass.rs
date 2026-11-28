use crate::render::CameraUniform;
use crate::render::bind_groups::{
    PROBE_BLEND_INPUTS, PROBE_TRACE_INPUTS, RAY_SCENE, compute_pipeline, pipeline_layout,
};
use crate::render::probes::{
    Cascade, DEPTH_MOMENTS_SIDE_TEXELS, IRRADIANCE_SIDE_TEXELS, PROBE_COUNTS,
};
use crate::render::readback::FrameTimestamps;
use crate::render::renderer::SharedBuffers;
use crate::render::shaders;
use crate::stats::{Counted, FrameRecorder, GpuMemory, GpuMemoryKind};

/// The Probe update's Timed pass, from the start of its trace to the end of
/// its last blend.
pub(crate) const PROBE_UPDATE_PASS: &str = "probe_update";

pub(crate) const PROBE_RAYS: u32 = 128;

pub(crate) const PROBE_LEVELS_PER_FRAME: u32 = 2;

pub(crate) const PROBE_TRACE_WORKGROUP_SIZE: u32 = 64;

/// The share of what a Probe held that an update, one each cycle of 8
/// frames, keeps: 0.917 a frame, within DDGI's 0.85 to 0.98
/// [ref:majercik-2019-ddgi-hysteresis]; 32 frames take 15/16 of a step.
const PROBE_HYSTERESIS: f64 = 0.5;

const PROBES_PER_FRAME: u32 = PROBE_LEVELS_PER_FRAME * PROBE_COUNTS[0] * PROBE_COUNTS[2];

const PROBE_RAYS_PER_FRAME: u32 = PROBES_PER_FRAME * PROBE_RAYS;

/// Frames that update every level once.
const FRAMES_PER_CYCLE: u64 = (PROBE_COUNTS[1] / PROBE_LEVELS_PER_FRAME) as u64;

// A frame's first level is a multiple of its levels, which the blends read
// each Probe's rays by.
const _: () = assert!(PROBE_COUNTS[1].is_multiple_of(PROBE_LEVELS_PER_FRAME));
const _: () = assert!(PROBE_RAYS.is_multiple_of(PROBE_TRACE_WORKGROUP_SIZE));

/// The Cascade's place, an Anchor and an offset (ADR 0014), and the first
/// level the frame updates.
#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ProbeUpdateUniform {
    pub cascade_anchor: [i32; 3],
    pub first_level: u32,
    pub cascade_offset: [f32; 3],
    pub _pad_cascade_offset: u32,
}

/// The Cascade it updates, its pipelines and the buffers its groups bind.
pub struct ProbeUpdatePass {
    cascade: Cascade,
    trace_pipeline: wgpu::ComputePipeline,
    irradiance_pipeline: wgpu::RenderPipeline,
    depth_moments_pipeline: wgpu::RenderPipeline,
    trace_bind_group: wgpu::BindGroup,
    blend_bind_group: wgpu::BindGroup,
    uniform_buffer: Counted<wgpu::Buffer>,
    /// Counted while the groups bind them.
    _buffers: [Counted<wgpu::Buffer>; 2],
    irradiance_view: wgpu::TextureView,
    depth_moments_view: wgpu::TextureView,
    uniform: ProbeUpdateUniform,
    /// Set from the camera of the first frame uploaded.
    placed: bool,
    updates: u64,
}

impl ProbeUpdatePass {
    /// A Cascade and its update, tracing the Ray scene, whose group is of
    /// `ray_scene_layout`, and reading `shared`'s camera and light uniform.
    pub fn new(
        device: &wgpu::Device,
        memory: &GpuMemory,
        ray_scene_layout: &wgpu::BindGroupLayout,
        shared: &SharedBuffers,
    ) -> ProbeUpdatePass {
        let cascade = Cascade::new(device, memory);
        let buffer = |label, size, usage, mapped_at_creation| {
            memory.create_buffer(
                device,
                GpuMemoryKind::Probes,
                &wgpu::BufferDescriptor {
                    label: Some(label),
                    size,
                    usage,
                    mapped_at_creation,
                },
            )
        };
        let uniform_buffer = buffer(
            "Probe Update Uniform",
            size_of::<ProbeUpdateUniform>() as u64,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            false,
        );
        let directions = probe_ray_directions();
        let ray_directions = buffer(
            "Probe Ray Directions",
            size_of_val(&directions) as u64,
            wgpu::BufferUsages::UNIFORM,
            true,
        );
        ray_directions
            .slice(..)
            .get_mapped_range_mut()
            .expect("a buffer created mapped is mapped")
            .copy_from_slice(bytemuck::cast_slice(&directions));
        ray_directions.unmap();
        let rays = buffer(
            "Probe Rays",
            u64::from(PROBE_RAYS_PER_FRAME) * size_of::<[f32; 4]>() as u64,
            wgpu::BufferUsages::STORAGE,
            false,
        );

        let trace_layout = PROBE_TRACE_INPUTS.layout(device);
        let trace_shader = device.create_shader_module(shaders::PROBE_TRACE.descriptor());
        let trace_pipeline = compute_pipeline(
            device,
            &pipeline_layout(
                device,
                "Probe Trace Pipeline Layout",
                &[
                    (&RAY_SCENE, ray_scene_layout),
                    (&PROBE_TRACE_INPUTS, &trace_layout),
                ],
            ),
            &trace_shader,
            "trace_probe_rays",
            &[],
        );
        let trace_bind_group = PROBE_TRACE_INPUTS.bind_group(
            device,
            &trace_layout,
            [
                ("camera", shared.camera.as_entire_binding()),
                ("light", shared.light.as_entire_binding()),
                ("probe_update", uniform_buffer.as_entire_binding()),
                ("probe_ray_directions", ray_directions.as_entire_binding()),
                ("probe_rays", rays.as_entire_binding()),
            ],
        );

        let blend_layout = PROBE_BLEND_INPUTS.layout(device);
        let blend_shader = device.create_shader_module(shaders::PROBE_BLEND.descriptor());
        let blend_pipeline_layout = pipeline_layout(
            device,
            "Probe Blend Pipeline Layout",
            &[(&PROBE_BLEND_INPUTS, &blend_layout)],
        );
        // What the pass's blend constant, the hysteresis, keeps of the texel.
        let hysteresis = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::OneMinusConstant,
            dst_factor: wgpu::BlendFactor::Constant,
            operation: wgpu::BlendOperation::Add,
        };
        let blend_pipeline = |entry_point, format| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry_point),
                layout: Some(&blend_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &blend_shader,
                    entry_point: Some("vs_full_screen"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &blend_shader,
                    entry_point: Some(entry_point),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState {
                            color: hysteresis,
                            alpha: hysteresis,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        let irradiance_pipeline = blend_pipeline("blend_irradiance", cascade.irradiance().format());
        let depth_moments_pipeline =
            blend_pipeline("blend_depth_moments", cascade.depth_moments().format());
        let blend_bind_group = PROBE_BLEND_INPUTS.bind_group(
            device,
            &blend_layout,
            [
                ("probe_ray_directions", ray_directions.as_entire_binding()),
                ("probe_rays", rays.as_entire_binding()),
            ],
        );
        let view = |texture: &wgpu::Texture| texture.create_view(&Default::default());
        ProbeUpdatePass {
            irradiance_view: view(cascade.irradiance()),
            depth_moments_view: view(cascade.depth_moments()),
            cascade,
            trace_pipeline,
            irradiance_pipeline,
            depth_moments_pipeline,
            trace_bind_group,
            blend_bind_group,
            uniform_buffer,
            _buffers: [ray_directions, rays],
            uniform: ProbeUpdateUniform::default(),
            placed: false,
            updates: 0,
        }
    }

    pub(crate) fn cascade(&self) -> &Cascade {
        &self.cascade
    }

    /// What `frame_upload` writes into, each frame.
    pub fn uniform_buffer(&self) -> &wgpu::Buffer {
        &self.uniform_buffer
    }

    /// The uniform buffer and what the frame of `camera` writes into it: the
    /// Cascade stands where the camera of the first frame stood.
    pub fn frame_upload(&mut self, camera: &CameraUniform) -> (&wgpu::Buffer, &[u8]) {
        if !self.placed {
            self.uniform.cascade_anchor = camera.anchor;
            self.uniform.cascade_offset = camera.offset;
            self.placed = true;
        }
        self.uniform.first_level =
            (self.updates % FRAMES_PER_CYCLE) as u32 * PROBE_LEVELS_PER_FRAME;
        (&self.uniform_buffer, bytemuck::bytes_of(&self.uniform))
    }

    /// Records the update of the levels `frame_upload` named into `encoder`,
    /// tracing the Ray scene of `ray_scene_bind_group`, timed into
    /// `timestamps` and counted in `recorder`. The first cycle of updates
    /// fills the Probes; each later one keeps the hysteresis of them.
    pub fn record(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ray_scene_bind_group: &wgpu::BindGroup,
        timestamps: &mut Option<FrameTimestamps<'_>>,
        recorder: &mut FrameRecorder,
    ) {
        let (trace_writes, blend_writes) = timestamps
            .as_mut()
            .map(|timestamps| timestamps.pass(PROBE_UPDATE_PASS).span())
            .unzip();
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(PROBE_UPDATE_PASS),
                timestamp_writes: trace_writes,
            });
            pass.set_pipeline(&self.trace_pipeline);
            pass.set_bind_group(RAY_SCENE.index, ray_scene_bind_group, &[]);
            pass.set_bind_group(PROBE_TRACE_INPUTS.index, &self.trace_bind_group, &[]);
            pass.dispatch_workgroups(PROBE_RAYS / PROBE_TRACE_WORKGROUP_SIZE, PROBES_PER_FRAME, 1);
        }
        let hysteresis = if self.updates < FRAMES_PER_CYCLE {
            0.0
        } else {
            PROBE_HYSTERESIS
        };
        let blends = [
            (
                &self.irradiance_view,
                &self.irradiance_pipeline,
                IRRADIANCE_SIDE_TEXELS,
                None,
            ),
            (
                &self.depth_moments_view,
                &self.depth_moments_pipeline,
                DEPTH_MOMENTS_SIDE_TEXELS,
                blend_writes,
            ),
        ];
        for (view, pipeline, side_texels, timestamp_writes) in blends {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(PROBE_UPDATE_PASS),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes,
                multiview_mask: None,
            });
            let level_width = (PROBE_COUNTS[0] * side_texels) as f32;
            pass.set_viewport(
                self.uniform.first_level as f32 * level_width,
                0.0,
                PROBE_LEVELS_PER_FRAME as f32 * level_width,
                (PROBE_COUNTS[2] * side_texels) as f32,
                0.0,
                1.0,
            );
            pass.set_blend_constant(wgpu::Color {
                r: hysteresis,
                g: hysteresis,
                b: hysteresis,
                a: hysteresis,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(PROBE_BLEND_INPUTS.index, &self.blend_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        self.updates += 1;
        recorder.counters.draw_calls += 2;
        recorder.counters.probes_updated = PROBES_PER_FRAME;
        recorder.counters.probe_rays_traced = PROBE_RAYS_PER_FRAME;
    }
}

/// Each ray's direction from a Probe, the same every update: a Fibonacci
/// spiral's even spread over the sphere about Y
/// [ref:majercik-2019-ddgi-ray-directions].
fn probe_ray_directions() -> [[f32; 4]; PROBE_RAYS as usize] {
    let golden_angle = std::f64::consts::PI * (3.0 - 5f64.sqrt());
    std::array::from_fn(|ray| {
        let height = 1.0 - (2.0 * ray as f64 + 1.0) / f64::from(PROBE_RAYS);
        let radius = (1.0 - height * height).sqrt();
        let angle = golden_angle * ray as f64;
        [radius * angle.cos(), height, radius * angle.sin(), 0.0].map(|value| value as f32)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_rays_are_unit_directions_spread_over_the_sphere() {
        let directions = probe_ray_directions().map(|[x, y, z, _]| glam::vec3(x, y, z));
        for direction in directions {
            assert!((direction.length() - 1.0).abs() < 1e-6, "{direction}");
        }
        let mean = directions.iter().sum::<glam::Vec3>() / PROBE_RAYS as f32;
        assert!(mean.length() < 1.0 / PROBE_RAYS as f32, "{mean}");
    }
}
