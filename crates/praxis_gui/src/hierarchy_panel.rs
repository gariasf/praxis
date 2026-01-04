//! Hierarchy panel for visualizing and manipulating the ECS scene graph.

use praxis_ecs::{Children, Entity, Name, Parent, World};
use std::collections::HashSet;

/// Selection state resource tracking selected entities.
#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    /// Currently selected entities (supports multi-selection).
    pub selected_entities: HashSet<Entity>,
    /// Primary selection for single-entity operations.
    pub primary_selection: Option<Entity>,
}

impl SelectionState {
    /// Creates a new empty selection state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Selects a single entity (clears previous selection).
    pub fn select_single(&mut self, entity: Entity) {
        self.selected_entities.clear();
        self.selected_entities.insert(entity);
        self.primary_selection = Some(entity);
    }

    /// Toggles entity selection (for multi-select).
    pub fn toggle_selection(&mut self, entity: Entity) {
        if self.selected_entities.contains(&entity) {
            self.selected_entities.remove(&entity);
            if self.primary_selection == Some(entity) {
                self.primary_selection = self.selected_entities.iter().next().copied();
            }
        } else {
            self.selected_entities.insert(entity);
            if self.primary_selection.is_none() {
                self.primary_selection = Some(entity);
            }
        }
    }

    /// Adds entity to selection.
    pub fn add_to_selection(&mut self, entity: Entity) {
        self.selected_entities.insert(entity);
        if self.primary_selection.is_none() {
            self.primary_selection = Some(entity);
        }
    }

    /// Clears all selections.
    pub fn clear(&mut self) {
        self.selected_entities.clear();
        self.primary_selection = None;
    }

    /// Checks if an entity is selected.
    pub fn is_selected(&self, entity: Entity) -> bool {
        self.selected_entities.contains(&entity)
    }

    /// Gets the number of selected entities.
    pub fn selection_count(&self) -> usize {
        self.selected_entities.len()
    }
}

/// Context menu state for entity operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextMenuTarget {
    Entity(Entity),
    Background,
}

/// Hierarchy panel for visualizing and editing the scene graph.
pub struct HierarchyPanel {
    /// Whether the panel is visible.
    pub visible: bool,
    /// Search filter for entity names.
    search_filter: String,
    /// Entity being dragged (for reparenting).
    drag_source: Option<Entity>,
    /// Context menu state.
    context_menu: Option<ContextMenuTarget>,
    /// Collapsed state for each entity.
    collapsed_entities: HashSet<Entity>,
    /// Selection state.
    pub selection_state: SelectionState,
}

impl Default for HierarchyPanel {
    fn default() -> Self {
        Self {
            visible: true,
            search_filter: String::new(),
            drag_source: None,
            context_menu: None,
            collapsed_entities: HashSet::new(),
            selection_state: SelectionState::new(),
        }
    }
}

impl HierarchyPanel {
    /// Creates a new hierarchy panel.
    pub fn new() -> Self {
        Self::default()
    }

    /// Renders the hierarchy panel UI.
    pub fn render(&mut self, ctx: &egui::Context, world: &mut World) {
        if !self.visible {
            return;
        }

        egui::Window::new("Hierarchy")
            .default_pos(egui::pos2(10.0, 50.0))
            .default_size(egui::vec2(300.0, 600.0))
            .resizable(true)
            .show(ctx, |ui| {
                self.render_toolbar(ui);
                ui.separator();
                self.render_search_bar(ui);
                ui.separator();
                self.render_entity_tree(ui, world);
            });

        // Handle context menu
        self.handle_context_menu(ctx, world);
    }

    /// Renders the toolbar with action buttons.
    fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("➕ Create Empty").clicked() {
                self.context_menu = Some(ContextMenuTarget::Background);
            }

            if ui.button("Clear Selection").clicked() {
                self.selection_state.clear();
            }
        });
    }

    /// Renders the search bar.
    fn render_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.search_filter);
            if ui.button("✖").clicked() {
                self.search_filter.clear();
            }
        });
    }

    /// Renders the entity tree view.
    fn render_entity_tree(&mut self, ui: &mut egui::Ui, world: &mut World) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Background click to deselect
            let response = ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::click());
            if response.clicked() {
                self.selection_state.clear();
            }
            if response.secondary_clicked() {
                self.context_menu = Some(ContextMenuTarget::Background);
            }

            // Find root entities (entities without parents)
            let mut root_entities = Vec::new();
            let mut query = world.inner_mut().query::<(Entity, Option<&Parent>)>();
            for (entity, parent) in query.iter(world.inner()) {
                if parent.is_none() {
                    root_entities.push(entity);
                }
            }

            // Sort root entities by name
            self.sort_entities_by_name(&mut root_entities, world);

            // Render each root entity and its children
            for entity in root_entities {
                self.render_entity_node(ui, world, entity, 0);
            }
        });
    }

    /// Renders a single entity node with its children.
    fn render_entity_node(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        entity: Entity,
        depth: usize,
    ) {
        // Check if entity still exists
        if world.inner().get_entity(entity).is_none() {
            return;
        }

        // Get entity name
        let entity_name = self.get_entity_name(world, entity);

        // Apply search filter
        if !self.search_filter.is_empty() {
            let matches_filter = entity_name
                .to_lowercase()
                .contains(&self.search_filter.to_lowercase());
            if !matches_filter {
                return;
            }
        }

        // Check if entity has children
        let has_children = world
            .inner_mut()
            .query::<&Children>()
            .get(world.inner(), entity)
            .map(|c| !c.is_empty())
            .unwrap_or(false);

        let is_collapsed = self.collapsed_entities.contains(&entity);
        let is_selected = self.selection_state.is_selected(entity);

        ui.horizontal(|ui| {
            // Indentation
            ui.add_space(depth as f32 * 20.0);

            // Collapse/expand arrow for entities with children
            if has_children {
                let arrow = if is_collapsed { "▶" } else { "▼" };
                if ui.small_button(arrow).clicked() {
                    if is_collapsed {
                        self.collapsed_entities.remove(&entity);
                    } else {
                        self.collapsed_entities.insert(entity);
                    }
                }
            } else {
                ui.add_space(20.0);
            }

            // Entity label with selection highlighting
            let label_text = format!("{entity_name} ({entity:?})");
            let label = if is_selected {
                egui::RichText::new(label_text)
                    .strong()
                    .color(egui::Color32::LIGHT_BLUE)
            } else {
                egui::RichText::new(label_text)
            };

            let response = ui.selectable_label(is_selected, label);

            // Handle clicks
            if response.clicked() {
                if ui.input(|i| i.modifiers.ctrl || i.modifiers.command) {
                    self.selection_state.toggle_selection(entity);
                } else if ui.input(|i| i.modifiers.shift) {
                    self.selection_state.add_to_selection(entity);
                } else {
                    self.selection_state.select_single(entity);
                }
            }

            // Right-click context menu
            if response.secondary_clicked() {
                self.context_menu = Some(ContextMenuTarget::Entity(entity));
            }

            // Drag and drop for reparenting
            if response.drag_started() {
                self.drag_source = Some(entity);
            }

            if let Some(drag_source) = self.drag_source {
                if response.hovered() && entity != drag_source {
                    ui.painter().rect_stroke(
                        response.rect,
                        2.0,
                        egui::Stroke::new(2.0, egui::Color32::YELLOW),
                    );
                }
            }

            if response.hovered() && ui.input(|i| i.pointer.any_released()) {
                if let Some(drag_source) = self.drag_source {
                    if entity != drag_source {
                        // Perform reparenting
                        self.reparent_entity(world, drag_source, Some(entity));
                    }
                    self.drag_source = None;
                }
            }
        });

        // Render children if not collapsed
        if !is_collapsed && has_children {
            if let Ok(children) = world
                .inner_mut()
                .query::<&Children>()
                .get(world.inner(), entity)
            {
                let mut child_entities: Vec<Entity> = children.iter().copied().collect();
                self.sort_entities_by_name(&mut child_entities, world);

                for child in child_entities {
                    self.render_entity_node(ui, world, child, depth + 1);
                }
            }
        }
    }

    /// Gets the display name for an entity.
    fn get_entity_name(&self, world: &mut World, entity: Entity) -> String {
        world
            .inner_mut()
            .query::<&Name>()
            .get(world.inner(), entity)
            .map(|n| n.as_str().to_string())
            .unwrap_or_else(|_| format!("Entity {entity:?}"))
    }

    /// Sorts entities by name.
    fn sort_entities_by_name(&self, entities: &mut [Entity], world: &mut World) {
        entities.sort_by_cached_key(|entity| self.get_entity_name(world, *entity));
    }

    /// Handles the context menu.
    fn handle_context_menu(&mut self, ctx: &egui::Context, world: &mut World) {
        let mut menu_open = self.context_menu.is_some();

        if let Some(target) = self.context_menu {
            egui::Window::new("Context Menu")
                .fixed_pos(ctx.pointer_hover_pos().unwrap_or_default())
                .title_bar(false)
                .resizable(false)
                .show(ctx, |ui| match target {
                    ContextMenuTarget::Entity(entity) => {
                        if ui.button("Create Child").clicked() {
                            self.create_child_entity(world, entity);
                            menu_open = false;
                        }
                        if ui.button("Duplicate").clicked() {
                            self.duplicate_entity(world, entity);
                            menu_open = false;
                        }
                        if ui.button("Delete").clicked() {
                            self.delete_entity(world, entity);
                            menu_open = false;
                        }
                        ui.separator();
                        if ui.button("Remove Parent").clicked() {
                            self.remove_parent(world, entity);
                            menu_open = false;
                        }
                    }
                    ContextMenuTarget::Background => {
                        if ui.button("Create Entity").clicked() {
                            self.create_root_entity(world);
                            menu_open = false;
                        }
                        if ui.button("Create Camera").clicked() {
                            self.create_camera_entity(world);
                            menu_open = false;
                        }
                        if ui.button("Create Light").clicked() {
                            self.create_light_entity(world);
                            menu_open = false;
                        }
                    }
                });
        }

        if !menu_open {
            self.context_menu = None;
        }

        // Close menu on outside click
        if ctx.input(|i| i.pointer.any_pressed()) && self.context_menu.is_some() {
            self.context_menu = None;
        }
    }

    /// Creates a new root entity.
    fn create_root_entity(&self, world: &mut World) {
        use praxis_ecs::{GlobalTransform, Transform};

        world.spawn((
            Name::new("New Entity"),
            Transform::default(),
            GlobalTransform::default(),
        ));
    }

    /// Creates a child entity under the specified parent.
    fn create_child_entity(&self, world: &mut World, parent: Entity) {
        use praxis_ecs::{GlobalTransform, Transform};

        world.spawn((
            Name::new("New Child"),
            Transform::default(),
            GlobalTransform::default(),
            Parent(parent),
        ));
    }

    /// Creates a camera entity.
    fn create_camera_entity(&self, world: &mut World) {
        use praxis_ecs::{
            Camera, CameraMatrices, GlobalTransform, PerspectiveProjection, Transform,
        };

        world.spawn((
            Name::new("Camera"),
            Transform::from_xyz(0.0, 5.0, 10.0),
            GlobalTransform::default(),
            Camera::default(),
            PerspectiveProjection::default(),
            CameraMatrices::default(),
        ));
    }

    /// Creates a point light entity.
    fn create_light_entity(&self, world: &mut World) {
        use praxis_ecs::{GlobalTransform, PointLight, Transform};
        use praxis_math::Vec3;

        world.spawn((
            Name::new("Point Light"),
            Transform::from_xyz(0.0, 5.0, 0.0),
            GlobalTransform::default(),
            PointLight::new(Vec3::ONE, 10.0, 20.0),
        ));
    }

    /// Duplicates an entity and its components.
    fn duplicate_entity(&self, world: &mut World, entity: Entity) {
        use praxis_ecs::{GlobalTransform, Transform};

        // Get components from original entity
        let name = world
            .inner_mut()
            .query::<&Name>()
            .get(world.inner(), entity)
            .map(|n| format!("{} (Copy)", n.as_str()))
            .unwrap_or_else(|_| "Duplicated Entity".to_string());

        let transform = world
            .inner_mut()
            .query::<&Transform>()
            .get(world.inner(), entity)
            .copied()
            .unwrap_or_default();

        let parent = world
            .inner_mut()
            .query::<&Parent>()
            .get(world.inner(), entity)
            .copied()
            .ok();

        // Create new entity with copied components
        let mut entity_builder =
            world
                .inner_mut()
                .spawn((Name::new(name), transform, GlobalTransform::default()));

        if let Some(parent) = parent {
            entity_builder.insert(parent);
        }
    }

    /// Deletes an entity and optionally its children.
    fn delete_entity(&mut self, world: &mut World, entity: Entity) {
        // Remove from parent's children list
        if let Ok(parent) = world
            .inner_mut()
            .query::<&Parent>()
            .get(world.inner(), entity)
        {
            let parent_entity = parent.0;
            if let Ok(mut children) = world
                .inner_mut()
                .query::<&mut Children>()
                .get_mut(world.inner_mut(), parent_entity)
            {
                children.remove(entity);
            }
        }

        // Delete children recursively
        if let Ok(children) = world
            .inner_mut()
            .query::<&Children>()
            .get(world.inner(), entity)
        {
            let child_entities: Vec<Entity> = children.iter().copied().collect();
            for child in child_entities {
                self.delete_entity(world, child);
            }
        }

        // Remove from selection
        self.selection_state.selected_entities.remove(&entity);
        if self.selection_state.primary_selection == Some(entity) {
            self.selection_state.primary_selection = self
                .selection_state
                .selected_entities
                .iter()
                .next()
                .copied();
        }

        // Delete the entity
        world.inner_mut().despawn(entity);
    }

    /// Removes parent from an entity.
    fn remove_parent(&self, world: &mut World, entity: Entity) {
        if let Ok(parent) = world
            .inner_mut()
            .query::<&Parent>()
            .get(world.inner(), entity)
        {
            let parent_entity = parent.0;

            // Remove from parent's children list
            if let Ok(mut children) = world
                .inner_mut()
                .query::<&mut Children>()
                .get_mut(world.inner_mut(), parent_entity)
            {
                children.remove(entity);
            }
        }

        // Remove Parent component
        world.inner_mut().entity_mut(entity).remove::<Parent>();
    }

    /// Reparents an entity to a new parent.
    fn reparent_entity(&self, world: &mut World, entity: Entity, new_parent: Option<Entity>) {
        // Check for circular dependency
        if let Some(parent) = new_parent {
            if entity == parent || self.is_ancestor_of(world, entity, parent) {
                return; // Would create circular reference
            }
        }

        // Remove from old parent
        if let Ok(old_parent) = world
            .inner_mut()
            .query::<&Parent>()
            .get(world.inner(), entity)
        {
            let old_parent_entity = old_parent.0;
            if let Ok(mut children) = world
                .inner_mut()
                .query::<&mut Children>()
                .get_mut(world.inner_mut(), old_parent_entity)
            {
                children.remove(entity);
            }
        }

        // Set new parent
        if let Some(parent) = new_parent {
            world.inner_mut().entity_mut(entity).insert(Parent(parent));

            // Add to new parent's children (will be handled by sync system)
        } else {
            world.inner_mut().entity_mut(entity).remove::<Parent>();
        }
    }

    /// Checks if an entity is an ancestor of another entity.
    fn is_ancestor_of(
        &self,
        world: &mut World,
        potential_ancestor: Entity,
        entity: Entity,
    ) -> bool {
        let mut current = entity;
        loop {
            if let Ok(parent) = world
                .inner_mut()
                .query::<&Parent>()
                .get(world.inner(), current)
            {
                if parent.0 == potential_ancestor {
                    return true;
                }
                current = parent.0;
            } else {
                return false;
            }
        }
    }

    /// Toggles the visibility of the hierarchy panel.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Sets the visibility of the hierarchy panel.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}
