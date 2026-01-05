//! Profiling visualization utilities.
//!
//! This module provides data structures and utilities for visualizing
//! profiling data in GUI systems.

use crate::{FrameBreakdown, FramePhase, FrameStatistics, SystemStats};
use std::collections::VecDeque;
use std::time::Duration;

/// Color for visualizing frame phases.
#[derive(Debug, Clone, Copy)]
pub struct PhaseColor {
    /// Red component (0.0 - 1.0)
    pub r: f32,
    /// Green component (0.0 - 1.0)
    pub g: f32,
    /// Blue component (0.0 - 1.0)
    pub b: f32,
    /// Alpha component (0.0 - 1.0)
    pub a: f32,
}

impl PhaseColor {
    /// Creates a new color.
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Returns the color for a given frame phase.
    pub fn for_phase(phase: FramePhase) -> Self {
        match phase {
            FramePhase::SystemUpdate => Self::new(0.2, 0.6, 0.9, 1.0), // Blue
            FramePhase::Physics => Self::new(0.9, 0.4, 0.2, 1.0),      // Orange
            FramePhase::RenderPrep => Self::new(0.6, 0.3, 0.8, 1.0),   // Purple
            FramePhase::Rendering => Self::new(0.2, 0.8, 0.4, 1.0),    // Green
            FramePhase::PostProcess => Self::new(0.8, 0.8, 0.2, 1.0),  // Yellow
            FramePhase::Gui => Self::new(0.9, 0.5, 0.7, 1.0),          // Pink
            FramePhase::Present => Self::new(0.5, 0.5, 0.5, 1.0),      // Gray
            FramePhase::Other => Self::new(0.7, 0.7, 0.7, 1.0),        // Light gray
        }
    }
}

/// Frame time graph data for visualization.
#[derive(Debug, Clone)]
pub struct FrameTimeGraph {
    /// Frame times (in milliseconds)
    pub frame_times: VecDeque<f32>,
    /// Maximum number of frames to keep
    pub max_frames: usize,
    /// Target frame time (for reference line)
    pub target_frame_time_ms: f32,
}

impl FrameTimeGraph {
    /// Creates a new frame time graph.
    pub fn new(max_frames: usize, target_fps: f64) -> Self {
        Self {
            frame_times: VecDeque::with_capacity(max_frames),
            max_frames,
            target_frame_time_ms: (1000.0 / target_fps) as f32,
        }
    }

    /// Adds a frame time to the graph.
    pub fn add_frame_time(&mut self, duration: Duration) {
        if self.frame_times.len() >= self.max_frames {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(duration.as_secs_f32() * 1000.0);
    }

    /// Returns the minimum frame time in the graph.
    pub fn min(&self) -> f32 {
        self.frame_times
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    /// Returns the maximum frame time in the graph.
    pub fn max(&self) -> f32 {
        self.frame_times
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    /// Returns the average frame time in the graph.
    pub fn average(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
    }

    /// Returns frame times as a slice for plotting.
    pub fn data(&self) -> Vec<f32> {
        self.frame_times.iter().copied().collect()
    }
}

impl Default for FrameTimeGraph {
    fn default() -> Self {
        Self::new(300, 60.0)
    }
}

/// Pie chart data for phase visualization.
#[derive(Debug, Clone)]
pub struct PhasePieChart {
    /// Phase segments with colors
    pub segments: Vec<(FramePhase, f32, PhaseColor)>,
}

impl PhasePieChart {
    /// Creates a pie chart from a frame breakdown.
    pub fn from_breakdown(breakdown: &FrameBreakdown) -> Self {
        let total = breakdown.total_duration.as_secs_f32();
        if total == 0.0 {
            return Self {
                segments: Vec::new(),
            };
        }

        let mut segments = Vec::new();
        for phase in [
            FramePhase::SystemUpdate,
            FramePhase::Physics,
            FramePhase::RenderPrep,
            FramePhase::Rendering,
            FramePhase::PostProcess,
            FramePhase::Gui,
            FramePhase::Present,
            FramePhase::Other,
        ] {
            let duration = breakdown
                .phase_times
                .get(&phase)
                .copied()
                .unwrap_or(Duration::ZERO);
            let percentage = (duration.as_secs_f32() / total) * 100.0;

            if percentage > 0.0 {
                segments.push((phase, percentage, PhaseColor::for_phase(phase)));
            }
        }

        segments.sort_by(|(_, a, _), (_, b, _)| b.partial_cmp(a).unwrap());

        Self { segments }
    }
}

/// Bar chart data for system timing visualization.
#[derive(Debug, Clone)]
pub struct SystemBarChart {
    /// System entries (name, time in ms, percentage)
    pub entries: Vec<(String, f32, f32)>,
}

impl SystemBarChart {
    /// Creates a bar chart from system statistics.
    pub fn from_system_stats(stats: &[SystemStats], limit: usize) -> Self {
        let entries = stats
            .iter()
            .take(limit)
            .map(|stat| {
                (
                    stat.name.clone(),
                    stat.avg_time.as_secs_f32() * 1000.0,
                    stat.frame_percentage,
                )
            })
            .collect();

        Self { entries }
    }
}

/// Memory usage graph for visualization.
#[derive(Debug, Clone)]
pub struct MemoryGraph {
    /// Memory usage over time (in bytes)
    pub memory_usage: VecDeque<usize>,
    /// Maximum number of samples to keep
    pub max_samples: usize,
}

impl MemoryGraph {
    /// Creates a new memory graph.
    pub fn new(max_samples: usize) -> Self {
        Self {
            memory_usage: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    /// Adds a memory usage sample.
    pub fn add_sample(&mut self, bytes: usize) {
        if self.memory_usage.len() >= self.max_samples {
            self.memory_usage.pop_front();
        }
        self.memory_usage.push_back(bytes);
    }

    /// Returns memory usage in megabytes for plotting.
    pub fn data_mb(&self) -> Vec<f32> {
        self.memory_usage
            .iter()
            .map(|&bytes| bytes as f32 / (1024.0 * 1024.0))
            .collect()
    }

    /// Returns the maximum memory usage.
    pub fn max_mb(&self) -> f32 {
        self.memory_usage.iter().max().copied().unwrap_or(0) as f32 / (1024.0 * 1024.0)
    }

    /// Returns the current memory usage.
    pub fn current_mb(&self) -> f32 {
        self.memory_usage.back().copied().unwrap_or(0) as f32 / (1024.0 * 1024.0)
    }
}

impl Default for MemoryGraph {
    fn default() -> Self {
        Self::new(300)
    }
}

/// Complete profiling visualization data.
#[derive(Debug, Clone)]
pub struct ProfilingVisualization {
    /// Frame time graph
    pub frame_time_graph: FrameTimeGraph,
    /// Phase pie chart
    pub phase_pie_chart: Option<PhasePieChart>,
    /// System bar chart
    pub system_bar_chart: SystemBarChart,
    /// Memory graph
    pub memory_graph: MemoryGraph,
}

impl ProfilingVisualization {
    /// Creates a new profiling visualization.
    pub fn new() -> Self {
        Self {
            frame_time_graph: FrameTimeGraph::default(),
            phase_pie_chart: None,
            system_bar_chart: SystemBarChart {
                entries: Vec::new(),
            },
            memory_graph: MemoryGraph::default(),
        }
    }

    /// Updates the visualization with new profiling data.
    pub fn update(
        &mut self,
        breakdown: Option<&FrameBreakdown>,
        stats: &FrameStatistics,
        system_stats: &[SystemStats],
        memory_bytes: usize,
    ) {
        // Update frame time graph
        if !stats.recent_frame_times.is_empty() {
            if let Some(&last_time) = stats.recent_frame_times.last() {
                self.frame_time_graph.add_frame_time(last_time);
            }
        }

        // Update phase pie chart
        if let Some(breakdown) = breakdown {
            self.phase_pie_chart = Some(PhasePieChart::from_breakdown(breakdown));
        }

        // Update system bar chart
        self.system_bar_chart = SystemBarChart::from_system_stats(system_stats, 10);

        // Update memory graph
        self.memory_graph.add_sample(memory_bytes);
    }
}

impl Default for ProfilingVisualization {
    fn default() -> Self {
        Self::new()
    }
}
