//! Demonstration of command serialization and deserialization.
//!
//! This example shows:
//! - Serializing individual commands to RON format
//! - Deserializing commands from RON strings
//! - Saving and loading entire command histories
//! - Round-trip serialization testing

#[cfg(feature = "editor")]
use bevy_ecs::world::World;
#[cfg(feature = "editor")]
use praxis_ecs::Transform;
#[cfg(feature = "editor")]
use praxis_editor::{
    CommandHistory, ComponentData, CompositeCommand, CreateEntityCommand, EditorCommand,
    SerializableCommand, TransformEditCommand,
};

#[cfg(feature = "editor")]
fn main() {
    println!("=== Command Serialization Demo ===\n");

    let mut world = World::new();

    demo_single_command_serialization(&mut world);
    demo_composite_serialization(&mut world);
    demo_history_serialization(&mut world);
    demo_round_trip(&mut world);

    println!("\n=== Demo Complete ===");
}

#[cfg(feature = "editor")]
fn demo_single_command_serialization(world: &mut World) {
    println!("--- Single Command Serialization ---");

    // Create a transform edit command
    let entity = world.spawn(Transform::default()).id();
    let command = TransformEditCommand::new(
        entity,
        Transform::default(),
        Transform::from_xyz(10.0, 5.0, 3.0),
    );

    // Serialize to RON
    let ron_string = command.to_ron().expect("Failed to serialize");
    println!("Serialized TransformEditCommand:");
    println!("{}", ron_string);

    // Deserialize from RON
    let deserialized = SerializableCommand::from_ron(&ron_string).expect("Failed to deserialize");
    println!("\nSuccessfully deserialized command");

    // Execute the deserialized command
    let mut trait_object = deserialized.to_trait_object();
    trait_object
        .execute(world)
        .expect("Failed to execute deserialized command");

    if let Some(transform) = world.get::<Transform>(entity) {
        println!(
            "Transform after executing deserialized command: ({}, {}, {})",
            transform.translation.x, transform.translation.y, transform.translation.z
        );
    }
}

#[cfg(feature = "editor")]
fn demo_composite_serialization(world: &mut World) {
    println!("\n--- Composite Command Serialization ---");

    // Create a composite command
    let mut composite = CompositeCommand::new("Create Three Entities".to_string());

    for i in 0..3 {
        let cmd =
            CreateEntityCommand::with_transform(Transform::from_xyz(i as f32 * 5.0, 0.0, 0.0));
        composite.add_command(SerializableCommand::CreateEntity(cmd));
    }

    // Serialize
    let ron_string = composite.to_ron().expect("Failed to serialize composite");
    println!(
        "Serialized CompositeCommand ({} operations):",
        composite.len()
    );
    println!("{}", ron_string);

    // Deserialize
    let deserialized =
        SerializableCommand::from_ron(&ron_string).expect("Failed to deserialize composite");
    println!("\nSuccessfully deserialized composite command");

    // Execute
    let mut trait_object = deserialized.to_trait_object();
    let entity_count_before = world.entities().len();
    trait_object
        .execute(world)
        .expect("Failed to execute composite");
    let entity_count_after = world.entities().len();

    println!(
        "Created {} entities via deserialized composite",
        entity_count_after - entity_count_before
    );
}

#[cfg(feature = "editor")]
fn demo_history_serialization(world: &mut World) {
    println!("\n--- Command History Serialization ---");

    let mut history = CommandHistory::new();

    // Execute several commands
    println!("Executing 5 commands...");
    for i in 0..5 {
        let entity = world.spawn(Transform::default()).id();
        let command = TransformEditCommand::new(
            entity,
            Transform::default(),
            Transform::from_xyz(i as f32, i as f32, i as f32),
        );
        history
            .execute(world, Box::new(command))
            .expect("Failed to execute command");
    }

    println!("History contains {} commands", history.undo_count());

    // Serialize entire history
    let history_ron = history.to_ron().expect("Failed to serialize history");
    println!("\nSerialized history:");
    println!("Length: {} bytes", history_ron.len());
    println!(
        "First 200 chars: {}...",
        &history_ron[..200.min(history_ron.len())]
    );

    // Deserialize into new history
    let mut new_history = CommandHistory::new();
    new_history
        .from_ron(&history_ron)
        .expect("Failed to deserialize history");

    println!(
        "\nDeserialized history contains {} commands",
        new_history.undo_count()
    );

    // Save to file (commented out for safety)
    /*
    std::fs::write("command_history.ron", &history_ron)
        .expect("Failed to write history to file");
    println!("Saved history to command_history.ron");
    */
}

#[cfg(feature = "editor")]
fn demo_round_trip(world: &mut World) {
    println!("\n--- Round-Trip Serialization Test ---");

    // Create various command types
    let entity = world.spawn(Transform::default()).id();

    let commands: Vec<Box<dyn praxis_editor::EditorCommand>> = vec![
        Box::new(TransformEditCommand::new(
            entity,
            Transform::default(),
            Transform::from_xyz(1.0, 2.0, 3.0),
        )),
        Box::new(CreateEntityCommand::with_transform(Transform::from_xyz(
            5.0, 0.0, 0.0,
        ))),
        Box::new(praxis_editor::AddComponentCommand::new(
            entity,
            ComponentData::Name {
                data: "TestEntity".to_string(),
            },
        )),
    ];

    println!("Testing {} different command types", commands.len());

    for (i, command) in commands.iter().enumerate() {
        let description = command.description();
        let type_id = command.type_id();

        // Serialize
        let ron = command.to_ron().expect("Failed to serialize");

        // Deserialize
        let deserialized = SerializableCommand::from_ron(&ron).expect("Failed to deserialize");

        // Convert back to trait object
        let trait_object = deserialized.to_trait_object();

        // Verify type ID matches
        assert_eq!(trait_object.type_id(), type_id);

        println!(
            "  {}: {} ({}) - ✓ Round-trip successful",
            i + 1,
            description,
            type_id
        );
    }

    println!("\nAll round-trip tests passed!");
}

#[cfg(not(feature = "editor"))]
fn main() {
    eprintln!("This example requires the 'editor' feature to be enabled.");
    eprintln!("Run with: cargo run --example command_serialization_demo --features editor");
    std::process::exit(1);
}
