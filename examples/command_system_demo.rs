//! Demonstration of the editor command system with undo/redo support.
//!
//! This example shows:
//! - Creating and executing various commands
//! - Undo/redo functionality
//! - Composite commands
//! - Command serialization to/from RON format
//! - Integration with the ECS World

use praxis_ecs::{Children, Name, Parent, Transform, World};
use praxis_editor::{
    AddComponentCommand, CommandHistory, ComponentData, CompositeCommand, CreateEntityCommand,
    DeleteEntityCommand, RemoveComponentCommand, SerializableCommand, SetParentCommand,
    TransformEditCommand,
};
use praxis_math::Vec3;

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

    // Demo 4: Hierarchy operations
    println!("\n--- Demo 4: Hierarchy Operations ---");
    demo_hierarchy_operations(&mut world, &mut history);

    // Demo 5: Composite commands
    println!("\n--- Demo 5: Composite Commands ---");
    demo_composite_commands(&mut world, &mut history);

    // Demo 6: Command serialization
    println!("\n--- Demo 6: Command Serialization ---");
    demo_serialization(&mut world, &mut history);

    // Demo 7: History management
    println!("\n--- Demo 7: History Management ---");
    demo_history_management(&mut history);

    println!("\n=== Demo Complete ===");
}

fn demo_transform_editing(world: &mut World, history: &mut CommandHistory) {
    // Create an entity with a transform
    let entity = world.spawn(Transform::default()).id();
    println!("Created entity {:?} at origin", entity);

    // Edit the transform
    let old_transform = Transform::default();
    let new_transform = Transform::from_xyz(10.0, 5.0, 3.0);

    let command = TransformEditCommand::new(entity, old_transform, new_transform);
    history
        .execute(world, Box::new(command))
        .expect("Failed to execute command");

    if let Some(transform) = world.get::<Transform>(entity) {
        println!(
            "After execute: position = ({}, {}, {})",
            transform.translation.x, transform.translation.y, transform.translation.z
        );
    }

    // Undo the change
    history.undo(world).expect("Failed to undo");
    if let Some(transform) = world.get::<Transform>(entity) {
        println!(
            "After undo: position = ({}, {}, {})",
            transform.translation.x, transform.translation.y, transform.translation.z
        );
    }

    // Redo the change
    history.redo(world).expect("Failed to redo");
    if let Some(transform) = world.get::<Transform>(entity) {
        println!(
            "After redo: position = ({}, {}, {})",
            transform.translation.x, transform.translation.y, transform.translation.z
        );
    }

    println!(
        "Command history: {} undo, {} redo",
        history.undo_count(),
        history.redo_count()
    );
}

fn demo_entity_creation(world: &mut World, history: &mut CommandHistory) {
    // Create an entity with a transform
    let command = CreateEntityCommand::with_transform(Transform::from_xyz(5.0, 0.0, 0.0));

    println!("Executing CreateEntityCommand...");
    history
        .execute(world, Box::new(command))
        .expect("Failed to create entity");

    let entity_count = world.entities().len();
    println!("Entity count after creation: {}", entity_count);

    // Undo creation
    println!("Undoing creation...");
    history.undo(world).expect("Failed to undo");

    let entity_count = world.entities().len();
    println!("Entity count after undo: {}", entity_count);

    // Redo creation
    println!("Redoing creation...");
    history.redo(world).expect("Failed to redo");

    let entity_count = world.entities().len();
    println!("Entity count after redo: {}", entity_count);
}

fn demo_component_management(world: &mut World, history: &mut CommandHistory) {
    // Create an entity without components
    let entity = world.spawn_empty().id();
    println!("Created empty entity {:?}", entity);

    // Add a Name component
    let add_command =
        AddComponentCommand::new(entity, ComponentData::Name("TestEntity".to_string()));

    println!("Adding Name component...");
    history
        .execute(world, Box::new(add_command))
        .expect("Failed to add component");

    if let Some(name) = world.get::<Name>(entity) {
        println!("Name after add: {}", name.0);
    }

    // Undo add
    println!("Undoing add...");
    history.undo(world).expect("Failed to undo");

    if world.get::<Name>(entity).is_none() {
        println!("Name component removed after undo");
    }

    // Redo add
    println!("Redoing add...");
    history.redo(world).expect("Failed to redo");

    if let Some(name) = world.get::<Name>(entity) {
        println!("Name after redo: {}", name.0);
    }

    // Now remove the component
    let name_value = world.get::<Name>(entity).unwrap().0.clone();
    let remove_command =
        RemoveComponentCommand::new(entity, ComponentData::Name(name_value.clone()));

    println!("Removing Name component...");
    history
        .execute(world, Box::new(remove_command))
        .expect("Failed to remove component");

    if world.get::<Name>(entity).is_none() {
        println!("Name component removed");
    }

    // Undo removal
    println!("Undoing removal...");
    history.undo(world).expect("Failed to undo");

    if let Some(name) = world.get::<Name>(entity) {
        println!("Name restored after undo: {}", name.0);
    }
}

fn demo_hierarchy_operations(world: &mut World, history: &mut CommandHistory) {
    // Create parent and child entities
    let parent = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();
    let child = world.spawn(Transform::from_xyz(5.0, 0.0, 0.0)).id();

    println!("Created parent {:?} and child {:?}", parent, child);

    // Set parent relationship
    let command = SetParentCommand::from_world(child, Some(parent), world)
        .expect("Failed to create set parent command");

    println!("Setting parent relationship...");
    history
        .execute(world, Box::new(command))
        .expect("Failed to set parent");

    if let Some(parent_comp) = world.get::<Parent>(child) {
        println!("Child now has parent: {:?}", parent_comp.0);
    }

    if let Some(children) = world.get::<Children>(parent) {
        println!("Parent has {} children", children.len());
    }

    // Undo parent relationship
    println!("Undoing parent relationship...");
    history.undo(world).expect("Failed to undo");

    if world.get::<Parent>(child).is_none() {
        println!("Child no longer has parent after undo");
    }

    // Redo parent relationship
    println!("Redoing parent relationship...");
    history.redo(world).expect("Failed to redo");

    if let Some(parent_comp) = world.get::<Parent>(child) {
        println!("Parent relationship restored: {:?}", parent_comp.0);
    }
}

fn demo_composite_commands(world: &mut World, history: &mut CommandHistory) {
    // Create a composite command that creates multiple entities
    let mut composite = CompositeCommand::new("Create Scene".to_string());

    println!("Building composite command to create 5 entities...");

    for i in 0..5 {
        let transform = Transform::from_xyz(i as f32 * 2.0, 0.0, 0.0);
        let cmd = CreateEntityCommand::with_transform(transform);
        composite.add_command(SerializableCommand::CreateEntity(cmd));
    }

    println!("Composite contains {} commands", composite.len());

    // Execute the composite
    let entity_count_before = world.entities().len();
    println!("Entity count before: {}", entity_count_before);

    history
        .execute(world, Box::new(composite))
        .expect("Failed to execute composite");

    let entity_count_after = world.entities().len();
    println!("Entity count after: {}", entity_count_after);
    println!(
        "Created {} entities",
        entity_count_after - entity_count_before
    );

    // Undo the composite (undoes all operations)
    println!("Undoing composite command...");
    history.undo(world).expect("Failed to undo composite");

    let entity_count_undone = world.entities().len();
    println!("Entity count after undo: {}", entity_count_undone);

    // Redo the composite
    println!("Redoing composite command...");
    history.redo(world).expect("Failed to redo composite");

    let entity_count_redone = world.entities().len();
    println!("Entity count after redo: {}", entity_count_redone);
}

fn demo_serialization(world: &mut World, history: &mut CommandHistory) {
    // Create a command and serialize it
    let entity = world.spawn(Transform::default()).id();
    let command = TransformEditCommand::new(
        entity,
        Transform::default(),
        Transform::from_xyz(1.0, 2.0, 3.0),
    );

    println!("Serializing TransformEditCommand to RON...");
    let ron_string = command.to_ron().expect("Failed to serialize");
    println!("RON output:\n{}", ron_string);

    // Deserialize the command
    println!("\nDeserializing from RON...");
    let deserialized = SerializableCommand::from_ron(&ron_string).expect("Failed to deserialize");
    println!("Successfully deserialized command");

    // Execute the deserialized command
    let mut trait_object = deserialized.to_trait_object();
    println!("Executing deserialized command...");
    trait_object
        .execute(world)
        .expect("Failed to execute deserialized command");

    if let Some(transform) = world.get::<Transform>(entity) {
        println!(
            "Transform after deserialized command: ({}, {}, {})",
            transform.translation.x, transform.translation.y, transform.translation.z
        );
    }

    // Serialize entire history
    println!("\nSerializing entire command history...");
    let history_ron = history.to_ron().expect("Failed to serialize history");
    println!("History RON length: {} bytes", history_ron.len());

    // Save to file (commented out to avoid file I/O in example)
    // std::fs::write("command_history.ron", &history_ron).expect("Failed to write file");
    // println!("Saved history to command_history.ron");

    // Load from RON
    println!("Loading history from RON...");
    let mut new_history = CommandHistory::new();
    new_history
        .from_ron(&history_ron)
        .expect("Failed to load history");
    println!("Loaded history with {} commands", new_history.undo_count());
}

fn demo_history_management(history: &mut CommandHistory) {
    println!("Current history state:");
    println!("  Can undo: {}", history.can_undo());
    println!("  Can redo: {}", history.can_redo());
    println!("  Undo stack size: {}", history.undo_count());
    println!("  Redo stack size: {}", history.redo_count());

    if let Some(desc) = history.undo_description() {
        println!("  Next undo: {}", desc);
    }

    if let Some(desc) = history.redo_description() {
        println!("  Next redo: {}", desc);
    }

    // Clear history
    println!("\nClearing history...");
    history.clear();
    println!("  Undo stack size after clear: {}", history.undo_count());
    println!("  Redo stack size after clear: {}", history.redo_count());
}
