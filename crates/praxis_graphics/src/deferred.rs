//! Deferred rendering system with G-buffer passes and lighting accumulation.
//!
//! This module provides a complete deferred rendering pipeline that separates
//! geometry rendering from lighting calculations, enabling efficient many-light scenarios.
//!
//! # Deferred Rendering Overview
//!
//! Deferred rendering uses a two-pass approach:
//!
//! ## Pass 1: Geometry Pass (G-Buffer)
//! Renders scene geometry to multiple render targets (G-buffer) storing:
//! - **Albedo**: Base color (RGB) + unused (A)
//! - **Normal**: World-space normals (RGB) + unused (A)
//! - **Metallic-Roughness**: Metallic (R), Roughness (G), Emissive (B), unused (A)
//! - **Depth**: Standard depth buffer for depth testing
//!
//! ## Pass 2: Lighting Pass
//! Full-screen pass that reads G-buffer textures and accumulates lighting:
//! - Sample all G-buffer textures for current fragment
//! - Calculate lighting from all lights (directional + point)
//! - Output final lit color to framebuffer
//!
//! # Benefits Over Forward Rendering
//!
//! - **Many Lights**: Lighting cost is O(lights * pixels) instead of O(lights * triangles)
//! - **Efficient Culling**: Only lit pixels are processed, not occluded geometry
//! - **Decoupled Shading**: Geometry and lighting are independent
//!
//! # Trade-offs
//!
//! - **Memory**: Requires multiple full-screen render targets (G-buffer)
//! - **Bandwidth**: Multiple render target writes and reads
//! - **Transparency**: Difficult to handle (requires separate forward pass)
//! - **MSAA**: Expensive with multiple render targets

use crate::{lighting, material, mesh, uniform_buffer, vertex::Vertex3D, DrawCommand};
use praxis_utils::{debug, eyre, info, trace, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        AutoCommandBufferBuilder, RenderPassBeginInfo, SubpassBeginInfo,
        SubpassEndInfo,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, DescriptorSet, WriteDescriptorSet,
    },
    device::Device,
    format::Format,
    image::{sampler::{Filter, Sampler, SamplerCreateInfo}, view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        DynamicState, GraphicsPipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
};

/// G-Buffer render targets storing geometry data.
///
/// The G-buffer contains multiple textures that store per-pixel geometry information:
/// - Albedo: Base color of the surface
/// - Normal: World-space normal vector
/// - Metallic-Roughness: PBR material properties
/// - Depth: Standard depth buffer
pub struct GBuffer {
    /// Albedo texture (RGB) + unused (A)
    pub albedo: Arc<ImageView>,
    /// Normal texture (RGB) + unused (A)
    pub normal: Arc<ImageView>,
    /// Metallic (R), Roughness (G), Emissive (B), unused (A)
    pub metallic_roughness: Arc<ImageView>,
    /// Depth texture
    pub depth: Arc<ImageView>,
    /// Framebuffer for geometry pass
    pub framebuffer: Arc<Framebuffer>,
}

impl GBuffer {
    /// Creates a new G-buffer with the given dimensions.
    pub fn new(
        _device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        render_pass: Arc<RenderPass>,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        debug!("Creating G-buffer: {}x{}", width, height);

        let extent = [width, height, 1];

        let create_attachment = |format: Format, usage: ImageUsage| -> Result<Arc<ImageView>> {
            let image = Image::new(
                memory_allocator.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format,
                    extent,
                    usage,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .map_err(|e| eyre::eyre!("Failed to create G-buffer image: {}", e))?;

            ImageView::new_default(image)
                .map_err(|e| eyre::eyre!("Failed to create G-buffer image view: {}", e))
        };

        let usage = ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED;

        let albedo = create_attachment(Format::R8G8B8A8_UNORM, usage)?;
        let normal = create_attachment(Format::R16G16B16A16_SFLOAT, usage)?;
        let metallic_roughness = create_attachment(Format::R8G8B8A8_UNORM, usage)?;

        let depth = {
            let image = Image::new(
                memory_allocator.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::D32_SFLOAT,
                    extent,
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::SAMPLED,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .map_err(|e| eyre::eyre!("Failed to create depth image: {}", e))?;

            ImageView::new_default(image)
                .map_err(|e| eyre::eyre!("Failed to create depth image view: {}", e))?
        };

        let framebuffer = Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![
                    albedo.clone(),
                    normal.clone(),
                    metallic_roughness.clone(),
                    depth.clone(),
                ],
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create G-buffer framebuffer: {}", e))?;

        debug!("G-buffer created successfully");

        Ok(Self {
            albedo,
            normal,
            metallic_roughness,
            depth,
            framebuffer,
        })
    }
}

/// Deferred renderer managing G-buffer passes and lighting accumulation.
///
/// This renderer implements a complete deferred rendering pipeline:
/// 1. Geometry pass: Render scene to G-buffer
/// 2. Lighting pass: Full-screen pass accumulating lighting from G-buffer
pub struct DeferredRenderer {
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

    /// Render pass for geometry pass (outputs to G-buffer)
    geometry_pass: Arc<RenderPass>,
    /// Render pass for lighting pass (outputs to swapchain)
    #[allow(dead_code)]
    lighting_pass: Arc<RenderPass>,

    /// Pipeline for geometry pass
    geometry_pipeline: Arc<GraphicsPipeline>,
    /// Pipeline for lighting pass
    lighting_pipeline: Arc<GraphicsPipeline>,

    /// G-buffer storing geometry data
    gbuffer: Option<GBuffer>,

    /// Full-screen quad vertex buffer for lighting pass
    fullscreen_quad_vertices: vulkano::buffer::Subbuffer<[FullscreenVertex]>,
    /// Full-screen quad index buffer
    fullscreen_quad_indices: vulkano::buffer::Subbuffer<[u32]>,

    /// Current viewport dimensions
    width: u32,
    height: u32,
}

/// Vertex format for full-screen quad in lighting pass.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, Vertex)]
struct FullscreenVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
    #[format(R32G32_SFLOAT)]
    uv: [f32; 2],
}

impl DeferredRenderer {
    /// Creates a new deferred renderer.
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        info!("Initializing deferred renderer: {}x{}", width, height);

        let geometry_pass = Self::create_geometry_pass(&device)?;
        let lighting_pass = Self::create_lighting_pass(&device)?;

        let geometry_pipeline =
            Self::create_geometry_pipeline(&device, &geometry_pass, [width, height])?;
        let lighting_pipeline =
            Self::create_lighting_pipeline(&device, &lighting_pass, [width, height])?;

        let gbuffer = GBuffer::new(
            device.clone(),
            memory_allocator.clone(),
            geometry_pass.clone(),
            width,
            height,
        )?;

        let (fullscreen_quad_vertices, fullscreen_quad_indices) =
            Self::create_fullscreen_quad(&memory_allocator)?;

        info!("Deferred renderer initialized successfully");

        Ok(Self {
            device,
            memory_allocator,
            descriptor_set_allocator,
            geometry_pass,
            lighting_pass,
            geometry_pipeline,
            lighting_pipeline,
            gbuffer: Some(gbuffer),
            fullscreen_quad_vertices,
            fullscreen_quad_indices,
            width,
            height,
        })
    }

    /// Creates the render pass for the geometry pass (G-buffer output).
    fn create_geometry_pass(device: &Arc<Device>) -> Result<Arc<RenderPass>> {
        debug!("Creating geometry pass render pass");

        vulkano::ordered_passes_renderpass!(
            device.clone(),
            attachments: {
                albedo: {
                    format: Format::R8G8B8A8_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                normal: {
                    format: Format::R16G16B16A16_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                metallic_roughness: {
                    format: Format::R8G8B8A8_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth: {
                    format: Format::D32_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            passes: [
                {
                    color: [albedo, normal, metallic_roughness],
                    depth_stencil: {depth},
                    input: []
                }
            ]
        )
        .map_err(|e| eyre::eyre!("Failed to create geometry pass: {}", e))
    }

    /// Creates the render pass for the lighting pass (swapchain output).
    fn create_lighting_pass(device: &Arc<Device>) -> Result<Arc<RenderPass>> {
        debug!("Creating lighting pass render pass");

        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: Format::R8G8B8A8_UNORM,
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
        .map_err(|e| eyre::eyre!("Failed to create lighting pass: {}", e))
    }

    /// Creates the graphics pipeline for the geometry pass.
    fn create_geometry_pipeline(
        device: &Arc<Device>,
        render_pass: &Arc<RenderPass>,
        extent: [u32; 2],
    ) -> Result<Arc<GraphicsPipeline>> {
        debug!("Creating geometry pipeline");

        let vs_module = crate::shaders::deferred_geometry_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load deferred geometry vertex shader: {}", e))?;

        let fs_module = crate::shaders::deferred_geometry_fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load deferred geometry fragment shader: {}", e))?;

        let vs_entry = vs_module
            .entry_point("main")
            .ok_or_else(|| eyre::eyre!("Vertex shader main entry point not found"))?;

        let fs_entry = fs_module
            .entry_point("main")
            .ok_or_else(|| eyre::eyre!("Fragment shader main entry point not found"))?;

        let stages = [
            PipelineShaderStageCreateInfo::new(vs_entry.clone()),
            PipelineShaderStageCreateInfo::new(fs_entry),
        ];

        let vertex_input_state = Vertex3D::per_vertex()
            .definition(&vs_entry)
            .map_err(|e| eyre::eyre!("Failed to create vertex input state: {}", e))?;

        let mut layout_create_infos = PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages);

        if let Some(set_0) = layout_create_infos.set_layouts.get_mut(0) {
            if let Some(binding) = set_0.bindings.get_mut(&1) {
                binding.descriptor_type =
                    vulkano::descriptor_set::layout::DescriptorType::UniformBufferDynamic;
            }
        }

        let layout_create_infos = layout_create_infos
            .into_pipeline_layout_create_info(device.clone())
            .map_err(|e| eyre::eyre!("Failed to create pipeline layout info: {}", e))?;

        let layout = PipelineLayout::new(device.clone(), layout_create_infos)
            .map_err(|e| eyre::eyre!("Failed to create pipeline layout: {}", e))?;

        let subpass = Subpass::from(render_pass.clone(), 0)
            .ok_or_else(|| eyre::eyre!("Failed to get subpass"))?;

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [extent[0] as f32, extent[1] as f32],
            depth_range: 0.0..=1.0,
        };

        let create_info = GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState {
                viewports: [viewport].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState::default()),
            multisample_state: Some(MultisampleState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState {
                    compare_op: CompareOp::Less,
                    write_enable: true,
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
        };

        GraphicsPipeline::new(device.clone(), None, create_info)
            .map_err(|e| eyre::eyre!("Failed to create geometry pipeline: {}", e))
    }

    /// Creates the graphics pipeline for the lighting pass.
    fn create_lighting_pipeline(
        device: &Arc<Device>,
        render_pass: &Arc<RenderPass>,
        extent: [u32; 2],
    ) -> Result<Arc<GraphicsPipeline>> {
        debug!("Creating lighting pipeline");

        let vs_module = crate::shaders::deferred_lighting_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load deferred lighting vertex shader: {}", e))?;

        let fs_module = crate::shaders::deferred_lighting_fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load deferred lighting fragment shader: {}", e))?;

        let vs_entry = vs_module
            .entry_point("main")
            .ok_or_else(|| eyre::eyre!("Vertex shader main entry point not found"))?;

        let fs_entry = fs_module
            .entry_point("main")
            .ok_or_else(|| eyre::eyre!("Fragment shader main entry point not found"))?;

        let stages = [
            PipelineShaderStageCreateInfo::new(vs_entry.clone()),
            PipelineShaderStageCreateInfo::new(fs_entry),
        ];

        let vertex_input_state = FullscreenVertex::per_vertex()
            .definition(&vs_entry)
            .map_err(|e| eyre::eyre!("Failed to create vertex input state: {}", e))?;

        let layout_create_infos = PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .map_err(|e| eyre::eyre!("Failed to create pipeline layout info: {}", e))?;

        let layout = PipelineLayout::new(device.clone(), layout_create_infos)
            .map_err(|e| eyre::eyre!("Failed to create pipeline layout: {}", e))?;

        let subpass = Subpass::from(render_pass.clone(), 0)
            .ok_or_else(|| eyre::eyre!("Failed to get subpass"))?;

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [extent[0] as f32, extent[1] as f32],
            depth_range: 0.0..=1.0,
        };

        let create_info = GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState {
                viewports: [viewport].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState::default()),
            multisample_state: Some(MultisampleState::default()),
            depth_stencil_state: None,
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            dynamic_state: [DynamicState::Viewport].into_iter().collect(),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        };

        GraphicsPipeline::new(device.clone(), None, create_info)
            .map_err(|e| eyre::eyre!("Failed to create lighting pipeline: {}", e))
    }

    /// Creates vertex and index buffers for full-screen quad.
    #[allow(clippy::type_complexity)]
    fn create_fullscreen_quad(
        memory_allocator: &Arc<StandardMemoryAllocator>,
    ) -> Result<(
        vulkano::buffer::Subbuffer<[FullscreenVertex]>,
        vulkano::buffer::Subbuffer<[u32]>,
    )> {
        let vertices = [
            FullscreenVertex {
                position: [-1.0, -1.0],
                uv: [0.0, 0.0],
            },
            FullscreenVertex {
                position: [1.0, -1.0],
                uv: [1.0, 0.0],
            },
            FullscreenVertex {
                position: [1.0, 1.0],
                uv: [1.0, 1.0],
            },
            FullscreenVertex {
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

    /// Resizes the G-buffer to match new dimensions.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        info!("Resizing deferred renderer: {}x{}", width, height);

        self.gbuffer = Some(GBuffer::new(
            self.device.clone(),
            self.memory_allocator.clone(),
            self.geometry_pass.clone(),
            width,
            height,
        )?);

        self.width = width;
        self.height = height;

        Ok(())
    }

    /// Renders the scene using deferred rendering.
    ///
    /// This performs two passes:
    /// 1. Geometry pass: Renders meshes to G-buffer
    /// 2. Lighting pass: Accumulates lighting from G-buffer to output framebuffer
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        builder: &mut AutoCommandBufferBuilder<impl vulkano::command_buffer::allocator::CommandBufferAllocator>,
        output_framebuffer: Arc<Framebuffer>,
        viewport: Viewport,
        draw_commands: &[DrawCommand],
        view_proj_buffer: vulkano::buffer::Subbuffer<uniform_buffer::ViewProjectionUniforms>,
        dynamic_uniform_buffer: &crate::uniform_buffer::DynamicUniformBuffer,
        mesh_manager: &mesh::MeshAssetManager,
        texture_manager: &crate::texture::TextureManager,
        lighting_buffer: vulkano::buffer::Subbuffer<lighting::LightingUniforms>,
    ) -> Result<()> {
        let gbuffer = self
            .gbuffer
            .as_ref()
            .ok_or_else(|| eyre::eyre!("G-buffer not initialized"))?;

        self.geometry_pass_render(
            builder,
            gbuffer,
            viewport.clone(),
            draw_commands,
            view_proj_buffer.clone(),
            dynamic_uniform_buffer,
            mesh_manager,
            texture_manager,
        )?;

        self.lighting_pass_render(
            builder,
            output_framebuffer,
            viewport,
            gbuffer,
            view_proj_buffer,
            lighting_buffer,
        )?;

        Ok(())
    }

    /// Executes the geometry pass, rendering meshes to the G-buffer.
    #[allow(clippy::too_many_arguments)]
    fn geometry_pass_render(
        &self,
        builder: &mut AutoCommandBufferBuilder<impl vulkano::command_buffer::allocator::CommandBufferAllocator>,
        gbuffer: &GBuffer,
        viewport: Viewport,
        draw_commands: &[DrawCommand],
        view_proj_buffer: vulkano::buffer::Subbuffer<uniform_buffer::ViewProjectionUniforms>,
        dynamic_uniform_buffer: &crate::uniform_buffer::DynamicUniformBuffer,
        mesh_manager: &mesh::MeshAssetManager,
        texture_manager: &crate::texture::TextureManager,
    ) -> Result<()> {
        trace!("Beginning geometry pass");

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some([0.0, 0.0, 0.0, 1.0].into()),
                        Some([0.0, 0.0, 0.0, 1.0].into()),
                        Some([0.0, 0.0, 0.0, 1.0].into()),
                        Some(1.0.into()),
                    ],
                    ..RenderPassBeginInfo::framebuffer(gbuffer.framebuffer.clone())
                },
                SubpassBeginInfo {
                    contents: vulkano::command_buffer::SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .map_err(|e| eyre::eyre!("Failed to begin geometry pass: {}", e))?;

        builder
            .bind_pipeline_graphics(self.geometry_pipeline.clone())
            .map_err(|e| eyre::eyre!("Failed to bind geometry pipeline: {}", e))?;

        builder
            .set_viewport(0, [viewport].into_iter().collect())
            .map_err(|e| eyre::eyre!("Failed to set viewport: {}", e))?;

        let default_texture = texture_manager
            .get_texture("_default_white")
            .ok_or_else(|| eyre::eyre!("Default white texture not found"))?;

        for (object_index, draw_cmd) in draw_commands.iter().enumerate() {
            let mesh = mesh_manager
                .get_mesh(&draw_cmd.mesh_id)
                .ok_or_else(|| eyre::eyre!("Mesh '{}' not found", draw_cmd.mesh_id))?;

            let texture = if let Some(ref tex_name) = draw_cmd.texture_name {
                texture_manager
                    .get_texture(tex_name)
                    .ok_or_else(|| eyre::eyre!("Texture '{}' not found", tex_name))?
            } else {
                default_texture
            };

            let material_props = draw_cmd
                .material_properties
                .unwrap_or_else(material::MaterialProperties::default);

            let material_buffer = Buffer::from_data(
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
                material_props,
            )
            .map_err(|e| eyre::eyre!("Failed to create material buffer: {}", e))?;

            let descriptor_set = DescriptorSet::new(
                self.descriptor_set_allocator.clone(),
                self.geometry_pipeline.layout().set_layouts()[0].clone(),
                [
                    WriteDescriptorSet::buffer(0, view_proj_buffer.clone()),
                    WriteDescriptorSet::buffer(1, dynamic_uniform_buffer.buffer().clone()),
                    WriteDescriptorSet::image_view_sampler(
                        2,
                        texture.view.clone(),
                        texture.sampler.clone(),
                    ),
                ],
                [],
            )
            .map_err(|e| eyre::eyre!("Failed to create geometry descriptor set: {}", e))?;

            let material_set = DescriptorSet::new(
                self.descriptor_set_allocator.clone(),
                self.geometry_pipeline.layout().set_layouts()[1].clone(),
                [WriteDescriptorSet::buffer(0, material_buffer)],
                [],
            )
            .map_err(|e| eyre::eyre!("Failed to create material descriptor set: {}", e))?;

            builder
                .bind_vertex_buffers(0, mesh.vertex_buffer.clone())
                .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
                .bind_index_buffer(mesh.index_buffer.clone())
                .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?;

            let dynamic_offset = dynamic_uniform_buffer.get_dynamic_offset(object_index);

            unsafe {
                let set_with_offsets = vulkano::descriptor_set::DescriptorSetWithOffsets::new(
                    descriptor_set,
                    [dynamic_offset],
                );

                builder.bind_descriptor_sets_unchecked(
                    vulkano::pipeline::PipelineBindPoint::Graphics,
                    self.geometry_pipeline.layout().clone(),
                    0,
                    set_with_offsets,
                );

                builder
                    .bind_descriptor_sets(
                        vulkano::pipeline::PipelineBindPoint::Graphics,
                        self.geometry_pipeline.layout().clone(),
                        1,
                        material_set,
                    )
                    .map_err(|e| eyre::eyre!("Failed to bind material descriptor set: {}", e))?;

                builder
                    .draw_indexed(mesh.index_count, 1, 0, 0, 0)
                    .map_err(|e| eyre::eyre!("Failed to draw indexed: {}", e))?;
            }
        }

        builder
            .end_render_pass(SubpassEndInfo::default())
            .map_err(|e| eyre::eyre!("Failed to end geometry pass: {}", e))?;

        trace!("Geometry pass complete");

        Ok(())
    }

    /// Executes the lighting pass, accumulating lighting from G-buffer to output.
    #[allow(clippy::too_many_arguments)]
    fn lighting_pass_render(
        &self,
        builder: &mut AutoCommandBufferBuilder<impl vulkano::command_buffer::allocator::CommandBufferAllocator>,
        output_framebuffer: Arc<Framebuffer>,
        viewport: Viewport,
        gbuffer: &GBuffer,
        view_proj_buffer: vulkano::buffer::Subbuffer<uniform_buffer::ViewProjectionUniforms>,
        lighting_buffer: vulkano::buffer::Subbuffer<lighting::LightingUniforms>,
    ) -> Result<()> {
        trace!("Beginning lighting pass");

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.1, 0.2, 0.3, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(output_framebuffer)
                },
                SubpassBeginInfo {
                    contents: vulkano::command_buffer::SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .map_err(|e| eyre::eyre!("Failed to begin lighting pass: {}", e))?;

        builder
            .bind_pipeline_graphics(self.lighting_pipeline.clone())
            .map_err(|e| eyre::eyre!("Failed to bind lighting pipeline: {}", e))?;

        builder
            .set_viewport(0, [viewport].into_iter().collect())
            .map_err(|e| eyre::eyre!("Failed to set viewport: {}", e))?;

        let sampler = Sampler::new(
            self.device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Nearest,
                min_filter: Filter::Nearest,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create G-buffer sampler: {}", e))?;

        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            self.lighting_pipeline.layout().set_layouts()[0].clone(),
            [
                WriteDescriptorSet::image_view_sampler(
                    0,
                    gbuffer.albedo.clone(),
                    sampler.clone(),
                ),
                WriteDescriptorSet::image_view_sampler(1, gbuffer.normal.clone(), sampler.clone()),
                WriteDescriptorSet::image_view_sampler(
                    2,
                    gbuffer.metallic_roughness.clone(),
                    sampler.clone(),
                ),
                WriteDescriptorSet::image_view_sampler(3, gbuffer.depth.clone(), sampler),
                WriteDescriptorSet::buffer(4, view_proj_buffer),
                WriteDescriptorSet::buffer(5, lighting_buffer),
            ],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create lighting descriptor set: {}", e))?;

        builder
            .bind_vertex_buffers(0, self.fullscreen_quad_vertices.clone())
            .map_err(|e| eyre::eyre!("Failed to bind fullscreen quad vertex buffer: {}", e))?
            .bind_index_buffer(self.fullscreen_quad_indices.clone())
            .map_err(|e| eyre::eyre!("Failed to bind fullscreen quad index buffer: {}", e))?;

        builder
            .bind_descriptor_sets(
                vulkano::pipeline::PipelineBindPoint::Graphics,
                self.lighting_pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .map_err(|e| eyre::eyre!("Failed to bind lighting descriptor set: {}", e))?;

        unsafe {
            builder
                .draw_indexed(6, 1, 0, 0, 0)
                .map_err(|e| eyre::eyre!("Failed to draw fullscreen quad: {}", e))?;
        }

        builder
            .end_render_pass(SubpassEndInfo::default())
            .map_err(|e| eyre::eyre!("Failed to end lighting pass: {}", e))?;

        trace!("Lighting pass complete");

        Ok(())
    }
}
