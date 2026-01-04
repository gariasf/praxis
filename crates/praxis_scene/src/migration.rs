//! Scene format migration system.
//!
//! This module provides functionality to migrate scene files from older versions
//! to the current version. When the scene format changes in a backwards-incompatible
//! way, migration code should be added here.

use crate::definition::{SceneDefinition, CURRENT_SCENE_VERSION};
use praxis_utils::{debug, info, warn, Result};

/// Migrates a scene definition from an older version to the current version.
///
/// This function applies all necessary migrations in sequence to bring the
/// scene definition up to the current format version.
///
/// # Arguments
///
/// * `scene` - The scene definition to migrate
///
/// # Returns
///
/// Returns `Ok(true)` if migration was performed, `Ok(false)` if no migration
/// was needed, or an error if migration failed.
///
/// # Errors
///
/// Returns an error if no migration path exists for the scene version or if
/// a migration step fails.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::{SceneDefinition, migrate_scene};
///
/// let mut scene = SceneDefinition::new("My Scene");
/// scene.version = 0; // Old version
///
/// migrate_scene(&mut scene).unwrap();
/// assert_eq!(scene.version, praxis_scene::CURRENT_SCENE_VERSION);
/// ```
pub fn migrate_scene(scene: &mut SceneDefinition) -> Result<bool> {
    let original_version = scene.version;

    if scene.version == CURRENT_SCENE_VERSION {
        debug!("Scene '{}' is already at current version", scene.name);
        return Ok(false);
    }

    if scene.version > CURRENT_SCENE_VERSION {
        warn!(
            "Scene '{}' has version {} which is newer than current version {}. \
             This may cause compatibility issues.",
            scene.name, scene.version, CURRENT_SCENE_VERSION
        );
        return Ok(false);
    }

    info!(
        "Migrating scene '{}' from version {} to {}",
        scene.name, scene.version, CURRENT_SCENE_VERSION
    );

    // Apply migrations in sequence
    while scene.version < CURRENT_SCENE_VERSION {
        let next_version = scene.version + 1;
        debug!("Applying migration to version {}", next_version);

        match next_version {
            1 => migrate_to_v1(scene),
            // Future migrations would go here:
            // 2 => migrate_to_v2(scene)?,
            // 3 => migrate_to_v3(scene)?,
            _ => {
                return Err(praxis_utils::eyre::eyre!(
                    "No migration path from version {} to {}",
                    scene.version,
                    next_version
                ));
            }
        }

        scene.version = next_version;
    }

    info!(
        "Successfully migrated scene '{}' from version {} to {}",
        scene.name, original_version, scene.version
    );

    Ok(true)
}

/// Migrates a scene from version 0 to version 1.
///
/// Version 1 introduces:
/// - Scene version field
/// - Editor data support (camera, selection, viewport settings)
///
/// Since version 0 scenes don't have a version field, this migration is
/// primarily about adding the version field. The serde defaults handle
/// adding the optional `editor_data` field.
fn migrate_to_v1(scene: &SceneDefinition) {
    debug!("Migrating scene '{}' to version 1", scene.name);

    // Version 0 didn't have a version field, so scenes loaded as version 0
    // just need to be marked as version 1. All other fields are compatible.

    // If there's any version-0-specific data that needs transformation,
    // it would be done here.
}

/// Validates that a scene definition is internally consistent.
///
/// This checks for common issues like:
/// - Invalid entity references
/// - Malformed data
/// - Missing required fields
///
/// # Arguments
///
/// * `scene` - The scene definition to validate
///
/// # Errors
///
/// Returns an error if validation fails, with a description of the specific issue.
pub fn validate_scene(scene: &SceneDefinition) -> Result<()> {
    // Check version
    if scene.version > CURRENT_SCENE_VERSION {
        warn!(
            "Scene '{}' version {} is newer than current version {}",
            scene.name, scene.version, CURRENT_SCENE_VERSION
        );
    }

    // Check name is not empty
    if scene.name.is_empty() {
        return Err(praxis_utils::eyre::eyre!("Scene name cannot be empty"));
    }

    // Validate entities recursively
    for (i, entity) in scene.entities.iter().enumerate() {
        validate_entity(entity, &format!("root[{i}]"))?;
    }

    // Validate editor data if present
    if let Some(ref editor_data) = scene.editor_data {
        validate_editor_data(editor_data)?;
    }

    Ok(())
}

/// Validates an entity definition recursively.
fn validate_entity(entity: &crate::definition::EntityDefinition, path: &str) -> Result<()> {
    // Check if entity has at least one component
    let has_components = entity.transform.is_some()
        || entity.mesh.is_some()
        || entity.texture.is_some()
        || entity.camera.is_some()
        || entity.directional_light.is_some()
        || entity.point_light.is_some();

    if !has_components && entity.children.is_empty() {
        warn!(
            "Entity '{}' at path '{}' has no components and no children",
            entity.name.as_deref().unwrap_or("<unnamed>"),
            path
        );
    }

    // Validate camera if present
    if let Some(ref camera) = entity.camera {
        if camera.near >= camera.far {
            return Err(praxis_utils::eyre::eyre!(
                "Entity at '{}': Camera near ({}) must be less than far ({})",
                path,
                camera.near,
                camera.far
            ));
        }

        if let Some(fov) = camera.fov {
            if fov <= 0.0 || fov >= std::f32::consts::PI {
                return Err(praxis_utils::eyre::eyre!(
                    "Entity at '{}': Camera FOV ({}) must be between 0 and π",
                    path,
                    fov
                ));
            }
        }
    }

    // Validate children recursively
    for (i, child) in entity.children.iter().enumerate() {
        let child_path = format!("{path}.children[{i}]");
        validate_entity(child, &child_path)?;
    }

    Ok(())
}

/// Validates editor data.
fn validate_editor_data(editor_data: &crate::definition::EditorData) -> Result<()> {
    // Validate camera if present
    if let Some(ref camera) = editor_data.camera {
        if camera.near_clip >= camera.far_clip {
            return Err(praxis_utils::eyre::eyre!(
                "Editor camera near clip ({}) must be less than far clip ({})",
                camera.near_clip,
                camera.far_clip
            ));
        }

        if camera.fov <= 0.0 || camera.fov >= 180.0 {
            return Err(praxis_utils::eyre::eyre!(
                "Editor camera FOV ({}) must be between 0 and 180 degrees",
                camera.fov
            ));
        }

        if camera.distance < 0.0 {
            return Err(praxis_utils::eyre::eyre!(
                "Editor camera distance ({}) cannot be negative",
                camera.distance
            ));
        }
    }

    // Validate viewport settings if present
    if let Some(ref viewport) = editor_data.viewport {
        if viewport.grid_size == 0 {
            return Err(praxis_utils::eyre::eyre!(
                "Viewport grid size cannot be zero"
            ));
        }

        if viewport.grid_spacing <= 0.0 {
            return Err(praxis_utils::eyre::eyre!(
                "Viewport grid spacing ({}) must be positive",
                viewport.grid_spacing
            ));
        }

        // Validate background color is in valid range
        let (r, g, b) = viewport.background_color;
        if !(0.0..=1.0).contains(&r) || !(0.0..=1.0).contains(&g) || !(0.0..=1.0).contains(&b) {
            return Err(praxis_utils::eyre::eyre!(
                "Viewport background color ({}, {}, {}) components must be in range [0, 1]",
                r,
                g,
                b
            ));
        }
    }

    // Validate preferences if present
    if let Some(ref prefs) = editor_data.preferences {
        if prefs.auto_save_interval < 0.0 {
            return Err(praxis_utils::eyre::eyre!(
                "Auto-save interval ({}) cannot be negative",
                prefs.auto_save_interval
            ));
        }

        if prefs.snap_size <= 0.0 {
            return Err(praxis_utils::eyre::eyre!(
                "Snap size ({}) must be positive",
                prefs.snap_size
            ));
        }

        if prefs.rotation_snap <= 0.0 || prefs.rotation_snap > 180.0 {
            return Err(praxis_utils::eyre::eyre!(
                "Rotation snap ({}) must be between 0 and 180 degrees",
                prefs.rotation_snap
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{
        EditorCamera, EditorData, EditorPreferences, EntityDefinition, TransformDef,
        ViewportSettings,
    };

    #[test]
    fn test_migrate_scene_current_version() {
        let mut scene = SceneDefinition::new("Test");
        assert_eq!(scene.version, CURRENT_SCENE_VERSION);

        let migrated = migrate_scene(&mut scene).unwrap();
        assert!(!migrated);
        assert_eq!(scene.version, CURRENT_SCENE_VERSION);
    }

    #[test]
    fn test_migrate_scene_version_0_to_1() {
        let mut scene = SceneDefinition::new("Test");
        scene.version = 0;

        let migrated = migrate_scene(&mut scene).unwrap();
        assert!(migrated);
        assert_eq!(scene.version, CURRENT_SCENE_VERSION);
    }

    #[test]
    fn test_migrate_scene_newer_version() {
        let mut scene = SceneDefinition::new("Test");
        scene.version = CURRENT_SCENE_VERSION + 1;

        let migrated = migrate_scene(&mut scene).unwrap();
        assert!(!migrated);
        assert_eq!(scene.version, CURRENT_SCENE_VERSION + 1);
    }

    #[test]
    fn test_validate_scene_empty_name() {
        let mut scene = SceneDefinition::new("");
        scene.name = "".to_string();

        let result = validate_scene(&scene);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_scene_valid() {
        let mut scene = SceneDefinition::new("Valid Scene");
        scene.add_entity(
            EntityDefinition::new()
                .with_name("Entity1")
                .with_transform(TransformDef::identity()),
        );

        let result = validate_scene(&scene);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_editor_camera() {
        let mut scene = SceneDefinition::new("Test");
        scene.editor_data = Some(EditorData {
            camera: Some(EditorCamera::new()),
            selected_entities: Vec::new(),
            viewport: None,
            preferences: None,
        });

        let result = validate_scene(&scene);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_editor_camera_invalid_fov() {
        let mut scene = SceneDefinition::new("Test");
        let mut camera = EditorCamera::new();
        camera.fov = 200.0; // Invalid: > 180

        scene.editor_data = Some(EditorData {
            camera: Some(camera),
            selected_entities: Vec::new(),
            viewport: None,
            preferences: None,
        });

        let result = validate_scene(&scene);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_editor_camera_invalid_near_far() {
        let mut scene = SceneDefinition::new("Test");
        let mut camera = EditorCamera::new();
        camera.near_clip = 100.0;
        camera.far_clip = 10.0; // Invalid: near > far

        scene.editor_data = Some(EditorData {
            camera: Some(camera),
            selected_entities: Vec::new(),
            viewport: None,
            preferences: None,
        });

        let result = validate_scene(&scene);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_viewport_settings() {
        let mut scene = SceneDefinition::new("Test");
        scene.editor_data = Some(EditorData {
            camera: None,
            selected_entities: Vec::new(),
            viewport: Some(ViewportSettings::new()),
            preferences: None,
        });

        let result = validate_scene(&scene);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_viewport_invalid_grid_size() {
        let mut scene = SceneDefinition::new("Test");
        let mut viewport = ViewportSettings::new();
        viewport.grid_size = 0; // Invalid

        scene.editor_data = Some(EditorData {
            camera: None,
            selected_entities: Vec::new(),
            viewport: Some(viewport),
            preferences: None,
        });

        let result = validate_scene(&scene);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_viewport_invalid_background_color() {
        let mut scene = SceneDefinition::new("Test");
        let mut viewport = ViewportSettings::new();
        viewport.background_color = (1.5, 0.5, 0.5); // Invalid: > 1.0

        scene.editor_data = Some(EditorData {
            camera: None,
            selected_entities: Vec::new(),
            viewport: Some(viewport),
            preferences: None,
        });

        let result = validate_scene(&scene);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_editor_preferences() {
        let mut scene = SceneDefinition::new("Test");
        scene.editor_data = Some(EditorData {
            camera: None,
            selected_entities: Vec::new(),
            viewport: None,
            preferences: Some(EditorPreferences::new()),
        });

        let result = validate_scene(&scene);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_editor_preferences_invalid_snap_size() {
        let mut scene = SceneDefinition::new("Test");
        let mut prefs = EditorPreferences::new();
        prefs.snap_size = -1.0; // Invalid: must be positive

        scene.editor_data = Some(EditorData {
            camera: None,
            selected_entities: Vec::new(),
            viewport: None,
            preferences: Some(prefs),
        });

        let result = validate_scene(&scene);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_entity_invalid_camera() {
        let mut scene = SceneDefinition::new("Test");
        let camera = crate::definition::CameraDef::perspective(3.0, 1.77, 100.0, 10.0);
        // Invalid: near (100.0) > far (10.0)

        let mut entity = EntityDefinition::new().with_name("Camera");
        entity.camera = Some(camera);
        scene.add_entity(entity);

        let result = validate_scene(&scene);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_entity_recursive() {
        let mut scene = SceneDefinition::new("Test");

        let grandchild = EntityDefinition::new()
            .with_name("Grandchild")
            .with_transform(TransformDef::identity());

        let child = EntityDefinition::new()
            .with_name("Child")
            .with_transform(TransformDef::identity())
            .with_child(grandchild);

        let parent = EntityDefinition::new()
            .with_name("Parent")
            .with_transform(TransformDef::identity())
            .with_child(child);

        scene.add_entity(parent);

        let result = validate_scene(&scene);
        assert!(result.is_ok());
    }
}
