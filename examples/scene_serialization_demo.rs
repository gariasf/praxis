//! Demonstrates the scene serialization system with versioning and editor data.
//!
//! This example shows:
//! - Creating scenes with editor data
//! - Saving scenes with complete state preservation
//! - Loading scenes with automatic migration
//! - Validation of scene data
//! - Runtime scene creation (without editor data)

use praxis_scene::{
    EditorCamera, EditorData, EditorPreferences, EntityDefinition, GizmoMode, SceneDefinition,
    SceneLoader, TransformDef, ViewportSettings,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Scene Serialization Demo ===\n");

    // Create a scene with entities
    let mut scene = create_sample_scene();

    // Add editor data
    add_editor_data(&mut scene);

    // Save the scene
    save_scene(&scene)?;

    // Load the scene back
    let loaded_scene = load_scene()?;

    // Display loaded scene information
    display_scene_info(&loaded_scene);

    // Create runtime scene (without editor data)
    create_runtime_scene(&loaded_scene)?;

    println!("\n=== Demo Complete ===");

    Ok(())
}

/// Creates a sample scene with entities and hierarchy.
fn create_sample_scene() -> SceneDefinition {
    println!("Creating sample scene...");

    let mut scene = SceneDefinition::new("Demo Scene");

    // Set metadata
    scene.metadata.description = Some("Example scene demonstrating serialization".to_string());
    scene.metadata.author = Some("Scene Demo".to_string());
    scene.metadata.version = Some("1.0.0".to_string());
    scene.metadata.tags = vec!["demo".to_string(), "example".to_string()];

    // Add camera entity
    let camera = EntityDefinition::perspective_camera(
        "MainCamera",
        (0.0, 5.0, 10.0),
        std::f32::consts::FRAC_PI_3,
        1.77,
    );
    scene.add_entity(camera);

    // Add directional light
    let light = EntityDefinition::directional_light("Sun", (0.5, -1.0, 0.3), (1.0, 0.95, 0.9), 1.5);
    scene.add_entity(light);

    // Add a parent entity with children (hierarchy)
    let child1 = EntityDefinition::mesh_entity("Child1", (1.0, 0.0, 0.0), "cube_mesh");
    let child2 = EntityDefinition::mesh_entity("Child2", (-1.0, 0.0, 0.0), "sphere_mesh");

    let parent = EntityDefinition::new()
        .with_name("Parent")
        .with_transform(TransformDef::from_translation(0.0, 0.0, 0.0))
        .with_child(child1)
        .with_child(child2);

    scene.add_entity(parent);

    println!(
        "  Created scene with {} entities ({} total including children)",
        scene.entity_count(),
        scene.total_entity_count()
    );

    scene
}

/// Adds editor data to the scene.
fn add_editor_data(scene: &mut SceneDefinition) {
    println!("\nAdding editor data...");

    // Create editor camera
    let editor_camera = EditorCamera::orbit(
        (0.0, 1.0, 0.0), // Target position
        15.0,            // Distance
        -0.4,            // Pitch (radians)
        0.8,             // Yaw (radians)
    );
    println!(
        "  Editor camera: Orbit mode at distance {}",
        editor_camera.distance
    );

    // Create viewport settings
    let mut viewport = ViewportSettings::new();
    viewport.show_grid = true;
    viewport.show_gizmos = true;
    viewport.gizmo_mode = GizmoMode::Translate;
    viewport.grid_size = 20;
    viewport.grid_spacing = 1.0;
    println!(
        "  Viewport: Grid enabled, {} x {} grid",
        viewport.grid_size, viewport.grid_size
    );

    // Create editor preferences
    let mut preferences = EditorPreferences::new();
    preferences.auto_save_enabled = true;
    preferences.auto_save_interval = 300.0;
    preferences.snap_to_grid = false;
    preferences.snap_size = 1.0;
    preferences.rotation_snap = 15.0;
    println!(
        "  Preferences: Auto-save enabled (every {:.0}s)",
        preferences.auto_save_interval
    );

    // Create editor data
    let editor_data = EditorData::new()
        .with_camera(editor_camera)
        .with_selected_entities(vec!["Parent".to_string()])
        .with_viewport(viewport)
        .with_preferences(preferences);

    scene.set_editor_data(editor_data);
    println!("  Editor data added to scene");
}

/// Saves the scene to a RON file.
fn save_scene(scene: &SceneDefinition) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nSaving scene...");

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(scene)?;

    println!(
        "  Scene serialized to RON format ({} bytes)",
        ron_string.len()
    );
    println!("  Version: {}", scene.version);

    // In a real application, you would save to a file:
    // loader.save_to_file(scene, "assets/scenes/demo.ron")?;

    // For demo purposes, we'll just print a preview
    let preview: String = ron_string.lines().take(20).collect::<Vec<_>>().join("\n");
    println!("\n--- RON Preview (first 20 lines) ---");
    println!("{preview}");
    println!("...\n--- End Preview ---");

    Ok(())
}

/// Loads the scene from RON string.
fn load_scene() -> Result<SceneDefinition, Box<dyn std::error::Error>> {
    println!("\nLoading scene...");

    // In this demo, we'll create a sample RON string
    // In a real application, you would load from a file:
    // let scene = loader.load_from_file("assets/scenes/demo.ron")?;

    let sample_ron = r#"
    (
        version: 1,
        name: "Demo Scene",
        entities: [
            (
                name: Some("MainCamera"),
                transform: Some((
                    translation: (0.0, 5.0, 10.0),
                    rotation: (0.0, 0.0, 0.0, 1.0),
                    scale: (1.0, 1.0, 1.0),
                )),
                camera: Some((
                    camera_type: Perspective,
                    fov: Some(1.0472),
                    aspect_ratio: Some(1.77),
                    near: 0.1,
                    far: 1000.0,
                    is_active: true,
                    priority: 0,
                )),
                children: [],
            ),
        ],
        metadata: (
            description: Some("Example scene"),
            author: Some("Scene Demo"),
            version: Some("1.0.0"),
            tags: ["demo", "example"],
        ),
        editor_data: Some((
            camera: Some((
                position: (10.0, 8.0, 15.0),
                target: (0.0, 1.0, 0.0),
                distance: 15.0,
                pitch: -0.4,
                yaw: 0.8,
                fov: 60.0,
                near_clip: 0.1,
                far_clip: 1000.0,
                mode: Orbit,
            )),
            selected_entities: ["MainCamera"],
            viewport: Some((
                show_grid: true,
                show_gizmos: true,
                show_wireframe: false,
                show_bounds: false,
                show_lights: true,
                show_cameras: false,
                grid_size: 20,
                grid_spacing: 1.0,
                background_color: (0.118, 0.118, 0.137),
                gizmo_mode: Translate,
            )),
        )),
    )
    "#;

    let loader = SceneLoader::new();
    let scene = loader.load_from_string(sample_ron)?;

    println!("  Scene loaded successfully");
    println!("  Automatic migration: Complete");
    println!("  Validation: Passed");

    Ok(scene)
}

/// Displays information about the loaded scene.
fn display_scene_info(scene: &SceneDefinition) {
    println!("\n=== Scene Information ===");
    println!("Name: {}", scene.name);
    println!("Version: {}", scene.version);
    println!(
        "Entities: {} root ({} total)",
        scene.entity_count(),
        scene.total_entity_count()
    );

    // Display metadata
    if let Some(ref desc) = scene.metadata.description {
        println!("Description: {desc}");
    }
    if let Some(ref author) = scene.metadata.author {
        println!("Author: {author}");
    }
    if !scene.metadata.tags.is_empty() {
        println!("Tags: {}", scene.metadata.tags.join(", "));
    }

    // Display editor data if present
    if scene.has_editor_data() {
        println!("\n=== Editor Data ===");
        let editor = scene.editor_data().unwrap();

        if let Some(ref camera) = editor.camera {
            println!("Editor Camera:");
            println!("  Mode: {:?}", camera.mode);
            println!("  Position: {:?}", camera.position);
            println!("  Target: {:?}", camera.target);
            println!("  Distance: {}", camera.distance);
            println!("  FOV: {}°", camera.fov);
        }

        if !editor.selected_entities.is_empty() {
            println!("Selected Entities: {}", editor.selected_entities.join(", "));
        }

        if let Some(ref viewport) = editor.viewport {
            println!("Viewport Settings:");
            println!(
                "  Grid: {} (size: {}, spacing: {})",
                viewport.show_grid, viewport.grid_size, viewport.grid_spacing
            );
            println!(
                "  Gizmos: {} (mode: {:?})",
                viewport.show_gizmos, viewport.gizmo_mode
            );
        }

        if let Some(ref prefs) = editor.preferences {
            println!("Editor Preferences:");
            println!(
                "  Auto-save: {} (interval: {:.0}s)",
                prefs.auto_save_enabled, prefs.auto_save_interval
            );
            println!(
                "  Snap to grid: {} (size: {})",
                prefs.snap_to_grid, prefs.snap_size
            );
            println!("  Rotation snap: {}°", prefs.rotation_snap);
        }
    } else {
        println!("\nNo editor data (runtime scene)");
    }
}

/// Creates a runtime scene without editor data.
fn create_runtime_scene(scene: &SceneDefinition) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Creating Runtime Scene ===");

    // Create a copy without editor data
    let runtime_scene = scene.to_runtime_scene();

    println!("Runtime scene created:");
    println!("  Name: {}", runtime_scene.name);
    println!("  Entities: {}", runtime_scene.entity_count());
    println!("  Has editor data: {}", runtime_scene.has_editor_data());

    // Save runtime scene
    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&runtime_scene)?;

    println!(
        "  Serialized size: {} bytes (vs {} bytes with editor data)",
        ron_string.len(),
        loader.save_to_string(scene)?.len()
    );

    // Verify editor_data is not in the output
    if !ron_string.contains("editor_data") {
        println!("  ✓ Editor data successfully excluded");
    } else {
        println!("  ✗ Warning: editor_data still present in output");
    }

    Ok(())
}
