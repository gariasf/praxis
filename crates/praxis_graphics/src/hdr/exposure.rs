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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exposure_calculator_manual_mode() {
        let mut calculator = ExposureCalculator::new(ExposureMode::Manual { exposure: 2.0 });
        
        assert_eq!(calculator.current_exposure(), 2.0);
        
        // Manual exposure should not change with luminance or time
        let exposure = calculator.calculate(0.5, 0.016);
        assert_eq!(exposure, 2.0);
        
        let exposure2 = calculator.calculate(1.0, 0.016);
        assert_eq!(exposure2, 2.0);
    }

    #[test]
    fn test_exposure_calculator_manual_mode_change() {
        let mut calculator = ExposureCalculator::new(ExposureMode::Manual { exposure: 1.0 });
        
        // Change to different manual exposure
        calculator.set_mode(ExposureMode::Manual { exposure: 3.0 });
        assert_eq!(calculator.current_exposure(), 3.0);
    }

    #[test]
    fn test_exposure_calculator_automatic_mode_basic() {
        let mut calculator = ExposureCalculator::new(ExposureMode::Automatic { speed: 10.0 });
        
        // Initial exposure should be 1.0
        assert_eq!(calculator.current_exposure(), 1.0);
        
        // With bright scene (high luminance), exposure should decrease
        // key_value / luminance = 0.18 / 1.0 = 0.18
        calculator.calculate(1.0, 0.1);
        
        // Should have adapted towards 0.18
        assert!(calculator.current_exposure() < 1.0);
    }

    #[test]
    fn test_exposure_calculator_automatic_adaptation_speed() {
        let mut fast = ExposureCalculator::new(ExposureMode::Automatic { speed: 10.0 });
        let mut slow = ExposureCalculator::new(ExposureMode::Automatic { speed: 1.0 });
        
        // Same luminance and time
        fast.calculate(0.5, 0.1);
        slow.calculate(0.5, 0.1);
        
        // Fast should have adapted more
        let fast_delta = (fast.current_exposure() - 1.0).abs();
        let slow_delta = (slow.current_exposure() - 1.0).abs();
        
        assert!(fast_delta > slow_delta, 
            "Fast adaptation ({}) should change more than slow ({})", 
            fast_delta, slow_delta);
    }

    #[test]
    fn test_exposure_calculator_target_exposure() {
        let mut calculator = ExposureCalculator::new(ExposureMode::Automatic { speed: 100.0 });
        
        // Very fast adaptation to see target
        for _ in 0..100 {
            calculator.calculate(0.5, 0.1);
        }
        
        // Target exposure is key_value / luminance = 0.18 / 0.5 = 0.36
        let final_exposure = calculator.current_exposure();
        assert!((final_exposure - 0.36).abs() < 0.05, 
            "Final exposure {} should be close to 0.36", final_exposure);
    }

    #[test]
    fn test_exposure_calculator_clamping_min() {
        let mut calculator = ExposureCalculator::new(ExposureMode::Automatic { speed: 100.0 });
        
        // Very bright scene should clamp to min exposure
        for _ in 0..100 {
            calculator.calculate(10.0, 0.1);
        }
        
        let final_exposure = calculator.current_exposure();
        assert!(final_exposure >= 0.1, "Exposure should not go below min (0.1)");
        assert!((final_exposure - 0.1).abs() < 0.01, "Should clamp to min exposure");
    }

    #[test]
    fn test_exposure_calculator_clamping_max() {
        let mut calculator = ExposureCalculator::new(ExposureMode::Automatic { speed: 100.0 });
        
        // Very dark scene should clamp to max exposure
        for _ in 0..100 {
            calculator.calculate(0.01, 0.1);
        }
        
        let final_exposure = calculator.current_exposure();
        assert!(final_exposure <= 10.0, "Exposure should not exceed max (10.0)");
        assert!((final_exposure - 10.0).abs() < 0.1, "Should clamp to max exposure");
    }

    #[test]
    fn test_exposure_calculator_zero_luminance() {
        let mut calculator = ExposureCalculator::new(ExposureMode::Automatic { speed: 2.0 });
        
        // Zero luminance should default to 1.0 exposure
        let exposure = calculator.calculate(0.0, 0.016);
        assert_eq!(exposure, 1.0);
    }

    #[test]
    fn test_exposure_calculator_very_small_luminance() {
        let mut calculator = ExposureCalculator::new(ExposureMode::Automatic { speed: 100.0 });
        
        // Very small luminance (below threshold) should default to 1.0
        for _ in 0..10 {
            calculator.calculate(0.0001, 0.1);
        }
        
        let final_exposure = calculator.current_exposure();
        assert_eq!(final_exposure, 1.0, "Very small luminance should result in default exposure");
    }

    #[test]
    fn test_exposure_calculator_smooth_adaptation() {
        let mut calculator = ExposureCalculator::new(ExposureMode::Automatic { speed: 2.0 });
        
        let exposure1 = calculator.calculate(0.5, 0.016);
        let exposure2 = calculator.calculate(0.5, 0.016);
        let exposure3 = calculator.calculate(0.5, 0.016);
        
        // Each step should move towards target
        assert!(exposure2 != exposure1, "Exposure should change");
        assert!(exposure3 != exposure2, "Exposure should continue changing");
        
        // Changes should get smaller as we approach target
        let delta1 = (exposure2 - exposure1).abs();
        let delta2 = (exposure3 - exposure2).abs();
        assert!(delta2 < delta1, "Adaptation should slow down near target");
    }

    #[test]
    fn test_exposure_calculator_key_value_adjustment() {
        let mut calculator = ExposureCalculator::new(ExposureMode::Automatic { speed: 100.0 });
        
        // Set custom key value for brighter scenes
        calculator.set_key_value(0.25);
        
        // Adapt to luminance
        for _ in 0..100 {
            calculator.calculate(0.5, 0.1);
        }
        
        // Target exposure should use new key value: 0.25 / 0.5 = 0.5
        let final_exposure = calculator.current_exposure();
        assert!((final_exposure - 0.5).abs() < 0.05, 
            "Exposure should use custom key value");
    }

    #[test]
    fn test_exposure_calculator_key_value_clamping() {
        let mut calculator = ExposureCalculator::new(ExposureMode::Automatic { speed: 2.0 });
        
        // Key value should be clamped to [0.01, 1.0]
        calculator.set_key_value(2.0);
        // Should be clamped, but we can't directly check without exposing the field
        // Just verify it doesn't crash and still works
        calculator.calculate(0.5, 0.016);
        
        calculator.set_key_value(-1.0);
        calculator.calculate(0.5, 0.016);
    }

    #[test]
    fn test_exposure_calculator_min_max_adjustment() {
        let mut calculator = ExposureCalculator::new(ExposureMode::Automatic { speed: 100.0 });
        
        // Set custom min/max
        calculator.set_min_exposure(0.5);
        calculator.set_max_exposure(5.0);
        
        // Very bright scene
        for _ in 0..100 {
            calculator.calculate(10.0, 0.1);
        }
        let bright_exposure = calculator.current_exposure();
        assert!(bright_exposure >= 0.5, "Should respect custom min");
        
        // Very dark scene
        for _ in 0..100 {
            calculator.calculate(0.001, 0.1);
        }
        let dark_exposure = calculator.current_exposure();
        assert!(dark_exposure <= 5.0, "Should respect custom max");
    }

    #[test]
    fn test_exposure_calculator_reset() {
        let mut calculator = ExposureCalculator::new(ExposureMode::Automatic { speed: 2.0 });
        
        // Adapt to some value
        calculator.calculate(0.5, 0.1);
        let adapted = calculator.current_exposure();
        assert_ne!(adapted, 1.0, "Should have adapted away from default");
        
        // Reset
        calculator.reset();
        assert_eq!(calculator.current_exposure(), 1.0, "Should reset to default");
    }

    #[test]
    fn test_exposure_calculator_mode_query() {
        let calculator = ExposureCalculator::new(ExposureMode::Manual { exposure: 2.5 });
        
        match calculator.mode() {
            ExposureMode::Manual { exposure } => {
                assert_eq!(exposure, 2.5);
            }
            _ => panic!("Mode should be Manual"),
        }
    }

    #[test]
    fn test_calculate_luminance_white() {
        let white = [1.0, 1.0, 1.0];
        let luminance = calculate_luminance(white);
        
        // For white (1,1,1), luminance should be 1.0
        assert!((luminance - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_luminance_black() {
        let black = [0.0, 0.0, 0.0];
        let luminance = calculate_luminance(black);
        
        assert_eq!(luminance, 0.0);
    }

    #[test]
    fn test_calculate_luminance_pure_red() {
        let red = [1.0, 0.0, 0.0];
        let luminance = calculate_luminance(red);
        
        // Red coefficient is 0.2126
        assert!((luminance - 0.2126).abs() < 0.001);
    }

    #[test]
    fn test_calculate_luminance_pure_green() {
        let green = [0.0, 1.0, 0.0];
        let luminance = calculate_luminance(green);
        
        // Green coefficient is 0.7152 (highest, as human eye is most sensitive to green)
        assert!((luminance - 0.7152).abs() < 0.001);
    }

    #[test]
    fn test_calculate_luminance_pure_blue() {
        let blue = [0.0, 0.0, 1.0];
        let luminance = calculate_luminance(blue);
        
        // Blue coefficient is 0.0722
        assert!((luminance - 0.0722).abs() < 0.001);
    }

    #[test]
    fn test_calculate_luminance_gray() {
        let gray = [0.5, 0.5, 0.5];
        let luminance = calculate_luminance(gray);
        
        // For gray (0.5,0.5,0.5), luminance should be 0.5
        assert!((luminance - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_calculate_luminance_hdr_values() {
        // HDR colors can exceed 1.0
        let bright_red = [2.0, 0.0, 0.0];
        let luminance = calculate_luminance(bright_red);
        
        assert!((luminance - 0.4252).abs() < 0.001); // 2.0 * 0.2126
        
        let very_bright = [5.0, 5.0, 5.0];
        let luminance2 = calculate_luminance(very_bright);
        assert!((luminance2 - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_luminance_weighted_correctly() {
        // Green should contribute most to luminance
        let mostly_green = [0.1, 0.9, 0.1];
        let mostly_red = [0.9, 0.1, 0.1];
        
        let green_lum = calculate_luminance(mostly_green);
        let red_lum = calculate_luminance(mostly_red);
        
        assert!(green_lum > red_lum, 
            "Green-dominant color should have higher luminance");
    }

    #[test]
    fn test_exposure_mode_default() {
        let mode = ExposureMode::default();
        
        match mode {
            ExposureMode::Automatic { speed } => {
                assert_eq!(speed, 2.0);
            }
            _ => panic!("Default should be Automatic"),
        }
    }

    #[test]
    fn test_exposure_calculator_default() {
        let calculator = ExposureCalculator::default();
        
        assert_eq!(calculator.current_exposure(), 1.0);
        
        match calculator.mode() {
            ExposureMode::Automatic { speed } => {
                assert_eq!(speed, 2.0);
            }
            _ => panic!("Default should be Automatic"),
        }
    }
}
