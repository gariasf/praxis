//! Full-screen quad rendering for post-processing.
//!
//! This module provides utilities for rendering a full-screen textured quad,
//! which is the primary method for applying post-processing effects.

use praxis_utils::{debug, eyre, info, trace, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::graphics::vertex_input::Vertex,
};

/// Vertex data for a full-screen quad.
///
/// This is a simplified vertex format containing only position and UV coordinates,
/// which is all that's needed for post-processing effects.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, Vertex)]
pub struct QuadVertex {
    /// Position in clip space (NDC).
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],

    /// UV texture coordinates.
    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2],
}

impl QuadVertex {
    /// Creates a new quad vertex.
    ///
    /// # Arguments
    ///
    /// * `position` - Position in clip space [-1, 1]
    /// * `uv` - UV coordinates [0, 1]
    pub fn new(position: [f32; 2], uv: [f32; 2]) -> Self {
        Self { position, uv }
    }
}

/// A full-screen quad for post-processing.
///
/// This struct manages the vertex and index buffers for a full-screen quad
/// that covers the entire viewport. The quad is defined in clip space
/// (normalized device coordinates) ranging from -1 to 1.
///
/// # Coordinate System
///
/// ```text
/// Clip Space (NDC):          UV Space:
///
///   y                          v
///   ^                          ^
///   │                          │
/// 1 │  (-1,1)──────(1,1)     1 │  (0,1)──────(1,1)
///   │    │           │         │    │          │
///   │    │           │         │    │          │
/// 0 ├────┼───────────┼─> x   0 │  (0,0)──────(1,0)
///   │    │           │         └────────────────> u
///   │    │           │           0              1
///-1 │  (-1,-1)─────(1,-1)
///   │
/// ```
///
/// # Usage
///
/// ```rust,no_run
/// # use praxis_graphics::post_process::FullScreenQuad;
/// # use std::sync::Arc;
/// # use vulkano::memory::allocator::StandardMemoryAllocator;
/// # fn example(memory_allocator: Arc<StandardMemoryAllocator>) -> praxis_utils::Result<()> {
/// let quad = FullScreenQuad::new(memory_allocator)?;
///
/// // In render pass:
/// // builder
/// //     .bind_vertex_buffers(0, quad.vertex_buffer().clone())
/// //     .bind_index_buffer(quad.index_buffer().clone())
/// //     .draw_indexed(quad.index_count(), 1, 0, 0, 0)?;
/// # Ok(())
/// # }
/// ```
pub struct FullScreenQuad {
    vertex_buffer: Subbuffer<[QuadVertex]>,
    index_buffer: Subbuffer<[u32]>,
    index_count: u32,
}

impl FullScreenQuad {
    /// Creates a new full-screen quad.
    ///
    /// # Arguments
    ///
    /// * `memory_allocator` - Allocator for buffer memory
    ///
    /// # Errors
    ///
    /// Returns an error if buffer creation fails.
    pub fn new(memory_allocator: Arc<StandardMemoryAllocator>) -> Result<Self> {
        debug!("Creating full-screen quad for post-processing");

        // Define vertices for a full-screen quad in clip space
        // Two triangles forming a quad covering [-1, 1] in x and y
        let vertices = vec![
            // Bottom-left
            QuadVertex::new([-1.0, -1.0], [0.0, 0.0]),
            // Bottom-right
            QuadVertex::new([1.0, -1.0], [1.0, 0.0]),
            // Top-right
            QuadVertex::new([1.0, 1.0], [1.0, 1.0]),
            // Top-left
            QuadVertex::new([-1.0, 1.0], [0.0, 1.0]),
        ];

        // Define indices for two triangles
        let indices: Vec<u32> = vec![
            0, 1, 2, // First triangle (bottom-left, bottom-right, top-right)
            2, 3, 0, // Second triangle (top-right, top-left, bottom-left)
        ];

        let index_count = indices.len() as u32;

        trace!("Creating vertex buffer with {} vertices", vertices.len());
        let vertex_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )
        .map_err(|e| eyre::eyre!("Failed to create quad vertex buffer: {}", e))?;

        trace!("Creating index buffer with {} indices", index_count);
        let index_buffer = Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices,
        )
        .map_err(|e| eyre::eyre!("Failed to create quad index buffer: {}", e))?;

        info!("Full-screen quad created successfully");

        Ok(Self {
            vertex_buffer,
            index_buffer,
            index_count,
        })
    }

    /// Returns a reference to the vertex buffer.
    pub fn vertex_buffer(&self) -> &Subbuffer<[QuadVertex]> {
        &self.vertex_buffer
    }

    /// Returns a reference to the index buffer.
    pub fn index_buffer(&self) -> &Subbuffer<[u32]> {
        &self.index_buffer
    }

    /// Returns the number of indices.
    pub fn index_count(&self) -> u32 {
        self.index_count
    }
}
