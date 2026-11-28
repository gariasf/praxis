use crate::render::CameraUniform;
use crate::render::bind_groups::{
    PROBE_TRACE_INPUTS, RAY_SCENE, compute_pipeline, pipeline_layout,
};
use crate::render::probes::{Cascade, PROBE_COUNTS};
use crate::render::readback::FrameTimestamps;
use crate::render::renderer::SharedBuffers;
use crate::render::shaders;
use crate::stats::{Counted, FrameRecorder, GpuMemory, GpuMemoryKind};

/// The Probe update's Timed pass.
pub(crate) const PROBE_UPDATE_PASS: &str = "probe_update";

pub(crate) const PROBE_RAYS: u32 = 128;

const PROBE_LEVELS_PER_FRAME: u32 = 2;

pub(crate) const PROBE_TRACE_WORKGROUP_SIZE: u32 = 64;

const PROBES_PER_FRAME: u32 = PROBE_LEVELS_PER_FRAME * PROBE_COUNTS[0] * PROBE_COUNTS[2];

const PROBE_RAYS_PER_FRAME: u32 = PROBES_PER_FRAME * PROBE_RAYS;

/// Frames that update every level once.
const FRAMES_PER_CYCLE: u64 = (PROBE_COUNTS[1] / PROBE_LEVELS_PER_FRAME) as u64;

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

/// The Cascade it updates, its pipeline and the buffers its group binds.
pub struct ProbeUpdatePass {
    cascade: Cascade,
    trace_pipeline: wgpu::ComputePipeline,
    trace_bind_group: wgpu::BindGroup,
    uniform_buffer: Counted<wgpu::Buffer>,
    /// Counted while the group binds them.
    _buffers: [Counted<wgpu::Buffer>; 2],
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
        ProbeUpdatePass {
            cascade,
            trace_pipeline,
            trace_bind_group,
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

    /// Records the trace of the levels `frame_upload` named into `encoder`,
    /// through the Ray scene of `ray_scene_bind_group`, timed into
    /// `timestamps` and counted in `recorder`.
    pub fn record(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ray_scene_bind_group: &wgpu::BindGroup,
        timestamps: &mut Option<FrameTimestamps<'_>>,
        recorder: &mut FrameRecorder,
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(PROBE_UPDATE_PASS),
            timestamp_writes: timestamps
                .as_mut()
                .map(|timestamps| timestamps.pass(PROBE_UPDATE_PASS).compute()),
        });
        pass.set_pipeline(&self.trace_pipeline);
        pass.set_bind_group(RAY_SCENE.index, ray_scene_bind_group, &[]);
        pass.set_bind_group(PROBE_TRACE_INPUTS.index, &self.trace_bind_group, &[]);
        pass.dispatch_workgroups(PROBE_RAYS / PROBE_TRACE_WORKGROUP_SIZE, PROBES_PER_FRAME, 1);
        self.updates += 1;
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
