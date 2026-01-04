//! Procedural texture generation integration for the graphics system.
//!
//! This module provides integration between the procedural texture generation
//! system and the graphics rendering system. It allows generated textures to
//! be used directly in rendering.

use crate::texture::Texture;
use praxis_procedural::{
    ProceduralTextureCache, ProceduralTextureGenerator, TextureCacheKey, TextureGenerationParams,
    TextureGraph,
};
use praxis_utils::{debug, info, trace, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        allocator::CommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        CopyBufferToImageInfo,
    },
    descriptor_set::allocator::DescriptorSetAllocator,
    device::{Device, Queue},
    format::Format,
    image::{
        sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
        view::ImageView,
        Image, ImageCreateInfo, ImageType, ImageUsage,
    },
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
    sync::{self, GpuFuture},
};

/// Manager for procedural texture generation and caching.
///
/// This manager combines the procedural texture generator with caching
/// and provides a simple interface for generating and using procedural
/// textures in rendering.
pub struct ProceduralTextureManager {
    generator: ProceduralTextureGenerator,
    cache: ProceduralTextureCache,
    memory_allocator: Arc<dyn MemoryAllocator>,
    command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
    queue: Arc<Queue>,
}

impl ProceduralTextureManager {
    /// Creates a new procedural texture manager.
    ///
    /// # Arguments
    ///
    /// * `device` - Vulkan device
    /// * `queue` - Queue for GPU operations
    /// * `memory_allocator` - Memory allocator
    /// * `command_buffer_allocator` - Command buffer allocator
    /// * `descriptor_set_allocator` - Descriptor set allocator
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        memory_allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        descriptor_set_allocator: Arc<dyn DescriptorSetAllocator>,
    ) -> Self {
        info!("Created procedural texture manager");

        Self {
            generator: ProceduralTextureGenerator::new(
                device,
                queue.clone(),
                memory_allocator.clone(),
                command_buffer_allocator.clone(),
                descriptor_set_allocator,
            ),
            cache: ProceduralTextureCache::with_defaults(),
            memory_allocator,
            command_buffer_allocator,
            queue,
        }
    }

    /// Generates a texture from a texture graph.
    ///
    /// If the texture has been generated before with the same parameters,
    /// it will be retrieved from the cache instead of regenerating.
    ///
    /// # Arguments
    ///
    /// * `graph` - Texture graph describing the texture
    /// * `params` - Generation parameters (width, height, seed)
    ///
    /// # Returns
    ///
    /// A GPU texture ready for rendering
    pub fn generate_texture(
        &mut self,
        graph: &TextureGraph,
        params: TextureGenerationParams,
    ) -> Result<Texture> {
        let cache_key = TextureCacheKey::new(graph, params);

        let data = if let Some(cached_data) = self.cache.get(&cache_key) {
            debug!(
                "Using cached procedural texture ({}x{})",
                params.width, params.height
            );
            cached_data
        } else {
            debug!(
                "Generating new procedural texture ({}x{})",
                params.width, params.height
            );
            let data = self.generator.generate(graph, params)?;
            self.cache
                .insert(cache_key, data.clone(), params.width, params.height);
            data
        };

        self.create_texture_from_data(&data, params.width, params.height)
    }

    /// Generates a texture and does not cache it.
    ///
    /// Useful for one-off textures or when caching is not desired.
    pub fn generate_texture_uncached(
        &mut self,
        graph: &TextureGraph,
        params: TextureGenerationParams,
    ) -> Result<Texture> {
        let data = self.generator.generate(graph, params)?;
        self.create_texture_from_data(&data, params.width, params.height)
    }

    /// Clears the texture cache.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Gets cache statistics.
    pub fn cache_statistics(&self) -> praxis_procedural::CacheStatistics {
        self.cache.statistics()
    }

    /// Resets cache statistics.
    pub fn reset_cache_statistics(&mut self) {
        self.cache.reset_statistics();
    }

    /// Returns the number of cached textures.
    pub fn cached_texture_count(&self) -> usize {
        self.cache.len()
    }

    /// Returns the memory used by cached textures in bytes.
    pub fn cache_memory_usage(&self) -> usize {
        self.cache.memory_usage()
    }

    fn create_texture_from_data(&self, data: &[u8], width: u32, height: u32) -> Result<Texture> {
        trace!(
            "Uploading procedural texture to GPU: {}x{} ({} bytes)",
            width,
            height,
            data.len()
        );

        let image = Image::new(
            self.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_SRGB,
                extent: [width, height, 1],
                usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create texture image: {}", e))?;

        let buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
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
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create staging buffer: {}", e))?;

        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create command buffer: {}", e))?;

        builder
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(buffer, image.clone()))
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to copy buffer to image: {}", e))?;

        let command_buffer = builder
            .build()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to build command buffer: {}", e))?;

        let future = sync::now(self.queue.device().clone())
            .then_execute(self.queue.clone(), command_buffer)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to execute command buffer: {}", e))?
            .then_signal_fence_and_flush()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to flush command buffer: {}", e))?;

        future
            .wait(None)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to wait for texture upload: {}", e))?;

        let view = ImageView::new_default(image.clone())
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to create image view: {}", e))?;

        let sampler = Sampler::new(
            self.queue.device().clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::Repeat; 3],
                mipmap_mode: vulkano::image::sampler::SamplerMipmapMode::Linear,
                ..Default::default()
            },
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create sampler: {}", e))?;

        trace!("Successfully created procedural texture {}x{}", width, height);

        Ok(Texture {
            image,
            view,
            sampler,
            width,
            height,
        })
    }
}
