//! Automatic and manual exposure calculation for HDR rendering.
//!
//! This module provides exposure calculation using average luminance
//! to automatically adapt to scene brightness.

use praxis_utils::{debug, info};

/// Exposure calculation mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExposureMode {
    /// Manual exposure with fixed value.
    Manual { exposure: f32 },
    /// Automatic exposure based on scene luminance.
    Automatic {
        /// Adaptation speed (higher = faster adaptation).
        speed: f32,
    },
}

impl Default for ExposureMode {
    fn default() -> Self {
        Self::Automatic { speed: 2.0 }
    }
}

/// Exposure calculator for HDR rendering.
///
/// Calculates exposure values based on scene luminance to automatically
/// adapt to varying brightness levels in the scene.
///
/// # Automatic Exposure
///
/// Uses the formula: `exposure = key_value / average_luminance`
///
/// Where:
/// - `key_value` is the target middle gray value (typically 0.18)
/// - `average_luminance` is calculated from the scene
///
/// # Usage
///
/// ```rust,no_run
/// use praxis_graphics::hdr::{ExposureCalculator, ExposureMode};
///
/// let mut calculator = ExposureCalculator::new(ExposureMode::Automatic { speed: 2.0 });
///
/// // In render loop:
/// let delta_time = 0.016; // 60 FPS
/// let average_luminance = 0.5; // From scene analysis
/// let exposure = calculator.calculate(average_luminance, delta_time);
/// ```
pub struct ExposureCalculator {
    mode: ExposureMode,
    current_exposure: f32,
    target_key_value: f32,
    min_exposure: f32,
    max_exposure: f32,
}

impl ExposureCalculator {
    /// Creates a new exposure calculator.
    ///
    /// # Arguments
    ///
    /// * `mode` - Exposure calculation mode (manual or automatic)
    pub fn new(mode: ExposureMode) -> Self {
        info!("Creating exposure calculator with mode: {:?}", mode);

        let current_exposure = match mode {
            ExposureMode::Manual { exposure } => exposure,
            ExposureMode::Automatic { .. } => 1.0,
        };

        Self {
            mode,
            current_exposure,
            target_key_value: 0.18,
            min_exposure: 0.1,
            max_exposure: 10.0,
        }
    }

    /// Calculates exposure for the current frame.
    ///
    /// # Arguments
    ///
    /// * `average_luminance` - Average scene luminance (from luminance calculation)
    /// * `delta_time` - Time since last frame in seconds
    ///
    /// # Returns
    ///
    /// The exposure value to use for tone mapping.
    pub fn calculate(&mut self, average_luminance: f32, delta_time: f32) -> f32 {
        match self.mode {
            ExposureMode::Manual { exposure } => {
                self.current_exposure = exposure;
                exposure
            }
            ExposureMode::Automatic { speed } => {
                // Calculate target exposure based on scene luminance
                let target_exposure = if average_luminance > 0.001 {
                    self.target_key_value / average_luminance
                } else {
                    1.0
                };

                // Clamp target exposure to reasonable range
                let target_exposure = target_exposure.clamp(self.min_exposure, self.max_exposure);

                // Smoothly interpolate towards target exposure
                let adaptation_rate = 1.0 - (-speed * delta_time).exp();
                self.current_exposure +=
                    (target_exposure - self.current_exposure) * adaptation_rate;

                debug!(
                    "Exposure: current={:.3}, target={:.3}, luminance={:.3}",
                    self.current_exposure, target_exposure, average_luminance
                );

                self.current_exposure
            }
        }
    }

    /// Returns the current exposure value without updating.
    pub fn current_exposure(&self) -> f32 {
        self.current_exposure
    }

    /// Sets the exposure mode.
    pub fn set_mode(&mut self, mode: ExposureMode) {
        self.mode = mode;
        if let ExposureMode::Manual { exposure } = mode {
            self.current_exposure = exposure;
        }
    }

    /// Returns the current exposure mode.
    pub fn mode(&self) -> ExposureMode {
        self.mode
    }

    /// Sets the target key value for automatic exposure.
    ///
    /// The key value represents the target middle gray value.
    /// Standard values:
    /// - 0.18: Photographic middle gray (default)
    /// - 0.12-0.15: Darker scenes
    /// - 0.20-0.25: Brighter scenes
    pub fn set_key_value(&mut self, key_value: f32) {
        self.target_key_value = key_value.clamp(0.01, 1.0);
    }

    /// Sets the minimum exposure limit.
    pub fn set_min_exposure(&mut self, min: f32) {
        self.min_exposure = min.max(0.01);
    }

    /// Sets the maximum exposure limit.
    pub fn set_max_exposure(&mut self, max: f32) {
        self.max_exposure = max;
    }

    /// Resets the current exposure to the default value.
    pub fn reset(&mut self) {
        self.current_exposure = match self.mode {
            ExposureMode::Manual { exposure } => exposure,
            ExposureMode::Automatic { .. } => 1.0,
        };
    }
}

impl Default for ExposureCalculator {
    fn default() -> Self {
        Self::new(ExposureMode::default())
    }
}

/// Simple average luminance calculation from HDR color.
///
/// Uses the standard formula: Y = 0.2126*R + 0.7152*G + 0.0722*B
pub fn calculate_luminance(rgb: [f32; 3]) -> f32 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}
