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
    /// # Arguments
    ///
    /// * `path` - Path to the RON scene file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load_from_file(&self, path: impl AsRef<Path>) -> Result<SceneDefinition> {
        let path = path.as_ref();
        let full_path = self.base_path.as_ref().map_or_else(
            || path.to_path_buf(),
            |base| Path::new(base).join(path),
        );

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
    /// # Arguments
    ///
    /// * `ron_string` - RON-formatted scene definition
    ///
    /// # Errors
    ///
    /// Returns an error if the RON cannot be parsed.
    pub fn load_from_string(&self, ron_string: &str) -> Result<SceneDefinition> {
        let scene: SceneDefinition = ron::from_str(ron_string)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to parse scene RON: {}", e))?;

        debug!("Parsed scene definition: {}", scene.name);

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
        let full_path = self.base_path.as_ref().map_or_else(
            || path.to_path_buf(),
            |base| Path::new(base).join(path),
        );

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
}
