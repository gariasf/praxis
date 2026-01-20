//! Inspector panel for viewing and editing entity components.

use super::EditorPanel;
use crate::selection::SelectionSystem;
use crate::undo::{TransformEditCommand, UndoRedoSystem};
use bevy_ecs::entity::Entity;
use egui::{Color32, DragValue, Ui};
use praxis_audio::AudioSource;
use praxis_ecs::{
    CullingDebug, CullingParams, MaterialHandle, MaterialPropertiesComponent, MeshHandle, Name,
    PerspectiveProjection, Transform, World,
};
use praxis_math::Quat;
use praxis_physics::{Collider, Mass, PhysicsVelocity, RigidBody};

/// Panel for inspecting and editing selected entity properties.
pub struct InspectorPanel {
    title: String,
    /// Cached old transform values for undo/redo
    cached_transforms: std::collections::HashMap<Entity, Transform>,
}

impl InspectorPanel {
    /// Creates a new inspector panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: "Inspector".to_string(),
            cached_transforms: std::collections::HashMap::new(),
        }
    }

    /// Displays the inspector UI with access to the ECS world and undo system.
    pub fn ui_with_world(&mut self, ui: &mut Ui, world: &mut World) {
        ui.heading("Inspector");
        ui.separator();

        // Get selection system
        let selection = world.get_resource::<SelectionSystem>();
        if selection.is_none() {
            ui.label("Selection system not available.");
            return;
        }

        let selected_entities: Vec<Entity> = selection.unwrap().selected_entities().collect();

        if selected_entities.is_empty() {
            ui.label("Select an entity to inspect its components.");
            return;
        }

        if selected_entities.len() > 1 {
            ui.label(format!("{} entities selected", selected_entities.len()));
            ui.label("Multi-entity editing not yet implemented.");
            return;
        }

        let entity = selected_entities[0];

        // Display entity header
        ui.horizontal(|ui| {
            ui.strong(format!("Entity {entity:?}"));
        });
        ui.separator();

        // Display and edit components
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.display_name_component(ui, world, entity);
            self.display_transform_component(ui, world, entity);
            self.display_mesh_handle_component(ui, world, entity);
            self.display_material_handle_component(ui, world, entity);
            self.display_material_properties_component(ui, world, entity);
            self.display_culling_params_component(ui, world, entity);
            self.display_rigidbody_component(ui, world, entity);
            self.display_collider_component(ui, world, entity);
            self.display_physics_velocity_component(ui, world, entity);
            self.display_mass_component(ui, world, entity);
            self.display_audio_source_component(ui, world, entity);
            self.display_camera_component(ui, world, entity);
        });
    }

    fn display_name_component(&mut self, ui: &mut Ui, world: &mut World, entity: Entity) {
        let name = world.get::<Name>(entity).map(|n| n.as_str().to_string());

        if let Some(mut name_str) = name {
            ui.collapsing("Name", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    if ui.text_edit_singleline(&mut name_str).changed() {
                        world.entity_mut(entity).insert(Name::new(name_str));
                    }
                });
            });
        }
    }

    fn display_transform_component(&mut self, ui: &mut Ui, world: &mut World, entity: Entity) {
        let transform = world.get::<Transform>(entity).copied();

        if let Some(original_transform) = transform {
            let mut transform = original_transform;

            // Cache the original transform if not already cached
            self.cached_transforms
                .entry(entity)
                .or_insert(original_transform);

            let mut changed = false;

            ui.collapsing("Transform", |ui| {
                ui.label("Position:");
                ui.horizontal(|ui| {
                    ui.label("X:");
                    changed |= ui
                        .add(DragValue::new(&mut transform.translation.x).speed(0.1))
                        .changed();
                    ui.label("Y:");
                    changed |= ui
                        .add(DragValue::new(&mut transform.translation.y).speed(0.1))
                        .changed();
                    ui.label("Z:");
                    changed |= ui
                        .add(DragValue::new(&mut transform.translation.z).speed(0.1))
                        .changed();
                });

                ui.label("Rotation (Euler degrees):");
                let (mut x, mut y, mut z) = transform.rotation.to_euler(praxis_math::EulerRot::XYZ);
                x = x.to_degrees();
                y = y.to_degrees();
                z = z.to_degrees();

                ui.horizontal(|ui| {
                    ui.label("X:");
                    if ui.add(DragValue::new(&mut x).speed(1.0)).changed() {
                        changed = true;
                    }
                    ui.label("Y:");
                    if ui.add(DragValue::new(&mut y).speed(1.0)).changed() {
                        changed = true;
                    }
                    ui.label("Z:");
                    if ui.add(DragValue::new(&mut z).speed(1.0)).changed() {
                        changed = true;
                    }
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
                    changed |= ui
                        .add(DragValue::new(&mut transform.scale.x).speed(0.01))
                        .changed();
                    ui.label("Y:");
                    changed |= ui
                        .add(DragValue::new(&mut transform.scale.y).speed(0.01))
                        .changed();
                    ui.label("Z:");
                    changed |= ui
                        .add(DragValue::new(&mut transform.scale.z).speed(0.01))
                        .changed();
                });
            });

            // Handle transform changes with undo/redo
            if changed {
                // Apply immediately
                world.entity_mut(entity).insert(transform);

                // On mouse release, create undo command
                if ui.input(|i| i.pointer.any_released()) {
                    let old_transform = self
                        .cached_transforms
                        .get(&entity)
                        .copied()
                        .unwrap_or(original_transform);
                    let command =
                        Box::new(TransformEditCommand::new(entity, old_transform, transform));

                    // Execute through undo system
                    if let Some(undo_system) = world.get_resource_mut::<UndoRedoSystem>() {
                        // Manually add to history since we already applied the change
                        undo_system.history.redo_stack.clear();
                        undo_system.history.undo_stack.push_back(command);
                        if undo_system.history.undo_stack.len() > 100 {
                            undo_system.history.undo_stack.pop_front();
                        }
                        undo_system.mark_dirty();
                    }

                    // Update cache with new value
                    self.cached_transforms.insert(entity, transform);
                }
            }
        }
    }

    fn display_mesh_handle_component(&mut self, ui: &mut Ui, world: &mut World, entity: Entity) {
        let mesh_handle = world.get::<MeshHandle>(entity).map(|m| m.id().to_string());

        if let Some(mut mesh_id) = mesh_handle {
            ui.collapsing("Mesh Handle", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Mesh ID:");
                    if ui.text_edit_singleline(&mut mesh_id).changed() {
                        world.entity_mut(entity).insert(MeshHandle::new(mesh_id));
                    }
                });
            });
        }
    }

    fn display_material_handle_component(
        &mut self,
        ui: &mut Ui,
        world: &mut World,
        entity: Entity,
    ) {
        let material_handle = world
            .get::<MaterialHandle>(entity)
            .map(|m| m.id().to_string());

        if let Some(mut material_id) = material_handle {
            ui.collapsing("Material Handle", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Material ID:");
                    if ui.text_edit_singleline(&mut material_id).changed() {
                        world
                            .entity_mut(entity)
                            .insert(MaterialHandle::new(material_id));
                    }
                });
            });
        }
    }

    fn display_material_properties_component(
        &mut self,
        ui: &mut Ui,
        world: &mut World,
        entity: Entity,
    ) {
        let mat_props = world
            .get::<MaterialPropertiesComponent>(entity)
            .map(|m| m.0);

        if let Some(mut props) = mat_props {
            let mut changed = false;

            ui.collapsing("Material Properties", |ui| {
                ui.label("Base Color:");
                ui.horizontal(|ui| {
                    let mut color = Color32::from_rgba_premultiplied(
                        (props.base_color[0] * 255.0) as u8,
                        (props.base_color[1] * 255.0) as u8,
                        (props.base_color[2] * 255.0) as u8,
                        (props.base_color[3] * 255.0) as u8,
                    );
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        props.base_color = [
                            color.r() as f32 / 255.0,
                            color.g() as f32 / 255.0,
                            color.b() as f32 / 255.0,
                            color.a() as f32 / 255.0,
                        ];
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Metallic:");
                    changed |= ui
                        .add(egui::Slider::new(&mut props.metallic, 0.0..=1.0))
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Roughness:");
                    changed |= ui
                        .add(egui::Slider::new(&mut props.roughness, 0.0..=1.0))
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Emissive:");
                    changed |= ui
                        .add(DragValue::new(&mut props.emissive_strength).speed(0.1))
                        .changed();
                });
            });

            if changed {
                world
                    .entity_mut(entity)
                    .insert(MaterialPropertiesComponent(props));
            }
        }
    }

    fn display_culling_params_component(&mut self, ui: &mut Ui, world: &mut World, entity: Entity) {
        let culling_params = world.get::<CullingParams>(entity).copied();

        if let Some(mut params) = culling_params {
            let mut changed = false;

            ui.collapsing("Culling Parameters", |ui| {
                // Preset buttons
                ui.label("Presets:");
                ui.horizontal(|ui| {
                    if ui
                        .button("Disabled")
                        .on_hover_text("Disable all culling")
                        .clicked()
                    {
                        params = CullingParams::disabled();
                        changed = true;
                    }
                    if ui
                        .button("Large")
                        .on_hover_text("Buildings, terrain")
                        .clicked()
                    {
                        params = CullingParams::large_static();
                        changed = true;
                    }
                    if ui
                        .button("Medium")
                        .on_hover_text("Trees, vehicles")
                        .clicked()
                    {
                        params = CullingParams::medium();
                        changed = true;
                    }
                    if ui.button("Small").on_hover_text("Rocks, debris").clicked() {
                        params = CullingParams::small_props();
                        changed = true;
                    }
                    if ui.button("Detail").on_hover_text("Grass").clicked() {
                        params = CullingParams::detail();
                        changed = true;
                    }
                });

                ui.separator();

                // Average Normal
                ui.label("Average Normal:");
                ui.horizontal(|ui| {
                    ui.label("X:");
                    let normal_changed = ui
                        .add(DragValue::new(&mut params.average_normal.x).speed(0.01))
                        .changed();
                    ui.label("Y:");
                    let normal_changed = normal_changed
                        || ui
                            .add(DragValue::new(&mut params.average_normal.y).speed(0.01))
                            .changed();
                    ui.label("Z:");
                    let normal_changed = normal_changed
                        || ui
                            .add(DragValue::new(&mut params.average_normal.z).speed(0.01))
                            .changed();

                    if normal_changed {
                        params.average_normal = params.average_normal.normalize();
                        changed = true;
                    }
                });

                // Back-face Threshold
                ui.horizontal(|ui| {
                    ui.label("Backface Threshold:");
                    changed |= ui
                        .add(
                            DragValue::new(&mut params.backface_threshold)
                                .speed(0.01)
                                .range(-1.0..=1.0),
                        )
                        .changed();
                });

                // Min Screen Size
                ui.horizontal(|ui| {
                    ui.label("Min Screen Size:");
                    changed |= ui
                        .add(
                            DragValue::new(&mut params.min_screen_size)
                                .speed(0.5)
                                .range(0.0..=100.0),
                        )
                        .changed();
                });

                // Max Render Distance
                ui.horizontal(|ui| {
                    ui.label("Max Distance:");
                    changed |= ui
                        .add(
                            DragValue::new(&mut params.max_render_distance)
                                .speed(10.0)
                                .range(-1.0..=10000.0),
                        )
                        .changed();
                });

                // Real-time preview from CullingDebug component
                if let Some(debug) = world.get::<CullingDebug>(entity) {
                    ui.separator();
                    ui.label("Real-time Preview:");

                    let color = debug.debug_color();
                    let color32 = Color32::from_rgb(
                        (color[0] * 255.0) as u8,
                        (color[1] * 255.0) as u8,
                        (color[2] * 255.0) as u8,
                    );

                    if debug.is_culled() {
                        ui.colored_label(
                            color32,
                            format!(
                                "❌ Culled by: {}",
                                debug.primary_cull_reason().unwrap_or("Unknown")
                            ),
                        );
                    } else {
                        ui.colored_label(color32, "✓ Visible");
                    }

                    ui.label(format!("Distance: {:.1}", debug.distance_from_camera));
                    ui.label(format!("Screen: {:.1}px", debug.screen_size_pixels));
                }
            });

            if changed {
                world.entity_mut(entity).insert(params);
            }
        }
    }

    fn display_rigidbody_component(&mut self, ui: &mut Ui, world: &mut World, entity: Entity) {
        let rigidbody = world.get::<RigidBody>(entity).copied();

        if let Some(original_rb) = rigidbody {
            let mut rb = original_rb;
            let mut changed = false;

            ui.collapsing("RigidBody", |ui| {
                let mut rb_type = match rb {
                    RigidBody::Dynamic => 0,
                    RigidBody::Static => 1,
                    RigidBody::Kinematic => 2,
                };

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    if ui.selectable_value(&mut rb_type, 0, "Dynamic").clicked() {
                        rb = RigidBody::Dynamic;
                        changed = true;
                    }
                    if ui.selectable_value(&mut rb_type, 1, "Static").clicked() {
                        rb = RigidBody::Static;
                        changed = true;
                    }
                    if ui.selectable_value(&mut rb_type, 2, "Kinematic").clicked() {
                        rb = RigidBody::Kinematic;
                        changed = true;
                    }
                });
            });

            if changed && rb != original_rb {
                world.entity_mut(entity).insert(rb);
            }
        }
    }

    fn display_collider_component(&mut self, ui: &mut Ui, world: &mut World, entity: Entity) {
        let collider = world.get::<Collider>(entity).cloned();

        if let Some(mut collider) = collider {
            let mut changed = false;

            ui.collapsing("Collider", |ui| match &mut collider {
                Collider::Cuboid { hx, hy, hz } => {
                    ui.label("Type: Cuboid");
                    ui.horizontal(|ui| {
                        ui.label("Half-X:");
                        changed |= ui.add(DragValue::new(hx).speed(0.1)).changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label("Half-Y:");
                        changed |= ui.add(DragValue::new(hy).speed(0.1)).changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label("Half-Z:");
                        changed |= ui.add(DragValue::new(hz).speed(0.1)).changed();
                    });
                }
                Collider::Sphere { radius } => {
                    ui.label("Type: Sphere");
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        changed |= ui.add(DragValue::new(radius).speed(0.1)).changed();
                    });
                }
                Collider::CapsuleY {
                    half_height,
                    radius,
                } => {
                    ui.label("Type: Capsule Y");
                    ui.horizontal(|ui| {
                        ui.label("Half-Height:");
                        changed |= ui.add(DragValue::new(half_height).speed(0.1)).changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        changed |= ui.add(DragValue::new(radius).speed(0.1)).changed();
                    });
                }
                Collider::CapsuleX {
                    half_height,
                    radius,
                } => {
                    ui.label("Type: Capsule X");
                    ui.horizontal(|ui| {
                        ui.label("Half-Height:");
                        changed |= ui.add(DragValue::new(half_height).speed(0.1)).changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        changed |= ui.add(DragValue::new(radius).speed(0.1)).changed();
                    });
                }
                Collider::CapsuleZ {
                    half_height,
                    radius,
                } => {
                    ui.label("Type: Capsule Z");
                    ui.horizontal(|ui| {
                        ui.label("Half-Height:");
                        changed |= ui.add(DragValue::new(half_height).speed(0.1)).changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        changed |= ui.add(DragValue::new(radius).speed(0.1)).changed();
                    });
                }
                Collider::CylinderY {
                    half_height,
                    radius,
                } => {
                    ui.label("Type: Cylinder Y");
                    ui.horizontal(|ui| {
                        ui.label("Half-Height:");
                        changed |= ui.add(DragValue::new(half_height).speed(0.1)).changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        changed |= ui.add(DragValue::new(radius).speed(0.1)).changed();
                    });
                }
            });

            if changed {
                world.entity_mut(entity).insert(collider);
            }
        }
    }

    fn display_physics_velocity_component(
        &mut self,
        ui: &mut Ui,
        world: &mut World,
        entity: Entity,
    ) {
        let velocity = world.get::<PhysicsVelocity>(entity).copied();

        if let Some(mut velocity) = velocity {
            let mut changed = false;

            ui.collapsing("Physics Velocity", |ui| {
                ui.label("Linear Velocity:");
                ui.horizontal(|ui| {
                    ui.label("X:");
                    changed |= ui
                        .add(DragValue::new(&mut velocity.linear.x).speed(0.1))
                        .changed();
                    ui.label("Y:");
                    changed |= ui
                        .add(DragValue::new(&mut velocity.linear.y).speed(0.1))
                        .changed();
                    ui.label("Z:");
                    changed |= ui
                        .add(DragValue::new(&mut velocity.linear.z).speed(0.1))
                        .changed();
                });

                ui.label("Angular Velocity:");
                ui.horizontal(|ui| {
                    ui.label("X:");
                    changed |= ui
                        .add(DragValue::new(&mut velocity.angular.x).speed(0.1))
                        .changed();
                    ui.label("Y:");
                    changed |= ui
                        .add(DragValue::new(&mut velocity.angular.y).speed(0.1))
                        .changed();
                    ui.label("Z:");
                    changed |= ui
                        .add(DragValue::new(&mut velocity.angular.z).speed(0.1))
                        .changed();
                });
            });

            if changed {
                world.entity_mut(entity).insert(velocity);
            }
        }
    }

    fn display_mass_component(&mut self, ui: &mut Ui, world: &mut World, entity: Entity) {
        let mass = world.get::<Mass>(entity).copied();

        if let Some(mut mass) = mass {
            let mut changed = false;

            ui.collapsing("Mass", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Mass:");
                    changed |= ui
                        .add(
                            DragValue::new(&mut mass.mass)
                                .speed(0.1)
                                .range(0.0..=f32::MAX),
                        )
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Angular Inertia:");
                    changed |= ui
                        .add(
                            DragValue::new(&mut mass.angular_inertia)
                                .speed(0.1)
                                .range(0.0..=f32::MAX),
                        )
                        .changed();
                });
            });

            if changed {
                world.entity_mut(entity).insert(mass);
            }
        }
    }

    fn display_audio_source_component(&mut self, ui: &mut Ui, world: &mut World, entity: Entity) {
        let audio_source = world.get::<AudioSource>(entity).cloned();

        if let Some(mut audio) = audio_source {
            let mut changed = false;

            ui.collapsing("Audio Source", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Path:");
                    changed |= ui.text_edit_singleline(&mut audio.path).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Volume:");
                    changed |= ui
                        .add(egui::Slider::new(&mut audio.volume, 0.0..=1.0))
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Spatial:");
                    changed |= ui.checkbox(&mut audio.spatial, "").changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Looping:");
                    changed |= ui.checkbox(&mut audio.looping, "").changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Max Distance:");
                    changed |= ui
                        .add(
                            DragValue::new(&mut audio.max_distance)
                                .speed(1.0)
                                .range(0.0..=f32::MAX),
                        )
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Reference Distance:");
                    changed |= ui
                        .add(
                            DragValue::new(&mut audio.reference_distance)
                                .speed(0.1)
                                .range(0.0..=f32::MAX),
                        )
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("State:");
                    ui.label(format!("{:?}", audio.state));
                });

                ui.horizontal(|ui| {
                    if ui.button("Play").clicked() {
                        audio.play();
                        changed = true;
                    }
                    if ui.button("Pause").clicked() {
                        audio.pause();
                        changed = true;
                    }
                    if ui.button("Stop").clicked() {
                        audio.stop();
                        changed = true;
                    }
                });
            });

            if changed {
                world.entity_mut(entity).insert(audio);
            }
        }
    }

    fn display_camera_component(&mut self, ui: &mut Ui, world: &mut World, entity: Entity) {
        let camera = world.get::<PerspectiveProjection>(entity).copied();

        if let Some(mut camera) = camera {
            let mut changed = false;

            ui.collapsing("Perspective Camera", |ui| {
                ui.horizontal(|ui| {
                    ui.label("FOV (degrees):");
                    let mut fov_degrees = camera.fov.to_degrees();
                    if ui
                        .add(
                            DragValue::new(&mut fov_degrees)
                                .speed(1.0)
                                .range(1.0..=179.0),
                        )
                        .changed()
                    {
                        camera.fov = fov_degrees.to_radians();
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Aspect Ratio:");
                    changed |= ui
                        .add(
                            DragValue::new(&mut camera.aspect_ratio)
                                .speed(0.01)
                                .range(0.1..=10.0),
                        )
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Near Plane:");
                    changed |= ui
                        .add(
                            DragValue::new(&mut camera.near)
                                .speed(0.01)
                                .range(0.001..=1000.0),
                        )
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Far Plane:");
                    changed |= ui
                        .add(
                            DragValue::new(&mut camera.far)
                                .speed(1.0)
                                .range(0.1..=10000.0),
                        )
                        .changed();
                });
            });

            if changed {
                world.entity_mut(entity).insert(camera);
            }
        }
    }
}

impl Default for InspectorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for InspectorPanel {
    fn title(&self) -> &str {
        &self.title
    }

    fn ui(
        &mut self,
        ui: &mut Ui,
        _world: Option<&praxis_ecs::World>,
        _render_context: Option<&mut praxis_graphics::RenderContext>,
    ) {
        ui.heading("Inspector");
        ui.separator();
        ui.label("Use ui_with_world() for full functionality.");
    }
}
