//! Velocity buffer generation for motion blur.
//!
//! This module provides functionality to generate per-pixel velocity buffers
//! used by motion blur post-processing effects.

use crate::{shaders, vertex::Vertex3D};
use praxis_math::Mat4;
use praxis_utils::{debug, eyre, info, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo,
        SubpassEndInfo,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, DescriptorSet, WriteDescriptorSet,
    },
    device::Device,
    format::Format,
    image::{
        view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage,
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

/// Velocity buffer for storing per-pixel motion vectors.
pub struct VelocityBuffer {
    /// Image storing velocity data (RG channels)
    pub image: Arc<Image>,
    /// Image view for the velocity buffer
    pub image_view: Arc<ImageView>,
    /// Framebuffer for rendering velocity
    pub framebuffer: Arc<Framebuffer>,
    /// Width of the buffer
    pub width: u32,
    /// Height of the buffer
    pub height: u32,
}

impl VelocityBuffer {
    /// Creates a new velocity buffer.
    pub fn new(
        memory_allocator: Arc<StandardMemoryAllocator>,
        render_pass: Arc<RenderPass>,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        debug!("Creating velocity buffer: {}x{}", width, height);

        let image = Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R16G16_SFLOAT,
                extent: [width, height, 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .map_err(|e| eyre::eyre!("Failed to create velocity buffer image: {}", e))?;

        let image_view = ImageView::new_default(image.clone())
            .map_err(|e| eyre::eyre!("Failed to create velocity buffer image view: {}", e))?;

        let framebuffer = Framebuffer::new(
            render_pass,
            FramebufferCreateInfo {
                attachments: vec![image_view.clone()],
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create velocity buffer framebuffer: {}", e))?;

        debug!("Velocity buffer created successfully");

        Ok(Self {
            image,
            image_view,
            framebuffer,
            width,
            height,
        })
    }
}

/// Velocity buffer renderer for generating motion vectors.
pub struct VelocityBufferRenderer {
    #[allow(dead_code)]
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
}

impl VelocityBufferRenderer {
    /// Creates a new velocity buffer renderer.
    pub fn new(device: Arc<Device>, memory_allocator: Arc<StandardMemoryAllocator>) -> Result<Self> {
        info!("Creating velocity buffer renderer");

        let render_pass = Self::create_render_pass(device.clone())?;

        let vs_module = shaders::velocity_buffer_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load velocity buffer vertex shader: {}", e))?;
        let fs_module = shaders::velocity_buffer_fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load velocity buffer fragment shader: {}", e))?;

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

        let vertex_input_state = Vertex3D::per_vertex()
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
        .map_err(|e| eyre::eyre!("Failed to create velocity buffer pipeline: {}", e))?;

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        debug!("Velocity buffer renderer created successfully");

        Ok(Self {
            device,
            memory_allocator,
            descriptor_set_allocator,
            render_pass,
            pipeline,
        })
    }

    /// Creates a render pass for velocity buffer generation.
    fn create_render_pass(device: Arc<Device>) -> Result<Arc<RenderPass>> {
        vulkano::single_pass_renderpass!(
            device,
            attachments: {
                velocity: {
                    format: Format::R16G16_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            pass: {
                color: [velocity],
                depth_stencil: {}
            }
        )
        .map_err(|e| eyre::eyre!("Failed to create velocity buffer render pass: {}", e))
    }

    /// Creates a velocity buffer with the specified dimensions.
    pub fn create_buffer(&self, width: u32, height: u32) -> Result<VelocityBuffer> {
        VelocityBuffer::new(
            self.memory_allocator.clone(),
            self.render_pass.clone(),
            width,
            height,
        )
    }

    /// Renders velocity buffer for a scene.
    ///
    /// This should be called before rendering the main scene to generate motion vectors
    /// based on the difference between current and previous frame transformations.
    #[allow(clippy::type_complexity)]
    pub fn render(
        &self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        velocity_buffer: &VelocityBuffer,
        current_mvp: Mat4,
        previous_mvp: Mat4,
        meshes: &[(
            &vulkano::buffer::Subbuffer<[crate::vertex::Vertex3D]>,
            &vulkano::buffer::Subbuffer<[u32]>,
            u32,
        )],
    ) -> Result<()> {
        debug!("Rendering velocity buffer");

        let velocity_uniforms = crate::post_process::VelocityUniforms::new(current_mvp, previous_mvp);

        let uniforms_buffer = Buffer::from_data(
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
            velocity_uniforms,
        )
        .map_err(|e| eyre::eyre!("Failed to create velocity uniforms buffer: {}", e))?;

        let layout = self.pipeline.layout().set_layouts()[0].clone();
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout,
            [WriteDescriptorSet::buffer(0, uniforms_buffer)],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create velocity descriptor set: {}", e))?;

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 0.0].into())],
                    ..RenderPassBeginInfo::framebuffer(velocity_buffer.framebuffer.clone())
                },
                SubpassBeginInfo {
                    contents: vulkano::command_buffer::SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .map_err(|e| eyre::eyre!("Failed to begin velocity render pass: {}", e))?;

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [velocity_buffer.width as f32, velocity_buffer.height as f32],
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

        for (vertex_buffer, index_buffer, index_count) in meshes {
            unsafe {
                builder
                    .bind_vertex_buffers(0, (*vertex_buffer).clone())
                    .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
                    .bind_index_buffer((*index_buffer).clone())
                    .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?
                    .draw_indexed(*index_count, 1, 0, 0, 0)
                    .map_err(|e| eyre::eyre!("Failed to draw indexed: {}", e))?;
            }
        }

        builder
            .end_render_pass(SubpassEndInfo::default())
            .map_err(|e| eyre::eyre!("Failed to end velocity render pass: {}", e))?;

        Ok(())
    }
}
