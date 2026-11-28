//! The buffer view pass: with the `buffer_view` Render switch naming one of
//! the frame's buffers, one triangle over the target, after the tonemapping
//! pass, that writes that buffer there as an image in place of the frame's
//! display colour.

use crate::render::bind_groups::{BUFFER_VIEW_INPUT, pipeline_layout};
use crate::render::froxel_pass::FroxelPass;
use crate::render::renderer::SharedBuffers;
use crate::render::shaders;
use crate::render::shadow_mask_pass::SHADOW_MASK_LIGHTS;
use crate::render::targets::RenderTargets;

/// One of a frame's buffers as an image, each in one fixed mapping to
/// colour (`buffer_view.wgsl`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BufferView {
    Depth,
    /// The normal interpolated over each triangle.
    SurfaceNormals,
    TriangleNormals,
    /// A level; one past the coarsest shows the coarsest.
    DepthPyramid(u32),
    PointLights,
    /// A light's bit of the Shadow mask; one past its bits is never hidden.
    ShadowMask(u32),
    Hdr,
}

impl BufferView {
    /// The names `from_name` takes, with `target`, which names no view, and
    /// what `buffer_view.wgsl` maps each to.
    pub const USAGE: &str = "`target`, the frame's display colour (the default); \
        `depth`, the view depth of each pixel's nearest sample as grey, 1 / (1 + metres / 16), black where nothing is drawn; \
        `normals`, that sample's interpolated normal, its x, y and z from -1 to 1 as red, green and blue, or `triangle-normals`, its triangle's own; \
        `depth-pyramid[:<level>]`, a level of the depth pyramid in depth's grey, level 0 unless named, the coarsest past the last; \
        `point-lights`, the point lights the pixel's Froxel lists as heat, black at none through red and yellow to white at 8 or more; \
        `shadow-mask[:<light>]`, dark grey where a light is hidden from the pixel's nearest surface and light grey elsewhere, the light named by its place in the lit pass's order, the sun's, 0, unless named; \
        `hdr`, the HDR target's exposed colour, clamped to 1 and sRGB-encoded with no tone curve";

    pub fn from_name(name: &str) -> Option<Self> {
        let (buffer, index) = match name.split_once(':') {
            Some((buffer, index)) => (buffer, Some(index.parse::<u32>().ok()?)),
            None => (name, None),
        };
        match (buffer, index) {
            ("depth", None) => Some(Self::Depth),
            ("normals", None) => Some(Self::SurfaceNormals),
            ("triangle-normals", None) => Some(Self::TriangleNormals),
            ("depth-pyramid", level) => Some(Self::DepthPyramid(level.unwrap_or(0))),
            ("point-lights", None) => Some(Self::PointLights),
            ("shadow-mask", light) => Some(light.unwrap_or(0))
                .filter(|&light| light < SHADOW_MASK_LIGHTS)
                .map(Self::ShadowMask),
            ("hdr", None) => Some(Self::Hdr),
            _ => None,
        }
    }

    /// Its fragment entry point, and its `BUFFER_VIEW_INDEX`.
    fn entry_point(self) -> (&'static str, u32) {
        match self {
            Self::Depth => ("fs_depth_view", 0),
            Self::SurfaceNormals => ("fs_surface_normals_view", 1),
            Self::TriangleNormals => ("fs_surface_normals_view", 0),
            Self::DepthPyramid(level) => ("fs_depth_pyramid_view", level),
            Self::PointLights => ("fs_point_lights_view", 0),
            Self::ShadowMask(light) => ("fs_shadow_mask_view", light),
            Self::Hdr => ("fs_hdr_view", 0),
        }
    }
}

pub struct BufferViewPass {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    camera: wgpu::Buffer,
    froxel_lists: wgpu::Buffer,
}

pub struct BufferViewTargetGroups {
    input: wgpu::BindGroup,
}

impl BufferViewPass {
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        shared: &SharedBuffers,
        froxel_pass: &FroxelPass,
        view: BufferView,
    ) -> BufferViewPass {
        let shader = device.create_shader_module(shaders::BUFFER_VIEW.descriptor());
        let layout = BUFFER_VIEW_INPUT.layout(device);
        let (entry_point, index) = view.entry_point();
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Buffer View Pipeline"),
            layout: Some(&pipeline_layout(
                device,
                "Buffer View Pipeline Layout",
                &[(&BUFFER_VIEW_INPUT, &layout)],
            )),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_full_screen"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(entry_point),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[("BUFFER_VIEW_INDEX", f64::from(index))],
                    ..wgpu::PipelineCompilationOptions::default()
                },
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        BufferViewPass {
            pipeline,
            layout,
            camera: wgpu::Buffer::clone(&shared.camera),
            froxel_lists: froxel_pass.froxel_lists().clone(),
        }
    }

    pub fn target_groups(
        &self,
        device: &wgpu::Device,
        render_targets: &RenderTargets,
    ) -> BufferViewTargetGroups {
        let texture = wgpu::BindingResource::TextureView;
        let input = BUFFER_VIEW_INPUT.bind_group(
            device,
            &self.layout,
            [
                ("camera", self.camera.as_entire_binding()),
                ("froxel_lists", self.froxel_lists.as_entire_binding()),
                ("depth", texture(render_targets.depth())),
                ("surface_normals", texture(render_targets.surface_normals())),
                ("depth_pyramid", texture(render_targets.depth_pyramid())),
                ("shadow_mask", texture(render_targets.shadow_mask())),
                ("hdr_target", texture(render_targets.hdr())),
            ],
        );
        BufferViewTargetGroups { input }
    }

    /// `target` is a view in this pass's format and the render targets' size.
    pub fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target_groups: &BufferViewTargetGroups,
        target: &wgpu::TextureView,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("buffer_view_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(BUFFER_VIEW_INPUT.index, &target_groups.input, &[]);
        pass.draw(0..3, 0..1);
    }
}
