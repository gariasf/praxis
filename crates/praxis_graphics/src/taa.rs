//! Temporal Anti-Aliasing (TAA) implementation.
//!
//! This module provides temporal anti-aliasing using velocity buffers for reprojection
//! and neighborhood clamping for history rejection.
//!
//! # Overview
//!
//! TAA works by blending the current frame with the previous frame(s) to reduce aliasing:
//! 1. **Velocity Generation**: Per-pixel motion vectors are computed during geometry pass
//! 2. **Temporal Reprojection**: Previous frame is sampled using velocity to find corresponding pixels
//! 3. **History Rejection**: Neighborhood clamping prevents ghosting artifacts
//! 4. **Blending**: Current and reprojected history are blended together
//!
//! # Benefits
//!
//! - Effectively reduces temporal aliasing (shimmering edges)
//! - Works well with camera jitter for sub-pixel detail
//! - Lower performance cost than MSAA
//! - Produces temporally stable image
//!
//! # Trade-offs
//!
//! - Can introduce ghosting on fast-moving objects (mitigated by neighborhood clamping)
//! - Requires velocity buffer generation
//! - Needs history buffer storage

use crate::post_process::QuadVertex;
use crate::shaders;
use praxis_math::Mat4;
use praxis_utils::{debug, eyre, info, trace, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        AutoCommandBufferBuilder, RenderPassBeginInfo, SubpassBeginInfo, SubpassEndInfo,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, DescriptorSet, WriteDescriptorSet,
    },
    device::Device,
    format::Format,
    image::{
        sampler::{Filter, Sampler, SamplerCreateInfo},
        view::ImageView,
        Image, ImageCreateInfo, ImageType, ImageUsage,
    },
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        GraphicsPipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
};

/// Configuration for TAA effect.
#[derive(Clone, Copy, Debug)]
pub struct TaaConfig {
    /// Jitter offset for sub-pixel sampling (typically in range [-0.5, 0.5])
    pub jitter_offset: [f32; 2],
    /// Blend factor between current and history (0.0 = all history, 1.0 = all current)
    /// Typical value: 0.05-0.1
    pub blend_factor: f32,
}

impl Default for TaaConfig {
    fn default() -> Self {
        Self {
            jitter_offset: [0.0, 0.0],
            blend_factor: 0.1,
        }
    }
}

/// Uniform buffer data for TAA shader.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TaaUniforms {
    jitter_offset: [f32; 2],
    blend_factor: f32,
    _padding: f32,
}

impl From<TaaConfig> for TaaUniforms {
    fn from(config: TaaConfig) -> Self {
        Self {
            jitter_offset: config.jitter_offset,
            blend_factor: config.blend_factor,
            _padding: 0.0,
        }
    }
}

/// TAA render target for storing history buffer.
pub struct TaaRenderTarget {
    /// Current frame color attachment
    pub color_image: Arc<Image>,
    /// Current frame image view
    pub color_view: Arc<ImageView>,
    /// History frame color attachment
    pub history_image: Arc<Image>,
    /// History frame image view
    pub history_view: Arc<ImageView>,
    /// Framebuffer for rendering TAA output
    pub framebuffer: Arc<Framebuffer>,
    /// Width of the render target
    pub width: u32,
    /// Height of the render target
    pub height: u32,
}

impl TaaRenderTarget {
    /// Creates a new TAA render target.
    pub fn new(
        memory_allocator: Arc<StandardMemoryAllocator>,
        render_pass: Arc<RenderPass>,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        debug!("Creating TAA render target: {}x{}", width, height);

        let create_image = || -> Result<(Arc<Image>, Arc<ImageView>)> {
            let image = Image::new(
                memory_allocator.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::R16G16B16A16_SFLOAT,
                    extent: [width, height, 1],
                    usage: ImageUsage::COLOR_ATTACHMENT
                        | ImageUsage::SAMPLED
                        | ImageUsage::TRANSFER_DST,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .map_err(|e| eyre::eyre!("Failed to create TAA image: {}", e))?;

            let view = ImageView::new_default(image.clone())
                .map_err(|e| eyre::eyre!("Failed to create TAA image view: {}", e))?;

            Ok((image, view))
        };

        let (color_image, color_view) = create_image()?;
        let (history_image, history_view) = create_image()?;

        let framebuffer = Framebuffer::new(
            render_pass,
            FramebufferCreateInfo {
                attachments: vec![color_view.clone()],
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create TAA framebuffer: {}", e))?;

        debug!("TAA render target created successfully");

        Ok(Self {
            color_image,
            color_view,
            history_image,
            history_view,
            framebuffer,
            width,
            height,
        })
    }

    /// Swaps current and history buffers.
    pub fn swap_buffers(&mut self) {
        std::mem::swap(&mut self.color_image, &mut self.history_image);
        std::mem::swap(&mut self.color_view, &mut self.history_view);
    }
}

/// TAA renderer implementing temporal anti-aliasing.
pub struct TaaRenderer {
    #[allow(dead_code)]
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    sampler: Arc<Sampler>,
    fullscreen_quad_vertices: vulkano::buffer::Subbuffer<[QuadVertex]>,
    fullscreen_quad_indices: vulkano::buffer::Subbuffer<[u32]>,
}

impl TaaRenderer {
    /// Creates a new TAA renderer.
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<Self> {
        info!("Creating TAA renderer");

        let render_pass = Self::create_render_pass(device.clone())?;
        let pipeline = Self::create_pipeline(device.clone(), &render_pass)?;

        let sampler = Sampler::new(
            device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create TAA sampler: {}", e))?;

        let (fullscreen_quad_vertices, fullscreen_quad_indices) =
            Self::create_fullscreen_quad(&memory_allocator)?;

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        debug!("TAA renderer created successfully");

        Ok(Self {
            device,
            memory_allocator,
            descriptor_set_allocator,
            render_pass,
            pipeline,
            sampler,
            fullscreen_quad_vertices,
            fullscreen_quad_indices,
        })
    }

    /// Creates the render pass for TAA.
    fn create_render_pass(device: Arc<Device>) -> Result<Arc<RenderPass>> {
        vulkano::single_pass_renderpass!(
            device,
            attachments: {
                color: {
                    format: Format::R16G16B16A16_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .map_err(|e| eyre::eyre!("Failed to create TAA render pass: {}", e))
    }

    /// Creates the graphics pipeline for TAA.
    fn create_pipeline(
        device: Arc<Device>,
        render_pass: &Arc<RenderPass>,
    ) -> Result<Arc<GraphicsPipeline>> {
        debug!("Creating TAA pipeline");

        let vs_module = shaders::taa_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load TAA vertex shader: {}", e))?;
        let fs_module = shaders::taa_fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load TAA fragment shader: {}", e))?;

        let vs_entry = vs_module
            .entry_point("main")
            .ok_or_else(|| eyre::eyre!("Failed to find main entry point in vertex shader"))?;
        let fs_entry = fs_module
            .entry_point("main")
            .ok_or_else(|| eyre::eyre!("Failed to find main entry point in fragment shader"))?;

        let stages = [
            PipelineShaderStageCreateInfo::new(vs_entry.clone()),
            PipelineShaderStageCreateInfo::new(fs_entry),
        ];

        let vertex_input_state = QuadVertex::per_vertex()
            .definition(&vs_entry)
            .map_err(|e| eyre::eyre!("Failed to create vertex input state: {}", e))?;

        let layout_create_infos = PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .map_err(|e| eyre::eyre!("Failed to create pipeline layout info: {}", e))?;

        let layout = PipelineLayout::new(device.clone(), layout_create_infos)
            .map_err(|e| eyre::eyre!("Failed to create pipeline layout: {}", e))?;

        let subpass = Subpass::from(render_pass.clone(), 0)
            .ok_or_else(|| eyre::eyre!("Failed to get subpass"))?;

        let pipeline = GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState::default()),
                rasterization_state: Some(RasterizationState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                dynamic_state: [vulkano::pipeline::DynamicState::Viewport]
                    .into_iter()
                    .collect(),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create TAA pipeline: {}", e))?;

        Ok(pipeline)
    }

    /// Creates vertex and index buffers for full-screen quad.
    #[allow(clippy::type_complexity)]
    fn create_fullscreen_quad(
        memory_allocator: &Arc<StandardMemoryAllocator>,
    ) -> Result<(
        vulkano::buffer::Subbuffer<[QuadVertex]>,
        vulkano::buffer::Subbuffer<[u32]>,
    )> {
        let vertices = [
            QuadVertex {
                position: [-1.0, -1.0],
                uv: [0.0, 0.0],
            },
            QuadVertex {
                position: [1.0, -1.0],
                uv: [1.0, 0.0],
            },
            QuadVertex {
                position: [1.0, 1.0],
                uv: [1.0, 1.0],
            },
            QuadVertex {
                position: [-1.0, 1.0],
                uv: [0.0, 1.0],
            },
        ];

        let indices = [0u32, 1, 2, 0, 2, 3];

        let vertex_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )
        .map_err(|e| eyre::eyre!("Failed to create fullscreen quad vertex buffer: {}", e))?;

        let index_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices,
        )
        .map_err(|e| eyre::eyre!("Failed to create fullscreen quad index buffer: {}", e))?;

        Ok((vertex_buffer, index_buffer))
    }

    /// Creates a TAA render target with the specified dimensions.
    pub fn create_render_target(&self, width: u32, height: u32) -> Result<TaaRenderTarget> {
        TaaRenderTarget::new(
            self.memory_allocator.clone(),
            self.render_pass.clone(),
            width,
            height,
        )
    }

    /// Applies TAA to the current frame.
    #[allow(clippy::too_many_arguments)]
    pub fn apply(
        &self,
        builder: &mut AutoCommandBufferBuilder<
            impl vulkano::command_buffer::allocator::CommandBufferAllocator,
        >,
        taa_target: &TaaRenderTarget,
        current_frame: Arc<ImageView>,
        velocity_buffer: Arc<ImageView>,
        depth_buffer: Arc<ImageView>,
        config: TaaConfig,
    ) -> Result<()> {
        trace!("Applying TAA");

        let config_uniforms = TaaUniforms::from(config);
        let config_buffer = Buffer::from_data(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            config_uniforms,
        )
        .map_err(|e| eyre::eyre!("Failed to create TAA config buffer: {}", e))?;

        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            self.pipeline.layout().set_layouts()[0].clone(),
            [
                WriteDescriptorSet::image_view_sampler(0, current_frame, self.sampler.clone()),
                WriteDescriptorSet::image_view_sampler(
                    1,
                    taa_target.history_view.clone(),
                    self.sampler.clone(),
                ),
                WriteDescriptorSet::image_view_sampler(2, velocity_buffer, self.sampler.clone()),
                WriteDescriptorSet::image_view_sampler(3, depth_buffer, self.sampler.clone()),
                WriteDescriptorSet::buffer(4, config_buffer),
            ],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create TAA descriptor set: {}", e))?;

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(taa_target.framebuffer.clone())
                },
                SubpassBeginInfo {
                    contents: vulkano::command_buffer::SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .map_err(|e| eyre::eyre!("Failed to begin TAA render pass: {}", e))?;

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [taa_target.width as f32, taa_target.height as f32],
            depth_range: 0.0..=1.0,
        };

        builder
            .set_viewport(0, [viewport].into_iter().collect())
            .map_err(|e| eyre::eyre!("Failed to set viewport: {}", e))?;

        builder
            .bind_pipeline_graphics(self.pipeline.clone())
            .map_err(|e| eyre::eyre!("Failed to bind pipeline: {}", e))?
            .bind_descriptor_sets(
                vulkano::pipeline::PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .map_err(|e| eyre::eyre!("Failed to bind descriptor sets: {}", e))?;

        builder
            .bind_vertex_buffers(0, self.fullscreen_quad_vertices.clone())
            .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
            .bind_index_buffer(self.fullscreen_quad_indices.clone())
            .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?;

        unsafe {
            builder
                .draw_indexed(6, 1, 0, 0, 0)
                .map_err(|e| eyre::eyre!("Failed to draw fullscreen quad: {}", e))?;
        }

        builder
            .end_render_pass(SubpassEndInfo::default())
            .map_err(|e| eyre::eyre!("Failed to end TAA render pass: {}", e))?;

        trace!("TAA applied successfully");

        Ok(())
    }
}

/// Halton sequence generator for temporal jitter patterns.
///
/// Generates low-discrepancy sequences for sub-pixel jittering in TAA.
/// This provides better coverage of sub-pixel locations across frames.
pub struct HaltonSequence {
    index: u32,
}

impl HaltonSequence {
    /// Creates a new Halton sequence generator.
    pub fn new() -> Self {
        Self { index: 0 }
    }

    /// Generates the next jitter offset using Halton(2,3) sequence.
    pub fn next_jitter(&mut self) -> [f32; 2] {
        self.index = (self.index + 1) % 16; // Use 16-sample pattern
        [
            Self::halton(self.index, 2) - 0.5,
            Self::halton(self.index, 3) - 0.5,
        ]
    }

    /// Computes the Halton sequence value for a given index and base.
    fn halton(mut index: u32, base: u32) -> f32 {
        let mut result = 0.0;
        let mut f = 1.0;
        let base_f = base as f32;

        while index > 0 {
            f /= base_f;
            result += f * (index % base) as f32;
            index /= base;
        }

        result
    }
}

impl Default for HaltonSequence {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to apply projection jitter for TAA.
pub fn apply_jitter_to_projection(proj: Mat4, jitter: [f32; 2], width: u32, height: u32) -> Mat4 {
    let jitter_x = jitter[0] * 2.0 / width as f32;
    let jitter_y = jitter[1] * 2.0 / height as f32;

    let jitter_matrix = Mat4::from_cols_array_2d(&[
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [jitter_x, jitter_y, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    jitter_matrix * proj
}
