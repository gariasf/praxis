//! Dynamic uniform buffer management with ring buffer allocation.
//!
//! This module provides efficient per-frame uniform buffer updates using a ring buffer
//! strategy. Instead of creating a new descriptor set and uniform buffer for each object
//! per frame, we use a single large buffer with dynamic offsets.
//!
//! # Architecture
//!
//! The ring buffer cycles through multiple frames worth of data to avoid CPU-GPU
//! synchronization stalls. Each frame gets its own region of the buffer, and we
//! cycle through these regions as frames progress.
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │         Dynamic Uniform Buffer          │
//! ├─────────────────────────────────────────┤
//! │ Frame 0 │ Frame 1 │ Frame 2 │ Frame 0..│
//! │  Obj 0  │  Obj 0  │  Obj 0  │  Obj 0   │
//! │  Obj 1  │  Obj 1  │  Obj 1  │  Obj 1   │
//! │  Obj 2  │  Obj 2  │  Obj 2  │  Obj 2   │
//! │   ...   │   ...   │   ...   │   ...    │
//! └─────────────────────────────────────────┘
//!          ▲                       ▲
//!    Current Frame            Next Frame
//! ```

use praxis_math::Mat4;
use praxis_utils::{debug, eyre, info, trace, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    device::Device,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

/// Uniforms for view and projection matrices (shared across all objects in a frame).
///
/// These matrices are constant for all objects in a single frame, so we store them
/// separately from the per-object model matrices.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewProjectionUniforms {
    /// Camera view matrix (world → view).
    pub view: [[f32; 4]; 4],
    /// Camera projection matrix (view → clip).
    pub proj: [[f32; 4]; 4],
    /// Camera position in world space.
    pub camera_position: [f32; 3],
    /// Padding for alignment (total size: 140 bytes).
    pub _padding: f32,
}

impl ViewProjectionUniforms {
    /// Creates new view-projection uniforms with camera position extracted from view matrix.
    ///
    /// # Arguments
    ///
    /// * `view` - Camera view matrix (world → view)
    /// * `proj` - Camera projection matrix (view → clip)
    ///
    /// # Returns
    ///
    /// A new `ViewProjectionUniforms` with camera position extracted from the inverse view matrix.
    pub fn new(view: Mat4, proj: Mat4) -> Self {
        // Extract camera position from inverse of view matrix
        // The camera position is the translation component of the inverse view matrix
        let view_inverse = view.inverse();
        let camera_position = [
            view_inverse.w_axis.x,
            view_inverse.w_axis.y,
            view_inverse.w_axis.z,
        ];

        Self {
            view: view.to_cols_array_2d(),
            proj: proj.to_cols_array_2d(),
            camera_position,
            _padding: 0.0,
        }
    }
}

/// Per-object uniform data containing the model matrix.
///
/// This is stored in the dynamic uniform buffer with proper alignment.
/// Each object gets its own aligned region in the buffer.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelUniforms {
    /// Model matrix (model → world).
    pub model: [[f32; 4]; 4],
}

/// Dynamic uniform buffer manager using a ring buffer strategy.
///
/// This manages a large buffer divided into frames, where each frame can hold
/// multiple objects' uniform data. The buffer is persistently mapped for efficient
/// CPU writes.
pub struct DynamicUniformBuffer {
    /// The underlying Vulkan buffer (host-visible and persistently mapped).
    buffer: Subbuffer<[u8]>,
    /// Number of frames in flight (ring buffer size).
    frames_in_flight: usize,
    /// Maximum number of objects per frame.
    max_objects_per_frame: usize,
    /// Current frame index (cycles 0..frames_in_flight).
    current_frame: usize,
    /// Aligned size of each object's uniform data.
    aligned_object_size: usize,
    /// Size of each frame's region in the buffer.
    frame_stride: usize,
}

impl DynamicUniformBuffer {
    /// Creates a new dynamic uniform buffer.
    ///
    /// # Arguments
    ///
    /// * `device` - The Vulkan device
    /// * `memory_allocator` - Memory allocator for buffer creation
    /// * `frames_in_flight` - Number of frames in flight (typically 2-3)
    /// * `max_objects_per_frame` - Maximum number of objects that can be drawn per frame
    ///
    /// # Returns
    ///
    /// A new `DynamicUniformBuffer` ready for use.
    pub fn new(
        device: &Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        frames_in_flight: usize,
        max_objects_per_frame: usize,
    ) -> Result<Self> {
        info!(
            "Creating dynamic uniform buffer: {} frames, {} objects per frame",
            frames_in_flight, max_objects_per_frame
        );

        // Get device limits for uniform buffer alignment
        let min_alignment = device
            .physical_device()
            .properties()
            .min_uniform_buffer_offset_alignment
            .as_devicesize() as usize;

        debug!(
            "Minimum uniform buffer offset alignment: {} bytes",
            min_alignment
        );

        // Calculate aligned size for each object's uniform data
        let object_size = std::mem::size_of::<ModelUniforms>();
        let aligned_object_size = Self::align_up(object_size, min_alignment);

        debug!(
            "Object uniform size: {} bytes, aligned: {} bytes",
            object_size, aligned_object_size
        );

        // Calculate total buffer size
        let frame_stride = aligned_object_size * max_objects_per_frame;
        let total_size = frame_stride * frames_in_flight;

        info!(
            "Allocating uniform buffer: {} bytes ({} bytes per frame)",
            total_size, frame_stride
        );

        // Create a single large buffer for all frames and objects
        let buffer = Buffer::new_slice::<u8>(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            total_size as u64,
        )
        .map_err(|e| eyre::eyre!("Failed to create dynamic uniform buffer: {}", e))?;

        info!("Dynamic uniform buffer created successfully");

        Ok(Self {
            buffer,
            frames_in_flight,
            max_objects_per_frame,
            current_frame: 0,
            aligned_object_size,
            frame_stride,
        })
    }

    /// Aligns a size up to the next multiple of alignment.
    fn align_up(size: usize, alignment: usize) -> usize {
        (size + alignment - 1) & !(alignment - 1)
    }

    /// Advances to the next frame in the ring buffer.
    ///
    /// Should be called once per frame before writing new uniform data.
    pub fn next_frame(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.frames_in_flight;
        trace!("Advanced to frame {}", self.current_frame);
    }

    /// Writes model matrices to the buffer for the current frame.
    ///
    /// # Arguments
    ///
    /// * `models` - Slice of model matrices to write
    ///
    /// # Returns
    ///
    /// The number of objects written
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer write fails or too many objects are provided.
    pub fn write_models(&self, models: &[Mat4]) -> Result<usize> {
        if models.len() > self.max_objects_per_frame {
            return Err(eyre::eyre!(
                "Too many objects: {} > {}",
                models.len(),
                self.max_objects_per_frame
            ));
        }

        if models.is_empty() {
            return Ok(0);
        }

        trace!(
            "Writing {} model matrices to frame {}",
            models.len(),
            self.current_frame
        );

        // Get write access to the buffer
        let mut write_lock = self
            .buffer
            .write()
            .map_err(|e| eyre::eyre!("Failed to lock uniform buffer for writing: {}", e))?;

        // Calculate starting offset for current frame
        let frame_offset = self.current_frame * self.frame_stride;

        // Write each model matrix at its aligned offset
        for (i, model) in models.iter().enumerate() {
            let object_offset = frame_offset + (i * self.aligned_object_size);

            let uniforms = ModelUniforms {
                model: model.to_cols_array_2d(),
            };

            let bytes = bytemuck::bytes_of(&uniforms);
            let dst = &mut write_lock[object_offset..object_offset + bytes.len()];
            dst.copy_from_slice(bytes);
        }

        Ok(models.len())
    }

    /// Returns the offset for a specific object in the current frame.
    ///
    /// This offset should be used with dynamic descriptor sets.
    ///
    /// # Arguments
    ///
    /// * `object_index` - Index of the object (0..max_objects_per_frame)
    pub fn get_dynamic_offset(&self, object_index: usize) -> u32 {
        let frame_offset = self.current_frame * self.frame_stride;
        let offset = frame_offset + (object_index * self.aligned_object_size);
        offset as u32
    }

    /// Returns the underlying buffer.
    pub fn buffer(&self) -> &Subbuffer<[u8]> {
        &self.buffer
    }

    /// Returns the aligned size of each object's uniform data.
    pub fn aligned_object_size(&self) -> usize {
        self.aligned_object_size
    }

    /// Returns the current frame index.
    pub fn current_frame(&self) -> usize {
        self.current_frame
    }
}
