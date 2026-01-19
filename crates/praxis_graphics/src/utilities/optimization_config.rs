//! Rendering optimization configuration system with runtime toggles.
//!
//! This module provides a centralized configuration system for enabling/disabling
//! rendering optimizations at runtime, allowing A/B performance comparison and
//! debugging of individual optimization techniques.
//!
//! # Supported Optimizations
//!
//! - **Multi-Draw Indirect**: Batch multiple draw calls into single indirect draw
//! - **GPU Culling**: Compute shader-based frustum and occlusion culling
//! - **GPU LOD Selection**: GPU-driven level-of-detail selection
//! - **Descriptor Caching**: Reuse descriptor sets across frames
//! - **Hi-Z Occlusion**: Hierarchical Z-buffer occlusion culling
//! - **Mesh Streaming**: Background async loading of mesh data
//!
//! # GUI Integration
//!
//! The configuration includes a built-in GUI panel for toggling optimizations:
//!
//! ```rust,ignore
//! use praxis_graphics::optimization_config::RenderingOptimizationConfig;
//!
//! let mut config = RenderingOptimizationConfig::default();
//!
//! // In GUI rendering code:
//! config.show_gui(ui);
//! ```
//!
//! # Key Bindings
//!
//! Each optimization can be toggled via keyboard:
//!
//! - `F1`: Multi-Draw Indirect
//! - `F2`: GPU Culling
//! - `F3`: GPU LOD Selection
//! - `F4`: Descriptor Caching
//! - `F5`: Hi-Z Occlusion
//! - `F6`: Mesh Streaming
//! - `F7`: Toggle entire panel visibility
//! - `F8`: Reset all to defaults
//!
//! # Performance Comparison
//!
//! The config tracks when settings change, allowing measurement of performance impact:
//!
//! ```rust,ignore
//! if config.has_changed() {
//!     println!("Optimization settings changed");
//!     // Reset performance counters for clean A/B comparison
//! }
//! ```
//!
//! # Example
//!
//! ```rust
//! use praxis_graphics::optimization_config::RenderingOptimizationConfig;
//!
//! let mut config = RenderingOptimizationConfig::default();
//!
//! // Check if optimizations are enabled
//! if config.multi_draw_indirect() {
//!     // Use multi-draw indirect rendering
//! } else {
//!     // Use individual draw calls
//! }
//!
//! // Manually toggle
//! config.set_gpu_culling(true);
//!
//! // Enable all optimizations
//! config.enable_all();
//!
//! // Disable all optimizations
//! config.disable_all();
//! ```

use praxis_utils::{debug, info};
use serde::{Deserialize, Serialize};

/// Configuration for rendering optimizations with runtime toggles.
///
/// This structure provides centralized control over various rendering optimizations,
/// allowing them to be enabled/disabled at runtime for performance comparison and debugging.
///
/// # Change Tracking
///
/// The config tracks when settings change via the `changed` flag, which is set to
/// `true` whenever any optimization is toggled. This allows the application to detect
/// when to reset performance metrics for accurate A/B testing.
///
/// # Persistence
///
/// The config implements `Serialize` and `Deserialize` for easy saving/loading of
/// optimization profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingOptimizationConfig {
    /// Enable multi-draw indirect for batching multiple draw calls.
    multi_draw_indirect: bool,

    /// Enable GPU culling for frustum and occlusion testing.
    gpu_culling: bool,

    /// Enable GPU-driven LOD selection.
    gpu_lod: bool,

    /// Enable descriptor set caching and reuse.
    descriptor_caching: bool,

    /// Enable Hi-Z occlusion culling.
    hiz_occlusion: bool,

    /// Enable background mesh streaming.
    mesh_streaming: bool,

    /// Flag indicating if any setting has changed since last reset.
    #[serde(skip)]
    changed: bool,

    /// Show the GUI panel for optimization toggles.
    #[serde(skip)]
    show_panel: bool,
}

impl Default for RenderingOptimizationConfig {
    fn default() -> Self {
        Self {
            multi_draw_indirect: true,
            gpu_culling: true,
            gpu_lod: true,
            descriptor_caching: true,
            hiz_occlusion: false,  // Disabled by default (requires setup)
            mesh_streaming: false, // Disabled by default (requires setup)
            changed: false,
            show_panel: true,
        }
    }
}

impl RenderingOptimizationConfig {
    /// Creates a new config with all optimizations enabled.
    pub fn all_enabled() -> Self {
        Self {
            multi_draw_indirect: true,
            gpu_culling: true,
            gpu_lod: true,
            descriptor_caching: true,
            hiz_occlusion: true,
            mesh_streaming: true,
            changed: false,
            show_panel: true,
        }
    }

    /// Creates a new config with all optimizations disabled.
    pub fn all_disabled() -> Self {
        Self {
            multi_draw_indirect: false,
            gpu_culling: false,
            gpu_lod: false,
            descriptor_caching: false,
            hiz_occlusion: false,
            mesh_streaming: false,
            changed: false,
            show_panel: true,
        }
    }

    /// Checks if multi-draw indirect is enabled.
    pub fn multi_draw_indirect(&self) -> bool {
        self.multi_draw_indirect
    }

    /// Checks if GPU culling is enabled.
    pub fn gpu_culling(&self) -> bool {
        self.gpu_culling
    }

    /// Checks if GPU LOD selection is enabled.
    pub fn gpu_lod(&self) -> bool {
        self.gpu_lod
    }

    /// Checks if descriptor caching is enabled.
    pub fn descriptor_caching(&self) -> bool {
        self.descriptor_caching
    }

    /// Checks if Hi-Z occlusion is enabled.
    pub fn hiz_occlusion(&self) -> bool {
        self.hiz_occlusion
    }

    /// Checks if mesh streaming is enabled.
    pub fn mesh_streaming(&self) -> bool {
        self.mesh_streaming
    }

    /// Sets multi-draw indirect enabled/disabled.
    pub fn set_multi_draw_indirect(&mut self, enabled: bool) {
        if self.multi_draw_indirect != enabled {
            self.multi_draw_indirect = enabled;
            self.changed = true;
            info!(
                "Multi-draw indirect: {}",
                if enabled { "enabled" } else { "disabled" }
            );
        }
    }

    /// Sets GPU culling enabled/disabled.
    pub fn set_gpu_culling(&mut self, enabled: bool) {
        if self.gpu_culling != enabled {
            self.gpu_culling = enabled;
            self.changed = true;
            info!(
                "GPU culling: {}",
                if enabled { "enabled" } else { "disabled" }
            );
        }
    }

    /// Sets GPU LOD selection enabled/disabled.
    pub fn set_gpu_lod(&mut self, enabled: bool) {
        if self.gpu_lod != enabled {
            self.gpu_lod = enabled;
            self.changed = true;
            info!(
                "GPU LOD selection: {}",
                if enabled { "enabled" } else { "disabled" }
            );
        }
    }

    /// Sets descriptor caching enabled/disabled.
    pub fn set_descriptor_caching(&mut self, enabled: bool) {
        if self.descriptor_caching != enabled {
            self.descriptor_caching = enabled;
            self.changed = true;
            info!(
                "Descriptor caching: {}",
                if enabled { "enabled" } else { "disabled" }
            );
        }
    }

    /// Sets Hi-Z occlusion enabled/disabled.
    pub fn set_hiz_occlusion(&mut self, enabled: bool) {
        if self.hiz_occlusion != enabled {
            self.hiz_occlusion = enabled;
            self.changed = true;
            info!(
                "Hi-Z occlusion: {}",
                if enabled { "enabled" } else { "disabled" }
            );
        }
    }

    /// Sets mesh streaming enabled/disabled.
    pub fn set_mesh_streaming(&mut self, enabled: bool) {
        if self.mesh_streaming != enabled {
            self.mesh_streaming = enabled;
            self.changed = true;
            info!(
                "Mesh streaming: {}",
                if enabled { "enabled" } else { "disabled" }
            );
        }
    }

    /// Enables all optimizations.
    pub fn enable_all(&mut self) {
        let old_state = (
            self.multi_draw_indirect,
            self.gpu_culling,
            self.gpu_lod,
            self.descriptor_caching,
            self.hiz_occlusion,
            self.mesh_streaming,
        );

        self.multi_draw_indirect = true;
        self.gpu_culling = true;
        self.gpu_lod = true;
        self.descriptor_caching = true;
        self.hiz_occlusion = true;
        self.mesh_streaming = true;

        let new_state = (
            self.multi_draw_indirect,
            self.gpu_culling,
            self.gpu_lod,
            self.descriptor_caching,
            self.hiz_occlusion,
            self.mesh_streaming,
        );

        if old_state != new_state {
            self.changed = true;
            info!("All rendering optimizations enabled");
        }
    }

    /// Disables all optimizations.
    pub fn disable_all(&mut self) {
        let old_state = (
            self.multi_draw_indirect,
            self.gpu_culling,
            self.gpu_lod,
            self.descriptor_caching,
            self.hiz_occlusion,
            self.mesh_streaming,
        );

        self.multi_draw_indirect = false;
        self.gpu_culling = false;
        self.gpu_lod = false;
        self.descriptor_caching = false;
        self.hiz_occlusion = false;
        self.mesh_streaming = false;

        let new_state = (
            self.multi_draw_indirect,
            self.gpu_culling,
            self.gpu_lod,
            self.descriptor_caching,
            self.hiz_occlusion,
            self.mesh_streaming,
        );

        if old_state != new_state {
            self.changed = true;
            info!("All rendering optimizations disabled");
        }
    }

    /// Resets to default settings.
    pub fn reset_to_defaults(&mut self) {
        let defaults = Self::default();
        *self = defaults;
        self.changed = true;
        info!("Optimization settings reset to defaults");
    }

    /// Checks if any settings have changed since last reset.
    pub fn has_changed(&self) -> bool {
        self.changed
    }

    /// Resets the changed flag.
    ///
    /// Call this after handling the change (e.g., after resetting performance counters).
    pub fn clear_changed_flag(&mut self) {
        self.changed = false;
    }

    /// Shows or hides the GUI panel.
    pub fn set_show_panel(&mut self, show: bool) {
        self.show_panel = show;
    }

    /// Checks if the GUI panel is visible.
    pub fn is_panel_visible(&self) -> bool {
        self.show_panel
    }

    /// Toggles the GUI panel visibility.
    pub fn toggle_panel(&mut self) {
        self.show_panel = !self.show_panel;
        debug!(
            "Optimization panel: {}",
            if self.show_panel { "visible" } else { "hidden" }
        );
    }

    /// Renders the GUI panel for optimization toggles.
    ///
    /// This method should be called within an egui rendering context.
    ///
    /// # Arguments
    ///
    /// * `ctx` - egui context for rendering
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use egui::Context;
    ///
    /// fn render_gui(ctx: &Context, config: &mut RenderingOptimizationConfig) {
    ///     config.show_gui(ctx);
    /// }
    /// ```
    #[cfg(feature = "gui")]
    pub fn show_gui(&mut self, ctx: &egui::Context) {
        use egui::{Color32, RichText};

        if !self.show_panel {
            return;
        }

        egui::Window::new("Rendering Optimizations")
            .default_pos([10.0, 10.0])
            .default_width(320.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Optimization Toggles");
                ui.separator();

                // Multi-draw indirect
                ui.horizontal(|ui| {
                    let mut enabled = self.multi_draw_indirect;
                    if ui.checkbox(&mut enabled, "Multi-Draw Indirect").changed() {
                        self.set_multi_draw_indirect(enabled);
                    }
                    ui.label(RichText::new("(F1)").color(Color32::GRAY).small());
                });
                ui.label("Batch multiple draw calls into single indirect draw");
                ui.add_space(5.0);

                // GPU culling
                ui.horizontal(|ui| {
                    let mut enabled = self.gpu_culling;
                    if ui.checkbox(&mut enabled, "GPU Culling").changed() {
                        self.set_gpu_culling(enabled);
                    }
                    ui.label(RichText::new("(F2)").color(Color32::GRAY).small());
                });
                ui.label("Compute shader frustum and occlusion culling");
                ui.add_space(5.0);

                // GPU LOD
                ui.horizontal(|ui| {
                    let mut enabled = self.gpu_lod;
                    if ui.checkbox(&mut enabled, "GPU LOD Selection").changed() {
                        self.set_gpu_lod(enabled);
                    }
                    ui.label(RichText::new("(F3)").color(Color32::GRAY).small());
                });
                ui.label("GPU-driven level-of-detail selection");
                ui.add_space(5.0);

                // Descriptor caching
                ui.horizontal(|ui| {
                    let mut enabled = self.descriptor_caching;
                    if ui.checkbox(&mut enabled, "Descriptor Caching").changed() {
                        self.set_descriptor_caching(enabled);
                    }
                    ui.label(RichText::new("(F4)").color(Color32::GRAY).small());
                });
                ui.label("Reuse descriptor sets across frames");
                ui.add_space(5.0);

                // Hi-Z occlusion
                ui.horizontal(|ui| {
                    let mut enabled = self.hiz_occlusion;
                    if ui.checkbox(&mut enabled, "Hi-Z Occlusion").changed() {
                        self.set_hiz_occlusion(enabled);
                    }
                    ui.label(RichText::new("(F5)").color(Color32::GRAY).small());
                });
                ui.label("Hierarchical Z-buffer occlusion culling");
                ui.add_space(5.0);

                // Mesh streaming
                ui.horizontal(|ui| {
                    let mut enabled = self.mesh_streaming;
                    if ui.checkbox(&mut enabled, "Mesh Streaming").changed() {
                        self.set_mesh_streaming(enabled);
                    }
                    ui.label(RichText::new("(F6)").color(Color32::GRAY).small());
                });
                ui.label("Background async loading of mesh data");
                ui.add_space(10.0);

                ui.separator();

                // Bulk operations
                ui.horizontal(|ui| {
                    if ui.button("Enable All").clicked() {
                        self.enable_all();
                    }
                    if ui.button("Disable All").clicked() {
                        self.disable_all();
                    }
                    if ui.button("Reset").clicked() {
                        self.reset_to_defaults();
                    }
                });

                ui.add_space(5.0);

                // Status indicator
                if self.has_changed() {
                    ui.label(
                        RichText::new("⚠ Settings changed - metrics may be affected")
                            .color(Color32::YELLOW)
                            .small(),
                    );
                }

                ui.add_space(5.0);
                ui.label(
                    RichText::new("Press F7 to hide/show this panel")
                        .color(Color32::GRAY)
                        .small(),
                );
                ui.label(
                    RichText::new("Press F8 to reset all settings")
                        .color(Color32::GRAY)
                        .small(),
                );
            });
    }

    /// Handles keyboard input for toggling optimizations.
    ///
    /// This method should be called each frame to process key presses.
    ///
    /// # Supported Keys
    ///
    /// - `F1`: Toggle Multi-Draw Indirect
    /// - `F2`: Toggle GPU Culling
    /// - `F3`: Toggle GPU LOD
    /// - `F4`: Toggle Descriptor Caching
    /// - `F5`: Toggle Hi-Z Occlusion
    /// - `F6`: Toggle Mesh Streaming
    /// - `F7`: Toggle Panel Visibility
    /// - `F8`: Reset to Defaults
    ///
    /// # Arguments
    ///
    /// * `ctx` - egui context for checking key states
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use egui::Context;
    ///
    /// fn handle_input(ctx: &Context, config: &mut RenderingOptimizationConfig) {
    ///     config.handle_keyboard_input(ctx);
    /// }
    /// ```
    #[cfg(feature = "gui")]
    pub fn handle_keyboard_input(&mut self, ctx: &egui::Context) {
        use egui::Key;

        // Check for key presses
        if ctx.input(|i| i.key_pressed(Key::F1)) {
            self.set_multi_draw_indirect(!self.multi_draw_indirect);
        }
        if ctx.input(|i| i.key_pressed(Key::F2)) {
            self.set_gpu_culling(!self.gpu_culling);
        }
        if ctx.input(|i| i.key_pressed(Key::F3)) {
            self.set_gpu_lod(!self.gpu_lod);
        }
        if ctx.input(|i| i.key_pressed(Key::F4)) {
            self.set_descriptor_caching(!self.descriptor_caching);
        }
        if ctx.input(|i| i.key_pressed(Key::F5)) {
            self.set_hiz_occlusion(!self.hiz_occlusion);
        }
        if ctx.input(|i| i.key_pressed(Key::F6)) {
            self.set_mesh_streaming(!self.mesh_streaming);
        }
        if ctx.input(|i| i.key_pressed(Key::F7)) {
            self.toggle_panel();
        }
        if ctx.input(|i| i.key_pressed(Key::F8)) {
            self.reset_to_defaults();
        }
    }

    /// Gets a summary of all optimization states.
    ///
    /// # Returns
    ///
    /// A string containing a human-readable summary of all optimizations and their states.
    pub fn summary(&self) -> String {
        format!(
            "Rendering Optimizations:\n\
             - Multi-Draw Indirect: {}\n\
             - GPU Culling: {}\n\
             - GPU LOD Selection: {}\n\
             - Descriptor Caching: {}\n\
             - Hi-Z Occlusion: {}\n\
             - Mesh Streaming: {}",
            if self.multi_draw_indirect {
                "enabled"
            } else {
                "disabled"
            },
            if self.gpu_culling {
                "enabled"
            } else {
                "disabled"
            },
            if self.gpu_lod { "enabled" } else { "disabled" },
            if self.descriptor_caching {
                "enabled"
            } else {
                "disabled"
            },
            if self.hiz_occlusion {
                "enabled"
            } else {
                "disabled"
            },
            if self.mesh_streaming {
                "enabled"
            } else {
                "disabled"
            },
        )
    }

    /// Counts how many optimizations are currently enabled.
    pub fn enabled_count(&self) -> usize {
        let mut count = 0;
        if self.multi_draw_indirect {
            count += 1;
        }
        if self.gpu_culling {
            count += 1;
        }
        if self.gpu_lod {
            count += 1;
        }
        if self.descriptor_caching {
            count += 1;
        }
        if self.hiz_occlusion {
            count += 1;
        }
        if self.mesh_streaming {
            count += 1;
        }
        count
    }

    /// Total number of available optimizations.
    pub const TOTAL_OPTIMIZATIONS: usize = 6;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RenderingOptimizationConfig::default();
        assert!(config.multi_draw_indirect());
        assert!(config.gpu_culling());
        assert!(config.gpu_lod());
        assert!(config.descriptor_caching());
        assert!(!config.hiz_occlusion()); // Disabled by default
        assert!(!config.mesh_streaming()); // Disabled by default
        assert!(!config.has_changed());
        assert!(config.is_panel_visible());
    }

    #[test]
    fn test_all_enabled() {
        let config = RenderingOptimizationConfig::all_enabled();
        assert!(config.multi_draw_indirect());
        assert!(config.gpu_culling());
        assert!(config.gpu_lod());
        assert!(config.descriptor_caching());
        assert!(config.hiz_occlusion());
        assert!(config.mesh_streaming());
        assert_eq!(
            config.enabled_count(),
            RenderingOptimizationConfig::TOTAL_OPTIMIZATIONS
        );
    }

    #[test]
    fn test_all_disabled() {
        let config = RenderingOptimizationConfig::all_disabled();
        assert!(!config.multi_draw_indirect());
        assert!(!config.gpu_culling());
        assert!(!config.gpu_lod());
        assert!(!config.descriptor_caching());
        assert!(!config.hiz_occlusion());
        assert!(!config.mesh_streaming());
        assert_eq!(config.enabled_count(), 0);
    }

    #[test]
    fn test_toggle_multi_draw_indirect() {
        let mut config = RenderingOptimizationConfig::default();
        let initial = config.multi_draw_indirect();

        config.set_multi_draw_indirect(!initial);
        assert_eq!(config.multi_draw_indirect(), !initial);
        assert!(config.has_changed());

        config.clear_changed_flag();
        assert!(!config.has_changed());
    }

    #[test]
    fn test_toggle_gpu_culling() {
        let mut config = RenderingOptimizationConfig::default();
        let initial = config.gpu_culling();

        config.set_gpu_culling(!initial);
        assert_eq!(config.gpu_culling(), !initial);
        assert!(config.has_changed());
    }

    #[test]
    fn test_toggle_gpu_lod() {
        let mut config = RenderingOptimizationConfig::default();
        let initial = config.gpu_lod();

        config.set_gpu_lod(!initial);
        assert_eq!(config.gpu_lod(), !initial);
        assert!(config.has_changed());
    }

    #[test]
    fn test_toggle_descriptor_caching() {
        let mut config = RenderingOptimizationConfig::default();
        let initial = config.descriptor_caching();

        config.set_descriptor_caching(!initial);
        assert_eq!(config.descriptor_caching(), !initial);
        assert!(config.has_changed());
    }

    #[test]
    fn test_toggle_hiz_occlusion() {
        let mut config = RenderingOptimizationConfig::default();
        let initial = config.hiz_occlusion();

        config.set_hiz_occlusion(!initial);
        assert_eq!(config.hiz_occlusion(), !initial);
        assert!(config.has_changed());
    }

    #[test]
    fn test_toggle_mesh_streaming() {
        let mut config = RenderingOptimizationConfig::default();
        let initial = config.mesh_streaming();

        config.set_mesh_streaming(!initial);
        assert_eq!(config.mesh_streaming(), !initial);
        assert!(config.has_changed());
    }

    #[test]
    fn test_enable_all() {
        let mut config = RenderingOptimizationConfig::all_disabled();
        assert_eq!(config.enabled_count(), 0);

        config.enable_all();
        assert_eq!(
            config.enabled_count(),
            RenderingOptimizationConfig::TOTAL_OPTIMIZATIONS
        );
        assert!(config.has_changed());
    }

    #[test]
    fn test_disable_all() {
        let mut config = RenderingOptimizationConfig::all_enabled();
        assert_eq!(
            config.enabled_count(),
            RenderingOptimizationConfig::TOTAL_OPTIMIZATIONS
        );

        config.disable_all();
        assert_eq!(config.enabled_count(), 0);
        assert!(config.has_changed());
    }

    #[test]
    fn test_reset_to_defaults() {
        let mut config = RenderingOptimizationConfig::all_disabled();
        config.reset_to_defaults();

        let defaults = RenderingOptimizationConfig::default();
        assert_eq!(config.multi_draw_indirect(), defaults.multi_draw_indirect());
        assert_eq!(config.gpu_culling(), defaults.gpu_culling());
        assert_eq!(config.gpu_lod(), defaults.gpu_lod());
        assert_eq!(config.descriptor_caching(), defaults.descriptor_caching());
        assert_eq!(config.hiz_occlusion(), defaults.hiz_occlusion());
        assert_eq!(config.mesh_streaming(), defaults.mesh_streaming());
        assert!(config.has_changed());
    }

    #[test]
    fn test_change_tracking() {
        let mut config = RenderingOptimizationConfig::default();
        assert!(!config.has_changed());

        config.set_gpu_culling(false);
        assert!(config.has_changed());

        config.clear_changed_flag();
        assert!(!config.has_changed());

        // No change when setting to same value
        config.set_gpu_culling(false);
        assert!(!config.has_changed());
    }

    #[test]
    fn test_panel_visibility() {
        let mut config = RenderingOptimizationConfig::default();
        assert!(config.is_panel_visible());

        config.set_show_panel(false);
        assert!(!config.is_panel_visible());

        config.toggle_panel();
        assert!(config.is_panel_visible());
    }

    #[test]
    fn test_enabled_count() {
        let mut config = RenderingOptimizationConfig::all_disabled();
        assert_eq!(config.enabled_count(), 0);

        config.set_multi_draw_indirect(true);
        assert_eq!(config.enabled_count(), 1);

        config.set_gpu_culling(true);
        assert_eq!(config.enabled_count(), 2);

        config.set_gpu_lod(true);
        assert_eq!(config.enabled_count(), 3);

        config.set_descriptor_caching(true);
        assert_eq!(config.enabled_count(), 4);

        config.set_hiz_occlusion(true);
        assert_eq!(config.enabled_count(), 5);

        config.set_mesh_streaming(true);
        assert_eq!(config.enabled_count(), 6);
    }

    #[test]
    fn test_summary() {
        let config = RenderingOptimizationConfig::default();
        let summary = config.summary();

        assert!(summary.contains("Multi-Draw Indirect"));
        assert!(summary.contains("GPU Culling"));
        assert!(summary.contains("GPU LOD Selection"));
        assert!(summary.contains("Descriptor Caching"));
        assert!(summary.contains("Hi-Z Occlusion"));
        assert!(summary.contains("Mesh Streaming"));
    }

    #[test]
    fn test_serialization() {
        let config = RenderingOptimizationConfig::all_enabled();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: RenderingOptimizationConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(
            config.multi_draw_indirect(),
            deserialized.multi_draw_indirect()
        );
        assert_eq!(config.gpu_culling(), deserialized.gpu_culling());
        assert_eq!(config.gpu_lod(), deserialized.gpu_lod());
        assert_eq!(
            config.descriptor_caching(),
            deserialized.descriptor_caching()
        );
        assert_eq!(config.hiz_occlusion(), deserialized.hiz_occlusion());
        assert_eq!(config.mesh_streaming(), deserialized.mesh_streaming());
    }

    #[test]
    fn test_no_spurious_changes() {
        let mut config = RenderingOptimizationConfig::default();
        config.clear_changed_flag();

        // Setting to same value should not trigger change
        config.set_multi_draw_indirect(config.multi_draw_indirect());
        assert!(!config.has_changed());

        config.set_gpu_culling(config.gpu_culling());
        assert!(!config.has_changed());
    }

    #[test]
    fn test_enable_all_idempotent() {
        let mut config = RenderingOptimizationConfig::all_enabled();
        config.clear_changed_flag();

        config.enable_all();
        // Should not set changed flag if already all enabled
        assert!(!config.has_changed());
    }

    #[test]
    fn test_disable_all_idempotent() {
        let mut config = RenderingOptimizationConfig::all_disabled();
        config.clear_changed_flag();

        config.disable_all();
        // Should not set changed flag if already all disabled
        assert!(!config.has_changed());
    }
}
