//! The Froxel pass: a compute pass that builds, from the light buffer, each
//! Froxel's list of the point lights whose range reaches into its bounds,
//! for the lit pass to shade a pixel with its Froxel's lights alone
//! (ADR 0019).

use crate::render::bind_groups::{FROXEL_LISTS, pipeline_layout};
use crate::render::shaders;

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

/// Lights a Froxel's list holds, provisional until M4's split of the 8 GB
/// Budget sizes it: with its count, a list is 512 bytes.
pub(crate) const FROXEL_LIGHT_CAPACITY: u32 = 127;

/// The Froxels: 18,432 lists, 9 MiB.
pub(crate) const FROXEL_COUNT: u32 = FROXEL_COLUMNS * FROXEL_ROWS * FROXEL_SLICES;

/// The pass's pipeline, and the lists it writes.
pub struct FroxelPass {
    pipeline: wgpu::ComputePipeline,
    /// Its group 3 (`FROXEL_LISTS`).
    bind_group: wgpu::BindGroup,
    /// Each Froxel's list: how many lights it holds, then their indices in
    /// the light buffer, in light order.
    froxel_lists: wgpu::Buffer,
    /// The lights the lists had no room for, summed over Froxels: cleared
    /// before the pass, copied to the GPU stats readback after it.
    froxel_lights_overflowed: wgpu::Buffer,
}

impl FroxelPass {
    /// The pass, listing the point lights of `lights`, as `light` counts
    /// them, seen by `camera`: the Renderer's buffers.
    pub fn new(
        device: &wgpu::Device,
        camera: &wgpu::Buffer,
        light: &wgpu::Buffer,
        lights: &wgpu::Buffer,
    ) -> FroxelPass {
        // Created at their fixed capacities; they never grow.
        let froxel_lists = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Froxel List Buffer"),
            size: u64::from(FROXEL_COUNT * (FROXEL_LIGHT_CAPACITY + 1)) * size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let froxel_lights_overflowed = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Froxel Overflow Buffer"),
            size: size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let layout = FROXEL_LISTS.layout(device);
        let bind_group = FROXEL_LISTS.bind_group(
            device,
            &layout,
            [
                ("camera", camera.as_entire_binding()),
                ("light", light.as_entire_binding()),
                ("lights", lights.as_entire_binding()),
                ("froxel_lists", froxel_lists.as_entire_binding()),
                (
                    "froxel_lights_overflowed",
                    froxel_lights_overflowed.as_entire_binding(),
                ),
            ],
        );
        let shader = device.create_shader_module(shaders::FROXEL.descriptor());
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("build_froxel_lists"),
            layout: Some(&pipeline_layout(
                device,
                "Froxel Pass Pipeline Layout",
                &[(&FROXEL_LISTS, &layout)],
            )),
            module: &shader,
            entry_point: Some("build_froxel_lists"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        FroxelPass {
            pipeline,
            bind_group,
            froxel_lists,
            froxel_lights_overflowed,
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
        encoder.clear_buffer(&self.froxel_lights_overflowed, 0, None);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(FROXEL_PASS),
            timestamp_writes,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(FROXEL_LISTS.index, &self.bind_group, &[]);
        pass.dispatch_workgroups(FROXEL_COLUMNS, FROXEL_ROWS, 1);
    }

    /// Each Froxel's list.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn froxel_lists(&self) -> &wgpu::Buffer {
        &self.froxel_lists
    }

    /// The lights the frame's lists had no room for, a u32.
    pub fn froxel_lights_overflowed(&self) -> &wgpu::Buffer {
        &self.froxel_lights_overflowed
    }

    /// Bytes of the lists and their overflow count.
    pub fn gpu_memory_bytes(&self) -> u64 {
        self.froxel_lists.size() + self.froxel_lights_overflowed.size()
    }
}
