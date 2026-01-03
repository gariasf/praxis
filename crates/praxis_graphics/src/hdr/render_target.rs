//! HDR render target for floating-point rendering.
//!
//! This module provides render targets with floating-point color formats
//! that can store HDR values beyond the [0,1] range.

use praxis_utils::{debug, eyre, trace, Result};
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

/// HDR render target with floating-point color format.
///
/// Uses R16G16B16A16_SFLOAT format to store HDR values that can exceed
/// the standard [0,1] range, allowing for more realistic lighting and
/// better bloom effects.
pub struct HdrRenderTarget {
    image: Arc<Image>,
    image_view: Arc<ImageView>,
    framebuffer: Arc<Framebuffer>,
    sampler: Arc<Sampler>,
    width: u32,
    height: u32,
}

impl HdrRenderTarget {
    /// Creates a new HDR render target.
    pub fn new(
        memory_allocator: Arc<StandardMemoryAllocator>,
        render_pass: Arc<RenderPass>,
        extent: [u32; 2],
    ) -> Result<Self> {
        debug!(
            "Creating HDR render target: {}x{}, format: R16G16B16A16_SFLOAT",
            extent[0], extent[1]
        );

        let format = Format::R16G16B16A16_SFLOAT;

        let image = Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format,
                extent: [extent[0], extent[1], 1],
                usage: ImageUsage::COLOR_ATTACHMENT
                    | ImageUsage::SAMPLED
                    | ImageUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .map_err(|e| eyre::eyre!("Failed to create HDR render target image: {}", e))?;

        let image_view = ImageView::new_default(image.clone())
            .map_err(|e| eyre::eyre!("Failed to create HDR render target image view: {}", e))?;

        let framebuffer = Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![image_view.clone()],
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create HDR render target framebuffer: {}", e))?;

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
        .map_err(|e| eyre::eyre!("Failed to create HDR render target sampler: {}", e))?;

        trace!("HDR render target created successfully");

        Ok(Self {
            image,
            image_view,
            framebuffer,
            sampler,
            width: extent[0],
            height: extent[1],
        })
    }

    pub fn framebuffer(&self) -> &Arc<Framebuffer> {
        &self.framebuffer
    }

    pub fn image_view(&self) -> &Arc<ImageView> {
        &self.image_view
    }

    pub fn sampler(&self) -> &Arc<Sampler> {
        &self.sampler
    }

    pub fn image(&self) -> &Arc<Image> {
        &self.image
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn extent(&self) -> [u32; 2] {
        [self.width, self.height]
    }

    pub fn format(&self) -> Format {
        Format::R16G16B16A16_SFLOAT
    }
}

impl Clone for HdrRenderTarget {
    fn clone(&self) -> Self {
        Self {
            image: self.image.clone(),
            image_view: self.image_view.clone(),
            framebuffer: self.framebuffer.clone(),
            sampler: self.sampler.clone(),
            width: self.width,
            height: self.height,
        }
    }
}
