//! The tonemapping pass: one triangle over the target that reads each pixel
//! of the HDR target and writes the pixel's display colour.

use crate::render::bind_groups::{TONEMAPPING_INPUT, pipeline_layout};
use crate::render::shaders;
use crate::render::targets::RenderTargets;

/// The tonemapping pass's label, and its name in frame stats.
pub(crate) const TONEMAPPING_PASS: &str = "tonemapping_pass";

/// The pass's pipeline, which draws into views of one format, and its bind
/// group.
pub struct TonemappingPass {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    /// Its group 3 (`TONEMAPPING_INPUT`), built from the render targets by
    /// `bind_render_targets`.
    input: wgpu::BindGroup,
}

impl TonemappingPass {
    /// The pass for `target_format` views, reading the HDR target of
    /// `render_targets`.
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        render_targets: &RenderTargets,
    ) -> TonemappingPass {
        let shader = device.create_shader_module(shaders::TONEMAPPING.descriptor());
        let layout = TONEMAPPING_INPUT.layout(device);
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Tonemapping Pipeline"),
            layout: Some(&pipeline_layout(
                device,
                "Tonemapping Pipeline Layout",
                &[(&TONEMAPPING_INPUT, &layout)],
            )),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_full_screen"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_tonemap"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let input = Self::input_bind_group(device, &layout, render_targets);
        TonemappingPass {
            pipeline,
            layout,
            input,
        }
    }

    /// Builds the bind group that reads `render_targets`: at creation, and
    /// after every resize, which creates them again.
    pub fn bind_render_targets(&mut self, device: &wgpu::Device, render_targets: &RenderTargets) {
        self.input = Self::input_bind_group(device, &self.layout, render_targets);
    }

    /// The bind group that reads `render_targets`' HDR target.
    fn input_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        render_targets: &RenderTargets,
    ) -> wgpu::BindGroup {
        TONEMAPPING_INPUT.bind_group(
            device,
            layout,
            [(
                "hdr_target",
                wgpu::BindingResource::TextureView(render_targets.hdr()),
            )],
        )
    }

    /// Records the pass into `encoder`: every pixel of `target`, a view in
    /// this pass's format and the render targets' size, takes its display
    /// colour, timed into `timestamp_writes`.
    pub fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        timestamp_writes: Option<wgpu::RenderPassTimestampWrites<'_>>,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(TONEMAPPING_PASS),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
                // The triangle covers every pixel; a clear spares the load.
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(TONEMAPPING_INPUT.index, &self.input, &[]);
        pass.draw(0..3, 0..1);
    }
}
