//! Graphics pipeline creation and management.
//!
//! This module handles the creation of Vulkan graphics pipelines, which define
//! the entire graphics rendering process from vertices to pixels.
//!
//! # Graphics Pipeline Overview
//!
//! A graphics pipeline is a series of stages that process vertex data into pixels:
//!
//! ```text
//! Vertex Data
//!     │
//!     ▼
//! ┌─────────────────┐
//! │ Vertex Shader   │ ← Transforms vertices from model space to clip space
//! └─────────────────┘
//!     │
//!     ▼
//! ┌─────────────────┐
//! │ Rasterization   │ ← Converts primitives to fragments (pixels)
//! └─────────────────┘
//!     │
//!     ▼
//! ┌─────────────────┐
//! │ Fragment Shader │ ← Computes color for each fragment
//! └─────────────────┘
//!     │
//!     ▼
//! ┌─────────────────┐
//! │ Color Blending  │ ← Combines fragment color with framebuffer
//! └─────────────────┘
//!     │
//!     ▼
//! Framebuffer
//! ```

use crate::shaders;
use crate::vertex::VertexData;
use praxis_utils::{Result, debug, eyre};
use std::sync::Arc;
use vulkano::{
    device::Device,
    pipeline::{
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, RasterizationState},
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    render_pass::{RenderPass, Subpass},
};

/// Configuration for creating a graphics pipeline.
///
/// This struct provides a builder-like interface for configuring pipeline options
/// while maintaining sensible defaults for common use cases.
pub struct PipelineConfig {
    /// The primitive topology - how vertices are assembled into primitives.
    ///
    /// Common values:
    /// - `TriangleList`: Every 3 vertices form a triangle
    /// - `TriangleStrip`: Each vertex forms a triangle with the previous two
    /// - `LineList`: Every 2 vertices form a line
    pub primitive_topology: PrimitiveTopology,

    /// Whether to cull (discard) back-facing triangles.
    ///
    /// - `None`: No culling (render both sides)
    /// - `Back`: Cull back-facing triangles (default)
    /// - `Front`: Cull front-facing triangles
    pub cull_mode: CullMode,

    /// Which winding order is considered front-facing.
    ///
    /// - `CounterClockwise`: CCW is front (Vulkan default)
    /// - `Clockwise`: CW is front
    pub front_face: FrontFace,

    /// Enable depth testing and writing.
    ///
    /// Currently not implemented as we're doing 2D rendering.
    pub depth_test: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            primitive_topology: PrimitiveTopology::TriangleList,
            cull_mode: CullMode::None, // Disable culling for now (good for debugging)
            front_face: FrontFace::CounterClockwise,
            depth_test: false,
        }
    }
}

/// Creates a graphics pipeline for rendering basic geometry.
///
/// This function sets up the entire graphics pipeline including:
/// - Shader stages (vertex and fragment)
/// - Vertex input configuration
/// - Primitive assembly
/// - Viewport and scissor
/// - Rasterization settings
/// - Multisampling (disabled)
/// - Color blending (standard alpha blending)
///
/// # Arguments
///
/// * `device` - The Vulkan device to create the pipeline on
/// * `render_pass` - The render pass this pipeline will be used with
/// * `extent` - The viewport dimensions [width, height]
/// * `config` - Pipeline configuration options
///
/// # Returns
///
/// The created graphics pipeline ready for use in rendering.
///
/// # Errors
///
/// Returns an error if:
/// - Shader loading fails
/// - Pipeline creation fails
/// - Invalid configuration is provided
pub fn create_graphics_pipeline(
    device: &Arc<Device>,
    render_pass: &Arc<RenderPass>,
    extent: [u32; 2],
    config: PipelineConfig,
) -> Result<Arc<GraphicsPipeline>> {
    debug!("Creating graphics pipeline...");

    let (vs_entry, fs_entry) = load_shaders(device)?;

    // Create pipeline stages from shader entry points
    let stages = [
        PipelineShaderStageCreateInfo::new(vs_entry.clone()),
        PipelineShaderStageCreateInfo::new(fs_entry),
    ];

    // This tells Vulkan how to interpret our vertex buffer data
    let vertex_input_state = VertexData::per_vertex()
        .definition(&vs_entry.info().input_interface)
        .map_err(|e| eyre::eyre!("Failed to create vertex input state: {}", e))?;

    debug!("Configured vertex input for VertexData format");

    // This defines the interface between shaders and the application
    let layout = create_pipeline_layout(device, &stages)?;

    // Get the first subpass of our render pass
    // A subpass is a rendering phase within a render pass
    let subpass = Subpass::from(render_pass.clone(), 0)
        .ok_or_else(|| eyre::eyre!("Failed to get subpass from render pass"))?;

    // This defines the area of the framebuffer that we render to
    let viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [extent[0] as f32, extent[1] as f32],
        depth_range: 0.0..=1.0, // Depth range for depth buffer (even if not used)
    };

    let create_info = GraphicsPipelineCreateInfo {
        stages: stages.into_iter().collect(),

        vertex_input_state: Some(vertex_input_state),

        input_assembly_state: Some(InputAssemblyState {
            topology: config.primitive_topology,
            ..Default::default()
        }),

        viewport_state: Some(ViewportState {
            viewports: [viewport].into_iter().collect(),
            ..Default::default()
        }),

        rasterization_state: Some(RasterizationState {
            cull_mode: config.cull_mode,
            front_face: config.front_face,
            ..Default::default()
        }),

        // Multisampling (anti-aliasing) - disabled for now
        multisample_state: Some(MultisampleState::default()),

        // This uses standard alpha blending: finalColor = srcColor * srcAlpha + dstColor * (1 - srcAlpha)
        color_blend_state: Some(ColorBlendState::with_attachment_states(
            subpass.num_color_attachments(),
            ColorBlendAttachmentState {
                blend: Some(vulkano::pipeline::graphics::color_blend::AttachmentBlend::alpha()),
                color_write_mask: vulkano::pipeline::graphics::color_blend::ColorComponents::all(),
                color_write_enable: true,
            },
        )),

        // Dynamic state - these can be changed at draw time without recreating the pipeline (what is this?)
        dynamic_state: [DynamicState::Viewport].into_iter().collect(),

        // The render pass and subpass this pipeline is compatible with
        subpass: Some(subpass.into()),

        ..GraphicsPipelineCreateInfo::layout(layout)
    };

    let pipeline = GraphicsPipeline::new(
        device.clone(),
        None, // No pipeline cache
        create_info,
    )
    .map_err(|e| eyre::eyre!("Failed to create graphics pipeline: {}", e))?;

    debug!("Successfully created graphics pipeline");

    Ok(pipeline)
}

/// Loads vertex and fragment shaders.
///
/// This function loads the compiled SPIR-V shaders and finds their entry points.
/// The shaders are compiled from GLSL to SPIR-V at build time by the
/// vulkano-shaders macro.
///
/// # Returns
///
/// A tuple of (vertex_shader_entry_point, fragment_shader_entry_point)
fn load_shaders(
    device: &Arc<Device>,
) -> Result<(vulkano::shader::EntryPoint, vulkano::shader::EntryPoint)> {
    debug!("Loading shaders...");

    let vs_module = shaders::vs::load(device.clone())
        .map_err(|e| eyre::eyre!("Failed to load vertex shader: {}", e))?;

    let vs_entry = vs_module
        .entry_point("main")
        .ok_or_else(|| eyre::eyre!("Failed to find 'main' entry point in vertex shader"))?;

    debug!("Loaded vertex shader");

    let fs_module = shaders::fs::load(device.clone())
        .map_err(|e| eyre::eyre!("Failed to load fragment shader: {}", e))?;

    let fs_entry = fs_module
        .entry_point("main")
        .ok_or_else(|| eyre::eyre!("Failed to find 'main' entry point in fragment shader"))?;

    debug!("Loaded fragment shader");

    Ok((vs_entry, fs_entry))
}

/// Creates a pipeline layout from shader stages.
///
/// The pipeline layout describes the interface between the pipeline and
/// descriptor sets (textures, buffers, etc.) that shaders can access.
///
/// Currently, our shaders don't use any descriptor sets, so this creates
/// an empty layout. In the future, this would define:
/// - Uniform buffer layouts (for matrices, etc.)
/// - Texture and sampler bindings
/// - Storage buffer bindings
fn create_pipeline_layout(
    device: &Arc<Device>,
    stages: &[PipelineShaderStageCreateInfo],
) -> Result<Arc<PipelineLayout>> {
    debug!("Creating pipeline layout...");

    // Automatically derive the layout from shader stages
    // This inspects the shaders to determine what resources they expect
    let layout_create_info = PipelineDescriptorSetLayoutCreateInfo::from_stages(stages)
        .into_pipeline_layout_create_info(device.clone())
        .map_err(|e| eyre::eyre!("Failed to create pipeline layout info: {}", e))?;

    let layout = PipelineLayout::new(device.clone(), layout_create_info)
        .map_err(|e| eyre::eyre!("Failed to create pipeline layout: {}", e))?;

    debug!("Created pipeline layout");

    Ok(layout)
}

/// Creates a simple graphics pipeline with default configuration.
///
/// This is a convenience function that uses sensible defaults for basic rendering.
pub fn create_simple_pipeline(
    device: &Arc<Device>,
    render_pass: &Arc<RenderPass>,
    extent: [u32; 2],
) -> Result<Arc<GraphicsPipeline>> {
    create_graphics_pipeline(device, render_pass, extent, PipelineConfig::default())
}
