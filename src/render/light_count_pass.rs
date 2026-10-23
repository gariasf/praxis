//! The light count pass: with the `count_point_lights` Render switch on, a
//! compute pass after the frame's others that counts, at each pixel where
//! the frame drew a surface at the first sample, the point lights its
//! Froxel lists and those whose range reaches that surface, into the light
//! counts the GPU stats read back (`LightCounts`). It reads the depth the
//! main pass keeps then, rather than counting in the lit pass, whose side
//! effects would make the GPU run it for fragments its depth test rejects.

use crate::render::bind_groups::{LIGHT_COUNT_INPUT, pipeline_layout};
use crate::render::froxel_pass::FroxelPass;
use crate::render::shaders;
use crate::render::targets::RenderTargets;

/// Pixels along each side of the tile each workgroup counts.
pub(crate) const LIGHT_COUNT_TILE_SIZE: u32 = 8;

/// The pass's pipeline, and its bind group of the buffers it reads and
/// writes with the depth target.
pub struct LightCountPass {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    buffers: LightCountBuffers,
    /// Built from the render targets by `bind_render_targets`.
    bind_group: wgpu::BindGroup,
    /// The depth target's size.
    size: wgpu::Extent3d,
}

/// The buffers the pass reads and writes, kept to rebuild its bind group:
/// the Renderer's camera and light buffer, and the Froxel pass's lists and
/// light counts.
struct LightCountBuffers {
    camera: wgpu::Buffer,
    lights: wgpu::Buffer,
    froxel_lists: wgpu::Buffer,
    light_counts: wgpu::Buffer,
}

impl LightCountPass {
    /// The pass, counting the lights of `lights`, seen by `camera`, the
    /// Renderer's buffers, through `froxel_pass`'s lists into its light
    /// counts, at the depth of `render_targets`.
    pub fn new(
        device: &wgpu::Device,
        camera: &wgpu::Buffer,
        lights: &wgpu::Buffer,
        froxel_pass: &FroxelPass,
        render_targets: &RenderTargets,
    ) -> LightCountPass {
        let layout = LIGHT_COUNT_INPUT.layout(device);
        let shader = device.create_shader_module(shaders::LIGHT_COUNT.descriptor());
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("count_point_lights"),
            layout: Some(&pipeline_layout(
                device,
                "Light Count Pass Pipeline Layout",
                &[(&LIGHT_COUNT_INPUT, &layout)],
            )),
            module: &shader,
            entry_point: Some("count_point_lights"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let buffers = LightCountBuffers {
            camera: camera.clone(),
            lights: lights.clone(),
            froxel_lists: froxel_pass.froxel_lists().clone(),
            light_counts: froxel_pass.light_counts().clone(),
        };
        let bind_group = bind_group(device, &layout, &buffers, render_targets);
        LightCountPass {
            pipeline,
            layout,
            buffers,
            bind_group,
            size: render_targets.depth().texture().size(),
        }
    }

    /// Builds the bind group that reads `render_targets` again, as `new`
    /// builds it: after every resize, which creates them again.
    pub fn bind_render_targets(&mut self, device: &wgpu::Device, render_targets: &RenderTargets) {
        self.bind_group = bind_group(device, &self.layout, &self.buffers, render_targets);
        self.size = render_targets.depth().texture().size();
    }

    /// Records the pass into `encoder`: a workgroup per tile of the depth
    /// target.
    pub fn record(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("light_count_pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(LIGHT_COUNT_INPUT.index, &self.bind_group, &[]);
        pass.dispatch_workgroups(
            self.size.width.div_ceil(LIGHT_COUNT_TILE_SIZE),
            self.size.height.div_ceil(LIGHT_COUNT_TILE_SIZE),
            1,
        );
    }
}

/// The pass's group 3 (`LIGHT_COUNT_INPUT`) of `layout`: `buffers`, and the
/// depth target of `render_targets`.
fn bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffers: &LightCountBuffers,
    render_targets: &RenderTargets,
) -> wgpu::BindGroup {
    LIGHT_COUNT_INPUT.bind_group(
        device,
        layout,
        [
            ("camera", buffers.camera.as_entire_binding()),
            ("lights", buffers.lights.as_entire_binding()),
            ("froxel_lists", buffers.froxel_lists.as_entire_binding()),
            ("light_counts", buffers.light_counts.as_entire_binding()),
            (
                "depth",
                wgpu::BindingResource::TextureView(render_targets.depth()),
            ),
        ],
    )
}
