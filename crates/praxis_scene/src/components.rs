//! Scene-related components for the Praxis ECS.

use bevy_ecs::component::Component;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_component_creation() {
        let handle = SceneHandle::new("test_scene");
        let scene = Scene::new(handle.clone());
        assert_eq!(scene.handle(), &handle);
    }

    #[test]
    fn test_scene_component_clone() {
        let handle = SceneHandle::new("test_scene");
        let scene1 = Scene::new(handle.clone());
        let scene2 = scene1.clone();
        assert_eq!(scene1, scene2);
    }

    #[test]
    fn test_scene_handle_new() {
        let handle = SceneHandle::new("my_scene");
        assert_eq!(handle.id(), "my_scene");
    }

    #[test]
    fn test_scene_handle_generate() {
        let handle1 = SceneHandle::generate();
        let handle2 = SceneHandle::generate();
        assert_ne!(handle1.id(), handle2.id());
        assert!(handle1.id().starts_with("scene_"));
        assert!(handle2.id().starts_with("scene_"));
    }

    #[test]
    fn test_scene_handle_from_str() {
        let handle = SceneHandle::from("test_scene");
        assert_eq!(handle.id(), "test_scene");
    }

    #[test]
    fn test_scene_handle_from_string() {
        let handle = SceneHandle::from(String::from("test_scene"));
        assert_eq!(handle.id(), "test_scene");
    }

    #[test]
    fn test_scene_handle_equality() {
        let handle1 = SceneHandle::new("scene1");
        let handle2 = SceneHandle::new("scene1");
        let handle3 = SceneHandle::new("scene2");
        assert_eq!(handle1, handle2);
        assert_ne!(handle1, handle3);
    }

    #[test]
    fn test_scene_handle_clone() {
        let handle1 = SceneHandle::new("test");
        let handle2 = handle1.clone();
        assert_eq!(handle1, handle2);
    }
}

/// Component marking an entity as belonging to a specific scene.
///
/// This component is used to track which scene an entity belongs to,
/// enabling selective scene unloading and management.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::{Scene, SceneHandle};
/// use praxis_ecs::World;
///
/// let mut world = World::new();
/// let scene_handle = SceneHandle::new("level1");
///
/// world.spawn(Scene(scene_handle.clone()));
/// ```
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Scene(pub SceneHandle);

impl Scene {
    /// Creates a new Scene component with the given handle.
    #[must_use]
    pub const fn new(handle: SceneHandle) -> Self {
        Self(handle)
    }

    /// Gets the scene handle.
    #[must_use]
    pub const fn handle(&self) -> &SceneHandle {
        &self.0
    }
}

/// Handle identifying a loaded scene.
///
/// This is used to reference and manage loaded scenes. Each loaded scene
/// instance gets a unique handle, even if multiple instances of the same
/// scene definition are loaded.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::SceneHandle;
///
/// let handle = SceneHandle::new("level1");
/// println!("Scene ID: {}", handle.id());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SceneHandle {
    /// Unique identifier for this scene instance.
    id: String,
}

impl SceneHandle {
    /// Creates a new scene handle with the given identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    /// Creates a new scene handle with a generated unique identifier.
    pub fn generate() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        Self {
            id: format!("scene_{id}"),
        }
    }

    /// Gets the scene identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl From<&str> for SceneHandle {
    fn from(id: &str) -> Self {
        Self::new(id)
    }
}

impl From<String> for SceneHandle {
    fn from(id: String) -> Self {
        Self { id }
    }
}
