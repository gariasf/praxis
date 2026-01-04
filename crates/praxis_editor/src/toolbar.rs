//! Toolbar system for the Praxis editor.
//!
//! This module provides a toolbar with quick-access buttons for common editor operations:
//! - Gizmo mode buttons (translate/rotate/scale)
//! - Space toggle (local/world)
//! - Snap settings
//! - Play/pause/stop buttons
//! - Camera preset buttons (top/front/side/perspective)

use crate::{EditorMode, GizmoMode, GizmoSpace};

/// Actions that can be triggered by toolbar buttons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolbarAction {
    // Gizmo mode actions
    SetGizmoTranslate,
    SetGizmoRotate,
    SetGizmoScale,

    // Space toggle
    ToggleGizmoSpace,

    // Snap toggle
    ToggleSnapEnabled,

    // Play/pause/stop
    Play,
    Pause,
    Stop,

    // Camera presets
    SetCameraPreset(CameraPreset),
}

/// Camera preset view angles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraPreset {
    /// Top-down view (looking down the Y axis).
    Top,
    /// Bottom-up view (looking up the Y axis).
    Bottom,
    /// Front view (looking down the Z axis).
    Front,
    /// Back view (looking up the Z axis).
    Back,
    /// Right side view (looking left along X axis).
    Right,
    /// Left side view (looking right along X axis).
    Left,
    /// Perspective view (free camera).
    Perspective,
}

impl CameraPreset {
    /// Returns the display name for this preset.
    pub fn name(&self) -> &str {
        match self {
            CameraPreset::Top => "Top",
            CameraPreset::Bottom => "Bottom",
            CameraPreset::Front => "Front",
            CameraPreset::Back => "Back",
            CameraPreset::Right => "Right",
            CameraPreset::Left => "Left",
            CameraPreset::Perspective => "Perspective",
        }
    }
}

/// Snap settings for grid and angle snapping.
#[derive(Debug, Clone, Copy)]
pub struct SnapSettings {
    /// Whether snapping is enabled.
    pub enabled: bool,
    /// Grid snap increment for translation (in world units).
    pub grid_size: f32,
    /// Angle snap increment for rotation (in degrees).
    pub angle_increment: f32,
    /// Scale snap increment.
    pub scale_increment: f32,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            grid_size: 1.0,
            angle_increment: 15.0,
            scale_increment: 0.1,
        }
    }
}

impl SnapSettings {
    /// Creates new snap settings with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggles snapping on/off.
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

/// State for the toolbar.
pub struct ToolbarState {
    /// Current gizmo mode.
    pub gizmo_mode: GizmoMode,
    /// Current gizmo space.
    pub gizmo_space: GizmoSpace,
    /// Snap settings.
    pub snap_settings: SnapSettings,
    /// Current editor mode.
    pub editor_mode: EditorMode,
    /// Current camera preset.
    pub camera_preset: CameraPreset,
}

impl Default for ToolbarState {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolbarState {
    /// Creates a new toolbar state with default values.
    pub fn new() -> Self {
        Self {
            gizmo_mode: GizmoMode::Translate,
            gizmo_space: GizmoSpace::World,
            snap_settings: SnapSettings::default(),
            editor_mode: EditorMode::Edit,
            camera_preset: CameraPreset::Perspective,
        }
    }
}

/// Renders the toolbar and returns any triggered actions.
///
/// The toolbar is displayed as a horizontal panel below the menu bar with icon buttons
/// for quick access to common editor operations.
///
/// # Arguments
///
/// * `ctx` - The egui context
/// * `state` - Mutable reference to toolbar state
///
/// # Returns
///
/// A vector of actions triggered by button clicks
pub fn render_toolbar(ctx: &egui::Context, state: &mut ToolbarState) -> Vec<ToolbarAction> {
    let mut actions = Vec::new();

    egui::TopBottomPanel::top("editor_toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            // Gizmo mode buttons
            ui.group(|ui| {
                ui.label("Gizmo:");

                if ui
                    .selectable_label(state.gizmo_mode == GizmoMode::Translate, "🔷 Move")
                    .on_hover_text("Translate mode (W)")
                    .clicked()
                {
                    actions.push(ToolbarAction::SetGizmoTranslate);
                }

                if ui
                    .selectable_label(state.gizmo_mode == GizmoMode::Rotate, "🔄 Rotate")
                    .on_hover_text("Rotate mode (E)")
                    .clicked()
                {
                    actions.push(ToolbarAction::SetGizmoRotate);
                }

                if ui
                    .selectable_label(state.gizmo_mode == GizmoMode::Scale, "📏 Scale")
                    .on_hover_text("Scale mode (R)")
                    .clicked()
                {
                    actions.push(ToolbarAction::SetGizmoScale);
                }
            });

            ui.separator();

            // Space toggle
            ui.group(|ui| {
                let space_text = match state.gizmo_space {
                    GizmoSpace::World => "🌍 World",
                    GizmoSpace::Local => "📍 Local",
                };

                if ui
                    .button(space_text)
                    .on_hover_text("Toggle between World and Local space (X)")
                    .clicked()
                {
                    actions.push(ToolbarAction::ToggleGizmoSpace);
                }
            });

            ui.separator();

            // Snap settings
            ui.group(|ui| {
                let snap_text = if state.snap_settings.enabled {
                    "🧲 Snap: ON"
                } else {
                    "🧲 Snap: OFF"
                };

                if ui
                    .button(snap_text)
                    .on_hover_text("Toggle grid snapping (Ctrl+\\)")
                    .clicked()
                {
                    actions.push(ToolbarAction::ToggleSnapEnabled);
                }

                if state.snap_settings.enabled {
                    ui.label(format!("Grid: {:.1}", state.snap_settings.grid_size));
                }
            });

            ui.separator();

            // Play/Pause/Stop controls
            ui.group(|ui| {
                ui.label("Playback:");

                let is_playing = state.editor_mode == EditorMode::Play;
                let is_edit = state.editor_mode == EditorMode::Edit;

                // Play button - green when ready to play
                let play_button = if is_edit {
                    egui::Button::new("▶ Play")
                        .fill(egui::Color32::from_rgb(40, 120, 50))
                } else {
                    egui::Button::new("▶ Play")
                };
                
                if ui
                    .add_enabled(is_edit, play_button)
                    .on_hover_text("Start play mode (F5)")
                    .clicked()
                {
                    actions.push(ToolbarAction::Play);
                }

                // Pause button - yellow/orange when playing
                let pause_button = if is_playing {
                    egui::Button::new("⏸ Pause")
                        .fill(egui::Color32::from_rgb(200, 150, 40))
                } else {
                    egui::Button::new("⏸ Pause")
                };

                if ui
                    .add_enabled(is_playing, pause_button)
                    .on_hover_text("Pause play mode (F6)")
                    .clicked()
                {
                    actions.push(ToolbarAction::Pause);
                }

                // Stop button - red when playing
                let stop_button = if is_playing {
                    egui::Button::new("⏹ Stop")
                        .fill(egui::Color32::from_rgb(180, 40, 40))
                } else {
                    egui::Button::new("⏹ Stop")
                };

                if ui
                    .add_enabled(is_playing, stop_button)
                    .on_hover_text("Stop play mode and return to edit (F7)")
                    .clicked()
                {
                    actions.push(ToolbarAction::Stop);
                }
            });

            ui.separator();

            // Camera presets
            ui.group(|ui| {
                ui.label("Camera:");

                ui.menu_button("📷 View", |ui| {
                    let presets = [
                        CameraPreset::Perspective,
                        CameraPreset::Top,
                        CameraPreset::Bottom,
                        CameraPreset::Front,
                        CameraPreset::Back,
                        CameraPreset::Right,
                        CameraPreset::Left,
                    ];

                    for preset in presets {
                        if ui
                            .selectable_label(state.camera_preset == preset, preset.name())
                            .clicked()
                        {
                            actions.push(ToolbarAction::SetCameraPreset(preset));
                            ui.close_menu();
                        }
                    }
                });

                // Quick access buttons for common views
                if ui.small_button("T").on_hover_text("Top view").clicked() {
                    actions.push(ToolbarAction::SetCameraPreset(CameraPreset::Top));
                }

                if ui.small_button("F").on_hover_text("Front view").clicked() {
                    actions.push(ToolbarAction::SetCameraPreset(CameraPreset::Front));
                }

                if ui.small_button("R").on_hover_text("Right view").clicked() {
                    actions.push(ToolbarAction::SetCameraPreset(CameraPreset::Right));
                }

                if ui
                    .small_button("P")
                    .on_hover_text("Perspective view")
                    .clicked()
                {
                    actions.push(ToolbarAction::SetCameraPreset(CameraPreset::Perspective));
                }
            });

            // Right-aligned info
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!(
                    "Mode: {:?} | Space: {:?}",
                    state.gizmo_mode, state.gizmo_space
                ));
            });
        });
    });

    actions
}

/// Handles toolbar actions by updating the state and returning command actions.
///
/// # Arguments
///
/// * `action` - The toolbar action to handle
/// * `state` - Mutable reference to toolbar state
///
/// # Returns
///
/// `true` if the action modifies the gizmo system and requires updating external state
pub fn handle_toolbar_action(action: ToolbarAction, state: &mut ToolbarState) -> bool {
    use praxis_utils::info;

    match action {
        ToolbarAction::SetGizmoTranslate => {
            state.gizmo_mode = GizmoMode::Translate;
            info!("Gizmo mode set to Translate");
            true
        }
        ToolbarAction::SetGizmoRotate => {
            state.gizmo_mode = GizmoMode::Rotate;
            info!("Gizmo mode set to Rotate");
            true
        }
        ToolbarAction::SetGizmoScale => {
            state.gizmo_mode = GizmoMode::Scale;
            info!("Gizmo mode set to Scale");
            true
        }
        ToolbarAction::ToggleGizmoSpace => {
            state.gizmo_space = match state.gizmo_space {
                GizmoSpace::World => GizmoSpace::Local,
                GizmoSpace::Local => GizmoSpace::World,
            };
            info!("Gizmo space toggled to {:?}", state.gizmo_space);
            true
        }
        ToolbarAction::ToggleSnapEnabled => {
            state.snap_settings.toggle();
            info!("Snap enabled: {}", state.snap_settings.enabled);
            false
        }
        ToolbarAction::Play => {
            // Note: Actual play mode transition is handled by EditorState
            // This just updates the toolbar state
            state.editor_mode = EditorMode::Play;
            info!("Play action triggered");
            false
        }
        ToolbarAction::Pause => {
            // Note: Actual pause is handled by EditorState
            // Pause returns to Edit mode for UI purposes
            state.editor_mode = EditorMode::Edit;
            info!("Pause action triggered");
            false
        }
        ToolbarAction::Stop => {
            // Note: Actual stop is handled by EditorState
            // Stop returns to Edit mode
            state.editor_mode = EditorMode::Edit;
            info!("Stop action triggered");
            false
        }
        ToolbarAction::SetCameraPreset(preset) => {
            state.camera_preset = preset;
            info!("Camera preset set to {:?}", preset);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolbar_state_default() {
        let state = ToolbarState::new();
        assert_eq!(state.gizmo_mode, GizmoMode::Translate);
        assert_eq!(state.gizmo_space, GizmoSpace::World);
        assert!(!state.snap_settings.enabled);
        assert_eq!(state.editor_mode, EditorMode::Edit);
        assert_eq!(state.camera_preset, CameraPreset::Perspective);
    }

    #[test]
    fn test_snap_settings_toggle() {
        let mut settings = SnapSettings::new();
        assert!(!settings.enabled);

        settings.toggle();
        assert!(settings.enabled);

        settings.toggle();
        assert!(!settings.enabled);
    }

    #[test]
    fn test_handle_gizmo_mode_actions() {
        let mut state = ToolbarState::new();

        assert!(handle_toolbar_action(
            ToolbarAction::SetGizmoRotate,
            &mut state
        ));
        assert_eq!(state.gizmo_mode, GizmoMode::Rotate);

        assert!(handle_toolbar_action(
            ToolbarAction::SetGizmoScale,
            &mut state
        ));
        assert_eq!(state.gizmo_mode, GizmoMode::Scale);

        assert!(handle_toolbar_action(
            ToolbarAction::SetGizmoTranslate,
            &mut state
        ));
        assert_eq!(state.gizmo_mode, GizmoMode::Translate);
    }

    #[test]
    fn test_handle_space_toggle() {
        let mut state = ToolbarState::new();
        assert_eq!(state.gizmo_space, GizmoSpace::World);

        assert!(handle_toolbar_action(
            ToolbarAction::ToggleGizmoSpace,
            &mut state
        ));
        assert_eq!(state.gizmo_space, GizmoSpace::Local);

        assert!(handle_toolbar_action(
            ToolbarAction::ToggleGizmoSpace,
            &mut state
        ));
        assert_eq!(state.gizmo_space, GizmoSpace::World);
    }

    #[test]
    fn test_handle_snap_toggle() {
        let mut state = ToolbarState::new();
        assert!(!state.snap_settings.enabled);

        assert!(!handle_toolbar_action(
            ToolbarAction::ToggleSnapEnabled,
            &mut state
        ));
        assert!(state.snap_settings.enabled);

        assert!(!handle_toolbar_action(
            ToolbarAction::ToggleSnapEnabled,
            &mut state
        ));
        assert!(!state.snap_settings.enabled);
    }

    #[test]
    fn test_handle_playback_actions() {
        let mut state = ToolbarState::new();
        assert_eq!(state.editor_mode, EditorMode::Edit);

        handle_toolbar_action(ToolbarAction::Play, &mut state);
        assert_eq!(state.editor_mode, EditorMode::Play);

        handle_toolbar_action(ToolbarAction::Pause, &mut state);
        assert_eq!(state.editor_mode, EditorMode::Edit);

        handle_toolbar_action(ToolbarAction::Play, &mut state);
        handle_toolbar_action(ToolbarAction::Stop, &mut state);
        assert_eq!(state.editor_mode, EditorMode::Edit);
    }

    #[test]
    fn test_handle_camera_preset() {
        let mut state = ToolbarState::new();
        assert_eq!(state.camera_preset, CameraPreset::Perspective);

        handle_toolbar_action(
            ToolbarAction::SetCameraPreset(CameraPreset::Top),
            &mut state,
        );
        assert_eq!(state.camera_preset, CameraPreset::Top);

        handle_toolbar_action(
            ToolbarAction::SetCameraPreset(CameraPreset::Front),
            &mut state,
        );
        assert_eq!(state.camera_preset, CameraPreset::Front);
    }

    #[test]
    fn test_camera_preset_names() {
        assert_eq!(CameraPreset::Top.name(), "Top");
        assert_eq!(CameraPreset::Bottom.name(), "Bottom");
        assert_eq!(CameraPreset::Front.name(), "Front");
        assert_eq!(CameraPreset::Back.name(), "Back");
        assert_eq!(CameraPreset::Right.name(), "Right");
        assert_eq!(CameraPreset::Left.name(), "Left");
        assert_eq!(CameraPreset::Perspective.name(), "Perspective");
    }
}
