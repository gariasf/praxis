//! The shadow mask pass: a compute pass that traces, at each pixel, one hard
//! shadow ray toward each light that reaches the surface its nearest sample
//! shows, against the static and dynamic TLAS (ADR 0020), and keeps a bit
//! per light its ray found hidden in the shadow mask, which the lit pass
//! reads. Where the pixel's other samples show another surface, it traces
//! from that one too, into the mask's second layer, and marks the samples
//! that show it, so that each fragment of an edge pixel reads its own
//! surface's bits. Where a directional light's verdict at a pixel's nearest
//! surface differs from a 4-neighbour's, a shadow edge, it traces that light
//! again from each other sample that shows the surface, and keeps each
//! sample's verdict, so the edge resolves as MSAA resolves geometry's. It
//! runs once both culling phases' Instances are in the depth target.

use crate::render::acceleration_structures::{DynamicTlas, StaticTlas};
use crate::render::bind_groups::{SHADOW_MASK_BUILD, SHADOW_RAY_SCENE, pipeline_layout};
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

/// Directional lights, the first in the light buffer, whose shadow edges the
/// pass traces again from each sample: the shadow mask samples hold each
/// one's verdict at every sample, SAMPLE_COUNT bits a light, above the
/// samples that show the second surface. Any other keeps one ray per pixel.
pub(crate) const SHADOW_EDGE_LIGHTS: u32 = 7;

// The pass loads each pixel's samples by index (`shadow_mask.wgsl`).
const _: () = assert!(SAMPLE_COUNT == 4);

// Each sample's verdicts and the second surface's samples fill the shadow
// mask samples' word; the edge lights' bits fit a byte of a pixel's
// directional verdicts (`shadow_mask.wgsl`).
const _: () = assert!(SAMPLE_COUNT * (1 + SHADOW_EDGE_LIGHTS) <= u32::BITS);
const _: () = assert!(SHADOW_EDGE_LIGHTS <= 8);

// Each pixel's word, two halves of 32 bits, holds the Froxel slice its bits
// follow above them, in the second half.
const _: () = assert!(FROXEL_SLICES.is_power_of_two());
const _: () = assert!(SHADOW_MASK_LIGHTS > u32::BITS);
const _: () = assert!(SHADOW_MASK_LIGHTS + FROXEL_SLICES.ilog2() == 2 * u32::BITS);

/// The pass's pipelines, its group 2 with the TLAS it traces, and its group
/// 3 with the buffers it reads.
pub struct ShadowMaskPass {
    /// Traces each pixel's nearest surface and lists its second, if any.
    nearest_surfaces_pipeline: wgpu::ComputePipeline,
    /// Traces the listed second surfaces.
    second_surfaces_pipeline: wgpu::ComputePipeline,
    /// Lists the pixels at a directional light's shadow edge.
    list_shadow_edges_pipeline: wgpu::ComputePipeline,
    /// Traces the listed shadow edges again from each sample.
    trace_shadow_edges_pipeline: wgpu::ComputePipeline,
    shadow_ray_scene: wgpu::BindGroup,
    layout: wgpu::BindGroupLayout,
    /// The camera, the lights' counts, the lights and the Froxel lists.
    buffers: [(&'static str, wgpu::Buffer); 4],
    /// The lights that reach a surface of a pixel past its bits, summed over
    /// the surfaces of every pixel:
    /// cleared before the pass, copied to the GPU stats readback after it.
    shadow_mask_lights_overflowed: wgpu::Buffer,
    /// The second dispatch's workgroups, which the first writes: cleared
    /// before the pass.
    second_surface_dispatch: wgpu::Buffer,
    /// The last dispatch's workgroups, which the third writes: cleared
    /// before the pass.
    shadow_edge_dispatch: wgpu::Buffer,
    /// Bound in place of either while a dispatch reads it, as no group
    /// bound then may write it.
    stand_in_dispatch: wgpu::Buffer,
    /// The render targets' lists of second surfaces and of shadow edges,
    /// whose counts are cleared before the pass.
    second_surfaces: wgpu::Buffer,
    shadow_edges: wgpu::Buffer,
    /// Built from the render targets by `bind_groups`: the first and third
    /// dispatches', the second's, with the stand-in for its dispatch, and
    /// the last's, with the stand-in for its own.
    nearest_surfaces_bind_group: wgpu::BindGroup,
    second_surfaces_bind_group: wgpu::BindGroup,
    sample_verdicts_bind_group: wgpu::BindGroup,
    /// The targets' size, in pixels.
    size: (u32, u32),
}

impl ShadowMaskPass {
    /// The pass, tracing against `static_tlas` and `dynamic_tlas`, reading the Renderer's `camera`, `light` and `lights`
    /// buffers and the Froxel pass's `froxel_lists`, and writing the shadow
    /// mask of `render_targets`, with its Froxel lookups as `switches` set.
    pub fn new(
        device: &wgpu::Device,
        static_tlas: &StaticTlas,
        dynamic_tlas: &DynamicTlas,
        buffers: [(&'static str, &wgpu::Buffer); 4],
        render_targets: &RenderTargets,
        switches: RenderSwitches,
    ) -> ShadowMaskPass {
        let shadow_ray_scene_layout = SHADOW_RAY_SCENE.layout(device);
        let shadow_ray_scene = SHADOW_RAY_SCENE.bind_group(
            device,
            &shadow_ray_scene_layout,
            [
                ("static_tlas", static_tlas.binding()),
                ("dynamic_tlas", dynamic_tlas.binding()),
            ],
        );
        let layout = SHADOW_MASK_BUILD.layout(device);
        let shader = device.create_shader_module(shaders::SHADOW_MASK.descriptor());
        let pipeline_layout = pipeline_layout(
            device,
            "Shadow Mask Pipeline Layout",
            &[
                (&SHADOW_RAY_SCENE, &shadow_ray_scene_layout),
                (&SHADOW_MASK_BUILD, &layout),
            ],
        );
        let constants = switches.constants();
        let pipeline = |entry_point| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry_point),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some(entry_point),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &constants,
                    ..wgpu::PipelineCompilationOptions::default()
                },
                cache: None,
            })
        };
        let buffers = buffers.map(|(name, buffer)| (name, buffer.clone()));
        let shadow_mask_lights_overflowed = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Mask Overflow Buffer"),
            size: size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let dispatch = |label, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: size_of::<wgpu::util::DispatchIndirectArgs>() as u64,
                usage: wgpu::BufferUsages::STORAGE | usage,
                mapped_at_creation: false,
            })
        };
        let second_surface_dispatch = dispatch(
            "Second Surface Dispatch Buffer",
            wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        );
        let shadow_edge_dispatch = dispatch(
            "Shadow Edge Dispatch Buffer",
            wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        );
        let stand_in_dispatch = dispatch("Stand-in Dispatch Buffer", wgpu::BufferUsages::empty());
        let [
            nearest_surfaces_bind_group,
            second_surfaces_bind_group,
            sample_verdicts_bind_group,
        ] = Self::bind_groups(
            device,
            &layout,
            &buffers,
            &shadow_mask_lights_overflowed,
            [
                &second_surface_dispatch,
                &shadow_edge_dispatch,
                &stand_in_dispatch,
            ],
            render_targets,
        );
        let size = render_targets.shadow_mask().texture().size();
        ShadowMaskPass {
            nearest_surfaces_pipeline: pipeline("build_shadow_mask"),
            second_surfaces_pipeline: pipeline("trace_second_surfaces"),
            list_shadow_edges_pipeline: pipeline("list_shadow_edges"),
            trace_shadow_edges_pipeline: pipeline("trace_shadow_edges"),
            shadow_ray_scene,
            layout,
            buffers,
            shadow_mask_lights_overflowed,
            second_surface_dispatch,
            shadow_edge_dispatch,
            stand_in_dispatch,
            second_surfaces: render_targets.second_surfaces().clone(),
            shadow_edges: render_targets.shadow_edges().clone(),
            nearest_surfaces_bind_group,
            second_surfaces_bind_group,
            sample_verdicts_bind_group,
            size: (size.width, size.height),
        }
    }

    /// Builds the bind groups that read and write `render_targets`: at
    /// creation, and after every resize, which creates them again.
    pub fn bind_render_targets(&mut self, device: &wgpu::Device, render_targets: &RenderTargets) {
        [
            self.nearest_surfaces_bind_group,
            self.second_surfaces_bind_group,
            self.sample_verdicts_bind_group,
        ] = Self::bind_groups(
            device,
            &self.layout,
            &self.buffers,
            &self.shadow_mask_lights_overflowed,
            [
                &self.second_surface_dispatch,
                &self.shadow_edge_dispatch,
                &self.stand_in_dispatch,
            ],
            render_targets,
        );
        self.second_surfaces = render_targets.second_surfaces().clone();
        self.shadow_edges = render_targets.shadow_edges().clone();
        let size = render_targets.shadow_mask().texture().size();
        self.size = (size.width, size.height);
    }

    /// The dispatches' groups, which differ in the `stand_in` each binds for
    /// the dispatch buffer it reads, `second_surface_dispatch` or
    /// `shadow_edge_dispatch`.
    fn bind_groups(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        buffers: &[(&'static str, wgpu::Buffer); 4],
        shadow_mask_lights_overflowed: &wgpu::Buffer,
        [second_surface_dispatch, shadow_edge_dispatch, stand_in]: [&wgpu::Buffer; 3],
        render_targets: &RenderTargets,
    ) -> [wgpu::BindGroup; 3] {
        [
            [second_surface_dispatch, shadow_edge_dispatch],
            [stand_in, shadow_edge_dispatch],
            [second_surface_dispatch, stand_in],
        ]
        .map(|dispatches| {
            Self::bind_group(
                device,
                layout,
                buffers,
                shadow_mask_lights_overflowed,
                dispatches,
                render_targets,
            )
        })
    }

    fn bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        buffers: &[(&'static str, wgpu::Buffer); 4],
        shadow_mask_lights_overflowed: &wgpu::Buffer,
        [second_surface_dispatch, shadow_edge_dispatch]: [&wgpu::Buffer; 2],
        render_targets: &RenderTargets,
    ) -> wgpu::BindGroup {
        let own = [
            (
                "shadow_mask_lights_overflowed",
                shadow_mask_lights_overflowed.as_entire_binding(),
            ),
            (
                "second_surfaces",
                render_targets.second_surfaces().as_entire_binding(),
            ),
            (
                "second_surface_dispatch",
                second_surface_dispatch.as_entire_binding(),
            ),
            (
                "directional_verdicts",
                render_targets.directional_verdicts().as_entire_binding(),
            ),
            (
                "shadow_edges",
                render_targets.shadow_edges().as_entire_binding(),
            ),
            (
                "shadow_edge_dispatch",
                shadow_edge_dispatch.as_entire_binding(),
            ),
            (
                "depth_target",
                wgpu::BindingResource::TextureView(render_targets.depth()),
            ),
            (
                "surface_normals",
                wgpu::BindingResource::TextureView(render_targets.surface_normals()),
            ),
            (
                "shadow_mask",
                wgpu::BindingResource::TextureView(render_targets.shadow_mask()),
            ),
            (
                "shadow_mask_samples",
                wgpu::BindingResource::TextureView(render_targets.shadow_mask_samples()),
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

    /// Records the pass into `encoder`, timed into `timestamp_writes`.
    pub fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        timestamp_writes: Option<wgpu::ComputePassTimestampWrites<'_>>,
    ) {
        encoder.clear_buffer(&self.shadow_mask_lights_overflowed, 0, None);
        encoder.clear_buffer(&self.second_surface_dispatch, 0, None);
        encoder.clear_buffer(&self.second_surfaces, 0, Some(size_of::<u32>() as u64));
        encoder.clear_buffer(&self.shadow_edge_dispatch, 0, None);
        encoder.clear_buffer(&self.shadow_edges, 0, Some(size_of::<u32>() as u64));
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(SHADOW_MASK_PASS),
            timestamp_writes,
        });
        pass.set_pipeline(&self.nearest_surfaces_pipeline);
        pass.set_bind_group(SHADOW_RAY_SCENE.index, &self.shadow_ray_scene, &[]);
        pass.set_bind_group(
            SHADOW_MASK_BUILD.index,
            &self.nearest_surfaces_bind_group,
            &[],
        );
        let (width, height) = self.size;
        pass.dispatch_workgroups(
            width.div_ceil(SHADOW_MASK_TILE_SIZE),
            height.div_ceil(SHADOW_MASK_TILE_SIZE),
            1,
        );
        pass.set_pipeline(&self.second_surfaces_pipeline);
        pass.set_bind_group(
            SHADOW_MASK_BUILD.index,
            &self.second_surfaces_bind_group,
            &[],
        );
        pass.dispatch_workgroups_indirect(&self.second_surface_dispatch, 0);
        pass.set_pipeline(&self.list_shadow_edges_pipeline);
        pass.set_bind_group(
            SHADOW_MASK_BUILD.index,
            &self.nearest_surfaces_bind_group,
            &[],
        );
        pass.dispatch_workgroups(
            width.div_ceil(SHADOW_MASK_TILE_SIZE),
            height.div_ceil(SHADOW_MASK_TILE_SIZE),
            1,
        );
        pass.set_pipeline(&self.trace_shadow_edges_pipeline);
        pass.set_bind_group(
            SHADOW_MASK_BUILD.index,
            &self.sample_verdicts_bind_group,
            &[],
        );
        pass.dispatch_workgroups_indirect(&self.shadow_edge_dispatch, 0);
    }

    /// The lights the frame's mask had no bit for at the pixels' surfaces
    /// they reach, a u32.
    pub fn shadow_mask_lights_overflowed(&self) -> &wgpu::Buffer {
        &self.shadow_mask_lights_overflowed
    }

    /// Bytes of its overflow count and dispatch buffers; the mask, the
    /// directional verdicts and the lists are render targets.
    pub fn gpu_memory_bytes(&self) -> u64 {
        self.shadow_mask_lights_overflowed.size()
            + self.second_surface_dispatch.size()
            + self.shadow_edge_dispatch.size()
            + self.stand_in_dispatch.size()
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
        let render_targets = RenderTargets::new(&context.device, width, height);
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
