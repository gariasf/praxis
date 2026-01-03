//! Tone mapping for HDR to LDR conversion.
//!
//! This module provides multiple tone mapping operators to convert
//! HDR values to displayable LDR range [0,1].

use super::{exposure::ExposureCalculator, render_target::HdrRenderTarget, ExposureMode};
use crate::{post_process::FullScreenQuad, shaders};
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
    format::Format,
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
    render_pass::{Framebuffer, RenderPass, Subpass},
};

/// Tone mapping operator selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToneMappingOperator {
    /// Reinhard tone mapping: simple and fast
    /// Formula: color / (color + 1)
    Reinhard,
    /// ACES Filmic tone mapping: industry standard, cinematic look
    /// Used in many AAA games and film production
    ACES,
    /// Uncharted 2 tone mapping: used in Uncharted 2, good contrast
    /// Also known as Hable tone mapping
    Uncharted2,
}

impl Default for ToneMappingOperator {
    fn default() -> Self {
        Self::ACES
    }
}

impl ToneMappingOperator {
    /// Returns the shader constant value for this operator.
    fn to_shader_value(self) -> u32 {
        match self {
            Self::Reinhard => 0,
            Self::ACES => 1,
            Self::Uncharted2 => 2,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ToneMapPushConstants {
    exposure: f32,
    gamma: f32,
    operator: u32,
    _padding: u32,
}

fn create_hdr_render_pass(device: Arc<Device>, format: Format) -> Result<Arc<RenderPass>> {
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
    .map_err(|e| eyre::eyre!("Failed to create HDR render pass: {}", e))
}

/// Tone mapping pass for HDR to LDR conversion.
///
/// Converts HDR floating-point values to LDR [0,1] range using
/// the selected tone mapping operator.
pub struct ToneMapPass {
    pipeline: Arc<GraphicsPipeline>,
    quad: FullScreenQuad,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    operator: ToneMappingOperator,
    gamma: f32,
    #[allow(dead_code)]
    render_pass: Arc<RenderPass>,
}

impl ToneMapPass {
    /// Creates a new tone mapping pass.
    ///
    /// # Arguments
    ///
    /// * `device` - Vulkan device
    /// * `memory_allocator` - Memory allocator for buffers
    /// * `format` - Output format (typically R8G8B8A8_UNORM)
    /// * `operator` - Tone mapping operator to use
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        format: Format,
        operator: ToneMappingOperator,
    ) -> Result<Self> {
        info!("Creating tone map pass with operator: {:?}", operator);

        let render_pass = create_hdr_render_pass(device.clone(), format)?;

        let vs_module = shaders::post_process_vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load post-process vertex shader: {}", e))?;
        let fs_module = shaders::hdr_tone_map_fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load HDR tone map fragment shader: {}", e))?;

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

        let vertex_input_state = crate::post_process::QuadVertex::per_vertex()
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
            operator,
            gamma: 2.2,
            render_pass,
        })
    }

    /// Sets the tone mapping operator.
    pub fn set_operator(&mut self, operator: ToneMappingOperator) {
        self.operator = operator;
    }

    /// Returns the current tone mapping operator.
    pub fn operator(&self) -> ToneMappingOperator {
        self.operator
    }

    /// Sets the gamma correction value.
    pub fn set_gamma(&mut self, gamma: f32) {
        self.gamma = gamma;
    }

    /// Returns the current gamma value.
    pub fn gamma(&self) -> f32 {
        self.gamma
    }

    /// Executes the tone mapping pass.
    ///
    /// # Arguments
    ///
    /// * `builder` - Command buffer builder
    /// * `hdr_input` - HDR input texture
    /// * `output_framebuffer` - LDR output framebuffer
    /// * `output_extent` - Output dimensions
    /// * `exposure` - Exposure value
    pub fn execute(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        hdr_input: &HdrRenderTarget,
        output_framebuffer: &Arc<Framebuffer>,
        output_extent: [u32; 2],
        exposure: f32,
    ) -> Result<()> {
        trace!("Executing tone map pass with operator: {:?}", self.operator);

        let layout = self.pipeline.layout().set_layouts()[0].clone();
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout,
            [WriteDescriptorSet::image_view_sampler(
                0,
                hdr_input.image_view().clone(),
                hdr_input.sampler().clone(),
            )],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create descriptor set: {}", e))?;

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(output_framebuffer.clone())
                },
                SubpassBeginInfo {
                    contents: vulkano::command_buffer::SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .map_err(|e| eyre::eyre!("Failed to begin render pass: {}", e))?;

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [output_extent[0] as f32, output_extent[1] as f32],
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

        let push_constants = ToneMapPushConstants {
            exposure,
            gamma: self.gamma,
            operator: self.operator.to_shader_value(),
            _padding: 0,
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

/// Complete tone mapper with exposure calculation.
///
/// Combines exposure calculation and tone mapping into a single
/// high-level interface for HDR to LDR conversion.
pub struct ToneMapper {
    tone_map_pass: ToneMapPass,
    exposure_calculator: ExposureCalculator,
}

impl ToneMapper {
    /// Creates a new tone mapper.
    ///
    /// # Arguments
    ///
    /// * `device` - Vulkan device
    /// * `memory_allocator` - Memory allocator
    /// * `format` - Output format
    /// * `operator` - Tone mapping operator
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        format: Format,
        operator: ToneMappingOperator,
    ) -> Result<Self> {
        info!("Creating tone mapper with operator: {:?}", operator);

        let tone_map_pass = ToneMapPass::new(device, memory_allocator, format, operator)?;
        let exposure_calculator = ExposureCalculator::new(ExposureMode::default());

        Ok(Self {
            tone_map_pass,
            exposure_calculator,
        })
    }

    /// Applies tone mapping with automatic exposure calculation.
    ///
    /// # Arguments
    ///
    /// * `builder` - Command buffer builder
    /// * `hdr_input` - HDR input texture
    /// * `output_framebuffer` - LDR output framebuffer
    /// * `output_extent` - Output dimensions
    /// * `average_luminance` - Scene average luminance for auto-exposure
    /// * `delta_time` - Time delta for smooth exposure adaptation
    pub fn apply(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        hdr_input: &HdrRenderTarget,
        output_framebuffer: &Arc<Framebuffer>,
        output_extent: [u32; 2],
        average_luminance: f32,
        delta_time: f32,
    ) -> Result<()> {
        let exposure = self
            .exposure_calculator
            .calculate(average_luminance, delta_time);

        self.tone_map_pass.execute(
            builder,
            hdr_input,
            output_framebuffer,
            output_extent,
            exposure,
        )
    }

    /// Applies tone mapping with manual exposure.
    pub fn apply_with_exposure(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        hdr_input: &HdrRenderTarget,
        output_framebuffer: &Arc<Framebuffer>,
        output_extent: [u32; 2],
        exposure: f32,
    ) -> Result<()> {
        self.tone_map_pass
            .execute(builder, hdr_input, output_framebuffer, output_extent, exposure)
    }

    /// Sets the tone mapping operator.
    pub fn set_operator(&mut self, operator: ToneMappingOperator) {
        self.tone_map_pass.set_operator(operator);
    }

    /// Returns the current tone mapping operator.
    pub fn operator(&self) -> ToneMappingOperator {
        self.tone_map_pass.operator()
    }

    /// Sets the exposure mode.
    pub fn set_exposure_mode(&mut self, mode: ExposureMode) {
        self.exposure_calculator.set_mode(mode);
    }

    /// Returns the current exposure mode.
    pub fn exposure_mode(&self) -> ExposureMode {
        self.exposure_calculator.mode()
    }

    /// Returns the current exposure value.
    pub fn current_exposure(&self) -> f32 {
        self.exposure_calculator.current_exposure()
    }

    /// Sets the gamma correction value.
    pub fn set_gamma(&mut self, gamma: f32) {
        self.tone_map_pass.set_gamma(gamma);
    }

    /// Returns the current gamma value.
    pub fn gamma(&self) -> f32 {
        self.tone_map_pass.gamma()
    }

    /// Returns a reference to the exposure calculator.
    pub fn exposure_calculator(&self) -> &ExposureCalculator {
        &self.exposure_calculator
    }

    /// Returns a mutable reference to the exposure calculator.
    pub fn exposure_calculator_mut(&mut self) -> &mut ExposureCalculator {
        &mut self.exposure_calculator
    }
}
