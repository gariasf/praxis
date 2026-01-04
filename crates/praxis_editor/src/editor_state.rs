//! Core editor state management and coordination.

use crate::editor_mode::EditorMode;
use crate::menu_bar::{
    check_keyboard_shortcuts, handle_menu_action, render_menu_bar, MenuBarState,
};
use crate::panels::{
    AssetsPanel, ConsolePanel, EditorPanel, HierarchyPanel, InspectorPanel, LogBuffer,
    SceneViewPanel,
};
use crate::play_mode::PlayModeSystem;
use crate::selection::SelectionSystem;
use crate::toolbar::{handle_toolbar_action, render_toolbar, ToolbarState};
use crate::UndoRedoSystem;
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use praxis_ecs::World;
use praxis_utils::{error, info};

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
    /// Menu bar state.
    menu_bar_state: MenuBarState,
    /// Toolbar state.
    toolbar_state: ToolbarState,
    /// Play mode system for managing edit/play transitions.
    play_mode_system: PlayModeSystem,
}

impl EditorState {
    /// Creates a new editor state with default layout.
    #[must_use]
    pub fn new() -> Self {
        let mut dock_state = DockState::new(vec![EditorTab::Scene]);

        let tree = dock_state.main_surface_mut();

        let [scene, right] = tree.split_right(NodeIndex::root(), 0.75, vec![EditorTab::Scene]);

        let [_right_top, right_bottom] = tree.split_below(right, 0.7, vec![EditorTab::Inspector]);

        let [left, _scene] = tree.split_left(scene, 0.2, vec![EditorTab::Hierarchy]);

        tree.split_below(left, 0.6, vec![EditorTab::Assets]);

        tree.split_below(right_bottom, 0.5, vec![EditorTab::Console]);

        Self {
            mode: EditorMode::default(),
            dock_state,
            scene_panel: SceneViewPanel::new(),
            hierarchy_panel: HierarchyPanel::new(),
            inspector_panel: InspectorPanel::new(),
            console_panel: ConsolePanel::new(),
            assets_panel: AssetsPanel::new(),
            visible: true,
            menu_bar_state: MenuBarState::new(),
            toolbar_state: ToolbarState::new(),
            play_mode_system: PlayModeSystem::new(),
        }
    }

    /// Creates a new editor state with a shared log buffer for console integration.
    #[must_use]
    pub fn with_log_buffer(log_buffer: LogBuffer) -> Self {
        let mut dock_state = DockState::new(vec![EditorTab::Scene]);

        let tree = dock_state.main_surface_mut();

        let [scene, right] = tree.split_right(NodeIndex::root(), 0.75, vec![EditorTab::Scene]);

        let [_right_top, right_bottom] = tree.split_below(right, 0.7, vec![EditorTab::Inspector]);

        let [left, _scene] = tree.split_left(scene, 0.2, vec![EditorTab::Hierarchy]);

        tree.split_below(left, 0.6, vec![EditorTab::Assets]);

        tree.split_below(right_bottom, 0.5, vec![EditorTab::Console]);

        Self {
            mode: EditorMode::default(),
            dock_state,
            scene_panel: SceneViewPanel::new(),
            hierarchy_panel: HierarchyPanel::new(),
            inspector_panel: InspectorPanel::new(),
            console_panel: ConsolePanel::with_buffer(log_buffer),
            assets_panel: AssetsPanel::new(),
            visible: true,
            menu_bar_state: MenuBarState::new(),
            toolbar_state: ToolbarState::new(),
            play_mode_system: PlayModeSystem::new(),
        }
    }

    /// Gets the log buffer from the console panel
    #[must_use]
    pub fn log_buffer(&self) -> &LogBuffer {
        self.console_panel.log_buffer()
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

    /// Gets a reference to the menu bar state.
    #[must_use]
    pub const fn menu_bar_state(&self) -> &MenuBarState {
        &self.menu_bar_state
    }

    /// Gets a mutable reference to the menu bar state.
    #[must_use]
    pub fn menu_bar_state_mut(&mut self) -> &mut MenuBarState {
        &mut self.menu_bar_state
    }

    /// Gets a reference to the toolbar state.
    #[must_use]
    pub const fn toolbar_state(&self) -> &ToolbarState {
        &self.toolbar_state
    }

    /// Gets a mutable reference to the toolbar state.
    #[must_use]
    pub fn toolbar_state_mut(&mut self) -> &mut ToolbarState {
        &mut self.toolbar_state
    }

    /// Gets a reference to the play mode system.
    #[must_use]
    pub const fn play_mode_system(&self) -> &PlayModeSystem {
        &self.play_mode_system
    }

    /// Gets a mutable reference to the play mode system.
    #[must_use]
    pub fn play_mode_system_mut(&mut self) -> &mut PlayModeSystem {
        &mut self.play_mode_system
    }

    /// Enters play mode by taking a snapshot and transitioning state.
    ///
    /// # Errors
    ///
    /// Returns an error if snapshot or state transition fails.
    pub fn enter_play_mode(&mut self, world: &mut praxis_ecs::World) -> praxis_utils::Result<()> {
        self.play_mode_system.enter_play_mode(world)?;
        self.mode = self.play_mode_system.editor_mode();
        self.toolbar_state.editor_mode = self.mode;
        self.menu_bar_state.mode = self.mode;
        Ok(())
    }

    /// Exits play mode and restores the snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error if restoration fails.
    pub fn exit_play_mode(&mut self, world: &mut praxis_ecs::World) -> praxis_utils::Result<()> {
        self.play_mode_system.exit_play_mode(world)?;
        self.mode = self.play_mode_system.editor_mode();
        self.toolbar_state.editor_mode = self.mode;
        self.menu_bar_state.mode = self.mode;
        Ok(())
    }

    /// Pauses play mode.
    pub fn pause_play_mode(&mut self) {
        self.play_mode_system.pause_play_mode();
        self.mode = self.play_mode_system.editor_mode();
        self.toolbar_state.editor_mode = self.mode;
        self.menu_bar_state.mode = self.mode;
    }

    /// Resumes play mode from paused state.
    pub fn resume_play_mode(&mut self) {
        self.play_mode_system.resume_play_mode();
        self.mode = self.play_mode_system.editor_mode();
        self.toolbar_state.editor_mode = self.mode;
        self.menu_bar_state.mode = self.mode;
    }

    /// Renders the editor UI.
    ///
    /// # Arguments
    /// * `ctx` - The egui context
    /// * `undo_system` - Optional mutable reference to the undo/redo system for menu integration
    /// * `world` - Optional mutable reference to the ECS world for executing undo/redo commands
    /// * `selection_system` - Optional mutable reference to the selection system
    /// * `render_context` - Optional mutable reference to the render context for panel access
    pub fn ui(
        &mut self,
        ctx: &egui::Context,
        mut undo_system: Option<&mut UndoRedoSystem>,
        mut world: Option<&mut World>,
        selection_system: Option<&mut SelectionSystem>,
        render_context: Option<&mut praxis_graphics::RenderContext>,
    ) {
        if !self.visible {
            return;
        }

        // Update menu bar state with current mode
        self.menu_bar_state.mode = self.mode;

        // Render menu bar and collect actions
        let mut menu_actions =
            render_menu_bar(ctx, &mut self.menu_bar_state, undo_system.as_deref());

        // Check for keyboard shortcuts
        menu_actions.extend(check_keyboard_shortcuts(ctx));

        // Handle all menu actions
        for action in menu_actions {
            handle_menu_action(
                action,
                &mut self.menu_bar_state,
                undo_system.as_deref_mut(),
                world.as_mut().map(|w| w.inner_mut()),
            );
        }

        // Sync mode back to EditorState from menu bar
        self.mode = self.menu_bar_state.mode;

        // Update toolbar state with current mode and sync gizmo state
        self.toolbar_state.editor_mode = self.mode;

        // Render toolbar and collect actions
        let toolbar_actions = render_toolbar(ctx, &mut self.toolbar_state);

        // Handle all toolbar actions
        for action in toolbar_actions {
            use crate::ToolbarAction;
            match action {
                ToolbarAction::Play => {
                    // Enter play mode if we have a world
                    if let Some(w) = world.as_mut() {
                        if let Err(e) = self.enter_play_mode(w) {
                            error!("Failed to enter play mode: {}", e);
                        }
                    } else {
                        info!("Cannot enter play mode without world");
                    }
                }
                ToolbarAction::Pause => {
                    self.pause_play_mode();
                }
                ToolbarAction::Stop => {
                    // Exit play mode if we have a world
                    if let Some(w) = world.as_mut() {
                        if let Err(e) = self.exit_play_mode(w) {
                            error!("Failed to exit play mode: {}", e);
                        }
                    } else {
                        info!("Cannot exit play mode without world");
                    }
                }
                _ => {
                    // Handle other actions normally
                    handle_toolbar_action(action, &mut self.toolbar_state);
                }
            }
        }

        // Sync mode back to EditorState from toolbar (for non-play actions)
        if !self.play_mode_system.is_playing() {
            self.mode = self.toolbar_state.editor_mode;
        }

        // Update scene panel border color based on play mode
        self.scene_panel
            .set_border_color(self.play_mode_system.viewport_border_color_egui());

        self.render_dock_area(ctx, world, undo_system, selection_system, render_context);
    }

    fn render_dock_area(
        &mut self,
        ctx: &egui::Context,
        world: Option<&mut World>,
        undo_system: Option<&mut UndoRedoSystem>,
        selection_system: Option<&mut SelectionSystem>,
        render_context: Option<&mut praxis_graphics::RenderContext>,
    ) {
        let mut tab_viewer = EditorTabViewer {
            scene_panel: &mut self.scene_panel,
            hierarchy_panel: &mut self.hierarchy_panel,
            inspector_panel: &mut self.inspector_panel,
            console_panel: &mut self.console_panel,
            assets_panel: &mut self.assets_panel,
            world,
            undo_system,
            selection_system,
            render_context,
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
    world: Option<&'a mut World>,
    undo_system: Option<&'a mut UndoRedoSystem>,
    selection_system: Option<&'a mut SelectionSystem>,
    render_context: Option<&'a mut praxis_graphics::RenderContext>,
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
            EditorTab::Scene => {
                let world_ref = self.world.as_ref().map(|w| w as &World);
                self.scene_panel
                    .ui(ui, world_ref, self.render_context.as_deref_mut());
            }
            EditorTab::Hierarchy => {
                // Check if we have all required resources
                if let (Some(world), Some(undo_system), Some(selection_system)) = (
                    self.world.as_mut().map(|w| w.inner_mut()),
                    self.undo_system.as_deref_mut(),
                    self.selection_system.as_deref_mut(),
                ) {
                    self.hierarchy_panel
                        .ui_with_world(ui, world, undo_system, selection_system);
                } else {
                    // Fallback to basic UI
                    let world_ref = self.world.as_ref().map(|w| w as &World);
                    self.hierarchy_panel
                        .ui(ui, world_ref, self.render_context.as_deref_mut());
                }
            }
            EditorTab::Inspector => {
                if let Some(world) = &mut self.world {
                    self.inspector_panel.ui_with_world(ui, world);
                } else {
                    let world_ref = self.world.as_ref().map(|w| w as &World);
                    self.inspector_panel
                        .ui(ui, world_ref, self.render_context.as_deref_mut());
                }
            }
            EditorTab::Console => {
                let world_ref = self.world.as_ref().map(|w| w as &World);
                self.console_panel
                    .ui(ui, world_ref, self.render_context.as_deref_mut());
            }
            EditorTab::Assets => {
                let world_ref = self.world.as_ref().map(|w| w as &World);
                self.assets_panel
                    .ui(ui, world_ref, self.render_context.as_deref_mut());
            }
        }
    }
}
