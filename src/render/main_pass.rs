//! The main pass, ADR 0019's render passes, which draw the culling pass's
//! frame's draw with one indirect call per phase and pipeline, the lit
//! pipeline's drawing both phases: a depth pre-pass that clears the depth
//! target and keeps the nearest depth of phase one's Instances, then phase
//! two's drawn into depth, then a lit pass that draws both phases' at depth
//! equal, each Meshlet with its record's Material, into the multisampled
//! colour, which it resolves into the HDR target. With shadow rays on, phase
//! two's depth is a pass of its own, so the shadow mask pass reads every
//! Instance's depth before the lit pass reads the mask; with them off, the
//! lit pass draws it first. Each draw is followed by its phase's Thin
//! Primitives' and small Instances' through a pipeline of their own, whose
//! depth passes write which samples a fragment keeps (ADR 0027), so no other
//! geometry pays for a written sample mask.

use crate::render::bind_groups::{BindGroupDefinition, MAIN_PASS_INPUTS, pipeline_layout};
use crate::render::culling_pass::{
    CullingPass, CullingPhase, PHASE_ONE_THIN_DRAW_OFFSET, PHASE_TWO_DRAW_OFFSET,
    PHASE_TWO_THIN_DRAW_OFFSET,
};
use crate::render::readback::FrameTimestamps;
use crate::render::renderer::RenderSwitches;
use crate::render::shaders;
use crate::render::targets::{self, RenderTargets};

/// The depth pre-pass's label, and its name in frame stats.
pub(crate) const DEPTH_PREPASS: &str = "depth_prepass";

/// Phase two's depth pass's label, and its name in frame stats.
pub(crate) const PHASE_TWO_DEPTH_PASS: &str = "phase_two_depth_pass";

/// The lit pass's label, and its name in frame stats.
pub(crate) const LIT_PASS: &str = "lit_pass";

/// The main pass's pipelines.
pub struct MainPass {
    depth_prepass_pipeline: wgpu::RenderPipeline,
    /// With shadow rays off, phase two's draw into depth, inside the lit
    /// pass, before it shades.
    lit_pass_depth_pipeline: wgpu::RenderPipeline,
    lit_pipeline: wgpu::RenderPipeline,
    /// Each pipeline's twin for Thin Primitives and small Instances.
    thin_depth_prepass_pipeline: wgpu::RenderPipeline,
    thin_lit_pass_depth_pipeline: wgpu::RenderPipeline,
    thin_lit_pipeline: wgpu::RenderPipeline,
    inputs_layout: wgpu::BindGroupLayout,
    meshlet_records_buffer: wgpu::Buffer,
    /// The frame's draw's packed indices, its index buffer.
    packed_indices_buffer: wgpu::Buffer,
    /// The frame's draw's indirect arguments, a draw per phase.
    frame_draw_buffer: wgpu::Buffer,
    /// Whether shadow rays are on, so phase two's depth is drawn in its own
    /// pass rather than by the lit pass.
    shadow_rays: bool,
    /// Whether the lit pass keeps the depth it ends with, for the light
    /// count pass (the `count_point_lights` Render switch) or the buffer
    /// view pass (`buffer_view`).
    keeps_depth: bool,
}

/// The pass's group 3 (`MAIN_PASS_INPUTS`), which reads the render targets
/// `target_groups` built it from.
pub struct MainTargetGroups {
    inputs: wgpu::BindGroup,
}

impl MainPass {
    /// The pass, given the layouts of the shared bind groups its shader
    /// module declares, which draws `culling_pass`'s frame's draw from its
    /// Meshlet records, packed indices and indirect arguments, its lit
    /// pipeline as `switches` set.
    pub fn new(
        device: &wgpu::Device,
        bind_group_layouts: &[(&BindGroupDefinition, &wgpu::BindGroupLayout)],
        culling_pass: &CullingPass,
        switches: RenderSwitches,
    ) -> MainPass {
        let inputs_layout = MAIN_PASS_INPUTS.layout(device);
        let shader = shaders::MAIN.create_with_unbounded_loops(device);
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
        let depth_pipeline = |label, vertex_entry_point, fragment| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&depth_prepass_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(vertex_entry_point),
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
        let depth_fragment = |entry_point, targets| {
            Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(entry_point),
                targets,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            })
        };
        let surface_normals_target = [Some(wgpu::ColorTargetState {
            format: targets::SURFACE_NORMALS_FORMAT,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        })];
        // With shadow rays on, the depth passes write each sample's surface
        // normals beside its depth, for the shadow mask pass.
        let depth_prepass_pipeline = if switches.shadow_rays {
            depth_pipeline(
                "Depth Pre-pass Pipeline",
                "vs_depth_normals",
                depth_fragment("fs_surface_normals", &surface_normals_target),
            )
        } else {
            depth_pipeline("Depth Pre-pass Pipeline", "vs_depth_prepass", None)
        };
        let thin_depth_prepass_pipeline = if switches.shadow_rays {
            depth_pipeline(
                "Thin Depth Pre-pass Pipeline",
                "vs_thin",
                depth_fragment("fs_thin_surface_normals", &surface_normals_target),
            )
        } else {
            depth_pipeline(
                "Thin Depth Pre-pass Pipeline",
                "vs_thin",
                depth_fragment("fs_thin_depth", &[]),
            )
        };
        // Every pipeline the lit pass sets names its colour target, so its
        // depth draws name it too, and write nothing to it.
        let unwritten_colour_target = [Some(wgpu::ColorTargetState {
            format: targets::HDR_FORMAT,
            blend: None,
            write_mask: wgpu::ColorWrites::empty(),
        })];
        let lit_pass_depth_pipeline = depth_pipeline(
            "Lit Pass Depth Pipeline",
            "vs_depth_prepass",
            depth_fragment("fs_depth", &unwritten_colour_target),
        );
        let thin_lit_pass_depth_pipeline = depth_pipeline(
            "Thin Lit Pass Depth Pipeline",
            "vs_thin",
            depth_fragment("fs_thin_depth", &unwritten_colour_target),
        );
        let constants = switches.constants();
        let mut lit_descriptor = wgpu::RenderPipelineDescriptor {
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
                    constants: &constants,
                    ..wgpu::PipelineCompilationOptions::default()
                },
            }),
            primitive,
            depth_stencil: Some(targets::depth_equal_state()),
            multisample,
            multiview_mask: None,
            cache: None,
        };
        let lit_pipeline = device.create_render_pipeline(&lit_descriptor);
        lit_descriptor.label = Some("Thin Lit Pass Pipeline");
        lit_descriptor.vertex.entry_point = Some("vs_thin");
        if let Some(fragment) = &mut lit_descriptor.fragment {
            fragment.entry_point = Some("fs_thin_main");
        }
        let thin_lit_pipeline = device.create_render_pipeline(&lit_descriptor);
        MainPass {
            depth_prepass_pipeline,
            lit_pass_depth_pipeline,
            lit_pipeline,
            thin_depth_prepass_pipeline,
            thin_lit_pass_depth_pipeline,
            thin_lit_pipeline,
            inputs_layout,
            meshlet_records_buffer: culling_pass.meshlet_records().clone(),
            packed_indices_buffer: culling_pass.packed_indices().clone(),
            frame_draw_buffer: culling_pass.frame_draw().clone(),
            shadow_rays: switches.shadow_rays,
            keeps_depth: switches.count_point_lights || switches.buffer_view.is_some(),
        }
    }

    /// Its group 3: the Meshlet records, and the shadow mask of
    /// `render_targets` and which samples show its second surface.
    pub fn target_groups(
        &self,
        device: &wgpu::Device,
        render_targets: &RenderTargets,
    ) -> MainTargetGroups {
        let inputs = MAIN_PASS_INPUTS.bind_group(
            device,
            &self.inputs_layout,
            [
                (
                    "meshlet_records",
                    self.meshlet_records_buffer.as_entire_binding(),
                ),
                (
                    "shadow_mask",
                    wgpu::BindingResource::TextureView(render_targets.shadow_mask()),
                ),
                (
                    "shadow_mask_samples",
                    wgpu::BindingResource::TextureView(render_targets.shadow_mask_samples()),
                ),
            ],
        );
        MainTargetGroups { inputs }
    }

    /// Records `phase`'s depth pass into `encoder`, with `target_groups` and
    /// the groups of `bind_groups` its vertex stage reads set, timed into
    /// `timestamps`: it draws the phase's draw into the depth target,
    /// and with shadow rays on into the surface normals. Phase one's, the
    /// depth pre-pass, clears them; phase two's, recorded only with shadow
    /// rays on, loads them and stores them for the shadow mask pass. It
    /// draws the phase's Thin Primitives and small Instances after.
    pub fn record_depth_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        render_targets: &RenderTargets,
        target_groups: &MainTargetGroups,
        bind_groups: &[(&BindGroupDefinition, &wgpu::BindGroup)],
        phase: CullingPhase,
        timestamps: &mut Option<FrameTimestamps<'_>>,
    ) {
        let (label, depth_attachment, clears_surface_normals, draw_offset) = match phase {
            CullingPhase::One => (DEPTH_PREPASS, render_targets.depth_attachment(), true, 0),
            CullingPhase::Two => (
                PHASE_TWO_DEPTH_PASS,
                render_targets.loaded_depth_attachment(),
                false,
                PHASE_TWO_DRAW_OFFSET,
            ),
        };
        let surface_normals = [Some(
            render_targets.surface_normals_attachment(clears_surface_normals),
        )];
        let mut depth_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: if self.shadow_rays {
                &surface_normals
            } else {
                &[]
            },
            depth_stencil_attachment: Some(depth_attachment),
            occlusion_query_set: None,
            timestamp_writes: timestamps
                .as_mut()
                .map(|timestamps| timestamps.pass(label).render()),
            multiview_mask: None,
        });
        depth_pass.set_pipeline(&self.depth_prepass_pipeline);
        self.bind(
            &mut depth_pass,
            target_groups,
            bind_groups
                .iter()
                .copied()
                .filter(|&(definition, _)| definition.visible_to(wgpu::ShaderStages::VERTEX)),
        );
        depth_pass.draw_indexed_indirect(&self.frame_draw_buffer, draw_offset);
        let thin_draw_offset = match phase {
            CullingPhase::One => PHASE_ONE_THIN_DRAW_OFFSET,
            CullingPhase::Two => PHASE_TWO_THIN_DRAW_OFFSET,
        };
        depth_pass.set_pipeline(&self.thin_depth_prepass_pipeline);
        depth_pass.draw_indexed_indirect(&self.frame_draw_buffer, thin_draw_offset);
    }

    /// Records the lit pass into `encoder`: clears the multisampled colour
    /// to `background`, exposed, draws phase two's draw into the depth
    /// target unless its own pass did, then the frame's draw of both phases
    /// at depth equal, with `target_groups` and `bind_groups` set, and
    /// resolves the colour into
    /// the HDR target, timed into `timestamps`. Each draw is followed
    /// by the Thin Primitives' and small Instances' of its phases.
    pub fn record_lit_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        render_targets: &RenderTargets,
        target_groups: &MainTargetGroups,
        bind_groups: &[(&BindGroupDefinition, &wgpu::BindGroup)],
        background: wgpu::Color,
        timestamps: &mut Option<FrameTimestamps<'_>>,
    ) {
        let mut lit_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(LIT_PASS),
            color_attachments: &[
                // This is what @location(0) in the fragment shader targets
                Some(render_targets.color_attachment(background)),
            ],
            depth_stencil_attachment: Some(render_targets.final_depth_attachment(self.keeps_depth)),
            occlusion_query_set: None,
            timestamp_writes: timestamps
                .as_mut()
                .map(|timestamps| timestamps.pass(LIT_PASS).render()),
            multiview_mask: None,
        });
        self.bind(&mut lit_pass, target_groups, bind_groups.iter().copied());
        if !self.shadow_rays {
            lit_pass.set_pipeline(&self.lit_pass_depth_pipeline);
            lit_pass.draw_indexed_indirect(&self.frame_draw_buffer, PHASE_TWO_DRAW_OFFSET);
            lit_pass.set_pipeline(&self.thin_lit_pass_depth_pipeline);
            lit_pass.draw_indexed_indirect(&self.frame_draw_buffer, PHASE_TWO_THIN_DRAW_OFFSET);
        }
        lit_pass.set_pipeline(&self.lit_pipeline);
        lit_pass.draw_indexed_indirect(&self.frame_draw_buffer, 0);
        lit_pass.set_pipeline(&self.thin_lit_pipeline);
        lit_pass.draw_indexed_indirect(&self.frame_draw_buffer, PHASE_ONE_THIN_DRAW_OFFSET);
        lit_pass.draw_indexed_indirect(&self.frame_draw_buffer, PHASE_TWO_THIN_DRAW_OFFSET);
    }

    /// Sets `bind_groups`, this pass's own group and the frame's draw's
    /// packed indices into `render_pass`.
    fn bind<'a>(
        &self,
        render_pass: &mut wgpu::RenderPass<'_>,
        target_groups: &MainTargetGroups,
        bind_groups: impl IntoIterator<Item = (&'a BindGroupDefinition, &'a wgpu::BindGroup)>,
    ) {
        for (definition, bind_group) in bind_groups {
            render_pass.set_bind_group(definition.index, bind_group, &[]);
        }
        render_pass.set_bind_group(MAIN_PASS_INPUTS.index, &target_groups.inputs, &[]);
        render_pass.set_index_buffer(
            self.packed_indices_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
    }
}
