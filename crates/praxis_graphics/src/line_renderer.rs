//! Line rendering system for debug visualization and editor gizmos.
//!
//! This module provides efficient batched rendering of colored lines in 3D space,
//! useful for debug visualization, gizmo rendering, grid overlays, and selection boxes.

use crate::uniform_buffer::ViewProjectionUniforms;
use praxis_math::{Mat4, Vec3};
use praxis_utils::{eyre, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, DescriptorSet, WriteDescriptorSet,
    },
    device::Device,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{DepthState, DepthStencilState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{CullMode, RasterizationState},
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        GraphicsPipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{RenderPass, Subpass},
};

/// Vertex format for line rendering.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, Vertex)]
pub struct LineVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
}

impl LineVertex {
    /// Creates a new line vertex.
    pub fn new(position: Vec3, color: Vec3) -> Self {
        Self {
            position: position.to_array(),
            color: color.to_array(),
        }
    }
}

/// A line segment defined by two vertices.
#[derive(Clone, Copy, Debug)]
pub struct Line {
    pub start: Vec3,
    pub end: Vec3,
    pub color: Vec3,
}

impl Line {
    /// Creates a new line segment.
    pub fn new(start: Vec3, end: Vec3, color: Vec3) -> Self {
        Self { start, end, color }
    }

    /// Converts the line to two vertices.
    pub fn to_vertices(&self) -> [LineVertex; 2] {
        [
            LineVertex::new(self.start, self.color),
            LineVertex::new(self.end, self.color),
        ]
    }
}

/// Batch of lines to be rendered together.
#[derive(Clone, Debug)]
pub struct LineBatch {
    lines: Vec<Line>,
}

impl LineBatch {
    /// Creates a new empty line batch.
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    /// Creates a line batch with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            lines: Vec::with_capacity(capacity),
        }
    }

    /// Adds a line to the batch.
    pub fn add_line(&mut self, line: Line) {
        self.lines.push(line);
    }

    /// Adds a line segment to the batch.
    pub fn add(&mut self, start: Vec3, end: Vec3, color: Vec3) {
        self.lines.push(Line::new(start, end, color));
    }

    /// Adds multiple lines to the batch.
    pub fn add_lines(&mut self, lines: impl IntoIterator<Item = Line>) {
        self.lines.extend(lines);
    }

    /// Clears all lines from the batch.
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// Returns the number of lines in the batch.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Returns true if the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Converts all lines to vertices.
    pub fn to_vertices(&self) -> Vec<LineVertex> {
        self.lines
            .iter()
            .flat_map(|line| line.to_vertices())
            .collect()
    }
}

impl Default for LineBatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Line renderer for drawing colored lines in 3D space.
pub struct LineRenderer {
    pipeline: Arc<GraphicsPipeline>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    view_proj_buffer: Subbuffer<ViewProjectionUniforms>,
}

impl LineRenderer {
    /// Creates a new line renderer.
    pub fn new(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        extent: [u32; 2],
    ) -> Result<Self> {
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let pipeline = Self::create_pipeline(device.clone(), render_pass, extent)?;

        let initial_view_proj = ViewProjectionUniforms {
            view: Mat4::IDENTITY.to_cols_array_2d(),
            proj: Mat4::IDENTITY.to_cols_array_2d(),
            camera_position: [0.0, 0.0, 0.0],
            _padding: 0.0,
        };

        let view_proj_buffer = Buffer::from_data(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            initial_view_proj,
        )
        .map_err(|e| eyre::eyre!("Failed to create view projection buffer: {}", e))?;

        Ok(Self {
            pipeline,
            descriptor_set_allocator,
            memory_allocator,
            view_proj_buffer,
        })
    }

    /// Creates the line rendering pipeline.
    fn create_pipeline(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        extent: [u32; 2],
    ) -> Result<Arc<GraphicsPipeline>> {
        mod vs {
            vulkano_shaders::shader! {
                ty: "vertex",
                path: "src/shaders/line.vert"
            }
        }

        mod fs {
            vulkano_shaders::shader! {
                ty: "fragment",
                path: "src/shaders/line.frag"
            }
        }

        let vs = vs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load line vertex shader: {}", e))?;
        let fs = fs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load line fragment shader: {}", e))?;

        let vertex_input_state = LineVertex::per_vertex()
            .definition(&vs.entry_point("main").unwrap())
            .map_err(|e| eyre::eyre!("Failed to create vertex input state: {}", e))?;

        let stages = [
            PipelineShaderStageCreateInfo::new(vs.entry_point("main").unwrap()),
            PipelineShaderStageCreateInfo::new(fs.entry_point("main").unwrap()),
        ];

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .map_err(|e| eyre::eyre!("Failed to create pipeline layout info: {}", e))?,
        )
        .map_err(|e| eyre::eyre!("Failed to create pipeline layout: {}", e))?;

        let subpass = Subpass::from(render_pass.clone(), 0)
            .ok_or_else(|| eyre::eyre!("Failed to create subpass"))?;

        let pipeline = GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState {
                    topology: PrimitiveTopology::LineList,
                    ..Default::default()
                }),
                viewport_state: Some(ViewportState {
                    viewports: [Viewport {
                        offset: [0.0, 0.0],
                        extent: [extent[0] as f32, extent[1] as f32],
                        depth_range: 0.0..=1.0,
                    }]
                    .into_iter()
                    .collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState {
                    cull_mode: CullMode::None,
                    ..Default::default()
                }),
                depth_stencil_state: Some(DepthStencilState {
                    depth: Some(DepthState::simple()),
                    ..Default::default()
                }),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create line graphics pipeline: {}", e))?;

        Ok(pipeline)
    }

    /// Updates the view and projection matrices.
    pub fn update_view_projection(
        &mut self,
        view: Mat4,
        proj: Mat4,
        camera_position: Vec3,
    ) -> Result<()> {
        let uniforms = ViewProjectionUniforms {
            view: view.to_cols_array_2d(),
            proj: proj.to_cols_array_2d(),
            camera_position: camera_position.to_array(),
            _padding: 0.0,
        };

        let mut write_lock = self
            .view_proj_buffer
            .write()
            .map_err(|e| eyre::eyre!("Failed to lock view projection buffer: {}", e))?;
        *write_lock = uniforms;

        Ok(())
    }

    /// Renders a batch of lines.
    pub fn render(
        &self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        batch: &LineBatch,
    ) -> Result<()> {
        if batch.is_empty() {
            return Ok(());
        }

        let vertices = batch.to_vertices();
        let vertex_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )
        .map_err(|e| eyre::eyre!("Failed to create vertex buffer: {}", e))?;

        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            self.pipeline.layout().set_layouts()[0].clone(),
            [WriteDescriptorSet::buffer(0, self.view_proj_buffer.clone())],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create descriptor set: {}", e))?;

        builder
            .bind_pipeline_graphics(self.pipeline.clone())
            .map_err(|e| eyre::eyre!("Failed to bind pipeline: {}", e))?
            .bind_descriptor_sets(
                vulkano::pipeline::PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .map_err(|e| eyre::eyre!("Failed to bind descriptor sets: {}", e))?
            .bind_vertex_buffers(0, vertex_buffer)
            .map_err(|e| eyre::eyre!("Failed to bind vertex buffers: {}", e))?;

        unsafe {
            builder
                .draw(batch.len() as u32 * 2, 1, 0, 0)
                .map_err(|e| eyre::eyre!("Failed to draw lines: {}", e))?;
        }

        Ok(())
    }
}
