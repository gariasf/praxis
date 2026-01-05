//! ECS component for attaching scripts to entities.

use bevy_ecs::component::Component;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Component that attaches a script to an entity.
///
/// Scripts attached to entities can have lifecycle methods that are automatically called:
/// - `on_start()`: Called once when the entity is spawned
/// - `on_update(delta_time)`: Called every frame
/// - `on_destroy()`: Called when the entity is destroyed
#[derive(Component, Clone)]
pub struct ScriptComponent {
    /// Unique name for this script instance
    pub name: String,

    /// Path to the script file
    pub script_path: PathBuf,

    /// Whether the script has been initialized
    pub initialized: bool,

    /// Custom data that persists between script calls
    pub user_data: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl ScriptComponent {
    /// Creates a new script component.
    pub fn new(name: impl Into<String>, script_path: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            script_path: script_path.into(),
            initialized: false,
            user_data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Sets a custom data value that persists between script calls.
    pub fn set_data(&self, key: impl Into<String>, value: serde_json::Value) {
        self.user_data.write().insert(key.into(), value);
    }

    /// Gets a custom data value.
    pub fn get_data(&self, key: &str) -> Option<serde_json::Value> {
        self.user_data.read().get(key).cloned()
    }

    /// Clears all custom data.
    pub fn clear_data(&self) {
        self.user_data.write().clear();
    }
}

/// Represents an active script instance with its own Lua state.
#[derive(Clone)]
pub struct ScriptInstance {
    /// The entity this script is attached to
    pub entity: praxis_ecs::Entity,

    /// Name of the script
    pub name: String,

    /// Whether on_start has been called
    pub started: bool,
}

impl ScriptInstance {
    /// Creates a new script instance.
    pub fn new(entity: praxis_ecs::Entity, name: String) -> Self {
        Self {
            entity,
            name,
            started: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_script_component() {
        let component = ScriptComponent::new("test", "test.lua");
        assert_eq!(component.name, "test");
        assert_eq!(component.script_path, PathBuf::from("test.lua"));
        assert!(!component.initialized);
    }

    #[test]
    fn test_user_data() {
        let component = ScriptComponent::new("test", "test.lua");

        component.set_data("health", serde_json::json!(100));
        component.set_data("name", serde_json::json!("Player"));

        assert_eq!(component.get_data("health"), Some(serde_json::json!(100)));
        assert_eq!(
            component.get_data("name"),
            Some(serde_json::json!("Player"))
        );
        assert_eq!(component.get_data("nonexistent"), None);

        component.clear_data();
        assert_eq!(component.get_data("health"), None);
    }
}
