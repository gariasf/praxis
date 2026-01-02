//! Mesh data structures and management for the graphics system.
//!
//! This module provides the core mesh data structures and GPU buffer management
//! for rendering 3D geometry. Meshes can be created procedurally or loaded from
//! external sources.

use crate::vertex::Vertex3D;
use praxis_utils::{Result, debug, eyre, trace};
use std::collections::HashMap;
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
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
    /// Creates a new GPU mesh from vertex and index data.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating buffers
    /// * `vertices` - Vertex data to upload
    /// * `indices` - Index data to upload
    ///
    /// # Errors
    ///
    /// Returns an error if buffer creation fails.
    pub fn new(
        allocator: Arc<dyn MemoryAllocator>,
        vertices: Vec<Vertex3D>,
        indices: Vec<u16>,
    ) -> Result<Self> {
        trace!(
            "Creating GPU mesh with {} vertices, {} indices",
            vertices.len(),
            indices.len()
        );

        let vertex_buffer = Buffer::from_iter(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices.iter().copied(),
        )
        .map_err(|e| eyre::eyre!("Failed to create vertex buffer: {}", e))?;

        let index_buffer = Buffer::from_iter(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices.iter().copied(),
        )
        .map_err(|e| eyre::eyre!("Failed to create index buffer: {}", e))?;

        Ok(Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            vertex_count: vertices.len() as u32,
        })
    }
}

/// CPU-side mesh data definition.
///
/// This structure holds the mesh data before it's uploaded to the GPU.
/// It supports various vertex attributes like positions, colors, normals, and UVs.
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
            indices,
        }
    }

    /// Converts this mesh data to a vector of `Vertex3D` for GPU upload.
    ///
    /// If colors are not provided, vertices will use white (1.0, 1.0, 1.0).
    pub fn to_vertices(&self) -> Vec<Vertex3D> {
        let default_color = [1.0, 1.0, 1.0];

        self.positions
            .iter()
            .enumerate()
            .map(|(i, &position)| {
                let color = self
                    .colors
                    .as_ref()
                    .and_then(|colors| colors.get(i))
                    .copied()
                    .unwrap_or(default_color);

                Vertex3D { position, color }
            })
            .collect()
    }

    /// Uploads this mesh data to the GPU.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating GPU buffers
    ///
    /// # Errors
    ///
    /// Returns an error if buffer creation fails.
    pub fn upload(&self, allocator: Arc<dyn MemoryAllocator>) -> Result<GpuMesh> {
        let vertices = self.to_vertices();
        GpuMesh::new(allocator, vertices, self.indices.clone())
    }
}

/// Mesh asset manager that stores and manages GPU meshes.
///
/// This structure acts as a cache for loaded meshes, avoiding duplicate
/// uploads to the GPU. Meshes are identified by unique string IDs.
pub struct MeshAssetManager {
    /// Map of mesh ID to GPU mesh data.
    meshes: HashMap<String, GpuMesh>,

    /// Memory allocator for creating GPU buffers.
    allocator: Arc<dyn MemoryAllocator>,
}

impl MeshAssetManager {
    /// Creates a new mesh asset manager.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating GPU buffers
    pub fn new(allocator: Arc<dyn MemoryAllocator>) -> Self {
        Self {
            meshes: HashMap::new(),
            allocator,
        }
    }

    /// Loads a mesh from mesh data.
    ///
    /// If a mesh with the same ID already exists, it will be replaced.
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

        let gpu_mesh = mesh_data.upload(self.allocator.clone())?;
        self.meshes.insert(id.clone(), gpu_mesh);

        trace!("Mesh '{}' loaded successfully", id);
        Ok(())
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
        self.meshes.remove(id).is_some()
    }

    /// Returns the number of loaded meshes.
    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    /// Clears all loaded meshes.
    pub fn clear(&mut self) {
        debug!("Clearing {} loaded meshes", self.meshes.len());
        self.meshes.clear();
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
        assert_eq!(vertices[0].color, [1.0, 1.0, 1.0]); // Default white
    }
}
