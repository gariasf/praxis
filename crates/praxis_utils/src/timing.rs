//! Simple timing utilities for frame-based updates.
//!
//! This module provides basic delta time tracking and optional frame rate limiting.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Global timing context for accessing frame timing information from anywhere.
static GLOBAL_TIMING: OnceLock<Mutex<GlobalTiming>> = OnceLock::new();

/// Global timing information updated each frame.
#[derive(Debug, Clone)]
struct GlobalTiming {
    /// Delta time since last frame
    delta_time: Duration,
    /// Delta time in seconds
    delta_secs: f32,
    /// Current frames per second
    fps: f64,
    /// Total elapsed time since start
    total_time: Duration,
    /// Current frame number
    frame_count: u64,
}

impl Default for GlobalTiming {
    fn default() -> Self {
        Self {
            delta_time: Duration::ZERO,
            delta_secs: 0.0,
            fps: 0.0,
            total_time: Duration::ZERO,
            frame_count: 0,
        }
    }
}

/// Gets the current frame's delta time in seconds.
///
/// # Example
///
/// ```rust,ignore
/// use praxis_utils::timing::delta_time;
///
/// // In any game system
/// fn update_player(player: &mut Player) {
///     let dt = delta_time();
///     player.position += player.velocity * dt;
/// }
/// ```
pub fn delta_time() -> f32 {
    GLOBAL_TIMING
        .get()
        .and_then(|timing| timing.lock().ok())
        .map(|timing| timing.delta_secs)
        .unwrap_or(0.0)
}

/// Gets the current frame's delta time as a Duration.
pub fn delta_duration() -> Duration {
    GLOBAL_TIMING
        .get()
        .and_then(|timing| timing.lock().ok())
        .map(|timing| timing.delta_time)
        .unwrap_or(Duration::ZERO)
}

/// Gets the current frames per second.
pub fn current_fps() -> f64 {
    GLOBAL_TIMING
        .get()
        .and_then(|timing| timing.lock().ok())
        .map(|timing| timing.fps)
        .unwrap_or(0.0)
}

/// Gets the total elapsed time since the timing system started.
pub fn total_time() -> Duration {
    GLOBAL_TIMING
        .get()
        .and_then(|timing| timing.lock().ok())
        .map(|timing| timing.total_time)
        .unwrap_or(Duration::ZERO)
}

/// Gets the current frame count.
pub fn frame_count() -> u64 {
    GLOBAL_TIMING
        .get()
        .and_then(|timing| timing.lock().ok())
        .map(|timing| timing.frame_count)
        .unwrap_or(0)
}

/// A simple frame timer that tracks delta time and optionally limits frame rate.
///
/// # Example
///
/// ```rust,ignore
/// use praxis_utils::timing::FrameTimer;
///
/// let mut timer = FrameTimer::new();
///
/// loop {
///     // Get delta time since last frame
///     let delta = timer.tick();
///
///     // Update game objects
///     update_game(delta.as_secs_f32());
///
///     // Render frame
///     render();
///
///     // Sleep if needed to maintain frame rate limit
///     timer.sleep_if_needed();
/// }
/// ```
#[derive(Debug, Clone)]
pub struct FrameTimer {
    /// The last frame's timestamp
    last_frame: Instant,

    /// Current frame's timestamp
    current_frame: Instant,

    /// Target frame duration (None = unlimited FPS)
    target_frame_duration: Option<Duration>,

    /// Last frame's delta time
    delta_time: Duration,

    /// Frame counter
    frame_count: u64,

    /// Timer for FPS calculation
    fps_timer: Instant,

    /// Frames counted for current FPS calculation
    fps_frame_count: u32,

    /// Current FPS value
    current_fps: f64,

    /// Start time of the timer
    start_time: Instant,

    /// Whether to update global timing
    update_global: bool,
}

impl FrameTimer {
    /// Creates a new frame timer with no frame rate limit.
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            last_frame: now,
            current_frame: now,
            target_frame_duration: None,
            delta_time: Duration::ZERO,
            frame_count: 0,
            fps_timer: now,
            fps_frame_count: 0,
            current_fps: 0.0,
            start_time: now,
            update_global: false,
        }
    }

    /// Creates a new frame timer that updates the global timing context.
    ///
    /// Only one timer should be created with this method (typically in the main game loop).
    pub fn new_with_global() -> Self {
        // Initialize global timing if not already done
        GLOBAL_TIMING.get_or_init(|| Mutex::new(GlobalTiming::default()));

        let mut timer = Self::new();
        timer.update_global = true;
        timer
    }

    /// Updates the timer and returns the time elapsed since the last frame.
    ///
    /// This should be called once at the beginning of each frame.
    pub fn tick(&mut self) -> Duration {
        self.last_frame = self.current_frame;
        self.current_frame = Instant::now();
        self.delta_time = self.current_frame - self.last_frame;

        // Clamp delta time to prevent huge jumps (e.g., after pause or debug break)
        const MAX_DELTA: Duration = Duration::from_millis(100); // 100ms = 10 FPS minimum
        if self.delta_time > MAX_DELTA {
            self.delta_time = MAX_DELTA;
        }

        self.frame_count += 1;
        self.fps_frame_count += 1;

        // Update global timing if enabled
        if self.update_global {
            if let Some(global) = GLOBAL_TIMING.get() {
                if let Ok(mut timing) = global.lock() {
                    timing.delta_time = self.delta_time;
                    timing.delta_secs = self.delta_time.as_secs_f32();
                    timing.fps = self.current_fps;
                    timing.total_time = self.current_frame - self.start_time;
                    timing.frame_count = self.frame_count;
                }
            }
        }

        // Update FPS every second
        let fps_elapsed = self.current_frame - self.fps_timer;
        if fps_elapsed >= Duration::from_secs(1) {
            self.current_fps = self.fps_frame_count as f64 / fps_elapsed.as_secs_f64();
            self.fps_frame_count = 0;
            self.fps_timer = self.current_frame;
        }

        self.delta_time
    }

    /// Returns the delta time of the last frame in seconds.
    pub fn delta(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }

    /// Returns the current frames per second.
    pub fn fps(&self) -> f64 {
        self.current_fps
    }

    /// Sets the target FPS limit.
    ///
    /// Pass `None` to remove the limit and run at maximum speed.
    pub fn set_target_fps(&mut self, target_fps: Option<f64>) {
        self.target_frame_duration = target_fps.map(|fps| Duration::from_secs_f64(1.0 / fps));
    }

    /// Sleeps if necessary to maintain the target frame rate.
    ///
    /// This should be called at the end of each frame if you want to
    /// enforce the FPS limit. Returns the actual time slept.
    pub fn sleep_if_needed(&self) -> Duration {
        if let Some(target_duration) = self.target_frame_duration {
            let frame_time = self.current_frame.elapsed();
            if frame_time < target_duration {
                let sleep_duration = target_duration - frame_time;
                std::thread::sleep(sleep_duration);
                return sleep_duration;
            }
        }
        Duration::ZERO
    }

    /// Returns timing statistics as a formatted string.
    pub fn stats(&self) -> String {
        format!(
            "FPS: {:.1}, Frame time: {:.2}ms, Total frames: {}",
            self.current_fps,
            self.delta_time.as_secs_f64() * 1000.0,
            self.frame_count
        )
    }
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_frame_timer_basic() {
        let mut timer = FrameTimer::new();

        // First tick
        thread::sleep(Duration::from_millis(16)); // Simulate ~60 FPS
        let delta = timer.tick();

        assert!(delta >= Duration::from_millis(16));
        assert!(timer.delta() >= 0.016);
    }

    #[test]
    fn test_delta_clamping() {
        let mut timer = FrameTimer::new();

        // Simulate a very long frame (e.g., after a pause)
        thread::sleep(Duration::from_millis(200));
        let delta = timer.tick();

        // Delta should be clamped to 100ms
        assert_eq!(delta, Duration::from_millis(100));
    }
}
