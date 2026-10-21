//! The depth pyramid pass: a compute pass that builds the render targets'
//! depth pyramid from the depth the depth pre-pass stored, a level at a
//! time, each texel the minimum depth, the farthest, of the texels below
//! whose share of the screen overlaps its own (ADRs 0015, 0019).

use crate::render::bind_groups::{DEPTH_PYRAMID_LEVEL, pipeline_layout};
use crate::render::shaders;
use crate::render::targets::{RenderTargets, SAMPLE_COUNT};

/// The depth pyramid pass's label, and its name in frame stats.
pub(crate) const DEPTH_PYRAMID_PASS: &str = "depth_pyramid_pass";

/// Texels along each side of the tile of a level each workgroup reduces.
pub(crate) const DEPTH_PYRAMID_TILE_SIZE: u32 = 8;

// Level 0 loads each pixel's samples by index, as Metal's compiler does not
// unroll a loop over them (`depth_pyramid.wgsl`).
const _: () = assert!(SAMPLE_COUNT == 4);

/// The pass's pipelines, level 0's and the later levels', and a bind group
/// per level with its size.
pub struct DepthPyramidPass {
    reduce_depth: wgpu::ComputePipeline,
    reduce_level: wgpu::ComputePipeline,
    reduce_depth_layout: wgpu::BindGroupLayout,
    reduce_level_layout: wgpu::BindGroupLayout,
    /// Built from the render targets by `bind_render_targets`.
    levels: Vec<(wgpu::BindGroup, wgpu::Extent3d)>,
}

impl DepthPyramidPass {
    /// The pass, building the pyramid of `render_targets`.
    pub fn new(device: &wgpu::Device, render_targets: &RenderTargets) -> DepthPyramidPass {
        let shader = device.create_shader_module(shaders::DEPTH_PYRAMID.descriptor());
        // Level 0 reduces the depth target, each later level the one below.
        let reduce_depth_layout = DEPTH_PYRAMID_LEVEL.layout_of(device, &["level", "depth"]);
        let reduce_level_layout = DEPTH_PYRAMID_LEVEL.layout_of(device, &["level", "level_below"]);
        let pipeline = |entry_point, layout| {
            let label = "Depth Pyramid Pipeline Layout";
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry_point),
                layout: Some(&pipeline_layout(
                    device,
                    label,
                    &[(&DEPTH_PYRAMID_LEVEL, layout)],
                )),
                module: &shader,
                entry_point: Some(entry_point),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };
        let mut pass = DepthPyramidPass {
            reduce_depth: pipeline("reduce_depth", &reduce_depth_layout),
            reduce_level: pipeline("reduce_level", &reduce_level_layout),
            reduce_depth_layout,
            reduce_level_layout,
            levels: Vec::new(),
        };
        pass.bind_render_targets(device, render_targets);
        pass
    }

    /// Builds the bind groups that read and write `render_targets`: at
    /// creation, and after every resize, which creates them again.
    pub fn bind_render_targets(&mut self, device: &wgpu::Device, render_targets: &RenderTargets) {
        let levels = render_targets.depth_pyramid_levels();
        let bind = |index: usize, layout, reduced: (&'static str, &wgpu::TextureView)| {
            let level = &levels[index];
            let bind_group = DEPTH_PYRAMID_LEVEL.bind_group(
                device,
                layout,
                [
                    ("level", wgpu::BindingResource::TextureView(level)),
                    (reduced.0, wgpu::BindingResource::TextureView(reduced.1)),
                ],
            );
            let size = level
                .texture()
                .size()
                .mip_level_size(index as u32, wgpu::TextureDimension::D2);
            (bind_group, size)
        };
        let reduce_depth = bind(
            0,
            &self.reduce_depth_layout,
            ("depth", render_targets.depth()),
        );
        self.levels = std::iter::once(reduce_depth)
            .chain((1..levels.len()).map(|index| {
                bind(
                    index,
                    &self.reduce_level_layout,
                    ("level_below", &levels[index - 1]),
                )
            }))
            .collect();
    }

    /// Records the pass into `encoder`, timed into `timestamp_writes`.
    pub fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        timestamp_writes: Option<wgpu::ComputePassTimestampWrites<'_>>,
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(DEPTH_PYRAMID_PASS),
            timestamp_writes,
        });
        // Level 0 reduces the depth target, each later level the one below.
        for (index, (bind_group, size)) in self.levels.iter().enumerate() {
            pass.set_pipeline(if index == 0 {
                &self.reduce_depth
            } else {
                &self.reduce_level
            });
            pass.set_bind_group(DEPTH_PYRAMID_LEVEL.index, bind_group, &[]);
            pass.dispatch_workgroups(
                size.width.div_ceil(DEPTH_PYRAMID_TILE_SIZE),
                size.height.div_ceil(DEPTH_PYRAMID_TILE_SIZE),
                1,
            );
        }
    }
}
