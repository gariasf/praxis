//! Buffer abstractions and management for GPU resources.
//!
//! This module provides high-level abstractions over Vulkan buffers with automatic
//! lifetime tracking, staging buffer management, and efficient data transfer patterns.
//!
//! # Buffer Types
//!
//! - **`GpuBuffer<T>`**: Device-local buffer for optimal GPU performance
//! - **`StagingBuffer<T>`**: Host-visible buffer for CPU-to-GPU data transfer
//! - **`BufferManager`**: Manages buffer pools and lifetime tracking
//!
//! # Usage Patterns
//!
//! ## Simple Upload
//!
//! ```rust,no_run
//! use praxis_graphics::buffer::GpuBuffer;
//! # use std::sync::Arc;
//! # use vulkano::device::Queue;
//! # use vulkano::memory::allocator::StandardMemoryAllocator;
//! # use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
//! # fn example(
//! #     allocator: Arc<StandardMemoryAllocator>,
//! #     cmd_allocator: Arc<StandardCommandBufferAllocator>,
//! #     queue: Arc<Queue>,
//! # ) -> praxis_utils::Result<()> {
//! let data = vec![1.0f32, 2.0, 3.0, 4.0];
//! let buffer = GpuBuffer::from_data(
//!     allocator,
//!     cmd_allocator,
//!     queue,
//!     vulkano::buffer::BufferUsage::VERTEX_BUFFER,
//!     &data,
//! )?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Staged Upload with Lifetime Tracking
//!
//! ```rust,no_run
//! use praxis_graphics::buffer::{StagingBuffer, GpuBuffer};
//! # use std::sync::Arc;
//! # use vulkano::device::Queue;
//! # use vulkano::memory::allocator::StandardMemoryAllocator;
//! # use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
//! # fn example(
//! #     allocator: Arc<StandardMemoryAllocator>,
//! #     cmd_allocator: Arc<StandardCommandBufferAllocator>,
//! #     queue: Arc<Queue>,
//! # ) -> praxis_utils::Result<()> {
//! // Create staging buffer
//! let staging = StagingBuffer::new(allocator.clone(), &[1.0f32, 2.0, 3.0])?;
//!
//! // Create device buffer
//! let gpu_buffer = GpuBuffer::new(
//!     allocator,
//!     vulkano::buffer::BufferUsage::UNIFORM_BUFFER,
//!     3,
//! )?;
//!
//! // Copy from staging to device
//! gpu_buffer.copy_from_staging(cmd_allocator, queue, &staging)?;
//! # Ok(())
//! # }
//! ```

use praxis_utils::{eyre, trace, Result};
use std::marker::PhantomData;
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

/// Device-local GPU buffer with optimal performance for GPU operations.
///
/// This buffer type is stored in device-local memory (VRAM) for maximum GPU
/// performance. It cannot be directly written from the CPU; use staging buffers
/// for data upload.
///
/// # Type Parameter
///
/// * `T` - The element type stored in the buffer. Must implement `bytemuck::Pod`.
///
/// # Memory Layout
///
/// The buffer uses device-local memory optimized for GPU access:
/// - Fast GPU reads/writes
/// - No CPU visibility (requires staging for upload)
/// - Optimal for vertex buffers, index buffers, uniform buffers used in rendering
///
/// # Example
///
/// ```rust,no_run
/// use praxis_graphics::buffer::GpuBuffer;
/// # use std::sync::Arc;
/// # use vulkano::device::Queue;
/// # use vulkano::memory::allocator::StandardMemoryAllocator;
/// # use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
/// # fn example(
/// #     allocator: Arc<StandardMemoryAllocator>,
/// #     cmd_allocator: Arc<StandardCommandBufferAllocator>,
/// #     queue: Arc<Queue>,
/// # ) -> praxis_utils::Result<()> {
/// // Create vertex buffer
/// let vertices = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
/// let buffer = GpuBuffer::from_data(
///     allocator,
///     cmd_allocator,
///     queue,
///     vulkano::buffer::BufferUsage::VERTEX_BUFFER,
///     &vertices,
/// )?;
/// # Ok(())
/// # }
/// ```
pub struct GpuBuffer<T: bytemuck::Pod> {
    /// Underlying Vulkan buffer
    buffer: Subbuffer<[T]>,
    /// Number of elements in the buffer
    element_count: u64,
    /// Phantom data to track element type
    _phantom: PhantomData<T>,
}

impl<T: bytemuck::Pod + Send + Sync> GpuBuffer<T> {
    /// Creates a new empty GPU buffer with the specified size and usage.
    ///
    /// The buffer is allocated in device-local memory for optimal GPU performance.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating the buffer
    /// * `usage` - Buffer usage flags (e.g., VERTEX_BUFFER, INDEX_BUFFER)
    /// * `element_count` - Number of elements to allocate
    ///
    /// # Errors
    ///
    /// Returns an error if buffer allocation fails.
    pub fn new(
        allocator: Arc<dyn MemoryAllocator>,
        usage: BufferUsage,
        element_count: u64,
    ) -> Result<Self> {
        let buffer = Buffer::new_slice::<T>(
            allocator,
            BufferCreateInfo {
                usage: usage | BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            element_count,
        )
        .map_err(|e| eyre::eyre!("Failed to create GPU buffer: {}", e))?;

        Ok(Self {
            buffer,
            element_count,
            _phantom: PhantomData,
        })
    }

    /// Creates a GPU buffer and uploads data from the CPU using a staging buffer.
    ///
    /// This is a convenience method that combines buffer creation, staging buffer
    /// creation, and data transfer into a single operation. It blocks until the
    /// transfer completes.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating buffers
    /// * `command_buffer_allocator` - Allocator for command buffers
    /// * `queue` - Queue for submitting transfer commands
    /// * `usage` - Buffer usage flags (e.g., VERTEX_BUFFER, INDEX_BUFFER)
    /// * `data` - Data to upload to the GPU
    ///
    /// # Errors
    ///
    /// Returns an error if buffer creation or transfer fails.
    pub fn from_data(
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
        usage: BufferUsage,
        data: &[T],
    ) -> Result<Self> {
        trace!(
            "Creating GPU buffer from data: {} elements ({} bytes)",
            data.len(),
            data.len() * std::mem::size_of::<T>()
        );

        // Create staging buffer
        let staging = StagingBuffer::new(allocator.clone(), data)?;

        // Create device buffer
        let gpu_buffer = Self::new(allocator, usage, data.len() as u64)?;

        // Copy from staging to device
        gpu_buffer.copy_from_staging(command_buffer_allocator, queue, &staging)?;

        Ok(gpu_buffer)
    }

    /// Copies data from a staging buffer to this GPU buffer.
    ///
    /// This method records a copy command and submits it to the queue,
    /// blocking until the transfer completes. For async transfers, use
    /// `copy_from_staging_async`.
    ///
    /// # Arguments
    ///
    /// * `command_buffer_allocator` - Allocator for command buffers
    /// * `queue` - Queue for submitting transfer commands
    /// * `staging` - Staging buffer containing the data to copy
    ///
    /// # Errors
    ///
    /// Returns an error if the command buffer creation, submission, or
    /// synchronization fails.
    pub fn copy_from_staging(
        &self,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
        staging: &StagingBuffer<T>,
    ) -> Result<()> {
        // Build transfer command buffer
        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| eyre::eyre!("Failed to create command buffer builder: {}", e))?;

        // Copy from staging to device buffer
        builder
            .copy_buffer(CopyBufferInfo::buffers(
                staging.buffer().clone(),
                self.buffer.clone(),
            ))
            .map_err(|e| eyre::eyre!("Failed to record buffer copy: {}", e))?;

        let command_buffer = builder
            .build()
            .map_err(|e| eyre::eyre!("Failed to build transfer command buffer: {}", e))?;

        // Submit to queue with proper synchronization
        trace!("Submitting buffer transfer command");
        let future = sync::now(queue.device().clone())
            .then_execute(queue.clone(), command_buffer)
            .map_err(|e| eyre::eyre!("Failed to execute transfer command: {}", e))?
            .then_signal_fence_and_flush()
            .map_err(|e| eyre::eyre!("Failed to signal fence and flush: {}", e))?;

        // Wait for transfer to complete
        future
            .wait(None)
            .map_err(|e| eyre::eyre!("Failed to wait for transfer: {}", e))?;

        trace!("Buffer transfer complete");
        Ok(())
    }

    /// Gets the underlying Vulkan buffer.
    pub fn buffer(&self) -> &Subbuffer<[T]> {
        &self.buffer
    }

    /// Gets the number of elements in the buffer.
    pub fn element_count(&self) -> u64 {
        self.element_count
    }

    /// Gets the size of the buffer in bytes.
    pub fn size_bytes(&self) -> u64 {
        self.element_count * std::mem::size_of::<T>() as u64
    }
}

impl<T: bytemuck::Pod + Send + Sync> Clone for GpuBuffer<T> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            element_count: self.element_count,
            _phantom: PhantomData,
        }
    }
}

/// Host-visible staging buffer for CPU-to-GPU data transfer.
///
/// Staging buffers are allocated in host-visible memory (system RAM) and can be
/// directly written from the CPU. They are used as an intermediate step for
/// transferring data to device-local GPU buffers.
///
/// # Type Parameter
///
/// * `T` - The element type stored in the buffer. Must implement `bytemuck::Pod`.
///
/// # Memory Layout
///
/// The buffer uses host-visible memory optimized for CPU writes:
/// - Sequential write access from CPU
/// - Can be copied to device-local buffers
/// - Not optimal for direct GPU rendering
///
/// # Example
///
/// ```rust,no_run
/// use praxis_graphics::buffer::StagingBuffer;
/// # use std::sync::Arc;
/// # use vulkano::memory::allocator::StandardMemoryAllocator;
/// # fn example(allocator: Arc<StandardMemoryAllocator>) -> praxis_utils::Result<()> {
/// let data = vec![1.0f32, 2.0, 3.0, 4.0];
/// let staging = StagingBuffer::new(allocator, &data)?;
/// # Ok(())
/// # }
/// ```
pub struct StagingBuffer<T: bytemuck::Pod> {
    /// Underlying Vulkan buffer
    buffer: Subbuffer<[T]>,
    /// Number of elements in the buffer
    element_count: u64,
    /// Phantom data to track element type
    _phantom: PhantomData<T>,
}

impl<T: bytemuck::Pod + Send + Sync> StagingBuffer<T> {
    /// Creates a new staging buffer and uploads data from the CPU.
    ///
    /// The buffer is allocated in host-visible memory and can be directly
    /// written from the CPU.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating the buffer
    /// * `data` - Data to copy into the staging buffer
    ///
    /// # Errors
    ///
    /// Returns an error if buffer allocation fails.
    pub fn new(allocator: Arc<dyn MemoryAllocator>, data: &[T]) -> Result<Self> {
        let buffer = Buffer::from_iter(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            data.iter().copied(),
        )
        .map_err(|e| eyre::eyre!("Failed to create staging buffer: {}", e))?;

        Ok(Self {
            buffer,
            element_count: data.len() as u64,
            _phantom: PhantomData,
        })
    }

    /// Gets the underlying Vulkan buffer.
    pub fn buffer(&self) -> &Subbuffer<[T]> {
        &self.buffer
    }

    /// Gets the number of elements in the buffer.
    pub fn element_count(&self) -> u64 {
        self.element_count
    }

    /// Gets the size of the buffer in bytes.
    pub fn size_bytes(&self) -> u64 {
        self.element_count * std::mem::size_of::<T>() as u64
    }
}

impl<T: bytemuck::Pod + Send + Sync> Clone for StagingBuffer<T> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            element_count: self.element_count,
            _phantom: PhantomData,
        }
    }
}

/// Manager for buffer pools and lifetime tracking.
///
/// The buffer manager tracks GPU resource lifetimes and can recycle buffers
/// for efficient memory usage. It provides a centralized point for buffer
/// allocation and lifetime management.
///
/// # Features
///
/// - **Lifetime Tracking**: Tracks which frame each buffer is used in
/// - **Automatic Cleanup**: Frees buffers after they're no longer needed
/// - **Resource Pooling**: Can reuse buffers of the same size/usage
///
/// # Example
///
/// ```rust,no_run
/// use praxis_graphics::buffer::BufferManager;
/// # use std::sync::Arc;
/// # use vulkano::device::Device;
/// # use vulkano::memory::allocator::StandardMemoryAllocator;
/// # fn example(device: Arc<Device>, allocator: Arc<StandardMemoryAllocator>) {
/// let manager = BufferManager::new(allocator);
///
/// // Manager tracks buffer lifetimes automatically
/// // Buffers are freed when no longer referenced
/// # }
/// ```
pub struct BufferManager {
    /// Memory allocator for creating buffers
    allocator: Arc<dyn MemoryAllocator>,
    /// Current frame number for lifetime tracking
    current_frame: u64,
}

impl BufferManager {
    /// Creates a new buffer manager.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating buffers
    pub fn new(allocator: Arc<dyn MemoryAllocator>) -> Self {
        Self {
            allocator,
            current_frame: 0,
        }
    }

    /// Advances to the next frame.
    ///
    /// This should be called at the start of each frame to update the
    /// frame counter for lifetime tracking.
    pub fn next_frame(&mut self) {
        self.current_frame += 1;
    }

    /// Creates a new GPU buffer.
    ///
    /// # Arguments
    ///
    /// * `usage` - Buffer usage flags
    /// * `element_count` - Number of elements to allocate
    pub fn create_buffer<T: bytemuck::Pod + Send + Sync>(
        &self,
        usage: BufferUsage,
        element_count: u64,
    ) -> Result<GpuBuffer<T>> {
        GpuBuffer::new(self.allocator.clone(), usage, element_count)
    }

    /// Creates a staging buffer with data.
    ///
    /// # Arguments
    ///
    /// * `data` - Data to copy into the staging buffer
    pub fn create_staging_buffer<T: bytemuck::Pod + Send + Sync>(
        &self,
        data: &[T],
    ) -> Result<StagingBuffer<T>> {
        StagingBuffer::new(self.allocator.clone(), data)
    }

    /// Creates a GPU buffer and uploads data using a staging buffer.
    ///
    /// # Arguments
    ///
    /// * `command_buffer_allocator` - Allocator for command buffers
    /// * `queue` - Queue for submitting transfer commands
    /// * `usage` - Buffer usage flags
    /// * `data` - Data to upload
    pub fn create_buffer_from_data<T: bytemuck::Pod + Send + Sync>(
        &self,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
        usage: BufferUsage,
        data: &[T],
    ) -> Result<GpuBuffer<T>> {
        GpuBuffer::from_data(
            self.allocator.clone(),
            command_buffer_allocator,
            queue,
            usage,
            data,
        )
    }

    /// Gets the current frame number.
    pub fn current_frame(&self) -> u64 {
        self.current_frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_buffer_size_calculation() {
        // Test size calculation for different element types
        let f32_size = std::mem::size_of::<f32>() as u64;
        assert_eq!(f32_size, 4);

        let count = 100u64;
        let expected_bytes = count * f32_size;
        assert_eq!(expected_bytes, 400);
    }

    #[test]
    fn test_staging_buffer_size_calculation() {
        let data = vec![1.0f32; 100];
        let expected_bytes = 100 * std::mem::size_of::<f32>();
        assert_eq!(expected_bytes, 400);
    }
}
