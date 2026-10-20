//! The main pass: one forward render pass that clears the target and the
//! depth target, then draws each Primitive of each Instance with its
//! Material. ADR 0019 plans it as a depth pre-pass and a lit pass.

use crate::render::bind_groups::{BindGroupDefinition, pipeline_layout};
use crate::render::prepare::Draw;
use crate::render::shaders;
use crate::render::targets::{self, RenderTargets};

/// The main pass's label, and its name in frame stats.
pub(crate) const MAIN_PASS: &str = "main_pass";

/// The main pass's pipeline, which draws into views of one format.
pub struct MainPass {
    pipeline: wgpu::RenderPipeline,
}

impl MainPass {
    /// The pass for `target_format` views, given the layouts of the bind
    /// groups its shader module declares.
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        bind_group_layouts: &[(&BindGroupDefinition, &wgpu::BindGroupLayout)],
    ) -> MainPass {
        let shader = device.create_shader_module(shaders::MAIN.descriptor());
        let layout = pipeline_layout(device, "Main Pass Pipeline Layout", bind_group_layouts);
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Main Pass Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(targets::depth_stencil_state()),
            multisample: wgpu::MultisampleState {
                count: targets::SAMPLE_COUNT,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });
        MainPass { pipeline }
    }

    /// Records the pass into `encoder`: clears `target`, a view in this
    /// pass's format and the render targets' size, and the depth target,
    /// then draws `draws` from the index Arena `index_buffer`, with
    /// `bind_groups` set, timed into `timestamp_writes`.
    #[expect(clippy::too_many_arguments)]
    pub fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        render_targets: &RenderTargets,
        bind_groups: &[(&BindGroupDefinition, &wgpu::BindGroup)],
        index_buffer: &wgpu::Buffer,
        draws: &[Draw],
        timestamp_writes: Option<wgpu::RenderPassTimestampWrites<'_>>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(MAIN_PASS),
            color_attachments: &[
                // This is what @location(0) in the fragment shader targets
                Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: Some(render_targets.depth_attachment()),
            occlusion_query_set: None,
            timestamp_writes,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        for &(definition, bind_group) in bind_groups {
            render_pass.set_bind_group(definition.index, bind_group, &[]);
        }
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        for draw in draws {
            render_pass.draw_indexed(
                draw.indices.clone(),
                draw.base_vertex,
                draw.instance_index..draw.instance_index + 1,
            );
        }
    }
}
