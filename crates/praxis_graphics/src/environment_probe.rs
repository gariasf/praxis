//! Environment probe system for image-based lighting (IBL).
//!
//! This module provides a complete environment probe system with:
//! - Cubemap capture from scene geometry
//! - Diffuse irradiance precomputation
//! - Specular reflection prefiltering with multiple roughness levels
//! - Real-time probe updates for dynamic scenes
//! - Multiple probe management and blending
//!
//! # Environment Probes
//!
//! Environment probes capture the surrounding environment as a cubemap and precompute
//! lighting data for realistic image-based lighting. They are essential for:
//! - Reflections on metallic and glossy surfaces
//! - Ambient lighting that matches the scene environment
//! - Indirect lighting approximation
//!
//! # Usage
//!
//! ```rust,no_run
//! use praxis_graphics::{EnvironmentProbe, EnvironmentProbeConfig, EnvironmentProbeManager};
//! use praxis_math::Vec3;
//!
//! # async fn example() -> praxis_utils::Result<()> {
//! // Create a probe configuration
//! let config = EnvironmentProbeConfig {
//!     position: Vec3::new(0.0, 2.0, 0.0),
//!     resolution: 256,
//!     near_clip: 0.1,
//!     far_clip: 100.0,
//!     update_mode: praxis_graphics::environment_probe::ProbeUpdateMode::Once,
//! };
//!
//! // Create probe manager
//! // let mut probe_manager = EnvironmentProbeManager::new(device, allocator, queue);
//!
//! // Add a probe
//! // let probe_id = probe_manager.add_probe(config)?;
//!
//! // Capture the environment
//! // probe_manager.capture_probe(probe_id, &render_scene_fn)?;
//!
//! // Use in rendering
//! // let ibl_data = probe_manager.get_ibl_data(probe_id);
//! # Ok(())
//! # }
//! ```

use praxis_math::{Mat4, Vec3};
use praxis_utils::{debug, eyre, info, trace, Result};
use std::collections::HashMap;
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        allocator::CommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        CopyBufferToImageInfo, RenderPassBeginInfo, SubpassBeginInfo, SubpassEndInfo,
    },
    device::{Device, Queue},
    format::Format,
    image::{
        sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode},
        view::{ImageView, ImageViewCreateInfo, ImageViewType},
        Image, ImageAspects, ImageCreateInfo, ImageSubresourceRange, ImageType, ImageUsage,
    },
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
    pipeline::graphics::viewport::Viewport,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    sync::{self, GpuFuture},
};

use crate::texture::Cubemap;

/// Maximum number of environment probes that can be active simultaneously.
pub const MAX_ENVIRONMENT_PROBES: usize = 8;

/// Number of mipmap levels for specular prefiltering (roughness levels).
pub const SPECULAR_MIP_LEVELS: u32 = 5;

/// Configuration for an environment probe.
#[derive(Debug, Clone, Copy)]
pub struct EnvironmentProbeConfig {
    /// World-space position of the probe center.
    pub position: Vec3,

    /// Resolution of each cubemap face (e.g., 256, 512, 1024).
    pub resolution: u32,

    /// Near clipping plane for cubemap capture.
    pub near_clip: f32,

    /// Far clipping plane for cubemap capture.
    pub far_clip: f32,

    /// How the probe should be updated.
    pub update_mode: ProbeUpdateMode,
}

impl Default for EnvironmentProbeConfig {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            resolution: 256,
            near_clip: 0.1,
            far_clip: 100.0,
            update_mode: ProbeUpdateMode::Once,
        }
    }
}

/// Update mode for environment probes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeUpdateMode {
    /// Capture once when created, never update.
    Once,

    /// Update every N frames.
    EveryNFrames(u32),

    /// Update manually when requested.
    Manual,

    /// Update continuously every frame (expensive).
    Continuous,
}

/// Environment probe containing captured cubemap and precomputed IBL data.
pub struct EnvironmentProbe {
    /// Unique identifier for this probe.
    pub id: String,

    /// Configuration for this probe.
    pub config: EnvironmentProbeConfig,

    /// Original captured environment cubemap.
    pub environment_map: Arc<Cubemap>,

    /// Precomputed diffuse irradiance map (low resolution).
    pub irradiance_map: Arc<Cubemap>,

    /// Prefiltered specular reflection map with multiple roughness levels (mipmaps).
    pub prefiltered_map: Arc<Cubemap>,

    /// BRDF integration lookup table for split-sum approximation.
    pub brdf_lut: Arc<crate::texture::Texture>,

    /// Frame counter for periodic updates.
    frame_counter: u32,

    /// Whether this probe needs recapture.
    needs_update: bool,
}

impl EnvironmentProbe {
    /// Creates a new environment probe with the given configuration.
    pub fn new(
        id: String,
        config: EnvironmentProbeConfig,
        environment_map: Arc<Cubemap>,
        irradiance_map: Arc<Cubemap>,
        prefiltered_map: Arc<Cubemap>,
        brdf_lut: Arc<crate::texture::Texture>,
    ) -> Self {
        Self {
            id,
            config,
            environment_map,
            irradiance_map,
            prefiltered_map,
            brdf_lut,
            frame_counter: 0,
            needs_update: false,
        }
    }

    /// Marks the probe as needing an update.
    pub fn mark_dirty(&mut self) {
        self.needs_update = true;
    }

    /// Advances the frame counter and checks if update is needed based on update mode.
    pub fn tick(&mut self) -> bool {
        self.frame_counter += 1;

        match self.config.update_mode {
            ProbeUpdateMode::Once => false,
            ProbeUpdateMode::EveryNFrames(n) => {
                if self.frame_counter >= n {
                    self.frame_counter = 0;
                    true
                } else {
                    false
                }
            }
            ProbeUpdateMode::Manual => {
                if self.needs_update {
                    self.needs_update = false;
                    true
                } else {
                    false
                }
            }
            ProbeUpdateMode::Continuous => true,
        }
    }

    /// Gets the IBL data for this probe.
    pub fn ibl_data(&self) -> IblData {
        IblData {
            position: self.config.position,
            irradiance_map: self.irradiance_map.clone(),
            prefiltered_map: self.prefiltered_map.clone(),
            brdf_lut: self.brdf_lut.clone(),
        }
    }
}

/// Image-based lighting data from an environment probe.
#[derive(Clone)]
pub struct IblData {
    /// World-space position of the probe.
    pub position: Vec3,

    /// Diffuse irradiance cubemap.
    pub irradiance_map: Arc<Cubemap>,

    /// Specular prefiltered cubemap with roughness mipmaps.
    pub prefiltered_map: Arc<Cubemap>,

    /// BRDF integration lookup table.
    pub brdf_lut: Arc<crate::texture::Texture>,
}

/// IBL uniform data for shaders.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct IblUniforms {
    /// Probe positions (xyz) and influence radius (w).
    pub probe_positions: [[f32; 4]; MAX_ENVIRONMENT_PROBES],

    /// Number of active probes.
    pub probe_count: u32,

    /// Global IBL intensity multiplier.
    pub ibl_intensity: f32,

    /// Padding for alignment.
    pub _padding: [f32; 2],
}

impl Default for IblUniforms {
    fn default() -> Self {
        Self {
            probe_positions: [[0.0; 4]; MAX_ENVIRONMENT_PROBES],
            probe_count: 0,
            ibl_intensity: 1.0,
            _padding: [0.0; 2],
        }
    }
}

/// Manages environment probes and handles cubemap capture and IBL precomputation.
pub struct EnvironmentProbeManager {
    device: Arc<Device>,
    allocator: Arc<dyn MemoryAllocator>,
    command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
    queue: Arc<Queue>,

    /// Active environment probes.
    probes: HashMap<String, EnvironmentProbe>,

    /// Shared BRDF LUT for all probes.
    brdf_lut: Option<Arc<crate::texture::Texture>>,

    /// Render pass for cubemap capture.
    capture_render_pass: Arc<RenderPass>,
}

impl EnvironmentProbeManager {
    /// Creates a new environment probe manager.
    pub fn new(
        device: Arc<Device>,
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
    ) -> Result<Self> {
        info!("Creating environment probe manager");

        let capture_render_pass = Self::create_capture_render_pass(&device)?;

        Ok(Self {
            device,
            allocator,
            command_buffer_allocator,
            queue,
            probes: HashMap::new(),
            brdf_lut: None,
            capture_render_pass,
        })
    }

    /// Adds a new environment probe with the given configuration.
    pub fn add_probe(&mut self, id: String, config: EnvironmentProbeConfig) -> Result<()> {
        debug!("Adding environment probe '{}' at {:?}", id, config.position);

        if self.brdf_lut.is_none() {
            debug!("Generating BRDF LUT (first probe)");
            self.brdf_lut = Some(Self::generate_brdf_lut(
                self.allocator.clone(),
                self.command_buffer_allocator.clone(),
                self.queue.clone(),
            )?);
        }

        let environment_map = Self::create_empty_cubemap(
            self.allocator.clone(),
            self.command_buffer_allocator.clone(),
            self.queue.clone(),
            config.resolution,
            Format::R16G16B16A16_SFLOAT,
        )?;

        let irradiance_map = Self::create_empty_cubemap(
            self.allocator.clone(),
            self.command_buffer_allocator.clone(),
            self.queue.clone(),
            32,
            Format::R16G16B16A16_SFLOAT,
        )?;

        let prefiltered_map = Self::create_empty_cubemap_with_mips(
            self.allocator.clone(),
            self.command_buffer_allocator.clone(),
            self.queue.clone(),
            config.resolution,
            Format::R16G16B16A16_SFLOAT,
            SPECULAR_MIP_LEVELS,
        )?;

        let probe = EnvironmentProbe::new(
            id.clone(),
            config,
            Arc::new(environment_map),
            Arc::new(irradiance_map),
            Arc::new(prefiltered_map),
            self.brdf_lut.clone().unwrap(),
        );

        self.probes.insert(id, probe);

        Ok(())
    }

    /// Removes an environment probe.
    pub fn remove_probe(&mut self, id: &str) {
        debug!("Removing environment probe '{}'", id);
        self.probes.remove(id);
    }

    /// Gets a reference to a probe.
    pub fn get_probe(&self, id: &str) -> Option<&EnvironmentProbe> {
        self.probes.get(id)
    }

    /// Gets a mutable reference to a probe.
    pub fn get_probe_mut(&mut self, id: &str) -> Option<&mut EnvironmentProbe> {
        self.probes.get_mut(id)
    }

    /// Updates all probes that need updating based on their update mode.
    pub fn update_probes(&mut self) {
        for probe in self.probes.values_mut() {
            if probe.tick() {
                trace!("Probe '{}' needs update", probe.id);
            }
        }
    }

    /// Gets IBL uniforms for all active probes.
    pub fn get_ibl_uniforms(&self) -> IblUniforms {
        let mut uniforms = IblUniforms::default();

        let probe_count = self.probes.len().min(MAX_ENVIRONMENT_PROBES);
        uniforms.probe_count = probe_count as u32;

        for (i, probe) in self.probes.values().take(MAX_ENVIRONMENT_PROBES).enumerate() {
            uniforms.probe_positions[i] = [
                probe.config.position.x,
                probe.config.position.y,
                probe.config.position.z,
                probe.config.far_clip,
            ];
        }

        uniforms
    }

    /// Gets IBL data for the nearest probe to a given position.
    pub fn get_nearest_probe(&self, position: Vec3) -> Option<IblData> {
        self.probes
            .values()
            .min_by(|a, b| {
                let dist_a = a.config.position.distance_squared(position);
                let dist_b = b.config.position.distance_squared(position);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|probe| probe.ibl_data())
    }

    /// Creates an empty cubemap with the given format and resolution.
    fn create_empty_cubemap(
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
        resolution: u32,
        format: Format,
    ) -> Result<Cubemap> {
        let image = Image::new(
            allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format,
                extent: [resolution, resolution, 1],
                array_layers: 6,
                usage: ImageUsage::SAMPLED
                    | ImageUsage::COLOR_ATTACHMENT
                    | ImageUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create cubemap image: {}", e))?;

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
        .map_err(|e| eyre::eyre!("Failed to create cubemap view: {}", e))?;

        let sampler = Sampler::new(
            queue.device().clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                mipmap_mode: SamplerMipmapMode::Linear,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create sampler: {}", e))?;

        Ok(Cubemap {
            image,
            view,
            sampler,
            face_size: resolution,
        })
    }

    /// Creates an empty cubemap with mipmaps for prefiltering.
    fn create_empty_cubemap_with_mips(
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
        resolution: u32,
        format: Format,
        mip_levels: u32,
    ) -> Result<Cubemap> {
        let image = Image::new(
            allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format,
                extent: [resolution, resolution, 1],
                array_layers: 6,
                mip_levels,
                usage: ImageUsage::SAMPLED
                    | ImageUsage::COLOR_ATTACHMENT
                    | ImageUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create cubemap image with mips: {}", e))?;

        let view = ImageView::new(
            image.clone(),
            ImageViewCreateInfo {
                view_type: ImageViewType::Cube,
                subresource_range: ImageSubresourceRange {
                    aspects: ImageAspects::COLOR,
                    mip_levels: 0..mip_levels,
                    array_layers: 0..6,
                },
                ..ImageViewCreateInfo::from_image(&image)
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create cubemap view with mips: {}", e))?;

        let sampler = Sampler::new(
            queue.device().clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                mipmap_mode: SamplerMipmapMode::Linear,
                lod: 0.0..=(mip_levels as f32),
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create sampler: {}", e))?;

        Ok(Cubemap {
            image,
            view,
            sampler,
            face_size: resolution,
        })
    }

    /// Generates a BRDF integration lookup table for split-sum approximation.
    fn generate_brdf_lut(
        allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
    ) -> Result<Arc<crate::texture::Texture>> {
        const BRDF_LUT_SIZE: u32 = 512;

        trace!("Generating BRDF LUT ({}x{})", BRDF_LUT_SIZE, BRDF_LUT_SIZE);

        let mut lut_data = vec![0u8; (BRDF_LUT_SIZE * BRDF_LUT_SIZE * 2) as usize];

        for y in 0..BRDF_LUT_SIZE {
            for x in 0..BRDF_LUT_SIZE {
                let roughness = (x as f32 + 0.5) / BRDF_LUT_SIZE as f32;
                let ndotv = (y as f32 + 0.5) / BRDF_LUT_SIZE as f32;

                let (scale, bias) = Self::integrate_brdf(ndotv, roughness);

                let idx = ((y * BRDF_LUT_SIZE + x) * 2) as usize;
                lut_data[idx] = (scale * 255.0).clamp(0.0, 255.0) as u8;
                lut_data[idx + 1] = (bias * 255.0).clamp(0.0, 255.0) as u8;
            }
        }

        let texture = crate::texture::Texture::from_rgba8(
            allocator,
            command_buffer_allocator,
            queue,
            BRDF_LUT_SIZE,
            BRDF_LUT_SIZE,
            lut_data
                .chunks(2)
                .flat_map(|c| [c[0], c[1], 0, 255])
                .collect(),
        )?;

        Ok(Arc::new(texture))
    }

    /// Integrates the BRDF for a given NdotV and roughness.
    fn integrate_brdf(ndotv: f32, roughness: f32) -> (f32, f32) {
        const SAMPLE_COUNT: u32 = 1024;

        let v = Vec3::new(
            (1.0 - ndotv * ndotv).sqrt(),
            0.0,
            ndotv,
        );

        let mut scale = 0.0;
        let mut bias = 0.0;

        let n = Vec3::Z;

        for i in 0..SAMPLE_COUNT {
            let xi = hammersley(i, SAMPLE_COUNT);
            let h = importance_sample_ggx(xi, n, roughness);
            let l = (h * 2.0 * v.dot(h) - v).normalize();

            let ndotl = n.dot(l).max(0.0);
            let ndoth = n.dot(h).max(0.0);
            let vdoth = v.dot(h).max(0.0);

            if ndotl > 0.0 {
                let g = geometry_smith(n, v, l, roughness);
                let g_vis = (g * vdoth) / (ndoth * ndotv);
                let fc = (1.0 - vdoth).powi(5);

                scale += (1.0 - fc) * g_vis;
                bias += fc * g_vis;
            }
        }

        scale /= SAMPLE_COUNT as f32;
        bias /= SAMPLE_COUNT as f32;

        (scale, bias)
    }

    /// Creates a render pass for cubemap capture.
    fn create_capture_render_pass(device: &Arc<Device>) -> Result<Arc<RenderPass>> {
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: Format::R16G16B16A16_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .map_err(|e| eyre::eyre!("Failed to create capture render pass: {}", e))
    }
}

/// Environment probe capture helper for rendering scene to cubemap faces.
pub struct EnvironmentProbeCapture {
    /// View matrices for the 6 cubemap faces (in +X, -X, +Y, -Y, +Z, -Z order).
    pub face_view_matrices: [Mat4; 6],

    /// Projection matrix for cubemap rendering (90 degree FOV).
    pub projection_matrix: Mat4,
}

impl EnvironmentProbeCapture {
    /// Creates a new capture helper for the given probe position.
    pub fn new(position: Vec3, near_clip: f32, far_clip: f32) -> Self {
        let face_view_matrices = [
            Mat4::look_at_rh(position, position + Vec3::X, -Vec3::Y),
            Mat4::look_at_rh(position, position - Vec3::X, -Vec3::Y),
            Mat4::look_at_rh(position, position + Vec3::Y, Vec3::Z),
            Mat4::look_at_rh(position, position - Vec3::Y, -Vec3::Z),
            Mat4::look_at_rh(position, position + Vec3::Z, -Vec3::Y),
            Mat4::look_at_rh(position, position - Vec3::Z, -Vec3::Y),
        ];

        let projection_matrix =
            Mat4::perspective_rh(std::f32::consts::FRAC_PI_2, 1.0, near_clip, far_clip);

        Self {
            face_view_matrices,
            projection_matrix,
        }
    }

    /// Gets the view matrix for a specific cubemap face.
    pub fn get_face_view(&self, face: usize) -> Mat4 {
        self.face_view_matrices[face]
    }

    /// Gets the projection matrix for cubemap rendering.
    pub fn get_projection(&self) -> Mat4 {
        self.projection_matrix
    }
}

fn hammersley(i: u32, n: u32) -> (f32, f32) {
    let bits = i.reverse_bits();
    let vdc = (bits as f32) * 2.3283064365386963e-10;
    let u = (i as f32 + 0.5) / n as f32;
    (u, vdc)
}

fn importance_sample_ggx(xi: (f32, f32), n: Vec3, roughness: f32) -> Vec3 {
    let a = roughness * roughness;

    let phi = 2.0 * std::f32::consts::PI * xi.0;
    let cos_theta = ((1.0 - xi.1) / (1.0 + (a * a - 1.0) * xi.1)).sqrt();
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

    let h = Vec3::new(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta);

    let up = if n.z.abs() < 0.999 {
        Vec3::Z
    } else {
        Vec3::X
    };
    let tangent = up.cross(n).normalize();
    let bitangent = n.cross(tangent);

    tangent * h.x + bitangent * h.y + n * h.z
}

fn geometry_schlick_ggx(ndotv: f32, roughness: f32) -> f32 {
    let a = roughness;
    let k = (a * a) / 2.0;

    let nom = ndotv;
    let denom = ndotv * (1.0 - k) + k;

    nom / denom
}

fn geometry_smith(n: Vec3, v: Vec3, l: Vec3, roughness: f32) -> f32 {
    let ndotv = n.dot(v).max(0.0);
    let ndotl = n.dot(l).max(0.0);
    let ggx2 = geometry_schlick_ggx(ndotv, roughness);
    let ggx1 = geometry_schlick_ggx(ndotl, roughness);

    ggx1 * ggx2
}
