//! Render target management for post-processing.
//!
//! This module provides render-to-texture functionality, allowing rendering operations
//! to target offscreen framebuffers instead of the swapchain. This is essential for
//! post-processing effects that need to read from the rendered scene.

use praxis_utils::{debug, eyre, info, trace, Result};
use std::sync::Arc;
use vulkano::{
    device::DeviceOwned,
    format::Format,
    image::{
        sampler::{Sampler, SamplerCreateInfo},
        view::ImageView,
        Image, ImageCreateInfo, ImageType, ImageUsage,
    },
    memory::allocator::{AllocationCreateInfo, StandardMemoryAllocator},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
};

/// A render target for offscreen rendering.
///
/// A render target consists of:
/// - A color attachment (texture) to render into
/// - An image view for accessing the texture
/// - A framebuffer binding the texture to a render pass
/// - A sampler for reading the texture in shaders
///
/// # Usage
///
/// Render targets are typically managed by a `RenderTargetPool` to avoid
/// repeated allocations. They can be used as both render destinations and
/// shader inputs.
///
/// # Example
///
/// ```rust,no_run
/// # use praxis_graphics::post_process::RenderTarget;
/// # use std::sync::Arc;
/// # use vulkano::render_pass::RenderPass;
/// # use vulkano::memory::allocator::StandardMemoryAllocator;
/// # fn example(
/// #     memory_allocator: Arc<StandardMemoryAllocator>,
/// #     render_pass: Arc<RenderPass>,
/// # ) -> praxis_utils::Result<()> {
/// let target = RenderTarget::new(
///     memory_allocator,
///     render_pass,
///     [1920, 1080],
///     vulkano::format::Format::R8G8B8A8_UNORM,
/// )?;
///
/// // Use target.framebuffer() for rendering
/// // Use target.image_view() and target.sampler() for reading in shaders
/// # Ok(())
/// # }
/// ```
pub struct RenderTarget {
    /// The color attachment image.
    image: Arc<Image>,
    /// Image view for the color attachment.
    image_view: Arc<ImageView>,
    /// Framebuffer for rendering.
    framebuffer: Arc<Framebuffer>,
    /// Sampler for reading the texture.
    sampler: Arc<Sampler>,
    /// Width of the render target.
    width: u32,
    /// Height of the render target.
    height: u32,
    /// Format of the color attachment.
    format: Format,
}

impl RenderTarget {
    /// Creates a new render target.
    ///
    /// # Arguments
    ///
    /// * `memory_allocator` - Allocator for image memory
    /// * `render_pass` - The render pass this target will be used with
    /// * `extent` - Dimensions [width, height]
    /// * `format` - Color format for the attachment
    ///
    /// # Errors
    ///
    /// Returns an error if image creation, view creation, or framebuffer creation fails.
    pub fn new(
        memory_allocator: Arc<StandardMemoryAllocator>,
        render_pass: Arc<RenderPass>,
        extent: [u32; 2],
        format: Format,
    ) -> Result<Self> {
        debug!(
            "Creating render target: {}x{}, format: {:?}",
            extent[0], extent[1], format
        );

        // Create the color attachment image
        let image = Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format,
                extent: [extent[0], extent[1], 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED | ImageUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .map_err(|e| eyre::eyre!("Failed to create render target image: {}", e))?;

        // Create image view for the attachment
        let image_view = ImageView::new_default(image.clone())
            .map_err(|e| eyre::eyre!("Failed to create render target image view: {}", e))?;

        // Create framebuffer
        let framebuffer = Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![image_view.clone()],
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create render target framebuffer: {}", e))?;

        // Create sampler for reading the texture
        let sampler = Sampler::new(
            memory_allocator.device().clone(),
            SamplerCreateInfo {
                mag_filter: vulkano::image::sampler::Filter::Linear,
                min_filter: vulkano::image::sampler::Filter::Linear,
                address_mode: [vulkano::image::sampler::SamplerAddressMode::ClampToEdge; 3],
                mipmap_mode: vulkano::image::sampler::SamplerMipmapMode::Linear,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create render target sampler: {}", e))?;

        trace!("Render target created successfully");

        Ok(Self {
            image,
            image_view,
            framebuffer,
            sampler,
            width: extent[0],
            height: extent[1],
            format,
        })
    }

    /// Returns a reference to the framebuffer.
    pub fn framebuffer(&self) -> &Arc<Framebuffer> {
        &self.framebuffer
    }

    /// Returns a reference to the image view.
    pub fn image_view(&self) -> &Arc<ImageView> {
        &self.image_view
    }

    /// Returns a reference to the sampler.
    pub fn sampler(&self) -> &Arc<Sampler> {
        &self.sampler
    }

    /// Returns a reference to the image.
    pub fn image(&self) -> &Arc<Image> {
        &self.image
    }

    /// Returns the width of the render target.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the render target.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the extent as [width, height].
    pub fn extent(&self) -> [u32; 2] {
        [self.width, self.height]
    }

    /// Returns the format of the color attachment.
    pub fn format(&self) -> Format {
        self.format
    }
}

/// Pool of render targets for efficient reuse.
///
/// Creating render targets is expensive (GPU memory allocation, view creation, etc.).
/// The pool maintains a set of render targets that can be acquired and released,
/// avoiding repeated allocations.
///
/// # Architecture
///
/// The pool maintains two lists:
/// - **Available**: Render targets ready for use
/// - **In-use**: Render targets currently in use
///
/// When a target is acquired, it moves from available to in-use.
/// When released, it moves back to available.
///
/// # Example
///
/// ```rust,no_run
/// # use praxis_graphics::post_process::RenderTargetPool;
/// # use std::sync::Arc;
/// # use vulkano::render_pass::RenderPass;
/// # use vulkano::memory::allocator::StandardMemoryAllocator;
/// # fn example(
/// #     memory_allocator: Arc<StandardMemoryAllocator>,
/// #     render_pass: Arc<RenderPass>,
/// # ) -> praxis_utils::Result<()> {
/// let mut pool = RenderTargetPool::new(
///     memory_allocator,
///     render_pass,
///     vulkano::format::Format::R8G8B8A8_UNORM,
/// );
///
/// // Acquire a target
/// let target = pool.acquire([1920, 1080])?;
///
/// // Use target for rendering...
///
/// // Release back to pool
/// pool.release(target);
/// # Ok(())
/// # }
/// ```
pub struct RenderTargetPool {
    memory_allocator: Arc<StandardMemoryAllocator>,
    render_pass: Arc<RenderPass>,
    format: Format,
    available: Vec<RenderTarget>,
    in_use: Vec<RenderTarget>,
}

impl RenderTargetPool {
    /// Creates a new render target pool.
    ///
    /// # Arguments
    ///
    /// * `memory_allocator` - Allocator for render target memory
    /// * `render_pass` - The render pass targets will be compatible with
    /// * `format` - Color format for all targets in this pool
    pub fn new(
        memory_allocator: Arc<StandardMemoryAllocator>,
        render_pass: Arc<RenderPass>,
        format: Format,
    ) -> Self {
        info!("Creating render target pool with format: {:?}", format);
        Self {
            memory_allocator,
            render_pass,
            format,
            available: Vec::new(),
            in_use: Vec::new(),
        }
    }

    /// Acquires a render target with the specified dimensions.
    ///
    /// If an available target with matching dimensions exists, it is returned.
    /// Otherwise, a new target is created.
    ///
    /// # Arguments
    ///
    /// * `extent` - Desired dimensions [width, height]
    ///
    /// # Returns
    ///
    /// A render target ready for use.
    ///
    /// # Errors
    ///
    /// Returns an error if a new target cannot be created.
    pub fn acquire(&mut self, extent: [u32; 2]) -> Result<RenderTarget> {
        // Try to find an available target with matching dimensions
        if let Some(index) = self
            .available
            .iter()
            .position(|t| t.width == extent[0] && t.height == extent[1])
        {
            let target = self.available.swap_remove(index);
            trace!(
                "Acquired existing render target from pool: {}x{}",
                extent[0],
                extent[1]
            );
            self.in_use.push(target);
            return Ok(self.in_use.last().unwrap().clone());
        }

        // Create a new target
        debug!(
            "Creating new render target for pool: {}x{}",
            extent[0], extent[1]
        );
        let target = RenderTarget::new(
            self.memory_allocator.clone(),
            self.render_pass.clone(),
            extent,
            self.format,
        )?;

        self.in_use.push(target);
        Ok(self.in_use.last().unwrap().clone())
    }

    /// Releases a render target back to the pool.
    ///
    /// The target becomes available for future acquisitions.
    ///
    /// # Arguments
    ///
    /// * `target` - The render target to release
    pub fn release(&mut self, target: RenderTarget) {
        trace!(
            "Releasing render target to pool: {}x{}",
            target.width,
            target.height
        );
        self.available.push(target);
    }

    /// Releases all in-use targets back to the available pool.
    ///
    /// This should be called at the end of a frame to make all targets
    /// available for the next frame.
    pub fn release_all(&mut self) {
        trace!("Releasing all in-use render targets to pool");
        self.available.append(&mut self.in_use);
    }

    /// Returns the number of available targets in the pool.
    pub fn available_count(&self) -> usize {
        self.available.len()
    }

    /// Returns the number of in-use targets.
    pub fn in_use_count(&self) -> usize {
        self.in_use.len()
    }

    /// Returns the total number of targets in the pool.
    pub fn total_count(&self) -> usize {
        self.available.len() + self.in_use.len()
    }
}

impl Clone for RenderTarget {
    fn clone(&self) -> Self {
        Self {
            image: self.image.clone(),
            image_view: self.image_view.clone(),
            framebuffer: self.framebuffer.clone(),
            sampler: self.sampler.clone(),
            width: self.width,
            height: self.height,
            format: self.format,
        }
    }
}
