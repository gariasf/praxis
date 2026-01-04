//! Project settings panel for configuring engine parameters.

use super::EditorPanel;
use egui::Ui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Graphics settings for the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsSettings {
    /// Window resolution width.
    pub resolution_width: u32,
    /// Window resolution height.
    pub resolution_height: u32,
    /// MSAA (Multi-Sample Anti-Aliasing) sample count. 1 = disabled, 2/4/8 = enabled.
    pub msaa_samples: u32,
    /// Whether VSync is enabled.
    pub vsync: bool,
    /// Whether fullscreen mode is enabled.
    pub fullscreen: bool,
    /// Target frame rate (0 = unlimited).
    pub target_fps: u32,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            resolution_width: 1920,
            resolution_height: 1080,
            msaa_samples: 1,
            vsync: true,
            fullscreen: false,
            target_fps: 60,
        }
    }
}

/// Physics settings for the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsSettings {
    /// Gravity vector (x, y, z).
    pub gravity_x: f32,
    pub gravity_y: f32,
    pub gravity_z: f32,
    /// Fixed timestep for physics simulation in seconds.
    pub timestep: f32,
    /// Number of solver iterations for position correction.
    pub position_iterations: u32,
    /// Number of solver iterations for velocity correction.
    pub velocity_iterations: u32,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            gravity_x: 0.0,
            gravity_y: -9.81,
            gravity_z: 0.0,
            timestep: 1.0 / 60.0,
            position_iterations: 4,
            velocity_iterations: 1,
        }
    }
}

/// Audio settings for the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    /// Master volume (0.0 to 1.0).
    pub master_volume: f32,
    /// Music volume (0.0 to 1.0).
    pub music_volume: f32,
    /// Sound effects volume (0.0 to 1.0).
    pub sfx_volume: f32,
    /// Doppler effect scale factor.
    pub doppler_scale: f32,
    /// Speed of sound in units per second (for doppler effect).
    pub speed_of_sound: f32,
    /// Maximum number of simultaneous audio sources.
    pub max_audio_sources: u32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 0.8,
            doppler_scale: 1.0,
            speed_of_sound: 343.0,
            max_audio_sources: 32,
        }
    }
}

/// Input settings for the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSettings {
    /// Mouse sensitivity multiplier.
    pub mouse_sensitivity: f32,
    /// Whether to invert mouse Y-axis.
    pub invert_mouse_y: bool,
    /// Gamepad deadzone (0.0 to 1.0).
    pub gamepad_deadzone: f32,
    /// Key bindings map (action name -> key binding description).
    pub key_bindings: HashMap<String, String>,
}

impl Default for InputSettings {
    fn default() -> Self {
        let mut key_bindings = HashMap::new();
        key_bindings.insert("Forward".to_string(), "W".to_string());
        key_bindings.insert("Backward".to_string(), "S".to_string());
        key_bindings.insert("Left".to_string(), "A".to_string());
        key_bindings.insert("Right".to_string(), "D".to_string());
        key_bindings.insert("Jump".to_string(), "Space".to_string());
        key_bindings.insert("Sprint".to_string(), "Shift".to_string());

        Self {
            mouse_sensitivity: 1.0,
            invert_mouse_y: false,
            gamepad_deadzone: 0.15,
            key_bindings,
        }
    }
}

/// Complete project settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// Graphics settings.
    pub graphics: GraphicsSettings,
    /// Physics settings.
    pub physics: PhysicsSettings,
    /// Audio settings.
    pub audio: AudioSettings,
    /// Input settings.
    pub input: InputSettings,
    /// Project name.
    pub project_name: String,
    /// Project version.
    pub project_version: String,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            graphics: GraphicsSettings::default(),
            physics: PhysicsSettings::default(),
            audio: AudioSettings::default(),
            input: InputSettings::default(),
            project_name: "Praxis Project".to_string(),
            project_version: "0.1.0".to_string(),
        }
    }
}

impl ProjectSettings {
    /// Loads project settings from a RON file.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;

        ron::from_str(&contents).map_err(|e| format!("Failed to parse settings: {}", e))
    }

    /// Saves project settings to a RON file.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref();

        let pretty_config = ron::ser::PrettyConfig::new()
            .depth_limit(4)
            .separate_tuple_members(true)
            .enumerate_arrays(false)
            .indentor("    ".to_string());

        let ron_string = ron::ser::to_string_pretty(self, pretty_config)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        std::fs::write(path, ron_string)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        Ok(())
    }
}

/// Active tab in the project settings panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    Graphics,
    Physics,
    Audio,
    Input,
    General,
}

/// Panel for editing project settings.
pub struct ProjectSettingsPanel {
    title: String,
    settings: ProjectSettings,
    active_tab: SettingsTab,
    settings_path: String,
    status_message: Option<String>,
    new_action_name: String,
    new_key_binding: String,
}

impl ProjectSettingsPanel {
    /// Creates a new project settings panel.
    #[must_use]
    pub fn new() -> Self {
        let settings_path = "project.ron".to_string();
        let settings = ProjectSettings::load_from_file(&settings_path).unwrap_or_default();

        Self {
            title: "Project Settings".to_string(),
            settings,
            active_tab: SettingsTab::General,
            settings_path,
            status_message: None,
            new_action_name: String::new(),
            new_key_binding: String::new(),
        }
    }

    /// Creates a new project settings panel with a specific settings path.
    #[must_use]
    pub fn with_path(path: String) -> Self {
        let settings = ProjectSettings::load_from_file(&path).unwrap_or_default();

        Self {
            title: "Project Settings".to_string(),
            settings,
            active_tab: SettingsTab::General,
            settings_path: path,
            status_message: None,
            new_action_name: String::new(),
            new_key_binding: String::new(),
        }
    }

    /// Gets a reference to the current settings.
    #[must_use]
    pub const fn settings(&self) -> &ProjectSettings {
        &self.settings
    }

    /// Gets a mutable reference to the current settings.
    #[must_use]
    pub fn settings_mut(&mut self) -> &mut ProjectSettings {
        &mut self.settings
    }

    /// Saves the current settings to file.
    pub fn save(&mut self) {
        match self.settings.save_to_file(&self.settings_path) {
            Ok(()) => {
                self.status_message = Some(format!("Settings saved to {}", self.settings_path));
            }
            Err(e) => {
                self.status_message = Some(format!("Error saving settings: {}", e));
            }
        }
    }

    /// Loads settings from file.
    pub fn load(&mut self) {
        match ProjectSettings::load_from_file(&self.settings_path) {
            Ok(settings) => {
                self.settings = settings;
                self.status_message = Some(format!("Settings loaded from {}", self.settings_path));
            }
            Err(e) => {
                self.status_message = Some(format!("Error loading settings: {}", e));
            }
        }
    }

    /// Resets settings to defaults.
    pub fn reset_to_defaults(&mut self) {
        self.settings = ProjectSettings::default();
        self.status_message = Some("Settings reset to defaults".to_string());
    }

    fn render_tabs(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.active_tab, SettingsTab::General, "General");
            ui.selectable_value(&mut self.active_tab, SettingsTab::Graphics, "Graphics");
            ui.selectable_value(&mut self.active_tab, SettingsTab::Physics, "Physics");
            ui.selectable_value(&mut self.active_tab, SettingsTab::Audio, "Audio");
            ui.selectable_value(&mut self.active_tab, SettingsTab::Input, "Input");
        });
        ui.separator();
    }

    fn render_general_tab(&mut self, ui: &mut Ui) {
        ui.heading("General Settings");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Project Name:");
            ui.text_edit_singleline(&mut self.settings.project_name);
        });

        ui.horizontal(|ui| {
            ui.label("Project Version:");
            ui.text_edit_singleline(&mut self.settings.project_version);
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Settings File:");
            ui.text_edit_singleline(&mut self.settings_path);
        });
    }

    fn render_graphics_tab(&mut self, ui: &mut Ui) {
        ui.heading("Graphics Settings");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Resolution Width:");
            ui.add(egui::DragValue::new(&mut self.settings.graphics.resolution_width).clamp_range(640..=7680));
        });

        ui.horizontal(|ui| {
            ui.label("Resolution Height:");
            ui.add(egui::DragValue::new(&mut self.settings.graphics.resolution_height).clamp_range(480..=4320));
        });

        ui.horizontal(|ui| {
            ui.label("MSAA Samples:");
            egui::ComboBox::from_id_source("msaa_combo")
                .selected_text(format!("{}", self.settings.graphics.msaa_samples))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.settings.graphics.msaa_samples, 1, "1 (Off)");
                    ui.selectable_value(&mut self.settings.graphics.msaa_samples, 2, "2");
                    ui.selectable_value(&mut self.settings.graphics.msaa_samples, 4, "4");
                    ui.selectable_value(&mut self.settings.graphics.msaa_samples, 8, "8");
                });
        });

        ui.checkbox(&mut self.settings.graphics.vsync, "VSync");
        ui.checkbox(&mut self.settings.graphics.fullscreen, "Fullscreen");

        ui.horizontal(|ui| {
            ui.label("Target FPS:");
            ui.add(egui::DragValue::new(&mut self.settings.graphics.target_fps).clamp_range(0..=300));
            ui.label("(0 = unlimited)");
        });
    }

    fn render_physics_tab(&mut self, ui: &mut Ui) {
        ui.heading("Physics Settings");
        ui.separator();

        ui.label("Gravity:");
        ui.horizontal(|ui| {
            ui.label("X:");
            ui.add(egui::DragValue::new(&mut self.settings.physics.gravity_x).speed(0.1));
            ui.label("Y:");
            ui.add(egui::DragValue::new(&mut self.settings.physics.gravity_y).speed(0.1));
            ui.label("Z:");
            ui.add(egui::DragValue::new(&mut self.settings.physics.gravity_z).speed(0.1));
        });

        ui.horizontal(|ui| {
            if ui.button("Earth Gravity").clicked() {
                self.settings.physics.gravity_x = 0.0;
                self.settings.physics.gravity_y = -9.81;
                self.settings.physics.gravity_z = 0.0;
            }
            if ui.button("Zero Gravity").clicked() {
                self.settings.physics.gravity_x = 0.0;
                self.settings.physics.gravity_y = 0.0;
                self.settings.physics.gravity_z = 0.0;
            }
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Timestep (seconds):");
            ui.add(egui::DragValue::new(&mut self.settings.physics.timestep).speed(0.001).clamp_range(0.001..=0.1));
        });

        ui.horizontal(|ui| {
            ui.label("Position Iterations:");
            ui.add(egui::DragValue::new(&mut self.settings.physics.position_iterations).clamp_range(1..=20));
        });

        ui.horizontal(|ui| {
            ui.label("Velocity Iterations:");
            ui.add(egui::DragValue::new(&mut self.settings.physics.velocity_iterations).clamp_range(1..=20));
        });
    }

    fn render_audio_tab(&mut self, ui: &mut Ui) {
        ui.heading("Audio Settings");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Master Volume:");
            ui.add(egui::Slider::new(&mut self.settings.audio.master_volume, 0.0..=1.0));
        });

        ui.horizontal(|ui| {
            ui.label("Music Volume:");
            ui.add(egui::Slider::new(&mut self.settings.audio.music_volume, 0.0..=1.0));
        });

        ui.horizontal(|ui| {
            ui.label("SFX Volume:");
            ui.add(egui::Slider::new(&mut self.settings.audio.sfx_volume, 0.0..=1.0));
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Doppler Scale:");
            ui.add(egui::DragValue::new(&mut self.settings.audio.doppler_scale).speed(0.01).clamp_range(0.0..=10.0));
        });

        ui.horizontal(|ui| {
            ui.label("Speed of Sound:");
            ui.add(egui::DragValue::new(&mut self.settings.audio.speed_of_sound).speed(1.0).clamp_range(100.0..=1000.0));
            ui.label("units/s");
        });

        ui.horizontal(|ui| {
            ui.label("Max Audio Sources:");
            ui.add(egui::DragValue::new(&mut self.settings.audio.max_audio_sources).clamp_range(1..=128));
        });
    }

    fn render_input_tab(&mut self, ui: &mut Ui) {
        ui.heading("Input Settings");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Mouse Sensitivity:");
            ui.add(egui::Slider::new(&mut self.settings.input.mouse_sensitivity, 0.1..=5.0));
        });

        ui.checkbox(&mut self.settings.input.invert_mouse_y, "Invert Mouse Y");

        ui.horizontal(|ui| {
            ui.label("Gamepad Deadzone:");
            ui.add(egui::Slider::new(&mut self.settings.input.gamepad_deadzone, 0.0..=0.5));
        });

        ui.separator();
        ui.heading("Key Bindings");

        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
            let mut to_remove = None;

            let bindings: Vec<_> = self.settings.input.key_bindings.iter().collect();
            for (i, (action, key)) in bindings.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", action));
                    ui.label(*key);
                    if ui.button("Remove").clicked() {
                        to_remove = Some((*action).clone());
                    }
                });

                if i < bindings.len() - 1 {
                    ui.separator();
                }
            }

            if let Some(action) = to_remove {
                self.settings.input.key_bindings.remove(&action);
            }
        });

        ui.separator();
        ui.heading("Add Key Binding");

        ui.horizontal(|ui| {
            ui.label("Action:");
            ui.text_edit_singleline(&mut self.new_action_name);
        });

        ui.horizontal(|ui| {
            ui.label("Key:");
            ui.text_edit_singleline(&mut self.new_key_binding);
        });

        if ui.button("Add Binding").clicked() {
            if !self.new_action_name.is_empty() && !self.new_key_binding.is_empty() {
                self.settings.input.key_bindings.insert(
                    self.new_action_name.clone(),
                    self.new_key_binding.clone(),
                );
                self.new_action_name.clear();
                self.new_key_binding.clear();
            }
        }
    }

    fn render_buttons(&mut self, ui: &mut Ui) {
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                self.save();
            }

            if ui.button("Load").clicked() {
                self.load();
            }

            if ui.button("Reset to Defaults").clicked() {
                self.reset_to_defaults();
            }
        });

        if let Some(message) = &self.status_message {
            ui.separator();
            ui.colored_label(egui::Color32::GREEN, message);
        }
    }
}

impl Default for ProjectSettingsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for ProjectSettingsPanel {
    fn title(&self) -> &str {
        &self.title
    }

    fn ui(&mut self, ui: &mut Ui) {
        self.render_tabs(ui);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                match self.active_tab {
                    SettingsTab::General => self.render_general_tab(ui),
                    SettingsTab::Graphics => self.render_graphics_tab(ui),
                    SettingsTab::Physics => self.render_physics_tab(ui),
                    SettingsTab::Audio => self.render_audio_tab(ui),
                    SettingsTab::Input => self.render_input_tab(ui),
                }

                self.render_buttons(ui);
            });
    }
}
