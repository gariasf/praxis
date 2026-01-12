//! God rays (crepuscular rays) system with radial blur.
//!
//! This module provides realistic light shaft rendering by performing
//! radial blur from light source positions in screen space. God rays
//! simulate light scattering through atmospheric particles, creating
//! dramatic lighting effects.
//!
//! # Architecture
//!
//! - **Occlusion Pass**: Render light-blocking geometry to extract bright areas
//! - **Radial Blur**: Apply directional blur from light source toward edges
//! - **Additive Blending**: Composite god rays over scene
//! - **Temporal Smoothing**: Optional frame blending for stability
//!
//! # Usage
//!
//! ```rust,no_run
//! use praxis_graphics::{GodRaysRenderer, GodRaysConfig};
//! use praxis_math::Vec3;
//!
//! # async fn example() -> praxis_utils::Result<()> {
//! let config = GodRaysConfig {
//!     num_samples: 64,
//!     density: 0.5,
//!     weight: 0.3,
//!     decay: 0.95,
//!     exposure: 0.8,
//!     threshold: 0.8,
//! };
//!
//! // let renderer = GodRaysRenderer::new(device, allocator, extent)?;
//! // renderer.render(command_buffer, light_position_screen, scene_texture, &config)?;
//! # Ok(())
//! # }
//! ```

use bytemuck::Zeroable;
use praxis_math::Vec2;
use praxis_utils::{eyre, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::Device,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

/// Configuration for god rays rendering.
#[derive(Debug, Clone, Copy)]
pub struct GodRaysConfig {
    pub num_samples: u32,
    pub density: f32,
    pub weight: f32,
    pub decay: f32,
    pub exposure: f32,
    pub threshold: f32,
}

impl Default for GodRaysConfig {
    fn default() -> Self {
        Self {
            num_samples: 64,
            density: 0.5,
            weight: 0.3,
            decay: 0.95,
            exposure: 0.8,
            threshold: 0.8,
        }
    }
}

/// God rays uniform data for GPU.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GodRaysUniforms {
    pub light_position_screen: [f32; 4],
    pub num_samples: u32,
    pub density: f32,
    pub weight: f32,
    pub decay: f32,
    pub exposure: f32,
    pub threshold: f32,
    pub _padding: [f32; 2],
}

impl GodRaysUniforms {
    pub fn new(light_pos: Vec2, config: &GodRaysConfig) -> Self {
        Self {
            light_position_screen: [light_pos.x, light_pos.y, 0.0, 0.0],
            num_samples: config.num_samples,
            density: config.density,
            weight: config.weight,
            decay: config.decay,
            exposure: config.exposure,
            threshold: config.threshold,
            _padding: [0.0; 2],
        }
    }
}

/// God rays component for ECS.
#[derive(Debug, Clone, Copy)]
pub struct GodRays {
    pub config: GodRaysConfig,
    pub enabled: bool,
    pub intensity: f32,
}

impl GodRays {
    pub fn new(config: GodRaysConfig) -> Self {
        Self {
            config,
            enabled: true,
            intensity: 1.0,
        }
    }

    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

impl Default for GodRays {
    fn default() -> Self {
        Self {
            config: GodRaysConfig::default(),
            enabled: true,
            intensity: 1.0,
        }
    }
}

/// Radial blur pass for god rays effect.
#[allow(dead_code)]
pub struct RadialBlurPass {
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    uniform_buffer: Subbuffer<GodRaysUniforms>,
}

impl RadialBlurPass {
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<Self> {
        let uniform_buffer = Buffer::from_data(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            GodRaysUniforms::zeroed(),
        )
        .map_err(|e| eyre::eyre!("Failed to create god rays uniform buffer: {}", e))?;

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        Ok(Self {
            device,
            memory_allocator,
            descriptor_set_allocator,
            uniform_buffer,
        })
    }

    pub fn update(&mut self, light_pos: Vec2, config: &GodRaysConfig) -> Result<()> {
        let uniforms = GodRaysUniforms::new(light_pos, config);
        let mut write_lock = self
            .uniform_buffer
            .write()
            .map_err(|e| eyre::eyre!("Failed to lock god rays uniform buffer: {}", e))?;
        *write_lock = uniforms;
        Ok(())
    }

    pub fn buffer(&self) -> &Subbuffer<GodRaysUniforms> {
        &self.uniform_buffer
    }
}

/// Renderer for god rays effects.
#[deprecated(
    since = "0.1.0",
    note = "GodRaysRenderer is an experimental stub with no implementation. \
            See tracking issue: https://github.com/praxis-engine/praxis/issues/TBD"
)]
#[allow(dead_code)]
pub struct GodRaysRenderer {
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    radial_blur_pass: RadialBlurPass,
}

#[allow(deprecated)]
impl GodRaysRenderer {
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<Self> {
        let radial_blur_pass = RadialBlurPass::new(device.clone(), memory_allocator.clone())?;

        Ok(Self {
            device,
            memory_allocator,
            radial_blur_pass,
        })
    }

    pub fn update_light_position(&mut self, light_pos: Vec2, config: &GodRaysConfig) -> Result<()> {
        self.radial_blur_pass.update(light_pos, config)
    }

    pub fn radial_blur_buffer(&self) -> &Subbuffer<GodRaysUniforms> {
        self.radial_blur_pass.buffer()
    }
}
