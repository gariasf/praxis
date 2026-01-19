//! Bloom post-processing effect.
//!
//! This module implements a complete bloom effect using:
//! - Brightness threshold extraction
//! - Separable Gaussian blur (horizontal and vertical passes)
//! - HDR tone mapping with configurable exposure
//! - Bloom intensity blending

use super::{full_screen_quad::FullScreenQuad, pass::PostProcessPass, render_target::RenderTarget};
use crate::shaders;
use praxis_utils::{debug, eyre, info, trace, Result};
use std::sync::Arc;
use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo,
        SubpassEndInfo,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, DescriptorSet, WriteDescriptorSet,
    },
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

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct BrightnessParams {
    threshold: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurParams {
    texel_size: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ToneMapParams {
    exposure: f32,
    bloom_intensity: f32,
}

pub struct BrightnessExtractionPass {
    pipeline: Arc<GraphicsPipeline>,
    quad: FullScreenQuad,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    threshold: f32,
    render_pass: Arc<RenderPass>,
}

impl BrightnessExtractionPass {
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        format: vulkano::format::Format,
        threshold: f32,
    ) -> Result<Self> {
        info!("Creating brightness extraction pass");

        let render_pass = create_post_process_render_pass(device.clone(), format)?;

        let vs_module = shaders::post_process_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load post-process vertex shader: {}", e))?;
        let fs_module =
            shaders::post_process_brightness_extract_fs::load(device.clone()).map_err(|e| {
                eyre::eyre!(
                    "Failed to load brightness extraction fragment shader: {}",
                    e
                )
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

        let quad = FullScreenQuad::new(memory_allocator)?;
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device,
            Default::default(),
        ));

        debug!("Brightness extraction pass created successfully");

        Ok(Self {
            pipeline,
            quad,
            descriptor_set_allocator,
            threshold,
            render_pass,
        })
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold;
    }
}

impl PostProcessPass for BrightnessExtractionPass {
    fn execute(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        input: &RenderTarget,
        output: &RenderTarget,
    ) -> Result<()> {
        trace!("Executing brightness extraction pass");

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

        let push_constants = BrightnessParams {
            threshold: self.threshold,
        };

        builder
            .push_constants(self.pipeline.layout().clone(), 0, push_constants)
            .map_err(|e| eyre::eyre!("Failed to push constants: {}", e))?;

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
        "BrightnessExtraction"
    }
}

pub struct GaussianBlurHorizontalPass {
    pipeline: Arc<GraphicsPipeline>,
    quad: FullScreenQuad,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    render_pass: Arc<RenderPass>,
}

impl GaussianBlurHorizontalPass {
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        format: vulkano::format::Format,
    ) -> Result<Self> {
        info!("Creating Gaussian blur horizontal pass");

        let render_pass = create_post_process_render_pass(device.clone(), format)?;

        let vs_module = shaders::post_process_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load post-process vertex shader: {}", e))?;
        let fs_module =
            shaders::post_process_gaussian_blur_h_fs::load(device.clone()).map_err(|e| {
                eyre::eyre!(
                    "Failed to load Gaussian blur horizontal fragment shader: {}",
                    e
                )
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

        let quad = FullScreenQuad::new(memory_allocator)?;
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device,
            Default::default(),
        ));

        debug!("Gaussian blur horizontal pass created successfully");

        Ok(Self {
            pipeline,
            quad,
            descriptor_set_allocator,
            render_pass,
        })
    }
}

impl PostProcessPass for GaussianBlurHorizontalPass {
    fn execute(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        input: &RenderTarget,
        output: &RenderTarget,
    ) -> Result<()> {
        trace!("Executing Gaussian blur horizontal pass");

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

        let push_constants = BlurParams {
            texel_size: [1.0 / input.width() as f32, 1.0 / input.height() as f32],
        };

        builder
            .push_constants(self.pipeline.layout().clone(), 0, push_constants)
            .map_err(|e| eyre::eyre!("Failed to push constants: {}", e))?;

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
        "GaussianBlurHorizontal"
    }
}

pub struct GaussianBlurVerticalPass {
    pipeline: Arc<GraphicsPipeline>,
    quad: FullScreenQuad,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    render_pass: Arc<RenderPass>,
}

impl GaussianBlurVerticalPass {
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        format: vulkano::format::Format,
    ) -> Result<Self> {
        info!("Creating Gaussian blur vertical pass");

        let render_pass = create_post_process_render_pass(device.clone(), format)?;

        let vs_module = shaders::post_process_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load post-process vertex shader: {}", e))?;
        let fs_module =
            shaders::post_process_gaussian_blur_v_fs::load(device.clone()).map_err(|e| {
                eyre::eyre!(
                    "Failed to load Gaussian blur vertical fragment shader: {}",
                    e
                )
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

        let quad = FullScreenQuad::new(memory_allocator)?;
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device,
            Default::default(),
        ));

        debug!("Gaussian blur vertical pass created successfully");

        Ok(Self {
            pipeline,
            quad,
            descriptor_set_allocator,
            render_pass,
        })
    }
}

impl PostProcessPass for GaussianBlurVerticalPass {
    fn execute(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        input: &RenderTarget,
        output: &RenderTarget,
    ) -> Result<()> {
        trace!("Executing Gaussian blur vertical pass");

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

        let push_constants = BlurParams {
            texel_size: [1.0 / input.width() as f32, 1.0 / input.height() as f32],
        };

        builder
            .push_constants(self.pipeline.layout().clone(), 0, push_constants)
            .map_err(|e| eyre::eyre!("Failed to push constants: {}", e))?;

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
        "GaussianBlurVertical"
    }
}

pub struct ToneMapPass {
    pipeline: Arc<GraphicsPipeline>,
    quad: FullScreenQuad,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    exposure: f32,
    bloom_intensity: f32,
    render_pass: Arc<RenderPass>,
}

impl ToneMapPass {
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        format: vulkano::format::Format,
        exposure: f32,
        bloom_intensity: f32,
    ) -> Result<Self> {
        info!("Creating tone map pass");

        let render_pass = create_post_process_render_pass(device.clone(), format)?;

        let vs_module = shaders::post_process_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load post-process vertex shader: {}", e))?;
        let fs_module = shaders::post_process_tone_map_fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load tone map fragment shader: {}", e))?;

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

        let quad = FullScreenQuad::new(memory_allocator)?;
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device,
            Default::default(),
        ));

        debug!("Tone map pass created successfully");

        Ok(Self {
            pipeline,
            quad,
            descriptor_set_allocator,
            exposure,
            bloom_intensity,
            render_pass,
        })
    }

    pub fn set_exposure(&mut self, exposure: f32) {
        self.exposure = exposure;
    }

    pub fn set_bloom_intensity(&mut self, bloom_intensity: f32) {
        self.bloom_intensity = bloom_intensity;
    }

    pub fn execute_with_bloom(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        scene_input: &RenderTarget,
        bloom_input: &RenderTarget,
        output: &RenderTarget,
    ) -> Result<()> {
        trace!("Executing tone map pass with bloom");

        let layout = self.pipeline.layout().set_layouts()[0].clone();
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout,
            [
                WriteDescriptorSet::image_view_sampler(
                    0,
                    scene_input.image_view().clone(),
                    scene_input.sampler().clone(),
                ),
                WriteDescriptorSet::image_view_sampler(
                    1,
                    bloom_input.image_view().clone(),
                    bloom_input.sampler().clone(),
                ),
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

        let push_constants = ToneMapParams {
            exposure: self.exposure,
            bloom_intensity: self.bloom_intensity,
        };

        builder
            .push_constants(self.pipeline.layout().clone(), 0, push_constants)
            .map_err(|e| eyre::eyre!("Failed to push constants: {}", e))?;

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

impl PostProcessPass for ToneMapPass {
    fn execute(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        input: &RenderTarget,
        output: &RenderTarget,
    ) -> Result<()> {
        self.execute_with_bloom(builder, input, input, output)
    }

    fn name(&self) -> &str {
        "ToneMap"
    }
}

#[derive(Debug, Clone)]
pub struct BloomConfig {
    pub brightness_threshold: f32,
    pub blur_iterations: u32,
    pub exposure: f32,
    pub bloom_intensity: f32,
}

impl Default for BloomConfig {
    fn default() -> Self {
        Self {
            brightness_threshold: 1.0,
            blur_iterations: 5,
            exposure: 1.0,
            bloom_intensity: 0.3,
        }
    }
}

impl BloomConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_brightness_threshold(mut self, threshold: f32) -> Self {
        self.brightness_threshold = threshold;
        self
    }

    pub fn with_blur_iterations(mut self, iterations: u32) -> Self {
        self.blur_iterations = iterations;
        self
    }

    pub fn with_exposure(mut self, exposure: f32) -> Self {
        self.exposure = exposure;
        self
    }

    pub fn with_bloom_intensity(mut self, intensity: f32) -> Self {
        self.bloom_intensity = intensity;
        self
    }
}

pub struct BloomEffect {
    brightness_pass: BrightnessExtractionPass,
    blur_h_pass: GaussianBlurHorizontalPass,
    blur_v_pass: GaussianBlurVerticalPass,
    tone_map_pass: ToneMapPass,
    config: BloomConfig,
    render_pass: Arc<RenderPass>,
}

impl BloomEffect {
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        format: vulkano::format::Format,
        config: BloomConfig,
    ) -> Result<Self> {
        info!("Creating bloom effect with config: {:?}", config);

        let render_pass = create_post_process_render_pass(device.clone(), format)?;

        let brightness_pass = BrightnessExtractionPass::new(
            device.clone(),
            memory_allocator.clone(),
            format,
            config.brightness_threshold,
        )?;

        let blur_h_pass =
            GaussianBlurHorizontalPass::new(device.clone(), memory_allocator.clone(), format)?;

        let blur_v_pass =
            GaussianBlurVerticalPass::new(device.clone(), memory_allocator.clone(), format)?;

        let tone_map_pass = ToneMapPass::new(
            device.clone(),
            memory_allocator.clone(),
            format,
            config.exposure,
            config.bloom_intensity,
        )?;

        info!("Bloom effect created successfully");

        Ok(Self {
            brightness_pass,
            blur_h_pass,
            blur_v_pass,
            tone_map_pass,
            config,
            render_pass,
        })
    }

    pub fn config(&self) -> &BloomConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut BloomConfig {
        &mut self.config
    }

    pub fn set_config(&mut self, config: BloomConfig) {
        self.brightness_pass
            .set_threshold(config.brightness_threshold);
        self.tone_map_pass.set_exposure(config.exposure);
        self.tone_map_pass
            .set_bloom_intensity(config.bloom_intensity);
        self.config = config;
    }

    pub fn update_config(&mut self) {
        self.brightness_pass
            .set_threshold(self.config.brightness_threshold);
        self.tone_map_pass.set_exposure(self.config.exposure);
        self.tone_map_pass
            .set_bloom_intensity(self.config.bloom_intensity);
    }

    pub fn render_pass(&self) -> &Arc<RenderPass> {
        &self.render_pass
    }

    pub fn apply(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        scene_input: &RenderTarget,
        output: &RenderTarget,
        pool: &mut super::render_target::RenderTargetPool,
    ) -> Result<()> {
        trace!("Applying bloom effect");

        let extent = scene_input.extent();

        let bright_target = pool.acquire(extent)?;
        self.brightness_pass
            .execute(builder, scene_input, &bright_target)?;

        let mut blur_input = bright_target.clone();
        for i in 0..self.config.blur_iterations {
            trace!(
                "Bloom blur iteration {}/{}",
                i + 1,
                self.config.blur_iterations
            );

            let blur_h_target = pool.acquire(extent)?;
            self.blur_h_pass
                .execute(builder, &blur_input, &blur_h_target)?;

            let blur_v_target = pool.acquire(extent)?;
            self.blur_v_pass
                .execute(builder, &blur_h_target, &blur_v_target)?;

            blur_input = blur_v_target;
        }

        self.tone_map_pass
            .execute_with_bloom(builder, scene_input, &blur_input, output)?;

        trace!("Bloom effect applied successfully");
        Ok(())
    }
}
