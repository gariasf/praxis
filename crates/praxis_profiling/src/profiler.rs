//! Main profiler coordinating all profiling subsystems.

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
#[derive(Debug, Clone)]
pub struct ProfilerConfig {
    /// Enable CPU profiling
    pub enable_cpu: bool,
    /// Enable GPU profiling
    pub enable_gpu: bool,
    /// Enable memory tracking
    pub enable_memory: bool,
    /// Enable system profiling
    pub enable_systems: bool,
    /// Maximum number of frames to keep in history
    pub max_frame_history: usize,
    /// Bottleneck detection threshold (percentage of frame time)
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
pub struct Profiler {
    /// Configuration
    config: ProfilerConfig,
    /// Current frame number
    frame_number: u64,
    /// Frame start time
    frame_start: Option<Instant>,
    /// Frame breakdown for current frame
    current_frame: Arc<Mutex<Option<FrameBreakdown>>>,
    /// Frame statistics
    frame_stats: Arc<Mutex<FrameStatistics>>,
    /// GPU profiler
    gpu_profiler: Option<Arc<Mutex<GpuProfiler>>>,
    /// Memory tracker
    memory_tracker: Arc<AllocationTracker>,
    /// Leak detector
    leak_detector: Arc<LeakDetector>,
    /// System profiler
    system_profiler: Arc<SystemProfiler>,
    /// Chrome trace exporter
    trace_exporter: Arc<Mutex<Option<ChromeTraceExporter>>>,
    /// Scope data collected this frame
    scope_data: Arc<Mutex<Vec<ScopeData>>>,
    /// Phase mapping for scopes
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

    /// Returns the current frame breakdown.
    pub fn current_frame_breakdown(&self) -> Option<FrameBreakdown> {
        self.current_frame.lock().clone()
    }

    /// Returns frame statistics.
    pub fn frame_statistics(&self) -> FrameStatistics {
        self.frame_stats.lock().clone()
    }

    /// Returns overall profiler statistics.
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
            gpu_time_ms: 0.0, // TODO: Get from GPU profiler
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
    pub fn begin_trace_export(&self) {
        let mut exporter = self.trace_exporter.lock();
        *exporter = Some(ChromeTraceExporter::new());
        info!("Chrome trace export started");
    }

    /// Stops trace export and saves to file.
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
