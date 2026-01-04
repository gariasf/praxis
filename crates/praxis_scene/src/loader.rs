//! Scene loading functionality for RON format.

use crate::definition::SceneDefinition;
use praxis_utils::{debug, info, Result};
use std::path::Path;

/// Scene loader for loading scene definitions from RON files.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::SceneLoader;
///
/// let loader = SceneLoader::new();
/// let scene = loader.load_from_file("assets/scenes/level1.ron").unwrap();
/// println!("Loaded scene: {}", scene.name);
/// ```
#[derive(Debug, Default)]
pub struct SceneLoader {
    /// Optional base path for scene files.
    base_path: Option<String>,
}

impl SceneLoader {
    /// Creates a new scene loader.
    #[must_use]
    pub const fn new() -> Self {
        Self { base_path: None }
    }

    /// Creates a scene loader with a base path.
    ///
    /// All relative paths will be resolved relative to this base path.
    pub fn with_base_path(base_path: impl Into<String>) -> Self {
        Self {
            base_path: Some(base_path.into()),
        }
    }

    /// Loads a scene definition from a RON file.
    ///
    /// This automatically applies any necessary migrations and validates the scene.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the RON scene file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, parsed, migrated, or validated.
    pub fn load_from_file(&self, path: impl AsRef<Path>) -> Result<SceneDefinition> {
        let path = path.as_ref();
        let full_path = self
            .base_path
            .as_ref()
            .map_or_else(|| path.to_path_buf(), |base| Path::new(base).join(path));

        debug!("Loading scene from: {}", full_path.display());

        let contents = std::fs::read_to_string(&full_path).map_err(|e| {
            praxis_utils::eyre::eyre!("Failed to read scene file '{}': {}", full_path.display(), e)
        })?;

        let scene = self.load_from_string(&contents)?;

        info!(
            "Loaded scene '{}' with {} root entities ({} total)",
            scene.name,
            scene.entity_count(),
            scene.total_entity_count()
        );

        Ok(scene)
    }

    /// Loads a scene definition from a RON string.
    ///
    /// This automatically applies any necessary migrations and validates the scene.
    ///
    /// # Arguments
    ///
    /// * `ron_string` - RON-formatted scene definition
    ///
    /// # Errors
    ///
    /// Returns an error if the RON cannot be parsed, migrated, or validated.
    pub fn load_from_string(&self, ron_string: &str) -> Result<SceneDefinition> {
        let mut scene: SceneDefinition = ron::from_str(ron_string)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to parse scene RON: {}", e))?;

        debug!("Parsed scene definition: {}", scene.name);

        // Apply migrations if needed
        crate::migration::migrate_scene(&mut scene)?;

        // Validate the scene
        crate::migration::validate_scene(&scene)?;

        Ok(scene)
    }

    /// Saves a scene definition to a RON file.
    ///
    /// # Arguments
    ///
    /// * `scene` - The scene definition to save
    /// * `path` - Path where the file should be written
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written or the scene cannot be serialized.
    pub fn save_to_file(&self, scene: &SceneDefinition, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let full_path = self
            .base_path
            .as_ref()
            .map_or_else(|| path.to_path_buf(), |base| Path::new(base).join(path));

        debug!("Saving scene to: {}", full_path.display());

        let ron_string = self.save_to_string(scene)?;

        std::fs::write(&full_path, ron_string).map_err(|e| {
            praxis_utils::eyre::eyre!(
                "Failed to write scene file '{}': {}",
                full_path.display(),
                e
            )
        })?;

        info!("Saved scene '{}' to {}", scene.name, full_path.display());

        Ok(())
    }

    /// Converts a scene definition to a RON string.
    ///
    /// # Arguments
    ///
    /// * `scene` - The scene definition to convert
    ///
    /// # Errors
    ///
    /// Returns an error if the scene cannot be serialized.
    pub fn save_to_string(&self, scene: &SceneDefinition) -> Result<String> {
        let pretty_config = ron::ser::PrettyConfig::new()
            .depth_limit(4)
            .separate_tuple_members(true)
            .enumerate_arrays(false)
            .indentor("    ".to_string());

        let ron_string = ron::ser::to_string_pretty(scene, pretty_config)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to serialize scene: {}", e))?;

        Ok(ron_string)
    }

    /// Sets the base path for scene files.
    pub fn set_base_path(&mut self, base_path: impl Into<String>) {
        self.base_path = Some(base_path.into());
    }

    /// Gets the base path if set.
    #[must_use]
    pub fn base_path(&self) -> Option<&str> {
        self.base_path.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{EntityDefinition, TransformDef};

    #[test]
    fn test_load_from_string() {
        let ron = r#"
        (
            name: "Test Scene",
            entities: [
                (
                    name: Some("TestEntity"),
                    transform: Some((
                        translation: (1.0, 2.0, 3.0),
                        rotation: (0.0, 0.0, 0.0, 1.0),
                        scale: (1.0, 1.0, 1.0),
                    )),
                    children: [],
                ),
            ],
            metadata: (
                description: Some("A test scene"),
                tags: ["test"],
            ),
        )
        "#;

        let loader = SceneLoader::new();
        let scene = loader.load_from_string(ron).unwrap();

        assert_eq!(scene.name, "Test Scene");
        assert_eq!(scene.entity_count(), 1);
        assert_eq!(scene.entities[0].name.as_deref(), Some("TestEntity"));
    }

    #[test]
    fn test_save_to_string() {
        let mut scene = SceneDefinition::new("Test Scene");
        scene.add_entity(
            EntityDefinition::new()
                .with_name("TestEntity")
                .with_transform(TransformDef::from_translation(1.0, 2.0, 3.0)),
        );

        let loader = SceneLoader::new();
        let ron_string = loader.save_to_string(&scene).unwrap();

        assert!(ron_string.contains("Test Scene"));
        assert!(ron_string.contains("TestEntity"));
    }

    #[test]
    fn test_roundtrip() {
        let mut scene = SceneDefinition::new("Roundtrip Test");
        scene.add_entity(
            EntityDefinition::new()
                .with_name("Entity1")
                .with_transform(TransformDef::from_translation(5.0, 10.0, 15.0))
                .with_mesh("cube"),
        );

        let loader = SceneLoader::new();
        let ron_string = loader.save_to_string(&scene).unwrap();
        let loaded_scene = loader.load_from_string(&ron_string).unwrap();

        assert_eq!(loaded_scene.name, scene.name);
        assert_eq!(loaded_scene.entity_count(), scene.entity_count());
        assert_eq!(loaded_scene.entities[0].name, scene.entities[0].name);
    }

    #[test]
    fn test_loader_new() {
        let loader = SceneLoader::new();
        assert!(loader.base_path().is_none());
    }

    #[test]
    fn test_loader_default() {
        let loader = SceneLoader::default();
        assert!(loader.base_path().is_none());
    }

    #[test]
    fn test_loader_with_base_path() {
        let loader = SceneLoader::with_base_path("assets/scenes");
        assert_eq!(loader.base_path(), Some("assets/scenes"));
    }

    #[test]
    fn test_loader_set_base_path() {
        let mut loader = SceneLoader::new();
        loader.set_base_path("custom/path");
        assert_eq!(loader.base_path(), Some("custom/path"));
    }

    #[test]
    fn test_load_from_string_with_children() {
        let ron = r#"
        (
            name: "Hierarchy Scene",
            entities: [
                (
                    name: Some("Parent"),
                    transform: Some((
                        translation: (0.0, 0.0, 0.0),
                        rotation: (0.0, 0.0, 0.0, 1.0),
                        scale: (1.0, 1.0, 1.0),
                    )),
                    children: [
                        (
                            name: Some("Child"),
                            transform: Some((
                                translation: (1.0, 0.0, 0.0),
                                rotation: (0.0, 0.0, 0.0, 1.0),
                                scale: (1.0, 1.0, 1.0),
                            )),
                            children: [],
                        ),
                    ],
                ),
            ],
            metadata: (),
        )
        "#;

        let loader = SceneLoader::new();
        let scene = loader.load_from_string(ron).unwrap();

        assert_eq!(scene.name, "Hierarchy Scene");
        assert_eq!(scene.entity_count(), 1);
        assert_eq!(scene.entities[0].children.len(), 1);
        assert_eq!(scene.total_entity_count(), 2);
    }

    #[test]
    fn test_load_from_string_empty_scene() {
        let ron = r#"
        (
            name: "Empty Scene",
            entities: [],
            metadata: (),
        )
        "#;

        let loader = SceneLoader::new();
        let scene = loader.load_from_string(ron).unwrap();

        assert_eq!(scene.name, "Empty Scene");
        assert_eq!(scene.entity_count(), 0);
    }

    #[test]
    fn test_load_from_string_with_metadata() {
        let ron = r#"
        (
            name: "Metadata Scene",
            entities: [],
            metadata: (
                description: Some("Test scene with metadata"),
                author: Some("Test Author"),
                version: Some("1.0.0"),
                tags: ["test", "demo"],
            ),
        )
        "#;

        let loader = SceneLoader::new();
        let scene = loader.load_from_string(ron).unwrap();

        assert_eq!(scene.name, "Metadata Scene");
        assert_eq!(
            scene.metadata.description.as_deref(),
            Some("Test scene with metadata")
        );
        assert_eq!(scene.metadata.author.as_deref(), Some("Test Author"));
        assert_eq!(scene.metadata.version.as_deref(), Some("1.0.0"));
        assert_eq!(scene.metadata.tags.len(), 2);
    }

    #[test]
    fn test_load_from_string_invalid_ron() {
        let invalid_ron = "this is not valid RON";

        let loader = SceneLoader::new();
        let result = loader.load_from_string(invalid_ron);

        assert!(result.is_err());
    }

    #[test]
    fn test_save_to_string_empty_scene() {
        let scene = SceneDefinition::new("Empty Scene");

        let loader = SceneLoader::new();
        let ron_string = loader.save_to_string(&scene).unwrap();

        assert!(ron_string.contains("Empty Scene"));
        assert!(ron_string.contains("entities: []"));
    }

    #[test]
    fn test_save_to_string_with_metadata() {
        let mut scene = SceneDefinition::new("Test Scene");
        scene.metadata.description = Some("A test scene".to_string());
        scene.metadata.author = Some("Test Author".to_string());
        scene.metadata.tags = vec!["test".to_string(), "demo".to_string()];

        let loader = SceneLoader::new();
        let ron_string = loader.save_to_string(&scene).unwrap();

        assert!(ron_string.contains("Test Scene"));
        assert!(ron_string.contains("A test scene"));
        assert!(ron_string.contains("Test Author"));
    }

    #[test]
    fn test_roundtrip_with_hierarchy() {
        let mut scene = SceneDefinition::new("Hierarchy Test");

        let child = EntityDefinition::new()
            .with_name("Child")
            .with_transform(TransformDef::from_translation(1.0, 0.0, 0.0));

        let parent = EntityDefinition::new()
            .with_name("Parent")
            .with_transform(TransformDef::from_translation(0.0, 0.0, 0.0))
            .with_child(child);

        scene.add_entity(parent);

        let loader = SceneLoader::new();
        let ron_string = loader.save_to_string(&scene).unwrap();
        let loaded_scene = loader.load_from_string(&ron_string).unwrap();

        assert_eq!(loaded_scene.name, scene.name);
        assert_eq!(
            loaded_scene.total_entity_count(),
            scene.total_entity_count()
        );
        assert_eq!(loaded_scene.entities[0].children.len(), 1);
    }

    #[test]
    fn test_save_and_load_complex_scene() {
        let mut scene = SceneDefinition::new("Complex Scene");

        scene.add_entity(EntityDefinition::perspective_camera(
            "MainCamera",
            (0.0, 5.0, 10.0),
            1.22,
            1.77,
        ));

        scene.add_entity(EntityDefinition::directional_light(
            "Sun",
            (0.0, -1.0, 0.0),
            (1.0, 1.0, 0.9),
            1.5,
        ));

        scene.add_entity(EntityDefinition::mesh_entity(
            "Cube",
            (0.0, 0.0, 0.0),
            "cube_mesh",
        ));

        let loader = SceneLoader::new();
        let ron_string = loader.save_to_string(&scene).unwrap();
        let loaded_scene = loader.load_from_string(&ron_string).unwrap();

        assert_eq!(loaded_scene.entity_count(), 3);
        assert!(loaded_scene.entities[0].camera.is_some());
        assert!(loaded_scene.entities[1].directional_light.is_some());
        assert!(loaded_scene.entities[2].mesh.is_some());
    }

    #[test]
    fn test_load_from_string_with_mesh_and_texture() {
        let ron = r#"
        (
            name: "Textured Scene",
            entities: [
                (
                    name: Some("TexturedCube"),
                    transform: Some((
                        translation: (0.0, 0.0, 0.0),
                        rotation: (0.0, 0.0, 0.0, 1.0),
                        scale: (1.0, 1.0, 1.0),
                    )),
                    mesh: Some("cube_mesh"),
                    texture: Some("cube_texture"),
                    children: [],
                ),
            ],
            metadata: (),
        )
        "#;

        let loader = SceneLoader::new();
        let scene = loader.load_from_string(ron).unwrap();

        assert_eq!(scene.entity_count(), 1);
        assert_eq!(scene.entities[0].mesh.as_deref(), Some("cube_mesh"));
        assert_eq!(scene.entities[0].texture.as_deref(), Some("cube_texture"));
    }

    #[test]
    fn test_roundtrip_preserves_visibility() {
        let mut scene = SceneDefinition::new("Visibility Test");
        let mut entity = EntityDefinition::new()
            .with_name("HiddenEntity")
            .with_transform(TransformDef::from_translation(0.0, 0.0, 0.0));
        entity.visible = Some(false);
        scene.add_entity(entity);

        let loader = SceneLoader::new();
        let ron_string = loader.save_to_string(&scene).unwrap();
        let loaded_scene = loader.load_from_string(&ron_string).unwrap();

        assert_eq!(loaded_scene.entities[0].visible, Some(false));
    }

    #[test]
    fn test_roundtrip_preserves_active_state() {
        let mut scene = SceneDefinition::new("Active Test");
        let mut entity = EntityDefinition::new()
            .with_name("InactiveEntity")
            .with_transform(TransformDef::from_translation(0.0, 0.0, 0.0));
        entity.active = Some(false);
        scene.add_entity(entity);

        let loader = SceneLoader::new();
        let ron_string = loader.save_to_string(&scene).unwrap();
        let loaded_scene = loader.load_from_string(&ron_string).unwrap();

        assert_eq!(loaded_scene.entities[0].active, Some(false));
    }

    #[test]
    fn test_save_and_load_with_editor_data() {
        use crate::definition::{EditorCamera, EditorData, ViewportSettings};

        let mut scene = SceneDefinition::new("Editor Scene");
        let editor_data = EditorData::new()
            .with_camera(EditorCamera::new())
            .with_selected_entities(vec!["Entity1".to_string()])
            .with_viewport(ViewportSettings::new());
        scene.set_editor_data(editor_data);

        scene.add_entity(EntityDefinition::mesh_entity(
            "Entity1",
            (0.0, 0.0, 0.0),
            "cube",
        ));

        let loader = SceneLoader::new();
        let ron_string = loader.save_to_string(&scene).unwrap();

        // Verify editor_data is in the serialized form
        assert!(ron_string.contains("editor_data"));

        let loaded_scene = loader.load_from_string(&ron_string).unwrap();

        assert!(loaded_scene.has_editor_data());
        let editor = loaded_scene.editor_data().unwrap();
        assert!(editor.camera.is_some());
        assert_eq!(editor.selected_entities.len(), 1);
        assert!(editor.viewport.is_some());
    }

    #[test]
    fn test_save_without_editor_data() {
        let mut scene = SceneDefinition::new("Runtime Scene");
        scene.add_entity(EntityDefinition::mesh_entity(
            "Entity1",
            (0.0, 0.0, 0.0),
            "cube",
        ));

        let loader = SceneLoader::new();
        let ron_string = loader.save_to_string(&scene).unwrap();

        // Verify editor_data is not in the serialized form
        assert!(!ron_string.contains("editor_data"));

        let loaded_scene = loader.load_from_string(&ron_string).unwrap();
        assert!(!loaded_scene.has_editor_data());
    }

    #[test]
    fn test_load_old_version_scene() {
        // Simulate an old version scene (version 0) without version field
        let old_scene_ron = r#"
        (
            name: "Old Scene",
            entities: [
                (
                    name: Some("OldEntity"),
                    transform: Some((
                        translation: (1.0, 2.0, 3.0),
                        rotation: (0.0, 0.0, 0.0, 1.0),
                        scale: (1.0, 1.0, 1.0),
                    )),
                    children: [],
                ),
            ],
            metadata: (),
        )
        "#;

        let loader = SceneLoader::new();
        let loaded_scene = loader.load_from_string(old_scene_ron).unwrap();

        // Scene should be migrated to current version
        assert_eq!(loaded_scene.version, crate::definition::CURRENT_SCENE_VERSION);
        assert_eq!(loaded_scene.name, "Old Scene");
        assert_eq!(loaded_scene.entity_count(), 1);
    }

    #[test]
    fn test_validation_on_load() {
        // Scene with invalid camera (near > far)
        let invalid_scene_ron = r#"
        (
            version: 1,
            name: "Invalid Scene",
            entities: [
                (
                    name: Some("BadCamera"),
                    camera: Some((
                        camera_type: Perspective,
                        fov: Some(1.0),
                        aspect_ratio: Some(1.77),
                        near: 100.0,
                        far: 10.0,
                        is_active: true,
                        priority: 0,
                    )),
                    children: [],
                ),
            ],
            metadata: (),
        )
        "#;

        let loader = SceneLoader::new();
        let result = loader.load_from_string(invalid_scene_ron);

        // Should fail validation
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_with_editor_camera() {
        use crate::definition::{CameraMode, EditorCamera, EditorData};

        let mut scene = SceneDefinition::new("Camera Test");
        let mut camera = EditorCamera::new();
        camera.position = (5.0, 10.0, 15.0);
        camera.target = (0.0, 1.0, 0.0);
        camera.distance = 20.0;
        camera.pitch = -0.5;
        camera.yaw = 1.2;
        camera.fov = 75.0;
        camera.mode = CameraMode::Free;

        scene.set_editor_data(EditorData::new().with_camera(camera));

        let loader = SceneLoader::new();
        let ron_string = loader.save_to_string(&scene).unwrap();
        let loaded_scene = loader.load_from_string(&ron_string).unwrap();

        let loaded_camera = loaded_scene
            .editor_data()
            .unwrap()
            .camera
            .as_ref()
            .unwrap();

        assert_eq!(loaded_camera.position, (5.0, 10.0, 15.0));
        assert_eq!(loaded_camera.target, (0.0, 1.0, 0.0));
        assert_eq!(loaded_camera.distance, 20.0);
        assert_eq!(loaded_camera.fov, 75.0);
        assert_eq!(loaded_camera.mode, CameraMode::Free);
    }

    #[test]
    fn test_roundtrip_with_viewport_settings() {
        use crate::definition::{EditorData, GizmoMode, ViewportSettings};

        let mut scene = SceneDefinition::new("Viewport Test");
        let mut viewport = ViewportSettings::new();
        viewport.show_grid = false;
        viewport.show_wireframe = true;
        viewport.grid_size = 30;
        viewport.grid_spacing = 2.0;
        viewport.background_color = (0.2, 0.3, 0.4);
        viewport.gizmo_mode = GizmoMode::Scale;

        scene.set_editor_data(EditorData::new().with_viewport(viewport));

        let loader = SceneLoader::new();
        let ron_string = loader.save_to_string(&scene).unwrap();
        let loaded_scene = loader.load_from_string(&ron_string).unwrap();

        let loaded_viewport = loaded_scene
            .editor_data()
            .unwrap()
            .viewport
            .as_ref()
            .unwrap();

        assert!(!loaded_viewport.show_grid);
        assert!(loaded_viewport.show_wireframe);
        assert_eq!(loaded_viewport.grid_size, 30);
        assert_eq!(loaded_viewport.grid_spacing, 2.0);
        assert_eq!(loaded_viewport.background_color, (0.2, 0.3, 0.4));
        assert_eq!(loaded_viewport.gizmo_mode, GizmoMode::Scale);
    }
}
