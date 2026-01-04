//! Texture loading and management for the graphics system.
//!
//! This module provides functionality for loading textures from common image formats
//! (PNG, JPEG) and managing them on the GPU. Textures are wrapped in Vulkan image
//! objects with associated samplers for texture filtering.

use praxis_utils::{debug, eyre, info, trace, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
        view::{ImageView, ImageViewCreateInfo, ImageViewType},
        Image, ImageAspects, ImageCreateInfo, ImageSubresourceRange, ImageType, ImageUsage,
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

    /// Creates a 1x1 flat normal map texture.
    ///
    /// This normal map represents a flat surface with normal pointing straight up
    /// in tangent space: (0, 0, 1) encoded as RGB (128, 128, 255).
    /// This is useful as a default when no normal map is specified.
    pub fn flat_normal(
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
    ) -> Result<Self> {
        trace!("Creating default flat normal map texture");
        Self::from_rgba8(
            allocator,
            command_buffer_allocator,
            queue,
            1,
            1,
            vec![128, 128, 255, 255],
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

    /// Map of file path to texture name for hot-reload support.
    path_to_name: HashMap<PathBuf, String>,

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
            path_to_name: HashMap::new(),
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
        self.path_to_name.insert(path.to_path_buf(), name.clone());
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
        let removed = self.textures.remove(name).is_some();
        if removed {
            self.path_to_name.retain(|_, v| v != name);
        }
        removed
    }

    /// Returns the number of cached textures.
    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    /// Clears all cached textures.
    pub fn clear(&mut self) {
        debug!("Clearing {} cached textures", self.textures.len());
        self.textures.clear();
        self.path_to_name.clear();
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

    /// Creates a default flat normal map texture and adds it to the cache.
    ///
    /// This is useful for objects that don't have a normal map assigned.
    /// The flat normal map represents (0, 0, 1) in tangent space (straight up).
    pub fn create_default_flat_normal(&mut self) -> Result<()> {
        let texture = Texture::flat_normal(
            self.allocator.clone(),
            self.command_buffer_allocator.clone(),
            self.queue.clone(),
        )?;
        self.add_texture("_default_flat_normal", texture);
        Ok(())
    }

    /// Reloads a texture from disk by its file path.
    ///
    /// This method is used for hot-reload functionality. If the file path
    /// corresponds to a loaded texture, it will be reloaded from disk and
    /// the GPU resource will be updated.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the texture file to reload
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if the texture was found and reloaded successfully,
    /// `Ok(false)` if the path doesn't correspond to any loaded texture,
    /// or an error if reloading failed.
    pub fn reload_texture(&mut self, path: impl AsRef<Path>) -> Result<bool> {
        let path = path.as_ref();
        
        if let Some(name) = self.path_to_name.get(path).cloned() {
            debug!("Reloading texture '{}' from '{}'", name, path.display());
            
            let texture = Texture::from_file(
                self.allocator.clone(),
                self.command_buffer_allocator.clone(),
                self.queue.clone(),
                path,
            )?;
            
            self.textures.insert(name.clone(), texture);
            info!("Texture '{}' reloaded successfully", name);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Cubemap face enumeration.
///
/// Defines the six faces of a cubemap in the standard Vulkan/OpenGL order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubemapFace {
    /// Positive X face (right).
    PositiveX = 0,
    /// Negative X face (left).
    NegativeX = 1,
    /// Positive Y face (top).
    PositiveY = 2,
    /// Negative Y face (bottom).
    NegativeY = 3,
    /// Positive Z face (front).
    PositiveZ = 4,
    /// Negative Z face (back).
    NegativeZ = 5,
}

impl CubemapFace {
    /// Returns all six cubemap faces in order.
    pub fn all() -> [Self; 6] {
        [
            Self::PositiveX,
            Self::NegativeX,
            Self::PositiveY,
            Self::NegativeY,
            Self::PositiveZ,
            Self::NegativeZ,
        ]
    }
}

/// GPU-side cubemap texture data.
///
/// A cubemap is a texture consisting of 6 square 2D images that represent the faces
/// of a cube. Cubemaps are commonly used for skyboxes and environment mapping.
#[derive(Clone)]
pub struct Cubemap {
    /// Vulkan image containing the cubemap data (6 layers).
    pub image: Arc<Image>,

    /// Image view for shader access (as a cube).
    pub view: Arc<ImageView>,

    /// Sampler for texture filtering and wrapping.
    pub sampler: Arc<Sampler>,

    /// Width and height of each cubemap face in pixels (must be square).
    pub face_size: u32,
}

impl Cubemap {
    /// Creates a new cubemap from raw RGBA8 pixel data for all six faces.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating GPU resources
    /// * `command_buffer_allocator` - Allocator for command buffers
    /// * `queue` - Queue for submitting upload commands
    /// * `face_size` - Width and height of each face in pixels (must be square)
    /// * `face_data` - RGBA8 pixel data for all 6 faces (in +X, -X, +Y, -Y, +Z, -Z order)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Image creation fails
    /// - Buffer upload fails
    /// - Command buffer submission fails
    pub fn from_faces(
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
        face_size: u32,
        face_data: [Vec<u8>; 6],
    ) -> Result<Self> {
        trace!(
            "Creating cubemap from face data: {}x{} per face",
            face_size,
            face_size
        );

        let image = Image::new(
            allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_SRGB,
                extent: [face_size, face_size, 1],
                array_layers: 6,
                usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create cubemap image: {}", e))?;

        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator.clone(),
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| eyre::eyre!("Failed to create command buffer: {}", e))?;

        for (face_index, data) in face_data.iter().enumerate() {
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
                data.iter().copied(),
            )
            .map_err(|e| {
                eyre::eyre!(
                    "Failed to create staging buffer for face {}: {}",
                    face_index,
                    e
                )
            })?;

            builder
                .copy_buffer_to_image(CopyBufferToImageInfo {
                    regions: [vulkano::command_buffer::BufferImageCopy {
                        buffer_offset: 0,
                        buffer_row_length: 0,
                        buffer_image_height: 0,
                        image_subresource: vulkano::image::ImageSubresourceLayers {
                            aspects: ImageAspects::COLOR,
                            mip_level: 0,
                            array_layers: face_index as u32..(face_index as u32 + 1),
                        },
                        image_offset: [0, 0, 0],
                        image_extent: [face_size, face_size, 1],
                        ..Default::default()
                    }]
                    .into(),
                    ..CopyBufferToImageInfo::buffer_image(buffer, image.clone())
                })
                .map_err(|e| {
                    eyre::eyre!(
                        "Failed to copy buffer to image for face {}: {}",
                        face_index,
                        e
                    )
                })?;
        }

        let command_buffer = builder
            .build()
            .map_err(|e| eyre::eyre!("Failed to build command buffer: {}", e))?;

        let future = sync::now(queue.device().clone())
            .then_execute(queue.clone(), command_buffer)
            .map_err(|e| eyre::eyre!("Failed to execute command buffer: {}", e))?
            .then_signal_fence_and_flush()
            .map_err(|e| eyre::eyre!("Failed to flush command buffer: {}", e))?;

        future
            .wait(None)
            .map_err(|e| eyre::eyre!("Failed to wait for cubemap upload: {}", e))?;

        let view = ImageView::new(
            image.clone(),
            ImageViewCreateInfo {
                view_type: ImageViewType::Cube,
                subresource_range: ImageSubresourceRange {
                    aspects: ImageAspects::COLOR,
                    mip_levels: 0..1,
                    array_layers: 0..6,
                },
                ..ImageViewCreateInfo::from_image(&image)
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create cubemap image view: {}", e))?;

        let sampler = Sampler::new(
            queue.device().clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                mipmap_mode: vulkano::image::sampler::SamplerMipmapMode::Linear,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create cubemap sampler: {}", e))?;

        trace!("Successfully created cubemap {}x{}", face_size, face_size);

        Ok(Self {
            image,
            view,
            sampler,
            face_size,
        })
    }

    /// Loads a cubemap from six image files (one per face).
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating GPU resources
    /// * `command_buffer_allocator` - Allocator for command buffers
    /// * `queue` - Queue for submitting upload commands
    /// * `face_paths` - Paths to the six face images in +X, -X, +Y, -Y, +Z, -Z order
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any file cannot be read
    /// - Image format is unsupported
    /// - Faces have different sizes
    /// - GPU upload fails
    pub fn from_files(
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
        face_paths: [impl AsRef<Path>; 6],
    ) -> Result<Self> {
        debug!("Loading cubemap from 6 face images");

        let mut face_data: Vec<Vec<u8>> = Vec::with_capacity(6);
        let mut face_size: Option<u32> = None;

        for (i, path) in face_paths.iter().enumerate() {
            let path = path.as_ref();
            debug!("  Face {}: {}", i, path.display());

            let img = image::open(path).map_err(|e| {
                eyre::eyre!("Failed to open cubemap face '{}': {}", path.display(), e)
            })?;

            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();

            if width != height {
                return Err(eyre::eyre!(
                    "Cubemap face '{}' is not square: {}x{}",
                    path.display(),
                    width,
                    height
                ));
            }

            if let Some(size) = face_size {
                if width != size {
                    return Err(eyre::eyre!(
                        "Cubemap face '{}' has different size than previous faces: {} vs {}",
                        path.display(),
                        width,
                        size
                    ));
                }
            } else {
                face_size = Some(width);
            }

            face_data.push(rgba.into_raw());
        }

        let size = face_size.unwrap();
        info!("Loaded cubemap: 6 faces at {}x{}", size, size);

        Self::from_faces(
            allocator,
            command_buffer_allocator,
            queue,
            size,
            [
                face_data[0].clone(),
                face_data[1].clone(),
                face_data[2].clone(),
                face_data[3].clone(),
                face_data[4].clone(),
                face_data[5].clone(),
            ],
        )
    }

    /// Loads a cubemap from an equirectangular HDR or LDR image.
    ///
    /// This converts a spherical panorama (equirectangular projection) into
    /// a cubemap by sampling the panorama for each face direction.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating GPU resources
    /// * `command_buffer_allocator` - Allocator for command buffers
    /// * `queue` - Queue for submitting upload commands
    /// * `path` - Path to the equirectangular image
    /// * `face_size` - Desired size for each cubemap face
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or conversion fails.
    pub fn from_equirectangular(
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
        path: impl AsRef<Path>,
        face_size: u32,
    ) -> Result<Self> {
        let path = path.as_ref();
        debug!("Loading cubemap from equirectangular: {}", path.display());

        let img = image::open(path).map_err(|e| {
            eyre::eyre!(
                "Failed to open equirectangular image '{}': {}",
                path.display(),
                e
            )
        })?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        info!(
            "Converting equirectangular {}x{} to cubemap {}x{}",
            width, height, face_size, face_size
        );

        let face_data = Self::convert_equirectangular_to_cubemap(&rgba, width, height, face_size);

        Self::from_faces(
            allocator,
            command_buffer_allocator,
            queue,
            face_size,
            face_data,
        )
    }

    fn convert_equirectangular_to_cubemap(
        equirect: &image::RgbaImage,
        width: u32,
        height: u32,
        face_size: u32,
    ) -> [Vec<u8>; 6] {
        use std::f32::consts::PI;

        let mut faces: [Vec<u8>; 6] = Default::default();

        for (face_index, face) in faces.iter_mut().enumerate() {
            let mut face_data = Vec::with_capacity((face_size * face_size * 4) as usize);

            for y in 0..face_size {
                for x in 0..face_size {
                    let u = (x as f32 + 0.5) / face_size as f32 * 2.0 - 1.0;
                    let v = (y as f32 + 0.5) / face_size as f32 * 2.0 - 1.0;

                    let dir = match face_index {
                        0 => [1.0, -v, -u],
                        1 => [-1.0, -v, u],
                        2 => [u, 1.0, v],
                        3 => [u, -1.0, -v],
                        4 => [u, -v, 1.0],
                        5 => [-u, -v, -1.0],
                        _ => unreachable!(),
                    };

                    let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
                    let dir = [dir[0] / len, dir[1] / len, dir[2] / len];

                    let theta = dir[1].asin();
                    let phi = dir[2].atan2(dir[0]);

                    let eq_u = ((phi + PI) / (2.0 * PI)) * width as f32;
                    let eq_v = ((PI / 2.0 - theta) / PI) * height as f32;

                    let eq_x = eq_u.clamp(0.0, width as f32 - 1.0) as u32;
                    let eq_y = eq_v.clamp(0.0, height as f32 - 1.0) as u32;

                    let pixel = equirect.get_pixel(eq_x, eq_y);
                    face_data.extend_from_slice(&pixel.0);
                }
            }

            *face = face_data;
        }

        faces
    }
}

impl TextureManager {
    /// Loads a cubemap from six face files and caches it.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for the cubemap
    /// * `face_paths` - Paths to the six face images in +X, -X, +Y, -Y, +Z, -Z order
    ///
    /// # Errors
    ///
    /// Returns an error if cubemap loading fails.
    pub fn load_cubemap(
        &mut self,
        name: impl Into<String>,
        face_paths: [impl AsRef<Path>; 6],
    ) -> Result<()> {
        let name = name.into();

        debug!("Loading cubemap '{}' from 6 face files", name);

        let cubemap = Cubemap::from_files(
            self.allocator.clone(),
            self.command_buffer_allocator.clone(),
            self.queue.clone(),
            face_paths,
        )?;

        self.textures.insert(
            name.clone(),
            Texture {
                image: cubemap.image,
                view: cubemap.view,
                sampler: cubemap.sampler,
                width: cubemap.face_size,
                height: cubemap.face_size,
            },
        );

        info!("Cubemap '{}' loaded and cached", name);
        Ok(())
    }

    /// Loads a cubemap from an equirectangular image and caches it.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for the cubemap
    /// * `path` - Path to the equirectangular image
    /// * `face_size` - Desired size for each cubemap face
    ///
    /// # Errors
    ///
    /// Returns an error if cubemap loading fails.
    pub fn load_cubemap_from_equirectangular(
        &mut self,
        name: impl Into<String>,
        path: impl AsRef<Path>,
        face_size: u32,
    ) -> Result<()> {
        let name = name.into();
        let path = path.as_ref();

        debug!(
            "Loading cubemap '{}' from equirectangular: {}",
            name,
            path.display()
        );

        let cubemap = Cubemap::from_equirectangular(
            self.allocator.clone(),
            self.command_buffer_allocator.clone(),
            self.queue.clone(),
            path,
            face_size,
        )?;

        self.textures.insert(
            name.clone(),
            Texture {
                image: cubemap.image,
                view: cubemap.view,
                sampler: cubemap.sampler,
                width: cubemap.face_size,
                height: cubemap.face_size,
            },
        );

        info!("Cubemap '{}' loaded from equirectangular", name);
        Ok(())
    }
}
