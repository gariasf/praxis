//! Screen-Space Reflections (SSR) with hierarchical ray marching and environment probe fallback.
//!
//! This module provides a complete SSR system with:
//! - Hierarchical ray marching for fast screen-space ray tracing
//! - Roughness-aware blur for physically-based reflections
//! - Fallback to environment probes for rays that miss screen-space geometry
//! - Integration with deferred rendering G-buffer
//!
//! # Educational: Screen-Space Ray Marching
//!
//! ## The Reflection Problem
//!
//! Reflections are expensive to compute accurately:
//! - **Ray tracing**: Trace rays through 3D scene → Accurate but slow
//! - **Cubemaps**: Pre-render environment → Fast but static
//! - **Planar reflections**: Render scene mirrored → Limited to flat surfaces
//!
//! ## Screen-Space Reflections (SSR)
//!
//! SSR uses a clever trick: "Ray march through the depth buffer"
//!
//! ### Key Insight
//!
//! We already have depth information for all visible pixels. By marching along
//! a reflection ray and checking depth, we can find intersection points!
//!
//! ```text
//! Side view:
//!
//! Ray
//!  ╱
//! ╱   Surface
//! ───────┐    ← Depth buffer tells us surface is here
//!        │
//!        └─── Wall
//!
//! Top-down (screen space):
//!
//! [Pixel]─────→[Hit]
//!  Start         Reflection found!
//! ```
//!
//! ## Ray Marching Algorithm
//!
//! ### Step 1: Setup
//! ```text
//! 1. Read G-buffer data for current pixel:
//!    - World position (reconstructed from depth)
//!    - Surface normal
//!    - Roughness
//!
//! 2. Calculate reflection ray:
//!    view_dir = normalize(camera_pos - world_pos)
//!    reflect_dir = reflect(-view_dir, normal)
//! ```
//!
//! ### Step 2: Ray Marching (The Core Algorithm)
//!
//! ```text
//! Initialize:
//!   ray_pos = world_pos           // Start at surface
//!   ray_dir = reflect_dir          // Reflection direction
//!   step_size = initial_step       // How far to march each step
//!
//! For each step (up to max_steps):
//!   1. March forward:
//!      ray_pos += ray_dir * step_size
//!
//!   2. Project to screen space:
//!      screen_pos = project(ray_pos)  // World → Screen UV
//!
//!   3. Read depth at this screen position:
//!      scene_depth = depth_buffer[screen_pos]
//!
//!   4. Compare depths:
//!      ray_depth = depth_of(ray_pos)
//!      
//!      if ray_depth > scene_depth:  // Ray is behind surface
//!         → Intersection found!
//!         return screen_pos
//! ```
//!
//! ### Step 3: Binary Search Refinement
//!
//! Problem: Step size might skip over thin geometry or be inaccurate.
//! Solution: Binary search for exact hit point.
//!
//! ```text
//! Once we overshoot:
//!   lo = previous_pos    // Before hit
//!   hi = current_pos     // After hit
//!
//!   For 8-16 iterations:
//!     mid = (lo + hi) / 2
//!     if mid depth > scene depth:
//!       hi = mid  // Overshoot, search earlier
//!     else:
//!       lo = mid  // Undershoot, search later
//!
//!   return mid  // Sub-pixel accurate hit!
//! ```
//!
//! ## Hierarchical Ray Marching (Optimization)
//!
//! ### Problem
//! Fixed step size is inefficient:
//! - Too large: Miss small details
//! - Too small: Too many steps, slow
//!
//! ### Solution: Adaptive Step Size
//!
//! Use a mipmap chain of the depth buffer:
//! ```text
//! Mip 0: 1920×1080 (full resolution)
//! Mip 1: 960×540   (half resolution)
//! Mip 2: 480×270   (quarter resolution)
//! Mip 3: 240×135   (eighth resolution)
//! ```
//!
//! Algorithm:
//! ```text
//! Start at coarse mip level (mip 3):
//!   - Large steps through scene
//!   - Fast, covers long distances
//!
//! When approaching surface:
//!   - Drop to finer mip (mip 2, then 1, then 0)
//!   - Smaller steps for accuracy
//!
//! When very close:
//!   - Binary search at mip 0 for sub-pixel precision
//! ```
//!
//! **Result**: 2-3× faster than fixed step size with same quality!
//!
//! ## Handling Edge Cases
//!
//! ### Problem 1: Ray Leaves Screen
//! ```text
//! [Screen]
//!  │
//!  │  Ray ──→ (goes off-screen)
//!  │
//! ```
//! Solution: Fade out reflection strength near edges.
//!
//! ### Problem 2: No Hit Found
//! ```text
//! Ray travels max_steps without hitting anything.
//! Maybe reflecting sky or off-screen object.
//! ```
//! Solution: Fall back to environment probe cubemap.
//!
//! ### Problem 3: Self-Intersection
//! ```text
//! Ray immediately hits its own surface.
//! ```
//! Solution: Start ray slightly offset from surface (thickness parameter).
//!
//! ## Roughness-Aware Blur
//!
//! Smooth surfaces (roughness=0.0) → Sharp reflections
//! Rough surfaces (roughness=1.0) → Blurry reflections
//!
//! ```text
//! Blur amount = roughness * max_blur_radius
//!
//! Smooth mirror (roughness=0.1):  blur_radius = 1 pixel
//! Rough metal (roughness=0.5):    blur_radius = 8 pixels
//! Very rough (roughness=1.0):     blur_radius = 16 pixels
//! ```
//!
//! Implementation: Separable Gaussian blur (horizontal then vertical passes).
//!
//! ## Environment Probe Fallback
//!
//! When SSR fails (no hit, off-screen, low confidence):
//! ```text
//! 1. Calculate reflection direction in world space
//! 2. Sample environment cubemap in that direction
//! 3. Blend based on SSR confidence:
//!    
//!    final_color = mix(environment_color, ssr_color, ssr_confidence)
//! ```
//!
//! This ensures we always have *some* reflection, never black holes.
//!
//! ## Performance Characteristics
//!
//! ### Costs (1080p)
//! - Ray marching: 2-4ms (depends on max_steps)
//! - Blur passes: 1-2ms (depends on roughness)
//! - Composite: 0.5ms
//!
//! **Total**: 3.5-6.5ms per frame
//!
//! ### Quality Factors
//! - max_steps: More steps = better accuracy, slower
//! - step_size: Smaller steps = fewer misses, slower
//! - binary_search_steps: More steps = sub-pixel accuracy, minimal cost
//!
//! ### Typical Settings
//! - max_steps: 32-64 (balanced)
//! - binary_search_steps: 8-16 (cheap, huge quality gain)
//! - thickness: 0.05-0.2 (scene-dependent)
//!
//! ## Limitations
//!
//! 1. **Screen-space only**: Can't reflect off-screen objects
//! 2. **Backface issues**: Can't see back of objects
//! 3. **Thin geometry**: Small objects might be missed
//! 4. **Cost**: Expensive for rough surfaces (lots of blur)
//!
//! These are mitigated by environment probe fallback and smart parameter tuning.
//!
//! # SSR Overview
//!
//! SSR is a screen-space technique that generates reflections by ray marching through
//! the depth buffer. It's efficient because it only considers visible geometry, but
//! has limitations (can't reflect off-screen objects). We mitigate this with environment
//! probe fallback.
//!
//! ## Algorithm Steps
//!
//! 1. **Ray Marching Pass**: For each pixel:
//!    - Reconstruct view-space position from depth
//!    - Calculate reflection ray from view direction and normal
//!    - March through depth buffer using hierarchical ray marching
//!    - If hit, store hit UV and confidence; otherwise mark for probe fallback
//! 2. **Roughness Blur Pass**: Apply variable-strength blur based on surface roughness
//! 3. **Environment Probe Fallback**: For pixels with no screen-space hit, sample environment probe
//!
//! # Hierarchical Ray Marching
//!
//! Uses a mipmap pyramid of depth buffer for efficient ray tracing:
//! - Coarse steps at higher mip levels (fewer samples, larger steps)
//! - Fine steps at lower mip levels (more samples, smaller steps)
//! - Adaptive refinement when approaching intersection
//!
//! # Usage
//!
//! ```rust,no_run
//! use praxis_graphics::ssr::{SsrRenderer, SsrConfig};
//! # use std::sync::Arc;
//! # use vulkano::device::Device;
//! # use vulkano::memory::allocator::StandardMemoryAllocator;
//! # fn example(
//! #     device: Arc<Device>,
//! #     memory_allocator: Arc<StandardMemoryAllocator>,
//! # ) -> praxis_utils::Result<()> {
//!
//! let config = SsrConfig::default()
//!     .with_max_steps(64)
//!     .with_max_binary_search_steps(8)
//!     .with_thickness(0.1)
//!     .with_max_roughness(0.8);
//!
//! let mut ssr = SsrRenderer::new(
//!     device,
//!     memory_allocator,
//!     1920,
//!     1080,
//!     config,
//! )?;
//!
//! // In render loop, after G-buffer pass:
//! // let ssr_texture = ssr.render(builder, gbuffer, view, proj, environment_probe)?;
//! # Ok(())
//! # }
//! ```

use crate::{deferred::GBuffer, environment_probe::IblData, post_process::QuadVertex, shaders};
use praxis_math::{Mat4, Vec3};
use praxis_utils::{eyre, info, trace, Result};
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
        sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode},
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

/// Configuration for SSR effect.
#[derive(Debug, Clone)]
pub struct SsrConfig {
    /// Maximum number of ray marching steps (default: 64)
    pub max_steps: u32,
    /// Maximum number of binary search refinement steps (default: 8)
    pub max_binary_search_steps: u32,
    /// Ray step size multiplier (default: 1.0)
    pub step_size: f32,
    /// Surface thickness for intersection testing (default: 0.1)
    pub thickness: f32,
    /// Maximum roughness for reflections (default: 0.8, higher = blurrier)
    pub max_roughness: f32,
    /// Minimum screen-space hit confidence to use SSR (default: 0.5)
    pub min_hit_confidence: f32,
    /// Fade out reflections near screen edges (default: 0.1)
    pub edge_fade_factor: f32,
    /// Number of blur passes for roughness-aware blur (default: 2)
    pub blur_passes: u32,
}

impl Default for SsrConfig {
    fn default() -> Self {
        Self {
            max_steps: 64,
            max_binary_search_steps: 8,
            step_size: 1.0,
            thickness: 0.1,
            max_roughness: 0.8,
            min_hit_confidence: 0.5,
            edge_fade_factor: 0.1,
            blur_passes: 2,
        }
    }
}

impl SsrConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_steps(mut self, max_steps: u32) -> Self {
        self.max_steps = max_steps;
        self
    }

    pub fn with_max_binary_search_steps(mut self, max_binary_search_steps: u32) -> Self {
        self.max_binary_search_steps = max_binary_search_steps;
        self
    }

    pub fn with_step_size(mut self, step_size: f32) -> Self {
        self.step_size = step_size;
        self
    }

    pub fn with_thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    pub fn with_max_roughness(mut self, max_roughness: f32) -> Self {
        self.max_roughness = max_roughness;
        self
    }

    pub fn with_min_hit_confidence(mut self, min_hit_confidence: f32) -> Self {
        self.min_hit_confidence = min_hit_confidence;
        self
    }

    pub fn with_edge_fade_factor(mut self, edge_fade_factor: f32) -> Self {
        self.edge_fade_factor = edge_fade_factor;
        self
    }

    pub fn with_blur_passes(mut self, blur_passes: u32) -> Self {
        self.blur_passes = blur_passes;
        self
    }
}

/// SSR uniform data matching shader layout.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SsrUniforms {
    projection: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    inv_projection: [[f32; 4]; 4],
    inv_view: [[f32; 4]; 4],
    camera_position: [f32; 3],
    max_steps: u32,
    max_binary_search_steps: u32,
    step_size: f32,
    thickness: f32,
    max_roughness: f32,
    min_hit_confidence: f32,
    edge_fade_factor: f32,
    _padding: [f32; 2],
}

/// Push constants for blur pass.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurPushConstants {
    texel_size: [f32; 2],
    blur_direction: [f32; 2],
}

/// SSR renderer managing render targets and pipelines.
pub struct SsrRenderer {
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

    // Pipelines
    ssr_pipeline: Arc<GraphicsPipeline>,
    blur_pipeline: Arc<GraphicsPipeline>,
    composite_pipeline: Arc<GraphicsPipeline>,

    // Render passes
    ssr_render_pass: Arc<RenderPass>,
    blur_render_pass: Arc<RenderPass>,
    composite_render_pass: Arc<RenderPass>,

    // Render targets
    ssr_texture: Arc<ImageView>,
    ssr_framebuffer: Arc<Framebuffer>,
    blur_texture_a: Arc<ImageView>,
    blur_framebuffer_a: Arc<Framebuffer>,
    blur_texture_b: Arc<ImageView>,
    blur_framebuffer_b: Arc<Framebuffer>,
    composite_texture: Arc<ImageView>,
    composite_framebuffer: Arc<Framebuffer>,

    // Full-screen quad
    quad_vertices: vulkano::buffer::Subbuffer<[QuadVertex]>,
    quad_indices: vulkano::buffer::Subbuffer<[u32]>,

    // Configuration
    config: SsrConfig,
    width: u32,
    height: u32,
}

impl SsrRenderer {
    /// Creates a new SSR renderer.
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        width: u32,
        height: u32,
        config: SsrConfig,
    ) -> Result<Self> {
        info!(
            "Creating SSR renderer: {}x{} with {} max steps",
            width, height, config.max_steps
        );

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        // Create render passes
        let ssr_render_pass = Self::create_ssr_render_pass(&device)?;
        let blur_render_pass = Self::create_blur_render_pass(&device)?;
        let composite_render_pass = Self::create_composite_render_pass(&device)?;

        // Create render targets
        let (ssr_texture, ssr_framebuffer) =
            Self::create_ssr_target(&memory_allocator, &ssr_render_pass, width, height)?;

        let (blur_texture_a, blur_framebuffer_a) =
            Self::create_blur_target(&memory_allocator, &blur_render_pass, width, height)?;

        let (blur_texture_b, blur_framebuffer_b) =
            Self::create_blur_target(&memory_allocator, &blur_render_pass, width, height)?;

        let (composite_texture, composite_framebuffer) = Self::create_composite_target(
            &memory_allocator,
            &composite_render_pass,
            width,
            height,
        )?;

        // Create pipelines
        let ssr_pipeline = Self::create_ssr_pipeline(&device, &ssr_render_pass, [width, height])?;
        let blur_pipeline =
            Self::create_blur_pipeline(&device, &blur_render_pass, [width, height])?;
        let composite_pipeline =
            Self::create_composite_pipeline(&device, &composite_render_pass, [width, height])?;

        // Create full-screen quad
        let (quad_vertices, quad_indices) = Self::create_fullscreen_quad(&memory_allocator)?;

        info!("SSR renderer created successfully");

        Ok(Self {
            device,
            memory_allocator,
            descriptor_set_allocator,
            ssr_pipeline,
            blur_pipeline,
            composite_pipeline,
            ssr_render_pass,
            blur_render_pass,
            composite_render_pass,
            ssr_texture,
            ssr_framebuffer,
            blur_texture_a,
            blur_framebuffer_a,
            blur_texture_b,
            blur_framebuffer_b,
            composite_texture,
            composite_framebuffer,
            quad_vertices,
            quad_indices,
            config,
            width,
            height,
        })
    }

    /// Creates the render pass for SSR (RGBA16F for reflections + confidence).
    fn create_ssr_render_pass(device: &Arc<Device>) -> Result<Arc<RenderPass>> {
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                reflection: {
                    format: Format::R16G16B16A16_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            pass: {
                color: [reflection],
                depth_stencil: {}
            }
        )
        .map_err(|e| eyre::eyre!("Failed to create SSR render pass: {}", e))
    }

    /// Creates the render pass for blur (RGBA16F).
    fn create_blur_render_pass(device: &Arc<Device>) -> Result<Arc<RenderPass>> {
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                blurred: {
                    format: Format::R16G16B16A16_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            pass: {
                color: [blurred],
                depth_stencil: {}
            }
        )
        .map_err(|e| eyre::eyre!("Failed to create SSR blur render pass: {}", e))
    }

    /// Creates the render pass for composite (RGBA16F).
    fn create_composite_render_pass(device: &Arc<Device>) -> Result<Arc<RenderPass>> {
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                composite: {
                    format: Format::R16G16B16A16_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            pass: {
                color: [composite],
                depth_stencil: {}
            }
        )
        .map_err(|e| eyre::eyre!("Failed to create SSR composite render pass: {}", e))
    }

    /// Creates a render target for SSR output.
    fn create_ssr_target(
        memory_allocator: &Arc<StandardMemoryAllocator>,
        render_pass: &Arc<RenderPass>,
        width: u32,
        height: u32,
    ) -> Result<(Arc<ImageView>, Arc<Framebuffer>)> {
        let image = Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R16G16B16A16_SFLOAT,
                extent: [width, height, 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .map_err(|e| eyre::eyre!("Failed to create SSR image: {}", e))?;

        let image_view = ImageView::new_default(image)
            .map_err(|e| eyre::eyre!("Failed to create SSR image view: {}", e))?;

        let framebuffer = Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![image_view.clone()],
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create SSR framebuffer: {}", e))?;

        Ok((image_view, framebuffer))
    }

    /// Creates a render target for blur output.
    fn create_blur_target(
        memory_allocator: &Arc<StandardMemoryAllocator>,
        render_pass: &Arc<RenderPass>,
        width: u32,
        height: u32,
    ) -> Result<(Arc<ImageView>, Arc<Framebuffer>)> {
        let image = Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R16G16B16A16_SFLOAT,
                extent: [width, height, 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .map_err(|e| eyre::eyre!("Failed to create SSR blur image: {}", e))?;

        let image_view = ImageView::new_default(image)
            .map_err(|e| eyre::eyre!("Failed to create SSR blur image view: {}", e))?;

        let framebuffer = Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![image_view.clone()],
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create SSR blur framebuffer: {}", e))?;

        Ok((image_view, framebuffer))
    }

    /// Creates a render target for composite output.
    fn create_composite_target(
        memory_allocator: &Arc<StandardMemoryAllocator>,
        render_pass: &Arc<RenderPass>,
        width: u32,
        height: u32,
    ) -> Result<(Arc<ImageView>, Arc<Framebuffer>)> {
        let image = Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R16G16B16A16_SFLOAT,
                extent: [width, height, 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .map_err(|e| eyre::eyre!("Failed to create SSR composite image: {}", e))?;

        let image_view = ImageView::new_default(image)
            .map_err(|e| eyre::eyre!("Failed to create SSR composite image view: {}", e))?;

        let framebuffer = Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![image_view.clone()],
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create SSR composite framebuffer: {}", e))?;

        Ok((image_view, framebuffer))
    }

    /// Creates the SSR ray marching graphics pipeline.
    fn create_ssr_pipeline(
        device: &Arc<Device>,
        render_pass: &Arc<RenderPass>,
        extent: [u32; 2],
    ) -> Result<Arc<GraphicsPipeline>> {
        let vs_module = shaders::ssr_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load SSR vertex shader: {}", e))?;
        let fs_module = shaders::ssr_fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load SSR fragment shader: {}", e))?;

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
        .map_err(|e| eyre::eyre!("Failed to create SSR pipeline: {}", e))
    }

    /// Creates the blur graphics pipeline.
    fn create_blur_pipeline(
        device: &Arc<Device>,
        render_pass: &Arc<RenderPass>,
        extent: [u32; 2],
    ) -> Result<Arc<GraphicsPipeline>> {
        let vs_module = shaders::ssr_blur_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load SSR blur vertex shader: {}", e))?;
        let fs_module = shaders::ssr_blur_fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load SSR blur fragment shader: {}", e))?;

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
        .map_err(|e| eyre::eyre!("Failed to create SSR blur pipeline: {}", e))
    }

    /// Creates the composite graphics pipeline.
    fn create_composite_pipeline(
        device: &Arc<Device>,
        render_pass: &Arc<RenderPass>,
        extent: [u32; 2],
    ) -> Result<Arc<GraphicsPipeline>> {
        let vs_module = shaders::ssr_composite_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load SSR composite vertex shader: {}", e))?;
        let fs_module = shaders::ssr_composite_fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load SSR composite fragment shader: {}", e))?;

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
        .map_err(|e| eyre::eyre!("Failed to create SSR composite pipeline: {}", e))
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
        .map_err(|e| eyre::eyre!("Failed to create SSR quad vertex buffer: {}", e))?;

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
        .map_err(|e| eyre::eyre!("Failed to create SSR quad index buffer: {}", e))?;

        Ok((vertex_buffer, index_buffer))
    }

    /// Renders SSR effect using G-buffer data.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        builder: &mut AutoCommandBufferBuilder<impl CommandBufferAllocator>,
        gbuffer: &GBuffer,
        scene_color: Arc<ImageView>,
        projection: Mat4,
        view: Mat4,
        camera_position: Vec3,
        ibl_data: Option<&IblData>,
    ) -> Result<Arc<ImageView>> {
        trace!("Rendering SSR");

        let inv_projection = projection.inverse();
        let inv_view = view.inverse();

        let ssr_uniforms = SsrUniforms {
            projection: projection.to_cols_array_2d(),
            view: view.to_cols_array_2d(),
            inv_projection: inv_projection.to_cols_array_2d(),
            inv_view: inv_view.to_cols_array_2d(),
            camera_position: [camera_position.x, camera_position.y, camera_position.z],
            max_steps: self.config.max_steps,
            max_binary_search_steps: self.config.max_binary_search_steps,
            step_size: self.config.step_size,
            thickness: self.config.thickness,
            max_roughness: self.config.max_roughness,
            min_hit_confidence: self.config.min_hit_confidence,
            edge_fade_factor: self.config.edge_fade_factor,
            _padding: [0.0; 2],
        };

        let ssr_uniform_buffer = Buffer::from_data(
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
            ssr_uniforms,
        )
        .map_err(|e| eyre::eyre!("Failed to create SSR uniform buffer: {}", e))?;

        let sampler = Sampler::new(
            self.device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                mipmap_mode: SamplerMipmapMode::Linear,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create sampler: {}", e))?;

        // SSR ray marching pass
        {
            let descriptor_set = DescriptorSet::new(
                self.descriptor_set_allocator.clone(),
                self.ssr_pipeline.layout().set_layouts()[0].clone(),
                [
                    WriteDescriptorSet::image_view_sampler(
                        0,
                        gbuffer.normal.clone(),
                        sampler.clone(),
                    ),
                    WriteDescriptorSet::image_view_sampler(
                        1,
                        gbuffer.depth.clone(),
                        sampler.clone(),
                    ),
                    WriteDescriptorSet::image_view_sampler(
                        2,
                        gbuffer.metallic_roughness.clone(),
                        sampler.clone(),
                    ),
                    WriteDescriptorSet::image_view_sampler(3, scene_color, sampler.clone()),
                    WriteDescriptorSet::buffer(4, ssr_uniform_buffer),
                ],
                [],
            )
            .map_err(|e| eyre::eyre!("Failed to create SSR descriptor set: {}", e))?;

            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.0, 0.0, 0.0, 0.0].into())],
                        ..RenderPassBeginInfo::framebuffer(self.ssr_framebuffer.clone())
                    },
                    SubpassBeginInfo {
                        contents: vulkano::command_buffer::SubpassContents::Inline,
                        ..Default::default()
                    },
                )
                .map_err(|e| eyre::eyre!("Failed to begin SSR render pass: {}", e))?;

            let viewport = Viewport {
                offset: [0.0, 0.0],
                extent: [self.width as f32, self.height as f32],
                depth_range: 0.0..=1.0,
            };

            builder
                .set_viewport(0, [viewport].into_iter().collect())
                .map_err(|e| eyre::eyre!("Failed to set viewport: {}", e))?;

            builder
                .bind_pipeline_graphics(self.ssr_pipeline.clone())
                .map_err(|e| eyre::eyre!("Failed to bind SSR pipeline: {}", e))?
                .bind_descriptor_sets(
                    vulkano::pipeline::PipelineBindPoint::Graphics,
                    self.ssr_pipeline.layout().clone(),
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
                .map_err(|e| eyre::eyre!("Failed to end SSR render pass: {}", e))?;
        }

        // Roughness-aware blur passes (ping-pong between two textures)
        let mut current_input = self.ssr_texture.clone();
        let mut blur_output = &self.blur_framebuffer_a;
        let mut blur_output_texture = &self.blur_texture_a;

        for pass_idx in 0..self.config.blur_passes {
            let blur_direction = if pass_idx % 2 == 0 {
                [1.0, 0.0] // Horizontal
            } else {
                [0.0, 1.0] // Vertical
            };

            let blur_sampler = Sampler::new(
                self.device.clone(),
                SamplerCreateInfo {
                    mag_filter: Filter::Linear,
                    min_filter: Filter::Linear,
                    ..Default::default()
                },
            )
            .map_err(|e| eyre::eyre!("Failed to create blur sampler: {}", e))?;

            let descriptor_set = DescriptorSet::new(
                self.descriptor_set_allocator.clone(),
                self.blur_pipeline.layout().set_layouts()[0].clone(),
                [
                    WriteDescriptorSet::image_view_sampler(0, current_input, blur_sampler.clone()),
                    WriteDescriptorSet::image_view_sampler(
                        1,
                        gbuffer.metallic_roughness.clone(),
                        blur_sampler,
                    ),
                ],
                [],
            )
            .map_err(|e| eyre::eyre!("Failed to create blur descriptor set: {}", e))?;

            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.0, 0.0, 0.0, 0.0].into())],
                        ..RenderPassBeginInfo::framebuffer(blur_output.clone())
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
                blur_direction,
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

            // Swap buffers
            current_input = blur_output_texture.clone();
            if pass_idx % 2 == 0 {
                blur_output = &self.blur_framebuffer_b;
                blur_output_texture = &self.blur_texture_b;
            } else {
                blur_output = &self.blur_framebuffer_a;
                blur_output_texture = &self.blur_texture_a;
            }
        }

        // Composite pass (blend SSR with environment probe fallback)
        if let Some(ibl) = ibl_data {
            let composite_sampler = Sampler::new(
                self.device.clone(),
                SamplerCreateInfo {
                    mag_filter: Filter::Linear,
                    min_filter: Filter::Linear,
                    ..Default::default()
                },
            )
            .map_err(|e| eyre::eyre!("Failed to create composite sampler: {}", e))?;

            let descriptor_set = DescriptorSet::new(
                self.descriptor_set_allocator.clone(),
                self.composite_pipeline.layout().set_layouts()[0].clone(),
                [
                    WriteDescriptorSet::image_view_sampler(
                        0,
                        current_input,
                        composite_sampler.clone(),
                    ),
                    WriteDescriptorSet::image_view_sampler(
                        1,
                        ibl.prefiltered_map.view.clone(),
                        ibl.prefiltered_map.sampler.clone(),
                    ),
                    WriteDescriptorSet::image_view_sampler(
                        2,
                        gbuffer.normal.clone(),
                        composite_sampler.clone(),
                    ),
                    WriteDescriptorSet::image_view_sampler(
                        3,
                        gbuffer.metallic_roughness.clone(),
                        composite_sampler,
                    ),
                ],
                [],
            )
            .map_err(|e| eyre::eyre!("Failed to create composite descriptor set: {}", e))?;

            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.0, 0.0, 0.0, 0.0].into())],
                        ..RenderPassBeginInfo::framebuffer(self.composite_framebuffer.clone())
                    },
                    SubpassBeginInfo {
                        contents: vulkano::command_buffer::SubpassContents::Inline,
                        ..Default::default()
                    },
                )
                .map_err(|e| eyre::eyre!("Failed to begin composite render pass: {}", e))?;

            let viewport = Viewport {
                offset: [0.0, 0.0],
                extent: [self.width as f32, self.height as f32],
                depth_range: 0.0..=1.0,
            };

            builder
                .set_viewport(0, [viewport].into_iter().collect())
                .map_err(|e| eyre::eyre!("Failed to set viewport: {}", e))?;

            builder
                .bind_pipeline_graphics(self.composite_pipeline.clone())
                .map_err(|e| eyre::eyre!("Failed to bind composite pipeline: {}", e))?
                .bind_descriptor_sets(
                    vulkano::pipeline::PipelineBindPoint::Graphics,
                    self.composite_pipeline.layout().clone(),
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
                .map_err(|e| eyre::eyre!("Failed to end composite render pass: {}", e))?;

            trace!("SSR rendering complete (with environment probe fallback)");
            Ok(self.composite_texture.clone())
        } else {
            trace!("SSR rendering complete (without environment probe fallback)");
            Ok(current_input)
        }
    }

    /// Returns the final SSR texture.
    pub fn reflection_texture(&self) -> &Arc<ImageView> {
        &self.composite_texture
    }

    /// Returns the configuration.
    pub fn config(&self) -> &SsrConfig {
        &self.config
    }

    /// Resizes the SSR renderer to match new dimensions.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        info!("Resizing SSR renderer: {}x{}", width, height);

        let (ssr_texture, ssr_framebuffer) =
            Self::create_ssr_target(&self.memory_allocator, &self.ssr_render_pass, width, height)?;

        let (blur_texture_a, blur_framebuffer_a) = Self::create_blur_target(
            &self.memory_allocator,
            &self.blur_render_pass,
            width,
            height,
        )?;

        let (blur_texture_b, blur_framebuffer_b) = Self::create_blur_target(
            &self.memory_allocator,
            &self.blur_render_pass,
            width,
            height,
        )?;

        let (composite_texture, composite_framebuffer) = Self::create_composite_target(
            &self.memory_allocator,
            &self.composite_render_pass,
            width,
            height,
        )?;

        self.ssr_texture = ssr_texture;
        self.ssr_framebuffer = ssr_framebuffer;
        self.blur_texture_a = blur_texture_a;
        self.blur_framebuffer_a = blur_framebuffer_a;
        self.blur_texture_b = blur_texture_b;
        self.blur_framebuffer_b = blur_framebuffer_b;
        self.composite_texture = composite_texture;
        self.composite_framebuffer = composite_framebuffer;
        self.width = width;
        self.height = height;

        Ok(())
    }
}
