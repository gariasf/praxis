//! Scene save/load operations for the editor.

use bevy_ecs::world::World;
use praxis_ecs::{
    Active, Camera, Children, DirectionalLight, GlobalTransform, MeshHandle, Name,
    OrthographicProjection, Parent, PerspectiveProjection, PointLight, TextureHandle, Transform,
    Visibility,
};
use praxis_scene::{EntityDefinition, SceneDefinition, SceneLoader, TransformDef};
use praxis_utils::{info, Result};
use std::path::Path;

/// Captures the current ECS world state into a `SceneDefinition`.
///
/// This function iterates through all entities in the world and converts them
/// to `EntityDefinition` structures, preserving hierarchy, components, and
/// relationships.
///
/// # Arguments
///
/// * `world` - The ECS world to capture
/// * `scene_name` - Name for the scene
///
/// # Returns
///
/// A `SceneDefinition` containing all entities and their components.
pub fn capture_scene_from_world(world: &mut World, scene_name: &str) -> SceneDefinition {
    let mut scene = SceneDefinition::new(scene_name);

    let mut query = world.query::<bevy_ecs::entity::Entity>();
    let mut root_entities = Vec::new();

    for entity in query.iter(world) {
        if world.get::<Parent>(entity).is_none() {
            root_entities.push(entity);
        }
    }

    for entity in root_entities {
        if let Some(entity_def) = capture_entity_recursive(world, entity) {
            scene.add_entity(entity_def);
        }
    }

    info!(
        "Captured scene '{}' with {} root entities",
        scene_name,
        scene.entity_count()
    );

    scene
}

/// Recursively captures an entity and its children.
fn capture_entity_recursive(world: &World, entity: bevy_ecs::entity::Entity) -> Option<EntityDefinition> {
    let mut entity_def = EntityDefinition::new();

    if let Some(name) = world.get::<Name>(entity) {
        entity_def.name = Some(name.0.clone());
    }

    if let Some(transform) = world.get::<Transform>(entity) {
        entity_def.transform = Some(TransformDef {
            translation: (
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
            ),
            rotation: (
                transform.rotation.x,
                transform.rotation.y,
                transform.rotation.z,
                transform.rotation.w,
            ),
            scale: (transform.scale.x, transform.scale.y, transform.scale.z),
        });
    }

    if let Some(mesh) = world.get::<MeshHandle>(entity) {
        entity_def.mesh = Some(mesh.id().to_string());
    }

    if let Some(texture) = world.get::<TextureHandle>(entity) {
        entity_def.texture = Some(texture.id().to_string());
    }

    if let Some(camera) = world.get::<Camera>(entity) {
        if let Some(perspective) = world.get::<PerspectiveProjection>(entity) {
            entity_def.camera = Some(praxis_scene::CameraDef::perspective(
                perspective.fov,
                perspective.aspect_ratio,
                perspective.near,
                perspective.far,
            ));
            if let Some(camera_mut) = entity_def.camera.as_mut() {
                camera_mut.is_active = camera.is_active;
                camera_mut.priority = camera.priority;
            }
        } else if let Some(ortho) = world.get::<OrthographicProjection>(entity) {
            entity_def.camera = Some(praxis_scene::CameraDef::orthographic(
                ortho.left,
                ortho.right,
                ortho.bottom,
                ortho.top,
                ortho.near,
                ortho.far,
            ));
            if let Some(camera_mut) = entity_def.camera.as_mut() {
                camera_mut.is_active = camera.is_active;
                camera_mut.priority = camera.priority;
            }
        }
    }

    if let Some(light) = world.get::<DirectionalLight>(entity) {
        entity_def.directional_light = Some(praxis_scene::DirectionalLightDef {
            direction: (light.direction.x, light.direction.y, light.direction.z),
            color: (light.color.x, light.color.y, light.color.z),
            intensity: light.intensity,
        });
    }

    if let Some(light) = world.get::<PointLight>(entity) {
        entity_def.point_light = Some(praxis_scene::PointLightDef {
            color: (light.color.x, light.color.y, light.color.z),
            intensity: light.intensity,
            range: light.range,
        });
    }

    if let Some(visibility) = world.get::<Visibility>(entity) {
        entity_def.visible = Some(*visibility == Visibility::Visible);
    }

    entity_def.active = Some(world.get::<Active>(entity).is_some());

    if let Some(children) = world.get::<Children>(entity) {
        for &child in &children.0 {
            if let Some(child_def) = capture_entity_recursive(world, child) {
                entity_def.children.push(child_def);
            }
        }
    }

    Some(entity_def)
}

/// Loads a scene from a file and spawns it into the world.
///
/// This clears the current world content and loads the new scene.
///
/// # Arguments
///
/// * `world` - The ECS world to load into
/// * `path` - Path to the scene file (.ron)
///
/// # Errors
///
/// Returns an error if the file cannot be loaded or entities cannot be spawned.
pub fn load_scene_into_world(world: &mut World, path: &Path) -> Result<()> {
    info!("Loading scene from: {}", path.display());

    let loader = SceneLoader::new();
    let scene = loader.load_from_file(path)?;

    world.clear_entities();

    for entity_def in &scene.entities {
        spawn_entity_recursive(world, entity_def, None)?;
    }

    info!(
        "Loaded scene '{}' with {} entities",
        scene.name,
        scene.total_entity_count()
    );

    Ok(())
}

/// Recursively spawns an entity and its children.
fn spawn_entity_recursive(
    world: &mut World,
    entity_def: &EntityDefinition,
    parent: Option<bevy_ecs::entity::Entity>,
) -> Result<bevy_ecs::entity::Entity> {
    let mut entity = world.spawn_empty();

    if let Some(ref name) = entity_def.name {
        entity.insert(Name::new(name.clone()));
    }

    if let Some(ref transform_def) = entity_def.transform {
        let (translation, rotation, scale) = transform_def.to_components();
        entity.insert(Transform {
            translation,
            rotation,
            scale,
        });
        entity.insert(GlobalTransform::default());
    }

    if let Some(ref mesh_id) = entity_def.mesh {
        entity.insert(MeshHandle::new(mesh_id));
    }

    if let Some(ref texture_id) = entity_def.texture {
        entity.insert(TextureHandle::new(texture_id));
    }

    if let Some(ref camera_def) = entity_def.camera {
        entity.insert(Camera {
            is_active: camera_def.is_active,
            priority: camera_def.priority,
        });

        match camera_def.camera_type {
            praxis_scene::CameraType::Perspective => {
                let fov = camera_def.fov.unwrap_or(70.0_f32.to_radians());
                let aspect_ratio = camera_def.aspect_ratio.unwrap_or(16.0 / 9.0);
                entity.insert(PerspectiveProjection {
                    fov,
                    aspect_ratio,
                    near: camera_def.near,
                    far: camera_def.far,
                });
            }
            praxis_scene::CameraType::Orthographic => {
                let left = camera_def.left.unwrap_or(-10.0);
                let right = camera_def.right.unwrap_or(10.0);
                let bottom = camera_def.bottom.unwrap_or(-10.0);
                let top = camera_def.top.unwrap_or(10.0);
                entity.insert(OrthographicProjection {
                    left,
                    right,
                    bottom,
                    top,
                    near: camera_def.near,
                    far: camera_def.far,
                });
            }
        }
    }

    if let Some(ref light_def) = entity_def.directional_light {
        let (direction, color, intensity) = light_def.to_components();
        entity.insert(DirectionalLight {
            direction,
            color,
            intensity,
        });
    }

    if let Some(ref light_def) = entity_def.point_light {
        let (color, intensity, range) = light_def.to_components();
        entity.insert(PointLight {
            color,
            intensity,
            range,
        });
    }

    let visibility = entity_def.visible.map_or(Visibility::Visible, |visible| {
        if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        }
    });
    entity.insert(visibility);

    if entity_def.active.unwrap_or(true) {
        entity.insert(Active);
    }

    if let Some(parent_entity) = parent {
        entity.insert(Parent(parent_entity));
    }

    let entity_id = entity.id();

    for child_def in &entity_def.children {
        spawn_entity_recursive(world, child_def, Some(entity_id))?;
    }

    Ok(entity_id)
}

/// Shows an unsaved changes dialog and returns the user's choice.
///
/// # Arguments
///
/// * `ctx` - The egui context
///
/// # Returns
///
/// * `Some(true)` - User chose to save changes
/// * `Some(false)` - User chose to discard changes
/// * `None` - User cancelled or dialog is still open
pub fn show_unsaved_changes_dialog(ctx: &egui::Context) -> Option<bool> {
    let mut result = None;

    egui::Window::new("Unsaved Changes")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("You have unsaved changes. Do you want to save them?");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    result = Some(true);
                }
                if ui.button("Don't Save").clicked() {
                    result = Some(false);
                }
                if ui.button("Cancel").clicked() {
                    result = None;
                }
            });
        });

    result
}
