//! GPU-driven culling system for efficient rendering of large scenes.
//!
//! This module provides a GPU-based culling system that performs frustum and occlusion
//! culling using compute shaders, generating indirect draw buffers to minimize CPU overhead.
//!
//! # Overview
//!
//! Traditional CPU-based culling requires:
//! - CPU-side frustum tests for every object
//! - CPU-side occlusion queries
//! - Rebuilding draw command lists every frame on CPU
//! - High CPU-GPU synchronization overhead
//!
//! GPU-driven culling moves this work to the GPU:
//! - All culling happens in compute shaders
//! - Generates indirect draw buffers directly on GPU
//! - Minimal CPU involvement per frame
//! - Scales to tens of thousands of objects
//!
//! # Architecture
//!
//! ```text
//! CPU Side:
//!   1. Upload draw commands (model matrices, bounding spheres)
//!   2. Upload mesh metadata (index counts, offsets)
//!   3. Dispatch compute shader
//!   4. Multi-draw indirect from GPU buffers
//!
//! GPU Side (Compute Shader):
//!   1. Read draw command
//!   2. Transform bounding sphere to world space
//!   3. Test against frustum planes
//!   4. Optional: Test against depth pyramid (occlusion)
//!   5. If visible: Atomically add to indirect buffer
//! ```
//!
//! # Performance Benefits
//!
//! - **Reduced CPU Overhead**: No per-object CPU culling tests
//! - **No CPU-GPU Sync**: Draw counts stay on GPU
//! - **Efficient Multi-Draw**: Single `vkCmdDrawIndexedIndirect` call
//! - **Scales to Large Scenes**: 10,000+ objects with minimal CPU cost
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use praxis_graphics::gpu_culling::{GpuCullingManager, GpuDrawCommand};
//!
//! // Initialize culling manager
//! let mut culling_manager = GpuCullingManager::new(
//!     device.clone(),
//!     memory_allocator.clone(),
//!     descriptor_set_allocator.clone(),
//! )?;
//!
//! // Prepare draw commands with bounding spheres
//! let draw_commands: Vec<GpuDrawCommand> = objects.iter().map(|obj| {
//!     GpuDrawCommand {
//!         model: obj.transform,
//!         bounding_sphere: obj.bounding_sphere,
//!         mesh_id: obj.mesh_index,
//!         material_id: obj.material_index,
//!     }
//! }).collect();
//!
//! // Perform GPU culling
//! culling_manager.prepare_frame(&draw_commands, &mesh_data)?;
//! culling_manager.dispatch_culling(command_buffer, &view_proj, &frustum_planes)?;
//!
//! // Draw with indirect buffer
//! culling_manager.draw_indirect(command_buffer)?;
//! ```

use crate::shaders;
use praxis_math::{Mat4, Vec3, Vec4};
use praxis_utils::{debug, eyre, trace, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    descriptor_set::{
        allocator::DescriptorSetAllocator, DescriptorSet, WriteDescriptorSet,
    },
    device::Device,
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
    pipeline::{
        compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo,
        ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
};

/// A single draw command for GPU culling.
///
/// This structure contains all data needed by the GPU to perform culling
/// and generate indirect draw commands.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuDrawCommand {
    /// Model matrix transforming from model to world space.
    pub model: [[f32; 4]; 4],
    
    /// Bounding sphere in model space (xyz = center, w = radius).
    pub bounding_sphere: [f32; 4],
    
    /// Index into mesh data buffer.
    pub mesh_id: u32,
    
    /// Index into material data buffer.
    pub material_id: u32,
    
    /// Padding for alignment.
    pub padding1: u32,
    
    /// Padding for alignment.
    pub padding2: u32,
}

impl GpuDrawCommand {
    /// Creates a new GPU draw command.
    ///
    /// # Arguments
    ///
    /// * `model` - Model matrix (4x4)
    /// * `bounding_sphere` - Bounding sphere (xyz = center, w = radius)
    /// * `mesh_id` - Mesh index
    /// * `material_id` - Material index
    pub fn new(
        model: Mat4,
        bounding_sphere: Vec4,
        mesh_id: u32,
        material_id: u32,
    ) -> Self {
        Self {
            model: model.to_cols_array_2d(),
            bounding_sphere: bounding_sphere.to_array(),
            mesh_id,
            material_id,
            padding1: 0,
            padding2: 0,
        }
    }
}

/// Mesh metadata for GPU culling.
///
/// Contains the data needed to construct indirect draw commands.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuMeshData {
    /// Number of indices in this mesh.
    pub index_count: u32,
    
    /// First index in the index buffer.
    pub first_index: u32,
    
    /// Vertex offset for this mesh.
    pub vertex_offset: i32,
    
    /// Padding for alignment.
    pub _padding: u32,
}

/// Indirect draw command structure (matches VkDrawIndexedIndirectCommand).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct IndirectDrawCommand {
    /// Number of indices to draw.
    pub index_count: u32,
    
    /// Number of instances to draw.
    pub instance_count: u32,
    
    /// First index in the index buffer.
    pub first_index: u32,
    
    /// Offset added to vertex index.
    pub vertex_offset: i32,
    
    /// First instance ID.
    pub first_instance: u32,
}

/// Culling uniforms passed to the compute shader.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CullingUniforms {
    /// View-projection matrix.
    pub view_proj: [[f32; 4]; 4],
    
    /// Frustum planes in world space [left, right, bottom, top, near, far].
    /// Each plane is (nx, ny, nz, d) where n is the normal and d is the distance.
    pub frustum_planes: [[f32; 4]; 6],
    
    /// Camera position in world space.
    pub camera_position: [f32; 3],
    pub _padding1: f32,
    
    /// Enable frustum culling (0 = disabled, 1 = enabled).
    pub enable_frustum_culling: u32,
    
    /// Enable occlusion culling (0 = disabled, 1 = enabled).
    pub enable_occlusion_culling: u32,
    
    /// Number of draw commands to process.
    pub draw_command_count: u32,
    
    pub _padding2: u32,
}

impl CullingUniforms {
    /// Creates culling uniforms with frustum culling enabled.
    pub fn new(
        view_proj: Mat4,
        frustum_planes: [Vec4; 6],
        camera_position: Vec3,
        draw_command_count: u32,
    ) -> Self {
        Self {
            view_proj: view_proj.to_cols_array_2d(),
            frustum_planes: [
                frustum_planes[0].to_array(),
                frustum_planes[1].to_array(),
                frustum_planes[2].to_array(),
                frustum_planes[3].to_array(),
                frustum_planes[4].to_array(),
                frustum_planes[5].to_array(),
            ],
            camera_position: camera_position.to_array(),
            _padding1: 0.0,
            enable_frustum_culling: 1,
            enable_occlusion_culling: 0,
            draw_command_count,
            _padding2: 0,
        }
    }
}

/// GPU-driven culling manager.
///
/// Manages compute shader dispatch for frustum and occlusion culling,
/// generating indirect draw buffers on the GPU.
pub struct GpuCullingManager {
    #[allow(dead_code)]
    device: Arc<Device>,
    memory_allocator: Arc<dyn MemoryAllocator>,
    descriptor_set_allocator: Arc<dyn DescriptorSetAllocator>,
    
    compute_pipeline: Arc<ComputePipeline>,
    
    // Buffers
    draw_command_buffer: Option<Subbuffer<[GpuDrawCommand]>>,
    mesh_data_buffer: Option<Subbuffer<[GpuMeshData]>>,
    indirect_draw_buffer: Option<Subbuffer<[IndirectDrawCommand]>>,
    visible_indices_buffer: Option<Subbuffer<[u32]>>,
    draw_count_buffer: Option<Subbuffer<u32>>,
    culling_uniforms_buffer: Option<Subbuffer<CullingUniforms>>,
    
    descriptor_set: Option<Arc<DescriptorSet>>,
    
    max_draw_commands: usize,
    current_draw_count: u32,
}

impl GpuCullingManager {
    /// Creates a new GPU culling manager.
    ///
    /// # Arguments
    ///
    /// * `device` - Vulkan device
    /// * `memory_allocator` - Memory allocator for buffers
    /// * `descriptor_set_allocator` - Descriptor set allocator
    ///
    /// # Errors
    ///
    /// Returns an error if pipeline or buffer creation fails.
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<dyn MemoryAllocator>,
        descriptor_set_allocator: Arc<dyn DescriptorSetAllocator>,
    ) -> Result<Self> {
        debug!("Creating GPU culling manager");
        
        // Create compute pipeline
        let compute_pipeline = Self::create_compute_pipeline(device.clone())?;
        
        Ok(Self {
            device,
            memory_allocator,
            descriptor_set_allocator,
            compute_pipeline,
            draw_command_buffer: None,
            mesh_data_buffer: None,
            indirect_draw_buffer: None,
            visible_indices_buffer: None,
            draw_count_buffer: None,
            culling_uniforms_buffer: None,
            descriptor_set: None,
            max_draw_commands: 0,
            current_draw_count: 0,
        })
    }
    
    /// Creates the GPU culling compute pipeline.
    fn create_compute_pipeline(device: Arc<Device>) -> Result<Arc<ComputePipeline>> {
        trace!("Loading GPU culling compute shader");
        
        let shader = shaders::load_gpu_culling_comp(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load GPU culling shader: {}", e))?;
        
        let stage = PipelineShaderStageCreateInfo::new(shader.entry_point("main").unwrap());
        
        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&[stage.clone()])
                .into_pipeline_layout_create_info(device.clone())
                .map_err(|e| eyre::eyre!("Failed to create pipeline layout info: {}", e))?,
        )
        .map_err(|e| eyre::eyre!("Failed to create pipeline layout: {}", e))?;
        
        ComputePipeline::new(
            device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(stage, layout),
        )
        .map_err(|e| eyre::eyre!("Failed to create compute pipeline: {}", e))
    }
    
    /// Prepares buffers for a new frame.
    ///
    /// This should be called once per frame with the current draw commands and mesh data.
    /// It allocates or resizes buffers as needed and uploads the data.
    ///
    /// # Arguments
    ///
    /// * `draw_commands` - Draw commands to process
    /// * `mesh_data` - Mesh metadata for all meshes
    ///
    /// # Errors
    ///
    /// Returns an error if buffer allocation or upload fails.
    pub fn prepare_frame(
        &mut self,
        draw_commands: &[GpuDrawCommand],
        mesh_data: &[GpuMeshData],
    ) -> Result<()> {
        let draw_count = draw_commands.len();
        self.current_draw_count = draw_count as u32;
        
        trace!(
            "Preparing GPU culling frame: {} draw commands, {} meshes",
            draw_count,
            mesh_data.len()
        );
        
        // Reallocate buffers if needed
        if draw_count > self.max_draw_commands {
            debug!(
                "Reallocating GPU culling buffers: {} -> {} draw commands",
                self.max_draw_commands, draw_count
            );
            self.allocate_buffers(draw_count)?;
        }
        
        // Upload draw commands
        if let Some(buffer) = &self.draw_command_buffer {
            let mut write = buffer
                .write()
                .map_err(|e| eyre::eyre!("Failed to map draw command buffer: {}", e))?;
            write[..draw_count].copy_from_slice(draw_commands);
        }
        
        // Upload mesh data
        if let Some(buffer) = &self.mesh_data_buffer {
            let mut write = buffer
                .write()
                .map_err(|e| eyre::eyre!("Failed to map mesh data buffer: {}", e))?;
            write[..mesh_data.len()].copy_from_slice(mesh_data);
        }
        
        // Reset draw count to zero
        if let Some(buffer) = &self.draw_count_buffer {
            let mut write = buffer
                .write()
                .map_err(|e| eyre::eyre!("Failed to map draw count buffer: {}", e))?;
            *write = 0;
        }
        
        Ok(())
    }
    
    /// Allocates GPU buffers for culling.
    fn allocate_buffers(&mut self, max_draw_commands: usize) -> Result<()> {
        debug!("Allocating GPU culling buffers for {} draw commands", max_draw_commands);
        
        // Draw command buffer (input)
        let draw_command_buffer = Buffer::new_slice::<GpuDrawCommand>(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            max_draw_commands as u64,
        )
        .map_err(|e| eyre::eyre!("Failed to create draw command buffer: {}", e))?;
        
        // Mesh data buffer (input)
        let mesh_data_buffer = Buffer::new_slice::<GpuMeshData>(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            max_draw_commands as u64,
        )
        .map_err(|e| eyre::eyre!("Failed to create mesh data buffer: {}", e))?;
        
        // Indirect draw buffer (output)
        let indirect_draw_buffer = Buffer::new_slice::<IndirectDrawCommand>(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER | BufferUsage::INDIRECT_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            max_draw_commands as u64,
        )
        .map_err(|e| eyre::eyre!("Failed to create indirect draw buffer: {}", e))?;
        
        // Visible indices buffer (output)
        let visible_indices_buffer = Buffer::new_slice::<u32>(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            max_draw_commands as u64,
        )
        .map_err(|e| eyre::eyre!("Failed to create visible indices buffer: {}", e))?;
        
        // Draw count buffer (output, atomic counter)
        let draw_count_buffer = Buffer::from_data(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER | BufferUsage::INDIRECT_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            0u32,
        )
        .map_err(|e| eyre::eyre!("Failed to create draw count buffer: {}", e))?;
        
        // Culling uniforms buffer
        let culling_uniforms_buffer = Buffer::from_data(
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
            CullingUniforms {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
                frustum_planes: [[0.0; 4]; 6],
                camera_position: [0.0; 3],
                _padding1: 0.0,
                enable_frustum_culling: 1,
                enable_occlusion_culling: 0,
                draw_command_count: 0,
                _padding2: 0,
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create culling uniforms buffer: {}", e))?;
        
        self.draw_command_buffer = Some(draw_command_buffer);
        self.mesh_data_buffer = Some(mesh_data_buffer);
        self.indirect_draw_buffer = Some(indirect_draw_buffer);
        self.visible_indices_buffer = Some(visible_indices_buffer);
        self.draw_count_buffer = Some(draw_count_buffer);
        self.culling_uniforms_buffer = Some(culling_uniforms_buffer);
        self.max_draw_commands = max_draw_commands;
        
        // Descriptor set will be recreated on next dispatch
        self.descriptor_set = None;
        
        Ok(())
    }
    
    /// Dispatches the GPU culling compute shader.
    ///
    /// This records commands into the provided command buffer to:
    /// 1. Bind the culling compute pipeline
    /// 2. Update culling uniforms
    /// 3. Dispatch compute work groups
    ///
    /// # Arguments
    ///
    /// * `builder` - Command buffer builder to record into
    /// * `view_proj` - View-projection matrix
    /// * `frustum_planes` - Six frustum planes [left, right, bottom, top, near, far]
    /// * `camera_position` - Camera position in world space
    ///
    /// # Errors
    ///
    /// Returns an error if command recording fails.
    pub fn dispatch_culling(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        view_proj: Mat4,
        frustum_planes: [Vec4; 6],
        camera_position: Vec3,
    ) -> Result<()> {
        if self.current_draw_count == 0 {
            return Ok(());
        }
        
        trace!("Dispatching GPU culling for {} draw commands", self.current_draw_count);
        
        // Update culling uniforms
        let uniforms = CullingUniforms::new(
            view_proj,
            frustum_planes,
            camera_position,
            self.current_draw_count,
        );
        
        if let Some(buffer) = &self.culling_uniforms_buffer {
            let mut write = buffer
                .write()
                .map_err(|e| eyre::eyre!("Failed to map culling uniforms buffer: {}", e))?;
            *write = uniforms;
        }
        
        // Create or get descriptor set
        if self.descriptor_set.is_none() {
            self.create_descriptor_set()?;
        }
        
        let descriptor_set = self.descriptor_set.as_ref().unwrap();
        
        // Bind pipeline and descriptor set
        builder
            .bind_pipeline_compute(self.compute_pipeline.clone())
            .map_err(|e| eyre::eyre!("Failed to bind compute pipeline: {}", e))?
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.compute_pipeline.layout().clone(),
                0,
                descriptor_set.clone(),
            )
            .map_err(|e| eyre::eyre!("Failed to bind descriptor sets: {}", e))?;
        
        // Dispatch compute work groups (64 threads per group)
        let work_group_count = self.current_draw_count.div_ceil(64);
        
        unsafe {
            builder
                .dispatch([work_group_count, 1, 1])
                .map_err(|e| eyre::eyre!("Failed to dispatch compute: {}", e))?;
        }
        
        trace!("Dispatched {} compute work groups", work_group_count);
        
        Ok(())
    }
    
    /// Creates the descriptor set for the culling compute shader.
    fn create_descriptor_set(&mut self) -> Result<()> {
        trace!("Creating GPU culling descriptor set");
        
        let layout = self.compute_pipeline.layout().set_layouts().first()
            .ok_or_else(|| eyre::eyre!("No descriptor set layout in pipeline"))?;
        
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout.clone(),
            [
                WriteDescriptorSet::buffer(
                    0,
                    self.culling_uniforms_buffer.clone().unwrap(),
                ),
                WriteDescriptorSet::buffer(
                    1,
                    self.draw_command_buffer.clone().unwrap(),
                ),
                WriteDescriptorSet::buffer(
                    2,
                    self.mesh_data_buffer.clone().unwrap(),
                ),
                WriteDescriptorSet::buffer(
                    3,
                    self.indirect_draw_buffer.clone().unwrap(),
                ),
                WriteDescriptorSet::buffer(
                    4,
                    self.visible_indices_buffer.clone().unwrap(),
                ),
                WriteDescriptorSet::buffer(
                    5,
                    self.draw_count_buffer.clone().unwrap(),
                ),
            ],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create descriptor set: {}", e))?;
        
        self.descriptor_set = Some(descriptor_set);
        
        Ok(())
    }
    
    /// Gets the indirect draw buffer for rendering.
    ///
    /// This buffer can be used with `vkCmdDrawIndexedIndirect` to render
    /// all visible objects in a single draw call.
    pub fn indirect_draw_buffer(&self) -> Option<&Subbuffer<[IndirectDrawCommand]>> {
        self.indirect_draw_buffer.as_ref()
    }
    
    /// Gets the visible indices buffer.
    ///
    /// This buffer contains the original indices of visible draw commands,
    /// useful for looking up per-object data like transforms and materials.
    pub fn visible_indices_buffer(&self) -> Option<&Subbuffer<[u32]>> {
        self.visible_indices_buffer.as_ref()
    }
    
    /// Gets the draw count buffer.
    ///
    /// This buffer contains the number of visible draw commands after culling.
    /// Can be used for indirect draw count with `vkCmdDrawIndexedIndirectCount`.
    pub fn draw_count_buffer(&self) -> Option<&Subbuffer<u32>> {
        self.draw_count_buffer.as_ref()
    }
    
    /// Reads back the number of visible objects after culling.
    ///
    /// This requires a CPU-GPU sync and should only be used for debugging
    /// or statistics gathering, not during normal rendering.
    ///
    /// # Errors
    ///
    /// Returns an error if buffer mapping fails.
    pub fn read_visible_count(&self) -> Result<u32> {
        if let Some(buffer) = &self.draw_count_buffer {
            let read = buffer
                .read()
                .map_err(|e| eyre::eyre!("Failed to read draw count buffer: {}", e))?;
            Ok(*read)
        } else {
            Ok(0)
        }
    }
}

/// Extracts frustum planes from a view-projection matrix.
///
/// Returns the six frustum planes in world space as [left, right, bottom, top, near, far].
/// Each plane is represented as (nx, ny, nz, d) where n is the outward-facing normal
/// and d is the distance from origin.
///
/// # Arguments
///
/// * `view_proj` - Combined view-projection matrix
///
/// # Returns
///
/// Array of 6 planes: [left, right, bottom, top, near, far]
pub fn extract_frustum_planes(view_proj: Mat4) -> [Vec4; 6] {
    let m = view_proj;
    
    // Extract rows
    let row0 = Vec4::new(m.x_axis.x, m.y_axis.x, m.z_axis.x, m.w_axis.x);
    let row1 = Vec4::new(m.x_axis.y, m.y_axis.y, m.z_axis.y, m.w_axis.y);
    let row2 = Vec4::new(m.x_axis.z, m.y_axis.z, m.z_axis.z, m.w_axis.z);
    let row3 = Vec4::new(m.x_axis.w, m.y_axis.w, m.z_axis.w, m.w_axis.w);
    
    // Extract and normalize planes
    let left = (row3 + row0).normalize();
    let right = (row3 - row0).normalize();
    let bottom = (row3 + row1).normalize();
    let top = (row3 - row1).normalize();
    let near = (row3 + row2).normalize();
    let far = (row3 - row2).normalize();
    
    [left, right, bottom, top, near, far]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gpu_draw_command_size() {
        // Should be 96 bytes for optimal GPU alignment
        assert_eq!(std::mem::size_of::<GpuDrawCommand>(), 96);
    }
    
    #[test]
    fn test_gpu_mesh_data_size() {
        // Should be 16 bytes
        assert_eq!(std::mem::size_of::<GpuMeshData>(), 16);
    }
    
    #[test]
    fn test_indirect_draw_command_size() {
        // Should match VkDrawIndexedIndirectCommand (20 bytes)
        assert_eq!(std::mem::size_of::<IndirectDrawCommand>(), 20);
    }
    
    #[test]
    fn test_culling_uniforms_alignment() {
        // Should be 16-byte aligned for std140
        assert_eq!(std::mem::align_of::<CullingUniforms>(), 16);
    }
}
