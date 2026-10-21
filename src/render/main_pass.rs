//! The main pass, ADR 0019's two render passes, which draw the culling
//! pass's frame's draw with one indirect call per phase and pipeline, the
//! lit pipeline's drawing both phases: a
//! depth pre-pass that clears the depth target and keeps the nearest depth
//! of phase one's Instances, then a lit pass that first draws phase two's
//! into depth, then both phases' at depth equal, each Meshlet with its
//! record's Material, into the multisampled colour, which it resolves into
//! the target.

use crate::render::bind_groups::{BindGroupDefinition, MESHLET_RECORDS, pipeline_layout};
use crate::render::culling_pass::PHASE_TWO_DRAW_OFFSET;
use crate::render::shaders;
use crate::render::targets::{self, RenderTargets};

/// The depth pre-pass's label, and its name in frame stats.
pub(crate) const DEPTH_PREPASS: &str = "depth_prepass";

/// The lit pass's label, and its name in frame stats.
pub(crate) const LIT_PASS: &str = "lit_pass";

/// The colour the lit pass clears to, linear.
const BACKGROUND: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.2,
    b: 0.3,
    a: 1.0,
};

/// The main pass's pipelines, which draw into views of one format.
pub struct MainPass {
    depth_prepass_pipeline: wgpu::RenderPipeline,
    /// Phase two's draw into depth, inside the lit pass, before it shades.
    lit_pass_depth_pipeline: wgpu::RenderPipeline,
    lit_pipeline: wgpu::RenderPipeline,
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
        let lit_layout = pipeline_layout(device, "Lit Pass Pipeline Layout", &layouts);
        // The depth pipelines' fragment stages read nothing, so they read
        // only the groups their vertex stage sees, as the pre-pass sets them.
        layouts.retain(|&(definition, _)| definition.visible_to(wgpu::ShaderStages::VERTEX));
        let depth_prepass_layout =
            pipeline_layout(device, "Depth Pre-pass Pipeline Layout", &layouts);

        // Every pipeline rasterises alike, so the lit pipeline meets at each
        // sample the depth the depth pipelines kept there.
        let primitive = wgpu::PrimitiveState {
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
        };
        let multisample = wgpu::MultisampleState {
            count: targets::SAMPLE_COUNT,
            mask: !0,
            alpha_to_coverage_enabled: false,
        };
        let depth_pipeline = |label, fragment| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&depth_prepass_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_depth_prepass"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment,
                primitive,
                depth_stencil: Some(targets::depth_stencil_state()),
                multisample,
                multiview_mask: None,
                cache: None,
            })
        };
        let depth_prepass_pipeline = depth_pipeline("Depth Pre-pass Pipeline", None);
        // Every pipeline the lit pass sets names its colour target, so its
        // depth draw names it too, and writes nothing to it.
        let lit_pass_depth_pipeline = depth_pipeline(
            "Lit Pass Depth Pipeline",
            Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_depth"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::empty(),
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
        );
        let lit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Lit Pass Pipeline"),
            layout: Some(&lit_layout),
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
            primitive,
            depth_stencil: Some(targets::depth_equal_state()),
            multisample,
            multiview_mask: None,
            cache: None,
        });
        MainPass {
            depth_prepass_pipeline,
            lit_pass_depth_pipeline,
            lit_pipeline,
            meshlet_records,
        }
    }

    /// Records the depth pre-pass into `encoder`: clears the depth target
    /// and draws the frame's draw as phase one leaves it, its packed indices
    /// `packed_indices` and its indirect arguments `frame_draw`, with the
    /// groups of `bind_groups` its vertex stage reads set, timed into
    /// `timestamp_writes`.
    pub fn record_depth_prepass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        render_targets: &RenderTargets,
        bind_groups: &[(&BindGroupDefinition, &wgpu::BindGroup)],
        packed_indices: &wgpu::Buffer,
        frame_draw: &wgpu::Buffer,
        timestamp_writes: Option<wgpu::RenderPassTimestampWrites<'_>>,
    ) {
        let mut depth_prepass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(DEPTH_PREPASS),
            color_attachments: &[],
            depth_stencil_attachment: Some(render_targets.depth_attachment()),
            occlusion_query_set: None,
            timestamp_writes,
            multiview_mask: None,
        });
        depth_prepass.set_pipeline(&self.depth_prepass_pipeline);
        self.bind(
            &mut depth_prepass,
            bind_groups
                .iter()
                .copied()
                .filter(|&(definition, _)| definition.visible_to(wgpu::ShaderStages::VERTEX)),
            packed_indices,
        );
        depth_prepass.draw_indexed_indirect(frame_draw, 0);
    }

    /// Records the lit pass into `encoder`: clears the multisampled colour,
    /// draws phase two's draw into the depth target, then the frame's draw of
    /// both phases at depth equal, the buffers as `record_depth_prepass`
    /// takes them, with `bind_groups` set, and resolves the colour into
    /// `target`, a view in this pass's format and the render targets' size,
    /// timed into `timestamp_writes`.
    #[expect(clippy::too_many_arguments)]
    pub fn record_lit_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        render_targets: &RenderTargets,
        bind_groups: &[(&BindGroupDefinition, &wgpu::BindGroup)],
        packed_indices: &wgpu::Buffer,
        frame_draw: &wgpu::Buffer,
        timestamp_writes: Option<wgpu::RenderPassTimestampWrites<'_>>,
    ) {
        let mut lit_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(LIT_PASS),
            color_attachments: &[
                // This is what @location(0) in the fragment shader targets
                Some(render_targets.color_attachment(target, BACKGROUND)),
            ],
            depth_stencil_attachment: Some(render_targets.final_depth_attachment()),
            occlusion_query_set: None,
            timestamp_writes,
            multiview_mask: None,
        });
        self.bind(&mut lit_pass, bind_groups.iter().copied(), packed_indices);
        lit_pass.set_pipeline(&self.lit_pass_depth_pipeline);
        lit_pass.draw_indexed_indirect(frame_draw, PHASE_TWO_DRAW_OFFSET);
        lit_pass.set_pipeline(&self.lit_pipeline);
        lit_pass.draw_indexed_indirect(frame_draw, 0);
    }

    /// Sets `bind_groups`, this pass's own group and the frame's draw's
    /// `packed_indices` into `render_pass`.
    fn bind<'a>(
        &self,
        render_pass: &mut wgpu::RenderPass<'_>,
        bind_groups: impl IntoIterator<Item = (&'a BindGroupDefinition, &'a wgpu::BindGroup)>,
        packed_indices: &wgpu::Buffer,
    ) {
        for (definition, bind_group) in bind_groups {
            render_pass.set_bind_group(definition.index, bind_group, &[]);
        }
        render_pass.set_bind_group(MESHLET_RECORDS.index, &self.meshlet_records, &[]);
        render_pass.set_index_buffer(packed_indices.slice(..), wgpu::IndexFormat::Uint32);
    }
}
