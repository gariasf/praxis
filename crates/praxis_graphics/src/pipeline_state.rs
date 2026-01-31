//! Pipeline state object management.
//!
//! This module provides a structured way to define and cache graphics pipeline
//! state objects (PSOs), which encapsulate all fixed-function GPU state.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use vulkano::{
    device::Device,
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, PolygonMode, RasterizationState},
            vertex_input::VertexDefinition,
            viewport::ViewportState,
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{RenderPass, Subpass},
};

use praxis_utils::{debug, eyre, info, Result};

/// Blend mode for color attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendMode {
    /// No blending (opaque).
    None,
    /// Alpha blending (src_alpha, one_minus_src_alpha).
    Alpha,
    /// Additive blending (one, one).
    Additive,
    /// Premultiplied alpha blending.
    PremultipliedAlpha,
}

impl BlendMode {
    /// Converts to Vulkan color blend attachment state.
    pub fn to_attachment_state(self) -> ColorBlendAttachmentState {
        use vulkano::pipeline::graphics::color_blend::{
            AttachmentBlend, BlendFactor, BlendOp, ColorComponents,
        };

        match self {
            BlendMode::None => ColorBlendAttachmentState {
                blend: None,
                color_write_mask: ColorComponents::all(),
                color_write_enable: true,
            },
            BlendMode::Alpha => ColorBlendAttachmentState {
                blend: Some(AttachmentBlend::alpha()),
                color_write_mask: ColorComponents::all(),
                color_write_enable: true,
            },
            BlendMode::Additive => ColorBlendAttachmentState {
                blend: Some(AttachmentBlend {
                    color_blend_op: BlendOp::Add,
                    src_color_blend_factor: BlendFactor::One,
                    dst_color_blend_factor: BlendFactor::One,
                    alpha_blend_op: BlendOp::Add,
                    src_alpha_blend_factor: BlendFactor::One,
                    dst_alpha_blend_factor: BlendFactor::One,
                }),
                color_write_mask: ColorComponents::all(),
                color_write_enable: true,
            },
            BlendMode::PremultipliedAlpha => ColorBlendAttachmentState {
                blend: Some(AttachmentBlend {
                    color_blend_op: BlendOp::Add,
                    src_color_blend_factor: BlendFactor::One,
                    dst_color_blend_factor: BlendFactor::OneMinusSrcAlpha,
                    alpha_blend_op: BlendOp::Add,
                    src_alpha_blend_factor: BlendFactor::One,
                    dst_alpha_blend_factor: BlendFactor::OneMinusSrcAlpha,
                }),
                color_write_mask: ColorComponents::all(),
                color_write_enable: true,
            },
        }
    }
}

/// Depth test configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DepthTestConfig {
    /// Enable depth testing.
    pub enable: bool,
    /// Enable depth writing.
    pub write_enable: bool,
    /// Comparison operator for depth test.
    pub compare_op: CompareOp,
}

impl Default for DepthTestConfig {
    fn default() -> Self {
        Self {
            enable: true,
            write_enable: true,
            compare_op: CompareOp::Less,
        }
    }
}

impl DepthTestConfig {
    /// Creates depth test config with no depth testing.
    pub fn disabled() -> Self {
        Self {
            enable: false,
            write_enable: false,
            compare_op: CompareOp::Always,
        }
    }

    /// Creates depth test config with depth testing but no writing.
    pub fn test_only() -> Self {
        Self {
            enable: true,
            write_enable: false,
            compare_op: CompareOp::Less,
        }
    }
}

/// Rasterization state configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RasterizationConfig {
    /// Polygon rasterization mode.
    pub polygon_mode: PolygonMode,
    /// Face culling mode.
    pub cull_mode: CullMode,
    /// Front-facing triangle winding order.
    pub front_face: FrontFace,
    /// Line width (only used if polygon_mode is Line).
    pub line_width: u32,
}

impl Default for RasterizationConfig {
    fn default() -> Self {
        Self {
            polygon_mode: PolygonMode::Fill,
            cull_mode: CullMode::Back,
            front_face: FrontFace::CounterClockwise,
            line_width: 1,
        }
    }
}

/// Complete pipeline state object configuration.
///
/// Encapsulates all fixed-function GPU state for a graphics pipeline.
#[derive(Clone)]
pub struct PipelineStateConfig {
    /// Primitive topology (triangles, lines, points).
    pub primitive_topology: PrimitiveTopology,
    /// Rasterization configuration.
    pub rasterization: RasterizationConfig,
    /// Depth test configuration.
    pub depth_test: DepthTestConfig,
    /// Color blend mode.
    pub blend_mode: BlendMode,
    /// Dynamic states (can be changed at draw time).
    pub dynamic_states: Vec<DynamicState>,
}

impl Default for PipelineStateConfig {
    fn default() -> Self {
        Self {
            primitive_topology: PrimitiveTopology::TriangleList,
            rasterization: RasterizationConfig::default(),
            depth_test: DepthTestConfig::default(),
            blend_mode: BlendMode::Alpha,
            dynamic_states: vec![DynamicState::Viewport],
        }
    }
}

impl PipelineStateConfig {
    /// Creates a new pipeline state config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets primitive topology.
    pub fn with_topology(mut self, topology: PrimitiveTopology) -> Self {
        self.primitive_topology = topology;
        self
    }

    /// Sets cull mode.
    pub fn with_cull_mode(mut self, cull_mode: CullMode) -> Self {
        self.rasterization.cull_mode = cull_mode;
        self
    }

    /// Sets front face winding order.
    pub fn with_front_face(mut self, front_face: FrontFace) -> Self {
        self.rasterization.front_face = front_face;
        self
    }

    /// Sets polygon mode.
    pub fn with_polygon_mode(mut self, polygon_mode: PolygonMode) -> Self {
        self.rasterization.polygon_mode = polygon_mode;
        self
    }

    /// Sets depth test configuration.
    pub fn with_depth_test(mut self, depth_test: DepthTestConfig) -> Self {
        self.depth_test = depth_test;
        self
    }

    /// Sets blend mode.
    pub fn with_blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    /// Adds a dynamic state.
    pub fn add_dynamic_state(mut self, state: DynamicState) -> Self {
        if !self.dynamic_states.contains(&state) {
            self.dynamic_states.push(state);
        }
        self
    }

    /// Computes a hash for this configuration.
    ///
    /// Used for pipeline caching - identical configurations produce identical hashes.
    pub fn compute_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        
        // Hash primitive topology
        std::mem::discriminant(&self.primitive_topology).hash(&mut hasher);
        
        // Hash rasterization config
        std::mem::discriminant(&self.rasterization.polygon_mode).hash(&mut hasher);
        std::mem::discriminant(&self.rasterization.cull_mode).hash(&mut hasher);
        std::mem::discriminant(&self.rasterization.front_face).hash(&mut hasher);
        self.rasterization.line_width.hash(&mut hasher);
        
        // Hash depth config
        self.depth_test.enable.hash(&mut hasher);
        self.depth_test.write_enable.hash(&mut hasher);
        std::mem::discriminant(&self.depth_test.compare_op).hash(&mut hasher);
        
        // Hash blend mode
        self.blend_mode.hash(&mut hasher);
        
        // Hash dynamic states
        for state in &self.dynamic_states {
            std::mem::discriminant(state).hash(&mut hasher);
        }
        
        hasher.finish()
    }
}

/// Pipeline cache for reusing pipeline state objects.
///
/// Graphics pipeline creation is expensive, so we cache pipelines based on
/// their configuration hash to avoid recreating identical pipelines.
pub struct PipelineCache {
    /// Cached pipelines indexed by configuration hash.
    pipelines: HashMap<u64, Arc<GraphicsPipeline>>,
    /// Device for pipeline creation.
    device: Arc<Device>,
}

impl PipelineCache {
    /// Creates a new pipeline cache.
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            pipelines: HashMap::new(),
            device,
        }
    }

    /// Gets or creates a pipeline with the given configuration.
    ///
    /// If a pipeline with matching configuration exists, returns the cached version.
    /// Otherwise, creates a new pipeline and caches it.
    ///
    /// # Arguments
    ///
    /// * `config` - Pipeline state configuration
    /// * `shaders` - Shader stages (vertex, fragment, etc.)
    /// * `vertex_input` - Vertex input definition
    /// * `render_pass` - Render pass compatibility
    /// * `viewport` - Initial viewport dimensions
    ///
    /// # Returns
    ///
    /// Cached or newly created pipeline.
    pub fn get_or_create<V>(
        &mut self,
        config: &PipelineStateConfig,
        shaders: Vec<PipelineShaderStageCreateInfo>,
        vertex_input: V,
        render_pass: Arc<RenderPass>,
        extent: [u32; 2],
    ) -> Result<Arc<GraphicsPipeline>>
    where
        V: VertexDefinition,
    {
        // Compute hash for this configuration
        let hash = config.compute_hash();

        // Check cache first
        if let Some(pipeline) = self.pipelines.get(&hash) {
            debug!("Using cached pipeline (hash: {})", hash);
            return Ok(pipeline.clone());
        }

        // Create new pipeline
        info!("Creating new pipeline (hash: {})", hash);
        let pipeline = self.create_pipeline(config, shaders, vertex_input, render_pass, extent)?;

        // Cache and return
        self.pipelines.insert(hash, pipeline.clone());
        Ok(pipeline)
    }

    /// Creates a new graphics pipeline with the given configuration.
    fn create_pipeline<V>(
        &self,
        config: &PipelineStateConfig,
        shaders: Vec<PipelineShaderStageCreateInfo>,
        vertex_input: V,
        render_pass: Arc<RenderPass>,
        extent: [u32; 2],
    ) -> Result<Arc<GraphicsPipeline>>
    where
        V: VertexDefinition,
    {
        // Get shader entry point for vertex input
        let vs_entry = shaders
            .iter()
            .find(|s| matches!(s.entry_point.info().execution_model, vulkano::shader::spirv::ExecutionModel::Vertex))
            .ok_or_else(|| eyre::eyre!("No vertex shader found"))?;

        // Create vertex input state
        let vertex_input_state = vertex_input
            .definition(&vs_entry.entry_point)
            .map_err(|e| eyre::eyre!("Failed to create vertex input state: {}", e))?;

        // Create pipeline layout from shader stages
        let layout = PipelineLayout::new(
            self.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&shaders)
                .into_pipeline_layout_create_info(self.device.clone())
                .map_err(|e| eyre::eyre!("Failed to create pipeline layout: {}", e))?,
        )
        .map_err(|e| eyre::eyre!("Failed to create pipeline layout: {}", e))?;

        // Get subpass
        let subpass = Subpass::from(render_pass, 0)
            .ok_or_else(|| eyre::eyre!("Failed to get subpass from render pass"))?;

        // Create depth stencil state
        let depth_stencil_state = if config.depth_test.enable {
            Some(DepthStencilState {
                depth: Some(DepthState {
                    compare_op: config.depth_test.compare_op,
                    write_enable: config.depth_test.write_enable,
                }),
                ..Default::default()
            })
        } else {
            None
        };

        // Create color blend state
        let color_blend_state = ColorBlendState::with_attachment_states(
            subpass.num_color_attachments(),
            config.blend_mode.to_attachment_state(),
        );

        // Create viewport state
        let viewport = vulkano::pipeline::graphics::viewport::Viewport {
            offset: [0.0, 0.0],
            extent: [extent[0] as f32, extent[1] as f32],
            depth_range: 0.0..=1.0,
        };

        // Create rasterization state
        let rasterization_state = RasterizationState {
            polygon_mode: config.rasterization.polygon_mode,
            cull_mode: config.rasterization.cull_mode,
            front_face: config.rasterization.front_face,
            ..Default::default()
        };

        // Create pipeline
        let create_info = GraphicsPipelineCreateInfo {
            stages: shaders.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState {
                topology: config.primitive_topology,
                ..Default::default()
            }),
            viewport_state: Some(ViewportState {
                viewports: [viewport].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(rasterization_state),
            multisample_state: Some(MultisampleState::default()),
            depth_stencil_state,
            color_blend_state: Some(color_blend_state),
            dynamic_state: config.dynamic_states.iter().cloned().collect(),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        };

        GraphicsPipeline::new(self.device.clone(), None, create_info)
            .map_err(|e| eyre::eyre!("Failed to create graphics pipeline: {}", e))
    }

    /// Clears all cached pipelines.
    pub fn clear(&mut self) {
        info!("Clearing pipeline cache ({} pipelines)", self.pipelines.len());
        self.pipelines.clear();
    }

    /// Returns the number of cached pipelines.
    pub fn len(&self) -> usize {
        self.pipelines.len()
    }

    /// Returns true if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.pipelines.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_hash() {
        let config1 = PipelineStateConfig::new()
            .with_cull_mode(CullMode::Back)
            .with_blend_mode(BlendMode::Alpha);

        let config2 = PipelineStateConfig::new()
            .with_cull_mode(CullMode::Back)
            .with_blend_mode(BlendMode::Alpha);

        let config3 = PipelineStateConfig::new()
            .with_cull_mode(CullMode::Front)
            .with_blend_mode(BlendMode::Alpha);

        // Same configuration should produce same hash
        assert_eq!(config1.compute_hash(), config2.compute_hash());

        // Different configuration should produce different hash
        assert_ne!(config1.compute_hash(), config3.compute_hash());
    }

    #[test]
    fn test_depth_test_config() {
        let default_config = DepthTestConfig::default();
        assert!(default_config.enable);
        assert!(default_config.write_enable);
        assert_eq!(default_config.compare_op, CompareOp::Less);

        let disabled_config = DepthTestConfig::disabled();
        assert!(!disabled_config.enable);
        assert!(!disabled_config.write_enable);

        let test_only_config = DepthTestConfig::test_only();
        assert!(test_only_config.enable);
        assert!(!test_only_config.write_enable);
    }

    #[test]
    fn test_blend_modes() {
        let alpha_state = BlendMode::Alpha.to_attachment_state();
        assert!(alpha_state.blend.is_some());

        let none_state = BlendMode::None.to_attachment_state();
        assert!(none_state.blend.is_none());

        let additive_state = BlendMode::Additive.to_attachment_state();
        assert!(additive_state.blend.is_some());
    }
}
