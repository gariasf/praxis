//! Window event types and resize strategies.
//!
//! This module defines types related to window event handling, particularly
//! resize debouncing strategies.

use std::time::{Duration, Instant};
use winit::dpi::PhysicalSize;

/// Strategy for handling window resize events.
///
/// Window resizing is complex because OS sends many events during drag operations.
/// Different strategies offer different trade-offs between responsiveness and performance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowResizeStrategy {
    /// No debouncing - process every resize immediately.
    ///
    /// **Pros**: Most responsive, immediate feedback
    /// **Cons**: Can cause hundreds of swapchain recreations during drag
    ///
    /// Use for: Applications where immediate resize response is critical
    Immediate,

    /// Debounce resize events with a delay.
    ///
    /// Waits for the specified duration of inactivity before processing resize.
    /// If another resize event arrives during the delay, the timer resets.
    ///
    /// **Pros**: Reduces recreations dramatically (hundreds → few)
    /// **Cons**: Small delay before resize takes effect
    ///
    /// Use for: Most applications (recommended default)
    Debounced(Duration),

    /// Process resize only when window drag ends.
    ///
    /// **Pros**: Minimal number of recreations (typically just one)
    /// **Cons**: No visual feedback during resize (window content frozen)
    ///
    /// Use for: Applications with very expensive resize operations
    OnDragEnd,
}

impl Default for WindowResizeStrategy {
    /// Default strategy is debounced with 16ms delay (approximately 1 frame at 60 FPS).
    fn default() -> Self {
        Self::Debounced(Duration::from_millis(16))
    }
}

impl WindowResizeStrategy {
    /// Creates a debounced strategy with the specified delay in milliseconds.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // 32ms delay (about 2 frames at 60 FPS)
    /// let strategy = WindowResizeStrategy::debounced_ms(32);
    /// ```
    pub fn debounced_ms(millis: u64) -> Self {
        Self::Debounced(Duration::from_millis(millis))
    }

    /// Creates a debounced strategy targeting a specific frame rate.
    ///
    /// The delay will be set to one frame duration at the target FPS.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Delay one frame at 60 FPS (16.67ms)
    /// let strategy = WindowResizeStrategy::debounced_for_fps(60.0);
    /// ```
    pub fn debounced_for_fps(fps: f32) -> Self {
        let frame_time = 1.0 / fps;
        let millis = (frame_time * 1000.0) as u64;
        Self::Debounced(Duration::from_millis(millis))
    }
}

/// Tracks a pending resize operation for debouncing.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PendingResize {
    /// The new size that was requested
    pub size: PhysicalSize<u32>,
    /// When the resize event was received
    pub timestamp: Instant,
}

impl PendingResize {
    /// Creates a new pending resize with current timestamp.
    pub fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            size,
            timestamp: Instant::now(),
        }
    }

    /// Returns true if enough time has elapsed based on the strategy.
    pub fn is_ready(&self, strategy: WindowResizeStrategy) -> bool {
        match strategy {
            WindowResizeStrategy::Immediate => true,
            WindowResizeStrategy::Debounced(duration) => self.timestamp.elapsed() >= duration,
            WindowResizeStrategy::OnDragEnd => false, // Handled separately
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize_strategy_default() {
        let strategy = WindowResizeStrategy::default();
        match strategy {
            WindowResizeStrategy::Debounced(duration) => {
                assert_eq!(duration, Duration::from_millis(16));
            }
            _ => panic!("Default should be Debounced"),
        }
    }

    #[test]
    fn test_debounced_ms() {
        let strategy = WindowResizeStrategy::debounced_ms(32);
        match strategy {
            WindowResizeStrategy::Debounced(duration) => {
                assert_eq!(duration, Duration::from_millis(32));
            }
            _ => panic!("Should be Debounced"),
        }
    }

    #[test]
    fn test_debounced_for_fps() {
        let strategy = WindowResizeStrategy::debounced_for_fps(60.0);
        match strategy {
            WindowResizeStrategy::Debounced(duration) => {
                assert_eq!(duration, Duration::from_millis(16));
            }
            _ => panic!("Should be Debounced"),
        }
    }

    #[test]
    fn test_pending_resize() {
        let size = PhysicalSize::new(800, 600);
        let pending = PendingResize::new(size);

        assert_eq!(pending.size.width, 800);
        assert_eq!(pending.size.height, 600);

        assert!(pending.is_ready(WindowResizeStrategy::Immediate));
        assert!(!pending.is_ready(WindowResizeStrategy::OnDragEnd));
    }
}
