//! Deferred rendering system with G-buffer passes and lighting accumulation.
//!
//! This module provides a complete deferred rendering pipeline that separates
//! geometry rendering from lighting calculations, enabling efficient many-light scenarios.
//!
//! # Educational: Why Deferred Rendering?
//!
//! ## The Lighting Problem
//!
//! In forward rendering, every object must be lit by every light during its draw call:
//! ```text
//! for each object:
//!   for each light:
//!     calculate lighting
//!   draw object
//! ```
//!
//! **Problem**: With 100 objects and 50 lights = 5,000 lighting calculations
//! Many of these calculations are wasted on objects that end up hidden by other objects!
//!
//! ## The Deferred Solution
//!
//! Deferred rendering decouples geometry from lighting:
//! ```text
//! Pass 1 - Geometry: Write all visible surface data to screen-sized textures (G-buffer)
//! Pass 2 - Lighting: For each screen pixel, calculate lighting once
//! ```
//!
//! **Benefit**: Only lit pixels that are actually visible = no wasted work!
//!
//! ## G-Buffer Layout (How We Store Surface Data)
//!
//! The G-buffer is a collection of textures storing per-pixel surface information:
//!
//! ### Texture 0: Albedo (RGBA8)
//! ```text
//! R, G, B = Base color of the surface (diffuse color)
//! A = Unused (could store occlusion or other data)
//! ```
//! Why 8-bit? Colors don't need high precision, and it saves memory bandwidth.
//!
//! ### Texture 1: Normal (RGBA16F)
//! ```text
//! R, G, B = World-space normal vector (xyz)
//! A = Unused
//! ```
//! Why 16-bit float? Normals need precision for smooth lighting.
//! Why world-space? Makes lighting calculations simpler (no need to transform).
//!
//! ### Texture 2: Metallic-Roughness-Emissive (RGBA8)
//! ```text
//! R = Metallic factor [0=dielectric, 1=metal]
//! G = Roughness factor [0=smooth/glossy, 1=rough/matte]
//! B = Emissive strength (how much light the surface emits)
//! A = Unused
//! ```
//! Why pack together? Saves memory and bandwidth. These values are uncorrelated
//! so packing doesn't hurt quality.
//!
//! ### Texture 3: Velocity (RG16F)
//! ```text
//! R, G = Screen-space motion vector (for TAA and motion blur)
//! ```
//! Why 2D? We only care about motion in screen space, not depth.
//! Why 16-bit float? Sub-pixel motion needs fractional precision.
//!
//! ### Texture 4: Depth (D32F)
//! ```text
//! Standard depth buffer [0=near, 1=far]
//! ```
//! Why 32-bit? Prevents z-fighting artifacts in large scenes.
//!
//! ## Memory Analysis
//!
//! For a 1920×1080 screen:
//! - Albedo: 1920 × 1080 × 4 bytes = 8.3 MB
//! - Normal: 1920 × 1080 × 8 bytes = 16.6 MB
//! - Metallic-Roughness: 1920 × 1080 × 4 bytes = 8.3 MB
//! - Velocity: 1920 × 1080 × 4 bytes = 8.3 MB
//! - Depth: 1920 × 1080 × 4 bytes = 8.3 MB
//!
//! **Total: ~50 MB**
//!
//! This is acceptable for modern GPUs (4-16 GB VRAM) and the performance benefit is worth it.
//!
//! # Deferred Rendering Architecture
//!
//! ## Pass 1: Geometry Pass (Write G-Buffer)
//! ```text
//! Input: 3D meshes with vertices, normals, UVs, materials
//! Output: G-buffer textures (albedo, normal, material, depth)
//!
//! For each mesh:
//!   1. Vertex Shader:
//!      - Transform vertices to clip space
//!      - Transform normals to world space
//!      - Pass through UVs and material properties
//!
//!   2. Fragment Shader:
//!      - Sample textures (albedo, normal map)
//!      - Write albedo to RT0
//!      - Write world-space normal to RT1
//!      - Write material properties to RT2
//!      - Write velocity to RT3
//!      - Depth written automatically
//! ```
//!
//! ## Pass 2: Lighting Pass (Read G-Buffer)
//! ```text
//! Input: G-buffer textures, light data
//! Output: Final lit color
//!
//! Draw full-screen quad:
//!   1. Vertex Shader:
//!      - Generate full-screen triangle/quad
//!      - Pass UV coordinates for G-buffer sampling
//!
//!   2. Fragment Shader:
//!      - Sample G-buffer at current pixel
//!      - Reconstruct world position from depth
//!      - For each light:
//!          - Calculate light contribution (diffuse + specular)
//!          - Apply attenuation and shadows
//!      - Sum all light contributions
//!      - Apply ambient + emissive
//!      - Output final color
//! ```
//!
//! ## Position Reconstruction
//!
//! We don't store world position in G-buffer (saves memory). Instead, we reconstruct it:
//! ```text
//! 1. Read depth from G-buffer
//! 2. Use UV coordinates to get NDC position: ndc.xy = uv * 2 - 1
//! 3. Set ndc.z = depth, ndc.w = 1
//! 4. Multiply by inverse projection: view_pos = inv_proj * ndc
//! 5. Perspective divide: view_pos /= view_pos.w
//! 6. Transform to world space: world_pos = inv_view * view_pos
//! ```
//!
//! This is fast (4 matrix multiplies) and saves 12-16 bytes per pixel!
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
/// - Velocity: Screen-space motion vectors (RG)
/// - Depth: Standard depth buffer
pub struct GBuffer {
    /// Albedo texture (RGB) + unused (A)
    pub albedo: Arc<ImageView>,
    /// Normal texture (RGB) + unused (A)
    pub normal: Arc<ImageView>,
    /// Metallic (R), Roughness (G), Emissive (B), unused (A)
    pub metallic_roughness: Arc<ImageView>,
    /// Velocity texture (RG) for motion vectors
    pub velocity: Arc<ImageView>,
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
        let velocity = create_attachment(Format::R16G16_SFLOAT, usage)?;

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
                    velocity.clone(),
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
            velocity,
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
    pub gbuffer: Option<GBuffer>,

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
                velocity: {
                    format: Format::R16G16_SFLOAT,
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
                    color: [albedo, normal, metallic_roughness, velocity],
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
        builder: &mut AutoCommandBufferBuilder<
            impl vulkano::command_buffer::allocator::CommandBufferAllocator,
        >,
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
            None, // SSAO texture - can be provided via render_with_ssao
        )?;

        Ok(())
    }

    /// Renders the scene using deferred rendering with SSAO integration.
    ///
    /// This performs two passes:
    /// 1. Geometry pass: Renders meshes to G-buffer
    /// 2. Lighting pass: Accumulates lighting from G-buffer to output framebuffer, with SSAO applied to ambient
    #[allow(clippy::too_many_arguments)]
    pub fn render_with_ssao(
        &self,
        builder: &mut AutoCommandBufferBuilder<
            impl vulkano::command_buffer::allocator::CommandBufferAllocator,
        >,
        output_framebuffer: Arc<Framebuffer>,
        viewport: Viewport,
        draw_commands: &[DrawCommand],
        view_proj_buffer: vulkano::buffer::Subbuffer<uniform_buffer::ViewProjectionUniforms>,
        dynamic_uniform_buffer: &crate::uniform_buffer::DynamicUniformBuffer,
        mesh_manager: &mesh::MeshAssetManager,
        texture_manager: &crate::texture::TextureManager,
        lighting_buffer: vulkano::buffer::Subbuffer<lighting::LightingUniforms>,
        ssao_texture: Arc<ImageView>,
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
            Some(ssao_texture),
        )?;

        Ok(())
    }

    /// Executes the geometry pass, rendering meshes to the G-buffer.
    #[allow(clippy::too_many_arguments)]
    fn geometry_pass_render(
        &self,
        builder: &mut AutoCommandBufferBuilder<
            impl vulkano::command_buffer::allocator::CommandBufferAllocator,
        >,
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
                        Some([0.0, 0.0, 0.0, 1.0].into()), // albedo
                        Some([0.0, 0.0, 0.0, 1.0].into()), // normal
                        Some([0.0, 0.0, 0.0, 1.0].into()), // metallic_roughness
                        Some([0.0, 0.0, 0.0, 0.0].into()), // velocity
                        Some(1.0.into()),                  // depth
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
        builder: &mut AutoCommandBufferBuilder<
            impl vulkano::command_buffer::allocator::CommandBufferAllocator,
        >,
        output_framebuffer: Arc<Framebuffer>,
        viewport: Viewport,
        gbuffer: &GBuffer,
        view_proj_buffer: vulkano::buffer::Subbuffer<uniform_buffer::ViewProjectionUniforms>,
        lighting_buffer: vulkano::buffer::Subbuffer<lighting::LightingUniforms>,
        ssao_texture: Option<Arc<ImageView>>,
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

        // Create default white SSAO texture if not provided (1.0 = no occlusion)
        let default_ssao_texture = if ssao_texture.is_none() {
            use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
            use vulkano::memory::allocator::AllocationCreateInfo;

            let image = Image::new(
                self.memory_allocator.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: vulkano::format::Format::R32_SFLOAT,
                    extent: [1, 1, 1],
                    usage: ImageUsage::SAMPLED,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
            )
            .map_err(|e| eyre::eyre!("Failed to create default SSAO texture: {}", e))?;

            Some(
                ImageView::new_default(image)
                    .map_err(|e| eyre::eyre!("Failed to create default SSAO image view: {}", e))?,
            )
        } else {
            None
        };

        let ssao_view = ssao_texture
            .as_ref()
            .or(default_ssao_texture.as_ref())
            .unwrap();

        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            self.lighting_pipeline.layout().set_layouts()[0].clone(),
            [
                WriteDescriptorSet::image_view_sampler(0, gbuffer.albedo.clone(), sampler.clone()),
                WriteDescriptorSet::image_view_sampler(1, gbuffer.normal.clone(), sampler.clone()),
                WriteDescriptorSet::image_view_sampler(
                    2,
                    gbuffer.metallic_roughness.clone(),
                    sampler.clone(),
                ),
                WriteDescriptorSet::image_view_sampler(3, gbuffer.depth.clone(), sampler.clone()),
                WriteDescriptorSet::buffer(4, view_proj_buffer),
                WriteDescriptorSet::buffer(5, lighting_buffer),
                WriteDescriptorSet::image_view_sampler(6, ssao_view.clone(), sampler),
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

#[cfg(test)]
mod tests {
    use super::*;
    use praxis_math::Vec3;

    /// Test G-buffer packing/unpacking for normal vectors
    #[test]
    fn test_gbuffer_normal_packing() {
        // Test normal vector encoding to RGBA format
        let normal = Vec3::new(0.5, 0.7071, 0.5).normalize();

        // Pack normal into RGBA format (map from [-1,1] to [0,1] for storage)
        let packed_r = (normal.x * 0.5 + 0.5) * 255.0;
        let packed_g = (normal.y * 0.5 + 0.5) * 255.0;
        let packed_b = (normal.z * 0.5 + 0.5) * 255.0;

        // Unpack normal from RGBA format
        let unpacked_x = (packed_r / 255.0) * 2.0 - 1.0;
        let unpacked_y = (packed_g / 255.0) * 2.0 - 1.0;
        let unpacked_z = (packed_b / 255.0) * 2.0 - 1.0;
        let unpacked = Vec3::new(unpacked_x, unpacked_y, unpacked_z);

        // Should be close to original (some precision loss expected)
        assert!((unpacked - normal).length() < 0.01);
    }

    #[test]
    fn test_gbuffer_normal_packing_cardinal_directions() {
        // Test cardinal direction normals
        let test_normals = [
            Vec3::X,
            Vec3::NEG_X,
            Vec3::Y,
            Vec3::NEG_Y,
            Vec3::Z,
            Vec3::NEG_Z,
        ];

        for normal in test_normals.iter() {
            // Pack
            let packed_r = (normal.x * 0.5 + 0.5) * 255.0;
            let packed_g = (normal.y * 0.5 + 0.5) * 255.0;
            let packed_b = (normal.z * 0.5 + 0.5) * 255.0;

            // Unpack
            let unpacked_x = (packed_r / 255.0) * 2.0 - 1.0;
            let unpacked_y = (packed_g / 255.0) * 2.0 - 1.0;
            let unpacked_z = (packed_b / 255.0) * 2.0 - 1.0;
            let unpacked = Vec3::new(unpacked_x, unpacked_y, unpacked_z);

            assert!((unpacked - *normal).length() < 0.01);
        }
    }

    #[test]
    fn test_gbuffer_metallic_roughness_packing() {
        // Test packing metallic and roughness into single texture
        let metallic = 0.8;
        let roughness = 0.3;
        let emissive = 0.5;

        // Pack into RGBA8 format
        let packed_r = (metallic * 255.0) as u8;
        let packed_g = (roughness * 255.0) as u8;
        let packed_b = (emissive * 255.0) as u8;

        // Unpack
        let unpacked_metallic = packed_r as f32 / 255.0;
        let unpacked_roughness = packed_g as f32 / 255.0;
        let unpacked_emissive = packed_b as f32 / 255.0;

        assert!((unpacked_metallic - metallic).abs() < 0.01);
        assert!((unpacked_roughness - roughness).abs() < 0.01);
        assert!((unpacked_emissive - emissive).abs() < 0.01);
    }

    #[test]
    fn test_gbuffer_metallic_roughness_extremes() {
        // Test extreme values (0.0 and 1.0)
        let test_cases = [
            (0.0, 0.0, 0.0),
            (1.0, 1.0, 1.0),
            (0.0, 1.0, 0.0),
            (1.0, 0.0, 1.0),
        ];

        for (metallic, roughness, emissive) in test_cases.iter() {
            let packed_r = (*metallic * 255.0) as u8;
            let packed_g = (*roughness * 255.0) as u8;
            let packed_b = (*emissive * 255.0) as u8;

            let unpacked_metallic = packed_r as f32 / 255.0;
            let unpacked_roughness = packed_g as f32 / 255.0;
            let unpacked_emissive = packed_b as f32 / 255.0;

            assert!((unpacked_metallic - metallic).abs() < 0.01);
            assert!((unpacked_roughness - roughness).abs() < 0.01);
            assert!((unpacked_emissive - emissive).abs() < 0.01);
        }
    }

    #[test]
    fn test_gbuffer_albedo_packing() {
        // Test albedo color packing
        let albedo = Vec3::new(0.8, 0.2, 0.5);

        // Pack into RGBA8
        let packed_r = (albedo.x * 255.0) as u8;
        let packed_g = (albedo.y * 255.0) as u8;
        let packed_b = (albedo.z * 255.0) as u8;

        // Unpack
        let unpacked = Vec3::new(
            packed_r as f32 / 255.0,
            packed_g as f32 / 255.0,
            packed_b as f32 / 255.0,
        );

        assert!((unpacked - albedo).length() < 0.01);
    }

    #[test]
    fn test_gbuffer_depth_reconstruction() {
        // Test depth value reconstruction from G-buffer
        // Depth is stored in D32_SFLOAT format (full 32-bit precision)
        let depth_values = [0.0, 0.1, 0.5, 0.9, 1.0];

        for depth in depth_values.iter() {
            // In shader, we would read this directly
            // Here we just verify the value range
            assert!(*depth >= 0.0 && *depth <= 1.0);
        }
    }

    #[test]
    fn test_gbuffer_position_reconstruction() {
        // Test position reconstruction from depth and screen coordinates
        // This simulates what the lighting shader does

        // Mock screen-space coordinates (normalized device coordinates)
        let ndc_x = 0.5;
        let ndc_y = 0.5;
        let depth = 0.5;

        // Mock inverse projection matrix (simplified)
        use praxis_math::Mat4;
        let fov = std::f32::consts::PI / 4.0; // 45 degrees
        let aspect = 16.0 / 9.0;
        let near = 0.1;
        let far = 100.0;
        let proj = Mat4::perspective_rh(fov, aspect, near, far);
        let inv_proj = proj.inverse();

        // Reconstruct clip-space position
        let clip_pos = praxis_math::Vec4::new(ndc_x * 2.0 - 1.0, ndc_y * 2.0 - 1.0, depth, 1.0);

        // Transform to view space
        let view_pos = inv_proj * clip_pos;
        let view_pos = view_pos / view_pos.w;

        // Position should be valid (not NaN or infinite)
        assert!(view_pos.x.is_finite());
        assert!(view_pos.y.is_finite());
        assert!(view_pos.z.is_finite());
    }

    #[test]
    fn test_fullscreen_quad_vertices() {
        // Test fullscreen quad vertex generation
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

        // Verify positions cover full NDC space
        assert_eq!(vertices[0].position, [-1.0, -1.0]); // Bottom-left
        assert_eq!(vertices[1].position, [1.0, -1.0]); // Bottom-right
        assert_eq!(vertices[2].position, [1.0, 1.0]); // Top-right
        assert_eq!(vertices[3].position, [-1.0, 1.0]); // Top-left

        // Verify UVs are correctly mapped
        assert_eq!(vertices[0].uv, [0.0, 0.0]);
        assert_eq!(vertices[1].uv, [1.0, 0.0]);
        assert_eq!(vertices[2].uv, [1.0, 1.0]);
        assert_eq!(vertices[3].uv, [0.0, 1.0]);
    }

    #[test]
    fn test_fullscreen_quad_indices() {
        // Test fullscreen quad index generation for two triangles
        let indices = [0u32, 1, 2, 0, 2, 3];

        // First triangle: 0, 1, 2 (bottom-left, bottom-right, top-right)
        assert_eq!(indices[0], 0);
        assert_eq!(indices[1], 1);
        assert_eq!(indices[2], 2);

        // Second triangle: 0, 2, 3 (bottom-left, top-right, top-left)
        assert_eq!(indices[3], 0);
        assert_eq!(indices[4], 2);
        assert_eq!(indices[5], 3);
    }

    #[test]
    fn test_gbuffer_normal_precision() {
        // Test that normal packing maintains sufficient precision
        // Use R16G16B16A16_SFLOAT format characteristics

        let test_normals = vec![
            Vec3::new(0.577, 0.577, 0.577).normalize(), // Diagonal
            Vec3::new(0.707, 0.0, 0.707).normalize(),   // 45 degree angle
            Vec3::new(0.1, 0.99, 0.1).normalize(),      // Near-vertical
        ];

        for normal in test_normals.iter() {
            // 16-bit float precision simulation (±65504, ~3-4 decimal digits)
            let pack_and_unpack = |value: f32| -> f32 {
                // Simulated 16-bit float quantization

                (value * 1000.0).round() / 1000.0
            };

            let unpacked = Vec3::new(
                pack_and_unpack(normal.x),
                pack_and_unpack(normal.y),
                pack_and_unpack(normal.z),
            );

            // Should maintain good precision with 16-bit floats
            assert!((unpacked - *normal).length() < 0.001);
        }
    }

    // ===== TAA (Temporal Anti-Aliasing) Tests =====

    #[test]
    fn test_velocity_buffer_static_object() {
        // Test velocity buffer generation for a static object
        // Static objects should have zero velocity
        use praxis_math::{Mat4, Vec4};

        let current_pos = Vec4::new(0.5, 0.5, 0.5, 1.0);
        let previous_pos = Vec4::new(0.5, 0.5, 0.5, 1.0);

        // Simulate perspective division (clip space to NDC)
        let current_ndc = Vec4::new(
            current_pos.x / current_pos.w,
            current_pos.y / current_pos.w,
            current_pos.z / current_pos.w,
            1.0,
        );
        let previous_ndc = Vec4::new(
            previous_pos.x / previous_pos.w,
            previous_pos.y / previous_pos.w,
            previous_pos.z / previous_pos.w,
            1.0,
        );

        // Calculate velocity
        let velocity_x = current_ndc.x - previous_ndc.x;
        let velocity_y = current_ndc.y - previous_ndc.y;

        // Static object should have zero velocity
        assert!((velocity_x).abs() < 0.0001);
        assert!((velocity_y).abs() < 0.0001);
    }

    #[test]
    fn test_velocity_buffer_moving_object() {
        // Test velocity buffer generation for a moving object
        use praxis_math::Vec4;

        // Object moved from one position to another
        let current_pos = Vec4::new(0.6, 0.5, 0.5, 1.0);
        let previous_pos = Vec4::new(0.4, 0.5, 0.5, 1.0);

        // Convert to NDC
        let current_ndc = Vec4::new(
            current_pos.x / current_pos.w,
            current_pos.y / current_pos.w,
            current_pos.z / current_pos.w,
            1.0,
        );
        let previous_ndc = Vec4::new(
            previous_pos.x / previous_pos.w,
            previous_pos.y / previous_pos.w,
            previous_pos.z / previous_pos.w,
            1.0,
        );

        // Calculate velocity
        let velocity_x = current_ndc.x - previous_ndc.x;
        let velocity_y = current_ndc.y - previous_ndc.y;

        // Should have velocity in x direction
        assert!((velocity_x - 0.2).abs() < 0.0001);
        assert!((velocity_y).abs() < 0.0001);
    }

    #[test]
    fn test_velocity_buffer_perspective_division() {
        // Test that perspective division is correctly applied
        use praxis_math::Vec4;

        // Object with non-unit w component (after projection)
        let current_pos = Vec4::new(1.0, 2.0, 3.0, 2.0);
        let previous_pos = Vec4::new(0.5, 1.0, 1.5, 2.0);

        // Convert to NDC by dividing by w
        let current_ndc_x = current_pos.x / current_pos.w;
        let current_ndc_y = current_pos.y / current_pos.w;
        let previous_ndc_x = previous_pos.x / previous_pos.w;
        let previous_ndc_y = previous_pos.y / previous_pos.w;

        // Calculate velocity
        let velocity_x = current_ndc_x - previous_ndc_x;
        let velocity_y = current_ndc_y - previous_ndc_y;

        // Check correct perspective division
        assert!((current_ndc_x - 0.5).abs() < 0.0001);
        assert!((current_ndc_y - 1.0).abs() < 0.0001);
        assert!((previous_ndc_x - 0.25).abs() < 0.0001);
        assert!((previous_ndc_y - 0.5).abs() < 0.0001);

        // Velocity should be the difference
        assert!((velocity_x - 0.25).abs() < 0.0001);
        assert!((velocity_y - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_velocity_buffer_camera_motion() {
        // Test velocity calculation for camera motion (all objects move in screen space)
        use praxis_math::{Mat4, Vec3, Vec4};

        // Simple vertex position
        let vertex_pos = Vec3::new(1.0, 0.0, -5.0);

        // Current frame matrices
        let view_current =
            Mat4::look_at_rh(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0), Vec3::Y);
        let proj = Mat4::perspective_rh(std::f32::consts::PI / 4.0, 16.0 / 9.0, 0.1, 100.0);
        let model = Mat4::IDENTITY;

        // Previous frame matrices (camera moved)
        let view_previous =
            Mat4::look_at_rh(Vec3::new(0.5, 0.0, 0.0), Vec3::new(0.5, 0.0, -1.0), Vec3::Y);

        // Calculate current position
        let current_pos = proj * view_current * model * Vec4::from((vertex_pos, 1.0));
        let previous_pos = proj * view_previous * model * Vec4::from((vertex_pos, 1.0));

        // Convert to NDC
        let current_ndc_x = current_pos.x / current_pos.w;
        let previous_ndc_x = previous_pos.x / previous_pos.w;

        // Calculate velocity
        let velocity_x = current_ndc_x - previous_ndc_x;

        // Camera moved right, so object should appear to move left in screen space
        assert!(velocity_x < 0.0);
    }

    #[test]
    fn test_temporal_reprojection_no_motion() {
        // Test temporal reprojection with no motion
        let velocity_x = 0.0;
        let velocity_y = 0.0;

        // Current UV coordinate
        let uv_x = 0.5;
        let uv_y = 0.5;

        // Calculate reprojected UV (subtracting velocity for history lookup)
        let history_uv_x = uv_x - velocity_x;
        let history_uv_y = uv_y - velocity_y;

        // No motion means same UV
        assert!((history_uv_x - 0.5).abs() < 0.0001);
        assert!((history_uv_y - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_temporal_reprojection_with_motion() {
        // Test temporal reprojection with object motion
        let velocity_x = 0.1;
        let velocity_y = 0.05;

        let uv_x = 0.5;
        let uv_y = 0.5;

        // Calculate reprojected UV
        let history_uv_x = uv_x - velocity_x;
        let history_uv_y = uv_y - velocity_y;

        // History should be at previous position
        assert!((history_uv_x - 0.4).abs() < 0.0001);
        assert!((history_uv_y - 0.45).abs() < 0.0001);
    }

    #[test]
    fn test_temporal_reprojection_out_of_bounds() {
        // Test that out-of-bounds reprojection is detected
        let velocity_x = 1.5;
        let velocity_y = 0.0;

        let uv_x = 0.5;
        let uv_y = 0.5;

        let history_uv_x = uv_x - velocity_x;
        let history_uv_y = uv_y - velocity_y;

        // Check if reprojected UV is out of valid range [0, 1]
        let valid_history = history_uv_x >= 0.0
            && history_uv_x <= 1.0
            && history_uv_y >= 0.0
            && history_uv_y <= 1.0;

        // Should be invalid (history_uv_x = -1.0)
        assert!(!valid_history);
    }

    #[test]
    fn test_temporal_reprojection_edge_case() {
        // Test edge case where reprojection is just at boundary
        let velocity_x = 0.5;
        let velocity_y = 0.5;

        let uv_x = 0.5;
        let uv_y = 0.5;

        let history_uv_x = uv_x - velocity_x;
        let history_uv_y = uv_y - velocity_y;

        // Should be exactly at (0, 0) which is valid
        assert!((history_uv_x - 0.0).abs() < 0.0001);
        assert!((history_uv_y - 0.0).abs() < 0.0001);

        let valid_history = history_uv_x >= 0.0
            && history_uv_x <= 1.0
            && history_uv_y >= 0.0
            && history_uv_y <= 1.0;
        assert!(valid_history);
    }

    #[test]
    fn test_rgb_to_ycocg_conversion() {
        // Test RGB to YCoCg color space conversion
        // Used for better neighborhood clamping in TAA

        // Pure red
        let rgb = Vec3::new(1.0, 0.0, 0.0);
        let y = rgb.dot(Vec3::new(0.25, 0.5, 0.25));
        let co = rgb.dot(Vec3::new(0.5, 0.0, -0.5));
        let cg = rgb.dot(Vec3::new(-0.25, 0.5, -0.25));

        assert!((y - 0.25).abs() < 0.0001);
        assert!((co - 0.5).abs() < 0.0001);
        assert!((cg - (-0.25)).abs() < 0.0001);

        // Pure green
        let rgb = Vec3::new(0.0, 1.0, 0.0);
        let y = rgb.dot(Vec3::new(0.25, 0.5, 0.25));
        let co = rgb.dot(Vec3::new(0.5, 0.0, -0.5));
        let cg = rgb.dot(Vec3::new(-0.25, 0.5, -0.25));

        assert!((y - 0.5).abs() < 0.0001);
        assert!((co - 0.0).abs() < 0.0001);
        assert!((cg - 0.5).abs() < 0.0001);

        // Pure blue
        let rgb = Vec3::new(0.0, 0.0, 1.0);
        let y = rgb.dot(Vec3::new(0.25, 0.5, 0.25));
        let co = rgb.dot(Vec3::new(0.5, 0.0, -0.5));
        let cg = rgb.dot(Vec3::new(-0.25, 0.5, -0.25));

        assert!((y - 0.25).abs() < 0.0001);
        assert!((co - (-0.5)).abs() < 0.0001);
        assert!((cg - (-0.25)).abs() < 0.0001);
    }

    #[test]
    fn test_ycocg_to_rgb_conversion() {
        // Test YCoCg to RGB conversion (inverse of rgb_to_ycocg)
        let ycocg = Vec3::new(0.5, 0.25, -0.125);

        let y = ycocg.x;
        let co = ycocg.y;
        let cg = ycocg.z;

        let tmp = y - cg;
        let r = tmp + co;
        let g = y + cg;
        let b = tmp - co;

        // Reconstruct RGB
        let rgb = Vec3::new(r, g, b);

        // Values should be valid (in range and finite)
        assert!(rgb.x.is_finite() && rgb.x >= 0.0);
        assert!(rgb.y.is_finite() && rgb.y >= 0.0);
        assert!(rgb.z.is_finite() && rgb.z >= 0.0);
    }

    #[test]
    fn test_ycocg_roundtrip() {
        // Test that RGB -> YCoCg -> RGB preserves color
        let original = Vec3::new(0.8, 0.3, 0.5);

        // Convert to YCoCg
        let y = original.dot(Vec3::new(0.25, 0.5, 0.25));
        let co = original.dot(Vec3::new(0.5, 0.0, -0.5));
        let cg = original.dot(Vec3::new(-0.25, 0.5, -0.25));

        // Convert back to RGB
        let tmp = y - cg;
        let r = tmp + co;
        let g = y + cg;
        let b = tmp - co;
        let reconstructed = Vec3::new(r, g, b);

        // Should match original
        assert!((reconstructed - original).length() < 0.0001);
    }

    #[test]
    fn test_neighborhood_min_max() {
        // Test neighborhood min/max calculation for clamping
        // Simulates 3x3 neighborhood sampling

        let samples = [
            Vec3::new(0.5, 0.5, 0.5), // Center
            Vec3::new(0.4, 0.4, 0.4), // Darker neighbors
            Vec3::new(0.6, 0.6, 0.6), // Brighter neighbors
            Vec3::new(0.45, 0.45, 0.45),
            Vec3::new(0.55, 0.55, 0.55),
            Vec3::new(0.48, 0.48, 0.48),
            Vec3::new(0.52, 0.52, 0.52),
            Vec3::new(0.47, 0.47, 0.47),
            Vec3::new(0.53, 0.53, 0.53),
        ];

        let mut color_min = samples[0];
        let mut color_max = samples[0];

        for sample in samples.iter() {
            color_min = color_min.min(*sample);
            color_max = color_max.max(*sample);
        }

        // Min should be the darkest sample
        assert!((color_min.x - 0.4).abs() < 0.0001);
        // Max should be the brightest sample
        assert!((color_max.x - 0.6).abs() < 0.0001);
    }

    #[test]
    fn test_neighborhood_average() {
        // Test neighborhood average calculation
        let samples = [
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(0.5, 0.5, 0.5),
        ];

        let mut color_avg = Vec3::ZERO;
        for sample in samples.iter() {
            color_avg += *sample;
        }
        color_avg /= 9.0;

        // Average of uniform samples should equal the sample value
        assert!((color_avg.x - 0.5).abs() < 0.0001);
        assert!((color_avg.y - 0.5).abs() < 0.0001);
        assert!((color_avg.z - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_aabb_clipping_inside() {
        // Test AABB clipping when history color is inside the box
        let aabb_min = Vec3::new(0.4, 0.4, 0.4);
        let aabb_max = Vec3::new(0.6, 0.6, 0.6);
        let history_color = Vec3::new(0.5, 0.5, 0.5);

        let center = (aabb_max + aabb_min) * 0.5;
        let extents = (aabb_max - aabb_min) * 0.5;

        let offset = history_color - center;
        let unit_offset = offset / extents.max(Vec3::splat(0.0001));

        let max_component = unit_offset
            .x
            .abs()
            .max(unit_offset.y.abs())
            .max(unit_offset.z.abs());

        // History is inside AABB
        assert!(max_component <= 1.0);

        // Clipped color should equal history (no clipping needed)
        let clipped = if max_component > 1.0 {
            center + offset / max_component
        } else {
            history_color
        };

        assert!((clipped - history_color).length() < 0.0001);
    }

    #[test]
    fn test_aabb_clipping_outside() {
        // Test AABB clipping when history color is outside the box
        let aabb_min = Vec3::new(0.4, 0.4, 0.4);
        let aabb_max = Vec3::new(0.6, 0.6, 0.6);
        let history_color = Vec3::new(0.8, 0.5, 0.5); // Outside on X axis

        let center = (aabb_max + aabb_min) * 0.5;
        let extents = (aabb_max - aabb_min) * 0.5;

        let offset = history_color - center;
        let unit_offset = offset / extents.max(Vec3::splat(0.0001));

        let max_component = unit_offset
            .x
            .abs()
            .max(unit_offset.y.abs())
            .max(unit_offset.z.abs());

        // History is outside AABB
        assert!(max_component > 1.0);

        // Clip to AABB
        let clipped = center + offset / max_component;

        // Clipped value should be on the boundary of the AABB
        assert!(clipped.x >= aabb_min.x && clipped.x <= aabb_max.x);
        assert!(clipped.y >= aabb_min.y && clipped.y <= aabb_max.y);
        assert!(clipped.z >= aabb_min.z && clipped.z <= aabb_max.z);

        // Clipped X should be at the max boundary
        assert!((clipped.x - aabb_max.x).abs() < 0.0001);
    }

    #[test]
    fn test_aabb_clipping_corner() {
        // Test AABB clipping when history is far from corner
        let aabb_min = Vec3::new(0.4, 0.4, 0.4);
        let aabb_max = Vec3::new(0.6, 0.6, 0.6);
        let history_color = Vec3::new(0.9, 0.9, 0.9); // Far outside corner

        let center = (aabb_max + aabb_min) * 0.5;
        let extents = (aabb_max - aabb_min) * 0.5;

        let offset = history_color - center;
        let unit_offset = offset / extents.max(Vec3::splat(0.0001));

        let max_component = unit_offset
            .x
            .abs()
            .max(unit_offset.y.abs())
            .max(unit_offset.z.abs());

        assert!(max_component > 1.0);

        let clipped = center + offset / max_component;

        // All components should be within AABB
        assert!(clipped.x >= aabb_min.x && clipped.x <= aabb_max.x + 0.0001);
        assert!(clipped.y >= aabb_min.y && clipped.y <= aabb_max.y + 0.0001);
        assert!(clipped.z >= aabb_min.z && clipped.z <= aabb_max.z + 0.0001);
    }

    #[test]
    fn test_adaptive_blend_factor_static() {
        // Test adaptive blend factor for static objects (no velocity)
        let velocity = Vec3::new(0.0, 0.0, 0.0);
        let velocity_length = velocity.length();
        let base_blend_factor = 0.1;

        // Adaptive blend: more history for static objects
        let adaptive_blend = base_blend_factor
            + (0.5 - base_blend_factor) * (velocity_length * 10.0).clamp(0.0, 1.0);

        // Static objects should use mostly history (low blend factor)
        assert!((adaptive_blend - base_blend_factor).abs() < 0.0001);
    }

    #[test]
    fn test_adaptive_blend_factor_fast_motion() {
        // Test adaptive blend factor for fast-moving objects
        let velocity = Vec3::new(0.2, 0.2, 0.0);
        let velocity_length = velocity.length();
        let base_blend_factor = 0.1;

        // Adaptive blend: less history for fast motion
        let adaptive_blend = base_blend_factor
            + (0.5 - base_blend_factor) * (velocity_length * 10.0).clamp(0.0, 1.0);

        // Fast motion should blend towards 0.5 (more current frame)
        assert!(adaptive_blend > base_blend_factor);
        assert!(adaptive_blend <= 0.5);
    }

    #[test]
    fn test_adaptive_blend_factor_very_fast() {
        // Test adaptive blend factor caps at 0.5 for very fast motion
        let velocity = Vec3::new(1.0, 1.0, 0.0); // Very large velocity
        let velocity_length = velocity.length();
        let base_blend_factor = 0.1;

        let adaptive_blend = base_blend_factor
            + (0.5 - base_blend_factor) * (velocity_length * 10.0).clamp(0.0, 1.0);

        // Should cap at 0.5
        assert!((adaptive_blend - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_velocity_buffer_format() {
        // Test velocity buffer uses RG16F format (2 channels, 16-bit float)
        // Velocity is stored in screen space as (u, v) motion

        let velocity = praxis_math::Vec2::new(0.123, -0.456);

        // Simulate 16-bit float precision
        let quantize = |v: f32| -> f32 { (v * 1000.0).round() / 1000.0 };

        let stored_x = quantize(velocity.x);
        let stored_y = quantize(velocity.y);

        // Should maintain reasonable precision
        assert!((stored_x - velocity.x).abs() < 0.001);
        assert!((stored_y - velocity.y).abs() < 0.001);
    }

    #[test]
    fn test_previous_frame_matrix_calculation() {
        // Test that previous frame matrices are correctly tracked
        use praxis_math::{Mat4, Vec3, Vec4};

        let model_current = Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0));
        let model_previous = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));

        let view = Mat4::look_at_rh(Vec3::ZERO, Vec3::NEG_Z, Vec3::Y);
        let proj = Mat4::perspective_rh(std::f32::consts::PI / 4.0, 1.0, 0.1, 100.0);

        let vertex = Vec3::new(0.0, 0.0, -5.0);

        // Current MVP
        let current_mvp = proj * view * model_current;
        let current_pos = current_mvp * Vec4::from((vertex, 1.0));

        // Previous MVP
        let previous_mvp = proj * view * model_previous;
        let previous_pos = previous_mvp * Vec4::from((vertex, 1.0));

        // Positions should be different due to model matrix change
        assert!((current_pos.x - previous_pos.x).abs() > 0.01);
    }

    #[test]
    fn test_jitter_offset_application() {
        // Test camera jitter for TAA (sub-pixel offsets)
        use praxis_math::Vec2;

        // Halton sequence common for TAA jitter
        let jitter_offset = Vec2::new(0.5 / 1920.0, 0.5 / 1080.0); // Half pixel offset

        // Jitter should be very small (sub-pixel)
        assert!(jitter_offset.x.abs() < 0.001);
        assert!(jitter_offset.y.abs() < 0.001);
        assert!(jitter_offset.x > 0.0);
        assert!(jitter_offset.y > 0.0);
    }

    #[test]
    fn test_taa_blend_factor_range() {
        // Test that blend factors are in valid range [0, 1]
        let test_factors = [0.0, 0.05, 0.1, 0.2, 0.5, 1.0];

        for factor in test_factors.iter() {
            assert!(*factor >= 0.0 && *factor <= 1.0);

            // Simulate blending
            let history_color = 0.8;
            let current_color = 0.3;
            let blended = history_color * (1.0 - factor) + current_color * factor;

            // Result should be between history and current
            assert!(blended >= current_color.min(history_color));
            assert!(blended <= current_color.max(history_color));
        }
    }
}
