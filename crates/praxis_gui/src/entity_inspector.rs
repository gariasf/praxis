//! Entity inspector for viewing and editing ECS component data.

use praxis_ecs::{
    Camera, Children, Entity, GlobalTransform, MeshHandle, Name, Parent, PerspectiveProjection,
    PointLight, Transform, World,
};
use praxis_math::Quat;

/// Entity inspector state and configuration.
pub struct EntityInspector {
    /// Whether the inspector is visible.
    pub visible: bool,
    /// Currently selected entity.
    pub selected_entity: Option<Entity>,
    /// Search filter for entity list.
    search_filter: String,
}

impl Default for EntityInspector {
    fn default() -> Self {
        Self {
            visible: true,
            selected_entity: None,
            search_filter: String::new(),
        }
    }
}

impl EntityInspector {
    /// Creates a new entity inspector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Renders the entity inspector UI.
    pub fn render(&mut self, ctx: &egui::Context, world: &mut World) {
        if !self.visible {
            return;
        }

        egui::Window::new("Entity Inspector")
            .default_pos(egui::pos2(300.0, 50.0))
            .default_size(egui::vec2(400.0, 600.0))
            .resizable(true)
            .show(ctx, |ui| {
                self.render_entity_list(ui, world);
                ui.separator();
                self.render_selected_entity(ui, world);
            });
    }

    /// Renders the entity list.
    fn render_entity_list(&mut self, ui: &mut egui::Ui, world: &mut World) {
        ui.heading("Entities");

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.search_filter);
        });

        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                let mut query = world.query::<(Entity, Option<&Name>)>();

                for (entity, name) in query.iter(world.inner()) {
                    let entity_name = if let Some(name) = name {
                        name.as_str()
                    } else {
                        "Unnamed Entity"
                    };

                    if !self.search_filter.is_empty()
                        && !entity_name
                            .to_lowercase()
                            .contains(&self.search_filter.to_lowercase())
                    {
                        continue;
                    }

                    let label = format!("{entity_name} (ID: {entity:?})");
                    let is_selected = self.selected_entity == Some(entity);

                    if ui.selectable_label(is_selected, label).clicked() {
                        self.selected_entity = Some(entity);
                    }
                }
            });
    }

    /// Renders details for the selected entity.
    fn render_selected_entity(&mut self, ui: &mut egui::Ui, world: &mut World) {
        let Some(entity) = self.selected_entity else {
            ui.label("No entity selected");
            return;
        };

        if world.inner().get_entity(entity).is_none() {
            ui.label("Selected entity no longer exists");
            self.selected_entity = None;
            return;
        }

        ui.heading("Components");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            self.render_name_component(ui, world, entity);
            self.render_transform_component(ui, world, entity);
            self.render_global_transform_component(ui, world, entity);
            self.render_mesh_component(ui, world, entity);
            self.render_camera_component(ui, world, entity);
            self.render_light_component(ui, world, entity);
            self.render_hierarchy_component(ui, world, entity);
        });
    }

    /// Renders the Name component editor.
    fn render_name_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut query = world.query::<&Name>();
        if let Ok(name) = query.get(world.inner(), entity) {
            egui::CollapsingHeader::new("Name")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(format!("Name: {}", name.as_str()));
                });
        }
    }

    /// Renders the Transform component editor.
    fn render_transform_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut query = world.query::<&mut Transform>();
        if let Ok(mut transform) = query.get_mut(world.inner_mut(), entity) {
            egui::CollapsingHeader::new("Transform")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label("Translation:");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut transform.translation.x).speed(0.1));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut transform.translation.y).speed(0.1));
                        ui.label("Z:");
                        ui.add(egui::DragValue::new(&mut transform.translation.z).speed(0.1));
                    });

                    ui.label("Rotation (Euler):");
                    let (mut x, mut y, mut z) = transform.rotation.to_euler(praxis_math::EulerRot::XYZ);
                    x = x.to_degrees();
                    y = y.to_degrees();
                    z = z.to_degrees();
                    
                    let mut changed = false;
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        changed |= ui.add(egui::DragValue::new(&mut x).speed(1.0)).changed();
                        ui.label("Y:");
                        changed |= ui.add(egui::DragValue::new(&mut y).speed(1.0)).changed();
                        ui.label("Z:");
                        changed |= ui.add(egui::DragValue::new(&mut z).speed(1.0)).changed();
                    });

                    if changed {
                        transform.rotation = Quat::from_euler(
                            praxis_math::EulerRot::XYZ,
                            x.to_radians(),
                            y.to_radians(),
                            z.to_radians(),
                        );
                    }

                    ui.label("Scale:");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut transform.scale.x).speed(0.01));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut transform.scale.y).speed(0.01));
                        ui.label("Z:");
                        ui.add(egui::DragValue::new(&mut transform.scale.z).speed(0.01));
                    });
                });
        }
    }

    /// Renders the GlobalTransform component (read-only).
    fn render_global_transform_component(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        entity: Entity,
    ) {
        let mut query = world.query::<&GlobalTransform>();
        if let Ok(global_transform) = query.get(world.inner(), entity) {
            egui::CollapsingHeader::new("Global Transform")
                .default_open(false)
                .show(ui, |ui| {
                    let translation = global_transform.translation();
                    ui.label(format!(
                        "World Position: [{:.2}, {:.2}, {:.2}]",
                        translation.x, translation.y, translation.z
                    ));

                    let scale = global_transform.scale();
                    ui.label(format!(
                        "World Scale: [{:.2}, {:.2}, {:.2}]",
                        scale.x, scale.y, scale.z
                    ));
                });
        }
    }

    /// Renders the MeshHandle component.
    fn render_mesh_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut query = world.query::<&MeshHandle>();
        if let Ok(mesh_handle) = query.get(world.inner(), entity) {
            egui::CollapsingHeader::new("Mesh")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(format!("Mesh ID: {}", mesh_handle.id()));
                });
        }
    }

    /// Renders the Camera component.
    fn render_camera_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut query = world.query::<(&Camera, Option<&PerspectiveProjection>)>();
        if let Ok((camera, projection)) = query.get(world.inner(), entity) {
            egui::CollapsingHeader::new("Camera")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(format!("Active: {}", camera.is_active));
                    ui.label(format!("Priority: {}", camera.priority));

                    if let Some(proj) = projection {
                        ui.separator();
                        ui.label("Perspective Projection:");
                        ui.label(format!("FOV: {:.1}°", proj.fov.to_degrees()));
                        ui.label(format!("Aspect: {:.2}", proj.aspect_ratio));
                        ui.label(format!("Near: {:.2}", proj.near));
                        ui.label(format!("Far: {:.2}", proj.far));
                    }
                });
        }
    }

    /// Renders the Light component.
    fn render_light_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut query = world.query::<&PointLight>();
        if let Ok(light) = query.get(world.inner(), entity) {
            egui::CollapsingHeader::new("Point Light")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(format!(
                        "Color: [{:.2}, {:.2}, {:.2}]",
                        light.color.x, light.color.y, light.color.z
                    ));
                    ui.label(format!("Intensity: {:.2}", light.intensity));
                    ui.label(format!("Range: {:.2}", light.range));
                });
        }
    }

    /// Renders hierarchy information.
    fn render_hierarchy_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut parent_query = world.query::<&Parent>();
        let mut children_query = world.query::<&Children>();

        let has_parent = parent_query.get(world.inner(), entity).is_ok();
        let has_children = children_query.get(world.inner(), entity).is_ok();

        if has_parent || has_children {
            egui::CollapsingHeader::new("Hierarchy")
                .default_open(true)
                .show(ui, |ui| {
                    if let Ok(parent) = parent_query.get(world.inner(), entity) {
                        ui.label(format!("Parent: {:?}", parent.0));
                    }

                    if let Ok(children) = children_query.get(world.inner(), entity) {
                        ui.label(format!("Children: {}", children.len()));
                        for child in children.iter() {
                            ui.label(format!("  - {child:?}"));
                        }
                    }
                });
        }
    }

    /// Toggles the visibility of the entity inspector.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Sets the visibility of the entity inspector.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Selects an entity.
    pub fn select_entity(&mut self, entity: Entity) {
        self.selected_entity = Some(entity);
    }
}
