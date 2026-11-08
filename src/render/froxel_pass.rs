//! The Froxel pass: a compute pass that builds, from the light buffer, each
//! Froxel's list of the point lights whose range reaches into its bounds,
//! for the lit pass to shade a pixel with its Froxel's lights alone
//! (ADR 0019).

use crate::render::RenderCapacities;
use crate::render::bind_groups::{FROXEL_LISTS, compute_pipeline, pipeline_layout};
use crate::render::shaders;
use crate::stats::{Counted, GpuMemory, GpuMemoryKind};

/// The Froxel pass's label, and its name in frame stats.
pub(crate) const FROXEL_PASS: &str = "froxel_pass";

/// Rectangles across and down the screen, all one size: 80 pixels square at
/// 2560 × 1440.
pub(crate) const FROXEL_COLUMNS: u32 = 32;
pub(crate) const FROXEL_ROWS: u32 = 18;

/// Slices of view depth each rectangle is cut into: the first nearer than
/// `FIRST_FROXEL_SLICE_DEPTH` metres, then `FROXEL_SLICES_PER_OCTAVE` to each
/// doubling of depth, the last from 1,024 m to infinity (ADR 0015).
pub(crate) const FROXEL_SLICES: u32 = 32;
pub(crate) const FIRST_FROXEL_SLICE_DEPTH: f32 = 1.0;
pub(crate) const FROXEL_SLICES_PER_OCTAVE: f32 = 3.0;

/// Lights a Froxel's list holds, fixed and held in the scene and pass
/// buffers' Budget: with its count, a list is 512 bytes.
pub(crate) const FROXEL_LIGHT_CAPACITY: u32 = 127;

/// The Froxels: 18,432 lists, 9 MiB.
pub(crate) const FROXEL_COUNT: u32 = FROXEL_COLUMNS * FROXEL_ROWS * FROXEL_SLICES;

/// What the frame's lights counted, as `common.wgsl` lays it out, which the
/// GPU stats readback reads: the lights the lists had no room for, then,
/// with the `count_point_lights` Render switch on, the pixels the light
/// count pass counted and the point lights their Froxels list and whose
/// range reaches them.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LightCounts {
    pub(crate) froxel_lights_overflowed: u32,
    pub(crate) lit_pixels: u32,
    pub(crate) listed_point_lights: u32,
    pub(crate) reaching_point_lights: u32,
}

/// The pass's pipeline, and the lists it writes.
pub struct FroxelPass {
    pipeline: wgpu::ComputePipeline,
    /// Its group 3 (`FROXEL_LISTS`).
    bind_group: wgpu::BindGroup,
    /// Each Froxel's list: how many lights it holds, then their indices in
    /// the light buffer, in light order.
    froxel_lists: Counted<wgpu::Buffer>,
    /// `LightCounts`: cleared before the pass, copied to the GPU stats
    /// readback at the frame's end.
    light_counts: Counted<wgpu::Buffer>,
}

impl FroxelPass {
    /// The pass, listing the point lights of `lights`, as `light` counts
    /// them, seen by `camera`: the Renderer's buffers. Its shader module is
    /// assembled with `capacities`.
    pub fn new(
        device: &wgpu::Device,
        memory: &GpuMemory,
        camera: &wgpu::Buffer,
        light: &wgpu::Buffer,
        lights: &wgpu::Buffer,
        capacities: &RenderCapacities,
    ) -> FroxelPass {
        // Created at their fixed capacities; they never grow.
        let froxel_lists = memory.create_buffer(
            device,
            GpuMemoryKind::SceneAndPassBuffers,
            &wgpu::BufferDescriptor {
                label: Some("Froxel List Buffer"),
                size: u64::from(FROXEL_COUNT * (FROXEL_LIGHT_CAPACITY + 1))
                    * size_of::<u32>() as u64,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            },
        );
        let light_counts = memory.create_buffer(
            device,
            GpuMemoryKind::SceneAndPassBuffers,
            &wgpu::BufferDescriptor {
                label: Some("Light Count Buffer"),
                size: size_of::<LightCounts>() as u64,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            },
        );
        let layout = FROXEL_LISTS.layout(device);
        let bind_group = FROXEL_LISTS.bind_group(
            device,
            &layout,
            [
                ("camera", camera.as_entire_binding()),
                ("light", light.as_entire_binding()),
                ("lights", lights.as_entire_binding()),
                ("froxel_lists", froxel_lists.as_entire_binding()),
                ("light_counts", light_counts.as_entire_binding()),
            ],
        );
        let shader = device.create_shader_module(shaders::FROXEL.descriptor(capacities));
        let groups = [(&FROXEL_LISTS, &layout)];
        let pipeline_layout = pipeline_layout(device, "Froxel Pass Pipeline Layout", &groups);
        let pipeline =
            compute_pipeline(device, &pipeline_layout, &shader, "build_froxel_lists", &[]);
        FroxelPass {
            pipeline,
            bind_group,
            froxel_lists,
            light_counts,
        }
    }

    /// Records the pass into `encoder`, timed into `timestamp_writes`: a
    /// workgroup per screen rectangle and an invocation per Froxel builds
    /// every list.
    pub fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        timestamp_writes: Option<wgpu::ComputePassTimestampWrites<'_>>,
    ) {
        encoder.clear_buffer(&self.light_counts, 0, None);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(FROXEL_PASS),
            timestamp_writes,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(FROXEL_LISTS.index, &self.bind_group, &[]);
        pass.dispatch_workgroups(FROXEL_COLUMNS, FROXEL_ROWS, 1);
    }

    /// Each Froxel's list, which the lit pass reads.
    pub fn froxel_lists(&self) -> &wgpu::Buffer {
        &self.froxel_lists
    }

    /// What the frame's lights counted (`LightCounts`), to which the light
    /// count pass adds.
    pub fn light_counts(&self) -> &wgpu::Buffer {
        &self.light_counts
    }
}
