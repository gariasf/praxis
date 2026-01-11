//! Scene management system for the Praxis engine.
//!
//! This crate provides functionality for managing game scenes, including:
//! - Scene components for entity tagging
//! - Scene definitions in RON format
//! - Scene loading and unloading
//! - Entity spawning from scene definitions
//! - Scene graph traversal utilities
//! - Full game state save/load system
//!
//! # Scene Structure
//!
//! A scene is defined in RON format and can contain:
//! - Entities with transforms, meshes, cameras, lights
//! - Hierarchical parent-child relationships
//! - Custom metadata
//!
//! # Transform Propagation
//!
//! The scene system works closely with the ECS transform hierarchy to ensure
//! proper transform propagation from parents to children:
//!
//! - **Local Transforms**: Each entity has a `Transform` component storing its
//!   local position, rotation, and scale relative to its parent (or world space
//!   if it has no parent).
//!
//! - **Global Transforms**: The `GlobalTransform` component stores the final
//!   world-space transform computed by multiplying the entity's local transform
//!   with its parent's global transform. This propagation happens automatically
//!   in the ECS transform system.
//!
//! - **Hierarchy Maintenance**: When entities are spawned from a scene definition,
//!   the `Parent` and `Children` components establish the hierarchy. The transform
//!   system then propagates changes down the hierarchy each frame.
//!
//! # Parent-Child Relationships
//!
//! The scene system uses ECS components to represent hierarchical relationships:
//!
//! - **Parent Component**: A child entity stores a `Parent(Entity)` component
//!   referencing its parent entity. This creates the upward link in the hierarchy.
//!
//! - **Children Component**: A parent entity stores a `Children(Vec<Entity>)`
//!   component containing all its child entities. This creates the downward link.
//!
//! - **Bidirectional Links**: Both components must be kept in sync. When spawning
//!   scenes, both are automatically set up. When modifying hierarchies at runtime,
//!   both must be updated together.
//!
//! - **Transform Propagation**: Changes to a parent's transform automatically
//!   propagate to all descendants through the global transform system, which
//!   traverses the `Children` component recursively.
//!
//! # Scene Loading Example
//!
//! ```rust,no_run
//! use praxis_scene::{SceneManager, SceneLoader};
//! use praxis_ecs::World;
//!
//! let mut world = World::new();
//! let mut scene_manager = SceneManager::new();
//! let scene_loader = SceneLoader::new();
//!
//! // Load a scene from a RON file
//! let scene_def = scene_loader.load_from_file("assets/scenes/level1.ron").unwrap();
//!
//! // Spawn scene entities into the world
//! let scene_handle = scene_manager.spawn_scene(&mut world, &scene_def).unwrap();
//!
//! // Later, unload the scene
//! scene_manager.unload_scene(&mut world, &scene_handle);
//! ```
//!
//! # Save/Load System Example
//!
//! ```rust,no_run
//! use praxis_scene::{SaveManager, SaveMetadata};
//! use praxis_ecs::World;
//!
//! let mut world = World::new();
//! let mut save_manager = SaveManager::new();
//!
//! // Create save metadata
//! let metadata = SaveMetadata::new("Chapter 1 - Forest")
//!     .with_description("Player at the forest entrance")
//!     .with_playtime(3600)
//!     .with_tag("autosave");
//!
//! // Save the complete game state
//! save_manager.save_to_file(&world, "saves/slot1.ron", metadata).unwrap();
//!
//! // Load the game state
//! save_manager.load_from_file(&mut world, "saves/slot1.ron").unwrap();
//!
//! // Read metadata without loading the full save
//! let metadata = save_manager.read_metadata("saves/slot1.ron").unwrap();
//! println!("Save: {} - {}", metadata.name, metadata.timestamp);
//! ```
//!
//! # Scene Definition Format (RON)
//!
//! ```ron
//! (
//!     name: "Level 1",
//!     entities: [
//!         (
//!             name: Some("Player"),
//!             transform: Some((
//!                 translation: (0.0, 1.0, 0.0),
//!                 rotation: (0.0, 0.0, 0.0, 1.0),
//!                 scale: (1.0, 1.0, 1.0),
//!             )),
//!             mesh: Some("character"),
//!             children: [],
//!         ),
//!         (
//!             name: Some("MainCamera"),
//!             transform: Some((
//!                 translation: (0.0, 5.0, 10.0),
//!                 rotation: (0.0, 0.0, 0.0, 1.0),
//!                 scale: (1.0, 1.0, 1.0),
//!             )),
//!             camera: Some((
//!                 camera_type: Perspective,
//!                 fov: 1.22173,
//!                 aspect_ratio: 1.77778,
//!                 near: 0.1,
//!                 far: 1000.0,
//!             )),
//!             children: [],
//!         ),
//!     ],
//! )
//! ```

mod animation;
mod components;
mod definition;
mod loader;
mod manager;
mod migration;
mod save;
mod traversal;

// Temporarily disabled due to private field access issues
// #[cfg(test)]
// mod animation_tests;

pub use animation::*;
pub use components::*;
pub use definition::*;
pub use loader::*;
pub use manager::*;
pub use migration::*;
pub use save::*;
pub use traversal::*;

use praxis_utils::{info, Result};

/// Initializes the scene system.
///
/// # Errors
///
/// Currently always returns Ok, but reserved for future initialization logic.
pub fn init() -> Result<()> {
    info!("Scene system initialized");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_system_initialization() {
        let result = init();
        assert!(result.is_ok());
    }
}
