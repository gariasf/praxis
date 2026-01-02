//! Debug UI for displaying performance metrics and engine information.

use praxis_utils::timing::{current_fps, delta_time, frame_count, total_time};

/// Debug UI state and configuration.
pub struct DebugUi {
    /// Whether the debug UI is visible.
    pub visible: bool,
    /// Whether to show the FPS counter.
    pub show_fps: bool,
    /// Whether to show detailed performance metrics.
    pub show_performance: bool,
}

impl Default for DebugUi {
    fn default() -> Self {
        Self {
            visible: true,
            show_fps: true,
            show_performance: true,
        }
    }
}

impl DebugUi {
    /// Creates a new debug UI.
    pub fn new() -> Self {
        Self::default()
    }

    /// Renders the debug UI.
    pub fn render(&mut self, ctx: &egui::Context) {
        if !self.visible {
            return;
        }

        if self.show_fps {
            self.render_fps_counter(ctx);
        }

        if self.show_performance {
            self.render_performance_window(ctx);
        }
    }

    /// Renders the FPS counter overlay.
    fn render_fps_counter(&self, ctx: &egui::Context) {
        egui::Area::new("fps_counter".into())
            .fixed_pos(egui::pos2(10.0, 10.0))
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(egui::Color32::from_black_alpha(180))
                    .inner_margin(8.0)
                    .rounding(4.0)
                    .show(ui, |ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(0, 255, 100),
                            format!("FPS: {:.1}", current_fps()),
                        );
                    });
            });
    }

    /// Renders the detailed performance window.
    fn render_performance_window(&self, ctx: &egui::Context) {
        egui::Window::new("Performance")
            .default_pos(egui::pos2(10.0, 50.0))
            .default_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Performance Metrics");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("FPS:");
                    ui.colored_label(
                        egui::Color32::from_rgb(0, 255, 100),
                        format!("{:.1}", current_fps()),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Frame Time:");
                    let frame_time_ms = delta_time() * 1000.0;
                    let color = if frame_time_ms > 33.0 {
                        egui::Color32::from_rgb(255, 100, 100)
                    } else if frame_time_ms > 16.6 {
                        egui::Color32::from_rgb(255, 200, 100)
                    } else {
                        egui::Color32::from_rgb(100, 255, 100)
                    };
                    ui.colored_label(color, format!("{frame_time_ms:.2} ms"));
                });

                ui.horizontal(|ui| {
                    ui.label("Delta Time:");
                    ui.label(format!("{:.4} s", delta_time()));
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Frame Count:");
                    ui.label(format!("{}", frame_count()));
                });

                ui.horizontal(|ui| {
                    ui.label("Total Time:");
                    let secs = total_time().as_secs();
                    let mins = secs / 60;
                    let hours = mins / 60;
                    ui.label(format!(
                        "{:02}:{:02}:{:02}",
                        hours,
                        mins % 60,
                        secs % 60
                    ));
                });
            });
    }

    /// Toggles the visibility of the debug UI.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Sets the visibility of the debug UI.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}
