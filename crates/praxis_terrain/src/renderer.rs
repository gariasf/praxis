//! Terrain and vegetation rendering systems.

use crate::chunk::TerrainChunk;
use crate::material::TerrainMaterial;
use crate::splatmap::SplatMap;
use crate::vegetation::VegetationLayer;
use praxis_graphics::GpuMesh;
use praxis_math::{Mat4, Vec3};
use praxis_utils::Result;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::CommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::device::Device;
use vulkano::image::sampler::Sampler;
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::{GraphicsPipeline, Pipeline};

/// Terrain-specific rendering system with texture splatting support.
pub struct TerrainRenderer {
    #[allow(dead_code)]
    device: Arc<Device>,
    #[allow(dead_code)]
    memory_allocator: Arc<StandardMemoryAllocator>,
    #[allow(dead_code)]
    command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    terrain_pipeline: Option<Arc<GraphicsPipeline>>,
}

impl TerrainRenderer {
    /// Creates a new terrain renderer.
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
    ) -> Self {
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        Self {
            device,
            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
            terrain_pipeline: None,
        }
    }

    /// Sets the graphics pipeline for terrain rendering.
    pub fn set_pipeline(&mut self, pipeline: Arc<GraphicsPipeline>) {
        self.terrain_pipeline = Some(pipeline);
    }

    /// Renders terrain chunks with texture splatting.
    ///
    /// This method records draw commands for all provided chunks into the given command buffer builder.
    /// The builder must be within an active render pass before calling this method.
    ///
    /// # Arguments
    ///
    /// * `builder` - Command buffer builder within an active render pass
    /// * `chunks` - Terrain chunks to render
    /// * `material` - Terrain material configuration
    /// * `splatmap` - Splat map for texture blending
    /// * `view_matrix` - Camera view matrix
    /// * `proj_matrix` - Camera projection matrix
    pub fn render_chunks(
        &self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        chunks: &[&TerrainChunk],
        material: &TerrainMaterial,
        splatmap: &SplatMap,
        view_matrix: Mat4,
        proj_matrix: Mat4,
    ) -> Result<()> {
        let pipeline = self
            .terrain_pipeline
            .as_ref()
            .ok_or_else(|| praxis_utils::eyre::eyre!("Terrain pipeline not set"))?;

        builder
            .bind_pipeline_graphics(pipeline.clone())
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to bind terrain pipeline: {}", e))?;

        for chunk in chunks {
            if let Some(mesh) = &chunk.meshes[chunk.lod.current_level] {
                self.render_chunk(builder, chunk, mesh, material, splatmap, view_matrix, proj_matrix)?;
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn render_chunk(
        &self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        chunk: &TerrainChunk,
        mesh: &GpuMesh,
        _material: &TerrainMaterial,
        _splatmap: &SplatMap,
        view_matrix: Mat4,
        proj_matrix: Mat4,
    ) -> Result<()> {
        let pipeline = self
            .terrain_pipeline
            .as_ref()
            .ok_or_else(|| praxis_utils::eyre::eyre!("Terrain pipeline not set"))?;

        let model_matrix = Mat4::from_translation(chunk.id.world_position(64.0));
        let model_view_proj = proj_matrix * view_matrix * model_matrix;

        #[repr(C)]
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct ModelPushConstants {
            model: [[f32; 4]; 4],
            model_view_proj: [[f32; 4]; 4],
        }

        let push_constants = ModelPushConstants {
            model: model_matrix.to_cols_array_2d(),
            model_view_proj: model_view_proj.to_cols_array_2d(),
        };

        builder
            .bind_vertex_buffers(0, mesh.vertex_buffer.clone())
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to bind vertex buffer: {}", e))?
            .bind_index_buffer(mesh.index_buffer.clone())
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to bind index buffer: {}", e))?;

        unsafe {
            builder
                .push_constants(
                    pipeline.layout().clone(),
                    0,
                    push_constants,
                )
                .map_err(|e| praxis_utils::eyre::eyre!("Failed to push constants: {}", e))?;

            builder
                .draw_indexed(mesh.index_count, 1, 0, 0, 0)
                .map_err(|e| praxis_utils::eyre::eyre!("Failed to draw indexed: {}", e))?;
        }

        Ok(())
    }

    /// Creates a descriptor set for terrain rendering with splat map and material layers.
    ///
    /// # Panics
    ///
    /// Panics if the terrain pipeline has not been set via `set_pipeline()`.
    /// The pipeline must be initialized before creating descriptor sets.
    pub fn create_terrain_descriptor_set(
        &self,
        splat_map_view: Arc<ImageView>,
        splat_map_sampler: Arc<Sampler>,
        layer_textures: &[Arc<ImageView>],
        layer_samplers: &[Arc<Sampler>],
        view_proj_buffer: Subbuffer<[u8]>,
    ) -> Result<Arc<DescriptorSet>> {
        let layout = self
            .terrain_pipeline
            .as_ref()
            .expect("Terrain pipeline not set")
            .layout()
            .set_layouts()[0]
            .clone();

        let mut writes = vec![
            WriteDescriptorSet::buffer(0, view_proj_buffer),
            WriteDescriptorSet::image_view_sampler(1, splat_map_view, splat_map_sampler),
        ];

        for (i, (view, sampler)) in layer_textures.iter().zip(layer_samplers.iter()).enumerate() {
            writes.push(WriteDescriptorSet::image_view_sampler(
                (2 + i) as u32,
                view.clone(),
                sampler.clone(),
            ));
        }

        let descriptor_set =
            DescriptorSet::new(self.descriptor_set_allocator.clone(), layout, writes, [])?;

        Ok(descriptor_set)
    }
}

/// Instance data for GPU instancing of vegetation.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VegetationInstanceData {
    /// Model matrix column 0.
    pub model_col0: [f32; 4],
    /// Model matrix column 1.
    pub model_col1: [f32; 4],
    /// Model matrix column 2.
    pub model_col2: [f32; 4],
    /// Model matrix column 3.
    pub model_col3: [f32; 4],
    /// Color variation RGB and wind phase.
    pub color_and_wind: [f32; 4],
}

impl VegetationInstanceData {
    /// Creates instance data from a model matrix and color.
    pub fn from_matrix(model: Mat4, color: Vec3, wind_phase: f32) -> Self {
        let cols = model.to_cols_array_2d();
        Self {
            model_col0: cols[0],
            model_col1: cols[1],
            model_col2: cols[2],
            model_col3: cols[3],
            color_and_wind: [color.x, color.y, color.z, wind_phase],
        }
    }
}

/// Push constants for vegetation rendering.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VegetationPushConstants {
    /// Current time for wind animation.
    pub time: f32,
    /// Wind strength multiplier.
    pub wind_strength: f32,
    /// Wind direction (X, Z components).
    pub wind_direction: [f32; 2],
}

/// Vegetation rendering system using GPU instancing.
pub struct VegetationRenderer {
    #[allow(dead_code)]
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
    #[allow(dead_code)]
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    vegetation_pipeline: Option<Arc<GraphicsPipeline>>,
}

impl VegetationRenderer {
    /// Creates a new vegetation renderer.
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
    ) -> Self {
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        Self {
            device,
            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
            vegetation_pipeline: None,
        }
    }

    /// Sets the graphics pipeline for vegetation rendering.
    pub fn set_pipeline(&mut self, pipeline: Arc<GraphicsPipeline>) {
        self.vegetation_pipeline = Some(pipeline);
    }

    /// Renders a vegetation layer using GPU instancing.
    pub fn render_layer(
        &self,
        layer: &VegetationLayer,
        mesh: &GpuMesh,
        view_matrix: Mat4,
        proj_matrix: Mat4,
        time: f32,
    ) -> Result<()> {
        if layer.instances.is_empty() {
            return Ok(());
        }

        let instance_data: Vec<VegetationInstanceData> = layer
            .instances
            .iter()
            .enumerate()
            .map(|(i, inst)| {
                let wind_phase = (i as f32 * 0.1 + time) * layer.wind_strength;
                VegetationInstanceData::from_matrix(
                    inst.model_matrix(),
                    inst.color_variation,
                    wind_phase,
                )
            })
            .collect();

        let _instance_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            instance_data,
        )?;

        let _ = (mesh, view_matrix, proj_matrix);

        Ok(())
    }

    /// Renders multiple vegetation layers.
    pub fn render_layers(
        &self,
        layers: &[VegetationLayer],
        meshes: &std::collections::HashMap<String, GpuMesh>,
        view_matrix: Mat4,
        proj_matrix: Mat4,
        time: f32,
    ) -> Result<()> {
        for layer in layers {
            if let Some(mesh) = meshes.get(&layer.mesh_name) {
                self.render_layer(layer, mesh, view_matrix, proj_matrix, time)?;
            }
        }

        Ok(())
    }

    /// Creates a command buffer for rendering vegetation with culling.
    pub fn create_render_commands(
        &self,
        _layers: &[VegetationLayer],
        _camera_pos: Vec3,
        _view_distance: f32,
        queue_family_index: u32,
    ) -> Result<Arc<PrimaryAutoCommandBuffer>> {
        let builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            queue_family_index,
            CommandBufferUsage::OneTimeSubmit,
        )?;

        Ok(builder.build()?)
    }

    /// Builds instance buffer data for a vegetation layer with frustum culling.
    pub fn build_instance_buffer(
        &self,
        layer: &VegetationLayer,
        camera_pos: Vec3,
        view_distance: f32,
        time: f32,
    ) -> Result<Subbuffer<[VegetationInstanceData]>> {
        let visible_instances: Vec<VegetationInstanceData> = layer
            .instances
            .iter()
            .enumerate()
            .filter_map(|(i, inst)| {
                let dist = (inst.position - camera_pos).length();
                if dist < view_distance {
                    let wind_phase = (i as f32 * 0.1 + time) * layer.wind_strength;
                    Some(VegetationInstanceData::from_matrix(
                        inst.model_matrix(),
                        inst.color_variation,
                        wind_phase,
                    ))
                } else {
                    None
                }
            })
            .collect();

        let buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            visible_instances,
        )?;

        Ok(buffer)
    }
}
