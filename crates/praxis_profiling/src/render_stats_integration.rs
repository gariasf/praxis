//! Integration between rendering statistics and Chrome trace export.
//!
//! This module provides seamless integration between `praxis_graphics::RenderStats`
//! and the profiling system's Chrome trace export functionality. When enabled,
//! rendering metrics are automatically exported as counter events in the trace
//! timeline, allowing visualization of rendering performance alongside CPU/GPU
//! profiling data.
//!
//! # Exported Metrics
//!
//! The following rendering metrics are exported as counter events:
//!
//! ## Culling Metrics
//! - **Culling Efficiency %**: Percentage of objects successfully culled (0-100)
//! - **Total Objects**: Total number of objects submitted for rendering
//! - **Visible Objects**: Objects rendered after culling
//! - **Frustum Culled**: Objects culled by frustum test
//! - **Occlusion Culled**: Objects culled by occlusion test
//!
//! ## Draw Call Metrics
//! - **Draw Calls**: Number of draw calls issued to GPU
//! - **Draw Call Reduction**: Objects culled minus draw calls (shows batching efficiency)
//! - **Descriptor Allocations**: Descriptor sets allocated this frame
//!
//! ## LOD Metrics
//! - **LOD Level 0-N %**: Percentage of objects at each LOD level
//!
//! ## Streaming Metrics
//! - **Streaming Queue Depth**: Number of meshes in streaming queue
//!
//! # Chrome Trace Visualization
//!
//! These metrics appear in chrome://tracing as counter tracks, allowing you to:
//! - Correlate rendering performance with CPU/GPU work
//! - Identify frames with poor culling efficiency
//! - Track LOD system behavior over time
//! - Monitor streaming system load
//! - Analyze draw call batching effectiveness
//!
//! # Usage
//!
//! The integration is automatic when using `Profiler::record_render_stats()`:
//!
//! ```rust,ignore
//! use praxis_profiling::Profiler;
//! use std::time::Instant;
//!
//! let mut profiler = Profiler::new(ProfilerConfig::default());
//! profiler.begin_trace_export();
//!
//! // In render loop:
//! let render_stats = render_context.current_render_stats();
//! profiler.record_render_stats(&render_stats, Instant::now());
//!
//! // Later:
//! profiler.end_trace_export("trace.json")?;
//! ```

use praxis_graphics::RenderStats;

/// Helper functions for converting RenderStats to trace metrics.
///
/// This module is primarily internal - users should use `Profiler::record_render_stats()`
/// instead of calling these functions directly.
pub mod conversion {
    use super::*;

    /// Converts RenderStats into a list of counter values suitable for Chrome trace export.
    ///
    /// Returns a vector of (name, category, value) tuples that can be passed to
    /// `ChromeTraceExporter::add_counters()`.
    pub fn render_stats_to_counters(stats: &RenderStats) -> Vec<(String, String, f64)> {
        let mut counters = Vec::new();

        // Core culling metrics
        counters.push((
            "Culling Efficiency %".to_string(),
            "Rendering".to_string(),
            stats.culling_efficiency() as f64,
        ));

        // Object counts
        counters.push((
            "Total Objects".to_string(),
            "Rendering".to_string(),
            stats.total_objects as f64,
        ));

        counters.push((
            "Visible Objects".to_string(),
            "Rendering".to_string(),
            stats.visible_objects as f64,
        ));

        counters.push((
            "Frustum Culled".to_string(),
            "Rendering".to_string(),
            stats.frustum_culled as f64,
        ));

        counters.push((
            "Occlusion Culled".to_string(),
            "Rendering".to_string(),
            stats.occlusion_culled as f64,
        ));

        // Draw call metrics
        counters.push((
            "Draw Calls".to_string(),
            "Rendering".to_string(),
            stats.draw_calls as f64,
        ));

        let draw_call_reduction = stats.total_objects.saturating_sub(stats.draw_calls);
        counters.push((
            "Draw Call Reduction".to_string(),
            "Rendering".to_string(),
            draw_call_reduction as f64,
        ));

        // Descriptor allocations
        counters.push((
            "Descriptor Allocations".to_string(),
            "Rendering".to_string(),
            stats.descriptor_allocations as f64,
        ));

        // LOD distribution
        let lod_distribution = stats.lod_distribution_percentages();
        for (level, percentage) in lod_distribution {
            counters.push((
                format!("LOD Level {} %", level),
                "Rendering/LOD".to_string(),
                percentage as f64,
            ));
        }

        // Streaming metrics
        counters.push((
            "Streaming Queue Depth".to_string(),
            "Rendering".to_string(),
            stats.streaming_queue_depth as f64,
        ));

        // Memory metrics (if available)
        if let Some(ref mem) = stats.memory_snapshot {
            counters.push((
                "VRAM Total (MB)".to_string(),
                "Memory".to_string(),
                mem.total_mb(),
            ));

            counters.push((
                "VRAM Texture (MB)".to_string(),
                "Memory/Breakdown".to_string(),
                mem.category_mb(
                    praxis_graphics::utilities::memory_profiler::MemoryCategory::Texture,
                ),
            ));

            counters.push((
                "VRAM Mesh (MB)".to_string(),
                "Memory/Breakdown".to_string(),
                mem.category_mb(
                    praxis_graphics::utilities::memory_profiler::MemoryCategory::MeshBuffer,
                ),
            ));

            counters.push((
                "VRAM Descriptor (MB)".to_string(),
                "Memory/Breakdown".to_string(),
                mem.category_mb(
                    praxis_graphics::utilities::memory_profiler::MemoryCategory::DescriptorSet,
                ),
            ));

            counters.push((
                "VRAM Compute (MB)".to_string(),
                "Memory/Breakdown".to_string(),
                mem.category_mb(
                    praxis_graphics::utilities::memory_profiler::MemoryCategory::ComputeBuffer,
                ),
            ));

            counters.push((
                "VRAM Render Target (MB)".to_string(),
                "Memory/Breakdown".to_string(),
                mem.category_mb(
                    praxis_graphics::utilities::memory_profiler::MemoryCategory::RenderTarget,
                ),
            ));

            counters.push((
                "Memory Allocations".to_string(),
                "Memory".to_string(),
                mem.allocation_count as f64,
            ));
        }

        counters
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_stats_to_counters() {
        let stats = RenderStats {
            frame_number: 1,
            total_objects: 1000,
            visible_objects: 250,
            frustum_culled: 650,
            occlusion_culled: 100,
            draw_calls: 120,
            descriptor_allocations: 15,
            active_lod_levels: vec![(0, 50), (1, 150), (2, 50)],
            streaming_queue_depth: 5,
            memory_snapshot: None,
        };

        let counters = conversion::render_stats_to_counters(&stats);

        // Should have base counters (8) + LOD levels (3), no memory stats
        assert_eq!(counters.len(), 11);

        // Verify culling efficiency
        let culling_eff = counters
            .iter()
            .find(|(name, _, _)| name == "Culling Efficiency %")
            .unwrap();
        assert_eq!(culling_eff.2, 75.0);

        // Verify draw call reduction
        let draw_call_reduction = counters
            .iter()
            .find(|(name, _, _)| name == "Draw Call Reduction")
            .unwrap();
        assert_eq!(draw_call_reduction.2, 880.0);

        // Verify LOD distribution is included
        let lod_0 = counters
            .iter()
            .find(|(name, _, _)| name == "LOD Level 0 %")
            .unwrap();
        assert_eq!(lod_0.2, 20.0);
    }

    #[test]
    fn test_empty_lod_distribution() {
        let stats = RenderStats {
            frame_number: 1,
            total_objects: 100,
            visible_objects: 100,
            frustum_culled: 0,
            occlusion_culled: 0,
            draw_calls: 50,
            descriptor_allocations: 5,
            active_lod_levels: vec![],
            streaming_queue_depth: 0,
            memory_snapshot: None,
        };

        let counters = conversion::render_stats_to_counters(&stats);

        // Should have only base counters (8) when no LOD levels and no memory
        assert_eq!(counters.len(), 8);
    }
}
