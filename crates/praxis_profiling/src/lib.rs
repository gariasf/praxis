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
//! - Rendering metrics are exported as counter events (culling, draw calls, LOD distribution)
//! - Frame markers show frame boundaries for correlation
//!
//! This format allows visualization of:
//! - Timeline view of all CPU and GPU work
//! - Nested scope relationships (parent/child)
//! - Multi-threaded execution patterns
//! - Performance across frames
//! - Rendering optimization metrics over time
//!
//! ## Rendering Statistics Integration
//!
//! When the `graphics_integration` feature is enabled (default), the profiler can automatically
//! export rendering statistics from `praxis_graphics::RenderStats` as counter events:
//!
//! ### Rendering Metrics
//! - **Culling Efficiency**: Percentage of objects successfully culled
//! - **Draw Call Reduction**: Number of draw calls saved by culling and batching
//! - **Visible Objects**: Objects rendered after culling
//! - **Frustum Culled**: Objects culled by frustum test
//! - **Occlusion Culled**: Objects culled by occlusion test
//! - **LOD Distribution**: Percentage of objects at each LOD level
//! - **Streaming Queue**: Mesh streaming queue depth
//!
//! ### GPU Memory Metrics
//! - **VRAM Total**: Total GPU memory usage in megabytes
//! - **VRAM Texture**: Memory used by textures
//! - **VRAM Mesh**: Memory used by mesh buffers (vertex/index)
//! - **VRAM Descriptor**: Memory used by descriptor sets
//! - **VRAM Compute**: Memory used by compute shader buffers
//! - **VRAM Render Target**: Memory used by render targets (framebuffers, shadow maps, etc.)
//! - **Memory Allocations**: Total number of active GPU allocations
//!
//! To export: call `begin_trace_export()`, run normally, then `end_trace_export(path)`.
//! Load the resulting JSON file in Chrome at chrome://tracing for interactive analysis.
//!
//! # Example
//!
//! ## Basic CPU Profiling
//!
//! ```rust,ignore
//! use praxis_profiling::{Profiler, ProfileScope};
//!
//! let mut profiler = Profiler::new(ProfilerConfig::default());
//! profiler.begin_frame();
//!
//! {
//!     let _scope = ProfileScope::new("update_physics");
//!     // Physics update code
//! }
//!
//! profiler.end_frame();
//! ```
//!
//! ## Chrome Trace Export with Rendering Statistics
//!
//! ```rust,ignore
//! use praxis_profiling::Profiler;
//! use std::time::Instant;
//!
//! let mut profiler = Profiler::new(ProfilerConfig::default());
//!
//! // Start trace export
//! profiler.begin_trace_export();
//!
//! // Main loop
//! for frame in 0..300 {
//!     profiler.begin_frame();
//!
//!     // ... rendering code ...
//!
//!     // Record rendering statistics (when graphics_integration feature is enabled)
//!     #[cfg(feature = "graphics_integration")]
//!     {
//!         let render_stats = render_context.current_render_stats();
//!         profiler.record_render_stats(&render_stats, Instant::now());
//!     }
//!
//!     profiler.end_frame();
//! }
//!
//! // Save trace file
//! profiler.end_trace_export("trace.json")?;
//! // Load trace.json in chrome://tracing to visualize performance
//! ```

mod chrome_trace;
mod frame_breakdown;
mod gpu_profiler;
mod integration;
mod memory_tracker;
mod profiler;
#[cfg(feature = "graphics_integration")]
mod render_stats_integration;
mod scope;
mod system_profiler;
mod visualization;

pub use chrome_trace::{ChromeTrace, ChromeTraceEvent, ChromeTraceExporter};
pub use frame_breakdown::{FrameBreakdown, FramePhase, FrameStatistics};
pub use gpu_profiler::{GpuProfiler, GpuTimestamp, TimestampQuery};
pub use integration::{ProfilerResource, SystemProfilerResource};
pub use memory_tracker::{AllocationTracker, LeakDetector, MemoryAllocation, MemoryStatistics};
pub use profiler::{Profiler, ProfilerConfig, ProfilerStats};
#[cfg(feature = "graphics_integration")]
pub use render_stats_integration::conversion;
pub use scope::{ProfileScope, ScopeId};
pub use system_profiler::{BottleneckInfo, BottleneckType, SystemProfiler, SystemStats};
pub use visualization::{
    FrameTimeGraph, MemoryGraph, PhaseColor, PhasePieChart, ProfilingVisualization, SystemBarChart,
};
