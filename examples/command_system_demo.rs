//! Demonstration of the editor command system with undo/redo support.
//!
//! This example shows:
//! - Creating and executing various commands
//! - Undo/redo functionality
//! - Composite commands
//! - Command serialization to/from RON format
//! - Integration with the ECS World
//!
//! **Note:** This demo uses `bevy_ecs::world::World` directly because
//! the editor commands are designed to work with bevy_ecs World.
//! See also: `undo_redo_system_demo.rs` for a simpler example.

#[cfg(feature = "editor")]
use bevy_ecs::world::World;
#[cfg(feature = "editor")]
use praxis_ecs::{Name, Transform};
#[cfg(feature = "editor")]
use praxis_editor::{
    AddComponentCommand, CommandHistory, ComponentData, CompositeCommand, CreateEntityCommand,
    EditorCommand, SerializableCommand, TransformEditCommand,
};

#[cfg(feature = "editor")]
fn main() {
    println!("=== Command System Demo ===\n");

    // Create ECS world and command history
    let mut world = World::new();
    let mut history = CommandHistory::new();

    // Demo 1: Transform editing with undo/redo
    println!("--- Demo 1: Transform Editing ---");
    demo_transform_editing(&mut world, &mut history);

    // Demo 2: Entity creation
    println!("\n--- Demo 2: Entity Creation ---");
    demo_entity_creation(&mut world, &mut history);

    // Demo 3: Component management
    println!("\n--- Demo 3: Component Management ---");
    demo_component_management(&mut world, &mut history);

    // Demo 4: Composite commands
    println!("\n--- Demo 4: Composite Commands ---");
    demo_composite_commands(&mut world, &mut history);

    // Demo 5: Command serialization
    println!("\n--- Demo 5: Command Serialization ---");
    demo_serialization(&mut world, &mut history);

    // Demo 6: History management
    println!("\n--- Demo 6: History Management ---");
    demo_history_management(&mut world, &mut history);

    println!("\n=== Demo Complete ===");
}

#[cfg(feature = "editor")]
fn demo_transform_editing(world: &mut World, history: &mut CommandHistory) {
    // Create an entity
    let entity = world.spawn(Transform::default()).id();
    println!("Created entity: {:?}", entity);

    // Edit transform via command
    let command = Box::new(TransformEditCommand::new(
        entity,
        Transform::default(),
        Transform::from_xyz(10.0, 5.0, 3.0),
    ));

    history
        .execute(world, command)
        .expect("Failed to execute command");

    let transform = world.get::<Transform>(entity).unwrap();
    println!(
        "After edit: position = ({}, {}, {})",
        transform.translation.x, transform.translation.y, transform.translation.z
    );

    // Undo the change
    history.undo(world).expect("Failed to undo");
    let transform = world.get::<Transform>(entity).unwrap();
    println!(
        "After undo: position = ({}, {}, {})",
        transform.translation.x, transform.translation.y, transform.translation.z
    );

    // Redo the change
    history.redo(world).expect("Failed to redo");
    let transform = world.get::<Transform>(entity).unwrap();
    println!(
        "After redo: position = ({}, {}, {})",
        transform.translation.x, transform.translation.y, transform.translation.z
    );
}

#[cfg(feature = "editor")]
fn demo_entity_creation(world: &mut World, history: &mut CommandHistory) {
    let initial_count = world.iter_entities().count();
    println!("Initial entity count: {}", initial_count);

    // Create entity via command
    let command = Box::new(CreateEntityCommand::with_transform(Transform::from_xyz(
        5.0, 0.0, 0.0,
    )));
    history
        .execute(world, command)
        .expect("Failed to create entity");

    let after_create = world.iter_entities().count();
    println!("After create: {} entities", after_create);

    // Undo creation
    history.undo(world).expect("Failed to undo create");
    let after_undo = world.iter_entities().count();
    println!("After undo create: {} entities", after_undo);

    // Redo creation
    history.redo(world).expect("Failed to redo create");
    let after_redo = world.iter_entities().count();
    println!("After redo create: {} entities", after_redo);
}

#[cfg(feature = "editor")]
fn demo_component_management(world: &mut World, history: &mut CommandHistory) {
    let entity = world.spawn(Transform::default()).id();
    println!("Created entity without Name: {:?}", entity);

    // Add Name component via command
    let command = Box::new(AddComponentCommand::new(
        entity,
        ComponentData::Name("TestEntity".to_string()),
    ));
    history
        .execute(world, command)
        .expect("Failed to add component");

    if let Some(name) = world.get::<Name>(entity) {
        println!("After add: name = '{}'", name.0);
    }

    // Undo add
    history.undo(world).expect("Failed to undo add");
    if world.get::<Name>(entity).is_none() {
        println!("After undo: name component removed");
    }

    // Redo add
    history.redo(world).expect("Failed to redo add");
    if let Some(name) = world.get::<Name>(entity) {
        println!("After redo: name = '{}'", name.0);
    }
}

#[cfg(feature = "editor")]
fn demo_composite_commands(world: &mut World, history: &mut CommandHistory) {
    let initial_count = world.iter_entities().count();
    println!("Initial entity count: {}", initial_count);

    // Create composite command
    let mut composite = CompositeCommand::new("Create Three Entities".to_string());

    for i in 0..3 {
        let cmd =
            CreateEntityCommand::with_transform(Transform::from_xyz(i as f32 * 5.0, 0.0, 0.0));
        composite.add_command(SerializableCommand::CreateEntity(cmd));
    }

    println!(
        "Composite command with {} sub-commands",
        composite.commands.len()
    );

    history
        .execute(world, Box::new(composite))
        .expect("Failed to execute composite");

    let after_composite = world.iter_entities().count();
    println!("After composite: {} entities", after_composite);

    // Undo composite (all 3 at once)
    history.undo(world).expect("Failed to undo composite");
    let after_undo = world.iter_entities().count();
    println!("After undo composite: {} entities", after_undo);

    // Redo composite
    history.redo(world).expect("Failed to redo composite");
    let after_redo = world.iter_entities().count();
    println!("After redo composite: {} entities", after_redo);
}

#[cfg(feature = "editor")]
fn demo_serialization(world: &mut World, _history: &mut CommandHistory) {
    let entity = world.spawn(Transform::default()).id();

    let command = TransformEditCommand::new(
        entity,
        Transform::default(),
        Transform::from_xyz(10.0, 5.0, 3.0),
    );

    // Serialize to RON
    let ron_string = command.to_ron().expect("Failed to serialize");
    println!("Serialized command:");
    println!("{}", ron_string);

    // Deserialize from RON
    let deserialized = SerializableCommand::from_ron(&ron_string).expect("Failed to deserialize");
    println!("\nDeserialized successfully");

    // Execute deserialized command
    let mut trait_object = deserialized.to_trait_object();
    trait_object
        .execute(world)
        .expect("Failed to execute deserialized");

    let transform = world.get::<Transform>(entity).unwrap();
    println!(
        "After executing deserialized: ({}, {}, {})",
        transform.translation.x, transform.translation.y, transform.translation.z
    );
}

#[cfg(feature = "editor")]
fn demo_history_management(world: &mut World, history: &mut CommandHistory) {
    // Clear history first
    history.clear();
    println!("History cleared");

    // Execute several commands
    for i in 0..5 {
        let entity = world.spawn(Transform::default()).id();
        let command = Box::new(TransformEditCommand::new(
            entity,
            Transform::default(),
            Transform::from_xyz(i as f32, 0.0, 0.0),
        ));
        history.execute(world, command).expect("Failed to execute");
    }

    println!("After 5 commands:");
    println!("  Can undo: {}", history.can_undo());
    println!("  Can redo: {}", history.can_redo());
    println!("  Undo count: {}", history.undo_count());
    println!("  Redo count: {}", history.redo_count());

    // Undo 2 commands
    history.undo(world).expect("undo 1");
    history.undo(world).expect("undo 2");

    println!("\nAfter 2 undos:");
    println!("  Undo count: {}", history.undo_count());
    println!("  Redo count: {}", history.redo_count());

    if let Some(desc) = history.undo_description() {
        println!("  Next undo: {}", desc);
    }
    if let Some(desc) = history.redo_description() {
        println!("  Next redo: {}", desc);
    }
}

#[cfg(not(feature = "editor"))]
fn main() {
    eprintln!("This example requires the 'editor' feature to be enabled.");
    eprintln!("Run with: cargo run --example command_system_demo --features editor");
    std::process::exit(1);
}
