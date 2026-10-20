//! The main pass: one forward render pass that clears the target and the
//! depth target, then draws the culling pass's frame's draw with one
//! indirect call, each Meshlet with its record's Material. ADR 0019 plans it
//! as a depth pre-pass and a lit pass.

use crate::render::bind_groups::{BindGroupDefinition, MESHLET_RECORDS, pipeline_layout};
use crate::render::shaders;
use crate::render::targets::{self, RenderTargets};

/// The main pass's label, and its name in frame stats.
pub(crate) const MAIN_PASS: &str = "main_pass";

/// The main pass's pipeline, which draws into views of one format.
pub struct MainPass {
    pipeline: wgpu::RenderPipeline,
    /// Its group 3 (`MESHLET_RECORDS`).
    meshlet_records: wgpu::BindGroup,
}

impl MainPass {
    /// The pass for `target_format` views, given the layouts of the shared
    /// bind groups its shader module declares, which draws the Meshlet
    /// records `meshlet_records` holds.
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        bind_group_layouts: &[(&BindGroupDefinition, &wgpu::BindGroupLayout)],
        meshlet_records: &wgpu::Buffer,
    ) -> MainPass {
        let meshlet_records_layout = MESHLET_RECORDS.layout(device);
        let meshlet_records = MESHLET_RECORDS.bind_group(
            device,
            &meshlet_records_layout,
            [("meshlet_records", meshlet_records.as_entire_binding())],
        );
        let shader = device.create_shader_module(shaders::MAIN.descriptor());
        let mut layouts = bind_group_layouts.to_vec();
        layouts.push((&MESHLET_RECORDS, &meshlet_records_layout));
        let layout = pipeline_layout(device, "Main Pass Pipeline Layout", &layouts);
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
        MainPass {
            pipeline,
            meshlet_records,
        }
    }

    /// Records the pass into `encoder`: clears `target`, a view in this
    /// pass's format and the render targets' size, and the depth target,
    /// then draws the frame's draw, its packed indices `packed_indices` and
    /// its indirect arguments `frame_draw`, with `bind_groups` set, timed
    /// into `timestamp_writes`.
    #[expect(clippy::too_many_arguments)]
    pub fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        render_targets: &RenderTargets,
        bind_groups: &[(&BindGroupDefinition, &wgpu::BindGroup)],
        packed_indices: &wgpu::Buffer,
        frame_draw: &wgpu::Buffer,
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
        render_pass.set_bind_group(MESHLET_RECORDS.index, &self.meshlet_records, &[]);
        render_pass.set_index_buffer(packed_indices.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed_indirect(frame_draw, 0);
    }
}
