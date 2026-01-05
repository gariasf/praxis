//! GPU-driven culling system using compute shaders.
//!
//! This module provides a comprehensive GPU culling system that offloads frustum culling
#![allow(unsafe_code)]
#![allow(dead_code)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::suboptimal_flops)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::manual_div_ceil)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::needless_pass_by_ref_mut)]

//!
//! and LOD selection from CPU to GPU using compute shaders. This is essential for scenes
//! with 10,000+ objects where CPU culling becomes a bottleneck.
//!
//! # Architecture
//!
//! The GPU culling pipeline works as follows:
//!
//! 1. **Upload Phase**: Object data (AABB, position, LOD group) is uploaded to GPU buffers
//! 2. **Compute Phase**: Compute shader processes all objects in parallel:
//!    - Frustum culling: Test AABB against frustum planes
//!    - Distance culling: Test distance from camera
//!    - LOD selection: Choose appropriate mesh based on distance
//! 3. **Result Phase**: Visible objects with selected LOD levels are written to result buffer
//! 4. **Readback Phase**: Application reads back visible object list for rendering
//!
//! # Performance Benefits
//!
//! GPU culling provides significant performance improvements:
//! - Processes thousands of objects in parallel on GPU
//! - Eliminates CPU bottleneck for large scenes
//! - Reduces CPU-GPU synchronization overhead
//! - Enables multi-draw indirect rendering (future extension)
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use praxis_graphics::gpu_culling::{GpuCullingManager, GpuCullingConfig};
//! use praxis_math::Vec3;
//!
//! # async fn example() -> praxis_utils::Result<()> {
//! // Create GPU culling manager
//! let config = GpuCullingConfig {
//!     max_objects: 20000,
//!     max_lod_groups: 1024,
//!     enable_lod_selection: true,
//!     enable_distance_culling: true,
//!     max_distance: 500.0,
//! };
//!
//! // let mut culling_manager = GpuCullingManager::new(
//! //     device,
//! //     allocator,
//! //     command_allocator,
//! //     queue,
//! //     config,
//! // )?;
//!
//! // Each frame:
//! // 1. Update object data
//! // culling_manager.update_objects(&objects)?;
//!
//! // 2. Run culling compute shader
//! // let visible_objects = culling_manager.cull(camera_view_proj, camera_position)?;
//!
//! // 3. Render visible objects
//! // for result in visible_objects {
//! //     render_mesh(result.mesh_id, objects[result.object_index].transform);
//! // }
//! # Ok(())
//! # }
//! ```

use crate::Aabb;
use praxis_math::{Mat4, Vec3};
use praxis_utils::{debug, eyre, info, trace, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::CommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, DescriptorSet, WriteDescriptorSet,
    },
    device::{Device, Queue},
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
    pipeline::{
        compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo,
        ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    sync::GpuFuture,
};

/// Maximum number of LOD levels per LOD group.
pub const MAX_LOD_LEVELS_PER_GROUP: usize = 8;

/// Maximum number of LOD groups supported.
pub const MAX_LOD_GROUPS: usize = 1024;

/// GPU representation of object data for culling.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuObjectData {
    /// Minimum corner of axis-aligned bounding box.
    pub aabb_min: [f32; 4],
    /// Maximum corner of axis-aligned bounding box.
    pub aabb_max: [f32; 4],
    /// World-space position of object.
    pub position: [f32; 4],
    /// Mesh identifier.
    pub mesh_id: u32,
    /// LOD group identifier (`u32::MAX` if no LOD).
    pub lod_group_id: u32,
    /// Bounding sphere radius.
    pub bounding_radius: f32,
    /// Padding for alignment.
    pub padding: u32,
}

/// GPU representation of a single LOD level.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuLodLevel {
    /// Mesh identifier for this LOD level.
    pub mesh_id: u32,
    /// Minimum distance squared for this level.
    pub min_distance_squared: f32,
    /// Maximum distance squared for this level.
    pub max_distance_squared: f32,
    /// Padding for alignment.
    pub padding: u32,
}

/// GPU representation of a LOD group.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuLodGroup {
    /// Array of LOD levels (up to 8).
    pub levels: [GpuLodLevel; MAX_LOD_LEVELS_PER_GROUP],
    /// Number of active LOD levels.
    pub level_count: u32,
    /// LOD bias (-1.0 to 1.0).
    pub lod_bias: f32,
    /// Padding for alignment.
    pub padding1: u32,
    /// Padding for alignment.
    pub padding2: u32,
}

impl Default for GpuLodGroup {
    fn default() -> Self {
        Self {
            levels: [GpuLodLevel::default(); MAX_LOD_LEVELS_PER_GROUP],
            level_count: 0,
            lod_bias: 0.0,
            padding1: 0,
            padding2: 0,
        }
    }
}

/// Result of GPU culling for a single object.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuCullingResult {
    /// Index in input object array.
    pub object_index: u32,
    /// Selected mesh ID.
    pub mesh_id: u32,
    /// 1 if visible, 0 if culled.
    pub is_visible: u32,
    /// Selected LOD level index.
    pub lod_level: u32,
}

/// Statistics counters for GPU culling.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuCullingCounters {
    /// Number of visible objects.
    pub visible_count: u32,
    /// Number of objects culled by frustum test.
    pub frustum_culled_count: u32,
    /// Number of objects culled by distance test.
    pub distance_culled_count: u32,
    /// Total number of objects processed.
    pub total_processed: u32,
}

/// Push constants for the culling compute shader.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CullingPushConstants {
    view_proj: [[f32; 4]; 4],
    camera_position: [f32; 4],
    frustum_planes: [[f32; 4]; 6],
    max_distance: f32,
    object_count: u32,
    enable_lod: u32,
    enable_distance_culling: u32,
}

/// Configuration for GPU culling system.
#[derive(Debug, Clone)]
pub struct GpuCullingConfig {
    /// Maximum number of objects that can be culled.
    pub max_objects: usize,
    /// Maximum number of LOD groups.
    pub max_lod_groups: usize,
    /// Enable LOD selection on GPU.
    pub enable_lod_selection: bool,
    /// Enable distance culling.
    pub enable_distance_culling: bool,
    /// Maximum rendering distance.
    pub max_distance: f32,
}

impl Default for GpuCullingConfig {
    fn default() -> Self {
        Self {
            max_objects: 10000,
            max_lod_groups: MAX_LOD_GROUPS,
            enable_lod_selection: true,
            enable_distance_culling: true,
            max_distance: 1000.0,
        }
    }
}

/// Statistics about GPU culling performance.
#[derive(Debug, Clone, Copy, Default)]
pub struct GpuCullingStats {
    /// Number of visible objects after culling.
    pub visible_count: usize,
    /// Number of objects culled by frustum test.
    pub frustum_culled: usize,
    /// Number of objects culled by distance test.
    pub distance_culled: usize,
    /// Total number of objects processed.
    pub total_processed: usize,
    /// Culling efficiency (percentage culled).
    pub cull_rate: f32,
}

/// GPU-driven culling manager.
///
/// Manages compute shader execution for frustum culling and LOD selection.
pub struct GpuCullingManager {
    device: Arc<Device>,
    queue: Arc<Queue>,
    allocator: Arc<dyn MemoryAllocator>,
    command_allocator: Arc<dyn CommandBufferAllocator>,
    config: GpuCullingConfig,

    compute_pipeline: Arc<ComputePipeline>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

    object_buffer: Subbuffer<[GpuObjectData]>,
    lod_group_buffer: Subbuffer<[GpuLodGroup]>,
    result_buffer: Subbuffer<[GpuCullingResult]>,
    counter_buffer: Subbuffer<GpuCullingCounters>,

    current_object_count: usize,
    current_lod_group_count: usize,
}

impl GpuCullingManager {
    /// Creates a new GPU culling manager.
    ///
    /// # Arguments
    ///
    /// * `device` - Vulkan device
    /// * `allocator` - Memory allocator for GPU buffers
    /// * `command_allocator` - Command buffer allocator
    /// * `queue` - Queue for submitting compute commands
    /// * `config` - Culling configuration
    ///
    /// # Errors
    ///
    /// Returns an error if pipeline creation or buffer allocation fails.
    pub fn new(
        device: Arc<Device>,
        allocator: Arc<dyn MemoryAllocator>,
        command_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
        config: GpuCullingConfig,
    ) -> Result<Self> {
        info!(
            "Initializing GPU culling manager (max objects: {}, max LOD groups: {})",
            config.max_objects, config.max_lod_groups
        );

        let compute_pipeline = Self::create_compute_pipeline(&device)?;
        let descriptor_set_allocator =
            Arc::new(StandardDescriptorSetAllocator::new(device.clone(), Default::default()));

        let object_buffer = Buffer::new_slice::<GpuObjectData>(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            config.max_objects as u64,
        )
        .map_err(|e| eyre::eyre!("Failed to create object buffer: {}", e))?;

        let lod_group_buffer = Buffer::new_slice::<GpuLodGroup>(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            config.max_lod_groups as u64,
        )
        .map_err(|e| eyre::eyre!("Failed to create LOD group buffer: {}", e))?;

        let result_buffer = Buffer::new_slice::<GpuCullingResult>(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            config.max_objects as u64,
        )
        .map_err(|e| eyre::eyre!("Failed to create result buffer: {}", e))?;

        let counter_buffer = Buffer::from_data(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            GpuCullingCounters::default(),
        )
        .map_err(|e| eyre::eyre!("Failed to create counter buffer: {}", e))?;

        info!("GPU culling manager initialized successfully");

        Ok(Self {
            device,
            queue,
            allocator,
            command_allocator,
            config,
            compute_pipeline,
            descriptor_set_allocator,
            object_buffer,
            lod_group_buffer,
            result_buffer,
            counter_buffer,
            current_object_count: 0,
            current_lod_group_count: 0,
        })
    }

    /// Creates the compute pipeline for GPU culling.
    fn create_compute_pipeline(device: &Arc<Device>) -> Result<Arc<ComputePipeline>> {
        let shader = gpu_cull_cs::load(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load compute shader: {}", e))?;

        let cs = shader.entry_point("main").ok_or_else(|| {
            eyre::eyre!("Compute shader entry point 'main' not found")
        })?;

        let stage = PipelineShaderStageCreateInfo::new(cs);
        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
                .into_pipeline_layout_create_info(device.clone())
                .map_err(|e| eyre::eyre!("Failed to create pipeline layout info: {}", e))?,
        )
        .map_err(|e| eyre::eyre!("Failed to create pipeline layout: {}", e))?;

        let pipeline = ComputePipeline::new(
            device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(stage, layout),
        )
        .map_err(|e| eyre::eyre!("Failed to create compute pipeline: {}", e))?;

        debug!("GPU culling compute pipeline created");
        Ok(pipeline)
    }

    /// Updates object data on GPU.
    ///
    /// # Arguments
    ///
    /// * `objects` - Slice of object data to upload
    ///
    /// # Errors
    ///
    /// Returns an error if buffer write fails or object count exceeds maximum.
    pub fn update_objects(&mut self, objects: &[GpuObjectData]) -> Result<()> {
        if objects.len() > self.config.max_objects {
            return Err(eyre::eyre!(
                "Object count {} exceeds maximum {}",
                objects.len(),
                self.config.max_objects
            ));
        }

        let mut write_guard = self
            .object_buffer
            .write()
            .map_err(|e| eyre::eyre!("Failed to lock object buffer: {}", e))?;

        write_guard[..objects.len()].copy_from_slice(objects);
        self.current_object_count = objects.len();

        trace!("Updated {} objects on GPU", objects.len());
        Ok(())
    }

    /// Updates LOD group data on GPU.
    ///
    /// # Arguments
    ///
    /// * `lod_groups` - Slice of LOD group data to upload
    ///
    /// # Errors
    ///
    /// Returns an error if buffer write fails or LOD group count exceeds maximum.
    pub fn update_lod_groups(&mut self, lod_groups: &[GpuLodGroup]) -> Result<()> {
        if lod_groups.len() > self.config.max_lod_groups {
            return Err(eyre::eyre!(
                "LOD group count {} exceeds maximum {}",
                lod_groups.len(),
                self.config.max_lod_groups
            ));
        }

        let mut write_guard = self
            .lod_group_buffer
            .write()
            .map_err(|e| eyre::eyre!("Failed to lock LOD group buffer: {}", e))?;

        write_guard[..lod_groups.len()].copy_from_slice(lod_groups);
        self.current_lod_group_count = lod_groups.len();

        trace!("Updated {} LOD groups on GPU", lod_groups.len());
        Ok(())
    }

    /// Executes GPU culling and returns visible objects.
    ///
    /// # Arguments
    ///
    /// * `view_proj` - Combined view-projection matrix
    /// * `camera_position` - Camera position in world space
    ///
    /// # Returns
    ///
    /// Tuple of (visible objects, culling statistics)
    ///
    /// # Errors
    ///
    /// Returns an error if compute dispatch or buffer readback fails.
    pub fn cull(
        &mut self,
        view_proj: Mat4,
        camera_position: Vec3,
    ) -> Result<(Vec<GpuCullingResult>, GpuCullingStats)> {
        if self.current_object_count == 0 {
            return Ok((Vec::new(), GpuCullingStats::default()));
        }

        self.reset_counters()?;

        let frustum_planes = Self::extract_frustum_planes(view_proj);

        let push_constants = CullingPushConstants {
            view_proj: view_proj.to_cols_array_2d(),
            camera_position: [camera_position.x, camera_position.y, camera_position.z, 1.0],
            frustum_planes,
            max_distance: self.config.max_distance,
            object_count: self.current_object_count as u32,
            enable_lod: if self.config.enable_lod_selection { 1 } else { 0 },
            enable_distance_culling: if self.config.enable_distance_culling { 1 } else { 0 },
        };

        let descriptor_set = self.create_descriptor_set()?;

        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| eyre::eyre!("Failed to create command buffer: {}", e))?;

        builder
            .bind_pipeline_compute(self.compute_pipeline.clone())
            .map_err(|e| eyre::eyre!("Failed to bind compute pipeline: {}", e))?
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.compute_pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .map_err(|e| eyre::eyre!("Failed to bind descriptor sets: {}", e))?
            .push_constants(
                self.compute_pipeline.layout().clone(),
                0,
                push_constants,
            )
            .map_err(|e| eyre::eyre!("Failed to push constants: {}", e))?;

        let workgroup_size = 256;
        let workgroup_count =
            (self.current_object_count + workgroup_size - 1) / workgroup_size;

        unsafe {
            builder
                .dispatch([workgroup_count as u32, 1, 1])
                .map_err(|e| eyre::eyre!("Failed to dispatch compute shader: {}", e))?;
        }

        let command_buffer = builder
            .build()
            .map_err(|e| eyre::eyre!("Failed to build command buffer: {}", e))?;

        let future = vulkano::sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .map_err(|e| eyre::eyre!("Failed to execute command buffer: {}", e))?
            .then_signal_fence_and_flush()
            .map_err(|e| eyre::eyre!("Failed to flush command buffer: {}", e))?;

        future
            .wait(None)
            .map_err(|e| eyre::eyre!("Failed to wait for GPU: {}", e))?;

        let counters = self.read_counters()?;
        let visible_objects = self.read_results(counters.visible_count as usize)?;

        let stats = GpuCullingStats {
            visible_count: counters.visible_count as usize,
            frustum_culled: counters.frustum_culled_count as usize,
            distance_culled: counters.distance_culled_count as usize,
            total_processed: counters.total_processed as usize,
            cull_rate: if counters.total_processed > 0 {
                (counters.total_processed - counters.visible_count) as f32
                    / counters.total_processed as f32
                    * 100.0
            } else {
                0.0
            },
        };

        trace!(
            "GPU culling complete: {} visible / {} total ({}% culled)",
            stats.visible_count,
            stats.total_processed,
            stats.cull_rate
        );

        Ok((visible_objects, stats))
    }

    /// Extracts frustum planes from view-projection matrix.
    fn extract_frustum_planes(view_proj: Mat4) -> [[f32; 4]; 6] {
        let m = view_proj.to_cols_array_2d();

        let normalize_plane = |plane: [f32; 4]| -> [f32; 4] {
            let length = (plane[0] * plane[0]
                + plane[1] * plane[1]
                + plane[2] * plane[2])
                .sqrt();
            [
                plane[0] / length,
                plane[1] / length,
                plane[2] / length,
                plane[3] / length,
            ]
        };

        [
            normalize_plane([
                m[0][3] + m[0][2],
                m[1][3] + m[1][2],
                m[2][3] + m[2][2],
                m[3][3] + m[3][2],
            ]),
            normalize_plane([
                m[0][3] - m[0][2],
                m[1][3] - m[1][2],
                m[2][3] - m[2][2],
                m[3][3] - m[3][2],
            ]),
            normalize_plane([
                m[0][3] + m[0][0],
                m[1][3] + m[1][0],
                m[2][3] + m[2][0],
                m[3][3] + m[3][0],
            ]),
            normalize_plane([
                m[0][3] - m[0][0],
                m[1][3] - m[1][0],
                m[2][3] - m[2][0],
                m[3][3] - m[3][0],
            ]),
            normalize_plane([
                m[0][3] - m[0][1],
                m[1][3] - m[1][1],
                m[2][3] - m[2][1],
                m[3][3] - m[3][1],
            ]),
            normalize_plane([
                m[0][3] + m[0][1],
                m[1][3] + m[1][1],
                m[2][3] + m[2][1],
                m[3][3] + m[3][1],
            ]),
        ]
    }

    /// Creates descriptor set for compute pipeline.
    fn create_descriptor_set(&self) -> Result<Arc<DescriptorSet>> {
        let layout = self.compute_pipeline.layout().set_layouts()[0].clone();

        DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout,
            [
                WriteDescriptorSet::buffer(0, self.object_buffer.clone()),
                WriteDescriptorSet::buffer(1, self.lod_group_buffer.clone()),
                WriteDescriptorSet::buffer(2, self.result_buffer.clone()),
                WriteDescriptorSet::buffer(3, self.counter_buffer.clone()),
            ],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create descriptor set: {}", e))
    }

    /// Resets culling counters to zero.
    fn reset_counters(&mut self) -> Result<()> {
        let mut write_guard = self
            .counter_buffer
            .write()
            .map_err(|e| eyre::eyre!("Failed to lock counter buffer: {}", e))?;

        *write_guard = GpuCullingCounters::default();
        Ok(())
    }

    /// Reads culling counters from GPU.
    fn read_counters(&self) -> Result<GpuCullingCounters> {
        let read_guard = self
            .counter_buffer
            .read()
            .map_err(|e| eyre::eyre!("Failed to read counter buffer: {}", e))?;

        Ok(*read_guard)
    }

    /// Reads culling results from GPU.
    fn read_results(&self, count: usize) -> Result<Vec<GpuCullingResult>> {
        let read_guard = self
            .result_buffer
            .read()
            .map_err(|e| eyre::eyre!("Failed to read result buffer: {}", e))?;

        Ok(read_guard[..count.min(self.config.max_objects)].to_vec())
    }

    /// Gets the current configuration.
    pub fn config(&self) -> &GpuCullingConfig {
        &self.config
    }

    /// Gets the current number of objects.
    pub fn object_count(&self) -> usize {
        self.current_object_count
    }

    /// Gets the current number of LOD groups.
    pub fn lod_group_count(&self) -> usize {
        self.current_lod_group_count
    }
}

mod gpu_cull_cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/gpu_cull.comp"
    }
}

/// Helper functions for converting engine types to GPU types.
pub mod conversions {
    use super::*;

    /// Converts an AABB to GPU format.
    pub fn aabb_to_gpu(aabb: &Aabb) -> ([f32; 4], [f32; 4]) {
        let min = [aabb.min.x, aabb.min.y, aabb.min.z, 0.0];
        let max = [aabb.max.x, aabb.max.y, aabb.max.z, 0.0];
        (min, max)
    }

    /// Creates a GPU object data structure.
    pub fn create_gpu_object(
        aabb: &Aabb,
        position: Vec3,
        mesh_id: u32,
        lod_group_id: Option<u32>,
    ) -> GpuObjectData {
        let (aabb_min, aabb_max) = aabb_to_gpu(aabb);
        let bounding_radius = aabb.half_extents().length();

        GpuObjectData {
            aabb_min,
            aabb_max,
            position: [position.x, position.y, position.z, 1.0],
            mesh_id,
            lod_group_id: lod_group_id.unwrap_or(u32::MAX),
            bounding_radius,
            padding: 0,
        }
    }

    /// Creates a GPU LOD level structure.
    pub fn create_gpu_lod_level(
        mesh_id: u32,
        min_distance: f32,
        max_distance: f32,
    ) -> GpuLodLevel {
        GpuLodLevel {
            mesh_id,
            min_distance_squared: min_distance * min_distance,
            max_distance_squared: max_distance * max_distance,
            padding: 0,
        }
    }

    /// Creates a GPU LOD group structure.
    pub fn create_gpu_lod_group(
        levels: &[(u32, f32, f32)],
        lod_bias: f32,
    ) -> GpuLodGroup {
        let mut gpu_group = GpuLodGroup::default();
        gpu_group.level_count = levels.len().min(super::MAX_LOD_LEVELS_PER_GROUP) as u32;
        gpu_group.lod_bias = lod_bias;

        for (i, &(mesh_id, min_dist, max_dist)) in levels.iter().take(super::MAX_LOD_LEVELS_PER_GROUP).enumerate() {
            gpu_group.levels[i] = create_gpu_lod_level(mesh_id, min_dist, max_dist);
        }

        gpu_group
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_culling_config_default() {
        let config = GpuCullingConfig::default();
        assert_eq!(config.max_objects, 10000);
        assert_eq!(config.max_lod_groups, MAX_LOD_GROUPS);
        assert!(config.enable_lod_selection);
        assert!(config.enable_distance_culling);
    }

    #[test]
    fn test_gpu_object_data_size() {
        assert_eq!(
            std::mem::size_of::<GpuObjectData>(),
            16 + 16 + 16 + 16
        );
    }

    #[test]
    fn test_gpu_lod_group_size() {
        let size = std::mem::size_of::<GpuLodGroup>();
        assert!(size > 0);
    }

    #[test]
    fn test_create_gpu_object() {
        let aabb = Aabb::from_min_max(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let position = Vec3::new(5.0, 10.0, 15.0);

        let gpu_obj = conversions::create_gpu_object(&aabb, position, 42, Some(7));

        assert_eq!(gpu_obj.mesh_id, 42);
        assert_eq!(gpu_obj.lod_group_id, 7);
        assert_eq!(gpu_obj.position[0], 5.0);
        assert_eq!(gpu_obj.position[1], 10.0);
        assert_eq!(gpu_obj.position[2], 15.0);
    }

    #[test]
    fn test_create_gpu_lod_level() {
        let lod_level = conversions::create_gpu_lod_level(123, 10.0, 50.0);

        assert_eq!(lod_level.mesh_id, 123);
        assert_eq!(lod_level.min_distance_squared, 100.0);
        assert_eq!(lod_level.max_distance_squared, 2500.0);
    }

    #[test]
    fn test_create_gpu_lod_group() {
        let levels = vec![(1, 0.0, 10.0), (2, 10.0, 50.0), (3, 50.0, 100.0)];

        let gpu_group = conversions::create_gpu_lod_group(&levels, 0.5);

        assert_eq!(gpu_group.level_count, 3);
        assert_eq!(gpu_group.lod_bias, 0.5);
        assert_eq!(gpu_group.levels[0].mesh_id, 1);
        assert_eq!(gpu_group.levels[1].mesh_id, 2);
        assert_eq!(gpu_group.levels[2].mesh_id, 3);
    }

    #[test]
    fn test_extract_frustum_planes() {
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0);
        let view_proj = proj * view;

        let planes = GpuCullingManager::extract_frustum_planes(view_proj);

        assert_eq!(planes.len(), 6);
        for plane in &planes {
            let length = (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2]).sqrt();
            assert!((length - 1.0).abs() < 0.01, "Plane should be normalized");
        }
    }

    #[test]
    fn test_gpu_culling_stats_default() {
        let stats = GpuCullingStats::default();
        assert_eq!(stats.visible_count, 0);
        assert_eq!(stats.frustum_culled, 0);
        assert_eq!(stats.distance_culled, 0);
        assert_eq!(stats.total_processed, 0);
        assert_eq!(stats.cull_rate, 0.0);
    }
}
