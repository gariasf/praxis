//! The main pass, ADR 0019's render passes, which draw the culling pass's
//! frame's draw with one indirect call per phase and pipeline, the lit
//! pipeline's drawing both phases: a depth pre-pass that clears the depth
//! target and keeps the nearest depth of phase one's Instances, then phase
//! two's drawn into depth, then a lit pass that draws both phases' at depth
//! equal, each Meshlet with its record's Material, into the multisampled
//! colour, which it resolves into the HDR target. With shadow rays on, phase
//! two's depth is a pass of its own, so the shadow mask pass reads every
//! Instance's depth before the lit pass reads the mask; with them off, the
//! lit pass draws it first. With the visibility pre-pass Render switch on,
//! phase two's depth is a pass of its own too, a visibility ID pass then
//! draws both phases' at depth equal and writes the triangle each sample
//! shows, and the lit pass draws the compacted draw the visibility pass
//! builds from those IDs.

use crate::render::bind_groups::{BindGroupDefinition, MAIN_PASS_INPUTS, pipeline_layout};
use crate::render::culling_pass::PHASE_TWO_DRAW_OFFSET;
use crate::render::renderer::RenderSwitches;
use crate::render::shaders;
use crate::render::targets::{self, RenderTargets};

/// The depth pre-pass's label, and its name in frame stats.
pub(crate) const DEPTH_PREPASS: &str = "depth_prepass";

/// Phase two's depth pass's label, and its name in frame stats.
pub(crate) const PHASE_TWO_DEPTH_PASS: &str = "phase_two_depth_pass";

/// The visibility ID pass's label, and its name in frame stats.
pub(crate) const VISIBILITY_ID_PASS: &str = "visibility_id_pass";

/// The lit pass's label, and its name in frame stats.
pub(crate) const LIT_PASS: &str = "lit_pass";

/// The main pass's pipelines.
pub struct MainPass {
    depth_prepass_pipeline: wgpu::RenderPipeline,
    /// With shadow rays off, phase two's draw into depth, inside the lit
    /// pass, before it shades.
    lit_pass_depth_pipeline: wgpu::RenderPipeline,
    /// With the visibility pre-pass Render switch on, the pipeline that
    /// writes each sample's triangle at depth equal.
    visibility_id_pipeline: Option<wgpu::RenderPipeline>,
    lit_pipeline: wgpu::RenderPipeline,
    inputs_layout: wgpu::BindGroupLayout,
    meshlet_records_buffer: wgpu::Buffer,
    /// Its group 3 (`MAIN_PASS_INPUTS`), built from the render targets by
    /// `bind_render_targets`.
    inputs: wgpu::BindGroup,
    /// Whether the lit pass draws phase two's draw into depth before it
    /// shades: with shadow rays and the visibility pre-pass both off, since
    /// either reads every Instance's depth before the lit pass.
    phase_two_in_lit_pass: bool,
    /// Whether the lit pass keeps the depth it ends with, for the light
    /// count pass (the `count_point_lights` Render switch).
    keeps_depth: bool,
}

impl MainPass {
    /// The pass, given the layouts of the shared bind groups its shader
    /// module declares, which draws the Meshlet records `meshlet_records`
    /// holds, lit through the shadow mask of `render_targets`, its lit
    /// pipeline as `switches` set.
    pub fn new(
        device: &wgpu::Device,
        bind_group_layouts: &[(&BindGroupDefinition, &wgpu::BindGroupLayout)],
        meshlet_records: &wgpu::Buffer,
        render_targets: &RenderTargets,
        switches: RenderSwitches,
    ) -> MainPass {
        let inputs_layout = MAIN_PASS_INPUTS.layout(device);
        let meshlet_records_buffer = meshlet_records.clone();
        let inputs = Self::inputs_bind_group(
            device,
            &inputs_layout,
            &meshlet_records_buffer,
            render_targets,
        );
        // The visibility ID pass's entry point needs the adapter's primitive
        // index, which its module alone enables.
        let shader = device.create_shader_module(if switches.visibility_prepass.is_on() {
            shaders::MAIN_VISIBILITY.descriptor()
        } else {
            shaders::MAIN.descriptor()
        });
        let mut layouts = bind_group_layouts.to_vec();
        layouts.push((&MAIN_PASS_INPUTS, &inputs_layout));
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
        let depth_pipeline =
            |label: &str, fragment: Option<wgpu::FragmentState<'_>>, depth_stencil| {
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
                    depth_stencil: Some(depth_stencil),
                    multisample,
                    multiview_mask: None,
                    cache: None,
                })
            };
        let depth_prepass_pipeline = depth_pipeline(
            "Depth Pre-pass Pipeline",
            None,
            targets::depth_stencil_state(),
        );
        // Every pipeline the lit pass sets names its colour target, so its
        // depth draw names it too, and writes nothing to it.
        let lit_pass_depth_pipeline = depth_pipeline(
            "Lit Pass Depth Pipeline",
            Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_depth"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: targets::HDR_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::empty(),
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            targets::depth_stencil_state(),
        );
        // At depth equal, as the lit pipeline draws, so the IDs name the
        // triangles the lit pass shows.
        let visibility_id_pipeline = switches.visibility_prepass.is_on().then(|| {
            depth_pipeline(
                "Visibility ID Pipeline",
                Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_visibility"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: targets::VISIBILITY_ID_FORMAT,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                targets::depth_equal_state(),
            )
        });
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
                    format: targets::HDR_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &switches.constants(),
                    ..wgpu::PipelineCompilationOptions::default()
                },
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
            visibility_id_pipeline,
            lit_pipeline,
            inputs_layout,
            meshlet_records_buffer,
            inputs,
            phase_two_in_lit_pass: !switches.shadow_rays && !switches.visibility_prepass.is_on(),
            keeps_depth: switches.count_point_lights,
        }
    }

    /// Builds the bind group that reads `render_targets`: at creation, and
    /// after every resize, which creates them again.
    pub fn bind_render_targets(&mut self, device: &wgpu::Device, render_targets: &RenderTargets) {
        self.inputs = Self::inputs_bind_group(
            device,
            &self.inputs_layout,
            &self.meshlet_records_buffer,
            render_targets,
        );
    }

    /// Group 3 of `layout`: `meshlet_records`, and the shadow mask of
    /// `render_targets`.
    fn inputs_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        meshlet_records: &wgpu::Buffer,
        render_targets: &RenderTargets,
    ) -> wgpu::BindGroup {
        MAIN_PASS_INPUTS.bind_group(
            device,
            layout,
            [
                ("meshlet_records", meshlet_records.as_entire_binding()),
                (
                    "shadow_mask",
                    wgpu::BindingResource::TextureView(render_targets.shadow_mask()),
                ),
            ],
        )
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

    /// Records phase two's depth pass into `encoder`, with shadow rays or
    /// the visibility pre-pass on: draws phase two's draw into the depth
    /// target, which it loads and stores for the passes that read every
    /// Instance's depth, the buffers and groups as `record_depth_prepass`
    /// takes them, timed into `timestamp_writes`.
    pub fn record_phase_two_depth(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        render_targets: &RenderTargets,
        bind_groups: &[(&BindGroupDefinition, &wgpu::BindGroup)],
        packed_indices: &wgpu::Buffer,
        frame_draw: &wgpu::Buffer,
        timestamp_writes: Option<wgpu::RenderPassTimestampWrites<'_>>,
    ) {
        let mut depth_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(PHASE_TWO_DEPTH_PASS),
            color_attachments: &[],
            depth_stencil_attachment: Some(render_targets.loaded_depth_attachment()),
            occlusion_query_set: None,
            timestamp_writes,
            multiview_mask: None,
        });
        depth_pass.set_pipeline(&self.depth_prepass_pipeline);
        self.bind(
            &mut depth_pass,
            bind_groups
                .iter()
                .copied()
                .filter(|&(definition, _)| definition.visible_to(wgpu::ShaderStages::VERTEX)),
            packed_indices,
        );
        depth_pass.draw_indexed_indirect(frame_draw, PHASE_TWO_DRAW_OFFSET);
    }

    /// Records the visibility ID pass into `encoder`, with the visibility
    /// pre-pass Render switch on: clears the visibility IDs of
    /// `render_targets` and draws the frame's draw of both phases at depth
    /// equal against the depth target, writing each sample's triangle, the
    /// buffers and groups as `record_depth_prepass` takes them, timed into
    /// `timestamp_writes`. Without the switch, it records nothing.
    pub fn record_visibility_ids(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        render_targets: &RenderTargets,
        bind_groups: &[(&BindGroupDefinition, &wgpu::BindGroup)],
        packed_indices: &wgpu::Buffer,
        frame_draw: &wgpu::Buffer,
        timestamp_writes: Option<wgpu::RenderPassTimestampWrites<'_>>,
    ) {
        let (Some(pipeline), Some(visibility_ids)) = (
            &self.visibility_id_pipeline,
            render_targets.visibility_ids_attachment(),
        ) else {
            return;
        };
        let mut id_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(VISIBILITY_ID_PASS),
            color_attachments: &[Some(visibility_ids)],
            depth_stencil_attachment: Some(render_targets.read_only_depth_attachment()),
            occlusion_query_set: None,
            timestamp_writes,
            multiview_mask: None,
        });
        id_pass.set_pipeline(pipeline);
        self.bind(
            &mut id_pass,
            bind_groups
                .iter()
                .copied()
                .filter(|&(definition, _)| definition.visible_to(wgpu::ShaderStages::VERTEX)),
            packed_indices,
        );
        id_pass.draw_indexed_indirect(frame_draw, 0);
    }

    /// Records the lit pass into `encoder`: clears the multisampled colour
    /// to `background`, exposed, draws phase two's draw into the depth
    /// target unless its own pass did, then the frame's draw of both phases
    /// at depth equal, the buffers as `record_depth_prepass` takes them, or
    /// the compacted draw's with the visibility pre-pass on, with
    /// `bind_groups` set, and resolves the colour into the HDR target, timed
    /// into `timestamp_writes`.
    #[expect(clippy::too_many_arguments)]
    pub fn record_lit_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        render_targets: &RenderTargets,
        bind_groups: &[(&BindGroupDefinition, &wgpu::BindGroup)],
        packed_indices: &wgpu::Buffer,
        frame_draw: &wgpu::Buffer,
        background: wgpu::Color,
        timestamp_writes: Option<wgpu::RenderPassTimestampWrites<'_>>,
    ) {
        let mut lit_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(LIT_PASS),
            color_attachments: &[
                // This is what @location(0) in the fragment shader targets
                Some(render_targets.color_attachment(background)),
            ],
            depth_stencil_attachment: Some(render_targets.final_depth_attachment(self.keeps_depth)),
            occlusion_query_set: None,
            timestamp_writes,
            multiview_mask: None,
        });
        self.bind(&mut lit_pass, bind_groups.iter().copied(), packed_indices);
        if self.phase_two_in_lit_pass {
            lit_pass.set_pipeline(&self.lit_pass_depth_pipeline);
            lit_pass.draw_indexed_indirect(frame_draw, PHASE_TWO_DRAW_OFFSET);
        }
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
        render_pass.set_bind_group(MAIN_PASS_INPUTS.index, &self.inputs, &[]);
        render_pass.set_index_buffer(packed_indices.slice(..), wgpu::IndexFormat::Uint32);
    }
}
