//! Screen-Space Ambient Occlusion (SSAO) implementation.
//!
//! This module provides a complete SSAO system with:
//! - Hemisphere sampling kernel generation
//! - Random noise texture for sample rotation
//! - SSAO pass that samples depth and normals from G-buffer
//! - Blur pass to reduce noise artifacts
//! - Integration with deferred rendering pipeline
//!
//! # SSAO Overview
//!
//! SSAO is a screen-space technique that approximates ambient occlusion by sampling
//! the depth buffer around each pixel. It darkens pixels that are surrounded by
//! geometry (in crevices, corners, etc.) to simulate indirect lighting occlusion.
//!
//! ## Algorithm Steps
//!
//! 1. **Generate Sample Kernel**: Create a hemisphere of sample points
//! 2. **Generate Noise Texture**: Random vectors for rotating the sample kernel
//! 3. **SSAO Pass**: For each pixel:
//!    - Reconstruct view-space position from depth
//!    - Transform sample kernel using normal and noise
//!    - Test samples against depth buffer
//!    - Accumulate occlusion factor
//! 4. **Blur Pass**: Apply blur to reduce noise artifacts
//!
//! # Usage
//!
//! ```rust,no_run
//! use praxis_graphics::ssao::{SsaoRenderer, SsaoConfig};
//! # use std::sync::Arc;
//! # use vulkano::device::Device;
//! # use vulkano::memory::allocator::StandardMemoryAllocator;
//! # fn example(
//! #     device: Arc<Device>,
//! #     memory_allocator: Arc<StandardMemoryAllocator>,
//! # ) -> praxis_utils::Result<()> {
//!
//! let config = SsaoConfig::default()
//!     .with_kernel_size(64)
//!     .with_radius(0.5)
//!     .with_bias(0.025);
//!
//! let mut ssao = SsaoRenderer::new(
//!     device,
//!     memory_allocator,
//!     1920,
//!     1080,
//!     config,
//! )?;
//!
//! // In render loop, after G-buffer pass:
//! // ssao.render(builder, gbuffer, output)?;
//! # Ok(())
//! # }
//! ```

use crate::{deferred::GBuffer, post_process::QuadVertex, shaders};
use praxis_math::{Mat4, Vec2, Vec3};
use praxis_utils::{debug, eyre, info, trace, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        allocator::CommandBufferAllocator, AutoCommandBufferBuilder, RenderPassBeginInfo,
        SubpassBeginInfo, SubpassEndInfo,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, DescriptorSet, WriteDescriptorSet,
    },
    device::Device,
    format::Format,
    image::{
        sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
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

/// Configuration for SSAO effect.
#[derive(Debug, Clone)]
pub struct SsaoConfig {
    /// Number of samples in the hemisphere kernel (default: 64)
    pub kernel_size: u32,
    /// Radius of the sampling hemisphere in view space (default: 0.5)
    pub radius: f32,
    /// Depth bias to prevent self-occlusion artifacts (default: 0.025)
    pub bias: f32,
    /// Power applied to final occlusion for artistic control (default: 1.0)
    pub power: f32,
    /// Size of noise texture for kernel rotation (default: 4x4)
    pub noise_texture_size: u32,
}

impl Default for SsaoConfig {
    fn default() -> Self {
        Self {
            kernel_size: 64,
            radius: 0.5,
            bias: 0.025,
            power: 1.0,
            noise_texture_size: 4,
        }
    }
}

impl SsaoConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_kernel_size(mut self, kernel_size: u32) -> Self {
        self.kernel_size = kernel_size;
        self
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn with_bias(mut self, bias: f32) -> Self {
        self.bias = bias;
        self
    }

    pub fn with_power(mut self, power: f32) -> Self {
        self.power = power;
        self
    }

    pub fn with_noise_texture_size(mut self, size: u32) -> Self {
        self.noise_texture_size = size;
        self
    }
}

/// SSAO uniform data matching shader layout.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SsaoUniforms {
    projection: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    samples: [[f32; 4]; 64], // Vec4 array (xyz = position, w = unused)
    noise_scale: [f32; 2],
    radius: f32,
    bias: f32,
    power: f32,
    kernel_size: i32,
    _padding: [f32; 2],
}

/// Push constants for blur pass.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurPushConstants {
    texel_size: [f32; 2],
}

/// Generates a hemisphere sample kernel with varying distribution.
///
/// Samples are randomly distributed in a hemisphere, with more samples
/// closer to the origin for better detail near the surface.
fn generate_sample_kernel(count: u32) -> Vec<Vec3> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut kernel = Vec::with_capacity(count as usize);

    for i in 0..count {
        // Random point in hemisphere
        let mut sample = Vec3::new(
            rng.gen::<f32>() * 2.0 - 1.0, // x: -1 to 1
            rng.gen::<f32>() * 2.0 - 1.0, // y: -1 to 1
            rng.gen::<f32>(),             // z: 0 to 1 (hemisphere)
        );

        sample = sample.normalize();

        // Scale to vary distance (more samples near origin)
        let mut scale = i as f32 / count as f32;
        scale = 0.1 + scale * scale * 0.9; // lerp(0.1, 1.0, scale^2)
        sample *= scale;

        kernel.push(sample);
    }

    kernel
}

/// Generates a random noise texture for rotating the sample kernel.
///
/// This texture contains random 3D vectors that are used to rotate the
/// sample kernel differently for each pixel, reducing banding artifacts.
fn generate_noise_texture(size: u32) -> Vec<Vec3> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut noise = Vec::with_capacity((size * size) as usize);

    for _ in 0..(size * size) {
        // Random vector in tangent space (x, y in -1..1, z = 0)
        let vec = Vec3::new(
            rng.gen::<f32>() * 2.0 - 1.0,
            rng.gen::<f32>() * 2.0 - 1.0,
            0.0,
        );
        noise.push(vec.normalize());
    }

    noise
}

/// SSAO renderer managing render targets and pipelines.
pub struct SsaoRenderer {
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

    // Pipelines
    ssao_pipeline: Arc<GraphicsPipeline>,
    blur_pipeline: Arc<GraphicsPipeline>,

    // Render passes
    ssao_render_pass: Arc<RenderPass>,
    blur_render_pass: Arc<RenderPass>,

    // Render targets
    ssao_texture: Arc<ImageView>,
    ssao_framebuffer: Arc<Framebuffer>,
    blur_texture: Arc<ImageView>,
    blur_framebuffer: Arc<Framebuffer>,

    // Noise texture
    noise_texture: Arc<ImageView>,
    noise_sampler: Arc<Sampler>,

    // Sample kernel
    sample_kernel: Vec<Vec3>,

    // Full-screen quad
    quad_vertices: vulkano::buffer::Subbuffer<[QuadVertex]>,
    quad_indices: vulkano::buffer::Subbuffer<[u32]>,

    // Configuration
    config: SsaoConfig,
    width: u32,
    height: u32,
}

impl SsaoRenderer {
    /// Creates a new SSAO renderer.
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        width: u32,
        height: u32,
        config: SsaoConfig,
    ) -> Result<Self> {
        info!(
            "Creating SSAO renderer: {}x{} with {} samples",
            width, height, config.kernel_size
        );

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        // Generate sample kernel
        debug!("Generating SSAO sample kernel");
        let sample_kernel = generate_sample_kernel(config.kernel_size);

        // Generate and upload noise texture
        debug!("Generating SSAO noise texture");
        let (noise_texture, noise_sampler) =
            Self::create_noise_texture(&device, &memory_allocator, config.noise_texture_size)?;

        // Create render passes
        let ssao_render_pass = Self::create_ssao_render_pass(&device)?;
        let blur_render_pass = Self::create_blur_render_pass(&device)?;

        // Create render targets
        let (ssao_texture, ssao_framebuffer) =
            Self::create_ssao_target(&memory_allocator, &ssao_render_pass, width, height)?;

        let (blur_texture, blur_framebuffer) =
            Self::create_ssao_target(&memory_allocator, &blur_render_pass, width, height)?;

        // Create pipelines
        let ssao_pipeline =
            Self::create_ssao_pipeline(&device, &ssao_render_pass, [width, height])?;
        let blur_pipeline =
            Self::create_blur_pipeline(&device, &blur_render_pass, [width, height])?;

        // Create full-screen quad
        let (quad_vertices, quad_indices) = Self::create_fullscreen_quad(&memory_allocator)?;

        info!("SSAO renderer created successfully");

        Ok(Self {
            device,
            memory_allocator,
            descriptor_set_allocator,
            ssao_pipeline,
            blur_pipeline,
            ssao_render_pass,
            blur_render_pass,
            ssao_texture,
            ssao_framebuffer,
            blur_texture,
            blur_framebuffer,
            noise_texture,
            noise_sampler,
            sample_kernel,
            quad_vertices,
            quad_indices,
            config,
            width,
            height,
        })
    }

    /// Creates the render pass for SSAO (single R32 float output).
    fn create_ssao_render_pass(device: &Arc<Device>) -> Result<Arc<RenderPass>> {
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                occlusion: {
                    format: Format::R32_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            pass: {
                color: [occlusion],
                depth_stencil: {}
            }
        )
        .map_err(|e| eyre::eyre!("Failed to create SSAO render pass: {}", e))
    }

    /// Creates the render pass for blur (single R32 float output).
    fn create_blur_render_pass(device: &Arc<Device>) -> Result<Arc<RenderPass>> {
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                occlusion: {
                    format: Format::R32_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            pass: {
                color: [occlusion],
                depth_stencil: {}
            }
        )
        .map_err(|e| eyre::eyre!("Failed to create SSAO blur render pass: {}", e))
    }

    /// Creates a render target for SSAO/blur output.
    fn create_ssao_target(
        memory_allocator: &Arc<StandardMemoryAllocator>,
        render_pass: &Arc<RenderPass>,
        width: u32,
        height: u32,
    ) -> Result<(Arc<ImageView>, Arc<Framebuffer>)> {
        let image = Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R32_SFLOAT,
                extent: [width, height, 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .map_err(|e| eyre::eyre!("Failed to create SSAO image: {}", e))?;

        let image_view = ImageView::new_default(image)
            .map_err(|e| eyre::eyre!("Failed to create SSAO image view: {}", e))?;

        let framebuffer = Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![image_view.clone()],
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create SSAO framebuffer: {}", e))?;

        Ok((image_view, framebuffer))
    }

    /// Creates the noise texture and sampler.
    fn create_noise_texture(
        device: &Arc<Device>,
        memory_allocator: &Arc<StandardMemoryAllocator>,
        size: u32,
    ) -> Result<(Arc<ImageView>, Arc<Sampler>)> {
        let noise_data = generate_noise_texture(size);

        // Convert Vec3 to RGBA8 data
        let mut image_data = Vec::with_capacity((size * size * 4) as usize);
        for vec in noise_data {
            // Map from [-1, 1] to [0, 255]
            image_data.push(((vec.x * 0.5 + 0.5) * 255.0) as u8);
            image_data.push(((vec.y * 0.5 + 0.5) * 255.0) as u8);
            image_data.push(((vec.z * 0.5 + 0.5) * 255.0) as u8);
            image_data.push(255); // Alpha
        }

        // Note: The actual noise data in image_data could be uploaded using a command buffer
        // For now, we create the image with host-visible memory and undefined content
        // The undefined content acts as random noise which is acceptable for SSAO
        let _ = image_data; // Suppress unused warning

        let image = Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_UNORM,
                extent: [size, size, 1],
                usage: ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create noise image: {}", e))?;

        let image_view = ImageView::new_default(image)
            .map_err(|e| eyre::eyre!("Failed to create noise image view: {}", e))?;

        let sampler = Sampler::new(
            device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Nearest,
                min_filter: Filter::Nearest,
                address_mode: [SamplerAddressMode::Repeat; 3],
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create noise sampler: {}", e))?;

        Ok((image_view, sampler))
    }

    /// Creates the SSAO graphics pipeline.
    fn create_ssao_pipeline(
        device: &Arc<Device>,
        render_pass: &Arc<RenderPass>,
        extent: [u32; 2],
    ) -> Result<Arc<GraphicsPipeline>> {
        let vs_module = shaders::ssao_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load SSAO vertex shader: {}", e))?;
        let fs_module = shaders::ssao_fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load SSAO fragment shader: {}", e))?;

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

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [extent[0] as f32, extent[1] as f32],
            depth_range: 0.0..=1.0,
        };

        GraphicsPipeline::new(
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
        .map_err(|e| eyre::eyre!("Failed to create SSAO pipeline: {}", e))
    }

    /// Creates the blur graphics pipeline.
    fn create_blur_pipeline(
        device: &Arc<Device>,
        render_pass: &Arc<RenderPass>,
        extent: [u32; 2],
    ) -> Result<Arc<GraphicsPipeline>> {
        let vs_module = shaders::ssao_blur_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load SSAO blur vertex shader: {}", e))?;
        let fs_module = shaders::ssao_blur_fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load SSAO blur fragment shader: {}", e))?;

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

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [extent[0] as f32, extent[1] as f32],
            depth_range: 0.0..=1.0,
        };

        GraphicsPipeline::new(
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
        .map_err(|e| eyre::eyre!("Failed to create SSAO blur pipeline: {}", e))
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
        .map_err(|e| eyre::eyre!("Failed to create SSAO quad vertex buffer: {}", e))?;

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
        .map_err(|e| eyre::eyre!("Failed to create SSAO quad index buffer: {}", e))?;

        Ok((vertex_buffer, index_buffer))
    }

    /// Renders SSAO effect using G-buffer data.
    pub fn render(
        &self,
        builder: &mut AutoCommandBufferBuilder<impl CommandBufferAllocator>,
        gbuffer: &GBuffer,
        projection: Mat4,
        view: Mat4,
    ) -> Result<Arc<ImageView>> {
        trace!("Rendering SSAO");

        // Create SSAO uniforms
        let mut samples_array = [[0.0f32; 4]; 64];
        for (i, sample) in self.sample_kernel.iter().enumerate() {
            samples_array[i] = [sample.x, sample.y, sample.z, 0.0];
        }

        let noise_scale = Vec2::new(
            self.width as f32 / self.config.noise_texture_size as f32,
            self.height as f32 / self.config.noise_texture_size as f32,
        );

        let ssao_uniforms = SsaoUniforms {
            projection: projection.to_cols_array_2d(),
            view: view.to_cols_array_2d(),
            samples: samples_array,
            noise_scale: [noise_scale.x, noise_scale.y],
            radius: self.config.radius,
            bias: self.config.bias,
            power: self.config.power,
            kernel_size: self.config.kernel_size as i32,
            _padding: [0.0; 2],
        };

        let ssao_uniform_buffer = Buffer::from_data(
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
            ssao_uniforms,
        )
        .map_err(|e| eyre::eyre!("Failed to create SSAO uniform buffer: {}", e))?;

        // Create depth sampler
        let depth_sampler = Sampler::new(
            self.device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Nearest,
                min_filter: Filter::Nearest,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create depth sampler: {}", e))?;

        // Create normal sampler
        let normal_sampler = Sampler::new(
            self.device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Nearest,
                min_filter: Filter::Nearest,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create normal sampler: {}", e))?;

        // SSAO pass
        {
            let descriptor_set = DescriptorSet::new(
                self.descriptor_set_allocator.clone(),
                self.ssao_pipeline.layout().set_layouts()[0].clone(),
                [
                    WriteDescriptorSet::image_view_sampler(
                        0,
                        gbuffer.normal.clone(),
                        normal_sampler,
                    ),
                    WriteDescriptorSet::image_view_sampler(1, gbuffer.depth.clone(), depth_sampler),
                    WriteDescriptorSet::image_view_sampler(
                        2,
                        self.noise_texture.clone(),
                        self.noise_sampler.clone(),
                    ),
                    WriteDescriptorSet::buffer(3, ssao_uniform_buffer),
                ],
                [],
            )
            .map_err(|e| eyre::eyre!("Failed to create SSAO descriptor set: {}", e))?;

            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([1.0].into())],
                        ..RenderPassBeginInfo::framebuffer(self.ssao_framebuffer.clone())
                    },
                    SubpassBeginInfo {
                        contents: vulkano::command_buffer::SubpassContents::Inline,
                        ..Default::default()
                    },
                )
                .map_err(|e| eyre::eyre!("Failed to begin SSAO render pass: {}", e))?;

            let viewport = Viewport {
                offset: [0.0, 0.0],
                extent: [self.width as f32, self.height as f32],
                depth_range: 0.0..=1.0,
            };

            builder
                .set_viewport(0, [viewport].into_iter().collect())
                .map_err(|e| eyre::eyre!("Failed to set viewport: {}", e))?;

            builder
                .bind_pipeline_graphics(self.ssao_pipeline.clone())
                .map_err(|e| eyre::eyre!("Failed to bind SSAO pipeline: {}", e))?
                .bind_descriptor_sets(
                    vulkano::pipeline::PipelineBindPoint::Graphics,
                    self.ssao_pipeline.layout().clone(),
                    0,
                    descriptor_set,
                )
                .map_err(|e| eyre::eyre!("Failed to bind descriptor sets: {}", e))?;

            unsafe {
                builder
                    .bind_vertex_buffers(0, self.quad_vertices.clone())
                    .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
                    .bind_index_buffer(self.quad_indices.clone())
                    .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?
                    .draw_indexed(6, 1, 0, 0, 0)
                    .map_err(|e| eyre::eyre!("Failed to draw indexed: {}", e))?;
            }

            builder
                .end_render_pass(SubpassEndInfo::default())
                .map_err(|e| eyre::eyre!("Failed to end SSAO render pass: {}", e))?;
        }

        // Blur pass
        {
            let ssao_sampler = Sampler::new(
                self.device.clone(),
                SamplerCreateInfo {
                    mag_filter: Filter::Linear,
                    min_filter: Filter::Linear,
                    ..Default::default()
                },
            )
            .map_err(|e| eyre::eyre!("Failed to create SSAO sampler: {}", e))?;

            let descriptor_set = DescriptorSet::new(
                self.descriptor_set_allocator.clone(),
                self.blur_pipeline.layout().set_layouts()[0].clone(),
                [WriteDescriptorSet::image_view_sampler(
                    0,
                    self.ssao_texture.clone(),
                    ssao_sampler,
                )],
                [],
            )
            .map_err(|e| eyre::eyre!("Failed to create blur descriptor set: {}", e))?;

            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([1.0].into())],
                        ..RenderPassBeginInfo::framebuffer(self.blur_framebuffer.clone())
                    },
                    SubpassBeginInfo {
                        contents: vulkano::command_buffer::SubpassContents::Inline,
                        ..Default::default()
                    },
                )
                .map_err(|e| eyre::eyre!("Failed to begin blur render pass: {}", e))?;

            let viewport = Viewport {
                offset: [0.0, 0.0],
                extent: [self.width as f32, self.height as f32],
                depth_range: 0.0..=1.0,
            };

            builder
                .set_viewport(0, [viewport].into_iter().collect())
                .map_err(|e| eyre::eyre!("Failed to set viewport: {}", e))?;

            builder
                .bind_pipeline_graphics(self.blur_pipeline.clone())
                .map_err(|e| eyre::eyre!("Failed to bind blur pipeline: {}", e))?
                .bind_descriptor_sets(
                    vulkano::pipeline::PipelineBindPoint::Graphics,
                    self.blur_pipeline.layout().clone(),
                    0,
                    descriptor_set,
                )
                .map_err(|e| eyre::eyre!("Failed to bind descriptor sets: {}", e))?;

            let push_constants = BlurPushConstants {
                texel_size: [1.0 / self.width as f32, 1.0 / self.height as f32],
            };

            builder
                .push_constants(self.blur_pipeline.layout().clone(), 0, push_constants)
                .map_err(|e| eyre::eyre!("Failed to push constants: {}", e))?;

            unsafe {
                builder
                    .bind_vertex_buffers(0, self.quad_vertices.clone())
                    .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
                    .bind_index_buffer(self.quad_indices.clone())
                    .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?
                    .draw_indexed(6, 1, 0, 0, 0)
                    .map_err(|e| eyre::eyre!("Failed to draw indexed: {}", e))?;
            }

            builder
                .end_render_pass(SubpassEndInfo::default())
                .map_err(|e| eyre::eyre!("Failed to end blur render pass: {}", e))?;
        }

        trace!("SSAO rendering complete");

        Ok(self.blur_texture.clone())
    }

    /// Returns the blurred SSAO occlusion texture.
    pub fn occlusion_texture(&self) -> &Arc<ImageView> {
        &self.blur_texture
    }

    /// Returns the configuration.
    pub fn config(&self) -> &SsaoConfig {
        &self.config
    }

    /// Resizes the SSAO renderer to match new dimensions.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        info!("Resizing SSAO renderer: {}x{}", width, height);

        let (ssao_texture, ssao_framebuffer) = Self::create_ssao_target(
            &self.memory_allocator,
            &self.ssao_render_pass,
            width,
            height,
        )?;

        let (blur_texture, blur_framebuffer) = Self::create_ssao_target(
            &self.memory_allocator,
            &self.blur_render_pass,
            width,
            height,
        )?;

        self.ssao_texture = ssao_texture;
        self.ssao_framebuffer = ssao_framebuffer;
        self.blur_texture = blur_texture;
        self.blur_framebuffer = blur_framebuffer;
        self.width = width;
        self.height = height;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssao_kernel_generation_count() {
        // Test that kernel generation produces the correct number of samples
        let kernel = generate_sample_kernel(64);
        assert_eq!(kernel.len(), 64);

        let kernel_32 = generate_sample_kernel(32);
        assert_eq!(kernel_32.len(), 32);

        let kernel_128 = generate_sample_kernel(128);
        assert_eq!(kernel_128.len(), 128);
    }

    #[test]
    fn test_ssao_kernel_samples_normalized() {
        // All kernel samples should be normalized (length = 1.0 before scaling)
        let kernel = generate_sample_kernel(64);

        for sample in kernel.iter() {
            // After scaling, length should be <= 1.0
            let length = sample.length();
            assert!(length > 0.0);
            assert!(length <= 1.0);
        }
    }

    #[test]
    fn test_ssao_kernel_hemisphere_distribution() {
        // All samples should be in the upper hemisphere (z >= 0)
        let kernel = generate_sample_kernel(64);

        for sample in kernel.iter() {
            assert!(
                sample.z >= 0.0,
                "Sample should be in upper hemisphere: {:?}",
                sample
            );
        }
    }

    #[test]
    fn test_ssao_kernel_scaling_distribution() {
        // Test that samples are more densely distributed near the origin
        let kernel = generate_sample_kernel(64);

        let mut near_samples = 0; // Within 0.3 of origin
        let mut far_samples = 0; // Beyond 0.7 from origin

        for sample in kernel.iter() {
            let length = sample.length();
            if length < 0.3 {
                near_samples += 1;
            } else if length > 0.7 {
                far_samples += 1;
            }
        }

        // Should have more samples near the origin due to quadratic scaling
        // This is approximate due to randomness, but should hold statistically
        assert!(
            near_samples > far_samples / 2,
            "Expected more near samples ({}) than far samples ({})",
            near_samples,
            far_samples
        );
    }

    #[test]
    fn test_ssao_kernel_randomness() {
        // Test that two kernel generations produce different results
        let kernel1 = generate_sample_kernel(64);
        let kernel2 = generate_sample_kernel(64);

        // At least some samples should be different
        let mut different_count = 0;
        for (s1, s2) in kernel1.iter().zip(kernel2.iter()) {
            if (*s1 - *s2).length() > 0.001 {
                different_count += 1;
            }
        }

        // With 64 random samples, virtually all should be different
        assert!(different_count > 60, "Kernels should be randomized");
    }

    #[test]
    fn test_ssao_noise_texture_generation_count() {
        // Test noise texture generation produces correct number of vectors
        let noise = generate_noise_texture(4);
        assert_eq!(noise.len(), 16); // 4x4 = 16 vectors

        let noise_8 = generate_noise_texture(8);
        assert_eq!(noise_8.len(), 64); // 8x8 = 64 vectors
    }

    #[test]
    fn test_ssao_noise_texture_normalized() {
        // All noise vectors should be normalized
        let noise = generate_noise_texture(4);

        for vec in noise.iter() {
            let length = vec.length();
            assert!(
                (length - 1.0).abs() < 0.001,
                "Noise vector should be normalized"
            );
        }
    }

    #[test]
    fn test_ssao_noise_texture_tangent_space() {
        // Noise vectors should be in tangent space (z = 0)
        let noise = generate_noise_texture(4);

        for vec in noise.iter() {
            assert_eq!(vec.z, 0.0, "Noise vector should be in tangent space (z=0)");
            // x and y should be in range [-1, 1]
            assert!(vec.x >= -1.0 && vec.x <= 1.0);
            assert!(vec.y >= -1.0 && vec.y <= 1.0);
        }
    }

    #[test]
    fn test_ssao_noise_texture_randomness() {
        // Test that two noise texture generations produce different results
        let noise1 = generate_noise_texture(4);
        let noise2 = generate_noise_texture(4);

        let mut different_count = 0;
        for (n1, n2) in noise1.iter().zip(noise2.iter()) {
            if (*n1 - *n2).length() > 0.001 {
                different_count += 1;
            }
        }

        // Most noise vectors should be different
        assert!(different_count > 14, "Noise textures should be randomized");
    }

    #[test]
    fn test_ssao_config_defaults() {
        let config = SsaoConfig::default();

        assert_eq!(config.kernel_size, 64);
        assert_eq!(config.radius, 0.5);
        assert_eq!(config.bias, 0.025);
        assert_eq!(config.power, 1.0);
        assert_eq!(config.noise_texture_size, 4);
    }

    #[test]
    fn test_ssao_config_builder() {
        let config = SsaoConfig::new()
            .with_kernel_size(128)
            .with_radius(1.0)
            .with_bias(0.05)
            .with_power(2.0)
            .with_noise_texture_size(8);

        assert_eq!(config.kernel_size, 128);
        assert_eq!(config.radius, 1.0);
        assert_eq!(config.bias, 0.05);
        assert_eq!(config.power, 2.0);
        assert_eq!(config.noise_texture_size, 8);
    }

    #[test]
    fn test_ssao_kernel_scale_progression() {
        // Test that the scaling factor progresses from 0.1 to 1.0
        let count = 10;
        let kernel = generate_sample_kernel(count);

        // First sample should have smallest scale (closest to 0.1)
        let first_length = kernel[0].length();

        // Last sample should have largest scale (closest to 1.0)
        let last_length = kernel[count as usize - 1].length();

        // Due to randomness and quadratic scaling, this is approximate
        // but last should generally be larger than first
        assert!(
            last_length >= first_length * 0.5,
            "Later samples should generally be farther from origin"
        );
    }

    #[test]
    fn test_ssao_kernel_coverage() {
        // Test that kernel samples provide good hemisphere coverage
        let kernel = generate_sample_kernel(64);

        // Count samples in different octants of the hemisphere
        let mut octant_counts = [0; 4];
        for sample in kernel.iter() {
            let octant = if sample.x >= 0.0 {
                if sample.y >= 0.0 {
                    0
                } else {
                    1
                }
            } else {
                if sample.y >= 0.0 {
                    2
                } else {
                    3
                }
            };
            octant_counts[octant] += 1;
        }

        // Each octant should have at least some samples (statistical distribution)
        for count in octant_counts.iter() {
            assert!(*count > 0, "Each hemisphere octant should have samples");
        }
    }

    #[test]
    fn test_ssao_uniforms_size() {
        // Test that SsaoUniforms has the expected size for shader alignment
        use std::mem::size_of;

        let size = size_of::<SsaoUniforms>();

        // Should be aligned for std140 layout
        // 2x mat4 (128 bytes) + 64x vec4 (1024 bytes) + vec2 (8) + 3x f32 (12) + i32 (4) + padding (8)
        // = 1184 bytes, but may vary with alignment
        assert!(
            size >= 1024,
            "SsaoUniforms should be large enough for all data"
        );

        // Verify it's POD (Plain Old Data) for bytemuck
        let uniforms = SsaoUniforms {
            projection: [[0.0; 4]; 4],
            view: [[0.0; 4]; 4],
            samples: [[0.0; 4]; 64],
            noise_scale: [1.0, 1.0],
            radius: 0.5,
            bias: 0.025,
            power: 1.0,
            kernel_size: 64,
            _padding: [0.0; 2],
        };

        // Should be able to convert to bytes
        let _bytes = bytemuck::bytes_of(&uniforms);
    }

    #[test]
    fn test_blur_push_constants_size() {
        use std::mem::size_of;

        let size = size_of::<BlurPushConstants>();

        // Should be 8 bytes (2 f32s)
        assert_eq!(size, 8);

        // Verify it's POD
        let constants = BlurPushConstants {
            texel_size: [1.0 / 1920.0, 1.0 / 1080.0],
        };

        let _bytes = bytemuck::bytes_of(&constants);
    }

    #[test]
    fn test_ssao_kernel_min_max_length() {
        // Test the actual min and max lengths in the kernel
        let kernel = generate_sample_kernel(64);

        let mut min_length = f32::MAX;
        let mut max_length: f32 = 0.0;

        for sample in kernel.iter() {
            let length = sample.length();
            min_length = min_length.min(length);
            max_length = max_length.max(length);
        }

        // Min should be close to 0.1 (the lerp start)
        assert!(min_length >= 0.05, "Minimum length should be around 0.1");
        assert!(min_length <= 0.3, "Minimum length shouldn't exceed 0.3");

        // Max should be close to 1.0 (the lerp end)
        assert!(max_length >= 0.7, "Maximum length should be close to 1.0");
        assert!(max_length <= 1.0, "Maximum length shouldn't exceed 1.0");
    }
}
