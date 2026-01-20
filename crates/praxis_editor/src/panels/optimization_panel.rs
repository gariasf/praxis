//! Rendering optimization configuration panel with performance comparison.

use super::EditorPanel;
use egui::{Color32, RichText, Ui};
use praxis_graphics::{RenderStats, RenderStatsHistory, RenderingOptimizationConfig};
use std::collections::VecDeque;

/// Preset optimization profiles for quick configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationPreset {
    /// Low quality - all optimizations disabled for debugging
    Low,
    /// Medium quality - basic optimizations enabled
    Medium,
    /// High quality - most optimizations enabled
    High,
    /// Ultra quality - all optimizations enabled
    Ultra,
    /// Custom preset
    Custom,
}

impl OptimizationPreset {
    /// Returns all available presets.
    pub fn all() -> [Self; 4] {
        [Self::Low, Self::Medium, Self::High, Self::Ultra]
    }

    /// Returns the preset name.
    pub fn name(&self) -> &str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Ultra => "Ultra",
            Self::Custom => "Custom",
        }
    }

    /// Applies the preset to the given configuration.
    pub fn apply_to(&self, config: &mut RenderingOptimizationConfig) {
        match self {
            Self::Low => {
                config.set_multi_draw_indirect(false);
                config.set_gpu_culling(false);
                config.set_gpu_lod(false);
                config.set_descriptor_caching(false);
                config.set_hiz_occlusion(false);
                config.set_mesh_streaming(false);
                config.set_backface_culling(false);
                config.set_small_object_culling(false);
                config.set_distance_culling(false);
            }
            Self::Medium => {
                config.set_multi_draw_indirect(true);
                config.set_gpu_culling(true);
                config.set_gpu_lod(false);
                config.set_descriptor_caching(true);
                config.set_hiz_occlusion(false);
                config.set_mesh_streaming(false);
                config.set_backface_culling(false);
                config.set_small_object_culling(false);
                config.set_distance_culling(false);
            }
            Self::High => {
                config.set_multi_draw_indirect(true);
                config.set_gpu_culling(true);
                config.set_gpu_lod(true);
                config.set_descriptor_caching(true);
                config.set_hiz_occlusion(true);
                config.set_mesh_streaming(false);
                config.set_backface_culling(true);
                config.set_small_object_culling(false);
                config.set_distance_culling(true);
            }
            Self::Ultra => {
                config.set_multi_draw_indirect(true);
                config.set_gpu_culling(true);
                config.set_gpu_lod(true);
                config.set_descriptor_caching(true);
                config.set_hiz_occlusion(true);
                config.set_mesh_streaming(true);
                config.set_backface_culling(true);
                config.set_small_object_culling(true);
                config.set_distance_culling(true);
            }
            Self::Custom => {}
        }
    }

    /// Detects the preset from the given configuration.
    pub fn detect_from(config: &RenderingOptimizationConfig) -> Self {
        for preset in Self::all() {
            let mut test_config = RenderingOptimizationConfig::default();
            preset.apply_to(&mut test_config);

            if config.multi_draw_indirect() == test_config.multi_draw_indirect()
                && config.gpu_culling() == test_config.gpu_culling()
                && config.gpu_lod() == test_config.gpu_lod()
                && config.descriptor_caching() == test_config.descriptor_caching()
                && config.hiz_occlusion() == test_config.hiz_occlusion()
                && config.mesh_streaming() == test_config.mesh_streaming()
                && config.backface_culling() == test_config.backface_culling()
                && config.small_object_culling() == test_config.small_object_culling()
                && config.distance_culling() == test_config.distance_culling()
            {
                return preset;
            }
        }
        Self::Custom
    }
}

/// Snapshot of render stats for before/after comparison.
#[derive(Debug, Clone)]
struct StatsSnapshot {
    /// Timestamp when snapshot was taken
    timestamp: std::time::Instant,
    /// Render statistics
    stats: RenderStats,
    /// Configuration state at the time
    config_summary: String,
}

/// Panel for configuring rendering optimizations with real-time performance comparison.
pub struct OptimizationPanel {
    title: String,
    /// Current optimization configuration (mutable reference held by panel)
    config_owned: Option<RenderingOptimizationConfig>,
    /// Current preset selection
    current_preset: OptimizationPreset,
    /// Stats history for the "before" state
    before_stats: Option<StatsSnapshot>,
    /// Stats history for the "after" state
    after_stats: Option<StatsSnapshot>,
    /// Whether comparison mode is active
    comparison_active: bool,
    /// Recent stats for live monitoring (last 60 frames)
    recent_stats: VecDeque<RenderStats>,
    /// Maximum frames to keep for live monitoring
    max_recent_frames: usize,
    /// Whether to show the performance graphs
    show_graphs: bool,
}

impl OptimizationPanel {
    /// Creates a new optimization panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: "Rendering Optimization".to_string(),
            config_owned: Some(RenderingOptimizationConfig::default()),
            current_preset: OptimizationPreset::detect_from(&RenderingOptimizationConfig::default()),
            before_stats: None,
            after_stats: None,
            comparison_active: false,
            recent_stats: VecDeque::with_capacity(60),
            max_recent_frames: 60,
            show_graphs: true,
        }
    }

    /// Updates the panel with new render stats.
    pub fn update_stats(&mut self, stats: RenderStats) {
        // Add to recent stats
        self.recent_stats.push_back(stats.clone());
        if self.recent_stats.len() > self.max_recent_frames {
            self.recent_stats.pop_front();
        }

        // Update after stats if comparison is active
        if self.comparison_active {
            if let Some(config) = &self.config_owned {
                self.after_stats = Some(StatsSnapshot {
                    timestamp: std::time::Instant::now(),
                    stats,
                    config_summary: config.summary(),
                });
            }
        }
    }

    /// Starts a new comparison by capturing the "before" state.
    fn start_comparison(&mut self) {
        if let Some(latest) = self.recent_stats.back().cloned() {
            if let Some(config) = &self.config_owned {
                self.before_stats = Some(StatsSnapshot {
                    timestamp: std::time::Instant::now(),
                    stats: latest,
                    config_summary: config.summary(),
                });
                self.after_stats = None;
                self.comparison_active = true;
            }
        }
    }

    /// Ends the comparison and captures the "after" state.
    fn end_comparison(&mut self) {
        if let Some(latest) = self.recent_stats.back().cloned() {
            if let Some(config) = &self.config_owned {
                self.after_stats = Some(StatsSnapshot {
                    timestamp: std::time::Instant::now(),
                    stats: latest,
                    config_summary: config.summary(),
                });
                self.comparison_active = false;
            }
        }
    }

    /// Renders the preset selector section.
    fn render_preset_selector(&mut self, ui: &mut Ui) {
        ui.heading("Optimization Presets");
        ui.separator();

        ui.horizontal(|ui| {
            for preset in OptimizationPreset::all() {
                let is_selected = self.current_preset == preset;
                let button_text = if is_selected {
                    RichText::new(preset.name()).color(Color32::WHITE)
                } else {
                    RichText::new(preset.name())
                };

                if ui.selectable_label(is_selected, button_text).clicked() {
                    self.current_preset = preset;
                    if let Some(config) = &mut self.config_owned {
                        preset.apply_to(config);
                    }
                }
            }
        });

        ui.add_space(5.0);

        // Show current preset description
        let description = match self.current_preset {
            OptimizationPreset::Low => {
                "All optimizations disabled. Use for debugging or baseline comparison."
            }
            OptimizationPreset::Medium => {
                "Basic optimizations: Multi-draw indirect, GPU culling, descriptor caching."
            }
            OptimizationPreset::High => {
                "Advanced optimizations: Adds GPU LOD, Hi-Z occlusion, backface/distance culling."
            }
            OptimizationPreset::Ultra => "All optimizations enabled. Maximum performance.",
            OptimizationPreset::Custom => "Custom configuration with manual toggles.",
        };

        ui.label(RichText::new(description).color(Color32::GRAY).small());
        ui.add_space(10.0);
    }

    /// Renders the individual optimization toggles section.
    fn render_optimization_toggles(&mut self, ui: &mut Ui) {
        if let Some(config) = &mut self.config_owned {
            ui.heading("Individual Optimizations");
            ui.separator();

            let mut changed = false;

            // Core optimizations
            ui.label(RichText::new("Core Optimizations").strong());
            changed |= self.render_toggle(
                ui,
                "Multi-Draw Indirect",
                config.multi_draw_indirect(),
                "Batch multiple draw calls into single indirect draw",
                |c, v| c.set_multi_draw_indirect(v),
            );
            changed |= self.render_toggle(
                ui,
                "GPU Culling",
                config.gpu_culling(),
                "Compute shader frustum and occlusion culling",
                |c, v| c.set_gpu_culling(v),
            );
            changed |= self.render_toggle(
                ui,
                "GPU LOD Selection",
                config.gpu_lod(),
                "GPU-driven level-of-detail selection",
                |c, v| c.set_gpu_lod(v),
            );
            changed |= self.render_toggle(
                ui,
                "Descriptor Caching",
                config.descriptor_caching(),
                "Reuse descriptor sets across frames",
                |c, v| c.set_descriptor_caching(v),
            );

            ui.add_space(5.0);

            // Advanced optimizations
            ui.label(RichText::new("Advanced Optimizations").strong());
            changed |= self.render_toggle(
                ui,
                "Hi-Z Occlusion",
                config.hiz_occlusion(),
                "Hierarchical Z-buffer occlusion culling",
                |c, v| c.set_hiz_occlusion(v),
            );
            changed |= self.render_toggle(
                ui,
                "Mesh Streaming",
                config.mesh_streaming(),
                "Background async loading of mesh data",
                |c, v| c.set_mesh_streaming(v),
            );

            ui.add_space(5.0);

            // Culling strategies
            ui.label(RichText::new("GPU Culling Strategies").strong());
            changed |= self.render_toggle(
                ui,
                "Backface Culling",
                config.backface_culling(),
                "Cull objects facing away from camera",
                |c, v| c.set_backface_culling(v),
            );
            changed |= self.render_toggle(
                ui,
                "Small Object Culling",
                config.small_object_culling(),
                "Cull objects below screen-space threshold",
                |c, v| c.set_small_object_culling(v),
            );
            changed |= self.render_toggle(
                ui,
                "Distance Culling",
                config.distance_culling(),
                "Cull objects beyond max render distance",
                |c, v| c.set_distance_culling(v),
            );

            if changed {
                // Detect if we've moved away from a preset
                self.current_preset = OptimizationPreset::detect_from(config);
            }

            ui.add_space(10.0);

            // Bulk operations
            ui.horizontal(|ui| {
                if ui.button("Enable All").clicked() {
                    config.enable_all();
                    self.current_preset = OptimizationPreset::Ultra;
                }
                if ui.button("Disable All").clicked() {
                    config.disable_all();
                    self.current_preset = OptimizationPreset::Low;
                }
                if ui.button("Reset to Default").clicked() {
                    config.reset_to_defaults();
                    self.current_preset =
                        OptimizationPreset::detect_from(&RenderingOptimizationConfig::default());
                }
            });

            ui.add_space(5.0);

            // Status
            let enabled_count = config.enabled_count();
            let total = RenderingOptimizationConfig::TOTAL_OPTIMIZATIONS;
            ui.label(format!("{enabled_count}/{total} optimizations enabled"));
        }
    }

    /// Renders a single optimization toggle with description.
    fn render_toggle<F>(
        &mut self,
        ui: &mut Ui,
        label: &str,
        mut value: bool,
        description: &str,
        setter: F,
    ) -> bool
    where
        F: FnOnce(&mut RenderingOptimizationConfig, bool),
    {
        let changed = ui.checkbox(&mut value, label).changed();
        ui.label(RichText::new(description).color(Color32::GRAY).small());
        ui.add_space(3.0);

        if changed {
            if let Some(config) = &mut self.config_owned {
                setter(config, value);
            }
        }

        changed
    }

    /// Renders the performance comparison section.
    fn render_performance_comparison(&mut self, ui: &mut Ui) {
        ui.heading("Performance Comparison");
        ui.separator();

        // Comparison controls
        ui.horizontal(|ui| {
            if !self.comparison_active {
                if ui.button("📸 Capture Before").clicked() {
                    self.start_comparison();
                }
            } else {
                if ui.button("✓ Capture After").clicked() {
                    self.end_comparison();
                }
                ui.label(RichText::new("⏳ Waiting for after capture...").color(Color32::YELLOW));
            }

            if self.before_stats.is_some() || self.after_stats.is_some() {
                if ui.button("Clear Comparison").clicked() {
                    self.before_stats = None;
                    self.after_stats = None;
                    self.comparison_active = false;
                }
            }
        });

        ui.add_space(10.0);

        // Show comparison results
        if let (Some(before), Some(after)) = (&self.before_stats, &self.after_stats) {
            self.render_comparison_results(ui, before, after);
        } else if let Some(before) = &self.before_stats {
            ui.label(
                RichText::new(
                    "Before state captured. Toggle optimizations and capture after state.",
                )
                .color(Color32::LIGHT_BLUE),
            );
            ui.add_space(5.0);
            self.render_snapshot_summary(ui, "Before", before);
        } else {
            ui.label(
                RichText::new("Capture before/after snapshots to compare optimization impact.")
                    .color(Color32::GRAY),
            );
        }
    }

    /// Renders the comparison results between before and after states.
    fn render_comparison_results(
        &self,
        ui: &mut Ui,
        before: &StatsSnapshot,
        after: &StatsSnapshot,
    ) {
        use egui::plot::{Line, Plot, PlotPoints};

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Before").strong().color(Color32::RED));
                self.render_snapshot_summary(ui, "", before);
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("After").strong().color(Color32::GREEN));
                self.render_snapshot_summary(ui, "", after);
            });
        });

        ui.add_space(10.0);
        ui.separator();
        ui.heading("Performance Delta");

        // Calculate improvements
        let draw_call_improvement =
            calculate_improvement(before.stats.draw_calls, after.stats.draw_calls);
        let visible_improvement =
            calculate_improvement(before.stats.visible_objects, after.stats.visible_objects);
        let culling_improvement =
            after.stats.culling_efficiency() - before.stats.culling_efficiency();

        // Display improvements
        self.render_metric_delta(
            ui,
            "Draw Calls",
            before.stats.draw_calls,
            after.stats.draw_calls,
            draw_call_improvement,
        );
        self.render_metric_delta(
            ui,
            "Visible Objects",
            before.stats.visible_objects,
            after.stats.visible_objects,
            visible_improvement,
        );
        ui.horizontal(|ui| {
            ui.label("Culling Efficiency:");
            ui.label(format!("{:.1}%", before.stats.culling_efficiency()));
            ui.label("→");
            ui.label(format!("{:.1}%", after.stats.culling_efficiency()));
            let color = if culling_improvement > 0.0 {
                Color32::GREEN
            } else if culling_improvement < 0.0 {
                Color32::RED
            } else {
                Color32::GRAY
            };
            ui.label(RichText::new(format!("({culling_improvement:+.1}%)")).color(color));
        });

        ui.add_space(10.0);

        // Show graph if enabled
        if self.show_graphs && !self.recent_stats.is_empty() {
            ui.checkbox(&mut self.show_graphs.clone(), "Show Live Graph");
            ui.add_space(5.0);

            Plot::new("live_stats")
                .height(150.0)
                .show_axes([true, true])
                .show(ui, |plot_ui| {
                    let draw_calls: PlotPoints = self
                        .recent_stats
                        .iter()
                        .enumerate()
                        .map(|(i, s)| [i as f64, s.draw_calls as f64])
                        .collect();
                    let visible: PlotPoints = self
                        .recent_stats
                        .iter()
                        .enumerate()
                        .map(|(i, s)| [i as f64, s.visible_objects as f64])
                        .collect();

                    plot_ui.line(
                        Line::new(draw_calls)
                            .name("Draw Calls")
                            .color(Color32::BLUE),
                    );
                    plot_ui.line(
                        Line::new(visible)
                            .name("Visible Objects")
                            .color(Color32::GREEN),
                    );
                });
        } else {
            ui.checkbox(&mut self.show_graphs.clone(), "Show Live Graph");
        }
    }

    /// Renders a metric delta comparison.
    fn render_metric_delta(
        &self,
        ui: &mut Ui,
        label: &str,
        before: usize,
        after: usize,
        improvement: f32,
    ) {
        ui.horizontal(|ui| {
            ui.label(format!("{label}:"));
            ui.label(before.to_string());
            ui.label("→");
            ui.label(after.to_string());

            let color = if improvement > 0.0 {
                Color32::GREEN
            } else if improvement < 0.0 {
                Color32::RED
            } else {
                Color32::GRAY
            };

            ui.label(RichText::new(format!("({improvement:+.1}%)")).color(color));
        });
    }

    /// Renders a snapshot summary.
    fn render_snapshot_summary(&self, ui: &mut Ui, prefix: &str, snapshot: &StatsSnapshot) {
        if !prefix.is_empty() {
            ui.label(RichText::new(prefix).strong());
        }
        ui.label(format!("Draw Calls: {}", snapshot.stats.draw_calls));
        ui.label(format!(
            "Visible Objects: {}",
            snapshot.stats.visible_objects
        ));
        ui.label(format!(
            "Culling Efficiency: {:.1}%",
            snapshot.stats.culling_efficiency()
        ));
        ui.label(format!(
            "Descriptor Allocations: {}",
            snapshot.stats.descriptor_allocations
        ));
    }

    /// Renders the live statistics section.
    fn render_live_stats(&self, ui: &mut Ui) {
        ui.heading("Live Statistics");
        ui.separator();

        if let Some(latest) = self.recent_stats.back() {
            ui.label(format!("Frame: {}", latest.frame_number));
            ui.label(format!("Total Objects: {}", latest.total_objects));
            ui.label(format!("Visible Objects: {}", latest.visible_objects));
            ui.label(format!("Frustum Culled: {}", latest.frustum_culled));
            ui.label(format!("Occlusion Culled: {}", latest.occlusion_culled));
            ui.label(format!("Draw Calls: {}", latest.draw_calls));
            ui.label(format!(
                "Descriptor Allocations: {}",
                latest.descriptor_allocations
            ));
            ui.label(format!(
                "Culling Efficiency: {:.1}%",
                latest.culling_efficiency()
            ));
            ui.label(format!("Streaming Queue: {}", latest.streaming_queue_depth));
        } else {
            ui.label(RichText::new("No statistics available yet.").color(Color32::GRAY));
        }
    }

    /// Gets the current optimization configuration.
    pub fn config(&self) -> Option<&RenderingOptimizationConfig> {
        self.config_owned.as_ref()
    }

    /// Gets a mutable reference to the optimization configuration.
    pub fn config_mut(&mut self) -> Option<&mut RenderingOptimizationConfig> {
        self.config_owned.as_mut()
    }

    /// Sets the optimization configuration.
    pub fn set_config(&mut self, config: RenderingOptimizationConfig) {
        self.current_preset = OptimizationPreset::detect_from(&config);
        self.config_owned = Some(config);
    }
}

impl Default for OptimizationPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for OptimizationPanel {
    fn title(&self) -> &str {
        &self.title
    }

    fn ui(
        &mut self,
        ui: &mut Ui,
        _world: Option<&praxis_ecs::World>,
        _render_context: Option<&mut praxis_graphics::RenderContext>,
    ) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Preset selector
            self.render_preset_selector(ui);

            ui.separator();
            ui.add_space(5.0);

            // Individual toggles
            self.render_optimization_toggles(ui);

            ui.separator();
            ui.add_space(5.0);

            // Live stats
            self.render_live_stats(ui);

            ui.separator();
            ui.add_space(5.0);

            // Performance comparison
            self.render_performance_comparison(ui);
        });
    }
}

/// Calculates the percentage improvement (negative means worse performance).
fn calculate_improvement(before: usize, after: usize) -> f32 {
    if before == 0 {
        return 0.0;
    }
    let diff = before as f32 - after as f32;
    (diff / before as f32) * 100.0
}
