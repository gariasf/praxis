//! Built-in post-processing pass implementations.
//!
//! This module provides common post-processing effects that can be used
//! out of the box.

use super::{full_screen_quad::FullScreenQuad, pass::PostProcessPass, render_target::RenderTarget};
use crate::shaders;
use praxis_utils::{debug, eyre, info, trace, Result};
use std::sync::Arc;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassEndInfo},
    descriptor_set::{allocator::StandardDescriptorSetAllocator, DescriptorSet, WriteDescriptorSet},
    device::Device,
    memory::allocator::StandardMemoryAllocator,
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
///
/// Creates a simple render pass with a single color attachment.
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

/// Copy pass - simply copies input to output.
///
/// This is the most basic post-processing pass. It's useful for testing
/// the post-processing infrastructure or as a template for custom effects.
pub struct CopyPass {
    pipeline: Arc<GraphicsPipeline>,
    quad: FullScreenQuad,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    #[allow(dead_code)]
    render_pass: Arc<RenderPass>,
}

impl CopyPass {
    /// Creates a new copy pass.
    ///
    /// # Arguments
    ///
    /// * `device` - Vulkan device
    /// * `memory_allocator` - Memory allocator for buffers
    /// * `format` - Color format for the output
    ///
    /// # Errors
    ///
    /// Returns an error if pipeline or resource creation fails.
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        format: vulkano::format::Format,
    ) -> Result<Self> {
        info!("Creating copy post-processing pass");

        // Create render pass
        let render_pass = create_post_process_render_pass(device.clone(), format)?;

        // Load shaders
        let vs_module = shaders::post_process_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load post-process vertex shader: {}", e))?;
        let fs_module = shaders::post_process_copy_fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load copy fragment shader: {}", e))?;

        let vs_entry = vs_module
            .entry_point("main")
            .ok_or_else(|| eyre::eyre!("Failed to find main entry point in vertex shader"))?;
        let fs_entry = fs_module
            .entry_point("main")
            .ok_or_else(|| eyre::eyre!("Failed to find main entry point in fragment shader"))?;

        // Create pipeline stages
        let stages = [
            PipelineShaderStageCreateInfo::new(vs_entry.clone()),
            PipelineShaderStageCreateInfo::new(fs_entry),
        ];

        // Vertex input state
        let vertex_input_state = super::full_screen_quad::QuadVertex::per_vertex()
            .definition(&vs_entry)
            .map_err(|e| eyre::eyre!("Failed to create vertex input state: {}", e))?;

        // Pipeline layout
        let layout_create_infos = PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .map_err(|e| eyre::eyre!("Failed to create pipeline layout info: {}", e))?;

        let layout = PipelineLayout::new(device.clone(), layout_create_infos)
            .map_err(|e| eyre::eyre!("Failed to create pipeline layout: {}", e))?;

        let subpass = Subpass::from(render_pass.clone(), 0)
            .ok_or_else(|| eyre::eyre!("Failed to get subpass"))?;

        // Create pipeline
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

        // Create full-screen quad
        let quad = FullScreenQuad::new(memory_allocator)?;

        // Create descriptor set allocator
        let descriptor_set_allocator =
            Arc::new(StandardDescriptorSetAllocator::new(device, Default::default()));

        debug!("Copy pass created successfully");

        Ok(Self {
            pipeline,
            quad,
            descriptor_set_allocator,
            render_pass,
        })
    }
}

impl PostProcessPass for CopyPass {
    fn execute(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        input: &RenderTarget,
        output: &RenderTarget,
    ) -> Result<()> {
        trace!("Executing copy pass");

        // Create descriptor set with input texture
        let layout = self.pipeline.layout().set_layouts()[0].clone();
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout,
            [WriteDescriptorSet::image_view_sampler(
                0,
                input.image_view().clone(),
                input.sampler().clone(),
            )],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create descriptor set: {}", e))?;

        // Begin render pass
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

        // Set viewport
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [output.width() as f32, output.height() as f32],
            depth_range: 0.0..=1.0,
        };

        builder
            .set_viewport(0, [viewport].into_iter().collect())
            .map_err(|e| eyre::eyre!("Failed to set viewport: {}", e))?;

        // Bind pipeline and descriptor set
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

        // Draw full-screen quad
        unsafe {
            builder
                .bind_vertex_buffers(0, self.quad.vertex_buffer().clone())
                .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
                .bind_index_buffer(self.quad.index_buffer().clone())
                .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?
                .draw_indexed(self.quad.index_count(), 1, 0, 0, 0)
                .map_err(|e| eyre::eyre!("Failed to draw indexed: {}", e))?;
        }

        // End render pass
        builder
            .end_render_pass(SubpassEndInfo::default())
            .map_err(|e| eyre::eyre!("Failed to end render pass: {}", e))?;

        Ok(())
    }

    fn name(&self) -> &str {
        "Copy"
    }
}

/// Grayscale pass - converts color to grayscale.
///
/// Uses the standard luminance formula: 0.299*R + 0.587*G + 0.114*B
pub struct GrayscalePass {
    pipeline: Arc<GraphicsPipeline>,
    quad: FullScreenQuad,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    #[allow(dead_code)]
    render_pass: Arc<RenderPass>,
}

impl GrayscalePass {
    /// Creates a new grayscale pass.
    ///
    /// # Arguments
    ///
    /// * `device` - Vulkan device
    /// * `memory_allocator` - Memory allocator for buffers
    /// * `format` - Color format for the output
    ///
    /// # Errors
    ///
    /// Returns an error if pipeline or resource creation fails.
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        format: vulkano::format::Format,
    ) -> Result<Self> {
        info!("Creating grayscale post-processing pass");

        // Create render pass
        let render_pass = create_post_process_render_pass(device.clone(), format)?;

        // Load shaders
        let vs_module = shaders::post_process_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load post-process vertex shader: {}", e))?;
        let fs_module = shaders::post_process_grayscale_fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load grayscale fragment shader: {}", e))?;

        let vs_entry = vs_module
            .entry_point("main")
            .ok_or_else(|| eyre::eyre!("Failed to find main entry point in vertex shader"))?;
        let fs_entry = fs_module
            .entry_point("main")
            .ok_or_else(|| eyre::eyre!("Failed to find main entry point in fragment shader"))?;

        // Create pipeline stages
        let stages = [
            PipelineShaderStageCreateInfo::new(vs_entry.clone()),
            PipelineShaderStageCreateInfo::new(fs_entry),
        ];

        // Vertex input state
        let vertex_input_state = super::full_screen_quad::QuadVertex::per_vertex()
            .definition(&vs_entry)
            .map_err(|e| eyre::eyre!("Failed to create vertex input state: {}", e))?;

        // Pipeline layout
        let layout_create_infos = PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .map_err(|e| eyre::eyre!("Failed to create pipeline layout info: {}", e))?;

        let layout = PipelineLayout::new(device.clone(), layout_create_infos)
            .map_err(|e| eyre::eyre!("Failed to create pipeline layout: {}", e))?;

        let subpass = Subpass::from(render_pass.clone(), 0)
            .ok_or_else(|| eyre::eyre!("Failed to get subpass"))?;

        // Create pipeline
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

        // Create full-screen quad
        let quad = FullScreenQuad::new(memory_allocator)?;

        // Create descriptor set allocator
        let descriptor_set_allocator =
            Arc::new(StandardDescriptorSetAllocator::new(device, Default::default()));

        debug!("Grayscale pass created successfully");

        Ok(Self {
            pipeline,
            quad,
            descriptor_set_allocator,
            render_pass,
        })
    }
}

impl PostProcessPass for GrayscalePass {
    fn execute(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        input: &RenderTarget,
        output: &RenderTarget,
    ) -> Result<()> {
        trace!("Executing grayscale pass");

        // Create descriptor set with input texture
        let layout = self.pipeline.layout().set_layouts()[0].clone();
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout,
            [WriteDescriptorSet::image_view_sampler(
                0,
                input.image_view().clone(),
                input.sampler().clone(),
            )],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create descriptor set: {}", e))?;

        // Begin render pass
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

        // Set viewport
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [output.width() as f32, output.height() as f32],
            depth_range: 0.0..=1.0,
        };

        builder
            .set_viewport(0, [viewport].into_iter().collect())
            .map_err(|e| eyre::eyre!("Failed to set viewport: {}", e))?;

        // Bind pipeline and descriptor set
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

        // Draw full-screen quad
        unsafe {
            builder
                .bind_vertex_buffers(0, self.quad.vertex_buffer().clone())
                .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
                .bind_index_buffer(self.quad.index_buffer().clone())
                .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?
                .draw_indexed(self.quad.index_count(), 1, 0, 0, 0)
                .map_err(|e| eyre::eyre!("Failed to draw indexed: {}", e))?;
        }

        // End render pass
        builder
            .end_render_pass(SubpassEndInfo::default())
            .map_err(|e| eyre::eyre!("Failed to end render pass: {}", e))?;

        Ok(())
    }

    fn name(&self) -> &str {
        "Grayscale"
    }
}
