//! Core editor state management and coordination.

use crate::editor_mode::EditorMode;
use crate::panels::{
    AssetsPanel, ConsolePanel, EditorPanel, HierarchyPanel, InspectorPanel, SceneViewPanel,
};
use crate::UndoRedoSystem;
use bevy_ecs::world::World;
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use praxis_utils::info;

/// Tab identifier for different editor panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTab {
    /// Scene view panel.
    Scene,
    /// Hierarchy panel.
    Hierarchy,
    /// Inspector panel.
    Inspector,
    /// Console panel.
    Console,
    /// Assets panel.
    Assets,
}

/// Main editor state that coordinates all editor panels and modes.
pub struct EditorState {
    /// Current editor mode (Edit or Play).
    mode: EditorMode,
    /// Dock state managing panel layout.
    dock_state: DockState<EditorTab>,
    /// Scene view panel.
    scene_panel: SceneViewPanel,
    /// Hierarchy panel.
    hierarchy_panel: HierarchyPanel,
    /// Inspector panel.
    inspector_panel: InspectorPanel,
    /// Console panel.
    console_panel: ConsolePanel,
    /// Assets panel.
    assets_panel: AssetsPanel,
    /// Whether the editor is visible.
    visible: bool,
}

impl EditorState {
    /// Creates a new editor state with default layout.
    #[must_use]
    pub fn new() -> Self {
        let mut dock_state = DockState::new(vec![EditorTab::Scene]);

        let tree = dock_state.main_surface_mut();

        let [scene, right] = tree.split_right(
            NodeIndex::root(),
            0.75,
            vec![EditorTab::Scene],
        );

        let [_right_top, right_bottom] = tree.split_below(
            right,
            0.7,
            vec![EditorTab::Inspector],
        );

        let [left, _scene] = tree.split_left(
            scene,
            0.2,
            vec![EditorTab::Hierarchy],
        );

        tree.split_below(
            left,
            0.6,
            vec![EditorTab::Assets],
        );

        tree.split_below(
            right_bottom,
            0.5,
            vec![EditorTab::Console],
        );

        Self {
            mode: EditorMode::default(),
            dock_state,
            scene_panel: SceneViewPanel::new(),
            hierarchy_panel: HierarchyPanel::new(),
            inspector_panel: InspectorPanel::new(),
            console_panel: ConsolePanel::new(),
            assets_panel: AssetsPanel::new(),
            visible: true,
        }
    }

    /// Returns the current editor mode.
    #[must_use]
    pub const fn mode(&self) -> EditorMode {
        self.mode
    }

    /// Sets the editor mode.
    pub fn set_mode(&mut self, mode: EditorMode) {
        if self.mode != mode {
            info!("Switching editor mode to {:?}", mode);
            self.mode = mode;
        }
    }

    /// Toggles between edit and play modes.
    pub fn toggle_mode(&mut self) {
        self.set_mode(self.mode.toggle());
    }

    /// Returns whether the editor is visible.
    #[must_use]
    pub const fn is_visible(&self) -> bool {
        self.visible
    }

    /// Sets the editor visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Toggles editor visibility.
    pub fn toggle_visible(&mut self) {
        self.visible = !self.visible;
    }

    /// Gets a mutable reference to the console panel.
    #[must_use]
    pub fn console_panel_mut(&mut self) -> &mut ConsolePanel {
        &mut self.console_panel
    }

    /// Gets a mutable reference to the hierarchy panel.
    #[must_use]
    pub fn hierarchy_panel_mut(&mut self) -> &mut HierarchyPanel {
        &mut self.hierarchy_panel
    }

    /// Gets a mutable reference to the inspector panel.
    #[must_use]
    pub fn inspector_panel_mut(&mut self) -> &mut InspectorPanel {
        &mut self.inspector_panel
    }

    /// Gets a mutable reference to the scene panel.
    #[must_use]
    pub fn scene_panel_mut(&mut self) -> &mut SceneViewPanel {
        &mut self.scene_panel
    }

    /// Gets a mutable reference to the assets panel.
    #[must_use]
    pub fn assets_panel_mut(&mut self) -> &mut AssetsPanel {
        &mut self.assets_panel
    }

    /// Renders the editor UI.
    /// 
    /// # Arguments
    /// * `ctx` - The egui context
    /// * `undo_system` - Optional mutable reference to the undo/redo system for menu integration
    /// * `world` - Optional mutable reference to the ECS world for executing undo/redo commands
    pub fn ui(&mut self, ctx: &egui::Context, undo_system: Option<&mut UndoRedoSystem>, world: Option<&mut World>) {
        if !self.visible {
            return;
        }

        self.render_menu_bar(ctx, undo_system, world);
        self.render_dock_area(ctx);
    }

    fn render_menu_bar(&mut self, ctx: &egui::Context, mut undo_system: Option<&mut UndoRedoSystem>, mut world: Option<&mut World>) {
        egui::TopBottomPanel::top("editor_menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // File menu
                ui.menu_button("File", |ui| {
                    if ui.button("New Scene").clicked() {
                        info!("New scene requested");
                        ui.close_menu();
                    }
                    if ui.button("Open Scene").clicked() {
                        info!("Open scene requested");
                        ui.close_menu();
                    }
                    
                    // Show dirty indicator in save button
                    let is_dirty = undo_system.as_ref().map_or(false, |s| s.is_dirty());
                    let save_text = if is_dirty {
                        "Save Scene *"
                    } else {
                        "Save Scene"
                    };
                    
                    if ui.button(save_text).clicked() {
                        info!("Save scene requested");
                        if let Some(system) = undo_system.as_mut() {
                            system.mark_saved();
                            info!("Scene marked as saved");
                        }
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        info!("Exit requested");
                        ui.close_menu();
                    }
                });

                // Edit menu with undo/redo
                ui.menu_button("Edit", |ui| {
                    // Collect state info before borrowing mutably
                    let (can_undo, can_redo, undo_text, redo_text, undo_count, redo_count) = 
                        if let Some(ref system) = undo_system {
                            let can_undo = system.can_undo();
                            let can_redo = system.can_redo();
                            
                            let undo_text = if let Some(desc) = system.undo_description() {
                                format!("Undo: {} (Ctrl+Z)", desc)
                            } else {
                                "Undo (Ctrl+Z)".to_string()
                            };
                            
                            let redo_text = if let Some(desc) = system.redo_description() {
                                format!("Redo: {} (Ctrl+Y)", desc)
                            } else {
                                "Redo (Ctrl+Y)".to_string()
                            };
                            
                            (can_undo, can_redo, undo_text, redo_text, system.undo_count(), system.redo_count())
                        } else {
                            (false, false, "Undo (Ctrl+Z)".to_string(), "Redo (Ctrl+Y)".to_string(), 0, 0)
                        };
                    
                    // Undo button with description and shortcut
                    let undo_button = ui.add_enabled(can_undo, egui::Button::new(&undo_text));
                    if undo_button.clicked() {
                        if let (Some(system), Some(world)) = (undo_system.as_mut(), world.as_mut()) {
                            if let Err(e) = system.undo(world) {
                                praxis_utils::error!("Undo failed: {}", e);
                            } else {
                                info!("Undo executed");
                            }
                        }
                        ui.close_menu();
                    }
                    
                    // Redo button with description and shortcut
                    let redo_button = ui.add_enabled(can_redo, egui::Button::new(&redo_text));
                    if redo_button.clicked() {
                        if let (Some(system), Some(world)) = (undo_system.as_mut(), world.as_mut()) {
                            if let Err(e) = system.redo(world) {
                                praxis_utils::error!("Redo failed: {}", e);
                            } else {
                                info!("Redo executed");
                            }
                        }
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    // Show command history info
                    ui.label(format!("History: {} undo / {} redo", undo_count, redo_count));
                });

                // View menu
                ui.menu_button("View", |ui| {
                    if ui
                        .checkbox(&mut self.visible, "Show Editor")
                        .clicked()
                    {
                        ui.close_menu();
                    }
                });

                ui.separator();

                // Play/Edit mode toggle
                let mode_text = match self.mode {
                    EditorMode::Edit => "▶ Play",
                    EditorMode::Play => "⏸ Edit",
                };

                if ui.button(mode_text).clicked() {
                    self.toggle_mode();
                }

                // Right-aligned status indicators
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Show dirty indicator
                    if undo_system.as_ref().map_or(false, |s| s.is_dirty()) {
                        ui.label(egui::RichText::new("● Unsaved").color(egui::Color32::from_rgb(255, 200, 0)));
                        ui.separator();
                    }
                    
                    ui.label(format!("Mode: {:?}", self.mode));
                });
            });
        });
    }

    fn render_dock_area(&mut self, ctx: &egui::Context) {
        let mut tab_viewer = EditorTabViewer {
            scene_panel: &mut self.scene_panel,
            hierarchy_panel: &mut self.hierarchy_panel,
            inspector_panel: &mut self.inspector_panel,
            console_panel: &mut self.console_panel,
            assets_panel: &mut self.assets_panel,
        };

        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}

struct EditorTabViewer<'a> {
    scene_panel: &'a mut SceneViewPanel,
    hierarchy_panel: &'a mut HierarchyPanel,
    inspector_panel: &'a mut InspectorPanel,
    console_panel: &'a mut ConsolePanel,
    assets_panel: &'a mut AssetsPanel,
}

impl TabViewer for EditorTabViewer<'_> {
    type Tab = EditorTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            EditorTab::Scene => self.scene_panel.title().into(),
            EditorTab::Hierarchy => self.hierarchy_panel.title().into(),
            EditorTab::Inspector => self.inspector_panel.title().into(),
            EditorTab::Console => self.console_panel.title().into(),
            EditorTab::Assets => self.assets_panel.title().into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            EditorTab::Scene => self.scene_panel.ui(ui),
            EditorTab::Hierarchy => self.hierarchy_panel.ui(ui),
            EditorTab::Inspector => self.inspector_panel.ui(ui),
            EditorTab::Console => self.console_panel.ui(ui),
            EditorTab::Assets => self.assets_panel.ui(ui),
        }
    }
}
