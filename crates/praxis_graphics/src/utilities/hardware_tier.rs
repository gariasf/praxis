//! Hardware tier detection and automatic quality preset configuration.
//!
//! This module provides GPU capability detection via Vulkan to automatically configure
//! rendering quality settings based on hardware tier. It queries:
//! - **VRAM**: Total and available GPU memory
//! - **Compute Capability**: Shader compute units, clock speeds
//! - **Vendor**: AMD, NVIDIA, Intel, Apple, etc.
//! - **Device Type**: Integrated, discrete, mobile, virtual
//!
//! Based on these properties, the system classifies GPUs into tiers and provides
//! preset configurations optimized for each tier.
//!
//! # Hardware Tiers
//!
//! - **Integrated**: Low-power integrated GPUs (Intel UHD, AMD Vega integrated)
//! - **Mobile**: Mid-range laptop GPUs
//! - **MidRange**: Desktop GPUs from 2-4 years ago
//! - **HighEnd**: Modern high-performance GPUs
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use praxis_graphics::utilities::hardware_tier::{HardwareTierDetector, HardwareTierConfig};
//! use vulkano::device::physical::PhysicalDevice;
//!
//! // Detect hardware tier from physical device
//! let detector = HardwareTierDetector::from_physical_device(&physical_device);
//!
//! // Get recommended quality preset
//! let preset = detector.recommended_preset();
//!
//! // Apply preset to configuration
//! let config = HardwareTierConfig::from_preset(preset);
//!
//! // Use configuration values
//! shadow_manager.set_resolution(config.shadow_resolution);
//! lod_manager.set_bias(config.lod_bias);
//! post_process.set_enabled(config.enable_bloom);
//! ```

use praxis_utils::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};

/// GPU vendor identification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GpuVendor {
    /// NVIDIA GPUs
    Nvidia,
    /// AMD GPUs
    Amd,
    /// Intel GPUs
    Intel,
    /// Apple Silicon GPUs
    Apple,
    /// ARM Mali GPUs
    Arm,
    /// Qualcomm Adreno GPUs
    Qualcomm,
    /// Unknown or other vendor
    Unknown,
}

impl GpuVendor {
    /// Identifies vendor from Vulkan vendor ID.
    ///
    /// Vendor IDs are standardized by Khronos:
    /// - 0x1002: AMD
    /// - 0x10DE: NVIDIA
    /// - 0x8086: Intel
    /// - 0x106B: Apple
    /// - 0x13B5: ARM
    /// - 0x5143: Qualcomm
    pub fn from_vendor_id(vendor_id: u32) -> Self {
        match vendor_id {
            0x1002 => Self::Amd,
            0x10DE => Self::Nvidia,
            0x8086 => Self::Intel,
            0x106B => Self::Apple,
            0x13B5 => Self::Arm,
            0x5143 => Self::Qualcomm,
            _ => Self::Unknown,
        }
    }

    /// Returns a human-readable vendor name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Nvidia => "NVIDIA",
            Self::Amd => "AMD",
            Self::Intel => "Intel",
            Self::Apple => "Apple",
            Self::Arm => "ARM",
            Self::Qualcomm => "Qualcomm",
            Self::Unknown => "Unknown",
        }
    }
}

/// Hardware tier classification based on GPU capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HardwareTier {
    /// Integrated GPUs with limited VRAM and compute capability.
    ///
    /// Examples: Intel UHD 620, AMD Vega 8 (integrated), Apple M1 base
    Integrated,

    /// Mobile/laptop GPUs with moderate capabilities.
    ///
    /// Examples: NVIDIA GTX 1650 Mobile, AMD RX 6600M
    Mobile,

    /// Mid-range desktop GPUs (2-4 years old or current mid-tier).
    ///
    /// Examples: NVIDIA GTX 1660/RTX 3060, AMD RX 5700/6700 XT
    MidRange,

    /// High-end GPUs with excellent performance.
    ///
    /// Examples: NVIDIA RTX 3080/4080, AMD RX 6900 XT/7900 XT
    HighEnd,
}

impl HardwareTier {
    /// Returns a human-readable tier name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Integrated => "Integrated",
            Self::Mobile => "Mobile",
            Self::MidRange => "Mid-Range",
            Self::HighEnd => "High-End",
        }
    }

    /// Returns a detailed description of the tier.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Integrated => "Low-power integrated graphics suitable for basic rendering",
            Self::Mobile => "Mobile/laptop GPU with moderate performance",
            Self::MidRange => "Mid-range desktop GPU capable of good quality rendering",
            Self::HighEnd => "High-performance GPU suitable for maximum quality settings",
        }
    }
}

/// Quality preset for different hardware tiers.
///
/// Each preset provides recommended settings for various rendering features
/// optimized for the target hardware tier.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum QualityPreset {
    /// Low quality settings for integrated GPUs.
    Low,
    /// Medium quality settings for mobile/mid-range GPUs.
    Medium,
    /// High quality settings for high-end GPUs.
    High,
    /// Ultra quality settings for top-tier GPUs with headroom.
    Ultra,
}

impl QualityPreset {
    /// Returns a human-readable preset name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Ultra => "Ultra",
        }
    }

    /// Returns the recommended preset for a hardware tier.
    pub fn from_tier(tier: HardwareTier) -> Self {
        match tier {
            HardwareTier::Integrated => Self::Low,
            HardwareTier::Mobile => Self::Medium,
            HardwareTier::MidRange => Self::High,
            HardwareTier::HighEnd => Self::Ultra,
        }
    }
}

/// Configuration settings optimized for a specific hardware tier.
///
/// This structure contains concrete values for all rendering parameters
/// that should be adjusted based on hardware capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareTierConfig {
    /// Shadow map resolution (must be power of two).
    pub shadow_resolution: u32,

    /// Maximum number of cascaded shadow maps.
    pub shadow_cascades: u32,

    /// Enable soft shadows (PCF filtering).
    pub enable_soft_shadows: bool,

    /// LOD bias adjustment (-1.0 = lower detail, +1.0 = higher detail).
    pub lod_bias: f32,

    /// Maximum LOD level to use (higher = allow lower detail meshes).
    pub max_lod_level: u32,

    /// Mesh streaming priority threshold (0.0 = load all, 100.0 = load only critical).
    pub mesh_streaming_threshold: f32,

    /// Enable mesh streaming system.
    pub enable_mesh_streaming: bool,

    /// Enable GPU culling optimizations.
    pub enable_gpu_culling: bool,

    /// Enable Hi-Z occlusion culling.
    pub enable_hiz_occlusion: bool,

    /// Texture quality level (0 = lowest, 3 = highest).
    pub texture_quality: u32,

    /// Anisotropic filtering level (1, 2, 4, 8, 16).
    pub anisotropic_filtering: u32,

    /// MSAA sample count (1 = disabled, 2, 4, 8).
    pub msaa_samples: u32,

    /// Enable post-processing effects.
    pub enable_post_processing: bool,

    /// Enable bloom effect.
    pub enable_bloom: bool,

    /// Enable screen-space ambient occlusion (SSAO).
    pub enable_ssao: bool,

    /// SSAO sample count.
    pub ssao_samples: u32,

    /// Enable screen-space reflections (SSR).
    pub enable_ssr: bool,

    /// Enable temporal anti-aliasing (TAA).
    pub enable_taa: bool,

    /// Enable particle system effects.
    pub enable_particles: bool,

    /// Maximum particle count.
    pub max_particles: u32,

    /// View distance multiplier (1.0 = default, 0.5 = half distance).
    pub view_distance_multiplier: f32,

    /// Enable environment probes.
    pub enable_environment_probes: bool,

    /// Environment probe resolution.
    pub environment_probe_resolution: u32,

    /// Target frame rate for adaptive quality.
    pub target_fps: f64,
}

impl HardwareTierConfig {
    /// Creates a configuration from a quality preset.
    pub fn from_preset(preset: QualityPreset) -> Self {
        match preset {
            QualityPreset::Low => Self::low(),
            QualityPreset::Medium => Self::medium(),
            QualityPreset::High => Self::high(),
            QualityPreset::Ultra => Self::ultra(),
        }
    }

    /// Low quality settings optimized for integrated GPUs.
    ///
    /// Prioritizes performance over visual quality:
    /// - Minimal shadow quality
    /// - Aggressive LOD bias
    /// - Disabled expensive effects (SSAO, SSR, bloom)
    /// - Low resolution textures
    pub fn low() -> Self {
        Self {
            shadow_resolution: 512,
            shadow_cascades: 2,
            enable_soft_shadows: false,
            lod_bias: -0.5,
            max_lod_level: 3,
            mesh_streaming_threshold: 50.0,
            enable_mesh_streaming: false,
            enable_gpu_culling: true,
            enable_hiz_occlusion: false,
            texture_quality: 0,
            anisotropic_filtering: 2,
            msaa_samples: 1,
            enable_post_processing: false,
            enable_bloom: false,
            enable_ssao: false,
            ssao_samples: 8,
            enable_ssr: false,
            enable_taa: false,
            enable_particles: true,
            max_particles: 1000,
            view_distance_multiplier: 0.5,
            enable_environment_probes: false,
            environment_probe_resolution: 64,
            target_fps: 30.0,
        }
    }

    /// Medium quality settings for mobile/mid-range GPUs.
    ///
    /// Balanced performance and quality:
    /// - Moderate shadow quality
    /// - Neutral LOD bias
    /// - Selective post-processing
    /// - Medium resolution textures
    pub fn medium() -> Self {
        Self {
            shadow_resolution: 1024,
            shadow_cascades: 3,
            enable_soft_shadows: true,
            lod_bias: 0.0,
            max_lod_level: 4,
            mesh_streaming_threshold: 20.0,
            enable_mesh_streaming: true,
            enable_gpu_culling: true,
            enable_hiz_occlusion: false,
            texture_quality: 1,
            anisotropic_filtering: 4,
            msaa_samples: 2,
            enable_post_processing: true,
            enable_bloom: true,
            enable_ssao: true,
            ssao_samples: 16,
            enable_ssr: false,
            enable_taa: false,
            enable_particles: true,
            max_particles: 5000,
            view_distance_multiplier: 0.75,
            enable_environment_probes: true,
            environment_probe_resolution: 128,
            target_fps: 60.0,
        }
    }

    /// High quality settings for high-end GPUs.
    ///
    /// High visual quality with good performance:
    /// - High shadow quality
    /// - Positive LOD bias for detail
    /// - Most post-processing enabled
    /// - High resolution textures
    pub fn high() -> Self {
        Self {
            shadow_resolution: 2048,
            shadow_cascades: 4,
            enable_soft_shadows: true,
            lod_bias: 0.3,
            max_lod_level: 5,
            mesh_streaming_threshold: 10.0,
            enable_mesh_streaming: true,
            enable_gpu_culling: true,
            enable_hiz_occlusion: true,
            texture_quality: 2,
            anisotropic_filtering: 8,
            msaa_samples: 4,
            enable_post_processing: true,
            enable_bloom: true,
            enable_ssao: true,
            ssao_samples: 32,
            enable_ssr: true,
            enable_taa: true,
            enable_particles: true,
            max_particles: 10000,
            view_distance_multiplier: 1.0,
            enable_environment_probes: true,
            environment_probe_resolution: 256,
            target_fps: 60.0,
        }
    }

    /// Ultra quality settings for top-tier GPUs.
    ///
    /// Maximum visual quality:
    /// - Maximum shadow quality
    /// - Maximum LOD bias for detail
    /// - All post-processing enabled
    /// - Maximum resolution textures
    pub fn ultra() -> Self {
        Self {
            shadow_resolution: 4096,
            shadow_cascades: 4,
            enable_soft_shadows: true,
            lod_bias: 0.5,
            max_lod_level: 6,
            mesh_streaming_threshold: 5.0,
            enable_mesh_streaming: true,
            enable_gpu_culling: true,
            enable_hiz_occlusion: true,
            texture_quality: 3,
            anisotropic_filtering: 16,
            msaa_samples: 8,
            enable_post_processing: true,
            enable_bloom: true,
            enable_ssao: true,
            ssao_samples: 64,
            enable_ssr: true,
            enable_taa: true,
            enable_particles: true,
            max_particles: 50000,
            view_distance_multiplier: 1.5,
            enable_environment_probes: true,
            environment_probe_resolution: 512,
            target_fps: 60.0,
        }
    }

    /// Returns a summary of the configuration.
    pub fn summary(&self) -> String {
        format!(
            "Hardware Tier Configuration:\n\
             - Shadow Resolution: {}x{} ({} cascades)\n\
             - Soft Shadows: {}\n\
             - LOD Bias: {:.2}\n\
             - Texture Quality: {}\n\
             - Anisotropic Filtering: {}x\n\
             - MSAA: {}x\n\
             - Post-Processing: {}\n\
             - Bloom: {}\n\
             - SSAO: {} ({} samples)\n\
             - SSR: {}\n\
             - TAA: {}\n\
             - GPU Culling: {}\n\
             - Hi-Z Occlusion: {}\n\
             - Mesh Streaming: {}\n\
             - Max Particles: {}\n\
             - View Distance: {:.1}x\n\
             - Target FPS: {:.0}",
            self.shadow_resolution,
            self.shadow_resolution,
            self.shadow_cascades,
            if self.enable_soft_shadows { "enabled" } else { "disabled" },
            self.lod_bias,
            self.texture_quality,
            self.anisotropic_filtering,
            self.msaa_samples,
            if self.enable_post_processing { "enabled" } else { "disabled" },
            if self.enable_bloom { "enabled" } else { "disabled" },
            if self.enable_ssao { "enabled" } else { "disabled" },
            self.ssao_samples,
            if self.enable_ssr { "enabled" } else { "disabled" },
            if self.enable_taa { "enabled" } else { "disabled" },
            if self.enable_gpu_culling { "enabled" } else { "disabled" },
            if self.enable_hiz_occlusion { "enabled" } else { "disabled" },
            if self.enable_mesh_streaming { "enabled" } else { "disabled" },
            self.max_particles,
            self.view_distance_multiplier,
            self.target_fps
        )
    }
}

/// Hardware tier detector that queries GPU properties via Vulkan.
///
/// This structure analyzes physical device properties to determine
/// the appropriate hardware tier and recommend quality presets.
#[derive(Debug, Clone)]
pub struct HardwareTierDetector {
    /// GPU vendor identification.
    pub vendor: GpuVendor,

    /// Physical device type (discrete, integrated, etc.).
    pub device_type: PhysicalDeviceType,

    /// Total VRAM in bytes.
    pub total_vram: u64,

    /// Device name as reported by Vulkan.
    pub device_name: String,

    /// Vulkan API version supported.
    pub api_version: u32,

    /// Driver version.
    pub driver_version: u32,

    /// Vulkan vendor ID.
    pub vendor_id: u32,

    /// Vulkan device ID.
    pub device_id: u32,

    /// Detected hardware tier.
    pub tier: HardwareTier,

    /// Maximum number of compute units (shader processors).
    pub compute_units: u32,

    /// Maximum texture dimension supported.
    pub max_texture_dimension: u32,

    /// Maximum number of bound descriptor sets.
    pub max_bound_descriptor_sets: u32,

    /// Supports timeline semaphores for advanced synchronization.
    pub supports_timeline_semaphores: bool,

    /// Supports bindless rendering (descriptor indexing).
    pub supports_descriptor_indexing: bool,
}

impl HardwareTierDetector {
    /// Creates a hardware tier detector from a Vulkan physical device.
    ///
    /// Queries all relevant GPU properties and determines the appropriate
    /// hardware tier based on VRAM, device type, vendor, and compute capability.
    ///
    /// # Arguments
    ///
    /// * `physical_device` - Vulkan physical device to analyze
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let detector = HardwareTierDetector::from_physical_device(&physical_device);
    /// println!("Detected GPU: {} ({})", detector.device_name, detector.tier.name());
    /// ```
    pub fn from_physical_device(physical_device: &Arc<PhysicalDevice>) -> Self {
        let properties = physical_device.properties();
        let vendor = GpuVendor::from_vendor_id(properties.vendor_id);
        let device_type = properties.device_type;

        // Query memory properties to determine VRAM
        let memory_properties = physical_device.memory_properties();
        let total_vram = Self::calculate_total_vram(memory_properties);

        // Query limits
        let limits = &properties.limits;
        let max_texture_dimension = limits.max_image_dimension2_d;
        let max_bound_descriptor_sets = limits.max_bound_descriptor_sets;

        // Query supported features
        let supported_extensions = physical_device.supported_extensions();
        let supports_descriptor_indexing = supported_extensions.ext_descriptor_indexing;

        // Note: Timeline semaphores require checking device features, which requires creating a device
        // For now, we'll estimate based on API version (timeline semaphores are in Vulkan 1.2+)
        let supports_timeline_semaphores = properties.api_version >= vulkano::Version::V1_2;

        // Estimate compute units based on vendor
        // This is a rough estimation since Vulkan doesn't directly expose this
        let compute_units = Self::estimate_compute_units(vendor, device_type, total_vram);

        let device_name = properties.device_name.clone();
        let api_version = properties.api_version.into();
        let driver_version = properties.driver_version;
        let vendor_id = properties.vendor_id;
        let device_id = properties.device_id;

        // Determine hardware tier
        let tier = Self::determine_tier(
            device_type,
            vendor,
            total_vram,
            compute_units,
            &device_name,
        );

        info!("Hardware Detection Results:");
        info!("  Device: {}", device_name);
        info!("  Vendor: {} (0x{:04X})", vendor.name(), vendor_id);
        info!("  Device ID: 0x{:04X}", device_id);
        info!("  Type: {:?}", device_type);
        info!("  Total VRAM: {} MB", total_vram / (1024 * 1024));
        info!("  Estimated Compute Units: {}", compute_units);
        info!("  Max Texture Dimension: {}", max_texture_dimension);
        info!("  Descriptor Indexing: {}", if supports_descriptor_indexing { "supported" } else { "not supported" });
        info!("  Timeline Semaphores: {}", if supports_timeline_semaphores { "supported" } else { "not supported" });
        info!("  Detected Tier: {} - {}", tier.name(), tier.description());

        Self {
            vendor,
            device_type,
            total_vram,
            device_name,
            api_version,
            driver_version,
            vendor_id,
            device_id,
            tier,
            compute_units,
            max_texture_dimension,
            max_bound_descriptor_sets,
            supports_timeline_semaphores,
            supports_descriptor_indexing,
        }
    }

    /// Calculates total VRAM from memory properties.
    ///
    /// Sums all device-local memory heaps to get total VRAM.
    fn calculate_total_vram(
        memory_properties: &vulkano::device::physical::MemoryProperties,
    ) -> u64 {
        use vulkano::memory::MemoryPropertyFlags;

        memory_properties
            .memory_heaps
            .iter()
            .filter(|heap| {
                // Find device-local heaps (actual VRAM)
                memory_properties.memory_types.iter().any(|mem_type| {
                    mem_type.heap_index as usize == heap.index()
                        && mem_type
                            .property_flags
                            .intersects(MemoryPropertyFlags::DEVICE_LOCAL)
                })
            })
            .map(|heap| heap.size)
            .sum()
    }

    /// Estimates compute units based on vendor, type, and VRAM.
    ///
    /// This is a rough heuristic since Vulkan doesn't expose shader core counts directly.
    fn estimate_compute_units(
        vendor: GpuVendor,
        device_type: PhysicalDeviceType,
        vram: u64,
    ) -> u32 {
        let vram_gb = (vram / (1024 * 1024 * 1024)) as u32;

        match device_type {
            PhysicalDeviceType::IntegratedGpu => {
                // Integrated GPUs typically have 8-24 compute units
                match vendor {
                    GpuVendor::Intel => 24,
                    GpuVendor::Amd => 12,
                    GpuVendor::Apple => 16, // M1/M2 base
                    _ => 16,
                }
            }
            PhysicalDeviceType::DiscreteGpu => {
                // Discrete GPUs: estimate based on VRAM
                // This is very rough but gives a ballpark figure
                match vram_gb {
                    0..=2 => 16,   // Low-end
                    3..=4 => 24,   // Entry-level
                    5..=6 => 32,   // Mid-range
                    7..=8 => 48,   // Upper mid-range
                    9..=12 => 64,  // High-end
                    13..=16 => 80, // Enthusiast
                    _ => 96,       // Extreme/professional
                }
            }
            PhysicalDeviceType::VirtualGpu => 8,
            PhysicalDeviceType::Cpu => 4,
            _ => 16,
        }
    }

    /// Determines hardware tier based on GPU properties.
    ///
    /// Uses a combination of device type, VRAM, and vendor-specific heuristics
    /// to classify the GPU into the appropriate tier.
    fn determine_tier(
        device_type: PhysicalDeviceType,
        vendor: GpuVendor,
        vram: u64,
        compute_units: u32,
        device_name: &str,
    ) -> HardwareTier {
        let vram_gb = (vram / (1024 * 1024 * 1024)) as u32;
        let device_name_lower = device_name.to_lowercase();

        // Check for integrated GPU indicators
        if matches!(device_type, PhysicalDeviceType::IntegratedGpu)
            || device_name_lower.contains("integrated")
            || device_name_lower.contains("uhd")
            || device_name_lower.contains("iris")
        {
            // Exception: Apple Silicon and high-end integrated can be mobile tier
            if matches!(vendor, GpuVendor::Apple) && vram_gb >= 8 {
                debug!("Classified as Mobile tier (Apple Silicon with sufficient VRAM)");
                return HardwareTier::Mobile;
            }

            debug!("Classified as Integrated tier (device type or name match)");
            return HardwareTier::Integrated;
        }

        // Check for mobile GPU indicators
        if device_name_lower.contains("mobile")
            || device_name_lower.contains("laptop")
            || device_name_lower.contains("max-q")
            || device_name_lower.contains("notebook")
        {
            debug!("Classified as Mobile tier (mobile GPU detected in name)");
            return HardwareTier::Mobile;
        }

        // For discrete GPUs, use VRAM as primary indicator
        match device_type {
            PhysicalDeviceType::DiscreteGpu => {
                if vram_gb <= 2 {
                    debug!("Classified as Mobile tier (discrete GPU with <= 2GB VRAM)");
                    HardwareTier::Mobile
                } else if vram_gb <= 6 {
                    // 3-6GB range: check compute units to distinguish mobile from mid-range
                    if compute_units < 28 {
                        debug!("Classified as Mobile tier (3-6GB VRAM, low compute units)");
                        HardwareTier::Mobile
                    } else {
                        debug!("Classified as MidRange tier (3-6GB VRAM, adequate compute units)");
                        HardwareTier::MidRange
                    }
                } else if vram_gb <= 10 {
                    debug!("Classified as MidRange tier (6-10GB VRAM)");
                    HardwareTier::MidRange
                } else {
                    // 12GB+: high-end or better
                    if vram_gb >= 16 && compute_units >= 80 {
                        debug!("Classified as HighEnd tier (>= 16GB VRAM, high compute units)");
                        HardwareTier::HighEnd
                    } else if vram_gb >= 12 {
                        debug!("Classified as HighEnd tier (>= 12GB VRAM)");
                        HardwareTier::HighEnd
                    } else {
                        debug!("Classified as MidRange tier (default for 10-12GB)");
                        HardwareTier::MidRange
                    }
                }
            }
            PhysicalDeviceType::VirtualGpu | PhysicalDeviceType::Cpu => {
                debug!("Classified as Integrated tier (virtual/CPU device)");
                HardwareTier::Integrated
            }
            _ => {
                warn!("Unknown device type, defaulting to MidRange tier");
                HardwareTier::MidRange
            }
        }
    }

    /// Gets the recommended quality preset for the detected hardware tier.
    pub fn recommended_preset(&self) -> QualityPreset {
        QualityPreset::from_tier(self.tier)
    }

    /// Gets the recommended configuration for the detected hardware tier.
    pub fn recommended_config(&self) -> HardwareTierConfig {
        HardwareTierConfig::from_preset(self.recommended_preset())
    }

    /// Returns a detailed summary of the detected hardware.
    pub fn summary(&self) -> String {
        format!(
            "GPU Hardware Summary:\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
             Device: {}\n\
             Vendor: {} (ID: 0x{:04X})\n\
             Device ID: 0x{:04X}\n\
             Type: {:?}\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
             VRAM: {} MB ({:.2} GB)\n\
             Compute Units (est.): {}\n\
             Max Texture Size: {}x{}\n\
             Max Descriptor Sets: {}\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
             API Version: {}.{}.{}\n\
             Driver Version: 0x{:08X}\n\
             Descriptor Indexing: {}\n\
             Timeline Semaphores: {}\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
             Detected Tier: {}\n\
             Description: {}\n\
             Recommended Preset: {}\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            self.device_name,
            self.vendor.name(),
            self.vendor_id,
            self.device_id,
            self.device_type,
            self.total_vram / (1024 * 1024),
            self.total_vram as f64 / (1024.0 * 1024.0 * 1024.0),
            self.compute_units,
            self.max_texture_dimension,
            self.max_texture_dimension,
            self.max_bound_descriptor_sets,
            (self.api_version >> 22) & 0x3FF,
            (self.api_version >> 12) & 0x3FF,
            self.api_version & 0xFFF,
            self.driver_version,
            if self.supports_descriptor_indexing {
                "Yes"
            } else {
                "No"
            },
            if self.supports_timeline_semaphores {
                "Yes"
            } else {
                "No"
            },
            self.tier.name(),
            self.tier.description(),
            self.recommended_preset().name()
        )
    }

    /// Checks if the GPU supports a specific feature based on tier.
    pub fn supports_feature(&self, feature: &str) -> bool {
        match feature {
            "ray_tracing" => matches!(self.tier, HardwareTier::HighEnd),
            "mesh_shaders" => matches!(self.tier, HardwareTier::MidRange | HardwareTier::HighEnd),
            "variable_rate_shading" => {
                matches!(self.tier, HardwareTier::MidRange | HardwareTier::HighEnd)
            }
            "hiz_occlusion" => {
                !matches!(self.tier, HardwareTier::Integrated)
            }
            "advanced_post_processing" => {
                !matches!(self.tier, HardwareTier::Integrated)
            }
            _ => false,
        }
    }

    /// Returns the tier classification.
    pub fn get_tier(&self) -> HardwareTier {
        self.tier
    }

    /// Checks if this is a mobile GPU.
    pub fn is_mobile(&self) -> bool {
        matches!(self.tier, HardwareTier::Mobile)
    }

    /// Checks if this is an integrated GPU.
    pub fn is_integrated(&self) -> bool {
        matches!(self.tier, HardwareTier::Integrated)
            || matches!(self.device_type, PhysicalDeviceType::IntegratedGpu)
    }

    /// Checks if this is a discrete GPU.
    pub fn is_discrete(&self) -> bool {
        matches!(self.device_type, PhysicalDeviceType::DiscreteGpu)
    }

    /// Checks if this is a high-end GPU.
    pub fn is_high_end(&self) -> bool {
        matches!(self.tier, HardwareTier::HighEnd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_identification() {
        assert_eq!(GpuVendor::from_vendor_id(0x1002), GpuVendor::Amd);
        assert_eq!(GpuVendor::from_vendor_id(0x10DE), GpuVendor::Nvidia);
        assert_eq!(GpuVendor::from_vendor_id(0x8086), GpuVendor::Intel);
        assert_eq!(GpuVendor::from_vendor_id(0x106B), GpuVendor::Apple);
        assert_eq!(GpuVendor::from_vendor_id(0x13B5), GpuVendor::Arm);
        assert_eq!(GpuVendor::from_vendor_id(0x5143), GpuVendor::Qualcomm);
        assert_eq!(GpuVendor::from_vendor_id(0xFFFF), GpuVendor::Unknown);
    }

    #[test]
    fn test_vendor_names() {
        assert_eq!(GpuVendor::Nvidia.name(), "NVIDIA");
        assert_eq!(GpuVendor::Amd.name(), "AMD");
        assert_eq!(GpuVendor::Intel.name(), "Intel");
        assert_eq!(GpuVendor::Apple.name(), "Apple");
        assert_eq!(GpuVendor::Arm.name(), "ARM");
        assert_eq!(GpuVendor::Qualcomm.name(), "Qualcomm");
        assert_eq!(GpuVendor::Unknown.name(), "Unknown");
    }

    #[test]
    fn test_hardware_tier_names() {
        assert_eq!(HardwareTier::Integrated.name(), "Integrated");
        assert_eq!(HardwareTier::Mobile.name(), "Mobile");
        assert_eq!(HardwareTier::MidRange.name(), "Mid-Range");
        assert_eq!(HardwareTier::HighEnd.name(), "High-End");
    }

    #[test]
    fn test_quality_preset_from_tier() {
        assert_eq!(
            QualityPreset::from_tier(HardwareTier::Integrated),
            QualityPreset::Low
        );
        assert_eq!(
            QualityPreset::from_tier(HardwareTier::Mobile),
            QualityPreset::Medium
        );
        assert_eq!(
            QualityPreset::from_tier(HardwareTier::MidRange),
            QualityPreset::High
        );
        assert_eq!(
            QualityPreset::from_tier(HardwareTier::HighEnd),
            QualityPreset::Ultra
        );
    }

    #[test]
    fn test_quality_preset_names() {
        assert_eq!(QualityPreset::Low.name(), "Low");
        assert_eq!(QualityPreset::Medium.name(), "Medium");
        assert_eq!(QualityPreset::High.name(), "High");
        assert_eq!(QualityPreset::Ultra.name(), "Ultra");
    }

    #[test]
    fn test_low_config() {
        let config = HardwareTierConfig::low();
        assert_eq!(config.shadow_resolution, 512);
        assert_eq!(config.shadow_cascades, 2);
        assert!(!config.enable_soft_shadows);
        assert_eq!(config.lod_bias, -0.5);
        assert!(!config.enable_ssao);
        assert!(!config.enable_ssr);
        assert_eq!(config.msaa_samples, 1);
        assert_eq!(config.target_fps, 30.0);
    }

    #[test]
    fn test_medium_config() {
        let config = HardwareTierConfig::medium();
        assert_eq!(config.shadow_resolution, 1024);
        assert_eq!(config.shadow_cascades, 3);
        assert!(config.enable_soft_shadows);
        assert_eq!(config.lod_bias, 0.0);
        assert!(config.enable_ssao);
        assert!(!config.enable_ssr);
        assert_eq!(config.msaa_samples, 2);
        assert_eq!(config.target_fps, 60.0);
    }

    #[test]
    fn test_high_config() {
        let config = HardwareTierConfig::high();
        assert_eq!(config.shadow_resolution, 2048);
        assert_eq!(config.shadow_cascades, 4);
        assert!(config.enable_soft_shadows);
        assert_eq!(config.lod_bias, 0.3);
        assert!(config.enable_ssao);
        assert!(config.enable_ssr);
        assert!(config.enable_taa);
        assert_eq!(config.msaa_samples, 4);
        assert_eq!(config.target_fps, 60.0);
    }

    #[test]
    fn test_ultra_config() {
        let config = HardwareTierConfig::ultra();
        assert_eq!(config.shadow_resolution, 4096);
        assert_eq!(config.shadow_cascades, 4);
        assert!(config.enable_soft_shadows);
        assert_eq!(config.lod_bias, 0.5);
        assert!(config.enable_ssao);
        assert!(config.enable_ssr);
        assert!(config.enable_taa);
        assert_eq!(config.msaa_samples, 8);
        assert_eq!(config.max_particles, 50000);
        assert_eq!(config.target_fps, 60.0);
    }

    #[test]
    fn test_config_from_preset() {
        let low = HardwareTierConfig::from_preset(QualityPreset::Low);
        assert_eq!(low.shadow_resolution, 512);

        let medium = HardwareTierConfig::from_preset(QualityPreset::Medium);
        assert_eq!(medium.shadow_resolution, 1024);

        let high = HardwareTierConfig::from_preset(QualityPreset::High);
        assert_eq!(high.shadow_resolution, 2048);

        let ultra = HardwareTierConfig::from_preset(QualityPreset::Ultra);
        assert_eq!(ultra.shadow_resolution, 4096);
    }

    #[test]
    fn test_compute_units_estimation() {
        // Integrated GPUs
        assert_eq!(
            HardwareTierDetector::estimate_compute_units(
                GpuVendor::Intel,
                PhysicalDeviceType::IntegratedGpu,
                2 * 1024 * 1024 * 1024
            ),
            24
        );
        assert_eq!(
            HardwareTierDetector::estimate_compute_units(
                GpuVendor::Amd,
                PhysicalDeviceType::IntegratedGpu,
                2 * 1024 * 1024 * 1024
            ),
            12
        );

        // Discrete GPUs with varying VRAM
        assert_eq!(
            HardwareTierDetector::estimate_compute_units(
                GpuVendor::Nvidia,
                PhysicalDeviceType::DiscreteGpu,
                2 * 1024 * 1024 * 1024
            ),
            16
        );
        assert_eq!(
            HardwareTierDetector::estimate_compute_units(
                GpuVendor::Nvidia,
                PhysicalDeviceType::DiscreteGpu,
                8 * 1024 * 1024 * 1024
            ),
            48
        );
        assert_eq!(
            HardwareTierDetector::estimate_compute_units(
                GpuVendor::Nvidia,
                PhysicalDeviceType::DiscreteGpu,
                16 * 1024 * 1024 * 1024
            ),
            80
        );
    }

    #[test]
    fn test_tier_determination() {
        // Integrated GPU
        let tier = HardwareTierDetector::determine_tier(
            PhysicalDeviceType::IntegratedGpu,
            GpuVendor::Intel,
            2 * 1024 * 1024 * 1024,
            24,
            "Intel UHD Graphics 620",
        );
        assert_eq!(tier, HardwareTier::Integrated);

        // Low VRAM discrete GPU
        let tier = HardwareTierDetector::determine_tier(
            PhysicalDeviceType::DiscreteGpu,
            GpuVendor::Nvidia,
            2 * 1024 * 1024 * 1024,
            16,
            "NVIDIA GTX 1050",
        );
        assert_eq!(tier, HardwareTier::Mobile);

        // Mid-range discrete GPU
        let tier = HardwareTierDetector::determine_tier(
            PhysicalDeviceType::DiscreteGpu,
            GpuVendor::Nvidia,
            6 * 1024 * 1024 * 1024,
            32,
            "NVIDIA RTX 3060",
        );
        assert_eq!(tier, HardwareTier::MidRange);

        // High-end discrete GPU
        let tier = HardwareTierDetector::determine_tier(
            PhysicalDeviceType::DiscreteGpu,
            GpuVendor::Nvidia,
            12 * 1024 * 1024 * 1024,
            68,
            "NVIDIA RTX 4070",
        );
        assert_eq!(tier, HardwareTier::HighEnd);

        // Mobile GPU by name
        let tier = HardwareTierDetector::determine_tier(
            PhysicalDeviceType::DiscreteGpu,
            GpuVendor::Nvidia,
            4 * 1024 * 1024 * 1024,
            28,
            "NVIDIA GTX 1660 Ti Mobile",
        );
        assert_eq!(tier, HardwareTier::Mobile);
    }

    #[test]
    fn test_serialization() {
        let config = HardwareTierConfig::high();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: HardwareTierConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.shadow_resolution, deserialized.shadow_resolution);
        assert_eq!(config.lod_bias, deserialized.lod_bias);
        assert_eq!(config.enable_ssao, deserialized.enable_ssao);
    }
}
