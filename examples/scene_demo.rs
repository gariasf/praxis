//! Scene management system demo.
//!
//! This example demonstrates:
//! - Loading scenes from RON files
//! - Spawning scene entities into the world
//! - Scene graph traversal
//! - Finding entities by name
//! - Unloading scenes

use praxis_ecs::{Name, Query, Transform, World};
use praxis_scene::{
    find_entity_by_name, get_all_children, get_entity_depth, CameraDef, CameraType,
    DirectionalLightDef, EntityDefinition, PointLightDef, SceneDefinition, SceneGraphIterator,
    SceneLoader, SceneManager, TransformDef, TraversalOrder,
};

fn main() -> praxis_utils::Result<()> {
    println!("=== Praxis Scene System Demo ===\n");

    // Initialize the scene system
    praxis_scene::init()?;

    // Create a world and scene manager
    let mut world = World::new();
    let mut scene_manager = SceneManager::new();

    // Demo 1: Create and spawn a simple scene programmatically
    println!("--- Demo 1: Programmatic Scene Creation ---");
    let scene1 = create_simple_scene();
    let handle1 = scene_manager.spawn_scene(&mut world, scene1)?;
    println!("Spawned scene with handle: {}", handle1.id());
    println!("Loaded scenes: {}\n", scene_manager.loaded_scene_count());

    // Demo 2: Query entities in the scene
    println!("--- Demo 2: Querying Scene Entities ---");
    {
        let mut query = world.query::<(&Name, &Transform)>();
        for (name, transform) in query.iter(&world) {
            println!(
                "Entity '{}' at position: ({:.2}, {:.2}, {:.2})",
                name.as_str(),
                transform.translation.x,
                transform.translation.y,
                transform.translation.z
            );
        }
    }
    println!();

    // Demo 3: Find entities by name
    println!("--- Demo 3: Finding Entities by Name ---");
    if let Some(player) = find_entity_by_name(&world, "SimplePlayer", None) {
        println!("Found 'SimplePlayer' entity: {:?}", player);
        let depth = get_entity_depth(&world, player);
        println!("Entity depth: {}", depth);

        let children = get_all_children(&world, player);
        println!("Number of descendants: {}", children.len());
    }
    println!();

    // Demo 4: Scene graph traversal
    println!("--- Demo 4: Scene Graph Traversal ---");
    if let Some(entities) = scene_manager.get_scene_entities(&handle1) {
        if let Some(&root) = entities.first() {
            println!("Traversing from root entity: {:?}", root);
            for (i, entity) in
                SceneGraphIterator::new(&world, root, TraversalOrder::DepthFirst).enumerate()
            {
                let depth = get_entity_depth(&world, entity);
                let indent = "  ".repeat(depth);

                let name = world
                    .get::<Name>(entity)
                    .map(|n| n.as_str())
                    .unwrap_or("<unnamed>");

                println!("{}{}: {:?} - {}", indent, i, entity, name);
            }
        }
    }
    println!();

    // Demo 5: Load scene from RON file
    println!("--- Demo 5: Loading Scene from RON File ---");
    let loader = SceneLoader::new();

    match loader.load_from_file("assets/scenes/example_scene.ron") {
        Ok(scene_def) => {
            println!("Loaded scene: {}", scene_def.name);
            println!("Root entities: {}", scene_def.entity_count());
            println!("Total entities: {}", scene_def.total_entity_count());

            if let Some(ref description) = scene_def.metadata.description {
                println!("Description: {}", description);
            }

            let handle2 = scene_manager.spawn_scene(&mut world, scene_def)?;
            println!("Spawned scene with handle: {}", handle2.id());
            println!(
                "Total loaded scenes: {}\n",
                scene_manager.loaded_scene_count()
            );

            // Query all entities
            println!("All entities in world:");
            let mut query = world.query::<&Name>();
            for name in query.iter(&world) {
                println!("  - {}", name.as_str());
            }
            println!();
        }
        Err(e) => {
            println!("Could not load example_scene.ron: {}", e);
            println!("(This is expected if the file doesn't exist yet)\n");
        }
    }

    // Demo 6: Create and save a scene
    println!("--- Demo 6: Creating and Saving Scene ---");
    let scene_to_save = create_complex_scene();
    println!(
        "Created scene with {} entities",
        scene_to_save.total_entity_count()
    );

    let ron_string = loader.save_to_string(&scene_to_save)?;
    println!("Scene serialized to RON ({} bytes)", ron_string.len());
    println!("First 200 characters:");
    println!("{}\n", &ron_string[..ron_string.len().min(200)]);

    // Demo 7: Unload scenes
    println!("--- Demo 7: Unloading Scenes ---");
    println!(
        "Loaded scenes before unload: {}",
        scene_manager.loaded_scene_count()
    );
    scene_manager.unload_all(&mut world);
    println!(
        "Loaded scenes after unload: {}",
        scene_manager.loaded_scene_count()
    );

    println!("\n=== Demo Complete ===");
    Ok(())
}

fn create_simple_scene() -> SceneDefinition {
    let mut scene = SceneDefinition::new("Simple Scene");

    // Add a parent entity
    let parent = EntityDefinition::new()
        .with_name("SimplePlayer")
        .with_transform(TransformDef::from_translation(0.0, 1.0, 0.0))
        .with_mesh("cube")
        .with_child(
            EntityDefinition::new()
                .with_name("SimpleWeapon")
                .with_transform(TransformDef::from_translation(1.0, 0.0, 0.0))
                .with_mesh("sword"),
        );

    scene.add_entity(parent);

    // Add a camera
    let mut camera_entity = EntityDefinition::new();
    camera_entity.name = Some("SimpleCamera".to_string());
    camera_entity.transform = Some(TransformDef::from_translation(0.0, 5.0, 10.0));
    camera_entity.camera = Some(CameraDef::perspective(
        70.0_f32.to_radians(),
        16.0 / 9.0,
        0.1,
        1000.0,
    ));
    scene.add_entity(camera_entity);

    scene
}

fn create_complex_scene() -> SceneDefinition {
    let mut scene = SceneDefinition::new("Complex Demo Scene");

    scene.metadata.description = Some("A complex scene with multiple entity types".to_string());
    scene.metadata.author = Some("Scene Demo".to_string());
    scene.metadata.version = Some("1.0.0".to_string());
    scene.metadata.tags = vec!["demo".to_string(), "complex".to_string()];

    // Camera
    let mut camera = EntityDefinition::new();
    camera.name = Some("MainCamera".to_string());
    camera.transform = Some(TransformDef::from_translation(0.0, 5.0, 10.0));
    camera.camera = Some(CameraDef::perspective(
        70.0_f32.to_radians(),
        16.0 / 9.0,
        0.1,
        1000.0,
    ));
    scene.add_entity(camera);

    // Directional light (Sun)
    let mut sun = EntityDefinition::new();
    sun.name = Some("Sun".to_string());
    sun.directional_light = Some(DirectionalLightDef::new(
        (0.5, -1.0, 0.3),
        (1.0, 0.95, 0.8),
        1.0,
    ));
    scene.add_entity(sun);

    // Player with child point light
    let player = EntityDefinition::new()
        .with_name("Player")
        .with_transform(TransformDef::from_translation(0.0, 1.0, 0.0))
        .with_mesh("character")
        .with_child(
            EntityDefinition::new()
                .with_name("PlayerLight")
                .with_transform(TransformDef::from_translation(0.0, 0.5, 0.0)),
        );

    let mut player_final = player;
    if let Some(light_child) = player_final.children.first_mut() {
        light_child.point_light = Some(PointLightDef::new((1.0, 0.8, 0.6), 5.0, 10.0));
    }
    scene.add_entity(player_final);

    // Environment
    let ground = EntityDefinition::new()
        .with_name("Ground")
        .with_transform(TransformDef {
            translation: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (10.0, 0.1, 10.0),
        })
        .with_mesh("cube");
    scene.add_entity(ground);

    scene
}
