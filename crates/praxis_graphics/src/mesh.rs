//! Mesh data structures and management for the graphics system.
//!
//! This module provides the core mesh data structures and GPU buffer management
//! for rendering 3D geometry. Meshes can be created procedurally or loaded from
//! external sources.

use crate::vertex::Vertex3D;
use praxis_utils::{debug, eyre, trace, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::CommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        CopyBufferInfo,
    },
    device::Queue,
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
    sync::{self, GpuFuture},
};

/// GPU-side mesh data containing vertex and index buffers.
///
/// This structure holds the Vulkan buffers for a mesh that has been uploaded
/// to the GPU. It's used by the renderer to draw geometry.
#[derive(Clone)]
pub struct GpuMesh {
    /// Vertex buffer containing mesh vertices.
    pub vertex_buffer: Subbuffer<[Vertex3D]>,

    /// Index buffer containing triangle indices.
    pub index_buffer: Subbuffer<[u16]>,

    /// Number of indices to draw.
    pub index_count: u32,

    /// Number of vertices in the mesh.
    pub vertex_count: u32,
}

impl GpuMesh {
    /// Creates a new GPU mesh from vertex and index data using staging buffers.
    ///
    /// This method uses a staging buffer approach for optimal performance:
    /// 1. Creates host-visible staging buffers for CPU-side data
    /// 2. Creates device-local GPU buffers for optimal rendering performance
    /// 3. Uses a transfer command buffer to copy data from staging to device buffers
    /// 4. Synchronizes with a fence to ensure the transfer completes
    ///
    /// This is the synchronous version that blocks until the transfer completes.
    /// For async uploads, use `new_async` which returns a future.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating buffers
    /// * `command_buffer_allocator` - Allocator for command buffers
    /// * `transfer_queue` - Queue for transfer operations
    /// * `vertices` - Vertex data to upload
    /// * `indices` - Index data to upload
    ///
    /// # Errors
    ///
    /// Returns an error if buffer creation, command recording, or submission fails.
    pub fn new(
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        transfer_queue: Arc<Queue>,
        vertices: Vec<Vertex3D>,
        indices: Vec<u16>,
    ) -> Result<Self> {
        trace!(
            "Creating GPU mesh with {} vertices, {} indices",
            vertices.len(),
            indices.len()
        );

        // Validate mesh data
        if vertices.is_empty() {
            return Err(eyre::eyre!("Cannot create GPU mesh with empty vertex data"));
        }
        if indices.is_empty() {
            return Err(eyre::eyre!("Cannot create GPU mesh with empty index data"));
        }

        let vertex_count = vertices.len() as u32;
        let index_count = indices.len() as u32;

        // Create staging buffers (host-visible, for CPU writes)
        let vertex_staging_buffer = Buffer::from_iter(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices.iter().copied(),
        )
        .map_err(|e| eyre::eyre!("Failed to create vertex staging buffer: {}", e))?;

        let index_staging_buffer = Buffer::from_iter(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices.iter().copied(),
        )
        .map_err(|e| eyre::eyre!("Failed to create index staging buffer: {}", e))?;

        // Create device-local buffers (GPU only, optimal performance)
        let vertex_buffer = Buffer::new_slice::<Vertex3D>(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            vertex_count as u64,
        )
        .map_err(|e| eyre::eyre!("Failed to create vertex buffer: {}", e))?;

        let index_buffer = Buffer::new_slice::<u16>(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            index_count as u64,
        )
        .map_err(|e| eyre::eyre!("Failed to create index buffer: {}", e))?;

        // Build transfer command buffer
        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator,
            transfer_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| eyre::eyre!("Failed to create command buffer builder: {}", e))?;

        // Copy vertex data from staging to device buffer
        builder
            .copy_buffer(CopyBufferInfo::buffers(
                vertex_staging_buffer.clone(),
                vertex_buffer.clone(),
            ))
            .map_err(|e| eyre::eyre!("Failed to record vertex buffer copy: {}", e))?;

        // Copy index data from staging to device buffer
        builder
            .copy_buffer(CopyBufferInfo::buffers(
                index_staging_buffer.clone(),
                index_buffer.clone(),
            ))
            .map_err(|e| eyre::eyre!("Failed to record index buffer copy: {}", e))?;

        // Vulkano 0.35 handles synchronization automatically through the command buffer builder
        // No explicit pipeline barrier needed for buffer copies

        let command_buffer = builder
            .build()
            .map_err(|e| eyre::eyre!("Failed to build transfer command buffer: {}", e))?;

        // Submit to transfer queue with proper fence synchronization
        trace!("Submitting mesh transfer command buffer");
        let future = sync::now(transfer_queue.device().clone())
            .then_execute(transfer_queue.clone(), command_buffer)
            .map_err(|e| eyre::eyre!("Failed to execute transfer command: {}", e))?
            .then_signal_fence_and_flush()
            .map_err(|e| eyre::eyre!("Failed to signal fence and flush: {}", e))?;

        // Wait for transfer to complete before returning
        // This ensures buffers are ready for use in rendering
        future
            .wait(None)
            .map_err(|e| eyre::eyre!("Failed to wait for mesh transfer: {}", e))?;

        trace!("Mesh transfer complete");

        Ok(Self {
            vertex_buffer,
            index_buffer,
            index_count,
            vertex_count,
        })
    }

    /// Creates a new GPU mesh with asynchronous transfer using staging buffers.
    ///
    /// This method implements efficient GPU upload using a multi-stage approach:
    /// 1. Creates host-visible staging buffers (CPU accessible, fast write)
    /// 2. Creates device-local destination buffers (GPU only, optimal for rendering)
    /// 3. Records a transfer command buffer to copy from staging to device buffers
    /// 4. Submits to the transfer queue with fence synchronization
    /// 5. Returns immediately with a future - caller can wait on it for completion
    ///
    /// # Performance Benefits
    ///
    /// - Staging buffers allow fast CPU writes without GPU memory mapping overhead
    /// - Device-local buffers provide maximum GPU access performance
    /// - Transfer queue can operate asynchronously with graphics operations
    /// - Fence synchronization allows overlapping CPU work with GPU transfers
    /// - Non-blocking: caller can continue work and wait on the future later
    ///
    /// # Usage Example
    ///
    /// ```rust,ignore
    /// let (mesh, future) = GpuMesh::new_async(
    ///     allocator,
    ///     cmd_allocator,
    ///     transfer_queue,
    ///     vertices,
    ///     indices
    /// )?;
    ///
    /// // Do other work here while GPU transfer happens
    /// do_other_work();
    ///
    /// // Wait for transfer to complete when needed
    /// future.wait(None)?;
    /// ```
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating buffers
    /// * `command_buffer_allocator` - Allocator for command buffers
    /// * `transfer_queue` - Queue for transfer operations
    /// * `vertices` - Vertex data to upload
    /// * `indices` - Index data to upload
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - The GPU mesh with device-local buffers
    /// - A future that completes when the transfer finishes
    ///
    /// # Errors
    ///
    /// Returns an error if buffer creation, command recording, or submission fails.
    pub fn new_async(
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        transfer_queue: Arc<Queue>,
        vertices: Vec<Vertex3D>,
        indices: Vec<u16>,
    ) -> Result<(Self, Box<dyn GpuFuture>)> {
        trace!(
            "Creating GPU mesh asynchronously with {} vertices, {} indices",
            vertices.len(),
            indices.len()
        );

        // Validate mesh data
        if vertices.is_empty() {
            return Err(eyre::eyre!("Cannot create GPU mesh with empty vertex data"));
        }
        if indices.is_empty() {
            return Err(eyre::eyre!("Cannot create GPU mesh with empty index data"));
        }

        let vertex_count = vertices.len() as u32;
        let index_count = indices.len() as u32;

        // Create staging buffers (host-visible, for CPU writes)
        let vertex_staging_buffer = Buffer::from_iter(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices.iter().copied(),
        )
        .map_err(|e| eyre::eyre!("Failed to create vertex staging buffer: {}", e))?;

        let index_staging_buffer = Buffer::from_iter(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices.iter().copied(),
        )
        .map_err(|e| eyre::eyre!("Failed to create index staging buffer: {}", e))?;

        // Create device-local buffers (GPU only, optimal performance)
        let vertex_buffer = Buffer::new_slice::<Vertex3D>(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            vertex_count as u64,
        )
        .map_err(|e| eyre::eyre!("Failed to create vertex buffer: {}", e))?;

        let index_buffer = Buffer::new_slice::<u16>(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            index_count as u64,
        )
        .map_err(|e| eyre::eyre!("Failed to create index buffer: {}", e))?;

        // Build transfer command buffer
        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator,
            transfer_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| eyre::eyre!("Failed to create command buffer builder: {}", e))?;

        // Copy vertex data from staging to device buffer
        builder
            .copy_buffer(CopyBufferInfo::buffers(
                vertex_staging_buffer.clone(),
                vertex_buffer.clone(),
            ))
            .map_err(|e| eyre::eyre!("Failed to record vertex buffer copy: {}", e))?;

        // Copy index data from staging to device buffer
        builder
            .copy_buffer(CopyBufferInfo::buffers(
                index_staging_buffer.clone(),
                index_buffer.clone(),
            ))
            .map_err(|e| eyre::eyre!("Failed to record index buffer copy: {}", e))?;

        // Vulkano 0.35 handles synchronization automatically through the command buffer builder
        // No explicit pipeline barrier needed for buffer copies

        let command_buffer = builder
            .build()
            .map_err(|e| eyre::eyre!("Failed to build transfer command buffer: {}", e))?;

        // Submit to transfer queue with proper fence synchronization (non-blocking)
        // Caller must wait on the returned future before using the mesh
        trace!("Submitting async mesh transfer command buffer");
        let future = sync::now(transfer_queue.device().clone())
            .then_execute(transfer_queue.clone(), command_buffer)
            .map_err(|e| eyre::eyre!("Failed to execute transfer command: {}", e))?
            .then_signal_fence_and_flush()
            .map_err(|e| eyre::eyre!("Failed to signal fence and flush: {}", e))?;

        trace!("Mesh transfer submitted asynchronously");

        let mesh = Self {
            vertex_buffer,
            index_buffer,
            index_count,
            vertex_count,
        };

        Ok((mesh, Box::new(future)))
    }
}

/// CPU-side mesh data definition.
///
/// This structure holds the mesh data before it's uploaded to the GPU.
/// It supports various vertex attributes like positions, colors, normals, UVs, and tangents.
#[derive(Debug, Clone)]
pub struct MeshData {
    /// Vertex positions in local space.
    pub positions: Vec<[f32; 3]>,

    /// Vertex colors (RGB). If None, uses white.
    pub colors: Option<Vec<[f32; 3]>>,

    /// Vertex normals. If None, normals are not used.
    pub normals: Option<Vec<[f32; 3]>>,

    /// Texture coordinates (UV). If None, UVs are not used.
    pub uvs: Option<Vec<[f32; 2]>>,

    /// Tangent vectors for normal mapping. Each tangent is a vec4 where xyz is the tangent
    /// direction and w is the handedness (+1 or -1) for computing the bitangent.
    pub tangents: Option<Vec<[f32; 4]>>,

    /// Triangle indices.
    pub indices: Vec<u16>,
}

impl MeshData {
    /// Creates a new mesh data with positions and indices.
    pub fn new(positions: Vec<[f32; 3]>, indices: Vec<u16>) -> Self {
        Self {
            positions,
            colors: None,
            normals: None,
            uvs: None,
            tangents: None,
            indices,
        }
    }

    /// Creates a mesh data with positions, colors, and indices.
    pub fn with_colors(positions: Vec<[f32; 3]>, colors: Vec<[f32; 3]>, indices: Vec<u16>) -> Self {
        Self {
            positions,
            colors: Some(colors),
            normals: None,
            uvs: None,
            tangents: None,
            indices,
        }
    }

    /// Creates a mesh data with positions, UVs, and indices.
    ///
    /// Colors will default to white [1.0, 1.0, 1.0].
    pub fn with_uvs(positions: Vec<[f32; 3]>, uvs: Vec<[f32; 2]>, indices: Vec<u16>) -> Self {
        Self {
            positions,
            colors: None,
            normals: None,
            uvs: Some(uvs),
            tangents: None,
            indices,
        }
    }

    /// Creates a mesh data with positions, colors, UVs, and indices.
    pub fn with_colors_and_uvs(
        positions: Vec<[f32; 3]>,
        colors: Vec<[f32; 3]>,
        uvs: Vec<[f32; 2]>,
        indices: Vec<u16>,
    ) -> Self {
        Self {
            positions,
            colors: Some(colors),
            normals: None,
            uvs: Some(uvs),
            tangents: None,
            indices,
        }
    }

    /// Converts this mesh data to a vector of `Vertex3D` for GPU upload.
    ///
    /// If colors are not provided, vertices will use white (1.0, 1.0, 1.0).
    /// If normals are not provided, vertices will use up direction (0.0, 1.0, 0.0).
    /// If UVs are not provided, vertices will use (0.0, 0.0).
    /// If tangents are not provided, vertices will use (1.0, 0.0, 0.0, 1.0).
    ///
    /// # Panics
    ///
    /// Panics if positions are empty. Mesh data must have at least one vertex.
    pub fn to_vertices(&self) -> Vec<Vertex3D> {
        assert!(
            !self.positions.is_empty(),
            "MeshData must have at least one position"
        );

        let default_color = [1.0, 1.0, 1.0];
        let default_normal = [0.0, 1.0, 0.0];
        let default_uv = [0.0, 0.0];
        let default_tangent = [1.0, 0.0, 0.0, 1.0];

        self.positions
            .iter()
            .enumerate()
            .map(|(i, &position)| {
                let normal = self
                    .normals
                    .as_ref()
                    .and_then(|normals| normals.get(i))
                    .copied()
                    .unwrap_or(default_normal);

                let color = self
                    .colors
                    .as_ref()
                    .and_then(|colors| colors.get(i))
                    .copied()
                    .unwrap_or(default_color);

                let uv = self
                    .uvs
                    .as_ref()
                    .and_then(|uvs| uvs.get(i))
                    .copied()
                    .unwrap_or(default_uv);

                let tangent = self
                    .tangents
                    .as_ref()
                    .and_then(|tangents| tangents.get(i))
                    .copied()
                    .unwrap_or(default_tangent);

                Vertex3D {
                    position,
                    normal,
                    color,
                    uv,
                    tangent,
                    bone_indices: [0, 0, 0, 0],
                    bone_weights: [1.0, 0.0, 0.0, 0.0],
                }
            })
            .collect()
    }

    /// Calculates a bounding sphere that encompasses all vertices in the mesh.
    ///
    /// The bounding sphere is computed using Ritter's algorithm:
    /// 1. Find the centroid of all vertices
    /// 2. Find the farthest vertex from the centroid
    /// 3. Set radius as distance from centroid to farthest vertex
    ///
    /// # Returns
    ///
    /// Returns (center, radius) of the bounding sphere in model space.
    /// Returns ((0,0,0), 0) if the mesh has no vertices.
    pub fn calculate_bounding_sphere(&self) -> ([f32; 3], f32) {
        if self.positions.is_empty() {
            return ([0.0, 0.0, 0.0], 0.0);
        }

        // Calculate centroid
        let mut center = [0.0f32, 0.0, 0.0];
        for pos in &self.positions {
            center[0] += pos[0];
            center[1] += pos[1];
            center[2] += pos[2];
        }
        let count = self.positions.len() as f32;
        center[0] /= count;
        center[1] /= count;
        center[2] /= count;

        // Find maximum distance from center
        let mut max_dist_sq = 0.0f32;
        for pos in &self.positions {
            let dx = pos[0] - center[0];
            let dy = pos[1] - center[1];
            let dz = pos[2] - center[2];
            let dist_sq = dx * dx + dy * dy + dz * dz;
            if dist_sq > max_dist_sq {
                max_dist_sq = dist_sq;
            }
        }

        let radius = max_dist_sq.sqrt();
        (center, radius)
    }

    /// Calculates tangent vectors for normal mapping using the method described by
    /// Lengyel's "Mathematics for 3D Game Programming and Computer Graphics".
    ///
    /// This method requires positions, normals, UVs, and indices to be present.
    /// It calculates tangents per-triangle and accumulates them at vertices,
    /// then orthogonalizes and normalizes them. The handedness is stored in the w component.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if tangents were calculated successfully, or an error if
    /// required data (normals, UVs) is missing.
    pub fn calculate_tangents(&mut self) -> Result<()> {
        if self.normals.is_none() {
            return Err(eyre::eyre!("Normals required for tangent calculation"));
        }
        if self.uvs.is_none() {
            return Err(eyre::eyre!("UVs required for tangent calculation"));
        }

        let vertex_count = self.positions.len();
        let mut tangents = vec![[0.0f32, 0.0, 0.0]; vertex_count];
        let mut bitangents = vec![[0.0f32, 0.0, 0.0]; vertex_count];

        let uvs = self.uvs.as_ref().unwrap();

        // Calculate tangents and bitangents per triangle
        for tri in self.indices.chunks_exact(3) {
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;

            let p0 = self.positions[i0];
            let p1 = self.positions[i1];
            let p2 = self.positions[i2];

            let uv0 = uvs[i0];
            let uv1 = uvs[i1];
            let uv2 = uvs[i2];

            // Calculate edge vectors in position space
            let edge1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let edge2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

            // Calculate edge vectors in UV space
            let delta_uv1 = [uv1[0] - uv0[0], uv1[1] - uv0[1]];
            let delta_uv2 = [uv2[0] - uv0[0], uv2[1] - uv0[1]];

            // Calculate tangent and bitangent
            let r = 1.0 / (delta_uv1[0] * delta_uv2[1] - delta_uv1[1] * delta_uv2[0]);

            let tangent = [
                r * (delta_uv2[1] * edge1[0] - delta_uv1[1] * edge2[0]),
                r * (delta_uv2[1] * edge1[1] - delta_uv1[1] * edge2[1]),
                r * (delta_uv2[1] * edge1[2] - delta_uv1[1] * edge2[2]),
            ];

            let bitangent = [
                r * (-delta_uv2[0] * edge1[0] + delta_uv1[0] * edge2[0]),
                r * (-delta_uv2[0] * edge1[1] + delta_uv1[0] * edge2[1]),
                r * (-delta_uv2[0] * edge1[2] + delta_uv1[0] * edge2[2]),
            ];

            // Accumulate for each vertex of the triangle
            for &idx in &[i0, i1, i2] {
                tangents[idx][0] += tangent[0];
                tangents[idx][1] += tangent[1];
                tangents[idx][2] += tangent[2];

                bitangents[idx][0] += bitangent[0];
                bitangents[idx][1] += bitangent[1];
                bitangents[idx][2] += bitangent[2];
            }
        }

        // Orthogonalize and normalize tangents
        let normals = self.normals.as_ref().unwrap();
        let mut final_tangents = vec![[0.0f32; 4]; vertex_count];

        for i in 0..vertex_count {
            let n = normals[i];
            let t = tangents[i];
            let b = bitangents[i];

            // Gram-Schmidt orthogonalize
            // t' = normalize(t - n * dot(n, t))
            let dot_nt = n[0] * t[0] + n[1] * t[1] + n[2] * t[2];
            let t_ortho = [
                t[0] - n[0] * dot_nt,
                t[1] - n[1] * dot_nt,
                t[2] - n[2] * dot_nt,
            ];

            // Normalize tangent
            let len = (t_ortho[0] * t_ortho[0] + t_ortho[1] * t_ortho[1] + t_ortho[2] * t_ortho[2])
                .sqrt();
            let t_normalized = if len > 0.0001 {
                [t_ortho[0] / len, t_ortho[1] / len, t_ortho[2] / len]
            } else {
                [1.0, 0.0, 0.0]
            };

            // Calculate handedness
            // cross(n, t) dot b < 0 => handedness = -1, else 1
            let cross = [
                n[1] * t_normalized[2] - n[2] * t_normalized[1],
                n[2] * t_normalized[0] - n[0] * t_normalized[2],
                n[0] * t_normalized[1] - n[1] * t_normalized[0],
            ];
            let dot_cross_b = cross[0] * b[0] + cross[1] * b[1] + cross[2] * b[2];
            let handedness = if dot_cross_b < 0.0 { -1.0 } else { 1.0 };

            final_tangents[i] = [
                t_normalized[0],
                t_normalized[1],
                t_normalized[2],
                handedness,
            ];
        }

        self.tangents = Some(final_tangents);
        Ok(())
    }

    /// Uploads this mesh data to the GPU using staging buffers.
    ///
    /// This is a synchronous operation that blocks until the transfer completes.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating GPU buffers
    /// * `command_buffer_allocator` - Allocator for command buffers
    /// * `transfer_queue` - Queue for transfer operations
    ///
    /// # Errors
    ///
    /// Returns an error if buffer creation or upload fails.
    pub fn upload(
        &self,
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        transfer_queue: Arc<Queue>,
    ) -> Result<GpuMesh> {
        let vertices = self.to_vertices();
        GpuMesh::new(
            allocator,
            command_buffer_allocator,
            transfer_queue,
            vertices,
            self.indices.clone(),
        )
    }

    /// Uploads this mesh data to the GPU asynchronously using staging buffers.
    ///
    /// This is a non-blocking operation that returns immediately with a future.
    /// The caller can wait on the future when the transfer needs to complete.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating GPU buffers
    /// * `command_buffer_allocator` - Allocator for command buffers
    /// * `transfer_queue` - Queue for transfer operations
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - The GPU mesh with device-local buffers
    /// - A future that completes when the transfer finishes
    ///
    /// # Errors
    ///
    /// Returns an error if buffer creation or upload fails.
    pub fn upload_async(
        &self,
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        transfer_queue: Arc<Queue>,
    ) -> Result<(GpuMesh, Box<dyn GpuFuture>)> {
        let vertices = self.to_vertices();
        GpuMesh::new_async(
            allocator,
            command_buffer_allocator,
            transfer_queue,
            vertices,
            self.indices.clone(),
        )
    }
}

/// Mesh asset manager that stores and manages GPU meshes.
///
/// This structure acts as a cache for loaded meshes, avoiding duplicate
/// uploads to the GPU. Meshes are identified by unique string IDs.
pub struct MeshAssetManager {
    /// Map of mesh ID to GPU mesh data.
    meshes: HashMap<String, GpuMesh>,

    /// Map of file path to mesh ID for hot-reload support.
    path_to_id: HashMap<PathBuf, String>,

    /// Memory allocator for creating GPU buffers.
    allocator: Arc<dyn MemoryAllocator>,

    /// Command buffer allocator for transfer operations.
    command_buffer_allocator: Arc<dyn CommandBufferAllocator>,

    /// Queue for transfer operations.
    transfer_queue: Arc<Queue>,
}

impl MeshAssetManager {
    /// Creates a new mesh asset manager.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating GPU buffers
    /// * `command_buffer_allocator` - Allocator for command buffers
    /// * `transfer_queue` - Queue for transfer operations
    pub fn new(
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        transfer_queue: Arc<Queue>,
    ) -> Self {
        Self {
            meshes: HashMap::new(),
            path_to_id: HashMap::new(),
            allocator,
            command_buffer_allocator,
            transfer_queue,
        }
    }

    /// Loads a mesh from mesh data.
    ///
    /// If a mesh with the same ID already exists, it will be replaced.
    /// Uses staging buffers for optimal GPU upload performance.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the mesh
    /// * `mesh_data` - Mesh data to upload
    ///
    /// # Errors
    ///
    /// Returns an error if GPU buffer creation fails.
    pub fn load_mesh(&mut self, id: impl Into<String>, mesh_data: MeshData) -> Result<()> {
        let id = id.into();
        debug!(
            "Loading mesh '{}' ({} vertices, {} indices)",
            id,
            mesh_data.positions.len(),
            mesh_data.indices.len()
        );

        let gpu_mesh = mesh_data.upload(
            self.allocator.clone(),
            self.command_buffer_allocator.clone(),
            self.transfer_queue.clone(),
        )?;
        self.meshes.insert(id.clone(), gpu_mesh);

        trace!("Mesh '{}' loaded successfully", id);
        Ok(())
    }

    /// Loads a mesh from mesh data asynchronously.
    ///
    /// If a mesh with the same ID already exists, it will be replaced.
    /// Returns a future that completes when the transfer finishes.
    /// The mesh is immediately available but should not be used until the future completes.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the mesh
    /// * `mesh_data` - Mesh data to upload
    ///
    /// # Returns
    ///
    /// A future that completes when the GPU transfer finishes.
    ///
    /// # Errors
    ///
    /// Returns an error if GPU buffer creation fails.
    pub fn load_mesh_async(
        &mut self,
        id: impl Into<String>,
        mesh_data: MeshData,
    ) -> Result<Box<dyn GpuFuture>> {
        let id = id.into();
        debug!(
            "Loading mesh '{}' asynchronously ({} vertices, {} indices)",
            id,
            mesh_data.positions.len(),
            mesh_data.indices.len()
        );

        let (gpu_mesh, future) = mesh_data.upload_async(
            self.allocator.clone(),
            self.command_buffer_allocator.clone(),
            self.transfer_queue.clone(),
        )?;
        self.meshes.insert(id.clone(), gpu_mesh);

        trace!("Mesh '{}' submitted for async loading", id);
        Ok(future)
    }

    /// Gets a mesh by its ID.
    ///
    /// Returns `None` if the mesh doesn't exist.
    pub fn get_mesh(&self, id: &str) -> Option<&GpuMesh> {
        self.meshes.get(id)
    }

    /// Checks if a mesh exists.
    pub fn contains_mesh(&self, id: &str) -> bool {
        self.meshes.contains_key(id)
    }

    /// Removes a mesh from the manager.
    ///
    /// Returns `true` if the mesh existed and was removed.
    pub fn remove_mesh(&mut self, id: &str) -> bool {
        let removed = self.meshes.remove(id).is_some();
        if removed {
            self.path_to_id.retain(|_, v| v != id);
        }
        removed
    }

    /// Returns the number of loaded meshes.
    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    /// Clears all loaded meshes.
    pub fn clear(&mut self) {
        debug!("Clearing {} loaded meshes", self.meshes.len());
        self.meshes.clear();
        self.path_to_id.clear();
    }

    /// Gets a reference to the memory allocator.
    ///
    /// This can be used to create custom GPU meshes outside of the asset manager.
    pub fn allocator(&self) -> &Arc<dyn MemoryAllocator> {
        &self.allocator
    }

    /// Gets a reference to the command buffer allocator.
    pub fn command_buffer_allocator(&self) -> &Arc<dyn CommandBufferAllocator> {
        &self.command_buffer_allocator
    }

    /// Gets a reference to the transfer queue.
    pub fn transfer_queue(&self) -> &Arc<Queue> {
        &self.transfer_queue
    }

    /// Loads a mesh from mesh data with a file path association.
    ///
    /// This method loads mesh data and associates it with a file path for hot-reload support.
    /// If a mesh with the same ID already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the mesh
    /// * `path` - Path to the mesh file for hot-reload tracking
    /// * `mesh_data` - Mesh data to upload
    ///
    /// # Errors
    ///
    /// Returns an error if GPU buffer creation fails.
    pub fn load_mesh_with_path(
        &mut self,
        id: impl Into<String>,
        path: impl AsRef<std::path::Path>,
        mesh_data: MeshData,
    ) -> Result<()> {
        let id = id.into();
        let path = path.as_ref();

        debug!("Loading mesh '{}' from file '{}'", id, path.display());

        let gpu_mesh = mesh_data.upload(
            self.allocator.clone(),
            self.command_buffer_allocator.clone(),
            self.transfer_queue.clone(),
        )?;

        self.meshes.insert(id.clone(), gpu_mesh);
        self.path_to_id.insert(path.to_path_buf(), id.clone());

        trace!("Mesh '{}' loaded successfully from file", id);
        Ok(())
    }

    /// Reloads a mesh from disk by its file path.
    ///
    /// This method is used for hot-reload functionality. If the file path
    /// corresponds to a loaded mesh, it will be reloaded using the provided
    /// mesh data and the GPU resource will be updated.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the mesh file to reload
    /// * `mesh_data` - New mesh data to upload
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if the mesh was found and reloaded successfully,
    /// `Ok(false)` if the path doesn't correspond to any loaded mesh,
    /// or an error if reloading failed.
    pub fn reload_mesh(
        &mut self,
        path: impl AsRef<std::path::Path>,
        mesh_data: MeshData,
    ) -> Result<bool> {
        let path = path.as_ref();

        if let Some(id) = self.path_to_id.get(path).cloned() {
            debug!("Reloading mesh '{}' from '{}'", id, path.display());

            let gpu_mesh = mesh_data.upload(
                self.allocator.clone(),
                self.command_buffer_allocator.clone(),
                self.transfer_queue.clone(),
            )?;

            self.meshes.insert(id.clone(), gpu_mesh);
            praxis_utils::info!("Mesh '{}' reloaded successfully", id);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_data_creation() {
        let positions = vec![[0.0, 1.0, 0.0], [-1.0, -1.0, 0.0], [1.0, -1.0, 0.0]];
        let indices = vec![0, 1, 2];

        let mesh = MeshData::new(positions.clone(), indices.clone());
        assert_eq!(mesh.positions.len(), 3);
        assert_eq!(mesh.indices.len(), 3);
        assert!(mesh.colors.is_none());
        assert!(mesh.normals.is_none());
        assert!(mesh.uvs.is_none());
    }

    #[test]
    fn test_mesh_data_with_colors() {
        let positions = vec![[0.0, 1.0, 0.0], [-1.0, -1.0, 0.0]];
        let colors = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![0, 1, 2];

        let mesh = MeshData::with_colors(positions.clone(), colors.clone(), indices.clone());
        assert_eq!(mesh.positions.len(), 2);
        assert_eq!(mesh.colors.as_ref().unwrap().len(), 2);
        assert_eq!(mesh.colors.as_ref().unwrap()[0], [1.0, 0.0, 0.0]);
        assert_eq!(mesh.colors.as_ref().unwrap()[1], [0.0, 1.0, 0.0]);
        assert!(mesh.uvs.is_none());
        assert!(mesh.normals.is_none());
    }

    #[test]
    fn test_mesh_data_with_uvs() {
        let positions = vec![[0.0, 1.0, 0.0], [-1.0, -1.0, 0.0]];
        let uvs = vec![[0.0, 0.0], [1.0, 1.0]];
        let indices = vec![0, 1, 2];

        let mesh = MeshData::with_uvs(positions.clone(), uvs.clone(), indices.clone());
        assert_eq!(mesh.positions.len(), 2);
        assert_eq!(mesh.uvs.as_ref().unwrap().len(), 2);
        assert_eq!(mesh.uvs.as_ref().unwrap()[0], [0.0, 0.0]);
        assert_eq!(mesh.uvs.as_ref().unwrap()[1], [1.0, 1.0]);
        assert!(mesh.colors.is_none());
        assert!(mesh.normals.is_none());
    }

    #[test]
    fn test_mesh_data_with_colors_and_uvs() {
        let positions = vec![[0.0, 1.0, 0.0], [-1.0, -1.0, 0.0]];
        let colors = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let uvs = vec![[0.0, 0.0], [1.0, 1.0]];
        let indices = vec![0, 1, 2];

        let mesh = MeshData::with_colors_and_uvs(
            positions.clone(),
            colors.clone(),
            uvs.clone(),
            indices.clone(),
        );
        assert_eq!(mesh.positions.len(), 2);
        assert_eq!(mesh.colors.as_ref().unwrap().len(), 2);
        assert_eq!(mesh.uvs.as_ref().unwrap().len(), 2);
        assert!(mesh.normals.is_none());
    }

    #[test]
    fn test_mesh_data_to_vertices() {
        let positions = vec![[0.0, 1.0, 0.0], [-1.0, -1.0, 0.0]];
        let colors = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![0, 1];

        let mesh = MeshData::with_colors(positions, colors, indices);
        let vertices = mesh.to_vertices();

        assert_eq!(vertices.len(), 2);
        assert_eq!(vertices[0].position, [0.0, 1.0, 0.0]);
        assert_eq!(vertices[0].color, [1.0, 0.0, 0.0]);
        assert_eq!(vertices[1].position, [-1.0, -1.0, 0.0]);
        assert_eq!(vertices[1].color, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_mesh_data_to_vertices_default_color() {
        let positions = vec![[0.0, 1.0, 0.0]];
        let indices = vec![0];

        let mesh = MeshData::new(positions, indices);
        let vertices = mesh.to_vertices();

        assert_eq!(vertices.len(), 1);
        assert_eq!(vertices[0].color, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_mesh_data_to_vertices_default_normal() {
        let positions = vec![[0.0, 1.0, 0.0]];
        let indices = vec![0];

        let mesh = MeshData::new(positions, indices);
        let vertices = mesh.to_vertices();

        assert_eq!(vertices.len(), 1);
        assert_eq!(vertices[0].normal, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_mesh_data_to_vertices_default_uv() {
        let positions = vec![[0.0, 1.0, 0.0]];
        let indices = vec![0];

        let mesh = MeshData::new(positions, indices);
        let vertices = mesh.to_vertices();

        assert_eq!(vertices.len(), 1);
        assert_eq!(vertices[0].uv, [0.0, 0.0]);
    }

    #[test]
    fn test_mesh_data_to_vertices_with_normals() {
        let positions = vec![[0.0, 1.0, 0.0], [1.0, 0.0, 0.0]];
        let normals = vec![[0.0, 0.0, 1.0], [1.0, 0.0, 0.0]];
        let indices = vec![0, 1];

        let mesh = MeshData {
            positions,
            colors: None,
            normals: Some(normals),
            uvs: None,
            tangents: None,
            indices,
        };
        let vertices = mesh.to_vertices();

        assert_eq!(vertices.len(), 2);
        assert_eq!(vertices[0].normal, [0.0, 0.0, 1.0]);
        assert_eq!(vertices[1].normal, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_mesh_data_to_vertices_with_all_attributes() {
        let positions = vec![[1.0, 2.0, 3.0]];
        let colors = vec![[0.5, 0.6, 0.7]];
        let normals = vec![[0.0, 1.0, 0.0]];
        let uvs = vec![[0.25, 0.75]];
        let indices = vec![0];

        let mesh = MeshData {
            positions,
            colors: Some(colors),
            normals: Some(normals),
            uvs: Some(uvs),
            tangents: None,
            indices,
        };
        let vertices = mesh.to_vertices();

        assert_eq!(vertices.len(), 1);
        assert_eq!(vertices[0].position, [1.0, 2.0, 3.0]);
        assert_eq!(vertices[0].color, [0.5, 0.6, 0.7]);
        assert_eq!(vertices[0].normal, [0.0, 1.0, 0.0]);
        assert_eq!(vertices[0].uv, [0.25, 0.75]);
    }

    #[test]
    fn test_mesh_data_empty_mesh() {
        let mesh = MeshData::new(vec![], vec![]);
        assert_eq!(mesh.positions.len(), 0);
        assert_eq!(mesh.indices.len(), 0);
        let vertices = mesh.to_vertices();
        assert_eq!(vertices.len(), 0);
    }

    #[test]
    fn test_mesh_data_mismatched_color_length() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let colors = vec![[1.0, 1.0, 1.0]];
        let indices = vec![0, 1];

        let mesh = MeshData {
            positions,
            colors: Some(colors),
            normals: None,
            uvs: None,
            tangents: None,
            indices,
        };
        let vertices = mesh.to_vertices();

        assert_eq!(vertices[0].color, [1.0, 1.0, 1.0]);
        assert_eq!(vertices[1].color, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_mesh_data_single_vertex() {
        let positions = vec![[5.0, 10.0, 15.0]];
        let indices = vec![0];

        let mesh = MeshData::new(positions, indices);
        let vertices = mesh.to_vertices();

        assert_eq!(vertices.len(), 1);
        assert_eq!(vertices[0].position, [5.0, 10.0, 15.0]);
    }

    #[test]
    fn test_mesh_data_triangle() {
        let positions = vec![[0.0, 1.0, 0.0], [-1.0, -1.0, 0.0], [1.0, -1.0, 0.0]];
        let colors = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let indices = vec![0, 1, 2];

        let mesh = MeshData::with_colors(positions, colors, indices);
        assert_eq!(mesh.positions.len(), 3);
        assert_eq!(mesh.indices.len(), 3);

        let vertices = mesh.to_vertices();
        assert_eq!(vertices.len(), 3);
    }

    #[test]
    fn test_calculate_tangents_simple_quad() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let normals = vec![
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let indices = vec![0, 1, 2, 0, 2, 3];

        let mut mesh = MeshData {
            positions,
            colors: None,
            normals: Some(normals),
            uvs: Some(uvs),
            tangents: None,
            indices,
        };

        let result = mesh.calculate_tangents();
        assert!(result.is_ok());
        assert!(mesh.tangents.is_some());

        let tangents = mesh.tangents.unwrap();
        assert_eq!(tangents.len(), 4);

        // For a flat quad in XY plane with standard UVs, tangent should point along +X
        for tangent in &tangents {
            assert!((tangent[0] - 1.0).abs() < 0.1, "Tangent X should be ~1.0");
            assert!(tangent[1].abs() < 0.1, "Tangent Y should be ~0.0");
            assert!(tangent[2].abs() < 0.1, "Tangent Z should be ~0.0");
            // Handedness should be +1 or -1
            assert!(tangent[3].abs() > 0.5);
        }
    }

    #[test]
    fn test_calculate_tangents_requires_normals() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
        let indices = vec![0, 1, 2];

        let mut mesh = MeshData {
            positions,
            colors: None,
            normals: None,
            uvs: Some(uvs),
            tangents: None,
            indices,
        };

        let result = mesh.calculate_tangents();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Normals required"));
    }

    #[test]
    fn test_calculate_tangents_requires_uvs() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
        let normals = vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
        let indices = vec![0, 1, 2];

        let mut mesh = MeshData {
            positions,
            colors: None,
            normals: Some(normals),
            uvs: None,
            tangents: None,
            indices,
        };

        let result = mesh.calculate_tangents();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("UVs required"));
    }

    #[test]
    fn test_calculate_tangents_orthogonality() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 2.0, 0.0],
            [0.0, 2.0, 0.0],
        ];
        let normals = vec![
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let indices = vec![0, 1, 2, 0, 2, 3];

        let mut mesh = MeshData {
            positions,
            colors: None,
            normals: Some(normals.clone()),
            uvs: Some(uvs),
            tangents: None,
            indices,
        };

        mesh.calculate_tangents().unwrap();
        let tangents = mesh.tangents.as_ref().unwrap();

        // Verify tangent is orthogonal to normal
        for i in 0..4 {
            let t = tangents[i];
            let n = normals[i];
            let dot = t[0] * n[0] + t[1] * n[1] + t[2] * n[2];
            assert!(
                dot.abs() < 0.01,
                "Tangent and normal should be orthogonal, dot={dot} at vertex {i}"
            );
        }
    }

    #[test]
    fn test_calculate_tangents_normalized() {
        let positions = vec![[0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [5.0, 5.0, 0.0]];
        let normals = vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
        let indices = vec![0, 1, 2];

        let mut mesh = MeshData {
            positions,
            colors: None,
            normals: Some(normals),
            uvs: Some(uvs),
            tangents: None,
            indices,
        };

        mesh.calculate_tangents().unwrap();
        let tangents = mesh.tangents.as_ref().unwrap();

        // Verify tangent vectors are normalized (length ~1.0)
        for (i, tangent) in tangents.iter().enumerate() {
            let length =
                (tangent[0] * tangent[0] + tangent[1] * tangent[1] + tangent[2] * tangent[2])
                    .sqrt();
            assert!(
                (length - 1.0).abs() < 0.01,
                "Tangent at vertex {i} should be normalized, length={length}"
            );
        }
    }

    #[test]
    fn test_calculate_tangents_handedness() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
        let normals = vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
        let indices = vec![0, 1, 2];

        let mut mesh = MeshData {
            positions,
            colors: None,
            normals: Some(normals),
            uvs: Some(uvs),
            tangents: None,
            indices,
        };

        mesh.calculate_tangents().unwrap();
        let tangents = mesh.tangents.as_ref().unwrap();

        // Verify handedness is either +1 or -1
        for (i, tangent) in tangents.iter().enumerate() {
            let w = tangent[3];
            assert!(
                (w - 1.0).abs() < 0.01 || (w + 1.0).abs() < 0.01,
                "Tangent handedness at vertex {i} should be +1 or -1, got {w}"
            );
        }
    }

    #[test]
    fn test_calculate_tangents_multiple_triangles() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [1.5, 1.0, 0.0],
        ];
        let normals = vec![
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let uvs = vec![
            [0.0, 0.0],
            [0.5, 0.0],
            [0.25, 1.0],
            [0.5, 0.0],
            [1.0, 0.0],
            [0.75, 1.0],
        ];
        let indices = vec![0, 1, 2, 3, 4, 5];

        let mut mesh = MeshData {
            positions,
            colors: None,
            normals: Some(normals),
            uvs: Some(uvs),
            tangents: None,
            indices,
        };

        let result = mesh.calculate_tangents();
        assert!(result.is_ok());
        assert!(mesh.tangents.is_some());

        let tangents = mesh.tangents.unwrap();
        assert_eq!(tangents.len(), 6);
    }

    #[test]
    fn test_calculate_tangents_shared_vertex() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, -1.0, 0.0],
        ];
        let normals = vec![
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let uvs = vec![[0.0, 0.5], [1.0, 0.5], [0.5, 1.0], [0.5, 0.0]];
        let indices = vec![0, 1, 2, 0, 1, 3];

        let mut mesh = MeshData {
            positions,
            colors: None,
            normals: Some(normals),
            uvs: Some(uvs),
            tangents: None,
            indices,
        };

        let result = mesh.calculate_tangents();
        assert!(result.is_ok());

        let tangents = mesh.tangents.as_ref().unwrap();
        assert_eq!(tangents.len(), 4);

        // Shared vertices (0 and 1) should accumulate tangents from both triangles
        for tangent in tangents {
            let length =
                (tangent[0] * tangent[0] + tangent[1] * tangent[1] + tangent[2] * tangent[2])
                    .sqrt();
            assert!((length - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_calculate_tangents_degenerate_uv() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
        let normals = vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
        let uvs = vec![[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]];
        let indices = vec![0, 1, 2];

        let mut mesh = MeshData {
            positions,
            colors: None,
            normals: Some(normals),
            uvs: Some(uvs),
            tangents: None,
            indices,
        };

        let result = mesh.calculate_tangents();
        assert!(result.is_ok());

        let tangents = mesh.tangents.as_ref().unwrap();
        for tangent in tangents {
            assert!(tangent[0].is_finite());
            assert!(tangent[1].is_finite());
            assert!(tangent[2].is_finite());
            assert!(tangent[3].is_finite());
        }
    }

    #[test]
    fn test_mesh_data_with_tangents() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let tangents = vec![[1.0, 0.0, 0.0, 1.0], [1.0, 0.0, 0.0, 1.0]];
        let indices = vec![0, 1];

        let mesh = MeshData {
            positions,
            colors: None,
            normals: None,
            uvs: None,
            tangents: Some(tangents.clone()),
            indices,
        };

        let vertices = mesh.to_vertices();
        assert_eq!(vertices.len(), 2);
        assert_eq!(vertices[0].tangent, tangents[0]);
        assert_eq!(vertices[1].tangent, tangents[1]);
    }

    #[test]
    fn test_mesh_data_default_tangent() {
        let positions = vec![[0.0, 0.0, 0.0]];
        let indices = vec![0];

        let mesh = MeshData::new(positions, indices);
        let vertices = mesh.to_vertices();

        assert_eq!(vertices.len(), 1);
        assert_eq!(vertices[0].tangent, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_calculate_bounding_sphere_empty() {
        let mesh = MeshData::new(vec![], vec![]);
        let (center, radius) = mesh.calculate_bounding_sphere();
        assert_eq!(center, [0.0, 0.0, 0.0]);
        assert_eq!(radius, 0.0);
    }

    #[test]
    fn test_calculate_bounding_sphere_single_point() {
        let positions = vec![[1.0, 2.0, 3.0]];
        let mesh = MeshData::new(positions, vec![0]);
        let (center, radius) = mesh.calculate_bounding_sphere();
        assert_eq!(center, [1.0, 2.0, 3.0]);
        assert_eq!(radius, 0.0);
    }

    #[test]
    fn test_calculate_bounding_sphere_unit_cube() {
        let positions = vec![
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
        ];
        let mesh = MeshData::new(positions, vec![]);
        let (center, radius) = mesh.calculate_bounding_sphere();

        // Center should be at origin
        assert!((center[0]).abs() < 0.01);
        assert!((center[1]).abs() < 0.01);
        assert!((center[2]).abs() < 0.01);

        // Radius should be sqrt(3) for unit cube
        let expected_radius = (3.0f32).sqrt();
        assert!((radius - expected_radius).abs() < 0.01);
    }

    #[test]
    fn test_calculate_bounding_sphere_offset() {
        let positions = vec![
            [5.0, 5.0, 5.0],
            [7.0, 5.0, 5.0],
            [7.0, 7.0, 5.0],
            [5.0, 7.0, 5.0],
        ];
        let mesh = MeshData::new(positions, vec![]);
        let (center, radius) = mesh.calculate_bounding_sphere();

        // Center should be at (6, 6, 5)
        assert!((center[0] - 6.0).abs() < 0.01);
        assert!((center[1] - 6.0).abs() < 0.01);
        assert!((center[2] - 5.0).abs() < 0.01);

        // Radius should be sqrt(2) for this square
        let expected_radius = (2.0f32).sqrt();
        assert!((radius - expected_radius).abs() < 0.01);
    }

    #[test]
    fn test_calculate_bounding_sphere_line() {
        let positions = vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]];
        let mesh = MeshData::new(positions, vec![]);
        let (center, radius) = mesh.calculate_bounding_sphere();

        // Center should be at midpoint
        assert!((center[0] - 5.0).abs() < 0.01);
        assert!((center[1]).abs() < 0.01);
        assert!((center[2]).abs() < 0.01);

        // Radius should be 5.0
        assert!((radius - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_tangents_cube_face() {
        let positions = vec![
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
        ];
        let normals = vec![
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let indices = vec![0, 1, 2, 0, 2, 3];

        let mut mesh = MeshData {
            positions,
            colors: None,
            normals: Some(normals.clone()),
            uvs: Some(uvs),
            tangents: None,
            indices,
        };

        mesh.calculate_tangents().unwrap();
        let tangents = mesh.tangents.as_ref().unwrap();

        for i in 0..4 {
            let t = [tangents[i][0], tangents[i][1], tangents[i][2]];
            let n = normals[i];

            let t_len = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
            assert!((t_len - 1.0).abs() < 0.01);

            let dot = t[0] * n[0] + t[1] * n[1] + t[2] * n[2];
            assert!(dot.abs() < 0.01);
        }
    }
}
