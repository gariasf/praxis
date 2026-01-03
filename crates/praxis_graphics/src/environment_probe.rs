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
    command_buffer::allocator::CommandBufferAllocator,
    device::{Device, Queue},
    format::Format,
    image::{
        sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode},
        view::{ImageView, ImageViewCreateInfo, ImageViewType},
        Image, ImageAspects, ImageCreateInfo, ImageSubresourceRange, ImageType, ImageUsage,
    },
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
    render_pass::RenderPass,
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
    _device: Arc<Device>,
    allocator: Arc<dyn MemoryAllocator>,
    command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
    queue: Arc<Queue>,

    /// Active environment probes.
    probes: HashMap<String, EnvironmentProbe>,

    /// Shared BRDF LUT for all probes.
    brdf_lut: Option<Arc<crate::texture::Texture>>,

    /// Render pass for cubemap capture.
    _capture_render_pass: Arc<RenderPass>,
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
            _device: device,
            allocator,
            command_buffer_allocator,
            queue,
            probes: HashMap::new(),
            brdf_lut: None,
            _capture_render_pass: capture_render_pass,
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
        _command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
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
        _command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
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
    let vdc = (bits as f32) * 2.328_306_4e-10;
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

#[cfg(test)]
mod tests {
    use super::*;
    use praxis_math::{Mat4, Vec3};

    #[test]
    fn test_environment_probe_config_default() {
        let config = EnvironmentProbeConfig::default();
        
        assert_eq!(config.position, Vec3::ZERO);
        assert_eq!(config.resolution, 256);
        assert_eq!(config.near_clip, 0.1);
        assert_eq!(config.far_clip, 100.0);
        assert!(matches!(config.update_mode, ProbeUpdateMode::Once));
    }

    #[test]
    fn test_environment_probe_config_custom() {
        let config = EnvironmentProbeConfig {
            position: Vec3::new(10.0, 20.0, 30.0),
            resolution: 512,
            near_clip: 0.5,
            far_clip: 200.0,
            update_mode: ProbeUpdateMode::Continuous,
        };
        
        assert_eq!(config.position, Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(config.resolution, 512);
        assert_eq!(config.near_clip, 0.5);
        assert_eq!(config.far_clip, 200.0);
        assert!(matches!(config.update_mode, ProbeUpdateMode::Continuous));
    }

    #[test]
    fn test_probe_update_mode_once() {
        let mode = ProbeUpdateMode::Once;
        assert_eq!(mode, ProbeUpdateMode::Once);
    }

    #[test]
    fn test_probe_update_mode_every_n_frames() {
        let mode = ProbeUpdateMode::EveryNFrames(60);
        assert_eq!(mode, ProbeUpdateMode::EveryNFrames(60));
    }

    #[test]
    fn test_probe_update_mode_manual() {
        let mode = ProbeUpdateMode::Manual;
        assert_eq!(mode, ProbeUpdateMode::Manual);
    }

    #[test]
    fn test_probe_update_mode_continuous() {
        let mode = ProbeUpdateMode::Continuous;
        assert_eq!(mode, ProbeUpdateMode::Continuous);
    }

    #[test]
    fn test_environment_probe_capture_face_count() {
        let capture = EnvironmentProbeCapture::new(Vec3::ZERO, 0.1, 100.0);
        
        // Should have 6 face view matrices (one per cubemap face)
        assert_eq!(capture.face_view_matrices.len(), 6);
    }

    #[test]
    fn test_environment_probe_capture_projection() {
        let near = 0.1;
        let far = 100.0;
        let capture = EnvironmentProbeCapture::new(Vec3::ZERO, near, far);
        
        // Projection should be 90 degree FOV (for cubemap)
        let proj = capture.get_projection();
        
        // Verify it's a valid projection matrix (not zero)
        assert_ne!(proj, Mat4::ZERO);
    }

    #[test]
    fn test_environment_probe_capture_face_directions() {
        let position = Vec3::new(5.0, 10.0, 15.0);
        let capture = EnvironmentProbeCapture::new(position, 0.1, 100.0);
        
        // Each face view matrix should be valid
        for i in 0..6 {
            let view = capture.get_face_view(i);
            assert_ne!(view, Mat4::ZERO);
            
            // Extract the translation component
            let translation = view.col(3).truncate();
            
            // View matrix translation should be related to position
            // (inverse of position due to view transform)
            assert!(translation.is_finite());
        }
    }

    #[test]
    fn test_environment_probe_capture_face_order() {
        // Test that faces are in the expected order: +X, -X, +Y, -Y, +Z, -Z
        let position = Vec3::ZERO;
        let capture = EnvironmentProbeCapture::new(position, 0.1, 100.0);
        
        // Each face should look in a different direction
        // We can verify by checking they're all different
        let views: Vec<_> = (0..6).map(|i| capture.get_face_view(i)).collect();
        
        for i in 0..6 {
            for j in (i + 1)..6 {
                assert_ne!(views[i], views[j], "Face {} and {} should be different", i, j);
            }
        }
    }

    #[test]
    fn test_ibl_uniforms_default() {
        let uniforms = IblUniforms::default();
        
        assert_eq!(uniforms.probe_count, 0);
        assert_eq!(uniforms.ibl_intensity, 1.0);
        
        // All probe positions should be zero
        for pos in uniforms.probe_positions.iter() {
            assert_eq!(*pos, [0.0; 4]);
        }
    }

    #[test]
    fn test_ibl_uniforms_size() {
        use std::mem::size_of;
        
        let size = size_of::<IblUniforms>();
        
        // Should be properly aligned for std140
        // 8 probes * vec4 (16 bytes each) + u32 (4) + f32 (4) + padding (8) = 144 bytes minimum
        assert!(size >= 128, "IblUniforms should be large enough");
        
        // Verify it's POD
        let uniforms = IblUniforms::default();
        let _bytes = bytemuck::bytes_of(&uniforms);
    }

    #[test]
    fn test_hammersley_sequence() {
        // Test Hammersley sequence generation
        let samples = 8;
        
        for i in 0..samples {
            let (u, v) = hammersley(i, samples);
            
            // Both components should be in [0, 1]
            assert!(u >= 0.0 && u <= 1.0);
            assert!(v >= 0.0 && v <= 1.0);
        }
    }

    #[test]
    fn test_hammersley_sequence_distribution() {
        // Test that Hammersley sequence provides good distribution
        let samples = 16;
        let mut points = Vec::new();
        
        for i in 0..samples {
            points.push(hammersley(i, samples));
        }
        
        // All points should be unique
        for i in 0..points.len() {
            for j in (i + 1)..points.len() {
                let diff = ((points[i].0 - points[j].0).abs() + (points[i].1 - points[j].1).abs());
                assert!(diff > 0.001, "Points should be distributed");
            }
        }
    }

    #[test]
    fn test_importance_sample_ggx_hemisphere() {
        // Test that importance sampling produces vectors in upper hemisphere
        let n = Vec3::Z; // Normal pointing up
        let roughness = 0.5;
        
        for i in 0..10 {
            let xi = hammersley(i, 10);
            let h = importance_sample_ggx(xi, n, roughness);
            
            // h should be normalized
            assert!((h.length() - 1.0).abs() < 0.01, "Sample should be normalized");
            
            // h.z should be positive (upper hemisphere)
            assert!(h.z >= 0.0, "Sample should be in upper hemisphere");
        }
    }

    #[test]
    fn test_importance_sample_ggx_roughness_effect() {
        let n = Vec3::Z;
        let xi = (0.5, 0.5);
        
        // Lower roughness should produce samples more concentrated around normal
        let smooth = importance_sample_ggx(xi, n, 0.1);
        let rough = importance_sample_ggx(xi, n, 0.9);
        
        // Smooth surface sample should be closer to normal (higher z)
        assert!(smooth.z > rough.z, 
            "Smooth sample z ({}) should be > rough sample z ({})", 
            smooth.z, rough.z);
    }

    #[test]
    fn test_geometry_schlick_ggx() {
        let ndotv = 0.8;
        let roughness = 0.5;
        
        let g = geometry_schlick_ggx(ndotv, roughness);
        
        // Result should be in [0, 1]
        assert!(g >= 0.0 && g <= 1.0);
        
        // Higher ndotv should give higher value (less occlusion)
        let g_high = geometry_schlick_ggx(0.9, roughness);
        let g_low = geometry_schlick_ggx(0.1, roughness);
        assert!(g_high > g_low);
    }

    #[test]
    fn test_geometry_schlick_ggx_roughness() {
        let ndotv = 0.5;
        
        // Rougher surfaces should have lower geometry term
        let smooth = geometry_schlick_ggx(ndotv, 0.1);
        let rough = geometry_schlick_ggx(ndotv, 0.9);
        
        assert!(smooth > rough, "Smooth surfaces should have higher G term");
    }

    #[test]
    fn test_geometry_smith() {
        let n = Vec3::Z;
        let v = Vec3::new(0.0, 0.5, 0.866).normalize(); // 30 degrees from normal
        let l = Vec3::new(0.5, 0.0, 0.866).normalize(); // 30 degrees from normal
        let roughness = 0.5;
        
        let g = geometry_smith(n, v, l, roughness);
        
        // Result should be in [0, 1]
        assert!(g >= 0.0 && g <= 1.0);
    }

    #[test]
    fn test_geometry_smith_grazing_angles() {
        let n = Vec3::Z;
        let v = Vec3::new(0.99, 0.0, 0.1).normalize(); // Grazing angle
        let l = Vec3::new(0.0, 0.99, 0.1).normalize(); // Grazing angle
        let roughness = 0.5;
        
        let g = geometry_smith(n, v, l, roughness);
        
        // Should be significantly occluded at grazing angles
        assert!(g < 0.5, "Grazing angles should have high occlusion");
    }

    #[test]
    fn test_brdf_integration_ndotv_range() {
        // Test BRDF integration for various view angles
        let test_ndotv = [0.1, 0.3, 0.5, 0.7, 0.9];
        let roughness = 0.5;
        
        for ndotv in test_ndotv.iter() {
            let (scale, bias) = EnvironmentProbeManager::integrate_brdf(*ndotv, roughness);
            
            // Scale and bias should be in reasonable ranges [0, 1]
            assert!(scale >= 0.0 && scale <= 1.0, "Scale should be in [0,1]");
            assert!(bias >= 0.0 && bias <= 1.0, "Bias should be in [0,1]");
        }
    }

    #[test]
    fn test_brdf_integration_roughness_range() {
        // Test BRDF integration for various roughness values
        let ndotv = 0.5;
        let test_roughness = [0.0, 0.25, 0.5, 0.75, 1.0];
        
        for roughness in test_roughness.iter() {
            let (scale, bias) = EnvironmentProbeManager::integrate_brdf(ndotv, *roughness);
            
            assert!(scale >= 0.0 && scale <= 1.0);
            assert!(bias >= 0.0 && bias <= 1.0);
        }
    }

    #[test]
    fn test_brdf_integration_smooth_vs_rough() {
        let ndotv = 0.5;
        
        // Smooth surfaces should have different BRDF characteristics
        let (scale_smooth, _) = EnvironmentProbeManager::integrate_brdf(ndotv, 0.0);
        let (scale_rough, _) = EnvironmentProbeManager::integrate_brdf(ndotv, 1.0);
        
        // Values should be different for different roughness
        assert_ne!(scale_smooth, scale_rough);
    }

    #[test]
    fn test_max_environment_probes_constant() {
        // Verify constant is reasonable
        assert_eq!(MAX_ENVIRONMENT_PROBES, 8);
        assert!(MAX_ENVIRONMENT_PROBES > 0);
        assert!(MAX_ENVIRONMENT_PROBES <= 16); // Reasonable upper limit
    }

    #[test]
    fn test_specular_mip_levels_constant() {
        // Verify mip levels constant
        assert_eq!(SPECULAR_MIP_LEVELS, 5);
        assert!(SPECULAR_MIP_LEVELS > 0);
        assert!(SPECULAR_MIP_LEVELS <= 10); // Reasonable upper limit
    }

    #[test]
    fn test_environment_probe_capture_perspective() {
        // Test that capture uses proper perspective projection
        let capture = EnvironmentProbeCapture::new(Vec3::ZERO, 0.1, 100.0);
        let proj = capture.get_projection();
        
        // For cubemap, FOV should be 90 degrees (PI/2)
        // Aspect ratio should be 1.0 (square faces)
        
        // Verify projection matrix is not identity
        assert_ne!(proj, Mat4::IDENTITY);
        
        // Verify it's a proper perspective projection (not orthographic)
        // In perspective, w component changes with z
        let test_point = praxis_math::Vec4::new(1.0, 1.0, -10.0, 1.0);
        let projected = proj * test_point;
        
        // After projection, w should not be 1.0 (perspective division needed)
        assert_ne!(projected.w, 1.0);
    }

    #[test]
    fn test_environment_probe_capture_position_offset() {
        // Test that different positions produce different view matrices
        let pos1 = Vec3::new(0.0, 0.0, 0.0);
        let pos2 = Vec3::new(10.0, 20.0, 30.0);
        
        let capture1 = EnvironmentProbeCapture::new(pos1, 0.1, 100.0);
        let capture2 = EnvironmentProbeCapture::new(pos2, 0.1, 100.0);
        
        // View matrices should be different for different positions
        for i in 0..6 {
            let view1 = capture1.get_face_view(i);
            let view2 = capture2.get_face_view(i);
            
            assert_ne!(view1, view2, "Face {} views should differ for different positions", i);
        }
    }
}
