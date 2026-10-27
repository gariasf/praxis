//! The shadow mask pass: a compute pass that traces, at each pixel, one hard
//! shadow ray toward each light that reaches the surface its nearest sample
//! shows, against the static and dynamic TLAS (ADR 0020), and keeps a bit
//! per light its ray found hidden in the shadow mask, which the lit pass
//! reads. It runs once both culling phases' Instances are in the depth
//! target.

use crate::render::bind_groups::{SCENE_GEOMETRY, SHADOW_MASK_BUILD, pipeline_layout};
use crate::render::froxel_pass::FROXEL_SLICES;
use crate::render::renderer::RenderSwitches;
use crate::render::shaders;
use crate::render::targets::{RenderTargets, SAMPLE_COUNT};

/// The shadow mask pass's label, and its name in frame stats.
pub(crate) const SHADOW_MASK_PASS: &str = "shadow_mask_pass";

/// Pixels along each side of the tile each workgroup traces.
pub(crate) const SHADOW_MASK_TILE_SIZE: u32 = 8;

/// Lights the mask holds a bit for at each pixel, of those that reach it in
/// the order the lit pass sums them: its directional lights, then the point
/// lights its Froxel lists. Any other casts no shadow there, and is counted.
/// The bits and the Froxel slice they follow fill the mask's 8 bytes per
/// pixel, and cover a Froxel's whole list in `town` as estimated.
pub(crate) const SHADOW_MASK_LIGHTS: u32 = 59;

/// Where each of a pixel's samples lies, from its top left corner, in
/// pixels, which the pass rebuilds each sample's surface at: the standard
/// pattern of four, Vulkan's standard sample locations, which an adapter
/// promises only with `standardSampleLocations` and wgpu does not report
/// [ref:vulkan-standard-sample-locations].
/// WGSL declares them as `SAMPLE_OFFSETS`, and a test checks the backend's
/// depth target against them.
pub(crate) const SAMPLE_OFFSETS: [[f32; 2]; SAMPLE_COUNT as usize] = [
    [0.375, 0.125],
    [0.875, 0.375],
    [0.125, 0.625],
    [0.625, 0.875],
];

// The pass loads each pixel's samples by index (`shadow_mask.wgsl`).
const _: () = assert!(SAMPLE_COUNT == 4);

// Each pixel's word, two halves of 32 bits, holds the Froxel slice its bits
// follow above them, in the second half.
const _: () = assert!(FROXEL_SLICES.is_power_of_two());
const _: () = assert!(SHADOW_MASK_LIGHTS > u32::BITS);
const _: () = assert!(SHADOW_MASK_LIGHTS + FROXEL_SLICES.ilog2() == 2 * u32::BITS);

/// The pass's pipeline, and its group 3 with the buffers it reads.
pub struct ShadowMaskPass {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    /// The camera, the lights' counts, the lights and the Froxel lists.
    buffers: [(&'static str, wgpu::Buffer); 4],
    /// The lights that reach a pixel past its bits, summed over pixels:
    /// cleared before the pass, copied to the GPU stats readback after it.
    shadow_mask_lights_overflowed: wgpu::Buffer,
    /// Built from the render targets by `bind_render_targets`.
    bind_group: wgpu::BindGroup,
    /// The targets' size, in pixels.
    size: (u32, u32),
}

impl ShadowMaskPass {
    /// The pass, given the scene geometry's layout, reading the Renderer's
    /// `camera`, `light` and `lights` buffers and the Froxel pass's
    /// `froxel_lists`, and writing the shadow mask of `render_targets`, with
    /// its Froxel lookups as `switches` set.
    pub fn new(
        device: &wgpu::Device,
        scene_geometry_layout: &wgpu::BindGroupLayout,
        buffers: [(&'static str, &wgpu::Buffer); 4],
        render_targets: &RenderTargets,
        switches: RenderSwitches,
    ) -> ShadowMaskPass {
        let layout = SHADOW_MASK_BUILD.layout(device);
        let shader = device.create_shader_module(shaders::SHADOW_MASK.descriptor());
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("build_shadow_mask"),
            layout: Some(&pipeline_layout(
                device,
                "Shadow Mask Pipeline Layout",
                &[
                    (&SCENE_GEOMETRY, scene_geometry_layout),
                    (&SHADOW_MASK_BUILD, &layout),
                ],
            )),
            module: &shader,
            entry_point: Some("build_shadow_mask"),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &switches.constants(),
                ..wgpu::PipelineCompilationOptions::default()
            },
            cache: None,
        });
        let buffers = buffers.map(|(name, buffer)| (name, buffer.clone()));
        let shadow_mask_lights_overflowed = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Mask Overflow Buffer"),
            size: size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let bind_group = Self::bind_group(
            device,
            &layout,
            &buffers,
            &shadow_mask_lights_overflowed,
            render_targets,
        );
        let size = render_targets.shadow_mask().texture().size();
        ShadowMaskPass {
            pipeline,
            layout,
            buffers,
            shadow_mask_lights_overflowed,
            bind_group,
            size: (size.width, size.height),
        }
    }

    /// Builds the bind group that reads and writes `render_targets`: at
    /// creation, and after every resize, which creates them again.
    pub fn bind_render_targets(&mut self, device: &wgpu::Device, render_targets: &RenderTargets) {
        self.bind_group = Self::bind_group(
            device,
            &self.layout,
            &self.buffers,
            &self.shadow_mask_lights_overflowed,
            render_targets,
        );
        let size = render_targets.shadow_mask().texture().size();
        self.size = (size.width, size.height);
    }

    fn bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        buffers: &[(&'static str, wgpu::Buffer); 4],
        shadow_mask_lights_overflowed: &wgpu::Buffer,
        render_targets: &RenderTargets,
    ) -> wgpu::BindGroup {
        let own = [
            (
                "shadow_mask_lights_overflowed",
                shadow_mask_lights_overflowed.as_entire_binding(),
            ),
            (
                "depth_target",
                wgpu::BindingResource::TextureView(render_targets.depth()),
            ),
            (
                "shadow_mask",
                wgpu::BindingResource::TextureView(render_targets.shadow_mask()),
            ),
        ];
        SHADOW_MASK_BUILD.bind_group(
            device,
            layout,
            buffers
                .iter()
                .map(|(name, buffer)| (*name, buffer.as_entire_binding()))
                .chain(own),
        )
    }

    /// Records the pass into `encoder`, with the scene geometry's
    /// `scene_geometry` set, timed into `timestamp_writes`.
    pub fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        scene_geometry: &wgpu::BindGroup,
        timestamp_writes: Option<wgpu::ComputePassTimestampWrites<'_>>,
    ) {
        encoder.clear_buffer(&self.shadow_mask_lights_overflowed, 0, None);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(SHADOW_MASK_PASS),
            timestamp_writes,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(SCENE_GEOMETRY.index, scene_geometry, &[]);
        pass.set_bind_group(SHADOW_MASK_BUILD.index, &self.bind_group, &[]);
        let (width, height) = self.size;
        pass.dispatch_workgroups(
            width.div_ceil(SHADOW_MASK_TILE_SIZE),
            height.div_ceil(SHADOW_MASK_TILE_SIZE),
            1,
        );
    }

    /// The lights the frame's mask had no bit for at the pixels they reach,
    /// a u32.
    pub fn shadow_mask_lights_overflowed(&self) -> &wgpu::Buffer {
        &self.shadow_mask_lights_overflowed
    }

    /// Bytes of its overflow count; the mask is a render target.
    pub fn gpu_memory_bytes(&self) -> u64 {
        self.shadow_mask_lights_overflowed.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::pools::tests::gpu_context;
    use crate::render::depth_pyramid_pass::tests::{draw_depth, read_depth_samples};

    /// One triangle over the whole target, its depth rising across it and
    /// down it: each sample of the depth target holds the triangle's depth
    /// where `SAMPLE_OFFSETS` places that sample, so the pass rebuilds each
    /// sample's surface where it was drawn. Depth is linear in screen space,
    /// so the expected value is the triangle's plane at the sample.
    #[test]
    fn each_depth_sample_lies_where_the_pass_rebuilds_it() {
        let Some(context) = gpu_context() else {
            return;
        };
        let (width, height) = (37, 23);
        let render_targets = RenderTargets::new(&context.device, width, height, false);
        // Clip-space corners, w 1, so depth is z: a triangle past each side.
        let corners = [
            [-1.0, -1.0, 0.2, 1.0],
            [3.0, -1.0, 0.8, 1.0],
            [-1.0, 3.0, 0.5, 1.0],
        ];
        draw_depth(&context, &render_targets, &[corners]);
        let samples = read_depth_samples(&context, render_targets.depth());

        // z = 0.2 + 0.15 (x + 1) + 0.075 (y + 1) through the corners, in
        // normalised device coordinates, y up from the bottom.
        let plane = |[x, y]: [f64; 2]| 0.2 + 0.15 * (x + 1.0) + 0.075 * (y + 1.0);
        let mut wrong = Vec::new();
        for row in 0..height {
            for column in 0..width {
                let pixel = (row * width + column) as usize;
                for (sample, [across, down]) in SAMPLE_OFFSETS.into_iter().enumerate() {
                    let point = [
                        (f64::from(column) + f64::from(across)) / f64::from(width) * 2.0 - 1.0,
                        1.0 - (f64::from(row) + f64::from(down)) / f64::from(height) * 2.0,
                    ];
                    let depth = f64::from(samples[pixel * SAMPLE_OFFSETS.len() + sample]);
                    if (depth - plane(point)).abs() > 1e-5 {
                        wrong.push((column, row, sample, depth, plane(point)));
                    }
                }
            }
        }
        assert!(
            wrong.is_empty(),
            "{} of {} samples lie elsewhere, the first: {:?}",
            wrong.len(),
            samples.len(),
            &wrong[..wrong.len().min(4)]
        );
    }
}
