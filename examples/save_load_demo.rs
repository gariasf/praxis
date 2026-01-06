//! Demonstrates the save/load system for full game state persistence.
//!
//! This example shows how to:
//! - Save the complete world state to disk
//! - Load saved game states
//! - Use save metadata for save file management
//! - Handle entity hierarchies in saves
//! - Exclude entities from saves with NoSave component

use praxis_ecs::{
    Active, Camera, DirectionalLight, MeshHandle, Name, NoSave, PerspectiveProjection,
    TextureHandle, Transform, Visibility, World,
};
use praxis_math::Vec3;
use praxis_scene::{SaveConfig, SaveManager, SaveMetadata};
use praxis_utils::Result;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("=== Save/Load System Demo ===\n");

    // Create a world with some entities
    let mut world = create_demo_world();

    // Create save manager
    let mut save_manager = SaveManager::new();

    // Configure save settings
    let config = SaveConfig {
        compress: false,
        include_editor_data: false,
        validate_after_save: true,
        pretty_print: true,
    };
    save_manager.set_config(config);

    // Determine save path
    let temp_dir = env::temp_dir();
    let save_path = temp_dir.join("praxis_demo_save.ron");
    println!("Save path: {}\n", save_path.display());

    // Demo 1: Basic save
    println!("=== Demo 1: Basic Save ===");
    demo_basic_save(&mut save_manager, &world, &save_path)?;

    // Demo 2: Save with metadata
    println!("\n=== Demo 2: Save with Rich Metadata ===");
    demo_save_with_metadata(&mut save_manager, &world, &save_path)?;

    // Demo 3: Load and verify
    println!("\n=== Demo 3: Load and Verify ===");
    demo_load_and_verify(&mut save_manager, &save_path)?;

    // Demo 4: Read metadata without loading
    println!("\n=== Demo 4: Read Metadata Only ===");
    demo_read_metadata(&save_manager, &save_path)?;

    // Demo 5: Multiple save slots
    println!("\n=== Demo 5: Multiple Save Slots ===");
    demo_multiple_slots(&mut save_manager, &world, &temp_dir)?;

    // Cleanup
    println!("\n=== Cleanup ===");
    let _ = std::fs::remove_file(&save_path);
    println!("Removed temporary save files");

    println!("\n=== Demo Complete ===");

    Ok(())
}

/// Creates a demo world with various entities
fn create_demo_world() -> World {
    let mut world = World::new();

    // Create a camera
    world.spawn((
        Name("MainCamera".to_string()),
        Transform::from_xyz(0.0, 5.0, 10.0),
        Camera::default(),
        PerspectiveProjection::default(),
        Active,
    ));

    // Create a directional light (sun)
    world.spawn((
        Name("Sun".to_string()),
        DirectionalLight {
            direction: Vec3::new(0.5, -1.0, 0.5).normalize(),
            color: Vec3::new(1.0, 0.95, 0.8),
            intensity: 1.0,
        },
        Active,
    ));

    // Create some game objects
    world.spawn((
        Name("Player".to_string()),
        Transform::from_xyz(0.0, 1.0, 0.0),
        MeshHandle::new("character"),
        TextureHandle::new("player_skin"),
        Visibility::Visible,
        Active,
    ));

    world.spawn((
        Name("Rock1".to_string()),
        Transform::from_xyz(5.0, 0.0, 2.0),
        MeshHandle::new("rock"),
        TextureHandle::new("rock_texture"),
        Visibility::Visible,
    ));

    world.spawn((
        Name("Tree1".to_string()),
        Transform::from_xyz(-3.0, 0.0, 4.0),
        MeshHandle::new("tree"),
        TextureHandle::new("bark_texture"),
        Visibility::Visible,
    ));

    // Create a temporary entity (will not be saved)
    world.spawn((
        Name("DebugMarker".to_string()),
        Transform::from_xyz(0.0, 2.0, 0.0),
        NoSave, // This entity will be excluded from saves
    ));

    println!(
        "Created demo world with {} entities",
        world.query::<&Name>().iter(&world).count()
    );

    world
}

/// Demonstrates basic save functionality
fn demo_basic_save(save_manager: &mut SaveManager, world: &World, path: &PathBuf) -> Result<()> {
    let metadata = SaveMetadata::new("Demo Save");

    save_manager.save_to_file(world, path, metadata)?;

    if let Some(stats) = save_manager.last_stats() {
        println!("Saved {} entities", stats.entity_count);
        println!("Saved {} components", stats.component_count);
        println!("Save took {:.2}ms", stats.duration_ms);
        if let Some(size) = stats.file_size_bytes {
            println!("File size: {} bytes ({:.2} KB)", size, size as f64 / 1024.0);
        }
    }

    Ok(())
}

/// Demonstrates saving with rich metadata
fn demo_save_with_metadata(
    save_manager: &mut SaveManager,
    world: &World,
    path: &PathBuf,
) -> Result<()> {
    let metadata = SaveMetadata::new("Chapter 1 - The Forest")
        .with_description("Player has just entered the forest area")
        .with_playtime(1847) // 30 minutes 47 seconds
        .with_game_version("0.1.0-alpha")
        .with_tag("autosave")
        .with_tag("chapter_1")
        .with_custom_data("location", "forest_entrance")
        .with_custom_data("quest_progress", "2/5")
        .with_custom_data("difficulty", "normal");

    save_manager.save_to_file(world, path, metadata)?;

    println!("Saved with comprehensive metadata");

    Ok(())
}

/// Demonstrates loading and verifying save data
fn demo_load_and_verify(save_manager: &mut SaveManager, path: &PathBuf) -> Result<()> {
    // Create a fresh world
    let mut new_world = World::new();

    // Load the save
    save_manager.load_from_file(&mut new_world, path)?;

    if let Some(stats) = save_manager.last_stats() {
        println!("Loaded {} entities", stats.entity_count);
        println!("Loaded {} components", stats.component_count);
        println!("Load took {:.2}ms", stats.duration_ms);
    }

    // Verify entities
    println!("\nVerifying loaded entities:");
    let mut camera_count = 0;
    let mut light_count = 0;
    let mut mesh_count = 0;

    for (name,) in new_world.query::<(&Name,)>().iter(&new_world) {
        println!("  - {}", name.0);

        // Count types
        if name.0.contains("Camera") {
            camera_count += 1;
        } else if name.0.contains("Sun") || name.0.contains("Light") {
            light_count += 1;
        } else if name.0.contains("Player") || name.0.contains("Rock") || name.0.contains("Tree") {
            mesh_count += 1;
        }
    }

    println!("\nLoaded entity summary:");
    println!("  Cameras: {}", camera_count);
    println!("  Lights: {}", light_count);
    println!("  Mesh objects: {}", mesh_count);

    // Verify NoSave entities were excluded
    let debug_marker_count = new_world
        .query::<(&Name,)>()
        .iter(&new_world)
        .filter(|(name,)| name.0 == "DebugMarker")
        .count();

    if debug_marker_count == 0 {
        println!("\n✓ NoSave entity correctly excluded from save");
    } else {
        println!("\n✗ Warning: NoSave entity was saved!");
    }

    Ok(())
}

/// Demonstrates reading metadata without loading the full save
fn demo_read_metadata(save_manager: &SaveManager, path: &PathBuf) -> Result<()> {
    let metadata = save_manager.read_metadata(path)?;

    println!("Save file metadata:");
    println!("  Name: {}", metadata.name);
    println!("  Timestamp: {}", metadata.timestamp);
    println!(
        "  Playtime: {}s ({:.1} min)",
        metadata.playtime_seconds,
        metadata.playtime_seconds as f64 / 60.0
    );

    if let Some(desc) = &metadata.description {
        println!("  Description: {}", desc);
    }

    if let Some(version) = &metadata.game_version {
        println!("  Game version: {}", version);
    }

    if !metadata.tags.is_empty() {
        println!("  Tags: {}", metadata.tags.join(", "));
    }

    if !metadata.custom_data.is_empty() {
        println!("  Custom data:");
        for (key, value) in &metadata.custom_data {
            println!("    {}: {}", key, value);
        }
    }

    Ok(())
}

/// Demonstrates managing multiple save slots
fn demo_multiple_slots(
    save_manager: &mut SaveManager,
    world: &World,
    save_dir: &PathBuf,
) -> Result<()> {
    // Create three different save slots
    let slots = [
        (
            "slot1.ron",
            SaveMetadata::new("Manual Save 1")
                .with_description("Before boss fight")
                .with_playtime(3600)
                .with_tag("manual"),
        ),
        (
            "slot2.ron",
            SaveMetadata::new("Manual Save 2")
                .with_description("After boss fight")
                .with_playtime(4200)
                .with_tag("manual"),
        ),
        (
            "autosave.ron",
            SaveMetadata::new("Autosave")
                .with_description("Auto-saved at checkpoint")
                .with_playtime(3900)
                .with_tag("autosave"),
        ),
    ];

    println!("Creating multiple save slots:");
    for (filename, metadata) in &slots {
        let path = save_dir.join(filename);
        save_manager.save_to_file(world, &path, metadata.clone())?;
        println!("  ✓ Created {}", filename);
    }

    println!("\nSave slot listing:");
    for (filename, _) in &slots {
        let path = save_dir.join(filename);
        if let Ok(metadata) = save_manager.read_metadata(&path) {
            println!(
                "  {} - {} ({}s)",
                filename, metadata.name, metadata.playtime_seconds
            );
        }
    }

    // Cleanup
    for (filename, _) in &slots {
        let path = save_dir.join(filename);
        let _ = std::fs::remove_file(path);
    }

    Ok(())
}
