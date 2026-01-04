//! Frame time breakdown and visualization.

use std::collections::HashMap;
use std::time::Duration;

/// Represents a phase of frame execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FramePhase {
    /// ECS system updates
    SystemUpdate,
    /// Physics simulation
    Physics,
    /// Rendering preparation
    RenderPrep,
    /// GPU rendering
    Rendering,
    /// Post-processing effects
    PostProcess,
    /// GUI rendering
    Gui,
    /// Present and swap
    Present,
    /// Other/unclassified time
    Other,
}

impl FramePhase {
    /// Returns a human-readable name for the phase.
    pub fn name(&self) -> &'static str {
        match self {
            Self::SystemUpdate => "System Update",
            Self::Physics => "Physics",
            Self::RenderPrep => "Render Prep",
            Self::Rendering => "Rendering",
            Self::PostProcess => "Post Process",
            Self::Gui => "GUI",
            Self::Present => "Present",
            Self::Other => "Other",
        }
    }
}

/// Detailed breakdown of frame timing.
#[derive(Debug, Clone)]
pub struct FrameBreakdown {
    /// Frame number
    pub frame_number: u64,
    /// Total frame time
    pub total_duration: Duration,
    /// Time spent in each phase
    pub phase_times: HashMap<FramePhase, Duration>,
    /// Individual scope timings with hierarchy
    pub scope_timings: Vec<ScopeTiming>,
}

/// Timing information for a single scope.
#[derive(Debug, Clone)]
pub struct ScopeTiming {
    /// Name of the scope
    pub name: String,
    /// Duration of the scope
    pub duration: Duration,
    /// Nesting depth
    pub depth: u32,
    /// Phase classification
    pub phase: FramePhase,
}

impl FrameBreakdown {
    /// Creates a new empty frame breakdown.
    pub fn new(frame_number: u64) -> Self {
        Self {
            frame_number,
            total_duration: Duration::ZERO,
            phase_times: HashMap::new(),
            scope_timings: Vec::new(),
        }
    }

    /// Adds a scope timing to this frame breakdown.
    pub fn add_scope(&mut self, name: String, duration: Duration, depth: u32, phase: FramePhase) {
        self.scope_timings.push(ScopeTiming {
            name,
            duration,
            depth,
            phase,
        });

        // Update phase timing
        *self.phase_times.entry(phase).or_insert(Duration::ZERO) += duration;
        self.total_duration += duration;
    }

    /// Returns the percentage of frame time spent in the given phase.
    pub fn phase_percentage(&self, phase: FramePhase) -> f32 {
        if self.total_duration.as_secs_f32() == 0.0 {
            return 0.0;
        }

        let phase_time = self
            .phase_times
            .get(&phase)
            .copied()
            .unwrap_or(Duration::ZERO);
        (phase_time.as_secs_f32() / self.total_duration.as_secs_f32()) * 100.0
    }

    /// Returns a formatted string showing the frame breakdown.
    pub fn format_breakdown(&self) -> String {
        let mut result = format!(
            "Frame {}: {:.2}ms total\n",
            self.frame_number,
            self.total_duration.as_secs_f64() * 1000.0
        );

        result.push_str("Phase breakdown:\n");
        let mut phases: Vec<_> = self.phase_times.iter().collect();
        phases.sort_by(|(_, a), (_, b)| b.cmp(a));

        for (phase, duration) in phases {
            let percentage = (duration.as_secs_f32() / self.total_duration.as_secs_f32()) * 100.0;
            result.push_str(&format!(
                "  {:<15} {:>7.2}ms ({:>5.1}%)\n",
                phase.name(),
                duration.as_secs_f64() * 1000.0,
                percentage
            ));
        }

        result
    }
}

/// Rolling statistics for frame timing over multiple frames.
#[derive(Debug, Clone)]
pub struct FrameStatistics {
    /// Number of frames tracked
    pub frame_count: usize,
    /// Average frame time
    pub avg_frame_time: Duration,
    /// Minimum frame time
    pub min_frame_time: Duration,
    /// Maximum frame time
    pub max_frame_time: Duration,
    /// Average time per phase
    pub avg_phase_times: HashMap<FramePhase, Duration>,
    /// Recent frame times (circular buffer)
    pub recent_frame_times: Vec<Duration>,
    /// Maximum number of recent frames to track
    max_recent_frames: usize,
}

impl FrameStatistics {
    /// Creates a new frame statistics tracker.
    pub fn new(max_recent_frames: usize) -> Self {
        Self {
            frame_count: 0,
            avg_frame_time: Duration::ZERO,
            min_frame_time: Duration::MAX,
            max_frame_time: Duration::ZERO,
            avg_phase_times: HashMap::new(),
            recent_frame_times: Vec::with_capacity(max_recent_frames),
            max_recent_frames,
        }
    }

    /// Updates statistics with a new frame breakdown.
    pub fn update(&mut self, breakdown: &FrameBreakdown) {
        self.frame_count += 1;

        // Update frame time statistics
        if breakdown.total_duration < self.min_frame_time {
            self.min_frame_time = breakdown.total_duration;
        }
        if breakdown.total_duration > self.max_frame_time {
            self.max_frame_time = breakdown.total_duration;
        }

        // Update rolling average
        let alpha = 0.1; // Smoothing factor
        if self.frame_count == 1 {
            self.avg_frame_time = breakdown.total_duration;
        } else {
            let current_avg = self.avg_frame_time.as_secs_f32();
            let new_value = breakdown.total_duration.as_secs_f32();
            let smoothed = current_avg * (1.0 - alpha) + new_value * alpha;
            self.avg_frame_time = Duration::from_secs_f32(smoothed);
        }

        // Update phase averages
        for (phase, duration) in &breakdown.phase_times {
            let avg = self
                .avg_phase_times
                .entry(*phase)
                .or_insert(Duration::ZERO);

            if self.frame_count == 1 {
                *avg = *duration;
            } else {
                let current_avg = avg.as_secs_f32();
                let new_value = duration.as_secs_f32();
                let smoothed = current_avg * (1.0 - alpha) + new_value * alpha;
                *avg = Duration::from_secs_f32(smoothed);
            }
        }

        // Update recent frame times (circular buffer)
        if self.recent_frame_times.len() >= self.max_recent_frames {
            self.recent_frame_times.remove(0);
        }
        self.recent_frame_times.push(breakdown.total_duration);
    }

    /// Returns the average FPS.
    pub fn avg_fps(&self) -> f64 {
        if self.avg_frame_time.as_secs_f64() > 0.0 {
            1.0 / self.avg_frame_time.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Returns the minimum FPS (from max frame time).
    pub fn min_fps(&self) -> f64 {
        if self.max_frame_time.as_secs_f64() > 0.0 {
            1.0 / self.max_frame_time.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Returns the maximum FPS (from min frame time).
    pub fn max_fps(&self) -> f64 {
        if self.min_frame_time.as_secs_f64() > 0.0 {
            1.0 / self.min_frame_time.as_secs_f64()
        } else {
            0.0
        }
    }
}

impl Default for FrameStatistics {
    fn default() -> Self {
        Self::new(300) // Track last 300 frames (~5 seconds at 60fps)
    }
}
