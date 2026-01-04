//! Cinematic post-processing effects.
//!
//! This module provides advanced post-processing effects for cinematic presentation:
//! - Depth-of-Field with circle of confusion and bokeh blur
//! - Motion blur using velocity buffers
//! - Chromatic aberration for lens distortion
//! - Vignette effect
//! - Film grain noise

use super::{full_screen_quad::FullScreenQuad, pass::PostProcessPass, render_target::RenderTarget};
use crate::shaders;
use praxis_math::Mat4;
use praxis_utils::{debug, eyre, info, trace, Result};
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
    image::view::ImageView,
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
    render_pass::{RenderPass, Subpass},
};

/// Helper function to create a render pass for post-processing.
fn create_post_process_render_pass(
    device: Arc<Device>,
    format: vulkano::format::Format,
) -> Result<Arc<RenderPass>> {
    vulkano::single_pass_renderpass!(
        device,
        attachments: {
            color: {
                format: format,
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
    .map_err(|e| eyre::eyre!("Failed to create post-process render pass: {}", e))
}

/// Configuration for depth-of-field effect.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DofConfig {
    /// Distance to focal plane in world units
    pub focus_distance: f32,
    /// Range around focal plane that stays sharp
    pub focus_range: f32,
    /// Maximum blur radius for out-of-focus areas
    pub bokeh_radius: f32,
    /// Aperture size (f-number)
    pub aperture: f32,
}

impl Default for DofConfig {
    fn default() -> Self {
        Self {
            focus_distance: 10.0,
            focus_range: 5.0,
            bokeh_radius: 8.0,
            aperture: 2.8,
        }
    }
}

/// Depth-of-Field post-processing effect.
///
/// Simulates realistic camera lens focus with circle of confusion calculation
/// and bokeh blur for out-of-focus areas.
pub struct DepthOfFieldPass {
    pipeline: Arc<GraphicsPipeline>,
    quad: FullScreenQuad,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    #[allow(dead_code)]
    render_pass: Arc<RenderPass>,
    config: DofConfig,
}

impl DepthOfFieldPass {
    /// Creates a new depth-of-field pass.
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        format: vulkano::format::Format,
        config: DofConfig,
    ) -> Result<Self> {
        info!("Creating depth-of-field post-processing pass");

        let render_pass = create_post_process_render_pass(device.clone(), format)?;

        let vs_module = shaders::post_process_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load post-process vertex shader: {}", e))?;
        let fs_module = shaders::post_process_dof_fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load DoF fragment shader: {}", e))?;

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

        let vertex_input_state = super::full_screen_quad::QuadVertex::per_vertex()
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
        .map_err(|e| eyre::eyre!("Failed to create graphics pipeline: {}", e))?;

        let quad = FullScreenQuad::new(memory_allocator.clone())?;
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device,
            Default::default(),
        ));

        debug!("Depth-of-field pass created successfully");

        Ok(Self {
            pipeline,
            quad,
            descriptor_set_allocator,
            memory_allocator,
            render_pass,
            config,
        })
    }

    /// Updates the depth-of-field configuration.
    pub fn set_config(&mut self, config: DofConfig) {
        self.config = config;
    }

    /// Gets the current depth-of-field configuration.
    pub fn config(&self) -> DofConfig {
        self.config
    }

    /// Executes the depth-of-field pass with a depth texture.
    pub fn execute_with_depth(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        input: &RenderTarget,
        depth: &Arc<ImageView>,
        depth_sampler: &Arc<vulkano::image::sampler::Sampler>,
        output: &RenderTarget,
    ) -> Result<()> {
        trace!("Executing depth-of-field pass");

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
            self.config,
        )
        .map_err(|e| eyre::eyre!("Failed to create DoF config buffer: {}", e))?;

        let layout = self.pipeline.layout().set_layouts()[0].clone();
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout,
            [
                WriteDescriptorSet::image_view_sampler(
                    0,
                    input.image_view().clone(),
                    input.sampler().clone(),
                ),
                WriteDescriptorSet::image_view_sampler(1, depth.clone(), depth_sampler.clone()),
                WriteDescriptorSet::buffer(2, config_buffer),
            ],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create descriptor set: {}", e))?;

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(output.framebuffer().clone())
                },
                SubpassBeginInfo {
                    contents: vulkano::command_buffer::SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .map_err(|e| eyre::eyre!("Failed to begin render pass: {}", e))?;

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [output.width() as f32, output.height() as f32],
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

        unsafe {
            builder
                .bind_vertex_buffers(0, self.quad.vertex_buffer().clone())
                .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
                .bind_index_buffer(self.quad.index_buffer().clone())
                .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?
                .draw_indexed(self.quad.index_count(), 1, 0, 0, 0)
                .map_err(|e| eyre::eyre!("Failed to draw indexed: {}", e))?;
        }

        builder
            .end_render_pass(SubpassEndInfo::default())
            .map_err(|e| eyre::eyre!("Failed to end render pass: {}", e))?;

        Ok(())
    }
}

impl PostProcessPass for DepthOfFieldPass {
    fn execute(
        &mut self,
        _builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        _input: &RenderTarget,
        _output: &RenderTarget,
    ) -> Result<()> {
        Err(eyre::eyre!(
            "DepthOfFieldPass requires depth texture. Use execute_with_depth() instead."
        ))
    }

    fn name(&self) -> &str {
        "Depth-of-Field"
    }

    fn requires_depth(&self) -> bool {
        true
    }
}

/// Configuration for motion blur effect.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MotionBlurConfig {
    /// Blur intensity multiplier
    pub intensity: f32,
    /// Number of samples along motion vector (max 32)
    pub sample_count: i32,
    /// Simulated camera shutter angle (0-360 degrees)
    pub shutter_angle: f32,
    /// Maximum blur radius in pixels
    pub max_blur_radius: f32,
}

impl Default for MotionBlurConfig {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            sample_count: 16,
            shutter_angle: 180.0,
            max_blur_radius: 32.0,
        }
    }
}

/// Motion blur post-processing effect using velocity buffer.
///
/// Creates realistic motion blur based on per-pixel velocity information.
pub struct MotionBlurPass {
    pipeline: Arc<GraphicsPipeline>,
    quad: FullScreenQuad,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    #[allow(dead_code)]
    render_pass: Arc<RenderPass>,
    config: MotionBlurConfig,
}

impl MotionBlurPass {
    /// Creates a new motion blur pass.
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        format: vulkano::format::Format,
        config: MotionBlurConfig,
    ) -> Result<Self> {
        info!("Creating motion blur post-processing pass");

        let render_pass = create_post_process_render_pass(device.clone(), format)?;

        let vs_module = shaders::post_process_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load post-process vertex shader: {}", e))?;
        let fs_module = shaders::post_process_motion_blur_fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load motion blur fragment shader: {}", e))?;

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

        let vertex_input_state = super::full_screen_quad::QuadVertex::per_vertex()
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
        .map_err(|e| eyre::eyre!("Failed to create graphics pipeline: {}", e))?;

        let quad = FullScreenQuad::new(memory_allocator.clone())?;
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device,
            Default::default(),
        ));

        debug!("Motion blur pass created successfully");

        Ok(Self {
            pipeline,
            quad,
            descriptor_set_allocator,
            memory_allocator,
            render_pass,
            config,
        })
    }

    /// Updates the motion blur configuration.
    pub fn set_config(&mut self, config: MotionBlurConfig) {
        self.config = config;
    }

    /// Gets the current motion blur configuration.
    pub fn config(&self) -> MotionBlurConfig {
        self.config
    }

    /// Executes the motion blur pass with a velocity texture.
    pub fn execute_with_velocity(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        input: &RenderTarget,
        velocity: &Arc<ImageView>,
        velocity_sampler: &Arc<vulkano::image::sampler::Sampler>,
        output: &RenderTarget,
    ) -> Result<()> {
        trace!("Executing motion blur pass");

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
            self.config,
        )
        .map_err(|e| eyre::eyre!("Failed to create motion blur config buffer: {}", e))?;

        let layout = self.pipeline.layout().set_layouts()[0].clone();
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout,
            [
                WriteDescriptorSet::image_view_sampler(
                    0,
                    input.image_view().clone(),
                    input.sampler().clone(),
                ),
                WriteDescriptorSet::image_view_sampler(
                    1,
                    velocity.clone(),
                    velocity_sampler.clone(),
                ),
                WriteDescriptorSet::buffer(2, config_buffer),
            ],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create descriptor set: {}", e))?;

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(output.framebuffer().clone())
                },
                SubpassBeginInfo {
                    contents: vulkano::command_buffer::SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .map_err(|e| eyre::eyre!("Failed to begin render pass: {}", e))?;

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [output.width() as f32, output.height() as f32],
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

        unsafe {
            builder
                .bind_vertex_buffers(0, self.quad.vertex_buffer().clone())
                .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
                .bind_index_buffer(self.quad.index_buffer().clone())
                .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?
                .draw_indexed(self.quad.index_count(), 1, 0, 0, 0)
                .map_err(|e| eyre::eyre!("Failed to draw indexed: {}", e))?;
        }

        builder
            .end_render_pass(SubpassEndInfo::default())
            .map_err(|e| eyre::eyre!("Failed to end render pass: {}", e))?;

        Ok(())
    }
}

impl PostProcessPass for MotionBlurPass {
    fn execute(
        &mut self,
        _builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        _input: &RenderTarget,
        _output: &RenderTarget,
    ) -> Result<()> {
        Err(eyre::eyre!(
            "MotionBlurPass requires velocity texture. Use execute_with_velocity() instead."
        ))
    }

    fn name(&self) -> &str {
        "Motion Blur"
    }
}

/// Configuration for chromatic aberration effect.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ChromaticAberrationConfig {
    /// Aberration intensity
    pub intensity: f32,
    /// How much effect increases toward edges
    pub radial_falloff: f32,
    /// Direction of aberration (x, y)
    pub direction: [f32; 2],
    /// Red channel offset multiplier
    pub red_offset: f32,
    /// Blue channel offset multiplier
    pub blue_offset: f32,
    /// Padding for alignment
    _padding: [f32; 3],
}

impl Default for ChromaticAberrationConfig {
    fn default() -> Self {
        Self {
            intensity: 0.003,
            radial_falloff: 2.0,
            direction: [0.0, 0.0],
            red_offset: 1.0,
            blue_offset: 1.0,
            _padding: [0.0; 3],
        }
    }
}

/// Chromatic aberration post-processing effect.
///
/// Simulates lens color fringing for realistic lens distortion.
pub struct ChromaticAberrationPass {
    pipeline: Arc<GraphicsPipeline>,
    quad: FullScreenQuad,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    #[allow(dead_code)]
    render_pass: Arc<RenderPass>,
    config: ChromaticAberrationConfig,
}

impl ChromaticAberrationPass {
    /// Creates a new chromatic aberration pass.
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        format: vulkano::format::Format,
        config: ChromaticAberrationConfig,
    ) -> Result<Self> {
        info!("Creating chromatic aberration post-processing pass");

        let render_pass = create_post_process_render_pass(device.clone(), format)?;

        let vs_module = shaders::post_process_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load post-process vertex shader: {}", e))?;
        let fs_module = shaders::post_process_chromatic_aberration_fs::load(device.clone())
            .map_err(|e| {
                eyre::eyre!("Failed to load chromatic aberration fragment shader: {}", e)
            })?;

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

        let vertex_input_state = super::full_screen_quad::QuadVertex::per_vertex()
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
        .map_err(|e| eyre::eyre!("Failed to create graphics pipeline: {}", e))?;

        let quad = FullScreenQuad::new(memory_allocator.clone())?;
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device,
            Default::default(),
        ));

        debug!("Chromatic aberration pass created successfully");

        Ok(Self {
            pipeline,
            quad,
            descriptor_set_allocator,
            memory_allocator,
            render_pass,
            config,
        })
    }

    /// Updates the chromatic aberration configuration.
    pub fn set_config(&mut self, config: ChromaticAberrationConfig) {
        self.config = config;
    }

    /// Gets the current chromatic aberration configuration.
    pub fn config(&self) -> ChromaticAberrationConfig {
        self.config
    }
}

impl PostProcessPass for ChromaticAberrationPass {
    fn execute(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        input: &RenderTarget,
        output: &RenderTarget,
    ) -> Result<()> {
        trace!("Executing chromatic aberration pass");

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
            self.config,
        )
        .map_err(|e| eyre::eyre!("Failed to create chromatic aberration config buffer: {}", e))?;

        let layout = self.pipeline.layout().set_layouts()[0].clone();
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout,
            [
                WriteDescriptorSet::image_view_sampler(
                    0,
                    input.image_view().clone(),
                    input.sampler().clone(),
                ),
                WriteDescriptorSet::buffer(1, config_buffer),
            ],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create descriptor set: {}", e))?;

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(output.framebuffer().clone())
                },
                SubpassBeginInfo {
                    contents: vulkano::command_buffer::SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .map_err(|e| eyre::eyre!("Failed to begin render pass: {}", e))?;

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [output.width() as f32, output.height() as f32],
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

        unsafe {
            builder
                .bind_vertex_buffers(0, self.quad.vertex_buffer().clone())
                .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
                .bind_index_buffer(self.quad.index_buffer().clone())
                .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?
                .draw_indexed(self.quad.index_count(), 1, 0, 0, 0)
                .map_err(|e| eyre::eyre!("Failed to draw indexed: {}", e))?;
        }

        builder
            .end_render_pass(SubpassEndInfo::default())
            .map_err(|e| eyre::eyre!("Failed to end render pass: {}", e))?;

        Ok(())
    }

    fn name(&self) -> &str {
        "Chromatic Aberration"
    }
}

/// Configuration for vignette effect.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VignetteConfig {
    /// Darkness intensity at edges
    pub intensity: f32,
    /// How gradual the vignette transition is
    pub smoothness: f32,
    /// Roundness of vignette shape (0 = rectangular, 1 = circular)
    pub roundness: f32,
    /// Center point of vignette effect (x, y)
    pub center: [f32; 2],
    /// Padding for alignment
    _padding: [f32; 2],
}

impl Default for VignetteConfig {
    fn default() -> Self {
        Self {
            intensity: 0.8,
            smoothness: 0.5,
            roundness: 1.0,
            center: [0.5, 0.5],
            _padding: [0.0; 2],
        }
    }
}

/// Vignette post-processing effect.
///
/// Darkens the edges of the image for cinematic presentation.
pub struct VignettePass {
    pipeline: Arc<GraphicsPipeline>,
    quad: FullScreenQuad,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    #[allow(dead_code)]
    render_pass: Arc<RenderPass>,
    config: VignetteConfig,
}

impl VignettePass {
    /// Creates a new vignette pass.
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        format: vulkano::format::Format,
        config: VignetteConfig,
    ) -> Result<Self> {
        info!("Creating vignette post-processing pass");

        let render_pass = create_post_process_render_pass(device.clone(), format)?;

        let vs_module = shaders::post_process_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load post-process vertex shader: {}", e))?;
        let fs_module = shaders::post_process_vignette_fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load vignette fragment shader: {}", e))?;

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

        let vertex_input_state = super::full_screen_quad::QuadVertex::per_vertex()
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
        .map_err(|e| eyre::eyre!("Failed to create graphics pipeline: {}", e))?;

        let quad = FullScreenQuad::new(memory_allocator.clone())?;
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device,
            Default::default(),
        ));

        debug!("Vignette pass created successfully");

        Ok(Self {
            pipeline,
            quad,
            descriptor_set_allocator,
            memory_allocator,
            render_pass,
            config,
        })
    }

    /// Updates the vignette configuration.
    pub fn set_config(&mut self, config: VignetteConfig) {
        self.config = config;
    }

    /// Gets the current vignette configuration.
    pub fn config(&self) -> VignetteConfig {
        self.config
    }
}

impl PostProcessPass for VignettePass {
    fn execute(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        input: &RenderTarget,
        output: &RenderTarget,
    ) -> Result<()> {
        trace!("Executing vignette pass");

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
            self.config,
        )
        .map_err(|e| eyre::eyre!("Failed to create vignette config buffer: {}", e))?;

        let layout = self.pipeline.layout().set_layouts()[0].clone();
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout,
            [
                WriteDescriptorSet::image_view_sampler(
                    0,
                    input.image_view().clone(),
                    input.sampler().clone(),
                ),
                WriteDescriptorSet::buffer(1, config_buffer),
            ],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create descriptor set: {}", e))?;

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(output.framebuffer().clone())
                },
                SubpassBeginInfo {
                    contents: vulkano::command_buffer::SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .map_err(|e| eyre::eyre!("Failed to begin render pass: {}", e))?;

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [output.width() as f32, output.height() as f32],
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

        unsafe {
            builder
                .bind_vertex_buffers(0, self.quad.vertex_buffer().clone())
                .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
                .bind_index_buffer(self.quad.index_buffer().clone())
                .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?
                .draw_indexed(self.quad.index_count(), 1, 0, 0, 0)
                .map_err(|e| eyre::eyre!("Failed to draw indexed: {}", e))?;
        }

        builder
            .end_render_pass(SubpassEndInfo::default())
            .map_err(|e| eyre::eyre!("Failed to end render pass: {}", e))?;

        Ok(())
    }

    fn name(&self) -> &str {
        "Vignette"
    }
}

/// Configuration for film grain effect.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FilmGrainConfig {
    /// Grain intensity
    pub intensity: f32,
    /// Grain particle size
    pub size: f32,
    /// How much grain intensity varies with luminance
    pub luminance_impact: f32,
    /// Time for animated grain
    pub time: f32,
}

impl Default for FilmGrainConfig {
    fn default() -> Self {
        Self {
            intensity: 0.05,
            size: 2.0,
            luminance_impact: 0.5,
            time: 0.0,
        }
    }
}

/// Film grain post-processing effect.
///
/// Adds procedural grain noise to simulate film stock for cinematic presentation.
pub struct FilmGrainPass {
    pipeline: Arc<GraphicsPipeline>,
    quad: FullScreenQuad,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    #[allow(dead_code)]
    render_pass: Arc<RenderPass>,
    config: FilmGrainConfig,
}

impl FilmGrainPass {
    /// Creates a new film grain pass.
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        format: vulkano::format::Format,
        config: FilmGrainConfig,
    ) -> Result<Self> {
        info!("Creating film grain post-processing pass");

        let render_pass = create_post_process_render_pass(device.clone(), format)?;

        let vs_module = shaders::post_process_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load post-process vertex shader: {}", e))?;
        let fs_module = shaders::post_process_film_grain_fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load film grain fragment shader: {}", e))?;

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

        let vertex_input_state = super::full_screen_quad::QuadVertex::per_vertex()
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
        .map_err(|e| eyre::eyre!("Failed to create graphics pipeline: {}", e))?;

        let quad = FullScreenQuad::new(memory_allocator.clone())?;
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device,
            Default::default(),
        ));

        debug!("Film grain pass created successfully");

        Ok(Self {
            pipeline,
            quad,
            descriptor_set_allocator,
            memory_allocator,
            render_pass,
            config,
        })
    }

    /// Updates the film grain configuration.
    pub fn set_config(&mut self, config: FilmGrainConfig) {
        self.config = config;
    }

    /// Gets the current film grain configuration.
    pub fn config(&self) -> FilmGrainConfig {
        self.config
    }
}

impl PostProcessPass for FilmGrainPass {
    fn execute(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        input: &RenderTarget,
        output: &RenderTarget,
    ) -> Result<()> {
        trace!("Executing film grain pass");

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
            self.config,
        )
        .map_err(|e| eyre::eyre!("Failed to create film grain config buffer: {}", e))?;

        let layout = self.pipeline.layout().set_layouts()[0].clone();
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout,
            [
                WriteDescriptorSet::image_view_sampler(
                    0,
                    input.image_view().clone(),
                    input.sampler().clone(),
                ),
                WriteDescriptorSet::buffer(1, config_buffer),
            ],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create descriptor set: {}", e))?;

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(output.framebuffer().clone())
                },
                SubpassBeginInfo {
                    contents: vulkano::command_buffer::SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .map_err(|e| eyre::eyre!("Failed to begin render pass: {}", e))?;

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [output.width() as f32, output.height() as f32],
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

        unsafe {
            builder
                .bind_vertex_buffers(0, self.quad.vertex_buffer().clone())
                .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
                .bind_index_buffer(self.quad.index_buffer().clone())
                .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?
                .draw_indexed(self.quad.index_count(), 1, 0, 0, 0)
                .map_err(|e| eyre::eyre!("Failed to draw indexed: {}", e))?;
        }

        builder
            .end_render_pass(SubpassEndInfo::default())
            .map_err(|e| eyre::eyre!("Failed to end render pass: {}", e))?;

        Ok(())
    }

    fn name(&self) -> &str {
        "Film Grain"
    }
}

/// Velocity buffer uniform data for motion blur.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VelocityUniforms {
    /// Current frame model-view-projection matrix
    pub current_mvp: [[f32; 4]; 4],
    /// Previous frame model-view-projection matrix
    pub previous_mvp: [[f32; 4]; 4],
}

impl VelocityUniforms {
    /// Creates velocity uniforms from current and previous MVP matrices.
    pub fn new(current_mvp: Mat4, previous_mvp: Mat4) -> Self {
        Self {
            current_mvp: current_mvp.to_cols_array_2d(),
            previous_mvp: previous_mvp.to_cols_array_2d(),
        }
    }
}
