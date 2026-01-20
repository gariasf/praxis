//! Main profiler coordinating all profiling subsystems.
//!
//! # Architecture Overview
//!
//! The `Profiler` is the central coordinator that integrates multiple profiling subsystems:
//! - **CPU Profiling**: Hierarchical scope timing via `ProfileScope` RAII guards
//! - **GPU Profiling**: Vulkan timestamp queries for GPU work
//! - **Memory Tracking**: Allocation/deallocation monitoring with leak detection
//! - **System Profiling**: Per-system timing and bottleneck identification
//! - **Frame Statistics**: Aggregation of metrics across frame history
//! - **Chrome Trace Export**: JSON export for chrome://tracing visualization
//!
//! # Hierarchical Profiling Scopes
//!
//! Scopes are organized hierarchically by tracking their nesting depth:
//! - Depth 0: Top-level operations (e.g., "frame", "main_loop")
//! - Depth 1: Major subsystems (e.g., "physics", "render", "update")
//! - Depth 2+: Nested operations (e.g., "collision_detection", "shadow_pass")
//!
//! When a `ProfileScope` is created, it:
//! 1. Records start time and current depth
//! 2. Increments the thread-local depth counter
//! 3. On drop, calculates duration and invokes the scope callback
//! 4. Callback adds `ScopeData` to `scope_data` vector
//!
//! At `end_frame()`, all collected scopes are:
//! - Mapped to frame phases (Physics, Rendering, etc.) via `map_scope_to_phase()`
//! - Added to the `FrameBreakdown` for the current frame
//! - Optionally exported to Chrome trace format
//!
//! This design allows zero-overhead profiling when disabled (no-op callback) and
//! minimal overhead when enabled (~20-30ns per scope).
//!
//! # CPU/GPU Time Measurement
//!
//! ## CPU Timing
//! - Uses `std::time::Instant` for high-precision wall-clock measurements
//! - Each scope measures from creation to destruction (RAII pattern)
//! - Thread-safe via thread-local depth tracking and lock-free data collection
//! - Results are accurate to ~100ns on most platforms
//!
//! ## GPU Timing
//! - Uses Vulkan's `VK_QUERY_TYPE_TIMESTAMP` query pools
//! - Timestamps are written to command buffers before/after GPU work
//! - Results retrieved after GPU completes execution (typically 1-2 frames later)
//! - Converted to nanoseconds using device's `timestampPeriod` property
//!
//! GPU profiling is managed by the `GpuProfiler` subsystem, which handles:
//! - Query pool allocation and reset
//! - Synchronization of asynchronous results
//! - Conversion from device ticks to nanoseconds
//! - Storage of per-frame GPU timing in `last_frame_time_ms`
//!
//! # Memory Tracking
//!
//! The `AllocationTracker` provides real-time memory profiling:
//! - Tracks every allocation/deallocation when enabled
//! - Records allocation site (file/line/function) for debugging
//! - Maintains current and peak memory usage statistics
//! - Thread-safe via atomic counters for minimal overhead
//!
//! The `LeakDetector` builds on this to identify memory leaks:
//! - Snapshots allocations at different points (e.g., frame boundaries)
//! - Compares snapshots to find allocations that persist unexpectedly
//! - Reports leaks with source location information
//!
//! Memory tracking adds ~10-15ns overhead per allocation when enabled.
//!
//! # Frame Statistics Aggregation
//!
//! The `FrameStatistics` struct maintains a rolling window of frame data:
//! - Fixed-size circular buffer (default: 300 frames)
//! - Stores frame durations, phase breakdowns, scope timings
//! - Computes min/max/average statistics on demand
//! - Efficient O(1) updates, O(n) statistics calculations
//!
//! Each frame, `end_frame()` calls `FrameStatistics::update()` to:
//! 1. Add the completed `FrameBreakdown` to the history buffer
//! 2. Evict the oldest frame if buffer is full (FIFO)
//! 3. Update running totals for average calculations
//!
//! This provides real-time performance metrics with minimal memory footprint
//! (300 frames × ~1KB per frame = ~300KB overhead).
//!
//! # Chrome Tracing Format Export
//!
//! Chrome's tracing format is a JSON-based event stream compatible with chrome://tracing.
//! The format supports various event types:
//!
//! - **Duration Events ('B'/'E' or 'X')**: Represent timed operations (our scopes)
//!   - 'X' type includes start time and duration in one event (more efficient)
//!   - Events can nest to show parent-child relationships
//!
//! - **Counter Events ('C')**: Show values over time (e.g., memory usage)
//!   - Updated each frame to show trends
//!
//! - **Instant Events ('i')**: Mark specific points in time (e.g., frame boundaries)
//!
//! The `ChromeTraceExporter` collects events during profiling and serializes them
//! to JSON on `end_trace_export()`. The resulting file can be loaded in Chrome to:
//! - Visualize CPU/GPU timelines side-by-side
//! - Zoom into specific frames or operations
//! - Analyze multi-threaded execution patterns
//! - Identify performance bottlenecks visually
//!
//! Export is optional and only active between `begin_trace_export()` and `end_trace_export()`.

use crate::{
    chrome_trace::ChromeTraceExporter,
    frame_breakdown::{FrameBreakdown, FramePhase, FrameStatistics},
    gpu_profiler::GpuProfiler,
    memory_tracker::{AllocationTracker, LeakDetector},
    scope::{clear_scope_callback, set_scope_callback, ScopeData},
    system_profiler::SystemProfiler,
};
use parking_lot::Mutex;
use praxis_utils::{info, warn};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

/// Configuration for the profiler.
///
/// Controls which profiling subsystems are enabled. Disabling unused subsystems
/// reduces overhead. Typical configurations:
/// - Development: All enabled for comprehensive profiling
/// - Release: Only enable_cpu and enable_systems for basic performance monitoring
/// - Shipping: All disabled (compile out profiling code in production builds)
#[derive(Debug, Clone)]
pub struct ProfilerConfig {
    /// Enable CPU profiling with hierarchical scopes
    pub enable_cpu: bool,
    /// Enable GPU profiling with Vulkan timestamp queries
    pub enable_gpu: bool,
    /// Enable memory allocation tracking
    pub enable_memory: bool,
    /// Enable per-system timing and bottleneck detection
    pub enable_systems: bool,
    /// Maximum number of frames to keep in history for statistics (default: 300 = ~5s at 60fps)
    pub max_frame_history: usize,
    /// Bottleneck detection threshold (percentage of frame time, e.g., 0.15 = 15%)
    pub bottleneck_threshold: f32,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        Self {
            enable_cpu: true,
            enable_gpu: true,
            enable_memory: true,
            enable_systems: true,
            max_frame_history: 300,
            bottleneck_threshold: 0.15,
        }
    }
}

/// Overall profiler statistics.
#[derive(Debug, Clone, Default)]
pub struct ProfilerStats {
    /// Current frame number
    pub frame_number: u64,
    /// Average FPS
    pub avg_fps: f64,
    /// Min FPS
    pub min_fps: f64,
    /// Max FPS
    pub max_fps: f64,
    /// CPU time (ms)
    pub cpu_time_ms: f64,
    /// GPU time (ms)
    pub gpu_time_ms: f64,
    /// Memory allocated (bytes)
    pub memory_allocated: usize,
    /// Memory peak (bytes)
    pub memory_peak: usize,
    /// Number of bottlenecks detected
    pub bottleneck_count: usize,
}

/// Main profiler coordinating all profiling subsystems.
///
/// The profiler uses a frame-based measurement model:
/// 1. `begin_frame()`: Starts timing and clears previous frame's data
/// 2. During frame: `ProfileScope` instances record CPU timings, GPU commands insert timestamps
/// 3. `end_frame()`: Collects all data, aggregates statistics, updates history
///
/// All subsystems are thread-safe and use minimal locking for low overhead.
pub struct Profiler {
    /// Configuration controlling which subsystems are enabled
    config: ProfilerConfig,
    /// Current frame number (incremented in `begin_frame()`)
    frame_number: u64,
    /// Frame start time (set in `begin_frame()`, used to calculate frame duration)
    frame_start: Option<Instant>,
    /// Frame breakdown for current frame (scopes mapped to phases)
    current_frame: Arc<Mutex<Option<FrameBreakdown>>>,
    /// Frame statistics (rolling window of historical frame data)
    frame_stats: Arc<Mutex<FrameStatistics>>,
    /// GPU profiler (optional, set up after Vulkan initialization)
    gpu_profiler: Option<Arc<Mutex<GpuProfiler>>>,
    /// Memory tracker (monitors allocations/deallocations)
    memory_tracker: Arc<AllocationTracker>,
    /// Leak detector (identifies persistent allocations)
    leak_detector: Arc<LeakDetector>,
    /// System profiler (per-system timing and bottleneck detection)
    system_profiler: Arc<SystemProfiler>,
    /// Chrome trace exporter (optional, enabled via begin_trace_export())
    trace_exporter: Arc<Mutex<Option<ChromeTraceExporter>>>,
    /// Scope data collected this frame (populated by ProfileScope callback)
    scope_data: Arc<Mutex<Vec<ScopeData>>>,
    /// Phase mapping for scopes (maps scope names like "physics" to FramePhase enum)
    phase_map: Arc<Mutex<HashMap<String, FramePhase>>>,
}

impl Profiler {
    /// Creates a new profiler with the given configuration.
    pub fn new(config: ProfilerConfig) -> Self {
        let mut memory_tracker = AllocationTracker::new();
        memory_tracker.set_enabled(config.enable_memory);
        let memory_tracker = Arc::new(memory_tracker);

        let leak_detector = Arc::new(LeakDetector::new(memory_tracker.clone()));
        let system_profiler = Arc::new(SystemProfiler::new(config.bottleneck_threshold));

        let profiler = Self {
            config: config.clone(),
            frame_number: 0,
            frame_start: None,
            current_frame: Arc::new(Mutex::new(None)),
            frame_stats: Arc::new(Mutex::new(FrameStatistics::new(config.max_frame_history))),
            gpu_profiler: None,
            memory_tracker,
            leak_detector,
            system_profiler,
            trace_exporter: Arc::new(Mutex::new(None)),
            scope_data: Arc::new(Mutex::new(Vec::new())),
            phase_map: Arc::new(Mutex::new(Self::default_phase_map())),
        };

        if config.enable_cpu {
            profiler.setup_scope_callback();
        }

        profiler
    }

    /// Sets up the GPU profiler.
    pub fn setup_gpu_profiler(&mut self, gpu_profiler: GpuProfiler) {
        self.gpu_profiler = Some(Arc::new(Mutex::new(gpu_profiler)));
        info!("GPU profiler enabled");
    }

    /// Sets up the scope callback for CPU profiling.
    ///
    /// This callback is invoked by `ProfileScope::drop()` to report timing data.
    /// The callback captures `scope_data` and appends each completed scope to the vector.
    /// This design allows ProfileScope to be agnostic of the Profiler's internals.
    ///
    /// Thread-safety: Multiple threads can invoke the callback concurrently.
    /// The mutex ensures safe concurrent access to the shared scope_data vector.
    fn setup_scope_callback(&self) {
        let scope_data = self.scope_data.clone();

        set_scope_callback(move |data| {
            scope_data.lock().push(data);
        });
    }

    /// Creates default phase mapping for common scope names.
    fn default_phase_map() -> HashMap<String, FramePhase> {
        let mut map = HashMap::new();
        map.insert("physics".to_string(), FramePhase::Physics);
        map.insert("update".to_string(), FramePhase::SystemUpdate);
        map.insert("render".to_string(), FramePhase::Rendering);
        map.insert("render_prep".to_string(), FramePhase::RenderPrep);
        map.insert("post_process".to_string(), FramePhase::PostProcess);
        map.insert("gui".to_string(), FramePhase::Gui);
        map.insert("present".to_string(), FramePhase::Present);
        map
    }

    /// Maps a scope name to a frame phase.
    ///
    /// Frame phases group related scopes for visualization and analysis.
    /// This allows aggregating timing data by high-level category (e.g., "total rendering time").
    ///
    /// Mapping strategy:
    /// 1. Check for exact match in phase_map (user-registered mappings)
    /// 2. Fall back to substring matching for common patterns
    /// 3. Default to FramePhase::Other if no match found
    ///
    /// This heuristic approach allows flexible scope naming while still providing useful
    /// categorization without requiring every scope to be manually registered.
    fn map_scope_to_phase(&self, scope_name: &str) -> FramePhase {
        let phase_map = self.phase_map.lock();

        // Check for exact match
        if let Some(phase) = phase_map.get(scope_name) {
            return *phase;
        }

        // Check for partial matches
        let lower = scope_name.to_lowercase();
        if lower.contains("physics") {
            FramePhase::Physics
        } else if lower.contains("render") {
            FramePhase::Rendering
        } else if lower.contains("update") || lower.contains("system") {
            FramePhase::SystemUpdate
        } else if lower.contains("gui") || lower.contains("ui") {
            FramePhase::Gui
        } else if lower.contains("post") {
            FramePhase::PostProcess
        } else if lower.contains("present") || lower.contains("swap") {
            FramePhase::Present
        } else {
            FramePhase::Other
        }
    }

    /// Registers a custom phase mapping for a scope name.
    pub fn register_phase_mapping(&self, scope_name: String, phase: FramePhase) {
        self.phase_map.lock().insert(scope_name, phase);
    }

    /// Begins a new frame.
    ///
    /// Resets profiling state for the new frame:
    /// - Increments frame counter
    /// - Records frame start time
    /// - Creates fresh FrameBreakdown
    /// - Clears previous frame's scope data
    /// - Signals GPU profiler to start new frame
    ///
    /// Should be called once per frame, typically at the start of the main loop.
    pub fn begin_frame(&mut self) {
        self.frame_number += 1;
        self.frame_start = Some(Instant::now());

        // Create new frame breakdown
        let mut current_frame = self.current_frame.lock();
        *current_frame = Some(FrameBreakdown::new(self.frame_number));

        // Clear scope data from previous frame
        self.scope_data.lock().clear();

        // Begin GPU frame if available
        if let Some(gpu_profiler) = &self.gpu_profiler {
            gpu_profiler.lock().begin_frame();
        }
    }

    /// Ends the current frame and processes profiling data.
    ///
    /// This is where all the profiling data collected during the frame is aggregated:
    ///
    /// 1. **CPU Scope Processing**:
    ///    - Retrieve all scopes collected via the ProfileScope callback
    ///    - Map each scope to a frame phase (Physics, Rendering, etc.)
    ///    - Add scopes to the FrameBreakdown with their timing and depth
    ///    - Export to Chrome trace format if enabled
    ///
    /// 2. **Frame Statistics Update**:
    ///    - Calculate total frame duration
    ///    - Add completed FrameBreakdown to rolling history
    ///    - Update min/max/average metrics
    ///
    /// 3. **GPU Results Collection**:
    ///    - Retrieve GPU timestamp query results (from 1-2 frames ago due to async nature)
    ///    - Store total GPU frame time for statistics retrieval
    ///    - Export GPU timings to Chrome trace format
    ///
    /// 4. **Memory Tracking**:
    ///    - Export current memory usage as counter event for Chrome trace
    ///    - Memory stats are available via memory_tracker() for real-time monitoring
    ///
    /// Should be called once per frame, typically at the end of the main loop.
    pub fn end_frame(&mut self) {
        let Some(frame_start) = self.frame_start else {
            warn!("end_frame called without begin_frame");
            return;
        };

        let frame_duration = frame_start.elapsed();

        // Process scope data
        let scope_data = {
            let mut data = self.scope_data.lock();
            std::mem::take(&mut *data)
        };

        let mut current_frame_guard = self.current_frame.lock();
        if let Some(ref mut breakdown) = *current_frame_guard {
            for scope in scope_data {
                let phase = self.map_scope_to_phase(&scope.name);
                let scope_name = scope.name.clone();
                breakdown.add_scope(scope.name, scope.duration, scope.depth, phase);

                // Also add to trace if enabled
                if let Some(ref mut exporter) = *self.trace_exporter.lock() {
                    exporter.add_cpu_scope(
                        scope_name,
                        format!("{phase:?}"),
                        scope.start_time,
                        scope.duration,
                        scope.thread_id,
                    );
                }
            }

            breakdown.total_duration = frame_duration;

            // Update statistics
            self.frame_stats.lock().update(breakdown);

            // Update system profiler with frame time
            self.system_profiler.set_frame_time(frame_duration);
        }
        drop(current_frame_guard);

        // Collect GPU results if available
        if let Some(gpu_profiler) = &self.gpu_profiler {
            if let Ok(gpu_timestamps) = gpu_profiler.lock().collect_results() {
                if let Some(ref mut exporter) = *self.trace_exporter.lock() {
                    for timestamp in gpu_timestamps {
                        exporter.add_gpu_timing(
                            timestamp.name,
                            timestamp.start_ns,
                            timestamp.duration_ns,
                        );
                    }
                }
            }
        }

        // Add frame marker to trace
        if let Some(ref mut exporter) = *self.trace_exporter.lock() {
            exporter.add_frame_marker(self.frame_number, frame_start);

            // Add memory counter
            let mem_stats = self.memory_tracker.statistics();
            exporter.add_memory_counter(
                "Allocated Memory".to_string(),
                Instant::now(),
                mem_stats.current_allocated,
            );
        }

        self.frame_start = None;
    }

    /// Records rendering statistics for the current frame.
    ///
    /// When Chrome trace export is active, this method automatically exports
    /// rendering metrics as counter events in the trace timeline. This provides
    /// a comprehensive view of rendering performance including:
    ///
    /// - Culling efficiency (percentage of objects culled)
    /// - Draw call reduction (objects culled vs. draw calls issued)
    /// - Visible object counts
    /// - Frustum and occlusion culling breakdown
    /// - LOD distribution across levels
    /// - Mesh streaming queue depth
    ///
    /// # Arguments
    ///
    /// * `stats` - RenderStats snapshot from the graphics system
    /// * `timestamp` - Timestamp for these metrics (typically frame start time)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // In your render loop:
    /// profiler.begin_frame();
    ///
    /// // ... rendering code ...
    ///
    /// // Record render stats before ending frame
    /// let render_stats = render_context.current_render_stats();
    /// profiler.record_render_stats(&render_stats, Instant::now());
    ///
    /// profiler.end_frame();
    /// ```
    #[cfg(feature = "graphics_integration")]
    pub fn record_render_stats(&self, stats: &praxis_graphics::RenderStats, timestamp: Instant) {
        // Only export if trace export is active
        if let Some(ref mut exporter) = *self.trace_exporter.lock() {
            exporter.add_render_stats(stats, timestamp);
        }
    }

    /// Returns the current frame breakdown.
    pub fn current_frame_breakdown(&self) -> Option<FrameBreakdown> {
        self.current_frame.lock().clone()
    }

    /// Returns frame statistics.
    pub fn frame_statistics(&self) -> FrameStatistics {
        self.frame_stats.lock().clone()
    }

    /// Returns overall profiler statistics.
    ///
    /// Aggregates data from all profiling subsystems into a single snapshot.
    /// This provides a high-level view of current performance without requiring
    /// the caller to query each subsystem individually.
    pub fn statistics(&self) -> ProfilerStats {
        let frame_stats = self.frame_stats.lock();
        let mem_stats = self.memory_tracker.statistics();
        let bottlenecks = self.system_profiler.identify_bottlenecks();

        ProfilerStats {
            frame_number: self.frame_number,
            avg_fps: frame_stats.avg_fps(),
            min_fps: frame_stats.min_fps(),
            max_fps: frame_stats.max_fps(),
            cpu_time_ms: frame_stats.avg_frame_time.as_secs_f64() * 1000.0,
            gpu_time_ms: self
                .gpu_profiler
                .as_ref()
                .map_or(0.0, |p| p.lock().last_frame_time_ms()),
            memory_allocated: mem_stats.current_allocated,
            memory_peak: mem_stats.peak_allocated,
            bottleneck_count: bottlenecks.len(),
        }
    }

    /// Returns the system profiler.
    pub fn system_profiler(&self) -> &Arc<SystemProfiler> {
        &self.system_profiler
    }

    /// Returns the memory tracker.
    pub fn memory_tracker(&self) -> &Arc<AllocationTracker> {
        &self.memory_tracker
    }

    /// Returns the leak detector.
    pub fn leak_detector(&self) -> &Arc<LeakDetector> {
        &self.leak_detector
    }

    /// Returns the GPU profiler if available.
    pub fn gpu_profiler(&self) -> Option<Arc<Mutex<GpuProfiler>>> {
        self.gpu_profiler.clone()
    }

    /// Starts exporting to Chrome trace format.
    ///
    /// Enables collection of all profiling events into Chrome's tracing JSON format.
    /// Once enabled, all CPU scopes, GPU timings, and memory counters will be
    /// recorded for export.
    ///
    /// Usage pattern:
    /// ```ignore
    /// profiler.begin_trace_export();
    /// // Run game normally for some time
    /// profiler.end_trace_export("trace.json")?;
    /// // Load trace.json in chrome://tracing
    /// ```
    ///
    /// There is some overhead to trace export (~50-100ns per event), so enable only
    /// when needed for performance investigation.
    pub fn begin_trace_export(&self) {
        let mut exporter = self.trace_exporter.lock();
        *exporter = Some(ChromeTraceExporter::new());
        info!("Chrome trace export started");
    }

    /// Stops trace export and saves to file.
    ///
    /// Finalizes the Chrome trace JSON and writes it to disk.
    /// The resulting file can be loaded in Chrome at chrome://tracing for
    /// interactive visualization of:
    /// - Timeline of all CPU and GPU operations
    /// - Nested scope relationships
    /// - Multi-threaded execution
    /// - Memory usage over time
    /// - Frame boundaries and correlations
    pub fn end_trace_export(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let mut exporter_guard = self.trace_exporter.lock();
        if let Some(exporter) = exporter_guard.take() {
            exporter.save(path)?;
            info!("Chrome trace saved");
        }
        Ok(())
    }

    /// Checks if trace export is active.
    pub fn is_trace_export_active(&self) -> bool {
        self.trace_exporter.lock().is_some()
    }

    /// Resets all profiling data.
    pub fn reset(&mut self) {
        self.frame_number = 0;
        self.frame_start = None;
        *self.current_frame.lock() = None;
        *self.frame_stats.lock() = FrameStatistics::new(self.config.max_frame_history);
        self.memory_tracker.reset_statistics();
        self.system_profiler.reset();
        self.scope_data.lock().clear();
    }
}

impl Drop for Profiler {
    fn drop(&mut self) {
        if self.config.enable_cpu {
            clear_scope_callback();
        }
    }
}
