//! Texture loading and management for the graphics system.
//!
//! This module provides functionality for loading textures from common image formats
//! (PNG, JPEG) and managing them on the GPU. Textures are wrapped in Vulkan image
//! objects with associated samplers for texture filtering.

use praxis_utils::{debug, eyre, info, trace, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        allocator::CommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        CopyBufferToImageInfo,
    },
    device::Queue,
    format::Format,
    image::{
        sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
        view::ImageView,
        Image, ImageCreateInfo, ImageType, ImageUsage,
    },
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
    sync::{self, GpuFuture},
};

/// GPU-side texture data containing image, view, and sampler.
///
/// This structure wraps a Vulkan image, image view, and sampler for use in rendering.
/// The image view allows shaders to access the image data, and the sampler defines
/// how the texture is filtered and wrapped.
#[derive(Clone)]
pub struct Texture {
    /// Vulkan image containing the texture data.
    pub image: Arc<Image>,

    /// Image view for shader access.
    pub view: Arc<ImageView>,

    /// Sampler for texture filtering and wrapping.
    pub sampler: Arc<Sampler>,

    /// Width of the texture in pixels.
    pub width: u32,

    /// Height of the texture in pixels.
    pub height: u32,
}

impl Texture {
    /// Creates a new texture from raw RGBA8 pixel data.
    ///
    /// This function uploads pixel data to the GPU and creates the necessary
    /// Vulkan objects for texture sampling.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating GPU resources
    /// * `command_buffer_allocator` - Allocator for command buffers
    /// * `queue` - Queue for submitting upload commands
    /// * `width` - Width of the texture in pixels
    /// * `height` - Height of the texture in pixels
    /// * `data` - RGBA8 pixel data (4 bytes per pixel, row-major)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Image creation fails
    /// - Buffer upload fails
    /// - Command buffer submission fails
    pub fn from_rgba8(
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
        width: u32,
        height: u32,
        data: Vec<u8>,
    ) -> Result<Self> {
        trace!(
            "Creating texture from RGBA8 data: {}x{} ({} bytes)",
            width,
            height,
            data.len()
        );

        // Create the GPU image
        let image = Image::new(
            allocator.clone(),
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
        .map_err(|e| eyre::eyre!("Failed to create texture image: {}", e))?;

        // Create a staging buffer for uploading data
        let buffer = Buffer::from_iter(
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
            data,
        )
        .map_err(|e| eyre::eyre!("Failed to create staging buffer: {}", e))?;

        // Create a command buffer to copy data to the image
        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| eyre::eyre!("Failed to create command buffer: {}", e))?;

        builder
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(buffer, image.clone()))
            .map_err(|e| eyre::eyre!("Failed to copy buffer to image: {}", e))?;

        let command_buffer = builder
            .build()
            .map_err(|e| eyre::eyre!("Failed to build command buffer: {}", e))?;

        // Submit the command buffer and wait for completion
        let future = sync::now(queue.device().clone())
            .then_execute(queue.clone(), command_buffer)
            .map_err(|e| eyre::eyre!("Failed to execute command buffer: {}", e))?
            .then_signal_fence_and_flush()
            .map_err(|e| eyre::eyre!("Failed to flush command buffer: {}", e))?;

        future
            .wait(None)
            .map_err(|e| eyre::eyre!("Failed to wait for texture upload: {}", e))?;

        // Create image view for shader access
        let view = ImageView::new_default(image.clone())
            .map_err(|e| eyre::eyre!("Failed to create image view: {}", e))?;

        // Create sampler for texture filtering
        let sampler = Sampler::new(
            queue.device().clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::Repeat; 3],
                mipmap_mode: vulkano::image::sampler::SamplerMipmapMode::Linear,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create sampler: {}", e))?;

        trace!("Successfully created texture {}x{}", width, height);

        Ok(Self {
            image,
            view,
            sampler,
            width,
            height,
        })
    }

    /// Loads a texture from an image file.
    ///
    /// Supports common image formats including PNG and JPEG through the `image` crate.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating GPU resources
    /// * `command_buffer_allocator` - Allocator for command buffers
    /// * `queue` - Queue for submitting upload commands
    /// * `path` - Path to the image file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File cannot be read
    /// - Image format is unsupported
    /// - GPU upload fails
    pub fn from_file(
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
        path: impl AsRef<Path>,
    ) -> Result<Self> {
        let path = path.as_ref();
        debug!("Loading texture from file: {}", path.display());

        // Load and decode the image
        let img = image::open(path)
            .map_err(|e| eyre::eyre!("Failed to open image file '{}': {}", path.display(), e))?;

        // Convert to RGBA8
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba.into_raw();

        info!(
            "Loaded texture from '{}': {}x{}",
            path.display(),
            width,
            height
        );

        Self::from_rgba8(
            allocator,
            command_buffer_allocator,
            queue,
            width,
            height,
            data,
        )
    }

    /// Creates a 1x1 white texture.
    ///
    /// This is useful as a default texture when no texture is specified.
    pub fn white(
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
    ) -> Result<Self> {
        trace!("Creating default white texture");
        Self::from_rgba8(
            allocator,
            command_buffer_allocator,
            queue,
            1,
            1,
            vec![255, 255, 255, 255],
        )
    }
}

/// Texture asset manager that caches loaded textures.
///
/// This manager maintains a cache of textures by name, avoiding redundant
/// loads of the same texture file. It provides convenient methods for
/// loading textures from files and managing the texture cache.
pub struct TextureManager {
    /// Map of texture name to GPU texture.
    textures: HashMap<String, Texture>,

    /// Memory allocator for creating GPU resources.
    allocator: Arc<dyn MemoryAllocator>,

    /// Command buffer allocator for upload commands.
    command_buffer_allocator: Arc<dyn CommandBufferAllocator>,

    /// Queue for submitting GPU commands.
    queue: Arc<Queue>,
}

impl TextureManager {
    /// Creates a new texture manager.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating GPU resources
    /// * `command_buffer_allocator` - Allocator for command buffers
    /// * `queue` - Queue for submitting GPU commands
    pub fn new(
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
    ) -> Self {
        Self {
            textures: HashMap::new(),
            allocator,
            command_buffer_allocator,
            queue,
        }
    }

    /// Loads a texture from a file and caches it.
    ///
    /// If a texture with the same name already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for the texture
    /// * `path` - Path to the image file
    ///
    /// # Errors
    ///
    /// Returns an error if texture loading fails.
    pub fn load_texture(&mut self, name: impl Into<String>, path: impl AsRef<Path>) -> Result<()> {
        let name = name.into();
        let path = path.as_ref();

        debug!("Loading texture '{}' from '{}'", name, path.display());

        let texture = Texture::from_file(
            self.allocator.clone(),
            self.command_buffer_allocator.clone(),
            self.queue.clone(),
            path,
        )?;

        self.textures.insert(name.clone(), texture);
        info!("Texture '{}' loaded and cached", name);

        Ok(())
    }

    /// Loads a texture from raw RGBA8 bytes and caches it.
    ///
    /// This is useful for procedurally generated textures or textures loaded
    /// from non-file sources.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for the texture
    /// * `data` - RGBA8 pixel data (4 bytes per pixel, row-major)
    /// * `width` - Width of the texture in pixels
    /// * `height` - Height of the texture in pixels
    ///
    /// # Errors
    ///
    /// Returns an error if texture creation fails.
    pub fn load_texture_from_bytes(
        &mut self,
        name: impl Into<String>,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<()> {
        let name = name.into();

        debug!(
            "Creating texture '{}' from bytes ({}x{})",
            name, width, height
        );

        let texture = Texture::from_rgba8(
            self.allocator.clone(),
            self.command_buffer_allocator.clone(),
            self.queue.clone(),
            width,
            height,
            data.to_vec(),
        )?;

        self.textures.insert(name.clone(), texture);
        info!("Texture '{}' created and cached", name);

        Ok(())
    }

    /// Adds a pre-created texture to the cache.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for the texture
    /// * `texture` - The texture to add
    pub fn add_texture(&mut self, name: impl Into<String>, texture: Texture) {
        let name = name.into();
        debug!("Adding texture '{}' to cache", name);
        self.textures.insert(name, texture);
    }

    /// Gets a texture by name.
    ///
    /// Returns `None` if the texture doesn't exist.
    pub fn get_texture(&self, name: &str) -> Option<&Texture> {
        self.textures.get(name)
    }

    /// Checks if a texture exists in the cache.
    pub fn contains_texture(&self, name: &str) -> bool {
        self.textures.contains_key(name)
    }

    /// Removes a texture from the cache.
    ///
    /// Returns `true` if the texture existed and was removed.
    pub fn remove_texture(&mut self, name: &str) -> bool {
        self.textures.remove(name).is_some()
    }

    /// Returns the number of cached textures.
    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    /// Clears all cached textures.
    pub fn clear(&mut self) {
        debug!("Clearing {} cached textures", self.textures.len());
        self.textures.clear();
    }

    /// Creates a default white texture and adds it to the cache.
    ///
    /// This is useful for objects that don't have a texture assigned.
    pub fn create_default_white_texture(&mut self) -> Result<()> {
        let texture = Texture::white(
            self.allocator.clone(),
            self.command_buffer_allocator.clone(),
            self.queue.clone(),
        )?;
        self.add_texture("_default_white", texture);
        Ok(())
    }
}
