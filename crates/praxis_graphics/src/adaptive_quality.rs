//! Adaptive Quality System for automatic rendering optimization.
//!
//! This module provides a system that automatically adjusts rendering quality settings
//! based on recent frame time history to maintain a target FPS. The system can adapt:
//!
//! - **LOD Bias**: Adjusts level-of-detail selection to prefer higher or lower detail meshes
//! - **Mesh Streaming Priority Thresholds**: Changes which meshes are loaded first
//! - **Shadow Map Resolution**: Dynamically scales shadow quality
//!
//! # How It Works
//!
//! The system maintains a moving average of recent frame times and compares it to a target
//! frame time (derived from target FPS). When GPU-bound (slow frames), it reduces quality.
//! When under budget (fast frames), it increases quality.
//!
//! # Quality Adjustments
//!
//! - **LOD Bias**: Ranges from -1.0 (lowest quality) to +1.0 (highest quality)
//!   - Negative values push objects to use lower-detail LOD levels
//!   - Positive values push objects to use higher-detail LOD levels
//!
//! - **Streaming Priority Threshold**: Controls which meshes get loaded
//!   - Lower threshold = more aggressive loading (higher quality, more memory)
//!   - Higher threshold = conservative loading (lower quality, less memory)
//!
//! - **Shadow Resolution**: Scales the shadow map size
//!   - Ranges from min_shadow_resolution to max_shadow_resolution
//!   - Must be power of two (512, 1024, 2048, 4096)
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use praxis_graphics::adaptive_quality::{AdaptiveQualitySystem, AdaptiveQualityConfig};
//!
//! // Create adaptive quality system targeting 60 FPS
//! let mut quality_system = AdaptiveQualitySystem::new(AdaptiveQualityConfig {
//!     target_fps: 60.0,
//!     ..Default::default()
//! });
//!
//! // Each frame, update with current frame time
//! quality_system.update(frame_time_seconds);
//!
//! // Apply the computed LOD bias to the LOD manager
//! lod_manager.set_global_lod_bias(quality_system.lod_bias());
//!
//! // Use streaming priority threshold
//! if mesh_priority > quality_system.streaming_priority_threshold() {
//!     mesh_streaming.load_mesh(mesh_id);
//! }
//!
//! // Check if shadow resolution changed
//! if quality_system.shadow_resolution_changed() {
//!     let new_resolution = quality_system.shadow_resolution();
//!     shadow_manager.update_resolution(new_resolution);
//! }
//! ```

use praxis_utils::{debug, info, trace};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Configuration for the adaptive quality system.
///
/// This defines the target performance and the ranges within which quality
/// can be adjusted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveQualityConfig {
    /// Target frames per second to maintain.
    ///
    /// The system will try to keep frame times at or below 1.0 / target_fps seconds.
    pub target_fps: f64,

    /// Number of frames to average for smoothing frame time measurements.
    ///
    /// Higher values provide more stable adjustments but react slower to changes.
    /// Typical values: 30-120 frames (0.5-2 seconds at 60 FPS).
    pub frame_history_size: usize,

    /// Minimum LOD bias value (most aggressive quality reduction).
    ///
    /// Range: -1.0 to 0.0
    /// Default: -1.0 (maximum quality reduction)
    pub min_lod_bias: f32,

    /// Maximum LOD bias value (highest quality).
    ///
    /// Range: 0.0 to 1.0
    /// Default: 0.5 (prefer higher detail but not maximum)
    pub max_lod_bias: f32,

    /// Initial LOD bias value.
    ///
    /// Default: 0.0 (neutral)
    pub initial_lod_bias: f32,

    /// Rate at which LOD bias changes per adjustment.
    ///
    /// Smaller values = smoother transitions, slower adaptation
    /// Larger values = faster adaptation, potentially more visible changes
    /// Default: 0.05 (5% change per adjustment)
    pub lod_bias_adjustment_rate: f32,

    /// Minimum mesh streaming priority threshold.
    ///
    /// Lower values = load more meshes (higher quality, more memory)
    /// Default: 0.0 (load even low-priority meshes)
    pub min_streaming_priority_threshold: f32,

    /// Maximum mesh streaming priority threshold.
    ///
    /// Higher values = load fewer meshes (lower quality, less memory)
    /// Default: 100.0 (only load very high priority meshes)
    pub max_streaming_priority_threshold: f32,

    /// Initial streaming priority threshold.
    ///
    /// Default: 10.0 (moderate threshold)
    pub initial_streaming_priority_threshold: f32,

    /// Rate at which streaming priority threshold changes.
    ///
    /// Default: 5.0
    pub streaming_threshold_adjustment_rate: f32,

    /// Minimum shadow map resolution (must be power of two).
    ///
    /// Default: 512
    pub min_shadow_resolution: u32,

    /// Maximum shadow map resolution (must be power of two).
    ///
    /// Default: 2048
    pub max_shadow_resolution: u32,

    /// Initial shadow map resolution (must be power of two).
    ///
    /// Default: 1024
    pub initial_shadow_resolution: u32,

    /// Tolerance for being under the target frame time (as a fraction).
    ///
    /// If frame time < target * (1.0 - under_budget_threshold), quality increases.
    /// Default: 0.1 (10% faster than target = room for quality increase)
    pub under_budget_threshold: f32,

    /// Tolerance for being over the target frame time (as a fraction).
    ///
    /// If frame time > target * (1.0 + over_budget_threshold), quality decreases.
    /// Default: 0.05 (5% slower than target = quality reduction needed)
    pub over_budget_threshold: f32,

    /// Enable adaptive LOD bias adjustment.
    ///
    /// Default: true
    pub enable_lod_adjustment: bool,

    /// Enable adaptive streaming priority threshold adjustment.
    ///
    /// Default: true
    pub enable_streaming_adjustment: bool,

    /// Enable adaptive shadow resolution adjustment.
    ///
    /// Default: true
    pub enable_shadow_resolution_adjustment: bool,
}

impl Default for AdaptiveQualityConfig {
    fn default() -> Self {
        Self {
            target_fps: 60.0,
            frame_history_size: 60,
            min_lod_bias: -1.0,
            max_lod_bias: 0.5,
            initial_lod_bias: 0.0,
            lod_bias_adjustment_rate: 0.05,
            min_streaming_priority_threshold: 0.0,
            max_streaming_priority_threshold: 100.0,
            initial_streaming_priority_threshold: 10.0,
            streaming_threshold_adjustment_rate: 5.0,
            min_shadow_resolution: 512,
            max_shadow_resolution: 2048,
            initial_shadow_resolution: 1024,
            under_budget_threshold: 0.1,
            over_budget_threshold: 0.05,
            enable_lod_adjustment: true,
            enable_streaming_adjustment: true,
            enable_shadow_resolution_adjustment: true,
        }
    }
}

impl AdaptiveQualityConfig {
    /// Validates the configuration and returns an error if invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.target_fps <= 0.0 {
            return Err("target_fps must be positive".to_string());
        }

        if self.frame_history_size == 0 {
            return Err("frame_history_size must be at least 1".to_string());
        }

        if self.min_lod_bias < -1.0 || self.min_lod_bias > 1.0 {
            return Err("min_lod_bias must be in range [-1.0, 1.0]".to_string());
        }

        if self.max_lod_bias < -1.0 || self.max_lod_bias > 1.0 {
            return Err("max_lod_bias must be in range [-1.0, 1.0]".to_string());
        }

        if self.min_lod_bias > self.max_lod_bias {
            return Err("min_lod_bias must be <= max_lod_bias".to_string());
        }

        if !is_power_of_two(self.min_shadow_resolution) {
            return Err("min_shadow_resolution must be power of two".to_string());
        }

        if !is_power_of_two(self.max_shadow_resolution) {
            return Err("max_shadow_resolution must be power of two".to_string());
        }

        if !is_power_of_two(self.initial_shadow_resolution) {
            return Err("initial_shadow_resolution must be power of two".to_string());
        }

        if self.min_shadow_resolution > self.max_shadow_resolution {
            return Err("min_shadow_resolution must be <= max_shadow_resolution".to_string());
        }

        if self.under_budget_threshold < 0.0 || self.under_budget_threshold > 1.0 {
            return Err("under_budget_threshold must be in range [0.0, 1.0]".to_string());
        }

        if self.over_budget_threshold < 0.0 || self.over_budget_threshold > 1.0 {
            return Err("over_budget_threshold must be in range [0.0, 1.0]".to_string());
        }

        Ok(())
    }
}

/// Adaptive quality system that adjusts rendering parameters based on frame time.
///
/// This system monitors frame times and automatically adjusts quality settings to
/// maintain the target FPS. It uses a moving average to smooth out fluctuations
/// and avoid rapid quality changes.
pub struct AdaptiveQualitySystem {
    /// Configuration settings.
    config: AdaptiveQualityConfig,

    /// Ring buffer of recent frame times (in seconds).
    frame_time_history: VecDeque<f32>,

    /// Current LOD bias value.
    lod_bias: f32,

    /// Current mesh streaming priority threshold.
    streaming_priority_threshold: f32,

    /// Current shadow map resolution.
    shadow_resolution: u32,

    /// Previous shadow map resolution (for detecting changes).
    previous_shadow_resolution: u32,

    /// Total number of quality adjustments made (for statistics).
    adjustment_count: u64,

    /// Number of times quality was reduced.
    reduction_count: u64,

    /// Number of times quality was increased.
    increase_count: u64,

    /// Whether the system is enabled.
    enabled: bool,
}

impl AdaptiveQualitySystem {
    /// Creates a new adaptive quality system with the given configuration.
    ///
    /// # Panics
    ///
    /// Panics if the configuration is invalid.
    pub fn new(config: AdaptiveQualityConfig) -> Self {
        config
            .validate()
            .expect("Invalid adaptive quality configuration");

        info!(
            "Creating adaptive quality system targeting {:.1} FPS",
            config.target_fps
        );
        info!(
            "  LOD bias range: [{:.2}, {:.2}]",
            config.min_lod_bias, config.max_lod_bias
        );
        info!(
            "  Streaming priority range: [{:.1}, {:.1}]",
            config.min_streaming_priority_threshold, config.max_streaming_priority_threshold
        );
        info!(
            "  Shadow resolution range: [{}x{}, {}x{}]",
            config.min_shadow_resolution,
            config.min_shadow_resolution,
            config.max_shadow_resolution,
            config.max_shadow_resolution
        );

        let lod_bias = config.initial_lod_bias;
        let streaming_priority_threshold = config.initial_streaming_priority_threshold;
        let shadow_resolution = config.initial_shadow_resolution;

        Self {
            config,
            frame_time_history: VecDeque::new(),
            lod_bias,
            streaming_priority_threshold,
            shadow_resolution,
            previous_shadow_resolution: shadow_resolution,
            adjustment_count: 0,
            reduction_count: 0,
            increase_count: 0,
            enabled: true,
        }
    }

    /// Updates the adaptive quality system with the current frame time.
    ///
    /// This should be called once per frame. The system will adjust quality
    /// settings based on the average frame time over recent frames.
    ///
    /// # Arguments
    ///
    /// * `frame_time_seconds` - Time taken to render the current frame (in seconds)
    pub fn update(&mut self, frame_time_seconds: f32) {
        if !self.enabled {
            return;
        }

        // Add frame time to history
        self.frame_time_history.push_back(frame_time_seconds);

        // Keep history size bounded
        while self.frame_time_history.len() > self.config.frame_history_size {
            self.frame_time_history.pop_front();
        }

        // Need enough history before making adjustments
        if self.frame_time_history.len() < self.config.frame_history_size / 2 {
            return;
        }

        // Calculate average frame time
        let avg_frame_time = self.average_frame_time();
        let target_frame_time = 1.0 / self.config.target_fps as f32;

        trace!(
            "Adaptive quality: avg={:.4}s, target={:.4}s",
            avg_frame_time,
            target_frame_time
        );

        // Determine if we need to adjust quality
        let under_budget_threshold = target_frame_time * (1.0 - self.config.under_budget_threshold);
        let over_budget_threshold = target_frame_time * (1.0 + self.config.over_budget_threshold);

        if avg_frame_time > over_budget_threshold {
            // GPU-bound: reduce quality
            self.reduce_quality();
        } else if avg_frame_time < under_budget_threshold {
            // Under budget: increase quality
            self.increase_quality();
        }
    }

    /// Reduces rendering quality to improve frame rate.
    fn reduce_quality(&mut self) {
        let mut adjusted = false;

        // Reduce LOD bias (prefer lower detail meshes)
        if self.config.enable_lod_adjustment && self.lod_bias > self.config.min_lod_bias {
            let old_bias = self.lod_bias;
            self.lod_bias = (self.lod_bias - self.config.lod_bias_adjustment_rate)
                .max(self.config.min_lod_bias);

            if self.lod_bias != old_bias {
                debug!("Reduced LOD bias: {:.3} -> {:.3}", old_bias, self.lod_bias);
                adjusted = true;
            }
        }

        // Increase streaming threshold (load fewer meshes)
        if self.config.enable_streaming_adjustment
            && self.streaming_priority_threshold < self.config.max_streaming_priority_threshold
        {
            let old_threshold = self.streaming_priority_threshold;
            self.streaming_priority_threshold = (self.streaming_priority_threshold
                + self.config.streaming_threshold_adjustment_rate)
                .min(self.config.max_streaming_priority_threshold);

            if self.streaming_priority_threshold != old_threshold {
                debug!(
                    "Increased streaming threshold: {:.1} -> {:.1}",
                    old_threshold, self.streaming_priority_threshold
                );
                adjusted = true;
            }
        }

        // Reduce shadow resolution (step down to next power of two)
        if self.config.enable_shadow_resolution_adjustment
            && self.shadow_resolution > self.config.min_shadow_resolution
        {
            let old_resolution = self.shadow_resolution;
            self.shadow_resolution =
                (self.shadow_resolution / 2).max(self.config.min_shadow_resolution);

            if self.shadow_resolution != old_resolution {
                info!(
                    "Reduced shadow resolution: {}x{} -> {}x{}",
                    old_resolution, old_resolution, self.shadow_resolution, self.shadow_resolution
                );
                adjusted = true;
            }
        }

        if adjusted {
            self.adjustment_count += 1;
            self.reduction_count += 1;
            trace!(
                "Quality reduced (total adjustments: {})",
                self.adjustment_count
            );
        }
    }

    /// Increases rendering quality when performance budget allows.
    fn increase_quality(&mut self) {
        let mut adjusted = false;

        // Increase LOD bias (prefer higher detail meshes)
        if self.config.enable_lod_adjustment && self.lod_bias < self.config.max_lod_bias {
            let old_bias = self.lod_bias;
            self.lod_bias = (self.lod_bias + self.config.lod_bias_adjustment_rate)
                .min(self.config.max_lod_bias);

            if self.lod_bias != old_bias {
                debug!(
                    "Increased LOD bias: {:.3} -> {:.3}",
                    old_bias, self.lod_bias
                );
                adjusted = true;
            }
        }

        // Decrease streaming threshold (load more meshes)
        if self.config.enable_streaming_adjustment
            && self.streaming_priority_threshold > self.config.min_streaming_priority_threshold
        {
            let old_threshold = self.streaming_priority_threshold;
            self.streaming_priority_threshold = (self.streaming_priority_threshold
                - self.config.streaming_threshold_adjustment_rate)
                .max(self.config.min_streaming_priority_threshold);

            if self.streaming_priority_threshold != old_threshold {
                debug!(
                    "Decreased streaming threshold: {:.1} -> {:.1}",
                    old_threshold, self.streaming_priority_threshold
                );
                adjusted = true;
            }
        }

        // Increase shadow resolution (step up to next power of two)
        if self.config.enable_shadow_resolution_adjustment
            && self.shadow_resolution < self.config.max_shadow_resolution
        {
            let old_resolution = self.shadow_resolution;
            self.shadow_resolution =
                (self.shadow_resolution * 2).min(self.config.max_shadow_resolution);

            if self.shadow_resolution != old_resolution {
                info!(
                    "Increased shadow resolution: {}x{} -> {}x{}",
                    old_resolution, old_resolution, self.shadow_resolution, self.shadow_resolution
                );
                adjusted = true;
            }
        }

        if adjusted {
            self.adjustment_count += 1;
            self.increase_count += 1;
            trace!(
                "Quality increased (total adjustments: {})",
                self.adjustment_count
            );
        }
    }

    /// Calculates the average frame time from recent history.
    fn average_frame_time(&self) -> f32 {
        if self.frame_time_history.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.frame_time_history.iter().sum();
        sum / self.frame_time_history.len() as f32
    }

    /// Gets the current LOD bias.
    ///
    /// This value should be applied to the LOD manager's global bias.
    pub fn lod_bias(&self) -> f32 {
        self.lod_bias
    }

    /// Gets the current mesh streaming priority threshold.
    ///
    /// Meshes with priority below this threshold should not be loaded.
    pub fn streaming_priority_threshold(&self) -> f32 {
        self.streaming_priority_threshold
    }

    /// Gets the current shadow map resolution.
    pub fn shadow_resolution(&self) -> u32 {
        self.shadow_resolution
    }

    /// Checks if the shadow resolution has changed since the last check.
    ///
    /// This can be used to detect when shadow maps need to be recreated.
    /// After calling this method and handling the change, call
    /// `clear_shadow_resolution_changed()` to reset the flag.
    pub fn shadow_resolution_changed(&self) -> bool {
        self.shadow_resolution != self.previous_shadow_resolution
    }

    /// Clears the shadow resolution changed flag.
    ///
    /// Call this after handling a shadow resolution change.
    pub fn clear_shadow_resolution_changed(&mut self) {
        self.previous_shadow_resolution = self.shadow_resolution;
    }

    /// Gets the average frame time from recent history.
    pub fn average_frame_time_ms(&self) -> f32 {
        self.average_frame_time() * 1000.0
    }

    /// Gets the target frame time in milliseconds.
    pub fn target_frame_time_ms(&self) -> f32 {
        (1.0 / self.config.target_fps as f32) * 1000.0
    }

    /// Gets the current average FPS based on frame time history.
    pub fn current_fps(&self) -> f32 {
        let avg_time = self.average_frame_time();
        if avg_time > 0.0 {
            1.0 / avg_time
        } else {
            0.0
        }
    }

    /// Gets statistics about quality adjustments.
    pub fn statistics(&self) -> AdaptiveQualityStatistics {
        AdaptiveQualityStatistics {
            adjustment_count: self.adjustment_count,
            reduction_count: self.reduction_count,
            increase_count: self.increase_count,
            current_lod_bias: self.lod_bias,
            current_streaming_threshold: self.streaming_priority_threshold,
            current_shadow_resolution: self.shadow_resolution,
            average_frame_time_ms: self.average_frame_time_ms(),
            current_fps: self.current_fps(),
            target_fps: self.config.target_fps as f32,
        }
    }

    /// Enables or disables the adaptive quality system.
    pub fn set_enabled(&mut self, enabled: bool) {
        if self.enabled != enabled {
            self.enabled = enabled;
            if enabled {
                info!("Adaptive quality system enabled");
            } else {
                info!("Adaptive quality system disabled");
            }
        }
    }

    /// Checks if the adaptive quality system is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Resets the adaptive quality system to initial state.
    pub fn reset(&mut self) {
        info!("Resetting adaptive quality system");

        self.frame_time_history.clear();
        self.lod_bias = self.config.initial_lod_bias;
        self.streaming_priority_threshold = self.config.initial_streaming_priority_threshold;
        self.shadow_resolution = self.config.initial_shadow_resolution;
        self.previous_shadow_resolution = self.shadow_resolution;
        self.adjustment_count = 0;
        self.reduction_count = 0;
        self.increase_count = 0;
    }

    /// Gets a reference to the configuration.
    pub fn config(&self) -> &AdaptiveQualityConfig {
        &self.config
    }

    /// Updates the configuration.
    ///
    /// # Panics
    ///
    /// Panics if the new configuration is invalid.
    pub fn set_config(&mut self, config: AdaptiveQualityConfig) {
        config
            .validate()
            .expect("Invalid adaptive quality configuration");

        info!("Updating adaptive quality configuration");
        self.config = config;

        // Clamp current values to new ranges
        self.lod_bias = self
            .lod_bias
            .clamp(self.config.min_lod_bias, self.config.max_lod_bias);
        self.streaming_priority_threshold = self.streaming_priority_threshold.clamp(
            self.config.min_streaming_priority_threshold,
            self.config.max_streaming_priority_threshold,
        );
        self.shadow_resolution = self.shadow_resolution.clamp(
            self.config.min_shadow_resolution,
            self.config.max_shadow_resolution,
        );
    }
}

/// Statistics about adaptive quality adjustments.
#[derive(Debug, Clone, Copy)]
pub struct AdaptiveQualityStatistics {
    /// Total number of quality adjustments made.
    pub adjustment_count: u64,

    /// Number of times quality was reduced.
    pub reduction_count: u64,

    /// Number of times quality was increased.
    pub increase_count: u64,

    /// Current LOD bias value.
    pub current_lod_bias: f32,

    /// Current streaming priority threshold.
    pub current_streaming_threshold: f32,

    /// Current shadow map resolution.
    pub current_shadow_resolution: u32,

    /// Average frame time in milliseconds.
    pub average_frame_time_ms: f32,

    /// Current FPS based on average frame time.
    pub current_fps: f32,

    /// Target FPS.
    pub target_fps: f32,
}

impl AdaptiveQualityStatistics {
    /// Returns a formatted string summary of the statistics.
    pub fn summary(&self) -> String {
        format!(
            "Adaptive Quality Statistics:\n\
             - Current FPS: {:.1} (target: {:.1})\n\
             - Avg frame time: {:.2}ms\n\
             - LOD bias: {:.3}\n\
             - Streaming threshold: {:.1}\n\
             - Shadow resolution: {}x{}\n\
             - Total adjustments: {}\n\
             - Quality reductions: {}\n\
             - Quality increases: {}",
            self.current_fps,
            self.target_fps,
            self.average_frame_time_ms,
            self.current_lod_bias,
            self.current_streaming_threshold,
            self.current_shadow_resolution,
            self.current_shadow_resolution,
            self.adjustment_count,
            self.reduction_count,
            self.increase_count
        )
    }
}

/// Helper function to check if a number is a power of two.
fn is_power_of_two(n: u32) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AdaptiveQualityConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.target_fps, 60.0);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AdaptiveQualityConfig::default();

        // Invalid target FPS
        config.target_fps = -1.0;
        assert!(config.validate().is_err());

        // Invalid LOD bias range
        config = AdaptiveQualityConfig::default();
        config.min_lod_bias = 1.0;
        config.max_lod_bias = -1.0;
        assert!(config.validate().is_err());

        // Invalid shadow resolution (not power of two)
        config = AdaptiveQualityConfig::default();
        config.min_shadow_resolution = 1000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_is_power_of_two() {
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(4));
        assert!(is_power_of_two(512));
        assert!(is_power_of_two(1024));
        assert!(is_power_of_two(2048));

        assert!(!is_power_of_two(0));
        assert!(!is_power_of_two(3));
        assert!(!is_power_of_two(1000));
    }

    #[test]
    fn test_system_creation() {
        let config = AdaptiveQualityConfig::default();
        let system = AdaptiveQualitySystem::new(config);

        assert_eq!(system.lod_bias(), 0.0);
        assert_eq!(system.streaming_priority_threshold(), 10.0);
        assert_eq!(system.shadow_resolution(), 1024);
        assert!(system.is_enabled());
    }

    #[test]
    fn test_quality_reduction_lod() {
        let config = AdaptiveQualityConfig {
            target_fps: 60.0,
            frame_history_size: 10,
            initial_lod_bias: 0.0,
            lod_bias_adjustment_rate: 0.1,
            enable_streaming_adjustment: false,
            enable_shadow_resolution_adjustment: false,
            ..Default::default()
        };

        let mut system = AdaptiveQualitySystem::new(config);

        // Simulate slow frames (GPU-bound)
        for _ in 0..20 {
            system.update(0.020); // 20ms = 50 FPS (below target of 60)
        }

        // LOD bias should have been reduced
        assert!(system.lod_bias() < 0.0);
    }

    #[test]
    fn test_quality_increase_lod() {
        let config = AdaptiveQualityConfig {
            target_fps: 60.0,
            frame_history_size: 10,
            initial_lod_bias: 0.0,
            lod_bias_adjustment_rate: 0.1,
            enable_streaming_adjustment: false,
            enable_shadow_resolution_adjustment: false,
            ..Default::default()
        };

        let mut system = AdaptiveQualitySystem::new(config);

        // Simulate fast frames (under budget)
        for _ in 0..20 {
            system.update(0.010); // 10ms = 100 FPS (well above target of 60)
        }

        // LOD bias should have been increased
        assert!(system.lod_bias() > 0.0);
    }

    #[test]
    fn test_shadow_resolution_reduction() {
        let config = AdaptiveQualityConfig {
            target_fps: 60.0,
            frame_history_size: 10,
            initial_shadow_resolution: 1024,
            min_shadow_resolution: 512,
            enable_lod_adjustment: false,
            enable_streaming_adjustment: false,
            ..Default::default()
        };

        let mut system = AdaptiveQualitySystem::new(config);

        // Simulate slow frames
        for _ in 0..20 {
            system.update(0.020); // 50 FPS
        }

        // Shadow resolution should have been reduced
        assert!(system.shadow_resolution() < 1024);
    }

    #[test]
    fn test_shadow_resolution_changed_flag() {
        let config = AdaptiveQualityConfig {
            target_fps: 60.0,
            frame_history_size: 10,
            enable_lod_adjustment: false,
            enable_streaming_adjustment: false,
            ..Default::default()
        };

        let mut system = AdaptiveQualitySystem::new(config);

        assert!(!system.shadow_resolution_changed());

        // Trigger quality reduction
        for _ in 0..20 {
            system.update(0.020);
        }

        if system.shadow_resolution() < 1024 {
            assert!(system.shadow_resolution_changed());
            system.clear_shadow_resolution_changed();
            assert!(!system.shadow_resolution_changed());
        }
    }

    #[test]
    fn test_reset() {
        let config = AdaptiveQualityConfig::default();
        let mut system = AdaptiveQualitySystem::new(config);

        // Make some adjustments
        for _ in 0..20 {
            system.update(0.020);
        }

        let stats_before = system.statistics();
        assert!(stats_before.adjustment_count > 0);

        system.reset();

        let stats_after = system.statistics();
        assert_eq!(stats_after.adjustment_count, 0);
        assert_eq!(system.lod_bias(), 0.0);
        assert_eq!(system.shadow_resolution(), 1024);
    }

    #[test]
    fn test_enable_disable() {
        let config = AdaptiveQualityConfig::default();
        let mut system = AdaptiveQualitySystem::new(config);

        assert!(system.is_enabled());

        system.set_enabled(false);
        assert!(!system.is_enabled());

        // Updates should not affect quality when disabled
        let initial_bias = system.lod_bias();
        for _ in 0..20 {
            system.update(0.020);
        }
        assert_eq!(system.lod_bias(), initial_bias);
    }

    #[test]
    fn test_statistics() {
        let config = AdaptiveQualityConfig::default();
        let system = AdaptiveQualitySystem::new(config);

        let stats = system.statistics();
        assert_eq!(stats.adjustment_count, 0);
        assert_eq!(stats.current_lod_bias, 0.0);
        assert_eq!(stats.current_shadow_resolution, 1024);
    }

    #[test]
    fn test_lod_bias_clamping() {
        let config = AdaptiveQualityConfig {
            min_lod_bias: -0.5,
            max_lod_bias: 0.5,
            frame_history_size: 10,
            ..Default::default()
        };

        let mut system = AdaptiveQualitySystem::new(config);

        // Try to push LOD bias beyond limits
        for _ in 0..100 {
            system.update(0.001); // Very fast frames
        }

        assert!(system.lod_bias() <= 0.5);

        for _ in 0..100 {
            system.update(0.100); // Very slow frames
        }

        assert!(system.lod_bias() >= -0.5);
    }

    #[test]
    fn test_average_frame_time() {
        let config = AdaptiveQualityConfig {
            frame_history_size: 5,
            ..Default::default()
        };

        let mut system = AdaptiveQualitySystem::new(config);

        // Add some frame times
        system.update(0.010);
        system.update(0.020);
        system.update(0.030);

        let avg = system.average_frame_time_ms();
        assert!((avg - 20.0).abs() < 1.0); // Should be around 20ms average
    }
}
