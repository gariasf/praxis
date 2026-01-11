//! Profiling and performance analysis tools for the Praxis engine.
//!
//! This crate provides comprehensive profiling capabilities including:
//! - Frame time breakdown and visualization
//! - GPU profiling with Vulkan timestamp queries
//! - Memory allocation tracking and leak detection
//! - Bottleneck identification for systems and entities
//! - Export to Chrome tracing format for analysis
//!
//! # Hierarchical Profiling Scopes
//!
//! The profiling system uses hierarchical scopes to measure nested code execution times.
//! Scopes are tracked by depth, allowing parent-child relationships to be visualized.
//! This enables identifying which subsystems contribute most to frame time.
//!
//! When a `ProfileScope` is created, it records the start time. When dropped (via RAII),
//! it calculates the elapsed duration and reports it to the profiler via a callback.
//! Each scope tracks:
//! - Name: Identifies what code is being measured (e.g., "physics", "render")
//! - Depth: Nesting level (0 = top-level, 1 = nested once, etc.)
//! - Start time: When the scope began
//! - Duration: How long the scope took to execute
//! - Thread ID: Which thread executed the scope
//!
//! # CPU/GPU Time Measurement
//!
//! ## CPU Profiling
//! CPU time is measured using `std::time::Instant` for high-precision timing.
//! Each `ProfileScope` measures wall-clock time from creation to destruction.
//! The profiler aggregates these measurements into frame phases and statistics.
//!
//! ## GPU Profiling
//! GPU time is measured using Vulkan timestamp queries:
//! 1. `vkCmdWriteTimestamp` commands are inserted into command buffers before/after GPU work
//! 2. After command buffer execution, timestamps are retrieved from the query pool
//! 3. Delta between timestamps gives GPU execution time for that work
//! 4. Results are converted to nanoseconds using the device's timestamp period
//!
//! GPU profiling is asynchronous - results from frame N may not be available until frame N+2
//! due to GPU pipelining. The `GpuProfiler` handles this synchronization automatically.
//!
//! # Memory Tracking
//!
//! The `AllocationTracker` monitors memory allocations and deallocations:
//! - Tracks current allocated bytes, peak usage, allocation count
//! - Records allocation sites (file, line, function) for leak detection
//! - `LeakDetector` identifies allocations that persist across frames
//! - Statistics are aggregated per-frame for trend analysis
//!
//! Memory tracking has minimal overhead (~10ns per allocation) when enabled.
//!
//! # Frame Statistics Aggregation
//!
//! The `FrameStatistics` struct maintains a rolling history of frame measurements:
//! - Frame durations (for FPS calculation)
//! - Phase breakdowns (how much time spent in physics, rendering, etc.)
//! - Min/max/average values over the history window
//! - Per-frame scope timings for detailed analysis
//!
//! Statistics are updated each frame in `Profiler::end_frame()`. The history size
//! is configurable (default: 300 frames = ~5 seconds at 60 FPS).
//!
//! # Chrome Tracing Format Export
//!
//! The profiler can export to Chrome's `chrome://tracing` format (JSON):
//! - Each scope becomes a duration event (start time + duration)
//! - GPU timings are exported as separate GPU track events
//! - Memory counters are exported as counter events (show memory over time)
//! - Frame markers show frame boundaries for correlation
//!
//! This format allows visualization of:
//! - Timeline view of all CPU and GPU work
//! - Nested scope relationships (parent/child)
//! - Multi-threaded execution patterns
//! - Performance across frames
//!
//! To export: call `begin_trace_export()`, run normally, then `end_trace_export(path)`.
//! Load the resulting JSON file in Chrome at chrome://tracing for interactive analysis.
//!
//! # Example
//!
//! ```rust,ignore
//! use praxis_profiling::{Profiler, ProfileScope};
//!
//! let mut profiler = Profiler::new();
//! profiler.begin_frame();
//!
//! {
//!     let _scope = ProfileScope::new("update_physics");
//!     // Physics update code
//! }
//!
//! profiler.end_frame();
//! ```

mod chrome_trace;
mod frame_breakdown;
mod gpu_profiler;
mod integration;
mod memory_tracker;
mod profiler;
mod scope;
mod system_profiler;
mod visualization;

pub use chrome_trace::{ChromeTrace, ChromeTraceEvent, ChromeTraceExporter};
pub use frame_breakdown::{FrameBreakdown, FramePhase, FrameStatistics};
pub use gpu_profiler::{GpuProfiler, GpuTimestamp, TimestampQuery};
pub use integration::{ProfilerResource, SystemProfilerResource};
pub use memory_tracker::{AllocationTracker, LeakDetector, MemoryAllocation, MemoryStatistics};
pub use profiler::{Profiler, ProfilerConfig, ProfilerStats};
pub use scope::{ProfileScope, ScopeId};
pub use system_profiler::{BottleneckInfo, BottleneckType, SystemProfiler, SystemStats};
pub use visualization::{
    FrameTimeGraph, MemoryGraph, PhaseColor, PhasePieChart, ProfilingVisualization, SystemBarChart,
};
