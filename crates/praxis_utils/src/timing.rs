//! Frame timing utilities for game loops and performance monitoring.
//!
//! This module provides high-precision timing for frame-based updates, FPS tracking,
//! and optional frame rate limiting. It uses `std::time::Instant` for reliable,
//! monotonic time measurement that doesn't depend on system clock adjustments.
//!
//! # Core Concepts
//!
//! ## Delta Time
//!
//! Delta time (Δt) is the elapsed time since the last frame. It's used to make
//! game logic frame-rate independent:
//!
//! ```rust,ignore
//! use praxis_utils::timing::delta_time;
//!
//! fn update_physics(mut query: Query<(&mut Transform, &Velocity)>) {
//!     let dt = delta_time(); // Seconds since last frame
//!     
//!     for (mut transform, velocity) in query.iter_mut() {
//!         // Movement scales with time, not frame rate
//!         transform.translation += velocity.0 * dt;
//!     }
//! }
//! ```
//!
//! **Without delta time**: A character moving at 60 FPS would move twice as fast
//! as one rendering at 30 FPS, making gameplay inconsistent.
//!
//! **With delta time**: Movement is time-based, so the character covers the same
//! distance per second regardless of frame rate.
//!
//! ## Frame Rate Independence
//!
//! All game logic should use delta time for time-based calculations:
//!
//! ```rust,ignore
//! // Bad: Frame-rate dependent (moves different distances at different FPS)
//! position.x += velocity.x;
//!
//! // Good: Frame-rate independent (moves consistent distance per second)
//! position.x += velocity.x * delta_time();
//! ```
//!
//! # Global Timing Access
//!
//! The module provides global accessors for timing information, updated
//! automatically by the main `FrameTimer`:
//!
//! - [`delta_time()`] - Delta time in seconds (f32)
//! - [`delta_duration()`] - Delta time as Duration
//! - [`current_fps()`] - Frames per second
//! - [`total_time()`] - Time since application start
//! - [`frame_count()`] - Total frames rendered
//!
//! These are safe to call from any system or thread, though they only update
//! once per frame from the main game loop.
//!
//! # Main Loop Integration
//!
//! ## Basic Usage
//!
//! ```rust,ignore
//! use praxis_utils::timing::FrameTimer;
//!
//! fn main() -> Result<()> {
//!     let mut timer = FrameTimer::new_with_global();
//!     
//!     loop {
//!         // Update timing (must be first each frame)
//!         timer.tick();
//!         
//!         // Now delta_time() returns the correct value
//!         update_systems();
//!         render();
//!         
//!         // Optional: maintain target frame rate
//!         timer.sleep_if_needed();
//!     }
//! }
//! ```
//!
//! ## With FPS Limiting
//!
//! ```rust,ignore
//! let mut timer = FrameTimer::new_with_global();
//! timer.set_target_fps(Some(60.0)); // Cap at 60 FPS
//!
//! loop {
//!     timer.tick();
//!     update_systems();
//!     render();
//!     
//!     // Sleep to maintain 60 FPS (if frame finished early)
//!     timer.sleep_if_needed();
//! }
//! ```
//!
//! ## Multiple Timers
//!
//! For specialized timing (e.g., animation playback, cooldowns), create
//! independent timers:
//!
//! ```rust,ignore
//! // Main loop timer (updates global timing)
//! let mut main_timer = FrameTimer::new_with_global();
//!
//! // Independent timer for profiling
//! let mut profile_timer = FrameTimer::new();
//! ```
//!
//! # Delta Time Clamping
//!
//! To prevent instability from huge time jumps (e.g., debugger breakpoints,
//! OS sleep, window drag), delta time is automatically clamped to a maximum
//! of 100ms (0.1 seconds, equivalent to 10 FPS).
//!
//! ## Why Clamping Matters
//!
//! **Without clamping**:
//! ```text
//! 1. User hits breakpoint for 5 seconds
//! 2. Next frame: delta_time = 5.0 seconds
//! 3. Physics: object_velocity = 10 m/s * 5.0s = 50 meters (teleportation!)
//! 4. Collisions missed, objects fall through floor
//! ```
//!
//! **With clamping** (100ms max):
//! ```text
//! 1. User hits breakpoint for 5 seconds
//! 2. Next frame: delta_time = 0.1 seconds (clamped)
//! 3. Physics: object_velocity = 10 m/s * 0.1s = 1 meter (reasonable)
//! 4. Simulation remains stable
//! ```
//!
//! ## Adjust Clamp for Your Game
//!
//! The 100ms default works for most games, but you might adjust it:
//!
//! - **Fast-paced games** (fighting, racing): Lower clamp (50ms) for tighter control
//! - **Slow-paced games** (strategy, puzzle): Higher clamp (200ms) for flexibility
//! - **Physics-heavy games**: Lower clamp to prevent instability
//!
//! Currently, the clamp is hardcoded. Future versions may make it configurable.
//!
//! # Performance Statistics
//!
//! ```rust,ignore
//! use praxis_utils::timing::{FrameTimer, frame_count};
//! use praxis_utils::info;
//!
//! let mut timer = FrameTimer::new_with_global();
//!
//! loop {
//!     timer.tick();
//!     
//!     // Update game...
//!     
//!     // Log stats every 60 frames
//!     if frame_count() % 60 == 0 {
//!         info!("{}", timer.stats());
//!         // Output: "FPS: 59.8, Frame time: 16.72ms, Total frames: 3600"
//!     }
//! }
//! ```
//!
//! # Common Patterns
//!
//! ## Fixed Timestep Physics
//!
//! Physics simulations often need fixed timesteps for stability:
//!
//! ```rust,ignore
//! const PHYSICS_DT: f32 = 1.0 / 60.0; // 60 Hz physics
//! let mut accumulator = 0.0;
//!
//! loop {
//!     timer.tick();
//!     accumulator += delta_time();
//!     
//!     // Run physics at fixed 60 Hz
//!     while accumulator >= PHYSICS_DT {
//!         update_physics(PHYSICS_DT);
//!         accumulator -= PHYSICS_DT;
//!     }
//!     
//!     // Render at variable frame rate
//!     render();
//! }
//! ```
//!
//! ## Interpolation for Smooth Rendering
//!
//! When physics runs at a fixed rate but rendering is variable:
//!
//! ```rust,ignore
//! let alpha = accumulator / PHYSICS_DT;
//! let render_position = previous_position.lerp(current_position, alpha);
//! ```
//!
//! This prevents visual stuttering when render FPS differs from physics FPS.
//!
//! ## Cooldowns and Timers
//!
//! ```rust,ignore
//! struct Weapon {
//!     cooldown_remaining: f32,
//! }
//!
//! fn update_weapon(weapon: &mut Weapon) {
//!     weapon.cooldown_remaining -= delta_time();
//!     
//!     if weapon.cooldown_remaining <= 0.0 {
//!         // Ready to fire
//!     }
//! }
//! ```
//!
//! # Thread Safety
//!
//! Global timing functions use `OnceLock` and `Mutex` for safe concurrent access:
//! - **Reads** (`delta_time`, `current_fps`, etc.) are fast (mutex lock + copy)
//! - **Writes** (`timer.tick()`) happen once per frame from the main thread
//!
//! This design prioritizes simplicity over raw performance. If profiling shows
//! timing as a bottleneck (unlikely), consider passing delta time explicitly
//! instead of using global accessors.
//!
//! # See Also
//!
//! - [Fix Your Timestep](https://gafferongames.com/post/fix_your_timestep/) - Classic article
//! - [`std::time::Instant`] - Monotonic clock documentation
//! - `praxis_physics` crate - Fixed timestep physics implementation

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Global timing context for accessing frame timing information from anywhere.
///
/// This is initialized by [`FrameTimer::new_with_global()`] and updated each
/// frame by calling [`FrameTimer::tick()`]. It provides thread-safe access to
/// timing information from any system or thread in the application.
static GLOBAL_TIMING: OnceLock<Mutex<GlobalTiming>> = OnceLock::new();

/// Global timing information updated each frame.
///
/// This struct stores timing data in a format optimized for quick access.
/// Delta time is stored both as `Duration` (precise) and `f32` (fast) for
/// convenience.
#[derive(Debug, Clone)]
struct GlobalTiming {
    /// Delta time since last frame (precise)
    delta_time: Duration,
    /// Delta time in seconds (convenient for calculations)
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
/// Delta time is the elapsed time since the last frame, used to make game logic
/// frame-rate independent. This is the most commonly used timing function.
///
/// # Returns
///
/// Delta time in seconds as `f32`. Returns `0.0` if global timing is not
/// initialized (e.g., before the first frame).
///
/// # Thread Safety
///
/// Safe to call from any thread. Uses a mutex internally, but contention is
/// minimal as updates only happen once per frame.
///
/// # Examples
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
///
/// # Performance
///
/// Very fast: mutex lock + copy of f32. If profiling shows this as a bottleneck
/// (unlikely), consider passing delta time as a parameter instead.
pub fn delta_time() -> f32 {
    GLOBAL_TIMING
        .get()
        .and_then(|timing| timing.lock().ok())
        .map_or(0.0, |timing| timing.delta_secs)
}

/// Gets the current frame's delta time as a `Duration`.
///
/// Similar to [`delta_time()`] but returns the precise `Duration` type instead
/// of a `f32`. Use this when you need precise timing or want to avoid floating
/// point errors.
///
/// # Returns
///
/// Delta time as `Duration`. Returns `Duration::ZERO` if global timing is not
/// initialized.
///
/// # Examples
///
/// ```rust,ignore
/// use praxis_utils::timing::delta_duration;
/// use std::time::Duration;
///
/// fn check_timeout(last_update: Duration) -> bool {
///     let elapsed = last_update + delta_duration();
///     elapsed > Duration::from_secs(5)
/// }
/// ```
pub fn delta_duration() -> Duration {
    GLOBAL_TIMING
        .get()
        .and_then(|timing| timing.lock().ok())
        .map_or(Duration::ZERO, |timing| timing.delta_time)
}

/// Gets the current frames per second.
///
/// FPS is calculated as a rolling average over the last second. This provides
/// a stable reading that doesn't fluctuate wildly frame-to-frame.
///
/// # Returns
///
/// Current FPS as `f64`. Returns `0.0` if global timing is not initialized or
/// if less than one second has elapsed since start.
///
/// # Examples
///
/// ```rust,ignore
/// use praxis_utils::timing::current_fps;
/// use praxis_utils::warn;
///
/// fn check_performance() {
///     if current_fps() < 30.0 {
///         warn!("Low frame rate detected: {:.1} FPS", current_fps());
///     }
/// }
/// ```
pub fn current_fps() -> f64 {
    GLOBAL_TIMING
        .get()
        .and_then(|timing| timing.lock().ok())
        .map_or(0.0, |timing| timing.fps)
}

/// Gets the total elapsed time since the timing system started.
///
/// This is the wall-clock time since [`FrameTimer::new_with_global()`] was
/// called, useful for game-wide timers and session tracking.
///
/// # Returns
///
/// Total elapsed time as `Duration`. Returns `Duration::ZERO` if global timing
/// is not initialized.
///
/// # Examples
///
/// ```rust,ignore
/// use praxis_utils::timing::total_time;
///
/// fn show_playtime() -> String {
///     let time = total_time();
///     let secs = time.as_secs();
///     format!("Playtime: {}:{:02}:{:02}", secs / 3600, (secs / 60) % 60, secs % 60)
/// }
/// ```
pub fn total_time() -> Duration {
    GLOBAL_TIMING
        .get()
        .and_then(|timing| timing.lock().ok())
        .map_or(Duration::ZERO, |timing| timing.total_time)
}

/// Gets the current frame count.
///
/// This is the total number of frames rendered since the timing system started.
/// Useful for periodic updates (e.g., every 60 frames) and debugging.
///
/// # Returns
///
/// Total frame count as `u64`. Returns `0` if global timing is not initialized.
///
/// # Examples
///
/// ```rust,ignore
/// use praxis_utils::timing::frame_count;
/// use praxis_utils::info;
///
/// fn periodic_log() {
///     if frame_count() % 60 == 0 {
///         info!("Still running... frame {}", frame_count());
///     }
/// }
/// ```
pub fn frame_count() -> u64 {
    GLOBAL_TIMING
        .get()
        .and_then(|timing| timing.lock().ok())
        .map_or(0, |timing| timing.frame_count)
}

/// A frame timer that tracks delta time and optionally limits frame rate.
///
/// This is the primary timing utility for game loops. Create one with
/// [`new_with_global()`](Self::new_with_global) in your main loop, call
/// [`tick()`](Self::tick) each frame, and use the global functions
/// ([`delta_time()`], [`current_fps()`], etc.) throughout your systems.
///
/// # Frame Rate Limiting
///
/// Optional FPS limiting via [`set_target_fps()`](Self::set_target_fps) and
/// [`sleep_if_needed()`](Self::sleep_if_needed). The timer calculates how much
/// time remains in the target frame duration and sleeps if the frame finished
/// early.
///
/// **Note**: This uses `std::thread::sleep()` which is not frame-perfect due to
/// OS scheduler granularity. For precise frame timing, consider `VSync` or a
/// proper frame pacer.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust,ignore
/// use praxis_utils::timing::FrameTimer;
///
/// let mut timer = FrameTimer::new();
///
/// loop {
///     let delta = timer.tick();
///     update_game(delta.as_secs_f32());
///     render();
/// }
/// ```
///
/// ## With Global Timing
///
/// ```rust,ignore
/// use praxis_utils::timing::{FrameTimer, delta_time};
///
/// let mut timer = FrameTimer::new_with_global();
///
/// loop {
///     timer.tick();
///     
///     // Systems can now use global accessors
///     update_physics(delta_time());
///     
///     render();
/// }
/// ```
///
/// ## With FPS Limiting
///
/// ```rust,ignore
/// let mut timer = FrameTimer::new_with_global();
/// timer.set_target_fps(Some(60.0));
///
/// loop {
///     timer.tick();
///     update_game();
///     render();
///     timer.sleep_if_needed(); // Sleep to maintain 60 FPS
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
    ///
    /// This timer does **not** update global timing. For most applications,
    /// prefer [`new_with_global()`](Self::new_with_global).
    ///
    /// # Use Cases
    ///
    /// - Specialized timing (e.g., profiling specific subsystems)
    /// - Multiple independent timers
    /// - Testing and benchmarking
    ///
    /// # Examples
    ///
    /// ```rust
    /// use praxis_utils::timing::FrameTimer;
    ///
    /// let mut timer = FrameTimer::new();
    /// let delta = timer.tick();
    /// assert!(delta.as_secs_f32() >= 0.0);
    /// ```
    #[must_use]
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
    /// This should be used for the main game loop timer. Only create one of
    /// these per application - creating multiple will cause global timing to
    /// fluctuate unpredictably.
    ///
    /// Calling this function initializes the global timing system if it hasn't
    /// been initialized yet.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use praxis_utils::timing::FrameTimer;
    ///
    /// fn main() {
    ///     let mut timer = FrameTimer::new_with_global();
    ///     
    ///     loop {
    ///         timer.tick(); // Updates global timing
    ///         // Now all systems can use delta_time(), current_fps(), etc.
    ///         update_systems();
    ///         render();
    ///     }
    /// }
    /// ```
    pub fn new_with_global() -> Self {
        // Initialize global timing if not already done
        GLOBAL_TIMING.get_or_init(|| Mutex::new(GlobalTiming::default()));

        let mut timer = Self::new();
        timer.update_global = true;
        timer
    }

    /// Updates the timer and returns the time elapsed since the last frame.
    ///
    /// **Call this once at the beginning of each frame**, before any game logic
    /// or rendering. This updates the timer's internal state and, if created
    /// with [`new_with_global()`](Self::new_with_global), updates the global
    /// timing context.
    ///
    /// # Delta Time Clamping
    ///
    /// Delta time is automatically clamped to a maximum of 100ms (0.1 seconds)
    /// to prevent instability from huge time jumps (debugger breakpoints, OS
    /// sleep, window drag). This prevents physics explosions and other issues.
    ///
    /// # Returns
    ///
    /// The clamped delta time since the last frame as a `Duration`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut timer = FrameTimer::new_with_global();
    ///
    /// loop {
    ///     let delta = timer.tick();
    ///     update_game(delta.as_secs_f32());
    ///     render();
    /// }
    /// ```
    pub fn tick(&mut self) -> Duration {
        // Clamp delta time to prevent huge jumps (e.g., after pause or debug break)
        // 100ms = 10 FPS minimum - prevents physics explosions and other instabilities
        const MAX_DELTA: Duration = Duration::from_millis(100);
        
        self.last_frame = self.current_frame;
        self.current_frame = Instant::now();
        self.delta_time = self.current_frame - self.last_frame;

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
            self.current_fps = f64::from(self.fps_frame_count) / fps_elapsed.as_secs_f64();
            self.fps_frame_count = 0;
            self.fps_timer = self.current_frame;
        }

        self.delta_time
    }

    /// Returns the delta time of the last frame in seconds.
    ///
    /// This is a convenience wrapper around the internal delta time. Prefer
    /// using the global [`delta_time()`] function in most cases.
    ///
    /// # Returns
    ///
    /// Delta time in seconds as `f32`.
    #[must_use]
    pub const fn delta(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }

    /// Returns the current frames per second.
    ///
    /// FPS is calculated as a rolling average over the last second.
    ///
    /// # Returns
    ///
    /// Current FPS as `f64`. May be `0.0` if less than one second has elapsed.
    #[must_use]
    pub const fn fps(&self) -> f64 {
        self.current_fps
    }

    /// Returns the total frame count.
    ///
    /// # Returns
    ///
    /// Total number of frames since the timer was created.
    #[must_use]
    pub const fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Returns the total elapsed time since the timer was created.
    ///
    /// # Returns
    ///
    /// Total elapsed time as `Duration`.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.current_frame - self.start_time
    }

    /// Sets the target FPS limit.
    ///
    /// When set, [`sleep_if_needed()`](Self::sleep_if_needed) will sleep to
    /// maintain the target frame rate. Pass `None` to remove the limit and run
    /// at maximum speed.
    ///
    /// # Arguments
    ///
    /// * `target_fps` - Target frames per second (e.g., `60.0`), or `None` for unlimited
    ///
    /// # Examples
    ///
    /// ```rust
    /// use praxis_utils::timing::FrameTimer;
    ///
    /// let mut timer = FrameTimer::new();
    /// timer.set_target_fps(Some(60.0)); // Cap at 60 FPS
    /// ```
    pub fn set_target_fps(&mut self, target_fps: Option<f64>) {
        self.target_frame_duration = target_fps.map(|fps| Duration::from_secs_f64(1.0 / fps));
    }

    /// Sleeps if necessary to maintain the target frame rate.
    ///
    /// This should be called at the end of each frame if you want to enforce
    /// the FPS limit set by [`set_target_fps()`](Self::set_target_fps).
    ///
    /// # How It Works
    ///
    /// 1. Measures how long the current frame has taken so far
    /// 2. If less than the target frame duration, sleeps for the difference
    /// 3. If already at or over the target duration, returns immediately
    ///
    /// # Returns
    ///
    /// The actual time slept as a `Duration`. Returns `Duration::ZERO` if no
    /// target FPS is set or if the frame already exceeded the target duration.
    ///
    /// # Caveats
    ///
    /// - Uses `std::thread::sleep()` which is not frame-perfect
    /// - OS scheduler granularity may cause oversleeping (~1-2ms typical)
    /// - For precise frame timing, prefer `VSync` or a proper frame pacer
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut timer = FrameTimer::new();
    /// timer.set_target_fps(Some(60.0));
    ///
    /// loop {
    ///     timer.tick();
    ///     update_game();
    ///     render();
    ///     timer.sleep_if_needed(); // Sleep to maintain 60 FPS
    /// }
    /// ```
    #[must_use]
    pub fn sleep_if_needed(&self) -> Duration {
        if let Some(target_duration) = self.target_frame_duration {
            let frame_time = self.current_frame.elapsed();
            if frame_time < target_duration {
                if let Some(sleep_duration) = target_duration.checked_sub(frame_time) {
                    std::thread::sleep(sleep_duration);
                    return sleep_duration;
                }
            }
        }
        Duration::ZERO
    }

    /// Returns timing statistics as a formatted string.
    ///
    /// Useful for debug output and performance monitoring.
    ///
    /// # Returns
    ///
    /// A formatted string with FPS, frame time, and total frame count.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use praxis_utils::timing::FrameTimer;
    /// use praxis_utils::info;
    ///
    /// let mut timer = FrameTimer::new();
    ///
    /// loop {
    ///     timer.tick();
    ///     
    ///     if timer.frame_count() % 60 == 0 {
    ///         info!("{}", timer.stats());
    ///         // Output: "FPS: 59.8, Frame time: 16.72ms, Total frames: 3600"
    ///     }
    /// }
    /// ```
    #[must_use]
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

        // First tick should have minimal delta (just call overhead)
        let delta = timer.tick();
        assert!(delta.as_secs_f32() >= 0.0);
        assert_eq!(timer.frame_count(), 1);

        // Second tick after sleep should have measurable delta
        thread::sleep(Duration::from_millis(16));
        let delta = timer.tick();
        assert!(delta >= Duration::from_millis(16));
        assert!(timer.delta() >= 0.016);
        assert_eq!(timer.frame_count(), 2);
    }

    #[test]
    fn test_delta_clamping() {
        let mut timer = FrameTimer::new();

        // First tick
        timer.tick();

        // Simulate a very long frame (e.g., after a pause or debugger break)
        thread::sleep(Duration::from_millis(200));
        let delta = timer.tick();

        // Delta should be clamped to 100ms
        assert_eq!(delta, Duration::from_millis(100));
        assert_eq!(timer.delta(), 0.1);
    }

    #[test]
    fn test_fps_limiting() {
        let mut timer = FrameTimer::new();
        timer.set_target_fps(Some(60.0));

        // First frame
        timer.tick();

        // Frame takes almost no time, should sleep
        let slept = timer.sleep_if_needed();
        assert!(slept > Duration::ZERO);
    }

    #[test]
    fn test_global_timing_initialization() {
        let _timer = FrameTimer::new_with_global();

        // Global timing should now be initialized
        assert!(GLOBAL_TIMING.get().is_some());
    }

    #[test]
    fn test_global_timing_accessors() {
        let mut timer = FrameTimer::new_with_global();

        // Initially, values should be zero
        assert_eq!(delta_time(), 0.0);
        assert_eq!(frame_count(), 0);

        // After tick, values should update
        timer.tick();
        assert_eq!(frame_count(), 1);

        // Delta time should be small but non-negative
        assert!(delta_time() >= 0.0);
        assert!(delta_duration() >= Duration::ZERO);
        assert!(total_time() >= Duration::ZERO);
    }

    #[test]
    fn test_stats_format() {
        let mut timer = FrameTimer::new();
        timer.tick();

        let stats = timer.stats();
        assert!(stats.contains("FPS:"));
        assert!(stats.contains("Frame time:"));
        assert!(stats.contains("Total frames:"));
    }

    #[test]
    fn test_elapsed_time() {
        let mut timer = FrameTimer::new();
        thread::sleep(Duration::from_millis(10));
        timer.tick();

        let elapsed = timer.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }
}
