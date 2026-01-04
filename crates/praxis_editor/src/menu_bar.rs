//! MenuBar system for the Praxis editor.
//!
//! This module provides a comprehensive menu bar with standard menus and keyboard shortcuts:
//! - File: New, Open, Save, Save As, Exit
//! - Edit: Undo, Redo, Copy, Paste, Duplicate
//! - Entity: Create Empty, Create Primitives, Delete
//! - View: Toggle Panels
//! - Help: About, Documentation

use crate::{EditorMode, UndoRedoSystem};
use bevy_ecs::world::World;
use egui::Key;

/// Actions that can be triggered by menu items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuBarAction {
    // File menu
    NewScene,
    OpenScene,
    SaveScene,
    SaveSceneAs,
    Exit,
    
    // Edit menu
    Undo,
    Redo,
    Copy,
    Paste,
    Duplicate,
    
    // Entity menu
    CreateEmpty,
    CreateCube,
    CreateSphere,
    CreatePlane,
    CreateCylinder,
    CreateCone,
    DeleteEntity,
    
    // View menu
    ToggleHierarchy,
    ToggleInspector,
    ToggleConsole,
    ToggleAssets,
    ToggleScene,
    
    // Help menu
    About,
    Documentation,
    
    // Mode toggle
    TogglePlayMode,
}

/// State for the menu bar.
pub struct MenuBarState {
    /// Current editor mode.
    pub mode: EditorMode,
    /// Whether panels are visible.
    pub hierarchy_visible: bool,
    pub inspector_visible: bool,
    pub console_visible: bool,
    pub assets_visible: bool,
    pub scene_visible: bool,
}

impl MenuBarState {
    /// Creates a new menu bar state with default visibility.
    #[must_use]
    pub fn new() -> Self {
        Self {
            mode: EditorMode::Edit,
            hierarchy_visible: true,
            inspector_visible: true,
            console_visible: true,
            assets_visible: true,
            scene_visible: true,
        }
    }
}

impl Default for MenuBarState {
    fn default() -> Self {
        Self::new()
    }
}

/// Renders the menu bar and returns any triggered actions.
pub fn render_menu_bar(
    ctx: &egui::Context,
    state: &mut MenuBarState,
    undo_system: Option<&UndoRedoSystem>,
) -> Vec<MenuBarAction> {
    let mut actions = Vec::new();
    
    egui::TopBottomPanel::top("editor_menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            // File Menu
            ui.menu_button("File", |ui| {
                if ui.add(egui::Button::new("New Scene").shortcut_text("Ctrl+N")).clicked() {
                    actions.push(MenuBarAction::NewScene);
                    ui.close_menu();
                }
                
                if ui.add(egui::Button::new("Open Scene").shortcut_text("Ctrl+O")).clicked() {
                    actions.push(MenuBarAction::OpenScene);
                    ui.close_menu();
                }
                
                ui.separator();
                
                let is_dirty = undo_system.is_some_and(|s| s.is_dirty());
                let save_text = if is_dirty {
                    "Save Scene *"
                } else {
                    "Save Scene"
                };
                
                if ui.add(egui::Button::new(save_text).shortcut_text("Ctrl+S")).clicked() {
                    actions.push(MenuBarAction::SaveScene);
                    ui.close_menu();
                }
                
                if ui.add(egui::Button::new("Save Scene As...").shortcut_text("Ctrl+Shift+S")).clicked() {
                    actions.push(MenuBarAction::SaveSceneAs);
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.add(egui::Button::new("Exit").shortcut_text("Alt+F4")).clicked() {
                    actions.push(MenuBarAction::Exit);
                    ui.close_menu();
                }
            });
            
            // Edit Menu
            ui.menu_button("Edit", |ui| {
                let (can_undo, can_redo, undo_text, redo_text) = if let Some(system) = undo_system {
                    let undo_desc = system.undo_description()
                        .map(|d| format!("Undo: {d}"))
                        .unwrap_or_else(|| "Undo".to_string());
                    let redo_desc = system.redo_description()
                        .map(|d| format!("Redo: {d}"))
                        .unwrap_or_else(|| "Redo".to_string());
                    (system.can_undo(), system.can_redo(), undo_desc, redo_desc)
                } else {
                    (false, false, "Undo".to_string(), "Redo".to_string())
                };
                
                if ui.add_enabled(
                    can_undo,
                    egui::Button::new(&undo_text).shortcut_text("Ctrl+Z")
                ).clicked() {
                    actions.push(MenuBarAction::Undo);
                    ui.close_menu();
                }
                
                if ui.add_enabled(
                    can_redo,
                    egui::Button::new(&redo_text).shortcut_text("Ctrl+Y")
                ).clicked() {
                    actions.push(MenuBarAction::Redo);
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.add(egui::Button::new("Copy").shortcut_text("Ctrl+C")).clicked() {
                    actions.push(MenuBarAction::Copy);
                    ui.close_menu();
                }
                
                if ui.add(egui::Button::new("Paste").shortcut_text("Ctrl+V")).clicked() {
                    actions.push(MenuBarAction::Paste);
                    ui.close_menu();
                }
                
                if ui.add(egui::Button::new("Duplicate").shortcut_text("Ctrl+D")).clicked() {
                    actions.push(MenuBarAction::Duplicate);
                    ui.close_menu();
                }
            });
            
            // Entity Menu
            ui.menu_button("Entity", |ui| {
                if ui.button("Create Empty").clicked() {
                    actions.push(MenuBarAction::CreateEmpty);
                    ui.close_menu();
                }
                
                ui.separator();
                
                ui.menu_button("Create Primitive", |ui| {
                    if ui.button("Cube").clicked() {
                        actions.push(MenuBarAction::CreateCube);
                        ui.close_menu();
                    }
                    
                    if ui.button("Sphere").clicked() {
                        actions.push(MenuBarAction::CreateSphere);
                        ui.close_menu();
                    }
                    
                    if ui.button("Plane").clicked() {
                        actions.push(MenuBarAction::CreatePlane);
                        ui.close_menu();
                    }
                    
                    if ui.button("Cylinder").clicked() {
                        actions.push(MenuBarAction::CreateCylinder);
                        ui.close_menu();
                    }
                    
                    if ui.button("Cone").clicked() {
                        actions.push(MenuBarAction::CreateCone);
                        ui.close_menu();
                    }
                });
                
                ui.separator();
                
                if ui.add(egui::Button::new("Delete").shortcut_text("Delete")).clicked() {
                    actions.push(MenuBarAction::DeleteEntity);
                    ui.close_menu();
                }
            });
            
            // View Menu
            ui.menu_button("View", |ui| {
                if ui.checkbox(&mut state.hierarchy_visible, "Hierarchy").clicked() {
                    actions.push(MenuBarAction::ToggleHierarchy);
                }
                
                if ui.checkbox(&mut state.inspector_visible, "Inspector").clicked() {
                    actions.push(MenuBarAction::ToggleInspector);
                }
                
                if ui.checkbox(&mut state.console_visible, "Console").clicked() {
                    actions.push(MenuBarAction::ToggleConsole);
                }
                
                if ui.checkbox(&mut state.assets_visible, "Assets").clicked() {
                    actions.push(MenuBarAction::ToggleAssets);
                }
                
                if ui.checkbox(&mut state.scene_visible, "Scene View").clicked() {
                    actions.push(MenuBarAction::ToggleScene);
                }
            });
            
            // Help Menu
            ui.menu_button("Help", |ui| {
                if ui.button("About Praxis").clicked() {
                    actions.push(MenuBarAction::About);
                    ui.close_menu();
                }
                
                if ui.add(egui::Button::new("Documentation").shortcut_text("F1")).clicked() {
                    actions.push(MenuBarAction::Documentation);
                    ui.close_menu();
                }
            });
            
            ui.separator();
            
            // Play/Edit mode toggle
            let mode_text = match state.mode {
                EditorMode::Edit => "▶ Play",
                EditorMode::Play => "⏸ Edit",
            };
            
            if ui.button(mode_text).clicked() {
                actions.push(MenuBarAction::TogglePlayMode);
            }
            
            // Right-aligned status indicators
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if undo_system.is_some_and(|s| s.is_dirty()) {
                    ui.label(egui::RichText::new("● Unsaved").color(egui::Color32::from_rgb(255, 200, 0)));
                    ui.separator();
                }
                
                ui.label(format!("Mode: {:?}", state.mode));
            });
        });
    });
    
    actions
}

/// Checks for keyboard shortcuts and returns any triggered actions.
pub fn check_keyboard_shortcuts(ctx: &egui::Context) -> Vec<MenuBarAction> {
    let mut actions = Vec::new();
    
    // Only process shortcuts if not typing in a text field
    if ctx.wants_keyboard_input() {
        return actions;
    }
    
    ctx.input(|i| {
        let ctrl = i.modifiers.ctrl;
        let shift = i.modifiers.shift;
        let alt = i.modifiers.alt;
        
        // File shortcuts
        if ctrl && !shift && i.key_pressed(Key::N) {
            actions.push(MenuBarAction::NewScene);
        }
        if ctrl && !shift && i.key_pressed(Key::O) {
            actions.push(MenuBarAction::OpenScene);
        }
        if ctrl && !shift && i.key_pressed(Key::S) {
            actions.push(MenuBarAction::SaveScene);
        }
        if ctrl && shift && i.key_pressed(Key::S) {
            actions.push(MenuBarAction::SaveSceneAs);
        }
        if alt && i.key_pressed(Key::F4) {
            actions.push(MenuBarAction::Exit);
        }
        
        // Edit shortcuts (Undo/Redo handled by command_shortcuts system)
        if ctrl && !shift && i.key_pressed(Key::C) {
            actions.push(MenuBarAction::Copy);
        }
        if ctrl && !shift && i.key_pressed(Key::V) {
            actions.push(MenuBarAction::Paste);
        }
        if ctrl && !shift && i.key_pressed(Key::D) {
            actions.push(MenuBarAction::Duplicate);
        }
        
        // Entity shortcuts
        if !ctrl && !shift && i.key_pressed(Key::Delete) {
            actions.push(MenuBarAction::DeleteEntity);
        }
        
        // Help shortcuts
        if !ctrl && !shift && i.key_pressed(Key::F1) {
            actions.push(MenuBarAction::Documentation);
        }
    });
    
    actions
}

/// Handles menu bar actions by executing them.
pub fn handle_menu_action(
    action: MenuBarAction,
    state: &mut MenuBarState,
    undo_system: Option<&mut UndoRedoSystem>,
    world: Option<&mut World>,
) {
    use praxis_utils::{error, info};
    
    match action {
        // File actions
        MenuBarAction::NewScene => {
            info!("New scene requested");
        }
        MenuBarAction::OpenScene => {
            info!("Open scene requested");
        }
        MenuBarAction::SaveScene => {
            info!("Save scene requested");
            if let Some(system) = undo_system {
                system.mark_saved();
                info!("Scene marked as saved");
            }
        }
        MenuBarAction::SaveSceneAs => {
            info!("Save scene as requested");
        }
        MenuBarAction::Exit => {
            info!("Exit requested");
        }
        
        // Edit actions
        MenuBarAction::Undo => {
            if let (Some(system), Some(world)) = (undo_system, world) {
                if let Err(e) = system.undo(world) {
                    error!("Undo failed: {}", e);
                } else {
                    info!("Undo executed");
                }
            }
        }
        MenuBarAction::Redo => {
            if let (Some(system), Some(world)) = (undo_system, world) {
                if let Err(e) = system.redo(world) {
                    error!("Redo failed: {}", e);
                } else {
                    info!("Redo executed");
                }
            }
        }
        MenuBarAction::Copy => {
            info!("Copy requested");
        }
        MenuBarAction::Paste => {
            info!("Paste requested");
        }
        MenuBarAction::Duplicate => {
            info!("Duplicate requested");
        }
        
        // Entity actions
        MenuBarAction::CreateEmpty => {
            info!("Create empty entity requested");
        }
        MenuBarAction::CreateCube => {
            info!("Create cube requested");
        }
        MenuBarAction::CreateSphere => {
            info!("Create sphere requested");
        }
        MenuBarAction::CreatePlane => {
            info!("Create plane requested");
        }
        MenuBarAction::CreateCylinder => {
            info!("Create cylinder requested");
        }
        MenuBarAction::CreateCone => {
            info!("Create cone requested");
        }
        MenuBarAction::DeleteEntity => {
            info!("Delete entity requested");
        }
        
        // View actions
        MenuBarAction::ToggleHierarchy => {
            state.hierarchy_visible = !state.hierarchy_visible;
            info!("Hierarchy panel visibility: {}", state.hierarchy_visible);
        }
        MenuBarAction::ToggleInspector => {
            state.inspector_visible = !state.inspector_visible;
            info!("Inspector panel visibility: {}", state.inspector_visible);
        }
        MenuBarAction::ToggleConsole => {
            state.console_visible = !state.console_visible;
            info!("Console panel visibility: {}", state.console_visible);
        }
        MenuBarAction::ToggleAssets => {
            state.assets_visible = !state.assets_visible;
            info!("Assets panel visibility: {}", state.assets_visible);
        }
        MenuBarAction::ToggleScene => {
            state.scene_visible = !state.scene_visible;
            info!("Scene view panel visibility: {}", state.scene_visible);
        }
        
        // Help actions
        MenuBarAction::About => {
            info!("About dialog requested");
        }
        MenuBarAction::Documentation => {
            info!("Documentation requested");
        }
        
        // Mode toggle
        MenuBarAction::TogglePlayMode => {
            state.mode = state.mode.toggle();
            info!("Switched editor mode to {:?}", state.mode);
        }
    }
}
