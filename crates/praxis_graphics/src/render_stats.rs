//! Render statistics collection and visualization system.
//!
//! This module provides comprehensive tracking of per-frame rendering metrics including:
//! - Total objects submitted for rendering
//! - Visible objects after culling
//! - Objects culled by frustum
//! - Objects culled by occlusion
//! - Draw calls issued to GPU
//! - Descriptor set allocations
//! - Active LOD levels
//! - Mesh streaming queue depth
//!
//! # Architecture
//!
//! The statistics system consists of:
//! - **`RenderStats`**: Per-frame metrics snapshot
//! - **`RenderStatsHistory`**: Rolling history with statistical aggregation
//! - **`RenderStatsVisualizer`**: Graph and chart generation for GUI
//! - **`RenderStatsCsvExporter`**: Export statistics to CSV for analysis
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use praxis_graphics::render_stats::{RenderStats, RenderStatsHistory};
//!
//! // Create history tracker
//! let mut history = RenderStatsHistory::new(300); // Track 300 frames
//!
//! // Each frame, record stats
//! let stats = RenderStats {
//!     frame_number: 1,
//!     total_objects: 1000,
//!     visible_objects: 250,
//!     frustum_culled: 650,
//!     occlusion_culled: 100,
//!     draw_calls: 120,
//!     descriptor_allocations: 15,
//!     active_lod_levels: vec![(0, 50), (1, 150), (2, 50)],
//!     streaming_queue_depth: 5,
//! };
//!
//! history.record(stats);
//!
//! // Get statistics
//! println!("Average visible objects: {:.1}", history.avg_visible_objects());
//! println!("Peak draw calls: {}", history.max_draw_calls());
//! ```

use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Per-frame rendering statistics.
///
/// Captures a complete snapshot of rendering metrics for a single frame.
/// All counts are exact values measured during frame rendering.
#[derive(Debug, Clone)]
pub struct RenderStats {
    /// Frame number for temporal tracking
    pub frame_number: u64,

    /// Total number of objects submitted for rendering
    pub total_objects: usize,

    /// Number of objects actually rendered (after all culling)
    pub visible_objects: usize,

    /// Number of objects culled by frustum test
    pub frustum_culled: usize,

    /// Number of objects culled by occlusion test
    pub occlusion_culled: usize,

    /// Number of draw calls issued to GPU
    pub draw_calls: usize,

    /// Number of descriptor sets allocated this frame
    pub descriptor_allocations: usize,

    /// Active LOD levels (level index, object count)
    /// Example: [(0, 50), (1, 100), (2, 25)] means 50 objects at LOD0, 100 at LOD1, 25 at LOD2
    pub active_lod_levels: Vec<(usize, usize)>,

    /// Number of meshes in streaming queue
    pub streaming_queue_depth: usize,
}

impl RenderStats {
    /// Creates a new render stats snapshot with all values zeroed.
    pub fn new(frame_number: u64) -> Self {
        Self {
            frame_number,
            total_objects: 0,
            visible_objects: 0,
            frustum_culled: 0,
            occlusion_culled: 0,
            draw_calls: 0,
            descriptor_allocations: 0,
            active_lod_levels: Vec::new(),
            streaming_queue_depth: 0,
        }
    }

    /// Calculates the culling efficiency as a percentage.
    ///
    /// Returns the percentage of objects successfully culled.
    /// Higher values indicate better culling performance.
    pub fn culling_efficiency(&self) -> f32 {
        if self.total_objects == 0 {
            return 0.0;
        }
        let culled = self.frustum_culled + self.occlusion_culled;
        (culled as f32 / self.total_objects as f32) * 100.0
    }

    /// Returns the total number of objects culled by any method.
    pub fn total_culled(&self) -> usize {
        self.frustum_culled + self.occlusion_culled
    }

    /// Returns the visibility ratio as a percentage.
    ///
    /// Percentage of objects that were actually rendered.
    pub fn visibility_ratio(&self) -> f32 {
        if self.total_objects == 0 {
            return 0.0;
        }
        (self.visible_objects as f32 / self.total_objects as f32) * 100.0
    }

    /// Returns the total number of objects across all LOD levels.
    pub fn total_lod_objects(&self) -> usize {
        self.active_lod_levels.iter().map(|(_, count)| count).sum()
    }

    /// Returns the distribution of objects per LOD level as percentages.
    pub fn lod_distribution_percentages(&self) -> Vec<(usize, f32)> {
        let total = self.total_lod_objects();
        if total == 0 {
            return Vec::new();
        }

        self.active_lod_levels
            .iter()
            .map(|&(level, count)| (level, (count as f32 / total as f32) * 100.0))
            .collect()
    }
}

impl Default for RenderStats {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Rolling history of render statistics with aggregation and analysis.
///
/// Maintains a circular buffer of recent frame statistics and computes
/// statistical metrics (min, max, average) for analysis and visualization.
#[derive(Debug, Clone)]
pub struct RenderStatsHistory {
    /// Recent frame statistics (circular buffer)
    frames: VecDeque<RenderStats>,

    /// Maximum number of frames to track
    max_frames: usize,

    /// Running totals for average calculation
    total_objects_sum: usize,
    visible_objects_sum: usize,
    frustum_culled_sum: usize,
    occlusion_culled_sum: usize,
    draw_calls_sum: usize,
    descriptor_allocations_sum: usize,

    /// Peak values (for max statistics)
    peak_total_objects: usize,
    peak_visible_objects: usize,
    peak_draw_calls: usize,
    peak_descriptor_allocations: usize,
    peak_streaming_queue: usize,

    /// Minimum values
    min_draw_calls: usize,
    min_visible_objects: usize,
}

impl RenderStatsHistory {
    /// Creates a new render stats history tracker.
    ///
    /// # Arguments
    ///
    /// * `max_frames` - Maximum number of frames to track (older frames are discarded)
    ///
    /// # Recommended Values
    ///
    /// - **300 frames**: ~5 seconds at 60 FPS (good for recent history)
    /// - **1800 frames**: ~30 seconds at 60 FPS (good for trend analysis)
    /// - **18000 frames**: ~5 minutes at 60 FPS (good for long-term profiling)
    pub fn new(max_frames: usize) -> Self {
        Self {
            frames: VecDeque::with_capacity(max_frames),
            max_frames,
            total_objects_sum: 0,
            visible_objects_sum: 0,
            frustum_culled_sum: 0,
            occlusion_culled_sum: 0,
            draw_calls_sum: 0,
            descriptor_allocations_sum: 0,
            peak_total_objects: 0,
            peak_visible_objects: 0,
            peak_draw_calls: 0,
            peak_descriptor_allocations: 0,
            peak_streaming_queue: 0,
            min_draw_calls: usize::MAX,
            min_visible_objects: usize::MAX,
        }
    }

    /// Records a new frame's statistics.
    ///
    /// If the history is full, the oldest frame is discarded.
    /// Updates all running totals and peak values.
    pub fn record(&mut self, stats: RenderStats) {
        // Update peaks
        self.peak_total_objects = self.peak_total_objects.max(stats.total_objects);
        self.peak_visible_objects = self.peak_visible_objects.max(stats.visible_objects);
        self.peak_draw_calls = self.peak_draw_calls.max(stats.draw_calls);
        self.peak_descriptor_allocations = self
            .peak_descriptor_allocations
            .max(stats.descriptor_allocations);
        self.peak_streaming_queue = self.peak_streaming_queue.max(stats.streaming_queue_depth);

        // Update minimums
        if stats.draw_calls > 0 {
            self.min_draw_calls = self.min_draw_calls.min(stats.draw_calls);
        }
        if stats.visible_objects > 0 {
            self.min_visible_objects = self.min_visible_objects.min(stats.visible_objects);
        }

        // Update running sums
        self.total_objects_sum += stats.total_objects;
        self.visible_objects_sum += stats.visible_objects;
        self.frustum_culled_sum += stats.frustum_culled;
        self.occlusion_culled_sum += stats.occlusion_culled;
        self.draw_calls_sum += stats.draw_calls;
        self.descriptor_allocations_sum += stats.descriptor_allocations;

        // If buffer is full, subtract the oldest entry from sums
        if self.frames.len() >= self.max_frames {
            if let Some(old) = self.frames.pop_front() {
                self.total_objects_sum = self.total_objects_sum.saturating_sub(old.total_objects);
                self.visible_objects_sum =
                    self.visible_objects_sum.saturating_sub(old.visible_objects);
                self.frustum_culled_sum =
                    self.frustum_culled_sum.saturating_sub(old.frustum_culled);
                self.occlusion_culled_sum = self
                    .occlusion_culled_sum
                    .saturating_sub(old.occlusion_culled);
                self.draw_calls_sum = self.draw_calls_sum.saturating_sub(old.draw_calls);
                self.descriptor_allocations_sum = self
                    .descriptor_allocations_sum
                    .saturating_sub(old.descriptor_allocations);
            }
        }

        // Add new stats
        self.frames.push_back(stats);
    }

    /// Returns the number of frames currently tracked.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Returns true if no frames have been recorded.
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Clears all recorded statistics and resets aggregations.
    pub fn clear(&mut self) {
        self.frames.clear();
        self.total_objects_sum = 0;
        self.visible_objects_sum = 0;
        self.frustum_culled_sum = 0;
        self.occlusion_culled_sum = 0;
        self.draw_calls_sum = 0;
        self.descriptor_allocations_sum = 0;
        self.peak_total_objects = 0;
        self.peak_visible_objects = 0;
        self.peak_draw_calls = 0;
        self.peak_descriptor_allocations = 0;
        self.peak_streaming_queue = 0;
        self.min_draw_calls = usize::MAX;
        self.min_visible_objects = usize::MAX;
    }

    /// Returns the most recent frame's statistics.
    pub fn latest(&self) -> Option<&RenderStats> {
        self.frames.back()
    }

    /// Returns an iterator over all recorded frames (oldest to newest).
    pub fn iter(&self) -> impl Iterator<Item = &RenderStats> {
        self.frames.iter()
    }

    // === Average Statistics ===

    /// Returns the average number of total objects per frame.
    pub fn avg_total_objects(&self) -> f32 {
        if self.frames.is_empty() {
            return 0.0;
        }
        self.total_objects_sum as f32 / self.frames.len() as f32
    }

    /// Returns the average number of visible objects per frame.
    pub fn avg_visible_objects(&self) -> f32 {
        if self.frames.is_empty() {
            return 0.0;
        }
        self.visible_objects_sum as f32 / self.frames.len() as f32
    }

    /// Returns the average number of frustum culled objects per frame.
    pub fn avg_frustum_culled(&self) -> f32 {
        if self.frames.is_empty() {
            return 0.0;
        }
        self.frustum_culled_sum as f32 / self.frames.len() as f32
    }

    /// Returns the average number of occlusion culled objects per frame.
    pub fn avg_occlusion_culled(&self) -> f32 {
        if self.frames.is_empty() {
            return 0.0;
        }
        self.occlusion_culled_sum as f32 / self.frames.len() as f32
    }

    /// Returns the average number of draw calls per frame.
    pub fn avg_draw_calls(&self) -> f32 {
        if self.frames.is_empty() {
            return 0.0;
        }
        self.draw_calls_sum as f32 / self.frames.len() as f32
    }

    /// Returns the average number of descriptor allocations per frame.
    pub fn avg_descriptor_allocations(&self) -> f32 {
        if self.frames.is_empty() {
            return 0.0;
        }
        self.descriptor_allocations_sum as f32 / self.frames.len() as f32
    }

    /// Returns the average culling efficiency across all frames.
    pub fn avg_culling_efficiency(&self) -> f32 {
        if self.frames.is_empty() {
            return 0.0;
        }
        let avg_total = self.avg_total_objects();
        if avg_total == 0.0 {
            return 0.0;
        }
        let avg_culled = self.avg_frustum_culled() + self.avg_occlusion_culled();
        (avg_culled / avg_total) * 100.0
    }

    // === Peak Values ===

    /// Returns the maximum number of total objects in any frame.
    pub fn max_total_objects(&self) -> usize {
        self.peak_total_objects
    }

    /// Returns the maximum number of visible objects in any frame.
    pub fn max_visible_objects(&self) -> usize {
        self.peak_visible_objects
    }

    /// Returns the maximum number of draw calls in any frame.
    pub fn max_draw_calls(&self) -> usize {
        self.peak_draw_calls
    }

    /// Returns the maximum number of descriptor allocations in any frame.
    pub fn max_descriptor_allocations(&self) -> usize {
        self.peak_descriptor_allocations
    }

    /// Returns the maximum streaming queue depth in any frame.
    pub fn max_streaming_queue(&self) -> usize {
        self.peak_streaming_queue
    }

    // === Minimum Values ===

    /// Returns the minimum number of draw calls in any frame (excluding zero).
    pub fn min_draw_calls(&self) -> usize {
        if self.min_draw_calls == usize::MAX {
            0
        } else {
            self.min_draw_calls
        }
    }

    /// Returns the minimum number of visible objects in any frame (excluding zero).
    pub fn min_visible_objects(&self) -> usize {
        if self.min_visible_objects == usize::MAX {
            0
        } else {
            self.min_visible_objects
        }
    }

    // === Data Access for Visualization ===

    /// Returns visible objects history as a vector for graphing.
    pub fn visible_objects_history(&self) -> Vec<f32> {
        self.frames
            .iter()
            .map(|s| s.visible_objects as f32)
            .collect()
    }

    /// Returns draw calls history as a vector for graphing.
    pub fn draw_calls_history(&self) -> Vec<f32> {
        self.frames.iter().map(|s| s.draw_calls as f32).collect()
    }

    /// Returns frustum culled objects history as a vector for graphing.
    pub fn frustum_culled_history(&self) -> Vec<f32> {
        self.frames
            .iter()
            .map(|s| s.frustum_culled as f32)
            .collect()
    }

    /// Returns occlusion culled objects history as a vector for graphing.
    pub fn occlusion_culled_history(&self) -> Vec<f32> {
        self.frames
            .iter()
            .map(|s| s.occlusion_culled as f32)
            .collect()
    }

    /// Returns descriptor allocations history as a vector for graphing.
    pub fn descriptor_allocations_history(&self) -> Vec<f32> {
        self.frames
            .iter()
            .map(|s| s.descriptor_allocations as f32)
            .collect()
    }

    /// Returns streaming queue depth history as a vector for graphing.
    pub fn streaming_queue_history(&self) -> Vec<f32> {
        self.frames
            .iter()
            .map(|s| s.streaming_queue_depth as f32)
            .collect()
    }

    /// Returns culling efficiency history as a vector for graphing.
    pub fn culling_efficiency_history(&self) -> Vec<f32> {
        self.frames.iter().map(|s| s.culling_efficiency()).collect()
    }

    /// Exports statistics to CSV file.
    ///
    /// Creates a CSV file with one row per frame containing all metrics.
    /// Suitable for analysis in spreadsheet software or data science tools.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output CSV file
    ///
    /// # Errors
    ///
    /// Returns an error if file creation or writing fails.
    ///
    /// # CSV Format
    ///
    /// ```csv
    /// frame_number,total_objects,visible_objects,frustum_culled,occlusion_culled,draw_calls,descriptor_allocations,streaming_queue_depth,culling_efficiency
    /// 1,1000,250,650,100,120,15,5,75.0
    /// 2,1000,245,655,100,118,15,4,75.5
    /// ```
    pub fn export_to_csv<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;

        // Write CSV header
        writeln!(
            file,
            "frame_number,total_objects,visible_objects,frustum_culled,occlusion_culled,draw_calls,descriptor_allocations,streaming_queue_depth,culling_efficiency"
        )?;

        // Write data rows
        for stats in &self.frames {
            writeln!(
                file,
                "{},{},{},{},{},{},{},{},{:.2}",
                stats.frame_number,
                stats.total_objects,
                stats.visible_objects,
                stats.frustum_culled,
                stats.occlusion_culled,
                stats.draw_calls,
                stats.descriptor_allocations,
                stats.streaming_queue_depth,
                stats.culling_efficiency()
            )?;
        }

        Ok(())
    }
}

impl Default for RenderStatsHistory {
    fn default() -> Self {
        Self::new(300) // Default to tracking 300 frames (~5 seconds at 60 FPS)
    }
}

/// Visualization data for rendering statistics graphs.
///
/// Provides structured data for GUI visualization of rendering metrics.
/// Compatible with egui plotting widgets and custom chart renderers.
#[derive(Debug, Clone)]
pub struct RenderStatsVisualizer {
    /// Graph of visible objects over time
    pub visible_objects_graph: Vec<f32>,

    /// Graph of draw calls over time
    pub draw_calls_graph: Vec<f32>,

    /// Graph of culling efficiency over time
    pub culling_efficiency_graph: Vec<f32>,

    /// Graph of descriptor allocations over time
    pub descriptor_allocations_graph: Vec<f32>,

    /// Graph of streaming queue depth over time
    pub streaming_queue_graph: Vec<f32>,

    /// Stacked area chart data for culling breakdown
    pub culling_breakdown: CullingBreakdown,

    /// Statistical summary
    pub summary: StatsSummary,
}

/// Culling breakdown for stacked area chart visualization.
#[derive(Debug, Clone)]
pub struct CullingBreakdown {
    /// Frame indices
    pub frames: Vec<usize>,

    /// Frustum culled objects per frame
    pub frustum_culled: Vec<f32>,

    /// Occlusion culled objects per frame
    pub occlusion_culled: Vec<f32>,

    /// Visible objects per frame
    pub visible: Vec<f32>,
}

/// Statistical summary for display.
#[derive(Debug, Clone)]
pub struct StatsSummary {
    /// Average visible objects
    pub avg_visible: f32,

    /// Peak visible objects
    pub peak_visible: usize,

    /// Average draw calls
    pub avg_draw_calls: f32,

    /// Peak draw calls
    pub peak_draw_calls: usize,

    /// Average culling efficiency percentage
    pub avg_culling_efficiency: f32,

    /// Average descriptor allocations
    pub avg_descriptor_allocations: f32,

    /// Peak descriptor allocations
    pub peak_descriptor_allocations: usize,
}

impl RenderStatsVisualizer {
    /// Creates visualization data from a stats history.
    ///
    /// Extracts and formats statistics for GUI rendering.
    pub fn from_history(history: &RenderStatsHistory) -> Self {
        let frames: Vec<usize> = (0..history.frame_count()).collect();

        Self {
            visible_objects_graph: history.visible_objects_history(),
            draw_calls_graph: history.draw_calls_history(),
            culling_efficiency_graph: history.culling_efficiency_history(),
            descriptor_allocations_graph: history.descriptor_allocations_history(),
            streaming_queue_graph: history.streaming_queue_history(),
            culling_breakdown: CullingBreakdown {
                frames,
                frustum_culled: history.frustum_culled_history(),
                occlusion_culled: history.occlusion_culled_history(),
                visible: history.visible_objects_history(),
            },
            summary: StatsSummary {
                avg_visible: history.avg_visible_objects(),
                peak_visible: history.max_visible_objects(),
                avg_draw_calls: history.avg_draw_calls(),
                peak_draw_calls: history.max_draw_calls(),
                avg_culling_efficiency: history.avg_culling_efficiency(),
                avg_descriptor_allocations: history.avg_descriptor_allocations(),
                peak_descriptor_allocations: history.max_descriptor_allocations(),
            },
        }
    }

    /// Renders the visualization using egui (if available).
    ///
    /// This method is designed to be called from an egui UI context.
    /// It renders all graphs and statistics in a formatted panel.
    #[cfg(feature = "egui")]
    pub fn render_ui(&self, ui: &mut egui::Ui) {
        use egui::plot::{Line, Plot};

        ui.heading("Render Statistics");

        ui.separator();

        // Summary statistics
        ui.label(format!(
            "Average Visible Objects: {:.1} (Peak: {})",
            self.summary.avg_visible, self.summary.peak_visible
        ));
        ui.label(format!(
            "Average Draw Calls: {:.1} (Peak: {})",
            self.summary.avg_draw_calls, self.summary.peak_draw_calls
        ));
        ui.label(format!(
            "Average Culling Efficiency: {:.1}%",
            self.summary.avg_culling_efficiency
        ));
        ui.label(format!(
            "Average Descriptor Allocations: {:.1} (Peak: {})",
            self.summary.avg_descriptor_allocations, self.summary.peak_descriptor_allocations
        ));

        ui.separator();

        // Visible objects graph
        ui.heading("Visible Objects");
        Plot::new("visible_objects_plot")
            .height(150.0)
            .show(ui, |plot_ui| {
                let points: Vec<[f64; 2]> = self
                    .visible_objects_graph
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| [i as f64, v as f64])
                    .collect();
                plot_ui.line(Line::new(points).name("Visible Objects"));
            });

        // Draw calls graph
        ui.heading("Draw Calls");
        Plot::new("draw_calls_plot")
            .height(150.0)
            .show(ui, |plot_ui| {
                let points: Vec<[f64; 2]> = self
                    .draw_calls_graph
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| [i as f64, v as f64])
                    .collect();
                plot_ui.line(Line::new(points).name("Draw Calls"));
            });

        // Culling efficiency graph
        ui.heading("Culling Efficiency");
        Plot::new("culling_efficiency_plot")
            .height(150.0)
            .show(ui, |plot_ui| {
                let points: Vec<[f64; 2]> = self
                    .culling_efficiency_graph
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| [i as f64, v as f64])
                    .collect();
                plot_ui.line(Line::new(points).name("Culling Efficiency %"));
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_stats_creation() {
        let stats = RenderStats::new(42);
        assert_eq!(stats.frame_number, 42);
        assert_eq!(stats.total_objects, 0);
        assert_eq!(stats.culling_efficiency(), 0.0);
    }

    #[test]
    fn test_culling_efficiency() {
        let stats = RenderStats {
            frame_number: 1,
            total_objects: 1000,
            visible_objects: 250,
            frustum_culled: 650,
            occlusion_culled: 100,
            draw_calls: 120,
            descriptor_allocations: 15,
            active_lod_levels: vec![],
            streaming_queue_depth: 0,
        };

        assert_eq!(stats.culling_efficiency(), 75.0);
        assert_eq!(stats.visibility_ratio(), 25.0);
    }

    #[test]
    fn test_history_recording() {
        let mut history = RenderStatsHistory::new(3);

        let stats1 = RenderStats {
            frame_number: 1,
            total_objects: 100,
            visible_objects: 50,
            frustum_culled: 40,
            occlusion_culled: 10,
            draw_calls: 25,
            descriptor_allocations: 5,
            active_lod_levels: vec![],
            streaming_queue_depth: 2,
        };

        history.record(stats1.clone());
        assert_eq!(history.frame_count(), 1);
        assert_eq!(history.avg_visible_objects(), 50.0);

        let stats2 = RenderStats {
            frame_number: 2,
            total_objects: 100,
            visible_objects: 60,
            frustum_culled: 30,
            occlusion_culled: 10,
            draw_calls: 30,
            descriptor_allocations: 6,
            active_lod_levels: vec![],
            streaming_queue_depth: 3,
        };

        history.record(stats2);
        assert_eq!(history.frame_count(), 2);
        assert_eq!(history.avg_visible_objects(), 55.0);
    }

    #[test]
    fn test_history_circular_buffer() {
        let mut history = RenderStatsHistory::new(2);

        for i in 1..=5 {
            let stats = RenderStats {
                frame_number: i,
                total_objects: 100,
                visible_objects: i as usize * 10,
                frustum_culled: 0,
                occlusion_culled: 0,
                draw_calls: 10,
                descriptor_allocations: 1,
                active_lod_levels: vec![],
                streaming_queue_depth: 0,
            };
            history.record(stats);
        }

        // Should only keep last 2 frames
        assert_eq!(history.frame_count(), 2);

        // Should have frames 4 and 5 (40 and 50 visible objects)
        assert_eq!(history.avg_visible_objects(), 45.0);
    }

    #[test]
    fn test_lod_distribution() {
        let stats = RenderStats {
            frame_number: 1,
            total_objects: 100,
            visible_objects: 100,
            frustum_culled: 0,
            occlusion_culled: 0,
            draw_calls: 50,
            descriptor_allocations: 5,
            active_lod_levels: vec![(0, 20), (1, 50), (2, 30)],
            streaming_queue_depth: 0,
        };

        assert_eq!(stats.total_lod_objects(), 100);

        let dist = stats.lod_distribution_percentages();
        assert_eq!(dist.len(), 3);
        assert_eq!(dist[0], (0, 20.0));
        assert_eq!(dist[1], (1, 50.0));
        assert_eq!(dist[2], (2, 30.0));
    }
}
