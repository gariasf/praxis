//! Skybox rendering system.
//!
//! This module provides functionality for rendering skyboxes with cubemap textures.
//! Skyboxes create the illusion of a distant environment (sky, space, etc.) by
//! rendering a large cube around the scene with reversed depth.

use crate::vertex::Vertex3D;
use praxis_utils::{debug, eyre, trace, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    descriptor_set::{
        allocator::DescriptorSetAllocator, DescriptorSet, WriteDescriptorSet,
    },
    device::Device,
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, RasterizationState},
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{RenderPass, Subpass},
    shader::ShaderModule,
};

/// Skybox renderer that manages pipeline and geometry.
///
/// The skybox is rendered as a cube with reversed depth to ensure it always
/// appears behind all other geometry in the scene.
pub struct SkyboxRenderer {
    /// Graphics pipeline for skybox rendering.
    pipeline: Arc<GraphicsPipeline>,

    /// Vertex buffer containing the skybox cube geometry.
    vertex_buffer: Subbuffer<[Vertex3D]>,

    /// Index buffer for the skybox cube.
    index_buffer: Subbuffer<[u32]>,

    /// Number of indices in the index buffer.
    index_count: u32,

    /// Descriptor set layout for skybox rendering.
    descriptor_set_layout: Arc<vulkano::descriptor_set::layout::DescriptorSetLayout>,
}

impl SkyboxRenderer {
    /// Creates a new skybox renderer.
    ///
    /// # Arguments
    ///
    /// * `device` - Vulkan device
    /// * `render_pass` - Render pass to render into
    /// * `viewport` - Viewport dimensions
    /// * `memory_allocator` - Memory allocator for buffers
    ///
    /// # Errors
    ///
    /// Returns an error if pipeline creation or buffer allocation fails.
    pub fn new(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        viewport: Viewport,
        memory_allocator: Arc<dyn MemoryAllocator>,
    ) -> Result<Self> {
        debug!("Creating skybox renderer");

        let (vertex_buffer, index_buffer, index_count) =
            Self::create_skybox_cube(memory_allocator)?;

        let vs_module = Self::load_vertex_shader(device.clone())?;
        let fs_module = Self::load_fragment_shader(device.clone())?;

        let vs = vs_module.entry_point("main")
            .ok_or_else(|| eyre::eyre!("Failed to find 'main' entry point in vertex shader"))?;
        let fs = fs_module.entry_point("main")
            .ok_or_else(|| eyre::eyre!("Failed to find 'main' entry point in fragment shader"))?;

        let vertex_input_state = Vertex3D::per_vertex()
            .definition(&vs)
            .map_err(|e| eyre::eyre!("Failed to create vertex input state: {}", e))?;

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .map_err(|e| eyre::eyre!("Failed to create pipeline layout: {}", e))?,
        )
        .map_err(|e| eyre::eyre!("Failed to create pipeline layout: {}", e))?;

        let descriptor_set_layout = layout.set_layouts()[0].clone();

        let subpass = Subpass::from(render_pass.clone(), 0)
            .ok_or_else(|| eyre::eyre!("Failed to create subpass"))?;

        let pipeline = GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into_iter().collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState {
                    cull_mode: CullMode::Back,
                    front_face: FrontFace::CounterClockwise,
                    ..Default::default()
                }),
                multisample_state: Some(MultisampleState::default()),
                depth_stencil_state: Some(DepthStencilState {
                    depth: Some(DepthState {
                        compare_op: CompareOp::LessOrEqual,
                        write_enable: false,
                    }),
                    ..Default::default()
                }),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create skybox pipeline: {}", e))?;

        debug!("Skybox renderer created successfully");

        Ok(Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            index_count,
            descriptor_set_layout,
        })
    }

    /// Creates the vertex and index buffers for a skybox cube.
    ///
    /// The cube extends from -1 to +1 on all axes.
    #[allow(clippy::type_complexity)]
    fn create_skybox_cube(
        memory_allocator: Arc<dyn MemoryAllocator>,
    ) -> Result<(Subbuffer<[Vertex3D]>, Subbuffer<[u32]>, u32)> {
        trace!("Creating skybox cube geometry");

        let vertices = vec![
            // Front face (+Z)
            Vertex3D::new([-1.0, -1.0, 1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([1.0, -1.0, 1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([1.0, 1.0, 1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([-1.0, 1.0, 1.0], [1.0, 1.0, 1.0]),
            // Back face (-Z)
            Vertex3D::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([-1.0, 1.0, -1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([1.0, 1.0, -1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([1.0, -1.0, -1.0], [1.0, 1.0, 1.0]),
            // Top face (+Y)
            Vertex3D::new([-1.0, 1.0, -1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([-1.0, 1.0, 1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([1.0, 1.0, 1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([1.0, 1.0, -1.0], [1.0, 1.0, 1.0]),
            // Bottom face (-Y)
            Vertex3D::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([1.0, -1.0, -1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([1.0, -1.0, 1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([-1.0, -1.0, 1.0], [1.0, 1.0, 1.0]),
            // Right face (+X)
            Vertex3D::new([1.0, -1.0, -1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([1.0, 1.0, -1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([1.0, 1.0, 1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([1.0, -1.0, 1.0], [1.0, 1.0, 1.0]),
            // Left face (-X)
            Vertex3D::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([-1.0, -1.0, 1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([-1.0, 1.0, 1.0], [1.0, 1.0, 1.0]),
            Vertex3D::new([-1.0, 1.0, -1.0], [1.0, 1.0, 1.0]),
        ];

        #[rustfmt::skip]
        let indices: Vec<u32> = vec![
            // Front
            0, 1, 2, 2, 3, 0,
            // Back
            4, 5, 6, 6, 7, 4,
            // Top
            8, 9, 10, 10, 11, 8,
            // Bottom
            12, 13, 14, 14, 15, 12,
            // Right
            16, 17, 18, 18, 19, 16,
            // Left
            20, 21, 22, 22, 23, 20,
        ];

        let index_count = indices.len() as u32;

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
        .map_err(|e| eyre::eyre!("Failed to create skybox vertex buffer: {}", e))?;

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
        .map_err(|e| eyre::eyre!("Failed to create skybox index buffer: {}", e))?;

        trace!("Skybox cube created with {} indices", index_count);

        Ok((vertex_buffer, index_buffer, index_count))
    }

    fn load_vertex_shader(device: Arc<Device>) -> Result<Arc<ShaderModule>> {
        crate::shaders::skybox_vs::load(device)
            .map_err(|e| eyre::eyre!("Failed to load skybox vertex shader: {}", e))
    }

    fn load_fragment_shader(device: Arc<Device>) -> Result<Arc<ShaderModule>> {
        crate::shaders::skybox_fs::load(device)
            .map_err(|e| eyre::eyre!("Failed to load skybox fragment shader: {}", e))
    }

    /// Gets the graphics pipeline.
    pub fn pipeline(&self) -> &Arc<GraphicsPipeline> {
        &self.pipeline
    }

    /// Gets the vertex buffer.
    pub fn vertex_buffer(&self) -> &Subbuffer<[Vertex3D]> {
        &self.vertex_buffer
    }

    /// Gets the index buffer.
    pub fn index_buffer(&self) -> &Subbuffer<[u32]> {
        &self.index_buffer
    }

    /// Gets the index count.
    pub fn index_count(&self) -> u32 {
        self.index_count
    }

    /// Creates a descriptor set for rendering a skybox.
    ///
    /// # Arguments
    ///
    /// * `descriptor_set_allocator` - Allocator for descriptor sets
    /// * `view_proj_buffer` - Buffer containing view and projection matrices
    /// * `cubemap_view` - Image view of the cubemap texture
    /// * `cubemap_sampler` - Sampler for the cubemap texture
    ///
    /// # Errors
    ///
    /// Returns an error if descriptor set creation fails.
    pub fn create_descriptor_set(
        &self,
        descriptor_set_allocator: Arc<dyn DescriptorSetAllocator>,
        view_proj_buffer: impl Into<vulkano::buffer::Subbuffer<crate::uniform_buffer::ViewProjectionUniforms>>,
        cubemap_view: Arc<vulkano::image::view::ImageView>,
        cubemap_sampler: Arc<vulkano::image::sampler::Sampler>,
    ) -> Result<Arc<DescriptorSet>> {
        DescriptorSet::new(
            descriptor_set_allocator,
            self.descriptor_set_layout.clone(),
            [
                WriteDescriptorSet::buffer(0, view_proj_buffer.into()),
                WriteDescriptorSet::image_view_sampler(1, cubemap_view, cubemap_sampler),
            ],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create skybox descriptor set: {}", e))
    }
}
