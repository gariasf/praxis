//! Profiling and performance analysis tools for the Praxis engine.
//!
//! This crate provides comprehensive profiling capabilities including:
//! - Frame time breakdown and visualization
//! - GPU profiling with Vulkan timestamp queries
//! - Memory allocation tracking and leak detection
//! - Bottleneck identification for systems and entities
//! - Export to Chrome tracing format for analysis
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
