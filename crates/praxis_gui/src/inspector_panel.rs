//! Inspector panel for viewing and editing ECS component data.
//!
//! The `InspectorPanel` provides a comprehensive UI for inspecting and modifying entity components.
//!
//! # Features
//!
//! - **Add/Remove Components**: Dropdown menu to add any supported component to an entity
//! - **Editable Fields**: All component properties can be modified directly in the UI
//! - **Apply-on-Change**: Changes are immediately applied to the ECS world
//! - **Component Categories**:
//!   - Transform & Hierarchy (Transform, GlobalTransform, Parent, Children)
//!   - Rendering (MeshHandle, TextureHandle, MaterialHandle, MaterialProperties)
//!   - Camera (Camera, PerspectiveProjection, OrthographicProjection)
//!   - Lighting (DirectionalLight, PointLight)
//!   - Physics (RigidBody, Collider, PhysicsVelocity)
//!   - Audio (AudioSource, AudioListener)
//!   - Utility (Name, Visibility)
//!
//! # Usage
//!
//! ```rust,no_run
//! use praxis_gui::InspectorPanel;
//! use praxis_ecs::{World, Entity};
//!
//! let mut inspector = InspectorPanel::new();
//! let mut world = World::new();
//! let selected_entity: Option<Entity> = None;
//!
//! // In your render loop
//! let ctx = egui::Context::default();
//! inspector.render(&ctx, &mut world, selected_entity);
//! ```

use praxis_audio::{AudioListener, AudioSource};
use praxis_ecs::{
    Camera, Children, CullingDebug, CullingParams, DirectionalLight, Entity, GlobalTransform,
    MaterialHandle, MaterialPropertiesComponent, MeshHandle, Name, OrthographicProjection, Parent,
    PerspectiveProjection, PointLight, TextureHandle, Transform, Visibility, World,
};
use praxis_math::Quat;
use praxis_physics::{Collider, PhysicsVelocity, RigidBody};

/// Inspector panel state and configuration.
pub struct InspectorPanel {
    /// Whether the inspector is visible.
    pub visible: bool,
    /// Whether the add component dropdown is open.
    add_component_open: bool,
}

impl Default for InspectorPanel {
    fn default() -> Self {
        Self {
            visible: true,
            add_component_open: false,
        }
    }
}

impl InspectorPanel {
    /// Creates a new inspector panel.
    pub fn new() -> Self {
        Self::default()
    }

    /// Renders the inspector panel UI.
    ///
    /// The selected_entity parameter should come from the HierarchyPanel's selection_state.
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        world: &mut World,
        selected_entity: Option<Entity>,
    ) {
        if !self.visible {
            return;
        }

        egui::Window::new("Inspector")
            .default_pos(egui::pos2(320.0, 50.0))
            .default_size(egui::vec2(400.0, 600.0))
            .resizable(true)
            .show(ctx, |ui| {
                self.render_selected_entity(ui, world, selected_entity);
            });
    }

    /// Renders details for the selected entity.
    fn render_selected_entity(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        selected_entity: Option<Entity>,
    ) {
        let Some(entity) = selected_entity else {
            ui.label("No entity selected");
            return;
        };

        if world.inner().get_entity(entity).is_none() {
            ui.label("Selected entity no longer exists");
            return;
        }

        ui.heading(format!("Entity: {entity:?}"));
        ui.separator();

        // Add Component button
        if ui.button("➕ Add Component").clicked() {
            self.add_component_open = !self.add_component_open;
        }

        if self.add_component_open {
            self.render_add_component_menu(ui, world, entity);
        }

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            self.render_name_component(ui, world, entity);
            self.render_transform_component(ui, world, entity);
            self.render_global_transform_component(ui, world, entity);
            self.render_mesh_component(ui, world, entity);
            self.render_texture_component(ui, world, entity);
            self.render_material_component(ui, world, entity);
            self.render_material_properties_component(ui, world, entity);
            self.render_culling_params_component(ui, world, entity);
            self.render_camera_component(ui, world, entity);
            self.render_perspective_projection_component(ui, world, entity);
            self.render_orthographic_projection_component(ui, world, entity);
            self.render_directional_light_component(ui, world, entity);
            self.render_point_light_component(ui, world, entity);
            self.render_rigid_body_component(ui, world, entity);
            self.render_collider_component(ui, world, entity);
            self.render_physics_velocity_component(ui, world, entity);
            self.render_audio_source_component(ui, world, entity);
            self.render_audio_listener_component(ui, world, entity);
            self.render_visibility_component(ui, world, entity);
            self.render_hierarchy_component(ui, world, entity);
        });
    }

    /// Renders the add component dropdown menu.
    fn render_add_component_menu(&mut self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        egui::ComboBox::from_label("Component Type")
            .selected_text("Select component to add...")
            .show_ui(ui, |ui| {
                let has_name = world.inner().get::<Name>(entity).is_some();
                let has_transform = world.inner().get::<Transform>(entity).is_some();
                let has_mesh = world.inner().get::<MeshHandle>(entity).is_some();
                let has_texture = world.inner().get::<TextureHandle>(entity).is_some();
                let has_material = world.inner().get::<MaterialHandle>(entity).is_some();
                let has_material_props = world
                    .inner()
                    .get::<MaterialPropertiesComponent>(entity)
                    .is_some();
                let has_camera = world.inner().get::<Camera>(entity).is_some();
                let has_persp_proj = world.inner().get::<PerspectiveProjection>(entity).is_some();
                let has_ortho_proj = world
                    .inner()
                    .get::<OrthographicProjection>(entity)
                    .is_some();
                let has_dir_light = world.inner().get::<DirectionalLight>(entity).is_some();
                let has_point_light = world.inner().get::<PointLight>(entity).is_some();
                let has_rigid_body = world.inner().get::<RigidBody>(entity).is_some();
                let has_collider = world.inner().get::<Collider>(entity).is_some();
                let has_physics_vel = world.inner().get::<PhysicsVelocity>(entity).is_some();
                let has_audio_source = world.inner().get::<AudioSource>(entity).is_some();
                let has_audio_listener = world.inner().get::<AudioListener>(entity).is_some();
                let has_visibility = world.inner().get::<Visibility>(entity).is_some();
                let has_culling_params = world.inner().get::<CullingParams>(entity).is_some();

                if !has_name && ui.selectable_label(false, "Name").clicked() {
                    let _ = world.insert_component(entity, Name::new("New Entity"));
                    self.add_component_open = false;
                }
                if !has_transform && ui.selectable_label(false, "Transform").clicked() {
                    let _ = world.insert_component(entity, Transform::default());
                    self.add_component_open = false;
                }
                if !has_mesh && ui.selectable_label(false, "Mesh Handle").clicked() {
                    let _ = world.insert_component(entity, MeshHandle::new(""));
                    self.add_component_open = false;
                }
                if !has_texture && ui.selectable_label(false, "Texture Handle").clicked() {
                    let _ = world.insert_component(entity, TextureHandle::new(""));
                    self.add_component_open = false;
                }
                if !has_material && ui.selectable_label(false, "Material Handle").clicked() {
                    let _ = world.insert_component(entity, MaterialHandle::new(""));
                    self.add_component_open = false;
                }
                if !has_material_props
                    && ui.selectable_label(false, "Material Properties").clicked()
                {
                    let _ = world.insert_component(entity, MaterialPropertiesComponent::default());
                    self.add_component_open = false;
                }
                if !has_camera && ui.selectable_label(false, "Camera").clicked() {
                    let _ = world.insert_component(entity, Camera::default());
                    self.add_component_open = false;
                }
                if !has_persp_proj
                    && ui
                        .selectable_label(false, "Perspective Projection")
                        .clicked()
                {
                    let _ = world.insert_component(entity, PerspectiveProjection::default());
                    self.add_component_open = false;
                }
                if !has_ortho_proj
                    && ui
                        .selectable_label(false, "Orthographic Projection")
                        .clicked()
                {
                    let _ = world.insert_component(entity, OrthographicProjection::default());
                    self.add_component_open = false;
                }
                if !has_dir_light && ui.selectable_label(false, "Directional Light").clicked() {
                    let _ = world.insert_component(entity, DirectionalLight::default());
                    self.add_component_open = false;
                }
                if !has_point_light && ui.selectable_label(false, "Point Light").clicked() {
                    let _ = world.insert_component(entity, PointLight::default());
                    self.add_component_open = false;
                }
                if !has_rigid_body && ui.selectable_label(false, "Rigid Body").clicked() {
                    let _ = world.insert_component(entity, RigidBody::default());
                    self.add_component_open = false;
                }
                if !has_collider && ui.selectable_label(false, "Collider").clicked() {
                    let _ = world.insert_component(entity, Collider::cuboid(0.5, 0.5, 0.5));
                    self.add_component_open = false;
                }
                if !has_physics_vel && ui.selectable_label(false, "Physics Velocity").clicked() {
                    let _ = world.insert_component(entity, PhysicsVelocity::default());
                    self.add_component_open = false;
                }
                if !has_audio_source && ui.selectable_label(false, "Audio Source").clicked() {
                    let _ = world.insert_component(entity, AudioSource::new(""));
                    self.add_component_open = false;
                }
                if !has_audio_listener && ui.selectable_label(false, "Audio Listener").clicked() {
                    let _ = world.insert_component(entity, AudioListener);
                    self.add_component_open = false;
                }
                if !has_visibility && ui.selectable_label(false, "Visibility").clicked() {
                    let _ = world.insert_component(entity, Visibility::default());
                    self.add_component_open = false;
                }
                if !has_culling_params && ui.selectable_label(false, "Culling Params").clicked() {
                    let _ = world.insert_component(entity, CullingParams::default());
                    self.add_component_open = false;
                }
            });
    }

    /// Renders the Name component editor.
    fn render_name_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut query = world.query::<&mut Name>();
        if let Ok(mut name) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Name")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mut name_text = name.0.clone();
                        if ui.text_edit_singleline(&mut name_text).changed() {
                            name.0 = name_text;
                        }
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });
                });
            if !open {
                world.remove_component::<Name>(entity);
            }
        }
    }

    /// Renders the Transform component editor.
    fn render_transform_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut query = world.query::<&mut Transform>();
        if let Ok(mut transform) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Transform")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Translation:");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut transform.translation.x).speed(0.1));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut transform.translation.y).speed(0.1));
                        ui.label("Z:");
                        ui.add(egui::DragValue::new(&mut transform.translation.z).speed(0.1));
                    });

                    ui.label("Rotation (Euler):");
                    let (mut x, mut y, mut z) =
                        transform.rotation.to_euler(praxis_math::EulerRot::XYZ);
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
            if !open {
                world.remove_component::<Transform>(entity);
            }
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
            egui::CollapsingHeader::new("Global Transform (Read-Only)")
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
        let mut query = world.query::<&mut MeshHandle>();
        if let Ok(mut mesh_handle) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Mesh Handle")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Mesh ID:");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });
                    let mut id_text = mesh_handle.id.clone();
                    if ui.text_edit_singleline(&mut id_text).changed() {
                        mesh_handle.id = id_text;
                    }
                });
            if !open {
                world.remove_component::<MeshHandle>(entity);
            }
        }
    }

    /// Renders the TextureHandle component.
    fn render_texture_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut query = world.query::<&mut TextureHandle>();
        if let Ok(mut texture_handle) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Texture Handle")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Texture ID:");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });
                    let mut id_text = texture_handle.id.clone();
                    if ui.text_edit_singleline(&mut id_text).changed() {
                        texture_handle.id = id_text;
                    }
                });
            if !open {
                world.remove_component::<TextureHandle>(entity);
            }
        }
    }

    /// Renders the MaterialHandle component.
    fn render_material_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut query = world.query::<&mut MaterialHandle>();
        if let Ok(mut material_handle) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Material Handle")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Material ID:");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });
                    let mut id_text = material_handle.id.clone();
                    if ui.text_edit_singleline(&mut id_text).changed() {
                        material_handle.id = id_text;
                    }
                });
            if !open {
                world.remove_component::<MaterialHandle>(entity);
            }
        }
    }

    /// Renders the MaterialPropertiesComponent.
    fn render_material_properties_component(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        entity: Entity,
    ) {
        let mut query = world.query::<&mut MaterialPropertiesComponent>();
        if let Ok(mut material_props) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Material Properties")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("PBR Material Properties");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });

                    ui.label("Base Color (RGBA):");
                    ui.horizontal(|ui| {
                        ui.label("R:");
                        ui.add(
                            egui::DragValue::new(&mut material_props.0.base_color[0])
                                .speed(0.01)
                                .range(0.0..=1.0),
                        );
                        ui.label("G:");
                        ui.add(
                            egui::DragValue::new(&mut material_props.0.base_color[1])
                                .speed(0.01)
                                .range(0.0..=1.0),
                        );
                        ui.label("B:");
                        ui.add(
                            egui::DragValue::new(&mut material_props.0.base_color[2])
                                .speed(0.01)
                                .range(0.0..=1.0),
                        );
                        ui.label("A:");
                        ui.add(
                            egui::DragValue::new(&mut material_props.0.base_color[3])
                                .speed(0.01)
                                .range(0.0..=1.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Metallic:");
                        ui.add(egui::Slider::new(&mut material_props.0.metallic, 0.0..=1.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Roughness:");
                        ui.add(egui::Slider::new(
                            &mut material_props.0.roughness,
                            0.0..=1.0,
                        ));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Emissive Strength:");
                        ui.add(
                            egui::DragValue::new(&mut material_props.0.emissive_strength)
                                .speed(0.1)
                                .range(0.0..=100.0),
                        );
                    });
                });
            if !open {
                world.remove_component::<MaterialPropertiesComponent>(entity);
            }
        }
    }

    /// Renders the CullingParams component editor.
    fn render_culling_params_component(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        entity: Entity,
    ) {
        // Fetch debug info first to avoid borrow conflicts
        let debug_info = world.inner().get::<CullingDebug>(entity).cloned();
        
        let mut query = world.query::<&mut CullingParams>();
        if let Ok(mut params) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Culling Parameters")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("GPU Culling Configuration");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });

                    ui.separator();

                    // Preset buttons
                    ui.label("Presets:");
                    ui.horizontal(|ui| {
                        if ui
                            .button("Disabled")
                            .on_hover_text("Disable all culling")
                            .clicked()
                        {
                            *params = CullingParams::disabled();
                        }
                        if ui
                            .button("Large Static")
                            .on_hover_text("Buildings, terrain")
                            .clicked()
                        {
                            *params = CullingParams::large_static();
                        }
                        if ui
                            .button("Medium")
                            .on_hover_text("Trees, vehicles")
                            .clicked()
                        {
                            *params = CullingParams::medium();
                        }
                        if ui
                            .button("Small Props")
                            .on_hover_text("Rocks, debris")
                            .clicked()
                        {
                            *params = CullingParams::small_props();
                        }
                        if ui
                            .button("Detail")
                            .on_hover_text("Grass, small stones")
                            .clicked()
                        {
                            *params = CullingParams::detail();
                        }
                    });

                    ui.separator();

                    // Average Normal
                    ui.label("Average Normal:");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        let mut changed = ui
                            .add(egui::DragValue::new(&mut params.average_normal.x).speed(0.01))
                            .changed();
                        ui.label("Y:");
                        changed |= ui
                            .add(egui::DragValue::new(&mut params.average_normal.y).speed(0.01))
                            .changed();
                        ui.label("Z:");
                        changed |= ui
                            .add(egui::DragValue::new(&mut params.average_normal.z).speed(0.01))
                            .changed();

                        if changed {
                            params.average_normal = params.average_normal.normalize();
                        }
                    });
                    ui.label("↳ Normal direction for back-face culling");

                    ui.separator();

                    // Back-face Threshold
                    ui.horizontal(|ui| {
                        ui.label("Back-face Threshold:");
                        ui.add(
                            egui::DragValue::new(&mut params.backface_threshold)
                                .speed(0.01)
                                .range(-1.0..=1.0),
                        );
                    });
                    ui.label("↳ Dot product threshold (-0.1 to 0.1 typical)");
                    if params.backface_threshold != 0.0 {
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            format!(
                                "  ⚠ Back-face culling active (threshold: {:.2})",
                                params.backface_threshold
                            ),
                        );
                    }

                    ui.separator();

                    // Minimum Screen Size
                    ui.horizontal(|ui| {
                        ui.label("Min Screen Size (pixels):");
                        ui.add(
                            egui::DragValue::new(&mut params.min_screen_size)
                                .speed(0.5)
                                .range(0.0..=100.0),
                        );
                    });
                    ui.label("↳ Cull objects smaller than this (0.0 = disabled)");
                    if params.min_screen_size > 0.0 {
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            format!(
                                "  ⚠ Small object culling active ({:.1} pixels)",
                                params.min_screen_size
                            ),
                        );
                    }

                    ui.separator();

                    // Maximum Render Distance
                    ui.horizontal(|ui| {
                        ui.label("Max Render Distance:");
                        ui.add(
                            egui::DragValue::new(&mut params.max_render_distance)
                                .speed(10.0)
                                .range(-1.0..=10000.0),
                        );
                    });
                    ui.label("↳ Cull beyond this distance (< 0 = disabled)");
                    if params.max_render_distance >= 0.0 {
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            format!(
                                "  ⚠ Distance culling active ({:.1} units)",
                                params.max_render_distance
                            ),
                        );
                    }

                    ui.separator();

                    // Summary
                    ui.label("Active Culling Strategies:");
                    let mut strategies = Vec::new();
                    if params.backface_threshold != 0.0 {
                        strategies.push("Back-face");
                    }
                    if params.min_screen_size > 0.0 {
                        strategies.push("Small object");
                    }
                    if params.max_render_distance >= 0.0 {
                        strategies.push("Distance");
                    }
                    if strategies.is_empty() {
                        ui.label("  None (all culling disabled)");
                    } else {
                        ui.label(format!("  {}", strategies.join(", ")));
                    }

                    // Real-time preview from CullingDebug component
                    if let Some(debug) = &debug_info {
                        ui.separator();
                        ui.label("Real-time Preview:");

                        let color = debug.debug_color();
                        let color32 = egui::Color32::from_rgb(
                            (color[0] * 255.0) as u8,
                            (color[1] * 255.0) as u8,
                            (color[2] * 255.0) as u8,
                        );

                        if debug.is_culled() {
                            ui.colored_label(
                                color32,
                                format!(
                                    "  ❌ Would be CULLED by: {}",
                                    debug.primary_cull_reason().unwrap_or("Unknown")
                                ),
                            );
                        } else {
                            ui.colored_label(color32, "  ✓ Would be VISIBLE");
                        }

                        // Detailed stats
                        ui.label(format!(
                            "  Distance: {:.1} units",
                            debug.distance_from_camera
                        ));
                        ui.label(format!(
                            "  Screen size: {:.1} pixels",
                            debug.screen_size_pixels
                        ));
                        ui.label(format!("  Backface dot: {:.2}", debug.backface_dot));

                        // Individual test results
                        ui.horizontal(|ui| {
                            ui.label("  Tests:");
                            if debug.culled_by_frustum {
                                ui.colored_label(egui::Color32::RED, "Frustum❌");
                            } else {
                                ui.colored_label(egui::Color32::GREEN, "Frustum✓");
                            }
                            if debug.culled_by_backface {
                                ui.colored_label(egui::Color32::LIGHT_RED, "Backface❌");
                            } else {
                                ui.colored_label(egui::Color32::GREEN, "Backface✓");
                            }
                            if debug.culled_by_screen_size {
                                ui.colored_label(egui::Color32::YELLOW, "Size❌");
                            } else {
                                ui.colored_label(egui::Color32::GREEN, "Size✓");
                            }
                            if debug.culled_by_distance {
                                ui.colored_label(egui::Color32::LIGHT_BLUE, "Dist❌");
                            } else {
                                ui.colored_label(egui::Color32::GREEN, "Dist✓");
                            }
                        });
                    } else {
                        ui.separator();
                        ui.colored_label(
                            egui::Color32::GRAY,
                            "  ℹ Enable culling debug visualization to see real-time preview",
                        );
                    }
                });
            if !open {
                world.remove_component::<CullingParams>(entity);
            }
        }
    }

    /// Renders the Camera component.
    fn render_camera_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut query = world.query::<&mut Camera>();
        if let Ok(mut camera) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Camera")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Camera Settings");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });

                    ui.checkbox(&mut camera.is_active, "Active");

                    ui.horizontal(|ui| {
                        ui.label("Priority:");
                        ui.add(egui::DragValue::new(&mut camera.priority).speed(1));
                    });
                });
            if !open {
                world.remove_component::<Camera>(entity);
            }
        }
    }

    /// Renders the PerspectiveProjection component.
    fn render_perspective_projection_component(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        entity: Entity,
    ) {
        let mut query = world.query::<&mut PerspectiveProjection>();
        if let Ok(mut projection) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Perspective Projection")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Perspective Settings");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });

                    let mut fov_degrees = projection.fov.to_degrees();
                    ui.horizontal(|ui| {
                        ui.label("FOV (degrees):");
                        if ui
                            .add(
                                egui::DragValue::new(&mut fov_degrees)
                                    .speed(1.0)
                                    .range(1.0..=179.0),
                            )
                            .changed()
                        {
                            projection.fov = fov_degrees.to_radians();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Aspect Ratio:");
                        ui.add(
                            egui::DragValue::new(&mut projection.aspect_ratio)
                                .speed(0.01)
                                .range(0.1..=10.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Near Plane:");
                        ui.add(
                            egui::DragValue::new(&mut projection.near)
                                .speed(0.01)
                                .range(0.001..=100.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Far Plane:");
                        ui.add(
                            egui::DragValue::new(&mut projection.far)
                                .speed(1.0)
                                .range(1.0..=10000.0),
                        );
                    });
                });
            if !open {
                world.remove_component::<PerspectiveProjection>(entity);
            }
        }
    }

    /// Renders the OrthographicProjection component.
    fn render_orthographic_projection_component(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        entity: Entity,
    ) {
        let mut query = world.query::<&mut OrthographicProjection>();
        if let Ok(mut projection) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Orthographic Projection")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Orthographic Settings");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Left:");
                        ui.add(egui::DragValue::new(&mut projection.left).speed(0.1));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Right:");
                        ui.add(egui::DragValue::new(&mut projection.right).speed(0.1));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Bottom:");
                        ui.add(egui::DragValue::new(&mut projection.bottom).speed(0.1));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Top:");
                        ui.add(egui::DragValue::new(&mut projection.top).speed(0.1));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Near:");
                        ui.add(
                            egui::DragValue::new(&mut projection.near)
                                .speed(0.01)
                                .range(0.001..=100.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Far:");
                        ui.add(
                            egui::DragValue::new(&mut projection.far)
                                .speed(1.0)
                                .range(1.0..=10000.0),
                        );
                    });
                });
            if !open {
                world.remove_component::<OrthographicProjection>(entity);
            }
        }
    }

    /// Renders the DirectionalLight component.
    fn render_directional_light_component(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        entity: Entity,
    ) {
        let mut query = world.query::<&mut DirectionalLight>();
        if let Ok(mut light) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Directional Light")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Directional Light Settings");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });

                    ui.label("Direction:");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        let mut changed = ui
                            .add(egui::DragValue::new(&mut light.direction.x).speed(0.01))
                            .changed();
                        ui.label("Y:");
                        changed |= ui
                            .add(egui::DragValue::new(&mut light.direction.y).speed(0.01))
                            .changed();
                        ui.label("Z:");
                        changed |= ui
                            .add(egui::DragValue::new(&mut light.direction.z).speed(0.01))
                            .changed();

                        if changed {
                            light.direction = light.direction.normalize();
                        }
                    });

                    ui.label("Color (RGB):");
                    ui.horizontal(|ui| {
                        ui.label("R:");
                        ui.add(
                            egui::DragValue::new(&mut light.color.x)
                                .speed(0.01)
                                .range(0.0..=1.0),
                        );
                        ui.label("G:");
                        ui.add(
                            egui::DragValue::new(&mut light.color.y)
                                .speed(0.01)
                                .range(0.0..=1.0),
                        );
                        ui.label("B:");
                        ui.add(
                            egui::DragValue::new(&mut light.color.z)
                                .speed(0.01)
                                .range(0.0..=1.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Intensity:");
                        ui.add(
                            egui::DragValue::new(&mut light.intensity)
                                .speed(0.1)
                                .range(0.0..=100.0),
                        );
                    });
                });
            if !open {
                world.remove_component::<DirectionalLight>(entity);
            }
        }
    }

    /// Renders the PointLight component.
    fn render_point_light_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut query = world.query::<&mut PointLight>();
        if let Ok(mut light) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Point Light")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Point Light Settings");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });

                    ui.label("Color (RGB):");
                    ui.horizontal(|ui| {
                        ui.label("R:");
                        ui.add(
                            egui::DragValue::new(&mut light.color.x)
                                .speed(0.01)
                                .range(0.0..=1.0),
                        );
                        ui.label("G:");
                        ui.add(
                            egui::DragValue::new(&mut light.color.y)
                                .speed(0.01)
                                .range(0.0..=1.0),
                        );
                        ui.label("B:");
                        ui.add(
                            egui::DragValue::new(&mut light.color.z)
                                .speed(0.01)
                                .range(0.0..=1.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Intensity:");
                        ui.add(
                            egui::DragValue::new(&mut light.intensity)
                                .speed(0.1)
                                .range(0.0..=100.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Range:");
                        ui.add(
                            egui::DragValue::new(&mut light.range)
                                .speed(0.1)
                                .range(0.1..=1000.0),
                        );
                    });
                });
            if !open {
                world.remove_component::<PointLight>(entity);
            }
        }
    }

    /// Renders the RigidBody component.
    fn render_rigid_body_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut query = world.query::<&mut RigidBody>();
        if let Ok(mut rigid_body) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Rigid Body")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Rigid Body Type");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.radio(rigid_body.is_dynamic(), "Dynamic").clicked() {
                            *rigid_body = RigidBody::Dynamic;
                        }
                        if ui.radio(rigid_body.is_static(), "Static").clicked() {
                            *rigid_body = RigidBody::Static;
                        }
                        if ui.radio(rigid_body.is_kinematic(), "Kinematic").clicked() {
                            *rigid_body = RigidBody::Kinematic;
                        }
                    });
                });
            if !open {
                world.remove_component::<RigidBody>(entity);
            }
        }
    }

    /// Renders the Collider component.
    fn render_collider_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut query = world.query::<&mut Collider>();
        if let Ok(mut collider) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Collider")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Collider Shape");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });

                    let current_type = match *collider {
                        Collider::Cuboid { .. } => "Cuboid",
                        Collider::Sphere { .. } => "Sphere",
                        Collider::CapsuleY { .. } => "Capsule Y",
                        Collider::CapsuleX { .. } => "Capsule X",
                        Collider::CapsuleZ { .. } => "Capsule Z",
                        Collider::CylinderY { .. } => "Cylinder Y",
                    };

                    egui::ComboBox::from_label("Shape Type")
                        .selected_text(current_type)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(current_type == "Cuboid", "Cuboid")
                                .clicked()
                            {
                                *collider = Collider::cuboid(0.5, 0.5, 0.5);
                            }
                            if ui
                                .selectable_label(current_type == "Sphere", "Sphere")
                                .clicked()
                            {
                                *collider = Collider::sphere(0.5);
                            }
                            if ui
                                .selectable_label(current_type == "Capsule Y", "Capsule Y")
                                .clicked()
                            {
                                *collider = Collider::capsule_y(1.0, 0.5);
                            }
                            if ui
                                .selectable_label(current_type == "Capsule X", "Capsule X")
                                .clicked()
                            {
                                *collider = Collider::capsule_x(1.0, 0.5);
                            }
                            if ui
                                .selectable_label(current_type == "Capsule Z", "Capsule Z")
                                .clicked()
                            {
                                *collider = Collider::capsule_z(1.0, 0.5);
                            }
                            if ui
                                .selectable_label(current_type == "Cylinder Y", "Cylinder Y")
                                .clicked()
                            {
                                *collider = Collider::cylinder_y(1.0, 0.5);
                            }
                        });

                    match &mut *collider {
                        Collider::Cuboid { hx, hy, hz } => {
                            ui.label("Half Extents:");
                            ui.horizontal(|ui| {
                                ui.label("X:");
                                ui.add(egui::DragValue::new(hx).speed(0.01).range(0.01..=100.0));
                                ui.label("Y:");
                                ui.add(egui::DragValue::new(hy).speed(0.01).range(0.01..=100.0));
                                ui.label("Z:");
                                ui.add(egui::DragValue::new(hz).speed(0.01).range(0.01..=100.0));
                            });
                        }
                        Collider::Sphere { radius } => {
                            ui.horizontal(|ui| {
                                ui.label("Radius:");
                                ui.add(
                                    egui::DragValue::new(radius).speed(0.01).range(0.01..=100.0),
                                );
                            });
                        }
                        Collider::CapsuleY {
                            half_height,
                            radius,
                        }
                        | Collider::CapsuleX {
                            half_height,
                            radius,
                        }
                        | Collider::CapsuleZ {
                            half_height,
                            radius,
                        } => {
                            ui.horizontal(|ui| {
                                ui.label("Half Height:");
                                ui.add(
                                    egui::DragValue::new(half_height)
                                        .speed(0.01)
                                        .range(0.01..=100.0),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("Radius:");
                                ui.add(
                                    egui::DragValue::new(radius).speed(0.01).range(0.01..=100.0),
                                );
                            });
                        }
                        Collider::CylinderY {
                            half_height,
                            radius,
                        } => {
                            ui.horizontal(|ui| {
                                ui.label("Half Height:");
                                ui.add(
                                    egui::DragValue::new(half_height)
                                        .speed(0.01)
                                        .range(0.01..=100.0),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("Radius:");
                                ui.add(
                                    egui::DragValue::new(radius).speed(0.01).range(0.01..=100.0),
                                );
                            });
                        }
                    }
                });
            if !open {
                world.remove_component::<Collider>(entity);
            }
        }
    }

    /// Renders the PhysicsVelocity component.
    fn render_physics_velocity_component(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        entity: Entity,
    ) {
        let mut query = world.query::<&mut PhysicsVelocity>();
        if let Ok(mut velocity) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Physics Velocity")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Velocity Settings");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });

                    ui.label("Linear Velocity:");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut velocity.linear.x).speed(0.1));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut velocity.linear.y).speed(0.1));
                        ui.label("Z:");
                        ui.add(egui::DragValue::new(&mut velocity.linear.z).speed(0.1));
                    });

                    ui.label("Angular Velocity:");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut velocity.angular.x).speed(0.1));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut velocity.angular.y).speed(0.1));
                        ui.label("Z:");
                        ui.add(egui::DragValue::new(&mut velocity.angular.z).speed(0.1));
                    });
                });
            if !open {
                world.remove_component::<PhysicsVelocity>(entity);
            }
        }
    }

    /// Renders the AudioSource component.
    fn render_audio_source_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut query = world.query::<&mut AudioSource>();
        if let Ok(mut audio_source) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Audio Source")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Audio Source Settings");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Audio Path:");
                    });
                    let mut path_text = audio_source.path.clone();
                    if ui.text_edit_singleline(&mut path_text).changed() {
                        audio_source.path = path_text;
                    }

                    ui.horizontal(|ui| {
                        ui.label("Volume:");
                        ui.add(egui::Slider::new(&mut audio_source.volume, 0.0..=1.0));
                    });

                    ui.checkbox(&mut audio_source.spatial, "Spatial Audio");
                    ui.checkbox(&mut audio_source.looping, "Looping");
                    ui.checkbox(&mut audio_source.doppler_enabled, "Doppler Effect");

                    if audio_source.spatial {
                        ui.horizontal(|ui| {
                            ui.label("Max Distance:");
                            ui.add(
                                egui::DragValue::new(&mut audio_source.max_distance)
                                    .speed(1.0)
                                    .range(0.1..=1000.0),
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Reference Distance:");
                            ui.add(
                                egui::DragValue::new(&mut audio_source.reference_distance)
                                    .speed(0.1)
                                    .range(0.1..=100.0),
                            );
                        });
                    }

                    if audio_source.doppler_enabled {
                        ui.horizontal(|ui| {
                            ui.label("Doppler Scale:");
                            ui.add(
                                egui::DragValue::new(&mut audio_source.doppler_scale)
                                    .speed(0.1)
                                    .range(0.0..=5.0),
                            );
                        });
                    }

                    ui.horizontal(|ui| {
                        if ui.button("▶ Play").clicked() {
                            audio_source.play();
                        }
                        if ui.button("⏸ Pause").clicked() {
                            audio_source.pause();
                        }
                        if ui.button("⏹ Stop").clicked() {
                            audio_source.stop();
                        }
                    });

                    ui.label(format!("State: {:?}", audio_source.state));
                });
            if !open {
                world.remove_component::<AudioSource>(entity);
            }
        }
    }

    /// Renders the AudioListener component.
    fn render_audio_listener_component(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        entity: Entity,
    ) {
        let mut query = world.query::<&AudioListener>();
        if query.get(world.inner(), entity).is_ok() {
            let mut open = true;
            egui::CollapsingHeader::new("Audio Listener")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Audio Listener (Marker Component)");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });
                    ui.label("This entity is the audio listener.");
                });
            if !open {
                world.remove_component::<AudioListener>(entity);
            }
        }
    }

    /// Renders the Visibility component.
    fn render_visibility_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut query = world.query::<&mut Visibility>();
        if let Ok(mut visibility) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Visibility")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Visibility Settings");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });

                    let mut is_visible = visibility.is_visible();
                    if ui.checkbox(&mut is_visible, "Visible").changed() {
                        if is_visible {
                            visibility.show();
                        } else {
                            visibility.hide();
                        }
                    }
                });
            if !open {
                world.remove_component::<Visibility>(entity);
            }
        }
    }

    /// Renders hierarchy information.
    fn render_hierarchy_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut parent_query = world.query::<&Parent>();
        let mut children_query = world.query::<&Children>();

        let has_parent = parent_query.get(world.inner(), entity).is_ok();
        let has_children = children_query.get(world.inner(), entity).is_ok();

        if has_parent || has_children {
            egui::CollapsingHeader::new("Hierarchy (Read-Only)")
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

    /// Toggles the visibility of the inspector panel.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Sets the visibility of the inspector panel.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}
