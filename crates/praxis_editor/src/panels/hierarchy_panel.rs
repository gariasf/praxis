//! Hierarchy panel for displaying and managing the scene entity hierarchy.

use super::EditorPanel;
use crate::entity_operations::EntityOperations;
use crate::selection::{SelectionMode, SelectionSystem};
use crate::undo::UndoRedoSystem;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use egui::{CursorIcon, Id, Response, Sense, Ui};
use praxis_ecs::{Children, Name, Parent};
use std::collections::HashSet;

/// Panel for displaying and manipulating the scene hierarchy.
///
/// This panel provides a tree view of all entities in the scene with their parent-child
/// relationships. It supports:
/// - Entity tree visualization with proper indentation
/// - Drag-and-drop reparenting
/// - Entity creation/deletion with undo support
/// - Multi-selection integration
/// - Live updates as entities spawn/despawn
pub struct HierarchyPanel {
    title: String,
    /// Entity operations for create/delete with undo
    entity_ops: EntityOperations,
    /// Currently dragged entity for reparenting
    drag_entity: Option<Entity>,
    /// Entities that are expanded in the tree view
    expanded: HashSet<Entity>,
}

impl HierarchyPanel {
    /// Creates a new hierarchy panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: "Hierarchy".to_string(),
            entity_ops: EntityOperations::new(),
            drag_entity: None,
            expanded: HashSet::new(),
        }
    }

    /// Renders the hierarchy panel with access to ECS world and undo system.
    ///
    /// # Arguments
    ///
    /// * `ui` - The egui UI context
    /// * `world` - The ECS world containing entities
    /// * `undo_system` - The undo/redo system for tracking changes
    /// * `selection_system` - The selection system for highlighting selected entities
    pub fn ui_with_world(
        &mut self,
        ui: &mut Ui,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        selection_system: &mut SelectionSystem,
    ) {
        ui.heading("Scene Hierarchy");
        ui.separator();

        // Toolbar with create/delete buttons
        ui.horizontal(|ui| {
            if ui.button("➕ Create Entity").clicked() {
                if let Ok(entity) = self.entity_ops.create_entity_with_components(
                    world,
                    undo_system,
                    "New Entity",
                    Default::default(),
                ) {
                    selection_system.select_entity(entity, SelectionMode::Replace);
                }
            }

            // Delete button - only enabled if there are selected entities
            ui.add_enabled_ui(!selection_system.is_empty(), |ui| {
                if ui.button("🗑 Delete").clicked() {
                    let selected: Vec<Entity> = selection_system.selected_entities().collect();
                    if let Err(e) = self
                        .entity_ops
                        .delete_entities(world, undo_system, selected)
                    {
                        eprintln!("Failed to delete entities: {}", e);
                    }
                    selection_system.clear();
                }
            });
        });

        ui.separator();

        // Display entity tree
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                self.render_entity_tree(ui, world, selection_system, undo_system);
            });

        // Handle drag-and-drop completion
        self.handle_drag_drop_completion(ui, world, undo_system);
    }

    /// Renders the entity tree recursively.
    fn render_entity_tree(
        &mut self,
        ui: &mut Ui,
        world: &mut World,
        selection_system: &mut SelectionSystem,
        undo_system: &mut UndoRedoSystem,
    ) {
        // Find root entities (entities without Parent component)
        let mut root_entities: Vec<Entity> = world
            .iter_entities()
            .filter(|entity_ref| entity_ref.get::<Parent>().is_none())
            .map(|entity_ref| entity_ref.id())
            .collect();

        // Sort for consistent ordering
        root_entities.sort_by_key(|e| e.index());

        // Render each root entity and its children
        for entity in root_entities.clone() {
            self.render_entity_node(ui, world, entity, 0, selection_system, undo_system);
        }

        // Show a message if the scene is empty
        if root_entities.is_empty() {
            ui.colored_label(ui.visuals().weak_text_color(), "Scene is empty");
        }
    }

    /// Renders a single entity node in the tree.
    fn render_entity_node(
        &mut self,
        ui: &mut Ui,
        world: &mut World,
        entity: Entity,
        depth: usize,
        selection_system: &mut SelectionSystem,
        undo_system: &mut UndoRedoSystem,
    ) {
        // Get entity data (name, children) first, then drop the borrow
        let (name, has_children, children_vec) = {
            let Some(entity_ref) = world.get_entity(entity) else {
                return;
            };

            let name = entity_ref
                .get::<Name>()
                .map(|n| n.0.clone())
                .unwrap_or_else(|| format!("Entity {:?}", entity));

            let has_children = entity_ref
                .get::<Children>()
                .map(|c| !c.0.is_empty())
                .unwrap_or(false);

            let children_vec = entity_ref
                .get::<Children>()
                .map(|c| c.0.clone())
                .unwrap_or_default();

            (name, has_children, children_vec)
        };

        let is_expanded = self.expanded.contains(&entity);
        let is_selected = selection_system.is_selected(entity);

        // Indentation
        let indent = depth as f32 * 20.0;
        ui.horizontal(|ui| {
            ui.add_space(indent);

            // Expansion arrow for entities with children
            if has_children {
                let arrow = if is_expanded { "▼" } else { "▶" };
                if ui.small_button(arrow).clicked() {
                    if is_expanded {
                        self.expanded.remove(&entity);
                    } else {
                        self.expanded.insert(entity);
                    }
                }
            } else {
                // Add spacing to align with entities that have children
                ui.add_space(20.0);
            }

            // Entity label with selection highlighting
            let response = self.render_entity_label(ui, &name, is_selected, entity);

            // Handle selection
            self.handle_entity_interaction(
                response,
                entity,
                ui,
                selection_system,
                world,
                undo_system,
            );
        });

        // Render children if expanded
        if is_expanded && has_children {
            for child in children_vec {
                self.render_entity_node(ui, world, child, depth + 1, selection_system, undo_system);
            }
        }
    }

    /// Renders the entity label with drag-and-drop support.
    fn render_entity_label(
        &mut self,
        ui: &mut Ui,
        name: &str,
        is_selected: bool,
        entity: Entity,
    ) -> Response {
        let _id = Id::new(("entity_label", entity));

        // Determine visual style
        let (bg_color, text_color) = if is_selected {
            (
                ui.visuals().selection.bg_fill,
                ui.visuals().selection.stroke.color,
            )
        } else {
            (
                ui.visuals().widgets.inactive.bg_fill,
                ui.visuals().text_color(),
            )
        };

        // Create selectable label
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 20.0),
            Sense::click_and_drag(),
        );

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);

            // Background
            if response.hovered() {
                ui.painter().rect_filled(rect, 2.0, visuals.bg_fill);
            } else if is_selected {
                ui.painter().rect_filled(rect, 2.0, bg_color);
            }

            // Text
            ui.painter().text(
                rect.left_center() + egui::vec2(4.0, 0.0),
                egui::Align2::LEFT_CENTER,
                name,
                egui::FontId::default(),
                if is_selected {
                    text_color
                } else {
                    ui.visuals().text_color()
                },
            );

            // Drag-and-drop source
            if response.drag_started() {
                self.drag_entity = Some(entity);
            }

            // Visual feedback during drag
            if let Some(drag_entity) = self.drag_entity {
                if drag_entity == entity {
                    ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                }
            }

            // Drop target highlight
            if response.hovered() && self.drag_entity.is_some() && self.drag_entity != Some(entity)
            {
                ui.painter().rect_stroke(
                    rect,
                    2.0,
                    egui::Stroke::new(2.0, ui.visuals().selection.stroke.color),
                );
            }
        }

        response
    }

    /// Handles entity interaction (selection, drag-and-drop).
    fn handle_entity_interaction(
        &mut self,
        response: Response,
        entity: Entity,
        ui: &Ui,
        selection_system: &mut SelectionSystem,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
    ) {
        // Handle selection on click
        if response.clicked() {
            let mode = if ui.input(|i| i.modifiers.shift) {
                SelectionMode::Add
            } else if ui.input(|i| i.modifiers.ctrl) {
                SelectionMode::Toggle
            } else {
                SelectionMode::Replace
            };

            selection_system.select_entity(entity, mode);
        }

        // Handle drop target for reparenting
        if response.hovered() && ui.input(|i| i.pointer.any_released()) {
            if let Some(drag_entity) = self.drag_entity {
                if drag_entity != entity {
                    self.reparent_entity(drag_entity, Some(entity), world, undo_system);
                }
            }
        }
    }

    /// Handles drag-and-drop completion (drop on empty space to remove parent).
    fn handle_drag_drop_completion(
        &mut self,
        ui: &Ui,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
    ) {
        // If drag ends without hovering over an entity, remove parent
        if let Some(drag_entity) = self.drag_entity {
            if ui.input(|i| i.pointer.any_released()) {
                // Check if we're not hovering over any entity
                if !ui.ui_contains_pointer() {
                    // Only remove parent if entity currently has one
                    if world
                        .get_entity(drag_entity)
                        .and_then(|e| e.get::<Parent>())
                        .is_some()
                    {
                        self.reparent_entity(drag_entity, None, world, undo_system);
                    }
                }
                self.drag_entity = None;
            }
        }
    }

    /// Reparents an entity using the undo system.
    ///
    /// Returns true if reparenting was successful, false otherwise.
    fn reparent_entity(
        &mut self,
        child: Entity,
        new_parent: Option<Entity>,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
    ) -> bool {
        // Get current parent
        let old_parent = world
            .get_entity(child)
            .and_then(|e| e.get::<Parent>())
            .map(|p| p.0);

        // Don't do anything if parent hasn't changed
        if old_parent == new_parent {
            return false;
        }

        // Prevent circular parenting (child becoming parent of its ancestor)
        if let Some(new_parent) = new_parent {
            if self.is_ancestor_of(world, child, new_parent) {
                eprintln!("Cannot reparent: would create circular hierarchy");
                return false;
            }
        }

        // Create and execute SetParent command
        use crate::undo::SetParentCommand;
        let command = SetParentCommand::new(child, old_parent, new_parent);

        match undo_system.execute_command(world, Box::new(command)) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Failed to reparent entity: {}", e);
                false
            }
        }
    }

    /// Checks if potential_ancestor is an ancestor of entity.
    fn is_ancestor_of(&self, world: &World, entity: Entity, potential_ancestor: Entity) -> bool {
        if entity == potential_ancestor {
            return true;
        }

        let mut current = entity;
        while let Some(entity_ref) = world.get_entity(current) {
            if let Some(parent) = entity_ref.get::<Parent>() {
                if parent.0 == potential_ancestor {
                    return true;
                }
                current = parent.0;
            } else {
                break;
            }
        }

        false
    }

    /// Expands all entities in the tree.
    pub fn expand_all(&mut self, world: &World) {
        for entity_ref in world.iter_entities() {
            self.expanded.insert(entity_ref.id());
        }
    }

    /// Collapses all entities in the tree.
    pub fn collapse_all(&mut self) {
        self.expanded.clear();
    }

    /// Expands the tree to show a specific entity.
    pub fn expand_to_entity(&mut self, world: &World, entity: Entity) {
        let mut current = entity;
        while let Some(entity_ref) = world.get_entity(current) {
            if let Some(parent) = entity_ref.get::<Parent>() {
                self.expanded.insert(parent.0);
                current = parent.0;
            } else {
                break;
            }
        }
    }
}

impl Default for HierarchyPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for HierarchyPanel {
    fn title(&self) -> &str {
        &self.title
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Scene Hierarchy");
        ui.separator();

        ui.colored_label(
            ui.visuals().weak_text_color(),
            "⚠ Hierarchy requires World and UndoRedoSystem",
        );
        ui.label("Use ui_with_world() to render with full functionality.");

        ui.separator();
        ui.label("This panel displays:");
        ui.label("• Entity tree with parent-child relationships");
        ui.label("• Drag-and-drop reparenting");
        ui.label("• Entity creation/deletion buttons");
        ui.label("• Multi-selection support");
    }
}
