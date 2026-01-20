//! Rendering utilities and supporting systems.
//!
//! This module consolidates various utility systems that support the main rendering pipeline:
//! - **Optimization Configuration**: Runtime toggles for rendering optimizations
//! - **Render Statistics**: Performance tracking and metrics collection
//! - **Velocity Buffers**: Motion vector generation for motion blur
//! - **Light Linking**: Channel-based light-object interaction control
//! - **Light Probes**: Dynamic global illumination using spherical harmonics
//! - **Hardware Tier Detection**: GPU capability detection and quality preset selection
//!
//! These systems are grouped together as they provide supporting functionality rather than
//! being core rendering features. They can be used independently or in combination.

pub mod hardware_tier;
pub mod light_linking;
pub mod light_probe;
pub mod optimization_config;
pub mod render_stats;
pub mod velocity_buffer;

// Re-export commonly used types for convenience
pub use hardware_tier::{
    GpuVendor, HardwareTier, HardwareTierConfig, HardwareTierDetector, QualityPreset,
};
pub use light_linking::{
    LightChannel, LightLinkingManager, LightLinkingMask, DEFAULT_LIGHT_CHANNEL,
};
pub use light_probe::{
    LightProbe, LightProbeData, LightProbeGrid, LightProbeManager, ProbeBlendMode,
    MAX_LIGHT_PROBES, PROBE_IRRADIANCE_COEFFS,
};
pub use optimization_config::RenderingOptimizationConfig;
pub use render_stats::{
    CullingBreakdown, RenderStats, RenderStatsHistory, RenderStatsVisualizer, StatsSummary,
};
pub use velocity_buffer::{VelocityBuffer, VelocityBufferRenderer};
