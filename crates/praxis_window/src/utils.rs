//! Utility functions for window management.
//!
//! This module provides helper functions for common windowing tasks like
//! querying monitor information, converting between coordinate systems, etc.

use winit::dpi::{LogicalSize, PhysicalSize};

/// Converts physical pixels to logical pixels using a DPI scale factor.
///
/// # Arguments
///
/// * `physical` - Physical size in pixels
/// * `scale_factor` - DPI scale factor (typically 1.0, 1.5, 2.0, etc.)
///
/// # Examples
///
/// ```rust,ignore
/// use praxis_window::utils::physical_to_logical;
/// use winit::dpi::PhysicalSize;
///
/// let physical = PhysicalSize::new(1920, 1080);
/// let logical = physical_to_logical(physical, 2.0); // Retina display
/// assert_eq!(logical.width, 960.0);
/// assert_eq!(logical.height, 540.0);
/// ```
pub fn physical_to_logical<T: Into<f64>>(
    physical: PhysicalSize<u32>,
    scale_factor: T,
) -> LogicalSize<f64> {
    let scale = scale_factor.into();
    LogicalSize::new(
        f64::from(physical.width) / scale,
        f64::from(physical.height) / scale,
    )
}

/// Converts logical pixels to physical pixels using a DPI scale factor.
///
/// # Arguments
///
/// * `logical` - Logical size (DPI-independent)
/// * `scale_factor` - DPI scale factor (typically 1.0, 1.5, 2.0, etc.)
///
/// # Examples
///
/// ```rust,ignore
/// use praxis_window::utils::logical_to_physical;
/// use winit::dpi::LogicalSize;
///
/// let logical = LogicalSize::new(960.0, 540.0);
/// let physical = logical_to_physical(logical, 2.0); // Retina display
/// assert_eq!(physical.width, 1920);
/// assert_eq!(physical.height, 1080);
/// ```
pub fn logical_to_physical<T: Into<f64>>(
    logical: LogicalSize<f64>,
    scale_factor: T,
) -> PhysicalSize<u32> {
    let scale = scale_factor.into();
    PhysicalSize::new(
        (logical.width * scale).round() as u32,
        (logical.height * scale).round() as u32,
    )
}

/// Calculates the aspect ratio (width / height) for a given size.
///
/// # Arguments
///
/// * `width` - Width in any unit
/// * `height` - Height in any unit
///
/// # Returns
///
/// The aspect ratio as a floating point number. Returns 0.0 if height is zero.
///
/// # Examples
///
/// ```rust,ignore
/// use praxis_window::utils::aspect_ratio;
///
/// assert_eq!(aspect_ratio(1920, 1080), 16.0 / 9.0);
/// assert_eq!(aspect_ratio(1280, 720), 16.0 / 9.0);
/// assert_eq!(aspect_ratio(1024, 768), 4.0 / 3.0);
/// ```
pub fn aspect_ratio(width: u32, height: u32) -> f32 {
    if height == 0 {
        0.0
    } else {
        width as f32 / height as f32
    }
}

/// Checks if a window size is valid (non-zero dimensions).
///
/// Windows report 0×0 size when minimized, which is invalid for most graphics
/// operations. Use this function to check before rendering or recreating resources.
///
/// # Arguments
///
/// * `width` - Window width
/// * `height` - Window height
///
/// # Returns
///
/// `true` if both dimensions are greater than zero, `false` otherwise.
///
/// # Examples
///
/// ```rust,ignore
/// use praxis_window::utils::is_valid_size;
///
/// assert!(is_valid_size(1920, 1080));
/// assert!(!is_valid_size(0, 0));
/// assert!(!is_valid_size(1920, 0));
/// ```
pub fn is_valid_size(width: u32, height: u32) -> bool {
    width > 0 && height > 0
}

/// Clamps a window size to reasonable bounds.
///
/// Prevents windows from being too small (unusable) or too large (system limits).
/// Useful for enforcing minimum/maximum window sizes.
///
/// # Arguments
///
/// * `width` - Requested width
/// * `height` - Requested height
/// * `min_width` - Minimum allowed width
/// * `min_height` - Minimum allowed height
/// * `max_width` - Maximum allowed width
/// * `max_height` - Maximum allowed height
///
/// # Returns
///
/// Clamped size within the specified bounds.
///
/// # Examples
///
/// ```rust,ignore
/// use praxis_window::utils::clamp_size;
///
/// let size = clamp_size(100, 100, 800, 600, 3840, 2160);
/// assert_eq!(size, (800, 600)); // Too small, clamped to minimum
///
/// let size = clamp_size(5000, 3000, 800, 600, 3840, 2160);
/// assert_eq!(size, (3840, 2160)); // Too large, clamped to maximum
/// ```
pub fn clamp_size(
    width: u32,
    height: u32,
    min_width: u32,
    min_height: u32,
    max_width: u32,
    max_height: u32,
) -> (u32, u32) {
    (
        width.clamp(min_width, max_width),
        height.clamp(min_height, max_height),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physical_to_logical() {
        let physical = PhysicalSize::new(1920, 1080);
        let logical = physical_to_logical(physical, 2.0);
        assert_eq!(logical.width, 960.0);
        assert_eq!(logical.height, 540.0);
    }

    #[test]
    fn test_logical_to_physical() {
        let logical = LogicalSize::new(960.0, 540.0);
        let physical = logical_to_physical(logical, 2.0);
        assert_eq!(physical.width, 1920);
        assert_eq!(physical.height, 1080);
    }

    #[test]
    fn test_aspect_ratio() {
        assert!((aspect_ratio(1920, 1080) - 16.0 / 9.0).abs() < 0.001);
        assert!((aspect_ratio(1280, 720) - 16.0 / 9.0).abs() < 0.001);
        assert!((aspect_ratio(1024, 768) - 4.0 / 3.0).abs() < 0.001);
        assert_eq!(aspect_ratio(100, 0), 0.0);
    }

    #[test]
    fn test_is_valid_size() {
        assert!(is_valid_size(1920, 1080));
        assert!(is_valid_size(1, 1));
        assert!(!is_valid_size(0, 0));
        assert!(!is_valid_size(1920, 0));
        assert!(!is_valid_size(0, 1080));
    }

    #[test]
    fn test_clamp_size() {
        assert_eq!(clamp_size(100, 100, 800, 600, 3840, 2160), (800, 600));
        assert_eq!(clamp_size(5000, 3000, 800, 600, 3840, 2160), (3840, 2160));
        assert_eq!(clamp_size(1920, 1080, 800, 600, 3840, 2160), (1920, 1080));
    }
}
