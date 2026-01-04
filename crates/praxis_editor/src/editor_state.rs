//! Core editor state management and coordination.

use crate::editor_mode::EditorMode;
use crate::menu_bar::{check_keyboard_shortcuts, handle_menu_action, render_menu_bar, MenuBarState};
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
    /// Menu bar state.
    menu_bar_state: MenuBarState,
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
            menu_bar_state: MenuBarState::new(),
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

    /// Renders the editor UI.
    /// 
    /// # Arguments
    /// * `ctx` - The egui context
    /// * `undo_system` - Optional mutable reference to the undo/redo system for menu integration
    /// * `world` - Optional mutable reference to the ECS world for executing undo/redo commands
    pub fn ui(&mut self, ctx: &egui::Context, mut undo_system: Option<&mut UndoRedoSystem>, mut world: Option<&mut World>) {
        if !self.visible {
            return;
        }

        // Update menu bar state with current mode
        self.menu_bar_state.mode = self.mode;

        // Render menu bar and collect actions
        let mut actions = render_menu_bar(ctx, &mut self.menu_bar_state, undo_system.as_deref());
        
        // Check for keyboard shortcuts
        actions.extend(check_keyboard_shortcuts(ctx));

        // Handle all actions
        for action in actions {
            handle_menu_action(action, &mut self.menu_bar_state, undo_system.as_deref_mut(), world.as_deref_mut());
        }

        // Sync mode back to EditorState
        self.mode = self.menu_bar_state.mode;

        self.render_dock_area(ctx);
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
